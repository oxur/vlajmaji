//! Public compact dictionary projection contracts.

#[path = "../../lexicon/examples/support/dictionary_fixture.rs"]
mod fixture;

use std::{
    collections::BTreeMap,
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use serde::Serialize;
use sha2::{Digest, Sha256};
use vlajmaji_lexicon::{
    AliasScope, Dictionary, JbotciImportOptions, LaLojbanImportOptions, LlgImportOptions, PackId,
    PackPayloadKind, RelationTarget, SutysiskuImportOptions, import_jbotci_export,
    import_la_lojban_export, import_llg_lists, import_sutysisku_export, seal_release,
};
use vlajmaji_search::{
    CollectionCompactQueryResult, CollectionCompactSearchEngine, CollectionQueryResult,
    CollectionQuerySelection, CollectionSearchEngine, DictionaryCompactQueryResult,
    DictionaryCompactSearchEngine, DictionaryDetailReferenceInvalidity, DictionaryExactOptions,
    DictionaryProjectionError, DictionaryQueryError, DictionaryRequest, DictionarySearchEngine,
};
use vlajmaji_store::{
    CollectionCatalogue, CollectionInput, CollectionRequest, CollectionSelection, DictionaryStore,
};

static NEXT_PATH: AtomicU64 = AtomicU64::new(0);

fn scratch() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "vlajmaji-s05-compact-dictionary-{}-{nonce}-{}",
        std::process::id(),
        NEXT_PATH.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    root
}

fn inventory_add<T: Serialize>(
    inventory: &mut BTreeMap<String, BTreeMap<String, usize>>,
    category: &str,
    rows: &[T],
) {
    let family = inventory.entry(category.into()).or_default();
    for row in rows {
        let bytes = serde_json::to_vec(row).unwrap();
        *family.entry(sha256(&bytes)).or_default() += 1;
    }
}

fn dictionary_inventory(
    document: &vlajmaji_lexicon::DictionaryDocument<vlajmaji_lexicon::RegistryDocument>,
) -> BTreeMap<String, BTreeMap<String, usize>> {
    let mut result = BTreeMap::new();
    let metadata = &document.metadata;
    inventory_add(&mut result, "code_notices", &metadata.code_notices);
    inventory_add(&mut result, "collections", &metadata.collections);
    inventory_add(&mut result, "components", &metadata.components);
    inventory_add(&mut result, "releases", &metadata.releases);
    inventory_add(&mut result, "entries", &metadata.entries);
    inventory_add(&mut result, "assertions", &metadata.assertions);
    inventory_add(&mut result, "contexts", &metadata.contexts);
    inventory_add(&mut result, "frames", &metadata.frames);
    inventory_add(&mut result, "forms", &metadata.forms);
    inventory_add(&mut result, "etymologies", &metadata.etymologies);
    inventory_add(&mut result, "relations", &metadata.relations);
    inventory_add(&mut result, "owned_revisions", &metadata.owned_revisions);
    inventory_add(&mut result, "owned_heads", &metadata.owned_heads);
    inventory_add(&mut result, "editorial_senses", &metadata.editorial_senses);
    inventory_add(&mut result, "computations", &metadata.computations);
    inventory_add(&mut result, "annotations", &metadata.annotations);
    inventory_add(&mut result, "extensions", &metadata.extensions);
    inventory_add(&mut result, "captures", &document.captures);
    let registry = &document.registry;
    inventory_add(&mut result, "sources", &registry.sources);
    inventory_add(&mut result, "contributors", &registry.contributors);
    inventory_add(&mut result, "distributors", &registry.distributors);
    inventory_add(&mut result, "aliases", &registry.aliases);
    inventory_add(&mut result, "lineage", &registry.lineage);
    inventory_add(&mut result, "snapshots", &registry.snapshots);
    inventory_add(&mut result, "occurrences", &registry.occurrences);
    result
}

fn unverified_document(
    dictionary: &Dictionary,
) -> vlajmaji_lexicon::DictionaryDocument<vlajmaji_lexicon::RegistryDocument> {
    let document = dictionary.document();
    vlajmaji_lexicon::DictionaryDocument {
        version: document.version,
        legacy_archive: document.legacy_archive.clone(),
        registry: document.registry.document(),
        captures: document.captures.clone(),
        metadata: document.metadata.clone(),
    }
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn mutate_cursor_payload_nibble(cursor: &str, payload_start: usize) -> String {
    let mut bytes = cursor.as_bytes().to_vec();
    assert!(payload_start < bytes.len());
    bytes[payload_start] = if bytes[payload_start] == b'0' {
        b'1'
    } else {
        b'0'
    };
    String::from_utf8(bytes).unwrap()
}

fn one_row_llg_dictionary() -> Dictionary {
    const GISMU_HEX: &str = "206661727669202020202020202020202020202067726f772020202020202020202020202020202020202020202020202020202020202020202020202020642020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020";
    let (pairs, remainder) = GISMU_HEX.as_bytes().as_chunks::<2>();
    assert!(remainder.is_empty());
    let gismu = pairs
        .iter()
        .map(|pair| {
            let digit = |byte: u8| (byte as char).to_digit(16).unwrap() as u8;
            (digit(pair[0]) << 4) | digit(pair[1])
        })
        .collect::<Vec<_>>();
    let cmavo = b"";
    let rafsi = b"";
    let notice = b"synthetic notice\n";
    import_llg_lists(
        &gismu,
        cmavo,
        rafsi,
        notice,
        &LlgImportOptions {
            expected_gismu_sha256: sha256(&gismu),
            expected_cmavo_sha256: sha256(cmavo),
            expected_rafsi_sha256: sha256(rafsi),
            expected_notice_sha256: sha256(notice),
            base_locator: "https://example.invalid/llg".into(),
        },
    )
    .unwrap()
    .into_dictionary()
}

#[test]
fn compact_exact_matches_full_occurrences_and_detail_roundtrips() {
    let root = scratch();
    let store = root.join("dictionary.sqlite3");
    let dictionary = fixture::complete_fixture().unwrap();
    let bytes = dictionary.to_json_bytes().unwrap();
    let full_engine = DictionarySearchEngine::build(&store, &bytes).unwrap();
    let compact_engine = DictionaryCompactSearchEngine::open(&store).unwrap();

    let full = full_engine
        .exact("farvi", DictionaryExactOptions::default())
        .unwrap();
    let compact = compact_engine
        .exact("farvi", DictionaryExactOptions::default())
        .unwrap();
    assert_eq!(compact.totals, full.totals);
    assert_eq!(compact.source_totals, full.source_totals);
    assert_eq!(
        compact.native_candidate_occurrences,
        full.native_candidate_occurrences
    );
    assert_eq!(compact.availability.len(), full.availability.len());

    for compact_hit in compact.groups.iter().flat_map(|group| &group.hits) {
        let full_hit = full
            .groups
            .iter()
            .flat_map(|group| &group.hits)
            .find(|hit| hit.occurrence_id == compact_hit.occurrence_id)
            .expect("compact hit has a full-query occurrence counterpart");
        assert_eq!(compact_hit.entry, full_hit.entry);
        assert_eq!(compact_hit.assertion, full_hit.assertion);
        assert_eq!(compact_hit.capture, full_hit.capture);
        let detail = compact_engine
            .detail(&compact_hit.detail_ref, Some(&compact.snapshot_sha256))
            .unwrap();
        assert_eq!(detail.hit.as_ref(), Some(full_hit));
        assert_eq!(detail.current_eligible, Some(true));
    }

    let mut omitted_nullable: serde_json::Value =
        serde_json::to_value(&compact.groups[0].hits[0].detail_ref).unwrap();
    omitted_nullable
        .as_object_mut()
        .unwrap()
        .remove("native_entry_id");
    let omitted = serde_json::to_vec(&omitted_nullable).unwrap();
    assert!(matches!(
        vlajmaji_search::DictionaryHitDetailRef::from_json_bytes(&omitted),
        Err(DictionaryProjectionError::ReferenceJson { .. })
    ));

    let mut wrong_role: serde_json::Value =
        serde_json::to_value(&compact.groups[0].hits[0].detail_ref).unwrap();
    wrong_role["role"] = serde_json::Value::String("entry".into());
    let wrong_role = serde_json::to_vec(&wrong_role).unwrap();
    assert!(matches!(
        vlajmaji_search::DictionaryHitDetailRef::from_json_bytes(&wrong_role),
        Err(DictionaryProjectionError::InvalidDetailReference {
            reason: DictionaryDetailReferenceInvalidity::WrongRole
        })
    ));

    let first_page = compact_engine
        .exact(
            "farvi",
            DictionaryExactOptions {
                page_size: Some(1),
                ..Default::default()
            },
        )
        .unwrap();
    let cursor = first_page.next_cursor.clone().unwrap();
    let malformed_foreign_compact =
        mutate_cursor_payload_nibble(&cursor, "dictionary-compact-v1:".len());
    assert!(matches!(
        full_engine.exact(
            "farvi",
            DictionaryExactOptions {
                page_size: Some(1),
                cursor: Some(malformed_foreign_compact),
                ..Default::default()
            }
        ),
        Err(DictionaryQueryError::MalformedCursor { .. })
    ));
    let next_page = compact_engine
        .exact(
            "farvi",
            DictionaryExactOptions {
                page_size: Some(1),
                cursor: Some(cursor.clone()),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(next_page.totals.occurrences, full.totals.occurrences);
    let reopened_compact = DictionaryCompactSearchEngine::open(&store).unwrap();
    let reopened_page = reopened_compact
        .exact(
            "farvi",
            DictionaryExactOptions {
                page_size: Some(1),
                cursor: Some(cursor.clone()),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(reopened_page, next_page);
    assert!(matches!(
        compact_engine.exact(
            "sie",
            DictionaryExactOptions {
                page_size: Some(1),
                cursor: Some(cursor),
                ..Default::default()
            }
        ),
        Err(DictionaryQueryError::StaleCursor)
    ));
    let full_cursor = full_engine
        .exact(
            "farvi",
            DictionaryExactOptions {
                page_size: Some(1),
                ..Default::default()
            },
        )
        .unwrap()
        .next_cursor
        .unwrap();
    let malformed_foreign_full = mutate_cursor_payload_nibble(&full_cursor, 0);
    assert!(matches!(
        compact_engine.exact(
            "farvi",
            DictionaryExactOptions {
                page_size: Some(1),
                cursor: Some(full_cursor),
                ..Default::default()
            }
        ),
        Err(DictionaryQueryError::StaleCursor)
    ));
    assert!(matches!(
        compact_engine.exact(
            "farvi",
            DictionaryExactOptions {
                page_size: Some(1),
                cursor: Some(malformed_foreign_full),
                ..Default::default()
            }
        ),
        Err(DictionaryQueryError::MalformedCursor { .. })
    ));
    assert!(matches!(
        full_engine.exact(
            "farvi",
            DictionaryExactOptions {
                page_size: Some(1),
                cursor: Some("dictionary-compact-v1:not-hex".into()),
                ..Default::default()
            }
        ),
        Err(DictionaryQueryError::MalformedCursor { .. })
    ));
    assert!(matches!(
        compact_engine.exact(
            "farvi",
            DictionaryExactOptions {
                page_size: Some(1),
                cursor: Some("not-a-full-cursor".into()),
                ..Default::default()
            }
        ),
        Err(DictionaryQueryError::MalformedCursor { .. })
    ));
    assert!(matches!(
        compact_engine.exact(
            "farvi",
            DictionaryExactOptions {
                page_size: Some(1),
                cursor: Some("collection-v1:garbage".into()),
                ..Default::default()
            }
        ),
        Err(DictionaryQueryError::MalformedCursor { .. })
    ));
    assert!(matches!(
        compact_engine.exact(
            "farvi",
            DictionaryExactOptions {
                page_size: Some(1),
                cursor: Some(format!("dictionary-compact-v1:aéx{}", "0".repeat(64))),
                ..DictionaryExactOptions::default()
            }
        ),
        Err(DictionaryQueryError::MalformedCursor { .. })
    ));
    let compact_hex = first_page
        .next_cursor
        .as_deref()
        .unwrap()
        .strip_prefix("dictionary-compact-v1:")
        .unwrap();
    let uppercase_index = compact_hex
        .char_indices()
        .find_map(|(index, value)| ('a'..='f').contains(&value).then_some(index))
        .unwrap();
    let mut uppercase_hex = compact_hex.to_owned();
    uppercase_hex.replace_range(
        uppercase_index..uppercase_index + 1,
        &compact_hex[uppercase_index..uppercase_index + 1].to_uppercase(),
    );
    let uppercase_body = format!("dictionary-compact-v1:{uppercase_hex}");
    assert!(matches!(
        compact_engine.exact(
            "farvi",
            DictionaryExactOptions {
                page_size: Some(1),
                cursor: Some(uppercase_body),
                ..DictionaryExactOptions::default()
            }
        ),
        Err(DictionaryQueryError::MalformedCursor { .. })
    ));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn test_resolve_profiled_local_projection_positive_and_balanced() {
    let root = scratch();
    let store = root.join("dictionary.sqlite3");
    let dictionary = fixture::complete_fixture().unwrap();
    let entry = dictionary.document().metadata.entries[0].id.clone();
    let bytes = dictionary.to_json_bytes().unwrap();
    let full_engine = DictionarySearchEngine::build(&store, &bytes).unwrap();
    let compact_engine = DictionaryCompactSearchEngine::open(&store).unwrap();
    let target = RelationTarget::Local(vlajmaji_lexicon::LocalRef::Entry(entry));

    let (full, full_profile) = full_engine.resolve_profiled(&target).unwrap();
    let (compact_result, compact_profile) = compact_engine
        .execute_profiled(DictionaryRequest::Resolve {
            target: target.clone(),
        })
        .unwrap();
    let DictionaryCompactQueryResult::Resolve(compact) = compact_result else {
        panic!("compact engine returned an exact response for a reference target");
    };
    assert_eq!(full.status, vlajmaji_search::ReferenceStatus::Resolved);
    assert_eq!(compact.status, vlajmaji_search::ReferenceStatus::Resolved);
    assert!(full_profile.projection > std::time::Duration::ZERO);
    assert!(compact_profile.projection > std::time::Duration::ZERO);
    assert_eq!(
        full_profile.total,
        full_profile.lookup + full_profile.projection
    );
    assert_eq!(
        compact_profile.total,
        compact_profile.lookup + compact_profile.projection
    );
    assert_eq!(full_profile.candidate_sql, std::time::Duration::ZERO);
    assert_eq!(compact_profile.candidate_sql, std::time::Duration::ZERO);

    let document = dictionary.document();
    let (catalog_release, catalog_assertion) = document
        .metadata
        .releases
        .iter()
        .filter(|release| {
            release.selected && release.availability == vlajmaji_lexicon::Availability::Available
        })
        .find_map(|release| {
            document
                .metadata
                .assertions
                .iter()
                .find(|assertion| release.assertions.contains(&assertion.id))
                .map(|assertion| (release, assertion))
        })
        .expect("fixture has a selected available release with an assertion");
    let catalog_entry = document
        .metadata
        .entries
        .iter()
        .find(|entry| entry.id == catalog_assertion.entry)
        .expect("catalogue assertion has its Entry");
    let catalog_target = RelationTarget::Catalog(vlajmaji_lexicon::CatalogTarget {
        source: catalog_entry.source.clone(),
        release: catalog_release.id.clone(),
        native_entry_id: None,
        native_assertion_id: None,
        occurrence: Some(catalog_assertion.occurrence.clone()),
    });
    let (full_catalog, full_catalog_profile) =
        full_engine.resolve_profiled(&catalog_target).unwrap();
    let (compact_catalog, compact_catalog_profile) = compact_engine
        .execute_profiled(DictionaryRequest::Resolve {
            target: catalog_target,
        })
        .unwrap();
    let DictionaryCompactQueryResult::Resolve(compact_catalog) = compact_catalog else {
        panic!("compact engine returned an exact response for a Catalog target");
    };
    assert_eq!(
        full_catalog.status,
        vlajmaji_search::ReferenceStatus::Resolved
    );
    assert_eq!(
        compact_catalog.status,
        vlajmaji_search::ReferenceStatus::Resolved
    );
    assert!(full_catalog_profile.projection > std::time::Duration::ZERO);
    assert!(compact_catalog_profile.projection > std::time::Duration::ZERO);
    assert_eq!(
        full_catalog_profile.total,
        full_catalog_profile.lookup + full_catalog_profile.projection
    );
    assert_eq!(
        compact_catalog_profile.total,
        compact_catalog_profile.lookup + compact_catalog_profile.projection
    );
    assert_eq!(
        full_catalog_profile.candidate_sql,
        std::time::Duration::ZERO
    );
    assert_eq!(
        compact_catalog_profile.candidate_sql,
        std::time::Duration::ZERO
    );
}

#[test]
fn test_full_exact_surfaces_corrupt_candidate_beyond_selected_page() {
    let root = scratch();
    let store = root.join("dictionary.sqlite3");
    let database = store.join("dictionary.sqlite3");
    let dictionary = fixture::complete_fixture().unwrap();
    let bytes = dictionary.to_json_bytes().unwrap();
    let engine = DictionarySearchEngine::build(&store, &bytes).unwrap();
    let first_page = engine
        .exact(
            "farvi",
            DictionaryExactOptions {
                page_size: Some(1),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(
        first_page
            .groups
            .iter()
            .map(|g| g.hits.len())
            .sum::<usize>(),
        1
    );
    assert!(first_page.totals.occurrences > 1);
    drop(engine);

    let mutation = Command::new("sqlite3")
        .arg(&database)
        .arg(
            "UPDATE assertions SET id='zz-corrupt-sql-assertion' WHERE id=(SELECT a.id FROM assertions AS a JOIN entries AS e ON e.id=a.entry_id WHERE e.spelling_value='farvi' AND a.eligibility='Lexical' ORDER BY e.source_id COLLATE BINARY DESC, a.occurrence_id COLLATE BINARY DESC LIMIT 1); SELECT changes();",
        )
        .output()
        .unwrap();
    assert!(
        mutation.status.success(),
        "sqlite candidate mutation failed: {}",
        String::from_utf8_lossy(&mutation.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&mutation.stdout).trim(), "1");

    let engine = DictionarySearchEngine::open(&store).unwrap();
    let error = engine.exact(
        "farvi",
        DictionaryExactOptions {
            page_size: Some(1),
            ..Default::default()
        },
    );
    assert!(
        matches!(error, Err(DictionaryQueryError::Integrity { .. })),
        "a corrupt SQL candidate beyond page one must retain eager integrity failure: {error:?}"
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn test_full_exact_surfaces_missing_raw_occurrence_beyond_selected_page() {
    let root = scratch();
    let store = root.join("dictionary.sqlite3");
    let database = store.join("dictionary.sqlite3");
    let dictionary = fixture::complete_fixture().unwrap();
    let bytes = dictionary.to_json_bytes().unwrap();
    let engine = DictionarySearchEngine::build(&store, &bytes).unwrap();
    let complete = engine
        .exact("farvi", DictionaryExactOptions::default())
        .unwrap();
    let last_hit = complete
        .groups
        .iter()
        .flat_map(|group| &group.hits)
        .last()
        .expect("fixture has a last farvi hit");
    let missing_capture = last_hit.assertion.occurrence.capture.clone();
    let missing_locator = last_hit.assertion.occurrence.locator.clone();
    let first_page = engine
        .exact(
            "farvi",
            DictionaryExactOptions {
                page_size: Some(1),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(
        first_page
            .groups
            .iter()
            .map(|group| group.hits.len())
            .sum::<usize>(),
        1
    );
    assert!(first_page.totals.occurrences > 1);
    let compact_engine = DictionaryCompactSearchEngine::open(&store).unwrap();
    let foreign_compact_cursor = compact_engine
        .exact(
            "farvi",
            DictionaryExactOptions {
                page_size: Some(1),
                ..Default::default()
            },
        )
        .unwrap()
        .next_cursor
        .expect("valid compact page has a continuation cursor");
    drop(compact_engine);
    drop(engine);
    let original_database = fs::read(&database).unwrap();

    let mut document = unverified_document(&dictionary);
    let capture = document
        .captures
        .iter_mut()
        .find(|capture| capture.identity == missing_capture)
        .expect("the last hit's capture is retained in the input");
    let before = capture.occurrences.len();
    capture
        .occurrences
        .retain(|occurrence| occurrence.locator != missing_locator);
    assert_eq!(capture.occurrences.len() + 1, before);
    let canonical = serde_json::to_vec(&document).unwrap();
    let digest = sha256(&canonical);
    let encoded = canonical
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let mutation = Command::new("sqlite3")
        .arg(&database)
        .arg(format!(
            "UPDATE canonical_document SET json=X'{encoded}' WHERE id=1; UPDATE metadata SET value='{digest}' WHERE key='canonical_sha256'; SELECT changes();"
        ))
        .output()
        .unwrap();
    assert!(
        mutation.status.success(),
        "raw occurrence mutation failed: {}",
        String::from_utf8_lossy(&mutation.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&mutation.stdout).trim(), "1");

    let engine = DictionarySearchEngine::open(&store).unwrap();
    let error = engine.exact(
        "farvi",
        DictionaryExactOptions {
            page_size: Some(1),
            ..Default::default()
        },
    );
    assert!(
        matches!(error, Err(DictionaryQueryError::Integrity { .. })),
        "a missing raw occurrence beyond page one must retain eager hydration failure: {error:?}"
    );
    let page_zero = engine.exact(
        "farvi",
        DictionaryExactOptions {
            page_size: Some(0),
            ..Default::default()
        },
    );
    assert!(
        matches!(page_zero, Err(DictionaryQueryError::Integrity { .. })),
        "page zero must retain full eager raw-hydration failure: {page_zero:?}"
    );
    let foreign_cursor = engine.exact(
        "farvi",
        DictionaryExactOptions {
            page_size: Some(1),
            cursor: Some(foreign_compact_cursor.clone()),
            ..Default::default()
        },
    );
    assert!(
        matches!(foreign_cursor, Err(DictionaryQueryError::Integrity { .. })),
        "eager full hydration error precedes foreign-cursor classification: {foreign_cursor:?}"
    );
    drop(engine);
    fs::write(&database, &original_database).unwrap();
    let restored = DictionarySearchEngine::open(&store).unwrap();
    assert!(
        restored
            .exact(
                "farvi",
                DictionaryExactOptions {
                    page_size: Some(0),
                    ..Default::default()
                },
            )
            .is_ok()
    );
    assert!(matches!(
        restored.exact(
            "farvi",
            DictionaryExactOptions {
                page_size: Some(1),
                cursor: Some(foreign_compact_cursor),
                ..Default::default()
            }
        ),
        Err(DictionaryQueryError::StaleCursor)
    ));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn test_full_exact_validates_unselected_sql_candidates_before_source_filtering() {
    let root = scratch();
    let store = root.join("dictionary.sqlite3");
    let database = store.join("dictionary.sqlite3");
    let dictionary = fixture::complete_fixture().unwrap();
    let bytes = dictionary.to_json_bytes().unwrap();
    let engine = DictionarySearchEngine::build(&store, &bytes).unwrap();
    let selected = engine
        .exact(
            "farvi",
            DictionaryExactOptions {
                sources: vec!["source-a".into()],
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(selected.resolved_sources.len(), 1);
    assert_eq!(selected.resolved_sources[0].as_str(), "source-a");
    assert!(selected.totals.occurrences > 0);
    drop(engine);

    let mutation = Command::new("sqlite3")
        .arg(&database)
        .arg(
            "UPDATE assertions SET id='zz-unselected-sql-assertion' WHERE id=(SELECT a.id FROM assertions AS a JOIN entries AS e ON e.id=a.entry_id WHERE e.spelling_value='farvi' AND a.eligibility='Lexical' AND e.source_id!='source-a' ORDER BY e.source_id COLLATE BINARY, a.occurrence_id COLLATE BINARY LIMIT 1); SELECT changes();",
        )
        .output()
        .unwrap();
    assert!(
        mutation.status.success(),
        "sqlite unselected-candidate mutation failed: {}",
        String::from_utf8_lossy(&mutation.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&mutation.stdout).trim(), "1");

    let engine = DictionarySearchEngine::open(&store).unwrap();
    let result = engine.exact(
        "farvi",
        DictionaryExactOptions {
            sources: vec!["source-a".into()],
            ..Default::default()
        },
    );
    assert!(
        matches!(result, Err(DictionaryQueryError::Integrity { .. })),
        "an invalid SQL candidate from an unselected source must be checked before source filtering: {result:?}"
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn test_selected_missing_entry_fails_but_unselected_candidate_is_skipped_after_lookup() {
    let root = scratch();
    let store = root.join("dictionary.sqlite3");
    let database = store.join("dictionary.sqlite3");
    let dictionary = fixture::complete_fixture().unwrap();
    DictionaryStore::create(&store, &dictionary).unwrap();
    let original_database = fs::read(&database).unwrap();

    let mut document = unverified_document(&dictionary);
    let missing = document
        .metadata
        .assertions
        .iter()
        .find_map(|assertion| {
            let entry = document.metadata.entries.iter().find(|entry| {
                entry.id == assertion.entry
                    && entry.source.as_str() == "source-b"
                    && matches!(
                        &entry.spelling,
                        vlajmaji_lexicon::Presence::Value(spelling) if spelling == "farvi"
                    )
            })?;
            Some((entry.id.clone(), assertion.id.clone()))
        })
        .expect("source-b has a farvi Entry and Assertion in the pre-input fixture");
    document
        .metadata
        .entries
        .retain(|entry| entry.id != missing.0);
    let canonical = serde_json::to_vec(&document).unwrap();
    let digest = sha256(&canonical);
    let encoded = canonical
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let mutation = Command::new("sqlite3")
        .arg(&database)
        .arg(format!(
            "UPDATE canonical_document SET json=X'{encoded}' WHERE id=1; UPDATE metadata SET value='{digest}' WHERE key='canonical_sha256'; SELECT changes();"
        ))
        .output()
        .unwrap();
    assert!(
        mutation.status.success(),
        "canonical Entry removal failed: {}",
        String::from_utf8_lossy(&mutation.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&mutation.stdout).trim(), "1");

    let full = DictionarySearchEngine::open(&store).unwrap();
    let compact = DictionaryCompactSearchEngine::open(&store).unwrap();
    {
        let sources = vec!["source-a".to_owned()];
        let full_result = full
            .exact(
                "farvi",
                DictionaryExactOptions {
                    sources: sources.clone(),
                    ..Default::default()
                },
            )
            .unwrap();
        assert!(
            full_result
                .groups
                .iter()
                .flat_map(|group| &group.hits)
                .all(|hit| hit.assertion.id != missing.1)
        );
        let compact_result = compact
            .exact(
                "farvi",
                DictionaryExactOptions {
                    sources,
                    ..Default::default()
                },
            )
            .unwrap();
        assert!(
            compact_result
                .groups
                .iter()
                .flat_map(|group| &group.hits)
                .all(|hit| hit.assertion.id != missing.1)
        );
    }
    drop(full);
    drop(compact);

    let full = DictionarySearchEngine::open(&store).unwrap();
    let compact = DictionaryCompactSearchEngine::open(&store).unwrap();
    assert!(matches!(
        full.exact(
            "farvi",
            DictionaryExactOptions {
                sources: vec!["source-b".into()],
                ..Default::default()
            }
        ),
        Err(DictionaryQueryError::Integrity { .. })
    ));
    assert!(matches!(
        compact.exact(
            "farvi",
            DictionaryExactOptions {
                sources: vec!["source-b".into()],
                ..Default::default()
            }
        ),
        Err(DictionaryQueryError::Integrity { .. })
    ));
    drop(full);
    drop(compact);

    let mutation = Command::new("sqlite3")
        .arg(&database)
        .arg(format!(
            "UPDATE assertions SET id='zz-unknown-unselected-sql-id' WHERE id='{}'; SELECT changes();",
            missing.1
        ))
        .output()
        .unwrap();
    assert!(
        mutation.status.success(),
        "unknown SQL candidate mutation failed: {}",
        String::from_utf8_lossy(&mutation.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&mutation.stdout).trim(), "1");
    let full = DictionarySearchEngine::open(&store).unwrap();
    let compact = DictionaryCompactSearchEngine::open(&store).unwrap();
    assert!(matches!(
        full.exact(
            "farvi",
            DictionaryExactOptions {
                sources: vec!["source-a".into()],
                ..Default::default()
            }
        ),
        Err(DictionaryQueryError::Integrity { .. })
    ));
    assert!(matches!(
        compact.exact(
            "farvi",
            DictionaryExactOptions {
                sources: vec!["source-a".into()],
                ..Default::default()
            }
        ),
        Err(DictionaryQueryError::Integrity { .. })
    ));
    drop(full);
    drop(compact);

    fs::write(&database, original_database).unwrap();
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn test_compact_exact_preserves_alias_and_owned_head_overlay_selection() {
    let root = scratch();
    let store = root.join("dictionary.sqlite3");
    let mut document = fixture::complete_fixture().unwrap().into_document();
    let head = document
        .metadata
        .owned_heads
        .iter()
        .find(|head| head.source.as_str() == "source-o")
        .unwrap()
        .revision
        .clone();
    assert_eq!(head.as_str(), "revision-r1");
    let revision_one_assertions = document
        .metadata
        .owned_revisions
        .iter()
        .find(|revision| revision.id.as_str() == head.as_str())
        .unwrap()
        .assertions
        .iter()
        .map(ToString::to_string)
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(revision_one_assertions.len(), 1);

    // The selected release exposes r2 while the independently recorded head
    // remains r1. Their intersection is empty; ignoring the head overlay would
    // incorrectly return r2's lexical row.
    for release in &mut document.metadata.releases {
        if release.id.as_str() == "release-o1" {
            release.selected = false;
        } else if release.id.as_str() == "release-o2" {
            release.selected = true;
        }
    }
    let dictionary = Dictionary::from_document(document.clone()).unwrap();
    let bytes = dictionary.to_json_bytes().unwrap();
    let full = DictionarySearchEngine::build(&store, &bytes).unwrap();
    let compact = DictionaryCompactSearchEngine::open(&store).unwrap();

    let active_alias = DictionaryExactOptions {
        sources: vec!["archive-a".into()],
        ..Default::default()
    };
    let full_alias = full.exact("farvi", active_alias.clone()).unwrap();
    let compact_alias = compact.exact("farvi", active_alias).unwrap();
    assert_eq!(full_alias.resolved_sources.len(), 1);
    assert_eq!(full_alias.resolved_sources[0].as_str(), "source-a");
    assert_eq!(
        full_alias
            .groups
            .iter()
            .flat_map(|group| &group.hits)
            .map(|hit| hit.assertion.id.to_string())
            .collect::<Vec<_>>(),
        compact_alias
            .groups
            .iter()
            .flat_map(|group| &group.hits)
            .map(|hit| hit.assertion.id.to_string())
            .collect::<Vec<_>>()
    );
    let retired_active = DictionaryExactOptions {
        sources: vec!["legacy-a".into()],
        alias_scope: AliasScope::ActiveOnly,
        ..Default::default()
    };
    assert!(matches!(
        full.exact("farvi", retired_active.clone()),
        Err(DictionaryQueryError::Registry(
            vlajmaji_lexicon::RegistryError::UnknownSource(_)
        ))
    ));
    assert!(matches!(
        compact.exact("farvi", retired_active),
        Err(DictionaryQueryError::Registry(
            vlajmaji_lexicon::RegistryError::UnknownSource(_)
        ))
    ));
    let retired_included = DictionaryExactOptions {
        sources: vec!["legacy-a".into()],
        alias_scope: AliasScope::IncludeRetired,
        ..Default::default()
    };
    let full_retired = full.exact("farvi", retired_included.clone()).unwrap();
    let compact_retired = compact.exact("farvi", retired_included).unwrap();
    assert_eq!(full_retired.totals, compact_retired.totals);
    assert_eq!(full_retired.source_totals, compact_retired.source_totals);

    let owned = DictionaryExactOptions {
        sources: vec!["source-o".into()],
        ..Default::default()
    };
    let full_owned = full.exact("farvi", owned.clone()).unwrap();
    let compact_owned = compact.exact("farvi", owned).unwrap();
    let expected_release = document
        .metadata
        .releases
        .iter()
        .find(|release| release.id.as_str() == "release-o2")
        .unwrap();
    assert!(expected_release.selected);
    let expected_owned_hits = expected_release
        .assertions
        .iter()
        .map(ToString::to_string)
        .filter(|assertion| revision_one_assertions.contains(assertion))
        .collect::<std::collections::BTreeSet<_>>();
    assert!(expected_owned_hits.is_empty());
    assert_eq!(full_owned.totals.occurrences, 0);
    assert_eq!(compact_owned.totals.occurrences, 0);
    assert_eq!(full_owned.resolved_sources.len(), 1);
    assert_eq!(compact_owned.resolved_sources.len(), 1);
    assert_eq!(
        full_owned
            .availability
            .iter()
            .find(|row| row.source.id.as_str() == "source-o")
            .unwrap()
            .owned_head
            .as_ref()
            .unwrap()
            .revision,
        head
    );
    assert_eq!(
        compact_owned
            .availability
            .iter()
            .find(|row| row.source.id.as_str() == "source-o")
            .unwrap()
            .owned_head
            .as_ref()
            .unwrap()
            .revision,
        document.metadata.owned_heads[0].revision
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn compact_detail_snapshot_tracks_legitimate_attached_context_changes() {
    let base = scratch();
    let before_root = base.join("before.sqlite3");
    let after_root = base.join("after.sqlite3");

    let before_dictionary = fixture::complete_fixture().unwrap();
    let mut changed_document = before_dictionary.clone().into_document();
    let attached_context = changed_document
        .metadata
        .contexts
        .first_mut()
        .expect("fixture has an attached context");
    let context_id = attached_context.id.clone();
    let assertion_id = attached_context.assertion.clone();
    let vlajmaji_lexicon::Presence::Value(value) = &mut attached_context.value else {
        panic!("fixture context has a present value");
    };
    value.text = "changed but valid context".into();
    let assertion = changed_document
        .metadata
        .assertions
        .iter()
        .find(|assertion| assertion.id == assertion_id)
        .unwrap();
    let entry = changed_document
        .metadata
        .entries
        .iter()
        .find(|entry| entry.id == assertion.entry)
        .unwrap();
    let vlajmaji_lexicon::Presence::Value(spelling) = &entry.spelling else {
        panic!("fixture assertion has a lexical spelling");
    };
    let spelling = spelling.clone();
    let after_dictionary = Dictionary::from_document(changed_document).unwrap();

    DictionaryStore::create(&before_root, &before_dictionary).unwrap();
    DictionaryStore::create(&after_root, &after_dictionary).unwrap();
    let before = DictionaryCompactSearchEngine::open(&before_root).unwrap();
    let after = DictionaryCompactSearchEngine::open(&after_root).unwrap();
    let before_result = before
        .exact(&spelling, DictionaryExactOptions::default())
        .unwrap();
    let after_result = after
        .exact(&spelling, DictionaryExactOptions::default())
        .unwrap();
    let before_hit = &before_result.groups[0].hits[0];
    let after_hit = &after_result.groups[0].hits[0];

    assert_eq!(before_hit.entry.id, after_hit.entry.id);
    assert_eq!(before_hit.assertion.id, after_hit.assertion.id);
    assert_eq!(before_hit.occurrence_id, after_hit.occurrence_id);
    assert_ne!(before_result.snapshot_sha256, after_result.snapshot_sha256);
    assert!(matches!(
        after.detail(&before_hit.detail_ref, Some(&before_result.snapshot_sha256)),
        Err(DictionaryProjectionError::StaleSnapshot { .. })
    ));

    let before_detail = before
        .detail(&before_hit.detail_ref, Some(&before_result.snapshot_sha256))
        .unwrap();
    let after_detail = after
        .detail(&after_hit.detail_ref, Some(&after_result.snapshot_sha256))
        .unwrap();
    let before_context = before_detail
        .hit
        .as_ref()
        .unwrap()
        .contexts
        .iter()
        .find(|context| context.id == context_id)
        .unwrap();
    let after_context = after_detail
        .hit
        .as_ref()
        .unwrap()
        .contexts
        .iter()
        .find(|context| context.id == context_id)
        .unwrap();
    let vlajmaji_lexicon::Presence::Value(before_value) = &before_context.value else {
        panic!("original context is present");
    };
    let vlajmaji_lexicon::Presence::Value(after_value) = &after_context.value else {
        panic!("updated context is present");
    };
    assert_ne!(before_value.text, after_value.text);
    assert_eq!(after_value.text, "changed but valid context");
    fs::remove_dir_all(base).unwrap();
}

#[test]
fn test_compact_detail_changed_assertion_body_expect_no_retarget() {
    let base = scratch();
    let before_root = base.join("before.sqlite3");
    let after_root = base.join("after.sqlite3");

    let before_dictionary = fixture::complete_fixture().unwrap();
    let before_document = before_dictionary.document();
    let before_assertion = before_document
        .metadata
        .assertions
        .iter()
        .find(|assertion| {
            assertion.eligibility == vlajmaji_lexicon::Eligibility::Lexical
                && before_document.metadata.entries.iter().any(|entry| {
                    entry.id == assertion.entry
                        && matches!(
                            &entry.spelling,
                            vlajmaji_lexicon::Presence::Value(spelling)
                                if spelling == "farvi"
                        )
                })
        })
        .expect("complete fixture has a lexical farvi assertion")
        .clone();

    let mut after_document = before_dictionary.clone().into_document();
    let after_assertion = after_document
        .metadata
        .assertions
        .iter_mut()
        .find(|assertion| assertion.id == before_assertion.id)
        .unwrap();
    after_assertion.native_sense_id =
        vlajmaji_lexicon::Presence::Value("changed-valid-native-sense".into());
    let after_assertion = after_assertion.clone();
    let after_dictionary = Dictionary::from_document(after_document).unwrap();

    // The stable identity/occurrence stays fixed while the complete typed
    // assertion record changes in a valid newly constructed dictionary.
    assert_eq!(before_assertion.id, after_assertion.id);
    assert_eq!(before_assertion.entry, after_assertion.entry);
    assert_eq!(before_assertion.occurrence, after_assertion.occurrence);
    assert_ne!(before_assertion, after_assertion);
    let before_entry = before_document
        .metadata
        .entries
        .iter()
        .find(|entry| entry.id == before_assertion.entry)
        .unwrap();
    let after_document = after_dictionary.document();
    let after_entry = after_document
        .metadata
        .entries
        .iter()
        .find(|entry| entry.id == after_assertion.entry)
        .unwrap();
    assert_eq!(before_entry, after_entry);

    DictionaryStore::create(&before_root, &before_dictionary).unwrap();
    DictionaryStore::create(&after_root, &after_dictionary).unwrap();
    let before = DictionaryCompactSearchEngine::open(&before_root).unwrap();
    let after = DictionaryCompactSearchEngine::open(&after_root).unwrap();
    let before_result = before
        .exact("farvi", DictionaryExactOptions::default())
        .unwrap();
    let before_hit = before_result
        .groups
        .iter()
        .flat_map(|group| &group.hits)
        .find(|hit| hit.assertion.id == before_assertion.id)
        .expect("the pre-change assertion is a compact hit");
    let old_key = before_hit.detail_ref.clone();

    let after_result = after
        .exact("farvi", DictionaryExactOptions::default())
        .unwrap();
    let after_hit = after_result
        .groups
        .iter()
        .flat_map(|group| &group.hits)
        .find(|hit| hit.assertion.id == after_assertion.id)
        .expect("the changed assertion remains a compact hit");
    assert_eq!(after_hit.entry, *before_entry);
    assert_eq!(after_hit.assertion, after_assertion);
    assert_eq!(old_key.source, after_hit.detail_ref.source);
    assert_eq!(old_key.entry_id, after_hit.detail_ref.entry_id);
    assert_eq!(old_key.assertion_id, after_hit.detail_ref.assertion_id);
    assert_eq!(old_key.occurrence, after_hit.detail_ref.occurrence);
    assert_eq!(old_key.entry_sha256, after_hit.detail_ref.entry_sha256);
    assert_ne!(
        old_key.assertion_sha256,
        after_hit.detail_ref.assertion_sha256
    );

    let old_key_result = after.detail(&old_key, None).unwrap();
    assert_eq!(
        old_key_result.status,
        vlajmaji_search::ReferenceStatus::NoMatch
    );
    assert!(old_key_result.hit.is_none());
    assert!(old_key_result.current_eligible.is_none());
    let new_key_result = after.detail(&after_hit.detail_ref, None).unwrap();
    assert_eq!(
        new_key_result.status,
        vlajmaji_search::ReferenceStatus::Resolved
    );
    assert_eq!(
        new_key_result.hit.as_ref().map(|hit| &hit.assertion),
        Some(&after_assertion)
    );

    let before_managed_root = base.join("before-managed");
    let after_managed_root = base.join("after-managed");
    let mut before_managed_document = before_dictionary.clone().into_document();
    before_managed_document.legacy_archive = None;
    let before_managed_dictionary = Dictionary::from_document(before_managed_document).unwrap();
    let mut after_managed_document = after_dictionary.clone().into_document();
    after_managed_document.legacy_archive = None;
    let after_managed_dictionary = Dictionary::from_document(after_managed_document).unwrap();
    for (root, dictionary) in [
        (&before_managed_root, &before_managed_dictionary),
        (&after_managed_root, &after_managed_dictionary),
    ] {
        let sealed = seal_release(PackId::Llg, PackPayloadKind::Synthetic, dictionary).unwrap();
        let request = CollectionRequest::new(
            vec![CollectionInput::SealedBytes {
                pack_id: PackId::Llg,
                release: std::borrow::Cow::Borrowed(sealed.release.as_slice()),
                seal: std::borrow::Cow::Borrowed(sealed.seal.as_slice()),
            }],
            CollectionSelection::Packs(vec![PackId::Llg]),
            true,
        )
        .unwrap();
        CollectionCatalogue::install(root, request).unwrap();
    }
    let mut before_managed = CollectionCompactSearchEngine::open(
        &before_managed_root,
        CollectionQuerySelection {
            packs: vec![PackId::Llg],
            disabled: vec![],
            profile: None,
        },
    )
    .unwrap();
    let managed_exact = before_managed
        .execute(DictionaryRequest::Exact {
            spelling: "farvi".into(),
            options: DictionaryExactOptions::default(),
        })
        .unwrap();
    let CollectionCompactQueryResult::Exact(managed_exact) = managed_exact.result else {
        panic!("expected managed compact exact response");
    };
    let managed_old_key = managed_exact.groups[0].hits[0].detail_ref.clone();
    let mut after_managed = CollectionCompactSearchEngine::open(
        &after_managed_root,
        CollectionQuerySelection {
            packs: vec![PackId::Llg],
            disabled: vec![],
            profile: None,
        },
    )
    .unwrap();
    let managed_old = after_managed.detail(&managed_old_key, None).unwrap();
    assert_eq!(
        managed_old.response.status,
        vlajmaji_search::ReferenceStatus::NoMatch
    );
    assert!(managed_old.response.hit.is_none());
    assert!(managed_old.response.current_eligible.is_none());
    assert!(managed_old.catalogue_matches.is_empty());
    let managed_new = after_managed.detail(&after_hit.detail_ref, None).unwrap();
    assert_eq!(
        managed_new.response.status,
        vlajmaji_search::ReferenceStatus::Resolved
    );
    assert_eq!(
        managed_new.response.hit.as_ref().map(|hit| &hit.assertion),
        Some(&after_assertion)
    );

    fs::remove_dir_all(base).unwrap();
}

#[test]
fn test_compact_detail_identity_conflicts_are_typed_and_absent_values_are_no_match() {
    let root = scratch();
    let dictionary = fixture::complete_fixture().unwrap();
    DictionaryStore::create(root.join("dictionary.sqlite3"), &dictionary).unwrap();
    let engine = DictionaryCompactSearchEngine::open(root.join("dictionary.sqlite3")).unwrap();
    let exact = engine
        .exact("farvi", DictionaryExactOptions::default())
        .unwrap();
    let valid = exact.groups[0].hits[0].detail_ref.clone();
    assert_eq!(
        engine.detail(&valid, None).unwrap().status,
        vlajmaji_search::ReferenceStatus::Resolved
    );

    let invalid_reference = |result: Result<_, DictionaryProjectionError>| {
        matches!(
            result,
            Err(DictionaryProjectionError::Query(error))
                if matches!(*error, DictionaryQueryError::InvalidReference { .. })
        )
    };

    let mut absent = valid.clone();
    absent.assertion_id = vlajmaji_lexicon::EntityId::new("absent-assertion").unwrap();
    let absent = engine.detail(&absent, None).unwrap();
    assert_eq!(absent.status, vlajmaji_search::ReferenceStatus::NoMatch);
    assert!(absent.hit.is_none());
    assert!(absent.current_eligible.is_none());

    let mut unknown_digest = valid.clone();
    unknown_digest.assertion_sha256 = "f".repeat(64);
    let unknown_digest = engine.detail(&unknown_digest, None).unwrap();
    assert_eq!(
        unknown_digest.status,
        vlajmaji_search::ReferenceStatus::NoMatch
    );
    assert!(unknown_digest.hit.is_none());

    let mut wrong_source = valid.clone();
    wrong_source.source = vlajmaji_lexicon::SourceId::new("source-b").unwrap();
    assert!(invalid_reference(engine.detail(&wrong_source, None)));

    let mut wrong_entry = valid.clone();
    wrong_entry.entry_id = vlajmaji_lexicon::EntityId::new("entry-b-farvi").unwrap();
    assert!(invalid_reference(engine.detail(&wrong_entry, None)));

    let mut wrong_occurrence = valid.clone();
    wrong_occurrence.occurrence.locator.push_str("-other");
    assert!(invalid_reference(engine.detail(&wrong_occurrence, None)));

    let mut wrong_capture_owner = valid.clone();
    wrong_capture_owner
        .occurrence
        .capture
        .namespace
        .push_str("-other");
    assert!(invalid_reference(engine.detail(&wrong_capture_owner, None)));

    let mut wrong_native_role = valid.clone();
    if let Some(native_assertion) = valid.native_assertion_id.as_ref() {
        wrong_native_role.native_entry_id = Some(native_assertion.clone());
        assert!(invalid_reference(engine.detail(&wrong_native_role, None)));
    } else {
        panic!("selected fixture hit has native assertion attribution");
    }

    let mut unknown_native_value = valid;
    unknown_native_value.native_entry_id = Some("unknown-native-entry".into());
    let unknown_native_value = engine.detail(&unknown_native_value, None).unwrap();
    assert_eq!(
        unknown_native_value.status,
        vlajmaji_search::ReferenceStatus::NoMatch
    );
    assert!(unknown_native_value.hit.is_none());

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn compact_local_reference_keeps_linked_assertions_and_status() {
    let root = scratch();
    let store = root.join("dictionary.sqlite3");
    let dictionary = fixture::complete_fixture().unwrap();
    let model = dictionary.document();
    let entry_id = model.metadata.forms[0].entry.clone();
    let assertion_id = model
        .metadata
        .assertions
        .iter()
        .find(|row| row.entry == entry_id)
        .unwrap()
        .id
        .clone();
    let form_id = model.metadata.forms[0].id.clone();
    let editorial_id = model.metadata.editorial_senses[0].id.clone();
    let bytes = dictionary.to_json_bytes().unwrap();
    let full_engine = DictionarySearchEngine::build(&store, &bytes).unwrap();
    let compact_engine = DictionaryCompactSearchEngine::open(&store).unwrap();
    let targets = [
        vlajmaji_lexicon::LocalRef::Entry(entry_id),
        vlajmaji_lexicon::LocalRef::Assertion(assertion_id),
        vlajmaji_lexicon::LocalRef::Form(form_id),
        vlajmaji_lexicon::LocalRef::EditorialSense(editorial_id),
    ];
    for target in targets {
        let target = RelationTarget::Local(target);
        let full = full_engine.resolve(&target).unwrap();
        let compact = compact_engine.resolve(&target).unwrap();
        assert_eq!(compact.status, full.status);
        assert_eq!(
            compact.entity.as_ref().map(|entity| entity.role.as_str()),
            full.entity.as_ref().map(|entity| match entity {
                vlajmaji_search::ReferenceEntity::Entry(_) => "entry",
                vlajmaji_search::ReferenceEntity::Assertion(_) => "assertion",
                vlajmaji_search::ReferenceEntity::Form(_) => "form",
                vlajmaji_search::ReferenceEntity::EditorialSense(_) => "editorial_sense",
            })
        );
        assert_eq!(
            compact
                .confirmed
                .iter()
                .map(|row| row.assertion.id.to_string())
                .collect::<Vec<_>>(),
            full.confirmed
                .iter()
                .map(|row| row.assertion.id.to_string())
                .collect::<Vec<_>>()
        );
        assert!(compact.unverified.is_empty());
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn compact_managed_catalog_reference_uses_targeted_projection() {
    let base = scratch();
    let root = base.join("catalogue");
    let mut document = fixture::complete_fixture().unwrap().into_document();
    document.legacy_archive = None;
    let dictionary = vlajmaji_lexicon::Dictionary::from_document(document).unwrap();
    let model = dictionary.document();
    let release = model
        .metadata
        .releases
        .iter()
        .find(|release| release.id.as_str() == "release-a")
        .unwrap();
    let assertion = model
        .metadata
        .assertions
        .iter()
        .find(|assertion| release.assertions.contains(&assertion.id))
        .unwrap();
    let farvi_entries = model
        .metadata
        .entries
        .iter()
        .filter(|entry| {
            matches!(
                &entry.spelling,
                vlajmaji_lexicon::Presence::Value(spelling) if spelling == "farvi"
            )
        })
        .map(|entry| entry.id.clone())
        .collect::<std::collections::BTreeSet<_>>();
    let selected_assertions = model
        .metadata
        .releases
        .iter()
        .filter(|row| row.selected && row.availability == vlajmaji_lexicon::Availability::Available)
        .flat_map(|row| row.assertions.iter().cloned())
        .collect::<std::collections::BTreeSet<_>>();
    let expected_farvi = model
        .metadata
        .assertions
        .iter()
        .filter(|id| selected_assertions.contains(&id.id) && farvi_entries.contains(&id.entry))
        .count();
    let target = RelationTarget::Catalog(vlajmaji_lexicon::CatalogTarget {
        source: vlajmaji_lexicon::SourceId::new("source-a").unwrap(),
        release: release.id.clone(),
        native_entry_id: None,
        native_assertion_id: None,
        occurrence: Some(assertion.occurrence.clone()),
    });
    let sealed = vlajmaji_lexicon::seal_release(
        vlajmaji_lexicon::PackId::Llg,
        vlajmaji_lexicon::PackPayloadKind::Synthetic,
        &dictionary,
    )
    .unwrap();
    let request = CollectionRequest::new(
        vec![CollectionInput::SealedBytes {
            pack_id: vlajmaji_lexicon::PackId::Llg,
            release: std::borrow::Cow::Borrowed(sealed.release.as_slice()),
            seal: std::borrow::Cow::Borrowed(sealed.seal.as_slice()),
        }],
        CollectionSelection::Packs(vec![vlajmaji_lexicon::PackId::Llg]),
        true,
    )
    .unwrap();
    CollectionCatalogue::install(&root, request).unwrap();

    let mut engine = CollectionCompactSearchEngine::open(
        &root,
        CollectionQuerySelection {
            packs: vec![vlajmaji_lexicon::PackId::Llg],
            disabled: vec![],
            profile: None,
        },
    )
    .unwrap();
    let (profiled_exact, first_profile) = engine
        .execute_profiled(DictionaryRequest::Exact {
            spelling: "farvi".into(),
            options: DictionaryExactOptions::default(),
        })
        .unwrap();
    let CollectionCompactQueryResult::Exact(profiled_exact) = profiled_exact.result else {
        panic!("expected a managed compact Exact response");
    };
    assert_eq!(profiled_exact.totals.occurrences, expected_farvi);
    assert!(first_profile.payload_setup.is_some());
    assert!(first_profile.total >= first_profile.query.total);
    assert_eq!(
        first_profile.total - first_profile.payload_setup.unwrap().total,
        first_profile.query.total
    );
    assert_eq!(
        first_profile.query.lookup + first_profile.query.projection,
        first_profile.query.total
    );
    assert!(first_profile.query.candidate_sql <= first_profile.query.lookup);

    let response = engine
        .execute_profiled(DictionaryRequest::Resolve {
            target: target.clone(),
        })
        .unwrap();
    assert!(response.1.payload_setup.is_none());
    assert!(
        response.1.total >= response.1.query.total,
        "profile={:?}",
        response.1
    );
    assert_eq!(response.1.total, response.1.query.total);
    assert_eq!(
        response.1.query.lookup + response.1.query.projection,
        response.1.query.total
    );
    assert!(response.1.query.projection > std::time::Duration::ZERO);
    assert!(response.1.query.candidate_sql <= response.1.query.lookup);
    let response = response.0;
    let CollectionCompactQueryResult::Resolve(response) = response.result else {
        panic!("expected a managed compact Resolve response");
    };
    assert_eq!(response.status, vlajmaji_search::ReferenceStatus::Resolved);
    assert_eq!(response.catalogue_matches.len(), 1);
    assert_eq!(
        response
            .requested_release
            .as_ref()
            .map(|release| release.id.as_str()),
        Some("release-a")
    );
    assert!(response.payload.is_some());
    let compact_page = engine
        .execute(DictionaryRequest::Exact {
            spelling: "farvi".into(),
            options: DictionaryExactOptions::default(),
        })
        .unwrap();
    let CollectionCompactQueryResult::Exact(compact_page) = compact_page.result else {
        panic!("expected compact detail-key source");
    };
    let mut unknown_body = compact_page.groups[0].hits[0].detail_ref.clone();
    unknown_body.assertion_sha256 = "f".repeat(64);
    let unknown_body_result = engine.detail(&unknown_body, None).unwrap();
    assert_eq!(
        unknown_body_result.response.status,
        vlajmaji_search::ReferenceStatus::NoMatch
    );
    assert!(unknown_body_result.response.hit.is_none());
    assert_eq!(unknown_body_result.response.current_eligible, None);
    assert!(unknown_body_result.catalogue_matches.is_empty());
    let mut wrong_entry_relation = compact_page.groups[0].hits[0].detail_ref.clone();
    wrong_entry_relation.entry_id = vlajmaji_lexicon::EntityId::new("entry-b-farvi").unwrap();
    assert!(matches!(
        engine.detail(&wrong_entry_relation, None),
        Err(vlajmaji_search::CollectionQueryError::Query(
            vlajmaji_search::DictionaryQueryError::InvalidReference { .. }
        ))
    ));
    let mut invalid_reference = compact_page.groups[0].hits[0].detail_ref.clone();
    invalid_reference.role = "entry".into();
    let registered_root = base.join("registered-only");
    let registration = CollectionRequest::new(
        vec![CollectionInput::SealedBytes {
            pack_id: vlajmaji_lexicon::PackId::Llg,
            release: std::borrow::Cow::Borrowed(sealed.release.as_slice()),
            seal: std::borrow::Cow::Borrowed(sealed.seal.as_slice()),
        }],
        CollectionSelection::Packs(vec![vlajmaji_lexicon::PackId::Llg]),
        true,
    )
    .unwrap();
    CollectionCatalogue::register(&registered_root, registration).unwrap();
    let mut registered_only = CollectionCompactSearchEngine::open(
        &registered_root,
        CollectionQuerySelection {
            packs: vec![vlajmaji_lexicon::PackId::Llg],
            disabled: vec![],
            profile: None,
        },
    )
    .unwrap();
    let registered_resolution = registered_only
        .execute(DictionaryRequest::Resolve {
            target: target.clone(),
        })
        .unwrap();
    let CollectionCompactQueryResult::Resolve(registered_resolution) = registered_resolution.result
    else {
        panic!("expected registered-only compact resolution");
    };
    assert_eq!(
        registered_resolution.status,
        vlajmaji_search::ReferenceStatus::CatalogueOnly
    );
    assert!(registered_resolution.payload.is_none());
    assert_eq!(registered_resolution.catalogue_matches.len(), 1);
    assert_eq!(
        registered_resolution.operational_reason.as_deref(),
        Some("registered_only")
    );
    let registered_detail = registered_only
        .detail(&compact_page.groups[0].hits[0].detail_ref, None)
        .unwrap();
    assert_eq!(
        registered_detail.response.status,
        vlajmaji_search::ReferenceStatus::CatalogueOnly
    );
    assert_eq!(
        registered_detail.response.operational_reason.as_deref(),
        Some("registered_only")
    );
    assert!(registered_detail.response.hit.is_none());
    assert_eq!(registered_detail.catalogue_matches.len(), 1);
    let stale_snapshot = "0".repeat(64);
    assert!(matches!(
        registered_only.detail(
            &compact_page.groups[0].hits[0].detail_ref,
            Some(&stale_snapshot)
        ),
        Err(vlajmaji_search::CollectionQueryError::Projection(
            DictionaryProjectionError::StaleSnapshot { .. }
        ))
    ));
    assert!(matches!(
        registered_only.detail(&invalid_reference, None),
        Err(vlajmaji_search::CollectionQueryError::Projection(
            DictionaryProjectionError::InvalidDetailReference {
                reason: DictionaryDetailReferenceInvalidity::WrongRole
            }
        ))
    ));
    assert!(matches!(
        registered_only.detail(&compact_page.groups[0].hits[0].detail_ref, Some("bad")),
        Err(vlajmaji_search::CollectionQueryError::Projection(
            DictionaryProjectionError::MalformedSnapshot { .. }
        ))
    ));
    let mut disabled = CollectionCompactSearchEngine::open(
        &root,
        CollectionQuerySelection {
            packs: vec![vlajmaji_lexicon::PackId::Llg],
            disabled: vec![vlajmaji_lexicon::PackId::Llg],
            profile: None,
        },
    )
    .unwrap();
    let disabled_resolution = disabled
        .execute(DictionaryRequest::Resolve {
            target: target.clone(),
        })
        .unwrap();
    let CollectionCompactQueryResult::Resolve(disabled_resolution) = disabled_resolution.result
    else {
        panic!("expected disabled compact resolution");
    };
    assert_eq!(
        disabled_resolution.status,
        vlajmaji_search::ReferenceStatus::CatalogueOnly
    );
    assert!(disabled_resolution.payload.is_none());
    assert_eq!(
        disabled_resolution.operational_reason.as_deref(),
        Some("disabled")
    );
    let disabled_detail = disabled
        .detail(&compact_page.groups[0].hits[0].detail_ref, None)
        .unwrap();
    assert_eq!(
        disabled_detail.response.status,
        vlajmaji_search::ReferenceStatus::CatalogueOnly
    );
    assert_eq!(
        disabled_detail.response.operational_reason.as_deref(),
        Some("disabled")
    );
    assert!(disabled_detail.response.hit.is_none());
    let local = engine
        .execute(DictionaryRequest::Resolve {
            target: RelationTarget::Local(vlajmaji_lexicon::LocalRef::Entry(
                assertion.entry.clone(),
            )),
        })
        .unwrap();
    let CollectionCompactQueryResult::Resolve(local) = local.result else {
        panic!("expected a managed compact Local Resolve response");
    };
    assert_eq!(local.status, vlajmaji_search::ReferenceStatus::Resolved);
    assert!(local.payload.is_some());

    let mut full_engine = CollectionSearchEngine::open(
        &root,
        CollectionQuerySelection {
            packs: vec![vlajmaji_lexicon::PackId::Llg],
            disabled: vec![],
            profile: None,
        },
    )
    .unwrap();
    let (full_profiled, full_profile) = full_engine
        .execute_profiled(DictionaryRequest::Exact {
            spelling: "farvi".into(),
            options: DictionaryExactOptions::default(),
        })
        .unwrap();
    let CollectionQueryResult::Exact(full_profiled) = full_profiled.result else {
        panic!("expected a managed full Exact response");
    };
    assert_eq!(full_profiled.totals.occurrences, expected_farvi);
    assert!(full_profile.payload_setup.is_some());
    assert!(full_profile.total >= full_profile.query.total);
    assert_eq!(
        full_profile.total - full_profile.payload_setup.unwrap().total,
        full_profile.query.total
    );
    assert_eq!(
        full_profile.query.lookup + full_profile.query.projection,
        full_profile.query.total
    );
    assert!(full_profile.query.candidate_sql <= full_profile.query.lookup);

    let (profiled_resolve, resolve_profile) = full_engine
        .execute_profiled(DictionaryRequest::Resolve {
            target: target.clone(),
        })
        .unwrap();
    assert!(resolve_profile.payload_setup.is_none());
    assert_eq!(resolve_profile.total, resolve_profile.query.total);
    assert_eq!(
        resolve_profile.query.lookup + resolve_profile.query.projection,
        resolve_profile.query.total
    );
    assert!(resolve_profile.query.projection > std::time::Duration::ZERO);
    assert!(resolve_profile.query.candidate_sql <= resolve_profile.query.lookup);
    let CollectionQueryResult::Resolve(profiled_resolve) = profiled_resolve.result else {
        panic!("expected a managed full Resolve response");
    };
    assert_eq!(
        profiled_resolve.status,
        vlajmaji_search::ReferenceStatus::Resolved
    );

    let full_page_options = DictionaryExactOptions {
        page_size: Some(1),
        ..DictionaryExactOptions::default()
    };
    let full_page = full_engine
        .execute(DictionaryRequest::Exact {
            spelling: "farvi".into(),
            options: full_page_options,
        })
        .unwrap();
    let CollectionQueryResult::Exact(full_page) = full_page.result else {
        panic!("expected full managed exact response");
    };
    let foreign_full_cursor = full_page.next_cursor.unwrap();
    let malformed_foreign_full_managed =
        mutate_cursor_payload_nibble(&foreign_full_cursor, "collection-v1:".len());
    assert!(matches!(
        engine.execute(DictionaryRequest::Exact {
            spelling: "farvi".into(),
            options: DictionaryExactOptions {
                cursor: Some(foreign_full_cursor),
                ..DictionaryExactOptions::default()
            },
        }),
        Err(vlajmaji_search::CollectionQueryError::StaleCursor)
    ));
    assert!(matches!(
        engine.execute(DictionaryRequest::Exact {
            spelling: "farvi".into(),
            options: DictionaryExactOptions {
                cursor: Some(malformed_foreign_full_managed),
                ..Default::default()
            },
        }),
        Err(vlajmaji_search::CollectionQueryError::MalformedCursor)
    ));
    assert!(matches!(
        engine.execute(DictionaryRequest::Exact {
            spelling: "farvi".into(),
            options: DictionaryExactOptions {
                cursor: Some("collection-v1:not-a-valid-cursor".into()),
                ..DictionaryExactOptions::default()
            },
        }),
        Err(vlajmaji_search::CollectionQueryError::MalformedCursor)
    ));

    let compact_page_options = DictionaryExactOptions {
        page_size: Some(1),
        ..DictionaryExactOptions::default()
    };
    let compact_page = engine
        .execute(DictionaryRequest::Exact {
            spelling: "farvi".into(),
            options: compact_page_options,
        })
        .unwrap();
    let CollectionCompactQueryResult::Exact(compact_page) = compact_page.result else {
        panic!("expected managed compact page");
    };
    let compact_cursor = compact_page.next_cursor.unwrap();
    let malformed_foreign_compact_managed =
        mutate_cursor_payload_nibble(&compact_cursor, "collection-compact-v1:".len());
    assert!(matches!(
        full_engine.execute(DictionaryRequest::Exact {
            spelling: "farvi".into(),
            options: DictionaryExactOptions {
                page_size: Some(1),
                cursor: Some(compact_cursor.clone()),
                ..Default::default()
            },
        }),
        Err(vlajmaji_search::CollectionQueryError::StaleCursor)
    ));
    assert!(matches!(
        full_engine.execute(DictionaryRequest::Exact {
            spelling: "farvi".into(),
            options: DictionaryExactOptions {
                cursor: Some(malformed_foreign_compact_managed.clone()),
                ..Default::default()
            },
        }),
        Err(vlajmaji_search::CollectionQueryError::MalformedCursor)
    ));
    assert!(matches!(
        full_engine.execute(DictionaryRequest::Exact {
            spelling: "farvi".into(),
            options: DictionaryExactOptions {
                cursor: Some("collection-compact-v1:not-hex:bad".into()),
                ..Default::default()
            },
        }),
        Err(vlajmaji_search::CollectionQueryError::MalformedCursor)
    ));
    let mut disabled_full_engine = CollectionSearchEngine::open(
        &root,
        CollectionQuerySelection {
            packs: vec![vlajmaji_lexicon::PackId::Llg],
            disabled: vec![vlajmaji_lexicon::PackId::Llg],
            profile: None,
        },
    )
    .unwrap();
    assert!(matches!(
        disabled_full_engine.execute(DictionaryRequest::Exact {
            spelling: "farvi".into(),
            options: DictionaryExactOptions {
                cursor: Some(compact_cursor.clone()),
                ..Default::default()
            },
        }),
        Err(vlajmaji_search::CollectionQueryError::StaleCursor)
    ));
    assert!(matches!(
        disabled_full_engine.execute(DictionaryRequest::Exact {
            spelling: "farvi".into(),
            options: DictionaryExactOptions {
                cursor: Some(malformed_foreign_compact_managed),
                ..Default::default()
            },
        }),
        Err(vlajmaji_search::CollectionQueryError::MalformedCursor)
    ));
    let mut fresh_engine = CollectionCompactSearchEngine::open(
        &root,
        CollectionQuerySelection {
            packs: vec![vlajmaji_lexicon::PackId::Llg],
            disabled: vec![],
            profile: None,
        },
    )
    .unwrap();
    let fresh_page = fresh_engine
        .execute(DictionaryRequest::Exact {
            spelling: "farvi".into(),
            options: DictionaryExactOptions {
                cursor: Some(compact_cursor),
                ..DictionaryExactOptions::default()
            },
        })
        .unwrap();
    let CollectionCompactQueryResult::Exact(fresh_page) = fresh_page.result else {
        panic!("expected a fresh-engine compact continuation");
    };
    assert_eq!(
        fresh_page
            .groups
            .iter()
            .flat_map(|group| &group.hits)
            .count(),
        5
    );
    fs::remove_dir_all(base).unwrap();
}

#[test]
fn compact_managed_local_reference_retains_all_contributor_scopes() {
    let base = scratch();
    let root = base.join("catalogue");
    let mut document = fixture::complete_fixture().unwrap().into_document();
    document.legacy_archive = None;
    let dictionary = vlajmaji_lexicon::Dictionary::from_document(document).unwrap();
    let entry_id = dictionary.document().metadata.entries[0].id.clone();
    let packs = [
        vlajmaji_lexicon::PackId::Llg,
        vlajmaji_lexicon::PackId::LaLojban,
    ];
    let sealed = packs
        .iter()
        .map(|pack| {
            (
                *pack,
                vlajmaji_lexicon::seal_release(
                    *pack,
                    vlajmaji_lexicon::PackPayloadKind::Synthetic,
                    &dictionary,
                )
                .unwrap(),
            )
        })
        .collect::<Vec<_>>();
    let inputs = sealed
        .iter()
        .map(|(pack, release)| CollectionInput::SealedBytes {
            pack_id: *pack,
            release: std::borrow::Cow::Borrowed(release.release.as_slice()),
            seal: std::borrow::Cow::Borrowed(release.seal.as_slice()),
        })
        .collect();
    let request =
        CollectionRequest::new(inputs, CollectionSelection::Packs(packs.to_vec()), true).unwrap();
    CollectionCatalogue::install(&root, request).unwrap();

    for (selected, disabled) in [
        (packs.to_vec(), vec![]),
        (vec![vlajmaji_lexicon::PackId::Llg], vec![]),
        (packs.to_vec(), vec![vlajmaji_lexicon::PackId::Llg]),
    ] {
        let mut engine = CollectionCompactSearchEngine::open(
            &root,
            CollectionQuerySelection {
                packs: selected,
                disabled,
                profile: None,
            },
        )
        .unwrap();
        let response = engine
            .execute(DictionaryRequest::Resolve {
                target: RelationTarget::Local(vlajmaji_lexicon::LocalRef::Entry(entry_id.clone())),
            })
            .unwrap();
        let CollectionCompactQueryResult::Resolve(response) = response.result else {
            panic!("expected compact Local reference response");
        };
        assert_eq!(response.status, vlajmaji_search::ReferenceStatus::Resolved);
        assert!(response.payload.is_some());
        assert_eq!(
            response.catalogue_matches.len(),
            1,
            "matches: {:#?}",
            response.catalogue_matches
        );
        assert_eq!(
            response.catalogue_matches[0].contributing_pack_ids,
            vec!["la-lojban".to_owned(), "llg".to_owned()]
        );
        let exact = engine
            .execute(DictionaryRequest::Exact {
                spelling: "farvi".into(),
                options: DictionaryExactOptions::default(),
            })
            .unwrap();
        let CollectionCompactQueryResult::Exact(exact) = exact.result else {
            panic!("expected compact exact for selected shared scope");
        };
        let detail_ref = &exact.groups[0].hits[0].detail_ref;
        let detail = engine.detail(detail_ref, None).unwrap();
        assert_eq!(
            detail.response.status,
            vlajmaji_search::ReferenceStatus::Resolved
        );
        assert!(detail.response.hit.is_some());
        assert_eq!(detail.response.current_eligible, Some(true));
        assert_eq!(detail.catalogue_matches.len(), 1);
        assert_eq!(
            detail.catalogue_matches[0].contributing_pack_ids,
            vec!["la-lojban".to_owned(), "llg".to_owned()]
        );
    }
    fs::remove_dir_all(base).unwrap();
}

#[test]
fn compact_local_reference_retains_distinct_assertion_scopes() {
    let base = scratch();
    let root = base.join("catalogue");
    let mut document = fixture::complete_fixture().unwrap().into_document();
    document.legacy_archive = None;
    let entry_id = document
        .metadata
        .entries
        .iter()
        .find(|entry| entry.id.as_str() == "entry-a-farvi")
        .unwrap()
        .id
        .clone();
    let selected_assertion = document
        .metadata
        .assertions
        .iter()
        .find(|assertion| {
            assertion.entry == entry_id
                && document.metadata.releases.iter().any(|release| {
                    release.id.as_str() == "release-a" && release.assertions.contains(&assertion.id)
                })
        })
        .unwrap()
        .id
        .clone();
    let alternate_assertion = document
        .metadata
        .assertions
        .iter()
        .find(|assertion| {
            assertion.entry == entry_id
                && assertion.id != selected_assertion
                && document.metadata.releases.iter().any(|release| {
                    release.id.as_str() == "release-a" && release.assertions.contains(&assertion.id)
                })
        })
        .unwrap()
        .id
        .clone();
    let dictionary = Dictionary::from_document(document.clone()).unwrap();

    let mut alternate_document = document;
    let release = alternate_document
        .metadata
        .releases
        .iter_mut()
        .find(|release| release.id.as_str() == "release-a")
        .unwrap();
    release.id = vlajmaji_lexicon::EntityId::new("release-alt-a").unwrap();
    let alternate_dictionary = Dictionary::from_document(alternate_document).unwrap();

    let packs = [PackId::Llg, PackId::LaLojban];
    let sealed = [
        (
            packs[0],
            seal_release(packs[0], PackPayloadKind::Synthetic, &dictionary).unwrap(),
        ),
        (
            packs[1],
            seal_release(packs[1], PackPayloadKind::Synthetic, &alternate_dictionary).unwrap(),
        ),
    ];
    let inputs = sealed
        .iter()
        .map(|(pack, release)| CollectionInput::SealedBytes {
            pack_id: *pack,
            release: std::borrow::Cow::Borrowed(release.release.as_slice()),
            seal: std::borrow::Cow::Borrowed(release.seal.as_slice()),
        })
        .collect();
    let request =
        CollectionRequest::new(inputs, CollectionSelection::Packs(packs.to_vec()), true).unwrap();
    CollectionCatalogue::install(&root, request).unwrap();

    let mut engine = CollectionCompactSearchEngine::open(
        &root,
        CollectionQuerySelection {
            packs: packs.to_vec(),
            disabled: vec![],
            profile: None,
        },
    )
    .unwrap();
    let resolved = engine
        .execute(DictionaryRequest::Resolve {
            target: RelationTarget::Local(vlajmaji_lexicon::LocalRef::Entry(entry_id.clone())),
        })
        .unwrap();
    let CollectionCompactQueryResult::Resolve(resolved) = resolved.result else {
        panic!("expected compact Local reference response");
    };
    assert_eq!(resolved.status, vlajmaji_search::ReferenceStatus::Resolved);
    assert_eq!(resolved.catalogue_matches.len(), 1);
    assert_eq!(
        resolved.catalogue_matches[0].contributing_pack_ids,
        vec!["la-lojban".to_owned(), "llg".to_owned()]
    );

    let exact = engine
        .execute(DictionaryRequest::Exact {
            spelling: "farvi".into(),
            options: DictionaryExactOptions::default(),
        })
        .unwrap();
    let CollectionCompactQueryResult::Exact(exact) = exact.result else {
        panic!("expected managed compact Exact response");
    };
    let hits = exact
        .groups
        .iter()
        .flat_map(|group| &group.hits)
        .filter(|hit| hit.entry.id == entry_id)
        .collect::<Vec<_>>();
    assert!(
        hits.len() >= 2,
        "distinct assertions must survive entry coalescing"
    );
    for assertion_id in [&selected_assertion, &alternate_assertion] {
        let hit = hits
            .iter()
            .find(|hit| &hit.assertion.id == assertion_id)
            .unwrap();
        let detail = engine.detail(&hit.detail_ref, None).unwrap();
        assert_eq!(
            detail.response.status,
            vlajmaji_search::ReferenceStatus::Resolved
        );
        assert!(detail.response.hit.is_some());
        assert_eq!(detail.response.current_eligible, Some(true));
        assert_eq!(detail.catalogue_matches.len(), 1);
        assert_eq!(
            detail.catalogue_matches[0].contributing_pack_ids,
            vec!["la-lojban".to_owned(), "llg".to_owned()]
        );
    }
    fs::remove_dir_all(base).unwrap();
}

#[test]
fn compact_local_reference_filters_distinct_pack_assertions_by_policy() {
    use std::collections::BTreeSet;

    let base = scratch();
    let first = one_row_llg_dictionary();
    let mut second_value = serde_json::to_value(first.clone().into_document()).unwrap();
    let mut renamed = BTreeMap::new();
    for category in [
        "releases",
        "assertions",
        "contexts",
        "frames",
        "forms",
        "etymologies",
        "relations",
        "owned_revisions",
        "editorial_senses",
        "computations",
        "annotations",
    ] {
        for row in second_value["metadata"][category].as_array().unwrap() {
            if let Some(id) = row.get("id").and_then(serde_json::Value::as_str) {
                renamed.insert(id.to_owned(), format!("second-{id}"));
            }
        }
    }
    for capture in first.captures() {
        renamed.insert(
            capture.identity.namespace.clone(),
            format!(
                "captured:second:{}",
                capture
                    .identity
                    .namespace
                    .strip_prefix("captured:")
                    .unwrap()
            ),
        );
    }
    fn rename_values(value: &mut serde_json::Value, map: &BTreeMap<String, String>) {
        match value {
            serde_json::Value::String(text) => {
                if let Some(next) = map.get(text) {
                    *text = next.clone();
                }
            }
            serde_json::Value::Array(rows) => {
                for row in rows {
                    rename_values(row, map);
                }
            }
            serde_json::Value::Object(fields) => {
                for value in fields.values_mut() {
                    rename_values(value, map);
                }
            }
            _ => {}
        }
    }
    rename_values(&mut second_value, &renamed);
    let second = Dictionary::from_json_bytes(&serde_json::to_vec(&second_value).unwrap()).unwrap();
    assert_eq!(first.metadata().entries, second.metadata().entries);
    let entry = first.metadata().entries[0].clone();
    let expected = [&first, &second].map(|dictionary| {
        dictionary
            .metadata()
            .assertions
            .iter()
            .filter(|assertion| assertion.entry == entry.id)
            .map(|assertion| (assertion.id.to_string(), assertion.clone()))
            .collect::<BTreeMap<_, _>>()
    });
    let expected_release_ids = [&first, &second].map(|dictionary| {
        dictionary
            .metadata()
            .assertions
            .iter()
            .map(|assertion| {
                let ids = dictionary
                    .metadata()
                    .releases
                    .iter()
                    .filter(|release| {
                        release.assertions.contains(&assertion.id)
                            && release.selected
                            && release.availability == vlajmaji_lexicon::Availability::Available
                    })
                    .map(|release| release.id.to_string())
                    .collect::<BTreeSet<_>>();
                (assertion.id.to_string(), ids)
            })
            .collect::<BTreeMap<_, _>>()
    });
    assert!(!expected[0].is_empty());
    assert!(!expected[1].is_empty());
    assert!(expected[0].keys().all(|id| !expected[1].contains_key(id)));
    let releases = [&first, &second].map(|dictionary| {
        dictionary
            .metadata()
            .releases
            .iter()
            .filter(|release| {
                release.selected
                    && release.availability == vlajmaji_lexicon::Availability::Available
            })
            .map(|release| release.id.to_string())
            .collect::<BTreeSet<_>>()
    });
    assert!(releases[0].is_disjoint(&releases[1]));

    let packs = [PackId::LaLojban, PackId::Llg];
    let dictionaries = [&first, &second];
    let sealed = packs
        .iter()
        .zip(dictionaries)
        .map(|(pack, dictionary)| {
            (
                *pack,
                seal_release(*pack, PackPayloadKind::Synthetic, dictionary).unwrap(),
            )
        })
        .collect::<Vec<_>>();
    let inputs = packs
        .iter()
        .zip(&sealed)
        .map(|(pack, (_, release))| CollectionInput::SealedBytes {
            pack_id: *pack,
            release: std::borrow::Cow::Borrowed(release.release.as_slice()),
            seal: std::borrow::Cow::Borrowed(release.seal.as_slice()),
        })
        .collect();
    let root = base.join("catalogue");
    CollectionCatalogue::install(
        &root,
        CollectionRequest::new(inputs, CollectionSelection::Packs(packs.to_vec()), true).unwrap(),
    )
    .unwrap();

    let cases = [
        ("all-selected", vec![0, 1], vec![]),
        ("first-selected", vec![0], vec![]),
        ("second-selected", vec![1], vec![]),
        ("first-disabled", vec![0, 1], vec![0]),
        ("second-disabled", vec![0, 1], vec![1]),
        ("all-disabled", vec![0, 1], vec![0, 1]),
    ];
    for (label, selected, disabled) in cases {
        let admitted = selected
            .iter()
            .copied()
            .filter(|index| !disabled.contains(index))
            .collect::<Vec<_>>();
        let wanted = admitted
            .iter()
            .flat_map(|index| expected[*index].clone())
            .collect::<BTreeMap<_, _>>();
        let mut engine = CollectionCompactSearchEngine::open(
            &root,
            CollectionQuerySelection {
                packs: selected.iter().map(|index| packs[*index]).collect(),
                disabled: disabled.iter().map(|index| packs[*index]).collect(),
                profile: None,
            },
        )
        .unwrap();
        let resolve = engine
            .execute(DictionaryRequest::Resolve {
                target: RelationTarget::Local(vlajmaji_lexicon::LocalRef::Entry(entry.id.clone())),
            })
            .unwrap();
        let CollectionCompactQueryResult::Resolve(resolve) = resolve.result else {
            panic!("expected compact Local Resolve");
        };
        if wanted.is_empty() {
            assert_eq!(
                resolve.status,
                vlajmaji_search::ReferenceStatus::CatalogueOnly,
                "{label}"
            );
            assert!(resolve.payload.is_none(), "{label}");
        } else {
            assert_eq!(
                resolve.status,
                vlajmaji_search::ReferenceStatus::Resolved,
                "{label}"
            );
            let payload = resolve.payload.expect("selected pack should load payload");
            let actual = payload
                .confirmed
                .iter()
                .map(|row| (row.assertion.id.to_string(), row.assertion.clone()))
                .collect::<BTreeMap<_, _>>();
            assert_eq!(
                actual,
                wanted,
                "{label}; selected releases={:?}",
                engine.selection().selected_releases
            );
            for row in &payload.confirmed {
                let expected_scope = admitted
                    .iter()
                    .find_map(|index| expected_release_ids[*index].get(row.assertion.id.as_str()))
                    .expect("selected assertion has a pre-input release scope");
                assert_eq!(
                    row.release_ids
                        .iter()
                        .map(ToString::to_string)
                        .collect::<BTreeSet<_>>(),
                    *expected_scope,
                    "{label}: exact release scope for {}",
                    row.assertion.id
                );
            }

            let exact = engine
                .execute(DictionaryRequest::Exact {
                    spelling: "farvi".into(),
                    options: DictionaryExactOptions::default(),
                })
                .unwrap();
            let CollectionCompactQueryResult::Exact(exact) = exact.result else {
                panic!("expected compact Exact");
            };
            let hits = exact
                .groups
                .iter()
                .flat_map(|group| &group.hits)
                .filter(|hit| hit.entry.id == entry.id)
                .collect::<Vec<_>>();
            let actual_hits = hits
                .iter()
                .map(|hit| (hit.assertion.id.to_string(), hit.assertion.clone()))
                .collect::<BTreeMap<_, _>>();
            assert_eq!(actual_hits, wanted, "{label}: Exact hit set");
            for hit in hits {
                let detail = engine.detail(&hit.detail_ref, None).unwrap();
                assert_eq!(
                    detail.response.status,
                    vlajmaji_search::ReferenceStatus::Resolved,
                    "{label}"
                );
                assert_eq!(
                    detail
                        .response
                        .hit
                        .as_ref()
                        .map(|row| row.assertion.clone()),
                    wanted.get(hit.assertion.id.as_str()).cloned(),
                    "{label}: detail body"
                );
                assert_eq!(detail.response.current_eligible, Some(true), "{label}");
            }
        }
    }
    fs::remove_dir_all(base).unwrap();
}

#[test]
fn compact_local_reference_deduplicates_equal_history_and_rejects_unequal_body() {
    for (conflicting_body, expect_ambiguity) in [(false, false), (true, true)] {
        let base = scratch();
        let root = base.join("catalogue");
        let mut current_document = fixture::complete_fixture().unwrap().into_document();
        current_document.legacy_archive = None;
        let assertion_id = current_document.metadata.assertions[0].id.clone();
        let current_dictionary = Dictionary::from_document(current_document.clone()).unwrap();

        let mut historical_document = current_document;
        historical_document.metadata.catalogue_revision += 1;
        historical_document.metadata.releases[0].id =
            vlajmaji_lexicon::EntityId::new("history-release").unwrap();
        if conflicting_body {
            historical_document.metadata.assertions[0].native_sense_id =
                vlajmaji_lexicon::Presence::Value("different-native-sense".into());
        }
        let historical_dictionary = Dictionary::from_document(historical_document).unwrap();

        // Freeze identity/body expectations from the two inputs before either
        // release is registered in the catalogue.
        assert_eq!(
            current_dictionary.document().metadata.assertions[0].id,
            historical_dictionary.document().metadata.assertions[0].id
        );
        assert_eq!(
            current_dictionary.document().metadata.assertions[0]
                == historical_dictionary.document().metadata.assertions[0],
            !conflicting_body
        );

        let current = vlajmaji_lexicon::seal_release(
            PackId::Llg,
            PackPayloadKind::Synthetic,
            &current_dictionary,
        )
        .unwrap();
        let historical = vlajmaji_lexicon::seal_release(
            PackId::Llg,
            PackPayloadKind::Synthetic,
            &historical_dictionary,
        )
        .unwrap();
        let install = CollectionRequest::new(
            vec![CollectionInput::SealedBytes {
                pack_id: PackId::Llg,
                release: std::borrow::Cow::Borrowed(current.release.as_slice()),
                seal: std::borrow::Cow::Borrowed(current.seal.as_slice()),
            }],
            CollectionSelection::Packs(vec![PackId::Llg]),
            true,
        )
        .unwrap();
        CollectionCatalogue::install(&root, install).unwrap();
        let register = CollectionRequest::new(
            vec![CollectionInput::SealedBytes {
                pack_id: PackId::Llg,
                release: std::borrow::Cow::Borrowed(historical.release.as_slice()),
                seal: std::borrow::Cow::Borrowed(historical.seal.as_slice()),
            }],
            CollectionSelection::Packs(vec![PackId::Llg]),
            true,
        )
        .unwrap();
        CollectionCatalogue::register(&root, register).unwrap();

        let mut engine = CollectionCompactSearchEngine::open(
            &root,
            CollectionQuerySelection {
                packs: vec![PackId::Llg],
                disabled: vec![],
                profile: None,
            },
        )
        .unwrap();
        let result = engine.execute(DictionaryRequest::Resolve {
            target: RelationTarget::Local(vlajmaji_lexicon::LocalRef::Assertion(assertion_id)),
        });
        if expect_ambiguity {
            assert!(matches!(
                result,
                Err(vlajmaji_search::CollectionQueryError::AmbiguousReference)
            ));
        } else {
            let result = result.unwrap();
            let CollectionCompactQueryResult::Resolve(reference) = result.result else {
                panic!("expected compact Resolve result");
            };
            assert_eq!(reference.status, vlajmaji_search::ReferenceStatus::Resolved);
            assert!(reference.payload.is_some());
            assert_eq!(reference.catalogue_matches.len(), 1);
            assert_eq!(
                reference.catalogue_matches[0].contributing_pack_ids,
                vec!["llg"]
            );
        }
        fs::remove_dir_all(base).unwrap();
    }
}

#[test]
fn compact_four_literal_install_matches_preinput_typed_family_union() {
    const LENSISKU: &[u8] = br#"[{"word":"farvi","word_type":"gismu","definition":"synthetic develops","definition_id":333,"score":1.2500,"user":{"username":"shared"}}]"#;
    const SUTYSISKU: &[u8] = br#"{"farvi":{"d":"synthetic one","t":"gismu","n":null},"farvi":{"d":"synthetic two","t":"gismu"},"zoizoizoi":{"d":"retained excluded","t":"unsupported"}}"#;
    const LLG_GISMU_HEX: &str = "206661727669202020202020202020202020202067726f772020202020202020202020202020202020202020202020202020202020202020202020202020642020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020";
    let decode_hex = |value: &str| {
        let bytes = value.as_bytes();
        assert_eq!(bytes.len() % 2, 0);
        let (pairs, remainder) = bytes.as_chunks::<2>();
        assert!(remainder.is_empty());
        pairs
            .iter()
            .map(|pair| {
                let digit = |byte: u8| (byte as char).to_digit(16).unwrap() as u8;
                (digit(pair[0]) << 4) | digit(pair[1])
            })
            .collect::<Vec<_>>()
    };
    let llg_gismu = decode_hex(LLG_GISMU_HEX);
    assert_eq!(LENSISKU.len(), 136);
    assert_eq!(
        sha256(LENSISKU),
        "63fb2e10e80f34e6d7f04fbe9bcfe1cd69c7ec0c4016045d02e8b3cfcd866429"
    );
    assert_eq!(SUTYSISKU.len(), 150);
    assert_eq!(
        sha256(SUTYSISKU),
        "3e7738c9412a4dfb6169e8492f8ecb7fd25003d2141dbb8fa11782e580d00b03"
    );
    assert_eq!(llg_gismu.len(), 158);
    assert_eq!(
        sha256(&llg_gismu),
        "0408fd8bf1adae9d0da1e7faabb35a6c2cdf64d012140bcadfdd814f061b2735"
    );
    let empty = b"";
    let notice = b"synthetic notice\n";
    let metadata = format!(
        "language_tag = \"en\"\nlanguage_realname = \"English\"\nsource_language_tag = \"jbo\"\nformat = \"json\"\npositive_scores_only = false\nfilename = \"dictionary-en.json\"\nmetadata_url = \"https://example.invalid/languages\"\ndownload_url = \"https://example.invalid/export\"\nlensisku_created_at = \"2026-10-04T03:07:08Z\"\nsha256 = \"{}\"\ndefinition_count = 1\nentry_count = 1\n",
        sha256(LENSISKU)
    );
    let jbotci = import_jbotci_export(
        LENSISKU,
        metadata.as_bytes(),
        &JbotciImportOptions {
            revision: "1111111111111111111111111111111111111111".into(),
            expected_raw_sha256: sha256(LENSISKU),
            expected_metadata_sha256: sha256(metadata.as_bytes()),
        },
    )
    .unwrap()
    .into_dictionary();
    let la = import_la_lojban_export(
        LENSISKU,
        &LaLojbanImportOptions {
            revision: "2222222222222222222222222222222222222222".into(),
            expected_raw_sha256: sha256(LENSISKU),
            raw_locator: None,
            notice_locator: None,
        },
    )
    .unwrap()
    .into_dictionary();
    let sut = import_sutysisku_export(
        SUTYSISKU,
        &SutysiskuImportOptions {
            revision: "3333333333333333333333333333333333333333".into(),
            expected_raw_sha256: sha256(SUTYSISKU),
            raw_locator: None,
            notice_locator: None,
        },
    )
    .unwrap()
    .into_dictionary();
    let llg = import_llg_lists(
        &llg_gismu,
        empty,
        empty,
        notice,
        &LlgImportOptions {
            expected_gismu_sha256: sha256(&llg_gismu),
            expected_cmavo_sha256: sha256(empty),
            expected_rafsi_sha256: sha256(empty),
            expected_notice_sha256: sha256(notice),
            base_locator: "https://example.invalid/llg".into(),
        },
    )
    .unwrap()
    .into_dictionary();
    let dictionaries = [
        (PackId::LensiskuJbotci, jbotci),
        (PackId::Llg, llg),
        (PackId::LaLojban, la),
        (PackId::Sutysisku, sut),
    ];

    // Freeze the expected maximum multiset across the individual inputs before
    // creating the catalogue or asking the product composer for a result.
    let per_pack = dictionaries
        .iter()
        .map(|(_, dictionary)| dictionary_inventory(&unverified_document(dictionary)))
        .collect::<Vec<_>>();
    let mut expected_hit_records = BTreeMap::new();
    for (_, dictionary) in &dictionaries {
        let document = unverified_document(dictionary);
        for assertion in &document.metadata.assertions {
            let entry = document
                .metadata
                .entries
                .iter()
                .find(|entry| entry.id == assertion.entry)
                .expect("every pre-input assertion has its linked Entry");
            let key = (entry.id.to_string(), assertion.id.to_string());
            let records = (entry.clone(), assertion.clone());
            if let Some(existing) = expected_hit_records.insert(key, records.clone()) {
                assert_eq!(existing, records, "duplicate input identity has one body");
            }
        }
    }
    let mut expected = BTreeMap::<String, BTreeMap<String, usize>>::new();
    for inventory in &per_pack {
        for (category, rows) in inventory {
            let target = expected.entry(category.clone()).or_default();
            for (digest, count) in rows {
                target
                    .entry(digest.clone())
                    .and_modify(|current| *current = (*current).max(*count))
                    .or_insert(*count);
            }
        }
    }
    assert_eq!(expected.len(), 25);
    assert_eq!(expected["assertions"].values().sum::<usize>(), 6);
    assert_eq!(expected["captures"].values().sum::<usize>(), 8);
    let mut reordered_sut = unverified_document(&dictionaries[3].1);
    reordered_sut.metadata.assertions.reverse();
    assert_eq!(dictionary_inventory(&reordered_sut), per_pack[3]);
    let mut missing_empty = expected.clone();
    missing_empty.remove("contexts");
    assert_ne!(missing_empty, expected);
    let mut extra_multiplicity = expected.clone();
    let (digest, count) = extra_multiplicity
        .get_mut("assertions")
        .unwrap()
        .iter_mut()
        .next()
        .unwrap();
    assert!(!digest.is_empty());
    *count += 1;
    assert_ne!(extra_multiplicity, expected);
    let mut changed_presence = unverified_document(&dictionaries[0].1);
    changed_presence.metadata.entries[0].spelling = vlajmaji_lexicon::Presence::Null;
    assert_ne!(dictionary_inventory(&changed_presence), per_pack[0]);

    let sealed = dictionaries
        .iter()
        .map(|(pack, dictionary)| {
            (
                *pack,
                seal_release(*pack, PackPayloadKind::Synthetic, dictionary).unwrap(),
            )
        })
        .collect::<Vec<_>>();
    let inputs = sealed
        .iter()
        .map(|(pack, release)| CollectionInput::SealedBytes {
            pack_id: *pack,
            release: std::borrow::Cow::Borrowed(release.release.as_slice()),
            seal: std::borrow::Cow::Borrowed(release.seal.as_slice()),
        })
        .collect();
    let packs = dictionaries
        .iter()
        .map(|(pack, _)| *pack)
        .collect::<Vec<_>>();
    let request =
        CollectionRequest::new(inputs, CollectionSelection::Packs(packs.clone()), true).unwrap();
    let base = scratch();
    let root = base.join("catalogue");
    let report = CollectionCatalogue::install(&root, request).unwrap();
    let generation = report.generation.unwrap();
    let store = DictionaryStore::open(&generation.store_path).unwrap();
    let actual = dictionary_inventory(&store.unverified_document().unwrap());
    assert_eq!(actual, expected);

    let compact = DictionaryCompactSearchEngine::open(&generation.store_path).unwrap();
    let result = compact
        .exact("farvi", DictionaryExactOptions::default())
        .unwrap();
    let full_engine = DictionarySearchEngine::open(&generation.store_path).unwrap();
    let full = full_engine
        .exact("farvi", DictionaryExactOptions::default())
        .unwrap();
    #[derive(Serialize)]
    struct ExpectedSnapshot<'a> {
        version: u32,
        policy: &'static str,
        recorded_state_sha256: &'a str,
        generation: Option<&'a str>,
        selected_release_ids: Option<&'a [vlajmaji_lexicon::EntityId]>,
        availability: Vec<ExpectedAvailability<'a>>,
    }
    #[derive(Serialize)]
    struct ExpectedAvailability<'a> {
        source_id: &'a vlajmaji_lexicon::SourceId,
        selected_available_release_ids: Vec<&'a vlajmaji_lexicon::EntityId>,
        loaded_captures: &'a [vlajmaji_lexicon::CaptureRef],
        missing_captures: &'a [vlajmaji_lexicon::CaptureRef],
        owned_head: Option<&'a vlajmaji_lexicon::OwnedHead>,
        selectable: bool,
        reason: Option<vlajmaji_search::SourceSelectionFailure>,
    }
    let expected_snapshot = ExpectedSnapshot {
        version: 1,
        policy: "compact-dictionary-v1",
        recorded_state_sha256: compact.state_sha256(),
        generation: None,
        selected_release_ids: None,
        availability: full
            .availability
            .iter()
            .map(|row| ExpectedAvailability {
                source_id: &row.source.id,
                selected_available_release_ids: row.selected_available_release_ids.iter().collect(),
                loaded_captures: &row.loaded_captures,
                missing_captures: &row.missing_captures,
                owned_head: row.owned_head.as_ref(),
                selectable: row.selectable,
                reason: row.reason,
            })
            .collect(),
    };
    assert_eq!(
        compact.snapshot_sha256(),
        sha256(&serde_json::to_vec(&expected_snapshot).unwrap()),
        "the compact snapshot hashes the declared availability body"
    );
    assert_eq!(result.totals, full.totals);
    assert_eq!(result.source_totals, full.source_totals);
    assert_eq!(
        result.native_candidate_occurrences,
        full.native_candidate_occurrences
    );
    assert_eq!(result.availability.len(), full.availability.len());
    for (compact_state, full_state) in result.availability.iter().zip(&full.availability) {
        assert_eq!(compact_state.source, full_state.source);
        assert_eq!(compact_state.aliases, full_state.aliases);
        assert_eq!(compact_state.lineage, full_state.lineage);
        assert_eq!(compact_state.collections, full_state.collections);
        assert_eq!(compact_state.components, full_state.components);
        assert_eq!(
            compact_state.selected_available_release_ids,
            full_state.selected_available_release_ids
        );
        assert_eq!(compact_state.loaded_captures, full_state.loaded_captures);
        assert_eq!(compact_state.missing_captures, full_state.missing_captures);
        assert_eq!(compact_state.owned_head, full_state.owned_head);
        assert_eq!(compact_state.selectable, full_state.selectable);
        assert_eq!(compact_state.reason, full_state.reason);
    }
    assert_eq!(result.totals.occurrences, 5);
    assert_eq!(result.totals.source_groups, 3);
    assert_eq!(
        result
            .source_totals
            .iter()
            .map(|row| (row.source.id.to_string(), row.occurrences))
            .collect::<BTreeMap<_, _>>(),
        BTreeMap::from([
            ("lensisku".into(), 2),
            ("llg".into(), 1),
            ("sutysisku".into(), 2),
        ])
    );
    let full_ids = full
        .groups
        .iter()
        .flat_map(|group| &group.hits)
        .map(|hit| hit.occurrence_id.clone())
        .collect::<Vec<_>>();
    for page_size in [1, 2, 3] {
        let mut page_ids = Vec::new();
        let mut cursor = None;
        loop {
            let page = compact
                .exact(
                    "farvi",
                    DictionaryExactOptions {
                        page_size: Some(page_size),
                        cursor: cursor.take(),
                        ..DictionaryExactOptions::default()
                    },
                )
                .unwrap();
            page_ids.extend(
                page.groups
                    .iter()
                    .flat_map(|group| &group.hits)
                    .map(|hit| hit.occurrence_id.clone()),
            );
            cursor = page.next_cursor;
            if cursor.is_none() {
                break;
            }
        }
        assert_eq!(page_ids, full_ids, "page size {page_size}");
    }
    assert_eq!(
        result.groups.iter().flat_map(|group| &group.hits).count(),
        5
    );
    let page_zero = compact
        .exact(
            "farvi",
            DictionaryExactOptions {
                page_size: Some(0),
                ..DictionaryExactOptions::default()
            },
        )
        .unwrap();
    assert!(page_zero.groups.is_empty());
    assert!(page_zero.next_cursor.is_none());
    assert_eq!(page_zero.totals, result.totals);
    assert_eq!(page_zero.source_totals, result.source_totals);
    for compact_hit in result.groups.iter().flat_map(|group| &group.hits) {
        let full_hit = full
            .groups
            .iter()
            .flat_map(|group| &group.hits)
            .find(|hit| hit.occurrence_id == compact_hit.occurrence_id)
            .expect("each compact literal occurrence has a full hit");
        let (expected_entry, expected_assertion) = expected_hit_records
            .get(&(
                compact_hit.entry.id.to_string(),
                compact_hit.assertion.id.to_string(),
            ))
            .expect("every compact literal identity is present in the original inputs");
        assert_eq!(&compact_hit.entry, expected_entry);
        assert_eq!(&compact_hit.assertion, expected_assertion);
        assert_eq!(compact_hit.detail_ref.role, "assertion");
        assert_eq!(compact_hit.detail_ref.source, expected_entry.source);
        assert_eq!(compact_hit.detail_ref.entry_id, expected_entry.id);
        assert_eq!(compact_hit.detail_ref.assertion_id, expected_assertion.id);
        assert_eq!(
            compact_hit.detail_ref.occurrence,
            expected_assertion.occurrence
        );
        assert_eq!(
            compact_hit.detail_ref.entry_sha256,
            sha256(&serde_json::to_vec(expected_entry).unwrap())
        );
        assert_eq!(
            compact_hit.detail_ref.assertion_sha256,
            sha256(&serde_json::to_vec(expected_assertion).unwrap())
        );
        assert_eq!(compact_hit.entry, full_hit.entry);
        assert_eq!(compact_hit.assertion, full_hit.assertion);
        assert_eq!(compact_hit.capture, full_hit.capture);
        let detail = compact
            .detail(&compact_hit.detail_ref, Some(&result.snapshot_sha256))
            .unwrap();
        assert_eq!(detail.hit.as_ref(), Some(full_hit));
        assert_eq!(detail.current_eligible, Some(true));
    }
    drop(compact);
    drop(store);
    fs::remove_dir_all(base).unwrap();
}
