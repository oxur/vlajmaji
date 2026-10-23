//! Cross-backend fixture and public failure-control integration checks.

#![cfg(feature = "graph")]

use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
use vlajmaji_lexicon::{Lexicon, Proposal, SourceFormat, SourceMetadata, TimePrecision};
use vlajmaji_store::{
    GraphStore, LexicalStore, PresenceState, SearchQuery, SqliteStore, StoreError, scan_reference,
};

fn root() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "relojban-store-fixture-{}-{nonce}",
        std::process::id()
    ))
}

fn fixture() -> (Vec<u8>, Lexicon) {
    let raw = r#"[
      {"word":"si'e","definition":"Snow plum flower ai y a'i \"quoted phrase\" OR SELECT * \u0000tail","rafsi":["si'e"],"selmaho":"MOI","nativeclass":null},
      {"word":"sie","definition":"develop phrase"},
      {"word":"si’e","definition":"curly apostrophe control"},
      {"word":"phrase","definition":"one complete phrase"},
      {"word":"clash","definition":"first competing source row"},
      {"word":"clash","definition":"second competing source row"},
      {"word":"empty","definition":"empty-value control","selmaho":"MOI","nativeclass":""},
      {"word":"object","definition":"object-value control","selmaho":"MOI","nativeclass":{}},
      {"word":"array","definition":"array-value control","selmaho":"MOI","nativeclass":[]}
    ]"#;
    let meta = SourceMetadata::new(
        "fixture:synthetic",
        "fixture.json",
        SourceFormat::LaLojbanJson,
        "https://example.invalid/fixture.json",
        "fixture-v1",
        None,
        TimePrecision::Unknown,
    );
    let mut lexicon = Lexicon::new();
    lexicon.import_source(meta, raw.as_bytes(), None).unwrap();
    lexicon.add_proposal(
        Proposal::new("fixture:proposal", "hypothetical", "explicitly synthetic").unwrap(),
    );
    let bytes = lexicon.to_archive_bytes().unwrap();
    (bytes, lexicon)
}

fn traversal_fixture() -> (Vec<u8>, Lexicon) {
    let raw = br#"[
      {"word":"left","definition":"left row","notes":"See also {right}."},
      {"word":"right","definition":"right row","notes":"See also {left}. See also {absent}."},
      {"word":"lonely","definition":"no captured links"}
    ]"#;
    let metadata = SourceMetadata::new(
        "fixture:traversal",
        "traversal.json",
        SourceFormat::LaLojbanJson,
        "https://example.invalid/traversal.json",
        "traversal-v1",
        None,
        TimePrecision::Unknown,
    );
    let mut lexicon = Lexicon::new();
    lexicon.import_source(metadata, raw, None).unwrap();
    (lexicon.to_archive_bytes().unwrap(), lexicon)
}

fn assert_equivalent(store: &impl LexicalStore, lexicon: &Lexicon, queries: &[SearchQuery]) {
    for query in queries {
        let actual = store.search(query).unwrap();
        let expected = scan_reference(lexicon, query);
        assert_eq!(
            actual.record_ids, expected.record_ids,
            "record IDs for {query:?}"
        );
        assert_eq!(
            actual.edges, expected.edges,
            "edge explanations for {query:?}"
        );
    }
}

#[test]
fn fixture_preserves_exact_literals_presence_roles_and_proposal() {
    fn require_debug<T: std::fmt::Debug>() {}
    require_debug::<SqliteStore>();
    require_debug::<GraphStore>();

    let base = root();
    let (archive, lexicon) = fixture();
    let queries = [
        SearchQuery::ExactSpelling("si'e".into()),
        SearchQuery::ExactSpelling("sie".into()),
        SearchQuery::ExactSpelling("si’e".into()),
        SearchQuery::ExactSpelling("clash".into()),
        SearchQuery::ExactSpelling("si".into()),
        SearchQuery::DefinitionContains("ai".into()),
        SearchQuery::DefinitionContains("y".into()),
        SearchQuery::DefinitionContains("a'i".into()),
        SearchQuery::DefinitionContains("\0".into()),
        SearchQuery::DefinitionContains("\"quoted phrase\" OR SELECT *".into()),
        SearchQuery::DefinitionContains("".into()),
        SearchQuery::Form {
            value: "si'e".into(),
            role: None,
        },
        SearchQuery::Form {
            value: "si'e".into(),
            role: Some(vlajmaji_lexicon::FormRole::SourceReportedForm),
        },
        SearchQuery::Field {
            name: "nativeclass".into(),
            state: PresenceState::Missing,
            raw_value: None,
        },
        SearchQuery::Field {
            name: "nativeclass".into(),
            state: PresenceState::Null,
            raw_value: None,
        },
        SearchQuery::Field {
            name: "nativeclass".into(),
            state: PresenceState::Value,
            raw_value: Some("\"\"".into()),
        },
        SearchQuery::Field {
            name: "nativeclass".into(),
            state: PresenceState::Value,
            raw_value: Some("[]".into()),
        },
        SearchQuery::Field {
            name: "nativeclass".into(),
            state: PresenceState::Value,
            raw_value: Some("{}".into()),
        },
        SearchQuery::Field {
            name: "selmaho".into(),
            state: PresenceState::Value,
            raw_value: Some("\"MOI\"".into()),
        },
        SearchQuery::Field {
            name: "uncaptured arbitrary field".into(),
            state: PresenceState::Missing,
            raw_value: None,
        },
        SearchQuery::ProposalExact("hypothetical".into()),
        SearchQuery::ProposalExact("si'e' OR *".into()),
    ];

    let sqlite_path = base.join("sqlite");
    let sqlite = SqliteStore::create(&sqlite_path, &archive).unwrap();
    let sqlite_debug = format!("{sqlite:?}");
    assert!(sqlite_debug.contains("SQLite/FTS5"));
    assert!(sqlite_debug.len() < 512);
    assert_equivalent(&sqlite, &lexicon, &queries);
    let raw_cases = ["nativeclass", "uncaptured field"]
        .into_iter()
        .flat_map(|name| {
            [
                PresenceState::Missing,
                PresenceState::Null,
                PresenceState::Value,
            ]
            .into_iter()
            .flat_map(move |state| {
                [None, Some("null"), Some("\"\""), Some("not-the-value")]
                    .into_iter()
                    .map(move |raw| SearchQuery::Field {
                        name: name.to_owned(),
                        state,
                        raw_value: raw.map(str::to_owned),
                    })
            })
        })
        .collect::<Vec<_>>();
    assert_equivalent(&sqlite, &lexicon, &raw_cases);
    let missing_raw = scan_reference(
        &lexicon,
        &SearchQuery::Field {
            name: "nativeclass".into(),
            state: PresenceState::Missing,
            raw_value: Some("null".into()),
        },
    );
    assert!(missing_raw.record_ids.is_empty());
    let null_raw = scan_reference(
        &lexicon,
        &SearchQuery::Field {
            name: "nativeclass".into(),
            state: PresenceState::Null,
            raw_value: Some("null".into()),
        },
    );
    assert_eq!(null_raw.record_ids.len(), 1);
    let mismatched_null = scan_reference(
        &lexicon,
        &SearchQuery::Field {
            name: "nativeclass".into(),
            state: PresenceState::Null,
            raw_value: Some("not-null".into()),
        },
    );
    assert!(mismatched_null.record_ids.is_empty());
    sqlite.flush().unwrap();
    assert_eq!(sqlite.archive_bytes().unwrap(), archive);
    drop(sqlite);
    let sqlite = SqliteStore::open(&sqlite_path).unwrap();
    assert_equivalent(&sqlite, &lexicon, &queries);
    assert!(matches!(
        SqliteStore::create(&sqlite_path, &archive),
        Err(StoreError::ExistingPath)
    ));
    drop(sqlite);

    let graph_path = base.join("graph");
    let graph = GraphStore::create(&graph_path, &archive).unwrap();
    let graph_debug = format!("{graph:?}");
    assert!(graph_debug.contains("Oxigraph/RocksDB"));
    assert!(graph_debug.len() < 512);
    assert_equivalent(&graph, &lexicon, &queries);
    assert_equivalent(&graph, &lexicon, &raw_cases);
    graph.flush().unwrap();
    assert_eq!(graph.archive_bytes().unwrap(), archive);
    drop(graph);
    let graph = GraphStore::open(&graph_path).unwrap();
    assert_equivalent(&graph, &lexicon, &queries);
    assert!(matches!(
        GraphStore::create(&graph_path, &archive),
        Err(StoreError::ExistingPath)
    ));
    drop(graph);

    assert!(SqliteStore::create(base.join("invalid-sqlite"), b"not an archive").is_err());
    let empty_sqlite = base.join("empty-sqlite");
    fs::create_dir_all(&empty_sqlite).unwrap();
    assert!(SqliteStore::open(&empty_sqlite).is_err());
    let empty_graph = base.join("empty-graph");
    fs::create_dir_all(&empty_graph).unwrap();
    assert!(GraphStore::open(&empty_graph).is_err());
    fs::remove_dir_all(base).unwrap();
}

#[test]
fn traversal_retains_cycle_closing_and_unresolved_edges() {
    let base = root();
    let (archive, lexicon) = traversal_fixture();
    let depth1 = SearchQuery::CrossReferences {
        spelling: "left".into(),
        depth: 1,
    };
    let depth2 = SearchQuery::CrossReferences {
        spelling: "left".into(),
        depth: 2,
    };
    let no_edges = SearchQuery::CrossReferences {
        spelling: "lonely".into(),
        depth: 2,
    };
    let expected1 = scan_reference(&lexicon, &depth1);
    let expected2 = scan_reference(&lexicon, &depth2);
    assert_eq!(expected1.record_ids.len(), 1);
    assert_eq!(expected1.edges.len(), 1);
    assert_eq!(expected2.record_ids.len(), 2);
    assert_eq!(expected2.edges.len(), 3);
    assert!(expected2.edges.iter().any(|edge| edge.target_id.is_none()));
    let start_id = scan_reference(&lexicon, &SearchQuery::ExactSpelling("left".into())).record_ids;
    assert!(start_id.is_subset(&expected2.record_ids));
    let expected_no_edges = scan_reference(&lexicon, &no_edges);
    assert!(expected_no_edges.record_ids.is_empty());
    assert!(expected_no_edges.edges.is_empty());

    let sqlite = SqliteStore::create(base.join("sqlite"), &archive).unwrap();
    assert_equivalent(
        &sqlite,
        &lexicon,
        &[depth1.clone(), depth2.clone(), no_edges.clone()],
    );
    let graph = GraphStore::create(base.join("graph"), &archive).unwrap();
    assert_equivalent(&graph, &lexicon, &[depth1, depth2, no_edges]);
    drop(sqlite);
    drop(graph);
    fs::remove_dir_all(base).unwrap();
}
