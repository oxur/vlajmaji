//! Validated source, contributor, distributor, alias, and lineage registry.

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fmt;

use serde::de::Deserializer;
use serde::{Deserialize, Serialize, Serializer};

use crate::{RecordIdentity, SnapshotIdentity};

macro_rules! registry_id {
    ($name:ident, $role:literal) => {
        #[doc = concat!("A stable validated ", $role, " identifier.")]
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(String);

        impl $name {
            /// Validates and constructs this role-specific identifier.
            ///
            /// # Errors
            ///
            /// Returns [`RegistryError::InvalidId`] unless the value follows
            /// the exact lowercase ASCII identifier grammar.
            pub fn new(value: impl Into<String>) -> Result<Self, RegistryError> {
                let value = value.into();
                if valid_id(&value) {
                    Ok(Self(value))
                } else {
                    Err(RegistryError::InvalidId { role: $role, value })
                }
            }

            /// Returns the identifier bytes as their original UTF-8 string.
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl TryFrom<String> for $name {
            type Error = RegistryError;

            fn try_from(value: String) -> Result<Self, Self::Error> {
                Self::new(value)
            }
        }

        impl Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                serializer.serialize_str(&self.0)
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let value = String::deserialize(deserializer)?;
                Self::try_from(value).map_err(serde::de::Error::custom)
            }
        }
    };
}

registry_id!(SourceId, "source");
registry_id!(ContributorId, "contributor");
registry_id!(DistributorId, "distributor");

fn valid_id(value: &str) -> bool {
    let bytes = value.as_bytes();
    !bytes.is_empty()
        && (bytes[0].is_ascii_lowercase() || bytes[0].is_ascii_digit())
        && bytes.iter().all(|byte| {
            byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
                || matches!(*byte, b'.' | b'_' | b':' | b'-')
        })
}

fn validate_text(field: &'static str, value: &str) -> Result<(), RegistryError> {
    if value.is_empty() || value.trim() != value || value.chars().any(char::is_control) {
        return Err(RegistryError::InvalidValue {
            field,
            value: value.to_owned(),
        });
    }
    Ok(())
}

fn validate_evidence(field: &'static str, value: &str) -> Result<(), RegistryError> {
    if value.trim().is_empty() {
        return Err(RegistryError::InvalidValue {
            field,
            value: value.to_owned(),
        });
    }
    Ok(())
}

/// Distinguishes externally maintained sources from explicitly owned drafts.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceOrigin {
    /// A source that makes assertions outside this project.
    Upstream,
    /// A source authored and maintained as an owned draft.
    OwnedDraft,
}

/// States whether an abbreviation was conventionally evidenced or assigned here.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AbbreviationKind {
    /// A conventional abbreviation with recorded evidence.
    Conventional,
    /// An explicitly project-assigned abbreviation.
    ProjectAssigned,
}

/// Lifecycle state for one exact source alias.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AliasStatus {
    /// Selectable under the active-only policy.
    Active,
    /// Retained for history and selectable only when requested.
    Retired,
}

/// Controls whether retired aliases participate in source resolution.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AliasScope {
    /// Match IDs, current abbreviations, and active aliases.
    ActiveOnly,
    /// Also match retired aliases, reporting any resulting ambiguity.
    IncludeRetired,
}

/// An unvalidated source draft row; [`SourceRegistry`] validates it before use.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceRecord {
    /// Stable source identity, distinct from its display name and abbreviation.
    pub id: SourceId,
    /// Human-readable name; never used as a selector.
    pub display_name: String,
    /// Exact-byte source selector, explicitly typed as conventional or assigned.
    pub abbreviation: String,
    /// Provenance class for the abbreviation.
    pub abbreviation_kind: AbbreviationKind,
    /// Evidence for a conventional abbreviation, when applicable.
    pub abbreviation_evidence: Option<String>,
    /// Source standing; owned drafts do not become upstream assertions.
    pub origin: SourceOrigin,
}

/// An unvalidated contributor draft row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContributorRecord {
    /// Stable contributor identity.
    pub id: ContributorId,
    /// Human-readable name; never used as identity.
    pub display_name: String,
}

/// An unvalidated distributor draft row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DistributorRecord {
    /// Stable distributor identity.
    pub id: DistributorId,
    /// Human-readable name; never used as identity.
    pub display_name: String,
}

/// A source alias retained with its lifecycle and evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceAlias {
    /// Source to which this exact selector refers.
    pub source: SourceId,
    /// Exact source spelling; no case, punctuation, or whitespace normalization.
    pub text: String,
    /// Whether ordinary resolution can select this alias.
    pub status: AliasStatus,
    /// Evidence for the alias when supplied.
    pub evidence: Option<String>,
}

/// A lineage target whose unknown state remains explicit.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LineageTarget {
    /// An explicitly registered upstream source.
    Known(SourceId),
    /// An unresolved locator or description; never auto-resolved.
    Unresolved(String),
}

/// An evidence-bearing, descriptive source-lineage edge.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceLineage {
    /// Source whose lineage is described.
    pub source: SourceId,
    /// Known source or explicitly unresolved target.
    pub upstream: LineageTarget,
    /// Evidence for a known edge, when supplied.
    pub evidence: Option<String>,
}

/// Strict wire shape for a snapshot identity inside the new registry format.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SnapshotIdentityWire {
    namespace: String,
    sha256: String,
}

impl From<SnapshotIdentityWire> for SnapshotIdentity {
    fn from(wire: SnapshotIdentityWire) -> Self {
        Self {
            namespace: wire.namespace,
            sha256: wire.sha256,
        }
    }
}

/// Strict wire shape for a record identity inside the new registry format.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RecordIdentityWire {
    snapshot: SnapshotIdentityWire,
    locator: String,
}

impl From<RecordIdentityWire> for RecordIdentity {
    fn from(wire: RecordIdentityWire) -> Self {
        Self {
            snapshot: wire.snapshot.into(),
            locator: wire.locator,
        }
    }
}

/// Binds one immutable capture to its asserting source and optional distributor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SnapshotAttribution {
    /// Exact captured snapshot identity.
    pub snapshot: SnapshotIdentity,
    /// Asserting source identity.
    pub source: SourceId,
    /// Distributor of this capture, if known.
    pub distributor: Option<DistributorId>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SnapshotAttributionWire {
    snapshot: SnapshotIdentityWire,
    source: SourceId,
    distributor: Option<DistributorId>,
}

impl<'de> Deserialize<'de> for SnapshotAttribution {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = SnapshotAttributionWire::deserialize(deserializer)?;
        Ok(Self {
            snapshot: wire.snapshot.into(),
            source: wire.source,
            distributor: wire.distributor,
        })
    }
}

/// Optional native identities for one immutable occurrence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OccurrenceAttribution {
    /// Exact captured record identity.
    pub occurrence: RecordIdentity,
    /// Contributor identity, if known.
    pub contributor: Option<ContributorId>,
    /// Source-native entry identity; an empty supplied string remains present.
    pub native_entry_id: Option<String>,
    /// Source-native assertion identity; it never deduplicates occurrences.
    pub native_assertion_id: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OccurrenceAttributionWire {
    occurrence: RecordIdentityWire,
    contributor: Option<ContributorId>,
    native_entry_id: Option<String>,
    native_assertion_id: Option<String>,
}

impl<'de> Deserialize<'de> for OccurrenceAttribution {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = OccurrenceAttributionWire::deserialize(deserializer)?;
        Ok(Self {
            occurrence: wire.occurrence.into(),
            contributor: wire.contributor,
            native_entry_id: wire.native_entry_id,
            native_assertion_id: wire.native_assertion_id,
        })
    }
}

/// Versioned unvalidated input document for a source registry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegistryDocument {
    /// Registry serialization version. Version 1 is currently supported.
    pub version: u32,
    /// Source entities in deterministic insertion order.
    pub sources: Vec<SourceRecord>,
    /// Contributors in deterministic insertion order.
    pub contributors: Vec<ContributorRecord>,
    /// Distributors in deterministic insertion order.
    pub distributors: Vec<DistributorRecord>,
    /// Active and historical aliases in insertion order.
    pub aliases: Vec<SourceAlias>,
    /// Evidence-bearing lineage rows in insertion order.
    pub lineage: Vec<SourceLineage>,
    /// Snapshot-to-source bindings in insertion order.
    pub snapshots: Vec<SnapshotAttribution>,
    /// Optional occurrence-level native identity rows in insertion order.
    pub occurrences: Vec<OccurrenceAttribution>,
}

impl RegistryDocument {
    /// Resolve an exact source ID, current abbreviation, or eligible alias
    /// without claiming that this draft registry is globally valid.
    pub fn resolve_source(
        &self,
        selector: &str,
        scope: AliasScope,
    ) -> Result<&SourceRecord, RegistryError> {
        let exact_id = SourceId::new(selector.to_owned());
        if let Ok(exact_id) = exact_id
            && let Some(source) = self.unique_source_by_id(&exact_id)?
        {
            return Ok(source);
        }
        let mut matches = BTreeMap::<SourceId, &SourceRecord>::new();
        for source in &self.sources {
            if source.abbreviation == selector {
                let source = self
                    .unique_source_by_id(&source.id)?
                    .ok_or_else(|| RegistryError::UnknownSource(selector.to_owned()))?;
                matches.insert(source.id.clone(), source);
            }
        }
        for alias in &self.aliases {
            if alias.text == selector
                && (alias.status == AliasStatus::Active || scope == AliasScope::IncludeRetired)
                && let Some(source) = self.unique_source_by_id(&alias.source)?
            {
                matches.insert(source.id.clone(), source);
            }
        }
        match matches.len() {
            0 => Err(RegistryError::UnknownSource(selector.to_owned())),
            1 => matches
                .into_values()
                .next()
                .ok_or_else(|| RegistryError::UnknownSource(selector.to_owned())),
            _ => Err(RegistryError::AmbiguousAlias {
                alias: selector.to_owned(),
                sources: matches.into_keys().collect(),
            }),
        }
    }

    fn unique_source_by_id(&self, id: &SourceId) -> Result<Option<&SourceRecord>, RegistryError> {
        let mut sources = self.sources.iter().filter(|source| &source.id == id);
        let Some(source) = sources.next() else {
            return Ok(None);
        };
        if sources.next().is_some() {
            return Err(RegistryError::DuplicateEntity {
                role: "source",
                id: id.to_string(),
            });
        }
        Ok(Some(source))
    }
}

impl Default for RegistryDocument {
    fn default() -> Self {
        Self {
            version: 1,
            sources: Vec::new(),
            contributors: Vec::new(),
            distributors: Vec::new(),
            aliases: Vec::new(),
            lineage: Vec::new(),
            snapshots: Vec::new(),
            occurrences: Vec::new(),
        }
    }
}

/// Matchable validation and attribution failures for registry operations.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistryError {
    /// An ID does not follow the exact lowercase ASCII grammar.
    InvalidId {
        /// Identifier role that rejected the value.
        role: &'static str,
        /// Rejected identifier value.
        value: String,
    },
    /// A named text or evidence value violates its field contract.
    InvalidValue {
        /// Field whose value failed validation.
        field: &'static str,
        /// Rejected field value.
        value: String,
    },
    /// The registry document version is unsupported.
    UnsupportedVersion(u32),
    /// An entity identity is repeated within one role.
    DuplicateEntity {
        /// Entity role containing the duplicate.
        role: &'static str,
        /// Duplicated identifier.
        id: String,
    },
    /// An attribution key is repeated or references an invalid state.
    DuplicateBinding {
        /// Binding role containing the duplicate.
        role: &'static str,
        /// Rendered duplicate binding identity.
        key: String,
    },
    /// A source/text alias pair is repeated.
    DuplicateAlias {
        /// Source carrying the repeated alias.
        source: SourceId,
        /// Exact repeated alias text.
        text: String,
    },
    /// A reference names no registered source.
    UnknownSource(String),
    /// A reference names no registered contributor.
    UnknownContributor(String),
    /// A reference names no registered distributor.
    UnknownDistributor(String),
    /// Two eligible selector spellings conflict across sources.
    AliasCollision {
        /// Exact selector spelling that collides.
        alias: String,
        /// Distinct source identities using the selector.
        sources: Vec<SourceId>,
    },
    /// A selector matches multiple source identities under the requested scope.
    AmbiguousAlias {
        /// Exact selector spelling that resolves to multiple sources.
        alias: String,
        /// Matching source identities.
        sources: Vec<SourceId>,
    },
    /// A lineage edge is malformed or self-referential.
    InvalidLineage {
        /// Source whose lineage row failed.
        source: String,
        /// Explanation of the invalid relationship.
        reason: String,
    },
    /// A snapshot has no explicit source attribution.
    UnboundSnapshot(String),
    /// A registered snapshot is absent from the supplied Lexicon.
    SnapshotNotFound(String),
    /// A registered occurrence is absent from the supplied Lexicon.
    OccurrenceNotFound(String),
    /// An occurrence references a snapshot without a registry binding.
    DanglingOccurrence(String),
    /// An update would erase or rewrite protected registry history.
    ProtectedHistory(String),
    /// JSON was malformed or contained a duplicate key.
    InvalidJson(String),
}

impl fmt::Display for RegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidId { role, value } => write!(f, "invalid {role} ID {value:?}"),
            Self::InvalidValue { field, value } => write!(f, "invalid {field} value {value:?}"),
            Self::UnsupportedVersion(version) => {
                write!(f, "unsupported registry document version {version}")
            }
            Self::DuplicateEntity { role, id } => write!(f, "duplicate {role} ID {id:?}"),
            Self::DuplicateBinding { role, key } => write!(f, "duplicate {role} binding {key}"),
            Self::DuplicateAlias { source, text } => {
                write!(f, "duplicate alias {text:?} for source {source}")
            }
            Self::UnknownSource(id) => write!(f, "unknown source {id:?}"),
            Self::UnknownContributor(id) => write!(f, "unknown contributor {id:?}"),
            Self::UnknownDistributor(id) => write!(f, "unknown distributor {id:?}"),
            Self::AliasCollision { alias, sources } => {
                write!(f, "selector {alias:?} collides across sources {sources:?}")
            }
            Self::AmbiguousAlias { alias, sources } => {
                write!(
                    f,
                    "selector {alias:?} is ambiguous across sources {sources:?}"
                )
            }
            Self::InvalidLineage { source, reason } => {
                write!(f, "invalid lineage for source {source:?}: {reason}")
            }
            Self::UnboundSnapshot(snapshot) => {
                write!(f, "snapshot has no source binding: {snapshot}")
            }
            Self::SnapshotNotFound(snapshot) => {
                write!(f, "registered snapshot not found: {snapshot}")
            }
            Self::OccurrenceNotFound(occurrence) => {
                write!(f, "registered occurrence not found: {occurrence}")
            }
            Self::DanglingOccurrence(occurrence) => {
                write!(f, "occurrence references an unbound snapshot: {occurrence}")
            }
            Self::ProtectedHistory(reason) => {
                write!(f, "registry update would rewrite history: {reason}")
            }
            Self::InvalidJson(message) => write!(f, "invalid registry JSON: {message}"),
        }
    }
}

impl std::error::Error for RegistryError {}

/// Validated registry state; use `from_document` or `from_json_bytes` to construct it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SourceRegistry {
    document: RegistryDocument,
}

impl Serialize for SourceRegistry {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.document.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for SourceRegistry {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let document = RegistryDocument::deserialize(deserializer)?;
        Self::from_document(document).map_err(serde::de::Error::custom)
    }
}

impl SourceRegistry {
    /// Creates an empty version-1 registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Validates and constructs a registry from the complete candidate document.
    ///
    /// # Errors
    ///
    /// Returns a typed validation error for unsupported versions, invalid rows,
    /// conflicting selectors, or dangling foreign references.
    pub fn from_document(document: RegistryDocument) -> Result<Self, RegistryError> {
        validate_document(&document)?;
        Ok(Self { document })
    }

    /// Returns an owned copy of the deterministic registry document.
    #[must_use]
    pub fn document(&self) -> RegistryDocument {
        self.document.clone()
    }

    /// Validates and applies a complete replacement while preserving prior history.
    ///
    /// No state is changed unless the candidate and all history checks succeed.
    ///
    /// # Errors
    ///
    /// Returns a typed validation or [`RegistryError::ProtectedHistory`] error;
    /// the registry remains unchanged on failure.
    pub fn apply_document(&mut self, document: RegistryDocument) -> Result<(), RegistryError> {
        validate_document(&document)?;
        validate_history(&self.document, &document)?;
        self.document = document;
        Ok(())
    }

    /// Serializes this registry as deterministic versioned JSON bytes.
    ///
    /// # Errors
    ///
    /// Returns [`RegistryError::InvalidJson`] if serialization fails.
    pub fn to_json_bytes(&self) -> Result<Vec<u8>, RegistryError> {
        serde_json::to_vec(self).map_err(|error| RegistryError::InvalidJson(error.to_string()))
    }

    /// Parses JSON, rejects duplicate keys, then validates the complete registry.
    ///
    /// # Errors
    ///
    /// Returns [`RegistryError::InvalidJson`] for malformed JSON or duplicate
    /// object keys, and typed registry errors for version or reference failures.
    pub fn from_json_bytes(bytes: &[u8]) -> Result<Self, RegistryError> {
        let text = std::str::from_utf8(bytes)
            .map_err(|error| RegistryError::InvalidJson(error.to_string()))?;
        crate::check_duplicate_keys(text)
            .map_err(|error| RegistryError::InvalidJson(error.to_string()))?;
        let document: RegistryDocument = serde_json::from_str(text)
            .map_err(|error| RegistryError::InvalidJson(error.to_string()))?;
        Self::from_document(document)
    }

    /// Resolves an exact source ID, current abbreviation, or eligible alias.
    ///
    /// # Errors
    ///
    /// Returns [`RegistryError::UnknownSource`] for no match or
    /// [`RegistryError::AmbiguousAlias`] for multiple source identities.
    pub fn resolve_source(
        &self,
        selector: &str,
        scope: AliasScope,
    ) -> Result<&SourceRecord, RegistryError> {
        self.document.resolve_source(selector, scope)
    }

    pub(crate) fn registry_document(&self) -> &RegistryDocument {
        &self.document
    }
}

fn validate_document(document: &RegistryDocument) -> Result<(), RegistryError> {
    if document.version != 1 {
        return Err(RegistryError::UnsupportedVersion(document.version));
    }

    let mut source_ids = BTreeSet::new();
    let mut contributor_ids = BTreeSet::new();
    let mut distributor_ids = BTreeSet::new();
    for source in &document.sources {
        if !source_ids.insert(source.id.clone()) {
            return Err(RegistryError::DuplicateEntity {
                role: "source",
                id: source.id.to_string(),
            });
        }
        validate_text("source display name", &source.display_name)?;
        validate_text("source abbreviation", &source.abbreviation)?;
        match source.abbreviation_evidence.as_deref() {
            Some(evidence) => validate_evidence("abbreviation evidence", evidence)?,
            None if source.abbreviation_kind == AbbreviationKind::Conventional => {
                validate_evidence("conventional abbreviation evidence", "")?
            }
            None => {}
        }
    }
    for contributor in &document.contributors {
        if !contributor_ids.insert(contributor.id.clone()) {
            return Err(RegistryError::DuplicateEntity {
                role: "contributor",
                id: contributor.id.to_string(),
            });
        }
        validate_text("contributor display name", &contributor.display_name)?;
    }
    for distributor in &document.distributors {
        if !distributor_ids.insert(distributor.id.clone()) {
            return Err(RegistryError::DuplicateEntity {
                role: "distributor",
                id: distributor.id.to_string(),
            });
        }
        validate_text("distributor display name", &distributor.display_name)?;
    }

    let mut aliases_seen = BTreeSet::new();
    for alias in &document.aliases {
        if !source_ids.contains(&alias.source) {
            return Err(RegistryError::UnknownSource(alias.source.to_string()));
        }
        validate_text("source alias", &alias.text)?;
        if let Some(evidence) = alias.evidence.as_deref() {
            validate_evidence("alias evidence", evidence)?;
        }
        if !aliases_seen.insert((alias.source.clone(), alias.text.clone())) {
            return Err(RegistryError::DuplicateAlias {
                source: alias.source.clone(),
                text: alias.text.clone(),
            });
        }
    }

    for source in &document.sources {
        if let Some(other) = source_ids
            .iter()
            .find(|id| id.as_str() == source.abbreviation && **id != source.id)
        {
            let mut ids = vec![source.id.clone(), other.clone()];
            ids.sort();
            ids.dedup();
            return Err(RegistryError::AliasCollision {
                alias: source.abbreviation.clone(),
                sources: ids,
            });
        }
    }
    for alias in &document.aliases {
        let abbreviation_owners: BTreeSet<_> = document
            .sources
            .iter()
            .filter(|source| source.id != alias.source && source.abbreviation == alias.text)
            .map(|source| source.id.clone())
            .collect();
        if !abbreviation_owners.is_empty() {
            let mut ids = abbreviation_owners;
            ids.insert(alias.source.clone());
            return Err(RegistryError::AliasCollision {
                alias: alias.text.clone(),
                sources: ids.into_iter().collect(),
            });
        }
        if source_ids
            .iter()
            .any(|id| id.as_str() == alias.text && id != &alias.source)
        {
            let mut ids = vec![alias.source.clone()];
            ids.extend(
                source_ids
                    .iter()
                    .filter(|id| id.as_str() == alias.text)
                    .cloned(),
            );
            ids.sort();
            ids.dedup();
            return Err(RegistryError::AliasCollision {
                alias: alias.text.clone(),
                sources: ids,
            });
        }
    }
    let mut active_selectors = BTreeMap::<String, BTreeSet<SourceId>>::new();
    for source in &document.sources {
        active_selectors
            .entry(source.abbreviation.clone())
            .or_default()
            .insert(source.id.clone());
    }
    for alias in &document.aliases {
        if alias.status == AliasStatus::Active {
            active_selectors
                .entry(alias.text.clone())
                .or_default()
                .insert(alias.source.clone());
        }
    }
    for (alias, ids) in active_selectors {
        if ids.len() > 1 {
            return Err(RegistryError::AliasCollision {
                alias,
                sources: ids.into_iter().collect(),
            });
        }
    }
    for row in &document.lineage {
        if !source_ids.contains(&row.source) {
            return Err(RegistryError::UnknownSource(row.source.to_string()));
        }
        match &row.upstream {
            LineageTarget::Known(target) => {
                if !source_ids.contains(target) {
                    return Err(RegistryError::InvalidLineage {
                        source: row.source.to_string(),
                        reason: format!("unknown target source {target}"),
                    });
                }
                if target == &row.source {
                    return Err(RegistryError::InvalidLineage {
                        source: row.source.to_string(),
                        reason: "self-lineage is invalid".into(),
                    });
                }
                validate_evidence(
                    "known lineage evidence",
                    row.evidence.as_deref().unwrap_or(""),
                )?;
            }
            LineageTarget::Unresolved(locator) => {
                validate_text("unresolved lineage locator", locator)?;
                if let Some(evidence) = row.evidence.as_deref() {
                    validate_evidence("unresolved lineage evidence", evidence)?;
                }
            }
        }
    }

    let mut snapshots = HashSet::new();
    for row in &document.snapshots {
        if !source_ids.contains(&row.source) {
            return Err(RegistryError::UnknownSource(row.source.to_string()));
        }
        if let Some(distributor) = &row.distributor
            && !distributor_ids.contains(distributor)
        {
            return Err(RegistryError::UnknownDistributor(distributor.to_string()));
        }
        if !snapshots.insert(row.snapshot.clone()) {
            return Err(RegistryError::DuplicateBinding {
                role: "snapshot",
                key: identity_text(&row.snapshot),
            });
        }
    }

    let mut occurrences = HashSet::new();
    for row in &document.occurrences {
        if let Some(contributor) = &row.contributor
            && !contributor_ids.contains(contributor)
        {
            return Err(RegistryError::UnknownContributor(contributor.to_string()));
        }
        if !snapshots.contains(row.occurrence.snapshot()) {
            return Err(RegistryError::DanglingOccurrence(identity_text(
                &row.occurrence,
            )));
        }
        if !occurrences.insert(row.occurrence.clone()) {
            return Err(RegistryError::DuplicateBinding {
                role: "occurrence",
                key: identity_text(&row.occurrence),
            });
        }
    }
    Ok(())
}

fn validate_history(old: &RegistryDocument, new: &RegistryDocument) -> Result<(), RegistryError> {
    for source in &old.sources {
        let next = new
            .sources
            .iter()
            .find(|candidate| candidate.id == source.id)
            .ok_or_else(|| {
                RegistryError::ProtectedHistory(format!("source {} was removed", source.id))
            })?;
        if source.origin != next.origin || source.abbreviation_kind != next.abbreviation_kind {
            return Err(RegistryError::ProtectedHistory(format!(
                "source standing or abbreviation class for {} was rewritten",
                source.id
            )));
        }
        if source.abbreviation_evidence != next.abbreviation_evidence {
            return Err(RegistryError::ProtectedHistory(format!(
                "abbreviation evidence for {} was rewritten",
                source.id
            )));
        }
        if source.abbreviation != next.abbreviation
            && !new
                .aliases
                .iter()
                .any(|alias| alias.source == source.id && alias.text == source.abbreviation)
        {
            return Err(RegistryError::ProtectedHistory(format!(
                "old abbreviation {:?} for source {} was not retained as an alias",
                source.abbreviation, source.id
            )));
        }
    }
    for contributor in &old.contributors {
        if !new
            .contributors
            .iter()
            .any(|candidate| candidate.id == contributor.id)
        {
            return Err(RegistryError::ProtectedHistory(format!(
                "contributor {} was removed",
                contributor.id
            )));
        }
    }
    for distributor in &old.distributors {
        if !new
            .distributors
            .iter()
            .any(|candidate| candidate.id == distributor.id)
        {
            return Err(RegistryError::ProtectedHistory(format!(
                "distributor {} was removed",
                distributor.id
            )));
        }
    }
    for alias in &old.aliases {
        let next = new
            .aliases
            .iter()
            .find(|candidate| candidate.source == alias.source && candidate.text == alias.text)
            .ok_or_else(|| {
                RegistryError::ProtectedHistory(format!(
                    "alias {:?} for {} was removed",
                    alias.text, alias.source
                ))
            })?;
        if alias.evidence != next.evidence {
            return Err(RegistryError::ProtectedHistory(format!(
                "evidence for alias {:?} was rewritten",
                alias.text
            )));
        }
        if alias.status == AliasStatus::Retired && next.status != AliasStatus::Retired {
            return Err(RegistryError::ProtectedHistory(format!(
                "retired alias {:?} was reactivated",
                alias.text
            )));
        }
    }
    for lineage in &old.lineage {
        if !new.lineage.contains(lineage) {
            return Err(RegistryError::ProtectedHistory(format!(
                "lineage for {} was removed or rewritten",
                lineage.source
            )));
        }
    }
    for snapshot in &old.snapshots {
        if !new.snapshots.contains(snapshot) {
            return Err(RegistryError::ProtectedHistory(format!(
                "snapshot binding {} changed",
                identity_text(&snapshot.snapshot)
            )));
        }
    }
    for occurrence in &old.occurrences {
        if !new.occurrences.contains(occurrence) {
            return Err(RegistryError::ProtectedHistory(format!(
                "occurrence binding {} changed",
                identity_text(&occurrence.occurrence)
            )));
        }
    }
    Ok(())
}

fn identity_text<T: IdentityText>(identity: &T) -> String {
    identity.identity_text()
}

trait IdentityText {
    fn identity_text(&self) -> String;
}

impl IdentityText for SnapshotIdentity {
    fn identity_text(&self) -> String {
        format!("{}:{}", self.namespace(), self.sha256())
    }
}

impl IdentityText for RecordIdentity {
    fn identity_text(&self) -> String {
        format!("{}:{}", identity_text(self.snapshot()), self.locator())
    }
}
