//! Source-registry and attribution contract tests.

use vlajmaji_lexicon::{
    AbbreviationKind, AliasScope, AliasStatus, ContributorId, ContributorRecord, DistributorId,
    DistributorRecord, FieldPresence, FormRole, Lexicon, LineageTarget, OccurrenceAttribution,
    Proposal, RegistryDocument, RegistryError, SnapshotAttribution, SourceAlias, SourceFormat,
    SourceId, SourceLineage, SourceMetadata, SourceOrigin, SourceRecord, SourceRegistry,
    TimePrecision,
};

const A1_D1: &[u8] = br#"[{"word":"farvi","definition":"move forward","notes":"","mystery":{"huge":123456789012345678901234567890}},{"word":"farvi","definition":"move forward","notes":"","mystery":{"huge":123456789012345678901234567890}},{"word":"farvi","definition":"cause to advance"}]"#;
const A2_D2: &[u8] = br#"[{"valsiword":"farvi","definition":"move forward"}]"#;
const B1_D1: &[u8] = br#"[{"word":"farvi","definition":"make progress"}]"#;
const A_LEXICAL: &[u8] = br#"[{"word":"si'e","definition":"cmavo","selmaho":"MOI"},{"word":"sie","definition":"plain spelling","notes":null},{"word":"Si'e","definition":"case variant"},{"word":"si\u2019e","definition":"curly apostrophe"},{"word":"le jaica","definition":"multiword entry"},{"word":"cpana","definition":"above","rafsi":"cpan"}]"#;
const A_ASSIGNED: &[u8] = b"si'e snime snow\n";

fn metadata(namespace: &str, format: SourceFormat) -> SourceMetadata {
    SourceMetadata::new(
        namespace,
        "synthetic fixture",
        format,
        format!("fixture://{namespace}"),
        "fixture-v1",
        None,
        TimePrecision::Unknown,
    )
}

fn fixture() -> (Lexicon, RegistryDocument) {
    let mut lexicon = Lexicon::new();
    for (namespace, format, bytes) in [
        ("captured:a:d1", SourceFormat::LaLojbanJson, A1_D1),
        ("captured:a:d2", SourceFormat::LensiskuJson, A2_D2),
        ("captured:b:d1", SourceFormat::LaLojbanJson, B1_D1),
        ("captured:a:lexical", SourceFormat::LaLojbanJson, A_LEXICAL),
        (
            "captured:a:assigned",
            SourceFormat::LlgRafsiRows,
            A_ASSIGNED,
        ),
    ] {
        lexicon
            .import_source(metadata(namespace, format), bytes, None)
            .unwrap();
    }
    let snapshots = lexicon.snapshots();
    let snapshot = |namespace: &str| {
        snapshots
            .iter()
            .find(|identity| identity.namespace() == namespace)
            .map(|identity| (*identity).clone())
            .unwrap()
    };
    let a = SourceId::new("src:a").unwrap();
    let b = SourceId::new("src:b").unwrap();
    let person = ContributorId::new("person:one").unwrap();
    let d1 = DistributorId::new("dist:one").unwrap();
    let d2 = DistributorId::new("dist:two").unwrap();
    let rows: Vec<_> = lexicon
        .lookup_exact("farvi")
        .into_iter()
        .map(|record| record.identity().clone())
        .collect();
    assert_eq!(rows.len(), 5);

    let document = RegistryDocument {
        sources: vec![
            SourceRecord {
                id: a.clone(),
                display_name: "Zeta Dictionary".into(),
                abbreviation: "FX-A".into(),
                abbreviation_kind: AbbreviationKind::ProjectAssigned,
                abbreviation_evidence: None,
                origin: SourceOrigin::Upstream,
            },
            SourceRecord {
                id: b.clone(),
                display_name: "Alpha Dictionary".into(),
                abbreviation: "FX-B".into(),
                abbreviation_kind: AbbreviationKind::ProjectAssigned,
                abbreviation_evidence: None,
                origin: SourceOrigin::Upstream,
            },
            SourceRecord {
                id: SourceId::new("src:draft").unwrap(),
                display_name: "Local draft".into(),
                abbreviation: "DRAFT".into(),
                abbreviation_kind: AbbreviationKind::ProjectAssigned,
                abbreviation_evidence: None,
                origin: SourceOrigin::OwnedDraft,
            },
        ],
        contributors: vec![ContributorRecord {
            id: person.clone(),
            display_name: "Fixture Author".into(),
        }],
        distributors: vec![
            DistributorRecord {
                id: d1.clone(),
                display_name: "Distributor One".into(),
            },
            DistributorRecord {
                id: d2.clone(),
                display_name: "Distributor Two".into(),
            },
        ],
        aliases: vec![SourceAlias {
            source: a.clone(),
            text: "OLD".into(),
            status: AliasStatus::Active,
            evidence: Some("synthetic historical selector".into()),
        }],
        lineage: vec![
            SourceLineage {
                source: a.clone(),
                upstream: LineageTarget::Unresolved("fixture://unresolved-source".into()),
                evidence: None,
            },
            SourceLineage {
                source: b.clone(),
                upstream: LineageTarget::Known(a.clone()),
                evidence: Some("synthetic fixture relationship".into()),
            },
        ],
        snapshots: vec![
            SnapshotAttribution {
                snapshot: snapshot("captured:a:d1"),
                source: a.clone(),
                distributor: Some(d1.clone()),
            },
            SnapshotAttribution {
                snapshot: snapshot("captured:a:d2"),
                source: a.clone(),
                distributor: Some(d2),
            },
            SnapshotAttribution {
                snapshot: snapshot("captured:b:d1"),
                source: b.clone(),
                distributor: Some(d1),
            },
            SnapshotAttribution {
                snapshot: snapshot("captured:a:lexical"),
                source: a.clone(),
                distributor: None,
            },
            SnapshotAttribution {
                snapshot: snapshot("captured:a:assigned"),
                source: a,
                distributor: None,
            },
        ],
        occurrences: vec![
            OccurrenceAttribution {
                occurrence: rows[0].clone(),
                contributor: Some(person.clone()),
                native_entry_id: Some("".into()),
                native_assertion_id: Some("upstream:farvi:1".into()),
            },
            OccurrenceAttribution {
                occurrence: rows[1].clone(),
                contributor: None,
                native_entry_id: None,
                native_assertion_id: Some("upstream:farvi:1".into()),
            },
            OccurrenceAttribution {
                occurrence: rows[2].clone(),
                contributor: None,
                native_entry_id: Some("entry-alt".into()),
                native_assertion_id: None,
            },
            OccurrenceAttribution {
                occurrence: rows[3].clone(),
                contributor: Some(person),
                native_entry_id: None,
                native_assertion_id: Some("upstream:farvi:1".into()),
            },
        ],
        ..RegistryDocument::default()
    };
    (lexicon, document)
}

#[test]
fn ids_and_registry_document_are_validated_and_role_distinct() {
    let source = SourceId::new("id:one").unwrap();
    let contributor = ContributorId::new("id:one").unwrap();
    assert_eq!(source.as_str(), contributor.as_str());
    assert_ne!(format!("{source:?}"), format!("{contributor:?}"));
    for invalid in ["", "Upper", " white", "a/b", "a\0b", "éclair"] {
        assert!(matches!(
            SourceId::new(invalid),
            Err(RegistryError::InvalidId { .. })
        ));
    }
    assert_eq!(RegistryDocument::default().version, 1);
    let (lexicon, document) = fixture();
    let registry = SourceRegistry::from_document(document).unwrap();
    registry.validate_lexicon(&lexicon).unwrap();
    assert_eq!(
        SourceRegistry::from_json_bytes(&registry.to_json_bytes().unwrap()).unwrap(),
        registry
    );
    assert_eq!(
        registry
            .resolve_source("FX-A", AliasScope::ActiveOnly)
            .unwrap()
            .id,
        SourceId::new("src:a").unwrap()
    );
    assert_eq!(
        registry
            .resolve_source("OLD", AliasScope::ActiveOnly)
            .unwrap()
            .id,
        SourceId::new("src:a").unwrap()
    );
    assert!(matches!(
        registry.resolve_source("Zeta Dictionary", AliasScope::ActiveOnly),
        Err(RegistryError::UnknownSource(_))
    ));
    assert!(matches!(
        registry.resolve_source("missing", AliasScope::ActiveOnly),
        Err(RegistryError::UnknownSource(_))
    ));
}

#[test]
fn attribution_preserves_five_occurrences_roles_raw_bytes_and_archive() {
    let (lexicon, document) = fixture();
    let registry = SourceRegistry::from_document(document).unwrap();
    let before_archive = lexicon.to_archive_bytes().unwrap();
    let before_ids: Vec<_> = lexicon
        .records()
        .iter()
        .map(|record| record.identity().clone())
        .collect();
    let before_bytes: Vec<_> = lexicon
        .snapshots()
        .iter()
        .map(|snapshot| lexicon.source_bytes(snapshot).unwrap().to_vec())
        .collect();

    let attributed = registry.attributed_records(&lexicon).unwrap();
    assert_eq!(attributed.len(), lexicon.records().len());
    let farvi: Vec<_> = attributed
        .iter()
        .filter(|row| row.record().word() == Some("farvi"))
        .collect();
    assert_eq!(farvi.len(), 5);
    assert_eq!(
        farvi
            .iter()
            .filter(|row| row.source().id == SourceId::new("src:a").unwrap())
            .count(),
        4
    );
    assert_eq!(
        farvi
            .iter()
            .filter(|row| row.source().id == SourceId::new("src:b").unwrap())
            .count(),
        1
    );
    assert_eq!(farvi[0].record().identity().locator(), "/0");
    assert_eq!(farvi[1].record().identity().locator(), "/1");
    assert_eq!(farvi[0].record().fields(), farvi[1].record().fields());
    assert_eq!(farvi[0].native_assertion_id(), Some("upstream:farvi:1"));
    assert_eq!(farvi[1].native_assertion_id(), Some("upstream:farvi:1"));
    assert_ne!(farvi[0].record().identity(), farvi[1].record().identity());
    assert_eq!(farvi[0].native_entry_id(), Some(""));
    assert_eq!(
        farvi[0].contributor().unwrap().id,
        ContributorId::new("person:one").unwrap()
    );
    assert!(farvi[1].contributor().is_none());
    assert_eq!(
        farvi[0].distributor().unwrap().id,
        DistributorId::new("dist:one").unwrap()
    );
    assert_eq!(
        farvi[3].distributor().unwrap().id,
        DistributorId::new("dist:two").unwrap()
    );
    assert_eq!(farvi[4].record().definition(), Some("make progress"));

    let exact = |spelling: &str| {
        attributed
            .iter()
            .filter(|row| row.record().word() == Some(spelling))
            .count()
    };
    assert_eq!(exact("si'e"), 1);
    assert_eq!(exact("sie"), 1);
    assert_eq!(exact("Si'e"), 1);
    assert_eq!(exact("si\u{2019}e"), 1);
    assert_eq!(exact("le jaica"), 1);
    assert_eq!(exact("cpana"), 1);
    assert_eq!(
        lexicon
            .forms()
            .iter()
            .filter(|form| form.form() == "cpan")
            .count(),
        1
    );
    let lexical_sihe = attributed
        .iter()
        .find(|row| row.record().word() == Some("si'e"))
        .unwrap();
    assert_eq!(
        lexical_sihe.record().field("selmaho"),
        FieldPresence::Value("\"MOI\"")
    );
    assert!(
        lexicon
            .forms()
            .iter()
            .any(|form| { form.form() == "si'e" && form.role() == FormRole::AssignedRafsi })
    );
    assert!(
        !lexicon
            .forms()
            .iter()
            .any(|form| { form.form() == "snim" && form.role() == FormRole::AssignedRafsi })
    );
    let raw = lexicon
        .records()
        .iter()
        .find(|record| record.identity().snapshot().namespace() == "captured:a:d1")
        .unwrap();
    assert_eq!(raw.field("notes"), FieldPresence::Value("\"\""));
    assert!(
        matches!(raw.field("mystery"), FieldPresence::Value(value) if value.contains("123456789012345678901234567890"))
    );

    let mut proposal_lexicon = lexicon.clone();
    proposal_lexicon.add_proposal(Proposal::new("fixture-draft", "farvi", "not captured").unwrap());
    let proposed = registry.attributed_records(&proposal_lexicon).unwrap();
    assert_eq!(
        proposed
            .iter()
            .filter(|row| row.record().word() == Some("farvi"))
            .count(),
        5
    );
    assert_eq!(proposal_lexicon.lookup_proposals("farvi").len(), 1);

    assert_eq!(lexicon.to_archive_bytes().unwrap(), before_archive);
    assert_eq!(
        lexicon
            .records()
            .iter()
            .map(|record| record.identity().clone())
            .collect::<Vec<_>>(),
        before_ids
    );
    assert_eq!(
        lexicon
            .snapshots()
            .iter()
            .map(|snapshot| lexicon.source_bytes(snapshot).unwrap().to_vec())
            .collect::<Vec<_>>(),
        before_bytes
    );
    assert_eq!(
        Lexicon::from_archive_bytes(&before_archive)
            .unwrap()
            .to_archive_bytes()
            .unwrap(),
        before_archive
    );
}

#[test]
fn aliases_rename_retire_and_reuse_without_losing_history() {
    let (_lexicon, document) = fixture();
    let mut registry = SourceRegistry::from_document(document).unwrap();
    let old_json = registry.to_json_bytes().unwrap();
    let mut renamed = registry.document();
    let source_a = renamed
        .sources
        .iter_mut()
        .find(|row| row.id.as_str() == "src:a")
        .unwrap();
    source_a.display_name = "Renamed Zeta".into();
    source_a.abbreviation = "FX-A2".into();
    renamed.aliases.push(SourceAlias {
        source: SourceId::new("src:a").unwrap(),
        text: "FX-A".into(),
        status: AliasStatus::Active,
        evidence: Some("preserved former abbreviation".into()),
    });
    registry.apply_document(renamed).unwrap();
    assert_eq!(
        registry
            .resolve_source("FX-A", AliasScope::ActiveOnly)
            .unwrap()
            .id
            .as_str(),
        "src:a"
    );
    assert_eq!(
        registry
            .resolve_source("src:a", AliasScope::ActiveOnly)
            .unwrap()
            .display_name,
        "Renamed Zeta"
    );

    let mut retired = registry.document();
    retired
        .aliases
        .iter_mut()
        .find(|row| row.text == "OLD" && row.source.as_str() == "src:a")
        .unwrap()
        .status = AliasStatus::Retired;
    registry.apply_document(retired).unwrap();
    assert!(matches!(
        registry.resolve_source("OLD", AliasScope::ActiveOnly),
        Err(RegistryError::UnknownSource(_))
    ));
    assert_eq!(
        registry
            .resolve_source("OLD", AliasScope::IncludeRetired)
            .unwrap()
            .id
            .as_str(),
        "src:a"
    );

    let mut reused = registry.document();
    reused.aliases.push(SourceAlias {
        source: SourceId::new("src:b").unwrap(),
        text: "OLD".into(),
        status: AliasStatus::Active,
        evidence: None,
    });
    registry.apply_document(reused).unwrap();
    assert_eq!(
        registry
            .resolve_source("OLD", AliasScope::ActiveOnly)
            .unwrap()
            .id
            .as_str(),
        "src:b"
    );
    assert!(matches!(
        registry.resolve_source("OLD", AliasScope::IncludeRetired),
        Err(RegistryError::AmbiguousAlias { sources, .. }) if sources.iter().map(SourceId::as_str).collect::<Vec<_>>() == ["src:a", "src:b"]
    ));
    assert_ne!(old_json, registry.to_json_bytes().unwrap());

    let before = registry.to_json_bytes().unwrap();
    let mut revive = registry.document();
    revive
        .aliases
        .iter_mut()
        .find(|row| row.text == "OLD" && row.source.as_str() == "src:a")
        .unwrap()
        .status = AliasStatus::Active;
    assert!(matches!(
        registry.apply_document(revive),
        Err(RegistryError::AliasCollision { .. }) | Err(RegistryError::ProtectedHistory(_))
    ));
    assert_eq!(registry.to_json_bytes().unwrap(), before);

    let mut remove_alias = registry.document();
    remove_alias
        .aliases
        .retain(|row| !(row.source.as_str() == "src:a" && row.text == "OLD"));
    assert!(matches!(
        registry.apply_document(remove_alias),
        Err(RegistryError::ProtectedHistory(_))
    ));
    assert_eq!(registry.to_json_bytes().unwrap(), before);

    let mut rewrite_source = registry.document();
    rewrite_source.sources[0].origin = SourceOrigin::OwnedDraft;
    assert!(matches!(
        registry.apply_document(rewrite_source),
        Err(RegistryError::ProtectedHistory(_))
    ));
    assert_eq!(registry.to_json_bytes().unwrap(), before);
    let mut rewrite_evidence = registry.document();
    rewrite_evidence
        .aliases
        .iter_mut()
        .find(|row| row.text == "OLD" && row.source.as_str() == "src:b")
        .unwrap()
        .evidence = Some("rewritten evidence".into());
    assert!(matches!(
        registry.apply_document(rewrite_evidence),
        Err(RegistryError::ProtectedHistory(_))
    ));
    assert_eq!(registry.to_json_bytes().unwrap(), before);
}

#[test]
fn document_validation_rejects_duplicate_roles_and_invalid_values() {
    let (lexicon, document) = fixture();
    let mut duplicate_source = document.clone();
    duplicate_source
        .sources
        .push(duplicate_source.sources[0].clone());
    assert!(matches!(
        SourceRegistry::from_document(duplicate_source),
        Err(RegistryError::DuplicateEntity { role: "source", .. })
    ));

    let mut duplicate_contributor = document.clone();
    duplicate_contributor
        .contributors
        .push(duplicate_contributor.contributors[0].clone());
    assert!(matches!(
        SourceRegistry::from_document(duplicate_contributor),
        Err(RegistryError::DuplicateEntity {
            role: "contributor",
            ..
        })
    ));

    let mut duplicate_distributor = document.clone();
    duplicate_distributor
        .distributors
        .push(duplicate_distributor.distributors[0].clone());
    assert!(matches!(
        SourceRegistry::from_document(duplicate_distributor),
        Err(RegistryError::DuplicateEntity {
            role: "distributor",
            ..
        })
    ));

    let mut duplicate_alias = document.clone();
    duplicate_alias
        .aliases
        .push(duplicate_alias.aliases[0].clone());
    assert!(matches!(
        SourceRegistry::from_document(duplicate_alias),
        Err(RegistryError::DuplicateAlias { .. })
    ));

    let mut duplicate_snapshot = document.clone();
    duplicate_snapshot
        .snapshots
        .push(duplicate_snapshot.snapshots[0].clone());
    assert!(matches!(
        SourceRegistry::from_document(duplicate_snapshot),
        Err(RegistryError::DuplicateBinding {
            role: "snapshot",
            ..
        })
    ));

    let mut duplicate_occurrence = document.clone();
    duplicate_occurrence
        .occurrences
        .push(duplicate_occurrence.occurrences[0].clone());
    assert!(matches!(
        SourceRegistry::from_document(duplicate_occurrence),
        Err(RegistryError::DuplicateBinding {
            role: "occurrence",
            ..
        })
    ));

    let mut abbreviation_collision = document.clone();
    abbreviation_collision.sources[1].abbreviation =
        abbreviation_collision.sources[0].abbreviation.clone();
    assert!(matches!(
        SourceRegistry::from_document(abbreviation_collision),
        Err(RegistryError::AliasCollision { .. })
    ));

    let mut blank_name = document.clone();
    blank_name.sources[0].display_name = " Zeta Dictionary".into();
    assert!(matches!(
        SourceRegistry::from_document(blank_name),
        Err(RegistryError::InvalidValue { .. })
    ));
    let mut blank_alias = document.clone();
    blank_alias.aliases[0].text = "OLD\n".into();
    assert!(matches!(
        SourceRegistry::from_document(blank_alias),
        Err(RegistryError::InvalidValue { .. })
    ));
    let mut blank_optional_evidence = document.clone();
    blank_optional_evidence.aliases[0].evidence = Some("  ".into());
    assert!(matches!(
        SourceRegistry::from_document(blank_optional_evidence),
        Err(RegistryError::InvalidValue { .. })
    ));
    let mut conventional_without_evidence = document.clone();
    conventional_without_evidence.sources[0].abbreviation_kind = AbbreviationKind::Conventional;
    conventional_without_evidence.sources[0].abbreviation_evidence = None;
    assert!(matches!(
        SourceRegistry::from_document(conventional_without_evidence),
        Err(RegistryError::InvalidValue { .. })
    ));

    let mut unknown_alias_source = document.clone();
    unknown_alias_source.aliases.push(SourceAlias {
        source: SourceId::new("src:missing").unwrap(),
        text: "GONE".into(),
        status: AliasStatus::Retired,
        evidence: None,
    });
    assert!(matches!(
        SourceRegistry::from_document(unknown_alias_source),
        Err(RegistryError::UnknownSource(_))
    ));

    let empty = Lexicon::new();
    let source_registry = SourceRegistry::from_document(document).unwrap();
    assert!(matches!(source_registry.validate_lexicon(&lexicon), Ok(())));
    assert!(matches!(
        source_registry.validate_lexicon(&empty),
        Err(RegistryError::SnapshotNotFound(_))
    ));
}

#[test]
fn aliases_cannot_shadow_another_sources_current_abbreviation() {
    let (_lexicon, document) = fixture();
    let source_a = SourceId::new("src:a").unwrap();
    let source_b = SourceId::new("src:b").unwrap();

    for status in [AliasStatus::Active, AliasStatus::Retired] {
        let mut candidate = document.clone();
        candidate.aliases.push(SourceAlias {
            source: source_a.clone(),
            text: "FX-B".into(),
            status,
            evidence: None,
        });
        assert!(matches!(
            SourceRegistry::from_document(candidate.clone()),
            Err(RegistryError::AliasCollision { alias, sources })
                if alias == "FX-B" && sources == [source_a.clone(), source_b.clone()]
        ));
        let candidate_json = serde_json::to_vec(&candidate).unwrap();
        assert!(SourceRegistry::from_json_bytes(&candidate_json).is_err());
        assert!(serde_json::from_slice::<SourceRegistry>(&candidate_json).is_err());

        let mut registry = SourceRegistry::from_document(document.clone()).unwrap();
        let before = registry.to_json_bytes().unwrap();
        assert!(matches!(
            registry.apply_document(candidate),
            Err(RegistryError::AliasCollision { alias, sources })
                if alias == "FX-B" && sources == [source_a.clone(), source_b.clone()]
        ));
        assert_eq!(registry.to_json_bytes().unwrap(), before);
    }
}

#[test]
fn registry_identity_wires_reject_unknown_nested_fields_without_tightening_legacy_ids() {
    let (_lexicon, document) = fixture();
    let registry = SourceRegistry::from_document(document).unwrap();
    let valid = registry.to_json_bytes().unwrap();
    let parsed: SourceRegistry = serde_json::from_slice(&valid).unwrap();
    assert_eq!(parsed.to_json_bytes().unwrap(), valid);
    assert_eq!(
        SourceRegistry::from_json_bytes(&valid)
            .unwrap()
            .to_json_bytes()
            .unwrap(),
        valid
    );

    let mut legacy_snapshot =
        serde_json::to_value(&registry.document().snapshots[0].snapshot).unwrap();
    legacy_snapshot["future"] = true.into();
    assert!(serde_json::from_value::<vlajmaji_lexicon::SnapshotIdentity>(legacy_snapshot).is_ok());

    for injection in 0..3 {
        let mut value: serde_json::Value = serde_json::from_slice(&valid).unwrap();
        match injection {
            0 => value["snapshots"][0]["snapshot"]["future"] = true.into(),
            1 => value["occurrences"][0]["occurrence"]["future"] = true.into(),
            2 => value["occurrences"][0]["occurrence"]["snapshot"]["future"] = true.into(),
            _ => unreachable!(),
        }
        let malformed = serde_json::to_vec(&value).unwrap();
        assert!(SourceRegistry::from_json_bytes(&malformed).is_err());
        assert!(serde_json::from_slice::<SourceRegistry>(&malformed).is_err());
    }
}

#[test]
fn phantom_occurrence_binding_is_rejected_against_the_supplied_lexicon() {
    let (lexicon, mut document) = fixture();
    let mut phantom = serde_json::to_value(&document.occurrences[0].occurrence).unwrap();
    phantom["locator"] = "/not-a-captured-locator".into();
    let phantom: vlajmaji_lexicon::RecordIdentity = serde_json::from_value(phantom).unwrap();
    let expected = format!(
        "{}:{}:{}",
        phantom.snapshot().namespace(),
        phantom.snapshot().sha256(),
        phantom.locator()
    );
    document.occurrences.push(OccurrenceAttribution {
        occurrence: phantom,
        contributor: None,
        native_entry_id: None,
        native_assertion_id: None,
    });
    let registry = SourceRegistry::from_document(document).unwrap();
    assert!(matches!(
        registry.validate_lexicon(&lexicon),
        Err(RegistryError::OccurrenceNotFound(found)) if found == expected
    ));
    assert!(matches!(
        registry.attributed_records(&lexicon),
        Err(RegistryError::OccurrenceNotFound(found)) if found == expected
    ));
}

#[test]
fn invalid_documents_updates_and_deserialization_cannot_bypass_invariants() {
    let (_lexicon, document) = fixture();
    let registry = SourceRegistry::from_document(document.clone()).unwrap();
    let before = registry.to_json_bytes().unwrap();

    let mut collision = document.clone();
    collision.aliases.extend([
        SourceAlias {
            source: SourceId::new("src:a").unwrap(),
            text: "CLASH".into(),
            status: AliasStatus::Active,
            evidence: None,
        },
        SourceAlias {
            source: SourceId::new("src:b").unwrap(),
            text: "CLASH".into(),
            status: AliasStatus::Active,
            evidence: None,
        },
    ]);
    assert!(matches!(
        SourceRegistry::from_document(collision.clone()),
        Err(RegistryError::AliasCollision { .. })
    ));
    assert!(
        serde_json::from_slice::<SourceRegistry>(&serde_json::to_vec(&collision).unwrap()).is_err()
    );

    let mut shadow = document.clone();
    shadow.aliases.push(SourceAlias {
        source: SourceId::new("src:a").unwrap(),
        text: "src:b".into(),
        status: AliasStatus::Retired,
        evidence: None,
    });
    assert!(matches!(
        SourceRegistry::from_document(shadow),
        Err(RegistryError::AliasCollision { .. })
    ));

    let mut unsupported = document.clone();
    unsupported.version = 9;
    assert!(matches!(
        SourceRegistry::from_document(unsupported.clone()),
        Err(RegistryError::UnsupportedVersion(9))
    ));
    assert!(matches!(
        SourceRegistry::from_json_bytes(&serde_json::to_vec(&unsupported).unwrap()),
        Err(RegistryError::UnsupportedVersion(9))
    ));
    assert!(SourceRegistry::from_json_bytes(b"{").is_err());
    assert!(SourceRegistry::from_json_bytes(&[0xff]).is_err());
    assert!(SourceRegistry::from_json_bytes(br#"{"version":1,"version":1}"#).is_err());
    assert!(SourceRegistry::from_json_bytes(br#"{"version":1,"unexpected":true}"#).is_err());
    assert!(serde_json::from_slice::<SourceRegistry>(br#"{"version":1,"sources":[],"contributors":[],"distributors":[],"aliases":[],"lineage":[],"snapshots":[],"occurrences":[],"future":1}"#).is_err());

    let mut unknown_contributor = document.clone();
    unknown_contributor.occurrences[0].contributor =
        Some(ContributorId::new("person:missing").unwrap());
    assert!(matches!(
        SourceRegistry::from_document(unknown_contributor),
        Err(RegistryError::UnknownContributor(_))
    ));
    let mut unknown_distributor = document.clone();
    unknown_distributor.snapshots[0].distributor =
        Some(DistributorId::new("dist:missing").unwrap());
    assert!(matches!(
        SourceRegistry::from_document(unknown_distributor),
        Err(RegistryError::UnknownDistributor(_))
    ));
    let mut bad_lineage = document.clone();
    bad_lineage.lineage.push(SourceLineage {
        source: SourceId::new("src:a").unwrap(),
        upstream: LineageTarget::Known(SourceId::new("src:absent").unwrap()),
        evidence: Some("evidence".into()),
    });
    assert!(matches!(
        SourceRegistry::from_document(bad_lineage),
        Err(RegistryError::InvalidLineage { .. })
    ));
    let mut self_lineage = document.clone();
    self_lineage.lineage.push(SourceLineage {
        source: SourceId::new("src:a").unwrap(),
        upstream: LineageTarget::Known(SourceId::new("src:a").unwrap()),
        evidence: Some("evidence".into()),
    });
    assert!(matches!(
        SourceRegistry::from_document(self_lineage),
        Err(RegistryError::InvalidLineage { .. })
    ));
    let mut no_evidence = document.clone();
    no_evidence.sources[0].abbreviation_kind = AbbreviationKind::Conventional;
    assert!(matches!(
        SourceRegistry::from_document(no_evidence),
        Err(RegistryError::InvalidValue { .. })
    ));
    let mut dangling = document.clone();
    dangling.occurrences[0].occurrence = dangling.occurrences[0].occurrence.clone();
    dangling.snapshots.clear();
    assert!(matches!(
        SourceRegistry::from_document(dangling),
        Err(RegistryError::DanglingOccurrence(_))
    ));

    let mut update = registry.clone();
    let mut invalid_update = update.document();
    invalid_update.sources[0].display_name = "".into();
    assert!(matches!(
        update.apply_document(invalid_update),
        Err(RegistryError::InvalidValue { .. })
    ));
    assert_eq!(update.to_json_bytes().unwrap(), before);
    let mut removed = update.document();
    removed.snapshots.pop();
    assert!(matches!(
        update.apply_document(removed),
        Err(RegistryError::ProtectedHistory(_))
    ));
    assert_eq!(update.to_json_bytes().unwrap(), before);
    let mut changed = update.document();
    changed.snapshots[0].source = SourceId::new("src:b").unwrap();
    assert!(matches!(
        update.apply_document(changed),
        Err(RegistryError::ProtectedHistory(_))
    ));
    assert_eq!(update.to_json_bytes().unwrap(), before);
}

#[test]
fn attribution_requires_explicit_legacy_binding_but_keeps_empty_metadata() {
    let mut legacy = Lexicon::new();
    let empty = SourceMetadata::new(
        "",
        "",
        SourceFormat::LaLojbanJson,
        "",
        "",
        None,
        TimePrecision::Unknown,
    );
    legacy
        .import_source(empty, br#"[{"word":"x","definition":"d"}]"#, None)
        .unwrap();
    assert_eq!(
        legacy
            .source_metadata(legacy.snapshots()[0])
            .unwrap()
            .namespace(),
        ""
    );
    let registry = SourceRegistry::new();
    assert!(matches!(
        registry.validate_lexicon(&legacy),
        Err(RegistryError::UnboundSnapshot(_))
    ));
    assert!(matches!(
        registry.attribute(&legacy.records()[0]),
        Err(RegistryError::UnboundSnapshot(_))
    ));

    let mut explicit = registry.document();
    explicit.sources.push(SourceRecord {
        id: SourceId::new("legacy:explicit").unwrap(),
        display_name: "Explicit legacy source".into(),
        abbreviation: "LEG".into(),
        abbreviation_kind: AbbreviationKind::ProjectAssigned,
        abbreviation_evidence: None,
        origin: SourceOrigin::Upstream,
    });
    explicit.snapshots.push(SnapshotAttribution {
        snapshot: legacy.snapshots()[0].clone(),
        source: SourceId::new("legacy:explicit").unwrap(),
        distributor: None,
    });
    let explicit = SourceRegistry::from_document(explicit).unwrap();
    assert_eq!(
        explicit.attributed_records(&legacy).unwrap()[0]
            .source()
            .id
            .as_str(),
        "legacy:explicit"
    );
}
