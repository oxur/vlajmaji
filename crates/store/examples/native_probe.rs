//! Reproducible full-corpus create, reopen, oracle and query-batch probe.

use serde::Serialize;
use std::{env, error::Error, fs, path::PathBuf, time::Instant};
use vlajmaji_lexicon::Lexicon;
use vlajmaji_store::{
    GraphStore, LexicalStore, PresenceState, SearchQuery, SearchResult, SqliteStore, scan_reference,
};

#[derive(Serialize)]
struct Probe<'a> {
    phase: &'a str,
    archive_sha256: String,
    records: usize,
    queries: usize,
    sqlite_setup_ms: u128,
    sqlite_query_batch_ms: u128,
    sqlite: Vec<serde_json::Value>,
    graph_setup_ms: u128,
    graph_query_batch_ms: u128,
    graph: Vec<serde_json::Value>,
    elapsed_ms: u128,
}

#[derive(Serialize)]
struct Trial {
    backend: &'static str,
    trial: usize,
    query_batch_ms: u128,
    checked_queries: usize,
}

#[derive(Serialize)]
struct Bench<'a> {
    phase: &'a str,
    archive_sha256: String,
    records: usize,
    queries: usize,
    sqlite_reopen_ms: u128,
    graph_reopen_ms: u128,
    sqlite_file_bytes_after_close: u64,
    graph_file_bytes_after_close: u64,
    warmups: Vec<Trial>,
    measured: Vec<Trial>,
}

fn queries(lexicon: &Lexicon) -> Vec<SearchQuery> {
    let mut out = Vec::new();
    for word in [
        "snime",
        "xrula",
        "tricu",
        "slabu",
        "citno",
        "cpana",
        "kalri",
        "banro",
        "farvi",
        "ricrprunu",
        "flaume",
        "rutrprunu",
        "si'e",
        "sie",
        "le jaica",
        "zzunknown",
    ] {
        out.push(SearchQuery::ExactSpelling(word.into()));
    }
    for term in [
        "snow",
        "plum",
        "flower",
        "develop",
        "sn",
        "ai",
        "y",
        "si'e",
        "a'i",
        "",
        "MATCH \" OR SELECT * ",
    ] {
        out.push(SearchQuery::DefinitionContains(term.into()));
    }
    for value in ["si'e", "snim", "cpan", "cpana"] {
        out.push(SearchQuery::Form {
            value: value.into(),
            role: None,
        });
        out.push(SearchQuery::Form {
            value: value.into(),
            role: Some(vlajmaji_lexicon::FormRole::AssignedRafsi),
        });
        out.push(SearchQuery::Form {
            value: value.into(),
            role: Some(vlajmaji_lexicon::FormRole::SourceReportedForm),
        });
    }
    for name in ["notes", "selmaho", "unknown"] {
        for state in [
            PresenceState::Missing,
            PresenceState::Null,
            PresenceState::Value,
        ] {
            out.push(SearchQuery::Field {
                name: name.into(),
                state,
                raw_value: None,
            });
        }
    }
    for name in ["definition", "uncaptured arbitrary field"] {
        out.push(SearchQuery::Field {
            name: name.into(),
            state: PresenceState::Missing,
            raw_value: None,
        });
    }
    for word in ["farvi", "snime", "xrula", "zzunknown"] {
        for depth in [1, 2] {
            out.push(SearchQuery::CrossReferences {
                spelling: word.into(),
                depth,
            });
        }
    }
    let first_origin = lexicon
        .cross_references()
        .first()
        .and_then(|edge| {
            lexicon
                .records()
                .iter()
                .find(|record| record.identity() == edge.origin())
        })
        .and_then(|record| record.word())
        .unwrap_or("zzunknown");
    for depth in [1, 2] {
        out.push(SearchQuery::CrossReferences {
            spelling: first_origin.into(),
            depth,
        });
    }
    out.push(SearchQuery::ProposalExact("zzunknown".into()));
    out
}

fn execute_batch(
    store: &dyn LexicalStore,
    queries: &[SearchQuery],
    expected: &[SearchResult],
) -> Result<(Vec<serde_json::Value>, u128), Box<dyn Error>> {
    let started = Instant::now();
    let mut actual_results = Vec::with_capacity(queries.len());
    for query in queries {
        actual_results.push(store.search(query)?);
    }
    let elapsed_ms = started.elapsed().as_millis();
    let mut output = Vec::with_capacity(queries.len());
    for (index, ((query, actual), expected)) in
        queries.iter().zip(actual_results).zip(expected).enumerate()
    {
        if actual.record_ids != expected.record_ids || actual.edges != expected.edges {
            return Err(format!(
                "query {index} mismatch: {query:?}; actual ids={} edges={}, scan ids={} edges={}",
                actual.record_ids.len(),
                actual.edges.len(),
                expected.record_ids.len(),
                expected.edges.len()
            )
            .into());
        }
        output.push(serde_json::json!({
            "index": index, "query": query, "records": actual.record_ids.len(),
            "edges": actual.edges.len(), "candidates": actual.candidates,
            "path": actual.path, "fallback": actual.fallback_reason
        }));
    }
    Ok((output, elapsed_ms))
}

fn directory_bytes(path: &std::path::Path) -> Result<u64, Box<dyn Error>> {
    let mut total = 0u64;
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let metadata = entry.metadata()?;
        if metadata.is_dir() {
            total = total.saturating_add(directory_bytes(&entry.path())?);
        } else if metadata.is_file() {
            total = total.saturating_add(metadata.len());
        }
    }
    Ok(total)
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 5 {
        return Err("usage: native_probe create|open|bench ARCHIVE SQLITE_DIR GRAPH_DIR".into());
    }
    let phase = &args[1];
    let archive_path = PathBuf::from(&args[2]);
    let sqlite_path = PathBuf::from(&args[3]);
    let graph_path = PathBuf::from(&args[4]);
    let bytes = fs::read(&archive_path)?;
    let lexicon = Lexicon::from_archive_bytes(&bytes)?;
    let all = queries(&lexicon);
    let expected: Vec<SearchResult> = all
        .iter()
        .map(|query| scan_reference(&lexicon, query))
        .collect();

    if phase == "bench" {
        let sqlite_started = Instant::now();
        let sqlite = SqliteStore::open(&sqlite_path)?;
        let sqlite_reopen_ms = sqlite_started.elapsed().as_millis();
        let graph_started = Instant::now();
        let graph = GraphStore::open(&graph_path)?;
        let graph_reopen_ms = graph_started.elapsed().as_millis();
        sqlite.flush()?;
        graph.flush()?;
        let mut warmups = Vec::new();
        for (name, store) in [
            ("sqlite", &sqlite as &dyn LexicalStore),
            ("oxigraph", &graph as &dyn LexicalStore),
        ] {
            let (_, elapsed) = execute_batch(store, &all, &expected)?;
            warmups.push(Trial {
                backend: name,
                trial: 0,
                query_batch_ms: elapsed,
                checked_queries: all.len(),
            });
        }
        let mut measured = Vec::new();
        // Five paired trials; pair order alternates to balance first/second position.
        let order = [
            "sqlite", "oxigraph", "oxigraph", "sqlite", "sqlite", "oxigraph", "oxigraph", "sqlite",
            "sqlite", "oxigraph",
        ];
        let mut per_backend = [0usize; 2];
        for name in order {
            let (store, slot) = if name == "sqlite" {
                (&sqlite as &dyn LexicalStore, 0)
            } else {
                (&graph as &dyn LexicalStore, 1)
            };
            per_backend[slot] += 1;
            let (_, elapsed) = execute_batch(store, &all, &expected)?;
            measured.push(Trial {
                backend: name,
                trial: per_backend[slot],
                query_batch_ms: elapsed,
                checked_queries: all.len(),
            });
        }
        sqlite.flush()?;
        graph.flush()?;
        drop(sqlite);
        drop(graph);
        let sqlite_file_bytes_after_close = directory_bytes(&sqlite_path)?;
        let graph_file_bytes_after_close = directory_bytes(&graph_path)?;
        let output = Bench {
            phase,
            archive_sha256: vlajmaji_store::archive_sha256(&bytes),
            records: lexicon.records().len(),
            queries: all.len(),
            sqlite_reopen_ms,
            graph_reopen_ms,
            sqlite_file_bytes_after_close,
            graph_file_bytes_after_close,
            warmups,
            measured,
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
        return Ok(());
    }

    let started = Instant::now();
    let sqlite_started = Instant::now();
    let sqlite = if phase == "create" {
        SqliteStore::create(&sqlite_path, &bytes)?
    } else {
        SqliteStore::open(&sqlite_path)?
    };
    let sqlite_setup_ms = sqlite_started.elapsed().as_millis();
    let (sqlite_rows, sqlite_query_batch_ms) = execute_batch(&sqlite, &all, &expected)?;
    sqlite.flush()?;
    let sqlite_archive_ok = sqlite.archive_bytes()? == bytes;
    drop(sqlite);
    if !sqlite_archive_ok {
        return Err("SQLite archive bytes differ".into());
    }

    let graph_started = Instant::now();
    let graph = if phase == "create" {
        GraphStore::create(&graph_path, &bytes)?
    } else {
        GraphStore::open(&graph_path)?
    };
    let graph_setup_ms = graph_started.elapsed().as_millis();
    let (graph_rows, graph_query_batch_ms) = execute_batch(&graph, &all, &expected)?;
    graph.flush()?;
    let graph_archive_ok = graph.archive_bytes()? == bytes;
    drop(graph);
    if !graph_archive_ok {
        return Err("graph archive bytes differ".into());
    }

    let output = Probe {
        phase,
        archive_sha256: vlajmaji_store::archive_sha256(&bytes),
        records: lexicon.records().len(),
        queries: all.len(),
        sqlite_setup_ms,
        sqlite_query_batch_ms,
        sqlite: sqlite_rows,
        graph_setup_ms,
        graph_query_batch_ms,
        graph: graph_rows,
        elapsed_ms: started.elapsed().as_millis(),
    };
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}
