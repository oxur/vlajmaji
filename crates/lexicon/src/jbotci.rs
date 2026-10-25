//! Lossless adapter for the pinned jbotci Lensisku export.
//!
//! The adapter keeps the caller's byte snapshots and per-row native lexemes as
//! the authority. Typed records are a conservative, rebuildable projection.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use serde::de::{Deserializer as _, Error as _, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;

use crate::dictionary::{
    AssertionRecord, Availability, CaptureMetadata, CaptureRef, CodeNotice, CollectionRecord,
    ComponentRecord, ContentNotice, ContextKind, ContextRecord, Dictionary, DictionaryDocument,
    DictionaryError, DictionaryMetadata, Eligibility, EntityId, EntryRecord, EtymologyRecord,
    ExtensionRecord, FormKind, FormRecord, NativeEncoding, NativeField, OccurrenceRef, Presence,
    RawCapture, RawJson, RawOccurrence, RedistributionStatus, ReleaseAccounting, ReleaseDiagnostic,
    ReleaseRecord, SourceEvidence, TextValue,
};
use crate::registry::{
    AbbreviationKind, ContributorId, ContributorRecord, DistributorId, DistributorRecord,
    LineageTarget, OccurrenceAttribution, RegistryDocument, RegistryError, SnapshotAttribution,
    SourceId, SourceLineage, SourceOrigin, SourceRecord, SourceRegistry,
};
use crate::{SnapshotIdentity, TimePrecision};

/// Supplier repository revision for the local export being normalized.
pub const JBOTCI_PINNED_REVISION: &str = "3d9fd41d9ed32de26311ee27ec86b6b165985aa1";
/// SHA-256 of the complete pinned Lensisku JSON export.
pub const JBOTCI_PINNED_RAW_SHA256: &str =
    "12e826751c3e50c10dd8df1adafb5f9eb32cdf542b22c4244f284b59d3824dde";
/// SHA-256 of the complete pinned supplier metadata file.
pub const JBOTCI_PINNED_METADATA_SHA256: &str =
    "fe101eead51e0be1b885f3ee1b4555136e333e839baa250ddbd7b84c9af5fc98";

/// Caller-supplied pins for one import. Defaults name the committed local snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JbotciImportOptions {
    /// Exact forty-character distributor Git revision.
    pub revision: String,
    /// Expected full raw-export SHA-256.
    pub expected_raw_sha256: String,
    /// Expected full metadata SHA-256.
    pub expected_metadata_sha256: String,
}
impl Default for JbotciImportOptions {
    fn default() -> Self {
        Self {
            revision: JBOTCI_PINNED_REVISION.into(),
            expected_raw_sha256: JBOTCI_PINNED_RAW_SHA256.into(),
            expected_metadata_sha256: JBOTCI_PINNED_METADATA_SHA256.into(),
        }
    }
}

/// Stable public summary of an import, separate from the complete dictionary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JbotciImportReport {
    /// Report schema version.
    pub version: u32,
    /// Distributor revision used for this normalized release.
    pub revision: String,
    /// Full raw-export digest.
    pub raw_sha256: String,
    /// Full supplier-metadata digest.
    pub metadata_sha256: String,
    /// Exact input byte counts.
    pub raw_bytes: u64,
    /// Exact metadata byte count.
    pub metadata_bytes: u64,
    /// Number of raw source-array rows.
    pub input_rows: u64,
    /// Number of ordered top-level native fields.
    pub native_fields: u64,
    /// Number of normalized assertion rows (one per raw row).
    pub normalized_assertions: u64,
    /// Number of rows eligible for the lexical projection.
    pub lexical_occurrences: u64,
    /// Number of excluded rows retained in the release.
    pub excluded_occurrences: u64,
    /// Number of distinct exact spellings among eligible rows.
    pub eligible_spellings: u64,
    /// Number of distinct string spellings among all rows, including empty.
    pub all_spellings: u64,
    /// Number of distinct source-scoped contributors.
    pub contributors: u64,
    /// Supplier metadata definition count.
    pub supplier_definition_count: u64,
    /// Supplier metadata selected entry count.
    pub supplier_entry_count: u64,
    /// Counts of each valid raw word_type string.
    pub word_types: BTreeMap<String, u64>,
    /// Counts of each top-level native field name.
    pub field_counts: BTreeMap<String, u64>,
    /// Per-code diagnostic counts, not excluded-row counts.
    pub diagnostics: BTreeMap<String, u64>,
    /// Whether the supplier definition count equals the observed raw-row count.
    pub definition_count_matches: bool,
    /// Whether the supplier entry count equals observed distinct eligible spellings.
    pub entry_count_matches: bool,
}

/// Immutable import result; callers can inspect or consume the validated model.
#[derive(Debug, Clone)]
pub struct JbotciImport {
    dictionary: Dictionary,
    report: JbotciImportReport,
}
impl JbotciImport {
    /// Borrow the fully validated dictionary.
    pub fn dictionary(&self) -> &Dictionary {
        &self.dictionary
    }
    /// Borrow the deterministic population and diagnostic report.
    pub fn report(&self) -> &JbotciImportReport {
        &self.report
    }
    /// Consume the result and return its validated dictionary.
    pub fn into_dictionary(self) -> Dictionary {
        self.dictionary
    }
}

/// Typed failures at the raw-input, metadata, registry, or model boundary.
#[derive(Debug)]
#[non_exhaustive]
pub enum JbotciImportError {
    /// A revision or expected digest did not follow its exact grammar.
    InvalidOptions(String),
    /// An input digest did not match its caller-supplied pin.
    HashMismatch {
        /// Name of the mismatched input (`raw` or `metadata`).
        input: String,
        /// Expected SHA-256 supplied by the caller.
        expected: String,
        /// SHA-256 computed over the complete supplied bytes.
        actual: String,
    },
    /// An input was not UTF-8.
    InvalidUtf8 {
        /// Name of the invalid input (`raw` or `metadata`).
        input: String,
    },
    /// Raw JSON syntax or root shape was invalid.
    InvalidJson(String),
    /// Supplier metadata was malformed, incomplete, or duplicated.
    InvalidMetadata(String),
    /// Supplier metadata was valid but outside the supported bounded format.
    UnsupportedMetadata(String),
    /// The source registry rejected a constructed attribution graph.
    Registry(RegistryError),
    /// The normalized model rejected the candidate graph.
    Dictionary(DictionaryError),
}
impl fmt::Display for JbotciImportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidOptions(s) => write!(f, "invalid import options: {s}"),
            Self::HashMismatch {
                input,
                expected,
                actual,
            } => write!(
                f,
                "{input} SHA-256 mismatch: expected {expected}, got {actual}"
            ),
            Self::InvalidUtf8 { input } => write!(f, "{input} input is not UTF-8"),
            Self::InvalidJson(s) => write!(f, "invalid raw JSON: {s}"),
            Self::InvalidMetadata(s) => write!(f, "invalid supplier metadata: {s}"),
            Self::UnsupportedMetadata(s) => write!(f, "unsupported supplier metadata: {s}"),
            Self::Registry(e) => write!(f, "invalid source registry: {e}"),
            Self::Dictionary(e) => write!(f, "invalid dictionary: {e}"),
        }
    }
}
impl std::error::Error for JbotciImportError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Registry(e) => Some(e),
            Self::Dictionary(e) => Some(e),
            _ => None,
        }
    }
}
impl From<RegistryError> for JbotciImportError {
    fn from(value: RegistryError) -> Self {
        Self::Registry(value)
    }
}
impl From<DictionaryError> for JbotciImportError {
    fn from(value: DictionaryError) -> Self {
        Self::Dictionary(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum MetaValue {
    String(String),
    Boolean(bool),
    Integer(u64),
}
#[derive(Debug, Clone)]
struct Metadata {
    values: BTreeMap<String, MetaValue>,
    unknown_keys: Vec<String>,
}
impl Metadata {
    fn string(&self, key: &str) -> Result<&str, JbotciImportError> {
        match self.values.get(key) {
            Some(MetaValue::String(v)) => Ok(v),
            Some(_) => Err(JbotciImportError::InvalidMetadata(format!(
                "{key} must be a string"
            ))),
            None => Err(JbotciImportError::InvalidMetadata(format!(
                "missing required key {key}"
            ))),
        }
    }
    fn integer(&self, key: &str) -> Result<u64, JbotciImportError> {
        match self.values.get(key) {
            Some(MetaValue::Integer(v)) => Ok(*v),
            Some(_) => Err(JbotciImportError::InvalidMetadata(format!(
                "{key} must be a nonnegative integer"
            ))),
            None => Err(JbotciImportError::InvalidMetadata(format!(
                "missing required key {key}"
            ))),
        }
    }
    fn boolean(&self, key: &str) -> Result<bool, JbotciImportError> {
        match self.values.get(key) {
            Some(MetaValue::Boolean(v)) => Ok(*v),
            Some(_) => Err(JbotciImportError::InvalidMetadata(format!(
                "{key} must be a boolean"
            ))),
            None => Err(JbotciImportError::InvalidMetadata(format!(
                "missing required key {key}"
            ))),
        }
    }
}

fn parse_metadata(bytes: &[u8]) -> Result<Metadata, JbotciImportError> {
    let text = std::str::from_utf8(bytes).map_err(|_| JbotciImportError::InvalidUtf8 {
        input: "metadata".into(),
    })?;
    let mut values = BTreeMap::new();
    let mut unknown_keys = Vec::new();
    for (line_index, original) in text.lines().enumerate() {
        let line = original.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, rhs)) = line.split_once('=') else {
            return Err(JbotciImportError::InvalidMetadata(format!(
                "line {} lacks '='",
                line_index + 1
            )));
        };
        let key = key.trim();
        if key.is_empty() {
            return Err(JbotciImportError::InvalidMetadata(format!(
                "line {} has an empty key",
                line_index + 1
            )));
        }
        if !key.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_') {
            return Err(JbotciImportError::UnsupportedMetadata(format!(
                "line {} has a non-bare key",
                line_index + 1
            )));
        }
        if values.contains_key(key) {
            return Err(JbotciImportError::InvalidMetadata(format!(
                "duplicate key {key}"
            )));
        }
        let rhs = rhs.trim();
        if rhs.is_empty() {
            return Err(JbotciImportError::InvalidMetadata(format!(
                "line {} has an empty value",
                line_index + 1
            )));
        }
        let value = if rhs.starts_with('"') {
            serde_json::from_str::<String>(rhs)
                .map(MetaValue::String)
                .map_err(|e| {
                    JbotciImportError::InvalidMetadata(format!("line {}: {e}", line_index + 1))
                })?
        } else if rhs == "true" {
            MetaValue::Boolean(true)
        } else if rhs == "false" {
            MetaValue::Boolean(false)
        } else if rhs.bytes().all(|b| b.is_ascii_digit()) {
            MetaValue::Integer(rhs.parse().map_err(|_| {
                JbotciImportError::InvalidMetadata(format!(
                    "line {} integer overflow",
                    line_index + 1
                ))
            })?)
        } else if serde_json::from_str::<serde_json::Number>(rhs).is_ok() {
            return Err(JbotciImportError::UnsupportedMetadata(format!(
                "line {} uses a valid JSON number outside strings/booleans/nonnegative integers",
                line_index + 1
            )));
        } else if rhs.starts_with("true")
            || rhs.starts_with("false")
            || rhs.bytes().next().is_some_and(|b| b.is_ascii_digit())
        {
            return Err(JbotciImportError::InvalidMetadata(format!(
                "line {} has trailing or malformed literal text",
                line_index + 1
            )));
        } else {
            return Err(JbotciImportError::UnsupportedMetadata(format!(
                "line {} uses a value outside strings/booleans/nonnegative integers",
                line_index + 1
            )));
        };
        if !matches!(
            key,
            "language_tag"
                | "language_realname"
                | "source_language_tag"
                | "format"
                | "positive_scores_only"
                | "filename"
                | "metadata_url"
                | "download_url"
                | "lensisku_created_at"
                | "sha256"
                | "definition_count"
                | "entry_count"
        ) {
            unknown_keys.push(key.to_owned());
        }
        values.insert(key.to_owned(), value);
    }
    let parsed = Metadata {
        values,
        unknown_keys,
    };
    for key in [
        "language_tag",
        "language_realname",
        "source_language_tag",
        "format",
        "filename",
        "metadata_url",
        "download_url",
        "lensisku_created_at",
        "sha256",
    ] {
        parsed.string(key)?;
    }
    for key in ["definition_count", "entry_count"] {
        parsed.integer(key)?;
    }
    parsed.boolean("positive_scores_only")?;
    for (key, want) in [
        ("language_tag", "en"),
        ("source_language_tag", "jbo"),
        ("format", "json"),
        ("filename", "dictionary-en.json"),
    ] {
        if parsed.string(key)? != want {
            return Err(JbotciImportError::UnsupportedMetadata(format!(
                "{key} must be {want:?}"
            )));
        }
    }
    if parsed.boolean("positive_scores_only")? {
        return Err(JbotciImportError::UnsupportedMetadata(
            "positive_scores_only must be false".into(),
        ));
    }
    Ok(parsed)
}

fn valid_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn sha256(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(bytes);
    let mut out = String::with_capacity(64);
    for byte in digest {
        use fmt::Write as _;
        let _ = write!(out, "{byte:02x}");
    }
    out
}

#[derive(Debug, Clone)]
struct Field {
    name: String,
    raw: String,
}
struct FieldVisitor;
impl<'de> Visitor<'de> for FieldVisitor {
    type Value = Vec<Field>;
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a JSON object")
    }
    fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Self::Value, M::Error> {
        let mut fields = Vec::new();
        while let Some(name) = map.next_key::<String>()? {
            let raw = map.next_value::<Box<RawValue>>()?;
            fields.push(Field {
                name,
                raw: raw.get().to_owned(),
            });
        }
        Ok(fields)
    }
}
fn parse_fields(raw: &str) -> Result<Vec<Field>, String> {
    let mut de = serde_json::Deserializer::from_str(raw);
    let fields = de
        .deserialize_map(FieldVisitor)
        .map_err(|e| e.to_string())?;
    de.end().map_err(|e| e.to_string())?;
    Ok(fields)
}
struct DuplicateVisitor;
impl<'de> Visitor<'de> for DuplicateVisitor {
    type Value = Vec<String>;
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a JSON object or array")
    }
    fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Self::Value, M::Error> {
        let mut seen = BTreeSet::new();
        let mut duplicates = Vec::new();
        while let Some(key) = map.next_key::<String>()? {
            if !seen.insert(key.clone()) {
                duplicates.push(key.clone());
            }
            let value = map.next_value::<Box<RawValue>>()?;
            duplicates.extend(scan_nested_duplicates(value.get()).map_err(M::Error::custom)?);
        }
        Ok(duplicates)
    }
    fn visit_seq<S: SeqAccess<'de>>(self, mut seq: S) -> Result<Self::Value, S::Error> {
        let mut duplicates = Vec::new();
        while let Some(value) = seq.next_element::<Box<RawValue>>()? {
            duplicates.extend(scan_nested_duplicates(value.get()).map_err(S::Error::custom)?);
        }
        Ok(duplicates)
    }
}
fn scan_nested_duplicates(raw: &str) -> Result<Vec<String>, String> {
    match raw.trim_start().chars().next() {
        Some('{') => {
            let mut de = serde_json::Deserializer::from_str(raw);
            let found = de
                .deserialize_map(DuplicateVisitor)
                .map_err(|e| e.to_string())?;
            de.end().map_err(|e| e.to_string())?;
            Ok(found)
        }
        Some('[') => {
            let mut de = serde_json::Deserializer::from_str(raw);
            let found = de
                .deserialize_seq(DuplicateVisitor)
                .map_err(|e| e.to_string())?;
            de.end().map_err(|e| e.to_string())?;
            Ok(found)
        }
        _ => Ok(Vec::new()),
    }
}
fn as_json_string(raw: &str) -> Option<String> {
    serde_json::from_str(raw).ok()
}
fn occurrence_id(
    namespace: &str,
    digest: &str,
    locator: &str,
) -> Result<crate::RecordIdentity, JbotciImportError> {
    let wire =
        serde_json::json!({"snapshot":{"namespace":namespace,"sha256":digest},"locator":locator});
    serde_json::from_value(wire)
        .map_err(|e| JbotciImportError::InvalidJson(format!("internal occurrence identity: {e}")))
}
fn snapshot_id(namespace: &str, digest: &str) -> Result<SnapshotIdentity, JbotciImportError> {
    let wire = serde_json::json!({"namespace":namespace,"sha256":digest});
    serde_json::from_value(wire)
        .map_err(|e| JbotciImportError::InvalidJson(format!("internal capture identity: {e}")))
}
fn eid(value: String) -> Result<EntityId, JbotciImportError> {
    EntityId::new(value)
        .map_err(|e| JbotciImportError::Dictionary(DictionaryError::InvalidId(e.to_string())))
}
fn source_id(value: &str) -> Result<SourceId, JbotciImportError> {
    SourceId::new(value).map_err(JbotciImportError::Registry)
}
fn text_value(text: String, language: &str) -> TextValue {
    TextValue {
        text,
        language: Presence::Value(language.into()),
        markup: Presence::Missing,
    }
}
fn source_evidence(occurrence: OccurrenceRef, field_index: Option<usize>) -> SourceEvidence {
    SourceEvidence {
        occurrence,
        field_index,
        citation: Presence::Missing,
        offset: None,
    }
}

#[derive(Debug)]
struct Row {
    fields: Vec<Field>,
    duplicates: Vec<String>,
    is_object: bool,
}
fn field_index(fields: &[Field], name: &str) -> Option<usize> {
    fields.iter().position(|f| f.name == name)
}
fn field<'a>(fields: &'a [Field], name: &str) -> Option<&'a Field> {
    field_index(fields, name).and_then(|i| fields.get(i))
}
fn presence_string(fields: &[Field], name: &str) -> Presence<String> {
    match field(fields, name) {
        None => Presence::Missing,
        Some(f) if f.raw == "null" => Presence::Null,
        Some(f) => as_json_string(&f.raw).map_or(Presence::Missing, Presence::Value),
    }
}
fn presence_raw(fields: &[Field], name: &str) -> Result<Presence<RawJson>, JbotciImportError> {
    match field(fields, name) {
        None => Ok(Presence::Missing),
        Some(f) if f.raw == "null" => Ok(Presence::Null),
        Some(f) => RawJson::new(f.raw.clone())
            .map(Presence::Value)
            .map_err(JbotciImportError::Dictionary),
    }
}
fn unique_field_index(fields: &[Field], name: &str) -> Option<usize> {
    let mut found = None;
    for (index, f) in fields.iter().enumerate() {
        if f.name == name {
            if found.is_some() {
                return None;
            }
            found = Some(index);
        }
    }
    found
}
fn add_diag(
    report: &mut BTreeMap<String, u64>,
    rows: &mut Vec<ReleaseDiagnostic>,
    code: &str,
    message: String,
    occurrence: Option<OccurrenceRef>,
) {
    *report.entry(code.to_owned()).or_default() += 1;
    rows.push(ReleaseDiagnostic {
        occurrence,
        code: code.to_owned(),
        message,
    });
}
fn raw_array(raw: &str) -> Option<Vec<Box<RawValue>>> {
    serde_json::from_str(raw).ok()
}
fn user_name(fields: &[Field]) -> Option<String> {
    let user = field(fields, "user")?;
    let nested = parse_fields(&user.raw).ok()?;
    if nested.iter().filter(|f| f.name == "username").count() != 1 {
        return None;
    }
    as_json_string(&field(&nested, "username")?.raw)
}
fn contributor_key(username: &str) -> String {
    format!("lensisku-user-{}", sha256(username.as_bytes()))
}
fn json_array_of_strings(raw: &str) -> Option<Vec<String>> {
    serde_json::from_str::<Vec<String>>(raw).ok()
}
fn forms_from_field(raw: &str) -> Option<Vec<String>> {
    if let Some(s) = as_json_string(raw) {
        Some(s.split_whitespace().map(str::to_owned).collect())
    } else {
        json_array_of_strings(raw)
    }
}
fn extension(
    owner: &EntityId,
    name: String,
    raw: &str,
    evidence: Option<SourceEvidence>,
) -> Result<ExtensionRecord, JbotciImportError> {
    Ok(ExtensionRecord {
        owner: owner.clone(),
        name,
        value: RawJson::new(raw.to_owned())?,
        evidence,
    })
}
fn context(
    id: EntityId,
    assertion: &EntityId,
    kind: ContextKind,
    value: Presence<TextValue>,
    occurrence: &OccurrenceRef,
    field_index: usize,
) -> ContextRecord {
    ContextRecord {
        id,
        assertion: assertion.clone(),
        kind,
        value,
        translated_from: None,
        evidence: source_evidence(occurrence.clone(), Some(field_index)),
    }
}

/// Normalize a pinned Lensisku export without opening files or changing caller state.
///
/// The full original bytes and ordered native fields are retained in the returned
/// dictionary. Malformed JSON or metadata is rejected before a result is created;
/// valid unsupported rows remain as excluded occurrences with diagnostics.
///
/// # Errors
///
/// Returns a typed error when pins/options, UTF-8, JSON, metadata, registry
/// attribution, or the complete dictionary model is invalid.
pub fn import_jbotci_export(
    raw: &[u8],
    metadata_bytes: &[u8],
    options: &JbotciImportOptions,
) -> Result<JbotciImport, JbotciImportError> {
    if !valid_hex(&options.revision, 40) {
        return Err(JbotciImportError::InvalidOptions(
            "revision must be 40 lowercase hexadecimal characters".into(),
        ));
    }
    if !valid_hex(&options.expected_raw_sha256, 64) {
        return Err(JbotciImportError::InvalidOptions(
            "expected raw SHA-256 must be 64 lowercase hexadecimal characters".into(),
        ));
    }
    if !valid_hex(&options.expected_metadata_sha256, 64) {
        return Err(JbotciImportError::InvalidOptions(
            "expected metadata SHA-256 must be 64 lowercase hexadecimal characters".into(),
        ));
    }
    let raw_sha256 = sha256(raw);
    if raw_sha256 != options.expected_raw_sha256 {
        return Err(JbotciImportError::HashMismatch {
            input: "raw".into(),
            expected: options.expected_raw_sha256.clone(),
            actual: raw_sha256,
        });
    }
    let metadata_sha256 = sha256(metadata_bytes);
    if metadata_sha256 != options.expected_metadata_sha256 {
        return Err(JbotciImportError::HashMismatch {
            input: "metadata".into(),
            expected: options.expected_metadata_sha256.clone(),
            actual: metadata_sha256,
        });
    }
    let metadata = parse_metadata(metadata_bytes)?;
    if metadata.string("sha256")? != options.expected_raw_sha256 {
        return Err(JbotciImportError::InvalidMetadata(
            "sha256 must equal the pinned raw-export digest".into(),
        ));
    }
    let raw_text = std::str::from_utf8(raw).map_err(|_| JbotciImportError::InvalidUtf8 {
        input: "raw".into(),
    })?;
    let raw_rows: Vec<Box<RawValue>> = serde_json::from_str(raw_text)
        .map_err(|e| JbotciImportError::InvalidJson(e.to_string()))?;

    let mut rows = Vec::with_capacity(raw_rows.len());
    let mut field_counts = BTreeMap::<String, u64>::new();
    let mut word_types = BTreeMap::<String, u64>::new();
    let mut all_spellings = BTreeSet::new();
    let mut eligible_spellings = BTreeSet::new();
    let mut users = BTreeSet::new();
    let mut native_fields = 0_u64;
    let mut nested_duplicate_rows = 0_u64;
    for raw_row in &raw_rows {
        let text = raw_row.get();
        let is_object = text.trim_start().starts_with('{');
        let (fields, duplicates) = if is_object {
            let fields = parse_fields(text).map_err(JbotciImportError::InvalidJson)?;
            let duplicates =
                scan_nested_duplicates(text).map_err(JbotciImportError::InvalidJson)?;
            (fields, duplicates)
        } else {
            (
                vec![Field {
                    name: "$row".into(),
                    raw: text.to_owned(),
                }],
                Vec::new(),
            )
        };
        native_fields += fields.len() as u64;
        for f in &fields {
            *field_counts.entry(f.name.clone()).or_default() += 1;
        }
        if !duplicates.is_empty() {
            nested_duplicate_rows += 1;
        }
        for value in fields
            .iter()
            .filter(|f| f.name == "word_type")
            .filter_map(|f| as_json_string(&f.raw))
        {
            *word_types.entry(value).or_default() += 1;
        }
        for value in fields
            .iter()
            .filter(|f| f.name == "word")
            .filter_map(|f| as_json_string(&f.raw))
        {
            all_spellings.insert(value);
        }
        if duplicates.is_empty()
            && let (Some(word), Some(class), Some(definition)) = (
                field(&fields, "word").and_then(|f| as_json_string(&f.raw)),
                field(&fields, "word_type").and_then(|f| as_json_string(&f.raw)),
                field(&fields, "definition").and_then(|f| as_json_string(&f.raw)),
            )
            && is_supported_class(&class)
            && !word.is_empty()
            && !definition.trim().is_empty()
        {
            eligible_spellings.insert(word);
        }
        if duplicates.is_empty()
            && let Some(username) = user_name(&fields)
        {
            users.insert(username);
        }
        rows.push(Row {
            fields,
            duplicates,
            is_object,
        });
    }

    let mut report_diagnostics = BTreeMap::<String, u64>::new();
    let mut release_diagnostics = Vec::<ReleaseDiagnostic>::new();
    for unknown in &metadata.unknown_keys {
        add_diag(
            &mut report_diagnostics,
            &mut release_diagnostics,
            "metadata_unknown_key",
            format!("unsupported metadata key {unknown:?} retained"),
            None,
        );
    }
    let supplier_definition_count = metadata.integer("definition_count")?;
    let supplier_entry_count = metadata.integer("entry_count")?;
    let definition_count_matches = supplier_definition_count == rows.len() as u64;
    let entry_count_matches = supplier_entry_count == eligible_spellings.len() as u64;
    if !definition_count_matches {
        add_diag(
            &mut report_diagnostics,
            &mut release_diagnostics,
            "definition_count_mismatch",
            format!(
                "supplier definition_count {supplier_definition_count} differs from {} raw rows",
                rows.len()
            ),
            None,
        );
    }
    if !entry_count_matches {
        add_diag(
            &mut report_diagnostics,
            &mut release_diagnostics,
            "entry_count_mismatch",
            format!(
                "supplier entry_count {supplier_entry_count} differs from {} exact eligible spellings",
                eligible_spellings.len()
            ),
            None,
        );
    }

    let raw_namespace = "captured:lensisku:jbotci:en";
    let metadata_namespace = "captured:lensisku:jbotci:en:metadata";
    let source = source_id("lensisku")?;
    let distributor = DistributorId::new("jbotci").map_err(JbotciImportError::Registry)?;
    let raw_capture_ref = CaptureRef {
        namespace: raw_namespace.into(),
        sha256: raw_sha256.clone(),
    };
    let metadata_capture_ref = CaptureRef {
        namespace: metadata_namespace.into(),
        sha256: metadata_sha256.clone(),
    };
    let mut registry_doc = RegistryDocument::default();
    registry_doc.sources.push(SourceRecord {
        id: source.clone(),
        display_name: "Lensisku".into(),
        abbreviation: "LSK".into(),
        abbreviation_kind: AbbreviationKind::ProjectAssigned,
        abbreviation_evidence: None,
        origin: SourceOrigin::Upstream,
    });
    registry_doc.distributors.push(DistributorRecord {
        id: distributor.clone(),
        display_name: "jbotci".into(),
    });
    registry_doc.lineage.push(SourceLineage {
        source: source.clone(),
        upstream: LineageTarget::Unresolved("https://jbovlaste.lojban.org/".into()),
        evidence: Some(JBOTCI_NOTICE_LOCATOR.into()),
    });
    let mut contributor_ids = BTreeMap::<String, ContributorId>::new();
    for username in &users {
        let id_text = contributor_key(username);
        let id = ContributorId::new(id_text).map_err(JbotciImportError::Registry)?;
        let display_valid = username.trim() == username
            && !username.is_empty()
            && !username.chars().any(char::is_control);
        let display_name = if display_valid {
            username.clone()
        } else {
            add_diag(
                &mut report_diagnostics,
                &mut release_diagnostics,
                "contributor_display_fallback",
                format!(
                    "username {:?} is not a registry display value; retained under its source-scoped hash identity",
                    username
                ),
                None,
            );
            format!(
                "Lensisku contributor {}",
                &sha256(username.as_bytes())[..16]
            )
        };
        registry_doc.contributors.push(ContributorRecord {
            id: id.clone(),
            display_name,
        });
        contributor_ids.insert(username.clone(), id);
    }
    let raw_snapshot = snapshot_id(raw_namespace, &raw_sha256)?;
    let meta_snapshot = snapshot_id(metadata_namespace, &metadata_sha256)?;
    for snapshot in [&raw_snapshot, &meta_snapshot] {
        registry_doc.snapshots.push(SnapshotAttribution {
            snapshot: snapshot.clone(),
            source: source.clone(),
            distributor: Some(distributor.clone()),
        });
    }
    for (index, row) in rows.iter().enumerate() {
        let occurrence = occurrence_id(raw_namespace, &raw_sha256, &format!("/{index}"))?;
        let native_id = if row.duplicates.is_empty() {
            field(&row.fields, "definition_id").and_then(|f| {
                if as_json_string(&f.raw).is_some()
                    || serde_json::from_str::<serde_json::Number>(&f.raw).is_ok()
                {
                    Some(f.raw.clone())
                } else {
                    None
                }
            })
        } else {
            None
        };
        let contributor = if row.duplicates.is_empty() {
            user_name(&row.fields).and_then(|n| contributor_ids.get(&n).cloned())
        } else {
            None
        };
        registry_doc.occurrences.push(OccurrenceAttribution {
            occurrence,
            contributor,
            native_entry_id: None,
            native_assertion_id: native_id,
        });
    }
    let registry = SourceRegistry::from_document(registry_doc)?;

    build_import(
        raw,
        metadata_bytes,
        raw_sha256,
        metadata_sha256,
        raw_namespace,
        metadata_namespace,
        raw_capture_ref,
        metadata_capture_ref,
        source,
        distributor,
        registry,
        metadata,
        options.revision.clone(),
        rows,
        users.len() as u64,
        native_fields,
        field_counts,
        word_types,
        all_spellings.len() as u64,
        eligible_spellings.len() as u64,
        definition_count_matches,
        entry_count_matches,
        supplier_definition_count,
        supplier_entry_count,
        report_diagnostics,
        release_diagnostics,
        nested_duplicate_rows,
    )
}

const SUPPORTED_CLASSES: [&str; 16] = [
    "bu-letteral",
    "cmavo",
    "cmavo-compound",
    "cmevla",
    "experimental cmavo",
    "experimental gismu",
    "fu'ivla",
    "gismu",
    "lujvo",
    "nalvla",
    "obsolete cmavo",
    "obsolete cmevla",
    "obsolete fu'ivla",
    "obsolete zei-lujvo",
    "phrase",
    "zei-lujvo",
];
const JBOTCI_NOTICE_LOCATOR: &str = "https://github.com/int19h/jbotci/blob/3d9fd41d9ed32de26311ee27ec86b6b165985aa1/THIRD-PARTY-NOTICES.md#L242-L252";
fn is_supported_class(value: &str) -> bool {
    SUPPORTED_CLASSES.contains(&value)
}

#[allow(clippy::too_many_arguments)]
fn build_import(
    raw: &[u8],
    metadata_bytes: &[u8],
    raw_sha256: String,
    metadata_sha256: String,
    raw_namespace: &str,
    metadata_namespace: &str,
    raw_capture_ref: CaptureRef,
    metadata_capture_ref: CaptureRef,
    source: SourceId,
    distributor: DistributorId,
    registry: SourceRegistry,
    metadata: Metadata,
    revision: String,
    rows: Vec<Row>,
    contributors: u64,
    native_fields: u64,
    field_counts: BTreeMap<String, u64>,
    word_types: BTreeMap<String, u64>,
    all_spellings: u64,
    eligible_spellings: u64,
    definition_count_matches: bool,
    entry_count_matches: bool,
    supplier_definition_count: u64,
    supplier_entry_count: u64,
    mut report_diagnostics: BTreeMap<String, u64>,
    mut release_diagnostics: Vec<ReleaseDiagnostic>,
    nested_duplicate_rows: u64,
) -> Result<JbotciImport, JbotciImportError> {
    let language = metadata.string("language_tag")?.to_owned();
    let source_language = metadata.string("source_language_tag")?.to_owned();
    let mut native_occurrences = Vec::with_capacity(rows.len());
    let mut entries = Vec::with_capacity(rows.len());
    let mut assertions = Vec::with_capacity(rows.len());
    let mut contexts = Vec::new();
    let mut etymologies = Vec::new();
    let mut forms = Vec::new();
    let mut extensions = Vec::new();
    let mut assertion_ids = Vec::with_capacity(rows.len());
    let mut eligible_count = 0_u64;
    let mut excluded_count = 0_u64;
    let mut context_counts = BTreeMap::<String, u64>::new();

    for (index, row) in rows.iter().enumerate() {
        let locator = format!("/{index}");
        let occurrence = OccurrenceRef {
            capture: raw_capture_ref.clone(),
            locator: locator.clone(),
        };
        if row.duplicates.is_empty()
            && unique_field_index(&row.fields, "user").is_some()
            && user_name(&row.fields).is_none()
        {
            add_diag(
                &mut report_diagnostics,
                &mut release_diagnostics,
                "invalid_user",
                format!(
                    "row {index} user is not an object with one string username; raw value retained"
                ),
                Some(occurrence.clone()),
            );
        }
        if row.is_object && row.duplicates.is_empty() {
            for (name, code) in [
                ("selmaho", "invalid_selmaho"),
                ("definition_id", "invalid_definition_id"),
            ] {
                if let Some(i) = unique_field_index(&row.fields, name) {
                    let raw_value = row.fields[i].raw.as_str();
                    let valid = raw_value == "null"
                        || if name == "selmaho" {
                            as_json_string(raw_value).is_some()
                        } else {
                            as_json_string(raw_value).is_some()
                                || serde_json::from_str::<serde_json::Number>(raw_value).is_ok()
                        };
                    if !valid {
                        add_diag(
                            &mut report_diagnostics,
                            &mut release_diagnostics,
                            code,
                            format!(
                                "row {index} {name} has an unsupported JSON shape; raw value retained"
                            ),
                            Some(occurrence.clone()),
                        );
                    }
                }
            }
        }
        let base = format!("jbotci-{raw_sha256}");
        let entry_id = eid(format!("{base}-entry-{index}"))?;
        let assertion_id = eid(format!("{base}-assertion-{index}"))?;
        assertion_ids.push(assertion_id.clone());
        native_occurrences.push(RawOccurrence {
            locator: locator.clone(),
            fields: row
                .fields
                .iter()
                .map(|f| NativeField {
                    name: f.name.clone(),
                    encoding: NativeEncoding::Json,
                    raw_value: f.raw.clone(),
                })
                .collect(),
            raw_line: None,
        });

        let has_duplicates = !row.duplicates.is_empty();
        if !row.is_object {
            add_diag(
                &mut report_diagnostics,
                &mut release_diagnostics,
                "non_object_row",
                format!("row {index} is not an object; retained under $row"),
                Some(occurrence.clone()),
            );
        }
        for key in &row.duplicates {
            add_diag(
                &mut report_diagnostics,
                &mut release_diagnostics,
                "duplicate_key",
                format!(
                    "row {index} contains repeated JSON key {key:?}; typed projection suppressed"
                ),
                Some(occurrence.clone()),
            );
        }
        if has_duplicates {
            excluded_count += 1;
        }

        let typed_fields = if row.is_object && !has_duplicates {
            row.fields.as_slice()
        } else {
            &[]
        };
        let word = field(typed_fields, "word").and_then(|f| as_json_string(&f.raw));
        let class = field(typed_fields, "word_type").and_then(|f| as_json_string(&f.raw));
        let definition = field(typed_fields, "definition").and_then(|f| as_json_string(&f.raw));
        let word_valid = word.as_ref().is_some_and(|s| !s.is_empty());
        let class_valid = class.as_deref().is_some_and(is_supported_class);
        let definition_valid = definition.as_deref().is_some_and(|s| !s.trim().is_empty());
        let mut reasons = Vec::new();
        if !row.is_object {
            reasons.push("non-object-row".to_owned());
        }
        if has_duplicates {
            reasons.push("duplicate-json-key".to_owned());
        }
        if !word_valid {
            reasons.push("missing-null-wrong-type-or-empty-word".to_owned());
            add_diag(
                &mut report_diagnostics,
                &mut release_diagnostics,
                "invalid_word",
                format!("row {index} has no nonempty string word"),
                Some(occurrence.clone()),
            );
        }
        if class.is_none() {
            reasons.push("missing-null-or-wrong-type-word_type".to_owned());
            add_diag(
                &mut report_diagnostics,
                &mut release_diagnostics,
                "invalid_word_type",
                format!("row {index} has no string word_type"),
                Some(occurrence.clone()),
            );
        } else if !class_valid {
            reasons.push("unsupported-word_type".to_owned());
            add_diag(
                &mut report_diagnostics,
                &mut release_diagnostics,
                "unsupported_word_type",
                format!(
                    "row {index} has unsupported word_type {:?}",
                    class.as_deref().unwrap_or_default()
                ),
                Some(occurrence.clone()),
            );
        }
        if !definition_valid {
            reasons.push("missing-null-wrong-type-or-empty-definition".to_owned());
            add_diag(
                &mut report_diagnostics,
                &mut release_diagnostics,
                "invalid_definition",
                format!("row {index} has no non-whitespace string definition"),
                Some(occurrence.clone()),
            );
        }
        let eligible = reasons.is_empty();
        if eligible {
            eligible_count += 1;
        } else if !has_duplicates {
            excluded_count += 1;
        }
        let eligibility = if eligible {
            Eligibility::Lexical
        } else {
            Eligibility::Excluded {
                reason: reasons.join(";"),
            }
        };

        let mut entry = EntryRecord {
            id: entry_id.clone(),
            source: source.clone(),
            spelling: if word_valid {
                Presence::Value(word.clone().unwrap_or_default())
            } else {
                presence_string(typed_fields, "word")
            },
            language: Presence::Value(source_language.clone()),
            script: Presence::Missing,
            native_word_class: if class.is_some() {
                Presence::Value(class.clone().unwrap_or_default())
            } else {
                presence_string(typed_fields, "word_type")
            },
            selmaho: presence_string(typed_fields, "selmaho"),
            mapped_pos: Presence::Missing,
        };
        if has_duplicates {
            entry.spelling = Presence::Missing;
            entry.native_word_class = Presence::Missing;
            entry.selmaho = Presence::Missing;
        }
        let definition_presence = match field(typed_fields, "definition") {
            None => Presence::Missing,
            Some(f) if f.raw == "null" => Presence::Null,
            Some(_) if definition.is_some() => Presence::Value(text_value(
                definition.clone().unwrap_or_default(),
                &language,
            )),
            Some(_) => Presence::Missing,
        };
        let source_score = presence_raw(typed_fields, "score")?;
        let mut assertion = AssertionRecord {
            id: assertion_id.clone(),
            entry: entry_id.clone(),
            occurrence: occurrence.clone(),
            native_sense_id: Presence::Missing,
            language: Presence::Value(language.clone()),
            definition: definition_presence,
            source_score,
            timestamps: Vec::new(),
            eligibility,
        };
        if has_duplicates {
            assertion.native_sense_id = Presence::Missing;
            assertion.definition = Presence::Missing;
            assertion.source_score = Presence::Missing;
        }
        entries.push(entry);
        assertions.push(assertion);

        if row.is_object {
            let known: BTreeSet<&str> = [
                "word",
                "word_type",
                "definition",
                "definition_id",
                "score",
                "selmaho",
                "notes",
                "jargon",
                "gloss_keywords",
                "place_keywords",
                "rafsi",
                "experimental_rafsi",
                "etymology",
                "user",
            ]
            .into_iter()
            .collect();
            for (i, f) in row.fields.iter().enumerate() {
                if !known.contains(f.name.as_str()) {
                    extensions.push(extension(
                        &assertion_id,
                        format!("lensisku.native_field.{i}.{}", f.name),
                        &f.raw,
                        Some(source_evidence(occurrence.clone(), Some(i))),
                    )?);
                    add_diag(
                        &mut report_diagnostics,
                        &mut release_diagnostics,
                        "unknown_native_field",
                        format!(
                            "row {index} unknown field {:?} retained as an extension",
                            f.name
                        ),
                        Some(occurrence.clone()),
                    );
                }
            }
        }

        if !row.is_object || has_duplicates {
            continue;
        }

        // Optional scalar fields are projected only in their advertised source shape.
        for (name, kind, diagnostic) in [
            ("notes", ContextKind::Note, "invalid_notes"),
            ("jargon", ContextKind::Domain, "invalid_jargon"),
        ] {
            if let Some(i) = unique_field_index(&row.fields, name) {
                let f = &row.fields[i];
                match as_json_string(&f.raw) {
                    Some(value) => {
                        let id = eid(format!("{base}-context-{index}-{name}"))?;
                        contexts.push(context(
                            id,
                            &assertion_id,
                            kind,
                            Presence::Value(text_value(value, &language)),
                            &occurrence,
                            i,
                        ));
                        *context_counts.entry(name.into()).or_default() += 1;
                    }
                    None if f.raw == "null" => {
                        let id = eid(format!("{base}-context-{index}-{name}"))?;
                        contexts.push(context(
                            id,
                            &assertion_id,
                            kind,
                            Presence::Null,
                            &occurrence,
                            i,
                        ));
                        *context_counts.entry(name.into()).or_default() += 1;
                    }
                    None => add_diag(
                        &mut report_diagnostics,
                        &mut release_diagnostics,
                        diagnostic,
                        format!("row {index} {name} is not a string or null; raw value retained"),
                        Some(occurrence.clone()),
                    ),
                }
            }
        }

        if let Some(i) = unique_field_index(&row.fields, "gloss_keywords") {
            let f = &row.fields[i];
            extensions.push(extension(
                &assertion_id,
                "lensisku.gloss_keywords.raw".into(),
                &f.raw,
                Some(source_evidence(occurrence.clone(), Some(i))),
            )?);
            if f.raw != "null" {
                if let Some(items) = raw_array(&f.raw) {
                    for (item_index, item) in items.iter().enumerate() {
                        let item_raw = item.get();
                        let nested_dup = scan_nested_duplicates(item_raw)
                            .map_err(JbotciImportError::InvalidJson)?;
                        if !item_raw.trim_start().starts_with('{') || !nested_dup.is_empty() {
                            add_diag(
                                &mut report_diagnostics,
                                &mut release_diagnostics,
                                "invalid_gloss_object",
                                format!(
                                    "row {index} gloss item {item_index} is not a unique-key object; whole field retained"
                                ),
                                Some(occurrence.clone()),
                            );
                            continue;
                        }
                        let object =
                            parse_fields(item_raw).map_err(JbotciImportError::InvalidJson)?;
                        let Some(word_field) = field(&object, "word") else {
                            add_diag(
                                &mut report_diagnostics,
                                &mut release_diagnostics,
                                "invalid_gloss_object",
                                format!("row {index} gloss item {item_index} lacks word"),
                                Some(occurrence.clone()),
                            );
                            continue;
                        };
                        let Some(gloss) = as_json_string(&word_field.raw) else {
                            add_diag(
                                &mut report_diagnostics,
                                &mut release_diagnostics,
                                "invalid_gloss_object",
                                format!("row {index} gloss item {item_index} word is not a string"),
                                Some(occurrence.clone()),
                            );
                            continue;
                        };
                        let id = eid(format!("{base}-context-gloss-{index}-{item_index}"))?;
                        let ctx = context(
                            id.clone(),
                            &assertion_id,
                            ContextKind::Gloss,
                            Presence::Value(text_value(gloss, &language)),
                            &occurrence,
                            i,
                        );
                        contexts.push(ctx);
                        extensions.push(extension(
                            &id,
                            "lensisku.gloss_keywords.object".into(),
                            item_raw,
                            Some(source_evidence(occurrence.clone(), Some(i))),
                        )?);
                        *context_counts.entry("gloss_keywords".into()).or_default() += 1;
                    }
                } else {
                    add_diag(
                        &mut report_diagnostics,
                        &mut release_diagnostics,
                        "invalid_gloss_keywords",
                        format!(
                            "row {index} gloss_keywords is not an array or null; raw value retained"
                        ),
                        Some(occurrence.clone()),
                    );
                }
            }
        }
        if let Some(i) = unique_field_index(&row.fields, "place_keywords") {
            let raw_value = &row.fields[i].raw;
            extensions.push(extension(
                &assertion_id,
                "lensisku.place_keywords".into(),
                raw_value,
                Some(source_evidence(occurrence.clone(), Some(i))),
            )?);
            if raw_value != "null" && raw_array(raw_value).is_none() {
                add_diag(
                    &mut report_diagnostics,
                    &mut release_diagnostics,
                    "invalid_place_keywords",
                    format!("row {index} place_keywords is not an array; raw value retained"),
                    Some(occurrence.clone()),
                );
            }
        }
        for name in ["rafsi", "experimental_rafsi"] {
            if let Some(i) = unique_field_index(&row.fields, name) {
                let f = &row.fields[i];
                if f.raw == "null" {
                    continue;
                }
                if let Some(tokens) = forms_from_field(&f.raw) {
                    for (form_index, spelling) in tokens.into_iter().enumerate() {
                        let id = eid(format!("{base}-form-{index}-{name}-{form_index}"))?;
                        forms.push(FormRecord {
                            id,
                            entry: entry_id.clone(),
                            spelling,
                            language: Presence::Value(source_language.clone()),
                            script: Presence::Missing,
                            pronunciation: Presence::Missing,
                            kind: FormKind::SourceReportedForm,
                            standing: Presence::Value(name.into()),
                            components: Vec::new(),
                            evidence: Some(source_evidence(occurrence.clone(), Some(i))),
                            computation: None,
                        });
                    }
                } else {
                    add_diag(
                        &mut report_diagnostics,
                        &mut release_diagnostics,
                        "invalid_reported_form",
                        format!(
                            "row {index} {name} is neither a string nor a string array; raw value retained"
                        ),
                        Some(occurrence.clone()),
                    );
                }
            }
        }
        if let Some(i) = unique_field_index(&row.fields, "etymology") {
            let f = &row.fields[i];
            let statement = match f.raw.as_str() {
                "null" => Presence::Null,
                _ => match as_json_string(&f.raw) {
                    Some(value) => Presence::Value(text_value(value, &language)),
                    None => {
                        add_diag(
                            &mut report_diagnostics,
                            &mut release_diagnostics,
                            "invalid_etymology",
                            format!(
                                "row {index} etymology is not a string or null; raw value retained"
                            ),
                            Some(occurrence.clone()),
                        );
                        Presence::Missing
                    }
                },
            };
            if !matches!(statement, Presence::Missing) {
                etymologies.push(EtymologyRecord {
                    id: eid(format!("{base}-etymology-{index}"))?,
                    assertion: assertion_id.clone(),
                    statement,
                    native_kind: Presence::Missing,
                    components: Vec::new(),
                    evidence: source_evidence(occurrence.clone(), Some(i)),
                });
            }
        }
    }

    let raw_capture = RawCapture::new(
        raw_namespace,
        CaptureMetadata {
            source_name: "Lensisku English dictionary export".into(),
            format_label: "jbotci-export-v1".into(),
            locator: metadata.string("download_url")?.into(),
            revision: Presence::Value(revision.clone()),
            captured_at: Presence::Missing,
            time_precision: TimePrecision::Unknown,
            notice_locator: Presence::Value(JBOTCI_NOTICE_LOCATOR.into()),
        },
        raw.to_vec(),
        native_occurrences,
    );
    let metadata_capture = RawCapture::new(
        metadata_namespace,
        CaptureMetadata {
            source_name: "jbotci supplier metadata".into(),
            format_label: "jbotci-metadata-v1".into(),
            locator: "crates/jbotci-dictionary-data/data/dictionary-en.metadata.toml".into(),
            revision: Presence::Value(revision.clone()),
            captured_at: Presence::Missing,
            time_precision: TimePrecision::Unknown,
            notice_locator: Presence::Missing,
        },
        metadata_bytes.to_vec(),
        Vec::new(),
    );
    let metadata_json =
        serde_json::to_string(std::str::from_utf8(metadata_bytes).map_err(|_| {
            JbotciImportError::InvalidUtf8 {
                input: "metadata".into(),
            }
        })?)
        .map_err(|e| JbotciImportError::InvalidMetadata(e.to_string()))?;
    let release_id = eid(format!(
        "jbotci-release-{}",
        sha256(
            format!("vlajmaji-jbotci-release-v1\0{revision}\0{raw_sha256}\0{metadata_sha256}")
                .as_bytes()
        ),
    ))?;
    let collection_id = eid("lensisku-jbotci-en-collection".into())?;
    let component_id = eid("lensisku-jbotci-en-component".into())?;
    let code_notice = CodeNotice {
        text: "Apache-2.0".into(),
        locator: Presence::Value("LICENSE".into()),
        license: Presence::Value("Apache-2.0".into()),
    };
    let content_notice_text = "The jbovlaste dictionary databases are placed in the public domain. The individual definitions are contributed by the Lojban community under those public-domain terms.";
    let release = ReleaseRecord {
        id: release_id.clone(),
        component: component_id.clone(),
        distributor: Some(distributor),
        upstream_revision: Presence::Missing,
        distributor_revision: Presence::Value(revision.clone()),
        captured_at: Presence::Missing,
        time_precision: TimePrecision::Unknown,
        schema_version: 1,
        normalizer: "vlajmaji-jbotci-export".into(),
        normalizer_version: "1".into(),
        captures: vec![raw_capture_ref.clone(), metadata_capture_ref.clone()],
        assertions: assertion_ids,
        accounting: ReleaseAccounting {
            input_occurrences: Some(rows.len() as u64),
            normalized_occurrences: Some(rows.len() as u64),
            query_excluded_occurrences: Some(excluded_count),
            unit: "source-array-row".into(),
        },
        diagnostics: release_diagnostics,
        selected: true,
        availability: Availability::Available,
        content_notices: vec![ContentNotice {
            text: Presence::Value(content_notice_text.into()),
            locator: Presence::Value(JBOTCI_NOTICE_LOCATOR.into()),
            license: Presence::Value(
                "supplier-claimed public domain; coverage not independently established".into(),
            ),
        }],
        redistribution: RedistributionStatus::Held,
        redistribution_evidence: vec![
            "Supplier notice is a claim; upstream source coverage remains unverified.".into(),
        ],
    };
    extensions.push(extension(
        &release_id,
        "jbotci.metadata.raw_text".into(),
        &metadata_json,
        None,
    )?);
    let dictionary = Dictionary::from_document(DictionaryDocument {
        version: 1,
        legacy_archive: None,
        registry,
        captures: vec![raw_capture, metadata_capture],
        metadata: DictionaryMetadata {
            catalogue_revision: 0,
            code_notices: vec![code_notice],
            collections: vec![CollectionRecord {
                id: collection_id.clone(),
                source: source.clone(),
                display_name: "Lensisku English lexical export".into(),
                description: Presence::Missing,
            }],
            components: vec![ComponentRecord {
                id: component_id.clone(),
                collection: collection_id,
                native_name: "dictionary-en.json".into(),
                languages: vec![source_language],
            }],
            releases: vec![release],
            entries,
            assertions,
            contexts,
            frames: Vec::new(),
            forms,
            etymologies,
            relations: Vec::new(),
            owned_revisions: Vec::new(),
            owned_heads: Vec::new(),
            editorial_senses: Vec::new(),
            computations: Vec::new(),
            annotations: Vec::new(),
            extensions,
        },
    })?;
    let report = JbotciImportReport {
        version: 1,
        revision,
        raw_sha256,
        metadata_sha256,
        raw_bytes: raw.len() as u64,
        metadata_bytes: metadata_bytes.len() as u64,
        input_rows: rows.len() as u64,
        native_fields,
        normalized_assertions: rows.len() as u64,
        lexical_occurrences: eligible_count,
        excluded_occurrences: excluded_count,
        eligible_spellings,
        all_spellings,
        contributors,
        supplier_definition_count,
        supplier_entry_count,
        word_types,
        field_counts,
        diagnostics: report_diagnostics,
        definition_count_matches,
        entry_count_matches,
    };
    let _ = nested_duplicate_rows;
    let _ = context_counts;
    Ok(JbotciImport { dictionary, report })
}
