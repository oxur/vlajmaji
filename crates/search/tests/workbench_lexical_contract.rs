//! Public checks for the lexical workbench over a small source-derived model.

use std::{
    borrow::Cow,
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

use sha2::{Digest, Sha256};
use vlajmaji_lexicon::{
    AbbreviationKind, AliasStatus, AssertionRecord, Availability, CaptureMetadata, CaptureRef,
    CollectionRecord, ComponentRecord, ComputationRecord, ContentNotice, ContributorId,
    ContributorRecord, Dictionary, DictionaryDocument, DictionaryMetadata, Eligibility, EntityId,
    EntryRecord, FormKind, FormRecord, FormRole, InputRef, LlgImportOptions, OccurrenceAttribution,
    OccurrenceRef, OwnedHead, OwnedRevisionRecord, PackId, PackPayloadKind, Presence, RawCapture,
    RawOccurrence, RedistributionStatus, RegistryDocument, RegistryError, ReleaseAccounting,
    ReleaseRecord, SnapshotAttribution, SourceAlias, SourceEvidence, SourceId, SourceOrigin,
    SourceRecord, SourceRegistry, TextValue, TimePrecision, import_llg_lists, seal_release,
};
use vlajmaji_search::{
    DictionaryQueryError, SourceSelectionFailure, WorkbenchFormRecord,
    WorkbenchLexicalCompactResult, WorkbenchLexicalError, WorkbenchLexicalFullResult,
    WorkbenchLexicalOptions, WorkbenchLexicalQuery, WorkbenchLexicalRequest, WorkbenchQueryError,
    WorkbenchSearchEngine,
};
use vlajmaji_store::{
    CollectionCatalogue, CollectionInput, CollectionRequest, CollectionSelection, OwnedJournal,
    WorkbenchBuildRequest, WorkbenchOwnedRevision, WorkbenchSelection, WorkbenchView,
};

static NEXT_ROOT: AtomicUsize = AtomicUsize::new(0);

fn fresh_root() -> PathBuf {
    for _ in 0..64 {
        let root = std::env::temp_dir().join(format!(
            "vlajmaji-workbench-lexical-{}-{}",
            std::process::id(),
            NEXT_ROOT.fetch_add(1, Ordering::Relaxed),
        ));
        match fs::create_dir(&root) {
            Ok(()) => return root,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => panic!("create exclusive lexical-test root: {error}"),
        }
    }
    panic!("could not allocate a lexical-test root after 64 attempts")
}

fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn gismu_rows() -> Vec<u8> {
    let row =
        |word: &str, rafsi_one: &str, rafsi_two: &str, rafsi_three: &str, definition: &str| {
            let mut row = vec![b' '; 158];
            for (start, end, value) in [
                (1, 6, word),
                (7, 10, rafsi_one),
                (11, 14, rafsi_two),
                (15, 19, rafsi_three),
                (20, 40, "grow"),
                (41, 61, "growth"),
                (62, 158, definition),
            ] {
                assert!(start + value.len() <= end);
                row[start..start + value.len()].copy_from_slice(value.as_bytes());
            }
            row
        };
    let mut rows = row("farvi", "far", "fav", "farv", "to grow");
    rows.push(b'\n');
    rows.extend(row("farvi", "bla", "bau", "blan", "to grow tall"));
    rows
}

fn source_dictionary_with_form() -> Dictionary {
    let gismu = gismu_rows();
    let empty = b"";
    let notice = b"synthetic lexical-query fixture\n";
    let imported = import_llg_lists(
        &gismu,
        empty,
        empty,
        notice,
        &LlgImportOptions {
            expected_gismu_sha256: digest(&gismu),
            expected_cmavo_sha256: digest(empty),
            expected_rafsi_sha256: digest(empty),
            expected_notice_sha256: digest(notice),
            base_locator: "https://example.invalid/lexical-test".into(),
        },
    )
    .expect("the fixed-column source row should import");

    let mut document = imported.into_dictionary().into_document();
    let assertion = document.metadata.assertions[0].clone();
    document.metadata.forms.push(FormRecord {
        id: EntityId::new("form-farvi").unwrap(),
        entry: assertion.entry,
        spelling: "farvi".into(),
        language: Presence::Value("jbo".into()),
        script: Presence::Missing,
        pronunciation: Presence::Missing,
        kind: FormKind::SourceVariant,
        standing: Presence::Missing,
        components: Vec::new(),
        evidence: Some(SourceEvidence {
            occurrence: assertion.occurrence,
            field_index: None,
            citation: Presence::Missing,
            offset: None,
        }),
        computation: None,
    });
    Dictionary::from_document(document).expect("the typed Form is source-backed")
}

fn source_dictionary_with_four_forms() -> Dictionary {
    let source = SourceId::new("lexical-upstream").unwrap();
    let contributor = ContributorId::new("lexical-upstream-editor").unwrap();
    let collection = EntityId::new("lexical-upstream-collection").unwrap();
    let component = EntityId::new("lexical-upstream-component").unwrap();
    let entry = EntityId::new("lexical-upstream-entry").unwrap();
    let capture = RawCapture::new(
        "captured:lexical-upstream:r1",
        CaptureMetadata {
            source_name: "Lexical upstream fixture".into(),
            format_label: "text/plain".into(),
            locator: "upstream r1".into(),
            revision: Presence::Missing,
            captured_at: Presence::Missing,
            time_precision: TimePrecision::Unknown,
            notice_locator: Presence::Missing,
        },
        b"GROW <b>literal</b>\ngrow\n".to_vec(),
        vec![
            RawOccurrence {
                locator: "/0".into(),
                fields: Vec::new(),
                raw_line: Some("GROW <b>literal</b>".into()),
            },
            RawOccurrence {
                locator: "/1".into(),
                fields: Vec::new(),
                raw_line: Some("grow".into()),
            },
        ],
    );
    let capture_ref = capture.identity.clone();
    let occurrences = ["/0", "/1"].map(|locator| OccurrenceRef {
        capture: capture_ref.clone(),
        locator: locator.into(),
    });
    let old_capture = CaptureRef {
        namespace: "captured:lexical-upstream-old".into(),
        sha256: "11".repeat(32),
    };
    let registry = SourceRegistry::from_document(RegistryDocument {
        sources: vec![SourceRecord {
            id: source.clone(),
            display_name: "Lexical upstream fixture".into(),
            abbreviation: "LU".into(),
            abbreviation_kind: AbbreviationKind::ProjectAssigned,
            abbreviation_evidence: Some("S04 lexical fixture".into()),
            origin: SourceOrigin::Upstream,
        }],
        contributors: vec![ContributorRecord {
            id: contributor.clone(),
            display_name: "Lexical fixture editor".into(),
        }],
        snapshots: vec![SnapshotAttribution {
            snapshot: capture_ref.to_snapshot_identity().unwrap(),
            source: source.clone(),
            distributor: None,
        }],
        occurrences: occurrences
            .iter()
            .enumerate()
            .map(|(index, occurrence)| OccurrenceAttribution {
                occurrence: occurrence.to_record_identity().unwrap(),
                contributor: Some(contributor.clone()),
                native_entry_id: Some("lexical-upstream-entry".into()),
                native_assertion_id: Some(format!(
                    "lexical-upstream-{}",
                    if index == 0 { "a" } else { "b" }
                )),
            })
            .collect(),
        aliases: vec![SourceAlias {
            source: source.clone(),
            text: "lexical-shared-legacy".into(),
            status: AliasStatus::Active,
            evidence: Some("S04 alias-selection fixture".into()),
        }],
        ..RegistryDocument::default()
    })
    .unwrap();

    let mut metadata = DictionaryMetadata::default();
    metadata.collections.push(CollectionRecord {
        id: collection.clone(),
        source: source.clone(),
        display_name: "Lexical upstream collection".into(),
        description: Presence::Missing,
    });
    metadata.components.push(ComponentRecord {
        id: component.clone(),
        collection,
        native_name: "farvi".into(),
        languages: vec!["jbo".into()],
    });
    let count = 2;
    let release = |id: &str,
                   captures: Vec<CaptureRef>,
                   assertions: Vec<EntityId>,
                   occurrence_count: u64,
                   availability| {
        ReleaseRecord {
            id: EntityId::new(id).unwrap(),
            component: component.clone(),
            distributor: None,
            upstream_revision: Presence::Missing,
            distributor_revision: Presence::Missing,
            captured_at: Presence::Missing,
            time_precision: TimePrecision::Unknown,
            schema_version: 1,
            normalizer: "lexical-contract-fixture".into(),
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
            availability,
            content_notices: vec![ContentNotice {
                text: Presence::Value("synthetic lexical contract fixture".into()),
                locator: Presence::Missing,
                license: Presence::Missing,
            }],
            redistribution: RedistributionStatus::Held,
            redistribution_evidence: Vec::new(),
        }
    };
    let assertion_ids = [
        EntityId::new("lexical-upstream-a").unwrap(),
        EntityId::new("lexical-upstream-b").unwrap(),
    ];
    metadata.releases.push(release(
        "lexical-upstream-release-r1",
        vec![capture_ref.clone()],
        assertion_ids.to_vec(),
        count,
        Availability::Available,
    ));
    metadata.releases.push(release(
        "lexical-upstream-old-release",
        vec![old_capture.clone()],
        Vec::new(),
        0,
        Availability::Missing,
    ));
    metadata.entries.push(EntryRecord {
        id: entry.clone(),
        source: source.clone(),
        spelling: Presence::Value("farvi".into()),
        language: Presence::Value("jbo".into()),
        script: Presence::Missing,
        native_word_class: Presence::Missing,
        selmaho: Presence::Missing,
        mapped_pos: Presence::Missing,
    });
    metadata.assertions = vec![
        AssertionRecord {
            id: assertion_ids[0].clone(),
            entry: entry.clone(),
            occurrence: occurrences[0].clone(),
            native_sense_id: Presence::Missing,
            language: Presence::Value("en".into()),
            definition: Presence::Value(TextValue {
                text: "GROW <b>literal</b>".into(),
                language: Presence::Missing,
                markup: Presence::Missing,
            }),
            source_score: Presence::Missing,
            timestamps: Vec::new(),
            eligibility: Eligibility::Lexical,
        },
        AssertionRecord {
            id: assertion_ids[1].clone(),
            entry: entry.clone(),
            occurrence: occurrences[1].clone(),
            native_sense_id: Presence::Missing,
            language: Presence::Value("en".into()),
            definition: Presence::Value(TextValue {
                text: "grow".into(),
                language: Presence::Missing,
                markup: Presence::Missing,
            }),
            source_score: Presence::Missing,
            timestamps: Vec::new(),
            eligibility: Eligibility::Lexical,
        },
    ];
    let evidence = |occurrence: OccurrenceRef| {
        Some(SourceEvidence {
            occurrence,
            field_index: None,
            citation: Presence::Missing,
            offset: None,
        })
    };
    metadata.forms = vec![
        FormRecord {
            id: EntityId::new("lexical-upstream-form-variant").unwrap(),
            entry: entry.clone(),
            spelling: "farvi-form".into(),
            language: Presence::Value("jbo".into()),
            script: Presence::Missing,
            pronunciation: Presence::Missing,
            kind: FormKind::SourceVariant,
            standing: Presence::Missing,
            components: Vec::new(),
            evidence: evidence(OccurrenceRef {
                capture: old_capture,
                locator: "/old".into(),
            }),
            computation: None,
        },
        FormRecord {
            id: EntityId::new("lexical-upstream-form-pronunciation").unwrap(),
            entry: entry.clone(),
            spelling: "farvi-form".into(),
            language: Presence::Value("jbo".into()),
            script: Presence::Missing,
            pronunciation: Presence::Value(TextValue {
                text: "farvi".into(),
                language: Presence::Value("jbo".into()),
                markup: Presence::Missing,
            }),
            kind: FormKind::SourcePronunciation,
            standing: Presence::Missing,
            components: Vec::new(),
            evidence: evidence(occurrences[0].clone()),
            computation: None,
        },
        FormRecord {
            id: EntityId::new("lexical-upstream-form-reported").unwrap(),
            entry: entry.clone(),
            spelling: "farvi-form".into(),
            language: Presence::Value("jbo".into()),
            script: Presence::Missing,
            pronunciation: Presence::Missing,
            kind: FormKind::SourceReportedForm,
            standing: Presence::Missing,
            components: Vec::new(),
            evidence: evidence(occurrences[1].clone()),
            computation: None,
        },
        FormRecord {
            id: EntityId::new("lexical-upstream-form-rafsi").unwrap(),
            entry,
            spelling: "farvi-form".into(),
            language: Presence::Value("jbo".into()),
            script: Presence::Missing,
            pronunciation: Presence::Missing,
            kind: FormKind::AssignedRafsi,
            standing: Presence::Missing,
            components: Vec::new(),
            evidence: evidence(occurrences[0].clone()),
            computation: None,
        },
    ];
    Dictionary::from_document(DictionaryDocument {
        registry,
        captures: vec![capture],
        metadata,
        ..DictionaryDocument::default()
    })
    .expect("fixed LU typed records preserve loaded and unavailable evidence")
}

fn build_view(root: &std::path::Path, dictionary: &Dictionary) -> PathBuf {
    let catalogue = root.join("catalogue");
    let sealed = seal_release(PackId::Llg, PackPayloadKind::Synthetic, dictionary).unwrap();
    let request = CollectionRequest::new(
        vec![CollectionInput::SealedBytes {
            pack_id: PackId::Llg,
            release: Cow::Borrowed(&sealed.release),
            seal: Cow::Borrowed(&sealed.seal),
        }],
        CollectionSelection::Packs(vec![PackId::Llg]),
        true,
    )
    .unwrap();
    CollectionCatalogue::install(&catalogue, request).unwrap();

    let view_root = root.join("view");
    WorkbenchView::build_new(
        &view_root,
        WorkbenchBuildRequest {
            catalogue_root: Some(catalogue),
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
    view_root
}

fn owned_release(
    id: &str,
    component: &EntityId,
    captures: Vec<CaptureRef>,
    assertions: Vec<EntityId>,
) -> ReleaseRecord {
    let count = assertions.len() as u64;
    ReleaseRecord {
        id: EntityId::new(id).unwrap(),
        component: component.clone(),
        distributor: None,
        upstream_revision: Presence::Missing,
        distributor_revision: Presence::Missing,
        captured_at: Presence::Missing,
        time_precision: TimePrecision::Unknown,
        schema_version: 1,
        normalizer: "lexical-contract-fixture".into(),
        normalizer_version: "1".into(),
        captures,
        assertions,
        accounting: ReleaseAccounting {
            input_occurrences: Some(count),
            normalized_occurrences: Some(count),
            query_excluded_occurrences: Some(0),
            unit: "occurrences".into(),
        },
        diagnostics: Vec::new(),
        selected: true,
        availability: Availability::Available,
        content_notices: vec![ContentNotice {
            text: Presence::Value("synthetic lexical contract fixture".into()),
            locator: Presence::Missing,
            license: Presence::Missing,
        }],
        redistribution: RedistributionStatus::Held,
        redistribution_evidence: Vec::new(),
    }
}

fn owned_genesis() -> Dictionary {
    let source = SourceId::new("lexical-owned").unwrap();
    let editor = ContributorId::new("lexical-editor").unwrap();
    let collection = EntityId::new("lexical-owned-collection").unwrap();
    let component = EntityId::new("lexical-owned-component").unwrap();
    let revision = EntityId::new("lexical-owned-r0").unwrap();
    let registry = SourceRegistry::from_document(RegistryDocument {
        sources: vec![SourceRecord {
            id: source.clone(),
            display_name: "Lexical owned fixture".into(),
            abbreviation: "LO".into(),
            abbreviation_kind: AbbreviationKind::ProjectAssigned,
            abbreviation_evidence: Some("test fixture".into()),
            origin: SourceOrigin::OwnedDraft,
        }],
        contributors: vec![ContributorRecord {
            id: editor.clone(),
            display_name: "Lexical fixture editor".into(),
        }],
        aliases: vec![SourceAlias {
            source: source.clone(),
            text: "lexical-shared-legacy".into(),
            status: AliasStatus::Retired,
            evidence: Some("S04 retained alias fixture".into()),
        }],
        ..RegistryDocument::default()
    })
    .unwrap();
    let mut metadata = DictionaryMetadata::default();
    metadata.collections.push(CollectionRecord {
        id: collection,
        source: source.clone(),
        display_name: "Lexical owned collection".into(),
        description: Presence::Missing,
    });
    metadata.components.push(ComponentRecord {
        id: component.clone(),
        collection: EntityId::new("lexical-owned-collection").unwrap(),
        native_name: "farvi forms".into(),
        languages: vec!["jbo".into()],
    });
    metadata.releases.push(owned_release(
        "lexical-owned-release-r0",
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
        rationale: Presence::Value(TextValue {
            text: "empty genesis".into(),
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

fn owned_revision_one(genesis: &Dictionary) -> Dictionary {
    let source = SourceId::new("lexical-owned").unwrap();
    let editor = ContributorId::new("lexical-editor").unwrap();
    let capture = RawCapture::new(
        "captured:lexical-owned:r1",
        CaptureMetadata {
            source_name: "Lexical owned fixture".into(),
            format_label: "text/plain".into(),
            locator: "owned r1".into(),
            revision: Presence::Missing,
            captured_at: Presence::Missing,
            time_precision: TimePrecision::Unknown,
            notice_locator: Presence::Missing,
        },
        b"farvi grow\nfarvi develop\n".to_vec(),
        vec![
            RawOccurrence {
                locator: "/a".into(),
                fields: Vec::new(),
                raw_line: Some("farvi grow".into()),
            },
            RawOccurrence {
                locator: "/b".into(),
                fields: Vec::new(),
                raw_line: Some("farvi develop".into()),
            },
        ],
    );
    let capture_ref = capture.identity.clone();
    let occurrences = ["/a", "/b"].map(|locator| OccurrenceRef {
        capture: capture_ref.clone(),
        locator: locator.into(),
    });
    let mut document = genesis.document();
    document.captures.push(capture);
    let mut registry = document.registry.document();
    registry.snapshots.push(SnapshotAttribution {
        snapshot: capture_ref.to_snapshot_identity().unwrap(),
        source: source.clone(),
        distributor: None,
    });
    for (index, occurrence) in occurrences.iter().enumerate() {
        registry.occurrences.push(OccurrenceAttribution {
            occurrence: occurrence.to_record_identity().unwrap(),
            contributor: Some(editor.clone()),
            native_entry_id: Some("lexical-owned-entry".into()),
            native_assertion_id: Some(format!(
                "lexical-owned-{}",
                if index == 0 { "a" } else { "b" }
            )),
        });
    }
    document.registry = SourceRegistry::from_document(registry).unwrap();
    let main_entry = EntityId::new("lexical-owned-entry").unwrap();
    let empty_entry = EntityId::new("lexical-owned-empty-entry").unwrap();
    document.metadata.entries = vec![
        EntryRecord {
            id: main_entry.clone(),
            source: source.clone(),
            spelling: Presence::Value("farvi".into()),
            language: Presence::Value("jbo".into()),
            script: Presence::Missing,
            native_word_class: Presence::Missing,
            selmaho: Presence::Missing,
            mapped_pos: Presence::Missing,
        },
        EntryRecord {
            id: empty_entry.clone(),
            source: source.clone(),
            spelling: Presence::Value("zero-associated".into()),
            language: Presence::Value("jbo".into()),
            script: Presence::Missing,
            native_word_class: Presence::Missing,
            selmaho: Presence::Missing,
            mapped_pos: Presence::Missing,
        },
    ];
    let assertion_a = EntityId::new("lexical-owned-a").unwrap();
    let assertion_b = EntityId::new("lexical-owned-b").unwrap();
    document.metadata.assertions = vec![
        AssertionRecord {
            id: assertion_a.clone(),
            entry: main_entry.clone(),
            occurrence: occurrences[0].clone(),
            native_sense_id: Presence::Missing,
            language: Presence::Value("en".into()),
            definition: Presence::Value(TextValue {
                text: "grow".into(),
                language: Presence::Missing,
                markup: Presence::Missing,
            }),
            source_score: Presence::Missing,
            timestamps: Vec::new(),
            eligibility: Eligibility::Lexical,
        },
        AssertionRecord {
            id: assertion_b.clone(),
            entry: main_entry.clone(),
            occurrence: occurrences[1].clone(),
            native_sense_id: Presence::Missing,
            language: Presence::Value("en".into()),
            definition: Presence::Value(TextValue {
                text: "develop".into(),
                language: Presence::Missing,
                markup: Presence::Missing,
            }),
            source_score: Presence::Missing,
            timestamps: Vec::new(),
            eligibility: Eligibility::Lexical,
        },
    ];
    let computation_ids = [
        EntityId::new("lexical-owned-computation-stem").unwrap(),
        EntityId::new("lexical-owned-computation-components").unwrap(),
        EntityId::new("lexical-owned-computation-candidate").unwrap(),
    ];
    document.metadata.computations = computation_ids
        .iter()
        .map(|id| ComputationRecord {
            id: id.clone(),
            tool: "fixture".into(),
            tool_version: "1".into(),
            inputs: vec![InputRef::Literal("farvi".into())],
            policy: "fixture-literal".into(),
            profile: Presence::Missing,
            explanation: Presence::Missing,
        })
        .collect();
    document.metadata.forms = vec![
        FormRecord {
            id: EntityId::new("lexical-owned-form-stem").unwrap(),
            entry: main_entry,
            spelling: "farvi-form".into(),
            language: Presence::Value("jbo".into()),
            script: Presence::Missing,
            pronunciation: Presence::Missing,
            kind: FormKind::ComputedStem,
            standing: Presence::Missing,
            components: Vec::new(),
            evidence: None,
            computation: Some(computation_ids[0].clone()),
        },
        FormRecord {
            id: EntityId::new("lexical-owned-form-components").unwrap(),
            entry: empty_entry.clone(),
            spelling: "farvi-form".into(),
            language: Presence::Value("jbo".into()),
            script: Presence::Missing,
            pronunciation: Presence::Missing,
            kind: FormKind::ComputedComponents,
            standing: Presence::Missing,
            components: Vec::new(),
            evidence: None,
            computation: Some(computation_ids[1].clone()),
        },
        FormRecord {
            id: EntityId::new("lexical-owned-form-candidate").unwrap(),
            entry: empty_entry,
            spelling: "farvi-form".into(),
            language: Presence::Value("jbo".into()),
            script: Presence::Missing,
            pronunciation: Presence::Missing,
            kind: FormKind::GeneratedCandidate,
            standing: Presence::Missing,
            components: Vec::new(),
            evidence: None,
            computation: Some(computation_ids[2].clone()),
        },
    ];
    let component = document.metadata.components[0].id.clone();
    document.metadata.releases.push(owned_release(
        "lexical-owned-release-r1",
        &component,
        vec![capture_ref],
        vec![assertion_a.clone(), assertion_b.clone()],
    ));
    let revision = EntityId::new("lexical-owned-r1").unwrap();
    document.metadata.catalogue_revision = 1;
    document.metadata.owned_revisions.push(OwnedRevisionRecord {
        id: revision.clone(),
        source: source.clone(),
        parent: Some(EntityId::new("lexical-owned-r0").unwrap()),
        editor,
        timestamp: Presence::Missing,
        time_precision: TimePrecision::Unknown,
        assertions: vec![assertion_a, assertion_b],
        tombstones: Vec::new(),
        rationale: Presence::Value(TextValue {
            text: "add a and b".into(),
            language: Presence::Missing,
            markup: Presence::Missing,
        }),
    });
    document.metadata.owned_heads[0].revision = revision;
    Dictionary::from_document(document).unwrap()
}

fn owned_revision_two(previous: &Dictionary) -> Dictionary {
    let mut document = previous.document();
    let source = SourceId::new("lexical-owned").unwrap();
    let editor = ContributorId::new("lexical-editor").unwrap();
    let capture = RawCapture::new(
        "captured:lexical-owned:r2",
        CaptureMetadata {
            source_name: "Lexical owned fixture".into(),
            format_label: "text/plain".into(),
            locator: "owned r2".into(),
            revision: Presence::Missing,
            captured_at: Presence::Missing,
            time_precision: TimePrecision::Unknown,
            notice_locator: Presence::Missing,
        },
        b"farvi flourish\n".to_vec(),
        vec![RawOccurrence {
            locator: "/c".into(),
            fields: Vec::new(),
            raw_line: Some("farvi flourish".into()),
        }],
    );
    let capture_ref = capture.identity.clone();
    let occurrence = OccurrenceRef {
        capture: capture_ref.clone(),
        locator: "/c".into(),
    };
    document.captures.push(capture);
    let mut registry = document.registry.document();
    registry.snapshots.push(SnapshotAttribution {
        snapshot: capture_ref.to_snapshot_identity().unwrap(),
        source: source.clone(),
        distributor: None,
    });
    registry.occurrences.push(OccurrenceAttribution {
        occurrence: occurrence.to_record_identity().unwrap(),
        contributor: Some(editor.clone()),
        native_entry_id: Some("lexical-owned-entry".into()),
        native_assertion_id: Some("lexical-owned-c".into()),
    });
    document.registry = SourceRegistry::from_document(registry).unwrap();
    let assertion = EntityId::new("lexical-owned-c").unwrap();
    document.metadata.assertions.push(AssertionRecord {
        id: assertion.clone(),
        entry: EntityId::new("lexical-owned-entry").unwrap(),
        occurrence,
        native_sense_id: Presence::Missing,
        language: Presence::Value("en".into()),
        definition: Presence::Value(TextValue {
            text: "flourish".into(),
            language: Presence::Missing,
            markup: Presence::Missing,
        }),
        source_score: Presence::Missing,
        timestamps: Vec::new(),
        eligibility: Eligibility::Lexical,
    });
    let component = document.metadata.components[0].id.clone();
    document.metadata.releases.push(owned_release(
        "lexical-owned-release-r2",
        &component,
        vec![capture_ref],
        vec![assertion.clone()],
    ));
    let previous_revision = document.metadata.owned_revisions.last().unwrap().clone();
    let revision = EntityId::new("lexical-owned-r2").unwrap();
    document.metadata.catalogue_revision = 2;
    document.metadata.owned_revisions.push(OwnedRevisionRecord {
        id: revision.clone(),
        source,
        parent: Some(previous_revision.id),
        editor,
        timestamp: Presence::Missing,
        time_precision: TimePrecision::Unknown,
        assertions: vec![assertion],
        tombstones: vec![
            EntityId::new("lexical-owned-a").unwrap(),
            EntityId::new("lexical-owned-b").unwrap(),
        ],
        rationale: Presence::Value(TextValue {
            text: "replace a and b with c".into(),
            language: Presence::Missing,
            markup: Presence::Missing,
        }),
    });
    document.metadata.owned_heads[0].revision = revision;
    Dictionary::from_document(document).unwrap()
}

fn build_mixed_view(
    root: &std::path::Path,
    upstream: &Dictionary,
    owned_root: &std::path::Path,
    owned_revision: WorkbenchOwnedRevision,
    view_name: &str,
) -> PathBuf {
    let catalogue = root.join("catalogue");
    if !catalogue.exists() {
        let sealed = seal_release(PackId::LaLojban, PackPayloadKind::Synthetic, upstream).unwrap();
        let request = CollectionRequest::new(
            vec![CollectionInput::SealedBytes {
                pack_id: PackId::LaLojban,
                release: Cow::Borrowed(&sealed.release),
                seal: Cow::Borrowed(&sealed.seal),
            }],
            CollectionSelection::Packs(vec![PackId::LaLojban]),
            true,
        )
        .unwrap();
        CollectionCatalogue::install(&catalogue, request).unwrap();
    }
    let view_root = root.join(view_name);
    WorkbenchView::build_new(
        &view_root,
        WorkbenchBuildRequest {
            catalogue_root: Some(catalogue),
            owned_root: Some(owned_root.to_path_buf()),
            selection: WorkbenchSelection {
                version: 1,
                packs: vec![PackId::LaLojban],
                disabled: Vec::new(),
                owned: Some(owned_revision),
                contexts: Vec::new(),
            },
        },
    )
    .unwrap();
    view_root
}

fn request(query: WorkbenchLexicalQuery) -> WorkbenchLexicalRequest {
    WorkbenchLexicalRequest {
        version: 1,
        query,
        options: WorkbenchLexicalOptions {
            sources: vec!["llg".into()],
            ..WorkbenchLexicalOptions::default()
        },
    }
}

fn request_for_sources(query: WorkbenchLexicalQuery, sources: &[&str]) -> WorkbenchLexicalRequest {
    let mut request = request(query);
    request.options.sources = sources.iter().map(|source| (*source).into()).collect();
    request
}

fn assert_stale_lexical_cursor<T>(result: Result<T, WorkbenchQueryError>) {
    assert!(matches!(
        result,
        Err(WorkbenchQueryError::Lexical(error))
            if matches!(error.as_ref(), WorkbenchLexicalError::StaleCursor)
    ));
}

fn assert_full_form_matches_input(record: &WorkbenchFormRecord, input: &DictionaryDocument) {
    let form = input
        .metadata
        .forms
        .iter()
        .find(|row| row.id == record.form.id)
        .expect("returned Form was declared in the pre-input model");
    assert_eq!(&record.form, form);
    let entry = input
        .metadata
        .entries
        .iter()
        .find(|row| row.id == form.entry)
        .expect("returned Form's Entry was declared in the pre-input model");
    assert_eq!(&record.entry, entry);
    let registry = input.registry.as_document();
    let source = registry
        .sources
        .iter()
        .find(|row| row.id == entry.source)
        .expect("returned Form's Source was declared in the pre-input registry");
    assert_eq!(&record.source, source);
    let mapped_role = match form.kind {
        FormKind::SourceReportedForm => Some(FormRole::SourceReportedForm),
        FormKind::AssignedRafsi => Some(FormRole::AssignedRafsi),
        _ => None,
    };
    assert_eq!(record.mapped_role, mapped_role);
    let computation = form.computation.as_ref().map(|id| {
        input
            .metadata
            .computations
            .iter()
            .find(|row| row.id == *id)
            .expect("computed Form's row was declared before the query")
            .clone()
    });
    assert_eq!(record.computation, computation);

    let Some(expected_evidence) = form.evidence.as_ref() else {
        assert!(record.evidence.is_none());
        return;
    };
    let observed = record
        .evidence
        .as_ref()
        .expect("source Form retains its original evidence");
    assert_eq!(&observed.evidence, expected_evidence);
    let snapshot_identity = expected_evidence
        .occurrence
        .capture
        .to_snapshot_identity()
        .unwrap();
    let snapshot = registry
        .snapshots
        .iter()
        .find(|row| row.snapshot == snapshot_identity);
    assert_eq!(observed.snapshot_attribution.as_ref(), snapshot);
    let occurrence_identity = expected_evidence.occurrence.to_record_identity().unwrap();
    let occurrence = registry
        .occurrences
        .iter()
        .find(|row| row.occurrence == occurrence_identity);
    assert_eq!(observed.occurrence_attribution.as_ref(), occurrence);
    let contributor = occurrence
        .and_then(|row| row.contributor.as_ref())
        .map(|id| {
            registry
                .contributors
                .iter()
                .find(|row| row.id == *id)
                .expect("attributed Contributor was declared before the query")
        });
    assert_eq!(observed.contributor.as_ref(), contributor);
    let distributor = snapshot.and_then(|row| row.distributor.as_ref()).map(|id| {
        registry
            .distributors
            .iter()
            .find(|row| row.id == *id)
            .expect("attributed Distributor was declared before the query")
    });
    assert_eq!(observed.distributor.as_ref(), distributor);

    if let Some(capture) = input
        .captures
        .iter()
        .find(|capture| capture.identity == expected_evidence.occurrence.capture)
    {
        let raw = capture
            .occurrences
            .iter()
            .find(|row| row.locator == expected_evidence.occurrence.locator)
            .expect("loaded evidence locator was declared before the query");
        assert_eq!(observed.status, "Loaded");
        assert_eq!(observed.reason, None);
        assert_eq!(observed.capture.as_ref(), Some(&capture.metadata));
        assert_eq!(observed.raw.as_ref(), Some(raw));
    } else {
        assert_eq!(observed.status, "CatalogueOnly");
        assert_eq!(observed.reason.as_deref(), Some("missing_capture"));
        assert!(observed.capture.is_none());
        assert!(observed.raw.is_none());
    }
}

#[test]
fn test_reverse_definition_and_stored_form_are_actual_rows_in_both_formats() {
    let root = fresh_root();
    let dictionary = source_dictionary_with_form();
    let input = dictionary.document();
    assert_eq!(dictionary.metadata().entries.len(), 1);
    assert_eq!(dictionary.metadata().assertions.len(), 2);
    let mut expected_reverse = dictionary.metadata().assertions.iter().collect::<Vec<_>>();
    for (assertion, (expected_word, expected_definition)) in expected_reverse
        .iter()
        .zip([("farvi", "to grow"), ("farvi", "to grow tall")])
    {
        assert!(input.captures.iter().any(|capture| {
            capture.identity == assertion.occurrence.capture
                && capture
                    .occurrences
                    .iter()
                    .any(|raw| raw.locator == assertion.occurrence.locator)
        }));
        assert!(matches!(
            assertion.eligibility,
            vlajmaji_lexicon::Eligibility::Lexical
        ));
        assert!(
            matches!(&assertion.definition, Presence::Value(text) if text.text == expected_definition)
        );
        let entry = dictionary
            .metadata()
            .entries
            .iter()
            .find(|entry| entry.id == assertion.entry)
            .unwrap();
        assert!(matches!(&entry.spelling, Presence::Value(spelling) if spelling == expected_word));
    }
    let release = &dictionary.metadata().releases[0];
    assert!(release.selected);
    assert_eq!(
        release.availability,
        vlajmaji_lexicon::Availability::Available
    );
    assert_eq!(release.assertions.len(), 2);
    for assertion in &dictionary.metadata().assertions {
        assert!(release.assertions.contains(&assertion.id));
    }
    let entry_spelling = |assertion: &&vlajmaji_lexicon::AssertionRecord| {
        dictionary
            .metadata()
            .entries
            .iter()
            .find(|entry| entry.id == assertion.entry)
            .and_then(|entry| match &entry.spelling {
                Presence::Value(spelling) => Some(spelling.as_str()),
                _ => None,
            })
            .unwrap()
    };
    expected_reverse.sort_by_key(entry_spelling);
    let expected_form = dictionary
        .metadata()
        .forms
        .iter()
        .find(|form| form.id == EntityId::new("form-farvi").unwrap())
        .unwrap()
        .clone();
    let expected_entry = dictionary
        .metadata()
        .entries
        .iter()
        .find(|entry| entry.id == expected_form.entry)
        .unwrap()
        .clone();
    let view_root = build_view(&root, &dictionary);
    let mut engine = WorkbenchSearchEngine::open(&view_root).unwrap();

    let reverse = request(WorkbenchLexicalQuery::ReverseDefinition {
        contains: "GROW".into(),
    });
    let full = engine.execute_lexical(&reverse).unwrap();
    let WorkbenchLexicalFullResult::ReverseDefinition(full_rows) = full.result else {
        panic!("reverse-definition request returned another operation")
    };
    assert_eq!(full_rows.totals.records, 2);
    assert_eq!(full_rows.groups.len(), 1);
    assert_eq!(full_rows.groups[0].hits.len(), 2);
    for (actual, expected) in full_rows.groups[0].hits.iter().zip(expected_reverse.iter()) {
        assert_eq!(&actual.assertion, *expected);
        assert_eq!(
            &actual.entry,
            dictionary
                .metadata()
                .entries
                .iter()
                .find(|entry| entry.id == expected.entry)
                .unwrap()
        );
    }

    let compact = engine.execute_lexical_compact(&reverse).unwrap();
    let WorkbenchLexicalCompactResult::ReverseDefinition(compact_rows) = compact.result else {
        panic!("compact reverse-definition request returned another operation")
    };
    assert_eq!(compact_rows.totals.records, full_rows.totals.records);
    assert_eq!(compact_rows.groups.len(), 1);
    assert_eq!(compact_rows.groups[0].hits.len(), 2);
    for (actual, expected) in compact_rows.groups[0]
        .hits
        .iter()
        .zip(expected_reverse.iter())
    {
        assert_eq!(&actual.assertion, *expected);
    }

    let mut boundary_input = input.clone();
    boundary_input.metadata.assertions[0].definition = Presence::Missing;
    boundary_input.metadata.assertions[1].definition = Presence::Null;
    let boundary_form = boundary_input
        .metadata
        .forms
        .iter_mut()
        .find(|form| form.id.as_str() == "form-farvi")
        .expect("the explicit fixture Form is present before querying");
    boundary_form.spelling.clear();
    let boundary_dictionary = Dictionary::from_document(boundary_input).unwrap();
    let boundary_root = root.join("empty-form-and-absent-definitions");
    fs::create_dir(&boundary_root).unwrap();
    let boundary_view = build_view(&boundary_root, &boundary_dictionary);
    let mut boundary_engine = WorkbenchSearchEngine::open(&boundary_view).unwrap();
    let empty_form_request = request(WorkbenchLexicalQuery::Form {
        value: String::new(),
        role: None,
        kind: None,
    });
    let empty_form_full = boundary_engine
        .execute_lexical(&empty_form_request)
        .unwrap();
    let WorkbenchLexicalFullResult::Form(empty_form_full_rows) = empty_form_full.result else {
        panic!("Full exact empty Form query returned another operation")
    };
    assert_eq!(empty_form_full_rows.totals.records, 1);
    assert_eq!(
        empty_form_full_rows.groups[0].hits[0].form.id.as_str(),
        "form-farvi"
    );
    let empty_form_compact = boundary_engine
        .execute_lexical_compact(&empty_form_request)
        .unwrap();
    let WorkbenchLexicalCompactResult::Form(empty_form_compact_rows) = empty_form_compact.result
    else {
        panic!("Compact exact empty Form query returned another operation")
    };
    assert_eq!(empty_form_compact_rows.totals.records, 1);
    assert_eq!(
        empty_form_compact_rows.groups[0].hits[0].form_id.as_str(),
        "form-farvi"
    );
    let untrimmed_form_request = request(WorkbenchLexicalQuery::Form {
        value: " ".into(),
        role: None,
        kind: None,
    });
    let untrimmed_forms = boundary_engine
        .execute_lexical(&untrimmed_form_request)
        .unwrap();
    let WorkbenchLexicalFullResult::Form(untrimmed_form_rows) = untrimmed_forms.result else {
        panic!("Full untrimmed Form query returned another operation")
    };
    assert_eq!(untrimmed_form_rows.totals.records, 0);
    let absent_definition_request = request(WorkbenchLexicalQuery::ReverseDefinition {
        contains: "grow".into(),
    });
    let absent_definitions = boundary_engine
        .execute_lexical(&absent_definition_request)
        .unwrap();
    let WorkbenchLexicalFullResult::ReverseDefinition(absent_definition_rows) =
        absent_definitions.result
    else {
        panic!("Full missing/null definition query returned another operation")
    };
    assert_eq!(absent_definition_rows.totals.records, 0);

    let mut unicode_definition_input = input.clone();
    unicode_definition_input.metadata.assertions[1].definition = Presence::Value(TextValue {
        text: "to grow tallé".into(),
        language: Presence::Value("en".into()),
        markup: Presence::Missing,
    });
    let unicode_definition_dictionary =
        Dictionary::from_document(unicode_definition_input).unwrap();
    let unicode_root = root.join("unicode-definition");
    fs::create_dir(&unicode_root).unwrap();
    let unicode_view = build_view(&unicode_root, &unicode_definition_dictionary);
    let mut unicode_engine = WorkbenchSearchEngine::open(&unicode_view).unwrap();
    let lowercase_non_ascii = request(WorkbenchLexicalQuery::ReverseDefinition {
        contains: "tallé".into(),
    });
    let full_lowercase_non_ascii = unicode_engine
        .execute_lexical(&lowercase_non_ascii)
        .unwrap();
    let WorkbenchLexicalFullResult::ReverseDefinition(full_lowercase_non_ascii_rows) =
        full_lowercase_non_ascii.result
    else {
        panic!("Full non-ASCII literal returned another operation")
    };
    assert_eq!(full_lowercase_non_ascii_rows.totals.records, 1);
    assert_eq!(
        full_lowercase_non_ascii_rows.groups[0].hits[0]
            .assertion
            .id
            .as_str(),
        expected_reverse[1].id.as_str()
    );
    let uppercase_non_ascii = request(WorkbenchLexicalQuery::ReverseDefinition {
        contains: "tallÉ".into(),
    });
    let full_uppercase_non_ascii = unicode_engine
        .execute_lexical(&uppercase_non_ascii)
        .unwrap();
    let WorkbenchLexicalFullResult::ReverseDefinition(full_uppercase_non_ascii_rows) =
        full_uppercase_non_ascii.result
    else {
        panic!("Full non-ASCII case query returned another operation")
    };
    assert_eq!(full_uppercase_non_ascii_rows.totals.records, 0);
    let compact_lowercase_non_ascii = unicode_engine
        .execute_lexical_compact(&lowercase_non_ascii)
        .unwrap();
    let WorkbenchLexicalCompactResult::ReverseDefinition(compact_lowercase_non_ascii_rows) =
        compact_lowercase_non_ascii.result
    else {
        panic!("Compact non-ASCII literal returned another operation")
    };
    assert_eq!(compact_lowercase_non_ascii_rows.totals.records, 1);
    assert_eq!(
        compact_lowercase_non_ascii_rows.groups[0].hits[0]
            .assertion
            .id
            .as_str(),
        expected_reverse[1].id.as_str()
    );
    let compact_uppercase_non_ascii = unicode_engine
        .execute_lexical_compact(&uppercase_non_ascii)
        .unwrap();
    let WorkbenchLexicalCompactResult::ReverseDefinition(compact_uppercase_non_ascii_rows) =
        compact_uppercase_non_ascii.result
    else {
        panic!("Compact non-ASCII case query returned another operation")
    };
    assert_eq!(compact_uppercase_non_ascii_rows.totals.records, 0);

    let mut absent_definition_input = input.clone();
    absent_definition_input.metadata.assertions[0].definition = Presence::Missing;
    absent_definition_input.metadata.assertions[1].definition = Presence::Null;
    let absent_definition_dictionary = Dictionary::from_document(absent_definition_input).unwrap();
    let absent_root = root.join("absent-definitions");
    fs::create_dir(&absent_root).unwrap();
    let absent_view = build_view(&absent_root, &absent_definition_dictionary);
    let mut absent_engine = WorkbenchSearchEngine::open(&absent_view).unwrap();
    let absent_query = request(WorkbenchLexicalQuery::ReverseDefinition {
        contains: "grow".into(),
    });
    let absent_full = absent_engine.execute_lexical(&absent_query).unwrap();
    let WorkbenchLexicalFullResult::ReverseDefinition(absent_full_rows) = absent_full.result else {
        panic!("Full missing/null definition query returned another operation")
    };
    assert_eq!(absent_full_rows.totals.records, 0);
    let absent_compact = absent_engine
        .execute_lexical_compact(&absent_query)
        .unwrap();
    let WorkbenchLexicalCompactResult::ReverseDefinition(absent_compact_rows) =
        absent_compact.result
    else {
        panic!("Compact missing/null definition query returned another operation")
    };
    assert_eq!(absent_compact_rows.totals.records, 0);

    let mut first_page_request = reverse.clone();
    first_page_request.options.page_size = Some(1);
    let first_full_page = engine.execute_lexical(&first_page_request).unwrap();
    let WorkbenchLexicalFullResult::ReverseDefinition(first_full_page_rows) =
        first_full_page.result
    else {
        panic!("full first page returned another operation")
    };
    assert_eq!(first_full_page_rows.totals.records, 2);
    assert_eq!(first_full_page_rows.source_totals[0].records, 2);
    assert_eq!(first_full_page_rows.groups[0].full_count, 2);
    assert_eq!(first_full_page_rows.groups[0].returned_count, 1);
    assert!(first_full_page_rows.groups[0].partial);
    assert_eq!(
        &first_full_page_rows.groups[0].hits[0].assertion,
        expected_reverse[0]
    );
    let full_cursor = first_full_page_rows.next_cursor.unwrap();

    let mut second_page_request = first_page_request.clone();
    second_page_request.options.page_size = Some(2);
    second_page_request.options.cursor = Some(full_cursor);
    let second_full_page = engine.execute_lexical(&second_page_request).unwrap();
    let WorkbenchLexicalFullResult::ReverseDefinition(second_full_page_rows) =
        second_full_page.result
    else {
        panic!("full continuation returned another operation")
    };
    assert_eq!(
        second_full_page.binding_sha256,
        first_full_page.binding_sha256
    );
    assert_eq!(second_full_page_rows.totals.records, 2);
    assert_eq!(second_full_page_rows.source_totals[0].records, 2);
    assert_eq!(second_full_page_rows.groups[0].full_count, 2);
    assert_eq!(second_full_page_rows.groups[0].returned_count, 1);
    assert!(second_full_page_rows.groups[0].partial);
    assert_eq!(
        &second_full_page_rows.groups[0].hits[0].assertion,
        expected_reverse[1]
    );
    assert!(second_full_page_rows.next_cursor.is_none());

    let first_compact_page = engine.execute_lexical_compact(&first_page_request).unwrap();
    let WorkbenchLexicalCompactResult::ReverseDefinition(first_compact_page_rows) =
        first_compact_page.result
    else {
        panic!("compact first page returned another operation")
    };
    assert_eq!(first_compact_page_rows.totals.records, 2);
    assert_eq!(first_compact_page_rows.source_totals[0].records, 2);
    assert_eq!(first_compact_page_rows.groups[0].full_count, 2);
    assert_eq!(first_compact_page_rows.groups[0].returned_count, 1);
    assert!(first_compact_page_rows.groups[0].partial);
    assert_eq!(
        &first_compact_page_rows.groups[0].hits[0].assertion,
        expected_reverse[0]
    );
    let compact_cursor = first_compact_page_rows.next_cursor.unwrap();
    let mut second_compact_request = first_page_request.clone();
    second_compact_request.options.page_size = Some(2);
    second_compact_request.options.cursor = Some(compact_cursor);
    let second_compact_page = engine
        .execute_lexical_compact(&second_compact_request)
        .unwrap();
    let WorkbenchLexicalCompactResult::ReverseDefinition(second_compact_page_rows) =
        second_compact_page.result
    else {
        panic!("compact continuation returned another operation")
    };
    assert_eq!(
        second_compact_page.binding_sha256,
        first_compact_page.binding_sha256
    );
    assert_eq!(second_compact_page_rows.totals.records, 2);
    assert_eq!(second_compact_page_rows.source_totals[0].records, 2);
    assert_eq!(second_compact_page_rows.groups[0].full_count, 2);
    assert_eq!(second_compact_page_rows.groups[0].returned_count, 1);
    assert!(second_compact_page_rows.groups[0].partial);
    assert_eq!(
        &second_compact_page_rows.groups[0].hits[0].assertion,
        expected_reverse[1]
    );
    assert!(second_compact_page_rows.next_cursor.is_none());

    let forms = request(WorkbenchLexicalQuery::Form {
        value: "farvi".into(),
        role: None,
        kind: Some(FormKind::SourceVariant),
    });
    let full_forms = engine.execute_lexical(&forms).unwrap();
    let WorkbenchLexicalFullResult::Form(full_forms) = full_forms.result else {
        panic!("Form request returned another operation")
    };
    assert_eq!(full_forms.totals.records, 1);
    let full_record = &full_forms.groups[0].hits[0];
    assert_eq!(full_record.form, expected_form);
    assert_eq!(full_record.entry, expected_entry);
    assert_eq!(full_record.associated_assertions.len(), 2);

    let compact_forms = engine.execute_lexical_compact(&forms).unwrap();
    let WorkbenchLexicalCompactResult::Form(compact_forms) = compact_forms.result else {
        panic!("compact Form request returned another operation")
    };
    assert_eq!(compact_forms.totals.records, 1);
    let compact_record = &compact_forms.groups[0].hits[0];
    assert_eq!(compact_record.form_id, full_record.form.id);
    assert_eq!(compact_record.associated_assertions.len(), 2);
    let detail = engine
        .form_detail(&compact_record.detail_ref, &compact.binding_sha256)
        .unwrap();
    assert_eq!(detail.form.form, full_record.form);
    assert_eq!(detail.form.entry, full_record.entry);
    assert_eq!(
        detail.form.associated_assertions,
        full_record.associated_assertions
    );

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn test_mixed_seven_form_query_tracks_owned_revision_and_preserves_evidence() {
    let root = fresh_root();
    let upstream = source_dictionary_with_four_forms();
    let upstream_input = upstream.document();
    assert_eq!(upstream_input.metadata.forms.len(), 4);
    assert_eq!(upstream_input.metadata.assertions.len(), 2);
    assert_eq!(
        upstream_input.registry.as_document().sources[0].id.as_str(),
        "lexical-upstream"
    );
    assert_eq!(
        upstream_input.registry.as_document().sources[0].abbreviation,
        "LU"
    );
    assert_eq!(upstream_input.captures.len(), 1);
    assert_eq!(
        upstream_input.captures[0]
            .occurrences
            .iter()
            .map(|occurrence| occurrence.locator.as_str())
            .collect::<Vec<_>>(),
        ["/0", "/1"]
    );
    assert_eq!(upstream_input.metadata.releases.len(), 2);
    assert_eq!(
        upstream_input.metadata.releases[0].assertions,
        vec![
            EntityId::new("lexical-upstream-a").unwrap(),
            EntityId::new("lexical-upstream-b").unwrap()
        ]
    );
    assert_eq!(
        upstream_input.metadata.releases[1].availability,
        Availability::Missing
    );
    assert!(upstream_input.metadata.releases[1].assertions.is_empty());
    assert_eq!(
        upstream_input
            .metadata
            .forms
            .iter()
            .map(|form| form.id.as_str())
            .collect::<Vec<_>>(),
        [
            "lexical-upstream-form-variant",
            "lexical-upstream-form-pronunciation",
            "lexical-upstream-form-reported",
            "lexical-upstream-form-rafsi",
        ]
    );
    assert!(
        upstream_input
            .metadata
            .forms
            .iter()
            .all(|form| form.entry.as_str() == "lexical-upstream-entry"
                && form.spelling == "farvi-form")
    );
    assert_eq!(
        upstream_input
            .metadata
            .forms
            .iter()
            .map(|form| form.kind)
            .collect::<Vec<_>>(),
        [
            FormKind::SourceVariant,
            FormKind::SourcePronunciation,
            FormKind::SourceReportedForm,
            FormKind::AssignedRafsi,
        ]
    );
    assert_eq!(
        upstream_input
            .metadata
            .assertions
            .iter()
            .map(|assertion| assertion.id.as_str())
            .collect::<Vec<_>>(),
        ["lexical-upstream-a", "lexical-upstream-b"]
    );
    assert!(matches!(
        &upstream_input.metadata.assertions[0].definition,
        Presence::Value(text) if text.text == "GROW <b>literal</b>"
    ));
    assert!(matches!(
        &upstream_input.metadata.assertions[1].definition,
        Presence::Value(text) if text.text == "grow"
    ));
    assert_eq!(
        upstream_input.metadata.assertions[0].entry,
        upstream_input.metadata.assertions[1].entry
    );
    assert!(upstream_input.metadata.forms.iter().any(|form| {
        form.evidence.as_ref().is_some_and(|evidence| {
            evidence.occurrence.capture.namespace == "captured:lexical-upstream-old"
        })
    }));
    let genesis = owned_genesis();
    let revision_one = owned_revision_one(&genesis);
    let revision_two = owned_revision_two(&revision_one);
    let r1_input = revision_one.document();
    let r2_input = revision_two.document();
    assert_eq!(r1_input.metadata.forms.len(), 3);
    assert_eq!(r1_input.metadata.computations.len(), 3);
    assert_eq!(r1_input.metadata.assertions.len(), 2);
    assert_eq!(
        r1_input
            .metadata
            .forms
            .iter()
            .map(|form| form.id.as_str())
            .collect::<Vec<_>>(),
        [
            "lexical-owned-form-stem",
            "lexical-owned-form-components",
            "lexical-owned-form-candidate",
        ]
    );
    assert_eq!(
        r1_input.metadata.owned_revisions[1].assertions,
        vec![
            EntityId::new("lexical-owned-a").unwrap(),
            EntityId::new("lexical-owned-b").unwrap()
        ]
    );
    let r2_revision = r2_input.metadata.owned_revisions.last().unwrap();
    assert_eq!(
        r2_revision.assertions,
        vec![EntityId::new("lexical-owned-c").unwrap()]
    );
    assert_eq!(
        r2_revision.tombstones,
        vec![
            EntityId::new("lexical-owned-a").unwrap(),
            EntityId::new("lexical-owned-b").unwrap()
        ]
    );
    assert!(
        r2_input
            .metadata
            .assertions
            .contains(&r1_input.metadata.assertions[0])
    );
    let expected_form_order = [
        "lexical-owned-form-stem",
        "lexical-owned-form-components",
        "lexical-owned-form-candidate",
        "lexical-upstream-form-variant",
        "lexical-upstream-form-pronunciation",
        "lexical-upstream-form-reported",
        "lexical-upstream-form-rafsi",
    ];
    let mut source_form_ids = r1_input
        .metadata
        .forms
        .iter()
        .chain(upstream_input.metadata.forms.iter())
        .map(|form| form.id.as_str())
        .collect::<Vec<_>>();
    source_form_ids.sort_unstable();
    let mut expected_sorted_ids = expected_form_order.to_vec();
    expected_sorted_ids.sort_unstable();
    assert_eq!(source_form_ids, expected_sorted_ids);
    assert!(matches!(
        &r1_input.metadata.assertions[0].definition,
        Presence::Value(text) if text.text == "grow"
    ));
    assert!(matches!(
        &r1_input.metadata.assertions[1].definition,
        Presence::Value(text) if text.text == "develop"
    ));
    assert!(
        r2_input
            .metadata
            .assertions
            .contains(&r1_input.metadata.assertions[1])
    );

    let owned_root = root.join("owned-journal");
    let (mut writer, created) = OwnedJournal::create(&owned_root, &genesis).unwrap();
    assert_eq!(created.head.sequence, 0);
    let r0_id = genesis.metadata().owned_revisions[0].id.clone();
    let committed_r1 = writer.commit(&r0_id, &revision_one).unwrap();
    assert_eq!(committed_r1.head.sequence, 1);
    let view_r1 = build_mixed_view(
        &root,
        &upstream,
        &owned_root,
        WorkbenchOwnedRevision::Head,
        "view-r1",
    );
    let mut engine_r1 = WorkbenchSearchEngine::open(&view_r1).unwrap();
    let mut unavailable_input = upstream_input.clone();
    let unavailable_source = SourceId::new("lexical-unreleased").unwrap();
    let mut unavailable_registry = unavailable_input.registry.document();
    unavailable_registry.sources.push(SourceRecord {
        id: unavailable_source,
        display_name: "Registered source with no releases".into(),
        abbreviation: "LX".into(),
        abbreviation_kind: AbbreviationKind::ProjectAssigned,
        abbreviation_evidence: Some("S04 unavailable-source control".into()),
        origin: SourceOrigin::Upstream,
    });
    unavailable_input.registry = SourceRegistry::from_document(unavailable_registry).unwrap();
    let unavailable_dictionary = Dictionary::from_document(unavailable_input).unwrap();
    let unavailable_root = root.join("registered-source-without-release");
    fs::create_dir(&unavailable_root).unwrap();
    let unavailable_view = build_view(&unavailable_root, &unavailable_dictionary);
    let mut unavailable_engine = WorkbenchSearchEngine::open(&unavailable_view).unwrap();
    let unavailable_source_request = request_for_sources(
        WorkbenchLexicalQuery::ReverseDefinition {
            contains: "grow".into(),
        },
        &["LX"],
    );
    let unavailable_source_result = unavailable_engine.execute_lexical(&unavailable_source_request);
    assert!(
        match &unavailable_source_result {
            Err(WorkbenchQueryError::Lexical(error)) => matches!(
                error.as_ref(),
                WorkbenchLexicalError::Dictionary(query_error)
                    if matches!(query_error.as_ref(), DictionaryQueryError::UnavailableSource {
                    source, reason: SourceSelectionFailure::NoReleases
                    } if source.as_str() == "lexical-unreleased")
            ),
            _ => false,
        },
        "disabled upstream selector produced {unavailable_source_result:?}"
    );
    let query = request_for_sources(
        WorkbenchLexicalQuery::Form {
            value: "farvi-form".into(),
            role: None,
            kind: None,
        },
        &["LU", "LO"],
    );
    let reverse_query = request_for_sources(
        WorkbenchLexicalQuery::ReverseDefinition {
            contains: "grow".into(),
        },
        &["LU", "LO"],
    );
    let reverse_r1 = engine_r1.execute_lexical(&reverse_query).unwrap();
    let WorkbenchLexicalFullResult::ReverseDefinition(reverse_r1_rows) = reverse_r1.result else {
        panic!("r1 reverse-definition request returned another operation")
    };
    assert_eq!(reverse_r1_rows.totals.records, 3);
    let reverse_r1_ids = reverse_r1_rows
        .groups
        .iter()
        .flat_map(|group| group.hits.iter().map(|hit| hit.assertion.id.as_str()))
        .collect::<Vec<_>>();
    assert_eq!(
        reverse_r1_ids,
        [
            "lexical-owned-a",
            "lexical-upstream-a",
            "lexical-upstream-b"
        ]
    );
    let alias_query = request_for_sources(
        WorkbenchLexicalQuery::ReverseDefinition {
            contains: "grow".into(),
        },
        &["lexical-shared-legacy"],
    );
    let alias_result = engine_r1.execute_lexical(&alias_query).unwrap();
    let WorkbenchLexicalFullResult::ReverseDefinition(alias_rows) = alias_result.result else {
        panic!("active alias query returned another operation")
    };
    assert_eq!(alias_rows.totals.records, 2);
    assert_eq!(alias_rows.groups.len(), 1);
    assert_eq!(alias_rows.groups[0].source.id.as_str(), "lexical-upstream");
    assert_eq!(
        alias_rows.groups[0]
            .hits
            .iter()
            .map(|hit| hit.assertion.id.as_str())
            .collect::<Vec<_>>(),
        ["lexical-upstream-a", "lexical-upstream-b"]
    );
    let mut ambiguous_alias = alias_query.clone();
    ambiguous_alias.options.alias_scope = vlajmaji_lexicon::AliasScope::IncludeRetired;
    assert!(matches!(
        engine_r1.execute_lexical(&ambiguous_alias),
        Err(WorkbenchQueryError::Lexical(error))
            if matches!(error.as_ref(), WorkbenchLexicalError::Dictionary(query_error)
                if matches!(query_error.as_ref(), DictionaryQueryError::Registry(
                    RegistryError::AmbiguousAlias { .. }
                )))
    ));
    let unknown_source = request_for_sources(
        WorkbenchLexicalQuery::ReverseDefinition {
            contains: "grow".into(),
        },
        &["lexical-not-registered"],
    );
    assert!(matches!(
        engine_r1.execute_lexical(&unknown_source),
        Err(WorkbenchQueryError::Lexical(error))
            if matches!(error.as_ref(), WorkbenchLexicalError::Dictionary(query_error)
                if matches!(query_error.as_ref(), DictionaryQueryError::Registry(
                    RegistryError::UnknownSource(_)
                )))
    ));
    for hit in reverse_r1_rows.groups.iter().flat_map(|group| &group.hits) {
        let expected = r1_input
            .metadata
            .assertions
            .iter()
            .chain(upstream_input.metadata.assertions.iter())
            .find(|assertion| assertion.id == hit.assertion.id)
            .unwrap();
        assert_eq!(&hit.assertion, expected);
    }
    let reverse_r1_compact = engine_r1.execute_lexical_compact(&reverse_query).unwrap();
    let WorkbenchLexicalCompactResult::ReverseDefinition(reverse_r1_compact_rows) =
        reverse_r1_compact.result
    else {
        panic!("compact r1 reverse-definition request returned another operation")
    };
    assert_eq!(reverse_r1_compact_rows.totals.records, 3);
    assert_eq!(
        reverse_r1_compact_rows
            .groups
            .iter()
            .flat_map(|group| group.hits.iter().map(|hit| hit.assertion.id.as_str()))
            .collect::<Vec<_>>(),
        [
            "lexical-owned-a",
            "lexical-upstream-a",
            "lexical-upstream-b"
        ]
    );
    for (needle, expected_ids) in [
        ("GROW <b>", vec!["lexical-upstream-a"]),
        (" ", vec!["lexical-upstream-a"]),
        ("\0", Vec::<&str>::new()),
        ("grów", Vec::<&str>::new()),
    ] {
        let edge_request = request_for_sources(
            WorkbenchLexicalQuery::ReverseDefinition {
                contains: needle.into(),
            },
            &["LU", "LO"],
        );
        let full_edge = engine_r1.execute_lexical(&edge_request).unwrap();
        let WorkbenchLexicalFullResult::ReverseDefinition(full_edge_rows) = full_edge.result else {
            panic!("Full edge query returned another operation")
        };
        let full_ids = full_edge_rows
            .groups
            .iter()
            .flat_map(|group| group.hits.iter().map(|hit| hit.assertion.id.as_str()))
            .collect::<Vec<_>>();
        assert_eq!(full_ids, expected_ids, "literal Full query {needle:?}");
        let compact_edge = engine_r1.execute_lexical_compact(&edge_request).unwrap();
        let WorkbenchLexicalCompactResult::ReverseDefinition(compact_edge_rows) =
            compact_edge.result
        else {
            panic!("Compact edge query returned another operation")
        };
        let compact_ids = compact_edge_rows
            .groups
            .iter()
            .flat_map(|group| group.hits.iter().map(|hit| hit.assertion.id.as_str()))
            .collect::<Vec<_>>();
        assert_eq!(
            compact_ids, expected_ids,
            "literal Compact query {needle:?}"
        );
    }
    let empty_contains = request_for_sources(
        WorkbenchLexicalQuery::ReverseDefinition {
            contains: String::new(),
        },
        &["LU", "LO"],
    );
    assert!(matches!(
        engine_r1.execute_lexical(&empty_contains),
        Err(WorkbenchQueryError::Lexical(error))
            if matches!(error.as_ref(), WorkbenchLexicalError::InvalidRequest { .. })
    ));
    assert!(matches!(
        engine_r1.execute_lexical_compact(&empty_contains),
        Err(WorkbenchQueryError::Lexical(error))
            if matches!(error.as_ref(), WorkbenchLexicalError::InvalidRequest { .. })
    ));
    let full = engine_r1.execute_lexical(&query).unwrap();
    let WorkbenchLexicalFullResult::Form(full_forms) = full.result else {
        panic!("mixed Form request returned another operation")
    };
    assert_eq!(full_forms.totals.records, 7);
    assert_eq!(full_forms.totals.source_groups, 2);
    assert_eq!(full_forms.totals.associated_assertions, 4);
    let mut all_full_ids = Vec::new();
    for group in &full_forms.groups {
        for record in &group.hits {
            all_full_ids.push(record.form.id.as_str().to_owned());
            if let Some(expected) = upstream_input
                .metadata
                .forms
                .iter()
                .find(|form| form.id == record.form.id)
            {
                assert_eq!(&record.form, expected);
                assert_full_form_matches_input(record, &upstream_input);
                assert_eq!(record.associated_assertions.len(), 2);
                for hit in &record.associated_assertions {
                    let expected_assertion = upstream_input
                        .metadata
                        .assertions
                        .iter()
                        .find(|assertion| assertion.id == hit.assertion.id)
                        .expect("eligible upstream Assertion was in the pre-input model");
                    assert_eq!(&hit.assertion, expected_assertion);
                }
                let evidence = record.evidence.as_ref().unwrap();
                if expected.id.as_str() == "lexical-upstream-form-variant" {
                    assert_eq!(evidence.status, "CatalogueOnly");
                    assert_eq!(evidence.reason.as_deref(), Some("missing_capture"));
                    assert!(evidence.capture.is_none());
                    assert!(evidence.raw.is_none());
                } else {
                    assert_eq!(evidence.status, "Loaded");
                    assert_eq!(evidence.reason, None);
                    assert!(evidence.capture.is_some());
                    assert!(evidence.raw.is_some());
                }
            } else {
                let expected = r1_input
                    .metadata
                    .forms
                    .iter()
                    .find(|form| form.id == record.form.id)
                    .unwrap();
                assert_eq!(&record.form, expected);
                assert_full_form_matches_input(record, &r1_input);
                if expected.id.as_str() == "lexical-owned-form-stem" {
                    assert_eq!(record.association_status, "eligible_assertions");
                    assert_eq!(record.associated_assertions.len(), 2);
                    for hit in &record.associated_assertions {
                        let expected_assertion = r1_input
                            .metadata
                            .assertions
                            .iter()
                            .find(|assertion| assertion.id == hit.assertion.id)
                            .expect("eligible owned Assertion was in the pre-input model");
                        assert_eq!(&hit.assertion, expected_assertion);
                    }
                } else {
                    assert_eq!(record.association_status, "no_current_eligible_assertion");
                    assert!(record.associated_assertions.is_empty());
                }
                assert!(record.evidence.is_none());
            }
        }
    }
    assert_eq!(
        all_full_ids,
        expected_form_order
            .iter()
            .map(|id| (*id).to_owned())
            .collect::<Vec<_>>()
    );
    all_full_ids.sort();
    let mut expected_ids = upstream_input
        .metadata
        .forms
        .iter()
        .chain(r1_input.metadata.forms.iter())
        .map(|form| form.id.as_str().to_owned())
        .collect::<Vec<_>>();
    expected_ids.sort();
    assert_eq!(all_full_ids, expected_ids);

    let compact = engine_r1.execute_lexical_compact(&query).unwrap();
    let compact_binding = compact.binding_sha256.clone();
    let WorkbenchLexicalCompactResult::Form(compact_forms) = compact.result else {
        panic!("mixed compact Form request returned another operation")
    };
    assert_eq!(compact_forms.totals, full_forms.totals);
    let assert_form_totals =
        |totals: &vlajmaji_search::WorkbenchLexicalTotals,
         source_totals: &[vlajmaji_search::WorkbenchLexicalSourceTotal]| {
            assert_eq!(totals.records, 7);
            assert_eq!(totals.source_groups, 2);
            assert_eq!(totals.associated_assertions, 4);
            assert_eq!(
                source_totals
                    .iter()
                    .map(|row| (
                        row.source.id.as_str(),
                        row.records,
                        row.associated_assertions
                    ))
                    .collect::<Vec<_>>(),
                [("lexical-owned", 3, 2), ("lexical-upstream", 4, 2)]
            );
        };
    let mut paged_full = query.clone();
    paged_full.options.page_size = Some(0);
    let zero_full = engine_r1.execute_lexical(&paged_full).unwrap();
    let WorkbenchLexicalFullResult::Form(zero_full_rows) = zero_full.result else {
        panic!("zero-size Full Form page returned another operation")
    };
    assert_form_totals(&zero_full_rows.totals, &zero_full_rows.source_totals);
    assert!(zero_full_rows.groups.is_empty());
    assert!(zero_full_rows.next_cursor.is_none());
    paged_full.options.page_size = Some(1);
    let first_full = engine_r1.execute_lexical(&paged_full).unwrap();
    let WorkbenchLexicalFullResult::Form(first_full_rows) = first_full.result else {
        panic!("first Full Form page returned another operation")
    };
    assert_form_totals(&first_full_rows.totals, &first_full_rows.source_totals);
    assert_eq!(
        first_full_rows
            .groups
            .iter()
            .map(|group| group.hits.len())
            .sum::<usize>(),
        1
    );
    assert_eq!(
        first_full_rows
            .groups
            .iter()
            .map(|group| (
                group.source.id.as_str(),
                group.full_count,
                group.returned_count,
                group.partial
            ))
            .collect::<Vec<_>>(),
        [("lexical-owned", 3, 1, true)]
    );
    let mut full_page_ids = first_full_rows
        .groups
        .iter()
        .flat_map(|group| group.hits.iter().map(|hit| hit.form.id.as_str().to_owned()))
        .collect::<Vec<_>>();
    let mut full_cursor = first_full_rows.next_cursor;
    assert!(full_cursor.is_some());
    for (page_size, should_continue, expected_groups) in [
        (
            3,
            true,
            vec![
                ("lexical-owned", 3, 2, true),
                ("lexical-upstream", 4, 1, true),
            ],
        ),
        (3, false, vec![("lexical-upstream", 4, 3, true)]),
    ] {
        paged_full.options.page_size = Some(page_size);
        paged_full.options.cursor = full_cursor.take();
        let page = engine_r1.execute_lexical(&paged_full).unwrap();
        let WorkbenchLexicalFullResult::Form(rows) = page.result else {
            panic!("continued Full Form page returned another operation")
        };
        assert_form_totals(&rows.totals, &rows.source_totals);
        assert_eq!(
            rows.groups
                .iter()
                .map(|group| (
                    group.source.id.as_str(),
                    group.full_count,
                    group.returned_count,
                    group.partial
                ))
                .collect::<Vec<_>>(),
            expected_groups
        );
        for group in &rows.groups {
            assert_eq!(
                group.full_count,
                if group.source.id.as_str() == "lexical-upstream" {
                    4
                } else {
                    3
                }
            );
            assert_eq!(group.returned_count, group.hits.len());
            assert_eq!(group.partial, group.returned_count < group.full_count);
            full_page_ids.extend(group.hits.iter().map(|hit| hit.form.id.as_str().to_owned()));
        }
        full_cursor = rows.next_cursor;
        assert_eq!(full_cursor.is_some(), should_continue);
    }
    assert_eq!(
        full_page_ids,
        expected_form_order
            .iter()
            .map(|id| (*id).to_owned())
            .collect::<Vec<_>>()
    );

    let mut paged_compact = query.clone();
    paged_compact.options.page_size = Some(0);
    let zero_compact = engine_r1.execute_lexical_compact(&paged_compact).unwrap();
    let WorkbenchLexicalCompactResult::Form(zero_compact_rows) = zero_compact.result else {
        panic!("zero-size Compact Form page returned another operation")
    };
    assert_form_totals(&zero_compact_rows.totals, &zero_compact_rows.source_totals);
    assert!(zero_compact_rows.groups.is_empty());
    assert!(zero_compact_rows.next_cursor.is_none());
    paged_compact.options.page_size = Some(1);
    let first_compact = engine_r1.execute_lexical_compact(&paged_compact).unwrap();
    let WorkbenchLexicalCompactResult::Form(first_compact_rows) = first_compact.result else {
        panic!("first Compact Form page returned another operation")
    };
    assert_form_totals(
        &first_compact_rows.totals,
        &first_compact_rows.source_totals,
    );
    assert_eq!(
        first_compact_rows
            .groups
            .iter()
            .map(|group| group.hits.len())
            .sum::<usize>(),
        1
    );
    assert_eq!(
        first_compact_rows
            .groups
            .iter()
            .map(|group| (
                group.source.id.as_str(),
                group.full_count,
                group.returned_count,
                group.partial
            ))
            .collect::<Vec<_>>(),
        [("lexical-owned", 3, 1, true)]
    );
    let mut compact_page_ids = first_compact_rows
        .groups
        .iter()
        .flat_map(|group| group.hits.iter().map(|hit| hit.form_id.as_str().to_owned()))
        .collect::<Vec<_>>();
    let mut compact_cursor = first_compact_rows.next_cursor;
    assert!(compact_cursor.is_some());
    for (page_size, should_continue, expected_groups) in [
        (
            3,
            true,
            vec![
                ("lexical-owned", 3, 2, true),
                ("lexical-upstream", 4, 1, true),
            ],
        ),
        (3, false, vec![("lexical-upstream", 4, 3, true)]),
    ] {
        paged_compact.options.page_size = Some(page_size);
        paged_compact.options.cursor = compact_cursor.take();
        let page = engine_r1.execute_lexical_compact(&paged_compact).unwrap();
        let WorkbenchLexicalCompactResult::Form(rows) = page.result else {
            panic!("continued Compact Form page returned another operation")
        };
        assert_form_totals(&rows.totals, &rows.source_totals);
        assert_eq!(
            rows.groups
                .iter()
                .map(|group| (
                    group.source.id.as_str(),
                    group.full_count,
                    group.returned_count,
                    group.partial
                ))
                .collect::<Vec<_>>(),
            expected_groups
        );
        for group in &rows.groups {
            assert_eq!(
                group.full_count,
                if group.source.id.as_str() == "lexical-upstream" {
                    4
                } else {
                    3
                }
            );
            assert_eq!(group.returned_count, group.hits.len());
            assert_eq!(group.partial, group.returned_count < group.full_count);
            compact_page_ids.extend(group.hits.iter().map(|hit| hit.form_id.as_str().to_owned()));
        }
        compact_cursor = rows.next_cursor;
        assert_eq!(compact_cursor.is_some(), should_continue);
    }
    assert_eq!(
        compact_page_ids,
        expected_form_order
            .iter()
            .map(|id| (*id).to_owned())
            .collect::<Vec<_>>()
    );
    for (value, role, kind, expected) in [
        (
            "farvi-form",
            Some(FormRole::SourceReportedForm),
            Some(FormKind::SourceReportedForm),
            vec!["lexical-upstream-form-reported"],
        ),
        (
            "farvi-form",
            Some(FormRole::AssignedRafsi),
            Some(FormKind::SourceReportedForm),
            Vec::<&str>::new(),
        ),
        (" farvi-form", None, None, Vec::<&str>::new()),
        ("", None, None, Vec::<&str>::new()),
    ] {
        let filtered = request_for_sources(
            WorkbenchLexicalQuery::Form {
                value: value.into(),
                role,
                kind,
            },
            &["LU", "LO"],
        );
        let filtered_full = engine_r1.execute_lexical(&filtered).unwrap();
        let WorkbenchLexicalFullResult::Form(filtered_full_rows) = filtered_full.result else {
            panic!("Full Form filter returned another operation")
        };
        let full_ids = filtered_full_rows
            .groups
            .iter()
            .flat_map(|group| group.hits.iter().map(|hit| hit.form.id.as_str()))
            .collect::<Vec<_>>();
        assert_eq!(full_ids, expected, "exact Full Form filter {value:?}");
        let filtered_compact = engine_r1.execute_lexical_compact(&filtered).unwrap();
        let WorkbenchLexicalCompactResult::Form(filtered_compact_rows) = filtered_compact.result
        else {
            panic!("Compact Form filter returned another operation")
        };
        let compact_ids = filtered_compact_rows
            .groups
            .iter()
            .flat_map(|group| group.hits.iter().map(|hit| hit.form_id.as_str()))
            .collect::<Vec<_>>();
        assert_eq!(compact_ids, expected, "exact Compact Form filter {value:?}");
    }
    let mut compact_ids = Vec::new();
    for group in &compact_forms.groups {
        for record in &group.hits {
            compact_ids.push(record.form_id.as_str().to_owned());
            let detail = engine_r1
                .form_detail(&record.detail_ref, &compact_binding)
                .unwrap();
            assert_eq!(detail.form.form.id, record.form_id);
            if let Some(expected) = upstream_input
                .metadata
                .forms
                .iter()
                .find(|form| form.id == record.form_id)
            {
                assert_eq!(&detail.form.form, expected);
                assert_full_form_matches_input(&detail.form, &upstream_input);
                assert_eq!(detail.form.associated_assertions.len(), 2);
                for hit in &detail.form.associated_assertions {
                    let expected_assertion = upstream_input
                        .metadata
                        .assertions
                        .iter()
                        .find(|assertion| assertion.id == hit.assertion.id)
                        .expect("detail Assertion was in the pre-input upstream model");
                    assert_eq!(&hit.assertion, expected_assertion);
                }
            } else {
                let expected = r1_input
                    .metadata
                    .forms
                    .iter()
                    .find(|form| form.id == record.form_id)
                    .unwrap();
                assert_eq!(&detail.form.form, expected);
                assert_full_form_matches_input(&detail.form, &r1_input);
                if expected.id.as_str() == "lexical-owned-form-stem" {
                    assert_eq!(detail.form.associated_assertions.len(), 2);
                    for hit in &detail.form.associated_assertions {
                        let expected_assertion = r1_input
                            .metadata
                            .assertions
                            .iter()
                            .find(|assertion| assertion.id == hit.assertion.id)
                            .expect("detail Assertion was in the pre-input owned model");
                        assert_eq!(&hit.assertion, expected_assertion);
                    }
                } else {
                    assert!(detail.form.associated_assertions.is_empty());
                }
            }
        }
    }
    assert_eq!(
        compact_ids,
        expected_form_order
            .iter()
            .map(|id| (*id).to_owned())
            .collect::<Vec<_>>()
    );
    compact_ids.sort();
    assert_eq!(compact_ids, expected_ids);

    let reported_detail_ref = compact_forms
        .groups
        .iter()
        .flat_map(|group| &group.hits)
        .find(|record| record.form_id.as_str() == "lexical-upstream-form-reported")
        .unwrap()
        .detail_ref
        .clone();
    let mut unknown_form = reported_detail_ref.clone();
    unknown_form.form_id = EntityId::new("lexical-missing-form").unwrap();
    assert!(matches!(
        engine_r1.form_detail(&unknown_form, &compact_binding),
        Err(WorkbenchQueryError::Lexical(error))
            if matches!(error.as_ref(), WorkbenchLexicalError::StaleFormDetail)
    ));
    let mut wrong_version = reported_detail_ref.clone();
    wrong_version.version = 2;
    assert!(matches!(
        engine_r1.form_detail(&wrong_version, &compact_binding),
        Err(WorkbenchQueryError::Lexical(error))
            if matches!(error.as_ref(), WorkbenchLexicalError::MalformedFormDetail { .. })
    ));
    let mut wrong_role = reported_detail_ref.clone();
    wrong_role.role = "assertion".into();
    assert!(matches!(
        engine_r1.form_detail(&wrong_role, &compact_binding),
        Err(WorkbenchQueryError::Lexical(error))
            if matches!(error.as_ref(), WorkbenchLexicalError::MalformedFormDetail { .. })
    ));
    let mut malformed_digest = reported_detail_ref.clone();
    malformed_digest.form_sha256 = "A".repeat(64);
    assert!(matches!(
        engine_r1.form_detail(&malformed_digest, &compact_binding),
        Err(WorkbenchQueryError::Lexical(error))
            if matches!(error.as_ref(), WorkbenchLexicalError::MalformedFormDetail { .. })
    ));
    let mut stale_digest = reported_detail_ref.clone();
    stale_digest.form_sha256 = "f".repeat(64);
    assert_ne!(stale_digest.form_sha256, reported_detail_ref.form_sha256);
    assert!(matches!(
        engine_r1.form_detail(&stale_digest, &compact_binding),
        Err(WorkbenchQueryError::Lexical(error))
            if matches!(error.as_ref(), WorkbenchLexicalError::StaleFormDetail)
    ));
    let mut stale_source = reported_detail_ref.clone();
    stale_source.source = SourceId::new("lexical-owned").unwrap();
    assert_ne!(stale_source.source, reported_detail_ref.source);
    assert!(matches!(
        engine_r1.form_detail(&stale_source, &compact_binding),
        Err(WorkbenchQueryError::Lexical(error))
            if matches!(error.as_ref(), WorkbenchLexicalError::StaleFormDetail)
    ));
    let mut stale_entry = reported_detail_ref.clone();
    stale_entry.entry_id = EntityId::new("lexical-owned-entry").unwrap();
    assert_ne!(stale_entry.entry_id, reported_detail_ref.entry_id);
    assert!(matches!(
        engine_r1.form_detail(&stale_entry, &compact_binding),
        Err(WorkbenchQueryError::Lexical(error))
            if matches!(error.as_ref(), WorkbenchLexicalError::StaleFormDetail)
    ));
    let computed_detail_ref = compact_forms
        .groups
        .iter()
        .flat_map(|group| &group.hits)
        .find(|record| record.form_id.as_str() == "lexical-owned-form-stem")
        .unwrap()
        .detail_ref
        .clone();
    assert!(computed_detail_ref.computation_sha256.is_some());
    let mut stale_computation = computed_detail_ref.clone();
    stale_computation.computation_sha256 = None;
    assert_ne!(
        stale_computation.computation_sha256,
        computed_detail_ref.computation_sha256
    );
    assert!(matches!(
        engine_r1.form_detail(&stale_computation, &compact_binding),
        Err(WorkbenchQueryError::Lexical(error))
            if matches!(error.as_ref(), WorkbenchLexicalError::StaleFormDetail)
    ));
    let mut stale_key_binding = reported_detail_ref.clone();
    stale_key_binding.binding_sha256 = "a".repeat(64);
    assert_ne!(stale_key_binding.binding_sha256, compact_binding);
    assert!(matches!(
        engine_r1.form_detail(&stale_key_binding, &compact_binding),
        Err(WorkbenchQueryError::Lexical(error))
            if matches!(error.as_ref(), WorkbenchLexicalError::StaleFormDetail)
    ));

    let historical_view = build_mixed_view(
        &root,
        &upstream,
        &owned_root,
        WorkbenchOwnedRevision::Revision(EntityId::new("lexical-owned-r1").unwrap()),
        "view-r1-historical",
    );
    let mut historical = WorkbenchSearchEngine::open(&historical_view).unwrap();
    let mut historical_first_request = reverse_query.clone();
    historical_first_request.options.page_size = Some(1);
    let historical_full_first = historical
        .execute_lexical(&historical_first_request)
        .unwrap();
    let WorkbenchLexicalFullResult::ReverseDefinition(historical_full_first_rows) =
        historical_full_first.result
    else {
        panic!("historical Full first page returned another operation")
    };
    assert_eq!(historical_full_first_rows.totals.records, 3);
    assert_eq!(historical_full_first_rows.groups[0].returned_count, 1);
    assert_eq!(
        historical_full_first_rows.groups[0].hits[0]
            .assertion
            .id
            .as_str(),
        "lexical-owned-a"
    );
    let historical_full_cursor = historical_full_first_rows.next_cursor.unwrap();
    let historical_compact_first = historical
        .execute_lexical_compact(&historical_first_request)
        .unwrap();
    let WorkbenchLexicalCompactResult::ReverseDefinition(historical_compact_first_rows) =
        historical_compact_first.result
    else {
        panic!("historical Compact first page returned another operation")
    };
    assert_eq!(historical_compact_first_rows.totals.records, 3);
    assert_eq!(historical_compact_first_rows.groups[0].returned_count, 1);
    assert_eq!(
        historical_compact_first_rows.groups[0].hits[0]
            .assertion
            .id
            .as_str(),
        "lexical-owned-a"
    );
    let historical_compact_cursor = historical_compact_first_rows.next_cursor.unwrap();

    let mut changed_query = historical_first_request.clone();
    changed_query.query = WorkbenchLexicalQuery::ReverseDefinition {
        contains: "GROW".into(),
    };
    changed_query.options.cursor = Some(historical_full_cursor.clone());
    assert_stale_lexical_cursor(historical.execute_lexical(&changed_query));
    changed_query.options.cursor = Some(historical_compact_cursor.clone());
    assert_stale_lexical_cursor(historical.execute_lexical_compact(&changed_query));

    let mut changed_source_selector = historical_first_request.clone();
    changed_source_selector.options.sources = vec!["LO".into(), "LU".into()];
    changed_source_selector.options.cursor = Some(historical_full_cursor.clone());
    assert_stale_lexical_cursor(historical.execute_lexical(&changed_source_selector));
    changed_source_selector.options.cursor = Some(historical_compact_cursor.clone());
    assert_stale_lexical_cursor(historical.execute_lexical_compact(&changed_source_selector));

    let mut changed_alias_scope = historical_first_request.clone();
    changed_alias_scope.options.alias_scope = vlajmaji_lexicon::AliasScope::IncludeRetired;
    changed_alias_scope.options.cursor = Some(historical_full_cursor.clone());
    assert_stale_lexical_cursor(historical.execute_lexical(&changed_alias_scope));
    changed_alias_scope.options.cursor = Some(historical_compact_cursor.clone());
    assert_stale_lexical_cursor(historical.execute_lexical_compact(&changed_alias_scope));

    let mut full_cursor_in_compact = historical_first_request.clone();
    full_cursor_in_compact.options.cursor = Some(historical_full_cursor.clone());
    assert_stale_lexical_cursor(historical.execute_lexical_compact(&full_cursor_in_compact));
    let mut compact_cursor_in_full = historical_first_request.clone();
    compact_cursor_in_full.options.cursor = Some(historical_compact_cursor.clone());
    assert_stale_lexical_cursor(historical.execute_lexical(&compact_cursor_in_full));

    let committed_r2 = writer
        .commit(&committed_r1.head.revision.id, &revision_two)
        .unwrap();
    assert_eq!(committed_r2.head.sequence, 2);
    assert!(matches!(
        engine_r1.execute_lexical(&query),
        Err(WorkbenchQueryError::View(_))
    ));
    assert!(matches!(
        engine_r1.form_detail(&reported_detail_ref, &compact_binding),
        Err(WorkbenchQueryError::View(_))
    ));
    let view_r2 = build_mixed_view(
        &root,
        &upstream,
        &owned_root,
        WorkbenchOwnedRevision::Head,
        "view-r2",
    );
    let mut engine_r2 = WorkbenchSearchEngine::open(&view_r2).unwrap();
    let full_r2 = engine_r2.execute_lexical(&query).unwrap();
    let WorkbenchLexicalFullResult::Form(full_r2_forms) = full_r2.result else {
        panic!("r2 Form request returned another operation")
    };
    assert_eq!(full_r2_forms.totals.records, 7);
    assert_eq!(full_r2_forms.totals.associated_assertions, 3);
    let owned_stem_r2 = full_r2_forms
        .groups
        .iter()
        .flat_map(|group| &group.hits)
        .find(|record| record.form.id.as_str() == "lexical-owned-form-stem")
        .unwrap();
    assert_eq!(owned_stem_r2.associated_assertions.len(), 1);
    assert_eq!(
        owned_stem_r2.associated_assertions[0].assertion.id.as_str(),
        "lexical-owned-c"
    );
    let reverse_r2 = engine_r2.execute_lexical(&reverse_query).unwrap();
    let WorkbenchLexicalFullResult::ReverseDefinition(reverse_r2_rows) = reverse_r2.result else {
        panic!("r2 reverse-definition request returned another operation")
    };
    assert_eq!(reverse_r2_rows.totals.records, 2);
    assert_eq!(
        reverse_r2_rows
            .groups
            .iter()
            .flat_map(|group| group.hits.iter().map(|hit| hit.assertion.id.as_str()))
            .collect::<Vec<_>>(),
        ["lexical-upstream-a", "lexical-upstream-b"]
    );
    let mut historical_continuation = historical_first_request.clone();
    historical_continuation.options.page_size = Some(2);
    historical_continuation.options.cursor = Some(historical_full_cursor);
    let historical_full_next = historical
        .execute_lexical(&historical_continuation)
        .unwrap();
    let WorkbenchLexicalFullResult::ReverseDefinition(historical_full_next_rows) =
        historical_full_next.result
    else {
        panic!("historical Full continuation returned another operation")
    };
    assert_eq!(historical_full_next_rows.totals.records, 3);
    assert_eq!(historical_full_next_rows.totals.source_groups, 2);
    assert_eq!(historical_full_next_rows.source_totals.len(), 2);
    assert_eq!(historical_full_next_rows.groups.len(), 1);
    assert_eq!(
        historical_full_next_rows.groups[0].source.id.as_str(),
        "lexical-upstream"
    );
    assert_eq!(historical_full_next_rows.groups[0].full_count, 2);
    assert_eq!(historical_full_next_rows.groups[0].returned_count, 2);
    assert!(!historical_full_next_rows.groups[0].partial);
    assert_eq!(
        historical_full_next_rows
            .groups
            .iter()
            .flat_map(|group| group.hits.iter().map(|hit| hit.assertion.id.as_str()))
            .collect::<Vec<_>>(),
        ["lexical-upstream-a", "lexical-upstream-b"]
    );
    for hit in historical_full_next_rows
        .groups
        .iter()
        .flat_map(|group| &group.hits)
    {
        let expected = upstream_input
            .metadata
            .assertions
            .iter()
            .find(|assertion| assertion.id == hit.assertion.id)
            .unwrap();
        assert_eq!(&hit.assertion, expected);
    }
    assert!(historical_full_next_rows.next_cursor.is_none());
    let mut historical_compact_continuation = historical_first_request.clone();
    historical_compact_continuation.options.page_size = Some(2);
    historical_compact_continuation.options.cursor = Some(historical_compact_cursor);
    let historical_compact_next = historical
        .execute_lexical_compact(&historical_compact_continuation)
        .unwrap();
    let WorkbenchLexicalCompactResult::ReverseDefinition(historical_compact_next_rows) =
        historical_compact_next.result
    else {
        panic!("historical Compact continuation returned another operation")
    };
    assert_eq!(historical_compact_next_rows.totals.records, 3);
    assert_eq!(historical_compact_next_rows.totals.source_groups, 2);
    assert_eq!(historical_compact_next_rows.source_totals.len(), 2);
    assert_eq!(historical_compact_next_rows.groups.len(), 1);
    assert_eq!(
        historical_compact_next_rows.groups[0].source.id.as_str(),
        "lexical-upstream"
    );
    assert_eq!(historical_compact_next_rows.groups[0].full_count, 2);
    assert_eq!(historical_compact_next_rows.groups[0].returned_count, 2);
    assert!(!historical_compact_next_rows.groups[0].partial);
    assert_eq!(
        historical_compact_next_rows
            .groups
            .iter()
            .flat_map(|group| group.hits.iter().map(|hit| hit.assertion.id.as_str()))
            .collect::<Vec<_>>(),
        ["lexical-upstream-a", "lexical-upstream-b"]
    );
    for hit in historical_compact_next_rows
        .groups
        .iter()
        .flat_map(|group| &group.hits)
    {
        let expected = upstream_input
            .metadata
            .assertions
            .iter()
            .find(|assertion| assertion.id == hit.assertion.id)
            .unwrap();
        assert_eq!(&hit.assertion, expected);
    }
    assert!(historical_compact_next_rows.next_cursor.is_none());
    let historical_result = historical.execute_lexical(&query).unwrap();
    let WorkbenchLexicalFullResult::Form(historical_forms) = historical_result.result else {
        panic!("historical Form request returned another operation")
    };
    let historical_stem = historical_forms
        .groups
        .iter()
        .flat_map(|group| &group.hits)
        .find(|record| record.form.id.as_str() == "lexical-owned-form-stem")
        .unwrap();
    assert_eq!(historical_stem.associated_assertions.len(), 2);
    assert!(
        historical_stem
            .associated_assertions
            .iter()
            .any(|hit| hit.assertion.id.as_str() == "lexical-owned-a")
    );
    assert!(
        historical_stem
            .associated_assertions
            .iter()
            .any(|hit| hit.assertion.id.as_str() == "lexical-owned-b")
    );

    // The installed release, seal, and derived database are advisory after
    // the view has been opened. Keep their exact bytes while exercising the
    // cached lexical and detail paths without those copies present.
    let catalogue_root = root.join("catalogue");
    let installed = CollectionCatalogue::open(&catalogue_root)
        .unwrap()
        .installed_artifact_locations(PackId::LaLojban)
        .unwrap()
        .unwrap();
    let compact_r2 = engine_r2.execute_lexical_compact(&query).unwrap();
    let WorkbenchLexicalCompactResult::Form(compact_r2_forms) = compact_r2.result else {
        panic!("r2 Compact Form request returned another operation")
    };
    assert_eq!(compact_r2_forms.totals.records, 7);
    assert_eq!(compact_r2_forms.totals.associated_assertions, 3);
    let detail_ref = compact_r2_forms
        .groups
        .iter()
        .flat_map(|group| &group.hits)
        .find(|record| record.form_id.as_str() == "lexical-upstream-form-reported")
        .unwrap()
        .detail_ref
        .clone();
    let compact_binding = compact_r2.binding_sha256.clone();
    let mut held = Vec::new();
    for path in [
        installed.release_path,
        installed.seal_path,
        installed.database_path,
    ] {
        let bytes = fs::read(&path).unwrap();
        let file_name = path.file_name().unwrap().to_string_lossy();
        let held_path = path.with_file_name(format!("{file_name}.s04-held"));
        assert!(!held_path.exists());
        fs::rename(&path, &held_path).unwrap();
        held.push((path, held_path, bytes));
    }
    let full_without_advisory_copies = engine_r2.execute_lexical(&query).unwrap();
    let WorkbenchLexicalFullResult::Form(full_without_advisory_rows) =
        full_without_advisory_copies.result
    else {
        panic!("r2 Full query without advisory copies returned another operation")
    };
    assert_eq!(full_without_advisory_rows.totals.records, 7);
    assert_eq!(full_without_advisory_rows.totals.associated_assertions, 3);
    let compact_without_advisory_copies = engine_r2.execute_lexical_compact(&query).unwrap();
    let WorkbenchLexicalCompactResult::Form(compact_without_advisory_rows) =
        compact_without_advisory_copies.result
    else {
        panic!("r2 Compact query without advisory copies returned another operation")
    };
    assert_eq!(compact_without_advisory_rows.totals.records, 7);
    assert_eq!(
        compact_without_advisory_rows.totals.associated_assertions,
        3
    );
    let detail = engine_r2
        .form_detail(&detail_ref, &compact_binding)
        .unwrap();
    assert_eq!(
        detail.form.form.id.as_str(),
        "lexical-upstream-form-reported"
    );
    assert_eq!(detail.form.associated_assertions.len(), 2);
    for (path, held_path, bytes) in held.into_iter().rev() {
        fs::rename(held_path, &path).unwrap();
        assert_eq!(fs::read(path).unwrap(), bytes);
    }
    fs::remove_dir_all(root).unwrap();
}
