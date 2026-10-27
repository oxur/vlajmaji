// Prospective SOURCE ONLY. A legitimate strictly reconstructed changed state,
// never unauthenticated SQL tamper, separates stable lexical identity from detail.
fn context_checks(base: &Path) {
    let original = draft(fixture::complete_fixture().unwrap());
    let context = original.metadata.contexts.iter().find(|c| matches!(c.value, Presence::Value(_))).unwrap();
    let context_id = context.id.clone();
    let assertion = original.metadata.assertions.iter().find(|a| a.id == context.assertion).unwrap();
    let reference = key(&original, assertion);
    let mut changed = original.clone();
    changed.metadata.contexts.iter_mut().find(|c| c.id == context_id).unwrap().value = Presence::Null;
    assert_ne!(inventory(&original), inventory(&changed));
    assert_eq!(reference, key(&changed, changed.metadata.assertions.iter().find(|a| a.id == reference.assertion_id).unwrap()));
    let before_path = base.join("context-before-store");
    let after_path = base.join("context-after-store");
    {
        let model = Dictionary::from_document(original.clone()).unwrap();
        drop(vlajmaji_store::DictionaryStore::create(&before_path, &model).unwrap());
    }
    {
        let model = Dictionary::from_document(changed.clone()).unwrap();
        drop(vlajmaji_store::DictionaryStore::create(&after_path, &model).unwrap());
    }
    let before = {
        let engine = DictionaryCompactSearchEngine::open(&before_path).unwrap();
        let response = engine.detail(&reference, None).unwrap();
        assert_eq!(response.status, ReferenceStatus::Resolved);
        assert_eq!(response.hit.as_ref().unwrap().contexts.iter().find(|c| c.id == context_id), original.metadata.contexts.iter().find(|c| c.id == context_id));
        response.snapshot_sha256
    };
    {
        let engine = DictionaryCompactSearchEngine::open(&after_path).unwrap();
        let response = engine.detail(&reference, None).unwrap();
        assert_eq!(response.status, ReferenceStatus::Resolved);
        assert_eq!(response.detail_ref, reference);
        assert_ne!(response.snapshot_sha256, before);
        assert_eq!(response.hit.as_ref().unwrap().contexts.iter().find(|c| c.id == context_id), changed.metadata.contexts.iter().find(|c| c.id == context_id));
        assert_eq!(engine.detail(&reference, Some(&before)).unwrap_err().report().code, "stale_snapshot");
    }
    println!("PASS legitimate changed attached context: unchanged lexical key, current complete detail, stale observed snapshot");

    // A different valid lexical body is the opposite case: same IDs/capture,
    // but the old syntactically valid digest must never select the new body.
    let mut lexical = original.clone();
    let changed_assertion = lexical.metadata.assertions.iter_mut()
        .find(|a| a.id == reference.assertion_id).unwrap();
    changed_assertion.native_sense_id = Presence::Value("independent-changed-lexical-body".into());
    assert_ne!(&*changed_assertion, assertion);
    let new_reference = key(&lexical, lexical.metadata.assertions.iter()
        .find(|a| a.id == reference.assertion_id).unwrap());
    assert_eq!(new_reference.entry_id, reference.entry_id);
    assert_eq!(new_reference.assertion_id, reference.assertion_id);
    assert_eq!(new_reference.occurrence, reference.occurrence);
    assert_eq!(new_reference.entry_sha256, reference.entry_sha256);
    assert_ne!(new_reference.assertion_sha256, reference.assertion_sha256);
    let lexical_path = base.join("changed-lexical-body-store");
    {
        let model = Dictionary::from_document(lexical.clone()).unwrap();
        drop(vlajmaji_store::DictionaryStore::create(&lexical_path, &model).unwrap());
    }
    {
        let engine = DictionaryCompactSearchEngine::open(&lexical_path).unwrap();
        assert_eq!(engine.detail(&reference, Some(&before)).unwrap_err().report().code, "stale_snapshot");
        let old = engine.detail(&reference, None).unwrap();
        assert_eq!(old.status, ReferenceStatus::NoMatch);
        assert!(old.hit.is_none() && old.current_eligible.is_none());
        let new = engine.detail(&new_reference, None).unwrap();
        assert_eq!(new.status, ReferenceStatus::Resolved);
        assert_eq!(&new.hit.as_ref().unwrap().assertion, lexical.metadata.assertions.iter()
            .find(|a| a.id == reference.assertion_id).unwrap());
    }
    println!("PASS valid changed lexical body: pre-input digest changes, old key NoMatch/null, new key resolves without retargeting");
}
