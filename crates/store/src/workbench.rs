//! Explicitly built, freshness-bound mixed upstream and owned dictionary views.

use std::{
    error::Error,
    fmt, fs, io,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use sha2::{Digest, Sha256};
use vlajmaji_lexicon::{
    Availability, Dictionary, EntityId, OwnedContextInput, OwnedCurationDocument,
    OwnedCurationReason, OwnedCurationResolution, OwnedCurationStatus, PackError, PackId,
    PackPayloadKind, compose_dictionaries, load_sealed_release,
};

use crate::{
    CollectionCatalogue, CollectionError, DictionarySnapshotManifest, DictionaryStore,
    DictionaryStoreError, NativePathReport, OwnedJournalError, OwnedJournalReader,
    OwnedRevisionIdentity, native_path_report,
};

const BINDING_KIND: &str = "vlajmaji-workbench";
const BINDING_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BuildGuardStage {
    BeforeReservation,
    BeforeBinding,
    AfterBinding,
}

/// Selects the owned revision used to build a workbench.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkbenchOwnedRevision {
    /// Capture the current owned head at build time.
    Head,
    /// Capture exactly one retained historical revision.
    Revision(EntityId),
}

/// The exact ordered pack set supplied for one immutable curation context.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkbenchContextBinding {
    /// Declared curation context identity.
    pub context: EntityId,
    /// Ordered selected packs used to satisfy this pin.
    pub packs: Vec<PackId>,
}

/// Strict version-one workbench selection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkbenchSelection {
    /// Selection wire version; only one is supported.
    pub version: u32,
    /// Requested catalogue packs in caller order.
    pub packs: Vec<PackId>,
    /// Requested packs deliberately excluded from this view.
    pub disabled: Vec<PackId>,
    /// Optional explicit owned source/revision.
    #[serde(deserialize_with = "deserialize_required_option")]
    pub owned: Option<WorkbenchOwnedRevision>,
    /// Bindings for every curation context, in declaration order.
    pub contexts: Vec<WorkbenchContextBinding>,
}

impl WorkbenchSelection {
    /// Decode and validate the strict version-one selection wire.
    ///
    /// # Errors
    ///
    /// Returns an invalid-selection or unsupported-version error for malformed,
    /// duplicate, unknown, or inconsistent request fields.
    pub fn from_json_bytes(bytes: &[u8]) -> Result<Self, WorkbenchError> {
        let selection: Self =
            serde_json::from_slice(bytes).map_err(|error| WorkbenchError::InvalidSelection {
                detail: error.to_string(),
            })?;
        selection.validate()?;
        Ok(selection)
    }

    /// Validate selection structure without reading catalogue or journal files.
    ///
    /// # Errors
    ///
    /// Returns an invalid-selection or unsupported-version error when a pack,
    /// disabled member, context, or owned selection is inconsistent.
    pub fn validate(&self) -> Result<(), WorkbenchError> {
        if self.version != 1 {
            return Err(WorkbenchError::UnsupportedVersion {
                kind: "workbench selection",
                found: self.version,
            });
        }
        let mut selected = std::collections::BTreeSet::new();
        if self.packs.iter().any(|pack| !selected.insert(*pack)) {
            return Err(invalid_selection("duplicate selected pack"));
        }
        let mut disabled = std::collections::BTreeSet::new();
        for pack in &self.disabled {
            if !disabled.insert(*pack) {
                return Err(invalid_selection("duplicate disabled pack"));
            }
            if !selected.contains(pack) {
                return Err(invalid_selection(
                    "disabled packs must be members of the selected pack list",
                ));
            }
        }
        if self.packs.iter().all(|pack| disabled.contains(pack)) && self.owned.is_none() {
            return Err(invalid_selection("selection has no effective input"));
        }
        let mut context_ids = std::collections::BTreeSet::new();
        for binding in &self.contexts {
            if !context_ids.insert(binding.context.clone()) {
                return Err(invalid_selection("duplicate context binding"));
            }
            if binding.packs.is_empty() {
                return Err(invalid_selection("context pack binding is empty"));
            }
            let mut bound = std::collections::BTreeSet::new();
            for pack in &binding.packs {
                if !bound.insert(*pack) {
                    return Err(invalid_selection("duplicate pack in context binding"));
                }
                if !selected.contains(pack) || disabled.contains(pack) {
                    return Err(invalid_selection(
                        "context packs must be selected and enabled",
                    ));
                }
            }
        }
        if self.owned.is_none() && !self.contexts.is_empty() {
            return Err(invalid_selection(
                "contexts require an explicit owned revision",
            ));
        }
        Ok(())
    }
}

/// Native input roots for an explicit build request.
#[derive(Debug, Clone)]
pub struct WorkbenchBuildRequest {
    /// Catalogue root, required exactly when packs are selected.
    pub catalogue_root: Option<PathBuf>,
    /// Owned-journal root, required exactly when an owned revision is selected.
    pub owned_root: Option<PathBuf>,
    /// Strict selected inputs and context bindings.
    pub selection: WorkbenchSelection,
}

/// Catalogue identity captured into the immutable binding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkbenchCatalogueBinding {
    /// Exact native catalogue root.
    #[serde(deserialize_with = "deserialize_native_path")]
    pub root: NativePathReport,
    /// Local catalogue revision.
    pub local_revision: u64,
    /// Current installed generation, if any.
    #[serde(deserialize_with = "deserialize_required_option")]
    pub generation_id: Option<String>,
    /// Current installed content identity, if any.
    #[serde(deserialize_with = "deserialize_required_option")]
    pub content_id: Option<String>,
    /// Recorded canonical digest for the current generation, if any.
    #[serde(deserialize_with = "deserialize_required_option")]
    pub recorded_state_sha256: Option<String>,
    /// Installed members in selected request order.
    pub members: Vec<WorkbenchCatalogueMember>,
}

/// One admitted installed release.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkbenchCatalogueMember {
    /// Stable selected pack identifier.
    pub pack_id: String,
    /// Digest of the exact sealed release bytes.
    pub release_sha256: String,
    /// Payload scope read from the strict seal.
    pub payload_kind: PackPayloadKind,
}

/// Selected owned revision and its compact checked identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkbenchOwnedBinding {
    /// Exact native owned-journal root.
    #[serde(deserialize_with = "deserialize_native_path")]
    pub root: NativePathReport,
    /// Head or explicit historical revision selection.
    pub selection: WorkbenchOwnedRevision,
    /// Compact checked persisted identity.
    pub identity: OwnedRevisionIdentity,
}

/// Retained declaration and optional body observation for one context pin.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkbenchContextObservation {
    /// Complete original pin declaration.
    pub pin: vlajmaji_lexicon::OwnedContextPin,
    /// Ordered packs supplied for this context, or `None` when deliberately omitted.
    #[serde(deserialize_with = "deserialize_required_option")]
    pub packs: Option<Vec<PackId>>,
    /// Observed canonical body digest, absent when the binding was omitted.
    #[serde(deserialize_with = "deserialize_required_option")]
    pub observed_model_sha256: Option<String>,
}

#[derive(Serialize)]
struct BindingPayload<'a> {
    version: u32,
    kind: &'a str,
    selection: &'a WorkbenchSelection,
    catalogue: &'a Option<WorkbenchCatalogueBinding>,
    owned: &'a Option<WorkbenchOwnedBinding>,
    contexts: &'a [WorkbenchContextObservation],
    resolutions: &'a [OwnedCurationResolution],
    selected_release_ids: &'a [EntityId],
    state_sha256: &'a str,
}

/// Immutable version-one persisted identity for one prepared workbench.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WorkbenchBinding {
    /// Binding wire version.
    version: u32,
    /// Stable family marker.
    kind: String,
    /// Complete declared request.
    selection: WorkbenchSelection,
    /// Captured catalogue identity, when selected.
    #[serde(deserialize_with = "deserialize_required_option")]
    catalogue: Option<WorkbenchCatalogueBinding>,
    /// Captured owned revision, when selected.
    #[serde(deserialize_with = "deserialize_required_option")]
    owned: Option<WorkbenchOwnedBinding>,
    /// Every declared pin and its supplied/omitted body observation.
    contexts: Vec<WorkbenchContextObservation>,
    /// Complete materialization outcomes in curation record order.
    resolutions: Vec<OwnedCurationResolution>,
    /// Selected release overlay supplied to the unified dictionary snapshot.
    selected_release_ids: Vec<EntityId>,
    /// Digest of the complete canonical composed dictionary state.
    state_sha256: String,
    /// Digest of the other nine fields in declared serialization order.
    binding_sha256: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WorkbenchResolutionWire {
    record: EntityId,
    status: OwnedCurationStatus,
    #[serde(deserialize_with = "deserialize_required_option")]
    reason: Option<OwnedCurationReason>,
    #[serde(deserialize_with = "deserialize_required_option")]
    member: Option<u64>,
    projected: bool,
}

impl From<WorkbenchResolutionWire> for OwnedCurationResolution {
    fn from(wire: WorkbenchResolutionWire) -> Self {
        Self {
            record: wire.record,
            status: wire.status,
            reason: wire.reason,
            member: wire.member,
            projected: wire.projected,
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WorkbenchBindingWire {
    version: u32,
    kind: String,
    selection: WorkbenchSelection,
    #[serde(deserialize_with = "deserialize_required_option")]
    catalogue: Option<WorkbenchCatalogueBinding>,
    #[serde(deserialize_with = "deserialize_required_option")]
    owned: Option<WorkbenchOwnedBinding>,
    contexts: Vec<WorkbenchContextObservation>,
    resolutions: Vec<WorkbenchResolutionWire>,
    selected_release_ids: Vec<EntityId>,
    state_sha256: String,
    binding_sha256: String,
}

impl<'de> Deserialize<'de> for WorkbenchBinding {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let wire = WorkbenchBindingWire::deserialize(deserializer)?;
        let binding = Self {
            version: wire.version,
            kind: wire.kind,
            selection: wire.selection,
            catalogue: wire.catalogue,
            owned: wire.owned,
            contexts: wire.contexts,
            resolutions: wire.resolutions.into_iter().map(Into::into).collect(),
            selected_release_ids: wire.selected_release_ids,
            state_sha256: wire.state_sha256,
            binding_sha256: wire.binding_sha256,
        };
        binding.validate().map_err(serde::de::Error::custom)?;
        Ok(binding)
    }
}

impl WorkbenchBinding {
    fn new(
        selection: WorkbenchSelection,
        catalogue: Option<WorkbenchCatalogueBinding>,
        owned: Option<WorkbenchOwnedBinding>,
        contexts: Vec<WorkbenchContextObservation>,
        resolutions: Vec<OwnedCurationResolution>,
        selected_release_ids: Vec<EntityId>,
        state_sha256: String,
    ) -> Result<Self, WorkbenchError> {
        let payload = BindingPayload {
            version: BINDING_VERSION,
            kind: BINDING_KIND,
            selection: &selection,
            catalogue: &catalogue,
            owned: &owned,
            contexts: &contexts,
            resolutions: &resolutions,
            selected_release_ids: &selected_release_ids,
            state_sha256: &state_sha256,
        };
        let binding_sha256 = digest(&serde_json::to_vec(&payload)?);
        let binding = Self {
            version: BINDING_VERSION,
            kind: BINDING_KIND.to_owned(),
            selection,
            catalogue,
            owned,
            contexts,
            resolutions,
            selected_release_ids,
            state_sha256,
            binding_sha256,
        };
        binding.validate()?;
        Ok(binding)
    }

    /// Decode and validate a complete persisted binding.
    ///
    /// # Errors
    ///
    /// Returns an unsupported-version, invalid-binding, or serialization error
    /// when the strict wire, native paths, digest, or fixed layout is invalid.
    pub fn from_json_bytes(bytes: &[u8]) -> Result<Self, WorkbenchError> {
        let header: serde_json::Value =
            serde_json::from_slice(bytes).map_err(|error| WorkbenchError::InvalidBinding {
                detail: error.to_string(),
            })?;
        if let Some(version) = header.get("version").and_then(serde_json::Value::as_u64)
            && version != u64::from(BINDING_VERSION)
        {
            return Err(WorkbenchError::UnsupportedVersion {
                kind: "workbench binding",
                found: u32::try_from(version).unwrap_or(u32::MAX),
            });
        }
        let binding: Self =
            serde_json::from_slice(bytes).map_err(|error| WorkbenchError::InvalidBinding {
                detail: error.to_string(),
            })?;
        binding.validate()?;
        Ok(binding)
    }

    fn validate(&self) -> Result<(), WorkbenchError> {
        if self.version != BINDING_VERSION {
            return Err(WorkbenchError::UnsupportedVersion {
                kind: "workbench binding",
                found: self.version,
            });
        }
        if self.kind != BINDING_KIND {
            return Err(WorkbenchError::InvalidBinding {
                detail: "binding family is not vlajmaji-workbench".into(),
            });
        }
        self.selection.validate()?;
        validate_sha(&self.state_sha256, "state_sha256")?;
        validate_sha(&self.binding_sha256, "binding_sha256")?;
        if let Some(catalogue) = &self.catalogue {
            if self.selection.packs.is_empty() {
                return Err(WorkbenchError::InvalidBinding {
                    detail: "catalogue binding requires a nonempty pack selection".into(),
                });
            }
            let generation_fields = [
                catalogue.generation_id.is_some(),
                catalogue.content_id.is_some(),
                catalogue.recorded_state_sha256.is_some(),
            ];
            if generation_fields
                .iter()
                .any(|present| *present != generation_fields[0])
            {
                return Err(WorkbenchError::InvalidBinding {
                    detail: "catalogue generation identity fields must be present together".into(),
                });
            }
            if !catalogue.members.is_empty() && catalogue.generation_id.is_none() {
                return Err(WorkbenchError::InvalidBinding {
                    detail: "enabled catalogue members require a captured generation".into(),
                });
            }
            if let Some(hash) = &catalogue.recorded_state_sha256 {
                validate_sha(hash, "catalogue recorded_state_sha256")?;
            }
            decode_native_path(&catalogue.root)?;
            for member in &catalogue.members {
                validate_sha(&member.release_sha256, "release_sha256")?;
            }
            let expected: Vec<_> = self
                .selection
                .packs
                .iter()
                .filter(|pack| !self.selection.disabled.contains(pack))
                .map(|pack| pack.as_str())
                .collect();
            let actual: Vec<_> = catalogue
                .members
                .iter()
                .map(|member| member.pack_id.as_str())
                .collect();
            if expected != actual {
                return Err(WorkbenchError::InvalidBinding {
                    detail: "catalogue members do not match enabled selection order".into(),
                });
            }
        } else if !self.selection.packs.is_empty() {
            return Err(WorkbenchError::InvalidBinding {
                detail: "selected packs require a catalogue binding".into(),
            });
        }
        if let Some(owned) = &self.owned {
            decode_native_path(&owned.root)?;
            if self.selection.owned.as_ref() != Some(&owned.selection) {
                return Err(WorkbenchError::InvalidBinding {
                    detail: "owned binding differs from the declared selection".into(),
                });
            }
            if let WorkbenchOwnedRevision::Revision(revision) = &owned.selection
                && revision != &owned.identity.revision
            {
                return Err(WorkbenchError::InvalidBinding {
                    detail: "owned identity differs from the selected historical revision".into(),
                });
            }
            validate_sha(&owned.identity.state_sha256, "owned state_sha256")?;
            match (
                owned.identity.journal_schema,
                &owned.identity.curation_state_sha256,
            ) {
                (1, None) => {}
                (2, Some(hash)) => validate_sha(hash, "owned curation_state_sha256")?,
                _ => {
                    return Err(WorkbenchError::InvalidBinding {
                        detail: "owned curation identity is inconsistent with journal schema"
                            .into(),
                    });
                }
            }
            if owned.identity.journal_schema == 1
                && (!self.contexts.is_empty() || !self.resolutions.is_empty())
            {
                return Err(WorkbenchError::InvalidBinding {
                    detail:
                        "schema-one owned identity cannot retain curation contexts or resolutions"
                            .into(),
                });
            }
        } else if self.selection.owned.is_some() {
            return Err(WorkbenchError::InvalidBinding {
                detail: "owned selection requires an owned binding".into(),
            });
        } else if !self.contexts.is_empty() || !self.resolutions.is_empty() {
            return Err(WorkbenchError::InvalidBinding {
                detail: "contexts and resolutions require an owned binding".into(),
            });
        }
        let declared_pins: std::collections::BTreeSet<_> =
            self.contexts.iter().map(|row| row.pin.id.clone()).collect();
        if declared_pins.len() != self.contexts.len() {
            return Err(WorkbenchError::InvalidBinding {
                detail: "duplicate retained context pin".into(),
            });
        }
        for context in &self.contexts {
            let owned_source = self
                .owned
                .as_ref()
                .expect("context observations require an owned binding")
                .identity
                .source
                .clone();
            validate_context_pin(&context.pin, &owned_source)?;
            let declared = self
                .selection
                .contexts
                .iter()
                .find(|binding| binding.context == context.pin.id);
            match (&context.packs, &context.observed_model_sha256) {
                (None, None) if declared.is_none() => {}
                (Some(packs), Some(hash)) => {
                    validate_sha(hash, "observed_model_sha256")?;
                    if packs.is_empty()
                        || hash != &context.pin.model_sha256
                        || declared.is_none_or(|binding| &binding.packs != packs)
                    {
                        return Err(WorkbenchError::InvalidBinding { detail: "context observation differs from its pin".into() });
                    }
                    for pack in packs {
                        if !self.selection.packs.contains(pack) || self.selection.disabled.contains(pack) {
                            return Err(WorkbenchError::InvalidBinding { detail: "context observation names an unselected pack".into() });
                        }
                    }
                }
                _ => return Err(WorkbenchError::InvalidBinding { detail: "context binding and observed digest must both be present or absent and match selection".into() }),
            }
        }
        if self.selection.contexts.iter().any(|binding| {
            !self
                .contexts
                .iter()
                .any(|observation| observation.pin.id == binding.context)
        }) {
            return Err(WorkbenchError::InvalidBinding {
                detail: "selection names a context absent from the retained pin list".into(),
            });
        }
        let mut release_ids = std::collections::BTreeSet::new();
        if self
            .selected_release_ids
            .iter()
            .any(|id| !release_ids.insert(id.clone()))
        {
            return Err(WorkbenchError::InvalidBinding {
                detail: "selected release overlay contains duplicates".into(),
            });
        }
        let expected = digest(&serde_json::to_vec(&BindingPayload {
            version: self.version,
            kind: &self.kind,
            selection: &self.selection,
            catalogue: &self.catalogue,
            owned: &self.owned,
            contexts: &self.contexts,
            resolutions: &self.resolutions,
            selected_release_ids: &self.selected_release_ids,
            state_sha256: &self.state_sha256,
        })?);
        if expected != self.binding_sha256 {
            return Err(WorkbenchError::InvalidBinding {
                detail: "binding_sha256 does not match the other binding fields".into(),
            });
        }
        Ok(())
    }

    /// Binding wire version.
    #[must_use]
    pub fn version(&self) -> u32 {
        self.version
    }
    /// Stable binding family marker.
    #[must_use]
    pub fn kind(&self) -> &str {
        &self.kind
    }
    /// Complete original request.
    #[must_use]
    pub fn selection(&self) -> &WorkbenchSelection {
        &self.selection
    }
    /// Captured catalogue identity, if selected.
    #[must_use]
    pub fn catalogue(&self) -> Option<&WorkbenchCatalogueBinding> {
        self.catalogue.as_ref()
    }
    /// Captured owned revision, if selected.
    #[must_use]
    pub fn owned(&self) -> Option<&WorkbenchOwnedBinding> {
        self.owned.as_ref()
    }
    /// Complete retained pin observations in declaration order.
    #[must_use]
    pub fn contexts(&self) -> &[WorkbenchContextObservation] {
        &self.contexts
    }
    /// Complete typed resolution outcomes in curation order.
    #[must_use]
    pub fn resolutions(&self) -> &[OwnedCurationResolution] {
        &self.resolutions
    }
    /// Producer-selected available release overlay.
    #[must_use]
    pub fn selected_release_ids(&self) -> &[EntityId] {
        &self.selected_release_ids
    }
    /// Canonical materialized dictionary digest.
    #[must_use]
    pub fn state_sha256(&self) -> &str {
        &self.state_sha256
    }
    /// Digest of the other binding fields.
    #[must_use]
    pub fn binding_sha256(&self) -> &str {
        &self.binding_sha256
    }
}

/// Successful completion of a new workbench build.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkbenchBuildReport {
    /// Response version.
    pub version: u32,
    /// Stable operation name.
    pub operation: String,
    /// Exact native output root.
    pub root: NativePathReport,
    /// Persisted workbench identity.
    pub binding: WorkbenchBinding,
}

/// A currently opened immutable workbench binding.
pub struct WorkbenchView {
    root: PathBuf,
    binding: WorkbenchBinding,
}

impl fmt::Debug for WorkbenchView {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WorkbenchView")
            .field("root", &self.root)
            .field("binding_sha256", &self.binding.binding_sha256)
            .finish_non_exhaustive()
    }
}

/// Dependency whose current recorded identity is guarded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkbenchDependency {
    /// Installed collection catalogue state.
    Catalogue,
    /// Selected owned journal revision.
    Owned,
}

/// Typed workbench construction, open, freshness, and persisted binding errors.
#[derive(Debug)]
#[non_exhaustive]
pub enum WorkbenchError {
    /// The caller supplied an inconsistent selection.
    InvalidSelection {
        /// Description of the inconsistent selection.
        detail: String,
    },
    /// A workbench wire version is unsupported.
    UnsupportedVersion {
        /// Workbench record kind with the unsupported version.
        kind: &'static str,
        /// Version value found in the record.
        found: u32,
    },
    /// The current platform cannot safely round-trip a path encoding.
    UnsupportedNativeEncoding {
        /// Native path encoding that cannot be round-tripped.
        encoding: String,
    },
    /// The destination exists, including a dangling symlink.
    ExistingDestination(PathBuf),
    /// A persisted binding is malformed or inconsistent.
    InvalidBinding {
        /// Description of the invalid persisted binding.
        detail: String,
    },
    /// A selected dependency no longer has the captured identity.
    StaleDependencies {
        /// Dependency whose current identity differs from the captured one.
        dependency: WorkbenchDependency,
        /// Identity recorded in the workbench binding.
        expected: String,
        /// Current identity, absent when the dependency disappeared.
        observed: Option<String>,
    },
    /// A filesystem operation failed at the named path.
    Io {
        /// Filesystem path associated with the failed operation.
        path: PathBuf,
        /// Underlying filesystem error.
        source: io::Error,
    },
    /// A catalogue operation failed.
    Collection(Box<CollectionError>),
    /// An owned-journal operation failed.
    Owned(Box<OwnedJournalError>),
    /// A sealed pack failed validation or loading.
    Pack(Box<PackError>),
    /// A curation materialization failed.
    Curation(Box<vlajmaji_lexicon::OwnedCurationError>),
    /// The new dictionary store failed to build or verify.
    Store(Box<DictionaryStoreError>),
    /// A typed workbench value could not be serialized.
    Serialization(serde_json::Error),
}

impl fmt::Display for WorkbenchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSelection { detail } => write!(f, "invalid workbench selection: {detail}"),
            Self::UnsupportedVersion { kind, found } => {
                write!(f, "unsupported {kind} version {found}")
            }
            Self::UnsupportedNativeEncoding { encoding } => {
                write!(f, "unsupported native path encoding: {encoding}")
            }
            Self::ExistingDestination(path) => {
                write!(
                    f,
                    "workbench destination already exists: {}",
                    path.display()
                )
            }
            Self::InvalidBinding { detail } => write!(f, "invalid workbench binding: {detail}"),
            Self::StaleDependencies {
                dependency,
                expected,
                observed,
            } => write!(
                f,
                "stale {dependency:?} dependency: expected {expected}, observed {observed:?}"
            ),
            Self::Io { path, source } => write!(f, "workbench IO at {}: {source}", path.display()),
            Self::Collection(error) => write!(f, "workbench catalogue: {error}"),
            Self::Owned(error) => write!(f, "workbench owned journal: {error}"),
            Self::Pack(error) => write!(f, "workbench pack: {error}"),
            Self::Curation(error) => write!(f, "workbench curation: {error}"),
            Self::Store(error) => write!(f, "workbench dictionary store: {error}"),
            Self::Serialization(error) => write!(f, "workbench serialization: {error}"),
        }
    }
}

impl Error for WorkbenchError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::Collection(error) => Some(error.as_ref()),
            Self::Owned(error) => Some(error.as_ref()),
            Self::Pack(error) => Some(error.as_ref()),
            Self::Curation(error) => Some(error.as_ref()),
            Self::Store(error) => Some(error.as_ref()),
            Self::Serialization(error) => Some(error),
            _ => None,
        }
    }
}

impl From<CollectionError> for WorkbenchError {
    fn from(value: CollectionError) -> Self {
        Self::Collection(Box::new(value))
    }
}
impl From<OwnedJournalError> for WorkbenchError {
    fn from(value: OwnedJournalError) -> Self {
        Self::Owned(Box::new(value))
    }
}
impl From<PackError> for WorkbenchError {
    fn from(value: PackError) -> Self {
        Self::Pack(Box::new(value))
    }
}
impl From<vlajmaji_lexicon::OwnedCurationError> for WorkbenchError {
    fn from(value: vlajmaji_lexicon::OwnedCurationError) -> Self {
        Self::Curation(Box::new(value))
    }
}
impl From<DictionaryStoreError> for WorkbenchError {
    fn from(value: DictionaryStoreError) -> Self {
        Self::Store(Box::new(value))
    }
}
impl From<serde_json::Error> for WorkbenchError {
    fn from(value: serde_json::Error) -> Self {
        Self::Serialization(value)
    }
}

impl WorkbenchView {
    /// Build a new immutable workbench at an exclusively created destination.
    ///
    /// # Errors
    ///
    /// Returns a selection, dependency, context, destination, integrity, or IO
    /// error. A failed new candidate is retained for diagnosis and is never
    /// replaced or cleaned up.
    pub fn build_new(
        destination: impl AsRef<Path>,
        request: WorkbenchBuildRequest,
    ) -> Result<WorkbenchBuildReport, WorkbenchError> {
        Self::build_new_with_hook(destination, request, |_| Ok(()))
    }

    fn build_new_with_hook<F>(
        destination: impl AsRef<Path>,
        request: WorkbenchBuildRequest,
        mut hook: F,
    ) -> Result<WorkbenchBuildReport, WorkbenchError>
    where
        F: FnMut(BuildGuardStage) -> Result<(), WorkbenchError>,
    {
        request.selection.validate()?;
        validate_roots(&request)?;
        let destination_input = destination.as_ref();
        let needs_cwd = !destination_input.is_absolute()
            || request
                .catalogue_root
                .as_deref()
                .is_some_and(|path| !path.is_absolute())
            || request
                .owned_root
                .as_deref()
                .is_some_and(|path| !path.is_absolute());
        let cwd = needs_cwd
            .then(std::env::current_dir)
            .transpose()
            .map_err(|source| WorkbenchError::Io {
                path: PathBuf::from("."),
                source,
            })?;
        let destination = absolute_from(cwd.as_deref(), destination_input);
        preflight_destination(&destination)?;

        let catalogue_root = request
            .catalogue_root
            .as_ref()
            .map(|path| absolute_from(cwd.as_deref(), path));
        let owned_root = request
            .owned_root
            .as_ref()
            .map(|path| absolute_from(cwd.as_deref(), path));
        let (catalogue_binding, pack_models, mut selected_release_ids) =
            capture_catalogue(&request.selection, catalogue_root.as_deref())?;
        let (owned_binding, owned_dictionary, curation, context_inputs) =
            capture_owned(&request.selection, owned_root.as_deref(), &pack_models)?;
        let owned_base = curation
            .as_ref()
            .map(|document| &document.as_data().dictionary)
            .or(owned_dictionary.as_ref());
        if let Some(dictionary) = owned_base {
            for release in dictionary
                .metadata()
                .releases
                .iter()
                .filter(|row| row.selected && row.availability == Availability::Available)
            {
                if !selected_release_ids.contains(&release.id) {
                    selected_release_ids.push(release.id.clone());
                }
            }
        }
        let contexts =
            capture_context_observations(&request.selection, curation.as_ref(), &context_inputs)?;
        let (materialized, resolutions) = if let Some(document) = curation {
            let output = document.materialize(context_inputs)?;
            (Some(output.dictionary), output.resolutions)
        } else {
            (owned_dictionary, Vec::new())
        };
        let mut dictionaries: Vec<Dictionary> = pack_models
            .into_iter()
            .map(|(_, dictionary)| dictionary)
            .collect();
        if let Some(dictionary) = materialized {
            dictionaries.push(dictionary);
        }
        if dictionaries.is_empty() {
            return Err(invalid_selection("selection has no effective input"));
        }
        let composed = compose_dictionaries(dictionaries)?;
        let state_sha256 = digest(&serde_json::to_vec(&composed)?);
        hook(BuildGuardStage::BeforeReservation)?;
        ensure_captured_dependencies_current(
            catalogue_root.as_deref(),
            owned_root.as_deref(),
            &request.selection,
            catalogue_binding.as_ref(),
            owned_binding.as_ref(),
        )?;
        preflight_destination(&destination)?;
        fs::create_dir(&destination).map_err(|error| {
            if error.kind() == io::ErrorKind::AlreadyExists {
                WorkbenchError::ExistingDestination(destination.clone())
            } else {
                WorkbenchError::Io {
                    path: destination.clone(),
                    source: error,
                }
            }
        })?;
        let dictionary_path = destination.join("dictionary");
        let store = DictionaryStore::create(&dictionary_path, &composed)?;
        let measured_state = store.recorded_state_sha256().to_owned();
        drop(store);
        if measured_state != state_sha256 {
            return Err(WorkbenchError::InvalidBinding {
                detail: "new dictionary store recorded a different state digest".into(),
            });
        }
        hook(BuildGuardStage::BeforeBinding)?;
        ensure_captured_dependencies_current(
            catalogue_root.as_deref(),
            owned_root.as_deref(),
            &request.selection,
            catalogue_binding.as_ref(),
            owned_binding.as_ref(),
        )?;
        let binding = WorkbenchBinding::new(
            request.selection.clone(),
            catalogue_binding,
            owned_binding,
            contexts,
            resolutions,
            selected_release_ids,
            state_sha256,
        )?;
        write_new_synced(
            &destination.join("binding.json"),
            &serde_json::to_vec(&binding)?,
        )?;
        hook(BuildGuardStage::AfterBinding)?;
        ensure_binding_dependencies_current(&binding)?;
        Ok(WorkbenchBuildReport {
            version: 1,
            operation: "build_new".into(),
            root: native_path_report(&destination),
            binding,
        })
    }

    /// Open an existing binding and check its current recorded dependencies.
    ///
    /// # Errors
    ///
    /// Returns a typed IO, binding, unsupported-version, or stale-dependency error.
    pub fn open(root: impl AsRef<Path>) -> Result<Self, WorkbenchError> {
        let root = root.as_ref();
        let cwd = if root.is_absolute() {
            None
        } else {
            Some(
                std::env::current_dir().map_err(|source| WorkbenchError::Io {
                    path: PathBuf::from("."),
                    source,
                })?,
            )
        };
        let root = absolute_from(cwd.as_deref(), root);
        let metadata = fs::symlink_metadata(&root).map_err(|source| WorkbenchError::Io {
            path: root.clone(),
            source,
        })?;
        if !metadata.file_type().is_dir() || metadata.file_type().is_symlink() {
            return Err(WorkbenchError::InvalidBinding {
                detail: "workbench root must be a real directory".into(),
            });
        }
        let path = root.join("binding.json");
        let file_metadata = fs::symlink_metadata(&path).map_err(|source| WorkbenchError::Io {
            path: path.clone(),
            source,
        })?;
        if !file_metadata.file_type().is_file() || file_metadata.file_type().is_symlink() {
            return Err(WorkbenchError::InvalidBinding {
                detail: "binding.json must be a regular file".into(),
            });
        }
        let bytes = fs::read(&path).map_err(|source| WorkbenchError::Io {
            path: path.clone(),
            source,
        })?;
        let binding = WorkbenchBinding::from_json_bytes(&bytes)?;
        let dictionary_path = root.join("dictionary");
        let dictionary_metadata =
            fs::symlink_metadata(&dictionary_path).map_err(|source| WorkbenchError::Io {
                path: dictionary_path.clone(),
                source,
            })?;
        if !dictionary_metadata.file_type().is_dir() || dictionary_metadata.file_type().is_symlink()
        {
            return Err(WorkbenchError::InvalidBinding {
                detail: "dictionary must be a real directory".into(),
            });
        }
        let dictionary_store = DictionaryStore::open(&dictionary_path)?;
        if dictionary_store.recorded_state_sha256() != binding.state_sha256 {
            return Err(WorkbenchError::InvalidBinding {
                detail: "dictionary store header differs from binding state_sha256".into(),
            });
        }
        drop(dictionary_store);
        let view = Self { root, binding };
        view.ensure_current()?;
        Ok(view)
    }

    /// Borrow the complete checked binding.
    #[must_use]
    pub fn binding(&self) -> &WorkbenchBinding {
        &self.binding
    }

    /// Return the Unified dictionary snapshot manifest for this immutable view.
    #[must_use]
    pub fn snapshot_manifest(&self) -> DictionarySnapshotManifest {
        DictionarySnapshotManifest {
            version: 1,
            generation: self.binding.binding_sha256.clone(),
            logical_state_sha256: self.binding.state_sha256.clone(),
            members: vec![crate::DictionarySnapshotMember {
                id: self.binding.binding_sha256.clone(),
                path: self.root.join("dictionary"),
                expected_state_sha256: self.binding.state_sha256.clone(),
            }],
            selected_release_ids: Some(self.binding.selected_release_ids.clone()),
        }
    }

    /// Check fresh short-lived dependency identities against this binding.
    ///
    /// # Errors
    ///
    /// Returns a typed stale, IO, catalogue, or owned-journal error.
    pub fn ensure_current(&self) -> Result<(), WorkbenchError> {
        ensure_binding_dependencies_current(&self.binding)
    }

    /// Return the native workbench root without changing it.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }
}

type CatalogueCapture = (
    Option<WorkbenchCatalogueBinding>,
    Vec<(PackId, Dictionary)>,
    Vec<EntityId>,
);

type OwnedCapture = (
    Option<WorkbenchOwnedBinding>,
    Option<Dictionary>,
    Option<OwnedCurationDocument>,
    Vec<OwnedContextInput>,
);

fn capture_catalogue(
    selection: &WorkbenchSelection,
    root: Option<&Path>,
) -> Result<CatalogueCapture, WorkbenchError> {
    if selection.packs.is_empty() {
        return Ok((None, Vec::new(), Vec::new()));
    }
    let root = root.ok_or_else(|| invalid_selection("catalogue root is required"))?;
    let catalogue = CollectionCatalogue::open(root)?;
    let local_revision = catalogue.local_revision()?;
    let generation = catalogue.generation()?;
    let mut members = Vec::new();
    let mut dictionaries = Vec::new();
    let mut selected_release_ids = Vec::new();
    for pack in selection
        .packs
        .iter()
        .copied()
        .filter(|pack| !selection.disabled.contains(pack))
    {
        let locations = catalogue
            .installed_artifact_locations(pack)?
            .ok_or_else(|| {
                invalid_selection(format!(
                    "selected pack {pack} has no current installed release"
                ))
            })?;
        let generation_member = generation
            .as_ref()
            .and_then(|generation| {
                generation
                    .members
                    .iter()
                    .find(|member| member.pack_id == pack.as_str())
            })
            .ok_or_else(|| {
                invalid_selection(format!(
                    "selected pack {pack} is absent from current generation"
                ))
            })?;
        let release = fs::read(&locations.release_path).map_err(|source| WorkbenchError::Io {
            path: locations.release_path.clone(),
            source,
        })?;
        let seal = fs::read(&locations.seal_path).map_err(|source| WorkbenchError::Io {
            path: locations.seal_path.clone(),
            source,
        })?;
        let actual_release_sha256 = digest(&release);
        let dictionary = load_sealed_release(pack, &release, &seal)?;
        let decoded: crate::CollectionGenerationMember = generation_member.clone();
        if decoded.release_sha256 != locations.release_sha256
            || actual_release_sha256 != locations.release_sha256
        {
            return Err(invalid_selection(format!(
                "selected pack {pack} release differs from current generation"
            )));
        }
        let payload_kind = seal_payload_kind(&seal)?;
        if decoded.payload_kind != pack_payload_kind_name(payload_kind) {
            return Err(invalid_selection(format!(
                "selected pack {pack} payload kind differs from current generation"
            )));
        }
        for release in dictionary
            .metadata()
            .releases
            .iter()
            .filter(|row| row.selected && row.availability == Availability::Available)
        {
            if !selected_release_ids.contains(&release.id) {
                selected_release_ids.push(release.id.clone());
            }
        }
        members.push(WorkbenchCatalogueMember {
            pack_id: pack.as_str().to_owned(),
            release_sha256: locations.release_sha256,
            payload_kind,
        });
        dictionaries.push((pack, dictionary));
    }
    let binding = WorkbenchCatalogueBinding {
        root: native_path_report(root),
        local_revision,
        generation_id: generation.as_ref().map(|value| value.generation_id.clone()),
        content_id: generation.as_ref().map(|value| value.content_id.clone()),
        recorded_state_sha256: generation
            .as_ref()
            .map(|value| value.canonical_sha256.clone()),
        members,
    };
    Ok((Some(binding), dictionaries, selected_release_ids))
}

fn capture_owned(
    selection: &WorkbenchSelection,
    root: Option<&Path>,
    pack_dictionaries: &[(PackId, Dictionary)],
) -> Result<OwnedCapture, WorkbenchError> {
    let Some(owned_selection) = &selection.owned else {
        return Ok((None, None, None, Vec::new()));
    };
    let root = root.ok_or_else(|| invalid_selection("owned root is required"))?;
    let reader = OwnedJournalReader::open(root)?;
    let identity = match owned_selection {
        WorkbenchOwnedRevision::Head => reader.head_identity()?,
        WorkbenchOwnedRevision::Revision(id) => reader.revision_identity(id)?,
    };
    let revision = identity.revision.clone();
    let (dictionary, curation) = if identity.journal_schema == 1 {
        (Some(reader.show(&revision)?.dictionary), None)
    } else {
        let document = reader.curation_show(&revision)?;
        (None, Some(document))
    };
    drop(reader);
    let pin_map = curation
        .as_ref()
        .map(|document| &document.as_data().contexts);
    let mut context_inputs = Vec::new();
    if let Some(pins) = pin_map {
        for pin in pins {
            if let Some(binding) = selection
                .contexts
                .iter()
                .find(|binding| binding.context == pin.id)
            {
                let mut matching = Vec::new();
                for pack in &binding.packs {
                    let (_, dictionary) = pack_dictionaries
                        .iter()
                        .find(|(candidate, _)| candidate == pack)
                        .ok_or_else(|| {
                            invalid_selection(format!(
                                "context {} binds an unavailable pack",
                                pin.id
                            ))
                        })?;
                    matching.push(dictionary.clone());
                }
                let dictionary = if matching.len() == 1 {
                    matching.pop().expect("one dictionary exists")
                } else {
                    compose_dictionaries(matching)?
                };
                context_inputs.push(OwnedContextInput {
                    id: pin.id.clone(),
                    dictionary,
                });
            }
        }
        if selection
            .contexts
            .iter()
            .any(|binding| !pins.iter().any(|pin| pin.id == binding.context))
        {
            return Err(invalid_selection(
                "selection binds an undeclared curation pin",
            ));
        }
    } else if !selection.contexts.is_empty() {
        return Err(invalid_selection(
            "schema-one owned revisions have no curation contexts",
        ));
    }
    let owned = WorkbenchOwnedBinding {
        root: native_path_report(root),
        selection: owned_selection.clone(),
        identity,
    };
    Ok((Some(owned), dictionary, curation, context_inputs))
}

fn capture_context_observations(
    selection: &WorkbenchSelection,
    curation: Option<&OwnedCurationDocument>,
    inputs: &[OwnedContextInput],
) -> Result<Vec<WorkbenchContextObservation>, WorkbenchError> {
    let pins = curation
        .map(|document| document.as_data().contexts.as_slice())
        .unwrap_or(&[]);
    pins.iter()
        .map(|pin| {
            let binding = selection
                .contexts
                .iter()
                .find(|item| item.context == pin.id);
            let input = inputs.iter().find(|item| item.id == pin.id);
            let observed_model_sha256 = input
                .map(|item| serde_json::to_vec(&item.dictionary).map(|bytes| digest(&bytes)))
                .transpose()?;
            Ok(WorkbenchContextObservation {
                pin: pin.clone(),
                packs: binding.map(|item| item.packs.clone()),
                observed_model_sha256,
            })
        })
        .collect()
}

fn ensure_captured_dependencies_current(
    catalogue_root: Option<&Path>,
    owned_root: Option<&Path>,
    selection: &WorkbenchSelection,
    catalogue: Option<&WorkbenchCatalogueBinding>,
    owned: Option<&WorkbenchOwnedBinding>,
) -> Result<(), WorkbenchError> {
    if let (Some(root), Some(expected)) = (catalogue_root, catalogue) {
        let actual = current_catalogue_binding(root, selection)?;
        if actual.as_ref() != Some(expected) {
            return Err(WorkbenchError::StaleDependencies {
                dependency: WorkbenchDependency::Catalogue,
                expected: digest(&serde_json::to_vec(expected)?),
                observed: actual
                    .as_ref()
                    .map(|value| serde_json::to_vec(value).map(|bytes| digest(&bytes)))
                    .transpose()?,
            });
        }
    }
    if let (Some(root), Some(expected)) = (owned_root, owned) {
        let reader = OwnedJournalReader::open(root)?;
        let observed = match expected.selection {
            WorkbenchOwnedRevision::Head => reader.head_identity()?,
            WorkbenchOwnedRevision::Revision(ref id) => reader.revision_identity(id)?,
        };
        drop(reader);
        if observed != expected.identity {
            return Err(WorkbenchError::StaleDependencies {
                dependency: WorkbenchDependency::Owned,
                expected: identity_digest(&expected.identity)?,
                observed: Some(identity_digest(&observed)?),
            });
        }
    }
    Ok(())
}

fn current_catalogue_binding(
    root: &Path,
    selection: &WorkbenchSelection,
) -> Result<Option<WorkbenchCatalogueBinding>, WorkbenchError> {
    let catalogue = CollectionCatalogue::open(root)?;
    let local_revision = catalogue.local_revision()?;
    let generation = catalogue.generation()?;
    let mut members = Vec::new();
    if !selection
        .packs
        .iter()
        .all(|pack| selection.disabled.contains(pack))
    {
        let Some(generation_value) = &generation else {
            return Ok(None);
        };
        for pack in selection
            .packs
            .iter()
            .copied()
            .filter(|pack| !selection.disabled.contains(pack))
        {
            let Some(generation_member) = generation_value
                .members
                .iter()
                .find(|member| member.pack_id == pack.as_str())
            else {
                return Ok(None);
            };
            let payload_kind = match generation_member.payload_kind.as_str() {
                "synthetic" => PackPayloadKind::Synthetic,
                "full" => PackPayloadKind::Full,
                _ => return Ok(None),
            };
            members.push(WorkbenchCatalogueMember {
                pack_id: pack.as_str().to_owned(),
                release_sha256: generation_member.release_sha256.clone(),
                payload_kind,
            });
        }
    }
    Ok(Some(WorkbenchCatalogueBinding {
        root: native_path_report(root),
        local_revision,
        generation_id: generation.as_ref().map(|value| value.generation_id.clone()),
        content_id: generation.as_ref().map(|value| value.content_id.clone()),
        recorded_state_sha256: generation
            .as_ref()
            .map(|value| value.canonical_sha256.clone()),
        members,
    }))
}

fn ensure_binding_dependencies_current(binding: &WorkbenchBinding) -> Result<(), WorkbenchError> {
    if let Some(catalogue) = &binding.catalogue {
        let root = decode_native_path(&catalogue.root)?;
        let current = current_catalogue_binding(&root, &binding.selection)?;
        if current.as_ref() != Some(catalogue) {
            return Err(WorkbenchError::StaleDependencies {
                dependency: WorkbenchDependency::Catalogue,
                expected: digest(&serde_json::to_vec(catalogue)?),
                observed: current
                    .as_ref()
                    .map(|value| serde_json::to_vec(value).map(|bytes| digest(&bytes)))
                    .transpose()?,
            });
        }
    }
    if let Some(owned) = &binding.owned {
        let root = decode_native_path(&owned.root)?;
        let reader = OwnedJournalReader::open(&root)?;
        let current = match owned.selection {
            WorkbenchOwnedRevision::Head => reader.head_identity()?,
            WorkbenchOwnedRevision::Revision(ref id) => reader.revision_identity(id)?,
        };
        drop(reader);
        if current != owned.identity {
            return Err(WorkbenchError::StaleDependencies {
                dependency: WorkbenchDependency::Owned,
                expected: identity_digest(&owned.identity)?,
                observed: Some(identity_digest(&current)?),
            });
        }
    }
    Ok(())
}

fn validate_roots(request: &WorkbenchBuildRequest) -> Result<(), WorkbenchError> {
    if request.selection.packs.is_empty() != request.catalogue_root.is_none() {
        return Err(invalid_selection(
            "catalogue root must be supplied exactly when packs are selected",
        ));
    }
    if request.selection.owned.is_some() != request.owned_root.is_some() {
        return Err(invalid_selection(
            "owned root must be supplied exactly when an owned revision is selected",
        ));
    }
    Ok(())
}

fn preflight_destination(path: &Path) -> Result<(), WorkbenchError> {
    match fs::symlink_metadata(path) {
        Ok(_) => Err(WorkbenchError::ExistingDestination(path.to_path_buf())),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            let parent = path
                .parent()
                .filter(|parent| !parent.as_os_str().is_empty())
                .unwrap_or_else(|| Path::new("."));
            if !parent.is_dir() {
                return Err(WorkbenchError::Io {
                    path: parent.to_path_buf(),
                    source: io::Error::new(
                        io::ErrorKind::NotFound,
                        "workbench parent must already exist",
                    ),
                });
            }
            Ok(())
        }
        Err(source) => Err(WorkbenchError::Io {
            path: path.to_path_buf(),
            source,
        }),
    }
}

fn write_new_synced(path: &Path, bytes: &[u8]) -> Result<(), WorkbenchError> {
    use std::io::Write as _;
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path).map_err(|source| WorkbenchError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    file.write_all(bytes)
        .and_then(|_| file.flush())
        .and_then(|_| file.sync_all())
        .map_err(|source| WorkbenchError::Io {
            path: path.to_path_buf(),
            source,
        })
}

fn absolute_from(cwd: Option<&Path>, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.expect("relative paths require a captured cwd")
            .join(path)
    }
}

fn deserialize_required_option<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: DeserializeOwned,
{
    Option::<T>::deserialize(deserializer)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NativePathReportWire {
    encoding: String,
    hex: String,
    #[serde(deserialize_with = "deserialize_required_option")]
    utf8: Option<String>,
}

fn deserialize_native_path<'de, D>(deserializer: D) -> Result<NativePathReport, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let wire = NativePathReportWire::deserialize(deserializer)?;
    Ok(NativePathReport {
        encoding: wire.encoding,
        hex: wire.hex,
        utf8: wire.utf8,
    })
}

fn invalid_selection(detail: impl Into<String>) -> WorkbenchError {
    WorkbenchError::InvalidSelection {
        detail: detail.into(),
    }
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn identity_digest(identity: &OwnedRevisionIdentity) -> Result<String, WorkbenchError> {
    Ok(digest(&serde_json::to_vec(identity)?))
}

fn validate_context_pin(
    pin: &vlajmaji_lexicon::OwnedContextPin,
    owned_source: &vlajmaji_lexicon::SourceId,
) -> Result<(), WorkbenchError> {
    validate_sha(&pin.model_sha256, "context pin model_sha256")?;
    if pin.source_ids.windows(2).any(|pair| pair[0] >= pair[1])
        || pin.release_ids.windows(2).any(|pair| pair[0] >= pair[1])
        || pin.source_ids.contains(owned_source)
    {
        return Err(WorkbenchError::InvalidBinding {
            detail: "context pin inventories must be sorted, unique, and exclude the owned source"
                .into(),
        });
    }
    Ok(())
}

fn validate_sha(value: &str, label: &str) -> Result<(), WorkbenchError> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(WorkbenchError::InvalidBinding {
            detail: format!("{label} is not lowercase SHA-256"),
        });
    }
    Ok(())
}

fn decode_native_path(report: &NativePathReport) -> Result<PathBuf, WorkbenchError> {
    if !report.hex.len().is_multiple_of(2)
        || !report
            .hex
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(WorkbenchError::InvalidBinding {
            detail: "native path hex is malformed".into(),
        });
    }
    let bytes = report
        .hex
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| {
            let digit = |byte: u8| match byte {
                b'0'..=b'9' => byte - b'0',
                b'a'..=b'f' => byte - b'a' + 10,
                _ => 0,
            };
            digit(pair[0]) * 16 + digit(pair[1])
        })
        .collect::<Vec<_>>();
    if bytes.contains(&0) {
        return Err(WorkbenchError::InvalidBinding {
            detail: "native path contains NUL".into(),
        });
    }
    #[cfg(unix)]
    let path = {
        use std::os::unix::ffi::OsStringExt;
        if report.encoding != "unix-bytes" {
            return Err(WorkbenchError::UnsupportedNativeEncoding {
                encoding: report.encoding.clone(),
            });
        }
        PathBuf::from(std::ffi::OsString::from_vec(bytes))
    };
    #[cfg(not(unix))]
    let path = return Err(WorkbenchError::UnsupportedNativeEncoding {
        encoding: report.encoding.clone(),
    });
    if !path.is_absolute() || path.to_str() != report.utf8.as_deref() {
        return Err(WorkbenchError::InvalidBinding {
            detail: "native path absolute/UTF-8 fields disagree".into(),
        });
    }
    Ok(path)
}

fn seal_payload_kind(bytes: &[u8]) -> Result<PackPayloadKind, WorkbenchError> {
    let value: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|error| WorkbenchError::InvalidBinding {
            detail: error.to_string(),
        })?;
    let kind = value
        .get("payload_kind")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| WorkbenchError::InvalidBinding {
            detail: "seal payload_kind is absent".into(),
        })?;
    match kind {
        "synthetic" => Ok(PackPayloadKind::Synthetic),
        "full" => Ok(PackPayloadKind::Full),
        _ => Err(WorkbenchError::InvalidBinding {
            detail: "seal payload_kind is unsupported".into(),
        }),
    }
}

fn pack_payload_kind_name(kind: PackPayloadKind) -> &'static str {
    match kind {
        PackPayloadKind::Synthetic => "synthetic",
        PackPayloadKind::Full => "full",
    }
}

#[cfg(test)]
mod tests {
    use std::{
        borrow::Cow,
        fs,
        path::{Path, PathBuf},
        sync::atomic::{AtomicUsize, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    use vlajmaji_lexicon::{
        AbbreviationKind, Availability, CollectionRecord, ComponentRecord, ContentNotice,
        ContributorId, ContributorRecord, Dictionary, DictionaryDocument, DictionaryMetadata,
        EntityId, OwnedHead, OwnedRevisionRecord, PackId, PackPayloadKind, Presence,
        RedistributionStatus, RegistryDocument, ReleaseAccounting, ReleaseRecord, SourceId,
        SourceOrigin, SourceRecord, SourceRegistry, TextValue, TimePrecision, seal_release,
    };

    use super::*;

    static NEXT_ROOT: AtomicUsize = AtomicUsize::new(0);

    fn fresh_root(label: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "vlajmaji-workbench-guard-{label}-{}-{}-{}",
            std::process::id(),
            NEXT_ROOT.fetch_add(1, Ordering::Relaxed),
            stamp,
        ))
    }

    fn install(root: &Path, pack: PackId) {
        let dictionary = Dictionary::from_document(Default::default()).unwrap();
        let sealed = seal_release(pack, PackPayloadKind::Synthetic, &dictionary).unwrap();
        let request = crate::CollectionRequest::new(
            vec![crate::CollectionInput::SealedBytes {
                pack_id: pack,
                release: Cow::Borrowed(&sealed.release),
                seal: Cow::Borrowed(&sealed.seal),
            }],
            crate::CollectionSelection::Packs(vec![pack]),
            true,
        )
        .unwrap();
        CollectionCatalogue::install(root, request).unwrap();
    }

    fn register(root: &Path, pack: PackId) {
        let dictionary = Dictionary::from_document(Default::default()).unwrap();
        let sealed = seal_release(pack, PackPayloadKind::Synthetic, &dictionary).unwrap();
        let request = crate::CollectionRequest::new(
            vec![crate::CollectionInput::SealedBytes {
                pack_id: pack,
                release: Cow::Borrowed(&sealed.release),
                seal: Cow::Borrowed(&sealed.seal),
            }],
            crate::CollectionSelection::Packs(vec![pack]),
            true,
        )
        .unwrap();
        CollectionCatalogue::register(root, request).unwrap();
    }

    fn owned_genesis() -> Dictionary {
        let source = SourceId::new("workbench-guard-owned").unwrap();
        let editor = ContributorId::new("workbench-guard-editor").unwrap();
        let collection = EntityId::new("workbench-guard-collection").unwrap();
        let component = EntityId::new("workbench-guard-component").unwrap();
        let revision = EntityId::new("workbench-guard-r0").unwrap();
        let registry = SourceRegistry::from_document(RegistryDocument {
            sources: vec![SourceRecord {
                id: source.clone(),
                display_name: "Owned workbench guard".into(),
                abbreviation: "OWG".into(),
                abbreviation_kind: AbbreviationKind::ProjectAssigned,
                abbreviation_evidence: None,
                origin: SourceOrigin::OwnedDraft,
            }],
            contributors: vec![ContributorRecord {
                id: editor.clone(),
                display_name: "Guard fixture".into(),
            }],
            ..RegistryDocument::default()
        })
        .unwrap();
        let mut metadata = DictionaryMetadata::default();
        metadata.collections.push(CollectionRecord {
            id: collection.clone(),
            source: source.clone(),
            display_name: "Owned guard collection".into(),
            description: Presence::Missing,
        });
        metadata.components.push(ComponentRecord {
            id: component.clone(),
            collection,
            native_name: "guard words".into(),
            languages: vec!["jbo".into()],
        });
        metadata.releases.push(ReleaseRecord {
            id: EntityId::new("workbench-guard-release-r0").unwrap(),
            component,
            distributor: None,
            upstream_revision: Presence::Missing,
            distributor_revision: Presence::Missing,
            captured_at: Presence::Missing,
            time_precision: TimePrecision::Unknown,
            schema_version: 1,
            normalizer: "guard fixture".into(),
            normalizer_version: "1".into(),
            captures: Vec::new(),
            assertions: Vec::new(),
            accounting: ReleaseAccounting {
                input_occurrences: Some(0),
                normalized_occurrences: Some(0),
                query_excluded_occurrences: Some(0),
                unit: "occurrences".into(),
            },
            diagnostics: Vec::new(),
            selected: true,
            availability: Availability::Available,
            content_notices: vec![ContentNotice {
                text: Presence::Value("synthetic fixture; no rights grant".into()),
                locator: Presence::Missing,
                license: Presence::Missing,
            }],
            redistribution: RedistributionStatus::Held,
            redistribution_evidence: Vec::new(),
        });
        metadata.owned_revisions.push(OwnedRevisionRecord {
            id: revision.clone(),
            source: source.clone(),
            parent: None,
            editor,
            timestamp: Presence::Missing,
            time_precision: TimePrecision::Unknown,
            assertions: Vec::new(),
            tombstones: Vec::new(),
            rationale: Presence::Value(TextValue {
                text: "genesis".into(),
                language: Presence::Missing,
                markup: Presence::Missing,
            }),
        });
        metadata.owned_heads.push(OwnedHead { source, revision });
        Dictionary::from_document(DictionaryDocument {
            registry,
            metadata,
            ..DictionaryDocument::default()
        })
        .unwrap()
    }

    fn owned_revision_one(seed: &Dictionary) -> Dictionary {
        let mut document = seed.document();
        let previous = document.metadata.owned_revisions[0].clone();
        let revision = EntityId::new("workbench-guard-r1").unwrap();
        document.metadata.catalogue_revision = 1;
        document.metadata.owned_revisions.push(OwnedRevisionRecord {
            id: revision.clone(),
            source: previous.source,
            parent: Some(previous.id),
            editor: previous.editor,
            timestamp: Presence::Missing,
            time_precision: TimePrecision::Unknown,
            assertions: Vec::new(),
            tombstones: Vec::new(),
            rationale: Presence::Value(TextValue {
                text: "advance head".into(),
                language: Presence::Missing,
                markup: Presence::Missing,
            }),
        });
        document.metadata.owned_heads[0].revision = revision;
        Dictionary::from_document(document).unwrap()
    }

    fn test_binding_with_omitted_context_pin() -> WorkbenchBinding {
        let source = SourceId::new("binding-owned-source").unwrap();
        let revision = EntityId::new("binding-owned-r0").unwrap();
        let selection = WorkbenchSelection {
            version: 1,
            packs: Vec::new(),
            disabled: Vec::new(),
            owned: Some(WorkbenchOwnedRevision::Head),
            contexts: Vec::new(),
        };
        let owned = WorkbenchOwnedBinding {
            root: native_path_report(Path::new("/tmp/binding-owned-journal")),
            selection: WorkbenchOwnedRevision::Head,
            identity: OwnedRevisionIdentity {
                journal_schema: 2,
                source: source.clone(),
                revision,
                sequence: 0,
                state_sha256: "1".repeat(64),
                curation_state_sha256: Some("5".repeat(64)),
            },
        };
        let pin = vlajmaji_lexicon::OwnedContextPin {
            id: EntityId::new("binding-context").unwrap(),
            model_sha256: "2".repeat(64),
            source_ids: Vec::new(),
            release_ids: Vec::new(),
        };
        WorkbenchBinding::new(
            selection,
            None,
            Some(owned),
            vec![WorkbenchContextObservation {
                pin,
                packs: None,
                observed_model_sha256: None,
            }],
            Vec::new(),
            Vec::new(),
            "3".repeat(64),
        )
        .unwrap()
    }

    fn recalculate_binding_digest(binding: &mut WorkbenchBinding) {
        let payload = BindingPayload {
            version: binding.version,
            kind: &binding.kind,
            selection: &binding.selection,
            catalogue: &binding.catalogue,
            owned: &binding.owned,
            contexts: &binding.contexts,
            resolutions: &binding.resolutions,
            selected_release_ids: &binding.selected_release_ids,
            state_sha256: &binding.state_sha256,
        };
        binding.binding_sha256 = digest(&serde_json::to_vec(&payload).unwrap());
    }

    #[test]
    fn test_binding_validation_rejects_inconsistent_selection_and_context_pins() {
        let valid = test_binding_with_omitted_context_pin();
        let bytes = serde_json::to_vec(&valid).unwrap();
        assert_eq!(WorkbenchBinding::from_json_bytes(&bytes).unwrap(), valid);

        let mut empty_schema_one = valid.clone();
        empty_schema_one.contexts.clear();
        empty_schema_one.selection.contexts.clear();
        let owned = empty_schema_one.owned.as_mut().unwrap();
        owned.identity.journal_schema = 1;
        owned.identity.curation_state_sha256 = None;
        recalculate_binding_digest(&mut empty_schema_one);
        assert_eq!(
            WorkbenchBinding::from_json_bytes(&serde_json::to_vec(&empty_schema_one).unwrap())
                .unwrap(),
            empty_schema_one
        );

        let mut empty_schema_two = valid.clone();
        empty_schema_two.contexts.clear();
        empty_schema_two.selection.contexts.clear();
        empty_schema_two.resolutions.clear();
        recalculate_binding_digest(&mut empty_schema_two);
        assert_eq!(
            WorkbenchBinding::from_json_bytes(&serde_json::to_vec(&empty_schema_two).unwrap())
                .unwrap(),
            empty_schema_two
        );

        let mut schema_one_with_context = empty_schema_one.clone();
        schema_one_with_context
            .contexts
            .push(valid.contexts[0].clone());
        recalculate_binding_digest(&mut schema_one_with_context);
        assert!(matches!(
            WorkbenchBinding::from_json_bytes(
                &serde_json::to_vec(&schema_one_with_context).unwrap()
            ),
            Err(WorkbenchError::InvalidBinding { detail })
                if detail.contains("schema-one owned identity cannot retain curation contexts or resolutions")
        ));

        let mut schema_one_with_resolution = empty_schema_one.clone();
        schema_one_with_resolution
            .resolutions
            .push(OwnedCurationResolution {
                record: EntityId::new("binding-resolution-r1").unwrap(),
                status: OwnedCurationStatus::Inactive,
                reason: None,
                member: None,
                projected: false,
            });
        recalculate_binding_digest(&mut schema_one_with_resolution);
        assert!(matches!(
            WorkbenchBinding::from_json_bytes(
                &serde_json::to_vec(&schema_one_with_resolution).unwrap()
            ),
            Err(WorkbenchError::InvalidBinding { detail })
                if detail.contains("schema-one owned identity cannot retain curation contexts or resolutions")
        ));

        let mut catalogue_without_selected_packs = valid.clone();
        catalogue_without_selected_packs.catalogue = Some(WorkbenchCatalogueBinding {
            root: native_path_report(Path::new("/tmp/binding-catalogue")),
            local_revision: 0,
            generation_id: None,
            content_id: None,
            recorded_state_sha256: None,
            members: Vec::new(),
        });
        recalculate_binding_digest(&mut catalogue_without_selected_packs);
        assert!(matches!(
            WorkbenchBinding::from_json_bytes(
                &serde_json::to_vec(&catalogue_without_selected_packs).unwrap()
            ),
            Err(WorkbenchError::InvalidBinding { detail })
                if detail.contains("catalogue binding requires a nonempty pack selection")
        ));

        let mut selected_without_catalogue = valid.clone();
        selected_without_catalogue.selection.packs = vec![PackId::Llg];
        selected_without_catalogue.selection.disabled = vec![PackId::Llg];
        recalculate_binding_digest(&mut selected_without_catalogue);
        assert!(matches!(
            WorkbenchBinding::from_json_bytes(
                &serde_json::to_vec(&selected_without_catalogue).unwrap()
            ),
            Err(WorkbenchError::InvalidBinding { detail })
                if detail.contains("selected packs require a catalogue binding")
        ));

        let mut partial_catalogue_generation = valid.clone();
        partial_catalogue_generation.selection.packs = vec![PackId::Llg];
        partial_catalogue_generation.selection.disabled = vec![PackId::Llg];
        partial_catalogue_generation.catalogue = Some(WorkbenchCatalogueBinding {
            root: native_path_report(Path::new("/tmp/binding-catalogue")),
            local_revision: 0,
            generation_id: Some("generation-x".into()),
            content_id: None,
            recorded_state_sha256: None,
            members: Vec::new(),
        });
        recalculate_binding_digest(&mut partial_catalogue_generation);
        assert!(matches!(
            WorkbenchBinding::from_json_bytes(
                &serde_json::to_vec(&partial_catalogue_generation).unwrap()
            ),
            Err(WorkbenchError::InvalidBinding { detail })
                if detail.contains("catalogue generation identity fields must be present together")
        ));

        let mut enabled_without_generation = valid.clone();
        enabled_without_generation.selection.packs = vec![PackId::Llg];
        enabled_without_generation.catalogue = Some(WorkbenchCatalogueBinding {
            root: native_path_report(Path::new("/tmp/binding-catalogue")),
            local_revision: 0,
            generation_id: None,
            content_id: None,
            recorded_state_sha256: None,
            members: vec![WorkbenchCatalogueMember {
                pack_id: "llg".into(),
                release_sha256: "4".repeat(64),
                payload_kind: PackPayloadKind::Full,
            }],
        });
        recalculate_binding_digest(&mut enabled_without_generation);
        assert!(matches!(
            WorkbenchBinding::from_json_bytes(
                &serde_json::to_vec(&enabled_without_generation).unwrap()
            ),
            Err(WorkbenchError::InvalidBinding { detail })
                if detail.contains("enabled catalogue members require a captured generation")
        ));

        let mut malformed_catalogue_digest = valid.clone();
        malformed_catalogue_digest.selection.packs = vec![PackId::Llg];
        malformed_catalogue_digest.selection.disabled = vec![PackId::Llg];
        malformed_catalogue_digest.catalogue = Some(WorkbenchCatalogueBinding {
            root: native_path_report(Path::new("/tmp/binding-catalogue")),
            local_revision: 0,
            generation_id: Some("generation-x".into()),
            content_id: Some("content-x".into()),
            recorded_state_sha256: Some("not-a-digest".into()),
            members: Vec::new(),
        });
        recalculate_binding_digest(&mut malformed_catalogue_digest);
        assert!(matches!(
            WorkbenchBinding::from_json_bytes(
                &serde_json::to_vec(&malformed_catalogue_digest).unwrap()
            ),
            Err(WorkbenchError::InvalidBinding { detail })
                if detail.contains("recorded_state_sha256")
        ));

        let mut mismatched_historical_identity = valid.clone();
        let selected = EntityId::new("binding-owned-r0").unwrap();
        mismatched_historical_identity.selection.owned =
            Some(WorkbenchOwnedRevision::Revision(selected.clone()));
        let owned = mismatched_historical_identity.owned.as_mut().unwrap();
        owned.selection = WorkbenchOwnedRevision::Revision(selected);
        owned.identity.revision = EntityId::new("binding-owned-r9").unwrap();
        recalculate_binding_digest(&mut mismatched_historical_identity);
        assert!(matches!(
            WorkbenchBinding::from_json_bytes(
                &serde_json::to_vec(&mismatched_historical_identity).unwrap()
            ),
            Err(WorkbenchError::InvalidBinding { detail })
                if detail.contains("owned identity differs from the selected historical revision")
        ));

        let mut malformed_pin = valid.clone();
        malformed_pin.contexts[0].pin.model_sha256 = "not-a-digest".into();
        recalculate_binding_digest(&mut malformed_pin);
        assert!(matches!(
            WorkbenchBinding::from_json_bytes(&serde_json::to_vec(&malformed_pin).unwrap()),
            Err(WorkbenchError::InvalidBinding { .. })
        ));

        let mut unsorted_sources = valid.clone();
        unsorted_sources.contexts[0].pin.source_ids = vec![
            SourceId::new("z-source").unwrap(),
            SourceId::new("a-source").unwrap(),
        ];
        recalculate_binding_digest(&mut unsorted_sources);
        assert!(matches!(
            WorkbenchBinding::from_json_bytes(&serde_json::to_vec(&unsorted_sources).unwrap()),
            Err(WorkbenchError::InvalidBinding { detail })
                if detail.contains("sorted, unique")
        ));

        let mut unsorted_releases = valid.clone();
        unsorted_releases.contexts[0].pin.release_ids = vec![
            EntityId::new("z-release").unwrap(),
            EntityId::new("a-release").unwrap(),
        ];
        recalculate_binding_digest(&mut unsorted_releases);
        assert!(matches!(
            WorkbenchBinding::from_json_bytes(&serde_json::to_vec(&unsorted_releases).unwrap()),
            Err(WorkbenchError::InvalidBinding { detail })
                if detail.contains("sorted, unique")
        ));

        let mut pin_contains_owned_source = valid;
        pin_contains_owned_source.contexts[0].pin.source_ids =
            vec![SourceId::new("binding-owned-source").unwrap()];
        recalculate_binding_digest(&mut pin_contains_owned_source);
        assert!(matches!(
            WorkbenchBinding::from_json_bytes(
                &serde_json::to_vec(&pin_contains_owned_source).unwrap()
            ),
            Err(WorkbenchError::InvalidBinding { detail })
                if detail.contains("exclude the owned source")
        ));
    }

    #[test]
    fn test_build_new_catalogue_mutation_at_each_guard_stage_returns_stale_without_success() {
        for stage in [
            BuildGuardStage::BeforeReservation,
            BuildGuardStage::BeforeBinding,
            BuildGuardStage::AfterBinding,
        ] {
            let parent = fresh_root("stage");
            fs::create_dir(&parent).unwrap();
            let catalogue = parent.join("catalogue");
            install(&catalogue, PackId::Llg);
            let destination = parent.join("view");
            let selection = WorkbenchSelection {
                version: 1,
                packs: vec![PackId::Llg],
                disabled: Vec::new(),
                owned: None,
                contexts: Vec::new(),
            };
            let mut fired = false;
            let result = WorkbenchView::build_new_with_hook(
                &destination,
                WorkbenchBuildRequest {
                    catalogue_root: Some(catalogue.clone()),
                    owned_root: None,
                    selection,
                },
                |current| {
                    if current == stage {
                        register(&catalogue, PackId::LaLojban);
                        fired = true;
                    }
                    Ok(())
                },
            );
            assert!(
                fired,
                "the actual successful catalogue writer must run at {stage:?}"
            );
            assert!(matches!(
                result,
                Err(WorkbenchError::StaleDependencies {
                    dependency: WorkbenchDependency::Catalogue,
                    ..
                })
            ));
            assert_eq!(
                destination.exists(),
                stage != BuildGuardStage::BeforeReservation
            );
            if stage == BuildGuardStage::AfterBinding {
                assert!(destination.join("binding.json").is_file());
                assert!(matches!(
                    WorkbenchView::open(&destination),
                    Err(WorkbenchError::StaleDependencies {
                        dependency: WorkbenchDependency::Catalogue,
                        ..
                    })
                ));
            }
            fs::remove_dir_all(parent).unwrap();
        }
    }

    #[test]
    fn test_build_new_owned_writer_after_reader_drop_is_checked_at_each_guard_stage() {
        for stage in [
            BuildGuardStage::BeforeReservation,
            BuildGuardStage::BeforeBinding,
            BuildGuardStage::AfterBinding,
        ] {
            let parent = fresh_root("owned-stage");
            fs::create_dir(&parent).unwrap();
            let journal_root = parent.join("journal");
            let genesis = owned_genesis();
            let (writer, created) = crate::OwnedJournal::create(&journal_root, &genesis).unwrap();
            drop(writer);
            let destination = parent.join("view");
            let mut fired = false;
            let result = WorkbenchView::build_new_with_hook(
                &destination,
                WorkbenchBuildRequest {
                    catalogue_root: None,
                    owned_root: Some(journal_root.clone()),
                    selection: WorkbenchSelection {
                        version: 1,
                        packs: Vec::new(),
                        disabled: Vec::new(),
                        owned: Some(WorkbenchOwnedRevision::Head),
                        contexts: Vec::new(),
                    },
                },
                |current| {
                    if current == stage {
                        let mut writer = crate::OwnedJournal::open(&journal_root).unwrap();
                        writer
                            .commit(&created.head.revision.id, &owned_revision_one(&genesis))
                            .unwrap();
                        fired = true;
                    }
                    Ok(())
                },
            );
            assert!(
                fired,
                "the writer must commit after the capture reader is dropped at {stage:?}"
            );
            assert!(matches!(
                result,
                Err(WorkbenchError::StaleDependencies {
                    dependency: WorkbenchDependency::Owned,
                    ..
                })
            ));
            assert_eq!(
                destination.exists(),
                stage != BuildGuardStage::BeforeReservation
            );
            if stage == BuildGuardStage::AfterBinding {
                assert!(destination.join("binding.json").is_file());
                assert!(matches!(
                    WorkbenchView::open(&destination),
                    Err(WorkbenchError::StaleDependencies {
                        dependency: WorkbenchDependency::Owned,
                        ..
                    })
                ));
            }
            fs::remove_dir_all(parent).unwrap();
        }
    }
}
