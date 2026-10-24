//! End-to-end dictionary command and structured-error wire contract.

#[path = "../../lexicon/examples/support/dictionary_fixture.rs"]
mod fixture;

use std::{
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

use serde_json::{Value, json};

static NEXT: AtomicUsize = AtomicUsize::new(0);

fn scratch() -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "vlajmaji-s04-cli-dictionary-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&path).unwrap();
    path
}

fn run(args: &[&str], stdin: Option<&str>) -> std::process::Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_vlajmaji"));
    command
        .args(args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    if stdin.is_some() {
        command.stdin(std::process::Stdio::piped());
    }
    let mut child = command.spawn().unwrap();
    if let Some(stdin) = stdin {
        use std::io::Write;
        child
            .stdin
            .take()
            .unwrap()
            .write_all(stdin.as_bytes())
            .unwrap();
    }
    child.wait_with_output().unwrap()
}

#[test]
fn dictionary_commands_work_and_structured_errors_use_the_envelope() {
    let root = scratch();
    let archive = fixture::complete_fixture()
        .unwrap()
        .to_json_bytes()
        .unwrap();
    let input = root.join("dictionary.json");
    let store = root.join("store");
    std::fs::write(&input, archive).unwrap();

    let build = run(
        &[
            "dictionary-build",
            input.to_str().unwrap(),
            store.to_str().unwrap(),
        ],
        None,
    );
    assert!(
        build.status.success(),
        "{}",
        String::from_utf8_lossy(&build.stderr)
    );
    let build_json: Value = serde_json::from_slice(&build.stdout).unwrap();
    assert_eq!(build_json["operation"], "dictionary-build");
    assert_eq!(build_json["occurrences"], 20);

    let request = json!({"operation":"exact","spelling":"farvi"}).to_string();
    let query = run(
        &[
            "dictionary-query",
            store.to_str().unwrap(),
            "--request",
            &request,
        ],
        None,
    );
    assert!(
        query.status.success(),
        "{}",
        String::from_utf8_lossy(&query.stderr)
    );
    let response: Value = serde_json::from_slice(&query.stdout).unwrap();
    assert_eq!(response["totals"]["occurrences"], 6);

    let mut paged_request = json!({
        "operation":"exact",
        "spelling":"farvi",
        "options":{"page_size":2}
    });
    let mut paged_ids = Vec::new();
    loop {
        let page = run(
            &["dictionary-query", store.to_str().unwrap()],
            Some(&paged_request.to_string()),
        );
        assert!(
            page.status.success(),
            "{}",
            String::from_utf8_lossy(&page.stderr)
        );
        let page_json: Value = serde_json::from_slice(&page.stdout).unwrap();
        assert_eq!(page_json["totals"]["occurrences"], 6);
        for group in page_json["groups"].as_array().unwrap() {
            for hit in group["hits"].as_array().unwrap() {
                paged_ids.push(hit["occurrence_id"].as_str().unwrap().to_owned());
            }
        }
        let Some(cursor) = page_json["next_cursor"].as_str().map(ToOwned::to_owned) else {
            break;
        };
        paged_request["options"] = json!({"page_size":3,"cursor":cursor});
    }
    let unpaged_ids = response["groups"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|group| group["hits"].as_array().unwrap())
        .map(|hit| hit["occurrence_id"].as_str().unwrap().to_owned())
        .collect::<Vec<_>>();
    assert_eq!(paged_ids, unpaged_ids);

    let malformed = run(
        &[
            "dictionary-query",
            store.to_str().unwrap(),
            "--request",
            "{",
        ],
        None,
    );
    assert!(!malformed.status.success());
    assert!(malformed.stdout.is_empty());
    let stderr: Value = serde_json::from_slice(&malformed.stderr).unwrap();
    assert_eq!(stderr["error"]["code"], "request_json");
    assert!(stderr["error"]["message"].is_string());
    assert_eq!(stderr["error"]["sources"], json!([]));
    assert!(stderr.get("code").is_none());

    let bad_input = root.join("bad-dictionary.json");
    std::fs::write(&bad_input, b"{").unwrap();
    let bad_build = run(
        &[
            "dictionary-build",
            bad_input.to_str().unwrap(),
            root.join("bad-store").to_str().unwrap(),
        ],
        None,
    );
    assert!(!bad_build.status.success());
    assert!(bad_build.stdout.is_empty());
    let build_error: Value = serde_json::from_slice(&bad_build.stderr).unwrap();
    assert_eq!(build_error["error"]["code"], "invalid_dictionary");

    std::fs::remove_dir_all(root).unwrap();
}
