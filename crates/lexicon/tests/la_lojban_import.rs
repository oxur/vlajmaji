//! Public contract for the pinned La-Lojban JSON adapter.

use vlajmaji_lexicon::{
    LaLojbanImportError, LaLojbanImportOptions, Presence, RedistributionStatus,
    import_la_lojban_export,
};

fn digest(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut value = String::new();
    for byte in Sha256::digest(bytes) {
        use std::fmt::Write as _;
        let _ = write!(value, "{byte:02x}");
    }
    value
}

fn options(raw: &[u8], revision: &str) -> LaLojbanImportOptions {
    LaLojbanImportOptions {
        revision: revision.into(),
        expected_raw_sha256: digest(raw),
        raw_locator: Some("https://example.invalid/en.json".into()),
        notice_locator: Some("https://example.invalid/README.md".into()),
    }
}

fn fixture() -> Vec<u8> {
    br#"[
      {"word":"coi","word_type":"gismu","definition":"hello","definition_id":7,"notes":"note","jargon":"greeting","user":{"username":"alice"},"extension":{"raw":1}},
      {"word":"coi","word_type":"gismu","definition":"salutation","definition_id":"07","rafsi":"co'i"},
      {"word":"x","word_type":"wiki","definition":"retained but unsupported"},
      {"word":"bad","word_type":"gismu","definition":"duplicate nested key","nested":{"a/b~":{"key":1,"key":2}}}
    ]"#.to_vec()
}

#[test]
fn test_la_import_preserves_rows_fields_attribution_and_exclusions() {
    let raw = fixture();
    let result = import_la_lojban_export(
        &raw,
        &options(&raw, "0000000000000000000000000000000000000000"),
    )
    .expect("fixture is a valid pinned-shape export");
    let report = result.report();
    assert_eq!(report.version, 1);
    assert_eq!(report.raw_bytes, raw.len() as u64);
    assert_eq!(report.input_rows, 4);
    assert_eq!(report.native_fields, 20);
    assert_eq!(report.normalized_assertions, 4);
    assert_eq!(report.lexical_occurrences, 2);
    assert_eq!(report.excluded_occurrences, 2);
    assert_eq!(report.eligible_spellings, 1);
    assert_eq!(report.all_spellings, 3);
    assert_eq!(report.contributors, 1);
    assert_eq!(report.word_types.get("gismu"), Some(&3));
    assert_eq!(report.word_types.get("wiki"), Some(&1));
    assert_eq!(report.diagnostics.get("duplicate_key_path"), Some(&1));
    assert!(
        result.dictionary().metadata().releases[0]
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "duplicate_key_path"
                && diagnostic.message.contains("/3/nested/a~1b~0/key"))
    );
    assert_eq!(result.dictionary().captures()[0].bytes, raw);
    let rows = &result.dictionary().captures()[0].occurrences;
    assert_eq!(rows.len(), 4);
    assert_eq!(rows[0].fields_named("word").len(), 1);
    assert_eq!(
        rows[0].fields_named("extension")[0].raw_value,
        r#"{"raw":1}"#
    );
    assert_eq!(
        rows[3].fields_named("nested")[0].raw_value,
        r#"{"a/b~":{"key":1,"key":2}}"#
    );
    assert_eq!(result.dictionary().metadata().entries.len(), 4);
    assert_eq!(result.dictionary().metadata().assertions.len(), 4);
    assert!(
        result.dictionary().metadata().entries[0]
            .id
            .as_str()
            .starts_with("la-lojban-")
    );
    assert_eq!(
        result.dictionary().metadata().releases[0].redistribution,
        RedistributionStatus::Held
    );
    assert_eq!(
        result.dictionary().metadata().releases[0].content_notices[0].license,
        Presence::Missing
    );
    let registry = result.dictionary().registry().document();
    assert_eq!(registry.sources[0].id.as_str(), "lensisku");
    assert_eq!(registry.distributors[0].id.as_str(), "la-lojban");
    assert_eq!(registry.lineage.len(), 0);
    assert_eq!(
        registry.occurrences[0].native_assertion_id.as_deref(),
        Some("7")
    );
    assert_eq!(
        registry.occurrences[1].native_assertion_id.as_deref(),
        Some(r#""07""#)
    );
    assert!(
        registry
            .occurrences
            .iter()
            .all(|row| row.native_entry_id.is_none())
    );
    assert_eq!(
        result
            .dictionary()
            .metadata()
            .extensions
            .iter()
            .filter(|e| e.name.contains("metadata"))
            .count(),
        0
    );
}

#[test]
fn test_la_import_revision_changes_release_but_not_capture_or_occurrence_identity() {
    let raw = fixture();
    let first = import_la_lojban_export(
        &raw,
        &options(&raw, "0000000000000000000000000000000000000000"),
    )
    .unwrap();
    let second_options = LaLojbanImportOptions {
        revision: "1111111111111111111111111111111111111111".into(),
        expected_raw_sha256: digest(&raw),
        ..Default::default()
    };
    let second = import_la_lojban_export(&raw, &second_options).unwrap();
    assert_eq!(
        first.dictionary().captures()[0].identity,
        second.dictionary().captures()[0].identity
    );
    assert_eq!(
        first.dictionary().captures()[0].occurrences,
        second.dictionary().captures()[0].occurrences
    );
    assert_ne!(
        first.dictionary().metadata().releases[0].id,
        second.dictionary().metadata().releases[0].id
    );
    assert_eq!(
        second.dictionary().captures()[0].metadata.revision,
        Presence::Value("1111111111111111111111111111111111111111".into())
    );
    assert_eq!(
        second.dictionary().captures()[0].metadata.locator,
        "https://github.com/La-Lojban/lojban/blob/1111111111111111111111111111111111111111/assets/dumps/en.json"
    );
    assert_eq!(second.dictionary().metadata().releases[0].content_notices[0].locator,
        Presence::Value("https://github.com/La-Lojban/lojban/blob/1111111111111111111111111111111111111111/README.md".into()));
    assert_eq!(
        first.dictionary().captures()[0].metadata.locator,
        "https://example.invalid/en.json"
    );
}

#[test]
fn test_la_import_rejects_mismatched_hash_and_invalid_options() {
    let raw = fixture();
    let mut bad = options(&raw, "0000000000000000000000000000000000000000");
    bad.expected_raw_sha256 = "0".repeat(64);
    assert!(matches!(
        import_la_lojban_export(&raw, &bad),
        Err(LaLojbanImportError::HashMismatch { .. })
    ));
    let mut bad = options(&raw, "0000000000000000000000000000000000000000");
    bad.raw_locator = Some(" ".into());
    assert!(matches!(
        import_la_lojban_export(&raw, &bad),
        Err(LaLojbanImportError::InvalidOptions(_))
    ));
    let mut bad = options(&raw, "0000000000000000000000000000000000000000");
    bad.revision = "not-a-revision".into();
    assert!(matches!(
        import_la_lojban_export(&raw, &bad),
        Err(LaLojbanImportError::InvalidOptions(_))
    ));
}

#[test]
fn test_la_import_rejects_invalid_utf8_and_malformed_json() {
    let raw = b"\xff";
    let mut opts = options(raw, "0000000000000000000000000000000000000000");
    assert!(matches!(
        import_la_lojban_export(raw, &opts),
        Err(LaLojbanImportError::InvalidUtf8)
    ));
    let raw = b"{";
    opts.expected_raw_sha256 = digest(raw);
    assert!(matches!(
        import_la_lojban_export(raw, &opts),
        Err(LaLojbanImportError::InvalidJson(_))
    ));
}
