//! Literal reverse-definition and stored-form queries for a mixed workbench.
//!
//! These queries operate only on retained typed dictionary rows. They never
//! synthesize forms or reinterpret source text as markup.

use std::{error::Error, fmt};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use vlajmaji_lexicon as lexicon;
use vlajmaji_lexicon::{AliasScope, CodeNotice, EntityId, FormKind, FormRole, SourceId};
use vlajmaji_store::WorkbenchProfileIdentity;

use crate::{
    DictionaryCompactHit, DictionaryCompactSourceAvailability, DictionaryHit, DictionaryQueryError,
    DictionarySourceAvailability,
};

use super::{
    ConfiguredWorkbenchQueryError, ConfiguredWorkbenchSearchEngine, WorkbenchQueryError,
    WorkbenchQuerySelection, WorkbenchSearchEngine,
};

/// The strict lexical request shape used by direct and configured workbenches.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Versioned request for literal reverse-definition, stored-Form, or bounded Links queries.
pub struct WorkbenchLexicalRequest {
    /// Lexical request schema version; currently exactly one.
    pub version: u32,
    /// One of the two supported literal operations.
    pub query: WorkbenchLexicalQuery,
    /// Source selection and continuation controls.
    pub options: WorkbenchLexicalOptions,
}

/// One supported lexical operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case", deny_unknown_fields)]
pub enum WorkbenchLexicalQuery {
    /// Find literal substrings in ASCII-lowercased stored definitions.
    ReverseDefinition {
        /// Non-empty literal substring. Only ASCII case is folded.
        contains: String,
    },
    /// Find actual stored Forms with this exact spelling.
    Form {
        /// Exact spelling bytes; no trimming or normalization is applied.
        value: String,
        /// Optional legacy role filter, required as a key even when null.
        #[serde(deserialize_with = "super::deserialize_required_option")]
        role: Option<FormRole>,
        /// Optional stored FormKind filter, required as a key even when null.
        #[serde(deserialize_with = "super::deserialize_required_option")]
        kind: Option<FormKind>,
    },
    /// Traverse bounded typed relations and source-clause references.
    Links {
        /// Exact starting Entry spelling; empty text is accepted.
        spelling: String,
        /// Maximum traversal depth, exactly one or two.
        depth: u8,
    },
}

/// Strict source and page controls for a lexical query.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Source, alias, and pagination controls shared by lexical operations.
pub struct WorkbenchLexicalOptions {
    /// Exact source IDs, abbreviations, or aliases in caller order.
    pub sources: Vec<String>,
    /// Alias resolution policy.
    pub alias_scope: AliasScope,
    /// Maximum number of returned records, or required null for all records.
    #[serde(deserialize_with = "super::deserialize_required_option")]
    /// Page size value.
    pub page_size: Option<usize>,
    /// Opaque continuation, or required null on a first page.
    #[serde(deserialize_with = "super::deserialize_required_option")]
    /// Cursor value.
    pub cursor: Option<String>,
}

impl Default for WorkbenchLexicalOptions {
    fn default() -> Self {
        Self {
            sources: Vec::new(),
            alias_scope: AliasScope::ActiveOnly,
            page_size: None,
            cursor: None,
        }
    }
}

/// Pre-page lexical counts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Counts over all matching records before page slicing.
pub struct WorkbenchLexicalTotals {
    /// Either `assertion_occurrences` or `stored_forms`.
    pub unit: String,
    /// Number of matching assertion occurrences or distinct Forms.
    pub records: usize,
    /// Number of sources with at least one matching record.
    pub source_groups: usize,
    /// Union of eligible associated assertion IDs across all matching Forms.
    pub associated_assertions: usize,
}

/// Pre-page counts for one matching source.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Pre-page matching and association counts for one source.
pub struct WorkbenchLexicalSourceTotal {
    /// Canonical source record.
    pub source: lexicon::SourceRecord,
    /// Matching records in this source.
    pub records: usize,
    /// Unique eligible assertion IDs associated with matching Forms.
    pub associated_assertions: usize,
}

/// A source-preserving Form evidence projection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Original evidence identity and the retained payload state for a Form.
pub struct WorkbenchFormEvidence {
    /// Original evidence identity, retained for loaded and catalogue-only rows.
    pub evidence: lexicon::SourceEvidence,
    /// `Loaded` or `CatalogueOnly`, preserving the source-evidence record.
    pub status: String,
    /// `missing_capture` only when the capture payload is absent.
    #[serde(deserialize_with = "super::deserialize_required_option")]
    /// `missing_capture` for a known evidence record whose payload is absent.
    pub reason: Option<String>,
    /// Actual capture metadata, when its payload is retained.
    #[serde(deserialize_with = "super::deserialize_required_option")]
    /// Retained capture metadata for loaded evidence.
    pub capture: Option<lexicon::CaptureMetadata>,
    /// Actual raw occurrence, when its payload is retained.
    #[serde(deserialize_with = "super::deserialize_required_option")]
    /// Retained raw occurrence for loaded evidence.
    pub raw: Option<lexicon::RawOccurrence>,
    /// Actual snapshot attribution when retained.
    #[serde(deserialize_with = "super::deserialize_required_option")]
    /// Retained snapshot attribution, when present.
    pub snapshot_attribution: Option<lexicon::SnapshotAttribution>,
    /// Actual occurrence attribution when retained.
    #[serde(deserialize_with = "super::deserialize_required_option")]
    /// Retained occurrence attribution, when present.
    pub occurrence_attribution: Option<lexicon::OccurrenceAttribution>,
    /// Actual contributor row when attributed.
    #[serde(deserialize_with = "super::deserialize_required_option")]
    /// Retained contributor row, when the evidence names one.
    pub contributor: Option<lexicon::ContributorRecord>,
    /// Actual distributor row when attributed.
    #[serde(deserialize_with = "super::deserialize_required_option")]
    /// Retained distributor row, when the evidence names one.
    pub distributor: Option<lexicon::DistributorRecord>,
}

/// Complete stored Form row and its independent eligible-assertion roster.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Complete typed Form row, its required related rows, evidence, and associations.
pub struct WorkbenchFormRecord {
    /// Original typed stored Form row.
    pub form: lexicon::FormRecord,
    /// Actual related entry.
    pub entry: lexicon::EntryRecord,
    /// Actual selectable source.
    pub source: lexicon::SourceRecord,
    /// Operational role derived only from the two specified FormKinds.
    #[serde(deserialize_with = "super::deserialize_required_option")]
    /// Mapped role value.
    pub mapped_role: Option<FormRole>,
    /// Actual computation row for computed Forms.
    #[serde(deserialize_with = "super::deserialize_required_option")]
    /// Computation value.
    pub computation: Option<lexicon::ComputationRecord>,
    /// Original evidence with actual loaded/catalogue-only status.
    #[serde(deserialize_with = "super::deserialize_required_option")]
    /// Evidence value.
    pub evidence: Option<WorkbenchFormEvidence>,
    /// `eligible_assertions` or `no_current_eligible_assertion`.
    pub association_status: String,
    /// Full actual hits for all eligible assertions on this entry.
    pub associated_assertions: Vec<DictionaryHit>,
}

/// Compact actual Form projection with a key for complete detail.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Compact stored-Form projection with exact identity and a complete-detail key.
pub struct WorkbenchCompactFormRecord {
    /// Stored Form identity.
    pub form_id: EntityId,
    /// Actual Entry identity.
    pub entry_id: EntityId,
    /// Selectable source identity.
    pub source_id: SourceId,
    /// Exact stored entry spelling and presence state.
    pub entry_spelling: lexicon::Presence<String>,
    /// Exact stored Form spelling.
    pub spelling: String,
    /// Stored classification.
    pub kind: FormKind,
    /// Operational legacy role, if one of the two kinds maps.
    #[serde(deserialize_with = "super::deserialize_required_option")]
    /// Mapped role value.
    pub mapped_role: Option<FormRole>,
    /// Original evidence, not a synthesized assertion.
    #[serde(deserialize_with = "super::deserialize_required_option")]
    /// Evidence value.
    pub evidence: Option<lexicon::SourceEvidence>,
    /// Actual computation ID for computed forms.
    #[serde(deserialize_with = "super::deserialize_required_option")]
    /// Computation id value.
    pub computation_id: Option<EntityId>,
    /// `eligible_assertions` or `no_current_eligible_assertion`.
    pub association_status: String,
    /// Complete compact assertion rows for this Entry.
    pub associated_assertions: Vec<DictionaryCompactHit>,
    /// Strict content and binding key for complete Form detail.
    pub detail_ref: WorkbenchFormDetailRef,
}

/// Strict, content-bound key for retrieving one complete stored Form.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Content-bound reference for retrieving a complete stored Form from the same view.
pub struct WorkbenchFormDetailRef {
    /// Key schema version.
    pub version: u32,
    /// Typed-row digest policy.
    pub digest_policy: String,
    /// Exact role discriminator `form`.
    pub role: String,
    /// Direct or configured effective view binding.
    pub binding_sha256: String,
    /// Actual source identity.
    pub source: SourceId,
    /// Actual Entry identity.
    pub entry_id: EntityId,
    /// Actual Form identity.
    pub form_id: EntityId,
    /// SHA-256 of canonical typed Entry bytes.
    pub entry_sha256: String,
    /// SHA-256 of canonical typed Form bytes.
    pub form_sha256: String,
    /// SHA-256 of typed Computation bytes, or required null.
    #[serde(deserialize_with = "super::deserialize_required_option")]
    /// Computation sha256 value.
    pub computation_sha256: Option<String>,
}

/// Full operation-tagged lexical payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "operation",
    content = "response",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum WorkbenchLexicalFullResult {
    /// Literal reverse-definition matches.
    ReverseDefinition(Box<WorkbenchReverseDefinitionFull>),
    /// Actual stored Form matches.
    Form(Box<WorkbenchFormFull>),
    /// Bounded typed-relation and source-clause traversal.
    Links(Box<super::links::WorkbenchLinksFull>),
}

/// Compact operation-tagged lexical payload.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "operation",
    content = "response",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum WorkbenchLexicalCompactResult {
    /// Literal reverse-definition matches.
    ReverseDefinition(Box<WorkbenchReverseDefinitionCompact>),
    /// Actual stored Form matches.
    Form(Box<WorkbenchFormCompact>),
    /// Bounded typed-relation and source-clause traversal.
    Links(Box<super::links::WorkbenchLinksCompact>),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// One source group in a Full reverse-definition page.
pub struct WorkbenchReverseDefinitionFullGroup {
    /// Canonical source for this group.
    pub source: lexicon::SourceRecord,
    /// Matching records before paging.
    pub full_count: usize,
    /// Matching records in this page.
    pub returned_count: usize,
    /// Whether the page omits matching records.
    pub partial: bool,
    /// Matching typed rows returned for this source.
    pub hits: Vec<DictionaryHit>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// One source group in a Compact reverse-definition page.
pub struct WorkbenchReverseDefinitionCompactGroup {
    /// Canonical source for this group.
    pub source: lexicon::SourceRecord,
    /// Matching records before paging.
    pub full_count: usize,
    /// Matching records in this page.
    pub returned_count: usize,
    /// Whether the page omits matching records.
    pub partial: bool,
    /// Matching typed rows returned for this source.
    pub hits: Vec<DictionaryCompactHit>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// One source group in a Full stored-Form page.
pub struct WorkbenchFormFullGroup {
    /// Canonical source for this group.
    pub source: lexicon::SourceRecord,
    /// Matching records before paging.
    pub full_count: usize,
    /// Matching records in this page.
    pub returned_count: usize,
    /// Whether the page omits matching records.
    pub partial: bool,
    /// Matching typed rows returned for this source.
    pub hits: Vec<WorkbenchFormRecord>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// One source group in a Compact stored-Form page.
pub struct WorkbenchFormCompactGroup {
    /// Canonical source for this group.
    pub source: lexicon::SourceRecord,
    /// Matching records before paging.
    pub full_count: usize,
    /// Matching records in this page.
    pub returned_count: usize,
    /// Whether the page omits matching records.
    pub partial: bool,
    /// Matching typed rows returned for this source.
    pub hits: Vec<WorkbenchCompactFormRecord>,
}

/// Full literal reverse-definition response payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Full reverse-definition results and pre-page counts.
pub struct WorkbenchReverseDefinitionFull {
    /// Versioned lexical projection policy.
    pub policy: String,
    /// Dictionary state digest observed by the provider.
    pub state_sha256: String,
    /// The validated lexical query.
    pub query: WorkbenchLexicalQuery,
    /// Selected canonical sources in query order.
    pub resolved_sources: Vec<SourceId>,
    /// Counts before page slicing.
    pub totals: WorkbenchLexicalTotals,
    /// Pre-page counts for each matching source.
    pub source_totals: Vec<WorkbenchLexicalSourceTotal>,
    /// Availability metadata for all registered sources.
    pub availability: Vec<DictionarySourceAvailability>,
    /// Matching rows grouped by source.
    pub groups: Vec<WorkbenchReverseDefinitionFullGroup>,
    #[serde(deserialize_with = "super::deserialize_required_option")]
    /// Continuation token, when more rows remain.
    pub next_cursor: Option<String>,
    /// Code notices retained by the dictionary.
    pub code_notices: Vec<CodeNotice>,
}

/// Compact literal reverse-definition response payload.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Compact reverse-definition results and pre-page counts.
pub struct WorkbenchReverseDefinitionCompact {
    /// Versioned lexical projection policy.
    pub policy: String,
    /// Dictionary state digest observed by the provider.
    pub state_sha256: String,
    /// The validated lexical query.
    pub query: WorkbenchLexicalQuery,
    /// Selected canonical sources in query order.
    pub resolved_sources: Vec<SourceId>,
    /// Counts before page slicing.
    pub totals: WorkbenchLexicalTotals,
    /// Pre-page counts for each matching source.
    pub source_totals: Vec<WorkbenchLexicalSourceTotal>,
    /// Availability metadata for all registered sources.
    pub availability: Vec<DictionaryCompactSourceAvailability>,
    /// Matching rows grouped by source.
    pub groups: Vec<WorkbenchReverseDefinitionCompactGroup>,
    #[serde(deserialize_with = "super::deserialize_required_option")]
    /// Continuation token, when more rows remain.
    pub next_cursor: Option<String>,
    /// Code notices retained by the dictionary.
    pub code_notices: Vec<CodeNotice>,
    /// Compact snapshot binding.
    pub snapshot_sha256: String,
}

/// Full stored-Form response payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Full stored-Form results and pre-page counts.
pub struct WorkbenchFormFull {
    /// Versioned lexical projection policy.
    pub policy: String,
    /// Dictionary state digest observed by the provider.
    pub state_sha256: String,
    /// The validated lexical query.
    pub query: WorkbenchLexicalQuery,
    /// Selected canonical sources in query order.
    pub resolved_sources: Vec<SourceId>,
    /// Counts before page slicing.
    pub totals: WorkbenchLexicalTotals,
    /// Pre-page counts for each matching source.
    pub source_totals: Vec<WorkbenchLexicalSourceTotal>,
    /// Availability metadata for all registered sources.
    pub availability: Vec<DictionarySourceAvailability>,
    /// Matching rows grouped by source.
    pub groups: Vec<WorkbenchFormFullGroup>,
    #[serde(deserialize_with = "super::deserialize_required_option")]
    /// Continuation token, when more rows remain.
    pub next_cursor: Option<String>,
    /// Code notices retained by the dictionary.
    pub code_notices: Vec<CodeNotice>,
}

/// Compact stored-Form response payload.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Compact stored-Form results and pre-page counts.
pub struct WorkbenchFormCompact {
    /// Versioned lexical projection policy.
    pub policy: String,
    /// Dictionary state digest observed by the provider.
    pub state_sha256: String,
    /// The validated lexical query.
    pub query: WorkbenchLexicalQuery,
    /// Selected canonical sources in query order.
    pub resolved_sources: Vec<SourceId>,
    /// Counts before page slicing.
    pub totals: WorkbenchLexicalTotals,
    /// Pre-page counts for each matching source.
    pub source_totals: Vec<WorkbenchLexicalSourceTotal>,
    /// Availability metadata for all registered sources.
    pub availability: Vec<DictionaryCompactSourceAvailability>,
    /// Matching rows grouped by source.
    pub groups: Vec<WorkbenchFormCompactGroup>,
    #[serde(deserialize_with = "super::deserialize_required_option")]
    /// Continuation token, when more rows remain.
    pub next_cursor: Option<String>,
    /// Code notices retained by the dictionary.
    pub code_notices: Vec<CodeNotice>,
    /// Compact snapshot binding.
    pub snapshot_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Direct Full lexical response envelope.
pub struct WorkbenchLexicalFullResponse {
    /// Response schema version.
    pub version: u32,
    /// Response kind discriminator.
    pub kind: String,
    /// Projection format discriminator.
    pub format: String,
    /// Direct or configured effective view binding.
    pub binding_sha256: String,
    /// The view selection used for this response.
    pub selection: WorkbenchQuerySelection,
    /// Operation-specific lexical response.
    pub result: WorkbenchLexicalFullResult,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Direct Compact lexical response envelope.
pub struct WorkbenchLexicalCompactResponse {
    /// Response schema version.
    pub version: u32,
    /// Response kind discriminator.
    pub kind: String,
    /// Projection format discriminator.
    pub format: String,
    /// Direct or configured effective view binding.
    pub binding_sha256: String,
    /// The view selection used for this response.
    pub selection: WorkbenchQuerySelection,
    /// Operation-specific lexical response.
    pub result: WorkbenchLexicalCompactResult,
}

/// Full lexical detail envelope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Direct complete Form-detail response envelope.
pub struct WorkbenchFormDetailResponse {
    /// Response schema version.
    pub version: u32,
    /// Response kind discriminator.
    pub kind: String,
    /// Direct or configured effective view binding.
    pub binding_sha256: String,
    /// The view selection used for this response.
    pub selection: WorkbenchQuerySelection,
    /// Validated Form-detail key.
    pub reference: WorkbenchFormDetailRef,
    /// Complete stored Form and related evidence.
    pub form: WorkbenchFormRecord,
}

/// Configured Full lexical response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Configured Full lexical response envelope.
pub struct ConfiguredWorkbenchLexicalFullResponse {
    /// Response schema version.
    pub version: u32,
    /// Response kind discriminator.
    pub kind: String,
    /// Projection format discriminator.
    pub format: String,
    /// Direct or configured effective view binding.
    pub binding_sha256: String,
    /// Configured profile identity used for this response.
    pub configuration: WorkbenchProfileIdentity,
    /// The view selection used for this response.
    pub selection: WorkbenchQuerySelection,
    /// Operation-specific lexical response.
    pub result: WorkbenchLexicalFullResult,
}

/// Configured Compact lexical response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Configured Compact lexical response envelope.
pub struct ConfiguredWorkbenchLexicalCompactResponse {
    /// Response schema version.
    pub version: u32,
    /// Response kind discriminator.
    pub kind: String,
    /// Projection format discriminator.
    pub format: String,
    /// Direct or configured effective view binding.
    pub binding_sha256: String,
    /// Configured profile identity used for this response.
    pub configuration: WorkbenchProfileIdentity,
    /// The view selection used for this response.
    pub selection: WorkbenchQuerySelection,
    /// Operation-specific lexical response.
    pub result: WorkbenchLexicalCompactResult,
}

/// Configured complete Form detail envelope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Configured complete Form-detail response envelope.
pub struct ConfiguredWorkbenchFormDetailResponse {
    /// Response schema version.
    pub version: u32,
    /// Response kind discriminator.
    pub kind: String,
    /// Direct or configured effective view binding.
    pub binding_sha256: String,
    /// Configured profile identity used for this response.
    pub configuration: WorkbenchProfileIdentity,
    /// The view selection used for this response.
    pub selection: WorkbenchQuerySelection,
    /// Validated Form-detail key.
    pub reference: WorkbenchFormDetailRef,
    /// Complete stored Form and related evidence.
    pub form: WorkbenchFormRecord,
}

/// Invalid request, continuation, or retained Form detail identity.
#[derive(Debug)]
#[non_exhaustive]
pub enum WorkbenchLexicalError {
    /// Request version, query, or options are invalid.
    InvalidRequest {
        /// Description of the invalid request field or value.
        detail: String,
    },
    /// Cursor text or its typed envelope is malformed.
    MalformedCursor {
        /// Description of the malformed cursor.
        detail: String,
    },
    /// Cursor is valid but belongs to another query/view/format.
    StaleCursor,
    /// Cursor position is not in the current ordered result.
    InvalidCursorPosition,
    /// Form-detail key shape or policy is invalid.
    MalformedFormDetail {
        /// Description of the malformed detail key.
        detail: String,
    },
    /// Form-detail key is stale for this view or stored row.
    StaleFormDetail,
    /// Link-detail key shape or policy is invalid.
    MalformedLinkDetail {
        /// Description of the malformed detail key.
        detail: String,
    },
    /// Link-detail key is stale for this view or stored row.
    StaleLinkDetail,
    /// Required retained typed relationships are inconsistent.
    Integrity {
        /// Description of an inconsistent retained relationship.
        detail: String,
    },
    /// Existing dictionary operation failed.
    Dictionary(Box<DictionaryQueryError>),
}

impl fmt::Display for WorkbenchLexicalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRequest { detail } => write!(f, "invalid lexical request: {detail}"),
            Self::MalformedCursor { detail } => write!(f, "malformed lexical cursor: {detail}"),
            Self::StaleCursor => f.write_str("stale lexical cursor"),
            Self::InvalidCursorPosition => f.write_str("invalid lexical cursor position"),
            Self::MalformedFormDetail { detail } => write!(f, "malformed form detail: {detail}"),
            Self::StaleFormDetail => f.write_str("stale form detail"),
            Self::MalformedLinkDetail { detail } => write!(f, "malformed link detail: {detail}"),
            Self::StaleLinkDetail => f.write_str("stale link detail"),
            Self::Integrity { detail } => write!(f, "lexical integrity error: {detail}"),
            Self::Dictionary(error) => write!(f, "dictionary query: {error}"),
        }
    }
}

impl Error for WorkbenchLexicalError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Dictionary(error) => Some(error.as_ref()),
            _ => None,
        }
    }
}

impl From<DictionaryQueryError> for WorkbenchLexicalError {
    fn from(error: DictionaryQueryError) -> Self {
        Self::Dictionary(Box::new(error))
    }
}

const CURSOR_PREFIX: &str = "workbench-lexical-v1:";
const CURSOR_DOMAIN: &str = "workbench-lexical-cursor-v1";
pub(crate) const DETAIL_POLICY: &str = "form-entry-computation-serde-json-v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct LexicalCursorBody {
    pub(crate) version: u32,
    pub(crate) domain: String,
    pub(crate) format: String,
    pub(crate) binding_sha256: String,
    pub(crate) query_sha256: String,
    pub(crate) last: WorkbenchLexicalPosition,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct LexicalCursorEnvelope {
    pub(crate) body: LexicalCursorBody,
    pub(crate) checksum: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum WorkbenchLexicalPosition {
    ReverseDefinition {
        source: SourceId,
        spelling: String,
        occurrence_id: String,
        assertion_id: EntityId,
    },
    Form {
        source: SourceId,
        entry_spelling: String,
        spelling: String,
        kind: FormKind,
        form_id: EntityId,
    },
    Links {
        source: SourceId,
        depth: u8,
        evidence_kind_rank: u8,
        evidence: super::links::WorkbenchLinkEvidenceId,
        orientation_rank: u8,
        from_assertion_id: EntityId,
    },
}

pub(crate) fn mapped_role(kind: FormKind) -> Option<FormRole> {
    match kind {
        FormKind::SourceReportedForm => Some(FormRole::SourceReportedForm),
        FormKind::AssignedRafsi => Some(FormRole::AssignedRafsi),
        _ => None,
    }
}

pub(crate) fn form_kind_rank(kind: FormKind) -> u8 {
    match kind {
        FormKind::SourceVariant => 0,
        FormKind::SourcePronunciation => 1,
        FormKind::SourceReportedForm => 2,
        FormKind::AssignedRafsi => 3,
        FormKind::ComputedStem => 4,
        FormKind::ComputedComponents => 5,
        FormKind::GeneratedCandidate => 6,
    }
}

pub(crate) fn resolve_sources(
    registry: &lexicon::RegistryDocument,
    options: &WorkbenchLexicalOptions,
    availability: &[(SourceId, bool, Option<crate::SourceSelectionFailure>)],
) -> Result<Vec<SourceId>, WorkbenchLexicalError> {
    let mut selected = std::collections::BTreeSet::new();
    if options.sources.is_empty() {
        selected.extend(
            availability
                .iter()
                .filter(|(_, selectable, _)| *selectable)
                .map(|(source, _, _)| source.clone()),
        );
    } else {
        for selector in &options.sources {
            let source = registry
                .resolve_source(selector, options.alias_scope)
                .map_err(|error| {
                    WorkbenchLexicalError::Dictionary(Box::new(DictionaryQueryError::Registry(
                        error,
                    )))
                })?;
            let row = availability
                .iter()
                .find(|(id, _, _)| id == &source.id)
                .ok_or_else(|| WorkbenchLexicalError::Integrity {
                    detail: "resolved source is absent from availability".into(),
                })?;
            if let Some(reason) = row.2 {
                return Err(WorkbenchLexicalError::Dictionary(Box::new(
                    DictionaryQueryError::UnavailableSource {
                        source: source.id.clone(),
                        reason,
                    },
                )));
            }
            selected.insert(source.id.clone());
        }
    }
    Ok(selected.into_iter().collect())
}

fn sha256(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut out = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(out, "{byte:02x}");
    }
    out
}

pub(crate) fn typed_sha256<T: Serialize>(value: &T) -> Result<String, WorkbenchLexicalError> {
    let bytes = serde_json::to_vec(value).map_err(|error| {
        WorkbenchLexicalError::Dictionary(Box::new(DictionaryQueryError::Dictionary(
            lexicon::DictionaryError::InvalidJson(error.to_string()),
        )))
    })?;
    Ok(sha256(&bytes))
}

pub(crate) fn make_form_detail_ref(
    binding_sha256: &str,
    source: &SourceId,
    entry: &lexicon::EntryRecord,
    form: &lexicon::FormRecord,
    computation: Option<&lexicon::ComputationRecord>,
) -> Result<WorkbenchFormDetailRef, WorkbenchLexicalError> {
    Ok(WorkbenchFormDetailRef {
        version: 1,
        digest_policy: DETAIL_POLICY.into(),
        role: "form".into(),
        binding_sha256: binding_sha256.into(),
        source: source.clone(),
        entry_id: entry.id.clone(),
        form_id: form.id.clone(),
        entry_sha256: typed_sha256(entry)?,
        form_sha256: typed_sha256(form)?,
        computation_sha256: computation.map(typed_sha256).transpose()?,
    })
}

#[derive(Serialize)]
struct LexicalQueryBinding<'a> {
    query: &'a WorkbenchLexicalQuery,
    selectors: &'a [String],
    resolved_sources: &'a [SourceId],
    alias_scope: AliasScope,
}

pub(crate) fn query_sha256(
    request: &WorkbenchLexicalRequest,
    resolved_sources: &[SourceId],
) -> Result<String, WorkbenchLexicalError> {
    typed_sha256(&LexicalQueryBinding {
        query: &request.query,
        selectors: &request.options.sources,
        resolved_sources,
        alias_scope: request.options.alias_scope,
    })
}

pub(crate) fn encode_cursor(
    request: &WorkbenchLexicalRequest,
    resolved_sources: &[SourceId],
    format: &str,
    binding_sha256: &str,
    last: WorkbenchLexicalPosition,
) -> Result<String, WorkbenchLexicalError> {
    let body = LexicalCursorBody {
        version: 1,
        domain: CURSOR_DOMAIN.into(),
        format: format.into(),
        binding_sha256: binding_sha256.into(),
        query_sha256: query_sha256(request, resolved_sources)?,
        last,
    };
    let checksum = typed_sha256(&body)?;
    let bytes = serde_json::to_vec(&LexicalCursorEnvelope { body, checksum }).map_err(|error| {
        WorkbenchLexicalError::Dictionary(Box::new(DictionaryQueryError::Dictionary(
            lexicon::DictionaryError::InvalidJson(error.to_string()),
        )))
    })?;
    let mut hex = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write as _;
        let _ = write!(hex, "{byte:02x}");
    }
    Ok(format!("{CURSOR_PREFIX}{hex}"))
}

pub(crate) fn decode_cursor(
    token: &str,
    request: &WorkbenchLexicalRequest,
    resolved_sources: &[SourceId],
    format: &str,
    binding_sha256: &str,
) -> Result<WorkbenchLexicalPosition, WorkbenchLexicalError> {
    let malformed = |detail: &str| WorkbenchLexicalError::MalformedCursor {
        detail: detail.into(),
    };
    let Some(hex) = token.strip_prefix(CURSOR_PREFIX) else {
        return Err(malformed("unsupported cursor namespace"));
    };
    if hex.len() % 2 != 0
        || !hex
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(malformed("cursor body is not lowercase even-length hex"));
    }
    let bytes = hex
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| {
            let digit = |byte: u8| match byte {
                b'0'..=b'9' => byte - b'0',
                b'a'..=b'f' => byte - b'a' + 10,
                _ => 0,
            };
            (digit(pair[0]) << 4) | digit(pair[1])
        })
        .collect::<Vec<_>>();
    let envelope: LexicalCursorEnvelope = serde_json::from_slice(&bytes)
        .map_err(|error| malformed(&format!("invalid typed envelope: {error}")))?;
    if envelope.body.version != 1 || envelope.body.domain != CURSOR_DOMAIN {
        return Err(malformed("unsupported cursor body version or domain"));
    }
    if !matches!(envelope.body.format.as_str(), "full" | "compact")
        || !is_sha256(&envelope.body.binding_sha256)
        || !is_sha256(&envelope.body.query_sha256)
        || !is_sha256(&envelope.checksum)
    {
        return Err(malformed("cursor format or digest syntax is invalid"));
    }
    let expected_checksum = typed_sha256(&envelope.body)?;
    if envelope.checksum != expected_checksum {
        return Err(malformed("cursor checksum does not match its typed body"));
    }
    if let WorkbenchLexicalPosition::Links {
        depth,
        evidence_kind_rank,
        evidence,
        orientation_rank,
        ..
    } = &envelope.body.last
    {
        let identity_shape_valid = match evidence {
            super::links::WorkbenchLinkEvidenceId::TypedRelation { relation_id } => {
                !relation_id.as_str().is_empty() && *evidence_kind_rank == 0
            }
            super::links::WorkbenchLinkEvidenceId::SourceClause {
                context_id,
                byte_start,
                byte_end,
                rule_version,
                target_spelling,
            } => {
                !context_id.as_str().is_empty()
                    && byte_end.checked_sub(*byte_start) == target_spelling.len().checked_add(2)
                    && rule_version == "see-also-braces-v2"
                    && !target_spelling.is_empty()
                    && *evidence_kind_rank == 1
            }
        };
        if !(1..=2).contains(depth) || *orientation_rank > 1 || !identity_shape_valid {
            return Err(malformed("Links cursor position is malformed"));
        }
    }
    let query_sha = query_sha256(request, resolved_sources)?;
    if envelope.body.format != format
        || envelope.body.binding_sha256 != binding_sha256
        || envelope.body.query_sha256 != query_sha
    {
        return Err(WorkbenchLexicalError::StaleCursor);
    }
    Ok(envelope.body.last)
}

impl WorkbenchSearchEngine {
    /// Execute one Full lexical query with before/after freshness checks.
    ///
    /// Stored Forms are selected by their exact spelling and retained kind;
    /// this operation does not generate or normalize a Form.
    ///
    /// ```no_run
    /// use std::path::Path;
    /// use vlajmaji_search::{
    ///     WorkbenchLexicalOptions, WorkbenchLexicalQuery, WorkbenchLexicalRequest,
    ///     WorkbenchSearchEngine,
    /// };
    ///
    /// let mut engine = WorkbenchSearchEngine::open(
    ///     Path::new("workbench-example/lexical-workbench/view-historical-r1"),
    /// )?;
    /// let request = WorkbenchLexicalRequest {
    ///     version: 1,
    ///     query: WorkbenchLexicalQuery::Form {
    ///         value: "farvi-form".into(),
    ///         role: None,
    ///         kind: None,
    ///     },
    ///     options: WorkbenchLexicalOptions::default(),
    /// };
    /// let response = engine.execute_lexical(&request)?;
    /// assert_eq!(response.format, "full");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    /// Returns freshness, request, source-selection, cursor, or retained-row errors.
    pub fn execute_lexical(
        &mut self,
        request: &WorkbenchLexicalRequest,
    ) -> Result<WorkbenchLexicalFullResponse, WorkbenchQueryError> {
        self.execute_lexical_with_hook(request, || {})
    }

    fn execute_lexical_with_hook<F>(
        &mut self,
        request: &WorkbenchLexicalRequest,
        after_inner: F,
    ) -> Result<WorkbenchLexicalFullResponse, WorkbenchQueryError>
    where
        F: FnOnce(),
    {
        self.view.ensure_current()?;
        let binding = self.view.binding().binding_sha256().to_owned();
        let result: Result<WorkbenchLexicalFullResult, WorkbenchQueryError> = (|| {
            validate_request(request).map_err(WorkbenchQueryError::from)?;
            if matches!(request.query, WorkbenchLexicalQuery::Links { .. }) {
                self.lexical_links_full_inner(request, &binding)
                    .map(|links| WorkbenchLexicalFullResult::Links(Box::new(links)))
            } else {
                self.full_engine()
                    .map_err(WorkbenchLexicalError::from)
                    .map_err(WorkbenchQueryError::from)?
                    .lexical_full(request, &binding)
                    .map_err(WorkbenchQueryError::from)
            }
        })();
        after_inner();
        self.view.ensure_current()?;
        let result = result?;
        Ok(WorkbenchLexicalFullResponse {
            version: 1,
            kind: "vlajmaji-workbench-lexical".into(),
            format: "full".into(),
            binding_sha256: binding,
            selection: self.selection.clone(),
            result,
        })
    }

    /// Execute one Compact lexical query with before/after freshness checks.
    ///
    /// Compact Form hits retain their typed identities and provide a detail
    /// reference for a later complete-row lookup.
    ///
    /// ```no_run
    /// use std::path::Path;
    /// use vlajmaji_search::{
    ///     WorkbenchLexicalOptions, WorkbenchLexicalQuery, WorkbenchLexicalRequest,
    ///     WorkbenchSearchEngine,
    /// };
    ///
    /// let mut engine = WorkbenchSearchEngine::open(
    ///     Path::new("workbench-example/lexical-workbench/view-historical-r1"),
    /// )?;
    /// let request = WorkbenchLexicalRequest {
    ///     version: 1,
    ///     query: WorkbenchLexicalQuery::Form {
    ///         value: "farvi-form".into(),
    ///         role: None,
    ///         kind: None,
    ///     },
    ///     options: WorkbenchLexicalOptions::default(),
    /// };
    /// let response = engine.execute_lexical_compact(&request)?;
    /// assert_eq!(response.format, "compact");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    /// Returns freshness, request, source-selection, cursor, or retained-row errors.
    pub fn execute_lexical_compact(
        &mut self,
        request: &WorkbenchLexicalRequest,
    ) -> Result<WorkbenchLexicalCompactResponse, WorkbenchQueryError> {
        self.execute_lexical_compact_with_hook(request, || {})
    }

    fn execute_lexical_compact_with_hook<F>(
        &mut self,
        request: &WorkbenchLexicalRequest,
        after_inner: F,
    ) -> Result<WorkbenchLexicalCompactResponse, WorkbenchQueryError>
    where
        F: FnOnce(),
    {
        self.view.ensure_current()?;
        let binding = self.view.binding().binding_sha256().to_owned();
        let result: Result<WorkbenchLexicalCompactResult, WorkbenchQueryError> = (|| {
            validate_request(request).map_err(WorkbenchQueryError::from)?;
            if matches!(request.query, WorkbenchLexicalQuery::Links { .. }) {
                self.lexical_links_compact_inner(request, &binding)
                    .map(|links| WorkbenchLexicalCompactResult::Links(Box::new(links)))
            } else {
                self.compact_engine()
                    .map_err(WorkbenchLexicalError::from)
                    .map_err(WorkbenchQueryError::from)?
                    .lexical_compact(request, &binding)
                    .map_err(WorkbenchQueryError::from)
            }
        })();
        after_inner();
        self.view.ensure_current()?;
        let result = result?;
        Ok(WorkbenchLexicalCompactResponse {
            version: 1,
            kind: "vlajmaji-workbench-lexical".into(),
            format: "compact".into(),
            binding_sha256: binding,
            selection: self.selection.clone(),
            result,
        })
    }

    /// Resolve complete Form detail through the same cached Compact engine.
    ///
    /// Obtain the reference from a Compact response and pass that response's
    /// binding back unchanged. A key from another view is stale by design.
    ///
    /// ```no_run
    /// use std::path::Path;
    /// use vlajmaji_search::{
    ///     WorkbenchLexicalCompactResult, WorkbenchLexicalOptions,
    ///     WorkbenchLexicalQuery, WorkbenchLexicalRequest, WorkbenchSearchEngine,
    /// };
    ///
    /// let mut engine = WorkbenchSearchEngine::open(
    ///     Path::new("workbench-example/lexical-workbench/view-historical-r1"),
    /// )?;
    /// let request = WorkbenchLexicalRequest {
    ///     version: 1,
    ///     query: WorkbenchLexicalQuery::Form {
    ///         value: "farvi-form".into(),
    ///         role: None,
    ///         kind: None,
    ///     },
    ///     options: WorkbenchLexicalOptions::default(),
    /// };
    /// let compact = engine.execute_lexical_compact(&request)?;
    /// let WorkbenchLexicalCompactResult::Form(forms) = compact.result else {
    ///     unreachable!("the request is a stored-Form query")
    /// };
    /// let detail_ref = &forms.groups[0].hits[0].detail_ref;
    /// let detail = engine.form_detail(detail_ref, &compact.binding_sha256)?;
    /// assert_eq!(detail.form.form.id, forms.groups[0].hits[0].form_id);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    /// Returns freshness, binding, key, source, or retained-row errors.
    pub fn form_detail(
        &mut self,
        reference: &WorkbenchFormDetailRef,
        expected_binding: &str,
    ) -> Result<WorkbenchFormDetailResponse, WorkbenchQueryError> {
        self.form_detail_with_hook(reference, expected_binding, || {})
    }

    fn form_detail_with_hook<F>(
        &mut self,
        reference: &WorkbenchFormDetailRef,
        expected_binding: &str,
        after_inner: F,
    ) -> Result<WorkbenchFormDetailResponse, WorkbenchQueryError>
    where
        F: FnOnce(),
    {
        self.view.ensure_current()?;
        let binding = self.view.binding().binding_sha256().to_owned();
        let result: Result<_, WorkbenchQueryError> = (|| {
            validate_outer_binding(expected_binding, &binding)?;
            validate_form_detail_key(reference, &binding)?;
            self.compact_engine()?
                .form_detail(reference, &binding)
                .map_err(WorkbenchQueryError::from)
        })();
        after_inner();
        self.view.ensure_current()?;
        let (reference, form) = result?;
        Ok(WorkbenchFormDetailResponse {
            version: 1,
            kind: "vlajmaji-workbench-form-detail".into(),
            binding_sha256: binding,
            selection: self.selection.clone(),
            reference,
            form,
        })
    }

    /// Resolve complete retained relation or source-clause detail using the same Compact view.
    ///
    /// Obtain the key from a Links Compact result and keep the response binding with it. Detail
    /// uses the current all-source view, so the original request's source filter does not narrow
    /// its evidence or target observations.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use vlajmaji_search::workbench::{
    ///     WorkbenchLexicalCompactResult, WorkbenchLexicalRequest, WorkbenchQueryError,
    ///     WorkbenchSearchEngine,
    /// };
    ///
    /// # fn show_detail(
    /// #     engine: &mut WorkbenchSearchEngine,
    /// #     request: &WorkbenchLexicalRequest,
    /// # ) -> Result<(), WorkbenchQueryError> {
    /// let response = engine.execute_lexical_compact(request)?;
    /// let binding = response.binding_sha256.clone();
    /// if let WorkbenchLexicalCompactResult::Links(links) = response.result {
    ///     if let Some(step) = links.groups.iter().flat_map(|group| &group.hits).next() {
    ///         let detail = engine.link_detail(&step.detail_ref, &binding)?;
    ///         assert_eq!(detail.reference, step.detail_ref);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns freshness errors when the prepared view changes, typed query or projection errors,
    /// and malformed or stale detail-key errors.
    pub fn link_detail(
        &mut self,
        reference: &super::links::WorkbenchLinkDetailRef,
        expected_binding: &str,
    ) -> Result<super::links::WorkbenchLinkDetailResponse, WorkbenchQueryError> {
        self.link_detail_with_hook(reference, expected_binding, || {})
    }

    fn link_detail_with_hook<F>(
        &mut self,
        reference: &super::links::WorkbenchLinkDetailRef,
        expected_binding: &str,
        after_inner: F,
    ) -> Result<super::links::WorkbenchLinkDetailResponse, WorkbenchQueryError>
    where
        F: FnOnce(),
    {
        self.view.ensure_current()?;
        let binding = self.view.binding().binding_sha256().to_owned();
        let result: Result<_, WorkbenchQueryError> = (|| {
            validate_outer_binding(expected_binding, &binding)?;
            let (record, origin_resolution, target_observations) =
                super::links::build_compact_link_detail(self, reference, &binding)?;
            Ok((record, origin_resolution, target_observations))
        })();
        after_inner();
        self.view.ensure_current()?;
        let (record, origin_resolution, target_observations) = result?;
        Ok(super::links::WorkbenchLinkDetailResponse {
            version: 1,
            kind: "vlajmaji-workbench-link-detail".into(),
            binding_sha256: binding,
            selection: self.selection.clone(),
            reference: reference.clone(),
            record,
            origin_resolution,
            target_observations,
        })
    }
}

impl ConfiguredWorkbenchSearchEngine {
    /// Execute one configured Full lexical query with source/config freshness guards.
    ///
    /// Open by profile name for named-mode identity, or pass `None` to query the
    /// current default. Both modes resolve the same stored rows when they select
    /// the same profile.
    ///
    /// ```no_run
    /// use std::path::Path;
    /// use vlajmaji_search::{
    ///     ConfiguredWorkbenchSearchEngine, WorkbenchLexicalOptions,
    ///     WorkbenchLexicalQuery, WorkbenchLexicalRequest,
    /// };
    ///
    /// let mut engine = ConfiguredWorkbenchSearchEngine::open(
    ///     Path::new("workbench-example/lexical-workbench/workbench-profiles.json"),
    ///     Some("lexical"),
    /// )?;
    /// let request = WorkbenchLexicalRequest {
    ///     version: 1,
    ///     query: WorkbenchLexicalQuery::Form {
    ///         value: "farvi-form".into(),
    ///         role: None,
    ///         kind: None,
    ///     },
    ///     options: WorkbenchLexicalOptions::default(),
    /// };
    /// let response = engine.execute_lexical(&request)?;
    /// assert_eq!(response.configuration.requested_profile.as_deref(), Some("lexical"));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    /// Returns profile, request, source-selection, cursor, or retained-row errors.
    pub fn execute_lexical(
        &mut self,
        request: &WorkbenchLexicalRequest,
    ) -> Result<ConfiguredWorkbenchLexicalFullResponse, ConfiguredWorkbenchQueryError> {
        self.execute_lexical_with_hook(request, || {})
    }

    fn execute_lexical_with_hook<F>(
        &mut self,
        request: &WorkbenchLexicalRequest,
        after_inner: F,
    ) -> Result<ConfiguredWorkbenchLexicalFullResponse, ConfiguredWorkbenchQueryError>
    where
        F: FnOnce(),
    {
        self.view
            .ensure_current()
            .map_err(|error| ConfiguredWorkbenchQueryError::Profile(Box::new(error)))?;
        let configuration = self.view.identity().clone();
        let binding = configuration.effective_binding_sha256.clone();
        let result: Result<WorkbenchLexicalFullResult, WorkbenchQueryError> = (|| {
            validate_request(request).map_err(WorkbenchQueryError::from)?;
            if matches!(request.query, WorkbenchLexicalQuery::Links { .. }) {
                self.lexical_links_full_inner(request, &binding)
                    .map(|links| WorkbenchLexicalFullResult::Links(Box::new(links)))
            } else {
                self.full_engine()
                    .map_err(WorkbenchQueryError::from)?
                    .lexical_full(request, &binding)
                    .map_err(WorkbenchQueryError::from)
            }
        })();
        after_inner();
        if let Err(error) = self.view.ensure_current() {
            return Err(ConfiguredWorkbenchQueryError::Profile(Box::new(error)));
        }
        let result =
            result.map_err(|error| ConfiguredWorkbenchQueryError::Query(Box::new(error)))?;
        Ok(ConfiguredWorkbenchLexicalFullResponse {
            version: 1,
            kind: "vlajmaji-workbench-lexical".into(),
            format: "full".into(),
            binding_sha256: binding,
            configuration,
            selection: self.selection.clone(),
            result,
        })
    }

    /// Execute one configured Compact lexical query with source/config guards.
    ///
    /// Use the Compact response's effective binding with its Form detail key.
    ///
    /// ```no_run
    /// use std::path::Path;
    /// use vlajmaji_search::{
    ///     ConfiguredWorkbenchLexicalCompactResponse, ConfiguredWorkbenchSearchEngine,
    ///     WorkbenchLexicalOptions, WorkbenchLexicalQuery, WorkbenchLexicalRequest,
    /// };
    ///
    /// let mut engine = ConfiguredWorkbenchSearchEngine::open(
    ///     Path::new("workbench-example/lexical-workbench/workbench-profiles.json"),
    ///     None,
    /// )?;
    /// let request = WorkbenchLexicalRequest {
    ///     version: 1,
    ///     query: WorkbenchLexicalQuery::Form {
    ///         value: "farvi-form".into(),
    ///         role: None,
    ///         kind: None,
    ///     },
    ///     options: WorkbenchLexicalOptions::default(),
    /// };
    /// let response: ConfiguredWorkbenchLexicalCompactResponse =
    ///     engine.execute_lexical_compact(&request)?;
    /// assert_eq!(response.format, "compact");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    /// Returns profile, request, source-selection, cursor, or retained-row errors.
    pub fn execute_lexical_compact(
        &mut self,
        request: &WorkbenchLexicalRequest,
    ) -> Result<ConfiguredWorkbenchLexicalCompactResponse, ConfiguredWorkbenchQueryError> {
        self.execute_lexical_compact_with_hook(request, || {})
    }

    fn execute_lexical_compact_with_hook<F>(
        &mut self,
        request: &WorkbenchLexicalRequest,
        after_inner: F,
    ) -> Result<ConfiguredWorkbenchLexicalCompactResponse, ConfiguredWorkbenchQueryError>
    where
        F: FnOnce(),
    {
        self.view
            .ensure_current()
            .map_err(|error| ConfiguredWorkbenchQueryError::Profile(Box::new(error)))?;
        let configuration = self.view.identity().clone();
        let binding = configuration.effective_binding_sha256.clone();
        let result: Result<WorkbenchLexicalCompactResult, WorkbenchQueryError> = (|| {
            validate_request(request).map_err(WorkbenchQueryError::from)?;
            if matches!(request.query, WorkbenchLexicalQuery::Links { .. }) {
                self.lexical_links_compact_inner(request, &binding)
                    .map(|links| WorkbenchLexicalCompactResult::Links(Box::new(links)))
            } else {
                self.compact_engine()
                    .map_err(WorkbenchQueryError::from)?
                    .lexical_compact(request, &binding)
                    .map_err(WorkbenchQueryError::from)
            }
        })();
        after_inner();
        if let Err(error) = self.view.ensure_current() {
            return Err(ConfiguredWorkbenchQueryError::Profile(Box::new(error)));
        }
        let result =
            result.map_err(|error| ConfiguredWorkbenchQueryError::Query(Box::new(error)))?;
        Ok(ConfiguredWorkbenchLexicalCompactResponse {
            version: 1,
            kind: "vlajmaji-workbench-lexical".into(),
            format: "compact".into(),
            binding_sha256: binding,
            configuration,
            selection: self.selection.clone(),
            result,
        })
    }

    /// Resolve configured complete Form detail through cached Compact state.
    ///
    /// The detail reference and expected binding must come from the same
    /// configured effective view. Refreshing or changing the configuration
    /// invalidates previously captured detail identity.
    ///
    /// ```no_run
    /// use std::path::Path;
    /// use vlajmaji_search::{
    ///     ConfiguredWorkbenchLexicalCompactResponse, ConfiguredWorkbenchSearchEngine,
    ///     WorkbenchLexicalOptions, WorkbenchLexicalQuery, WorkbenchLexicalRequest,
    /// };
    ///
    /// let mut engine = ConfiguredWorkbenchSearchEngine::open(
    ///     Path::new("workbench-example/lexical-workbench/workbench-profiles.json"),
    ///     Some("lexical"),
    /// )?;
    /// let request = WorkbenchLexicalRequest {
    ///     version: 1,
    ///     query: WorkbenchLexicalQuery::Form {
    ///         value: "farvi-form".into(),
    ///         role: None,
    ///         kind: None,
    ///     },
    ///     options: WorkbenchLexicalOptions::default(),
    /// };
    /// let compact: ConfiguredWorkbenchLexicalCompactResponse =
    ///     engine.execute_lexical_compact(&request)?;
    /// let detail_ref = match &compact.result {
    ///     vlajmaji_search::WorkbenchLexicalCompactResult::Form(forms) =>
    ///         &forms.groups[0].hits[0].detail_ref,
    ///     _ => unreachable!("the request is a stored-Form query"),
    /// };
    /// let detail = engine.form_detail(detail_ref, &compact.binding_sha256)?;
    /// assert_eq!(detail.binding_sha256, compact.binding_sha256);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    /// Returns profile, binding, key, source, or retained-row errors.
    pub fn form_detail(
        &mut self,
        reference: &WorkbenchFormDetailRef,
        expected_effective_binding: &str,
    ) -> Result<ConfiguredWorkbenchFormDetailResponse, ConfiguredWorkbenchQueryError> {
        self.form_detail_with_hook(reference, expected_effective_binding, || {})
    }

    fn form_detail_with_hook<F>(
        &mut self,
        reference: &WorkbenchFormDetailRef,
        expected_effective_binding: &str,
        after_inner: F,
    ) -> Result<ConfiguredWorkbenchFormDetailResponse, ConfiguredWorkbenchQueryError>
    where
        F: FnOnce(),
    {
        self.view
            .ensure_current()
            .map_err(|error| ConfiguredWorkbenchQueryError::Profile(Box::new(error)))?;
        let configuration = self.view.identity().clone();
        let binding = configuration.effective_binding_sha256.clone();
        let result: Result<_, WorkbenchQueryError> = (|| {
            validate_outer_binding(expected_effective_binding, &binding)?;
            validate_form_detail_key(reference, &binding).map_err(WorkbenchQueryError::from)?;
            self.compact_engine()
                .map_err(WorkbenchQueryError::from)?
                .form_detail(reference, &binding)
                .map_err(WorkbenchQueryError::from)
        })();
        after_inner();
        if let Err(error) = self.view.ensure_current() {
            return Err(ConfiguredWorkbenchQueryError::Profile(Box::new(error)));
        }
        let (reference, form) =
            result.map_err(|error| ConfiguredWorkbenchQueryError::Query(Box::new(error)))?;
        Ok(ConfiguredWorkbenchFormDetailResponse {
            version: 1,
            kind: "vlajmaji-workbench-form-detail".into(),
            binding_sha256: binding,
            configuration,
            selection: self.selection.clone(),
            reference,
            form,
        })
    }

    /// Resolve complete retained relation or source-clause detail in this configured view.
    ///
    /// Use a key and effective binding returned by a Compact Links query from this same configured
    /// engine. The detail projection uses all sources in the configured view; it is not restricted
    /// by the query's source filter.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use vlajmaji_search::workbench::{
    ///     ConfiguredWorkbenchSearchEngine, WorkbenchLexicalCompactResult,
    ///     WorkbenchLexicalRequest, ConfiguredWorkbenchQueryError,
    /// };
    ///
    /// # fn show_detail(
    /// #     engine: &mut ConfiguredWorkbenchSearchEngine,
    /// #     request: &WorkbenchLexicalRequest,
    /// # ) -> Result<(), ConfiguredWorkbenchQueryError> {
    /// let response = engine.execute_lexical_compact(request)?;
    /// let effective_binding = response.binding_sha256.clone();
    /// if let WorkbenchLexicalCompactResult::Links(links) = response.result {
    ///     if let Some(step) = links.groups.iter().flat_map(|group| &group.hits).next() {
    ///         let detail = engine.link_detail(&step.detail_ref, &effective_binding)?;
    ///         assert_eq!(detail.reference, step.detail_ref);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns profile freshness and query errors, including malformed or stale detail-key
    /// errors. A configuration change during resolution is reported as a profile error.
    pub fn link_detail(
        &mut self,
        reference: &super::links::WorkbenchLinkDetailRef,
        expected_effective_binding: &str,
    ) -> Result<super::links::ConfiguredWorkbenchLinkDetailResponse, ConfiguredWorkbenchQueryError>
    {
        self.link_detail_with_hook(reference, expected_effective_binding, || {})
    }

    fn link_detail_with_hook<F>(
        &mut self,
        reference: &super::links::WorkbenchLinkDetailRef,
        expected_effective_binding: &str,
        after_inner: F,
    ) -> Result<super::links::ConfiguredWorkbenchLinkDetailResponse, ConfiguredWorkbenchQueryError>
    where
        F: FnOnce(),
    {
        self.view
            .ensure_current()
            .map_err(|error| ConfiguredWorkbenchQueryError::Profile(Box::new(error)))?;
        let configuration = self.view.identity().clone();
        let binding = configuration.effective_binding_sha256.clone();
        let result: Result<_, WorkbenchQueryError> = (|| {
            validate_outer_binding(expected_effective_binding, &binding)?;
            super::links::build_compact_link_detail(self, reference, &binding)
        })();
        after_inner();
        if let Err(error) = self.view.ensure_current() {
            return Err(ConfiguredWorkbenchQueryError::Profile(Box::new(error)));
        }
        let (record, origin_resolution, target_observations) =
            result.map_err(|error| ConfiguredWorkbenchQueryError::Query(Box::new(error)))?;
        Ok(super::links::ConfiguredWorkbenchLinkDetailResponse {
            version: 1,
            kind: "vlajmaji-workbench-link-detail".into(),
            binding_sha256: binding,
            configuration,
            selection: self.selection.clone(),
            reference: reference.clone(),
            record,
            origin_resolution,
            target_observations,
        })
    }
}

fn validate_request(request: &WorkbenchLexicalRequest) -> Result<(), WorkbenchLexicalError> {
    if request.version != 1 {
        return Err(WorkbenchLexicalError::InvalidRequest {
            detail: format!("unsupported lexical request version {}", request.version),
        });
    }
    match &request.query {
        WorkbenchLexicalQuery::ReverseDefinition { contains } if contains.is_empty() => {
            return Err(WorkbenchLexicalError::InvalidRequest {
                detail: "reverse-definition contains must not be empty".into(),
            });
        }
        WorkbenchLexicalQuery::Links { depth, .. } if !matches!(depth, 1 | 2) => {
            return Err(WorkbenchLexicalError::InvalidRequest {
                detail: "Links depth must be exactly one or two".into(),
            });
        }
        _ => {}
    }
    Ok(())
}

fn validate_outer_binding(expected: &str, observed: &str) -> Result<(), WorkbenchQueryError> {
    if !is_sha256(expected) {
        return Err(WorkbenchQueryError::MalformedBinding {
            value: expected.to_owned(),
        });
    }
    if expected != observed {
        return Err(WorkbenchQueryError::StaleBinding {
            expected: expected.to_owned(),
            observed: observed.to_owned(),
        });
    }
    Ok(())
}

fn validate_form_detail_key(
    reference: &WorkbenchFormDetailRef,
    binding: &str,
) -> Result<(), WorkbenchLexicalError> {
    if reference.version != 1
        || reference.digest_policy != DETAIL_POLICY
        || reference.role != "form"
        || !is_sha256(&reference.entry_sha256)
        || !is_sha256(&reference.form_sha256)
        || reference
            .computation_sha256
            .as_deref()
            .is_some_and(|sha| !is_sha256(sha))
        || !is_sha256(&reference.binding_sha256)
    {
        return Err(WorkbenchLexicalError::MalformedFormDetail {
            detail: "unsupported key version, policy, role, or digest syntax".into(),
        });
    }
    if reference.binding_sha256 != binding {
        return Err(WorkbenchLexicalError::StaleFormDetail);
    }
    Ok(())
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

#[cfg(test)]
mod tests {
    use std::{
        borrow::Cow,
        fs,
        path::{Path, PathBuf},
        sync::atomic::{AtomicUsize, Ordering},
    };

    use super::*;
    use crate::workbench::WorkbenchView;

    static NEXT_FIXTURE: AtomicUsize = AtomicUsize::new(0);

    fn raw_sha256(bytes: &[u8]) -> String {
        Sha256::digest(bytes)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect()
    }

    fn test_root() -> PathBuf {
        for _ in 0..64 {
            let root = std::env::temp_dir().join(format!(
                "vlajmaji-lexical-hook-{}-{}",
                std::process::id(),
                NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed),
            ));
            match fs::create_dir(&root) {
                Ok(()) => return root,
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => panic!("create exclusive lexical test root: {error}"),
            }
        }
        panic!("could not allocate a unique lexical test root after 64 attempts")
    }

    fn test_dictionary() -> lexicon::Dictionary {
        let mut first_row = vec![b' '; 158];
        for (start, value) in [
            (1, "farvi"),
            (7, "far"),
            (11, "fav"),
            (15, "farv"),
            (20, "grow"),
            (41, "growth"),
            (62, "to grow"),
        ] {
            first_row[start..start + value.len()].copy_from_slice(value.as_bytes());
        }
        let mut second_row = vec![b' '; 158];
        for (start, value) in [
            (1, "blanu"),
            (7, "bla"),
            (11, "ban"),
            (15, "blan"),
            (20, "grow"),
            (41, "growth"),
            (62, "to grow"),
        ] {
            second_row[start..start + value.len()].copy_from_slice(value.as_bytes());
        }
        let row = [first_row.as_slice(), b"\n", second_row.as_slice()].concat();
        let empty = b"";
        let notice = b"lexical freshness fixture\n";
        let imported = lexicon::import_llg_lists(
            &row,
            empty,
            empty,
            notice,
            &lexicon::LlgImportOptions {
                expected_gismu_sha256: raw_sha256(&row),
                expected_cmavo_sha256: raw_sha256(empty),
                expected_rafsi_sha256: raw_sha256(empty),
                expected_notice_sha256: raw_sha256(notice),
                base_locator: "https://example.invalid/lexical-hook".into(),
            },
        )
        .unwrap();
        let mut document = imported.into_dictionary().into_document();
        let assertion = document.metadata.assertions[0].clone();
        document.metadata.forms.push(lexicon::FormRecord {
            id: EntityId::new("lexical-hook-form").unwrap(),
            entry: assertion.entry,
            spelling: "farvi".into(),
            language: lexicon::Presence::Value("jbo".into()),
            script: lexicon::Presence::Missing,
            pronunciation: lexicon::Presence::Missing,
            kind: FormKind::SourceVariant,
            standing: lexicon::Presence::Missing,
            components: Vec::new(),
            evidence: Some(lexicon::SourceEvidence {
                occurrence: assertion.occurrence,
                field_index: None,
                citation: lexicon::Presence::Missing,
                offset: None,
            }),
            computation: None,
        });
        lexicon::Dictionary::from_document(document).unwrap()
    }

    fn links_test_dictionary() -> lexicon::Dictionary {
        let mut document = test_dictionary().into_document();
        let first = document.metadata.assertions[0].clone();
        let second = document.metadata.assertions[1].clone();
        document.metadata.relations.push(lexicon::RelationRecord {
            id: EntityId::new("lexical-hook-relation").unwrap(),
            origin: lexicon::LocalRef::Assertion(first.id),
            kind: lexicon::RelationKind::SourceSeeAlso,
            direction: lexicon::Direction::Forward,
            scope: lexicon::RelationScope::Assertion,
            target: lexicon::RelationTarget::Local(lexicon::LocalRef::Entry(second.entry)),
            evidence: Some(lexicon::SourceEvidence {
                occurrence: first.occurrence,
                field_index: None,
                citation: lexicon::Presence::Missing,
                offset: None,
            }),
            editorial_revision: None,
        });
        lexicon::Dictionary::from_document(document).unwrap()
    }

    fn reverse_fixture_assertion_order(dictionary: &lexicon::Dictionary) -> Vec<EntityId> {
        let mut matches = dictionary
            .metadata()
            .assertions
            .iter()
            .filter_map(|assertion| {
                let lexicon::Eligibility::Lexical = assertion.eligibility else {
                    return None;
                };
                let lexicon::Presence::Value(definition) = &assertion.definition else {
                    return None;
                };
                if !definition.text.to_ascii_lowercase().contains("grow") {
                    return None;
                }
                let entry = dictionary
                    .metadata()
                    .entries
                    .iter()
                    .find(|entry| entry.id == assertion.entry)
                    .expect("validated input Assertion has its Entry");
                let lexicon::Presence::Value(spelling) = &entry.spelling else {
                    return None;
                };
                Some((
                    spelling.clone(),
                    assertion.occurrence.locator.clone(),
                    assertion.id.clone(),
                ))
            })
            .collect::<Vec<_>>();
        assert_eq!(matches.len(), 2);
        matches.sort();
        assert_ne!(matches[0].1, matches[1].1);
        matches.into_iter().map(|(_, _, id)| id).collect()
    }

    fn install_test_catalogue(root: &Path, dictionary: &lexicon::Dictionary) {
        let sealed = lexicon::seal_release(
            lexicon::PackId::Llg,
            lexicon::PackPayloadKind::Synthetic,
            dictionary,
        )
        .unwrap();
        let request = vlajmaji_store::CollectionRequest::new(
            vec![vlajmaji_store::CollectionInput::SealedBytes {
                pack_id: lexicon::PackId::Llg,
                release: Cow::Borrowed(&sealed.release),
                seal: Cow::Borrowed(&sealed.seal),
            }],
            vlajmaji_store::CollectionSelection::Packs(vec![lexicon::PackId::Llg]),
            true,
        )
        .unwrap();
        vlajmaji_store::CollectionCatalogue::install(root, request).unwrap();
    }

    fn test_selection() -> vlajmaji_store::WorkbenchSelection {
        vlajmaji_store::WorkbenchSelection {
            version: 1,
            packs: vec![lexicon::PackId::Llg],
            disabled: Vec::new(),
            owned: None,
            contexts: Vec::new(),
        }
    }

    fn direct_fixture() -> (PathBuf, PathBuf, PathBuf, Vec<EntityId>) {
        let root = test_root();
        let catalogue = root.join("catalogue");
        let dictionary = test_dictionary();
        let expected_assertions = reverse_fixture_assertion_order(&dictionary);
        install_test_catalogue(&catalogue, &dictionary);
        let view = root.join("view");
        WorkbenchView::build_new(
            &view,
            vlajmaji_store::WorkbenchBuildRequest {
                catalogue_root: Some(catalogue.clone()),
                owned_root: None,
                selection: test_selection(),
            },
        )
        .unwrap();
        (root, catalogue, view, expected_assertions)
    }

    fn configured_fixture() -> (PathBuf, PathBuf) {
        let root = test_root();
        let catalogue = root.join("catalogue");
        let dictionary = test_dictionary();
        install_test_catalogue(&catalogue, &dictionary);
        let config = root.join("profiles.json");
        let configured = vlajmaji_store::WorkbenchProfiles::configure(
            &config,
            vlajmaji_store::WorkbenchProfileConfigureRequest {
                name: "alpha".into(),
                catalogue_root: Some(catalogue.clone()),
                owned_root: None,
                selection: test_selection(),
                make_default: false,
            },
            None,
        )
        .unwrap();
        assert_eq!(configured.revision, 1);
        let refreshed = vlajmaji_store::WorkbenchProfiles::refresh(
            &config,
            Some("alpha"),
            root.join("prepared-alpha"),
            configured.revision,
        )
        .unwrap();
        assert_eq!(refreshed.publication.revision, 2);
        let selected = vlajmaji_store::WorkbenchProfiles::set_default(
            &config,
            Some("alpha"),
            refreshed.publication.revision,
        )
        .unwrap();
        assert!(selected.changed);
        assert_eq!(selected.revision, 3);
        (root, config)
    }

    fn form_request() -> WorkbenchLexicalRequest {
        WorkbenchLexicalRequest {
            version: 1,
            query: WorkbenchLexicalQuery::Form {
                value: "farvi".into(),
                role: None,
                kind: None,
            },
            options: WorkbenchLexicalOptions::default(),
        }
    }

    fn assert_catalogue_stale<T>(result: Result<T, WorkbenchQueryError>) {
        assert!(matches!(
            result,
            Err(WorkbenchQueryError::View(error))
                if matches!(error.as_ref(), vlajmaji_store::WorkbenchError::StaleDependencies {
                    dependency: vlajmaji_store::WorkbenchDependency::Catalogue,
                    ..
                })
        ));
    }

    fn rebuild_catalogue(root: &Path) -> u64 {
        let revision = vlajmaji_store::CollectionCatalogue::open(root)
            .unwrap()
            .local_revision()
            .unwrap();
        let report = vlajmaji_store::CollectionCatalogue::rebuild(root, revision).unwrap();
        assert!(report.ok);
        assert_eq!(report.local_revision, revision + 1);
        report.local_revision
    }

    fn change_default(config: &Path, profile: Option<&str>, revision: u64) -> u64 {
        let report =
            vlajmaji_store::WorkbenchProfiles::set_default(config, profile, revision).unwrap();
        assert!(report.changed);
        assert_eq!(report.revision, revision + 1);
        report.revision
    }

    fn assert_config_changed<T>(result: Result<T, ConfiguredWorkbenchQueryError>) {
        assert!(matches!(
            result,
            Err(ConfiguredWorkbenchQueryError::Profile(error))
                if matches!(error.as_ref(), vlajmaji_store::WorkbenchProfileError::ConfigChanged { .. })
        ));
    }

    fn links_request(depth: u8) -> WorkbenchLexicalRequest {
        WorkbenchLexicalRequest {
            version: 1,
            query: WorkbenchLexicalQuery::Links {
                spelling: "farvi".into(),
                depth,
            },
            options: WorkbenchLexicalOptions::default(),
        }
    }

    fn configured_links_fixture() -> (PathBuf, PathBuf, PathBuf) {
        let root = test_root();
        let catalogue = root.join("catalogue");
        let replacement_catalogue = root.join("replacement-catalogue");
        let dictionary = links_test_dictionary();
        install_test_catalogue(&catalogue, &dictionary);
        install_test_catalogue(&replacement_catalogue, &dictionary);
        let config = root.join("profiles.json");
        let configured = vlajmaji_store::WorkbenchProfiles::configure(
            &config,
            vlajmaji_store::WorkbenchProfileConfigureRequest {
                name: "alpha".into(),
                catalogue_root: Some(catalogue),
                owned_root: None,
                selection: test_selection(),
                make_default: false,
            },
            None,
        )
        .unwrap();
        assert_eq!(configured.revision, 1);
        let refreshed = vlajmaji_store::WorkbenchProfiles::refresh(
            &config,
            Some("alpha"),
            root.join("prepared-alpha"),
            configured.revision,
        )
        .unwrap();
        assert_eq!(refreshed.publication.revision, 2);
        let selected = vlajmaji_store::WorkbenchProfiles::set_default(
            &config,
            Some("alpha"),
            refreshed.publication.revision,
        )
        .unwrap();
        assert!(selected.changed);
        assert_eq!(selected.revision, 3);
        assert!(selected.active.is_some());
        (root, config, replacement_catalogue)
    }

    fn reverse_request() -> WorkbenchLexicalRequest {
        WorkbenchLexicalRequest {
            version: 1,
            query: WorkbenchLexicalQuery::ReverseDefinition {
                contains: "grow".into(),
            },
            options: WorkbenchLexicalOptions::default(),
        }
    }

    fn invalid_reverse_request() -> WorkbenchLexicalRequest {
        WorkbenchLexicalRequest {
            version: 1,
            query: WorkbenchLexicalQuery::ReverseDefinition {
                contains: String::new(),
            },
            options: WorkbenchLexicalOptions::default(),
        }
    }

    fn token_for(body: LexicalCursorBody) -> String {
        let checksum = typed_sha256(&body).unwrap();
        let bytes = serde_json::to_vec(&LexicalCursorEnvelope { body, checksum }).unwrap();
        let hex = bytes
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        format!("{CURSOR_PREFIX}{hex}")
    }

    #[test]
    fn test_direct_links_rechecks_after_catalogue_rebuild() {
        let root = test_root();
        let catalogue = root.join("catalogue");
        let mut cursor_dictionary = links_test_dictionary().into_document();
        let first = cursor_dictionary.metadata.assertions[0].clone();
        let second = cursor_dictionary.metadata.assertions[1].clone();
        let mut second_relation = cursor_dictionary.metadata.relations[0].clone();
        second_relation.id = EntityId::new("lexical-hook-relation-second").unwrap();
        second_relation.origin = lexicon::LocalRef::Assertion(second.id.clone());
        second_relation.target =
            lexicon::RelationTarget::Local(lexicon::LocalRef::Entry(first.entry.clone()));
        second_relation.evidence.as_mut().unwrap().occurrence = second.occurrence.clone();
        cursor_dictionary.metadata.relations.push(second_relation);
        let cursor_dictionary = lexicon::Dictionary::from_document(cursor_dictionary).unwrap();
        install_test_catalogue(&catalogue, &cursor_dictionary);
        let view = root.join("view");
        WorkbenchView::build_new(
            &view,
            vlajmaji_store::WorkbenchBuildRequest {
                catalogue_root: Some(catalogue.clone()),
                owned_root: None,
                selection: test_selection(),
            },
        )
        .unwrap();
        let mut engine = WorkbenchSearchEngine::open(&view).unwrap();
        let mut issued_request = links_request(2);
        issued_request.options.page_size = Some(1);
        let initial_compact = engine.execute_lexical_compact(&issued_request).unwrap();
        assert!(engine.full.is_none());
        let binding = initial_compact.binding_sha256.clone();
        let WorkbenchLexicalCompactResult::Links(initial_links) = initial_compact.result else {
            panic!("expected the fixed source relation in Compact Links output")
        };
        let resolved_sources = initial_links.resolved_sources.clone();
        let issued_token = initial_links
            .next_cursor
            .clone()
            .expect("the fixture has a second retained Links step");
        let issued_hex = issued_token.strip_prefix(CURSOR_PREFIX).unwrap();
        let (hex_pairs, remainder) = issued_hex.as_bytes().as_chunks::<2>();
        assert!(
            remainder.is_empty(),
            "issued cursor hex has an odd byte count"
        );
        let issued_bytes = hex_pairs
            .iter()
            .map(|pair| {
                let digit = |byte: u8| match byte {
                    b'0'..=b'9' => byte - b'0',
                    b'a'..=b'f' => byte - b'a' + 10,
                    _ => unreachable!("issued cursor has validated lowercase hex"),
                };
                (digit(pair[0]) << 4) | digit(pair[1])
            })
            .collect::<Vec<_>>();
        let issued_envelope: LexicalCursorEnvelope = serde_json::from_slice(&issued_bytes).unwrap();
        let mut positive_continuation = issued_request.clone();
        positive_continuation.options.page_size = Some(7);
        positive_continuation.options.cursor = Some(issued_token.clone());
        let WorkbenchLexicalCompactResult::Links(continued_links) = engine
            .execute_lexical_compact(&positive_continuation)
            .unwrap()
            .result
        else {
            panic!("issued Links cursor continued into another operation")
        };
        assert_eq!(
            continued_links
                .groups
                .iter()
                .flat_map(|group| &group.hits)
                .map(|step| match &step.identity.evidence {
                    super::super::links::WorkbenchLinkEvidenceId::TypedRelation { relation_id } =>
                        relation_id.as_str(),
                    super::super::links::WorkbenchLinkEvidenceId::SourceClause { .. } => {
                        "unexpected-clause"
                    }
                })
                .collect::<Vec<_>>(),
            vec!["lexical-hook-relation-second"]
        );
        assert!(continued_links.next_cursor.is_none());
        let WorkbenchLexicalPosition::Links { evidence, .. } = &issued_envelope.body.last else {
            panic!("actual Compact Links cursor has a non-Links last position")
        };
        let WorkbenchLexicalPosition::Links {
            source,
            depth,
            evidence_kind_rank,
            orientation_rank,
            from_assertion_id,
            ..
        } = &issued_envelope.body.last
        else {
            unreachable!()
        };
        let invalid_last = WorkbenchLexicalPosition::Links {
            source: source.clone(),
            depth: *depth,
            evidence_kind_rank: *evidence_kind_rank,
            evidence: match evidence {
                super::super::links::WorkbenchLinkEvidenceId::TypedRelation { .. } => {
                    super::super::links::WorkbenchLinkEvidenceId::TypedRelation {
                        relation_id: EntityId::new("lexical-hook-absent-relation").unwrap(),
                    }
                }
                super::super::links::WorkbenchLinkEvidenceId::SourceClause { .. } => {
                    unreachable!("the first page is a typed relation")
                }
            },
            orientation_rank: *orientation_rank,
            from_assertion_id: from_assertion_id.clone(),
        };
        let invalid_position_token = encode_cursor(
            &issued_request,
            &resolved_sources,
            "compact",
            &binding,
            invalid_last,
        )
        .unwrap();
        let mut invalid_position_request = issued_request.clone();
        invalid_position_request.options.cursor = Some(invalid_position_token);
        assert!(matches!(
            engine.execute_lexical_compact(&invalid_position_request),
            Err(WorkbenchQueryError::Lexical(error))
                if matches!(error.as_ref(), WorkbenchLexicalError::InvalidCursorPosition)
        ));
        let mut foreign_namespace_request = issued_request.clone();
        foreign_namespace_request.options.cursor = Some("dictionary-compact-v1:00".into());
        assert!(matches!(
            engine.execute_lexical_compact(&foreign_namespace_request),
            Err(WorkbenchQueryError::Lexical(error))
                if matches!(error.as_ref(), WorkbenchLexicalError::MalformedCursor { .. })
        ));
        let malformed_shape = token_for(LexicalCursorBody {
            version: 1,
            domain: CURSOR_DOMAIN.into(),
            format: "compact".into(),
            binding_sha256: binding.clone(),
            query_sha256: query_sha256(&issued_request, &resolved_sources).unwrap(),
            last: WorkbenchLexicalPosition::Links {
                source: resolved_sources[0].clone(),
                depth: 0,
                evidence_kind_rank: 0,
                evidence: super::super::links::WorkbenchLinkEvidenceId::TypedRelation {
                    relation_id: EntityId::new("lexical-hook-relation").unwrap(),
                },
                orientation_rank: 0,
                from_assertion_id: EntityId::new("lexical-hook-assertion").unwrap(),
            },
        });
        let mut malformed_shape_request = issued_request.clone();
        malformed_shape_request.options.cursor = Some(malformed_shape);
        assert!(matches!(
            engine.execute_lexical_compact(&malformed_shape_request),
            Err(WorkbenchQueryError::Lexical(error))
                if matches!(error.as_ref(), WorkbenchLexicalError::MalformedCursor { .. })
        ));
        let malformed_rank = token_for(LexicalCursorBody {
            version: 1,
            domain: CURSOR_DOMAIN.into(),
            format: "compact".into(),
            binding_sha256: binding.clone(),
            query_sha256: query_sha256(&issued_request, &resolved_sources).unwrap(),
            last: WorkbenchLexicalPosition::Links {
                source: resolved_sources[0].clone(),
                depth: 1,
                evidence_kind_rank: 1,
                evidence: super::super::links::WorkbenchLinkEvidenceId::TypedRelation {
                    relation_id: EntityId::new("lexical-hook-relation").unwrap(),
                },
                orientation_rank: 0,
                from_assertion_id: EntityId::new("lexical-hook-assertion").unwrap(),
            },
        });
        let mut malformed_rank_request = issued_request.clone();
        malformed_rank_request.options.cursor = Some(malformed_rank);
        assert!(matches!(
            engine.execute_lexical_compact(&malformed_rank_request),
            Err(WorkbenchQueryError::Lexical(error))
                if matches!(error.as_ref(), WorkbenchLexicalError::MalformedCursor { .. })
        ));
        let malformed_version = token_for(LexicalCursorBody {
            version: 2,
            domain: CURSOR_DOMAIN.into(),
            format: "compact".into(),
            binding_sha256: binding.clone(),
            query_sha256: query_sha256(&issued_request, &resolved_sources).unwrap(),
            last: issued_envelope.body.last.clone(),
        });
        let mut malformed_version_request = issued_request.clone();
        malformed_version_request.options.cursor = Some(malformed_version);
        assert!(matches!(
            engine.execute_lexical_compact(&malformed_version_request),
            Err(WorkbenchQueryError::Lexical(error))
                if matches!(error.as_ref(), WorkbenchLexicalError::MalformedCursor { .. })
        ));
        let detail_ref = initial_links.groups[0].hits[0].detail_ref.clone();
        let baseline_detail = engine.link_detail(&detail_ref, &binding).unwrap();
        assert!(engine.full.is_none());
        let saved_document = engine
            .compact
            .as_mut()
            .unwrap()
            .test_with_unverified_link_document(|document| {
                let saved = document.clone();
                // Deliberately unverified cache corruption for precedence testing only.
                let relation = document
                    .metadata
                    .relations
                    .iter()
                    .find(|row| row.id == EntityId::new("lexical-hook-relation").unwrap())
                    .unwrap();
                let source_evidence = relation.evidence.as_ref().unwrap();
                let capture = document
                    .captures
                    .iter_mut()
                    .find(|row| row.identity == source_evidence.occurrence.capture)
                    .unwrap();
                let position = capture
                    .occurrences
                    .iter()
                    .position(|row| row.locator == source_evidence.occurrence.locator)
                    .unwrap();
                capture.occurrences.remove(position);
                saved
            });
        let mut forged_secondary_key = detail_ref.clone();
        forged_secondary_key.record_sha256 = "f".repeat(64);
        assert!(matches!(
            engine.link_detail(&forged_secondary_key, &binding),
            Err(WorkbenchQueryError::Lexical(error))
                if matches!(error.as_ref(), WorkbenchLexicalError::Integrity { .. })
        ));
        engine
            .compact
            .as_mut()
            .unwrap()
            .test_with_unverified_link_document(|document| *document = saved_document);
        let restored_detail = engine.link_detail(&detail_ref, &binding).unwrap();
        assert_eq!(restored_detail.record, baseline_detail.record);
        assert!(engine.full.is_none());

        let request = links_request(3);
        assert!(matches!(
            engine.execute_lexical(&request),
            Err(WorkbenchQueryError::Lexical(error))
                if matches!(error.as_ref(), WorkbenchLexicalError::InvalidRequest { .. })
        ));

        let mut writer_revision = None;
        let result = engine.execute_lexical_with_hook(&request, || {
            writer_revision = Some(rebuild_catalogue(&catalogue));
        });
        assert_catalogue_stale(result);
        assert_eq!(writer_revision, Some(2));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn test_configured_compact_link_detail_rechecks_after_active_clear() {
        let (root, config, replacement_catalogue) = configured_links_fixture();
        let mut engine = ConfiguredWorkbenchSearchEngine::open(&config, Some("alpha")).unwrap();
        assert!(engine.full.is_none());
        let compact = engine.execute_lexical_compact(&links_request(1)).unwrap();
        assert!(engine.full.is_none());
        assert!(engine.compact.is_some());
        let WorkbenchLexicalCompactResult::Links(links) = &compact.result else {
            panic!("configured Links request returned another operation")
        };
        let step = links
            .groups
            .iter()
            .flat_map(|group| &group.hits)
            .next()
            .expect("the retained source relation must produce a Compact link step");
        let detail_ref = step.detail_ref.clone();
        let binding = compact.binding_sha256.clone();
        let baseline = engine.link_detail(&detail_ref, &binding).unwrap();
        assert_eq!(baseline.record.identity, detail_ref.identity);
        assert!(engine.full.is_none());

        let mut writer_observation = None;
        let result = engine.link_detail_with_hook(&detail_ref, &binding, || {
            let report = vlajmaji_store::WorkbenchProfiles::configure(
                &config,
                vlajmaji_store::WorkbenchProfileConfigureRequest {
                    name: "alpha".into(),
                    catalogue_root: Some(replacement_catalogue.clone()),
                    owned_root: None,
                    selection: test_selection(),
                    make_default: false,
                },
                Some(3),
            )
            .unwrap();
            assert!(report.changed);
            let profile = vlajmaji_store::WorkbenchProfiles::read(&config)
                .unwrap()
                .profile(Some("alpha"))
                .unwrap()
                .clone();
            writer_observation = Some((
                report.revision,
                report.active.is_none(),
                profile.active_generation,
            ));
        });
        assert_config_changed(result);
        assert_eq!(writer_observation, Some((4, true, None)));
        assert!(engine.full.is_none());
        drop(engine);

        let refreshed = vlajmaji_store::WorkbenchProfiles::refresh(
            &config,
            Some("alpha"),
            root.join("configured-links-after-clear"),
            4,
        )
        .unwrap();
        assert_eq!(refreshed.publication.revision, 5);
        let mut current = ConfiguredWorkbenchSearchEngine::open(&config, Some("alpha")).unwrap();
        let current_compact = current.execute_lexical_compact(&links_request(1)).unwrap();
        let WorkbenchLexicalCompactResult::Links(current_links) = &current_compact.result else {
            panic!("refreshed configured Links request returned another operation")
        };
        let current_ref = current_links
            .groups
            .iter()
            .flat_map(|group| &group.hits)
            .next()
            .unwrap()
            .detail_ref
            .clone();
        assert!(matches!(
            current.link_detail(&detail_ref, &current_compact.binding_sha256),
            Err(ConfiguredWorkbenchQueryError::Query(error))
                if matches!(error.as_ref(), WorkbenchQueryError::Lexical(inner)
                    if matches!(inner.as_ref(), WorkbenchLexicalError::StaleLinkDetail))
        ));
        assert_eq!(
            current
                .link_detail(&current_ref, &current_compact.binding_sha256)
                .unwrap()
                .reference,
            current_ref
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn test_required_nullable_request_fields_reject_missing_duplicate_and_unknown_keys() {
        let valid = r#"{"version":1,"query":{"operation":"form","value":"x","role":null,"kind":null},"options":{"sources":[],"alias_scope":"ActiveOnly","page_size":null,"cursor":null}}"#;
        let parsed: WorkbenchLexicalRequest = serde_json::from_str(valid).unwrap();
        assert_eq!(parsed.version, 1);

        for invalid in [
            r#"{"version":1,"query":{"operation":"form","value":"x","role":null},"options":{"sources":[],"alias_scope":"ActiveOnly","page_size":null,"cursor":null}}"#,
            r#"{"version":1,"query":{"operation":"form","value":"x","role":null,"kind":null},"options":{"sources":[],"alias_scope":"ActiveOnly","page_size":null}}"#,
            r#"{"version":1,"version":1,"query":{"operation":"form","value":"x","role":null,"kind":null},"options":{"sources":[],"alias_scope":"ActiveOnly","page_size":null,"cursor":null}}"#,
            r#"{"version":1,"query":{"operation":"form","value":"x","role":null,"kind":null,"extra":true},"options":{"sources":[],"alias_scope":"ActiveOnly","page_size":null,"cursor":null}}"#,
        ] {
            assert!(serde_json::from_str::<WorkbenchLexicalRequest>(invalid).is_err());
        }
    }

    #[test]
    fn test_cursor_matches_fixed_typed_protocol_oracle() {
        let mut request = WorkbenchLexicalRequest {
            version: 1,
            query: WorkbenchLexicalQuery::Form {
                value: "farvi-form".into(),
                role: None,
                kind: None,
            },
            options: WorkbenchLexicalOptions {
                sources: vec!["LU".into(), "LO".into()],
                ..WorkbenchLexicalOptions::default()
            },
        };
        let resolved = vec![
            SourceId::new("lexical-owned").unwrap(),
            SourceId::new("lexical-upstream").unwrap(),
        ];
        let expected_query = r#"{"query":{"operation":"form","value":"farvi-form","role":null,"kind":null},"selectors":["LU","LO"],"resolved_sources":["lexical-owned","lexical-upstream"],"alias_scope":"ActiveOnly"}"#;
        let binding = LexicalQueryBinding {
            query: &request.query,
            selectors: &request.options.sources,
            resolved_sources: &resolved,
            alias_scope: request.options.alias_scope,
        };
        assert_eq!(serde_json::to_string(&binding).unwrap(), expected_query);
        assert_eq!(
            query_sha256(&request, &resolved).unwrap(),
            "670ff975f8187867a8901d6b547472cef8d49363470fdcc70538a1cc03c550dd"
        );

        request.options.sources = vec!["LO".into(), "LU".into()];
        assert_eq!(
            query_sha256(&request, &resolved).unwrap(),
            "426379cde0fe8cf91ef19bbb56fd7266f755c1db1f28be3830878226f1ae25d7"
        );
        request.options.sources = vec!["LU".into(), "LO".into()];
        let position = WorkbenchLexicalPosition::Form {
            source: resolved[0].clone(),
            entry_spelling: "farvi".into(),
            spelling: "farvi-form".into(),
            kind: FormKind::ComputedStem,
            form_id: EntityId::new("lexical-owned-form-stem").unwrap(),
        };
        let body = LexicalCursorBody {
            version: 1,
            domain: CURSOR_DOMAIN.into(),
            format: "compact".into(),
            binding_sha256: "1".repeat(64),
            query_sha256: "670ff975f8187867a8901d6b547472cef8d49363470fdcc70538a1cc03c550dd".into(),
            last: position.clone(),
        };
        let expected_body = r#"{"version":1,"domain":"workbench-lexical-cursor-v1","format":"compact","binding_sha256":"1111111111111111111111111111111111111111111111111111111111111111","query_sha256":"670ff975f8187867a8901d6b547472cef8d49363470fdcc70538a1cc03c550dd","last":{"operation":"form","source":"lexical-owned","entry_spelling":"farvi","spelling":"farvi-form","kind":"ComputedStem","form_id":"lexical-owned-form-stem"}}"#;
        assert_eq!(serde_json::to_string(&body).unwrap(), expected_body);
        let checksum = typed_sha256(&body).unwrap();
        assert_eq!(
            checksum,
            "3b3f73718cdefcf6d7063d50bc1f69140152b865a867e1289a576c4ad9f206f2"
        );
        let envelope = LexicalCursorEnvelope { body, checksum };
        let expected_envelope = r#"{"body":{"version":1,"domain":"workbench-lexical-cursor-v1","format":"compact","binding_sha256":"1111111111111111111111111111111111111111111111111111111111111111","query_sha256":"670ff975f8187867a8901d6b547472cef8d49363470fdcc70538a1cc03c550dd","last":{"operation":"form","source":"lexical-owned","entry_spelling":"farvi","spelling":"farvi-form","kind":"ComputedStem","form_id":"lexical-owned-form-stem"}},"checksum":"3b3f73718cdefcf6d7063d50bc1f69140152b865a867e1289a576c4ad9f206f2"}"#;
        assert_eq!(serde_json::to_string(&envelope).unwrap(), expected_envelope);
        let expected_token = format!(
            "{CURSOR_PREFIX}{}",
            serde_json::to_vec(&envelope)
                .unwrap()
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>()
        );
        assert_eq!(
            encode_cursor(
                &request,
                &resolved,
                "compact",
                &"1".repeat(64),
                position.clone()
            )
            .unwrap(),
            expected_token
        );
        request.options.page_size = Some(7);
        assert_eq!(
            decode_cursor(
                &expected_token,
                &request,
                &resolved,
                "compact",
                &"1".repeat(64)
            )
            .unwrap(),
            position
        );
    }

    #[test]
    fn test_links_cursor_matches_fixed_typed_protocol_oracle() {
        let mut request = WorkbenchLexicalRequest {
            version: 1,
            query: WorkbenchLexicalQuery::Links {
                spelling: "seed".into(),
                depth: 2,
            },
            options: WorkbenchLexicalOptions {
                sources: vec!["KU".into(), "KO".into()],
                ..WorkbenchLexicalOptions::default()
            },
        };
        let resolved = vec![
            SourceId::new("links-owned").unwrap(),
            SourceId::new("links-upstream").unwrap(),
        ];
        let expected_query = r#"{"query":{"operation":"links","spelling":"seed","depth":2},"selectors":["KU","KO"],"resolved_sources":["links-owned","links-upstream"],"alias_scope":"ActiveOnly"}"#;
        let binding = LexicalQueryBinding {
            query: &request.query,
            selectors: &request.options.sources,
            resolved_sources: &resolved,
            alias_scope: request.options.alias_scope,
        };
        assert_eq!(serde_json::to_string(&binding).unwrap(), expected_query);
        assert_eq!(
            query_sha256(&request, &resolved).unwrap(),
            "fb8da7420bfbcdf9f095b22f5ed64662f988a0fc93c4c70e7f0572ceed2b2bd8"
        );
        request.options.sources = vec!["KO".into(), "KU".into()];
        assert_eq!(
            query_sha256(&request, &resolved).unwrap(),
            "82157d8d1149057f1d403d7bd44cc7321d07ea29e5b4e64460fd1a12ba2f3e8e"
        );
        request.options.sources = vec!["KU".into(), "KO".into()];

        let position = WorkbenchLexicalPosition::Links {
            source: resolved[0].clone(),
            depth: 1,
            evidence_kind_rank: 0,
            evidence: super::super::links::WorkbenchLinkEvidenceId::TypedRelation {
                relation_id: EntityId::new("links-relation-03").unwrap(),
            },
            orientation_rank: 0,
            from_assertion_id: EntityId::new("links-owned-a").unwrap(),
        };
        let body = LexicalCursorBody {
            version: 1,
            domain: CURSOR_DOMAIN.into(),
            format: "compact".into(),
            binding_sha256: "1".repeat(64),
            query_sha256: "fb8da7420bfbcdf9f095b22f5ed64662f988a0fc93c4c70e7f0572ceed2b2bd8".into(),
            last: position.clone(),
        };
        let expected_body = r#"{"version":1,"domain":"workbench-lexical-cursor-v1","format":"compact","binding_sha256":"1111111111111111111111111111111111111111111111111111111111111111","query_sha256":"fb8da7420bfbcdf9f095b22f5ed64662f988a0fc93c4c70e7f0572ceed2b2bd8","last":{"operation":"links","source":"links-owned","depth":1,"evidence_kind_rank":0,"evidence":{"kind":"typed_relation","relation_id":"links-relation-03"},"orientation_rank":0,"from_assertion_id":"links-owned-a"}}"#;
        assert_eq!(serde_json::to_string(&body).unwrap(), expected_body);
        let checksum = typed_sha256(&body).unwrap();
        assert_eq!(
            checksum,
            "449721121f681e73942061578e74cecbd297ea120f89268660d691271341cccd"
        );
        let envelope = LexicalCursorEnvelope { body, checksum };
        let expected_envelope = r#"{"body":{"version":1,"domain":"workbench-lexical-cursor-v1","format":"compact","binding_sha256":"1111111111111111111111111111111111111111111111111111111111111111","query_sha256":"fb8da7420bfbcdf9f095b22f5ed64662f988a0fc93c4c70e7f0572ceed2b2bd8","last":{"operation":"links","source":"links-owned","depth":1,"evidence_kind_rank":0,"evidence":{"kind":"typed_relation","relation_id":"links-relation-03"},"orientation_rank":0,"from_assertion_id":"links-owned-a"}},"checksum":"449721121f681e73942061578e74cecbd297ea120f89268660d691271341cccd"}"#;
        assert_eq!(serde_json::to_string(&envelope).unwrap(), expected_envelope);
        let expected_token = format!(
            "{CURSOR_PREFIX}{}",
            serde_json::to_vec(&envelope)
                .unwrap()
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>()
        );
        assert_eq!(
            encode_cursor(
                &request,
                &resolved,
                "compact",
                &"1".repeat(64),
                position.clone()
            )
            .unwrap(),
            expected_token
        );
        request.options.page_size = Some(5);
        assert_eq!(
            decode_cursor(
                &expected_token,
                &request,
                &resolved,
                "compact",
                &"1".repeat(64)
            )
            .unwrap(),
            position
        );
    }

    #[test]
    fn test_cursor_excludes_page_size_and_checks_malformed_identity_before_staleness() {
        let source = SourceId::new("lensisku").unwrap();
        let assertion_id = EntityId::new("assertion-a").unwrap();
        let position = WorkbenchLexicalPosition::ReverseDefinition {
            source: source.clone(),
            spelling: "farvi".into(),
            occurrence_id: "0011".into(),
            assertion_id,
        };
        let mut first = reverse_request();
        first.options.page_size = Some(1);
        let token = encode_cursor(
            &first,
            std::slice::from_ref(&source),
            "full",
            &"a".repeat(64),
            position.clone(),
        )
        .unwrap();

        let mut next_page = first.clone();
        next_page.options.page_size = Some(25);
        assert_eq!(
            decode_cursor(
                &token,
                &next_page,
                std::slice::from_ref(&source),
                "full",
                &"a".repeat(64)
            )
            .unwrap(),
            position
        );
        assert!(matches!(
            decode_cursor(
                &token,
                &next_page,
                std::slice::from_ref(&source),
                "compact",
                &"a".repeat(64)
            ),
            Err(WorkbenchLexicalError::StaleCursor)
        ));

        let malformed = token_for(LexicalCursorBody {
            version: 1,
            domain: CURSOR_DOMAIN.into(),
            format: "full".into(),
            binding_sha256: "not-a-digest".into(),
            query_sha256: "b".repeat(64),
            last: position,
        });
        assert!(matches!(
            decode_cursor(
                &malformed,
                &next_page,
                std::slice::from_ref(&source),
                "full",
                &"c".repeat(64)
            ),
            Err(WorkbenchLexicalError::MalformedCursor { .. })
        ));
    }

    #[test]
    fn test_form_detail_key_syntax_precedes_binding_staleness() {
        let reference = WorkbenchFormDetailRef {
            version: 1,
            digest_policy: DETAIL_POLICY.into(),
            role: "form".into(),
            binding_sha256: "bad".into(),
            source: SourceId::new("lensisku").unwrap(),
            entry_id: EntityId::new("entry-a").unwrap(),
            form_id: EntityId::new("form-a").unwrap(),
            entry_sha256: "d".repeat(64),
            form_sha256: "e".repeat(64),
            computation_sha256: None,
        };
        assert!(matches!(
            validate_form_detail_key(&reference, &"f".repeat(64)),
            Err(WorkbenchLexicalError::MalformedFormDetail { .. })
        ));

        let mut stale = reference;
        stale.binding_sha256 = "a".repeat(64);
        assert!(matches!(
            validate_form_detail_key(&stale, &"b".repeat(64)),
            Err(WorkbenchLexicalError::StaleFormDetail)
        ));
    }

    #[test]
    fn test_direct_lexical_wrappers_recheck_after_catalogue_writer() {
        let (root, catalogue, view, expected_assertions) = direct_fixture();
        let mut engine = WorkbenchSearchEngine::open(&view).unwrap();
        assert!(matches!(
            engine.execute_lexical(&invalid_reverse_request()),
            Err(WorkbenchQueryError::Lexical(error))
                if matches!(error.as_ref(), WorkbenchLexicalError::InvalidRequest { .. })
        ));
        let mut paged_request = reverse_request();
        paged_request.options.page_size = Some(1);
        let page = engine.execute_lexical(&paged_request).unwrap();
        let WorkbenchLexicalFullResult::ReverseDefinition(page_rows) = page.result else {
            panic!("direct cursor precondition returned another operation")
        };
        assert_eq!(page_rows.totals.records, 2);
        assert_eq!(page_rows.groups.len(), 1);
        assert_eq!(page_rows.groups[0].hits.len(), 1);
        assert_eq!(
            page_rows.groups[0].hits[0].assertion.id,
            expected_assertions[0]
        );
        let issued_cursor = page_rows
            .next_cursor
            .clone()
            .expect("first page must issue a continuation cursor");
        let source = page_rows.resolved_sources[0].clone();
        let mut continuation_request = paged_request.clone();
        continuation_request.options.cursor = Some(issued_cursor.clone());
        let continuation = engine
            .execute_lexical(&continuation_request)
            .expect("issued cursor must continue on the same view");
        let WorkbenchLexicalFullResult::ReverseDefinition(continuation_rows) = continuation.result
        else {
            panic!("direct cursor continuation returned another operation")
        };
        assert_eq!(continuation_rows.totals.records, 2);
        assert_eq!(continuation_rows.groups.len(), 1);
        assert_eq!(continuation_rows.groups[0].hits.len(), 1);
        assert_eq!(
            continuation_rows.groups[0].hits[0].assertion.id,
            expected_assertions[1]
        );
        assert!(continuation_rows.next_cursor.is_none());

        let encoded = issued_cursor
            .strip_prefix(CURSOR_PREFIX)
            .expect("issued cursor uses the current lexical namespace");
        let bytes = encoded
            .as_bytes()
            .as_chunks::<2>()
            .0
            .iter()
            .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
            .collect::<Vec<_>>();
        let mut envelope: LexicalCursorEnvelope = serde_json::from_slice(&bytes).unwrap();
        let missing_position = WorkbenchLexicalPosition::ReverseDefinition {
            source: source.clone(),
            spelling: "not-present-in-candidates".into(),
            occurrence_id: "00".into(),
            assertion_id: EntityId::new("missing-assertion").unwrap(),
        };
        envelope.body.last = missing_position;
        let valid_wire_wrong_position = token_for(envelope.body);
        let mut wrong_position_request = paged_request.clone();
        wrong_position_request.options.cursor = Some(valid_wire_wrong_position.clone());
        assert!(matches!(
            engine.execute_lexical(&wrong_position_request),
            Err(WorkbenchQueryError::Lexical(error))
                if matches!(error.as_ref(), WorkbenchLexicalError::InvalidCursorPosition)
        ));
        assert!(matches!(
            decode_cursor(
                &format!(
                    "dictionary-compact-v1:{}",
                    valid_wire_wrong_position
                        .split_once(':')
                        .expect("issued cursor has a namespace delimiter")
                        .1
                ),
                &paged_request,
                &page_rows.resolved_sources,
                "full",
                &page.binding_sha256,
            ),
            Err(WorkbenchLexicalError::MalformedCursor { .. })
        ));
        let mut writer_revision = None;
        let result = engine.execute_lexical_with_hook(&invalid_reverse_request(), || {
            writer_revision = Some(rebuild_catalogue(&catalogue));
        });
        assert_catalogue_stale(result);
        assert_eq!(writer_revision, Some(2));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn test_configured_lexical_wrappers_recheck_after_config_writer() {
        let (root, config) = configured_fixture();
        let mut engine = ConfiguredWorkbenchSearchEngine::open(&config, Some("alpha")).unwrap();
        assert!(engine.full.is_none());
        let compact = engine.execute_lexical_compact(&form_request()).unwrap();
        assert!(engine.full.is_none());
        assert!(engine.compact.is_some());
        let WorkbenchLexicalCompactResult::Form(forms) = &compact.result else {
            panic!("configured stored-Form request returned another operation")
        };
        let detail_ref = forms.groups[0].hits[0].detail_ref.clone();
        let binding = compact.binding_sha256.clone();
        let baseline = engine.form_detail(&detail_ref, &binding).unwrap();
        assert_eq!(baseline.form.form.id, forms.groups[0].hits[0].form_id);
        assert!(engine.full.is_none());
        let mut writer_revision = None;
        let result = engine.form_detail_with_hook(&detail_ref, &binding, || {
            writer_revision = Some(change_default(&config, None, 3));
        });
        assert_config_changed(result);
        assert_eq!(writer_revision, Some(4));
        assert!(engine.full.is_none());

        let mut current = ConfiguredWorkbenchSearchEngine::open(&config, Some("alpha")).unwrap();
        assert!(current.full.is_none());
        let current_compact = current.execute_lexical_compact(&form_request()).unwrap();
        let current_binding = current_compact.binding_sha256.clone();
        assert_ne!(current_binding, binding);
        assert!(matches!(
            current.form_detail(&detail_ref, &current_binding),
            Err(ConfiguredWorkbenchQueryError::Query(error))
                if matches!(error.as_ref(), WorkbenchQueryError::Lexical(error)
                if matches!(error.as_ref(), WorkbenchLexicalError::StaleFormDetail)
                )
        ));
        let WorkbenchLexicalCompactResult::Form(current_forms) = &current_compact.result else {
            panic!("current configured stored-Form request returned another operation")
        };
        let current_ref = current_forms.groups[0].hits[0].detail_ref.clone();
        let current_detail = current.form_detail(&current_ref, &current_binding).unwrap();
        assert_eq!(
            current_detail.form.form.id,
            current_forms.groups[0].hits[0].form_id
        );
        assert!(current.full.is_none());
        fs::remove_dir_all(root).unwrap();
    }
}
