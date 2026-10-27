//! Compact, borrowed dictionary projections and complete-hit detail lookup.

use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fmt,
    path::Path,
    time::Duration,
};

#[cfg(test)]
use std::cell::Cell;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use vlajmaji_lexicon as lexicon;
use vlajmaji_lexicon::{
    Availability, CaptureMetadata, CaptureRef, EntityId, EntryRecord, OccurrenceRef, Presence,
    RelationTarget, SourceId,
};
use vlajmaji_store::{
    DictionaryReadSnapshot, DictionaryReadTopology, DictionarySnapshotManifest, DictionaryStore,
};

use super::{
    DictionaryExactOptions, DictionaryHit, DictionaryQueryError, DictionaryQueryProvider,
    DictionaryRequest, DictionarySnapshotOpenProfile, DictionarySourceTotal, DictionaryTotals,
    borrowed,
};

/// Stable identity for compact standalone-query responses and cursors.
pub const COMPACT_POLICY: &str = "compact-dictionary-v1";
const DETAIL_POLICY: &str = "entry-assertion-serde-json-v1";

#[cfg(test)]
thread_local! {
    static COMPACT_HIT_PROJECTIONS: Cell<usize> = const { Cell::new(0) };
}

type CompactCatalogResolution = (
    super::ReferenceStatus,
    Option<vlajmaji_store::CollectionEntityReference>,
    Vec<DictionaryCompactReferenceAssertion>,
    Vec<DictionaryCompactReferenceAssertion>,
    Option<vlajmaji_store::CollectionReleaseSummary>,
);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
/// Timings captured by one compact dictionary operation.
pub struct DictionaryProjectionProfile {
    /// Wall time for the complete operation.
    pub total: Duration,
    /// Time spent looking up candidate rows and applying selection policy.
    pub lookup: Duration,
    /// Time spent projecting compact response fields after lookup.
    pub projection: Duration,
    /// Time spent retrieving candidate assertion IDs from SQLite.
    pub candidate_sql: Duration,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Stable key for requesting a complete hit from a compact result.
pub struct DictionaryHitDetailRef {
    /// Version of this key contract.
    pub version: u32,
    /// Encoding policy used for entry and assertion digests.
    pub digest_policy: String,
    /// Entity role addressed by the key; currently `assertion`.
    pub role: String,
    /// Source owning the lexical entry.
    pub source: SourceId,
    /// Stable entry ID whose complete record is checked.
    pub entry_id: EntityId,
    /// Stable assertion ID whose complete record is checked.
    pub assertion_id: EntityId,
    /// Exact source occurrence for the assertion.
    pub occurrence: OccurrenceRef,
    /// Lower-case SHA-256 of the complete serialized entry record.
    pub entry_sha256: String,
    /// Lower-case SHA-256 of the complete serialized assertion record.
    pub assertion_sha256: String,
    #[serde(deserialize_with = "required_nullable")]
    /// Source-native entry identifier, or explicit JSON null.
    pub native_entry_id: Option<String>,
    #[serde(deserialize_with = "required_nullable")]
    /// Source-native assertion identifier, or explicit JSON null.
    pub native_assertion_id: Option<String>,
    #[serde(deserialize_with = "required_nullable")]
    /// Contributor attribution, or explicit JSON null.
    pub contributor_id: Option<lexicon::ContributorId>,
    #[serde(deserialize_with = "required_nullable")]
    /// Distributor attribution, or explicit JSON null.
    pub distributor_id: Option<lexicon::DistributorId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// Reason a syntactically valid detail key fails semantic validation.
pub enum DictionaryDetailReferenceInvalidity {
    /// The key uses an unsupported schema version.
    UnsupportedVersion,
    /// The key names an unsupported record digest policy.
    UnsupportedDigestPolicy,
    /// The key does not address an assertion.
    WrongRole,
    /// An entry or assertion digest is not lower-case SHA-256.
    InvalidDigest,
    /// The occurrence locator is empty.
    EmptyLocator,
    /// The referenced occurrence or records cannot be matched.
    InvalidCapture,
}

#[derive(Debug)]
#[non_exhaustive]
/// Typed failures while parsing a detail key or requesting a complete hit.
pub enum DictionaryProjectionError {
    /// Failure from the existing dictionary query path.
    Query(Box<DictionaryQueryError>),
    /// The supplied key bytes are not a valid strict JSON key.
    ReferenceJson {
        /// Underlying JSON parser failure.
        source: serde_json::Error,
    },
    /// A decoded key violates its semantic contract.
    InvalidDetailReference {
        /// Specific semantic failure.
        reason: DictionaryDetailReferenceInvalidity,
    },
    /// The expected snapshot token is not lower-case 64-digit hexadecimal.
    MalformedSnapshot {
        /// Supplied token.
        value: String,
    },
    /// The valid expected snapshot does not match this engine's snapshot.
    StaleSnapshot {
        /// Snapshot requested by the caller.
        expected: String,
        /// Snapshot bound to this engine.
        observed: String,
    },
}
impl fmt::Display for DictionaryProjectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Query(error) => write!(f, "dictionary query: {error}"),
            Self::ReferenceJson { source } => write!(f, "detail reference JSON: {source}"),
            Self::InvalidDetailReference { reason } => {
                write!(f, "invalid detail reference: {reason:?}")
            }
            Self::MalformedSnapshot { value } => write!(f, "malformed snapshot digest: {value}"),
            Self::StaleSnapshot { expected, observed } => write!(
                f,
                "stale snapshot: expected {expected}, observed {observed}"
            ),
        }
    }
}
impl Error for DictionaryProjectionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Query(error) => Some(error),
            Self::ReferenceJson { source } => Some(source),
            _ => None,
        }
    }
}
impl From<DictionaryQueryError> for DictionaryProjectionError {
    fn from(value: DictionaryQueryError) -> Self {
        Self::Query(Box::new(value))
    }
}
impl DictionaryProjectionError {
    /// Return the stable public error report.
    pub fn report(&self) -> super::DictionaryErrorReport {
        let code = match self {
            Self::Query(error) => return error.report(),
            Self::ReferenceJson { .. } | Self::InvalidDetailReference { .. } => {
                "invalid_detail_reference"
            }
            Self::MalformedSnapshot { .. } => "malformed_snapshot",
            Self::StaleSnapshot { .. } => "stale_snapshot",
        };
        super::DictionaryErrorReport {
            code: code.into(),
            message: self.to_string(),
            sources: Vec::new(),
        }
    }
}

impl DictionaryHitDetailRef {
    /// Parse a strict detail reference and validate its semantic fields.
    pub fn from_json_bytes(bytes: &[u8]) -> Result<Self, DictionaryProjectionError> {
        let reference: Self = serde_json::from_slice(bytes)
            .map_err(|source| DictionaryProjectionError::ReferenceJson { source })?;
        reference.validate()?;
        Ok(reference)
    }

    /// Validate the version, policy, role, digest, and locator fields.
    pub fn validate(&self) -> Result<(), DictionaryProjectionError> {
        validate_detail_reference(self)
    }
}

/// Validate a lower-case SHA-256 detail snapshot binding.
pub fn validate_detail_snapshot_sha256(value: &str) -> Result<(), DictionaryProjectionError> {
    if valid_sha256(value) {
        Ok(())
    } else {
        Err(DictionaryProjectionError::MalformedSnapshot {
            value: value.to_owned(),
        })
    }
}

fn required_nullable<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// A compact lexical occurrence with a stable key for later detail retrieval.
pub struct DictionaryCompactHit {
    /// Hex encoding of the occurrence tuple, not a content hash.
    pub occurrence_id: String,
    /// The matching source entry.
    pub entry: EntryRecord,
    /// The matching lexical assertion.
    pub assertion: lexicon::AssertionRecord,
    /// Capture metadata associated with the occurrence.
    pub capture: CaptureMetadata,
    /// Registry attribution for the source snapshot.
    pub snapshot_attribution: lexicon::SnapshotAttribution,
    /// Registry attribution for this occurrence, when present.
    pub occurrence_attribution: Option<lexicon::OccurrenceAttribution>,
    /// Contributor record associated with the occurrence, when present.
    pub contributor: Option<lexicon::ContributorRecord>,
    /// Distributor record associated with the capture, when present.
    pub distributor: Option<lexicon::DistributorRecord>,
    /// Selected available releases that contain the assertion.
    pub release_ids: Vec<EntityId>,
    /// Validated key for requesting the complete hit.
    pub detail_ref: DictionaryHitDetailRef,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// One source group in a compact exact response.
pub struct DictionaryCompactGroup {
    /// Full source metadata used to identify the result group.
    pub source: lexicon::SourceRecord,
    /// Number of matching occurrences before paging.
    pub full_count: usize,
    /// Number of occurrences returned in this page.
    pub returned_count: usize,
    /// Whether the page contains only part of this source's matches.
    pub partial: bool,
    /// Compact hits returned for this source.
    pub hits: Vec<DictionaryCompactHit>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Compact exact-query response with the established query totals and ordering.
pub struct DictionaryCompactExactResponse {
    /// Response schema version.
    pub version: u32,
    /// Compact projection policy identifier.
    pub projection_policy: String,
    /// Existing exact-query policy identifier.
    pub policy: String,
    /// Dictionary store schema version observed by the provider.
    pub store_schema: u32,
    /// Catalogue-local revision associated with this typed document.
    pub catalogue_revision: u64,
    /// Recorded store digest; ordinary open does not authenticate it.
    pub state_sha256: String,
    /// Binding of the exact compact selection and ordering snapshot.
    pub snapshot_sha256: String,
    /// Exact UTF-8 spelling requested.
    pub spelling: String,
    /// Sources resolved from the request's selectors.
    pub resolved_sources: Vec<SourceId>,
    /// Number of native SQL candidate IDs before compact policy filtering.
    pub native_candidate_occurrences: usize,
    /// Counts before cursor or page slicing.
    pub totals: DictionaryTotals,
    /// Per-source counts before cursor or page slicing.
    pub source_totals: Vec<DictionarySourceTotal>,
    /// Compact operational availability for every registered source.
    pub availability: Vec<DictionaryCompactSourceAvailability>,
    /// Source groups and hits for this page.
    pub groups: Vec<DictionaryCompactGroup>,
    /// Opaque continuation token when additional rows remain.
    pub next_cursor: Option<String>,
    /// Versioned notices retained in the dictionary metadata.
    pub code_notices: Vec<lexicon::CodeNotice>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Compact source state without release assertion inventories or payload bodies.
pub struct DictionaryCompactSourceAvailability {
    /// Canonical source record.
    pub source: lexicon::SourceRecord,
    /// Aliases that identify the source.
    pub aliases: Vec<lexicon::SourceAlias>,
    /// Lineage records attached to the source.
    pub lineage: Vec<lexicon::SourceLineage>,
    /// Collections belonging to the source.
    pub collections: Vec<lexicon::CollectionRecord>,
    /// Components belonging to those collections.
    pub components: Vec<lexicon::ComponentRecord>,
    /// Summary metadata for the source's releases.
    pub releases: Vec<vlajmaji_store::CollectionReleaseSummary>,
    /// Selected available releases, in source metadata order.
    pub selected_available_release_ids: Vec<EntityId>,
    /// Captures whose payload is loaded.
    pub loaded_captures: Vec<CaptureRef>,
    /// Captures named by retained releases but absent from the document.
    pub missing_captures: Vec<CaptureRef>,
    /// Current owned-source head when this source is owned.
    pub owned_head: Option<lexicon::OwnedHead>,
    /// Whether the source can participate in exact selection.
    pub selectable: bool,
    /// Reason the source is not selectable, when present.
    pub reason: Option<super::SourceSelectionFailure>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// One reference assertion with compact attribution and optional detail key.
pub struct DictionaryCompactReferenceAssertion {
    /// Entry containing the assertion.
    pub entry: EntryRecord,
    /// Typed assertion record.
    pub assertion: lexicon::AssertionRecord,
    /// Hex encoding of the exact occurrence tuple.
    pub occurrence_id: String,
    /// Native registry attribution for this occurrence, if recorded.
    pub occurrence_attribution: Option<lexicon::OccurrenceAttribution>,
    /// Capture metadata when the capture is loaded.
    pub capture_metadata: Option<CaptureMetadata>,
    /// Snapshot attribution when available.
    pub snapshot_attribution: Option<lexicon::SnapshotAttribution>,
    /// Whether the referenced capture and locator are present.
    pub payload_present: bool,
    /// Whether current exact-query policy selects this assertion.
    pub current_eligible: bool,
    /// Releases that contain this assertion, in metadata order.
    pub release_ids: Vec<EntityId>,
    /// Owned revisions that include this assertion.
    pub owned_revision_ids: Vec<EntityId>,
    /// Owned revisions that tombstone this assertion.
    pub tombstoned_by: Vec<EntityId>,
    /// Complete-hit detail key when its required capture is available.
    pub detail_ref: Option<DictionaryHitDetailRef>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "operation", content = "response", rename_all = "snake_case")]
/// Standalone compact operation result with an operation-tagged response.
pub enum DictionaryCompactQueryResult {
    /// Exact spelling result.
    Exact(Box<DictionaryCompactExactResponse>),
    /// Typed reference-resolution result.
    Resolve(Box<DictionaryCompactReferenceResolution>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Compact resolution of a local or catalogue reference.
pub struct DictionaryCompactReferenceResolution {
    /// Response contract version.
    pub version: u32,
    /// Projection policy identifier.
    pub projection_policy: String,
    /// Recorded dictionary state digest.
    pub state_sha256: String,
    /// Binding of the exact compact selection snapshot.
    pub snapshot_sha256: String,
    /// Exact relation target supplied by the caller.
    pub target: RelationTarget,
    /// Classification of the target against this document.
    pub status: super::ReferenceStatus,
    /// Role-qualified entity projection for a local target, when applicable.
    pub entity: Option<vlajmaji_store::CollectionEntityReference>,
    /// Assertions confirmed by retained source metadata.
    pub confirmed: Vec<DictionaryCompactReferenceAssertion>,
    /// Assertions whose identity could not be fully confirmed.
    pub unverified: Vec<DictionaryCompactReferenceAssertion>,
    /// Compact availability for registered sources.
    pub availability: Vec<DictionaryCompactSourceAvailability>,
    /// Requested release summary for catalogue targets, when applicable.
    pub requested_release: Option<vlajmaji_store::CollectionReleaseSummary>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Complete-hit detail response for a previously issued detail key.
pub struct DictionaryHitDetailResponse {
    /// Response schema version.
    pub version: u32,
    /// Detail projection policy identifier.
    pub projection_policy: String,
    /// Key used to request this complete hit.
    pub detail_ref: DictionaryHitDetailRef,
    /// Resolution status of the referenced assertion.
    pub status: super::ReferenceStatus,
    /// Operational explanation when the hit is not available.
    pub operational_reason: Option<String>,
    /// Recorded state digest when an engine is available.
    pub state_sha256: Option<String>,
    /// Snapshot binding used to validate the request.
    pub snapshot_sha256: String,
    /// Complete hit when the key resolves.
    pub hit: Option<DictionaryHit>,
    /// Whether the assertion is eligible in the current selection.
    pub current_eligible: Option<bool>,
}

/// A compact engine retains one typed unverified document and provider, never a full engine.
pub struct DictionaryCompactSearchEngine {
    provider: DictionaryQueryProvider,
    document: lexicon::UnverifiedDictionaryDocument,
    indices: borrowed::CompactIndices,
    state_sha256: String,
    overlay: Option<Vec<EntityId>>,
    selected: BTreeMap<SourceId, BTreeSet<String>>,
    availability: Vec<DictionaryCompactSourceAvailability>,
    source_map: BTreeMap<SourceId, lexicon::SourceRecord>,
    snapshot_sha256: String,
}

impl DictionaryCompactSearchEngine {
    /// Open a standalone dictionary store for compact queries.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, DictionaryQueryError> {
        Self::from_store(DictionaryStore::open(path)?)
    }
    /// Open a pinned multi-store snapshot without constructing the full engine.
    pub fn open_snapshot(
        topology: DictionaryReadTopology,
        manifest: DictionarySnapshotManifest,
    ) -> Result<Self, DictionaryQueryError> {
        let mut snapshot = DictionaryReadSnapshot::open(topology, manifest)?;
        let document = snapshot.load_document()?;
        Self::from_provider(DictionaryQueryProvider::Snapshot(snapshot), document)
    }
    /// Open a pinned snapshot and report provider, hydration, and setup timings.
    pub fn open_snapshot_profiled(
        topology: DictionaryReadTopology,
        manifest: DictionarySnapshotManifest,
    ) -> Result<(Self, DictionarySnapshotOpenProfile), DictionaryQueryError> {
        let started = std::time::Instant::now();
        let provider_started = started;
        let mut snapshot = DictionaryReadSnapshot::open(topology, manifest)?;
        let provider_open = provider_started.elapsed();
        let load_started = std::time::Instant::now();
        let document = snapshot.load_document()?;
        let document_load_and_composition = load_started.elapsed();
        let index_started = std::time::Instant::now();
        let engine = Self::from_provider(DictionaryQueryProvider::Snapshot(snapshot), document)?;
        let common_index_setup = index_started.elapsed();
        Ok((
            engine,
            DictionarySnapshotOpenProfile {
                provider_open,
                document_load_and_composition,
                common_index_setup,
                total: started.elapsed(),
            },
        ))
    }
    fn from_store(store: DictionaryStore) -> Result<Self, DictionaryQueryError> {
        let document = store.unverified_document()?;
        Self::from_provider(DictionaryQueryProvider::Store(store), document)
    }
    fn from_provider(
        provider: DictionaryQueryProvider,
        document: lexicon::UnverifiedDictionaryDocument,
    ) -> Result<Self, DictionaryQueryError> {
        let state_sha256 = provider.state_sha256().to_owned();
        let generation = provider.generation().map(str::to_owned);
        let overlay = provider.selected_release_ids();
        if let Some(ids) = &overlay {
            let known: BTreeSet<_> = document
                .metadata
                .releases
                .iter()
                .map(|row| row.id.clone())
                .collect();
            if let Some(unknown) = ids.iter().find(|id| !known.contains(*id)) {
                return Err(DictionaryQueryError::Integrity {
                    detail: format!("snapshot selected unknown release {unknown}"),
                });
            }
        }
        let (indices, (availability, selected)) =
            borrowed::index_document(&document, overlay.as_deref())?;
        let registry = &document.registry;
        let sources = registry
            .sources
            .iter()
            .map(|source| (source.id.clone(), source.clone()))
            .collect::<BTreeMap<_, _>>();
        let snapshot_sha256 = compact_snapshot_sha256(
            &state_sha256,
            generation.as_deref(),
            overlay.as_deref(),
            &availability,
        )?;
        Ok(Self {
            provider,
            document,
            indices,
            state_sha256,
            overlay,
            selected,
            availability,
            source_map: sources,
            snapshot_sha256,
        })
    }
    /// Return the provider's recorded state digest.
    pub fn state_sha256(&self) -> &str {
        &self.state_sha256
    }
    /// Return this engine's compact snapshot binding.
    pub fn snapshot_sha256(&self) -> &str {
        &self.snapshot_sha256
    }
    /// Execute an exact or reference query and return compact output.
    pub fn execute(
        &self,
        request: DictionaryRequest,
    ) -> Result<DictionaryCompactQueryResult, DictionaryQueryError> {
        match request {
            DictionaryRequest::Exact { spelling, options } => self
                .exact(&spelling, options)
                .map(|response| DictionaryCompactQueryResult::Exact(Box::new(response))),
            DictionaryRequest::Resolve { target } => self
                .resolve(&target)
                .map(|response| DictionaryCompactQueryResult::Resolve(Box::new(response))),
        }
    }
    /// Execute a compact query and record same-call timing.
    pub fn execute_profiled(
        &self,
        request: DictionaryRequest,
    ) -> Result<(DictionaryCompactQueryResult, DictionaryProjectionProfile), DictionaryQueryError>
    {
        let started = std::time::Instant::now();
        let (result, mut profile) = match request {
            DictionaryRequest::Exact { spelling, options } => {
                let (response, profile) = self.exact_with_profile(&spelling, options)?;
                (
                    DictionaryCompactQueryResult::Exact(Box::new(response)),
                    profile,
                )
            }
            DictionaryRequest::Resolve { target } => {
                let (response, profile) = self.resolve_profiled(&target)?;
                (
                    DictionaryCompactQueryResult::Resolve(Box::new(response)),
                    profile,
                )
            }
        };
        profile.total = started.elapsed();
        profile.lookup = profile.total.saturating_sub(profile.projection);
        Ok((result, profile))
    }
    /// Query byte-exact spelling matches with compact hit bodies.
    pub fn exact(
        &self,
        spelling: &str,
        options: DictionaryExactOptions,
    ) -> Result<DictionaryCompactExactResponse, DictionaryQueryError> {
        self.exact_with_profile(spelling, options)
            .map(|(response, _)| response)
    }

    fn exact_with_profile(
        &self,
        spelling: &str,
        options: DictionaryExactOptions,
    ) -> Result<(DictionaryCompactExactResponse, DictionaryProjectionProfile), DictionaryQueryError>
    {
        let total_started = std::time::Instant::now();
        let lookup_started = std::time::Instant::now();
        let resolved_sources: Vec<SourceId> = if options.sources.is_empty() {
            self.availability
                .iter()
                .filter(|row| row.selectable)
                .map(|row| row.source.id.clone())
                .collect()
        } else {
            let mut result = BTreeSet::new();
            for selector in &options.sources {
                let id = self
                    .document
                    .registry
                    .resolve_source(selector, options.alias_scope)?
                    .id
                    .clone();
                let state = self
                    .availability
                    .iter()
                    .find(|row| row.source.id == id)
                    .ok_or_else(|| DictionaryQueryError::Integrity {
                        detail: "resolved source is absent from compact availability".into(),
                    })?;
                if let Some(reason) = state.reason {
                    return Err(DictionaryQueryError::UnavailableSource { source: id, reason });
                }
                result.insert(id);
            }
            result.into_iter().collect()
        };
        let cursor_binding = compact_query_binding(
            &self.snapshot_sha256,
            spelling,
            &resolved_sources,
            options.alias_scope,
        )?;
        let sql = std::time::Instant::now();
        let candidate_ids = self.provider.exact_assertion_ids(spelling)?;
        let candidate_sql = sql.elapsed();
        let mut candidate_keys: BTreeSet<String> =
            candidate_ids.iter().map(ToString::to_string).collect();
        let mut rows = Vec::<(usize, usize)>::new();
        for id in std::mem::take(&mut candidate_keys) {
            let assertion_index = *self.indices.assertions.get(&id).ok_or_else(|| {
                DictionaryQueryError::Integrity {
                    detail: format!(
                        "SQL candidate {id} is absent from the loaded dictionary draft"
                    ),
                }
            })?;
            let assertion = &self.document.metadata.assertions[assertion_index];
            let selected_for_source = resolved_sources.iter().any(|source| {
                self.selected
                    .get(source)
                    .is_some_and(|ids| ids.contains(id.as_str()))
            });
            if !selected_for_source {
                continue;
            }
            if !matches!(assertion.eligibility, lexicon::Eligibility::Lexical) {
                continue;
            }
            let entry_id = assertion.entry.to_string();
            let entry_index = *self.indices.entries.get(&entry_id).ok_or_else(|| {
                DictionaryQueryError::Integrity {
                    detail: format!("assertion {} references missing entry", assertion.id),
                }
            })?;
            rows.push((entry_index, assertion_index));
        }
        rows.sort_by(
            |(left_entry, left_assertion), (right_entry, right_assertion)| {
                let left_entry = &self.document.metadata.entries[*left_entry];
                let right_entry = &self.document.metadata.entries[*right_entry];
                let left_assertion = &self.document.metadata.assertions[*left_assertion];
                let right_assertion = &self.document.metadata.assertions[*right_assertion];
                (
                    &left_entry.source,
                    presence_value(&left_entry.spelling)
                        .map(String::as_str)
                        .unwrap_or(""),
                    super::tuple_hex(&left_assertion.occurrence),
                    left_assertion.id.to_string(),
                )
                    .cmp(&(
                        &right_entry.source,
                        presence_value(&right_entry.spelling)
                            .map(String::as_str)
                            .unwrap_or(""),
                        super::tuple_hex(&right_assertion.occurrence),
                        right_assertion.id.to_string(),
                    ))
            },
        );
        let lookup = lookup_started.elapsed();
        let projection_started = std::time::Instant::now();
        let total = rows.len();
        let mut source_totals = Vec::new();
        let mut groups = Vec::new();
        let mut offset = 0usize;
        if let Some(cursor) = options.cursor.as_deref() {
            offset = cursor_position(cursor, &cursor_binding, spelling, &rows, &self.document)?;
        }
        let page_end = options
            .page_size
            .map_or(total, |size| offset.saturating_add(size).min(total));
        let selected_rows = &rows[offset.min(total)..page_end];
        let mut by_source: BTreeMap<SourceId, Vec<DictionaryCompactHit>> = BTreeMap::new();
        for (entry_index, assertion_index) in selected_rows {
            let entry = &self.document.metadata.entries[*entry_index];
            let assertion = &self.document.metadata.assertions[*assertion_index];
            let source = &entry.source;
            by_source
                .entry(source.clone())
                .or_default()
                .push(self.compact_hit(entry, assertion)?);
        }
        for source in &resolved_sources {
            let count = rows
                .iter()
                .filter(|(entry_index, _)| {
                    self.document.metadata.entries[*entry_index].source == *source
                })
                .count();
            if count == 0 {
                continue;
            }
            let record = self
                .source_map
                .get(source)
                .ok_or_else(|| DictionaryQueryError::Integrity {
                    detail: format!("source {source} missing from registry"),
                })?
                .clone();
            source_totals.push(DictionarySourceTotal {
                source: record.clone(),
                occurrences: count,
                distinct_spellings: 1,
            });
            let hits = by_source.remove(source).unwrap_or_default();
            if !hits.is_empty() {
                groups.push(DictionaryCompactGroup {
                    source: record,
                    full_count: count,
                    returned_count: hits.len(),
                    partial: hits.len() < count,
                    hits,
                });
            }
        }
        let next_cursor = if page_end < total {
            selected_rows.last().map(|(entry_index, assertion_index)| {
                let entry = &self.document.metadata.entries[*entry_index];
                let assertion = &self.document.metadata.assertions[*assertion_index];
                make_cursor(
                    &cursor_binding,
                    spelling,
                    &entry.source,
                    &assertion.occurrence,
                    &assertion.id,
                )
            })
        } else {
            None
        };
        let totals = DictionaryTotals {
            occurrences: total,
            distinct_spellings: usize::from(total > 0),
            source_groups: source_totals.len(),
        };
        let response = DictionaryCompactExactResponse {
            version: 1,
            projection_policy: COMPACT_POLICY.into(),
            policy: super::EXACT_POLICY.into(),
            store_schema: 1,
            catalogue_revision: self.document.metadata.catalogue_revision,
            state_sha256: self.state_sha256.clone(),
            snapshot_sha256: self.snapshot_sha256.clone(),
            spelling: spelling.into(),
            resolved_sources,
            native_candidate_occurrences: candidate_ids.len(),
            totals,
            source_totals,
            availability: self.availability.clone(),
            groups,
            next_cursor,
            code_notices: self.document.metadata.code_notices.clone(),
        };
        let projection = projection_started.elapsed();
        let total_time = total_started.elapsed();
        Ok((
            response,
            DictionaryProjectionProfile {
                total: total_time,
                lookup,
                projection,
                candidate_sql,
            },
        ))
    }
    /// Resolve a local or catalogue target without loading full hit bodies.
    pub fn resolve(
        &self,
        target: &RelationTarget,
    ) -> Result<DictionaryCompactReferenceResolution, DictionaryQueryError> {
        self.resolve_inner(target, None)
    }

    /// Resolve a reference and report lookup and response-projection timings.
    fn resolve_profiled(
        &self,
        target: &RelationTarget,
    ) -> Result<
        (
            DictionaryCompactReferenceResolution,
            DictionaryProjectionProfile,
        ),
        DictionaryQueryError,
    > {
        let started = std::time::Instant::now();
        let mut projection = Duration::ZERO;
        let response = self.resolve_inner(target, Some(&mut projection))?;
        let total = started.elapsed();
        let lookup = total.saturating_sub(projection);
        Ok((
            response,
            DictionaryProjectionProfile {
                total,
                lookup,
                projection,
                candidate_sql: Duration::ZERO,
            },
        ))
    }

    fn resolve_inner(
        &self,
        target: &RelationTarget,
        mut projection: Option<&mut Duration>,
    ) -> Result<DictionaryCompactReferenceResolution, DictionaryQueryError> {
        let projection_started = projection.as_ref().map(|_| std::time::Instant::now());
        let availability = self.availability.clone();
        if let (Some(projection), Some(started)) = (projection.as_deref_mut(), projection_started) {
            *projection += started.elapsed();
        }
        let (status, entity, assertions, unverified, requested_release) = match target {
            RelationTarget::Unknown => (
                super::ReferenceStatus::Unknown,
                None,
                Vec::new(),
                Vec::new(),
                None,
            ),
            RelationTarget::Unresolved(value) if value.is_empty() => {
                return Err(super::invalid_ref("Unresolved target must be nonempty"));
            }
            RelationTarget::Unresolved(_) => (
                super::ReferenceStatus::Unresolved,
                None,
                Vec::new(),
                Vec::new(),
                None,
            ),
            RelationTarget::Local(local) => {
                let (entity, rows) = self.resolve_local(local, projection.as_deref_mut())?;
                (
                    super::ReferenceStatus::Resolved,
                    Some(entity),
                    rows,
                    Vec::new(),
                    None,
                )
            }
            RelationTarget::Catalog(catalog) => {
                self.resolve_catalog(catalog, projection.as_deref_mut())?
            }
        };
        let projection_started = projection.as_ref().map(|_| std::time::Instant::now());
        let response = DictionaryCompactReferenceResolution {
            version: 1,
            projection_policy: COMPACT_POLICY.into(),
            state_sha256: self.state_sha256.clone(),
            snapshot_sha256: self.snapshot_sha256.clone(),
            target: target.clone(),
            status,
            entity,
            confirmed: assertions,
            unverified,
            availability,
            requested_release,
        };
        if let (Some(projection), Some(started)) = (projection, projection_started) {
            *projection += started.elapsed();
        }
        Ok(response)
    }

    fn resolve_local(
        &self,
        local: &lexicon::LocalRef,
        mut projection: Option<&mut Duration>,
    ) -> Result<
        (
            vlajmaji_store::CollectionEntityReference,
            Vec<DictionaryCompactReferenceAssertion>,
        ),
        DictionaryQueryError,
    > {
        use lexicon::LocalRef;
        let assertion_selected = |assertion: &lexicon::AssertionRecord| {
            let source = self
                .document
                .metadata
                .entries
                .iter()
                .find(|entry| entry.id == assertion.entry)
                .map(|entry| &entry.source);
            source
                .and_then(|source| self.selected.get(source))
                .is_some_and(|ids| ids.contains(&assertion.id.to_string()))
        };
        let (role, id, entry_ids, assertion_ids, spelling) = match local {
            LocalRef::Entry(id) => {
                let entry = self
                    .document
                    .metadata
                    .entries
                    .iter()
                    .find(|row| row.id == *id)
                    .ok_or_else(|| {
                        super::invalid_ref("LocalRef does not exist in its specific role table")
                    })?;
                let assertions = self
                    .document
                    .metadata
                    .assertions
                    .iter()
                    .filter(|row| row.entry == *id && assertion_selected(row))
                    .map(|row| row.id.to_string())
                    .collect();
                (
                    "entry",
                    id,
                    vec![id.to_string()],
                    assertions,
                    Some(entry.spelling.clone()),
                )
            }
            LocalRef::Assertion(id) => {
                let assertion = self
                    .document
                    .metadata
                    .assertions
                    .iter()
                    .find(|row| row.id == *id)
                    .ok_or_else(|| {
                        super::invalid_ref("LocalRef does not exist in its specific role table")
                    })?;
                (
                    "assertion",
                    id,
                    vec![assertion.entry.to_string()],
                    if assertion_selected(assertion) {
                        vec![id.to_string()]
                    } else {
                        Vec::new()
                    },
                    None,
                )
            }
            LocalRef::Form(id) => {
                let form = self
                    .document
                    .metadata
                    .forms
                    .iter()
                    .find(|row| row.id == *id)
                    .ok_or_else(|| {
                        super::invalid_ref("LocalRef does not exist in its specific role table")
                    })?;
                let assertions = self
                    .document
                    .metadata
                    .assertions
                    .iter()
                    .filter(|row| row.entry == form.entry && assertion_selected(row))
                    .map(|row| row.id.to_string())
                    .collect();
                ("form", id, vec![form.entry.to_string()], assertions, None)
            }
            LocalRef::EditorialSense(id) => {
                let sense = self
                    .document
                    .metadata
                    .editorial_senses
                    .iter()
                    .find(|row| row.id == *id)
                    .ok_or_else(|| {
                        super::invalid_ref("LocalRef does not exist in its specific role table")
                    })?;
                let entries = sense
                    .assertions
                    .iter()
                    .filter_map(|assertion| {
                        self.document
                            .metadata
                            .assertions
                            .iter()
                            .find(|row| row.id == *assertion)
                            .filter(|row| assertion_selected(row))
                            .map(|row| row.entry.to_string())
                    })
                    .collect();
                (
                    "editorial_sense",
                    id,
                    entries,
                    sense
                        .assertions
                        .iter()
                        .filter(|id| {
                            self.document
                                .metadata
                                .assertions
                                .iter()
                                .find(|row| row.id == **id)
                                .is_some_and(&assertion_selected)
                        })
                        .map(ToString::to_string)
                        .collect(),
                    None,
                )
            }
        };
        let source_ids = entry_ids
            .iter()
            .filter_map(|entry| {
                self.document
                    .metadata
                    .entries
                    .iter()
                    .find(|row| row.id.to_string() == *entry)
                    .map(|row| row.source.to_string())
            })
            .collect();
        let projection_started = projection.as_ref().map(|_| std::time::Instant::now());
        let entity = vlajmaji_store::CollectionEntityReference {
            role: role.to_owned(),
            id: id.to_string(),
            entry_ids,
            assertion_ids: assertion_ids.clone(),
            source_ids,
            occurrence: match local {
                LocalRef::Assertion(id) => self
                    .document
                    .metadata
                    .assertions
                    .iter()
                    .find(|row| row.id == *id)
                    .map(|row| row.occurrence.clone()),
                _ => None,
            },
            evidence: None,
            spelling,
        };
        if let (Some(projection), Some(started)) = (projection.as_deref_mut(), projection_started) {
            *projection += started.elapsed();
        }
        let mut views = Vec::with_capacity(assertion_ids.len());
        for id in &assertion_ids {
            let Some(row) = self
                .document
                .metadata
                .assertions
                .iter()
                .find(|row| row.id.to_string() == *id)
            else {
                continue;
            };
            views.push(self.reference_assertion(row, projection.as_deref_mut())?);
        }
        Ok((entity, views))
    }

    fn reference_assertion(
        &self,
        assertion: &lexicon::AssertionRecord,
        projection: Option<&mut Duration>,
    ) -> Result<DictionaryCompactReferenceAssertion, DictionaryQueryError> {
        let entry = self
            .document
            .metadata
            .entries
            .iter()
            .find(|row| row.id == assertion.entry)
            .ok_or_else(|| DictionaryQueryError::Integrity {
                detail: "reference assertion has no entry".into(),
            })?;
        let occurrence_id = super::tuple_hex(&assertion.occurrence);
        let occurrence_attribution = self
            .document
            .registry
            .occurrences
            .iter()
            .find(|row| super::record_key(&row.occurrence) == occurrence_id);
        let capture = self
            .document
            .captures
            .iter()
            .find(|row| row.identity == assertion.occurrence.capture);
        let snapshot_attribution = self.document.registry.snapshots.iter().find(|row| {
            row.snapshot.namespace() == assertion.occurrence.capture.namespace
                && row.snapshot.sha256() == assertion.occurrence.capture.sha256
        });
        let current_eligible = matches!(assertion.eligibility, lexicon::Eligibility::Lexical)
            && self
                .selected
                .get(&entry.source)
                .is_some_and(|ids| ids.contains(&assertion.id.to_string()));
        let projection_started = projection.as_ref().map(|_| std::time::Instant::now());
        let detail_ref = if capture.is_some() {
            Some(DictionaryHitDetailRef {
                version: 1,
                digest_policy: DETAIL_POLICY.into(),
                role: "assertion".into(),
                source: entry.source.clone(),
                entry_id: entry.id.clone(),
                assertion_id: assertion.id.clone(),
                occurrence: assertion.occurrence.clone(),
                entry_sha256: digest_for_query(entry)?,
                assertion_sha256: digest_for_query(assertion)?,
                native_entry_id: occurrence_attribution
                    .as_ref()
                    .and_then(|row| row.native_entry_id.clone()),
                native_assertion_id: occurrence_attribution
                    .as_ref()
                    .and_then(|row| row.native_assertion_id.clone()),
                contributor_id: occurrence_attribution
                    .as_ref()
                    .and_then(|row| row.contributor.clone()),
                distributor_id: snapshot_attribution
                    .as_ref()
                    .and_then(|row| row.distributor.clone()),
            })
        } else {
            None
        };
        let projected = DictionaryCompactReferenceAssertion {
            entry: entry.clone(),
            assertion: assertion.clone(),
            occurrence_id,
            occurrence_attribution: occurrence_attribution.cloned(),
            capture_metadata: capture.map(|row| row.metadata.clone()),
            snapshot_attribution: snapshot_attribution.cloned(),
            payload_present: capture.is_some_and(|row| {
                row.occurrences
                    .iter()
                    .any(|raw| raw.locator == assertion.occurrence.locator)
            }),
            current_eligible,
            release_ids: self
                .document
                .metadata
                .releases
                .iter()
                .filter(|row| {
                    row.assertions.contains(&assertion.id)
                        && self
                            .overlay
                            .as_ref()
                            .is_none_or(|selected| selected.contains(&row.id))
                })
                .map(|row| row.id.clone())
                .collect(),
            owned_revision_ids: self
                .document
                .metadata
                .owned_revisions
                .iter()
                .filter(|row| row.assertions.contains(&assertion.id))
                .map(|row| row.id.clone())
                .collect(),
            tombstoned_by: self
                .document
                .metadata
                .owned_revisions
                .iter()
                .filter(|row| row.tombstones.contains(&assertion.id))
                .map(|row| row.id.clone())
                .collect(),
            detail_ref,
        };
        if let (Some(projection), Some(started)) = (projection, projection_started) {
            *projection += started.elapsed();
        }
        Ok(projected)
    }

    fn resolve_catalog(
        &self,
        catalog: &lexicon::CatalogTarget,
        mut projection: Option<&mut Duration>,
    ) -> Result<CompactCatalogResolution, DictionaryQueryError> {
        let metadata = &self.document.metadata;
        let release = metadata
            .releases
            .iter()
            .find(|row| row.id == catalog.release)
            .ok_or_else(|| super::invalid_ref("catalogue release does not exist"))?;
        let component = metadata
            .components
            .iter()
            .find(|row| row.id == release.component)
            .ok_or_else(|| super::invalid_ref("catalogue release has no component"))?;
        let collection = metadata
            .collections
            .iter()
            .find(|row| row.id == component.collection)
            .ok_or_else(|| super::invalid_ref("catalogue release has no collection"))?;
        if collection.source != catalog.source {
            return Err(super::invalid_ref(
                "catalogue release is not owned by the requested source",
            ));
        }
        if catalog.native_entry_id.is_none()
            && catalog.native_assertion_id.is_none()
            && catalog.occurrence.is_none()
        {
            return Err(super::invalid_ref(
                "catalogue target supplies no native identity or occurrence",
            ));
        }
        if let Some(occurrence) = &catalog.occurrence
            && (occurrence.locator.is_empty() || !release.captures.contains(&occurrence.capture))
        {
            return Err(super::invalid_ref(
                "catalogue occurrence is outside the requested release",
            ));
        }
        let mut confirmed = Vec::new();
        let mut unverified_views = Vec::new();
        for id in &release.assertions {
            let Some(assertion) = metadata.assertions.iter().find(|row| row.id == *id) else {
                continue;
            };
            if catalog
                .occurrence
                .as_ref()
                .is_some_and(|occurrence| occurrence != &assertion.occurrence)
            {
                continue;
            }
            let Some(entry) = metadata
                .entries
                .iter()
                .find(|row| row.id == assertion.entry)
            else {
                continue;
            };
            if entry.source != catalog.source {
                continue;
            }
            let attribution = self.document.registry.occurrences.iter().find(|row| {
                super::record_key(&row.occurrence) == super::tuple_hex(&assertion.occurrence)
            });
            let pairs = [
                (
                    &catalog.native_entry_id,
                    attribution.and_then(|row| row.native_entry_id.as_ref()),
                ),
                (
                    &catalog.native_assertion_id,
                    attribution.and_then(|row| row.native_assertion_id.as_ref()),
                ),
            ];
            if pairs.iter().any(|(expected, actual)| {
                expected
                    .as_ref()
                    .zip(*actual)
                    .is_some_and(|(expected, actual)| expected != actual)
            }) {
                continue;
            }
            if pairs
                .iter()
                .any(|(expected, actual)| expected.is_some() && actual.is_none())
            {
                unverified_views
                    .push(self.reference_assertion(assertion, projection.as_deref_mut())?);
                continue;
            }
            confirmed.push(self.reference_assertion(assertion, projection.as_deref_mut())?);
        }
        let status = if confirmed.is_empty() && unverified_views.is_empty() {
            if release.availability == Availability::Available {
                super::ReferenceStatus::NoMatch
            } else {
                super::ReferenceStatus::CatalogueOnly
            }
        } else if !unverified_views.is_empty() {
            super::ReferenceStatus::Unverified
        } else {
            super::ReferenceStatus::Resolved
        };
        let projection_started = projection.as_ref().map(|_| std::time::Instant::now());
        let requested_release = self
            .availability
            .iter()
            .flat_map(|row| &row.releases)
            .find(|row| row.id == release.id.to_string())
            .cloned();
        if let (Some(projection), Some(started)) = (projection, projection_started) {
            *projection += started.elapsed();
        }
        Ok((status, None, confirmed, unverified_views, requested_release))
    }
    /// Resolve a detail key and materialize the complete hit on demand.
    pub fn detail(
        &self,
        reference: &DictionaryHitDetailRef,
        expected_snapshot: Option<&str>,
    ) -> Result<DictionaryHitDetailResponse, DictionaryProjectionError> {
        reference.validate()?;
        if let Some(expected) = expected_snapshot {
            validate_detail_snapshot_sha256(expected)?;
            if expected != self.snapshot_sha256 {
                return Err(DictionaryProjectionError::StaleSnapshot {
                    expected: expected.into(),
                    observed: self.snapshot_sha256.clone(),
                });
            }
        }
        let no_match = || DictionaryHitDetailResponse {
            version: 1,
            projection_policy: "complete-dictionary-hit-v1".into(),
            detail_ref: reference.clone(),
            status: super::ReferenceStatus::NoMatch,
            operational_reason: None,
            state_sha256: Some(self.state_sha256.clone()),
            snapshot_sha256: self.snapshot_sha256.clone(),
            hit: None,
            current_eligible: None,
        };
        let Some(assertion) = self
            .document
            .metadata
            .assertions
            .iter()
            .find(|row| row.id == reference.assertion_id)
        else {
            return Ok(no_match());
        };
        if assertion.occurrence != reference.occurrence {
            return Err(DictionaryProjectionError::Query(Box::new(
                DictionaryQueryError::InvalidReference {
                    detail: "detail assertion occurrence does not match the requested key".into(),
                },
            )));
        }
        if assertion.entry != reference.entry_id {
            return Err(DictionaryProjectionError::Query(Box::new(
                DictionaryQueryError::InvalidReference {
                    detail: "detail assertion does not link to the requested entry".into(),
                },
            )));
        }
        let Some(entry) = self
            .document
            .metadata
            .entries
            .iter()
            .find(|row| row.id == reference.entry_id)
        else {
            return Ok(no_match());
        };
        if entry.source != reference.source {
            return Err(DictionaryProjectionError::Query(Box::new(
                DictionaryQueryError::InvalidReference {
                    detail: "detail entry source does not match the requested key".into(),
                },
            )));
        }
        if digest(entry)? != reference.entry_sha256
            || digest(assertion)? != reference.assertion_sha256
        {
            return Ok(no_match());
        }

        let capture = self
            .document
            .captures
            .iter()
            .find(|row| row.identity == assertion.occurrence.capture);
        if capture.is_none() {
            return Err(DictionaryProjectionError::Query(Box::new(
                DictionaryQueryError::InvalidReference {
                    detail: "detail occurrence capture owner is not retained".into(),
                },
            )));
        }
        let native = self.document.registry.occurrences.iter().find(|row| {
            super::record_key(&row.occurrence) == super::tuple_hex(&assertion.occurrence)
        });
        let snapshot = self.document.registry.snapshots.iter().find(|row| {
            row.snapshot.namespace() == assertion.occurrence.capture.namespace
                && row.snapshot.sha256() == assertion.occurrence.capture.sha256
        });
        let actual_native_entry = native.and_then(|row| row.native_entry_id.as_deref());
        let actual_native_assertion = native.and_then(|row| row.native_assertion_id.as_deref());
        if reference
            .native_entry_id
            .as_deref()
            .is_some_and(|requested| {
                actual_native_entry != Some(requested) && actual_native_assertion == Some(requested)
            })
            || reference
                .native_assertion_id
                .as_deref()
                .is_some_and(|requested| {
                    actual_native_assertion != Some(requested)
                        && actual_native_entry == Some(requested)
                })
        {
            return Err(DictionaryProjectionError::Query(Box::new(
                DictionaryQueryError::InvalidReference {
                    detail: "native identity was supplied in the wrong role slot".into(),
                },
            )));
        }
        if actual_native_entry != reference.native_entry_id.as_deref()
            || actual_native_assertion != reference.native_assertion_id.as_deref()
            || native.and_then(|row| row.contributor.as_ref()) != reference.contributor_id.as_ref()
            || snapshot.and_then(|row| row.distributor.as_ref())
                != reference.distributor_id.as_ref()
        {
            return Ok(no_match());
        }
        let hit = self.complete_hit(entry, assertion)?;
        let current_eligible = matches!(assertion.eligibility, lexicon::Eligibility::Lexical)
            && self
                .selected
                .get(&entry.source)
                .is_some_and(|ids| ids.contains(&assertion.id.to_string()));
        Ok(DictionaryHitDetailResponse {
            version: 1,
            projection_policy: "complete-dictionary-hit-v1".into(),
            detail_ref: reference.clone(),
            status: super::ReferenceStatus::Resolved,
            operational_reason: None,
            state_sha256: Some(self.state_sha256.clone()),
            snapshot_sha256: self.snapshot_sha256.clone(),
            hit: Some(hit),
            current_eligible: Some(current_eligible),
        })
    }
    fn compact_hit(
        &self,
        entry: &EntryRecord,
        assertion: &lexicon::AssertionRecord,
    ) -> Result<DictionaryCompactHit, DictionaryQueryError> {
        #[cfg(test)]
        COMPACT_HIT_PROJECTIONS.with(|count| count.set(count.get() + 1));

        let capture = self
            .document
            .captures
            .iter()
            .find(|row| row.identity == assertion.occurrence.capture)
            .ok_or_else(|| DictionaryQueryError::Integrity {
                detail: format!("selected assertion {} has no capture", assertion.id),
            })?;
        let snapshot_attribution = self
            .document
            .registry
            .snapshots
            .iter()
            .find(|row| {
                row.snapshot.namespace() == assertion.occurrence.capture.namespace
                    && row.snapshot.sha256() == assertion.occurrence.capture.sha256
            })
            .cloned()
            .ok_or_else(|| DictionaryQueryError::Integrity {
                detail: "loaded capture has no snapshot attribution".into(),
            })?;
        let occurrence_attribution = self
            .document
            .registry
            .occurrences
            .iter()
            .find(|row| {
                super::record_key(&row.occurrence) == super::tuple_hex(&assertion.occurrence)
            })
            .cloned();
        let contributor = occurrence_attribution
            .as_ref()
            .and_then(|row| row.contributor.as_ref())
            .and_then(|id| {
                self.document
                    .registry
                    .contributors
                    .iter()
                    .find(|row| &row.id == id)
            })
            .cloned();
        let distributor = snapshot_attribution
            .distributor
            .as_ref()
            .and_then(|id| {
                self.document
                    .registry
                    .distributors
                    .iter()
                    .find(|row| &row.id == id)
            })
            .cloned();
        let selected: BTreeSet<_> = self
            .document
            .metadata
            .releases
            .iter()
            .filter(|release| {
                release.selected
                    && release.availability == Availability::Available
                    && release.assertions.contains(&assertion.id)
                    && self
                        .overlay
                        .as_ref()
                        .is_none_or(|ids| ids.contains(&release.id))
            })
            .map(|release| release.id.clone())
            .collect();
        let release_ids = self
            .document
            .metadata
            .releases
            .iter()
            .filter(|release| {
                selected.contains(&release.id) && release.assertions.contains(&assertion.id)
            })
            .map(|release| release.id.clone())
            .collect();
        let detail_ref = DictionaryHitDetailRef {
            version: 1,
            digest_policy: DETAIL_POLICY.into(),
            role: "assertion".into(),
            source: entry.source.clone(),
            entry_id: entry.id.clone(),
            assertion_id: assertion.id.clone(),
            occurrence: assertion.occurrence.clone(),
            entry_sha256: digest_for_query(entry)?,
            assertion_sha256: digest_for_query(assertion)?,
            native_entry_id: occurrence_attribution
                .as_ref()
                .and_then(|row| row.native_entry_id.clone()),
            native_assertion_id: occurrence_attribution
                .as_ref()
                .and_then(|row| row.native_assertion_id.clone()),
            contributor_id: occurrence_attribution
                .as_ref()
                .and_then(|row| row.contributor.clone()),
            distributor_id: snapshot_attribution.distributor.clone(),
        };
        Ok(DictionaryCompactHit {
            occurrence_id: super::tuple_hex(&assertion.occurrence),
            entry: entry.clone(),
            assertion: assertion.clone(),
            capture: capture.metadata.clone(),
            snapshot_attribution,
            occurrence_attribution,
            contributor,
            distributor,
            release_ids,
            detail_ref,
        })
    }
    fn complete_hit(
        &self,
        entry: &EntryRecord,
        assertion: &lexicon::AssertionRecord,
    ) -> Result<DictionaryHit, DictionaryQueryError> {
        let document = &self.document;
        let registry = &document.registry;
        let capture = document
            .captures
            .iter()
            .find(|row| row.identity == assertion.occurrence.capture)
            .ok_or_else(|| DictionaryQueryError::Integrity {
                detail: "detail capture is missing".into(),
            })?;
        let raw = capture
            .occurrences
            .iter()
            .find(|row| row.locator == assertion.occurrence.locator)
            .cloned()
            .ok_or_else(|| DictionaryQueryError::Integrity {
                detail: "detail native occurrence is missing".into(),
            })?;
        let snapshot_attribution = registry
            .snapshots
            .iter()
            .find(|row| {
                row.snapshot.namespace() == assertion.occurrence.capture.namespace
                    && row.snapshot.sha256() == assertion.occurrence.capture.sha256
            })
            .cloned()
            .ok_or_else(|| DictionaryQueryError::Integrity {
                detail: "detail snapshot attribution is missing".into(),
            })?;
        let occurrence_attribution = registry
            .occurrences
            .iter()
            .find(|row| {
                super::record_key(&row.occurrence) == super::tuple_hex(&assertion.occurrence)
            })
            .cloned();
        let contributor = occurrence_attribution
            .as_ref()
            .and_then(|row| row.contributor.as_ref())
            .and_then(|id| registry.contributors.iter().find(|row| &row.id == id))
            .cloned();
        let distributor = snapshot_attribution
            .distributor
            .as_ref()
            .and_then(|id| registry.distributors.iter().find(|row| &row.id == id))
            .cloned();
        let contexts: Vec<_> = document
            .metadata
            .contexts
            .iter()
            .filter(|row| row.assertion == assertion.id)
            .cloned()
            .collect();
        let frames: Vec<_> = document
            .metadata
            .frames
            .iter()
            .filter(|row| row.assertion == assertion.id)
            .cloned()
            .collect();
        let forms: Vec<_> = document
            .metadata
            .forms
            .iter()
            .filter(|row| row.entry == entry.id)
            .cloned()
            .collect();
        let etymologies: Vec<_> = document
            .metadata
            .etymologies
            .iter()
            .filter(|row| row.assertion == assertion.id)
            .cloned()
            .collect();
        let editorial_senses: Vec<_> = document
            .metadata
            .editorial_senses
            .iter()
            .filter(|row| row.assertions.contains(&assertion.id))
            .cloned()
            .collect();
        let mut local_keys = BTreeSet::from([
            super::local_ref_key(&lexicon::LocalRef::Entry(entry.id.clone())),
            super::local_ref_key(&lexicon::LocalRef::Assertion(assertion.id.clone())),
        ]);
        for form in &forms {
            local_keys.insert(super::local_ref_key(&lexicon::LocalRef::Form(
                form.id.clone(),
            )));
        }
        for sense in &editorial_senses {
            local_keys.insert(super::local_ref_key(&lexicon::LocalRef::EditorialSense(
                sense.id.clone(),
            )));
        }
        let relations: Vec<_> = document
            .metadata
            .relations
            .iter()
            .filter(|row| local_keys.contains(&super::local_ref_key(&row.origin)))
            .cloned()
            .collect();
        let annotations: Vec<_> = document
            .metadata
            .annotations
            .iter()
            .filter(|row| local_keys.contains(&super::local_ref_key(&row.target)))
            .cloned()
            .collect();
        let computations: Vec<_> = document
            .metadata
            .computations
            .iter()
            .filter(|row| {
                forms
                    .iter()
                    .any(|form| form.computation.as_ref() == Some(&row.id))
                    || annotations
                        .iter()
                        .any(|annotation| annotation.computation == row.id)
            })
            .cloned()
            .collect();
        let source_availability = self
            .availability
            .iter()
            .find(|row| row.source.id == entry.source)
            .ok_or_else(|| DictionaryQueryError::Integrity {
                detail: "hit source absent from compact availability".into(),
            })?;
        let release_ids: Vec<_> = document
            .metadata
            .releases
            .iter()
            .filter(|row| {
                row.selected
                    && row.availability == Availability::Available
                    && row.assertions.contains(&assertion.id)
                    && self
                        .overlay
                        .as_ref()
                        .is_none_or(|ids| ids.contains(&row.id))
            })
            .map(|row| row.id.clone())
            .collect();
        let mut owned_ids: BTreeSet<_> = [entry.id.to_string(), assertion.id.to_string()]
            .into_iter()
            .collect();
        owned_ids.extend(forms.iter().map(|row| row.id.to_string()));
        owned_ids.extend(editorial_senses.iter().map(|row| row.id.to_string()));
        owned_ids.extend(contexts.iter().map(|row| row.id.to_string()));
        owned_ids.extend(frames.iter().map(|row| row.id.to_string()));
        owned_ids.extend(etymologies.iter().map(|row| row.id.to_string()));
        owned_ids.extend(relations.iter().map(|row| row.id.to_string()));
        owned_ids.extend(computations.iter().map(|row| row.id.to_string()));
        owned_ids.extend(annotations.iter().map(|row| row.id.to_string()));
        owned_ids.extend(
            source_availability
                .collections
                .iter()
                .map(|row| row.id.to_string()),
        );
        owned_ids.extend(
            source_availability
                .components
                .iter()
                .map(|row| row.id.to_string()),
        );
        owned_ids.extend(
            source_availability
                .selected_available_release_ids
                .iter()
                .map(ToString::to_string),
        );
        let extensions = document
            .metadata
            .extensions
            .iter()
            .filter(|row| owned_ids.contains(&row.owner.to_string()))
            .cloned()
            .collect();
        let owned_revision_ids = document
            .metadata
            .owned_revisions
            .iter()
            .filter(|row| row.assertions.contains(&assertion.id))
            .map(|row| row.id.clone())
            .collect();
        let tombstoned_by = document
            .metadata
            .owned_revisions
            .iter()
            .filter(|row| row.tombstones.contains(&assertion.id))
            .map(|row| row.id.clone())
            .collect();
        Ok(DictionaryHit {
            occurrence_id: super::tuple_hex(&assertion.occurrence),
            entry: entry.clone(),
            assertion: assertion.clone(),
            capture: capture.metadata.clone(),
            raw,
            snapshot_attribution,
            occurrence_attribution,
            contributor,
            distributor,
            release_ids,
            contexts,
            frames,
            forms,
            etymologies,
            relations,
            editorial_senses,
            computations,
            annotations,
            extensions,
            owned_revision_ids,
            tombstoned_by,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    mod fixture {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../lexicon/examples/support/dictionary_fixture.rs"
        ));
    }

    static NEXT: AtomicU64 = AtomicU64::new(1);

    fn scratch() -> std::path::PathBuf {
        let root = std::env::temp_dir().join(format!(
            "vlajmaji-compact-projection-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&root).unwrap();
        root
    }

    #[test]
    fn compact_zero_page_skips_hit_projection_but_keeps_counts() {
        let root = scratch();
        let store_path = root.join("dictionary.sqlite3");
        let dictionary = fixture::complete_fixture().unwrap();
        let store = DictionaryStore::create(&store_path, &dictionary).unwrap();
        drop(store);
        let engine = DictionaryCompactSearchEngine::open(&store_path).unwrap();
        COMPACT_HIT_PROJECTIONS.with(|count| count.set(0));

        let zero = engine
            .exact(
                "farvi",
                DictionaryExactOptions {
                    page_size: Some(0),
                    ..DictionaryExactOptions::default()
                },
            )
            .unwrap();
        assert!(zero.totals.occurrences > 0);
        assert!(!zero.source_totals.is_empty());
        assert!(zero.groups.is_empty());
        assert!(zero.native_candidate_occurrences > zero.totals.occurrences);
        assert_eq!(COMPACT_HIT_PROJECTIONS.with(Cell::get), 0);

        let one = engine
            .exact(
                "farvi",
                DictionaryExactOptions {
                    page_size: Some(1),
                    ..DictionaryExactOptions::default()
                },
            )
            .unwrap();
        assert_eq!(
            COMPACT_HIT_PROJECTIONS.with(Cell::get),
            one.groups
                .iter()
                .map(|group| group.hits.len())
                .sum::<usize>()
        );
        drop(engine);
        std::fs::remove_dir_all(root).unwrap();
    }
}

fn digest<T: Serialize>(record: &T) -> Result<String, DictionaryProjectionError> {
    let bytes = serde_json::to_vec(record)
        .map_err(|source| DictionaryProjectionError::ReferenceJson { source })?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}
fn digest_for_query<T: Serialize>(record: &T) -> Result<String, DictionaryQueryError> {
    let bytes = serde_json::to_vec(record).map_err(|error| DictionaryQueryError::Integrity {
        detail: format!("serialize detail digest: {error}"),
    })?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}
fn presence_value<T>(presence: &Presence<T>) -> Option<&T> {
    match presence {
        Presence::Value(value) => Some(value),
        Presence::Missing | Presence::Null => None,
    }
}
fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
fn compact_query_binding(
    snapshot: &str,
    spelling: &str,
    sources: &[SourceId],
    alias_scope: lexicon::AliasScope,
) -> Result<String, DictionaryQueryError> {
    #[derive(Serialize)]
    struct Binding<'a> {
        version: u32,
        policy: &'a str,
        snapshot_sha256: &'a str,
        spelling: &'a str,
        resolved_sources: &'a [SourceId],
        alias_scope: lexicon::AliasScope,
        selection_policy: &'a str,
    }
    let bytes = serde_json::to_vec(&Binding {
        version: 1,
        policy: COMPACT_POLICY,
        snapshot_sha256: snapshot,
        spelling,
        resolved_sources: sources,
        alias_scope,
        selection_policy: super::SELECTION_POLICY,
    })
    .map_err(|error| DictionaryQueryError::Integrity {
        detail: format!("serialize compact cursor binding: {error}"),
    })?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

fn validate_detail_reference(
    reference: &DictionaryHitDetailRef,
) -> Result<(), DictionaryProjectionError> {
    let reason = if reference.version != 1 {
        Some(DictionaryDetailReferenceInvalidity::UnsupportedVersion)
    } else if reference.digest_policy != DETAIL_POLICY {
        Some(DictionaryDetailReferenceInvalidity::UnsupportedDigestPolicy)
    } else if reference.role != "assertion" {
        Some(DictionaryDetailReferenceInvalidity::WrongRole)
    } else if !valid_sha256(&reference.entry_sha256) || !valid_sha256(&reference.assertion_sha256) {
        Some(DictionaryDetailReferenceInvalidity::InvalidDigest)
    } else if reference.occurrence.capture.namespace.is_empty()
        || !valid_sha256(&reference.occurrence.capture.sha256)
    {
        Some(DictionaryDetailReferenceInvalidity::InvalidCapture)
    } else if reference.occurrence.locator.is_empty() {
        Some(DictionaryDetailReferenceInvalidity::EmptyLocator)
    } else {
        None
    };
    reason.map_or(Ok(()), |reason| {
        Err(DictionaryProjectionError::InvalidDetailReference { reason })
    })
}

fn compact_snapshot_sha256(
    state: &str,
    generation: Option<&str>,
    overlay: Option<&[EntityId]>,
    availability: &[DictionaryCompactSourceAvailability],
) -> Result<String, DictionaryQueryError> {
    #[derive(Serialize)]
    struct Body<'a> {
        version: u32,
        policy: &'static str,
        recorded_state_sha256: &'a str,
        generation: Option<&'a str>,
        selected_release_ids: Option<&'a [EntityId]>,
        availability: Vec<CompactSnapshotSource<'a>>,
    }
    #[derive(Serialize)]
    struct CompactSnapshotSource<'a> {
        source_id: &'a SourceId,
        selected_available_release_ids: Vec<&'a EntityId>,
        loaded_captures: &'a [CaptureRef],
        missing_captures: &'a [CaptureRef],
        owned_head: Option<&'a lexicon::OwnedHead>,
        selectable: bool,
        reason: Option<super::SourceSelectionFailure>,
    }
    let mut ids = overlay.map(<[EntityId]>::to_vec);
    if let Some(ids) = &mut ids {
        ids.sort();
    }
    let sources = availability
        .iter()
        .map(|row| CompactSnapshotSource {
            source_id: &row.source.id,
            selected_available_release_ids: row.selected_available_release_ids.iter().collect(),
            loaded_captures: &row.loaded_captures,
            missing_captures: &row.missing_captures,
            owned_head: row.owned_head.as_ref(),
            selectable: row.selectable,
            reason: row.reason,
        })
        .collect();
    let bytes = serde_json::to_vec(&Body {
        version: 1,
        policy: COMPACT_POLICY,
        recorded_state_sha256: state,
        generation,
        selected_release_ids: ids.as_deref(),
        availability: sources,
    })
    .map_err(|error| DictionaryQueryError::Integrity {
        detail: format!("serialize compact snapshot: {error}"),
    })?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CursorBody {
    version: u32,
    binding: String,
    last: CursorKey,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CursorKey {
    source: SourceId,
    spelling: String,
    occurrence_id: String,
    assertion_id: String,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CursorEnvelope {
    body: CursorBody,
    checksum: String,
}
fn make_cursor(
    binding: &str,
    spelling: &str,
    source: &SourceId,
    occurrence: &OccurrenceRef,
    assertion: &EntityId,
) -> String {
    let body = CursorBody {
        version: 1,
        binding: binding.into(),
        last: CursorKey {
            source: source.clone(),
            spelling: spelling.into(),
            occurrence_id: super::tuple_hex(occurrence),
            assertion_id: assertion.to_string(),
        },
    };
    let bytes = serde_json::to_vec(&body).expect("cursor body serialization is infallible");
    let checksum = format!("{:x}", Sha256::digest(&bytes));
    let envelope = serde_json::to_vec(&CursorEnvelope { body, checksum })
        .expect("cursor envelope serialization is infallible");
    format!("dictionary-compact-v1:{}", hex(&envelope))
}
fn cursor_position(
    cursor: &str,
    binding: &str,
    spelling: &str,
    rows: &[(usize, usize)],
    document: &lexicon::UnverifiedDictionaryDocument,
) -> Result<usize, DictionaryQueryError> {
    let Some(hex_data) = cursor.strip_prefix("dictionary-compact-v1:") else {
        return if is_valid_foreign_cursor(cursor) {
            Err(DictionaryQueryError::StaleCursor)
        } else {
            Err(DictionaryQueryError::MalformedCursor {
                detail: "unknown or malformed cursor projection".into(),
            })
        };
    };
    let bytes = decode_hex(hex_data).ok_or_else(|| DictionaryQueryError::MalformedCursor {
        detail: "invalid cursor hex".into(),
    })?;
    let envelope: CursorEnvelope =
        serde_json::from_slice(&bytes).map_err(|error| DictionaryQueryError::MalformedCursor {
            detail: error.to_string(),
        })?;
    if envelope.body.version != 1 {
        return Err(DictionaryQueryError::MalformedCursor {
            detail: "unsupported cursor version".into(),
        });
    }
    let body = serde_json::to_vec(&envelope.body).map_err(|error| {
        DictionaryQueryError::MalformedCursor {
            detail: error.to_string(),
        }
    })?;
    if !valid_sha256(&envelope.body.binding)
        || !valid_sha256(&envelope.checksum)
        || format!("{:x}", Sha256::digest(&body)) != envelope.checksum
        || !valid_occurrence_hex(&envelope.body.last.occurrence_id)
    {
        return Err(DictionaryQueryError::MalformedCursor {
            detail: "cursor checksum or binding is invalid".into(),
        });
    }
    if envelope.body.binding != binding {
        return Err(DictionaryQueryError::StaleCursor);
    }
    if envelope.body.last.spelling != spelling {
        return Err(DictionaryQueryError::InvalidCursorPosition);
    }
    rows.iter()
        .position(|(entry_index, assertion_index)| {
            let entry = &document.metadata.entries[*entry_index];
            let assertion = &document.metadata.assertions[*assertion_index];
            entry.source == envelope.body.last.source
                && presence_value(&entry.spelling).is_some_and(|value| value == spelling)
                && super::tuple_hex(&assertion.occurrence) == envelope.body.last.occurrence_id
                && assertion.id.to_string() == envelope.body.last.assertion_id
        })
        .map(|position| position + 1)
        .ok_or(DictionaryQueryError::InvalidCursorPosition)
}

fn is_valid_foreign_cursor(cursor: &str) -> bool {
    if let Some(raw_hex) = cursor.strip_prefix("collection-v1:") {
        let Some((encoded, checksum)) = raw_hex.split_once(':') else {
            return false;
        };
        if !valid_sha256(checksum) {
            return false;
        }
        let Some(bytes) = decode_hex(encoded) else {
            return false;
        };
        if format!("{:x}", Sha256::digest(&bytes)) != checksum {
            return false;
        }
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct ManagedCursor {
            version: u32,
            snapshot_binding: String,
            inner: String,
        }
        let Ok(managed) = serde_json::from_slice::<ManagedCursor>(&bytes) else {
            return false;
        };
        return managed.version == 1
            && valid_sha256(&managed.snapshot_binding)
            && is_valid_full_cursor(&managed.inner);
    }
    is_valid_full_cursor(cursor)
}

fn is_valid_full_cursor(cursor: &str) -> bool {
    #[derive(Deserialize, Serialize)]
    #[serde(deny_unknown_fields)]
    struct FullBody {
        version: u32,
        binding: String,
        last: FullKey,
    }
    #[derive(Deserialize, Serialize)]
    #[serde(deny_unknown_fields)]
    struct FullKey {
        source: SourceId,
        spelling: String,
        occurrence_id: String,
    }
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct FullEnvelope {
        body: FullBody,
        checksum: String,
    }
    let Some(bytes) = decode_hex(cursor) else {
        return false;
    };
    let Ok(envelope) = serde_json::from_slice::<FullEnvelope>(&bytes) else {
        return false;
    };
    let Ok(body) = serde_json::to_vec(&envelope.body) else {
        return false;
    };
    let _last_is_well_formed = (
        envelope.body.last.source,
        envelope.body.last.spelling,
        envelope.body.last.occurrence_id,
    );
    envelope.body.version == 1
        && valid_sha256(&envelope.body.binding)
        && valid_sha256(&envelope.checksum)
        && valid_occurrence_hex(&_last_is_well_formed.2)
        && format!("{:x}", Sha256::digest(body)) == envelope.checksum
}

fn valid_occurrence_hex(value: &str) -> bool {
    !value.is_empty()
        && value.len().is_multiple_of(2)
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn decode_hex(value: &str) -> Option<Vec<u8>> {
    if value.is_empty()
        || !value.len().is_multiple_of(2)
        || !value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return None;
    }
    (0..value.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&value[index..index + 2], 16).ok())
        .collect()
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
