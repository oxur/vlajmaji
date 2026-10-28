//! Targeted retained-metadata projection contracts.

use std::{
    borrow::Cow,
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use vlajmaji_lexicon::{
    CatalogTarget, EntityId, LocalRef, PackId, PackPayloadKind, RelationTarget, SourceId,
    seal_release,
};
use vlajmaji_store::{
    CollectionCatalogue, CollectionInput, CollectionRequest, CollectionSelection,
};

#[path = "../../lexicon/examples/support/dictionary_fixture.rs"]
mod fixture_support;

static SCRATCH_ID: AtomicU64 = AtomicU64::new(0);

fn scratch() -> PathBuf {
    let base = std::env::temp_dir().join(format!(
        "vlajmaji-reference-projection-{}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
        SCRATCH_ID.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&base).unwrap();
    base.join("catalogue")
}

#[test]
fn test_reference_projection_assertion_keeps_target_links_and_skips_unrelated_bodies() {
    let root = scratch();
    let dictionary = fixture_support::complete_fixture().unwrap();
    let document = dictionary.document();
    let target_assertion = document.metadata.assertions[0].id.clone();
    let sealed = seal_release(
        PackId::LensiskuJbotci,
        PackPayloadKind::Synthetic,
        &dictionary,
    )
    .unwrap();
    let request = CollectionRequest::new(
        vec![CollectionInput::SealedBytes {
            pack_id: PackId::LensiskuJbotci,
            release: Cow::Borrowed(sealed.release.as_slice()),
            seal: Cow::Borrowed(sealed.seal.as_slice()),
        }],
        CollectionSelection::Packs(vec![PackId::LensiskuJbotci]),
        true,
    )
    .unwrap();
    let report = CollectionCatalogue::register(&root, request).unwrap();
    let digest = report.inputs[0].release_sha256.as_deref().unwrap();
    let catalogue = CollectionCatalogue::open(&root).unwrap();
    let projection = catalogue
        .reference_projection(
            PackId::LensiskuJbotci,
            digest,
            &RelationTarget::Local(LocalRef::Assertion(target_assertion.clone())),
        )
        .unwrap();

    assert!(
        projection
            .entities
            .iter()
            .any(|entity| { entity.role == "assertion" && entity.id == target_assertion.as_str() })
    );
    let assertion = projection
        .entities
        .iter()
        .find(|entity| entity.role == "assertion" && entity.id == target_assertion.as_str())
        .unwrap();
    assert!(!assertion.entry_ids.is_empty());
    for entry_id in &assertion.entry_ids {
        assert!(
            projection
                .entities
                .iter()
                .any(|entity| { entity.role == "entry" && entity.id.as_str() == entry_id })
        );
    }
    assert!(
        projection
            .body_sha256
            .contains_key(&format!("assertion:{}", target_assertion.as_str()))
    );
    assert!(projection.entities.iter().all(|entity| {
        entity.role == "assertion"
            || entity.role == "entry"
            || entity.id == target_assertion.as_str()
    }));
    assert!(
        projection
            .releases
            .iter()
            .any(|release| release.matching_assertion_ids.contains(&target_assertion))
    );

    let form = document.metadata.forms.first().unwrap();
    let form_projection = catalogue
        .reference_projection(
            PackId::LensiskuJbotci,
            digest,
            &RelationTarget::Local(LocalRef::Form(form.id.clone())),
        )
        .unwrap();
    assert!(
        form_projection
            .entities
            .iter()
            .any(|entity| { entity.role == "form" && entity.id == form.id.as_str() })
    );
    assert!(
        form_projection
            .entities
            .iter()
            .any(|entity| { entity.role == "entry" && entity.id == form.entry.as_str() })
    );
    for assertion in document
        .metadata
        .assertions
        .iter()
        .filter(|assertion| assertion.entry == form.entry)
    {
        assert!(
            form_projection
                .entities
                .iter()
                .any(|entity| { entity.role == "assertion" && entity.id == assertion.id.as_str() })
        );
    }

    let editorial = document.metadata.editorial_senses.first().unwrap();
    let editorial_projection = catalogue
        .reference_projection(
            PackId::LensiskuJbotci,
            digest,
            &RelationTarget::Local(LocalRef::EditorialSense(editorial.id.clone())),
        )
        .unwrap();
    assert!(
        editorial_projection.entities.iter().any(|entity| {
            entity.role == "editorial_sense" && entity.id == editorial.id.as_str()
        })
    );
    for assertion_id in &editorial.assertions {
        let assertion = document
            .metadata
            .assertions
            .iter()
            .find(|assertion| &assertion.id == assertion_id)
            .unwrap();
        assert!(
            editorial_projection
                .entities
                .iter()
                .any(|entity| { entity.role == "assertion" && entity.id == assertion_id.as_str() })
        );
        assert!(
            editorial_projection
                .entities
                .iter()
                .any(|entity| { entity.role == "entry" && entity.id == assertion.entry.as_str() })
        );
    }
    drop(catalogue);
    let _ = fs::remove_dir_all(root.parent().unwrap());
}

#[test]
fn test_reference_projection_catalog_release_keeps_requested_source_occurrence() {
    let root = scratch();
    let dictionary = fixture_support::complete_fixture().unwrap();
    let document = dictionary.document();
    let release = document
        .metadata
        .releases
        .iter()
        .find(|release| release.id.as_str() == "release-a")
        .unwrap();
    let assertion = document
        .metadata
        .assertions
        .iter()
        .find(|assertion| release.assertions.contains(&assertion.id))
        .unwrap();
    let assertion_id = assertion.id.clone();
    let occurrence = assertion.occurrence.clone();
    let sealed = seal_release(
        PackId::LensiskuJbotci,
        PackPayloadKind::Synthetic,
        &dictionary,
    )
    .unwrap();
    let request = CollectionRequest::new(
        vec![CollectionInput::SealedBytes {
            pack_id: PackId::LensiskuJbotci,
            release: Cow::Borrowed(sealed.release.as_slice()),
            seal: Cow::Borrowed(sealed.seal.as_slice()),
        }],
        CollectionSelection::Packs(vec![PackId::LensiskuJbotci]),
        true,
    )
    .unwrap();
    let report = CollectionCatalogue::register(&root, request).unwrap();
    let digest = report.inputs[0].release_sha256.as_deref().unwrap();
    let catalogue = CollectionCatalogue::open(&root).unwrap();
    let projection = catalogue
        .reference_projection(
            PackId::LensiskuJbotci,
            digest,
            &RelationTarget::Catalog(CatalogTarget {
                source: SourceId::new("source-a").unwrap(),
                release: EntityId::new("release-a").unwrap(),
                native_entry_id: None,
                native_assertion_id: None,
                occurrence: Some(occurrence),
            }),
        )
        .unwrap();

    assert!(
        projection
            .entities
            .iter()
            .any(|entity| { entity.role == "assertion" && entity.id == assertion_id.as_str() })
    );
    assert!(
        projection
            .entities
            .iter()
            .any(|entity| { entity.role == "entry" && entity.id == assertion.entry.as_str() })
    );
    assert_eq!(projection.releases.len(), 1);
    assert_eq!(projection.releases[0].summary.id, "release-a");
    assert_eq!(
        projection.releases[0].matching_assertion_ids,
        vec![assertion_id]
    );
    drop(catalogue);
    let _ = fs::remove_dir_all(root.parent().unwrap());
}

#[test]
fn test_reference_projection_skips_unrelated_malformed_body_and_reports_selected_error() {
    let root = scratch();
    let dictionary = fixture_support::complete_fixture().unwrap();
    let document = dictionary.document();
    let target_assertion = document.metadata.assertions[0].id.clone();
    let expected_release_ids = document
        .metadata
        .releases
        .iter()
        .filter(|release| release.assertions.contains(&target_assertion))
        .map(|release| release.id.to_string())
        .collect::<std::collections::BTreeSet<_>>();
    let unrelated_release_id = document
        .metadata
        .releases
        .iter()
        .find(|release| !release.assertions.contains(&target_assertion))
        .expect("fixture includes a release unrelated to the target assertion")
        .id
        .to_string();
    let sealed = seal_release(
        PackId::LensiskuJbotci,
        PackPayloadKind::Synthetic,
        &dictionary,
    )
    .unwrap();
    let request = CollectionRequest::new(
        vec![CollectionInput::SealedBytes {
            pack_id: PackId::LensiskuJbotci,
            release: Cow::Borrowed(sealed.release.as_slice()),
            seal: Cow::Borrowed(sealed.seal.as_slice()),
        }],
        CollectionSelection::Packs(vec![PackId::LensiskuJbotci]),
        true,
    )
    .unwrap();
    let report = CollectionCatalogue::register(&root, request).unwrap();
    let digest = report.inputs[0]
        .release_sha256
        .as_deref()
        .unwrap()
        .to_owned();
    drop(report);

    let database = root.join("catalogue.sqlite3");
    let connection = rusqlite::Connection::open(&database).unwrap();
    let original: Vec<u8> = connection
        .query_row(
            "SELECT descriptor_json FROM registrations_v2 WHERE pack_id=?1 AND release_sha256=?2",
            rusqlite::params![PackId::LensiskuJbotci.as_str(), digest],
            |row| row.get(0),
        )
        .unwrap();
    let mut descriptor: serde_json::Value = serde_json::from_slice(&original).unwrap();
    let unrelated_release = descriptor["source_metadata"]["releases"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|release| release["id"] == unrelated_release_id)
        .unwrap();
    unrelated_release["assertions"] = serde_json::Value::Array(
        (0..1024)
            .map(|index| serde_json::Value::String(format!("unrelated-sentinel-{index:04}")))
            .collect(),
    );
    descriptor["entities"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!({
        "role": "assertion",
        "id": "unrelated-sentinel",
        "entry_ids": [],
        "assertion_ids": [],
        "source_ids": [],
        "occurrence": null,
        "spelling": 7
        }));
    let sentinel_bytes = serde_json::to_vec(&descriptor).unwrap();
    connection
        .execute(
            "UPDATE registrations_v2 SET descriptor_json=?1 WHERE pack_id=?2 AND release_sha256=?3",
            rusqlite::params![sentinel_bytes, PackId::LensiskuJbotci.as_str(), digest],
        )
        .unwrap();
    drop(connection);

    let catalogue = CollectionCatalogue::open(&root).unwrap();
    let projection = catalogue
        .reference_projection(
            PackId::LensiskuJbotci,
            &digest,
            &RelationTarget::Local(LocalRef::Assertion(target_assertion.clone())),
        )
        .unwrap();
    assert!(
        projection
            .entities
            .iter()
            .any(|entity| { entity.role == "assertion" && entity.id == target_assertion.as_str() })
    );
    assert_eq!(
        projection
            .releases
            .iter()
            .map(|release| release.summary.id.clone())
            .collect::<std::collections::BTreeSet<_>>(),
        expected_release_ids
    );
    assert!(
        projection
            .releases
            .iter()
            .all(|release| release.matching_assertion_ids == vec![target_assertion.clone()])
    );
    drop(catalogue);

    let mut selected_bad: serde_json::Value = serde_json::from_slice(&sentinel_bytes).unwrap();
    let target = selected_bad["entities"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|entity| entity["role"] == "assertion" && entity["id"] == target_assertion.as_str())
        .unwrap();
    target["entry_ids"] = serde_json::json!(7);
    let selected_bad_bytes = serde_json::to_vec(&selected_bad).unwrap();
    let connection = rusqlite::Connection::open(&database).unwrap();
    connection
        .execute(
            "UPDATE registrations_v2 SET descriptor_json=?1 WHERE pack_id=?2 AND release_sha256=?3",
            rusqlite::params![selected_bad_bytes, PackId::LensiskuJbotci.as_str(), digest],
        )
        .unwrap();
    drop(connection);
    let catalogue = CollectionCatalogue::open(&root).unwrap();
    assert!(matches!(
        catalogue.reference_projection(
            PackId::LensiskuJbotci,
            &digest,
            &RelationTarget::Local(LocalRef::Assertion(target_assertion.clone()))
        ),
        Err(vlajmaji_store::CollectionError::ReferenceProjection { .. })
    ));
    drop(catalogue);

    let mut trailing = original.clone();
    trailing.extend_from_slice(b" trailing");
    let connection = rusqlite::Connection::open(&database).unwrap();
    connection
        .execute(
            "UPDATE registrations_v2 SET descriptor_json=?1 WHERE pack_id=?2 AND release_sha256=?3",
            rusqlite::params![trailing, PackId::LensiskuJbotci.as_str(), digest],
        )
        .unwrap();
    drop(connection);
    let catalogue = CollectionCatalogue::open(&root).unwrap();
    assert!(matches!(
        catalogue.reference_projection(
            PackId::LensiskuJbotci,
            &digest,
            &RelationTarget::Local(LocalRef::Assertion(target_assertion))
        ),
        Err(vlajmaji_store::CollectionError::ReferenceProjection { .. })
    ));
    drop(catalogue);
    let _ = fs::remove_dir_all(root.parent().unwrap());
}
