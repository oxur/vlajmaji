//! End-to-end checks for the new direct and configured lexical CLI routes.

use std::{
    borrow::Cow,
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};

#[cfg(unix)]
use std::{
    ffi::OsString,
    fs::OpenOptions,
    io::Write,
    os::{
        fd::OwnedFd,
        unix::{
            ffi::OsStringExt,
            net::{UnixDatagram, UnixStream},
        },
    },
    process::Stdio,
};

use sha2::{Digest, Sha256};
use vlajmaji_lexicon::{
    CaptureRef, Dictionary, EntityId, FormKind, FormRecord, LlgImportOptions, OccurrenceRef,
    PackId, PackPayloadKind, Presence, SourceEvidence, SourceId, import_llg_lists, seal_release,
};
use vlajmaji_store::{
    CollectionCatalogue, CollectionInput, CollectionRequest, CollectionSelection,
    WorkbenchBuildRequest, WorkbenchSelection, WorkbenchView,
};

static NEXT_ROOT: AtomicUsize = AtomicUsize::new(0);

fn fresh_root() -> PathBuf {
    for _ in 0..64 {
        let root = std::env::temp_dir().join(format!(
            "vlajmaji-cli-lexical-{}-{}",
            std::process::id(),
            NEXT_ROOT.fetch_add(1, Ordering::Relaxed),
        ));
        match fs::create_dir(&root) {
            Ok(()) => return root,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => panic!("create exclusive CLI lexical root: {error}"),
        }
    }
    panic!("could not allocate CLI lexical root after 64 attempts")
}

fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn fixture_dictionary() -> Dictionary {
    let mut row = vec![b' '; 158];
    for (start, value) in [
        (1, "farvi"),
        (7, "far"),
        (11, "fav"),
        (15, "farv"),
        (20, "grow"),
        (41, "growth"),
        (62, "to grow"),
    ] {
        row[start..start + value.len()].copy_from_slice(value.as_bytes());
    }
    let empty = b"";
    let notice = b"CLI lexical fixture\n";
    let imported = import_llg_lists(
        &row,
        empty,
        empty,
        notice,
        &LlgImportOptions {
            expected_gismu_sha256: digest(&row),
            expected_cmavo_sha256: digest(empty),
            expected_rafsi_sha256: digest(empty),
            expected_notice_sha256: digest(notice),
            base_locator: "https://example.invalid/cli-lexical".into(),
        },
    )
    .unwrap();
    let mut document = imported.into_dictionary().into_document();
    let assertion = document.metadata.assertions[0].clone();
    document.metadata.forms.push(FormRecord {
        id: vlajmaji_lexicon::EntityId::new("cli-farvi-form").unwrap(),
        entry: assertion.entry,
        spelling: "farvi".into(),
        language: Presence::Value("jbo".into()),
        script: Presence::Missing,
        pronunciation: Presence::Missing,
        kind: FormKind::SourceVariant,
        standing: Presence::Missing,
        components: Vec::new(),
        evidence: Some(SourceEvidence {
            occurrence: assertion.occurrence,
            field_index: None,
            citation: Presence::Missing,
            offset: None,
        }),
        computation: None,
    });
    Dictionary::from_document(document).unwrap()
}

fn build_view(root: &Path) -> PathBuf {
    let dictionary = fixture_dictionary();
    let sealed = seal_release(PackId::Llg, PackPayloadKind::Synthetic, &dictionary).unwrap();
    let catalogue_root = root.join("catalogue");
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
    CollectionCatalogue::install(&catalogue_root, request).unwrap();
    let view_root = root.join("view");
    WorkbenchView::build_new(
        &view_root,
        WorkbenchBuildRequest {
            catalogue_root: Some(catalogue_root),
            owned_root: None,
            selection: WorkbenchSelection {
                version: 1,
                packs: vec![PackId::Llg],
                disabled: Vec::new(),
                owned: None,
                contexts: Vec::new(),
            },
        },
    )
    .unwrap();
    view_root
}

fn cli(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
        .args(args)
        .output()
        .expect("launch vlajmaji CLI")
}

fn write_json(path: &Path, value: &serde_json::Value) {
    fs::write(path, serde_json::to_vec(value).unwrap()).unwrap();
}

fn success(output: Output) -> serde_json::Value {
    assert!(
        output.status.success(),
        "CLI failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

fn cli_error(output: &Output, code: &str) -> serde_json::Value {
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert_eq!(
        output.stderr.iter().filter(|byte| **byte == b'\n').count(),
        1,
        "CLI errors are one JSON line"
    );
    let error: serde_json::Value = serde_json::from_slice(&output.stderr).unwrap();
    let mut outer = error["error"]
        .as_object()
        .unwrap()
        .keys()
        .collect::<Vec<_>>();
    outer.sort_unstable();
    assert_eq!(outer, vec!["code", "message", "sources"]);
    assert_eq!(error.as_object().unwrap().len(), 1);
    assert_eq!(error["error"]["code"], code);
    assert_eq!(error["error"]["sources"], serde_json::json!([]));
    error
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
            Ok(_) => unreachable!("single-byte write must complete or fail"),
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => break,
            #[cfg(target_os = "macos")]
            Err(error) if error.raw_os_error() == Some(55) => break,
            Err(error) => panic!("unexpected datagram saturation error: {error}"),
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
        .expect_err("closed UnixStream peer must reject a write");
    assert_eq!(error.kind(), std::io::ErrorKind::BrokenPipe);
    file
}

#[cfg(unix)]
#[test]
#[ignore = "invoked as an isolated child after deleting its own working directory"]
fn test_lexical_deleted_cwd_child() {
    let cwd = PathBuf::from(std::env::var_os("VLJ_LEXICAL_DELETED_CWD").unwrap());
    std::env::set_current_dir(&cwd).unwrap();
    fs::remove_dir(&cwd).unwrap();
    assert!(std::env::current_dir().is_err());

    let config = std::env::var_os("VLJ_LEXICAL_DELETED_CONFIG").unwrap();
    let view = std::env::var_os("VLJ_LEXICAL_DELETED_VIEW").unwrap();
    let request = std::env::var_os("VLJ_LEXICAL_DELETED_REQUEST").unwrap();
    let reference = std::env::var_os("VLJ_LEXICAL_DELETED_REFERENCE").unwrap();
    let binding = std::env::var("VLJ_LEXICAL_DELETED_BINDING").unwrap();
    let cli_path = PathBuf::from(env!("CARGO_BIN_EXE_vlajmaji"));
    match std::env::var("VLJ_LEXICAL_DELETED_MODE").as_deref() {
        Ok("absolute-paths") => {
            assert!(PathBuf::from(&config).is_absolute());
            assert!(PathBuf::from(&view).is_absolute());
            assert!(PathBuf::from(&request).is_absolute());
            assert!(PathBuf::from(&reference).is_absolute());

            let configured = Command::new(&cli_path)
                .args(["workbench", "profiles", "lexical", "--config"])
                .arg(&config)
                .args(["--profile", "alpha", "--request"])
                .arg(&request)
                .args(["--format", "full"])
                .output()
                .unwrap();
            let configured: serde_json::Value = success(configured);
            assert_eq!(configured["configuration"]["config_revision"], 2);
            assert_eq!(configured["result"]["response"]["totals"]["records"], 1);

            let direct = Command::new(&cli_path)
                .args(["workbench", "lexical"])
                .arg(&view)
                .arg("--request")
                .arg(&request)
                .args(["--format", "full"])
                .output()
                .unwrap();
            let direct: serde_json::Value = success(direct);
            assert_eq!(direct["result"]["response"]["totals"]["records"], 1);

            let detail = Command::new(cli_path)
                .args(["workbench", "form-detail"])
                .arg(&view)
                .arg("--reference")
                .arg(&reference)
                .arg("--expected-binding")
                .arg(binding)
                .args(["--format", "full"])
                .output()
                .unwrap();
            let detail: serde_json::Value = success(detail);
            assert_eq!(detail["form"]["form"]["id"], "cli-farvi-form");
        }
        Ok("relative-config") => {
            assert!(!PathBuf::from(&config).is_absolute());
            let output = Command::new(cli_path)
                .args(["workbench", "profiles", "lexical", "--config"])
                .arg(config)
                .args(["--profile", "alpha", "--request"])
                .arg(request)
                .args(["--format", "full"])
                .output()
                .unwrap();
            cli_error(&output, "workbench_config_io");
        }
        mode => panic!("unexpected deleted-cwd lexical child mode: {mode:?}"),
    }
}

#[cfg(unix)]
fn run_lexical_after_child_deletes_cwd(
    mode: &str,
    cwd: &Path,
    config: &Path,
    view: &Path,
    request: &Path,
    reference: &Path,
    binding: &str,
) {
    let output = Command::new(std::env::current_exe().unwrap())
        .args([
            "--ignored",
            "--exact",
            "test_lexical_deleted_cwd_child",
            "--nocapture",
        ])
        .env("VLJ_LEXICAL_DELETED_MODE", mode)
        .env("VLJ_LEXICAL_DELETED_CWD", cwd)
        .env("VLJ_LEXICAL_DELETED_CONFIG", config)
        .env("VLJ_LEXICAL_DELETED_VIEW", view)
        .env("VLJ_LEXICAL_DELETED_REQUEST", request)
        .env("VLJ_LEXICAL_DELETED_REFERENCE", reference)
        .env("VLJ_LEXICAL_DELETED_BINDING", binding)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "deleted-cwd lexical child {mode} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[cfg(unix)]
#[test]
fn test_lexical_absolute_paths_survive_deleted_cwd() {
    let root = fresh_root();
    let view = build_view(&root);
    let request = root.join("deleted-cwd-request.json");
    write_json(
        &request,
        &serde_json::json!({
            "version": 1,
            "query": {"operation": "form", "value": "farvi", "role": null, "kind": null},
            "options": {"sources": [], "alias_scope": "ActiveOnly", "page_size": null, "cursor": null}
        }),
    );
    let compact = success(cli(&[
        "workbench",
        "lexical",
        view.to_str().unwrap(),
        "--request",
        request.to_str().unwrap(),
        "--format",
        "compact",
    ]));
    let reference = root.join("deleted-cwd-detail.json");
    write_json(
        &reference,
        &compact["result"]["response"]["groups"][0]["hits"][0]["detail_ref"],
    );
    let binding = compact["binding_sha256"].as_str().unwrap().to_owned();

    let config = root.join("deleted-cwd-profiles.json");
    let selection = root.join("deleted-cwd-selection.json");
    write_json(
        &selection,
        &serde_json::json!({
            "version": 1, "packs": ["llg"], "disabled": [], "owned": null, "contexts": []
        }),
    );
    let configure = cli(&[
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
        root.join("catalogue").to_str().unwrap(),
    ]);
    assert_eq!(success(configure)["revision"], 1);
    let prepared = root.join("deleted-cwd-prepared");
    let refresh = cli(&[
        "workbench",
        "profiles",
        "refresh",
        prepared.to_str().unwrap(),
        "--config",
        config.to_str().unwrap(),
        "--profile",
        "alpha",
        "--expected-revision",
        "1",
    ]);
    assert_eq!(success(refresh)["publication"]["revision"], 2);

    let original_config = fs::read(&config).unwrap();
    for (mode, child_config) in [
        ("absolute-paths", config.clone()),
        (
            "relative-config",
            PathBuf::from("deleted-cwd-profiles.json"),
        ),
    ] {
        let cwd = root.join(format!("lexical-deleted-cwd-{mode}"));
        fs::create_dir(&cwd).unwrap();
        run_lexical_after_child_deletes_cwd(
            mode,
            &cwd,
            &child_config,
            &view,
            &request,
            &reference,
            &binding,
        );
        assert!(!cwd.exists());
        assert_eq!(fs::read(&config).unwrap(), original_config);
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn test_direct_and_configured_lexical_cli_emit_full_compact_readable_and_detail() {
    let root = fresh_root();
    let view = build_view(&root);
    let request = root.join("form request ü.json");
    write_json(
        &request,
        &serde_json::json!({
            "version": 1,
            "query": {"operation": "form", "value": "farvi", "role": null, "kind": null},
            "options": {"sources": [], "alias_scope": "ActiveOnly", "page_size": null, "cursor": null}
        }),
    );
    let request_arg = request.to_str().unwrap();
    let view_arg = view.to_str().unwrap();
    let direct_binding_before_errors = fs::read(view.join("binding.json")).unwrap();
    let direct_dictionary_before_errors =
        fs::read(view.join("dictionary/dictionary.sqlite3")).unwrap();

    let malformed_request = root.join("malformed lexical ü.json");
    let malformed_bytes = b"{\"version\":1,";
    fs::write(&malformed_request, malformed_bytes).unwrap();
    let malformed = cli(&[
        "workbench",
        "lexical",
        root.join("missing-view").to_str().unwrap(),
        "--request",
        malformed_request.to_str().unwrap(),
        "--format",
        "full",
    ]);
    cli_error(&malformed, "request_json");
    assert_eq!(fs::read(&malformed_request).unwrap(), malformed_bytes);
    assert!(!root.join("missing-view").exists());

    let old_detail_key = root.join("old assertion detail key.json");
    let old_detail_value = serde_json::to_value(vlajmaji_search::DictionaryHitDetailRef {
        version: 1,
        digest_policy: "entry-assertion-serde-json-v1".into(),
        role: "assertion".into(),
        source: SourceId::new("old-source").unwrap(),
        entry_id: EntityId::new("old-entry").unwrap(),
        assertion_id: EntityId::new("old-assertion").unwrap(),
        occurrence: OccurrenceRef {
            capture: CaptureRef {
                namespace: "fixture:old-reference".into(),
                sha256: "0".repeat(64),
            },
            locator: "/0".into(),
        },
        entry_sha256: "1".repeat(64),
        assertion_sha256: "2".repeat(64),
        native_entry_id: None,
        native_assertion_id: None,
        contributor_id: None,
        distributor_id: None,
    })
    .unwrap();
    write_json(&old_detail_key, &old_detail_value);
    let old_key_bytes = fs::read(&old_detail_key).unwrap();
    let old_key_error = cli(&[
        "workbench",
        "form-detail",
        root.join("missing-view").to_str().unwrap(),
        "--reference",
        old_detail_key.to_str().unwrap(),
        "--expected-binding",
        &"0".repeat(64),
        "--format",
        "full",
    ]);
    cli_error(&old_key_error, "request_json");
    assert_eq!(fs::read(&old_detail_key).unwrap(), old_key_bytes);
    assert!(!root.join("missing-view").exists());

    let missing_request = root.join("missing lexical ü.json");
    let missing = cli(&[
        "workbench",
        "lexical",
        view_arg,
        "--request",
        missing_request.to_str().unwrap(),
        "--format",
        "full",
    ]);
    cli_error(&missing, "workbench_io");
    assert_eq!(
        fs::read(view.join("binding.json")).unwrap(),
        direct_binding_before_errors
    );
    assert_eq!(
        fs::read(view.join("dictionary/dictionary.sqlite3")).unwrap(),
        direct_dictionary_before_errors
    );

    #[cfg(unix)]
    {
        let raw_request = root.join(OsString::from_vec(b"lexical-request-\xff.json".to_vec()));
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&raw_request)
        {
            Ok(mut file) => {
                let request_bytes = fs::read(&request).unwrap();
                file.write_all(&request_bytes).unwrap();
                drop(file);
                let output = Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
                    .args(["workbench", "lexical"])
                    .arg(view.as_os_str())
                    .arg("--request")
                    .arg(raw_request.as_os_str())
                    .args(["--format", "full"])
                    .output()
                    .unwrap();
                assert!(
                    output.status.success(),
                    "raw-native lexical path: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
                let response: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
                assert_eq!(response["format"], "full");
                assert_eq!(response["result"]["response"]["totals"]["records"], 1);
                assert_eq!(fs::read(&raw_request).unwrap(), request_bytes);
            }
            Err(error) if cfg!(target_os = "macos") && error.raw_os_error() == Some(92) => {
                eprintln!("raw-byte lexical request path unsupported here: macOS EILSEQ 92");
                let output = Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
                    .args(["workbench", "lexical"])
                    .arg(view.as_os_str())
                    .arg("--request")
                    .arg(raw_request.as_os_str())
                    .args(["--format", "full"])
                    .output()
                    .unwrap();
                cli_error(&output, "workbench_io");
            }
            Err(error) => panic!("raw-byte lexical request path preflight failed: {error}"),
        }

        let (saturated, peer) = saturated_stdout();
        let output_error = Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
            .args([
                "workbench",
                "lexical",
                view_arg,
                "--request",
                request_arg,
                "--format",
                "full",
            ])
            .stdout(Stdio::from(saturated))
            .output()
            .unwrap();
        drop(peer);
        let error = cli_error(&output_error, "response_output_io");
        assert!(
            error["error"]["message"]
                .as_str()
                .unwrap()
                .contains("not_applicable")
        );
        assert!(
            error["error"]["message"]
                .as_str()
                .unwrap()
                .contains(view_arg)
        );

        let broken_pipe = Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
            .args([
                "workbench",
                "lexical",
                view_arg,
                "--request",
                request_arg,
                "--format",
                "full",
            ])
            .stdout(Stdio::from(broken_pipe_stdout()))
            .output()
            .unwrap();
        assert_eq!(broken_pipe.status.code(), Some(0));
        assert!(broken_pipe.stdout.is_empty());
        assert!(broken_pipe.stderr.is_empty());
    }
    assert_eq!(
        fs::read(view.join("binding.json")).unwrap(),
        direct_binding_before_errors
    );
    assert_eq!(
        fs::read(view.join("dictionary/dictionary.sqlite3")).unwrap(),
        direct_dictionary_before_errors
    );

    let full = success(cli(&[
        "workbench",
        "lexical",
        view_arg,
        "--request",
        request_arg,
        "--format",
        "full",
    ]));
    assert_eq!(full["version"], 1);
    assert_eq!(full["kind"], "vlajmaji-workbench-lexical");
    assert_eq!(full["format"], "full");
    assert_eq!(full["result"]["operation"], "form");
    assert_eq!(full["result"]["response"]["totals"]["records"], 1);
    assert_eq!(
        full["result"]["response"]["groups"][0]["hits"][0]["form"]["id"],
        "cli-farvi-form"
    );

    let compact = success(cli(&[
        "workbench",
        "lexical",
        view_arg,
        "--request",
        request_arg,
        "--format",
        "compact",
    ]));
    assert_eq!(compact["format"], "compact");
    assert_eq!(compact["result"]["response"]["totals"]["records"], 1);
    let form = &compact["result"]["response"]["groups"][0]["hits"][0];
    assert_eq!(form["form_id"], "cli-farvi-form");
    let binding = compact["binding_sha256"].as_str().unwrap();
    let reference = root.join("form detail ü.json");
    write_json(&reference, form.get("detail_ref").unwrap());

    #[cfg(unix)]
    {
        let reference_bytes = fs::read(&reference).unwrap();
        let raw_reference = root.join(OsString::from_vec(b"form-detail-\xff.json".to_vec()));
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&raw_reference)
        {
            Ok(mut file) => {
                file.write_all(&reference_bytes).unwrap();
                drop(file);
                let output = Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
                    .args(["workbench", "form-detail"])
                    .arg(view.as_os_str())
                    .arg("--reference")
                    .arg(raw_reference.as_os_str())
                    .arg("--expected-binding")
                    .arg(binding)
                    .args(["--format", "full"])
                    .output()
                    .unwrap();
                let response = success(output);
                assert_eq!(response["form"]["form"]["id"], "cli-farvi-form");
                assert_eq!(fs::read(&raw_reference).unwrap(), reference_bytes);
                fs::remove_file(raw_reference).unwrap();
            }
            Err(error) if cfg!(target_os = "macos") && error.raw_os_error() == Some(92) => {
                eprintln!("raw-byte Form-detail path unsupported here: macOS EILSEQ 92");
                let output = Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
                    .args(["workbench", "form-detail"])
                    .arg(view.as_os_str())
                    .arg("--reference")
                    .arg(raw_reference.as_os_str())
                    .arg("--expected-binding")
                    .arg(binding)
                    .args(["--format", "full"])
                    .output()
                    .unwrap();
                cli_error(&output, "workbench_io");
            }
            Err(error) => panic!("raw-byte Form-detail path preflight failed: {error}"),
        }

        let (saturated, peer) = saturated_stdout();
        let output_error = Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
            .args(["workbench", "form-detail"])
            .arg(view.as_os_str())
            .arg("--reference")
            .arg(&reference)
            .arg("--expected-binding")
            .arg(binding)
            .args(["--format", "full"])
            .stdout(Stdio::from(saturated))
            .output()
            .unwrap();
        drop(peer);
        let error = cli_error(&output_error, "response_output_io");
        assert!(
            error["error"]["message"]
                .as_str()
                .unwrap()
                .contains(view_arg)
        );
        assert!(
            error["error"]["message"]
                .as_str()
                .unwrap()
                .contains("not_applicable")
        );

        let broken_pipe = Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
            .args(["workbench", "form-detail"])
            .arg(view.as_os_str())
            .arg("--reference")
            .arg(&reference)
            .arg("--expected-binding")
            .arg(binding)
            .args(["--format", "full"])
            .stdout(Stdio::from(broken_pipe_stdout()))
            .output()
            .unwrap();
        assert_eq!(broken_pipe.status.code(), Some(0));
        assert!(broken_pipe.stdout.is_empty());
        assert!(broken_pipe.stderr.is_empty());
        assert_eq!(fs::read(&reference).unwrap(), reference_bytes);
    }

    let detail = success(cli(&[
        "workbench",
        "form-detail",
        view_arg,
        "--reference",
        reference.to_str().unwrap(),
        "--expected-binding",
        binding,
        "--format",
        "full",
    ]));
    assert_eq!(detail["kind"], "vlajmaji-workbench-form-detail");
    assert_eq!(detail["form"]["form"]["id"], "cli-farvi-form");
    assert_eq!(detail["form"]["entry"]["id"], form["entry_id"]);

    let readable = cli(&[
        "workbench",
        "lexical",
        view_arg,
        "--request",
        request_arg,
        "--format",
        "readable",
    ]);
    assert!(readable.status.success());
    let readable = String::from_utf8(readable.stdout).unwrap();
    for label in [
        "workbench_binding: ",
        "workbench_selection: ",
        "lexical_operation: ",
        "lexical_policy: ",
        "lexical_totals: ",
        "lexical_source_totals: ",
        "lexical_next_cursor: ",
        "  form: ",
        "    form_detail: ",
    ] {
        assert!(readable.contains(label), "missing Readable label {label:?}");
    }

    let config = root.join("profiles.json");
    let selection = root.join("selection.json");
    write_json(
        &selection,
        &serde_json::json!({
            "version": 1, "packs": ["llg"], "disabled": [], "owned": null, "contexts": []
        }),
    );
    let configured_catalogue = root.join("catalogue");
    let configure = cli(&[
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
        configured_catalogue.to_str().unwrap(),
    ]);
    assert!(
        configure.status.success(),
        "configure: {}",
        String::from_utf8_lossy(&configure.stderr)
    );
    let destination = root.join("prepared-alpha");
    let refresh = cli(&[
        "workbench",
        "profiles",
        "refresh",
        destination.to_str().unwrap(),
        "--config",
        config.to_str().unwrap(),
        "--profile",
        "alpha",
        "--expected-revision",
        "1",
    ]);
    assert!(
        refresh.status.success(),
        "refresh: {}",
        String::from_utf8_lossy(&refresh.stderr)
    );
    let configured_bytes_before_output = fs::read(&config).unwrap();

    #[cfg(unix)]
    {
        let config_before_output = fs::read(&config).unwrap();
        let prepared_binding_before_output = fs::read(destination.join("binding.json")).unwrap();
        let prepared_dictionary_before_output =
            fs::read(destination.join("dictionary/dictionary.sqlite3")).unwrap();
        let (saturated, peer) = saturated_stdout();
        let output_error = Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
            .args(["workbench", "profiles", "lexical", "--config"])
            .arg(&config)
            .args(["--profile", "alpha", "--request"])
            .arg(&request)
            .args(["--format", "full"])
            .stdout(Stdio::from(saturated))
            .output()
            .unwrap();
        drop(peer);
        let error = cli_error(&output_error, "response_output_io");
        let message = error["error"]["message"].as_str().unwrap();
        assert!(message.contains(config.to_str().unwrap()));
        assert!(message.contains("alpha"));
        assert!(message.contains("not_applicable"));
        assert_eq!(fs::read(&config).unwrap(), config_before_output);
        assert_eq!(
            fs::read(destination.join("binding.json")).unwrap(),
            prepared_binding_before_output
        );
        assert_eq!(
            fs::read(destination.join("dictionary/dictionary.sqlite3")).unwrap(),
            prepared_dictionary_before_output
        );
        assert_eq!(fs::read(&request).unwrap(), serde_json::to_vec(&serde_json::json!({
            "version": 1,
            "query": {"operation": "form", "value": "farvi", "role": null, "kind": null},
            "options": {"sources": [], "alias_scope": "ActiveOnly", "page_size": null, "cursor": null}
        })).unwrap());
    }

    let configured = success(cli(&[
        "workbench",
        "profiles",
        "lexical",
        "--config",
        config.to_str().unwrap(),
        "--profile",
        "alpha",
        "--request",
        request_arg,
        "--format",
        "compact",
    ]));
    assert_eq!(configured["kind"], "vlajmaji-workbench-lexical");
    assert_eq!(configured["format"], "compact");
    assert!(configured["configuration"].is_object());
    assert_eq!(configured["result"]["response"]["totals"]["records"], 1);
    let configured_key = &configured["result"]["response"]["groups"][0]["hits"][0]["detail_ref"];
    write_json(&reference, configured_key);
    let configured_detail = success(cli(&[
        "workbench",
        "profiles",
        "form-detail",
        "--config",
        config.to_str().unwrap(),
        "--profile",
        "alpha",
        "--reference",
        reference.to_str().unwrap(),
        "--expected-binding",
        configured["binding_sha256"].as_str().unwrap(),
        "--format",
        "full",
    ]));
    assert!(configured_detail["configuration"].is_object());
    assert_eq!(configured_detail["form"]["form"]["id"], "cli-farvi-form");
    assert_eq!(
        fs::read(view.join("binding.json")).unwrap(),
        direct_binding_before_errors
    );
    assert_eq!(
        fs::read(view.join("dictionary/dictionary.sqlite3")).unwrap(),
        direct_dictionary_before_errors
    );
    assert_eq!(fs::read(&config).unwrap(), configured_bytes_before_output);
    fs::remove_dir_all(root).unwrap();
}
