//! Demonstrates exact source registration and occurrence attribution.

use vlajmaji_lexicon::{
    AbbreviationKind, AliasStatus, ContributorId, ContributorRecord, DistributorId,
    DistributorRecord, Lexicon, LineageTarget, OccurrenceAttribution, RegistryDocument,
    SnapshotAttribution, SourceAlias, SourceFormat, SourceId, SourceLineage, SourceMetadata,
    SourceOrigin, SourceRecord, SourceRegistry, TimePrecision,
};

fn metadata(namespace: &str, format: SourceFormat) -> SourceMetadata {
    SourceMetadata::new(
        namespace,
        "synthetic source fixture",
        format,
        format!("fixture://{namespace}"),
        "fixture-v1",
        None,
        TimePrecision::Unknown,
    )
}

fn snapshot(lexicon: &Lexicon, namespace: &str) -> vlajmaji_lexicon::SnapshotIdentity {
    lexicon
        .snapshots()
        .into_iter()
        .find(|identity| identity.namespace() == namespace)
        .cloned()
        .expect("fixture snapshot was imported")
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut lexicon = Lexicon::new();
    lexicon.import_source(
        metadata("captured:a:d1", SourceFormat::LaLojbanJson),
        br#"[{"word":"farvi","definition":"move forward"},{"word":"farvi","definition":"move forward"},{"word":"farvi","definition":"cause to advance"}]"#,
        None,
    )?;
    lexicon.import_source(
        metadata("captured:a:d2", SourceFormat::LensiskuJson),
        br#"[{"valsiword":"farvi","definition":"move forward"}]"#,
        None,
    )?;
    lexicon.import_source(
        metadata("captured:b:d1", SourceFormat::LaLojbanJson),
        br#"[{"word":"farvi","definition":"make progress"}]"#,
        None,
    )?;

    let source_a = SourceId::new("src:a")?;
    let source_b = SourceId::new("src:b")?;
    let distributor_one = DistributorId::new("dist:one")?;
    let distributor_two = DistributorId::new("dist:two")?;
    let contributor = ContributorId::new("person:one")?;
    let farvi: Vec<_> = lexicon
        .lookup_exact("farvi")
        .into_iter()
        .map(|record| record.identity().clone())
        .collect();
    assert_eq!(farvi.len(), 5);

    let registry = SourceRegistry::from_document(RegistryDocument {
        sources: vec![
            SourceRecord {
                id: source_a.clone(),
                display_name: "Zeta Dictionary".into(),
                abbreviation: "FX-A".into(),
                abbreviation_kind: AbbreviationKind::ProjectAssigned,
                abbreviation_evidence: None,
                origin: SourceOrigin::Upstream,
            },
            SourceRecord {
                id: source_b.clone(),
                display_name: "Alpha Dictionary".into(),
                abbreviation: "FX-B".into(),
                abbreviation_kind: AbbreviationKind::ProjectAssigned,
                abbreviation_evidence: None,
                origin: SourceOrigin::Upstream,
            },
            SourceRecord {
                id: SourceId::new("src:draft")?,
                display_name: "Local draft".into(),
                abbreviation: "DRAFT".into(),
                abbreviation_kind: AbbreviationKind::ProjectAssigned,
                abbreviation_evidence: None,
                origin: SourceOrigin::OwnedDraft,
            },
        ],
        contributors: vec![ContributorRecord {
            id: contributor.clone(),
            display_name: "Fixture Author".into(),
        }],
        distributors: vec![
            DistributorRecord {
                id: distributor_one.clone(),
                display_name: "Distributor One".into(),
            },
            DistributorRecord {
                id: distributor_two.clone(),
                display_name: "Distributor Two".into(),
            },
        ],
        aliases: vec![SourceAlias {
            source: source_a.clone(),
            text: "OLD".into(),
            status: AliasStatus::Active,
            evidence: Some("synthetic fixture alias".into()),
        }],
        lineage: vec![
            SourceLineage {
                source: source_a.clone(),
                upstream: LineageTarget::Unresolved("fixture://unresolved".into()),
                evidence: None,
            },
            SourceLineage {
                source: source_b.clone(),
                upstream: LineageTarget::Known(source_a.clone()),
                evidence: Some("synthetic fixture relationship".into()),
            },
        ],
        snapshots: vec![
            SnapshotAttribution {
                snapshot: snapshot(&lexicon, "captured:a:d1"),
                source: source_a.clone(),
                distributor: Some(distributor_one.clone()),
            },
            SnapshotAttribution {
                snapshot: snapshot(&lexicon, "captured:a:d2"),
                source: source_a.clone(),
                distributor: Some(distributor_two),
            },
            SnapshotAttribution {
                snapshot: snapshot(&lexicon, "captured:b:d1"),
                source: source_b,
                distributor: Some(distributor_one),
            },
        ],
        occurrences: vec![
            OccurrenceAttribution {
                occurrence: farvi[0].clone(),
                contributor: Some(contributor),
                native_entry_id: Some(String::new()),
                native_assertion_id: Some("upstream:farvi:1".into()),
            },
            OccurrenceAttribution {
                occurrence: farvi[1].clone(),
                contributor: None,
                native_entry_id: None,
                native_assertion_id: Some("upstream:farvi:1".into()),
            },
        ],
        ..RegistryDocument::default()
    })?;

    registry.validate_lexicon(&lexicon)?;
    assert_eq!(
        registry
            .resolve_source("OLD", vlajmaji_lexicon::AliasScope::ActiveOnly)?
            .id,
        source_a
    );
    let attributed = registry.attributed_records(&lexicon)?;
    let farvi_rows: Vec<_> = attributed
        .iter()
        .filter(|row| row.record().word() == Some("farvi"))
        .collect();
    assert_eq!(farvi_rows.len(), 5);
    assert_eq!(
        farvi_rows
            .iter()
            .filter(|row| row.source().id.as_str() == "src:a")
            .count(),
        4
    );
    assert_eq!(
        farvi_rows
            .iter()
            .filter(|row| row.source().id.as_str() == "src:b")
            .count(),
        1
    );
    assert_eq!(farvi_rows[0].native_entry_id(), Some(""));
    assert_eq!(
        farvi_rows[1].native_assertion_id(),
        Some("upstream:farvi:1")
    );

    println!(
        "{} captured farvi occurrences across two asserting sources",
        farvi_rows.len()
    );
    Ok(())
}
