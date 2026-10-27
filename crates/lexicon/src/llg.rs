//! Lossless normalization of the Logical Language Group's fixed-column lists.
//!
//! The original four byte captures remain authoritative. This module adds a
//! conservative typed projection without interpreting prose as linguistic
//! structure.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::dictionary::{
    AssertionRecord, Availability, CaptureMetadata, CaptureRef, CodeNotice, CollectionRecord,
    ComponentRecord, ContentNotice, ContextKind, ContextRecord, Dictionary, DictionaryDocument,
    DictionaryError, DictionaryMetadata, Eligibility, EntityId, EntryRecord, ExtensionRecord,
    FormKind, FormRecord, NativeEncoding, NativeField, OccurrenceRef, Presence, RawCapture,
    RawJson, RawOccurrence, RedistributionStatus, ReleaseAccounting, ReleaseDiagnostic,
    ReleaseRecord, SourceEvidence, TextValue,
};
use crate::registry::{
    AbbreviationKind, OccurrenceAttribution, RegistryDocument, RegistryError, SnapshotAttribution,
    SourceId, SourceOrigin, SourceRecord, SourceRegistry,
};
use crate::{SnapshotIdentity, TimePrecision};

/// Exact source URLs used by the locally acquired LLG list-family pin.
pub const LLG_BASE_LOCATOR: &str = "https://www.lojban.org/publications/wordlists";
/// SHA-256 of the complete gismu list.
pub const LLG_GISMU_SHA256: &str =
    "f2fc4cff4f265ad02717d56733b4627ffa78c616db78dcaa44caab694e1dd5b7";
/// SHA-256 of the complete cmavo list.
pub const LLG_CMAVO_SHA256: &str =
    "544414d61f3ff35dd2cf4e4f79f3c86bf9bb0568eb3403f57288057e16796402";
/// SHA-256 of the complete assigned rafsi list.
pub const LLG_RAFSI_SHA256: &str =
    "31b90a8551cb3cb9f4d0c6873867af1906e9210c398bf49f20e5c21aa02d0d24";
/// SHA-256 of the complete source public-domain notice.
pub const LLG_NOTICE_SHA256: &str =
    "9fb2ae1345252ba30999523a0980e1ae69795f5940d2fd9cea8a56c67be18a55";

/// Caller-controlled byte identities and locator base for one import.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LlgImportOptions {
    /// Expected complete gismu-list digest.
    pub expected_gismu_sha256: String,
    /// Expected complete cmavo-list digest.
    pub expected_cmavo_sha256: String,
    /// Expected complete rafsi-list digest.
    pub expected_rafsi_sha256: String,
    /// Expected complete notice digest.
    pub expected_notice_sha256: String,
    /// Base locator used to construct component and notice locators.
    pub base_locator: String,
}

impl Default for LlgImportOptions {
    fn default() -> Self {
        Self {
            expected_gismu_sha256: LLG_GISMU_SHA256.into(),
            expected_cmavo_sha256: LLG_CMAVO_SHA256.into(),
            expected_rafsi_sha256: LLG_RAFSI_SHA256.into(),
            expected_notice_sha256: LLG_NOTICE_SHA256.into(),
            base_locator: LLG_BASE_LOCATOR.into(),
        }
    }
}

/// A source-declared row count retained from one physical component header.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LlgHeaderClaim {
    /// Exact source occurrence for this header row.
    pub occurrence: OccurrenceRef,
    /// Body row count declared by this header.
    pub declared_body_count: u64,
    /// Whether this claim matches the observed number of body rows.
    pub count_matches: bool,
}

/// Per-component source and projection accounting.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LlgComponentReport {
    /// Stable list-family component name.
    pub component: String,
    /// Complete captured input byte length.
    pub input_bytes: u64,
    /// Complete captured input SHA-256.
    pub input_sha256: String,
    /// Number of physical lines represented as raw occurrences.
    pub input_lines: u64,
    /// Number of source-native fields retained across physical rows.
    pub native_fields: u64,
    /// Definition-bearing lexical assertions.
    pub lexical_assertions: u64,
    /// Supported rafsi rows retained as form-only assertions.
    pub form_only_assertions: u64,
    /// Retained but nonlexical assertions.
    pub excluded_assertions: u64,
    /// Rows that are recognized list headers.
    pub headers: u64,
    /// Unsupported, malformed, blank, separator, or non-ASCII rows.
    pub unsupported_rows: u64,
    /// All source header claims in physical order.
    pub header_claims: Vec<LlgHeaderClaim>,
    /// Agreed source-declared body row count, or `None` when absent/conflicting.
    pub declared_body_count: Option<u64>,
    /// Number of physical non-header rows; headers may occur anywhere.
    pub observed_body_count: u64,
    /// Whether agreed source claims match the body count; `None` if absent or conflicting.
    pub declared_count_matches: Option<bool>,
    /// Number of assigned-rafsi form assertions supported by this component.
    pub assigned_forms: u64,
    /// Number of nonempty normalized context rows.
    pub contexts: u64,
    /// Counts of diagnostic codes for this component.
    pub diagnostics: BTreeMap<String, u64>,
}

/// Versioned summary of one complete LLG list-family import.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LlgImportReport {
    /// Report schema version.
    pub version: u32,
    /// Component summaries in gismu, cmavo, rafsi order.
    pub components: Vec<LlgComponentReport>,
    /// Number of complete captured source snapshots.
    pub captures: u64,
    /// Number of raw occurrences and normalized assertions.
    pub occurrences: u64,
    /// Number of distinct lexical entries, including non-entry placeholders.
    pub entries: u64,
    /// Definition-bearing assertions eligible for lexical use.
    pub lexical_assertions: u64,
    /// Retained assertions excluded from lexical use.
    pub excluded_assertions: u64,
    /// Number of distinct eligible headword spellings.
    pub eligible_spellings: u64,
    /// Number of distinct spellings among all entries with a spelling.
    pub all_spellings: u64,
    /// Number of exact ordered native fields retained.
    pub native_fields: u64,
    /// Number of assigned-rafsi form rows and slots.
    pub assigned_forms: u64,
    /// Number of evidence-bearing contexts.
    pub contexts: u64,
    /// Number of historical native teaching-code extension rows.
    pub teaching_code_extensions: u64,
    /// Number of historical frequency extension rows.
    pub frequency_extensions: u64,
    /// Total number of retained historical extension rows.
    pub extensions: u64,
    /// Overall counts of release diagnostics by code.
    pub diagnostics: BTreeMap<String, u64>,
}

/// Validated dictionary and its deterministic source-derived import report.
#[derive(Debug, Clone)]
pub struct LlgImport {
    dictionary: Dictionary,
    report: LlgImportReport,
}

impl LlgImport {
    /// Borrow the completely validated dictionary.
    #[must_use]
    pub fn dictionary(&self) -> &Dictionary {
        &self.dictionary
    }

    /// Borrow the source-derived component and population report.
    #[must_use]
    pub fn report(&self) -> &LlgImportReport {
        &self.report
    }

    /// Consume the import result and return its validated dictionary.
    #[must_use]
    pub fn into_dictionary(self) -> Dictionary {
        self.dictionary
    }
}

/// Typed failure while validating LLG pins, bytes, registry, or model.
#[derive(Debug)]
#[non_exhaustive]
pub enum LlgImportError {
    /// A caller supplied an invalid pin or base locator.
    InvalidOptions(String),
    /// A complete input digest differs from its expected pin.
    HashMismatch {
        /// Input whose digest differs (`gismu`, `cmavo`, `rafsi`, or `notice`).
        input: String,
        /// Expected digest supplied by the caller.
        expected: String,
        /// Digest computed over all supplied bytes.
        actual: String,
    },
    /// An input is not valid UTF-8.
    InvalidUtf8 {
        /// Input whose decoding failed.
        input: String,
    },
    /// The source registry rejected a constructed attribution graph.
    Registry(RegistryError),
    /// The typed dictionary rejected a constructed normalized model.
    Dictionary(DictionaryError),
}

impl fmt::Display for LlgImportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidOptions(message) => write!(f, "invalid LLG import options: {message}"),
            Self::HashMismatch {
                input,
                expected,
                actual,
            } => write!(
                f,
                "{input} SHA-256 mismatch: expected {expected}, got {actual}"
            ),
            Self::InvalidUtf8 { input } => write!(f, "{input} input is not UTF-8"),
            Self::Registry(error) => write!(f, "invalid source registry: {error}"),
            Self::Dictionary(error) => write!(f, "invalid dictionary: {error}"),
        }
    }
}

impl std::error::Error for LlgImportError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Registry(error) => Some(error),
            Self::Dictionary(error) => Some(error),
            _ => None,
        }
    }
}

impl From<RegistryError> for LlgImportError {
    fn from(value: RegistryError) -> Self {
        Self::Registry(value)
    }
}

impl From<DictionaryError> for LlgImportError {
    fn from(value: DictionaryError) -> Self {
        Self::Dictionary(value)
    }
}

#[derive(Clone, Copy)]
enum ComponentKind {
    Gismu,
    Cmavo,
    Rafsi,
}

impl ComponentKind {
    const fn name(self) -> &'static str {
        match self {
            Self::Gismu => "gismu",
            Self::Cmavo => "cmavo",
            Self::Rafsi => "rafsi",
        }
    }

    const fn file_name(self) -> &'static str {
        match self {
            Self::Gismu => "gismu.txt",
            Self::Cmavo => "cmavo.txt",
            Self::Rafsi => "rafsi.txt",
        }
    }

    const fn normalizer_format(self) -> &'static str {
        match self {
            Self::Gismu => "llg-gismu-fixed-columns",
            Self::Cmavo => "llg-cmavo-fixed-columns",
            Self::Rafsi => "llg-assigned-rafsi-rows",
        }
    }
}

struct ParsedRow {
    locator: String,
    raw_line: String,
    fields: Vec<NativeField>,
    kind: ParsedKind,
}

enum ParsedKind {
    Header(u64),
    Lexical {
        spelling: String,
        definition: String,
    },
    Rafsi {
        headword: String,
        form: String,
        gloss: String,
    },
    Unsupported(String),
}

/// Import all three complete LLG text lists and their shared public-domain notice.
///
/// Input pins are checked before UTF-8 decoding. Every physical line becomes a
/// source occurrence; unsupported rows remain visible as explicitly excluded
/// assertions. The caller's bytes are never normalized or modified.
///
/// # Errors
///
/// Returns a typed options, hash, UTF-8, registry, or dictionary error.
pub fn import_llg_lists(
    gismu: &[u8],
    cmavo: &[u8],
    rafsi: &[u8],
    notice: &[u8],
    options: &LlgImportOptions,
) -> Result<LlgImport, LlgImportError> {
    validate_options(options)?;
    let inputs = [
        ("gismu", gismu, options.expected_gismu_sha256.as_str()),
        ("cmavo", cmavo, options.expected_cmavo_sha256.as_str()),
        ("rafsi", rafsi, options.expected_rafsi_sha256.as_str()),
        ("notice", notice, options.expected_notice_sha256.as_str()),
    ];
    let mut hashes = BTreeMap::new();
    for (name, bytes, expected) in inputs {
        let actual = sha256(bytes);
        if actual != expected {
            return Err(LlgImportError::HashMismatch {
                input: name.into(),
                expected: expected.into(),
                actual,
            });
        }
        hashes.insert(name, actual);
    }
    let gismu_text = std::str::from_utf8(gismu).map_err(|_| LlgImportError::InvalidUtf8 {
        input: "gismu".into(),
    })?;
    let cmavo_text = std::str::from_utf8(cmavo).map_err(|_| LlgImportError::InvalidUtf8 {
        input: "cmavo".into(),
    })?;
    let rafsi_text = std::str::from_utf8(rafsi).map_err(|_| LlgImportError::InvalidUtf8 {
        input: "rafsi".into(),
    })?;
    let notice_text = std::str::from_utf8(notice).map_err(|_| LlgImportError::InvalidUtf8 {
        input: "notice".into(),
    })?;

    let mut parsed = vec![
        parse_component(ComponentKind::Gismu, gismu_text),
        parse_component(ComponentKind::Cmavo, cmavo_text),
        parse_component(ComponentKind::Rafsi, rafsi_text),
    ];
    let source = SourceId::new("llg")?;
    let collection_id = eid("llg-list-family-collection")?;
    let source_registry = build_registry(&source, &parsed, &hashes)?;
    let (metadata, captures, report) = build_model(BuildModelInput {
        source: &source,
        collection_id,
        notice_text,
        notice,
        component_bytes: [gismu, cmavo, rafsi],
        options,
        hashes: &hashes,
        parsed: &mut parsed,
    })?;
    let dictionary = Dictionary::from_document(DictionaryDocument {
        version: 1,
        legacy_archive: None,
        registry: source_registry,
        captures,
        metadata,
    })?;
    Ok(LlgImport { dictionary, report })
}

fn validate_options(options: &LlgImportOptions) -> Result<(), LlgImportError> {
    for (name, digest) in [
        ("gismu", options.expected_gismu_sha256.as_str()),
        ("cmavo", options.expected_cmavo_sha256.as_str()),
        ("rafsi", options.expected_rafsi_sha256.as_str()),
        ("notice", options.expected_notice_sha256.as_str()),
    ] {
        if digest.len() != 64
            || !digest
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(LlgImportError::InvalidOptions(format!(
                "expected_{name}_sha256 must contain 64 lowercase hexadecimal characters"
            )));
        }
    }
    if options.base_locator.trim().is_empty() || options.base_locator.chars().any(char::is_control)
    {
        return Err(LlgImportError::InvalidOptions(
            "base_locator must be nonempty and contain no control characters".into(),
        ));
    }
    Ok(())
}

fn parse_component(kind: ComponentKind, text: &str) -> Vec<ParsedRow> {
    physical_lines(text)
        .into_iter()
        .enumerate()
        .map(|(index, line)| parse_line(kind, index + 1, line))
        .collect()
}

fn physical_lines(text: &str) -> Vec<&str> {
    if text.is_empty() {
        return Vec::new();
    }
    text.split_inclusive('\n')
        .map(|line| {
            if let Some(without_lf) = line.strip_suffix('\n') {
                without_lf.strip_suffix('\r').unwrap_or(without_lf)
            } else {
                line
            }
        })
        .collect()
}

fn parse_line(kind: ComponentKind, line_number: usize, line: &str) -> ParsedRow {
    let locator = line_number.to_string();
    let line_field = || NativeField {
        name: "$line".into(),
        encoding: NativeEncoding::Text,
        raw_value: line.into(),
    };
    if !line.is_ascii() {
        return ParsedRow {
            locator,
            raw_line: line.into(),
            fields: vec![line_field()],
            kind: ParsedKind::Unsupported("non_ascii_line".into()),
        };
    }
    if matches!(kind, ComponentKind::Gismu | ComponentKind::Cmavo)
        && let Some(count) = header_count(line)
    {
        return ParsedRow {
            locator,
            raw_line: line.into(),
            fields: vec![line_field()],
            kind: ParsedKind::Header(count),
        };
    }
    match kind {
        ComponentKind::Gismu => parse_gismu(locator, line, line_field),
        ComponentKind::Cmavo => parse_cmavo(locator, line, line_field),
        ComponentKind::Rafsi => parse_rafsi(locator, line, line_field),
    }
}

fn header_count(line: &str) -> Option<u64> {
    let mut parts = line.split_whitespace();
    let first = parts.next()?;
    if first.len() != 5
        || !first.bytes().all(|byte| byte.is_ascii_digit())
        || parts.next()? != "Lojban"
    {
        return None;
    }
    first.parse().ok()
}

fn field(name: &str, raw_value: &str) -> NativeField {
    NativeField {
        name: name.into(),
        encoding: NativeEncoding::Text,
        raw_value: raw_value.into(),
    }
}

fn parse_gismu(locator: String, line: &str, line_field: impl FnOnce() -> NativeField) -> ParsedRow {
    if line.trim().is_empty() {
        return unsupported(locator, line, line_field(), "blank_line");
    }
    if line.len() < 158 {
        return unsupported(locator, line, line_field(), "short_gismu_line");
    }
    let mut fields = vec![
        field("word", &line[1..6]),
        field("assigned_rafsi_cvc", &line[7..10]),
        field("assigned_rafsi_ccv", &line[11..14]),
        field("assigned_rafsi_cvv", &line[15..19]),
        field("keyword", &line[20..40]),
        field("hint", &line[41..61]),
        field("definition", &line[62..158]),
    ];
    for (name, start, end) in [
        ("teaching_code", 159, 161),
        ("frequency_1993", 161, 165),
        ("notes", 165, usize::MAX),
    ] {
        if line.len() > start {
            fields.push(field(name, &line[start..line.len().min(end)]));
        }
    }
    let spelling = line[1..6].trim().to_owned();
    let definition = line[62..158].trim().to_owned();
    if spelling.is_empty() || line[20..40].trim().is_empty() || definition.is_empty() {
        return unsupported(locator, line, field("$line", line), "malformed_gismu_row");
    }
    ParsedRow {
        locator,
        raw_line: line.into(),
        fields,
        kind: ParsedKind::Lexical {
            spelling,
            definition,
        },
    }
}

fn parse_cmavo(locator: String, line: &str, line_field: impl FnOnce() -> NativeField) -> ParsedRow {
    if line.trim().is_empty() {
        return unsupported(locator, line, line_field(), "blank_line");
    }
    if line.len() < 62 {
        return unsupported(locator, line, line_field(), "short_cmavo_line");
    }
    let fields = vec![
        field("word", &line[..10]),
        field("selmaho", &line[10..20]),
        field("keyword", &line[20..61]),
        field("definition", &line[61..]),
    ];
    let spelling = line[..10].trim().to_owned();
    let definition = line[61..].trim().to_owned();
    if spelling.is_empty()
        || line[10..20].trim().is_empty()
        || line[20..61].trim().is_empty()
        || definition.is_empty()
    {
        return unsupported(locator, line, field("$line", line), "malformed_cmavo_row");
    }
    ParsedRow {
        locator,
        raw_line: line.into(),
        fields,
        kind: ParsedKind::Lexical {
            spelling,
            definition,
        },
    }
}

fn parse_rafsi(locator: String, line: &str, line_field: impl FnOnce() -> NativeField) -> ParsedRow {
    if line.trim().is_empty() {
        return unsupported(locator, line, line_field(), "blank_line");
    }
    let spans = token_spans(line);
    if spans.len() < 3 {
        return unsupported(locator, line, line_field(), "malformed_rafsi_row");
    }
    let (rafsi_start, rafsi_end) = spans[0];
    let (head_start, head_end) = spans[1];
    let gloss = &line[head_end..];
    if line[rafsi_start..rafsi_end].is_empty()
        || line[head_start..head_end].is_empty()
        || gloss.trim().is_empty()
    {
        return unsupported(locator, line, line_field(), "malformed_rafsi_row");
    }
    ParsedRow {
        locator,
        raw_line: line.into(),
        fields: vec![
            field("rafsi", &line[rafsi_start..rafsi_end]),
            field("word", &line[head_start..head_end]),
            field("gloss", gloss),
        ],
        kind: ParsedKind::Rafsi {
            headword: line[head_start..head_end].into(),
            form: line[rafsi_start..rafsi_end].into(),
            gloss: gloss.trim().into(),
        },
    }
}

fn token_spans(line: &str) -> Vec<(usize, usize)> {
    let bytes = line.as_bytes();
    let mut spans = Vec::new();
    let mut cursor = 0;
    while cursor < bytes.len() {
        while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        let start = cursor;
        while cursor < bytes.len() && !bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        if start < cursor {
            spans.push((start, cursor));
        }
    }
    spans
}

fn unsupported(locator: String, line: &str, field: NativeField, reason: &str) -> ParsedRow {
    ParsedRow {
        locator,
        raw_line: line.into(),
        fields: vec![field],
        kind: ParsedKind::Unsupported(reason.into()),
    }
}

fn build_registry(
    source: &SourceId,
    parsed: &[Vec<ParsedRow>],
    hashes: &BTreeMap<&str, String>,
) -> Result<SourceRegistry, LlgImportError> {
    let mut registry = RegistryDocument::default();
    registry.sources.push(SourceRecord {
        id: source.clone(),
        display_name: "The Logical Language Group, Inc.".into(),
        abbreviation: "LLG".into(),
        abbreviation_kind: AbbreviationKind::ProjectAssigned,
        abbreviation_evidence: None,
        origin: SourceOrigin::Upstream,
    });
    for (index, kind) in [
        ComponentKind::Gismu,
        ComponentKind::Cmavo,
        ComponentKind::Rafsi,
    ]
    .into_iter()
    .enumerate()
    {
        let namespace = format!("captured:llg:{}", kind.name());
        let digest = hashes[kind.name()].clone();
        let snapshot = snapshot_identity(&namespace, &digest)?;
        registry.snapshots.push(SnapshotAttribution {
            snapshot,
            source: source.clone(),
            distributor: None,
        });
        for row in &parsed[index] {
            let occurrence = occurrence_identity(&namespace, &digest, &row.locator)?;
            let native_entry_id = match &row.kind {
                ParsedKind::Lexical { spelling, .. } => Some(spelling.clone()),
                ParsedKind::Rafsi { headword, .. } => Some(headword.clone()),
                ParsedKind::Header(_) | ParsedKind::Unsupported(_) => None,
            };
            registry.occurrences.push(OccurrenceAttribution {
                occurrence,
                contributor: None,
                native_entry_id,
                native_assertion_id: None,
            });
        }
    }
    let notice_namespace = "captured:llg:notice";
    registry.snapshots.push(SnapshotAttribution {
        snapshot: snapshot_identity(notice_namespace, &hashes["notice"])?,
        source: source.clone(),
        distributor: None,
    });
    Ok(SourceRegistry::from_document(registry)?)
}

struct BuildModelInput<'a> {
    source: &'a SourceId,
    collection_id: EntityId,
    notice_text: &'a str,
    notice: &'a [u8],
    component_bytes: [&'a [u8]; 3],
    options: &'a LlgImportOptions,
    hashes: &'a BTreeMap<&'static str, String>,
    parsed: &'a mut [Vec<ParsedRow>],
}

fn build_model(
    input: BuildModelInput<'_>,
) -> Result<(DictionaryMetadata, Vec<RawCapture>, LlgImportReport), LlgImportError> {
    let BuildModelInput {
        source,
        collection_id,
        notice_text,
        notice,
        component_bytes,
        options,
        hashes,
        parsed,
    } = input;
    let kinds = [
        ComponentKind::Gismu,
        ComponentKind::Cmavo,
        ComponentKind::Rafsi,
    ];
    let notice_locator = format!(
        "{}/public_domain.txt",
        options.base_locator.trim_end_matches('/')
    );
    let notice_capture = RawCapture::new(
        "captured:llg:notice",
        CaptureMetadata {
            source_name: "The Logical Language Group, Inc.".into(),
            format_label: "llg-public-domain-notice".into(),
            locator: notice_locator.clone(),
            revision: Presence::Missing,
            captured_at: Presence::Missing,
            time_precision: TimePrecision::Unknown,
            notice_locator: Presence::Missing,
        },
        notice.to_vec(),
        Vec::new(),
    );
    let notice_ref = notice_capture.identity.clone();
    let mut captures = vec![notice_capture];
    let mut metadata = DictionaryMetadata {
        collections: vec![CollectionRecord {
            id: collection_id.clone(),
            source: source.clone(),
            display_name: "LLG complete word lists".into(),
            description: Presence::Value(TextValue {
                text: "Gismu, cmavo, and assigned rafsi lists prepared by The Logical Language Group, Inc.".into(),
                language: Presence::Value("en".into()),
                markup: Presence::Missing,
            }),
        }],
        ..DictionaryMetadata::default()
    };
    metadata.code_notices.push(CodeNotice {
        text: "Apache-2.0".into(),
        locator: Presence::Value("LICENSE".into()),
        license: Presence::Value("Apache-2.0".into()),
    });

    let mut component_captures = Vec::new();
    for (index, kind) in kinds.into_iter().enumerate() {
        let namespace = format!("captured:llg:{}", kind.name());
        let notice_url = notice_locator.clone();
        let capture = RawCapture::new(
            namespace,
            CaptureMetadata {
                source_name: "The Logical Language Group, Inc.".into(),
                format_label: kind.normalizer_format().into(),
                locator: format!(
                    "{}/{}",
                    options.base_locator.trim_end_matches('/'),
                    kind.file_name()
                ),
                revision: Presence::Missing,
                captured_at: Presence::Missing,
                time_precision: TimePrecision::Unknown,
                notice_locator: Presence::Value(notice_url),
            },
            component_bytes[index].to_vec(),
            parsed[index]
                .iter()
                .map(|row| RawOccurrence {
                    locator: row.locator.clone(),
                    fields: row.fields.clone(),
                    raw_line: Some(row.raw_line.clone()),
                })
                .collect(),
        );
        component_captures.push(capture.identity.clone());
        captures.push(capture);
    }

    let mut reports = Vec::new();
    let mut diagnostics_total = BTreeMap::<String, u64>::new();
    let mut all_heads = BTreeSet::new();
    let mut eligible_heads = BTreeSet::new();
    let mut cmavo_classes = BTreeMap::<String, BTreeSet<String>>::new();
    let mut entries = BTreeMap::<String, EntryRecord>::new();
    let mut placeholders = Vec::<EntryRecord>::new();
    let mut assertion_ids: Vec<Vec<EntityId>> = Vec::new();
    let mut release_diagnostics: Vec<Vec<ReleaseDiagnostic>> = Vec::new();
    let mut total_fields = 0u64;
    let mut teaching_extensions = 0u64;
    let mut frequency_extensions = 0u64;

    for (index, kind) in kinds.into_iter().enumerate() {
        let capture = &component_captures[index];
        let mut component_report = LlgComponentReport {
            component: kind.name().into(),
            input_bytes: component_bytes[index].len() as u64,
            input_sha256: hashes[kind.name()].clone(),
            input_lines: parsed[index].len() as u64,
            native_fields: parsed[index]
                .iter()
                .map(|row| row.fields.len() as u64)
                .sum(),
            lexical_assertions: 0,
            form_only_assertions: 0,
            excluded_assertions: 0,
            headers: 0,
            unsupported_rows: 0,
            header_claims: Vec::new(),
            declared_body_count: None,
            observed_body_count: 0,
            declared_count_matches: None,
            assigned_forms: 0,
            contexts: 0,
            diagnostics: BTreeMap::new(),
        };
        let header_count = parsed[index]
            .iter()
            .filter(|row| matches!(row.kind, ParsedKind::Header(_)))
            .count() as u64;
        component_report.observed_body_count = component_report.input_lines - header_count;
        total_fields += component_report.native_fields;
        let mut ids = Vec::new();
        let mut diags = Vec::new();
        for row in &parsed[index] {
            let occurrence = OccurrenceRef {
                capture: capture.clone(),
                locator: row.locator.clone(),
            };
            let assertion = eid(format!(
                "llg-assertion-{}",
                hash_parts(&[
                    "llg-assertion-v1",
                    &capture.sha256,
                    kind.name(),
                    &row.locator
                ])
            ))?;
            ids.push(assertion.clone());
            let (entry_id, definition, eligibility, assertion_language) = match &row.kind {
                ParsedKind::Header(count) => {
                    component_report.headers += 1;
                    component_report.excluded_assertions += 1;
                    component_report.header_claims.push(LlgHeaderClaim {
                        occurrence: occurrence.clone(),
                        declared_body_count: *count,
                        count_matches: *count == component_report.observed_body_count,
                    });
                    *component_report
                        .diagnostics
                        .entry("source_header".into())
                        .or_default() += 1;
                    *diagnostics_total.entry("source_header".into()).or_default() += 1;
                    diags.push(ReleaseDiagnostic {
                        occurrence: Some(occurrence.clone()),
                        code: "source_header".into(),
                        message: "recognized LLG row-count header retained and excluded from lexical assertions".into(),
                    });
                    let id = placeholder_id(capture, &row.locator)?;
                    placeholders.push(placeholder_record(id.clone(), source));
                    (
                        id,
                        Presence::Missing,
                        Eligibility::Excluded {
                            reason: "source_header".into(),
                        },
                        Presence::Missing,
                    )
                }
                ParsedKind::Unsupported(code) => {
                    component_report.unsupported_rows += 1;
                    component_report.excluded_assertions += 1;
                    *component_report
                        .diagnostics
                        .entry(code.clone())
                        .or_default() += 1;
                    *diagnostics_total.entry(code.clone()).or_default() += 1;
                    diags.push(ReleaseDiagnostic {
                        occurrence: Some(occurrence.clone()),
                        code: code.clone(),
                        message: format!(
                            "physical line retained as an excluded unsupported row: {code}"
                        ),
                    });
                    let id = placeholder_id(capture, &row.locator)?;
                    placeholders.push(placeholder_record(id.clone(), source));
                    (
                        id,
                        Presence::Missing,
                        Eligibility::Excluded {
                            reason: code.clone(),
                        },
                        Presence::Missing,
                    )
                }
                ParsedKind::Lexical {
                    spelling,
                    definition,
                } => {
                    component_report.lexical_assertions += 1;
                    all_heads.insert(spelling.clone());
                    eligible_heads.insert(spelling.clone());
                    if matches!(kind, ComponentKind::Cmavo) {
                        cmavo_classes
                            .entry(spelling.clone())
                            .or_default()
                            .insert(row.fields[1].raw_value.trim().to_owned());
                    }
                    (
                        entry_id(spelling)?,
                        Presence::Value(text_value(definition.clone(), "en")),
                        Eligibility::Lexical,
                        Presence::Value("en".into()),
                    )
                }
                ParsedKind::Rafsi { headword, .. } => {
                    component_report.form_only_assertions += 1;
                    component_report.excluded_assertions += 1;
                    all_heads.insert(headword.clone());
                    (
                        entry_id(headword)?,
                        Presence::Missing,
                        Eligibility::Excluded {
                            reason: "assigned_rafsi_form_only".into(),
                        },
                        Presence::Missing,
                    )
                }
            };
            metadata.assertions.push(AssertionRecord {
                id: assertion.clone(),
                entry: entry_id,
                occurrence: occurrence.clone(),
                native_sense_id: Presence::Missing,
                language: assertion_language,
                definition,
                source_score: Presence::Missing,
                timestamps: Vec::new(),
                eligibility: eligibility.clone(),
            });
            if matches!(row.kind, ParsedKind::Lexical { .. }) {
                project_contexts(
                    &mut metadata,
                    &assertion,
                    &occurrence,
                    row,
                    kind,
                    &mut component_report.contexts,
                )?;
                for (field_index, native) in row.fields.iter().enumerate() {
                    let (name, count) = match native.name.as_str() {
                        "teaching_code" => ("llg.teaching_code", &mut teaching_extensions),
                        "frequency_1993" => ("llg.frequency_1993", &mut frequency_extensions),
                        _ => continue,
                    };
                    *count += 1;
                    metadata.extensions.push(ExtensionRecord {
                        owner: assertion.clone(),
                        name: name.into(),
                        value: RawJson::new(
                            serde_json::to_string(&native.raw_value)
                                .map_err(|e| DictionaryError::InvalidRawJson(e.to_string()))?,
                        )?,
                        evidence: Some(evidence(occurrence.clone(), field_index)),
                    });
                }
            } else if let ParsedKind::Rafsi { gloss, .. } = &row.kind {
                push_context(
                    &mut metadata,
                    &assertion,
                    &occurrence,
                    2,
                    ContextKind::Gloss,
                    gloss.clone(),
                )?;
                component_report.contexts += 1;
            }
        }
        if let Some(first_claim) = component_report.header_claims.first() {
            let claims_agree = component_report
                .header_claims
                .iter()
                .all(|claim| claim.declared_body_count == first_claim.declared_body_count);
            if claims_agree {
                component_report.declared_body_count = Some(first_claim.declared_body_count);
                component_report.declared_count_matches =
                    Some(first_claim.declared_body_count == component_report.observed_body_count);
            } else {
                let code = "conflicting_header_counts".to_owned();
                *component_report
                    .diagnostics
                    .entry(code.clone())
                    .or_default() += 1;
                *diagnostics_total.entry(code.clone()).or_default() += 1;
                let claims = component_report
                    .header_claims
                    .iter()
                    .map(|claim| {
                        format!(
                            "{} at locator {}",
                            claim.declared_body_count, claim.occurrence.locator
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                diags.push(ReleaseDiagnostic {
                    occurrence: None,
                    code,
                    message: format!(
                        "headers declare conflicting body row counts ({claims}); source bytes are retained"
                    ),
                });
            }
        }
        for claim in &component_report.header_claims {
            if claim.count_matches {
                continue;
            }
            let code = "header_count_mismatch".to_owned();
            *component_report
                .diagnostics
                .entry(code.clone())
                .or_default() += 1;
            *diagnostics_total.entry(code.clone()).or_default() += 1;
            diags.push(ReleaseDiagnostic {
                occurrence: Some(claim.occurrence.clone()),
                code,
                message: format!(
                    "header declares {} body rows but observed {}; source bytes are retained",
                    claim.declared_body_count, component_report.observed_body_count
                ),
            });
        }
        let component_id = eid(format!("llg-component-{}", kind.name()))?;
        metadata.components.push(ComponentRecord {
            id: component_id,
            collection: collection_id.clone(),
            native_name: kind.file_name().into(),
            languages: vec!["jbo".into()],
        });
        assertion_ids.push(ids);
        release_diagnostics.push(diags);
        reports.push(component_report);
    }

    for spelling in &all_heads {
        let selmaho = match cmavo_classes.get(spelling) {
            Some(classes) if classes.len() == 1 => {
                Presence::Value(classes.iter().next().cloned().unwrap_or_default())
            }
            Some(classes) if classes.len() > 1 => {
                *diagnostics_total
                    .entry("conflicting_cmavo_classes".into())
                    .or_default() += 1;
                if let Some(index) = parsed[1].iter().position(|row| {
                    matches!(&row.kind, ParsedKind::Lexical { spelling: head, .. } if head == spelling)
                }) {
                    let occurrence = OccurrenceRef {
                        capture: component_captures[1].clone(),
                        locator: parsed[1][index].locator.clone(),
                    };
                    reports[1].diagnostics.entry("conflicting_cmavo_classes".into()).and_modify(|count| *count += 1).or_insert(1);
                    release_diagnostics[1].push(ReleaseDiagnostic {
                        occurrence: Some(occurrence),
                        code: "conflicting_cmavo_classes".into(),
                        message: format!("multiple supplied cmavo classes for {spelling}: {}", classes.iter().cloned().collect::<Vec<_>>().join(", ")),
                    });
                }
                Presence::Missing
            }
            _ => Presence::Missing,
        };
        entries.insert(
            spelling.clone(),
            EntryRecord {
                id: entry_id(spelling)?,
                source: source.clone(),
                spelling: Presence::Value(spelling.clone()),
                language: Presence::Value("jbo".into()),
                script: Presence::Missing,
                native_word_class: Presence::Missing,
                selmaho,
                mapped_pos: Presence::Missing,
            },
        );
    }
    metadata.entries.extend(entries.into_values());
    metadata.entries.extend(placeholders);

    for (index, kind) in kinds.into_iter().enumerate() {
        for row in &parsed[index] {
            let occurrence = OccurrenceRef {
                capture: component_captures[index].clone(),
                locator: row.locator.clone(),
            };
            let headword = match &row.kind {
                ParsedKind::Lexical { spelling, .. } => Some(spelling.as_str()),
                ParsedKind::Rafsi { headword, .. } => Some(headword.as_str()),
                _ => None,
            };
            let Some(headword) = headword else { continue };
            let entry = metadata
                .entries
                .iter()
                .find(
                    |entry| matches!(&entry.spelling, Presence::Value(value) if value == headword),
                )
                .ok_or_else(|| {
                    DictionaryError::MissingReference(format!("LLG entry {headword}"))
                })?;
            if matches!(kind, ComponentKind::Gismu) {
                for (field_index, native) in row.fields.iter().enumerate() {
                    if !matches!(
                        native.name.as_str(),
                        "assigned_rafsi_cvc" | "assigned_rafsi_ccv" | "assigned_rafsi_cvv"
                    ) {
                        continue;
                    }
                    let spelling = native.raw_value.trim();
                    if spelling.is_empty() {
                        continue;
                    }
                    metadata.forms.push(FormRecord {
                        id: eid(format!(
                            "llg-form-{}",
                            hash_parts(&[
                                "llg-form-v1",
                                &occurrence.capture.sha256,
                                &row.locator,
                                &field_index.to_string()
                            ])
                        ))?,
                        entry: entry.id.clone(),
                        spelling: spelling.into(),
                        language: Presence::Value("jbo".into()),
                        script: Presence::Missing,
                        pronunciation: Presence::Missing,
                        kind: FormKind::AssignedRafsi,
                        standing: Presence::Value("LLG assigned gismu slot".into()),
                        components: Vec::new(),
                        evidence: Some(evidence(occurrence.clone(), field_index)),
                        computation: None,
                    });
                    reports[index].assigned_forms += 1;
                }
            } else if let ParsedKind::Rafsi { form, .. } = &row.kind {
                metadata.forms.push(FormRecord {
                    id: eid(format!(
                        "llg-form-{}",
                        hash_parts(&[
                            "llg-form-v1",
                            &occurrence.capture.sha256,
                            &row.locator,
                            "rafsi-list"
                        ])
                    ))?,
                    entry: entry.id.clone(),
                    spelling: form.clone(),
                    language: Presence::Value("jbo".into()),
                    script: Presence::Missing,
                    pronunciation: Presence::Missing,
                    kind: FormKind::AssignedRafsi,
                    standing: Presence::Value("LLG assigned rafsi list".into()),
                    components: Vec::new(),
                    evidence: Some(evidence(occurrence, 0)),
                    computation: None,
                });
                reports[index].assigned_forms += 1;
            }
        }
    }

    for index in 0..3 {
        let kind = kinds[index];
        let component_id = eid(format!("llg-component-{}", kind.name()))?;
        metadata.releases.push(ReleaseRecord {
            id: release_id(kind, &component_captures[index], &notice_ref, options)?,
            component: component_id,
            distributor: None,
            upstream_revision: Presence::Missing,
            distributor_revision: Presence::Missing,
            captured_at: Presence::Missing,
            time_precision: TimePrecision::Unknown,
            schema_version: 1,
            normalizer: "vlajmaji-llg-lists".into(),
            normalizer_version: "1".into(),
            captures: vec![component_captures[index].clone(), notice_ref.clone()],
            assertions: assertion_ids[index].clone(),
            accounting: ReleaseAccounting {
                input_occurrences: Some(reports[index].input_lines),
                normalized_occurrences: Some(reports[index].input_lines),
                query_excluded_occurrences: Some(reports[index].excluded_assertions),
                unit: "physical-line".into(),
            },
            diagnostics: release_diagnostics[index].clone(),
            selected: true,
            availability: Availability::Available,
            content_notices: vec![ContentNotice {
                text: Presence::Value(notice_text.into()),
                locator: Presence::Value(notice_locator.clone()),
                license: Presence::Value("source-claimed public domain; scope and requested conditions are not independently established".into()),
            }],
            redistribution: RedistributionStatus::Held,
            redistribution_evidence: vec!["The LLG notice is a source claim, not independent legal advice or a broadened redistribution determination.".into()],
        });
    }

    let lexical = metadata
        .assertions
        .iter()
        .filter(|row| matches!(row.eligibility, Eligibility::Lexical))
        .count() as u64;
    let excluded = metadata.assertions.len() as u64 - lexical;
    let teaching_count = teaching_extensions;
    let frequency_count = frequency_extensions;
    let report = LlgImportReport {
        version: 1,
        components: reports,
        captures: captures.len() as u64,
        occurrences: metadata.assertions.len() as u64,
        entries: metadata.entries.len() as u64,
        lexical_assertions: lexical,
        excluded_assertions: excluded,
        eligible_spellings: eligible_heads.len() as u64,
        all_spellings: all_heads.len() as u64,
        native_fields: total_fields,
        assigned_forms: metadata.forms.len() as u64,
        contexts: metadata.contexts.len() as u64,
        teaching_code_extensions: teaching_count,
        frequency_extensions: frequency_count,
        extensions: metadata.extensions.len() as u64,
        diagnostics: diagnostics_total,
    };
    Ok((metadata, captures, report))
}

fn project_contexts(
    metadata: &mut DictionaryMetadata,
    assertion: &EntityId,
    occurrence: &OccurrenceRef,
    row: &ParsedRow,
    kind: ComponentKind,
    count: &mut u64,
) -> Result<(), LlgImportError> {
    for (field_index, native) in row.fields.iter().enumerate() {
        let trimmed = native.raw_value.trim();
        if trimmed.is_empty() {
            continue;
        }
        let (context_kind, value) = match (kind, native.name.as_str()) {
            (ComponentKind::Gismu | ComponentKind::Cmavo, "keyword") => {
                (ContextKind::Gloss, trimmed.to_owned())
            }
            (ComponentKind::Gismu, "hint") => {
                (ContextKind::Note, format!("Native hint: {trimmed}"))
            }
            (ComponentKind::Gismu, "notes") => (ContextKind::Note, trimmed.to_owned()),
            _ => continue,
        };
        push_context(
            metadata,
            assertion,
            occurrence,
            field_index,
            context_kind,
            value,
        )?;
        *count += 1;
    }
    Ok(())
}

fn push_context(
    metadata: &mut DictionaryMetadata,
    assertion: &EntityId,
    occurrence: &OccurrenceRef,
    field_index: usize,
    kind: ContextKind,
    value: String,
) -> Result<(), LlgImportError> {
    metadata.contexts.push(ContextRecord {
        id: eid(format!(
            "llg-context-{}",
            hash_parts(&[
                "llg-context-v1",
                &occurrence.capture.sha256,
                &occurrence.locator,
                &field_index.to_string(),
                &format!("{kind:?}")
            ])
        ))?,
        assertion: assertion.clone(),
        kind,
        value: Presence::Value(text_value(value, "en")),
        translated_from: None,
        evidence: evidence(occurrence.clone(), field_index),
    });
    Ok(())
}

fn text_value(text: String, language: &str) -> TextValue {
    TextValue {
        text,
        language: Presence::Value(language.into()),
        markup: Presence::Missing,
    }
}

fn evidence(occurrence: OccurrenceRef, field_index: usize) -> SourceEvidence {
    SourceEvidence {
        occurrence,
        field_index: Some(field_index),
        citation: Presence::Missing,
        offset: None,
    }
}

fn placeholder_record(id: EntityId, source: &SourceId) -> EntryRecord {
    EntryRecord {
        id,
        source: source.clone(),
        spelling: Presence::Missing,
        language: Presence::Missing,
        script: Presence::Missing,
        native_word_class: Presence::Missing,
        selmaho: Presence::Missing,
        mapped_pos: Presence::Missing,
    }
}

fn eid(value: impl Into<String>) -> Result<EntityId, LlgImportError> {
    EntityId::new(value.into()).map_err(LlgImportError::Dictionary)
}

fn entry_id(spelling: &str) -> Result<EntityId, LlgImportError> {
    eid(format!(
        "llg-entry-{}",
        hash_parts(&["llg-entry-v1", spelling])
    ))
}

fn placeholder_id(capture: &CaptureRef, locator: &str) -> Result<EntityId, LlgImportError> {
    eid(format!(
        "llg-placeholder-{}",
        hash_parts(&[
            "llg-placeholder-v1",
            &capture.namespace,
            &capture.sha256,
            locator
        ])
    ))
}

fn release_id(
    kind: ComponentKind,
    capture: &CaptureRef,
    notice: &CaptureRef,
    options: &LlgImportOptions,
) -> Result<EntityId, LlgImportError> {
    eid(format!(
        "llg-release-{}",
        hash_parts(&[
            "vlajmaji-llg-lists-v1",
            kind.name(),
            &capture.sha256,
            &notice.sha256,
            &options.base_locator
        ])
    ))
}

fn hash_parts(parts: &[&str]) -> String {
    let mut digest = Sha256::new();
    for part in parts {
        digest.update((part.len() as u64).to_be_bytes());
        digest.update(part.as_bytes());
    }
    digest
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn snapshot_identity(namespace: &str, digest: &str) -> Result<SnapshotIdentity, LlgImportError> {
    serde_json::from_value(serde_json::json!({"namespace":namespace,"sha256":digest})).map_err(
        |error| {
            LlgImportError::InvalidOptions(format!("invalid internal capture identity: {error}"))
        },
    )
}

fn occurrence_identity(
    namespace: &str,
    digest: &str,
    locator: &str,
) -> Result<crate::RecordIdentity, LlgImportError> {
    serde_json::from_value(
        serde_json::json!({"snapshot":{"namespace":namespace,"sha256":digest},"locator":locator}),
    )
    .map_err(|error| {
        LlgImportError::InvalidOptions(format!("invalid internal occurrence identity: {error}"))
    })
}
