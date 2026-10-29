//! Public CLI lifecycle checks for explicitly rooted owned journals.

use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    sync::atomic::{AtomicUsize, Ordering},
};

use serde_json::Value;
#[cfg(unix)]
use std::os::unix::ffi::OsStringExt;
#[cfg(unix)]
use std::{
    os::fd::OwnedFd,
    os::unix::net::{UnixDatagram, UnixStream},
};
use vlajmaji_lexicon::{
    AbbreviationKind, Availability, CollectionRecord, ComponentRecord, ContentNotice,
    ContributorId, ContributorRecord, Dictionary, DictionaryDocument, DictionaryMetadata, EntityId,
    OwnedHead, OwnedRevisionRecord, Presence, RedistributionStatus, RegistryDocument,
    ReleaseAccounting, ReleaseRecord, SourceId, SourceOrigin, SourceRecord, SourceRegistry,
    TextValue, TimePrecision,
};

static NEXT_ROOT: AtomicUsize = AtomicUsize::new(0);

struct Scratch(PathBuf);

impl Scratch {
    fn create() -> Self {
        for _ in 0..128 {
            let path = std::env::temp_dir().join(format!(
                "vlajmaji-owned-cli-{}-{}",
                std::process::id(),
                NEXT_ROOT.fetch_add(1, Ordering::Relaxed)
            ));
            match fs::create_dir(&path) {
                Ok(()) => return Self(path),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => panic!("create exclusive owned CLI scratch: {error}"),
            }
        }
        panic!("could not allocate owned CLI scratch after 128 attempts");
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

fn write_new(path: &Path, bytes: &[u8]) {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .unwrap();
    file.write_all(bytes).unwrap();
    file.sync_all().unwrap();
}

fn notice() -> ContentNotice {
    ContentNotice {
        text: Presence::Value("synthetic CLI test; no licence grant".into()),
        locator: Presence::Missing,
        license: Presence::Missing,
    }
}

fn release(component: &EntityId) -> ReleaseRecord {
    ReleaseRecord {
        id: EntityId::new("owned-cli-release").unwrap(),
        component: component.clone(),
        distributor: None,
        upstream_revision: Presence::Missing,
        distributor_revision: Presence::Missing,
        captured_at: Presence::Missing,
        time_precision: TimePrecision::Unknown,
        schema_version: 1,
        normalizer: "owned-cli-test".into(),
        normalizer_version: "1".into(),
        captures: Vec::new(),
        assertions: Vec::new(),
        accounting: ReleaseAccounting {
            input_occurrences: Some(0),
            normalized_occurrences: Some(0),
            query_excluded_occurrences: Some(0),
            unit: "occurrences".into(),
        },
        diagnostics: Vec::new(),
        selected: true,
        availability: Availability::Available,
        content_notices: vec![notice()],
        redistribution: RedistributionStatus::Held,
        redistribution_evidence: Vec::new(),
    }
}

fn seed() -> Dictionary {
    let source = SourceId::new("owned-cli-source").unwrap();
    let editor = ContributorId::new("owned-cli-editor").unwrap();
    let collection = EntityId::new("owned-cli-collection").unwrap();
    let component = EntityId::new("owned-cli-component").unwrap();
    let revision = EntityId::new("owned-cli-r0").unwrap();
    let registry = SourceRegistry::from_document(RegistryDocument {
        sources: vec![SourceRecord {
            id: source.clone(),
            display_name: "CLI-owned dictionary".into(),
            abbreviation: "OWN".into(),
            abbreviation_kind: AbbreviationKind::ProjectAssigned,
            abbreviation_evidence: Some("synthetic CLI fixture".into()),
            origin: SourceOrigin::OwnedDraft,
        }],
        contributors: vec![ContributorRecord {
            id: editor.clone(),
            display_name: "Local editor".into(),
        }],
        ..RegistryDocument::default()
    })
    .unwrap();
    let mut metadata = DictionaryMetadata::default();
    metadata.collections.push(CollectionRecord {
        id: collection.clone(),
        source: source.clone(),
        display_name: "Personal dictionary".into(),
        description: Presence::Value(TextValue {
            text: "A local owned dictionary".into(),
            language: Presence::Missing,
            markup: Presence::Missing,
        }),
    });
    metadata.components.push(ComponentRecord {
        id: component.clone(),
        collection,
        native_name: "entries".into(),
        languages: vec!["jbo".into()],
    });
    metadata.releases.push(release(&component));
    metadata.owned_revisions.push(OwnedRevisionRecord {
        id: revision.clone(),
        source: source.clone(),
        parent: None,
        editor,
        timestamp: Presence::Missing,
        time_precision: TimePrecision::Unknown,
        assertions: Vec::new(),
        tombstones: Vec::new(),
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

fn append_revision(current: &Dictionary, number: u8) -> Dictionary {
    let mut document = current.document();
    let previous = document.metadata.owned_revisions.last().unwrap().clone();
    document.metadata.catalogue_revision += 1;
    let revision = EntityId::new(format!("owned-cli-r{number}")).unwrap();
    document.metadata.owned_revisions.push(OwnedRevisionRecord {
        id: revision.clone(),
        source: previous.source.clone(),
        parent: Some(previous.id),
        editor: previous.editor,
        timestamp: Presence::Missing,
        time_precision: TimePrecision::Unknown,
        assertions: Vec::new(),
        tombstones: Vec::new(),
        rationale: Presence::Value(TextValue {
            text: format!("revision {number}"),
            language: Presence::Missing,
            markup: Presence::Missing,
        }),
    });
    document.metadata.owned_heads[0].revision = revision;
    Dictionary::from_document(document).unwrap()
}

fn run(args: &[&str], input: Option<&[u8]>, scratch: &Scratch) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_vlajmaji"));
    command
        .args(args)
        .env("HOME", scratch.path().join("unused-home"))
        .env("XDG_DATA_HOME", scratch.path().join("unused-data"))
        .env("XDG_CONFIG_HOME", scratch.path().join("unused-config"))
        .env("TMPDIR", scratch.path())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if input.is_some() {
        command.stdin(Stdio::piped());
    }
    let mut child = command.spawn().unwrap();
    if let Some(input) = input {
        child.stdin.take().unwrap().write_all(input).unwrap();
    }
    child.wait_with_output().unwrap()
}

fn value(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
        panic!(
            "stdout is not JSON: {error}; stderr={}",
            String::from_utf8_lossy(&output.stderr)
        )
    })
}

#[test]
fn explicit_root_file_stdin_history_show_replay_and_postcommit_output_failure() {
    let scratch = Scratch::create();
    let inputs = scratch.path().join("inputs");
    fs::create_dir(&inputs).unwrap();
    let root = scratch.path().join("journal");
    let initial = seed();
    let first = append_revision(&initial, 1);
    let second = append_revision(&first, 2);
    let third = append_revision(&second, 3);
    let initial_bytes = initial.to_json_bytes().unwrap();
    let first_bytes = first.to_json_bytes().unwrap();
    let second_bytes = second.to_json_bytes().unwrap();
    let third_bytes = third.to_json_bytes().unwrap();
    let seed_path = inputs.join("seed.json");
    let first_path = inputs.join("first.json");
    let second_path = inputs.join("second.json");
    let third_path = inputs.join("third.json");
    write_new(&seed_path, &initial_bytes);
    write_new(&first_path, &first_bytes);
    write_new(&second_path, &second_bytes);
    write_new(&third_path, &third_bytes);
    let root_arg = root.to_str().unwrap();
    let seed_arg = seed_path.to_str().unwrap();
    let first_arg = first_path.to_str().unwrap();
    let second_arg = second_path.to_str().unwrap();
    let third_arg = third_path.to_str().unwrap();

    let initialized = run(
        &["owned", "init", root_arg, "--input", seed_arg],
        None,
        &scratch,
    );
    assert!(
        initialized.status.success(),
        "{}",
        String::from_utf8_lossy(&initialized.stderr)
    );
    let initialized_value = value(&initialized);
    assert_eq!(initialized_value["operation"], "init");
    assert_eq!(initialized_value["head"]["sequence"], 0);
    assert_eq!(initialized_value["head"]["revision"]["id"], "owned-cli-r0");

    let head = run(&["owned", "head", root_arg], None, &scratch);
    assert!(
        head.status.success(),
        "{}",
        String::from_utf8_lossy(&head.stderr)
    );
    assert_eq!(value(&head)["head"]["revision"]["id"], "owned-cli-r0");
    let history = run(&["owned", "history", root_arg], None, &scratch);
    assert!(
        history.status.success(),
        "{}",
        String::from_utf8_lossy(&history.stderr)
    );
    assert_eq!(value(&history)["commits"].as_array().unwrap().len(), 1);
    let show = run(&["owned", "show", root_arg], None, &scratch);
    assert!(
        show.status.success(),
        "{}",
        String::from_utf8_lossy(&show.stderr)
    );
    assert_eq!(
        value(&show)["dictionary"],
        serde_json::from_slice::<Value>(&initial_bytes).unwrap()
    );
    let old_show = run(
        &["owned", "show", root_arg, "--revision", "owned-cli-r0"],
        None,
        &scratch,
    );
    assert!(
        old_show.status.success(),
        "{}",
        String::from_utf8_lossy(&old_show.stderr)
    );
    assert_eq!(
        value(&old_show)["dictionary"],
        serde_json::from_slice::<Value>(&initial_bytes).unwrap()
    );

    let edit = run(
        &["owned", "edit", root_arg, "--expected-head", "owned-cli-r0"],
        Some(&first_bytes),
        &scratch,
    );
    assert!(
        edit.status.success(),
        "{}",
        String::from_utf8_lossy(&edit.stderr)
    );
    assert_eq!(value(&edit)["operation"], "edit");
    assert_eq!(value(&edit)["head"]["revision"]["id"], "owned-cli-r1");
    let history = run(&["owned", "history", root_arg], None, &scratch);
    assert!(
        history.status.success(),
        "{}",
        String::from_utf8_lossy(&history.stderr)
    );
    assert_eq!(value(&history)["commits"].as_array().unwrap().len(), 2);

    let replay = run(
        &[
            "owned",
            "edit",
            root_arg,
            "--expected-head",
            "owned-cli-r0",
            "--input",
            first_arg,
        ],
        None,
        &scratch,
    );
    assert!(!replay.status.success());
    let replay_error: Value = serde_json::from_slice(&replay.stderr).unwrap();
    assert_eq!(replay_error["error"]["code"], "owned_conflict");
    let unchanged = run(&["owned", "show", root_arg], None, &scratch);
    assert!(unchanged.status.success());
    assert_eq!(
        value(&unchanged)["dictionary"],
        serde_json::from_slice::<Value>(&first_bytes).unwrap()
    );

    #[cfg(unix)]
    {
        let (full_sender, _unread_peer) = UnixDatagram::pair().unwrap();
        full_sender.set_nonblocking(true).unwrap();
        let packet = [b'x'; 1];
        let mut reached_would_block = false;
        for _ in 0..16_384 {
            match full_sender.send(&packet) {
                Ok(_) => {}
                Err(error)
                    if error.kind() == std::io::ErrorKind::WouldBlock
                        || (cfg!(target_os = "macos") && error.raw_os_error() == Some(55)) =>
                {
                    // Darwin reports a full nonblocking datagram queue as ENOBUFS (55).
                    reached_would_block = true;
                    break;
                }
                Err(error) => panic!("fill owned output datagram queue: {error}"),
            }
        }
        assert!(
            reached_would_block,
            "bounded output socket fill did not saturate"
        );
        let sender_fd: std::os::fd::OwnedFd = full_sender.into();
        let mut sender_file = File::from(sender_fd);
        let write_error = sender_file
            .write(b"x")
            .expect_err("one-byte stdout preflight must fail on the full queue");
        assert!(
            write_error.kind() == std::io::ErrorKind::WouldBlock
                || (cfg!(target_os = "macos") && write_error.raw_os_error() == Some(55)),
            "unexpected saturated stdout error: {write_error}"
        );
        let mut command = Command::new(env!("CARGO_BIN_EXE_vlajmaji"));
        command
            .args([
                "owned",
                "edit",
                root_arg,
                "--expected-head",
                "owned-cli-r1",
                "--input",
                second_arg,
            ])
            .env("HOME", scratch.path().join("unused-home"))
            .env("XDG_DATA_HOME", scratch.path().join("unused-data"))
            .env("XDG_CONFIG_HOME", scratch.path().join("unused-config"))
            .env("TMPDIR", scratch.path())
            .stdin(Stdio::null())
            .stdout(Stdio::from(sender_file))
            .stderr(Stdio::piped());
        let failed_output = command.spawn().unwrap().wait_with_output().unwrap();
        assert!(
            !failed_output.status.success(),
            "full nonblocking stdout unexpectedly succeeded: status={:?}; stderr={}",
            failed_output.status,
            String::from_utf8_lossy(&failed_output.stderr)
        );
        let output_error: Value = serde_json::from_slice(&failed_output.stderr).unwrap();
        assert_eq!(output_error["error"]["code"], "response_output_io");
        let committed = run(&["owned", "show", root_arg], None, &scratch);
        assert!(
            committed.status.success(),
            "{}",
            String::from_utf8_lossy(&committed.stderr)
        );
        assert_eq!(
            value(&committed)["dictionary"],
            serde_json::from_slice::<Value>(&second_bytes).unwrap()
        );

        let (broken_sender, broken_peer) = UnixStream::pair().unwrap();
        drop(broken_peer);
        let sender_fd: OwnedFd = broken_sender.into();
        let mut sender_file = File::from(sender_fd);
        let broken_pipe = sender_file
            .write(b"x")
            .expect_err("closed-pipe stdout preflight must fail");
        assert_eq!(broken_pipe.kind(), std::io::ErrorKind::BrokenPipe);
        let mut command = Command::new(env!("CARGO_BIN_EXE_vlajmaji"));
        command
            .args([
                "owned",
                "edit",
                root_arg,
                "--expected-head",
                "owned-cli-r2",
                "--input",
                third_arg,
            ])
            .env("HOME", scratch.path().join("unused-home"))
            .env("XDG_DATA_HOME", scratch.path().join("unused-data"))
            .env("XDG_CONFIG_HOME", scratch.path().join("unused-config"))
            .env("TMPDIR", scratch.path())
            .stdin(Stdio::null())
            .stdout(Stdio::from(sender_file))
            .stderr(Stdio::piped());
        let broken_output = command.spawn().unwrap().wait_with_output().unwrap();
        assert!(broken_output.status.success());
        assert!(broken_output.stderr.is_empty());
        let committed = run(&["owned", "show", root_arg], None, &scratch);
        assert!(committed.status.success());
        assert_eq!(
            value(&committed)["dictionary"],
            serde_json::from_slice::<Value>(&third_bytes).unwrap()
        );
    }

    assert!(!scratch.path().join("unused-home").exists());
    assert!(!scratch.path().join("unused-data").exists());
    assert!(!scratch.path().join("unused-config").exists());
}

#[test]
fn invalid_input_is_rejected_before_creating_explicit_root() {
    let scratch = Scratch::create();
    let root = scratch.path().join("invalid-journal");
    let output = run(
        &["owned", "init", root.to_str().unwrap()],
        Some(b"{"),
        &scratch,
    );
    assert!(!output.status.success());
    let error: Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(error["error"]["code"], "owned_invalid_dictionary");
    assert!(!root.exists());

    let valid_bytes = seed().to_json_bytes().unwrap();
    let mut invalid_cases = Vec::new();
    let mut unknown_editor: Value = serde_json::from_slice(&valid_bytes).unwrap();
    unknown_editor["metadata"]["owned_revisions"][0]["editor"] =
        Value::String("unregistered-editor".into());
    invalid_cases.push((
        "unknown-editor",
        serde_json::to_vec(&unknown_editor).unwrap(),
    ));

    let mut non_genesis_parent: Value = serde_json::from_slice(&valid_bytes).unwrap();
    non_genesis_parent["metadata"]["owned_revisions"][0]["parent"] =
        Value::String("owned-absent-parent".into());
    invalid_cases.push((
        "non-genesis-parent",
        serde_json::to_vec(&non_genesis_parent).unwrap(),
    ));

    let mut multiple_heads: Value = serde_json::from_slice(&valid_bytes).unwrap();
    let head = multiple_heads["metadata"]["owned_heads"][0].clone();
    multiple_heads["metadata"]["owned_heads"]
        .as_array_mut()
        .unwrap()
        .push(head);
    invalid_cases.push((
        "multiple-heads",
        serde_json::to_vec(&multiple_heads).unwrap(),
    ));

    let mut unsupported_model_version: Value = serde_json::from_slice(&valid_bytes).unwrap();
    unsupported_model_version["version"] = Value::from(99);
    invalid_cases.push((
        "unsupported-model-version",
        serde_json::to_vec(&unsupported_model_version).unwrap(),
    ));

    let mut duplicate_keys = String::from_utf8(valid_bytes).unwrap();
    assert!(duplicate_keys.contains("\"version\":1"));
    duplicate_keys = duplicate_keys.replacen("\"version\":1", "\"version\":1,\"version\":1", 1);
    invalid_cases.push(("duplicate-json-key", duplicate_keys.into_bytes()));

    for (name, bytes) in invalid_cases {
        let case_root = scratch.path().join(format!("invalid-{name}"));
        let output = run(
            &["owned", "init", case_root.to_str().unwrap()],
            Some(&bytes),
            &scratch,
        );
        assert!(!output.status.success(), "{name} unexpectedly initialized");
        let error: Value = serde_json::from_slice(&output.stderr).unwrap();
        assert_eq!(error["error"]["code"], "owned_invalid_dictionary", "{name}");
        assert!(!case_root.exists(), "{name} created a journal root");
    }

    let mut foreign_origin = seed().document();
    let mut registry = foreign_origin.registry.document();
    registry.sources[0].origin = SourceOrigin::Upstream;
    foreign_origin.registry = SourceRegistry::from_document(registry).unwrap();
    let bytes = serde_json::to_vec(&foreign_origin).unwrap();
    let output = run(
        &["owned", "init", root.to_str().unwrap()],
        Some(&bytes),
        &scratch,
    );
    assert!(!output.status.success());
    let error: Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(error["error"]["code"], "owned_invalid_dictionary");
    assert!(!root.exists());
}

#[cfg(unix)]
#[test]
fn native_paths_and_deleted_working_directory_use_explicit_absolute_paths() {
    let scratch = Scratch::create();
    let native_root = scratch
        .path()
        .join(std::ffi::OsString::from_vec(b"journal-\xff".to_vec()));
    let native_input = scratch
        .path()
        .join(std::ffi::OsString::from_vec(b"seed-\xfe.json".to_vec()));
    let initial = seed();
    let revised = append_revision(&initial, 1);
    let initial_bytes = initial.to_json_bytes().unwrap();
    let revised_bytes = revised.to_json_bytes().unwrap();
    let native_supported = match OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&native_input)
    {
        Ok(mut file) => {
            file.write_all(&initial_bytes).unwrap();
            file.sync_all().unwrap();
            true
        }
        Err(error) if cfg!(target_os = "macos") && error.raw_os_error() == Some(92) => false,
        Err(error) => panic!("create native-path input fixture: {error}"),
    };
    // macOS filesystems reject non-UTF-8 names with EILSEQ; Linux runs the native-path branch.
    let (root, input) = if native_supported {
        (native_root.clone(), native_input)
    } else {
        let input = scratch.path().join("dictionärÿ seed.json");
        write_new(&input, &initial_bytes);
        (scratch.path().join("owned journal λ"), input)
    };

    let initialized = run_os(
        vec![
            "owned".into(),
            "init".into(),
            root.as_os_str().to_os_string(),
            "--input".into(),
            input.as_os_str().to_os_string(),
        ],
        None,
        &scratch,
    );
    assert!(
        initialized.status.success(),
        "{}",
        String::from_utf8_lossy(&initialized.stderr)
    );
    assert_eq!(
        value(&initialized)["head"]["revision"]["id"],
        "owned-cli-r0"
    );
    if cfg!(target_os = "macos") && !native_supported {
        let rejected = run_os(
            vec![
                "owned".into(),
                "init".into(),
                native_root.as_os_str().to_os_string(),
                "--input".into(),
                input.as_os_str().to_os_string(),
            ],
            None,
            &scratch,
        );
        assert!(!rejected.status.success());
        let error: Value = serde_json::from_slice(&rejected.stderr).unwrap();
        assert_eq!(error["error"]["code"], "owned_io");
        assert!(!native_root.exists());
    }

    let deleted_cwd = scratch.path().join("deleted-current-directory");
    fs::create_dir(&deleted_cwd).unwrap();
    let mut command = Command::new("/bin/sh");
    command
        .args(["-c", "rmdir \"$1\" || exit; shift; exec \"$@\"", "sh"])
        .arg(&deleted_cwd)
        .arg(env!("CARGO_BIN_EXE_vlajmaji"))
        .args(["owned", "edit"])
        .arg(&root)
        .args(["--expected-head", "owned-cli-r0"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .current_dir(&deleted_cwd)
        .env("HOME", scratch.path().join("unused-home"))
        .env("XDG_DATA_HOME", scratch.path().join("unused-data"))
        .env("XDG_CONFIG_HOME", scratch.path().join("unused-config"))
        .env("TMPDIR", scratch.path());
    let mut child = command.spawn().unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(&revised_bytes)
        .unwrap();
    let edited = child.wait_with_output().unwrap();
    assert!(
        edited.status.success(),
        "{}",
        String::from_utf8_lossy(&edited.stderr)
    );
    assert_eq!(value(&edited)["head"]["revision"]["id"], "owned-cli-r1");

    let shown = run_os(
        vec![
            "owned".into(),
            "show".into(),
            root.as_os_str().to_os_string(),
        ],
        None,
        &scratch,
    );
    assert!(shown.status.success());
    assert_eq!(
        value(&shown)["dictionary"],
        serde_json::from_slice::<Value>(&revised_bytes).unwrap()
    );
    assert!(!scratch.path().join("unused-home").exists());
    assert!(!scratch.path().join("unused-data").exists());
    assert!(!scratch.path().join("unused-config").exists());
}

#[cfg(unix)]
fn run_os(args: Vec<std::ffi::OsString>, input: Option<&[u8]>, scratch: &Scratch) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_vlajmaji"));
    command
        .args(args)
        .env("HOME", scratch.path().join("unused-home"))
        .env("XDG_DATA_HOME", scratch.path().join("unused-data"))
        .env("XDG_CONFIG_HOME", scratch.path().join("unused-config"))
        .env("TMPDIR", scratch.path())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if input.is_some() {
        command.stdin(Stdio::piped());
    }
    let mut child = command.spawn().unwrap();
    if let Some(input) = input {
        child.stdin.take().unwrap().write_all(input).unwrap();
    }
    child.wait_with_output().unwrap()
}
