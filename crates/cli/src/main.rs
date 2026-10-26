//! JSON command-line interface for the research lexical-search prototype.

use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use clap::{Args as ClapArgs, Parser, Subcommand, ValueEnum};
use vlajmaji_lexicon::{
    Dictionary, JBOTCI_PINNED_METADATA_SHA256, JBOTCI_PINNED_RAW_SHA256, JBOTCI_PINNED_REVISION,
    JbotciImportError, JbotciImportOptions, LA_LOJBAN_PINNED_RAW_SHA256, LA_LOJBAN_PINNED_REVISION,
    LLG_BASE_LOCATOR, LLG_CMAVO_SHA256, LLG_GISMU_SHA256, LLG_NOTICE_SHA256, LLG_RAFSI_SHA256,
    LaLojbanImportError, LaLojbanImportOptions, LlgImportError, LlgImportOptions, PackId,
    PackPayloadKind, SUTYSISKU_PINNED_RAW_SHA256, SUTYSISKU_PINNED_REVISION, SutysiskuImportError,
    SutysiskuImportOptions, import_jbotci_export, import_la_lojban_export, import_llg_lists,
    import_sutysisku_export, seal_release,
};
use vlajmaji_search::{
    DictionaryErrorReport, DictionaryRequest, DictionarySearchEngine, Query, SearchEngine,
    SearchOptions,
};
use vlajmaji_store::{
    CollectionCatalogue, CollectionError, CollectionInput, CollectionRequest, CollectionSelection,
    DictionaryStoreVerification, DictionaryStoreVerificationCheck,
    DictionaryStoreVerificationReport,
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
    /// Explicitly audit one or all integrity boundaries of an existing grouped store.
    #[command(
        after_help = "STORE defaults to XDG_DATA_HOME/vlajmaji/stores/default, falling back to HOME/.local/share/vlajmaji/stores/default. An explicit STORE overrides the environment. Verify only opens an existing store and never creates or downloads it. A named check certifies only that scope; `all` attempts every check. The default location is intended for a future dictionary-of-dictionaries; the initial Lensisku-only store is provisional."
    )]
    Verify {
        #[arg(value_enum)]
        check: VerifyCheck,
        store: Option<PathBuf>,
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
    /// Prepare an ignored local pack payload without opening a dictionary store.
    DataPack {
        #[command(subcommand)]
        command: DataPackCommand,
    },
    /// Register or install explicitly selected local source packs.
    #[command(
        after_help = "Collection commands use only their explicit ROOT; they do not resolve or modify HOME/XDG defaults, select sources, or download data. Register records metadata only. Install creates and verifies a new Unified generation. Manifest paths are relative to the manifest's parent; Synthetic inputs require --allow-synthetic."
    )]
    Collections {
        #[command(subcommand)]
        command: CollectionsCommand,
    },
}

#[derive(Debug, Subcommand)]
enum CollectionsCommand {
    /// Retain strict source/reference metadata without copying release payloads.
    #[command(
        after_help = "Use --manifest with --pack ID (repeatable) or --all. --all requires all four pack records. Registration copies no raw/release payload and does not install a generation."
    )]
    Register(CollectionMutationArgs),
    /// Add selected source packs into one verified Unified store generation.
    #[command(
        after_help = "Use --manifest with --pack ID (repeatable) or --all. --all requires all four pack records. A different installed release reports update_required; this command does not alter source selection or the user's default store."
    )]
    Install(CollectionMutationArgs),
    /// List registrations and the current installed generation.
    #[command(
        after_help = "Read-only summary of an existing explicit catalogue. This does not create a catalogue or rehash pack/store payloads."
    )]
    List { root: PathBuf },
    /// Inspect one source pack's registration, terms and reference summary.
    #[command(
        after_help = "Read-only minimum-format and file-presence summary. Use the explicit dictionary verify command for a full store audit."
    )]
    Inspect { root: PathBuf, pack: String },
}

#[derive(Debug, ClapArgs)]
struct CollectionMutationArgs {
    root: PathBuf,
    /// Strict batch manifest; relative paths resolve beside this file.
    #[arg(long, conflicts_with_all = ["release", "seal"])]
    manifest: Option<PathBuf>,
    /// Select all four pack IDs from a manifest.
    #[arg(long, requires = "manifest", conflicts_with = "pack")]
    all: bool,
    /// Select a pack ID from a manifest, repeatable.
    #[arg(long)]
    pack: Vec<String>,
    /// One directly supplied sealed release (direct mode only).
    #[arg(long, requires = "seal")]
    release: Option<PathBuf>,
    /// One directly supplied parsed seal (direct mode only).
    #[arg(long, requires = "release")]
    seal: Option<PathBuf>,
    /// Explicitly permit Synthetic payload declarations.
    #[arg(long)]
    allow_synthetic: bool,
}

#[derive(Debug, Subcommand)]
enum DataPackCommand {
    /// Materialize a selected pack from local inputs.
    Materialize {
        #[command(subcommand)]
        pack: Box<MaterializePack>,
    },
    /// Prepare a small synthetic pack for CI and local feature checks.
    Fixture {
        #[command(subcommand)]
        pack: FixturePack,
    },
}

#[derive(Debug, Subcommand)]
enum MaterializePack {
    /// Import the exact local jbotci raw export and supplier metadata.
    Jbotci {
        raw: PathBuf,
        metadata: PathBuf,
        output: PathBuf,
        #[arg(long, default_value = JBOTCI_PINNED_REVISION)]
        revision: String,
        #[arg(long, default_value = JBOTCI_PINNED_RAW_SHA256)]
        raw_sha256: String,
        #[arg(long, default_value = JBOTCI_PINNED_METADATA_SHA256)]
        metadata_sha256: String,
    },
    /// Import the three local LLG lists and their shared notice.
    Llg {
        gismu: PathBuf,
        cmavo: PathBuf,
        rafsi: PathBuf,
        notice: PathBuf,
        output: PathBuf,
        #[arg(long, default_value = LLG_GISMU_SHA256)]
        gismu_sha256: String,
        #[arg(long, default_value = LLG_CMAVO_SHA256)]
        cmavo_sha256: String,
        #[arg(long, default_value = LLG_RAFSI_SHA256)]
        rafsi_sha256: String,
        #[arg(long, default_value = LLG_NOTICE_SHA256)]
        notice_sha256: String,
        #[arg(long, default_value = LLG_BASE_LOCATOR)]
        base_locator: String,
    },
    /// Import the pinned La-Lojban English Lensisku export.
    LaLojban {
        raw: PathBuf,
        output: PathBuf,
        #[arg(long, default_value = LA_LOJBAN_PINNED_REVISION)]
        revision: String,
        #[arg(long, default_value = LA_LOJBAN_PINNED_RAW_SHA256)]
        raw_sha256: String,
        #[arg(long)]
        raw_locator: Option<String>,
        #[arg(long)]
        notice_locator: Option<String>,
    },
    /// Import the pinned Sutysisku English JSON projection.
    Sutysisku {
        raw: PathBuf,
        output: PathBuf,
        #[arg(long, default_value = SUTYSISKU_PINNED_REVISION)]
        revision: String,
        #[arg(long, default_value = SUTYSISKU_PINNED_RAW_SHA256)]
        raw_sha256: String,
        #[arg(long)]
        raw_locator: Option<String>,
        #[arg(long)]
        notice_locator: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
enum FixturePack {
    /// Build a synthetic three-row Lensisku fixture through the real importer.
    Jbotci { output: PathBuf },
    /// Build a small synthetic gismu/cmavo/rafsi fixture through the real importer.
    Llg { output: PathBuf },
    /// Build a small synthetic La-Lojban fixture through the real importer.
    LaLojban { output: PathBuf },
    /// Build a tiny synthetic Sutysisku pack through the real importer.
    Sutysisku { output: PathBuf },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum VerifyCheck {
    Schema,
    Metadata,
    Hashes,
    Model,
    Fks,
    Projection,
    All,
}

impl From<VerifyCheck> for DictionaryStoreVerificationCheck {
    fn from(value: VerifyCheck) -> Self {
        match value {
            VerifyCheck::Schema => Self::Schema,
            VerifyCheck::Metadata => Self::Metadata,
            VerifyCheck::Hashes => Self::Hashes,
            VerifyCheck::Model => Self::Model,
            VerifyCheck::Fks => Self::Fks,
            VerifyCheck::Projection => Self::Projection,
            VerifyCheck::All => Self::All,
        }
    }
}

#[derive(Debug)]
enum CliError {
    Legacy(String),
    Structured(DictionaryErrorReport),
    Verification(DictionaryStoreVerificationReport),
    CollectionFailure(Box<vlajmaji_store::CollectionFailureReport>),
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
        Command::Verify { check, store } => {
            let selected = resolve_store(store)?;
            let report = DictionaryStoreVerification::verify(&selected.path, check.into())
                .map_err(|error| structured_error("store", error.to_string()))?;
            if !report.ok {
                return Err(CliError::Verification(report));
            }
            serde_json::to_value(report)
                .map_err(|error| structured_error("verification_json", error.to_string()))?
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
        Command::DataPack { command } => match command {
            DataPackCommand::Materialize { pack } => match *pack {
                MaterializePack::Jbotci {
                    raw,
                    metadata,
                    output,
                    revision,
                    raw_sha256,
                    metadata_sha256,
                } => {
                    let raw_bytes = std::fs::read(&raw).map_err(|error| {
                        structured_error("io", format!("read {}: {error}", raw.display()))
                    })?;
                    let metadata_bytes = std::fs::read(&metadata).map_err(|error| {
                        structured_error("io", format!("read {}: {error}", metadata.display()))
                    })?;
                    materialize_jbotci_bytes(
                        &raw_bytes,
                        &metadata_bytes,
                        &output,
                        revision,
                        raw_sha256,
                        metadata_sha256,
                        PackPayloadKind::Full,
                    )?
                }
                MaterializePack::Llg {
                    gismu,
                    cmavo,
                    rafsi,
                    notice,
                    output,
                    gismu_sha256,
                    cmavo_sha256,
                    rafsi_sha256,
                    notice_sha256,
                    base_locator,
                } => {
                    let gismu_bytes = std::fs::read(&gismu).map_err(|error| {
                        structured_error("io", format!("read {}: {error}", gismu.display()))
                    })?;
                    let cmavo_bytes = std::fs::read(&cmavo).map_err(|error| {
                        structured_error("io", format!("read {}: {error}", cmavo.display()))
                    })?;
                    let rafsi_bytes = std::fs::read(&rafsi).map_err(|error| {
                        structured_error("io", format!("read {}: {error}", rafsi.display()))
                    })?;
                    let notice_bytes = std::fs::read(&notice).map_err(|error| {
                        structured_error("io", format!("read {}: {error}", notice.display()))
                    })?;
                    materialize_llg_bytes(
                        &gismu_bytes,
                        &cmavo_bytes,
                        &rafsi_bytes,
                        &notice_bytes,
                        &output,
                        LlgImportOptions {
                            expected_gismu_sha256: gismu_sha256,
                            expected_cmavo_sha256: cmavo_sha256,
                            expected_rafsi_sha256: rafsi_sha256,
                            expected_notice_sha256: notice_sha256,
                            base_locator,
                        },
                        PackPayloadKind::Full,
                    )?
                }
                MaterializePack::LaLojban {
                    raw,
                    output,
                    revision,
                    raw_sha256,
                    raw_locator,
                    notice_locator,
                } => {
                    let raw_bytes = std::fs::read(&raw).map_err(|error| {
                        structured_error("io", format!("read {}: {error}", raw.display()))
                    })?;
                    materialize_la_lojban_bytes(
                        &raw_bytes,
                        &output,
                        LaLojbanImportOptions {
                            revision,
                            expected_raw_sha256: raw_sha256,
                            raw_locator,
                            notice_locator,
                        },
                        PackPayloadKind::Full,
                    )?
                }
                MaterializePack::Sutysisku {
                    raw,
                    output,
                    revision,
                    raw_sha256,
                    raw_locator,
                    notice_locator,
                } => {
                    let raw_bytes = std::fs::read(&raw).map_err(|error| {
                        structured_error("io", format!("read {}: {error}", raw.display()))
                    })?;
                    materialize_sutysisku_bytes(
                        &raw_bytes,
                        &output,
                        SutysiskuImportOptions {
                            revision,
                            expected_raw_sha256: raw_sha256,
                            raw_locator,
                            notice_locator,
                        },
                        PackPayloadKind::Full,
                    )?
                }
            },
            DataPackCommand::Fixture {
                pack: FixturePack::Jbotci { output },
            } => {
                let raw = br#"[{"word":"farvi","word_type":"gismu","definition":"to grow","definition_id":7},{"word":"farvi","word_type":"gismu","definition":"to develop","definition_id":8},{"word":"farvi","word_type":"wiki","definition":"community note"}]"#;
                let metadata = jbotci_fixture_metadata(raw);
                materialize_jbotci_bytes(
                    raw,
                    &metadata,
                    &output,
                    "0000000000000000000000000000000000000000".into(),
                    sha256(raw),
                    sha256(&metadata),
                    PackPayloadKind::Synthetic,
                )?
            }
            DataPackCommand::Fixture {
                pack: FixturePack::Llg { output },
            } => {
                let (gismu, cmavo, rafsi, notice) = llg_fixture();
                materialize_llg_bytes(
                    &gismu,
                    &cmavo,
                    &rafsi,
                    &notice,
                    &output,
                    LlgImportOptions {
                        expected_gismu_sha256: sha256(&gismu),
                        expected_cmavo_sha256: sha256(&cmavo),
                        expected_rafsi_sha256: sha256(&rafsi),
                        expected_notice_sha256: sha256(&notice),
                        base_locator: "https://example.invalid/llg".into(),
                    },
                    PackPayloadKind::Synthetic,
                )?
            }
            DataPackCommand::Fixture {
                pack: FixturePack::LaLojban { output },
            } => {
                let raw = br#"[{"word":"farvi","word_type":"gismu","definition":"to grow","definition_id":7},{"word":"farvi","word_type":"gismu","definition":"to develop","definition_id":8},{"word":"farvi","word_type":"wiki","definition":"community note"}]"#;
                materialize_la_lojban_bytes(
                    raw,
                    &output,
                    LaLojbanImportOptions {
                        revision: "0000000000000000000000000000000000000000".into(),
                        expected_raw_sha256: sha256(raw),
                        raw_locator: Some("https://example.invalid/la-lojban.json".into()),
                        notice_locator: Some("https://example.invalid/README.md".into()),
                    },
                    PackPayloadKind::Synthetic,
                )?
            }
            DataPackCommand::Fixture {
                pack: FixturePack::Sutysisku { output },
            } => {
                let raw = br#"{"farvi":{"d":"to grow","t":"gismu","r":["far","fari"]},"note":{"d":"","t":"wiki","n":null}}"#;
                materialize_sutysisku_bytes(
                    raw,
                    &output,
                    SutysiskuImportOptions {
                        revision: "0000000000000000000000000000000000000000".into(),
                        expected_raw_sha256: sha256(raw),
                        raw_locator: Some("https://example.invalid/sutysisku.json".into()),
                        notice_locator: Some("https://example.invalid/README.md".into()),
                    },
                    PackPayloadKind::Synthetic,
                )?
            }
        },
        Command::Collections { command } => collections_command(command)?,
    };
    Ok(output)
}

fn collections_command(command: CollectionsCommand) -> Result<serde_json::Value, CliError> {
    match command {
        CollectionsCommand::Register(args) => collection_mutation(args, "register"),
        CollectionsCommand::Install(args) => collection_mutation(args, "install"),
        CollectionsCommand::List { root } => {
            let catalogue = CollectionCatalogue::open(&root)
                .map_err(|error| collection_cli_error(error, "list"))?;
            serde_json::to_value(
                catalogue
                    .list()
                    .map_err(|error| collection_cli_error(error, "list"))?,
            )
            .map_err(|error| structured_error("collections_json", error.to_string()))
        }
        CollectionsCommand::Inspect { root, pack } => {
            let pack = parse_collection_pack(&pack)
                .map_err(|error| collection_cli_error(error, "inspect"))?;
            let catalogue = CollectionCatalogue::open(&root)
                .map_err(|error| collection_cli_error(error, "inspect"))?;
            serde_json::to_value(
                catalogue
                    .inspect(pack)
                    .map_err(|error| collection_cli_error(error, "inspect"))?,
            )
            .map_err(|error| structured_error("collections_json", error.to_string()))
        }
    }
}

fn collection_mutation(
    args: CollectionMutationArgs,
    operation: &str,
) -> Result<serde_json::Value, CliError> {
    let request = if let Some(manifest) = args.manifest.as_ref() {
        if args.release.is_some() || args.seal.is_some() {
            return Err(structured_error(
                "collections_request",
                "manifest mode cannot be combined with release or seal",
            ));
        }
        let selection = if args.all {
            if !args.pack.is_empty() {
                return Err(structured_error(
                    "collections_request",
                    "--all cannot be combined with --pack",
                ));
            }
            CollectionSelection::All
        } else if args.pack.is_empty() {
            return Err(structured_error(
                "collections_request",
                "manifest mode requires --all or one or more --pack",
            ));
        } else {
            CollectionSelection::Packs(
                args.pack
                    .iter()
                    .map(|pack| parse_collection_pack(pack))
                    .collect::<Result<_, _>>()
                    .map_err(|error| collection_cli_error(error, operation))?,
            )
        };
        read_collection_manifest(manifest, selection, args.allow_synthetic, operation)?
    } else {
        if args.all || args.pack.len() != 1 {
            return Err(structured_error(
                "collections_request",
                "direct sealed mode requires exactly one --pack and does not accept --all",
            ));
        }
        let pack = parse_collection_pack(&args.pack[0])
            .map_err(|error| collection_cli_error(error, operation))?;
        let (release, seal) = match (args.release.clone(), args.seal.clone()) {
            (Some(release), Some(seal)) => (release, seal),
            _ => {
                return Err(structured_error(
                    "collections_request",
                    "direct sealed mode requires both --release and --seal",
                ));
            }
        };
        CollectionRequest::new(
            vec![CollectionInput::SealedFiles {
                pack_id: pack,
                release,
                seal,
            }],
            CollectionSelection::Packs(vec![pack]),
            args.allow_synthetic,
        )
        .map_err(|error| collection_cli_error(error, operation))?
    };
    let report = match operation {
        "register" => CollectionCatalogue::register(&args.root, request)
            .map_err(|error| collection_cli_error(error, operation))?,
        "install" => CollectionCatalogue::install(&args.root, request)
            .map_err(|error| collection_cli_error(error, operation))?,
        _ => {
            return Err(structured_error(
                "collections_operation",
                "unsupported collection mutation",
            ));
        }
    };
    serde_json::to_value(report)
        .map_err(|error| structured_error("collections_json", error.to_string()))
}

fn parse_collection_pack(value: &str) -> Result<PackId, CollectionError> {
    match value {
        "lensisku-jbotci" => Ok(PackId::LensiskuJbotci),
        "llg" => Ok(PackId::Llg),
        "la-lojban" => Ok(PackId::LaLojban),
        "sutysisku" => Ok(PackId::Sutysisku),
        _ => Err(CollectionError::InvalidRequest(format!(
            "unknown pack id {value:?}"
        ))),
    }
}

fn collection_cli_error(error: CollectionError, operation: &str) -> CliError {
    CliError::CollectionFailure(Box::new(error.report(operation)))
}

fn read_collection_manifest(
    path: &Path,
    selection: CollectionSelection,
    allow_synthetic: bool,
    operation: &str,
) -> Result<CollectionRequest<'static>, CliError> {
    let bytes = std::fs::read(path).map_err(|error| {
        structured_error(
            "io",
            format!("read collection manifest {}: {error}", path.display()),
        )
    })?;
    CollectionRequest::from_manifest(&bytes, path, selection, allow_synthetic)
        .map_err(|error| collection_cli_error(error, operation))
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
        let summary = store
            .summary()
            .map_err(|error| structured_error("store", error.to_string()))?;
        return Ok(serde_json::json!({
            "operation":"info",
            "version":1,
            "format":"dictionary",
            "captures":summary.captures,
            "occurrences":summary.occurrences,
            "entries":summary.entries,
            "assertions":summary.assertions,
            "source_count":summary.sources.len(),
            "sources":summary.sources
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

fn materialize_jbotci_bytes(
    raw: &[u8],
    metadata: &[u8],
    output_path: &Path,
    revision: String,
    expected_raw_sha256: String,
    expected_metadata_sha256: String,
    payload_kind: PackPayloadKind,
) -> Result<serde_json::Value, CliError> {
    let normalized = import_jbotci_export(
        raw,
        metadata,
        &JbotciImportOptions {
            revision,
            expected_raw_sha256,
            expected_metadata_sha256,
        },
    )
    .map_err(map_jbotci_error)?;
    let report = normalized.report();
    if !report.definition_count_matches || !report.entry_count_matches {
        return Err(structured_error(
            "jbotci_accounting",
            "supplier metadata counts do not match observed raw rows and eligible spellings",
        ));
    }
    let sealed = seal_release(
        PackId::LensiskuJbotci,
        payload_kind,
        normalized.dictionary(),
    )
    .map_err(|error| structured_error("pack", error.to_string()))?;
    let report_bytes = serde_json::to_vec_pretty(report)
        .map_err(|error| structured_error("report_json", error.to_string()))?;

    std::fs::create_dir(output_path).map_err(|error| {
        structured_error(
            "io",
            format!(
                "create fresh pack directory {}: {error}",
                output_path.display()
            ),
        )
    })?;
    write_new(&output_path.join("release.json"), &sealed.release)?;
    write_new(&output_path.join("report.json"), &report_bytes)?;
    // The seal is the completion marker: publish it only after all pack data
    // has been written successfully. A seal-write failure can still leave the
    // release and report files behind, but the failed command never reports a
    // completed pack.
    write_new(&output_path.join("seal.json"), &sealed.seal)?;
    let payload = match payload_kind {
        PackPayloadKind::Synthetic => "synthetic",
        PackPayloadKind::Full => "full",
    };
    Ok(serde_json::json!({
        "operation":"data-pack-materialize",
        "version":1,
        "pack_id":PackId::LensiskuJbotci.as_str(),
        "payload_kind":payload,
        "release_sha256":sha256(&sealed.release),
        "report":report
    }))
}

fn materialize_llg_bytes(
    gismu: &[u8],
    cmavo: &[u8],
    rafsi: &[u8],
    notice: &[u8],
    output_path: &Path,
    options: LlgImportOptions,
    payload_kind: PackPayloadKind,
) -> Result<serde_json::Value, CliError> {
    let normalized =
        import_llg_lists(gismu, cmavo, rafsi, notice, &options).map_err(map_llg_error)?;
    let report = normalized.report();
    let sealed = seal_release(PackId::Llg, payload_kind, normalized.dictionary())
        .map_err(|error| structured_error("pack", error.to_string()))?;
    let report_bytes = serde_json::to_vec_pretty(report)
        .map_err(|error| structured_error("report_json", error.to_string()))?;

    std::fs::create_dir(output_path).map_err(|error| {
        structured_error(
            "io",
            format!(
                "create fresh pack directory {}: {error}",
                output_path.display()
            ),
        )
    })?;
    write_new(&output_path.join("release.json"), &sealed.release)?;
    write_new(&output_path.join("report.json"), &report_bytes)?;
    write_new(&output_path.join("seal.json"), &sealed.seal)?;
    let payload = match payload_kind {
        PackPayloadKind::Synthetic => "synthetic",
        PackPayloadKind::Full => "full",
    };
    Ok(serde_json::json!({
        "operation":"data-pack-materialize",
        "version":1,
        "pack_id":PackId::Llg.as_str(),
        "payload_kind":payload,
        "release_sha256":sha256(&sealed.release),
        "report":report
    }))
}

fn materialize_la_lojban_bytes(
    raw: &[u8],
    output_path: &Path,
    options: LaLojbanImportOptions,
    payload_kind: PackPayloadKind,
) -> Result<serde_json::Value, CliError> {
    let normalized = import_la_lojban_export(raw, &options).map_err(map_la_lojban_error)?;
    let report = normalized.report();
    let sealed = seal_release(PackId::LaLojban, payload_kind, normalized.dictionary())
        .map_err(|error| structured_error("pack", error.to_string()))?;
    let report_bytes = serde_json::to_vec_pretty(report)
        .map_err(|error| structured_error("report_json", error.to_string()))?;
    std::fs::create_dir(output_path).map_err(|error| {
        structured_error(
            "io",
            format!(
                "create fresh pack directory {}: {error}",
                output_path.display()
            ),
        )
    })?;
    write_new(&output_path.join("release.json"), &sealed.release)?;
    write_new(&output_path.join("report.json"), &report_bytes)?;
    // A seal is the completion marker and is written only after release/report.
    write_new(&output_path.join("seal.json"), &sealed.seal)?;
    let payload = match payload_kind {
        PackPayloadKind::Synthetic => "synthetic",
        PackPayloadKind::Full => "full",
    };
    Ok(serde_json::json!({
        "operation":"data-pack-materialize",
        "version":1,
        "pack_id":PackId::LaLojban.as_str(),
        "payload_kind":payload,
        "release_sha256":sha256(&sealed.release),
        "report":report
    }))
}

fn materialize_sutysisku_bytes(
    raw: &[u8],
    output_path: &Path,
    options: SutysiskuImportOptions,
    payload_kind: PackPayloadKind,
) -> Result<serde_json::Value, CliError> {
    let normalized = import_sutysisku_export(raw, &options).map_err(map_sutysisku_error)?;
    let report = normalized.report();
    let sealed = seal_release(PackId::Sutysisku, payload_kind, normalized.dictionary())
        .map_err(|error| structured_error("pack", error.to_string()))?;
    let report_bytes = serde_json::to_vec_pretty(report)
        .map_err(|error| structured_error("report_json", error.to_string()))?;
    std::fs::create_dir(output_path).map_err(|error| {
        structured_error(
            "io",
            format!(
                "create fresh pack directory {}: {error}",
                output_path.display()
            ),
        )
    })?;
    write_new(&output_path.join("release.json"), &sealed.release)?;
    write_new(&output_path.join("report.json"), &report_bytes)?;
    write_new(&output_path.join("seal.json"), &sealed.seal)?;
    let payload = match payload_kind {
        PackPayloadKind::Synthetic => "synthetic",
        PackPayloadKind::Full => "full",
    };
    Ok(serde_json::json!({
        "operation":"data-pack-materialize",
        "version":1,
        "pack_id":PackId::Sutysisku.as_str(),
        "payload_kind":payload,
        "release_sha256":sha256(&sealed.release),
        "report":report
    }))
}

fn llg_fixture() -> (Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>) {
    let mut gismu = b"00001 Lojban synthetic gismu list\r\n".to_vec();
    let mut row = vec![b' '; 158];
    row[1..6].copy_from_slice(b"farvi");
    row[7..10].copy_from_slice(b"far");
    row[11..14].copy_from_slice(b"fav");
    row[20..24].copy_from_slice(b"grow");
    row[41..47].copy_from_slice(b"growth");
    row[62..69].copy_from_slice(b"to grow");
    gismu.extend_from_slice(&row);
    gismu.extend_from_slice(b"\r\n");
    let cmavo = b"00001 Lojban synthetic cmavo list\r\ndo        BAI       because                                 because it is so\r\n".to_vec();
    let rafsi = b"far farvi growth\r\n".to_vec();
    let notice = b"Synthetic LLG notice; no redistribution claim.\r\n".to_vec();
    (gismu, cmavo, rafsi, notice)
}

fn map_llg_error(error: LlgImportError) -> CliError {
    let code = match &error {
        LlgImportError::InvalidOptions(_) => "llg_options",
        LlgImportError::HashMismatch { .. } => "llg_hash_mismatch",
        LlgImportError::InvalidUtf8 { .. } => "llg_utf8",
        LlgImportError::Registry(_) => "llg_registry",
        LlgImportError::Dictionary(_) => "llg_dictionary",
        _ => "llg_import",
    };
    structured_error(code, error.to_string())
}

fn jbotci_fixture_metadata(raw: &[u8]) -> Vec<u8> {
    format!(
        "language_tag = \"en\"\nlanguage_realname = \"English\"\nsource_language_tag = \"jbo\"\nformat = \"json\"\npositive_scores_only = false\nfilename = \"dictionary-en.json\"\nmetadata_url = \"https://example.invalid/languages\"\ndownload_url = \"https://example.invalid/synthetic-export\"\nlensisku_created_at = \"2026-10-04T03:07:08Z\"\nsha256 = \"{}\"\ndefinition_count = 3\nentry_count = 1\n",
        sha256(raw)
    )
    .into_bytes()
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<(), CliError> {
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| {
            structured_error(
                "io",
                format!("create {} without replacement: {error}", path.display()),
            )
        })?;
    file.write_all(bytes)
        .and_then(|()| file.flush())
        .map_err(|error| structured_error("io", format!("write {}: {error}", path.display())))
}

fn sha256(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
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

fn map_la_lojban_error(error: LaLojbanImportError) -> CliError {
    let code = match &error {
        LaLojbanImportError::InvalidOptions(_) => "la_lojban_options",
        LaLojbanImportError::HashMismatch { .. } => "la_lojban_hash_mismatch",
        LaLojbanImportError::InvalidUtf8 => "la_lojban_utf8",
        LaLojbanImportError::InvalidJson(_) => "la_lojban_json",
        LaLojbanImportError::Registry(_) => "la_lojban_registry",
        LaLojbanImportError::Dictionary(_) => "la_lojban_dictionary",
        _ => "la_lojban_import",
    };
    structured_error(code, error.to_string())
}

fn map_sutysisku_error(error: SutysiskuImportError) -> CliError {
    let code = match &error {
        SutysiskuImportError::InvalidOptions(_) => "sutysisku_options",
        SutysiskuImportError::HashMismatch { .. } => "sutysisku_hash_mismatch",
        SutysiskuImportError::InvalidUtf8 => "sutysisku_utf8",
        SutysiskuImportError::InvalidJson(_) => "sutysisku_json",
        SutysiskuImportError::Registry(_) => "sutysisku_registry",
        SutysiskuImportError::Dictionary(_) => "sutysisku_dictionary",
        _ => "sutysisku_import",
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
        Err(CliError::CollectionFailure(report)) => {
            let mut stderr = io::stderr().lock();
            if serde_json::to_writer(&mut stderr, &serde_json::json!({"error": report})).is_ok() {
                let _ = stderr.write_all(b"\n");
            }
            return std::process::ExitCode::FAILURE;
        }
        Err(CliError::Verification(report)) => {
            let mut stderr = io::stderr().lock();
            let envelope = serde_json::json!({
                "error":{"code":"verification_failed","message":"one or more verification checks failed or were blocked","sources":[]},
                "verification":report
            });
            if serde_json::to_writer(&mut stderr, &envelope).is_ok() {
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

#[cfg(test)]
mod collection_manifest_tests {
    use super::read_collection_manifest;
    use vlajmaji_store::CollectionSelection;

    #[test]
    fn test_manifest_duplicate_keys_rejects_escaped_equivalents_recursively() {
        let root = std::env::temp_dir().join(format!(
            "vlajmaji-manifest-duplicate-{}",
            std::process::id()
        ));
        std::fs::create_dir(&root).unwrap();
        let manifest = root.join("batch.json");
        for bytes in [
            br#"{"version":1,"version":1,"inputs":[]}"#.as_slice(),
            br#"{"version":1,"inputs":[{"kind":"la-lojban","pack_id":"la-lojban","payload_kind":"synthetic","raw":"raw","options":{"revision":"a","revis\u0069on":"b","expected_raw_sha256":"hash"}}]}"#.as_slice(),
        ] {
            std::fs::write(&manifest, bytes).unwrap();
            assert!(read_collection_manifest(&manifest, CollectionSelection::Packs(vec![vlajmaji_lexicon::PackId::LaLojban]), true, "register").is_err());
        }
        std::fs::write(&manifest, r#"{"version":1,"inputs":[{"kind":"la-lojban","pack_id":"la-lojban","payload_kind":"synthetic","raw":"λ","options":{"revision":"a","expected_raw_sha256":"hash"}}]}"#).unwrap();
        assert!(
            read_collection_manifest(
                &manifest,
                CollectionSelection::Packs(vec![vlajmaji_lexicon::PackId::LaLojban]),
                true,
                "register"
            )
            .is_ok()
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn test_manifest_validation_freezes_relative_paths_without_reading_them() {
        let root =
            std::env::temp_dir().join(format!("vlajmaji-manifest-unit-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let manifest = root.join("batch.json");
        std::fs::write(&manifest, br#"{"version":1,"inputs":[{"kind":"sealed","pack_id":"llg","release":"missing release.json","seal":"missing seal.json"}]}"#).unwrap();
        let _request = read_collection_manifest(
            &manifest,
            CollectionSelection::Packs(vec![vlajmaji_lexicon::PackId::Llg]),
            true,
            "register",
        )
        .unwrap();
        assert!(!root.join("missing release.json").exists());
        std::fs::remove_dir_all(root).unwrap();
    }
}
