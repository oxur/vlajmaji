//! End-to-end contract checks for authoritative owned dictionary journals.

use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

use serde_json::Value;

use vlajmaji_lexicon::{
    AbbreviationKind, Availability, CaptureRef, CollectionRecord, ComponentRecord, ContentNotice,
    ContributorId, ContributorRecord, Dictionary, DictionaryDocument, DictionaryMetadata, EntityId,
    OwnedHead, OwnedRevisionRecord, Presence, RedistributionStatus, RegistryDocument,
    ReleaseAccounting, ReleaseRecord, SourceId, SourceOrigin, SourceRecord, SourceRegistry,
    TextValue, TimePrecision,
};
#[path = "../../lexicon/examples/support/dictionary_fixture.rs"]
mod fixture;

use vlajmaji_store::{
    OwnedJournal, OwnedJournalErrorCategory, OwnedJournalReader, OwnedWriteOperation,
};

static NEXT_ROOT: AtomicUsize = AtomicUsize::new(0);

fn fresh_root() -> PathBuf {
    for _ in 0..64 {
        let parent = std::env::temp_dir().join(format!(
            "vlajmaji-owned-contract-{}-{}",
            std::process::id(),
            NEXT_ROOT.fetch_add(1, Ordering::Relaxed)
        ));
        match fs::create_dir(&parent) {
            Ok(()) => return parent.join("journal"),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => panic!("create isolated parent: {error}"),
        }
    }
    panic!("could not allocate an exclusive test parent after 64 attempts")
}

#[derive(Debug, PartialEq, Eq)]
struct JournalSqlSnapshot {
    records: Vec<(String, i64, Vec<u8>)>,
    commits: Vec<(String, Option<String>, i64, String)>,
    roster: Vec<(String, String)>,
}

fn sql_journal_snapshot(root: &std::path::Path) -> JournalSqlSnapshot {
    let connection = rusqlite::Connection::open(root.join("owned.sqlite")).unwrap();
    let mut records_statement = connection
        .prepare("SELECT section,ordinal,payload FROM owned_records ORDER BY section,ordinal")
        .unwrap();
    let records = records_statement
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let mut commits_statement = connection
        .prepare(
            "SELECT revision_id,parent_revision,sequence,state_sha256 FROM owned_commits ORDER BY sequence",
        )
        .unwrap();
    let commits = commits_statement
        .query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let mut roster_statement = connection
        .prepare(
            "SELECT revision_id,section FROM owned_commit_sections ORDER BY revision_id,section",
        )
        .unwrap();
    let roster = roster_statement
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    JournalSqlSnapshot {
        records,
        commits,
        roster,
    }
}

fn release(id: &str, component: &EntityId) -> ReleaseRecord {
    ReleaseRecord {
        id: EntityId::new(id).unwrap(),
        component: component.clone(),
        distributor: None,
        upstream_revision: Presence::Missing,
        distributor_revision: Presence::Missing,
        captured_at: Presence::Missing,
        time_precision: TimePrecision::Unknown,
        schema_version: 1,
        normalizer: "fixture-normalizer".into(),
        normalizer_version: "1".into(),
        captures: Vec::new(),
        assertions: Vec::new(),
        accounting: ReleaseAccounting {
            input_occurrences: Some(0),
            normalized_occurrences: Some(0),
            query_excluded_occurrences: Some(0),
            unit: "occurrences".into(),
        },
        diagnostics: Vec::new(),
        selected: true,
        availability: Availability::Available,
        content_notices: vec![ContentNotice {
            text: Presence::Value("fixture only; no licence grant".into()),
            locator: Presence::Missing,
            license: Presence::Missing,
        }],
        redistribution: RedistributionStatus::Held,
        redistribution_evidence: Vec::new(),
    }
}

fn genesis() -> Dictionary {
    let source = SourceId::new("owned-source").unwrap();
    let editor = ContributorId::new("owned-editor").unwrap();
    let collection_id = EntityId::new("owned-collection").unwrap();
    let component_id = EntityId::new("owned-component").unwrap();
    let revision_id = EntityId::new("owned-r0").unwrap();
    let registry = SourceRegistry::from_document(RegistryDocument {
        sources: vec![SourceRecord {
            id: source.clone(),
            display_name: "Personal dictionary".into(),
            abbreviation: "OWN".into(),
            abbreviation_kind: AbbreviationKind::ProjectAssigned,
            abbreviation_evidence: None,
            origin: SourceOrigin::OwnedDraft,
        }],
        contributors: vec![ContributorRecord {
            id: editor.clone(),
            display_name: "Fixture editor".into(),
        }],
        ..RegistryDocument::default()
    })
    .unwrap();
    let mut metadata = DictionaryMetadata::default();
    metadata.collections.push(CollectionRecord {
        id: collection_id.clone(),
        source: source.clone(),
        display_name: "Owned collection".into(),
        description: Presence::Missing,
    });
    metadata.components.push(ComponentRecord {
        id: component_id.clone(),
        collection: collection_id,
        native_name: "owned component".into(),
        languages: vec!["jbo".into()],
    });
    metadata
        .releases
        .push(release("owned-release-r0", &component_id));
    metadata.owned_revisions.push(OwnedRevisionRecord {
        id: revision_id.clone(),
        source: source.clone(),
        parent: None,
        editor,
        timestamp: Presence::Missing,
        time_precision: TimePrecision::Unknown,
        assertions: Vec::new(),
        tombstones: Vec::new(),
        rationale: Presence::Value(TextValue {
            text: "initial draft".into(),
            language: Presence::Missing,
            markup: Presence::Missing,
        }),
    });
    metadata.owned_heads.push(OwnedHead {
        source,
        revision: revision_id,
    });
    Dictionary::from_document(DictionaryDocument {
        registry,
        metadata,
        ..DictionaryDocument::default()
    })
    .unwrap()
}

fn next_revision(current: &Dictionary) -> Dictionary {
    let mut document = current.document();
    let old = document.metadata.owned_revisions.last().unwrap().clone();
    let next_number = document.metadata.owned_revisions.len();
    let next_id = EntityId::new(format!("owned-r{next_number}")).unwrap();
    let editor = old.editor.clone();
    let component = document.metadata.components[0].id.clone();
    document.metadata.releases.push(release(
        &format!("owned-release-r{next_number}"),
        &component,
    ));
    document.metadata.catalogue_revision += 1;
    document.metadata.owned_revisions.push(OwnedRevisionRecord {
        id: next_id.clone(),
        source: old.source.clone(),
        parent: Some(old.id),
        editor,
        timestamp: Presence::Missing,
        time_precision: TimePrecision::Unknown,
        assertions: old.assertions.clone(),
        tombstones: Vec::new(),
        rationale: Presence::Value(TextValue {
            text: "second draft".into(),
            language: Presence::Null,
            markup: Presence::Missing,
        }),
    });
    document.metadata.owned_heads[0].revision = next_id;
    Dictionary::from_document(document).unwrap()
}

fn rich_owned_genesis() -> Dictionary {
    let mut document = fixture::complete_fixture()
        .expect("complete fixture")
        .document();
    let owner = vlajmaji_lexicon::SourceId::new("source-o").unwrap();
    let mut registry = document.registry.document();
    registry.sources.retain(|source| source.id == owner);
    registry.sources[0].origin = SourceOrigin::OwnedDraft;
    registry.aliases.retain(|alias| alias.source == owner);
    registry.aliases.push(vlajmaji_lexicon::SourceAlias {
        source: owner.clone(),
        text: "owned-fixture-alias".into(),
        status: vlajmaji_lexicon::AliasStatus::Active,
        evidence: Some("synthetic contract fixture".into()),
    });
    registry.aliases.push(vlajmaji_lexicon::SourceAlias {
        source: owner.clone(),
        text: "owned-fixture-alias-second".into(),
        status: vlajmaji_lexicon::AliasStatus::Active,
        evidence: Some("second synthetic contract fixture alias".into()),
    });
    registry.lineage.retain(|edge| edge.source == owner);
    registry.lineage.push(vlajmaji_lexicon::SourceLineage {
        source: owner.clone(),
        upstream: vlajmaji_lexicon::LineageTarget::Unresolved("synthetic fixture target".into()),
        evidence: Some("synthetic contract fixture".into()),
    });
    registry.lineage.push(vlajmaji_lexicon::SourceLineage {
        source: owner.clone(),
        upstream: vlajmaji_lexicon::LineageTarget::Unresolved(
            "second synthetic fixture target".into(),
        ),
        evidence: Some("second synthetic contract fixture lineage".into()),
    });
    for edge in &mut registry.lineage {
        if let vlajmaji_lexicon::LineageTarget::Known(_) = edge.upstream {
            edge.upstream = vlajmaji_lexicon::LineageTarget::Unresolved(
                "synthetic retained lineage target".into(),
            );
        }
    }
    for snapshot in &mut registry.snapshots {
        snapshot.source = owner.clone();
    }
    document.registry = vlajmaji_lexicon::SourceRegistry::from_document(registry).unwrap();

    document.legacy_archive = None;
    document.metadata.catalogue_revision = 0;
    for collection in &mut document.metadata.collections {
        collection.source = owner.clone();
    }
    for entry in &mut document.metadata.entries {
        entry.source = owner.clone();
    }
    for relation in &mut document.metadata.relations {
        if let vlajmaji_lexicon::RelationTarget::Catalog(target) = &mut relation.target {
            target.source = owner.clone();
        }
    }
    for release in &mut document.metadata.releases {
        if release.content_notices.is_empty() {
            release
                .content_notices
                .push(vlajmaji_lexicon::ContentNotice {
                    text: Presence::Value("synthetic notice".into()),
                    locator: Presence::Missing,
                    license: Presence::Missing,
                });
        }
    }
    let mut ordered_extension = document.metadata.extensions[0].clone();
    ordered_extension.name = "synthetic-second-extension".into();
    ordered_extension.value = vlajmaji_lexicon::RawJson::new(r#"{"ordered":2}"#).unwrap();
    document.metadata.extensions.push(ordered_extension);
    let current = document
        .metadata
        .owned_revisions
        .last()
        .expect("fixture owned head")
        .clone();
    let mut genesis_revision = current;
    genesis_revision.parent = None;
    genesis_revision.tombstones.clear();
    document.metadata.owned_revisions = vec![genesis_revision.clone()];
    document.metadata.owned_heads = vec![vlajmaji_lexicon::OwnedHead {
        source: owner,
        revision: genesis_revision.id,
    }];
    Dictionary::from_document(document).unwrap()
}

#[test]
fn create_read_and_append_preserve_typed_history() {
    let root = fresh_root();
    let initial = genesis();
    let (mut writer, created) = OwnedJournal::create(&root, &initial).unwrap();
    assert_eq!(created.operation, OwnedWriteOperation::Init);
    assert_eq!(created.head.sequence, 0);

    let reader = OwnedJournalReader::open(&root).unwrap();
    assert_eq!(reader.show_head().unwrap().dictionary, initial);
    assert_eq!(reader.history().unwrap().commits.len(), 1);
    drop(reader);

    let revised = next_revision(&initial);
    let appended = writer.commit(&created.head.revision.id, &revised).unwrap();
    assert_eq!(appended.head.sequence, 1);
    assert_eq!(
        appended.previous_head,
        Some(created.head.revision.id.clone())
    );

    let reader = OwnedJournalReader::open(&root).unwrap();
    assert_eq!(reader.show_head().unwrap().dictionary, revised);
    assert_eq!(
        reader.show(&appended.head.revision.id).unwrap().dictionary,
        revised
    );
    assert_eq!(
        reader.show(&created.head.revision.id).unwrap().dictionary,
        initial
    );
    assert_eq!(reader.history().unwrap().commits.len(), 2);
}

#[test]
fn complete_owned_state_round_trips_all_ordered_sections_and_presence() {
    let root = fresh_root();
    let dictionary = rich_owned_genesis();
    let (writer, created) = OwnedJournal::create(&root, &dictionary).unwrap();
    drop(writer);

    let connection = rusqlite::Connection::open(root.join("owned.sqlite")).unwrap();
    let (section_count, populated_count): (i64, i64) = connection
        .query_row(
            "SELECT COUNT(*), SUM(CASE WHEN length>0 THEN 1 ELSE 0 END) FROM owned_commit_sections WHERE revision_id=?1",
            [created.head.revision.id.to_string()],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_eq!(section_count, 24);
    assert_eq!(populated_count, 24);
    drop(connection);

    let view = OwnedJournalReader::open(&root)
        .unwrap()
        .show_head()
        .unwrap();
    assert_eq!(view.dictionary, dictionary);
    assert_eq!(
        view.dictionary.captures().len(),
        dictionary.captures().len()
    );
    assert!(view.dictionary.metadata().entries.iter().any(|entry| {
        matches!(entry.spelling, Presence::Missing) || matches!(entry.spelling, Presence::Null)
    }));
    assert!(view.dictionary.captures().iter().any(|capture| {
        capture.occurrences.iter().any(|occurrence| {
            occurrence.fields.len() > 1
                && occurrence
                    .fields
                    .iter()
                    .any(|field| field.encoding == vlajmaji_lexicon::NativeEncoding::Json)
                && occurrence.raw_line.is_some()
        })
    }));
}

#[test]
fn complete_vector_history_records_removal_and_explicit_reactivation() {
    let root = fresh_root();
    let mut initial_document = rich_owned_genesis().document();
    let mut owned_ids = initial_document
        .metadata
        .assertions
        .iter()
        .filter(|assertion| {
            initial_document.metadata.entries.iter().any(|entry| {
                entry.id == assertion.entry && entry.source == SourceId::new("source-o").unwrap()
            })
        })
        .take(4)
        .map(|assertion| assertion.id.clone())
        .collect::<Vec<_>>();
    if owned_ids.len() < 4 {
        owned_ids = initial_document
            .metadata
            .assertions
            .iter()
            .take(4)
            .map(|assertion| assertion.id.clone())
            .collect();
    }
    assert_eq!(owned_ids.len(), 4);
    assert_eq!(
        owned_ids
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        4
    );
    let initial_revision = initial_document
        .metadata
        .owned_revisions
        .last_mut()
        .unwrap();
    initial_revision.assertions = vec![
        owned_ids[0].clone(),
        owned_ids[1].clone(),
        owned_ids[3].clone(),
    ];
    let initial = Dictionary::from_document(initial_document).unwrap();
    let (mut writer, created) = OwnedJournal::create(&root, &initial).unwrap();

    let mut middle_document = next_revision(&initial).document();
    let middle_revision = middle_document.metadata.owned_revisions.last_mut().unwrap();
    middle_revision.assertions = vec![owned_ids[0].clone(), owned_ids[1].clone()];
    middle_revision.tombstones = vec![owned_ids[3].clone()];
    let middle = Dictionary::from_document(middle_document).unwrap();
    let middle_report = writer.commit(&created.head.revision.id, &middle).unwrap();
    assert_eq!(middle_report.head.sequence, 1);

    let mut candidate_document = next_revision(&middle).document();
    let mut new_assertion = candidate_document.metadata.assertions[0].clone();
    new_assertion.id = EntityId::new("owned-assertion-c").unwrap();
    new_assertion.occurrence = vlajmaji_lexicon::OccurrenceRef {
        capture: CaptureRef {
            namespace: "catalogue:m:future".into(),
            sha256: "0".repeat(64),
        },
        locator: "/journal-only-c".into(),
    };
    new_assertion.eligibility = vlajmaji_lexicon::Eligibility::Excluded {
        reason: "unavailable catalogue assertion used only for journal policy".into(),
    };
    let new_assertion_id = new_assertion.id.clone();
    candidate_document.metadata.assertions.push(new_assertion);
    let mut unavailable_release = candidate_document
        .metadata
        .releases
        .iter()
        .find(|release| release.id == EntityId::new("release-m").unwrap())
        .unwrap()
        .clone();
    unavailable_release.id = EntityId::new("release-owned-c").unwrap();
    unavailable_release.assertions = vec![new_assertion_id.clone()];
    candidate_document
        .metadata
        .releases
        .push(unavailable_release);
    let candidate_revision = candidate_document
        .metadata
        .owned_revisions
        .last_mut()
        .unwrap();
    candidate_revision.assertions = vec![owned_ids[1].clone(), new_assertion_id.clone()];
    candidate_revision.tombstones = vec![owned_ids[0].clone()];
    let valid_candidate = Dictionary::from_document(candidate_document.clone()).unwrap();

    let mut omitted_tombstone_document = candidate_document.clone();
    omitted_tombstone_document
        .metadata
        .owned_revisions
        .last_mut()
        .unwrap()
        .tombstones
        .clear();
    let omitted_tombstone = Dictionary::from_document(omitted_tombstone_document).unwrap();
    assert!(matches!(
        writer.commit(&middle_report.head.revision.id, &omitted_tombstone),
        Err(vlajmaji_store::OwnedJournalError::InvalidRequest(_))
    ));

    let mut omitted_new_assertion_document = candidate_document.clone();
    let omitted_revision = omitted_new_assertion_document
        .metadata
        .owned_revisions
        .last_mut()
        .unwrap();
    omitted_revision.assertions = vec![owned_ids[1].clone()];
    omitted_revision.tombstones = vec![owned_ids[0].clone()];
    let omitted_new_assertion = Dictionary::from_document(omitted_new_assertion_document).unwrap();
    assert!(matches!(
        writer.commit(&middle_report.head.revision.id, &omitted_new_assertion),
        Err(vlajmaji_store::OwnedJournalError::InvalidRequest(message))
            if message.contains("every appended assertion")
    ));

    let mut extra_tombstone_document = candidate_document.clone();
    extra_tombstone_document
        .metadata
        .owned_revisions
        .last_mut()
        .unwrap()
        .tombstones
        .push(owned_ids[3].clone());
    let extra_tombstone = Dictionary::from_document(extra_tombstone_document).unwrap();
    assert!(matches!(
        writer.commit(&middle_report.head.revision.id, &extra_tombstone),
        Err(vlajmaji_store::OwnedJournalError::InvalidRequest(_))
    ));

    let mut multiple_document = candidate_document;
    let mut extra_revision = multiple_document
        .metadata
        .owned_revisions
        .last()
        .unwrap()
        .clone();
    extra_revision.id = EntityId::new("owned-r3").unwrap();
    extra_revision.parent = Some(EntityId::new("owned-r2").unwrap());
    extra_revision.tombstones.clear();
    multiple_document
        .metadata
        .owned_revisions
        .push(extra_revision.clone());
    multiple_document.metadata.owned_heads[0].revision = extra_revision.id;
    multiple_document.metadata.catalogue_revision += 1;
    let multiple = Dictionary::from_document(multiple_document).unwrap();
    assert!(matches!(
        writer.commit(&middle_report.head.revision.id, &multiple),
        Err(vlajmaji_store::OwnedJournalError::InvalidRequest(_))
    ));

    let active_id = owned_ids[0].clone();
    let removed_report = writer
        .commit(&middle_report.head.revision.id, &valid_candidate)
        .unwrap();
    assert_eq!(
        removed_report.head.revision.assertions,
        vec![owned_ids[1].clone(), new_assertion_id.clone()]
    );
    assert_eq!(
        removed_report.head.revision.tombstones,
        vec![active_id.clone()]
    );
    assert_eq!(removed_report.head.sequence, 2);

    let mut restored_document = next_revision(&valid_candidate).document();
    let restored_revision = restored_document
        .metadata
        .owned_revisions
        .last_mut()
        .unwrap();
    restored_revision.assertions = vec![owned_ids[0].clone(), owned_ids[1].clone()];
    restored_revision.tombstones = vec![new_assertion_id];
    let restored = Dictionary::from_document(restored_document).unwrap();
    let restored_report = writer
        .commit(&removed_report.head.revision.id, &restored)
        .unwrap();
    assert_eq!(restored_report.head.sequence, 3);

    let reader = OwnedJournalReader::open(&root).unwrap();
    assert_eq!(reader.show_head().unwrap().dictionary, restored);
    assert_eq!(
        reader.show(&created.head.revision.id).unwrap().dictionary,
        initial
    );
    assert_eq!(
        reader
            .show(&middle_report.head.revision.id)
            .unwrap()
            .dictionary,
        middle
    );
    assert_eq!(
        reader
            .show(&removed_report.head.revision.id)
            .unwrap()
            .dictionary,
        valid_candidate
    );
    assert_eq!(reader.history().unwrap().commits.len(), 4);
    assert_eq!(
        reader
            .show(&removed_report.head.revision.id)
            .unwrap()
            .commit
            .revision
            .tombstones,
        vec![active_id]
    );
}

fn change_first_row(section: &str, document: &mut DictionaryDocument) {
    let mut registry = document.registry.document();
    match section {
        "registry.sources" => registry.sources[0].display_name.push_str(" changed"),
        "registry.contributors" => registry.contributors[0].display_name.push_str(" changed"),
        "registry.distributors" => registry.distributors[0].display_name.push_str(" changed"),
        "registry.aliases" => registry.aliases[0].evidence = Some("changed".into()),
        "registry.lineage" => registry.lineage[0].evidence = Some("changed".into()),
        "registry.snapshots" => {
            assert!(registry.snapshots[0].distributor.is_some());
            registry.snapshots[0].distributor = None;
        }
        "registry.occurrences" => {
            registry.occurrences[0].native_entry_id = Some("changed".into());
        }
        "captures" => document.captures[0]
            .metadata
            .source_name
            .push_str(" changed"),
        "metadata.code_notices" => document.metadata.code_notices[0].text.push_str(" changed"),
        "metadata.collections" => document.metadata.collections[0]
            .display_name
            .push_str(" changed"),
        "metadata.components" => document.metadata.components[0]
            .native_name
            .push_str(" changed"),
        "metadata.releases" => document.metadata.releases[0]
            .normalizer
            .push_str(" changed"),
        "metadata.entries" => {
            document.metadata.entries[0].spelling = Presence::Value("farvi changed".into());
        }
        "metadata.assertions" => document.metadata.assertions[0].language = Presence::Null,
        "metadata.contexts" => document.metadata.contexts[0].value = Presence::Null,
        "metadata.frames" => document.metadata.frames[0].slots[0]
            .native_label
            .push_str(" changed"),
        "metadata.forms" => document.metadata.forms[0].spelling.push_str(" changed"),
        "metadata.etymologies" => document.metadata.etymologies[0].native_kind = Presence::Null,
        "metadata.relations" => {
            document.metadata.relations[0].direction = vlajmaji_lexicon::Direction::Reverse;
        }
        "metadata.owned_revisions" => {
            if let Presence::Value(rationale) = &mut document.metadata.owned_revisions[0].rationale
            {
                rationale.text.push_str(" changed");
            }
        }
        "metadata.editorial_senses" => document.metadata.editorial_senses[0]
            .rationale
            .text
            .push_str(" changed"),
        "metadata.computations" => document.metadata.computations[0]
            .policy
            .push_str(" changed"),
        "metadata.annotations" => document.metadata.annotations[0].kind.push_str(" changed"),
        "metadata.extensions" => document.metadata.extensions[0].name.push_str(" changed"),
        _ => panic!("unknown section {section}"),
    }
    document.registry = vlajmaji_lexicon::SourceRegistry::from_document(registry).unwrap();
}

fn alter_registry_rows(section: &str, document: &mut DictionaryDocument, reorder: bool) -> usize {
    let mut registry = document.registry.document();
    let ordinal = match section {
        "registry.aliases" => {
            let ordinal = if reorder {
                registry.aliases.len() - 2
            } else {
                registry.aliases.len() - 1
            };
            if reorder {
                assert_ne!(registry.aliases[ordinal], registry.aliases[ordinal + 1]);
                registry.aliases.swap(ordinal, ordinal + 1);
            } else {
                registry.aliases.remove(ordinal);
            }
            ordinal
        }
        "registry.lineage" => {
            let ordinal = if reorder {
                registry.lineage.len() - 2
            } else {
                registry.lineage.len() - 1
            };
            if reorder {
                assert_ne!(registry.lineage[ordinal], registry.lineage[ordinal + 1]);
                registry.lineage.swap(ordinal, ordinal + 1);
            } else {
                registry.lineage.remove(ordinal);
            }
            ordinal
        }
        _ => panic!("unsupported registry row alteration {section}"),
    };
    document.registry = vlajmaji_lexicon::SourceRegistry::from_document(registry).unwrap();
    ordinal
}

fn mutate_section_bytes(section: &str, bytes: &[u8], remove: bool) -> Option<Vec<u8>> {
    let mut value: Value = serde_json::from_slice(bytes).unwrap();
    let mut current = &mut value;
    for part in section.split('.') {
        current = current.get_mut(part)?;
    }
    let rows = current.as_array_mut()?;
    let prefix_end = if matches!(section, "metadata.releases" | "metadata.owned_revisions") {
        rows.len().checked_sub(2)?
    } else {
        rows.len().checked_sub(1)?
    };
    if remove {
        rows.remove(prefix_end);
    } else {
        if prefix_end == 0 {
            return None;
        }
        let previous = prefix_end - 1;
        if rows[previous] == rows[prefix_end] {
            return None;
        }
        rows.swap(previous, prefix_end);
    }
    Some(serde_json::to_vec(&value).unwrap())
}

fn assert_prefix_mutation_result(
    section: &str,
    candidate_bytes: &[u8],
    initial: &Dictionary,
) -> bool {
    let root = fresh_root();
    let (mut writer, created) = OwnedJournal::create(&root, initial).unwrap();
    match Dictionary::from_json_bytes(candidate_bytes) {
        Ok(candidate) => {
            let Err(error) = writer.commit(&created.head.revision.id, &candidate) else {
                panic!("valid {section} prefix mutation was accepted")
            };
            assert!(
                matches!(
                    error,
                    vlajmaji_store::OwnedJournalError::ImmutableRecord { section: ref actual, .. }
                        if actual == section
                ),
                "valid {section} prefix mutation returned {error:?}"
            );
            true
        }
        Err(error) => {
            eprintln!("{section} mutation excluded by model validation: {error:?}");
            false
        }
    }
}

#[test]
fn applicable_typed_section_removal_and_reorder_preserve_the_committed_prefix() {
    const SECTIONS: [&str; 24] = [
        "registry.sources",
        "registry.contributors",
        "registry.distributors",
        "registry.aliases",
        "registry.lineage",
        "registry.snapshots",
        "registry.occurrences",
        "captures",
        "metadata.code_notices",
        "metadata.collections",
        "metadata.components",
        "metadata.releases",
        "metadata.entries",
        "metadata.assertions",
        "metadata.contexts",
        "metadata.frames",
        "metadata.forms",
        "metadata.etymologies",
        "metadata.relations",
        "metadata.owned_revisions",
        "metadata.editorial_senses",
        "metadata.computations",
        "metadata.annotations",
        "metadata.extensions",
    ];
    let initial = rich_owned_genesis();
    let candidate = next_revision(&initial);
    let candidate_bytes = candidate.to_json_bytes().unwrap();
    let mut valid_removals = Vec::new();
    let mut invalid_removals = Vec::new();
    let mut valid_reorders = Vec::new();
    let mut invalid_reorders = Vec::new();
    let mut inapplicable_reorders = Vec::new();

    for section in SECTIONS {
        if let Some(bytes) = mutate_section_bytes(section, &candidate_bytes, true) {
            if assert_prefix_mutation_result(section, &bytes, &initial) {
                valid_removals.push(section);
            } else {
                invalid_removals.push(section);
            }
        }
        if let Some(bytes) = mutate_section_bytes(section, &candidate_bytes, false) {
            if assert_prefix_mutation_result(section, &bytes, &initial) {
                valid_reorders.push(section);
            } else {
                invalid_reorders.push(section);
            }
        } else {
            inapplicable_reorders.push(section);
        }
    }
    assert!(!valid_removals.is_empty());
    assert!(!valid_reorders.is_empty());
    eprintln!(
        "typed-section prefix census: valid removals={valid_removals:?}; model-invalid removals={invalid_removals:?}; valid reorders={valid_reorders:?}; model-invalid reorders={invalid_reorders:?}; singleton/equal-tail reorder not applicable={inapplicable_reorders:?}"
    );
}

#[test]
fn every_typed_section_rejects_a_changed_prior_row_and_preserves_its_order() {
    const SECTIONS: [&str; 24] = [
        "registry.sources",
        "registry.contributors",
        "registry.distributors",
        "registry.aliases",
        "registry.lineage",
        "registry.snapshots",
        "registry.occurrences",
        "captures",
        "metadata.code_notices",
        "metadata.collections",
        "metadata.components",
        "metadata.releases",
        "metadata.entries",
        "metadata.assertions",
        "metadata.contexts",
        "metadata.frames",
        "metadata.forms",
        "metadata.etymologies",
        "metadata.relations",
        "metadata.owned_revisions",
        "metadata.editorial_senses",
        "metadata.computations",
        "metadata.annotations",
        "metadata.extensions",
    ];
    let initial = rich_owned_genesis();
    for section in SECTIONS {
        let root = fresh_root();
        let (mut writer, created) = OwnedJournal::create(&root, &initial).unwrap();
        let mut changed_document = initial.document();
        change_first_row(section, &mut changed_document);
        let changed_prefix = Dictionary::from_document(changed_document).unwrap();
        assert_ne!(
            initial.to_json_bytes().unwrap(),
            changed_prefix.to_json_bytes().unwrap(),
            "{section} fixture mutation must change typed input bytes"
        );
        let candidate = next_revision(&changed_prefix);
        let error = match writer.commit(&created.head.revision.id, &candidate) {
            Ok(report) => panic!("accepted changed prior {section} row: {report:?}"),
            Err(error) => error,
        };
        match error {
            vlajmaji_store::OwnedJournalError::ImmutableRecord {
                section: actual,
                ordinal,
            } => {
                assert_eq!(actual, section);
                assert_eq!(ordinal, 0);
            }
            other => panic!("unexpected mutation result for {section}: {other}"),
        }
        assert_eq!(
            OwnedJournalReader::open(&root)
                .unwrap()
                .show_head()
                .unwrap()
                .dictionary,
            initial,
            "rejected {section} mutation must leave genesis unchanged"
        );
    }
}

#[test]
fn removal_reordering_and_duplicate_multiplicity_are_rejected() {
    let initial = rich_owned_genesis();
    let section = "metadata.extensions";
    let root = fresh_root();
    let (mut writer, created) = OwnedJournal::create(&root, &initial).unwrap();
    let mut removed_document = next_revision(&initial).document();
    let removed_ordinal = removed_document.metadata.extensions.len() - 1;
    removed_document.metadata.extensions.pop();
    let removed = Dictionary::from_document(removed_document).unwrap();
    let error = writer
        .commit(&created.head.revision.id, &removed)
        .unwrap_err();
    assert!(matches!(
        error,
        vlajmaji_store::OwnedJournalError::ImmutableRecord { section: ref actual, ordinal }
            if actual == section && ordinal == removed_ordinal as u64
    ));

    for section in ["registry.aliases", "registry.lineage"] {
        let initial = rich_owned_genesis();
        let root = fresh_root();
        let (mut writer, created) = OwnedJournal::create(&root, &initial).unwrap();
        let mut removed_document = next_revision(&initial).document();
        let ordinal = alter_registry_rows(section, &mut removed_document, false);
        let removed = Dictionary::from_document(removed_document).unwrap();
        let error = writer
            .commit(&created.head.revision.id, &removed)
            .unwrap_err();
        assert!(matches!(
            error,
            vlajmaji_store::OwnedJournalError::ImmutableRecord {
                section: ref actual,
                ordinal: actual_ordinal
            } if actual == section && actual_ordinal == ordinal as u64
        ));

        let mut reordered_document = next_revision(&initial).document();
        let ordinal = alter_registry_rows(section, &mut reordered_document, true);
        let reordered = Dictionary::from_document(reordered_document).unwrap();
        let error = writer
            .commit(&created.head.revision.id, &reordered)
            .unwrap_err();
        assert!(matches!(
            error,
            vlajmaji_store::OwnedJournalError::ImmutableRecord {
                section: ref actual,
                ordinal: actual_ordinal
            } if actual == section && actual_ordinal == ordinal as u64
        ));
    }

    let mut reordered_document = next_revision(&initial).document();
    let last = reordered_document.metadata.extensions.len() - 1;
    assert_ne!(
        reordered_document.metadata.extensions[last - 1],
        reordered_document.metadata.extensions[last],
        "order fixture rows must be distinct"
    );
    reordered_document.metadata.extensions.swap(last - 1, last);
    let reordered = Dictionary::from_document(reordered_document).unwrap();
    let error = writer
        .commit(&created.head.revision.id, &reordered)
        .unwrap_err();
    assert!(matches!(
        error,
        vlajmaji_store::OwnedJournalError::ImmutableRecord { section: ref actual, ordinal }
            if actual == section && ordinal == (last - 1) as u64
    ));

    let mut duplicated_document = initial.document();
    let duplicate = duplicated_document.metadata.extensions[0].clone();
    duplicated_document.metadata.extensions.push(duplicate);
    let duplicated = Dictionary::from_document(duplicated_document).unwrap();
    let duplicate_root = fresh_root();
    let (mut duplicate_writer, duplicate_created) =
        OwnedJournal::create(&duplicate_root, &duplicated).unwrap();
    let mut candidate_document = next_revision(&duplicated).document();
    let duplicate_ordinal = candidate_document.metadata.extensions.len() - 1;
    candidate_document.metadata.extensions.pop();
    let candidate = Dictionary::from_document(candidate_document).unwrap();
    let error = duplicate_writer
        .commit(&duplicate_created.head.revision.id, &candidate)
        .unwrap_err();
    assert!(matches!(
        error,
        vlajmaji_store::OwnedJournalError::ImmutableRecord { section: ref actual, ordinal }
            if actual == section && ordinal == duplicate_ordinal as u64
    ));
}

#[test]
fn changing_a_prior_release_selection_is_rejected_as_an_immutable_prefix() {
    let root = fresh_root();
    let initial = genesis();
    let (mut writer, created) = OwnedJournal::create(&root, &initial).unwrap();
    let mut document = next_revision(&initial).document();
    document.metadata.releases[0].selected = false;
    let candidate = Dictionary::from_document(document).unwrap();
    let error = writer
        .commit(&created.head.revision.id, &candidate)
        .unwrap_err();
    match error {
        vlajmaji_store::OwnedJournalError::ImmutableRecord { section, ordinal } => {
            assert_eq!(section, "metadata.releases");
            assert_eq!(ordinal, 0);
        }
        other => panic!("unexpected prior release mutation result: {other}"),
    }
    assert_eq!(
        OwnedJournalReader::open(&root)
            .unwrap()
            .show_head()
            .unwrap()
            .dictionary,
        initial
    );
}

#[test]
fn two_open_writers_compare_the_persisted_head() {
    let root = fresh_root();
    let initial = genesis();
    let (mut first, created) = OwnedJournal::create(&root, &initial).unwrap();
    let mut second = OwnedJournal::open(&root).unwrap();
    let revised = next_revision(&initial);
    first.commit(&created.head.revision.id, &revised).unwrap();
    let revised_again = next_revision(&revised);

    let before = OwnedJournalReader::open(&root).unwrap();
    let before_dictionary = before.show_head().unwrap().dictionary;
    let before_history = before.history().unwrap();
    drop(before);
    let before_sql = sql_journal_snapshot(&root);

    for candidate in [&revised_again, &revised] {
        let error = second
            .commit(&created.head.revision.id, candidate)
            .unwrap_err();
        assert_eq!(error.category(), OwnedJournalErrorCategory::Conflict);
        match error {
            vlajmaji_store::OwnedJournalError::Conflict { expected, actual } => {
                assert_eq!(expected, created.head.revision.id);
                assert_eq!(actual, EntityId::new("owned-r1").unwrap());
            }
            other => panic!("unexpected stale-writer result: {other}"),
        }
        let after = OwnedJournalReader::open(&root).unwrap();
        assert_eq!(after.show_head().unwrap().dictionary, before_dictionary);
        assert_eq!(after.history().unwrap(), before_history);
        drop(after);
        assert_eq!(sql_journal_snapshot(&root), before_sql);
    }
}

#[test]
fn pinned_reader_busy_rolls_back_then_explicit_retry_succeeds() {
    let root = fresh_root();
    let initial = genesis();
    let (mut writer, created) = OwnedJournal::create(&root, &initial).unwrap();
    let before_sql = sql_journal_snapshot(&root);
    let reader = OwnedJournalReader::open(&root).unwrap();
    assert_eq!(reader.head().unwrap().head.sequence, 0);
    let revised = next_revision(&initial);

    let error = writer
        .commit(&created.head.revision.id, &revised)
        .unwrap_err();
    assert_eq!(error.category(), OwnedJournalErrorCategory::Busy);
    assert_eq!(reader.head().unwrap().head.sequence, 0);
    assert_eq!(sql_journal_snapshot(&root), before_sql);
    assert_eq!(reader.history().unwrap().commits.len(), 1);
    drop(reader);

    assert_eq!(
        writer
            .commit(&created.head.revision.id, &revised)
            .unwrap()
            .head
            .sequence,
        1
    );
    assert_eq!(
        OwnedJournalReader::open(&root)
            .unwrap()
            .head()
            .unwrap()
            .head
            .sequence,
        1
    );
}

#[test]
fn aborting_head_update_rolls_back_suffix_and_allows_explicit_retry() {
    let root = fresh_root();
    let initial = genesis();
    let (mut writer, created) = OwnedJournal::create(&root, &initial).unwrap();
    let revised = next_revision(&initial);
    let database = root.join("owned.sqlite");
    let connection = rusqlite::Connection::open(&database).unwrap();
    let before = sql_journal_snapshot(&root);
    let before_reader = OwnedJournalReader::open(&root).unwrap();
    let before_dictionary = before_reader.show_head().unwrap().dictionary;
    let before_history = before_reader.history().unwrap();
    drop(before_reader);
    connection
        .execute_batch(
            "CREATE TRIGGER reject_owned_head BEFORE UPDATE OF head_revision ON owned_meta
             BEGIN SELECT RAISE(ABORT, 'test-only head failure'); END;",
        )
        .unwrap();

    let error = writer
        .commit(&created.head.revision.id, &revised)
        .unwrap_err();
    assert_eq!(error.category(), OwnedJournalErrorCategory::Backend);
    assert_eq!(
        sql_journal_snapshot(&root),
        before,
        "failed edit must leave all records, commits, and control roster unchanged"
    );
    drop(connection);
    let reader = OwnedJournalReader::open(&root).unwrap();
    assert_eq!(reader.show_head().unwrap().dictionary, before_dictionary);
    assert_eq!(reader.head().unwrap().head.sequence, 0);
    assert_eq!(reader.history().unwrap(), before_history);
    drop(reader);

    let connection = rusqlite::Connection::open(&database).unwrap();
    connection
        .execute_batch("DROP TRIGGER reject_owned_head")
        .unwrap();
    drop(connection);
    assert_eq!(
        writer
            .commit(&created.head.revision.id, &revised)
            .unwrap()
            .head
            .sequence,
        1
    );
}

#[test]
fn invalid_genesis_and_existing_foreign_root_are_not_replaced() {
    let invalid_root = fresh_root();
    let mut invalid_document = genesis().document();
    invalid_document.metadata.releases[0]
        .content_notices
        .clear();
    let invalid = Dictionary::from_document(invalid_document).unwrap();
    let error = OwnedJournal::create(&invalid_root, &invalid).err().unwrap();
    assert_eq!(error.category(), OwnedJournalErrorCategory::InvalidRequest);
    assert!(!invalid_root.exists());

    let no_releases_root = fresh_root();
    let mut no_releases_document = genesis().document();
    no_releases_document.metadata.releases.clear();
    no_releases_document.metadata.components.clear();
    no_releases_document.metadata.collections.clear();
    let no_releases = Dictionary::from_document(no_releases_document).unwrap();
    let error = OwnedJournal::create(&no_releases_root, &no_releases)
        .err()
        .unwrap();
    assert_eq!(error.category(), OwnedJournalErrorCategory::InvalidRequest);
    assert!(!no_releases_root.exists());

    let root = fresh_root();
    fs::create_dir(&root).unwrap();
    let sentinel = root.join("keep.txt");
    fs::write(&sentinel, b"do not replace").unwrap();
    let error = OwnedJournal::create(&root, &genesis()).err().unwrap();
    assert_eq!(error.category(), OwnedJournalErrorCategory::Io);
    assert_eq!(fs::read(sentinel).unwrap(), b"do not replace");
}

#[test]
fn catalogue_revision_above_sqlite_integer_range_is_rejected_without_mutation() {
    let root = fresh_root();
    let initial = genesis();
    let (mut writer, created) = OwnedJournal::create(&root, &initial).unwrap();
    let mut document = next_revision(&initial).document();
    document.metadata.catalogue_revision = i64::MAX as u64 + 1;
    let out_of_range = Dictionary::from_document(document).unwrap();
    assert!(matches!(
        writer.commit(&created.head.revision.id, &out_of_range),
        Err(vlajmaji_store::OwnedJournalError::InvalidRequest(_))
    ));
    let reader = OwnedJournalReader::open(&root).unwrap();
    assert_eq!(reader.show_head().unwrap().dictionary, initial);
    assert_eq!(reader.history().unwrap().commits.len(), 1);
}

#[test]
fn foreign_database_and_missing_open_are_rejected_without_creation() {
    let missing = fresh_root();
    assert_eq!(
        OwnedJournal::open(&missing).err().unwrap().category(),
        OwnedJournalErrorCategory::Io
    );
    assert!(!missing.exists());

    let foreign = fresh_root();
    fs::create_dir(&foreign).unwrap();
    let connection = rusqlite::Connection::open(foreign.join("owned.sqlite")).unwrap();
    connection
        .execute_batch("CREATE TABLE unrelated(value TEXT)")
        .unwrap();
    drop(connection);
    assert_eq!(
        OwnedJournalReader::open(&foreign).err().unwrap().category(),
        OwnedJournalErrorCategory::WrongFamily
    );

    let versioned = fresh_root();
    let (_, created) = OwnedJournal::create(&versioned, &genesis()).unwrap();
    let connection = rusqlite::Connection::open(versioned.join("owned.sqlite")).unwrap();
    connection.pragma_update(None, "user_version", 2).unwrap();
    drop(connection);
    assert_eq!(
        OwnedJournalReader::open(&versioned)
            .err()
            .unwrap()
            .category(),
        OwnedJournalErrorCategory::UnsupportedVersion
    );
    let connection = rusqlite::Connection::open(versioned.join("owned.sqlite")).unwrap();
    connection.pragma_update(None, "user_version", 1).unwrap();
    drop(connection);
    assert_eq!(
        OwnedJournalReader::open(&versioned)
            .unwrap()
            .show(&created.head.revision.id)
            .unwrap()
            .dictionary,
        genesis()
    );
}

#[test]
fn show_detects_corrupt_payload_unknown_sections_and_excess_current_rows() {
    let root = fresh_root();
    let initial = genesis();
    let (mut writer, created) = OwnedJournal::create(&root, &initial).unwrap();
    let revised = next_revision(&initial);
    let appended = writer.commit(&created.head.revision.id, &revised).unwrap();
    let database = root.join("owned.sqlite");

    let missing_id = EntityId::new("owned-not-present").unwrap();
    assert!(matches!(
        OwnedJournalReader::open(&root).unwrap().show(&missing_id),
        Err(vlajmaji_store::OwnedJournalError::NotFound(actual)) if actual == missing_id
    ));

    let connection = rusqlite::Connection::open(&database).unwrap();
    connection
        .execute(
            "UPDATE owned_commits SET state_sha256=?1 WHERE revision_id=?2",
            rusqlite::params!["0".repeat(64), appended.head.revision.id.to_string()],
        )
        .unwrap();
    drop(connection);
    assert_eq!(
        OwnedJournalReader::open(&root)
            .unwrap()
            .show(&appended.head.revision.id)
            .unwrap_err()
            .category(),
        OwnedJournalErrorCategory::Integrity
    );
    let connection = rusqlite::Connection::open(&database).unwrap();
    connection
        .execute(
            "UPDATE owned_commits SET state_sha256=?1 WHERE revision_id=?2",
            rusqlite::params![
                appended.head.state_sha256,
                appended.head.revision.id.to_string()
            ],
        )
        .unwrap();
    drop(connection);
    assert_eq!(
        OwnedJournalReader::open(&root)
            .unwrap()
            .show(&appended.head.revision.id)
            .unwrap()
            .dictionary,
        revised
    );

    let connection = rusqlite::Connection::open(&database).unwrap();
    connection
        .execute(
            "DELETE FROM owned_commit_sections WHERE revision_id=?1 AND section='metadata.collections'",
            [created.head.revision.id.to_string()],
        )
        .unwrap();
    drop(connection);
    assert_eq!(
        OwnedJournalReader::open(&root)
            .unwrap()
            .show(&created.head.revision.id)
            .unwrap_err()
            .category(),
        OwnedJournalErrorCategory::Integrity
    );
    let connection = rusqlite::Connection::open(&database).unwrap();
    connection
        .execute(
            "INSERT INTO owned_commit_sections(revision_id,section,length) VALUES(?1,'metadata.collections',?2)",
            rusqlite::params![
                created.head.revision.id.to_string(),
                initial.metadata().collections.len() as i64
            ],
        )
        .unwrap();
    drop(connection);
    assert_eq!(
        OwnedJournalReader::open(&root)
            .unwrap()
            .show(&created.head.revision.id)
            .unwrap()
            .dictionary,
        initial
    );

    let connection = rusqlite::Connection::open(&database).unwrap();
    connection
        .execute(
            "UPDATE owned_records SET payload=?1 WHERE section='metadata.collections' AND ordinal=0",
            [b"{}".as_slice()],
        )
        .unwrap();
    drop(connection);
    let reader = OwnedJournalReader::open(&root).unwrap();
    assert_eq!(reader.history().unwrap().commits.len(), 2);
    assert_eq!(
        reader
            .show(&appended.head.revision.id)
            .unwrap_err()
            .category(),
        OwnedJournalErrorCategory::Integrity
    );
    drop(reader);

    let connection = rusqlite::Connection::open(&database).unwrap();
    let original_collection = serde_json::to_vec(&initial.metadata().collections[0]).unwrap();
    connection
        .execute(
            "UPDATE owned_records SET payload=?1 WHERE section='metadata.collections' AND ordinal=0",
            [original_collection],
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO owned_records(section,ordinal,payload) VALUES('metadata.collections',2,?1)",
            [serde_json::to_vec(&revised.metadata().collections[0]).unwrap()],
        )
        .unwrap();
    drop(connection);
    let reader = OwnedJournalReader::open(&root).unwrap();
    assert_eq!(
        reader.show_head().unwrap_err().category(),
        OwnedJournalErrorCategory::Integrity
    );
    drop(reader);

    let connection = rusqlite::Connection::open(&database).unwrap();
    connection
        .execute(
            "DELETE FROM owned_records WHERE section='metadata.collections' AND ordinal=2",
            [],
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO owned_records(section,ordinal,payload) VALUES('unrecognized.section',0,?1)",
            [b"{}".as_slice()],
        )
        .unwrap();
    drop(connection);
    let reader = OwnedJournalReader::open(&root).unwrap();
    assert_eq!(
        reader.show_head().unwrap_err().category(),
        OwnedJournalErrorCategory::Integrity
    );
}

#[test]
fn head_rejects_a_consistent_but_non_genesis_parent_at_sequence_zero() {
    let root = fresh_root();
    let initial = genesis();
    let (_, created) = OwnedJournal::create(&root, &initial).unwrap();
    let database = root.join("owned.sqlite");
    let mut wrong_revision = initial.metadata().owned_revisions[0].clone();
    wrong_revision.parent = Some(EntityId::new("owned-not-a-genesis-parent").unwrap());
    let connection = rusqlite::Connection::open(&database).unwrap();
    connection
        .execute(
            "UPDATE owned_records SET payload=?1 WHERE section='metadata.owned_revisions' AND ordinal=0",
            [serde_json::to_vec(&wrong_revision).unwrap()],
        )
        .unwrap();
    connection
        .execute(
            "UPDATE owned_commits SET parent_revision=?1 WHERE revision_id=?2",
            rusqlite::params![
                wrong_revision.parent.as_ref().unwrap().to_string(),
                created.head.revision.id.to_string()
            ],
        )
        .unwrap();
    drop(connection);

    let reader = OwnedJournalReader::open(&root).unwrap();
    assert_eq!(
        reader.head().unwrap_err().category(),
        OwnedJournalErrorCategory::Integrity
    );
    drop(reader);

    let connection = rusqlite::Connection::open(&database).unwrap();
    connection
        .execute(
            "UPDATE owned_records SET payload=?1 WHERE section='metadata.owned_revisions' AND ordinal=0",
            [serde_json::to_vec(&initial.metadata().owned_revisions[0]).unwrap()],
        )
        .unwrap();
    connection
        .execute(
            "UPDATE owned_commits SET parent_revision=NULL WHERE revision_id=?1",
            [created.head.revision.id.to_string()],
        )
        .unwrap();
    drop(connection);
    assert_eq!(
        OwnedJournalReader::open(&root)
            .unwrap()
            .head()
            .unwrap()
            .head
            .sequence,
        0
    );
}
