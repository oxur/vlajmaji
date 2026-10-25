//! Process-level contract tests for the explicit store verification CLI.

use serde_json::Value;
use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};
use vlajmaji_store::DictionaryStore;

#[path = "../../lexicon/examples/support/dictionary_fixture.rs"]
mod fixture_support;

fn scratch() -> PathBuf {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let path = std::env::temp_dir().join(format!(
        "vlajmaji-cli-verify-{}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&path).unwrap();
    path
}

fn invoke(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn test_verify_cli_success_failure_envelope_default_help_and_no_create() {
    let base = scratch();
    let store = base.join("valid store");
    let dictionary = fixture_support::complete_fixture().unwrap();
    DictionaryStore::create(&store, &dictionary).unwrap();
    let output = invoke(&["verify", "all", store.to_str().unwrap()]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["operation"], "verify");
    assert_eq!(report["ok"], true);
    assert_eq!(report["checks"].as_array().unwrap().len(), 6);

    let broken = base.join("bad store");
    fs::create_dir(&broken).unwrap();
    fs::copy(
        store.join("dictionary.sqlite3"),
        broken.join("dictionary.sqlite3"),
    )
    .unwrap();
    let tamper = Command::new("sqlite3")
        .arg(broken.join("dictionary.sqlite3"))
        .arg("UPDATE metadata SET value='wrong' WHERE key='format';")
        .output()
        .unwrap();
    assert!(
        tamper.status.success(),
        "sqlite3 test fixture tamper: {}",
        String::from_utf8_lossy(&tamper.stderr)
    );
    let failed = invoke(&["verify", "all", broken.to_str().unwrap()]);
    assert!(!failed.status.success());
    assert!(
        failed.stdout.is_empty(),
        "failed verification must not write JSON to stdout"
    );
    let envelope: Value = serde_json::from_slice(&failed.stderr).unwrap();
    assert_eq!(envelope["error"]["code"], "verification_failed");
    assert_eq!(
        envelope["verification"]["checks"].as_array().unwrap().len(),
        6
    );

    let missing = base.join("missing store");
    let missing_output = invoke(&["verify", "schema", missing.to_str().unwrap()]);
    assert!(!missing_output.status.success());
    assert!(!missing.exists(), "verify must not create its store path");

    for args in [
        vec!["--help"],
        vec!["info", "--help"],
        vec!["dictionary-build", "--help"],
        vec!["dictionary-query", "--help"],
        vec!["verify", "--help"],
    ] {
        let output = invoke(&args);
        assert!(output.status.success());
        let help = String::from_utf8(output.stdout).unwrap();
        assert!(
            help.contains("vlajmaji/stores/default"),
            "missing default path in {args:?}: {help}"
        );
        assert!(
            help.contains("XDG_DATA_HOME"),
            "missing XDG explanation in {args:?}: {help}"
        );
        assert!(
            help.contains("HOME"),
            "missing HOME fallback in {args:?}: {help}"
        );
        assert!(
            help.contains("future dictionary-of-dictionaries"),
            "missing provisional intent in {args:?}: {help}"
        );
    }
    fs::remove_dir_all(base).unwrap();
}

#[test]
fn test_verify_cli_explicit_path_overrides_invalid_default_environment() {
    let base = scratch();
    let store = base.join("explicit");
    DictionaryStore::create(&store, &fixture_support::complete_fixture().unwrap()).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
        .args(["verify", "metadata", store.to_str().unwrap()])
        .env("XDG_DATA_HOME", base.join("wrong-default"))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["checks"].as_array().unwrap().len(), 1);
    assert_eq!(report["checks"][0]["check"], "metadata");
    assert_eq!(report["ok"], true);
    assert!(!base.join("wrong-default").exists());
    fs::remove_dir_all(base).unwrap();
}
