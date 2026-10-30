//! JSON command-line interface for the research lexical-search prototype.

mod query_output;

use std::ffi::OsString;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use clap::{Args as ClapArgs, Parser, Subcommand, ValueEnum};
use query_output::QueryOutput;
use vlajmaji_lexicon::EntityId;
use vlajmaji_lexicon::{
    Dictionary, JBOTCI_PINNED_METADATA_SHA256, JBOTCI_PINNED_RAW_SHA256, JBOTCI_PINNED_REVISION,
    JbotciImportError, JbotciImportOptions, LA_LOJBAN_PINNED_RAW_SHA256, LA_LOJBAN_PINNED_REVISION,
    LLG_BASE_LOCATOR, LLG_CMAVO_SHA256, LLG_GISMU_SHA256, LLG_NOTICE_SHA256, LLG_RAFSI_SHA256,
    LaLojbanImportError, LaLojbanImportOptions, LlgImportError, LlgImportOptions,
    OwnedContextInput, OwnedCurationDocument, PackId, PackPayloadKind, SUTYSISKU_PINNED_RAW_SHA256,
    SUTYSISKU_PINNED_REVISION, SutysiskuImportError, SutysiskuImportOptions, import_jbotci_export,
    import_la_lojban_export, import_llg_lists, import_sutysisku_export, load_owned_snapshot,
    seal_owned_snapshot, seal_release,
};
use vlajmaji_search::{
    CollectionCompactSearchEngine, CollectionQueryError, CollectionQueryErrorReport,
    CollectionSearchEngine, DictionaryCompactQueryResult, DictionaryCompactSearchEngine,
    DictionaryErrorReport, DictionaryExactOptions, DictionaryHitDetailRef, DictionaryRequest,
    DictionarySearchEngine, Query, SearchEngine, SearchOptions, render_collection_compact,
    render_collection_detail, render_dictionary_compact, render_dictionary_detail,
    validate_detail_snapshot_sha256,
};
use vlajmaji_store::{
    CollectionCatalogue, CollectionError, CollectionInput, CollectionPolicySelection,
    CollectionProfileEnablementRequest, CollectionProfileMembershipWriteRequest,
    CollectionProfiles, CollectionRequest, CollectionSelection, DictionaryStoreVerification,
    DictionaryStoreVerificationCheck, DictionaryStoreVerificationReport, OwnedJournal,
    OwnedJournalError, OwnedJournalReader,
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
        #[arg(long, value_enum, default_value_t = QueryFormat::Full)]
        format: QueryFormat,
    },
    /// Resolve a compact hit key into its complete standalone dictionary hit.
    DictionaryDetail {
        store: Option<PathBuf>,
        #[arg(long)]
        reference: String,
        #[arg(long)]
        expected_snapshot: Option<String>,
        #[arg(long, value_enum, default_value_t = DetailFormat::Full)]
        format: DetailFormat,
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
        after_help = "Register, install, list, and inspect use an explicit ROOT and never download data. Register records metadata only; install creates and verifies a new Unified generation. Manifest paths are relative to the manifest's parent; Synthetic inputs require --allow-synthetic. The configure, enable, disable, default, profiles, and selection subcommands use an explicit --config or resolve XDG_CONFIG_HOME/HOME configuration; they do not alter the global default store."
    )]
    Collections {
        #[command(subcommand)]
        command: CollectionsCommand,
    },
    /// Query the configured collection selection without changing existing STORE routes.
    #[command(
        after_help = "Reads only the explicit --config file or XDG_CONFIG_HOME/vlajmaji/config.toml, falling back to HOME/.config/vlajmaji/config.toml. This command never creates or downloads a config or store."
    )]
    Lookup {
        word: Option<String>,
        #[arg(long, conflicts_with = "word")]
        request: Option<String>,
        #[arg(long, requires = "word")]
        source: Vec<String>,
        #[arg(long, requires = "word")]
        page_size: Option<usize>,
        #[arg(long, requires = "word")]
        cursor: Option<String>,
        #[arg(long)]
        config: Option<PathBuf>,
        #[arg(long)]
        profile: Option<String>,
        #[arg(long)]
        pack: Vec<String>,
        #[arg(long, value_enum, default_value_t = QueryFormat::Full)]
        format: QueryFormat,
    },
    /// Resolve a compact hit key against the configured collection selection.
    LookupDetail {
        #[arg(long)]
        reference: String,
        #[arg(long)]
        expected_snapshot: Option<String>,
        #[arg(long)]
        config: Option<PathBuf>,
        #[arg(long)]
        profile: Option<String>,
        #[arg(long)]
        pack: Vec<String>,
        #[arg(long, value_enum, default_value_t = DetailFormat::Full)]
        format: DetailFormat,
    },
    /// Create, append to, or inspect one explicitly rooted authoritative dictionary journal.
    #[command(
        after_help = "Owned journals use only the explicit ROOT and never consult configuration or the default store. Init/edit read a complete versioned dictionary from --input FILE or stdin."
    )]
    Owned {
        #[command(subcommand)]
        command: OwnedCommand,
    },
}

#[derive(Debug, Subcommand)]
enum OwnedCommand {
    /// Create a new owned journal from a complete genesis dictionary.
    Init {
        root: PathBuf,
        #[arg(long)]
        input: Option<PathBuf>,
    },
    /// Append one complete dictionary revision at the expected persisted head.
    Edit {
        root: PathBuf,
        #[arg(long)]
        expected_head: String,
        #[arg(long)]
        input: Option<PathBuf>,
    },
    /// Read the checked current head summary.
    Head { root: PathBuf },
    /// Read the checked contiguous revision summaries.
    History { root: PathBuf },
    /// Reconstruct and validate the current or explicitly selected revision.
    Show {
        root: PathBuf,
        #[arg(long)]
        revision: Option<String>,
    },
    /// Explicitly migrate a schema-one journal to curation schema two.
    MigrateCuration {
        root: PathBuf,
        #[arg(long)]
        expected_head: String,
    },
    /// Append a strict complete curation document at the expected base head.
    Curate {
        root: PathBuf,
        #[arg(long)]
        expected_head: String,
        #[arg(long)]
        input: Option<PathBuf>,
    },
    /// Read and verify the complete curation document at the head or revision.
    CurationShow {
        root: PathBuf,
        #[arg(long)]
        revision: Option<String>,
    },
    /// Export and seal the complete history visible to a pinned reader.
    Export {
        root: PathBuf,
        #[arg(long)]
        output: PathBuf,
        #[arg(long)]
        seal: PathBuf,
    },
    /// Strictly validate a complete snapshot/seal pair without opening a journal root.
    Check {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        seal: PathBuf,
    },
    /// Import a complete validated history or recognize an exact replay.
    Import {
        root: PathBuf,
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        seal: PathBuf,
    },
    /// Extract one historical base Dictionary without opening a journal root.
    SnapshotModel {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        seal: PathBuf,
        #[arg(long)]
        revision: Option<String>,
        #[arg(long)]
        output: PathBuf,
    },
    /// Materialize an explicit curation view to a new output file.
    Materialize {
        root: PathBuf,
        #[arg(long)]
        revision: Option<String>,
        #[arg(long = "context")]
        contexts: Vec<PathBuf>,
        #[arg(long)]
        output: PathBuf,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Default)]
enum QueryFormat {
    #[default]
    Full,
    Compact,
    Readable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Default)]
enum DetailFormat {
    #[default]
    Full,
    Readable,
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
    /// Explicitly migrate a compatible version-1 catalogue to version 2.
    Migrate {
        root: PathBuf,
        #[arg(long)]
        expected_revision: u64,
    },
    /// Replace selected currently installed packs at an expected revision.
    #[command(
        after_help = "Uses the same --manifest or direct sealed inputs as install. Only selected currently installed packs are replaced; --expected-revision is checked before staging. This never changes a profile or the global default."
    )]
    Update {
        #[command(flatten)]
        args: CollectionMutationArgs,
        #[arg(long)]
        expected_revision: u64,
    },
    /// Rebuild the current derived generation from retained sealed inputs.
    Rebuild {
        root: PathBuf,
        #[arg(long)]
        expected_revision: u64,
    },
    /// Remove one or more installed or registered packs without deleting retained inputs.
    Remove {
        root: PathBuf,
        #[arg(long, required = true)]
        pack: Vec<String>,
        #[arg(long)]
        expected_revision: u64,
    },
    /// Restore one exact previously removed retained release.
    Reinstall {
        #[command(flatten)]
        args: CollectionMutationArgs,
        #[arg(long = "release-sha256")]
        registered_release_sha256: String,
        #[arg(long)]
        expected_revision: u64,
    },
    /// Observe SQLite recovery and retained attempts without logical activation.
    Reconcile { root: PathBuf },
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
    /// Write one explicit user selection profile.
    Configure {
        root: PathBuf,
        #[arg(long)]
        profile: String,
        #[arg(long, required_unless_present = "all", conflicts_with = "all")]
        pack: Vec<String>,
        #[arg(long, required_unless_present = "pack", conflicts_with = "pack")]
        all: bool,
        #[arg(long)]
        make_default: bool,
        #[arg(long)]
        config: Option<PathBuf>,
    },
    /// Change enabled state for all or explicitly selected profile members.
    Enable {
        #[arg(long)]
        profile: Option<String>,
        #[arg(long, required_unless_present = "all", conflicts_with = "all")]
        pack: Vec<String>,
        #[arg(long, required_unless_present = "pack", conflicts_with = "pack")]
        all: bool,
        #[arg(long)]
        config: Option<PathBuf>,
    },
    /// Disable all or explicitly selected profile members.
    Disable {
        #[arg(long)]
        profile: Option<String>,
        #[arg(long, required_unless_present = "all", conflicts_with = "all")]
        pack: Vec<String>,
        #[arg(long, required_unless_present = "pack", conflicts_with = "pack")]
        all: bool,
        #[arg(long)]
        config: Option<PathBuf>,
    },
    /// Set the default profile or explicitly clear it.
    Default {
        #[arg(long, required_unless_present = "clear", conflicts_with = "clear")]
        profile: Option<String>,
        #[arg(long, required_unless_present = "profile", conflicts_with = "profile")]
        clear: bool,
        #[arg(long)]
        config: Option<PathBuf>,
    },
    /// Read the explicit config file without opening collection roots.
    Profiles {
        #[arg(long)]
        config: Option<PathBuf>,
    },
    /// Capture one configured operational selection without loading a payload.
    Selection {
        #[arg(long)]
        config: Option<PathBuf>,
        #[arg(long)]
        profile: Option<String>,
        #[arg(long)]
        pack: Vec<String>,
    },
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
    Managed(Box<CollectionQueryErrorReport>),
}

fn structured_error(code: &str, message: impl Into<String>) -> CliError {
    CliError::Structured(DictionaryErrorReport {
        code: code.to_owned(),
        message: message.into(),
        sources: Vec::new(),
    })
}

fn owned_error(error: OwnedJournalError) -> CliError {
    let category = serde_json::to_value(error.category())
        .ok()
        .and_then(|value| value.as_str().map(str::to_owned))
        .unwrap_or_else(|| "backend".into());
    structured_error(&format!("owned_{category}"), error.to_string())
}

fn read_owned_input(input: Option<PathBuf>) -> Result<Vec<u8>, CliError> {
    match input {
        Some(path) => std::fs::read(&path).map_err(|error| {
            owned_error(OwnedJournalError::Io(io::Error::new(
                error.kind(),
                format!("read {}: {error}", path.display()),
            )))
        }),
        None => {
            let mut bytes = Vec::new();
            io::stdin()
                .read_to_end(&mut bytes)
                .map_err(|error| owned_error(OwnedJournalError::Io(error)))?;
            Ok(bytes)
        }
    }
}

fn parse_owned_dictionary(bytes: &[u8]) -> Result<Dictionary, CliError> {
    Dictionary::from_json_bytes(bytes)
        .map_err(|error| owned_error(OwnedJournalError::InvalidDictionary(error)))
}

fn parse_owned_id(value: &str) -> Result<EntityId, CliError> {
    EntityId::new(value)
        .map_err(|error| owned_error(OwnedJournalError::InvalidRequest(error.to_string())))
}

fn read_owned_file(path: &Path) -> Result<Vec<u8>, CliError> {
    std::fs::read(path).map_err(|error| {
        owned_error(OwnedJournalError::Io(io::Error::new(
            error.kind(),
            format!("read {}: {error}", path.display()),
        )))
    })
}

fn create_owned_file(path: &Path) -> Result<std::fs::File, CliError> {
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| {
            owned_error(OwnedJournalError::Io(io::Error::new(
                error.kind(),
                format!("create {}: {error}", path.display()),
            )))
        })
}

fn write_owned_handle(mut file: std::fs::File, path: &Path, bytes: &[u8]) -> Result<(), CliError> {
    file.write_all(bytes).map_err(|error| {
        owned_error(OwnedJournalError::Io(io::Error::new(
            error.kind(),
            format!("write {}: {error}", path.display()),
        )))
    })?;
    file.flush().map_err(|error| {
        owned_error(OwnedJournalError::Io(io::Error::new(
            error.kind(),
            format!("flush {}: {error}", path.display()),
        )))
    })?;
    file.sync_all().map_err(|error| {
        owned_error(OwnedJournalError::Io(io::Error::new(
            error.kind(),
            format!("sync {}: {error}", path.display()),
        )))
    })
}

fn write_owned_file(path: &Path, bytes: &[u8]) -> Result<(), CliError> {
    write_owned_handle(create_owned_file(path)?, path, bytes)
}

fn preflight_owned_outputs(paths: [&Path; 2]) -> Result<(), CliError> {
    for path in paths {
        match std::fs::symlink_metadata(path) {
            Ok(_) => {
                return Err(owned_error(OwnedJournalError::Io(io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    format!("output already exists: {}", path.display()),
                ))));
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(owned_error(OwnedJournalError::Io(io::Error::new(
                    error.kind(),
                    format!("inspect {}: {error}", path.display()),
                ))));
            }
        }
    }
    Ok(())
}

#[derive(Clone)]
struct ManagedOutputContext {
    operation: String,
    config_path: Option<vlajmaji_store::NativePathReport>,
    root_path: Option<vlajmaji_store::NativePathReport>,
    profile: Option<vlajmaji_store::CollectionProfileIdentity>,
    requested_profile: Option<String>,
    snapshot: Option<vlajmaji_search::CollectionSelectionSnapshot>,
    known_pack_ids: Vec<String>,
    outcome: String,
    collection_failure: Option<vlajmaji_store::CollectionFailureReport>,
}

impl ManagedOutputContext {
    fn report(&self, detail: String) -> serde_json::Value {
        if matches!(
            self.operation.as_str(),
            "dictionary_detail" | "dictionary_query"
        ) || self.operation.starts_with("owned_")
        {
            return serde_json::json!(DictionaryErrorReport {
                code: "response_output_io".into(),
                message: detail,
                sources: Vec::new(),
            });
        }
        if let Some(mut report) = self.collection_failure.clone() {
            report.category = "response_output_io".into();
            report.detail = format!("{detail}; {}", report.detail);
            return serde_json::json!(report);
        }
        serde_json::json!(CollectionQueryErrorReport {
            version: 1,
            ok: false,
            category: "response_output_io".into(),
            detail,
            operation: self.operation.clone(),
            config_path: self.config_path.clone(),
            root_path: self.root_path.clone(),
            profile: self.profile.clone(),
            requested_profile: self.requested_profile.clone(),
            snapshot: self.snapshot.clone(),
            known_pack_ids: self.known_pack_ids.clone(),
            outcome: self.outcome.clone(),
            writer: None,
        })
    }
}

fn dictionary_readable_output_context(store: &Path, operation: &str) -> ManagedOutputContext {
    ManagedOutputContext {
        operation: operation.into(),
        config_path: None,
        root_path: Some(vlajmaji_store::native_path_report(store)),
        profile: None,
        requested_profile: None,
        snapshot: None,
        known_pack_ids: Vec::new(),
        outcome: "not_applicable".into(),
        collection_failure: None,
    }
}

fn managed_output_context(command: &Command) -> Option<ManagedOutputContext> {
    let context = |operation: &str,
                   config: Option<&Path>,
                   root: Option<&Path>,
                   profile: Option<&str>,
                   packs: Vec<String>,
                   outcome: &str| ManagedOutputContext {
        operation: operation.into(),
        config_path: config.map(vlajmaji_store::native_path_report),
        root_path: root.map(vlajmaji_store::native_path_report),
        profile: None,
        requested_profile: profile.map(str::to_owned),
        snapshot: None,
        known_pack_ids: packs,
        outcome: outcome.into(),
        collection_failure: None,
    };
    match command {
        Command::Lookup { profile, pack, .. } => Some(context(
            "lookup",
            None,
            None,
            profile.as_deref(),
            pack.clone(),
            "not_applicable",
        )),
        Command::LookupDetail { profile, pack, .. } => Some(context(
            "lookup_detail",
            None,
            None,
            profile.as_deref(),
            pack.clone(),
            "not_applicable",
        )),
        Command::DictionaryDetail { store, .. } => Some(context(
            "dictionary_detail",
            None,
            store.as_deref(),
            None,
            Vec::new(),
            "not_applicable",
        )),
        Command::Owned { command } => {
            let (operation, root) = match command {
                OwnedCommand::Init { root, .. } => ("owned_init", Some(root.as_path())),
                OwnedCommand::Edit { root, .. } => ("owned_edit", Some(root.as_path())),
                OwnedCommand::Head { root } => ("owned_head", Some(root.as_path())),
                OwnedCommand::History { root } => ("owned_history", Some(root.as_path())),
                OwnedCommand::Show { root, .. } => ("owned_show", Some(root.as_path())),
                OwnedCommand::MigrateCuration { root, .. } => {
                    ("owned_migrate_curation", Some(root.as_path()))
                }
                OwnedCommand::Curate { root, .. } => ("owned_curate", Some(root.as_path())),
                OwnedCommand::CurationShow { root, .. } => {
                    ("owned_curation_show", Some(root.as_path()))
                }
                OwnedCommand::Export { root, .. } => ("owned_export", Some(root.as_path())),
                OwnedCommand::Check { .. } => ("owned_check", None),
                OwnedCommand::Import { root, .. } => ("owned_import", Some(root.as_path())),
                OwnedCommand::SnapshotModel { .. } => ("owned_snapshot_model", None),
                OwnedCommand::Materialize { root, .. } => {
                    ("owned_materialize", Some(root.as_path()))
                }
            };
            Some(context(
                operation,
                None,
                root,
                None,
                Vec::new(),
                "not_applicable",
            ))
        }
        Command::Collections { command } => match command {
            CollectionsCommand::Configure {
                root,
                profile,
                pack,
                all,
                config,
                ..
            } => {
                let packs = if *all {
                    [
                        PackId::LensiskuJbotci,
                        PackId::Llg,
                        PackId::LaLojban,
                        PackId::Sutysisku,
                    ]
                    .into_iter()
                    .map(|pack| pack.to_string())
                    .collect()
                } else {
                    pack.clone()
                };
                Some(context(
                    "configure",
                    resolve_config_path(config.as_deref()).ok().as_deref(),
                    Some(root),
                    Some(profile),
                    packs,
                    "committed",
                ))
            }
            CollectionsCommand::Profiles { config } => Some(context(
                "profiles",
                resolve_config_path(config.as_deref()).ok().as_deref(),
                None,
                None,
                Vec::new(),
                "not_applicable",
            )),
            CollectionsCommand::Enable {
                config,
                profile,
                pack,
                ..
            } => Some(context(
                "enable",
                resolve_config_path(config.as_deref()).ok().as_deref(),
                None,
                profile.as_deref(),
                pack.clone(),
                "unchanged",
            )),
            CollectionsCommand::Disable {
                config,
                profile,
                pack,
                ..
            } => Some(context(
                "disable",
                resolve_config_path(config.as_deref()).ok().as_deref(),
                None,
                profile.as_deref(),
                pack.clone(),
                "unchanged",
            )),
            CollectionsCommand::Default {
                config, profile, ..
            } => Some(context(
                if profile.is_some() {
                    "set_default"
                } else {
                    "clear_default"
                },
                resolve_config_path(config.as_deref()).ok().as_deref(),
                None,
                profile.as_deref(),
                Vec::new(),
                "unchanged",
            )),
            CollectionsCommand::Selection {
                config,
                profile,
                pack,
            } => Some(context(
                "selection",
                resolve_config_path(config.as_deref()).ok().as_deref(),
                None,
                profile.as_deref(),
                pack.clone(),
                "not_applicable",
            )),
            _ => None,
        },
        _ => None,
    }
}

fn mutation_output_context(
    operation: &str,
    root: &Path,
    report: &vlajmaji_store::CollectionMutationReport,
    prior_revision: u64,
) -> ManagedOutputContext {
    let generation = report.generation.as_ref();
    let committed = match operation {
        "update" => report.outcome == "updated",
        "rebuild" => report.outcome == "rebuilt",
        "remove" => report.outcome == "removed",
        "reinstall" => report.outcome == "reinstalled",
        _ => false,
    };
    let proposed_generation = if committed {
        match operation {
            "update" | "rebuild" => generation.map(|item| item.generation_id.clone()),
            "remove" | "reinstall" => generation.and_then(|item| {
                let stage_name = item.store_path.parent()?.file_name()?;
                let attempt = report.attempt_id.as_deref()?;
                (stage_name == std::ffi::OsStr::new(attempt)).then(|| item.generation_id.clone())
            }),
            _ => None,
        }
    } else {
        None
    };
    let activation = if committed {
        "committed"
    } else {
        "not_applicable"
    };
    let output = vlajmaji_store::CollectionFailureReport {
        version: 1,
        ok: false,
        operation: operation.into(),
        category: "response_output_io".into(),
        detail: format!(
            "{operation} completed at local_revision={}",
            report.local_revision
        ),
        path: Some(vlajmaji_store::native_path_report(root)),
        pack_id: None,
        attempt_id: report.attempt_id.clone(),
        prior_local_revision: Some(prior_revision),
        proposed_generation,
        observed_generation: generation.map(|item| item.generation_id.clone()),
        activation: Some(activation.into()),
        inputs: report.inputs.clone(),
    };
    lifecycle_output_context(operation, output)
}

fn reconciliation_output_context(
    report: &vlajmaji_store::CollectionReconciliationReport,
) -> ManagedOutputContext {
    let output = vlajmaji_store::CollectionFailureReport {
        version: 1,
        ok: false,
        operation: "reconcile".into(),
        category: "response_output_io".into(),
        detail: "reconcile completed without logical activation".into(),
        path: Some(report.root_path.clone()),
        pack_id: None,
        attempt_id: None,
        prior_local_revision: None,
        proposed_generation: None,
        observed_generation: report.active_generation.clone(),
        activation: Some("not_applicable".into()),
        inputs: Vec::new(),
    };
    lifecycle_output_context("reconcile", output)
}

fn migration_output_context(
    report: &vlajmaji_store::CollectionCatalogueMigrationReport,
) -> ManagedOutputContext {
    let output = vlajmaji_store::CollectionFailureReport {
        version: 1,
        ok: false,
        operation: "migrate".into(),
        category: "response_output_io".into(),
        detail: format!(
            "migrate completed at local_revision={}",
            report.local_revision
        ),
        path: Some(report.root_path.clone()),
        pack_id: None,
        attempt_id: None,
        prior_local_revision: Some(report.prior_local_revision),
        proposed_generation: None,
        observed_generation: report.active_generation.clone(),
        activation: Some(
            if report.outcome == "migrated" {
                "committed"
            } else {
                "not_applicable"
            }
            .into(),
        ),
        inputs: Vec::new(),
    };
    lifecycle_output_context("migrate", output)
}

fn lifecycle_output_context(
    operation: &str,
    collection_failure: vlajmaji_store::CollectionFailureReport,
) -> ManagedOutputContext {
    ManagedOutputContext {
        operation: operation.into(),
        config_path: None,
        root_path: collection_failure.path.clone(),
        profile: None,
        requested_profile: None,
        snapshot: None,
        known_pack_ids: Vec::new(),
        outcome: collection_failure
            .activation
            .clone()
            .unwrap_or_else(|| "not_applicable".into()),
        collection_failure: Some(collection_failure),
    }
}

fn lifecycle_serialization_error(context: &ManagedOutputContext, detail: String) -> CliError {
    let mut failure = context
        .collection_failure
        .clone()
        .expect("lifecycle output context retains its collection report");
    failure.category = "response_serialization".into();
    failure.detail = format!("{}: {detail}", failure.detail);
    CliError::CollectionFailure(Box::new(failure))
}

fn execute() -> Result<(QueryOutput, Option<ManagedOutputContext>), CliError> {
    let args = Args::parse();
    let mut output_context = managed_output_context(&args.command);
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
        Command::DictionaryQuery {
            store,
            request,
            format,
        } => {
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
            match format {
                QueryFormat::Full => {
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
                QueryFormat::Compact => {
                    let engine = DictionaryCompactSearchEngine::open(&selected.path)
                        .map_err(|error| CliError::Structured(error.report()))?;
                    let result = engine
                        .execute(request)
                        .map_err(|error| CliError::Structured(error.report()))?;
                    match result {
                        DictionaryCompactQueryResult::Exact(response) => {
                            serde_json::to_value(*response)
                        }
                        DictionaryCompactQueryResult::Resolve(response) => {
                            serde_json::to_value(*response)
                        }
                    }
                    .map_err(|error| structured_error("response_json", error.to_string()))?
                }
                QueryFormat::Readable => {
                    let engine = DictionaryCompactSearchEngine::open(&selected.path)
                        .map_err(|error| CliError::Structured(error.report()))?;
                    let result = engine
                        .execute(request)
                        .map_err(|error| CliError::Structured(error.report()))?;
                    output_context = Some(dictionary_readable_output_context(
                        &selected.path,
                        "dictionary_query",
                    ));
                    return Ok((
                        QueryOutput::Readable(render_dictionary_compact(&result)),
                        output_context,
                    ));
                }
            }
        }
        Command::DictionaryDetail {
            store,
            reference,
            expected_snapshot,
            format,
        } => {
            let reference = DictionaryHitDetailRef::from_json_bytes(reference.as_bytes())
                .map_err(|error| CliError::Structured(error.report()))?;
            if let Some(snapshot) = expected_snapshot.as_deref() {
                validate_detail_snapshot_sha256(snapshot)
                    .map_err(|error| CliError::Structured(error.report()))?;
            }
            let selected = resolve_store(store)?;
            let engine = DictionaryCompactSearchEngine::open(&selected.path)
                .map_err(|error| CliError::Structured(error.report()))?;
            let response = engine
                .detail(&reference, expected_snapshot.as_deref())
                .map_err(|error| CliError::Structured(error.report()))?;
            match format {
                DetailFormat::Full => serde_json::to_value(response)
                    .map_err(|error| structured_error("response_json", error.to_string()))?,
                DetailFormat::Readable => {
                    return Ok((
                        QueryOutput::Readable(render_dictionary_detail(&response)),
                        output_context,
                    ));
                }
            }
        }
        Command::Owned { command } => match command {
            OwnedCommand::Init { root, input } => {
                let bytes = read_owned_input(input)?;
                let genesis = parse_owned_dictionary(&bytes)?;
                let (_, report) = OwnedJournal::create(&root, &genesis).map_err(owned_error)?;
                serde_json::to_value(report)
                    .map_err(|error| structured_error("owned_response", error.to_string()))?
            }
            OwnedCommand::Edit {
                root,
                expected_head,
                input,
            } => {
                let expected_head = parse_owned_id(&expected_head)?;
                let bytes = read_owned_input(input)?;
                let candidate = parse_owned_dictionary(&bytes)?;
                let mut journal = OwnedJournal::open(&root).map_err(owned_error)?;
                let report = journal
                    .commit(&expected_head, &candidate)
                    .map_err(owned_error)?;
                serde_json::to_value(report)
                    .map_err(|error| structured_error("owned_response", error.to_string()))?
            }
            OwnedCommand::Head { root } => {
                let reader = OwnedJournalReader::open(&root).map_err(owned_error)?;
                let report = reader.head().map_err(owned_error)?;
                serde_json::to_value(report)
                    .map_err(|error| structured_error("owned_response", error.to_string()))?
            }
            OwnedCommand::History { root } => {
                let reader = OwnedJournalReader::open(&root).map_err(owned_error)?;
                let report = reader.history().map_err(owned_error)?;
                serde_json::to_value(report)
                    .map_err(|error| structured_error("owned_response", error.to_string()))?
            }
            OwnedCommand::Show { root, revision } => {
                let revision = revision.as_deref().map(parse_owned_id).transpose()?;
                let reader = OwnedJournalReader::open(&root).map_err(owned_error)?;
                let report = match revision {
                    Some(revision) => reader.show(&revision),
                    None => reader.show_head(),
                }
                .map_err(owned_error)?;
                serde_json::to_value(report)
                    .map_err(|error| structured_error("owned_response", error.to_string()))?
            }
            OwnedCommand::MigrateCuration {
                root,
                expected_head,
            } => {
                let expected_head = parse_owned_id(&expected_head)?;
                let mut journal = OwnedJournal::open(&root).map_err(owned_error)?;
                let report = journal
                    .migrate_curation(&expected_head)
                    .map_err(owned_error)?;
                serde_json::to_value(report)
                    .map_err(|error| structured_error("owned_response", error.to_string()))?
            }
            OwnedCommand::Curate {
                root,
                expected_head,
                input,
            } => {
                let expected_head = parse_owned_id(&expected_head)?;
                let bytes = read_owned_input(input)?;
                let candidate = OwnedCurationDocument::from_json_bytes(&bytes)
                    .map_err(|error| owned_error(OwnedJournalError::InvalidCuration(error)))?;
                let mut journal = OwnedJournal::open(&root).map_err(owned_error)?;
                let report = journal
                    .commit_curated(&expected_head, &candidate)
                    .map_err(owned_error)?;
                serde_json::to_value(report)
                    .map_err(|error| structured_error("owned_response", error.to_string()))?
            }
            OwnedCommand::CurationShow { root, revision } => {
                let revision = revision.as_deref().map(parse_owned_id).transpose()?;
                let reader = OwnedJournalReader::open(&root).map_err(owned_error)?;
                let document = match revision {
                    Some(revision) => reader.curation_show(&revision),
                    None => reader.curation_show_head(),
                }
                .map_err(owned_error)?;
                serde_json::to_value(document)
                    .map_err(|error| structured_error("owned_response", error.to_string()))?
            }
            OwnedCommand::Export { root, output, seal } => {
                let reader = OwnedJournalReader::open(&root).map_err(owned_error)?;
                let snapshot = reader.snapshot().map_err(owned_error)?;
                let sealed = seal_owned_snapshot(&snapshot)
                    .map_err(|error| owned_error(OwnedJournalError::InvalidSnapshot(error)))?;
                preflight_owned_outputs([output.as_path(), seal.as_path()])?;
                let output_file = create_owned_file(&output)?;
                let seal_file = create_owned_file(&seal)?;
                write_owned_handle(output_file, &output, &sealed.snapshot)?;
                write_owned_handle(seal_file, &seal, &sealed.seal)?;
                let data = snapshot.as_data();
                serde_json::json!({
                    "version": 1,
                    "source": data.source,
                    "head": data.head,
                    "journal_schema": data.journal_schema,
                    "revision_count": data.commits.len() as u64,
                    "output": vlajmaji_store::native_path_report(&output),
                    "output_bytes": sealed.snapshot.len() as u64,
                    "output_sha256": sha256(&sealed.snapshot),
                    "seal": vlajmaji_store::native_path_report(&seal),
                    "seal_bytes": sealed.seal.len() as u64,
                    "seal_sha256": sha256(&sealed.seal),
                })
            }
            OwnedCommand::Check { input, seal } => {
                let snapshot_bytes = read_owned_file(&input)?;
                let seal_bytes = read_owned_file(&seal)?;
                let snapshot = load_owned_snapshot(&snapshot_bytes, &seal_bytes)
                    .map_err(|error| owned_error(OwnedJournalError::InvalidSnapshot(error)))?;
                let data = snapshot.as_data();
                serde_json::json!({
                    "version": 1,
                    "source": data.source,
                    "head": data.head,
                    "journal_schema": data.journal_schema,
                    "revision_count": data.commits.len() as u64,
                    "snapshot_bytes": snapshot_bytes.len() as u64,
                    "snapshot_sha256": sha256(&snapshot_bytes),
                    "seal_bytes": seal_bytes.len() as u64,
                    "seal_sha256": sha256(&seal_bytes),
                })
            }
            OwnedCommand::Import { root, input, seal } => {
                let snapshot_bytes = read_owned_file(&input)?;
                let seal_bytes = read_owned_file(&seal)?;
                let snapshot = load_owned_snapshot(&snapshot_bytes, &seal_bytes)
                    .map_err(|error| owned_error(OwnedJournalError::InvalidSnapshot(error)))?;
                let (_, report) =
                    OwnedJournal::import_snapshot(&root, &snapshot).map_err(owned_error)?;
                serde_json::to_value(report)
                    .map_err(|error| structured_error("owned_response", error.to_string()))?
            }
            OwnedCommand::SnapshotModel {
                input,
                seal,
                revision,
                output,
            } => {
                let revision = revision.as_deref().map(parse_owned_id).transpose()?;
                let snapshot_bytes = read_owned_file(&input)?;
                let seal_bytes = read_owned_file(&seal)?;
                let snapshot = load_owned_snapshot(&snapshot_bytes, &seal_bytes)
                    .map_err(|error| owned_error(OwnedJournalError::InvalidSnapshot(error)))?;
                let revision = revision.unwrap_or_else(|| snapshot.as_data().head.clone());
                let dictionary = snapshot
                    .dictionary_at(&revision)
                    .map_err(|error| owned_error(OwnedJournalError::InvalidSnapshot(error)))?;
                let bytes = dictionary
                    .to_json_bytes()
                    .map_err(|error| owned_error(OwnedJournalError::InvalidDictionary(error)))?;
                write_owned_file(&output, &bytes)?;
                serde_json::json!({
                    "version": 1,
                    "source": snapshot.as_data().source,
                    "revision": revision,
                    "journal_schema": snapshot.as_data().journal_schema,
                    "output": vlajmaji_store::native_path_report(&output),
                    "output_bytes": bytes.len() as u64,
                    "output_sha256": sha256(&bytes),
                })
            }
            OwnedCommand::Materialize {
                root,
                revision,
                contexts,
                output,
            } => {
                let revision = revision.as_deref().map(parse_owned_id).transpose()?;
                let mut inputs = Vec::with_capacity(contexts.len());
                for path in contexts {
                    let bytes = read_owned_input(Some(path))?;
                    let input = OwnedContextInput::from_json_bytes(&bytes)
                        .map_err(|error| owned_error(OwnedJournalError::InvalidCuration(error)))?;
                    inputs.push(input);
                }
                let reader = OwnedJournalReader::open(&root).map_err(owned_error)?;
                let document = match revision {
                    Some(revision) => reader.curation_show(&revision),
                    None => reader.curation_show_head(),
                }
                .map_err(owned_error)?;
                let materialized = document
                    .materialize(inputs)
                    .map_err(|error| owned_error(OwnedJournalError::InvalidCuration(error)))?;
                let bytes = materialized
                    .dictionary
                    .to_json_bytes()
                    .map_err(|error| owned_error(OwnedJournalError::InvalidDictionary(error)))?;
                let mut file = std::fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&output)
                    .map_err(|error| {
                        owned_error(OwnedJournalError::Io(io::Error::new(
                            error.kind(),
                            format!("create {}: {error}", output.display()),
                        )))
                    })?;
                file.write_all(&bytes).map_err(|error| {
                    owned_error(OwnedJournalError::Io(io::Error::new(
                        error.kind(),
                        format!("write {}: {error}", output.display()),
                    )))
                })?;
                file.flush().map_err(|error| {
                    owned_error(OwnedJournalError::Io(io::Error::new(
                        error.kind(),
                        format!("flush {}: {error}", output.display()),
                    )))
                })?;
                serde_json::json!({
                    "version": 1,
                    "source": materialized.source,
                    "revision": materialized.revision,
                    "output": vlajmaji_store::native_path_report(&output),
                    "output_sha256": sha256(&bytes),
                    "resolutions": materialized.resolutions,
                })
            }
        },
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
        Command::Collections { command } => collections_command(command, &mut output_context)?,
        Command::Lookup {
            word,
            request,
            source,
            page_size,
            cursor,
            config,
            profile,
            pack,
            format,
        } => {
            // Decode caller input before touching configuration or catalogue paths so
            // malformed request data has deterministic precedence over unavailable stores.
            let preliminary_packs = pack
                .iter()
                .filter_map(|value| parse_collection_pack(value).ok())
                .collect::<Vec<_>>();
            let request = if let Some(word) = word {
                DictionaryRequest::Exact {
                    spelling: word,
                    options: DictionaryExactOptions {
                        sources: source,
                        page_size,
                        cursor,
                        ..Default::default()
                    },
                }
            } else {
                if !source.is_empty() || page_size.is_some() || cursor.is_some() {
                    return Err(managed_pre_config_error(
                        "invalid_request",
                        "direct source/page/cursor flags require WORD and cannot override request JSON or stdin",
                        "lookup",
                        profile.as_deref(),
                        &preliminary_packs,
                    ));
                }
                let text = match request {
                    Some(text) => text,
                    None => {
                        let mut text = String::new();
                        io::stdin().read_to_string(&mut text).map_err(|error| {
                            managed_pre_config_error(
                                "request_io",
                                format!("read dictionary request: {error}"),
                                "lookup",
                                profile.as_deref(),
                                &preliminary_packs,
                            )
                        })?;
                        text
                    }
                };
                serde_json::from_str::<DictionaryRequest>(&text).map_err(|error| {
                    managed_pre_config_error(
                        "request_invalid",
                        format!("decode dictionary request: {error}"),
                        "lookup",
                        profile.as_deref(),
                        &preliminary_packs,
                    )
                })?
            };
            let location_packs = pack
                .iter()
                .filter_map(|value| parse_collection_pack(value).ok())
                .collect::<Vec<_>>();
            let config_path = resolve_config_path(config.as_deref()).map_err(|error| {
                config_location_error(error, "lookup", None, profile.as_deref(), &location_packs)
            })?;
            let profiles = CollectionProfiles::read(config_path.clone()).map_err(|error| {
                with_managed_context(
                    managed_error_for(CollectionQueryError::Profile(error), "lookup"),
                    Some(&config_path),
                    None,
                    profile.as_deref(),
                    None,
                    &[],
                )
            })?;
            let configured_profile = profiles.profile(profile.as_deref()).map_err(|error| {
                with_managed_context(
                    managed_error_for(CollectionQueryError::Profile(error), "lookup"),
                    Some(&config_path),
                    None,
                    profile.as_deref(),
                    None,
                    &[],
                )
            })?;
            let profile_identity = profiles.identity(&configured_profile);
            let override_packs = if pack.is_empty() {
                None
            } else {
                let mut parsed = Vec::with_capacity(pack.len());
                for value in &pack {
                    match parse_collection_pack(value) {
                        Ok(pack) => parsed.push(pack),
                        Err(error) => {
                            return Err(with_managed_context(
                                managed_error_for(
                                    CollectionQueryError::InvalidSelection(error.to_string()),
                                    "lookup",
                                ),
                                Some(&config_path),
                                Some(&configured_profile.root),
                                profile.as_deref(),
                                Some(profile_identity.clone()),
                                &configured_profile.packs,
                            ));
                        }
                    }
                }
                Some(parsed)
            };
            if let Some(context) = output_context.as_mut() {
                context.config_path = Some(profile_identity.config_path.clone());
                context.root_path =
                    Some(vlajmaji_store::native_path_report(&configured_profile.root));
                context.profile = Some(profile_identity.clone());
                context.known_pack_ids = override_packs
                    .as_deref()
                    .unwrap_or(&configured_profile.packs)
                    .iter()
                    .map(ToString::to_string)
                    .collect();
            }
            let operation = match &request {
                DictionaryRequest::Exact { .. } => "exact",
                DictionaryRequest::Resolve { .. } => "resolve",
            };
            if let Some(context) = output_context.as_mut() {
                context.operation = operation.into();
            }
            let (response_value, snapshot, collections) = match format {
                QueryFormat::Full => {
                    let mut engine = CollectionSearchEngine::open_configured(
                        &profiles,
                        profile.as_deref(),
                        override_packs.as_deref(),
                    )
                    .map_err(|error| {
                        let known_pack_ids = override_packs
                            .as_deref()
                            .unwrap_or(&configured_profile.packs);
                        with_managed_context(
                            managed_error_for(error, "lookup"),
                            Some(&config_path),
                            Some(&configured_profile.root),
                            profile.as_deref(),
                            Some(profile_identity.clone()),
                            known_pack_ids,
                        )
                    })?;
                    let response = engine.execute(request.clone()).map_err(|error| {
                        let mut report = error.report_for(operation);
                        report.snapshot = Some(engine.selection().clone());
                        report.profile = engine.selection().profile.clone();
                        report.config_path = Some(profile_identity.config_path.clone());
                        report.root_path =
                            Some(vlajmaji_store::native_path_report(&configured_profile.root));
                        report.known_pack_ids = engine
                            .selection()
                            .collections
                            .iter()
                            .map(|row| row.pack_id.clone())
                            .collect();
                        CliError::Managed(Box::new(report))
                    })?;
                    let value = serde_json::to_value(&response).map_err(|error| {
                        managed_cli_error("response_serialization", error.to_string(), operation)
                    })?;
                    (value, response.snapshot, response.collections)
                }
                QueryFormat::Compact => {
                    let mut engine = CollectionCompactSearchEngine::open_configured(
                        &profiles,
                        profile.as_deref(),
                        override_packs.as_deref(),
                    )
                    .map_err(|error| {
                        let known_pack_ids = override_packs
                            .as_deref()
                            .unwrap_or(&configured_profile.packs);
                        with_managed_context(
                            managed_error_for(error, "lookup"),
                            Some(&config_path),
                            Some(&configured_profile.root),
                            profile.as_deref(),
                            Some(profile_identity.clone()),
                            known_pack_ids,
                        )
                    })?;
                    let response = engine.execute(request).map_err(|error| {
                        let mut report = error.report_for(operation);
                        report.snapshot = Some(engine.selection().clone());
                        report.profile = engine.selection().profile.clone();
                        report.config_path = Some(profile_identity.config_path.clone());
                        report.root_path =
                            Some(vlajmaji_store::native_path_report(&configured_profile.root));
                        report.known_pack_ids = engine
                            .selection()
                            .collections
                            .iter()
                            .map(|row| row.pack_id.clone())
                            .collect();
                        CliError::Managed(Box::new(report))
                    })?;
                    let value = serde_json::to_value(&response).map_err(|error| {
                        managed_cli_error("response_serialization", error.to_string(), operation)
                    })?;
                    (value, response.snapshot, response.collections)
                }
                QueryFormat::Readable => {
                    let mut engine = CollectionCompactSearchEngine::open_configured(
                        &profiles,
                        profile.as_deref(),
                        override_packs.as_deref(),
                    )
                    .map_err(|error| {
                        let known_pack_ids = override_packs
                            .as_deref()
                            .unwrap_or(&configured_profile.packs);
                        with_managed_context(
                            managed_error_for(error, "lookup"),
                            Some(&config_path),
                            Some(&configured_profile.root),
                            profile.as_deref(),
                            Some(profile_identity.clone()),
                            known_pack_ids,
                        )
                    })?;
                    let response = engine.execute(request).map_err(|error| {
                        let mut report = error.report_for(operation);
                        report.snapshot = Some(engine.selection().clone());
                        report.profile = engine.selection().profile.clone();
                        report.config_path = Some(profile_identity.config_path.clone());
                        report.root_path =
                            Some(vlajmaji_store::native_path_report(&configured_profile.root));
                        report.known_pack_ids = engine
                            .selection()
                            .collections
                            .iter()
                            .map(|row| row.pack_id.clone())
                            .collect();
                        CliError::Managed(Box::new(report))
                    })?;
                    if let Some(context) = output_context.as_mut() {
                        context.snapshot = Some(response.snapshot.clone());
                        context.known_pack_ids = response
                            .collections
                            .iter()
                            .map(|row| row.pack_id.clone())
                            .collect();
                    }
                    return Ok((
                        QueryOutput::Readable(render_collection_compact(&response)),
                        output_context,
                    ));
                }
            };
            if let Some(context) = output_context.as_mut() {
                context.snapshot = Some(snapshot);
                context.known_pack_ids =
                    collections.iter().map(|row| row.pack_id.clone()).collect();
            }
            response_value
        }
        Command::LookupDetail {
            reference,
            expected_snapshot,
            config,
            profile,
            pack,
            format,
        } => {
            let detail_ref = DictionaryHitDetailRef::from_json_bytes(reference.as_bytes())
                .map_err(|error| {
                    managed_pre_config_error(
                        "invalid_detail_reference",
                        error.to_string(),
                        "lookup_detail",
                        None,
                        &[],
                    )
                })?;
            if let Some(snapshot) = expected_snapshot.as_deref() {
                validate_detail_snapshot_sha256(snapshot).map_err(|error| {
                    managed_pre_config_error(
                        "malformed_snapshot",
                        error.to_string(),
                        "lookup_detail",
                        None,
                        &[],
                    )
                })?;
            }
            let location_packs = pack
                .iter()
                .filter_map(|value| parse_collection_pack(value).ok())
                .collect::<Vec<_>>();
            let config_path = resolve_config_path(config.as_deref()).map_err(|error| {
                config_location_error(
                    error,
                    "lookup_detail",
                    None,
                    profile.as_deref(),
                    &location_packs,
                )
            })?;
            let profiles = CollectionProfiles::read(config_path.clone()).map_err(|error| {
                with_managed_context(
                    managed_error_for(CollectionQueryError::Profile(error), "lookup_detail"),
                    Some(&config_path),
                    None,
                    profile.as_deref(),
                    None,
                    &[],
                )
            })?;
            let configured_profile = profiles.profile(profile.as_deref()).map_err(|error| {
                with_managed_context(
                    managed_error_for(CollectionQueryError::Profile(error), "lookup_detail"),
                    Some(&config_path),
                    None,
                    profile.as_deref(),
                    None,
                    &[],
                )
            })?;
            let profile_identity = profiles.identity(&configured_profile);
            let override_packs = if pack.is_empty() {
                None
            } else {
                let mut parsed = Vec::with_capacity(pack.len());
                for value in &pack {
                    parsed.push(parse_collection_pack(value).map_err(|error| {
                        with_managed_context(
                            managed_error_for(
                                CollectionQueryError::InvalidSelection(error.to_string()),
                                "lookup_detail",
                            ),
                            Some(&config_path),
                            Some(&configured_profile.root),
                            profile.as_deref(),
                            Some(profile_identity.clone()),
                            &configured_profile.packs,
                        )
                    })?);
                }
                Some(parsed)
            };
            let mut engine = CollectionCompactSearchEngine::open_configured(
                &profiles,
                profile.as_deref(),
                override_packs.as_deref(),
            )
            .map_err(|error| {
                with_managed_context(
                    managed_error_for(error, "lookup_detail"),
                    Some(&config_path),
                    Some(&configured_profile.root),
                    profile.as_deref(),
                    Some(profile_identity.clone()),
                    override_packs
                        .as_deref()
                        .unwrap_or(&configured_profile.packs),
                )
            })?;
            if let Some(context) = output_context.as_mut() {
                context.config_path = Some(profile_identity.config_path.clone());
                context.root_path =
                    Some(vlajmaji_store::native_path_report(&configured_profile.root));
                context.profile = Some(profile_identity.clone());
                context.known_pack_ids = override_packs
                    .as_deref()
                    .unwrap_or(&configured_profile.packs)
                    .iter()
                    .map(ToString::to_string)
                    .collect();
            }
            let response = engine
                .detail(&detail_ref, expected_snapshot.as_deref())
                .map_err(|error| {
                    let mut report = error.report_for("lookup_detail");
                    report.snapshot = Some(engine.selection().clone());
                    report.profile = engine.selection().profile.clone();
                    report.config_path = Some(profile_identity.config_path.clone());
                    report.root_path =
                        Some(vlajmaji_store::native_path_report(&configured_profile.root));
                    report.known_pack_ids = engine
                        .selection()
                        .collections
                        .iter()
                        .map(|row| row.pack_id.clone())
                        .collect();
                    CliError::Managed(Box::new(report))
                })?;
            if let Some(context) = output_context.as_mut() {
                context.snapshot = Some(response.snapshot.clone());
                context.known_pack_ids = response
                    .collections
                    .iter()
                    .map(|row| row.pack_id.clone())
                    .collect();
            }
            match format {
                DetailFormat::Full => serde_json::to_value(response).map_err(|error| {
                    managed_cli_error("response_serialization", error.to_string(), "lookup_detail")
                })?,
                DetailFormat::Readable => {
                    return Ok((
                        QueryOutput::Readable(render_collection_detail(&response)),
                        output_context,
                    ));
                }
            }
        }
    };
    Ok((QueryOutput::Json(output), output_context))
}

fn collections_command(
    command: CollectionsCommand,
    output_context: &mut Option<ManagedOutputContext>,
) -> Result<serde_json::Value, CliError> {
    match command {
        CollectionsCommand::Register(args) => {
            collection_mutation(args, "register", None, output_context)
        }
        CollectionsCommand::Install(args) => {
            collection_mutation(args, "install", None, output_context)
        }
        CollectionsCommand::Migrate {
            root,
            expected_revision,
        } => {
            let report = CollectionCatalogue::migrate(root, expected_revision)
                .map_err(|error| collection_cli_error(error, "migrate"))?;
            let context = migration_output_context(&report);
            *output_context = Some(context.clone());
            serde_json::to_value(report)
                .map_err(|error| lifecycle_serialization_error(&context, error.to_string()))
        }
        CollectionsCommand::Update {
            args,
            expected_revision,
        } => collection_mutation(args, "update", Some(expected_revision), output_context),
        CollectionsCommand::Rebuild {
            root,
            expected_revision,
        } => {
            let report = CollectionCatalogue::rebuild(&root, expected_revision)
                .map_err(|error| collection_cli_error(error, "rebuild"))?;
            let context = mutation_output_context("rebuild", &root, &report, expected_revision);
            *output_context = Some(context.clone());
            serde_json::to_value(report)
                .map_err(|error| lifecycle_serialization_error(&context, error.to_string()))
        }
        CollectionsCommand::Remove {
            root,
            pack,
            expected_revision,
        } => {
            let packs = pack
                .iter()
                .map(|pack| parse_collection_pack(pack))
                .collect::<Result<Vec<_>, _>>()
                .map_err(|error| collection_cli_error(error, "remove"))?;
            let report = CollectionCatalogue::remove(&root, &packs, expected_revision)
                .map_err(|error| collection_cli_error(error, "remove"))?;
            let context = mutation_output_context("remove", &root, &report, expected_revision);
            *output_context = Some(context.clone());
            serde_json::to_value(report)
                .map_err(|error| lifecycle_serialization_error(&context, error.to_string()))
        }
        CollectionsCommand::Reinstall {
            args,
            registered_release_sha256,
            expected_revision,
        } => {
            let request = collection_request(&args, "reinstall")?;
            let report = CollectionCatalogue::reinstall(
                &args.root,
                request,
                &registered_release_sha256,
                expected_revision,
            )
            .map_err(|error| collection_cli_error(error, "reinstall"))?;
            let context =
                mutation_output_context("reinstall", &args.root, &report, expected_revision);
            *output_context = Some(context.clone());
            serde_json::to_value(report)
                .map_err(|error| lifecycle_serialization_error(&context, error.to_string()))
        }
        CollectionsCommand::Reconcile { root } => {
            let report = CollectionCatalogue::reconcile(&root)
                .map_err(|error| collection_cli_error(error, "reconcile"))?;
            let context = reconciliation_output_context(&report);
            *output_context = Some(context.clone());
            serde_json::to_value(report)
                .map_err(|error| lifecycle_serialization_error(&context, error.to_string()))
        }
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
        CollectionsCommand::Configure {
            root,
            profile,
            pack,
            all,
            make_default,
            config,
        } => {
            let location_packs = if all {
                vec![
                    PackId::LensiskuJbotci,
                    PackId::Llg,
                    PackId::LaLojban,
                    PackId::Sutysisku,
                ]
            } else {
                pack.iter()
                    .filter_map(|value| parse_collection_pack(value).ok())
                    .collect()
            };
            let path = resolve_config_path(config.as_deref()).map_err(|error| {
                config_location_error(
                    error,
                    "configure",
                    Some(&root),
                    Some(&profile),
                    &location_packs,
                )
            })?;
            let packs = if all {
                vec![
                    PackId::LensiskuJbotci,
                    PackId::Llg,
                    PackId::LaLojban,
                    PackId::Sutysisku,
                ]
            } else {
                let mut parsed = Vec::with_capacity(pack.len());
                for value in &pack {
                    match parse_collection_pack(value) {
                        Ok(pack) => parsed.push(pack),
                        Err(error) => {
                            return Err(with_managed_context(
                                managed_error_for(
                                    CollectionQueryError::InvalidSelection(error.to_string()),
                                    "configure",
                                ),
                                Some(&path),
                                Some(&root),
                                Some(&profile),
                                None,
                                &parsed,
                            ));
                        }
                    }
                }
                parsed
            };
            let report = CollectionProfiles::configure_membership(
                path.clone(),
                CollectionProfileMembershipWriteRequest {
                    name: profile.clone(),
                    root: root.clone(),
                    packs: packs.clone(),
                    make_default,
                },
            )
            .map_err(|error| {
                let mapped = match error {
                    vlajmaji_store::CollectionProfileError::Catalogue { source } => {
                        managed_error_for(CollectionQueryError::Catalogue(*source), "configure")
                    }
                    other => managed_error_for(CollectionQueryError::Profile(other), "configure"),
                };
                match mapped {
                    CliError::Managed(mut report) => {
                        report.config_path = Some(vlajmaji_store::native_path_report(&path));
                        report.root_path = Some(vlajmaji_store::native_path_report(&root));
                        report.requested_profile = Some(profile.clone());
                        report.known_pack_ids = packs.iter().map(ToString::to_string).collect();
                        CliError::Managed(report)
                    }
                    other => other,
                }
            })?;
            if let Some(context) = output_context.as_mut() {
                context.config_path = Some(report.path.clone());
                context.root_path = Some(vlajmaji_store::native_path_report(&root));
                context.requested_profile = Some(profile.clone());
                context.known_pack_ids = packs.iter().map(ToString::to_string).collect();
                context.outcome = report.outcome.clone();
            }
            serde_json::to_value(report).map_err(|error| {
                managed_cli_error("response_serialization", error.to_string(), "configure")
            })
        }
        CollectionsCommand::Enable {
            config,
            profile,
            pack,
            all,
        } => profile_enablement_command(config, profile, pack, all, true, output_context),
        CollectionsCommand::Disable {
            config,
            profile,
            pack,
            all,
        } => profile_enablement_command(config, profile, pack, all, false, output_context),
        CollectionsCommand::Default {
            config,
            profile,
            clear: _,
        } => {
            let operation = if profile.is_some() {
                "set_default"
            } else {
                "clear_default"
            };
            let path = resolve_config_path(config.as_deref()).map_err(|error| {
                config_location_error(error, operation, None, profile.as_deref(), &[])
            })?;
            let report =
                CollectionProfiles::set_default(&path, profile.as_deref()).map_err(|error| {
                    with_managed_context(
                        managed_profile_error(error, operation),
                        Some(&path),
                        None,
                        profile.as_deref(),
                        None,
                        &[],
                    )
                })?;
            if let Some(context) = output_context.as_mut() {
                context.operation = operation.to_owned();
                context.config_path = Some(report.path.clone());
                context.root_path = report.root.clone();
                context.requested_profile = profile.clone();
                context.known_pack_ids = report.packs.iter().map(ToString::to_string).collect();
                context.outcome = report.outcome.clone();
            }
            serde_json::to_value(report).map_err(|error| {
                managed_cli_error("response_serialization", error.to_string(), operation)
            })
        }
        CollectionsCommand::Profiles { config } => {
            let path = resolve_config_path(config.as_deref())
                .map_err(|error| config_location_error(error, "profiles", None, None, &[]))?;
            let profiles = CollectionProfiles::read(path.clone()).map_err(|error| {
                with_managed_context(
                    managed_error_for(CollectionQueryError::Profile(error), "profiles"),
                    Some(&path),
                    None,
                    None,
                    None,
                    &[],
                )
            })?;
            serde_json::to_value(profiles.report()).map_err(|error| {
                managed_cli_error("response_serialization", error.to_string(), "profiles")
            })
        }
        CollectionsCommand::Selection {
            config,
            profile,
            pack,
        } => {
            let location_packs = pack
                .iter()
                .filter_map(|value| parse_collection_pack(value).ok())
                .collect::<Vec<_>>();
            let path = resolve_config_path(config.as_deref()).map_err(|error| {
                config_location_error(
                    error,
                    "selection",
                    None,
                    profile.as_deref(),
                    &location_packs,
                )
            })?;
            let profiles = CollectionProfiles::read(path.clone()).map_err(|error| {
                with_managed_context(
                    managed_error_for(CollectionQueryError::Profile(error), "selection"),
                    Some(&path),
                    None,
                    profile.as_deref(),
                    None,
                    &[],
                )
            })?;
            let configured_profile = profiles.profile(profile.as_deref()).map_err(|error| {
                with_managed_context(
                    managed_error_for(CollectionQueryError::Profile(error), "selection"),
                    Some(&path),
                    None,
                    profile.as_deref(),
                    None,
                    &[],
                )
            })?;
            let identity = profiles.identity(&configured_profile);
            let override_packs = if pack.is_empty() {
                None
            } else {
                let mut parsed = Vec::with_capacity(pack.len());
                for value in &pack {
                    match parse_collection_pack(value) {
                        Ok(pack) => parsed.push(pack),
                        Err(error) => {
                            return Err(with_managed_context(
                                managed_error_for(
                                    CollectionQueryError::InvalidSelection(error.to_string()),
                                    "selection",
                                ),
                                Some(&path),
                                Some(&configured_profile.root),
                                profile.as_deref(),
                                Some(identity.clone()),
                                &configured_profile.packs,
                            ));
                        }
                    }
                }
                Some(parsed)
            };
            let engine = CollectionSearchEngine::open_configured(
                &profiles,
                profile.as_deref(),
                override_packs.as_deref(),
            )
            .map_err(|error| {
                with_managed_context(
                    managed_error_for(error, "selection"),
                    Some(&path),
                    Some(&configured_profile.root),
                    profile.as_deref(),
                    Some(identity.clone()),
                    &configured_profile.packs,
                )
            })?;
            if let Some(context) = output_context.as_mut() {
                context.root_path =
                    Some(vlajmaji_store::native_path_report(&configured_profile.root));
                context.profile = Some(identity);
                context.known_pack_ids = override_packs
                    .as_deref()
                    .unwrap_or(&configured_profile.packs)
                    .iter()
                    .map(ToString::to_string)
                    .collect();
                context.snapshot = Some(engine.selection().clone());
            }
            serde_json::to_value(engine.selection()).map_err(|error| {
                managed_cli_error("response_serialization", error.to_string(), "selection")
            })
        }
    }
}

fn managed_error_for(error: CollectionQueryError, operation: &str) -> CliError {
    CliError::Managed(Box::new(error.report_for(operation)))
}

fn managed_profile_error(
    error: vlajmaji_store::CollectionProfileError,
    operation: &str,
) -> CliError {
    match error {
        vlajmaji_store::CollectionProfileError::Catalogue { source } => {
            managed_error_for(CollectionQueryError::Catalogue(*source), operation)
        }
        other => managed_error_for(CollectionQueryError::Profile(other), operation),
    }
}

fn profile_enablement_command(
    config: Option<PathBuf>,
    profile: Option<String>,
    pack: Vec<String>,
    all: bool,
    enabled: bool,
    output_context: &mut Option<ManagedOutputContext>,
) -> Result<serde_json::Value, CliError> {
    let operation = if enabled { "enable" } else { "disable" };
    let location_packs = pack
        .iter()
        .filter_map(|value| parse_collection_pack(value).ok())
        .collect::<Vec<_>>();
    let path = resolve_config_path(config.as_deref()).map_err(|error| {
        config_location_error(error, operation, None, profile.as_deref(), &location_packs)
    })?;
    let selection = if all {
        CollectionPolicySelection::All
    } else {
        let mut parsed = Vec::with_capacity(pack.len());
        for value in &pack {
            match parse_collection_pack(value) {
                Ok(value) => parsed.push(value),
                Err(error) => {
                    return Err(with_managed_context(
                        managed_error_for(
                            CollectionQueryError::InvalidSelection(error.to_string()),
                            operation,
                        ),
                        Some(&path),
                        None,
                        profile.as_deref(),
                        None,
                        &parsed,
                    ));
                }
            }
        }
        CollectionPolicySelection::Packs(parsed)
    };
    let report = CollectionProfiles::set_enabled(
        &path,
        CollectionProfileEnablementRequest {
            profile: profile.clone(),
            selection,
            enabled,
        },
    )
    .map_err(|error| {
        with_managed_context(
            managed_profile_error(error, operation),
            Some(&path),
            None,
            profile.as_deref(),
            None,
            &location_packs,
        )
    })?;
    if let Some(context) = output_context.as_mut() {
        context.operation = operation.to_owned();
        context.config_path = Some(report.path.clone());
        context.root_path = report.root.clone();
        context.requested_profile = report.profile.clone();
        context.known_pack_ids = report.packs.iter().map(ToString::to_string).collect();
        context.outcome = report.outcome.clone();
    }
    serde_json::to_value(report)
        .map_err(|error| managed_cli_error("response_serialization", error.to_string(), operation))
}

fn with_managed_context(
    error: CliError,
    config_path: Option<&Path>,
    root_path: Option<&Path>,
    requested_profile: Option<&str>,
    profile: Option<vlajmaji_store::CollectionProfileIdentity>,
    known_pack_ids: &[PackId],
) -> CliError {
    match error {
        CliError::Managed(mut report) => {
            report.config_path = config_path.map(vlajmaji_store::native_path_report);
            report.root_path = root_path.map(vlajmaji_store::native_path_report);
            report.requested_profile = requested_profile.map(str::to_owned);
            report.profile = profile;
            if !known_pack_ids.is_empty() {
                report.known_pack_ids = known_pack_ids.iter().map(ToString::to_string).collect();
            }
            CliError::Managed(report)
        }
        other => other,
    }
}

fn managed_cli_error(category: &str, detail: impl Into<String>, operation: &str) -> CliError {
    CliError::Managed(Box::new(CollectionQueryErrorReport {
        version: 1,
        ok: false,
        category: category.to_owned(),
        detail: detail.into(),
        operation: operation.to_owned(),
        config_path: None,
        root_path: None,
        profile: None,
        requested_profile: None,
        snapshot: None,
        known_pack_ids: Vec::new(),
        outcome: "not_applicable".into(),
        writer: None,
    }))
}

fn managed_pre_config_error(
    category: &str,
    detail: impl Into<String>,
    operation: &str,
    requested_profile: Option<&str>,
    known_pack_ids: &[PackId],
) -> CliError {
    with_managed_context(
        managed_cli_error(category, detail, operation),
        None,
        None,
        requested_profile,
        None,
        known_pack_ids,
    )
}

#[derive(Debug)]
enum ConfigLocationError {
    HomeUnavailable,
    HomeNotAbsolute(OsString),
}

impl ConfigLocationError {
    fn detail(&self) -> String {
        match self {
            Self::HomeUnavailable => {
                "HOME is unavailable and XDG_CONFIG_HOME is not an absolute nonempty path".into()
            }
            Self::HomeNotAbsolute(home) => format!(
                "HOME must be an absolute native path when XDG_CONFIG_HOME is unusable (got {home:?})"
            ),
        }
    }
}

fn config_location_error(
    error: ConfigLocationError,
    operation: &str,
    root: Option<&Path>,
    requested_profile: Option<&str>,
    known_pack_ids: &[PackId],
) -> CliError {
    let mut report = match managed_cli_error("config_path", error.detail(), operation) {
        CliError::Managed(report) => report,
        _ => unreachable!("managed_cli_error always returns its managed envelope"),
    };
    report.root_path = root.map(vlajmaji_store::native_path_report);
    report.requested_profile = requested_profile.map(str::to_owned);
    report.known_pack_ids = known_pack_ids.iter().map(ToString::to_string).collect();
    if operation == "configure" {
        report.outcome = "unchanged".into();
    }
    CliError::Managed(report)
}

fn resolve_config_path(explicit: Option<&Path>) -> Result<PathBuf, ConfigLocationError> {
    if let Some(path) = explicit {
        return Ok(path.to_path_buf());
    }
    if let Some(value) = std::env::var_os("XDG_CONFIG_HOME") {
        let path = PathBuf::from(value);
        if path.is_absolute() && !path.as_os_str().is_empty() {
            return Ok(path.join("vlajmaji/config.toml"));
        }
    }
    let home = std::env::var_os("HOME").ok_or(ConfigLocationError::HomeUnavailable)?;
    let home = PathBuf::from(home);
    if !home.is_absolute() {
        return Err(ConfigLocationError::HomeNotAbsolute(home.into_os_string()));
    }
    Ok(home.join(".config/vlajmaji/config.toml"))
}

fn collection_mutation(
    args: CollectionMutationArgs,
    operation: &str,
    expected_revision: Option<u64>,
    output_context: &mut Option<ManagedOutputContext>,
) -> Result<serde_json::Value, CliError> {
    let request = collection_request(&args, operation)?;
    let report = match operation {
        "register" => CollectionCatalogue::register(&args.root, request)
            .map_err(|error| collection_cli_error(error, operation))?,
        "install" => CollectionCatalogue::install(&args.root, request)
            .map_err(|error| collection_cli_error(error, operation))?,
        "update" => CollectionCatalogue::update(
            &args.root,
            request,
            expected_revision.expect("update CLI requires an expected revision"),
        )
        .map_err(|error| collection_cli_error(error, operation))?,
        _ => {
            return Err(structured_error(
                "collections_operation",
                "unsupported collection mutation",
            ));
        }
    };
    if operation == "update" {
        let context = mutation_output_context(
            "update",
            &args.root,
            &report,
            expected_revision.expect("update CLI requires an expected revision"),
        );
        *output_context = Some(context.clone());
        serde_json::to_value(report)
            .map_err(|error| lifecycle_serialization_error(&context, error.to_string()))
    } else {
        serde_json::to_value(report)
            .map_err(|error| structured_error("collections_json", error.to_string()))
    }
}

fn collection_request(
    args: &CollectionMutationArgs,
    operation: &str,
) -> Result<CollectionRequest<'static>, CliError> {
    if let Some(manifest) = args.manifest.as_ref() {
        if args.release.is_some() || args.seal.is_some() {
            return Err(collection_request_error(
                "manifest mode cannot be combined with release or seal",
                operation,
            ));
        }
        let selection = if args.all {
            if !args.pack.is_empty() {
                return Err(collection_request_error(
                    "--all cannot be combined with --pack",
                    operation,
                ));
            }
            CollectionSelection::All
        } else if args.pack.is_empty() {
            return Err(collection_request_error(
                "manifest mode requires --all or one or more --pack",
                operation,
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
        Ok(read_collection_manifest(
            manifest,
            selection,
            args.allow_synthetic,
            operation,
        )?)
    } else {
        if args.all || args.pack.len() != 1 {
            return Err(collection_request_error(
                "direct sealed mode requires exactly one --pack and does not accept --all",
                operation,
            ));
        }
        let pack = parse_collection_pack(&args.pack[0])
            .map_err(|error| collection_cli_error(error, operation))?;
        let (release, seal) = match (args.release.clone(), args.seal.clone()) {
            (Some(release), Some(seal)) => (release, seal),
            _ => {
                return Err(collection_request_error(
                    "direct sealed mode requires both --release and --seal",
                    operation,
                ));
            }
        };
        Ok(CollectionRequest::new(
            vec![CollectionInput::SealedFiles {
                pack_id: pack,
                release,
                seal,
            }],
            CollectionSelection::Packs(vec![pack]),
            args.allow_synthetic,
        )
        .map_err(|error| collection_cli_error(error, operation))?)
    }
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

fn collection_request_error(detail: impl Into<String>, operation: &str) -> CliError {
    collection_cli_error(CollectionError::InvalidRequest(detail.into()), operation)
}

fn read_collection_manifest(
    path: &Path,
    selection: CollectionSelection,
    allow_synthetic: bool,
    operation: &str,
) -> Result<CollectionRequest<'static>, CliError> {
    let bytes = std::fs::read(path).map_err(|source| {
        collection_cli_error(
            CollectionError::InputIo {
                path: path.to_path_buf(),
                source,
            },
            operation,
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
    let (output, output_context) = match execute() {
        Ok(result) => result,
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
        Err(CliError::Managed(report)) => {
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
    let mut stderr = io::stderr().lock();
    if emit_success_response(&output, &mut stdout, &mut stderr, output_context.as_ref()) {
        std::process::ExitCode::SUCCESS
    } else {
        std::process::ExitCode::FAILURE
    }
}

enum ResponseWriteFailure {
    BrokenPipe,
    Other(String),
}

fn emit_success_response<W: Write, E: Write>(
    output: &QueryOutput,
    stdout: &mut W,
    stderr: &mut E,
    context: Option<&ManagedOutputContext>,
) -> bool {
    let write_result = match output {
        QueryOutput::Json(value) => serde_json::to_writer(&mut *stdout, value)
            .map_err(|error| match error.io_error_kind() {
                Some(io::ErrorKind::BrokenPipe) => ResponseWriteFailure::BrokenPipe,
                _ => ResponseWriteFailure::Other(error.to_string()),
            })
            .and_then(|()| write_response_bytes(stdout, b"\n")),
        QueryOutput::Readable(text) => write_response_bytes(stdout, text.as_bytes()),
    }
    .and_then(|()| stdout.flush().map_err(response_write_failure));
    match write_result {
        Ok(()) => true,
        Err(ResponseWriteFailure::BrokenPipe) => true,
        Err(ResponseWriteFailure::Other(detail)) => {
            write_output_error(stderr, context, detail);
            false
        }
    }
}

fn write_response_bytes<W: Write>(
    stdout: &mut W,
    bytes: &[u8],
) -> Result<(), ResponseWriteFailure> {
    stdout.write_all(bytes).map_err(response_write_failure)
}

fn response_write_failure(error: io::Error) -> ResponseWriteFailure {
    if error.kind() == io::ErrorKind::BrokenPipe {
        ResponseWriteFailure::BrokenPipe
    } else {
        ResponseWriteFailure::Other(error.to_string())
    }
}

fn write_output_error<E: Write>(
    stderr: &mut E,
    context: Option<&ManagedOutputContext>,
    detail: String,
) {
    if let Some(context) = context {
        let envelope = serde_json::json!({"error": context.report(detail)});
        if serde_json::to_writer(&mut *stderr, &envelope).is_ok() {
            let _ = stderr.write_all(b"\n").and_then(|()| stderr.flush());
        }
    } else {
        let _ = writeln!(stderr, "write JSON: {detail}");
    }
}

#[cfg(test)]
mod managed_output_tests {
    use super::{
        ManagedOutputContext, QueryOutput, emit_success_response, lifecycle_output_context,
        mutation_output_context,
    };
    use std::{
        borrow::Cow,
        fs,
        io::{self, Write},
        time::{SystemTime, UNIX_EPOCH},
    };
    use vlajmaji_lexicon::{Dictionary, PackId, PackPayloadKind, seal_release};
    use vlajmaji_store::{
        CollectionCatalogue, CollectionInput, CollectionRequest, CollectionSelection,
    };

    struct BadDescriptorWriter;

    impl Write for BadDescriptorWriter {
        fn write(&mut self, _buffer: &[u8]) -> io::Result<usize> {
            Err(io::Error::from_raw_os_error(9))
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    struct FlushFailureWriter;

    impl Write for FlushFailureWriter {
        fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
            Ok(buffer.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Err(io::Error::from_raw_os_error(5))
        }
    }

    struct BrokenPipeWriter;

    impl Write for BrokenPipeWriter {
        fn write(&mut self, _buffer: &[u8]) -> io::Result<usize> {
            Err(io::Error::from(io::ErrorKind::BrokenPipe))
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[derive(Default)]
    struct FlushBrokenPipeAfterWrite {
        bytes: Vec<u8>,
        flushes: usize,
    }

    impl Write for FlushBrokenPipeAfterWrite {
        fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
            self.bytes.extend_from_slice(buffer);
            Ok(buffer.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            self.flushes += 1;
            Err(io::Error::from(io::ErrorKind::BrokenPipe))
        }
    }

    #[test]
    fn test_shared_json_and_readable_writer_handles_flush_errors_and_broken_pipe() {
        let context = ManagedOutputContext {
            operation: "lookup".into(),
            config_path: None,
            root_path: None,
            profile: None,
            requested_profile: Some("research".into()),
            snapshot: None,
            known_pack_ids: vec!["llg".into()],
            outcome: "not_applicable".into(),
            collection_failure: None,
        };

        for output in [
            QueryOutput::Json(serde_json::json!({"ok":true})),
            QueryOutput::Readable("readable body".into()),
        ] {
            let mut stderr = Vec::new();
            assert!(!emit_success_response(
                &output,
                &mut FlushFailureWriter,
                &mut stderr,
                Some(&context),
            ));
            let envelope: serde_json::Value = serde_json::from_slice(&stderr).unwrap();
            assert_eq!(envelope["error"]["category"], "response_output_io");
            assert_eq!(envelope["error"]["operation"], "lookup");
            assert_eq!(envelope["error"]["outcome"], "not_applicable");
            assert_eq!(
                envelope["error"]["known_pack_ids"],
                serde_json::json!(["llg"])
            );
        }

        for output in [
            QueryOutput::Json(serde_json::json!({"ok":true})),
            QueryOutput::Readable("readable body".into()),
        ] {
            let mut stderr = Vec::new();
            assert!(emit_success_response(
                &output,
                &mut BrokenPipeWriter,
                &mut stderr,
                Some(&context),
            ));
            assert!(stderr.is_empty());
        }

        let mut json_writer = FlushBrokenPipeAfterWrite::default();
        let mut stderr = Vec::new();
        assert!(emit_success_response(
            &QueryOutput::Json(serde_json::json!({"ok":true})),
            &mut json_writer,
            &mut stderr,
            Some(&context),
        ));
        assert_eq!(json_writer.bytes, b"{\"ok\":true}\n");
        assert_eq!(json_writer.flushes, 1);
        assert!(stderr.is_empty());

        let mut readable_writer = FlushBrokenPipeAfterWrite::default();
        let mut stderr = Vec::new();
        assert!(emit_success_response(
            &QueryOutput::Readable("readable body".into()),
            &mut readable_writer,
            &mut stderr,
            Some(&context),
        ));
        assert_eq!(readable_writer.bytes, b"readable body");
        assert_eq!(readable_writer.flushes, 1);
        assert!(stderr.is_empty());
    }

    #[test]
    fn test_emit_managed_response_io_failure_reports_committed_context() {
        let context = ManagedOutputContext {
            operation: "configure".into(),
            config_path: None,
            root_path: None,
            profile: None,
            requested_profile: Some("research".into()),
            snapshot: None,
            known_pack_ids: vec!["llg".into()],
            outcome: "committed".into(),
            collection_failure: None,
        };
        let mut stderr = Vec::new();
        let ok = emit_success_response(
            &QueryOutput::Json(serde_json::json!({"ok":true})),
            &mut BadDescriptorWriter,
            &mut stderr,
            Some(&context),
        );
        assert!(!ok);
        let envelope: serde_json::Value = serde_json::from_slice(&stderr).unwrap();
        assert_eq!(envelope["error"]["version"], 1);
        assert_eq!(envelope["error"]["ok"], false);
        assert_eq!(envelope["error"]["category"], "response_output_io");
        assert_eq!(envelope["error"]["operation"], "configure");
        assert_eq!(envelope["error"]["outcome"], "committed");
        assert_eq!(envelope["error"]["requested_profile"], "research");
        assert_eq!(
            envelope["error"]["known_pack_ids"],
            serde_json::json!(["llg"])
        );
        assert!(
            envelope["error"]["detail"]
                .as_str()
                .unwrap()
                .contains("Bad file descriptor")
        );

        let read_context = ManagedOutputContext {
            operation: "lookup".into(),
            config_path: None,
            root_path: None,
            profile: None,
            requested_profile: Some("research".into()),
            snapshot: None,
            known_pack_ids: vec!["llg".into()],
            outcome: "not_applicable".into(),
            collection_failure: None,
        };
        let mut read_stderr = Vec::new();
        assert!(!emit_success_response(
            &QueryOutput::Json(serde_json::json!({"ok":true})),
            &mut BadDescriptorWriter,
            &mut read_stderr,
            Some(&read_context),
        ));
        let read_envelope: serde_json::Value = serde_json::from_slice(&read_stderr).unwrap();
        assert_eq!(read_envelope["error"]["operation"], "lookup");
        assert_eq!(read_envelope["error"]["outcome"], "not_applicable");
    }

    #[test]
    fn test_emit_readable_uses_shared_writer_without_adding_a_newline() {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        assert!(emit_success_response(
            &QueryOutput::Readable("readable body\n".into()),
            &mut stdout,
            &mut stderr,
            None,
        ));
        assert_eq!(stdout, b"readable body\n");
        assert!(stderr.is_empty());

        let context = ManagedOutputContext {
            operation: "lookup".into(),
            config_path: None,
            root_path: None,
            profile: None,
            requested_profile: Some("readable".into()),
            snapshot: None,
            known_pack_ids: vec!["llg".into()],
            outcome: "not_applicable".into(),
            collection_failure: None,
        };
        let mut stderr = Vec::new();
        assert!(!emit_success_response(
            &QueryOutput::Readable("readable body\n".into()),
            &mut BadDescriptorWriter,
            &mut stderr,
            Some(&context),
        ));
        let envelope: serde_json::Value = serde_json::from_slice(&stderr).unwrap();
        assert_eq!(envelope["error"]["category"], "response_output_io");
        assert_eq!(envelope["error"]["operation"], "lookup");
        assert_eq!(envelope["error"]["outcome"], "not_applicable");
    }

    #[test]
    fn test_emit_dictionary_detail_io_failure_uses_structured_error_report() {
        let context = ManagedOutputContext {
            operation: "dictionary_detail".into(),
            config_path: None,
            root_path: None,
            profile: None,
            requested_profile: None,
            snapshot: None,
            known_pack_ids: Vec::new(),
            outcome: "not_applicable".into(),
            collection_failure: None,
        };
        let mut stderr = Vec::new();
        assert!(!emit_success_response(
            &QueryOutput::Json(serde_json::json!({"version":1,"hit":{}})),
            &mut BadDescriptorWriter,
            &mut stderr,
            Some(&context),
        ));
        let envelope: serde_json::Value = serde_json::from_slice(&stderr).unwrap();
        let report = envelope["error"].as_object().unwrap();
        assert_eq!(report.len(), 3);
        assert_eq!(report["code"], "response_output_io");
        assert_eq!(report["sources"], serde_json::json!([]));
        assert!(
            report["message"]
                .as_str()
                .unwrap()
                .contains("Bad file descriptor")
        );
    }

    #[test]
    fn test_emit_collection_response_io_failure_retains_committed_lifecycle_context() {
        let context = lifecycle_output_context(
            "update",
            vlajmaji_store::CollectionFailureReport {
                version: 1,
                ok: false,
                operation: "update".into(),
                category: "response_output_io".into(),
                detail: "update completed at local_revision=8".into(),
                path: Some(vlajmaji_store::NativePathReport {
                    encoding: "unix-bytes".into(),
                    hex: "2f746d702f636174616c6f677565".into(),
                    utf8: Some("/tmp/catalogue".into()),
                }),
                pack_id: None,
                attempt_id: Some("attempt-8".into()),
                prior_local_revision: Some(7),
                proposed_generation: Some("generation-8".into()),
                observed_generation: Some("generation-8".into()),
                activation: Some("committed".into()),
                inputs: vec![vlajmaji_store::CollectionInputDisposition {
                    pack_id: "llg".into(),
                    disposition: "validated".into(),
                    payload_kind: Some("synthetic".into()),
                    release_sha256: Some("a".repeat(64)),
                    report: None,
                }],
            },
        );
        let mut stderr = Vec::new();
        assert!(!emit_success_response(
            &QueryOutput::Json(serde_json::json!({"operation":"update"})),
            &mut BadDescriptorWriter,
            &mut stderr,
            Some(&context),
        ));
        let envelope: serde_json::Value = serde_json::from_slice(&stderr).unwrap();
        assert_eq!(envelope["error"]["category"], "response_output_io");
        assert_eq!(envelope["error"]["operation"], "update");
        assert_eq!(envelope["error"]["activation"], "committed");
        assert_eq!(envelope["error"]["prior_local_revision"], 7);
        assert_eq!(envelope["error"]["proposed_generation"], "generation-8");
        assert_eq!(envelope["error"]["observed_generation"], "generation-8");
        assert_eq!(envelope["error"]["attempt_id"], "attempt-8");
        assert_eq!(envelope["error"]["inputs"][0]["pack_id"], "llg");
        assert!(
            envelope["error"]["detail"]
                .as_str()
                .unwrap()
                .contains("local_revision=8")
        );
    }

    #[test]
    fn test_mutation_output_context_distinguishes_published_kept_and_cleared_generations() {
        let root = std::path::Path::new("/tmp/catalogue");
        let generation = |id: &str, attempt: &str| vlajmaji_store::CollectionGeneration {
            generation_id: id.into(),
            content_id: "content".into(),
            canonical_sha256: "a".repeat(64),
            store_path: root.join("generations").join(attempt).join("store"),
            path: vlajmaji_store::NativePathReport {
                encoding: "unix-bytes".into(),
                hex: "2f746d702f636174616c6f677565".into(),
                utf8: Some("/tmp/catalogue".into()),
            },
            members: Vec::new(),
        };
        let report = |outcome: &str,
                      attempt: Option<&str>,
                      current: Option<vlajmaji_store::CollectionGeneration>| {
            vlajmaji_store::CollectionMutationReport {
                version: 1,
                ok: true,
                operation: "remove".into(),
                outcome: outcome.into(),
                local_revision: 4,
                policy_state: "unconfigured".into(),
                attempt_id: attempt.map(str::to_owned),
                generation: current,
                inputs: Vec::new(),
            }
        };

        let published = mutation_output_context(
            "remove",
            root,
            &report(
                "removed",
                Some("attempt-4"),
                Some(generation("generation-4", "attempt-4")),
            ),
            3,
        );
        let failure = published.collection_failure.unwrap();
        assert_eq!(failure.activation.as_deref(), Some("committed"));
        assert_eq!(failure.proposed_generation.as_deref(), Some("generation-4"));
        assert_eq!(failure.observed_generation.as_deref(), Some("generation-4"));

        let kept = mutation_output_context(
            "remove",
            root,
            &report(
                "removed",
                Some("attempt-4"),
                Some(generation("generation-3", "attempt-3")),
            ),
            3,
        );
        let failure = kept.collection_failure.unwrap();
        assert_eq!(failure.activation.as_deref(), Some("committed"));
        assert!(failure.proposed_generation.is_none());
        assert_eq!(failure.observed_generation.as_deref(), Some("generation-3"));

        let cleared = mutation_output_context(
            "remove",
            root,
            &report("removed", Some("attempt-4"), None),
            3,
        );
        let failure = cleared.collection_failure.unwrap();
        assert_eq!(failure.activation.as_deref(), Some("committed"));
        assert!(failure.proposed_generation.is_none());
        assert!(failure.observed_generation.is_none());

        let idempotent = mutation_output_context(
            "reinstall",
            root,
            &report(
                "idempotent",
                None,
                Some(generation("generation-3", "attempt-3")),
            ),
            3,
        );
        let failure = idempotent.collection_failure.unwrap();
        assert_eq!(failure.activation.as_deref(), Some("not_applicable"));
        assert!(failure.proposed_generation.is_none());

        for (operation, outcome) in [("update", "updated"), ("rebuild", "rebuilt")] {
            let context = mutation_output_context(
                operation,
                root,
                &report(
                    outcome,
                    Some("attempt-5"),
                    Some(generation("generation-5", "attempt-5")),
                ),
                4,
            );
            let failure = context.collection_failure.as_ref().unwrap();
            assert_eq!(failure.operation, operation);
            assert_eq!(failure.activation.as_deref(), Some("committed"));
            assert_eq!(failure.proposed_generation.as_deref(), Some("generation-5"));
            assert_eq!(failure.observed_generation.as_deref(), Some("generation-5"));
            let mut stderr = Vec::new();
            assert!(!emit_success_response(
                &QueryOutput::Json(serde_json::json!({"operation": operation})),
                &mut BadDescriptorWriter,
                &mut stderr,
                Some(&context),
            ));
            let envelope: serde_json::Value = serde_json::from_slice(&stderr).unwrap();
            assert_eq!(envelope["error"]["operation"], operation);
            assert_eq!(envelope["error"]["activation"], "committed");
            assert_eq!(envelope["error"]["proposed_generation"], "generation-5");
        }
    }

    #[test]
    fn test_actual_remove_reports_keep_subset_and_clear_output_contexts() {
        let root = std::env::temp_dir().join(format!(
            "vlajmaji-s04-output-context-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fn request(pack: PackId) -> CollectionRequest<'static> {
            let dictionary = Dictionary::from_document(Default::default()).unwrap();
            let release = seal_release(pack, PackPayloadKind::Synthetic, &dictionary).unwrap();
            CollectionRequest::new(
                vec![CollectionInput::SealedBytes {
                    pack_id: pack,
                    release: Cow::Owned(release.release),
                    seal: Cow::Owned(release.seal),
                }],
                CollectionSelection::Packs(vec![pack]),
                true,
            )
            .unwrap()
        }
        fn assert_receipt(
            context: &ManagedOutputContext,
            activation: &str,
            proposed: Option<&str>,
            observed: Option<&str>,
        ) {
            let mut stderr = Vec::new();
            assert!(!emit_success_response(
                &QueryOutput::Json(serde_json::json!({"operation":"remove"})),
                &mut BadDescriptorWriter,
                &mut stderr,
                Some(context),
            ));
            let envelope: serde_json::Value = serde_json::from_slice(&stderr).unwrap();
            let error = &envelope["error"];
            assert_eq!(error["operation"], "remove");
            assert_eq!(error["activation"], activation);
            assert_eq!(error["proposed_generation"].as_str(), proposed);
            assert_eq!(error["observed_generation"].as_str(), observed);
        }

        let llg = request(PackId::Llg);
        let jb = request(PackId::LensiskuJbotci);
        let la = request(PackId::LaLojban);
        CollectionCatalogue::register(&root, llg.clone()).unwrap();
        CollectionCatalogue::register(&root, jb.clone()).unwrap();
        CollectionCatalogue::install(&root, llg).unwrap();
        let installed_both = CollectionCatalogue::install(&root, jb).unwrap();
        let original_generation = installed_both.generation.unwrap().generation_id;
        CollectionCatalogue::register(&root, la).unwrap();
        let before_keep = CollectionCatalogue::open(&root)
            .unwrap()
            .local_revision()
            .unwrap();

        let keep = CollectionCatalogue::remove(&root, &[PackId::LaLojban], before_keep).unwrap();
        let kept_generation = keep.generation.as_ref().unwrap().generation_id.clone();
        assert_eq!(kept_generation, original_generation);
        let context = mutation_output_context("remove", &root, &keep, before_keep);
        assert_receipt(&context, "committed", None, Some(kept_generation.as_str()));

        let subset =
            CollectionCatalogue::remove(&root, &[PackId::LensiskuJbotci], keep.local_revision)
                .unwrap();
        let subset_generation = subset.generation.as_ref().unwrap().generation_id.clone();
        assert_ne!(subset_generation, kept_generation);
        let context = mutation_output_context("remove", &root, &subset, keep.local_revision);
        assert_receipt(
            &context,
            "committed",
            Some(subset_generation.as_str()),
            Some(subset_generation.as_str()),
        );

        let clear =
            CollectionCatalogue::remove(&root, &[PackId::Llg], subset.local_revision).unwrap();
        assert!(clear.generation.is_none());
        let context = mutation_output_context("remove", &root, &clear, subset.local_revision);
        assert_receipt(&context, "committed", None, None);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn test_emit_migrated_response_io_failure_reports_migration_context() {
        let report = vlajmaji_store::CollectionCatalogueMigrationReport {
            version: 1,
            ok: true,
            operation: "migrate".into(),
            outcome: "migrated".into(),
            root_path: vlajmaji_store::NativePathReport {
                encoding: "unix-bytes".into(),
                hex: "2f746d702f636174616c6f677565".into(),
                utf8: Some("/tmp/catalogue".into()),
            },
            prior_schema_version: 1,
            schema_version: 2,
            prior_local_revision: 6,
            local_revision: 6,
            active_generation: Some("generation-6".into()),
        };
        let context = super::migration_output_context(&report);
        let mut stderr = Vec::new();
        assert!(!emit_success_response(
            &QueryOutput::Json(serde_json::json!({"operation":"migrate","outcome":"migrated"})),
            &mut BadDescriptorWriter,
            &mut stderr,
            Some(&context),
        ));
        let envelope: serde_json::Value = serde_json::from_slice(&stderr).unwrap();
        let error = &envelope["error"];
        assert_eq!(error["category"], "response_output_io");
        assert_eq!(error["operation"], "migrate");
        assert_eq!(error["activation"], "committed");
        assert_eq!(error["prior_local_revision"], 6);
        assert_eq!(error["observed_generation"], "generation-6");
        assert!(error["proposed_generation"].is_null());
        assert!(error["attempt_id"].is_null());
        assert_eq!(error["inputs"], serde_json::json!([]));
        assert!(
            error["detail"]
                .as_str()
                .unwrap()
                .contains("local_revision=6")
        );
    }
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
    use std::{
        fs,
        path::{Path, PathBuf},
        sync::atomic::{AtomicUsize, Ordering},
    };
    use vlajmaji_store::CollectionSelection;

    struct OwnedScratch(PathBuf);

    impl OwnedScratch {
        fn create(label: &str) -> Self {
            static NEXT_ID: AtomicUsize = AtomicUsize::new(0);
            for _ in 0..128 {
                let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
                let path = std::env::temp_dir().join(format!(
                    "vlajmaji-manifest-{label}-{}-{id}",
                    std::process::id()
                ));
                match fs::create_dir(&path) {
                    Ok(()) => return Self(path),
                    Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                    Err(error) => panic!("create exclusively owned manifest scratch: {error}"),
                }
            }
            panic!("could not allocate a fresh manifest scratch directory after 128 attempts");
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for OwnedScratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn test_manifest_duplicate_keys_rejects_escaped_equivalents_recursively() {
        let root = OwnedScratch::create("duplicate");
        let manifest = root.path().join("batch.json");
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
    }

    #[test]
    fn test_manifest_validation_freezes_relative_paths_without_reading_them() {
        let root = OwnedScratch::create("relative-path");
        let manifest = root.path().join("batch.json");
        std::fs::write(&manifest, br#"{"version":1,"inputs":[{"kind":"sealed","pack_id":"llg","release":"missing release.json","seal":"missing seal.json"}]}"#).unwrap();
        let _request = read_collection_manifest(
            &manifest,
            CollectionSelection::Packs(vec![vlajmaji_lexicon::PackId::Llg]),
            true,
            "register",
        )
        .unwrap();
        assert!(!root.path().join("missing release.json").exists());
    }
}
