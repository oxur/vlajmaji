//! Managed compact projection precedence controls against a tiny Synthetic catalogue.

#[path = "../../lexicon/examples/support/dictionary_fixture.rs"]
mod fixture;

use std::{
    borrow::Cow,
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use vlajmaji_lexicon::{Dictionary, PackId, PackPayloadKind, seal_release};
use vlajmaji_search::{
    CollectionCompactQueryResult, CollectionCompactSearchEngine, CollectionQueryError,
    CollectionQuerySelection, DictionaryExactOptions, DictionaryRequest,
};
use vlajmaji_store::{
    CollectionCatalogue, CollectionInput, CollectionRequest, CollectionSelection,
};

fn fresh_root(label: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "vlajmaji-s05-compact-projection-{label}-{}-{stamp}",
        std::process::id()
    ))
}

fn install_fixture(root: &Path) -> vlajmaji_store::CollectionGeneration {
    let mut document = fixture::complete_fixture().unwrap().into_document();
    // The fixture includes a derived legacy archive, which pack installation
    // intentionally rejects; preserve the rest of the typed fixture model.
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
    CollectionCatalogue::install(root, request)
        .unwrap()
        .generation
        .unwrap()
}

fn selection() -> CollectionQuerySelection {
    CollectionQuerySelection {
        packs: vec![PackId::Llg],
        disabled: vec![],
        profile: None,
    }
}

#[test]
fn test_compact_detail_stale_snapshot_precedes_selected_bad_payload_and_restores() {
    let root = fresh_root("compact-detail-payload-restoration");
    let generation = install_fixture(&root);
    let database = generation.store_path.join("dictionary.sqlite3");
    let original_database = fs::read(&database).unwrap();

    let mut original = CollectionCompactSearchEngine::open(&root, selection()).unwrap();
    let original_response = original
        .execute(DictionaryRequest::Exact {
            spelling: "farvi".into(),
            options: DictionaryExactOptions::default(),
        })
        .unwrap();
    let CollectionCompactQueryResult::Exact(original_exact) = original_response.result else {
        panic!("expected compact exact response");
    };
    let detail_ref = original_exact.groups[0].hits[0].detail_ref.clone();
    let original_binding = original.selection().binding_sha256.clone();
    drop(original);

    let mutation = Command::new("sqlite3")
        .arg(&database)
        .arg("UPDATE metadata SET value='999' WHERE key='schema_version';")
        .output()
        .unwrap();
    assert!(
        mutation.status.success(),
        "unsupported-version mutation failed: {}",
        String::from_utf8_lossy(&mutation.stderr)
    );
    let mut damaged = CollectionCompactSearchEngine::open(&root, selection()).unwrap();
    let damaged_binding = damaged.selection().binding_sha256.clone();
    assert_ne!(damaged_binding, original_binding);
    assert!(matches!(
        damaged.detail(&detail_ref, Some(&original_binding)),
        Err(CollectionQueryError::Projection(
            vlajmaji_search::DictionaryProjectionError::StaleSnapshot { .. }
        ))
    ));
    let selected_error = damaged.detail(&detail_ref, Some(&damaged_binding));
    assert!(
        matches!(
            selected_error,
            Err(CollectionQueryError::Query(
                vlajmaji_search::DictionaryQueryError::Snapshot(
                    vlajmaji_store::DictionarySnapshotError::Store(
                        vlajmaji_store::DictionaryStoreError::InvalidStore {
                            reason: vlajmaji_store::DictionaryStoreInvalidity::SchemaVersion,
                            ..
                        }
                    )
                )
            ))
        ),
        "selected malformed payload should retain its version cause: {selected_error:?}"
    );
    drop(damaged);

    fs::write(&database, original_database).unwrap();
    let mut restored = CollectionCompactSearchEngine::open(&root, selection()).unwrap();
    assert_eq!(restored.selection().binding_sha256, original_binding);
    let restored_detail = restored
        .detail(&detail_ref, Some(&original_binding))
        .unwrap();
    assert_eq!(
        restored_detail.response.status,
        vlajmaji_search::ReferenceStatus::Resolved
    );
    assert!(restored_detail.response.hit.is_some());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn test_compact_detail_reports_removed_history_without_opening_payload() {
    let root = fresh_root("compact-detail-removed-history");
    let generation = install_fixture(&root);
    let mut before = CollectionCompactSearchEngine::open(&root, selection()).unwrap();
    let exact = before
        .execute(DictionaryRequest::Exact {
            spelling: "farvi".into(),
            options: DictionaryExactOptions::default(),
        })
        .unwrap();
    let CollectionCompactQueryResult::Exact(exact) = exact.result else {
        panic!("expected compact exact response");
    };
    let detail_ref = exact.groups[0].hits[0].detail_ref.clone();
    let old_binding = before.selection().binding_sha256.clone();
    drop(before);

    let removed = CollectionCatalogue::remove(&root, &[PackId::Llg], 1).unwrap();
    assert_eq!(removed.local_revision, 2);
    assert_eq!(removed.generation, None);
    assert_eq!(removed.outcome, "removed");

    let mut after = CollectionCompactSearchEngine::open(&root, selection()).unwrap();
    let current_binding = after.selection().binding_sha256.clone();
    assert_ne!(current_binding, old_binding);
    assert!(matches!(
        after.detail(&detail_ref, Some(&old_binding)),
        Err(CollectionQueryError::Projection(
            vlajmaji_search::DictionaryProjectionError::StaleSnapshot { .. }
        ))
    ));
    let metadata_only = after.detail(&detail_ref, Some(&current_binding)).unwrap();
    assert_eq!(
        metadata_only.response.status,
        vlajmaji_search::ReferenceStatus::CatalogueOnly
    );
    assert_eq!(
        metadata_only.response.operational_reason.as_deref(),
        Some("removed")
    );
    assert!(metadata_only.response.hit.is_none());
    assert_eq!(metadata_only.response.current_eligible, None);
    assert_eq!(metadata_only.catalogue_matches.len(), 1);
    assert_eq!(metadata_only.snapshot.local_revision, 2);
    assert!(generation.store_path.exists());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn test_compact_detail_key_survives_rebuild_remove_and_exact_reinstall() {
    let root = fresh_root("compact-detail-stable-key-lifecycle");
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
    let installed = CollectionCatalogue::install(&root, request).unwrap();
    let initial_generation = installed.generation.unwrap().generation_id;
    let registered_release = installed.inputs[0]
        .release_sha256
        .clone()
        .expect("normalized install records its release digest");

    let mut initial = CollectionCompactSearchEngine::open(&root, selection()).unwrap();
    let first_page = initial
        .execute(DictionaryRequest::Exact {
            spelling: "farvi".into(),
            options: DictionaryExactOptions {
                page_size: Some(1),
                ..Default::default()
            },
        })
        .unwrap();
    let CollectionCompactQueryResult::Exact(first_page) = first_page.result else {
        panic!("expected compact Exact");
    };
    let detail_ref = first_page.groups[0].hits[0].detail_ref.clone();
    let detail_hit = initial
        .detail(&detail_ref, None)
        .unwrap()
        .response
        .hit
        .unwrap();
    let installed_binding = initial.selection().binding_sha256.clone();
    let installed_cursor = first_page.next_cursor.unwrap();
    drop(initial);

    let rebuilt = CollectionCatalogue::rebuild(&root, installed.local_revision).unwrap();
    let rebuilt_generation = rebuilt.generation.unwrap().generation_id;
    assert_ne!(rebuilt_generation, initial_generation);
    let mut after_rebuild = CollectionCompactSearchEngine::open(&root, selection()).unwrap();
    let rebuilt_binding = after_rebuild.selection().binding_sha256.clone();
    assert_ne!(rebuilt_binding, installed_binding);
    assert!(matches!(
        after_rebuild.detail(&detail_ref, Some(&installed_binding)),
        Err(CollectionQueryError::Projection(
            vlajmaji_search::DictionaryProjectionError::StaleSnapshot { .. }
        ))
    ));
    assert!(matches!(
        after_rebuild.execute(DictionaryRequest::Exact {
            spelling: "farvi".into(),
            options: DictionaryExactOptions {
                page_size: Some(1),
                cursor: Some(installed_cursor),
                ..Default::default()
            },
        }),
        Err(CollectionQueryError::StaleCursor)
    ));
    assert_eq!(
        after_rebuild
            .detail(&detail_ref, Some(&rebuilt_binding))
            .unwrap()
            .response
            .hit,
        Some(detail_hit.clone())
    );
    drop(after_rebuild);

    let removed =
        CollectionCatalogue::remove(&root, &[PackId::Llg], rebuilt.local_revision).unwrap();
    assert_eq!(removed.generation, None);
    let mut after_remove = CollectionCompactSearchEngine::open(&root, selection()).unwrap();
    let removed_binding = after_remove.selection().binding_sha256.clone();
    assert_ne!(removed_binding, rebuilt_binding);
    assert!(matches!(
        after_remove.detail(&detail_ref, Some(&rebuilt_binding)),
        Err(CollectionQueryError::Projection(
            vlajmaji_search::DictionaryProjectionError::StaleSnapshot { .. }
        ))
    ));
    let removed_detail = after_remove
        .detail(&detail_ref, Some(&removed_binding))
        .unwrap();
    assert_eq!(
        removed_detail.response.status,
        vlajmaji_search::ReferenceStatus::CatalogueOnly
    );
    assert_eq!(
        removed_detail.response.operational_reason.as_deref(),
        Some("removed")
    );
    assert!(removed_detail.response.hit.is_none());
    drop(after_remove);

    let reinstall_request = CollectionRequest::new(
        vec![CollectionInput::SealedBytes {
            pack_id: PackId::Llg,
            release: Cow::Borrowed(&sealed.release),
            seal: Cow::Borrowed(&sealed.seal),
        }],
        CollectionSelection::Packs(vec![PackId::Llg]),
        true,
    )
    .unwrap();
    let reinstalled = CollectionCatalogue::reinstall(
        &root,
        reinstall_request,
        &registered_release,
        removed.local_revision,
    )
    .unwrap();
    assert_eq!(reinstalled.outcome, "reinstalled");
    let mut after_reinstall = CollectionCompactSearchEngine::open(&root, selection()).unwrap();
    let reinstalled_binding = after_reinstall.selection().binding_sha256.clone();
    assert_ne!(reinstalled_binding, removed_binding);
    assert!(matches!(
        after_reinstall.detail(&detail_ref, Some(&removed_binding)),
        Err(CollectionQueryError::Projection(
            vlajmaji_search::DictionaryProjectionError::StaleSnapshot { .. }
        ))
    ));
    let reinstalled_detail = after_reinstall
        .detail(&detail_ref, Some(&reinstalled_binding))
        .unwrap();
    assert_eq!(
        reinstalled_detail.response.status,
        vlajmaji_search::ReferenceStatus::Resolved
    );
    assert_eq!(reinstalled_detail.response.hit, Some(detail_hit));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn test_compact_detail_preserves_selected_missing_and_corrupt_payload_causes() {
    let root = fresh_root("compact-detail-missing-corrupt-payload");
    let generation = install_fixture(&root);
    let database = generation.store_path.join("dictionary.sqlite3");
    let original_database = fs::read(&database).unwrap();
    let mut initial = CollectionCompactSearchEngine::open(&root, selection()).unwrap();
    let exact = initial
        .execute(DictionaryRequest::Exact {
            spelling: "farvi".into(),
            options: DictionaryExactOptions::default(),
        })
        .unwrap();
    let CollectionCompactQueryResult::Exact(exact) = exact.result else {
        panic!("expected compact exact response");
    };
    let detail_ref = exact.groups[0].hits[0].detail_ref.clone();
    let original_binding = initial.selection().binding_sha256.clone();
    drop(initial);

    fs::remove_dir_all(&generation.store_path).unwrap();
    let mut missing = CollectionCompactSearchEngine::open(&root, selection()).unwrap();
    let missing_binding = missing.selection().binding_sha256.clone();
    assert_ne!(missing_binding, original_binding);
    assert!(matches!(
        missing.detail(&detail_ref, Some(&original_binding)),
        Err(CollectionQueryError::Projection(
            vlajmaji_search::DictionaryProjectionError::StaleSnapshot { .. }
        ))
    ));
    let missing_error = missing.detail(&detail_ref, Some(&missing_binding));
    assert!(
        matches!(
            missing_error,
            Err(CollectionQueryError::Query(
                vlajmaji_search::DictionaryQueryError::Snapshot(
                    vlajmaji_store::DictionarySnapshotError::Store(
                        vlajmaji_store::DictionaryStoreError::InvalidStore {
                            reason: vlajmaji_store::DictionaryStoreInvalidity::MissingDatabase,
                            ..
                        }
                    )
                )
            ))
        ),
        "selected missing payload should retain its cause: {missing_error:?}"
    );
    drop(missing);

    fs::create_dir_all(&generation.store_path).unwrap();
    fs::write(&database, b"not a sqlite database").unwrap();
    let mut corrupt = CollectionCompactSearchEngine::open(&root, selection()).unwrap();
    let corrupt_binding = corrupt.selection().binding_sha256.clone();
    assert_ne!(corrupt_binding, original_binding);
    assert!(matches!(
        corrupt.detail(&detail_ref, Some(&original_binding)),
        Err(CollectionQueryError::Projection(
            vlajmaji_search::DictionaryProjectionError::StaleSnapshot { .. }
        ))
    ));
    let corrupt_error = corrupt.detail(&detail_ref, Some(&corrupt_binding));
    assert!(
        matches!(
            corrupt_error,
            Err(CollectionQueryError::Query(
                vlajmaji_search::DictionaryQueryError::Snapshot(
                    vlajmaji_store::DictionarySnapshotError::Store(
                        vlajmaji_store::DictionaryStoreError::Backend(_)
                    )
                )
            ))
        ),
        "selected corrupt payload should retain its store cause: {corrupt_error:?}"
    );
    drop(corrupt);

    fs::write(&database, original_database).unwrap();
    let mut restored = CollectionCompactSearchEngine::open(&root, selection()).unwrap();
    let restored_binding = restored.selection().binding_sha256.clone();
    let detail = restored
        .detail(&detail_ref, Some(&restored_binding))
        .unwrap();
    assert_eq!(
        detail.response.status,
        vlajmaji_search::ReferenceStatus::Resolved
    );
    assert!(detail.response.hit.is_some());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn test_compact_detail_distinguishes_native_role_conflict_from_native_value_no_match() {
    let root = fresh_root("compact-detail-native-role");
    install_fixture(&root);
    let mut engine = CollectionCompactSearchEngine::open(&root, selection()).unwrap();
    let exact = engine
        .execute(DictionaryRequest::Exact {
            spelling: "farvi".into(),
            options: DictionaryExactOptions::default(),
        })
        .unwrap();
    let CollectionCompactQueryResult::Exact(exact) = exact.result else {
        panic!("expected compact exact response");
    };
    let detail_ref = exact
        .groups
        .iter()
        .flat_map(|group| &group.hits)
        .map(|hit| &hit.detail_ref)
        .find(|detail_ref| {
            detail_ref.native_entry_id.is_some() && detail_ref.native_assertion_id.is_some()
        })
        .expect("fixture has a hit with both native identity roles")
        .clone();
    let valid = engine.detail(&detail_ref, None).unwrap();
    assert_eq!(
        valid.response.status,
        vlajmaji_search::ReferenceStatus::Resolved
    );
    assert!(valid.response.hit.is_some());

    let mut wrong_role = detail_ref.clone();
    wrong_role.native_entry_id = detail_ref.native_assertion_id.clone();
    assert!(matches!(
        engine.detail(&wrong_role, None),
        Err(CollectionQueryError::Query(
            vlajmaji_search::DictionaryQueryError::InvalidReference { .. }
        ))
    ));

    let mut unknown_native_value = detail_ref.clone();
    unknown_native_value.native_entry_id = Some("unknown-native-entry-for-control".into());
    let no_match = engine.detail(&unknown_native_value, None).unwrap();
    assert_eq!(
        no_match.response.status,
        vlajmaji_search::ReferenceStatus::NoMatch
    );
    assert!(no_match.response.hit.is_none());
    assert_eq!(no_match.response.current_eligible, None);
    assert!(no_match.catalogue_matches.is_empty());
    fs::remove_dir_all(root).unwrap();
}
