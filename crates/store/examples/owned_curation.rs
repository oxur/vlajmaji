//! Generate a tiny owned edit with a three-member Sense and reviewed Link.
//!
//! Usage: `cargo run -p vlajmaji-store --example owned_curation -- INPUTS_DIR JOURNAL_ROOT OUTPUT`
//!
//! The example writes `seed.json`, `curation.json`, and `context.json` into a
//! fresh input directory. `seed.json` starts at r0; the journal advances to r1
//! before `curation.json` supplies r2, its explicit context pin, and its
//! reviewed records. It first materializes without the context so unavailable
//! and dependent-link results are visible,
//! then supplies the exact pinned context and writes a normalized view.

use std::{
    env,
    error::Error,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

use vlajmaji_lexicon::{
    AbbreviationKind, Availability, CatalogTarget, CollectionRecord, ComponentRecord,
    ContentNotice, ContributorId, ContributorRecord, Dictionary, DictionaryDocument,
    DictionaryMetadata, Direction, EntityId, EntryRecord, LaLojbanImportOptions, LocalRef,
    OccurrenceRef, OwnedContextInput, OwnedContextPin, OwnedCurationData, OwnedCurationDocument,
    OwnedCurationKind, OwnedCurationRecord, OwnedCurationRevision, OwnedHead, OwnedPinnedTarget,
    OwnedRevisionRecord, OwnedSenseMember, Presence, RedistributionStatus, RegistryDocument,
    ReleaseAccounting, ReleaseRecord, SourceId, SourceOrigin, SourceRecord, SourceRegistry,
    TextValue, TimePrecision,
};
use vlajmaji_store::{OwnedJournal, OwnedJournalReader};

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
    let source = SourceId::new("curation-example-owner")?;
    let editor = ContributorId::new("curation-example-editor")?;
    let collection = EntityId::new("curation-example-collection")?;
    let component = EntityId::new("curation-example-component")?;
    let revision = EntityId::new("curation-example-r0")?;
    let registry = SourceRegistry::from_document(RegistryDocument {
        sources: vec![SourceRecord {
            id: source.clone(),
            display_name: "Personal curation example".into(),
            abbreviation: "OWN".into(),
            abbreviation_kind: AbbreviationKind::ProjectAssigned,
            abbreviation_evidence: Some("Example-assigned selector".into()),
            origin: SourceOrigin::OwnedDraft,
        }],
        contributors: vec![ContributorRecord {
            id: editor.clone(),
            display_name: "Example editor".into(),
        }],
        ..RegistryDocument::default()
    })?;
    let mut metadata = DictionaryMetadata::default();
    metadata.collections.push(CollectionRecord {
        id: collection.clone(),
        source: source.clone(),
        display_name: "Personal dictionary".into(),
        description: Presence::Value(value("A tiny owned base")),
    });
    metadata.components.push(ComponentRecord {
        id: component.clone(),
        collection,
        native_name: "owned entries".into(),
        languages: vec!["jbo".into()],
    });
    metadata.entries.push(EntryRecord {
        id: EntityId::new("curation-example-entry")?,
        source: source.clone(),
        spelling: Presence::Value("farvi".into()),
        language: Presence::Value("jbo".into()),
        script: Presence::Missing,
        native_word_class: Presence::Missing,
        selmaho: Presence::Missing,
        mapped_pos: Presence::Missing,
    });
    metadata.releases.push(ReleaseRecord {
        id: EntityId::new("curation-example-release-r0")?,
        component,
        distributor: None,
        upstream_revision: Presence::Missing,
        distributor_revision: Presence::Missing,
        captured_at: Presence::Missing,
        time_precision: TimePrecision::Unknown,
        schema_version: 1,
        normalizer: "owned-curation-example".into(),
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
            text: Presence::Value("synthetic example; no licence grant".into()),
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
        rationale: Presence::Value(value("Start with one owned entry")),
    });
    metadata.owned_heads.push(OwnedHead { source, revision });
    Ok(Dictionary::from_document(DictionaryDocument {
        registry,
        metadata,
        ..DictionaryDocument::default()
    })?)
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
    let revision = EntityId::new(format!("curation-example-r{next_sequence}"))?;
    document.metadata.catalogue_revision = next_sequence;
    document.metadata.releases.push(ReleaseRecord {
        id: EntityId::new(format!("curation-example-release-r{next_sequence}"))?,
        ..document.metadata.releases[0].clone()
    });
    document.metadata.owned_revisions.push(OwnedRevisionRecord {
        id: revision.clone(),
        source: previous.source.clone(),
        parent: Some(previous.id),
        editor: previous.editor,
        timestamp: Presence::Missing,
        time_precision: TimePrecision::Unknown,
        assertions: vec![],
        tombstones: vec![],
        rationale: Presence::Value(value("Review one external definition")),
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
    let pin =
        OwnedContextPin::from_dictionary(EntityId::new("curation-example-context")?, context)?;
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

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args_os().skip(1).map(PathBuf::from);
    let inputs = args
        .next()
        .ok_or("usage: owned_curation INPUTS_DIR JOURNAL_ROOT OUTPUT")?;
    let root = args
        .next()
        .ok_or("usage: owned_curation INPUTS_DIR JOURNAL_ROOT OUTPUT")?;
    let output = args
        .next()
        .ok_or("usage: owned_curation INPUTS_DIR JOURNAL_ROOT OUTPUT")?;
    if args.next().is_some() {
        return Err("usage: owned_curation INPUTS_DIR JOURNAL_ROOT OUTPUT".into());
    }
    fs::create_dir(&inputs)?;

    let genesis = own_genesis()?;
    let seed = next_owned_revision(&genesis)?;
    let candidate = next_owned_revision(&seed)?;
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
    let (_, second) = target_from_context(&context, 1)?;
    let (_, third) = target_from_context(&context, 2)?;
    let revision = candidate.metadata().owned_revisions[2].id.clone();
    let sense_id = EntityId::new("curation-example-sense")?;
    let sense = OwnedCurationRecord {
        id: sense_id.clone(),
        revision: revision.clone(),
        editor: ContributorId::new("curation-example-editor")?,
        rationale: value("Group three reviewed occurrences from the supplied synthetic context"),
        kind: OwnedCurationKind::Sense {
            members: vec![
                OwnedSenseMember::External(first.clone()),
                OwnedSenseMember::External(second),
                OwnedSenseMember::External(third),
            ],
        },
    };
    let link_id = EntityId::new("curation-example-link")?;
    let link = OwnedCurationRecord {
        id: link_id.clone(),
        revision: revision.clone(),
        editor: ContributorId::new("curation-example-editor")?,
        rationale: value("Link the reviewed Sense to its first occurrence"),
        kind: OwnedCurationKind::Link {
            origin: LocalRef::EditorialSense(sense_id),
            direction: Direction::Forward,
            target: first,
        },
    };
    let document = OwnedCurationDocument::from_data(OwnedCurationData {
        version: 1,
        dictionary: candidate,
        contexts: vec![pin.clone()],
        records: vec![sense, link],
        revisions: vec![
            OwnedCurationRevision {
                revision: genesis.metadata().owned_revisions[0].id.clone(),
                active: vec![],
                tombstones: vec![],
            },
            OwnedCurationRevision {
                revision: seed.metadata().owned_revisions[1].id.clone(),
                active: vec![],
                tombstones: vec![],
            },
            OwnedCurationRevision {
                revision,
                active: vec![EntityId::new("curation-example-sense")?, link_id],
                tombstones: vec![],
            },
        ],
    })?;

    let seed_path = inputs.join("seed.json");
    let r1_path = inputs.join("r1.json");
    let curation_path = inputs.join("curation.json");
    let context_path = inputs.join("context.json");
    write_new(&seed_path, &genesis.to_json_bytes()?)?;
    write_new(&r1_path, &seed.to_json_bytes()?)?;
    write_new(&curation_path, &document.to_json_bytes()?)?;
    write_new(
        &context_path,
        OwnedContextInput {
            id: pin.id.clone(),
            dictionary: context.clone(),
        }
        .to_json_bytes()?
        .as_slice(),
    )?;

    let genesis_head = genesis.metadata().owned_revisions[0].id.clone();
    let expected_head = seed.metadata().owned_revisions[1].id.clone();
    let (mut journal, _) = OwnedJournal::create(&root, &genesis)?;
    journal.commit(&genesis_head, &seed)?;
    journal.migrate_curation(&expected_head)?;
    journal.commit_curated(&expected_head, &document)?;

    // Show returns the retained definition; it does not resolve its target.
    let reader = OwnedJournalReader::open(&root)?;
    let retained = reader.curation_show_head()?;
    if retained != document {
        return Err("stored curation document differs from submitted input".into());
    }
    drop(reader);

    let unavailable = document.materialize(Vec::new())?;
    for resolution in &unavailable.resolutions {
        println!(
            "{}: {:?} ({:?})",
            resolution.record, resolution.status, resolution.reason
        );
    }
    let supplied = OwnedContextInput::from_json_bytes(&fs::read(&context_path)?)?;
    let materialized = document.materialize(vec![supplied])?;
    if !materialized
        .resolutions
        .iter()
        .all(|row| row.projected && row.status == vlajmaji_lexicon::OwnedCurationStatus::Resolved)
    {
        return Err("the exact pinned context did not resolve every active record".into());
    }
    write_new(&output, &materialized.dictionary.to_json_bytes()?)?;
    println!(
        "wrote normalized view to {}; inputs: {}, {}, {}, {}",
        output.display(),
        seed_path.display(),
        r1_path.display(),
        curation_path.display(),
        context_path.display()
    );
    Ok(())
}
