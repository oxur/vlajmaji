//! Explicit versioned configuration for named workbench profiles.
//!
//! This module reads only the caller-supplied path; it never searches HOME,
//! XDG, or an ambient application default.

use std::{
    error::Error,
    fmt,
    fs::{self, File, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use sha2::{Digest, Sha256};

use crate::workbench::decode_native_path;
use crate::{
    CollectionCatalogue, CollectionError, CollectionMetadataGeneration,
    CollectionMetadataRegistration, CollectionMetadataRoster, DictionarySnapshotManifest,
    NativePathReport, OwnedJournalReader, WorkbenchBuildReport, WorkbenchBuildRequest,
    WorkbenchError, WorkbenchSelection, WorkbenchView, native_path_report,
};
use vlajmaji_lexicon::PackId;

const CONFIG_VERSION: u32 = 1;
const CONFIG_KIND: &str = "vlajmaji-workbench-profiles";
const IDENTITY_KIND: &str = "vlajmaji-configured-workbench";

/// One configured profile with absolute, lossless native roots.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkbenchProfile {
    /// Validated profile name.
    pub name: String,
    /// Catalogue root; required when catalogue packs are selected.
    #[serde(deserialize_with = "deserialize_required_native_path")]
    pub catalogue_root: Option<NativePathReport>,
    /// Owned journal root; required when an owned revision is selected.
    #[serde(deserialize_with = "deserialize_required_native_path")]
    pub owned_root: Option<NativePathReport>,
    /// Complete S01 selection policy.
    pub selection: WorkbenchSelection,
    /// Active immutable prepared-history sequence.
    #[serde(deserialize_with = "deserialize_required_option")]
    pub active_generation: Option<u64>,
}

/// One immutable prepared workbench retained in profile history.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkbenchPrepared {
    /// Global zero-based publication sequence.
    pub sequence: u64,
    /// Profile name captured by the publication.
    pub profile: String,
    /// Absolute native workbench root.
    #[serde(deserialize_with = "deserialize_native_path")]
    pub root: NativePathReport,
    /// Exact S01 binding digest.
    pub binding_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ConfigWire {
    version: u32,
    kind: String,
    revision: u64,
    #[serde(deserialize_with = "deserialize_required_option")]
    default_profile: Option<String>,
    profiles: Vec<WorkbenchProfile>,
    prepared: Vec<WorkbenchPrepared>,
}

/// Request to configure one profile without reading its source roots.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkbenchProfileConfigureRequest {
    /// Profile name.
    pub name: String,
    /// Catalogue root when selected packs are nonempty.
    pub catalogue_root: Option<PathBuf>,
    /// Owned root when an owned revision is selected.
    pub owned_root: Option<PathBuf>,
    /// Complete S01 selection.
    pub selection: WorkbenchSelection,
    /// Make this the declared default.
    pub make_default: bool,
}

/// Captured configuration metadata and retained publication history.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkbenchProfilesReport {
    /// Report schema version.
    pub version: u32,
    /// Configuration kind.
    pub kind: String,
    /// Exact native config path.
    pub config: NativePathReport,
    /// Captured config revision.
    pub revision: u64,
    /// Exact config bytes digest.
    pub sha256: String,
    /// Declared default profile.
    pub default_profile: Option<String>,
    /// Profiles in validated name order.
    pub profiles: Vec<WorkbenchProfile>,
    /// All prepared roots in publication sequence order.
    pub prepared: Vec<WorkbenchPrepared>,
}

/// Retained publication history, optionally filtered by profile name.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkbenchProfileHistoryReport {
    /// Report schema version.
    pub version: u32,
    /// Exact native config path.
    pub config: NativePathReport,
    /// Captured config revision.
    pub revision: u64,
    /// Exact config bytes digest.
    pub sha256: String,
    /// Requested profile filter; null selects all rows.
    pub profile: Option<String>,
    /// Matching rows in original sequence order.
    pub prepared: Vec<WorkbenchPrepared>,
}

/// Result of one profile configuration/default/removal/publication write.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkbenchProfileWriteReport {
    /// Report schema version.
    pub version: u32,
    /// Operation name.
    pub operation: String,
    /// Exact native config path.
    pub config: NativePathReport,
    /// Whether new bytes were committed.
    pub changed: bool,
    /// Prior revision or null when creating the file.
    pub prior_revision: Option<u64>,
    /// Resulting revision.
    pub revision: u64,
    /// Resulting exact bytes digest.
    pub sha256: String,
    /// Affected profile, null only when clearing default.
    pub profile: Option<String>,
    /// Resulting declared default.
    pub default_profile: Option<String>,
    /// Active prepared row after the operation.
    pub active: Option<WorkbenchPrepared>,
}

/// Explicit metadata-only status for one configured profile.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkbenchProfileStatusReport {
    /// Report schema version.
    pub version: u32,
    /// Scope of the metadata observation.
    pub scope: String,
    /// Exact native config path.
    pub config: NativePathReport,
    /// Captured config revision.
    pub config_revision: u64,
    /// Captured exact config bytes digest.
    pub config_sha256: String,
    /// Requested profile, null when default mode was requested.
    pub requested_profile: Option<String>,
    /// Selected validated profile.
    pub profile: WorkbenchProfile,
    /// Minimum active-root observation, if prepared.
    pub active: Option<WorkbenchPreparedStatusReport>,
    /// Catalogue metadata, if selected.
    pub catalogue: Option<WorkbenchConfiguredCatalogueStatus>,
    /// Compact owned identity, if selected.
    pub owned: Option<crate::WorkbenchOwnedBinding>,
}

/// Minimum observed state for an immutable prepared root.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkbenchPreparedStatusReport {
    /// The retained publication row.
    pub prepared: WorkbenchPrepared,
    /// Current/stale/unavailable/invalid classification.
    pub state: WorkbenchPreparedState,
    /// Actual observed error, absent only for current.
    pub error: Option<WorkbenchPreparedStatusError>,
}

/// Minimum prepared-root classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkbenchPreparedState {
    /// The retained root matches current configuration and source state.
    Current,
    /// The retained root is valid but no longer matches the current state.
    Stale,
    /// Required local state is not currently available.
    Unavailable,
    /// The retained root or its metadata is invalid.
    Invalid,
}

/// Actual category and message for a non-current prepared root.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkbenchPreparedStatusError {
    /// Stable underlying error category.
    pub code: String,
    /// Original typed error display.
    pub message: String,
}

/// Catalogue metadata associated with the configured selection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkbenchConfiguredCatalogueStatus {
    /// Configured catalogue root.
    pub root: NativePathReport,
    /// Catalogue-local revision.
    pub local_revision: u64,
    /// Installed generation, when present.
    pub generation: Option<CollectionMetadataGeneration>,
    /// Selected packs followed by registered unselected packs.
    pub packs: Vec<WorkbenchConfiguredPackStatus>,
}

/// One registered or selected pack's logical catalogue state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkbenchConfiguredPackStatus {
    /// Stable pack identity.
    pub pack_id: String,
    /// Whether selected by the profile.
    pub selected: bool,
    /// Whether deliberately disabled.
    pub disabled: bool,
    /// Logical installation state; null only for selected/unregistered IDs.
    pub installation_state: Option<crate::CollectionInstallationState>,
    /// Release selected by the active generation.
    pub installed_release_sha256: Option<String>,
    /// Registered metadata rows.
    pub registrations: Vec<CollectionMetadataRegistration>,
}

/// Result of one refresh and its durable config publication.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkbenchProfileRefreshReport {
    /// Report schema version.
    pub version: u32,
    /// Operation name.
    pub operation: String,
    /// Config publication details.
    pub publication: WorkbenchProfileWriteReport,
    /// Result of the single S01 build.
    pub view: WorkbenchBuildReport,
}

/// Exact publication details retained for a failure after rename.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkbenchPublicationContext {
    /// Exact native config path.
    pub config: NativePathReport,
    /// Previous revision, if any.
    pub prior_revision: Option<u64>,
    /// Previous exact bytes digest, if any.
    pub prior_sha256: Option<String>,
    /// Proposed resulting revision.
    pub proposed_revision: u64,
    /// Proposed exact bytes digest.
    pub proposed_sha256: String,
    /// Best-effort visible revision.
    pub observed_revision: Option<u64>,
    /// Best-effort visible exact bytes digest.
    pub observed_sha256: Option<String>,
    /// Prepared row the proposed config activates.
    pub prepared: Option<WorkbenchPrepared>,
}

/// Typed config, validation, publication, or underlying S01-view failure.
#[derive(Debug)]
#[non_exhaustive]
pub enum WorkbenchProfileError {
    /// Native file IO failure.
    Io {
        /// Path associated with the failed operation.
        path: PathBuf,
        /// Underlying operating-system error.
        source: io::Error,
    },
    /// Strict JSON decode failure.
    Decode {
        /// Configuration file path.
        path: PathBuf,
        /// Decoder's diagnostic.
        detail: String,
    },
    /// Unsupported version or kind.
    UnsupportedVersion {
        /// Expected configuration kind.
        kind: &'static str,
        /// Observed schema version.
        found: u32,
    },
    /// Unsupported native encoding.
    UnsupportedNativeEncoding {
        /// Encoding tag present in the config.
        encoding: String,
    },
    /// Invalid path, name, or cross-reference policy.
    InvalidPolicy {
        /// Configuration file path.
        path: PathBuf,
        /// Policy validation detail.
        detail: String,
    },
    /// Explicit config file is missing.
    MissingFile(PathBuf),
    /// Config has no default.
    NoDefault,
    /// Named profile does not exist.
    UnknownProfile {
        /// Requested profile name.
        name: String,
    },
    /// Selected profile has no active preparation.
    Unprepared {
        /// Profile without an active prepared generation.
        name: String,
    },
    /// Latest config revision differs from caller expectation.
    Conflict {
        /// Caller-supplied expected revision.
        expected: Option<u64>,
        /// Revision read under the writer lock.
        observed: Option<u64>,
    },
    /// Exclusive sibling lock already exists.
    Busy {
        /// Path of the occupied lock file.
        lock: PathBuf,
    },
    /// Revision or sequence exceeds supported SQLite range.
    RevisionOverflow(u64),
    /// Captured config bytes changed or disappeared.
    ConfigChanged {
        /// Configuration path whose bytes changed.
        config: PathBuf,
        /// Digest captured before observation.
        expected_sha256: Option<String>,
        /// Digest observed during the operation.
        observed_sha256: Option<String>,
    },
    /// Prepared binding differs from configured authority.
    BindingMismatch {
        /// Expected binding or selection digest.
        expected: String,
        /// Observed binding or selection digest.
        observed: String,
    },
    /// Existing S01 view/source failure.
    View(Box<WorkbenchError>),
    /// Rename was visible but a post-publication step failed.
    Indeterminate {
        /// Exact publication state visible after rename.
        publication: Box<WorkbenchPublicationContext>,
        /// The initiating or subsequent failure.
        source: Box<WorkbenchProfileError>,
    },
}

impl fmt::Display for WorkbenchProfileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, source } => {
                write!(f, "workbench profile IO at {}: {source}", path.display())
            }
            Self::Decode { path, detail } => write!(
                f,
                "invalid workbench profile JSON at {}: {detail}",
                path.display()
            ),
            Self::UnsupportedVersion { kind, found } => {
                write!(f, "unsupported {kind} version {found}")
            }
            Self::UnsupportedNativeEncoding { encoding } => {
                write!(f, "unsupported native path encoding {encoding}")
            }
            Self::InvalidPolicy { path, detail } => write!(
                f,
                "invalid workbench profile policy at {}: {detail}",
                path.display()
            ),
            Self::MissingFile(path) => {
                write!(f, "workbench profile file is missing: {}", path.display())
            }
            Self::NoDefault => f.write_str("workbench profile configuration has no default"),
            Self::UnknownProfile { name } => write!(f, "unknown workbench profile {name:?}"),
            Self::Unprepared { name } => {
                write!(f, "workbench profile {name:?} has no active prepared root")
            }
            Self::Conflict { expected, observed } => write!(
                f,
                "workbench profile revision conflict: expected {expected:?}, observed {observed:?}"
            ),
            Self::Busy { lock } => {
                write!(f, "workbench profile writer is busy: {}", lock.display())
            }
            Self::RevisionOverflow(value) => {
                write!(f, "workbench revision {value} cannot be incremented")
            }
            Self::ConfigChanged {
                config,
                expected_sha256,
                observed_sha256,
            } => write!(
                f,
                "workbench config changed at {}: expected {expected_sha256:?}, observed {observed_sha256:?}",
                config.display()
            ),
            Self::BindingMismatch { expected, observed } => write!(
                f,
                "configured binding mismatch: expected {expected}, observed {observed}"
            ),
            Self::View(source) => write!(f, "workbench view: {source}"),
            Self::Indeterminate {
                publication,
                source,
            } => write!(
                f,
                "workbench publication is indeterminate ({publication:?}): {source}"
            ),
        }
    }
}

impl Error for WorkbenchProfileError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::View(source) => Some(source.as_ref()),
            Self::Indeterminate { source, .. } => Some(source.as_ref()),
            _ => None,
        }
    }
}

impl From<WorkbenchError> for WorkbenchProfileError {
    fn from(value: WorkbenchError) -> Self {
        Self::View(Box::new(value))
    }
}

impl From<CollectionError> for WorkbenchProfileError {
    fn from(value: CollectionError) -> Self {
        Self::View(Box::new(WorkbenchError::from(value)))
    }
}

impl From<crate::OwnedJournalError> for WorkbenchProfileError {
    fn from(value: crate::OwnedJournalError) -> Self {
        Self::View(Box::new(WorkbenchError::from(value)))
    }
}

/// Exact typed identity of one configured capture.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkbenchProfileIdentity {
    /// Identity schema version.
    pub version: u32,
    /// Stable identity kind.
    pub kind: String,
    /// Exact native configuration path.
    #[serde(deserialize_with = "deserialize_native_path")]
    pub config: NativePathReport,
    /// Captured configuration revision.
    pub config_revision: u64,
    /// Exact configuration byte digest.
    pub config_sha256: String,
    /// Explicit requested profile, null when default-selection mode was requested.
    #[serde(deserialize_with = "deserialize_required_option")]
    pub requested_profile: Option<String>,
    /// Resolved profile name.
    pub profile: String,
    /// Active prepared sequence.
    pub prepared_sequence: u64,
    /// Direct S01 view binding digest.
    pub view_binding_sha256: String,
    /// Effective configured identity digest.
    pub effective_binding_sha256: String,
}

#[derive(Serialize)]
struct IdentityPayload<'a> {
    version: u32,
    kind: &'static str,
    config: &'a NativePathReport,
    config_revision: u64,
    config_sha256: &'a str,
    requested_profile: &'a Option<String>,
    profile: &'a str,
    prepared_sequence: u64,
    view_binding_sha256: &'a str,
}

/// Exact byte capture of one explicit profile config.
pub struct WorkbenchProfiles {
    path: PathBuf,
    bytes: Vec<u8>,
    sha256: String,
    wire: ConfigWire,
}

impl WorkbenchProfiles {
    /// Reads exactly the supplied config path.
    ///
    /// # Errors
    /// Returns missing-file, IO, strict decode, native-path, or policy errors.
    pub fn read(config: impl AsRef<Path>) -> Result<Self, WorkbenchProfileError> {
        let path = absolute_path(config.as_ref())?;
        match fs::symlink_metadata(&path) {
            Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_file() => {
                return Err(invalid(
                    &path,
                    "configuration source must be a regular non-symlink file",
                ));
            }
            Ok(_) => {}
            Err(source) if source.kind() == io::ErrorKind::NotFound => {
                return Err(WorkbenchProfileError::MissingFile(path));
            }
            Err(source) => return Err(WorkbenchProfileError::Io { path, source }),
        }
        let bytes = fs::read(&path).map_err(|source| {
            if source.kind() == io::ErrorKind::NotFound {
                WorkbenchProfileError::MissingFile(path.clone())
            } else {
                WorkbenchProfileError::Io {
                    path: path.clone(),
                    source,
                }
            }
        })?;
        Self::from_bytes(path, bytes)
    }

    fn from_bytes(path: PathBuf, bytes: Vec<u8>) -> Result<Self, WorkbenchProfileError> {
        let wire: ConfigWire =
            serde_json::from_slice(&bytes).map_err(|error| WorkbenchProfileError::Decode {
                path: path.clone(),
                detail: error.to_string(),
            })?;
        validate_wire(&path, &wire)?;
        Ok(Self {
            path,
            sha256: digest(&bytes),
            bytes,
            wire,
        })
    }

    /// Selects a named or default profile as metadata; active-null is valid here.
    ///
    /// # Errors
    /// Returns NoDefault or UnknownProfile.
    pub fn profile(&self, name: Option<&str>) -> Result<&WorkbenchProfile, WorkbenchProfileError> {
        let name = match name {
            Some(name) => name,
            None => self
                .wire
                .default_profile
                .as_deref()
                .ok_or(WorkbenchProfileError::NoDefault)?,
        };
        self.wire
            .profiles
            .iter()
            .find(|profile| profile.name == name)
            .ok_or_else(|| WorkbenchProfileError::UnknownProfile {
                name: name.to_owned(),
            })
    }

    /// Returns captured metadata and complete retained publication history.
    #[must_use]
    pub fn report(&self) -> WorkbenchProfilesReport {
        WorkbenchProfilesReport {
            version: 1,
            kind: CONFIG_KIND.into(),
            config: native_path_report(&self.path),
            revision: self.wire.revision,
            sha256: self.sha256.clone(),
            default_profile: self.wire.default_profile.clone(),
            profiles: self.wire.profiles.clone(),
            prepared: self.wire.prepared.clone(),
        }
    }

    /// Returns all history, or rows for an exact name including removed profiles.
    ///
    /// # Errors
    /// Returns a policy error for an invalid filter name.
    pub fn history(
        &self,
        profile: Option<&str>,
    ) -> Result<WorkbenchProfileHistoryReport, WorkbenchProfileError> {
        if let Some(name) = profile {
            validate_name(name, &self.path)?;
        }
        Ok(WorkbenchProfileHistoryReport {
            version: 1,
            config: native_path_report(&self.path),
            revision: self.wire.revision,
            sha256: self.sha256.clone(),
            profile: profile.map(str::to_owned),
            prepared: self
                .wire
                .prepared
                .iter()
                .filter(|row| profile.is_none_or(|name| name == row.profile))
                .cloned()
                .collect(),
        })
    }

    /// Explicitly observes current source metadata and the active prepared root.
    ///
    /// # Errors
    /// Returns a config, catalogue, or owned-journal error from the observation.
    pub fn status(
        &self,
        name: Option<&str>,
    ) -> Result<WorkbenchProfileStatusReport, WorkbenchProfileError> {
        self.verify_unchanged()?;
        let observation: Result<WorkbenchProfileStatusReport, WorkbenchProfileError> = (|| {
            let profile = self.profile(name)?.clone();
            let active = profile
                .active_generation
                .and_then(|sequence| self.wire.prepared.get(sequence as usize))
                .map(|prepared| observe_prepared(&profile, prepared.clone()));
            let catalogue = match (&profile.catalogue_root, profile.selection.packs.is_empty()) {
                (Some(root), false) => {
                    let root_path = decode_root(root)?;
                    let catalogue = CollectionCatalogue::open(&root_path)?;
                    let roster = catalogue.metadata_roster()?;
                    drop(catalogue);
                    Some(configured_catalogue_status(root.clone(), &profile, roster))
                }
                _ => None,
            };
            let owned = match (&profile.owned_root, &profile.selection.owned) {
                (Some(root), Some(selection)) => {
                    let root_path = decode_root(root)?;
                    let reader = OwnedJournalReader::open(&root_path)?;
                    let identity = match selection {
                        crate::WorkbenchOwnedRevision::Head => reader.head_identity()?,
                        crate::WorkbenchOwnedRevision::Revision(revision) => {
                            reader.revision_identity(revision)?
                        }
                    };
                    drop(reader);
                    Some(crate::WorkbenchOwnedBinding {
                        root: root.clone(),
                        selection: selection.clone(),
                        identity,
                    })
                }
                _ => None,
            };
            Ok(WorkbenchProfileStatusReport {
                version: 1,
                scope: "configured-metadata-and-active-minimum-v1".into(),
                config: native_path_report(&self.path),
                config_revision: self.wire.revision,
                config_sha256: self.sha256.clone(),
                requested_profile: name.map(str::to_owned),
                profile,
                active,
                catalogue,
                owned,
            })
        })();
        self.verify_unchanged()?;
        observation
    }

    /// Verifies exact captured bytes at the same path.
    ///
    /// # Errors
    /// Returns ConfigChanged or the underlying IO error.
    pub fn verify_unchanged(&self) -> Result<(), WorkbenchProfileError> {
        verify_bytes(&self.path, &self.bytes, &self.sha256)
    }

    /// Returns the absolute config path captured at read time.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Creates or replaces one profile using a latest-state revision check.
    ///
    /// # Errors
    /// Returns strict policy, conflict, busy, IO, overflow, or indeterminate-publication errors.
    pub fn configure(
        config: impl AsRef<Path>,
        request: WorkbenchProfileConfigureRequest,
        expected_revision: Option<u64>,
    ) -> Result<WorkbenchProfileWriteReport, WorkbenchProfileError> {
        let (path, cwd) = resolve_inputs(
            config.as_ref(),
            [
                request.catalogue_root.as_deref(),
                request.owned_root.as_deref(),
            ],
            None,
        )?;
        validate_name(&request.name, &path)?;
        request
            .selection
            .validate()
            .map_err(|error| invalid(&path, error.to_string()))?;
        let catalogue_root = resolve_optional(request.catalogue_root.as_deref(), cwd.as_deref());
        let owned_root = resolve_optional(request.owned_root.as_deref(), cwd.as_deref());
        validate_roots(
            &path,
            &request.selection,
            catalogue_root.as_deref(),
            owned_root.as_deref(),
        )?;
        let profile = WorkbenchProfile {
            name: request.name.clone(),
            catalogue_root: catalogue_root.as_deref().map(native_path_report),
            owned_root: owned_root.as_deref().map(native_path_report),
            selection: request.selection,
            active_generation: None,
        };
        if let Some(root) = profile.catalogue_root.as_ref() {
            decode_root(root)?;
        }
        if let Some(root) = profile.owned_root.as_ref() {
            decode_root(root)?;
        }
        let (prior, mut wire, _guard) = begin_write(&path, expected_revision)?;
        let existing_index = wire
            .profiles
            .iter()
            .position(|item| item.name == profile.name);
        let prior_profile = existing_index.map(|index| wire.profiles[index].clone());
        let policy_changed = prior_profile.as_ref().is_none_or(|old| {
            old.catalogue_root != profile.catalogue_root
                || old.owned_root != profile.owned_root
                || old.selection != profile.selection
        });
        let mut new_profile = profile;
        if !policy_changed {
            new_profile.active_generation = prior_profile.and_then(|old| old.active_generation);
        }
        match existing_index {
            Some(index) => wire.profiles[index] = new_profile,
            None => wire.profiles.push(new_profile),
        }
        wire.profiles.sort_by(|a, b| a.name.cmp(&b.name));
        if request.make_default {
            wire.default_profile = Some(request.name.clone());
        }
        let profile_changed = prior.as_ref().is_none_or(|old| {
            old.wire.profiles != wire.profiles || old.wire.default_profile != wire.default_profile
        });
        if !profile_changed {
            return unchanged_report(&path, prior.as_ref(), "configure", Some(&request.name));
        }
        publish_report(
            &path,
            prior.as_ref(),
            &mut wire,
            "configure",
            Some(&request.name),
            None,
            None,
        )
    }

    /// Sets or clears the default profile in the latest configuration.
    ///
    /// # Errors
    /// Returns missing/default/name/conflict/busy/IO/overflow/publication errors.
    pub fn set_default(
        config: impl AsRef<Path>,
        profile: Option<&str>,
        expected_revision: u64,
    ) -> Result<WorkbenchProfileWriteReport, WorkbenchProfileError> {
        let (path, _) = resolve_inputs(config.as_ref(), [None, None], None)?;
        if let Some(name) = profile {
            validate_name(name, &path)?;
        }
        let (prior, mut wire, _guard) = begin_write(&path, Some(expected_revision))?;
        if let Some(name) = profile
            && !wire.profiles.iter().any(|item| item.name == name)
        {
            return Err(WorkbenchProfileError::UnknownProfile { name: name.into() });
        }
        if wire.default_profile.as_deref() == profile {
            return unchanged_report(&path, prior.as_ref(), "set_default", profile);
        }
        wire.default_profile = profile.map(str::to_owned);
        publish_report(
            &path,
            prior.as_ref(),
            &mut wire,
            "set_default",
            profile,
            None,
            None,
        )
    }

    /// Removes one configured profile while retaining every immutable history row.
    ///
    /// # Errors
    /// Returns missing-profile/conflict/busy/IO/overflow/publication errors.
    pub fn remove(
        config: impl AsRef<Path>,
        name: &str,
        expected_revision: u64,
    ) -> Result<WorkbenchProfileWriteReport, WorkbenchProfileError> {
        let (path, _) = resolve_inputs(config.as_ref(), [None, None], None)?;
        validate_name(name, &path)?;
        let (prior, mut wire, _guard) = begin_write(&path, Some(expected_revision))?;
        let Some(index) = wire.profiles.iter().position(|item| item.name == name) else {
            return Err(WorkbenchProfileError::UnknownProfile { name: name.into() });
        };
        wire.profiles.remove(index);
        if wire.default_profile.as_deref() == Some(name) {
            wire.default_profile = None;
        }
        publish_report(
            &path,
            prior.as_ref(),
            &mut wire,
            "remove",
            Some(name),
            None,
            None,
        )
    }

    /// Refreshes a profile exactly once into a new root, then publishes its active history row.
    ///
    /// # Errors
    /// Returns profile, build, conflict, IO, overflow or indeterminate-publication errors.
    pub fn refresh(
        config: impl AsRef<Path>,
        profile: Option<&str>,
        destination: impl AsRef<Path>,
        expected_revision: u64,
    ) -> Result<WorkbenchProfileRefreshReport, WorkbenchProfileError> {
        let (path, cwd) =
            resolve_inputs(config.as_ref(), [None, None], Some(destination.as_ref()))?;
        let destination = resolve_optional(Some(destination.as_ref()), cwd.as_deref())
            .expect("destination supplied");
        let (prior, mut wire, _guard) = begin_write(&path, Some(expected_revision))?;
        let name = match profile {
            Some(name) => name.to_owned(),
            None => wire
                .default_profile
                .clone()
                .ok_or(WorkbenchProfileError::NoDefault)?,
        };
        validate_name(&name, &path)?;
        let Some(index) = wire.profiles.iter().position(|item| item.name == name) else {
            return Err(WorkbenchProfileError::UnknownProfile { name });
        };
        let selected = wire.profiles[index].clone();
        let catalogue_root = selected
            .catalogue_root
            .as_ref()
            .map(decode_root)
            .transpose()?;
        let owned_root = selected.owned_root.as_ref().map(decode_root).transpose()?;
        let build = WorkbenchView::build_new(
            &destination,
            WorkbenchBuildRequest {
                catalogue_root,
                owned_root,
                selection: selected.selection.clone(),
            },
        )?;
        if build.binding.selection() != &selected.selection {
            return Err(WorkbenchProfileError::BindingMismatch {
                expected: digest(
                    &serde_json::to_vec(&selected.selection)
                        .map_err(|error| decode_error(&path, error))?,
                ),
                observed: digest(
                    &serde_json::to_vec(build.binding.selection())
                        .map_err(|error| decode_error(&path, error))?,
                ),
            });
        }
        let sequence = u64::try_from(wire.prepared.len())
            .map_err(|_| WorkbenchProfileError::RevisionOverflow(u64::MAX))?;
        if sequence > i64::MAX as u64 {
            return Err(WorkbenchProfileError::RevisionOverflow(sequence));
        }
        let prepared = WorkbenchPrepared {
            sequence,
            profile: name.clone(),
            root: build.root.clone(),
            binding_sha256: build.binding.binding_sha256().to_owned(),
        };
        wire.prepared.push(prepared.clone());
        wire.profiles[index].active_generation = Some(sequence);
        let verify_built = || -> Result<(), WorkbenchProfileError> {
            let current = WorkbenchView::open(&destination)?;
            current.ensure_current()?;
            if current.binding() != &build.binding {
                return Err(WorkbenchProfileError::BindingMismatch {
                    expected: build.binding.binding_sha256().to_owned(),
                    observed: current.binding().binding_sha256().to_owned(),
                });
            }
            Ok(())
        };
        let publication = publish_report(
            &path,
            prior.as_ref(),
            &mut wire,
            "refresh",
            Some(&name),
            Some(prepared.clone()),
            Some(&verify_built),
        )?;
        let reopened = WorkbenchView::open(&destination).map_err(|error| {
            indeterminate_after_publication(
                &path,
                prior.as_ref(),
                &publication,
                Some(prepared.clone()),
                WorkbenchProfileError::from(error),
            )
        })?;
        if let Err(error) = reopened.ensure_current() {
            return Err(indeterminate_after_publication(
                &path,
                prior.as_ref(),
                &publication,
                Some(prepared.clone()),
                WorkbenchProfileError::from(error),
            ));
        }
        if reopened.binding() != &build.binding {
            return Err(indeterminate_after_publication(
                &path,
                prior.as_ref(),
                &publication,
                Some(prepared),
                invalid(&path, "published view differs from built view"),
            ));
        }
        Ok(WorkbenchProfileRefreshReport {
            version: 1,
            operation: "refresh".into(),
            publication,
            view: build,
        })
    }
}

struct WriterGuard {
    path: PathBuf,
    // Keep the opened inode alive until cleanup has compared the directory entry.
    // This prevents an unlinked lock/stage inode from being recycled underneath
    // the identity check while the guard is still active.
    _file: File,
    identity: FileIdentity,
    active: bool,
}

struct FileIdentity {
    #[cfg(unix)]
    dev: u64,
    #[cfg(unix)]
    ino: u64,
    #[cfg(not(unix))]
    len: u64,
}

impl WriterGuard {
    fn new(path: PathBuf, file: &File) -> io::Result<Self> {
        let metadata = file.metadata()?;
        let retained_file = file.try_clone()?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            Ok(Self {
                path,
                _file: retained_file,
                identity: FileIdentity {
                    dev: metadata.dev(),
                    ino: metadata.ino(),
                },
                active: true,
            })
        }
        #[cfg(not(unix))]
        {
            Ok(Self {
                path,
                _file: retained_file,
                identity: FileIdentity {
                    len: metadata.len(),
                },
                active: true,
            })
        }
    }
}

impl Drop for WriterGuard {
    fn drop(&mut self) {
        if !self.active {
            return;
        }
        let Ok(metadata) = fs::symlink_metadata(&self.path) else {
            return;
        };
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            if metadata.dev() == self.identity.dev && metadata.ino() == self.identity.ino {
                let _ = fs::remove_file(&self.path);
            }
        }
        #[cfg(not(unix))]
        if metadata.is_file() && metadata.len() == self.identity.len {
            let _ = fs::remove_file(&self.path);
        }
    }
}

fn begin_write(
    path: &Path,
    expected_revision: Option<u64>,
) -> Result<(Option<WorkbenchProfiles>, ConfigWire, WriterGuard), WorkbenchProfileError> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let parent_meta = fs::metadata(parent).map_err(|source| WorkbenchProfileError::Io {
        path: parent.to_path_buf(),
        source,
    })?;
    if !parent_meta.is_dir() {
        return Err(invalid(path, "configuration parent is not a directory"));
    }
    let lock = sibling(path, ".lock");
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(&lock).map_err(|source| {
        if source.kind() == io::ErrorKind::AlreadyExists {
            WorkbenchProfileError::Busy { lock: lock.clone() }
        } else {
            WorkbenchProfileError::Io {
                path: lock.clone(),
                source,
            }
        }
    })?;
    let guard =
        WriterGuard::new(lock.clone(), &file).map_err(|source| WorkbenchProfileError::Io {
            path: lock.clone(),
            source,
        })?;
    file.write_all(b"vlajmaji workbench profile writer\n")
        .and_then(|_| file.sync_all())
        .map_err(|source| WorkbenchProfileError::Io { path: lock, source })?;
    drop(file);

    let prior = read_optional(path)?;
    let observed = prior.as_ref().map(|value| value.wire.revision);
    let expected_matches = match (expected_revision, observed) {
        (None, None) => true,
        (Some(expected), Some(observed)) => expected == observed,
        _ => false,
    };
    if !expected_matches {
        return Err(WorkbenchProfileError::Conflict {
            expected: expected_revision,
            observed,
        });
    }
    let wire = prior
        .as_ref()
        .map(|value| value.wire.clone())
        .unwrap_or_else(empty_wire);
    Ok((prior, wire, guard))
}

fn publish_report(
    path: &Path,
    prior: Option<&WorkbenchProfiles>,
    wire: &mut ConfigWire,
    operation: &str,
    profile: Option<&str>,
    prepared: Option<WorkbenchPrepared>,
    before_visible: Option<&dyn Fn() -> Result<(), WorkbenchProfileError>>,
) -> Result<WorkbenchProfileWriteReport, WorkbenchProfileError> {
    let prior_revision = prior.map(|value| value.wire.revision);
    let revision = wire
        .revision
        .checked_add(1)
        .filter(|value| *value <= i64::MAX as u64)
        .ok_or(WorkbenchProfileError::RevisionOverflow(wire.revision))?;
    wire.revision = revision;
    validate_wire(path, wire)?;
    let mut bytes = serde_json::to_vec(wire).map_err(|error| decode_error(path, error))?;
    bytes.push(b'\n');
    let sha256 = digest(&bytes);
    verify_disk_matches(path, prior)?;
    if let Some(check) = before_visible {
        #[cfg(test)]
        run_before_visible_test_hook()?;
        check()?;
    }
    let stage = sibling(path, &format!(".stage-{}-{}", std::process::id(), nonce()));
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(if prior.is_some() {
            current_mode(path).unwrap_or(0o600)
        } else {
            0o600
        });
    }
    let mut file = options
        .open(&stage)
        .map_err(|source| WorkbenchProfileError::Io {
            path: stage.clone(),
            source,
        })?;
    let stage_guard =
        WriterGuard::new(stage.clone(), &file).map_err(|source| WorkbenchProfileError::Io {
            path: stage.clone(),
            source,
        })?;
    #[cfg(test)]
    if REPLACE_NEXT_STAGE.with(|replace| replace.replace(false)) {
        let retained = stage.with_extension("stage-original");
        fs::rename(&stage, &retained).map_err(|source| WorkbenchProfileError::Io {
            path: stage.clone(),
            source,
        })?;
        fs::write(&stage, b"stage replacement sentinel").map_err(|source| {
            WorkbenchProfileError::Io {
                path: stage.clone(),
                source,
            }
        })?;
        return Err(WorkbenchProfileError::Io {
            path: stage,
            source: io::Error::other("injected stage replacement failure"),
        });
    }
    file.write_all(&bytes)
        .and_then(|_| file.flush())
        .and_then(|_| file.sync_all())
        .map_err(|source| WorkbenchProfileError::Io {
            path: stage.clone(),
            source,
        })?;
    drop(file);
    if prior.is_some() {
        copy_mode(path, &stage).map_err(|source| WorkbenchProfileError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        File::open(&stage)
            .and_then(|file| file.sync_all())
            .map_err(|source| WorkbenchProfileError::Io {
                path: stage.clone(),
                source,
            })?;
    }
    verify_disk_matches(path, prior)?;
    if let Some(check) = before_visible {
        check()?;
    }
    fs::rename(&stage, path).map_err(|source| WorkbenchProfileError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let mut stage_guard = stage_guard;
    stage_guard.active = false;
    if let Err(source) = sync_parent(path.parent().unwrap_or_else(|| Path::new("."))) {
        let cause = WorkbenchProfileError::Io {
            path: path.to_path_buf(),
            source,
        };
        return Err(indeterminate_after_publication(
            path,
            prior,
            &WorkbenchProfileWriteReport {
                version: 1,
                operation: operation.into(),
                changed: true,
                prior_revision,
                revision,
                sha256: sha256.clone(),
                profile: profile.map(str::to_owned),
                default_profile: wire.default_profile.clone(),
                active: active_for(wire, profile),
                config: native_path_report(path),
            },
            prepared,
            cause,
        ));
    }
    if let Err(source) = verify_bytes(path, &bytes, &sha256) {
        return Err(indeterminate_after_publication(
            path,
            prior,
            &WorkbenchProfileWriteReport {
                version: 1,
                operation: operation.into(),
                config: native_path_report(path),
                changed: true,
                prior_revision,
                revision,
                sha256: sha256.clone(),
                profile: profile.map(str::to_owned),
                default_profile: wire.default_profile.clone(),
                active: active_for(wire, profile),
            },
            prepared,
            source,
        ));
    }
    let active = active_for(wire, profile);
    Ok(WorkbenchProfileWriteReport {
        version: 1,
        operation: operation.into(),
        changed: true,
        prior_revision,
        revision,
        sha256,
        profile: profile.map(str::to_owned),
        default_profile: wire.default_profile.clone(),
        active,
        config: native_path_report(path),
    })
}

fn unchanged_report(
    path: &Path,
    prior: Option<&WorkbenchProfiles>,
    operation: &str,
    profile: Option<&str>,
) -> Result<WorkbenchProfileWriteReport, WorkbenchProfileError> {
    let prior = prior.ok_or_else(|| invalid(path, "cannot report an unchanged absent config"))?;
    Ok(WorkbenchProfileWriteReport {
        version: 1,
        operation: operation.into(),
        changed: false,
        prior_revision: Some(prior.wire.revision),
        revision: prior.wire.revision,
        sha256: prior.sha256.clone(),
        profile: profile.map(str::to_owned),
        default_profile: prior.wire.default_profile.clone(),
        active: profile
            .and_then(|name| prior.wire.profiles.iter().find(|item| item.name == name))
            .and_then(|item| item.active_generation)
            .and_then(|sequence| prior.wire.prepared.get(sequence as usize).cloned()),
        config: native_path_report(path),
    })
}

fn active_for(wire: &ConfigWire, profile: Option<&str>) -> Option<WorkbenchPrepared> {
    let name = profile?;
    let sequence = wire
        .profiles
        .iter()
        .find(|item| item.name == name)?
        .active_generation?;
    wire.prepared.get(sequence as usize).cloned()
}

fn verify_disk_matches(
    path: &Path,
    prior: Option<&WorkbenchProfiles>,
) -> Result<(), WorkbenchProfileError> {
    let current = read_optional(path)?;
    match (prior, current) {
        (None, None) => Ok(()),
        (Some(old), Some(new)) if old.bytes == new.bytes => Ok(()),
        (old, new) => Err(WorkbenchProfileError::ConfigChanged {
            config: path.to_path_buf(),
            expected_sha256: old.map(|value| value.sha256.clone()),
            observed_sha256: new.map(|value| value.sha256),
        }),
    }
}

fn read_optional(path: &Path) -> Result<Option<WorkbenchProfiles>, WorkbenchProfileError> {
    match fs::symlink_metadata(path) {
        Ok(meta) if meta.file_type().is_symlink() || !meta.is_file() => Err(invalid(
            path,
            "configuration destination must be a regular non-symlink file",
        )),
        Ok(_) => {
            let bytes = fs::read(path).map_err(|source| WorkbenchProfileError::Io {
                path: path.to_path_buf(),
                source,
            })?;
            WorkbenchProfiles::from_bytes(path.to_path_buf(), bytes).map(Some)
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(source) => Err(WorkbenchProfileError::Io {
            path: path.to_path_buf(),
            source,
        }),
    }
}

fn indeterminate_after_publication(
    path: &Path,
    prior: Option<&WorkbenchProfiles>,
    proposed: &WorkbenchProfileWriteReport,
    prepared: Option<WorkbenchPrepared>,
    source: WorkbenchProfileError,
) -> WorkbenchProfileError {
    let observed = fs::read(path).ok();
    let observed_sha256 = observed.as_deref().map(digest);
    let observed_revision = observed.and_then(|bytes| {
        WorkbenchProfiles::from_bytes(path.to_path_buf(), bytes)
            .ok()
            .map(|profiles| profiles.wire.revision)
    });
    WorkbenchProfileError::Indeterminate {
        publication: Box::new(WorkbenchPublicationContext {
            config: native_path_report(path),
            prior_revision: prior.map(|value| value.wire.revision),
            prior_sha256: prior.map(|value| value.sha256.clone()),
            proposed_revision: proposed.revision,
            proposed_sha256: proposed.sha256.clone(),
            observed_revision,
            observed_sha256,
            prepared,
        }),
        source: Box::new(source),
    }
}

fn empty_wire() -> ConfigWire {
    ConfigWire {
        version: CONFIG_VERSION,
        kind: CONFIG_KIND.into(),
        revision: 0,
        default_profile: None,
        profiles: Vec::new(),
        prepared: Vec::new(),
    }
}

fn validate_wire(path: &Path, wire: &ConfigWire) -> Result<(), WorkbenchProfileError> {
    if wire.kind != CONFIG_KIND {
        return Err(invalid(
            path,
            format!("unexpected workbench profile kind {:?}", wire.kind),
        ));
    }
    if wire.version != CONFIG_VERSION {
        return Err(WorkbenchProfileError::UnsupportedVersion {
            kind: CONFIG_KIND,
            found: wire.version,
        });
    }
    if wire.revision > i64::MAX as u64 {
        return Err(invalid(path, "revision exceeds i64::MAX"));
    }
    if wire
        .profiles
        .windows(2)
        .any(|pair| pair[0].name >= pair[1].name)
    {
        return Err(invalid(path, "profiles must be sorted by unique name"));
    }
    for profile in &wire.profiles {
        validate_name(&profile.name, path)?;
        profile
            .selection
            .validate()
            .map_err(|error| invalid(path, error.to_string()))?;
        let catalogue = profile
            .catalogue_root
            .as_ref()
            .map(decode_root)
            .transpose()?;
        let owned = profile.owned_root.as_ref().map(decode_root).transpose()?;
        validate_roots(
            path,
            &profile.selection,
            catalogue.as_deref(),
            owned.as_deref(),
        )?;
        if profile
            .active_generation
            .is_some_and(|sequence| sequence > i64::MAX as u64)
        {
            return Err(invalid(path, "active generation exceeds i64::MAX"));
        }
    }
    if wire
        .default_profile
        .as_ref()
        .is_some_and(|name| !wire.profiles.iter().any(|item| &item.name == name))
    {
        return Err(invalid(path, "default_profile does not name a profile"));
    }
    if wire.prepared.len() as u64 > i64::MAX as u64 {
        return Err(invalid(path, "prepared sequence exceeds i64::MAX"));
    }
    for (index, row) in wire.prepared.iter().enumerate() {
        if row.sequence != index as u64 {
            return Err(invalid(path, "prepared sequence must be contiguous"));
        }
        validate_name(&row.profile, path)?;
        let root = decode_root(&row.root)?;
        if !root.is_absolute() {
            return Err(invalid(path, "prepared root must be absolute"));
        }
        validate_sha(&row.binding_sha256).map_err(|detail| invalid(path, detail))?;
    }
    for profile in &wire.profiles {
        if let Some(sequence) = profile.active_generation {
            let row = wire
                .prepared
                .get(sequence as usize)
                .ok_or_else(|| invalid(path, "active generation does not name a prepared row"))?;
            if row.sequence != sequence || row.profile != profile.name {
                return Err(invalid(
                    path,
                    "active generation row belongs to another profile",
                ));
            }
        }
    }
    Ok(())
}

fn validate_roots(
    path: &Path,
    selection: &WorkbenchSelection,
    catalogue: Option<&Path>,
    owned: Option<&Path>,
) -> Result<(), WorkbenchProfileError> {
    if selection.packs.is_empty() != catalogue.is_none() {
        return Err(invalid(
            path,
            "catalogue root presence must match selected packs",
        ));
    }
    if selection.owned.is_some() != owned.is_some() {
        return Err(invalid(
            path,
            "owned root presence must match owned selection",
        ));
    }
    for root in catalogue.into_iter().chain(owned) {
        if !root.is_absolute() {
            return Err(invalid(path, "configured roots must be absolute"));
        }
    }
    Ok(())
}

fn validate_name(name: &str, path: &Path) -> Result<(), WorkbenchProfileError> {
    if name.is_empty()
        || name.len() > 64
        || !name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
    {
        return Err(invalid(path, format!("invalid profile name {name:?}")));
    }
    Ok(())
}

fn validate_sha(value: &str) -> Result<(), String> {
    if value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        Ok(())
    } else {
        Err("digest must be lowercase SHA-256".into())
    }
}

fn deserialize_required_option<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: DeserializeOwned,
{
    Option::<T>::deserialize(deserializer)
}

fn deserialize_required_native_path<'de, D>(
    deserializer: D,
) -> Result<Option<NativePathReport>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum OptionalNativePath {
        Null(()),
        Value(NativePathReportWire),
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct NativePathReportWire {
        encoding: String,
        hex: String,
        #[serde(deserialize_with = "deserialize_required_option")]
        utf8: Option<String>,
    }

    match OptionalNativePath::deserialize(deserializer)? {
        OptionalNativePath::Null(()) => Ok(None),
        OptionalNativePath::Value(wire) => Ok(Some(NativePathReport {
            encoding: wire.encoding,
            hex: wire.hex,
            utf8: wire.utf8,
        })),
    }
}

fn deserialize_native_path<'de, D>(deserializer: D) -> Result<NativePathReport, D::Error>
where
    D: serde::Deserializer<'de>,
{
    crate::workbench::deserialize_native_path(deserializer)
}

fn absolute_path(path: &Path) -> Result<PathBuf, WorkbenchProfileError> {
    if path.is_absolute() {
        return Ok(path.to_path_buf());
    }
    std::env::current_dir()
        .map(|cwd| cwd.join(path))
        .map_err(|source| WorkbenchProfileError::Io {
            path: path.to_path_buf(),
            source,
        })
}

fn resolve_inputs<const N: usize>(
    config: &Path,
    paths: [Option<&Path>; N],
    destination: Option<&Path>,
) -> Result<(PathBuf, Option<PathBuf>), WorkbenchProfileError> {
    let needs_cwd = !config.is_absolute()
        || paths.into_iter().flatten().any(|path| !path.is_absolute())
        || destination.is_some_and(|path| !path.is_absolute());
    let cwd = needs_cwd
        .then(std::env::current_dir)
        .transpose()
        .map_err(|source| WorkbenchProfileError::Io {
            path: config.to_path_buf(),
            source,
        })?;
    let resolved_config = resolve_with_cwd(config, cwd.as_deref());
    decode_root(&native_path_report(&resolved_config))?;
    Ok((resolved_config, cwd))
}

fn resolve_optional(path: Option<&Path>, cwd: Option<&Path>) -> Option<PathBuf> {
    path.map(|path| resolve_with_cwd(path, cwd))
}

fn resolve_with_cwd(path: &Path, cwd: Option<&Path>) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.expect("relative input requires captured current directory")
            .join(path)
    }
}

fn decode_root(report: &NativePathReport) -> Result<PathBuf, WorkbenchProfileError> {
    decode_native_path(report).map_err(|error| match error {
        WorkbenchError::UnsupportedNativeEncoding { encoding } => {
            WorkbenchProfileError::UnsupportedNativeEncoding { encoding }
        }
        other => invalid(Path::new("<profile>"), other.to_string()),
    })
}

fn verify_bytes(
    path: &Path,
    expected: &[u8],
    expected_sha256: &str,
) -> Result<(), WorkbenchProfileError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_file() => {
            return Err(WorkbenchProfileError::ConfigChanged {
                config: path.to_path_buf(),
                expected_sha256: Some(expected_sha256.to_owned()),
                observed_sha256: None,
            });
        }
        Ok(_) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Err(WorkbenchProfileError::ConfigChanged {
                config: path.to_path_buf(),
                expected_sha256: Some(expected_sha256.to_owned()),
                observed_sha256: None,
            });
        }
        Err(source) => {
            return Err(WorkbenchProfileError::Io {
                path: path.to_path_buf(),
                source,
            });
        }
    }
    let actual = match fs::read(path) {
        Ok(bytes) => Some(bytes),
        Err(error) if error.kind() == io::ErrorKind::NotFound => None,
        Err(source) => {
            return Err(WorkbenchProfileError::Io {
                path: path.to_path_buf(),
                source,
            });
        }
    };
    if actual.as_deref() == Some(expected) {
        return Ok(());
    }
    Err(WorkbenchProfileError::ConfigChanged {
        config: path.to_path_buf(),
        expected_sha256: Some(expected_sha256.to_owned()),
        observed_sha256: actual.as_deref().map(digest),
    })
}

fn invalid(path: &Path, detail: impl Into<String>) -> WorkbenchProfileError {
    WorkbenchProfileError::InvalidPolicy {
        path: path.to_path_buf(),
        detail: detail.into(),
    }
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn decode_error(path: &Path, error: serde_json::Error) -> WorkbenchProfileError {
    WorkbenchProfileError::Decode {
        path: path.to_path_buf(),
        detail: error.to_string(),
    }
}

fn sibling(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(suffix);
    path.with_file_name(name)
}

fn nonce() -> u128 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    time ^ u128::from(NEXT.fetch_add(1, Ordering::Relaxed))
}

#[cfg(unix)]
fn current_mode(path: &Path) -> io::Result<u32> {
    use std::os::unix::fs::PermissionsExt;
    Ok(fs::metadata(path)?.permissions().mode() & 0o7777)
}

#[cfg(unix)]
fn copy_mode(source: &Path, destination: &Path) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mode = fs::metadata(source)?.permissions().mode();
    fs::set_permissions(destination, fs::Permissions::from_mode(mode))
}

#[cfg(not(unix))]
fn copy_mode(_source: &Path, _destination: &Path) -> io::Result<()> {
    Ok(())
}

#[cfg(test)]
thread_local! {
    static FAIL_NEXT_PARENT_SYNC: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static REPLACE_NEXT_STAGE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static REBUILD_CATALOGUE_BEFORE_VISIBLE: std::cell::RefCell<Option<(PathBuf, u64)>> = const { std::cell::RefCell::new(None) };
    static REBUILD_CATALOGUE_AFTER_RENAME: std::cell::RefCell<Option<(PathBuf, u64)>> = const { std::cell::RefCell::new(None) };
}

#[cfg(test)]
fn run_before_visible_test_hook() -> Result<(), WorkbenchProfileError> {
    if let Some((catalogue, expected_revision)) =
        REBUILD_CATALOGUE_BEFORE_VISIBLE.with(|hook| hook.borrow_mut().take())
    {
        CollectionCatalogue::rebuild(&catalogue, expected_revision)
            .map_err(WorkbenchProfileError::from)?;
    }
    Ok(())
}

fn sync_parent(path: &Path) -> io::Result<()> {
    #[cfg(test)]
    if let Some((catalogue, expected_revision)) =
        REBUILD_CATALOGUE_AFTER_RENAME.with(|hook| hook.borrow_mut().take())
    {
        CollectionCatalogue::rebuild(&catalogue, expected_revision)
            .map_err(|error| io::Error::other(error.to_string()))?;
    }
    #[cfg(test)]
    if FAIL_NEXT_PARENT_SYNC.with(|fail| fail.replace(false)) {
        return Err(io::Error::other("injected parent sync failure"));
    }
    File::open(path)?.sync_all()
}

fn configured_catalogue_status(
    root: NativePathReport,
    profile: &WorkbenchProfile,
    roster: CollectionMetadataRoster,
) -> WorkbenchConfiguredCatalogueStatus {
    let selected: std::collections::BTreeSet<_> = profile
        .selection
        .packs
        .iter()
        .map(ToString::to_string)
        .collect();
    let disabled: std::collections::BTreeSet<_> = profile
        .selection
        .disabled
        .iter()
        .map(ToString::to_string)
        .collect();
    let mut rows = Vec::new();
    for name in profile.selection.packs.iter().map(ToString::to_string) {
        let registered = roster
            .packs
            .iter()
            .find(|pack| pack.pack_id.to_string() == name);
        rows.push(WorkbenchConfiguredPackStatus {
            pack_id: name.clone(),
            selected: true,
            disabled: disabled.contains(&name),
            installation_state: registered.map(|pack| pack.installation_state),
            installed_release_sha256: registered
                .and_then(|pack| pack.installed_release_sha256.clone()),
            registrations: registered.map_or_else(Vec::new, |pack| pack.registrations.clone()),
        });
    }
    let mut remaining: Vec<_> = roster.packs.iter().collect();
    remaining.sort_by_key(|pack| {
        [
            PackId::LensiskuJbotci,
            PackId::Llg,
            PackId::LaLojban,
            PackId::Sutysisku,
        ]
        .iter()
        .position(|pack_id| pack_id.as_str() == pack.pack_id.to_string())
        .unwrap_or(usize::MAX)
    });
    for pack in remaining {
        let name = pack.pack_id.to_string();
        if selected.contains(&name) {
            continue;
        }
        rows.push(WorkbenchConfiguredPackStatus {
            pack_id: name,
            selected: false,
            disabled: false,
            installation_state: Some(pack.installation_state),
            installed_release_sha256: pack.installed_release_sha256.clone(),
            registrations: pack.registrations.clone(),
        });
    }
    WorkbenchConfiguredCatalogueStatus {
        root,
        local_revision: roster.local_revision,
        generation: roster.generation,
        packs: rows,
    }
}

fn observe_prepared(
    profile: &WorkbenchProfile,
    prepared: WorkbenchPrepared,
) -> WorkbenchPreparedStatusReport {
    match ConfiguredWorkbenchView::open_for_profile(profile, &prepared) {
        Ok(_) => WorkbenchPreparedStatusReport {
            prepared,
            state: WorkbenchPreparedState::Current,
            error: None,
        },
        Err(error) => {
            let state = match &error {
                WorkbenchProfileError::View(source)
                    if matches!(source.as_ref(), WorkbenchError::StaleDependencies { .. }) =>
                {
                    WorkbenchPreparedState::Stale
                }
                WorkbenchProfileError::Io { .. } | WorkbenchProfileError::MissingFile(_) => {
                    WorkbenchPreparedState::Unavailable
                }
                WorkbenchProfileError::View(source)
                    if matches!(source.as_ref(), WorkbenchError::Io { .. }) =>
                {
                    WorkbenchPreparedState::Unavailable
                }
                WorkbenchProfileError::View(source)
                    if missing_prepared_database(source, &prepared) =>
                {
                    WorkbenchPreparedState::Unavailable
                }
                WorkbenchProfileError::View(source) if unreadable_prepared_database(source) => {
                    WorkbenchPreparedState::Unavailable
                }
                _ => WorkbenchPreparedState::Invalid,
            };
            WorkbenchPreparedStatusReport {
                prepared,
                state,
                error: Some(WorkbenchPreparedStatusError {
                    code: status_error_code(&error).into(),
                    message: error.to_string(),
                }),
            }
        }
    }
}

fn missing_prepared_database(error: &WorkbenchError, prepared: &WorkbenchPrepared) -> bool {
    if !matches!(
        error,
        WorkbenchError::Store(store)
            if matches!(
                store.as_ref(),
                crate::DictionaryStoreError::InvalidStore {
                    reason: crate::DictionaryStoreInvalidity::MissingDatabase,
                    ..
                }
            )
    ) {
        return false;
    }
    let Ok(root) = decode_root(&prepared.root) else {
        return false;
    };
    match fs::metadata(root.join("dictionary/dictionary.sqlite3")) {
        Err(error) => matches!(
            error.kind(),
            io::ErrorKind::NotFound | io::ErrorKind::PermissionDenied
        ),
        Ok(_) => false,
    }
}

fn unreadable_prepared_database(error: &WorkbenchError) -> bool {
    matches!(
        error,
        WorkbenchError::Store(store)
            if matches!(
                store.as_ref(),
                crate::DictionaryStoreError::Io(error)
                    if matches!(error.kind(), io::ErrorKind::NotFound | io::ErrorKind::PermissionDenied)
            ) || matches!(
                store.as_ref(),
                crate::DictionaryStoreError::Backend(rusqlite::Error::SqliteFailure(failure, _))
                    if failure.code == rusqlite::ErrorCode::CannotOpen
            )
    )
}

fn status_error_code(error: &WorkbenchProfileError) -> &'static str {
    match error {
        WorkbenchProfileError::Io { .. } => "workbench_config_io",
        WorkbenchProfileError::Decode { .. } | WorkbenchProfileError::InvalidPolicy { .. } => {
            "invalid_workbench_config"
        }
        WorkbenchProfileError::UnsupportedVersion { .. }
        | WorkbenchProfileError::UnsupportedNativeEncoding { .. } => "unsupported_workbench_config",
        WorkbenchProfileError::MissingFile(_) => "workbench_config_missing",
        WorkbenchProfileError::NoDefault => "workbench_no_default",
        WorkbenchProfileError::UnknownProfile { .. } => "unknown_workbench_profile",
        WorkbenchProfileError::Unprepared { .. } => "unprepared_workbench_profile",
        WorkbenchProfileError::Conflict { .. } => "workbench_config_conflict",
        WorkbenchProfileError::Busy { .. } => "workbench_config_busy",
        WorkbenchProfileError::RevisionOverflow(_) => "invalid_workbench_config",
        WorkbenchProfileError::ConfigChanged { .. } => "stale_workbench_config",
        WorkbenchProfileError::BindingMismatch { .. } => "invalid_workbench_config_binding",
        WorkbenchProfileError::View(error) => workbench_status_error_code(error),
        WorkbenchProfileError::Indeterminate { .. } => "workbench_config_indeterminate",
    }
}

fn workbench_status_error_code(error: &WorkbenchError) -> &'static str {
    match error {
        WorkbenchError::InvalidSelection { .. } => "invalid_workbench_selection",
        WorkbenchError::UnsupportedVersion { .. }
        | WorkbenchError::UnsupportedNativeEncoding { .. } => "unsupported_workbench_version",
        WorkbenchError::ExistingDestination(_) => "existing_workbench",
        WorkbenchError::InvalidBinding { .. } => "invalid_workbench",
        WorkbenchError::StaleDependencies { .. } => "stale_workbench",
        WorkbenchError::Io { .. } => "workbench_io",
        WorkbenchError::Curation(_) => "workbench_context",
        WorkbenchError::Collection(_)
        | WorkbenchError::Owned(_)
        | WorkbenchError::Pack(_)
        | WorkbenchError::Store(_) => "workbench_dependency",
        WorkbenchError::Serialization(_) => "workbench_response",
    }
}

/// A captured config/profile and the exact immutable S01 view it names.
pub struct ConfiguredWorkbenchView {
    profiles: WorkbenchProfiles,
    view: crate::WorkbenchView,
    identity: WorkbenchProfileIdentity,
}

impl ConfiguredWorkbenchView {
    /// Captures an explicit config, resolves one profile and opens its active root.
    ///
    /// # Errors
    /// Returns config, profile, active-root, binding, or freshness errors.
    pub fn open(
        config: impl AsRef<Path>,
        name: Option<&str>,
    ) -> Result<Self, WorkbenchProfileError> {
        let profiles = WorkbenchProfiles::read(config)?;
        let profile = profiles.profile(name)?.clone();
        let sequence =
            profile
                .active_generation
                .ok_or_else(|| WorkbenchProfileError::Unprepared {
                    name: profile.name.clone(),
                })?;
        let prepared = profiles
            .wire
            .prepared
            .get(sequence as usize)
            .cloned()
            .ok_or_else(|| invalid(&profiles.path, "active prepared row is missing"))?;
        let opened = Self::open_for_profile(&profile, &prepared);
        profiles.verify_unchanged()?;
        let view = opened?;
        let identity = identity_for(
            &profiles,
            name,
            &profile,
            &prepared,
            view.binding().binding_sha256(),
        )?;
        let captured = Self {
            profiles,
            view,
            identity,
        };
        captured.ensure_current()?;
        Ok(captured)
    }

    fn open_for_profile(
        profile: &WorkbenchProfile,
        prepared: &WorkbenchPrepared,
    ) -> Result<crate::WorkbenchView, WorkbenchProfileError> {
        let root = decode_root(&prepared.root)?;
        let view = crate::WorkbenchView::open(root)?;
        let observed = view.binding().binding_sha256().to_owned();
        if observed != prepared.binding_sha256 {
            return Err(WorkbenchProfileError::BindingMismatch {
                expected: prepared.binding_sha256.clone(),
                observed,
            });
        }
        if view.binding().selection() != &profile.selection
            || view
                .binding()
                .catalogue()
                .as_ref()
                .map(|binding| &binding.root)
                != profile.catalogue_root.as_ref()
            || view.binding().owned().as_ref().map(|binding| &binding.root)
                != profile.owned_root.as_ref()
        {
            return Err(WorkbenchProfileError::BindingMismatch {
                expected: digest(
                    &serde_json::to_vec(profile)
                        .map_err(|error| decode_error(Path::new("<profile>"), error))?,
                ),
                observed: view.binding().binding_sha256().to_owned(),
            });
        }
        Ok(view)
    }

    /// Borrows the compact configured identity.
    #[must_use]
    pub fn identity(&self) -> &WorkbenchProfileIdentity {
        &self.identity
    }

    /// Borrows the accepted S01 view.
    #[must_use]
    pub fn view(&self) -> &crate::WorkbenchView {
        &self.view
    }

    /// Checks exact config bytes around the existing S01 dependency guard.
    ///
    /// # Errors
    /// Returns ConfigChanged or the actual S01 freshness error.
    pub fn ensure_current(&self) -> Result<(), WorkbenchProfileError> {
        self.profiles.verify_unchanged()?;
        let source = self
            .view
            .ensure_current()
            .map_err(|error| WorkbenchProfileError::View(Box::new(error)));
        self.profiles.verify_unchanged()?;
        source
    }

    /// Returns the S01 snapshot manifest with only its generation replaced by the effective identity.
    #[must_use]
    pub fn snapshot_manifest(&self) -> DictionarySnapshotManifest {
        let mut manifest = self.view.snapshot_manifest();
        manifest.generation = self.identity.effective_binding_sha256.clone();
        manifest
    }
}

fn identity_for(
    profiles: &WorkbenchProfiles,
    requested: Option<&str>,
    profile: &WorkbenchProfile,
    prepared: &WorkbenchPrepared,
    view_binding_sha256: &str,
) -> Result<WorkbenchProfileIdentity, WorkbenchProfileError> {
    let config = native_path_report(&profiles.path);
    let requested_profile = requested.map(str::to_owned);
    let payload = IdentityPayload {
        version: 1,
        kind: IDENTITY_KIND,
        config: &config,
        config_revision: profiles.wire.revision,
        config_sha256: &profiles.sha256,
        requested_profile: &requested_profile,
        profile: &profile.name,
        prepared_sequence: prepared.sequence,
        view_binding_sha256,
    };
    let effective_binding_sha256 =
        digest(&serde_json::to_vec(&payload).map_err(|error| decode_error(&profiles.path, error))?);
    Ok(WorkbenchProfileIdentity {
        version: 1,
        kind: IDENTITY_KIND.into(),
        config,
        config_revision: profiles.wire.revision,
        config_sha256: profiles.sha256.clone(),
        requested_profile: requested.map(str::to_owned),
        profile: profile.name.clone(),
        prepared_sequence: prepared.sequence,
        view_binding_sha256: view_binding_sha256.into(),
        effective_binding_sha256,
    })
}

#[cfg(all(test, unix))]
mod native_sibling_tests {
    use super::sibling;
    use std::{ffi::OsStr, os::unix::ffi::OsStrExt, path::Path};

    #[test]
    fn test_sibling_preserves_non_utf8_basename_bytes() {
        let path = Path::new(OsStr::from_bytes(b"profiles-\xff.json"));
        let sibling = sibling(path, ".lock");
        assert_eq!(sibling.as_os_str().as_bytes(), b"profiles-\xff.json.lock");
    }
}

#[cfg(test)]
mod publication_tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn test_root(label: &str) -> PathBuf {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        for _ in 0..64 {
            let path = std::env::temp_dir().join(format!(
                "vlajmaji-profile-private-{label}-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            match fs::create_dir(&path) {
                Ok(()) => return path,
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => panic!("create private profile test root: {error}"),
            }
        }
        panic!("could not create exclusive private profile test root")
    }

    #[test]
    fn test_publish_pre_visible_error_preserves_original_bytes() {
        let root = test_root("pre-visible");
        let config = root.join("profiles.json");
        let mut initial = empty_wire();
        let created =
            publish_report(&config, None, &mut initial, "test", None, None, None).unwrap();
        let original = fs::read(&config).unwrap();
        assert_eq!(created.revision, 1);

        let prior = WorkbenchProfiles::read(&config).unwrap();
        let mut next = prior.wire.clone();
        let calls = std::cell::Cell::new(0_u8);
        let stop_after_stage = || {
            let call = calls.get() + 1;
            calls.set(call);
            if call == 2 {
                Err(WorkbenchProfileError::InvalidPolicy {
                    path: config.clone(),
                    detail: "injected pre-visible failure".into(),
                })
            } else {
                Ok(())
            }
        };
        let error = publish_report(
            &config,
            Some(&prior),
            &mut next,
            "test",
            None,
            None,
            Some(&stop_after_stage),
        )
        .unwrap_err();
        assert!(matches!(
            error,
            WorkbenchProfileError::InvalidPolicy { detail, .. }
                if detail == "injected pre-visible failure"
        ));
        assert_eq!(
            calls.get(),
            2,
            "the error occurs after staged bytes are synced"
        );
        assert_eq!(fs::read(&config).unwrap(), original);
        assert_eq!(
            fs::read_dir(&root)
                .unwrap()
                .filter_map(Result::ok)
                .filter(|entry| entry.file_name().to_string_lossy().contains(".stage-"))
                .count(),
            0
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn test_publish_after_parent_sync_failure_reports_visible_indeterminate() {
        let root = test_root("post-visible");
        let config = root.join("profiles.json");
        let mut wire = empty_wire();
        FAIL_NEXT_PARENT_SYNC.with(|fail| fail.set(true));
        let error = publish_report(&config, None, &mut wire, "test", None, None, None).unwrap_err();
        let WorkbenchProfileError::Indeterminate {
            publication,
            source,
        } = error
        else {
            panic!("post-rename sync failure must be Indeterminate")
        };
        assert!(matches!(source.as_ref(), WorkbenchProfileError::Io { .. }));
        assert_eq!(publication.prior_revision, None);
        assert_eq!(publication.proposed_revision, 1);
        assert_eq!(publication.observed_revision, Some(1));
        let visible = fs::read(&config).unwrap();
        assert_eq!(digest(&visible), publication.proposed_sha256);
        assert_eq!(
            WorkbenchProfiles::read(&config).unwrap().wire.revision,
            publication.proposed_revision
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn test_writer_guard_keeps_replaced_lock_sentinel() {
        let root = test_root("lock-sentinel");
        let config = root.join("profiles.json");
        let (_prior, _wire, guard) = begin_write(&config, None).unwrap();
        let lock = sibling(&config, ".lock");
        let original_lock = lock.with_extension("lock-original");
        fs::rename(&lock, &original_lock).unwrap();
        fs::write(&lock, b"replacement sentinel").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            let original = fs::metadata(&original_lock).unwrap();
            let replacement = fs::metadata(&lock).unwrap();
            assert_ne!(
                (original.dev(), original.ino()),
                (replacement.dev(), replacement.ino())
            );
        }
        drop(guard);
        assert_eq!(fs::read(&lock).unwrap(), b"replacement sentinel");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn test_publish_stage_guard_keeps_replacement_sentinel() {
        let root = test_root("stage-sentinel");
        let config = root.join("profiles.json");
        let mut wire = empty_wire();
        publish_report(&config, None, &mut wire, "test", None, None, None).unwrap();
        let original = fs::read(&config).unwrap();
        let prior = WorkbenchProfiles::read(&config).unwrap();
        let mut next = prior.wire.clone();
        REPLACE_NEXT_STAGE.with(|replace| replace.set(true));
        let error =
            publish_report(&config, Some(&prior), &mut next, "test", None, None, None).unwrap_err();
        assert!(matches!(error, WorkbenchProfileError::Io { .. }));
        assert_eq!(fs::read(&config).unwrap(), original);
        let entries = fs::read_dir(&root)
            .unwrap()
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .collect::<Vec<_>>();
        let sentinel = entries
            .iter()
            .find(|path| fs::read(path).is_ok_and(|bytes| bytes == b"stage replacement sentinel"))
            .unwrap();
        let retained = entries
            .iter()
            .find(|path| {
                path.file_name()
                    .is_some_and(|name| name.to_string_lossy().ends_with(".stage-original"))
            })
            .unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            let original = fs::metadata(retained).unwrap();
            let replacement = fs::metadata(sentinel).unwrap();
            assert_ne!(
                (original.dev(), original.ino()),
                (replacement.dev(), replacement.ino())
            );
        }
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn test_refresh_source_change_after_build_preserves_published_config() {
        let root = test_root("refresh-source-change");
        let catalogue = root.join("catalogue");
        let dictionary = vlajmaji_lexicon::Dictionary::from_document(Default::default()).unwrap();
        let sealed = vlajmaji_lexicon::seal_release(
            PackId::Llg,
            vlajmaji_lexicon::PackPayloadKind::Synthetic,
            &dictionary,
        )
        .unwrap();
        let request = crate::CollectionRequest::new(
            vec![crate::CollectionInput::SealedBytes {
                pack_id: PackId::Llg,
                release: std::borrow::Cow::Borrowed(&sealed.release),
                seal: std::borrow::Cow::Borrowed(&sealed.seal),
            }],
            crate::CollectionSelection::Packs(vec![PackId::Llg]),
            true,
        )
        .unwrap();
        CollectionCatalogue::install(&catalogue, request).unwrap();
        let config = root.join("profiles.json");
        let profile = WorkbenchProfileConfigureRequest {
            name: "alpha".into(),
            catalogue_root: Some(catalogue.clone()),
            owned_root: None,
            selection: WorkbenchSelection {
                version: 1,
                packs: vec![PackId::Llg],
                disabled: Vec::new(),
                owned: None,
                contexts: Vec::new(),
            },
            make_default: true,
        };
        WorkbenchProfiles::configure(&config, profile, None).unwrap();
        WorkbenchProfiles::refresh(&config, None, root.join("prepared-r1"), 1).unwrap();
        let before = fs::read(&config).unwrap();

        REBUILD_CATALOGUE_BEFORE_VISIBLE.with(|hook| {
            *hook.borrow_mut() = Some((catalogue.clone(), 1));
        });
        let error =
            WorkbenchProfiles::refresh(&config, None, root.join("prepared-r2"), 2).unwrap_err();
        assert!(matches!(
            error,
            WorkbenchProfileError::View(ref source)
                if matches!(source.as_ref(), WorkbenchError::StaleDependencies { .. })
        ));
        assert_eq!(fs::read(&config).unwrap(), before);
        let latest = CollectionCatalogue::open(&catalogue).unwrap();
        assert_eq!(latest.local_revision().unwrap(), 2);
        let latest_generation = latest.generation().unwrap().unwrap().generation_id;
        drop(latest);
        let report = WorkbenchProfiles::read(&config).unwrap().report();
        assert_eq!(report.revision, 2);
        assert_eq!(report.prepared.len(), 1);
        assert_eq!(report.profiles[0].active_generation, Some(0));

        let refreshed_root = root.join("prepared-r2-after-source-change");
        let refreshed =
            WorkbenchProfiles::refresh(&config, None, &refreshed_root, report.revision).unwrap();
        assert_eq!(refreshed.publication.revision, 3);
        assert_eq!(
            refreshed
                .view
                .binding
                .catalogue()
                .unwrap()
                .generation_id
                .as_deref(),
            Some(latest_generation.as_str())
        );
        let latest_report = WorkbenchProfiles::read(&config).unwrap().report();
        assert_eq!(latest_report.revision, 3);
        assert_eq!(latest_report.prepared.len(), 2);
        assert_eq!(latest_report.profiles[0].active_generation, Some(1));
        assert_eq!(latest_report.prepared[1].sequence, 1);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn test_refresh_source_change_after_rename_reports_indeterminate() {
        let root = test_root("refresh-source-after-rename");
        let catalogue = root.join("catalogue");
        let dictionary = vlajmaji_lexicon::Dictionary::from_document(Default::default()).unwrap();
        let sealed = vlajmaji_lexicon::seal_release(
            PackId::Llg,
            vlajmaji_lexicon::PackPayloadKind::Synthetic,
            &dictionary,
        )
        .unwrap();
        let request = crate::CollectionRequest::new(
            vec![crate::CollectionInput::SealedBytes {
                pack_id: PackId::Llg,
                release: std::borrow::Cow::Borrowed(&sealed.release),
                seal: std::borrow::Cow::Borrowed(&sealed.seal),
            }],
            crate::CollectionSelection::Packs(vec![PackId::Llg]),
            true,
        )
        .unwrap();
        CollectionCatalogue::install(&catalogue, request).unwrap();
        let config = root.join("profiles.json");
        let profile = WorkbenchProfileConfigureRequest {
            name: "alpha".into(),
            catalogue_root: Some(catalogue.clone()),
            owned_root: None,
            selection: WorkbenchSelection {
                version: 1,
                packs: vec![PackId::Llg],
                disabled: Vec::new(),
                owned: None,
                contexts: Vec::new(),
            },
            make_default: true,
        };
        WorkbenchProfiles::configure(&config, profile, None).unwrap();
        WorkbenchProfiles::refresh(&config, None, root.join("prepared-r1"), 1).unwrap();
        WorkbenchProfiles::refresh(&config, None, root.join("prepared-r2"), 2).unwrap();
        let prior = WorkbenchProfiles::read(&config).unwrap();
        assert_eq!(prior.report().revision, 3);
        let destination = root.join("prepared-r3");

        REBUILD_CATALOGUE_AFTER_RENAME.with(|hook| {
            *hook.borrow_mut() = Some((catalogue.clone(), 1));
        });
        let error = WorkbenchProfiles::refresh(&config, None, &destination, 3).unwrap_err();
        let WorkbenchProfileError::Indeterminate {
            publication,
            source,
        } = error
        else {
            panic!("a source change after the config rename must be Indeterminate")
        };
        assert!(matches!(
            source.as_ref(),
            WorkbenchProfileError::View(inner)
                if matches!(inner.as_ref(), WorkbenchError::StaleDependencies { .. })
        ));
        assert_eq!(publication.prior_revision, Some(3));
        assert_eq!(publication.proposed_revision, 4);
        assert_eq!(publication.observed_revision, Some(4));
        assert_eq!(
            publication.prepared.as_ref().unwrap().root,
            native_path_report(&destination)
        );
        let visible = fs::read(&config).unwrap();
        assert_eq!(digest(&visible), publication.proposed_sha256);
        let after = WorkbenchProfiles::read(&config).unwrap().report();
        assert_eq!(after.revision, 4);
        assert_eq!(after.prepared.len(), 3);
        assert_eq!(after.profiles[0].active_generation, Some(2));
        assert!(destination.join("dictionary/dictionary.sqlite3").is_file());
        fs::remove_dir_all(root).unwrap();
    }
}
