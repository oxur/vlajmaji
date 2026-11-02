//! Native CLI coverage for computed lexical requests and their strict error boundary.

use std::{
    borrow::Cow,
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};

use sha2::{Digest, Sha256};
use vlajmaji_lexicon::{
    Dictionary, LlgImportOptions, PackId, PackPayloadKind, import_llg_lists, seal_release,
};
use vlajmaji_store::{
    CollectionCatalogue, CollectionInput, CollectionRequest, CollectionSelection,
    WorkbenchBuildRequest, WorkbenchSelection, WorkbenchView,
};

static NEXT_ROOT: AtomicUsize = AtomicUsize::new(0);

fn fresh_root() -> PathBuf {
    for _ in 0..64 {
        let root = std::env::temp_dir().join(format!(
            "vlajmaji-cli-computed-{}-{}",
            std::process::id(),
            NEXT_ROOT.fetch_add(1, Ordering::Relaxed),
        ));
        match fs::create_dir(&root) {
            Ok(()) => return root,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => panic!("create exclusive computed CLI root: {error}"),
        }
    }
    panic!("could not allocate an exclusive computed CLI root after 64 attempts")
}

fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn fixed_dictionary() -> Dictionary {
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
    let notice = b"S06 computed CLI fixture\n";
    import_llg_lists(
        &row,
        empty,
        empty,
        notice,
        &LlgImportOptions {
            expected_gismu_sha256: digest(&row),
            expected_cmavo_sha256: digest(empty),
            expected_rafsi_sha256: digest(empty),
            expected_notice_sha256: digest(notice),
            base_locator: "https://example.invalid/s06-computed-cli".into(),
        },
    )
    .unwrap()
    .into_dictionary()
}

fn build_view(root: &Path, dictionary: &Dictionary) -> PathBuf {
    let sealed = seal_release(PackId::Llg, PackPayloadKind::Synthetic, dictionary).unwrap();
    let catalogue_root = root.join("catalogue");
    let install = CollectionRequest::new(
        vec![CollectionInput::SealedBytes {
            pack_id: PackId::Llg,
            release: Cow::Borrowed(&sealed.release),
            seal: Cow::Borrowed(&sealed.seal),
        }],
        CollectionSelection::Packs(vec![PackId::Llg]),
        true,
    )
    .unwrap();
    CollectionCatalogue::install(&catalogue_root, install).unwrap();
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
        .expect("launch native CLI")
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
        .expect("launch computed CLI after removing its current directory")
}

fn assert_cli_error(output: &Output, expected_code: &str) -> serde_json::Value {
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert_eq!(output.stderr.last(), Some(&b'\n'));
    assert_eq!(
        output.stderr.iter().filter(|byte| **byte == b'\n').count(),
        1
    );
    let envelope: serde_json::Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(envelope.as_object().unwrap().len(), 1);
    assert_eq!(envelope["error"].as_object().unwrap().len(), 3);
    assert_eq!(envelope["error"]["code"], expected_code);
    assert_eq!(envelope["error"]["sources"], serde_json::json!([]));
    assert!(envelope["error"]["message"].is_string());
    envelope
}

#[test]
fn test_computed_cli_preserves_readable_values_and_decodes_before_open() {
    let root = fresh_root();
    let dictionary = fixed_dictionary();
    let expected_assertion_id = dictionary.document().metadata.assertions[0].id.to_string();
    let view = build_view(&root, &dictionary);
    let request_path = root.join("request ü.json");
    fs::write(
        &request_path,
        br#"{"version":1,"query":{"operation":"spelling_distance","anchor":"farvi","max_distance":null},"options":{"sources":["llg"],"alias_scope":"ActiveOnly","page_size":null,"cursor":null}}"#,
    )
    .unwrap();

    let output = cli(&[
        "workbench",
        "lexical",
        view.to_str().unwrap(),
        "--request",
        request_path.to_str().unwrap(),
        "--format",
        "readable",
    ]);
    assert!(
        output.status.success(),
        "computed CLI failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let lines = String::from_utf8(output.stdout).unwrap();
    let parsed = lines
        .lines()
        .map(|line| {
            let (label, value) = line.split_once(": ").expect("readable label/value");
            (
                label.to_owned(),
                serde_json::from_str::<serde_json::Value>(value).unwrap(),
            )
        })
        .collect::<Vec<_>>();
    let labels = parsed
        .iter()
        .map(|(label, _)| label.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        labels,
        [
            "workbench_binding",
            "workbench_selection",
            "lexical_operation",
            "lexical_policy_ids",
            "lexical_totals",
            "lexical_source_totals",
            "lexical_exclusions",
            "lexical_next_cursor",
            "computed_hit",
        ]
    );
    assert_eq!(parsed[2].1, "spelling_distance");
    assert_eq!(parsed[4].1["unit"], "assertion_occurrences");
    assert_eq!(parsed[4].1["candidate_assertions"], 1);
    assert_eq!(parsed[4].1["records"], 1);
    assert_eq!(parsed[8].1["hit"]["assertion"]["id"], expected_assertion_id);

    let invalid_path = root.join("strict-invalid.json");
    fs::write(
        &invalid_path,
        br#"{"version":1,"query":{"operation":"spelling_distance","anchor":"farvi","max_distance":null},"options":{"sources":[],"alias_scope":"ActiveOnly","page_size":null,"cursor":null},"unexpected":true}"#,
    )
    .unwrap();
    let missing_view = root.join("does-not-exist");
    let strict = cli(&[
        "workbench",
        "lexical",
        missing_view.to_str().unwrap(),
        "--request",
        invalid_path.to_str().unwrap(),
    ]);
    let strict_error = assert_cli_error(&strict, "request_json");
    assert!(
        !strict_error["error"]["message"]
            .as_str()
            .unwrap()
            .contains("does-not-exist")
    );

    fs::write(
        &invalid_path,
        br#"{"version":1,"query":{"operation":"spelling_distance","anchor":"farvi"},"options":{"sources":[],"alias_scope":"ActiveOnly","page_size":null,"cursor":null}}"#,
    )
    .unwrap();
    let missing_required_nullable = cli(&[
        "workbench",
        "lexical",
        missing_view.to_str().unwrap(),
        "--request",
        invalid_path.to_str().unwrap(),
    ]);
    let missing_error = assert_cli_error(&missing_required_nullable, "request_json");
    assert!(
        !missing_error["error"]["message"]
            .as_str()
            .unwrap()
            .contains("does-not-exist")
    );

    let semantic_path = root.join("semantic-invalid.json");
    fs::write(
        &semantic_path,
        br#"{"version":1,"query":{"operation":"spelling_distance","anchor":"","max_distance":null},"options":{"sources":["llg"],"alias_scope":"ActiveOnly","page_size":null,"cursor":null}}"#,
    )
    .unwrap();
    let semantic = cli(&[
        "workbench",
        "lexical",
        view.to_str().unwrap(),
        "--request",
        semantic_path.to_str().unwrap(),
    ]);
    let semantic_error = assert_cli_error(&semantic, "invalid_lexical_request");
    assert_eq!(
        semantic_error["error"]["message"],
        "invalid query: spelling-distance anchor must not be empty"
    );

    #[cfg(unix)]
    {
        let deleted_cwd = root.join("empty-cwd");
        fs::create_dir(&deleted_cwd).unwrap();
        let absolute_arguments = vec![
            OsString::from("workbench"),
            OsString::from("lexical"),
            view.as_os_str().to_owned(),
            OsString::from("--request"),
            request_path.as_os_str().to_owned(),
        ];
        let absolute = run_after_removing_cwd(&deleted_cwd, &absolute_arguments);
        assert!(
            absolute.status.success(),
            "absolute request/view paths should work after cwd removal: {}",
            String::from_utf8_lossy(&absolute.stderr)
        );
        assert!(absolute.stderr.is_empty());
        let absolute_response: serde_json::Value =
            serde_json::from_slice(&absolute.stdout).unwrap();
        assert_eq!(
            absolute_response["result"]["operation"],
            "spelling_distance"
        );
        assert_eq!(
            absolute_response["result"]["response"]["totals"]["records"],
            1
        );
        assert_eq!(
            absolute_response["result"]["response"]["hits"][0]["hit"]["assertion"]["id"],
            expected_assertion_id
        );
        assert!(!deleted_cwd.exists());

        let relative_cwd = root.join("relative-empty-cwd");
        fs::create_dir(&relative_cwd).unwrap();
        let relative_arguments = vec![
            OsString::from("workbench"),
            OsString::from("lexical"),
            view.as_os_str().to_owned(),
            OsString::from("--request"),
            OsString::from("request.json"),
        ];
        let relative = run_after_removing_cwd(&relative_cwd, &relative_arguments);
        let relative_error = assert_cli_error(&relative, "workbench_io");
        assert!(
            !relative_error["error"]["message"]
                .as_str()
                .unwrap()
                .contains(view.to_str().unwrap())
        );
    }

    fs::remove_dir_all(root).unwrap();
}
