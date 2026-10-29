// Prospective SOURCE ONLY: exact target/link/digest expectations precede projection reads.
fn store_projection_checks(root: &Path, docs: &[UnverifiedDictionaryDocument], sealed: &[(PackId, Vec<u8>, Vec<u8>)]) {
    let catalogue = CollectionCatalogue::open(root).unwrap();
    let mut total = 0;
    for ((pack, release_bytes, _), input) in sealed.iter().zip(docs) {
        let release_sha = sha(release_bytes);
        for assertion in &input.metadata.assertions {
            let entry = input.metadata.entries.iter().find(|e| e.id == assertion.entry).unwrap();
            let expected_body = BTreeMap::from([
                (format!("entry:{}", entry.id), sha(&serde_json::to_vec(entry).unwrap())),
                (format!("assertion:{}", assertion.id), sha(&serde_json::to_vec(assertion).unwrap())),
            ]);
            let expected_entities = vec![
                vlajmaji_store::CollectionEntityReference {
                    role: "entry".into(), id: entry.id.to_string(), entry_ids: vec![],
                    assertion_ids: vec![], source_ids: vec![entry.source.to_string()],
                    occurrence: None, evidence: None, spelling: Some(entry.spelling.clone()),
                },
                vlajmaji_store::CollectionEntityReference {
                    role: "assertion".into(), id: assertion.id.to_string(),
                    entry_ids: vec![entry.id.to_string()], assertion_ids: vec![],
                    source_ids: vec![entry.source.to_string()], occurrence: Some(assertion.occurrence.clone()),
                    evidence: None, spelling: None,
                },
            ];
            let target = RelationTarget::Local(LocalRef::Assertion(assertion.id.clone()));
            let projection = catalogue.reference_projection(*pack, &release_sha, &target).unwrap();
            assert_eq!(projection.pack_id, pack.as_str());
            assert_eq!(projection.release_sha256, release_sha);
            assert_eq!(projection.payload_kind, "synthetic");
            assert_eq!(projection.entities, expected_entities);
            assert_eq!(projection.body_sha256, expected_body);
            assert_eq!(projection.captures.len(), 1);
            let capture = input.captures.iter().find(|c| c.identity == assertion.occurrence.capture).unwrap();
            assert_eq!(projection.captures[0].capture, capture.identity);
            assert_eq!(projection.captures[0].metadata, capture.metadata);
            assert_eq!(projection.captures[0].byte_length, capture.bytes.len());
            assert_eq!(projection.captures[0].locator_presence,
                       BTreeMap::from([(assertion.occurrence.locator.clone(), true)]));
            for release in &projection.releases {
                let original = input.metadata.releases.iter().find(|r| r.id.to_string() == release.summary.id).unwrap();
                let matching: Vec<_> = original.assertions.iter().filter(|id| **id == assertion.id).cloned().collect();
                assert_eq!(release.matching_assertion_ids, matching);
            }
            assert!(projection.releases.iter().any(|r| r.matching_assertion_ids.contains(&assertion.id)));
            total += 1;
        }
    }
    assert_eq!(total, 6);
    println!("PASS 6 target-only store projections: exact linked Entry/assertion/digests/capture locator and release intersection");
}
