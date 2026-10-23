//! End-to-end JSON CLI behavior over a small canonical archive.

use std::{
    path::PathBuf,
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};

use serde_json::{Value, json};
use vlajmaji_lexicon::{Lexicon, SourceFormat, SourceMetadata, TimePrecision};

static NEXT: AtomicUsize = AtomicUsize::new(0);

fn temp_root() -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "relojban-search-cli-test-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&path).unwrap();
    path
}

fn archive_bytes() -> Vec<u8> {
    let mut lexicon = Lexicon::new();
    lexicon
        .import_source(
            SourceMetadata::new(
                "cli-test",
                "CLI test fixture",
                SourceFormat::LaLojbanJson,
                "fixture.json",
                "v1",
                None,
                TimePrecision::Unknown,
            ),
            br#"[{"word":"farvi","word_type":"gismu","definition":"grow"}]"#,
            None,
        )
        .unwrap();
    lexicon.to_archive_bytes().unwrap()
}

fn run(args: &[&str], input: Option<&str>) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_relojban-search"));
    command.args(args);
    command
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    if input.is_some() {
        command.stdin(std::process::Stdio::piped());
    }
    let mut child = command.spawn().unwrap();
    if let Some(input) = input {
        use std::io::Write;
        child
            .stdin
            .take()
            .unwrap()
            .write_all(input.as_bytes())
            .unwrap();
    }
    child.wait_with_output().unwrap()
}

#[test]
fn help_and_version_are_available() {
    assert!(run(&["--help"], None).status.success());
    assert!(run(&["--version"], None).status.success());
}

#[test]
fn valid_json_limit_zero_and_failures_are_inspectable() {
    let root = temp_root();
    let archive_path = root.join("input.archive");
    let store = root.join("store");
    std::fs::write(&archive_path, archive_bytes()).unwrap();
    let archive = archive_path.to_str().unwrap();
    let store_str = store.to_str().unwrap();

    let build = run(&["build", archive, store_str], None);
    assert!(
        build.status.success(),
        "{}",
        String::from_utf8_lossy(&build.stderr)
    );
    let built: Value = serde_json::from_slice(&build.stdout).unwrap();
    assert_eq!(built["records"], 1);

    let query = json!({"operation":"exact","spelling":"farvi"}).to_string();
    let valid = run(&["search", store_str, "--limit", "0"], Some(&query));
    assert!(
        valid.status.success(),
        "{}",
        String::from_utf8_lossy(&valid.stderr)
    );
    let response: Value = serde_json::from_slice(&valid.stdout).unwrap();
    assert_eq!(response["total_records"], 1);
    assert_eq!(response["hits"].as_array().unwrap().len(), 0);

    let info = run(&["info", store_str], None);
    assert!(info.status.success());
    let info_value: Value = serde_json::from_slice(&info.stdout).unwrap();
    assert_eq!(info_value["records"], 1);

    let malformed = run(&["search", store_str], Some("{"));
    assert!(!malformed.status.success());
    assert!(!malformed.stderr.is_empty());

    let invalid_query = run(&["search", store_str], Some(r#"{"operation":"unknown"}"#));
    assert!(!invalid_query.status.success());
    assert!(!invalid_query.stderr.is_empty());

    let unknown_kind = json!({
        "operation":"prosody",
        "kind":"not-a-mapped-kind",
        "syllables":null,
        "nuclei":null,
        "primary_nucleus":null
    })
    .to_string();
    let invalid_kind = run(&["search", store_str], Some(&unknown_kind));
    assert!(!invalid_kind.status.success());
    assert!(!invalid_kind.stderr.is_empty());

    let large_limit = run(&["search", store_str, "--limit", "1001"], Some(&query));
    assert!(
        large_limit.status.success(),
        "{}",
        String::from_utf8_lossy(&large_limit.stderr)
    );
    let large_value: Value = serde_json::from_slice(&large_limit.stdout).unwrap();
    assert_eq!(large_value["total_records"], 1);
    assert_eq!(large_value["hits"].as_array().unwrap().len(), 1);
    let invalid_limit = run(&["search", store_str, "--limit", "-1"], Some(&query));
    assert!(!invalid_limit.status.success());
    assert!(!invalid_limit.stderr.is_empty());

    let invalid_argument = run(&["search", store_str, "--unknown-option"], None);
    assert!(!invalid_argument.status.success());
    assert!(!invalid_argument.stderr.is_empty());

    let missing_store = root.join("missing");
    let invalid_store = run(&["search", missing_store.to_str().unwrap()], Some(&query));
    assert!(!invalid_store.status.success());
    assert!(!invalid_store.stderr.is_empty());

    let second_build = run(&["build", archive, store_str], None);
    assert!(!second_build.status.success());
    assert!(!second_build.stderr.is_empty());
    let after_existing_path = run(&["info", store_str], None);
    assert!(after_existing_path.status.success());
    let still_valid: Value = serde_json::from_slice(&after_existing_path.stdout).unwrap();
    assert_eq!(still_valid["records"], 1);

    let non_ascii =
        json!({"operation":"spelling_distance","anchor":"žanru","max_distance":0}).to_string();
    let non_ascii_result = run(&["search", store_str, "--limit", "1"], Some(&non_ascii));
    assert!(non_ascii_result.status.success());
    let non_ascii_response: Value = serde_json::from_slice(&non_ascii_result.stdout).unwrap();
    assert_eq!(non_ascii_response["total_records"], 0);

    std::fs::remove_dir_all(root).unwrap();
}
