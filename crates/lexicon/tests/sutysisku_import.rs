//! Literal Sutysisku importer fixture; numeric expectations are added only after
//! the independently retained source-literal census.
const RAW_FIXTURE: &[u8] = br#"{
  "f\u0061rvi":{"d":"to grow","t":"gismu","s":null,"g":["to flourish",null,{"kind":"raw"}],"n":"a note","r":["far","fari"],"b":{"opaque":true},"x/y~z":"extra"},
  "farvi":{"d":"to develop","t":"gismu","g":["to thrive"],"r":["far"],"b":"opaque alternative"},
  "nested":{"nested":{"dup":"one","dup":"two"},"d":"ambiguous","t":"gismu","g":["retained raw"]},
  "scalar":true,
  "empty":{"d":"","t":"gismu","g":["retained despite missing definition"]},
  "unknown-class":{"d":"unmapped","t":"wiki","g":[]},
  "nullable":{"d":"to mark","t":"cmavo","s":null,"g":null,"n":null,"r":null,"b":null},
  "wrong-shapes":{"d":"to denote","t":"cmavo","s":false,"g":17,"n":[],"r":["far",7],"b":[1,2]}
}"#;

use vlajmaji_lexicon::{
    ContextKind, Eligibility, FormKind, Presence, SutysiskuImportError, SutysiskuImportOptions,
    import_sutysisku_export,
};

fn options_for(raw: &[u8]) -> SutysiskuImportOptions {
    let digest = {
        use sha2::{Digest, Sha256};
        Sha256::digest(raw)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect()
    };
    SutysiskuImportOptions {
        expected_raw_sha256: digest,
        ..SutysiskuImportOptions::default()
    }
}

#[test]
fn test_import_sutysisku_export_literal_census_and_raw_projection_are_preserved() {
    let imported = import_sutysisku_export(RAW_FIXTURE, &options_for(RAW_FIXTURE)).unwrap();
    let report = imported.report();
    assert_eq!(report.version, 1);
    assert_eq!(report.input_rows, 8);
    assert_eq!(report.normalized_assertions, 8);
    assert_eq!(report.lexical_occurrences, 4);
    assert_eq!(report.excluded_occurrences, 4);
    assert_eq!(report.eligible_spellings, 3);
    assert_eq!(report.all_spellings, 7);
    assert_eq!(report.root_keys, 8);
    assert_eq!(report.native_child_fields, 37);
    assert_eq!(report.non_object_rows, 1);
    assert_eq!(report.native_fields, 46);
    assert_eq!(report.contexts, 4);
    assert_eq!(report.reported_forms, 3);
    assert_eq!(report.extensions, 13);
    assert_eq!(report.word_types.get("gismu"), Some(&4));
    assert_eq!(report.word_types.get("cmavo"), Some(&2));
    assert_eq!(report.word_types.get("wiki"), Some(&1));
    assert_eq!(report.field_counts.get("$key"), Some(&8));
    assert_eq!(report.field_counts.get("$row"), Some(&1));
    assert_eq!(report.field_counts.get("d"), Some(&7));
    assert_eq!(report.field_counts.get("g"), Some(&7));
    assert_eq!(report.field_counts.get("nested"), Some(&1));
    assert_eq!(report.diagnostics.get("duplicate_root_key"), Some(&2));
    assert_eq!(report.diagnostics.get("duplicate_key"), Some(&1));
    assert_eq!(report.diagnostics.get("non_object_row"), Some(&1));
    assert_eq!(report.diagnostics.get("unknown_native_field"), Some(&2));

    let document = imported.dictionary().document();
    let capture = &document.captures[0];
    assert_eq!(capture.bytes, RAW_FIXTURE);
    assert_eq!(capture.occurrences.len(), 8);
    assert_eq!(capture.occurrences[0].locator, "/0");
    assert_eq!(capture.occurrences[0].fields[0].name, "$key");
    assert_eq!(
        capture.occurrences[0].fields[0].raw_value,
        r#""f\u0061rvi""#
    );
    assert_eq!(capture.occurrences[0].fields[3].raw_value, "null");
    assert_eq!(capture.occurrences[0].fields[4].name, "g");
    assert_eq!(
        capture.occurrences[2].fields[1].raw_value,
        r#"{"dup":"one","dup":"two"}"#
    );
    assert_eq!(capture.occurrences[3].fields[1].name, "$row");
    assert_eq!(capture.occurrences[3].fields[1].raw_value, "true");

    let farvi = document
        .metadata
        .entries
        .iter()
        .filter(|entry| entry.spelling == Presence::Value("farvi".into()))
        .collect::<Vec<_>>();
    assert_eq!(farvi.len(), 2);
    assert_ne!(farvi[0].id, farvi[1].id);
    assert_eq!(farvi[0].native_word_class, Presence::Value("gismu".into()));
    assert_eq!(farvi[1].native_word_class, Presence::Value("gismu".into()));
    assert!(
        document
            .metadata
            .assertions
            .iter()
            .take(2)
            .all(|row| row.eligibility == Eligibility::Lexical)
    );

    let ambiguous_entry = &document.metadata.entries[2];
    let ambiguous_assertion = &document.metadata.assertions[2];
    assert_eq!(ambiguous_entry.spelling, Presence::Value("nested".into()));
    assert_eq!(ambiguous_entry.native_word_class, Presence::Missing);
    assert_eq!(ambiguous_assertion.definition, Presence::Missing);
    assert!(matches!(
        ambiguous_assertion.eligibility,
        Eligibility::Excluded { .. }
    ));

    let null_note = document
        .metadata
        .contexts
        .iter()
        .find(|context| context.kind == ContextKind::Note && context.value == Presence::Null)
        .expect("explicit null note is a retained context");
    assert_eq!(null_note.evidence.field_index, Some(5));

    let forms = &document.metadata.forms;
    assert_eq!(
        forms
            .iter()
            .filter(|form| form.kind == FormKind::SourceReportedForm)
            .count(),
        3
    );
    assert!(
        forms
            .iter()
            .all(|form| form.language == Presence::Value("jbo".into()))
    );
    assert!(
        forms
            .iter()
            .all(|form| form.standing == Presence::Value("r".into()))
    );

    let gloss = document
        .metadata
        .extensions
        .iter()
        .find(|extension| extension.name == "sutysisku.g.raw")
        .unwrap();
    assert_eq!(
        gloss.value.as_str(),
        r#"["to flourish",null,{"kind":"raw"}]"#
    );
    assert_eq!(gloss.evidence.as_ref().unwrap().field_index, Some(4));
    let opaque = document
        .metadata
        .extensions
        .iter()
        .find(|extension| extension.name == "sutysisku.b")
        .unwrap();
    assert_eq!(opaque.value.as_str(), r#"{"opaque":true}"#);
    assert_eq!(opaque.evidence.as_ref().unwrap().field_index, Some(7));
    let unknown = document
        .metadata
        .extensions
        .iter()
        .find(|extension| extension.name == "sutysisku.native_field.8.x/y~z")
        .unwrap();
    assert_eq!(unknown.value.as_str(), r#""extra""#);
    assert_eq!(unknown.evidence.as_ref().unwrap().field_index, Some(8));

    let registry = document.registry.as_document();
    assert_eq!(
        registry.occurrences[0].native_entry_id.as_deref(),
        Some("farvi")
    );
    assert_eq!(registry.occurrences[0].native_assertion_id, None);
    assert_eq!(
        document.metadata.releases[0].redistribution,
        vlajmaji_lexicon::RedistributionStatus::Held
    );
}

#[test]
fn test_import_sutysisku_export_invalid_pin_and_root_inputs_return_typed_errors() {
    let mut options = options_for(RAW_FIXTURE);
    options.revision = "bad".into();
    assert!(matches!(
        import_sutysisku_export(RAW_FIXTURE, &options),
        Err(SutysiskuImportError::InvalidOptions(_))
    ));

    let mut options = options_for(RAW_FIXTURE);
    options.raw_locator = Some("  ".into());
    assert!(matches!(
        import_sutysisku_export(RAW_FIXTURE, &options),
        Err(SutysiskuImportError::InvalidOptions(_))
    ));

    let mut options = options_for(RAW_FIXTURE);
    options.expected_raw_sha256 = "0".repeat(64);
    assert!(matches!(
        import_sutysisku_export(RAW_FIXTURE, &options),
        Err(SutysiskuImportError::HashMismatch { .. })
    ));

    let invalid_utf8 = [0xff, 0xfe];
    assert!(matches!(
        import_sutysisku_export(&invalid_utf8, &options_for(&invalid_utf8)),
        Err(SutysiskuImportError::InvalidUtf8)
    ));

    for invalid in [b"[]".as_slice(), b"{".as_slice(), b"null".as_slice()] {
        assert!(matches!(
            import_sutysisku_export(invalid, &options_for(invalid)),
            Err(SutysiskuImportError::InvalidJson(_))
        ));
    }
}

#[test]
fn test_import_sutysisku_export_revision_changes_provenance_not_capture_identity() {
    let original = import_sutysisku_export(RAW_FIXTURE, &options_for(RAW_FIXTURE)).unwrap();
    let mut revised_options = options_for(RAW_FIXTURE);
    revised_options.revision = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into();
    let revised = import_sutysisku_export(RAW_FIXTURE, &revised_options).unwrap();
    let original_doc = original.dictionary().document();
    let revised_doc = revised.dictionary().document();
    assert_eq!(
        original_doc.captures[0].identity,
        revised_doc.captures[0].identity
    );
    assert_eq!(
        original_doc.metadata.entries[0].id,
        revised_doc.metadata.entries[0].id
    );
    assert_ne!(
        original_doc.metadata.releases[0].id,
        revised_doc.metadata.releases[0].id
    );
    assert_eq!(
        revised_doc.captures[0].metadata.locator,
        format!(
            "https://github.com/La-Lojban/sutysisku/blob/{}/data/parsed-en.json",
            revised_options.revision
        )
    );
    assert_eq!(
        revised_doc.captures[0].metadata.notice_locator,
        Presence::Value(format!(
            "https://github.com/La-Lojban/sutysisku/blob/{}/README.md",
            revised_options.revision
        ))
    );

    revised_options.raw_locator = Some("file:///tmp/local-sut.json".into());
    revised_options.notice_locator = Some("file:///tmp/local-terms.txt".into());
    let explicit = import_sutysisku_export(RAW_FIXTURE, &revised_options).unwrap();
    let explicit_doc = explicit.dictionary().document();
    assert_eq!(
        explicit_doc.captures[0].metadata.locator,
        "file:///tmp/local-sut.json"
    );
    assert_eq!(
        explicit_doc.captures[0].metadata.notice_locator,
        Presence::Value("file:///tmp/local-terms.txt".into())
    );
}

#[test]
fn test_import_sutysisku_export_preserves_repeated_and_long_source_reported_forms() {
    let raw = br#"{"farvi":{"d":"grow","t":"gismu","r":["far","far","farvi"]}}"#;
    let imported = import_sutysisku_export(raw, &options_for(raw)).unwrap();
    let forms = &imported.dictionary().document().metadata.forms;
    assert_eq!(forms.len(), 3);
    assert_eq!(forms[0].spelling, "far");
    assert_eq!(forms[1].spelling, "far");
    assert_eq!(forms[2].spelling, "farvi");
    assert_eq!(imported.report().reported_forms, 3);
}
