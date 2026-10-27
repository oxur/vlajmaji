//! Lifecycle-state interpretation in the configured query snapshot.

#[path = "../../lexicon/examples/support/dictionary_fixture.rs"]
mod fixture;

use std::{
    borrow::Cow,
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use vlajmaji_lexicon::{
    CaptureRef, CatalogTarget, Dictionary, LocalRef, OccurrenceRef, PackId, PackPayloadKind,
    RelationTarget, SourceId, seal_release,
};
use vlajmaji_search::{
    CollectionQueryError, CollectionQuerySelection, CollectionSearchEngine, DictionaryExactOptions,
    DictionaryRequest,
};
use vlajmaji_store::{
    CollectionCatalogue, CollectionInput, CollectionRequest, CollectionSelection,
};

fn register_empty(root: &std::path::Path, pack_id: PackId) {
    let dictionary = Dictionary::from_document(Default::default()).unwrap();
    let sealed = seal_release(pack_id, PackPayloadKind::Synthetic, &dictionary).unwrap();
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
}

fn fresh_root() -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "vlajmaji-s04-query-lifecycle-{}-{stamp}",
        std::process::id()
    ))
}

#[test]
fn test_open_removed_pack_reports_removed_without_reopening_payload() {
    let root = fresh_root();
    let mut document = fixture::complete_fixture().unwrap().into_document();
    document.legacy_archive = None;
    let dictionary = Dictionary::from_document(document).unwrap();
    let sealed = seal_release(PackId::Llg, PackPayloadKind::Synthetic, &dictionary).unwrap();
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
    CollectionCatalogue::install(&root, request).unwrap();
    register_empty(&root, PackId::LaLojban);

    let mut before = CollectionSearchEngine::open(
        &root,
        CollectionQuerySelection {
            packs: vec![PackId::Llg],
            disabled: Vec::new(),
            profile: None,
        },
    )
    .unwrap();
    let first = before
        .execute(DictionaryRequest::Exact {
            spelling: "farvi".into(),
            options: DictionaryExactOptions {
                page_size: Some(1),
                ..Default::default()
            },
        })
        .unwrap();
    let vlajmaji_search::CollectionQueryResult::Exact(first_page) = first.result else {
        panic!("wrong operation result");
    };
    let cursor = first_page
        .next_cursor
        .expect("fixture has multiple farvi hits");

    let removed = CollectionCatalogue::remove(&root, &[PackId::Llg], 2).unwrap();
    assert_eq!(removed.outcome, "removed");
    assert!(removed.generation.is_none());

    let selection = CollectionQuerySelection {
        packs: vec![PackId::Llg],
        disabled: Vec::new(),
        profile: None,
    };
    let mut engine = CollectionSearchEngine::open(&root, selection).unwrap();
    assert_eq!(
        engine.selection().selection_policy,
        "selected-available-owned-head-v2"
    );
    let row = engine
        .selection()
        .collections
        .iter()
        .find(|row| row.pack_id == "llg")
        .unwrap();
    assert_eq!(row.derived_state, "removed");
    assert!(matches!(
        engine.execute(DictionaryRequest::Exact {
            spelling: "farvi".into(),
            options: DictionaryExactOptions::default(),
        }),
        Err(CollectionQueryError::UnavailablePack { pack_id, reason })
            if pack_id == "llg" && reason == "removed"
    ));

    let old_cursor = engine.execute(DictionaryRequest::Exact {
        spelling: "farvi".into(),
        options: DictionaryExactOptions {
            page_size: Some(1),
            cursor: Some(cursor),
            ..Default::default()
        },
    });
    assert!(matches!(old_cursor, Err(CollectionQueryError::StaleCursor)));

    let capture = dictionary
        .captures()
        .iter()
        .find(|capture| capture.identity.namespace == "captured:a:d1")
        .unwrap();
    let catalog_target = RelationTarget::Catalog(CatalogTarget {
        source: SourceId::new("source-a").unwrap(),
        release: vlajmaji_lexicon::EntityId::new("release-a").unwrap(),
        native_entry_id: None,
        native_assertion_id: None,
        occurrence: Some(OccurrenceRef {
            capture: CaptureRef {
                namespace: capture.identity.namespace.clone(),
                sha256: capture.identity.sha256.clone(),
            },
            locator: "/0".into(),
        }),
    });
    for (policy, expected_reason) in [
        (
            CollectionQuerySelection {
                packs: vec![PackId::Llg],
                disabled: vec![PackId::Llg],
                profile: None,
            },
            "disabled",
        ),
        (
            CollectionQuerySelection {
                packs: vec![PackId::LaLojban],
                disabled: Vec::new(),
                profile: None,
            },
            "not_selected",
        ),
    ] {
        let mut policy_engine = CollectionSearchEngine::open(&root, policy).unwrap();
        let local = policy_engine
            .execute(DictionaryRequest::Resolve {
                target: RelationTarget::Local(LocalRef::Entry(
                    vlajmaji_lexicon::EntityId::new("entry-a-farvi").unwrap(),
                )),
            })
            .unwrap();
        let vlajmaji_search::CollectionQueryResult::Resolve(local) = local.result else {
            panic!("wrong operation result");
        };
        assert_eq!(
            local.status,
            vlajmaji_search::ReferenceStatus::CatalogueOnly
        );
        assert_eq!(local.operational_reason.as_deref(), Some(expected_reason));

        let catalog = policy_engine
            .execute(DictionaryRequest::Resolve {
                target: catalog_target.clone(),
            })
            .unwrap();
        let vlajmaji_search::CollectionQueryResult::Resolve(catalog) = catalog.result else {
            panic!("wrong operation result");
        };
        assert_eq!(
            catalog.status,
            vlajmaji_search::ReferenceStatus::CatalogueOnly
        );
        assert_eq!(catalog.operational_reason.as_deref(), Some(expected_reason));
    }

    let local = engine
        .execute(DictionaryRequest::Resolve {
            target: RelationTarget::Local(LocalRef::Entry(
                vlajmaji_lexicon::EntityId::new("entry-a-farvi").unwrap(),
            )),
        })
        .unwrap();
    let vlajmaji_search::CollectionQueryResult::Resolve(local) = local.result else {
        panic!("wrong operation result");
    };
    assert_eq!(
        local.status,
        vlajmaji_search::ReferenceStatus::CatalogueOnly
    );
    assert_eq!(local.operational_reason.as_deref(), Some("removed"));

    let catalog = engine
        .execute(DictionaryRequest::Resolve {
            target: catalog_target,
        })
        .unwrap();
    let vlajmaji_search::CollectionQueryResult::Resolve(catalog) = catalog.result else {
        panic!("wrong operation result");
    };
    assert_eq!(
        catalog.status,
        vlajmaji_search::ReferenceStatus::CatalogueOnly
    );
    assert_eq!(catalog.operational_reason.as_deref(), Some("removed"));

    fs::remove_dir_all(root).unwrap();
}
