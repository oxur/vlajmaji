//! Explicit grouped-store verification contract.
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use rusqlite::Connection;
use sha2::{Digest, Sha256};
use vlajmaji_lexicon::Dictionary;
use vlajmaji_store::{
    DictionaryStore, DictionaryStoreCheckStatus as Status, DictionaryStoreVerification as Verify,
    DictionaryStoreVerificationCheck as Check,
};

#[path = "../../lexicon/examples/support/dictionary_fixture.rs"]
mod fixture_support;

fn scratch() -> PathBuf {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let path = std::env::temp_dir().join(format!(
        "vlajmaji-verification-{}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
        NEXT.fetch_add(1, Ordering::Relaxed),
    ));
    fs::create_dir(&path).unwrap();
    path
}

fn copied_store(base: &std::path::Path, name: &str) -> PathBuf {
    let origin = base.join(format!("origin-{name}"));
    let store =
        DictionaryStore::create(&origin, &fixture_support::complete_fixture().unwrap()).unwrap();
    let source = store.database_path();
    drop(store);
    let target = base.join(format!("case-{name}"));
    fs::create_dir(&target).unwrap();
    fs::copy(source, target.join("dictionary.sqlite3")).unwrap();
    target
}

#[test]
fn test_valid_store_passes_all_scopes_and_selective_reports_have_one_row() {
    let base = scratch();
    let path = base.join("valid");
    DictionaryStore::create(&path, &fixture_support::complete_fixture().unwrap()).unwrap();
    let all = Verify::verify(&path, Check::All).unwrap();
    assert!(all.ok, "{all:?}");
    assert_eq!(
        all.checks
            .iter()
            .map(|row| row.check.as_str())
            .collect::<Vec<_>>(),
        ["schema", "metadata", "hashes", "model", "fks", "projection"]
    );
    assert!(all.checks.iter().all(|row| row.status == Status::Passed));
    for check in [
        Check::Schema,
        Check::Metadata,
        Check::Hashes,
        Check::Model,
        Check::Fks,
        Check::Projection,
    ] {
        let report = Verify::verify(&path, check).unwrap();
        assert!(report.ok, "{check:?}: {report:?}");
        assert_eq!(report.checks.len(), 1, "{check:?}");
        assert_eq!(report.checks[0].status, Status::Passed, "{check:?}");
    }
    fs::remove_dir_all(base).unwrap();
}

#[test]
fn test_selected_schema_and_fks_run_without_canonical_prerequisites() {
    let base = scratch();
    let path = copied_store(&base, "independent");
    let conn = Connection::open(path.join("dictionary.sqlite3")).unwrap();
    conn.execute("UPDATE metadata SET value='wrong' WHERE key='format'", [])
        .unwrap();
    conn.execute("UPDATE canonical_document SET json=x'7b' WHERE id=1", [])
        .unwrap();
    drop(conn);
    let before = fs::read(path.join("dictionary.sqlite3")).unwrap();
    let schema = Verify::verify(&path, Check::Schema).unwrap();
    assert!(
        schema.ok,
        "schema does not depend on metadata/canonical: {schema:?}"
    );
    assert!(schema.prerequisites.is_empty());
    let fks = Verify::verify(&path, Check::Fks).unwrap();
    assert!(fks.ok, "fks does not depend on metadata/canonical: {fks:?}");
    assert!(fks.prerequisites.is_empty());
    let all = Verify::verify(&path, Check::All).unwrap();
    assert_eq!(all.checks.len(), 6);
    assert_eq!(all.checks[0].status, Status::Passed);
    assert_eq!(all.checks[1].status, Status::Failed);
    assert_eq!(all.checks[4].status, Status::Passed);
    assert_eq!(fs::read(path.join("dictionary.sqlite3")).unwrap(), before);
    fs::remove_dir_all(base).unwrap();
}

#[test]
fn test_all_reports_independent_simultaneous_schema_metadata_and_fk_failures() {
    let base = scratch();
    let path = copied_store(&base, "simultaneous");
    let conn = Connection::open(path.join("dictionary.sqlite3")).unwrap();
    conn.execute("UPDATE metadata SET value='wrong' WHERE key='format'", [])
        .unwrap();
    conn.execute_batch("CREATE TABLE unexpected_extra(value TEXT); PRAGMA foreign_keys=OFF; INSERT INTO source_aliases(ordinal,source_id,text,status) VALUES(900,'absent-source','bad-alias','Active');").unwrap();
    drop(conn);
    let before = fs::read(path.join("dictionary.sqlite3")).unwrap();
    let report = Verify::verify(&path, Check::All).unwrap();
    assert!(!report.ok);
    assert_eq!(report.checks.len(), 6);
    assert_eq!(
        report.checks[0].status,
        Status::Failed,
        "schema attempted despite bad metadata"
    );
    assert_eq!(report.checks[1].status, Status::Failed);
    assert_eq!(
        report.checks[4].status,
        Status::Failed,
        "FK check attempted despite bad metadata"
    );
    assert_eq!(fs::read(path.join("dictionary.sqlite3")).unwrap(), before);
    fs::remove_dir_all(base).unwrap();
}

#[test]
fn test_unparseable_canonical_blocks_dependent_checks_but_model_fails() {
    let base = scratch();
    let path = copied_store(&base, "bad-canonical");
    let bytes = b"{";
    let digest = format!("{:x}", Sha256::digest(bytes));
    let conn = Connection::open(path.join("dictionary.sqlite3")).unwrap();
    conn.execute("UPDATE canonical_document SET json=?1 WHERE id=1", [bytes])
        .unwrap();
    conn.execute(
        "UPDATE metadata SET value=?1 WHERE key='canonical_sha256'",
        [&digest],
    )
    .unwrap();
    drop(conn);
    let report = Verify::verify(&path, Check::All).unwrap();
    assert_eq!(
        report.checks[1].status,
        Status::Blocked,
        "metadata cannot compare to undecodable canonical"
    );
    assert_eq!(
        report.checks[2].status,
        Status::Blocked,
        "hash captures/legacy require typed draft"
    );
    assert_eq!(report.checks[3].status, Status::Failed);
    assert_eq!(report.checks[5].status, Status::Blocked);
    fs::remove_dir_all(base).unwrap();
}

#[test]
fn test_unsupported_header_versions_fail_before_unparseable_canonical_blocks() {
    let base = scratch();
    for (index, key) in ["dictionary_version", "registry_version"]
        .into_iter()
        .enumerate()
    {
        let path = copied_store(&base, key);
        let conn = Connection::open(path.join("dictionary.sqlite3")).unwrap();
        conn.execute("UPDATE canonical_document SET json=x'7b' WHERE id=1", [])
            .unwrap();
        conn.execute("UPDATE metadata SET value='2' WHERE key=?1", [key])
            .unwrap();
        drop(conn);

        let report = Verify::verify(&path, Check::All).unwrap();
        assert!(!report.ok, "case {index} unexpectedly passed: {report:?}");
        assert_eq!(report.checks[1].status, Status::Failed, "{key}: {report:?}");
        assert!(
            report.checks[1]
                .diagnostic
                .contains(&format!("{key} is not supported")),
            "{key}: {report:?}"
        );
        assert_eq!(report.checks[2].status, Status::Failed, "{key}: {report:?}");
        assert_eq!(report.checks[3].status, Status::Failed, "{key}: {report:?}");
        assert_eq!(
            report.checks[5].status,
            Status::Blocked,
            "{key}: {report:?}"
        );
        fs::remove_dir_all(path).unwrap();
    }
    fs::remove_dir_all(base).unwrap();
}

#[test]
fn test_duplicate_json_keys_remain_a_strict_model_failure_and_block_draft_dependents() {
    let base = scratch();
    let path = copied_store(&base, "duplicate-json");
    let conn = Connection::open(path.join("dictionary.sqlite3")).unwrap();
    let mut bytes: Vec<u8> = conn
        .query_row(
            "SELECT json FROM canonical_document WHERE id=1",
            [],
            |row| row.get(0),
        )
        .unwrap();
    drop(conn);
    let text = std::str::from_utf8(&bytes).unwrap();
    let duplicated = text.replacen("\"version\":1", "\"version\":1,\"version\":1", 1);
    assert_ne!(duplicated, text, "the canonical version key must be found");
    assert!(
        Dictionary::from_json_bytes(duplicated.as_bytes()).is_err(),
        "strict public constructor rejects duplicate canonical keys"
    );
    bytes = duplicated.into_bytes();
    let digest = format!("{:x}", Sha256::digest(&bytes));
    let conn = Connection::open(path.join("dictionary.sqlite3")).unwrap();
    conn.execute("UPDATE canonical_document SET json=?1 WHERE id=1", [&bytes])
        .unwrap();
    conn.execute(
        "UPDATE metadata SET value=?1 WHERE key='canonical_sha256'",
        [&digest],
    )
    .unwrap();
    drop(conn);
    let report = Verify::verify(&path, Check::All).unwrap();
    assert_eq!(
        report.checks[1].status,
        Status::Blocked,
        "draft metadata comparison requires a typed draft"
    );
    assert_eq!(
        report.checks[2].status,
        Status::Blocked,
        "hash detail requires a typed draft"
    );
    assert_eq!(
        report.checks[3].status,
        Status::Failed,
        "strict model check rejects duplicate keys"
    );
    assert_eq!(report.checks[5].status, Status::Blocked);
    fs::remove_dir_all(base).unwrap();
}
