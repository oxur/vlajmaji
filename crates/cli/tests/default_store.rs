//! Default-store behavior through the public command-line process boundary.

#[path = "../../lexicon/examples/support/dictionary_fixture.rs"]
mod fixture;

use std::{
    ffi::{OsStr, OsString},
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    sync::atomic::{AtomicUsize, Ordering},
};

use serde_json::Value;
use vlajmaji_lexicon::{Lexicon, SourceFormat, SourceMetadata, TimePrecision};

static NEXT: AtomicUsize = AtomicUsize::new(0);

fn scratch() -> PathBuf {
    for _ in 0..100 {
        let path = std::env::temp_dir().join(format!(
            "vlajmaji-cli-default-store-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        match std::fs::create_dir(&path) {
            Ok(()) => return path,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => panic!("create exclusive scratch directory: {error}"),
        }
    }
    panic!("could not allocate unique scratch directory after 100 attempts")
}

fn run(args: &[&OsStr], env: &[(&str, Option<&OsStr>)], stdin: Option<&[u8]>) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_vlajmaji"));
    command.args(args).env_clear();
    for (key, value) in env {
        if let Some(value) = value {
            command.env(key, value);
        } else {
            command.env_remove(key);
        }
    }
    if stdin.is_some() {
        command.stdin(Stdio::piped());
    }
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = command.spawn().expect("spawn CLI");
    if let Some(stdin) = stdin {
        child
            .stdin
            .take()
            .expect("piped stdin")
            .write_all(stdin)
            .expect("write child stdin");
    }
    child.wait_with_output().expect("wait for CLI")
}

fn args<'a>(values: &'a [&'a str]) -> Vec<&'a OsStr> {
    values.iter().map(OsStr::new).collect()
}

fn env<'a>(
    home: Option<&'a OsStr>,
    xdg: Option<&'a OsStr>,
) -> [(&'static str, Option<&'a OsStr>); 2] {
    [("HOME", home), ("XDG_DATA_HOME", xdg)]
}

fn write_fixture(path: &Path) {
    let bytes = fixture::complete_fixture()
        .expect("complete dictionary fixture")
        .to_json_bytes()
        .expect("serialize dictionary fixture");
    std::fs::write(path, bytes).expect("write fixture")
}

fn error_json(output: &Output, code: &str) -> Value {
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let value: Value = serde_json::from_slice(&output.stderr).expect("structured stderr");
    assert_eq!(value["error"]["code"], code);
    value
}

#[test]
fn test_xdg_default_build_query_stdin_and_info_match_explicit_store() {
    let root = scratch();
    let xdg = root.join("data root");
    let home = root.join("other-home");
    std::fs::create_dir(&home).unwrap();
    let input = root.join("dictionary.json");
    let expected_dictionary = fixture::complete_fixture().unwrap();
    write_fixture(&input);
    let expected_store = xdg.join("vlajmaji/stores/default");

    let build = run(
        &args(&["dictionary-build", input.to_str().unwrap()]),
        &env(Some(home.as_os_str()), Some(xdg.as_os_str())),
        None,
    );
    assert!(
        build.status.success(),
        "{}",
        String::from_utf8_lossy(&build.stderr)
    );
    let build_json: Value = serde_json::from_slice(&build.stdout).unwrap();
    assert_eq!(build_json["store"], expected_store.to_str().unwrap());
    assert!(expected_store.join("dictionary.sqlite3").is_file());
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(xdg.join("vlajmaji"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o700
        );
        assert_eq!(
            std::fs::metadata(xdg.join("vlajmaji/stores"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o700
        );
        let existing_xdg = root.join("existing-xdg");
        std::fs::create_dir(&existing_xdg).unwrap();
        std::fs::set_permissions(&existing_xdg, std::fs::Permissions::from_mode(0o751)).unwrap();
        let second_store = existing_xdg.join("vlajmaji/stores/default");
        let second_build = run(
            &args(&["dictionary-build", input.to_str().unwrap()]),
            &env(None, Some(existing_xdg.as_os_str())),
            None,
        );
        assert!(second_build.status.success());
        assert_eq!(
            std::fs::metadata(&existing_xdg)
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o751
        );
        assert_eq!(
            std::fs::metadata(second_store.parent().unwrap())
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o700
        );
    }

    let request = br#"{"operation":"exact","spelling":"farvi"}"#;
    let default_query = run(
        &args(&["dictionary-query"]),
        &env(Some(home.as_os_str()), Some(xdg.as_os_str())),
        Some(request),
    );
    let explicit_query = run(
        &args(&["dictionary-query", expected_store.to_str().unwrap()]),
        &env(None, None),
        Some(request),
    );
    assert!(default_query.status.success());
    assert!(explicit_query.status.success());
    let default_value: Value = serde_json::from_slice(&default_query.stdout).unwrap();
    let explicit_value: Value = serde_json::from_slice(&explicit_query.stdout).unwrap();
    assert_eq!(default_value, explicit_value);
    let default_argument_query = run(
        &args(&[
            "dictionary-query",
            "--request",
            std::str::from_utf8(request).unwrap(),
        ]),
        &env(Some(home.as_os_str()), Some(xdg.as_os_str())),
        None,
    );
    assert!(default_argument_query.status.success());
    assert_eq!(default_argument_query.stdout, default_query.stdout);

    let info_default = run(
        &args(&["info"]),
        &env(Some(home.as_os_str()), Some(xdg.as_os_str())),
        None,
    );
    let info_explicit = run(
        &args(&["info", expected_store.to_str().unwrap()]),
        &env(None, None),
        None,
    );
    assert!(
        info_default.status.success(),
        "{}",
        String::from_utf8_lossy(&info_default.stderr)
    );
    assert_eq!(info_default.stdout, info_explicit.stdout);
    let info_value: Value = serde_json::from_slice(&info_default.stdout).unwrap();
    assert_eq!(info_value["format"], "dictionary");
    let expected_occurrences = expected_dictionary
        .captures()
        .iter()
        .map(|capture| capture.occurrences.len())
        .sum::<usize>();
    let expected_sources = expected_dictionary.registry().document().sources;
    assert_eq!(info_value["captures"], expected_dictionary.captures().len());
    assert_eq!(info_value["occurrences"], expected_occurrences);
    assert_eq!(
        info_value["entries"],
        expected_dictionary.metadata().entries.len()
    );
    assert_eq!(
        info_value["assertions"],
        expected_dictionary.metadata().assertions.len()
    );
    assert_eq!(info_value["source_count"], expected_sources.len());
    assert_eq!(
        info_value["sources"],
        serde_json::to_value(expected_sources).unwrap()
    );
    assert!(info_value.get("store").is_none());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn test_home_fallback_explicit_override_and_missing_default_are_truthful() {
    let root = scratch();
    let home = root.join("home");
    std::fs::create_dir(&home).unwrap();
    let expected = home.join(".local/share/vlajmaji/stores/default");
    for xdg in [
        None,
        Some(OsStr::new("")),
        Some(OsStr::new("relative/path")),
    ] {
        let output = run(&args(&["info"]), &env(Some(home.as_os_str()), xdg), None);
        let error = error_json(&output, "store");
        assert!(
            error["error"]["message"]
                .as_str()
                .unwrap()
                .contains(expected.to_str().unwrap())
        );
    }
    assert!(!expected.exists());

    let input = root.join("input.json");
    write_fixture(&input);
    let explicit = root.join("explicit");
    let build = run(
        &args(&[
            "dictionary-build",
            input.to_str().unwrap(),
            explicit.to_str().unwrap(),
        ]),
        &env(None, Some(OsStr::new("relative"))),
        None,
    );
    assert!(
        build.status.success(),
        "{}",
        String::from_utf8_lossy(&build.stderr)
    );
    assert!(explicit.join("dictionary.sqlite3").is_file());
    assert!(!root.join("relative/vlajmaji").exists());

    let no_base = run(&args(&["info"]), &env(None, None), None);
    error_json(&no_base, "store_path");
    let query = run(
        &args(&["dictionary-query"]),
        &env(Some(home.as_os_str()), Some(root.as_os_str())),
        Some(br#"{"operation":"exact","spelling":"farvi"}"#),
    );
    assert!(!query.status.success());
    assert!(query.stdout.is_empty());
    assert!(!expected.exists());

    let blocked_xdg = root.join("xdg-is-file");
    std::fs::write(&blocked_xdg, b"not a directory").unwrap();
    let blocked = run(
        &args(&["info"]),
        &env(None, Some(blocked_xdg.as_os_str())),
        None,
    );
    error_json(&blocked, "io");
    let blocked_build = run(
        &args(&["dictionary-build", input.to_str().unwrap()]),
        &env(None, Some(blocked_xdg.as_os_str())),
        None,
    );
    error_json(&blocked_build, "io");
    assert_eq!(std::fs::read(&blocked_xdg).unwrap(), b"not a directory");
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn test_default_build_input_errors_and_occupied_leaves_have_bounded_side_effects() {
    let root = scratch();
    let xdg = root.join("xdg");
    let missing_input = root.join("missing.json");
    let missing = run(
        &args(&["dictionary-build", missing_input.to_str().unwrap()]),
        &env(None, Some(xdg.as_os_str())),
        None,
    );
    error_json(&missing, "io");
    assert!(!xdg.exists());

    let bad_input = root.join("bad.json");
    std::fs::write(&bad_input, b"{").unwrap();
    let bad = run(
        &args(&["dictionary-build", bad_input.to_str().unwrap()]),
        &env(None, Some(xdg.as_os_str())),
        None,
    );
    error_json(&bad, "invalid_dictionary");
    let store = xdg.join("vlajmaji/stores/default");
    assert!(store.parent().unwrap().is_dir());
    assert!(!store.exists());

    let input = root.join("good.json");
    write_fixture(&input);
    let explicit_missing_parent = root.join("explicit-parent-missing/store");
    let explicit_error = run(
        &args(&[
            "dictionary-build",
            input.to_str().unwrap(),
            explicit_missing_parent.to_str().unwrap(),
        ]),
        &env(None, None),
        None,
    );
    assert!(!explicit_error.status.success());
    assert!(explicit_error.stdout.is_empty());
    assert!(!explicit_missing_parent.parent().unwrap().exists());

    let marker = store.join("dictionary.sqlite3");
    for leaf in ["file", "directory", "symlink"] {
        if let Ok(metadata) = std::fs::symlink_metadata(&store) {
            if metadata.file_type().is_dir() {
                std::fs::remove_dir_all(&store).unwrap();
            } else {
                std::fs::remove_file(&store).unwrap();
            }
        }
        match leaf {
            "file" => std::fs::write(&store, b"preserve me").unwrap(),
            "directory" => std::fs::create_dir(&store).unwrap(),
            "symlink" => {
                #[cfg(unix)]
                std::os::unix::fs::symlink(root.join("missing-target"), &store).unwrap();
                #[cfg(not(unix))]
                continue;
            }
            _ => unreachable!(),
        }
        let occupied = run(
            &args(&["dictionary-build", input.to_str().unwrap()]),
            &env(None, Some(xdg.as_os_str())),
            None,
        );
        assert!(!occupied.status.success());
        assert!(!marker.exists());
        match leaf {
            "file" => assert_eq!(std::fs::read(&store).unwrap(), b"preserve me"),
            "directory" => assert!(store.is_dir()),
            "symlink" => assert!(
                std::fs::symlink_metadata(&store)
                    .unwrap()
                    .file_type()
                    .is_symlink()
            ),
            _ => unreachable!(),
        }
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn test_default_info_detects_ambiguous_and_corrupt_grouped_markers_without_fallback() {
    let root = scratch();
    let xdg = root.join("xdg");
    let store = xdg.join("vlajmaji/stores/default");
    std::fs::create_dir_all(&store).unwrap();
    std::fs::write(store.join("dictionary.sqlite3"), b"not sqlite").unwrap();
    let corrupt = run(&args(&["info"]), &env(None, Some(xdg.as_os_str())), None);
    let corrupt_error = error_json(&corrupt, "store");
    assert!(
        corrupt_error["error"]["message"]
            .as_str()
            .is_some_and(|message| !message.is_empty())
    );

    std::fs::write(store.join("index.sqlite3"), b"legacy marker").unwrap();
    let ambiguous = run(&args(&["info"]), &env(None, Some(xdg.as_os_str())), None);
    error_json(&ambiguous, "store_format");
    assert!(store.join("dictionary.sqlite3").is_file());
    std::fs::remove_file(store.join("dictionary.sqlite3")).unwrap();
    std::fs::remove_file(store.join("index.sqlite3")).unwrap();
    std::fs::write(store.join("index.sqlite3"), b"not legacy sqlite").unwrap();
    let corrupt_legacy = run(&args(&["info"]), &env(None, Some(xdg.as_os_str())), None);
    error_json(&corrupt_legacy, "store");
    std::fs::remove_dir_all(root).unwrap();
}

#[cfg(unix)]
#[test]
fn test_non_utf8_environment_is_preserved_and_build_fails_before_mutation() {
    use std::os::unix::ffi::OsStringExt;

    let root = scratch();
    let mut native_bytes = root.as_os_str().as_encoded_bytes().to_vec();
    native_bytes.extend_from_slice(b"/xdg-\xff");
    let xdg = OsString::from_vec(native_bytes);
    let store = PathBuf::from(&xdg).join("vlajmaji/stores/default");
    let lossy_store = PathBuf::from(xdg.to_string_lossy().as_ref()).join("vlajmaji/stores/default");

    let input = root.join("input.json");
    write_fixture(&input);
    let build = run(
        &[OsStr::new("dictionary-build"), input.as_os_str()],
        &env(None, Some(&xdg)),
        None,
    );
    error_json(&build, "store_path");
    assert!(!store.exists());

    let request = br#"{"operation":"exact","spelling":"farvi"}"#;
    let dictionary = fixture::complete_fixture().unwrap();
    match std::fs::create_dir_all(store.parent().unwrap()) {
        Ok(()) => {
            vlajmaji_store::DictionaryStore::create(&store, &dictionary).unwrap();
            let query = run(
                &args(&["dictionary-query"]),
                &env(None, Some(&xdg)),
                Some(request),
            );
            assert!(
                query.status.success(),
                "{}",
                String::from_utf8_lossy(&query.stderr)
            );
            let explicit_query = run(
                &[OsStr::new("dictionary-query"), store.as_os_str()],
                &env(None, None),
                Some(request),
            );
            assert_eq!(query.stdout, explicit_query.stdout);
            let info = run(&args(&["info"]), &env(None, Some(&xdg)), None);
            let explicit_info = run(
                &[OsStr::new("info"), store.as_os_str()],
                &env(None, None),
                None,
            );
            assert!(info.status.success());
            assert_eq!(info.stdout, explicit_info.stdout);
        }
        Err(error) if matches!(error.raw_os_error(), Some(92) | Some(84)) => {
            // This filesystem rejects invalid filename bytes. A valid U+FFFD
            // counterpart makes lossy routing observably wrong: it would open
            // this decoy store and incorrectly succeed.
            std::fs::create_dir_all(lossy_store.parent().unwrap()).unwrap();
            vlajmaji_store::DictionaryStore::create(&lossy_store, &dictionary).unwrap();
            let query = run(
                &args(&["dictionary-query"]),
                &env(None, Some(&xdg)),
                Some(request),
            );
            assert!(!query.status.success());
            assert!(query.stdout.is_empty());
            assert!(serde_json::from_slice::<Value>(&query.stderr).is_ok());
            let info = run(&args(&["info"]), &env(None, Some(&xdg)), None);
            assert!(!info.status.success());
            assert!(info.stdout.is_empty());
            assert!(serde_json::from_slice::<Value>(&info.stderr).is_ok());
        }
        Err(error) => panic!("unexpected filesystem error for native path probe: {error}"),
    }

    let explicit_store = root.join("explicit-store");
    let explicit_build = run(
        &[
            OsStr::new("dictionary-build"),
            input.as_os_str(),
            explicit_store.as_os_str(),
        ],
        &env(None, Some(&xdg)),
        None,
    );
    assert!(
        explicit_build.status.success(),
        "{}",
        String::from_utf8_lossy(&explicit_build.stderr)
    );
    assert!(explicit_store.join("dictionary.sqlite3").is_file());

    let mut explicit_bytes = root.as_os_str().as_encoded_bytes().to_vec();
    explicit_bytes.extend_from_slice(b"/explicit-\xff");
    let explicit_non_utf8 = OsString::from_vec(explicit_bytes);
    let rejected_explicit = run(
        &[
            OsStr::new("dictionary-build"),
            input.as_os_str(),
            &explicit_non_utf8,
        ],
        &env(None, None),
        None,
    );
    error_json(&rejected_explicit, "store_path");

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn test_optional_store_positionals_are_visible_in_help_and_legacy_info_stays_legacy() {
    let root = scratch();
    let assert_path_help = |output: &Output| {
        assert!(output.status.success());
        let help = String::from_utf8_lossy(&output.stdout);
        assert!(
            help.contains("XDG_DATA_HOME/vlajmaji/stores/default"),
            "{help}"
        );
        assert!(
            help.contains("HOME/.local/share/vlajmaji/stores/default"),
            "{help}"
        );
        assert!(help.contains("Lensisku-only"), "{help}");
    };
    let root_help = run(&args(&["--help"]), &env(None, None), None);
    assert_path_help(&root_help);
    let build_help = run(
        &args(&["dictionary-build", "--help"]),
        &env(None, None),
        None,
    );
    assert_path_help(&build_help);
    assert!(String::from_utf8_lossy(&build_help.stdout).contains("[STORE]"));
    let query_help = run(
        &args(&["dictionary-query", "--help"]),
        &env(None, None),
        None,
    );
    assert_path_help(&query_help);
    assert!(String::from_utf8_lossy(&query_help.stdout).contains("[STORE]"));
    let info_help = run(&args(&["info", "--help"]), &env(None, None), None);
    assert_path_help(&info_help);
    assert!(String::from_utf8_lossy(&info_help.stdout).contains("[STORE]"));

    let archive_input = br#"[{"word":"farvi","definition":"to send"}]"#;
    let mut lexicon = Lexicon::new();
    lexicon
        .import_source(
            SourceMetadata::new(
                "fixture",
                "fixture.json",
                SourceFormat::LaLojbanJson,
                "https://example.invalid/fixture.json",
                "fixture-v1",
                None,
                TimePrecision::Unknown,
            ),
            archive_input,
            None,
        )
        .unwrap();
    let archive = lexicon.to_archive_bytes().unwrap();
    let archive_path = root.join("archive.json");
    let legacy_store = root.join("legacy");
    std::fs::write(&archive_path, archive).unwrap();
    let build = run(
        &args(&[
            "build",
            archive_path.to_str().unwrap(),
            legacy_store.to_str().unwrap(),
        ]),
        &env(None, None),
        None,
    );
    assert!(
        build.status.success(),
        "{}",
        String::from_utf8_lossy(&build.stderr)
    );
    let legacy_info = run(
        &args(&["info", legacy_store.to_str().unwrap()]),
        &env(None, None),
        None,
    );
    assert!(legacy_info.status.success());
    let value: Value = serde_json::from_slice(&legacy_info.stdout).unwrap();
    assert_eq!(
        value
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["morphology_enabled", "records", "snapshots"]
    );
    assert!(value.get("format").is_none());
    assert!(value.get("operation").is_none());

    let missing_explicit = run(
        &args(&["info", root.join("missing-legacy").to_str().unwrap()]),
        &env(None, None),
        None,
    );
    assert!(!missing_explicit.status.success());
    assert!(missing_explicit.stdout.is_empty());
    assert!(!missing_explicit.stderr.starts_with(b"{\"error\":"));
    std::fs::remove_dir_all(root).unwrap();
}
