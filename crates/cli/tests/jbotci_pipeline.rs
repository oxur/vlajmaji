//! Real Bash and CLI controls for the pinned local jbotci workflow.

use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    time::{SystemTime, UNIX_EPOCH},
};

use serde_json::Value;

const RAW_SHA: &str = "a0e15949bdcfaeeba24007350d5dae0c8e39c991476ebdd0d4d06739ccac1805";
const GOOD_META_SHA: &str = "12d026b5f9afa397135d952c516868d619eb5ffaa47d06b129661cf74e5bf388";
const BAD_META_SHA: &str = "362cdcb137405de0048a6cbabd09afdc13d149b5e0325a475a6c72e48fd9bcbe";
const REVISION_A: &str = "0000000000000000000000000000000000000000";
const REVISION_B: &str = "1111111111111111111111111111111111111111";
const RAW: &[u8] = br#"[{"word":"farvi","word_type":"gismu","definition":"to vary","score":-1},{"word":"farvi","word_type":"gismu","definition":"to vary","score":-1}]"#;
const GOOD_METADATA: &str = "language_tag = \"en\"\nlanguage_realname = \"English\"\nsource_language_tag = \"jbo\"\nformat = \"json\"\npositive_scores_only = false\nfilename = \"dictionary-en.json\"\nmetadata_url = \"https://example.invalid/languages\"\ndownload_url = \"https://example.invalid/export\"\nlensisku_created_at = \"2026-10-04T03:07:08Z\"\nsha256 = \"a0e15949bdcfaeeba24007350d5dae0c8e39c991476ebdd0d4d06739ccac1805\"\ndefinition_count = 2\nentry_count = 1\n";
const BAD_METADATA: &str = "language_tag = \"en\"\nlanguage_realname = \"English\"\nsource_language_tag = \"jbo\"\nformat = \"json\"\npositive_scores_only = false\nfilename = \"dictionary-en.json\"\nmetadata_url = \"https://example.invalid/languages\"\ndownload_url = \"https://example.invalid/export\"\nlensisku_created_at = \"2026-10-04T03:07:08Z\"\nsha256 = \"a0e15949bdcfaeeba24007350d5dae0c8e39c991476ebdd0d4d06739ccac1805\"\ndefinition_count = 1\nentry_count = 1\n";

fn scratch() -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "vlajmaji s02 pinned pipeline {} {}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&path).unwrap();
    path
}

fn script() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/jbotci-import.sh")
}

fn pins(path: &Path, revision: &str, metadata_sha: &str) {
    fs::write(
        path,
        format!(
            "  # exact local pin\n\nversion=1\nrevision={revision}\nraw_sha256={RAW_SHA}\nmetadata_sha256={metadata_sha}\n"
        ),
    )
    .unwrap();
}

fn invoke_pipeline(args: &[&str], fake_curl: Option<&Path>) -> Output {
    let mut command = Command::new("/bin/bash");
    command
        .arg(script())
        .args(args)
        .env("VLAJMAJI_BIN", env!("CARGO_BIN_EXE_vlajmaji"));
    command.env_remove("CARGO_TARGET_DIR");
    if let Some(curl) = fake_curl {
        command.env("CURL", curl);
    }
    command.output().unwrap()
}

fn json_output(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
        panic!(
            "stdout was not JSON ({error}); status={}; stderr={}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        )
    })
}

fn fake_curl(root: &Path) -> PathBuf {
    let script = root.join("curl transport with spaces");
    fs::write(
        &script,
        r##"#!/bin/bash
set -eu
[ "$1" = --fail ] && [ "$2" = --location ] && [ "$3" = --silent ] && [ "$4" = --show-error ] && [ "$5" = --output ]
output=$6
url=$7
count=0
if [ -f "$FAKE_CURL_COUNT" ]; then count=$(cat "$FAKE_CURL_COUNT"); fi
count=$((count + 1))
printf '%s\n' "$count" > "$FAKE_CURL_COUNT"
printf '%s\n' "$url" >> "$FAKE_URL_LOG"
if [ "${FAKE_CURL_FAIL_AT:-}" = "$count" ]; then exit 23; fi
case "$url" in
    */dictionary-en.json) cp "$FAKE_RAW_SOURCE" "$output" ;;
    */dictionary-en.metadata.toml) cp "$FAKE_META_SOURCE" "$output" ;;
    *) exit 24 ;;
esac
"##,
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&script, fs::Permissions::from_mode(0o755)).unwrap();
    }
    script
}

fn set_fake_inputs(command: &mut Command, root: &Path, meta: &Path, urls: &Path) {
    command
        .env("FAKE_RAW_SOURCE", root.join("raw fixture.json"))
        .env("FAKE_META_SOURCE", meta)
        .env("FAKE_CURL_COUNT", root.join("curl count"))
        .env("FAKE_URL_LOG", urls);
}

fn copy_tree(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let target = destination.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).unwrap();
        }
    }
}

fn cache_leaf(root: &Path, revision: &str, metadata_sha: &str) -> PathBuf {
    root.join(format!("{revision}-{RAW_SHA}-{metadata_sha}"))
}

#[test]
fn pinned_operations_preserve_literal_paths_attempts_and_duplicate_rows() {
    let root = scratch();
    let raw = root.join("raw fixture.json");
    let metadata = root.join("metadata fixture.toml");
    let bad_metadata = root.join("bad metadata fixture.toml");
    fs::write(&raw, RAW).unwrap();
    fs::write(&metadata, GOOD_METADATA).unwrap();
    fs::write(&bad_metadata, BAD_METADATA).unwrap();

    let pins_a = root.join("pins A.env");
    pins(&pins_a, REVISION_A, GOOD_META_SHA);
    let pins_b = root.join("pins B.env");
    pins(&pins_b, REVISION_B, GOOD_META_SHA);
    let pins_bad = root.join("pins bad.env");
    pins(&pins_bad, REVISION_B, BAD_META_SHA);

    let cache = root.join("cache $dollar `tick'quote; semi");
    let curl = fake_curl(&root);
    let urls = root.join("requested URLs.log");
    let leaf_a = cache_leaf(&cache, REVISION_A, GOOD_META_SHA);
    let mut acquire = Command::new("/bin/bash");
    acquire
        .arg(script())
        .args(["acquire", pins_a.to_str().unwrap(), cache.to_str().unwrap()]);
    acquire
        .env("VLAJMAJI_BIN", env!("CARGO_BIN_EXE_vlajmaji"))
        .env_remove("CARGO_TARGET_DIR")
        .env("CURL", &curl);
    set_fake_inputs(&mut acquire, &root, &metadata, &urls);
    let acquired = acquire.output().unwrap();
    assert!(
        acquired.status.success(),
        "{}",
        String::from_utf8_lossy(&acquired.stderr)
    );
    let acquired_json = json_output(&acquired);
    assert_eq!(acquired_json["operation"], "jbotci-check");
    assert_eq!(acquired_json["report"]["input_rows"], 2);
    assert!(leaf_a.join("complete").is_file());
    assert_eq!(fs::read(leaf_a.join("dictionary-en.json")).unwrap(), RAW);
    assert_eq!(
        fs::read(leaf_a.join("dictionary-en.metadata.toml")).unwrap(),
        GOOD_METADATA.as_bytes()
    );
    assert_eq!(fs::read_to_string(&urls).unwrap().lines().count(), 2);

    let before_raw = fs::read(leaf_a.join("dictionary-en.json")).unwrap();
    let before_meta = fs::read(leaf_a.join("dictionary-en.metadata.toml")).unwrap();
    let before_marker = fs::read(leaf_a.join("complete")).unwrap();
    let checked = invoke_pipeline(
        &["check", pins_a.to_str().unwrap(), cache.to_str().unwrap()],
        None,
    );
    assert!(
        checked.status.success(),
        "{}",
        String::from_utf8_lossy(&checked.stderr)
    );
    assert_eq!(json_output(&checked)["report"]["input_rows"], 2);
    assert_eq!(
        fs::read(leaf_a.join("dictionary-en.json")).unwrap(),
        before_raw
    );
    assert_eq!(
        fs::read(leaf_a.join("dictionary-en.metadata.toml")).unwrap(),
        before_meta
    );
    assert_eq!(fs::read(leaf_a.join("complete")).unwrap(), before_marker);
    let cached_acquire = invoke_pipeline(
        &["acquire", pins_a.to_str().unwrap(), cache.to_str().unwrap()],
        None,
    );
    assert!(cached_acquire.status.success());
    assert_eq!(fs::read_to_string(&urls).unwrap().lines().count(), 2);

    let release = root.join("release output.json");
    let store = root.join("new store");
    let replay = invoke_pipeline(
        &[
            "replay",
            pins_a.to_str().unwrap(),
            cache.to_str().unwrap(),
            release.to_str().unwrap(),
            store.to_str().unwrap(),
        ],
        None,
    );
    assert!(
        replay.status.success(),
        "{}",
        String::from_utf8_lossy(&replay.stderr)
    );
    let report_bytes = fs::read(root.join("release output.json.report.json")).unwrap();
    assert_eq!(report_bytes.last(), Some(&b'\n'));
    let report: Value = serde_json::from_slice(&report_bytes).unwrap();
    assert_eq!(report["report"]["input_rows"], 2);
    assert_eq!(json_output(&replay)["report"]["input_rows"], 2);
    let request = r#"{"operation":"exact","spelling":"farvi"}"#;
    let queried = Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
        .args([
            "dictionary-query",
            store.to_str().unwrap(),
            "--request",
            request,
        ])
        .output()
        .unwrap();
    assert!(
        queried.status.success(),
        "{}",
        String::from_utf8_lossy(&queried.stderr)
    );
    let response: Value = serde_json::from_slice(&queried.stdout).unwrap();
    assert_eq!(response["totals"]["occurrences"], 2);
    assert_eq!(response["totals"]["distinct_spellings"], 1);

    let occupied_release = root.join("occupied release.json");
    let occupied_report = root.join("occupied release.json.report.json");
    fs::write(&occupied_report, b"report sentinel\n").unwrap();
    let refused_sidecar = invoke_pipeline(
        &[
            "replay",
            pins_a.to_str().unwrap(),
            cache.to_str().unwrap(),
            occupied_release.to_str().unwrap(),
        ],
        None,
    );
    assert!(!refused_sidecar.status.success());
    assert!(!occupied_release.exists());
    assert_eq!(fs::read(&occupied_report).unwrap(), b"report sentinel\n");

    let occupied_store_release = root.join("occupied store release.json");
    let occupied_store = root.join("occupied store");
    fs::create_dir(&occupied_store).unwrap();
    let refused_store = invoke_pipeline(
        &[
            "replay",
            pins_a.to_str().unwrap(),
            cache.to_str().unwrap(),
            occupied_store_release.to_str().unwrap(),
            occupied_store.to_str().unwrap(),
        ],
        None,
    );
    assert!(!refused_store.status.success());
    assert!(!occupied_store_release.exists());
    assert_eq!(fs::read_dir(&occupied_store).unwrap().count(), 0);

    let release_before_reject = fs::read(&release).unwrap();
    let replay_again = invoke_pipeline(
        &[
            "replay",
            pins_a.to_str().unwrap(),
            cache.to_str().unwrap(),
            release.to_str().unwrap(),
        ],
        None,
    );
    assert!(!replay_again.status.success());
    assert_eq!(fs::read(&release).unwrap(), release_before_reject);

    let old_tuple = (
        fs::read(leaf_a.join("dictionary-en.json")).unwrap(),
        fs::read(leaf_a.join("dictionary-en.metadata.toml")).unwrap(),
        fs::read(leaf_a.join("complete")).unwrap(),
    );
    let mismatch_cache = root.join("mismatch cache");
    let mismatch_leaf = cache_leaf(&mismatch_cache, REVISION_B, BAD_META_SHA);
    let mismatch_release = root.join("mismatch candidate.json");
    let mismatch_store = root.join("must not build store");
    let bad_curl = fake_curl(&root);
    let mut update_bad = Command::new("/bin/bash");
    update_bad.arg(script()).args([
        "update",
        pins_bad.to_str().unwrap(),
        mismatch_cache.to_str().unwrap(),
        mismatch_release.to_str().unwrap(),
        mismatch_store.to_str().unwrap(),
    ]);
    update_bad
        .env("VLAJMAJI_BIN", env!("CARGO_BIN_EXE_vlajmaji"))
        .env_remove("CARGO_TARGET_DIR")
        .env("CURL", &bad_curl);
    set_fake_inputs(
        &mut update_bad,
        &root,
        &bad_metadata,
        &root.join("bad URLs.log"),
    );
    let updated_bad = update_bad.output().unwrap();
    assert!(!updated_bad.status.success());
    assert!(mismatch_release.is_file());
    let mismatch_report = root.join("mismatch candidate.json.report.json");
    let report: Value = serde_json::from_slice(&fs::read(&mismatch_report).unwrap()).unwrap();
    assert_eq!(report["report"]["definition_count_matches"], false);
    assert!(!mismatch_leaf.join("complete").exists());
    assert!(!mismatch_store.exists());
    assert_eq!(
        old_tuple,
        (
            fs::read(leaf_a.join("dictionary-en.json")).unwrap(),
            fs::read(leaf_a.join("dictionary-en.metadata.toml")).unwrap(),
            fs::read(leaf_a.join("complete")).unwrap(),
        )
    );

    let good_cache_b = root.join("valid update cache");
    let good_leaf_b = cache_leaf(&good_cache_b, REVISION_B, GOOD_META_SHA);
    let release_b = root.join("revision B release.json");
    let mut update_good = Command::new("/bin/bash");
    update_good.arg(script()).args([
        "update",
        pins_b.to_str().unwrap(),
        good_cache_b.to_str().unwrap(),
        release_b.to_str().unwrap(),
    ]);
    update_good
        .env("VLAJMAJI_BIN", env!("CARGO_BIN_EXE_vlajmaji"))
        .env_remove("CARGO_TARGET_DIR")
        .env("CURL", &curl);
    set_fake_inputs(
        &mut update_good,
        &root,
        &metadata,
        &root.join("good update URLs.log"),
    );
    let updated_good = update_good.output().unwrap();
    assert!(
        updated_good.status.success(),
        "{}",
        String::from_utf8_lossy(&updated_good.stderr)
    );
    assert!(good_leaf_b.join("complete").is_file());
    let release_a: Value = serde_json::from_slice(&fs::read(release).unwrap()).unwrap();
    let release_b_json: Value = serde_json::from_slice(&fs::read(&release_b).unwrap()).unwrap();
    assert_ne!(
        release_a["metadata"]["releases"][0]["id"],
        release_b_json["metadata"]["releases"][0]["id"]
    );
    assert_eq!(
        release_a["captures"][0]["identity"],
        release_b_json["captures"][0]["identity"]
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn malformed_manifest_and_download_failure_leave_no_completion_marker() {
    let root = scratch();
    let malformed = root.join("duplicate key.env");
    fs::write(
        &malformed,
        format!("version=1\nrevision={REVISION_A}\nraw_sha256={RAW_SHA}\nraw_sha256={RAW_SHA}\nmetadata_sha256={GOOD_META_SHA}\n"),
    )
    .unwrap();
    let never_created = root.join("must remain absent");
    let invalid = invoke_pipeline(
        &[
            "acquire",
            malformed.to_str().unwrap(),
            never_created.to_str().unwrap(),
        ],
        None,
    );
    assert!(!invalid.status.success());
    assert!(!never_created.exists());

    let invalid_bodies: Vec<Vec<u8>> = vec![
        format!("version=1\nrevision={REVISION_A}\nraw_sha256={RAW_SHA}\nmetadata_sha256={GOOD_META_SHA}\nunknown=x\n").into_bytes(),
        format!("version=1\nrevision={REVISION_A}\nraw_sha256={RAW_SHA}\n").into_bytes(),
        format!("version=1\nrevision={REVISION_A}\nraw_sha256={}\nmetadata_sha256={GOOD_META_SHA}\n", RAW_SHA.to_ascii_uppercase()).into_bytes(),
        format!("version=1\r\nrevision={REVISION_A}\nraw_sha256={RAW_SHA}\nmetadata_sha256={GOOD_META_SHA}\n").into_bytes(),
        format!("version=1\nrevision={REVISION_A}\nraw_sha256={RAW_SHA}\nmetadata_sha256={GOOD_META_SHA}\ntrailing = junk\n").into_bytes(),
        format!("version=1\nrevision={REVISION_A}\nraw_sha256={RAW_SHA}\nmetadata_sha256={GOOD_META_SHA}").into_bytes(),
        format!("version=1\nrevision={REVISION_A}\nraw_sha256={RAW_SHA}\nmetadata_sha256={GOOD_META_SHA}\n# snowman: ☃\n").into_bytes(),
    ];
    for (index, body) in invalid_bodies.iter().enumerate() {
        let path = root.join(format!("invalid variant {index}.env"));
        fs::write(&path, body).unwrap();
        let output_root = root.join(format!("invalid output {index}"));
        let rejected = invoke_pipeline(
            &[
                "acquire",
                path.to_str().unwrap(),
                output_root.to_str().unwrap(),
            ],
            None,
        );
        assert!(
            !rejected.status.success(),
            "invalid manifest variant {index} passed"
        );
        assert!(
            !output_root.exists(),
            "invalid manifest variant {index} mutated cache root"
        );
    }

    let pins_path = root.join("valid.env");
    pins(&pins_path, REVISION_A, GOOD_META_SHA);
    let raw = root.join("raw fixture.json");
    let meta = root.join("meta fixture.toml");
    fs::write(raw, RAW).unwrap();
    fs::write(meta, GOOD_METADATA).unwrap();
    let curl = fake_curl(&root);
    let cache = root.join("partial cache");
    let leaf = cache_leaf(&cache, REVISION_A, GOOD_META_SHA);
    let urls = root.join("fail URLs.log");
    let mut command = Command::new("/bin/bash");
    command.arg(script()).args([
        "acquire",
        pins_path.to_str().unwrap(),
        cache.to_str().unwrap(),
    ]);
    command
        .env("VLAJMAJI_BIN", env!("CARGO_BIN_EXE_vlajmaji"))
        .env_remove("CARGO_TARGET_DIR")
        .env("CURL", curl)
        .env("FAKE_RAW_SOURCE", root.join("raw fixture.json"))
        .env("FAKE_META_SOURCE", root.join("meta fixture.toml"))
        .env("FAKE_CURL_COUNT", root.join("failure count"))
        .env("FAKE_URL_LOG", urls)
        .env("FAKE_CURL_FAIL_AT", "2");
    let failed = command.output().unwrap();
    assert!(!failed.status.success());
    assert!(leaf.is_dir());
    assert!(leaf.join("dictionary-en.json").is_file());
    assert!(!leaf.join("complete").exists());
    let retry = invoke_pipeline(
        &[
            "acquire",
            pins_path.to_str().unwrap(),
            cache.to_str().unwrap(),
        ],
        None,
    );
    assert!(
        !retry.status.success(),
        "an incomplete leaf must not be resumed or replaced"
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn make_forwards_metacharacter_paths_and_requires_explicit_cargo_target() {
    let root = scratch();
    let raw = root.join("raw.json");
    let metadata = root.join("meta.toml");
    fs::write(&raw, RAW).unwrap();
    fs::write(&metadata, GOOD_METADATA).unwrap();
    let pins_path = root.join("pins.env");
    pins(&pins_path, REVISION_A, GOOD_META_SHA);
    let cache = root.join("cache $cash `ticks' quote; semi");
    let curl = fake_curl(&root);
    let urls = root.join("urls.log");
    let mut acquire = Command::new("/bin/bash");
    acquire.arg(script()).args([
        "acquire",
        pins_path.to_str().unwrap(),
        cache.to_str().unwrap(),
    ]);
    acquire
        .env("VLAJMAJI_BIN", env!("CARGO_BIN_EXE_vlajmaji"))
        .env("CURL", &curl)
        .env("FAKE_RAW_SOURCE", &raw)
        .env("FAKE_META_SOURCE", &metadata)
        .env("FAKE_CURL_COUNT", root.join("count"))
        .env("FAKE_URL_LOG", urls);
    let acquired = acquire.output().unwrap();
    assert!(
        acquired.status.success(),
        "{}",
        String::from_utf8_lossy(&acquired.stderr)
    );

    let missing_target = Command::new("make")
        .current_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."))
        .arg("jbotci-check")
        .env("JBOTCI_PINS", &pins_path)
        .env("JBOTCI_CACHE", &cache)
        .env_remove("VLAJMAJI_BIN")
        .env_remove("CARGO_TARGET_DIR")
        .output()
        .unwrap();
    assert!(!missing_target.status.success());
    assert!(String::from_utf8_lossy(&missing_target.stderr).contains("explicit CARGO_TARGET_DIR"));

    let make_check = Command::new("make")
        .current_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."))
        .arg("jbotci-check")
        .env("JBOTCI_PINS", &pins_path)
        .env("JBOTCI_CACHE", &cache)
        .env("VLAJMAJI_BIN", env!("CARGO_BIN_EXE_vlajmaji"))
        .env_remove("CARGO_TARGET_DIR")
        .output()
        .unwrap();
    assert!(
        make_check.status.success(),
        "{}",
        String::from_utf8_lossy(&make_check.stderr)
    );
    assert_eq!(json_output(&make_check)["report"]["input_rows"], 2);

    let sentinel = root.join("make-evaluated-command-line-path");
    let literal_cache = root.join(format!(
        "cache $(shell touch {}) $cash `ticks' quote; semi",
        sentinel.display()
    ));
    copy_tree(&cache, &literal_cache);
    let make_command_line = Command::new("make")
        .current_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."))
        .arg("jbotci-check")
        .arg(format!("JBOTCI_PINS={}", pins_path.display()))
        .arg(format!("JBOTCI_CACHE={}", literal_cache.display()))
        .arg(format!("VLAJMAJI_BIN={}", env!("CARGO_BIN_EXE_vlajmaji")))
        .env_remove("CARGO_TARGET_DIR")
        .env_remove("MAKEFLAGS")
        .output()
        .unwrap();
    assert!(
        make_command_line.status.success(),
        "{}",
        String::from_utf8_lossy(&make_command_line.stderr)
    );
    assert_eq!(json_output(&make_command_line)["report"]["input_rows"], 2);
    assert!(
        !sentinel.exists(),
        "GNU Make evaluated command-line path data"
    );
    for artifact in [
        "79075", "execute", "must", "not", "path", "pinned", "pipeline", "s02", "this",
    ] {
        assert!(
            !Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../..")
                .join(artifact)
                .exists(),
            "unexpected shell-splitting artifact from Make variable: {artifact}"
        );
    }
    fs::remove_dir_all(root).unwrap();
}
