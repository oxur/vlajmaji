//! JSON command-line interface for the research lexical-search prototype.

use std::io::{self, Read, Write};

use clap::{Parser, Subcommand};
use vlajmaji_search::{
    DictionaryErrorReport, DictionaryRequest, DictionarySearchEngine, Query, SearchEngine,
    SearchOptions,
};

#[derive(Debug, Parser)]
#[command(
    name = "vlajmaji",
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
    /// Build a fresh grouped-dictionary store from versioned dictionary JSON.
    DictionaryBuild {
        input_json: std::path::PathBuf,
        store: std::path::PathBuf,
    },
    /// Execute a grouped exact or stable-reference request from --request or stdin.
    DictionaryQuery {
        store: std::path::PathBuf,
        #[arg(long)]
        request: Option<String>,
    },
}

#[derive(Debug)]
enum CliError {
    Legacy(String),
    Structured(DictionaryErrorReport),
}

fn structured_error(code: &str, message: impl Into<String>) -> CliError {
    CliError::Structured(DictionaryErrorReport {
        code: code.to_owned(),
        message: message.into(),
        sources: Vec::new(),
    })
}

fn execute() -> Result<serde_json::Value, CliError> {
    let args = Args::parse();
    let output = match args.command {
        Command::Build { archive, store } => {
            let bytes = std::fs::read(&archive).map_err(|e| {
                CliError::Legacy(format!("read archive {}: {e}", archive.display()))
            })?;
            let engine =
                SearchEngine::build(&store, &bytes).map_err(|e| CliError::Legacy(e.to_string()))?;
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
                        .map_err(|e| CliError::Legacy(format!("read query JSON: {e}")))?;
                    text
                }
            };
            let request: Query = serde_json::from_str(&query_text)
                .map_err(|e| CliError::Legacy(format!("decode query JSON: {e}")))?;
            let engine = SearchEngine::open(&store)
                .map_err(|e| CliError::Legacy(format!("open store {}: {e}", store.display())))?;
            let response = engine
                .search(
                    request,
                    SearchOptions {
                        limit,
                        include_proposals,
                    },
                )
                .map_err(|e| CliError::Legacy(e.to_string()))?;
            serde_json::to_value(response)
                .map_err(|e| CliError::Legacy(format!("encode response JSON: {e}")))?
        }
        Command::Info { store } => {
            let engine = SearchEngine::open(&store)
                .map_err(|e| CliError::Legacy(format!("open store {}: {e}", store.display())))?;
            let (records, snapshots) = engine.corpus_counts();
            serde_json::json!({"records":records,"snapshots":snapshots,"morphology_enabled":vlajmaji_search::morphology_enabled()})
        }
        Command::DictionaryBuild { input_json, store } => {
            let bytes = std::fs::read(&input_json).map_err(|error| {
                structured_error(
                    "io",
                    format!("read dictionary JSON {}: {error}", input_json.display()),
                )
            })?;
            let engine = DictionarySearchEngine::build(&store, &bytes)
                .map_err(|error| CliError::Structured(error.report()))?;
            let (captures, occurrences, entries, assertions) = engine.dictionary_counts();
            serde_json::json!({
                "operation":"dictionary-build",
                "version":1,
                "state_sha256":engine.state_sha256(),
                "store":store,
                "captures":captures,
                "occurrences":occurrences,
                "entries":entries,
                "assertions":assertions
            })
        }
        Command::DictionaryQuery { store, request } => {
            let request_text = match request {
                Some(text) => text,
                None => {
                    let mut text = String::new();
                    io::stdin().read_to_string(&mut text).map_err(|error| {
                        structured_error("io", format!("read dictionary request: {error}"))
                    })?;
                    text
                }
            };
            let request: DictionaryRequest =
                serde_json::from_str(&request_text).map_err(|error| {
                    structured_error(
                        "request_json",
                        format!("decode dictionary request: {error}"),
                    )
                })?;
            let engine = DictionarySearchEngine::open(&store)
                .map_err(|error| CliError::Structured(error.report()))?;
            match request {
                DictionaryRequest::Exact { spelling, options } => serde_json::to_value(
                    engine
                        .exact(&spelling, options)
                        .map_err(|error| CliError::Structured(error.report()))?,
                )
                .map_err(|error| structured_error("response_json", error.to_string()))?,
                DictionaryRequest::Resolve { target } => serde_json::to_value(
                    engine
                        .resolve(&target)
                        .map_err(|error| CliError::Structured(error.report()))?,
                )
                .map_err(|error| structured_error("response_json", error.to_string()))?,
            }
        }
    };
    Ok(output)
}

fn main() -> std::process::ExitCode {
    let output = match execute() {
        Ok(value) => value,
        Err(CliError::Legacy(message)) => {
            let _ = writeln!(io::stderr().lock(), "{message}");
            return std::process::ExitCode::FAILURE;
        }
        Err(CliError::Structured(report)) => {
            let mut stderr = io::stderr().lock();
            if serde_json::to_writer(&mut stderr, &serde_json::json!({"error": report})).is_ok() {
                let _ = stderr.write_all(b"\n");
            }
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
