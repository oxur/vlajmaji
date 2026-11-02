//! Explicit queries over a freshness-bound mixed dictionary workbench.

use std::{error::Error, fmt, path::Path};

use serde::{Deserialize, Serialize};
use vlajmaji_lexicon::OwnedCurationStatus;
use vlajmaji_store::{
    ConfiguredWorkbenchView, DictionaryReadTopology, WorkbenchBinding, WorkbenchError,
    WorkbenchOwnedRevision, WorkbenchProfileError, WorkbenchProfileIdentity, WorkbenchView,
};

use crate::{
    DictionaryCompactQueryResult, DictionaryCompactSearchEngine, DictionaryExactResponse,
    DictionaryHitDetailRef, DictionaryHitDetailResponse, DictionaryProjectionError,
    DictionaryQueryError, DictionaryRequest, DictionarySearchEngine, ReferenceResolution,
};

pub(crate) mod computed;
/// Literal, graph, computed, proposal, and morphology workbench queries.
pub mod lexical;
pub mod links;
pub(crate) mod reference;
pub use computed::{
    WorkbenchAnalysisClassEvidence, WorkbenchAnalysisEvidence, WorkbenchComputedCompact,
    WorkbenchComputedCompactHit, WorkbenchComputedFull, WorkbenchComputedFullHit,
    WorkbenchComputedSourceTotal, WorkbenchComputedTotals, WorkbenchGeneratedResult,
    WorkbenchProposalInput, WorkbenchProposalRecord, WorkbenchProposalResult,
    WorkbenchProposalTotals,
};
pub use lexical::{
    ConfiguredWorkbenchFormDetailResponse, ConfiguredWorkbenchLexicalCompactResponse,
    ConfiguredWorkbenchLexicalFullResponse, WorkbenchCompactFormRecord, WorkbenchFormCompact,
    WorkbenchFormCompactGroup, WorkbenchFormDetailRef, WorkbenchFormDetailResponse,
    WorkbenchFormEvidence, WorkbenchFormFull, WorkbenchFormFullGroup, WorkbenchFormRecord,
    WorkbenchLexicalCompactResponse, WorkbenchLexicalCompactResult, WorkbenchLexicalError,
    WorkbenchLexicalFullResponse, WorkbenchLexicalFullResult, WorkbenchLexicalOptions,
    WorkbenchLexicalQuery, WorkbenchLexicalRequest, WorkbenchLexicalSourceTotal,
    WorkbenchLexicalTotals, WorkbenchReverseDefinitionCompact,
    WorkbenchReverseDefinitionCompactGroup, WorkbenchReverseDefinitionFull,
    WorkbenchReverseDefinitionFullGroup,
};
pub use links::{
    ConfiguredWorkbenchLinkDetailResponse, WorkbenchCompactLinkRecord, WorkbenchLinkCandidate,
    WorkbenchLinkCompactStep, WorkbenchLinkDetailRef, WorkbenchLinkDetailResponse,
    WorkbenchLinkEvidenceId, WorkbenchLinkFullStep, WorkbenchLinkRecord, WorkbenchLinkStepId,
    WorkbenchLinkTargetCompact, WorkbenchLinkTargetFull, WorkbenchLinksCompact,
    WorkbenchLinksCompactGroup, WorkbenchLinksFull, WorkbenchLinksFullGroup,
    WorkbenchLinksSourceTotal, WorkbenchLinksTotals,
};
pub use reference::{
    ConfiguredWorkbenchReferenceCompactResponse, ConfiguredWorkbenchReferenceFullResponse,
    WorkbenchReferenceCompactResponse, WorkbenchReferenceCompactResult,
    WorkbenchReferenceFullResponse, WorkbenchReferenceFullResult, WorkbenchReferenceRegistration,
};

/// The small cached selection summary attached to every workbench query result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkbenchQuerySelection {
    /// Summary contract version.
    pub version: u32,
    /// Selected pack IDs in caller order.
    pub packs: Vec<String>,
    /// Disabled selected pack IDs.
    pub disabled: Vec<String>,
    /// Current upstream identity, without member/path inventory.
    #[serde(deserialize_with = "deserialize_required_option")]
    pub upstream: Option<WorkbenchUpstreamSummary>,
    /// Selected owned identity, without complete revision rosters.
    #[serde(deserialize_with = "deserialize_required_option")]
    pub owned: Option<WorkbenchOwnedSummary>,
    /// Declared and supplied curation context counts.
    pub context_counts: WorkbenchContextCounts,
    /// Curation outcome counts, absent for schema-one/no-owned views.
    #[serde(deserialize_with = "deserialize_required_option")]
    pub curation_counts: Option<WorkbenchCurationCounts>,
}

/// Small upstream identity included in a query envelope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkbenchUpstreamSummary {
    /// Catalogue-local revision.
    pub local_revision: u64,
    /// Installed generation identity.
    #[serde(deserialize_with = "deserialize_required_option")]
    pub generation_id: Option<String>,
    /// Installed content identity.
    #[serde(deserialize_with = "deserialize_required_option")]
    pub content_id: Option<String>,
    /// Recorded canonical catalogue digest.
    #[serde(deserialize_with = "deserialize_required_option")]
    pub state_sha256: Option<String>,
}

/// Small owned identity included in a query envelope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkbenchOwnedSummary {
    /// Requested Head or historical revision mode.
    pub mode: WorkbenchOwnedRevision,
    /// Sole owned source.
    pub source: String,
    /// Selected revision identity.
    pub revision: String,
    /// Selected sequence.
    pub sequence: u64,
    /// Persisted journal schema.
    pub journal_schema: u32,
    /// Base dictionary digest.
    pub state_sha256: String,
    /// Curation state digest, explicit null only for schema one.
    #[serde(deserialize_with = "deserialize_required_option")]
    pub curation_sha256: Option<String>,
}

/// Counts of retained declared and supplied context bindings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkbenchContextCounts {
    /// Number of pins declared by the selected curation.
    pub declared: usize,
    /// Number of pins for which input bodies were supplied.
    pub provided: usize,
}

/// Counts of materialized outcomes, retaining all four original statuses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkbenchCurationCounts {
    /// Total retained curation records.
    pub records: usize,
    /// Records whose normalized rows were projected.
    pub projected: usize,
    /// Inactive records.
    pub inactive: usize,
    /// Records missing a required context body.
    pub unavailable: usize,
    /// Records with supplied but insufficient context evidence.
    pub unresolved: usize,
    /// Fully resolved records.
    pub resolved: usize,
}

/// Full result operation tag and its existing complete typed response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "operation", content = "response", rename_all = "snake_case")]
pub enum WorkbenchFullResult {
    /// Exact spelling response.
    Exact(Box<DictionaryExactResponse>),
    /// Typed relation-target resolution.
    Resolve(Box<ReferenceResolution>),
}

/// Full workbench query envelope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkbenchFullResponse {
    /// Envelope contract version.
    pub version: u32,
    /// Outer workbench binding identity.
    pub binding_sha256: String,
    /// Small cached selection summary.
    pub selection: WorkbenchQuerySelection,
    /// Existing full typed query result.
    pub result: WorkbenchFullResult,
}

/// Compact workbench query envelope.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkbenchCompactResponse {
    /// Envelope contract version.
    pub version: u32,
    /// Outer workbench binding identity.
    pub binding_sha256: String,
    /// Small cached selection summary.
    pub selection: WorkbenchQuerySelection,
    /// Existing compact query result and its distinct inner snapshot identity.
    pub result: DictionaryCompactQueryResult,
}

/// Complete-hit detail envelope with a mandatory outer binding identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkbenchDetailResponse {
    /// Envelope contract version.
    pub version: u32,
    /// Outer workbench binding identity.
    pub binding_sha256: String,
    /// Small cached selection summary.
    pub selection: WorkbenchQuerySelection,
    /// Existing complete-hit response; its compact token remains distinct.
    pub result: DictionaryHitDetailResponse,
}

/// Configured Full response with both effective and captured config identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfiguredWorkbenchFullResponse {
    /// Response version.
    pub version: u32,
    /// Effective configured identity used by this query.
    pub binding_sha256: String,
    /// Exact captured config/profile identity.
    pub configuration: WorkbenchProfileIdentity,
    /// Small existing workbench selection summary.
    pub selection: WorkbenchQuerySelection,
    /// Existing operation-tagged Full response.
    pub result: WorkbenchFullResult,
}

/// Configured Compact response with the existing inner snapshot identity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfiguredWorkbenchCompactResponse {
    /// Response version.
    pub version: u32,
    /// Effective configured identity used by this query.
    pub binding_sha256: String,
    /// Exact captured config/profile identity.
    pub configuration: WorkbenchProfileIdentity,
    /// Small existing workbench selection summary.
    pub selection: WorkbenchQuerySelection,
    /// Existing compact result with its separate inner snapshot hash.
    pub result: DictionaryCompactQueryResult,
}

/// Configured detail response with the effective outer binding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfiguredWorkbenchDetailResponse {
    /// Response version.
    pub version: u32,
    /// Effective configured identity used by this detail lookup.
    pub binding_sha256: String,
    /// Exact captured config/profile identity.
    pub configuration: WorkbenchProfileIdentity,
    /// Small existing workbench selection summary.
    pub selection: WorkbenchQuerySelection,
    /// Existing complete detail body.
    pub result: DictionaryHitDetailResponse,
}

/// Config-file freshness or existing query failure for a configured engine.
#[derive(Debug)]
#[non_exhaustive]
pub enum ConfiguredWorkbenchQueryError {
    /// Explicit config/profile/source failure.
    Profile(Box<WorkbenchProfileError>),
    /// Existing query, projection, or detail failure.
    Query(Box<WorkbenchQueryError>),
}

impl fmt::Display for ConfiguredWorkbenchQueryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Profile(error) => write!(f, "configured workbench profile: {error}"),
            Self::Query(error) => write!(f, "configured workbench query: {error}"),
        }
    }
}

impl Error for ConfiguredWorkbenchQueryError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Profile(error) => Some(error.as_ref()),
            Self::Query(error) => Some(error.as_ref()),
        }
    }
}

/// Freshness, existing query, projection, or outer-key failure.
#[derive(Debug)]
#[non_exhaustive]
pub enum WorkbenchQueryError {
    /// A dependency changed or failed its freshness check.
    View(Box<WorkbenchError>),
    /// Existing full query engine failure.
    Query(Box<DictionaryQueryError>),
    /// Existing compact detail projection failure.
    Projection(Box<DictionaryProjectionError>),
    /// A new lexical request, cursor, detail key, or retained-record failure.
    Lexical(Box<WorkbenchLexicalError>),
    /// A retained catalogue descriptor could not be read or projected.
    Catalogue(Box<vlajmaji_store::CollectionError>),
    /// Matching retained rows or prepared bodies disagree for one role and ID.
    AmbiguousReference {
        /// Entity role whose retained and prepared rows disagree.
        role: String,
        /// Stable entity ID with inconsistent rows.
        id: String,
    },
    /// Caller key names a different workbench binding.
    StaleBinding {
        /// Binding identity supplied by the caller.
        expected: String,
        /// Binding identity stored in the selected workbench.
        observed: String,
    },
    /// Caller supplied a malformed outer binding digest.
    MalformedBinding {
        /// Invalid digest string supplied by the caller.
        value: String,
    },
}

impl fmt::Display for WorkbenchQueryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::View(error) => write!(f, "workbench freshness: {error}"),
            Self::Query(error) => write!(f, "workbench query: {error}"),
            Self::Projection(error) => write!(f, "workbench detail: {error}"),
            Self::Lexical(error) => write!(f, "workbench lexical query: {error}"),
            Self::Catalogue(error) => write!(f, "workbench catalogue: {error}"),
            Self::AmbiguousReference { role, id } => {
                write!(f, "ambiguous {role} reference {id}")
            }
            Self::StaleBinding { expected, observed } => write!(
                f,
                "detail belongs to workbench binding {expected}, not {observed}"
            ),
            Self::MalformedBinding { value } => {
                write!(f, "invalid workbench binding digest {value:?}")
            }
        }
    }
}

impl Error for WorkbenchQueryError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::View(error) => Some(error.as_ref()),
            Self::Query(error) => Some(error.as_ref()),
            Self::Projection(error) => Some(error.as_ref()),
            Self::Lexical(error) => Some(error.as_ref()),
            Self::Catalogue(error) => Some(error.as_ref()),
            _ => None,
        }
    }
}

impl From<WorkbenchError> for WorkbenchQueryError {
    fn from(value: WorkbenchError) -> Self {
        Self::View(Box::new(value))
    }
}
impl From<DictionaryQueryError> for WorkbenchQueryError {
    fn from(value: DictionaryQueryError) -> Self {
        Self::Query(Box::new(value))
    }
}
impl From<DictionaryProjectionError> for WorkbenchQueryError {
    fn from(value: DictionaryProjectionError) -> Self {
        Self::Projection(Box::new(value))
    }
}
impl From<WorkbenchLexicalError> for WorkbenchQueryError {
    fn from(value: WorkbenchLexicalError) -> Self {
        Self::Lexical(Box::new(value))
    }
}
impl From<vlajmaji_store::CollectionError> for WorkbenchQueryError {
    fn from(value: vlajmaji_store::CollectionError) -> Self {
        Self::Catalogue(Box::new(value))
    }
}

/// Reusable Full/Compact query engines bound to one immutable workbench.
pub struct WorkbenchSearchEngine {
    view: WorkbenchView,
    selection: WorkbenchQuerySelection,
    full: Option<DictionarySearchEngine>,
    compact: Option<DictionaryCompactSearchEngine>,
}

/// Reusable query engines bound to one captured configured profile.
pub struct ConfiguredWorkbenchSearchEngine {
    view: ConfiguredWorkbenchView,
    selection: WorkbenchQuerySelection,
    full: Option<DictionarySearchEngine>,
    compact: Option<DictionaryCompactSearchEngine>,
}

impl fmt::Debug for ConfiguredWorkbenchSearchEngine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ConfiguredWorkbenchSearchEngine")
            .field(
                "binding_sha256",
                &self.view.identity().effective_binding_sha256,
            )
            .field("full_initialized", &self.full.is_some())
            .field("compact_initialized", &self.compact.is_some())
            .finish()
    }
}

impl ConfiguredWorkbenchSearchEngine {
    /// Capture a named or default profile and retain lazy query engines.
    ///
    /// # Errors
    /// Returns config, profile, source, or binding errors.
    pub fn open(
        config: impl AsRef<Path>,
        profile: Option<&str>,
    ) -> Result<Self, ConfiguredWorkbenchQueryError> {
        let view = ConfiguredWorkbenchView::open(config, profile)
            .map_err(|error| ConfiguredWorkbenchQueryError::Profile(Box::new(error)))?;
        let selection = query_selection(view.view().binding());
        Ok(Self {
            view,
            selection,
            full: None,
            compact: None,
        })
    }

    /// Borrows the captured effective configuration identity.
    #[must_use]
    pub fn identity(&self) -> &WorkbenchProfileIdentity {
        self.view.identity()
    }

    /// Borrows the complete direct S01 binding.
    #[must_use]
    pub fn binding(&self) -> &WorkbenchBinding {
        self.view.view().binding()
    }

    /// Executes one Full Exact or Resolve request with config/source guards.
    ///
    /// # Errors
    /// Returns config/source freshness or existing dictionary query errors.
    pub fn execute(
        &mut self,
        request: DictionaryRequest,
    ) -> Result<ConfiguredWorkbenchFullResponse, ConfiguredWorkbenchQueryError> {
        self.execute_with_hook(request, || {})
    }

    fn execute_with_hook<F>(
        &mut self,
        request: DictionaryRequest,
        after_inner: F,
    ) -> Result<ConfiguredWorkbenchFullResponse, ConfiguredWorkbenchQueryError>
    where
        F: FnOnce(),
    {
        self.view
            .ensure_current()
            .map_err(|error| ConfiguredWorkbenchQueryError::Profile(Box::new(error)))?;
        let result = (|| {
            let engine = self.full_engine()?;
            match request {
                DictionaryRequest::Exact { spelling, options } => engine
                    .exact(&spelling, options)
                    .map(|response| WorkbenchFullResult::Exact(Box::new(response))),
                DictionaryRequest::Resolve { target } => engine
                    .resolve(&target)
                    .map(|response| WorkbenchFullResult::Resolve(Box::new(response))),
            }
        })();
        after_inner();
        let freshness = self.view.ensure_current();
        if let Err(error) = freshness {
            return Err(ConfiguredWorkbenchQueryError::Profile(Box::new(error)));
        }
        let result = result.map_err(|error| {
            ConfiguredWorkbenchQueryError::Query(Box::new(WorkbenchQueryError::Query(Box::new(
                error,
            ))))
        })?;
        Ok(ConfiguredWorkbenchFullResponse {
            version: 1,
            binding_sha256: self.view.identity().effective_binding_sha256.clone(),
            configuration: self.view.identity().clone(),
            selection: self.selection.clone(),
            result,
        })
    }

    /// Executes one Compact Exact or Resolve request with config/source guards.
    ///
    /// # Errors
    /// Returns config/source freshness or existing dictionary query errors.
    pub fn execute_compact(
        &mut self,
        request: DictionaryRequest,
    ) -> Result<ConfiguredWorkbenchCompactResponse, ConfiguredWorkbenchQueryError> {
        self.execute_compact_with_hook(request, || {})
    }

    fn execute_compact_with_hook<F>(
        &mut self,
        request: DictionaryRequest,
        after_inner: F,
    ) -> Result<ConfiguredWorkbenchCompactResponse, ConfiguredWorkbenchQueryError>
    where
        F: FnOnce(),
    {
        self.view
            .ensure_current()
            .map_err(|error| ConfiguredWorkbenchQueryError::Profile(Box::new(error)))?;
        let result = self
            .compact_engine()
            .and_then(|engine| engine.execute(request));
        after_inner();
        let freshness = self.view.ensure_current();
        if let Err(error) = freshness {
            return Err(ConfiguredWorkbenchQueryError::Profile(Box::new(error)));
        }
        let result = result.map_err(|error| {
            ConfiguredWorkbenchQueryError::Query(Box::new(WorkbenchQueryError::Query(Box::new(
                error,
            ))))
        })?;
        Ok(ConfiguredWorkbenchCompactResponse {
            version: 1,
            binding_sha256: self.view.identity().effective_binding_sha256.clone(),
            configuration: self.view.identity().clone(),
            selection: self.selection.clone(),
            result,
        })
    }

    /// Resolves detail using the same cached effective-generation Compact engine.
    ///
    /// # Errors
    /// Returns config/source freshness, outer binding, or existing detail errors.
    pub fn detail(
        &mut self,
        reference: &DictionaryHitDetailRef,
        expected_effective_binding: &str,
    ) -> Result<ConfiguredWorkbenchDetailResponse, ConfiguredWorkbenchQueryError> {
        self.detail_with_hook(reference, expected_effective_binding, || {})
    }

    fn detail_with_hook<F>(
        &mut self,
        reference: &DictionaryHitDetailRef,
        expected_effective_binding: &str,
        after_inner: F,
    ) -> Result<ConfiguredWorkbenchDetailResponse, ConfiguredWorkbenchQueryError>
    where
        F: FnOnce(),
    {
        self.view
            .ensure_current()
            .map_err(|error| ConfiguredWorkbenchQueryError::Profile(Box::new(error)))?;
        let expected = expected_effective_binding.to_owned();
        let result = (|| {
            if !is_sha256(&expected) {
                return Err(WorkbenchQueryError::MalformedBinding {
                    value: expected.clone(),
                });
            }
            let observed = &self.view.identity().effective_binding_sha256;
            if &expected != observed {
                return Err(WorkbenchQueryError::StaleBinding {
                    expected: expected.clone(),
                    observed: observed.clone(),
                });
            }
            self.compact_engine()?
                .detail(reference, None)
                .map_err(Into::into)
        })();
        after_inner();
        let freshness = self.view.ensure_current();
        if let Err(error) = freshness {
            return Err(ConfiguredWorkbenchQueryError::Profile(Box::new(error)));
        }
        let result =
            result.map_err(|error| ConfiguredWorkbenchQueryError::Query(Box::new(error)))?;
        Ok(ConfiguredWorkbenchDetailResponse {
            version: 1,
            binding_sha256: self.view.identity().effective_binding_sha256.clone(),
            configuration: self.view.identity().clone(),
            selection: self.selection.clone(),
            result,
        })
    }

    fn full_engine(&mut self) -> Result<&mut DictionarySearchEngine, DictionaryQueryError> {
        if self.full.is_none() {
            self.full = Some(DictionarySearchEngine::open_snapshot(
                DictionaryReadTopology::Unified,
                self.view.snapshot_manifest(),
            )?);
        }
        Ok(self.full.as_mut().expect("full engine initialized"))
    }

    fn compact_engine(
        &mut self,
    ) -> Result<&mut DictionaryCompactSearchEngine, DictionaryQueryError> {
        if self.compact.is_none() {
            self.compact = Some(DictionaryCompactSearchEngine::open_snapshot(
                DictionaryReadTopology::Unified,
                self.view.snapshot_manifest(),
            )?);
        }
        Ok(self.compact.as_mut().expect("compact engine initialized"))
    }
}

impl fmt::Debug for WorkbenchSearchEngine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WorkbenchSearchEngine")
            .field("binding_sha256", &self.view.binding().binding_sha256())
            .field("full_initialized", &self.full.is_some())
            .field("compact_initialized", &self.compact.is_some())
            .finish()
    }
}

impl WorkbenchSearchEngine {
    /// Open a workbench and retain only its checked binding until a route is used.
    ///
    /// # Errors
    ///
    /// Returns binding, IO, or stale-dependency errors.
    pub fn open(root: impl AsRef<Path>) -> Result<Self, WorkbenchQueryError> {
        let view = WorkbenchView::open(root)?;
        let selection = query_selection(view.binding());
        Ok(Self {
            view,
            selection,
            full: None,
            compact: None,
        })
    }

    /// Borrow the complete persisted binding for info or diagnostics.
    #[must_use]
    pub fn binding(&self) -> &WorkbenchBinding {
        self.view.binding()
    }

    /// Execute one full Exact or Resolve request with before/after freshness checks.
    ///
    /// # Errors
    ///
    /// Returns stale-dependency or existing dictionary query errors.
    pub fn execute(
        &mut self,
        request: DictionaryRequest,
    ) -> Result<WorkbenchFullResponse, WorkbenchQueryError> {
        self.execute_with_hook(request, || {})
    }

    fn execute_with_hook<F>(
        &mut self,
        request: DictionaryRequest,
        after_inner: F,
    ) -> Result<WorkbenchFullResponse, WorkbenchQueryError>
    where
        F: FnOnce(),
    {
        self.view.ensure_current()?;
        let result = (|| {
            let engine = self.full_engine()?;
            match request {
                DictionaryRequest::Exact { spelling, options } => engine
                    .exact(&spelling, options)
                    .map(|response| WorkbenchFullResult::Exact(Box::new(response))),
                DictionaryRequest::Resolve { target } => engine
                    .resolve(&target)
                    .map(|response| WorkbenchFullResult::Resolve(Box::new(response))),
            }
        })();
        after_inner();
        let freshness = self.view.ensure_current();
        match freshness {
            Err(error) => Err(error.into()),
            Ok(()) => {
                let result = result?;
                Ok(WorkbenchFullResponse {
                    version: 1,
                    binding_sha256: self.view.binding().binding_sha256().to_owned(),
                    selection: self.selection.clone(),
                    result,
                })
            }
        }
    }

    /// Execute one existing compact Exact or Resolve request with freshness checks.
    ///
    /// # Errors
    ///
    /// Returns stale-dependency or existing dictionary query errors.
    pub fn execute_compact(
        &mut self,
        request: DictionaryRequest,
    ) -> Result<WorkbenchCompactResponse, WorkbenchQueryError> {
        self.execute_compact_with_hook(request, || {})
    }

    fn execute_compact_with_hook<F>(
        &mut self,
        request: DictionaryRequest,
        after_inner: F,
    ) -> Result<WorkbenchCompactResponse, WorkbenchQueryError>
    where
        F: FnOnce(),
    {
        self.view.ensure_current()?;
        let result = self
            .compact_engine()
            .and_then(|engine| engine.execute(request));
        after_inner();
        let freshness = self.view.ensure_current();
        match freshness {
            Err(error) => Err(error.into()),
            Ok(()) => Ok(WorkbenchCompactResponse {
                version: 1,
                binding_sha256: self.view.binding().binding_sha256().to_owned(),
                selection: self.selection.clone(),
                result: result?,
            }),
        }
    }

    /// Resolve a complete hit after checking the mandatory outer binding key.
    ///
    /// # Errors
    ///
    /// Returns stale-dependency, outer binding, or existing detail-key errors.
    pub fn detail(
        &mut self,
        reference: &DictionaryHitDetailRef,
        expected_binding: &str,
    ) -> Result<WorkbenchDetailResponse, WorkbenchQueryError> {
        self.detail_with_hook(reference, expected_binding, || {})
    }

    fn detail_with_hook<F>(
        &mut self,
        reference: &DictionaryHitDetailRef,
        expected_binding: &str,
        after_inner: F,
    ) -> Result<WorkbenchDetailResponse, WorkbenchQueryError>
    where
        F: FnOnce(),
    {
        self.view.ensure_current()?;
        let expected = expected_binding.to_owned();
        let result = (|| {
            if !is_sha256(&expected) {
                return Err(WorkbenchQueryError::MalformedBinding {
                    value: expected.clone(),
                });
            }
            let observed = self.view.binding().binding_sha256();
            if expected != observed {
                return Err(WorkbenchQueryError::StaleBinding {
                    expected: expected.clone(),
                    observed: observed.to_owned(),
                });
            }
            self.compact_engine()?
                .detail(reference, None)
                .map_err(Into::into)
        })();
        after_inner();
        let freshness = self.view.ensure_current();
        match freshness {
            Err(error) => Err(error.into()),
            Ok(()) => Ok(WorkbenchDetailResponse {
                version: 1,
                binding_sha256: self.view.binding().binding_sha256().to_owned(),
                selection: self.selection.clone(),
                result: result?,
            }),
        }
    }

    fn full_engine(&mut self) -> Result<&mut DictionarySearchEngine, DictionaryQueryError> {
        if self.full.is_none() {
            self.full = Some(DictionarySearchEngine::open_snapshot(
                DictionaryReadTopology::Unified,
                self.view.snapshot_manifest(),
            )?);
        }
        Ok(self.full.as_mut().expect("full engine initialized"))
    }

    fn compact_engine(
        &mut self,
    ) -> Result<&mut DictionaryCompactSearchEngine, DictionaryQueryError> {
        if self.compact.is_none() {
            self.compact = Some(DictionaryCompactSearchEngine::open_snapshot(
                DictionaryReadTopology::Unified,
                self.view.snapshot_manifest(),
            )?);
        }
        Ok(self.compact.as_mut().expect("compact engine initialized"))
    }
}

fn query_selection(binding: &WorkbenchBinding) -> WorkbenchQuerySelection {
    let upstream = binding
        .catalogue()
        .map(|catalogue| WorkbenchUpstreamSummary {
            local_revision: catalogue.local_revision,
            generation_id: catalogue.generation_id.clone(),
            content_id: catalogue.content_id.clone(),
            state_sha256: catalogue.recorded_state_sha256.clone(),
        });
    let owned = binding.owned().map(|owned| WorkbenchOwnedSummary {
        mode: owned.selection.clone(),
        source: owned.identity.source.to_string(),
        revision: owned.identity.revision.to_string(),
        sequence: owned.identity.sequence,
        journal_schema: owned.identity.journal_schema,
        state_sha256: owned.identity.state_sha256.clone(),
        curation_sha256: owned.identity.curation_state_sha256.clone(),
    });
    let curation_counts = binding
        .owned()
        .filter(|owned| owned.identity.curation_state_sha256.is_some())
        .map(|_| {
            let mut counts = WorkbenchCurationCounts {
                records: binding.resolutions().len(),
                projected: 0,
                inactive: 0,
                unavailable: 0,
                unresolved: 0,
                resolved: 0,
            };
            for resolution in binding.resolutions() {
                if resolution.projected {
                    counts.projected += 1;
                }
                match resolution.status {
                    OwnedCurationStatus::Inactive => counts.inactive += 1,
                    OwnedCurationStatus::Unavailable => counts.unavailable += 1,
                    OwnedCurationStatus::Unresolved => counts.unresolved += 1,
                    OwnedCurationStatus::Resolved => counts.resolved += 1,
                }
            }
            counts
        });
    WorkbenchQuerySelection {
        version: 1,
        packs: binding
            .selection()
            .packs
            .iter()
            .map(ToString::to_string)
            .collect(),
        disabled: binding
            .selection()
            .disabled
            .iter()
            .map(ToString::to_string)
            .collect(),
        upstream,
        owned,
        context_counts: WorkbenchContextCounts {
            declared: binding.contexts().len(),
            provided: binding
                .contexts()
                .iter()
                .filter(|row| row.packs.is_some())
                .count(),
        },
        curation_counts,
    }
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn deserialize_required_option<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::de::DeserializeOwned,
{
    Option::<T>::deserialize(deserializer)
}

#[cfg(test)]
mod tests {
    use std::{
        borrow::Cow,
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use vlajmaji_lexicon::{
        CaptureRef, Dictionary, OccurrenceRef, PackId, PackPayloadKind, seal_release,
    };
    use vlajmaji_store::{
        CollectionCatalogue, CollectionInput, CollectionRequest, CollectionSelection,
        WorkbenchBuildRequest, WorkbenchDependency, WorkbenchError,
        WorkbenchProfileConfigureRequest, WorkbenchProfileError, WorkbenchProfiles,
        WorkbenchSelection, WorkbenchView,
    };

    use crate::DictionaryExactOptions;

    use super::*;

    fn fresh_root() -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "vlajmaji-workbench-query-{}-{stamp}",
            std::process::id()
        ))
    }

    fn catalogue_write(root: &std::path::Path, pack: PackId, install: bool) {
        let dictionary = Dictionary::from_document(Default::default()).unwrap();
        let sealed = seal_release(pack, PackPayloadKind::Synthetic, &dictionary).unwrap();
        let request = CollectionRequest::new(
            vec![CollectionInput::SealedBytes {
                pack_id: pack,
                release: Cow::Borrowed(&sealed.release),
                seal: Cow::Borrowed(&sealed.seal),
            }],
            CollectionSelection::Packs(vec![pack]),
            true,
        )
        .unwrap();
        if install {
            CollectionCatalogue::install(root, request).unwrap();
        } else {
            CollectionCatalogue::register(root, request).unwrap();
        }
    }

    #[test]
    fn test_execute_suppresses_inner_response_when_catalogue_changes_after_query() {
        let parent = fresh_root();
        fs::create_dir(&parent).unwrap();
        let catalogue = parent.join("catalogue");
        catalogue_write(&catalogue, PackId::Llg, true);
        let view_root = parent.join("view");
        WorkbenchView::build_new(
            &view_root,
            WorkbenchBuildRequest {
                catalogue_root: Some(catalogue.clone()),
                owned_root: None,
                selection: WorkbenchSelection {
                    version: 1,
                    packs: vec![PackId::Llg],
                    disabled: Vec::new(),
                    owned: None,
                    contexts: Vec::new(),
                },
            },
        )
        .unwrap();
        let mut engine = WorkbenchSearchEngine::open(&view_root).unwrap();
        assert!(engine.full.is_none());
        assert!(engine.compact.is_none());
        let result = engine.execute_with_hook(
            DictionaryRequest::Exact {
                spelling: "absent".into(),
                options: crate::DictionaryExactOptions::default(),
            },
            || catalogue_write(&catalogue, PackId::LaLojban, false),
        );
        assert!(matches!(
            result,
            Err(WorkbenchQueryError::View(error))
                if matches!(error.as_ref(), WorkbenchError::StaleDependencies {
                    dependency: WorkbenchDependency::Catalogue,
                    ..
                })
        ));
        assert!(engine.full.is_some());
        assert!(engine.compact.is_none());
        fs::remove_dir_all(parent).unwrap();
    }

    #[test]
    fn test_compact_and_detail_suppress_inner_results_when_catalogue_changes() {
        let parent = fresh_root();
        fs::create_dir(&parent).unwrap();
        let catalogue = parent.join("catalogue");
        catalogue_write(&catalogue, PackId::Llg, true);
        let view_root = parent.join("compact-view");
        WorkbenchView::build_new(
            &view_root,
            WorkbenchBuildRequest {
                catalogue_root: Some(catalogue.clone()),
                owned_root: None,
                selection: WorkbenchSelection {
                    version: 1,
                    packs: vec![PackId::Llg],
                    disabled: Vec::new(),
                    owned: None,
                    contexts: Vec::new(),
                },
            },
        )
        .unwrap();
        let mut engine = WorkbenchSearchEngine::open(&view_root).unwrap();
        assert!(engine.full.is_none());
        assert!(engine.compact.is_none());
        let compact = engine.execute_compact_with_hook(
            DictionaryRequest::Exact {
                spelling: "absent".into(),
                options: crate::DictionaryExactOptions::default(),
            },
            || catalogue_write(&catalogue, PackId::LaLojban, false),
        );
        assert!(matches!(
            compact,
            Err(WorkbenchQueryError::View(error))
                if matches!(error.as_ref(), WorkbenchError::StaleDependencies {
                    dependency: WorkbenchDependency::Catalogue,
                    ..
                })
        ));
        assert!(engine.full.is_none());
        assert!(engine.compact.is_some());

        fs::remove_dir_all(&parent).unwrap();
        let parent = fresh_root();
        fs::create_dir(&parent).unwrap();
        let catalogue = parent.join("catalogue");
        catalogue_write(&catalogue, PackId::Llg, true);
        let view_root = parent.join("detail-view");
        let report = WorkbenchView::build_new(
            &view_root,
            WorkbenchBuildRequest {
                catalogue_root: Some(catalogue.clone()),
                owned_root: None,
                selection: WorkbenchSelection {
                    version: 1,
                    packs: vec![PackId::Llg],
                    disabled: Vec::new(),
                    owned: None,
                    contexts: Vec::new(),
                },
            },
        )
        .unwrap();
        let mut engine = WorkbenchSearchEngine::open(&view_root).unwrap();
        assert!(engine.full.is_none());
        assert!(engine.compact.is_none());
        let reference = DictionaryHitDetailRef {
            version: 1,
            digest_policy: "entry-assertion-serde-json-v1".into(),
            role: "assertion".into(),
            source: vlajmaji_lexicon::SourceId::new("source-a").unwrap(),
            entry_id: vlajmaji_lexicon::EntityId::new("entry-a").unwrap(),
            assertion_id: vlajmaji_lexicon::EntityId::new("assertion-a").unwrap(),
            occurrence: OccurrenceRef {
                capture: CaptureRef {
                    namespace: "fixture:workbench".into(),
                    sha256: "0".repeat(64),
                },
                locator: "/0".into(),
            },
            entry_sha256: "1".repeat(64),
            assertion_sha256: "2".repeat(64),
            native_entry_id: None,
            native_assertion_id: None,
            contributor_id: None,
            distributor_id: None,
        };
        let detail = engine.detail_with_hook(&reference, report.binding.binding_sha256(), || {
            catalogue_write(&catalogue, PackId::LaLojban, false)
        });
        assert!(matches!(
            detail,
            Err(WorkbenchQueryError::View(error))
                if matches!(error.as_ref(), WorkbenchError::StaleDependencies {
                    dependency: WorkbenchDependency::Catalogue,
                    ..
                })
        ));
        assert!(engine.full.is_none());
        assert!(engine.compact.is_some());
        fs::remove_dir_all(parent).unwrap();
    }

    #[test]
    fn test_configured_query_config_change_suppresses_inner_error() {
        let parent = fresh_root();
        fs::create_dir(&parent).unwrap();
        let catalogue = parent.join("catalogue");
        catalogue_write(&catalogue, PackId::Llg, true);
        let selection = WorkbenchSelection {
            version: 1,
            packs: vec![PackId::Llg],
            disabled: Vec::new(),
            owned: None,
            contexts: Vec::new(),
        };
        let config = parent.join("profiles.json");
        WorkbenchProfiles::configure(
            &config,
            WorkbenchProfileConfigureRequest {
                name: "alpha".into(),
                catalogue_root: Some(catalogue.clone()),
                owned_root: None,
                selection: selection.clone(),
                make_default: true,
            },
            None,
        )
        .unwrap();
        WorkbenchProfiles::refresh(&config, None, parent.join("prepared-alpha"), 1).unwrap();
        WorkbenchProfiles::configure(
            &config,
            WorkbenchProfileConfigureRequest {
                name: "beta".into(),
                catalogue_root: Some(catalogue),
                owned_root: None,
                selection,
                make_default: false,
            },
            Some(2),
        )
        .unwrap();
        let mut engine = ConfiguredWorkbenchSearchEngine::open(&config, Some("alpha")).unwrap();
        assert_eq!(engine.identity().config_revision, 3);
        let unchanged_error = engine
            .execute(DictionaryRequest::Exact {
                spelling: "absent".into(),
                options: DictionaryExactOptions {
                    cursor: Some("malformed-cursor".into()),
                    ..DictionaryExactOptions::default()
                },
            })
            .unwrap_err();
        assert!(matches!(
            unchanged_error,
            ConfiguredWorkbenchQueryError::Query(error)
                if matches!(error.as_ref(), WorkbenchQueryError::Query(inner)
                    if matches!(inner.as_ref(), DictionaryQueryError::MalformedCursor { .. }))
        ));
        let result = engine.execute_with_hook(
            DictionaryRequest::Exact {
                spelling: "absent".into(),
                options: DictionaryExactOptions {
                    cursor: Some("malformed-cursor".into()),
                    ..DictionaryExactOptions::default()
                },
            },
            || {
                let changed = WorkbenchProfiles::set_default(&config, Some("beta"), 3).unwrap();
                assert!(changed.changed);
                assert_eq!(changed.revision, 4);
            },
        );
        assert!(matches!(
            result,
            Err(ConfiguredWorkbenchQueryError::Profile(error))
                if matches!(error.as_ref(), WorkbenchProfileError::ConfigChanged { .. })
        ));
        fs::remove_dir_all(parent).unwrap();
    }

    #[test]
    fn test_reference_catalogue_write_after_inner_error_returns_stale_dependencies() {
        let parent = fresh_root();
        fs::create_dir(&parent).unwrap();
        let catalogue = parent.join("catalogue");
        catalogue_write(&catalogue, PackId::Llg, true);
        let view_root = parent.join("view");
        WorkbenchView::build_new(
            &view_root,
            WorkbenchBuildRequest {
                catalogue_root: Some(catalogue.clone()),
                owned_root: None,
                selection: WorkbenchSelection {
                    version: 1,
                    packs: vec![PackId::Llg],
                    disabled: Vec::new(),
                    owned: None,
                    contexts: Vec::new(),
                },
            },
        )
        .unwrap();
        let mut engine = WorkbenchSearchEngine::open(&view_root).unwrap();
        let invalid = vlajmaji_lexicon::RelationTarget::Unresolved(String::new());
        let result = engine.resolve_reference_with_hook(&invalid, || {
            let prior_revision = {
                let reader = CollectionCatalogue::open(&catalogue).unwrap();
                reader.local_revision().unwrap()
            };
            let rebuilt = CollectionCatalogue::rebuild(&catalogue, prior_revision).unwrap();
            assert!(rebuilt.ok);
            assert_eq!(rebuilt.local_revision, prior_revision + 1);
        });
        assert!(matches!(
            result,
            Err(WorkbenchQueryError::View(error))
                if matches!(error.as_ref(), WorkbenchError::StaleDependencies {
                    dependency: WorkbenchDependency::Catalogue,
                    ..
                })
        ));
        fs::remove_dir_all(parent).unwrap();
    }

    #[test]
    fn test_configured_reference_writer_suppresses_inner_success() {
        let parent = fresh_root();
        fs::create_dir(&parent).unwrap();
        let catalogue = parent.join("catalogue");
        catalogue_write(&catalogue, PackId::Llg, true);
        let selection = WorkbenchSelection {
            version: 1,
            packs: vec![PackId::Llg],
            disabled: Vec::new(),
            owned: None,
            contexts: Vec::new(),
        };
        let config = parent.join("profiles.json");
        WorkbenchProfiles::configure(
            &config,
            WorkbenchProfileConfigureRequest {
                name: "alpha".into(),
                catalogue_root: Some(catalogue.clone()),
                owned_root: None,
                selection: selection.clone(),
                make_default: true,
            },
            None,
        )
        .unwrap();
        WorkbenchProfiles::refresh(&config, None, parent.join("prepared-alpha"), 1).unwrap();
        WorkbenchProfiles::configure(
            &config,
            WorkbenchProfileConfigureRequest {
                name: "beta".into(),
                catalogue_root: Some(catalogue),
                owned_root: None,
                selection,
                make_default: false,
            },
            Some(2),
        )
        .unwrap();
        let mut engine = ConfiguredWorkbenchSearchEngine::open(&config, Some("alpha")).unwrap();
        assert_eq!(engine.identity().config_revision, 3);
        let target = vlajmaji_lexicon::RelationTarget::Unknown;
        assert_eq!(
            engine.resolve_reference(&target).unwrap().result.status,
            crate::ReferenceStatus::Unknown
        );
        let result = engine.resolve_reference_with_hook(&target, || {
            let changed = WorkbenchProfiles::set_default(&config, Some("beta"), 3).unwrap();
            assert!(changed.changed);
            assert_eq!(changed.revision, 4);
        });
        assert!(matches!(
            result,
            Err(ConfiguredWorkbenchQueryError::Profile(error))
                if matches!(error.as_ref(), WorkbenchProfileError::ConfigChanged {
                    ..
                })
        ));
        fs::remove_dir_all(parent).unwrap();
    }
}
