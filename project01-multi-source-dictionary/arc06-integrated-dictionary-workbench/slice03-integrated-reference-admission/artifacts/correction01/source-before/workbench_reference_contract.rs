//! Public CLI contracts for direct and configured typed references.

use std::{
    borrow::Cow,
    fs,
    path::PathBuf,
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};

#[cfg(unix)]
use std::{
    io::Write,
    os::unix::net::{UnixDatagram, UnixStream},
    os::{fd::OwnedFd, unix::ffi::OsStringExt},
    process::Stdio,
};

use serde_json::{Value, json};
use vlajmaji_lexicon::{Dictionary, PackId, PackPayloadKind, seal_release};
use vlajmaji_store::{
    CollectionCatalogue, CollectionInput, CollectionRequest, CollectionSelection,
    WorkbenchSelection,
};

static NEXT_ROOT: AtomicUsize = AtomicUsize::new(0);

fn fresh_root() -> PathBuf {
    for _ in 0..64 {
        let root = std::env::temp_dir().join(format!(
            "vlajmaji-s03-reference-cli-{}-{}",
            std::process::id(),
            NEXT_ROOT.fetch_add(1, Ordering::Relaxed)
        ));
        match fs::create_dir(&root) {
            Ok(()) => return root,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => panic!("create exclusive CLI fixture root: {error}"),
        }
    }
    panic!("could not allocate exclusive CLI fixture root after 64 attempts")
}

fn cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
}

fn install_synthetic_catalogue(root: &std::path::Path) {
    let dictionary = Dictionary::from_document(Default::default()).unwrap();
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

fn json_line(output: &Output) -> Value {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    assert_eq!(output.stdout.last(), Some(&b'\n'));
    serde_json::from_slice(&output.stdout).unwrap()
}

fn assert_exact_keys(value: &Value, keys: &[&str]) {
    let mut actual = value
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect::<Vec<_>>();
    actual.sort_unstable();
    assert_eq!(actual, keys);
}

#[cfg(unix)]
#[test]
#[ignore = "invoked as an isolated child after deleting its own working directory"]
fn test_workbench_profiles_reference_deleted_cwd_child() {
    let cwd = PathBuf::from(std::env::var_os("VLJ_REFERENCE_DELETED_CWD").unwrap());
    std::env::set_current_dir(&cwd).unwrap();
    fs::remove_dir(&cwd).unwrap();
    assert!(std::env::current_dir().is_err());

    let config = std::env::var_os("VLJ_REFERENCE_DELETED_CONFIG").unwrap();
    let target = std::env::var_os("VLJ_REFERENCE_DELETED_TARGET").unwrap();
    let prepared = std::env::var_os("VLJ_REFERENCE_DELETED_PREPARED").unwrap();
    let cli_path = PathBuf::from(env!("CARGO_BIN_EXE_vlajmaji"));
    match std::env::var("VLJ_REFERENCE_DELETED_MODE").as_deref() {
        Ok("absolute-reference") => {
            assert!(PathBuf::from(&config).is_absolute());
            assert!(PathBuf::from(&prepared).is_absolute());
            let output = Command::new(cli_path)
                .args(["workbench", "profiles", "reference", "--config"])
                .arg(config)
                .args(["--profile", "alpha", "--target"])
                .arg(&target)
                .args(["--format", "full"])
                .output()
                .unwrap();
            let response = json_line(&output);
            assert_eq!(response["configuration"]["config_revision"], 2);
            assert_eq!(response["result"]["status"], "Unknown");

            let direct = Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
                .args(["workbench", "reference"])
                .arg(prepared)
                .arg("--target")
                .arg(target)
                .args(["--format", "full"])
                .output()
                .unwrap();
            assert_eq!(json_line(&direct)["result"]["status"], "Unknown");
        }
        Ok("relative-config") => {
            assert!(!PathBuf::from(&config).is_absolute());
            let output = Command::new(cli_path)
                .args(["workbench", "profiles", "reference", "--config"])
                .arg(config)
                .args(["--profile", "alpha", "--target"])
                .arg(target)
                .args(["--format", "full"])
                .output()
                .unwrap();
            assert_eq!(output.status.code(), Some(1));
            assert!(output.stdout.is_empty());
            let error: Value = serde_json::from_slice(&output.stderr).unwrap();
            assert_eq!(error["error"]["code"], "workbench_config_io");
            assert_eq!(error["error"]["sources"], json!([]));
        }
        mode => panic!("unexpected deleted-cwd child mode: {mode:?}"),
    }
}

#[cfg(unix)]
fn run_profiles_after_child_deletes_cwd(
    mode: &str,
    cwd: &std::path::Path,
    config: &std::path::Path,
    target: &std::path::Path,
    prepared: &std::path::Path,
) {
    let output = Command::new(std::env::current_exe().unwrap())
        .args([
            "--ignored",
            "--exact",
            "test_workbench_profiles_reference_deleted_cwd_child",
            "--nocapture",
        ])
        .env("VLJ_REFERENCE_DELETED_MODE", mode)
        .env("VLJ_REFERENCE_DELETED_CWD", cwd)
        .env("VLJ_REFERENCE_DELETED_CONFIG", config)
        .env("VLJ_REFERENCE_DELETED_TARGET", target)
        .env("VLJ_REFERENCE_DELETED_PREPARED", prepared)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "deleted-cwd child {mode} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[cfg(unix)]
fn saturated_stdout() -> (std::fs::File, UnixDatagram) {
    let (sender, peer) = UnixDatagram::pair().unwrap();
    sender.set_nonblocking(true).unwrap();
    let mut file = std::fs::File::from(OwnedFd::from(sender));
    let mut writes = 0usize;
    loop {
        match file.write(b"x") {
            Ok(1) => {
                writes += 1;
                assert!(writes < 1_000_000, "datagram queue did not saturate");
            }
            Ok(_) => unreachable!("single byte write must be complete or fail"),
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => break,
            #[cfg(target_os = "macos")]
            Err(error) if error.raw_os_error() == Some(55) => break,
            Err(error) => panic!("unexpected saturation error: {error}"),
        }
    }
    assert!(writes > 0);
    (file, peer)
}

#[cfg(unix)]
fn broken_pipe_stdout() -> std::fs::File {
    let (peer, writer) = UnixStream::pair().unwrap();
    drop(peer);
    let mut file = std::fs::File::from(OwnedFd::from(writer));
    let error = file
        .write(b"x")
        .expect_err("closed peer must reject a write");
    assert_eq!(error.kind(), std::io::ErrorKind::BrokenPipe);
    file
}

#[test]
fn test_workbench_reference_direct_and_profile_formats_keep_distinct_wire_shapes() {
    let root = fresh_root();
    let catalogue = root.join("catalogue");
    install_synthetic_catalogue(&catalogue);
    let selection = root.join("selection ü.json");
    fs::write(
        &selection,
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
    let direct_root = root.join("direct");
    let built = cli()
        .args(["workbench", "build"])
        .arg(&direct_root)
        .arg("--catalogue")
        .arg(&catalogue)
        .arg("--selection")
        .arg(&selection)
        .output()
        .unwrap();
    json_line(&built);

    let target = root.join("reference target ü.json");
    fs::write(&target, br#""Unknown""#).unwrap();
    let target_bytes = fs::read(&target).unwrap();
    let direct_binding_bytes = fs::read(direct_root.join("binding.json")).unwrap();
    let direct_dictionary_bytes =
        fs::read(direct_root.join("dictionary/dictionary.sqlite3")).unwrap();
    let direct_full = cli()
        .args(["workbench", "reference"])
        .arg(&direct_root)
        .arg("--target")
        .arg(&target)
        .arg("--format")
        .arg("full")
        .output()
        .unwrap();
    let full = json_line(&direct_full);
    assert_exact_keys(
        &full,
        &[
            "binding_sha256",
            "format",
            "kind",
            "result",
            "selection",
            "version",
        ],
    );
    assert_eq!(full["kind"], "vlajmaji-workbench-reference");
    assert_eq!(full["format"], "full");
    assert_eq!(full["result"]["target"], "Unknown");
    assert_eq!(full["result"]["status"], "Unknown");
    assert_eq!(full["result"]["payload"], Value::Null);
    assert_eq!(full["result"]["catalogue_matches"], json!([]));
    assert_eq!(full["result"]["registrations"], json!([]));

    let direct_compact = cli()
        .args(["workbench", "reference"])
        .arg(&direct_root)
        .arg("--target")
        .arg(&target)
        .arg("--format")
        .arg("compact")
        .output()
        .unwrap();
    let compact = json_line(&direct_compact);
    assert_exact_keys(
        &compact,
        &[
            "binding_sha256",
            "format",
            "kind",
            "result",
            "selection",
            "version",
        ],
    );
    assert_eq!(compact["format"], "compact");
    assert_eq!(compact["result"]["status"], "Unknown");

    let readable = cli()
        .args(["workbench", "reference"])
        .arg(&direct_root)
        .arg("--target")
        .arg(&target)
        .arg("--format")
        .arg("readable")
        .output()
        .unwrap();
    assert!(readable.status.success());
    let readable = String::from_utf8(readable.stdout).unwrap();
    for label in [
        "workbench_binding: ",
        "workbench_selection: ",
        "reference_target: ",
        "reference_status: ",
        "reference_operational_reason: ",
        "reference_catalogue_matches: ",
        "reference_registrations: ",
        "reference_requested_release: ",
    ] {
        assert!(
            readable.contains(label),
            "missing {label:?} in {readable:?}"
        );
    }

    #[cfg(unix)]
    {
        let (saturated, peer) = saturated_stdout();
        let other_io = cli()
            .args(["workbench", "reference"])
            .arg(&direct_root)
            .arg("--target")
            .arg(&target)
            .stdout(Stdio::from(saturated))
            .output()
            .unwrap();
        drop(peer);
        assert_eq!(other_io.status.code(), Some(1));
        assert!(other_io.stdout.is_empty());
        let error: Value = serde_json::from_slice(&other_io.stderr).unwrap();
        assert_eq!(error["error"]["code"], "response_output_io");
        assert_eq!(error["error"]["sources"], json!([]));
        assert!(
            !error["error"]["message"]
                .as_str()
                .unwrap()
                .contains("publication")
        );

        let broken_pipe = cli()
            .args(["workbench", "reference"])
            .arg(&direct_root)
            .arg("--target")
            .arg(&target)
            .stdout(Stdio::from(broken_pipe_stdout()))
            .output()
            .unwrap();
        assert_eq!(broken_pipe.status.code(), Some(0));
        assert!(broken_pipe.stdout.is_empty());
        assert!(broken_pipe.stderr.is_empty());
        assert_eq!(
            fs::read(direct_root.join("binding.json")).unwrap(),
            direct_binding_bytes
        );
        assert_eq!(
            fs::read(direct_root.join("dictionary/dictionary.sqlite3")).unwrap(),
            direct_dictionary_bytes
        );
    }

    let config = root.join("profiles.json");
    let configure = cli()
        .args(["workbench", "profiles", "configure", "--config"])
        .arg(&config)
        .args(["--name", "alpha", "--catalogue"])
        .arg(&catalogue)
        .arg("--selection")
        .arg(&selection)
        .arg("--make-default")
        .output()
        .unwrap();
    assert_eq!(json_line(&configure)["revision"], 1);
    let prepared = root.join("prepared");
    let refresh = cli()
        .args(["workbench", "profiles", "refresh"])
        .arg(&prepared)
        .arg("--config")
        .arg(&config)
        .args(["--profile", "alpha", "--expected-revision", "1"])
        .output()
        .unwrap();
    assert_eq!(json_line(&refresh)["publication"]["revision"], 2);

    let config_bytes = fs::read(&config).unwrap();
    let prepared_binding_bytes = fs::read(prepared.join("binding.json")).unwrap();
    let prepared_dictionary_bytes =
        fs::read(prepared.join("dictionary/dictionary.sqlite3")).unwrap();
    let configured_full = cli()
        .args(["workbench", "profiles", "reference", "--config"])
        .arg(&config)
        .args(["--profile", "alpha", "--target"])
        .arg(&target)
        .arg("--format")
        .arg("full")
        .output()
        .unwrap();
    let configured_full = json_line(&configured_full);
    assert_exact_keys(
        &configured_full,
        &[
            "binding_sha256",
            "configuration",
            "format",
            "kind",
            "result",
            "selection",
            "version",
        ],
    );
    assert_eq!(configured_full["result"]["status"], "Unknown");

    let configured_compact = cli()
        .args(["workbench", "profiles", "reference", "--config"])
        .arg(&config)
        .args(["--profile", "alpha", "--target"])
        .arg(&target)
        .arg("--format")
        .arg("compact")
        .output()
        .unwrap();
    let configured_compact = json_line(&configured_compact);
    assert_exact_keys(
        &configured_compact,
        &[
            "binding_sha256",
            "configuration",
            "format",
            "kind",
            "result",
            "selection",
            "version",
        ],
    );
    assert_eq!(configured_compact["format"], "compact");

    let configured_readable = cli()
        .args(["workbench", "profiles", "reference", "--config"])
        .arg(&config)
        .args(["--profile", "alpha", "--target"])
        .arg(&target)
        .arg("--format")
        .arg("readable")
        .output()
        .unwrap();
    assert!(configured_readable.status.success());
    let configured_readable = String::from_utf8(configured_readable.stdout).unwrap();
    assert!(configured_readable.contains("workbench_configuration: "));
    assert!(configured_readable.contains("reference_requested_release: "));
    assert_eq!(fs::read(&target).unwrap(), target_bytes);
    assert_eq!(fs::read(&config).unwrap(), config_bytes);
    assert_eq!(
        fs::read(prepared.join("binding.json")).unwrap(),
        prepared_binding_bytes
    );
    assert_eq!(
        fs::read(prepared.join("dictionary/dictionary.sqlite3")).unwrap(),
        prepared_dictionary_bytes
    );
    assert!(prepared.is_dir());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn test_workbench_reference_missing_target_reports_existing_io_category() {
    let output = cli()
        .args([
            "workbench",
            "reference",
            "missing-root",
            "--target",
            "missing-target.json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    let error: Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(error["error"]["code"], "workbench_io");
    assert_eq!(error["error"]["sources"], json!([]));

    let root = fresh_root();
    let malformed_target = root.join("malformed-target.json");
    let malformed_bytes = b"{\"Local\":\n";
    fs::write(&malformed_target, malformed_bytes).unwrap();
    let malformed = cli()
        .args(["workbench", "reference"])
        .arg(root.join("missing-workbench"))
        .arg("--target")
        .arg(&malformed_target)
        .output()
        .unwrap();
    assert_eq!(malformed.status.code(), Some(1));
    assert!(malformed.stdout.is_empty());
    let error: Value = serde_json::from_slice(&malformed.stderr).unwrap();
    assert_eq!(error["error"]["code"], "request_json");
    assert_eq!(error["error"]["sources"], json!([]));
    assert_eq!(fs::read(&malformed_target).unwrap(), malformed_bytes);
    assert!(!root.join("missing-workbench").exists());
    fs::remove_dir_all(root).unwrap();
}

#[cfg(unix)]
#[test]
fn test_workbench_profiles_reference_absolute_config_survives_deleted_cwd() {
    let root = fresh_root();
    let config = root.join("profiles.json");
    let catalogue = root.join("catalogue");
    install_synthetic_catalogue(&catalogue);
    let selection = root.join("selection.json");
    fs::write(
        &selection,
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
    let configured = cli()
        .args(["workbench", "profiles", "configure", "--config"])
        .arg(&config)
        .args(["--name", "alpha", "--catalogue"])
        .arg(&catalogue)
        .arg("--selection")
        .arg(&selection)
        .arg("--make-default")
        .output()
        .unwrap();
    assert_eq!(json_line(&configured)["revision"], 1);
    let prepared = root.join("prepared");
    let refreshed = cli()
        .args(["workbench", "profiles", "refresh"])
        .arg(&prepared)
        .arg("--config")
        .arg(&config)
        .args(["--profile", "alpha", "--expected-revision", "1"])
        .output()
        .unwrap();
    assert_eq!(json_line(&refreshed)["publication"]["revision"], 2);
    let target = root.join("unknown-target.json");
    fs::write(&target, br#""Unknown""#).unwrap();
    let original_config = fs::read(&config).unwrap();
    for mode in ["absolute-reference", "relative-config"] {
        let cwd = root.join(format!("deleted-cwd-{mode}"));
        fs::create_dir(&cwd).unwrap();
        let child_config = if mode == "relative-config" {
            PathBuf::from("profiles.json")
        } else {
            config.clone()
        };
        run_profiles_after_child_deletes_cwd(mode, &cwd, &child_config, &target, &prepared);
        assert!(!cwd.exists());
        assert_eq!(fs::read(&config).unwrap(), original_config);
    }
    fs::remove_dir_all(root).unwrap();
}

#[cfg(unix)]
#[test]
fn test_workbench_reference_target_uses_native_file_argument_when_supported() {
    use std::{ffi::OsString, fs::OpenOptions};

    let root = fresh_root();
    let catalogue = root.join("catalogue");
    install_synthetic_catalogue(&catalogue);
    let selection = root.join("selection.json");
    fs::write(
        &selection,
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
    let direct_root = root.join("direct");
    let built = cli()
        .args(["workbench", "build"])
        .arg(&direct_root)
        .arg("--catalogue")
        .arg(&catalogue)
        .arg("--selection")
        .arg(&selection)
        .output()
        .unwrap();
    json_line(&built);

    let target = root.join(OsString::from_vec(b"reference-\xff.json".to_vec()));
    let probe = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&target);
    match probe {
        Ok(file) => {
            drop(file);
            fs::write(&target, br#""Unknown""#).unwrap();
            let output = cli()
                .args(["workbench", "reference"])
                .arg(&direct_root)
                .arg("--target")
                .arg(target.as_os_str())
                .output()
                .unwrap();
            let response = json_line(&output);
            assert_eq!(response["result"]["status"], "Unknown");
            fs::remove_dir_all(root).unwrap();
        }
        Err(error) if cfg!(target_os = "macos") && error.raw_os_error() == Some(92) => {
            eprintln!("raw-byte reference target unsupported here: macOS EILSEQ 92");
            let output = cli()
                .args(["workbench", "reference"])
                .arg(&direct_root)
                .arg("--target")
                .arg(target.as_os_str())
                .output()
                .unwrap();
            assert_eq!(output.status.code(), Some(1));
            assert!(output.stdout.is_empty());
            let error: Value = serde_json::from_slice(&output.stderr).unwrap();
            assert_eq!(error["error"]["code"], "workbench_io");
            assert_eq!(error["error"]["sources"], json!([]));
            fs::remove_dir_all(root).unwrap();
        }
        Err(error) => panic!("raw-byte reference-target probe failed: {error}"),
    }
}
