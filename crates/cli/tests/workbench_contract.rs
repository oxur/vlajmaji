//! Executable native-file contracts for explicit mixed workbench commands.

#[path = "../../lexicon/examples/support/dictionary_fixture.rs"]
mod fixture;

use std::{
    borrow::Cow,
    ffi::OsString,
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    sync::atomic::{AtomicUsize, Ordering},
};

use serde_json::Value;
use vlajmaji_lexicon::{Dictionary, PackId, PackPayloadKind, seal_release};
use vlajmaji_store::{
    CollectionCatalogue, CollectionInput, CollectionRequest, CollectionSelection,
    WorkbenchOwnedRevision, WorkbenchSelection, WorkbenchView,
};

static NEXT_ROOT: AtomicUsize = AtomicUsize::new(0);

fn fresh_root() -> PathBuf {
    for _ in 0..64 {
        let parent = std::env::temp_dir().join(format!(
            "vlajmaji-workbench-cli-{}-{}",
            std::process::id(),
            NEXT_ROOT.fetch_add(1, Ordering::Relaxed)
        ));
        match fs::create_dir(&parent) {
            Ok(()) => return parent,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => panic!("create isolated parent: {error}"),
        }
    }
    panic!("could not allocate an exclusive parent after 64 attempts")
}

fn upstream_dictionary() -> Dictionary {
    let mut document = fixture::complete_fixture().unwrap().into_document();
    document.legacy_archive = None;
    Dictionary::from_document(document).unwrap()
}

fn install(root: &std::path::Path) {
    let dictionary = upstream_dictionary();
    let sealed = seal_release(PackId::Llg, PackPayloadKind::Synthetic, &dictionary).unwrap();
    let request = CollectionRequest::new(
        vec![CollectionInput::SealedBytes {
            pack_id: PackId::Llg,
            release: Cow::Borrowed(&sealed.release),
            seal: Cow::Borrowed(&sealed.seal),
        }],
        CollectionSelection::Packs(vec![PackId::Llg]),
        true,
    )
    .unwrap();
    CollectionCatalogue::install(root, request).unwrap();
}

fn cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
}

#[cfg(unix)]
fn run_after_removing_cwd(cwd: &Path, arguments: &[OsString]) -> Output {
    let executable = PathBuf::from(env!("CARGO_BIN_EXE_vlajmaji"));
    Command::new("/bin/sh")
        .current_dir(cwd)
        .args(["-c", "rmdir \"$1\" || exit; shift; exec \"$@\"", "sh"])
        .arg(cwd)
        .arg(executable)
        .args(arguments)
        .output()
        .unwrap()
}

#[test]
fn test_workbench_build_info_and_query_use_explicit_native_file_roots() {
    let parent = fresh_root();
    let catalogue = parent.join("catalogue");
    install(&catalogue);
    let selection_path = parent.join("selection ü.json");
    let selection = WorkbenchSelection {
        version: 1,
        packs: vec![PackId::Llg],
        disabled: Vec::new(),
        owned: None,
        contexts: Vec::new(),
    };
    fs::write(&selection_path, serde_json::to_vec(&selection).unwrap()).unwrap();
    let root = parent.join("prepared vïew");
    let build = cli()
        .args([
            "workbench",
            "build",
            root.to_str().unwrap(),
            "--selection",
            selection_path.to_str().unwrap(),
            "--catalogue",
            catalogue.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        build.status.success(),
        "{}",
        String::from_utf8_lossy(&build.stderr)
    );
    assert!(build.stderr.is_empty());
    let build_json: Value = serde_json::from_slice(&build.stdout).unwrap();
    assert_eq!(build_json["operation"], "build_new");
    assert_eq!(
        build_json
            .as_object()
            .unwrap()
            .keys()
            .cloned()
            .collect::<Vec<_>>(),
        ["binding", "operation", "root", "version"]
    );

    let info = cli()
        .args(["workbench", "info", root.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        info.status.success(),
        "{}",
        String::from_utf8_lossy(&info.stderr)
    );
    let info_json: Value = serde_json::from_slice(&info.stdout).unwrap();
    assert_eq!(
        info_json["binding"]["binding_sha256"],
        build_json["binding"]["binding_sha256"]
    );
    assert_eq!(
        info_json
            .as_object()
            .unwrap()
            .keys()
            .cloned()
            .collect::<Vec<_>>(),
        ["binding", "root", "version"]
    );

    let request_path = parent.join("request ü.json");
    fs::write(
        &request_path,
        br#"{"operation":"exact","spelling":"si'e","options":{}}"#,
    )
    .unwrap();
    let mut full_value = None;
    let mut compact_value = None;
    for format in ["full", "compact", "readable"] {
        let output = cli()
            .args([
                "workbench",
                "query",
                root.to_str().unwrap(),
                "--request",
                request_path.to_str().unwrap(),
                "--format",
                format,
            ])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "format={format}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        if format == "readable" {
            let text = String::from_utf8(output.stdout).unwrap();
            assert!(text.starts_with("workbench_binding: \""));
            assert!(text.contains("\nworkbench_selection: {"));
        } else {
            let value: Value = serde_json::from_slice(&output.stdout).unwrap();
            assert_eq!(value["selection"]["packs"], serde_json::json!(["llg"]));
            assert_eq!(
                value["binding_sha256"],
                build_json["binding"]["binding_sha256"]
            );
            if format == "full" {
                full_value = Some(value);
            } else {
                compact_value = Some(value);
            }
        }
        assert!(output.stderr.is_empty());
    }
    let compact_value = compact_value.expect("the compact response was captured");
    let full_value = full_value.expect("the Full response was captured");
    let detail_reference =
        compact_value["result"]["response"]["groups"][0]["hits"][0]["detail_ref"].clone();
    let reference_path = parent.join("reference ü.json");
    fs::write(
        &reference_path,
        serde_json::to_vec(&detail_reference).unwrap(),
    )
    .unwrap();
    let expected_binding = build_json["binding"]["binding_sha256"].as_str().unwrap();
    let valid_detail = cli()
        .args([
            "workbench",
            "detail",
            root.to_str().unwrap(),
            "--reference",
            reference_path.to_str().unwrap(),
            "--expected-binding",
            expected_binding,
            "--format",
            "full",
        ])
        .output()
        .unwrap();
    assert!(
        valid_detail.status.success(),
        "{}",
        String::from_utf8_lossy(&valid_detail.stderr)
    );
    let detail_value: Value = serde_json::from_slice(&valid_detail.stdout).unwrap();
    assert_eq!(
        detail_value["result"]["hit"],
        full_value["result"]["response"]["groups"][0]["hits"][0]
    );
    assert_eq!(
        detail_value["binding_sha256"],
        build_json["binding"]["binding_sha256"]
    );
    fs::write(
        &reference_path,
        serde_json::json!({
            "version": 1,
            "digest_policy": "entry-assertion-serde-json-v1",
            "role": "assertion",
            "source": "source-a",
            "entry_id": "entry-a",
            "assertion_id": "assertion-a",
            "occurrence": {
                "capture": {"namespace": "fixture:workbench", "sha256": "0".repeat(64)},
                "locator": "/0"
            },
            "entry_sha256": "1".repeat(64),
            "assertion_sha256": "2".repeat(64),
            "native_entry_id": null,
            "native_assertion_id": null,
            "contributor_id": null,
            "distributor_id": null
        })
        .to_string(),
    )
    .unwrap();
    let detail = cli()
        .args([
            "workbench",
            "detail",
            root.to_str().unwrap(),
            "--reference",
            reference_path.to_str().unwrap(),
            "--expected-binding",
            "not-a-digest",
        ])
        .output()
        .unwrap();
    assert_eq!(detail.status.code(), Some(1));
    assert!(detail.stdout.is_empty());
    let error: Value = serde_json::from_slice(&detail.stderr).unwrap();
    assert_eq!(error["error"]["code"], "invalid_workbench_binding");
    fs::remove_dir_all(parent).unwrap();
}

#[cfg(unix)]
#[test]
fn test_workbench_absolute_file_roots_survive_deleted_current_directory() {
    let parent = fresh_root();
    let catalogue = parent.join("catalogue");
    install(&catalogue);
    let selection_path = parent.join("selection.json");
    fs::write(
        &selection_path,
        serde_json::to_vec(&WorkbenchSelection {
            version: 1,
            packs: vec![PackId::Llg],
            disabled: Vec::new(),
            owned: None,
            contexts: Vec::new(),
        })
        .unwrap(),
    )
    .unwrap();
    let deleted_cwd = parent.join("empty-cwd");
    fs::create_dir(&deleted_cwd).unwrap();
    let root = parent.join("view-after-cwd-removal");
    let arguments = vec![
        OsString::from("workbench"),
        OsString::from("build"),
        root.as_os_str().to_owned(),
        OsString::from("--selection"),
        selection_path.as_os_str().to_owned(),
        OsString::from("--catalogue"),
        catalogue.as_os_str().to_owned(),
    ];
    let output = run_after_removing_cwd(&deleted_cwd, &arguments);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!deleted_cwd.exists());
    assert!(WorkbenchView::open(&root).is_ok());
    fs::remove_dir_all(parent).unwrap();
}

#[test]
fn test_workbench_build_collision_preserves_existing_sentinel() {
    let parent = fresh_root();
    let catalogue = parent.join("catalogue");
    install(&catalogue);
    let selection_path = parent.join("selection.json");
    fs::write(
        &selection_path,
        serde_json::to_vec(&WorkbenchSelection {
            version: 1,
            packs: vec![PackId::Llg],
            disabled: Vec::new(),
            owned: Some(WorkbenchOwnedRevision::Head),
            contexts: Vec::new(),
        })
        .unwrap(),
    )
    .unwrap();
    let root = parent.join("occupied");
    fs::create_dir(&root).unwrap();
    fs::write(root.join("sentinel"), b"leave this file byte-identical\n").unwrap();
    let output = cli()
        .args([
            "workbench",
            "build",
            root.to_str().unwrap(),
            "--selection",
            selection_path.to_str().unwrap(),
            "--owned",
            parent.join("missing journal").to_str().unwrap(),
            "--catalogue",
            catalogue.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    let error: Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(error["error"]["code"], "existing_workbench");
    assert_eq!(
        fs::read(root.join("sentinel")).unwrap(),
        b"leave this file byte-identical\n"
    );
    fs::remove_dir_all(parent).unwrap();
}

#[test]
fn test_workbench_build_rejects_strict_selection_before_creating_root() {
    let parent = fresh_root();
    let selection_path = parent.join("selection.json");
    fs::write(
        &selection_path,
        br#"{"version":1,"packs":[],"disabled":[],"owned":null,"contexts":[],"future":true}"#,
    )
    .unwrap();
    let root = parent.join("must-not-be-created");
    let output = cli()
        .args([
            "workbench",
            "build",
            root.to_str().unwrap(),
            "--selection",
            selection_path.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    let error: Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(error["error"]["code"], "invalid_workbench_selection");
    assert!(!root.exists());
    fs::remove_dir_all(parent).unwrap();
}

#[cfg(target_os = "linux")]
#[test]
fn test_workbench_build_accepts_non_utf8_root_and_selection_file_paths() {
    use std::os::unix::ffi::OsStringExt;

    let parent = fresh_root();
    let catalogue = parent.join("catalogue");
    install(&catalogue);
    let selection = WorkbenchSelection {
        version: 1,
        packs: vec![PackId::Llg],
        disabled: Vec::new(),
        owned: None,
        contexts: Vec::new(),
    };
    let selection_path = parent.join(std::ffi::OsString::from_vec(
        b"selection-\xff.json".to_vec(),
    ));
    fs::write(&selection_path, serde_json::to_vec(&selection).unwrap()).unwrap();
    let root = parent.join(std::ffi::OsString::from_vec(b"view-\xfe".to_vec()));
    let output = cli()
        .args(["workbench", "build"])
        .arg(&root)
        .arg("--selection")
        .arg(&selection_path)
        .arg("--catalogue")
        .arg(&catalogue)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["operation"], "build_new");
    let expected_hex: String = root
        .as_os_str()
        .as_encoded_bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    assert_eq!(report["root"]["encoding"], "unix-bytes");
    assert_eq!(report["root"]["hex"], expected_hex);
    assert!(WorkbenchView::open(&root).is_ok());
    fs::remove_dir_all(parent).unwrap();
}

#[cfg(target_os = "macos")]
#[test]
fn test_workbench_build_handles_non_utf8_root_according_to_filesystem_capability() {
    use std::os::unix::ffi::OsStringExt;

    let parent = fresh_root();
    let catalogue = parent.join("catalogue");
    install(&catalogue);
    let selection_path = parent.join("selection.json");
    fs::write(
        &selection_path,
        serde_json::to_vec(&WorkbenchSelection {
            version: 1,
            packs: vec![PackId::Llg],
            disabled: Vec::new(),
            owned: None,
            contexts: Vec::new(),
        })
        .unwrap(),
    )
    .unwrap();
    let raw_name = std::ffi::OsString::from_vec(b"view-\xff".to_vec());
    let capability_probe = parent.join(&raw_name);
    match fs::create_dir(&capability_probe) {
        Ok(()) => fs::remove_dir(&capability_probe).unwrap(),
        Err(error) if error.raw_os_error() == Some(92) => {
            let root = parent.join(&raw_name);
            let output = cli()
                .args(["workbench", "build"])
                .arg(&root)
                .arg("--selection")
                .arg(&selection_path)
                .arg("--catalogue")
                .arg(&catalogue)
                .output()
                .unwrap();
            assert_eq!(output.status.code(), Some(1));
            assert!(output.stdout.is_empty());
            let error: Value = serde_json::from_slice(&output.stderr).unwrap();
            assert_eq!(error["error"]["code"], "workbench_io");
            assert!(!root.exists());
            fs::remove_dir_all(parent).unwrap();
            return;
        }
        Err(error) => panic!("unexpected raw-byte capability probe failure: {error}"),
    }

    let root = parent.join(&raw_name);
    let output = cli()
        .args(["workbench", "build"])
        .arg(&root)
        .arg("--selection")
        .arg(&selection_path)
        .arg("--catalogue")
        .arg(&catalogue)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["root"]["encoding"], "unix-bytes");
    assert!(WorkbenchView::open(&root).is_ok());
    fs::remove_dir_all(parent).unwrap();
}

#[cfg(unix)]
#[test]
fn test_workbench_build_persists_before_other_or_broken_pipe_stdout_failure() {
    use std::{
        fs::File,
        os::fd::OwnedFd,
        os::unix::net::{UnixDatagram, UnixStream},
    };

    let parent = fresh_root();
    let catalogue = parent.join("catalogue");
    install(&catalogue);
    let selection_path = parent.join("selection.json");
    fs::write(
        &selection_path,
        serde_json::to_vec(&WorkbenchSelection {
            version: 1,
            packs: vec![PackId::Llg],
            disabled: Vec::new(),
            owned: None,
            contexts: Vec::new(),
        })
        .unwrap(),
    )
    .unwrap();

    // Fill the nonblocking datagram queue with one-byte packets so the real
    // child's stdout write, not only a setup send, is known to fail with an
    // ordinary Other-class error. Keep the unread peer alive through wait.
    let (sender, peer) = UnixDatagram::pair().unwrap();
    sender.set_nonblocking(true).unwrap();
    let fd: OwnedFd = sender.into();
    let mut full_file = File::from(fd);
    let mut sent = 0usize;
    let saturation_kind = loop {
        match full_file.write(b"x") {
            Ok(1) => {
                sent += 1;
                assert!(sent < 1_000_000, "datagram queue did not fill within bound");
            }
            Ok(_) => unreachable!("one-byte write must be complete or fail"),
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                break (error.kind(), error.raw_os_error());
            }
            #[cfg(target_os = "macos")]
            Err(error) if error.raw_os_error() == Some(55) => {
                break (error.kind(), error.raw_os_error());
            }
            Err(error) => panic!("unexpected datagram saturation error: {error}"),
        }
    };
    assert!(sent > 0);
    match full_file.write(b"x") {
        Err(error) => {
            assert_eq!(error.kind(), saturation_kind.0);
            assert_eq!(error.raw_os_error(), saturation_kind.1);
        }
        Ok(_) => panic!("preflight write unexpectedly fit after queue saturation"),
    }
    let other_root = parent.join("other-output-failure");
    let other = cli()
        .args([
            "workbench",
            "build",
            other_root.to_str().unwrap(),
            "--selection",
            selection_path.to_str().unwrap(),
            "--catalogue",
            catalogue.to_str().unwrap(),
        ])
        .stdout(Stdio::from(full_file))
        .output()
        .unwrap();
    drop(peer);
    assert_eq!(other.status.code(), Some(1));
    assert!(other.stdout.is_empty());
    let error: Value = serde_json::from_slice(&other.stderr).unwrap();
    assert_eq!(error["error"]["code"], "response_output_io");
    assert_eq!(error["error"]["sources"], serde_json::json!([]));
    assert!(WorkbenchView::open(&other_root).is_ok());

    let (writer, peer) = UnixStream::pair().unwrap();
    drop(peer);
    let mut broken_file = File::from(OwnedFd::from(writer));
    let preflight = broken_file
        .write(b"x")
        .expect_err("closed peer must reject a real write");
    assert_eq!(preflight.kind(), std::io::ErrorKind::BrokenPipe);
    let broken_root = parent.join("broken-pipe-output");
    let broken = cli()
        .args([
            "workbench",
            "build",
            broken_root.to_str().unwrap(),
            "--selection",
            selection_path.to_str().unwrap(),
            "--catalogue",
            catalogue.to_str().unwrap(),
        ])
        .stdout(Stdio::from(broken_file))
        .output()
        .unwrap();
    assert_eq!(broken.status.code(), Some(0));
    assert!(broken.stdout.is_empty());
    assert!(broken.stderr.is_empty());
    assert!(WorkbenchView::open(&broken_root).is_ok());
    fs::remove_dir_all(parent).unwrap();
}
