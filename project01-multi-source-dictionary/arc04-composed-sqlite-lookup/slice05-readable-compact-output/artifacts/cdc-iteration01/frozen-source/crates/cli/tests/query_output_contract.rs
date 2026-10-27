//! Compact query and detail CLI routes preserve full defaults and strict keys.

#[path = "../../lexicon/examples/support/dictionary_fixture.rs"]
mod fixture;

use std::{
    borrow::Cow,
    collections::BTreeSet,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use serde_json::Value;
use vlajmaji_lexicon::{
    Dictionary, LocalRef, PackId, PackPayloadKind, RelationTarget, seal_release,
};
use vlajmaji_search::DictionaryRequest;
use vlajmaji_store::{
    CollectionCatalogue, CollectionInput, CollectionRequest, CollectionSelection,
};

static NEXT: AtomicUsize = AtomicUsize::new(0);

fn scratch() -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "vlajmaji-s05-query-output-{}-{stamp}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir(&root).unwrap();
    root
}

fn run(args: &[&Path]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
        .args(args)
        .env_clear()
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .unwrap()
}

fn run_strings(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
        .args(args)
        .env_clear()
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .unwrap()
}

#[cfg(target_os = "linux")]
fn run_os(args: &[std::ffi::OsString]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
        .args(args)
        .env_clear()
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .unwrap()
}

#[cfg(unix)]
fn run_after_removing_cwd(cwd: &Path, args: &[std::ffi::OsString]) -> Output {
    let mut command = Command::new("/bin/sh");
    command
        .current_dir(cwd)
        .args(["-c", "rmdir \"$1\" || exit; shift; exec \"$@\"", "sh"])
        .arg(cwd)
        .arg(env!("CARGO_BIN_EXE_vlajmaji"))
        .args(args)
        .env_clear()
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    command.output().unwrap()
}

fn assert_json_keys(value: &Value, expected: &[&str]) {
    let actual: BTreeSet<_> = value
        .as_object()
        .expect("response is a JSON object")
        .keys()
        .map(String::as_str)
        .collect();
    let expected: BTreeSet<_> = expected.iter().copied().collect();
    assert_eq!(actual, expected);
}

fn install(root: &Path) -> PathBuf {
    let mut document = fixture::complete_fixture().unwrap().into_document();
    document.legacy_archive = None;
    let dictionary = Dictionary::from_document(document).unwrap();
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
    let report = CollectionCatalogue::install(root, request).unwrap();
    report.generation.unwrap().store_path
}

#[test]
fn test_dictionary_compact_query_and_detail_keep_full_default() {
    let root = scratch();
    let input = root.join("dictionary.json");
    let store = root.join("dictionary-store");
    let bytes = fixture::complete_fixture()
        .unwrap()
        .to_json_bytes()
        .unwrap();
    std::fs::write(&input, bytes).unwrap();
    let build = run(&[Path::new("dictionary-build"), &input, &store]);
    assert!(
        build.status.success(),
        "{}",
        String::from_utf8_lossy(&build.stderr)
    );

    let full = run_strings(&[
        "dictionary-query",
        store.to_str().unwrap(),
        "--request",
        r#"{"operation":"exact","spelling":"farvi"}"#,
    ]);
    assert!(
        full.status.success(),
        "{}",
        String::from_utf8_lossy(&full.stderr)
    );
    let full: Value = serde_json::from_slice(&full.stdout).unwrap();
    assert!(full["groups"][0]["hits"][0].get("assertion").is_some());
    let explicit_full = run_strings(&[
        "dictionary-query",
        store.to_str().unwrap(),
        "--request",
        r#"{"operation":"exact","spelling":"farvi"}"#,
        "--format",
        "full",
    ]);
    assert!(explicit_full.status.success());
    assert_eq!(
        full,
        serde_json::from_slice::<Value>(&explicit_full.stdout).unwrap()
    );

    let compact = run_strings(&[
        "dictionary-query",
        store.to_str().unwrap(),
        "--request",
        r#"{"operation":"exact","spelling":"farvi"}"#,
        "--format",
        "compact",
    ]);
    assert!(
        compact.status.success(),
        "{}",
        String::from_utf8_lossy(&compact.stderr)
    );
    let compact: Value = serde_json::from_slice(&compact.stdout).unwrap();
    assert_json_keys(
        &compact,
        &[
            "version",
            "projection_policy",
            "policy",
            "store_schema",
            "catalogue_revision",
            "state_sha256",
            "snapshot_sha256",
            "spelling",
            "resolved_sources",
            "native_candidate_occurrences",
            "totals",
            "source_totals",
            "availability",
            "groups",
            "next_cursor",
            "code_notices",
        ],
    );
    assert!(compact.get("operation").is_none());
    assert!(compact.get("response").is_none());
    let detail_ref = compact["groups"][0]["hits"][0]["detail_ref"].clone();
    assert!(compact["groups"][0]["hits"][0].get("raw").is_none());
    assert!(compact["groups"][0]["hits"][0].get("contexts").is_none());
    let detail = run_strings(&[
        "dictionary-detail",
        store.to_str().unwrap(),
        "--reference",
        &detail_ref.to_string(),
        "--expected-snapshot",
        compact["snapshot_sha256"].as_str().unwrap(),
    ]);
    assert!(
        detail.status.success(),
        "{}",
        String::from_utf8_lossy(&detail.stderr)
    );
    let detail: Value = serde_json::from_slice(&detail.stdout).unwrap();
    assert_eq!(detail["hit"]["assertion"]["id"], detail_ref["assertion_id"]);

    let mut invalid_detail_cases = vec![
        {
            let mut value = detail_ref.clone();
            value["role"] = Value::String("entry".into());
            value
        },
        {
            let mut value = detail_ref.clone();
            value["version"] = Value::from(2);
            value
        },
        {
            let mut value = detail_ref.clone();
            value.as_object_mut().unwrap().remove("native_entry_id");
            value
        },
    ];
    for required_nullable in ["native_assertion_id", "contributor_id", "distributor_id"] {
        let mut missing = detail_ref.clone();
        missing.as_object_mut().unwrap().remove(required_nullable);
        invalid_detail_cases.push(missing);
        let mut wrong_type = detail_ref.clone();
        wrong_type[required_nullable] = Value::from(7);
        invalid_detail_cases.push(wrong_type);
    }
    for (field, value) in [
        ("source", Value::String("invalid source id".into())),
        ("entry_id", Value::String("invalid entry id".into())),
        ("assertion_id", Value::String("invalid assertion id".into())),
    ] {
        let mut invalid_id = detail_ref.clone();
        invalid_id[field] = value;
        invalid_detail_cases.push(invalid_id);
    }
    let mut empty_locator = detail_ref.clone();
    empty_locator["occurrence"]["locator"] = Value::String(String::new());
    invalid_detail_cases.push(empty_locator);
    let mut empty_capture_namespace = detail_ref.clone();
    empty_capture_namespace["occurrence"]["capture"]["namespace"] = Value::String(String::new());
    invalid_detail_cases.push(empty_capture_namespace);
    let mut invalid_capture_digest = detail_ref.clone();
    invalid_capture_digest["occurrence"]["capture"]["sha256"] = Value::String("A".repeat(64));
    invalid_detail_cases.push(invalid_capture_digest);
    for invalid_reference in invalid_detail_cases {
        let invalid = run_strings(&[
            "dictionary-detail",
            root.join("missing-store").to_str().unwrap(),
            "--reference",
            &invalid_reference.to_string(),
        ]);
        assert!(!invalid.status.success());
        let error: Value = serde_json::from_slice(&invalid.stderr).unwrap();
        assert_eq!(error["error"]["code"], "invalid_detail_reference");
    }

    let dict = fixture::complete_fixture().unwrap();
    let entry = dict.document().metadata.entries[0].id.clone();
    let resolve_request = serde_json::to_string(&DictionaryRequest::Resolve {
        target: RelationTarget::Local(LocalRef::Entry(entry)),
    })
    .unwrap();
    let resolved = run_strings(&[
        "dictionary-query",
        store.to_str().unwrap(),
        "--request",
        &resolve_request,
        "--format",
        "compact",
    ]);
    assert!(
        resolved.status.success(),
        "{}",
        String::from_utf8_lossy(&resolved.stderr)
    );
    let resolved: Value = serde_json::from_slice(&resolved.stdout).unwrap();
    assert_json_keys(
        &resolved,
        &[
            "version",
            "projection_policy",
            "state_sha256",
            "snapshot_sha256",
            "target",
            "status",
            "entity",
            "confirmed",
            "unverified",
            "availability",
            "requested_release",
        ],
    );
    assert!(resolved.get("operation").is_none());
    assert!(resolved.get("response").is_none());
    assert!(resolved["target"].is_object());
    assert!(resolved["status"].is_string());

    let malformed = run_strings(&[
        "dictionary-detail",
        root.join("missing-store").to_str().unwrap(),
        "--reference",
        "{",
    ]);
    assert!(!malformed.status.success());
    let error: Value = serde_json::from_slice(&malformed.stderr).unwrap();
    assert_eq!(error["error"]["code"], "invalid_detail_reference");
    let trailing = run_strings(&[
        "dictionary-detail",
        root.join("missing-store").to_str().unwrap(),
        "--reference",
        "{} {}",
    ]);
    assert!(!trailing.status.success());
    let error: Value = serde_json::from_slice(&trailing.stderr).unwrap();
    assert_eq!(error["error"]["code"], "invalid_detail_reference");
    let managed_malformed = run_strings(&[
        "lookup-detail",
        "--reference",
        "{",
        "--config",
        root.join("missing-config.toml").to_str().unwrap(),
        "--profile",
        "requested",
        "--pack",
        "llg",
    ]);
    assert!(!managed_malformed.status.success());
    let managed_error: Value = serde_json::from_slice(&managed_malformed.stderr).unwrap();
    assert_eq!(
        managed_error["error"]["category"],
        "invalid_detail_reference"
    );
    assert_eq!(managed_error["error"]["operation"], "lookup_detail");
    assert_eq!(managed_error["error"]["outcome"], "not_applicable");
    assert!(managed_error["error"]["snapshot"].is_null());
    assert_eq!(managed_error["error"]["config_path"], Value::Null);
    assert_eq!(managed_error["error"]["root_path"], Value::Null);
    assert_eq!(managed_error["error"]["profile"], Value::Null);
    assert!(managed_error["error"]["requested_profile"].is_null());
    assert_eq!(
        managed_error["error"]["known_pack_ids"],
        serde_json::json!([])
    );
    assert!(managed_error["error"]["writer"].is_null());
    for key in [
        "version",
        "ok",
        "category",
        "detail",
        "operation",
        "config_path",
        "root_path",
        "profile",
        "requested_profile",
        "snapshot",
        "known_pack_ids",
        "outcome",
        "writer",
    ] {
        assert!(
            managed_error["error"].get(key).is_some(),
            "missing field {key}"
        );
    }
    let malformed_snapshot = run_strings(&[
        "lookup-detail",
        "--reference",
        &detail_ref.to_string(),
        "--expected-snapshot",
        "not-a-digest",
        "--config",
        root.join("missing-config.toml").to_str().unwrap(),
        "--profile",
        "requested",
        "--pack",
        "llg",
    ]);
    assert!(!malformed_snapshot.status.success());
    let malformed_snapshot: Value = serde_json::from_slice(&malformed_snapshot.stderr).unwrap();
    assert_eq!(
        malformed_snapshot["error"]["category"],
        "malformed_snapshot"
    );
    assert!(malformed_snapshot["error"]["config_path"].is_null());
    assert!(malformed_snapshot["error"]["root_path"].is_null());
    assert!(malformed_snapshot["error"]["profile"].is_null());
    assert!(malformed_snapshot["error"]["requested_profile"].is_null());
    assert!(malformed_snapshot["error"]["snapshot"].is_null());
    assert_eq!(
        malformed_snapshot["error"]["known_pack_ids"],
        serde_json::json!([])
    );
    assert!(malformed_snapshot["error"]["writer"].is_null());
    let mut opaque_unicode_namespace = detail_ref.clone();
    opaque_unicode_namespace["occurrence"]["capture"]["namespace"] =
        Value::String("opaque/λ/é".into());
    let unicode_static_reference = run_strings(&[
        "lookup-detail",
        "--reference",
        &opaque_unicode_namespace.to_string(),
        "--config",
        root.join("missing-config.toml").to_str().unwrap(),
        "--profile",
        "requested",
        "--pack",
        "llg",
    ]);
    assert!(!unicode_static_reference.status.success());
    let unicode_error: Value = serde_json::from_slice(&unicode_static_reference.stderr).unwrap();
    assert_eq!(unicode_error["error"]["category"], "config_missing");
    assert_eq!(unicode_error["error"]["operation"], "lookup_detail");
    assert_eq!(unicode_error["error"]["outcome"], "not_applicable");
    let wrong_role = {
        let mut value = detail_ref.clone();
        value["role"] = Value::String("entry".into());
        value
    };
    let managed_invalid = run_strings(&[
        "lookup-detail",
        "--reference",
        &wrong_role.to_string(),
        "--config",
        root.join("missing-config.toml").to_str().unwrap(),
        "--profile",
        "requested",
        "--pack",
        "llg",
    ]);
    assert!(!managed_invalid.status.success());
    let managed_invalid: Value = serde_json::from_slice(&managed_invalid.stderr).unwrap();
    assert_eq!(
        managed_invalid["error"]["category"],
        "invalid_detail_reference"
    );
    assert!(managed_invalid["error"]["config_path"].is_null());
    assert!(managed_invalid["error"]["root_path"].is_null());
    assert!(managed_invalid["error"]["profile"].is_null());
    assert!(managed_invalid["error"]["requested_profile"].is_null());
    assert!(managed_invalid["error"]["snapshot"].is_null());
    assert_eq!(
        managed_invalid["error"]["known_pack_ids"],
        serde_json::json!([])
    );
    assert_eq!(managed_invalid["error"]["outcome"], "not_applicable");
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn test_lookup_compact_and_detail_preserve_managed_envelope() {
    let base = scratch();
    let root = base.join("catalogue");
    let store_path = install(&root);
    let config = base.join("config.toml");
    let configure = run_strings(&[
        "collections",
        "configure",
        root.to_str().unwrap(),
        "--profile",
        "compact-test",
        "--pack",
        "llg",
        "--make-default",
        "--config",
        config.to_str().unwrap(),
    ]);
    assert!(
        configure.status.success(),
        "{}",
        String::from_utf8_lossy(&configure.stderr)
    );

    let lookup = run_strings(&[
        "lookup",
        "farvi",
        "--config",
        config.to_str().unwrap(),
        "--profile",
        "compact-test",
        "--format",
        "compact",
    ]);
    assert!(
        lookup.status.success(),
        "{}",
        String::from_utf8_lossy(&lookup.stderr)
    );
    let lookup: Value = serde_json::from_slice(&lookup.stdout).unwrap();
    assert_eq!(lookup["result"]["operation"], "exact");
    assert!(lookup["snapshot"]["binding_sha256"].is_string());
    let detail_ref = lookup["result"]["response"]["groups"][0]["hits"][0]["detail_ref"].clone();
    let detail = run_strings(&[
        "lookup-detail",
        "--reference",
        &detail_ref.to_string(),
        "--expected-snapshot",
        lookup["snapshot"]["binding_sha256"].as_str().unwrap(),
        "--config",
        config.to_str().unwrap(),
        "--profile",
        "compact-test",
    ]);
    assert!(
        detail.status.success(),
        "{}",
        String::from_utf8_lossy(&detail.stderr)
    );
    let detail: Value = serde_json::from_slice(&detail.stdout).unwrap();
    assert_eq!(
        detail["response"]["hit"]["assertion"]["id"],
        detail_ref["assertion_id"]
    );
    assert!(detail["snapshot"]["binding_sha256"].is_string());
    assert!(store_path.exists());
    std::fs::remove_dir_all(base).unwrap();
}

#[cfg(unix)]
#[test]
fn test_managed_compact_lookup_and_detail_resolve_absolute_paths_after_deleted_cwd() {
    let base = scratch();
    let root = base.join("catalogue");
    install(&root);
    let config = base.join("config.toml");
    let configure = run_strings(&[
        "collections",
        "configure",
        root.to_str().unwrap(),
        "--profile",
        "compact-cwd",
        "--pack",
        "llg",
        "--make-default",
        "--config",
        config.to_str().unwrap(),
    ]);
    assert!(
        configure.status.success(),
        "{}",
        String::from_utf8_lossy(&configure.stderr)
    );

    let absolute_cwd = base.join("absolute-child-cwd");
    std::fs::create_dir(&absolute_cwd).unwrap();
    let absolute_lookup_args = vec![
        "lookup".into(),
        "farvi".into(),
        "--config".into(),
        config.as_os_str().to_owned(),
        "--profile".into(),
        "compact-cwd".into(),
        "--format".into(),
        "compact".into(),
    ];
    let lookup = run_after_removing_cwd(&absolute_cwd, &absolute_lookup_args);
    assert!(
        lookup.status.success(),
        "{}",
        String::from_utf8_lossy(&lookup.stderr)
    );
    assert!(!absolute_cwd.exists());
    let lookup: Value = serde_json::from_slice(&lookup.stdout).unwrap();
    let detail_ref = lookup["result"]["response"]["groups"][0]["hits"][0]["detail_ref"].clone();
    let expected_snapshot = lookup["snapshot"]["binding_sha256"].as_str().unwrap();

    let detail_cwd = base.join("detail-child-cwd");
    std::fs::create_dir(&detail_cwd).unwrap();
    let absolute_detail_args = vec![
        "lookup-detail".into(),
        "--reference".into(),
        detail_ref.to_string().into(),
        "--expected-snapshot".into(),
        expected_snapshot.into(),
        "--config".into(),
        config.as_os_str().to_owned(),
        "--profile".into(),
        "compact-cwd".into(),
    ];
    let detail = run_after_removing_cwd(&detail_cwd, &absolute_detail_args);
    assert!(
        detail.status.success(),
        "{}",
        String::from_utf8_lossy(&detail.stderr)
    );
    assert!(!detail_cwd.exists());
    let detail: Value = serde_json::from_slice(&detail.stdout).unwrap();
    assert_eq!(
        detail["response"]["hit"]["assertion"]["id"],
        detail_ref["assertion_id"]
    );

    let relative_cwd = base.join("relative-child-cwd");
    std::fs::create_dir(&relative_cwd).unwrap();
    let relative_lookup_args = vec![
        "lookup".into(),
        "farvi".into(),
        "--config".into(),
        "relative.toml".into(),
        "--profile".into(),
        "compact-cwd".into(),
        "--format".into(),
        "compact".into(),
    ];
    let relative = run_after_removing_cwd(&relative_cwd, &relative_lookup_args);
    assert!(!relative.status.success());
    assert!(!relative_cwd.exists());
    let error: Value = serde_json::from_slice(&relative.stderr).unwrap();
    assert_eq!(error["error"]["category"], "config_io");
    assert_eq!(error["error"]["operation"], "lookup");
    assert_eq!(error["error"]["config_path"]["utf8"], "relative.toml");
    assert_eq!(error["error"]["outcome"], "not_applicable");
    std::fs::remove_dir_all(base).unwrap();
}

#[cfg(target_os = "linux")]
#[test]
fn test_managed_compact_routes_roundtrip_native_root_and_config() {
    use std::os::unix::ffi::OsStringExt;

    let base = scratch();
    let native_root = base.join(std::ffi::OsString::from_vec(b"catalogue-\xff".to_vec()));
    install(&native_root);
    let native_config = base.join(std::ffi::OsString::from_vec(b"profiles-\xfe.toml".to_vec()));
    let configure_args = vec![
        "collections".into(),
        "configure".into(),
        native_root.as_os_str().to_owned(),
        "--profile".into(),
        "native-compact".into(),
        "--pack".into(),
        "llg".into(),
        "--make-default".into(),
        "--config".into(),
        native_config.as_os_str().to_owned(),
    ];
    let configure = run_os(&configure_args);
    assert!(
        configure.status.success(),
        "{}",
        String::from_utf8_lossy(&configure.stderr)
    );
    assert!(native_config.exists());

    let lookup_args = vec![
        "lookup".into(),
        "farvi".into(),
        "--config".into(),
        native_config.as_os_str().to_owned(),
        "--profile".into(),
        "native-compact".into(),
        "--format".into(),
        "compact".into(),
    ];
    let lookup = run_os(&lookup_args);
    assert!(
        lookup.status.success(),
        "{}",
        String::from_utf8_lossy(&lookup.stderr)
    );
    let lookup: Value = serde_json::from_slice(&lookup.stdout).unwrap();
    let detail_ref = lookup["result"]["response"]["groups"][0]["hits"][0]["detail_ref"].clone();
    let expected_snapshot = lookup["snapshot"]["binding_sha256"].as_str().unwrap();
    let detail_args = vec![
        "lookup-detail".into(),
        "--reference".into(),
        detail_ref.to_string().into(),
        "--expected-snapshot".into(),
        expected_snapshot.into(),
        "--config".into(),
        native_config.as_os_str().to_owned(),
        "--profile".into(),
        "native-compact".into(),
    ];
    let detail = run_os(&detail_args);
    assert!(
        detail.status.success(),
        "{}",
        String::from_utf8_lossy(&detail.stderr)
    );
    let detail: Value = serde_json::from_slice(&detail.stdout).unwrap();
    assert_eq!(
        detail["response"]["hit"]["assertion"]["id"],
        detail_ref["assertion_id"]
    );

    std::fs::remove_dir_all(&native_root).unwrap();
    let missing_root = run_os(&lookup_args);
    assert!(!missing_root.status.success());
    let error: Value = serde_json::from_slice(&missing_root.stderr).unwrap();
    assert_eq!(
        error["error"]["root_path"],
        serde_json::to_value(vlajmaji_store::native_path_report(&native_root)).unwrap()
    );
    assert_eq!(
        error["error"]["config_path"],
        serde_json::to_value(vlajmaji_store::native_path_report(&native_config)).unwrap()
    );
    let lossy_root = std::path::PathBuf::from(native_root.to_string_lossy().into_owned());
    assert!(!lossy_root.exists());
    std::fs::remove_dir_all(base).unwrap();
}

#[test]
fn test_managed_compact_lookup_and_detail_tolerate_broken_pipe() {
    let base = scratch();
    let root = base.join("catalogue");
    install(&root);
    let config = base.join("config.toml");
    let configure = run_strings(&[
        "collections",
        "configure",
        root.to_str().unwrap(),
        "--profile",
        "compact-test",
        "--pack",
        "llg",
        "--make-default",
        "--config",
        config.to_str().unwrap(),
    ]);
    assert!(
        configure.status.success(),
        "{}",
        String::from_utf8_lossy(&configure.stderr)
    );
    let lookup_args = [
        "lookup",
        "farvi",
        "--config",
        config.to_str().unwrap(),
        "--profile",
        "compact-test",
        "--format",
        "compact",
    ];
    let lookup = run_strings(&lookup_args);
    assert!(
        lookup.status.success(),
        "{}",
        String::from_utf8_lossy(&lookup.stderr)
    );
    let response: Value = serde_json::from_slice(&lookup.stdout).unwrap();
    let detail_ref =
        response["result"]["response"]["groups"][0]["hits"][0]["detail_ref"].to_string();
    let detail_args = [
        "lookup-detail",
        "--reference",
        detail_ref.as_str(),
        "--expected-snapshot",
        response["snapshot"]["binding_sha256"].as_str().unwrap(),
        "--config",
        config.to_str().unwrap(),
        "--profile",
        "compact-test",
    ];
    let detail = run_strings(&detail_args);
    assert!(
        detail.status.success(),
        "{}",
        String::from_utf8_lossy(&detail.stderr)
    );

    for args in [&lookup_args[..], &detail_args[..]] {
        let mut child = Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
            .args(args)
            .env_clear()
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        drop(child.stdout.take());
        let result = child.wait_with_output().unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert!(result.stderr.is_empty());
    }
    std::fs::remove_dir_all(base).unwrap();
}

#[cfg(target_os = "linux")]
#[test]
fn test_dictionary_compact_query_reports_full_device_output_failure() {
    let base = scratch();
    let store = base.join("dictionary-store");
    let dictionary = fixture::complete_fixture().unwrap();
    vlajmaji_store::DictionaryStore::create(&store, &dictionary).unwrap();
    let request = r#"{"operation":"exact","spelling":"farvi"}"#;

    let ordinary = run_strings(&[
        "dictionary-query",
        store.to_str().unwrap(),
        "--request",
        request,
        "--format",
        "compact",
    ]);
    assert!(
        ordinary.status.success(),
        "{}",
        String::from_utf8_lossy(&ordinary.stderr)
    );
    let response: Value = serde_json::from_slice(&ordinary.stdout).unwrap();
    assert!(response.get("operation").is_none());
    assert!(response.get("response").is_none());
    assert!(!response["groups"].as_array().unwrap().is_empty());
    let detail_ref = response["groups"][0]["hits"][0]["detail_ref"].to_string();
    let detail_args = [
        "dictionary-detail",
        store.to_str().unwrap(),
        "--reference",
        &detail_ref,
    ];
    let ordinary_detail = run_strings(&detail_args);
    assert!(
        ordinary_detail.status.success(),
        "{}",
        String::from_utf8_lossy(&ordinary_detail.stderr)
    );
    let detail: Value = serde_json::from_slice(&ordinary_detail.stdout).unwrap();
    assert!(detail["hit"].is_object());

    let full = std::fs::OpenOptions::new()
        .write(true)
        .open("/dev/full")
        .unwrap();
    let failed = Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
        .args([
            "dictionary-query",
            store.to_str().unwrap(),
            "--request",
            request,
            "--format",
            "compact",
        ])
        .env_clear()
        .stdout(full)
        .stderr(std::process::Stdio::piped())
        .output()
        .unwrap();
    assert!(!failed.status.success());
    let stderr = String::from_utf8_lossy(&failed.stderr);
    assert!(stderr.contains("write JSON:"), "{stderr}");
    assert!(stderr.contains("No space left on device"), "{stderr}");

    let full = std::fs::OpenOptions::new()
        .write(true)
        .open("/dev/full")
        .unwrap();
    let failed_detail = Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
        .args(detail_args)
        .env_clear()
        .stdout(full)
        .stderr(std::process::Stdio::piped())
        .output()
        .unwrap();
    assert!(!failed_detail.status.success());
    let envelope: Value = serde_json::from_slice(&failed_detail.stderr).unwrap();
    let report = envelope["error"].as_object().unwrap();
    assert_eq!(report.len(), 3);
    assert_eq!(report["code"], "response_output_io");
    assert_eq!(report["sources"], serde_json::json!([]));
    assert!(
        report["message"]
            .as_str()
            .unwrap()
            .contains("No space left on device")
    );
    std::fs::remove_dir_all(base).unwrap();
}

#[cfg(target_os = "linux")]
#[test]
fn test_managed_compact_lookup_and_detail_report_full_device_envelopes() {
    let base = scratch();
    let root = base.join("catalogue");
    let store_path = install(&root);
    let config = base.join("config.toml");
    let configure = run_strings(&[
        "collections",
        "configure",
        root.to_str().unwrap(),
        "--profile",
        "compact-test",
        "--pack",
        "llg",
        "--make-default",
        "--config",
        config.to_str().unwrap(),
    ]);
    assert!(
        configure.status.success(),
        "{}",
        String::from_utf8_lossy(&configure.stderr)
    );

    let lookup_args = [
        "lookup",
        "farvi",
        "--config",
        config.to_str().unwrap(),
        "--profile",
        "compact-test",
        "--format",
        "compact",
    ];
    let lookup = run_strings(&lookup_args);
    assert!(
        lookup.status.success(),
        "{}",
        String::from_utf8_lossy(&lookup.stderr)
    );
    let lookup: Value = serde_json::from_slice(&lookup.stdout).unwrap();
    assert_eq!(lookup["result"]["operation"], "exact");
    let detail_ref = lookup["result"]["response"]["groups"][0]["hits"][0]["detail_ref"].clone();
    let detail_ref_text = detail_ref.to_string();
    let detail_args = [
        "lookup-detail",
        "--reference",
        &detail_ref_text,
        "--expected-snapshot",
        lookup["snapshot"]["binding_sha256"].as_str().unwrap(),
        "--config",
        config.to_str().unwrap(),
        "--profile",
        "compact-test",
    ];
    let detail = run_strings(&detail_args);
    assert!(
        detail.status.success(),
        "{}",
        String::from_utf8_lossy(&detail.stderr)
    );
    let detail: Value = serde_json::from_slice(&detail.stdout).unwrap();
    assert_eq!(
        detail["response"]["hit"]["assertion"]["id"],
        detail_ref["assertion_id"]
    );

    let native_path = |path: &Path| {
        let bytes = path.as_os_str().as_encoded_bytes();
        serde_json::json!({
            "encoding": "unix-bytes",
            "hex": bytes.iter().map(|byte| format!("{byte:02x}")).collect::<String>(),
            "utf8": path.to_str(),
        })
    };
    let assert_managed_error =
        |failed: &Output, operation: &str, expected_snapshot: &Value, expected_packs: &Value| {
            assert!(!failed.status.success());
            let envelope: Value = serde_json::from_slice(&failed.stderr).unwrap();
            let report = &envelope["error"];
            for key in [
                "version",
                "ok",
                "category",
                "detail",
                "operation",
                "config_path",
                "root_path",
                "profile",
                "requested_profile",
                "snapshot",
                "known_pack_ids",
                "outcome",
                "writer",
            ] {
                assert!(report.get(key).is_some(), "missing report key {key}");
            }
            assert_eq!(report["version"], 1);
            assert_eq!(report["ok"], false);
            assert_eq!(report["category"], "response_output_io");
            assert_eq!(report["operation"], operation);
            assert_eq!(report["config_path"], native_path(&config));
            assert_eq!(report["root_path"], native_path(&root));
            assert_eq!(report["requested_profile"], "compact-test");
            assert_eq!(report["known_pack_ids"], expected_packs);
            assert_eq!(report["outcome"], "not_applicable");
            assert_eq!(report["snapshot"], expected_snapshot);
            assert!(report["profile"].is_object());
            assert!(report["writer"].is_null());
            assert!(
                report["detail"]
                    .as_str()
                    .unwrap()
                    .contains("No space left on device")
            );
        };

    let full_device = || {
        std::fs::OpenOptions::new()
            .write(true)
            .open("/dev/full")
            .unwrap()
    };
    let failed_lookup = Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
        .args(&lookup_args)
        .env_clear()
        .stdout(full_device())
        .stderr(std::process::Stdio::piped())
        .output()
        .unwrap();
    assert_managed_error(
        &failed_lookup,
        "lookup",
        &lookup["snapshot"],
        &lookup["snapshot"]["collections"]
            .as_array()
            .unwrap()
            .iter()
            .map(|row| row["pack_id"].clone())
            .collect::<Value>(),
    );

    let failed_detail = Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
        .args(&detail_args)
        .env_clear()
        .stdout(full_device())
        .stderr(std::process::Stdio::piped())
        .output()
        .unwrap();
    assert_managed_error(
        &failed_detail,
        "lookup_detail",
        &detail["snapshot"],
        &detail["snapshot"]["collections"]
            .as_array()
            .unwrap()
            .iter()
            .map(|row| row["pack_id"].clone())
            .collect::<Value>(),
    );
    assert!(store_path.exists());
    std::fs::remove_dir_all(base).unwrap();
}
