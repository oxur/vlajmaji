// Prospective SOURCE ONLY. Run only against NEW owned Tiny roots after freeze.
// Restore the owned derived bytes before evaluating captured errors, retaining
// the original fault results and the valid public counterpart in the same run.
fn derived_fault_checks(base: &Path, docs: &[UnverifiedDictionaryDocument], sealed: &[(PackId, Vec<u8>, Vec<u8>)]) {
    for mode in ["missing", "corrupt", "version"] {
        let root = base.join(format!("derived-{mode}-catalogue"));
        let (pack_id, release, seal) = &sealed[1];
        assert_eq!(*pack_id, PackId::Llg);
        let request = CollectionRequest::new(vec![CollectionInput::SealedBytes {
            pack_id: *pack_id, release: Cow::Borrowed(release), seal: Cow::Borrowed(seal),
        }], CollectionSelection::Packs(vec![*pack_id]), true).unwrap();
        let installed = CollectionCatalogue::install(&root, request).unwrap();
        let store = installed.generation.unwrap().store_path;
        assert_owned_store_path(&root, &store);
        let assertion = docs[1].metadata.assertions.iter().find(|a| matches!(a.eligibility, Eligibility::Lexical)).unwrap();
        let reference = key(&docs[1], assertion);
        let open = || CollectionCompactSearchEngine::open(&root, CollectionQuerySelection {
            packs: vec![PackId::Llg], disabled: vec![], profile: None,
        });
        let before_snapshot = {
            let mut engine = open().unwrap();
            let result = engine.detail(&reference, None).unwrap();
            assert_eq!(result.response.status, ReferenceStatus::Resolved);
            result.response.snapshot_sha256
        };
        let database = store.join("dictionary.sqlite3");
        let held = store.join("dictionary.sqlite3.original-held");
        let original = std::fs::read(&database).unwrap();
        let catalogue_before = sha(&std::fs::read(root.join("catalogue.sqlite3")).unwrap());
        std::fs::rename(&database, &held).unwrap();
        match mode {
            "missing" => {},
            "corrupt" => std::fs::write(&database, b"not a SQLite dictionary\n").unwrap(),
            "version" => {
                std::fs::write(&database, &original).unwrap();
                let connection = rusqlite::Connection::open(&database).unwrap();
                assert_eq!(connection.execute("UPDATE metadata SET value='unsupported-s05-test-version' WHERE key='schema_version'", []).unwrap(), 1);
            },
            _ => unreachable!(),
        }
        let direct = vlajmaji_store::DictionaryStore::open(&store).err().expect("equivalent direct minimum must fail");
        let (stale, detail, exact_error, reference_error) = {
            let mut engine = open().unwrap();
            let stale = engine.detail(&reference, Some(&before_snapshot));
            let detail = engine.detail(&reference, None);
            let exact_error = engine.execute(query(None, None));
            let reference_error = engine.execute(DictionaryRequest::Resolve {
                target: RelationTarget::Local(LocalRef::Assertion(assertion.id.clone())),
            });
            (stale, detail, exact_error, reference_error)
        };
        // No candidate handles remain pinned when restoring this NEW fixture.
        if mode != "missing" { std::fs::remove_file(&database).unwrap(); }
        std::fs::rename(&held, &database).unwrap();
        assert_eq!(std::fs::read(&database).unwrap(), original);
        assert_eq!(sha(&std::fs::read(root.join("catalogue.sqlite3")).unwrap()), catalogue_before);
        let stale = stale.err().expect("changed observed availability must stale before payload");
        let detail = detail.err().expect("selected detail must retain derived failure");
        let exact_error = exact_error.err().expect("selected exact must retain derived failure");
        let reference_error = reference_error.err().expect("selected reference must retain derived failure");
        assert_eq!(stale.report_for("lookup_detail").category, "stale_snapshot");
        for error in [&detail, &exact_error, &reference_error] {
            assert!(same_store_boundary(concrete_store_cause(error).expect("typed public Store cause survives"), &direct), "{mode}: {error:?} vs {direct:?}");
        }
        let mut restored = open().unwrap();
        assert_eq!(restored.detail(&reference, None).unwrap().response.status, ReferenceStatus::Resolved);
        assert_eq!(exact(restored.execute(query(None, None)).unwrap()).totals.occurrences, 1);
        println!("PASS {mode} derived: exact/reference/detail preserve equivalent minimum cause; old snapshot stale first; exact bytes restored and valid counterpart");
    }
}
