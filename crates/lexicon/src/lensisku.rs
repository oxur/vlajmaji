//! Private ordered Lensisku row decoding and native projection shared by adapters.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use serde::de::{Deserializer as _, Error as _, MapAccess, Visitor};
use serde_json::value::RawValue;

use crate::CaptureRef;
use crate::dictionary::{
    AssertionRecord, ContextKind, ContextRecord, DictionaryError, Eligibility, EntityId,
    EntryRecord, EtymologyRecord, ExtensionRecord, FormKind, FormRecord, NativeEncoding,
    NativeField, OccurrenceRef, Presence, RawJson, RawOccurrence, ReleaseDiagnostic,
    SourceEvidence, TextValue,
};
use crate::registry::{ContributorId, ContributorRecord, RegistryError, SourceId};

#[derive(Debug, Clone)]
pub(crate) struct Field {
    pub(crate) name: String,
    pub(crate) raw: String,
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
pub(crate) fn parse_fields(raw: &str) -> Result<Vec<Field>, String> {
    let mut de = serde_json::Deserializer::from_str(raw);
    let fields = de
        .deserialize_map(FieldVisitor)
        .map_err(|e| e.to_string())?;
    de.end().map_err(|e| e.to_string())?;
    Ok(fields)
}

#[derive(Debug, Default)]
pub(crate) struct DuplicateScan {
    pub(crate) keys: Vec<String>,
    pub(crate) paths: Vec<Vec<String>>,
}
struct DuplicateVisitor {
    path: Vec<String>,
}
impl<'de> Visitor<'de> for DuplicateVisitor {
    type Value = DuplicateScan;
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a JSON object or array")
    }
    fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Self::Value, M::Error> {
        let mut seen = BTreeSet::new();
        let mut duplicates = DuplicateScan::default();
        while let Some(key) = map.next_key::<String>()? {
            let mut path = self.path.clone();
            path.push(key.clone());
            if !seen.insert(key.clone()) {
                duplicates.keys.push(key);
                duplicates.paths.push(path.clone());
            }
            let value = map.next_value::<Box<RawValue>>()?;
            let nested = scan_nested_duplicates_at(value.get(), path).map_err(M::Error::custom)?;
            duplicates.keys.extend(nested.keys);
            duplicates.paths.extend(nested.paths);
        }
        Ok(duplicates)
    }
    fn visit_seq<S: serde::de::SeqAccess<'de>>(self, mut seq: S) -> Result<Self::Value, S::Error> {
        let mut duplicates = DuplicateScan::default();
        let mut index = 0_usize;
        while let Some(value) = seq.next_element::<Box<RawValue>>()? {
            let mut path = self.path.clone();
            path.push(index.to_string());
            let nested = scan_nested_duplicates_at(value.get(), path).map_err(S::Error::custom)?;
            duplicates.keys.extend(nested.keys);
            duplicates.paths.extend(nested.paths);
            index += 1;
        }
        Ok(duplicates)
    }
}
pub(crate) fn scan_nested_duplicates(raw: &str) -> Result<DuplicateScan, String> {
    scan_nested_duplicates_at(raw, Vec::new())
}
fn scan_nested_duplicates_at(raw: &str, path: Vec<String>) -> Result<DuplicateScan, String> {
    match raw.trim_start().chars().next() {
        Some('{') => {
            let mut de = serde_json::Deserializer::from_str(raw);
            let found = de
                .deserialize_map(DuplicateVisitor { path })
                .map_err(|e| e.to_string())?;
            de.end().map_err(|e| e.to_string())?;
            Ok(found)
        }
        Some('[') => {
            let mut de = serde_json::Deserializer::from_str(raw);
            let found = de
                .deserialize_seq(DuplicateVisitor { path })
                .map_err(|e| e.to_string())?;
            de.end().map_err(|e| e.to_string())?;
            Ok(found)
        }
        _ => Ok(DuplicateScan::default()),
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Row {
    pub(crate) fields: Vec<Field>,
    pub(crate) duplicates: Vec<String>,
    pub(crate) duplicate_paths: Vec<Vec<String>>,
    pub(crate) is_object: bool,
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

pub(crate) fn is_supported_class(value: &str) -> bool {
    SUPPORTED_CLASSES.contains(&value)
}
pub(crate) fn as_json_string(raw: &str) -> Option<String> {
    serde_json::from_str(raw).ok()
}
pub(crate) fn sha256(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut out = String::with_capacity(64);
    for byte in Sha256::digest(bytes) {
        use fmt::Write as _;
        let _ = write!(out, "{byte:02x}");
    }
    out
}
pub(crate) fn valid_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn field_index(fields: &[Field], name: &str) -> Option<usize> {
    fields.iter().position(|f| f.name == name)
}
pub(crate) fn field<'a>(fields: &'a [Field], name: &str) -> Option<&'a Field> {
    field_index(fields, name).and_then(|i| fields.get(i))
}
pub(crate) fn presence_string(fields: &[Field], name: &str) -> Presence<String> {
    match field(fields, name) {
        None => Presence::Missing,
        Some(f) if f.raw == "null" => Presence::Null,
        Some(f) => as_json_string(&f.raw).map_or(Presence::Missing, Presence::Value),
    }
}
pub(crate) fn presence_raw(
    fields: &[Field],
    name: &str,
) -> Result<Presence<RawJson>, DictionaryError> {
    match field(fields, name) {
        None => Ok(Presence::Missing),
        Some(f) if f.raw == "null" => Ok(Presence::Null),
        Some(f) => RawJson::new(f.raw.clone()).map(Presence::Value),
    }
}
pub(crate) fn unique_field_index(fields: &[Field], name: &str) -> Option<usize> {
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
pub(crate) fn add_diag(
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
pub(crate) fn raw_array(raw: &str) -> Option<Vec<Box<RawValue>>> {
    serde_json::from_str(raw).ok()
}
pub(crate) fn user_name(fields: &[Field]) -> Option<String> {
    let user = field(fields, "user")?;
    let nested = parse_fields(&user.raw).ok()?;
    if nested.iter().filter(|f| f.name == "username").count() != 1 {
        return None;
    }
    as_json_string(&field(&nested, "username")?.raw)
}
pub(crate) fn contributor_key(username: &str) -> String {
    format!("lensisku-user-{}", sha256(username.as_bytes()))
}
pub(crate) fn json_array_of_strings(raw: &str) -> Option<Vec<String>> {
    serde_json::from_str::<Vec<String>>(raw).ok()
}
pub(crate) fn forms_from_field(raw: &str) -> Option<Vec<String>> {
    if let Some(s) = as_json_string(raw) {
        Some(s.split_whitespace().map(str::to_owned).collect())
    } else {
        json_array_of_strings(raw)
    }
}
fn entity_id(value: String) -> Result<EntityId, DictionaryError> {
    EntityId::new(value).map_err(|e| DictionaryError::InvalidId(e.to_string()))
}
fn source_evidence(occurrence: OccurrenceRef, field_index: Option<usize>) -> SourceEvidence {
    SourceEvidence {
        occurrence,
        field_index,
        citation: Presence::Missing,
        offset: None,
    }
}
fn text_value(text: String, language: &str) -> TextValue {
    TextValue {
        text,
        language: Presence::Value(language.into()),
        markup: Presence::Missing,
    }
}
pub(crate) fn native_extension(
    owner: &EntityId,
    name: String,
    raw: &str,
    evidence: Option<SourceEvidence>,
) -> Result<ExtensionRecord, DictionaryError> {
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

#[derive(Debug)]
pub(crate) struct DecodedExport {
    pub(crate) rows: Vec<Row>,
    pub(crate) field_counts: BTreeMap<String, u64>,
    pub(crate) word_types: BTreeMap<String, u64>,
    pub(crate) all_spellings: u64,
    pub(crate) eligible_spellings: u64,
    pub(crate) users: BTreeSet<String>,
    pub(crate) native_fields: u64,
    pub(crate) nested_duplicate_rows: u64,
}

pub(crate) fn decode_export(raw_text: &str) -> Result<DecodedExport, String> {
    let raw_rows: Vec<Box<RawValue>> = serde_json::from_str(raw_text).map_err(|e| e.to_string())?;
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
        let (fields, duplicates, duplicate_paths) = if is_object {
            let fields = parse_fields(text)?;
            let duplicates = scan_nested_duplicates(text)?;
            (fields, duplicates.keys, duplicates.paths)
        } else {
            (
                vec![Field {
                    name: "$row".into(),
                    raw: text.to_owned(),
                }],
                Vec::new(),
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
            duplicate_paths,
            is_object,
        });
    }
    Ok(DecodedExport {
        rows,
        field_counts,
        word_types,
        all_spellings: all_spellings.len() as u64,
        eligible_spellings: eligible_spellings.len() as u64,
        users,
        native_fields,
        nested_duplicate_rows,
    })
}

#[derive(Debug)]
pub(crate) struct ContributorSet {
    pub(crate) records: Vec<ContributorRecord>,
    pub(crate) ids: BTreeMap<String, ContributorId>,
    pub(crate) fallback_usernames: Vec<String>,
}
pub(crate) fn contributors(users: &BTreeSet<String>) -> Result<ContributorSet, RegistryError> {
    let mut records = Vec::with_capacity(users.len());
    let mut ids = BTreeMap::new();
    let mut fallback_usernames = Vec::new();
    for username in users {
        let id = ContributorId::new(contributor_key(username))?;
        let display_valid = username.trim() == username
            && !username.is_empty()
            && !username.chars().any(char::is_control);
        let display_name = if display_valid {
            username.clone()
        } else {
            fallback_usernames.push(username.clone());
            format!(
                "Lensisku contributor {}",
                &sha256(username.as_bytes())[..16]
            )
        };
        records.push(ContributorRecord {
            id: id.clone(),
            display_name,
        });
        ids.insert(username.clone(), id);
    }
    Ok(ContributorSet {
        records,
        ids,
        fallback_usernames,
    })
}

pub(crate) struct ProjectionOutput {
    pub(crate) native_occurrences: Vec<RawOccurrence>,
    pub(crate) entries: Vec<EntryRecord>,
    pub(crate) assertions: Vec<AssertionRecord>,
    pub(crate) contexts: Vec<ContextRecord>,
    pub(crate) etymologies: Vec<EtymologyRecord>,
    pub(crate) forms: Vec<FormRecord>,
    pub(crate) extensions: Vec<ExtensionRecord>,
    pub(crate) assertion_ids: Vec<EntityId>,
    pub(crate) eligible_count: u64,
    pub(crate) excluded_count: u64,
}

pub(crate) struct ProjectionContext<'a> {
    pub(crate) entity_prefix: &'a str,
    pub(crate) raw_sha256: &'a str,
    pub(crate) raw_capture_ref: &'a CaptureRef,
    pub(crate) source: &'a SourceId,
    pub(crate) language: &'a str,
    pub(crate) source_language: &'a str,
}

pub(crate) fn project_rows(
    rows: &[Row],
    projection_context: ProjectionContext<'_>,
    report_diagnostics: &mut BTreeMap<String, u64>,
    release_diagnostics: &mut Vec<ReleaseDiagnostic>,
) -> Result<ProjectionOutput, DictionaryError> {
    let ProjectionContext {
        entity_prefix,
        raw_sha256,
        raw_capture_ref,
        source,
        language,
        source_language,
    } = projection_context;
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
                report_diagnostics,
                release_diagnostics,
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
                            report_diagnostics,
                            release_diagnostics,
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
        let base = format!("{entity_prefix}-{raw_sha256}");
        let entry_id = entity_id(format!("{base}-entry-{index}"))?;
        let assertion_id = entity_id(format!("{base}-assertion-{index}"))?;
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
                report_diagnostics,
                release_diagnostics,
                "non_object_row",
                format!("row {index} is not an object; retained under $row"),
                Some(occurrence.clone()),
            );
        }
        for key in &row.duplicates {
            add_diag(
                report_diagnostics,
                release_diagnostics,
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
                report_diagnostics,
                release_diagnostics,
                "invalid_word",
                format!("row {index} has no nonempty string word"),
                Some(occurrence.clone()),
            );
        }
        if class.is_none() {
            reasons.push("missing-null-or-wrong-type-word_type".to_owned());
            add_diag(
                report_diagnostics,
                release_diagnostics,
                "invalid_word_type",
                format!("row {index} has no string word_type"),
                Some(occurrence.clone()),
            );
        } else if !class_valid {
            reasons.push("unsupported-word_type".to_owned());
            add_diag(
                report_diagnostics,
                release_diagnostics,
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
                report_diagnostics,
                release_diagnostics,
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
            language: Presence::Value(source_language.to_owned()),
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
            Some(_) if definition.is_some() => {
                Presence::Value(text_value(definition.clone().unwrap_or_default(), language))
            }
            Some(_) => Presence::Missing,
        };
        let source_score = presence_raw(typed_fields, "score")?;
        let mut assertion = AssertionRecord {
            id: assertion_id.clone(),
            entry: entry_id.clone(),
            occurrence: occurrence.clone(),
            native_sense_id: Presence::Missing,
            language: Presence::Value(language.to_owned()),
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
                    extensions.push(native_extension(
                        &assertion_id,
                        format!("lensisku.native_field.{i}.{}", f.name),
                        &f.raw,
                        Some(source_evidence(occurrence.clone(), Some(i))),
                    )?);
                    add_diag(
                        report_diagnostics,
                        release_diagnostics,
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
                        let id = entity_id(format!("{base}-context-{index}-{name}"))?;
                        contexts.push(context(
                            id,
                            &assertion_id,
                            kind,
                            Presence::Value(text_value(value, language)),
                            &occurrence,
                            i,
                        ));
                        *context_counts.entry(name.into()).or_default() += 1;
                    }
                    None if f.raw == "null" => {
                        let id = entity_id(format!("{base}-context-{index}-{name}"))?;
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
                        report_diagnostics,
                        release_diagnostics,
                        diagnostic,
                        format!("row {index} {name} is not a string or null; raw value retained"),
                        Some(occurrence.clone()),
                    ),
                }
            }
        }

        if let Some(i) = unique_field_index(&row.fields, "gloss_keywords") {
            let f = &row.fields[i];
            extensions.push(native_extension(
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
                            .map_err(DictionaryError::InvalidJson)?;
                        if !item_raw.trim_start().starts_with('{') || !nested_dup.keys.is_empty() {
                            add_diag(
                                report_diagnostics,
                                release_diagnostics,
                                "invalid_gloss_object",
                                format!(
                                    "row {index} gloss item {item_index} is not a unique-key object; whole field retained"
                                ),
                                Some(occurrence.clone()),
                            );
                            continue;
                        }
                        let object =
                            parse_fields(item_raw).map_err(DictionaryError::InvalidJson)?;
                        let Some(word_field) = field(&object, "word") else {
                            add_diag(
                                report_diagnostics,
                                release_diagnostics,
                                "invalid_gloss_object",
                                format!("row {index} gloss item {item_index} lacks word"),
                                Some(occurrence.clone()),
                            );
                            continue;
                        };
                        let Some(gloss) = as_json_string(&word_field.raw) else {
                            add_diag(
                                report_diagnostics,
                                release_diagnostics,
                                "invalid_gloss_object",
                                format!("row {index} gloss item {item_index} word is not a string"),
                                Some(occurrence.clone()),
                            );
                            continue;
                        };
                        let id = entity_id(format!("{base}-context-gloss-{index}-{item_index}"))?;
                        let ctx = context(
                            id.clone(),
                            &assertion_id,
                            ContextKind::Gloss,
                            Presence::Value(text_value(gloss, language)),
                            &occurrence,
                            i,
                        );
                        contexts.push(ctx);
                        extensions.push(native_extension(
                            &id,
                            "lensisku.gloss_keywords.object".into(),
                            item_raw,
                            Some(source_evidence(occurrence.clone(), Some(i))),
                        )?);
                        *context_counts.entry("gloss_keywords".into()).or_default() += 1;
                    }
                } else {
                    add_diag(
                        report_diagnostics,
                        release_diagnostics,
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
            extensions.push(native_extension(
                &assertion_id,
                "lensisku.place_keywords".into(),
                raw_value,
                Some(source_evidence(occurrence.clone(), Some(i))),
            )?);
            if raw_value != "null" && raw_array(raw_value).is_none() {
                add_diag(
                    report_diagnostics,
                    release_diagnostics,
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
                        let id = entity_id(format!("{base}-form-{index}-{name}-{form_index}"))?;
                        forms.push(FormRecord {
                            id,
                            entry: entry_id.clone(),
                            spelling,
                            language: Presence::Value(source_language.to_owned()),
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
                        report_diagnostics,
                        release_diagnostics,
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
                    Some(value) => Presence::Value(text_value(value, language)),
                    None => {
                        add_diag(
                            report_diagnostics,
                            release_diagnostics,
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
                    id: entity_id(format!("{base}-etymology-{index}"))?,
                    assertion: assertion_id.clone(),
                    statement,
                    native_kind: Presence::Missing,
                    components: Vec::new(),
                    evidence: source_evidence(occurrence.clone(), Some(i)),
                });
            }
        }
    }

    Ok(ProjectionOutput {
        native_occurrences,
        entries,
        assertions,
        contexts,
        etymologies,
        forms,
        extensions,
        assertion_ids,
        eligible_count,
        excluded_count,
    })
}
