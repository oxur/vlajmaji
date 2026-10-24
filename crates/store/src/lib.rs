//! Concrete local persistence for the accepted source-preserving lexical archive.
//!
//! The archive remains canonical. SQLite/FTS5 and optional disk Oxigraph/RocksDB
//! are rebuildable indexes; this crate does not normalize spelling or merge sources.

use std::{collections::BTreeSet, error::Error, fmt, path::Path};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use vlajmaji_lexicon::{FieldPresence, FormRole, Lexicon, RecordIdentity, SourceCrossReference};

mod sqlite;
pub use sqlite::SqliteStore;

mod dictionary;
mod dictionary_schema;
pub use dictionary::{DictionaryStore, DictionaryStoreError, DictionaryStoreInvalidity};

#[cfg(feature = "graph")]
mod graph;
#[cfg(feature = "graph")]
pub use graph::GraphStore;

/// A finite, backend-independent query accepted by both local stores.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum SearchQuery {
    /// Byte-exact captured headword lookup.
    ExactSpelling(String),
    /// Literal substring of an ASCII-lowercased source definition.
    DefinitionContains(String),
    /// Exact form string, optionally restricted to its source role.
    Form {
        /// Exact form bytes represented as a Rust string.
        value: String,
        /// Optional source assertion role.
        role: Option<FormRole>,
    },
    /// Captured cross-reference traversal from an exact headword.
    CrossReferences {
        /// Exact start spelling.
        spelling: String,
        /// Maximum traversal depth, one or two.
        depth: u8,
    },
    /// Query per-record field presence, optionally requiring exact retained value.
    Field {
        /// Exact top-level source field name.
        name: String,
        /// Required source presence state.
        state: PresenceState,
        /// Optional exact retained raw value.
        raw_value: Option<String>,
    },
    /// Exact proposal spelling, kept separate from captured records.
    ProposalExact(String),
}

/// Field-state selector for [`SearchQuery::Field`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum PresenceState {
    /// The source record has no field with this name.
    Missing,
    /// The JSON field exists and its value is JSON null.
    Null,
    /// The field exists and has a retained source value.
    Value,
}

/// One edge traversed by a cross-reference query.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct EdgeHit {
    /// Stable source-record ID of the edge origin.
    pub origin_id: String,
    /// Stable source-record ID of the target.
    pub target_id: Option<String>,
    /// Exact target spelling in the source note.
    pub target_spelling: String,
    /// Decoded-note UTF-8 start byte offset.
    pub byte_start: usize,
    /// Decoded-note UTF-8 exclusive end byte offset.
    pub byte_end: usize,
    /// Bounded parser rule identifier.
    pub rule_version: String,
}

/// Complete, comparable result from one native query execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchResult {
    /// Sorted distinct source record identities.
    pub record_ids: BTreeSet<String>,
    /// Distinct traversed source edges, populated only for link queries.
    pub edges: BTreeSet<EdgeHit>,
    /// Backend-reported candidate count before any exact postfilter.
    pub candidates: usize,
    /// Native execution path label.
    pub path: String,
    /// Visible reason for the native SQL scan fallback, if selected.
    pub fallback_reason: Option<String>,
}

/// Typed failure from opening, building, or querying a local native store.
#[derive(Debug)]
#[non_exhaustive]
pub enum StoreError {
    /// Caller-supplied path already exists; creation never replaces it.
    ExistingPath,
    /// Store metadata is absent, incomplete, has another schema, or another archive hash.
    InvalidStore(String),
    /// Accepted lexical archive failed validation.
    InvalidArchive(String),
    /// Backend operation failed with its original diagnostic.
    Backend(String),
    /// Filesystem operation failed.
    Io(std::io::Error),
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ExistingPath => write!(f, "store path already exists"),
            Self::InvalidStore(message) => write!(f, "invalid store: {message}"),
            Self::InvalidArchive(message) => write!(f, "invalid archive: {message}"),
            Self::Backend(message) => write!(f, "backend error: {message}"),
            Self::Io(error) => write!(f, "filesystem error: {error}"),
        }
    }
}

impl Error for StoreError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

/// Computes the lowercase hexadecimal SHA-256 of archive bytes.
#[must_use]
pub fn archive_sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

/// Opens and validates the canonical versioned archive before building an index.
pub fn validate_archive(bytes: &[u8]) -> Result<Lexicon, StoreError> {
    Lexicon::from_archive_bytes(bytes)
        .map_err(|error| StoreError::InvalidArchive(error.to_string()))
}

/// Computes the stable lowercase hexadecimal identifier used by this store for
/// one source record identity. The encoding is the byte-exact tuple
/// `(snapshot namespace, snapshot SHA-256, locator)`, with each UTF-8 component
/// prefixed by its big-endian `u64` byte length. It is not a content hash.
#[must_use]
pub fn record_id(identity: &RecordIdentity) -> String {
    let parts = [
        identity.snapshot().namespace().as_bytes(),
        identity.snapshot().sha256().as_bytes(),
        identity.locator().as_bytes(),
    ];
    let mut tuple = Vec::new();
    for part in parts {
        tuple.extend_from_slice(&(part.len() as u64).to_be_bytes());
        tuple.extend_from_slice(part);
    }
    let mut out = String::with_capacity(tuple.len() * 2);
    for byte in tuple {
        use fmt::Write as _;
        let _ = write!(out, "{byte:02x}");
    }
    out
}

pub(crate) fn edge_hits(reference: &SourceCrossReference) -> Vec<EdgeHit> {
    let targets: Vec<Option<String>> = if reference.targets().is_empty() {
        vec![None]
    } else {
        reference
            .targets()
            .iter()
            .map(|target| Some(record_id(target)))
            .collect()
    };
    targets
        .into_iter()
        .map(|target_id| EdgeHit {
            origin_id: record_id(reference.origin()),
            target_id,
            target_spelling: reference.target_spelling().to_owned(),
            byte_start: reference.byte_start(),
            byte_end: reference.byte_end(),
            rule_version: reference.rule_version().to_owned(),
        })
        .collect()
}

pub(crate) fn role_name(role: FormRole) -> &'static str {
    match role {
        FormRole::SourceReportedForm => "source-reported",
        FormRole::AssignedRafsi => "assigned-rafsi",
        _ => "other-source-role",
    }
}

/// Executes the separate in-memory full scan used as a correctness oracle.
///
/// This function is intended for controlled comparison and does not use a native index.
#[must_use]
pub fn scan_reference(lexicon: &Lexicon, query: &SearchQuery) -> SearchResult {
    let mut result = empty_result("independent-rust-scan", None);
    match query {
        SearchQuery::ExactSpelling(spelling) => {
            for record in lexicon.lookup_exact(spelling) {
                result.record_ids.insert(record_id(record.identity()));
            }
        }
        SearchQuery::DefinitionContains(term) => {
            let needle = term.to_ascii_lowercase();
            if needle.is_empty() {
                return result;
            }
            for record in lexicon.records() {
                if record
                    .definition()
                    .is_some_and(|definition| definition.to_ascii_lowercase().contains(&needle))
                {
                    result.record_ids.insert(record_id(record.identity()));
                }
            }
        }
        SearchQuery::Form { value, role } => {
            for form in lexicon.forms() {
                if form.form() == value && role.is_none_or(|expected| form.role() == expected) {
                    result.record_ids.insert(record_id(form.source_record()));
                }
            }
        }
        SearchQuery::CrossReferences { spelling, depth } => {
            if !(1..=2).contains(depth) {
                return result;
            }
            let mut frontier: BTreeSet<String> = lexicon
                .lookup_exact(spelling)
                .into_iter()
                .map(|record| record_id(record.identity()))
                .collect();
            for _ in 0..*depth {
                let mut next = BTreeSet::new();
                for reference in lexicon.cross_references() {
                    for hit in edge_hits(reference) {
                        if frontier.contains(&hit.origin_id) {
                            result.edges.insert(hit.clone());
                            if let Some(target_id) = hit.target_id {
                                next.insert(target_id.clone());
                                result.record_ids.insert(target_id);
                            }
                        }
                    }
                }
                frontier = next;
            }
        }
        SearchQuery::Field {
            name,
            state,
            raw_value,
        } => {
            for record in lexicon.records() {
                let presence = record.field(name);
                let matches = match (state, presence) {
                    (PresenceState::Missing, FieldPresence::Missing) => raw_value.is_none(),
                    (PresenceState::Null, FieldPresence::Null) => raw_value
                        .as_deref()
                        .is_none_or(|expected| expected == "null"),
                    (PresenceState::Value, FieldPresence::Value(raw)) => {
                        raw_value.as_deref().is_none_or(|expected| expected == raw)
                    }
                    _ => false,
                };
                if matches {
                    result.record_ids.insert(record_id(record.identity()));
                }
            }
        }
        SearchQuery::ProposalExact(spelling) => {
            for proposal in lexicon.lookup_proposals(spelling) {
                result.record_ids.insert(proposal.id().to_owned());
            }
        }
    }
    result.candidates = result.record_ids.len();
    result
}

pub(crate) fn empty_result(path: &str, fallback_reason: Option<String>) -> SearchResult {
    SearchResult {
        record_ids: BTreeSet::new(),
        edges: BTreeSet::new(),
        candidates: 0,
        path: path.to_owned(),
        fallback_reason,
    }
}

/// Native disk-store API shared by the concrete backends.
pub trait LexicalStore {
    /// Executes one query against the persistent native index.
    fn search(&self, query: &SearchQuery) -> Result<SearchResult, StoreError>;
    /// Returns exact canonical archive bytes retained in the index.
    fn archive_bytes(&self) -> Result<Vec<u8>, StoreError>;
    /// Flushes backend-managed data and closes are represented by dropping the store.
    fn flush(&self) -> Result<(), StoreError>;
    /// Returns logical bytes occupied by files below the store path after flush.
    fn logical_bytes(&self) -> Result<u64, StoreError>;
}

pub(crate) fn require_fresh(path: &Path) -> Result<(), StoreError> {
    if path.exists() {
        return Err(StoreError::ExistingPath);
    }
    Ok(())
}

pub(crate) fn directory_bytes(path: &Path) -> Result<u64, StoreError> {
    fn walk(path: &Path, total: &mut u64) -> Result<(), StoreError> {
        for entry in std::fs::read_dir(path).map_err(StoreError::Io)? {
            let entry = entry.map_err(StoreError::Io)?;
            let metadata = entry.metadata().map_err(StoreError::Io)?;
            if metadata.is_dir() {
                walk(&entry.path(), total)?;
            } else if metadata.is_file() {
                *total = total.saturating_add(metadata.len());
            }
        }
        Ok(())
    }
    let mut total = 0;
    walk(path, &mut total)?;
    Ok(total)
}

#[cfg(test)]
mod identity_tests {
    use super::*;
    use vlajmaji_lexicon::{SourceFormat, SourceMetadata, TimePrecision};

    #[test]
    fn tuple_identity_is_length_prefixed_and_byte_exact() {
        let mut lex = Lexicon::new();
        let src = SourceMetadata::new(
            "a\0b",
            "x",
            SourceFormat::LaLojbanJson,
            "m",
            "r",
            None,
            TimePrecision::Unknown,
        );
        lex.import_source(src, br#"[{"word":"c","definition":"d"}]"#, None)
            .unwrap();
        let left = record_id(lex.records()[0].identity());
        let mut other = Lexicon::new();
        let src = SourceMetadata::new(
            "a",
            "x",
            SourceFormat::LaLojbanJson,
            "m",
            "r",
            None,
            TimePrecision::Unknown,
        );
        other
            .import_source(src, br#"[{"word":"b\u0000c","definition":"d"}]"#, None)
            .unwrap();
        assert_ne!(left, record_id(other.records()[0].identity()));
    }
}
