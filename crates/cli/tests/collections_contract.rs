//! Public collection commands exercised through the actual CLI process.

use std::{
    path::{Path, PathBuf},
    process::{Command, Output},
    time::{SystemTime, UNIX_EPOCH},
};

use serde_json::Value;
use sha2::{Digest, Sha256};
use vlajmaji_lexicon::{Dictionary, PackId, PackPayloadKind, seal_release};
use vlajmaji_search::{DictionaryExactOptions, DictionarySearchEngine};
use vlajmaji_store::{CollectionCatalogue, DictionaryReadTopology};

fn fresh_root(label: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "vlajmaji-s02-cli-{label}-{}-{stamp}",
        std::process::id()
    ));
    std::fs::create_dir(&path).expect("create fresh CLI test root");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700)).unwrap();
    }
    path
}

fn run(args: &[&Path]) -> Output {
    run_from(Path::new("."), args)
}

fn run_from(cwd: &Path, args: &[&Path]) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_vlajmaji"));
    command
        .current_dir(cwd)
        .args(args)
        .env_clear()
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    command.output().expect("run collection CLI")
}

fn json(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout).expect("CLI JSON stdout")
}

fn collection_failure(output: &Output, category: &str, operation: &str) -> Value {
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let envelope: Value = serde_json::from_slice(&output.stderr).expect("collection error JSON");
    let report = envelope.get("error").expect("error envelope");
    assert_eq!(report["version"], 1);
    assert_eq!(report["ok"], false);
    assert_eq!(report["category"], category);
    assert_eq!(report["operation"], operation);
    assert_eq!(report["activation"], "not_activated");
    for field in [
        "pack_id",
        "attempt_id",
        "prior_local_revision",
        "proposed_generation",
        "observed_generation",
    ] {
        assert!(report[field].is_null(), "{field} must be explicit null");
    }
    assert_eq!(report["inputs"], serde_json::json!([]));
    report.clone()
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[test]
fn test_collections_cli_register_install_list_inspect_expect_structured_reports() {
    let root = fresh_root("lifecycle");
    let files = root.join("sealed inputs");
    std::fs::create_dir(&files).unwrap();
    let data = Dictionary::from_document(Default::default()).unwrap();
    let sealed = seal_release(PackId::Llg, PackPayloadKind::Synthetic, &data).unwrap();
    let release = files.join("release.json");
    let seal = files.join("seal.json");
    std::fs::write(&release, &sealed.release).unwrap();
    std::fs::write(&seal, &sealed.seal).unwrap();
    let catalogue = root.join("catalogue with spaces");

    let register = run(&[
        Path::new("collections"),
        Path::new("register"),
        &catalogue,
        Path::new("--pack"),
        Path::new("llg"),
        Path::new("--release"),
        &release,
        Path::new("--seal"),
        &seal,
        Path::new("--allow-synthetic"),
    ]);
    assert!(
        register.status.success(),
        "{}",
        String::from_utf8_lossy(&register.stderr)
    );
    let registered = json(&register);
    assert_eq!(registered["operation"], "register");
    assert_eq!(registered["outcome"], "registered");
    assert_eq!(registered["local_revision"], 1);
    assert!(registered["generation"].is_null());
    assert!(
        !catalogue
            .join("generations")
            .read_dir()
            .unwrap()
            .next()
            .is_some()
    );
    let registration_only = run(&[
        Path::new("collections"),
        Path::new("inspect"),
        &catalogue,
        Path::new("llg"),
    ]);
    assert!(registration_only.status.success());
    assert_eq!(
        json(&registration_only)["installed_history"],
        serde_json::json!([])
    );

    let install = run(&[
        Path::new("collections"),
        Path::new("install"),
        &catalogue,
        Path::new("--pack"),
        Path::new("llg"),
        Path::new("--release"),
        &release,
        Path::new("--seal"),
        &seal,
        Path::new("--allow-synthetic"),
    ]);
    assert!(
        install.status.success(),
        "{}",
        String::from_utf8_lossy(&install.stderr)
    );
    let installed = json(&install);
    assert_eq!(installed["operation"], "install");
    assert_eq!(installed["outcome"], "installed");
    assert_eq!(installed["local_revision"], 2);
    let store = PathBuf::from(installed["generation"]["path"]["utf8"].as_str().unwrap());
    assert!(store.join("dictionary.sqlite3").is_file());

    let list = run(&[Path::new("collections"), Path::new("list"), &catalogue]);
    assert!(
        list.status.success(),
        "{}",
        String::from_utf8_lossy(&list.stderr)
    );
    let listed = json(&list);
    assert_eq!(listed["local_revision"], 2);
    assert_eq!(listed["packs"][0]["pack_id"], "llg");
    assert_eq!(
        listed["generation"]["generation_id"],
        installed["generation"]["generation_id"]
    );

    let inspect = run(&[
        Path::new("collections"),
        Path::new("inspect"),
        &catalogue,
        Path::new("llg"),
    ]);
    assert!(
        inspect.status.success(),
        "{}",
        String::from_utf8_lossy(&inspect.stderr)
    );
    let inspected = json(&inspect);
    assert_eq!(inspected["pack_id"], "llg");
    assert_eq!(inspected["last_verification_scope"], "all");
    assert_eq!(inspected["policy_state"], "unconfigured");
    assert_eq!(inspected["installed_history"].as_array().unwrap().len(), 1);
    assert_eq!(inspected["installed_history"][0]["current"], true);

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn test_collections_cli_lost_install_response_expect_committed_generation_and_query() {
    use std::io::Write;
    use std::process::Stdio;

    let root = fresh_root("lost-response");
    let files = root.join("sealed inputs");
    std::fs::create_dir(&files).unwrap();
    let data = Dictionary::from_document(Default::default()).unwrap();
    let sealed = seal_release(PackId::Llg, PackPayloadKind::Synthetic, &data).unwrap();
    let release = files.join("release.json");
    let seal = files.join("seal.json");
    std::fs::write(&release, &sealed.release).unwrap();
    std::fs::write(&seal, &sealed.seal).unwrap();
    let catalogue = root.join("catalogue");

    // Keep the CLI behind a stdin gate until the parent has closed the only
    // stdout reader. The install can then commit, while its JSON response is
    // guaranteed to encounter a closed pipe rather than winning a timing race.
    let mut child = Command::new("/bin/sh")
        .arg("-c")
        .arg("IFS= read -r _ || exit 125; exec \"$@\"")
        .arg("collections-install-output-loss")
        .arg(env!("CARGO_BIN_EXE_vlajmaji"))
        .arg("collections")
        .arg("install")
        .arg(&catalogue)
        .arg("--pack")
        .arg("llg")
        .arg("--release")
        .arg(&release)
        .arg("--seal")
        .arg(&seal)
        .arg("--allow-synthetic")
        .env_clear()
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("start gated install CLI");
    drop(child.stdout.take().expect("install stdout pipe"));
    child
        .stdin
        .take()
        .expect("install stdin gate")
        .write_all(b"go\n")
        .unwrap();
    let result = child
        .wait_with_output()
        .expect("wait for gated install CLI");
    assert!(
        result.status.success(),
        "lost response must not turn a committed install into failure: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(
        result.stderr.is_empty(),
        "successful broken-pipe handling is quiet"
    );

    let listed = run(&[Path::new("collections"), Path::new("list"), &catalogue]);
    assert!(
        listed.status.success(),
        "{}",
        String::from_utf8_lossy(&listed.stderr)
    );
    let listed_json = json(&listed);
    assert_eq!(listed_json["local_revision"], 1);
    let generation = &listed_json["generation"];
    let generation_id = generation["generation_id"]
        .as_str()
        .expect("committed generation id");
    let store_path = PathBuf::from(
        generation["path"]["utf8"]
            .as_str()
            .expect("UTF-8 generation path"),
    );
    assert!(store_path.join("dictionary.sqlite3").is_file());

    let inspected = run(&[
        Path::new("collections"),
        Path::new("inspect"),
        &catalogue,
        Path::new("llg"),
    ]);
    assert!(
        inspected.status.success(),
        "{}",
        String::from_utf8_lossy(&inspected.stderr)
    );
    let inspected_json = json(&inspected);
    assert_eq!(
        inspected_json["installed_history"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        inspected_json["installed_history"][0]["generation_id"],
        generation_id
    );
    assert_eq!(inspected_json["installed_history"][0]["current"], true);

    let catalogue_state = CollectionCatalogue::open(&catalogue)
        .unwrap()
        .list()
        .unwrap();
    let committed = catalogue_state.generation.expect("committed generation");
    assert_eq!(committed.generation_id, generation_id);
    assert_eq!(committed.store_path, store_path);
    let engine = DictionarySearchEngine::open_snapshot(
        DictionaryReadTopology::Unified,
        committed.snapshot_manifest(),
    )
    .unwrap();
    let query = engine
        .exact("farvi", DictionaryExactOptions::default())
        .unwrap();
    assert_eq!(query.state_sha256, committed.canonical_sha256);
    assert_eq!(query.spelling, "farvi");
    assert_eq!(
        query.totals.occurrences, 0,
        "the installed empty fixture remains queryable"
    );

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn test_collections_cli_list_absent_catalogue_expect_typed_error() {
    let parent = fresh_root("absent");
    let absent = parent.join("no catalogue");
    let output = run(&[Path::new("collections"), Path::new("list"), &absent]);
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let error: Value = serde_json::from_slice(&output.stderr).expect("structured stderr");
    assert_eq!(error["error"]["version"], 1);
    assert_eq!(error["error"]["ok"], false);
    assert_eq!(error["error"]["category"], "missing_catalogue");
    assert_eq!(error["error"]["operation"], "list");
    assert_eq!(error["error"]["activation"], "not_applicable");
    assert!(
        error["error"]["detail"]
            .as_str()
            .is_some_and(|detail| detail.contains("missing"))
    );
    assert!(error["error"]["path"]["hex"].as_str().is_some());
    for field in [
        "pack_id",
        "attempt_id",
        "prior_local_revision",
        "proposed_generation",
        "observed_generation",
    ] {
        assert!(
            error["error"][field].is_null(),
            "{field} must be explicitly nullable"
        );
    }
    assert_eq!(error["error"]["inputs"], serde_json::json!([]));
    assert!(error["error"]["observed_generation"].is_null());
    assert!(!absent.exists());
    let _ = std::fs::remove_dir_all(parent);
}

#[test]
fn test_collections_cli_runtime_validation_expect_versioned_report_and_clap_boundary() {
    let parent = fresh_root("runtime-errors");

    let manifest_root = parent.join("manifest-no-selector-root");
    let manifest = parent.join("unused manifest.json");
    let no_selector = run(&[
        Path::new("collections"),
        Path::new("install"),
        &manifest_root,
        Path::new("--manifest"),
        &manifest,
    ]);
    let report = collection_failure(&no_selector, "invalid_request", "install");
    assert!(
        report["detail"]
            .as_str()
            .unwrap()
            .contains("requires --all")
    );
    assert!(report["path"].is_null());
    assert!(!manifest_root.exists());

    let direct_root = parent.join("direct-no-files-root");
    let no_files = run(&[
        Path::new("collections"),
        Path::new("register"),
        &direct_root,
        Path::new("--pack"),
        Path::new("llg"),
    ]);
    let report = collection_failure(&no_files, "invalid_request", "register");
    assert!(report["detail"].as_str().unwrap().contains("requires both"));
    assert!(report["path"].is_null());
    assert!(!direct_root.exists());

    let missing_root = parent.join("missing-manifest-root");
    let missing_manifest = parent.join("missing manifest 'λ'.json");
    let direct_error = std::fs::read(&missing_manifest).unwrap_err();
    let missing = run(&[
        Path::new("collections"),
        Path::new("install"),
        &missing_root,
        Path::new("--manifest"),
        &missing_manifest,
        Path::new("--all"),
    ]);
    let report = collection_failure(&missing, "input_io", "install");
    assert!(
        report["detail"]
            .as_str()
            .unwrap()
            .contains(&direct_error.to_string())
    );
    assert_eq!(report["path"]["utf8"], missing_manifest.to_str().unwrap());
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        let expected = missing_manifest
            .as_os_str()
            .as_bytes()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        assert_eq!(report["path"]["hex"], expected);
    }
    assert!(!missing_root.exists());

    // These conflicting flags are rejected by Clap before collection runtime
    // conversion and intentionally retain their plain parser diagnostics.
    let conflict_root = parent.join("clap-conflict-root");
    let conflict = Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
        .args([
            "collections",
            "install",
            conflict_root.to_str().unwrap(),
            "--manifest",
            manifest.to_str().unwrap(),
            "--release",
            "release.json",
        ])
        .env_clear()
        .output()
        .unwrap();
    assert!(!conflict.status.success());
    assert!(conflict.stdout.is_empty());
    assert!(serde_json::from_slice::<Value>(&conflict.stderr).is_err());
    assert!(!conflict_root.exists());

    let _ = std::fs::remove_dir_all(parent);
}

#[cfg(unix)]
#[test]
fn test_collections_cli_native_missing_manifest_expect_exact_path_report() {
    use std::os::unix::ffi::{OsStrExt, OsStringExt};

    let parent = fresh_root("native-manifest-error");
    let mut native = parent.as_os_str().as_bytes().to_vec();
    native.extend_from_slice(b"/missing-\xff-manifest.json");
    let manifest = PathBuf::from(std::ffi::OsString::from_vec(native));
    let root = parent.join("catalogue-not-created");
    let direct_error = std::fs::read(&manifest).unwrap_err();
    let output = run(&[
        Path::new("collections"),
        Path::new("install"),
        &root,
        Path::new("--manifest"),
        &manifest,
        Path::new("--all"),
    ]);
    let report = collection_failure(&output, "input_io", "install");
    assert!(
        report["detail"]
            .as_str()
            .unwrap()
            .contains(&direct_error.to_string())
    );
    let expected = manifest
        .as_os_str()
        .as_bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    assert_eq!(report["path"]["encoding"], "unix-bytes");
    assert_eq!(report["path"]["hex"], expected);
    assert!(report["path"]["utf8"].is_null());
    assert!(!root.exists());
    let _ = std::fs::remove_dir_all(parent);
}

#[cfg(target_os = "linux")]
#[test]
fn test_collections_cli_native_release_seal_and_root_paths_register_inspect_exactly() {
    use std::os::unix::ffi::{OsStrExt, OsStringExt};

    let parent = fresh_root("native-existing-inputs");
    let mut release_name = b"release-".to_vec();
    release_name.push(0xff);
    release_name.extend_from_slice(b".json");
    let release = parent.join(std::ffi::OsString::from_vec(release_name));

    let mut seal_name = b"seal-".to_vec();
    seal_name.push(0xfe);
    seal_name.extend_from_slice(b".json");
    let seal = parent.join(std::ffi::OsString::from_vec(seal_name));

    let catalogue = parent.join(std::ffi::OsString::from_vec(b"catalogue-\xfd".to_vec()));
    let dictionary = Dictionary::from_document(Default::default()).unwrap();
    let sealed = seal_release(PackId::Llg, PackPayloadKind::Synthetic, &dictionary).unwrap();
    std::fs::write(&release, &sealed.release).unwrap();
    std::fs::write(&seal, &sealed.seal).unwrap();
    let release_before = std::fs::read(&release).unwrap();
    let seal_before = std::fs::read(&seal).unwrap();

    let register = run(&[
        Path::new("collections"),
        Path::new("register"),
        catalogue.as_path(),
        Path::new("--pack"),
        Path::new("llg"),
        Path::new("--release"),
        release.as_path(),
        Path::new("--seal"),
        seal.as_path(),
        Path::new("--allow-synthetic"),
    ]);
    assert!(
        register.status.success(),
        "{}",
        String::from_utf8_lossy(&register.stderr)
    );
    let registered = json(&register);
    assert_eq!(registered["version"], 1);
    assert_eq!(registered["operation"], "register");
    assert_eq!(registered["outcome"], "registered");
    assert_eq!(registered["local_revision"], 1);
    assert!(registered["generation"].is_null());
    assert!(std::fs::symlink_metadata(&catalogue).unwrap().is_dir());
    assert!(catalogue.join("catalogue.sqlite3").is_file());

    let inspect = run(&[
        Path::new("collections"),
        Path::new("inspect"),
        catalogue.as_path(),
        Path::new("llg"),
    ]);
    assert!(
        inspect.status.success(),
        "{}",
        String::from_utf8_lossy(&inspect.stderr)
    );
    let inspected = json(&inspect);
    assert_eq!(inspected["version"], 1);
    assert_eq!(inspected["pack_id"], "llg");
    let registration = &inspected["registrations"][0];
    assert_eq!(registration["inputs_present"], true);
    assert_eq!(registration["input_ownership"], "caller_managed");
    let input_paths = registration["inputs"].as_array().unwrap();
    assert_eq!(input_paths.len(), 2);
    let mut actual_hex = input_paths
        .iter()
        .map(|path| {
            assert_eq!(path["encoding"], "unix-bytes");
            assert!(path["utf8"].is_null());
            path["hex"].as_str().unwrap().to_owned()
        })
        .collect::<Vec<_>>();
    actual_hex.sort();
    let mut expected_hex = [&release, &seal]
        .into_iter()
        .map(|path| {
            path.as_os_str()
                .as_bytes()
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>()
        })
        .collect::<Vec<_>>();
    expected_hex.sort();
    assert_eq!(actual_hex, expected_hex);

    assert_eq!(std::fs::read(&release).unwrap(), release_before);
    assert_eq!(std::fs::read(&seal).unwrap(), seal_before);

    // These lossy replacement paths are only negative witnesses; all actual
    // file creation and CLI arguments above use the original native Paths.
    let replacement_path = |path: &Path| {
        PathBuf::from(String::from_utf8_lossy(path.as_os_str().as_bytes()).into_owned())
    };
    assert!(!replacement_path(&release).exists());
    assert!(!replacement_path(&seal).exists());
    assert!(!replacement_path(&catalogue).exists());

    let _ = std::fs::remove_dir_all(parent);
}

#[test]
fn test_collections_cli_relative_paths_are_frozen_across_working_directories() {
    let scratch = fresh_root("relative-paths");
    let prepared = run_from(
        &scratch,
        &[
            Path::new("data-pack"),
            Path::new("fixture"),
            Path::new("llg"),
            Path::new("pack files"),
        ],
    );
    assert!(
        prepared.status.success(),
        "{}",
        String::from_utf8_lossy(&prepared.stderr)
    );

    let installed = run_from(
        &scratch,
        &[
            Path::new("collections"),
            Path::new("install"),
            Path::new("catalogue"),
            Path::new("--pack"),
            Path::new("llg"),
            Path::new("--release"),
            Path::new("pack files/release.json"),
            Path::new("--seal"),
            Path::new("pack files/seal.json"),
            Path::new("--allow-synthetic"),
        ],
    );
    assert!(
        installed.status.success(),
        "{}",
        String::from_utf8_lossy(&installed.stderr)
    );

    let catalogue = scratch.join("catalogue");
    let listed = run(&[Path::new("collections"), Path::new("list"), &catalogue]);
    assert!(
        listed.status.success(),
        "{}",
        String::from_utf8_lossy(&listed.stderr)
    );
    let registration = &json(&listed)["packs"][0]["registrations"][0];
    assert_eq!(registration["inputs_present"], true);
    assert_eq!(registration["input_ownership"], "caller_managed");
    let release_path = registration["inputs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|path| {
            path["utf8"]
                .as_str()
                .is_some_and(|value| value.ends_with("pack files/release.json"))
        })
        .expect("frozen absolute release input path");
    assert_eq!(
        release_path["utf8"],
        scratch
            .canonicalize()
            .unwrap()
            .join("pack files/release.json")
            .to_str()
            .unwrap()
    );
    let _ = std::fs::remove_dir_all(scratch);
}

#[test]
fn test_collections_cli_all_raw_inputs_expect_four_pack_install() {
    let scratch = fresh_root("all-raw");
    let input_dir = scratch.join("raw inputs");
    std::fs::create_dir(&input_dir).unwrap();

    let jbotci = br#"[{"word":"farvi","word_type":"gismu","definition":"to grow","definition_id":7},{"word":"farvi","word_type":"gismu","definition":"to develop","definition_id":8},{"word":"farvi","word_type":"wiki","definition":"community note"}]"#;
    let jbotci_metadata = format!(
        "language_tag = \"en\"\nlanguage_realname = \"English\"\nsource_language_tag = \"jbo\"\nformat = \"json\"\npositive_scores_only = false\nfilename = \"dictionary-en.json\"\nmetadata_url = \"https://example.invalid/languages\"\ndownload_url = \"https://example.invalid/synthetic-export\"\nlensisku_created_at = \"2026-10-04T03:07:08Z\"\nsha256 = \"{}\"\ndefinition_count = 3\nentry_count = 1\n",
        sha256(jbotci)
    );
    let la_lojban = jbotci;
    let sutysisku = br#"{"farvi":{"d":"to grow","t":"gismu","r":["far","fari"]},"note":{"d":"","t":"wiki","n":null}}"#;

    let mut llg_gismu = b"00001 Lojban synthetic gismu list\r\n".to_vec();
    let mut row = vec![b' '; 158];
    row[1..6].copy_from_slice(b"farvi");
    row[7..10].copy_from_slice(b"far");
    row[11..14].copy_from_slice(b"fav");
    row[20..24].copy_from_slice(b"grow");
    row[41..47].copy_from_slice(b"growth");
    row[62..69].copy_from_slice(b"to grow");
    llg_gismu.extend_from_slice(&row);
    llg_gismu.extend_from_slice(b"\r\n");
    let llg_cmavo = b"00001 Lojban synthetic cmavo list\r\ndo        BAI       because                                 because it is so\r\n";
    let llg_rafsi = b"far farvi growth\r\n";
    let llg_notice = b"Synthetic LLG notice; no redistribution claim.\r\n";

    for (name, bytes) in [
        ("jbotci.json", jbotci.as_slice()),
        ("jbotci.toml", jbotci_metadata.as_bytes()),
        ("la-lojban.json", la_lojban.as_slice()),
        ("sutysisku.json", sutysisku.as_slice()),
        ("llg-gismu.txt", llg_gismu.as_slice()),
        ("llg-cmavo.txt", llg_cmavo.as_slice()),
        ("llg-rafsi.txt", llg_rafsi.as_slice()),
        ("llg-notice.txt", llg_notice.as_slice()),
    ] {
        std::fs::write(input_dir.join(name), bytes).unwrap();
    }

    let manifest = serde_json::json!({
        "version": 1,
        "inputs": [
            {"kind":"jbotci", "pack_id":"lensisku-jbotci", "payload_kind":"synthetic", "raw":"jbotci.json", "metadata":"jbotci.toml", "options":{"revision":"0000000000000000000000000000000000000000", "expected_raw_sha256":sha256(jbotci), "expected_metadata_sha256":sha256(jbotci_metadata.as_bytes())}},
            {"kind":"llg", "pack_id":"llg", "payload_kind":"synthetic", "gismu":"llg-gismu.txt", "cmavo":"llg-cmavo.txt", "rafsi":"llg-rafsi.txt", "notice":"llg-notice.txt", "options":{"expected_gismu_sha256":sha256(&llg_gismu), "expected_cmavo_sha256":sha256(llg_cmavo), "expected_rafsi_sha256":sha256(llg_rafsi), "expected_notice_sha256":sha256(llg_notice), "base_locator":"https://example.invalid/llg"}},
            {"kind":"la-lojban", "pack_id":"la-lojban", "payload_kind":"synthetic", "raw":"la-lojban.json", "options":{"revision":"0000000000000000000000000000000000000000", "expected_raw_sha256":sha256(la_lojban), "raw_locator":null, "notice_locator":null}},
            {"kind":"sutysisku", "pack_id":"sutysisku", "payload_kind":"synthetic", "raw":"sutysisku.json", "options":{"revision":"0000000000000000000000000000000000000000", "expected_raw_sha256":sha256(sutysisku), "raw_locator":null, "notice_locator":null}}
        ]
    });
    let manifest_path = input_dir.join("manifest.json");
    std::fs::write(&manifest_path, serde_json::to_vec(&manifest).unwrap()).unwrap();

    let selected_catalogue = scratch.join("selected catalogue");
    let selected = run(&[
        Path::new("collections"),
        Path::new("install"),
        &selected_catalogue,
        Path::new("--manifest"),
        &manifest_path,
        Path::new("--pack"),
        Path::new("llg"),
        Path::new("--allow-synthetic"),
    ]);
    assert!(
        selected.status.success(),
        "{}",
        String::from_utf8_lossy(&selected.stderr)
    );
    let selected_inputs = json(&selected)["inputs"].as_array().unwrap().clone();
    assert_eq!(selected_inputs.len(), 4);
    assert_eq!(
        selected_inputs
            .iter()
            .map(|item| item["disposition"].as_str().unwrap())
            .collect::<Vec<_>>(),
        vec![
            "skipped_without_io",
            "installed",
            "skipped_without_io",
            "skipped_without_io"
        ]
    );

    let mismatched_metadata = jbotci_metadata.replace("entry_count = 1", "entry_count = 2");
    let mismatched_metadata_path = input_dir.join("jbotci-mismatched.toml");
    std::fs::write(&mismatched_metadata_path, mismatched_metadata.as_bytes()).unwrap();
    let mut mismatched_manifest = manifest.clone();
    mismatched_manifest["inputs"][0]["metadata"] =
        serde_json::Value::String("jbotci-mismatched.toml".into());
    mismatched_manifest["inputs"][0]["options"]["expected_metadata_sha256"] =
        serde_json::Value::String(sha256(mismatched_metadata.as_bytes()));
    let mismatched_manifest_path = input_dir.join("manifest-mismatched-metadata.json");
    std::fs::write(
        &mismatched_manifest_path,
        serde_json::to_vec(&mismatched_manifest).unwrap(),
    )
    .unwrap();
    let mismatched_catalogue = scratch.join("mismatched catalogue");
    let mismatch = run(&[
        Path::new("collections"),
        Path::new("install"),
        &mismatched_catalogue,
        Path::new("--manifest"),
        &mismatched_manifest_path,
        Path::new("--pack"),
        Path::new("lensisku-jbotci"),
        Path::new("--allow-synthetic"),
    ]);
    assert!(!mismatch.status.success());
    let mismatch_error: Value = serde_json::from_slice(&mismatch.stderr).unwrap();
    assert_eq!(mismatch_error["error"]["category"], "input_invalid");
    assert_eq!(mismatch_error["error"]["activation"], "not_activated");
    assert_eq!(
        mismatch_error["error"]["inputs"][0]["disposition"],
        "failed"
    );
    assert!(
        !mismatched_catalogue.exists(),
        "supplier-count mismatch does not create a catalogue"
    );

    let mut failing_manifest = manifest.clone();
    failing_manifest["inputs"][1]["gismu"] = serde_json::Value::String("missing-gismu.txt".into());
    let failing_manifest_path = input_dir.join("manifest-failing.json");
    std::fs::write(
        &failing_manifest_path,
        serde_json::to_vec(&failing_manifest).unwrap(),
    )
    .unwrap();
    let failing_catalogue = scratch.join("failing catalogue");
    let failed = run(&[
        Path::new("collections"),
        Path::new("install"),
        &failing_catalogue,
        Path::new("--manifest"),
        &failing_manifest_path,
        Path::new("--all"),
        Path::new("--allow-synthetic"),
    ]);
    assert!(!failed.status.success());
    let failure: Value = serde_json::from_slice(&failed.stderr).unwrap();
    let failed_inputs = failure["error"]["inputs"].as_array().unwrap();
    assert_eq!(failed_inputs.len(), 4);
    assert_eq!(failed_inputs[0]["disposition"], "validated");
    assert!(failed_inputs[0]["release_sha256"].as_str().is_some());
    assert!(failed_inputs[0]["report"].is_object());
    assert_eq!(failed_inputs[1]["disposition"], "failed");
    assert_eq!(failed_inputs[2]["disposition"], "unprocessed");
    assert_eq!(failed_inputs[3]["disposition"], "unprocessed");
    assert!(
        !failing_catalogue.exists(),
        "all-input failure allocates no catalogue"
    );

    let catalogue = scratch.join("catalogue");
    let output = run(&[
        Path::new("collections"),
        Path::new("install"),
        &catalogue,
        Path::new("--manifest"),
        &manifest_path,
        Path::new("--all"),
        Path::new("--allow-synthetic"),
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let installed = json(&output);
    assert_eq!(installed["outcome"], "installed");
    assert_eq!(installed["inputs"].as_array().unwrap().len(), 4);
    assert!(
        installed["inputs"]
            .as_array()
            .unwrap()
            .iter()
            .all(|input| input["disposition"] == "installed")
    );

    let listed = json(&run(&[
        Path::new("collections"),
        Path::new("list"),
        &catalogue,
    ]));
    assert_eq!(listed["packs"].as_array().unwrap().len(), 4);
    assert_eq!(listed["generation"]["members"].as_array().unwrap().len(), 4);
    for pack in listed["packs"].as_array().unwrap() {
        let registration = &pack["registrations"][0];
        assert_eq!(registration["input_ownership"], "caller_managed");
        assert_eq!(registration["installed_pack_copy_ownership"], "tool_owned");
        assert_eq!(registration["derived_store_ownership"], "tool_owned");
    }
    let _ = std::fs::remove_dir_all(scratch);
}

#[test]
fn test_collections_cli_opposite_pack_orders_expect_same_content_and_distinct_generations() {
    let scratch = fresh_root("order");
    let jbotci_dir = scratch.join("jbotci pack");
    let llg_dir = scratch.join("llg pack");
    for (pack, output) in [("jbotci", &jbotci_dir), ("llg", &llg_dir)] {
        let prepared = run(&[
            Path::new("data-pack"),
            Path::new("fixture"),
            Path::new(pack),
            output,
        ]);
        assert!(
            prepared.status.success(),
            "{}",
            String::from_utf8_lossy(&prepared.stderr)
        );
    }

    let root_llg_first = scratch.join("llg-first");
    let root_jb_first = scratch.join("jb-first");
    let install = |root: &Path, pack: &str, directory: &Path| {
        let release = directory.join("release.json");
        let seal = directory.join("seal.json");
        let output = run(&[
            Path::new("collections"),
            Path::new("install"),
            root,
            Path::new("--pack"),
            Path::new(pack),
            Path::new("--release"),
            &release,
            Path::new("--seal"),
            &seal,
            Path::new("--allow-synthetic"),
        ]);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        json(&output)
    };

    let _ = install(&root_llg_first, "llg", &llg_dir);
    let llg_first_generation =
        install(&root_llg_first, "lensisku-jbotci", &jbotci_dir)["generation"].clone();
    let jb_first_generation =
        install(&root_jb_first, "lensisku-jbotci", &jbotci_dir)["generation"].clone();
    let jb_first_store = PathBuf::from(jb_first_generation["path"]["utf8"].as_str().unwrap());
    let jb_first_store_bytes = std::fs::read(jb_first_store.join("dictionary.sqlite3")).unwrap();
    let _installed = install(&root_jb_first, "llg", &llg_dir);
    let final_generation = json(&run(&[
        Path::new("collections"),
        Path::new("list"),
        &root_jb_first,
    ]))["generation"]
        .clone();
    assert_eq!(
        llg_first_generation["content_id"],
        final_generation["content_id"]
    );
    assert_eq!(
        llg_first_generation["canonical_sha256"],
        final_generation["canonical_sha256"]
    );
    assert_ne!(
        llg_first_generation["generation_id"],
        final_generation["generation_id"]
    );
    assert!(jb_first_store.join("dictionary.sqlite3").is_file());
    assert_eq!(
        std::fs::read(jb_first_store.join("dictionary.sqlite3")).unwrap(),
        jb_first_store_bytes,
        "later collection generations must leave the prior immutable store byte-identical"
    );
    let history = json(&run(&[
        Path::new("collections"),
        Path::new("inspect"),
        &root_jb_first,
        Path::new("lensisku-jbotci"),
    ]));
    assert_eq!(history["installed_history"].as_array().unwrap().len(), 2);
    assert_eq!(history["installed_history"][0]["current"], true);
    assert_eq!(history["installed_history"][1]["current"], false);

    let query = |store: &Path| {
        run(&[
            Path::new("dictionary-query"),
            store,
            Path::new("--request"),
            Path::new(r#"{"operation":"exact","spelling":"farvi"}"#),
        ])
    };
    let first_store = PathBuf::from(llg_first_generation["path"]["utf8"].as_str().unwrap());
    let response_a = query(&first_store);
    let response_b = query(&PathBuf::from(
        final_generation["path"]["utf8"].as_str().unwrap(),
    ));
    assert!(
        response_a.status.success(),
        "{}",
        String::from_utf8_lossy(&response_a.stderr)
    );
    assert!(
        response_b.status.success(),
        "{}",
        String::from_utf8_lossy(&response_b.stderr)
    );
    assert_eq!(
        response_a.stdout, response_b.stdout,
        "same canonical content must preserve full CLI response bytes"
    );
    let exact = json(&response_a);
    assert_eq!(exact["spelling"], "farvi");
    assert_eq!(exact["totals"]["occurrences"], 3);
    assert_eq!(exact["totals"]["source_groups"], 2);
    let source_occurrences = exact["source_totals"]
        .as_array()
        .unwrap()
        .iter()
        .map(|source| {
            (
                source["source"]["id"].as_str().unwrap().to_owned(),
                source["occurrences"].as_u64().unwrap(),
            )
        })
        .collect::<std::collections::BTreeMap<_, _>>();
    assert_eq!(
        source_occurrences,
        std::collections::BTreeMap::from([("lensisku".to_owned(), 2), ("llg".to_owned(), 1),])
    );

    let _ = std::fs::remove_dir_all(scratch);
}

#[test]
fn test_collections_cli_tampered_repeat_expect_prior_generation_unchanged() {
    let scratch = fresh_root("tampered-repeat");
    let pack_dir = scratch.join("llg pack");
    let prepared = run(&[
        Path::new("data-pack"),
        Path::new("fixture"),
        Path::new("llg"),
        &pack_dir,
    ]);
    assert!(
        prepared.status.success(),
        "{}",
        String::from_utf8_lossy(&prepared.stderr)
    );
    let root = scratch.join("catalogue");
    let install = |release: &Path| {
        run(&[
            Path::new("collections"),
            Path::new("install"),
            &root,
            Path::new("--pack"),
            Path::new("llg"),
            Path::new("--release"),
            release,
            Path::new("--seal"),
            &pack_dir.join("seal.json"),
            Path::new("--allow-synthetic"),
        ])
    };
    let original = install(&pack_dir.join("release.json"));
    assert!(
        original.status.success(),
        "{}",
        String::from_utf8_lossy(&original.stderr)
    );
    let before = json(&run(&[Path::new("collections"), Path::new("list"), &root]));
    let generation_id = before["generation"]["generation_id"].clone();
    let revision = before["local_revision"].clone();

    let mut tampered = std::fs::read(pack_dir.join("release.json")).unwrap();
    tampered.push(b' ');
    let tampered_path = scratch.join("release with trailing byte.json");
    std::fs::write(&tampered_path, &tampered).unwrap();
    let rejected = install(&tampered_path);
    assert!(!rejected.status.success());
    assert!(rejected.stdout.is_empty());
    let error: Value = serde_json::from_slice(&rejected.stderr).unwrap();
    assert_eq!(error["error"]["version"], 1);
    assert_eq!(error["error"]["ok"], false);
    assert_eq!(error["error"]["category"], "input_invalid");
    assert_eq!(error["error"]["operation"], "install");
    assert_eq!(error["error"]["activation"], "not_activated");
    assert_eq!(error["error"]["inputs"][0]["disposition"], "failed");
    assert!(error["error"]["observed_generation"].is_null());
    let after = json(&run(&[Path::new("collections"), Path::new("list"), &root]));
    assert_eq!(after["local_revision"], revision);
    assert_eq!(after["generation"]["generation_id"], generation_id);
    assert_eq!(std::fs::read(&tampered_path).unwrap(), tampered);
    let stages = std::fs::read_dir(root.join("generations")).unwrap().count();
    assert_eq!(
        stages, 1,
        "invalid input must fail before allocating a stage"
    );
    let _ = std::fs::remove_dir_all(scratch);
}

#[test]
fn test_collections_cli_install_help_explains_selection_and_update_boundary() {
    let output = run(&[
        Path::new("collections"),
        Path::new("install"),
        Path::new("--help"),
    ]);
    assert!(output.status.success());
    let help = String::from_utf8(output.stdout).unwrap();
    assert!(help.contains("--manifest"));
    assert!(help.contains("--all"));
    assert!(help.contains("--pack"));
    assert!(help.contains("all four pack records"));
    assert!(help.contains("update_required"));
    assert!(help.contains("default store"));
}
