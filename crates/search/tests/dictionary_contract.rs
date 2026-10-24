//! Public S04 lookup contract checks against the unchanged accepted fixture.

#[path = "../../lexicon/examples/support/dictionary_fixture.rs"]
mod fixture;

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use vlajmaji_lexicon::{
    CaptureRef, CatalogTarget, ComputedAnnotation, Dictionary, EntityId, LocalRef, OccurrenceRef,
    RawJson, RelationTarget,
};
use vlajmaji_search::dictionary::{
    DictionaryExactOptions, DictionaryQueryError, DictionarySearchEngine, ReferenceStatus,
};

static NEXT_PATH: AtomicU64 = AtomicU64::new(0);

fn scratch() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "vlajmaji-s04-dictionary-contract-{}-{nonce}-{}",
        std::process::id(),
        NEXT_PATH.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&path).unwrap();
    path
}

fn engine(root: &std::path::Path) -> DictionarySearchEngine {
    let dictionary = fixture::complete_fixture().unwrap();
    let bytes = dictionary.to_json_bytes().unwrap();
    DictionarySearchEngine::build(root.join("index"), &bytes).unwrap()
}

fn farvi_rows(
    engine: &DictionarySearchEngine,
    options: DictionaryExactOptions,
) -> Vec<(String, String, String)> {
    engine
        .exact("farvi", options)
        .unwrap()
        .groups
        .into_iter()
        .flat_map(|group| {
            group.hits.into_iter().map(move |hit| {
                (
                    group.source.id.to_string(),
                    hit.raw.locator,
                    hit.capture.source_name,
                )
            })
        })
        .collect()
}

#[test]
fn exact_farvi_matches_the_complete_occurrence_reference_after_reopen() {
    let root = scratch();
    let built = engine(&root);
    let first = built
        .exact("farvi", DictionaryExactOptions::default())
        .unwrap();
    assert_eq!(first.native_candidate_occurrences, 7);
    assert_eq!(first.totals.occurrences, 6);
    assert_eq!(first.totals.distinct_spellings, 1);
    assert_eq!(first.totals.source_groups, 3);
    assert_eq!(
        first
            .source_totals
            .iter()
            .map(|row| (row.source.id.to_string(), row.occurrences))
            .collect::<Vec<_>>(),
        vec![
            ("source-a".into(), 4),
            ("source-b".into(), 1),
            ("source-o".into(), 1),
        ]
    );
    assert_eq!(
        farvi_rows(&built, DictionaryExactOptions::default()),
        vec![
            ("source-a".into(), "/0".into(), "A dictionary v1".into()),
            ("source-a".into(), "/1".into(), "A dictionary v1".into()),
            ("source-a".into(), "/2".into(), "A dictionary v1".into()),
            ("source-a".into(), "/0".into(), "A mirror".into()),
            ("source-b".into(), "/0".into(), "B dictionary".into()),
            ("source-o".into(), "/0".into(), "Owned r1".into()),
        ]
    );
    let expected_rows = farvi_rows(&built, DictionaryExactOptions::default());

    let owned = first
        .availability
        .iter()
        .find(|source| source.source.id.as_str() == "source-o")
        .unwrap();
    assert_eq!(
        owned
            .selected_available_release_ids
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>(),
        vec!["release-o1"]
    );
    assert!(
        owned
            .releases
            .iter()
            .any(|release| { release.id.as_str() == "release-o2" && !release.selected })
    );
    assert_eq!(
        first.groups[0].hits[0]
            .occurrence_attribution
            .as_ref()
            .unwrap()
            .native_assertion_id
            .as_deref(),
        Some("69002")
    );

    drop(built);
    let reopened = DictionarySearchEngine::open(root.join("index")).unwrap();
    assert_eq!(
        farvi_rows(&reopened, DictionaryExactOptions::default()),
        expected_rows
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn pages_zero_totals_cursor_binding_and_stale_cursor_are_explicit() {
    let root = scratch();
    let engine = engine(&root);
    let complete = farvi_rows(&engine, DictionaryExactOptions::default());

    let zero = engine
        .exact(
            "farvi",
            DictionaryExactOptions {
                page_size: Some(0),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(zero.totals.occurrences, 6);
    assert!(zero.groups.is_empty());
    assert!(zero.next_cursor.is_none());

    for page_size in [1, 4] {
        let mut cursor = None;
        let mut rows = Vec::new();
        loop {
            let page = engine
                .exact(
                    "farvi",
                    DictionaryExactOptions {
                        page_size: Some(page_size),
                        cursor: cursor.clone(),
                        ..Default::default()
                    },
                )
                .unwrap();
            rows.extend(page.groups.into_iter().flat_map(|group| {
                group.hits.into_iter().map(move |hit| {
                    (
                        group.source.id.to_string(),
                        hit.raw.locator,
                        hit.capture.source_name,
                    )
                })
            }));
            cursor = page.next_cursor;
            if cursor.is_none() {
                break;
            }
        }
        assert_eq!(rows, complete);
    }

    let first = engine
        .exact(
            "farvi",
            DictionaryExactOptions {
                page_size: Some(1),
                ..Default::default()
            },
        )
        .unwrap();
    let cursor = first.next_cursor.unwrap();
    let err = engine.exact(
        "sie",
        DictionaryExactOptions {
            page_size: Some(1),
            cursor: Some(cursor),
            ..Default::default()
        },
    );
    assert!(matches!(err, Err(DictionaryQueryError::StaleCursor)));

    let malformed = engine
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
    let mut bytes = decode_hex(&malformed);
    let mut envelope: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    envelope["body"]["binding"] = serde_json::Value::String("not-a-digest".into());
    bytes = serde_json::to_vec(&envelope).unwrap();
    let err = engine.exact(
        "farvi",
        DictionaryExactOptions {
            page_size: Some(1),
            cursor: Some(encode_hex(&bytes)),
            ..Default::default()
        },
    );
    assert!(matches!(
        err,
        Err(DictionaryQueryError::MalformedCursor { .. })
    ));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn direct_references_preserve_catalogue_only_and_known_native_mismatch() {
    let root = scratch();
    let engine = engine(&root);
    let dictionary = fixture::complete_fixture().unwrap();
    let document = dictionary.document();
    let catalogue_relation = document
        .metadata
        .relations
        .iter()
        .find_map(|relation| match &relation.target {
            RelationTarget::Catalog(target) if target.source.as_str() == "source-m" => {
                Some(RelationTarget::Catalog(target.clone()))
            }
            _ => None,
        })
        .unwrap();
    let unresolved = engine.resolve(&catalogue_relation).unwrap();
    assert_eq!(unresolved.status, ReferenceStatus::CatalogueOnly);
    assert!(unresolved.confirmed.is_empty());
    assert!(unresolved.unverified.is_empty());
    assert!(unresolved.requested_release.is_some());

    let a_capture = dictionary
        .captures()
        .iter()
        .find(|capture| capture.identity.namespace == "captured:a:d1")
        .unwrap();
    let mismatch = RelationTarget::Catalog(CatalogTarget {
        source: vlajmaji_lexicon::SourceId::new("source-a").unwrap(),
        release: EntityId::new("release-a").unwrap(),
        native_entry_id: Some("known-wrong-entry".into()),
        native_assertion_id: None,
        occurrence: Some(OccurrenceRef {
            capture: CaptureRef {
                namespace: "captured:a:d1".into(),
                sha256: a_capture.identity.sha256.clone(),
            },
            locator: "/0".into(),
        }),
    });
    let mismatch = engine.resolve(&mismatch).unwrap();
    assert_eq!(mismatch.status, ReferenceStatus::NoMatch);
    assert!(mismatch.confirmed.is_empty());
    assert!(mismatch.unverified.is_empty());

    let local = engine
        .resolve(&RelationTarget::Local(LocalRef::Entry(
            EntityId::new("entry-a-farvi").unwrap(),
        )))
        .unwrap();
    assert_eq!(local.status, ReferenceStatus::Resolved);
    assert!(local.entity.is_some());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn repeated_editorial_membership_is_preserved_but_hit_views_are_unique() {
    let root = scratch();
    let mut document = fixture::complete_fixture().unwrap().document();
    let sense = document
        .metadata
        .editorial_senses
        .iter_mut()
        .find(|sense| sense.id.as_str() == "editorial-farvi")
        .unwrap();
    let first_assertion = sense.assertions[0].clone();
    sense.assertions.push(first_assertion);
    assert_eq!(sense.assertions.len(), 3);
    assert_eq!(sense.assertions[0], sense.assertions[2]);

    let dictionary = Dictionary::from_document(document).unwrap();
    let bytes = dictionary.to_json_bytes().unwrap();
    let engine = DictionarySearchEngine::build(root.join("index"), &bytes).unwrap();
    let response = engine
        .exact("farvi", DictionaryExactOptions::default())
        .unwrap();
    let hit = response
        .groups
        .iter()
        .flat_map(|group| &group.hits)
        .find(|hit| hit.capture.source_name == "A dictionary v1" && hit.raw.locator == "/0")
        .unwrap();
    assert_eq!(hit.editorial_senses.len(), 1);
    assert_eq!(hit.editorial_senses[0].assertions.len(), 3);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn hit_computations_follow_dictionary_metadata_order() {
    let root = scratch();
    let mut document = fixture::complete_fixture().unwrap().document();
    let assertion = document
        .metadata
        .assertions
        .iter()
        .find(|row| row.entry.as_str() == "entry-a-farvi")
        .unwrap()
        .id
        .clone();
    let computation_one = document.metadata.computations[0].clone();
    let mut computation_zero = computation_one.clone();
    computation_zero.id = EntityId::new("computation-0").unwrap();
    document.metadata.computations.push(computation_zero);
    let mut unrelated_computation = computation_one.clone();
    unrelated_computation.id = EntityId::new("computation-unrelated").unwrap();
    document.metadata.computations.push(unrelated_computation);
    document.metadata.annotations.extend([
        ComputedAnnotation {
            id: EntityId::new("annotation-farvi-one").unwrap(),
            computation: computation_one.id,
            target: LocalRef::Assertion(assertion.clone()),
            kind: "ordering-control".into(),
            value: RawJson::new(r#"{"order":1}"#).unwrap(),
            explanation: Default::default(),
        },
        ComputedAnnotation {
            id: EntityId::new("annotation-farvi-zero").unwrap(),
            computation: EntityId::new("computation-0").unwrap(),
            target: LocalRef::Assertion(assertion.clone()),
            kind: "ordering-control".into(),
            value: RawJson::new(r#"{"order":0}"#).unwrap(),
            explanation: Default::default(),
        },
    ]);

    let dictionary = Dictionary::from_document(document).unwrap();
    let engine =
        DictionarySearchEngine::build(root.join("index"), &dictionary.to_json_bytes().unwrap())
            .unwrap();
    let response = engine
        .exact("farvi", DictionaryExactOptions::default())
        .unwrap();
    let hit = response
        .groups
        .iter()
        .flat_map(|group| &group.hits)
        .find(|hit| hit.assertion.id == assertion)
        .unwrap();
    assert_eq!(hit.computations.len(), 2);
    assert_eq!(
        hit.computations
            .iter()
            .map(|row| row.id.as_str())
            .collect::<Vec<_>>(),
        vec!["computation-1", "computation-0"]
    );
    fs::remove_dir_all(root).unwrap();
}

fn decode_hex(input: &str) -> Vec<u8> {
    let (pairs, remainder) = input.as_bytes().as_chunks::<2>();
    assert!(remainder.is_empty());
    pairs
        .iter()
        .map(|pair| {
            let high = (pair[0] as char).to_digit(16).unwrap();
            let low = (pair[1] as char).to_digit(16).unwrap();
            ((high << 4) | low) as u8
        })
        .collect()
}

fn encode_hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(DIGITS[(byte >> 4) as usize] as char);
        encoded.push(DIGITS[(byte & 0x0f) as usize] as char);
    }
    encoded
}
