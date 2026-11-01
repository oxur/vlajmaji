//! Public Full/Compact query contracts for explicit mixed workbench bindings.

#[path = "../../lexicon/examples/support/dictionary_fixture.rs"]
mod fixture;

use std::{
    borrow::Cow,
    collections::BTreeSet,
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

use vlajmaji_lexicon::{
    AbbreviationKind, AliasScope, AliasStatus, Availability, CaptureRef, CatalogTarget,
    CollectionRecord, ComponentRecord, ContentNotice, ContributorId, ContributorRecord, Dictionary,
    DictionaryDocument, DictionaryMetadata, Direction, EntityId, LocalRef, OccurrenceRef,
    OwnedContextPin, OwnedCurationData, OwnedCurationDocument, OwnedCurationKind,
    OwnedCurationRecord, OwnedCurationRevision, OwnedHead, OwnedPinnedTarget, OwnedRevisionRecord,
    OwnedSenseMember, PackId, PackPayloadKind, Presence, RedistributionStatus, RegistryDocument,
    RelationTarget, ReleaseAccounting, ReleaseRecord, SourceAlias, SourceId, SourceOrigin,
    SourceRecord, SourceRegistry, TextValue, TimePrecision, seal_release,
};
use vlajmaji_search::{
    DictionaryCompactQueryResult, DictionaryExactOptions, DictionaryQueryError, DictionaryRequest,
    DictionarySearchEngine, ReferenceEntity, ReferenceStatus, SourceSelectionFailure,
    WorkbenchFullResult, WorkbenchQueryError, WorkbenchSearchEngine, render_dictionary_compact,
};
use vlajmaji_store::{
    CollectionCatalogue, CollectionInput, CollectionRequest, CollectionSelection, OwnedJournal,
    WorkbenchBuildRequest, WorkbenchError, WorkbenchOwnedRevision, WorkbenchSelection,
    WorkbenchView,
};

static NEXT_ROOT: AtomicUsize = AtomicUsize::new(0);

fn fresh_root(label: &str) -> PathBuf {
    for _ in 0..64 {
        let parent = std::env::temp_dir().join(format!(
            "vlajmaji-workbench-search-{label}-{}-{}",
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

fn upstream_dictionary() -> Dictionary {
    let mut document = fixture::complete_fixture().unwrap().into_document();
    document.legacy_archive = None;
    Dictionary::from_document(document).unwrap()
}

fn install(root: &std::path::Path, dictionary: &Dictionary) {
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
    CollectionCatalogue::install(root, request).unwrap();
}

fn selection() -> WorkbenchSelection {
    WorkbenchSelection {
        version: 1,
        packs: vec![PackId::Llg],
        disabled: Vec::new(),
        owned: None,
        contexts: Vec::new(),
    }
}

fn owned_genesis() -> Dictionary {
    let source = SourceId::new("workbench-owned").unwrap();
    let editor = ContributorId::new("workbench-editor").unwrap();
    let collection = EntityId::new("workbench-collection").unwrap();
    let component = EntityId::new("workbench-component").unwrap();
    let revision = EntityId::new("workbench-r0").unwrap();
    let registry = vlajmaji_lexicon::SourceRegistry::from_document(RegistryDocument {
        sources: vec![SourceRecord {
            id: source.clone(),
            display_name: "Owned fixture".into(),
            abbreviation: "OWN".into(),
            abbreviation_kind: AbbreviationKind::ProjectAssigned,
            abbreviation_evidence: None,
            origin: SourceOrigin::OwnedDraft,
        }],
        contributors: vec![ContributorRecord {
            id: editor.clone(),
            display_name: "Workbench test".into(),
        }],
        ..RegistryDocument::default()
    })
    .unwrap();
    let mut metadata = DictionaryMetadata::default();
    metadata.collections.push(CollectionRecord {
        id: collection.clone(),
        source: source.clone(),
        display_name: "Owned fixture".into(),
        description: Presence::Missing,
    });
    metadata.components.push(ComponentRecord {
        id: component.clone(),
        collection,
        native_name: "owned".into(),
        languages: vec!["jbo".into()],
    });
    metadata.releases.push(ReleaseRecord {
        id: EntityId::new("workbench-release-r0").unwrap(),
        component,
        distributor: None,
        upstream_revision: Presence::Missing,
        distributor_revision: Presence::Missing,
        captured_at: Presence::Missing,
        time_precision: TimePrecision::Unknown,
        schema_version: 1,
        normalizer: "fixture".into(),
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
            text: Presence::Value("test fixture only; no rights grant".into()),
            locator: Presence::Missing,
            license: Presence::Missing,
        }],
        redistribution: RedistributionStatus::Held,
        redistribution_evidence: Vec::new(),
    });
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

fn owned_revision_one(genesis: &Dictionary) -> Dictionary {
    let mut document = genesis.document();
    let revision = EntityId::new("workbench-r1").unwrap();
    let previous = document.metadata.owned_revisions[0].clone();
    document.metadata.catalogue_revision = 1;
    document.metadata.owned_revisions.push(OwnedRevisionRecord {
        id: revision.clone(),
        source: previous.source,
        parent: Some(previous.id),
        editor: previous.editor,
        timestamp: Presence::Missing,
        time_precision: TimePrecision::Unknown,
        assertions: Vec::new(),
        tombstones: Vec::new(),
        rationale: Presence::Value(TextValue {
            text: "second revision".into(),
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

fn fixture_occurrence_id(reference: &OccurrenceRef) -> String {
    let mut bytes = Vec::new();
    for part in [
        reference.capture.namespace.as_bytes(),
        reference.capture.sha256.as_bytes(),
        reference.locator.as_bytes(),
    ] {
        bytes.extend_from_slice(&(part.len() as u64).to_be_bytes());
        bytes.extend_from_slice(part);
    }
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn fixture_farvi_occurrences(dictionary: &Dictionary) -> Vec<OccurrenceRef> {
    let mut occurrences = dictionary
        .captures()
        .iter()
        .filter(|capture| {
            matches!(
                capture.identity.namespace.as_str(),
                "captured:a:d1" | "captured:a:d2" | "captured:b:d1" | "owned:o:r1"
            )
        })
        .flat_map(|capture| {
            capture
                .occurrences
                .iter()
                .filter(|occurrence| {
                    occurrence.fields.iter().any(|field| {
                        (field.name == "word" || field.name == "valsiword")
                            && field.raw_value == r#""farvi""#
                    })
                })
                .map(move |occurrence| OccurrenceRef {
                    capture: capture.identity.clone(),
                    locator: occurrence.locator.clone(),
                })
        })
        .collect::<Vec<_>>();
    occurrences.sort_by_key(|reference| {
        let assertion = dictionary
            .metadata()
            .assertions
            .iter()
            .find(|row| row.occurrence == *reference)
            .expect("fixture farvi occurrence has a typed assertion");
        let entry = dictionary
            .metadata()
            .entries
            .iter()
            .find(|row| row.id == assertion.entry)
            .expect("fixture farvi assertion has a typed entry");
        (entry.source.to_string(), fixture_occurrence_id(reference))
    });
    occurrences
}

#[test]
fn test_execute_full_compact_and_cached_summary_preserve_query_contract() {
    let parent = fresh_root("query");
    let catalogue = parent.join("catalogue");
    let mut dictionary_document = upstream_dictionary().document();
    let unavailable_release = dictionary_document
        .metadata
        .releases
        .iter_mut()
        .find(|release| release.id.as_str() == "release-m")
        .expect("the fixture includes a registered unavailable release");
    unavailable_release.selected = true;
    let mut registry = dictionary_document.registry.document();
    let alias_sources = registry
        .sources
        .iter()
        .take(2)
        .map(|source| source.id.clone())
        .collect::<Vec<_>>();
    assert_eq!(
        alias_sources.len(),
        2,
        "the fixture supplies two source identities"
    );
    registry.aliases.push(SourceAlias {
        source: alias_sources[0].clone(),
        text: "workbench-single-source".into(),
        status: AliasStatus::Active,
        evidence: Some("workbench query contract fixture".into()),
    });
    registry
        .aliases
        .extend(alias_sources.iter().map(|source| SourceAlias {
            source: source.clone(),
            text: "workbench-ambiguous-source".into(),
            status: AliasStatus::Retired,
            evidence: Some("workbench ambiguity contract fixture".into()),
        }));
    dictionary_document.registry = SourceRegistry::from_document(registry).unwrap();
    let dictionary = Dictionary::from_document(dictionary_document).unwrap();
    install(&catalogue, &dictionary);
    let installed = CollectionCatalogue::open(&catalogue)
        .unwrap()
        .installed_artifact_locations(PackId::Llg)
        .unwrap()
        .unwrap();
    let release_bytes = fs::read(&installed.release_path).unwrap();
    let seal_bytes = fs::read(&installed.seal_path).unwrap();
    let output = parent.join("prepared view");
    let report = WorkbenchView::build_new(
        &output,
        WorkbenchBuildRequest {
            catalogue_root: Some(catalogue.clone()),
            owned_root: None,
            selection: selection(),
        },
    )
    .unwrap();
    let held_release = parent.join("held-release.json");
    let held_seal = parent.join("held-seal.json");
    assert!(!held_release.exists());
    assert!(!held_seal.exists());
    fs::rename(&installed.release_path, &held_release).unwrap();
    fs::rename(&installed.seal_path, &held_seal).unwrap();
    assert!(!installed.release_path.exists());
    assert!(!installed.seal_path.exists());
    let mut engine = WorkbenchSearchEngine::open(&output).unwrap();
    let request = DictionaryRequest::Exact {
        spelling: "si'e".into(),
        options: DictionaryExactOptions::default(),
    };
    let direct = DictionarySearchEngine::build(
        parent.join("direct.sqlite"),
        &dictionary.to_json_bytes().unwrap(),
    )
    .unwrap();
    let full_result = engine.execute(request.clone());
    fs::rename(&held_release, &installed.release_path).unwrap();
    fs::rename(&held_seal, &installed.seal_path).unwrap();
    assert_eq!(fs::read(&installed.release_path).unwrap(), release_bytes);
    assert_eq!(fs::read(&installed.seal_path).unwrap(), seal_bytes);
    let full = full_result.unwrap();
    let full_wire = serde_json::to_value(&full).unwrap();
    assert_eq!(
        full_wire
            .as_object()
            .unwrap()
            .keys()
            .cloned()
            .collect::<BTreeSet<_>>(),
        ["binding_sha256", "result", "selection", "version"]
            .into_iter()
            .map(str::to_owned)
            .collect(),
    );
    assert_eq!(
        serde_json::to_value(&full.selection)
            .unwrap()
            .as_object()
            .unwrap()
            .keys()
            .cloned()
            .collect::<BTreeSet<_>>(),
        [
            "context_counts",
            "curation_counts",
            "disabled",
            "owned",
            "packs",
            "upstream",
            "version"
        ]
        .into_iter()
        .map(str::to_owned)
        .collect(),
    );
    assert_eq!(full.binding_sha256, report.binding.binding_sha256());
    assert_eq!(full.selection.packs, vec!["llg".to_owned()]);
    assert_eq!(full.selection.context_counts.declared, 0);
    assert_eq!(full.selection.curation_counts, None);
    assert_eq!(
        full.result,
        WorkbenchFullResult::Exact(Box::new(
            direct
                .exact("si'e", DictionaryExactOptions::default())
                .unwrap(),
        ))
    );
    let page_one = engine
        .execute(DictionaryRequest::Exact {
            spelling: "farvi".into(),
            options: DictionaryExactOptions {
                page_size: Some(1),
                ..DictionaryExactOptions::default()
            },
        })
        .unwrap();
    let WorkbenchFullResult::Exact(page_one_exact) = page_one.result else {
        panic!("the paged request must retain the exact result tag");
    };
    let first_page_hit = page_one_exact.groups[0].hits[0].clone();
    let next_cursor = page_one_exact
        .next_cursor
        .clone()
        .expect("the fixture has more than one farvi occurrence");
    let cursor_for_other_binding = next_cursor.clone();
    let page_two = engine
        .execute(DictionaryRequest::Exact {
            spelling: "farvi".into(),
            options: DictionaryExactOptions {
                page_size: Some(1),
                cursor: Some(next_cursor),
                ..DictionaryExactOptions::default()
            },
        })
        .unwrap();
    let WorkbenchFullResult::Exact(page_two_exact) = page_two.result else {
        panic!("the cursor request must retain the exact result tag");
    };
    let direct_farvi = direct
        .exact("farvi", DictionaryExactOptions::default())
        .unwrap();
    assert_eq!(direct_farvi.totals.occurrences, 6);
    let all_direct_farvi_hits = direct_farvi
        .groups
        .iter()
        .flat_map(|group| group.hits.iter())
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(
        direct_farvi
            .groups
            .iter()
            .map(|group| (
                group.source.id.as_str(),
                group.full_count,
                group.returned_count
            ))
            .collect::<Vec<_>>(),
        vec![("source-a", 4, 4), ("source-b", 1, 1), ("source-o", 1, 1)],
        "the direct source engine exposes the input-derived eligible source roster",
    );
    let mut input_farvi_occurrences = dictionary
        .captures()
        .iter()
        .filter(|capture| {
            matches!(
                capture.identity.namespace.as_str(),
                "captured:a:d1" | "captured:a:d2" | "captured:b:d1" | "owned:o:r1"
            )
        })
        .flat_map(|capture| {
            capture
                .occurrences
                .iter()
                .filter(|occurrence| {
                    occurrence
                        .fields
                        .iter()
                        .any(|field| field.name == "word" && field.raw_value == r#""farvi""#)
                        || occurrence.fields.iter().any(|field| {
                            field.name == "valsiword" && field.raw_value == r#""farvi""#
                        })
                })
                .map(move |occurrence| vlajmaji_lexicon::OccurrenceRef {
                    capture: capture.identity.clone(),
                    locator: occurrence.locator.clone(),
                })
        })
        .collect::<Vec<_>>();
    let occurrence_id = |reference: &vlajmaji_lexicon::OccurrenceRef| {
        let mut bytes = Vec::new();
        for part in [
            reference.capture.namespace.as_bytes(),
            reference.capture.sha256.as_bytes(),
            reference.locator.as_bytes(),
        ] {
            bytes.extend_from_slice(&(part.len() as u64).to_be_bytes());
            bytes.extend_from_slice(part);
        }
        bytes
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    };
    input_farvi_occurrences.sort_by_key(|reference| {
        let assertion = dictionary
            .metadata()
            .assertions
            .iter()
            .find(|row| row.occurrence == *reference)
            .expect("every farvi capture row has a typed assertion");
        let entry = dictionary
            .metadata()
            .entries
            .iter()
            .find(|row| row.id == assertion.entry)
            .expect("every farvi assertion has a typed entry");
        (entry.source.to_string(), occurrence_id(reference))
    });
    assert_eq!(input_farvi_occurrences.len(), 6);
    assert_eq!(
        all_direct_farvi_hits
            .iter()
            .map(|hit| occurrence_id(&hit.assertion.occurrence))
            .collect::<Vec<_>>(),
        input_farvi_occurrences
            .iter()
            .map(&occurrence_id)
            .collect::<Vec<_>>(),
        "the query's ordered six-occurrence roster is derived from captured input rows",
    );
    for (hit, occurrence) in all_direct_farvi_hits.iter().zip(&input_farvi_occurrences) {
        let assertion = dictionary
            .metadata()
            .assertions
            .iter()
            .find(|row| row.occurrence == *occurrence)
            .expect("every selected fixture row has a typed assertion");
        let entry = dictionary
            .metadata()
            .entries
            .iter()
            .find(|row| row.id == assertion.entry)
            .expect("every selected assertion has its typed entry");
        let capture = dictionary
            .capture(&occurrence.capture)
            .expect("every selected occurrence has its captured source");
        assert_eq!(&hit.assertion, assertion);
        assert_eq!(&hit.entry, entry);
        assert_eq!(hit.occurrence_id, occurrence_id(occurrence));
        assert_eq!(hit.capture, capture.metadata);
        assert_eq!(
            hit.raw,
            *dictionary
                .occurrence(occurrence)
                .expect("every selected occurrence retains its exact native row")
        );
    }
    let workbench_farvi = engine
        .execute(DictionaryRequest::Exact {
            spelling: "farvi".into(),
            options: DictionaryExactOptions::default(),
        })
        .unwrap();
    let WorkbenchFullResult::Exact(workbench_farvi) = workbench_farvi.result else {
        panic!("the full farvi query must retain the Exact result tag");
    };
    let workbench_farvi_hits = workbench_farvi
        .groups
        .iter()
        .flat_map(|group| group.hits.iter())
        .collect::<Vec<_>>();
    assert_eq!(
        workbench_farvi_hits,
        all_direct_farvi_hits.iter().collect::<Vec<_>>(),
        "the retained workbench returns every direct hit in source/occurrence order",
    );
    let mut paged_hits = vec![first_page_hit];
    paged_hits.extend(
        page_two_exact
            .groups
            .iter()
            .flat_map(|group| group.hits.iter().cloned()),
    );
    let mut cursor = page_two_exact.next_cursor.clone();
    for _ in 0..6 {
        let Some(next_cursor) = cursor.take() else {
            break;
        };
        let page = engine
            .execute(DictionaryRequest::Exact {
                spelling: "farvi".into(),
                options: DictionaryExactOptions {
                    page_size: Some(2),
                    cursor: Some(next_cursor),
                    ..DictionaryExactOptions::default()
                },
            })
            .unwrap();
        let WorkbenchFullResult::Exact(exact) = page.result else {
            panic!("each cursor request must retain the exact result tag");
        };
        paged_hits.extend(
            exact
                .groups
                .iter()
                .flat_map(|group| group.hits.iter().cloned()),
        );
        cursor = exact.next_cursor;
    }
    assert!(
        cursor.is_none(),
        "the finite input roster must terminate paging"
    );
    assert_eq!(paged_hits.len(), input_farvi_occurrences.len());
    assert_eq!(paged_hits, all_direct_farvi_hits);

    let empty_page = engine
        .execute(DictionaryRequest::Exact {
            spelling: "farvi".into(),
            options: DictionaryExactOptions {
                page_size: Some(0),
                ..DictionaryExactOptions::default()
            },
        })
        .unwrap();
    let WorkbenchFullResult::Exact(empty_page_exact) = empty_page.result else {
        panic!("the zero-sized page must retain the exact result tag");
    };
    assert!(empty_page_exact.groups.is_empty());
    assert_eq!(empty_page_exact.totals, direct_farvi.totals);
    assert!(empty_page_exact.next_cursor.is_none());

    let alias_result = engine
        .execute(DictionaryRequest::Exact {
            spelling: "si'e".into(),
            options: DictionaryExactOptions {
                sources: vec!["workbench-single-source".into()],
                alias_scope: AliasScope::ActiveOnly,
                ..DictionaryExactOptions::default()
            },
        })
        .unwrap();
    let WorkbenchFullResult::Exact(alias_exact) = alias_result.result else {
        panic!("the alias request must retain the exact result tag");
    };
    assert_eq!(alias_exact.resolved_sources, vec![alias_sources[0].clone()]);
    assert_eq!(alias_exact.groups.len(), 1);
    assert_eq!(alias_exact.groups[0].source.id, alias_sources[0]);
    assert_eq!(alias_exact.groups[0].full_count, 1);
    assert_eq!(alias_exact.groups[0].returned_count, 1);
    assert_eq!(alias_exact.groups[0].hits.len(), 1);
    assert!(!alias_exact.groups[0].partial);
    let input_si_e_assertion = dictionary
        .metadata()
        .assertions
        .iter()
        .find(|assertion| {
            assertion.occurrence.capture.namespace == "captured:a:lexical"
                && assertion.occurrence.locator == "/0"
        })
        .expect("the exact alias fixture input includes its si'e assertion");
    let input_si_e_entry = dictionary
        .metadata()
        .entries
        .iter()
        .find(|entry| entry.id == input_si_e_assertion.entry)
        .expect("the si'e assertion has its typed source entry");
    let alias_hit = &alias_exact.groups[0].hits[0];
    assert_eq!(&alias_hit.entry, input_si_e_entry);
    assert_eq!(&alias_hit.assertion, input_si_e_assertion);
    assert_eq!(
        alias_hit.raw,
        *dictionary
            .occurrence(&input_si_e_assertion.occurrence)
            .expect("the si'e source occurrence remains available")
    );
    let source_result = engine
        .execute(DictionaryRequest::Exact {
            spelling: "si'e".into(),
            options: DictionaryExactOptions {
                sources: vec![alias_sources[0].to_string()],
                ..DictionaryExactOptions::default()
            },
        })
        .unwrap();
    let WorkbenchFullResult::Exact(source_exact) = source_result.result else {
        panic!("the source-ID request must retain the exact result tag");
    };
    assert_eq!(
        source_exact, alias_exact,
        "ID and alias filters select the same complete result"
    );
    assert!(matches!(
        engine.execute(DictionaryRequest::Exact {
            spelling: "si'e".into(),
            options: DictionaryExactOptions {
            sources: vec!["workbench-ambiguous-source".into()],
            alias_scope: AliasScope::IncludeRetired,
            ..DictionaryExactOptions::default()
            },
        }),
        Err(WorkbenchQueryError::Query(error))
            if matches!(error.as_ref(), vlajmaji_search::DictionaryQueryError::Registry(
                vlajmaji_lexicon::RegistryError::AmbiguousAlias { .. }
            ))
    ));
    assert!(matches!(
        engine.execute(DictionaryRequest::Exact {
            spelling: "si'e".into(),
            options: DictionaryExactOptions {
                sources: vec!["workbench-missing-source".into()],
                ..DictionaryExactOptions::default()
            },
        }),
        Err(WorkbenchQueryError::Query(error))
            if matches!(error.as_ref(), vlajmaji_search::DictionaryQueryError::Registry(
                vlajmaji_lexicon::RegistryError::UnknownSource(_)
            ))
    ));
    assert!(matches!(
        engine.execute(DictionaryRequest::Exact {
            spelling: "si'e".into(),
            options: DictionaryExactOptions {
                sources: vec!["source-m".into()],
                ..DictionaryExactOptions::default()
            },
        }),
        Err(WorkbenchQueryError::Query(error))
            if matches!(error.as_ref(), DictionaryQueryError::UnavailableSource {
                source,
                reason: SourceSelectionFailure::NoSelectedAvailableRelease,
            } if source.as_str() == "source-m")
    ));
    let mut alternate_document = dictionary.document();
    let mut alternate_registry = alternate_document.registry.document();
    alternate_registry.aliases.push(SourceAlias {
        source: alias_sources[0].clone(),
        text: "workbench-other-binding".into(),
        status: AliasStatus::Active,
        evidence: Some("cursor binding discriminator".into()),
    });
    alternate_document.registry = SourceRegistry::from_document(alternate_registry).unwrap();
    let alternate_dictionary = Dictionary::from_document(alternate_document).unwrap();
    let alternate_catalogue = parent.join("alternate-catalogue");
    install(&alternate_catalogue, &alternate_dictionary);
    let alternate_output = parent.join("alternate-view");
    WorkbenchView::build_new(
        &alternate_output,
        WorkbenchBuildRequest {
            catalogue_root: Some(alternate_catalogue),
            owned_root: None,
            selection: selection(),
        },
    )
    .unwrap();
    let mut alternate_engine = WorkbenchSearchEngine::open(&alternate_output).unwrap();
    assert!(matches!(
        alternate_engine.execute(DictionaryRequest::Exact {
            spelling: "farvi".into(),
            options: DictionaryExactOptions {
                page_size: Some(1),
                cursor: Some(cursor_for_other_binding),
                ..DictionaryExactOptions::default()
            },
        }),
        Err(WorkbenchQueryError::Query(error))
            if matches!(error.as_ref(), DictionaryQueryError::StaleCursor)
    ));
    let compact = engine.execute_compact(request).unwrap();
    let compact_wire = serde_json::to_value(&compact).unwrap();
    assert_eq!(
        compact_wire
            .as_object()
            .unwrap()
            .keys()
            .cloned()
            .collect::<BTreeSet<_>>(),
        ["binding_sha256", "result", "selection", "version"]
            .into_iter()
            .map(str::to_owned)
            .collect(),
    );
    assert_eq!(compact.binding_sha256, full.binding_sha256);
    assert_eq!(compact.selection, full.selection);
    let DictionaryCompactQueryResult::Exact(compact_exact) = &compact.result else {
        panic!("the exact request must retain its compact Exact tag");
    };
    let detail_ref = compact_exact.groups[0].hits[0].detail_ref.clone();
    let WorkbenchFullResult::Exact(full_exact) = &full.result else {
        unreachable!("the earlier exact response was asserted above");
    };
    let full_hit = &full_exact.groups[0].hits[0];
    let detail = engine.detail(&detail_ref, &full.binding_sha256).unwrap();
    let detail_wire = serde_json::to_value(&detail).unwrap();
    assert_eq!(
        detail_wire
            .as_object()
            .unwrap()
            .keys()
            .cloned()
            .collect::<BTreeSet<_>>(),
        ["binding_sha256", "result", "selection", "version"]
            .into_iter()
            .map(str::to_owned)
            .collect(),
    );
    assert_eq!(detail.binding_sha256, full.binding_sha256);
    assert_eq!(detail.result.hit.as_ref(), Some(full_hit));
    assert_eq!(detail.result.snapshot_sha256, compact_exact.snapshot_sha256);
    assert_ne!(detail.binding_sha256, detail.result.snapshot_sha256);
    fs::remove_dir_all(parent).unwrap();
}

#[test]
fn test_workbench_query_summary_omits_persisted_resolution_rows() {
    let parent = fresh_root("summary-rows");
    let catalogue = parent.join("catalogue");
    let upstream = upstream_dictionary();
    install(&catalogue, &upstream);

    let context_id = EntityId::new("summary-context").unwrap();
    let pin = OwnedContextPin::from_dictionary(context_id.clone(), &upstream).unwrap();
    let target = fixture_target(&upstream, &context_id, 0);
    let second_target = fixture_target(&upstream, &context_id, 1);
    assert_ne!(target.assertion, second_target.assertion);
    let declared_member_ids = vec![second_target.assertion.clone(), target.assertion.clone()];
    let mut expected_confirmation_ids = declared_member_ids.clone();
    expected_confirmation_ids.sort_by_key(|id| {
        let assertion = upstream
            .metadata()
            .assertions
            .iter()
            .find(|row| row.id == *id)
            .unwrap();
        let entry = upstream
            .metadata()
            .entries
            .iter()
            .find(|row| row.id == assertion.entry)
            .unwrap();
        let spelling = match &entry.spelling {
            Presence::Value(value) => value.clone(),
            Presence::Missing | Presence::Null => String::new(),
        };
        (
            entry.source.to_string(),
            spelling,
            fixture_occurrence_id(&assertion.occurrence),
        )
    });
    assert_ne!(
        declared_member_ids, expected_confirmation_ids,
        "the input declaration order must differ from canonical confirmation order",
    );
    let member_sources = declared_member_ids
        .iter()
        .map(|id| {
            let assertion = upstream
                .metadata()
                .assertions
                .iter()
                .find(|row| row.id == *id)
                .unwrap();
            upstream
                .metadata()
                .entries
                .iter()
                .find(|entry| entry.id == assertion.entry)
                .unwrap()
                .source
                .clone()
        })
        .collect::<Vec<_>>();
    assert_eq!(
        member_sources[0], member_sources[1],
        "the two member rows share one source, so Readable preserves their incoming order",
    );
    let genesis = owned_genesis();
    let r0 = EntityId::new("workbench-r0").unwrap();
    let r1 = EntityId::new("workbench-r1").unwrap();
    let sense_id = EntityId::new("summary-sense").unwrap();
    let mut records = vec![OwnedCurationRecord {
        id: sense_id.clone(),
        revision: r1.clone(),
        editor: ContributorId::new("workbench-editor").unwrap(),
        rationale: TextValue {
            text: "the summary fixture's two distinct pinned assertions".into(),
            language: Presence::Missing,
            markup: Presence::Missing,
        },
        kind: OwnedCurationKind::Sense {
            members: vec![
                OwnedSenseMember::External(second_target.clone()),
                OwnedSenseMember::External(target.clone()),
            ],
        },
    }];
    let mut active = vec![sense_id.clone()];
    for index in 0..7 {
        let id = EntityId::new(format!("summary-link-{index}")).unwrap();
        active.push(id.clone());
        records.push(OwnedCurationRecord {
            id,
            revision: r1.clone(),
            editor: ContributorId::new("workbench-editor").unwrap(),
            rationale: TextValue {
                text: format!("summary fixture relation {index}"),
                language: Presence::Missing,
                markup: Presence::Missing,
            },
            kind: OwnedCurationKind::Link {
                origin: LocalRef::EditorialSense(sense_id.clone()),
                direction: Direction::Forward,
                target: target.clone(),
            },
        });
    }
    let expected_resolution_ids = active.clone();
    let curation = OwnedCurationDocument::from_data(OwnedCurationData {
        version: 1,
        dictionary: owned_revision_one(&genesis),
        contexts: vec![pin],
        records,
        revisions: vec![
            OwnedCurationRevision {
                revision: r0.clone(),
                active: Vec::new(),
                tombstones: Vec::new(),
            },
            OwnedCurationRevision {
                revision: r1.clone(),
                active,
                tombstones: Vec::new(),
            },
        ],
    })
    .unwrap();
    let journal = parent.join("journal");
    let (mut writer, _) = OwnedJournal::create(&journal, &genesis).unwrap();
    writer.migrate_curation(&r0).unwrap();
    writer.commit_curated(&r0, &curation).unwrap();
    drop(writer);

    let output = parent.join("view");
    let context_binding = vlajmaji_store::WorkbenchContextBinding {
        context: context_id.clone(),
        packs: vec![PackId::Llg],
    };
    let report = WorkbenchView::build_new(
        &output,
        WorkbenchBuildRequest {
            catalogue_root: Some(catalogue.clone()),
            owned_root: Some(journal.clone()),
            selection: WorkbenchSelection {
                version: 1,
                packs: vec![PackId::Llg],
                disabled: Vec::new(),
                owned: Some(WorkbenchOwnedRevision::Head),
                contexts: vec![context_binding],
            },
        },
    )
    .unwrap();
    assert_eq!(report.binding.resolutions().len(), 8);
    let mut engine = WorkbenchSearchEngine::open(&output).unwrap();
    let result = engine
        .execute(DictionaryRequest::Exact {
            spelling: "si'e".into(),
            options: DictionaryExactOptions::default(),
        })
        .unwrap();
    assert_eq!(result.selection.context_counts.declared, 1);
    assert_eq!(result.selection.context_counts.provided, 1);
    let WorkbenchFullResult::Exact(exact) = &result.result else {
        panic!("the summary request must retain its exact result tag");
    };
    assert_eq!(exact.totals.occurrences, 1);
    let counts = result.selection.curation_counts.unwrap();
    assert_eq!(
        (counts.records, counts.projected, counts.inactive),
        (8, 8, 0)
    );
    assert_eq!(
        (counts.unavailable, counts.unresolved, counts.resolved),
        (0, 0, 8)
    );
    let summary = serde_json::to_value(&result.selection).unwrap();
    assert_eq!(
        summary
            .as_object()
            .unwrap()
            .keys()
            .cloned()
            .collect::<BTreeSet<_>>(),
        [
            "context_counts",
            "curation_counts",
            "disabled",
            "owned",
            "packs",
            "upstream",
            "version",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect(),
    );
    assert!(!summary.as_object().unwrap().contains_key("resolutions"));
    let owned_summary = summary["owned"].as_object().unwrap();
    assert_eq!(
        owned_summary.keys().cloned().collect::<BTreeSet<_>>(),
        [
            "curation_sha256",
            "journal_schema",
            "mode",
            "revision",
            "sequence",
            "source",
            "state_sha256",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect(),
    );
    assert_eq!(owned_summary["journal_schema"], 2);
    assert_eq!(
        summary["curation_counts"]
            .as_object()
            .unwrap()
            .keys()
            .cloned()
            .collect::<BTreeSet<_>>(),
        [
            "inactive",
            "projected",
            "records",
            "resolved",
            "unavailable",
            "unresolved"
        ]
        .into_iter()
        .map(str::to_owned)
        .collect(),
    );
    let sense_target = RelationTarget::Local(LocalRef::EditorialSense(sense_id.clone()));
    let resolved_full = engine
        .execute(DictionaryRequest::Resolve {
            target: sense_target.clone(),
        })
        .unwrap();
    let WorkbenchFullResult::Resolve(resolved_full) = resolved_full.result else {
        panic!("the local sense request must retain the Resolve result tag");
    };
    assert_eq!(resolved_full.target, sense_target);
    assert_eq!(resolved_full.status, ReferenceStatus::Resolved);
    assert!(resolved_full.unverified.is_empty());
    let Some(ReferenceEntity::EditorialSense(full_sense)) = resolved_full.entity.as_ref() else {
        panic!("the full local resolve must return its typed editorial sense");
    };
    assert_eq!(full_sense.id, sense_id);
    assert_eq!(full_sense.assertions, declared_member_ids);
    assert_eq!(
        resolved_full
            .confirmed
            .iter()
            .map(|view| view.assertion.id.clone())
            .collect::<Vec<_>>(),
        expected_confirmation_ids,
        "Full confirmation rows use canonical input-derived source/spelling/occurrence order",
    );
    let resolved_compact = engine
        .execute_compact(DictionaryRequest::Resolve {
            target: sense_target.clone(),
        })
        .unwrap();
    let DictionaryCompactQueryResult::Resolve(resolved_compact) = resolved_compact.result else {
        panic!("the compact local sense request must retain the Resolve result tag");
    };
    assert_eq!(resolved_compact.target, sense_target);
    assert_eq!(resolved_compact.status, ReferenceStatus::Resolved);
    let compact_sense = resolved_compact.entity.as_ref().unwrap();
    assert_eq!(compact_sense.role, "editorial_sense");
    assert_eq!(compact_sense.id, sense_id.as_str());
    assert_eq!(
        compact_sense.assertion_ids,
        declared_member_ids
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>(),
        "the compact local entity preserves the input-declared member order",
    );
    assert_eq!(
        resolved_compact
            .confirmed
            .iter()
            .map(|view| view.assertion.id.clone())
            .collect::<Vec<_>>(),
        declared_member_ids,
        "Compact confirmation rows preserve declared Sense member order",
    );
    let readable = render_dictionary_compact(&DictionaryCompactQueryResult::Resolve(
        resolved_compact.clone(),
    ));
    let readable_member_ids = readable
        .lines()
        .filter(|line| line.starts_with("  ") && !line.starts_with("    "))
        .filter_map(|line| {
            line.strip_prefix("  ")
                .and_then(|line| line.split_once(" @ "))
                .map(|(id, _)| id.to_owned())
        })
        .collect::<Vec<_>>();
    assert_eq!(
        readable_member_ids,
        declared_member_ids
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>(),
        "Readable groups by source but preserves declared Compact order within that source",
    );
    assert_eq!(
        engine
            .binding()
            .resolutions()
            .iter()
            .map(|resolution| resolution.record.clone())
            .collect::<Vec<_>>(),
        expected_resolution_ids,
    );
    assert_eq!(
        engine
            .binding()
            .resolutions()
            .iter()
            .filter(|resolution| resolution.projected && resolution.reason.is_none())
            .count(),
        8,
    );
    drop(engine);

    let historical_output = parent.join("historical-r1-view");
    WorkbenchView::build_new(
        &historical_output,
        WorkbenchBuildRequest {
            catalogue_root: Some(catalogue.clone()),
            owned_root: Some(journal.clone()),
            selection: WorkbenchSelection {
                version: 1,
                packs: vec![PackId::Llg],
                disabled: Vec::new(),
                owned: Some(WorkbenchOwnedRevision::Revision(r1.clone())),
                contexts: vec![vlajmaji_store::WorkbenchContextBinding {
                    context: context_id,
                    packs: vec![PackId::Llg],
                }],
            },
        },
    )
    .unwrap();
    let expected_farvi_occurrences = fixture_farvi_occurrences(&upstream);
    assert_eq!(expected_farvi_occurrences.len(), 6);
    let mut historical_engine = WorkbenchSearchEngine::open(&historical_output).unwrap();
    let first_history_page = historical_engine
        .execute(DictionaryRequest::Exact {
            spelling: "farvi".into(),
            options: DictionaryExactOptions {
                page_size: Some(1),
                ..DictionaryExactOptions::default()
            },
        })
        .unwrap();
    let historical_binding = first_history_page.binding_sha256.clone();
    let WorkbenchFullResult::Exact(first_history_page) = first_history_page.result else {
        panic!("the historical cursor source must retain the exact result tag");
    };
    let history_cursor = first_history_page
        .next_cursor
        .clone()
        .expect("the six input-derived farvi rows issue a next cursor");
    assert_eq!(
        first_history_page.groups[0].hits[0].assertion.occurrence,
        expected_farvi_occurrences[0]
    );
    let first_history_occurrence = first_history_page.groups[0].hits[0]
        .assertion
        .occurrence
        .clone();
    drop(historical_engine);

    let mut r2_document = owned_revision_one(&genesis).document();
    let prior_owned_revision = r2_document.metadata.owned_revisions[1].clone();
    let r2 = EntityId::new("workbench-r2").unwrap();
    r2_document.metadata.catalogue_revision = 2;
    r2_document
        .metadata
        .owned_revisions
        .push(OwnedRevisionRecord {
            id: r2.clone(),
            source: prior_owned_revision.source,
            parent: Some(prior_owned_revision.id),
            editor: prior_owned_revision.editor,
            timestamp: Presence::Missing,
            time_precision: TimePrecision::Unknown,
            assertions: Vec::new(),
            tombstones: Vec::new(),
            rationale: Presence::Value(TextValue {
                text: "same-active revision for historical cursor continuation".into(),
                language: Presence::Missing,
                markup: Presence::Missing,
            }),
        });
    r2_document.metadata.owned_heads[0].revision = r2.clone();
    let r2_dictionary = Dictionary::from_document(r2_document).unwrap();
    let mut r2_data = curation.clone().into_data();
    r2_data.dictionary = r2_dictionary;
    let retained_active = r2_data.revisions.last().unwrap().active.clone();
    r2_data.revisions.push(OwnedCurationRevision {
        revision: r2.clone(),
        active: retained_active,
        tombstones: Vec::new(),
    });
    let r2_curation = OwnedCurationDocument::from_data(r2_data).unwrap();
    let mut writer = OwnedJournal::open(&journal).unwrap();
    writer.commit_curated(&r1, &r2_curation).unwrap();
    drop(writer);

    let mut historical_engine = WorkbenchSearchEngine::open(&historical_output).unwrap();
    let continued_history = historical_engine
        .execute(DictionaryRequest::Exact {
            spelling: "farvi".into(),
            options: DictionaryExactOptions {
                page_size: Some(1),
                cursor: Some(history_cursor),
                ..DictionaryExactOptions::default()
            },
        })
        .unwrap();
    assert_eq!(continued_history.binding_sha256, historical_binding);
    assert_eq!(
        continued_history.selection.owned.as_ref().unwrap().revision,
        "workbench-r1"
    );
    let WorkbenchFullResult::Exact(continued_history) = continued_history.result else {
        panic!("the historical cursor continuation must retain the exact result tag");
    };
    assert_eq!(continued_history.groups[0].hits.len(), 1);
    let next_expected_occurrence = &expected_farvi_occurrences[1];
    let next_expected_assertion = upstream
        .metadata()
        .assertions
        .iter()
        .find(|row| row.occurrence == *next_expected_occurrence)
        .unwrap();
    let next_expected_entry = upstream
        .metadata()
        .entries
        .iter()
        .find(|row| row.id == next_expected_assertion.entry)
        .unwrap();
    assert_eq!(
        continued_history.groups[0].hits[0].assertion,
        *next_expected_assertion
    );
    assert_eq!(
        continued_history.groups[0].hits[0].entry,
        *next_expected_entry
    );
    assert_ne!(
        continued_history.groups[0].hits[0].assertion.occurrence, first_history_occurrence,
        "the retained cursor advances to the next row of the same historical query",
    );
    drop(historical_engine);
    fs::remove_dir_all(parent).unwrap();
}

#[test]
fn test_detail_requires_valid_matching_outer_binding() {
    let parent = fresh_root("detail-key");
    let catalogue = parent.join("catalogue");
    install(&catalogue, &upstream_dictionary());
    let output = parent.join("view");
    let report = WorkbenchView::build_new(
        &output,
        WorkbenchBuildRequest {
            catalogue_root: Some(catalogue),
            owned_root: None,
            selection: selection(),
        },
    )
    .unwrap();
    let mut engine = WorkbenchSearchEngine::open(&output).unwrap();
    let reference = vlajmaji_search::DictionaryHitDetailRef {
        version: 1,
        digest_policy: "entry-assertion-serde-json-v1".into(),
        role: "assertion".into(),
        source: SourceId::new("source-a").unwrap(),
        entry_id: EntityId::new("entry-a").unwrap(),
        assertion_id: EntityId::new("assertion-a").unwrap(),
        occurrence: OccurrenceRef {
            capture: CaptureRef {
                namespace: "fixture:workbench".into(),
                sha256: "0".repeat(64),
            },
            locator: "/0".into(),
        },
        entry_sha256: "1".repeat(64),
        assertion_sha256: "2".repeat(64),
        native_entry_id: None,
        native_assertion_id: None,
        contributor_id: None,
        distributor_id: None,
    };
    assert!(matches!(
        engine.detail(&reference, "not-a-digest"),
        Err(WorkbenchQueryError::MalformedBinding { .. })
    ));
    assert!(matches!(
        engine.detail(&reference, &"0".repeat(64)),
        Err(WorkbenchQueryError::StaleBinding { .. })
    ));
    assert_eq!(report.binding.binding_sha256().len(), 64);
    fs::remove_dir_all(parent).unwrap();
}

#[test]
fn test_execute_head_becomes_stale_after_writer_commit_but_history_stays_pinned() {
    let parent = fresh_root("owned-head-refresh");
    let journal_root = parent.join("journal");
    let genesis = owned_genesis();
    let (writer, created) = OwnedJournal::create(&journal_root, &genesis).unwrap();
    assert_eq!(created.head.revision.id.as_str(), "workbench-r0");
    drop(writer);

    let head_root = parent.join("head-view");
    let head_report = WorkbenchView::build_new(
        &head_root,
        WorkbenchBuildRequest {
            catalogue_root: None,
            owned_root: Some(journal_root.clone()),
            selection: WorkbenchSelection {
                version: 1,
                packs: Vec::new(),
                disabled: Vec::new(),
                owned: Some(WorkbenchOwnedRevision::Head),
                contexts: Vec::new(),
            },
        },
    )
    .unwrap();
    let historical_root = parent.join("historical-view");
    WorkbenchView::build_new(
        &historical_root,
        WorkbenchBuildRequest {
            catalogue_root: None,
            owned_root: Some(journal_root.clone()),
            selection: WorkbenchSelection {
                version: 1,
                packs: Vec::new(),
                disabled: Vec::new(),
                owned: Some(WorkbenchOwnedRevision::Revision(
                    EntityId::new("workbench-r0").unwrap(),
                )),
                contexts: Vec::new(),
            },
        },
    )
    .unwrap();
    let mut head_engine = WorkbenchSearchEngine::open(&head_root).unwrap();
    let mut historical_engine = WorkbenchSearchEngine::open(&historical_root).unwrap();

    let mut writer = OwnedJournal::open(&journal_root).unwrap();
    writer
        .commit(
            &EntityId::new("workbench-r0").unwrap(),
            &owned_revision_one(&genesis),
        )
        .unwrap();

    let request = DictionaryRequest::Exact {
        spelling: "absent".into(),
        options: DictionaryExactOptions::default(),
    };
    assert!(matches!(
        head_engine.execute(request.clone()),
        Err(WorkbenchQueryError::View(error))
            if matches!(error.as_ref(), WorkbenchError::StaleDependencies {
                dependency: vlajmaji_store::WorkbenchDependency::Owned,
                ..
            })
    ));
    let historical = historical_engine.execute(request).unwrap();
    assert_eq!(
        historical.binding_sha256,
        historical_engine.binding().binding_sha256()
    );
    assert_eq!(
        historical.selection.owned.as_ref().unwrap().revision,
        "workbench-r0"
    );
    assert_eq!(
        head_report
            .binding
            .owned()
            .unwrap()
            .identity
            .revision
            .as_str(),
        "workbench-r0"
    );
    fs::remove_dir_all(parent).unwrap();
}
