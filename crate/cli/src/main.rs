//! JSON command-line interface for the research lexical-search prototype.

use std::io::{self, Read, Write};

use clap::{Parser, Subcommand};
use vlajmaji_search::{Query, SearchEngine, SearchOptions};

#[derive(Debug, Parser)]
#[command(
    name = "relojban-search",
    version,
    about = "Research CLI for the source-preserving lexical search prototype"
)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Build a new SQLite index from an accepted archive; the destination must not exist.
    Build {
        archive: std::path::PathBuf,
        store: std::path::PathBuf,
    },
    /// Read a JSON Query from stdin or --query and emit one JSON response.
    Search {
        store: std::path::PathBuf,
        #[arg(long)]
        query: Option<String>,
        #[arg(long, default_value_t = 20)]
        limit: usize,
        #[arg(long)]
        include_proposals: bool,
    },
    /// Emit the current corpus record and snapshot counts.
    Info { store: std::path::PathBuf },
}

fn execute() -> Result<serde_json::Value, String> {
    let args = Args::parse();
    let output = match args.command {
        Command::Build { archive, store } => {
            let bytes = std::fs::read(&archive)
                .map_err(|e| format!("read archive {}: {e}", archive.display()))?;
            let engine = SearchEngine::build(&store, &bytes).map_err(|e| e.to_string())?;
            let (records, snapshots) = engine.corpus_counts();
            serde_json::json!({"store":store,"records":records,"snapshots":snapshots,"source_archive_bytes":bytes.len()})
        }
        Command::Search {
            store,
            query,
            limit,
            include_proposals,
        } => {
            let query_text = match query {
                Some(text) => text,
                None => {
                    let mut text = String::new();
                    io::stdin()
                        .read_to_string(&mut text)
                        .map_err(|e| format!("read query JSON: {e}"))?;
                    text
                }
            };
            let request: Query =
                serde_json::from_str(&query_text).map_err(|e| format!("decode query JSON: {e}"))?;
            let engine = SearchEngine::open(&store)
                .map_err(|e| format!("open store {}: {e}", store.display()))?;
            let response = engine
                .search(
                    request,
                    SearchOptions {
                        limit,
                        include_proposals,
                    },
                )
                .map_err(|e| e.to_string())?;
            serde_json::to_value(response).map_err(|e| format!("encode response JSON: {e}"))?
        }
        Command::Info { store } => {
            let engine = SearchEngine::open(&store)
                .map_err(|e| format!("open store {}: {e}", store.display()))?;
            let (records, snapshots) = engine.corpus_counts();
            serde_json::json!({"records":records,"snapshots":snapshots,"morphology_enabled":vlajmaji_search::morphology_enabled()})
        }
    };
    Ok(output)
}

fn main() -> std::process::ExitCode {
    let output = match execute() {
        Ok(value) => value,
        Err(message) => {
            let _ = writeln!(io::stderr().lock(), "{message}");
            return std::process::ExitCode::FAILURE;
        }
    };
    let mut stdout = io::stdout().lock();
    if let Err(error) = serde_json::to_writer(&mut stdout, &output) {
        if error.io_error_kind() == Some(io::ErrorKind::BrokenPipe) {
            return std::process::ExitCode::SUCCESS;
        }
        let _ = writeln!(io::stderr().lock(), "write JSON: {error}");
        return std::process::ExitCode::FAILURE;
    }
    if let Err(error) = stdout.write_all(b"\n").and_then(|()| stdout.flush()) {
        if error.kind() == io::ErrorKind::BrokenPipe {
            return std::process::ExitCode::SUCCESS;
        }
        let _ = writeln!(io::stderr().lock(), "write JSON: {error}");
        return std::process::ExitCode::FAILURE;
    }
    std::process::ExitCode::SUCCESS
}
