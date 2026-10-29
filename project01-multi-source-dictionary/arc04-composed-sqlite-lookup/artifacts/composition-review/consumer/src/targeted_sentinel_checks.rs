// Prospective SOURCE ONLY. Deliberately mutate ONLY an owned Tiny descriptor;
// this is projection-scope/error qualification, not authenticated model proof.
fn targeted_sentinel_checks(base: &Path, docs: &[UnverifiedDictionaryDocument], sealed: &[(PackId, Vec<u8>, Vec<u8>)]) {
    let root = base.join("targeted-sentinel-catalogue");
    let (pack, release, seal) = &sealed[1];
    assert_eq!(*pack, PackId::Llg);
    let request = CollectionRequest::new(vec![CollectionInput::SealedBytes {
        pack_id: *pack, release: Cow::Borrowed(release), seal: Cow::Borrowed(seal),
    }], CollectionSelection::Packs(vec![*pack]), true).unwrap();
    CollectionCatalogue::register(&root, request).unwrap();
    let digest = sha(release);
    let assertion = &docs[1].metadata.assertions[0];
    let target = RelationTarget::Local(LocalRef::Assertion(assertion.id.clone()));
    let baseline = CollectionCatalogue::open(&root).unwrap().reference_projection(*pack, &digest, &target).unwrap();
    let database = root.join("catalogue.sqlite3");
    let original: Vec<u8> = {
        let connection = rusqlite::Connection::open(&database).unwrap();
        connection.query_row("SELECT descriptor_json FROM registrations_v2 WHERE pack_id=?1 AND release_sha256=?2", rusqlite::params![pack.as_str(),digest], |r| r.get(0)).unwrap()
    };
    let mut descriptor: serde_json::Value = serde_json::from_slice(&original).unwrap();
    let releases = descriptor["source_metadata"]["releases"].as_array_mut().unwrap();
    let mut unrelated = releases[0].clone();
    unrelated["id"] = serde_json::json!("unrelated-sentinel-release");
    unrelated["selected"] = serde_json::json!(false);
    unrelated["assertions"] = serde_json::json!((0..50_000).map(|i| format!("sentinel-assertion-{i:06}")).collect::<Vec<_>>());
    releases.push(unrelated);
    let sentinel = serde_json::to_vec(&descriptor).unwrap();
    assert!(sentinel.len() > 1_000_000);
    let write = |bytes: &[u8]| {
        let connection = rusqlite::Connection::open(&database).unwrap();
        assert_eq!(connection.execute("UPDATE registrations_v2 SET descriptor_json=?1 WHERE pack_id=?2 AND release_sha256=?3", rusqlite::params![bytes,pack.as_str(),digest]).unwrap(), 1);
    };
    write(&sentinel);
    let before = sha(&std::fs::read(&database).unwrap());
    {
        let catalogue = CollectionCatalogue::open(&root).unwrap();
        let full = catalogue.reference_metadata(*pack, &digest).unwrap();
        let witness = full.source_metadata.releases.iter().find(|r| r.id.as_str() == "unrelated-sentinel-release").unwrap();
        assert_eq!(witness.assertions.len(), 50_000); // Establish setup at the full typed boundary.
        drop(full);
        let projected = catalogue.reference_projection(*pack, &digest, &target).unwrap();
        assert_eq!(projected, baseline); // No unrelated release/assertion inventory appears.
    }
    assert_eq!(sha(&std::fs::read(&database).unwrap()), before);
    let mut trailing = sentinel.clone();
    trailing.extend_from_slice(b" false");
    write(&trailing);
    let failure = CollectionCatalogue::open(&root).unwrap().reference_projection(*pack, &digest, &target).err().expect("trailing descriptor must fail");
    write(&original); // Restore before asserting the error, retaining all captured results.
    assert!(matches!(failure, vlajmaji_store::CollectionError::ReferenceProjection { .. }));
    assert!(std::error::Error::source(&failure).is_some());
    assert_eq!(CollectionCatalogue::open(&root).unwrap().reference_projection(*pack, &digest, &target).unwrap(), baseline);
    println!("PASS >1MB/50000 unrelated assertion sentinel has no projected effect; trailing descriptor retains typed cause; original valid projection restored (allocation/lightness also requires source inspection)");
}
