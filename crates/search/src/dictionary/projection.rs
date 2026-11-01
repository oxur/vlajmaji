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

pub(super) fn measure_projection<T>(
    profile: &mut Option<&mut DictionaryProjectionProfile>,
    operation: impl FnOnce() -> T,
) -> T {
    if let Some(profile) = profile.as_deref_mut() {
        let started = std::time::Instant::now();
        let value = operation();
        profile.projection += started.elapsed();
        value
    } else {
        operation()
    }
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
    snapshot_sha256: String,
}

impl DictionaryCompactSearchEngine {
    pub(crate) fn owned_release_summary(
        &self,
        release_id: &EntityId,
        source_id: &SourceId,
    ) -> Option<vlajmaji_store::CollectionReleaseSummary> {
        let metadata = &self.document.metadata;
        let release = metadata
            .releases
            .iter()
            .find(|release| release.id == *release_id)?;
        let component = metadata
            .components
            .iter()
            .find(|component| component.id == release.component)?;
        let collection = metadata
            .collections
            .iter()
            .find(|collection| collection.id == component.collection)?;
        (collection.source == *source_id)
            .then(|| vlajmaji_store::CollectionReleaseSummary::from(release))
    }

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
        let (indices, (availability, selected)) =
            borrowed::index_document(&document, overlay.as_deref())?;
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

    pub(crate) fn lexical_compact(
        &self,
        request: &crate::workbench::lexical::WorkbenchLexicalRequest,
        binding_sha256: &str,
    ) -> Result<
        crate::workbench::lexical::WorkbenchLexicalCompactResult,
        crate::workbench::lexical::WorkbenchLexicalError,
    > {
        use crate::workbench::lexical;
        let available = self
            .availability
            .iter()
            .map(|row| (row.source.id.clone(), row.selectable, row.reason))
            .collect::<Vec<_>>();
        let resolved_sources =
            lexical::resolve_sources(&self.document.registry, &request.options, &available)?;
        let selected_sources: BTreeSet<_> = resolved_sources.iter().cloned().collect();
        match &request.query {
            lexical::WorkbenchLexicalQuery::ReverseDefinition { contains } => {
                let needle = contains.to_ascii_lowercase();
                let mut candidates = Vec::new();
                for assertion in &self.document.metadata.assertions {
                    if !matches!(assertion.eligibility, lexicon::Eligibility::Lexical) {
                        continue;
                    }
                    let entry_index = *self
                        .indices
                        .entries
                        .get(&assertion.entry.to_string())
                        .ok_or_else(|| lexical::WorkbenchLexicalError::Integrity {
                            detail: format!(
                                "assertion {} references a missing Entry",
                                assertion.id
                            ),
                        })?;
                    let entry = &self.document.metadata.entries[entry_index];
                    if !selected_sources.contains(&entry.source)
                        || !self
                            .selected
                            .get(&entry.source)
                            .is_some_and(|ids| ids.contains(assertion.id.as_str()))
                    {
                        continue;
                    }
                    let lexicon::Presence::Value(definition) = &assertion.definition else {
                        continue;
                    };
                    if !definition.text.to_ascii_lowercase().contains(&needle) {
                        continue;
                    }
                    let spelling = match &entry.spelling {
                        lexicon::Presence::Value(value) => value.clone(),
                        _ => String::new(),
                    };
                    candidates.push((
                        entry_index,
                        assertion,
                        spelling,
                        super::tuple_hex(&assertion.occurrence),
                    ));
                }
                candidates.sort_by(|left, right| {
                    let left_entry = &self.document.metadata.entries[left.0];
                    let right_entry = &self.document.metadata.entries[right.0];
                    (&left_entry.source, &left.2, &left.3, &left.1.id).cmp(&(
                        &right_entry.source,
                        &right.2,
                        &right.3,
                        &right.1.id,
                    ))
                });
                let start = if let Some(token) = request.options.cursor.as_deref() {
                    let position = lexical::decode_cursor(
                        token,
                        request,
                        &resolved_sources,
                        "compact",
                        binding_sha256,
                    )?;
                    candidates
                        .iter()
                        .position(|candidate| match &position {
                            lexical::WorkbenchLexicalPosition::ReverseDefinition {
                                source,
                                spelling,
                                occurrence_id,
                                assertion_id,
                            } => {
                                let entry = &self.document.metadata.entries[candidate.0];
                                &entry.source == source
                                    && &candidate.2 == spelling
                                    && &candidate.3 == occurrence_id
                                    && &candidate.1.id == assertion_id
                            }
                            _ => false,
                        })
                        .map(|index| index + 1)
                        .ok_or(lexical::WorkbenchLexicalError::InvalidCursorPosition)?
                } else {
                    0
                };
                let end = request.options.page_size.map_or(candidates.len(), |size| {
                    start.saturating_add(size).min(candidates.len())
                });
                let mut counts = BTreeMap::<SourceId, usize>::new();
                let mut page = BTreeMap::<SourceId, Vec<usize>>::new();
                for (index, candidate) in candidates.iter().enumerate() {
                    let source = &self.document.metadata.entries[candidate.0].source;
                    *counts.entry(source.clone()).or_default() += 1;
                    if (start..end).contains(&index) {
                        page.entry(source.clone()).or_default().push(index);
                    }
                }
                let source_totals = counts
                    .iter()
                    .map(|(id, count)| {
                        let source = self
                            .availability
                            .iter()
                            .find(|row| &row.source.id == id)
                            .map(|row| row.source.clone())
                            .ok_or_else(|| lexical::WorkbenchLexicalError::Integrity {
                                detail: format!("matching source {id} is missing"),
                            })?;
                        Ok(lexical::WorkbenchLexicalSourceTotal {
                            source,
                            records: *count,
                            associated_assertions: *count,
                        })
                    })
                    .collect::<Result<Vec<_>, lexical::WorkbenchLexicalError>>()?;
                let totals = lexical::WorkbenchLexicalTotals {
                    unit: "assertion_occurrences".into(),
                    records: candidates.len(),
                    source_groups: counts.len(),
                    associated_assertions: candidates.len(),
                };
                let groups = page
                    .into_iter()
                    .map(|(id, indices)| {
                        let source = self
                            .availability
                            .iter()
                            .find(|row| row.source.id == id)
                            .map(|row| row.source.clone())
                            .ok_or_else(|| lexical::WorkbenchLexicalError::Integrity {
                                detail: format!("matching source {id} is missing"),
                            })?;
                        let hits = indices
                            .iter()
                            .map(|index| {
                                let candidate = &candidates[*index];
                                let entry = &self.document.metadata.entries[candidate.0];
                                self.compact_hit(entry, candidate.1, None)
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        let full_count = counts[&id];
                        let returned_count = hits.len();
                        Ok(lexical::WorkbenchReverseDefinitionCompactGroup {
                            source,
                            full_count,
                            returned_count,
                            partial: returned_count < full_count,
                            hits,
                        })
                    })
                    .collect::<Result<Vec<_>, lexical::WorkbenchLexicalError>>()?;
                let next_cursor = if end < candidates.len() && end > start {
                    let candidate = &candidates[end - 1];
                    let entry = &self.document.metadata.entries[candidate.0];
                    Some(lexical::encode_cursor(
                        request,
                        &resolved_sources,
                        "compact",
                        binding_sha256,
                        lexical::WorkbenchLexicalPosition::ReverseDefinition {
                            source: entry.source.clone(),
                            spelling: candidate.2.clone(),
                            occurrence_id: candidate.3.clone(),
                            assertion_id: candidate.1.id.clone(),
                        },
                    )?)
                } else {
                    None
                };
                Ok(lexical::WorkbenchLexicalCompactResult::ReverseDefinition(
                    Box::new(lexical::WorkbenchReverseDefinitionCompact {
                        policy: "literal-definition-ascii-v1".into(),
                        state_sha256: self.state_sha256.clone(),
                        query: request.query.clone(),
                        resolved_sources,
                        totals,
                        source_totals,
                        availability: self.availability.clone(),
                        groups,
                        next_cursor,
                        code_notices: self.document.metadata.code_notices.clone(),
                        snapshot_sha256: self.snapshot_sha256.clone(),
                    }),
                ))
            }
            lexical::WorkbenchLexicalQuery::Form { value, role, kind } => {
                struct Candidate<'a> {
                    form: &'a lexicon::FormRecord,
                    entry: &'a lexicon::EntryRecord,
                    entry_spelling: String,
                }
                let mut candidates = Vec::new();
                for form in &self.document.metadata.forms {
                    if form.spelling != *value
                        || kind.is_some_and(|expected| expected != form.kind)
                        || role.is_some_and(|expected| {
                            lexical::mapped_role(form.kind) != Some(expected)
                        })
                    {
                        continue;
                    }
                    let entry_index = *self
                        .indices
                        .entries
                        .get(&form.entry.to_string())
                        .ok_or_else(|| lexical::WorkbenchLexicalError::Integrity {
                            detail: format!("Form {} references a missing Entry", form.id),
                        })?;
                    let entry = &self.document.metadata.entries[entry_index];
                    if !selected_sources.contains(&entry.source) {
                        continue;
                    }
                    let entry_spelling = match &entry.spelling {
                        lexicon::Presence::Value(value) => value.clone(),
                        _ => String::new(),
                    };
                    candidates.push(Candidate {
                        form,
                        entry,
                        entry_spelling,
                    });
                }
                candidates.sort_by(|left, right| {
                    (
                        &left.entry.source,
                        &left.entry_spelling,
                        &left.form.spelling,
                        lexical::form_kind_rank(left.form.kind),
                        &left.form.id,
                    )
                        .cmp(&(
                            &right.entry.source,
                            &right.entry_spelling,
                            &right.form.spelling,
                            lexical::form_kind_rank(right.form.kind),
                            &right.form.id,
                        ))
                });
                let candidate_sources = candidates
                    .iter()
                    .map(|candidate| (candidate.entry.id.clone(), candidate.entry.source.clone()))
                    .collect::<BTreeMap<_, _>>();
                let mut associations: BTreeMap<String, Vec<&lexicon::AssertionRecord>> =
                    BTreeMap::new();
                let mut union_by_source = BTreeMap::<SourceId, BTreeSet<EntityId>>::new();
                for assertion in &self.document.metadata.assertions {
                    let Some(source) = candidate_sources.get(&assertion.entry) else {
                        continue;
                    };
                    if !matches!(assertion.eligibility, lexicon::Eligibility::Lexical)
                        || !self
                            .selected
                            .get(source)
                            .is_some_and(|ids| ids.contains(assertion.id.as_str()))
                    {
                        continue;
                    }
                    associations
                        .entry(assertion.entry.to_string())
                        .or_default()
                        .push(assertion);
                    union_by_source
                        .entry(source.clone())
                        .or_default()
                        .insert(assertion.id.clone());
                }
                for rows in associations.values_mut() {
                    rows.sort_by(|left, right| {
                        (super::tuple_hex(&left.occurrence), &left.id)
                            .cmp(&(super::tuple_hex(&right.occurrence), &right.id))
                    });
                }
                let start = if let Some(token) = request.options.cursor.as_deref() {
                    let position = lexical::decode_cursor(
                        token,
                        request,
                        &resolved_sources,
                        "compact",
                        binding_sha256,
                    )?;
                    candidates.iter().position(|candidate| matches!(&position,
                        lexical::WorkbenchLexicalPosition::Form { source, entry_spelling, spelling, kind, form_id }
                        if &candidate.entry.source == source && &candidate.entry_spelling == entry_spelling
                            && &candidate.form.spelling == spelling && candidate.form.kind == *kind && &candidate.form.id == form_id
                    )).map(|index| index + 1).ok_or(lexical::WorkbenchLexicalError::InvalidCursorPosition)?
                } else {
                    0
                };
                let end = request.options.page_size.map_or(candidates.len(), |size| {
                    start.saturating_add(size).min(candidates.len())
                });
                let mut counts = BTreeMap::<SourceId, usize>::new();
                let mut page = BTreeMap::<SourceId, Vec<usize>>::new();
                for (index, candidate) in candidates.iter().enumerate() {
                    *counts.entry(candidate.entry.source.clone()).or_default() += 1;
                    if (start..end).contains(&index) {
                        page.entry(candidate.entry.source.clone())
                            .or_default()
                            .push(index);
                    }
                }
                let source_totals = counts
                    .iter()
                    .map(|(id, count)| {
                        let source = self
                            .availability
                            .iter()
                            .find(|row| &row.source.id == id)
                            .map(|row| row.source.clone())
                            .ok_or_else(|| lexical::WorkbenchLexicalError::Integrity {
                                detail: format!("matching source {id} is missing"),
                            })?;
                        Ok(lexical::WorkbenchLexicalSourceTotal {
                            source,
                            records: *count,
                            associated_assertions: union_by_source.get(id).map_or(0, BTreeSet::len),
                        })
                    })
                    .collect::<Result<Vec<_>, lexical::WorkbenchLexicalError>>()?;
                let totals = lexical::WorkbenchLexicalTotals {
                    unit: "stored_forms".into(),
                    records: candidates.len(),
                    source_groups: counts.len(),
                    associated_assertions: union_by_source.values().map(BTreeSet::len).sum(),
                };
                let needed_computations = candidates
                    .iter()
                    .filter_map(|candidate| {
                        candidate.form.computation.as_ref().map(ToString::to_string)
                    })
                    .collect::<BTreeSet<_>>();
                let computations = self
                    .document
                    .metadata
                    .computations
                    .iter()
                    .filter(|row| needed_computations.contains(&row.id.to_string()))
                    .map(|row| (row.id.to_string(), row))
                    .collect::<BTreeMap<_, _>>();
                let groups = page
                    .into_iter()
                    .map(|(id, indices)| {
                        let source = self
                            .availability
                            .iter()
                            .find(|row| row.source.id == id)
                            .map(|row| row.source.clone())
                            .ok_or_else(|| lexical::WorkbenchLexicalError::Integrity {
                                detail: format!("matching source {id} is missing"),
                            })?;
                        let mut hits = Vec::new();
                        for index in &indices {
                            let candidate = &candidates[*index];
                            let form = candidate.form;
                            let computation = form
                                .computation
                                .as_ref()
                                .map(|id| {
                                    computations.get(&id.to_string()).copied().ok_or_else(|| {
                                        lexical::WorkbenchLexicalError::Integrity {
                                            detail: format!(
                                                "Form {} references missing Computation {id}",
                                                form.id
                                            ),
                                        }
                                    })
                                })
                                .transpose()?;
                            let detail_ref = lexical::make_form_detail_ref(
                                binding_sha256,
                                &candidate.entry.source,
                                candidate.entry,
                                form,
                                computation,
                            )?;
                            let associated_assertions = associations
                                .get(&candidate.entry.id.to_string())
                                .into_iter()
                                .flatten()
                                .map(|assertion| self.compact_hit(candidate.entry, assertion, None))
                                .collect::<Result<Vec<_>, _>>()?;
                            hits.push(lexical::WorkbenchCompactFormRecord {
                                form_id: form.id.clone(),
                                entry_id: candidate.entry.id.clone(),
                                source_id: candidate.entry.source.clone(),
                                entry_spelling: candidate.entry.spelling.clone(),
                                spelling: form.spelling.clone(),
                                kind: form.kind,
                                mapped_role: lexical::mapped_role(form.kind),
                                evidence: form.evidence.clone(),
                                computation_id: form.computation.clone(),
                                association_status: if associated_assertions.is_empty() {
                                    "no_current_eligible_assertion"
                                } else {
                                    "eligible_assertions"
                                }
                                .into(),
                                associated_assertions,
                                detail_ref,
                            });
                        }
                        let full_count = counts[&id];
                        let returned_count = hits.len();
                        Ok(lexical::WorkbenchFormCompactGroup {
                            source,
                            full_count,
                            returned_count,
                            partial: returned_count < full_count,
                            hits,
                        })
                    })
                    .collect::<Result<Vec<_>, lexical::WorkbenchLexicalError>>()?;
                let next_cursor = if end < candidates.len() && end > start {
                    let candidate = &candidates[end - 1];
                    Some(lexical::encode_cursor(
                        request,
                        &resolved_sources,
                        "compact",
                        binding_sha256,
                        lexical::WorkbenchLexicalPosition::Form {
                            source: candidate.entry.source.clone(),
                            entry_spelling: candidate.entry_spelling.clone(),
                            spelling: candidate.form.spelling.clone(),
                            kind: candidate.form.kind,
                            form_id: candidate.form.id.clone(),
                        },
                    )?)
                } else {
                    None
                };
                Ok(lexical::WorkbenchLexicalCompactResult::Form(Box::new(
                    lexical::WorkbenchFormCompact {
                        policy: "stored-form-kind-v1".into(),
                        state_sha256: self.state_sha256.clone(),
                        query: request.query.clone(),
                        resolved_sources,
                        totals,
                        source_totals,
                        availability: self.availability.clone(),
                        groups,
                        next_cursor,
                        code_notices: self.document.metadata.code_notices.clone(),
                        snapshot_sha256: self.snapshot_sha256.clone(),
                    },
                )))
            }
        }
    }

    pub(crate) fn form_detail(
        &self,
        reference: &crate::workbench::lexical::WorkbenchFormDetailRef,
        binding_sha256: &str,
    ) -> Result<
        (
            crate::workbench::lexical::WorkbenchFormDetailRef,
            crate::workbench::lexical::WorkbenchFormRecord,
        ),
        crate::workbench::lexical::WorkbenchLexicalError,
    > {
        use crate::workbench::lexical::{self as lexical, WorkbenchLexicalError};

        let form = self
            .document
            .metadata
            .forms
            .iter()
            .find(|row| row.id == reference.form_id)
            .ok_or(WorkbenchLexicalError::StaleFormDetail)?;
        let entry = self
            .document
            .metadata
            .entries
            .iter()
            .find(|row| row.id == form.entry)
            .ok_or_else(|| WorkbenchLexicalError::Integrity {
                detail: format!("Form {} references a missing Entry", form.id),
            })?;
        let computation = form
            .computation
            .as_ref()
            .map(|id| {
                self.document
                    .metadata
                    .computations
                    .iter()
                    .find(|row| row.id == *id)
                    .ok_or_else(|| WorkbenchLexicalError::Integrity {
                        detail: format!("Form {} references missing Computation {id}", form.id),
                    })
            })
            .transpose()?;
        let evidence = form
            .evidence
            .as_ref()
            .map(
                |evidence| -> Result<lexical::WorkbenchFormEvidence, WorkbenchLexicalError> {
                    let capture_index = self.indices.captures.get(&evidence.occurrence.capture);
                    let snapshot_key = super::snapshot_key_for_ref(&evidence.occurrence.capture);
                    let snapshot = self
                        .indices
                        .snapshots
                        .get(&snapshot_key)
                        .map(|index| &self.document.registry.snapshots[*index]);
                    let occurrence_key = super::tuple_hex(&evidence.occurrence);
                    let occurrence = self
                        .indices
                        .occurrence_attributions
                        .get(&occurrence_key)
                        .map(|index| &self.document.registry.occurrences[*index]);
                    if snapshot.is_some_and(|row| row.source != entry.source) {
                        return Err(WorkbenchLexicalError::Integrity {
                            detail: "Form evidence snapshot belongs to another source".into(),
                        });
                    }
                    if occurrence.is_some_and(|row| {
                        super::record_key(&row.occurrence) != super::tuple_hex(&evidence.occurrence)
                    }) {
                        return Err(WorkbenchLexicalError::Integrity {
                            detail: "Form evidence occurrence attribution names another occurrence"
                                .into(),
                        });
                    }
                    let contributor = occurrence
                        .and_then(|row| row.contributor.as_ref())
                        .map(|id| {
                            self.indices
                                .contributors
                                .get(&id.to_string())
                                .map(|index| self.document.registry.contributors[*index].clone())
                                .ok_or_else(|| WorkbenchLexicalError::Integrity {
                                    detail: format!("Form evidence names missing Contributor {id}"),
                                })
                        })
                        .transpose()?;
                    let distributor = snapshot
                        .and_then(|row| row.distributor.as_ref())
                        .map(|id| {
                            self.indices
                                .distributors
                                .get(&id.to_string())
                                .map(|index| self.document.registry.distributors[*index].clone())
                                .ok_or_else(|| WorkbenchLexicalError::Integrity {
                                    detail: format!("Form evidence names missing Distributor {id}"),
                                })
                        })
                        .transpose()?;
                    let known_unavailable_owner =
                        self.document.metadata.releases.iter().any(|release| {
                            release.availability != Availability::Available
                                && release.captures.contains(&evidence.occurrence.capture)
                                && self.document.metadata.components.iter().any(|component| {
                                    component.id == release.component
                                        && self.document.metadata.collections.iter().any(
                                            |collection| {
                                                collection.id == component.collection
                                                    && collection.source == entry.source
                                            },
                                        )
                                })
                        });
                    if let Some(index) = capture_index {
                        let capture = &self.document.captures[*index];
                        let raw = capture
                            .occurrences
                            .iter()
                            .find(|row| row.locator == evidence.occurrence.locator)
                            .cloned()
                            .ok_or_else(|| WorkbenchLexicalError::Integrity {
                                detail: "loaded Form evidence locator is missing".into(),
                            })?;
                        let snapshot =
                            snapshot
                                .cloned()
                                .ok_or_else(|| WorkbenchLexicalError::Integrity {
                                    detail: "loaded Form evidence has no snapshot attribution"
                                        .into(),
                                })?;
                        Ok(lexical::WorkbenchFormEvidence {
                            evidence: evidence.clone(),
                            status: "Loaded".into(),
                            reason: None,
                            capture: Some(capture.metadata.clone()),
                            raw: Some(raw),
                            snapshot_attribution: Some(snapshot),
                            occurrence_attribution: occurrence.cloned(),
                            contributor,
                            distributor,
                        })
                    } else {
                        if !known_unavailable_owner {
                            return Err(WorkbenchLexicalError::Integrity {
                                detail:
                                    "catalogue-only Form evidence has no owning retained release"
                                        .into(),
                            });
                        }
                        Ok(lexical::WorkbenchFormEvidence {
                            evidence: evidence.clone(),
                            status: "CatalogueOnly".into(),
                            reason: Some("missing_capture".into()),
                            capture: None,
                            raw: None,
                            snapshot_attribution: snapshot.cloned(),
                            occurrence_attribution: occurrence.cloned(),
                            contributor,
                            distributor,
                        })
                    }
                },
            )
            .transpose()?;
        let source_availability = self
            .availability
            .iter()
            .find(|row| row.source.id == entry.source)
            .ok_or_else(|| WorkbenchLexicalError::Integrity {
                detail: "Form source is absent from compact availability".into(),
            })?;
        let observed =
            lexical::make_form_detail_ref(binding_sha256, &entry.source, entry, form, computation)?;
        if form.entry != reference.entry_id
            || entry.source != reference.source
            || &observed != reference
        {
            return Err(WorkbenchLexicalError::StaleFormDetail);
        }
        if !source_availability.selectable {
            return Err(WorkbenchLexicalError::Dictionary(Box::new(
                crate::DictionaryQueryError::UnavailableSource {
                    source: entry.source.clone(),
                    reason: source_availability.reason.ok_or_else(|| {
                        WorkbenchLexicalError::Integrity {
                            detail: "unselectable Form source has no reason".into(),
                        }
                    })?,
                },
            )));
        }
        let mut associated_assertions = Vec::new();
        let mut matching_assertions = self
            .document
            .metadata
            .assertions
            .iter()
            .filter(|row| {
                row.entry == entry.id
                    && matches!(row.eligibility, lexicon::Eligibility::Lexical)
                    && self
                        .selected
                        .get(&entry.source)
                        .is_some_and(|ids| ids.contains(row.id.as_str()))
            })
            .collect::<Vec<_>>();
        matching_assertions.sort_by(|left, right| {
            (super::tuple_hex(&left.occurrence), &left.id)
                .cmp(&(super::tuple_hex(&right.occurrence), &right.id))
        });
        for assertion in matching_assertions {
            associated_assertions.push(self.complete_hit(entry, assertion)?);
        }
        Ok((
            reference.clone(),
            lexical::WorkbenchFormRecord {
                form: form.clone(),
                entry: entry.clone(),
                source: source_availability.source.clone(),
                mapped_role: lexical::mapped_role(form.kind),
                computation: computation.cloned(),
                evidence,
                association_status: if associated_assertions.is_empty() {
                    "no_current_eligible_assertion"
                } else {
                    "eligible_assertions"
                }
                .into(),
                associated_assertions,
            },
        ))
    }

    /// Execute an exact or reference query and return compact output.
    pub fn execute(
        &self,
        request: DictionaryRequest,
    ) -> Result<DictionaryCompactQueryResult, DictionaryQueryError> {
        self.execute_inner(request, None)
    }
    /// Execute a compact query and record same-call timing.
    pub fn execute_profiled(
        &self,
        request: DictionaryRequest,
    ) -> Result<(DictionaryCompactQueryResult, DictionaryProjectionProfile), DictionaryQueryError>
    {
        let started = std::time::Instant::now();
        let mut profile = DictionaryProjectionProfile::default();
        let result = self.execute_inner(request, Some(&mut profile))?;
        profile.total = started.elapsed();
        profile.lookup = profile.total.saturating_sub(profile.projection);
        Ok((result, profile))
    }

    fn execute_inner(
        &self,
        request: DictionaryRequest,
        mut profile: Option<&mut DictionaryProjectionProfile>,
    ) -> Result<DictionaryCompactQueryResult, DictionaryQueryError> {
        match request {
            DictionaryRequest::Exact { spelling, options } => {
                let response = self.exact_inner(&spelling, options, profile.as_deref_mut())?;
                Ok(measure_projection(&mut profile, || {
                    DictionaryCompactQueryResult::Exact(Box::new(response))
                }))
            }
            DictionaryRequest::Resolve { target } => {
                let response = self.resolve_inner(&target, profile.as_deref_mut())?;
                Ok(measure_projection(&mut profile, || {
                    DictionaryCompactQueryResult::Resolve(Box::new(response))
                }))
            }
        }
    }
    /// Query byte-exact spelling matches with compact hit bodies.
    pub fn exact(
        &self,
        spelling: &str,
        options: DictionaryExactOptions,
    ) -> Result<DictionaryCompactExactResponse, DictionaryQueryError> {
        self.exact_inner(spelling, options, None)
    }

    fn exact_inner(
        &self,
        spelling: &str,
        options: DictionaryExactOptions,
        mut profile: Option<&mut DictionaryProjectionProfile>,
    ) -> Result<DictionaryCompactExactResponse, DictionaryQueryError> {
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
        let sql = profile.as_ref().map(|_| std::time::Instant::now());
        let candidate_ids = self.provider.exact_assertion_ids(spelling)?;
        let candidate_sql = sql.map_or(Duration::ZERO, |started| started.elapsed());
        if let Some(profile) = profile.as_deref_mut() {
            profile.candidate_sql += candidate_sql;
        }
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
        let total = rows.len();
        let mut offset = 0usize;
        if let Some(cursor) = options.cursor.as_deref() {
            offset = cursor_position(cursor, &cursor_binding, spelling, &rows, &self.document)?;
        }
        let page_end = options
            .page_size
            .map_or(total, |size| offset.saturating_add(size).min(total));
        let selected_rows = &rows[offset.min(total)..page_end];
        let source_counts: Vec<_> = resolved_sources
            .iter()
            .map(|source| {
                let count = rows
                    .iter()
                    .filter(|(entry_index, _)| {
                        self.document.metadata.entries[*entry_index].source == *source
                    })
                    .count();
                (source, count)
            })
            .filter(|(_, count)| *count != 0)
            .collect();

        let mut source_totals = Vec::new();
        let mut groups = Vec::new();
        let mut by_source: BTreeMap<SourceId, Vec<DictionaryCompactHit>> = BTreeMap::new();
        for (entry_index, assertion_index) in selected_rows {
            let entry = &self.document.metadata.entries[*entry_index];
            let assertion = &self.document.metadata.assertions[*assertion_index];
            let source = &entry.source;
            let hit = self.compact_hit(entry, assertion, profile.as_deref_mut())?;
            measure_projection(&mut profile, || {
                by_source.entry(source.clone()).or_default().push(hit);
            });
        }
        for (source, count) in source_counts {
            let Some(source_index) = self.indices.sources.get(source) else {
                return Err(DictionaryQueryError::Integrity {
                    detail: format!("source {source} missing from registry"),
                });
            };
            let record = &self.document.registry.sources[*source_index];
            let (total_record, group_record, hits) = measure_projection(&mut profile, || {
                let total_record = record.clone();
                let group_record = record.clone();
                let hits = by_source.remove(source).unwrap_or_default();
                (total_record, group_record, hits)
            });
            source_totals.push(DictionarySourceTotal {
                source: total_record,
                occurrences: count,
                distinct_spellings: 1,
            });
            if !hits.is_empty() {
                measure_projection(&mut profile, || {
                    groups.push(DictionaryCompactGroup {
                        source: group_record,
                        full_count: count,
                        returned_count: hits.len(),
                        partial: hits.len() < count,
                        hits,
                    });
                });
            }
        }
        let next_cursor = measure_projection(&mut profile, || {
            if page_end < total {
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
            }
        });
        let totals = DictionaryTotals {
            occurrences: total,
            distinct_spellings: usize::from(total > 0),
            source_groups: source_totals.len(),
        };
        Ok(measure_projection(&mut profile, || {
            DictionaryCompactExactResponse {
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
            }
        }))
    }
    /// Resolve a local or catalogue target without loading full hit bodies.
    pub fn resolve(
        &self,
        target: &RelationTarget,
    ) -> Result<DictionaryCompactReferenceResolution, DictionaryQueryError> {
        self.resolve_inner(target, None)
    }

    fn resolve_inner(
        &self,
        target: &RelationTarget,
        mut profile: Option<&mut DictionaryProjectionProfile>,
    ) -> Result<DictionaryCompactReferenceResolution, DictionaryQueryError> {
        let availability = measure_projection(&mut profile, || self.availability.clone());
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
                let (entity, rows) = self.resolve_local(local, profile.as_deref_mut())?;
                (
                    super::ReferenceStatus::Resolved,
                    Some(entity),
                    rows,
                    Vec::new(),
                    None,
                )
            }
            RelationTarget::Catalog(catalog) => {
                self.resolve_catalog(catalog, profile.as_deref_mut())?
            }
        };
        Ok(measure_projection(&mut profile, || {
            DictionaryCompactReferenceResolution {
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
            }
        }))
    }

    fn resolve_local(
        &self,
        local: &lexicon::LocalRef,
        mut profile: Option<&mut DictionaryProjectionProfile>,
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
                .indices
                .entries
                .get(&assertion.entry.to_string())
                .map(|index| &self.document.metadata.entries[*index].source);
            source
                .and_then(|source| self.selected.get(source))
                .is_some_and(|ids| ids.contains(&assertion.id.to_string()))
        };
        let (role, id, entry_ids, assertion_ids, spelling) = match local {
            LocalRef::Entry(id) => {
                let entry_index = self.indices.entries.get(&id.to_string()).ok_or_else(|| {
                    super::invalid_ref("LocalRef does not exist in its specific role table")
                })?;
                let entry = &self.document.metadata.entries[*entry_index];
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
                    Some(&entry.spelling),
                )
            }
            LocalRef::Assertion(id) => {
                let assertion_index =
                    self.indices
                        .assertions
                        .get(&id.to_string())
                        .ok_or_else(|| {
                            super::invalid_ref("LocalRef does not exist in its specific role table")
                        })?;
                let assertion = &self.document.metadata.assertions[*assertion_index];
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
                let form_index = self.indices.forms.get(&id.to_string()).ok_or_else(|| {
                    super::invalid_ref("LocalRef does not exist in its specific role table")
                })?;
                let form = &self.document.metadata.forms[*form_index];
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
                let sense_index = self
                    .indices
                    .editorial_senses
                    .get(&id.to_string())
                    .ok_or_else(|| {
                        super::invalid_ref("LocalRef does not exist in its specific role table")
                    })?;
                let sense = &self.document.metadata.editorial_senses[*sense_index];
                let entries = sense
                    .assertions
                    .iter()
                    .filter_map(|assertion| {
                        self.indices
                            .assertions
                            .get(&assertion.to_string())
                            .map(|index| &self.document.metadata.assertions[*index])
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
                            self.indices
                                .assertions
                                .get(&id.to_string())
                                .is_some_and(|index| {
                                    assertion_selected(&self.document.metadata.assertions[*index])
                                })
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
                self.indices
                    .entries
                    .get(entry)
                    .map(|index| &self.document.metadata.entries[*index])
                    .map(|row| row.source.to_string())
            })
            .collect();
        let occurrence = match local {
            LocalRef::Assertion(id) => self
                .indices
                .assertions
                .get(&id.to_string())
                .map(|index| &self.document.metadata.assertions[*index].occurrence),
            _ => None,
        };
        let entity =
            measure_projection(&mut profile, || vlajmaji_store::CollectionEntityReference {
                role: role.to_owned(),
                id: id.to_string(),
                entry_ids,
                assertion_ids: assertion_ids.clone(),
                source_ids,
                occurrence: occurrence.cloned(),
                evidence: None,
                spelling: spelling.cloned(),
            });
        let mut views = Vec::with_capacity(assertion_ids.len());
        for id in &assertion_ids {
            let Some(index) = self.indices.assertions.get(id) else {
                continue;
            };
            let row = &self.document.metadata.assertions[*index];
            views.push(self.reference_assertion(row, profile.as_deref_mut())?);
        }
        Ok((entity, views))
    }

    fn reference_assertion(
        &self,
        assertion: &lexicon::AssertionRecord,
        mut profile: Option<&mut DictionaryProjectionProfile>,
    ) -> Result<DictionaryCompactReferenceAssertion, DictionaryQueryError> {
        let entry_index = self
            .indices
            .entries
            .get(&assertion.entry.to_string())
            .ok_or_else(|| DictionaryQueryError::Integrity {
                detail: "reference assertion has no entry".into(),
            })?;
        let entry = &self.document.metadata.entries[*entry_index];
        let occurrence_id = super::tuple_hex(&assertion.occurrence);
        let occurrence_attribution = self
            .indices
            .occurrence_attributions
            .get(&occurrence_id)
            .map(|index| &self.document.registry.occurrences[*index]);
        let capture = self
            .indices
            .captures
            .get(&assertion.occurrence.capture)
            .map(|index| &self.document.captures[*index]);
        let snapshot_key = super::snapshot_key_for_ref(&assertion.occurrence.capture);
        let snapshot_attribution = self
            .indices
            .snapshots
            .get(&snapshot_key)
            .map(|index| &self.document.registry.snapshots[*index]);
        let current_eligible = matches!(assertion.eligibility, lexicon::Eligibility::Lexical)
            && self
                .selected
                .get(&entry.source)
                .is_some_and(|ids| ids.contains(&assertion.id.to_string()));
        let release_records: Vec<_> = self
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
            .collect();
        let owned_revision_records: Vec<_> = self
            .document
            .metadata
            .owned_revisions
            .iter()
            .filter(|row| row.assertions.contains(&assertion.id))
            .collect();
        let tombstone_records: Vec<_> = self
            .document
            .metadata
            .owned_revisions
            .iter()
            .filter(|row| row.tombstones.contains(&assertion.id))
            .collect();
        let detail_ref = if capture.is_some() {
            let (entry_sha256, assertion_sha256) = measure_projection(&mut profile, || {
                (digest_for_query(entry), digest_for_query(assertion))
            });
            let entry_sha256 = entry_sha256?;
            let assertion_sha256 = assertion_sha256?;
            Some(measure_projection(&mut profile, || {
                DictionaryHitDetailRef {
                    version: 1,
                    digest_policy: DETAIL_POLICY.into(),
                    role: "assertion".into(),
                    source: entry.source.clone(),
                    entry_id: entry.id.clone(),
                    assertion_id: assertion.id.clone(),
                    occurrence: assertion.occurrence.clone(),
                    entry_sha256,
                    assertion_sha256,
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
                }
            }))
        } else {
            None
        };
        let payload_present = self.indices.occurrences.contains_key(&(
            assertion.occurrence.capture.clone(),
            assertion.occurrence.locator.clone(),
        ));
        let projected = measure_projection(&mut profile, || DictionaryCompactReferenceAssertion {
            entry: entry.clone(),
            assertion: assertion.clone(),
            occurrence_id,
            occurrence_attribution: occurrence_attribution.cloned(),
            capture_metadata: capture.map(|row| row.metadata.clone()),
            snapshot_attribution: snapshot_attribution.cloned(),
            payload_present,
            current_eligible,
            release_ids: release_records.iter().map(|row| row.id.clone()).collect(),
            owned_revision_ids: owned_revision_records
                .iter()
                .map(|row| row.id.clone())
                .collect(),
            tombstoned_by: tombstone_records.iter().map(|row| row.id.clone()).collect(),
            detail_ref,
        });
        Ok(projected)
    }

    fn resolve_catalog(
        &self,
        catalog: &lexicon::CatalogTarget,
        mut profile: Option<&mut DictionaryProjectionProfile>,
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
            let Some(assertion_index) = self.indices.assertions.get(id.as_str()) else {
                continue;
            };
            let assertion = &metadata.assertions[*assertion_index];
            if catalog
                .occurrence
                .as_ref()
                .is_some_and(|occurrence| occurrence != &assertion.occurrence)
            {
                continue;
            }
            let Some(entry_index) = self.indices.entries.get(assertion.entry.as_str()) else {
                continue;
            };
            let entry = &metadata.entries[*entry_index];
            if entry.source != catalog.source {
                continue;
            }
            let attribution_key = super::tuple_hex(&assertion.occurrence);
            let attribution = self
                .indices
                .occurrence_attributions
                .get(&attribution_key)
                .map(|index| &self.document.registry.occurrences[*index]);
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
                unverified_views.push(self.reference_assertion(assertion, profile.as_deref_mut())?);
                continue;
            }
            confirmed.push(self.reference_assertion(assertion, profile.as_deref_mut())?);
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
        let requested_release = self
            .availability
            .iter()
            .flat_map(|row| &row.releases)
            .find(|row| row.id == release.id.to_string());
        let requested_release = measure_projection(&mut profile, || requested_release.cloned());
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
        mut profile: Option<&mut DictionaryProjectionProfile>,
    ) -> Result<DictionaryCompactHit, DictionaryQueryError> {
        #[cfg(test)]
        COMPACT_HIT_PROJECTIONS.with(|count| count.set(count.get() + 1));

        let capture_index = self
            .indices
            .captures
            .get(&assertion.occurrence.capture)
            .ok_or_else(|| DictionaryQueryError::Integrity {
                detail: format!("selected assertion {} has no capture", assertion.id),
            })?;
        let capture = &self.document.captures[*capture_index];
        let snapshot_key = super::snapshot_key_for_ref(&assertion.occurrence.capture);
        let snapshot_index = self.indices.snapshots.get(&snapshot_key).ok_or_else(|| {
            DictionaryQueryError::Integrity {
                detail: "loaded capture has no snapshot attribution".into(),
            }
        })?;
        let snapshot_attribution = measure_projection(&mut profile, || {
            self.document.registry.snapshots[*snapshot_index].clone()
        });
        let occurrence_key = super::tuple_hex(&assertion.occurrence);
        let occurrence_index = self.indices.occurrence_attributions.get(&occurrence_key);
        let occurrence_id = occurrence_key;
        let occurrence_attribution = occurrence_index.map(|index| {
            measure_projection(&mut profile, || {
                self.document.registry.occurrences[*index].clone()
            })
        });
        let contributor_record = occurrence_attribution
            .as_ref()
            .and_then(|row| row.contributor.as_ref())
            .and_then(|id| self.indices.contributors.get(&id.to_string()))
            .map(|index| &self.document.registry.contributors[*index]);
        let contributor = measure_projection(&mut profile, || contributor_record.cloned());
        let distributor_record = snapshot_attribution
            .distributor
            .as_ref()
            .and_then(|id| self.indices.distributors.get(&id.to_string()))
            .map(|index| &self.document.registry.distributors[*index]);
        let distributor = measure_projection(&mut profile, || distributor_record.cloned());
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
        let release_records: Vec<_> = self
            .document
            .metadata
            .releases
            .iter()
            .filter(|release| {
                selected.contains(&release.id) && release.assertions.contains(&assertion.id)
            })
            .collect();
        let release_ids = measure_projection(&mut profile, || {
            release_records
                .iter()
                .map(|release| release.id.clone())
                .collect()
        });
        let (entry_sha256, assertion_sha256) = measure_projection(&mut profile, || {
            (digest_for_query(entry), digest_for_query(assertion))
        });
        let entry_sha256 = entry_sha256?;
        let assertion_sha256 = assertion_sha256?;
        let detail_ref = measure_projection(&mut profile, || DictionaryHitDetailRef {
            version: 1,
            digest_policy: DETAIL_POLICY.into(),
            role: "assertion".into(),
            source: entry.source.clone(),
            entry_id: entry.id.clone(),
            assertion_id: assertion.id.clone(),
            occurrence: assertion.occurrence.clone(),
            entry_sha256,
            assertion_sha256,
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
        });
        Ok(measure_projection(&mut profile, || DictionaryCompactHit {
            occurrence_id,
            entry: entry.clone(),
            assertion: assertion.clone(),
            capture: capture.metadata.clone(),
            snapshot_attribution,
            occurrence_attribution,
            contributor,
            distributor,
            release_ids,
            detail_ref,
        }))
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

    #[test]
    fn test_compact_form_detail_reports_missing_related_computation_as_integrity() {
        let root = scratch_exclusive();
        let store_path = root.join("dictionary.sqlite3");
        let source = lexicon::SourceId::new("projection-source").unwrap();
        let entry_id = EntityId::new("projection-entry").unwrap();
        let form_id = EntityId::new("projection-form").unwrap();
        let computation_id = EntityId::new("projection-computation").unwrap();
        let collection_id = EntityId::new("projection-collection").unwrap();
        let component_id = EntityId::new("projection-component").unwrap();
        let release_id = EntityId::new("projection-release").unwrap();
        let expected_form = lexicon::FormRecord {
            id: form_id.clone(),
            entry: entry_id.clone(),
            spelling: "computed-form".into(),
            language: Presence::Missing,
            script: Presence::Missing,
            pronunciation: Presence::Missing,
            kind: lexicon::FormKind::ComputedStem,
            standing: Presence::Missing,
            components: Vec::new(),
            evidence: None,
            computation: Some(computation_id.clone()),
        };
        let expected_computation = lexicon::ComputationRecord {
            id: computation_id,
            tool: "projection-test".into(),
            tool_version: "1".into(),
            inputs: vec![lexicon::InputRef::Literal("fixed input".into())],
            policy: "projection-test-v1".into(),
            profile: Presence::Missing,
            explanation: Presence::Missing,
        };
        let registry = lexicon::SourceRegistry::from_document(lexicon::RegistryDocument {
            version: 1,
            sources: vec![lexicon::SourceRecord {
                id: source.clone(),
                display_name: "Projection test source".into(),
                abbreviation: "PTS".into(),
                abbreviation_kind: lexicon::AbbreviationKind::ProjectAssigned,
                abbreviation_evidence: Some("projection unit test".into()),
                origin: lexicon::SourceOrigin::Upstream,
            }],
            ..lexicon::RegistryDocument::default()
        })
        .unwrap();
        let dictionary = lexicon::Dictionary::from_document(lexicon::DictionaryDocument {
            registry,
            metadata: lexicon::DictionaryMetadata {
                collections: vec![lexicon::CollectionRecord {
                    id: collection_id.clone(),
                    source: source.clone(),
                    display_name: "Projection test collection".into(),
                    description: Presence::Missing,
                }],
                components: vec![lexicon::ComponentRecord {
                    id: component_id.clone(),
                    collection: collection_id,
                    native_name: "projection".into(),
                    languages: Vec::new(),
                }],
                releases: vec![lexicon::ReleaseRecord {
                    id: release_id,
                    component: component_id,
                    distributor: None,
                    upstream_revision: Presence::Missing,
                    distributor_revision: Presence::Missing,
                    captured_at: Presence::Missing,
                    time_precision: lexicon::TimePrecision::Unknown,
                    schema_version: 1,
                    normalizer: "projection-test".into(),
                    normalizer_version: "1".into(),
                    captures: Vec::new(),
                    assertions: Vec::new(),
                    accounting: lexicon::ReleaseAccounting {
                        input_occurrences: Some(0),
                        normalized_occurrences: Some(0),
                        query_excluded_occurrences: Some(0),
                        unit: "assertions".into(),
                    },
                    diagnostics: Vec::new(),
                    selected: true,
                    availability: lexicon::Availability::Available,
                    content_notices: Vec::new(),
                    redistribution: lexicon::RedistributionStatus::Unknown,
                    redistribution_evidence: Vec::new(),
                }],
                entries: vec![lexicon::EntryRecord {
                    id: entry_id,
                    source,
                    spelling: Presence::Value("entry".into()),
                    language: Presence::Missing,
                    script: Presence::Missing,
                    native_word_class: Presence::Missing,
                    selmaho: Presence::Missing,
                    mapped_pos: Presence::Missing,
                }],
                forms: vec![expected_form.clone()],
                computations: vec![expected_computation.clone()],
                ..lexicon::DictionaryMetadata::default()
            },
            ..lexicon::DictionaryDocument::default()
        })
        .unwrap();
        let store = DictionaryStore::create(&store_path, &dictionary).unwrap();
        drop(store);
        let mut engine = DictionaryCompactSearchEngine::open(&store_path).unwrap();
        let binding = "a".repeat(64);
        assert!(
            engine
                .availability
                .iter()
                .any(|row| row.source.id.as_str() == "projection-source" && row.selectable)
        );
        let request = crate::workbench::lexical::WorkbenchLexicalRequest {
            version: 1,
            query: crate::workbench::lexical::WorkbenchLexicalQuery::Form {
                value: expected_form.spelling.clone(),
                role: None,
                kind: Some(expected_form.kind),
            },
            options: crate::workbench::lexical::WorkbenchLexicalOptions::default(),
        };
        let crate::workbench::lexical::WorkbenchLexicalCompactResult::Form(forms) =
            engine.lexical_compact(&request, &binding).unwrap()
        else {
            panic!("computed-stem request returned another operation")
        };
        assert_eq!(forms.totals.records, 1);
        let detail_ref = forms.groups[0].hits[0].detail_ref.clone();
        assert_eq!(forms.groups[0].hits[0].form_id, expected_form.id);

        let baseline_document = engine.document.clone();
        assert_eq!(
            engine.form_detail(&detail_ref, &binding).unwrap().1.form,
            expected_form
        );
        // This deliberately corrupted cached snapshot is UNVERIFIED; restore the
        // actual store-loaded baseline before the same-key positive check.
        engine.document.metadata.computations.clear();
        let mut forged_secondary_key = detail_ref.clone();
        forged_secondary_key.entry_id = EntityId::new("projection-other-entry").unwrap();
        let missing = engine
            .form_detail(&forged_secondary_key, &binding)
            .unwrap_err();
        assert!(matches!(
            missing,
            crate::workbench::lexical::WorkbenchLexicalError::Integrity { detail }
                if detail.contains("missing Computation")
        ));
        engine.document = baseline_document;
        let (_, restored) = engine.form_detail(&detail_ref, &binding).unwrap();
        assert_eq!(restored.form, expected_form);
        assert_eq!(restored.computation, Some(expected_computation));

        drop(engine);
        std::fs::remove_dir_all(root).unwrap();
    }

    fn scratch_exclusive() -> std::path::PathBuf {
        for _ in 0..64 {
            let root = std::env::temp_dir().join(format!(
                "vlajmaji-compact-projection-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            match std::fs::create_dir(&root) {
                Ok(()) => return root,
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => panic!("create exclusive projection test root: {error}"),
            }
        }
        panic!("could not allocate projection test root after 64 attempts")
    }

    #[test]
    fn compact_catalogue_ordinals_preserve_release_order_and_missing_link_skips() {
        let root = scratch();
        let store_path = root.join("dictionary.sqlite3");
        let dictionary = fixture::complete_fixture().unwrap();
        let store = DictionaryStore::create(&store_path, &dictionary).unwrap();
        drop(store);
        let mut engine = DictionaryCompactSearchEngine::open(&store_path).unwrap();

        let release_index = engine
            .document
            .metadata
            .releases
            .iter()
            .position(|release| {
                release.selected
                    && release.availability == lexicon::Availability::Available
                    && release.assertions.len() >= 2
            })
            .expect("fixture has a selected available release with two assertions");
        let release = &engine.document.metadata.releases[release_index];
        let first_id = release.assertions[0].clone();
        let second_id = release.assertions[1].clone();
        let first_index = engine
            .document
            .metadata
            .assertions
            .iter()
            .position(|assertion| assertion.id == first_id)
            .unwrap();
        let second_index = engine
            .document
            .metadata
            .assertions
            .iter()
            .position(|assertion| assertion.id == second_id)
            .unwrap();
        let occurrence = engine.document.metadata.assertions[first_index]
            .occurrence
            .clone();
        let source = engine
            .document
            .metadata
            .entries
            .iter()
            .find(|entry| entry.id == engine.document.metadata.assertions[first_index].entry)
            .unwrap()
            .source
            .clone();
        let common_entry = engine.document.metadata.assertions[first_index]
            .entry
            .clone();
        engine.document.metadata.assertions[second_index].entry = common_entry;
        engine.document.metadata.assertions[second_index].occurrence = occurrence.clone();
        engine.document.metadata.releases[release_index].assertions =
            vec![first_id.clone(), second_id.clone(), first_id.clone()];

        let target = lexicon::CatalogTarget {
            source,
            release: engine.document.metadata.releases[release_index].id.clone(),
            native_entry_id: None,
            native_assertion_id: None,
            occurrence: Some(occurrence),
        };
        let result = engine.resolve_catalog(&target, None).unwrap();
        assert_eq!(
            result
                .2
                .iter()
                .map(|row| row.assertion.id.clone())
                .collect::<Vec<_>>(),
            vec![first_id.clone(), second_id, first_id.clone()]
        );

        engine.document.metadata.releases[release_index].assertions = vec![
            lexicon::EntityId::new("missing-assertion").unwrap(),
            first_id.clone(),
        ];
        let missing_assertion = engine.resolve_catalog(&target, None).unwrap();
        assert_eq!(
            missing_assertion
                .2
                .iter()
                .map(|row| row.assertion.id.clone())
                .collect::<Vec<_>>(),
            vec![first_id.clone()],
            "a missing assertion ID is skipped in its declared release position"
        );

        engine.document.metadata.releases[release_index].assertions = vec![first_id.clone()];
        engine.document.metadata.assertions[first_index].entry =
            lexicon::EntityId::new("missing-entry").unwrap();
        let missing_entry = engine.resolve_catalog(&target, None).unwrap();
        assert!(missing_entry.2.is_empty());
        assert!(missing_entry.3.is_empty());
        assert_eq!(missing_entry.0, super::super::ReferenceStatus::NoMatch);

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

impl DictionaryCompactSearchEngine {
    /// Borrow exact typed role rows from the retained compact document before resolution.
    pub(crate) fn typed_reference_witnesses(
        &self,
        target: &RelationTarget,
    ) -> Result<Vec<crate::workbench::reference::TypedRoleWitness>, DictionaryQueryError> {
        use crate::workbench::reference::{
            assertion_witness, entry_witness, form_witness, sense_witness,
        };
        let metadata = &self.document.metadata;
        let mut result = Vec::new();
        macro_rules! add_assertion {
            ($row:expr) => {
                let entry_index = self
                    .indices
                    .entries
                    .get(&($row).entry.to_string())
                    .ok_or_else(|| DictionaryQueryError::Integrity {
                        detail: format!("assertion {} references missing entry", ($row).id),
                    })?;
                let entry = &metadata.entries[*entry_index];
                result.push(assertion_witness($row, entry)?);
                result.push(entry_witness(entry)?);
            };
        }
        match target {
            RelationTarget::Local(lexicon::LocalRef::Entry(id)) => {
                if let Some(entry_index) = self.indices.entries.get(&id.to_string()) {
                    let entry = &metadata.entries[*entry_index];
                    result.push(entry_witness(entry)?);
                }
            }
            RelationTarget::Local(lexicon::LocalRef::Assertion(id)) => {
                if let Some(row_index) = self.indices.assertions.get(&id.to_string()) {
                    let row = &metadata.assertions[*row_index];
                    add_assertion!(row);
                }
            }
            RelationTarget::Local(lexicon::LocalRef::Form(id)) => {
                if let Some(row_index) = self.indices.forms.get(&id.to_string()) {
                    let row = &metadata.forms[*row_index];
                    let entry_index = self
                        .indices
                        .entries
                        .get(&row.entry.to_string())
                        .ok_or_else(|| DictionaryQueryError::Integrity {
                            detail: format!("form {} references missing entry", row.id),
                        })?;
                    let entry = &metadata.entries[*entry_index];
                    result.push(form_witness(row, entry)?);
                    result.push(entry_witness(entry)?);
                }
            }
            RelationTarget::Local(lexicon::LocalRef::EditorialSense(id)) => {
                if let Some(row_index) = self.indices.editorial_senses.get(&id.to_string()) {
                    let row = &metadata.editorial_senses[*row_index];
                    let assertions: Vec<_> = row
                        .assertions
                        .iter()
                        .map(|id| {
                            self.indices
                                .assertions
                                .get(&id.to_string())
                                .map(|index| &metadata.assertions[*index])
                                .ok_or_else(|| DictionaryQueryError::Integrity {
                                    detail: format!(
                                        "editorial sense {} references missing assertion {id}",
                                        row.id
                                    ),
                                })
                        })
                        .collect::<Result<_, _>>()?;
                    let entries: Vec<_> = assertions
                        .iter()
                        .map(|assertion| {
                            self.indices.entries.get(&assertion.entry.to_string()).map(|index| {
                                &metadata.entries[*index]
                            }).ok_or_else(|| DictionaryQueryError::Integrity {
                                detail: format!("editorial sense {} references assertion with missing entry", row.id),
                            })
                        })
                        .collect::<Result<_, _>>()?;
                    let linked: Vec<_> = assertions.into_iter().zip(entries).collect();
                    result.push(sense_witness(row, &linked)?);
                }
            }
            RelationTarget::Catalog(wanted) => {
                if let Some(release) = metadata
                    .releases
                    .iter()
                    .find(|release| release.id == wanted.release)
                {
                    for id in &release.assertions {
                        let row_index =
                            self.indices
                                .assertions
                                .get(&id.to_string())
                                .ok_or_else(|| DictionaryQueryError::Integrity {
                                    detail: format!(
                                        "release {} references missing assertion {id}",
                                        wanted.release
                                    ),
                                })?;
                        let row = &metadata.assertions[*row_index];
                        if wanted
                            .occurrence
                            .as_ref()
                            .is_some_and(|occurrence| occurrence != &row.occurrence)
                        {
                            continue;
                        }
                        let entry_index = self
                            .indices
                            .entries
                            .get(&row.entry.to_string())
                            .ok_or_else(|| DictionaryQueryError::Integrity {
                                detail: format!("assertion {} references missing entry", row.id),
                            })?;
                        let entry = &metadata.entries[*entry_index];
                        if entry.source != wanted.source {
                            continue;
                        }
                        result.push(assertion_witness(row, entry)?);
                        result.push(entry_witness(entry)?);
                    }
                }
            }
            RelationTarget::Unknown | RelationTarget::Unresolved(_) => {}
        }
        Ok(result)
    }
}
