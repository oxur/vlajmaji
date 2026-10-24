//! Process-level checks for the jbotci import and check commands.

use std::fs;
use std::process::Command;

fn scratch() -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!(
        "vlajmaji jbotci cli-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&path).unwrap();
    path
}

fn metadata(definitions: u64, entries: u64) -> Vec<u8> {
    format!(
        "language_tag = \"en\"\nlanguage_realname = \"English\"\nsource_language_tag = \"jbo\"\nformat = \"json\"\npositive_scores_only = false\nfilename = \"dictionary-en.json\"\nmetadata_url = \"https://example.invalid/languages\"\ndownload_url = \"https://example.invalid/export\"\nlensisku_created_at = \"2026-10-04T03:07:08Z\"\nsha256 = \"{}\"\ndefinition_count = {}\nentry_count = {}\n",
        "934149f29768993404fcd6529cb7d3dac697d7df8bce55e5a4b694d6987bdee8",
        definitions,
        entries
    )
    .into_bytes()
}

fn metadata_for(raw_sha256: &str, definitions: u64, entries: u64) -> Vec<u8> {
    format!(
        "language_tag = \"en\"\nlanguage_realname = \"English\"\nsource_language_tag = \"jbo\"\nformat = \"json\"\npositive_scores_only = false\nfilename = \"dictionary-en.json\"\nmetadata_url = \"https://example.invalid/languages\"\ndownload_url = \"https://example.invalid/export\"\nlensisku_created_at = \"2026-10-04T03:07:08Z\"\nsha256 = \"{}\"\ndefinition_count = {}\nentry_count = {}\n",
        raw_sha256, definitions, entries
    ).into_bytes()
}

fn invoke(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn import_check_are_pinned_non_destructive_and_structured_on_failure() {
    let dir = scratch();
    let raw = br#"[{"word":"farvi","word_type":"gismu","definition":"to vary","score":-1}]"#;
    let metadata_bytes = metadata(1, 1);
    let raw_path = dir.join("raw.json");
    let meta_path = dir.join("metadata.ini");
    let release_path = dir.join("release.json");
    fs::write(&raw_path, raw).unwrap();
    fs::write(&meta_path, &metadata_bytes).unwrap();

    let imported = invoke(&[
        "jbotci-import",
        raw_path.to_str().unwrap(),
        meta_path.to_str().unwrap(),
        release_path.to_str().unwrap(),
        "--revision",
        "0000000000000000000000000000000000000000",
        "--raw-sha256",
        "934149f29768993404fcd6529cb7d3dac697d7df8bce55e5a4b694d6987bdee8",
        "--metadata-sha256",
        "60285fe479cadba56ac987a9e1b6645ed788f1212442e4d23f2a8a947a40f9ec",
    ]);
    assert!(
        imported.status.success(),
        "{}",
        String::from_utf8_lossy(&imported.stderr)
    );
    let summary: serde_json::Value = serde_json::from_slice(&imported.stdout).unwrap();
    assert_eq!(summary["report"]["input_rows"], 1);
    assert_eq!(summary["report"]["lexical_occurrences"], 1);
    assert!(release_path.exists());

    let check = invoke(&[
        "jbotci-check",
        raw_path.to_str().unwrap(),
        meta_path.to_str().unwrap(),
        "--release",
        release_path.to_str().unwrap(),
        "--revision",
        "0000000000000000000000000000000000000000",
        "--raw-sha256",
        "934149f29768993404fcd6529cb7d3dac697d7df8bce55e5a4b694d6987bdee8",
        "--metadata-sha256",
        "60285fe479cadba56ac987a9e1b6645ed788f1212442e4d23f2a8a947a40f9ec",
    ]);
    assert!(
        check.status.success(),
        "{}",
        String::from_utf8_lossy(&check.stderr)
    );
    let checked: serde_json::Value = serde_json::from_slice(&check.stdout).unwrap();
    assert_eq!(checked["release_checked"], true);

    let changed_revision = invoke(&[
        "jbotci-check",
        raw_path.to_str().unwrap(),
        meta_path.to_str().unwrap(),
        "--release",
        release_path.to_str().unwrap(),
        "--revision",
        "1111111111111111111111111111111111111111",
        "--raw-sha256",
        "934149f29768993404fcd6529cb7d3dac697d7df8bce55e5a4b694d6987bdee8",
        "--metadata-sha256",
        "60285fe479cadba56ac987a9e1b6645ed788f1212442e4d23f2a8a947a40f9ec",
    ]);
    assert!(!changed_revision.status.success());
    assert!(changed_revision.stdout.is_empty());
    let failure: serde_json::Value = serde_json::from_slice(&changed_revision.stderr).unwrap();
    assert_eq!(failure["error"]["code"], "jbotci_release_mismatch");

    let before = fs::read(&release_path).unwrap();
    let overwrite = invoke(&[
        "jbotci-import",
        raw_path.to_str().unwrap(),
        meta_path.to_str().unwrap(),
        release_path.to_str().unwrap(),
        "--revision",
        "0000000000000000000000000000000000000000",
        "--raw-sha256",
        "934149f29768993404fcd6529cb7d3dac697d7df8bce55e5a4b694d6987bdee8",
        "--metadata-sha256",
        "60285fe479cadba56ac987a9e1b6645ed788f1212442e4d23f2a8a947a40f9ec",
    ]);
    assert!(!overwrite.status.success());
    assert!(overwrite.stdout.is_empty());
    assert_eq!(fs::read(&release_path).unwrap(), before);

    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        let referent = dir.join("referent.json");
        let link = dir.join("occupied-link.json");
        fs::write(&referent, b"preserve me").unwrap();
        symlink(&referent, &link).unwrap();
        let occupied_link = invoke(&[
            "jbotci-import",
            raw_path.to_str().unwrap(),
            meta_path.to_str().unwrap(),
            link.to_str().unwrap(),
            "--revision",
            "0000000000000000000000000000000000000000",
            "--raw-sha256",
            "934149f29768993404fcd6529cb7d3dac697d7df8bce55e5a4b694d6987bdee8",
            "--metadata-sha256",
            "60285fe479cadba56ac987a9e1b6645ed788f1212442e4d23f2a8a947a40f9ec",
        ]);
        assert!(!occupied_link.status.success());
        assert!(occupied_link.stdout.is_empty());
        let failure: serde_json::Value = serde_json::from_slice(&occupied_link.stderr).unwrap();
        assert_eq!(failure["error"]["code"], "io");
        assert!(
            fs::symlink_metadata(&link)
                .unwrap()
                .file_type()
                .is_symlink()
        );
        assert_eq!(fs::read(&referent).unwrap(), b"preserve me");
    }
    let occupied_dir = dir.join("occupied-directory");
    fs::create_dir(&occupied_dir).unwrap();
    let directory_output = invoke(&[
        "jbotci-import",
        raw_path.to_str().unwrap(),
        meta_path.to_str().unwrap(),
        occupied_dir.to_str().unwrap(),
        "--revision",
        "0000000000000000000000000000000000000000",
        "--raw-sha256",
        "934149f29768993404fcd6529cb7d3dac697d7df8bce55e5a4b694d6987bdee8",
        "--metadata-sha256",
        "60285fe479cadba56ac987a9e1b6645ed788f1212442e4d23f2a8a947a40f9ec",
    ]);
    assert!(!directory_output.status.success());
    assert!(directory_output.stdout.is_empty());
    let failure: serde_json::Value = serde_json::from_slice(&directory_output.stderr).unwrap();
    assert_eq!(failure["error"]["code"], "io");
    assert!(occupied_dir.is_dir());
    assert_eq!(fs::read_dir(&occupied_dir).unwrap().count(), 0);

    let shape_raw = br#"[{"word":"one","word_type":"gismu","definition":"ok","selmaho":true,"definition_id":{}},{"word":"two","word_type":"gismu","definition":"ok","selmaho":{},"definition_id":123456789012345678901234567890}]"#;
    let shape_meta = metadata_for(
        "6d90a0fc2127488101718837a89362f3549765f381fb89f3c0c2bfb35787fa0d",
        2,
        2,
    );
    let shape_raw_path = dir.join("shape-raw.json");
    let shape_meta_path = dir.join("shape-metadata.toml");
    let shape_release_path = dir.join("shape-release.json");
    fs::write(&shape_raw_path, shape_raw).unwrap();
    fs::write(&shape_meta_path, &shape_meta).unwrap();
    let shapes = invoke(&[
        "jbotci-import",
        shape_raw_path.to_str().unwrap(),
        shape_meta_path.to_str().unwrap(),
        shape_release_path.to_str().unwrap(),
        "--revision",
        "0000000000000000000000000000000000000000",
        "--raw-sha256",
        "6d90a0fc2127488101718837a89362f3549765f381fb89f3c0c2bfb35787fa0d",
        "--metadata-sha256",
        "5236d70066884f1b76faea95a33bc42ac668f5fa5a30efadf235e22cba87830d",
    ]);
    assert!(
        shapes.status.success(),
        "{}",
        String::from_utf8_lossy(&shapes.stderr)
    );
    let summary: serde_json::Value = serde_json::from_slice(&shapes.stdout).unwrap();
    assert_eq!(summary["report"]["lexical_occurrences"], 2);
    assert_eq!(summary["report"]["diagnostics"]["invalid_selmaho"], 2);
    assert_eq!(summary["report"]["diagnostics"]["invalid_definition_id"], 1);

    let malformed_metadata = b"format = true junk\n";
    let malformed_path = dir.join("malformed-metadata.toml");
    let malformed_output = dir.join("must-not-exist.json");
    fs::write(&malformed_path, malformed_metadata).unwrap();
    let malformed = invoke(&[
        "jbotci-import",
        raw_path.to_str().unwrap(),
        malformed_path.to_str().unwrap(),
        malformed_output.to_str().unwrap(),
        "--revision",
        "0000000000000000000000000000000000000000",
        "--raw-sha256",
        "934149f29768993404fcd6529cb7d3dac697d7df8bce55e5a4b694d6987bdee8",
        "--metadata-sha256",
        "5e536af8f9bf5dd10831a8eead4a69696c9f9c8c05b1e747a133eed1d738a172",
    ]);
    assert!(!malformed.status.success());
    assert!(malformed.stdout.is_empty());
    let failure: serde_json::Value = serde_json::from_slice(&malformed.stderr).unwrap();
    assert_eq!(failure["error"]["code"], "jbotci_metadata");
    assert!(!malformed_output.exists());

    let bad_pin = invoke(&[
        "jbotci-check",
        raw_path.to_str().unwrap(),
        meta_path.to_str().unwrap(),
        "--raw-sha256",
        &"0".repeat(64),
        "--metadata-sha256",
        "60285fe479cadba56ac987a9e1b6645ed788f1212442e4d23f2a8a947a40f9ec",
    ]);
    assert!(!bad_pin.status.success());
    assert!(bad_pin.stdout.is_empty());
    let failure: serde_json::Value = serde_json::from_slice(&bad_pin.stderr).unwrap();
    assert_eq!(failure["error"]["code"], "jbotci_hash_mismatch");

    let bad_options = invoke(&[
        "jbotci-check",
        raw_path.to_str().unwrap(),
        meta_path.to_str().unwrap(),
        "--revision",
        "not-a-40-character-revision",
    ]);
    assert!(!bad_options.status.success());
    assert!(bad_options.stdout.is_empty());
    let failure: serde_json::Value = serde_json::from_slice(&bad_options.stderr).unwrap();
    assert_eq!(failure["error"]["code"], "jbotci_options");

    let wrong_counts = metadata(0, 0);
    fs::write(&meta_path, &wrong_counts).unwrap();
    let count_check = invoke(&[
        "jbotci-check",
        raw_path.to_str().unwrap(),
        meta_path.to_str().unwrap(),
        "--raw-sha256",
        "934149f29768993404fcd6529cb7d3dac697d7df8bce55e5a4b694d6987bdee8",
        "--metadata-sha256",
        "b0ef63d37a42ddee7d1553514a17f1ddee18ee41eadcfc9567b376e2799de1cf",
    ]);
    assert!(!count_check.status.success());
    assert!(count_check.stdout.is_empty());
    let failure: serde_json::Value = serde_json::from_slice(&count_check.stderr).unwrap();
    assert_eq!(failure["error"]["code"], "jbotci_accounting");
    assert_eq!(fs::read(&release_path).unwrap(), before);

    let missing = invoke(&[
        "jbotci-check",
        dir.join("missing raw.json").to_str().unwrap(),
        meta_path.to_str().unwrap(),
    ]);
    assert!(!missing.status.success());
    assert!(missing.stdout.is_empty());
    let failure: serde_json::Value = serde_json::from_slice(&missing.stderr).unwrap();
    assert_eq!(failure["error"]["code"], "io");
    assert_eq!(fs::read(&release_path).unwrap(), before);
    fs::remove_dir_all(dir).unwrap();
}
