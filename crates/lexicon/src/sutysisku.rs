//! Lossless normalization of the pinned Sutysisku English projection.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use serde::de::{Deserializer as _, Error as _, MapAccess, Visitor};
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;

use crate::dictionary::{
    AssertionRecord, Availability, CaptureMetadata, CaptureRef, CodeNotice, CollectionRecord,
    ComponentRecord, ContentNotice, ContextKind, ContextRecord, Dictionary, DictionaryDocument,
    DictionaryError, DictionaryMetadata, Eligibility, EntityId, EntryRecord, ExtensionRecord,
    FormKind, FormRecord, NativeEncoding, NativeField, OccurrenceRef, Presence, RawCapture,
    RawJson, RawOccurrence, RedistributionStatus, ReleaseAccounting, ReleaseDiagnostic,
    ReleaseRecord, SourceEvidence, TextValue,
};
use crate::lensisku::{
    Field, as_json_string, is_supported_class, parse_fields, scan_nested_duplicates, sha256,
    valid_hex,
};
use crate::registry::{
    AbbreviationKind, DistributorId, DistributorRecord, LineageTarget, OccurrenceAttribution,
    RegistryDocument, RegistryError, SnapshotAttribution, SourceId, SourceLineage, SourceOrigin,
    SourceRecord, SourceRegistry,
};
use crate::{RecordIdentity, SnapshotIdentity, TimePrecision};

/// Repository revision containing the pinned Sutysisku English projection.
pub const SUTYSISKU_PINNED_REVISION: &str = "2c71205747143ea965749ffc3ac3b48ef5c0a517";
/// SHA-256 of the complete pinned Sutysisku projection bytes.
pub const SUTYSISKU_PINNED_RAW_SHA256: &str =
    "8e80f6626d2e68a17025d52c3f0cf5ea8c32cbfeeb0c2ff1417fcba539bd043d";
const RAW_NAMESPACE: &str = "captured:sutysisku:parsed-en";
const FIELDS: [&str; 7] = ["d", "t", "s", "g", "n", "r", "b"];

/// Pins and provenance locators for one local Sutysisku import.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SutysiskuImportOptions {
    /// Exact forty-character repository revision.
    pub revision: String,
    /// Expected SHA-256 of the full raw export.
    pub expected_raw_sha256: String,
    /// Source locator retained in capture metadata.
    pub raw_locator: Option<String>,
    /// Notice locator retained with the capture.
    pub notice_locator: Option<String>,
}
impl Default for SutysiskuImportOptions {
    fn default() -> Self {
        Self {
            revision: SUTYSISKU_PINNED_REVISION.into(),
            expected_raw_sha256: SUTYSISKU_PINNED_RAW_SHA256.into(),
            raw_locator: None,
            notice_locator: None,
        }
    }
}

/// Deterministic population report for one Sutysisku import.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SutysiskuImportReport {
    /// Report schema version.
    pub version: u32,
    /// Distributor repository revision used for this import.
    pub revision: String,
    /// Full raw-export SHA-256.
    pub raw_sha256: String,
    /// Exact input byte count.
    pub raw_bytes: u64,
    /// Number of root-object members, including repeated decoded names.
    pub input_rows: u64,
    /// Number of normalized assertions.
    pub normalized_assertions: u64,
    /// Number of eligible lexical occurrences.
    pub lexical_occurrences: u64,
    /// Number of retained but excluded occurrences.
    pub excluded_occurrences: u64,
    /// Number of distinct eligible root spellings.
    pub eligible_spellings: u64,
    /// Number of distinct decoded root spellings.
    pub all_spellings: u64,
    /// Number of root member keys and therefore key-carrier fields.
    pub root_keys: u64,
    /// Number of child object members, excluding root key carriers.
    pub native_child_fields: u64,
    /// Number of non-object root values represented by a $row carrier.
    pub non_object_rows: u64,
    /// Total retained native fields, including $key and $row carriers.
    pub native_fields: u64,
    /// Counts of string-valued native t members.
    pub word_types: BTreeMap<String, u64>,
    /// Counts of ordered native child field names and carrier names.
    pub field_counts: BTreeMap<String, u64>,
    /// Number of retained typed contexts.
    pub contexts: u64,
    /// Number of retained source-reported forms.
    pub reported_forms: u64,
    /// Number of raw native extension records.
    pub extensions: u64,
    /// Per-code diagnostics; not an excluded-row count.
    pub diagnostics: BTreeMap<String, u64>,
}

/// Imported dictionary and its deterministic report.
#[derive(Debug, Clone)]
pub struct SutysiskuImport {
    dictionary: Dictionary,
    report: SutysiskuImportReport,
}
impl SutysiskuImport {
    /// Borrow the imported dictionary.
    pub fn dictionary(&self) -> &Dictionary {
        &self.dictionary
    }
    /// Borrow the deterministic report.
    pub fn report(&self) -> &SutysiskuImportReport {
        &self.report
    }
    /// Consume the result and return its dictionary.
    pub fn into_dictionary(self) -> Dictionary {
        self.dictionary
    }
}

/// Typed failures while validating or normalizing a Sutysisku projection.
#[derive(Debug)]
#[non_exhaustive]
pub enum SutysiskuImportError {
    /// A supplied revision, digest, or locator is invalid.
    InvalidOptions(String),
    /// The raw bytes do not match the caller's full-byte pin.
    HashMismatch {
        /// Expected lowercase SHA-256 from the caller's pin.
        expected: String,
        /// Digest computed from the supplied full byte slice.
        actual: String,
    },
    /// The supplied export is not UTF-8.
    InvalidUtf8,
    /// The export is malformed JSON or not a root object.
    InvalidJson(String),
    /// Registry attribution was rejected.
    Registry(RegistryError),
    /// The normalized dictionary model was rejected.
    Dictionary(DictionaryError),
}
impl fmt::Display for SutysiskuImportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidOptions(s) => write!(f, "invalid Sutysisku import options: {s}"),
            Self::HashMismatch { expected, actual } => {
                write!(f, "raw SHA-256 mismatch: expected {expected}, got {actual}")
            }
            Self::InvalidUtf8 => f.write_str("raw input is not UTF-8"),
            Self::InvalidJson(s) => write!(f, "invalid raw JSON: {s}"),
            Self::Registry(e) => write!(f, "invalid source registry: {e}"),
            Self::Dictionary(e) => write!(f, "invalid dictionary: {e}"),
        }
    }
}
impl std::error::Error for SutysiskuImportError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Registry(e) => Some(e),
            Self::Dictionary(e) => Some(e),
            _ => None,
        }
    }
}
impl From<RegistryError> for SutysiskuImportError {
    fn from(error: RegistryError) -> Self {
        Self::Registry(error)
    }
}
impl From<DictionaryError> for SutysiskuImportError {
    fn from(error: DictionaryError) -> Self {
        Self::Dictionary(error)
    }
}

fn locator(value: &Option<String>, label: &str) -> Result<String, SutysiskuImportError> {
    let Some(value) = value.as_deref() else {
        return Err(SutysiskuImportError::InvalidOptions(format!(
            "{label} is required"
        )));
    };
    if value.is_empty() || value.trim() != value || value.chars().any(char::is_control) {
        return Err(SutysiskuImportError::InvalidOptions(format!(
            "{label} must be nonempty, trimmed, and control-free"
        )));
    }
    Ok(value.to_owned())
}

struct RootMember {
    key: String,
    key_raw: String,
    value_raw: String,
}
struct RootVisitor;
impl<'de> Visitor<'de> for RootVisitor {
    type Value = Vec<RootMember>;
    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a JSON root object")
    }
    fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Self::Value, M::Error> {
        let mut members = Vec::new();
        while let Some(key) = map.next_key::<Box<RawValue>>()? {
            let key_raw = key.get().to_owned();
            let decoded = serde_json::from_str::<String>(&key_raw).map_err(M::Error::custom)?;
            let value = map.next_value::<Box<RawValue>>()?;
            members.push(RootMember {
                key: decoded,
                key_raw,
                value_raw: value.get().to_owned(),
            });
        }
        Ok(members)
    }
}
fn decode_root(raw_text: &str) -> Result<Vec<RootMember>, String> {
    let mut deserializer = serde_json::Deserializer::from_str(raw_text);
    let members = deserializer
        .deserialize_map(RootVisitor)
        .map_err(|error| error.to_string())?;
    deserializer.end().map_err(|error| error.to_string())?;
    Ok(members)
}
fn entity(value: String) -> Result<EntityId, DictionaryError> {
    EntityId::new(value)
}
fn snapshot(namespace: &str, digest: &str) -> Result<SnapshotIdentity, SutysiskuImportError> {
    serde_json::from_value(serde_json::json!({"namespace":namespace,"sha256":digest})).map_err(
        |error| SutysiskuImportError::InvalidJson(format!("internal snapshot identity: {error}")),
    )
}
fn occurrence_identity(
    namespace: &str,
    digest: &str,
    locator: &str,
) -> Result<RecordIdentity, SutysiskuImportError> {
    serde_json::from_value(
        serde_json::json!({"snapshot":{"namespace":namespace,"sha256":digest},"locator":locator}),
    )
    .map_err(|error| {
        SutysiskuImportError::InvalidJson(format!("internal occurrence identity: {error}"))
    })
}
fn evidence(occurrence: &OccurrenceRef, field_index: Option<usize>) -> SourceEvidence {
    SourceEvidence {
        occurrence: occurrence.clone(),
        field_index,
        citation: Presence::Missing,
        offset: None,
    }
}
fn text(value: String, language: &str) -> TextValue {
    TextValue {
        text: value,
        language: Presence::Value(language.into()),
        markup: Presence::Missing,
    }
}
fn unique_field<'a>(fields: &'a [Field], name: &str) -> Option<&'a Field> {
    let mut matches = fields.iter().filter(|field| field.name == name);
    let first = matches.next()?;
    matches.next().is_none().then_some(first)
}
fn is_null(field: &Field) -> bool {
    field.raw == "null"
}
fn pointer_segment(value: &str) -> String {
    value.replace('~', "~0").replace('/', "~1")
}
fn add_diagnostic(
    report: &mut BTreeMap<String, u64>,
    release: &mut Vec<ReleaseDiagnostic>,
    code: &str,
    message: String,
    occurrence: Option<OccurrenceRef>,
) {
    *report.entry(code.to_owned()).or_default() += 1;
    release.push(ReleaseDiagnostic {
        occurrence,
        code: code.to_owned(),
        message,
    });
}
fn extension(
    owner: &EntityId,
    name: String,
    raw: &str,
    evidence: SourceEvidence,
) -> Result<ExtensionRecord, DictionaryError> {
    Ok(ExtensionRecord {
        owner: owner.clone(),
        name,
        value: RawJson::new(raw.to_owned())?,
        evidence: Some(evidence),
    })
}

/// Normalize caller-supplied pinned Sutysisku bytes without filesystem or network access.
///
/// Every ordered root member, key lexeme, and child value lexeme remains attached to
/// the resulting dictionary. Unsupported rows remain represented as excluded occurrences.
///
/// # Errors
///
/// Returns a typed error for invalid pins/locators, hash or UTF-8 failures, malformed
/// JSON, invalid attribution, or a rejected normalized dictionary.
pub fn import_sutysisku_export(
    raw: &[u8],
    options: &SutysiskuImportOptions,
) -> Result<SutysiskuImport, SutysiskuImportError> {
    if !valid_hex(&options.revision, 40) {
        return Err(SutysiskuImportError::InvalidOptions(
            "revision must be 40 lowercase hexadecimal characters".into(),
        ));
    }
    if !valid_hex(&options.expected_raw_sha256, 64) {
        return Err(SutysiskuImportError::InvalidOptions(
            "expected raw SHA-256 must be 64 lowercase hexadecimal characters".into(),
        ));
    }
    let raw_locator = match &options.raw_locator {
        Some(_) => locator(&options.raw_locator, "raw locator")?,
        None => format!(
            "https://github.com/La-Lojban/sutysisku/blob/{}/data/parsed-en.json",
            options.revision
        ),
    };
    let notice_locator = match &options.notice_locator {
        Some(_) => locator(&options.notice_locator, "notice locator")?,
        None => format!(
            "https://github.com/La-Lojban/sutysisku/blob/{}/README.md",
            options.revision
        ),
    };
    let actual = sha256(raw);
    if actual != options.expected_raw_sha256 {
        return Err(SutysiskuImportError::HashMismatch {
            expected: options.expected_raw_sha256.clone(),
            actual,
        });
    }
    let raw_text = std::str::from_utf8(raw).map_err(|_| SutysiskuImportError::InvalidUtf8)?;
    let members = decode_root(raw_text).map_err(SutysiskuImportError::InvalidJson)?;
    let source = SourceId::new("sutysisku")?;
    let distributor = DistributorId::new("la-lojban")?;
    let capture_ref = CaptureRef {
        namespace: RAW_NAMESPACE.into(),
        sha256: actual.clone(),
    };
    let snapshot_id = snapshot(RAW_NAMESPACE, &actual)?;
    let mut registry_doc = RegistryDocument::default();
    registry_doc.sources.push(SourceRecord {
        id: source.clone(),
        display_name: "Sutysisku".into(),
        abbreviation: "SUT".into(),
        abbreviation_kind: AbbreviationKind::ProjectAssigned,
        abbreviation_evidence: None,
        origin: SourceOrigin::Upstream,
    });
    registry_doc.distributors.push(DistributorRecord {
        id: distributor.clone(),
        display_name: "La-Lojban".into(),
    });
    registry_doc.lineage.push(SourceLineage {
        source: source.clone(),
        upstream: LineageTarget::Unresolved(
            "Exact upstream lexical dictionary lineage is unestablished for the pinned Sutysisku projection.".into(),
        ),
        evidence: None,
    });
    registry_doc.snapshots.push(SnapshotAttribution {
        snapshot: snapshot_id.clone(),
        source: source.clone(),
        distributor: Some(distributor.clone()),
    });

    let mut root_groups = BTreeMap::<String, Vec<usize>>::new();
    for (ordinal, member) in members.iter().enumerate() {
        root_groups
            .entry(member.key.clone())
            .or_default()
            .push(ordinal);
    }
    let duplicate_root_groups: BTreeMap<String, Vec<usize>> = root_groups
        .into_iter()
        .filter(|(_, ordinals)| ordinals.len() > 1)
        .collect();

    let mut field_counts = BTreeMap::<String, u64>::new();
    let mut word_types = BTreeMap::<String, u64>::new();
    let mut all_spellings = BTreeSet::new();
    let mut eligible_spellings = BTreeSet::new();
    let mut diagnostics = BTreeMap::new();
    let mut release_diagnostics = Vec::new();
    let mut occurrences = Vec::with_capacity(members.len());
    let mut entries = Vec::with_capacity(members.len());
    let mut assertions = Vec::with_capacity(members.len());
    let mut contexts = Vec::new();
    let mut forms = Vec::new();
    let mut extensions = Vec::new();
    let mut assertion_ids = Vec::with_capacity(members.len());
    let mut child_field_total = 0_u64;
    let mut non_object_rows = 0_u64;
    let mut eligible_count = 0_u64;

    for (ordinal, member) in members.iter().enumerate() {
        let locator = format!("/{ordinal}");
        let occurrence = OccurrenceRef {
            capture: capture_ref.clone(),
            locator: locator.clone(),
        };
        let is_object = member.value_raw.trim_start().starts_with('{');
        let fields = if is_object {
            parse_fields(&member.value_raw).map_err(SutysiskuImportError::InvalidJson)?
        } else {
            Vec::new()
        };
        let mut duplicate_paths = Vec::<Vec<String>>::new();
        let mut seen = BTreeSet::new();
        for child in &fields {
            if !seen.insert(child.name.clone()) {
                duplicate_paths.push(vec![child.name.clone()]);
            }
            let nested =
                scan_nested_duplicates(&child.raw).map_err(SutysiskuImportError::InvalidJson)?;
            for mut path in nested.paths {
                path.insert(0, child.name.clone());
                duplicate_paths.push(path);
            }
        }
        let ambiguous = !duplicate_paths.is_empty();
        *field_counts.entry("$key".into()).or_default() += 1;
        if is_object {
            child_field_total += fields.len() as u64;
            for child in &fields {
                *field_counts.entry(child.name.clone()).or_default() += 1;
                if child.name == "t"
                    && let Some(value) = as_json_string(&child.raw)
                {
                    *word_types.entry(value).or_default() += 1;
                }
            }
        } else {
            non_object_rows += 1;
            *field_counts.entry("$row".into()).or_default() += 1;
        }
        all_spellings.insert(member.key.clone());
        let definition = if !ambiguous {
            unique_field(&fields, "d").and_then(|item| as_json_string(&item.raw))
        } else {
            None
        };
        let class = if !ambiguous {
            unique_field(&fields, "t").and_then(|item| as_json_string(&item.raw))
        } else {
            None
        };
        let eligible = is_object
            && !ambiguous
            && !member.key.is_empty()
            && definition
                .as_deref()
                .is_some_and(|value| !value.trim().is_empty())
            && class.as_deref().is_some_and(is_supported_class);
        if eligible {
            eligible_count += 1;
            eligible_spellings.insert(member.key.clone());
        }
        let entry_id = entity(format!("sutysisku-{actual}-entry-{ordinal}"))?;
        let assertion_id = entity(format!("sutysisku-{actual}-assertion-{ordinal}"))?;
        assertion_ids.push(assertion_id.clone());
        let native_occurrence = if is_object {
            let mut native_fields = Vec::with_capacity(fields.len() + 1);
            native_fields.push(NativeField {
                name: "$key".into(),
                encoding: NativeEncoding::Json,
                raw_value: member.key_raw.clone(),
            });
            native_fields.extend(fields.iter().map(|field| NativeField {
                name: field.name.clone(),
                encoding: NativeEncoding::Json,
                raw_value: field.raw.clone(),
            }));
            native_fields
        } else {
            vec![
                NativeField {
                    name: "$key".into(),
                    encoding: NativeEncoding::Json,
                    raw_value: member.key_raw.clone(),
                },
                NativeField {
                    name: "$row".into(),
                    encoding: NativeEncoding::Json,
                    raw_value: member.value_raw.clone(),
                },
            ]
        };
        occurrences.push(RawOccurrence {
            locator: locator.clone(),
            fields: native_occurrence,
            raw_line: None,
        });
        registry_doc.occurrences.push(OccurrenceAttribution {
            occurrence: occurrence_identity(RAW_NAMESPACE, &actual, &locator)?,
            contributor: None,
            native_entry_id: Some(member.key.clone()),
            native_assertion_id: None,
        });

        entries.push(EntryRecord {
            id: entry_id.clone(),
            source: source.clone(),
            spelling: Presence::Value(member.key.clone()),
            language: Presence::Value("jbo".into()),
            script: Presence::Missing,
            native_word_class: if ambiguous {
                Presence::Missing
            } else {
                match unique_field(&fields, "t") {
                    Some(item) if item.raw == "null" => Presence::Null,
                    Some(item) => {
                        as_json_string(&item.raw).map_or(Presence::Missing, Presence::Value)
                    }
                    None => Presence::Missing,
                }
            },
            selmaho: if ambiguous {
                Presence::Missing
            } else {
                match unique_field(&fields, "s") {
                    Some(item) if item.raw == "null" => Presence::Null,
                    Some(item) => {
                        as_json_string(&item.raw).map_or(Presence::Missing, Presence::Value)
                    }
                    None => Presence::Missing,
                }
            },
            mapped_pos: Presence::Missing,
        });
        let eligibility = if eligible {
            Eligibility::Lexical
        } else {
            let reason = if !is_object {
                "non-object-row"
            } else if ambiguous {
                "duplicate-json-key"
            } else if member.key.is_empty() {
                "empty-root-key"
            } else if definition
                .as_deref()
                .is_none_or(|value| value.trim().is_empty())
            {
                "missing-null-wrong-type-or-empty-definition"
            } else if class.is_none() {
                "missing-null-or-wrong-type-word-type"
            } else {
                "unsupported-word-type"
            };
            Eligibility::Excluded {
                reason: reason.into(),
            }
        };
        let definition_presence = if ambiguous {
            Presence::Missing
        } else {
            match unique_field(&fields, "d") {
                Some(item) if item.raw == "null" => Presence::Null,
                Some(item) => as_json_string(&item.raw)
                    .map(|value| Presence::Value(text(value, "en")))
                    .unwrap_or(Presence::Missing),
                None => Presence::Missing,
            }
        };
        assertions.push(AssertionRecord {
            id: assertion_id.clone(),
            entry: entry_id.clone(),
            occurrence: occurrence.clone(),
            native_sense_id: Presence::Missing,
            language: Presence::Value("en".into()),
            definition: definition_presence,
            source_score: Presence::Missing,
            timestamps: Vec::new(),
            eligibility,
        });

        if let Some(ordinals) = duplicate_root_groups.get(&member.key) {
            add_diagnostic(
                &mut diagnostics,
                &mut release_diagnostics,
                "duplicate_root_key",
                format!(
                    "decoded root key {:?} occurs at member ordinals {ordinals:?}",
                    member.key
                ),
                Some(occurrence.clone()),
            );
        }
        if !is_object {
            add_diagnostic(
                &mut diagnostics,
                &mut release_diagnostics,
                "non_object_row",
                format!("root member {ordinal} is not an object; retained under $row"),
                Some(occurrence.clone()),
            );
        } else if !ambiguous {
            if definition
                .as_deref()
                .is_none_or(|value| value.trim().is_empty())
            {
                add_diagnostic(
                    &mut diagnostics,
                    &mut release_diagnostics,
                    "invalid_definition",
                    format!("root member {ordinal} d is not a non-whitespace string"),
                    Some(occurrence.clone()),
                );
            }
            if class.is_none() {
                add_diagnostic(
                    &mut diagnostics,
                    &mut release_diagnostics,
                    "invalid_word_type",
                    format!("root member {ordinal} t is not a string"),
                    Some(occurrence.clone()),
                );
            } else if !class.as_deref().is_some_and(is_supported_class) {
                add_diagnostic(
                    &mut diagnostics,
                    &mut release_diagnostics,
                    "unsupported_word_type",
                    format!(
                        "root member {ordinal} has unsupported t value {:?}",
                        class.as_deref().unwrap_or_default()
                    ),
                    Some(occurrence.clone()),
                );
            }
            if member.key.is_empty() {
                add_diagnostic(
                    &mut diagnostics,
                    &mut release_diagnostics,
                    "invalid_word",
                    format!("root member {ordinal} has an empty decoded key"),
                    Some(occurrence.clone()),
                );
            }
        }
        for path in &duplicate_paths {
            let escaped = path
                .iter()
                .map(|part| pointer_segment(part))
                .collect::<Vec<_>>()
                .join("/");
            add_diagnostic(
                &mut diagnostics,
                &mut release_diagnostics,
                "duplicate_key",
                format!(
                    "root member {ordinal} contains a repeated JSON key at /{ordinal}/{escaped}; typed body projection suppressed"
                ),
                Some(occurrence.clone()),
            );
        }

        if !is_object {
            continue;
        }
        let mut known_extensions = Vec::new();
        for (field_index, child) in fields.iter().enumerate() {
            let model_index = field_index + 1;
            if child.name == "g" {
                extensions.push(extension(
                    &assertion_id,
                    "sutysisku.g.raw".into(),
                    &child.raw,
                    evidence(&occurrence, Some(model_index)),
                )?);
            } else if child.name == "b" {
                extensions.push(extension(
                    &assertion_id,
                    "sutysisku.b".into(),
                    &child.raw,
                    evidence(&occurrence, Some(model_index)),
                )?);
            } else if !FIELDS.contains(&child.name.as_str()) {
                extensions.push(extension(
                    &assertion_id,
                    format!("sutysisku.native_field.{model_index}.{}", child.name),
                    &child.raw,
                    evidence(&occurrence, Some(model_index)),
                )?);
                known_extensions.push(child.name.as_str());
            }
        }
        for name in known_extensions {
            add_diagnostic(
                &mut diagnostics,
                &mut release_diagnostics,
                "unknown_native_field",
                format!(
                    "root member {ordinal} unknown child field {name:?} retained as a raw extension"
                ),
                Some(occurrence.clone()),
            );
        }
        if ambiguous {
            continue;
        }

        if let Some(gloss) = unique_field(&fields, "g") {
            if gloss.raw == "null" {
                // Explicit native null has no gloss context; the whole field remains an extension.
            } else if let Ok(items) = serde_json::from_str::<Vec<String>>(&gloss.raw) {
                let field_index = fields
                    .iter()
                    .position(|item| std::ptr::eq(item, gloss))
                    .unwrap_or(0)
                    + 1;
                for (item_index, item) in items.into_iter().enumerate() {
                    contexts.push(ContextRecord {
                        id: entity(format!(
                            "sutysisku-{actual}-context-gloss-{ordinal}-{item_index}"
                        ))?,
                        assertion: assertion_id.clone(),
                        kind: ContextKind::Gloss,
                        value: Presence::Value(text(item, "en")),
                        translated_from: None,
                        evidence: evidence(&occurrence, Some(field_index)),
                    });
                }
            } else {
                add_diagnostic(
                    &mut diagnostics,
                    &mut release_diagnostics,
                    "invalid_glosses",
                    format!(
                        "root member {ordinal} g is not null or an array of strings; raw value retained"
                    ),
                    Some(occurrence.clone()),
                );
            }
        }
        if let Some(note) = unique_field(&fields, "n") {
            let field_index = fields
                .iter()
                .position(|item| std::ptr::eq(item, note))
                .unwrap_or(0)
                + 1;
            match as_json_string(&note.raw) {
                Some(value) => contexts.push(ContextRecord {
                    id: entity(format!("sutysisku-{actual}-context-note-{ordinal}"))?,
                    assertion: assertion_id.clone(),
                    kind: ContextKind::Note,
                    value: Presence::Value(text(value, "en")),
                    translated_from: None,
                    evidence: evidence(&occurrence, Some(field_index)),
                }),
                None if is_null(note) => contexts.push(ContextRecord {
                    id: entity(format!("sutysisku-{actual}-context-note-{ordinal}"))?,
                    assertion: assertion_id.clone(),
                    kind: ContextKind::Note,
                    value: Presence::Null,
                    translated_from: None,
                    evidence: evidence(&occurrence, Some(field_index)),
                }),
                None => add_diagnostic(
                    &mut diagnostics,
                    &mut release_diagnostics,
                    "invalid_notes",
                    format!("root member {ordinal} n is not a string or null; raw value retained"),
                    Some(occurrence.clone()),
                ),
            }
        }
        if let Some(selmaho) = unique_field(&fields, "s")
            && !selmaho.raw.starts_with('"')
            && !is_null(selmaho)
        {
            add_diagnostic(
                &mut diagnostics,
                &mut release_diagnostics,
                "invalid_selmaho",
                format!("root member {ordinal} s is not a string or null; raw value retained"),
                Some(occurrence.clone()),
            );
        }
        if let Some(reported) = unique_field(&fields, "r") {
            if is_null(reported) {
                // Native null is retained and yields no source-reported forms.
            } else if let Ok(items) = serde_json::from_str::<Vec<String>>(&reported.raw) {
                let field_index = fields
                    .iter()
                    .position(|item| std::ptr::eq(item, reported))
                    .unwrap_or(0)
                    + 1;
                for (item_index, spelling) in items.into_iter().enumerate() {
                    forms.push(FormRecord {
                        id: entity(format!("sutysisku-{actual}-form-{ordinal}-r-{item_index}"))?,
                        entry: entry_id.clone(),
                        spelling,
                        language: Presence::Value("jbo".into()),
                        script: Presence::Missing,
                        pronunciation: Presence::Missing,
                        kind: FormKind::SourceReportedForm,
                        standing: Presence::Value("r".into()),
                        components: Vec::new(),
                        evidence: Some(evidence(&occurrence, Some(field_index))),
                        computation: None,
                    });
                }
            } else {
                add_diagnostic(
                    &mut diagnostics,
                    &mut release_diagnostics,
                    "invalid_reported_form",
                    format!(
                        "root member {ordinal} r is not null or an array of strings; raw value retained"
                    ),
                    Some(occurrence.clone()),
                );
            }
        }
    }

    let registry = SourceRegistry::from_document(registry_doc)?;
    let root_keys = members.len() as u64;
    let native_fields = root_keys + child_field_total + non_object_rows;
    let capture = RawCapture::new(
        RAW_NAMESPACE,
        CaptureMetadata {
            source_name: "Sutysisku English projection".into(),
            format_label: "sutysisku-parsed-en-v1".into(),
            locator: raw_locator,
            revision: Presence::Value(options.revision.clone()),
            captured_at: Presence::Missing,
            time_precision: TimePrecision::Unknown,
            notice_locator: Presence::Value(notice_locator.clone()),
        },
        raw.to_vec(),
        occurrences,
    );
    let release_id = entity(format!(
        "sutysisku-release-{}",
        sha256(
            format!(
                "vlajmaji-sutysisku-release-v1\0{}\0{actual}",
                options.revision
            )
            .as_bytes()
        )
    ))?;
    let collection_id = entity("sutysisku-en-collection".into())?;
    let component_id = entity("sutysisku-parsed-en-component".into())?;
    let excluded_count = members.len() as u64 - eligible_count;
    let release = ReleaseRecord {
        id: release_id,
        component: component_id.clone(),
        distributor: Some(distributor),
        upstream_revision: Presence::Missing,
        distributor_revision: Presence::Value(options.revision.clone()),
        captured_at: Presence::Missing,
        time_precision: TimePrecision::Unknown,
        schema_version: 1,
        normalizer: "vlajmaji-sutysisku-export".into(),
        normalizer_version: "1".into(),
        captures: vec![capture.identity.clone()],
        assertions: assertion_ids,
        accounting: ReleaseAccounting {
            input_occurrences: Some(members.len() as u64),
            normalized_occurrences: Some(members.len() as u64),
            query_excluded_occurrences: Some(excluded_count),
            unit: "source-object-member".into(),
        },
        diagnostics: release_diagnostics,
        selected: true,
        availability: Availability::Available,
        content_notices: vec![ContentNotice {
            text: Presence::Missing,
            locator: Presence::Value(notice_locator),
            license: Presence::Missing,
        }],
        redistribution: RedistributionStatus::Held,
        redistribution_evidence: vec![
            "The pinned repository's MIT software license does not establish rights to redistribute dictionary content.".into(),
            "Exact upstream lexical dictionary lineage is unestablished for this Sutysisku projection.".into(),
        ],
    };
    let dictionary = Dictionary::from_document(DictionaryDocument {
        version: 1,
        legacy_archive: None,
        registry,
        captures: vec![capture],
        metadata: DictionaryMetadata {
            catalogue_revision: 0,
            code_notices: vec![CodeNotice {
                text: "Apache-2.0".into(),
                locator: Presence::Value("LICENSE".into()),
                license: Presence::Value("Apache-2.0".into()),
            }],
            collections: vec![CollectionRecord {
                id: collection_id.clone(),
                source,
                display_name: "Sutysisku English projection".into(),
                description: Presence::Value(text(
                    "Unknown upstream description; this collection represents the pinned English projection.".into(),
                    "en",
                )),
            }],
            components: vec![ComponentRecord {
                id: component_id,
                collection: collection_id,
                native_name: "data/parsed-en.json".into(),
                languages: vec!["jbo".into()],
            }],
            releases: vec![release],
            entries,
            assertions,
            contexts,
            frames: Vec::new(),
            forms,
            etymologies: Vec::new(),
            relations: Vec::new(),
            owned_revisions: Vec::new(),
            owned_heads: Vec::new(),
            editorial_senses: Vec::new(),
            computations: Vec::new(),
            annotations: Vec::new(),
            extensions,
        },
    })?;
    let report = SutysiskuImportReport {
        version: 1,
        revision: options.revision.clone(),
        raw_sha256: actual,
        raw_bytes: raw.len() as u64,
        input_rows: members.len() as u64,
        normalized_assertions: members.len() as u64,
        lexical_occurrences: eligible_count,
        excluded_occurrences: excluded_count,
        eligible_spellings: eligible_spellings.len() as u64,
        all_spellings: all_spellings.len() as u64,
        root_keys,
        native_child_fields: child_field_total,
        non_object_rows,
        native_fields,
        word_types,
        field_counts,
        contexts: dictionary.metadata().contexts.len() as u64,
        reported_forms: dictionary.metadata().forms.len() as u64,
        extensions: dictionary.metadata().extensions.len() as u64,
        diagnostics,
    };
    Ok(SutysiskuImport { dictionary, report })
}
