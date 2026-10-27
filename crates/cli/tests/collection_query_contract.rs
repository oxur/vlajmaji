//! Configured lookup commands use one explicit profile and preserve the old STORE routes.

#[path = "../../lexicon/examples/support/dictionary_fixture.rs"]
mod fixture;

use std::{
    borrow::Cow,
    path::{Path, PathBuf},
    process::{Command, Output},
    time::{SystemTime, UNIX_EPOCH},
};

use serde_json::Value;
use vlajmaji_lexicon::{Dictionary, PackId, PackPayloadKind, seal_release};
use vlajmaji_store::{
    CollectionCatalogue, CollectionInput, CollectionRequest, CollectionSelection,
};

fn fresh_root(label: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "vlajmaji-s03-cli-{label}-{}-{stamp}",
        std::process::id()
    ));
    std::fs::create_dir(&root).unwrap();
    root
}

fn cli(cwd: &Path, args: &[&Path], variables: &[(&str, &Path)]) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_vlajmaji"));
    command
        .current_dir(cwd)
        .args(args)
        .env_clear()
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    for (name, value) in variables {
        command.env(name, value);
    }
    command.output().unwrap()
}

fn tiny_installed(root: &Path) {
    let mut document = fixture::complete_fixture().unwrap().into_document();
    // The contract fixture's derived archive is intentionally not a pack payload.
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
    CollectionCatalogue::install(root, request).unwrap();
}

fn json(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn test_configure_profiles_selection_and_lookup_use_explicit_config_and_root() {
    let root = fresh_root("commands");
    let config = root.join("private").join("config.toml");
    let catalogue = root.join("catalogue");
    tiny_installed(&catalogue);

    let configured = cli(
        &root,
        &[
            Path::new("collections"),
            Path::new("configure"),
            &catalogue,
            Path::new("--profile"),
            Path::new("research"),
            Path::new("--pack"),
            Path::new("llg"),
            Path::new("--make-default"),
            Path::new("--config"),
            &config,
        ],
        &[],
    );
    assert!(
        configured.status.success(),
        "{}",
        String::from_utf8_lossy(&configured.stderr)
    );
    assert_eq!(json(&configured)["outcome"], "committed");
    assert_eq!(json(&configured)["profile"], "research");

    let profiles = cli(
        &root,
        &[
            Path::new("collections"),
            Path::new("profiles"),
            Path::new("--config"),
            &config,
        ],
        &[],
    );
    assert!(
        profiles.status.success(),
        "{}",
        String::from_utf8_lossy(&profiles.stderr)
    );
    assert_eq!(json(&profiles)["default_profile"], "research");
    assert_eq!(
        json(&profiles)["profiles"][0]["packs"],
        serde_json::json!(["llg"])
    );

    let selection = cli(
        &root,
        &[
            Path::new("collections"),
            Path::new("selection"),
            Path::new("--config"),
            &config,
        ],
        &[],
    );
    assert!(
        selection.status.success(),
        "{}",
        String::from_utf8_lossy(&selection.stderr)
    );
    assert_eq!(json(&selection)["local_revision"], 1);

    let lookup = cli(
        &root,
        &[
            Path::new("lookup"),
            Path::new("farvi"),
            Path::new("--config"),
            &config,
        ],
        &[],
    );
    assert!(
        lookup.status.success(),
        "{}",
        String::from_utf8_lossy(&lookup.stderr)
    );
    assert_eq!(json(&lookup)["version"], 1);
    assert_eq!(json(&lookup)["result"]["operation"], "exact");
    assert!(
        json(&lookup)["result"]["response"]["totals"]["occurrences"]
            .as_u64()
            .is_some_and(|count| count > 0)
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn test_configure_relative_file_and_root_share_child_cwd() {
    let cwd = fresh_root("relative-file-root");
    let catalogue = cwd.join("catalogue");
    tiny_installed(&catalogue);
    let relative_root = Path::new("catalogue");
    let relative_config = Path::new("settings/profiles.toml");
    let configured = cli(
        &cwd,
        &[
            Path::new("collections"),
            Path::new("configure"),
            relative_root,
            Path::new("--profile"),
            Path::new("research"),
            Path::new("--pack"),
            Path::new("llg"),
            Path::new("--make-default"),
            Path::new("--config"),
            relative_config,
        ],
        &[],
    );
    assert!(
        configured.status.success(),
        "{}",
        String::from_utf8_lossy(&configured.stderr)
    );
    let profiles = cli(
        &cwd,
        &[
            Path::new("collections"),
            Path::new("profiles"),
            Path::new("--config"),
            relative_config,
        ],
        &[],
    );
    assert!(
        profiles.status.success(),
        "{}",
        String::from_utf8_lossy(&profiles.stderr)
    );
    assert_eq!(
        json(&profiles)["profiles"][0]["root"]["utf8"],
        std::fs::canonicalize(&cwd)
            .unwrap()
            .join(relative_root)
            .to_string_lossy()
            .as_ref()
    );
    assert!(cwd.join(relative_config).is_file());
    std::fs::remove_dir_all(cwd).unwrap();
}

#[cfg(target_os = "macos")]
#[test]
fn test_profiles_reports_exact_native_config_io_and_keeps_ascii_counterpart_working() {
    use std::os::unix::ffi::OsStringExt;

    let root = fresh_root("native-config-io");
    let catalogue = root.join("catalogue");
    tiny_installed(&catalogue);
    let native_path = root.join(std::ffi::OsString::from_vec(b"profiles-\xff.toml".to_vec()));
    let lock_path = root.join(std::ffi::OsString::from_vec(
        b"profiles-\xff.toml.lock".to_vec(),
    ));
    let direct = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&lock_path)
        .unwrap_err();
    let lossy_path = root.join("profiles-\u{fffd}.toml");
    assert!(!lossy_path.exists());

    let failed = cli(
        &root,
        &[
            Path::new("collections"),
            Path::new("configure"),
            &catalogue,
            Path::new("--profile"),
            Path::new("research"),
            Path::new("--pack"),
            Path::new("llg"),
            Path::new("--config"),
            &native_path,
        ],
        &[],
    );
    assert!(!failed.status.success());
    let envelope: Value = serde_json::from_slice(&failed.stderr).unwrap();
    let error = &envelope["error"];
    assert_eq!(error["version"], 1);
    assert_eq!(error["category"], "config_io");
    assert_eq!(error["operation"], "configure");
    assert_eq!(error["outcome"], "unchanged");
    assert_eq!(error["config_path"]["encoding"], "unix-bytes");
    assert_eq!(
        error["config_path"]["hex"],
        vlajmaji_store::native_path_report(&native_path).hex
    );
    assert_eq!(error["config_path"]["utf8"], Value::Null);
    assert!(
        error["detail"]
            .as_str()
            .unwrap()
            .contains(&direct.to_string()),
        "detail {:?}; direct {:?}; envelope {envelope}",
        error["detail"].as_str(),
        direct
    );
    assert!(!lossy_path.exists());

    let ascii_config = root.join("profiles.toml");
    let configured = cli(
        &root,
        &[
            Path::new("collections"),
            Path::new("configure"),
            &catalogue,
            Path::new("--profile"),
            Path::new("research"),
            Path::new("--pack"),
            Path::new("llg"),
            Path::new("--config"),
            &ascii_config,
        ],
        &[],
    );
    assert!(configured.status.success());
    let valid = cli(
        &root,
        &[
            Path::new("collections"),
            Path::new("profiles"),
            Path::new("--config"),
            &ascii_config,
        ],
        &[],
    );
    assert!(valid.status.success());
    assert_eq!(json(&valid)["profiles"][0]["name"], "research");
    assert!(!lossy_path.exists());
    std::fs::remove_dir_all(root).unwrap();
}

#[cfg(target_os = "macos")]
#[test]
fn test_configure_reports_exact_native_root_path_and_keeps_ascii_root_working() {
    use std::os::unix::ffi::OsStringExt;

    let root = fresh_root("native-root-path");
    let config = root.join("profiles.toml");
    let native_root = root.join(std::ffi::OsString::from_vec(b"catalogue-\xff".to_vec()));
    let lossy_root = root.join("catalogue-\u{fffd}");
    assert!(!lossy_root.exists());
    let failed = cli(
        &root,
        &[
            Path::new("collections"),
            Path::new("configure"),
            &native_root,
            Path::new("--profile"),
            Path::new("research"),
            Path::new("--pack"),
            Path::new("llg"),
            Path::new("--config"),
            &config,
        ],
        &[],
    );
    assert!(!failed.status.success());
    let envelope: Value = serde_json::from_slice(&failed.stderr).unwrap();
    let error = &envelope["error"];
    assert_eq!(error["version"], 1);
    assert_eq!(error["operation"], "configure");
    assert_eq!(error["outcome"], "unchanged");
    assert_eq!(error["root_path"]["encoding"], "unix-bytes");
    assert_eq!(
        error["root_path"]["hex"],
        vlajmaji_store::native_path_report(&native_root).hex
    );
    assert_eq!(error["root_path"]["utf8"], Value::Null);
    assert_eq!(error["category"], "catalogue");
    assert!(
        error["detail"]
            .as_str()
            .unwrap()
            .contains("catalogue is missing")
    );
    assert!(!lossy_root.exists());

    let ascii_root = root.join("catalogue");
    tiny_installed(&ascii_root);
    let configured = cli(
        &root,
        &[
            Path::new("collections"),
            Path::new("configure"),
            &ascii_root,
            Path::new("--profile"),
            Path::new("research"),
            Path::new("--pack"),
            Path::new("llg"),
            Path::new("--config"),
            &config,
        ],
        &[],
    );
    assert!(configured.status.success());
    assert!(config.is_file());
    assert!(!lossy_root.exists());
    std::fs::remove_dir_all(root).unwrap();
}

#[cfg(target_os = "linux")]
#[test]
fn test_non_utf8_installed_root_roundtrips_through_profile_and_lookup() {
    use std::os::unix::ffi::OsStringExt;

    let parent = fresh_root("native-root-roundtrip");
    let native_root = parent.join(std::ffi::OsString::from_vec(b"catalogue-\xff".to_vec()));
    let lossy_root = parent.join("catalogue-\u{fffd}");
    tiny_installed(&native_root);
    assert!(!lossy_root.exists());
    let config = parent.join("profiles.toml");
    let configured = cli(
        &parent,
        &[
            Path::new("collections"),
            Path::new("configure"),
            &native_root,
            Path::new("--profile"),
            Path::new("research"),
            Path::new("--pack"),
            Path::new("llg"),
            Path::new("--make-default"),
            Path::new("--config"),
            &config,
        ],
        &[],
    );
    assert!(
        configured.status.success(),
        "{}",
        String::from_utf8_lossy(&configured.stderr)
    );
    assert_eq!(json(&configured)["outcome"], "committed");

    let profiles = cli(
        &parent,
        &[
            Path::new("collections"),
            Path::new("profiles"),
            Path::new("--config"),
            &config,
        ],
        &[],
    );
    assert!(
        profiles.status.success(),
        "{}",
        String::from_utf8_lossy(&profiles.stderr)
    );
    let root_report = &json(&profiles)["profiles"][0]["root"];
    assert_eq!(root_report["encoding"], "unix-bytes");
    assert_eq!(
        root_report["hex"],
        vlajmaji_store::native_path_report(&native_root).hex
    );
    assert_eq!(root_report["utf8"], Value::Null);
    assert!(!lossy_root.exists());

    let lookup = cli(
        &parent,
        &[
            Path::new("lookup"),
            Path::new("farvi"),
            Path::new("--config"),
            &config,
        ],
        &[],
    );
    assert!(
        lookup.status.success(),
        "{}",
        String::from_utf8_lossy(&lookup.stderr)
    );
    assert!(
        json(&lookup)["result"]["response"]["totals"]["occurrences"]
            .as_u64()
            .is_some_and(|count| count > 0)
    );
    assert!(!lossy_root.exists());
    std::fs::remove_dir_all(parent).unwrap();
}

#[cfg(unix)]
#[test]
fn test_absolute_config_and_root_work_without_child_cwd_but_relative_config_fails_typed() {
    let root = fresh_root("deleted-cwd");
    let catalogue = root.join("catalogue");
    tiny_installed(&catalogue);
    let config = root.join("profiles.toml");
    let absolute_cwd = root.join("absolute-child-cwd");
    std::fs::create_dir(&absolute_cwd).unwrap();
    let absolute = Command::new("/bin/sh")
        .current_dir(&absolute_cwd)
        .args(["-c", "rmdir \"$1\" || exit; shift; exec \"$@\"", "sh"])
        .arg(&absolute_cwd)
        .arg(env!("CARGO_BIN_EXE_vlajmaji"))
        .args([
            "collections",
            "configure",
            catalogue.to_str().unwrap(),
            "--profile",
            "research",
            "--pack",
            "llg",
            "--config",
            config.to_str().unwrap(),
        ])
        .env_clear()
        .output()
        .unwrap();
    assert!(
        absolute.status.success(),
        "{}",
        String::from_utf8_lossy(&absolute.stderr)
    );
    assert!(!absolute_cwd.exists());
    assert!(config.is_file());

    let relative_cwd = root.join("relative-child-cwd");
    std::fs::create_dir(&relative_cwd).unwrap();
    let relative = Command::new("/bin/sh")
        .current_dir(&relative_cwd)
        .args(["-c", "rmdir \"$1\" || exit; shift; exec \"$@\"", "sh"])
        .arg(&relative_cwd)
        .arg(env!("CARGO_BIN_EXE_vlajmaji"))
        .args(["collections", "profiles", "--config", "relative.toml"])
        .env_clear()
        .output()
        .unwrap();
    assert!(!relative.status.success());
    assert!(!relative_cwd.exists());
    let error: Value = serde_json::from_slice(&relative.stderr).unwrap();
    assert_eq!(error["error"]["version"], 1);
    assert_eq!(error["error"]["category"], "config_io");
    assert_eq!(error["error"]["operation"], "profiles");
    assert_eq!(error["error"]["config_path"]["utf8"], "relative.toml");
    let detail = error["error"]["detail"].as_str().unwrap();
    assert!(detail.contains("resolve current directory"), "{detail}");
    assert!(
        detail.contains("os error"),
        "missing current-dir cause: {detail}"
    );

    let absent = root.join("absent-profile-file.toml");
    let absent_output = cli(
        &root,
        &[
            Path::new("collections"),
            Path::new("profiles"),
            Path::new("--config"),
            &absent,
        ],
        &[],
    );
    assert!(!absent_output.status.success());
    let absent_error: Value = serde_json::from_slice(&absent_output.stderr).unwrap();
    assert_eq!(absent_error["error"]["category"], "config_missing");
    assert_eq!(
        absent_error["error"]["config_path"]["utf8"],
        absent.to_str().unwrap()
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn test_lookup_resolves_absolute_xdg_config_and_rejects_direct_override_of_request() {
    let root = fresh_root("xdg");
    let catalogue = root.join("catalogue");
    let config_home = root.join("xdg");
    let config = config_home.join("vlajmaji/config.toml");
    tiny_installed(&catalogue);
    let configured = cli(
        &root,
        &[
            Path::new("collections"),
            Path::new("configure"),
            &catalogue,
            Path::new("--profile"),
            Path::new("research"),
            Path::new("--pack"),
            Path::new("llg"),
            Path::new("--make-default"),
            Path::new("--config"),
            &config,
        ],
        &[],
    );
    assert!(
        configured.status.success(),
        "{}",
        String::from_utf8_lossy(&configured.stderr)
    );

    let lookup = cli(
        &root,
        &[Path::new("lookup"), Path::new("farvi")],
        &[("XDG_CONFIG_HOME", &config_home)],
    );
    assert!(
        lookup.status.success(),
        "{}",
        String::from_utf8_lossy(&lookup.stderr)
    );
    assert_eq!(json(&lookup)["result"]["operation"], "exact");

    let conflict = cli(
        &root,
        &[
            Path::new("lookup"),
            Path::new("farvi"),
            Path::new("--request"),
            Path::new("{\"operation\":\"exact\",\"spelling\":\"farvi\"}"),
        ],
        &[("XDG_CONFIG_HOME", &config_home)],
    );
    assert_eq!(conflict.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&conflict.stderr).contains("cannot be used with"));
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn test_lookup_request_parse_precedes_missing_config_and_uses_managed_error_envelope() {
    let root = fresh_root("request-precedence");
    let missing_config = root.join("does-not-exist.toml");
    let output = cli(
        &root,
        &[
            Path::new("lookup"),
            Path::new("--request"),
            Path::new("{not json"),
            Path::new("--config"),
            &missing_config,
        ],
        &[],
    );
    assert!(!output.status.success());
    let envelope: Value = serde_json::from_slice(&output.stderr).unwrap();
    let report = &envelope["error"];
    assert_eq!(report["version"], 1);
    assert_eq!(report["ok"], false);
    assert_eq!(report["category"], "request_invalid");
    assert_eq!(report["operation"], "lookup");
    assert_eq!(report["outcome"], "not_applicable");
    assert_eq!(report["config_path"], Value::Null);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn test_all_managed_commands_report_config_path_for_unusable_home_fallback() {
    let cwd = fresh_root("config-location-errors");
    let missing_root = cwd.join("must-not-be-created");
    let cases: [(Option<&Path>, Option<&Path>); 3] = [
        (None, None),
        (Some(Path::new("")), Some(Path::new(""))),
        (
            Some(Path::new("relative-home")),
            Some(Path::new("relative-xdg")),
        ),
    ];
    for (home, xdg) in cases {
        let mut variables = Vec::new();
        if let Some(value) = home {
            variables.push(("HOME", value));
        }
        if let Some(value) = xdg {
            variables.push(("XDG_CONFIG_HOME", value));
        }
        let commands: Vec<Vec<&Path>> = vec![
            vec![
                Path::new("collections"),
                Path::new("configure"),
                &missing_root,
                Path::new("--profile"),
                Path::new("research"),
                Path::new("--pack"),
                Path::new("llg"),
            ],
            vec![Path::new("collections"), Path::new("profiles")],
            vec![
                Path::new("collections"),
                Path::new("selection"),
                Path::new("--profile"),
                Path::new("research"),
                Path::new("--pack"),
                Path::new("llg"),
            ],
            vec![
                Path::new("lookup"),
                Path::new("farvi"),
                Path::new("--profile"),
                Path::new("research"),
                Path::new("--pack"),
                Path::new("llg"),
            ],
        ];
        for (index, args) in commands.iter().enumerate() {
            let output = cli(&cwd, args, &variables);
            assert!(!output.status.success());
            let envelope: Value = serde_json::from_slice(&output.stderr).unwrap();
            let report = &envelope["error"];
            let operation = ["configure", "profiles", "selection", "lookup"][index];
            assert_eq!(report["version"], 1);
            assert_eq!(report["ok"], false);
            assert_eq!(report["category"], "config_path");
            assert_eq!(report["operation"], operation);
            assert_eq!(report["config_path"], Value::Null);
            assert_eq!(
                report["outcome"],
                if operation == "configure" {
                    "unchanged"
                } else {
                    "not_applicable"
                }
            );
            if operation == "configure" {
                assert_eq!(report["root_path"]["utf8"], missing_root.to_str().unwrap());
                assert_eq!(report["requested_profile"], "research");
                assert_eq!(report["known_pack_ids"], serde_json::json!(["llg"]));
            } else if operation == "selection" || operation == "lookup" {
                assert_eq!(report["requested_profile"], "research");
                assert_eq!(report["known_pack_ids"], serde_json::json!(["llg"]));
            }
        }
    }
    assert!(!missing_root.exists());
    std::fs::remove_dir_all(cwd).unwrap();
}

#[test]
fn test_configure_failure_reports_known_file_root_profile_and_requested_packs() {
    let parent = fresh_root("configure-error-context");
    let config = parent.join("profiles.toml");
    let root = parent.join("missing-catalogue");
    let output = cli(
        &parent,
        &[
            Path::new("collections"),
            Path::new("configure"),
            &root,
            Path::new("--profile"),
            Path::new("research"),
            Path::new("--pack"),
            Path::new("llg"),
            Path::new("--config"),
            &config,
        ],
        &[],
    );

    assert!(!output.status.success());
    let envelope: Value = serde_json::from_slice(&output.stderr).unwrap();
    let report = &envelope["error"];
    assert_eq!(report["version"], 1);
    assert_eq!(report["ok"], false);
    assert_eq!(report["operation"], "configure");
    assert_eq!(report["category"], "catalogue");
    assert_eq!(report["outcome"], "unchanged");
    assert!(report["config_path"].is_object());
    assert!(report["root_path"].is_object());
    assert_eq!(report["requested_profile"], "research");
    assert_eq!(report["known_pack_ids"], serde_json::json!(["llg"]));
    assert_eq!(report["writer"], Value::Null);
    assert!(!config.exists());
    std::fs::remove_dir_all(parent).unwrap();
}

#[test]
fn test_configure_invalid_pack_keeps_known_context_without_creating_files() {
    let parent = fresh_root("configure-invalid-pack-context");
    let config = parent.join("profiles.toml");
    let root = parent.join("catalogue");
    let output = cli(
        &parent,
        &[
            Path::new("collections"),
            Path::new("configure"),
            &root,
            Path::new("--profile"),
            Path::new("research"),
            Path::new("--pack"),
            Path::new("not-a-pack"),
            Path::new("--config"),
            &config,
        ],
        &[],
    );

    assert!(!output.status.success());
    let envelope: Value = serde_json::from_slice(&output.stderr).unwrap();
    let report = &envelope["error"];
    assert_eq!(report["category"], "unknown_selection");
    assert_eq!(report["operation"], "configure");
    assert_eq!(report["outcome"], "unchanged");
    assert!(report["config_path"].is_object());
    assert!(report["root_path"].is_object());
    assert_eq!(report["requested_profile"], "research");
    assert_eq!(report["known_pack_ids"], serde_json::json!([]));
    assert!(!config.exists());
    assert!(!root.exists());
    std::fs::remove_dir_all(parent).unwrap();
}

#[test]
fn test_cli_configure_preserves_hand_authored_disabled_policy_and_noop_bytes() {
    let parent = fresh_root("configure-preserves-disabled");
    let config = parent.join("profiles.toml");
    let root = parent.join("catalogue");
    tiny_installed(&root);
    let initial = format!(
        "version = 1\nrevision = 9\n\n[profiles.research]\nroot = {:?}\npacks = []\ndisabled = ['llg']\n",
        root.to_string_lossy()
    );
    std::fs::write(&config, &initial).unwrap();

    let invoke = || {
        cli(
            &parent,
            &[
                Path::new("collections"),
                Path::new("configure"),
                &root,
                Path::new("--profile"),
                Path::new("research"),
                Path::new("--pack"),
                Path::new("llg"),
                Path::new("--make-default"),
                Path::new("--config"),
                &config,
            ],
            &[],
        )
    };
    let configured = invoke();
    assert!(
        configured.status.success(),
        "{}",
        String::from_utf8_lossy(&configured.stderr)
    );
    assert_eq!(json(&configured)["outcome"], "committed");
    let profiles = cli(
        &parent,
        &[
            Path::new("collections"),
            Path::new("profiles"),
            Path::new("--config"),
            &config,
        ],
        &[],
    );
    assert!(
        profiles.status.success(),
        "{}",
        String::from_utf8_lossy(&profiles.stderr)
    );
    assert_eq!(
        json(&profiles)["profiles"][0]["packs"],
        serde_json::json!(["llg"])
    );
    assert_eq!(
        json(&profiles)["profiles"][0]["disabled"],
        serde_json::json!(["llg"])
    );
    assert_eq!(json(&profiles)["default_profile"], "research");

    let bytes = std::fs::read(&config).unwrap();
    let sha = json(&profiles)["sha256"].as_str().unwrap().to_owned();
    let revision = json(&profiles)["revision"].as_u64().unwrap();
    let noop = invoke();
    assert!(
        noop.status.success(),
        "{}",
        String::from_utf8_lossy(&noop.stderr)
    );
    assert_eq!(json(&noop)["outcome"], "unchanged");
    let after = cli(
        &parent,
        &[
            Path::new("collections"),
            Path::new("profiles"),
            Path::new("--config"),
            &config,
        ],
        &[],
    );
    assert!(
        after.status.success(),
        "{}",
        String::from_utf8_lossy(&after.stderr)
    );
    assert_eq!(json(&after)["sha256"], sha);
    assert_eq!(json(&after)["revision"], revision);
    assert_eq!(std::fs::read(&config).unwrap(), bytes);
    std::fs::remove_dir_all(parent).unwrap();
}

#[test]
fn test_lookup_profile_and_disabled_selection_fail_with_typed_context() {
    let parent = fresh_root("typed-profile-selection-errors");
    let config = parent.join("profiles.toml");
    let root = parent.join("catalogue");
    tiny_installed(&root);

    std::fs::write(
        &config,
        format!(
            "version = 1\nrevision = 1\ndefault_profile = 'research'\n\n[profiles.research]\nroot = {:?}\npacks = []\ndisabled = ['llg']\n",
            root.to_string_lossy()
        ),
    )
    .unwrap();

    let unknown = cli(
        &parent,
        &[
            Path::new("lookup"),
            Path::new("farvi"),
            Path::new("--profile"),
            Path::new("missing"),
            Path::new("--config"),
            &config,
        ],
        &[],
    );
    assert!(!unknown.status.success());
    let report = &serde_json::from_slice::<Value>(&unknown.stderr).unwrap()["error"];
    assert_eq!(report["category"], "unknown_profile");
    assert_eq!(report["operation"], "lookup");
    assert_eq!(report["requested_profile"], "missing");
    assert!(report["config_path"].is_object());
    assert_eq!(report["root_path"], Value::Null);

    let disabled = cli(
        &parent,
        &[
            Path::new("lookup"),
            Path::new("farvi"),
            Path::new("--pack"),
            Path::new("llg"),
            Path::new("--config"),
            &config,
        ],
        &[],
    );
    assert!(!disabled.status.success());
    let report = &serde_json::from_slice::<Value>(&disabled.stderr).unwrap()["error"];
    assert_eq!(report["category"], "disabled_selection");
    assert_eq!(report["operation"], "lookup");
    assert!(report["config_path"].is_object());
    assert!(report["root_path"].is_object());
    assert_eq!(report["profile"]["name"], "research");
    assert_eq!(report["known_pack_ids"], serde_json::json!(["llg"]));

    std::fs::write(
        &config,
        format!(
            "version = 1\nrevision = 2\n[profiles.research]\nroot = {:?}\npacks = []\ndisabled = []\n",
            root.to_string_lossy()
        ),
    )
    .unwrap();
    let missing_default = cli(
        &parent,
        &[
            Path::new("lookup"),
            Path::new("farvi"),
            Path::new("--config"),
            &config,
        ],
        &[],
    );
    assert!(!missing_default.status.success());
    let report = &serde_json::from_slice::<Value>(&missing_default.stderr).unwrap()["error"];
    assert_eq!(report["category"], "profile_default_missing");
    assert_eq!(report["operation"], "lookup");
    assert!(report["config_path"].is_object());
    assert_eq!(report["requested_profile"], Value::Null);
    std::fs::remove_dir_all(parent).unwrap();
}

#[test]
fn test_configure_commits_when_stdout_reader_closes_early() {
    let parent = fresh_root("configure-lost-output");
    let config = parent.join("profiles.toml");
    let root = parent.join("catalogue");
    tiny_installed(&root);
    std::fs::write(
        &config,
        format!(
            "version = 1\nrevision = 1\ndefault_profile = 'research'\n\n[profiles.research]\nroot = {:?}\npacks = []\ndisabled = []\n",
            root.to_string_lossy()
        ),
    )
    .unwrap();

    let mut child = Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
        .current_dir(&parent)
        .args([
            "collections",
            "configure",
            root.to_str().unwrap(),
            "--profile",
            "research",
            "--pack",
            "llg",
            "--config",
            config.to_str().unwrap(),
        ])
        .env_clear()
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap();
    drop(child.stdout.take());
    let status = child.wait().unwrap();
    assert!(
        status.success(),
        "mutation should remain successful after BrokenPipe"
    );

    let profiles = cli(
        &parent,
        &[
            Path::new("collections"),
            Path::new("profiles"),
            Path::new("--config"),
            &config,
        ],
        &[],
    );
    assert!(profiles.status.success());
    assert_eq!(json(&profiles)["revision"], 2);
    assert_eq!(
        json(&profiles)["profiles"][0]["packs"],
        serde_json::json!(["llg"])
    );
    std::fs::remove_dir_all(parent).unwrap();
}

#[test]
fn test_profiles_reports_not_applicable_context_when_stdout_file_limit_fails() {
    let parent = fresh_root("configure-output-file-limit");
    let config = parent.join("profiles.toml");
    let root = parent.join("catalogue");
    let output_file = parent.join("child-stdout.json");
    tiny_installed(&root);
    std::fs::write(
        &config,
        format!(
            "version = 1\nrevision = 1\ndefault_profile = 'research'\n\n[profiles.research]\nroot = {:?}\npacks = []\ndisabled = []\n",
            root.to_string_lossy()
        ),
    )
    .unwrap();
    let output_sink = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&output_file)
        .unwrap();

    let output = Command::new("/bin/bash")
        .current_dir(&parent)
        .args([
            "-c",
            "trap '' XFSZ; ulimit -f 0; exec \"$@\"",
            "wrapper",
            env!("CARGO_BIN_EXE_vlajmaji"),
            "collections",
            "profiles",
            "--config",
            config.to_str().unwrap(),
        ])
        .env_clear()
        .stdout(output_sink)
        .stderr(std::process::Stdio::piped())
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert_eq!(std::fs::metadata(&output_file).unwrap().len(), 0);
    let envelope: Value = serde_json::from_slice(&output.stderr).unwrap();
    let report = &envelope["error"];
    assert_eq!(report["version"], 1);
    assert_eq!(report["ok"], false);
    assert_eq!(report["category"], "response_output_io");
    assert_eq!(report["operation"], "profiles");
    assert_eq!(report["outcome"], "not_applicable");
    assert!(report["config_path"].is_object());
    assert_eq!(report["root_path"], Value::Null);
    assert_eq!(report["requested_profile"], Value::Null);
    assert_eq!(report["known_pack_ids"], serde_json::json!([]));
    assert!(
        report["detail"]
            .as_str()
            .unwrap()
            .contains("File too large")
    );

    let profiles = cli(
        &parent,
        &[
            Path::new("collections"),
            Path::new("profiles"),
            Path::new("--config"),
            &config,
        ],
        &[],
    );
    assert!(profiles.status.success());
    assert_eq!(json(&profiles)["revision"], 1);
    assert_eq!(
        json(&profiles)["profiles"][0]["packs"],
        serde_json::json!([])
    );
    std::fs::remove_dir_all(parent).unwrap();
}

#[cfg(target_os = "linux")]
#[test]
fn test_managed_stdout_full_reports_read_and_committed_write_failures() {
    let parent = fresh_root("configure-dev-full");
    let config = parent.join("profiles.toml");
    let root = parent.join("catalogue");
    tiny_installed(&root);
    std::fs::write(
        &config,
        format!(
            "version = 1\nrevision = 1\ndefault_profile = 'research'\n\n[profiles.research]\nroot = {:?}\npacks = []\ndisabled = []\n",
            root.to_string_lossy()
        ),
    )
    .unwrap();

    let read_failure = Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
        .current_dir(&parent)
        .args([
            "collections",
            "profiles",
            "--config",
            config.to_str().unwrap(),
        ])
        .env_clear()
        .stdout(std::process::Stdio::from(
            std::fs::OpenOptions::new()
                .write(true)
                .open("/dev/full")
                .unwrap(),
        ))
        .stderr(std::process::Stdio::piped())
        .output()
        .unwrap();
    assert!(!read_failure.status.success());
    let read_report: Value = serde_json::from_slice(&read_failure.stderr).unwrap();
    assert_eq!(read_report["error"]["category"], "response_output_io");
    assert_eq!(read_report["error"]["operation"], "profiles");
    assert_eq!(read_report["error"]["outcome"], "not_applicable");

    let write_failure = Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
        .current_dir(&parent)
        .args([
            "collections",
            "configure",
            root.to_str().unwrap(),
            "--profile",
            "research",
            "--pack",
            "llg",
            "--config",
            config.to_str().unwrap(),
        ])
        .env_clear()
        .stdout(std::process::Stdio::from(
            std::fs::OpenOptions::new()
                .write(true)
                .open("/dev/full")
                .unwrap(),
        ))
        .stderr(std::process::Stdio::piped())
        .output()
        .unwrap();
    assert!(!write_failure.status.success());
    let write_report: Value = serde_json::from_slice(&write_failure.stderr).unwrap();
    assert_eq!(write_report["error"]["category"], "response_output_io");
    assert_eq!(write_report["error"]["operation"], "configure");
    assert_eq!(write_report["error"]["outcome"], "committed");
    assert_eq!(write_report["error"]["requested_profile"], "research");
    assert_eq!(
        write_report["error"]["known_pack_ids"],
        serde_json::json!(["llg"])
    );

    let committed = cli(
        &parent,
        &[
            Path::new("collections"),
            Path::new("profiles"),
            Path::new("--config"),
            &config,
        ],
        &[],
    );
    assert!(committed.status.success());
    assert_eq!(json(&committed)["revision"], 2);
    assert_eq!(
        json(&committed)["profiles"][0]["packs"],
        serde_json::json!(["llg"])
    );
    std::fs::remove_dir_all(parent).unwrap();
}

#[test]
fn test_collections_help_distinguishes_explicit_root_from_config_home_resolution() {
    let root = fresh_root("collections-help");
    let output = cli(&root, &[Path::new("collections"), Path::new("--help")], &[]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let help = String::from_utf8(output.stdout).unwrap();
    assert!(help.contains("explicit ROOT"));
    assert!(help.contains("XDG_CONFIG_HOME/HOME configuration"));
    assert!(!help.contains("do not resolve or modify HOME/XDG defaults"));
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn test_all_four_fixture_packs_keep_source_attribution_in_configured_lookup() {
    let root = fresh_root("four-pack");
    let catalogue = root.join("catalogue");
    let mut records = Vec::new();
    for (command, pack_id, label) in [
        ("jbotci", "lensisku-jbotci", "jbotci"),
        ("llg", "llg", "llg"),
        ("la-lojban", "la-lojban", "la"),
        ("sutysisku", "sutysisku", "sut"),
    ] {
        let output = root.join(format!("{label}-payload"));
        let materialized = cli(
            &root,
            &[
                Path::new("data-pack"),
                Path::new("fixture"),
                Path::new(command),
                &output,
            ],
            &[],
        );
        assert!(
            materialized.status.success(),
            "{command}: {}",
            String::from_utf8_lossy(&materialized.stderr)
        );
        records.push(serde_json::json!({
            "kind":"sealed", "pack_id":pack_id,
            "release":output.join("release.json"), "seal":output.join("seal.json")
        }));
    }
    let manifest = root.join("packs.json");
    std::fs::write(
        &manifest,
        serde_json::to_vec(&serde_json::json!({"version":1,"inputs":records})).unwrap(),
    )
    .unwrap();
    let registered = cli(
        &root,
        &[
            Path::new("collections"),
            Path::new("register"),
            &catalogue,
            Path::new("--manifest"),
            &manifest,
            Path::new("--all"),
            Path::new("--allow-synthetic"),
        ],
        &[],
    );
    assert!(
        registered.status.success(),
        "{}",
        String::from_utf8_lossy(&registered.stderr)
    );
    let installed = cli(
        &root,
        &[
            Path::new("collections"),
            Path::new("install"),
            &catalogue,
            Path::new("--manifest"),
            &manifest,
            Path::new("--all"),
            Path::new("--allow-synthetic"),
        ],
        &[],
    );
    assert!(
        installed.status.success(),
        "{}",
        String::from_utf8_lossy(&installed.stderr)
    );
    let configured = cli(
        &root,
        &[
            Path::new("collections"),
            Path::new("configure"),
            &catalogue,
            Path::new("--profile"),
            Path::new("all"),
            Path::new("--all"),
            Path::new("--make-default"),
            Path::new("--config"),
            &root.join("config.toml"),
        ],
        &[],
    );
    assert!(
        configured.status.success(),
        "{}",
        String::from_utf8_lossy(&configured.stderr)
    );
    let config = root.join("config.toml");
    let lookup = cli(
        &root,
        &[
            Path::new("lookup"),
            Path::new("farvi"),
            Path::new("--config"),
            &config,
        ],
        &[],
    );
    assert!(
        lookup.status.success(),
        "{}",
        String::from_utf8_lossy(&lookup.stderr)
    );
    let response = json(&lookup);
    // These are the literal CLI fixture populations: two lexical rows each in
    // jbotci and La, one LLG gismu, and one Sutysisku gismu (plus wiki rows).
    assert_eq!(response["result"]["response"]["totals"]["occurrences"], 6);
    assert_eq!(response["result"]["response"]["totals"]["source_groups"], 3);
    let source_totals = response["result"]["response"]["source_totals"]
        .as_array()
        .unwrap();
    let counts: std::collections::BTreeMap<_, _> = source_totals
        .iter()
        .map(|row| {
            (
                row["source"]["id"].as_str().unwrap().to_owned(),
                row["occurrences"].as_u64().unwrap(),
            )
        })
        .collect();
    assert_eq!(counts.get("lensisku"), Some(&4));
    assert_eq!(counts.get("llg"), Some(&1));
    assert_eq!(counts.get("sutysisku"), Some(&1));

    let malformed_cursor = cli(
        &root,
        &[
            Path::new("lookup"),
            Path::new("farvi"),
            Path::new("--cursor"),
            Path::new(
                "collection-v1:aéx:0000000000000000000000000000000000000000000000000000000000000000",
            ),
            Path::new("--config"),
            &config,
        ],
        &[],
    );
    assert!(!malformed_cursor.status.success());
    let malformed_error: Value = serde_json::from_slice(&malformed_cursor.stderr).unwrap();
    assert_eq!(malformed_error["error"]["category"], "malformed_cursor");

    let original_config = std::fs::read(&config).unwrap();
    let page_one = cli(
        &root,
        &[
            Path::new("lookup"),
            Path::new("farvi"),
            Path::new("--page-size"),
            Path::new("2"),
            Path::new("--config"),
            &config,
        ],
        &[],
    );
    assert!(
        page_one.status.success(),
        "{}",
        String::from_utf8_lossy(&page_one.stderr)
    );
    let page_one_json = json(&page_one);
    let page_one_response = &page_one_json["result"]["response"];
    assert_eq!(page_one_response["totals"]["occurrences"], 6);
    let first_cursor = page_one_response["next_cursor"]
        .as_str()
        .unwrap()
        .to_owned();
    let hit_ids = |groups: &Value| {
        groups
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|group| group["hits"].as_array().unwrap())
            .map(|hit| hit["occurrence_id"].clone())
            .collect::<Vec<_>>()
    };

    // A changed policy invalidates the captured cursor before no-enabled admission.
    std::fs::write(
        &config,
        format!(
            "version = 1\nrevision = 99\ndefault_profile = 'all'\n\n[profiles.all]\nroot = {:?}\npacks = ['lensisku-jbotci', 'llg', 'la-lojban', 'sutysisku']\ndisabled = ['lensisku-jbotci', 'llg', 'la-lojban', 'sutysisku']\n",
            catalogue.to_string_lossy()
        ),
    )
    .unwrap();
    let stale = cli(
        &root,
        &[
            Path::new("lookup"),
            Path::new("farvi"),
            Path::new("--page-size"),
            Path::new("2"),
            Path::new("--cursor"),
            Path::new(&first_cursor),
            Path::new("--config"),
            &config,
        ],
        &[],
    );
    assert!(!stale.status.success());
    let stale_error: Value = serde_json::from_slice(&stale.stderr).unwrap();
    assert_eq!(stale_error["error"]["category"], "stale_cursor");
    let unavailable = cli(
        &root,
        &[
            Path::new("lookup"),
            Path::new("farvi"),
            Path::new("--config"),
            &config,
        ],
        &[],
    );
    assert!(!unavailable.status.success());
    let unavailable_error: Value = serde_json::from_slice(&unavailable.stderr).unwrap();
    assert_eq!(
        unavailable_error["error"]["category"],
        "no_enabled_selection"
    );

    std::fs::write(&config, &original_config).unwrap();
    let expected_totals = response["result"]["response"]["totals"].clone();
    let expected_source_totals = response["result"]["response"]["source_totals"].clone();
    let expected_hit_ids = hit_ids(&response["result"]["response"]["groups"]);
    let mut joined_hit_ids = hit_ids(&page_one_response["groups"]);
    let mut cursor = Some(first_cursor);
    let mut page_count = 1;
    while let Some(cursor_value) = cursor.take() {
        assert!(
            page_count < 8,
            "bounded page traversal unexpectedly exceeded seven pages"
        );
        let output = cli(
            &root,
            &[
                Path::new("lookup"),
                Path::new("farvi"),
                Path::new("--page-size"),
                Path::new("2"),
                Path::new("--cursor"),
                Path::new(&cursor_value),
                Path::new("--config"),
                &config,
            ],
            &[],
        );
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let page_json = json(&output);
        let response = &page_json["result"]["response"];
        assert_eq!(response["totals"], expected_totals);
        assert_eq!(response["source_totals"], expected_source_totals);
        joined_hit_ids.extend(hit_ids(&response["groups"]));
        cursor = response["next_cursor"].as_str().map(str::to_owned);
        page_count += 1;
    }
    assert_eq!(page_count, 3);
    assert_eq!(joined_hit_ids, expected_hit_ids);
    std::fs::remove_dir_all(root).unwrap();
}
