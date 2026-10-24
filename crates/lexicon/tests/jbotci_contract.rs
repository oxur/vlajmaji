//! Public contract checks for the pinned jbotci adapter.

use vlajmaji_lexicon::{
    Eligibility, JbotciImportError, JbotciImportOptions, Presence, import_jbotci_export,
};

fn digest(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut out = String::new();
    for byte in Sha256::digest(bytes) {
        use std::fmt::Write as _;
        let _ = write!(out, "{byte:02x}");
    }
    out
}

fn metadata(raw: &[u8], entry_count: u64) -> Vec<u8> {
    metadata_with_row_count(raw, count_rows(raw), entry_count)
}

fn metadata_with_row_count(raw: &[u8], row_count: u64, entry_count: u64) -> Vec<u8> {
    format!(
        "language_tag = \"en\"\nlanguage_realname = \"English\"\nsource_language_tag = \"jbo\"\nformat = \"json\"\npositive_scores_only = false\nfilename = \"dictionary-en.json\"\nmetadata_url = \"https://example.invalid/languages\"\ndownload_url = \"https://example.invalid/export\"\nlensisku_created_at = \"2026-10-04T03:07:08Z\"\nsha256 = \"{}\"\ndefinition_count = {}\nentry_count = {}\n",
        digest(raw), row_count, entry_count,
    )
    .into_bytes()
}

fn count_rows(raw: &[u8]) -> u64 {
    serde_json::from_slice::<Vec<serde_json::Value>>(raw)
        .expect("test raw rows")
        .len() as u64
}

fn options(raw: &[u8], metadata: &[u8], revision: &str) -> JbotciImportOptions {
    JbotciImportOptions {
        revision: revision.into(),
        expected_raw_sha256: digest(raw),
        expected_metadata_sha256: digest(metadata),
    }
}

#[test]
fn revision_is_release_provenance_while_occurrence_identity_is_byte_stable() {
    let raw = br#"[{"word":"same","word_type":"gismu","definition":"first","definition_id":7,"score":-1.25,"rafsi":"sma sma","notes":"note"}]"#;
    let metadata = metadata(raw, 1);
    let first_options = options(raw, &metadata, "0000000000000000000000000000000000000000");
    let second_options = options(raw, &metadata, "1111111111111111111111111111111111111111");

    let first = import_jbotci_export(raw, &metadata, &first_options).expect("valid first import");
    let repeated =
        import_jbotci_export(raw, &metadata, &first_options).expect("valid repeated import");
    let second_revision = import_jbotci_export(raw, &metadata, &second_options)
        .expect("valid provenance counterpart");

    assert_eq!(first.report(), repeated.report());
    assert_eq!(
        first.dictionary().to_json_bytes().unwrap(),
        repeated.dictionary().to_json_bytes().unwrap()
    );
    assert_ne!(
        first.dictionary().metadata().releases[0].id,
        second_revision.dictionary().metadata().releases[0].id
    );
    assert_ne!(
        first.dictionary().to_json_bytes().unwrap(),
        second_revision.dictionary().to_json_bytes().unwrap()
    );
    assert_eq!(second_revision.report().revision, second_options.revision);
    assert_eq!(
        second_revision.dictionary().metadata().releases[0].distributor_revision,
        Presence::Value(second_options.revision.clone())
    );
    assert_eq!(
        first.dictionary().captures()[0].metadata.revision,
        Presence::Value(first_options.revision.clone())
    );
    assert_eq!(
        second_revision.dictionary().captures()[0].metadata.revision,
        Presence::Value(second_options.revision.clone())
    );
    assert_eq!(
        second_revision.dictionary().metadata().releases[0].accounting,
        first.dictionary().metadata().releases[0].accounting
    );
    assert_eq!(
        first
            .dictionary()
            .captures()
            .iter()
            .map(|c| c.identity.clone())
            .collect::<Vec<_>>(),
        second_revision
            .dictionary()
            .captures()
            .iter()
            .map(|c| c.identity.clone())
            .collect::<Vec<_>>()
    );
    assert_eq!(
        first.dictionary().metadata().entries,
        second_revision.dictionary().metadata().entries
    );
    assert_eq!(
        first.dictionary().metadata().assertions,
        second_revision.dictionary().metadata().assertions
    );
    let notice = "https://github.com/int19h/jbotci/blob/3d9fd41d9ed32de26311ee27ec86b6b165985aa1/THIRD-PARTY-NOTICES.md#L242-L252";
    let first_lineage = first.dictionary().registry().document();
    let second_lineage = second_revision.dictionary().registry().document();
    assert_eq!(first_lineage.lineage[0].evidence.as_deref(), Some(notice));
    assert_eq!(second_lineage.lineage[0].evidence.as_deref(), Some(notice));
    assert_eq!(
        first.dictionary().captures()[0].metadata.notice_locator,
        Presence::Value(notice.into())
    );
    assert_eq!(
        second_revision.dictionary().captures()[0]
            .metadata
            .notice_locator,
        Presence::Value(notice.into())
    );
    assert_eq!(
        first.dictionary().metadata().releases[0].content_notices[0].locator,
        Presence::Value(notice.into())
    );
    assert_eq!(
        second_revision.dictionary().metadata().releases[0].content_notices[0].locator,
        Presence::Value(notice.into())
    );
    assert_eq!(
        first.dictionary().metadata().code_notices[0].locator,
        Presence::Value("LICENSE".into())
    );
    assert_eq!(
        second_revision.dictionary().metadata().code_notices[0].locator,
        Presence::Value("LICENSE".into())
    );
    assert_eq!(
        first.dictionary().metadata().contexts,
        second_revision.dictionary().metadata().contexts
    );
    assert_eq!(
        first.dictionary().metadata().forms,
        second_revision.dictionary().metadata().forms
    );
    assert_eq!(
        first.dictionary().metadata().extensions[0].name,
        second_revision.dictionary().metadata().extensions[0].name
    );
    assert_eq!(
        first.dictionary().metadata().extensions[0].value,
        second_revision.dictionary().metadata().extensions[0].value
    );
    let first_registry = first.dictionary().registry().document();
    let second_registry = second_revision.dictionary().registry().document();
    assert_eq!(first_registry.occurrences, second_registry.occurrences);
    assert_eq!(first_registry.snapshots, second_registry.snapshots);
    assert_eq!(first_registry.contributors, second_registry.contributors);
    let release = &first.dictionary().metadata().releases[0];
    assert_eq!(release.upstream_revision, Presence::Missing);
    assert_eq!(
        release.redistribution,
        vlajmaji_lexicon::RedistributionStatus::Held
    );
    assert!(matches!(
        &release.content_notices[0].license,
        Presence::Value(value) if value.contains("supplier-claimed")
    ));
    assert_eq!(
        first.dictionary().metadata().code_notices[0].license,
        Presence::Value("Apache-2.0".into())
    );
    assert!(first.dictionary().metadata().frames.is_empty());
    assert!(first.dictionary().metadata().relations.is_empty());
    assert!(first.dictionary().metadata().owned_revisions.is_empty());
    assert!(first.dictionary().metadata().owned_heads.is_empty());
    assert!(first.dictionary().metadata().editorial_senses.is_empty());
    assert!(first.dictionary().metadata().computations.is_empty());
    assert!(first.dictionary().metadata().annotations.is_empty());
    assert_eq!(first.report().input_rows, 1);
    assert_eq!(first.report().normalized_assertions, 1);
    assert_eq!(first.report().lexical_occurrences, 1);
    assert_eq!(first.report().excluded_occurrences, 0);
    assert_eq!(first.report().eligible_spellings, 1);
    assert_eq!(
        first.dictionary().metadata().entries[0].id.as_str(),
        format!("jbotci-{}-entry-0", digest(raw))
    );
}

#[test]
fn duplicate_fields_nested_values_and_nonobjects_remain_raw_and_excluded() {
    let raw = br#"[{"word":"one","word":"two","word_type":"gismu","word_type":"lujvo","definition":"a","definition":"b","opaque":{"huge":1234567890123456789012345678901234567890},"opaque":[true]}, {"word":"nested","word_type":"gismu","definition":"ok","unknown":{"x":1,"x":2}}, 17]"#;
    let metadata = metadata(raw, 0);
    let opts = options(raw, &metadata, "0000000000000000000000000000000000000000");
    let imported =
        import_jbotci_export(raw, &metadata, &opts).expect("valid JSON remains importable");
    let dict = imported.dictionary();
    assert_eq!(imported.report().input_rows, 3);
    assert_eq!(imported.report().normalized_assertions, 3);
    assert_eq!(imported.report().excluded_occurrences, 3);
    assert_eq!(imported.report().diagnostics.get("duplicate_key"), Some(&5));
    assert_eq!(
        imported.report().diagnostics.get("unknown_native_field"),
        Some(&3)
    );
    assert_eq!(imported.report().word_types.get("gismu"), Some(&2));
    assert_eq!(imported.report().word_types.get("lujvo"), Some(&1));
    assert_eq!(
        dict.captures()[0].occurrences[0].fields_named("word").len(),
        2
    );
    let extensions = &dict.metadata().extensions;
    let duplicate_extensions: Vec<_> = extensions
        .iter()
        .filter(|extension| {
            extension
                .evidence
                .as_ref()
                .is_some_and(|e| e.occurrence.locator == "/0")
        })
        .collect();
    assert_eq!(duplicate_extensions.len(), 2);
    assert_eq!(
        duplicate_extensions[0].name,
        "lensisku.native_field.6.opaque"
    );
    assert_eq!(
        duplicate_extensions[0].value.as_str(),
        r#"{"huge":1234567890123456789012345678901234567890}"#
    );
    assert_eq!(
        duplicate_extensions[0]
            .evidence
            .as_ref()
            .unwrap()
            .field_index,
        Some(6)
    );
    assert_eq!(
        duplicate_extensions[0].owner,
        dict.metadata().assertions[0].id
    );
    assert_eq!(
        duplicate_extensions[0]
            .evidence
            .as_ref()
            .unwrap()
            .occurrence
            .locator,
        "/0"
    );
    assert_eq!(
        duplicate_extensions[1].name,
        "lensisku.native_field.7.opaque"
    );
    assert_eq!(duplicate_extensions[1].value.as_str(), "[true]");
    assert_eq!(
        duplicate_extensions[1]
            .evidence
            .as_ref()
            .unwrap()
            .field_index,
        Some(7)
    );
    assert_eq!(
        dict.captures()[0].occurrences[0]
            .fields_named("definition")
            .len(),
        2
    );
    assert_eq!(
        dict.captures()[0].occurrences[0].fields_named("opaque")[0].raw_value,
        r#"{"huge":1234567890123456789012345678901234567890}"#
    );
    assert_eq!(dict.captures()[0].occurrences[2].fields[0].name, "$row");
    assert!(matches!(
        dict.metadata().assertions[0].eligibility,
        Eligibility::Excluded { .. }
    ));
    assert_eq!(dict.metadata().entries[0].spelling, Presence::Missing);
}

#[test]
fn repeated_native_ids_and_spellings_stay_distinct_and_negative_scores_are_not_filters() {
    let raw = br#"[{"word":"same","word_type":"gismu","definition":"one","definition_id":9,"score":-2.0},{"word":"same","word_type":"gismu","definition":"two","definition_id":9,"score":-9.5}]"#;
    let metadata = metadata(raw, 1);
    let opts = options(raw, &metadata, "0000000000000000000000000000000000000000");
    let imported = import_jbotci_export(raw, &metadata, &opts).expect("valid repeated source IDs");
    assert_eq!(imported.report().input_rows, 2);
    assert_eq!(imported.report().lexical_occurrences, 2);
    assert_eq!(imported.report().eligible_spellings, 1);
    assert_eq!(
        imported.report().diagnostics.get("entry_count_mismatch"),
        None
    );
    let assertions = &imported.dictionary().metadata().assertions;
    assert_ne!(assertions[0].id, assertions[1].id);
    assert_ne!(assertions[0].occurrence, assertions[1].occurrence);
    assert_eq!(
        assertions[0].source_score,
        Presence::Value(vlajmaji_lexicon::RawJson::new("-2.0").unwrap())
    );
    assert_eq!(
        assertions[1].source_score,
        Presence::Value(vlajmaji_lexicon::RawJson::new("-9.5").unwrap())
    );
}

#[test]
fn changed_pinned_raw_bytes_change_capture_and_row_ids_and_wrong_hash_fails() {
    let raw_a = br#"[{"word":"a","word_type":"gismu","definition":"one"}]"#;
    let raw_b = br#"[{"word":"a","word_type":"gismu","definition":"two"}]"#;
    let metadata_a = metadata(raw_a, 1);
    let metadata_b = metadata(raw_b, 1);
    let opts_a = options(
        raw_a,
        &metadata_a,
        "0000000000000000000000000000000000000000",
    );
    let opts_b = options(
        raw_b,
        &metadata_b,
        "0000000000000000000000000000000000000000",
    );
    let a = import_jbotci_export(raw_a, &metadata_a, &opts_a).unwrap();
    let b = import_jbotci_export(raw_b, &metadata_b, &opts_b).unwrap();
    assert_ne!(
        a.dictionary().captures()[0].identity,
        b.dictionary().captures()[0].identity
    );
    assert_ne!(
        a.dictionary().metadata().entries[0].id,
        b.dictionary().metadata().entries[0].id
    );
    assert_ne!(
        a.dictionary().metadata().assertions[0].id,
        b.dictionary().metadata().assertions[0].id
    );

    let bad = JbotciImportOptions {
        expected_raw_sha256: "0".repeat(64),
        ..opts_a
    };
    assert!(
        matches!(import_jbotci_export(raw_a, &metadata_a, &bad), Err(JbotciImportError::HashMismatch { input, .. }) if input == "raw")
    );
}

#[test]
fn metadata_duplicates_and_unsupported_shapes_are_typed_failures() {
    let raw = br#"[]"#;
    let mut meta = metadata(raw, 0);
    meta.extend_from_slice(b"entry_count = 0\n");
    let opts = options(raw, &meta, "0000000000000000000000000000000000000000");
    assert!(matches!(
        import_jbotci_export(raw, &meta, &opts),
        Err(JbotciImportError::InvalidMetadata(_))
    ));

    let meta = b"language_tag = 'en'\n".to_vec();
    let opts = options(raw, &meta, "0000000000000000000000000000000000000000");
    assert!(matches!(
        import_jbotci_export(raw, &meta, &opts),
        Err(JbotciImportError::UnsupportedMetadata(_))
    ));

    for malformed in [
        b"language_tag = true junk\n".as_slice(),
        b"language_tag = false junk\n",
        b"language_tag = 12 junk\n",
        b"language_tag = \"ok\" junk\n",
        b"language_tag = \n",
        b"= true\n",
        b"language_tag = 18446744073709551616\n",
    ] {
        let opts = options(raw, malformed, "0000000000000000000000000000000000000000");
        assert!(
            matches!(
                import_jbotci_export(raw, malformed, &opts),
                Err(JbotciImportError::InvalidMetadata(_))
            ),
            "{malformed:?}"
        );
    }
    for unsupported in [
        b"'quoted key' = \"value\"\n".as_slice(),
        b"language_tag = 'en'\n",
    ] {
        let opts = options(raw, unsupported, "0000000000000000000000000000000000000000");
        assert!(
            matches!(
                import_jbotci_export(raw, unsupported, &opts),
                Err(JbotciImportError::UnsupportedMetadata(_))
            ),
            "{unsupported:?}"
        );
    }
}

#[test]
fn metadata_numeric_values_follow_json_validity_before_subset_errors() {
    let raw = br#"[]"#;
    let revision = "0000000000000000000000000000000000000000";

    for literal in ["1.5", "1e3", "1e9999", "-1.5"] {
        let mut meta = metadata(raw, 0);
        meta.extend_from_slice(format!("unknown_numeric = {literal}\n").as_bytes());
        let opts = options(raw, &meta, revision);
        assert!(
            matches!(
                import_jbotci_export(raw, &meta, &opts),
                Err(JbotciImportError::UnsupportedMetadata(_))
            ),
            "{literal}"
        );
    }

    let mut integer_meta = metadata(raw, 0);
    integer_meta.extend_from_slice(b"unknown_integer = 12\n");
    let integer_opts = options(raw, &integer_meta, revision);
    let integer_import = import_jbotci_export(raw, &integer_meta, &integer_opts)
        .expect("unknown simple unsigned integer remains supported metadata");
    assert_eq!(
        integer_import
            .report()
            .diagnostics
            .get("metadata_unknown_key"),
        Some(&1)
    );
    let metadata_capture = integer_import
        .dictionary()
        .captures()
        .iter()
        .find(|capture| capture.identity.namespace == "captured:lensisku:jbotci:en:metadata")
        .expect("exact metadata capture");
    assert_eq!(metadata_capture.bytes, integer_meta);

    for assignment in [
        "unknown_numeric = 12 junk\n",
        "unknown_numeric = 1.5 junk\n",
        "unknown_numeric = true junk\n",
        "unknown_numeric = false junk\n",
        "unknown_numeric = \"ok\" junk\n",
        " = true\n",
        "unknown_numeric = \n",
        "unknown_numeric = 18446744073709551616\n",
    ] {
        let mut meta = metadata(raw, 0);
        meta.extend_from_slice(assignment.as_bytes());
        let opts = options(raw, &meta, revision);
        assert!(
            matches!(
                import_jbotci_export(raw, &meta, &opts),
                Err(JbotciImportError::InvalidMetadata(_))
            ),
            "{assignment:?}"
        );
    }
}

#[test]
fn invalid_pins_utf8_and_json_root_return_typed_failures() {
    let raw = br#"[]"#;
    let meta = metadata(raw, 0);
    let mut opts = options(raw, &meta, "0000000000000000000000000000000000000000");
    opts.revision = "bad".into();
    assert!(matches!(
        import_jbotci_export(raw, &meta, &opts),
        Err(JbotciImportError::InvalidOptions(_))
    ));

    let invalid_utf8 = [0xff];
    let meta = metadata_with_row_count(&invalid_utf8, 0, 0);
    let opts = options(
        &invalid_utf8,
        &meta,
        "0000000000000000000000000000000000000000",
    );
    assert!(matches!(
        import_jbotci_export(&invalid_utf8, &meta, &opts),
        Err(JbotciImportError::InvalidUtf8 { input }) if input == "raw"
    ));

    let raw = br#"{"not":"array"}"#;
    let meta = metadata_with_row_count(raw, 0, 0);
    let opts = options(raw, &meta, "0000000000000000000000000000000000000000");
    assert!(matches!(
        import_jbotci_export(raw, &meta, &opts),
        Err(JbotciImportError::InvalidJson(_))
    ));
}

#[test]
fn missing_null_and_empty_critical_values_remain_distinguishable() {
    let raw = br#"[{"word":null,"word_type":"gismu","definition":null},{"word":"","word_type":"gismu","definition":""},{"word":"present","word_type":"gismu"}]"#;
    let meta = metadata(raw, 0);
    let opts = options(raw, &meta, "0000000000000000000000000000000000000000");
    let imported = import_jbotci_export(raw, &meta, &opts).unwrap();
    let entries = &imported.dictionary().metadata().entries;
    let assertions = &imported.dictionary().metadata().assertions;
    assert_eq!(entries[0].spelling, Presence::Null);
    assert_eq!(entries[1].spelling, Presence::Value(String::new()));
    assert_eq!(entries[2].spelling, Presence::Value("present".into()));
    assert_eq!(assertions[0].definition, Presence::Null);
    assert!(matches!(&assertions[1].definition, Presence::Value(value) if value.text.is_empty()));
    assert_eq!(assertions[2].definition, Presence::Missing);
    assert_eq!(imported.report().lexical_occurrences, 0);
    assert_eq!(imported.report().excluded_occurrences, 3);
}

#[test]
fn malformed_optional_shapes_are_diagnostic_without_excluding_valid_rows() {
    let raw = br#"[{"word":"ok","word_type":"gismu","definition":"valid","notes":7,"jargon":true,"gloss_keywords":[{"word":7},null],"place_keywords":false,"rafsi":17,"experimental_rafsi":["ab","four"],"etymology":{},"user":{"username":9},"future":{"x":1}}]"#;
    let meta = metadata(raw, 1);
    let opts = options(raw, &meta, "0000000000000000000000000000000000000000");
    let imported = import_jbotci_export(raw, &meta, &opts).unwrap();
    assert_eq!(imported.report().lexical_occurrences, 1);
    assert_eq!(imported.report().excluded_occurrences, 0);
    for key in [
        "invalid_notes",
        "invalid_jargon",
        "invalid_gloss_object",
        "invalid_place_keywords",
        "invalid_reported_form",
        "invalid_etymology",
        "invalid_user",
        "unknown_native_field",
    ] {
        let expected = if key == "invalid_gloss_object" { 2 } else { 1 };
        assert_eq!(
            imported.report().diagnostics.get(key),
            Some(&expected),
            "{key}"
        );
    }
    let occurrence = &imported.dictionary().captures()[0].occurrences[0];
    assert_eq!(
        occurrence.fields_named("place_keywords")[0].raw_value,
        "false"
    );
    assert_eq!(occurrence.fields_named("future")[0].raw_value, r#"{"x":1}"#);
}

#[test]
fn optional_selmaho_and_definition_id_shapes_are_diagnostic_without_loss() {
    let raw = br#"[{"word":"missing","word_type":"gismu","definition":"ok"},{"word":"null","word_type":"gismu","definition":"ok","selmaho":null,"definition_id":null},{"word":"empty","word_type":"gismu","definition":"ok","selmaho":"","definition_id":""},{"word":"numeric","word_type":"gismu","definition":"ok","definition_id":123456789012345678901234567890},{"word":"bad1","word_type":"gismu","definition":"ok","selmaho":{}},{"word":"bad2","word_type":"gismu","definition":"ok","selmaho":[],"definition_id":true},{"word":"bad3","word_type":"gismu","definition":"ok","definition_id":[]} ]"#;
    let meta = metadata(raw, 7);
    let opts = options(raw, &meta, "0000000000000000000000000000000000000000");
    let imported = import_jbotci_export(raw, &meta, &opts).unwrap();
    assert_eq!(imported.report().lexical_occurrences, 7);
    assert_eq!(imported.report().excluded_occurrences, 0);
    assert_eq!(
        imported.report().diagnostics.get("invalid_selmaho"),
        Some(&2)
    );
    assert_eq!(
        imported.report().diagnostics.get("invalid_definition_id"),
        Some(&2)
    );
    let entries = &imported.dictionary().metadata().entries;
    assert_eq!(entries[0].selmaho, Presence::Missing);
    assert_eq!(entries[1].selmaho, Presence::Null);
    assert_eq!(entries[2].selmaho, Presence::Value(String::new()));
    assert_eq!(entries[3].selmaho, Presence::Missing);
    assert_eq!(
        imported.dictionary().captures()[0].occurrences[3].fields_named("definition_id")[0]
            .raw_value,
        "123456789012345678901234567890"
    );
    let registry_rows = imported.dictionary().registry().document().occurrences;
    assert_eq!(
        registry_rows[3].native_assertion_id.as_deref(),
        Some("123456789012345678901234567890")
    );
    assert_eq!(registry_rows[4].native_assertion_id, None);
    assert_eq!(registry_rows[5].native_assertion_id, None);
    assert_eq!(registry_rows[6].native_assertion_id, None);
    assert_eq!(
        imported.dictionary().metadata().releases[0]
            .diagnostics
            .iter()
            .filter(|d| d.code == "invalid_selmaho")
            .count(),
        2
    );
    assert_eq!(
        imported.dictionary().metadata().releases[0]
            .diagnostics
            .iter()
            .filter(|d| d.code == "invalid_definition_id")
            .count(),
        2
    );
}

#[test]
fn non_display_username_keeps_hashed_attribution_with_a_diagnostic() {
    let raw =
        br#"[{"word":"ok","word_type":"gismu","definition":"valid","user":{"username":" bad "}}]"#;
    let meta = metadata(raw, 1);
    let opts = options(raw, &meta, "0000000000000000000000000000000000000000");
    let imported = import_jbotci_export(raw, &meta, &opts).unwrap();
    assert_eq!(imported.report().contributors, 1);
    assert_eq!(
        imported
            .report()
            .diagnostics
            .get("contributor_display_fallback"),
        Some(&1)
    );
    let occurrence = &imported.dictionary().captures()[0].occurrences[0];
    assert_eq!(
        occurrence.fields_named("user")[0].raw_value,
        r#"{"username":" bad "}"#
    );
}

#[test]
fn all_supported_classes_and_cross_column_forms_remain_distinct() {
    let classes = [
        "bu-letteral",
        "cmavo",
        "cmavo-compound",
        "cmevla",
        "experimental cmavo",
        "experimental gismu",
        "fu'ivla",
        "gismu",
        "lujvo",
        "nalvla",
        "obsolete cmavo",
        "obsolete cmevla",
        "obsolete fu'ivla",
        "obsolete zei-lujvo",
        "phrase",
        "zei-lujvo",
    ];
    let mut rows = Vec::new();
    for (index, class) in classes.iter().enumerate() {
        let mut row = serde_json::json!({
            "word": format!("word-{index}"),
            "word_type": class,
            "definition": format!("definition {index}"),
        });
        if index == 0 {
            row["rafsi"] = serde_json::json!(["x", "", "four", "four"]);
            row["experimental_rafsi"] = serde_json::json!("four extra");
        }
        rows.push(row);
    }
    rows.push(serde_json::json!({
        "word":"wikiword", "word_type":"wiki", "definition":"wiki body",
        "selmaho":"KOhA", "rafsi":"wik"
    }));
    rows.push(serde_json::json!({
        "word":"unknownword", "word_type":"future class", "definition":"future body"
    }));
    let raw = serde_json::to_vec(&rows).unwrap();
    let meta = metadata(&raw, 16);
    let opts = options(&raw, &meta, "0000000000000000000000000000000000000000");
    let imported = import_jbotci_export(&raw, &meta, &opts).unwrap();
    assert_eq!(imported.report().input_rows, 18);
    assert_eq!(imported.report().lexical_occurrences, 16);
    assert_eq!(imported.report().excluded_occurrences, 2);
    assert_eq!(imported.report().eligible_spellings, 16);
    let forms = &imported.dictionary().metadata().forms;
    let repeated_four = forms
        .iter()
        .filter(|form| form.spelling == "four")
        .collect::<Vec<_>>();
    assert_eq!(repeated_four.len(), 3);
    assert_eq!(repeated_four[0].standing, Presence::Value("rafsi".into()));
    assert_eq!(repeated_four[1].standing, Presence::Value("rafsi".into()));
    assert_eq!(
        repeated_four[2].standing,
        Presence::Value("experimental_rafsi".into())
    );
    assert_eq!(forms[0].spelling, "x");
    assert!(forms.iter().any(|form| form.spelling.is_empty()));
    let wiki = &imported.dictionary().metadata().assertions[16];
    assert!(matches!(wiki.eligibility, Eligibility::Excluded { .. }));
    let wiki_entry = &imported.dictionary().metadata().entries[16];
    assert_eq!(wiki_entry.native_word_class, Presence::Value("wiki".into()));
    assert_eq!(
        imported.dictionary().captures()[0].occurrences[16].fields_named("rafsi")[0].raw_value,
        "\"wik\""
    );
}
