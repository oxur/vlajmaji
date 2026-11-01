//! Public command-line contracts for explicit workbench profiles.

use std::{
    borrow::Cow,
    fs,
    path::PathBuf,
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};

#[cfg(unix)]
use std::{
    ffi::OsString,
    io::Write,
    os::fd::OwnedFd,
    os::unix::ffi::{OsStrExt, OsStringExt},
    os::unix::net::{UnixDatagram, UnixStream},
    process::Stdio,
};

use serde_json::{Value, json};
use vlajmaji_lexicon::{Dictionary, PackId, PackPayloadKind, seal_release};
use vlajmaji_store::{
    CollectionCatalogue, CollectionInput, CollectionRequest, CollectionSelection, NativePathReport,
    WorkbenchProfileConfigureRequest, WorkbenchProfileError, WorkbenchProfiles,
    WorkbenchPublicationContext, WorkbenchSelection,
};

static NEXT_ROOT: AtomicUsize = AtomicUsize::new(0);

fn fresh_root() -> PathBuf {
    for _ in 0..64 {
        let root = std::env::temp_dir().join(format!(
            "vlajmaji-workbench-profiles-cli-{}-{}",
            std::process::id(),
            NEXT_ROOT.fetch_add(1, Ordering::Relaxed)
        ));
        match fs::create_dir(&root) {
            Ok(()) => return root,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => panic!("create exclusive CLI fixture root: {error}"),
        }
    }
    panic!("could not allocate an exclusive CLI fixture root after 64 attempts")
}

fn cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
}

#[cfg(unix)]
#[test]
#[ignore = "invoked as an isolated child after deleting its own working directory"]
fn test_profiles_deleted_cwd_child() {
    let cwd = PathBuf::from(std::env::var_os("VLJ_PROFILES_DELETED_CWD").unwrap());
    std::env::set_current_dir(&cwd).unwrap();
    fs::remove_dir(&cwd).unwrap();
    assert!(std::env::current_dir().is_err());

    let config = std::env::var_os("VLJ_PROFILES_DELETED_CONFIG").unwrap();
    let cli_path = PathBuf::from(env!("CARGO_BIN_EXE_vlajmaji"));
    match std::env::var("VLJ_PROFILES_DELETED_MODE").as_deref() {
        Ok("absolute-info") => {
            assert!(PathBuf::from(&config).is_absolute());
            let output = Command::new(cli_path)
                .args(["workbench", "profiles", "info", "--config"])
                .arg(config)
                .output()
                .unwrap();
            let info = assert_json_line(&output);
            assert_eq!(info["revision"], 1);
            assert_eq!(info["default_profile"], "alpha");
        }
        Ok("relative-config") => {
            let catalogue = std::env::var_os("VLJ_PROFILES_DELETED_CATALOGUE").unwrap();
            let selection = std::env::var_os("VLJ_PROFILES_DELETED_SELECTION").unwrap();
            assert!(!PathBuf::from(&config).is_absolute());
            assert!(PathBuf::from(&catalogue).is_absolute());
            assert!(PathBuf::from(&selection).is_absolute());
            let output = Command::new(cli_path)
                .args(["workbench", "profiles", "configure", "--config"])
                .arg(config)
                .args(["--name", "beta", "--catalogue"])
                .arg(catalogue)
                .args(["--selection"])
                .arg(selection)
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
    catalogue: &std::path::Path,
    selection: &std::path::Path,
) -> Output {
    let output = Command::new(std::env::current_exe().unwrap())
        .args([
            "--ignored",
            "--exact",
            "test_profiles_deleted_cwd_child",
            "--nocapture",
        ])
        .env("VLJ_PROFILES_DELETED_MODE", mode)
        .env("VLJ_PROFILES_DELETED_CWD", cwd)
        .env("VLJ_PROFILES_DELETED_CONFIG", config)
        .env("VLJ_PROFILES_DELETED_CATALOGUE", catalogue)
        .env("VLJ_PROFILES_DELETED_SELECTION", selection)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "deleted-cwd child {mode} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

#[cfg(unix)]
#[test]
fn test_profiles_explicit_absolute_config_survives_deleted_cwd_and_relative_config_fails() {
    let root = fresh_root();
    let config = root.join("profiles.json");
    let catalogue = root.join("catalogue");
    install_synthetic_catalogue(&catalogue);
    let selection = root.join("selection.json");
    fs::write(
        &selection,
        br#"{"version":1,"packs":["llg"],"disabled":[],"owned":null,"contexts":[]}"#,
    )
    .unwrap();
    let initial = cli()
        .args(["workbench", "profiles", "configure", "--config"])
        .arg(&config)
        .args(["--name", "alpha", "--catalogue"])
        .arg(&catalogue)
        .args(["--selection"])
        .arg(&selection)
        .args(["--make-default"])
        .output()
        .unwrap();
    assert_json_line(&initial);
    let config_before = fs::read(&config).unwrap();

    let absolute_cwd = root.join("absolute-info-cwd");
    let relative_cwd = root.join("relative-input-cwd");
    fs::create_dir(&absolute_cwd).unwrap();
    fs::create_dir(&relative_cwd).unwrap();
    run_profiles_after_child_deletes_cwd(
        "absolute-info",
        &absolute_cwd,
        &config,
        &catalogue,
        &selection,
    );
    assert!(!absolute_cwd.exists());
    run_profiles_after_child_deletes_cwd(
        "relative-config",
        &relative_cwd,
        std::path::Path::new("profiles.json"),
        &catalogue,
        &selection,
    );
    assert!(!relative_cwd.exists());
    assert_eq!(fs::read(&config).unwrap(), config_before);
    fs::remove_dir_all(root).unwrap();
}

#[cfg(unix)]
#[test]
fn test_profiles_raw_byte_config_path_uses_native_argv_when_filesystem_supports_it() {
    use std::fs::OpenOptions;

    let root = fresh_root();
    let raw_name = OsString::from_vec(b"profiles-\xff.json".to_vec());
    let config = root.join(raw_name);
    let expected_config_hex = config
        .as_os_str()
        .as_bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let probe = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&config);
    match probe {
        Ok(file) => {
            drop(file);
            fs::remove_file(&config).unwrap();
        }
        Err(error) if cfg!(target_os = "macos") && error.raw_os_error() == Some(92) => {
            eprintln!("raw-byte profile config path unsupported here: macOS EILSEQ 92");
            let output = cli()
                .args(["workbench", "profiles", "info", "--config"])
                .arg(config.as_os_str())
                .output()
                .unwrap();
            assert_eq!(output.status.code(), Some(1));
            assert!(output.stdout.is_empty());
            let error: Value = serde_json::from_slice(&output.stderr).unwrap();
            assert_eq!(error["error"]["code"], "workbench_config_missing");
            assert_eq!(error["error"]["sources"], json!([]));
            fs::remove_dir_all(root).unwrap();
            return;
        }
        Err(error) => panic!("raw-byte config probe failed unexpectedly: {error}"),
    }

    let catalogue = root.join("catalogue");
    install_synthetic_catalogue(&catalogue);
    let selection = root.join("selection.json");
    fs::write(
        &selection,
        br#"{"version":1,"packs":["llg"],"disabled":[],"owned":null,"contexts":[]}"#,
    )
    .unwrap();
    let configured = cli()
        .args(["workbench", "profiles", "configure", "--config"])
        .arg(config.as_os_str())
        .args(["--name", "alpha", "--catalogue"])
        .arg(&catalogue)
        .args(["--selection"])
        .arg(&selection)
        .args(["--make-default"])
        .output()
        .unwrap();
    let configured = assert_json_line(&configured);
    assert_eq!(configured["config"]["encoding"], "unix_bytes");
    assert_eq!(configured["config"]["hex"], expected_config_hex);
    assert_eq!(configured["config"]["utf8"], Value::Null);

    let info = cli()
        .args(["workbench", "profiles", "info", "--config"])
        .arg(config.as_os_str())
        .output()
        .unwrap();
    let info = assert_json_line(&info);
    assert_eq!(info["revision"], 1);
    assert_eq!(info["default_profile"], "alpha");
    assert_eq!(info["config"]["hex"], configured["config"]["hex"]);
    fs::remove_dir_all(root).unwrap();
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

#[cfg(unix)]
fn saturated_stdout() -> (std::fs::File, UnixDatagram) {
    let (sender, peer) = UnixDatagram::pair().unwrap();
    sender.set_nonblocking(true).unwrap();
    let fd: OwnedFd = sender.into();
    let mut file = std::fs::File::from(fd);
    let mut sent = 0usize;
    let saturation = loop {
        match file.write(b"x") {
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
    match file.write(b"x") {
        Err(error) => {
            assert_eq!(error.kind(), saturation.0);
            assert_eq!(error.raw_os_error(), saturation.1);
        }
        Ok(_) => panic!("preflight write unexpectedly fit after queue saturation"),
    }
    (file, peer)
}

#[cfg(unix)]
fn broken_pipe_stdout() -> std::fs::File {
    let (peer, writer) = UnixStream::pair().unwrap();
    drop(peer);
    let fd: OwnedFd = writer.into();
    let mut file = std::fs::File::from(fd);
    let error = file
        .write(b"x")
        .expect_err("closed peer must reject a real write");
    assert_eq!(error.kind(), std::io::ErrorKind::BrokenPipe);
    file
}

fn assert_json_line(output: &Output) -> Value {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    assert_eq!(output.stdout.last(), Some(&b'\n'));
    assert_ne!(
        output.stdout.get(output.stdout.len().saturating_sub(2)),
        Some(&b'\n')
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

fn assert_keys(value: &Value, expected: &[&str]) {
    let mut actual = value
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect::<Vec<_>>();
    actual.sort_unstable();
    assert_eq!(actual, expected);
}

#[test]
fn test_publication_context_display_retains_full_public_native_context_and_cause() {
    let context = WorkbenchPublicationContext {
        config: NativePathReport {
            encoding: "unix_bytes".into(),
            hex: "ff".into(),
            utf8: None,
        },
        prior_revision: Some(4),
        prior_sha256: Some("prior-digest".into()),
        proposed_revision: 5,
        proposed_sha256: "proposed-digest".into(),
        observed_revision: None,
        observed_sha256: Some("observed-digest".into()),
        prepared: None,
    };
    let error = WorkbenchProfileError::Indeterminate {
        publication: Box::new(context.clone()),
        source: Box::new(WorkbenchProfileError::RevisionOverflow(5)),
    };
    let message = error.to_string();
    assert!(message.contains(&format!("{context:?}")));
    assert!(message.contains("revision 5 cannot be incremented"));
}

#[test]
fn test_profiles_configure_info_status_and_unprepared_query_have_explicit_contracts() {
    let root = fresh_root();
    let config = root.join("profiles ü.json");
    let catalogue = root.join("catalogue");
    install_synthetic_catalogue(&catalogue);
    let selection = root.join("selection ü.json");
    let request = root.join("request ü.json");
    fs::write(
        &selection,
        br#"{"version":1,"packs":["llg"],"disabled":[],"owned":null,"contexts":[]}"#,
    )
    .unwrap();
    fs::write(
        &request,
        br#"{"operation":"exact","spelling":"farvi","options":{}}"#,
    )
    .unwrap();

    let configured = cli()
        .args([
            "workbench",
            "profiles",
            "configure",
            "--config",
            config.to_str().unwrap(),
            "--name",
            "alpha",
            "--catalogue",
            catalogue.to_str().unwrap(),
            "--selection",
            selection.to_str().unwrap(),
            "--make-default",
        ])
        .output()
        .unwrap();
    let configured = assert_json_line(&configured);
    assert_keys(
        &configured,
        &[
            "active",
            "changed",
            "config",
            "default_profile",
            "operation",
            "prior_revision",
            "profile",
            "revision",
            "sha256",
            "version",
        ],
    );
    assert_eq!(configured["operation"], "configure");
    assert_eq!(configured["changed"], true);
    assert_eq!(configured["prior_revision"], Value::Null);
    assert_eq!(configured["revision"], 1);
    assert_eq!(configured["profile"], "alpha");
    assert_eq!(configured["default_profile"], "alpha");
    assert_eq!(configured["active"], Value::Null);

    let info = cli()
        .args([
            "workbench",
            "profiles",
            "info",
            "--config",
            config.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    let info = assert_json_line(&info);
    assert_keys(
        &info,
        &[
            "config",
            "default_profile",
            "kind",
            "prepared",
            "profiles",
            "revision",
            "sha256",
            "version",
        ],
    );
    assert_eq!(info["revision"], 1);
    assert_eq!(info["default_profile"], "alpha");
    assert_eq!(info["prepared"], json!([]));
    assert_eq!(info["profiles"].as_array().unwrap().len(), 1);
    assert_eq!(info["profiles"][0]["name"], "alpha");
    assert_eq!(info["profiles"][0]["active_generation"], Value::Null);

    let status = cli()
        .args([
            "workbench",
            "profiles",
            "status",
            "--config",
            config.to_str().unwrap(),
            "--profile",
            "alpha",
        ])
        .output()
        .unwrap();
    let status = assert_json_line(&status);
    assert_eq!(status["config_revision"], 1);
    assert_eq!(status["requested_profile"], "alpha");
    assert_eq!(status["profile"]["name"], "alpha");
    assert_eq!(status["active"], Value::Null);
    assert_eq!(status["catalogue"]["packs"][0]["pack_id"], "llg");
    assert_eq!(status["catalogue"]["packs"][0]["selected"], true);
    assert_eq!(status["owned"], Value::Null);

    let query = cli()
        .args([
            "workbench",
            "profiles",
            "query",
            "--config",
            config.to_str().unwrap(),
            "--request",
            request.to_str().unwrap(),
            "--format",
            "full",
        ])
        .output()
        .unwrap();
    assert_eq!(query.status.code(), Some(1));
    assert!(query.stdout.is_empty());
    assert_eq!(query.stderr.last(), Some(&b'\n'));
    let error: Value = serde_json::from_slice(&query.stderr).unwrap();
    assert_keys(&error, &["error"]);
    assert_eq!(error["error"]["code"], "unprepared_workbench_profile");
    assert_eq!(error["error"]["sources"], json!([]));

    let wrong_kind = root.join("wrong-kind.json");
    fs::write(
        &wrong_kind,
        br#"{"version":1,"kind":"not-workbench-profiles","revision":1,"default_profile":null,"profiles":[],"prepared":[]}"#,
    )
    .unwrap();
    let invalid = cli()
        .args([
            "workbench",
            "profiles",
            "info",
            "--config",
            wrong_kind.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_eq!(invalid.status.code(), Some(1));
    assert!(invalid.stdout.is_empty());
    let invalid_error: Value = serde_json::from_slice(&invalid.stderr).unwrap();
    assert_eq!(invalid_error["error"]["code"], "invalid_workbench_config");
    assert_eq!(invalid_error["error"]["sources"], json!([]));
    fs::remove_dir_all(root).unwrap();
}

#[cfg(unix)]
#[test]
fn test_profiles_noop_and_refresh_other_stdout_errors_report_durable_state() {
    let root = fresh_root();
    let catalogue = root.join("catalogue");
    install_synthetic_catalogue(&catalogue);
    let config = root.join("profiles.json");
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
    let configure = || WorkbenchProfileConfigureRequest {
        name: "alpha".into(),
        catalogue_root: Some(catalogue.clone()),
        owned_root: None,
        selection: WorkbenchSelection {
            version: 1,
            packs: vec![PackId::Llg],
            disabled: Vec::new(),
            owned: None,
            contexts: Vec::new(),
        },
        make_default: true,
    };
    let first = WorkbenchProfiles::configure(&config, configure(), None).unwrap();
    let original_bytes = fs::read(&config).unwrap();
    assert_eq!(first.revision, 1);

    let (full_file, peer) = saturated_stdout();
    let noop = cli()
        .args([
            "workbench",
            "profiles",
            "configure",
            "--config",
            config.to_str().unwrap(),
            "--name",
            "alpha",
            "--selection",
            selection.to_str().unwrap(),
            "--catalogue",
            catalogue.to_str().unwrap(),
            "--make-default",
            "--expected-revision",
            "1",
        ])
        .stdout(Stdio::from(full_file))
        .output()
        .unwrap();
    drop(peer);
    assert_eq!(noop.status.code(), Some(1));
    assert!(noop.stdout.is_empty());
    let noop_error: Value = serde_json::from_slice(&noop.stderr).unwrap();
    assert_eq!(noop_error["error"]["code"], "response_output_io");
    let noop_message = noop_error["error"]["message"].as_str().unwrap();
    let noop_context: Value = serde_json::from_str(
        noop_message
            .split_once("workbench profile publication context: ")
            .unwrap()
            .1,
    )
    .unwrap();
    assert_eq!(
        noop_context["config"],
        json!(vlajmaji_store::native_path_report(&config))
    );
    assert_eq!(noop_context["profile"], "alpha");
    assert_eq!(noop_context["destination"], Value::Null);
    let noop_publication = noop_context["publication"].as_str().unwrap();
    assert!(
        noop_publication
            .starts_with("unchanged; changed=false; prior_revision=Some(1); revision=1; sha256=")
    );
    assert!(!noop_publication.contains("published"));
    assert_eq!(fs::read(&config).unwrap(), original_bytes);

    let destination = root.join("alpha-root");
    let (full_file, peer) = saturated_stdout();
    let refresh = cli()
        .args([
            "workbench",
            "profiles",
            "refresh",
            destination.to_str().unwrap(),
            "--config",
            config.to_str().unwrap(),
            "--expected-revision",
            "1",
        ])
        .stdout(Stdio::from(full_file))
        .output()
        .unwrap();
    drop(peer);
    assert_eq!(refresh.status.code(), Some(1));
    assert!(refresh.stdout.is_empty());
    let refresh_error: Value = serde_json::from_slice(&refresh.stderr).unwrap();
    assert_eq!(refresh_error["error"]["code"], "response_output_io");
    let refresh_message = refresh_error["error"]["message"].as_str().unwrap();
    let refresh_context: Value = serde_json::from_str(
        refresh_message
            .split_once("workbench profile publication context: ")
            .unwrap()
            .1,
    )
    .unwrap();
    assert_eq!(
        refresh_context["config"],
        json!(vlajmaji_store::native_path_report(&config))
    );
    assert_eq!(refresh_context["profile"], Value::Null);
    assert_eq!(
        refresh_context["destination"],
        json!(vlajmaji_store::native_path_report(&destination))
    );
    let publication = refresh_context["publication"].as_str().unwrap();
    assert!(
        publication
            .starts_with("published; changed=true; prior_revision=Some(1); revision=2; sha256=")
    );
    assert!(publication.contains("active=Some(WorkbenchPrepared"));
    assert!(destination.exists());
    let info = WorkbenchProfiles::read(&config).unwrap().report();
    assert_eq!(info.revision, 2);
    assert_eq!(info.prepared.len(), 1);
    assert_eq!(info.prepared[0].profile, "alpha");
    assert_eq!(
        info.profiles[0].active_generation,
        Some(info.prepared[0].sequence)
    );

    let broken_pipe = cli()
        .args([
            "workbench",
            "profiles",
            "default",
            "--config",
            config.to_str().unwrap(),
            "--clear",
            "--expected-revision",
            "2",
        ])
        .stdout(Stdio::from(broken_pipe_stdout()))
        .output()
        .unwrap();
    assert_eq!(broken_pipe.status.code(), Some(0));
    assert!(broken_pipe.stdout.is_empty());
    assert!(broken_pipe.stderr.is_empty());
    let after_broken_pipe = WorkbenchProfiles::read(&config).unwrap().report();
    assert_eq!(after_broken_pipe.revision, 3);
    assert_eq!(after_broken_pipe.default_profile, None);
    assert_eq!(after_broken_pipe.prepared, info.prepared);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn test_profiles_refresh_default_remove_and_history_preserve_publication_rows() {
    let root = fresh_root();
    let catalogue = root.join("catalogue");
    install_synthetic_catalogue(&catalogue);
    let config = root.join("profiles.json");
    let selection = root.join("selection.json");
    fs::write(
        &selection,
        br#"{"version":1,"packs":["llg"],"disabled":[],"owned":null,"contexts":[]}"#,
    )
    .unwrap();
    let configure = |name: &str, expected: Option<u64>, make_default: bool| {
        let mut command = cli();
        command.args([
            "workbench",
            "profiles",
            "configure",
            "--config",
            config.to_str().unwrap(),
            "--name",
            name,
            "--selection",
            selection.to_str().unwrap(),
            "--catalogue",
            catalogue.to_str().unwrap(),
        ]);
        if make_default {
            command.arg("--make-default");
        }
        let expected = expected.map(|revision| revision.to_string());
        if let Some(revision) = expected.as_deref() {
            command.args(["--expected-revision", revision]);
        }
        command.output().unwrap()
    };

    let alpha = assert_json_line(&configure("alpha", None, true));
    assert_eq!(alpha["revision"], 1);
    let alpha_root = root.join("alpha-root");
    let refresh = cli()
        .args([
            "workbench",
            "profiles",
            "refresh",
            alpha_root.to_str().unwrap(),
            "--config",
            config.to_str().unwrap(),
            "--profile",
            "alpha",
            "--expected-revision",
            "1",
        ])
        .output()
        .unwrap();
    let refresh = assert_json_line(&refresh);
    assert_eq!(refresh["publication"]["revision"], 2);
    assert_eq!(refresh["publication"]["active"]["sequence"], 0);

    let beta = assert_json_line(&configure("beta", Some(2), false));
    assert_eq!(beta["revision"], 3);
    assert_eq!(beta["default_profile"], "alpha");
    assert_eq!(beta["active"], Value::Null);

    let replacement_selection = root.join("replacement-selection.json");
    fs::write(
        &replacement_selection,
        br#"{"version":1,"packs":["llg","la-lojban"],"disabled":["la-lojban"],"owned":null,"contexts":[]}"#,
    )
    .unwrap();
    let replaced = cli()
        .args([
            "workbench",
            "profiles",
            "configure",
            "--config",
            config.to_str().unwrap(),
            "--name",
            "alpha",
            "--selection",
            replacement_selection.to_str().unwrap(),
            "--catalogue",
            catalogue.to_str().unwrap(),
            "--expected-revision",
            "3",
        ])
        .output()
        .unwrap();
    let replaced = assert_json_line(&replaced);
    assert_eq!(replaced["revision"], 4);
    assert_eq!(replaced["active"], Value::Null);
    let alpha_history_after_reconfigure = WorkbenchProfiles::read(&config)
        .unwrap()
        .history(Some("alpha"))
        .unwrap();
    assert_eq!(alpha_history_after_reconfigure.prepared.len(), 1);
    assert_eq!(
        alpha_history_after_reconfigure.prepared[0]
            .root
            .utf8
            .as_deref(),
        Some(alpha_root.to_str().unwrap())
    );
    let request = root.join("reconfigured-request.json");
    fs::write(
        &request,
        br#"{"operation":"exact","spelling":"farvi","options":{}}"#,
    )
    .unwrap();
    let unprepared_query = cli()
        .args([
            "workbench",
            "profiles",
            "query",
            "--config",
            config.to_str().unwrap(),
            "--profile",
            "alpha",
            "--request",
            request.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_eq!(unprepared_query.status.code(), Some(1));
    assert!(unprepared_query.stdout.is_empty());
    let unprepared_error: Value = serde_json::from_slice(&unprepared_query.stderr).unwrap();
    assert_eq!(
        unprepared_error["error"]["code"],
        "unprepared_workbench_profile"
    );

    let history = cli()
        .args([
            "workbench",
            "profiles",
            "history",
            "--config",
            config.to_str().unwrap(),
            "--profile",
            "alpha",
        ])
        .output()
        .unwrap();
    let history = assert_json_line(&history);
    assert_eq!(history["prepared"].as_array().unwrap().len(), 1);
    assert_eq!(
        history["prepared"][0]["root"]["utf8"],
        alpha_root.to_str().unwrap()
    );

    let default = cli()
        .args([
            "workbench",
            "profiles",
            "default",
            "--config",
            config.to_str().unwrap(),
            "--profile",
            "beta",
            "--expected-revision",
            "4",
        ])
        .output()
        .unwrap();
    let default = assert_json_line(&default);
    assert_eq!(default["revision"], 5);
    assert_eq!(default["default_profile"], "beta");

    let remove = cli()
        .args([
            "workbench",
            "profiles",
            "remove",
            "--config",
            config.to_str().unwrap(),
            "--profile",
            "alpha",
            "--expected-revision",
            "5",
        ])
        .output()
        .unwrap();
    let remove = assert_json_line(&remove);
    assert_eq!(remove["revision"], 6);
    assert_eq!(remove["default_profile"], "beta");
    assert!(alpha_root.join("dictionary/dictionary.sqlite3").is_file());
    assert_eq!(remove["active"], Value::Null);

    let history_after_remove = cli()
        .args([
            "workbench",
            "profiles",
            "history",
            "--config",
            config.to_str().unwrap(),
            "--profile",
            "alpha",
        ])
        .output()
        .unwrap();
    let history_after_remove = assert_json_line(&history_after_remove);
    assert_eq!(history_after_remove["prepared"], history["prepared"]);

    let readded = assert_json_line(&configure("alpha", Some(6), false));
    assert_eq!(readded["revision"], 7);
    assert_eq!(readded["active"], Value::Null);
    let readded_info = WorkbenchProfiles::read(&config).unwrap().report();
    assert_eq!(readded_info.revision, 7);
    assert_eq!(readded_info.profiles[0].name, "alpha");
    assert_eq!(readded_info.profiles[0].active_generation, None);
    let retained_after_readd = WorkbenchProfiles::read(&config)
        .unwrap()
        .history(Some("alpha"))
        .unwrap();
    assert_eq!(
        serde_json::to_value(retained_after_readd.prepared).unwrap(),
        history["prepared"]
    );

    let make_readded_default = cli()
        .args([
            "workbench",
            "profiles",
            "default",
            "--config",
            config.to_str().unwrap(),
            "--profile",
            "alpha",
            "--expected-revision",
            "7",
        ])
        .output()
        .unwrap();
    let make_readded_default = assert_json_line(&make_readded_default);
    assert_eq!(make_readded_default["revision"], 8);
    let remove_default = cli()
        .args([
            "workbench",
            "profiles",
            "remove",
            "--config",
            config.to_str().unwrap(),
            "--profile",
            "alpha",
            "--expected-revision",
            "8",
        ])
        .output()
        .unwrap();
    let remove_default = assert_json_line(&remove_default);
    assert_eq!(remove_default["revision"], 9);
    assert_eq!(remove_default["default_profile"], Value::Null);
    fs::remove_dir_all(root).unwrap();
}
