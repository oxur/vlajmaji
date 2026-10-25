//! Lossless normalization of the pinned La-Lojban English JSON export.

use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::dictionary::{
    Availability, CaptureMetadata, CaptureRef, CodeNotice, CollectionRecord, ComponentRecord,
    ContentNotice, Dictionary, DictionaryDocument, DictionaryError, DictionaryMetadata, EntityId,
    Presence, RawCapture, RedistributionStatus, ReleaseAccounting, ReleaseDiagnostic,
    ReleaseRecord,
};
use crate::lensisku::{
    ProjectionContext, add_diag, contributors, decode_export, field, project_rows, sha256,
    user_name, valid_hex,
};
use crate::registry::{
    AbbreviationKind, DistributorId, DistributorRecord, OccurrenceAttribution, RegistryDocument,
    RegistryError, SnapshotAttribution, SourceId, SourceOrigin, SourceRecord, SourceRegistry,
};
use crate::{RecordIdentity, SnapshotIdentity, TimePrecision};

/// Repository revision containing the pinned public JSON export.
pub const LA_LOJBAN_PINNED_REVISION: &str = "de47055c68141c283841b1035cd7425bb32bf6ca";
/// SHA-256 of the complete pinned export.
pub const LA_LOJBAN_PINNED_RAW_SHA256: &str =
    "861e71f66173b383298f6b1823e3ee1ed76db4b9c3fb97e7177de1fd7b667d35";
const RAW_NAMESPACE: &str = "captured:lensisku:la-lojban:en";

/// Pins and provenance locators for one local La-Lojban import.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaLojbanImportOptions {
    /// Exact forty-character repository revision.
    pub revision: String,
    /// Expected SHA-256 of the full raw export.
    pub expected_raw_sha256: String,
    /// Source locator retained in capture metadata.
    pub raw_locator: Option<String>,
    /// Notice locator retained with the capture.
    pub notice_locator: Option<String>,
}
impl Default for LaLojbanImportOptions {
    fn default() -> Self {
        Self {
            revision: LA_LOJBAN_PINNED_REVISION.into(),
            expected_raw_sha256: LA_LOJBAN_PINNED_RAW_SHA256.into(),
            raw_locator: None,
            notice_locator: None,
        }
    }
}

/// Deterministic population report for one La-Lojban import.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LaLojbanImportReport {
    /// Report schema version.
    pub version: u32,
    /// Distributor repository revision used for this import.
    pub revision: String,
    /// Full raw-export SHA-256.
    pub raw_sha256: String,
    /// Exact input byte count.
    pub raw_bytes: u64,
    /// Number of source-array rows.
    pub input_rows: u64,
    /// Number of ordered top-level native fields.
    pub native_fields: u64,
    /// Number of normalized assertions.
    pub normalized_assertions: u64,
    /// Number of lexical occurrences.
    pub lexical_occurrences: u64,
    /// Number of retained but excluded occurrences.
    pub excluded_occurrences: u64,
    /// Number of distinct exact eligible spellings.
    pub eligible_spellings: u64,
    /// Number of distinct string spellings across all rows.
    pub all_spellings: u64,
    /// Number of source-scoped contributors.
    pub contributors: u64,
    /// Counts of valid string-valued native word types.
    pub word_types: BTreeMap<String, u64>,
    /// Counts of ordered top-level native fields.
    pub field_counts: BTreeMap<String, u64>,
    /// Per-code diagnostics; not an excluded-row count.
    pub diagnostics: BTreeMap<String, u64>,
}

/// Imported dictionary and its deterministic report.
#[derive(Debug, Clone)]
pub struct LaLojbanImport {
    dictionary: Dictionary,
    report: LaLojbanImportReport,
}
impl LaLojbanImport {
    /// Borrow the imported dictionary.
    pub fn dictionary(&self) -> &Dictionary {
        &self.dictionary
    }
    /// Borrow the deterministic report.
    pub fn report(&self) -> &LaLojbanImportReport {
        &self.report
    }
    /// Consume the result and return its dictionary.
    pub fn into_dictionary(self) -> Dictionary {
        self.dictionary
    }
}

/// Typed failures while validating or normalizing a La-Lojban export.
#[derive(Debug)]
#[non_exhaustive]
pub enum LaLojbanImportError {
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
    /// The export is malformed JSON or not a JSON row array.
    InvalidJson(String),
    /// Registry attribution was rejected.
    Registry(RegistryError),
    /// The normalized dictionary model was rejected.
    Dictionary(DictionaryError),
}
impl fmt::Display for LaLojbanImportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidOptions(s) => write!(f, "invalid La-Lojban import options: {s}"),
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
impl std::error::Error for LaLojbanImportError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Registry(e) => Some(e),
            Self::Dictionary(e) => Some(e),
            _ => None,
        }
    }
}
impl From<RegistryError> for LaLojbanImportError {
    fn from(e: RegistryError) -> Self {
        Self::Registry(e)
    }
}
impl From<DictionaryError> for LaLojbanImportError {
    fn from(e: DictionaryError) -> Self {
        Self::Dictionary(e)
    }
}

fn locator(value: &Option<String>, label: &str) -> Result<String, LaLojbanImportError> {
    let value = value
        .as_deref()
        .ok_or_else(|| LaLojbanImportError::InvalidOptions(format!("{label} is required")))?;
    if value.is_empty() || value.trim() != value || value.chars().any(char::is_control) {
        return Err(LaLojbanImportError::InvalidOptions(format!(
            "{label} must be nonempty, trimmed, and control-free"
        )));
    }
    Ok(value.to_owned())
}
fn snapshot(namespace: &str, digest: &str) -> Result<SnapshotIdentity, LaLojbanImportError> {
    serde_json::from_value(serde_json::json!({"namespace":namespace,"sha256":digest}))
        .map_err(|e| LaLojbanImportError::InvalidJson(format!("internal capture identity: {e}")))
}
fn occurrence(
    namespace: &str,
    digest: &str,
    path: &str,
) -> Result<RecordIdentity, LaLojbanImportError> {
    serde_json::from_value(
        serde_json::json!({"snapshot":{"namespace":namespace,"sha256":digest},"locator":path}),
    )
    .map_err(|e| LaLojbanImportError::InvalidJson(format!("internal occurrence identity: {e}")))
}
fn entity(value: String) -> Result<EntityId, LaLojbanImportError> {
    EntityId::new(value).map_err(LaLojbanImportError::Dictionary)
}
fn source_id(value: &str) -> Result<SourceId, LaLojbanImportError> {
    SourceId::new(value).map_err(Into::into)
}
fn pointer_segment(value: &str) -> String {
    value.replace('~', "~0").replace('/', "~1")
}

/// Normalize caller-supplied pinned La-Lojban bytes without filesystem or network access.
///
/// The complete input bytes and every ordered native field remain attached to the
/// resulting dictionary. Unsupported rows remain represented as excluded occurrences.
///
/// # Errors
///
/// Returns a typed error for invalid pins/locators, hash or UTF-8 failures, malformed
/// JSON, invalid attribution, or a rejected normalized dictionary.
pub fn import_la_lojban_export(
    raw: &[u8],
    options: &LaLojbanImportOptions,
) -> Result<LaLojbanImport, LaLojbanImportError> {
    if !valid_hex(&options.revision, 40) {
        return Err(LaLojbanImportError::InvalidOptions(
            "revision must be 40 lowercase hexadecimal characters".into(),
        ));
    }
    if !valid_hex(&options.expected_raw_sha256, 64) {
        return Err(LaLojbanImportError::InvalidOptions(
            "expected raw SHA-256 must be 64 lowercase hexadecimal characters".into(),
        ));
    }
    let raw_locator = match &options.raw_locator {
        Some(_) => locator(&options.raw_locator, "raw locator")?,
        None => format!(
            "https://github.com/La-Lojban/lojban/blob/{}/assets/dumps/en.json",
            options.revision
        ),
    };
    let notice_locator = match &options.notice_locator {
        Some(_) => locator(&options.notice_locator, "notice locator")?,
        None => format!(
            "https://github.com/La-Lojban/lojban/blob/{}/README.md",
            options.revision
        ),
    };
    let actual = sha256(raw);
    if actual != options.expected_raw_sha256 {
        return Err(LaLojbanImportError::HashMismatch {
            expected: options.expected_raw_sha256.clone(),
            actual,
        });
    }
    let text = std::str::from_utf8(raw).map_err(|_| LaLojbanImportError::InvalidUtf8)?;
    let decoded = decode_export(text).map_err(LaLojbanImportError::InvalidJson)?;
    let source = source_id("lensisku")?;
    let distributor = DistributorId::new("la-lojban")?;
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
        display_name: "La-Lojban".into(),
    });
    let contributor_set = contributors(&decoded.users)?;
    registry_doc.contributors.extend(contributor_set.records);
    let capture_identity = CaptureRef {
        namespace: RAW_NAMESPACE.into(),
        sha256: actual.clone(),
    };
    let snapshot_id = snapshot(RAW_NAMESPACE, &actual)?;
    registry_doc.snapshots.push(SnapshotAttribution {
        snapshot: snapshot_id,
        source: source.clone(),
        distributor: Some(distributor.clone()),
    });
    for (index, row) in decoded.rows.iter().enumerate() {
        let path = format!("/{index}");
        let assertion = if row.duplicates.is_empty() {
            field(&row.fields, "definition_id").and_then(|f| {
                serde_json::from_str::<serde_json::Value>(&f.raw)
                    .ok()
                    .filter(|v| v.is_string() || v.is_number())
                    .map(|_| f.raw.clone())
            })
        } else {
            None
        };
        let contributor = if row.duplicates.is_empty() {
            user_name(&row.fields).and_then(|name| contributor_set.ids.get(&name).cloned())
        } else {
            None
        };
        registry_doc.occurrences.push(OccurrenceAttribution {
            occurrence: occurrence(RAW_NAMESPACE, &actual, &path)?,
            contributor,
            native_entry_id: None,
            native_assertion_id: assertion,
        });
    }
    let registry = SourceRegistry::from_document(registry_doc)?;
    let snapshot_ref = capture_identity.clone();
    let mut diagnostics = BTreeMap::new();
    let mut release_diagnostics = Vec::<ReleaseDiagnostic>::new();
    for username in &contributor_set.fallback_usernames {
        add_diag(
            &mut diagnostics,
            &mut release_diagnostics,
            "contributor_display_fallback",
            format!(
                "username {username:?} is not a registry display value; retained under its source-scoped hash identity"
            ),
            None,
        );
    }
    for (index, row) in decoded.rows.iter().enumerate() {
        let occurrence = crate::dictionary::OccurrenceRef {
            capture: capture_identity.clone(),
            locator: format!("/{index}"),
        };
        for path in &row.duplicate_paths {
            add_diag(
                &mut diagnostics,
                &mut release_diagnostics,
                "duplicate_key_path",
                format!(
                    "row {index} contains a repeated JSON key at /{index}/{}",
                    path.iter()
                        .map(|part| pointer_segment(part))
                        .collect::<Vec<_>>()
                        .join("/")
                ),
                Some(occurrence.clone()),
            );
        }
    }
    let projection = project_rows(
        &decoded.rows,
        ProjectionContext {
            entity_prefix: "la-lojban",
            raw_sha256: &actual,
            raw_capture_ref: &snapshot_ref,
            source: &source,
            language: "en",
            source_language: "jbo",
        },
        &mut diagnostics,
        &mut release_diagnostics,
    )
    .map_err(LaLojbanImportError::Dictionary)?;
    let input_rows = decoded.rows.len() as u64;
    let release_id = entity(format!(
        "la-lojban-release-{}",
        sha256(
            format!(
                "vlajmaji-la-lojban-release-v1\0{}\0{actual}",
                options.revision
            )
            .as_bytes()
        )
    ))?;
    let collection_id = entity("lensisku-la-lojban-en-collection".into())?;
    let component_id = entity("lensisku-la-lojban-en-component".into())?;
    let capture = RawCapture::new(
        RAW_NAMESPACE,
        CaptureMetadata {
            source_name: "La-Lojban English dictionary export".into(),
            format_label: "la-lojban-export-v1".into(),
            locator: raw_locator,
            revision: Presence::Value(options.revision.clone()),
            captured_at: Presence::Missing,
            time_precision: TimePrecision::Unknown,
            notice_locator: Presence::Value(notice_locator.clone()),
        },
        raw.to_vec(),
        projection.native_occurrences,
    );
    let release = ReleaseRecord {
        id: release_id, component: component_id.clone(), distributor: Some(distributor),
        upstream_revision: Presence::Missing, distributor_revision: Presence::Value(options.revision.clone()),
        captured_at: Presence::Missing, time_precision: TimePrecision::Unknown, schema_version: 1,
        normalizer: "vlajmaji-la-lojban-export".into(), normalizer_version: "1".into(),
        captures: vec![capture.identity.clone()], assertions: projection.assertion_ids,
        accounting: ReleaseAccounting { input_occurrences: Some(input_rows), normalized_occurrences: Some(input_rows), query_excluded_occurrences: Some(projection.excluded_count), unit: "source-array-row".into() },
        diagnostics: release_diagnostics, selected: true, availability: Availability::Available,
        content_notices: vec![ContentNotice { text: Presence::Missing, locator: Presence::Value(notice_locator), license: Presence::Missing }],
        redistribution: RedistributionStatus::Held,
        redistribution_evidence: vec!["The repository software license does not establish rights to redistribute dictionary content.".into()],
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
                display_name: "La-Lojban English lexical export".into(),
                description: Presence::Missing,
            }],
            components: vec![ComponentRecord {
                id: component_id,
                collection: collection_id,
                native_name: "assets/dumps/en.json".into(),
                languages: vec!["jbo".into()],
            }],
            releases: vec![release],
            entries: projection.entries,
            assertions: projection.assertions,
            contexts: projection.contexts,
            frames: Vec::new(),
            forms: projection.forms,
            etymologies: projection.etymologies,
            relations: Vec::new(),
            owned_revisions: Vec::new(),
            owned_heads: Vec::new(),
            editorial_senses: Vec::new(),
            computations: Vec::new(),
            annotations: Vec::new(),
            extensions: projection.extensions,
        },
    })?;
    let report = LaLojbanImportReport {
        version: 1,
        revision: options.revision.clone(),
        raw_sha256: actual,
        raw_bytes: raw.len() as u64,
        input_rows,
        native_fields: decoded.native_fields,
        normalized_assertions: input_rows,
        lexical_occurrences: projection.eligible_count,
        excluded_occurrences: projection.excluded_count,
        eligible_spellings: decoded.eligible_spellings,
        all_spellings: decoded.all_spellings,
        contributors: contributor_set.ids.len() as u64,
        word_types: decoded.word_types,
        field_counts: decoded.field_counts,
        diagnostics,
    };
    Ok(LaLojbanImport { dictionary, report })
}
