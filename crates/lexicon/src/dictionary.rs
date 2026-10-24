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

#[derive(Debug, Clone, PartialEq, Eq)]
/// Versioned dictionary state vocabulary `DictionaryError`; see the crate documentation for preservation rules.
pub enum DictionaryError {
    /// The `InvalidId` case.
    InvalidId(String),
    /// The `InvalidRawJson` case.
    InvalidRawJson(String),
    /// The `InvalidDocument` case.
    InvalidDocument(String),
    /// The `UnsupportedVersion` case.
    UnsupportedVersion(u32),
    /// The `InvalidJson` case.
    InvalidJson(String),
    /// The `InvalidLegacy` case.
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
    /// The `DuplicateCapture` case.
    DuplicateCapture(String),
    /// The `DuplicateOccurrence` case.
    DuplicateOccurrence(String),
    /// The `DuplicateEntity` case.
    DuplicateEntity(String),
    /// The `MissingReference` case.
    MissingReference(String),
    /// The `InvalidAvailability` case.
    InvalidAvailability(String),
    /// The `InvalidAccounting` case.
    InvalidAccounting(String),
    /// The `InvalidProvenance` case.
    InvalidProvenance(String),
    /// The `InvalidHistory` case.
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
    /// Value retained for `fn new(value` without implicit normalization.
    /// Construct a validated value from its external representation.
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
    /// Borrow the validated identifier or raw JSON source text.
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
    /// Value retained for `fn new(value` without implicit normalization.
    /// Construct a validated value from its external representation.
    pub fn new(value: impl Into<String>) -> Result<Self, DictionaryError> {
        let value = value.into();
        RawValue::from_string(value.clone())
            .map_err(|e| DictionaryError::InvalidRawJson(e.to_string()))?;
        Ok(Self(value))
    }
    /// Borrow the validated identifier or raw JSON source text.
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
    /// Construct a validated value from its external representation.
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
pub struct DictionaryDocument {
    /// Value retained for `version` without implicit normalization.
    pub version: u32,
    /// Value retained for `legacy_archive` without implicit normalization.
    pub legacy_archive: Option<Vec<u8>>,
    /// Value retained for `registry` without implicit normalization.
    pub registry: SourceRegistry,
    /// Value retained for `captures` without implicit normalization.
    pub captures: Vec<RawCapture>,
    /// Capture-time source description and temporal precision.
    pub metadata: DictionaryMetadata,
}
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
    /// Construct a validated value from its external representation.
    pub fn new() -> Self {
        Self::default()
    }
    /// Validate and construct a dictionary from a complete versioned document.
    pub fn from_document(document: DictionaryDocument) -> Result<Self, DictionaryError> {
        validate(&document)?;
        Ok(Self(document))
    }
    /// Clone the complete serializable document.
    pub fn document(&self) -> DictionaryDocument {
        self.0.clone()
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
    /// Parse JSON and validate the complete versioned dictionary.
    pub fn from_json_bytes(bytes: &[u8]) -> Result<Self, DictionaryError> {
        let s =
            std::str::from_utf8(bytes).map_err(|e| DictionaryError::InvalidJson(e.to_string()))?;
        super::check_duplicate_keys(s).map_err(|e| DictionaryError::InvalidJson(e.to_string()))?;
        let d = serde_json::from_str(s).map_err(|e| DictionaryError::InvalidJson(e.to_string()))?;
        Self::from_document(d)
    }
    /// Build an initial model while preserving the legacy archive and verifying its source rows.
    pub fn from_legacy_archive(
        archive: &[u8],
        registry: SourceRegistry,
        metadata: DictionaryMetadata,
    ) -> Result<Self, DictionaryError> {
        let lex = Lexicon::from_archive_bytes(archive)?;
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
            let rows = lex
                .records()
                .iter()
                .filter(|r| r.identity().snapshot() == snap)
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
    if doc.version != 1 {
        return Err(DictionaryError::UnsupportedVersion(doc.version));
    }
    let mut captures = BTreeMap::new();
    let mut occurrences = BTreeSet::new();
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
            if !occurrences.insert(OccurrenceRef {
                capture: c.identity.clone(),
                locator: o.locator.clone(),
            }) {
                return Err(DictionaryError::DuplicateOccurrence(o.locator.clone()));
            }
            for f in &o.fields {
                if f.encoding == NativeEncoding::Json {
                    RawValue::from_string(f.raw_value.clone())
                        .map_err(|e| DictionaryError::InvalidRawJson(e.to_string()))?;
                }
            }
        }
    }
    let reg = doc.registry.document();
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
            return Err(DictionaryError::InvalidDocument(
                "capture lacks registry source binding".into(),
            ));
        }
    }
    let m = &doc.metadata;
    let mut ids = BTreeSet::new();
    macro_rules! idrows {
        ($rows:expr) => {
            for r in $rows {
                if !ids.insert(r.id.clone()) {
                    return Err(DictionaryError::InvalidDocument(format!(
                        "duplicate metadata id {}",
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
    let forms: BTreeSet<_> = m.forms.iter().map(|x| x.id.clone()).collect();
    let revisions: BTreeMap<_, _> = m
        .owned_revisions
        .iter()
        .map(|x| (x.id.clone(), x))
        .collect();
    let comps: BTreeMap<_, _> = m.components.iter().map(|x| (x.id.clone(), x)).collect();
    let collections: BTreeMap<_, _> = m.collections.iter().map(|x| (x.id.clone(), x)).collect();
    let releases: BTreeMap<_, _> = m.releases.iter().map(|x| (x.id.clone(), x)).collect();
    let editorial_ids: BTreeSet<_> = m.editorial_senses.iter().map(|x| x.id.clone()).collect();
    let local_exists = |target: &LocalRef| match target {
        LocalRef::Entry(id) => entries.contains_key(id),
        LocalRef::Assertion(id) => assertions.contains_key(id),
        LocalRef::Form(id) => forms.contains(id),
        LocalRef::EditorialSense(id) => editorial_ids.contains(id),
    };
    let local_source = |target: &LocalRef| -> Option<&super::SourceId> {
        match target {
            LocalRef::Entry(id) => entries.get(id).map(|entry| &entry.source),
            LocalRef::Assertion(id) => assertions
                .get(id)
                .and_then(|a| entries.get(&a.entry))
                .map(|entry| &entry.source),
            LocalRef::Form(id) => m
                .forms
                .iter()
                .find(|form| &form.id == id)
                .and_then(|form| entries.get(&form.entry))
                .map(|entry| &entry.source),
            LocalRef::EditorialSense(id) => m
                .editorial_senses
                .iter()
                .find(|sense| &sense.id == id)
                .and_then(|sense| revisions.get(&sense.revision))
                .map(|revision| &revision.source),
        }
    };
    for c in &m.components {
        if !collections.contains_key(&c.collection) {
            return Err(DictionaryError::InvalidDocument(
                "component collection missing".into(),
            ));
        }
    }
    let mut absent_capture_sources = BTreeMap::<CaptureRef, super::SourceId>::new();
    for r in &m.releases {
        let comp = comps
            .get(&r.component)
            .ok_or_else(|| DictionaryError::InvalidDocument("release component missing".into()))?;
        let source = &collections[&comp.collection].source;
        let mut capture_membership = BTreeSet::new();
        if r.captures
            .iter()
            .any(|cap| !capture_membership.insert(cap.clone()))
        {
            return Err(DictionaryError::InvalidDocument(
                "duplicate release capture".into(),
            ));
        }
        let mut assertion_membership = BTreeSet::new();
        if r.assertions
            .iter()
            .any(|id| !assertion_membership.insert(id.clone()))
        {
            return Err(DictionaryError::InvalidDocument(
                "duplicate release assertion".into(),
            ));
        }
        for cap in &r.captures {
            validate_capture_ref(cap)?;
            if r.availability != Availability::Available
                && !captures.contains_key(cap)
                && absent_capture_sources
                    .insert(cap.clone(), source.clone())
                    .is_some_and(|old| old != *source)
            {
                return Err(DictionaryError::InvalidDocument(
                    "unavailable capture has conflicting catalogue sources".into(),
                ));
            }
            if !bindings.contains_key(&(cap.namespace.clone(), cap.sha256.clone()))
                && r.availability == Availability::Available
            {
                return Err(DictionaryError::InvalidDocument(
                    "available release capture unbound".into(),
                ));
            }
            if let Some(bound) = bindings.get(&(cap.namespace.clone(), cap.sha256.clone())) {
                if bound != source {
                    return Err(DictionaryError::InvalidDocument(
                        "capture source mismatch".into(),
                    ));
                }
                if r.availability == Availability::Available && !captures.contains_key(cap) {
                    return Err(DictionaryError::InvalidDocument(
                        "available capture payload missing".into(),
                    ));
                }
            }
        }
        if r.availability == Availability::Available
            && (r.accounting.input_occurrences.is_none()
                || r.accounting.normalized_occurrences.is_none()
                || r.accounting.query_excluded_occurrences.is_none())
        {
            return Err(DictionaryError::InvalidDocument(
                "available accounting unknown".into(),
            ));
        }
        if r.redistribution == RedistributionStatus::Permitted
            && r.redistribution_evidence.is_empty()
        {
            return Err(DictionaryError::InvalidDocument(
                "permission lacks evidence".into(),
            ));
        }
    }
    for (namespace, sha256) in bindings.keys() {
        let reference = CaptureRef {
            namespace: namespace.clone(),
            sha256: sha256.clone(),
        };
        validate_capture_ref(&reference)?;
        if !captures.contains_key(&reference) && !absent_capture_sources.contains_key(&reference) {
            return Err(DictionaryError::InvalidDocument(
                "registry snapshot absent without a non-available catalogue release".into(),
            ));
        }
    }
    let mut asserted = BTreeSet::new();
    for a in &m.assertions {
        if !entries.contains_key(&a.entry) {
            return Err(DictionaryError::InvalidDocument(
                "assertion entry missing".into(),
            ));
        }
        let loaded = occurrences.contains(&a.occurrence);
        validate_capture_ref(&a.occurrence.capture)?;
        if a.occurrence.locator.is_empty() {
            return Err(DictionaryError::InvalidDocument(
                "empty assertion locator".into(),
            ));
        }
        let catalogue_membership = m.releases.iter().any(|r| {
            r.availability != Availability::Available
                && r.captures.contains(&a.occurrence.capture)
                && r.assertions.contains(&a.id)
        });
        if !loaded
            && (!absent_capture_sources.contains_key(&a.occurrence.capture)
                || !catalogue_membership)
        {
            return Err(DictionaryError::InvalidDocument(
                "assertion occurrence is neither loaded nor named by an unavailable release".into(),
            ));
        }
        if !asserted.insert(a.occurrence.clone()) {
            return Err(DictionaryError::InvalidDocument(
                "occurrence has multiple assertions".into(),
            ));
        }
        let src = bindings
            .get(&(
                a.occurrence.capture.namespace.clone(),
                a.occurrence.capture.sha256.clone(),
            ))
            .or_else(|| absent_capture_sources.get(&a.occurrence.capture))
            .ok_or_else(|| {
                DictionaryError::InvalidDocument(
                    "assertion capture has no source/catalogue binding".into(),
                )
            })?;
        if &entries[&a.entry].source != src {
            return Err(DictionaryError::InvalidDocument(
                "assertion entry source differs from capture source".into(),
            ));
        }
        if matches!(a.eligibility,Eligibility::Excluded{ref reason} if reason.is_empty()) {
            return Err(DictionaryError::InvalidDocument("empty exclusion".into()));
        }
    }
    if !occurrences.is_subset(&asserted) {
        return Err(DictionaryError::InvalidDocument(
            "loaded occurrence lacks normalized assertion".into(),
        ));
    }
    for r in &m.releases {
        if r.availability == Availability::Available {
            let listed: BTreeSet<_> = r.assertions.iter().cloned().collect();
            if listed.len() != r.assertions.len() {
                return Err(DictionaryError::InvalidDocument(
                    "duplicate release assertion".into(),
                ));
            }
            let comp = &comps[&r.component];
            let source = &collections[&comp.collection].source;
            let mut expected = BTreeSet::new();
            for id in &r.assertions {
                let a = assertions.get(id).ok_or_else(|| {
                    DictionaryError::InvalidDocument("release assertion missing".into())
                })?;
                if &entries[&a.entry].source != source {
                    return Err(DictionaryError::InvalidDocument(
                        "release assertion source mismatch".into(),
                    ));
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
                return Err(DictionaryError::InvalidDocument(
                    "available release occurrence membership mismatch".into(),
                ));
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
                return Err(DictionaryError::InvalidDocument(
                    "release accounting mismatch".into(),
                ));
            }
        }
    }
    for c in &m.contexts {
        let a = assertions
            .get(&c.assertion)
            .ok_or_else(|| DictionaryError::InvalidDocument("context assertion missing".into()))?;
        validate_evidence(doc, &absent_capture_sources, &c.evidence)?;
        if c.evidence.occurrence != a.occurrence {
            return Err(DictionaryError::InvalidDocument(
                "context evidence is not its assertion".into(),
            ));
        }
        if c.translated_from
            .as_ref()
            .is_some_and(|id| !m.contexts.iter().any(|x| &x.id == id))
        {
            return Err(DictionaryError::InvalidDocument(
                "translation context missing".into(),
            ));
        }
    }
    for f in &m.frames {
        let a = assertions
            .get(&f.assertion)
            .ok_or_else(|| DictionaryError::InvalidDocument("frame assertion missing".into()))?;
        validate_evidence(doc, &absent_capture_sources, &f.evidence)?;
        if f.evidence.occurrence != a.occurrence {
            return Err(DictionaryError::InvalidDocument(
                "frame evidence is not its assertion".into(),
            ));
        }
        unique_orders(f.slots.iter().map(|x| x.order))?;
    }
    for f in &m.forms {
        if !entries.contains_key(&f.entry) {
            return Err(DictionaryError::InvalidDocument(
                "form entry missing".into(),
            ));
        }
        match f.kind {
            FormKind::ComputedStem
            | FormKind::ComputedComponents
            | FormKind::GeneratedCandidate => {
                if f.computation.is_none() || f.evidence.is_some() {
                    return Err(DictionaryError::InvalidDocument(
                        "computed form provenance mismatch".into(),
                    ));
                }
            }
            _ => {
                if f.evidence.is_none() || f.computation.is_some() {
                    return Err(DictionaryError::InvalidDocument(
                        "source form provenance mismatch".into(),
                    ));
                }
            }
        }
        if let Some(e) = &f.evidence {
            validate_evidence(doc, &absent_capture_sources, e)?;
            let source = bindings
                .get(&(
                    e.occurrence.capture.namespace.clone(),
                    e.occurrence.capture.sha256.clone(),
                ))
                .or_else(|| absent_capture_sources.get(&e.occurrence.capture))
                .ok_or_else(|| {
                    DictionaryError::InvalidDocument("form evidence has no source".into())
                })?;
            if source != &entries[&f.entry].source {
                return Err(DictionaryError::InvalidDocument(
                    "form evidence belongs to another source".into(),
                ));
            }
        }
        if f.computation
            .as_ref()
            .is_some_and(|id| !m.computations.iter().any(|c| &c.id == id))
        {
            return Err(DictionaryError::InvalidDocument(
                "form computation missing".into(),
            ));
        }
        if f.components
            .iter()
            .filter_map(|c| c.target.as_ref())
            .any(|t| !local_exists(t))
        {
            return Err(DictionaryError::InvalidDocument(
                "form component target missing".into(),
            ));
        }
        unique_orders(f.components.iter().map(|x| x.order))?;
    }
    for e in &m.etymologies {
        let a = assertions.get(&e.assertion).ok_or_else(|| {
            DictionaryError::InvalidDocument("etymology assertion missing".into())
        })?;
        validate_evidence(doc, &absent_capture_sources, &e.evidence)?;
        if e.evidence.occurrence != a.occurrence {
            return Err(DictionaryError::InvalidDocument(
                "etymology evidence is not its assertion".into(),
            ));
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
            return Err(DictionaryError::InvalidDocument(
                "relation provenance mismatch".into(),
            ));
        }
        if let LocalRef::Entry(id) = &r.origin {
            if !entries.contains_key(id) {
                return Err(DictionaryError::InvalidDocument(
                    "relation origin entry missing".into(),
                ));
            }
        } else if !local_exists(&r.origin) {
            return Err(DictionaryError::InvalidDocument(
                "relation origin missing".into(),
            ));
        }
        if let RelationTarget::Local(target) = &r.target
            && !local_exists(target)
        {
            return Err(DictionaryError::InvalidDocument(
                "local relation target missing".into(),
            ));
        }
        if r.editorial_revision
            .as_ref()
            .is_some_and(|id| !revisions.contains_key(id))
        {
            return Err(DictionaryError::InvalidDocument(
                "editorial relation revision missing".into(),
            ));
        }
        if let Some(e) = &r.evidence {
            validate_evidence(doc, &absent_capture_sources, e)?;
            let evidence_source = bindings
                .get(&(
                    e.occurrence.capture.namespace.clone(),
                    e.occurrence.capture.sha256.clone(),
                ))
                .or_else(|| absent_capture_sources.get(&e.occurrence.capture));
            if evidence_source != local_source(&r.origin) {
                return Err(DictionaryError::InvalidDocument(
                    "relation evidence belongs to another source".into(),
                ));
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
            if !releases.contains_key(&t.release) {
                return Err(DictionaryError::InvalidDocument(
                    "catalogue release missing".into(),
                ));
            }
            let release = releases[&t.release];
            let component = &comps[&release.component];
            if collections[&component.collection].source != t.source {
                return Err(DictionaryError::InvalidDocument(
                    "catalogue target source differs from release".into(),
                ));
            }
            if t.occurrence
                .as_ref()
                .is_some_and(|o| !release.captures.contains(&o.capture))
            {
                return Err(DictionaryError::InvalidDocument(
                    "catalogue occurrence is outside target release".into(),
                ));
            }
            if let Some(occurrence) = &t.occurrence {
                validate_capture_ref(&occurrence.capture)?;
                if release.availability == Availability::Available
                    && !occurrences.contains(occurrence)
                {
                    return Err(DictionaryError::InvalidDocument(
                        "available catalogue occurrence is not loaded".into(),
                    ));
                }
            }
        }
    }
    for r in &m.owned_revisions {
        if !matches!(
            reg.sources
                .iter()
                .find(|s| s.id == r.source)
                .map(|s| s.origin),
            Some(super::SourceOrigin::OwnedDraft)
        ) {
            return Err(DictionaryError::InvalidDocument(
                "revision source is not owned".into(),
            ));
        }
        if r.parent
            .as_ref()
            .is_some_and(|p| !revisions.contains_key(p))
        {
            return Err(DictionaryError::InvalidDocument(
                "revision parent missing".into(),
            ));
        }
        if let Some(parent) = &r.parent
            && revisions[parent].source != r.source
        {
            return Err(DictionaryError::InvalidDocument(
                "revision parent belongs to another source".into(),
            ));
        }
        let mut assertion_ids = BTreeSet::new();
        for a in &r.assertions {
            if !assertion_ids.insert(a.clone())
                || !assertions.contains_key(a)
                || entries[&assertions[a].entry].source != r.source
            {
                return Err(DictionaryError::InvalidDocument(
                    "owned assertion is missing, repeated, or belongs to another source".into(),
                ));
            }
        }
        let mut ancestor_assertions = BTreeSet::new();
        let mut ancestor = r.parent.clone();
        let mut ancestor_seen = BTreeSet::new();
        while let Some(parent_id) = ancestor {
            if !ancestor_seen.insert(parent_id.clone()) || parent_id == r.id {
                return Err(DictionaryError::InvalidDocument(
                    "owned revision cycle".into(),
                ));
            }
            let parent = revisions[&parent_id];
            ancestor_assertions.extend(parent.assertions.iter().cloned());
            ancestor = parent.parent.clone();
        }
        let mut tombstones = BTreeSet::new();
        for a in &r.tombstones {
            if !tombstones.insert(a.clone())
                || !assertions.contains_key(a)
                || r.assertions.contains(a)
                || !ancestor_assertions.contains(a)
            {
                return Err(DictionaryError::InvalidDocument("invalid tombstone".into()));
            }
        }
    }
    for r in &m.owned_revisions {
        let mut visited = BTreeSet::new();
        let mut current = Some(&r.id);
        while let Some(id) = current {
            if !visited.insert(id.clone()) {
                return Err(DictionaryError::InvalidDocument(
                    "owned revision cycle".into(),
                ));
            }
            current = revisions[id].parent.as_ref();
        }
    }
    for h in &m.owned_heads {
        if revisions
            .get(&h.revision)
            .is_none_or(|r| r.source != h.source)
        {
            return Err(DictionaryError::InvalidDocument(
                "invalid owned head".into(),
            ));
        }
    }
    for e in &m.editorial_senses {
        if !revisions.contains_key(&e.revision)
            || e.assertions.iter().any(|a| !assertions.contains_key(a))
        {
            return Err(DictionaryError::InvalidDocument(
                "invalid editorial sense".into(),
            ));
        }
    }
    for c in &m.computations {
        if c.tool.is_empty() || c.tool_version.is_empty() || c.policy.is_empty() {
            return Err(DictionaryError::InvalidDocument(
                "incomplete computation provenance".into(),
            ));
        }
        if c.inputs
            .iter()
            .any(|i| matches!(i, InputRef::Entity(local) if !local_exists(local)))
        {
            return Err(DictionaryError::InvalidDocument(
                "computation input target missing".into(),
            ));
        }
    }
    for a in &m.annotations {
        if !m.computations.iter().any(|c| c.id == a.computation) || !local_exists(&a.target) {
            return Err(DictionaryError::InvalidDocument(
                "annotation computation or target missing".into(),
            ));
        }
    }
    for x in &m.extensions {
        if !ids.contains(&x.owner) {
            return Err(DictionaryError::InvalidDocument(
                "extension owner missing".into(),
            ));
        }
        if let Some(e) = &x.evidence {
            validate_evidence(doc, &absent_capture_sources, e)?;
        }
    }
    if let Some(raw) = &doc.legacy_archive {
        let lex = Lexicon::from_archive_bytes(raw)?;
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
            let rows: Vec<_> = lex
                .records()
                .iter()
                .filter(|row| row.identity().snapshot() == s)
                .collect();
            if cap.occurrences.len() != rows.len() {
                return Err(DictionaryError::InvalidDocument(
                    "legacy occurrence set changed".into(),
                ));
            }
            for row in rows {
                let found = cap
                    .occurrences
                    .iter()
                    .find(|o| o.locator == row.identity().locator())
                    .ok_or_else(|| {
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
    let _ = forms;
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
    doc: &DictionaryDocument,
    absent_capture_sources: &BTreeMap<CaptureRef, super::SourceId>,
    e: &SourceEvidence,
) -> Result<(), DictionaryError> {
    let occ = doc
        .captures
        .iter()
        .find(|c| c.identity == e.occurrence.capture)
        .and_then(|c| {
            c.occurrences
                .iter()
                .find(|o| o.locator == e.occurrence.locator)
        });
    let Some(occ) = occ else {
        if !absent_capture_sources.contains_key(&e.occurrence.capture) {
            return Err(DictionaryError::InvalidDocument(
                "evidence occurrence is neither loaded nor catalogued unavailable".into(),
            ));
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
