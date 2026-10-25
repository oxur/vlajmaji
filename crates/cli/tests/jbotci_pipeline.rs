//! Real Bash and CLI controls for the pinned local jbotci workflow.

use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
    thread,
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

static SCRATCH_SEQUENCE: AtomicU64 = AtomicU64::new(0);

fn create_scratch_with(mut candidate: impl FnMut(usize) -> PathBuf) -> PathBuf {
    const MAX_ATTEMPTS: usize = 256;

    for attempt in 0..MAX_ATTEMPTS {
        let path = candidate(attempt);
        match fs::create_dir(&path) {
            Ok(()) => return path,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => panic!(
                "could not exclusively create test scratch directory {}: {error}",
                path.display()
            ),
        }
    }

    panic!(
        "could not exclusively allocate a test scratch directory after {MAX_ATTEMPTS} candidates"
    );
}

fn scratch() -> PathBuf {
    let temp = std::env::temp_dir();
    let pid = std::process::id();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock is before the Unix epoch")
        .as_nanos();
    let sequence = SCRATCH_SEQUENCE.fetch_add(1, Ordering::Relaxed);

    create_scratch_with(|attempt| {
        temp.join(format!(
            "vlajmaji s02 pinned pipeline {pid} {nanos} {sequence} {attempt}"
        ))
    })
}

#[test]
fn scratch_retries_occupied_candidates_without_touching_them() {
    let root = scratch();
    let occupied_directory = root.join("occupied directory");
    let directory_sentinel = occupied_directory.join("caller sentinel");
    fs::create_dir(&occupied_directory).unwrap();
    fs::write(&directory_sentinel, b"preserve occupied directory\n").unwrap();

    let occupied_file = root.join("occupied file");
    fs::write(&occupied_file, b"preserve occupied file\n").unwrap();

    let link_target = root.join("occupied link target");
    let occupied_link = root.join("occupied link");
    fs::write(&link_target, b"preserve occupied link target\n").unwrap();
    std::os::unix::fs::symlink(&link_target, &occupied_link).unwrap();

    let fresh = root.join("fresh owned candidate");
    let candidates = [
        occupied_directory.clone(),
        occupied_file.clone(),
        occupied_link.clone(),
        fresh.clone(),
    ];
    let allocated = create_scratch_with(|attempt| candidates[attempt].clone());

    assert_eq!(allocated, fresh);
    assert!(allocated.is_dir());
    assert_eq!(
        fs::read(&directory_sentinel).unwrap(),
        b"preserve occupied directory\n"
    );
    assert_eq!(
        fs::read(&occupied_file).unwrap(),
        b"preserve occupied file\n"
    );
    assert!(occupied_link.is_symlink());
    assert_eq!(
        fs::read(&link_target).unwrap(),
        b"preserve occupied link target\n"
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn concurrent_scratch_allocations_are_distinct_and_joined() {
    const WORKERS: usize = 8;

    let results = thread::scope(|scope| {
        let handles: Vec<_> = (0..WORKERS).map(|_| scope.spawn(scratch)).collect();
        handles
            .into_iter()
            .map(|handle| handle.join())
            .collect::<Vec<_>>()
    });

    let mut paths = Vec::with_capacity(WORKERS);
    let mut panicked = false;
    for result in results {
        match result {
            Ok(path) => paths.push(path),
            Err(_) => panicked = true,
        }
    }

    if !panicked {
        let mut sorted = paths.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), WORKERS, "scratch paths must be unique");
        assert!(paths.iter().all(|path| path.is_dir()));
    }
    for path in paths {
        fs::remove_dir_all(path).unwrap();
    }
    assert!(!panicked, "a scratch allocation worker panicked");
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
    */dictionary-en.metadata.toml)
        cp "$FAKE_META_SOURCE" "$output"
        if [ -n "${FAKE_CURL_CREATE_REPORT:-}" ]; then
            (set -C; printf 'report race sentinel\n' > "$FAKE_CURL_CREATE_REPORT")
        fi
        ;;
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

    let occupied_link_release = root.join("occupied link release.json");
    let occupied_link_report = root.join("occupied link release.json.report.json");
    let occupied_link_target = root.join("occupied link target");
    fs::write(&occupied_link_target, b"link target sentinel\n").unwrap();
    std::os::unix::fs::symlink(&occupied_link_target, &occupied_link_report).unwrap();
    let refused_link = invoke_pipeline(
        &[
            "replay",
            pins_a.to_str().unwrap(),
            cache.to_str().unwrap(),
            occupied_link_release.to_str().unwrap(),
        ],
        None,
    );
    assert!(!refused_link.status.success());
    assert!(!occupied_link_release.exists());
    assert!(occupied_link_report.is_symlink());
    assert_eq!(
        fs::read(&occupied_link_target).unwrap(),
        b"link target sentinel\n"
    );

    let occupied_dir_release = root.join("occupied directory release.json");
    let occupied_dir_report = root.join("occupied directory release.json.report.json");
    fs::create_dir(&occupied_dir_report).unwrap();
    let refused_directory = invoke_pipeline(
        &[
            "replay",
            pins_a.to_str().unwrap(),
            cache.to_str().unwrap(),
            occupied_dir_release.to_str().unwrap(),
        ],
        None,
    );
    assert!(!refused_directory.status.success());
    assert!(!occupied_dir_release.exists());
    assert_eq!(fs::read_dir(&occupied_dir_report).unwrap().count(), 0);

    let dangling_release = root.join("dangling report release.json");
    let dangling_report = root.join("dangling report release.json.report.json");
    let dangling_target = root.join("dangling report target");
    std::os::unix::fs::symlink(&dangling_target, &dangling_report).unwrap();
    let refused_dangling = invoke_pipeline(
        &[
            "replay",
            pins_a.to_str().unwrap(),
            cache.to_str().unwrap(),
            dangling_release.to_str().unwrap(),
        ],
        None,
    );
    assert!(!refused_dangling.status.success());
    assert!(!dangling_release.exists());
    assert!(dangling_report.is_symlink());
    assert!(!dangling_target.exists());

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
    let mismatch_stderr = String::from_utf8_lossy(&updated_bad.stderr);
    for path in [
        &mismatch_leaf,
        &mismatch_release,
        &mismatch_report,
        &mismatch_store,
    ] {
        assert!(
            mismatch_stderr.contains(path.to_str().unwrap()),
            "{mismatch_stderr}"
        );
    }
    assert!(
        updated_bad.stdout.is_empty(),
        "a rejected update must not look successful on stdout"
    );
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

fn race_cli_wrapper(root: &Path) -> (PathBuf, PathBuf) {
    let wrapper = root.join("CLI race wrapper");
    let calls = root.join("CLI calls.log");
    fs::write(
        &wrapper,
        r#"#!/bin/bash
set -eu
printf '%s\n' "$*" >> "$CLI_CALL_LOG"
"$REAL_CLI" "$@"
if [ "$1" = jbotci-check ] && [ -n "${CLI_RACE_REPORT:-}" ]; then
    case " $* " in
        *' --release '*) ;;
        *) (set -C; printf 'report race sentinel\n' > "$CLI_RACE_REPORT") ;;
    esac
fi
"#,
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&wrapper, fs::Permissions::from_mode(0o755)).unwrap();
    }
    (wrapper, calls)
}

#[test]
fn report_creation_races_preserve_caller_bytes_and_skip_importer() {
    let root = scratch();
    let raw = root.join("raw fixture.json");
    let metadata = root.join("meta fixture.toml");
    fs::write(&raw, RAW).unwrap();
    fs::write(&metadata, GOOD_METADATA).unwrap();
    let pins_path = root.join("pins.env");
    pins(&pins_path, REVISION_A, GOOD_META_SHA);
    let curl = fake_curl(&root);
    let cache = root.join("cache");
    let mut acquire = Command::new("/bin/bash");
    acquire.arg(script()).args([
        "acquire",
        pins_path.to_str().unwrap(),
        cache.to_str().unwrap(),
    ]);
    acquire
        .env("VLAJMAJI_BIN", env!("CARGO_BIN_EXE_vlajmaji"))
        .env("CURL", &curl);
    set_fake_inputs(&mut acquire, &root, &metadata, &root.join("acquire URLs"));
    let acquired = acquire.output().unwrap();
    assert!(
        acquired.status.success(),
        "{}",
        String::from_utf8_lossy(&acquired.stderr)
    );
    let leaf = cache_leaf(&cache, REVISION_A, GOOD_META_SHA);
    let marker_before = fs::read(leaf.join("complete")).unwrap();

    let (wrapper, replay_calls) = race_cli_wrapper(&root);
    let replay_release = root.join("replay race release.json");
    let replay_report = root.join("replay race release.json.report.json");
    let replay_store = root.join("replay race store");
    let replay = Command::new("/bin/bash")
        .arg(script())
        .args([
            "replay",
            pins_path.to_str().unwrap(),
            cache.to_str().unwrap(),
            replay_release.to_str().unwrap(),
            replay_store.to_str().unwrap(),
        ])
        .env("VLAJMAJI_BIN", &wrapper)
        .env("REAL_CLI", env!("CARGO_BIN_EXE_vlajmaji"))
        .env("CLI_CALL_LOG", &replay_calls)
        .env("CLI_RACE_REPORT", &replay_report)
        .output()
        .unwrap();
    assert_eq!(replay.status.code(), Some(1));
    assert!(replay.stdout.is_empty());
    assert_eq!(fs::read(&replay_report).unwrap(), b"report race sentinel\n");
    assert!(!replay_release.exists());
    assert!(!replay_store.exists());
    let replay_call_text = fs::read_to_string(&replay_calls).unwrap();
    assert_eq!(replay_call_text.lines().count(), 1);
    assert!(replay_call_text.contains("jbotci-check"));
    let replay_stderr = String::from_utf8_lossy(&replay.stderr);
    for path in [&leaf, &replay_release, &replay_report, &replay_store] {
        assert!(
            replay_stderr.contains(path.to_str().unwrap()),
            "{replay_stderr}"
        );
    }
    assert_eq!(fs::read(leaf.join("complete")).unwrap(), marker_before);

    let update_cache = root.join("update cache");
    let update_release = root.join("update race release.json");
    let update_report = root.join("update race release.json.report.json");
    let update_store = root.join("update race store");
    let update_calls = root.join("update CLI calls.log");
    let mut update = Command::new("/bin/bash");
    update
        .arg(script())
        .args([
            "update",
            pins_path.to_str().unwrap(),
            update_cache.to_str().unwrap(),
            update_release.to_str().unwrap(),
            update_store.to_str().unwrap(),
        ])
        .env("VLAJMAJI_BIN", &wrapper)
        .env("REAL_CLI", env!("CARGO_BIN_EXE_vlajmaji"))
        .env("CLI_CALL_LOG", &update_calls)
        .env("CURL", &curl)
        .env("FAKE_CURL_CREATE_REPORT", &update_report);
    set_fake_inputs(&mut update, &root, &metadata, &root.join("update URLs"));
    let updated = update.output().unwrap();
    assert_eq!(updated.status.code(), Some(1));
    assert!(updated.stdout.is_empty());
    assert_eq!(fs::read(&update_report).unwrap(), b"report race sentinel\n");
    assert!(!update_release.exists());
    assert!(!update_store.exists());
    assert!(
        !update_calls.exists(),
        "importer wrapper ran after report reservation failed"
    );
    let update_leaf = cache_leaf(&update_cache, REVISION_A, GOOD_META_SHA);
    assert!(update_leaf.join("dictionary-en.json").is_file());
    assert!(update_leaf.join("dictionary-en.metadata.toml").is_file());
    assert!(!update_leaf.join("complete").exists());
    let update_stderr = String::from_utf8_lossy(&updated.stderr);
    for path in [&update_leaf, &update_release, &update_report, &update_store] {
        assert!(
            update_stderr.contains(path.to_str().unwrap()),
            "{update_stderr}"
        );
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn malformed_manifest_and_download_failure_leave_no_completion_marker() {
    let root = scratch();
    let raw = root.join("raw fixture.json");
    let meta = root.join("meta fixture.toml");
    fs::write(&raw, RAW).unwrap();
    fs::write(&meta, GOOD_METADATA).unwrap();
    let curl = fake_curl(&root);
    let urls = root.join("manifest URLs.log");
    let valid_text = format!(
        "version=1\nrevision={REVISION_A}\nraw_sha256={RAW_SHA}\nmetadata_sha256={GOOD_META_SHA}\n"
    );
    let invalid_bodies: Vec<Vec<u8>> = vec![
        format!("{valid_text}unknown=x\n").into_bytes(),
        format!("version=1\nrevision={REVISION_A}\nraw_sha256={RAW_SHA}\n").into_bytes(),
        format!("version=1\nrevision={REVISION_A}\nraw_sha256={}\nmetadata_sha256={GOOD_META_SHA}\n", RAW_SHA.to_ascii_uppercase()).into_bytes(),
        format!("version=1\r\nrevision={REVISION_A}\nraw_sha256={RAW_SHA}\nmetadata_sha256={GOOD_META_SHA}\n").into_bytes(),
        format!("{valid_text}trailing = junk\n").into_bytes(),
        format!("version=1\n revision={REVISION_A}\nraw_sha256={RAW_SHA}\nmetadata_sha256={GOOD_META_SHA}\n").into_bytes(),
        format!("{valid_text}\0\n").into_bytes(),
        format!("{valid_text}# snowman: ☃\n").into_bytes(),
    ];
    for (index, body) in invalid_bodies.iter().enumerate() {
        let path = root.join(format!("invalid variant {index}.env"));
        fs::write(&path, body).unwrap();
        let output_root = root.join(format!("invalid output {index}"));
        let mut command = Command::new("/bin/bash");
        command
            .arg(script())
            .args([
                "acquire",
                path.to_str().unwrap(),
                output_root.to_str().unwrap(),
            ])
            .env("VLAJMAJI_BIN", env!("CARGO_BIN_EXE_vlajmaji"))
            .env("CURL", &curl);
        set_fake_inputs(&mut command, &root, &meta, &urls);
        let rejected = command.output().unwrap();
        assert!(
            !rejected.status.success(),
            "invalid manifest variant {index} passed"
        );
        assert!(
            !output_root.exists(),
            "invalid manifest variant {index} mutated cache root"
        );
        assert!(
            !root.join("curl count").exists(),
            "invalid manifest variant {index} reached transport"
        );
    }

    let pins_path = root.join("valid no-final-LF.env");
    fs::write(&pins_path, valid_text.trim_end_matches('\n')).unwrap();
    let eof_cache = root.join("cache from final unterminated assignment");
    let mut eof_acquire = Command::new("/bin/bash");
    eof_acquire
        .arg(script())
        .args([
            "acquire",
            pins_path.to_str().unwrap(),
            eof_cache.to_str().unwrap(),
        ])
        .env("VLAJMAJI_BIN", env!("CARGO_BIN_EXE_vlajmaji"))
        .env("CURL", &curl);
    set_fake_inputs(&mut eof_acquire, &root, &meta, &urls);
    let eof_valid = eof_acquire.output().unwrap();
    assert!(
        eof_valid.status.success(),
        "{}",
        String::from_utf8_lossy(&eof_valid.stderr)
    );
    assert!(
        cache_leaf(&eof_cache, REVISION_A, GOOD_META_SHA)
            .join("complete")
            .is_file()
    );
    assert_eq!(fs::read_to_string(&urls).unwrap().lines().count(), 2);

    fs::remove_file(root.join("curl count")).unwrap();
    let first_failure_cache = root.join("first download failure cache");
    let first_failure_leaf = cache_leaf(&first_failure_cache, REVISION_A, GOOD_META_SHA);
    let first_urls = root.join("first failure URLs.log");
    let mut first_failure = Command::new("/bin/bash");
    first_failure
        .arg(script())
        .args([
            "acquire",
            pins_path.to_str().unwrap(),
            first_failure_cache.to_str().unwrap(),
        ])
        .env("VLAJMAJI_BIN", env!("CARGO_BIN_EXE_vlajmaji"))
        .env("CURL", &curl)
        .env("FAKE_CURL_FAIL_AT", "1");
    set_fake_inputs(&mut first_failure, &root, &meta, &first_urls);
    let first_failed = first_failure.output().unwrap();
    assert_eq!(first_failed.status.code(), Some(23));
    assert!(first_failed.stdout.is_empty());
    assert!(
        String::from_utf8_lossy(&first_failed.stderr)
            .contains(first_failure_leaf.to_str().unwrap())
    );
    assert!(first_failure_leaf.is_dir());
    assert!(!first_failure_leaf.join("dictionary-en.json").exists());
    assert!(
        !first_failure_leaf
            .join("dictionary-en.metadata.toml")
            .exists()
    );
    assert!(!first_failure_leaf.join("complete").exists());
    assert_eq!(fs::read_to_string(root.join("curl count")).unwrap(), "1\n");
    assert_eq!(fs::read_to_string(first_urls).unwrap().lines().count(), 1);

    fs::remove_file(root.join("curl count")).unwrap();
    let second_failure_cache = root.join("second download failure cache");
    let second_failure_leaf = cache_leaf(&second_failure_cache, REVISION_A, GOOD_META_SHA);
    let second_urls = root.join("second failure URLs.log");
    let mut second_failure = Command::new("/bin/bash");
    second_failure
        .arg(script())
        .args([
            "acquire",
            pins_path.to_str().unwrap(),
            second_failure_cache.to_str().unwrap(),
        ])
        .env("VLAJMAJI_BIN", env!("CARGO_BIN_EXE_vlajmaji"))
        .env("CURL", &curl)
        .env("FAKE_CURL_FAIL_AT", "2");
    set_fake_inputs(&mut second_failure, &root, &meta, &second_urls);
    let second_failed = second_failure.output().unwrap();
    assert_eq!(second_failed.status.code(), Some(23));
    assert!(second_failure_leaf.join("dictionary-en.json").is_file());
    assert!(
        !second_failure_leaf
            .join("dictionary-en.metadata.toml")
            .exists()
    );
    assert!(!second_failure_leaf.join("complete").exists());
    assert_eq!(fs::read_to_string(root.join("curl count")).unwrap(), "2\n");
    assert_eq!(fs::read_to_string(second_urls).unwrap().lines().count(), 2);
    let retry = invoke_pipeline(
        &[
            "acquire",
            pins_path.to_str().unwrap(),
            second_failure_cache.to_str().unwrap(),
        ],
        None,
    );
    assert!(
        !retry.status.success(),
        "an incomplete leaf must not be resumed or replaced"
    );

    fs::remove_file(root.join("curl count")).unwrap();
    let corrupt_cache = root.join("corrupt completed cache");
    let mut valid_acquire = Command::new("/bin/bash");
    valid_acquire
        .arg(script())
        .args([
            "acquire",
            pins_path.to_str().unwrap(),
            corrupt_cache.to_str().unwrap(),
        ])
        .env("VLAJMAJI_BIN", env!("CARGO_BIN_EXE_vlajmaji"))
        .env("CURL", &curl);
    set_fake_inputs(
        &mut valid_acquire,
        &root,
        &meta,
        &root.join("corrupt cache URLs.log"),
    );
    assert!(valid_acquire.output().unwrap().status.success());
    let corrupt_leaf = cache_leaf(&corrupt_cache, REVISION_A, GOOD_META_SHA);
    fs::write(
        corrupt_leaf.join("dictionary-en.json"),
        b"corrupted pinned bytes",
    )
    .unwrap();
    let release = root.join("corrupt replay release.json");
    let report = root.join("corrupt replay release.json.report.json");
    let store = root.join("corrupt replay store");
    let rejected = invoke_pipeline(
        &[
            "replay",
            pins_path.to_str().unwrap(),
            corrupt_cache.to_str().unwrap(),
            release.to_str().unwrap(),
            store.to_str().unwrap(),
        ],
        None,
    );
    assert!(!rejected.status.success());
    for path in [&corrupt_leaf, &release, &report, &store] {
        assert!(String::from_utf8_lossy(&rejected.stderr).contains(path.to_str().unwrap()));
    }
    assert_eq!(
        fs::read(corrupt_leaf.join("dictionary-en.json")).unwrap(),
        b"corrupted pinned bytes"
    );
    assert_eq!(
        fs::read(corrupt_leaf.join("complete")).unwrap(),
        b"vlajmaji-jbotci-cache-v1\n"
    );
    assert!(!release.exists());
    assert!(!report.exists());
    assert!(!store.exists());
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

    let cargo_probe = root.join("cargo must not run");
    let cargo_called = root.join("cargo was called");
    fs::write(
        &cargo_probe,
        format!(
            "#!/bin/bash\nprintf called > '{}'\nexit 90\n",
            cargo_called.display()
        ),
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&cargo_probe, fs::Permissions::from_mode(0o755)).unwrap();
    }
    let missing_target = Command::new("make")
        .current_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."))
        .arg("jbotci-check")
        .env("JBOTCI_PINS", &pins_path)
        .env("JBOTCI_CACHE", &cache)
        .env_remove("VLAJMAJI_BIN")
        .env_remove("CARGO_TARGET_DIR")
        .env("CARGO", &cargo_probe)
        .output()
        .unwrap();
    assert!(!missing_target.status.success());
    assert!(String::from_utf8_lossy(&missing_target.stderr).contains("explicit CARGO_TARGET_DIR"));
    assert!(
        !cargo_called.exists(),
        "missing-target preflight invoked Cargo"
    );

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
    let unrelated = root.join("unrelated caller file");
    fs::write(&unrelated, b"preserve caller data\n").unwrap();
    let pin_alias = root.join("pins symlink");
    std::os::unix::fs::symlink(&pins_path, &pin_alias).unwrap();
    let pin_alias_check = invoke_pipeline(
        &[
            "check",
            pin_alias.to_str().unwrap(),
            cache.to_str().unwrap(),
        ],
        None,
    );
    assert!(
        pin_alias_check.status.success(),
        "{}",
        String::from_utf8_lossy(&pin_alias_check.stderr)
    );
    let cache_alias = root.join("cache root symlink");
    std::os::unix::fs::symlink(&cache, &cache_alias).unwrap();
    let cache_alias_check = invoke_pipeline(
        &[
            "check",
            pins_path.to_str().unwrap(),
            cache_alias.to_str().unwrap(),
        ],
        None,
    );
    assert!(
        cache_alias_check.status.success(),
        "{}",
        String::from_utf8_lossy(&cache_alias_check.stderr)
    );
    let bin_alias = root.join("CLI symlink");
    std::os::unix::fs::symlink(env!("CARGO_BIN_EXE_vlajmaji"), &bin_alias).unwrap();
    fs::copy(&raw, root.join("raw fixture.json")).unwrap();
    let tmp_cache = Path::new("/tmp").join(format!(
        "vlajmaji-s02-tmp-alias-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    assert!(!tmp_cache.exists());
    let mut tmp_acquire = Command::new("/bin/bash");
    tmp_acquire
        .arg(script())
        .args([
            "acquire",
            pin_alias.to_str().unwrap(),
            tmp_cache.to_str().unwrap(),
        ])
        .env("VLAJMAJI_BIN", env!("CARGO_BIN_EXE_vlajmaji"))
        .env("CURL", &curl);
    set_fake_inputs(
        &mut tmp_acquire,
        &root,
        &metadata,
        &root.join("tmp alias URLs"),
    );
    let tmp_acquired = tmp_acquire.output().unwrap();
    assert!(
        tmp_acquired.status.success(),
        "{}",
        String::from_utf8_lossy(&tmp_acquired.stderr)
    );
    fs::remove_dir_all(&tmp_cache).unwrap();

    let bin_check = Command::new("/bin/bash")
        .arg(script())
        .args([
            "check",
            pins_path.to_str().unwrap(),
            cache.to_str().unwrap(),
        ])
        .env("VLAJMAJI_BIN", &bin_alias)
        .output()
        .unwrap();
    assert!(
        bin_check.status.success(),
        "{}",
        String::from_utf8_lossy(&bin_check.stderr)
    );

    let literal_cache = root.join(format!(
        "cache $(shell touch '{}') $cash {}ticks' quote; semi",
        sentinel.display(),
        char::from(96)
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
    assert_eq!(fs::read(&unrelated).unwrap(), b"preserve caller data\n");
    fs::remove_dir_all(root).unwrap();
}
