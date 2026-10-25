//! JSON command-line interface for the research lexical-search prototype.

use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};
use vlajmaji_lexicon::{
    Dictionary, JBOTCI_PINNED_METADATA_SHA256, JBOTCI_PINNED_RAW_SHA256, JBOTCI_PINNED_REVISION,
    JbotciImportError, JbotciImportOptions, import_jbotci_export,
};
use vlajmaji_search::{
    DictionaryErrorReport, DictionaryRequest, DictionarySearchEngine, Query, SearchEngine,
    SearchOptions,
};

#[derive(Debug, Parser)]
#[command(
    name = "vlajmaji",
    version,
    about = "Research CLI for the source-preserving lexical search prototype",
    after_help = "Grouped dictionary commands use an explicit STORE when supplied; otherwise they use XDG_DATA_HOME/vlajmaji/stores/default, falling back to HOME/.local/share/vlajmaji/stores/default. Query and info never create or download a store. This default location is intended for a future dictionary-of-dictionaries; the initial Lensisku-only store is provisional."
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
    /// Emit counts for the grouped dictionary or legacy archive store.
    #[command(
        after_help = "STORE defaults to XDG_DATA_HOME/vlajmaji/stores/default, falling back to HOME/.local/share/vlajmaji/stores/default when XDG_DATA_HOME is unset, empty, or relative. Info only opens an existing store and never creates or downloads it. The default location is intended for a future dictionary-of-dictionaries; the initial Lensisku-only store is provisional."
    )]
    Info { store: Option<PathBuf> },
    /// Build a fresh grouped-dictionary store (default: XDG/HOME grouped store).
    #[command(
        after_help = "STORE defaults to XDG_DATA_HOME/vlajmaji/stores/default, falling back to HOME/.local/share/vlajmaji/stores/default when XDG_DATA_HOME is unset, empty, or relative. An explicit STORE overrides the environment. Build creates missing default parent directories but never replaces an existing store. The default location is intended for a future dictionary-of-dictionaries; the initial Lensisku-only store is provisional."
    )]
    DictionaryBuild {
        input_json: std::path::PathBuf,
        store: Option<PathBuf>,
    },
    /// Query the grouped store (default: XDG/HOME path); reads --request or stdin.
    #[command(
        after_help = "STORE defaults to XDG_DATA_HOME/vlajmaji/stores/default, falling back to HOME/.local/share/vlajmaji/stores/default when XDG_DATA_HOME is unset, empty, or relative. Query only opens an existing store and never creates or downloads it. The default location is intended for a future dictionary-of-dictionaries; the initial Lensisku-only store is provisional."
    )]
    DictionaryQuery {
        store: Option<PathBuf>,
        #[arg(long)]
        request: Option<String>,
    },
    /// Normalize a pinned raw jbotci export into the complete versioned dictionary document.
    JbotciImport {
        raw: std::path::PathBuf,
        metadata: std::path::PathBuf,
        output: std::path::PathBuf,
        #[arg(long, default_value = JBOTCI_PINNED_REVISION)]
        revision: String,
        #[arg(long, default_value = JBOTCI_PINNED_RAW_SHA256)]
        raw_sha256: String,
        #[arg(long, default_value = JBOTCI_PINNED_METADATA_SHA256)]
        metadata_sha256: String,
    },
    /// Validate the raw export and optionally compare it with a normalized release document.
    JbotciCheck {
        raw: std::path::PathBuf,
        metadata: std::path::PathBuf,
        #[arg(long)]
        release: Option<std::path::PathBuf>,
        #[arg(long, default_value = JBOTCI_PINNED_REVISION)]
        revision: String,
        #[arg(long, default_value = JBOTCI_PINNED_RAW_SHA256)]
        raw_sha256: String,
        #[arg(long, default_value = JBOTCI_PINNED_METADATA_SHA256)]
        metadata_sha256: String,
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
        Command::Info { store } => info(store)?,
        Command::DictionaryBuild { input_json, store } => {
            let bytes = std::fs::read(&input_json).map_err(|error| {
                structured_error(
                    "io",
                    format!("read dictionary JSON {}: {error}", input_json.display()),
                )
            })?;
            let selected = resolve_store(store)?;
            let store_text = selected.path.to_str().ok_or_else(|| {
                structured_error(
                    "store_path",
                    "dictionary-build requires a UTF-8 store path for its JSON response",
                )
            })?;
            if selected.is_default {
                create_store_parents(&selected.path).map_err(|error| {
                    structured_error(
                        "io",
                        format!("create default store parent directories: {error}"),
                    )
                })?;
            }
            let engine = DictionarySearchEngine::build(&selected.path, &bytes)
                .map_err(|error| CliError::Structured(error.report()))?;
            let (captures, occurrences, entries, assertions) = engine.dictionary_counts();
            serde_json::json!({
                "operation":"dictionary-build",
                "version":1,
                "state_sha256":engine.state_sha256(),
                "store":store_text,
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
            let selected = resolve_store(store)?;
            let engine = DictionarySearchEngine::open(&selected.path)
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
        Command::JbotciImport {
            raw,
            metadata,
            output,
            revision,
            raw_sha256,
            metadata_sha256,
        } => {
            let normalized =
                normalize_jbotci(&raw, &metadata, revision, raw_sha256, metadata_sha256)?;
            let document = normalized
                .dictionary()
                .to_json_bytes()
                .map_err(|error| structured_error("jbotci_dictionary", error.to_string()))?;
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&output)
                .map_err(|error| {
                    structured_error(
                        "io",
                        format!("create {} without replacement: {error}", output.display()),
                    )
                })?;
            file.write_all(&document)
                .and_then(|()| file.write_all(b"\n"))
                .and_then(|()| file.flush())
                .map_err(|error| {
                    structured_error("io", format!("write {}: {error}", output.display()))
                })?;
            serde_json::json!({"operation":"jbotci-import","version":1,"output":output,"report":normalized.report()})
        }
        Command::JbotciCheck {
            raw,
            metadata,
            release,
            revision,
            raw_sha256,
            metadata_sha256,
        } => {
            let normalized =
                normalize_jbotci(&raw, &metadata, revision, raw_sha256, metadata_sha256)?;
            let report = normalized.report();
            if !report.definition_count_matches || !report.entry_count_matches {
                return Err(structured_error(
                    "jbotci_accounting",
                    "supplier metadata counts do not match the observed export",
                ));
            }
            let release_matches = if let Some(path) = release {
                let bytes = std::fs::read(&path).map_err(|error| {
                    structured_error("io", format!("read {}: {error}", path.display()))
                })?;
                let supplied = Dictionary::from_json_bytes(&bytes)
                    .map_err(|error| structured_error("jbotci_dictionary", error.to_string()))?;
                let expected = normalized
                    .dictionary()
                    .to_json_bytes()
                    .map_err(|error| structured_error("jbotci_dictionary", error.to_string()))?;
                let actual = supplied
                    .to_json_bytes()
                    .map_err(|error| structured_error("jbotci_dictionary", error.to_string()))?;
                if expected != actual {
                    return Err(structured_error(
                        "jbotci_release_mismatch",
                        "normalized release does not match the pinned raw export and metadata",
                    ));
                }
                true
            } else {
                false
            };
            serde_json::json!({"operation":"jbotci-check","version":1,"release_checked":release_matches,"report":report})
        }
    };
    Ok(output)
}

#[derive(Debug)]
struct ResolvedStore {
    path: PathBuf,
    is_default: bool,
}

fn resolve_store(explicit: Option<PathBuf>) -> Result<ResolvedStore, CliError> {
    if let Some(path) = explicit {
        return Ok(ResolvedStore {
            path,
            is_default: false,
        });
    }

    let path = default_store_path(std::env::var_os("XDG_DATA_HOME"), std::env::var_os("HOME"))?;
    Ok(ResolvedStore {
        path,
        is_default: true,
    })
}

fn default_store_path(
    xdg: Option<std::ffi::OsString>,
    home: Option<std::ffi::OsString>,
) -> Result<PathBuf, CliError> {
    let xdg = xdg.filter(|value| !value.is_empty());
    let base = xdg
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .or_else(|| {
            home.filter(|value| !value.is_empty())
                .map(PathBuf::from)
                .filter(|path| path.is_absolute())
                .map(|home| home.join(".local").join("share"))
        })
        .ok_or_else(|| {
            structured_error(
                "store_path",
                "no default store path: set an absolute XDG_DATA_HOME or HOME",
            )
        })?;

    Ok(base.join("vlajmaji").join("stores").join("default"))
}

fn create_store_parents(store: &Path) -> io::Result<()> {
    let parent = store
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let mut builder = std::fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    builder.create(parent)
}

fn info(explicit: Option<PathBuf>) -> Result<serde_json::Value, CliError> {
    let selected = resolve_store(explicit)?;
    let grouped_marker = marker_exists(
        &selected.path.join("dictionary.sqlite3"),
        selected.is_default,
    )?;
    let legacy_marker = marker_exists(&selected.path.join("index.sqlite3"), selected.is_default)?;

    if grouped_marker && legacy_marker {
        return Err(structured_error(
            "store_format",
            "store path contains both grouped-dictionary and legacy archive markers",
        ));
    }
    if grouped_marker {
        let store = vlajmaji_store::DictionaryStore::open(&selected.path)
            .map_err(|error| structured_error("store", error.to_string()))?;
        let dictionary = store.dictionary();
        let source_records = dictionary.registry().document().sources;
        let occurrences = dictionary
            .captures()
            .iter()
            .map(|capture| capture.occurrences.len())
            .sum::<usize>();
        return Ok(serde_json::json!({
            "operation":"info",
            "version":1,
            "format":"dictionary",
            "captures":dictionary.captures().len(),
            "occurrences":occurrences,
            "entries":dictionary.metadata().entries.len(),
            "assertions":dictionary.metadata().assertions.len(),
            "source_count":source_records.len(),
            "sources":source_records
        }));
    }
    if selected.is_default && !legacy_marker {
        return Err(structured_error(
            "store",
            format!("default store does not exist: {}", selected.path.display()),
        ));
    }
    if legacy_marker || !selected.is_default {
        let engine = SearchEngine::open(&selected.path).map_err(|error| {
            let message = format!("open store {}: {error}", selected.path.display());
            if selected.is_default {
                structured_error("store", message)
            } else {
                CliError::Legacy(message)
            }
        })?;
        let (records, snapshots) = engine.corpus_counts();
        return Ok(serde_json::json!({
            "records":records,
            "snapshots":snapshots,
            "morphology_enabled":vlajmaji_search::morphology_enabled()
        }));
    }
    Err(structured_error(
        "store",
        format!("default store does not exist: {}", selected.path.display()),
    ))
}

fn marker_exists(path: &Path, is_default: bool) -> Result<bool, CliError> {
    match std::fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => {
            let message = format!(
                "inspect {} store marker {}: {error}",
                if is_default { "default" } else { "explicit" },
                path.display()
            );
            if is_default {
                Err(structured_error("io", message))
            } else {
                Err(CliError::Legacy(message))
            }
        }
    }
}

fn normalize_jbotci(
    raw_path: &std::path::Path,
    metadata_path: &std::path::Path,
    revision: String,
    expected_raw_sha256: String,
    expected_metadata_sha256: String,
) -> Result<vlajmaji_lexicon::JbotciImport, CliError> {
    let raw = std::fs::read(raw_path)
        .map_err(|error| structured_error("io", format!("read {}: {error}", raw_path.display())))?;
    let metadata = std::fs::read(metadata_path).map_err(|error| {
        structured_error("io", format!("read {}: {error}", metadata_path.display()))
    })?;
    import_jbotci_export(
        &raw,
        &metadata,
        &JbotciImportOptions {
            revision,
            expected_raw_sha256,
            expected_metadata_sha256,
        },
    )
    .map_err(map_jbotci_error)
}

fn map_jbotci_error(error: JbotciImportError) -> CliError {
    let code = match &error {
        JbotciImportError::InvalidOptions(_) => "jbotci_options",
        JbotciImportError::HashMismatch { .. } => "jbotci_hash_mismatch",
        JbotciImportError::InvalidUtf8 { .. } => "jbotci_utf8",
        JbotciImportError::InvalidJson(_) => "jbotci_json",
        JbotciImportError::InvalidMetadata(_) | JbotciImportError::UnsupportedMetadata(_) => {
            "jbotci_metadata"
        }
        JbotciImportError::Registry(_) => "jbotci_registry",
        JbotciImportError::Dictionary(_) => "jbotci_dictionary",
        _ => "jbotci_import",
    };
    structured_error(code, error.to_string())
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

#[cfg(test)]
mod default_path_tests {
    use super::{CliError, default_store_path, marker_exists};

    fn test_scratch() -> std::path::PathBuf {
        let path =
            std::env::temp_dir().join(format!("vlajmaji-cli-marker-unit-{}", std::process::id()));
        std::fs::create_dir(&path).expect("create marker-test scratch directory");
        path
    }

    #[cfg(unix)]
    #[test]
    fn test_default_path_preserves_native_non_utf8_bytes() {
        use std::{ffi::OsString, os::unix::ffi::OsStringExt, path::PathBuf};

        let xdg = OsString::from_vec(b"/tmp/vlajmaji-xdg-\xff".to_vec());
        let expected = PathBuf::from(&xdg).join("vlajmaji/stores/default");
        assert_eq!(default_store_path(Some(xdg), None).unwrap(), expected);
    }

    #[test]
    fn test_marker_exists_distinguishes_absence_and_propagates_metadata_errors() {
        let root = test_scratch();
        let marker = root.join("dictionary.sqlite3");
        std::fs::write(&marker, b"marker").expect("write marker");
        assert!(marker_exists(&marker, true).unwrap());

        let missing = root.join("missing.sqlite3");
        assert!(!marker_exists(&missing, true).unwrap());
        assert!(!marker_exists(&missing, false).unwrap());

        let ancestor = root.join("not-a-directory");
        std::fs::write(&ancestor, b"file").expect("write non-directory ancestor");
        let inaccessible_marker = ancestor.join("dictionary.sqlite3");
        let default_error = marker_exists(&inaccessible_marker, true)
            .expect_err("default marker inspection error propagates");
        let CliError::Structured(report) = default_error else {
            panic!("default marker error should remain structured");
        };
        assert_eq!(report.code, "io");
        assert!(report.message.contains("inspect default store marker"));

        let explicit_error = marker_exists(&inaccessible_marker, false)
            .expect_err("explicit marker inspection error propagates");
        let CliError::Legacy(message) = explicit_error else {
            panic!("explicit legacy marker error should preserve plaintext shape");
        };
        assert!(message.contains("inspect explicit store marker"));
        assert!(message.contains(&inaccessible_marker.display().to_string()));

        std::fs::remove_dir_all(root).expect("remove marker-test scratch directory");
    }
}
