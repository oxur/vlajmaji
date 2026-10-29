// Prospective SOURCE ONLY: original stable lexical key vs observed operational snapshot.
fn history_checks(root: &Path, docs: &[UnverifiedDictionaryDocument], sealed: &[(PackId, Vec<u8>, Vec<u8>)]) {
    let assertion = docs[3].metadata.assertions.iter()
        .find(|a| matches!(a.eligibility, Eligibility::Lexical)).unwrap();
    let reference = key(&docs[3], assertion); // Original input, not a projected candidate.
    let (snapshot1, cursor1) = {
        let mut engine = selected(root);
        let detail = engine.detail(&reference, None).unwrap();
        let cursor = exact(engine.execute(query(Some(1), None)).unwrap()).next_cursor.unwrap();
        (detail.response.snapshot_sha256, cursor)
    };
    let rebuilt = CollectionCatalogue::rebuild(root, 1).unwrap();
    assert_eq!(rebuilt.local_revision, 2);
    assert_owned_store_path(root, &rebuilt.generation.as_ref().unwrap().store_path);
    let (snapshot2, cursor2) = {
        let mut engine = selected(root);
        assert_eq!(engine.detail(&reference, None).unwrap().response.status, ReferenceStatus::Resolved);
        assert_eq!(engine.detail(&reference, Some(&snapshot1)).unwrap_err().report_for("lookup_detail").category, "stale_snapshot");
        assert_eq!(engine.execute(query(Some(1), Some(cursor1))).unwrap_err().report().category, "stale_cursor");
        let detail = engine.detail(&reference, None).unwrap();
        let cursor = exact(engine.execute(query(Some(1), None)).unwrap()).next_cursor.unwrap();
        (detail.response.snapshot_sha256, cursor)
    };
    let removed = CollectionCatalogue::remove(root, &[PackId::Sutysisku], 2).unwrap();
    assert_eq!(removed.local_revision, 3);
    assert_owned_store_path(root, &removed.generation.as_ref().unwrap().store_path);
    {
        let mut engine = selected(root);
        assert_eq!(engine.execute(query(Some(1), Some(cursor2))).unwrap_err().report().category, "stale_cursor");
        assert!(matches!(engine.execute(query(None, None)), Err(CollectionQueryError::UnavailablePack { reason, .. }) if reason == "removed"));
        assert_eq!(engine.detail(&reference, Some(&snapshot2)).unwrap_err().report_for("lookup_detail").category, "stale_snapshot");
        let detail = engine.detail(&reference, None).unwrap();
        assert_eq!(detail.response.status, ReferenceStatus::CatalogueOnly);
        assert_eq!(detail.response.operational_reason.as_deref(), Some("removed"));
        assert!(detail.response.hit.is_none());
        // An unrelated unavailable selected pack must not block a valid scoped reference.
        let jb = &docs[0].metadata.assertions[0];
        let response = engine.execute(DictionaryRequest::Resolve {
            target: RelationTarget::Local(LocalRef::Assertion(jb.id.clone())),
        }).unwrap();
        let CollectionCompactQueryResult::Resolve(response) = response.result else { panic!("expected reference") };
        assert_eq!(response.status, ReferenceStatus::Resolved);
    }
    let (pack_id, release, seal) = &sealed[3];
    assert_eq!(*pack_id, PackId::Sutysisku);
    let request = CollectionRequest::new(vec![CollectionInput::SealedBytes {
        pack_id: *pack_id, release: Cow::Borrowed(release), seal: Cow::Borrowed(seal),
    }], CollectionSelection::Packs(vec![*pack_id]), true).unwrap();
    let restored = CollectionCatalogue::reinstall(root, request, &sha(release), 3).unwrap();
    assert_eq!(restored.local_revision, 4);
    assert_owned_store_path(root, &restored.generation.as_ref().unwrap().store_path);
    {
        let mut engine = selected(root);
        let detail = engine.detail(&reference, None).unwrap();
        assert_eq!(detail.response.status, ReferenceStatus::Resolved);
        assert_eq!(detail.response.hit.unwrap().assertion, *assertion);
        assert_eq!(detail.response.detail_ref, reference);
        assert_eq!(exact(engine.execute(query(None, None)).unwrap()).totals.occurrences, 5);
    }
    println!("PASS stable original key after rebuild/remove/exact restore; stale cursor/snapshot before unavailable; unrelated scoped reference; final all4 active revision4");
}
