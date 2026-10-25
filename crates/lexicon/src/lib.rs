//! A source-preserving envelope for captured lexical material.
//!
//! The crate stores original bytes and exposes exact source records without
//! merging definitions, languages, source IDs, or form roles.
//!
//! # Examples
//!
//! ```
//! use vlajmaji_lexicon::{Lexicon, SourceFormat, SourceMetadata, TimePrecision};
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let input = br#"[{"word":"farvi","definition":"first"},{"word":"farvi","definition":"second"}]"#;
//! let metadata = SourceMetadata::new(
//!     "fixture", "farvi.json", SourceFormat::LaLojbanJson,
//!     "https://example.invalid/farvi.json", "fixture-v1", None,
//!     TimePrecision::Unknown,
//! );
//! let mut lexicon = Lexicon::new();
//! lexicon.import_source(metadata, input, None)?;
//! assert_eq!(lexicon.lookup_exact("farvi").len(), 2);
//! # Ok(())
//! # }
//! ```
//!
//! [`Lexicon`] is the entry point. [`CapturedRecord`] provides a borrowed
//! source view, while [`SourceMetadata`] and [`RecordIdentity`] retain identity.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use serde::de::Deserializer as _;
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
use sha2::{Digest, Sha256};

mod attribution;
pub mod dictionary;
mod jbotci;
pub mod llg;
pub mod pack;
pub mod registry;

pub use attribution::AttributedOccurrence;
pub use dictionary::{
    ArgumentFrameRecord, ArgumentSlot, AssertionRecord, Availability, CaptureMetadata, CaptureRef,
    CatalogTarget, CodeNotice, CollectionRecord, ComponentRecord, ComputationRecord,
    ComputedAnnotation, ContentNotice, ContextKind, ContextRecord, Dictionary, DictionaryDocument,
    DictionaryError, DictionaryMetadata, Direction, EditorialSenseRecord, Eligibility, EntityId,
    EntryRecord, EtymologyRecord, ExtensionRecord, FormKind, FormRecord, InputRef, LocalRef,
    NamedTimestamp, NativeEncoding, NativeField, OccurrenceRef, OrderedComponent, OwnedHead,
    OwnedRevisionRecord, PosMapping, Presence, RawCapture, RawJson, RawOccurrence,
    RedistributionStatus, RelationKind, RelationRecord, RelationScope, RelationTarget,
    ReleaseAccounting, ReleaseDiagnostic, ReleaseRecord, SourceEvidence, TextRange, TextValue,
    UnverifiedDictionaryDocument,
};
pub use jbotci::{
    JBOTCI_PINNED_METADATA_SHA256, JBOTCI_PINNED_RAW_SHA256, JBOTCI_PINNED_REVISION, JbotciImport,
    JbotciImportError, JbotciImportOptions, JbotciImportReport, import_jbotci_export,
};
pub use llg::{
    LLG_BASE_LOCATOR, LLG_CMAVO_SHA256, LLG_GISMU_SHA256, LLG_NOTICE_SHA256, LLG_RAFSI_SHA256,
    LlgComponentReport, LlgHeaderClaim, LlgImport, LlgImportError, LlgImportOptions,
    LlgImportReport, import_llg_lists,
};
pub use pack::{
    PackError, PackId, PackPayloadKind, PackSeal, SealedRelease, compose_dictionaries,
    load_sealed_release, seal_release,
};
pub use registry::{
    AbbreviationKind, AliasScope, AliasStatus, ContributorId, ContributorRecord, DistributorId,
    DistributorRecord, LineageTarget, OccurrenceAttribution, RegistryDocument, RegistryError,
    SnapshotAttribution, SourceAlias, SourceId, SourceLineage, SourceOrigin, SourceRecord,
    SourceRegistry,
};

/// The captured representation received from one source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum SourceFormat {
    /// Array of La-Lojban JSON dictionary rows.
    LaLojbanJson,
    /// Lensisku JSON response array.
    LensiskuJson,
    /// LLG gismu fixed-column text list.
    LlgGismuFixed,
    /// LLG cmavo fixed-column text list.
    LlgCmavoFixed,
    /// LLG assigned rafsi whitespace-delimited list.
    LlgRafsiRows,
}

/// Precision available for a capture timestamp.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum TimePrecision {
    /// Capture timestamp has second precision.
    Second,
    /// Only the capture date is known.
    Day,
    /// The source time is unknown or was not supplied.
    Unknown,
}

/// Caller-supplied source and capture provenance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceMetadata {
    namespace: String,
    source_name: String,
    format: SourceFormat,
    locator: String,
    revision: String,
    captured_at: Option<String>,
    time_precision: TimePrecision,
    notice_locator: Option<String>,
}

impl SourceMetadata {
    /// Constructs source provenance without opening files or making requests.
    ///
    /// # Errors
    ///
    /// This constructor is infallible; empty values are retained as supplied.
    ///
    /// # Examples
    ///
    /// ```
    /// use vlajmaji_lexicon::{SourceFormat, SourceMetadata, TimePrecision};
    /// let source = SourceMetadata::new("la", "export", SourceFormat::LaLojbanJson,
    ///     "https://example.invalid/la.json", "sha-or-revision", None,
    ///     TimePrecision::Unknown);
    /// assert_eq!(source.namespace(), "la");
    /// ```
    #[must_use]
    pub fn new(
        namespace: impl Into<String>,
        source_name: impl Into<String>,
        format: SourceFormat,
        locator: impl Into<String>,
        revision: impl Into<String>,
        captured_at: Option<String>,
        time_precision: TimePrecision,
    ) -> Self {
        Self {
            namespace: namespace.into(),
            source_name: source_name.into(),
            format,
            locator: locator.into(),
            revision: revision.into(),
            captured_at,
            time_precision,
            notice_locator: None,
        }
    }

    /// Returns a copy of this metadata with a content/reuse notice locator.
    #[must_use]
    pub fn with_notice_locator(mut self, locator: impl Into<String>) -> Self {
        self.notice_locator = Some(locator.into());
        self
    }

    /// Returns the source namespace used in snapshot identity.
    #[must_use]
    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    /// Returns the captured source's display name.
    #[must_use]
    pub fn source_name(&self) -> &str {
        &self.source_name
    }

    /// Returns the source representation kind.
    #[must_use]
    pub const fn format(&self) -> SourceFormat {
        self.format
    }

    /// Returns the supplied source locator.
    #[must_use]
    pub fn locator(&self) -> &str {
        &self.locator
    }

    /// Returns the source revision or capture identity.
    #[must_use]
    pub fn revision(&self) -> &str {
        &self.revision
    }

    /// Returns the timestamp text when known.
    #[must_use]
    pub fn captured_at(&self) -> Option<&str> {
        self.captured_at.as_deref()
    }

    /// Returns the precision of the timestamp claim.
    #[must_use]
    pub const fn time_precision(&self) -> TimePrecision {
        self.time_precision
    }

    /// Returns the notice locator when supplied.
    #[must_use]
    pub fn notice_locator(&self) -> Option<&str> {
        self.notice_locator.as_deref()
    }
}

/// Stable identity of one immutable source snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SnapshotIdentity {
    namespace: String,
    sha256: String,
}

impl SnapshotIdentity {
    /// Returns the source namespace.
    #[must_use]
    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    /// Returns the lowercase hexadecimal SHA-256 of the captured bytes.
    #[must_use]
    pub fn sha256(&self) -> &str {
        &self.sha256
    }
}

/// Stable identity for a record within its source snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RecordIdentity {
    snapshot: SnapshotIdentity,
    locator: String,
}

impl RecordIdentity {
    /// Returns the containing snapshot identity.
    #[must_use]
    pub fn snapshot(&self) -> &SnapshotIdentity {
        &self.snapshot
    }

    /// Returns the original array pointer or one-based line locator.
    #[must_use]
    pub fn locator(&self) -> &str {
        &self.locator
    }
}

/// Presence and retained source spelling of a source field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum FieldPresence<'a> {
    /// The field key is not present in this record.
    Missing,
    /// A JSON field key is present with JSON `null`.
    Null,
    /// The field key is present; this is its JSON lexeme or decoded text field.
    Value(&'a str),
}

/// A field's decoded name and retained source value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RawField {
    name: String,
    raw_value: String,
}

impl RawField {
    /// Returns the decoded field name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the original JSON value lexeme, or the decoded trimmed text field.
    #[must_use]
    pub fn raw_value(&self) -> &str {
        &self.raw_value
    }
}

/// One captured lexical or morphology record with its source-native fields.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapturedRecord {
    identity: RecordIdentity,
    format: SourceFormat,
    raw_line: Option<String>,
    fields: Vec<RawField>,
    word: Option<String>,
    definition: Option<String>,
    notes: Option<String>,
}

impl CapturedRecord {
    /// Returns the stable source record identity.
    #[must_use]
    pub fn identity(&self) -> &RecordIdentity {
        &self.identity
    }

    /// Returns the original source format.
    #[must_use]
    pub const fn format(&self) -> SourceFormat {
        self.format
    }

    /// Returns the exact source spelling when this row has a lexical headword.
    #[must_use]
    pub fn word(&self) -> Option<&str> {
        self.word.as_deref()
    }

    /// Returns the source definition prose if present as a string.
    #[must_use]
    pub fn definition(&self) -> Option<&str> {
        self.definition.as_deref()
    }

    /// Returns the source-native gloss for an assigned-rafsi row.
    #[must_use]
    pub fn gloss(&self) -> Option<&str> {
        if self.format != SourceFormat::LlgRafsiRows {
            return None;
        }
        self.fields
            .iter()
            .find(|field| field.name == "gloss")
            .map(|field| field.raw_value.as_str())
    }

    /// Returns notes prose if present as a string.
    #[must_use]
    pub fn notes(&self) -> Option<&str> {
        self.notes.as_deref()
    }

    /// Returns the original non-JSON line for fixed-column/text rows.
    #[must_use]
    pub fn raw_line(&self) -> Option<&str> {
        self.raw_line.as_deref()
    }

    /// Returns all decoded top-level names and retained source values.
    #[must_use]
    pub fn fields(&self) -> &[RawField] {
        &self.fields
    }

    /// Returns one field's presence and retained source spelling.
    #[must_use]
    pub fn field(&self, name: &str) -> FieldPresence<'_> {
        match self.fields.iter().find(|field| field.name == name) {
            None => FieldPresence::Missing,
            Some(field)
                if matches!(
                    self.format,
                    SourceFormat::LaLojbanJson | SourceFormat::LensiskuJson
                ) && field.raw_value == "null" =>
            {
                FieldPresence::Null
            }
            Some(field) => FieldPresence::Value(&field.raw_value),
        }
    }
}

/// Distinguishes source-reported forms from assigned LLG rafsi.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum FormRole {
    /// A form reported in a La-Lojban or Lensisku source field.
    SourceReportedForm,
    /// An explicit LLG assigned-rafsi row or fixed-column assertion.
    AssignedRafsi,
}

/// A source-backed or source-reported form assertion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FormAssertion {
    source_record: RecordIdentity,
    headword: String,
    form: String,
    role: FormRole,
    source_field: String,
}

impl FormAssertion {
    /// Returns the source record supporting this form assertion.
    #[must_use]
    pub fn source_record(&self) -> &RecordIdentity {
        &self.source_record
    }

    /// Returns the source headword.
    #[must_use]
    pub fn headword(&self) -> &str {
        &self.headword
    }

    /// Returns the exact form string retained from its source field.
    #[must_use]
    pub fn form(&self) -> &str {
        &self.form
    }

    /// Returns the assertion role.
    #[must_use]
    pub const fn role(&self) -> FormRole {
        self.role
    }

    /// Returns the source field or fixed-column label.
    #[must_use]
    pub fn source_field(&self) -> &str {
        &self.source_field
    }
}

/// One bounded literal cross-reference parsed from an explicit notes clause.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceCrossReference {
    origin: RecordIdentity,
    field: String,
    byte_start: usize,
    byte_end: usize,
    target_spelling: String,
    rule_version: String,
    targets: Vec<RecordIdentity>,
}

impl SourceCrossReference {
    /// Returns the source record containing the clause.
    #[must_use]
    pub fn origin(&self) -> &RecordIdentity {
        &self.origin
    }

    /// Returns the source field name (`notes`).
    #[must_use]
    pub fn field(&self) -> &str {
        &self.field
    }

    /// Returns the start byte offset in decoded UTF-8 notes text.
    #[must_use]
    pub const fn byte_start(&self) -> usize {
        self.byte_start
    }

    /// Returns the exclusive end byte offset in decoded UTF-8 notes text.
    #[must_use]
    pub const fn byte_end(&self) -> usize {
        self.byte_end
    }

    /// Returns the exact decoded target spelling.
    #[must_use]
    pub fn target_spelling(&self) -> &str {
        &self.target_spelling
    }

    /// Returns the bounded parser rule identity.
    #[must_use]
    pub fn rule_version(&self) -> &str {
        &self.rule_version
    }

    /// Returns all exact captured record identities matching the spelling.
    #[must_use]
    pub fn targets(&self) -> &[RecordIdentity] {
        &self.targets
    }
}

/// An explicitly experimental proposal, separate from captured sources.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Proposal {
    id: String,
    namespace: String,
    spelling: String,
    text: String,
}

impl Proposal {
    /// Constructs a proposal under an explicit non-captured namespace.
    ///
    /// # Errors
    ///
    /// Returns an error when the namespace is empty or begins with `captured:`.
    ///
    /// # Examples
    ///
    /// ```
    /// use vlajmaji_lexicon::Proposal;
    /// let proposal = Proposal::new("experiment-01", "flaume", "plum")?;
    /// assert_eq!(proposal.spelling(), "flaume");
    /// # Ok::<(), vlajmaji_lexicon::ImportError>(())
    /// ```
    pub fn new(namespace: &str, spelling: &str, text: &str) -> Result<Self, ImportError> {
        if namespace.is_empty() || namespace.starts_with("captured:") {
            return Err(ImportError::InvalidProposalNamespace);
        }
        let id = proposal_id(namespace, spelling, text);
        Ok(Self {
            id,
            namespace: namespace.into(),
            spelling: spelling.into(),
            text: text.into(),
        })
    }

    /// Returns this proposal's deterministic identifier.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns the isolated proposal namespace.
    #[must_use]
    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    /// Returns the proposal spelling.
    #[must_use]
    pub fn spelling(&self) -> &str {
        &self.spelling
    }

    /// Returns the proposal text.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }
}

/// Typed import and archive failures.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ImportError {
    /// Input bytes do not match the caller's expected SHA-256.
    HashMismatch {
        /// SHA-256 supplied by the caller.
        expected: String,
        /// SHA-256 computed from the input bytes.
        actual: String,
    },
    /// Input is not valid UTF-8 where text is required.
    InvalidUtf8,
    /// The JSON document is malformed or has the wrong root shape.
    InvalidJson(String),
    /// An object repeats a key at any depth.
    DuplicateKey(String),
    /// A known source field has the wrong JSON type.
    WrongFieldType {
        /// Source field name.
        field: String,
        /// Required type description.
        expected: String,
    },
    /// A row is missing a required source field.
    MissingField(String),
    /// A source row could not be interpreted in its declared format.
    InvalidRecord {
        /// Original array pointer or text line.
        locator: String,
        /// Bounded validation failure.
        reason: String,
    },
    /// An archive format version is unsupported.
    UnsupportedArchiveVersion(u32),
    /// A proposal used a captured-source namespace.
    InvalidProposalNamespace,
    /// A proposal archive row's ID does not match its content.
    InvalidProposalIdentity,
}

impl fmt::Display for ImportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::HashMismatch { expected, actual } => write!(
                f,
                "source SHA-256 mismatch: expected {expected}, got {actual}"
            ),
            Self::InvalidUtf8 => f.write_str("source text is not valid UTF-8"),
            Self::InvalidJson(message) => write!(f, "invalid source JSON: {message}"),
            Self::DuplicateKey(key) => write!(f, "duplicate JSON object key {key:?}"),
            Self::WrongFieldType { field, expected } => {
                write!(f, "source field {field:?} must be {expected}")
            }
            Self::MissingField(field) => write!(f, "required source field {field:?} is missing"),
            Self::InvalidRecord { locator, reason } => {
                write!(f, "invalid source record at {locator}: {reason}")
            }
            Self::UnsupportedArchiveVersion(version) => {
                write!(f, "unsupported archive version {version}")
            }
            Self::InvalidProposalNamespace => f.write_str(
                "proposal namespace must be non-empty and must not use captured: prefix",
            ),
            Self::InvalidProposalIdentity => {
                f.write_str("proposal identity does not match its content")
            }
        }
    }
}

impl std::error::Error for ImportError {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct SourceSnapshot {
    metadata: SourceMetadata,
    identity: SnapshotIdentity,
    raw_bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ArchiveDocument {
    version: u32,
    sources: Vec<SourceSnapshot>,
    proposals: Vec<ArchiveProposal>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ArchiveProposal {
    id: String,
    namespace: String,
    spelling: String,
    text: String,
}

impl From<&Proposal> for ArchiveProposal {
    fn from(proposal: &Proposal) -> Self {
        Self {
            id: proposal.id.clone(),
            namespace: proposal.namespace.clone(),
            spelling: proposal.spelling.clone(),
            text: proposal.text.clone(),
        }
    }
}

fn proposal_id(namespace: &str, spelling: &str, text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"relojban-lexicon-proposal-v1\0");
    for component in [namespace.as_bytes(), spelling.as_bytes(), text.as_bytes()] {
        hasher.update((component.len() as u64).to_be_bytes());
        hasher.update(component);
    }
    format!("{:x}", hasher.finalize())
}

fn legacy_proposal_id(namespace: &str, spelling: &str, text: &str) -> String {
    sha256(format!("{namespace}\0{spelling}\0{text}").as_bytes())
}

/// In-memory collection of captured source records and separate proposals.
#[derive(Debug, Clone, Default)]
pub struct Lexicon {
    sources: Vec<SourceSnapshot>,
    records: Vec<CapturedRecord>,
    forms: Vec<FormAssertion>,
    references: Vec<SourceCrossReference>,
    proposals: Vec<Proposal>,
}

impl Lexicon {
    /// Creates an empty lexical collection.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Imports one captured source atomically, retaining its original bytes.
    ///
    /// `expected_sha256` is checked before parsing or mutation. An identical
    /// namespace/hash returns `Ok(false)`; a new hash under that namespace is
    /// a distinct snapshot and returns `Ok(true)`.
    ///
    /// # Errors
    ///
    /// Returns an error for hash mismatch, malformed/invalid text or JSON,
    /// duplicate JSON keys, invalid row shape, or a wrong known-field type.
    /// On every error the collection is unchanged.
    ///
    /// # Examples
    ///
    /// ```
    /// use vlajmaji_lexicon::{Lexicon, SourceFormat, SourceMetadata, TimePrecision};
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let source = SourceMetadata::new("fixture", "x", SourceFormat::LaLojbanJson,
    ///     "memory", "v1", None, TimePrecision::Unknown);
    /// let mut lexicon = Lexicon::new();
    /// assert!(lexicon.import_source(source, br#"[{"word":"x","definition":"y"}]"#, None)?);
    /// assert!(!lexicon.import_source(SourceMetadata::new("fixture", "x",
    ///     SourceFormat::LaLojbanJson, "memory", "v1", None, TimePrecision::Unknown),
    ///     br#"[{"word":"x","definition":"y"}]"#, None)?);
    /// # Ok(())
    /// # }
    /// ```
    pub fn import_source(
        &mut self,
        metadata: SourceMetadata,
        bytes: &[u8],
        expected_sha256: Option<&str>,
    ) -> Result<bool, ImportError> {
        let actual = sha256(bytes);
        if let Some(expected) = expected_sha256
            && expected != actual
        {
            return Err(ImportError::HashMismatch {
                expected: expected.into(),
                actual,
            });
        }
        let identity = SnapshotIdentity {
            namespace: metadata.namespace.clone(),
            sha256: actual,
        };
        if self
            .sources
            .iter()
            .any(|source| source.identity == identity)
        {
            return Ok(false);
        }
        let snapshot = SourceSnapshot {
            metadata,
            identity: identity.clone(),
            raw_bytes: bytes.to_vec(),
        };
        let parsed = parse_source(&snapshot)?;
        // Mutation begins only after the full source has passed all validation.
        self.sources.push(snapshot);
        self.records.extend(parsed.records);
        self.forms.extend(parsed.forms);
        self.references.extend(parsed.references);
        self.resolve_references();
        Ok(true)
    }

    /// Returns captured records in source insertion and source row order.
    #[must_use]
    pub fn records(&self) -> &[CapturedRecord] {
        &self.records
    }

    /// Returns the exact-spelling captured records, without normalization.
    #[must_use]
    pub fn lookup_exact(&self, spelling: &str) -> Vec<&CapturedRecord> {
        self.records
            .iter()
            .filter(|record| record.word.as_deref() == Some(spelling))
            .collect()
    }

    /// Returns all captured source snapshots in insertion order.
    #[must_use]
    pub fn snapshots(&self) -> Vec<&SnapshotIdentity> {
        self.sources.iter().map(|source| &source.identity).collect()
    }

    /// Returns the caller-supplied metadata for a source snapshot.
    #[must_use]
    pub fn source_metadata(&self, identity: &SnapshotIdentity) -> Option<&SourceMetadata> {
        self.sources
            .iter()
            .find(|source| &source.identity == identity)
            .map(|source| &source.metadata)
    }

    /// Returns the exact original bytes for a source snapshot.
    #[must_use]
    pub fn source_bytes(&self, identity: &SnapshotIdentity) -> Option<&[u8]> {
        self.sources
            .iter()
            .find(|source| &source.identity == identity)
            .map(|source| source.raw_bytes.as_slice())
    }

    /// Returns all source-backed form assertions in source insertion order.
    #[must_use]
    pub fn forms(&self) -> &[FormAssertion] {
        &self.forms
    }

    /// Returns bounded literal cross-references extracted from notes.
    #[must_use]
    pub fn cross_references(&self) -> &[SourceCrossReference] {
        &self.references
    }

    /// Adds a separate proposal; it is never included in captured lookup.
    pub fn add_proposal(&mut self, proposal: Proposal) {
        self.proposals.push(proposal);
    }

    /// Returns proposals with an exact spelling match.
    #[must_use]
    pub fn lookup_proposals(&self, spelling: &str) -> Vec<&Proposal> {
        self.proposals
            .iter()
            .filter(|proposal| proposal.spelling == spelling)
            .collect()
    }

    /// Returns every separately stored proposal in insertion order.
    ///
    /// This borrowed view lets rebuildable indexes retain arbitrary proposals
    /// without guessing their spellings or mixing them into captured lookup.
    #[must_use]
    pub fn proposals(&self) -> &[Proposal] {
        &self.proposals
    }

    /// Serializes a deterministic, versioned archive containing source bytes and proposals.
    ///
    /// # Errors
    ///
    /// Returns an error only if the internal archive cannot be serialized.
    pub fn to_archive_bytes(&self) -> Result<Vec<u8>, ImportError> {
        serde_json::to_vec(&ArchiveDocument {
            version: 2,
            sources: self.sources.clone(),
            proposals: self.proposals.iter().map(ArchiveProposal::from).collect(),
        })
        .map_err(|error| ImportError::InvalidJson(error.to_string()))
    }

    /// Rebuilds all source views by revalidating a versioned archive.
    ///
    /// # Errors
    ///
    /// Returns an error for malformed archive JSON, duplicate keys, an
    /// unsupported version, or any invalid source reconstructed from its bytes.
    pub fn from_archive_bytes(bytes: &[u8]) -> Result<Self, ImportError> {
        let text = std::str::from_utf8(bytes).map_err(|_| ImportError::InvalidUtf8)?;
        check_duplicate_keys(text)?;
        let archive: ArchiveDocument = serde_json::from_str(text)
            .map_err(|error| ImportError::InvalidJson(error.to_string()))?;
        if !matches!(archive.version, 1 | 2) {
            return Err(ImportError::UnsupportedArchiveVersion(archive.version));
        }
        let mut lexicon = Self::new();
        for source in archive.sources {
            let expected = source.identity.sha256.clone();
            if source.identity.namespace != source.metadata.namespace
                || source.identity.sha256 != sha256(&source.raw_bytes)
            {
                return Err(ImportError::InvalidJson(
                    "archive snapshot identity does not match metadata and source bytes".into(),
                ));
            }
            let added =
                lexicon.import_source(source.metadata, &source.raw_bytes, Some(&expected))?;
            if !added {
                return Err(ImportError::InvalidJson(
                    "archive repeats a snapshot identity".into(),
                ));
            }
        }
        for proposal in archive.proposals {
            let validated = Proposal::new(&proposal.namespace, &proposal.spelling, &proposal.text)?;
            let expected_id = if archive.version == 1 {
                legacy_proposal_id(&proposal.namespace, &proposal.spelling, &proposal.text)
            } else {
                validated.id.clone()
            };
            if proposal.id != expected_id {
                return Err(ImportError::InvalidProposalIdentity);
            }
            lexicon.proposals.push(validated);
        }
        Ok(lexicon)
    }

    fn resolve_references(&mut self) {
        let exact: BTreeMap<&str, Vec<RecordIdentity>> = {
            let mut map: BTreeMap<&str, Vec<RecordIdentity>> = BTreeMap::new();
            for record in &self.records {
                if let Some(word) = record.word.as_deref() {
                    map.entry(word).or_default().push(record.identity.clone());
                }
            }
            map
        };
        for reference in &mut self.references {
            reference.targets = exact
                .get(reference.target_spelling.as_str())
                .cloned()
                .unwrap_or_default();
        }
    }
}

struct ParsedSource {
    records: Vec<CapturedRecord>,
    forms: Vec<FormAssertion>,
    references: Vec<SourceCrossReference>,
}

fn parse_source(snapshot: &SourceSnapshot) -> Result<ParsedSource, ImportError> {
    match snapshot.metadata.format {
        SourceFormat::LaLojbanJson | SourceFormat::LensiskuJson => parse_json_source(snapshot),
        SourceFormat::LlgGismuFixed | SourceFormat::LlgCmavoFixed | SourceFormat::LlgRafsiRows => {
            parse_text_source(snapshot)
        }
    }
}

fn parse_json_source(snapshot: &SourceSnapshot) -> Result<ParsedSource, ImportError> {
    let text = std::str::from_utf8(&snapshot.raw_bytes).map_err(|_| ImportError::InvalidUtf8)?;
    let raw_rows: Vec<Box<RawValue>> =
        serde_json::from_str(text).map_err(|error| ImportError::InvalidJson(error.to_string()))?;
    let mut parsed = ParsedSource {
        records: Vec::new(),
        forms: Vec::new(),
        references: Vec::new(),
    };
    for (index, raw) in raw_rows.iter().enumerate() {
        check_duplicate_keys(raw.get())?;
        let fields = parse_object(raw.get()).map_err(|error| ImportError::InvalidRecord {
            locator: format!("/{index}"),
            reason: error.to_string(),
        })?;
        validate_known_fields(snapshot.metadata.format, &fields)?;
        let values: BTreeMap<&str, &str> = fields
            .iter()
            .map(|field| (field.name.as_str(), field.raw_value.as_str()))
            .collect();
        let (word_key, def_key, notes_key) = match snapshot.metadata.format {
            SourceFormat::LaLojbanJson => ("word", "definition", "notes"),
            SourceFormat::LensiskuJson => ("valsiword", "definition", "notes"),
            _ => unreachable!(),
        };
        let word = decode_string(values.get(word_key).copied())
            .ok_or_else(|| ImportError::MissingField(word_key.into()))?;
        let definition = decode_string(values.get(def_key).copied())
            .ok_or_else(|| ImportError::MissingField(def_key.into()))?;
        let notes = decode_string(values.get(notes_key).copied());
        let identity = RecordIdentity {
            snapshot: snapshot.identity.clone(),
            locator: format!("/{index}"),
        };
        let record = CapturedRecord {
            identity: identity.clone(),
            format: snapshot.metadata.format,
            raw_line: None,
            fields,
            word: Some(word),
            definition: Some(definition),
            notes: notes.clone(),
        };
        if let Some(notes) = notes.as_deref() {
            parsed.references.extend(extract_see_also(&identity, notes));
        }
        parsed.forms.extend(json_forms(&record));
        parsed.records.push(record);
    }
    Ok(parsed)
}

fn validate_known_fields(format: SourceFormat, fields: &[RawField]) -> Result<(), ImportError> {
    let (expected, required): (BTreeMap<&str, (&str, bool)>, &[&str]) = match format {
        SourceFormat::LaLojbanJson => (
            BTreeMap::from([
                ("word", ("string", true)),
                ("definition", ("string", true)),
                ("notes", ("string", false)),
                ("word_type", ("string", false)),
                ("selmaho", ("string", false)),
                ("definition_id", ("number", false)),
                ("score", ("number", false)),
                ("gloss_keywords", ("array", false)),
                ("place_keywords", ("array", false)),
                ("jargon", ("string", false)),
                ("etymology", ("string", false)),
                ("rafsi", ("string-or-form-array", false)),
                ("user", ("object", false)),
            ]),
            &["word", "definition"],
        ),
        SourceFormat::LensiskuJson => (
            BTreeMap::from([
                ("valsiword", ("string", true)),
                ("definition", ("string", true)),
                ("notes", ("string", false)),
                ("type_name", ("string", false)),
                ("langrealname", ("string", false)),
                ("username", ("string", false)),
                ("sound_url", ("string", false)),
                ("rafsi", ("string-or-form-array", false)),
                ("official_rafsi", ("string-or-form-array", false)),
                ("experimental_rafsi", ("string-or-form-array", false)),
                ("selmaho", ("string", false)),
                ("etymology", ("string", false)),
                ("jargon", ("string", false)),
                ("langid", ("number", false)),
                ("definitionid", ("number", false)),
                ("definitionnum", ("number", false)),
                ("score", ("number", false)),
                ("time", ("number", false)),
                ("valsiid", ("number", false)),
                ("comment_count", ("number", false)),
                ("gloss_keywords", ("array", false)),
                ("place_keywords", ("array", false)),
                ("user", ("object", false)),
                ("can_edit", ("boolean", false)),
                ("has_image", ("boolean", false)),
                ("owner_only", ("boolean", false)),
                ("embedding", ("array", false)),
                ("similarity", ("number", false)),
                ("user_vote", ("number", false)),
            ]),
            &["valsiword", "definition"],
        ),
        _ => return Ok(()),
    };
    for field in fields {
        if let Some((want, required_field)) = expected.get(field.name.as_str()) {
            if field.raw_value == "null" && !*required_field {
                continue;
            }
            let first = field.raw_value.trim_start().chars().next().unwrap_or('\0');
            let valid = match *want {
                "string" => first == '"',
                "number" => first == '-' || first.is_ascii_digit(),
                "array" => first == '[',
                "object" => first == '{',
                "boolean" => field.raw_value == "true" || field.raw_value == "false",
                "string-or-form-array" => first == '"' || string_array_is_valid(&field.raw_value),
                _ => true,
            };
            if !valid {
                return Err(ImportError::WrongFieldType {
                    field: field.name.clone(),
                    expected: (*want).into(),
                });
            }
        }
    }
    for key in required {
        if !fields.iter().any(|field| field.name == *key) {
            return Err(ImportError::MissingField((*key).into()));
        }
    }
    Ok(())
}

fn string_array_is_valid(raw: &str) -> bool {
    serde_json::from_str::<Vec<String>>(raw).is_ok()
}

fn json_forms(record: &CapturedRecord) -> Vec<FormAssertion> {
    let mut forms = Vec::new();
    for field in &record.fields {
        if !matches!(
            field.name.as_str(),
            "rafsi" | "official_rafsi" | "experimental_rafsi"
        ) || field.raw_value == "null"
        {
            continue;
        }
        let strings: Vec<String> =
            if let Ok(value) = serde_json::from_str::<String>(&field.raw_value) {
                vec![value]
            } else {
                serde_json::from_str::<Vec<String>>(&field.raw_value).unwrap_or_default()
            };
        for form in strings
            .into_iter()
            .flat_map(|value| {
                value
                    .split_whitespace()
                    .map(str::to_owned)
                    .collect::<Vec<_>>()
            })
            .filter(|form| !form.is_empty())
        {
            forms.push(FormAssertion {
                source_record: record.identity.clone(),
                headword: record.word.clone().unwrap_or_default(),
                form,
                role: FormRole::SourceReportedForm,
                source_field: field.name.clone(),
            });
        }
    }
    forms
}

fn parse_text_source(snapshot: &SourceSnapshot) -> Result<ParsedSource, ImportError> {
    let text = std::str::from_utf8(&snapshot.raw_bytes).map_err(|_| ImportError::InvalidUtf8)?;
    if !text.is_ascii() {
        return Err(ImportError::InvalidRecord {
            locator: "text".into(),
            reason: "LLG source is not ASCII".into(),
        });
    }
    let mut parsed = ParsedSource {
        records: Vec::new(),
        forms: Vec::new(),
        references: Vec::new(),
    };
    let lines: Vec<&str> = text.lines().collect();
    let start = if snapshot.metadata.format == SourceFormat::LlgRafsiRows {
        0
    } else {
        1
    };
    for (idx, line) in lines.iter().enumerate().skip(start) {
        if line.trim().is_empty() {
            continue;
        }
        let line_no = idx + 1;
        let mut fields = Vec::new();
        let (word, definition) = match snapshot.metadata.format {
            SourceFormat::LlgGismuFixed => {
                if line.len() < 158 {
                    return Err(ImportError::InvalidRecord {
                        locator: line_no.to_string(),
                        reason: "gismu line shorter than fixed definition columns".into(),
                    });
                }
                let word = line.get(1..6).unwrap_or("").trim().to_owned();
                let definition = line.get(62..158).unwrap_or("").trim().to_owned();
                for (label, form) in [
                    ("assigned_rafsi_cvc", line.get(7..10)),
                    ("assigned_rafsi_ccv", line.get(11..14)),
                    ("assigned_rafsi_cvv", line.get(15..19)),
                ] {
                    if let Some(value) = form.map(str::trim) {
                        fields.push(RawField {
                            name: label.into(),
                            raw_value: value.into(),
                        });
                        if value.is_empty() {
                            continue;
                        }
                        parsed.forms.push(FormAssertion {
                            source_record: RecordIdentity {
                                snapshot: snapshot.identity.clone(),
                                locator: line_no.to_string(),
                            },
                            headword: word.clone(),
                            form: value.into(),
                            role: FormRole::AssignedRafsi,
                            source_field: label.into(),
                        });
                    }
                }
                fields.push(RawField {
                    name: "word".into(),
                    raw_value: word.clone(),
                });
                fields.push(RawField {
                    name: "definition".into(),
                    raw_value: definition.clone(),
                });
                (word, definition)
            }
            SourceFormat::LlgCmavoFixed => {
                if line.len() < 62 {
                    return Err(ImportError::InvalidRecord {
                        locator: line_no.to_string(),
                        reason: "cmavo line shorter than required fields".into(),
                    });
                }
                let word = line.get(0..10).unwrap_or("").trim().to_owned();
                let class = line.get(10..20).unwrap_or("").trim().to_owned();
                let keyword = line.get(20..61).unwrap_or("").trim().to_owned();
                let definition = line.get(61..).unwrap_or("").trim().to_owned();
                if word.is_empty()
                    || class.is_empty()
                    || keyword.is_empty()
                    || definition.is_empty()
                {
                    return Err(ImportError::InvalidRecord {
                        locator: line_no.to_string(),
                        reason: "cmavo required fixed-column field empty".into(),
                    });
                }
                fields.extend([
                    RawField {
                        name: "word".into(),
                        raw_value: word.clone(),
                    },
                    RawField {
                        name: "selmaho".into(),
                        raw_value: class,
                    },
                    RawField {
                        name: "keyword".into(),
                        raw_value: keyword,
                    },
                    RawField {
                        name: "definition".into(),
                        raw_value: definition.clone(),
                    },
                ]);
                (word, definition)
            }
            SourceFormat::LlgRafsiRows => {
                let trimmed = line.trim();
                let mut words = trimmed.split_whitespace();
                let rafsi = words.next().unwrap_or("");
                let remainder = trimmed
                    .find(rafsi)
                    .map_or("", |start| &trimmed[start + rafsi.len()..])
                    .trim_start();
                let word = remainder.split_whitespace().next().unwrap_or("");
                let gloss = remainder
                    .find(word)
                    .map_or("", |start| &remainder[start + word.len()..])
                    .trim();
                if rafsi.is_empty() || word.is_empty() || gloss.is_empty() {
                    return Err(ImportError::InvalidRecord {
                        locator: line_no.to_string(),
                        reason: "rafsi row needs form, headword, and gloss".into(),
                    });
                }
                fields.extend([
                    RawField {
                        name: "rafsi".into(),
                        raw_value: rafsi.into(),
                    },
                    RawField {
                        name: "word".into(),
                        raw_value: word.into(),
                    },
                    RawField {
                        name: "gloss".into(),
                        raw_value: gloss.into(),
                    },
                ]);
                parsed.forms.push(FormAssertion {
                    source_record: RecordIdentity {
                        snapshot: snapshot.identity.clone(),
                        locator: line_no.to_string(),
                    },
                    headword: word.into(),
                    form: rafsi.into(),
                    role: FormRole::AssignedRafsi,
                    source_field: "rafsi".into(),
                });
                (word.into(), String::new())
            }
            _ => unreachable!(),
        };
        let identity = RecordIdentity {
            snapshot: snapshot.identity.clone(),
            locator: line_no.to_string(),
        };
        parsed.records.push(CapturedRecord {
            identity,
            format: snapshot.metadata.format,
            raw_line: Some((*line).into()),
            fields,
            word: Some(word),
            definition: (snapshot.metadata.format != SourceFormat::LlgRafsiRows)
                .then_some(definition),
            notes: None,
        });
    }
    Ok(parsed)
}

fn extract_see_also(origin: &RecordIdentity, notes: &str) -> Vec<SourceCrossReference> {
    let mut result = Vec::new();
    let lower = notes.to_ascii_lowercase();
    let markers: Vec<usize> = lower
        .match_indices("see also")
        .filter_map(|(start, marker)| {
            let end = start + marker.len();
            let left_boundary = notes[..start]
                .chars()
                .next_back()
                .is_none_or(|ch| !ch.is_alphanumeric());
            let right_boundary = notes[end..]
                .chars()
                .next()
                .is_none_or(|ch| !ch.is_alphanumeric());
            (left_boundary && right_boundary).then_some(start)
        })
        .collect();
    for (marker_index, start) in markers.iter().copied().enumerate() {
        let marker_end = start + "see also".len();
        let clause_end = notes[marker_end..]
            .find(['.', '!', '?', '\n', '\r', ';'])
            .map_or(notes.len(), |offset| marker_end + offset);
        let end = markers
            .get(marker_index + 1)
            .copied()
            .filter(|next| *next < clause_end)
            .unwrap_or(clause_end);
        let clause = &notes[marker_end..end];
        let mut cursor = 0;
        let mut in_math = false;
        while let Some(open) = clause[cursor..].find('{') {
            let open_at = cursor + open;
            let math_prefix = &clause[cursor..open_at];
            in_math ^= math_prefix.chars().filter(|ch| *ch == '$').count() % 2 == 1;
            if in_math {
                cursor = open_at + 1;
                continue;
            }
            let Some(close) = clause[open_at + 1..].find('}') else {
                break;
            };
            let close_at = open_at + 1 + close;
            if clause[open_at + 1..close_at].contains('$') {
                in_math ^= clause[open_at + 1..close_at]
                    .chars()
                    .filter(|ch| *ch == '$')
                    .count()
                    % 2
                    == 1;
                cursor = close_at + 1;
                continue;
            }
            let target = &clause[open_at + 1..close_at];
            if !target.is_empty() {
                result.push(SourceCrossReference {
                    origin: origin.clone(),
                    field: "notes".into(),
                    byte_start: marker_end + open_at,
                    byte_end: marker_end + close_at + 1,
                    target_spelling: target.into(),
                    rule_version: "see-also-braces-v2".into(),
                    targets: Vec::new(),
                });
            }
            cursor = close_at + 1;
        }
    }
    result
}

fn decode_string(raw: Option<&str>) -> Option<String> {
    raw.and_then(|raw| serde_json::from_str::<String>(raw).ok())
}

fn parse_object(raw: &str) -> Result<Vec<RawField>, ImportError> {
    let mut deserializer = serde_json::Deserializer::from_str(raw);
    // A local visitor returns a side-channel field vector through its value.
    struct CaptureVisitor;
    impl<'de> serde::de::Visitor<'de> for CaptureVisitor {
        type Value = Vec<RawField>;
        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("a JSON object")
        }
        fn visit_map<M: serde::de::MapAccess<'de>>(
            self,
            mut map: M,
        ) -> Result<Self::Value, M::Error> {
            let mut fields = Vec::new();
            let mut seen = BTreeSet::new();
            while let Some(name) = map.next_key::<String>()? {
                if !seen.insert(name.clone()) {
                    return Err(serde::de::Error::custom(format!("duplicate key {name:?}")));
                }
                let value = map.next_value::<Box<RawValue>>()?;
                fields.push(RawField {
                    name,
                    raw_value: value.get().to_owned(),
                });
            }
            Ok(fields)
        }
    }
    let fields = deserializer
        .deserialize_map(CaptureVisitor)
        .map_err(|error| {
            let message = error.to_string();
            if let Some(key) = message.strip_prefix("duplicate key ") {
                ImportError::DuplicateKey(key.into())
            } else {
                ImportError::InvalidJson(message)
            }
        })?;
    deserializer
        .end()
        .map_err(|error| ImportError::InvalidJson(error.to_string()))?;
    Ok(fields)
}

pub(crate) fn check_duplicate_keys(raw: &str) -> Result<(), ImportError> {
    struct AnyVisitor;
    impl<'de> serde::de::Visitor<'de> for AnyVisitor {
        type Value = ();
        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("any JSON value")
        }
        fn visit_bool<E: serde::de::Error>(self, _: bool) -> Result<(), E> {
            Ok(())
        }
        fn visit_i64<E: serde::de::Error>(self, _: i64) -> Result<(), E> {
            Ok(())
        }
        fn visit_u64<E: serde::de::Error>(self, _: u64) -> Result<(), E> {
            Ok(())
        }
        fn visit_f64<E: serde::de::Error>(self, _: f64) -> Result<(), E> {
            Ok(())
        }
        fn visit_str<E: serde::de::Error>(self, _: &str) -> Result<(), E> {
            Ok(())
        }
        fn visit_string<E: serde::de::Error>(self, _: String) -> Result<(), E> {
            Ok(())
        }
        fn visit_unit<E: serde::de::Error>(self) -> Result<(), E> {
            Ok(())
        }
        fn visit_none<E: serde::de::Error>(self) -> Result<(), E> {
            Ok(())
        }
        fn visit_some<D: serde::Deserializer<'de>>(self, d: D) -> Result<(), D::Error> {
            d.deserialize_any(Self)
        }
        fn visit_newtype_struct<D: serde::Deserializer<'de>>(self, d: D) -> Result<(), D::Error> {
            d.deserialize_any(Self)
        }
        fn visit_seq<S: serde::de::SeqAccess<'de>>(self, mut seq: S) -> Result<(), S::Error> {
            while let Some(value) = seq.next_element::<Box<RawValue>>()? {
                check_duplicate_keys(value.get()).map_err(serde::de::Error::custom)?;
            }
            Ok(())
        }
        fn visit_map<M: serde::de::MapAccess<'de>>(self, mut map: M) -> Result<(), M::Error> {
            let mut seen = BTreeSet::new();
            while let Some((key, value)) = map.next_entry::<String, Box<RawValue>>()? {
                if !seen.insert(key.clone()) {
                    return Err(serde::de::Error::custom(format!("duplicate key {key:?}")));
                }
                check_duplicate_keys(value.get()).map_err(serde::de::Error::custom)?;
            }
            Ok(())
        }
    }
    let first = raw.trim_start().chars().next();
    let mut deserializer = serde_json::Deserializer::from_str(raw);
    let scanned = match first {
        Some('{') => deserializer.deserialize_map(AnyVisitor),
        Some('[') => deserializer.deserialize_seq(AnyVisitor),
        Some(_) => return Ok(()),
        None => return Err(ImportError::InvalidJson("empty JSON value".into())),
    };
    scanned.map_err(|error| {
        let message = error.to_string();
        if let Some(pos) = message.find("duplicate key ") {
            ImportError::DuplicateKey(message[pos + 13..].trim_matches('"').to_owned())
        } else {
            ImportError::InvalidJson(message)
        }
    })?;
    deserializer
        .end()
        .map_err(|error| ImportError::InvalidJson(error.to_string()))
}

fn sha256(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut out = String::with_capacity(64);
    for byte in digest {
        use fmt::Write as _;
        let _ = write!(out, "{byte:02x}");
    }
    out
}
