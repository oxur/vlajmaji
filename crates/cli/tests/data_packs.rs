//! Process-level contract tests for local pack materialization.

use std::process::Command;

use vlajmaji_lexicon::{PackId, load_sealed_release};

fn scratch(name: &str) -> std::path::PathBuf {
    static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let path = std::env::temp_dir().join(format!(
        "vlajmaji-data-pack-{name}-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    std::fs::create_dir(&path).expect("create test scratch");
    path
}

fn cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
}

#[test]
fn test_synthetic_fixture_uses_the_importer_and_emits_a_loadable_pack() {
    let root = scratch("fixture");
    let output = root.join("pack");
    let result = cli()
        .args(["data-pack", "fixture", "jbotci"])
        .arg(&output)
        .output()
        .expect("run fixture materializer");
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let response: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(response["operation"], "data-pack-materialize");
    assert_eq!(response["payload_kind"], "synthetic");
    assert_eq!(response["report"]["input_rows"], 3);
    assert_eq!(response["report"]["normalized_assertions"], 3);
    assert_eq!(response["report"]["lexical_occurrences"], 2);
    assert_eq!(response["report"]["excluded_occurrences"], 1);
    assert_eq!(response["report"]["eligible_spellings"], 1);
    let release = std::fs::read(output.join("release.json")).unwrap();
    let seal = std::fs::read(output.join("seal.json")).unwrap();
    assert!(output.join("report.json").is_file());
    assert_eq!(
        load_sealed_release(PackId::LensiskuJbotci, &release, &seal)
            .unwrap()
            .metadata()
            .assertions
            .len(),
        3
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn test_occupied_destination_is_preserved_and_bad_pins_create_no_output() {
    let root = scratch("negative");
    let occupied = root.join("occupied");
    std::fs::create_dir(&occupied).unwrap();
    let sentinel = occupied.join("keep.txt");
    std::fs::write(&sentinel, b"untouched").unwrap();
    let occupied_result = cli()
        .args(["data-pack", "fixture", "jbotci"])
        .arg(&occupied)
        .output()
        .unwrap();
    assert!(!occupied_result.status.success());
    let error: serde_json::Value = serde_json::from_slice(&occupied_result.stderr).unwrap();
    assert_eq!(error["error"]["code"], "io");
    assert_eq!(std::fs::read(&sentinel).unwrap(), b"untouched");

    let raw = root.join("wrong-raw.json");
    let metadata = root.join("metadata.toml");
    std::fs::write(&raw, b"[]").unwrap();
    std::fs::write(&metadata, b"not used after raw digest failure").unwrap();
    let missing_output = root.join("must-not-exist");
    let invalid = cli()
        .args(["data-pack", "materialize", "jbotci"])
        .arg(&raw)
        .arg(&metadata)
        .arg(&missing_output)
        .output()
        .unwrap();
    assert!(!invalid.status.success());
    let error: serde_json::Value = serde_json::from_slice(&invalid.stderr).unwrap();
    assert_eq!(error["error"]["code"], "jbotci_hash_mismatch");
    assert!(!missing_output.exists());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn test_llg_synthetic_fixture_uses_importer_and_refuses_occupied_output() {
    let root = scratch("llg-fixture");
    let output = root.join("pack");
    let result = cli()
        .args(["data-pack", "fixture", "llg"])
        .arg(&output)
        .output()
        .expect("run LLG fixture materializer");
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let response: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(response["operation"], "data-pack-materialize");
    assert_eq!(response["pack_id"], "llg");
    assert_eq!(response["payload_kind"], "synthetic");
    assert_eq!(response["report"]["occurrences"], 5);
    assert_eq!(response["report"]["lexical_assertions"], 2);
    assert_eq!(response["report"]["excluded_assertions"], 3);
    assert_eq!(response["report"]["entries"], 4);
    assert_eq!(response["report"]["assigned_forms"], 3);
    let release = std::fs::read(output.join("release.json")).unwrap();
    let seal = std::fs::read(output.join("seal.json")).unwrap();
    assert!(output.join("report.json").is_file());
    let loaded = load_sealed_release(PackId::Llg, &release, &seal).unwrap();
    assert_eq!(loaded.captures().len(), 4);
    assert_eq!(loaded.metadata().assertions.len(), 5);
    assert_eq!(loaded.metadata().entries.len(), 4);
    assert_eq!(loaded.metadata().forms.len(), 3);
    assert_eq!(loaded.metadata().releases.len(), 3);

    let occupied = root.join("occupied");
    std::fs::create_dir(&occupied).unwrap();
    let sentinel = occupied.join("keep.txt");
    std::fs::write(&sentinel, b"untouched").unwrap();
    let refused = cli()
        .args(["data-pack", "fixture", "llg"])
        .arg(&occupied)
        .output()
        .unwrap();
    assert!(!refused.status.success());
    assert_eq!(std::fs::read(&sentinel).unwrap(), b"untouched");

    let bad = root.join("bad-pin-output");
    let gismu = root.join("gismu.txt");
    let cmavo = root.join("cmavo.txt");
    let rafsi = root.join("rafsi.txt");
    let notice = root.join("notice.txt");
    for path in [&gismu, &cmavo, &rafsi, &notice] {
        std::fs::write(path, b"bytes").unwrap();
    }
    let invalid = cli()
        .args(["data-pack", "materialize", "llg"])
        .arg(&gismu)
        .arg(&cmavo)
        .arg(&rafsi)
        .arg(&notice)
        .arg(&bad)
        .output()
        .unwrap();
    assert!(!invalid.status.success());
    let error: serde_json::Value = serde_json::from_slice(&invalid.stderr).unwrap();
    assert_eq!(error["error"]["code"], "llg_hash_mismatch");
    assert!(!bad.exists());
    std::fs::remove_dir_all(root).unwrap();
}
