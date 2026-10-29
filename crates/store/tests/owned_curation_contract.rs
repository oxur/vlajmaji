//! Persistence boundary tests for the explicit owned-curation schema.

use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

use vlajmaji_lexicon::{
    AbbreviationKind, Availability, CaptureRef, CatalogTarget, CollectionRecord, ComponentRecord,
    ContentNotice, ContributorId, ContributorRecord, Dictionary, DictionaryDocument,
    DictionaryMetadata, EntityId, EntryRecord, LocalRef, OccurrenceRef, OwnedContextPin,
    OwnedCurationData, OwnedCurationDocument, OwnedCurationKind, OwnedCurationRecord,
    OwnedCurationRevision, OwnedHead, OwnedRevisionRecord, Presence, RedistributionStatus,
    RegistryDocument, ReleaseAccounting, ReleaseRecord, SourceId, SourceOrigin, SourceRecord,
    SourceRegistry, TextValue, TimePrecision,
};
use vlajmaji_store::{OwnedJournal, OwnedJournalErrorCategory, OwnedMigrationOutcome};

static NEXT_ROOT: AtomicUsize = AtomicUsize::new(0);

fn fresh_root() -> PathBuf {
    let parent = std::env::temp_dir().join(format!(
        "vlajmaji-owned-curation-{}-{}",
        std::process::id(),
        NEXT_ROOT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&parent).expect("create exclusive test parent");
    parent.join("journal")
}

fn genesis() -> Dictionary {
    let source = SourceId::new("curated-source").unwrap();
    let editor = ContributorId::new("curated-editor").unwrap();
    let collection = EntityId::new("curated-collection").unwrap();
    let component = EntityId::new("curated-component").unwrap();
    let revision = EntityId::new("curated-r0").unwrap();
    let registry = SourceRegistry::from_document(RegistryDocument {
        sources: vec![SourceRecord {
            id: source.clone(),
            display_name: "Curated source".into(),
            abbreviation: "CUR".into(),
            abbreviation_kind: AbbreviationKind::ProjectAssigned,
            abbreviation_evidence: None,
            origin: SourceOrigin::OwnedDraft,
        }],
        contributors: vec![ContributorRecord {
            id: editor.clone(),
            display_name: "Editor".into(),
        }],
        ..RegistryDocument::default()
    })
    .unwrap();
    let mut metadata = DictionaryMetadata::default();
    metadata.entries.push(EntryRecord {
        id: EntityId::new("curated-entry").unwrap(),
        source: source.clone(),
        spelling: Presence::Value("curated".into()),
        language: Presence::Missing,
        script: Presence::Missing,
        native_word_class: Presence::Missing,
        selmaho: Presence::Missing,
        mapped_pos: Presence::Missing,
    });
    metadata.collections.push(CollectionRecord {
        id: collection.clone(),
        source: source.clone(),
        display_name: "Collection".into(),
        description: Presence::Missing,
    });
    metadata.components.push(ComponentRecord {
        id: component.clone(),
        collection,
        native_name: "component".into(),
        languages: vec!["jbo".into()],
    });
    metadata.releases.push(ReleaseRecord {
        id: EntityId::new("curated-release").unwrap(),
        component,
        distributor: None,
        upstream_revision: Presence::Missing,
        distributor_revision: Presence::Missing,
        captured_at: Presence::Missing,
        time_precision: TimePrecision::Unknown,
        schema_version: 1,
        normalizer: "test".into(),
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
            text: Presence::Value("fixture; no licence grant".into()),
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
        rationale: Presence::Value(TextValue {
            text: "genesis".into(),
            language: Presence::Missing,
            markup: Presence::Missing,
        }),
    });
    metadata.owned_heads.push(OwnedHead { source, revision });
    Dictionary::from_document(DictionaryDocument {
        registry,
        metadata,
        ..DictionaryDocument::default()
    })
    .unwrap()
}

fn next_revision(current: &Dictionary) -> Dictionary {
    let mut document = current.document();
    let previous = document.metadata.owned_revisions.last().unwrap().clone();
    let sequence = document.metadata.owned_revisions.len();
    let revision = EntityId::new(format!("curated-r{sequence}")).unwrap();
    document.metadata.catalogue_revision = sequence as u64;
    document.metadata.releases.push(ReleaseRecord {
        id: EntityId::new(format!("curated-release-r{sequence}")).unwrap(),
        ..document.metadata.releases[0].clone()
    });
    document.metadata.owned_revisions.push(OwnedRevisionRecord {
        id: revision.clone(),
        source: previous.source.clone(),
        parent: Some(previous.id),
        editor: previous.editor,
        timestamp: Presence::Missing,
        time_precision: TimePrecision::Unknown,
        assertions: previous.assertions,
        tombstones: Vec::new(),
        rationale: Presence::Value(TextValue {
            text: format!("revision {sequence}"),
            language: Presence::Missing,
            markup: Presence::Missing,
        }),
    });
    document.metadata.owned_heads[0].revision = revision;
    Dictionary::from_document(document).unwrap()
}

fn empty_curation(dictionary: Dictionary) -> OwnedCurationDocument {
    let revision = dictionary.metadata().owned_revisions[0].id.clone();
    OwnedCurationDocument::from_data(OwnedCurationData {
        version: 1,
        dictionary,
        contexts: vec![],
        records: vec![],
        revisions: vec![OwnedCurationRevision {
            revision,
            active: vec![],
            tombstones: vec![],
        }],
    })
    .unwrap()
}

fn linked_curation(current: &Dictionary) -> OwnedCurationDocument {
    let candidate = next_revision(current);
    let source = SourceId::new("external-source").unwrap();
    let context = OwnedContextPin {
        id: EntityId::new("external-context").unwrap(),
        model_sha256: "a".repeat(64),
        source_ids: vec![source.clone()],
        release_ids: vec![EntityId::new("external-release").unwrap()],
    };
    let record = OwnedCurationRecord {
        id: EntityId::new("curated-link").unwrap(),
        revision: candidate.metadata().owned_revisions[1].id.clone(),
        editor: ContributorId::new("curated-editor").unwrap(),
        rationale: TextValue {
            text: "retained pending its context".into(),
            language: Presence::Missing,
            markup: Presence::Missing,
        },
        kind: OwnedCurationKind::Link {
            origin: LocalRef::Entry(EntityId::new("curated-entry").unwrap()),
            direction: vlajmaji_lexicon::Direction::Forward,
            target: vlajmaji_lexicon::OwnedPinnedTarget {
                context: context.id.clone(),
                target: CatalogTarget {
                    source,
                    release: EntityId::new("external-release").unwrap(),
                    native_entry_id: None,
                    native_assertion_id: None,
                    occurrence: Some(OccurrenceRef {
                        capture: CaptureRef {
                            namespace: "external:capture".into(),
                            sha256: "b".repeat(64),
                        },
                        locator: "/row/1".into(),
                    }),
                },
                entry: EntityId::new("external-entry").unwrap(),
                assertion: EntityId::new("external-assertion").unwrap(),
            },
        },
    };
    let mut second_record = record.clone();
    second_record.id = EntityId::new("curated-link-second").unwrap();
    second_record.rationale.text = "second retained pending link".into();
    OwnedCurationDocument::from_data(OwnedCurationData {
        version: 1,
        dictionary: candidate.clone(),
        contexts: vec![context],
        records: vec![record, second_record],
        revisions: vec![
            OwnedCurationRevision {
                revision: current.metadata().owned_revisions[0].id.clone(),
                active: vec![],
                tombstones: vec![],
            },
            OwnedCurationRevision {
                revision: candidate.metadata().owned_revisions[1].id.clone(),
                active: vec![
                    EntityId::new("curated-link").unwrap(),
                    EntityId::new("curated-link-second").unwrap(),
                ],
                tombstones: vec![],
            },
        ],
    })
    .unwrap()
}

#[test]
fn explicit_migration_preserves_genesis_and_materializes_empty_history() {
    let root = fresh_root();
    let initial = genesis();
    let (mut writer, created) = OwnedJournal::create(&root, &initial).unwrap();
    let schema_one_edit = next_revision(&initial);
    let edited = writer
        .commit(&created.head.revision.id, &schema_one_edit)
        .unwrap();
    let before_reader = vlajmaji_store::OwnedJournalReader::open(&root).unwrap();
    let before = before_reader.head().unwrap();
    let before_history = before_reader.history().unwrap();
    drop(before_reader);
    let migrated = writer.migrate_curation(&edited.head.revision.id).unwrap();
    assert_eq!(migrated.outcome, OwnedMigrationOutcome::Migrated);
    assert_eq!(migrated.from_schema, 1);
    assert_eq!(migrated.to_schema, 2);
    assert_eq!(
        vlajmaji_store::OwnedJournalReader::open(&root)
            .unwrap()
            .head()
            .unwrap(),
        before
    );
    let after_reader = vlajmaji_store::OwnedJournalReader::open(&root).unwrap();
    assert_eq!(after_reader.history().unwrap(), before_history);
    drop(after_reader);
    let reader = vlajmaji_store::OwnedJournalReader::open(&root).unwrap();
    let shown = reader.curation_show_head().unwrap();
    assert_eq!(shown.as_data().dictionary, schema_one_edit);
    assert!(shown.as_data().contexts.is_empty());
    assert!(shown.as_data().records.is_empty());
    assert_eq!(shown.as_data().revisions.len(), 2);
    assert!(
        shown
            .as_data()
            .revisions
            .iter()
            .all(|row| row.active.is_empty())
    );
    drop(reader);
    assert_eq!(
        writer
            .migrate_curation(&edited.head.revision.id)
            .unwrap()
            .outcome,
        OwnedMigrationOutcome::AlreadyCurrent
    );
}

#[test]
fn curated_write_requires_migration_before_mutation() {
    let root = fresh_root();
    let initial = genesis();
    let (mut writer, created) = OwnedJournal::create(&root, &initial).unwrap();
    let candidate = empty_curation(initial);
    let expected_head = vlajmaji_store::OwnedHeadReport {
        version: 1,
        source: created.source.clone(),
        head: created.head.clone(),
    };
    let error = writer
        .commit_curated(&created.head.revision.id, &candidate)
        .unwrap_err();
    assert_eq!(
        error.category(),
        OwnedJournalErrorCategory::MigrationRequired
    );
    assert_eq!(
        vlajmaji_store::OwnedJournalReader::open(&root)
            .unwrap()
            .head()
            .unwrap(),
        expected_head
    );
}

#[test]
fn migration_observes_head_conflict_and_pinned_reader_busy_without_partial_schema() {
    let root = fresh_root();
    let initial = genesis();
    let (mut writer, created) = OwnedJournal::create(&root, &initial).unwrap();
    let mut preopened_writer = OwnedJournal::open(&root).unwrap();
    // A schema-one metadata header must not be accepted under a schema-two
    // SQLite user_version; restore the real header before the migration cases.
    let schema_probe = rusqlite::Connection::open(root.join("owned.sqlite")).unwrap();
    schema_probe.pragma_update(None, "user_version", 2).unwrap();
    assert_eq!(
        OwnedJournal::open(&root).err().unwrap().category(),
        OwnedJournalErrorCategory::UnsupportedVersion
    );
    schema_probe.pragma_update(None, "user_version", 1).unwrap();
    drop(schema_probe);
    let reader = vlajmaji_store::OwnedJournalReader::open(&root).unwrap();
    assert_eq!(reader.head().unwrap().head, created.head);

    let busy = writer
        .migrate_curation(&created.head.revision.id)
        .unwrap_err();
    assert_eq!(busy.category(), OwnedJournalErrorCategory::Busy);
    let pragma: i32 = rusqlite::Connection::open(root.join("owned.sqlite"))
        .unwrap()
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .unwrap();
    assert_eq!(pragma, 1);
    drop(reader);

    let migrated = writer.migrate_curation(&created.head.revision.id).unwrap();
    assert_eq!(migrated.outcome, OwnedMigrationOutcome::Migrated);
    assert_eq!(
        preopened_writer
            .migrate_curation(&created.head.revision.id)
            .unwrap()
            .outcome,
        OwnedMigrationOutcome::AlreadyCurrent
    );
    let stale = writer
        .migrate_curation(&EntityId::new("other-head").unwrap())
        .unwrap_err();
    assert_eq!(stale.category(), OwnedJournalErrorCategory::Conflict);
}

#[test]
fn schema_two_ordinary_and_curated_commits_append_matching_history() {
    let root = fresh_root();
    let initial = genesis();
    let (mut writer, created) = OwnedJournal::create(&root, &initial).unwrap();
    writer.migrate_curation(&created.head.revision.id).unwrap();

    let ordinary = next_revision(&initial);
    let ordinary_report = writer.commit(&created.head.revision.id, &ordinary).unwrap();
    assert_eq!(
        ordinary_report.operation,
        vlajmaji_store::OwnedWriteOperation::Edit
    );
    let reader = vlajmaji_store::OwnedJournalReader::open(&root).unwrap();
    let carried = reader.curation_show_head().unwrap();
    assert_eq!(carried.as_data().dictionary, ordinary);
    assert_eq!(carried.as_data().revisions.len(), 2);
    assert!(
        carried
            .as_data()
            .revisions
            .iter()
            .all(|row| row.active.is_empty())
    );
    drop(reader);

    let curated_base = next_revision(&ordinary);
    let mut curated_data = carried.into_data();
    let revision = curated_base
        .metadata()
        .owned_revisions
        .last()
        .unwrap()
        .id
        .clone();
    curated_data.dictionary = curated_base.clone();
    let active = curated_data.revisions.last().unwrap().active.clone();
    curated_data.revisions.push(OwnedCurationRevision {
        revision,
        active,
        tombstones: Vec::new(),
    });
    let curated = OwnedCurationDocument::from_data(curated_data).unwrap();
    writer
        .commit_curated(
            &ordinary.metadata().owned_revisions.last().unwrap().id,
            &curated,
        )
        .unwrap();
    let reader = vlajmaji_store::OwnedJournalReader::open(&root).unwrap();
    let shown = reader.curation_show_head().unwrap();
    assert_eq!(shown, curated);
    assert_eq!(shown.as_data().revisions.len(), 3);
}

#[test]
fn schema_two_persists_nonempty_pin_record_and_active_selection_prefixes() {
    let root = fresh_root();
    let initial = genesis();
    let (mut writer, created) = OwnedJournal::create(&root, &initial).unwrap();
    writer.migrate_curation(&created.head.revision.id).unwrap();
    let mut same_head_writer = OwnedJournal::open(&root).unwrap();
    let candidate = linked_curation(&initial);
    let stale = writer
        .commit_curated(&EntityId::new("stale-curation-head").unwrap(), &candidate)
        .unwrap_err();
    assert_eq!(stale.category(), OwnedJournalErrorCategory::Conflict);
    assert_eq!(
        vlajmaji_store::OwnedJournalReader::open(&root)
            .unwrap()
            .head()
            .unwrap()
            .head
            .revision
            .id,
        created.head.revision.id
    );
    writer
        .commit_curated(&created.head.revision.id, &candidate)
        .unwrap();
    let same_head_conflict = same_head_writer
        .commit_curated(&created.head.revision.id, &candidate)
        .unwrap_err();
    assert_eq!(
        same_head_conflict.category(),
        OwnedJournalErrorCategory::Conflict
    );

    let reader = vlajmaji_store::OwnedJournalReader::open(&root).unwrap();
    let restored = reader.curation_show_head().unwrap();
    assert_eq!(restored, candidate);
    assert_eq!(restored.as_data().contexts.len(), 1);
    assert_eq!(restored.as_data().records.len(), 2);
    assert_eq!(
        restored.as_data().revisions[1].active[0].as_str(),
        "curated-link"
    );
    let unavailable = restored.materialize(vec![]).unwrap();
    assert_eq!(
        unavailable.resolutions[0].status,
        vlajmaji_lexicon::OwnedCurationStatus::Unavailable
    );
    assert_eq!(
        unavailable.resolutions[0].reason,
        Some(vlajmaji_lexicon::OwnedCurationReason::MissingContext)
    );
    drop(reader);

    let next_base = next_revision(&restored.as_data().dictionary);
    let next_base_head = next_base
        .metadata()
        .owned_revisions
        .last()
        .unwrap()
        .id
        .clone();
    let current_head = restored
        .as_data()
        .dictionary
        .metadata()
        .owned_revisions
        .last()
        .unwrap()
        .id
        .clone();
    for mutate_prefix in ["context", "record", "revision"] {
        let mut changed = restored.clone().into_data();
        changed.dictionary = next_base.clone();
        changed.revisions.push(OwnedCurationRevision {
            revision: next_base_head.clone(),
            active: restored.as_data().revisions[1].active.clone(),
            tombstones: Vec::new(),
        });
        if mutate_prefix == "record" {
            changed.records[0].rationale.text.push_str(" changed");
        } else if mutate_prefix == "context" {
            changed.contexts[0].model_sha256 = "c".repeat(64);
        } else {
            changed.revisions[1].active.reverse();
        }
        let changed = OwnedCurationDocument::from_data(changed).unwrap();
        let prefix_conflict = writer.commit_curated(&current_head, &changed).unwrap_err();
        assert_eq!(
            prefix_conflict.category(),
            OwnedJournalErrorCategory::InvalidCuration
        );
        assert_eq!(
            vlajmaji_store::OwnedJournalReader::open(&root)
                .unwrap()
                .curation_show_head()
                .unwrap(),
            restored
        );
    }

    let ordinary = next_revision(&candidate.as_data().dictionary);
    writer.commit(&current_head, &ordinary).unwrap();
    let carried = vlajmaji_store::OwnedJournalReader::open(&root)
        .unwrap()
        .curation_show_head()
        .unwrap();
    assert_eq!(carried.as_data().dictionary, ordinary);
    assert_eq!(carried.as_data().contexts, candidate.as_data().contexts);
    assert_eq!(carried.as_data().records, candidate.as_data().records);
    assert_eq!(carried.as_data().revisions.len(), 3);
    assert_eq!(
        carried.as_data().revisions[2].active,
        candidate.as_data().revisions[1].active
    );

    let link_id = EntityId::new("curated-link").unwrap();
    let r2 = ordinary
        .metadata()
        .owned_revisions
        .last()
        .unwrap()
        .id
        .clone();
    let r3_dictionary = next_revision(&ordinary);
    let mut tombstoned_data = carried.into_data();
    tombstoned_data.dictionary = r3_dictionary.clone();
    tombstoned_data.revisions.push(OwnedCurationRevision {
        revision: r3_dictionary
            .metadata()
            .owned_revisions
            .last()
            .unwrap()
            .id
            .clone(),
        active: vec![EntityId::new("curated-link-second").unwrap()],
        tombstones: vec![link_id.clone()],
    });
    let tombstoned = OwnedCurationDocument::from_data(tombstoned_data).unwrap();
    writer.commit_curated(&r2, &tombstoned).unwrap();
    let inactive = vlajmaji_store::OwnedJournalReader::open(&root)
        .unwrap()
        .curation_show_head()
        .unwrap();
    assert_eq!(
        inactive.materialize(Vec::new()).unwrap().resolutions[0].status,
        vlajmaji_lexicon::OwnedCurationStatus::Inactive
    );

    let tombstone_head = r3_dictionary
        .metadata()
        .owned_revisions
        .last()
        .unwrap()
        .id
        .clone();
    let r4_dictionary = next_revision(&r3_dictionary);
    let mut reactivated_data = inactive.into_data();
    reactivated_data.dictionary = r4_dictionary.clone();
    reactivated_data.revisions.push(OwnedCurationRevision {
        revision: r4_dictionary
            .metadata()
            .owned_revisions
            .last()
            .unwrap()
            .id
            .clone(),
        active: vec![link_id, EntityId::new("curated-link-second").unwrap()],
        tombstones: Vec::new(),
    });
    let reactivated = OwnedCurationDocument::from_data(reactivated_data).unwrap();
    writer
        .commit_curated(&tombstone_head, &reactivated)
        .unwrap();
    let restored = vlajmaji_store::OwnedJournalReader::open(&root)
        .unwrap()
        .curation_show_head()
        .unwrap();
    assert_eq!(restored, reactivated);
    assert_eq!(
        restored.materialize(Vec::new()).unwrap().resolutions[0].reason,
        Some(vlajmaji_lexicon::OwnedCurationReason::MissingContext)
    );
}
