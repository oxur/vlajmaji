//! Public Full/Compact reference contracts over a small synthetic catalogue.

use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    borrow::Cow,
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

use vlajmaji_lexicon::{
    AbbreviationKind, AssertionRecord, Availability, CaptureMetadata, CaptureRef, CatalogTarget,
    CollectionRecord, ComponentRecord, ContentNotice, ContributorId, ContributorRecord, Dictionary,
    DictionaryDocument, DictionaryMetadata, EditorialSenseRecord, Eligibility, EntityId,
    EntryRecord, FormKind, FormRecord, LocalRef, OccurrenceAttribution, OccurrenceRef,
    OwnedRevisionRecord, PackId, PackPayloadKind, Presence, RawCapture, RawOccurrence,
    RedistributionStatus, RegistryDocument, RelationTarget, ReleaseAccounting, ReleaseRecord,
    SnapshotAttribution, SourceEvidence, SourceId, SourceOrigin, SourceRecord, SourceRegistry,
    TextValue, TimePrecision, seal_release,
};
use vlajmaji_search::{
    DictionaryQueryError, ReferenceEntity, ReferenceStatus, WorkbenchQueryError,
    WorkbenchSearchEngine,
};
use vlajmaji_store::{
    CollectionCatalogue, CollectionEntityReference, CollectionInput, CollectionRequest,
    CollectionSelection, OwnedJournal, WorkbenchBuildRequest, WorkbenchOwnedRevision,
    WorkbenchSelection, WorkbenchView,
};

static NEXT_ROOT: AtomicUsize = AtomicUsize::new(0);

fn fresh_root(label: &str) -> PathBuf {
    for _ in 0..64 {
        let root = std::env::temp_dir().join(format!(
            "vlajmaji-s03-reference-{label}-{}-{}",
            std::process::id(),
            NEXT_ROOT.fetch_add(1, Ordering::Relaxed)
        ));
        match fs::create_dir(&root) {
            Ok(()) => return root,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => panic!("create exclusive test root: {error}"),
        }
    }
    panic!("could not allocate a unique test root")
}

fn typed_dictionary(spelling: &str) -> (Dictionary, SourceId, EntityId, OccurrenceRef) {
    typed_dictionary_for("reference", spelling)
}

fn typed_dictionary_for(
    prefix: &str,
    spelling: &str,
) -> (Dictionary, SourceId, EntityId, OccurrenceRef) {
    let source = SourceId::new(format!("{prefix}-upstream")).unwrap();
    let collection = EntityId::new(format!("{prefix}-collection")).unwrap();
    let component = EntityId::new(format!("{prefix}-component")).unwrap();
    let release = EntityId::new(format!("{prefix}-release")).unwrap();
    let entry = EntityId::new(format!("{prefix}-entry")).unwrap();
    let assertion = EntityId::new(format!("{prefix}-assertion")).unwrap();
    let revision = EntityId::new(format!("{prefix}-revision")).unwrap();
    let editor = ContributorId::new(format!("{prefix}-editor")).unwrap();
    let capture = RawCapture::new(
        format!("captured:{prefix}:source"),
        CaptureMetadata {
            source_name: format!("{prefix} fixture"),
            format_label: "synthetic-reference".into(),
            locator: "/0".into(),
            revision: Presence::Missing,
            captured_at: Presence::Missing,
            time_precision: TimePrecision::Unknown,
            notice_locator: Presence::Missing,
        },
        b"reference fixture row\n".to_vec(),
        vec![RawOccurrence {
            locator: "/0".into(),
            fields: Vec::new(),
            raw_line: Some("reference fixture row".into()),
        }],
    );
    let occurrence = OccurrenceRef {
        capture: capture.identity.clone(),
        locator: "/0".into(),
    };
    let registry = SourceRegistry::from_document(RegistryDocument {
        version: 1,
        sources: vec![SourceRecord {
            id: source.clone(),
            display_name: format!("{prefix} fixture"),
            abbreviation: if prefix == "alternate" {
                "ALT".into()
            } else {
                "REF".into()
            },
            abbreviation_kind: AbbreviationKind::ProjectAssigned,
            abbreviation_evidence: Some("test fixture".into()),
            origin: SourceOrigin::OwnedDraft,
        }],
        contributors: vec![ContributorRecord {
            id: editor.clone(),
            display_name: format!("{prefix} fixture editor"),
        }],
        distributors: Vec::new(),
        aliases: Vec::new(),
        lineage: Vec::new(),
        snapshots: vec![SnapshotAttribution {
            snapshot: occurrence.capture.to_snapshot_identity().unwrap(),
            source: source.clone(),
            distributor: None,
        }],
        occurrences: vec![OccurrenceAttribution {
            occurrence: occurrence.to_record_identity().unwrap(),
            contributor: None,
            native_entry_id: Some("native-entry-0".into()),
            native_assertion_id: Some("native-assertion-0".into()),
        }],
    })
    .unwrap();
    let metadata = DictionaryMetadata {
        collections: vec![CollectionRecord {
            id: collection.clone(),
            source: source.clone(),
            display_name: format!("{prefix} fixture"),
            description: Presence::Missing,
        }],
        components: vec![ComponentRecord {
            id: component.clone(),
            collection,
            native_name: prefix.into(),
            languages: vec!["en".into()],
        }],
        releases: vec![ReleaseRecord {
            id: release.clone(),
            component,
            distributor: None,
            upstream_revision: Presence::Missing,
            distributor_revision: Presence::Missing,
            captured_at: Presence::Missing,
            time_precision: TimePrecision::Unknown,
            schema_version: 1,
            normalizer: "fixture".into(),
            normalizer_version: "1".into(),
            captures: vec![occurrence.capture.clone()],
            assertions: vec![assertion.clone()],
            accounting: ReleaseAccounting {
                input_occurrences: Some(1),
                normalized_occurrences: Some(1),
                query_excluded_occurrences: Some(0),
                unit: "occurrences".into(),
            },
            diagnostics: Vec::new(),
            selected: true,
            availability: Availability::Available,
            content_notices: vec![ContentNotice {
                text: Presence::Value("test fixture only".into()),
                locator: Presence::Missing,
                license: Presence::Missing,
            }],
            redistribution: RedistributionStatus::Held,
            redistribution_evidence: Vec::new(),
        }],
        entries: vec![EntryRecord {
            id: entry.clone(),
            source: source.clone(),
            spelling: Presence::Value(spelling.into()),
            language: Presence::Missing,
            script: Presence::Missing,
            native_word_class: Presence::Missing,
            selmaho: Presence::Missing,
            mapped_pos: Presence::Missing,
        }],
        assertions: vec![AssertionRecord {
            id: assertion.clone(),
            entry: entry.clone(),
            occurrence: occurrence.clone(),
            native_sense_id: Presence::Missing,
            language: Presence::Missing,
            definition: Presence::Value(TextValue {
                text: format!("a controlled {prefix} fixture"),
                language: Presence::Missing,
                markup: Presence::Missing,
            }),
            source_score: Presence::Missing,
            timestamps: Vec::new(),
            eligibility: Eligibility::Lexical,
        }],
        forms: vec![FormRecord {
            id: EntityId::new(format!("{prefix}-form")).unwrap(),
            entry: entry.clone(),
            spelling: format!("{spelling}-form"),
            language: Presence::Missing,
            script: Presence::Missing,
            pronunciation: Presence::Missing,
            kind: FormKind::SourceVariant,
            standing: Presence::Missing,
            components: Vec::new(),
            evidence: Some(SourceEvidence {
                occurrence: occurrence.clone(),
                field_index: None,
                citation: Presence::Missing,
                offset: None,
            }),
            computation: None,
        }],
        owned_revisions: vec![OwnedRevisionRecord {
            id: revision.clone(),
            source: source.clone(),
            parent: None,
            editor: editor.clone(),
            timestamp: Presence::Missing,
            time_precision: TimePrecision::Unknown,
            assertions: vec![assertion],
            tombstones: Vec::new(),
            rationale: Presence::Missing,
        }],
        owned_heads: vec![vlajmaji_lexicon::OwnedHead {
            source: source.clone(),
            revision: revision.clone(),
        }],
        editorial_senses: vec![
            EditorialSenseRecord {
                id: EntityId::new(format!("{prefix}-empty-sense")).unwrap(),
                assertions: Vec::new(),
                editor,
                revision: revision.clone(),
                rationale: TextValue {
                    text: "empty member list fixture".into(),
                    language: Presence::Missing,
                    markup: Presence::Missing,
                },
            },
            EditorialSenseRecord {
                id: EntityId::new(format!("{prefix}-populated-sense")).unwrap(),
                assertions: vec![EntityId::new(format!("{prefix}-assertion")).unwrap()],
                editor: ContributorId::new(format!("{prefix}-editor")).unwrap(),
                revision,
                rationale: TextValue {
                    text: "one linked assertion".into(),
                    language: Presence::Missing,
                    markup: Presence::Missing,
                },
            },
        ],
        ..DictionaryMetadata::default()
    };
    let dictionary = Dictionary::from_document(DictionaryDocument {
        registry,
        captures: vec![capture],
        metadata,
        ..DictionaryDocument::default()
    })
    .unwrap();
    (dictionary, source, release, occurrence)
}

fn rich_owned_history() -> (Dictionary, Dictionary) {
    let (seed, source, _release, occurrence) = typed_dictionary_for("mi", "mi");
    let mut genesis_document = seed.document();
    genesis_document.captures.clear();
    let mut registry = genesis_document.registry.document();
    registry.snapshots.clear();
    registry.occurrences.clear();
    genesis_document.registry = SourceRegistry::from_document(registry).unwrap();
    genesis_document.metadata.entries.clear();
    genesis_document.metadata.assertions.clear();
    genesis_document.metadata.forms.clear();
    genesis_document.metadata.editorial_senses.clear();
    genesis_document.metadata.releases[0].id = EntityId::new("mi-release-r0").unwrap();
    genesis_document.metadata.releases[0].captures.clear();
    genesis_document.metadata.releases[0].assertions.clear();
    genesis_document.metadata.releases[0]
        .accounting
        .input_occurrences = Some(0);
    genesis_document.metadata.releases[0]
        .accounting
        .normalized_occurrences = Some(0);
    genesis_document.metadata.owned_revisions[0].id = EntityId::new("mi-revision-r0").unwrap();
    genesis_document.metadata.owned_revisions[0]
        .assertions
        .clear();
    genesis_document.metadata.owned_heads[0].revision = EntityId::new("mi-revision-r0").unwrap();
    genesis_document.metadata.catalogue_revision = 0;
    let genesis = Dictionary::from_document(genesis_document).unwrap();

    let seed_document = seed.document();
    let mut capture = seed_document.captures[0].clone();
    capture.occurrences.push(RawOccurrence {
        locator: "/1".into(),
        fields: Vec::new(),
        raw_line: Some("reference fixture row".into()),
    });
    let second_occurrence = OccurrenceRef {
        capture: occurrence.capture.clone(),
        locator: "/1".into(),
    };
    let second_entry = EntityId::new("mi-entry-b").unwrap();
    let second_assertion = EntityId::new("mi-assertion-b").unwrap();
    let revision_zero = EntityId::new("mi-revision-r0").unwrap();
    let revision_one = EntityId::new("mi-revision-r1").unwrap();
    let assertion_a = EntityId::new("mi-assertion").unwrap();

    let mut candidate_document = genesis.document();
    candidate_document.captures = vec![capture.clone()];
    let mut registry = seed_document.registry.document();
    registry.occurrences.push(OccurrenceAttribution {
        occurrence: second_occurrence.to_record_identity().unwrap(),
        contributor: None,
        native_entry_id: Some("native-entry-1".into()),
        native_assertion_id: Some("native-assertion-1".into()),
    });
    candidate_document.registry = SourceRegistry::from_document(registry).unwrap();
    candidate_document.metadata.entries = seed_document.metadata.entries.clone();
    candidate_document.metadata.entries.push(EntryRecord {
        id: second_entry.clone(),
        source: source.clone(),
        spelling: Presence::Value("zz".into()),
        language: Presence::Missing,
        script: Presence::Missing,
        native_word_class: Presence::Missing,
        selmaho: Presence::Missing,
        mapped_pos: Presence::Missing,
    });
    candidate_document.metadata.assertions = seed_document.metadata.assertions.clone();
    let mut second_row = candidate_document.metadata.assertions[0].clone();
    second_row.id = second_assertion.clone();
    second_row.entry = second_entry;
    second_row.occurrence = second_occurrence;
    candidate_document.metadata.assertions.push(second_row);
    candidate_document.metadata.forms = seed_document.metadata.forms.clone();
    candidate_document.metadata.releases.push(ReleaseRecord {
        id: EntityId::new("mi-release-r1").unwrap(),
        component: seed_document.metadata.releases[0].component.clone(),
        distributor: None,
        upstream_revision: Presence::Missing,
        distributor_revision: Presence::Missing,
        captured_at: Presence::Missing,
        time_precision: TimePrecision::Unknown,
        schema_version: 1,
        normalizer: "fixture".into(),
        normalizer_version: "1".into(),
        captures: vec![capture.identity.clone()],
        assertions: vec![assertion_a.clone(), second_assertion.clone()],
        accounting: ReleaseAccounting {
            input_occurrences: Some(2),
            normalized_occurrences: Some(2),
            query_excluded_occurrences: Some(0),
            unit: "occurrences".into(),
        },
        diagnostics: Vec::new(),
        selected: true,
        availability: Availability::Available,
        content_notices: seed_document.metadata.releases[0].content_notices.clone(),
        redistribution: RedistributionStatus::Held,
        redistribution_evidence: Vec::new(),
    });
    let mut revision = genesis.metadata().owned_revisions[0].clone();
    revision.id = revision_one.clone();
    revision.parent = Some(revision_zero);
    revision.assertions = vec![assertion_a.clone(), second_assertion.clone()];
    candidate_document.metadata.owned_revisions.push(revision);
    candidate_document.metadata.owned_heads[0].revision = revision_one.clone();
    candidate_document.metadata.editorial_senses = vec![
        EditorialSenseRecord {
            id: EntityId::new("mi-empty-sense").unwrap(),
            assertions: Vec::new(),
            editor: ContributorId::new("mi-editor").unwrap(),
            revision: revision_one.clone(),
            rationale: TextValue {
                text: "empty member list fixture".into(),
                language: Presence::Missing,
                markup: Presence::Missing,
            },
        },
        EditorialSenseRecord {
            id: EntityId::new("mi-populated-sense").unwrap(),
            assertions: vec![second_assertion, assertion_a],
            editor: ContributorId::new("mi-editor").unwrap(),
            revision: revision_one,
            rationale: TextValue {
                text: "two linked assertions".into(),
                language: Presence::Missing,
                markup: Presence::Missing,
            },
        },
    ];
    candidate_document.metadata.catalogue_revision = 1;
    let rich = Dictionary::from_document(candidate_document).unwrap();
    (genesis, rich)
}

fn create_then_commit_rich_journal(
    root: &std::path::Path,
    genesis: &Dictionary,
    rich: &Dictionary,
) {
    let genesis_revision = genesis.metadata().owned_revisions[0].clone();
    let (mut writer, created) = OwnedJournal::create(root, genesis).unwrap();
    assert_eq!(created.head.sequence, 0);
    let committed = writer.commit(&genesis_revision.id, rich).unwrap();
    assert_eq!(committed.head.sequence, 1);
    assert_eq!(committed.head.revision.id.as_str(), "mi-revision-r1");
}

fn typed_row_sha<T: Serialize>(row: &T) -> String {
    format!("{:x}", Sha256::digest(serde_json::to_vec(row).unwrap()))
}

fn dictionary_with_form_spelling(dictionary: &Dictionary, spelling: &str) -> Dictionary {
    let mut document = dictionary.document();
    document.metadata.forms[0].spelling = spelling.into();
    Dictionary::from_document(document).unwrap()
}

fn dictionary_with_sense_rationale(dictionary: &Dictionary, rationale: &str) -> Dictionary {
    let mut document = dictionary.document();
    document.metadata.editorial_senses[1].rationale.text = rationale.into();
    Dictionary::from_document(document).unwrap()
}

fn dictionary_with_unknown_native_attribution(dictionary: &Dictionary) -> Dictionary {
    let mut document = dictionary.document();
    let mut registry = document.registry.document();
    for occurrence in &mut registry.occurrences {
        occurrence.native_entry_id = None;
        occurrence.native_assertion_id = None;
    }
    document.registry = SourceRegistry::from_document(registry).unwrap();
    Dictionary::from_document(document).unwrap()
}

fn dictionary_with_second_occurrence(dictionary: &Dictionary) -> (Dictionary, OccurrenceRef) {
    let mut document = dictionary.document();
    let capture = &mut document.captures[0];
    capture.occurrences.push(RawOccurrence {
        locator: "/1".into(),
        fields: Vec::new(),
        raw_line: Some("second reference fixture row".into()),
    });
    let second_occurrence = OccurrenceRef {
        capture: capture.identity.clone(),
        locator: "/1".into(),
    };
    let mut registry = document.registry.document();
    registry.occurrences.push(OccurrenceAttribution {
        occurrence: second_occurrence.to_record_identity().unwrap(),
        contributor: None,
        native_entry_id: Some("native-entry-0".into()),
        native_assertion_id: Some("native-assertion-0".into()),
    });
    document.registry = SourceRegistry::from_document(registry).unwrap();
    let assertion = &mut document.metadata.assertions[0];
    let mut second_assertion = assertion.clone();
    second_assertion.id = EntityId::new("reference-assertion-second").unwrap();
    second_assertion.occurrence = second_occurrence.clone();
    document.metadata.assertions.push(second_assertion.clone());
    let release = &mut document.metadata.releases[0];
    release.assertions.push(second_assertion.id.clone());
    release.accounting.input_occurrences = Some(2);
    release.accounting.normalized_occurrences = Some(2);
    document.metadata.owned_revisions[0]
        .assertions
        .push(second_assertion.id);
    (
        Dictionary::from_document(document).unwrap(),
        second_occurrence,
    )
}

fn install_dictionaries(root: &std::path::Path, dictionaries: &[(PackId, &Dictionary)]) {
    let sealed: Vec<_> = dictionaries
        .iter()
        .map(|(pack_id, dictionary)| {
            seal_release(*pack_id, PackPayloadKind::Synthetic, dictionary).unwrap()
        })
        .collect();
    let request = CollectionRequest::new(
        dictionaries
            .iter()
            .zip(&sealed)
            .map(|((pack_id, _), sealed)| CollectionInput::SealedBytes {
                pack_id: *pack_id,
                release: Cow::Borrowed(&sealed.release),
                seal: Cow::Borrowed(&sealed.seal),
            })
            .collect(),
        CollectionSelection::Packs(dictionaries.iter().map(|(pack_id, _)| *pack_id).collect()),
        true,
    )
    .unwrap();
    CollectionCatalogue::install(root, request).unwrap();
}

fn install_dictionary(root: &std::path::Path, dictionary: &Dictionary) {
    install_dictionaries(root, &[(PackId::Llg, dictionary)]);
}

fn register_dictionary(root: &std::path::Path, pack_id: PackId, dictionary: &Dictionary) -> String {
    let sealed = seal_release(pack_id, PackPayloadKind::Synthetic, dictionary).unwrap();
    let release_sha256 = format!("{:x}", Sha256::digest(&sealed.release));
    let request = CollectionRequest::new(
        vec![CollectionInput::SealedBytes {
            pack_id,
            release: Cow::Borrowed(&sealed.release),
            seal: Cow::Borrowed(&sealed.seal),
        }],
        CollectionSelection::Packs(vec![pack_id]),
        true,
    )
    .unwrap();
    CollectionCatalogue::register(root, request).unwrap();
    release_sha256
}

fn sqlite_exec(database: &std::path::Path, sql: &str) {
    let output = Command::new("sqlite3")
        .arg(database)
        .arg(sql)
        .output()
        .expect("sqlite3 must be available for the isolated catalogue mutation");
    assert!(
        output.status.success(),
        "sqlite3 mutation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn sqlite_scalar(database: &std::path::Path, sql: &str) -> String {
    let output = Command::new("sqlite3")
        .arg(database)
        .arg(sql)
        .output()
        .expect("sqlite3 must be available for the isolated catalogue mutation");
    assert!(
        output.status.success(),
        "sqlite3 query failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout)
        .expect("sqlite3 scalar output is UTF-8")
        .trim()
        .to_owned()
}

fn sqlite_blob(value: &[u8]) -> String {
    let mut literal = String::with_capacity(3 + value.len() * 2);
    literal.push_str("x'");
    for byte in value {
        use std::fmt::Write as _;
        write!(literal, "{byte:02x}").unwrap();
    }
    literal.push('\'');
    literal
}

fn decode_hex(value: &str) -> Vec<u8> {
    assert_eq!(value.len() % 2, 0);
    value
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}

fn build(root: &std::path::Path, catalogue: &std::path::Path, disabled: bool) {
    WorkbenchView::build_new(
        root,
        WorkbenchBuildRequest {
            catalogue_root: Some(catalogue.to_path_buf()),
            owned_root: None,
            selection: WorkbenchSelection {
                version: 1,
                packs: if disabled {
                    vec![PackId::Llg, PackId::LaLojban]
                } else {
                    vec![PackId::Llg]
                },
                disabled: disabled.then_some(vec![PackId::Llg]).unwrap_or_default(),
                owned: None,
                contexts: Vec::new(),
            },
        },
    )
    .unwrap();
}

fn form_descriptor(row: &FormRecord, source: &SourceId) -> CollectionEntityReference {
    CollectionEntityReference {
        role: "form".into(),
        id: row.id.to_string(),
        entry_ids: vec![row.entry.to_string()],
        assertion_ids: Vec::new(),
        source_ids: vec![source.to_string()],
        occurrence: row
            .evidence
            .as_ref()
            .map(|evidence| evidence.occurrence.clone()),
        evidence: row.evidence.clone(),
        spelling: None,
    }
}

fn sense_descriptor(
    dictionary: &Dictionary,
    row: &EditorialSenseRecord,
) -> CollectionEntityReference {
    let mut entry_ids = std::collections::BTreeSet::new();
    let mut source_ids = std::collections::BTreeSet::new();
    for assertion_id in &row.assertions {
        let assertion = dictionary
            .metadata()
            .assertions
            .iter()
            .find(|assertion| assertion.id == *assertion_id)
            .unwrap();
        let entry = dictionary
            .metadata()
            .entries
            .iter()
            .find(|entry| entry.id == assertion.entry)
            .unwrap();
        entry_ids.insert(entry.id.to_string());
        source_ids.insert(entry.source.to_string());
    }
    CollectionEntityReference {
        role: "editorial_sense".into(),
        id: row.id.to_string(),
        entry_ids: entry_ids.into_iter().collect(),
        assertion_ids: row.assertions.iter().map(ToString::to_string).collect(),
        source_ids: source_ids.into_iter().collect(),
        occurrence: None,
        evidence: None,
        spelling: None,
    }
}

fn target(source: SourceId, release: EntityId, occurrence: OccurrenceRef) -> RelationTarget {
    RelationTarget::Catalog(CatalogTarget {
        source,
        release,
        native_entry_id: Some("native-entry-0".into()),
        native_assertion_id: Some("native-assertion-0".into()),
        occurrence: Some(occurrence),
    })
}

#[test]
fn test_resolve_reference_admitted_catalog_and_local_targets_resolve() {
    let parent = fresh_root("admitted");
    let catalogue = parent.join("catalogue");
    let root = parent.join("workbench");
    let (dictionary, source, release, occurrence) = typed_dictionary("mi");
    install_dictionary(&catalogue, &dictionary);
    build(&root, &catalogue, false);

    let mut engine = WorkbenchSearchEngine::open(&root).unwrap();
    let catalog_target = target(source.clone(), release, occurrence);
    let catalog = engine.resolve_reference(&catalog_target).unwrap();
    assert_eq!(catalog.result.status, ReferenceStatus::Resolved);
    assert!(catalog.result.payload.is_some());
    assert_eq!(catalog.result.catalogue_matches.len(), 1);
    assert!(catalog.result.registrations[0].admitted);

    let mut compact_engine = WorkbenchSearchEngine::open(&root).unwrap();
    let compact = compact_engine
        .resolve_reference_compact(&catalog_target)
        .unwrap();
    assert_eq!(compact.result.status, ReferenceStatus::Resolved);
    assert!(compact.result.payload.is_some());
    assert_eq!(compact.result.catalogue_matches.len(), 1);
    assert!(compact.result.registrations[0].admitted);

    // The installed release, seal, and derived database are advisory copies
    // after the workbench has been built. Keep their exact task-owned bytes,
    // move only those copies aside, and exercise reference plus Compact detail
    // through the already-open engines while the copies are absent.
    let installed = CollectionCatalogue::open(&catalogue)
        .unwrap()
        .installed_artifact_locations(PackId::Llg)
        .unwrap()
        .unwrap();
    let installed_paths = [
        installed.release_path,
        installed.seal_path,
        installed.database_path,
    ];
    let mut held = Vec::new();
    for path in installed_paths {
        let bytes = fs::read(&path).unwrap();
        let file_name = path.file_name().unwrap().to_string_lossy();
        let held_path = path.with_file_name(format!("{file_name}.s03-held"));
        assert!(!held_path.exists());
        fs::rename(&path, &held_path).unwrap();
        held.push((path, held_path, bytes));
    }
    let full_without_advisory_copies = engine.resolve_reference(&catalog_target).unwrap();
    assert_eq!(
        full_without_advisory_copies.result.status,
        ReferenceStatus::Resolved
    );
    let compact_without_advisory_copies = compact_engine
        .resolve_reference_compact(&catalog_target)
        .unwrap();
    assert_eq!(
        compact_without_advisory_copies.result.status,
        ReferenceStatus::Resolved
    );
    assert_eq!(
        compact_without_advisory_copies.binding_sha256,
        compact.binding_sha256
    );
    let detail_ref = compact_without_advisory_copies
        .result
        .payload
        .as_ref()
        .and_then(|payload| payload.confirmed.first())
        .and_then(|row| row.detail_ref.as_ref())
        .expect("reference Compact result supplies its existing detail key");
    let detail = compact_engine
        .detail(detail_ref, &compact_without_advisory_copies.binding_sha256)
        .unwrap();
    assert_eq!(
        detail.binding_sha256,
        compact_without_advisory_copies.binding_sha256
    );
    assert!(detail.result.hit.is_some());
    for (path, held_path, bytes) in held.into_iter().rev() {
        fs::rename(held_path, &path).unwrap();
        assert_eq!(fs::read(path).unwrap(), bytes);
    }

    let local_target = RelationTarget::Local(vlajmaji_lexicon::LocalRef::Entry(
        EntityId::new("reference-entry").unwrap(),
    ));
    let local = engine.resolve_reference(&local_target).unwrap();
    assert_eq!(local.result.status, ReferenceStatus::Resolved);
    assert!(local.result.payload.is_some());

    let local_compact = compact_engine
        .resolve_reference_compact(&local_target)
        .unwrap();
    assert_eq!(local_compact.result.status, ReferenceStatus::Resolved);
    assert!(local_compact.result.payload.is_some());

    let empty_sense_target = RelationTarget::Local(LocalRef::EditorialSense(
        EntityId::new("reference-empty-sense").unwrap(),
    ));
    let empty_sense = engine.resolve_reference(&empty_sense_target).unwrap();
    assert_eq!(empty_sense.result.status, ReferenceStatus::Resolved);
    assert!(empty_sense.result.payload.is_some());
    assert!(empty_sense.result.registrations[0].sources.is_empty());
    let empty_sense_compact = compact_engine
        .resolve_reference_compact(&empty_sense_target)
        .unwrap();
    assert_eq!(empty_sense_compact.result.status, ReferenceStatus::Resolved);
    assert!(empty_sense_compact.result.payload.is_some());
    assert!(
        empty_sense_compact.result.registrations[0]
            .sources
            .is_empty()
    );

    let form_target =
        RelationTarget::Local(LocalRef::Form(EntityId::new("reference-form").unwrap()));
    let form = engine.resolve_reference(&form_target).unwrap();
    assert_eq!(form.result.status, ReferenceStatus::Resolved);
    assert!(form.result.payload.is_some());

    let populated_sense_target = RelationTarget::Local(LocalRef::EditorialSense(
        EntityId::new("reference-populated-sense").unwrap(),
    ));
    let populated_sense = engine.resolve_reference(&populated_sense_target).unwrap();
    assert_eq!(populated_sense.result.status, ReferenceStatus::Resolved);
    assert!(populated_sense.result.payload.is_some());
    assert_eq!(
        populated_sense.result.registrations[0].sources[0].id,
        source
    );
}

#[test]
fn test_resolve_reference_rejects_invalid_catalogue_callers_in_both_formats() {
    let parent = fresh_root("caller-errors");
    let catalogue = parent.join("catalogue");
    let workbench = parent.join("workbench");
    let (dictionary, source, release, occurrence) = typed_dictionary("mi");
    install_dictionary(&catalogue, &dictionary);
    build(&workbench, &catalogue, false);

    let wrong_role_id = EntityId::new("reference-entry").unwrap();
    assert!(
        dictionary
            .document()
            .metadata
            .entries
            .iter()
            .any(|entry| entry.id == wrong_role_id)
    );
    assert!(
        dictionary
            .document()
            .metadata
            .assertions
            .iter()
            .all(|assertion| assertion.id != wrong_role_id)
    );

    let mut full = WorkbenchSearchEngine::open(&workbench).unwrap();
    let mut compact = WorkbenchSearchEngine::open(&workbench).unwrap();

    let wrong_local_role = RelationTarget::Local(LocalRef::Assertion(wrong_role_id));
    for error in [
        full.resolve_reference(&wrong_local_role).unwrap_err(),
        compact
            .resolve_reference_compact(&wrong_local_role)
            .unwrap_err(),
    ] {
        assert!(matches!(
            error,
            WorkbenchQueryError::Query(source)
                if matches!(*source, DictionaryQueryError::InvalidReference { ref detail }
                    if detail.contains("specific role table"))
        ));
    }

    let unknown_release = RelationTarget::Catalog(CatalogTarget {
        source: source.clone(),
        release: EntityId::new("missing-release").unwrap(),
        native_entry_id: Some("native-entry-0".into()),
        native_assertion_id: Some("native-assertion-0".into()),
        occurrence: Some(occurrence.clone()),
    });
    for error in [
        full.resolve_reference(&unknown_release).unwrap_err(),
        compact
            .resolve_reference_compact(&unknown_release)
            .unwrap_err(),
    ] {
        assert!(matches!(
            error,
            WorkbenchQueryError::Query(source)
                if matches!(*source, DictionaryQueryError::InvalidReference { ref detail }
                    if detail.contains("unknown"))
        ));
    }

    let wrong_source_release = RelationTarget::Catalog(CatalogTarget {
        source: SourceId::new("other-known-release-source").unwrap(),
        release: release.clone(),
        native_entry_id: Some("native-entry-0".into()),
        native_assertion_id: Some("native-assertion-0".into()),
        occurrence: Some(occurrence.clone()),
    });
    for error in [
        full.resolve_reference(&wrong_source_release).unwrap_err(),
        compact
            .resolve_reference_compact(&wrong_source_release)
            .unwrap_err(),
    ] {
        assert!(matches!(
            error,
            WorkbenchQueryError::Query(source)
                if matches!(*source, DictionaryQueryError::InvalidReference { ref detail }
                    if detail.contains("not owned by the requested source"))
        ));
    }

    let unknown_release_no_identity = RelationTarget::Catalog(CatalogTarget {
        source: source.clone(),
        release: EntityId::new("missing-release").unwrap(),
        native_entry_id: None,
        native_assertion_id: None,
        occurrence: None,
    });
    let full_error = full
        .resolve_reference(&unknown_release_no_identity)
        .unwrap_err();
    assert!(matches!(
        full_error,
        WorkbenchQueryError::Query(source)
            if matches!(*source, DictionaryQueryError::InvalidReference { ref detail }
                if detail.contains("unknown"))
    ));
    let compact_error = compact
        .resolve_reference_compact(&unknown_release_no_identity)
        .unwrap_err();
    assert!(matches!(
        compact_error,
        WorkbenchQueryError::Query(source)
            if matches!(*source, DictionaryQueryError::InvalidReference { ref detail }
                if detail.contains("no native identity or occurrence"))
    ));

    let empty_locator = RelationTarget::Catalog(CatalogTarget {
        source: source.clone(),
        release: release.clone(),
        native_entry_id: Some("native-entry-0".into()),
        native_assertion_id: Some("native-assertion-0".into()),
        occurrence: Some(OccurrenceRef {
            capture: occurrence.capture.clone(),
            locator: String::new(),
        }),
    });
    for error in [
        full.resolve_reference(&empty_locator).unwrap_err(),
        compact
            .resolve_reference_compact(&empty_locator)
            .unwrap_err(),
    ] {
        assert!(matches!(
            error,
            WorkbenchQueryError::Query(source)
                if matches!(*source, DictionaryQueryError::InvalidReference { ref detail }
                    if detail.contains("locator is empty"))
        ));
    }

    let absent_locator = RelationTarget::Catalog(CatalogTarget {
        source: source.clone(),
        release: release.clone(),
        native_entry_id: Some("native-entry-0".into()),
        native_assertion_id: Some("native-assertion-0".into()),
        occurrence: Some(OccurrenceRef {
            capture: occurrence.capture.clone(),
            locator: "/404".into(),
        }),
    });
    for error in [
        full.resolve_reference(&absent_locator).unwrap_err(),
        compact
            .resolve_reference_compact(&absent_locator)
            .unwrap_err(),
    ] {
        assert!(matches!(
            error,
            WorkbenchQueryError::Query(source)
                if matches!(*source, DictionaryQueryError::InvalidReference { ref detail }
                    if detail.contains("occurrence is not present in the requested release"))
        ));
    }

    let other_capture = RelationTarget::Catalog(CatalogTarget {
        source: source.clone(),
        release: release.clone(),
        native_entry_id: Some("native-entry-0".into()),
        native_assertion_id: Some("native-assertion-0".into()),
        occurrence: Some(OccurrenceRef {
            capture: CaptureRef {
                namespace: occurrence.capture.namespace.clone(),
                sha256: "a".repeat(64),
            },
            locator: occurrence.locator.clone(),
        }),
    });
    for error in [
        full.resolve_reference(&other_capture).unwrap_err(),
        compact
            .resolve_reference_compact(&other_capture)
            .unwrap_err(),
    ] {
        assert!(matches!(
            error,
            WorkbenchQueryError::Query(source)
                if matches!(*source, DictionaryQueryError::InvalidReference { ref detail }
                    if detail.contains("occurrence is not present in the requested release"))
        ));
    }

    let no_identity = RelationTarget::Catalog(CatalogTarget {
        source,
        release,
        native_entry_id: None,
        native_assertion_id: None,
        occurrence: None,
    });
    for error in [
        full.resolve_reference(&no_identity).unwrap_err(),
        compact.resolve_reference_compact(&no_identity).unwrap_err(),
    ] {
        assert!(matches!(
            error,
            WorkbenchQueryError::Query(source)
            if matches!(*source, DictionaryQueryError::InvalidReference { ref detail }
                    if detail.contains("no native identity or occurrence"))
        ));
    }

    let unresolved = RelationTarget::Unresolved("future editorial target".into());
    assert_eq!(
        full.resolve_reference(&unresolved).unwrap().result.status,
        ReferenceStatus::Unresolved
    );
    assert_eq!(
        compact
            .resolve_reference_compact(&unresolved)
            .unwrap()
            .result
            .status,
        ReferenceStatus::Unresolved
    );
    let empty_unresolved = RelationTarget::Unresolved(String::new());
    for error in [
        full.resolve_reference(&empty_unresolved).unwrap_err(),
        compact
            .resolve_reference_compact(&empty_unresolved)
            .unwrap_err(),
    ] {
        assert!(matches!(
            error,
            WorkbenchQueryError::Query(source)
                if matches!(*source, DictionaryQueryError::InvalidReference { ref detail }
                    if detail.contains("Unresolved target must be nonempty"))
        ));
    }
}

#[test]
fn test_resolve_reference_disabled_catalog_metadata_remains_catalogue_only() {
    let parent = fresh_root("disabled");
    let catalogue = parent.join("catalogue");
    let root = parent.join("workbench");
    let (dictionary, source, release, occurrence) = typed_dictionary("mi");
    let (alternate, _, _, _) = typed_dictionary_for("alternate", "do");
    install_dictionaries(
        &catalogue,
        &[
            (PackId::Llg, &dictionary),
            (PackId::LaLojban, &dictionary),
            (PackId::LensiskuJbotci, &alternate),
        ],
    );
    WorkbenchView::build_new(
        &root,
        WorkbenchBuildRequest {
            catalogue_root: Some(catalogue.clone()),
            owned_root: None,
            selection: WorkbenchSelection {
                version: 1,
                packs: vec![PackId::LaLojban, PackId::LensiskuJbotci],
                disabled: vec![PackId::LaLojban],
                owned: None,
                contexts: Vec::new(),
            },
        },
    )
    .unwrap();

    let mut engine = WorkbenchSearchEngine::open(&root).unwrap();
    let response = engine
        .resolve_reference(&target(source.clone(), release.clone(), occurrence.clone()))
        .unwrap();
    assert_eq!(response.result.status, ReferenceStatus::CatalogueOnly);
    assert!(response.result.payload.is_none());
    assert_eq!(response.result.catalogue_matches.len(), 2);
    assert_eq!(response.result.registrations.len(), 2);
    assert_eq!(response.result.registrations[0].pack_id, PackId::Llg);
    assert_eq!(response.result.registrations[1].pack_id, PackId::LaLojban);
    assert_eq!(
        response.result.operational_reason.as_deref(),
        Some("not_selected")
    );
    assert!(!response.result.registrations[0].selected);
    assert_eq!(
        response.result.registrations[0]
            .operational_reason
            .as_deref(),
        Some("not_selected")
    );
    assert!(response.result.registrations[1].selected);
    assert!(response.result.registrations[1].disabled);
    assert_eq!(
        response.result.registrations[1]
            .operational_reason
            .as_deref(),
        Some("disabled")
    );

    let disabled_no_identity = RelationTarget::Catalog(CatalogTarget {
        source: source.clone(),
        release: release.clone(),
        native_entry_id: None,
        native_assertion_id: None,
        occurrence: None,
    });
    let mut compact_engine = WorkbenchSearchEngine::open(&root).unwrap();
    for error in [
        engine.resolve_reference(&disabled_no_identity).unwrap_err(),
        compact_engine
            .resolve_reference_compact(&disabled_no_identity)
            .unwrap_err(),
    ] {
        assert!(matches!(
            error,
            WorkbenchQueryError::Query(source)
                if matches!(*source, DictionaryQueryError::InvalidReference { ref detail }
                    if detail.contains("no native identity or occurrence"))
        ));
    }

    let empty_sense_target = RelationTarget::Local(LocalRef::EditorialSense(
        EntityId::new("reference-empty-sense").unwrap(),
    ));
    let empty_sense = engine.resolve_reference(&empty_sense_target).unwrap();
    assert_eq!(empty_sense.result.status, ReferenceStatus::CatalogueOnly);
    assert!(empty_sense.result.payload.is_none());
    assert_eq!(empty_sense.result.registrations.len(), 2);
    assert!(
        empty_sense
            .result
            .registrations
            .iter()
            .all(|row| row.sources.is_empty())
    );
    assert_eq!(
        empty_sense.result.operational_reason.as_deref(),
        Some("not_selected")
    );

    drop(engine);
    let catalogue_revision = CollectionCatalogue::open(&catalogue)
        .unwrap()
        .local_revision()
        .unwrap();
    CollectionCatalogue::remove(&catalogue, &[PackId::Llg], catalogue_revision).unwrap();
    let removed_root = parent.join("removed-workbench");
    WorkbenchView::build_new(
        &removed_root,
        WorkbenchBuildRequest {
            catalogue_root: Some(catalogue.clone()),
            owned_root: None,
            selection: WorkbenchSelection {
                version: 1,
                packs: vec![PackId::LensiskuJbotci],
                disabled: Vec::new(),
                owned: None,
                contexts: Vec::new(),
            },
        },
    )
    .unwrap();
    let mut removed_engine = WorkbenchSearchEngine::open(&removed_root).unwrap();
    let removed = removed_engine
        .resolve_reference(&target(source, release, occurrence))
        .unwrap();
    let removed_registration = removed
        .result
        .registrations
        .iter()
        .find(|row| row.pack_id == PackId::Llg)
        .unwrap();
    assert!(!removed_registration.selected);
    assert_eq!(
        removed_registration.installation_state,
        vlajmaji_store::CollectionInstallationState::Removed
    );
    assert_eq!(
        removed_registration.operational_reason.as_deref(),
        Some("not_selected")
    );

    // Retain an old registration while installing a newer release for the
    // same pack. The old target remains a metadata match but cannot borrow the
    // newer installed release's admission.
    let old_root = parent.join("old-registration-catalogue");
    let old_workbench_root = parent.join("old-registration-workbench");
    let (old_dictionary, old_source, old_release, old_occurrence) = typed_dictionary("mi");
    let mut current_document = old_dictionary.document();
    current_document.metadata.releases[0].id = EntityId::new("mi-release-current").unwrap();
    let current_dictionary = Dictionary::from_document(current_document).unwrap();
    assert_eq!(old_source, SourceId::new("reference-upstream").unwrap());
    assert_ne!(old_release, current_dictionary.metadata().releases[0].id);
    install_dictionary(&old_root, &old_dictionary);
    let old_sealed =
        seal_release(PackId::Llg, PackPayloadKind::Synthetic, &old_dictionary).unwrap();
    let old_sha = format!("{:x}", Sha256::digest(&old_sealed.release));
    let current_sha = register_dictionary(&old_root, PackId::Llg, &current_dictionary);
    assert_ne!(old_sha, current_sha);
    let sealed_current =
        seal_release(PackId::Llg, PackPayloadKind::Synthetic, &current_dictionary).unwrap();
    let update = CollectionRequest::new(
        vec![CollectionInput::SealedBytes {
            pack_id: PackId::Llg,
            release: Cow::Borrowed(&sealed_current.release),
            seal: Cow::Borrowed(&sealed_current.seal),
        }],
        CollectionSelection::Packs(vec![PackId::Llg]),
        true,
    )
    .unwrap();
    let revision = CollectionCatalogue::open(&old_root)
        .unwrap()
        .local_revision()
        .unwrap();
    CollectionCatalogue::update(&old_root, update, revision).unwrap();
    WorkbenchView::build_new(
        &old_workbench_root,
        WorkbenchBuildRequest {
            catalogue_root: Some(old_root),
            owned_root: None,
            selection: WorkbenchSelection {
                version: 1,
                packs: vec![PackId::Llg],
                disabled: Vec::new(),
                owned: None,
                contexts: Vec::new(),
            },
        },
    )
    .unwrap();
    let mut old_engine = WorkbenchSearchEngine::open(&old_workbench_root).unwrap();
    let old_result = old_engine
        .resolve_reference(&target(old_source, old_release, old_occurrence))
        .unwrap();
    assert_eq!(old_result.result.status, ReferenceStatus::CatalogueOnly);
    assert!(old_result.result.payload.is_none());
    assert_eq!(old_result.result.catalogue_matches.len(), 1);
    let old_row = &old_result.result.registrations[0];
    assert_eq!(old_row.pack_id, PackId::Llg);
    assert!(old_row.selected);
    assert!(!old_row.admitted);
    assert_eq!(
        old_row.installed_release_sha256.as_deref(),
        Some(current_sha.as_str())
    );
    assert_eq!(
        old_row.operational_reason.as_deref(),
        Some("registered_only")
    );
}

#[test]
fn test_resolve_reference_native_metadata_wildcard_digest_integrity_and_restore() {
    let parent = fresh_root("native-metadata");
    let catalogue_root = parent.join("catalogue");
    let workbench_root = parent.join("workbench");
    let (dictionary, source, release, occurrence) = typed_dictionary("mi");
    let (alternate, _, _, _) = typed_dictionary_for("alternate", "do");

    let unknown_catalogue = parent.join("unknown-catalogue");
    let unknown_workbench = parent.join("unknown-workbench");
    let unknown_dictionary = dictionary_with_unknown_native_attribution(&dictionary);
    assert_eq!(
        unknown_dictionary.registry().as_document().occurrences[0].native_entry_id,
        None
    );
    install_dictionaries(
        &unknown_catalogue,
        &[
            (PackId::Llg, &unknown_dictionary),
            (PackId::LaLojban, &alternate),
        ],
    );
    build(&unknown_workbench, &unknown_catalogue, true);
    let requested = target(source.clone(), release.clone(), occurrence.clone());
    let mut unknown_full = WorkbenchSearchEngine::open(&unknown_workbench).unwrap();
    let unknown_full_result = unknown_full.resolve_reference(&requested).unwrap();
    assert_eq!(
        unknown_full_result.result.status,
        ReferenceStatus::Unverified
    );
    assert!(unknown_full_result.result.payload.is_none());
    let mut unknown_compact = WorkbenchSearchEngine::open(&unknown_workbench).unwrap();
    let unknown_compact_result = unknown_compact
        .resolve_reference_compact(&requested)
        .unwrap();
    assert_eq!(
        unknown_compact_result.result.status,
        ReferenceStatus::NoMatch
    );
    assert!(unknown_compact_result.result.catalogue_matches.is_empty());
    let occurrence_only = RelationTarget::Catalog(CatalogTarget {
        source: source.clone(),
        release: release.clone(),
        native_entry_id: None,
        native_assertion_id: None,
        occurrence: Some(occurrence.clone()),
    });
    let mut occurrence_full = WorkbenchSearchEngine::open(&unknown_workbench).unwrap();
    assert_eq!(
        occurrence_full
            .resolve_reference(&occurrence_only)
            .unwrap()
            .result
            .status,
        ReferenceStatus::CatalogueOnly
    );
    let mut occurrence_compact = WorkbenchSearchEngine::open(&unknown_workbench).unwrap();
    assert_eq!(
        occurrence_compact
            .resolve_reference_compact(&occurrence_only)
            .unwrap()
            .result
            .status,
        ReferenceStatus::CatalogueOnly
    );

    let (two_occurrence_dictionary, second_occurrence) =
        dictionary_with_second_occurrence(&dictionary);
    let mut mixed_document = two_occurrence_dictionary.document();
    let mut mixed_registry = mixed_document.registry.document();
    let second_attribution = mixed_registry
        .occurrences
        .iter_mut()
        .find(|row| row.occurrence.locator() == second_occurrence.locator)
        .unwrap();
    second_attribution.native_entry_id = None;
    second_attribution.native_assertion_id = None;
    mixed_document.registry = SourceRegistry::from_document(mixed_registry).unwrap();
    let mixed_dictionary = Dictionary::from_document(mixed_document).unwrap();
    assert_eq!(mixed_dictionary.metadata().releases[0].assertions.len(), 2);
    let mixed_catalogue = parent.join("mixed-catalogue");
    let mixed_workbench = parent.join("mixed-workbench");
    install_dictionary(&mixed_catalogue, &mixed_dictionary);
    build(&mixed_workbench, &mixed_catalogue, false);
    let mixed_target = RelationTarget::Catalog(CatalogTarget {
        source: source.clone(),
        release: release.clone(),
        native_entry_id: Some("native-entry-0".into()),
        native_assertion_id: Some("native-assertion-0".into()),
        occurrence: None,
    });
    let mut mixed_full = WorkbenchSearchEngine::open(&mixed_workbench).unwrap();
    let mixed_full_result = mixed_full.resolve_reference(&mixed_target).unwrap();
    assert_eq!(mixed_full_result.result.status, ReferenceStatus::Resolved);
    let full_payload = mixed_full_result.result.payload.unwrap();
    assert_eq!(full_payload.confirmed.len(), 1);
    assert_eq!(full_payload.unverified.len(), 1);
    let mut mixed_compact = WorkbenchSearchEngine::open(&mixed_workbench).unwrap();
    let mixed_compact_result = mixed_compact
        .resolve_reference_compact(&mixed_target)
        .unwrap();
    assert_eq!(
        mixed_compact_result.result.status,
        ReferenceStatus::Unverified
    );
    let compact_payload = mixed_compact_result.result.payload.unwrap();
    assert_eq!(compact_payload.confirmed.len(), 1);
    assert_eq!(compact_payload.unverified.len(), 1);

    install_dictionaries(
        &catalogue_root,
        &[(PackId::Llg, &dictionary), (PackId::LaLojban, &alternate)],
    );
    build(&workbench_root, &catalogue_root, true);

    let sealed = seal_release(PackId::Llg, PackPayloadKind::Synthetic, &dictionary).unwrap();
    let release_sha256 = format!("{:x}", Sha256::digest(&sealed.release));
    let database = catalogue_root.join("catalogue.sqlite3");
    let registration = format!("pack_id='llg' AND release_sha256='{release_sha256}'");
    let original_hex = sqlite_scalar(
        &database,
        &format!("SELECT hex(descriptor_json) FROM registrations_v2 WHERE {registration};"),
    );
    let original_bytes = decode_hex(&original_hex);
    let original_json: serde_json::Value = serde_json::from_slice(&original_bytes).unwrap();
    let mut unverified_json = original_json.clone();
    let occurrence_rows = unverified_json["registry"]["occurrences"]
        .as_array_mut()
        .unwrap();
    let attribution = occurrence_rows
        .iter_mut()
        .find(|row| {
            row["occurrence"]["snapshot"]["namespace"] == occurrence.capture.namespace
                && row["occurrence"]["snapshot"]["sha256"] == occurrence.capture.sha256
                && row["occurrence"]["locator"] == occurrence.locator
        })
        .expect("input dictionary has the requested occurrence attribution");
    attribution["native_entry_id"] = serde_json::Value::Null;
    attribution["native_assertion_id"] = serde_json::Value::Null;
    let unverified_bytes = serde_json::to_vec(&unverified_json).unwrap();
    sqlite_exec(
        &database,
        &format!(
            "UPDATE registrations_v2 SET descriptor_json={} WHERE {registration};",
            sqlite_blob(&unverified_bytes)
        ),
    );

    let mut full = WorkbenchSearchEngine::open(&workbench_root).unwrap();
    let full_unverified = full.resolve_reference(&requested).unwrap();
    assert_eq!(full_unverified.result.status, ReferenceStatus::Unverified);
    assert!(full_unverified.result.payload.is_none());
    assert_eq!(full_unverified.result.catalogue_matches.len(), 1);
    let mut compact = WorkbenchSearchEngine::open(&workbench_root).unwrap();
    let compact_strict = compact.resolve_reference_compact(&requested).unwrap();
    assert_eq!(compact_strict.result.status, ReferenceStatus::NoMatch);
    assert!(compact_strict.result.catalogue_matches.is_empty());

    sqlite_exec(
        &database,
        &format!(
            "UPDATE registrations_v2 SET descriptor_json={} WHERE {registration};",
            sqlite_blob(&original_bytes)
        ),
    );
    let restored_hex = sqlite_scalar(
        &database,
        &format!("SELECT hex(descriptor_json) FROM registrations_v2 WHERE {registration};"),
    );
    assert_eq!(restored_hex, original_hex);

    let mut multiple_attribution_json = original_json.clone();
    let occurrence_rows = multiple_attribution_json["registry"]["occurrences"]
        .as_array_mut()
        .unwrap();
    let attribution_index = occurrence_rows
        .iter()
        .position(|row| {
            row["occurrence"]["snapshot"]["namespace"] == occurrence.capture.namespace
                && row["occurrence"]["snapshot"]["sha256"] == occurrence.capture.sha256
                && row["occurrence"]["locator"] == occurrence.locator
        })
        .expect("valid baseline has one target attribution");
    occurrence_rows[attribution_index]["native_entry_id"] = serde_json::Value::Null;
    occurrence_rows[attribution_index]["native_assertion_id"] = serde_json::Value::Null;
    let mut later_attribution = occurrence_rows[attribution_index].clone();
    later_attribution["native_entry_id"] = serde_json::json!("native-entry-0");
    later_attribution["native_assertion_id"] = serde_json::json!("native-assertion-0");
    occurrence_rows.insert(attribution_index + 1, later_attribution);
    let multiple_attribution_bytes = serde_json::to_vec(&multiple_attribution_json).unwrap();
    sqlite_exec(
        &database,
        &format!(
            "UPDATE registrations_v2 SET descriptor_json={} WHERE {registration};",
            sqlite_blob(&multiple_attribution_bytes)
        ),
    );
    let mut full = WorkbenchSearchEngine::open(&workbench_root).unwrap();
    let first_attribution = full.resolve_reference(&requested).unwrap();
    assert_eq!(first_attribution.result.status, ReferenceStatus::Unverified);
    assert!(first_attribution.result.payload.is_none());
    let mut compact = WorkbenchSearchEngine::open(&workbench_root).unwrap();
    let raw_any_then_strict = compact.resolve_reference_compact(&requested).unwrap();
    assert_eq!(raw_any_then_strict.result.status, ReferenceStatus::NoMatch);
    assert!(raw_any_then_strict.result.catalogue_matches.is_empty());
    sqlite_exec(
        &database,
        &format!(
            "UPDATE registrations_v2 SET descriptor_json={} WHERE {registration};",
            sqlite_blob(&original_bytes)
        ),
    );
    let restored_hex = sqlite_scalar(
        &database,
        &format!("SELECT hex(descriptor_json) FROM registrations_v2 WHERE {registration};"),
    );
    assert_eq!(restored_hex, original_hex);

    let wrong_native = match requested.clone() {
        RelationTarget::Catalog(mut catalog) => {
            catalog.native_entry_id = Some("native-entry-wrong".into());
            RelationTarget::Catalog(catalog)
        }
        _ => unreachable!(),
    };
    let mut full = WorkbenchSearchEngine::open(&workbench_root).unwrap();
    let wrong_full = full.resolve_reference(&wrong_native).unwrap();
    assert_eq!(wrong_full.result.status, ReferenceStatus::NoMatch);
    assert!(wrong_full.result.catalogue_matches.is_empty());
    let mut compact = WorkbenchSearchEngine::open(&workbench_root).unwrap();
    let wrong_compact = compact.resolve_reference_compact(&wrong_native).unwrap();
    assert_eq!(wrong_compact.result.status, ReferenceStatus::NoMatch);
    assert!(wrong_compact.result.catalogue_matches.is_empty());

    let assertion_id = dictionary.metadata().releases[0].assertions[0].to_string();
    let mut missing_digest_json = original_json;
    let removed = missing_digest_json["body_sha256"]
        .as_object_mut()
        .unwrap()
        .remove(&format!("assertion:{assertion_id}"));
    assert!(removed.is_some());
    let missing_digest_bytes = serde_json::to_vec(&missing_digest_json).unwrap();
    sqlite_exec(
        &database,
        &format!(
            "UPDATE registrations_v2 SET descriptor_json={} WHERE {registration};",
            sqlite_blob(&missing_digest_bytes)
        ),
    );
    let mut full = WorkbenchSearchEngine::open(&workbench_root).unwrap();
    assert!(matches!(
        full.resolve_reference(&requested),
        Err(WorkbenchQueryError::Query(source))
            if matches!(*source, DictionaryQueryError::Integrity { ref detail }
                if detail.contains("has no body digest"))
    ));
    let mut compact = WorkbenchSearchEngine::open(&workbench_root).unwrap();
    assert!(matches!(
        compact.resolve_reference_compact(&requested),
        Err(WorkbenchQueryError::Query(source))
            if matches!(*source, DictionaryQueryError::Integrity { ref detail }
                if detail.contains("has no body digest"))
    ));
    sqlite_exec(
        &database,
        &format!(
            "UPDATE registrations_v2 SET descriptor_json={} WHERE {registration};",
            sqlite_blob(&original_bytes)
        ),
    );
    let final_hex = sqlite_scalar(
        &database,
        &format!("SELECT hex(descriptor_json) FROM registrations_v2 WHERE {registration};"),
    );
    assert_eq!(final_hex, original_hex);

    let mut malformed_digest_json: serde_json::Value =
        serde_json::from_slice(&original_bytes).unwrap();
    malformed_digest_json["body_sha256"][format!("assertion:{assertion_id}")] =
        serde_json::Value::String("A".repeat(64));
    let malformed_digest_bytes = serde_json::to_vec(&malformed_digest_json).unwrap();
    sqlite_exec(
        &database,
        &format!(
            "UPDATE registrations_v2 SET descriptor_json={} WHERE {registration};",
            sqlite_blob(&malformed_digest_bytes)
        ),
    );
    let mut full = WorkbenchSearchEngine::open(&workbench_root).unwrap();
    assert!(matches!(
        full.resolve_reference(&requested),
        Err(WorkbenchQueryError::Query(source))
            if matches!(*source, DictionaryQueryError::Integrity { ref detail }
                if detail.contains("has a malformed body digest"))
    ));
    let mut compact = WorkbenchSearchEngine::open(&workbench_root).unwrap();
    assert!(matches!(
        compact.resolve_reference_compact(&requested),
        Err(WorkbenchQueryError::Query(source))
            if matches!(*source, DictionaryQueryError::Integrity { ref detail }
                if detail.contains("has a malformed body digest"))
    ));
    sqlite_exec(
        &database,
        &format!(
            "UPDATE registrations_v2 SET descriptor_json={} WHERE {registration};",
            sqlite_blob(&original_bytes)
        ),
    );
    let restored_after_malformed = sqlite_scalar(
        &database,
        &format!("SELECT hex(descriptor_json) FROM registrations_v2 WHERE {registration};"),
    );
    assert_eq!(restored_after_malformed, original_hex);

    let mut full = WorkbenchSearchEngine::open(&workbench_root).unwrap();
    assert_eq!(
        full.resolve_reference(&requested).unwrap().result.status,
        ReferenceStatus::CatalogueOnly
    );
    let mut compact = WorkbenchSearchEngine::open(&workbench_root).unwrap();
    assert_eq!(
        compact
            .resolve_reference_compact(&requested)
            .unwrap()
            .result
            .status,
        ReferenceStatus::CatalogueOnly
    );
}

#[test]
fn test_resolve_reference_owned_release_is_known_outside_bound_catalogue() {
    let parent = fresh_root("owned-release");
    let catalogue = parent.join("catalogue");
    let workbench = parent.join("workbench");
    let journal = parent.join("journal");
    let (owned_dictionary, source, release, occurrence) = typed_dictionary("mi");
    let (catalogue_dictionary, _, _, _) = typed_dictionary_for("alternate", "do");
    install_dictionaries(&catalogue, &[(PackId::LaLojban, &catalogue_dictionary)]);
    OwnedJournal::create(&journal, &owned_dictionary).unwrap();
    WorkbenchView::build_new(
        &workbench,
        WorkbenchBuildRequest {
            catalogue_root: Some(catalogue),
            owned_root: Some(journal),
            selection: WorkbenchSelection {
                version: 1,
                packs: vec![PackId::LaLojban],
                disabled: Vec::new(),
                owned: Some(WorkbenchOwnedRevision::Head),
                contexts: Vec::new(),
            },
        },
    )
    .unwrap();

    let target = target(source, release.clone(), occurrence);
    let mut full = WorkbenchSearchEngine::open(&workbench).unwrap();
    let full_result = full.resolve_reference(&target).unwrap();
    assert_eq!(full_result.result.status, ReferenceStatus::Resolved);
    assert!(full_result.result.payload.is_some());
    assert!(full_result.result.catalogue_matches.is_empty());
    assert!(full_result.result.registrations.is_empty());
    assert_eq!(
        full_result
            .result
            .requested_release
            .as_ref()
            .map(|value| value.id.as_str()),
        Some(release.as_str())
    );

    let mut compact = WorkbenchSearchEngine::open(&workbench).unwrap();
    let compact_result = compact.resolve_reference_compact(&target).unwrap();
    assert_eq!(compact_result.result.status, ReferenceStatus::Resolved);
    assert!(compact_result.result.payload.is_some());
    assert!(compact_result.result.catalogue_matches.is_empty());
    assert!(compact_result.result.registrations.is_empty());
    assert_eq!(
        compact_result
            .result
            .requested_release
            .as_ref()
            .map(|value| value.id.as_str()),
        Some(release.as_str())
    );
}

#[test]
fn test_resolve_reference_equal_retained_rows_preserve_raw_order_and_installed_representative() {
    let parent = fresh_root("equal-retained");
    let catalogue = parent.join("catalogue");
    let workbench = parent.join("workbench");
    let journal = parent.join("journal");
    let (genesis, dictionary) = rich_owned_history();
    let declared_sense = &dictionary.metadata().editorial_senses[1];
    assert_eq!(
        declared_sense.assertions,
        vec![
            EntityId::new("mi-assertion-b").unwrap(),
            EntityId::new("mi-assertion").unwrap()
        ]
    );
    assert_eq!(
        dictionary.metadata().entries[0].spelling,
        Presence::Value("mi".into())
    );
    assert_eq!(
        dictionary.metadata().entries[1].spelling,
        Presence::Value("zz".into())
    );
    // Full confirmations sort these same-source members by input spelling;
    // Compact confirmations retain the declared Sense member order.
    assert_ne!(
        declared_sense.assertions,
        vec![
            EntityId::new("mi-assertion").unwrap(),
            EntityId::new("mi-assertion-b").unwrap()
        ]
    );
    let (selected_other, _, _, _) = typed_dictionary_for("alternate", "do");
    install_dictionaries(
        &catalogue,
        &[
            (PackId::Llg, &dictionary),
            (PackId::LaLojban, &dictionary),
            (PackId::LensiskuJbotci, &selected_other),
        ],
    );
    create_then_commit_rich_journal(&journal, &genesis, &dictionary);
    WorkbenchView::build_new(
        &workbench,
        WorkbenchBuildRequest {
            catalogue_root: Some(catalogue),
            owned_root: Some(journal),
            selection: WorkbenchSelection {
                version: 1,
                packs: vec![PackId::Llg, PackId::LaLojban],
                disabled: vec![PackId::Llg, PackId::LaLojban],
                owned: Some(WorkbenchOwnedRevision::Head),
                contexts: Vec::new(),
            },
        },
    )
    .unwrap();

    let source = SourceId::new("mi-upstream").unwrap();
    let release = EntityId::new("mi-release-r1").unwrap();
    let occurrence = OccurrenceRef {
        capture: dictionary.metadata().releases[1].captures[0].clone(),
        locator: "/0".into(),
    };
    let catalog_target = target(source, release, occurrence);
    let mut engine = WorkbenchSearchEngine::open(&workbench).unwrap();
    let catalog = engine.resolve_reference(&catalog_target).unwrap();
    assert_eq!(catalog.result.status, ReferenceStatus::Resolved);
    assert!(catalog.result.payload.is_some());
    assert_eq!(catalog.result.catalogue_matches.len(), 2);
    assert_eq!(catalog.result.catalogue_matches[0].pack_id, "llg");
    assert_eq!(catalog.result.catalogue_matches[1].pack_id, "la-lojban");
    assert_eq!(catalog.result.operational_reason, None);

    let sense_id = EntityId::new("mi-populated-sense").unwrap();
    let local_target = RelationTarget::Local(LocalRef::EditorialSense(sense_id.clone()));
    let local = engine.resolve_reference(&local_target).unwrap();
    assert_eq!(local.result.status, ReferenceStatus::Resolved);
    assert!(local.result.payload.is_some());
    assert_eq!(local.result.catalogue_matches.len(), 1);
    assert_eq!(local.result.catalogue_matches[0].pack_id, "la-lojban");
    assert_eq!(
        local.result.catalogue_matches[0].contributing_pack_ids,
        vec!["la-lojban", "llg"]
    );
    assert_eq!(local.result.registrations.len(), 2);
    assert!(local.result.registrations.iter().all(|row| row.disabled));
    assert!(local.result.registrations.iter().all(|row| !row.admitted));
    let full_payload = local
        .result
        .payload
        .as_ref()
        .expect("owned provider resolves the Full local sense");
    assert_eq!(
        full_payload.entity,
        Some(ReferenceEntity::EditorialSense(
            dictionary.metadata().editorial_senses[1].clone()
        ))
    );
    let full_member_ids: Vec<_> = full_payload
        .confirmed
        .iter()
        .map(|row| row.assertion.id.as_str())
        .collect();
    assert_eq!(full_member_ids, vec!["mi-assertion", "mi-assertion-b"]);

    let mut compact_engine = WorkbenchSearchEngine::open(&workbench).unwrap();
    let compact = compact_engine
        .resolve_reference_compact(&local_target)
        .unwrap();
    assert_eq!(compact.result.status, ReferenceStatus::Resolved);
    assert!(compact.result.payload.is_some());
    assert_eq!(compact.result.catalogue_matches.len(), 1);
    assert_eq!(compact.result.catalogue_matches[0].pack_id, "la-lojban");
    assert_eq!(
        compact.result.catalogue_matches[0].contributing_pack_ids,
        vec!["la-lojban", "llg"]
    );
    assert_eq!(compact.result.registrations.len(), 2);
    assert!(compact.result.registrations.iter().all(|row| row.disabled));
    let compact_payload = compact
        .result
        .payload
        .as_ref()
        .expect("owned provider resolves the local sense");
    let compact_member_ids: Vec<_> = compact_payload
        .confirmed
        .iter()
        .map(|row| row.assertion.id.as_str())
        .collect();
    assert_eq!(compact_member_ids, vec!["mi-assertion-b", "mi-assertion"]);
    assert_eq!(
        dictionary.metadata().editorial_senses[1].assertions,
        vec![
            EntityId::new("mi-assertion-b").unwrap(),
            EntityId::new("mi-assertion").unwrap()
        ]
    );
}

#[test]
fn test_resolve_reference_unequal_retained_role_bodies_are_ambiguous() {
    let (genesis, baseline) = rich_owned_history();
    let changed_form = dictionary_with_form_spelling(&baseline, "mi-form-alternate");
    let changed_sense = dictionary_with_sense_rationale(&baseline, "different typed rationale");
    let original_form = &baseline.metadata().forms[0];
    let changed_form_row = &changed_form.metadata().forms[0];
    assert_eq!(original_form.id, changed_form_row.id);
    assert_eq!(original_form.entry, changed_form_row.entry);
    assert_eq!(original_form.evidence, changed_form_row.evidence);
    assert_eq!(
        form_descriptor(original_form, &SourceId::new("mi-upstream").unwrap()),
        form_descriptor(changed_form_row, &SourceId::new("mi-upstream").unwrap())
    );
    assert_ne!(original_form.spelling, changed_form_row.spelling);
    assert_ne!(
        typed_row_sha(original_form),
        typed_row_sha(changed_form_row)
    );
    let original_sense = &baseline.metadata().editorial_senses[1];
    let changed_sense_row = &changed_sense.metadata().editorial_senses[1];
    assert_eq!(original_sense.id, changed_sense_row.id);
    assert_eq!(original_sense.assertions, changed_sense_row.assertions);
    assert_eq!(original_sense.revision, changed_sense_row.revision);
    assert_eq!(
        sense_descriptor(&baseline, original_sense),
        sense_descriptor(&changed_sense, changed_sense_row)
    );
    assert_ne!(original_sense.rationale, changed_sense_row.rationale);
    assert_ne!(
        typed_row_sha(original_sense),
        typed_row_sha(changed_sense_row)
    );

    for (label, changed, local_target, role, id) in [
        (
            "form-only",
            &changed_form,
            RelationTarget::Local(LocalRef::Form(EntityId::new("mi-form").unwrap())),
            "form",
            "mi-form",
        ),
        (
            "sense-only",
            &changed_sense,
            RelationTarget::Local(LocalRef::EditorialSense(
                EntityId::new("mi-populated-sense").unwrap(),
            )),
            "editorial_sense",
            "mi-populated-sense",
        ),
    ] {
        let parent = fresh_root(label);
        let catalogue = parent.join("catalogue");
        let workbench = parent.join("workbench");
        let journal = parent.join("journal");
        let (unrelated, _, _, _) = typed_dictionary_for("alternate", "do");
        install_dictionary(&catalogue, &unrelated);
        register_dictionary(&catalogue, PackId::LaLojban, changed);
        create_then_commit_rich_journal(&journal, &genesis, &baseline);
        WorkbenchView::build_new(
            &workbench,
            WorkbenchBuildRequest {
                catalogue_root: Some(catalogue),
                owned_root: Some(journal),
                selection: WorkbenchSelection {
                    version: 1,
                    packs: vec![PackId::Llg, PackId::LaLojban],
                    disabled: vec![PackId::Llg, PackId::LaLojban],
                    owned: Some(WorkbenchOwnedRevision::Head),
                    contexts: Vec::new(),
                },
            },
        )
        .unwrap();

        let mut full = WorkbenchSearchEngine::open(&workbench).unwrap();
        assert!(matches!(
            full.resolve_reference(&local_target),
            Err(WorkbenchQueryError::AmbiguousReference { role: actual_role, id: actual_id })
                if actual_role == role && actual_id == id
        ));
        let mut compact = WorkbenchSearchEngine::open(&workbench).unwrap();
        assert!(matches!(
            compact.resolve_reference_compact(&local_target),
            Err(WorkbenchQueryError::AmbiguousReference { role: actual_role, id: actual_id })
                if actual_role == role && actual_id == id
        ));
    }
}
