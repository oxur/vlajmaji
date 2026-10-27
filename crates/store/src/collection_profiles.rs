//! Explicit, versioned user selection profiles for installed collections.
//!
//! This module reads exactly the path supplied by its caller. Environment and
//! XDG/HOME location resolution belongs to the command-line application.

use std::{
    collections::BTreeMap,
    error::Error,
    fmt,
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use vlajmaji_lexicon::PackId;

use crate::{CollectionCatalogue, NativePathReport, native_path_report};

mod policy;
use policy::DisabledPolicy;

const CONFIG_VERSION: u32 = 1;

/// Caller-owned policy values for one named collection selection.
///
/// This value is not a validated model by itself; persisted reads and writer
/// requests validate its fields at their respective boundaries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CollectionProfile {
    /// Profile name.
    pub name: String,
    /// Absolute native catalogue root.
    pub root: PathBuf,
    /// Ordered pack membership. An empty vector deliberately selects none.
    pub packs: Vec<PackId>,
    /// Independently disabled registered pack IDs.
    pub disabled: Vec<PackId>,
}

/// Exact file identity captured by a successful explicit configuration read.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CollectionProfileIdentity {
    /// Profile name selected by the caller.
    pub name: String,
    /// Native configuration file path.
    pub config_path: NativePathReport,
    /// Configuration format version.
    pub version: u32,
    /// Monotonic user configuration revision.
    pub revision: u64,
    /// SHA-256 of the exact captured file bytes.
    pub sha256: String,
}

/// Public summary of one configured profile.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CollectionProfileSummary {
    /// Profile name.
    pub name: String,
    /// Absolute native root descriptor.
    pub root: NativePathReport,
    /// Ordered pack membership.
    pub packs: Vec<PackId>,
    /// Independently disabled IDs.
    pub disabled: Vec<PackId>,
}

/// Versioned report for a successfully read configuration file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CollectionProfilesReport {
    /// Report schema version.
    pub version: u32,
    /// Native explicit configuration path.
    pub path: NativePathReport,
    /// Configuration wire version.
    pub config_version: u32,
    /// Monotonic configuration revision.
    pub revision: u64,
    /// Exact captured file SHA-256.
    pub sha256: String,
    /// Default profile name, if declared.
    pub default_profile: Option<String>,
    /// Every profile in deterministic name order.
    pub profiles: Vec<CollectionProfileSummary>,
}

/// Successful configure result; semantic no-op writes preserve original bytes/revision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CollectionProfileWriteReport {
    /// Report schema version.
    pub version: u32,
    /// Successful write report.
    pub ok: bool,
    /// Operation name.
    pub operation: String,
    /// `committed` or `unchanged`.
    pub outcome: String,
    /// Native configuration path.
    pub path: NativePathReport,
    /// Profile changed or confirmed.
    pub profile: String,
    /// Resulting default profile.
    pub default_profile: Option<String>,
    /// Prior revision, if a file existed.
    pub prior_revision: Option<u64>,
    /// Resulting revision.
    pub revision: u64,
    /// Prior exact bytes hash, if a file existed.
    pub prior_sha256: Option<String>,
    /// Resulting exact bytes hash.
    pub sha256: String,
    /// Catalogue local revision observed during validation.
    pub observed_catalogue_revision: u64,
    /// Availability observation at configure time; not an integrity audit.
    pub observed_availability: String,
}

/// Request for one explicit named-profile configuration update.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollectionProfileWriteRequest {
    /// Profile name to set.
    pub name: String,
    /// Native catalogue root. A direct relative root is resolved against the
    /// same frozen cwd as a relative config path; a ROOT read from the file is
    /// instead resolved beside that config file.
    pub root: PathBuf,
    /// Ordered pack membership; empty means no selected packs.
    pub packs: Vec<PackId>,
    /// Independently disabled registered IDs.
    pub disabled: Vec<PackId>,
    /// Whether to make this profile the default.
    pub make_default: bool,
}

/// Request a membership change while preserving the profile's latest disabled set.
///
/// Unlike [`CollectionProfileWriteRequest`], this request does not state an
/// enablement policy. The existing disabled IDs are read only after the
/// configuration writer lock is held; a new profile starts with no disabled IDs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollectionProfileMembershipWriteRequest {
    /// Profile name to set.
    pub name: String,
    /// Native catalogue root. A direct relative root is resolved against the
    /// same frozen cwd as a relative config path.
    pub root: PathBuf,
    /// Ordered pack membership; empty means no selected packs.
    pub packs: Vec<PackId>,
    /// Whether to make this profile the default.
    pub make_default: bool,
}

/// Choose all current profile members or a unique explicit subset for a
/// partial enablement-policy change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CollectionPolicySelection {
    /// Select every pack currently present in the latest profile snapshot.
    All,
    /// Select a nonempty unique subset of current profile membership.
    Packs(Vec<PackId>),
}

/// Change enabled state for a current profile member or explicit subset.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollectionProfileEnablementRequest {
    /// Profile name, or `None` to select the latest configured default.
    pub profile: Option<String>,
    /// Current members whose enabled state should change.
    pub selection: CollectionPolicySelection,
    /// `true` enables; `false` disables.
    pub enabled: bool,
}

/// Versioned report for an atomic partial profile-policy change.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CollectionProfilePolicyReport {
    /// Report schema version.
    pub version: u32,
    /// Whether the operation completed.
    pub ok: bool,
    /// `enable`, `disable`, `set_default`, or `clear_default`.
    pub operation: String,
    /// `committed` or `unchanged`.
    pub outcome: String,
    /// Native configuration file path.
    pub path: NativePathReport,
    /// Target profile, absent for clear-default.
    pub profile: Option<String>,
    /// Target profile's native root, absent for clear-default.
    pub root: Option<NativePathReport>,
    /// Target profile's ordered membership.
    pub packs: Vec<PackId>,
    /// Target profile's ordered disabled IDs.
    pub disabled: Vec<PackId>,
    /// Only IDs whose enablement bit changed, in membership order.
    pub affected_pack_ids: Vec<PackId>,
    /// Resulting default profile.
    pub default_profile: Option<String>,
    /// Prior configuration revision.
    pub prior_revision: u64,
    /// Resulting configuration revision.
    pub revision: u64,
    /// Prior exact file hash.
    pub prior_sha256: String,
    /// Resulting exact file hash.
    pub sha256: String,
    /// Latest target catalogue revision, absent for clear-default.
    pub observed_catalogue_revision: Option<u64>,
    /// Latest target availability observation, absent for clear-default.
    pub observed_availability: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WriterPhase {
    StageCreated,
    BeforeReplace,
    AfterReplace,
}

/// Typed failure while reading or atomically replacing a profile file.
#[derive(Debug)]
#[non_exhaustive]
pub enum CollectionProfileError {
    /// The process current directory could not be resolved for a relative path.
    CurrentDirectory {
        /// Original relative configuration path.
        path: PathBuf,
        /// Original current-directory lookup error.
        source: io::Error,
    },
    /// The configuration file itself was absent when read.
    MissingFile {
        /// Absolute configuration path that was read.
        path: PathBuf,
        /// Original file-read error.
        source: io::Error,
    },
    /// Native file read or write failed.
    Io {
        /// Native path involved in the failed operation.
        path: PathBuf,
        /// Original filesystem error.
        source: io::Error,
    },
    /// Configuration was not UTF-8.
    Utf8 {
        /// Native path that contained invalid bytes.
        path: PathBuf,
        /// Original UTF-8 conversion error.
        source: std::string::FromUtf8Error,
    },
    /// Strict TOML decode or typed build failed.
    Invalid {
        /// Native configuration path being read or written.
        path: PathBuf,
        /// Stable policy or parser diagnostic.
        detail: String,
    },
    /// The underlying configuration builder rejected the TOML document.
    ConfigDecode {
        /// Native config path being read.
        path: PathBuf,
        /// Original strict typed-configuration error.
        source: confyg::ConfigError,
    },
    /// TOML serialization failed before publication.
    TomlSerialize {
        /// Native config path being written.
        path: PathBuf,
        /// Original TOML serializer error.
        source: toml::ser::Error,
    },
    /// The referenced collection catalogue could not be inspected.
    Catalogue {
        /// Original typed catalogue operation error.
        source: Box<crate::CollectionError>,
    },
    /// A supported configuration version is required.
    UnsupportedVersion(u32),
    /// The explicit destination is already held by another writer.
    Busy(PathBuf),
    /// No default profile is configured for a default-profile lookup.
    NoDefaultProfile {
        /// Native config path inspected for a default.
        path: PathBuf,
    },
    /// The explicitly requested profile name is not present in the file.
    UnknownProfile {
        /// Native config path inspected for the profile.
        path: PathBuf,
        /// Requested profile name.
        name: String,
    },
    /// An explicit pack is not a member of the selected profile.
    NotProfileMember {
        /// Native config path inspected for the profile.
        path: PathBuf,
        /// Selected profile name.
        profile: String,
        /// Explicit pack outside current membership.
        pack: PackId,
    },
    /// Replacement may have happened but durable status could not be proven.
    Indeterminate {
        /// Native configuration path whose publication durability is uncertain.
        path: PathBuf,
        /// Additional operation-stage context.
        detail: String,
        /// Prior revision, if one was present.
        prior_revision: Option<u64>,
        /// Proposed revision already written to the visible path.
        proposed_revision: u64,
        /// Exact prior file hash, if one existed.
        prior_sha256: Option<Box<String>>,
        /// Exact proposed file hash.
        proposed_sha256: Box<String>,
        /// Hash observed at the path after the uncertain failure, if readable.
        observed_sha256: Option<Box<String>>,
        /// Revision observed in the file after the uncertain failure, if it
        /// remained readable and valid.
        observed_revision: Option<u64>,
        /// Original directory-sync error.
        source: io::Error,
    },
}

impl fmt::Display for CollectionProfileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CurrentDirectory { path, source } => write!(
                f,
                "resolve current directory for relative profile path {}: {source}",
                path.display()
            ),
            Self::MissingFile { path, source } => {
                write!(f, "profile file {}: {source}", path.display())
            }
            Self::Io { path, source } => write!(f, "profile file {}: {source}", path.display()),
            Self::Utf8 { path, source } => {
                write!(f, "profile file {} is not UTF-8: {source}", path.display())
            }
            Self::Invalid { path, detail } => {
                write!(f, "invalid profile file {}: {detail}", path.display())
            }
            Self::ConfigDecode { path, source } => {
                write!(f, "invalid profile file {}: {source}", path.display())
            }
            Self::TomlSerialize { path, source } => {
                write!(f, "serialize profile file {}: {source}", path.display())
            }
            Self::UnsupportedVersion(version) => {
                write!(f, "unsupported collection profile version {version}")
            }
            Self::Busy(path) => write!(f, "profile writer is busy at {}", path.display()),
            Self::NoDefaultProfile { path } => {
                write!(f, "no default profile is configured in {}", path.display())
            }
            Self::UnknownProfile { path, name } => {
                write!(f, "unknown profile {name:?} in {}", path.display())
            }
            Self::NotProfileMember {
                path,
                profile,
                pack,
            } => write!(
                f,
                "pack {pack} is not a member of profile {profile:?} in {}",
                path.display()
            ),
            Self::Indeterminate {
                path,
                detail,
                prior_revision,
                proposed_revision,
                prior_sha256,
                proposed_sha256,
                observed_sha256,
                observed_revision,
                source,
            } => write!(
                f,
                "profile write outcome is indeterminate at {}: {detail}; prior_revision={prior_revision:?}, proposed_revision={proposed_revision}, prior_sha256={prior_sha256:?}, proposed_sha256={proposed_sha256}, observed_sha256={observed_sha256:?}, observed_revision={observed_revision:?}: {source}",
                path.display()
            ),
            Self::Catalogue { source } => {
                write!(f, "profile catalogue validation failed: {source}")
            }
        }
    }
}

impl Error for CollectionProfileError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::CurrentDirectory { source, .. } | Self::MissingFile { source, .. } => {
                Some(source)
            }
            Self::Io { source, .. } => Some(source),
            Self::Utf8 { source, .. } => Some(source),
            Self::Indeterminate { source, .. } => Some(source),
            Self::Catalogue { source } => Some(source.as_ref()),
            Self::ConfigDecode { source, .. } => Some(source),
            Self::TomlSerialize { source, .. } => Some(source),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ConfigWire {
    version: u32,
    revision: u64,
    #[serde(default)]
    default_profile: Option<String>,
    profiles: BTreeMap<String, ProfileWire>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProfileWire {
    #[serde(default)]
    root: Option<String>,
    #[serde(default)]
    root_native: Option<NativeRootWire>,
    packs: Vec<String>,
    #[serde(default)]
    disabled: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct NativeRootWire {
    encoding: String,
    hex: String,
}

/// An immutable snapshot of one explicit version-1 profile file.
#[derive(Debug, Clone)]
pub struct CollectionProfiles {
    path: PathBuf,
    bytes: Vec<u8>,
    wire: ConfigWire,
    profiles: BTreeMap<String, CollectionProfile>,
    sha256: String,
}

impl CollectionProfiles {
    /// Reads exactly `path`; does not search environment, defaults or adjacent files.
    ///
    /// # Errors
    /// Returns a typed IO, UTF-8, strict TOML, unsupported-version, or policy error.
    pub fn read(path: impl AsRef<Path>) -> Result<Self, CollectionProfileError> {
        let path = absolute(path.as_ref())?;
        let bytes = fs::read(&path).map_err(|source| {
            if source.kind() == io::ErrorKind::NotFound {
                CollectionProfileError::MissingFile {
                    path: path.clone(),
                    source,
                }
            } else {
                CollectionProfileError::Io {
                    path: path.clone(),
                    source,
                }
            }
        })?;
        Self::from_bytes(path, bytes)
    }

    /// Selects one explicit or configured default profile.
    ///
    /// # Errors
    /// Returns an error when no default exists or the requested profile is unknown.
    pub fn profile(&self, name: Option<&str>) -> Result<CollectionProfile, CollectionProfileError> {
        let name = name
            .or(self.wire.default_profile.as_deref())
            .ok_or_else(|| CollectionProfileError::NoDefaultProfile {
                path: self.path.clone(),
            })?;
        self.profiles
            .get(name)
            .cloned()
            .ok_or_else(|| CollectionProfileError::UnknownProfile {
                path: self.path.clone(),
                name: name.to_owned(),
            })
    }

    /// Returns the immutable exact-file report.
    #[must_use]
    pub fn report(&self) -> CollectionProfilesReport {
        CollectionProfilesReport {
            version: 1,
            path: native_path_report(&self.path),
            config_version: self.wire.version,
            revision: self.wire.revision,
            sha256: self.sha256.clone(),
            default_profile: self.wire.default_profile.clone(),
            profiles: self
                .profiles
                .values()
                .map(|profile| CollectionProfileSummary {
                    name: profile.name.clone(),
                    root: native_path_report(&profile.root),
                    packs: profile.packs.clone(),
                    disabled: profile.disabled.clone(),
                })
                .collect(),
        }
    }

    /// Returns the captured explicit path.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }
    /// Returns the captured monotonic revision.
    #[must_use]
    pub fn revision(&self) -> u64 {
        self.wire.revision
    }
    /// Returns the exact file SHA-256.
    #[must_use]
    pub fn sha256(&self) -> &str {
        &self.sha256
    }

    /// Returns the selected profile's captured file identity.
    pub fn identity(&self, profile: &CollectionProfile) -> CollectionProfileIdentity {
        CollectionProfileIdentity {
            name: profile.name.clone(),
            config_path: native_path_report(&self.path),
            version: self.wire.version,
            revision: self.wire.revision,
            sha256: self.sha256.clone(),
        }
    }

    /// Re-reads the exact path and rejects any byte change since this snapshot.
    ///
    /// # Errors
    /// Returns IO or invalidation when the original file cannot be read or changed.
    pub fn verify_unchanged(&self) -> Result<(), CollectionProfileError> {
        let bytes = fs::read(&self.path).map_err(|source| CollectionProfileError::Io {
            path: self.path.clone(),
            source,
        })?;
        if bytes == self.bytes {
            Ok(())
        } else {
            Err(invalid(
                &self.path,
                "configuration bytes changed since this snapshot",
            ))
        }
    }

    /// Atomically writes one profile in an explicit configuration file.
    ///
    /// A semantic no-op preserves the existing file bytes, comments, formatting,
    /// and revision. A real change serializes the parsed TOML again and may not
    /// preserve comments or formatting. The sibling lock serializes cooperating
    /// writers. Before rename, the writer checks that the destination still has
    /// the bytes it read (or remains absent for initial creation); a
    /// non-cooperating editor can still race between that check and rename.
    ///
    /// # Errors
    /// Returns typed policy, IO, busy, or indeterminate publication failures.
    pub fn configure(
        path: impl AsRef<Path>,
        request: CollectionProfileWriteRequest,
    ) -> Result<CollectionProfileWriteReport, CollectionProfileError> {
        Self::configure_with_policy(path, request, DisabledPolicy::Explicit)
    }

    /// Atomically changes profile membership while preserving its latest disabled set.
    ///
    /// The prior disabled IDs are selected under the same exclusive writer lock
    /// used to publish the new profile. New profiles begin with an empty set.
    ///
    /// # Errors
    /// Returns typed policy, IO, busy, catalogue, or indeterminate publication failures.
    pub fn configure_membership(
        path: impl AsRef<Path>,
        request: CollectionProfileMembershipWriteRequest,
    ) -> Result<CollectionProfileWriteReport, CollectionProfileError> {
        Self::configure_with_policy(
            path,
            CollectionProfileWriteRequest {
                name: request.name,
                root: request.root,
                packs: request.packs,
                disabled: Vec::new(),
                make_default: request.make_default,
            },
            DisabledPolicy::PreserveLatest,
        )
    }

    /// Changes the enabled state of current profile members without replacing
    /// unrelated profile policy dimensions.
    pub fn set_enabled(
        path: impl AsRef<Path>,
        request: CollectionProfileEnablementRequest,
    ) -> Result<CollectionProfilePolicyReport, CollectionProfileError> {
        policy::set_enabled(path, request)
    }

    /// Sets or explicitly clears the default profile in the latest config.
    pub fn set_default(
        path: impl AsRef<Path>,
        profile: Option<&str>,
    ) -> Result<CollectionProfilePolicyReport, CollectionProfileError> {
        policy::set_default(path, profile)
    }

    fn configure_with_policy(
        path: impl AsRef<Path>,
        request: CollectionProfileWriteRequest,
        disabled_policy: DisabledPolicy,
    ) -> Result<CollectionProfileWriteReport, CollectionProfileError> {
        Self::configure_with_policy_hook(path, request, disabled_policy, || {})
    }

    fn configure_with_policy_hook<F>(
        path: impl AsRef<Path>,
        request: CollectionProfileWriteRequest,
        disabled_policy: DisabledPolicy,
        before_lock: F,
    ) -> Result<CollectionProfileWriteReport, CollectionProfileError>
    where
        F: FnOnce(),
    {
        Self::configure_with_hooks(path, request, disabled_policy, before_lock, |_, _| Ok(()))
    }

    fn configure_with_hooks<F, H>(
        path: impl AsRef<Path>,
        request: CollectionProfileWriteRequest,
        disabled_policy: DisabledPolicy,
        before_lock: F,
        mut phase_hook: H,
    ) -> Result<CollectionProfileWriteReport, CollectionProfileError>
    where
        F: FnOnce(),
        H: FnMut(WriterPhase, &Path) -> io::Result<()>,
    {
        let supplied_path = path.as_ref();
        let needs_cwd = !supplied_path.is_absolute() || !request.root.is_absolute();
        let cwd = if needs_cwd {
            Some(std::env::current_dir().map_err(|source| {
                CollectionProfileError::CurrentDirectory {
                    path: if supplied_path.is_absolute() {
                        request.root.clone()
                    } else {
                        supplied_path.to_path_buf()
                    },
                    source,
                }
            })?)
        } else {
            None
        };
        let path = if supplied_path.is_absolute() {
            supplied_path.to_path_buf()
        } else {
            cwd.as_ref().expect("cwd captured").join(supplied_path)
        };
        let parent = path.parent().unwrap_or_else(|| Path::new("."));
        validate_name(&request.name, &path)?;
        validate_ids(&request.packs, "packs", &path)?;
        if matches!(disabled_policy, DisabledPolicy::Explicit) {
            validate_ids(&request.disabled, "disabled", &path)?;
        }
        // Resolve both relative arguments against the same frozen cwd. The config
        // read path's own relative-root semantics remain config-parent-relative.
        let root = if request.root.is_absolute() {
            request.root.clone()
        } else {
            cwd.as_ref().expect("cwd captured").join(&request.root)
        };
        // Validate selected registrations without requiring a generation or hashing release bytes.
        let catalogue = CollectionCatalogue::open(&root).map_err(|source| {
            CollectionProfileError::Catalogue {
                source: Box::new(source),
            }
        })?;
        let observed_revision =
            catalogue
                .local_revision()
                .map_err(|source| CollectionProfileError::Catalogue {
                    source: Box::new(source),
                })?;
        let listed = catalogue
            .list()
            .map_err(|source| CollectionProfileError::Catalogue {
                source: Box::new(source),
            })?;
        let known: BTreeMap<_, _> = listed
            .packs
            .iter()
            .map(|pack| (pack.pack_id.as_str(), pack))
            .collect();
        for id in request.packs.iter().chain(request.disabled.iter()) {
            if !known.contains_key(id.as_str()) {
                return Err(invalid(
                    &path,
                    format!("pack {id} is not registered in the configured catalogue"),
                ));
            }
        }
        let availability = if listed.generation.is_some() {
            "generation_observed"
        } else {
            "registered_only"
        };
        create_private_parents(parent).map_err(|source| CollectionProfileError::Io {
            path: parent.to_path_buf(),
            source,
        })?;
        before_lock();
        let lock_path = sibling(&path, ".lock");
        let lock_token = format!("vlajmaji collection profile writer {}\n", unique_nonce());
        let mut lock_options = OpenOptions::new();
        lock_options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            lock_options.mode(0o600);
        }
        let mut lock_file = lock_options.open(&lock_path).map_err(|source| {
            if source.kind() == io::ErrorKind::AlreadyExists {
                CollectionProfileError::Busy(lock_path.clone())
            } else {
                CollectionProfileError::Io {
                    path: lock_path.clone(),
                    source,
                }
            }
        })?;
        let lock = OwnedSibling::new(lock_path.clone(), &lock_file).map_err(|source| {
            CollectionProfileError::Io {
                path: lock_path.clone(),
                source,
            }
        })?;
        lock_file
            .write_all(lock_token.as_bytes())
            .and_then(|_| lock_file.sync_all())
            .map_err(|source| CollectionProfileError::Io {
                path: lock_path.clone(),
                source,
            })?;
        drop(lock_file);

        let prior = read_optional_config(&path)?;
        let (mut wire, prior_hash, prior_revision) = if let Some(prior) = prior.as_ref() {
            (
                prior.wire.clone(),
                Some(prior.sha256.clone()),
                Some(prior.wire.revision),
            )
        } else {
            (
                ConfigWire {
                    version: CONFIG_VERSION,
                    revision: 0,
                    default_profile: None,
                    profiles: BTreeMap::new(),
                },
                None,
                None,
            )
        };
        let disabled = policy::resolve_disabled(
            disabled_policy,
            &request.disabled,
            prior
                .as_ref()
                .and_then(|snapshot| snapshot.wire.profiles.get(&request.name))
                .map(|profile| profile.disabled.as_slice()),
            &path,
        )?;
        validate_ids(&disabled, "disabled", &path)?;
        for id in &disabled {
            if !known.contains_key(id.as_str()) {
                return Err(invalid(
                    &path,
                    format!(
                        "preserved disabled pack {id} is not registered in the configured catalogue"
                    ),
                ));
            }
        }
        if wire.revision > i64::MAX as u64 {
            return Err(invalid(&path, "revision exceeds TOML signed integer range"));
        }
        let profile_wire = profile_to_wire(&path, parent, &root, &request.packs, &disabled)?;
        let unchanged_profile = prior
            .as_ref()
            .and_then(|snapshot| snapshot.profiles.get(&request.name))
            .is_some_and(|existing| {
                existing.root == root
                    && existing.packs == request.packs
                    && existing.disabled == disabled
            })
            && (!request.make_default || wire.default_profile.as_deref() == Some(&request.name));
        if unchanged_profile {
            let prior = prior.expect("existing profile implies existing configuration");
            drop(lock);
            return Ok(CollectionProfileWriteReport {
                version: 1,
                ok: true,
                operation: "configure".into(),
                outcome: "unchanged".into(),
                path: native_path_report(&path),
                profile: request.name,
                default_profile: wire.default_profile,
                prior_revision,
                revision: prior.wire.revision,
                prior_sha256: prior_hash,
                sha256: prior.sha256,
                observed_catalogue_revision: observed_revision,
                observed_availability: availability.into(),
            });
        }
        wire.profiles.insert(request.name.clone(), profile_wire);
        if request.make_default {
            wire.default_profile = Some(request.name.clone());
        }
        let publication =
            publish_config_wire(&path, parent, prior.as_ref(), &mut wire, |phase, stage| {
                phase_hook(phase, stage)
            })?;
        drop(lock);
        Ok(CollectionProfileWriteReport {
            version: 1,
            ok: true,
            operation: "configure".into(),
            outcome: "committed".into(),
            path: native_path_report(&path),
            profile: request.name,
            default_profile: wire.default_profile,
            prior_revision: publication.prior_revision,
            revision: publication.revision,
            prior_sha256: publication.prior_sha256,
            sha256: publication.sha256,
            observed_catalogue_revision: observed_revision,
            observed_availability: availability.into(),
        })
    }

    fn from_bytes(path: PathBuf, bytes: Vec<u8>) -> Result<Self, CollectionProfileError> {
        let text =
            String::from_utf8(bytes.clone()).map_err(|source| CollectionProfileError::Utf8 {
                path: path.clone(),
                source,
            })?;
        let mut builder =
            confyg::Confygery::new().map_err(|source| CollectionProfileError::ConfigDecode {
                path: path.clone(),
                source,
            })?;
        builder
            .add_str(&text)
            .map_err(|source| CollectionProfileError::ConfigDecode {
                path: path.clone(),
                source,
            })?;
        let wire: ConfigWire =
            builder
                .build()
                .map_err(|source| CollectionProfileError::ConfigDecode {
                    path: path.clone(),
                    source,
                })?;
        validate_wire(&path, &wire)?;
        let base = path.parent().unwrap_or_else(|| Path::new("."));
        let mut profiles = BTreeMap::new();
        for (name, profile) in &wire.profiles {
            validate_name(name, &path)?;
            let root = parse_root(&path, base, profile)?;
            let packs = parse_ids(&path, &profile.packs, "packs")?;
            let disabled = parse_ids(&path, &profile.disabled, "disabled")?;
            validate_ids(&packs, "packs", &path)?;
            validate_ids(&disabled, "disabled", &path)?;
            profiles.insert(
                name.clone(),
                CollectionProfile {
                    name: name.clone(),
                    root,
                    packs,
                    disabled,
                },
            );
        }
        let sha256 = hex(&Sha256::digest(&bytes));
        Ok(Self {
            path,
            bytes,
            wire,
            profiles,
            sha256,
        })
    }
}

fn validate_wire(path: &Path, wire: &ConfigWire) -> Result<(), CollectionProfileError> {
    if wire.version != CONFIG_VERSION {
        return Err(CollectionProfileError::UnsupportedVersion(wire.version));
    }
    if wire.revision > i64::MAX as u64 {
        return Err(invalid(path, "revision exceeds TOML signed integer range"));
    }
    if wire
        .default_profile
        .as_ref()
        .is_some_and(|name| !wire.profiles.contains_key(name))
    {
        return Err(invalid(
            path,
            "default_profile does not name a declared profile",
        ));
    }
    for name in wire.profiles.keys() {
        validate_name(name, path)?;
    }
    Ok(())
}

fn validate_name(name: &str, path: &Path) -> Result<(), CollectionProfileError> {
    let mut bytes = name.bytes();
    let first = bytes.next();
    if name.is_empty()
        || name.len() > 64
        || !first.is_some_and(|byte| byte.is_ascii_alphanumeric())
        || !bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-')
    {
        return Err(invalid(path, format!("invalid profile name {name:?}")));
    }
    Ok(())
}

fn validate_ids(ids: &[PackId], field: &str, path: &Path) -> Result<(), CollectionProfileError> {
    let mut seen = std::collections::BTreeSet::new();
    for id in ids {
        if !seen.insert(*id) {
            return Err(invalid(path, format!("duplicate pack ID {id} in {field}")));
        }
    }
    Ok(())
}

fn parse_ids(
    path: &Path,
    values: &[String],
    field: &str,
) -> Result<Vec<PackId>, CollectionProfileError> {
    values
        .iter()
        .map(|value| {
            parse_pack(value)
                .ok_or_else(|| invalid(path, format!("unknown pack ID {value:?} in {field}")))
        })
        .collect()
}

fn parse_pack(value: &str) -> Option<PackId> {
    match value {
        "lensisku-jbotci" => Some(PackId::LensiskuJbotci),
        "llg" => Some(PackId::Llg),
        "la-lojban" => Some(PackId::LaLojban),
        "sutysisku" => Some(PackId::Sutysisku),
        _ => None,
    }
}

fn parse_root(
    path: &Path,
    base: &Path,
    wire: &ProfileWire,
) -> Result<PathBuf, CollectionProfileError> {
    match (&wire.root, &wire.root_native) {
        (Some(text), None) if !text.is_empty() => {
            let root = PathBuf::from(text);
            if root.as_os_str().as_encoded_bytes().contains(&0) {
                return Err(invalid(path, "profile root contains NUL"));
            }
            let root = if root.is_absolute() {
                root
            } else {
                base.join(root)
            };
            if root.is_absolute() {
                Ok(root)
            } else {
                Err(invalid(path, "profile root cannot be made absolute"))
            }
        }
        (None, Some(native)) => decode_native(path, native),
        (Some(_), Some(_)) => Err(invalid(
            path,
            "profile must provide exactly one of root or root_native",
        )),
        _ => Err(invalid(
            path,
            "profile must provide exactly one nonempty root or root_native",
        )),
    }
}

fn decode_native(path: &Path, value: &NativeRootWire) -> Result<PathBuf, CollectionProfileError> {
    if value.encoding != "unix-bytes" || !cfg!(unix) {
        return Err(invalid(
            path,
            "root_native encoding is unsupported on this host",
        ));
    }
    let bytes = decode_hex(&value.hex).ok_or_else(|| {
        invalid(
            path,
            "root_native hex must be nonempty lowercase hexadecimal",
        )
    })?;
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStringExt;
        let root = PathBuf::from(std::ffi::OsString::from_vec(bytes));
        if !root.is_absolute()
            || root.as_os_str().is_empty()
            || root.as_os_str().as_encoded_bytes().contains(&0)
        {
            return Err(invalid(
                path,
                "root_native must decode to an absolute NUL-free path",
            ));
        }
        Ok(root)
    }
    #[cfg(not(unix))]
    {
        let _ = bytes;
        Err(invalid(
            path,
            "root_native encoding is unsupported on this host",
        ))
    }
}

fn profile_to_wire(
    path: &Path,
    config_parent: &Path,
    root: &Path,
    packs: &[PackId],
    disabled: &[PackId],
) -> Result<ProfileWire, CollectionProfileError> {
    if root.as_os_str().as_encoded_bytes().contains(&0) {
        return Err(invalid(path, "profile root contains NUL"));
    }
    let relative = root
        .strip_prefix(config_parent)
        .ok()
        .map(|relative| relative.to_string_lossy().into_owned());
    let _ = relative; // Keep caller roots absolute in the library wire.
    if let Some(text) = root.to_str() {
        Ok(ProfileWire {
            root: Some(text.to_owned()),
            root_native: None,
            packs: packs.iter().map(|id| id.as_str().to_owned()).collect(),
            disabled: disabled.iter().map(|id| id.as_str().to_owned()).collect(),
        })
    } else {
        #[cfg(unix)]
        {
            use std::os::unix::ffi::OsStrExt;
            Ok(ProfileWire {
                root: None,
                root_native: Some(NativeRootWire {
                    encoding: "unix-bytes".into(),
                    hex: encode_hex(root.as_os_str().as_bytes()),
                }),
                packs: packs.iter().map(|id| id.as_str().to_owned()).collect(),
                disabled: disabled.iter().map(|id| id.as_str().to_owned()).collect(),
            })
        }
        #[cfg(not(unix))]
        {
            Err(invalid(
                path,
                "native non-UTF-8 roots are unsupported on this host",
            ))
        }
    }
}

fn absolute(path: &Path) -> Result<PathBuf, CollectionProfileError> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        std::env::current_dir()
            .map(|cwd| cwd.join(path))
            .map_err(|source| CollectionProfileError::CurrentDirectory {
                path: path.to_path_buf(),
                source,
            })
    }
}
fn sibling(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(suffix);
    path.with_file_name(name)
}
fn invalid(path: &Path, detail: impl Into<String>) -> CollectionProfileError {
    CollectionProfileError::Invalid {
        path: path.to_path_buf(),
        detail: detail.into(),
    }
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
fn encode_hex(bytes: &[u8]) -> String {
    hex(bytes)
}
fn decode_hex(value: &str) -> Option<Vec<u8>> {
    if value.is_empty()
        || value
            .bytes()
            .any(|byte| !byte.is_ascii_hexdigit() || byte.is_ascii_uppercase())
        || !value.len().is_multiple_of(2)
    {
        return None;
    }
    (0..value.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&value[i..i + 2], 16).ok())
        .collect()
}

#[cfg(unix)]
fn existing_mode(path: &Path) -> Option<u32> {
    use std::os::unix::fs::PermissionsExt;
    fs::metadata(path)
        .ok()
        .map(|meta| meta.permissions().mode() & 0o777)
}
#[cfg(not(unix))]
fn existing_mode(_path: &Path) -> Option<u32> {
    None
}

struct OwnedSibling {
    path: PathBuf,
    identity: FileIdentity,
    active: bool,
}

struct PublishedConfig {
    revision: u64,
    prior_revision: Option<u64>,
    prior_sha256: Option<String>,
    sha256: String,
}

fn publish_config_wire<H>(
    path: &Path,
    parent: &Path,
    prior: Option<&CollectionProfiles>,
    wire: &mut ConfigWire,
    mut phase_hook: H,
) -> Result<PublishedConfig, CollectionProfileError>
where
    H: FnMut(WriterPhase, &Path) -> io::Result<()>,
{
    let prior_revision = prior.map(|snapshot| snapshot.wire.revision);
    let prior_sha256 = prior.map(|snapshot| snapshot.sha256.clone());
    let next_revision = wire
        .revision
        .checked_add(1)
        .filter(|value| *value <= i64::MAX as u64)
        .ok_or_else(|| invalid(path, "profile revision overflow"))?;
    wire.revision = next_revision;
    validate_wire(path, wire)?;
    let text =
        toml::to_string_pretty(wire).map_err(|source| CollectionProfileError::TomlSerialize {
            path: path.to_path_buf(),
            source,
        })?;
    let bytes = text.into_bytes();
    let new_hash = hex(&Sha256::digest(&bytes));
    let now = read_optional_config(path)?;
    let same_as_prior = match (prior, now.as_ref()) {
        (Some(snapshot), Some(current)) => snapshot.bytes == current.bytes,
        (None, None) => true,
        _ => false,
    };
    if !same_as_prior {
        return Err(invalid(
            path,
            "configuration changed while writer lock was held",
        ));
    }
    let stage = sibling(
        path,
        &format!(".stage-{}-{}", std::process::id(), unique_nonce()),
    );
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(if prior.is_some() {
            existing_mode(path).unwrap_or(0o600)
        } else {
            0o600
        });
    }
    let mut file = options
        .open(&stage)
        .map_err(|source| CollectionProfileError::Io {
            path: stage.clone(),
            source,
        })?;
    let stage_guard =
        OwnedSibling::new(stage.clone(), &file).map_err(|source| CollectionProfileError::Io {
            path: stage.clone(),
            source,
        })?;
    file.write_all(&bytes)
        .and_then(|_| file.sync_all())
        .map_err(|source| CollectionProfileError::Io {
            path: stage.clone(),
            source,
        })?;
    drop(file);
    phase_hook(WriterPhase::StageCreated, &stage).map_err(|source| CollectionProfileError::Io {
        path: stage.clone(),
        source,
    })?;
    if prior.is_some() {
        preserve_existing_mode(path, &stage).map_err(|source| CollectionProfileError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        fs::File::open(&stage)
            .and_then(|file| file.sync_all())
            .map_err(|source| CollectionProfileError::Io {
                path: stage.clone(),
                source,
            })?;
    }
    phase_hook(WriterPhase::BeforeReplace, &stage).map_err(|source| {
        CollectionProfileError::Io {
            path: stage.clone(),
            source,
        }
    })?;
    let current = read_optional_config(path)?;
    let unchanged = match (prior, current.as_ref()) {
        (Some(before), Some(now)) => before.bytes == now.bytes,
        (None, None) => true,
        _ => false,
    };
    if !unchanged {
        return Err(invalid(
            path,
            "configuration changed while staged replacement was being written",
        ));
    }
    fs::rename(&stage, path).map_err(|source| CollectionProfileError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    stage_guard.disarm();
    if let Err(source) =
        phase_hook(WriterPhase::AfterReplace, path).and_then(|()| sync_parent(parent))
    {
        let observed = fs::read(path).ok();
        let observed_sha256 = observed.as_ref().map(|bytes| hex(&Sha256::digest(bytes)));
        let observed_revision = observed
            .and_then(|bytes| CollectionProfiles::from_bytes(path.to_path_buf(), bytes).ok())
            .map(|profiles| profiles.wire.revision);
        return Err(CollectionProfileError::Indeterminate {
            path: path.to_path_buf(),
            detail: "directory entry sync failed after replacement".into(),
            prior_revision,
            proposed_revision: next_revision,
            prior_sha256: prior_sha256.clone().map(Box::new),
            proposed_sha256: Box::new(new_hash),
            observed_sha256: observed_sha256.map(Box::new),
            observed_revision,
            source,
        });
    }
    Ok(PublishedConfig {
        revision: next_revision,
        prior_revision,
        prior_sha256,
        sha256: new_hash,
    })
}
#[derive(Clone, Copy)]
struct FileIdentity {
    #[cfg(unix)]
    dev: u64,
    #[cfg(unix)]
    ino: u64,
    #[cfg(not(unix))]
    len: u64,
}
impl OwnedSibling {
    fn new(path: PathBuf, file: &fs::File) -> io::Result<Self> {
        let metadata = file.metadata()?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            Ok(Self {
                path,
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
                identity: FileIdentity {
                    len: metadata.len(),
                },
                active: true,
            })
        }
    }
    fn disarm(mut self) {
        self.active = false;
    }
}
impl Drop for OwnedSibling {
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

fn read_optional_config(path: &Path) -> Result<Option<CollectionProfiles>, CollectionProfileError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_file() => Err(invalid(
            path,
            "configuration destination must be a regular non-symlink file",
        )),
        Ok(_) => {
            let bytes = fs::read(path).map_err(|source| CollectionProfileError::Io {
                path: path.to_path_buf(),
                source,
            })?;
            CollectionProfiles::from_bytes(path.to_path_buf(), bytes).map(Some)
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(source) => Err(CollectionProfileError::Io {
            path: path.to_path_buf(),
            source,
        }),
    }
}

fn create_private_parents(parent: &Path) -> io::Result<()> {
    let mut missing = Vec::new();
    let mut current = parent;
    loop {
        match fs::symlink_metadata(current) {
            Ok(metadata) => {
                let followed = if metadata.file_type().is_symlink() {
                    fs::metadata(current)?
                } else {
                    metadata
                };
                if !followed.is_dir() {
                    return Err(io::Error::new(
                        io::ErrorKind::NotADirectory,
                        "configuration parent is not a real directory",
                    ));
                }
                break;
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                missing.push(current.to_path_buf());
                current = current.parent().ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::NotFound,
                        "no existing configuration ancestor",
                    )
                })?;
            }
            Err(error) => return Err(error),
        }
    }
    for dir in missing.iter().rev() {
        let mut builder = fs::DirBuilder::new();
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            builder.mode(0o700);
        }
        match builder.create(dir) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                let metadata = fs::symlink_metadata(dir)?;
                let followed = if metadata.file_type().is_symlink() {
                    fs::metadata(dir)?
                } else {
                    metadata
                };
                if !followed.is_dir() {
                    return Err(io::Error::new(
                        io::ErrorKind::NotADirectory,
                        "configuration parent raced with a non-directory",
                    ));
                }
            }
            Err(error) => return Err(error),
        }
        if let Some(ancestor) = dir.parent() {
            sync_parent(ancestor)?;
        }
    }
    Ok(())
}

fn preserve_existing_mode(existing: &Path, staged: &Path) -> io::Result<()> {
    let metadata = fs::symlink_metadata(existing)?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "configuration destination changed to a non-regular file",
        ));
    }
    fs::set_permissions(staged, metadata.permissions())
}

fn unique_nonce() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
}
fn sync_parent(parent: &Path) -> io::Result<()> {
    fs::File::open(parent)?.sync_all()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CollectionInput, CollectionRequest, CollectionSelection};
    use std::{
        borrow::Cow,
        sync::mpsc,
        time::{SystemTime, UNIX_EPOCH},
    };
    use vlajmaji_lexicon::{Dictionary, PackPayloadKind, seal_release};

    #[test]
    fn test_configure_membership_reads_latest_policy_after_lock() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "vlajmaji-s03-membership-race-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir(&root).unwrap();
        let catalogue_root = root.join("catalogue");
        let dictionary = Dictionary::from_document(Default::default()).unwrap();
        let sealed = seal_release(PackId::Llg, PackPayloadKind::Synthetic, &dictionary).unwrap();
        CollectionCatalogue::register(
            &catalogue_root,
            CollectionRequest::new(
                vec![CollectionInput::SealedBytes {
                    pack_id: PackId::Llg,
                    release: Cow::Borrowed(&sealed.release),
                    seal: Cow::Borrowed(&sealed.seal),
                }],
                CollectionSelection::Packs(vec![PackId::Llg]),
                true,
            )
            .unwrap(),
        )
        .unwrap();
        let config = root.join("config.toml");
        fs::write(
            &config,
            format!(
                "version=1\nrevision=0\n[profiles.research]\nroot={:?}\npacks=[]\ndisabled=[]\n",
                catalogue_root.to_string_lossy()
            ),
        )
        .unwrap();

        let (reached_tx, reached_rx) = mpsc::channel();
        let (continue_tx, continue_rx) = mpsc::channel();
        let worker_config = config.clone();
        let worker_root = catalogue_root.clone();
        let worker = std::thread::spawn(move || {
            CollectionProfiles::configure_with_policy_hook(
                &worker_config,
                CollectionProfileWriteRequest {
                    name: "research".into(),
                    root: worker_root,
                    packs: vec![PackId::Llg],
                    disabled: Vec::new(),
                    make_default: false,
                },
                DisabledPolicy::PreserveLatest,
                || {
                    reached_tx.send(()).unwrap();
                    continue_rx.recv().unwrap();
                },
            )
        });
        reached_rx.recv().unwrap();

        // This explicit writer commits a newer disabled policy after the
        // membership operation's pre-lock observations but before it locks.
        CollectionProfiles::configure(
            &config,
            CollectionProfileWriteRequest {
                name: "research".into(),
                root: catalogue_root,
                packs: Vec::new(),
                disabled: vec![PackId::Llg],
                make_default: false,
            },
        )
        .unwrap();
        continue_tx.send(()).unwrap();
        assert_eq!(worker.join().unwrap().unwrap().outcome, "committed");
        let final_profile = CollectionProfiles::read(&config)
            .unwrap()
            .profile(Some("research"))
            .unwrap();
        assert_eq!(final_profile.disabled, vec![PackId::Llg]);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn test_configure_writer_phases_preserve_precommit_and_report_uncertainty() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "vlajmaji-s03-writer-phases-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir(&root).unwrap();
        let catalogue_root = root.join("catalogue");
        let dictionary = Dictionary::from_document(Default::default()).unwrap();
        let sealed = seal_release(PackId::Llg, PackPayloadKind::Synthetic, &dictionary).unwrap();
        CollectionCatalogue::register(
            &catalogue_root,
            CollectionRequest::new(
                vec![CollectionInput::SealedBytes {
                    pack_id: PackId::Llg,
                    release: Cow::Borrowed(&sealed.release),
                    seal: Cow::Borrowed(&sealed.seal),
                }],
                CollectionSelection::Packs(vec![PackId::Llg]),
                true,
            )
            .unwrap(),
        )
        .unwrap();
        let config = root.join("profiles.toml");
        let original = format!(
            "version=1\nrevision=1\n[profiles.research]\nroot={:?}\npacks=[]\ndisabled=[]\n",
            catalogue_root.to_string_lossy()
        )
        .into_bytes();
        fs::write(&config, &original).unwrap();
        let request = || CollectionProfileWriteRequest {
            name: "research".into(),
            root: catalogue_root.clone(),
            packs: vec![PackId::Llg],
            disabled: Vec::new(),
            make_default: false,
        };

        let before_entries: std::collections::BTreeSet<_> = fs::read_dir(&root)
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect();
        let precommit = CollectionProfiles::configure_with_hooks(
            &config,
            request(),
            DisabledPolicy::Explicit,
            || {},
            |phase, _| {
                if phase == WriterPhase::StageCreated {
                    Err(io::Error::other("injected precommit failure"))
                } else {
                    Ok(())
                }
            },
        )
        .unwrap_err();
        assert!(matches!(precommit, CollectionProfileError::Io { .. }));
        assert_eq!(fs::read(&config).unwrap(), original);
        let after_entries: std::collections::BTreeSet<_> = fs::read_dir(&root)
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect();
        assert_eq!(
            before_entries, after_entries,
            "owned stage must be removed on known precommit failure"
        );

        let external = b"version=1\nrevision=22\nprofiles={}\n";
        let external_edit = CollectionProfiles::configure_with_hooks(
            &config,
            request(),
            DisabledPolicy::Explicit,
            || {},
            |phase, _| {
                if phase == WriterPhase::BeforeReplace {
                    fs::write(&config, external)?;
                    Ok(())
                } else {
                    Ok(())
                }
            },
        )
        .unwrap_err();
        assert!(matches!(
            external_edit,
            CollectionProfileError::Invalid { .. }
        ));
        assert_eq!(fs::read(&config).unwrap(), external);

        let absent = root.join("absent.toml");
        let competitor = b"version=1\nrevision=5\nprofiles={}\n";
        let absence_race = CollectionProfiles::configure_with_hooks(
            &absent,
            request(),
            DisabledPolicy::Explicit,
            || {},
            |phase, _| {
                if phase == WriterPhase::BeforeReplace {
                    fs::write(&absent, competitor)?;
                    Ok(())
                } else {
                    Ok(())
                }
            },
        )
        .unwrap_err();
        assert!(matches!(
            absence_race,
            CollectionProfileError::Invalid { .. }
        ));
        assert_eq!(fs::read(&absent).unwrap(), competitor);

        fs::write(&config, &original).unwrap();
        let uncertain = CollectionProfiles::configure_with_hooks(
            &config,
            request(),
            DisabledPolicy::Explicit,
            || {},
            |phase, _| {
                if phase == WriterPhase::AfterReplace {
                    Err(io::Error::other("injected parent sync failure"))
                } else {
                    Ok(())
                }
            },
        )
        .unwrap_err();
        match uncertain {
            CollectionProfileError::Indeterminate {
                prior_revision,
                proposed_revision,
                observed_revision,
                observed_sha256,
                ..
            } => {
                assert_eq!(prior_revision, Some(1));
                assert_eq!(proposed_revision, 2);
                assert_eq!(observed_revision, Some(2));
                assert_eq!(
                    observed_sha256.as_deref().map(String::as_str),
                    Some(hex(&Sha256::digest(fs::read(&config).unwrap())).as_str())
                );
            }
            other => panic!("expected post-replace uncertainty, got {other}"),
        }
        assert_eq!(CollectionProfiles::read(&config).unwrap().revision(), 2);
        fs::remove_dir_all(root).unwrap();
    }
}
