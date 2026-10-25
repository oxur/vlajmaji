//! Versioned source-preserving dictionary data. This module is an explicit
//! normalized projection over immutable captures; it does not infer linguistic
//! structure or rewrite legacy archives. Presence is explicit throughout:
//! `Missing`, source-level `Null`, and present values remain distinct.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use serde::{Deserialize, Deserializer, Serialize};
use serde_json::value::RawValue;

use super::{
    ImportError, Lexicon, RecordIdentity, SnapshotIdentity, SourceFormat, SourceRegistry,
    TimePrecision,
};

/// A typed failure while constructing or validating a versioned dictionary.
///
/// The variants distinguish malformed wire data, identity collisions, dangling
/// references, unavailable payloads, accounting, provenance, and owned history.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DictionaryError {
    /// An identifier does not satisfy the exact model identifier grammar.
    InvalidId(String),
    /// A retained native or extension value is not one valid JSON value.
    InvalidRawJson(String),
    /// A document violates a structural invariant without a more specific category.
    InvalidDocument(String),
    /// A versioned input uses a document version that this implementation cannot read.
    UnsupportedVersion(u32),
    /// JSON syntax, UTF-8, duplicate-key, or typed deserialization failed.
    InvalidJson(String),
    /// The preserved legacy archive could not be parsed or validated.
    InvalidLegacy(ImportError),
    /// The `CaptureHashMismatch ` case.
    CaptureHashMismatch {
        /// Capture namespace whose payload disagrees with its identity.
        namespace: String,
        /// Digest asserted by the capture reference.
        expected: String,
        /// Digest computed from the preserved bytes.
        actual: String,
    },
    /// The document repeats a capture identity.
    DuplicateCapture(String),
    /// One capture repeats an exact source-native occurrence locator.
    DuplicateOccurrence(String),
    /// Two metadata rows claim the same typed model identity.
    DuplicateEntity(String),
    /// A typed relationship names an absent entity or an entity of the wrong role.
    MissingReference(String),
    /// Availability status conflicts with the known capture payload state.
    InvalidAvailability(String),
    /// Release counts do not agree with known input and normalized membership.
    InvalidAccounting(String),
    /// Source evidence and computed/editorial provenance are inconsistent.
    InvalidProvenance(String),
    /// An owned revision chain, head, or tombstone is inconsistent.
    InvalidHistory(String),
}
impl fmt::Display for DictionaryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for DictionaryError {}
impl From<ImportError> for DictionaryError {
    fn from(e: ImportError) -> Self {
        Self::InvalidLegacy(e)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
/// Versioned dictionary data type `EntityId(String);`; see the crate documentation for preservation rules.
pub struct EntityId(String);
impl EntityId {
    /// Construct an identifier using the exact registry-compatible ASCII grammar.
    ///
    /// The accepted value is retained byte-for-byte; this does not trim or
    /// normalize it. Digit-first identifiers are valid.
    ///
    /// # Errors
    ///
    /// Returns [`DictionaryError::InvalidId`] for an empty value or any byte
    /// outside lowercase ASCII letters, digits, `-`, `_`, `.`, and `:`.
    pub fn new(value: impl Into<String>) -> Result<Self, DictionaryError> {
        let value = value.into();
        if value.is_empty()
            || !value.bytes().all(|b| {
                b.is_ascii_lowercase()
                    || b.is_ascii_digit()
                    || b == b'-'
                    || b == b'_'
                    || b == b'.'
                    || b == b':'
            })
            || !(value.as_bytes()[0].is_ascii_lowercase() || value.as_bytes()[0].is_ascii_digit())
        {
            return Err(DictionaryError::InvalidId(value));
        }
        Ok(Self(value))
    }
    /// Borrow the exact validated identifier string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl fmt::Display for EntityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
impl TryFrom<String> for EntityId {
    type Error = DictionaryError;
    fn try_from(v: String) -> Result<Self, Self::Error> {
        Self::new(v)
    }
}
impl<'de> Deserialize<'de> for EntityId {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Self::new(String::deserialize(d)?).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
/// Versioned dictionary state vocabulary `Presence`; see the crate documentation for preservation rules.
pub enum Presence<T> {
    /// The `Missing` case.
    #[default]
    Missing,
    /// The `Null` case.
    Null,
    /// The `Value` case.
    Value(T),
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Versioned dictionary data type `TextValue`; see the crate documentation for preservation rules.
pub struct TextValue {
    /// Value retained for `text` without implicit normalization.
    pub text: String,
    /// Language value retained with its original presence state.
    pub language: Presence<String>,
    /// Value retained for `markup` without implicit normalization.
    pub markup: Presence<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
/// Versioned dictionary data type `RawJson(String);`; see the crate documentation for preservation rules.
pub struct RawJson(String);
impl RawJson {
    /// Validate and retain exactly one JSON value as its original lexical text.
    ///
    /// This preserves number precision, object key order, and string spelling;
    /// callers should use it for opaque native or extension values.
    ///
    /// # Errors
    ///
    /// Returns [`DictionaryError::InvalidRawJson`] when the string is not valid
    /// JSON syntax for one value.
    pub fn new(value: impl Into<String>) -> Result<Self, DictionaryError> {
        let value = value.into();
        RawValue::from_string(value.clone())
            .map_err(|e| DictionaryError::InvalidRawJson(e.to_string()))?;
        Ok(Self(value))
    }
    /// Borrow the exact retained JSON lexical representation.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl<'de> Deserialize<'de> for RawJson {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Self::new(String::deserialize(d)?).map_err(serde::de::Error::custom)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Versioned dictionary data type `CaptureRef`; see the crate documentation for preservation rules.
pub struct CaptureRef {
    /// Value retained for `namespace` without implicit normalization.
    pub namespace: String,
    /// Value retained for `sha256` without implicit normalization.
    pub sha256: String,
}
impl From<&SnapshotIdentity> for CaptureRef {
    fn from(v: &SnapshotIdentity) -> Self {
        Self {
            namespace: v.namespace().into(),
            sha256: v.sha256().into(),
        }
    }
}
impl CaptureRef {
    /// Convert to the legacy snapshot identity without changing namespace or digest.
    pub fn to_snapshot_identity(&self) -> Result<SnapshotIdentity, DictionaryError> {
        Ok(SnapshotIdentity {
            namespace: self.namespace.clone(),
            sha256: self.sha256.clone(),
        })
    }
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Versioned dictionary data type `OccurrenceRef`; see the crate documentation for preservation rules.
pub struct OccurrenceRef {
    /// Immutable source capture identified by namespace and content digest.
    pub capture: CaptureRef,
    /// Source-native row locator within the capture.
    pub locator: String,
}
impl From<&RecordIdentity> for OccurrenceRef {
    fn from(v: &RecordIdentity) -> Self {
        Self {
            capture: CaptureRef::from(v.snapshot()),
            locator: v.locator().into(),
        }
    }
}
impl OccurrenceRef {
    /// Convert to the legacy record identity without changing its exact locator.
    pub fn to_record_identity(&self) -> Result<RecordIdentity, DictionaryError> {
        Ok(RecordIdentity {
            snapshot: self.capture.to_snapshot_identity()?,
            locator: self.locator.clone(),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Versioned dictionary state vocabulary `NativeEncoding`; see the crate documentation for preservation rules.
pub enum NativeEncoding {
    /// The `Json` case.
    Json,
    /// The `Text` case.
    Text,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Versioned dictionary data type `NativeField`; see the crate documentation for preservation rules.
pub struct NativeField {
    /// Value retained for `name` without implicit normalization.
    pub name: String,
    /// Value retained for `encoding` without implicit normalization.
    pub encoding: NativeEncoding,
    /// Unmodified lexical representation of the native field value.
    pub raw_value: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Versioned dictionary data type `RawOccurrence`; see the crate documentation for preservation rules.
pub struct RawOccurrence {
    /// Source-native row locator within the capture.
    pub locator: String,
    /// Ordered native fields as parsed from the original row.
    pub fields: Vec<NativeField>,
    /// Exact source line when the format has line-oriented records.
    pub raw_line: Option<String>,
}
impl RawOccurrence {
    /// Return all native fields with this exact name, preserving repeated fields.
    pub fn fields_named(&self, name: &str) -> Vec<&NativeField> {
        self.fields.iter().filter(|f| f.name == name).collect()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Versioned dictionary data type `CaptureMetadata`; see the crate documentation for preservation rules.
pub struct CaptureMetadata {
    /// Value retained for `source_name` without implicit normalization.
    pub source_name: String,
    /// Value retained for `format_label` without implicit normalization.
    pub format_label: String,
    /// Source-native row locator within the capture.
    pub locator: String,
    /// Value retained for `revision` without implicit normalization.
    pub revision: Presence<String>,
    /// Value retained for `captured_at` without implicit normalization.
    pub captured_at: Presence<String>,
    /// Value retained for `time_precision` without implicit normalization.
    pub time_precision: TimePrecision,
    /// Value retained for `notice_locator` without implicit normalization.
    pub notice_locator: Presence<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Versioned dictionary data type `RawCapture`; see the crate documentation for preservation rules.
pub struct RawCapture {
    /// Capture identity derived from its namespace and full-byte digest.
    pub identity: CaptureRef,
    /// Capture-time source description and temporal precision.
    pub metadata: CaptureMetadata,
    /// Original captured source bytes; never normalized in place.
    pub bytes: Vec<u8>,
    /// Value retained for `occurrences` without implicit normalization.
    pub occurrences: Vec<RawOccurrence>,
}
impl RawCapture {
    /// Construct a raw-capture draft and derive its SHA-256 identity.
    ///
    /// This convenience constructor checks no registry binding, locator
    /// uniqueness, normalized assertion, or cross-record reference. Pass the
    /// resulting draft through [`Dictionary::from_document`] or another
    /// validating constructor before treating it as trusted state.
    pub fn new(
        namespace: impl Into<String>,
        metadata: CaptureMetadata,
        bytes: Vec<u8>,
        occurrences: Vec<RawOccurrence>,
    ) -> Self {
        let sha256 = super::sha256(&bytes);
        Self {
            identity: CaptureRef {
                namespace: namespace.into(),
                sha256,
            },
            metadata,
            bytes,
            occurrences,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Versioned dictionary data type `TextRange`; see the crate documentation for preservation rules.
pub struct TextRange {
    /// Value retained for `start` without implicit normalization.
    pub start: usize,
    /// Value retained for `end` without implicit normalization.
    pub end: usize,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Versioned dictionary data type `SourceEvidence`; see the crate documentation for preservation rules.
pub struct SourceEvidence {
    /// Exact captured source occurrence supporting this row.
    pub occurrence: OccurrenceRef,
    /// Value retained for `field_index` without implicit normalization.
    pub field_index: Option<usize>,
    /// Value retained for `citation` without implicit normalization.
    pub citation: Presence<String>,
    /// Value retained for `offset` without implicit normalization.
    pub offset: Option<TextRange>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Versioned dictionary data type `CollectionRecord`; see the crate documentation for preservation rules.
pub struct CollectionRecord {
    /// Stable model identifier for this row.
    pub id: EntityId,
    /// The registry source that owns or published this data.
    pub source: super::SourceId,
    /// Value retained for `display_name` without implicit normalization.
    pub display_name: String,
    /// Value retained for `description` without implicit normalization.
    pub description: Presence<TextValue>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Versioned dictionary data type `ComponentRecord`; see the crate documentation for preservation rules.
pub struct ComponentRecord {
    /// Stable model identifier for this row.
    pub id: EntityId,
    /// Value retained for `collection` without implicit normalization.
    pub collection: EntityId,
    /// Value retained for `native_name` without implicit normalization.
    pub native_name: String,
    /// Value retained for `languages` without implicit normalization.
    pub languages: Vec<String>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Versioned dictionary state vocabulary `Availability`; see the crate documentation for preservation rules.
pub enum Availability {
    /// The `Available` case.
    Available,
    /// The `Disabled` case.
    Disabled,
    /// The `Missing` case.
    Missing,
    /// The `Removed` case.
    Removed,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Versioned dictionary data type `ReleaseAccounting`; see the crate documentation for preservation rules.
pub struct ReleaseAccounting {
    /// Value retained for `input_occurrences` without implicit normalization.
    pub input_occurrences: Option<u64>,
    /// Value retained for `normalized_occurrences` without implicit normalization.
    pub normalized_occurrences: Option<u64>,
    /// Value retained for `query_excluded_occurrences` without implicit normalization.
    pub query_excluded_occurrences: Option<u64>,
    /// Value retained for `unit` without implicit normalization.
    pub unit: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Versioned dictionary data type `ReleaseDiagnostic`; see the crate documentation for preservation rules.
pub struct ReleaseDiagnostic {
    /// Exact captured source occurrence supporting this row.
    pub occurrence: Option<OccurrenceRef>,
    /// Value retained for `code` without implicit normalization.
    pub code: String,
    /// Value retained for `message` without implicit normalization.
    pub message: String,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Versioned dictionary state vocabulary `RedistributionStatus`; see the crate documentation for preservation rules.
pub enum RedistributionStatus {
    /// The `Unknown` case.
    Unknown,
    /// The `Held` case.
    Held,
    /// The `Permitted` case.
    Permitted,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Versioned dictionary data type `ContentNotice`; see the crate documentation for preservation rules.
pub struct ContentNotice {
    /// Value retained for `text` without implicit normalization.
    pub text: Presence<String>,
    /// Source-native row locator within the capture.
    pub locator: Presence<String>,
    /// Value retained for `license` without implicit normalization.
    pub license: Presence<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Versioned dictionary data type `CodeNotice`; see the crate documentation for preservation rules.
pub struct CodeNotice {
    /// Value retained for `text` without implicit normalization.
    pub text: String,
    /// Source-native row locator within the capture.
    pub locator: Presence<String>,
    /// Value retained for `license` without implicit normalization.
    pub license: Presence<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Versioned dictionary data type `ReleaseRecord`; see the crate documentation for preservation rules.
pub struct ReleaseRecord {
    /// Stable model identifier for this row.
    pub id: EntityId,
    /// Value retained for `component` without implicit normalization.
    pub component: EntityId,
    /// Value retained for `distributor` without implicit normalization.
    pub distributor: Option<super::DistributorId>,
    /// Value retained for `upstream_revision` without implicit normalization.
    pub upstream_revision: Presence<String>,
    /// Value retained for `distributor_revision` without implicit normalization.
    pub distributor_revision: Presence<String>,
    /// Value retained for `captured_at` without implicit normalization.
    pub captured_at: Presence<String>,
    /// Value retained for `time_precision` without implicit normalization.
    pub time_precision: TimePrecision,
    /// Value retained for `schema_version` without implicit normalization.
    pub schema_version: u32,
    /// Value retained for `normalizer` without implicit normalization.
    pub normalizer: String,
    /// Value retained for `normalizer_version` without implicit normalization.
    pub normalizer_version: String,
    /// Value retained for `captures` without implicit normalization.
    pub captures: Vec<CaptureRef>,
    /// Value retained for `assertions` without implicit normalization.
    pub assertions: Vec<EntityId>,
    /// Observed input and normalized counts, with unknowns left absent.
    pub accounting: ReleaseAccounting,
    /// Value retained for `diagnostics` without implicit normalization.
    pub diagnostics: Vec<ReleaseDiagnostic>,
    /// Value retained for `selected` without implicit normalization.
    pub selected: bool,
    /// Whether payload is available, known missing, or otherwise unresolved.
    pub availability: Availability,
    /// Value retained for `content_notices` without implicit normalization.
    pub content_notices: Vec<ContentNotice>,
    /// Value retained for `redistribution` without implicit normalization.
    pub redistribution: RedistributionStatus,
    /// Value retained for `redistribution_evidence` without implicit normalization.
    pub redistribution_evidence: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Versioned dictionary data type `PosMapping`; see the crate documentation for preservation rules.
pub struct PosMapping {
    /// Value retained for `value` without implicit normalization.
    pub value: String,
    /// Value retained for `rule` without implicit normalization.
    pub rule: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Versioned dictionary data type `EntryRecord`; see the crate documentation for preservation rules.
pub struct EntryRecord {
    /// Stable model identifier for this row.
    pub id: EntityId,
    /// The registry source that owns or published this data.
    pub source: super::SourceId,
    /// Value retained for `spelling` without implicit normalization.
    pub spelling: Presence<String>,
    /// Language value retained with its original presence state.
    pub language: Presence<String>,
    /// Value retained for `script` without implicit normalization.
    pub script: Presence<String>,
    /// Value retained for `native_word_class` without implicit normalization.
    pub native_word_class: Presence<String>,
    /// Value retained for `selmaho` without implicit normalization.
    pub selmaho: Presence<String>,
    /// Value retained for `mapped_pos` without implicit normalization.
    pub mapped_pos: Presence<PosMapping>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Versioned dictionary state vocabulary `Eligibility`; see the crate documentation for preservation rules.
#[serde(deny_unknown_fields)]
pub enum Eligibility {
    /// The `Lexical` case.
    Lexical,
    /// The `Excluded ` case.
    Excluded {
        /// Explicit reason this source assertion is outside the selected projection.
        reason: String,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Versioned dictionary data type `NamedTimestamp`; see the crate documentation for preservation rules.
pub struct NamedTimestamp {
    /// Value retained for `field` without implicit normalization.
    pub field: String,
    /// Value retained for `value` without implicit normalization.
    pub value: RawJson,
    /// Value retained for `precision` without implicit normalization.
    pub precision: TimePrecision,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Versioned dictionary data type `AssertionRecord`; see the crate documentation for preservation rules.
pub struct AssertionRecord {
    /// Stable model identifier for this row.
    pub id: EntityId,
    /// The model entry associated with this assertion or form.
    pub entry: EntityId,
    /// Exact captured source occurrence supporting this row.
    pub occurrence: OccurrenceRef,
    /// Value retained for `native_sense_id` without implicit normalization.
    pub native_sense_id: Presence<String>,
    /// Language value retained with its original presence state.
    pub language: Presence<String>,
    /// Value retained for `definition` without implicit normalization.
    pub definition: Presence<TextValue>,
    /// Value retained for `source_score` without implicit normalization.
    pub source_score: Presence<RawJson>,
    /// Value retained for `timestamps` without implicit normalization.
    pub timestamps: Vec<NamedTimestamp>,
    /// Whether the assertion participates in the selected lexical projection.
    pub eligibility: Eligibility,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Versioned dictionary state vocabulary `ContextKind`; see the crate documentation for preservation rules.
pub enum ContextKind {
    /// The `Gloss` case.
    Gloss,
    /// The `Note` case.
    Note,
    /// The `Label` case.
    Label,
    /// The `Register` case.
    Register,
    /// The `Domain` case.
    Domain,
    /// The `Example` case.
    Example,
    /// The `Translation` case.
    Translation,
    /// The `Citation` case.
    Citation,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Versioned dictionary data type `ContextRecord`; see the crate documentation for preservation rules.
pub struct ContextRecord {
    /// Stable model identifier for this row.
    pub id: EntityId,
    /// The source assertion associated with this row.
    pub assertion: EntityId,
    /// Explicit classification; it does not infer a source-native value.
    pub kind: ContextKind,
    /// Value retained for `value` without implicit normalization.
    pub value: Presence<TextValue>,
    /// Value retained for `translated_from` without implicit normalization.
    pub translated_from: Option<EntityId>,
    /// Source occurrence and optional field-level citation for this assertion.
    pub evidence: SourceEvidence,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Versioned dictionary data type `ArgumentSlot`; see the crate documentation for preservation rules.
pub struct ArgumentSlot {
    /// Value retained for `order` without implicit normalization.
    pub order: u32,
    /// Value retained for `native_label` without implicit normalization.
    pub native_label: String,
    /// Value retained for `description` without implicit normalization.
    pub description: Presence<TextValue>,
    /// Value retained for `grammatical_role` without implicit normalization.
    pub grammatical_role: Presence<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Versioned dictionary data type `ArgumentFrameRecord`; see the crate documentation for preservation rules.
pub struct ArgumentFrameRecord {
    /// Stable model identifier for this row.
    pub id: EntityId,
    /// The source assertion associated with this row.
    pub assertion: EntityId,
    /// Value retained for `slots` without implicit normalization.
    pub slots: Vec<ArgumentSlot>,
    /// Source occurrence and optional field-level citation for this assertion.
    pub evidence: SourceEvidence,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Versioned dictionary state vocabulary `LocalRef`; see the crate documentation for preservation rules.
pub enum LocalRef {
    /// The `Entry` case.
    Entry(EntityId),
    /// The `Assertion` case.
    Assertion(EntityId),
    /// The `Form` case.
    Form(EntityId),
    /// The `EditorialSense` case.
    EditorialSense(EntityId),
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Versioned dictionary data type `OrderedComponent`; see the crate documentation for preservation rules.
pub struct OrderedComponent {
    /// Value retained for `order` without implicit normalization.
    pub order: u32,
    /// Value retained for `spelling` without implicit normalization.
    pub spelling: String,
    /// Value retained for `target` without implicit normalization.
    pub target: Option<LocalRef>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Versioned dictionary state vocabulary `FormKind`; see the crate documentation for preservation rules.
pub enum FormKind {
    /// The `SourceVariant` case.
    SourceVariant,
    /// The `SourcePronunciation` case.
    SourcePronunciation,
    /// The `SourceReportedForm` case.
    SourceReportedForm,
    /// The `AssignedRafsi` case.
    AssignedRafsi,
    /// The `ComputedStem` case.
    ComputedStem,
    /// The `ComputedComponents` case.
    ComputedComponents,
    /// The `GeneratedCandidate` case.
    GeneratedCandidate,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Versioned dictionary data type `FormRecord`; see the crate documentation for preservation rules.
pub struct FormRecord {
    /// Stable model identifier for this row.
    pub id: EntityId,
    /// The model entry associated with this assertion or form.
    pub entry: EntityId,
    /// Value retained for `spelling` without implicit normalization.
    pub spelling: String,
    /// Language value retained with its original presence state.
    pub language: Presence<String>,
    /// Value retained for `script` without implicit normalization.
    pub script: Presence<String>,
    /// Value retained for `pronunciation` without implicit normalization.
    pub pronunciation: Presence<TextValue>,
    /// Explicit classification; it does not infer a source-native value.
    pub kind: FormKind,
    /// Value retained for `standing` without implicit normalization.
    pub standing: Presence<String>,
    /// Ordered components, preserving duplicates and unresolved targets.
    pub components: Vec<OrderedComponent>,
    /// Source occurrence and optional field-level citation for this assertion.
    pub evidence: Option<SourceEvidence>,
    /// Value retained for `computation` without implicit normalization.
    pub computation: Option<EntityId>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Versioned dictionary data type `EtymologyRecord`; see the crate documentation for preservation rules.
pub struct EtymologyRecord {
    /// Stable model identifier for this row.
    pub id: EntityId,
    /// The source assertion associated with this row.
    pub assertion: EntityId,
    /// Value retained for `statement` without implicit normalization.
    pub statement: Presence<TextValue>,
    /// Value retained for `native_kind` without implicit normalization.
    pub native_kind: Presence<String>,
    /// Ordered components, preserving duplicates and unresolved targets.
    pub components: Vec<OrderedComponent>,
    /// Source occurrence and optional field-level citation for this assertion.
    pub evidence: SourceEvidence,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Versioned dictionary state vocabulary `RelationKind`; see the crate documentation for preservation rules.
pub enum RelationKind {
    /// The `SourceSeeAlso` case.
    SourceSeeAlso,
    /// The `SourceEtymology` case.
    SourceEtymology,
    /// The `EditorialLink` case.
    EditorialLink,
    /// The `Other` case.
    Other(String),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Versioned dictionary state vocabulary `Direction`; see the crate documentation for preservation rules.
pub enum Direction {
    /// The `Forward` case.
    Forward,
    /// The `Reverse` case.
    Reverse,
    /// The `Bidirectional` case.
    Bidirectional,
    /// The `Unspecified` case.
    Unspecified,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Versioned dictionary state vocabulary `RelationScope`; see the crate documentation for preservation rules.
pub enum RelationScope {
    /// The `Entry` case.
    Entry,
    /// The `Assertion` case.
    Assertion,
    /// The `Form` case.
    Form,
    /// The `Editorial` case.
    Editorial,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Versioned dictionary data type `CatalogTarget`; see the crate documentation for preservation rules.
pub struct CatalogTarget {
    /// The registry source that owns or published this data.
    pub source: super::SourceId,
    /// Value retained for `release` without implicit normalization.
    pub release: EntityId,
    /// Value retained for `native_entry_id` without implicit normalization.
    pub native_entry_id: Option<String>,
    /// Value retained for `native_assertion_id` without implicit normalization.
    pub native_assertion_id: Option<String>,
    /// Exact captured source occurrence supporting this row.
    pub occurrence: Option<OccurrenceRef>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Versioned dictionary state vocabulary `RelationTarget`; see the crate documentation for preservation rules.
pub enum RelationTarget {
    /// The `Local` case.
    Local(LocalRef),
    /// The `Catalog` case.
    Catalog(CatalogTarget),
    /// The `Unknown` case.
    Unknown,
    /// The `Unresolved` case.
    Unresolved(String),
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Versioned dictionary data type `RelationRecord`; see the crate documentation for preservation rules.
pub struct RelationRecord {
    /// Stable model identifier for this row.
    pub id: EntityId,
    /// Value retained for `origin` without implicit normalization.
    pub origin: LocalRef,
    /// Explicit classification; it does not infer a source-native value.
    pub kind: RelationKind,
    /// Value retained for `direction` without implicit normalization.
    pub direction: Direction,
    /// Value retained for `scope` without implicit normalization.
    pub scope: RelationScope,
    /// Value retained for `target` without implicit normalization.
    pub target: RelationTarget,
    /// Source occurrence and optional field-level citation for this assertion.
    pub evidence: Option<SourceEvidence>,
    /// Value retained for `editorial_revision` without implicit normalization.
    pub editorial_revision: Option<EntityId>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Versioned dictionary data type `OwnedRevisionRecord`; see the crate documentation for preservation rules.
pub struct OwnedRevisionRecord {
    /// Stable model identifier for this row.
    pub id: EntityId,
    /// The registry source that owns or published this data.
    pub source: super::SourceId,
    /// Immediate prior revision in the same owned history, if any.
    pub parent: Option<EntityId>,
    /// Value retained for `editor` without implicit normalization.
    pub editor: super::ContributorId,
    /// Value retained for `timestamp` without implicit normalization.
    pub timestamp: Presence<String>,
    /// Value retained for `time_precision` without implicit normalization.
    pub time_precision: TimePrecision,
    /// Value retained for `assertions` without implicit normalization.
    pub assertions: Vec<EntityId>,
    /// Value retained for `tombstones` without implicit normalization.
    pub tombstones: Vec<EntityId>,
    /// Value retained for `rationale` without implicit normalization.
    pub rationale: Presence<TextValue>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Versioned dictionary data type `OwnedHead`; see the crate documentation for preservation rules.
pub struct OwnedHead {
    /// The registry source that owns or published this data.
    pub source: super::SourceId,
    /// Value retained for `revision` without implicit normalization.
    pub revision: EntityId,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Versioned dictionary data type `EditorialSenseRecord`; see the crate documentation for preservation rules.
pub struct EditorialSenseRecord {
    /// Stable model identifier for this row.
    pub id: EntityId,
    /// Value retained for `assertions` without implicit normalization.
    pub assertions: Vec<EntityId>,
    /// Value retained for `editor` without implicit normalization.
    pub editor: super::ContributorId,
    /// Value retained for `revision` without implicit normalization.
    pub revision: EntityId,
    /// Value retained for `rationale` without implicit normalization.
    pub rationale: TextValue,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Versioned dictionary state vocabulary `InputRef`; see the crate documentation for preservation rules.
pub enum InputRef {
    /// The `Occurrence` case.
    Occurrence(OccurrenceRef),
    /// The `Entity` case.
    Entity(LocalRef),
    /// The `Literal` case.
    Literal(String),
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Versioned dictionary data type `ComputationRecord`; see the crate documentation for preservation rules.
pub struct ComputationRecord {
    /// Stable model identifier for this row.
    pub id: EntityId,
    /// Value retained for `tool` without implicit normalization.
    pub tool: String,
    /// Value retained for `tool_version` without implicit normalization.
    pub tool_version: String,
    /// Value retained for `inputs` without implicit normalization.
    pub inputs: Vec<InputRef>,
    /// Value retained for `policy` without implicit normalization.
    pub policy: String,
    /// Value retained for `profile` without implicit normalization.
    pub profile: Presence<String>,
    /// Value retained for `explanation` without implicit normalization.
    pub explanation: Presence<TextValue>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Versioned dictionary data type `ComputedAnnotation`; see the crate documentation for preservation rules.
pub struct ComputedAnnotation {
    /// Stable model identifier for this row.
    pub id: EntityId,
    /// Value retained for `computation` without implicit normalization.
    pub computation: EntityId,
    /// Value retained for `target` without implicit normalization.
    pub target: LocalRef,
    /// Explicit classification; it does not infer a source-native value.
    pub kind: String,
    /// Value retained for `value` without implicit normalization.
    pub value: RawJson,
    /// Value retained for `explanation` without implicit normalization.
    pub explanation: Presence<TextValue>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Versioned dictionary data type `ExtensionRecord`; see the crate documentation for preservation rules.
pub struct ExtensionRecord {
    /// Value retained for `owner` without implicit normalization.
    pub owner: EntityId,
    /// Value retained for `name` without implicit normalization.
    pub name: String,
    /// Value retained for `value` without implicit normalization.
    pub value: RawJson,
    /// Source occurrence and optional field-level citation for this assertion.
    pub evidence: Option<SourceEvidence>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Versioned dictionary data type `DictionaryMetadata`; see the crate documentation for preservation rules.
pub struct DictionaryMetadata {
    /// Value retained for `catalogue_revision` without implicit normalization.
    pub catalogue_revision: u64,
    /// Value retained for `code_notices` without implicit normalization.
    pub code_notices: Vec<CodeNotice>,
    /// Value retained for `collections` without implicit normalization.
    pub collections: Vec<CollectionRecord>,
    /// Ordered components, preserving duplicates and unresolved targets.
    pub components: Vec<ComponentRecord>,
    /// Value retained for `releases` without implicit normalization.
    pub releases: Vec<ReleaseRecord>,
    /// Value retained for `entries` without implicit normalization.
    pub entries: Vec<EntryRecord>,
    /// Value retained for `assertions` without implicit normalization.
    pub assertions: Vec<AssertionRecord>,
    /// Value retained for `contexts` without implicit normalization.
    pub contexts: Vec<ContextRecord>,
    /// Value retained for `frames` without implicit normalization.
    pub frames: Vec<ArgumentFrameRecord>,
    /// Value retained for `forms` without implicit normalization.
    pub forms: Vec<FormRecord>,
    /// Value retained for `etymologies` without implicit normalization.
    pub etymologies: Vec<EtymologyRecord>,
    /// Value retained for `relations` without implicit normalization.
    pub relations: Vec<RelationRecord>,
    /// Value retained for `owned_revisions` without implicit normalization.
    pub owned_revisions: Vec<OwnedRevisionRecord>,
    /// Value retained for `owned_heads` without implicit normalization.
    pub owned_heads: Vec<OwnedHead>,
    /// Value retained for `editorial_senses` without implicit normalization.
    pub editorial_senses: Vec<EditorialSenseRecord>,
    /// Value retained for `computations` without implicit normalization.
    pub computations: Vec<ComputationRecord>,
    /// Value retained for `annotations` without implicit normalization.
    pub annotations: Vec<ComputedAnnotation>,
    /// Value retained for `extensions` without implicit normalization.
    pub extensions: Vec<ExtensionRecord>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Versioned dictionary data type `DictionaryDocument`; see the crate documentation for preservation rules.
pub struct DictionaryDocument<R = SourceRegistry> {
    /// Value retained for `version` without implicit normalization.
    pub version: u32,
    /// Value retained for `legacy_archive` without implicit normalization.
    pub legacy_archive: Option<Vec<u8>>,
    /// Value retained for `registry` without implicit normalization.
    pub registry: R,
    /// Value retained for `captures` without implicit normalization.
    pub captures: Vec<RawCapture>,
    /// Capture-time source description and temporal precision.
    pub metadata: DictionaryMetadata,
}

/// A typed dictionary draft whose registry and complete graph have not been validated.
///
/// This is suitable for reading persisted data before an explicit verification
/// operation. It must not be treated as a [`Dictionary`].
pub type UnverifiedDictionaryDocument = DictionaryDocument<crate::RegistryDocument>;
impl Default for DictionaryDocument {
    fn default() -> Self {
        Self {
            version: 1,
            legacy_archive: None,
            registry: SourceRegistry::new(),
            captures: Vec::new(),
            metadata: DictionaryMetadata::default(),
        }
    }
}

impl DictionaryDocument<crate::RegistryDocument> {
    /// Parse a typed draft without validating graph relationships or duplicate
    /// JSON object keys. Call [`Dictionary::from_document`] after converting and
    /// validating its registry before treating the data as a model.
    pub fn from_unverified_json_bytes(
        bytes: &[u8],
    ) -> Result<UnverifiedDictionaryDocument, DictionaryError> {
        let text = std::str::from_utf8(bytes)
            .map_err(|error| DictionaryError::InvalidJson(error.to_string()))?;
        let document: UnverifiedDictionaryDocument = serde_json::from_str(text)
            .map_err(|error| DictionaryError::InvalidJson(error.to_string()))?;
        if document.version != 1 {
            return Err(DictionaryError::UnsupportedVersion(document.version));
        }
        if document.registry.version != 1 {
            return Err(DictionaryError::InvalidJson(format!(
                "unsupported registry document version {}",
                document.registry.version
            )));
        }
        Ok(document)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
/// A validated versioned dictionary envelope.
pub struct Dictionary(DictionaryDocument);
impl<'de> Deserialize<'de> for Dictionary {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let doc = DictionaryDocument::deserialize(d)?;
        Self::from_document(doc).map_err(serde::de::Error::custom)
    }
}
impl Dictionary {
    /// Construct an empty validated dictionary document.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
    /// Validate and construct a dictionary from the complete versioned document.
    ///
    /// Validation checks every loaded source row, registered binding, typed
    /// reference, release membership, evidence range, and legacy archive
    /// projection before making an immutable [`Dictionary`]. Missing catalogue
    /// payloads retain their explicit unknown bounds; they are not authenticated
    /// from unavailable bytes.
    ///
    /// # Errors
    ///
    /// Returns a specific [`DictionaryError`] variant for invalid identities,
    /// references, availability, accounting, provenance, history, wire values,
    /// or legacy source bytes. Untrusted malformed graphs are rejected without
    /// indexing through unchecked references.
    pub fn from_document(document: DictionaryDocument) -> Result<Self, DictionaryError> {
        validate(&document)?;
        Ok(Self(document))
    }
    /// Clone the complete serializable document.
    pub fn document(&self) -> DictionaryDocument {
        self.0.clone()
    }
    /// Consume this validated dictionary and move out its complete document.
    #[must_use]
    pub fn into_document(self) -> DictionaryDocument {
        self.0
    }
    /// Borrow the source registry.
    pub fn registry(&self) -> &SourceRegistry {
        &self.0.registry
    }
    /// Borrow all immutable source captures.
    pub fn captures(&self) -> &[RawCapture] {
        &self.0.captures
    }
    /// Borrow the normalized catalogue and assertion metadata.
    pub fn metadata(&self) -> &DictionaryMetadata {
        &self.0.metadata
    }
    /// Find a capture by its exact namespace and digest.
    pub fn capture(&self, reference: &CaptureRef) -> Option<&RawCapture> {
        self.0.captures.iter().find(|c| &c.identity == reference)
    }
    /// Find an occurrence by capture identity and exact locator.
    pub fn occurrence(&self, reference: &OccurrenceRef) -> Option<&RawOccurrence> {
        self.capture(&reference.capture)?
            .occurrences
            .iter()
            .find(|o| o.locator == reference.locator)
    }
    /// Serialize the validated dictionary as versioned JSON bytes.
    pub fn to_json_bytes(&self) -> Result<Vec<u8>, DictionaryError> {
        serde_json::to_vec(self).map_err(|e| DictionaryError::InvalidJson(e.to_string()))
    }
    /// Parse versioned JSON, reject duplicate object keys, and validate the full document.
    ///
    /// # Errors
    ///
    /// Returns [`DictionaryError::InvalidJson`] for UTF-8, syntax, duplicate-key,
    /// or typed-wire failures, otherwise the validation error returned by
    /// [`Dictionary::from_document`].
    pub fn from_json_bytes(bytes: &[u8]) -> Result<Self, DictionaryError> {
        let s =
            std::str::from_utf8(bytes).map_err(|e| DictionaryError::InvalidJson(e.to_string()))?;
        super::check_duplicate_keys(s).map_err(|e| DictionaryError::InvalidJson(e.to_string()))?;
        let d = serde_json::from_str(s).map_err(|e| DictionaryError::InvalidJson(e.to_string()))?;
        Self::from_document(d)
    }
    /// Build a versioned model from an accepted legacy archive and explicit metadata.
    ///
    /// The adapter retains the exact archive and source bytes, then validates
    /// the caller-supplied registry and normalized metadata against those rows.
    /// It does not infer lexical identities or repair unsupported legacy data.
    ///
    /// # Errors
    ///
    /// Returns [`DictionaryError::InvalidLegacy`] for an unsupported or malformed
    /// archive, or the relevant document validation error when supplied metadata
    /// disagrees with preserved source rows.
    pub fn from_legacy_archive(
        archive: &[u8],
        registry: SourceRegistry,
        metadata: DictionaryMetadata,
    ) -> Result<Self, DictionaryError> {
        let lex = Lexicon::from_archive_bytes(archive)?;
        let mut records_by_capture: BTreeMap<CaptureRef, Vec<_>> = BTreeMap::new();
        for record in lex.records() {
            records_by_capture
                .entry(CaptureRef::from(record.identity().snapshot()))
                .or_default()
                .push(record);
        }
        let mut captures = Vec::new();
        for snap in lex.snapshots() {
            let sm = lex.source_metadata(snap).ok_or_else(|| {
                DictionaryError::InvalidDocument("archive snapshot metadata absent".into())
            })?;
            let bytes = lex
                .source_bytes(snap)
                .ok_or_else(|| {
                    DictionaryError::InvalidDocument("archive snapshot bytes absent".into())
                })?
                .to_vec();
            let rows = records_by_capture
                .get(&CaptureRef::from(snap))
                .into_iter()
                .flatten()
                .map(|r| RawOccurrence {
                    locator: r.identity().locator().into(),
                    raw_line: r.raw_line().map(str::to_owned),
                    fields: r
                        .fields()
                        .iter()
                        .map(|f| NativeField {
                            name: f.name().into(),
                            encoding: if matches!(
                                r.format(),
                                SourceFormat::LaLojbanJson | SourceFormat::LensiskuJson
                            ) {
                                NativeEncoding::Json
                            } else {
                                NativeEncoding::Text
                            },
                            raw_value: f.raw_value().into(),
                        })
                        .collect(),
                })
                .collect();
            captures.push(RawCapture {
                identity: CaptureRef::from(snap),
                metadata: CaptureMetadata {
                    source_name: sm.source_name().into(),
                    format_label: format_label(sm.format()).into(),
                    locator: sm.locator().into(),
                    revision: Presence::Value(sm.revision().into()),
                    captured_at: sm
                        .captured_at()
                        .map_or(Presence::Missing, |s| Presence::Value(s.into())),
                    time_precision: sm.time_precision(),
                    notice_locator: sm
                        .notice_locator()
                        .map_or(Presence::Missing, |s| Presence::Value(s.into())),
                },
                bytes,
                occurrences: rows,
            });
        }
        let doc = DictionaryDocument {
            version: 1,
            legacy_archive: Some(archive.to_vec()),
            registry,
            captures,
            metadata,
        };
        Self::from_document(doc)
    }
}
fn format_label(f: SourceFormat) -> &'static str {
    match f {
        SourceFormat::LaLojbanJson => "LaLojbanJson",
        SourceFormat::LensiskuJson => "LensiskuJson",
        SourceFormat::LlgGismuFixed => "LlgGismuFixed",
        SourceFormat::LlgCmavoFixed => "LlgCmavoFixed",
        SourceFormat::LlgRafsiRows => "LlgRafsiRows",
    }
}

fn validate(doc: &DictionaryDocument) -> Result<(), DictionaryError> {
    let missing = |detail: String| DictionaryError::MissingReference(detail);
    if doc.version != 1 {
        return Err(DictionaryError::UnsupportedVersion(doc.version));
    }
    let mut captures = BTreeMap::new();
    let mut occurrences = BTreeSet::new();
    let mut occurrence_rows = BTreeMap::new();
    for c in &doc.captures {
        validate_capture_ref(&c.identity)?;
        let actual = super::sha256(&c.bytes);
        if actual != c.identity.sha256 {
            return Err(DictionaryError::CaptureHashMismatch {
                namespace: c.identity.namespace.clone(),
                expected: c.identity.sha256.clone(),
                actual,
            });
        }
        if captures.insert(c.identity.clone(), c).is_some() {
            return Err(DictionaryError::DuplicateCapture(
                c.identity.namespace.clone(),
            ));
        }
        for o in &c.occurrences {
            if o.locator.is_empty() {
                return Err(DictionaryError::InvalidDocument(
                    "empty occurrence locator".into(),
                ));
            }
            let occurrence = OccurrenceRef {
                capture: c.identity.clone(),
                locator: o.locator.clone(),
            };
            if !occurrences.insert(occurrence.clone()) {
                return Err(DictionaryError::DuplicateOccurrence(o.locator.clone()));
            }
            occurrence_rows.insert(occurrence, o);
            for f in &o.fields {
                if f.encoding == NativeEncoding::Json {
                    RawValue::from_string(f.raw_value.clone())
                        .map_err(|e| DictionaryError::InvalidRawJson(e.to_string()))?;
                }
            }
        }
    }
    let reg = doc.registry.document();
    let source_ids: BTreeSet<_> = reg.sources.iter().map(|x| x.id.clone()).collect();
    let source_origins: BTreeMap<_, _> = reg
        .sources
        .iter()
        .map(|x| (x.id.clone(), x.origin))
        .collect();
    let contributor_ids: BTreeSet<_> = reg.contributors.iter().map(|x| x.id.clone()).collect();
    let distributor_ids: BTreeSet<_> = reg.distributors.iter().map(|x| x.id.clone()).collect();
    for attribution in &reg.occurrences {
        let occurrence = OccurrenceRef::from(&attribution.occurrence);
        if attribution
            .contributor
            .as_ref()
            .is_some_and(|id| !contributor_ids.contains(id))
        {
            return Err(missing(format!(
                "registry attribution names unregistered contributor {}",
                attribution.contributor.as_ref().unwrap()
            )));
        }
        if captures.contains_key(&occurrence.capture) && !occurrences.contains(&occurrence) {
            return Err(missing(format!(
                "registry attribution points at nonexistent loaded occurrence {}",
                occurrence.locator
            )));
        }
    }
    let bindings: BTreeMap<_, _> = reg
        .snapshots
        .iter()
        .map(|s| {
            (
                (
                    s.snapshot.namespace().to_owned(),
                    s.snapshot.sha256().to_owned(),
                ),
                s.source.clone(),
            )
        })
        .collect();
    for c in &doc.captures {
        if !bindings.contains_key(&(c.identity.namespace.clone(), c.identity.sha256.clone())) {
            return Err(missing(format!(
                "capture {} lacks registry source binding",
                c.identity.namespace
            )));
        }
    }
    let m = &doc.metadata;
    let mut ids = BTreeSet::new();
    macro_rules! idrows {
        ($rows:expr) => {
            for r in $rows {
                if !ids.insert(r.id.clone()) {
                    return Err(DictionaryError::DuplicateEntity(format!(
                        "metadata entity {} is repeated across typed roles",
                        r.id
                    )));
                }
            }
        };
    }
    idrows!(&m.collections);
    idrows!(&m.components);
    idrows!(&m.releases);
    idrows!(&m.entries);
    idrows!(&m.assertions);
    idrows!(&m.contexts);
    idrows!(&m.frames);
    idrows!(&m.forms);
    idrows!(&m.etymologies);
    idrows!(&m.relations);
    idrows!(&m.owned_revisions);
    idrows!(&m.editorial_senses);
    idrows!(&m.computations);
    idrows!(&m.annotations);
    let entries: BTreeMap<_, _> = m.entries.iter().map(|x| (x.id.clone(), x)).collect();
    let assertions: BTreeMap<_, _> = m.assertions.iter().map(|x| (x.id.clone(), x)).collect();
    let forms: BTreeMap<_, _> = m.forms.iter().map(|x| (x.id.clone(), x)).collect();
    let revisions: BTreeMap<_, _> = m
        .owned_revisions
        .iter()
        .map(|x| (x.id.clone(), x))
        .collect();
    let comps: BTreeMap<_, _> = m.components.iter().map(|x| (x.id.clone(), x)).collect();
    let collections: BTreeMap<_, _> = m.collections.iter().map(|x| (x.id.clone(), x)).collect();
    let releases: BTreeMap<_, _> = m.releases.iter().map(|x| (x.id.clone(), x)).collect();
    let mut release_capture_memberships = BTreeMap::<EntityId, BTreeSet<CaptureRef>>::new();
    let editorial_ids: BTreeMap<_, _> = m
        .editorial_senses
        .iter()
        .map(|x| (x.id.clone(), x))
        .collect();
    let context_ids: BTreeSet<_> = m.contexts.iter().map(|x| x.id.clone()).collect();
    let computation_ids: BTreeSet<_> = m.computations.iter().map(|x| x.id.clone()).collect();
    let local_exists = |target: &LocalRef| match target {
        LocalRef::Entry(id) => entries.contains_key(id),
        LocalRef::Assertion(id) => assertions.contains_key(id),
        LocalRef::Form(id) => forms.contains_key(id),
        LocalRef::EditorialSense(id) => editorial_ids.contains_key(id),
    };
    let local_source = |target: &LocalRef| -> Option<&super::SourceId> {
        match target {
            LocalRef::Entry(id) => entries.get(id).map(|entry| &entry.source),
            LocalRef::Assertion(id) => assertions
                .get(id)
                .and_then(|a| entries.get(&a.entry))
                .map(|entry| &entry.source),
            LocalRef::Form(id) => forms
                .get(id)
                .and_then(|form| entries.get(&form.entry))
                .map(|entry| &entry.source),
            LocalRef::EditorialSense(id) => editorial_ids
                .get(id)
                .and_then(|sense| revisions.get(&sense.revision))
                .map(|revision| &revision.source),
        }
    };
    for c in &m.collections {
        if !source_ids.contains(&c.source) {
            return Err(DictionaryError::MissingReference(format!(
                "collection {} names unregistered source {}",
                c.id, c.source
            )));
        }
    }
    for e in &m.entries {
        if !source_ids.contains(&e.source) {
            return Err(DictionaryError::MissingReference(format!(
                "entry {} names unregistered source {}",
                e.id, e.source
            )));
        }
    }
    for c in &m.components {
        if !collections.contains_key(&c.collection) {
            return Err(missing(format!(
                "component {} collection {} is absent or has the wrong role",
                c.id, c.collection
            )));
        }
    }
    let mut absent_capture_sources = BTreeMap::<CaptureRef, super::SourceId>::new();
    let mut unavailable_assertions = BTreeSet::new();
    for r in &m.releases {
        if let Some(distributor) = &r.distributor
            && !distributor_ids.contains(distributor)
        {
            return Err(DictionaryError::MissingReference(format!(
                "release {} names unregistered distributor {}",
                r.id, distributor
            )));
        }
        let comp = comps.get(&r.component).ok_or_else(|| {
            missing(format!(
                "release {} component {} is absent or has the wrong role",
                r.id, r.component
            ))
        })?;
        let source = &collections[&comp.collection].source;
        if r.availability != Availability::Available {
            unavailable_assertions.extend(r.assertions.iter().cloned());
        }
        let mut capture_membership = BTreeSet::new();
        if r.captures
            .iter()
            .any(|cap| !capture_membership.insert(cap.clone()))
        {
            return Err(DictionaryError::InvalidAvailability(format!(
                "release {} repeats a capture",
                r.id
            )));
        }
        release_capture_memberships.insert(r.id.clone(), capture_membership.clone());
        let mut assertion_membership = BTreeSet::new();
        if r.assertions
            .iter()
            .any(|id| !assertion_membership.insert(id.clone()))
        {
            return Err(DictionaryError::InvalidAccounting(format!(
                "release {} repeats an assertion",
                r.id
            )));
        }
        for id in &r.assertions {
            let assertion = assertions.get(id).ok_or_else(|| {
                DictionaryError::MissingReference(format!(
                    "release {} assertion member {} is absent or has the wrong role",
                    r.id, id
                ))
            })?;
            let entry = entries.get(&assertion.entry).ok_or_else(|| {
                DictionaryError::MissingReference(format!(
                    "release {} assertion {} has no entry",
                    r.id, id
                ))
            })?;
            if entry.source != *source {
                return Err(DictionaryError::MissingReference(format!(
                    "release {} assertion {} belongs to another source",
                    r.id, id
                )));
            }
        }
        for diagnostic in &r.diagnostics {
            if let Some(occurrence) = &diagnostic.occurrence {
                validate_capture_ref(&occurrence.capture)?;
                if occurrence.locator.is_empty() {
                    return Err(missing(format!(
                        "release {} diagnostic has an empty occurrence locator",
                        r.id
                    )));
                }
                if !capture_membership.contains(&occurrence.capture) {
                    return Err(DictionaryError::MissingReference(format!(
                        "release {} diagnostic references a capture outside its release",
                        r.id
                    )));
                }
                if captures.contains_key(&occurrence.capture) && !occurrences.contains(occurrence) {
                    return Err(DictionaryError::MissingReference(format!(
                        "release {} diagnostic references a phantom loaded occurrence {}",
                        r.id, occurrence.locator
                    )));
                }
            }
        }
        for cap in &r.captures {
            validate_capture_ref(cap)?;
            if r.availability != Availability::Available
                && !captures.contains_key(cap)
                && absent_capture_sources
                    .insert(cap.clone(), source.clone())
                    .is_some_and(|old| old != *source)
            {
                return Err(missing(format!(
                    "unavailable capture {} has conflicting catalogue sources",
                    cap.namespace
                )));
            }
            if !bindings.contains_key(&(cap.namespace.clone(), cap.sha256.clone()))
                && r.availability == Availability::Available
            {
                return Err(DictionaryError::InvalidAvailability(format!(
                    "available release {} capture {} lacks a registry source binding",
                    r.id, cap.namespace
                )));
            }
            if let Some(bound) = bindings.get(&(cap.namespace.clone(), cap.sha256.clone())) {
                if bound != source {
                    return Err(missing(format!(
                        "release {} capture {} is bound to a different source",
                        r.id, cap.namespace
                    )));
                }
                if r.availability == Availability::Available && !captures.contains_key(cap) {
                    return Err(DictionaryError::InvalidAvailability(format!(
                        "available release {} capture {} has no payload",
                        r.id, cap.namespace
                    )));
                }
            }
        }
        if r.availability == Availability::Available
            && (r.accounting.input_occurrences.is_none()
                || r.accounting.normalized_occurrences.is_none()
                || r.accounting.query_excluded_occurrences.is_none())
        {
            return Err(DictionaryError::InvalidAccounting(format!(
                "available release {} has unknown counts",
                r.id
            )));
        }
        if r.redistribution == RedistributionStatus::Permitted
            && r.redistribution_evidence.is_empty()
        {
            return Err(DictionaryError::InvalidProvenance(format!(
                "release {} permits redistribution without evidence",
                r.id
            )));
        }
    }
    for (namespace, sha256) in bindings.keys() {
        let reference = CaptureRef {
            namespace: namespace.clone(),
            sha256: sha256.clone(),
        };
        validate_capture_ref(&reference)?;
        if !captures.contains_key(&reference) && !absent_capture_sources.contains_key(&reference) {
            return Err(missing(format!(
                "registry snapshot {} has no loaded or unavailable catalogue capture",
                reference.namespace
            )));
        }
    }
    let mut asserted = BTreeSet::new();
    for a in &m.assertions {
        let Some(entry) = entries.get(&a.entry) else {
            return Err(missing(format!(
                "assertion {} entry {} is absent or has the wrong role",
                a.id, a.entry
            )));
        };
        if matches!(a.eligibility, Eligibility::Lexical)
            && !matches!(entry.spelling, Presence::Value(_))
        {
            return Err(DictionaryError::InvalidProvenance(format!(
                "lexical assertion {} requires a present entry spelling",
                a.id
            )));
        }
        let loaded = occurrences.contains(&a.occurrence);
        validate_capture_ref(&a.occurrence.capture)?;
        if a.occurrence.locator.is_empty() {
            return Err(DictionaryError::InvalidDocument(
                "empty assertion locator".into(),
            ));
        }
        let catalogue_membership = unavailable_assertions.contains(&a.id);
        if !loaded
            && (!absent_capture_sources.contains_key(&a.occurrence.capture)
                || !catalogue_membership)
        {
            return Err(missing(format!(
                "assertion {} occurrence is neither loaded nor named by an unavailable release",
                a.id
            )));
        }
        if !asserted.insert(a.occurrence.clone()) {
            return Err(DictionaryError::DuplicateOccurrence(format!(
                "{}:{} has multiple normalized assertions",
                a.occurrence.capture.namespace, a.occurrence.locator
            )));
        }
        let src = bindings
            .get(&(
                a.occurrence.capture.namespace.clone(),
                a.occurrence.capture.sha256.clone(),
            ))
            .or_else(|| absent_capture_sources.get(&a.occurrence.capture))
            .ok_or_else(|| {
                missing(format!(
                    "assertion {} capture has no source/catalogue binding",
                    a.id
                ))
            })?;
        if &entries[&a.entry].source != src {
            return Err(missing(format!(
                "assertion {} entry source differs from capture source",
                a.id
            )));
        }
        if matches!(a.eligibility,Eligibility::Excluded{ref reason} if reason.is_empty()) {
            return Err(DictionaryError::InvalidDocument("empty exclusion".into()));
        }
    }
    if !occurrences.is_subset(&asserted) {
        return Err(DictionaryError::InvalidAccounting(
            "a loaded occurrence lacks exactly one normalized assertion".into(),
        ));
    }
    for r in &m.releases {
        if r.availability == Availability::Available {
            let listed: BTreeSet<_> = r.assertions.iter().cloned().collect();
            if listed.len() != r.assertions.len() {
                return Err(DictionaryError::InvalidAccounting(format!(
                    "release {} repeats an assertion",
                    r.id
                )));
            }
            let comp = &comps[&r.component];
            let source = &collections[&comp.collection].source;
            let mut expected = BTreeSet::new();
            for id in &r.assertions {
                let a = assertions.get(id).ok_or_else(|| {
                    missing(format!(
                        "release {} assertion {} is absent or has the wrong role",
                        r.id, id
                    ))
                })?;
                if &entries[&a.entry].source != source {
                    return Err(missing(format!(
                        "release {} assertion {} belongs to another source",
                        r.id, id
                    )));
                }
                expected.insert(a.occurrence.clone());
            }
            let actual: BTreeSet<_> = r
                .captures
                .iter()
                .flat_map(|c| {
                    captures.get(c).into_iter().flat_map(move |x| {
                        x.occurrences.iter().map(move |o| OccurrenceRef {
                            capture: c.clone(),
                            locator: o.locator.clone(),
                        })
                    })
                })
                .collect();
            if expected != actual {
                return Err(DictionaryError::InvalidAccounting(format!(
                    "available release {} occurrence membership differs from its capture set",
                    r.id
                )));
            }
            let excluded = r
                .assertions
                .iter()
                .filter(|id| matches!(assertions[*id].eligibility, Eligibility::Excluded { .. }))
                .count() as u64;
            if r.accounting.input_occurrences != Some(actual.len() as u64)
                || r.accounting.normalized_occurrences != Some(r.assertions.len() as u64)
                || r.accounting.query_excluded_occurrences != Some(excluded)
            {
                return Err(DictionaryError::InvalidAccounting(format!(
                    "available release {} counts do not match its occurrence and assertion sets",
                    r.id
                )));
            }
        }
    }
    for c in &m.contexts {
        let a = assertions.get(&c.assertion).ok_or_else(|| {
            missing(format!(
                "context {} assertion {} is absent or has the wrong role",
                c.id, c.assertion
            ))
        })?;
        validate_evidence(&occurrence_rows, &absent_capture_sources, &c.evidence)?;
        if c.evidence.occurrence != a.occurrence {
            return Err(DictionaryError::InvalidProvenance(format!(
                "context {} evidence does not identify its assertion occurrence",
                c.id
            )));
        }
        if c.translated_from
            .as_ref()
            .is_some_and(|id| !context_ids.contains(id))
        {
            return Err(missing(format!(
                "context {} translated_from target is absent",
                c.id
            )));
        }
    }
    for f in &m.frames {
        let a = assertions.get(&f.assertion).ok_or_else(|| {
            missing(format!(
                "frame {} assertion {} is absent or has the wrong role",
                f.id, f.assertion
            ))
        })?;
        validate_evidence(&occurrence_rows, &absent_capture_sources, &f.evidence)?;
        if f.evidence.occurrence != a.occurrence {
            return Err(DictionaryError::InvalidProvenance(format!(
                "frame {} evidence does not identify its assertion occurrence",
                f.id
            )));
        }
        unique_orders(f.slots.iter().map(|x| x.order))?;
    }
    for f in &m.forms {
        if !entries.contains_key(&f.entry) {
            return Err(missing(format!(
                "form {} entry {} is absent or has the wrong role",
                f.id, f.entry
            )));
        }
        match f.kind {
            FormKind::ComputedStem
            | FormKind::ComputedComponents
            | FormKind::GeneratedCandidate => {
                if f.computation.is_none() || f.evidence.is_some() {
                    return Err(DictionaryError::InvalidProvenance(format!(
                        "computed form {} requires computation and excludes source evidence",
                        f.id
                    )));
                }
            }
            _ => {
                if f.evidence.is_none() || f.computation.is_some() {
                    return Err(DictionaryError::InvalidProvenance(format!(
                        "source form {} requires evidence and excludes computation",
                        f.id
                    )));
                }
            }
        }
        if let Some(e) = &f.evidence {
            validate_evidence(&occurrence_rows, &absent_capture_sources, e)?;
            let source = bindings
                .get(&(
                    e.occurrence.capture.namespace.clone(),
                    e.occurrence.capture.sha256.clone(),
                ))
                .or_else(|| absent_capture_sources.get(&e.occurrence.capture))
                .ok_or_else(|| missing(format!("form {} evidence has no source binding", f.id)))?;
            if source != &entries[&f.entry].source {
                return Err(DictionaryError::InvalidProvenance(format!(
                    "form {} evidence belongs to another source",
                    f.id
                )));
            }
        }
        if f.computation
            .as_ref()
            .is_some_and(|id| !computation_ids.contains(id))
        {
            return Err(missing(format!(
                "form {} computation is absent or has the wrong role",
                f.id
            )));
        }
        for component in &f.components {
            if let Some(target) = &component.target
                && !local_exists(target)
            {
                return Err(missing(format!(
                    "form {} component target is absent or has the wrong role",
                    f.id
                )));
            }
        }
        unique_orders(f.components.iter().map(|x| x.order))?;
    }
    for e in &m.etymologies {
        let a = assertions.get(&e.assertion).ok_or_else(|| {
            missing(format!(
                "etymology {} assertion {} is absent or has the wrong role",
                e.id, e.assertion
            ))
        })?;
        validate_evidence(&occurrence_rows, &absent_capture_sources, &e.evidence)?;
        if e.evidence.occurrence != a.occurrence {
            return Err(DictionaryError::InvalidProvenance(format!(
                "etymology {} evidence does not identify its assertion occurrence",
                e.id
            )));
        }
        for component in &e.components {
            if let Some(target) = &component.target
                && !local_exists(target)
            {
                return Err(missing(format!(
                    "etymology {} component target is absent or has the wrong role",
                    e.id
                )));
            }
        }
        unique_orders(e.components.iter().map(|x| x.order))?;
    }
    for r in &m.relations {
        let provenance_ok = match r.kind {
            RelationKind::SourceSeeAlso | RelationKind::SourceEtymology => {
                r.evidence.is_some() && r.editorial_revision.is_none()
            }
            RelationKind::EditorialLink => r.evidence.is_none() && r.editorial_revision.is_some(),
            RelationKind::Other(_) => !(r.evidence.is_some() && r.editorial_revision.is_some()),
        };
        if !provenance_ok {
            return Err(DictionaryError::InvalidProvenance(format!(
                "relation {} has inconsistent source/editorial provenance",
                r.id
            )));
        }
        if let LocalRef::Entry(id) = &r.origin {
            if !entries.contains_key(id) {
                return Err(missing(format!(
                    "relation {} origin entry {} is absent or has the wrong role",
                    r.id, id
                )));
            }
        } else if !local_exists(&r.origin) {
            return Err(missing(format!(
                "relation {} origin is absent or has the wrong role",
                r.id
            )));
        }
        if let RelationTarget::Local(target) = &r.target
            && !local_exists(target)
        {
            return Err(missing(format!(
                "relation {} local target is absent or has the wrong role",
                r.id
            )));
        }
        if r.editorial_revision
            .as_ref()
            .is_some_and(|id| !revisions.contains_key(id))
        {
            return Err(missing(format!(
                "relation {} editorial revision is absent or has the wrong role",
                r.id
            )));
        }
        if let Some(e) = &r.evidence {
            validate_evidence(&occurrence_rows, &absent_capture_sources, e)?;
            let evidence_source = bindings
                .get(&(
                    e.occurrence.capture.namespace.clone(),
                    e.occurrence.capture.sha256.clone(),
                ))
                .or_else(|| absent_capture_sources.get(&e.occurrence.capture));
            if evidence_source != local_source(&r.origin) {
                return Err(DictionaryError::InvalidProvenance(format!(
                    "relation {} evidence belongs to another source",
                    r.id
                )));
            }
        }
        if let RelationTarget::Unresolved(s) = &r.target
            && s.is_empty()
        {
            return Err(DictionaryError::InvalidDocument(
                "empty unresolved target".into(),
            ));
        }
        if let RelationTarget::Catalog(t) = &r.target {
            if t.native_entry_id.is_none()
                && t.native_assertion_id.is_none()
                && t.occurrence.is_none()
            {
                return Err(DictionaryError::InvalidDocument(
                    "empty catalogue target".into(),
                ));
            }
            let Some(release) = releases.get(&t.release) else {
                return Err(missing(format!(
                    "catalogue target {} release {} is absent or has the wrong role",
                    r.id, t.release
                )));
            };
            let component = comps.get(&release.component).ok_or_else(|| {
                missing(format!(
                    "catalogue target {} release component is absent",
                    r.id
                ))
            })?;
            let collection = collections.get(&component.collection).ok_or_else(|| {
                missing(format!(
                    "catalogue target {} component collection is absent",
                    r.id
                ))
            })?;
            if collection.source != t.source {
                return Err(missing(format!(
                    "catalogue target {} source differs from its referenced release",
                    r.id
                )));
            }
            if t.occurrence.as_ref().is_some_and(|o| {
                !release_capture_memberships
                    .get(&t.release)
                    .is_some_and(|members| members.contains(&o.capture))
            }) {
                return Err(missing(format!(
                    "catalogue target {} occurrence is outside its release",
                    r.id
                )));
            }
            if let Some(occurrence) = &t.occurrence {
                validate_capture_ref(&occurrence.capture)?;
                if occurrence.locator.is_empty() {
                    return Err(missing(format!(
                        "catalogue target {} has an empty occurrence locator",
                        r.id
                    )));
                }
                if captures.contains_key(&occurrence.capture) && !occurrences.contains(occurrence) {
                    return Err(missing(format!(
                        "catalogue target {} names phantom loaded occurrence {}",
                        r.id, occurrence.locator
                    )));
                }
                if !captures.contains_key(&occurrence.capture)
                    && !absent_capture_sources.contains_key(&occurrence.capture)
                    && release.availability != Availability::Available
                {
                    return Err(missing(format!(
                        "catalogue target {} occurrence is not known through a loaded or unavailable source",
                        r.id
                    )));
                }
            }
        }
    }
    for r in &m.owned_revisions {
        if !contributor_ids.contains(&r.editor) {
            return Err(missing(format!(
                "owned revision {} names unregistered editor {}",
                r.id, r.editor
            )));
        }
        if source_origins.get(&r.source) != Some(&super::SourceOrigin::OwnedDraft) {
            return Err(DictionaryError::InvalidHistory(format!(
                "revision {} source {} is not registered as an owned draft",
                r.id, r.source
            )));
        }
        if let Some(parent_id) = &r.parent {
            let parent = revisions.get(parent_id).ok_or_else(|| {
                missing(format!(
                    "revision {} parent {} is absent or has the wrong role",
                    r.id, parent_id
                ))
            })?;
            if parent.source != r.source {
                return Err(DictionaryError::InvalidHistory(format!(
                    "revision {} parent belongs to another source",
                    r.id
                )));
            }
        }
    }
    for r in &m.owned_revisions {
        let mut assertion_ids = BTreeSet::new();
        for a in &r.assertions {
            if !assertion_ids.insert(a.clone())
                || !assertions.contains_key(a)
                || entries[&assertions[a].entry].source != r.source
            {
                return Err(DictionaryError::InvalidHistory(format!(
                    "revision {} has a repeated, absent, or foreign-source assertion",
                    r.id
                )));
            }
        }
        let mut ancestor_assertions = BTreeSet::new();
        let mut ancestor = r.parent.clone();
        let mut ancestor_seen = BTreeSet::new();
        while let Some(parent_id) = ancestor {
            if !ancestor_seen.insert(parent_id.clone()) || parent_id == r.id {
                return Err(DictionaryError::InvalidHistory(format!(
                    "cycle in owned revision {} ancestry",
                    r.id
                )));
            }
            let Some(parent) = revisions.get(&parent_id) else {
                return Err(missing(format!(
                    "revision {} ancestry names absent parent {}",
                    r.id, parent_id
                )));
            };
            ancestor_assertions.extend(parent.assertions.iter().cloned());
            ancestor = parent.parent.clone();
        }
        let mut tombstones = BTreeSet::new();
        for a in &r.tombstones {
            if !tombstones.insert(a.clone())
                || !assertions.contains_key(a)
                || assertion_ids.contains(a)
                || !ancestor_assertions.contains(a)
            {
                return Err(DictionaryError::InvalidHistory(format!(
                    "revision {} has an invalid tombstone",
                    r.id
                )));
            }
        }
    }
    let mut head_sources = BTreeSet::new();
    for h in &m.owned_heads {
        if !head_sources.insert(h.source.clone()) {
            return Err(DictionaryError::InvalidHistory(format!(
                "multiple owned heads for source {}",
                h.source
            )));
        }
        if revisions
            .get(&h.revision)
            .is_none_or(|r| r.source != h.source)
        {
            return Err(DictionaryError::InvalidHistory(format!(
                "owned head for {} does not identify a revision of that source",
                h.source
            )));
        }
    }
    for e in &m.editorial_senses {
        if !contributor_ids.contains(&e.editor) {
            return Err(missing(format!(
                "editorial sense {} names unregistered editor {}",
                e.id, e.editor
            )));
        }
        if !revisions.contains_key(&e.revision)
            || e.assertions.iter().any(|a| !assertions.contains_key(a))
        {
            return Err(missing(format!(
                "editorial sense {} names an absent revision or assertion",
                e.id
            )));
        }
    }
    for c in &m.computations {
        if c.tool.is_empty() || c.tool_version.is_empty() || c.policy.is_empty() {
            return Err(DictionaryError::InvalidProvenance(format!(
                "computation {} requires tool, version, and policy",
                c.id
            )));
        }
        for input in &c.inputs {
            match input {
                InputRef::Entity(local) if !local_exists(local) => {
                    return Err(missing(format!(
                        "computation {} input entity is absent or has the wrong role",
                        c.id
                    )));
                }
                InputRef::Occurrence(occurrence) => {
                    validate_capture_ref(&occurrence.capture)?;
                    if captures.contains_key(&occurrence.capture)
                        && !occurrences.contains(occurrence)
                    {
                        return Err(missing(format!(
                            "computation {} input names phantom loaded occurrence {}",
                            c.id, occurrence.locator
                        )));
                    }
                    if !captures.contains_key(&occurrence.capture)
                        && !absent_capture_sources.contains_key(&occurrence.capture)
                    {
                        return Err(missing(format!(
                            "computation {} input occurrence is not loaded or catalogued unavailable",
                            c.id
                        )));
                    }
                }
                _ => {}
            }
        }
    }
    for a in &m.annotations {
        if !computation_ids.contains(&a.computation) || !local_exists(&a.target) {
            return Err(missing(format!(
                "annotation {} computation or typed target is absent",
                a.id
            )));
        }
    }
    for x in &m.extensions {
        if !ids.contains(&x.owner) {
            return Err(missing(format!(
                "extension {} owner {} is absent",
                x.name, x.owner
            )));
        }
        if let Some(e) = &x.evidence {
            validate_evidence(&occurrence_rows, &absent_capture_sources, e)?;
        }
    }
    if let Some(raw) = &doc.legacy_archive {
        let lex = Lexicon::from_archive_bytes(raw)?;
        let mut legacy_rows_by_capture: BTreeMap<_, BTreeMap<String, _>> = lex
            .snapshots()
            .into_iter()
            .map(|snapshot| (CaptureRef::from(snapshot), BTreeMap::new()))
            .collect();
        for row in lex.records() {
            let capture = CaptureRef::from(row.identity().snapshot());
            let rows = legacy_rows_by_capture.get_mut(&capture).ok_or_else(|| {
                DictionaryError::InvalidLegacy(ImportError::InvalidRecord {
                    locator: row.identity().locator().into(),
                    reason: "legacy record references an absent snapshot".into(),
                })
            })?;
            rows.insert(row.identity().locator().to_owned(), row);
        }
        for s in lex.snapshots() {
            let r = CaptureRef::from(s);
            let sm = lex.source_metadata(s).ok_or_else(|| {
                DictionaryError::InvalidDocument("legacy snapshot metadata absent".into())
            })?;
            let cap = captures.get(&r).ok_or_else(|| {
                DictionaryError::InvalidDocument("legacy capture absent from model".into())
            })?;
            let original = lex.source_bytes(s).ok_or_else(|| {
                DictionaryError::InvalidDocument("legacy source bytes absent".into())
            })?;
            if cap.bytes != original {
                return Err(DictionaryError::InvalidDocument(
                    "legacy source bytes changed".into(),
                ));
            }
            let captured_at = sm
                .captured_at()
                .map_or(Presence::Missing, |s| Presence::Value(s.into()));
            let notice_locator = sm
                .notice_locator()
                .map_or(Presence::Missing, |s| Presence::Value(s.into()));
            if cap.metadata.source_name != sm.source_name()
                || cap.metadata.format_label != format_label(sm.format())
                || cap.metadata.locator != sm.locator()
                || cap.metadata.revision != Presence::Value(sm.revision().into())
                || cap.metadata.captured_at != captured_at
                || cap.metadata.time_precision != sm.time_precision()
                || cap.metadata.notice_locator != notice_locator
            {
                return Err(DictionaryError::InvalidDocument(
                    "legacy source metadata changed".into(),
                ));
            }
            let rows = legacy_rows_by_capture.get(&r).ok_or_else(|| {
                DictionaryError::InvalidLegacy(ImportError::InvalidRecord {
                    locator: r.namespace.clone(),
                    reason: "legacy snapshot has no indexed record group".into(),
                })
            })?;
            let captured_rows: BTreeMap<_, _> = cap
                .occurrences
                .iter()
                .map(|occurrence| (occurrence.locator.as_str(), occurrence))
                .collect();
            if captured_rows.len() != cap.occurrences.len() || captured_rows.len() != rows.len() {
                return Err(DictionaryError::InvalidDocument(
                    "legacy occurrence set changed".into(),
                ));
            }
            for (locator, row) in rows {
                let found = captured_rows.get(locator.as_str()).ok_or_else(|| {
                    DictionaryError::InvalidDocument("legacy locator changed".into())
                })?;
                if found.raw_line.as_deref() != row.raw_line()
                    || found.fields.len() != row.fields().len()
                    || found.fields.iter().zip(row.fields()).any(|(a, b)| {
                        a.name != b.name()
                            || a.raw_value != b.raw_value()
                            || a.encoding
                                != if matches!(
                                    row.format(),
                                    SourceFormat::LaLojbanJson | SourceFormat::LensiskuJson
                                ) {
                                    NativeEncoding::Json
                                } else {
                                    NativeEncoding::Text
                                }
                    })
                {
                    return Err(DictionaryError::InvalidDocument(
                        "legacy native fields changed".into(),
                    ));
                }
            }
        }
    }
    Ok(())
}
fn unique_orders<I: Iterator<Item = u32>>(it: I) -> Result<(), DictionaryError> {
    let mut seen = BTreeSet::new();
    for n in it {
        if !seen.insert(n) {
            return Err(DictionaryError::InvalidDocument(
                "duplicate ordered position".into(),
            ));
        }
    }
    Ok(())
}

fn validate_capture_ref(reference: &CaptureRef) -> Result<(), DictionaryError> {
    if reference.sha256.len() != 64
        || !reference
            .sha256
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(DictionaryError::InvalidId(format!(
            "capture SHA-256 is not lowercase hexadecimal: {}",
            reference.sha256
        )));
    }
    Ok(())
}

fn validate_evidence(
    occurrence_rows: &BTreeMap<OccurrenceRef, &RawOccurrence>,
    absent_capture_sources: &BTreeMap<CaptureRef, super::SourceId>,
    e: &SourceEvidence,
) -> Result<(), DictionaryError> {
    let occ = occurrence_rows.get(&e.occurrence).copied();
    let Some(occ) = occ else {
        if !absent_capture_sources.contains_key(&e.occurrence.capture) {
            return Err(DictionaryError::MissingReference(format!(
                "evidence occurrence {} is neither loaded nor catalogued unavailable",
                e.occurrence.locator
            )));
        }
        if e.offset
            .as_ref()
            .is_some_and(|range| range.start > range.end)
        {
            return Err(DictionaryError::InvalidDocument(
                "unavailable evidence range ordering invalid".into(),
            ));
        }
        if e.field_index.is_none() && e.offset.is_some() {
            return Err(DictionaryError::InvalidDocument(
                "range without field".into(),
            ));
        }
        return Ok(());
    };
    if let Some(i) = e.field_index {
        let f = occ.fields.get(i).ok_or_else(|| {
            DictionaryError::InvalidDocument("evidence field index invalid".into())
        })?;
        if let Some(r) = &e.offset {
            let text = match f.encoding {
                NativeEncoding::Text => Some(f.raw_value.clone()),
                NativeEncoding::Json => serde_json::from_str::<String>(&f.raw_value).ok(),
            };
            let text = text.ok_or_else(|| {
                DictionaryError::InvalidDocument("text range on non-text field".into())
            })?;
            if r.start > r.end
                || r.end > text.len()
                || !text.is_char_boundary(r.start)
                || !text.is_char_boundary(r.end)
            {
                return Err(DictionaryError::InvalidDocument(
                    "evidence text range invalid".into(),
                ));
            }
        }
    } else if e.offset.is_some() {
        return Err(DictionaryError::InvalidDocument(
            "range without field".into(),
        ));
    }
    Ok(())
}
