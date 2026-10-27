//! Grouped exact queries over typed drafts from persisted source dictionaries.
//!
//! This module keeps source identity, native occurrences, editorial material,
//! and computed annotations separate. Exact lookup is byte-exact and uses the
//! store's persisted lexical candidate query before applying catalogue policy.

use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fmt,
    path::Path,
    time::{Duration, Instant},
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use vlajmaji_lexicon as lexicon;
use vlajmaji_lexicon::{
    AliasScope, Availability, CaptureMetadata, CaptureRef, CatalogTarget, CodeNotice,
    CollectionRecord, ComponentRecord, ComputationRecord, ComputedAnnotation, ContextRecord,
    Dictionary, DictionaryDocument, DictionaryError, DistributorRecord, EditorialSenseRecord,
    Eligibility, EntityId, EntryRecord, EtymologyRecord, ExtensionRecord, FormRecord, LocalRef,
    OccurrenceAttribution, OccurrenceRef, OwnedHead, RawOccurrence, RegistryError, RelationRecord,
    RelationTarget, ReleaseRecord, SnapshotAttribution, SourceAlias, SourceId, SourceLineage,
    SourceRecord,
};
use vlajmaji_store::{
    DictionaryReadSnapshot, DictionaryReadTopology, DictionarySnapshotError,
    DictionarySnapshotManifest, DictionaryStore, DictionaryStoreError,
};

mod borrowed;
pub mod projection;
pub use projection::{
    DictionaryCompactExactResponse, DictionaryCompactGroup, DictionaryCompactHit,
    DictionaryCompactQueryResult, DictionaryCompactReferenceAssertion,
    DictionaryCompactReferenceResolution, DictionaryCompactSearchEngine,
    DictionaryCompactSourceAvailability, DictionaryDetailReferenceInvalidity,
    DictionaryHitDetailRef, DictionaryHitDetailResponse, DictionaryProjectionError,
    DictionaryProjectionProfile, validate_detail_snapshot_sha256,
};

/// Exact lookup policy included in every result and cursor binding.
pub const EXACT_POLICY: &str = "exact-entry-v1";
/// Selection policy included in every cursor binding.
pub const SELECTION_POLICY: &str = "selected-available-owned-head-v1";

/// A strict request accepted by the public dictionary search API.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case", deny_unknown_fields)]
pub enum DictionaryRequest {
    /// Find all eligible exact-spelling assertions under the requested options.
    Exact {
        /// Exact UTF-8 spelling bytes.
        spelling: String,
        /// Selection and page controls; defaults to all selectable sources.
        #[serde(default)]
        options: DictionaryExactOptions,
    },
    /// Resolve one stable local or catalogue relation target.
    Resolve {
        /// Target to validate and resolve against the retained catalogue.
        target: RelationTarget,
    },
}

/// Source and page controls for a byte-exact dictionary query.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DictionaryExactOptions {
    /// Exact IDs, abbreviations, or aliases. Empty selects every selectable source.
    #[serde(default)]
    pub sources: Vec<String>,
    /// Whether retired aliases may participate in explicit source resolution.
    #[serde(default = "active_aliases")]
    pub alias_scope: AliasScope,
    /// Maximum returned occurrences. `None` returns the complete result.
    #[serde(default)]
    pub page_size: Option<usize>,
    /// Opaque continuation returned by an earlier page.
    #[serde(default)]
    pub cursor: Option<String>,
}

impl Default for DictionaryExactOptions {
    fn default() -> Self {
        Self {
            sources: Vec::new(),
            alias_scope: AliasScope::ActiveOnly,
            page_size: None,
            cursor: None,
        }
    }
}

fn active_aliases() -> AliasScope {
    AliasScope::ActiveOnly
}

/// Typed source-selection failure for an otherwise registered source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceSelectionFailure {
    /// The source has no release records.
    NoReleases,
    /// No release for the source is currently available.
    NoSelectedAvailableRelease,
    /// The owned source does not declare a current head.
    NoOwnedHead,
}

/// Complete counts for an exact query, before cursor or page slicing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DictionaryTotals {
    /// Number of selected source occurrences.
    pub occurrences: usize,
    /// Exact queries have zero or one distinct spelling.
    pub distinct_spellings: usize,
    /// Number of sources with at least one selected occurrence.
    pub source_groups: usize,
}

/// Complete pre-page count for one matching source.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DictionarySourceTotal {
    /// Source identity and description.
    pub source: SourceRecord,
    /// Number of matching selected occurrences.
    pub occurrences: usize,
    /// Zero or one for exact spelling.
    pub distinct_spellings: usize,
}

/// One source group present on the current page.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DictionaryGroup {
    /// Source identity and description.
    pub source: SourceRecord,
    /// Full matching count for this source, before pagination.
    pub full_count: usize,
    /// Count returned on this page.
    pub returned_count: usize,
    /// Whether only part of the group is present on this page.
    pub partial: bool,
    /// Ordered occurrence-level hits returned on this page.
    pub hits: Vec<DictionaryHit>,
}

/// Catalogue state for one registered source, independent of query filters.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DictionarySourceAvailability {
    /// Registered source identity.
    pub source: SourceRecord,
    /// Aliases retained in registry order.
    pub aliases: Vec<SourceAlias>,
    /// Descriptive lineage retained in registry order.
    pub lineage: Vec<SourceLineage>,
    /// Collections owned by this source.
    pub collections: Vec<CollectionRecord>,
    /// Components in the source's collections.
    pub components: Vec<ComponentRecord>,
    /// All source releases in metadata order.
    pub releases: Vec<ReleaseRecord>,
    /// Available releases used by exact lookup.
    pub selected_available_release_ids: Vec<EntityId>,
    /// Distinct loaded captures, in first-seen release order.
    pub loaded_captures: Vec<CaptureRef>,
    /// Distinct catalogue captures without payload, in first-seen release order.
    pub missing_captures: Vec<CaptureRef>,
    /// Explicit current owned head, when present.
    pub owned_head: Option<OwnedHead>,
    /// Whether ordinary exact queries may select the source.
    pub selectable: bool,
    /// Why the source is not selectable.
    pub reason: Option<SourceSelectionFailure>,
}

/// One exact source occurrence, retaining typed source and editorial provenance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DictionaryHit {
    /// Lowercase hex encoding of the exact length-prefixed occurrence tuple; not a hash.
    pub occurrence_id: String,
    /// The source entry, including all field-presence states.
    pub entry: EntryRecord,
    /// The lexical source assertion.
    pub assertion: lexicon::AssertionRecord,
    /// Capture metadata for the immutable source bytes.
    pub capture: CaptureMetadata,
    /// Exact native row and original line, without duplicating capture bytes.
    pub raw: RawOccurrence,
    /// Source/distributor attribution for the loaded snapshot.
    pub snapshot_attribution: SnapshotAttribution,
    /// Optional contributor and native identities for this occurrence.
    pub occurrence_attribution: Option<OccurrenceAttribution>,
    /// Resolved contributor metadata, if attributed.
    pub contributor: Option<lexicon::ContributorRecord>,
    /// Resolved distributor metadata, if attributed.
    pub distributor: Option<DistributorRecord>,
    /// Selected available releases containing this assertion.
    pub release_ids: Vec<EntityId>,
    /// Assertion-owned contexts, in model order.
    pub contexts: Vec<ContextRecord>,
    /// Assertion-owned argument frames, in model order.
    pub frames: Vec<lexicon::ArgumentFrameRecord>,
    /// Entry-owned forms, in model order.
    pub forms: Vec<FormRecord>,
    /// Assertion-owned etymologies, in model order.
    pub etymologies: Vec<EtymologyRecord>,
    /// Relations whose origin is the represented local entity set.
    pub relations: Vec<RelationRecord>,
    /// Editorial senses explicitly containing this assertion.
    pub editorial_senses: Vec<EditorialSenseRecord>,
    /// Computations referenced by represented forms or annotations.
    pub computations: Vec<ComputationRecord>,
    /// Computed annotations whose target is represented by this hit.
    pub annotations: Vec<ComputedAnnotation>,
    /// Extensions owned by a represented entity or source collection/component/release.
    pub extensions: Vec<ExtensionRecord>,
    /// Owned revisions listing this assertion.
    pub owned_revision_ids: Vec<EntityId>,
    /// Owned revisions tombstoning this assertion.
    pub tombstoned_by: Vec<EntityId>,
}

/// Complete exact-query response, including full pre-page counts and availability.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DictionaryExactResponse {
    /// Response contract version.
    pub version: u32,
    /// Stable query-policy identity.
    pub policy: String,
    /// Persisted dictionary schema version.
    pub store_schema_version: u32,
    /// Catalogue revision from canonical model metadata.
    pub catalogue_revision: u64,
    /// Recorded canonical SHA-256 from the store header; not checked during fast open.
    pub state_sha256: String,
    /// Exact query spelling.
    pub spelling: String,
    /// Every selected selectable source, including sources with zero hits.
    pub resolved_sources: Vec<SourceId>,
    /// SQL lexical candidates before release/source/head selection.
    pub native_candidate_occurrences: usize,
    /// Full counts before page/cursor slicing.
    pub totals: DictionaryTotals,
    /// Full matching counts per source before page/cursor slicing.
    pub source_totals: Vec<DictionarySourceTotal>,
    /// All registered source availability, regardless of filter or page.
    pub availability: Vec<DictionarySourceAvailability>,
    /// Only the occurrence groups represented on this page.
    pub groups: Vec<DictionaryGroup>,
    /// Continuation cursor when another occurrence remains.
    pub next_cursor: Option<String>,
    /// Code notices retained by the canonical catalogue.
    pub code_notices: Vec<CodeNotice>,
}

/// A typed status for resolving one stable local or catalogue target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReferenceStatus {
    /// Every supplied catalogue identity matched known source attribution.
    Resolved,
    /// Candidate source records exist, but some supplied identity is unknown.
    Unverified,
    /// The catalogue release is known but no assessable payload is available.
    CatalogueOnly,
    /// Known identity mismatches yield this status regardless of release availability; an available release with no candidates also yields it.
    NoMatch,
    /// The request explicitly carries an unknown target.
    Unknown,
    /// The request explicitly carries an unresolved description.
    Unresolved,
}

/// Typed local entity selected by a valid LocalRef.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReferenceEntity {
    /// Entry identity.
    Entry(EntryRecord),
    /// Assertion identity.
    Assertion(lexicon::AssertionRecord),
    /// Form identity.
    Form(FormRecord),
    /// Editorial sense identity.
    EditorialSense(EditorialSenseRecord),
}

/// One source assertion considered for a catalogue target.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferenceAssertionView {
    /// Entry containing this assertion.
    pub entry: EntryRecord,
    /// Source assertion identity and content.
    pub assertion: lexicon::AssertionRecord,
    /// Exact occurrence tuple ID; not a hash.
    pub occurrence_id: String,
    /// Optional native attribution.
    pub occurrence_attribution: Option<OccurrenceAttribution>,
    /// Raw row when its capture payload is available.
    pub raw: Option<RawOccurrence>,
    /// Capture metadata when its payload is available.
    pub capture_metadata: Option<CaptureMetadata>,
    /// Whether exact capture and locator are present.
    pub payload_present: bool,
    /// Whether default exact lookup selects this assertion now.
    pub current_eligible: bool,
    /// All release memberships in metadata order.
    pub release_ids: Vec<EntityId>,
    /// Owned revisions that include this assertion.
    pub owned_revision_ids: Vec<EntityId>,
    /// Owned revisions that tombstone this assertion.
    pub tombstoned_by: Vec<EntityId>,
}

/// Stable typed resolution of a requested relation target against a loaded draft.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferenceResolution {
    /// Response contract version.
    pub version: u32,
    /// Recorded canonical SHA-256 from the store header; not checked during fast open.
    pub state_sha256: String,
    /// Exact requested target.
    pub target: RelationTarget,
    /// Resolution classification.
    pub status: ReferenceStatus,
    /// Corresponding typed local entity, if this was a LocalRef.
    pub entity: Option<ReferenceEntity>,
    /// Assertions confirmed against every supplied catalogue identity.
    pub confirmed: Vec<ReferenceAssertionView>,
    /// Assertions whose available attribution cannot confirm all supplied IDs.
    pub unverified: Vec<ReferenceAssertionView>,
    /// Complete catalogue availability.
    pub availability: Vec<DictionarySourceAvailability>,
    /// Requested catalogue release, including unavailable releases.
    pub requested_release: Option<ReleaseRecord>,
}

/// A stable machine-readable projection of a query failure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DictionaryErrorReport {
    /// Stable machine-readable failure category.
    pub code: String,
    /// Actionable diagnostic text.
    pub message: String,
    /// Relevant ambiguous or unavailable source IDs.
    pub sources: Vec<SourceId>,
}

/// Typed query, selection, cursor, reference, and persistence failures.
#[derive(Debug)]
#[non_exhaustive]
pub enum DictionaryQueryError {
    /// A persisted store failed to open, create, or query.
    Store(DictionaryStoreError),
    /// A composed read snapshot failed to open or match its frozen identity.
    Snapshot(DictionarySnapshotError),
    /// Input dictionary JSON or validation failed.
    Dictionary(DictionaryError),
    /// Source alias resolution failed.
    Registry(RegistryError),
    /// A known source is registered but not currently selectable.
    UnavailableSource {
        /// Stable source ID that could not be selected.
        source: SourceId,
        /// Explicit catalogue availability reason.
        reason: SourceSelectionFailure,
    },
    /// Cursor bytes, JSON, version, binding encoding, or checksum are invalid.
    MalformedCursor {
        /// Parsing, encoding, version, or checksum diagnostic.
        detail: String,
    },
    /// Cursor belongs to a different canonical state or query binding.
    StaleCursor,
    /// Cursor's full last key is not a result in this exact query.
    InvalidCursorPosition,
    /// Caller-supplied relation reference fails graph/role validation.
    InvalidReference {
        /// Failed direct-reference validation boundary.
        detail: String,
    },
    /// A loaded query index or required reference cannot be reconciled.
    Integrity {
        /// Store/model inconsistency that prevented a reliable result.
        detail: String,
    },
}

impl fmt::Display for DictionaryQueryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Store(error) => write!(f, "dictionary store: {error}"),
            Self::Snapshot(error) => write!(f, "dictionary snapshot: {error}"),
            Self::Dictionary(error) => write!(f, "dictionary input: {error}"),
            Self::Registry(error) => write!(f, "source registry: {error}"),
            Self::UnavailableSource { source, reason } => {
                write!(f, "source {source} is unavailable: {reason:?}")
            }
            Self::MalformedCursor { detail } => write!(f, "malformed dictionary cursor: {detail}"),
            Self::StaleCursor => f.write_str("dictionary cursor is stale for this query or state"),
            Self::InvalidCursorPosition => {
                f.write_str("dictionary cursor position is not a result of this query")
            }
            Self::InvalidReference { detail } => {
                write!(f, "invalid dictionary reference: {detail}")
            }
            Self::Integrity { detail } => write!(f, "dictionary integrity error: {detail}"),
        }
    }
}

impl Error for DictionaryQueryError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Store(e) => Some(e),
            Self::Snapshot(e) => Some(e),
            Self::Dictionary(e) => Some(e),
            Self::Registry(e) => Some(e),
            _ => None,
        }
    }
}
impl From<DictionaryStoreError> for DictionaryQueryError {
    fn from(value: DictionaryStoreError) -> Self {
        Self::Store(value)
    }
}
impl From<DictionarySnapshotError> for DictionaryQueryError {
    fn from(value: DictionarySnapshotError) -> Self {
        Self::Snapshot(value)
    }
}
impl From<DictionaryError> for DictionaryQueryError {
    fn from(value: DictionaryError) -> Self {
        Self::Dictionary(value)
    }
}
impl From<RegistryError> for DictionaryQueryError {
    fn from(value: RegistryError) -> Self {
        Self::Registry(value)
    }
}

impl DictionaryQueryError {
    /// Convert a typed error to its stable JSON-facing category.
    #[must_use]
    pub fn report(&self) -> DictionaryErrorReport {
        let (code, sources) = match self {
            Self::Store(_) => ("store", Vec::new()),
            Self::Snapshot(_) => ("snapshot", Vec::new()),
            Self::Dictionary(_) => ("invalid_dictionary", Vec::new()),
            Self::Registry(RegistryError::UnknownSource(_)) => ("unknown_source", Vec::new()),
            Self::Registry(RegistryError::AmbiguousAlias { sources, .. }) => {
                ("ambiguous_alias", sources.clone())
            }
            Self::Registry(_) => ("registry", Vec::new()),
            Self::UnavailableSource { source, .. } => ("unavailable_source", vec![source.clone()]),
            Self::MalformedCursor { .. } => ("malformed_cursor", Vec::new()),
            Self::StaleCursor => ("stale_cursor", Vec::new()),
            Self::InvalidCursorPosition => ("invalid_cursor_position", Vec::new()),
            Self::InvalidReference { .. } => ("invalid_reference", Vec::new()),
            Self::Integrity { .. } => ("integrity", Vec::new()),
        };
        DictionaryErrorReport {
            code: code.to_owned(),
            message: self.to_string(),
            sources,
        }
    }
}

/// A public exact-query engine retaining a typed draft from a read-only SQLite snapshot.
pub struct DictionarySearchEngine {
    provider: DictionaryQueryProvider,
    dictionary: DictionaryDocument<lexicon::RegistryDocument>,
    state_sha256: String,
    capture_index: BTreeMap<CaptureRef, usize>,
    occurrence_index: BTreeMap<(CaptureRef, String), (usize, usize)>,
    entries: BTreeMap<String, EntryRecord>,
    assertions: BTreeMap<String, lexicon::AssertionRecord>,
    sources: BTreeMap<SourceId, SourceRecord>,
    availability: Vec<DictionarySourceAvailability>,
    selected_assertions: BTreeMap<SourceId, BTreeSet<String>>,
    contexts_by_assertion: BTreeMap<String, Vec<ContextRecord>>,
    frames_by_assertion: BTreeMap<String, Vec<lexicon::ArgumentFrameRecord>>,
    forms_by_entry: BTreeMap<String, Vec<FormRecord>>,
    forms_by_id: BTreeMap<String, FormRecord>,
    etymologies_by_assertion: BTreeMap<String, Vec<EtymologyRecord>>,
    editorial_by_assertion: BTreeMap<String, Vec<EditorialSenseRecord>>,
    editorial_by_id: BTreeMap<String, EditorialSenseRecord>,
    relations_by_origin: BTreeMap<String, Vec<(usize, RelationRecord)>>,
    annotations_by_target: BTreeMap<String, Vec<(usize, ComputedAnnotation)>>,
    computations_by_id: BTreeMap<String, (usize, ComputationRecord)>,
    extensions_by_owner: BTreeMap<String, Vec<(usize, ExtensionRecord)>>,
    snapshots_by_key: BTreeMap<String, SnapshotAttribution>,
    occurrences_by_key: BTreeMap<String, OccurrenceAttribution>,
    contributors_by_id: BTreeMap<String, lexicon::ContributorRecord>,
    distributors_by_id: BTreeMap<String, DistributorRecord>,
    revisions_by_assertion: BTreeMap<String, Vec<EntityId>>,
    tombstones_by_assertion: BTreeMap<String, Vec<EntityId>>,
    assertions_by_entry: BTreeMap<String, Vec<lexicon::AssertionRecord>>,
    release_ids_by_assertion: BTreeMap<String, Vec<EntityId>>,
    generation: Option<String>,
    selected_release_ids: Option<Vec<EntityId>>,
}

#[derive(Debug)]
enum DictionaryQueryProvider {
    Store(DictionaryStore),
    Snapshot(DictionaryReadSnapshot),
}
impl DictionaryQueryProvider {
    fn state_sha256(&self) -> &str {
        match self {
            Self::Store(store) => store.recorded_state_sha256(),
            Self::Snapshot(snapshot) => snapshot.logical_state_sha256(),
        }
    }
    fn exact_assertion_ids(&self, spelling: &str) -> Result<Vec<EntityId>, DictionaryQueryError> {
        match self {
            Self::Store(store) => store.exact_assertion_ids(spelling).map_err(Into::into),
            Self::Snapshot(snapshot) => snapshot
                .exact_assertion_ids(spelling)
                .map_err(DictionaryQueryError::Snapshot),
        }
    }
    fn generation(&self) -> Option<&str> {
        match self {
            Self::Store(_) => None,
            Self::Snapshot(snapshot) => Some(snapshot.generation()),
        }
    }
    fn selected_release_ids(&self) -> Option<Vec<EntityId>> {
        match self {
            Self::Store(_) => None,
            Self::Snapshot(snapshot) => snapshot.selected_release_ids().map(<[EntityId]>::to_vec),
        }
    }
}

/// Time spent opening and indexing a dictionary snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DictionarySnapshotOpenProfile {
    /// Minimum-header connection/open and attachment setup.
    pub provider_open: Duration,
    /// Canonical reads, typed parsing, A/F composition, canonical serialization and digest.
    pub document_load_and_composition: Duration,
    /// Common typed query-index setup.
    pub common_index_setup: Duration,
    /// Total of the complete open operation.
    pub total: Duration,
}

/// Same-call exact-query timing boundaries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DictionaryExactProfile {
    /// Whole exact call.
    pub total: Duration,
    /// SQL candidate lookup.
    pub candidate_sql: Duration,
    /// Availability and returned-hit hydration/cloning.
    pub hydration: Duration,
    /// Conservative remaining lookup/selection/order/paging time.
    pub lookup_order_paging: Duration,
}

impl fmt::Debug for DictionarySearchEngine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DictionarySearchEngine")
            .field("provider", &self.provider)
            .field("state_sha256", &self.state_sha256)
            .finish_non_exhaustive()
    }
}

impl DictionarySearchEngine {
    /// Parse, validate, create, and reopen a fresh immutable dictionary store.
    ///
    /// # Errors
    ///
    /// Returns typed dictionary, store, or filesystem failures. Existing paths
    /// are never overwritten.
    pub fn build(path: impl AsRef<Path>, json_bytes: &[u8]) -> Result<Self, DictionaryQueryError> {
        let dictionary = Dictionary::from_json_bytes(json_bytes)?;
        let store = DictionaryStore::create(path, &dictionary)?;
        Self::from_store(store)
    }

    /// Open a minimally checked store and retain its typed, unverified document.
    ///
    /// # Errors
    ///
    /// Returns a typed store validation or filesystem failure.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, DictionaryQueryError> {
        Self::from_store(DictionaryStore::open(path)?)
    }

    /// Open an explicit composed snapshot provider.
    ///
    /// # Errors
    ///
    /// Returns typed manifest, store, composition, or state-identity failures.
    pub fn open_snapshot(
        topology: DictionaryReadTopology,
        manifest: DictionarySnapshotManifest,
    ) -> Result<Self, DictionaryQueryError> {
        let mut snapshot = DictionaryReadSnapshot::open(topology, manifest)?;
        let document = snapshot.load_document()?;
        Self::from_provider(DictionaryQueryProvider::Snapshot(snapshot), document)
    }

    /// Open a snapshot and report provider load plus common index setup time.
    ///
    /// # Errors
    ///
    /// Returns the same failures as [`Self::open_snapshot`].
    pub fn open_snapshot_profiled(
        topology: DictionaryReadTopology,
        manifest: DictionarySnapshotManifest,
    ) -> Result<(Self, DictionarySnapshotOpenProfile), DictionaryQueryError> {
        let total_started = Instant::now();
        let stage = Instant::now();
        let mut snapshot = DictionaryReadSnapshot::open(topology, manifest)?;
        let provider_open = stage.elapsed();
        let stage = Instant::now();
        let document = snapshot.load_document()?;
        let document_load_and_composition = stage.elapsed();
        let stage = Instant::now();
        let engine = Self::from_provider(DictionaryQueryProvider::Snapshot(snapshot), document)?;
        let common_index_setup = stage.elapsed();
        Ok((
            engine,
            DictionarySnapshotOpenProfile {
                provider_open,
                document_load_and_composition,
                common_index_setup,
                total: total_started.elapsed(),
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
        let selected_release_ids = provider.selected_release_ids();
        let capture_index = unique_map(
            document
                .captures
                .iter()
                .enumerate()
                .map(|(index, capture)| (capture.identity.clone(), index)),
            "capture",
        )?;
        let occurrence_index = unique_map(
            document
                .captures
                .iter()
                .enumerate()
                .flat_map(|(capture_index, capture)| {
                    capture.occurrences.iter().enumerate().map(
                        move |(occurrence_index, occurrence)| {
                            (
                                (capture.identity.clone(), occurrence.locator.clone()),
                                (capture_index, occurrence_index),
                            )
                        },
                    )
                }),
            "capture occurrence",
        )?;
        let entries = unique_map(
            document
                .metadata
                .entries
                .iter()
                .cloned()
                .map(|row| (row.id.to_string(), row)),
            "entry",
        )?;
        let assertions = unique_map(
            document
                .metadata
                .assertions
                .iter()
                .cloned()
                .map(|row| (row.id.to_string(), row)),
            "assertion",
        )?;
        let registry = document.registry.clone();
        let sources = unique_map(
            registry
                .sources
                .iter()
                .cloned()
                .map(|row| (row.id.clone(), row)),
            "source",
        )?;
        let mut availability = build_availability(&registry, &document);
        if let Some(selected) = &selected_release_ids {
            let known: BTreeSet<_> = document
                .metadata
                .releases
                .iter()
                .map(|release| release.id.clone())
                .collect();
            if let Some(unknown) = selected.iter().find(|id| !known.contains(*id)) {
                return Err(DictionaryQueryError::Integrity {
                    detail: format!("snapshot selected unknown release {unknown}"),
                });
            }
            for source in &mut availability {
                source
                    .selected_available_release_ids
                    .retain(|id| selected.contains(id));
                let reason = if source.releases.is_empty() {
                    Some(SourceSelectionFailure::NoReleases)
                } else if source.selected_available_release_ids.is_empty() {
                    Some(SourceSelectionFailure::NoSelectedAvailableRelease)
                } else if source.source.origin == lexicon::SourceOrigin::OwnedDraft
                    && source.owned_head.is_none()
                {
                    Some(SourceSelectionFailure::NoOwnedHead)
                } else {
                    None
                };
                source.selectable = reason.is_none();
                source.reason = reason;
            }
        }
        let selected_assertions = build_selected_assertions(&document, &availability)?;
        let contexts_by_assertion = group_by(document.metadata.contexts.iter().cloned(), |row| {
            row.assertion.to_string()
        });
        let frames_by_assertion = group_by(document.metadata.frames.iter().cloned(), |row| {
            row.assertion.to_string()
        });
        let forms_by_entry = group_by(document.metadata.forms.iter().cloned(), |row| {
            row.entry.to_string()
        });
        let forms_by_id = unique_map(
            document
                .metadata
                .forms
                .iter()
                .cloned()
                .map(|row| (row.id.to_string(), row)),
            "form",
        )?;
        let etymologies_by_assertion =
            group_by(document.metadata.etymologies.iter().cloned(), |row| {
                row.assertion.to_string()
            });
        let editorial_by_assertion = document
            .metadata
            .editorial_senses
            .iter()
            .flat_map(|row| {
                row.assertions
                    .iter()
                    .map(ToString::to_string)
                    .collect::<BTreeSet<_>>()
                    .into_iter()
                    .map(move |assertion| (assertion, row.clone()))
            })
            .fold(
                BTreeMap::<String, Vec<EditorialSenseRecord>>::new(),
                |mut map, (key, row)| {
                    map.entry(key).or_default().push(row);
                    map
                },
            );
        let editorial_by_id = unique_map(
            document
                .metadata
                .editorial_senses
                .iter()
                .cloned()
                .map(|row| (row.id.to_string(), row)),
            "editorial sense",
        )?;
        let relations_by_origin =
            indexed_group(document.metadata.relations.iter().cloned(), |row| {
                local_ref_key(&row.origin)
            });
        let annotations_by_target =
            indexed_group(document.metadata.annotations.iter().cloned(), |row| {
                local_ref_key(&row.target)
            });
        let computations_by_id = unique_map(
            document
                .metadata
                .computations
                .iter()
                .cloned()
                .enumerate()
                .map(|(ordinal, row)| (row.id.to_string(), (ordinal, row))),
            "computation",
        )?;
        let extensions_by_owner =
            indexed_group(document.metadata.extensions.iter().cloned(), |row| {
                row.owner.to_string()
            });
        let snapshots_by_key = unique_map(
            registry
                .snapshots
                .iter()
                .cloned()
                .map(|row| (snapshot_key(&row), row)),
            "snapshot attribution",
        )?;
        let occurrences_by_key = unique_map(
            registry
                .occurrences
                .iter()
                .cloned()
                .map(|row| (record_key(&row.occurrence), row)),
            "occurrence attribution",
        )?;
        let contributors_by_id = unique_map(
            registry
                .contributors
                .iter()
                .cloned()
                .map(|row| (row.id.to_string(), row)),
            "contributor",
        )?;
        let distributors_by_id = unique_map(
            registry
                .distributors
                .iter()
                .cloned()
                .map(|row| (row.id.to_string(), row)),
            "distributor",
        )?;
        let mut revisions_by_assertion = BTreeMap::<String, Vec<EntityId>>::new();
        let mut tombstones_by_assertion = BTreeMap::<String, Vec<EntityId>>::new();
        for revision in &document.metadata.owned_revisions {
            for assertion in &revision.assertions {
                revisions_by_assertion
                    .entry(assertion.to_string())
                    .or_default()
                    .push(revision.id.clone());
            }
            for assertion in &revision.tombstones {
                tombstones_by_assertion
                    .entry(assertion.to_string())
                    .or_default()
                    .push(revision.id.clone());
            }
        }
        let assertions_by_entry = group_by(document.metadata.assertions.iter().cloned(), |row| {
            row.entry.to_string()
        });
        let release_ids_by_assertion = document
            .metadata
            .releases
            .iter()
            .flat_map(|release| {
                release
                    .assertions
                    .iter()
                    .map(move |assertion| (assertion.to_string(), release.id.clone()))
            })
            .fold(
                BTreeMap::<String, Vec<EntityId>>::new(),
                |mut map, (key, id)| {
                    map.entry(key).or_default().push(id);
                    map
                },
            );
        Ok(Self {
            provider,
            dictionary: document,
            state_sha256,
            capture_index,
            occurrence_index,
            entries,
            assertions,
            sources,
            availability,
            selected_assertions,
            contexts_by_assertion,
            frames_by_assertion,
            forms_by_entry,
            forms_by_id,
            etymologies_by_assertion,
            editorial_by_assertion,
            editorial_by_id,
            relations_by_origin,
            annotations_by_target,
            computations_by_id,
            extensions_by_owner,
            snapshots_by_key,
            occurrences_by_key,
            contributors_by_id,
            distributors_by_id,
            revisions_by_assertion,
            tombstones_by_assertion,
            assertions_by_entry,
            release_ids_by_assertion,
            generation,
            selected_release_ids,
        })
    }

    /// Return the recorded header digest; it is not authenticated by fast open.
    #[must_use]
    pub fn state_sha256(&self) -> &str {
        &self.state_sha256
    }

    fn capture(&self, reference: &CaptureRef) -> Option<&lexicon::RawCapture> {
        self.capture_index
            .get(reference)
            .and_then(|index| self.dictionary.captures.get(*index))
    }

    fn occurrence(&self, reference: &OccurrenceRef) -> Option<&RawOccurrence> {
        let (capture_index, occurrence_index) = self
            .occurrence_index
            .get(&(reference.capture.clone(), reference.locator.clone()))?;
        self.dictionary
            .captures
            .get(*capture_index)?
            .occurrences
            .get(*occurrence_index)
    }

    /// Return canonical fixture population counts for the additive build CLI.
    #[must_use]
    pub fn dictionary_counts(&self) -> (usize, usize, usize, usize) {
        let document = &self.dictionary;
        (
            document.captures.len(),
            document
                .captures
                .iter()
                .map(|capture| capture.occurrences.len())
                .sum(),
            document.metadata.entries.len(),
            document.metadata.assertions.len(),
        )
    }

    /// Execute a byte-exact lexical query and return full pre-page counts.
    ///
    /// # Errors
    ///
    /// Returns typed registry, unavailable-source, cursor, store, or integrity
    /// errors. Exact spelling is never normalized.
    pub fn exact(
        &self,
        spelling: &str,
        options: DictionaryExactOptions,
    ) -> Result<DictionaryExactResponse, DictionaryQueryError> {
        self.exact_inner(spelling, options, None)
    }

    /// Execute the exact query and measure the actual same-call lookup and hydration.
    ///
    /// # Errors
    ///
    /// Returns the same errors as [`Self::exact`].
    pub fn exact_profiled(
        &self,
        spelling: &str,
        options: DictionaryExactOptions,
    ) -> Result<(DictionaryExactResponse, DictionaryExactProfile), DictionaryQueryError> {
        let total_started = Instant::now();
        let mut profile = DictionaryExactProfile::default();
        let response = self.exact_inner(spelling, options, Some(&mut profile))?;
        profile.total = total_started.elapsed();
        profile.lookup_order_paging = profile
            .total
            .saturating_sub(profile.hydration)
            .saturating_sub(profile.candidate_sql);
        Ok((response, profile))
    }

    fn exact_inner(
        &self,
        spelling: &str,
        options: DictionaryExactOptions,
        mut profile: Option<&mut DictionaryExactProfile>,
    ) -> Result<DictionaryExactResponse, DictionaryQueryError> {
        let hydration_started = Instant::now();
        let availability = self.availability.clone();
        if let Some(profile) = profile.as_mut() {
            profile.hydration += hydration_started.elapsed();
        }
        let resolved_sources = self.resolve_sources(&options, &availability)?;
        let sql_started = Instant::now();
        let candidate_ids = self.provider.exact_assertion_ids(spelling)?;
        let candidate_sql = sql_started.elapsed();
        if let Some(profile) = profile.as_mut() {
            profile.candidate_sql = candidate_sql;
        }
        let native_candidate_occurrences = candidate_ids.len();
        let candidate_set: BTreeSet<_> = candidate_ids.iter().map(ToString::to_string).collect();
        let mut hits = Vec::new();
        for assertion_id in &candidate_set {
            let assertion = require_candidate(&self.assertions, assertion_id)?;
            let selected_for_source = resolved_sources.iter().any(|source| {
                self.selected_assertions
                    .get(source)
                    .is_some_and(|assertions| assertions.contains(assertion_id))
            });
            if !selected_for_source {
                continue;
            }
            if !matches!(assertion.eligibility, Eligibility::Lexical) {
                continue;
            }
            let started = Instant::now();
            hits.push(self.make_hit(assertion)?);
            if let Some(profile) = profile.as_deref_mut() {
                profile.hydration += started.elapsed();
            }
        }
        hits.sort_by_key(hit_key);
        let mut by_source: BTreeMap<SourceId, Vec<DictionaryHit>> = BTreeMap::new();
        for hit in hits {
            by_source
                .entry(hit.entry.source.clone())
                .or_default()
                .push(hit);
        }
        let source_totals: Vec<_> = by_source
            .iter()
            .filter_map(|(source, rows)| {
                self.sources
                    .get(source)
                    .map(|record| DictionarySourceTotal {
                        source: record.clone(),
                        occurrences: rows.len(),
                        distinct_spellings: usize::from(!rows.is_empty()),
                    })
            })
            .collect();
        let occurrences = by_source.values().map(Vec::len).sum();
        let totals = DictionaryTotals {
            occurrences,
            distinct_spellings: usize::from(occurrences > 0),
            source_groups: by_source.len(),
        };
        let binding = self.binding(spelling, &resolved_sources, options.alias_scope)?;
        let all_hits: Vec<DictionaryHit> = by_source.into_values().flatten().collect();
        let start = if let Some(cursor) = options.cursor.as_deref() {
            cursor_position(cursor, &binding, &all_hits)?
        } else {
            0
        };
        let remaining = all_hits
            .get(start..)
            .ok_or(DictionaryQueryError::InvalidCursorPosition)?;
        let count = options
            .page_size
            .unwrap_or(remaining.len())
            .min(remaining.len());
        let page = &remaining[..count];
        let next_cursor = if count > 0 && start.saturating_add(count) < all_hits.len() {
            Some(make_cursor(
                &binding,
                page.last()
                    .ok_or(DictionaryQueryError::InvalidCursorPosition)?,
            )?)
        } else {
            None
        };
        let full_by_source: BTreeMap<SourceId, usize> =
            all_hits.iter().fold(BTreeMap::new(), |mut m, hit| {
                *m.entry(hit.entry.source.clone()).or_default() += 1;
                m
            });
        let mut groups: Vec<DictionaryGroup> = Vec::new();
        let group_hydration_started = Instant::now();
        for hit in page {
            let source = hit.entry.source.clone();
            if let Some(last) = groups.last_mut().filter(|g| g.source.id == source) {
                last.hits.push(hit.clone());
            } else if let Some(record) = self.sources.get(&source) {
                groups.push(DictionaryGroup {
                    source: record.clone(),
                    full_count: *full_by_source.get(&source).unwrap_or(&0),
                    returned_count: 0,
                    partial: false,
                    hits: vec![hit.clone()],
                });
            }
        }
        for group in &mut groups {
            group.returned_count = group.hits.len();
            group.partial = group.returned_count < group.full_count;
        }
        if let Some(profile) = profile {
            profile.hydration += group_hydration_started.elapsed();
        }
        let metadata = &self.dictionary.metadata;
        let response = DictionaryExactResponse {
            version: 1,
            policy: EXACT_POLICY.into(),
            store_schema_version: 1,
            catalogue_revision: metadata.catalogue_revision,
            state_sha256: self.state_sha256.clone(),
            spelling: spelling.to_owned(),
            resolved_sources,
            native_candidate_occurrences,
            totals,
            source_totals,
            availability,
            groups,
            next_cursor,
            code_notices: metadata.code_notices.clone(),
        };
        Ok(response)
    }

    /// Resolve a stable source-local or catalogue target without spelling inference.
    ///
    /// # Errors
    ///
    /// Returns [`DictionaryQueryError::InvalidReference`] for malformed or
    /// mismatched direct requests, and typed integrity/store failures.
    pub fn resolve(
        &self,
        target: &RelationTarget,
    ) -> Result<ReferenceResolution, DictionaryQueryError> {
        self.resolve_inner(target, None)
    }

    /// Resolve a local or catalogue target and report same-call lookup and
    /// response-projection timings.
    pub fn resolve_profiled(
        &self,
        target: &RelationTarget,
    ) -> Result<(ReferenceResolution, DictionaryProjectionProfile), DictionaryQueryError> {
        let started = Instant::now();
        let mut profile = DictionaryProjectionProfile::default();
        let response = self.resolve_inner(target, Some(&mut profile))?;
        profile.total = started.elapsed();
        profile.lookup = profile.total.saturating_sub(profile.projection);
        Ok((response, profile))
    }

    fn resolve_inner(
        &self,
        target: &RelationTarget,
        mut profile: Option<&mut DictionaryProjectionProfile>,
    ) -> Result<ReferenceResolution, DictionaryQueryError> {
        let metadata = &self.dictionary.metadata;
        let projection_started = profile.as_ref().map(|_| Instant::now());
        let availability = self.availability.clone();
        if let (Some(profile), Some(started)) = (profile.as_deref_mut(), projection_started) {
            profile.projection += started.elapsed();
        }
        let response = match target {
            RelationTarget::Unknown => {
                let projection_started = profile.as_ref().map(|_| Instant::now());
                let response = ReferenceResolution {
                    version: 1,
                    state_sha256: self.state_sha256.clone(),
                    target: target.clone(),
                    status: ReferenceStatus::Unknown,
                    entity: None,
                    confirmed: vec![],
                    unverified: vec![],
                    availability,
                    requested_release: None,
                };
                if let (Some(profile), Some(started)) = (profile.as_deref_mut(), projection_started)
                {
                    profile.projection += started.elapsed();
                }
                response
            }
            RelationTarget::Unresolved(text) => {
                if text.is_empty() {
                    return Err(invalid_ref("Unresolved target must be nonempty"));
                }
                let projection_started = profile.as_ref().map(|_| Instant::now());
                let response = ReferenceResolution {
                    version: 1,
                    state_sha256: self.state_sha256.clone(),
                    target: target.clone(),
                    status: ReferenceStatus::Unresolved,
                    entity: None,
                    confirmed: vec![],
                    unverified: vec![],
                    availability,
                    requested_release: None,
                };
                if let (Some(profile), Some(started)) = (profile.as_deref_mut(), projection_started)
                {
                    profile.projection += started.elapsed();
                }
                response
            }
            RelationTarget::Local(local) => {
                let entity = self
                    .local_entity(local, profile.as_deref_mut())
                    .ok_or_else(|| {
                        invalid_ref("LocalRef does not exist in its specific role table")
                    })?;
                let ids: BTreeSet<String> = self.local_assertion_ids(local).into_iter().collect();
                let views = self.reference_views(&ids, profile.as_deref_mut());
                let projection_started = profile.as_ref().map(|_| Instant::now());
                let response = ReferenceResolution {
                    version: 1,
                    state_sha256: self.state_sha256.clone(),
                    target: target.clone(),
                    status: ReferenceStatus::Resolved,
                    entity: Some(entity),
                    confirmed: views,
                    unverified: vec![],
                    availability,
                    requested_release: None,
                };
                if let (Some(profile), Some(started)) = (profile.as_deref_mut(), projection_started)
                {
                    profile.projection += started.elapsed();
                }
                response
            }
            RelationTarget::Catalog(catalog) => {
                self.resolve_catalog(target, catalog, metadata, availability, profile)?
            }
        };
        Ok(response)
    }

    fn resolve_sources(
        &self,
        options: &DictionaryExactOptions,
        availability: &[DictionarySourceAvailability],
    ) -> Result<Vec<SourceId>, DictionaryQueryError> {
        let mut selected = BTreeSet::new();
        if options.sources.is_empty() {
            for row in availability.iter().filter(|row| row.selectable) {
                selected.insert(row.source.id.clone());
            }
        } else {
            for selector in &options.sources {
                let source = self
                    .dictionary
                    .registry
                    .resolve_source(selector, options.alias_scope)?;
                let state = availability
                    .iter()
                    .find(|row| row.source.id == source.id)
                    .ok_or_else(|| DictionaryQueryError::Integrity {
                        detail: "resolved source is absent from availability".into(),
                    })?;
                if let Some(reason) = state.reason {
                    return Err(DictionaryQueryError::UnavailableSource {
                        source: source.id.clone(),
                        reason,
                    });
                }
                selected.insert(source.id.clone());
            }
        }
        Ok(selected.into_iter().collect())
    }

    fn make_hit(
        &self,
        assertion: &lexicon::AssertionRecord,
    ) -> Result<DictionaryHit, DictionaryQueryError> {
        let entry = self
            .entries
            .get(&assertion.entry.to_string())
            .cloned()
            .ok_or_else(|| DictionaryQueryError::Integrity {
                detail: format!("assertion {} references missing entry", assertion.id),
            })?;
        let capture = self.capture(&assertion.occurrence.capture).ok_or_else(|| {
            DictionaryQueryError::Integrity {
                detail: format!("selected occurrence {} has no capture", assertion.id),
            }
        })?;
        let raw = self
            .occurrence(&assertion.occurrence)
            .ok_or_else(|| DictionaryQueryError::Integrity {
                detail: format!("selected occurrence {} has no raw row", assertion.id),
            })?
            .clone();
        let snapshot_attribution = self
            .snapshots_by_key
            .get(&snapshot_key_for_ref(&assertion.occurrence.capture))
            .cloned()
            .ok_or_else(|| DictionaryQueryError::Integrity {
                detail: "loaded capture has no snapshot attribution".into(),
            })?;
        let occurrence_attribution = self
            .occurrences_by_key
            .get(&tuple_hex(&assertion.occurrence))
            .cloned();
        let contributor = occurrence_attribution
            .as_ref()
            .and_then(|attr| attr.contributor.as_ref())
            .and_then(|id| self.contributors_by_id.get(&id.to_string()))
            .cloned();
        let distributor = snapshot_attribution
            .distributor
            .as_ref()
            .and_then(|id| self.distributors_by_id.get(&id.to_string()))
            .cloned();
        let source_availability = self
            .availability
            .iter()
            .find(|row| row.source.id == entry.source)
            .ok_or_else(|| DictionaryQueryError::Integrity {
                detail: "hit source absent from registry".into(),
            })?;
        let selected_available_releases: BTreeSet<_> = source_availability
            .selected_available_release_ids
            .iter()
            .cloned()
            .collect();
        let release_ids: Vec<_> = self
            .release_ids_by_assertion
            .get(&assertion.id.to_string())
            .into_iter()
            .flatten()
            .filter(|release| selected_available_releases.contains(*release))
            .cloned()
            .collect();
        let assertion_key = assertion.id.to_string();
        let entry_key = entry.id.to_string();
        let contexts = self
            .contexts_by_assertion
            .get(&assertion_key)
            .cloned()
            .unwrap_or_default();
        let frames = self
            .frames_by_assertion
            .get(&assertion_key)
            .cloned()
            .unwrap_or_default();
        let forms = self
            .forms_by_entry
            .get(&entry_key)
            .cloned()
            .unwrap_or_default();
        let etymologies = self
            .etymologies_by_assertion
            .get(&assertion_key)
            .cloned()
            .unwrap_or_default();
        let editorial_senses = self
            .editorial_by_assertion
            .get(&assertion_key)
            .cloned()
            .unwrap_or_default();
        let mut local_keys = BTreeSet::new();
        local_keys.insert(local_ref_key(&LocalRef::Entry(entry.id.clone())));
        local_keys.insert(local_ref_key(&LocalRef::Assertion(assertion.id.clone())));
        for form in &forms {
            local_keys.insert(local_ref_key(&LocalRef::Form(form.id.clone())));
        }
        for sense in &editorial_senses {
            local_keys.insert(local_ref_key(&LocalRef::EditorialSense(sense.id.clone())));
        }
        let relations = collect_indexed(&self.relations_by_origin, &local_keys);
        let annotations = collect_indexed(&self.annotations_by_target, &local_keys);
        let mut computation_ids: BTreeSet<String> = forms
            .iter()
            .filter_map(|row| row.computation.as_ref().map(ToString::to_string))
            .collect();
        computation_ids.extend(annotations.iter().map(|row| row.computation.to_string()));
        let mut selected_computations: Vec<_> = computation_ids
            .iter()
            .map(|id| {
                self.computations_by_id
                    .get(id)
                    .ok_or_else(|| DictionaryQueryError::Integrity {
                        detail: format!(
                            "associated computation {id} is absent from the loaded dictionary draft"
                        ),
                    })
            })
            .collect::<Result<_, _>>()?;
        selected_computations.sort_by_key(|(ordinal, _)| *ordinal);
        let computations: Vec<_> = selected_computations
            .into_iter()
            .map(|(_, record)| record.clone())
            .collect();
        let mut owned_ids: BTreeSet<String> = [entry.id.to_string(), assertion.id.to_string()]
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
        let extensions = collect_indexed(&self.extensions_by_owner, &owned_ids);
        let owned_revision_ids = self
            .revisions_by_assertion
            .get(&assertion_key)
            .cloned()
            .unwrap_or_default();
        let tombstoned_by = self
            .tombstones_by_assertion
            .get(&assertion_key)
            .cloned()
            .unwrap_or_default();
        let occurrence_id = tuple_hex(&assertion.occurrence);
        Ok(DictionaryHit {
            occurrence_id,
            entry,
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

    fn binding(
        &self,
        spelling: &str,
        sources: &[SourceId],
        alias_scope: AliasScope,
    ) -> Result<String, DictionaryQueryError> {
        #[derive(Serialize)]
        struct Binding<'a> {
            version: u32,
            policy: &'a str,
            store_schema_version: u32,
            state_sha256: &'a str,
            spelling: &'a str,
            resolved_sources: &'a [SourceId],
            alias_scope: AliasScope,
            selection_policy: &'a str,
        }
        let old_binding = Binding {
            version: 1,
            policy: EXACT_POLICY,
            store_schema_version: 1,
            state_sha256: &self.state_sha256,
            spelling,
            resolved_sources: sources,
            alias_scope,
            selection_policy: SELECTION_POLICY,
        };
        let bytes = if let Some(generation) = &self.generation {
            #[derive(Serialize)]
            struct SnapshotBinding<'a> {
                version: u32,
                policy: &'a str,
                store_schema_version: u32,
                state_sha256: &'a str,
                spelling: &'a str,
                resolved_sources: &'a [SourceId],
                alias_scope: AliasScope,
                selection_policy: &'a str,
                generation: &'a str,
                selected_release_ids: &'a Option<Vec<EntityId>>,
            }
            serde_json::to_vec(&SnapshotBinding {
                version: old_binding.version,
                policy: old_binding.policy,
                store_schema_version: old_binding.store_schema_version,
                state_sha256: old_binding.state_sha256,
                spelling: old_binding.spelling,
                resolved_sources: old_binding.resolved_sources,
                alias_scope: old_binding.alias_scope,
                selection_policy: old_binding.selection_policy,
                generation,
                selected_release_ids: &self.selected_release_ids,
            })
        } else {
            serde_json::to_vec(&old_binding)
        }
        .map_err(|e| DictionaryQueryError::Integrity {
            detail: e.to_string(),
        })?;
        Ok(hex(&Sha256::digest(bytes)))
    }

    fn local_entity(
        &self,
        reference: &LocalRef,
        mut profile: Option<&mut DictionaryProjectionProfile>,
    ) -> Option<ReferenceEntity> {
        match reference {
            LocalRef::Entry(id) => {
                let row = self.entries.get(&id.to_string())?;
                let started = profile.as_ref().map(|_| Instant::now());
                let entity = ReferenceEntity::Entry(row.clone());
                if let (Some(profile), Some(started)) = (profile.as_mut(), started) {
                    profile.projection += started.elapsed();
                }
                Some(entity)
            }
            LocalRef::Assertion(id) => {
                let row = self.assertions.get(&id.to_string())?;
                let started = profile.as_ref().map(|_| Instant::now());
                let entity = ReferenceEntity::Assertion(row.clone());
                if let (Some(profile), Some(started)) = (profile.as_mut(), started) {
                    profile.projection += started.elapsed();
                }
                Some(entity)
            }
            LocalRef::Form(id) => {
                let row = self.forms_by_id.get(&id.to_string())?;
                let started = profile.as_ref().map(|_| Instant::now());
                let entity = ReferenceEntity::Form(row.clone());
                if let (Some(profile), Some(started)) = (profile.as_mut(), started) {
                    profile.projection += started.elapsed();
                }
                Some(entity)
            }
            LocalRef::EditorialSense(id) => {
                let row = self.editorial_by_id.get(&id.to_string())?;
                let started = profile.as_ref().map(|_| Instant::now());
                let entity = ReferenceEntity::EditorialSense(row.clone());
                if let (Some(profile), Some(started)) = (profile.as_mut(), started) {
                    profile.projection += started.elapsed();
                }
                Some(entity)
            }
        }
    }

    fn local_assertion_ids(&self, reference: &LocalRef) -> Vec<String> {
        match reference {
            LocalRef::Entry(id) => self
                .assertions_by_entry
                .get(&id.to_string())
                .map(|rows| rows.iter().map(|row| row.id.to_string()).collect())
                .unwrap_or_default(),
            LocalRef::Assertion(id) => self
                .assertions
                .get(&id.to_string())
                .map(|row| row.id.to_string())
                .into_iter()
                .collect(),
            LocalRef::Form(id) => self
                .forms_by_id
                .get(&id.to_string())
                .and_then(|form| self.assertions_by_entry.get(&form.entry.to_string()))
                .map(|rows| rows.iter().map(|row| row.id.to_string()).collect())
                .unwrap_or_default(),
            LocalRef::EditorialSense(id) => self
                .editorial_by_id
                .get(&id.to_string())
                .map(|sense| sense.assertions.iter().map(ToString::to_string).collect())
                .unwrap_or_default(),
        }
    }

    fn resolve_catalog(
        &self,
        target: &RelationTarget,
        catalog: &CatalogTarget,
        metadata: &lexicon::DictionaryMetadata,
        availability: Vec<DictionarySourceAvailability>,
        mut profile: Option<&mut DictionaryProjectionProfile>,
    ) -> Result<ReferenceResolution, DictionaryQueryError> {
        let release = metadata
            .releases
            .iter()
            .find(|row| row.id == catalog.release)
            .ok_or_else(|| invalid_ref("catalogue release does not exist"))?;
        let component = metadata
            .components
            .iter()
            .find(|row| row.id == release.component)
            .ok_or_else(|| invalid_ref("catalogue release has no component"))?;
        let collection = metadata
            .collections
            .iter()
            .find(|row| row.id == component.collection)
            .ok_or_else(|| invalid_ref("catalogue release has no collection"))?;
        if collection.source != catalog.source {
            return Err(invalid_ref(
                "catalogue release is not owned by the requested source",
            ));
        }
        if catalog.native_entry_id.is_none()
            && catalog.native_assertion_id.is_none()
            && catalog.occurrence.is_none()
        {
            return Err(invalid_ref(
                "catalogue target supplies no native identity or occurrence",
            ));
        }
        if let Some(occurrence) = &catalog.occurrence {
            if occurrence.locator.is_empty() {
                return Err(invalid_ref("catalogue occurrence locator is empty"));
            }
            if !release.captures.contains(&occurrence.capture) {
                return Err(invalid_ref(
                    "catalogue occurrence capture is not in the requested release",
                ));
            }
            if self.capture(&occurrence.capture).is_some() && self.occurrence(occurrence).is_none()
            {
                return Err(invalid_ref(
                    "loaded capture does not contain requested locator",
                ));
            }
        }
        let base_ids: BTreeSet<String> = release
            .assertions
            .iter()
            .filter(|id| {
                let Some(assertion) = self.assertions.get(&id.to_string()) else {
                    return false;
                };
                if let Some(occurrence) = &catalog.occurrence
                    && assertion.occurrence != *occurrence
                {
                    return false;
                }
                true
            })
            .map(ToString::to_string)
            .collect();
        let mut confirmed_ids = BTreeSet::new();
        let mut unverified_ids = BTreeSet::new();
        for id in &base_ids {
            let assertion =
                self.assertions
                    .get(id)
                    .ok_or_else(|| DictionaryQueryError::Integrity {
                        detail: "release assertion missing from model".into(),
                    })?;
            let entry = self
                .entries
                .get(&assertion.entry.to_string())
                .ok_or_else(|| DictionaryQueryError::Integrity {
                    detail: "release entry missing from model".into(),
                })?;
            if entry.source != catalog.source {
                continue;
            }
            let attr = self
                .occurrences_by_key
                .get(&tuple_hex(&assertion.occurrence));
            let mut unknown = false;
            let mut mismatch = false;
            for (expected, actual) in [
                (
                    catalog.native_entry_id.as_ref(),
                    attr.and_then(|a| a.native_entry_id.as_ref()),
                ),
                (
                    catalog.native_assertion_id.as_ref(),
                    attr.and_then(|a| a.native_assertion_id.as_ref()),
                ),
            ] {
                if let Some(expected) = expected {
                    match actual {
                        Some(actual) if actual == expected => {}
                        Some(_) => mismatch = true,
                        None => unknown = true,
                    }
                }
            }
            if mismatch {
                continue;
            }
            if unknown {
                unverified_ids.insert(id.clone());
            } else {
                confirmed_ids.insert(id.clone());
            }
        }
        let confirmed = self.reference_views(&confirmed_ids, profile.as_deref_mut());
        let unverified = self.reference_views(&unverified_ids, profile.as_deref_mut());
        let status = if !confirmed.is_empty() {
            ReferenceStatus::Resolved
        } else if !unverified.is_empty() {
            ReferenceStatus::Unverified
        } else if base_ids.is_empty() && release.availability != Availability::Available {
            ReferenceStatus::CatalogueOnly
        } else {
            ReferenceStatus::NoMatch
        };
        let projection_started = profile.as_ref().map(|_| Instant::now());
        let response = ReferenceResolution {
            version: 1,
            state_sha256: self.state_sha256.clone(),
            target: target.clone(),
            status,
            entity: None,
            confirmed,
            unverified,
            availability,
            requested_release: Some(release.clone()),
        };
        if let (Some(profile), Some(started)) = (profile, projection_started) {
            profile.projection += started.elapsed();
        }
        Ok(response)
    }

    fn reference_views(
        &self,
        ids: &BTreeSet<String>,
        mut profile: Option<&mut DictionaryProjectionProfile>,
    ) -> Vec<ReferenceAssertionView> {
        let mut views = Vec::new();
        for id in ids {
            let Some(assertion) = self.assertions.get(id) else {
                continue;
            };
            let Some(entry) = self.entries.get(&assertion.entry.to_string()) else {
                continue;
            };
            let capture = self.capture(&assertion.occurrence.capture);
            let raw = self.occurrence(&assertion.occurrence);
            let attr = self
                .occurrences_by_key
                .get(&tuple_hex(&assertion.occurrence));
            let releases = self.release_ids_by_assertion.get(id);
            let owned_revision_ids = self.revisions_by_assertion.get(id);
            let tombstoned_by = self.tombstones_by_assertion.get(id);
            let current_eligible = self
                .selected_assertions
                .get(&entry.source)
                .is_some_and(|selected| selected.contains(id))
                && matches!(assertion.eligibility, Eligibility::Lexical);
            let payload_present =
                capture.is_some() && self.occurrence(&assertion.occurrence).is_some();
            let projection_started = profile.as_ref().map(|_| Instant::now());
            views.push(ReferenceAssertionView {
                entry: entry.clone(),
                assertion: assertion.clone(),
                occurrence_id: tuple_hex(&assertion.occurrence),
                occurrence_attribution: attr.cloned(),
                raw: raw.cloned(),
                capture_metadata: capture.map(|row| row.metadata.clone()),
                payload_present,
                current_eligible,
                release_ids: releases.cloned().unwrap_or_default(),
                owned_revision_ids: owned_revision_ids.cloned().unwrap_or_default(),
                tombstoned_by: tombstoned_by.cloned().unwrap_or_default(),
            });
            if let (Some(profile), Some(started)) = (profile.as_deref_mut(), projection_started) {
                profile.projection += started.elapsed();
            }
        }
        views.sort_by(|a, b| {
            (
                a.entry.source.as_str(),
                spelling_key(&a.entry),
                &a.occurrence_id,
            )
                .cmp(&(
                    b.entry.source.as_str(),
                    spelling_key(&b.entry),
                    &b.occurrence_id,
                ))
        });
        views
    }
}

fn group_by<T>(
    rows: impl Iterator<Item = T>,
    key: impl Fn(&T) -> String,
) -> BTreeMap<String, Vec<T>> {
    let mut grouped = BTreeMap::<String, Vec<T>>::new();
    for row in rows {
        grouped.entry(key(&row)).or_default().push(row);
    }
    grouped
}

fn unique_map<K: Ord, V>(
    rows: impl IntoIterator<Item = (K, V)>,
    role: &str,
) -> Result<BTreeMap<K, V>, DictionaryQueryError> {
    let mut indexed = BTreeMap::new();
    for (key, row) in rows {
        if indexed.insert(key, row).is_some() {
            return Err(DictionaryQueryError::Integrity {
                detail: format!("duplicate {role} identity in query index"),
            });
        }
    }
    Ok(indexed)
}

fn require_candidate<'a, T>(
    assertions: &'a BTreeMap<String, T>,
    assertion_id: &str,
) -> Result<&'a T, DictionaryQueryError> {
    assertions
        .get(assertion_id)
        .ok_or_else(|| DictionaryQueryError::Integrity {
            detail: format!(
                "SQL candidate {assertion_id} is absent from the loaded dictionary draft"
            ),
        })
}

#[cfg(test)]
mod unverified_index_tests {
    use super::{DictionaryQueryError, require_candidate, unique_map};
    use std::collections::BTreeMap;

    #[test]
    fn test_required_lookup_indexes_keep_unique_ids_and_reject_duplicates() {
        let valid = unique_map([(String::from("entry-a"), 1usize)], "entry")
            .expect("one required lookup identity is valid");
        assert_eq!(valid.get("entry-a"), Some(&1));

        let duplicate = unique_map(
            [
                (String::from("entry-a"), 1usize),
                (String::from("entry-a"), 2usize),
            ],
            "entry",
        )
        .expect_err("duplicate IDs must not silently overwrite an earlier row");
        assert!(matches!(
            duplicate,
            super::DictionaryQueryError::Integrity { .. }
        ));
    }

    #[test]
    fn test_sql_candidate_must_exist_before_source_selection_can_skip_it() {
        let assertions = BTreeMap::from([(String::from("known-assertion"), 17usize)]);
        assert_eq!(
            *require_candidate(&assertions, "known-assertion").unwrap(),
            17
        );
        assert!(matches!(
            require_candidate(&assertions, "corrupt-sql-assertion"),
            Err(DictionaryQueryError::Integrity { .. })
        ));
    }
}

fn indexed_group<T>(
    rows: impl Iterator<Item = T>,
    key: impl Fn(&T) -> String,
) -> BTreeMap<String, Vec<(usize, T)>> {
    let mut grouped = BTreeMap::<String, Vec<(usize, T)>>::new();
    for (index, row) in rows.enumerate() {
        grouped.entry(key(&row)).or_default().push((index, row));
    }
    grouped
}

fn collect_indexed<T: Clone>(
    index: &BTreeMap<String, Vec<(usize, T)>>,
    keys: &BTreeSet<String>,
) -> Vec<T> {
    let mut rows: Vec<_> = keys
        .iter()
        .filter_map(|key| index.get(key))
        .flatten()
        .collect();
    rows.sort_by_key(|(position, _)| *position);
    rows.into_iter().map(|(_, row)| row.clone()).collect()
}

fn build_availability(
    registry: &lexicon::RegistryDocument,
    document: &lexicon::UnverifiedDictionaryDocument,
) -> Vec<DictionarySourceAvailability> {
    let mut sources = registry.sources.clone();
    sources.sort_by(|a, b| a.id.cmp(&b.id));
    sources
        .into_iter()
        .map(|source| {
            let collections: Vec<_> = document
                .metadata
                .collections
                .iter()
                .filter(|row| row.source == source.id)
                .cloned()
                .collect();
            let collection_ids: BTreeSet<_> =
                collections.iter().map(|row| row.id.clone()).collect();
            let components: Vec<_> = document
                .metadata
                .components
                .iter()
                .filter(|row| collection_ids.contains(&row.collection))
                .cloned()
                .collect();
            let component_ids: BTreeSet<_> = components.iter().map(|row| row.id.clone()).collect();
            let releases: Vec<_> = document
                .metadata
                .releases
                .iter()
                .filter(|row| component_ids.contains(&row.component))
                .cloned()
                .collect();
            let selected_available_release_ids: Vec<_> = releases
                .iter()
                .filter(|row| row.selected && row.availability == Availability::Available)
                .map(|row| row.id.clone())
                .collect();
            let owned_head = document
                .metadata
                .owned_heads
                .iter()
                .find(|head| head.source == source.id)
                .cloned();
            let reason = if releases.is_empty() {
                Some(SourceSelectionFailure::NoReleases)
            } else if selected_available_release_ids.is_empty() {
                Some(SourceSelectionFailure::NoSelectedAvailableRelease)
            } else if source.origin == lexicon::SourceOrigin::OwnedDraft && owned_head.is_none() {
                Some(SourceSelectionFailure::NoOwnedHead)
            } else {
                None
            };
            let (loaded_captures, missing_captures) =
                capture_partition(&releases, &document.captures);
            DictionarySourceAvailability {
                aliases: registry
                    .aliases
                    .iter()
                    .filter(|alias| alias.source == source.id)
                    .cloned()
                    .collect(),
                lineage: registry
                    .lineage
                    .iter()
                    .filter(|edge| edge.source == source.id)
                    .cloned()
                    .collect(),
                source,
                collections,
                components,
                releases,
                selected_available_release_ids,
                loaded_captures,
                missing_captures,
                owned_head,
                selectable: reason.is_none(),
                reason,
            }
        })
        .collect()
}

fn build_selected_assertions(
    document: &lexicon::UnverifiedDictionaryDocument,
    availability: &[DictionarySourceAvailability],
) -> Result<BTreeMap<SourceId, BTreeSet<String>>, DictionaryQueryError> {
    let revisions = unique_map(
        document
            .metadata
            .owned_revisions
            .iter()
            .map(|row| (row.id.clone(), row)),
        "owned revision",
    )?;
    let mut selected = BTreeMap::new();
    for source in availability {
        let release_ids: BTreeSet<_> = source
            .selected_available_release_ids
            .iter()
            .cloned()
            .collect();
        let release_assertions: BTreeSet<String> = source
            .releases
            .iter()
            .filter(|release| release_ids.contains(&release.id))
            .flat_map(|release| release.assertions.iter().map(ToString::to_string))
            .collect();
        let ids = if source.source.origin == lexicon::SourceOrigin::OwnedDraft {
            source
                .owned_head
                .as_ref()
                .and_then(|head| revisions.get(&head.revision))
                .map(|revision| {
                    release_assertions
                        .intersection(
                            &revision
                                .assertions
                                .iter()
                                .map(ToString::to_string)
                                .collect(),
                        )
                        .cloned()
                        .collect()
                })
                .unwrap_or_default()
        } else {
            release_assertions
        };
        selected.insert(source.source.id.clone(), ids);
    }
    Ok(selected)
}

fn local_ref_key(reference: &LocalRef) -> String {
    match reference {
        LocalRef::Entry(id) => format!("entry:{id}"),
        LocalRef::Assertion(id) => format!("assertion:{id}"),
        LocalRef::Form(id) => format!("form:{id}"),
        LocalRef::EditorialSense(id) => format!("editorial_sense:{id}"),
    }
}

fn key_parts(parts: &[&[u8]]) -> String {
    let mut bytes = Vec::new();
    for part in parts {
        bytes.extend_from_slice(&(part.len() as u64).to_be_bytes());
        bytes.extend_from_slice(part);
    }
    hex(&bytes)
}

fn snapshot_key(row: &SnapshotAttribution) -> String {
    key_parts(&[
        row.snapshot.namespace().as_bytes(),
        row.snapshot.sha256().as_bytes(),
    ])
}
fn snapshot_key_for_ref(capture: &CaptureRef) -> String {
    key_parts(&[capture.namespace.as_bytes(), capture.sha256.as_bytes()])
}
fn record_key(row: &lexicon::RecordIdentity) -> String {
    tuple_hex(&OccurrenceRef::from(row))
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CursorBody {
    version: u32,
    binding: String,
    last: CursorKey,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CursorKey {
    source: SourceId,
    spelling: String,
    occurrence_id: String,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CursorEnvelope {
    body: CursorBody,
    checksum: String,
}

fn hit_key(hit: &DictionaryHit) -> (String, String, String) {
    (
        hit.entry.source.to_string(),
        spelling_key(&hit.entry),
        hit.occurrence_id.clone(),
    )
}
fn spelling_key(entry: &EntryRecord) -> String {
    match &entry.spelling {
        lexicon::Presence::Value(value) => value.clone(),
        _ => String::new(),
    }
}
fn tuple_hex(reference: &OccurrenceRef) -> String {
    let parts = [
        reference.capture.namespace.as_bytes(),
        reference.capture.sha256.as_bytes(),
        reference.locator.as_bytes(),
    ];
    let mut bytes = Vec::new();
    for part in parts {
        bytes.extend_from_slice(&(part.len() as u64).to_be_bytes());
        bytes.extend_from_slice(part);
    }
    hex(&bytes)
}
fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(DIGITS[(byte >> 4) as usize] as char);
        out.push(DIGITS[(byte & 0xf) as usize] as char);
    }
    out
}
fn unhex(text: &str) -> Result<Vec<u8>, DictionaryQueryError> {
    if !text.len().is_multiple_of(2)
        || !text
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(malformed_cursor(
            "cursor body is not lowercase even-length hex",
        ));
    }
    let (pairs, remainder) = text.as_bytes().as_chunks::<2>();
    debug_assert!(remainder.is_empty());
    pairs
        .iter()
        .map(|pair| {
            let high = (pair[0] as char)
                .to_digit(16)
                .ok_or_else(|| malformed_cursor("invalid hex"))?;
            let low = (pair[1] as char)
                .to_digit(16)
                .ok_or_else(|| malformed_cursor("invalid hex"))?;
            Ok(((high << 4) | low) as u8)
        })
        .collect()
}
fn malformed_cursor(detail: impl Into<String>) -> DictionaryQueryError {
    DictionaryQueryError::MalformedCursor {
        detail: detail.into(),
    }
}
fn invalid_ref(detail: impl Into<String>) -> DictionaryQueryError {
    DictionaryQueryError::InvalidReference {
        detail: detail.into(),
    }
}
fn capture_partition(
    releases: &[ReleaseRecord],
    captures: &[lexicon::RawCapture],
) -> (Vec<CaptureRef>, Vec<CaptureRef>) {
    let loaded: BTreeSet<_> = captures
        .iter()
        .map(|capture| capture.identity.clone())
        .collect();
    let mut seen = BTreeSet::new();
    let mut yes = Vec::new();
    let mut no = Vec::new();
    for capture in releases.iter().flat_map(|release| &release.captures) {
        if seen.insert(capture.clone()) {
            if loaded.contains(capture) {
                yes.push(capture.clone());
            } else {
                no.push(capture.clone());
            }
        }
    }
    (yes, no)
}
fn make_cursor(binding: &str, hit: &DictionaryHit) -> Result<String, DictionaryQueryError> {
    let key = CursorKey {
        source: hit.entry.source.clone(),
        spelling: spelling_key(&hit.entry),
        occurrence_id: hit.occurrence_id.clone(),
    };
    let body = CursorBody {
        version: 1,
        binding: binding.to_owned(),
        last: key,
    };
    let body_bytes =
        serde_json::to_vec(&body).map_err(|error| DictionaryQueryError::Integrity {
            detail: error.to_string(),
        })?;
    let checksum = hex(&Sha256::digest(&body_bytes));
    let envelope = CursorEnvelope { body, checksum };
    let bytes = serde_json::to_vec(&envelope).map_err(|error| DictionaryQueryError::Integrity {
        detail: error.to_string(),
    })?;
    Ok(hex(&bytes))
}
fn cursor_position(
    cursor: &str,
    binding: &str,
    hits: &[DictionaryHit],
) -> Result<usize, DictionaryQueryError> {
    if cursor.starts_with("dictionary-compact-v1:") {
        return if is_valid_compact_cursor(cursor) {
            Err(DictionaryQueryError::StaleCursor)
        } else {
            Err(malformed_cursor("malformed foreign compact cursor"))
        };
    }
    let bytes = unhex(cursor)?;
    let envelope: CursorEnvelope =
        serde_json::from_slice(&bytes).map_err(|error| malformed_cursor(error.to_string()))?;
    if !is_sha256_hex(&envelope.body.binding) || !is_sha256_hex(&envelope.checksum) {
        return Err(malformed_cursor(
            "cursor binding or checksum is not a lowercase SHA-256 digest",
        ));
    }
    let body_bytes =
        serde_json::to_vec(&envelope.body).map_err(|error| malformed_cursor(error.to_string()))?;
    if envelope.body.version != 1 || hex(&Sha256::digest(&body_bytes)) != envelope.checksum {
        return Err(malformed_cursor("cursor version or checksum is invalid"));
    }
    if envelope.body.binding != binding {
        return Err(DictionaryQueryError::StaleCursor);
    }
    let wanted = (
        envelope.body.last.source.to_string(),
        envelope.body.last.spelling,
        envelope.body.last.occurrence_id,
    );
    hits.iter()
        .position(|hit| hit_key(hit) == wanted)
        .map(|index| index + 1)
        .ok_or(DictionaryQueryError::InvalidCursorPosition)
}

fn is_valid_compact_cursor(cursor: &str) -> bool {
    #[derive(Deserialize, Serialize)]
    #[serde(deny_unknown_fields)]
    struct CompactBody {
        version: u32,
        binding: String,
        last: CompactKey,
    }
    #[derive(Deserialize, Serialize)]
    #[serde(deny_unknown_fields)]
    struct CompactKey {
        source: SourceId,
        spelling: String,
        occurrence_id: String,
        assertion_id: String,
    }
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct CompactEnvelope {
        body: CompactBody,
        checksum: String,
    }

    let Some(encoded) = cursor.strip_prefix("dictionary-compact-v1:") else {
        return false;
    };
    let Ok(bytes) = unhex(encoded) else {
        return false;
    };
    let Ok(envelope) = serde_json::from_slice::<CompactEnvelope>(&bytes) else {
        return false;
    };
    let Ok(body) = serde_json::to_vec(&envelope.body) else {
        return false;
    };
    envelope.body.version == 1
        && is_sha256_hex(&envelope.body.binding)
        && is_sha256_hex(&envelope.checksum)
        && !envelope.body.last.assertion_id.is_empty()
        && !envelope.body.last.spelling.is_empty()
        && valid_cursor_occurrence_hex(&envelope.body.last.occurrence_id)
        && hex(&Sha256::digest(body)) == envelope.checksum
}

fn valid_cursor_occurrence_hex(value: &str) -> bool {
    !value.is_empty()
        && value.len().is_multiple_of(2)
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn is_sha256_hex(text: &str) -> bool {
    text.len() == 64
        && text
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
