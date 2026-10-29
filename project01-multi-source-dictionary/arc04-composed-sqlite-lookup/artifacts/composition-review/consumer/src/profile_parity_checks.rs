// Original issued parity denominator, same existing nonempty four-pack Tiny root.
fn profile_parity_checks(root: &Path, store: &Path, docs: &[UnverifiedDictionaryDocument]) {
    let doc = docs.iter().find(|d| d.metadata.assertions.iter().any(|a| matches!(a.eligibility, Eligibility::Lexical))).unwrap();
    let assertion = doc.metadata.assertions.iter().find(|a| matches!(a.eligibility, Eligibility::Lexical)).unwrap();
    let detail = key(doc, assertion);
    let release = doc.metadata.releases.iter().find(|r| r.assertions.contains(&assertion.id)).unwrap();
    let local = RelationTarget::Local(LocalRef::Assertion(assertion.id.clone()));
    let catalog = RelationTarget::Catalog(CatalogTarget { source: detail.source, release: release.id.clone(), native_entry_id: detail.native_entry_id, native_assertion_id: detail.native_assertion_id, occurrence: Some(assertion.occurrence.clone()) });
    let requests = [query(None, None), DictionaryRequest::Resolve { target: local.clone() }, DictionaryRequest::Resolve { target: catalog.clone() }];
    let equal = |a: serde_json::Value, b: serde_json::Value| assert_eq!(a, b, "WHOLE typed response parity");
    {
        let full = DictionarySearchEngine::open(store).unwrap();
        let ordinary = full.exact("farvi", Default::default()).unwrap();
        let (profiled, p) = full.exact_profiled("farvi", Default::default()).unwrap();
        equal(serde_json::to_value(ordinary).unwrap(), serde_json::to_value(profiled).unwrap());
        assert_eq!(p.total, p.lookup_order_paging + p.hydration + p.candidate_sql);
        for target in [&local, &catalog] {
            let ordinary = full.resolve(target).unwrap();
            let (profiled, p) = full.resolve_profiled(target).unwrap();
            equal(serde_json::to_value(ordinary).unwrap(), serde_json::to_value(profiled).unwrap());
            assert_eq!(p.total, p.lookup + p.projection);
            assert_eq!(p.candidate_sql, std::time::Duration::ZERO);
            assert!(p.projection > std::time::Duration::ZERO);
        }
    }
    println!("PASS full standalone Exact+Local+Catalog WHOLE ordinary/profile parity");
    {
        let compact = DictionaryCompactSearchEngine::open(store).unwrap();
        for request in &requests {
            let ordinary = compact.execute(request.clone()).unwrap();
            let (profiled, p) = compact.execute_profiled(request.clone()).unwrap();
            equal(serde_json::to_value(ordinary).unwrap(), serde_json::to_value(profiled).unwrap());
            assert_eq!(p.total, p.lookup + p.projection);
            assert!(p.candidate_sql <= p.lookup);
            assert!(p.projection > std::time::Duration::ZERO);
        }
    }
    println!("PASS compact standalone Exact+Local+Catalog WHOLE ordinary/profile parity");
    let selection = CollectionQuerySelection { packs: PACKS.to_vec(), disabled: vec![], profile: None };
    {
        let mut profiled = CollectionSearchEngine::open(root, selection.clone()).unwrap();
        let mut ordinary = CollectionSearchEngine::open(root, selection.clone()).unwrap();
        for (index, request) in requests.iter().enumerate() {
            let expected = ordinary.execute(request.clone()).unwrap();
            let (actual, p) = profiled.execute_profiled(request.clone()).unwrap();
            equal(serde_json::to_value(expected).unwrap(), serde_json::to_value(actual).unwrap());
            assert_eq!(p.payload_setup.is_some(), index == 0, "first Exact setupSome; warm Local/Catalog None");
            let setup = p.payload_setup.map(|s| s.total).unwrap_or_default();
            assert_eq!(p.query.total, p.total - setup);
            assert_eq!(p.query.total, p.query.lookup + p.query.projection);
            assert!(p.query.candidate_sql <= p.query.lookup);
            assert!(p.query.projection > std::time::Duration::ZERO);
        }
    }
    println!("PASS full managed Exact+Local+Catalog WHOLE parity; firstSome+warmNone");
    {
        let mut profiled = CollectionCompactSearchEngine::open(root, selection.clone()).unwrap();
        let mut ordinary = CollectionCompactSearchEngine::open(root, selection).unwrap();
        for (index, request) in requests.iter().enumerate() {
            let expected = ordinary.execute(request.clone()).unwrap();
            let (actual, p) = profiled.execute_profiled(request.clone()).unwrap();
            equal(serde_json::to_value(expected).unwrap(), serde_json::to_value(actual).unwrap());
            assert_eq!(p.payload_setup.is_some(), index == 0, "first Exact setupSome; warm Local/Catalog None");
            let setup = p.payload_setup.map(|s| s.total).unwrap_or_default();
            assert_eq!(p.query.total, p.total - setup);
            assert_eq!(p.query.total, p.query.lookup + p.query.projection);
            assert!(p.query.candidate_sql <= p.query.lookup);
            assert!(p.query.projection > std::time::Duration::ZERO);
        }
    }
    println!("PASS compact managed Exact+Local+Catalog WHOLE parity; firstSome+warmNone");
}
