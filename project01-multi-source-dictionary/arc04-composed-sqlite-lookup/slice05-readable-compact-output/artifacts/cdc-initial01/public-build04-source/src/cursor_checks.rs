// Prospective SOURCE ONLY; cursors are produced by actual public APIs at execution.
fn cursor_checks(store: &Path) {
    let full = DictionarySearchEngine::open(store).unwrap();
    let compact = DictionaryCompactSearchEngine::open(store).unwrap();
    let page = || DictionaryExactOptions { page_size: Some(1), ..Default::default() };
    let legacy = full.exact("farvi", page()).unwrap().next_cursor.unwrap();
    let projected = compact.exact("farvi", page()).unwrap().next_cursor.unwrap();
    assert!(matches!(compact.exact("farvi", DictionaryExactOptions {
        cursor: Some(legacy.clone()), ..page()
    }), Err(DictionaryQueryError::StaleCursor)));
    assert!(matches!(full.exact("farvi", DictionaryExactOptions {
        cursor: Some(projected.clone()), ..page()
    }), Err(DictionaryQueryError::StaleCursor)));
    for (good, malformed) in [
        (legacy.clone(), format!("{legacy}zz")),
        (projected.clone(), format!("{projected}zz")),
    ] {
        // Valid foreign grammar is stale; invalid foreign grammar is malformed.
        assert_ne!(good, malformed);
        assert!(matches!(compact.exact("farvi", DictionaryExactOptions {
            cursor: Some(malformed.clone()), ..page()
        }), Err(DictionaryQueryError::MalformedCursor { .. })));
        assert!(matches!(full.exact("farvi", DictionaryExactOptions {
            cursor: Some(malformed), ..page()
        }), Err(DictionaryQueryError::MalformedCursor { .. })));
    }
    for malformed in ["dictionary-compact-v1:aéx".to_owned(), "é".to_owned()] {
        assert!(matches!(compact.exact("farvi", DictionaryExactOptions {
            cursor: Some(malformed), ..page()
        }), Err(DictionaryQueryError::MalformedCursor { .. })));
    }
    // Same query in a newly opened compact engine accepts prior cursor/page-size change.
    let fresh = DictionaryCompactSearchEngine::open(store).unwrap();
    assert!(fresh.exact("farvi", DictionaryExactOptions {
        page_size: Some(2), cursor: Some(projected), ..Default::default()
    }).is_ok());
    println!("PASS valid foreign stale / malformed foreign malformed / Unicode no panic / fresh compact continuation");
}
