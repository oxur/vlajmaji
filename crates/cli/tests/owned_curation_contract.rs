//! Public CLI coverage for the explicit owned-curation lifecycle.

use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    sync::atomic::{AtomicUsize, Ordering},
};

#[cfg(unix)]
use std::ffi::OsString;

use serde_json::Value;
use sha2::{Digest, Sha256};
use vlajmaji_lexicon::{
    AbbreviationKind, Availability, CatalogTarget, CollectionRecord, ComponentRecord,
    ContentNotice, ContributorId, ContributorRecord, Dictionary, DictionaryDocument,
    DictionaryMetadata, Direction, EntityId, EntryRecord, LocalRef, OccurrenceRef,
    OwnedContextInput, OwnedContextPin, OwnedCurationData, OwnedCurationDocument,
    OwnedCurationKind, OwnedCurationRecord, OwnedCurationRevision, OwnedHead, OwnedPinnedTarget,
    OwnedRevisionRecord, Presence, RedistributionStatus, RegistryDocument, ReleaseAccounting,
    ReleaseRecord, SourceId, SourceOrigin, SourceRecord, SourceRegistry, TextValue, TimePrecision,
};

#[path = "../../lexicon/examples/support/dictionary_fixture.rs"]
mod dictionary_fixture;

static NEXT: AtomicUsize = AtomicUsize::new(0);

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        for _ in 0..128 {
            let path = std::env::temp_dir().join(format!(
                "vlajmaji-owned-curation-cli-{}-{}",
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
    let source = SourceId::new("curation-cli-source").unwrap();
    let editor = ContributorId::new("curation-cli-editor").unwrap();
    let collection = EntityId::new("curation-cli-collection").unwrap();
    let component = EntityId::new("curation-cli-component").unwrap();
    let revision = EntityId::new("curation-cli-r0").unwrap();
    let registry = SourceRegistry::from_document(RegistryDocument {
        sources: vec![SourceRecord {
            id: source.clone(),
            display_name: "Curation CLI fixture".into(),
            abbreviation: "CCF".into(),
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
    metadata.entries.push(EntryRecord {
        id: EntityId::new("curation-cli-entry").unwrap(),
        source: source.clone(),
        spelling: Presence::Value("curated".into()),
        language: Presence::Missing,
        script: Presence::Missing,
        native_word_class: Presence::Missing,
        selmaho: Presence::Missing,
        mapped_pos: Presence::Missing,
    });
    metadata.collections.push(CollectionRecord {
        id: collection.clone(),
        source: source.clone(),
        display_name: "Curation CLI fixture".into(),
        description: Presence::Missing,
    });
    metadata.components.push(ComponentRecord {
        id: component.clone(),
        collection,
        native_name: "fixture entries".into(),
        languages: vec!["jbo".into()],
    });
    metadata.releases.push(ReleaseRecord {
        id: EntityId::new("curation-cli-release").unwrap(),
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
    let revision = EntityId::new("curation-cli-r1").unwrap();
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
            text: "curated edit".into(),
            language: Presence::Missing,
            markup: Presence::Missing,
        }),
    });
    document.metadata.owned_heads[0].revision = revision;
    Dictionary::from_document(document).unwrap()
}

fn external_target(context: &Dictionary) -> (OwnedContextPin, OwnedPinnedTarget) {
    let assertion = &context.metadata().assertions[0];
    let entry = context
        .metadata()
        .entries
        .iter()
        .find(|row| row.id == assertion.entry)
        .unwrap();
    let release = context
        .metadata()
        .releases
        .iter()
        .find(|row| row.assertions.contains(&assertion.id))
        .unwrap();
    let component = context
        .metadata()
        .components
        .iter()
        .find(|row| row.id == release.component)
        .unwrap();
    let collection = context
        .metadata()
        .collections
        .iter()
        .find(|row| row.id == component.collection)
        .unwrap();
    let occurrence_identity = assertion.occurrence.to_record_identity().unwrap();
    let attribution = context
        .registry()
        .as_document()
        .occurrences
        .iter()
        .find(|row| row.occurrence == occurrence_identity)
        .unwrap();
    let pin =
        OwnedContextPin::from_dictionary(EntityId::new("curation-cli-context").unwrap(), context)
            .unwrap();
    (
        pin.clone(),
        OwnedPinnedTarget {
            context: pin.id,
            target: CatalogTarget {
                source: collection.source.clone(),
                release: release.id.clone(),
                native_entry_id: attribution.native_entry_id.clone(),
                native_assertion_id: attribution.native_assertion_id.clone(),
                occurrence: Some(OccurrenceRef::from(&occurrence_identity)),
            },
            entry: entry.id.clone(),
            assertion: assertion.id.clone(),
        },
    )
}

fn expected_detail_reference(
    context: &Dictionary,
    target: &OwnedPinnedTarget,
) -> vlajmaji_search::DictionaryHitDetailRef {
    let entry = context
        .metadata()
        .entries
        .iter()
        .find(|row| row.id == target.entry)
        .unwrap();
    let assertion = context
        .metadata()
        .assertions
        .iter()
        .find(|row| row.id == target.assertion)
        .unwrap();
    let occurrence_identity = assertion.occurrence.to_record_identity().unwrap();
    let attribution = context
        .registry()
        .as_document()
        .occurrences
        .iter()
        .find(|row| row.occurrence == occurrence_identity)
        .unwrap();
    let snapshot_identity = assertion.occurrence.capture.to_snapshot_identity().unwrap();
    let snapshot = context
        .registry()
        .as_document()
        .snapshots
        .iter()
        .find(|row| row.snapshot == snapshot_identity)
        .unwrap();
    let entry_digest = format!("{:x}", Sha256::digest(serde_json::to_vec(entry).unwrap()));
    let assertion_digest = format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(assertion).unwrap())
    );
    vlajmaji_search::DictionaryHitDetailRef {
        version: 1,
        digest_policy: "entry-assertion-serde-json-v1".into(),
        role: "assertion".into(),
        source: target.target.source.clone(),
        entry_id: target.entry.clone(),
        assertion_id: target.assertion.clone(),
        occurrence: target.target.occurrence.clone().unwrap(),
        entry_sha256: entry_digest,
        assertion_sha256: assertion_digest,
        native_entry_id: target.target.native_entry_id.clone(),
        native_assertion_id: target.target.native_assertion_id.clone(),
        contributor_id: attribution.contributor.clone(),
        distributor_id: snapshot.distributor.clone(),
    }
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

fn run_with_stdin(args: &[&str], scratch: &Scratch, input: &[u8]) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
        .args(args)
        .env("HOME", scratch.path().join("home"))
        .env("XDG_DATA_HOME", scratch.path().join("data"))
        .env("XDG_CONFIG_HOME", scratch.path().join("config"))
        .env("TMPDIR", scratch.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(input).unwrap();
    child.wait_with_output().unwrap()
}

#[cfg(unix)]
fn run_with_stdin_and_stdout(
    args: &[&str],
    scratch: &Scratch,
    input: &[u8],
    stdout: Stdio,
) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
        .args(args)
        .env("HOME", scratch.path().join("home"))
        .env("XDG_DATA_HOME", scratch.path().join("data"))
        .env("XDG_CONFIG_HOME", scratch.path().join("config"))
        .env("TMPDIR", scratch.path())
        .stdin(Stdio::piped())
        .stdout(stdout)
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(input).unwrap();
    child.wait_with_output().unwrap()
}

#[cfg(unix)]
fn run_with_stdout(args: &[&str], scratch: &Scratch, stdout: Stdio) -> Output {
    Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
        .args(args)
        .env("HOME", scratch.path().join("home"))
        .env("XDG_DATA_HOME", scratch.path().join("data"))
        .env("XDG_CONFIG_HOME", scratch.path().join("config"))
        .env("TMPDIR", scratch.path())
        .stdin(Stdio::null())
        .stdout(stdout)
        .stderr(Stdio::piped())
        .output()
        .unwrap()
}

#[cfg(unix)]
fn run_native(args: &[OsString], scratch: &Scratch) -> Output {
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
fn run_native_with_stdin(args: &[OsString], scratch: &Scratch, input: &[u8]) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
        .args(args)
        .env("HOME", scratch.path().join("home"))
        .env("XDG_DATA_HOME", scratch.path().join("data"))
        .env("XDG_CONFIG_HOME", scratch.path().join("config"))
        .env("TMPDIR", scratch.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(input).unwrap();
    child.wait_with_output().unwrap()
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

#[test]
fn cli_requires_explicit_migration_and_materializes_to_a_new_file() {
    let scratch = Scratch::new();
    let root = scratch.path().join("journal");
    let seed_path = scratch.path().join("seed.json");
    let curation_path = scratch.path().join("curation.json");
    let context_path = scratch.path().join("context.json");
    let output_path = scratch.path().join("materialized.json");
    let resolved_path = scratch.path().join("resolved.json");
    let wrong_context_path = scratch.path().join("wrong-context.json");
    let rejected_output_path = scratch.path().join("rejected-materialization.json");
    let dictionary = genesis();
    let dictionary_bytes = dictionary.to_json_bytes().unwrap();
    write_new(&seed_path, &dictionary_bytes);
    let base_revision = dictionary.metadata().owned_revisions[0].id.clone();
    let candidate = next_revision(&dictionary);
    let mut context_document = dictionary_fixture::complete_fixture().unwrap().document();
    context_document.legacy_archive = None;
    let context = Dictionary::from_document(context_document).unwrap();
    let (pin, target) = external_target(&context);
    let expected_entry_id = target.entry.as_str().to_owned();
    let expected_detail_reference = expected_detail_reference(&context, &target);
    let expected_spelling = match context
        .metadata()
        .entries
        .iter()
        .find(|row| row.id == target.entry)
        .unwrap()
        .spelling
        .clone()
    {
        Presence::Value(spelling) => spelling,
        other => panic!("fixture target spelling must be present, got {other:?}"),
    };
    let record = OwnedCurationRecord {
        id: EntityId::new("curation-cli-link").unwrap(),
        revision: candidate.metadata().owned_revisions[1].id.clone(),
        editor: ContributorId::new("curation-cli-editor").unwrap(),
        rationale: TextValue {
            text: "reviewed synthetic target".into(),
            language: Presence::Missing,
            markup: Presence::Missing,
        },
        kind: OwnedCurationKind::Link {
            origin: LocalRef::Entry(EntityId::new("curation-cli-entry").unwrap()),
            direction: Direction::Forward,
            target: target.clone(),
        },
    };
    let curation = OwnedCurationDocument::from_data(OwnedCurationData {
        version: 1,
        dictionary: candidate.clone(),
        contexts: vec![pin.clone()],
        records: vec![record],
        revisions: vec![
            OwnedCurationRevision {
                revision: base_revision.clone(),
                active: vec![],
                tombstones: vec![],
            },
            OwnedCurationRevision {
                revision: candidate.metadata().owned_revisions[1].id.clone(),
                active: vec![EntityId::new("curation-cli-link").unwrap()],
                tombstones: vec![],
            },
        ],
    })
    .unwrap();
    write_new(&curation_path, &curation.to_json_bytes().unwrap());
    write_new(
        &context_path,
        &OwnedContextInput {
            id: pin.id.clone(),
            dictionary: context.clone(),
        }
        .to_json_bytes()
        .unwrap(),
    );
    let missing_root = scratch.path().join("must-not-exist-before-parse");
    let malformed_curation_path = scratch.path().join("duplicate-curation-key.json");
    write_new(&malformed_curation_path, br#"{"version":1,"version":1}"#);
    let malformed_curate = run(
        &[
            "owned",
            "curate",
            missing_root.to_str().unwrap(),
            "--expected-head",
            base_revision.as_str(),
            "--input",
            malformed_curation_path.to_str().unwrap(),
        ],
        &scratch,
    );
    assert!(!malformed_curate.status.success());
    assert_eq!(
        serde_json::from_slice::<Value>(&malformed_curate.stderr).unwrap()["error"]["code"],
        "owned_invalid_curation"
    );
    assert!(!missing_root.exists());

    let malformed_context_path = scratch.path().join("duplicate-context-key.json");
    write_new(
        &malformed_context_path,
        br#"{"id":"context","id":"context","dictionary":{}}"#,
    );
    let missing_materialize_output = scratch.path().join("must-not-be-created.json");
    let malformed_materialize = run(
        &[
            "owned",
            "materialize",
            missing_root.to_str().unwrap(),
            "--context",
            malformed_context_path.to_str().unwrap(),
            "--output",
            missing_materialize_output.to_str().unwrap(),
        ],
        &scratch,
    );
    assert!(!malformed_materialize.status.success());
    assert_eq!(
        serde_json::from_slice::<Value>(&malformed_materialize.stderr).unwrap()["error"]["code"],
        "owned_invalid_curation"
    );
    assert!(!missing_root.exists());
    assert!(!missing_materialize_output.exists());

    let root = root.to_str().unwrap();
    let seed = seed_path.to_str().unwrap();
    let curation_input = curation_path.to_str().unwrap();
    let output = output_path.to_str().unwrap();
    let context_input = context_path.to_str().unwrap();
    let resolved_output = resolved_path.to_str().unwrap();

    let init = run(&["owned", "init", root, "--input", seed], &scratch);
    assert!(
        init.status.success(),
        "{}",
        String::from_utf8_lossy(&init.stderr)
    );
    assert_eq!(
        json(&init)["head"]["revision"]["id"],
        base_revision.as_str()
    );

    let pre_migration_show = run(&["owned", "curation-show", root], &scratch);
    assert!(!pre_migration_show.status.success());
    assert_eq!(
        serde_json::from_slice::<Value>(&pre_migration_show.stderr).unwrap()["error"]["code"],
        "owned_migration_required"
    );

    let migrated = run(
        &[
            "owned",
            "migrate-curation",
            root,
            "--expected-head",
            base_revision.as_str(),
        ],
        &scratch,
    );
    assert!(
        migrated.status.success(),
        "{}",
        String::from_utf8_lossy(&migrated.stderr)
    );
    assert_eq!(json(&migrated)["outcome"], "migrated");

    let curated = run(
        &[
            "owned",
            "curate",
            root,
            "--expected-head",
            base_revision.as_str(),
            "--input",
            curation_input,
        ],
        &scratch,
    );
    assert!(
        curated.status.success(),
        "{}",
        String::from_utf8_lossy(&curated.stderr)
    );

    let shown = run(&["owned", "curation-show", root], &scratch);
    assert!(
        shown.status.success(),
        "{}",
        String::from_utf8_lossy(&shown.stderr)
    );
    assert_eq!(
        json(&shown)["dictionary"],
        serde_json::from_slice::<Value>(&candidate.to_json_bytes().unwrap()).unwrap()
    );
    assert_eq!(json(&shown)["revisions"].as_array().unwrap().len(), 2);
    assert!(json(&shown).get("resolutions").is_none());

    let materialized = run(
        &["owned", "materialize", root, "--output", output],
        &scratch,
    );
    assert!(
        materialized.status.success(),
        "{}",
        String::from_utf8_lossy(&materialized.stderr)
    );
    let report = json(&materialized);
    assert_eq!(report["version"], 1);
    assert_eq!(report["source"], "curation-cli-source");
    assert_eq!(report["revision"], "curation-cli-r1");
    assert_eq!(report["resolutions"][0]["status"], "unavailable");
    assert_eq!(report["resolutions"][0]["reason"], "missing_context");
    let output_bytes = fs::read(&output_path).unwrap();

    #[cfg(unix)]
    {
        use std::{fs::File, os::fd::OwnedFd, os::unix::net::UnixStream};

        let report_failure_path = scratch.path().join("materialized-before-broken-pipe.json");
        let (peer, writer) = UnixStream::pair().unwrap();
        drop(peer);
        let writer_fd: OwnedFd = writer.into();
        let mut broken_stdout = File::from(writer_fd);
        assert_eq!(
            broken_stdout.write_all(b"preflight").unwrap_err().kind(),
            std::io::ErrorKind::BrokenPipe
        );
        let report_failure = run_with_stdout(
            &[
                "owned",
                "materialize",
                root,
                "--output",
                report_failure_path.to_str().unwrap(),
            ],
            &scratch,
            Stdio::from(broken_stdout),
        );
        assert!(report_failure.status.success());
        assert_eq!(
            fs::read(&report_failure_path).unwrap(),
            candidate.to_json_bytes().unwrap()
        );
        assert_eq!(
            json(&run(&["owned", "curation-show", root], &scratch)),
            json(&shown)
        );
    }
    assert_eq!(output_bytes, candidate.to_json_bytes().unwrap());
    assert_eq!(
        report["output_sha256"],
        format!("{:x}", Sha256::digest(&output_bytes))
    );

    let resolved = run(
        &[
            "owned",
            "materialize",
            root,
            "--context",
            context_input,
            "--output",
            resolved_output,
        ],
        &scratch,
    );
    assert!(
        resolved.status.success(),
        "{}",
        String::from_utf8_lossy(&resolved.stderr)
    );
    assert_eq!(json(&resolved)["resolutions"][0]["status"], "resolved");
    let resolved_dictionary =
        Dictionary::from_json_bytes(&fs::read(&resolved_path).unwrap()).unwrap();
    assert!(
        resolved_dictionary
            .metadata()
            .relations
            .iter()
            .any(|row| row.id.as_str() == "curation-cli-link")
    );

    let mut changed_context_document = context.document();
    changed_context_document.metadata.releases[0]
        .normalizer_version
        .push_str("-changed");
    let changed_context = Dictionary::from_document(changed_context_document).unwrap();
    write_new(
        &wrong_context_path,
        &OwnedContextInput {
            id: pin.id.clone(),
            dictionary: changed_context,
        }
        .to_json_bytes()
        .unwrap(),
    );
    let rejected = run(
        &[
            "owned",
            "materialize",
            root,
            "--context",
            wrong_context_path.to_str().unwrap(),
            "--output",
            rejected_output_path.to_str().unwrap(),
        ],
        &scratch,
    );
    assert!(!rejected.status.success());
    assert_eq!(
        serde_json::from_slice::<Value>(&rejected.stderr).unwrap()["error"]["code"],
        "owned_invalid_curation"
    );
    assert!(!rejected_output_path.exists());
    let retained_after_rejection = run(&["owned", "curation-show", root], &scratch);
    assert!(retained_after_rejection.status.success());
    assert_eq!(json(&retained_after_rejection), json(&shown));

    let derived_store = scratch.path().join("derived-store");
    let build = run(
        &[
            "dictionary-build",
            resolved_output,
            derived_store.to_str().unwrap(),
        ],
        &scratch,
    );
    assert!(
        build.status.success(),
        "{}",
        String::from_utf8_lossy(&build.stderr)
    );
    let request = serde_json::json!({"operation":"exact","spelling":expected_spelling});
    let query = run(
        &[
            "dictionary-query",
            derived_store.to_str().unwrap(),
            "--request",
            &request.to_string(),
            "--format",
            "compact",
        ],
        &scratch,
    );
    assert!(
        query.status.success(),
        "{}",
        String::from_utf8_lossy(&query.stderr)
    );
    let queried = json(&query);
    let target_hit = queried["groups"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|group| group["hits"].as_array().unwrap())
        .find(|hit| hit["entry"]["id"] == expected_entry_id)
        .expect("derived query includes its input-derived target entry");
    assert_eq!(
        serde_json::from_value::<vlajmaji_search::DictionaryHitDetailRef>(
            target_hit["detail_ref"].clone()
        )
        .unwrap(),
        expected_detail_reference
    );
    let reference = target_hit["detail_ref"].to_string();
    let detail = run(
        &[
            "dictionary-detail",
            derived_store.to_str().unwrap(),
            "--reference",
            &reference,
        ],
        &scratch,
    );
    assert!(
        detail.status.success(),
        "{}",
        String::from_utf8_lossy(&detail.stderr)
    );
    let detail = json(&detail);
    assert_eq!(detail["status"], "Resolved");
    assert_eq!(detail["hit"]["entry"]["id"], expected_entry_id);
    let duplicate_output = run(
        &["owned", "materialize", root, "--output", output],
        &scratch,
    );
    assert!(!duplicate_output.status.success());
    assert_eq!(
        serde_json::from_slice::<Value>(&duplicate_output.stderr).unwrap()["error"]["code"],
        "owned_io"
    );
    let retained_after_output_error = run(&["owned", "curation-show", root], &scratch);
    assert!(retained_after_output_error.status.success());
    assert_eq!(json(&retained_after_output_error), json(&shown));

    let stdin_root = scratch.path().join("stdin-journal");
    let stdin_root = stdin_root.to_str().unwrap();
    let stdin_init = run(&["owned", "init", stdin_root, "--input", seed], &scratch);
    assert!(stdin_init.status.success());
    let stdin_migrate = run(
        &[
            "owned",
            "migrate-curation",
            stdin_root,
            "--expected-head",
            base_revision.as_str(),
        ],
        &scratch,
    );
    assert!(stdin_migrate.status.success());
    let stdin_curate = run_with_stdin(
        &[
            "owned",
            "curate",
            stdin_root,
            "--expected-head",
            base_revision.as_str(),
        ],
        &scratch,
        &curation.to_json_bytes().unwrap(),
    );
    assert!(
        stdin_curate.status.success(),
        "{}",
        String::from_utf8_lossy(&stdin_curate.stderr)
    );
    assert_eq!(
        json(&run(&["owned", "curation-show", stdin_root], &scratch))["dictionary"],
        serde_json::from_slice::<Value>(&candidate.to_json_bytes().unwrap()).unwrap()
    );

    #[cfg(unix)]
    {
        use std::{fs::File, os::fd::OwnedFd, os::unix::net::UnixStream};

        let pipe_root = scratch.path().join("broken-pipe-journal");
        let pipe_root = pipe_root.to_str().unwrap();
        let pipe_init = run(&["owned", "init", pipe_root, "--input", seed], &scratch);
        assert!(pipe_init.status.success());
        let pipe_migrate = run(
            &[
                "owned",
                "migrate-curation",
                pipe_root,
                "--expected-head",
                base_revision.as_str(),
            ],
            &scratch,
        );
        assert!(pipe_migrate.status.success());

        let (peer, writer) = UnixStream::pair().unwrap();
        drop(peer);
        let writer_fd: OwnedFd = writer.into();
        let mut broken_stdout = File::from(writer_fd);
        assert_eq!(
            broken_stdout.write_all(b"preflight").unwrap_err().kind(),
            std::io::ErrorKind::BrokenPipe
        );
        let committed_through_broken_pipe = run_with_stdin_and_stdout(
            &[
                "owned",
                "curate",
                pipe_root,
                "--expected-head",
                base_revision.as_str(),
            ],
            &scratch,
            &curation.to_json_bytes().unwrap(),
            Stdio::from(broken_stdout),
        );
        assert!(committed_through_broken_pipe.status.success());
        let pipe_show = run(&["owned", "curation-show", pipe_root], &scratch);
        assert!(pipe_show.status.success());
        assert_eq!(
            json(&pipe_show)["dictionary"],
            serde_json::from_slice::<Value>(&candidate.to_json_bytes().unwrap()).unwrap()
        );
    }

    #[cfg(unix)]
    {
        use std::os::unix::ffi::{OsStrExt, OsStringExt};

        let native_root = scratch.path().join("native root λ");
        let native_output = scratch.path().join("native output λ.json");
        let native_root_arg = native_root.as_os_str().to_os_string();
        let native_seed_arg = seed_path.as_os_str().to_os_string();
        let native_init = run_native(
            &[
                "owned".into(),
                "init".into(),
                native_root_arg.clone(),
                "--input".into(),
                native_seed_arg,
            ],
            &scratch,
        );
        assert!(
            native_init.status.success(),
            "{}",
            String::from_utf8_lossy(&native_init.stderr)
        );
        let native_migrate = run_native(
            &[
                "owned".into(),
                "migrate-curation".into(),
                native_root_arg.clone(),
                "--expected-head".into(),
                base_revision.to_string().into(),
            ],
            &scratch,
        );
        assert!(
            native_migrate.status.success(),
            "{}",
            String::from_utf8_lossy(&native_migrate.stderr)
        );
        let native_curate = run_native_with_stdin(
            &[
                "owned".into(),
                "curate".into(),
                native_root_arg.clone(),
                "--expected-head".into(),
                base_revision.to_string().into(),
            ],
            &scratch,
            &curation.to_json_bytes().unwrap(),
        );
        assert!(
            native_curate.status.success(),
            "{}",
            String::from_utf8_lossy(&native_curate.stderr)
        );
        let native_output_run = run_native(
            &[
                "owned".into(),
                "materialize".into(),
                native_root_arg,
                "--output".into(),
                native_output.as_os_str().to_os_string(),
            ],
            &scratch,
        );
        assert!(
            native_output_run.status.success(),
            "{}",
            String::from_utf8_lossy(&native_output_run.stderr)
        );
        let report = json(&native_output_run);
        assert_eq!(report["output"]["encoding"], "unix-bytes");
        assert_eq!(report["output"]["utf8"], native_output.to_str().unwrap());
        let expected_hex = native_output
            .as_os_str()
            .as_bytes()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        assert_eq!(report["output"]["hex"], expected_hex);
        assert_eq!(
            fs::read(&native_output).unwrap(),
            candidate.to_json_bytes().unwrap()
        );

        #[cfg(target_os = "linux")]
        {
            let raw_root = scratch
                .path()
                .join(OsString::from_vec(b"native-root-\xff".to_vec()));
            let raw_seed = scratch
                .path()
                .join(OsString::from_vec(b"native-seed-\xfd.json".to_vec()));
            let raw_output = scratch
                .path()
                .join(OsString::from_vec(b"native-output-\xfe.json".to_vec()));
            write_new(&raw_seed, &dictionary_bytes);
            let raw_root_arg = raw_root.as_os_str().to_os_string();
            let raw_init = run_native(
                &[
                    "owned".into(),
                    "init".into(),
                    raw_root_arg.clone(),
                    "--input".into(),
                    raw_seed.as_os_str().to_os_string(),
                ],
                &scratch,
            );
            assert!(
                raw_init.status.success(),
                "{}",
                String::from_utf8_lossy(&raw_init.stderr)
            );
            let raw_migrate = run_native(
                &[
                    "owned".into(),
                    "migrate-curation".into(),
                    raw_root_arg.clone(),
                    "--expected-head".into(),
                    base_revision.to_string().into(),
                ],
                &scratch,
            );
            assert!(
                raw_migrate.status.success(),
                "{}",
                String::from_utf8_lossy(&raw_migrate.stderr)
            );
            let raw_curate = run_native_with_stdin(
                &[
                    "owned".into(),
                    "curate".into(),
                    raw_root_arg.clone(),
                    "--expected-head".into(),
                    base_revision.to_string().into(),
                ],
                &scratch,
                &curation.to_json_bytes().unwrap(),
            );
            assert!(
                raw_curate.status.success(),
                "{}",
                String::from_utf8_lossy(&raw_curate.stderr)
            );
            let raw_output_run = run_native(
                &[
                    "owned".into(),
                    "materialize".into(),
                    raw_root_arg,
                    "--output".into(),
                    raw_output.as_os_str().to_os_string(),
                ],
                &scratch,
            );
            assert!(
                raw_output_run.status.success(),
                "{}",
                String::from_utf8_lossy(&raw_output_run.stderr)
            );
            assert_eq!(
                fs::read(&raw_output).unwrap(),
                candidate.to_json_bytes().unwrap()
            );
        }

        #[cfg(target_os = "macos")]
        {
            let impossible_root = scratch
                .path()
                .join(OsString::from_vec(b"native-root-\xff".to_vec()));
            let filesystem_error = fs::create_dir(&impossible_root).unwrap_err();
            assert_eq!(filesystem_error.raw_os_error(), Some(92));
            let impossible_root_arg = impossible_root.as_os_str().to_os_string();
            let impossible_init = run_native(
                &[
                    "owned".into(),
                    "init".into(),
                    impossible_root_arg,
                    "--input".into(),
                    seed_path.as_os_str().to_os_string(),
                ],
                &scratch,
            );
            assert!(!impossible_init.status.success());
            let error: Value = serde_json::from_slice(&impossible_init.stderr).unwrap();
            assert_eq!(error["error"]["code"], "owned_io");
            assert!(!impossible_root.exists());
        }
    }
}
