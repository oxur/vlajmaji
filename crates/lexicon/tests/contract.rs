//! Contract tests for identity, fidelity, full-source counts, and exact lookup.

use vlajmaji_lexicon::{
    FieldPresence, FormRole, ImportError, Lexicon, Proposal, SourceFormat, SourceMetadata,
    TimePrecision,
};

fn metadata(namespace: &str) -> SourceMetadata {
    SourceMetadata::new(
        namespace,
        "fixture",
        SourceFormat::LaLojbanJson,
        "memory://fixture",
        "fixture-v1",
        None,
        TimePrecision::Unknown,
    )
}

#[test]
fn import_is_atomic_and_source_identity_is_byte_exact() {
    let mut lexicon = Lexicon::new();
    let good = br#"[{"word":"si'e","definition":"cmavo","selmaho":"MOI"}]"#;
    assert!(lexicon.import_source(metadata("la"), good, None).unwrap());
    assert!(!lexicon.import_source(metadata("la"), good, None).unwrap());
    let before = lexicon.records().to_vec();

    for bad in [
        br#"[{"word":"x","definition":"d","nested":{"k":1,"k":2}}]"#.as_slice(),
        br#"[{"word":"x","word":"y","definition":"d"}]"#.as_slice(),
        br#"[{"word":"x","definition":7}]"#.as_slice(),
        br#"[{"word":null,"definition":"d"}]"#.as_slice(),
        br#"[{"word":"x","definition":"d","score":"high"}]"#.as_slice(),
        br#"[{"word":"x","definition":"d","gloss_keywords":"not an array"}]"#.as_slice(),
        br#"[{"word":"x","definition":"d","rafsi":[1]}]"#.as_slice(),
        b"[{\"word\":\"x\",\"definition\":\"d\"".as_slice(),
        &[0xff][..],
    ] {
        assert!(lexicon.import_source(metadata("bad"), bad, None).is_err());
        assert_eq!(lexicon.records(), before.as_slice());
    }
    assert!(matches!(
        lexicon.import_source(metadata("bad-hash"), good, Some(&"0".repeat(64))),
        Err(ImportError::HashMismatch { .. })
    ));
    assert_eq!(lexicon.records(), before.as_slice());
}

#[test]
fn exact_queries_keep_source_rows_and_proposals_separate() {
    let mut lexicon = Lexicon::new();
    let source = br#"[{"word":"si'e","definition":"cmavo","notes":"See also {target}. $x_{1}$ {free}"},{"word":"target","definition":"target meaning"},{"word":"si'e","definition":"assigned form is separate row"}]"#;
    lexicon
        .import_source(metadata("captured:la"), source, None)
        .unwrap();
    assert_eq!(lexicon.lookup_exact("si'e").len(), 2);
    assert!(lexicon.lookup_exact("sie").is_empty());
    assert_eq!(lexicon.cross_references().len(), 1);
    let reference = &lexicon.cross_references()[0];
    assert_eq!(reference.origin().locator(), "/0");
    assert_eq!(reference.target_spelling(), "target");
    assert_eq!(reference.targets().len(), 1);
    assert_eq!(
        &"See also {target}. $x_{1}$ {free}"[reference.byte_start()..reference.byte_end()],
        "{target}"
    );

    lexicon.add_proposal(Proposal::new("experiment-1", "si'e", "proposal").unwrap());
    assert_eq!(lexicon.lookup_exact("si'e").len(), 2);
    assert_eq!(lexicon.lookup_proposals("si'e").len(), 1);
    assert!(Proposal::new("captured:fake", "si'e", "not captured").is_err());
}

#[test]
fn see_also_parser_uses_explicit_marker_and_excludes_tex_math() {
    let notes =
        "See also $x_{1}$ and {target}. Foresee also {substring}. See also {boundary}; {outside}.";
    let mut lexicon = Lexicon::new();
    lexicon
        .import_source(
            metadata("captured:notes"),
            format!(
                r#"[{{"word":"origin","definition":"d","notes":{}}},{{"word":"target","definition":"d"}},{{"word":"substring","definition":"d"}},{{"word":"boundary","definition":"d"}},{{"word":"outside","definition":"d"}}]"#,
                serde_json::to_string(notes).unwrap()
            )
            .as_bytes(),
            None,
        )
        .unwrap();
    let references = lexicon.cross_references();
    assert_eq!(references.len(), 2);
    assert_eq!(references[0].target_spelling(), "target");
    assert_eq!(references[1].target_spelling(), "boundary");
    assert!(references.iter().all(|reference| {
        reference.rule_version() == "see-also-braces-v2" && reference.targets().len() == 1
    }));
    assert_eq!(
        &notes[references[0].byte_start()..references[0].byte_end()],
        "{target}"
    );
}

#[test]
fn proposal_identity_is_unambiguous_and_archive_validation_is_typed() {
    let left = Proposal::new("a\0b", "c", "d").unwrap();
    let right = Proposal::new("a", "b\0c", "d").unwrap();
    assert_ne!(left.id(), right.id());

    let mut lexicon = Lexicon::new();
    lexicon.add_proposal(left);
    let archive = lexicon.to_archive_bytes().unwrap();
    assert_eq!(
        Lexicon::from_archive_bytes(&archive)
            .unwrap()
            .to_archive_bytes()
            .unwrap(),
        archive
    );

    let mut legacy: serde_json::Value = serde_json::from_slice(&archive).unwrap();
    legacy["version"] = serde_json::json!(1);
    let legacy_tuple = format!("{}\0{}\0{}", "a\0b", "c", "d");
    use sha2::{Digest, Sha256};
    legacy["proposals"][0]["id"] =
        serde_json::json!(format!("{:x}", Sha256::digest(legacy_tuple.as_bytes())));
    let migrated = Lexicon::from_archive_bytes(&serde_json::to_vec(&legacy).unwrap()).unwrap();
    let migrated_archive = migrated.to_archive_bytes().unwrap();
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&migrated_archive).unwrap()["version"],
        2
    );

    let mut invalid_namespace: serde_json::Value = serde_json::from_slice(&archive).unwrap();
    invalid_namespace["proposals"][0]["namespace"] = serde_json::json!("captured:forged");
    assert!(matches!(
        Lexicon::from_archive_bytes(&serde_json::to_vec(&invalid_namespace).unwrap()),
        Err(ImportError::InvalidProposalNamespace)
    ));

    let mut invalid_id: serde_json::Value = serde_json::from_slice(&archive).unwrap();
    invalid_id["proposals"][0]["id"] = serde_json::json!("forged");
    assert!(matches!(
        Lexicon::from_archive_bytes(&serde_json::to_vec(&invalid_id).unwrap()),
        Err(ImportError::InvalidProposalIdentity)
    ));
}

#[test]
fn archive_roundtrip_preserves_raw_presence_unknowns_and_large_numbers() {
    let raw = br#"[{"word":"x","definition":"d","notes":"","unknown":{"empty":[],"huge":1234567890123456789012345678901234567890}},{"word":"y","definition":"d","notes":null,"extra":{}},{"word":"z","definition":"d"}]"#;
    let mut lexicon = Lexicon::new();
    lexicon
        .import_source(metadata("captured:test"), raw, None)
        .unwrap();
    assert_eq!(
        lexicon.records()[0].field("notes"),
        FieldPresence::Value("\"\"")
    );
    assert_eq!(lexicon.records()[1].field("notes"), FieldPresence::Null);
    assert_eq!(lexicon.records()[2].field("notes"), FieldPresence::Missing);
    assert!(
        lexicon.records()[0]
            .field("unknown")
            .eq(&FieldPresence::Value(
                r#"{"empty":[],"huge":1234567890123456789012345678901234567890}"#
            ))
    );
    let snapshot = lexicon.snapshots()[0].clone();
    assert_eq!(lexicon.source_bytes(&snapshot), Some(raw.as_slice()));
    let archive = lexicon.to_archive_bytes().unwrap();
    let restored = Lexicon::from_archive_bytes(&archive).unwrap();
    assert_eq!(restored.to_archive_bytes().unwrap(), archive);
    assert_eq!(restored.source_bytes(&snapshot), Some(raw.as_slice()));

    let mut tampered: serde_json::Value = serde_json::from_slice(&archive).unwrap();
    tampered["sources"][0]["raw_bytes"][0] = serde_json::json!(b'x');
    let tampered = serde_json::to_vec(&tampered).unwrap();
    assert!(Lexicon::from_archive_bytes(&tampered).is_err());

    let mut unsupported = archive.clone();
    let marker = b"\"version\":2";
    let at = unsupported
        .windows(marker.len())
        .position(|w| w == marker)
        .unwrap();
    unsupported[at + marker.len() - 1] = b'9';
    assert!(matches!(
        Lexicon::from_archive_bytes(&unsupported),
        Err(ImportError::UnsupportedArchiveVersion(9))
    ));
}

#[test]
fn corpus_sources_preserve_form_roles() {
    let mut lexicon = Lexicon::new();
    let assigned = SourceMetadata::new(
        "captured:llg:rafsi",
        "LLG rafsi list",
        SourceFormat::LlgRafsiRows,
        "fixture://llg-rafsi",
        "fixture-v1",
        None,
        TimePrecision::Unknown,
    );
    lexicon
        .import_source(assigned, b"si'e snime snow\n", None)
        .unwrap();
    let rafsi_record = lexicon.records().first().unwrap();
    assert_eq!(rafsi_record.definition(), None);
    assert_eq!(rafsi_record.gloss(), Some("snow"));
    assert_eq!(rafsi_record.field("gloss"), FieldPresence::Value("snow"));
    assert_eq!(rafsi_record.field("rafsi"), FieldPresence::Value("si'e"));
    let literal_null = SourceMetadata::new(
        "captured:llg:literal-null",
        "LLG rafsi fixture",
        SourceFormat::LlgRafsiRows,
        "fixture://llg-rafsi-null",
        "fixture-v1",
        None,
        TimePrecision::Unknown,
    );
    lexicon
        .import_source(literal_null, b"fav farvi null\n", None)
        .unwrap();
    let literal_null_record = lexicon.records().last().unwrap();
    assert_eq!(
        literal_null_record.field("gloss"),
        FieldPresence::Value("null")
    );
    assert_eq!(literal_null_record.gloss(), Some("null"));
    assert_eq!(literal_null_record.definition(), None);
    let json = br#"[{"word":"snime","definition":"snow","rafsi":"si'e"}]"#;
    lexicon.import_source(metadata("la"), json, None).unwrap();
    assert!(
        lexicon
            .forms()
            .iter()
            .any(|form| form.role() == FormRole::AssignedRafsi && form.form() == "si'e")
    );
    assert!(
        lexicon
            .forms()
            .iter()
            .any(|form| { form.role() == FormRole::SourceReportedForm && form.form() == "si'e" })
    );
}

#[test]
fn proposals_borrowed_view_exposes_all_entries_without_captured_lookup() {
    let mut lexicon = Lexicon::new();
    lexicon.add_proposal(Proposal::new("draft-a", "x", "one").unwrap());
    lexicon.add_proposal(Proposal::new("draft-b", "y", "two").unwrap());

    let proposals = lexicon.proposals();
    assert_eq!(proposals.len(), 2);
    assert_eq!(proposals[0].spelling(), "x");
    assert_eq!(proposals[1].spelling(), "y");
    assert!(lexicon.lookup_exact("x").is_empty());
    assert_eq!(lexicon.lookup_proposals("x").len(), 1);
}

#[test]
#[ignore = "requires RELOJBAN_RESEARCH_ROOT pointing at the preserved research checkout"]
fn full_corpus_import_reconciles_all_input_populations() {
    use sha2::{Digest, Sha256};
    use std::path::PathBuf;
    use vlajmaji_lexicon::{SourceFormat, SourceMetadata};

    let root = PathBuf::from(
        std::env::var_os("RELOJBAN_RESEARCH_ROOT")
            .expect("RELOJBAN_RESEARCH_ROOT is required for this external corpus test"),
    );
    let e02 = root.join("experiments/E02-resource-discovery/raw/lexical");
    let mut lexicon = Lexicon::new();
    let la = std::fs::read(e02.join("la-lojban-en.json")).unwrap();
    let la_hash = format!("{:x}", Sha256::digest(&la));
    lexicon
        .import_source(
            SourceMetadata::new(
                "captured:la",
                "la export",
                SourceFormat::LaLojbanJson,
                "E02/raw/lexical/la-lojban-en.json",
                &la_hash,
                Some("2026-10-23".into()),
                TimePrecision::Day,
            ),
            &la,
            Some(&la_hash),
        )
        .unwrap();
    assert_eq!(lexicon.records().len(), 17_056);
    assert_eq!(lexicon.lookup_exact("si'e").len(), 1);
    assert!(lexicon.lookup_exact("sie").is_empty());
    assert_eq!(lexicon.lookup_exact("le jaica").len(), 1);

    let mut api_count = 0;
    for path in [
        "banro",
        "citno",
        "cpana",
        "farvi",
        "flaume",
        "kalri",
        "ricrprunu",
        "rutrprunu",
        "slabu",
        "snime",
        "tricu",
        "xrula",
    ] {
        let raw = std::fs::read(e02.join(format!("lensisku-{path}.json"))).unwrap();
        let hash = format!("{:x}", Sha256::digest(&raw));
        let metadata = SourceMetadata::new(
            format!("captured:lensisku:{path}"),
            "Lensisku response",
            SourceFormat::LensiskuJson,
            format!("E02/raw/lexical/lensisku-{path}.json"),
            &hash,
            Some("2026-10-23".into()),
            TimePrecision::Day,
        );
        lexicon.import_source(metadata, &raw, Some(&hash)).unwrap();
        api_count += lexicon
            .records()
            .iter()
            .filter(|record| record.identity().snapshot().namespace().ends_with(path))
            .count();
    }
    assert_eq!(api_count, 113);
    let farvi = lexicon.lookup_exact("farvi");
    assert!(
        farvi
            .iter()
            .any(|row| row.field("definitionid") == FieldPresence::Value("69002"))
    );
    for form in ["fav", "farv"] {
        assert!(lexicon.forms().iter().any(|assertion| {
            assertion.headword() == "farvi"
                && assertion.form() == form
                && assertion.role() == FormRole::SourceReportedForm
                && assertion.source_field() == "rafsi"
        }));
    }
    assert!(lexicon.cross_references().iter().any(|reference| {
        reference.origin().snapshot().namespace() == "captured:lensisku:farvi"
            && reference.origin().locator() == "/1"
            && reference.target_spelling() == "pruce"
            && !reference.targets().is_empty()
    }));
    assert!(
        farvi
            .iter()
            .any(|row| row.field("definitionid") == FieldPresence::Value("333"))
    );
    assert!(
        farvi
            .iter()
            .any(|row| row.field("langid") == FieldPresence::Value("1"))
    );
    assert!(
        farvi
            .iter()
            .any(|row| row.field("langid") == FieldPresence::Value("2"))
    );

    for (file, format, namespace, expected) in [
        (
            "llg-gismu.txt",
            SourceFormat::LlgGismuFixed,
            "captured:llg:gismu",
            1_436,
        ),
        (
            "llg-cmavo.txt",
            SourceFormat::LlgCmavoFixed,
            "captured:llg:cmavo",
            1_091,
        ),
        (
            "llg-rafsi.txt",
            SourceFormat::LlgRafsiRows,
            "captured:llg:rafsi",
            1_545,
        ),
    ] {
        let raw = std::fs::read(e02.join(file)).unwrap();
        let hash = format!("{:x}", Sha256::digest(&raw));
        let before = lexicon.records().len();
        lexicon
            .import_source(
                SourceMetadata::new(
                    namespace,
                    file,
                    format,
                    format!("E02/raw/lexical/{file}"),
                    &hash,
                    Some("2026-10-23".into()),
                    TimePrecision::Day,
                ),
                &raw,
                Some(&hash),
            )
            .unwrap();
        assert_eq!(lexicon.records().len() - before, expected);
    }
    let rafsi_heads: std::collections::BTreeSet<_> = lexicon
        .records()
        .iter()
        .filter(|record| record.format() == SourceFormat::LlgRafsiRows)
        .filter_map(|record| record.word())
        .collect();
    assert_eq!(rafsi_heads.len(), 1_196);
    assert!(lexicon.forms().iter().any(|form| {
        form.headword() == "snime"
            && form.form() == "si'e"
            && form.role() == FormRole::AssignedRafsi
    }));
    assert!(lexicon.lookup_exact("si'e").iter().any(|record| {
        record.format() == SourceFormat::LlgCmavoFixed
            && record.field("selmaho") == FieldPresence::Value("MOI")
    }));
    assert!(lexicon.records().iter().any(|record| {
        record.format() == SourceFormat::LlgGismuFixed
            && record.word() == Some("cpana")
            && [
                "assigned_rafsi_cvc",
                "assigned_rafsi_ccv",
                "assigned_rafsi_cvv",
            ]
            .iter()
            .all(|name| record.field(name) == FieldPresence::Value(""))
    }));
    assert!(lexicon.forms().iter().any(|form| {
        form.headword() == "cpana"
            && form.form() == "cpan"
            && form.role() == FormRole::SourceReportedForm
    }));
    assert!(
        !lexicon
            .forms()
            .iter()
            .any(|form| { form.form() == "snim" && form.role() == FormRole::AssignedRafsi })
    );
    assert_eq!(lexicon.records().len(), 21_241);
    assert_eq!(
        lexicon
            .records()
            .iter()
            .filter(|record| record.definition().is_some())
            .count(),
        19_696
    );
    assert_eq!(
        lexicon
            .records()
            .iter()
            .filter(|record| {
                record.format() == SourceFormat::LlgRafsiRows
                    && record.definition().is_none()
                    && record.gloss().is_some()
            })
            .count(),
        1_545
    );
}
