//! Public store contract for explicitly built mixed workbench bindings.

#[path = "../../lexicon/examples/support/dictionary_fixture.rs"]
mod fixture;

use std::{
    borrow::Cow,
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

use vlajmaji_lexicon::{
    AbbreviationKind, AssertionRecord, Availability, CaptureMetadata, CatalogTarget,
    CollectionRecord, ComponentRecord, ContentNotice, ContributorId, ContributorRecord, Dictionary,
    DictionaryDocument, DictionaryMetadata, Direction, Eligibility, EntityId, EntryRecord,
    LocalRef, NativeEncoding, NativeField, OccurrenceAttribution, OccurrenceRef, OwnedContextPin,
    OwnedCurationData, OwnedCurationDocument, OwnedCurationKind, OwnedCurationRecord,
    OwnedCurationRevision, OwnedHead, OwnedPinnedTarget, OwnedRevisionRecord, OwnedSenseMember,
    PackError, PackId, PackPayloadKind, Presence, RawCapture, RawOccurrence, RedistributionStatus,
    RegistryDocument, ReleaseAccounting, ReleaseRecord, SnapshotAttribution, SourceId,
    SourceOrigin, SourceRecord, SourceRegistry, TextValue, TimePrecision, compose_dictionaries,
    seal_release,
};
use vlajmaji_store::{
    CollectionCatalogue, CollectionInput, CollectionRequest, CollectionSelection,
    DictionaryReadTopology, DictionaryStore, OwnedJournal, OwnedJournalReader, WorkbenchBinding,
    WorkbenchBuildRequest, WorkbenchContextBinding, WorkbenchError, WorkbenchOwnedRevision,
    WorkbenchSelection, WorkbenchView,
};

static NEXT_ROOT: AtomicUsize = AtomicUsize::new(0);

fn fresh_root(label: &str) -> PathBuf {
    for _ in 0..64 {
        let parent = std::env::temp_dir().join(format!(
            "vlajmaji-workbench-{label}-{}-{}",
            std::process::id(),
            NEXT_ROOT.fetch_add(1, Ordering::Relaxed)
        ));
        match fs::create_dir(&parent) {
            Ok(()) => return parent,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => panic!("create isolated parent: {error}"),
        }
    }
    panic!("could not allocate an exclusive parent after 64 attempts")
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
    metadata.releases.push(ReleaseRecord {
        id: EntityId::new("owned-release-r0").unwrap(),
        component: component_id,
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
    });
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

fn owned_head_selection() -> WorkbenchSelection {
    WorkbenchSelection {
        version: 1,
        packs: Vec::new(),
        disabled: Vec::new(),
        owned: Some(WorkbenchOwnedRevision::Head),
        contexts: Vec::new(),
    }
}

fn revision_one(seed: &Dictionary) -> Dictionary {
    let mut document = seed.document();
    let next = EntityId::new("owned-r1").unwrap();
    let source = SourceId::new("owned-source").unwrap();
    let editor = ContributorId::new("owned-editor").unwrap();
    document.metadata.catalogue_revision = 1;
    document.metadata.owned_revisions.push(OwnedRevisionRecord {
        id: next.clone(),
        source: source.clone(),
        parent: Some(EntityId::new("owned-r0").unwrap()),
        editor,
        timestamp: Presence::Missing,
        time_precision: TimePrecision::Unknown,
        assertions: Vec::new(),
        tombstones: Vec::new(),
        rationale: Presence::Value(TextValue {
            text: "second empty revision".into(),
            language: Presence::Missing,
            markup: Presence::Missing,
        }),
    });
    document.metadata.owned_heads[0].revision = next;
    Dictionary::from_document(document).unwrap()
}

fn revision_one_with_assertion(seed: &Dictionary) -> (Dictionary, EntityId, EntityId) {
    let mut document = revision_one(seed).document();
    let source = SourceId::new("owned-source").unwrap();
    let editor = ContributorId::new("owned-editor").unwrap();
    let capture = RawCapture::new(
        "owned:workbench-capture",
        CaptureMetadata {
            source_name: "Owned workbench fixture".into(),
            format_label: "application/json".into(),
            locator: "memory:owned-workbench".into(),
            revision: Presence::Value("one".into()),
            captured_at: Presence::Missing,
            time_precision: TimePrecision::Unknown,
            notice_locator: Presence::Missing,
        },
        br#"[{"word":"farvi","definition":"owned wording"},{"word":"farvi","definition":"owned alternative"}]"#.to_vec(),
        vec![
            RawOccurrence {
                locator: "/0".into(),
                fields: vec![NativeField {
                    name: "definition".into(),
                    encoding: NativeEncoding::Text,
                    raw_value: "owned wording".into(),
                }],
                raw_line: Some("farvi — owned wording".into()),
            },
            RawOccurrence {
                locator: "/1".into(),
                fields: vec![NativeField {
                    name: "definition".into(),
                    encoding: NativeEncoding::Text,
                    raw_value: "owned alternative".into(),
                }],
                raw_line: Some("farvi — owned alternative".into()),
            },
        ],
    );
    let capture_ref = capture.identity.clone();
    let occurrences = ["/0", "/1"].map(|locator| OccurrenceRef {
        capture: capture_ref.clone(),
        locator: locator.into(),
    });
    document.captures.push(capture);
    let entry_id = EntityId::new("owned-entry-farvi").unwrap();
    let assertion_ids = [
        EntityId::new("owned-assertion-farvi-a").unwrap(),
        EntityId::new("owned-assertion-farvi-b").unwrap(),
    ];
    document.metadata.entries.push(EntryRecord {
        id: entry_id.clone(),
        source: source.clone(),
        spelling: Presence::Value("farvi".into()),
        language: Presence::Value("jbo".into()),
        script: Presence::Missing,
        native_word_class: Presence::Missing,
        selmaho: Presence::Missing,
        mapped_pos: Presence::Missing,
    });
    for (index, (id, definition)) in assertion_ids
        .iter()
        .cloned()
        .zip(["owned wording", "owned alternative"])
        .enumerate()
    {
        let occurrence = occurrences[index].clone();
        document.metadata.assertions.push(AssertionRecord {
            id,
            entry: entry_id.clone(),
            occurrence: occurrence.clone(),
            native_sense_id: Presence::Missing,
            language: Presence::Value("en".into()),
            definition: Presence::Value(TextValue {
                text: definition.into(),
                language: Presence::Missing,
                markup: Presence::Missing,
            }),
            source_score: Presence::Missing,
            timestamps: Vec::new(),
            eligibility: Eligibility::Lexical,
        });
    }
    let mut registry = document.registry.document();
    registry.snapshots.push(SnapshotAttribution {
        snapshot: capture_ref.to_snapshot_identity().unwrap(),
        source,
        distributor: None,
    });
    for (index, occurrence) in occurrences.iter().enumerate() {
        registry.occurrences.push(OccurrenceAttribution {
            occurrence: occurrence.to_record_identity().unwrap(),
            contributor: Some(editor.clone()),
            native_entry_id: Some("farvi".into()),
            native_assertion_id: Some(format!(
                "owned-farvi-{}",
                if index == 0 { "a" } else { "b" }
            )),
        });
    }
    document.registry = SourceRegistry::from_document(registry).unwrap();
    let mut release = document.metadata.releases[0].clone();
    release.id = EntityId::new("owned-release-r1").unwrap();
    release.captures.push(capture_ref);
    release.assertions.extend(assertion_ids.iter().cloned());
    release.accounting.input_occurrences = Some(2);
    release.accounting.normalized_occurrences = Some(2);
    release.accounting.query_excluded_occurrences = Some(0);
    document.metadata.releases.push(release);
    document
        .metadata
        .owned_revisions
        .last_mut()
        .unwrap()
        .assertions
        .extend(assertion_ids.iter().cloned());
    (
        Dictionary::from_document(document).unwrap(),
        assertion_ids[0].clone(),
        assertion_ids[1].clone(),
    )
}

fn next_same_active_owned_revision(previous: &Dictionary, id: &str) -> Dictionary {
    let mut document = previous.document();
    let prior = document.metadata.owned_revisions.last().unwrap();
    let parent = prior.id.clone();
    let assertions = prior.assertions.clone();
    let revision = EntityId::new(id).unwrap();
    document.metadata.catalogue_revision += 1;
    document.metadata.owned_revisions.push(OwnedRevisionRecord {
        id: revision.clone(),
        source: SourceId::new("owned-source").unwrap(),
        parent: Some(parent),
        editor: ContributorId::new("owned-editor").unwrap(),
        timestamp: Presence::Missing,
        time_precision: TimePrecision::Unknown,
        assertions,
        tombstones: Vec::new(),
        rationale: Presence::Value(TextValue {
            text: "same active assertions at a curation-only workbench update".into(),
            language: Presence::Missing,
            markup: Presence::Missing,
        }),
    });
    document.metadata.owned_heads[0].revision = revision;
    Dictionary::from_document(document).unwrap()
}

fn fixture_target(
    context: &Dictionary,
    context_id: &EntityId,
    assertion_index: usize,
) -> OwnedPinnedTarget {
    let assertion = &context.metadata().assertions[assertion_index];
    let entry = context
        .metadata()
        .entries
        .iter()
        .find(|row| row.id == assertion.entry)
        .unwrap();
    let release = context
        .metadata()
        .releases
        .iter()
        .find(|row| row.assertions.contains(&assertion.id))
        .unwrap();
    let component = context
        .metadata()
        .components
        .iter()
        .find(|row| row.id == release.component)
        .unwrap();
    let collection = context
        .metadata()
        .collections
        .iter()
        .find(|row| row.id == component.collection)
        .unwrap();
    let occurrence = assertion.occurrence.to_record_identity().unwrap();
    let attribution = context
        .registry()
        .as_document()
        .occurrences
        .iter()
        .find(|row| row.occurrence == occurrence)
        .unwrap();
    OwnedPinnedTarget {
        context: context_id.clone(),
        target: CatalogTarget {
            source: collection.source.clone(),
            release: release.id.clone(),
            native_entry_id: attribution.native_entry_id.clone(),
            native_assertion_id: attribution.native_assertion_id.clone(),
            occurrence: Some(assertion.occurrence.clone()),
        },
        entry: entry.id.clone(),
        assertion: assertion.id.clone(),
    }
}

fn install_catalogue(root: &std::path::Path, pack: PackId, dictionary: &Dictionary) {
    install_catalogue_many(root, &[pack], dictionary);
}

fn install_catalogue_many(root: &std::path::Path, packs: &[PackId], dictionary: &Dictionary) {
    let dictionaries: Vec<_> = packs.iter().map(|pack| (*pack, dictionary)).collect();
    install_catalogue_inputs(root, &dictionaries);
}

fn install_catalogue_inputs(root: &std::path::Path, dictionaries: &[(PackId, &Dictionary)]) {
    let sealed: Vec<_> = dictionaries
        .iter()
        .map(|(pack, dictionary)| {
            seal_release(*pack, PackPayloadKind::Synthetic, dictionary).unwrap()
        })
        .collect();
    let inputs = dictionaries
        .iter()
        .zip(&sealed)
        .map(|((pack_id, _), release)| CollectionInput::SealedBytes {
            pack_id: *pack_id,
            release: Cow::Borrowed(&release.release),
            seal: Cow::Borrowed(&release.seal),
        })
        .collect();
    let request = CollectionRequest::new(
        inputs,
        CollectionSelection::Packs(dictionaries.iter().map(|(pack, _)| *pack).collect()),
        true,
    )
    .unwrap();
    CollectionCatalogue::install(root, request).unwrap();
}

#[test]
fn test_workbench_selection_rejects_unknown_duplicate_and_missing_fields() {
    let valid = br#"{"version":1,"packs":[],"disabled":[],"owned":"head","contexts":[]}"#;
    let selection = WorkbenchSelection::from_json_bytes(valid).unwrap();
    assert_eq!(selection, owned_head_selection());

    let duplicate =
        br#"{"version":1,"version":1,"packs":[],"disabled":[],"owned":"head","contexts":[]}"#;
    assert!(matches!(
        WorkbenchSelection::from_json_bytes(duplicate),
        Err(WorkbenchError::InvalidSelection { .. })
    ));
    let unknown =
        br#"{"version":1,"packs":[],"disabled":[],"owned":"head","contexts":[],"future":true}"#;
    assert!(matches!(
        WorkbenchSelection::from_json_bytes(unknown),
        Err(WorkbenchError::InvalidSelection { .. })
    ));
    let missing_nullable = br#"{"version":1,"packs":[],"disabled":[],"contexts":[]}"#;
    assert!(matches!(
        WorkbenchSelection::from_json_bytes(missing_nullable),
        Err(WorkbenchError::InvalidSelection { .. })
    ));
    let nested_unknown = br#"{"version":1,"packs":["llg"],"disabled":[],"owned":null,"contexts":[{"context":"context-a","packs":["llg"],"future":true}]}"#;
    assert!(matches!(
        WorkbenchSelection::from_json_bytes(nested_unknown),
        Err(WorkbenchError::InvalidSelection { .. })
    ));
}

#[test]
fn test_workbench_selection_rejects_contexts_without_enabled_selected_packs() {
    let context = EntityId::new("context-a").unwrap();
    let mut selection = WorkbenchSelection {
        version: 1,
        packs: vec![vlajmaji_lexicon::PackId::Llg],
        disabled: Vec::new(),
        owned: None,
        contexts: vec![vlajmaji_store::WorkbenchContextBinding {
            context: context.clone(),
            packs: vec![vlajmaji_lexicon::PackId::Llg],
        }],
    };
    assert!(matches!(
        selection.validate(),
        Err(WorkbenchError::InvalidSelection { .. })
    ));

    selection.owned = Some(WorkbenchOwnedRevision::Head);
    selection.contexts[0].packs.clear();
    assert!(matches!(
        selection.validate(),
        Err(WorkbenchError::InvalidSelection { .. })
    ));

    selection.contexts[0].packs = vec![vlajmaji_lexicon::PackId::Llg];
    selection.disabled = vec![vlajmaji_lexicon::PackId::Llg];
    assert!(matches!(
        selection.validate(),
        Err(WorkbenchError::InvalidSelection { .. })
    ));

    selection.disabled.clear();
    selection.contexts[0].packs =
        vec![vlajmaji_lexicon::PackId::Llg, vlajmaji_lexicon::PackId::Llg];
    assert!(matches!(
        selection.validate(),
        Err(WorkbenchError::InvalidSelection { .. })
    ));

    selection.contexts[0].packs = vec![vlajmaji_lexicon::PackId::LaLojban];
    assert!(matches!(
        selection.validate(),
        Err(WorkbenchError::InvalidSelection { .. })
    ));

    selection.contexts[0].packs = vec![vlajmaji_lexicon::PackId::Llg];
    selection.contexts.push(selection.contexts[0].clone());
    assert!(matches!(
        selection.validate(),
        Err(WorkbenchError::InvalidSelection { .. })
    ));
}

#[test]
fn test_workbench_owned_only_build_persists_checked_binding_and_store() {
    let parent = fresh_root("owned-build");
    let journal_root = parent.join("journal");
    let (writer, created) = OwnedJournal::create(&journal_root, &genesis()).unwrap();
    assert_eq!(created.head.revision.id.as_str(), "owned-r0");
    drop(writer);

    let output = parent.join("view");
    let request = WorkbenchBuildRequest {
        catalogue_root: None,
        owned_root: Some(journal_root.clone()),
        selection: owned_head_selection(),
    };
    let report = WorkbenchView::build_new(&output, request).unwrap();
    assert_eq!(report.operation, "build_new");
    assert_eq!(report.binding.owned().unwrap().identity.journal_schema, 1);
    assert_eq!(report.binding.selected_release_ids().len(), 1);

    let mut missing_nullable = serde_json::to_value(&report.binding).unwrap();
    missing_nullable["owned"]["identity"]
        .as_object_mut()
        .unwrap()
        .remove("curation_state_sha256");
    assert!(
        WorkbenchBinding::from_json_bytes(&serde_json::to_vec(&missing_nullable).unwrap()).is_err()
    );

    let mut nested_unknown = serde_json::to_value(&report.binding).unwrap();
    nested_unknown["owned"]["identity"]["future"] = serde_json::Value::Bool(true);
    assert!(
        WorkbenchBinding::from_json_bytes(&serde_json::to_vec(&nested_unknown).unwrap()).is_err()
    );

    let mut bad_digest = serde_json::to_value(&report.binding).unwrap();
    bad_digest["binding_sha256"] = serde_json::Value::String("0".repeat(64));
    assert!(WorkbenchBinding::from_json_bytes(&serde_json::to_vec(&bad_digest).unwrap()).is_err());

    let opened = WorkbenchView::open(&output).unwrap();
    assert_eq!(opened.binding(), &report.binding);
    let manifest = opened.snapshot_manifest();
    assert_eq!(manifest.generation, report.binding.binding_sha256());
    assert_eq!(
        manifest.selected_release_ids.as_deref(),
        Some(report.binding.selected_release_ids())
    );
    let snapshot =
        vlajmaji_store::DictionaryReadSnapshot::open(DictionaryReadTopology::Unified, manifest)
            .unwrap();
    assert_eq!(
        snapshot.logical_state_sha256(),
        report.binding.state_sha256()
    );

    let reader = OwnedJournalReader::open(&journal_root).unwrap();
    assert_eq!(
        reader.head_identity().unwrap(),
        report.binding.owned().unwrap().identity
    );
    drop(reader);
    fs::remove_dir_all(parent).unwrap();
}

#[test]
fn test_workbench_destination_collision_preserves_existing_sentinel() {
    let parent = fresh_root("collision");
    let journal_root = parent.join("journal");
    let (writer, _) = OwnedJournal::create(&journal_root, &genesis()).unwrap();
    drop(writer);
    let output = parent.join("occupied");
    fs::create_dir(&output).unwrap();
    fs::write(output.join("sentinel"), b"keep exact bytes\n").unwrap();

    let error = WorkbenchView::build_new(
        &output,
        WorkbenchBuildRequest {
            catalogue_root: None,
            owned_root: Some(journal_root),
            selection: owned_head_selection(),
        },
    )
    .unwrap_err();
    assert!(matches!(error, WorkbenchError::ExistingDestination(_)));
    assert_eq!(
        fs::read(output.join("sentinel")).unwrap(),
        b"keep exact bytes\n"
    );
    fs::remove_dir_all(parent).unwrap();
}

#[cfg(unix)]
#[test]
fn test_workbench_symlink_candidate_rejects_without_touching_target() {
    use std::os::unix::fs::symlink;

    let parent = fresh_root("symlink-candidate");
    let catalogue_root = parent.join("catalogue");
    install_catalogue(&catalogue_root, PackId::Llg, &genesis());
    let held_directory = parent.join("held-directory");
    fs::create_dir(&held_directory).unwrap();
    let sentinel = held_directory.join("sentinel");
    fs::write(
        &sentinel,
        b"preserve through rejected symlink destination\n",
    )
    .unwrap();
    let candidate = parent.join("candidate-link");
    symlink(&held_directory, &candidate).unwrap();

    let result = WorkbenchView::build_new(
        &candidate,
        WorkbenchBuildRequest {
            catalogue_root: Some(catalogue_root),
            owned_root: None,
            selection: WorkbenchSelection {
                version: 1,
                packs: vec![PackId::Llg],
                disabled: Vec::new(),
                owned: None,
                contexts: Vec::new(),
            },
        },
    );
    assert!(matches!(result, Err(WorkbenchError::ExistingDestination(path)) if path == candidate));
    assert_eq!(
        fs::read(&sentinel).unwrap(),
        b"preserve through rejected symlink destination\n"
    );
    assert!(!held_directory.join("binding.json").exists());
    fs::remove_dir_all(parent).unwrap();
}

#[test]
fn test_workbench_open_owned_head_stales_but_retained_historical_revision_remains_current() {
    let parent = fresh_root("owned-advance");
    let journal_root = parent.join("journal");
    let seed = genesis();
    let (mut writer, created) = OwnedJournal::create(&journal_root, &seed).unwrap();

    let head_root = parent.join("head-view");
    WorkbenchView::build_new(
        &head_root,
        WorkbenchBuildRequest {
            catalogue_root: None,
            owned_root: Some(journal_root.clone()),
            selection: owned_head_selection(),
        },
    )
    .unwrap();
    let historical_root = parent.join("historical-view");
    let mut historical = owned_head_selection();
    historical.owned = Some(WorkbenchOwnedRevision::Revision(
        EntityId::new("owned-r0").unwrap(),
    ));
    WorkbenchView::build_new(
        &historical_root,
        WorkbenchBuildRequest {
            catalogue_root: None,
            owned_root: Some(journal_root.clone()),
            selection: historical,
        },
    )
    .unwrap();

    writer
        .commit(&created.head.revision.id, &revision_one(&seed))
        .unwrap();
    drop(writer);

    assert!(matches!(
        WorkbenchView::open(&head_root),
        Err(WorkbenchError::StaleDependencies { .. })
    ));
    let retained = WorkbenchView::open(&historical_root).unwrap();
    assert_eq!(
        retained
            .binding()
            .owned()
            .unwrap()
            .identity
            .revision
            .as_str(),
        "owned-r0"
    );
    assert_eq!(retained.binding().owned().unwrap().identity.sequence, 0);
    fs::remove_dir_all(parent).unwrap();
}

#[test]
fn test_workbench_schema_two_migration_stales_old_binding_and_records_empty_curation_digest() {
    let parent = fresh_root("schema-migration");
    let journal_root = parent.join("journal");
    let seed = genesis();
    let (mut writer, created) = OwnedJournal::create(&journal_root, &seed).unwrap();
    let old_root = parent.join("schema-one-view");
    WorkbenchView::build_new(
        &old_root,
        WorkbenchBuildRequest {
            catalogue_root: None,
            owned_root: Some(journal_root.clone()),
            selection: owned_head_selection(),
        },
    )
    .unwrap();

    let migrated = writer.migrate_curation(&created.head.revision.id).unwrap();
    assert_eq!(migrated.from_schema, 1);
    assert_eq!(migrated.to_schema, 2);
    assert!(matches!(
        WorkbenchView::open(&old_root),
        Err(WorkbenchError::StaleDependencies { .. })
    ));

    let schema_two_root = parent.join("schema-two-view");
    WorkbenchView::build_new(
        &schema_two_root,
        WorkbenchBuildRequest {
            catalogue_root: None,
            owned_root: Some(journal_root),
            selection: owned_head_selection(),
        },
    )
    .unwrap();
    let fresh = WorkbenchView::open(&schema_two_root).unwrap();
    let identity = &fresh.binding().owned().unwrap().identity;
    assert_eq!(identity.journal_schema, 2);
    assert!(
        identity
            .curation_state_sha256
            .as_ref()
            .is_some_and(|digest| digest.len() == 64)
    );
    fs::remove_dir_all(parent).unwrap();
}

#[test]
fn test_workbench_open_uses_captured_catalogue_identity_without_rereading_release_files() {
    let parent = fresh_root("no-reread");
    let catalogue_root = parent.join("catalogue");
    let mut source_document = fixture::complete_fixture().unwrap().document();
    source_document.legacy_archive = None;
    let source_dictionary = Dictionary::from_document(source_document).unwrap();
    let expected_dictionary = compose_dictionaries(vec![source_dictionary.clone()]).unwrap();
    let source_input_bytes = source_dictionary.to_json_bytes().unwrap();
    let sealed = seal_release(PackId::Llg, PackPayloadKind::Synthetic, &source_dictionary).unwrap();
    let release_input_bytes = sealed.release.clone();
    let seal_input_bytes = sealed.seal.clone();
    let install = CollectionRequest::new(
        vec![CollectionInput::SealedBytes {
            pack_id: PackId::Llg,
            release: Cow::Borrowed(&sealed.release),
            seal: Cow::Borrowed(&sealed.seal),
        }],
        CollectionSelection::Packs(vec![PackId::Llg]),
        true,
    )
    .unwrap();
    CollectionCatalogue::install(&catalogue_root, install).unwrap();
    let catalogue = CollectionCatalogue::open(&catalogue_root).unwrap();
    let locations = catalogue
        .installed_artifact_locations(PackId::Llg)
        .unwrap()
        .unwrap();
    assert_eq!(
        fs::read(&locations.release_path).unwrap(),
        release_input_bytes
    );
    assert_eq!(fs::read(&locations.seal_path).unwrap(), seal_input_bytes);
    drop(catalogue);
    let selection = WorkbenchSelection {
        version: 1,
        packs: vec![PackId::Llg],
        disabled: Vec::new(),
        owned: None,
        contexts: Vec::new(),
    };
    let output = parent.join("view");
    let built = WorkbenchView::build_new(
        &output,
        WorkbenchBuildRequest {
            catalogue_root: Some(catalogue_root.clone()),
            owned_root: None,
            selection,
        },
    )
    .unwrap();
    let catalogue = CollectionCatalogue::open(&catalogue_root).unwrap();
    let locations = catalogue
        .installed_artifact_locations(PackId::Llg)
        .unwrap()
        .unwrap();
    drop(catalogue);
    let installed_release_bytes = fs::read(&locations.release_path).unwrap();
    let installed_seal_bytes = fs::read(&locations.seal_path).unwrap();
    assert_eq!(installed_release_bytes, release_input_bytes);
    assert_eq!(installed_seal_bytes, seal_input_bytes);
    fs::remove_file(locations.release_path).unwrap();
    fs::remove_file(locations.seal_path).unwrap();

    let opened = WorkbenchView::open(&output).unwrap();
    assert_eq!(opened.binding(), &built.binding);
    assert_eq!(opened.binding().catalogue().unwrap().local_revision, 1);
    let dictionary_store = DictionaryStore::open(output.join("dictionary")).unwrap();
    let actual_dictionary = dictionary_store.dictionary().unwrap();
    assert_eq!(actual_dictionary, expected_dictionary);
    assert_eq!(
        dictionary_store.canonical_bytes().unwrap(),
        expected_dictionary.to_json_bytes().unwrap(),
    );
    assert_eq!(
        source_dictionary.to_json_bytes().unwrap(),
        source_input_bytes,
        "building the upstream-only view must not mutate its typed input",
    );
    fs::remove_dir_all(parent).unwrap();
}

#[test]
fn test_workbench_build_retains_and_materializes_supplied_curation_context() {
    let parent = fresh_root("curation-context");
    let catalogue_root = parent.join("catalogue");
    let journal_root = parent.join("journal");

    let mut context_document = fixture::complete_fixture().unwrap().document();
    context_document.legacy_archive = None;
    let context = Dictionary::from_document(context_document).unwrap();
    let mut extra_registry = RegistryDocument::default();
    extra_registry.sources.push(SourceRecord {
        id: SourceId::new("supplemental-workbench-source").unwrap(),
        display_name: "Supplemental valid source".into(),
        abbreviation: "SWS".into(),
        abbreviation_kind: AbbreviationKind::ProjectAssigned,
        abbreviation_evidence: None,
        origin: SourceOrigin::Upstream,
    });
    let supplemental = Dictionary::from_document(DictionaryDocument {
        registry: SourceRegistry::from_document(extra_registry).unwrap(),
        ..DictionaryDocument::default()
    })
    .unwrap();
    let composed_context =
        compose_dictionaries(vec![context.clone(), supplemental.clone()]).unwrap();
    assert_ne!(
        context.registry().as_document().sources.len(),
        composed_context.registry().as_document().sources.len()
    );
    let reversed_composition =
        compose_dictionaries(vec![supplemental.clone(), context.clone()]).unwrap();
    assert_ne!(
        composed_context.to_json_bytes().unwrap(),
        reversed_composition.to_json_bytes().unwrap(),
        "the fixture must distinguish actual pack-composition order",
    );
    let context_id = EntityId::new("workbench-context").unwrap();
    let pin = OwnedContextPin::from_dictionary(context_id.clone(), &composed_context).unwrap();
    let second_context_id = EntityId::new("workbench-context-second").unwrap();
    let second_pin =
        OwnedContextPin::from_dictionary(second_context_id.clone(), &composed_context).unwrap();
    let first = fixture_target(&context, &context_id, 0);

    let genesis = genesis();
    let (candidate, owned_assertion_a, owned_assertion_b) = revision_one_with_assertion(&genesis);
    let original_owned_entries = candidate.metadata().entries.clone();
    let original_owned_assertions = candidate.metadata().assertions.clone();
    let r0 = EntityId::new("owned-r0").unwrap();
    let r1 = EntityId::new("owned-r1").unwrap();
    let sense_id = EntityId::new("workbench-external-sense").unwrap();
    let link_id = EntityId::new("workbench-sense-link").unwrap();
    let sense = OwnedCurationRecord {
        id: sense_id.clone(),
        revision: r1.clone(),
        editor: ContributorId::new("owned-editor").unwrap(),
        rationale: TextValue {
            text: "two owned members and one pinned external member".into(),
            language: Presence::Missing,
            markup: Presence::Missing,
        },
        kind: OwnedCurationKind::Sense {
            members: vec![
                OwnedSenseMember::OwnedAssertion(owned_assertion_a),
                OwnedSenseMember::OwnedAssertion(owned_assertion_b),
                OwnedSenseMember::External(first.clone()),
            ],
        },
    };
    let link = OwnedCurationRecord {
        id: link_id.clone(),
        revision: r1.clone(),
        editor: ContributorId::new("owned-editor").unwrap(),
        rationale: TextValue {
            text: "link the mixed sense to its pinned target".into(),
            language: Presence::Missing,
            markup: Presence::Missing,
        },
        kind: OwnedCurationKind::Link {
            origin: LocalRef::EditorialSense(sense_id.clone()),
            direction: Direction::Forward,
            target: first,
        },
    };
    let curation = OwnedCurationDocument::from_data(OwnedCurationData {
        version: 1,
        dictionary: candidate,
        contexts: vec![pin.clone(), second_pin.clone()],
        records: vec![sense, link],
        revisions: vec![
            OwnedCurationRevision {
                revision: r0.clone(),
                active: Vec::new(),
                tombstones: Vec::new(),
            },
            OwnedCurationRevision {
                revision: r1.clone(),
                active: vec![sense_id.clone(), link_id.clone()],
                tombstones: Vec::new(),
            },
        ],
    })
    .unwrap();

    let (mut writer, _) = OwnedJournal::create(&journal_root, &genesis).unwrap();
    writer.migrate_curation(&r0).unwrap();
    writer.commit_curated(&r0, &curation).unwrap();
    drop(writer);
    install_catalogue_inputs(
        &catalogue_root,
        &[(PackId::Llg, &context), (PackId::LaLojban, &supplemental)],
    );

    let view_root = parent.join("view");
    let selection = WorkbenchSelection {
        version: 1,
        packs: vec![PackId::Llg, PackId::LaLojban],
        disabled: Vec::new(),
        owned: Some(WorkbenchOwnedRevision::Head),
        contexts: vec![
            WorkbenchContextBinding {
                context: second_context_id,
                packs: vec![PackId::Llg, PackId::LaLojban],
            },
            WorkbenchContextBinding {
                context: context_id,
                packs: vec![PackId::Llg, PackId::LaLojban],
            },
        ],
    };
    let report = WorkbenchView::build_new(
        &view_root,
        WorkbenchBuildRequest {
            catalogue_root: Some(catalogue_root.clone()),
            owned_root: Some(journal_root.clone()),
            selection: selection.clone(),
        },
    )
    .unwrap();

    assert_eq!(report.binding.contexts().len(), 2);
    let observed = &report.binding.contexts()[0];
    assert_eq!(observed.pin, pin);
    assert_eq!(observed.packs, Some(vec![PackId::Llg, PackId::LaLojban]));
    assert_eq!(
        observed.observed_model_sha256.as_deref(),
        Some(pin.model_sha256.as_str())
    );
    assert_eq!(report.binding.contexts()[1].pin, second_pin);
    assert_eq!(
        report.binding.contexts()[1].packs,
        Some(vec![PackId::Llg, PackId::LaLojban])
    );
    assert_eq!(report.binding.resolutions().len(), 2);
    assert_eq!(report.binding.resolutions()[0].record, sense_id);
    assert_eq!(
        report.binding.resolutions()[0].status,
        vlajmaji_lexicon::OwnedCurationStatus::Resolved
    );
    assert!(report.binding.resolutions()[0].projected);
    assert_eq!(report.binding.resolutions()[1].record, link_id);
    assert_eq!(
        report.binding.resolutions()[1].status,
        vlajmaji_lexicon::OwnedCurationStatus::Resolved
    );
    assert!(report.binding.resolutions()[1].projected);
    let valid_binding_bytes = serde_json::to_vec(&report.binding).unwrap();
    assert_eq!(
        WorkbenchBinding::from_json_bytes(&valid_binding_bytes).unwrap(),
        report.binding,
        "a complete resolved row with explicit nullable fields remains valid",
    );
    let complete_wire = serde_json::to_value(&report.binding).unwrap();
    assert!(complete_wire["resolutions"][0]["reason"].is_null());
    assert!(complete_wire["resolutions"][0]["member"].is_null());
    for field in ["reason", "member"] {
        let mut missing_nullable = complete_wire.clone();
        missing_nullable["resolutions"][0]
            .as_object_mut()
            .unwrap()
            .remove(field);
        assert!(matches!(
            WorkbenchBinding::from_json_bytes(&serde_json::to_vec(&missing_nullable).unwrap()),
            Err(WorkbenchError::InvalidBinding { .. })
        ));
    }

    let conflicting_source = Dictionary::from_document(DictionaryDocument {
        registry: SourceRegistry::from_document(RegistryDocument {
            sources: vec![SourceRecord {
                id: SourceId::new("owned-source").unwrap(),
                display_name: "Different upstream body for the owned identity".into(),
                abbreviation: "OTHER".into(),
                abbreviation_kind: AbbreviationKind::ProjectAssigned,
                abbreviation_evidence: None,
                origin: SourceOrigin::Upstream,
            }],
            ..RegistryDocument::default()
        })
        .unwrap(),
        ..DictionaryDocument::default()
    })
    .unwrap();
    assert_eq!(
        conflicting_source.registry().as_document().sources[0]
            .id
            .as_str(),
        "owned-source",
    );
    assert_ne!(
        conflicting_source.registry().as_document().sources[0].display_name,
        genesis.registry().as_document().sources[0].display_name,
    );
    let conflicting_catalogue = parent.join("same-role-conflicting-catalogue");
    install_catalogue(&conflicting_catalogue, PackId::Llg, &conflicting_source);
    let conflict_destination = parent.join("same-role-conflict-view");
    let same_role_conflict = WorkbenchView::build_new(
        &conflict_destination,
        WorkbenchBuildRequest {
            catalogue_root: Some(conflicting_catalogue),
            owned_root: Some(journal_root.clone()),
            selection: WorkbenchSelection {
                version: 1,
                packs: vec![PackId::Llg],
                disabled: Vec::new(),
                owned: Some(WorkbenchOwnedRevision::Head),
                contexts: Vec::new(),
            },
        },
    );
    assert!(matches!(
        same_role_conflict,
        Err(WorkbenchError::Pack(error))
            if matches!(error.as_ref(), PackError::Conflict { category: "source", key } if key == "owned-source")
    ));
    assert!(!conflict_destination.exists());

    let missing_context = WorkbenchView::build_new(
        parent.join("missing-context-view"),
        WorkbenchBuildRequest {
            catalogue_root: Some(parent.join("catalogue")),
            owned_root: Some(parent.join("journal")),
            selection: WorkbenchSelection {
                version: 1,
                packs: vec![PackId::Llg],
                disabled: Vec::new(),
                owned: Some(WorkbenchOwnedRevision::Head),
                contexts: Vec::new(),
            },
        },
    )
    .unwrap();
    assert_eq!(missing_context.binding.contexts().len(), 2);
    assert_eq!(missing_context.binding.contexts()[0].packs, None);
    assert_eq!(
        missing_context.binding.contexts()[0].observed_model_sha256,
        None
    );
    assert_eq!(
        missing_context.binding.resolutions()[0].status,
        vlajmaji_lexicon::OwnedCurationStatus::Unavailable
    );
    assert_eq!(
        missing_context.binding.resolutions()[0].reason,
        Some(vlajmaji_lexicon::OwnedCurationReason::MissingContext)
    );
    assert_eq!(
        missing_context.binding.resolutions()[1].status,
        vlajmaji_lexicon::OwnedCurationStatus::Unresolved
    );
    assert_eq!(
        missing_context.binding.resolutions()[1].reason,
        Some(vlajmaji_lexicon::OwnedCurationReason::DependencyUnavailable)
    );

    let undeclared_pin = WorkbenchView::build_new(
        parent.join("undeclared-pin-view"),
        WorkbenchBuildRequest {
            catalogue_root: Some(parent.join("catalogue")),
            owned_root: Some(parent.join("journal")),
            selection: WorkbenchSelection {
                version: 1,
                packs: vec![PackId::Llg, PackId::LaLojban],
                disabled: Vec::new(),
                owned: Some(WorkbenchOwnedRevision::Head),
                contexts: vec![WorkbenchContextBinding {
                    context: EntityId::new("not-a-declared-pin").unwrap(),
                    packs: vec![PackId::Llg],
                }],
            },
        },
    );
    assert!(matches!(
        undeclared_pin,
        Err(WorkbenchError::InvalidSelection { .. })
    ));
    assert!(!parent.join("undeclared-pin-view").exists());

    let reversed_body = WorkbenchView::build_new(
        parent.join("reversed-context-view"),
        WorkbenchBuildRequest {
            catalogue_root: Some(parent.join("catalogue")),
            owned_root: Some(parent.join("journal")),
            selection: WorkbenchSelection {
                version: 1,
                packs: vec![PackId::Llg, PackId::LaLojban],
                disabled: Vec::new(),
                owned: Some(WorkbenchOwnedRevision::Head),
                contexts: vec![WorkbenchContextBinding {
                    context: pin.id.clone(),
                    packs: vec![PackId::LaLojban, PackId::Llg],
                }],
            },
        },
    );
    assert!(matches!(
        reversed_body,
        Err(WorkbenchError::Curation(error))
            if matches!(error.as_ref(), vlajmaji_lexicon::OwnedCurationError::ContextMismatch { .. })
    ));
    assert!(!parent.join("reversed-context-view").exists());

    let mut changed_document = context.document();
    let mut changed_registry = changed_document.registry.document();
    changed_registry.sources[0].display_name = "different canonical context body".into();
    changed_document.registry = SourceRegistry::from_document(changed_registry).unwrap();
    let changed_context = Dictionary::from_document(changed_document).unwrap();
    let changed_catalogue = parent.join("changed-catalogue");
    install_catalogue_inputs(
        &changed_catalogue,
        &[
            (PackId::Llg, &changed_context),
            (PackId::LaLojban, &supplemental),
        ],
    );
    let changed_body = WorkbenchView::build_new(
        parent.join("changed-context-view"),
        WorkbenchBuildRequest {
            catalogue_root: Some(changed_catalogue),
            owned_root: Some(parent.join("journal")),
            selection: WorkbenchSelection {
                version: 1,
                packs: vec![PackId::Llg, PackId::LaLojban],
                disabled: Vec::new(),
                owned: Some(WorkbenchOwnedRevision::Head),
                contexts: vec![WorkbenchContextBinding {
                    context: pin.id.clone(),
                    packs: vec![PackId::Llg, PackId::LaLojban],
                }],
            },
        },
    );
    assert!(matches!(
        changed_body,
        Err(WorkbenchError::Curation(error))
            if matches!(error.as_ref(), vlajmaji_lexicon::OwnedCurationError::ContextMismatch { .. })
    ));
    assert!(!parent.join("changed-context-view").exists());

    let previous_curation_state = curation.state_sha256().unwrap();
    let mut next_data = curation.as_data().clone();
    let r2 = EntityId::new("owned-r2").unwrap();
    next_data.dictionary = next_same_active_owned_revision(&next_data.dictionary, "owned-r2");
    assert_eq!(
        next_data.dictionary.metadata().entries,
        original_owned_entries,
        "a same-active revision preserves complete owned entry bodies",
    );
    assert_eq!(
        next_data.dictionary.metadata().assertions,
        original_owned_assertions,
        "a same-active revision preserves complete eligible owned assertion bodies",
    );
    let current_link = next_data
        .records
        .iter()
        .find_map(|record| match &record.kind {
            OwnedCurationKind::Link {
                origin,
                direction,
                target,
            } if record.id == link_id => Some((origin.clone(), *direction, target.clone())),
            _ => None,
        })
        .unwrap();
    let next_link_id = EntityId::new("workbench-sense-link-r2").unwrap();
    next_data.records.push(OwnedCurationRecord {
        id: next_link_id.clone(),
        revision: r2.clone(),
        editor: ContributorId::new("owned-editor").unwrap(),
        rationale: TextValue {
            text: "append a curation-only link while retaining the same active assertions".into(),
            language: Presence::Missing,
            markup: Presence::Missing,
        },
        kind: OwnedCurationKind::Link {
            origin: current_link.0,
            direction: current_link.1,
            target: current_link.2,
        },
    });
    let mut active = next_data.revisions.last().unwrap().active.clone();
    active.push(next_link_id.clone());
    next_data.revisions.push(OwnedCurationRevision {
        revision: r2.clone(),
        active,
        tombstones: Vec::new(),
    });
    let next_curation = OwnedCurationDocument::from_data(next_data).unwrap();
    assert_ne!(
        next_curation.state_sha256().unwrap(),
        previous_curation_state
    );

    let mut writer = OwnedJournal::open(&journal_root).unwrap();
    writer.commit_curated(&r1, &next_curation).unwrap();
    drop(writer);
    assert!(matches!(
        WorkbenchView::open(&view_root),
        Err(WorkbenchError::StaleDependencies {
            dependency: vlajmaji_store::WorkbenchDependency::Owned,
            ..
        })
    ));
    let refreshed_root = parent.join("curation-only-head-view");
    let refreshed = WorkbenchView::build_new(
        &refreshed_root,
        WorkbenchBuildRequest {
            catalogue_root: Some(catalogue_root),
            owned_root: Some(journal_root),
            selection,
        },
    )
    .unwrap();
    let refreshed_identity = &refreshed.binding.owned().unwrap().identity;
    assert_eq!(refreshed_identity.revision.as_str(), "owned-r2");
    assert!(
        refreshed
            .binding
            .selected_release_ids()
            .iter()
            .any(|release| release.as_str() == "owned-release-r1")
    );
    let refreshed_dictionary =
        vlajmaji_store::DictionaryStore::open(refreshed_root.join("dictionary"))
            .unwrap()
            .dictionary()
            .unwrap();
    let current_owned_revision = refreshed_dictionary
        .metadata()
        .owned_revisions
        .last()
        .unwrap();
    assert_eq!(current_owned_revision.id.as_str(), "owned-r2");
    assert_eq!(
        current_owned_revision.assertions,
        original_owned_assertions
            .iter()
            .map(|assertion| assertion.id.clone())
            .collect::<Vec<_>>(),
        "the current curation-only revision retains the prior active assertion IDs",
    );
    for expected in &original_owned_entries {
        assert_eq!(
            refreshed_dictionary
                .metadata()
                .entries
                .iter()
                .find(|actual| actual.id == expected.id),
            Some(expected),
            "the refreshed composed dictionary retains each complete owned entry body",
        );
    }
    for expected in &original_owned_assertions {
        assert_eq!(
            refreshed_dictionary
                .metadata()
                .assertions
                .iter()
                .find(|actual| actual.id == expected.id),
            Some(expected),
            "the refreshed composed dictionary retains each complete owned assertion body",
        );
    }
    assert!(
        refreshed_dictionary
            .metadata()
            .releases
            .iter()
            .any(|release| release.id.as_str() == "owned-release-r1"
                && release.availability == Availability::Available)
    );
    assert_ne!(
        refreshed_identity.curation_state_sha256.as_deref(),
        report
            .binding
            .owned()
            .unwrap()
            .identity
            .curation_state_sha256
            .as_deref()
    );
    assert_eq!(refreshed.binding.resolutions().len(), 3);
    assert_eq!(refreshed.binding.resolutions()[2].record, next_link_id);
    assert_eq!(
        refreshed.binding.resolutions()[2].status,
        vlajmaji_lexicon::OwnedCurationStatus::Resolved
    );
    fs::remove_dir_all(parent).unwrap();
}

#[test]
fn test_workbench_single_pack_uses_original_context_without_composition_deduplication() {
    let parent = fresh_root("single-context-direct");
    let catalogue_root = parent.join("catalogue");
    let journal_root = parent.join("journal");
    let mut document = fixture::complete_fixture().unwrap().document();
    document.legacy_archive = None;
    document
        .metadata
        .code_notices
        .push(document.metadata.code_notices[0].clone());
    let mut registry = document.registry.document();
    registry.lineage.push(registry.lineage[0].clone());
    document.registry = SourceRegistry::from_document(registry).unwrap();
    let context = Dictionary::from_document(document).unwrap();
    let composed_single = compose_dictionaries(vec![context.clone()]).unwrap();
    assert_ne!(
        context.to_json_bytes().unwrap(),
        composed_single.to_json_bytes().unwrap(),
        "anonymous code notices and lineage distinguish direct singleton use from composition",
    );

    let context_id = EntityId::new("single-direct-context").unwrap();
    let pin = OwnedContextPin::from_dictionary(context_id.clone(), &context).unwrap();
    let r0 = EntityId::new("owned-r0").unwrap();
    let r1 = EntityId::new("owned-r1").unwrap();
    let record_id = EntityId::new("single-direct-sense").unwrap();
    let genesis = genesis();
    let curation = OwnedCurationDocument::from_data(OwnedCurationData {
        version: 1,
        dictionary: revision_one(&genesis),
        contexts: vec![pin.clone()],
        records: vec![OwnedCurationRecord {
            id: record_id.clone(),
            revision: r1.clone(),
            editor: ContributorId::new("owned-editor").unwrap(),
            rationale: TextValue {
                text: "the singleton input is retained byte-for-byte".into(),
                language: Presence::Missing,
                markup: Presence::Missing,
            },
            kind: OwnedCurationKind::Sense {
                members: vec![
                    OwnedSenseMember::External(fixture_target(&context, &context_id, 0)),
                    OwnedSenseMember::External(fixture_target(&context, &context_id, 1)),
                ],
            },
        }],
        revisions: vec![
            OwnedCurationRevision {
                revision: r0.clone(),
                active: Vec::new(),
                tombstones: Vec::new(),
            },
            OwnedCurationRevision {
                revision: r1,
                active: vec![record_id],
                tombstones: Vec::new(),
            },
        ],
    })
    .unwrap();
    let (mut writer, _) = OwnedJournal::create(&journal_root, &genesis).unwrap();
    writer.migrate_curation(&r0).unwrap();
    writer.commit_curated(&r0, &curation).unwrap();
    drop(writer);
    install_catalogue(&catalogue_root, PackId::Llg, &context);

    let report = WorkbenchView::build_new(
        parent.join("view"),
        WorkbenchBuildRequest {
            catalogue_root: Some(catalogue_root),
            owned_root: Some(journal_root),
            selection: WorkbenchSelection {
                version: 1,
                packs: vec![PackId::Llg],
                disabled: Vec::new(),
                owned: Some(WorkbenchOwnedRevision::Head),
                contexts: vec![WorkbenchContextBinding {
                    context: context_id,
                    packs: vec![PackId::Llg],
                }],
            },
        },
    )
    .unwrap();
    assert_eq!(
        report.binding.contexts()[0]
            .observed_model_sha256
            .as_deref(),
        Some(pin.model_sha256.as_str()),
    );
    assert_eq!(
        report.binding.resolutions()[0].record.as_str(),
        "single-direct-sense"
    );
    assert_eq!(
        report.binding.resolutions()[0].status,
        vlajmaji_lexicon::OwnedCurationStatus::Resolved
    );
    fs::remove_dir_all(parent).unwrap();
}

#[test]
fn test_workbench_disabled_missing_catalogue_member_is_not_opened() {
    let parent = fresh_root("disabled-missing-pack");
    let catalogue_root = parent.join("catalogue");
    let journal_root = parent.join("journal");
    let (writer, _) = OwnedJournal::create(&journal_root, &genesis()).unwrap();
    drop(writer);
    let mut context_document = fixture::complete_fixture().unwrap().document();
    context_document.legacy_archive = None;
    let other_pack = Dictionary::from_document(context_document).unwrap();
    install_catalogue(&catalogue_root, PackId::LaLojban, &other_pack);

    let report = WorkbenchView::build_new(
        parent.join("view"),
        WorkbenchBuildRequest {
            catalogue_root: Some(catalogue_root),
            owned_root: Some(journal_root),
            selection: WorkbenchSelection {
                version: 1,
                packs: vec![PackId::Llg],
                disabled: vec![PackId::Llg],
                owned: Some(WorkbenchOwnedRevision::Head),
                contexts: Vec::new(),
            },
        },
    )
    .unwrap();

    assert!(report.binding.catalogue().unwrap().members.is_empty());
    assert_eq!(report.binding.selected_release_ids().len(), 1);

    let enabled_missing = WorkbenchView::build_new(
        parent.join("enabled-missing-view"),
        WorkbenchBuildRequest {
            catalogue_root: Some(parent.join("catalogue")),
            owned_root: Some(parent.join("journal")),
            selection: WorkbenchSelection {
                version: 1,
                packs: vec![PackId::Llg],
                disabled: Vec::new(),
                owned: Some(WorkbenchOwnedRevision::Head),
                contexts: Vec::new(),
            },
        },
    );
    assert!(matches!(
        enabled_missing,
        Err(WorkbenchError::Collection(error))
            if matches!(error.as_ref(), vlajmaji_store::CollectionError::InvalidRequest(detail)
                if detail == "pack llg is not registered")
    ));
    assert!(!parent.join("enabled-missing-view").exists());
    fs::remove_dir_all(parent).unwrap();
}
