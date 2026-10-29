// Prospective SOURCE ONLY: admission/identity controls on a generation-free registration.
fn metadata_only_checks(base: &Path, input: &[UnverifiedDictionaryDocument], sealed: &[(PackId, Vec<u8>, Vec<u8>)]) {
    let root = base.join("registered-only");
    let request = CollectionRequest::new(sealed.iter().map(|(pack_id, release, seal)|
        CollectionInput::SealedBytes { pack_id: *pack_id, release: Cow::Borrowed(release), seal: Cow::Borrowed(seal) }
    ).collect(), CollectionSelection::Packs(PACKS.to_vec()), true).unwrap();
    let registered = CollectionCatalogue::register(&root, request).unwrap();
    assert!(registered.generation.is_none());
    let before = sha(&std::fs::read(root.join("catalogue.sqlite3")).unwrap());
    let mut engine = selected(&root);
    assert!(matches!(engine.execute(query(None, None)), Err(CollectionQueryError::UnavailablePack { reason, .. }) if reason == "registered_only"));
    for doc in input {
        for assertion in &doc.metadata.assertions {
            let reference = key(doc, assertion);
            let result = engine.detail(&reference, None).unwrap();
            assert_eq!(result.response.status, ReferenceStatus::CatalogueOnly);
            assert_eq!(result.response.operational_reason.as_deref(), Some("registered_only"));
            assert!(result.response.hit.is_none());
            assert!(result.response.current_eligible.is_none());
            assert!(!result.catalogue_matches.is_empty());
            let mut different_body = reference.clone();
            different_body.assertion_sha256 = "0".repeat(64);
            let changed = engine.detail(&different_body, None).unwrap();
            assert_eq!(changed.response.status, ReferenceStatus::NoMatch);
            assert!(changed.response.hit.is_none());
        }
    }
    for target in [RelationTarget::Unknown, RelationTarget::Unresolved("metadata-only unresolved".into())] {
        let response = engine.execute(DictionaryRequest::Resolve { target: target.clone() }).unwrap();
        let CollectionCompactQueryResult::Resolve(response) = response.result else { panic!("expected resolve") };
        let expected = if matches!(target, RelationTarget::Unknown) { ReferenceStatus::Unknown } else { ReferenceStatus::Unresolved };
        assert_eq!(response.status, expected);
        assert!(response.payload.is_none());
    }
    drop(engine);
    let mut disabled = CollectionCompactSearchEngine::open(&root, CollectionQuerySelection {
        packs: PACKS.to_vec(), disabled: PACKS.to_vec(), profile: None,
    }).unwrap();
    assert!(matches!(disabled.execute(query(None, None)), Err(CollectionQueryError::NoEnabledPacks)));
    let reference = key(&input[0], &input[0].metadata.assertions[0]);
    let result = disabled.detail(&reference, None).unwrap();
    assert_eq!(result.response.status, ReferenceStatus::CatalogueOnly);
    assert_eq!(result.response.operational_reason.as_deref(), Some("disabled"));
    drop(disabled);
    assert_eq!(sha(&std::fs::read(root.join("catalogue.sqlite3")).unwrap()), before);
    println!("PASS generation-free registered-only detail/Unknown/Unresolved, disabled precedence and no retarget on changed digest");
}
