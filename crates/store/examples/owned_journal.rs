//! Generate a small owned-dictionary seed/edit pair and exercise the public journal API.
//!
//! Usage: `cargo run -p vlajmaji-store --example owned_journal -- INPUTS_DIR JOURNAL_ROOT`

use std::{
    env,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

use vlajmaji_lexicon::{
    AbbreviationKind, AssertionRecord, CaptureMetadata, CaptureRef, CollectionRecord,
    ComponentRecord, ContentNotice, ContributorId, ContributorRecord, Dictionary,
    DictionaryDocument, DictionaryMetadata, Eligibility, EntityId, EntryRecord, NativeEncoding,
    NativeField, OccurrenceAttribution, OccurrenceRef, OwnedHead, OwnedRevisionRecord, Presence,
    RawCapture, RawOccurrence, RedistributionStatus, RegistryDocument, ReleaseAccounting,
    ReleaseRecord, SnapshotAttribution, SourceId, SourceOrigin, SourceRecord, SourceRegistry,
    TextValue, TimePrecision,
};
use vlajmaji_store::{OwnedJournal, OwnedJournalReader};

fn value(text: &str) -> TextValue {
    TextValue {
        text: text.into(),
        language: Presence::Missing,
        markup: Presence::Missing,
    }
}

fn release(
    id: &str,
    component: &EntityId,
    captures: Vec<CaptureRef>,
    assertions: Vec<EntityId>,
) -> ReleaseRecord {
    let occurrence_count = assertions.len() as u64;
    ReleaseRecord {
        id: EntityId::new(id).expect("example release ID"),
        component: component.clone(),
        distributor: None,
        upstream_revision: Presence::Missing,
        distributor_revision: Presence::Missing,
        captured_at: Presence::Missing,
        time_precision: TimePrecision::Unknown,
        schema_version: 1,
        normalizer: "owned-example".into(),
        normalizer_version: "1".into(),
        captures,
        assertions,
        accounting: ReleaseAccounting {
            input_occurrences: Some(occurrence_count),
            normalized_occurrences: Some(occurrence_count),
            query_excluded_occurrences: Some(0),
            unit: "occurrences".into(),
        },
        diagnostics: Vec::new(),
        selected: true,
        availability: vlajmaji_lexicon::Availability::Available,
        content_notices: vec![ContentNotice {
            text: Presence::Value("example data only; no licence grant".into()),
            locator: Presence::Missing,
            license: Presence::Missing,
        }],
        redistribution: RedistributionStatus::Held,
        redistribution_evidence: Vec::new(),
    }
}

fn genesis() -> Dictionary {
    let source = SourceId::new("owned-example").expect("example source ID");
    let editor = ContributorId::new("editor-example").expect("example editor ID");
    let collection = EntityId::new("owned-collection").expect("example collection ID");
    let component = EntityId::new("owned-component").expect("example component ID");
    let revision = EntityId::new("owned-r0").expect("example revision ID");
    let registry = SourceRegistry::from_document(RegistryDocument {
        sources: vec![SourceRecord {
            id: source.clone(),
            display_name: "Personal dictionary".into(),
            abbreviation: "OWN".into(),
            abbreviation_kind: AbbreviationKind::ProjectAssigned,
            abbreviation_evidence: Some("Example-assigned selector".into()),
            origin: SourceOrigin::OwnedDraft,
        }],
        contributors: vec![ContributorRecord {
            id: editor.clone(),
            display_name: "Dictionary editor".into(),
        }],
        ..RegistryDocument::default()
    })
    .expect("valid example registry");
    let mut metadata = DictionaryMetadata::default();
    metadata.collections.push(CollectionRecord {
        id: collection.clone(),
        source: source.clone(),
        display_name: "My dictionary".into(),
        description: Presence::Value(value("An explicitly owned personal dictionary")),
    });
    metadata.components.push(ComponentRecord {
        id: component.clone(),
        collection,
        native_name: "owned lexical entries".into(),
        languages: vec!["jbo".into()],
    });
    metadata.releases.push(release(
        "owned-release-r0",
        &component,
        Vec::new(),
        Vec::new(),
    ));
    metadata.owned_revisions.push(OwnedRevisionRecord {
        id: revision.clone(),
        source: source.clone(),
        parent: None,
        editor,
        timestamp: Presence::Missing,
        time_precision: TimePrecision::Unknown,
        assertions: Vec::new(),
        tombstones: Vec::new(),
        rationale: Presence::Value(value("Start with an empty active set")),
    });
    metadata.owned_heads.push(OwnedHead { source, revision });
    Dictionary::from_document(DictionaryDocument {
        registry,
        metadata,
        ..DictionaryDocument::default()
    })
    .expect("valid owned genesis")
}

fn edit(seed: &Dictionary) -> Dictionary {
    let mut document = seed.document();
    let source = SourceId::new("owned-example").unwrap();
    let editor = ContributorId::new("editor-example").unwrap();
    let component = document.metadata.components[0].id.clone();
    let capture_bytes = br#"{"word":"farvi","definitions":["grow","develop"]}"#.to_vec();
    let occurrences = [
        RawOccurrence {
            locator: "/a".into(),
            fields: vec![NativeField {
                name: "definition".into(),
                encoding: NativeEncoding::Text,
                raw_value: "grow".into(),
            }],
            raw_line: Some("farvi — grow".into()),
        },
        RawOccurrence {
            locator: "/b".into(),
            fields: vec![NativeField {
                name: "definition".into(),
                encoding: NativeEncoding::Text,
                raw_value: "develop".into(),
            }],
            raw_line: Some("farvi — develop".into()),
        },
    ];
    let capture = RawCapture::new(
        "owned-example-capture",
        CaptureMetadata {
            source_name: "Example input".into(),
            format_label: "application/json".into(),
            locator: "seed/edit example".into(),
            revision: Presence::Missing,
            captured_at: Presence::Missing,
            time_precision: TimePrecision::Unknown,
            notice_locator: Presence::Missing,
        },
        capture_bytes,
        occurrences.to_vec(),
    );
    let capture_ref = capture.identity.clone();
    document.captures.push(capture);
    let refs = ["/a", "/b"].map(|locator| OccurrenceRef {
        capture: capture_ref.clone(),
        locator: locator.into(),
    });
    let mut registry = document.registry.document();
    registry.snapshots.push(SnapshotAttribution {
        snapshot: capture_ref
            .to_snapshot_identity()
            .expect("snapshot identity"),
        source: source.clone(),
        distributor: None,
    });
    for (index, occurrence) in refs.iter().enumerate() {
        registry.occurrences.push(OccurrenceAttribution {
            occurrence: occurrence.to_record_identity().expect("record identity"),
            contributor: Some(editor.clone()),
            native_entry_id: Some("farvi".into()),
            native_assertion_id: Some(format!("example-{}", if index == 0 { "a" } else { "b" })),
        });
    }
    document.registry = SourceRegistry::from_document(registry).expect("valid example registry");

    let entry = EntityId::new("owned-entry-farvi").unwrap();
    let assertion_a = EntityId::new("owned-assertion-a").unwrap();
    let assertion_b = EntityId::new("owned-assertion-b").unwrap();
    document.metadata.entries.push(EntryRecord {
        id: entry.clone(),
        source,
        spelling: Presence::Value("farvi".into()),
        language: Presence::Value("jbo".into()),
        script: Presence::Missing,
        native_word_class: Presence::Missing,
        selmaho: Presence::Null,
        mapped_pos: Presence::Missing,
    });
    for (id, occurrence, definition) in [
        (assertion_a.clone(), refs[0].clone(), "grow"),
        (assertion_b.clone(), refs[1].clone(), "develop"),
    ] {
        document.metadata.assertions.push(AssertionRecord {
            id,
            entry: entry.clone(),
            occurrence,
            native_sense_id: Presence::Missing,
            language: Presence::Value("en".into()),
            definition: Presence::Value(value(definition)),
            source_score: Presence::Null,
            timestamps: Vec::new(),
            eligibility: Eligibility::Lexical,
        });
    }
    document.metadata.releases.push(release(
        "owned-release-r1",
        &component,
        vec![capture_ref],
        vec![assertion_a.clone(), assertion_b.clone()],
    ));
    let next = EntityId::new("owned-r1").unwrap();
    document.metadata.catalogue_revision = 1;
    document.metadata.owned_revisions.push(OwnedRevisionRecord {
        id: next.clone(),
        source: SourceId::new("owned-example").unwrap(),
        parent: Some(EntityId::new("owned-r0").unwrap()),
        editor,
        timestamp: Presence::Missing,
        time_precision: TimePrecision::Unknown,
        assertions: vec![assertion_a, assertion_b],
        tombstones: Vec::new(),
        rationale: Presence::Value(value("Add two source-distinct definitions")),
    });
    document.metadata.owned_heads[0].revision = next;
    Dictionary::from_document(document).expect("valid complete edit")
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args_os().skip(1);
    let inputs_dir = PathBuf::from(args.next().ok_or("expected INPUTS_DIR and JOURNAL_ROOT")?);
    let journal_root = PathBuf::from(args.next().ok_or("expected JOURNAL_ROOT")?);
    if args.next().is_some() {
        return Err("too many arguments".into());
    }
    fs::create_dir(&inputs_dir)?;
    let seed = genesis();
    let revised = edit(&seed);
    let seed_bytes = seed.to_json_bytes()?;
    let edit_bytes = revised.to_json_bytes()?;
    write_new(&inputs_dir.join("seed.json"), &seed_bytes)?;
    write_new(&inputs_dir.join("edit.json"), &edit_bytes)?;

    let (mut journal, created) = OwnedJournal::create(&journal_root, &seed)?;
    println!("{}", serde_json::to_string_pretty(&created)?);
    let committed = journal.commit(&created.head.revision.id, &revised)?;
    println!("{}", serde_json::to_string_pretty(&committed)?);
    drop(journal);

    let reader = OwnedJournalReader::open(&journal_root)?;
    println!("{}", serde_json::to_string_pretty(&reader.head()?)?);
    println!("{}", serde_json::to_string_pretty(&reader.history()?)?);
    println!("{}", serde_json::to_string_pretty(&reader.show_head()?)?);
    println!(
        "{}",
        serde_json::to_string_pretty(&reader.show(&created.head.revision.id)?)?
    );
    Ok(())
}
