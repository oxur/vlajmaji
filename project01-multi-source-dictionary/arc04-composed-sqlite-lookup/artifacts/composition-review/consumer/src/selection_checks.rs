// Prospective SOURCE ONLY; exact expected IDs are frozen from each original input.
fn selection_checks(root: &Path, docs: &[UnverifiedDictionaryDocument]) {
    for (pack, document) in PACKS.iter().zip(docs) {
        let expected: std::collections::BTreeSet<_> = document.metadata.assertions.iter()
            .filter(|a| matches!(a.eligibility, Eligibility::Lexical))
            .map(|a| a.id.to_string()).collect();
        let mut engine = CollectionCompactSearchEngine::open(root, CollectionQuerySelection {
            packs: vec![*pack], disabled: vec![], profile: None,
        }).unwrap();
        let response = exact(engine.execute(query(None, None)).unwrap());
        let actual: std::collections::BTreeSet<_> = response.groups.iter()
            .flat_map(|g| g.hits.iter().map(|h| h.assertion.id.to_string())).collect();
        assert_eq!(actual, expected, "{pack:?}");
        assert_eq!(response.totals.occurrences as usize, expected.len());
    }
    let (cursor, snapshot) = {
        let mut engine = selected(root);
        let cursor = exact(engine.execute(query(Some(1), None)).unwrap()).next_cursor.unwrap();
        let reference = key(&docs[0], &docs[0].metadata.assertions[0]);
        let snapshot = engine.detail(&reference, None).unwrap().response.snapshot_sha256;
        (cursor, snapshot)
    };
    let reference = key(&docs[0], &docs[0].metadata.assertions[0]);
    let mut disabled = CollectionCompactSearchEngine::open(root, CollectionQuerySelection {
        packs: PACKS.to_vec(), disabled: PACKS.to_vec(), profile: None,
    }).unwrap();
    assert_eq!(disabled.execute(query(Some(1), Some(cursor))).unwrap_err().report().category, "stale_cursor");
    assert!(matches!(disabled.execute(query(None, None)), Err(CollectionQueryError::NoEnabledPacks)));
    assert_eq!(disabled.detail(&reference, Some(&snapshot)).unwrap_err().report_for("lookup_detail").category, "stale_snapshot");
    let response = disabled.detail(&reference, None).unwrap();
    assert_eq!(response.response.status, ReferenceStatus::CatalogueOnly);
    assert_eq!(response.response.operational_reason.as_deref(), Some("disabled"));
    assert!(response.response.hit.is_none());
    println!("PASS each sole selected original pack exact IDs (including distinct JB/La); disabled stale cursor/snapshot before fresh no-enabled and metadata-only detail");
}

fn remove_all_checks(base: &Path, docs: &[UnverifiedDictionaryDocument], sealed: &[(PackId, Vec<u8>, Vec<u8>)]) {
    let root = base.join("remove-all-catalogue");
    let request = CollectionRequest::new(sealed.iter().map(|(pack_id,release,seal)| CollectionInput::SealedBytes {
        pack_id:*pack_id,release:Cow::Borrowed(release),seal:Cow::Borrowed(seal),
    }).collect(),CollectionSelection::Packs(PACKS.to_vec()),true).unwrap();
    let initial = CollectionCatalogue::install(&root, request).unwrap();
    assert_eq!(initial.local_revision, 1);
    let reference = key(&docs[0], &docs[0].metadata.assertions[0]);
    let (cursor, snapshot) = {
        let mut engine = selected(&root);
        let cursor = exact(engine.execute(query(Some(1), None)).unwrap()).next_cursor.unwrap();
        let snapshot = engine.detail(&reference, None).unwrap().response.snapshot_sha256;
        (cursor, snapshot)
    };
    let removed = CollectionCatalogue::remove(&root, &PACKS, 1).unwrap();
    assert_eq!(removed.local_revision, 2);
    assert!(removed.generation.is_none());
    assert!(CollectionCatalogue::open(&root).unwrap().generation().unwrap().is_none());
    let before = sha(&std::fs::read(root.join("catalogue.sqlite3")).unwrap());
    let mut engine = selected(&root);
    assert_eq!(engine.execute(query(Some(1), Some(cursor))).unwrap_err().report().category, "stale_cursor");
    assert!(matches!(engine.execute(query(None, None)), Err(CollectionQueryError::UnavailablePack { reason, .. }) if reason == "removed"));
    assert_eq!(engine.detail(&reference, Some(&snapshot)).unwrap_err().report_for("lookup_detail").category, "stale_snapshot");
    let detail = engine.detail(&reference, None).unwrap();
    assert_eq!(detail.response.status, ReferenceStatus::CatalogueOnly);
    assert_eq!(detail.response.operational_reason.as_deref(), Some("removed"));
    assert!(detail.response.hit.is_none());
    drop(engine);
    assert_eq!(sha(&std::fs::read(root.join("catalogue.sqlite3")).unwrap()), before);
    println!("PASS real nonempty remove-all null head: prior cursor/snapshot stale before unavailable, retained removed detail without fallback; read-only bytes unchanged");
}
