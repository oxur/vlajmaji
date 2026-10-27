//! Managed exact/reference queries over an explicitly selected local catalogue.
//!
//! This wrapper captures policy separately from source records. It does not
//! change the existing explicit-STORE query route or authenticate payload bytes.

mod projection;
pub use projection::{
    CollectionCompactQueryResponse, CollectionCompactQueryResult,
    CollectionCompactReferenceResponse, CollectionCompactSearchEngine, CollectionHitDetailResponse,
    CollectionProjectionProfile,
};

use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fmt,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use vlajmaji_lexicon::{Availability, CaptureRef, EntityId, LocalRef, PackId, RelationTarget};
use vlajmaji_store::{
    CollectionCatalogue, CollectionError, CollectionGeneration, CollectionProfileError,
    CollectionProfileIdentity, CollectionProfiles, CollectionReleaseSummary,
    DictionaryReadTopology, DictionaryStore, DictionaryStoreError, DictionaryStoreInvalidity,
};

use crate::{
    DictionaryExactResponse, DictionaryQueryError, DictionaryRequest, DictionarySearchEngine,
    ReferenceResolution, ReferenceStatus,
};

/// Explicit pack membership and independently disabled packs for a managed query.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CollectionQuerySelection {
    /// Default or explicitly overridden pack membership, in caller order.
    pub packs: Vec<PackId>,
    /// Registered packs disabled independently of membership.
    pub disabled: Vec<PackId>,
    /// Exact profile identity, when loaded from a config file.
    pub profile: Option<CollectionProfileIdentity>,
}

/// Availability of one registered pack in the captured policy/catalogue view.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CollectionPackAvailability {
    /// Stable pack ID.
    pub pack_id: String,
    /// Whether the pack is a member of the configured/default set.
    pub default_member: bool,
    /// Whether a caller explicitly selected the pack.
    pub request_selected: bool,
    /// Whether the pack is independently disabled.
    pub disabled: bool,
    /// Current installed release digest, if present.
    pub installed_release_sha256: Option<String>,
    /// Whether the installed release has a producer-selected available release.
    pub producer_selectable: bool,
    /// Minimum derived-store observation captured by the catalogue.
    pub derived_state: String,
    /// Whether every original input path currently resolves.
    pub original_inputs_present: Option<bool>,
    /// Whether the currently installed release/seal/report copies are present.
    /// This observation is advisory and is excluded from cursor binding.
    pub owned_files_present: Option<bool>,
    /// Maintenance warning for missing tool-owned copies, when present.
    pub owned_files_warning: Option<String>,
    /// Ownership of the tool-installed release/seal copies.
    pub installed_copy_ownership: String,
}

/// Immutable operational view used to bind a managed cursor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CollectionSelectionSnapshot {
    /// Snapshot report schema version.
    pub version: u32,
    /// Explicit profile identity, if configured.
    pub profile: Option<CollectionProfileIdentity>,
    /// Catalogue-local revision.
    pub local_revision: u64,
    /// Physical Unified generation identity, if installed.
    pub generation_id: Option<String>,
    /// Data-derived generation identity, if installed.
    pub content_id: Option<String>,
    /// Catalogue-recorded logical digest; ordinary open does not authenticate it.
    pub recorded_state_sha256: Option<String>,
    /// Membership order, including disabled IDs.
    pub packs: Vec<PackId>,
    /// Independent disabled IDs.
    pub disabled: Vec<PackId>,
    /// Selected pack-to-release identity mapping.
    pub selected_releases: BTreeMap<String, Vec<String>>,
    /// Ordered pack status summary.
    pub collections: Vec<CollectionPackAvailability>,
    /// Query ordering/policy version.
    pub selection_policy: String,
    /// Stable exact-result ordering version.
    pub order_policy: String,
    /// Stable result projection version.
    pub projection_policy: String,
    /// Stable canonical snapshot binding.
    pub binding_sha256: String,
}

/// One metadata-only descriptor match; it is not a validated dictionary entity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CollectionReferenceMatch {
    /// Pack whose retained descriptor matched.
    pub pack_id: String,
    /// Sealed release digest.
    pub release_sha256: String,
    /// Model source owning the matching release/entity.
    pub source_id: String,
    /// Release ID in the source model.
    pub model_release_id: String,
    /// Role-qualified local entity ID, if applicable.
    pub role: Option<String>,
    /// Entity ID, if applicable.
    pub entity_id: Option<String>,
    /// Occurrence locator retained by the descriptor, if applicable.
    pub occurrence: Option<String>,
    /// Capture identity corresponding to the occurrence, when present.
    pub capture: Option<CaptureRef>,
    /// Native entry identity, when retained for the occurrence.
    pub native_entry_id: Option<String>,
    /// Native assertion identity, when retained for the occurrence.
    pub native_assertion_id: Option<String>,
    /// Packs retaining an exactly equal shared local entity.
    pub contributing_pack_ids: Vec<String>,
}

/// Publicly deduplicated matches plus the registration scopes used for admission.
/// Equal public bodies may belong to several packs and releases, so those scopes
/// must survive until selection and release filtering are complete.
struct MetadataMatches {
    public: Vec<CollectionReferenceMatch>,
    admission: Vec<CollectionReferenceMatch>,
    requested_release: Option<CollectionReleaseSummary>,
}

/// Managed wrapper around an existing full resolution or a metadata-only match.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CollectionReferenceResponse {
    /// Original caller target.
    pub target: RelationTarget,
    /// Existing resolution status.
    pub status: ReferenceStatus,
    /// Why an otherwise known reference has no available payload.
    pub operational_reason: Option<String>,
    /// Existing full response only when selected payload was admitted.
    pub payload: Option<ReferenceResolution>,
    /// Exact metadata-only matches retained by registration descriptors.
    pub catalogue_matches: Vec<CollectionReferenceMatch>,
    /// Requested release metadata for a catalog target, when known.
    pub requested_release: Option<CollectionReleaseSummary>,
}

/// Version-1 managed query response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "operation", content = "response", rename_all = "snake_case")]
pub enum CollectionQueryResult {
    /// Complete existing dictionary exact response.
    Exact(Box<DictionaryExactResponse>),
    /// Reference response, full or metadata-only.
    Resolve(Box<CollectionReferenceResponse>),
}

/// Complete envelope returned only by the managed configured route.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CollectionQueryResponse {
    /// Response schema version.
    pub version: u32,
    /// Captured operational identity.
    pub snapshot: CollectionSelectionSnapshot,
    /// Per-pack availability and selection facts.
    pub collections: Vec<CollectionPackAvailability>,
    /// Operation result.
    pub result: CollectionQueryResult,
}

/// Stable version-1 runtime error envelope for managed commands.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CollectionQueryErrorReport {
    /// Error schema version.
    pub version: u32,
    /// Managed failures are not successful responses.
    pub ok: bool,
    /// Stable category token.
    pub category: String,
    /// Human-readable detail.
    pub detail: String,
    /// Managed operation.
    pub operation: String,
    /// Native configuration path, when known.
    pub config_path: Option<vlajmaji_store::NativePathReport>,
    /// Native collection catalogue root, when known.
    pub root_path: Option<vlajmaji_store::NativePathReport>,
    /// Captured profile identity, when known.
    pub profile: Option<CollectionProfileIdentity>,
    /// Profile name supplied to a writer/read command before an identity was captured.
    pub requested_profile: Option<String>,
    /// Captured operational snapshot, when the request passed snapshot capture.
    pub snapshot: Option<CollectionSelectionSnapshot>,
    /// Registered pack IDs known at failure time.
    pub known_pack_ids: Vec<String>,
    /// Read failures have no activation effect.
    pub outcome: String,
    /// Known configuration writer revision/hash context after an uncertain publication.
    pub writer: Option<CollectionProfileWriteFailureContext>,
}

/// Prior/proposed/observed identities retained for an indeterminate config write.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CollectionProfileWriteFailureContext {
    /// Revision before the attempted write, if a config existed.
    pub prior_revision: Option<u64>,
    /// Revision encoded in the staged replacement.
    pub proposed_revision: u64,
    /// Exact prior file digest, if present.
    pub prior_sha256: Option<String>,
    /// Exact proposed file digest.
    pub proposed_sha256: String,
    /// Digest observed after the uncertain publication, if readable.
    pub observed_sha256: Option<String>,
    /// Revision observed in a valid file after uncertain publication, if any.
    pub observed_revision: Option<u64>,
}

/// Typed failure from configured policy capture or managed execution.
#[derive(Debug)]
#[non_exhaustive]
pub enum CollectionQueryError {
    /// Explicit profile decoding or selection failed.
    Profile(CollectionProfileError),
    /// Catalogue state could not be captured.
    Catalogue(CollectionError),
    /// Existing full query engine failed.
    Query(DictionaryQueryError),
    /// Compact detail-key validation or lookup failed.
    Projection(crate::DictionaryProjectionError),
    /// Selection refers to an unknown or duplicate pack.
    InvalidSelection(String),
    /// The requested profile exists, but explicitly selects a disabled pack.
    DisabledSelection {
        /// Pack explicitly requested despite being disabled.
        pack_id: String,
    },
    /// Snapshot changed before a configured view could be returned.
    ConfigChanged,
    /// Managed cursor is malformed or belongs to a different captured snapshot.
    StaleCursor,
    /// Managed cursor structure, encoding, or checksum is malformed.
    MalformedCursor,
    /// No enabled pack is selected.
    NoEnabledPacks,
    /// A local identity has incompatible metadata across retained registrations.
    AmbiguousReference,
    /// An enabled selected pack cannot participate in exact lookup.
    UnavailablePack {
        /// Pack that prevents the exact operation.
        pack_id: String,
        /// Current registered/install/payload reason.
        reason: String,
    },
}

impl CollectionQueryError {
    /// Convert an error into the stable managed-command envelope.
    #[must_use]
    pub fn report(&self) -> CollectionQueryErrorReport {
        self.report_for("lookup")
    }

    /// Convert an error into the stable envelope for its actual managed operation.
    #[must_use]
    pub fn report_for(&self, operation: &str) -> CollectionQueryErrorReport {
        let category = match self {
            Self::Profile(CollectionProfileError::Busy(_)) => "config_busy",
            Self::Profile(CollectionProfileError::UnsupportedVersion(_)) => "config_version",
            Self::Profile(CollectionProfileError::NoDefaultProfile { .. }) => {
                "profile_default_missing"
            }
            Self::Profile(CollectionProfileError::UnknownProfile { .. }) => "unknown_profile",
            Self::Profile(CollectionProfileError::NotProfileMember { .. }) => "not_profile_member",
            Self::Profile(CollectionProfileError::MissingFile { .. }) => "config_missing",
            Self::Profile(CollectionProfileError::CurrentDirectory { .. }) => "config_io",
            Self::Profile(CollectionProfileError::Io { source, .. })
                if source.kind() == std::io::ErrorKind::NotFound =>
            {
                "config_io"
            }
            Self::Profile(CollectionProfileError::Io { .. }) => "config_io",
            Self::Profile(CollectionProfileError::Indeterminate { .. }) => "config_indeterminate",
            Self::Profile(_) => "config_invalid",
            Self::Catalogue(_) => "catalogue",
            Self::Query(error) => {
                return CollectionQueryErrorReport {
                    version: 1,
                    ok: false,
                    category: error.report().code,
                    detail: self.to_string(),
                    operation: operation.into(),
                    config_path: None,
                    root_path: None,
                    profile: None,
                    requested_profile: None,
                    snapshot: None,
                    known_pack_ids: Vec::new(),
                    outcome: if matches!(
                        self,
                        Self::Profile(CollectionProfileError::Indeterminate { .. })
                    ) {
                        "indeterminate"
                    } else if operation == "configure" {
                        "unchanged"
                    } else {
                        "not_applicable"
                    }
                    .into(),
                    writer: profile_writer_context(self),
                };
            }
            Self::Projection(crate::DictionaryProjectionError::Query(_)) => "query",
            Self::Projection(
                crate::DictionaryProjectionError::ReferenceJson { .. }
                | crate::DictionaryProjectionError::InvalidDetailReference { .. },
            ) => "invalid_detail_reference",
            Self::Projection(crate::DictionaryProjectionError::MalformedSnapshot { .. }) => {
                "malformed_snapshot"
            }
            Self::Projection(crate::DictionaryProjectionError::StaleSnapshot { .. }) => {
                "stale_snapshot"
            }
            Self::InvalidSelection(_) => "unknown_selection",
            Self::DisabledSelection { .. } => "disabled_selection",
            Self::ConfigChanged => "config_changed",
            Self::StaleCursor => "stale_cursor",
            Self::MalformedCursor => "malformed_cursor",
            Self::NoEnabledPacks => "no_enabled_selection",
            Self::AmbiguousReference => "ambiguous_reference",
            Self::UnavailablePack { .. } => "unavailable_selection",
        };
        let config_path = match self {
            Self::Profile(
                CollectionProfileError::Io { path, .. }
                | CollectionProfileError::MissingFile { path, .. }
                | CollectionProfileError::CurrentDirectory { path, .. }
                | CollectionProfileError::Utf8 { path, .. }
                | CollectionProfileError::Invalid { path, .. }
                | CollectionProfileError::ConfigDecode { path, .. }
                | CollectionProfileError::TomlSerialize { path, .. }
                | CollectionProfileError::NoDefaultProfile { path }
                | CollectionProfileError::UnknownProfile { path, .. },
            ) => Some(vlajmaji_store::native_path_report(path)),
            Self::Profile(CollectionProfileError::Busy(path)) => {
                Some(vlajmaji_store::native_path_report(path))
            }
            Self::Profile(CollectionProfileError::Indeterminate { path, .. }) => {
                Some(vlajmaji_store::native_path_report(path))
            }
            _ => None,
        };
        let root_path = match self {
            Self::Catalogue(error) => error.report(operation).path,
            _ => None,
        };
        CollectionQueryErrorReport {
            version: 1,
            ok: false,
            category: category.into(),
            detail: self.to_string(),
            operation: operation.into(),
            config_path,
            root_path,
            profile: None,
            requested_profile: match self {
                Self::Profile(CollectionProfileError::UnknownProfile { name, .. }) => {
                    Some(name.clone())
                }
                _ => None,
            },
            snapshot: None,
            known_pack_ids: match self {
                Self::DisabledSelection { pack_id } => vec![pack_id.clone()],
                _ => Vec::new(),
            },
            outcome: if matches!(
                self,
                Self::Profile(CollectionProfileError::Indeterminate { .. })
            ) {
                "indeterminate"
            } else if operation == "configure" {
                "unchanged"
            } else {
                "not_applicable"
            }
            .into(),
            writer: profile_writer_context(self),
        }
    }
}

fn profile_writer_context(
    error: &CollectionQueryError,
) -> Option<CollectionProfileWriteFailureContext> {
    let CollectionQueryError::Profile(CollectionProfileError::Indeterminate {
        prior_revision,
        proposed_revision,
        prior_sha256,
        proposed_sha256,
        observed_sha256,
        observed_revision,
        ..
    }) = error
    else {
        return None;
    };
    Some(CollectionProfileWriteFailureContext {
        prior_revision: *prior_revision,
        proposed_revision: *proposed_revision,
        prior_sha256: prior_sha256.as_deref().cloned(),
        proposed_sha256: proposed_sha256.as_str().to_owned(),
        observed_sha256: observed_sha256.as_deref().cloned(),
        observed_revision: *observed_revision,
    })
}

impl fmt::Display for CollectionQueryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Profile(error) => write!(f, "collection profile: {error}"),
            Self::Catalogue(error) => write!(f, "collection catalogue: {error}"),
            Self::Query(error) => write!(f, "dictionary query: {error}"),
            Self::Projection(error) => write!(f, "dictionary projection: {error}"),
            Self::InvalidSelection(detail) => write!(f, "invalid collection selection: {detail}"),
            Self::DisabledSelection { pack_id } => {
                write!(f, "pack {pack_id} is disabled by the selected profile")
            }
            Self::ConfigChanged => {
                f.write_str("collection profile changed while the query snapshot was captured")
            }
            Self::StaleCursor => f.write_str("collection query cursor is stale or malformed"),
            Self::MalformedCursor => f.write_str("collection query cursor is malformed"),
            Self::NoEnabledPacks => f.write_str("no enabled collection packs are selected"),
            Self::AmbiguousReference => {
                f.write_str("local reference is ambiguous across retained registrations")
            }
            Self::UnavailablePack { pack_id, reason } => {
                write!(f, "selected pack {pack_id} is unavailable: {reason}")
            }
        }
    }
}
impl Error for CollectionQueryError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Profile(error) => Some(error),
            Self::Catalogue(error) => Some(error),
            Self::Query(error) => Some(error),
            Self::Projection(error) => Some(error),
            _ => None,
        }
    }
}
impl From<CollectionProfileError> for CollectionQueryError {
    fn from(error: CollectionProfileError) -> Self {
        Self::Profile(error)
    }
}
impl From<CollectionError> for CollectionQueryError {
    fn from(error: CollectionError) -> Self {
        Self::Catalogue(error)
    }
}
impl From<DictionaryQueryError> for CollectionQueryError {
    fn from(error: DictionaryQueryError) -> Self {
        Self::Query(error)
    }
}
impl From<crate::DictionaryProjectionError> for CollectionQueryError {
    fn from(error: crate::DictionaryProjectionError) -> Self {
        Self::Projection(error)
    }
}

/// Captured collection selection with payload setup deferred until needed.
pub struct CollectionSearchEngine {
    root: PathBuf,
    selection: CollectionQuerySelection,
    snapshot: CollectionSelectionSnapshot,
    generation: Option<CollectionGeneration>,
    manifest: Option<vlajmaji_store::DictionarySnapshotManifest>,
    payload: Option<DictionarySearchEngine>,
}

impl CollectionSearchEngine {
    /// Capture one explicit catalogue and policy snapshot without requiring a generation.
    ///
    /// # Errors
    /// Returns typed selection or catalogue errors; unavailable payload state is reported by the snapshot.
    pub fn open(
        root: impl AsRef<Path>,
        selection: CollectionQuerySelection,
    ) -> Result<Self, CollectionQueryError> {
        let default_packs = selection.packs.clone();
        Self::open_with_defaults(root, selection, default_packs)
    }

    fn open_with_defaults(
        root: impl AsRef<Path>,
        selection: CollectionQuerySelection,
        default_packs: Vec<PackId>,
    ) -> Result<Self, CollectionQueryError> {
        validate_selection(&selection)?;
        let root = absolute(root.as_ref())?;
        let catalogue = CollectionCatalogue::open(&root)?;
        let local_revision = catalogue.local_revision()?;
        let list = catalogue.list()?;
        let known: BTreeMap<_, _> = list
            .packs
            .iter()
            .map(|row| (row.pack_id.as_str(), row))
            .collect();
        for pack in default_packs
            .iter()
            .chain(&selection.packs)
            .chain(&selection.disabled)
        {
            if !known.contains_key(pack.as_str()) {
                return Err(CollectionQueryError::InvalidSelection(format!(
                    "pack {pack} is not registered"
                )));
            }
        }
        let generation = list.generation.clone();
        let minimum_store_state = generation
            .as_ref()
            .map(|generation| minimum_store_state(&generation.store_path));
        let selected = selection
            .packs
            .iter()
            .filter(|pack| !selection.disabled.contains(pack));
        let selected_ids: BTreeSet<_> = selected.map(|pack| pack.as_str()).collect();
        let mut collections = Vec::new();
        for row in &list.packs {
            let is_member = default_packs
                .iter()
                .any(|pack| pack.as_str() == row.pack_id);
            let is_selected = selected_ids.contains(row.pack_id.as_str());
            let current = row.installed_release_sha256.clone();
            let registration = current.as_ref().and_then(|digest| {
                row.registrations
                    .iter()
                    .find(|registration| &registration.release_sha256 == digest)
            });
            let producer_selectable = registration.is_some_and(|registration| {
                registration.releases.iter().any(|release| {
                    release.selected && release.availability == Availability::Available
                })
            });
            collections.push(CollectionPackAvailability {
                pack_id: row.pack_id.clone(),
                default_member: is_member,
                request_selected: is_selected,
                disabled: selection
                    .disabled
                    .iter()
                    .any(|pack| pack.as_str() == row.pack_id),
                installed_release_sha256: current,
                producer_selectable,
                derived_state: match row.installation_state {
                    vlajmaji_store::CollectionInstallationState::Installed => {
                        minimum_store_state.unwrap_or("minimum_unavailable").into()
                    }
                    vlajmaji_store::CollectionInstallationState::RegisteredOnly => {
                        "registered_only".into()
                    }
                    vlajmaji_store::CollectionInstallationState::Removed => "removed".into(),
                },
                original_inputs_present: registration
                    .map(|registration| registration.inputs_present),
                owned_files_present: row.owned_files_present,
                owned_files_warning: (row.owned_files_present == Some(false))
                    .then(|| "installed_copy_missing".into()),
                installed_copy_ownership: registration
                    .map(|registration| registration.installed_pack_copy_ownership.clone())
                    .unwrap_or_else(|| "not_installed".into()),
            });
        }
        let mut selected_releases = BTreeMap::<String, Vec<String>>::new();
        let mut usable_releases = BTreeSet::new();
        if let Some(generation) = &generation {
            for member in &generation.members {
                if !selected_ids.contains(member.pack_id.as_str()) {
                    continue;
                }
                let Some(row) = known.get(member.pack_id.as_str()) else {
                    continue;
                };
                let Some(registration) = row
                    .registrations
                    .iter()
                    .find(|registration| registration.release_sha256 == member.release_sha256)
                else {
                    continue;
                };
                let ids: Vec<String> = registration
                    .releases
                    .iter()
                    .filter(|release| {
                        release.selected && release.availability == Availability::Available
                    })
                    .map(|release| release.id.clone())
                    .collect();
                for id in &ids {
                    if let Ok(id) = EntityId::new(id.clone()) {
                        usable_releases.insert(id);
                    }
                }
                selected_releases.insert(member.pack_id.clone(), ids);
            }
        }
        let mut snapshot = CollectionSelectionSnapshot {
            version: 1,
            profile: selection.profile.clone(),
            local_revision,
            generation_id: generation.as_ref().map(|row| row.generation_id.clone()),
            content_id: generation.as_ref().map(|row| row.content_id.clone()),
            recorded_state_sha256: generation.as_ref().map(|row| row.canonical_sha256.clone()),
            packs: selection.packs.clone(),
            disabled: selection.disabled.clone(),
            selected_releases,
            collections: collections.clone(),
            selection_policy: "selected-available-owned-head-v2".into(),
            order_policy: "source-release-occurrence-v1".into(),
            projection_policy: "full-dictionary-exact-v1".into(),
            binding_sha256: String::new(),
        };
        snapshot.binding_sha256 = binding(&snapshot)?;
        let manifest = if let Some(generation) = &generation {
            let mut manifest = generation.snapshot_manifest();
            manifest.generation = snapshot.binding_sha256.clone();
            manifest.selected_release_ids = Some(usable_releases.iter().cloned().collect());
            Some(manifest)
        } else {
            None
        };
        let this = Self {
            root,
            selection,
            snapshot,
            generation,
            manifest,
            payload: None,
        };
        let revision_after = CollectionCatalogue::open(&this.root)?.local_revision()?;
        if revision_after != this.snapshot.local_revision {
            return Err(CollectionQueryError::ConfigChanged);
        }
        Ok(this)
    }

    /// Capture using one named or default profile in an already-read explicit config.
    ///
    /// # Errors
    /// Returns typed profile, catalogue, or selection errors.
    pub fn open_configured(
        profiles: &CollectionProfiles,
        name: Option<&str>,
        override_packs: Option<&[PackId]>,
    ) -> Result<Self, CollectionQueryError> {
        let profile = profiles.profile(name)?;
        let packs = override_packs.map_or_else(|| profile.packs.clone(), <[PackId]>::to_vec);
        if override_packs.is_some_and(|override_packs| {
            override_packs
                .iter()
                .any(|pack| profile.disabled.contains(pack))
        }) {
            let pack_id = override_packs
                .into_iter()
                .flatten()
                .find(|pack| profile.disabled.contains(pack))
                .expect("disabled pack was detected above");
            return Err(CollectionQueryError::DisabledSelection {
                pack_id: pack_id.to_string(),
            });
        }
        let identity = profiles.identity(&profile);
        let defaults = profile.packs.clone();
        let mut engine = Self::open_with_defaults(
            &profile.root,
            CollectionQuerySelection {
                packs,
                disabled: profile.disabled,
                profile: Some(identity),
            },
            defaults,
        )?;
        profiles.verify_unchanged().map_err(|error| match error {
            CollectionProfileError::Invalid { ref detail, .. }
                if detail == "configuration bytes changed since this snapshot" =>
            {
                CollectionQueryError::ConfigChanged
            }
            other => CollectionQueryError::Profile(other),
        })?;
        engine.selection.profile = engine.snapshot.profile.clone();
        Ok(engine)
    }

    /// Capture a named/default profile and report configuration/policy setup time.
    pub fn open_configured_profiled(
        profiles: &CollectionProfiles,
        name: Option<&str>,
        override_packs: Option<&[PackId]>,
    ) -> Result<(Self, Duration), CollectionQueryError> {
        let started = Instant::now();
        let engine = Self::open_configured(profiles, name, override_packs)?;
        Ok((engine, started.elapsed()))
    }

    /// Return the captured metadata-only operational snapshot.
    #[must_use]
    pub fn selection(&self) -> &CollectionSelectionSnapshot {
        &self.snapshot
    }

    /// Execute an exact lexical query or a typed reference target.
    ///
    /// # Errors
    /// Returns typed cursor, selection, payload, profile, catalogue, or query failures.
    pub fn execute(
        &mut self,
        request: DictionaryRequest,
    ) -> Result<CollectionQueryResponse, CollectionQueryError> {
        self.execute_inner(request, None)
    }

    /// Execute a managed request and capture same-call payload and query timing.
    pub fn execute_profiled(
        &mut self,
        request: DictionaryRequest,
    ) -> Result<(CollectionQueryResponse, CollectionProjectionProfile), CollectionQueryError> {
        let started = Instant::now();
        let mut profile = CollectionProjectionProfile::default();
        let response = self.execute_inner(request, Some(&mut profile))?;
        profile.total = started.elapsed();
        Ok((response, profile))
    }

    fn execute_inner(
        &mut self,
        request: DictionaryRequest,
        mut profile: Option<&mut CollectionProjectionProfile>,
    ) -> Result<CollectionQueryResponse, CollectionQueryError> {
        match request {
            DictionaryRequest::Exact {
                spelling,
                mut options,
            } => {
                let inner_cursor = match options.cursor.as_deref() {
                    Some(cursor) => Some(unwrap_cursor(cursor, &self.snapshot.binding_sha256)?),
                    None => None,
                };
                if let Some(cursor) = inner_cursor {
                    options.cursor = Some(cursor);
                }
                self.admit_exact()?;
                self.ensure_payload(profile.as_deref_mut())?;
                let engine = self.payload.as_ref().ok_or_else(|| {
                    CollectionQueryError::Query(DictionaryQueryError::Integrity {
                        detail: "payload engine is unavailable after setup".into(),
                    })
                })?;
                let mut response = if let Some(profile) = profile.as_deref_mut() {
                    let (response, exact_profile) = engine.exact_profiled(&spelling, options)?;
                    profile.query.total = exact_profile.total;
                    profile.query.candidate_sql = exact_profile.candidate_sql;
                    profile.query.lookup = exact_profile
                        .lookup_order_paging
                        .saturating_add(exact_profile.candidate_sql);
                    profile.query.projection = exact_profile.hydration;
                    response
                } else {
                    engine.exact(&spelling, options)?
                };
                if let Some(cursor) = response.next_cursor.take() {
                    response.next_cursor =
                        Some(wrap_cursor(&cursor, &self.snapshot.binding_sha256)?);
                }
                Ok(self.response(CollectionQueryResult::Exact(Box::new(response))))
            }
            DictionaryRequest::Resolve { target } => {
                let query_started = Instant::now();
                let response = self.resolve_target(target, profile.as_deref_mut())?;
                if let Some(profile) = profile {
                    profile.query.total = query_started.elapsed().saturating_sub(
                        profile
                            .payload_setup
                            .map_or(Duration::ZERO, |setup| setup.total),
                    );
                }
                Ok(self.response(CollectionQueryResult::Resolve(Box::new(response))))
            }
        }
    }

    fn admit_exact(&self) -> Result<(), CollectionQueryError> {
        let enabled: Vec<_> = self
            .selection
            .packs
            .iter()
            .filter(|pack| !self.selection.disabled.contains(pack))
            .collect();
        if enabled.is_empty() {
            return Err(CollectionQueryError::NoEnabledPacks);
        }
        for pack in enabled {
            let Some(state) = self
                .snapshot
                .collections
                .iter()
                .find(|row| row.pack_id == pack.as_str())
            else {
                return Err(CollectionQueryError::InvalidSelection(format!(
                    "pack {pack} is not registered"
                )));
            };
            if state.installed_release_sha256.is_none() {
                return Err(CollectionQueryError::UnavailablePack {
                    pack_id: pack.to_string(),
                    reason: if state.derived_state == "removed" {
                        "removed"
                    } else {
                        "registered_only"
                    }
                    .into(),
                });
            }
            if !state.producer_selectable {
                return Err(CollectionQueryError::UnavailablePack {
                    pack_id: pack.to_string(),
                    reason: "no_selected_available_release".into(),
                });
            }
        }
        if self.generation.is_none() {
            return Err(CollectionQueryError::UnavailablePack {
                pack_id: "unified".into(),
                reason: "no_generation".into(),
            });
        }
        Ok(())
    }

    fn ensure_payload(
        &mut self,
        profile: Option<&mut CollectionProjectionProfile>,
    ) -> Result<(), CollectionQueryError> {
        if self.payload.is_some() {
            return Ok(());
        }
        let manifest =
            self.manifest
                .clone()
                .ok_or_else(|| CollectionQueryError::UnavailablePack {
                    pack_id: "unified".into(),
                    reason: "no_generation".into(),
                })?;
        match DictionarySearchEngine::open_snapshot_profiled(
            DictionaryReadTopology::Unified,
            manifest,
        ) {
            Ok((engine, setup)) => {
                self.payload = Some(engine);
                if let Some(profile) = profile {
                    profile.payload_setup = Some(setup);
                }
                Ok(())
            }
            Err(error) => Err(CollectionQueryError::Query(error)),
        }
    }

    fn resolve_target(
        &mut self,
        target: RelationTarget,
        profile: Option<&mut CollectionProjectionProfile>,
    ) -> Result<CollectionReferenceResponse, CollectionQueryError> {
        match &target {
            RelationTarget::Unknown => {
                return Ok(CollectionReferenceResponse {
                    target,
                    status: ReferenceStatus::Unknown,
                    operational_reason: None,
                    payload: None,
                    catalogue_matches: Vec::new(),
                    requested_release: None,
                });
            }
            RelationTarget::Unresolved(text) if text.is_empty() => {
                return Err(CollectionQueryError::Query(
                    DictionaryQueryError::InvalidReference {
                        detail: "Unresolved target must be nonempty".into(),
                    },
                ));
            }
            RelationTarget::Unresolved(_) => {
                return Ok(CollectionReferenceResponse {
                    target,
                    status: ReferenceStatus::Unresolved,
                    operational_reason: None,
                    payload: None,
                    catalogue_matches: Vec::new(),
                    requested_release: None,
                });
            }
            _ => {}
        }
        let matches = self.metadata_matches(&target)?;
        if let Some((pack, release)) = self.selected_payload_for_target(&target, &matches.admission)
        {
            let _ = (pack, release);
            self.ensure_payload(profile)?;
            let engine = self.payload.as_ref().ok_or_else(|| {
                CollectionQueryError::Query(DictionaryQueryError::Integrity {
                    detail: "payload engine is unavailable after setup".into(),
                })
            })?;
            let mut payload = engine.resolve(&target)?;
            if matches!(target, RelationTarget::Local(_)) {
                let admitted: BTreeSet<String> = matches
                    .admission
                    .iter()
                    .filter(|matched| {
                        self.selection
                            .packs
                            .iter()
                            .any(|pack| pack.as_str() == matched.pack_id)
                            && !self
                                .selection
                                .disabled
                                .iter()
                                .any(|pack| pack.as_str() == matched.pack_id)
                            && self.generation.as_ref().is_some_and(|generation| {
                                generation.members.iter().any(|member| {
                                    member.pack_id == matched.pack_id
                                        && member.release_sha256 == matched.release_sha256
                                })
                            })
                    })
                    .flat_map(|matched| {
                        self.snapshot
                            .selected_releases
                            .get(&matched.pack_id)
                            .into_iter()
                            .flatten()
                            .cloned()
                    })
                    .collect();
                payload.confirmed.retain(|assertion| {
                    assertion
                        .release_ids
                        .iter()
                        .any(|release| admitted.contains(release.as_str()))
                });
                payload.unverified.retain(|assertion| {
                    assertion
                        .release_ids
                        .iter()
                        .any(|release| admitted.contains(release.as_str()))
                });
            }
            let requested_release = match &target {
                RelationTarget::Catalog(catalog) => match matches
                    .public
                    .iter()
                    .find(|matched| matched.model_release_id == catalog.release.as_str())
                {
                    Some(matched) => self.release_summary(
                        &matched.pack_id,
                        &matched.release_sha256,
                        &catalog.release,
                    )?,
                    None => None,
                },
                _ => None,
            };
            return Ok(CollectionReferenceResponse {
                target,
                status: payload.status,
                operational_reason: None,
                payload: Some(payload),
                catalogue_matches: matches.public,
                requested_release,
            });
        }
        let requested_release = if let RelationTarget::Catalog(catalog) = &target {
            match matches
                .public
                .iter()
                .find(|matched| matched.model_release_id == catalog.release.as_str())
            {
                Some(matched) => self.release_summary(
                    &matched.pack_id,
                    &matched.release_sha256,
                    &catalog.release,
                )?,
                None => self.release_summary_for_catalog(catalog)?,
            }
        } else {
            None
        };
        let reason = matches
            .public
            .iter()
            .find_map(|matched| self.operational_reason(&matched.pack_id, &matched.release_sha256))
            .unwrap_or_else(|| "registered_only".into());
        let status = if matches.public.is_empty() {
            ReferenceStatus::NoMatch
        } else if let RelationTarget::Catalog(catalog) = &target {
            let unknown = matches.public.iter().any(|matched| {
                catalog
                    .native_entry_id
                    .as_ref()
                    .is_some_and(|expected| matched.native_entry_id.as_ref() != Some(expected))
                    || catalog
                        .native_assertion_id
                        .as_ref()
                        .is_some_and(|expected| {
                            matched.native_assertion_id.as_ref() != Some(expected)
                        })
            });
            if unknown {
                ReferenceStatus::Unverified
            } else {
                ReferenceStatus::CatalogueOnly
            }
        } else {
            ReferenceStatus::CatalogueOnly
        };
        Ok(CollectionReferenceResponse {
            target,
            status,
            operational_reason: (!matches.public.is_empty()).then_some(reason),
            payload: None,
            catalogue_matches: matches.public,
            requested_release,
        })
    }

    fn metadata_matches(
        &self,
        target: &RelationTarget,
    ) -> Result<MetadataMatches, CollectionQueryError> {
        let catalogue = CollectionCatalogue::open(&self.root)?;
        if catalogue.local_revision()? != self.snapshot.local_revision {
            return Err(CollectionQueryError::Catalogue(
                CollectionError::InvalidRequest(
                    "stale_snapshot: catalogue revision changed".into(),
                ),
            ));
        }
        let list = catalogue.list()?;
        let mut results = Vec::new();
        let mut requested_release = None;
        match target {
            RelationTarget::Catalog(requested) => {
                let mut release_owner_found = false;
                for pack in &list.packs {
                    let pack_id = parse_pack_id(&pack.pack_id).ok_or_else(|| {
                        CollectionQueryError::InvalidSelection(format!(
                            "unsupported registered pack ID {}",
                            pack.pack_id
                        ))
                    })?;
                    for registration in &pack.registrations {
                        let descriptor =
                            catalogue.reference_metadata(pack_id, &registration.release_sha256)?;
                        let release = descriptor
                            .source_metadata
                            .releases
                            .iter()
                            .find(|release| release.id == requested.release);
                        let Some(release) = release else {
                            continue;
                        };
                        if requested_release.is_none() {
                            requested_release = Some(CollectionReleaseSummary::from(release));
                        }
                        let source = descriptor
                            .source_metadata
                            .collections
                            .iter()
                            .find(|collection| {
                                descriptor
                                    .source_metadata
                                    .components
                                    .iter()
                                    .any(|component| {
                                        component.id == release.component
                                            && component.collection == collection.id
                                    })
                            })
                            .map(|collection| &collection.source);
                        if source != Some(&requested.source) {
                            continue;
                        }
                        release_owner_found = true;
                        if requested.occurrence.is_none()
                            && requested.native_entry_id.is_none()
                            && requested.native_assertion_id.is_none()
                        {
                            return Err(CollectionQueryError::Query(
                                DictionaryQueryError::InvalidReference {
                                    detail:
                                        "catalogue target needs a native identity or occurrence"
                                            .into(),
                                },
                            ));
                        }
                        if requested.occurrence.as_ref().is_some_and(|occurrence| {
                            occurrence.locator.is_empty()
                                || !release.captures.contains(&occurrence.capture)
                                || !descriptor.captures.iter().any(|capture| {
                                    capture.capture == occurrence.capture
                                        && capture
                                            .occurrences
                                            .iter()
                                            .any(|known| known.locator == occurrence.locator)
                                })
                        }) {
                            return Err(CollectionQueryError::Query(
                                DictionaryQueryError::InvalidReference {
                                    detail: "catalogue occurrence is not in the requested release"
                                        .into(),
                                },
                            ));
                        }
                        for assertion_id in &release.assertions {
                            let entity = descriptor.entities.iter().find(|entity| entity.role == "assertion" && entity.id == assertion_id.as_str()).ok_or_else(|| CollectionQueryError::Query(DictionaryQueryError::Integrity { detail: "release assertion is absent from the retained descriptor".into() }))?;
                            if !entity
                                .source_ids
                                .iter()
                                .any(|source| source == requested.source.as_str())
                            {
                                continue;
                            }
                            let Some(occurrence) = entity.occurrence.as_ref() else {
                                continue;
                            };
                            if requested
                                .occurrence
                                .as_ref()
                                .is_some_and(|expected| expected != occurrence)
                            {
                                continue;
                            }
                            let native_entry = find_native(&descriptor, occurrence, "entry");
                            let native_assertion =
                                find_native(&descriptor, occurrence, "assertion");
                            if requested.native_entry_id.as_ref().is_some_and(|expected| {
                                native_entry
                                    .as_ref()
                                    .is_some_and(|actual| actual != expected)
                            }) {
                                continue;
                            }
                            if requested
                                .native_assertion_id
                                .as_ref()
                                .is_some_and(|expected| {
                                    native_assertion
                                        .as_ref()
                                        .is_some_and(|actual| actual != expected)
                                })
                            {
                                continue;
                            }
                            results.push(CollectionReferenceMatch {
                                pack_id: pack.pack_id.clone(),
                                release_sha256: registration.release_sha256.clone(),
                                source_id: requested.source.to_string(),
                                model_release_id: release.id.to_string(),
                                role: Some("assertion".into()),
                                entity_id: Some(assertion_id.to_string()),
                                occurrence: Some(occurrence.locator.clone()),
                                capture: Some(occurrence.capture.clone()),
                                native_entry_id: native_entry,
                                native_assertion_id: native_assertion,
                                contributing_pack_ids: vec![pack.pack_id.clone()],
                            });
                        }
                    }
                }
                if !release_owner_found {
                    return Err(CollectionQueryError::Query(DictionaryQueryError::InvalidReference {
                        detail: "catalogue release is unknown or does not belong to the requested source".into(),
                    }));
                }
            }
            RelationTarget::Local(local) => {
                let (role, id) = local_role_id(local);
                for pack in &list.packs {
                    let pack_id = parse_pack_id(&pack.pack_id).ok_or_else(|| {
                        CollectionQueryError::InvalidSelection(format!(
                            "unsupported registered pack ID {}",
                            pack.pack_id
                        ))
                    })?;
                    for registration in &pack.registrations {
                        let descriptor =
                            catalogue.reference_metadata(pack_id, &registration.release_sha256)?;
                        for entity in descriptor
                            .entities
                            .iter()
                            .filter(|entity| entity.role == role && entity.id == id)
                        {
                            results.push(CollectionReferenceMatch {
                                pack_id: pack.pack_id.clone(),
                                release_sha256: registration.release_sha256.clone(),
                                source_id: entity.source_ids.first().cloned().unwrap_or_default(),
                                model_release_id: String::new(),
                                role: Some(role.into()),
                                entity_id: Some(id.into()),
                                occurrence: entity
                                    .occurrence
                                    .as_ref()
                                    .map(|occurrence| occurrence.locator.clone()),
                                capture: entity
                                    .occurrence
                                    .as_ref()
                                    .map(|occurrence| occurrence.capture.clone()),
                                native_entry_id: None,
                                native_assertion_id: None,
                                contributing_pack_ids: vec![pack.pack_id.clone()],
                            });
                        }
                    }
                }
                if results.is_empty() {
                    return Err(CollectionQueryError::Query(
                        DictionaryQueryError::InvalidReference {
                            detail: format!(
                                "local {role} identity {id} is not present in retained registrations"
                            ),
                        },
                    ));
                }
                if let Some(first) = results.first() {
                    let first_pack = parse_pack_id(&first.pack_id).ok_or_else(|| {
                        CollectionQueryError::InvalidSelection(
                            "unsupported registered pack ID".into(),
                        )
                    })?;
                    let first_descriptor =
                        catalogue.reference_metadata(first_pack, &first.release_sha256)?;
                    let first_entity = first_descriptor
                        .entities
                        .iter()
                        .find(|entity| entity.role == role && entity.id == id)
                        .ok_or_else(|| {
                            CollectionQueryError::Query(DictionaryQueryError::Integrity {
                                detail: "local entity disappeared from its retained descriptor"
                                    .into(),
                            })
                        })?;
                    let first_body = first_descriptor.body_sha256.get(&format!("{role}:{id}"));
                    for row in results.iter().skip(1) {
                        let pack = parse_pack_id(&row.pack_id).ok_or_else(|| {
                            CollectionQueryError::InvalidSelection(
                                "unsupported registered pack ID".into(),
                            )
                        })?;
                        let descriptor = catalogue.reference_metadata(pack, &row.release_sha256)?;
                        let entity = descriptor
                            .entities
                            .iter()
                            .find(|entity| entity.role == role && entity.id == id)
                            .ok_or_else(|| {
                                CollectionQueryError::Query(DictionaryQueryError::Integrity {
                                    detail: "local entity disappeared from its retained descriptor"
                                        .into(),
                                })
                            })?;
                        if entity != first_entity
                            || descriptor.body_sha256.get(&format!("{role}:{id}")) != first_body
                        {
                            return Err(CollectionQueryError::AmbiguousReference);
                        }
                    }
                }
            }
            _ => {}
        }
        let admission = results.clone();
        let public = if matches!(target, RelationTarget::Local(_)) {
            let mut deduplicated: Vec<CollectionReferenceMatch> = Vec::new();
            for row in &results {
                let same_public_identity = |existing: &CollectionReferenceMatch| {
                    existing.source_id == row.source_id
                        && existing.role == row.role
                        && existing.entity_id == row.entity_id
                        && existing.occurrence == row.occurrence
                        && existing.capture == row.capture
                        && existing.native_entry_id == row.native_entry_id
                        && existing.native_assertion_id == row.native_assertion_id
                };
                if let Some(existing) = deduplicated
                    .iter_mut()
                    .find(|row| same_public_identity(row))
                {
                    if !existing.contributing_pack_ids.contains(&row.pack_id) {
                        existing.contributing_pack_ids.push(row.pack_id.clone());
                        existing.contributing_pack_ids.sort();
                    }
                    let row_is_current = self.generation.as_ref().is_some_and(|generation| {
                        generation.members.iter().any(|member| {
                            member.pack_id == row.pack_id
                                && member.release_sha256 == row.release_sha256
                        })
                    });
                    if row_is_current {
                        existing.pack_id = row.pack_id.clone();
                        existing.release_sha256 = row.release_sha256.clone();
                    }
                } else {
                    deduplicated.push(row.clone());
                }
            }
            deduplicated
        } else {
            results
        };
        Ok(MetadataMatches {
            public,
            admission,
            requested_release,
        })
    }

    fn selected_payload_for_target(
        &self,
        target: &RelationTarget,
        matches: &[CollectionReferenceMatch],
    ) -> Option<(String, String)> {
        matches.iter().find_map(|matched| {
            let pack_selected = self
                .selection
                .packs
                .iter()
                .any(|pack| pack.as_str() == matched.pack_id)
                && !self
                    .selection
                    .disabled
                    .iter()
                    .any(|pack| pack.as_str() == matched.pack_id);
            let member = self.generation.as_ref()?.members.iter().any(|member| {
                member.pack_id == matched.pack_id && member.release_sha256 == matched.release_sha256
            });
            if !pack_selected || !member {
                return None;
            }
            match target {
                RelationTarget::Catalog(_) => {
                    Some((matched.pack_id.clone(), matched.release_sha256.clone()))
                }
                RelationTarget::Local(local) => {
                    let _ = local;
                    Some((matched.pack_id.clone(), matched.release_sha256.clone()))
                }
                _ => None,
            }
        })
    }

    fn operational_reason(&self, pack: &str, release: &str) -> Option<String> {
        if self.selection.disabled.iter().any(|id| id.as_str() == pack) {
            return Some("disabled".into());
        }
        if !self.selection.packs.iter().any(|id| id.as_str() == pack) {
            return Some("not_selected".into());
        }
        let availability = self
            .snapshot
            .collections
            .iter()
            .find(|row| row.pack_id == pack)?;
        if availability.derived_state == "removed" {
            return Some("removed".into());
        }
        if availability.installed_release_sha256.as_deref() != Some(release) {
            return Some("registered_only".into());
        }
        if availability.derived_state != "minimum_ok" {
            return Some(availability.derived_state.clone());
        }
        if !availability.producer_selectable {
            return Some("no_selected_available_release".into());
        }
        None
    }

    fn release_summary(
        &self,
        pack: &str,
        digest: &str,
        release_id: &EntityId,
    ) -> Result<Option<CollectionReleaseSummary>, CollectionQueryError> {
        let catalogue = CollectionCatalogue::open(&self.root)?;
        let pack = parse_pack_id(pack).ok_or_else(|| {
            CollectionQueryError::InvalidSelection(format!("unsupported registered pack ID {pack}"))
        })?;
        let row = catalogue
            .list()?
            .packs
            .into_iter()
            .find(|row| row.pack_id == pack.as_str());
        Ok(row
            .and_then(|row| {
                row.registrations
                    .into_iter()
                    .find(|registration| registration.release_sha256 == digest)
            })
            .and_then(|registration| {
                registration
                    .releases
                    .into_iter()
                    .find(|release| release.id == release_id.as_str())
            }))
    }

    fn release_summary_for_catalog(
        &self,
        target: &vlajmaji_lexicon::CatalogTarget,
    ) -> Result<Option<CollectionReleaseSummary>, CollectionQueryError> {
        let catalogue = CollectionCatalogue::open(&self.root)?;
        if catalogue.local_revision()? != self.snapshot.local_revision {
            return Err(CollectionQueryError::Catalogue(
                CollectionError::InvalidRequest(
                    "stale_snapshot: catalogue revision changed".into(),
                ),
            ));
        }
        let list = catalogue.list()?;
        for row in list.packs {
            let pack = parse_pack_id(&row.pack_id).ok_or_else(|| {
                CollectionQueryError::InvalidSelection(format!(
                    "unsupported registered pack ID {}",
                    row.pack_id
                ))
            })?;
            for registration in row.registrations {
                let descriptor =
                    catalogue.reference_metadata(pack, &registration.release_sha256)?;
                let Some(release) = descriptor
                    .source_metadata
                    .releases
                    .iter()
                    .find(|release| release.id == target.release)
                else {
                    continue;
                };
                let Some(component) = descriptor
                    .source_metadata
                    .components
                    .iter()
                    .find(|component| component.id == release.component)
                else {
                    continue;
                };
                let Some(collection) = descriptor
                    .source_metadata
                    .collections
                    .iter()
                    .find(|collection| collection.id == component.collection)
                else {
                    continue;
                };
                if collection.source == target.source {
                    return self.release_summary(
                        pack.as_str(),
                        &registration.release_sha256,
                        &target.release,
                    );
                }
            }
        }
        Ok(None)
    }

    fn response(&self, result: CollectionQueryResult) -> CollectionQueryResponse {
        CollectionQueryResponse {
            version: 1,
            snapshot: self.snapshot.clone(),
            collections: self.snapshot.collections.clone(),
            result,
        }
    }
}

fn validate_selection(selection: &CollectionQuerySelection) -> Result<(), CollectionQueryError> {
    let mut seen = BTreeSet::new();
    for pack in &selection.packs {
        if !seen.insert(*pack) {
            return Err(CollectionQueryError::InvalidSelection(format!(
                "duplicate selected pack {pack}"
            )));
        }
    }
    seen.clear();
    for pack in &selection.disabled {
        if !seen.insert(*pack) {
            return Err(CollectionQueryError::InvalidSelection(format!(
                "duplicate disabled pack {pack}"
            )));
        }
    }
    Ok(())
}

fn parse_pack_id(value: &str) -> Option<PackId> {
    match value {
        "lensisku-jbotci" => Some(PackId::LensiskuJbotci),
        "llg" => Some(PackId::Llg),
        "la-lojban" => Some(PackId::LaLojban),
        "sutysisku" => Some(PackId::Sutysisku),
        _ => None,
    }
}

fn local_role_id(local: &LocalRef) -> (&'static str, &str) {
    match local {
        LocalRef::Entry(id) => ("entry", id.as_str()),
        LocalRef::Assertion(id) => ("assertion", id.as_str()),
        LocalRef::Form(id) => ("form", id.as_str()),
        LocalRef::EditorialSense(id) => ("editorial_sense", id.as_str()),
    }
}

fn find_native(
    descriptor: &vlajmaji_store::CollectionReferenceMetadata,
    occurrence: &vlajmaji_lexicon::OccurrenceRef,
    role: &str,
) -> Option<String> {
    let attribution = descriptor.registry.occurrences.iter().find(|attribution| {
        attribution.occurrence.snapshot().namespace() == occurrence.capture.namespace
            && attribution.occurrence.snapshot().sha256() == occurrence.capture.sha256
            && attribution.occurrence.locator() == occurrence.locator
    })?;
    match role {
        "entry" => attribution.native_entry_id.clone(),
        "assertion" => attribution.native_assertion_id.clone(),
        _ => None,
    }
}

fn minimum_store_state(path: &Path) -> &'static str {
    match DictionaryStore::open(path) {
        Ok(store) => {
            drop(store);
            "minimum_ok"
        }
        Err(DictionaryStoreError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => {
            "minimum_missing"
        }
        Err(DictionaryStoreError::InvalidStore {
            reason: DictionaryStoreInvalidity::MissingDatabase,
            ..
        }) => "minimum_missing",
        Err(DictionaryStoreError::InvalidStore {
            reason: DictionaryStoreInvalidity::SchemaVersion,
            ..
        }) => "minimum_unsupported_version",
        Err(_) => "minimum_invalid",
    }
}

fn absolute(path: &Path) -> Result<PathBuf, CollectionQueryError> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        std::env::current_dir()
            .map(|cwd| cwd.join(path))
            .map_err(|error| {
                CollectionQueryError::Catalogue(CollectionError::ArtifactIoSource {
                    path: Some(path.to_path_buf()),
                    source: Box::new(error),
                })
            })
    }
}

fn binding(snapshot: &CollectionSelectionSnapshot) -> Result<String, CollectionQueryError> {
    #[derive(Serialize)]
    struct BoundCollection<'a> {
        pack_id: &'a str,
        default_member: bool,
        request_selected: bool,
        disabled: bool,
        installed_release_sha256: &'a Option<String>,
        producer_selectable: bool,
        derived_state: &'a str,
    }
    #[derive(Serialize)]
    struct Binding<'a> {
        version: u32,
        profile: &'a Option<CollectionProfileIdentity>,
        local_revision: u64,
        generation_id: &'a Option<String>,
        content_id: &'a Option<String>,
        recorded_state_sha256: &'a Option<String>,
        packs: &'a [PackId],
        disabled: &'a [PackId],
        selected_releases: &'a BTreeMap<String, Vec<String>>,
        collections: Vec<BoundCollection<'a>>,
        selection_policy: &'a str,
        order_policy: &'a str,
        projection_policy: &'a str,
    }
    let collections = snapshot
        .collections
        .iter()
        .map(|row| BoundCollection {
            pack_id: &row.pack_id,
            default_member: row.default_member,
            request_selected: row.request_selected,
            disabled: row.disabled,
            installed_release_sha256: &row.installed_release_sha256,
            producer_selectable: row.producer_selectable,
            derived_state: &row.derived_state,
        })
        .collect();
    let bytes = serde_json::to_vec(&Binding {
        version: snapshot.version,
        profile: &snapshot.profile,
        local_revision: snapshot.local_revision,
        generation_id: &snapshot.generation_id,
        content_id: &snapshot.content_id,
        recorded_state_sha256: &snapshot.recorded_state_sha256,
        packs: &snapshot.packs,
        disabled: &snapshot.disabled,
        selected_releases: &snapshot.selected_releases,
        collections,
        selection_policy: &snapshot.selection_policy,
        order_policy: &snapshot.order_policy,
        projection_policy: &snapshot.projection_policy,
    })
    .map_err(|error| CollectionQueryError::InvalidSelection(error.to_string()))?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

fn wrap_cursor(inner: &str, binding: &str) -> Result<String, CollectionQueryError> {
    #[derive(Serialize)]
    struct Wire<'a> {
        version: u32,
        snapshot_binding: &'a str,
        inner: &'a str,
    }
    let bytes = serde_json::to_vec(&Wire {
        version: 1,
        snapshot_binding: binding,
        inner,
    })
    .map_err(|_| CollectionQueryError::StaleCursor)?;
    let digest = format!("{:x}", Sha256::digest(&bytes));
    let encoded = bytes
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    Ok(format!("collection-v1:{encoded}:{digest}"))
}

fn unwrap_cursor(cursor: &str, binding: &str) -> Result<String, CollectionQueryError> {
    let Some(encoded) = cursor.strip_prefix("collection-v1:") else {
        return Err(if projection::is_valid_compact_managed_cursor(cursor) {
            CollectionQueryError::StaleCursor
        } else {
            CollectionQueryError::MalformedCursor
        });
    };
    let Some((hex, digest)) = encoded.split_once(':') else {
        return Err(CollectionQueryError::MalformedCursor);
    };
    if hex.is_empty()
        || hex.len() % 2 != 0
        || !lower_hex(hex)
        || digest.len() != 64
        || !lower_hex(digest)
    {
        return Err(CollectionQueryError::MalformedCursor);
    }
    let raw = hex.as_bytes();
    let (pairs, remainder) = raw.as_chunks::<2>();
    if !remainder.is_empty() {
        return Err(CollectionQueryError::MalformedCursor);
    }
    let bytes: Vec<u8> = pairs
        .iter()
        .map(|pair| {
            let high = (pair[0] as char)
                .to_digit(16)
                .ok_or(CollectionQueryError::MalformedCursor)?;
            let low = (pair[1] as char)
                .to_digit(16)
                .ok_or(CollectionQueryError::MalformedCursor)?;
            Ok::<u8, CollectionQueryError>(((high << 4) | low) as u8)
        })
        .collect::<Result<_, _>>()?;
    if format!("{:x}", Sha256::digest(&bytes)) != digest {
        return Err(CollectionQueryError::MalformedCursor);
    }
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Wire {
        version: u32,
        snapshot_binding: String,
        inner: String,
    }
    let wire: Wire =
        serde_json::from_slice(&bytes).map_err(|_| CollectionQueryError::MalformedCursor)?;
    if wire.version != 1
        || wire.inner.is_empty()
        || wire.snapshot_binding.len() != 64
        || !lower_hex(&wire.snapshot_binding)
    {
        return Err(CollectionQueryError::MalformedCursor);
    }
    if wire.snapshot_binding != binding {
        return Err(CollectionQueryError::StaleCursor);
    }
    Ok(wire.inner)
}

fn lower_hex(value: &str) -> bool {
    value
        .bytes()
        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[cfg(test)]
mod cursor_tests {
    use std::path::PathBuf;

    use sha2::{Digest, Sha256};

    use super::{CollectionQueryError, unwrap_cursor, wrap_cursor};
    use vlajmaji_store::CollectionProfileError;

    fn encoded_wire(bytes: &[u8]) -> String {
        let hex = bytes
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        format!("collection-v1:{hex}:{:x}", Sha256::digest(bytes))
    }

    #[test]
    fn test_managed_cursor_roundtrip_and_snapshot_binding() {
        let cursor = wrap_cursor("opaque-inner", &"a".repeat(64)).unwrap();
        assert_eq!(
            unwrap_cursor(&cursor, &"a".repeat(64)).unwrap(),
            "opaque-inner"
        );
        assert!(matches!(
            unwrap_cursor(&cursor, &"b".repeat(64)),
            Err(CollectionQueryError::StaleCursor)
        ));
    }

    #[test]
    fn test_managed_cursor_rejects_malformed_unicode_hex_without_panicking() {
        let error = unwrap_cursor(
            &format!("collection-v1:aéx:{}", "0".repeat(64)),
            &"a".repeat(64),
        )
        .unwrap_err();
        assert!(matches!(error, CollectionQueryError::MalformedCursor));
    }

    #[test]
    fn test_managed_cursor_rejects_malformed_envelopes_and_wire_shapes() {
        let binding = "a".repeat(64);
        let mut uppercase_hex = encoded_wire(
            format!(r#"{{"version":1,"snapshot_binding":"{binding}","inner":"cursor"}}"#)
                .as_bytes(),
        );
        let colon = uppercase_hex.find(':').unwrap() + 1;
        uppercase_hex.replace_range(colon..colon + 2, "7B");

        let mut uppercase_digest = encoded_wire(
            format!(r#"{{"version":1,"snapshot_binding":"{binding}","inner":"cursor"}}"#)
                .as_bytes(),
        );
        let digest_at = uppercase_digest.rfind(':').unwrap() + 1;
        uppercase_digest.replace_range(digest_at..digest_at + 1, "A");

        let bad_wires = [
            "wrong-prefix:00:0000000000000000000000000000000000000000000000000000000000000000".into(),
            "collection-v1".into(),
            "collection-v1::0000000000000000000000000000000000000000000000000000000000000000".into(),
            "collection-v1:0:0000000000000000000000000000000000000000000000000000000000000000".into(),
            uppercase_hex,
            uppercase_digest,
            "collection-v1:00:0000000000000000000000000000000000000000000000000000000000000000".into(),
            encoded_wire(b"{\"version\":1,\"snapshot_binding\":\"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\",\"inner\":\"x\",\"extra\":true}"),
            encoded_wire(format!(r#"{{"version":2,"snapshot_binding":"{binding}","inner":"x"}}"#).as_bytes()),
            encoded_wire(format!(r#"{{"version":1,"snapshot_binding":"{binding}","inner":""}}"#).as_bytes()),
            encoded_wire(b"{\"version\":1,\"snapshot_binding\":\"short\",\"inner\":\"x\"}"),
            encoded_wire(format!(r#"{{"version":1,"snapshot_binding":"{}","inner":"x"}}"#, "g".repeat(64)).as_bytes()),
        ];
        for cursor in bad_wires {
            assert!(
                matches!(
                    unwrap_cursor(&cursor, &binding),
                    Err(CollectionQueryError::MalformedCursor)
                ),
                "expected malformed cursor rejection"
            );
        }
        assert_eq!(
            unwrap_cursor(&wrap_cursor("cursor", &binding).unwrap(), &binding).unwrap(),
            "cursor"
        );
    }

    #[test]
    fn test_indeterminate_config_report_retains_prior_proposed_and_observed_revisions() {
        let error = CollectionQueryError::Profile(CollectionProfileError::Indeterminate {
            path: PathBuf::from("/tmp/profiles.toml"),
            detail: "parent sync failed".into(),
            prior_revision: Some(4),
            proposed_revision: 5,
            prior_sha256: Some(Box::new("a".repeat(64))),
            proposed_sha256: Box::new("b".repeat(64)),
            observed_sha256: Some(Box::new("b".repeat(64))),
            observed_revision: Some(5),
            source: std::io::Error::other("sync failed"),
        });
        let report = error.report_for("configure");
        assert_eq!(report.category, "config_indeterminate");
        assert_eq!(report.operation, "configure");
        assert_eq!(report.outcome, "indeterminate");
        let writer = report.writer.unwrap();
        assert_eq!(writer.prior_revision, Some(4));
        assert_eq!(writer.proposed_revision, 5);
        assert_eq!(writer.observed_revision, Some(5));
        assert_eq!(writer.observed_sha256, Some("b".repeat(64)));
    }
}
