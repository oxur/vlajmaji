//! Public CLI coverage for sealed owned-history transport.

use std::{
    ffi::OsString,
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    sync::atomic::{AtomicUsize, Ordering},
};

#[cfg(unix)]
use std::os::fd::OwnedFd;
#[cfg(unix)]
use std::os::unix::ffi::{OsStrExt, OsStringExt};

use serde_json::Value;
#[cfg(unix)]
use sha2::{Digest, Sha256};
use vlajmaji_lexicon::seal_owned_snapshot;
use vlajmaji_lexicon::{
    AbbreviationKind, Availability, CollectionRecord, ComponentRecord, ContentNotice,
    ContributorId, ContributorRecord, Dictionary, DictionaryDocument, DictionaryMetadata, EntityId,
    OwnedHead, OwnedRevisionRecord, Presence, RedistributionStatus, RegistryDocument,
    ReleaseAccounting, ReleaseRecord, SourceId, SourceOrigin, SourceRecord, SourceRegistry,
    TextValue, TimePrecision,
};
use vlajmaji_store::{OwnedJournal, OwnedJournalReader};

static NEXT: AtomicUsize = AtomicUsize::new(0);

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        for _ in 0..128 {
            let path = std::env::temp_dir().join(format!(
                "vlajmaji-owned-snapshot-cli-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            match fs::create_dir(&path) {
                Ok(()) => return Self(path),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => panic!("create exclusive scratch directory: {error}"),
            }
        }
        panic!("could not allocate exclusive scratch directory");
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn genesis() -> Dictionary {
    let source = SourceId::new("snapshot-cli-source").unwrap();
    let editor = ContributorId::new("snapshot-cli-editor").unwrap();
    let collection = EntityId::new("snapshot-cli-collection").unwrap();
    let component = EntityId::new("snapshot-cli-component").unwrap();
    let revision = EntityId::new("snapshot-cli-r0").unwrap();
    let registry = SourceRegistry::from_document(RegistryDocument {
        sources: vec![SourceRecord {
            id: source.clone(),
            display_name: "Snapshot CLI fixture".into(),
            abbreviation: "SCF".into(),
            abbreviation_kind: AbbreviationKind::ProjectAssigned,
            abbreviation_evidence: None,
            origin: SourceOrigin::OwnedDraft,
        }],
        contributors: vec![ContributorRecord {
            id: editor.clone(),
            display_name: "Fixture editor".into(),
        }],
        ..RegistryDocument::default()
    })
    .unwrap();
    let mut metadata = DictionaryMetadata::default();
    metadata.collections.push(CollectionRecord {
        id: collection.clone(),
        source: source.clone(),
        display_name: "Snapshot CLI fixture".into(),
        description: Presence::Missing,
    });
    metadata.components.push(ComponentRecord {
        id: component.clone(),
        collection,
        native_name: "fixture entries".into(),
        languages: vec!["jbo".into()],
    });
    metadata.releases.push(ReleaseRecord {
        id: EntityId::new("snapshot-cli-release").unwrap(),
        component,
        distributor: None,
        upstream_revision: Presence::Missing,
        distributor_revision: Presence::Missing,
        captured_at: Presence::Missing,
        time_precision: TimePrecision::Unknown,
        schema_version: 1,
        normalizer: "test".into(),
        normalizer_version: "1".into(),
        captures: vec![],
        assertions: vec![],
        accounting: ReleaseAccounting {
            input_occurrences: Some(0),
            normalized_occurrences: Some(0),
            query_excluded_occurrences: Some(0),
            unit: "occurrences".into(),
        },
        diagnostics: vec![],
        selected: true,
        availability: Availability::Available,
        content_notices: vec![ContentNotice {
            text: Presence::Value("synthetic test; no licence grant".into()),
            locator: Presence::Missing,
            license: Presence::Missing,
        }],
        redistribution: RedistributionStatus::Held,
        redistribution_evidence: vec![],
    });
    metadata.owned_revisions.push(OwnedRevisionRecord {
        id: revision.clone(),
        source: source.clone(),
        parent: None,
        editor,
        timestamp: Presence::Missing,
        time_precision: TimePrecision::Unknown,
        assertions: vec![],
        tombstones: vec![],
        rationale: Presence::Value(TextValue {
            text: "genesis".into(),
            language: Presence::Missing,
            markup: Presence::Missing,
        }),
    });
    metadata.owned_heads.push(OwnedHead { source, revision });
    Dictionary::from_document(DictionaryDocument {
        registry,
        metadata,
        ..DictionaryDocument::default()
    })
    .unwrap()
}

fn next_revision(current: &Dictionary) -> Dictionary {
    let mut document = current.document();
    let previous = document.metadata.owned_revisions.last().unwrap().clone();
    let revision = EntityId::new("snapshot-cli-r1").unwrap();
    document.metadata.catalogue_revision += 1;
    document.metadata.owned_revisions.push(OwnedRevisionRecord {
        id: revision.clone(),
        source: previous.source.clone(),
        parent: Some(previous.id),
        editor: previous.editor,
        timestamp: Presence::Missing,
        time_precision: TimePrecision::Unknown,
        assertions: vec![],
        tombstones: vec![],
        rationale: Presence::Value(TextValue {
            text: "second revision".into(),
            language: Presence::Missing,
            markup: Presence::Missing,
        }),
    });
    document.metadata.owned_heads[0].revision = revision;
    Dictionary::from_document(document).unwrap()
}

fn run(args: &[&str], scratch: &Scratch) -> Output {
    Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
        .args(args)
        .env("HOME", scratch.path().join("home"))
        .env("XDG_DATA_HOME", scratch.path().join("data"))
        .env("XDG_CONFIG_HOME", scratch.path().join("config"))
        .env("TMPDIR", scratch.path())
        .stdin(Stdio::null())
        .output()
        .unwrap()
}

#[cfg(unix)]
fn run_with_stdout(args: &[&str], scratch: &Scratch, stdout: Stdio) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_vlajmaji"));
    command
        .args(args)
        .env("HOME", scratch.path().join("home"))
        .env("XDG_DATA_HOME", scratch.path().join("data"))
        .env("XDG_CONFIG_HOME", scratch.path().join("config"))
        .env("TMPDIR", scratch.path())
        .stdin(Stdio::null())
        .stdout(stdout)
        .stderr(Stdio::piped());
    command.spawn().unwrap().wait_with_output().unwrap()
}

#[cfg(unix)]
fn run_native(args: &[OsString], scratch: &Scratch) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_vlajmaji"));
    command
        .args(args)
        .env("HOME", scratch.path().join("home"))
        .env("XDG_DATA_HOME", scratch.path().join("data"))
        .env("XDG_CONFIG_HOME", scratch.path().join("config"))
        .env("TMPDIR", scratch.path())
        .stdin(Stdio::null())
        .output()
        .unwrap()
}

#[cfg(unix)]
fn expected_native_path(path: &Path) -> Value {
    let bytes = path.as_os_str().as_bytes();
    serde_json::json!({
        "encoding": "unix-bytes",
        "hex": bytes.iter().map(|byte| format!("{byte:02x}")).collect::<String>(),
        "utf8": path.to_str(),
    })
}

#[cfg(unix)]
fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[cfg(unix)]
fn exact_json_report(output: &Output) -> Value {
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    assert_eq!(output.stdout.last(), Some(&b'\n'));
    assert_ne!(
        output.stdout.get(output.stdout.len().saturating_sub(2)),
        Some(&b'\n')
    );
    json(output)
}

#[cfg(unix)]
fn exercise_native_snapshot_transport(root: &Path, scratch: &Scratch, suffix: &[u8]) {
    let input = scratch.path().join(OsString::from_vec(
        [b"native-input-".as_slice(), suffix, b".json"].concat(),
    ));
    let seal_path = scratch.path().join(OsString::from_vec(
        [b"native-seal-".as_slice(), suffix, b".json"].concat(),
    ));
    let model = scratch.path().join(OsString::from_vec(
        [b"native-model-".as_slice(), suffix, b".json"].concat(),
    ));
    let imported = scratch.path().join(OsString::from_vec(
        [b"native-import-".as_slice(), suffix].concat(),
    ));

    let expected = OwnedJournalReader::open(root).unwrap().snapshot().unwrap();
    let sealed = seal_owned_snapshot(&expected).unwrap();
    let expected_model = expected.dictionary_at(&expected.as_data().head).unwrap();
    let expected_model_bytes = expected_model.to_json_bytes().unwrap();

    let export = run_native(
        &[
            "owned".into(),
            "export".into(),
            root.as_os_str().to_os_string(),
            "--output".into(),
            input.as_os_str().to_os_string(),
            "--seal".into(),
            seal_path.as_os_str().to_os_string(),
        ],
        scratch,
    );
    let export_report = exact_json_report(&export);
    assert_eq!(
        export_report,
        serde_json::json!({
            "version": 1,
            "source": expected.as_data().source.as_str(),
            "head": expected.as_data().head.as_str(),
            "journal_schema": expected.as_data().journal_schema,
            "revision_count": expected.as_data().commits.len(),
            "output": expected_native_path(&input),
            "output_bytes": sealed.snapshot.len(),
            "output_sha256": sha256_hex(&sealed.snapshot),
            "seal": expected_native_path(&seal_path),
            "seal_bytes": sealed.seal.len(),
            "seal_sha256": sha256_hex(&sealed.seal),
        })
    );
    assert_eq!(fs::read(&input).unwrap(), sealed.snapshot);
    assert_eq!(fs::read(&seal_path).unwrap(), sealed.seal);

    let check = run_native(
        &[
            "owned".into(),
            "check".into(),
            "--input".into(),
            input.as_os_str().to_os_string(),
            "--seal".into(),
            seal_path.as_os_str().to_os_string(),
        ],
        scratch,
    );
    let check_report = exact_json_report(&check);
    assert_eq!(
        check_report,
        serde_json::json!({
            "version": 1,
            "source": expected.as_data().source.as_str(),
            "head": expected.as_data().head.as_str(),
            "journal_schema": expected.as_data().journal_schema,
            "revision_count": expected.as_data().commits.len(),
            "snapshot_bytes": sealed.snapshot.len(),
            "snapshot_sha256": sha256_hex(&sealed.snapshot),
            "seal_bytes": sealed.seal.len(),
            "seal_sha256": sha256_hex(&sealed.seal),
        })
    );

    let snapshot_model = run_native(
        &[
            "owned".into(),
            "snapshot-model".into(),
            "--input".into(),
            input.as_os_str().to_os_string(),
            "--seal".into(),
            seal_path.as_os_str().to_os_string(),
            "--revision".into(),
            expected.as_data().head.as_str().into(),
            "--output".into(),
            model.as_os_str().to_os_string(),
        ],
        scratch,
    );
    let model_report = exact_json_report(&snapshot_model);
    assert_eq!(
        model_report,
        serde_json::json!({
            "version": 1,
            "source": expected.as_data().source.as_str(),
            "revision": expected.as_data().head.as_str(),
            "journal_schema": expected.as_data().journal_schema,
            "output": expected_native_path(&model),
            "output_bytes": expected_model_bytes.len(),
            "output_sha256": sha256_hex(&expected_model_bytes),
        })
    );
    assert_eq!(fs::read(&model).unwrap(), expected_model_bytes);

    let import = run_native(
        &[
            "owned".into(),
            "import".into(),
            imported.as_os_str().to_os_string(),
            "--input".into(),
            input.as_os_str().to_os_string(),
            "--seal".into(),
            seal_path.as_os_str().to_os_string(),
        ],
        scratch,
    );
    let import_report = exact_json_report(&import);
    assert_eq!(
        import_report,
        serde_json::json!({
            "version": 1,
            "outcome": "imported",
            "source": expected.as_data().source.as_str(),
            "head": expected.as_data().head.as_str(),
            "journal_schema": expected.as_data().journal_schema,
            "revision_count": expected.as_data().commits.len(),
            "snapshot_sha256": sha256_hex(&sealed.snapshot),
        })
    );
    assert_eq!(
        OwnedJournalReader::open(&imported)
            .unwrap()
            .snapshot()
            .unwrap(),
        expected
    );
}

fn write_new(path: &Path, bytes: &[u8]) {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .unwrap();
    file.write_all(bytes).unwrap();
    file.sync_all().unwrap();
}

fn json(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
        panic!(
            "expected JSON stdout: {error}; stderr={}",
            String::from_utf8_lossy(&output.stderr)
        )
    })
}

fn error_json(output: &Output) -> Value {
    serde_json::from_slice(&output.stderr).unwrap_or_else(|error| {
        panic!(
            "expected structured error stderr: {error}; stdout={}",
            String::from_utf8_lossy(&output.stdout)
        )
    })
}

#[test]
fn export_check_import_replay_and_historical_model_are_explicit_and_rooted() {
    let scratch = Scratch::new();
    let root = scratch.path().join("journal");
    let imported_root = scratch.path().join("imported");
    let payload = scratch.path().join("snapshot.json");
    let seal = scratch.path().join("snapshot.seal.json");
    let r0_model = scratch.path().join("r0.json");
    let head_model = scratch.path().join("head.json");
    let original = genesis();
    let r0 = original.metadata().owned_revisions[0].id.clone();
    let r1 = next_revision(&original);
    let r1_id = r1.metadata().owned_revisions[1].id.clone();
    let (journal, _) = OwnedJournal::create(&root, &original).unwrap();
    drop(journal);
    let mut journal = OwnedJournal::open(&root).unwrap();
    journal.commit(&r0, &r1).unwrap();
    drop(journal);

    let exported = run(
        &[
            "owned",
            "export",
            root.to_str().unwrap(),
            "--output",
            payload.to_str().unwrap(),
            "--seal",
            seal.to_str().unwrap(),
        ],
        &scratch,
    );
    assert!(
        exported.status.success(),
        "{}",
        String::from_utf8_lossy(&exported.stderr)
    );
    let export_report = json(&exported);
    assert_eq!(
        export_report
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        [
            "head",
            "journal_schema",
            "output",
            "output_bytes",
            "output_sha256",
            "revision_count",
            "seal",
            "seal_bytes",
            "seal_sha256",
            "source",
            "version",
        ]
    );
    assert_eq!(export_report["source"], "snapshot-cli-source");
    assert_eq!(export_report["head"], "snapshot-cli-r1");
    assert_eq!(export_report["journal_schema"], 1);
    assert_eq!(export_report["revision_count"], 2);

    let uncreated_seal = scratch.path().join("must-not-be-created.seal.json");
    let existing_output = run(
        &[
            "owned",
            "export",
            root.to_str().unwrap(),
            "--output",
            payload.to_str().unwrap(),
            "--seal",
            uncreated_seal.to_str().unwrap(),
        ],
        &scratch,
    );
    assert!(!existing_output.status.success());
    assert_eq!(error_json(&existing_output)["error"]["code"], "owned_io");
    assert!(!uncreated_seal.exists());

    let uncreated_output = scratch.path().join("must-not-be-created.json");
    let existing_seal = scratch.path().join("already-exists.seal.json");
    write_new(&existing_seal, b"occupied");
    let existing_seal_result = run(
        &[
            "owned",
            "export",
            root.to_str().unwrap(),
            "--output",
            uncreated_output.to_str().unwrap(),
            "--seal",
            existing_seal.to_str().unwrap(),
        ],
        &scratch,
    );
    assert!(!existing_seal_result.status.success());
    assert_eq!(
        error_json(&existing_seal_result)["error"]["code"],
        "owned_io"
    );
    assert!(!uncreated_output.exists());
    assert_eq!(fs::read(&existing_seal).unwrap(), b"occupied");

    let first_open_output = scratch.path().join("partial-pair-output.json");
    let second_open_seal = scratch
        .path()
        .join("missing-parent")
        .join("partial-pair-seal.json");
    let second_open = run(
        &[
            "owned",
            "export",
            root.to_str().unwrap(),
            "--output",
            first_open_output.to_str().unwrap(),
            "--seal",
            second_open_seal.to_str().unwrap(),
        ],
        &scratch,
    );
    assert!(!second_open.status.success());
    assert_eq!(error_json(&second_open)["error"]["code"], "owned_io");
    assert!(first_open_output.exists());
    assert!(fs::read(&first_open_output).unwrap().is_empty());
    assert!(!second_open_seal.exists());

    let checked = run(
        &[
            "owned",
            "check",
            "--input",
            payload.to_str().unwrap(),
            "--seal",
            seal.to_str().unwrap(),
        ],
        &scratch,
    );
    assert!(
        checked.status.success(),
        "{}",
        String::from_utf8_lossy(&checked.stderr)
    );
    let check_report = json(&checked);
    assert!(check_report.get("root_path").is_none());
    assert_eq!(
        check_report["snapshot_bytes"],
        export_report["output_bytes"]
    );
    assert_eq!(
        check_report["snapshot_sha256"],
        export_report["output_sha256"]
    );
    assert_eq!(check_report["seal_bytes"], export_report["seal_bytes"]);
    assert_eq!(check_report["seal_sha256"], export_report["seal_sha256"]);
    assert_eq!(check_report["revision_count"], 2);

    let imported = run(
        &[
            "owned",
            "import",
            imported_root.to_str().unwrap(),
            "--input",
            payload.to_str().unwrap(),
            "--seal",
            seal.to_str().unwrap(),
        ],
        &scratch,
    );
    assert!(
        imported.status.success(),
        "{}",
        String::from_utf8_lossy(&imported.stderr)
    );
    assert_eq!(json(&imported)["outcome"], "imported");
    let replay = run(
        &[
            "owned",
            "import",
            imported_root.to_str().unwrap(),
            "--input",
            payload.to_str().unwrap(),
            "--seal",
            seal.to_str().unwrap(),
        ],
        &scratch,
    );
    assert!(
        replay.status.success(),
        "{}",
        String::from_utf8_lossy(&replay.stderr)
    );
    assert_eq!(json(&replay)["outcome"], "already_present");
    assert_eq!(
        OwnedJournalReader::open(&imported_root)
            .unwrap()
            .show(&r1_id)
            .unwrap()
            .dictionary,
        r1
    );

    let old_model = run(
        &[
            "owned",
            "snapshot-model",
            "--input",
            payload.to_str().unwrap(),
            "--seal",
            seal.to_str().unwrap(),
            "--revision",
            r0.as_str(),
            "--output",
            r0_model.to_str().unwrap(),
        ],
        &scratch,
    );
    assert!(
        old_model.status.success(),
        "{}",
        String::from_utf8_lossy(&old_model.stderr)
    );
    let old_report = json(&old_model);
    assert!(old_report.get("root_path").is_none());
    assert_eq!(old_report["revision"], r0.as_str());
    assert_eq!(
        fs::read(&r0_model).unwrap(),
        original.to_json_bytes().unwrap()
    );

    let current_model = run(
        &[
            "owned",
            "snapshot-model",
            "--input",
            payload.to_str().unwrap(),
            "--seal",
            seal.to_str().unwrap(),
            "--output",
            head_model.to_str().unwrap(),
        ],
        &scratch,
    );
    assert!(
        current_model.status.success(),
        "{}",
        String::from_utf8_lossy(&current_model.stderr)
    );
    assert_eq!(json(&current_model)["revision"], r1_id.as_str());
    assert_eq!(fs::read(&head_model).unwrap(), r1.to_json_bytes().unwrap());

    let missing_model = scratch.path().join("missing.json");
    let missing = run(
        &[
            "owned",
            "snapshot-model",
            "--input",
            payload.to_str().unwrap(),
            "--seal",
            seal.to_str().unwrap(),
            "--revision",
            "snapshot-cli-missing",
            "--output",
            missing_model.to_str().unwrap(),
        ],
        &scratch,
    );
    assert!(!missing.status.success());
    assert_eq!(missing.status.code(), Some(1));
    assert!(missing.stdout.is_empty());
    assert_eq!(
        error_json(&missing)["error"]["code"],
        "owned_invalid_snapshot"
    );
    assert!(!missing_model.exists());

    let invalid_root = scratch.path().join("not-created-before-strict-load");
    let invalid_seal = scratch.path().join("invalid-seal.json");
    write_new(&invalid_seal, br#"{"version":1,"kind":"wrong"}"#);
    let invalid_import = run(
        &[
            "owned",
            "import",
            invalid_root.to_str().unwrap(),
            "--input",
            payload.to_str().unwrap(),
            "--seal",
            invalid_seal.to_str().unwrap(),
        ],
        &scratch,
    );
    assert!(!invalid_import.status.success());
    assert_eq!(invalid_import.status.code(), Some(1));
    assert!(invalid_import.stdout.is_empty());
    assert_eq!(
        error_json(&invalid_import)["error"]["code"],
        "owned_invalid_snapshot"
    );
    assert!(!invalid_root.exists());

    assert_eq!(
        fs::read(&payload).unwrap(),
        OwnedJournalReader::open(&root)
            .unwrap()
            .snapshot()
            .unwrap()
            .to_json_bytes()
            .unwrap()
    );
}

#[cfg(unix)]
#[test]
fn import_commits_before_stdout_other_or_broken_pipe() {
    use std::os::unix::net::{UnixDatagram, UnixStream};

    let scratch = Scratch::new();
    let root = scratch.path().join("journal");
    let genesis = genesis();
    let (writer, _) = OwnedJournal::create(&root, &genesis).unwrap();
    drop(writer);
    let expected = OwnedJournalReader::open(&root).unwrap().snapshot().unwrap();
    let sealed = seal_owned_snapshot(&expected).unwrap();
    let input = scratch.path().join("post-commit-snapshot.json");
    let seal = scratch.path().join("post-commit-snapshot.seal.json");
    write_new(&input, &sealed.snapshot);
    write_new(&seal, &sealed.seal);

    let other_root = scratch.path().join("import-other-root");
    let (sender, _unread_peer) = UnixDatagram::pair().unwrap();
    sender.set_nonblocking(true).unwrap();
    let mut full = false;
    for _ in 0..16_384 {
        match sender.send(b"x") {
            Ok(_) => {}
            Err(error)
                if error.kind() == std::io::ErrorKind::WouldBlock
                    || (cfg!(target_os = "macos") && error.raw_os_error() == Some(55)) =>
            {
                full = true;
                break;
            }
            Err(error) => panic!("fill bounded stdout datagram queue: {error}"),
        }
    }
    assert!(full, "bounded stdout datagram queue did not saturate");
    let sender_fd: OwnedFd = sender.into();
    let mut sender_file = File::from(sender_fd);
    let write_error = sender_file
        .write(b"x")
        .expect_err("one-byte stdout preflight must fail before child launch");
    assert!(
        write_error.kind() == std::io::ErrorKind::WouldBlock
            || (cfg!(target_os = "macos") && write_error.raw_os_error() == Some(55)),
        "unexpected full stdout error: {write_error}"
    );
    let other = run_with_stdout(
        &[
            "owned",
            "import",
            other_root.to_str().unwrap(),
            "--input",
            input.to_str().unwrap(),
            "--seal",
            seal.to_str().unwrap(),
        ],
        &scratch,
        Stdio::from(sender_file),
    );
    assert_eq!(other.status.code(), Some(1));
    assert!(other.stdout.is_empty());
    assert_eq!(error_json(&other)["error"]["code"], "response_output_io");
    assert_eq!(
        OwnedJournalReader::open(&other_root)
            .unwrap()
            .snapshot()
            .unwrap(),
        expected
    );

    let broken_root = scratch.path().join("import-broken-pipe-root");
    let (sender, peer) = UnixStream::pair().unwrap();
    drop(peer);
    let sender_fd: OwnedFd = sender.into();
    let mut sender_file = File::from(sender_fd);
    assert_eq!(
        sender_file.write(b"x").unwrap_err().kind(),
        std::io::ErrorKind::BrokenPipe
    );
    let broken = run_with_stdout(
        &[
            "owned",
            "import",
            broken_root.to_str().unwrap(),
            "--input",
            input.to_str().unwrap(),
            "--seal",
            seal.to_str().unwrap(),
        ],
        &scratch,
        Stdio::from(sender_file),
    );
    assert_eq!(broken.status.code(), Some(0));
    assert!(broken.stdout.is_empty());
    assert!(broken.stderr.is_empty());
    assert_eq!(
        OwnedJournalReader::open(&broken_root)
            .unwrap()
            .snapshot()
            .unwrap(),
        expected
    );
}

#[cfg(unix)]
#[test]
fn explicit_native_root_handles_unicode_space_and_raw_os_strings() {
    let scratch = Scratch::new();
    let unicode_root = scratch.path().join("owned ü dictionary");
    let (writer, _) = OwnedJournal::create(&unicode_root, &genesis()).unwrap();
    drop(writer);
    let unicode = run(&["owned", "show", unicode_root.to_str().unwrap()], &scratch);
    assert!(unicode.status.success());
    assert_eq!(json(&unicode)["source"], "snapshot-cli-source");
    assert_eq!(
        json(&unicode)["commit"]["revision"]["id"],
        "snapshot-cli-r0"
    );
    exercise_native_snapshot_transport(&unicode_root, &scratch, "ü space".as_bytes());

    let valid_snapshot = genesis();
    let valid_journal_root = scratch.path().join("valid-input-journal");
    let (writer, _) = OwnedJournal::create(&valid_journal_root, &valid_snapshot).unwrap();
    drop(writer);
    let valid_snapshot = OwnedJournalReader::open(&valid_journal_root)
        .unwrap()
        .snapshot()
        .unwrap();
    let valid_sealed = seal_owned_snapshot(&valid_snapshot).unwrap();
    let valid_input = scratch.path().join("valid-snapshot.json");
    let valid_seal = scratch.path().join("valid-snapshot.seal.json");
    write_new(&valid_input, &valid_sealed.snapshot);
    write_new(&valid_seal, &valid_sealed.seal);
    let raw_input = scratch
        .path()
        .join(OsString::from_vec(b"invalid-\xff-input.json".to_vec()));
    let raw_file_error = run_native(
        &[
            OsString::from("owned"),
            OsString::from("check"),
            OsString::from("--input"),
            raw_input.as_os_str().to_os_string(),
            OsString::from("--seal"),
            valid_seal.as_os_str().to_os_string(),
        ],
        &scratch,
    );
    assert_eq!(raw_file_error.status.code(), Some(1));
    assert!(raw_file_error.stdout.is_empty());
    assert_eq!(error_json(&raw_file_error)["error"]["code"], "owned_io");

    let raw_root = scratch
        .path()
        .join(OsString::from_vec(b"owned-\xff-root".to_vec()));
    match fs::create_dir(&raw_root) {
        Ok(()) => {
            eprintln!("raw-byte root creation is supported on this Unix host");
            fs::remove_dir(&raw_root).unwrap();
            let (writer, _) = OwnedJournal::create(&raw_root, &genesis()).unwrap();
            drop(writer);
            let raw = run_native(
                &[
                    OsString::from("owned"),
                    OsString::from("show"),
                    raw_root.as_os_str().to_os_string(),
                ],
                &scratch,
            );
            assert!(raw.status.success());
            let report = json(&raw);
            assert_eq!(report["source"], "snapshot-cli-source");
            assert_eq!(report["commit"]["revision"]["id"], "snapshot-cli-r0");
            exercise_native_snapshot_transport(&raw_root, &scratch, b"raw-\xff");
        }
        Err(error) if cfg!(target_os = "macos") && error.raw_os_error() == Some(92) => {
            eprintln!("macOS raw-byte root creation returned EILSEQ 92: {error}");
            let negative = run_native(
                &[
                    OsString::from("owned"),
                    OsString::from("import"),
                    raw_root.as_os_str().to_os_string(),
                    OsString::from("--input"),
                    valid_input.as_os_str().to_os_string(),
                    OsString::from("--seal"),
                    valid_seal.as_os_str().to_os_string(),
                ],
                &scratch,
            );
            assert_eq!(negative.status.code(), Some(1));
            assert!(negative.stdout.is_empty());
            assert_eq!(error_json(&negative)["error"]["code"], "owned_io");
            assert!(!raw_root.exists());
        }
        Err(error) => panic!("create raw-byte native root: {error}"),
    }
}
