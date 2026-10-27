// Prospective SOURCE ONLY. This is deliberately unverified, inconsistent Tiny
// canonical data behind an unchanged minimal header, not a legitimate state or
// snapshot-authentication test. It discriminates the inherited full eager path.
fn eager_checks(base: &Path) {
    let model = fixture::complete_fixture().unwrap();
    let mut document = draft(model.clone());
    let entry = document.metadata.entries.iter().find(|e| e.source.as_str() == "source-b" && matches!(&e.spelling, Presence::Value(s) if s == "farvi")).unwrap();
    let target = document.metadata.assertions.iter().find(|a| a.entry == entry.id && matches!(a.eligibility, Eligibility::Lexical)).unwrap().clone();
    let store = base.join("eager-unverified-store");
    drop(vlajmaji_store::DictionaryStore::create(&store, &model).unwrap());
    drop(model);
    let (first_id, total, compact_cursor) = {
        let full = DictionarySearchEngine::open(&store).unwrap();
        let compact = DictionaryCompactSearchEngine::open(&store).unwrap();
        let options = DictionaryExactOptions { page_size: Some(1), ..Default::default() };
        let full_page = full.exact("farvi", options.clone()).unwrap();
        let compact_page = compact.exact("farvi", options).unwrap();
        let first = full_page.groups.iter().flat_map(|g| &g.hits).next().unwrap().assertion.id.clone();
        assert_ne!(first, target.id, "pre-input chosen missing row is outside the first valid page");
        assert_eq!(compact_page.groups.iter().flat_map(|g| &g.hits).next().unwrap().assertion.id, first);
        (first, full_page.totals, compact_page.next_cursor.unwrap())
    };
    let capture = document.captures.iter_mut().find(|c| c.identity == target.occurrence.capture).unwrap();
    let before_rows = capture.occurrences.len();
    capture.occurrences.retain(|r| r.locator != target.occurrence.locator);
    assert_eq!(capture.occurrences.len() + 1, before_rows);
    let database = store.join("dictionary.sqlite3");
    let original = std::fs::read(&database).unwrap();
    {
        let connection = rusqlite::Connection::open(&database).unwrap();
        assert_eq!(connection.execute("UPDATE canonical_document SET json=?1 WHERE id=1", rusqlite::params![serde_json::to_vec(&document).unwrap()]).unwrap(), 1);
    }
    {
        let minimum = vlajmaji_store::DictionaryStore::open(&store).unwrap();
        let observed = minimum.unverified_document().unwrap();
        assert!(!observed.captures.iter().find(|c| c.identity == target.occurrence.capture).unwrap().occurrences.iter().any(|r| r.locator == target.occurrence.locator));
    }
    let (full_zero, full_foreign, compact_zero, compact_first, compact_all) = {
        let full = DictionarySearchEngine::open(&store).unwrap();
        let compact = DictionaryCompactSearchEngine::open(&store).unwrap();
        let full_zero = full.exact("farvi", DictionaryExactOptions { page_size: Some(0), ..Default::default() });
        let full_foreign = full.exact("farvi", DictionaryExactOptions { page_size: Some(1), cursor: Some(compact_cursor.clone()), ..Default::default() });
        let compact_zero = compact.exact("farvi", DictionaryExactOptions { page_size: Some(0), ..Default::default() });
        let compact_first = compact.exact("farvi", DictionaryExactOptions { page_size: Some(1), ..Default::default() });
        let compact_all = compact.exact("farvi", DictionaryExactOptions::default());
        (full_zero, full_foreign, compact_zero, compact_first, compact_all)
    };
    std::fs::write(&database, &original).unwrap();
    assert_eq!(std::fs::read(&database).unwrap(), original);
    assert!(matches!(full_zero, Err(DictionaryQueryError::Integrity { .. })));
    assert!(matches!(full_foreign, Err(DictionaryQueryError::Integrity { .. })), "full eager raw error must precede valid foreign-cursor stale classification");
    let zero = compact_zero.unwrap();
    assert_eq!(zero.totals, total);
    assert!(zero.groups.is_empty() && zero.next_cursor.is_none());
    let first = compact_first.unwrap();
    assert_eq!(first.totals, total);
    assert_eq!(first.groups.iter().flat_map(|g| &g.hits).next().unwrap().assertion.id, first_id);
    assert!(matches!(compact_all, Err(DictionaryQueryError::Integrity { .. })));
    {
        let full = DictionarySearchEngine::open(&store).unwrap();
        assert_eq!(full.exact("farvi", DictionaryExactOptions::default()).unwrap().totals, total);
        assert!(matches!(full.exact("farvi", DictionaryExactOptions { page_size: Some(1), cursor: Some(compact_cursor), ..Default::default() }), Err(DictionaryQueryError::StaleCursor)));
    }
    println!("PASS full eager out-of-page/foreign-cursor precedence; compact page0/first-page avoids missing unrelated raw; complete valid bytes restored");
}
