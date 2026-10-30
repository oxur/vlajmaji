//! Generate a sealed owned snapshot with a three-member Sense and reviewed Link.
//!
//! Usage: `cargo run -p vlajmaji-store --example owned_snapshot -- INPUTS_DIR JOURNAL_ROOT IMPORTED_ROOT`
//!
//! The example writes base-history, curation, context, snapshot, and seal inputs into a
//! fresh input directory. `seed.json` starts at r0; the journal advances to r1
//! before `r2-curation.json` supplies r2, its explicit context pin, and its
//! reviewed records. It advances a schema-two journal through r0..r4, seals
//! its full history, and imports that snapshot into a separate fresh root.
//! The documented CLI journey separately demonstrates zero-context and
//! pinned-context materialization.

use std::{
    env,
    error::Error,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

use vlajmaji_lexicon::{
    AbbreviationKind, AssertionRecord, Availability, CaptureMetadata, CatalogTarget,
    CollectionRecord, ComponentRecord, ContentNotice, ContributorId, ContributorRecord, Dictionary,
    DictionaryDocument, DictionaryMetadata, Direction, Eligibility, EntityId, EntryRecord,
    LaLojbanImportOptions, LocalRef, NativeEncoding, NativeField, OccurrenceAttribution,
    OccurrenceRef, OwnedContextInput, OwnedContextPin, OwnedCurationData, OwnedCurationDocument,
    OwnedCurationKind, OwnedCurationRecord, OwnedCurationRevision, OwnedHead, OwnedPinnedTarget,
    OwnedRevisionRecord, OwnedSenseMember, Presence, RawCapture, RawOccurrence,
    RedistributionStatus, RegistryDocument, ReleaseAccounting, ReleaseRecord, SnapshotAttribution,
    SourceId, SourceOrigin, SourceRecord, SourceRegistry, TextValue, TimePrecision,
};
use vlajmaji_lexicon::{load_owned_snapshot, seal_owned_snapshot};
use vlajmaji_store::{OwnedImportOutcome, OwnedJournal, OwnedJournalError, OwnedJournalReader};

const CONTEXT_EXPORT: &[u8] = br#"[
  {"word":"coi","word_type":"gismu","definition":"greeting","definition_id":7,"user":{"username":"alice"}},
  {"word":"coi","word_type":"gismu","definition":"salutation","definition_id":8,"user":{"username":"bob"}},
  {"word":"xamgu","word_type":"gismu","definition":"good","definition_id":9,"user":{"username":"carol"}}
]"#;

fn value(text: &str) -> TextValue {
    TextValue {
        text: text.into(),
        language: Presence::Missing,
        markup: Presence::Missing,
    }
}

fn own_genesis() -> Result<Dictionary, Box<dyn Error>> {
    let source = SourceId::new("owned-example")?;
    let editor = ContributorId::new("editor-example")?;
    let collection = EntityId::new("owned-collection")?;
    let component = EntityId::new("owned-component")?;
    let revision = EntityId::new("owned-r0")?;
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
    })?;
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
    metadata.releases.push(ReleaseRecord {
        id: EntityId::new("owned-release-r0")?,
        component,
        distributor: None,
        upstream_revision: Presence::Missing,
        distributor_revision: Presence::Missing,
        captured_at: Presence::Missing,
        time_precision: TimePrecision::Unknown,
        schema_version: 1,
        normalizer: "owned-example".into(),
        normalizer_version: "1".into(),
        captures: vec![],
        assertions: vec![],
        accounting: ReleaseAccounting {
            input_occurrences: Some(0),
            normalized_occurrences: Some(0),
            query_excluded_occurrences: Some(0),
            unit: "occurrences".into(),
        },
        diagnostics: vec![],
        selected: true,
        availability: Availability::Available,
        content_notices: vec![ContentNotice {
            text: Presence::Value("example data only; no licence grant".into()),
            locator: Presence::Missing,
            license: Presence::Missing,
        }],
        redistribution: RedistributionStatus::Held,
        redistribution_evidence: vec![],
    });
    metadata.owned_revisions.push(OwnedRevisionRecord {
        id: revision.clone(),
        source: source.clone(),
        parent: None,
        editor,
        timestamp: Presence::Missing,
        time_precision: TimePrecision::Unknown,
        assertions: vec![],
        tombstones: vec![],
        rationale: Presence::Value(value("Start with an empty active set")),
    });
    metadata.owned_heads.push(OwnedHead { source, revision });
    Ok(Dictionary::from_document(DictionaryDocument {
        registry,
        metadata,
        ..DictionaryDocument::default()
    })?)
}

fn own_edit(seed: &Dictionary) -> Result<Dictionary, Box<dyn Error>> {
    let mut document = seed.document();
    let source = SourceId::new("owned-example")?;
    let editor = ContributorId::new("editor-example")?;
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
        br#"{"word":"farvi","definitions":["grow","develop"]}"#.to_vec(),
        vec![
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
        ],
    );
    let capture_ref = capture.identity.clone();
    document.captures.push(capture);
    let occurrences = ["/a", "/b"].map(|locator| OccurrenceRef {
        capture: capture_ref.clone(),
        locator: locator.into(),
    });
    let mut registry = document.registry.document();
    registry.snapshots.push(SnapshotAttribution {
        snapshot: capture_ref.to_snapshot_identity()?,
        source: source.clone(),
        distributor: None,
    });
    for (index, occurrence) in occurrences.iter().enumerate() {
        registry.occurrences.push(OccurrenceAttribution {
            occurrence: occurrence.to_record_identity()?,
            contributor: Some(editor.clone()),
            native_entry_id: Some("farvi".into()),
            native_assertion_id: Some(format!("example-{}", if index == 0 { "a" } else { "b" })),
        });
    }
    document.registry = SourceRegistry::from_document(registry)?;

    let entry = EntityId::new("owned-entry-farvi")?;
    let assertion_a = EntityId::new("owned-assertion-a")?;
    let assertion_b = EntityId::new("owned-assertion-b")?;
    document.metadata.entries.push(EntryRecord {
        id: entry.clone(),
        source: source.clone(),
        spelling: Presence::Value("farvi".into()),
        language: Presence::Value("jbo".into()),
        script: Presence::Missing,
        native_word_class: Presence::Missing,
        selmaho: Presence::Null,
        mapped_pos: Presence::Missing,
    });
    for (id, occurrence, definition) in [
        (assertion_a.clone(), occurrences[0].clone(), "grow"),
        (assertion_b.clone(), occurrences[1].clone(), "develop"),
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
    let mut release = document.metadata.releases[0].clone();
    release.id = EntityId::new("owned-release-r1")?;
    release.captures.push(capture_ref);
    release.assertions = vec![assertion_a.clone(), assertion_b.clone()];
    release.accounting.input_occurrences = Some(2);
    release.accounting.normalized_occurrences = Some(2);
    document.metadata.releases.push(release);

    let revision = EntityId::new("owned-r1")?;
    document.metadata.catalogue_revision = 1;
    document.metadata.owned_revisions.push(OwnedRevisionRecord {
        id: revision.clone(),
        source,
        parent: Some(EntityId::new("owned-r0")?),
        editor,
        timestamp: Presence::Missing,
        time_precision: TimePrecision::Unknown,
        assertions: vec![assertion_a, assertion_b],
        tombstones: Vec::new(),
        rationale: Presence::Value(value("Add two source-distinct definitions")),
    });
    document.metadata.owned_heads[0].revision = revision;
    Ok(Dictionary::from_document(document)?)
}

fn next_owned_revision(seed: &Dictionary) -> Result<Dictionary, Box<dyn Error>> {
    let mut document = seed.document();
    let previous = document
        .metadata
        .owned_revisions
        .last()
        .cloned()
        .ok_or("owned journal has no revision")?;
    let next_sequence = document.metadata.catalogue_revision + 1;
    let revision = EntityId::new(format!("owned-r{next_sequence}"))?;
    document.metadata.catalogue_revision = next_sequence;
    document.metadata.owned_revisions.push(OwnedRevisionRecord {
        id: revision.clone(),
        source: previous.source.clone(),
        parent: Some(previous.id),
        editor: previous.editor,
        timestamp: Presence::Missing,
        time_precision: TimePrecision::Unknown,
        assertions: previous.assertions,
        tombstones: previous.tombstones,
        rationale: Presence::Value(value(match next_sequence {
            2 => "Review explicit own and external occurrences",
            3 => "Explicit curation selection owned-r3",
            _ => "Explicit curation selection owned-r4",
        })),
    });
    document.metadata.owned_heads[0].revision = revision;
    Ok(Dictionary::from_document(document)?)
}

fn target_from_context(
    context: &Dictionary,
    assertion_index: usize,
) -> Result<(OwnedContextPin, OwnedPinnedTarget), Box<dyn Error>> {
    let assertion = context
        .metadata()
        .assertions
        .get(assertion_index)
        .ok_or("fixture assertion index is absent")?;
    let entry = context
        .metadata()
        .entries
        .iter()
        .find(|row| row.id == assertion.entry)
        .ok_or("fixture assertion has no entry")?;
    let release = context
        .metadata()
        .releases
        .iter()
        .find(|row| row.assertions.contains(&assertion.id))
        .ok_or("fixture assertion has no release")?;
    let component = context
        .metadata()
        .components
        .iter()
        .find(|row| row.id == release.component)
        .ok_or("fixture release has no component")?;
    let collection = context
        .metadata()
        .collections
        .iter()
        .find(|row| row.id == component.collection)
        .ok_or("fixture component has no collection")?;
    let identity = assertion.occurrence.to_record_identity()?;
    let attribution = context
        .registry()
        .as_document()
        .occurrences
        .iter()
        .find(|row| row.occurrence == identity)
        .ok_or("fixture assertion has no occurrence attribution")?;
    let pin = OwnedContextPin::from_dictionary(EntityId::new("cdc-context")?, context)?;
    Ok((
        pin.clone(),
        OwnedPinnedTarget {
            context: pin.id,
            target: CatalogTarget {
                source: collection.source.clone(),
                release: release.id.clone(),
                native_entry_id: attribution.native_entry_id.clone(),
                native_assertion_id: attribution.native_assertion_id.clone(),
                occurrence: Some(OccurrenceRef::from(&identity)),
            },
            entry: entry.id.clone(),
            assertion: assertion.id.clone(),
        },
    ))
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<(), Box<dyn Error>> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}

fn curation_for(
    dictionary: Dictionary,
    contexts: &[OwnedContextPin],
    records: &[OwnedCurationRecord],
    revisions: Vec<OwnedCurationRevision>,
) -> Result<OwnedCurationDocument, Box<dyn Error>> {
    Ok(OwnedCurationDocument::from_data(OwnedCurationData {
        version: 1,
        dictionary,
        contexts: contexts.to_vec(),
        records: records.to_vec(),
        revisions,
    })?)
}

fn write_dictionary(
    inputs: &Path,
    name: &str,
    dictionary: &Dictionary,
) -> Result<(), Box<dyn Error>> {
    write_new(&inputs.join(name), &dictionary.to_json_bytes()?)
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args_os().skip(1).map(PathBuf::from);
    let inputs = args
        .next()
        .ok_or("usage: owned_snapshot INPUTS_DIR JOURNAL_ROOT IMPORTED_ROOT")?;
    let root = args
        .next()
        .ok_or("usage: owned_snapshot INPUTS_DIR JOURNAL_ROOT IMPORTED_ROOT")?;
    let imported_root = args
        .next()
        .ok_or("usage: owned_snapshot INPUTS_DIR JOURNAL_ROOT IMPORTED_ROOT")?;
    if args.next().is_some() {
        return Err("usage: owned_snapshot INPUTS_DIR JOURNAL_ROOT IMPORTED_ROOT".into());
    }
    fs::create_dir(&inputs)?;

    let genesis = own_genesis()?;
    let r1 = own_edit(&genesis)?;
    let r2 = next_owned_revision(&r1)?;
    let context = vlajmaji_lexicon::import_la_lojban_export(
        CONTEXT_EXPORT,
        &LaLojbanImportOptions {
            revision: "0000000000000000000000000000000000000000".into(),
            expected_raw_sha256: "4667ccabc6b00fd874b95a4e12d0639fa7dfb85aa0fc1046e8108ef198332a15"
                .into(),
            raw_locator: Some("https://example.invalid/tiny/en.json".into()),
            notice_locator: Some("https://example.invalid/tiny/README.md".into()),
        },
    )?
    .dictionary()
    .clone();
    let (pin, first) = target_from_context(&context, 0)?;
    let sense_id = EntityId::new("cdc-reviewed-sense")?;
    let link_id = EntityId::new("cdc-reviewed-link")?;
    let editor = ContributorId::new("editor-example")?;
    let r2_id = r2.metadata().owned_revisions[2].id.clone();
    let records = vec![
        OwnedCurationRecord {
            id: sense_id.clone(),
            revision: r2_id.clone(),
            editor: editor.clone(),
            rationale: value("Review two retained owned assertions and one supplied occurrence"),
            kind: OwnedCurationKind::Sense {
                members: vec![
                    OwnedSenseMember::OwnedAssertion(EntityId::new("owned-assertion-a")?),
                    OwnedSenseMember::OwnedAssertion(EntityId::new("owned-assertion-b")?),
                    OwnedSenseMember::External(first.clone()),
                ],
            },
        },
        OwnedCurationRecord {
            id: link_id.clone(),
            revision: r2_id.clone(),
            editor,
            rationale: value("Link the reviewed sense to the pinned external occurrence"),
            kind: OwnedCurationKind::Link {
                origin: LocalRef::EditorialSense(sense_id.clone()),
                direction: Direction::Forward,
                target: first,
            },
        },
    ];
    let r0_id = genesis.metadata().owned_revisions[0].id.clone();
    let r1_id = r1.metadata().owned_revisions[1].id.clone();
    let doc_r0 = curation_for(
        genesis.clone(),
        &[],
        &[],
        vec![OwnedCurationRevision {
            revision: r0_id.clone(),
            active: vec![],
            tombstones: vec![],
        }],
    )?;
    let doc_r1 = curation_for(
        r1.clone(),
        &[],
        &[],
        vec![
            OwnedCurationRevision {
                revision: r0_id.clone(),
                active: vec![],
                tombstones: vec![],
            },
            OwnedCurationRevision {
                revision: r1_id.clone(),
                active: vec![],
                tombstones: vec![],
            },
        ],
    )?;
    let doc_r2 = curation_for(
        r2.clone(),
        std::slice::from_ref(&pin),
        &records,
        vec![
            OwnedCurationRevision {
                revision: r0_id.clone(),
                active: vec![],
                tombstones: vec![],
            },
            OwnedCurationRevision {
                revision: r1_id.clone(),
                active: vec![],
                tombstones: vec![],
            },
            OwnedCurationRevision {
                revision: r2_id.clone(),
                active: vec![sense_id.clone(), link_id.clone()],
                tombstones: vec![],
            },
        ],
    )?;

    let r3 = next_owned_revision(&r2)?;
    let r3_id = r3.metadata().owned_revisions[3].id.clone();
    let mut r3_history = doc_r2.as_data().revisions.clone();
    r3_history.push(OwnedCurationRevision {
        revision: r3_id.clone(),
        active: vec![sense_id.clone()],
        tombstones: vec![link_id.clone()],
    });
    let doc_r3 = curation_for(r3.clone(), std::slice::from_ref(&pin), &records, r3_history)?;

    let r4 = next_owned_revision(&r3)?;
    let r4_id = r4.metadata().owned_revisions[4].id.clone();
    let mut r4_history = doc_r3.as_data().revisions.clone();
    r4_history.push(OwnedCurationRevision {
        revision: r4_id.clone(),
        active: vec![sense_id, link_id],
        tombstones: vec![],
    });
    let doc_r4 = curation_for(r4.clone(), std::slice::from_ref(&pin), &records, r4_history)?;

    write_dictionary(&inputs, "seed.json", &genesis)?;
    write_dictionary(&inputs, "r1.json", &r1)?;
    write_dictionary(&inputs, "r2.json", &r2)?;
    write_dictionary(&inputs, "r3.json", &r3)?;
    write_dictionary(&inputs, "r4.json", &r4)?;
    write_dictionary(&inputs, "base.json", &r4)?;
    write_new(&inputs.join("r2-curation.json"), &doc_r2.to_json_bytes()?)?;
    write_new(&inputs.join("r3-curation.json"), &doc_r3.to_json_bytes()?)?;
    write_new(&inputs.join("r4-curation.json"), &doc_r4.to_json_bytes()?)?;
    write_new(
        &inputs.join("context.json"),
        &OwnedContextInput {
            id: pin.id.clone(),
            dictionary: context,
        }
        .to_json_bytes()?,
    )?;

    let r0_head = genesis.metadata().owned_revisions[0].id.clone();
    let r1_head = r1.metadata().owned_revisions[1].id.clone();
    let r2_head = r2.metadata().owned_revisions[2].id.clone();
    let r3_head = r3.metadata().owned_revisions[3].id.clone();
    let (mut journal, _) = OwnedJournal::create(&root, &genesis)?;
    journal.commit(&r0_head, &r1)?;
    let schema1_snapshot = {
        let reader = OwnedJournalReader::open(&root)?;
        reader.snapshot()?
    };
    journal.migrate_curation(&r1_head)?;
    journal.commit_curated(&r1_head, &doc_r2)?;
    journal.commit_curated(&r2_head, &doc_r3)?;
    journal.commit_curated(&r3_head, &doc_r4)?;

    let reader = OwnedJournalReader::open(&root)?;
    if reader.show(&r0_head)?.dictionary != genesis
        || reader.show(&r1_head)?.dictionary != r1
        || reader.show(&r2_head)?.dictionary != r2
        || reader.show(&r3_head)?.dictionary != r3
        || reader.show(&r4_id)?.dictionary != r4
    {
        return Err("journal history differs from generated input dictionaries".into());
    }
    if reader.curation_show(&r0_head)? != doc_r0
        || reader.curation_show(&r1_head)? != doc_r1
        || reader.curation_show(&r2_head)? != doc_r2
        || reader.curation_show(&r3_head)? != doc_r3
        || reader.curation_show(&r4_id)? != doc_r4
    {
        return Err("journal curation history differs from generated input documents".into());
    }
    let sealed = seal_owned_snapshot(&reader.snapshot()?)?;
    write_new(&inputs.join("snapshot.json"), &sealed.snapshot)?;
    write_new(&inputs.join("seal.json"), &sealed.seal)?;
    drop(reader);

    let (imported, report) = OwnedJournal::import_snapshot(
        &imported_root,
        &load_owned_snapshot(&sealed.snapshot, &sealed.seal)?,
    )?;
    if report.outcome != OwnedImportOutcome::Imported {
        return Err("fresh snapshot import was not reported as Imported".into());
    }
    let imported_reader = OwnedJournalReader::open(&imported_root)?;
    if imported_reader.snapshot()?.to_json_bytes()? != sealed.snapshot {
        return Err("imported journal does not reproduce the sealed snapshot".into());
    }
    drop(imported_reader);
    drop(imported);
    let (_, replay) = OwnedJournal::import_snapshot(
        &imported_root,
        &load_owned_snapshot(&sealed.snapshot, &sealed.seal)?,
    )?;
    if replay.outcome != OwnedImportOutcome::AlreadyPresent {
        return Err("exact snapshot replay was not recognized".into());
    }
    match OwnedJournal::import_snapshot(&imported_root, &schema1_snapshot) {
        Err(OwnedJournalError::ImportConflict(_)) => {}
        Ok(_) => return Err("a different valid history was not rejected as a conflict".into()),
        Err(error) => {
            return Err(
                format!("different valid history returned the wrong error: {error}").into(),
            );
        }
    }
    println!(
        "wrote schema-two history r0..r4 and its sealed snapshot to {}; imported {:?} at {}",
        root.display(),
        report.outcome,
        report.head
    );
    Ok(())
}
