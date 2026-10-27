// Prospective SOURCE ONLY. Expected scopes come from the individual input models.
fn original_reference_checks(root: &Path, input: &[UnverifiedDictionaryDocument]) {
    let mut engine = selected(root);
    let mut observed = 0;
    for doc in input {
        for assertion in &doc.metadata.assertions {
            let expected = key(doc, assertion);
            let release = doc.metadata.releases.iter()
                .find(|r| r.assertions.contains(&assertion.id)).unwrap();
            let target = RelationTarget::Catalog(CatalogTarget {
                source: expected.source.clone(),
                release: release.id.clone(),
                native_entry_id: expected.native_entry_id.clone(),
                native_assertion_id: expected.native_assertion_id.clone(),
                occurrence: Some(assertion.occurrence.clone()),
            });
            let response = engine.execute(DictionaryRequest::Resolve { target: target.clone() }).unwrap();
            let CollectionCompactQueryResult::Resolve(response) = response.result else { panic!("expected resolve") };
            assert_eq!(response.target, target);
            assert_eq!(response.status, ReferenceStatus::Resolved);
            let payload = response.payload.expect("loaded exact immutable native tuple has payload");
            assert_eq!(payload.target, target);
            assert_eq!(payload.confirmed.len(), 1);
            assert!(payload.unverified.is_empty());
            let retained = &payload.confirmed[0];
            assert_eq!(&retained.assertion, assertion);
            assert_eq!(retained.current_eligible, matches!(assertion.eligibility, Eligibility::Lexical));
            assert_eq!(retained.detail_ref.as_ref(), Some(&expected));
            assert!(retained.payload_present);
            assert_eq!(retained.release_ids, vec![release.id.clone()]);
            let local = RelationTarget::Local(LocalRef::Assertion(assertion.id.clone()));
            let response = engine.execute(DictionaryRequest::Resolve { target: local }).unwrap();
            let CollectionCompactQueryResult::Resolve(response) = response.result else { panic!("expected local resolve") };
            assert_eq!(response.status, ReferenceStatus::Resolved);
            assert_eq!(response.payload.unwrap().confirmed[0].assertion, *assertion);
            // Excluded rows remain resolvable and receive complete detail, never lexical promotion.
            if !matches!(assertion.eligibility, Eligibility::Lexical) {
                let detail = engine.detail(&expected, None).unwrap();
                assert_eq!(detail.response.current_eligible, Some(false));
                assert_eq!(detail.response.hit.unwrap().assertion, *assertion);
            }
            observed += 1;
        }
    }
    assert_eq!(observed, 6);
    for (target, status) in [
        (RelationTarget::Unknown, ReferenceStatus::Unknown),
        (RelationTarget::Unresolved("explicit independent unresolved".into()), ReferenceStatus::Unresolved),
    ] {
        let response = engine.execute(DictionaryRequest::Resolve { target: target.clone() }).unwrap();
        let CollectionCompactQueryResult::Resolve(response) = response.result else { panic!("expected explicit status") };
        assert_eq!(response.status, status);
        assert_eq!(response.target, target);
    }
    println!("PASS 6 original Catalog/native tuples + Local assertions, excluded detail, Unknown/Unresolved");
}
