//! Local, explicit collection registration and installation.
//!
//! A collection catalogue records source-pack ownership separately from user
//! selection. Opening it is a minimum-format check, not a full payload audit.

use std::{
    borrow::Cow,
    error::Error,
    fmt, fs, io,
    path::{Path, PathBuf},
};

use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use vlajmaji_lexicon::{
    Availability, CaptureMetadata, CaptureRef, CodeNotice, CollectionRecord, ComponentRecord,
    Dictionary, DistributorRecord, JbotciImportOptions, LaLojbanImportOptions, LlgImportOptions,
    OccurrenceRef, PackId, PackPayloadKind, PackSeal, Presence, RegistryDocument,
    ReleaseAccounting, ReleaseRecord, SourceEvidence, SourceLineage, SourceRecord,
    SutysiskuImportOptions, TimePrecision, import_jbotci_export, import_la_lojban_export,
    import_llg_lists, import_sutysisku_export, load_sealed_release, seal_release,
};

/// Explicit source material used to register or install one local pack.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum CollectionInput<'a> {
    /// A sealed release and seal read from native paths.
    SealedFiles {
        /// Pack identity expected by the caller.
        pack_id: PackId,
        /// Canonical release path.
        release: PathBuf,
        /// Strict seal path.
        seal: PathBuf,
    },
    /// A sealed release supplied directly by an embedded or in-memory caller.
    SealedBytes {
        /// Pack identity expected by the caller.
        pack_id: PackId,
        /// Canonical release bytes.
        release: Cow<'a, [u8]>,
        /// Strict seal bytes.
        seal: Cow<'a, [u8]>,
    },
    /// Raw Lensisku export and supplier metadata.
    Jbotci {
        /// Caller-declared input scope.
        payload_kind: PackPayloadKind,
        /// Raw export path.
        raw: PathBuf,
        /// Supplier metadata path.
        metadata: PathBuf,
        /// Existing importer pins and revision.
        options: JbotciImportOptions,
    },
    /// Raw LLG list family and notice.
    Llg {
        /// Caller-declared input scope.
        payload_kind: PackPayloadKind,
        /// Gismu-list path.
        gismu: PathBuf,
        /// Cmavo-list path.
        cmavo: PathBuf,
        /// Assigned rafsi-list path.
        rafsi: PathBuf,
        /// Source notice path.
        notice: PathBuf,
        /// Existing importer pins and locator base.
        options: LlgImportOptions,
    },
    /// Raw La-Lojban export.
    LaLojban {
        /// Caller-declared input scope.
        payload_kind: PackPayloadKind,
        /// Raw export path.
        raw: PathBuf,
        /// Existing importer pins and locators.
        options: LaLojbanImportOptions,
    },
    /// Raw Sutysisku export.
    Sutysisku {
        /// Caller-declared input scope.
        payload_kind: PackPayloadKind,
        /// Raw export path.
        raw: PathBuf,
        /// Existing importer pins and locators.
        options: SutysiskuImportOptions,
    },
}

/// Which pack inputs a batch operation will consume.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CollectionSelection {
    /// Require and consume every supported pack input.
    All,
    /// Consume exactly these pack IDs.
    Packs(Vec<PackId>),
}

/// A validated operation request. Construction does not read files.
#[derive(Debug, Clone)]
pub struct CollectionRequest<'a> {
    /// Supplied pack inputs.
    inputs: Vec<CollectionInput<'a>>,
    /// Selection for this operation.
    selection: CollectionSelection,
    /// Whether synthetic inputs are explicitly allowed.
    allow_synthetic: bool,
}

impl<'a> CollectionRequest<'a> {
    /// Creates a request after checking pack uniqueness and selection shape.
    ///
    /// # Errors
    ///
    /// Returns an error when a pack is duplicated or a selected pack is absent.
    pub fn new(
        inputs: Vec<CollectionInput<'a>>,
        selection: CollectionSelection,
        allow_synthetic: bool,
    ) -> Result<Self, CollectionError> {
        validate_selection(&inputs, &selection)?;
        Ok(Self {
            inputs,
            selection,
            allow_synthetic,
        })
    }

    /// Decodes a strict version-1 batch manifest and resolves relative inputs beside `path`.
    ///
    /// This validates structure and selection only; it does not read pack paths.
    ///
    /// # Errors
    ///
    /// Returns a typed request, version, selection, or current-directory error
    /// when the manifest is malformed or cannot be resolved.
    pub fn from_manifest(
        bytes: &[u8],
        path: impl AsRef<Path>,
        selection: CollectionSelection,
        allow_synthetic: bool,
    ) -> Result<CollectionRequest<'static>, CollectionError> {
        let manifest: CollectionManifest = serde_json::from_slice(bytes).map_err(|error| {
            CollectionError::InvalidRequest(format!("invalid collection manifest: {error}"))
        })?;
        if manifest.version != 1 {
            return Err(CollectionError::UnsupportedVersion(format!(
                "collection manifest version {}",
                manifest.version
            )));
        }
        let path = path.as_ref();
        let absolute = if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir()
                .map_err(|error| CollectionError::ArtifactIoSource {
                    path: Some(path.to_path_buf()),
                    source: Box::new(error),
                })?
                .join(path)
        };
        let parent = absolute.parent().unwrap_or_else(|| Path::new("."));
        let resolve = |value: String| {
            let supplied = PathBuf::from(value);
            if supplied.is_absolute() {
                supplied
            } else {
                parent.join(supplied)
            }
        };
        let mut inputs = Vec::with_capacity(manifest.inputs.len());
        for record in manifest.inputs {
            let input = match record {
                CollectionManifestInputKind::Sealed {
                    pack_id,
                    release,
                    seal,
                } => CollectionInput::SealedFiles {
                    pack_id: parse_manifest_pack(&pack_id)?,
                    release: resolve(release),
                    seal: resolve(seal),
                },
                CollectionManifestInputKind::Jbotci {
                    pack_id,
                    payload_kind,
                    raw,
                    metadata,
                    options,
                } => {
                    let pack_id = parse_manifest_pack(&pack_id)?;
                    if pack_id != PackId::LensiskuJbotci {
                        return Err(CollectionError::InvalidRequest(
                            "kind jbotci requires pack_id lensisku-jbotci".into(),
                        ));
                    }
                    CollectionInput::Jbotci {
                        payload_kind,
                        raw: resolve(raw),
                        metadata: resolve(metadata),
                        options: options.into(),
                    }
                }
                CollectionManifestInputKind::Llg {
                    pack_id,
                    payload_kind,
                    gismu,
                    cmavo,
                    rafsi,
                    notice,
                    options,
                } => {
                    let pack_id = parse_manifest_pack(&pack_id)?;
                    if pack_id != PackId::Llg {
                        return Err(CollectionError::InvalidRequest(
                            "kind llg requires pack_id llg".into(),
                        ));
                    }
                    CollectionInput::Llg {
                        payload_kind,
                        gismu: resolve(gismu),
                        cmavo: resolve(cmavo),
                        rafsi: resolve(rafsi),
                        notice: resolve(notice),
                        options: options.into(),
                    }
                }
                CollectionManifestInputKind::LaLojban {
                    pack_id,
                    payload_kind,
                    raw,
                    options,
                } => {
                    let pack_id = parse_manifest_pack(&pack_id)?;
                    if pack_id != PackId::LaLojban {
                        return Err(CollectionError::InvalidRequest(
                            "kind la-lojban requires pack_id la-lojban".into(),
                        ));
                    }
                    CollectionInput::LaLojban {
                        payload_kind,
                        raw: resolve(raw),
                        options: options.into(),
                    }
                }
                CollectionManifestInputKind::Sutysisku {
                    pack_id,
                    payload_kind,
                    raw,
                    options,
                } => {
                    let pack_id = parse_manifest_pack(&pack_id)?;
                    if pack_id != PackId::Sutysisku {
                        return Err(CollectionError::InvalidRequest(
                            "kind sutysisku requires pack_id sutysisku".into(),
                        ));
                    }
                    CollectionInput::Sutysisku {
                        payload_kind,
                        raw: resolve(raw),
                        options: options.into(),
                    }
                }
            };
            inputs.push(input);
        }
        CollectionRequest::new(inputs, selection, allow_synthetic)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CollectionManifest {
    version: u32,
    inputs: Vec<CollectionManifestInputKind>,
}

#[derive(Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
enum CollectionManifestInputKind {
    #[serde(rename = "sealed")]
    Sealed {
        pack_id: String,
        release: String,
        seal: String,
    },
    #[serde(rename = "jbotci")]
    Jbotci {
        pack_id: String,
        payload_kind: PackPayloadKind,
        raw: String,
        metadata: String,
        options: JbotciImportOptionsManifest,
    },
    #[serde(rename = "llg")]
    Llg {
        pack_id: String,
        payload_kind: PackPayloadKind,
        gismu: String,
        cmavo: String,
        rafsi: String,
        notice: String,
        options: LlgImportOptionsManifest,
    },
    #[serde(rename = "la-lojban")]
    LaLojban {
        pack_id: String,
        payload_kind: PackPayloadKind,
        raw: String,
        options: LaLojbanImportOptionsManifest,
    },
    #[serde(rename = "sutysisku")]
    Sutysisku {
        pack_id: String,
        payload_kind: PackPayloadKind,
        raw: String,
        options: SutysiskuImportOptionsManifest,
    },
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct JbotciImportOptionsManifest {
    revision: String,
    expected_raw_sha256: String,
    expected_metadata_sha256: String,
}
impl From<JbotciImportOptionsManifest> for JbotciImportOptions {
    fn from(value: JbotciImportOptionsManifest) -> Self {
        Self {
            revision: value.revision,
            expected_raw_sha256: value.expected_raw_sha256,
            expected_metadata_sha256: value.expected_metadata_sha256,
        }
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LlgImportOptionsManifest {
    expected_gismu_sha256: String,
    expected_cmavo_sha256: String,
    expected_rafsi_sha256: String,
    expected_notice_sha256: String,
    base_locator: String,
}
impl From<LlgImportOptionsManifest> for LlgImportOptions {
    fn from(value: LlgImportOptionsManifest) -> Self {
        Self {
            expected_gismu_sha256: value.expected_gismu_sha256,
            expected_cmavo_sha256: value.expected_cmavo_sha256,
            expected_rafsi_sha256: value.expected_rafsi_sha256,
            expected_notice_sha256: value.expected_notice_sha256,
            base_locator: value.base_locator,
        }
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LaLojbanImportOptionsManifest {
    revision: String,
    expected_raw_sha256: String,
    #[serde(default)]
    raw_locator: Option<String>,
    #[serde(default)]
    notice_locator: Option<String>,
}
impl From<LaLojbanImportOptionsManifest> for LaLojbanImportOptions {
    fn from(value: LaLojbanImportOptionsManifest) -> Self {
        Self {
            revision: value.revision,
            expected_raw_sha256: value.expected_raw_sha256,
            raw_locator: value.raw_locator,
            notice_locator: value.notice_locator,
        }
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SutysiskuImportOptionsManifest {
    revision: String,
    expected_raw_sha256: String,
    #[serde(default)]
    raw_locator: Option<String>,
    #[serde(default)]
    notice_locator: Option<String>,
}
impl From<SutysiskuImportOptionsManifest> for SutysiskuImportOptions {
    fn from(value: SutysiskuImportOptionsManifest) -> Self {
        Self {
            revision: value.revision,
            expected_raw_sha256: value.expected_raw_sha256,
            raw_locator: value.raw_locator,
            notice_locator: value.notice_locator,
        }
    }
}

fn parse_manifest_pack(value: &str) -> Result<PackId, CollectionError> {
    match value {
        "lensisku-jbotci" => Ok(PackId::LensiskuJbotci),
        "llg" => Ok(PackId::Llg),
        "la-lojban" => Ok(PackId::LaLojban),
        "sutysisku" => Ok(PackId::Sutysisku),
        _ => Err(CollectionError::InvalidRequest(format!(
            "unknown pack id {value:?}"
        ))),
    }
}

/// Stable categories for collection operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum CollectionErrorCategory {
    /// The request or typed input is malformed.
    InvalidRequest,
    /// A catalogue, manifest, or model version is unsupported.
    UnsupportedVersion,
    /// No owned catalogue exists at the requested path.
    MissingCatalogue,
    /// A caller-owned destination already exists.
    OccupiedDestination,
    /// A selected input could not be read.
    InputIo,
    /// A selected input failed pin, seal, or model validation.
    InputInvalid,
    /// Co-active identities conflict.
    IdentityConflict,
    /// Replacing an installed pack is reserved for a later lifecycle operation.
    UpdateRequired,
    /// The catalogue writer is busy.
    Busy,
    /// An owned artifact or catalogue write failed.
    ArtifactIo,
    /// Explicit full verification failed.
    VerificationFailed,
    /// Durable commit outcome cannot be established.
    Indeterminate,
}

/// Typed failure from collection catalogue operations.
#[derive(Debug)]
#[non_exhaustive]
pub enum CollectionError {
    /// Request structure or policy is invalid.
    InvalidRequest(String),
    /// Catalogue or model schema is not supported.
    UnsupportedVersion(String),
    /// The requested path has no initialized catalogue.
    MissingCatalogue(PathBuf),
    /// A protected destination already exists.
    OccupiedDestination(PathBuf),
    /// Selected input could not be read.
    InputIo {
        /// Native path that could not be read.
        path: PathBuf,
        /// Original filesystem error.
        source: io::Error,
    },
    /// Selected input did not satisfy its declared identity.
    InputInvalid {
        /// Pack whose input was invalid, when known.
        pack: Option<PackId>,
        /// Concise validation diagnostic.
        detail: String,
    },
    /// A sealed input failed validation and its typed PackError is retained.
    InputInvalidSource {
        /// Pack whose input was invalid, when known.
        pack: Option<PackId>,
        /// Concise validation context for the failure.
        detail: String,
        /// Original typed seal or release validation error.
        source: Box<dyn Error + Send + Sync>,
    },
    /// Existing source adapter rejected its selected raw input.
    Adapter {
        /// Pack whose adapter failed.
        pack: PackId,
        /// Original typed importer error.
        source: Box<dyn Error + Send + Sync>,
    },
    /// Co-active pack descriptors disagree.
    IdentityConflict {
        /// Identity category that conflicted.
        category: String,
        /// Stable key for the conflict.
        key: String,
    },
    /// Typed pack-composition identity conflict.
    PackConflict {
        /// Identity category that conflicted.
        category: String,
        /// Stable key for the conflict.
        key: String,
        /// Original typed pack error.
        source: Box<dyn Error + Send + Sync>,
    },
    /// A different release is already installed and updates belong to S04.
    UpdateRequired(PackId),
    /// Catalogue writer could not obtain its immediate transaction.
    Busy(String),
    /// Busy SQLite operation with its original database error retained.
    BusySource(rusqlite::Error),
    /// An owned artifact or catalogue operation failed.
    ArtifactIo {
        /// Native path involved, when one is known.
        path: Option<PathBuf>,
        /// Filesystem or database diagnostic.
        detail: String,
    },
    /// Artifact/database failure retaining the typed underlying cause.
    ArtifactIoSource {
        /// Native path involved, when one is known.
        path: Option<PathBuf>,
        /// Original filesystem, SQLite, or serialization error.
        source: Box<dyn Error + Send + Sync>,
    },
    /// Full verification failed.
    VerificationFailed(String),
    /// Verification failure retaining its typed underlying cause.
    VerificationFailedSource {
        /// Concise verification context.
        detail: String,
        /// Original typed verification or decoding error, when one exists.
        source: Option<Box<dyn Error + Send + Sync>>,
    },
    /// Commit or rollback outcome is not safely knowable.
    Indeterminate(String),
    /// Indeterminate mutation retaining initiating and rollback causes.
    IndeterminateSources {
        /// Concise mutation context.
        detail: String,
        /// Initiating or commit error, when available.
        source: Option<Box<dyn Error + Send + Sync>>,
        /// Rollback failure, retained independently from the initiating error.
        rollback_source: Option<Box<dyn Error + Send + Sync>>,
    },
    /// Existing adapter, sealing, or model validation failed.
    Pack(Box<dyn Error + Send + Sync>),
    /// One selected input failed during ordered batch normalization.
    Batch {
        /// Underlying typed failure from the selected input.
        source: Box<CollectionError>,
        /// Complete selected/skipped/unprocessed disposition for the batch.
        inputs: Vec<CollectionInputDisposition>,
    },
    /// Mutation failure enriched with the transaction's known identity context.
    MutationContext {
        /// Original typed operation failure.
        source: Box<CollectionError>,
        /// Attempt identity allocated before failure, when any.
        attempt_id: Option<String>,
        /// Prior catalogue revision, when read successfully.
        prior_local_revision: Option<u64>,
        /// Proposed generation, when allocated.
        proposed_generation: Option<String>,
        /// Generation known after rollback/commit observation, when any.
        observed_generation: Option<String>,
        /// Selected inputs already validated before the transaction failure.
        inputs: Vec<CollectionInputDisposition>,
    },
}

impl CollectionError {
    /// Returns the stable category for this error.
    #[must_use]
    pub fn category(&self) -> CollectionErrorCategory {
        match self {
            Self::InvalidRequest(_) => CollectionErrorCategory::InvalidRequest,
            Self::UnsupportedVersion(_) => CollectionErrorCategory::UnsupportedVersion,
            Self::MissingCatalogue(_) => CollectionErrorCategory::MissingCatalogue,
            Self::OccupiedDestination(_) => CollectionErrorCategory::OccupiedDestination,
            Self::InputIo { .. } => CollectionErrorCategory::InputIo,
            Self::InputInvalid { .. }
            | Self::InputInvalidSource { .. }
            | Self::Adapter { .. }
            | Self::Pack(_) => CollectionErrorCategory::InputInvalid,
            Self::IdentityConflict { .. } | Self::PackConflict { .. } => {
                CollectionErrorCategory::IdentityConflict
            }
            Self::UpdateRequired(_) => CollectionErrorCategory::UpdateRequired,
            Self::Busy(_) | Self::BusySource(_) => CollectionErrorCategory::Busy,
            Self::ArtifactIo { .. } | Self::ArtifactIoSource { .. } => {
                CollectionErrorCategory::ArtifactIo
            }
            Self::VerificationFailed(_) | Self::VerificationFailedSource { .. } => {
                CollectionErrorCategory::VerificationFailed
            }
            Self::Indeterminate(_) | Self::IndeterminateSources { .. } => {
                CollectionErrorCategory::Indeterminate
            }
            Self::Batch { source, .. } | Self::MutationContext { source, .. } => source.category(),
        }
    }

    /// Builds a versioned failure receipt while preserving native path encoding.
    #[must_use]
    pub fn report(&self, operation: &str) -> CollectionFailureReport {
        if let Self::MutationContext {
            source,
            attempt_id,
            prior_local_revision,
            proposed_generation,
            observed_generation,
            inputs,
        } = self
        {
            let mut report = source.report(operation);
            report.attempt_id = attempt_id.clone();
            report.prior_local_revision = *prior_local_revision;
            report.proposed_generation = proposed_generation.clone();
            report.observed_generation = observed_generation.clone();
            if !inputs.is_empty() {
                report.inputs = inputs.clone();
            }
            return report;
        }
        let batch_pack_id = match self {
            Self::Batch { inputs, .. } => {
                let mut failed = inputs.iter().filter(|item| item.disposition == "failed");
                let only = failed.next().map(|item| item.pack_id.clone());
                if failed.next().is_none() { only } else { None }
            }
            _ => None,
        };
        let source = match self {
            Self::Batch { source, .. } => source.as_ref(),
            other => other,
        };
        let (path, pack_id) = match source {
            Self::MissingCatalogue(path)
            | Self::OccupiedDestination(path)
            | Self::InputIo { path, .. } => (Some(native_path_report(path)), None),
            Self::ArtifactIo { path, .. } | Self::ArtifactIoSource { path, .. } => {
                (path.as_deref().map(native_path_report), None)
            }
            Self::InputInvalid { pack, .. } | Self::InputInvalidSource { pack, .. } => {
                (None, pack.map(|id| id.as_str().to_owned()))
            }
            Self::PackConflict { .. } => (None, None),
            Self::Adapter { pack, .. } | Self::UpdateRequired(pack) => {
                (None, Some(pack.as_str().to_owned()))
            }
            _ => (None, None),
        };
        let pack_id = pack_id.or(batch_pack_id);
        let detail = match source {
            Self::InputInvalidSource {
                pack,
                detail,
                source,
            } => format!(
                "invalid {} input: {detail}: {source}",
                pack.map_or("collection pack".into(), |id| id.to_string())
            ),
            _ => self.to_string(),
        };
        CollectionFailureReport {
            version: 1,
            ok: false,
            operation: operation.to_owned(),
            category: category_name(self.category()).into(),
            detail,
            path,
            pack_id,
            attempt_id: None,
            prior_local_revision: None,
            proposed_generation: None,
            observed_generation: None,
            activation: if matches!(
                source,
                Self::Indeterminate(_) | Self::IndeterminateSources { .. }
            ) {
                Some("indeterminate".into())
            } else if matches!(operation, "register" | "install") {
                Some("not_activated".into())
            } else {
                Some("not_applicable".into())
            },
            inputs: match self {
                Self::Batch { inputs, .. } => inputs.clone(),
                _ => Vec::new(),
            },
        }
    }
}

fn category_name(category: CollectionErrorCategory) -> &'static str {
    match category {
        CollectionErrorCategory::InvalidRequest => "invalid_request",
        CollectionErrorCategory::UnsupportedVersion => "unsupported_version",
        CollectionErrorCategory::MissingCatalogue => "missing_catalogue",
        CollectionErrorCategory::OccupiedDestination => "occupied_destination",
        CollectionErrorCategory::InputIo => "input_io",
        CollectionErrorCategory::InputInvalid => "input_invalid",
        CollectionErrorCategory::IdentityConflict => "identity_conflict",
        CollectionErrorCategory::UpdateRequired => "update_required",
        CollectionErrorCategory::Busy => "busy",
        CollectionErrorCategory::ArtifactIo => "artifact_io",
        CollectionErrorCategory::VerificationFailed => "verification_failed",
        CollectionErrorCategory::Indeterminate => "indeterminate",
    }
}

impl fmt::Display for CollectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRequest(detail) => write!(f, "invalid collection request: {detail}"),
            Self::UnsupportedVersion(detail) => {
                write!(f, "unsupported collection version: {detail}")
            }
            Self::MissingCatalogue(path) => {
                write!(f, "collection catalogue is missing: {}", path.display())
            }
            Self::OccupiedDestination(path) => write!(
                f,
                "collection destination already exists: {}",
                path.display()
            ),
            Self::InputIo { path, source } => {
                write!(f, "read collection input {}: {source}", path.display())
            }
            Self::InputInvalid { pack, detail } => write!(
                f,
                "invalid {} input: {detail}",
                pack.map_or("collection pack".into(), |id| id.to_string())
            ),
            Self::InputInvalidSource { pack, detail, .. } => write!(
                f,
                "invalid {} input: {detail}",
                pack.map_or("collection pack".into(), |id| id.to_string())
            ),
            Self::Adapter { pack, source } => {
                write!(f, "{} importer rejected input: {source}", pack.as_str())
            }
            Self::IdentityConflict { category, key } => {
                write!(f, "conflicting {category} identity {key}")
            }
            Self::PackConflict {
                category,
                key,
                source,
            } => write!(f, "conflicting {category} identity {key}: {source}"),
            Self::UpdateRequired(pack) => write!(
                f,
                "updating installed pack {pack} is reserved for a later lifecycle operation"
            ),
            Self::Busy(detail) => write!(f, "collection catalogue is busy: {detail}"),
            Self::BusySource(source) => write!(f, "collection catalogue is busy: {source}"),
            Self::ArtifactIo {
                path: Some(path),
                detail,
            } => write!(f, "collection artifact {}: {detail}", path.display()),
            Self::ArtifactIo { path: None, detail } => write!(f, "collection artifact: {detail}"),
            Self::ArtifactIoSource {
                path: Some(path),
                source,
            } => write!(f, "collection artifact {}: {source}", path.display()),
            Self::ArtifactIoSource { path: None, source } => {
                write!(f, "collection artifact: {source}")
            }
            Self::VerificationFailed(detail) => {
                write!(f, "collection verification failed: {detail}")
            }
            Self::VerificationFailedSource { detail, source } => {
                write!(f, "collection verification failed: {detail}")?;
                if let Some(source) = source {
                    write!(f, ": {source}")?;
                }
                Ok(())
            }
            Self::Indeterminate(detail) => {
                write!(f, "collection mutation outcome is indeterminate: {detail}")
            }
            Self::IndeterminateSources {
                detail,
                source,
                rollback_source,
            } => {
                write!(f, "collection mutation outcome is indeterminate: {detail}")?;
                if let Some(source) = source {
                    write!(f, "; cause: {source}")?;
                }
                if let Some(rollback) = rollback_source {
                    write!(f, "; rollback cause: {rollback}")?;
                }
                Ok(())
            }
            Self::Pack(error) => write!(f, "collection pack error: {error}"),
            Self::Batch { source, .. } => write!(f, "{source}"),
            Self::MutationContext {
                source,
                attempt_id,
                prior_local_revision,
                proposed_generation,
                observed_generation,
                ..
            } => write!(
                f,
                "{source} (attempt={attempt_id:?}, prior_revision={prior_local_revision:?}, proposed_generation={proposed_generation:?}, observed_generation={observed_generation:?})"
            ),
        }
    }
}
impl Error for CollectionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InputIo { source, .. } => Some(source),
            Self::InputInvalidSource { source, .. } => Some(source.as_ref()),
            Self::Adapter { source, .. } => Some(source.as_ref()),
            Self::Pack(source) => Some(source.as_ref()),
            Self::BusySource(source) => Some(source),
            Self::PackConflict { source, .. } => Some(source.as_ref()),
            Self::ArtifactIoSource { source, .. } => Some(source.as_ref()),
            Self::VerificationFailedSource {
                source: Some(source),
                ..
            } => Some(source.as_ref()),
            Self::IndeterminateSources {
                source: Some(source),
                ..
            } => Some(source.as_ref()),
            Self::Batch { source, .. } | Self::MutationContext { source, .. } => {
                Some(source.as_ref())
            }
            _ => None,
        }
    }
}

/// A native filesystem path suitable for explicit JSON reporting.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativePathReport {
    /// Native encoding label.
    pub encoding: String,
    /// Exact native bytes as lowercase hexadecimal.
    pub hex: String,
    /// UTF-8 spelling when the native bytes are valid UTF-8.
    pub utf8: Option<String>,
}

/// Summary of one requested pack's processing disposition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CollectionInputDisposition {
    /// Stable pack identifier.
    pub pack_id: String,
    /// Outcome such as validated, skipped_without_io, failed or unprocessed.
    pub disposition: String,
    /// Input payload kind when known.
    pub payload_kind: Option<String>,
    /// Release digest when known.
    pub release_sha256: Option<String>,
    /// Importer accounting report, when raw normalization was performed.
    pub report: Option<serde_json::Value>,
}

/// Versioned structured receipt for a failed operation.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CollectionFailureReport {
    /// Receipt schema version.
    pub version: u32,
    /// Failure receipts are never successful operations.
    pub ok: bool,
    /// Requested operation, such as register or install.
    pub operation: String,
    /// Stable category token.
    pub category: String,
    /// Human-readable diagnostic detail.
    pub detail: String,
    /// Native path involved, where known.
    pub path: Option<NativePathReport>,
    /// Pack identity involved, where known.
    pub pack_id: Option<String>,
    /// Mutation attempt identity, when allocated.
    pub attempt_id: Option<String>,
    /// Catalogue revision observed before the attempted mutation, when known.
    pub prior_local_revision: Option<u64>,
    /// Proposed generation identity, when known.
    pub proposed_generation: Option<String>,
    /// Generation identity observed after the operation, when known.
    pub observed_generation: Option<String>,
    /// Activation observation: not_activated, committed, indeterminate, or not_applicable.
    pub activation: Option<String>,
    /// Selected, skipped and unprocessed input dispositions.
    pub inputs: Vec<CollectionInputDisposition>,
}

/// Immutable identity for a completed Unified generation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CollectionGeneration {
    /// Physical artifact identity, unique within its catalogue.
    pub generation_id: String,
    /// Data-derived identity for the ordered pack/release set.
    pub content_id: String,
    /// Expected canonical dictionary digest.
    pub canonical_sha256: String,
    /// Native path to the derived store directory.
    #[serde(skip)]
    pub store_path: PathBuf,
    /// JSON-safe native path descriptor.
    pub path: NativePathReport,
    /// Ordered installed members.
    pub members: Vec<CollectionGenerationMember>,
}

/// One release represented in a generation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CollectionGenerationMember {
    /// Stable pack ID.
    pub pack_id: String,
    /// Sealed canonical release digest.
    pub release_sha256: String,
    /// Declared payload scope.
    pub payload_kind: String,
}

/// Native paths for the currently installed pack copies and derived store.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CollectionInstalledArtifactLocations {
    /// Stable pack identifier.
    pub pack_id: String,
    /// Sealed release identity.
    pub release_sha256: String,
    /// Tool-owned canonical release file.
    pub release_path: PathBuf,
    /// Tool-owned seal file.
    pub seal_path: PathBuf,
    /// Optional importer report file.
    pub report_path: Option<PathBuf>,
    /// Tool-owned derived-store directory.
    pub store_path: PathBuf,
    /// Tool-owned SQLite file within the derived store.
    pub database_path: PathBuf,
}

impl CollectionGeneration {
    /// Builds a Unified manifest without certifying current bytes.
    #[must_use]
    pub fn snapshot_manifest(&self) -> crate::DictionarySnapshotManifest {
        crate::DictionarySnapshotManifest {
            version: 1,
            generation: self.generation_id.clone(),
            logical_state_sha256: self.canonical_sha256.clone(),
            members: vec![crate::DictionarySnapshotMember {
                id: "unified".into(),
                path: self.store_path.clone(),
                expected_state_sha256: self.canonical_sha256.clone(),
            }],
            selected_release_ids: None,
        }
    }
}

/// Metadata-only retained source and typed-reference catalogue descriptor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CollectionReferenceMetadata {
    /// Report schema version.
    pub version: u32,
    /// Stable pack identity.
    pub pack_id: String,
    /// Exact canonical release digest.
    pub release_sha256: String,
    /// Declared input scope.
    pub payload_kind: String,
    /// Dictionary schema version.
    pub dictionary_version: u32,
    /// Registry schema version.
    pub registry_version: u32,
    /// Upstream catalogue revision, distinct from local catalogue revision.
    pub upstream_catalogue_revision: u64,
    /// Exact source/distributor/alias/lineage/snapshot/occurrence registry.
    pub registry: RegistryDocument,
    /// Collections, components, releases and code notices.
    pub source_metadata: CollectionSourceMetadata,
    /// Capture metadata and occurrence locators, without captured bytes.
    pub captures: Vec<CollectionCaptureReference>,
    /// Role-qualified local references with sensitive body values omitted.
    pub entities: Vec<CollectionEntityReference>,
    /// Canonical record digests keyed by role and native ID.
    pub body_sha256: BTreeMap<String, String>,
    /// Extension digests retain source order and duplicate multiplicity.
    pub extension_row_sha256_ordered_multiset: Vec<String>,
    /// Counts for every native record category, including empty categories.
    pub category_counts: CollectionCategoryCounts,
}

/// Source-level catalogue rows retained without lexical body payloads.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CollectionSourceMetadata {
    /// Source collections with exact provenance fields.
    pub collections: Vec<CollectionRecord>,
    /// Components assigned within those collections.
    pub components: Vec<ComponentRecord>,
    /// Releases, notices, accounting and assertion membership.
    pub releases: Vec<ReleaseRecord>,
    /// Code redistribution notices.
    pub code_notices: Vec<CodeNotice>,
}

/// A capture's identity, metadata, length and ordered occurrence map.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CollectionCaptureReference {
    /// Native capture identity.
    pub capture: CaptureRef,
    /// Capture-time source metadata.
    pub metadata: CaptureMetadata,
    /// Captured byte length, without captured bytes.
    pub byte_length: usize,
    /// Exact occurrence locators in source order.
    pub occurrences: Vec<OccurrenceRef>,
}

/// Role-qualified entity ID and its non-body references.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CollectionEntityReference {
    /// Typed entity role.
    pub role: String,
    /// Stable native model ID.
    pub id: String,
    /// Related entry IDs, where applicable.
    pub entry_ids: Vec<String>,
    /// Related assertion IDs, where applicable.
    pub assertion_ids: Vec<String>,
    /// Related source IDs, where applicable.
    pub source_ids: Vec<String>,
    /// Exact occurrence reference when the entity is occurrence-backed.
    pub occurrence: Option<OccurrenceRef>,
    /// Exact field-level evidence when present.
    pub evidence: Option<SourceEvidence>,
    /// Spelling presence state for Entry entities, without other lexical body.
    pub spelling: Option<Presence<String>>,
}

/// Complete native category counts for a reference descriptor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CollectionCategoryCounts {
    /// Entry rows.
    pub entries: usize,
    /// Assertion rows.
    pub assertions: usize,
    /// Context rows.
    pub contexts: usize,
    /// Frame rows.
    pub frames: usize,
    /// Form rows.
    pub forms: usize,
    /// Etymology rows.
    pub etymologies: usize,
    /// Relation rows.
    pub relations: usize,
    /// Owned revision rows.
    pub owned_revisions: usize,
    /// Owned head rows.
    pub owned_heads: usize,
    /// Editorial sense rows.
    pub editorial_senses: usize,
    /// Computation rows.
    pub computations: usize,
    /// Annotation rows.
    pub annotations: usize,
    /// Extension rows, including duplicates.
    pub extensions: usize,
}

/// Read-only handle pinned to one collection catalogue snapshot.
pub struct CollectionCatalogue {
    root: PathBuf,
    connection: Connection,
}

impl std::fmt::Debug for CollectionCatalogue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CollectionCatalogue")
            .field("root", &self.root)
            .finish_non_exhaustive()
    }
}

impl CollectionCatalogue {
    /// Opens an existing catalogue read-only without certifying payload bytes.
    ///
    /// # Errors
    ///
    /// Returns a missing/version/format error when the catalogue is absent or
    /// does not belong to this catalogue family.
    pub fn open(root: impl AsRef<Path>) -> Result<Self, CollectionError> {
        let root = absolute_path(root.as_ref())?;
        let connection = crate::collections_schema::open_readonly(&root)?;
        Ok(Self { root, connection })
    }

    /// Returns the current catalogue-local registration/generation revision.
    ///
    /// This is a light catalogue observation and does not audit installed payloads.
    ///
    /// # Errors
    /// Returns a typed catalogue error when the revision cannot be read.
    pub fn local_revision(&self) -> Result<u64, CollectionError> {
        crate::collections_schema::local_revision(&self.connection, &self.root)
    }

    /// Returns exact current tool-owned artifact paths for an installed pack.
    ///
    /// This decodes catalogue path metadata only; it does not test file
    /// existence, hash bytes, or open the derived store. Registered-only packs
    /// return `Ok(None)`.
    ///
    /// # Errors
    /// Returns a typed catalogue error for an unknown pack or malformed metadata.
    pub fn installed_artifact_locations(
        &self,
        pack: PackId,
    ) -> Result<Option<CollectionInstalledArtifactLocations>, CollectionError> {
        let registered: bool = self
            .connection
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM registrations WHERE pack_id=?1)",
                [pack.as_str()],
                |row| row.get(0),
            )
            .map_err(|error| db_error(&self.root, error))?;
        if !registered {
            return Err(CollectionError::InvalidRequest(format!(
                "pack {} is not registered",
                pack.as_str()
            )));
        }
        let Some(generation) = read_generation(&self.connection, &self.root)? else {
            return Ok(None);
        };
        let Some(member) = current_members(&self.connection, &self.root)?
            .into_iter()
            .find(|member| member.pack_id == pack.as_str())
        else {
            return Ok(None);
        };
        let database_path = generation.store_path.join("dictionary.sqlite3");
        Ok(Some(CollectionInstalledArtifactLocations {
            pack_id: pack.as_str().to_owned(),
            release_sha256: member.release_sha256,
            release_path: self.root.join(member.release_path),
            seal_path: self.root.join(member.seal_path),
            report_path: member.report_path.map(|path| self.root.join(path)),
            store_path: generation.store_path,
            database_path,
        }))
    }

    /// Returns the catalogue's local revision and pack summaries.
    ///
    /// # Errors
    ///
    /// Returns a typed catalogue error when its read snapshot cannot be decoded.
    pub fn list(&self) -> Result<CollectionListReport, CollectionError> {
        let local_revision =
            crate::collections_schema::local_revision(&self.connection, &self.root)?;
        let generation = read_generation(&self.connection, &self.root)?;
        let mut statement = self
            .connection
            .prepare("SELECT DISTINCT pack_id FROM registrations ORDER BY pack_id")
            .map_err(|error| db_error(&self.root, error))?;
        let ids = statement
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|error| db_error(&self.root, error))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| db_error(&self.root, error))?;
        let packs = ids
            .iter()
            .map(|id| self.inspect_id(id))
            .collect::<Result<Vec<_>, _>>()?;
        let mut staged = self
            .connection
            .prepare(
                "SELECT attempt_id FROM attempts WHERE status <> 'committed' ORDER BY attempt_id",
            )
            .map_err(|error| db_error(&self.root, error))?;
        let mut staged_attempts = staged
            .query_map([], |row| row.get(0))
            .map_err(|error| db_error(&self.root, error))?
            .collect::<Result<Vec<String>, _>>()
            .map_err(|error| db_error(&self.root, error))?;
        staged_attempts.extend(orphan_stage_attempts(&self.root, &self.connection)?);
        staged_attempts.sort();
        staged_attempts.dedup();
        Ok(CollectionListReport {
            version: 1,
            local_revision,
            generation,
            packs,
            staged_attempts,
        })
    }

    /// Returns registration and installation summaries for one pack.
    ///
    /// # Errors
    ///
    /// Returns an invalid-request error for a pack without a registration.
    pub fn inspect(&self, pack: PackId) -> Result<CollectionInspection, CollectionError> {
        self.inspect_id(pack.as_str())
    }

    fn inspect_id(&self, pack_id: &str) -> Result<CollectionInspection, CollectionError> {
        let installed: Option<String> = self
            .connection
            .query_row(
                "SELECT release_sha256 FROM installed_heads WHERE pack_id=?1",
                [pack_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|error| db_error(&self.root, error))?;
        let mut statement = self.connection.prepare(
            "SELECT r.release_sha256,r.payload_kind,l.input_paths_json,r.summary_json FROM registrations r
             JOIN registration_locations l USING(pack_id,release_sha256)
             WHERE r.pack_id=?1 ORDER BY r.release_sha256,l.input_paths_json"
        ).map_err(|error| db_error(&self.root, error))?;
        let mut rows = statement
            .query([pack_id])
            .map_err(|error| db_error(&self.root, error))?;
        let generation = read_generation(&self.connection, &self.root)?;
        let installed_member = generation.as_ref().and_then(|generation| {
            generation
                .members
                .iter()
                .find(|member| member.pack_id == pack_id)
        });
        let owned_files_present = installed_member
            .map(|_| owned_member_files_present(&self.root, &self.connection, pack_id));
        let minimum_store =
            generation.as_ref().map(|generation| {
                match crate::DictionaryStore::open(&generation.store_path) {
                    Ok(store) => {
                        drop(store);
                        CollectionDerivedStoreMinimum {
                            status: "supported".into(),
                            diagnostic: None,
                        }
                    }
                    Err(error) => CollectionDerivedStoreMinimum {
                        status: "unsupported_or_unreadable".into(),
                        diagnostic: Some(error.to_string()),
                    },
                }
            });
        let mut registrations = Vec::new();
        while let Some(row) = rows.next().map_err(|error| db_error(&self.root, error))? {
            let release_sha256: String = row.get(0).map_err(|error| db_error(&self.root, error))?;
            let payload_kind: String = row.get(1).map_err(|error| db_error(&self.root, error))?;
            let paths: Vec<u8> = row.get(2).map_err(|error| db_error(&self.root, error))?;
            let summary_bytes: Vec<u8> = row.get(3).map_err(|error| db_error(&self.root, error))?;
            let path_reports: Vec<NativePathReport> =
                serde_json::from_slice(&paths).map_err(|error| {
                    CollectionError::UnsupportedVersion(format!(
                        "registration paths are malformed: {error}"
                    ))
                })?;
            let input_ownership = if path_reports.is_empty() {
                "embedded_or_borrowed"
            } else {
                "caller_managed"
            };
            let installed_for_registration =
                installed_member.is_some_and(|member| member.release_sha256 == release_sha256);
            let summary: StoredCollectionRegistrationSummary =
                serde_json::from_slice(&summary_bytes).map_err(|error| {
                    CollectionError::UnsupportedVersion(format!(
                        "registration summary is malformed: {error}"
                    ))
                })?;
            registrations.push(CollectionRegistrationSummary {
                release_sha256,
                payload_kind,
                inputs_present: path_reports.iter().all(|path| {
                    native_report_path(path).is_some_and(|path| fs::symlink_metadata(path).is_ok())
                }),
                inputs: path_reports,
                input_ownership: input_ownership.into(),
                installed_pack_copy_ownership: if installed_for_registration {
                    "tool_owned".into()
                } else {
                    "not_installed".into()
                },
                derived_store_ownership: if installed_for_registration {
                    "tool_owned".into()
                } else {
                    "not_installed".into()
                },
                compatibility: "not_evaluated".into(),
                source_ids: summary.source_ids,
                sources: summary.sources,
                distributor_ids: summary.distributor_ids,
                distributors: summary.distributors,
                lineage: summary.lineage,
                captures: summary.captures,
                collection_ids: summary.collection_ids,
                collections: summary.collections,
                components: summary.components,
                releases: summary.releases,
                upstream_catalogue_revision: summary.upstream_catalogue_revision,
                dictionary_version: summary.dictionary_version,
                registry_version: summary.registry_version,
                reference_counts: summary.reference_counts,
                terms: summary.terms,
            });
        }
        drop(rows);
        drop(statement);
        if registrations.is_empty() {
            return Err(CollectionError::InvalidRequest(format!(
                "pack {pack_id} is not registered"
            )));
        }
        let has_installation = installed_member.is_some();
        let installed_history = self.installed_history(pack_id)?;
        Ok(CollectionInspection {
            version: 1,
            pack_id: pack_id.into(),
            installed_release_sha256: installed,
            installed_history,
            registrations,
            owned_files_present,
            last_verification_scope: if has_installation {
                Some("all".into())
            } else {
                None
            },
            derived_store_minimum: minimum_store,
            policy_state: "unconfigured".into(),
        })
    }

    fn installed_history(
        &self,
        pack_id: &str,
    ) -> Result<Vec<CollectionInstalledHistory>, CollectionError> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT generation_id,content_id,canonical_sha256,local_revision,members_json
             FROM generations ORDER BY local_revision DESC",
            )
            .map_err(|error| db_error(&self.root, error))?;
        let rows = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, Vec<u8>>(4)?,
                ))
            })
            .map_err(|error| db_error(&self.root, error))?;
        let current_generation = read_generation(&self.connection, &self.root)?
            .map(|generation| generation.generation_id);
        let mut history = Vec::new();
        for row in rows {
            let (generation_id, content_id, canonical_sha256, revision, members_bytes) =
                row.map_err(|error| db_error(&self.root, error))?;
            let members: Vec<InstalledMember> =
                serde_json::from_slice(&members_bytes).map_err(|error| {
                    CollectionError::UnsupportedVersion(format!(
                        "generation members are malformed: {error}"
                    ))
                })?;
            if let Some(member) = members.iter().find(|member| member.pack_id == pack_id) {
                history.push(CollectionInstalledHistory {
                    generation_id: generation_id.clone(),
                    content_id: content_id.clone(),
                    canonical_sha256: canonical_sha256.clone(),
                    local_revision: u64::try_from(revision).map_err(|_| {
                        CollectionError::UnsupportedVersion(
                            "generation local revision is negative".into(),
                        )
                    })?,
                    release_sha256: member.release_sha256.clone(),
                    payload_kind: member.payload_kind.clone(),
                    current: current_generation.as_deref() == Some(generation_id.as_str()),
                });
            }
        }
        Ok(history)
    }

    /// Returns the current installed Unified generation, if one exists.
    ///
    /// This only decodes its recorded identity and path; it does not rehash or
    /// verify the store. Use the dictionary verification API for that boundary.
    ///
    /// # Errors
    ///
    /// Returns a typed catalogue error when the current generation record is
    /// malformed or cannot be read.
    pub fn generation(&self) -> Result<Option<CollectionGeneration>, CollectionError> {
        read_generation(&self.connection, &self.root)
    }

    /// Returns retained typed attribution without inventing missing payload rows.
    ///
    /// # Errors
    ///
    /// Returns an invalid-request error if the named release was never registered.
    pub fn reference_metadata(
        &self,
        pack: PackId,
        release_sha256: &str,
    ) -> Result<CollectionReferenceMetadata, CollectionError> {
        let (payload_kind, bytes): (String, Vec<u8>) = self.connection.query_row(
            "SELECT payload_kind,descriptor_json FROM registrations WHERE pack_id=?1 AND release_sha256=?2",
            params![pack.as_str(), release_sha256],
            |row| Ok((row.get(0)?, row.get(1)?)),
        ).optional().map_err(|error| db_error(&self.root, error))?
          .ok_or_else(|| CollectionError::InvalidRequest(format!("no registered {} release {release_sha256}", pack.as_str())))?;
        let descriptor: CollectionReferenceMetadata =
            serde_json::from_slice(&bytes).map_err(|error| {
                CollectionError::UnsupportedVersion(format!(
                    "reference descriptor is malformed: {error}"
                ))
            })?;
        if descriptor.version != 1
            || descriptor.pack_id != pack.as_str()
            || descriptor.release_sha256 != release_sha256
            || descriptor.payload_kind != payload_kind
        {
            return Err(CollectionError::UnsupportedVersion(
                "reference descriptor identity does not match its catalogue key".into(),
            ));
        }
        Ok(descriptor)
    }

    /// Registers selected source metadata without copying release payloads.
    ///
    /// # Errors
    ///
    /// Returns typed request, input, identity, filesystem or transaction errors.
    pub fn register(
        root: impl AsRef<Path>,
        request: CollectionRequest<'_>,
    ) -> Result<CollectionMutationReport, CollectionError> {
        mutate_register(root.as_ref(), request)
    }

    /// Installs selected packs additively into one new verified Unified generation.
    ///
    /// # Errors
    ///
    /// Returns typed request, input, identity, store or transaction errors.
    pub fn install(
        root: impl AsRef<Path>,
        request: CollectionRequest<'_>,
    ) -> Result<CollectionMutationReport, CollectionError> {
        mutate_install(root.as_ref(), request)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct InstalledMember {
    pack_id: String,
    release_sha256: String,
    payload_kind: String,
    release_path: String,
    seal_path: String,
    report_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct StageMarker {
    version: u32,
    attempt_id: String,
    operation: String,
}

#[derive(Default, Clone)]
struct MutationErrorContext {
    attempt_id: Option<String>,
    prior_local_revision: Option<u64>,
    proposed_generation: Option<String>,
    observed_generation: Option<String>,
    inputs: Vec<CollectionInputDisposition>,
}

fn mutation_context(error: CollectionError, context: MutationErrorContext) -> CollectionError {
    CollectionError::MutationContext {
        source: Box::new(error),
        attempt_id: context.attempt_id,
        prior_local_revision: context.prior_local_revision,
        proposed_generation: context.proposed_generation,
        observed_generation: context.observed_generation,
        inputs: context.inputs,
    }
}

fn mutate_register(
    root: &Path,
    request: CollectionRequest<'_>,
) -> Result<CollectionMutationReport, CollectionError> {
    validate_selection(&request.inputs, &request.selection)?;
    let cwd = operation_cwd(root, &request)?;
    let root = absolute_path_with_cwd(root, cwd.as_deref())?;
    let input_ids = request.inputs.iter().map(input_pack_id).collect::<Vec<_>>();
    let selected_ids = selected_pack_ids(&request.inputs, &request.selection);
    let normalized = normalize_selected(request, cwd.as_deref())?;
    let mut error_context = MutationErrorContext {
        inputs: input_dispositions(&input_ids, &selected_ids, &normalized),
        ..MutationErrorContext::default()
    };
    let (connection, newly_created) =
        writer_connection(&root).map_err(|error| mutation_context(error, error_context.clone()))?;
    if newly_created {
        create_private_dir(&root.join("generations"))
            .map_err(|error| mutation_context(error, error_context.clone()))?;
        crate::collections_schema::sync_dir(&root)
            .map_err(|error| mutation_context(error, error_context.clone()))?;
    }
    let mut transaction = TransactionGuard::begin(&connection, &root)
        .map_err(|error| mutation_context(error, error_context.clone()))?;
    let result = (|| -> Result<CollectionMutationReport, CollectionError> {
        let prior_revision = crate::collections_schema::local_revision(&connection, &root)?;
        error_context.prior_local_revision = Some(prior_revision);
        let mut changed = false;
        let mut completed_attempt = None;
        for pack in &normalized {
            let digest = release_sha256(&pack.release);
            let kind = payload_kind_name(pack.seal.payload_kind);
            let descriptor = descriptor_bytes(pack)?;
            let summary = registration_summary_bytes(&descriptor)?;
            let locations = serde_json::to_vec(
                &pack
                    .input_paths
                    .iter()
                    .map(|path| native_path_report(path))
                    .collect::<Vec<_>>(),
            )
            .map_err(|error| CollectionError::InvalidRequest(error.to_string()))?;
            let old: Option<(String, Vec<u8>, Vec<u8>)> = connection.query_row(
            "SELECT payload_kind,seal_json,descriptor_json FROM registrations WHERE pack_id=?1 AND release_sha256=?2",
            params![pack.pack_id.as_str(), digest],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        ).optional().map_err(|error| db_error(&root, error))?;
            match old {
                Some((old_kind, old_seal, old_descriptor)) => {
                    let old_seal = PackSeal::from_json_bytes(&old_seal)
                        .map_err(|error| CollectionError::UnsupportedVersion(error.to_string()))?;
                    if old_kind != kind || old_seal != pack.seal || old_descriptor != descriptor {
                        return Err(CollectionError::IdentityConflict {
                            category: "registration".into(),
                            key: format!("{}:{digest}", pack.pack_id),
                        });
                    }
                    let inserted = connection.execute(
                    "INSERT OR IGNORE INTO registration_locations(pack_id,release_sha256,input_paths_json) VALUES(?1,?2,?3)",
                    params![pack.pack_id.as_str(), digest, locations],
                ).map_err(|error| db_error(&root, error))?;
                    changed |= inserted > 0;
                }
                None => {
                    crate::collections_schema::insert_registration(
                        &connection,
                        crate::collections_schema::RegistrationInsert {
                            pack_id: pack.pack_id.as_str(),
                            release_sha256: &digest,
                            payload_kind: kind,
                            seal_json: &pack.seal_bytes,
                            descriptor_json: &descriptor,
                            summary_json: &summary,
                            input_paths_json: &locations,
                        },
                    )
                    .map_err(|error| db_error(&root, error))?;
                    changed = true;
                }
            }
        }
        let revision = if changed {
            prior_revision.checked_add(1).ok_or_else(|| {
                CollectionError::UnsupportedVersion(
                    "catalogue local_revision overflowed u64".into(),
                )
            })?
        } else {
            prior_revision
        };
        if changed {
            let prior_sql = i64::try_from(prior_revision).map_err(|_| {
                CollectionError::UnsupportedVersion(
                    "catalogue local_revision exceeds SQLite INTEGER range".into(),
                )
            })?;
            crate::collections_schema::update_revision(&connection, revision)
                .map_err(|error| db_error(&root, error))?;
            let id = attempt_id(&root, "register", revision);
            error_context.attempt_id = Some(id.clone());
            connection.execute("INSERT INTO attempts(attempt_id,operation,status,prior_revision,detail) VALUES(?1,'register','committed',?2,'metadata-only registration')", params![id, prior_sql])
            .map_err(|error| db_error(&root, error))?;
            completed_attempt = Some(id);
        }
        // Resolve all report reads before commit so no post-commit read can turn a
        // completed registration into an apparent failed operation.
        let generation = read_generation(&connection, &root)?;
        error_context.observed_generation =
            generation.as_ref().map(|item| item.generation_id.clone());
        Ok(CollectionMutationReport {
            version: 1,
            ok: true,
            operation: "register".into(),
            outcome: if changed { "registered" } else { "idempotent" }.into(),
            local_revision: revision,
            policy_state: "unconfigured".into(),
            attempt_id: completed_attempt,
            generation,
            inputs: error_context.inputs.clone(),
        })
    })();
    match result {
        Ok(report) => match transaction.commit() {
            Ok(()) => Ok(report),
            Err(error) => Err(mutation_context(error, error_context)),
        },
        Err(error) => match transaction.rollback() {
            Ok(()) => Err(mutation_context(error, error_context)),
            Err(rollback) => Err(mutation_context(
                CollectionError::IndeterminateSources {
                    detail: "operation failed and rollback could not be confirmed".into(),
                    source: Some(Box::new(error)),
                    rollback_source: Some(Box::new(rollback)),
                },
                error_context,
            )),
        },
    }
}

fn mutate_install(
    root: &Path,
    request: CollectionRequest<'_>,
) -> Result<CollectionMutationReport, CollectionError> {
    validate_selection(&request.inputs, &request.selection)?;
    let cwd = operation_cwd(root, &request)?;
    let root = absolute_path_with_cwd(root, cwd.as_deref())?;
    let input_ids = request.inputs.iter().map(input_pack_id).collect::<Vec<_>>();
    let selected_ids = selected_pack_ids(&request.inputs, &request.selection);
    let normalized = normalize_selected(request, cwd.as_deref())?;
    let mut error_context = MutationErrorContext {
        inputs: input_dispositions(&input_ids, &selected_ids, &normalized),
        ..MutationErrorContext::default()
    };
    let (connection, newly_created) =
        writer_connection(&root).map_err(|error| mutation_context(error, error_context.clone()))?;
    if newly_created {
        create_private_dir(&root.join("generations"))
            .map_err(|error| mutation_context(error, error_context.clone()))?;
        crate::collections_schema::sync_dir(&root)
            .map_err(|error| mutation_context(error, error_context.clone()))?;
    }
    let mut transaction = TransactionGuard::begin(&connection, &root)
        .map_err(|error| mutation_context(error, error_context.clone()))?;
    let result = (|| -> Result<CollectionMutationReport, CollectionError> {
        let prior_revision = crate::collections_schema::local_revision(&connection, &root)?;
        error_context.prior_local_revision = Some(prior_revision);
        let revision = prior_revision.checked_add(1).ok_or_else(|| {
            CollectionError::UnsupportedVersion("catalogue local_revision overflowed u64".into())
        })?;
        let prior_sql = i64::try_from(prior_revision).map_err(|_| {
            CollectionError::UnsupportedVersion(
                "catalogue local_revision exceeds SQLite INTEGER range".into(),
            )
        })?;
        let revision_sql = i64::try_from(revision).map_err(|_| {
            CollectionError::UnsupportedVersion(
                "next local_revision exceeds SQLite INTEGER range".into(),
            )
        })?;
        let previous_generation = current_members(&connection, &root)?;
        error_context.observed_generation =
            read_generation(&connection, &root)?.map(|item| item.generation_id);
        let mut members = previous_generation.clone();
        let mut new_packs = std::collections::BTreeMap::new();
        let mut dispositions = Vec::new();
        for pack in normalized {
            let digest = release_sha256(&pack.release);
            if let Some(existing) = members
                .iter()
                .find(|member| member.pack_id == pack.pack_id.as_str())
            {
                if existing.release_sha256 != digest {
                    return Err(CollectionError::UpdateRequired(pack.pack_id));
                }
                if existing.payload_kind != payload_kind_name(pack.seal.payload_kind) {
                    return Err(CollectionError::IdentityConflict {
                        category: "payload kind".into(),
                        key: format!("{}:{digest}", pack.pack_id),
                    });
                }
                let existing_release = fs::read(root.join(&existing.release_path))
                    .map_err(|error| artifact_error(root.join(&existing.release_path), error))?;
                let existing_seal = fs::read(root.join(&existing.seal_path))
                    .map_err(|error| artifact_error(root.join(&existing.seal_path), error))?;
                let old_seal =
                    PackSeal::from_json_bytes(&existing_seal).map_err(verification_error)?;
                if existing_release != pack.release || old_seal != pack.seal {
                    return Err(CollectionError::IdentityConflict {
                        category: "installed release/seal".into(),
                        key: format!("{}:{digest}", pack.pack_id),
                    });
                }
                dispositions.push(disposition(&pack, "validated"));
            } else {
                new_packs.insert(pack.pack_id.as_str().to_owned(), pack);
            }
        }
        if new_packs.is_empty() && !members.is_empty() {
            let generation = read_generation(&connection, &root)?;
            let store_path = generation
                .as_ref()
                .map(|item| item.store_path.clone())
                .ok_or_else(|| {
                    CollectionError::VerificationFailed("installed heads have no generation".into())
                })?;
            let mut dictionaries = Vec::new();
            for member in &members {
                let release_path = root.join(&member.release_path);
                let seal_path = root.join(&member.seal_path);
                let release = fs::read(&release_path)
                    .map_err(|error| artifact_error(release_path.clone(), error))?;
                let seal = fs::read(&seal_path)
                    .map_err(|error| artifact_error(seal_path.clone(), error))?;
                dictionaries.push(
                    load_sealed_release(parse_pack_id(&member.pack_id)?, &release, &seal)
                        .map_err(verification_error)?,
                );
            }
            let composed =
                vlajmaji_lexicon::compose_dictionaries(dictionaries).map_err(composition_error)?;
            let canonical = composed.to_json_bytes().map_err(verification_error)?;
            if generation
                .as_ref()
                .is_none_or(|item| release_sha256(&canonical) != item.canonical_sha256)
            {
                return Err(CollectionError::VerificationFailed(
                    "existing generation canonical digest changed".into(),
                ));
            }
            let opened = crate::DictionaryStore::open(&store_path).map_err(verification_error)?;
            if opened.recorded_state_sha256()
                != generation
                    .as_ref()
                    .map_or("", |item| item.canonical_sha256.as_str())
            {
                return Err(CollectionError::VerificationFailed(
                    "existing store header digest differs from catalogue canonical digest".into(),
                ));
            }
            drop(opened);
            let verification = crate::DictionaryStoreVerification::verify(
                &store_path,
                crate::DictionaryStoreVerificationCheck::All,
            )
            .map_err(verification_error)?;
            if !verification.ok {
                return Err(CollectionError::VerificationFailed(
                    "existing generation failed All verification".into(),
                ));
            }
            return Ok(CollectionMutationReport {
                version: 1,
                ok: true,
                operation: "install".into(),
                outcome: "idempotent".into(),
                local_revision: prior_revision,
                policy_state: "unconfigured".into(),
                attempt_id: None,
                generation,
                inputs: error_context.inputs.clone(),
            });
        }

        let catalogue_id = crate::collections_schema::catalogue_identity(&connection)
            .map_err(|error| db_error(&root, error))?;
        let (attempt, generation_id, stage) =
            allocate_stage(&root, revision, &catalogue_id, attempt_id)?;
        error_context.attempt_id = Some(attempt.clone());
        error_context.proposed_generation = Some(generation_id.clone());
        #[cfg(test)]
        if TEST_INTERRUPT_AFTER_STAGE_ALLOCATION.with(|interrupt| interrupt.replace(false)) {
            return Err(CollectionError::InvalidRequest(
                "injected interruption after stage allocation".into(),
            ));
        }
        let relative_stage = PathBuf::from("generations").join(&attempt);
        write_new_file(
            &stage.join("attempt.json"),
            &serde_json::to_vec(&StageMarker {
                version: 1,
                attempt_id: attempt.clone(),
                operation: "install".into(),
            })
            .map_err(|error| CollectionError::InvalidRequest(error.to_string()))?,
        )?;
        create_private_dir(&stage.join("inputs"))?;
        for (pack_id, pack) in &new_packs {
            let input_dir = stage.join("inputs").join(pack_id);
            create_private_dir(&input_dir)?;
            write_new_file(&input_dir.join("release.json"), &pack.release)?;
            write_new_file(&input_dir.join("seal.json"), &pack.seal_bytes)?;
            if let Some(report) = &pack.report {
                let bytes = serde_json::to_vec(report).map_err(|error| {
                    CollectionError::ArtifactIoSource {
                        path: Some(input_dir.join("report.json")),
                        source: Box::new(error),
                    }
                })?;
                write_new_file(&input_dir.join("report.json"), &bytes)?;
            }
            crate::collections_schema::sync_dir(&input_dir)?;
            let prefix = relative_stage.join("inputs").join(pack_id);
            members.push(InstalledMember {
                pack_id: pack_id.clone(),
                release_sha256: release_sha256(&pack.release),
                payload_kind: payload_kind_name(pack.seal.payload_kind).into(),
                release_path: prefix.join("release.json").to_string_lossy().into_owned(),
                seal_path: prefix.join("seal.json").to_string_lossy().into_owned(),
                report_path: pack
                    .report
                    .as_ref()
                    .map(|_| prefix.join("report.json").to_string_lossy().into_owned()),
            });
            let descriptor = descriptor_bytes(pack)?;
            let summary = registration_summary_bytes(&descriptor)?;
            let locations = serde_json::to_vec(
                &pack
                    .input_paths
                    .iter()
                    .map(|path| native_path_report(path))
                    .collect::<Vec<_>>(),
            )
            .map_err(|error| CollectionError::InvalidRequest(error.to_string()))?;
            let existing: Option<(String, Vec<u8>, Vec<u8>)> = connection.query_row(
            "SELECT payload_kind,seal_json,descriptor_json FROM registrations WHERE pack_id=?1 AND release_sha256=?2",
            params![pack_id, release_sha256(&pack.release)],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        ).optional().map_err(|error| db_error(&root, error))?;
            match existing {
                Some((kind, seal, old_descriptor))
                    if kind == payload_kind_name(pack.seal.payload_kind)
                        && PackSeal::from_json_bytes(&seal).ok().as_ref() == Some(&pack.seal)
                        && old_descriptor == descriptor =>
                {
                    connection.execute("INSERT OR IGNORE INTO registration_locations(pack_id,release_sha256,input_paths_json) VALUES(?1,?2,?3)", params![pack_id, release_sha256(&pack.release), locations]).map_err(|error| db_error(&root, error))?;
                }
                Some(_) => {
                    return Err(CollectionError::IdentityConflict {
                        category: "registration".into(),
                        key: format!("{pack_id}:{}", release_sha256(&pack.release)),
                    });
                }
                None => crate::collections_schema::insert_registration(
                    &connection,
                    crate::collections_schema::RegistrationInsert {
                        pack_id,
                        release_sha256: &release_sha256(&pack.release),
                        payload_kind: payload_kind_name(pack.seal.payload_kind),
                        seal_json: &pack.seal_bytes,
                        descriptor_json: &descriptor,
                        summary_json: &summary,
                        input_paths_json: &locations,
                    },
                )
                .map_err(|error| db_error(&root, error))?,
            };
        }
        members.sort_by_key(|member| pack_order(&member.pack_id));
        let new_dispositions = new_packs
            .values()
            .map(|pack| disposition(pack, "installed"))
            .collect::<Vec<_>>();
        // Inputs have been sealed and copied into the owned stage. Release the
        // adapter's typed models before loading the complete composing set.
        drop(new_packs);
        let mut dictionaries = Vec::new();
        for member in &members {
            let release_path = root.join(&member.release_path);
            let seal_path = root.join(&member.seal_path);
            let release = fs::read(&release_path)
                .map_err(|error| artifact_error(release_path.clone(), error))?;
            let seal =
                fs::read(&seal_path).map_err(|error| artifact_error(seal_path.clone(), error))?;
            dictionaries.push(
                load_sealed_release(parse_pack_id(&member.pack_id)?, &release, &seal)
                    .map_err(verification_error)?,
            );
        }
        let composed =
            vlajmaji_lexicon::compose_dictionaries(dictionaries).map_err(composition_error)?;
        let canonical =
            composed
                .to_json_bytes()
                .map_err(|error| CollectionError::InputInvalid {
                    pack: None,
                    detail: error.to_string(),
                })?;
        let canonical_sha256 = release_sha256(&canonical);
        let content_id = content_identity(&members)?;
        let store_path = stage.join("store");
        let created_store =
            crate::DictionaryStore::create(&store_path, &composed).map_err(verification_error)?;
        if created_store.recorded_state_sha256() != canonical_sha256 {
            drop(created_store);
            return Err(CollectionError::VerificationFailed(
                "created Unified store digest differs from composed release".into(),
            ));
        }
        drop(created_store);
        set_private_file(&store_path.join("dictionary.sqlite3"))?;
        set_private_directory(&store_path)?;
        crate::collections_schema::sync_file(&store_path.join("dictionary.sqlite3"))?;
        crate::collections_schema::sync_dir(&store_path)?;
        let verification = crate::DictionaryStoreVerification::verify(
            &store_path,
            crate::DictionaryStoreVerificationCheck::All,
        )
        .map_err(verification_error)?;
        if !verification.ok {
            return Err(CollectionError::VerificationFailed(
                "new Unified store did not pass All verification".into(),
            ));
        }
        write_new_file(
            &stage.join("verified.json"),
            &serde_json::to_vec(&verification).map_err(|error| {
                CollectionError::ArtifactIoSource {
                    path: Some(stage.join("verified.json")),
                    source: Box::new(error),
                }
            })?,
        )?;
        crate::collections_schema::sync_dir(&stage.join("inputs"))?;
        crate::collections_schema::sync_dir(&stage)?;
        let members_bytes =
            serde_json::to_vec(&members).map_err(|error| CollectionError::ArtifactIoSource {
                path: None,
                source: Box::new(error),
            })?;
        connection.execute(
        "INSERT INTO generations(generation_id,content_id,canonical_sha256,store_path,members_json,local_revision) VALUES(?1,?2,?3,?4,?5,?6)",
        params![generation_id, content_id, canonical_sha256, native_path_bytes(&store_path), members_bytes, revision_sql],
    ).map_err(|error| db_error(&root, error))?;
        connection
            .execute("DELETE FROM installed_heads", [])
            .map_err(|error| db_error(&root, error))?;
        for member in &members {
            connection.execute("INSERT INTO installed_heads(pack_id,release_sha256,payload_kind,generation_id) VALUES(?1,?2,?3,?4)", params![member.pack_id, member.release_sha256, member.payload_kind, generation_id])
            .map_err(|error| db_error(&root, error))?;
        }
        crate::collections_schema::update_revision(&connection, revision)
            .map_err(|error| db_error(&root, error))?;
        connection.execute("INSERT INTO attempts(attempt_id,operation,status,prior_revision,detail) VALUES(?1,'install','committed',?2,'Unified store passed All verification')", params![attempt, prior_sql])
        .map_err(|error| db_error(&root, error))?;
        // Make all owned directory entries durable before the SQLite commit. The
        // catalogue commit itself publishes the already-complete stage.
        crate::collections_schema::sync_dir(&root.join("generations"))?;
        let generation = Some(CollectionGeneration {
            generation_id: generation_id.clone(),
            content_id: content_id.clone(),
            canonical_sha256: canonical_sha256.clone(),
            path: native_path_report(&store_path),
            store_path,
            members: members
                .iter()
                .map(|member| CollectionGenerationMember {
                    pack_id: member.pack_id.clone(),
                    release_sha256: member.release_sha256.clone(),
                    payload_kind: member.payload_kind.clone(),
                })
                .collect(),
        });
        let completed_by_id = new_dispositions
            .iter()
            .chain(&dispositions)
            .map(|input| (input.pack_id.as_str(), input))
            .collect::<std::collections::BTreeMap<_, _>>();
        let inputs = error_context
            .inputs
            .iter()
            .map(|input| {
                completed_by_id.get(input.pack_id.as_str()).map_or_else(
                    || input.clone(),
                    |completed| CollectionInputDisposition {
                        disposition: completed.disposition.clone(),
                        ..input.clone()
                    },
                )
            })
            .collect();
        Ok(CollectionMutationReport {
            version: 1,
            ok: true,
            operation: "install".into(),
            outcome: "installed".into(),
            local_revision: revision,
            policy_state: "unconfigured".into(),
            attempt_id: Some(attempt),
            generation,
            inputs,
        })
    })();
    match result {
        Ok(report) if report.outcome == "idempotent" => match transaction.rollback() {
            Ok(()) => Ok(report),
            Err(error) => Err(mutation_context(error, error_context)),
        },
        Ok(report) => match transaction.commit() {
            Ok(()) => Ok(report),
            Err(error) => {
                if matches!(
                    error,
                    CollectionError::Indeterminate(_)
                        | CollectionError::IndeterminateSources { .. }
                ) {
                    error_context.observed_generation = None;
                }
                Err(mutation_context(error, error_context))
            }
        },
        Err(error) => match transaction.rollback() {
            Ok(()) => Err(mutation_context(error, error_context)),
            Err(rollback) => Err(mutation_context(
                CollectionError::IndeterminateSources {
                    detail: "operation failed and rollback could not be confirmed".into(),
                    source: Some(Box::new(error)),
                    rollback_source: Some(Box::new(rollback)),
                },
                MutationErrorContext {
                    observed_generation: None,
                    ..error_context
                },
            )),
        },
    }
}

fn writer_connection(root: &Path) -> Result<(Connection, bool), CollectionError> {
    match fs::symlink_metadata(root) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            Ok((crate::collections_schema::initialize_exclusive(root)?, true))
        }
        Err(error) => Err(artifact_error(root.to_path_buf(), error)),
        Ok(_) => match crate::collections_schema::open_writable(root) {
            Ok(connection) => Ok((connection, false)),
            Err(CollectionError::MissingCatalogue(_)) => {
                Err(CollectionError::OccupiedDestination(root.to_path_buf()))
            }
            Err(error) => Err(error),
        },
    }
}

fn normalize_selected(
    request: CollectionRequest<'_>,
    cwd: Option<&Path>,
) -> Result<Vec<NormalizedPack>, CollectionError> {
    validate_selection(&request.inputs, &request.selection)?;
    let all_ids = request.inputs.iter().map(input_pack_id).collect::<Vec<_>>();
    let selected: std::collections::BTreeSet<PackId> = match request.selection {
        CollectionSelection::All => [
            PackId::LensiskuJbotci,
            PackId::Llg,
            PackId::LaLojban,
            PackId::Sutysisku,
        ]
        .into_iter()
        .collect(),
        CollectionSelection::Packs(ids) => ids.into_iter().collect(),
    };
    let selected_inputs = request
        .inputs
        .into_iter()
        .filter(|input| selected.contains(&input_pack_id(input)))
        .collect::<Vec<_>>();
    let mut inputs = selected_inputs
        .into_iter()
        .map(|input| freeze_input_paths(input, cwd.unwrap_or_else(|| Path::new(""))))
        .collect::<Vec<_>>();
    inputs.sort_by_key(input_pack_id);
    let ordered_ids = inputs.iter().map(input_pack_id).collect::<Vec<_>>();
    let mut normalized = Vec::with_capacity(inputs.len());
    for (index, input) in inputs.iter().enumerate() {
        match normalize_input(input, request.allow_synthetic) {
            Ok(pack) => normalized.push(pack),
            Err(source) => {
                let failed_pack = ordered_ids[index];
                let dispositions = all_ids
                    .iter()
                    .map(|id| {
                        if !selected.contains(id) {
                            CollectionInputDisposition {
                                pack_id: id.as_str().into(),
                                disposition: "skipped_without_io".into(),
                                payload_kind: None,
                                release_sha256: None,
                                report: None,
                            }
                        } else if *id == failed_pack {
                            CollectionInputDisposition {
                                pack_id: id.as_str().into(),
                                disposition: "failed".into(),
                                payload_kind: None,
                                release_sha256: None,
                                report: None,
                            }
                        } else if let Some(pack) =
                            normalized.iter().find(|pack| pack.pack_id == *id)
                        {
                            disposition(pack, "validated")
                        } else {
                            CollectionInputDisposition {
                                pack_id: id.as_str().into(),
                                disposition: "unprocessed".into(),
                                payload_kind: None,
                                release_sha256: None,
                                report: None,
                            }
                        }
                    })
                    .collect();
                return Err(CollectionError::Batch {
                    source: Box::new(source),
                    inputs: dispositions,
                });
            }
        }
    }
    Ok(normalized)
}

fn selected_pack_ids(
    inputs: &[CollectionInput<'_>],
    selection: &CollectionSelection,
) -> std::collections::BTreeSet<PackId> {
    match selection {
        CollectionSelection::All => inputs.iter().map(input_pack_id).collect(),
        CollectionSelection::Packs(ids) => ids.iter().copied().collect(),
    }
}

fn input_dispositions(
    input_ids: &[PackId],
    selected_ids: &std::collections::BTreeSet<PackId>,
    normalized: &[NormalizedPack],
) -> Vec<CollectionInputDisposition> {
    input_ids
        .iter()
        .map(|id| {
            normalized
                .iter()
                .find(|pack| pack.pack_id == *id)
                .map(|pack| disposition(pack, "validated"))
                .unwrap_or_else(|| CollectionInputDisposition {
                    pack_id: id.as_str().into(),
                    disposition: if selected_ids.contains(id) {
                        "unprocessed"
                    } else {
                        "skipped_without_io"
                    }
                    .into(),
                    payload_kind: None,
                    release_sha256: None,
                    report: None,
                })
        })
        .collect()
}

fn freeze_input_paths(input: CollectionInput<'_>, cwd: &Path) -> CollectionInput<'static> {
    let absolute = |path: PathBuf| {
        if path.is_absolute() {
            path
        } else {
            cwd.join(path)
        }
    };
    match input {
        CollectionInput::SealedFiles {
            pack_id,
            release,
            seal,
        } => CollectionInput::SealedFiles {
            pack_id,
            release: absolute(release),
            seal: absolute(seal),
        },
        CollectionInput::SealedBytes {
            pack_id,
            release,
            seal,
        } => CollectionInput::SealedBytes {
            pack_id,
            release: Cow::Owned(release.into_owned()),
            seal: Cow::Owned(seal.into_owned()),
        },
        CollectionInput::Jbotci {
            payload_kind,
            raw,
            metadata,
            options,
        } => CollectionInput::Jbotci {
            payload_kind,
            raw: absolute(raw),
            metadata: absolute(metadata),
            options,
        },
        CollectionInput::Llg {
            payload_kind,
            gismu,
            cmavo,
            rafsi,
            notice,
            options,
        } => CollectionInput::Llg {
            payload_kind,
            gismu: absolute(gismu),
            cmavo: absolute(cmavo),
            rafsi: absolute(rafsi),
            notice: absolute(notice),
            options,
        },
        CollectionInput::LaLojban {
            payload_kind,
            raw,
            options,
        } => CollectionInput::LaLojban {
            payload_kind,
            raw: absolute(raw),
            options,
        },
        CollectionInput::Sutysisku {
            payload_kind,
            raw,
            options,
        } => CollectionInput::Sutysisku {
            payload_kind,
            raw: absolute(raw),
            options,
        },
    }
}

fn absolute_path(path: &Path) -> Result<PathBuf, CollectionError> {
    absolute_path_with_cwd(path, None)
}

fn absolute_path_with_cwd(path: &Path, cwd: Option<&Path>) -> Result<PathBuf, CollectionError> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else if let Some(cwd) = cwd {
        Ok(cwd.join(path))
    } else {
        std::env::current_dir()
            .map(|cwd| cwd.join(path))
            .map_err(|error| artifact_error(path.to_path_buf(), error))
    }
}

fn operation_cwd(
    root: &Path,
    request: &CollectionRequest<'_>,
) -> Result<Option<PathBuf>, CollectionError> {
    validate_selection(&request.inputs, &request.selection)?;
    let selected = match &request.selection {
        CollectionSelection::All => request.inputs.iter().collect::<Vec<_>>(),
        CollectionSelection::Packs(ids) => request
            .inputs
            .iter()
            .filter(|input| ids.contains(&input_pack_id(input)))
            .collect(),
    };
    if !root.is_absolute() || selected.iter().any(|input| input_has_relative_paths(input)) {
        std::env::current_dir()
            .map(Some)
            .map_err(|error| artifact_error(PathBuf::from("."), error))
    } else {
        Ok(None)
    }
}

fn current_members(
    connection: &Connection,
    root: &Path,
) -> Result<Vec<InstalledMember>, CollectionError> {
    let bytes: Option<Vec<u8>> = connection
        .query_row(
            "SELECT members_json FROM generations ORDER BY local_revision DESC LIMIT 1",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| db_error(root, error))?;
    bytes
        .map(|bytes| {
            serde_json::from_slice(&bytes).map_err(|error| {
                CollectionError::UnsupportedVersion(format!(
                    "generation member list is malformed: {error}"
                ))
            })
        })
        .transpose()
        .map(Option::unwrap_or_default)
}

fn orphan_stage_attempts(
    root: &Path,
    connection: &Connection,
) -> Result<Vec<String>, CollectionError> {
    let committed = {
        let mut statement = connection
            .prepare("SELECT attempt_id FROM attempts WHERE status='committed'")
            .map_err(|error| db_error(root, error))?;
        statement
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|error| db_error(root, error))?
            .collect::<Result<std::collections::BTreeSet<_>, _>>()
            .map_err(|error| db_error(root, error))?
    };
    let directory = root.join("generations");
    let entries = match fs::read_dir(&directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(artifact_error(directory, error)),
    };
    let mut orphaned = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| artifact_error(directory.clone(), error))?;
        let path = entry.path();
        let metadata =
            fs::symlink_metadata(&path).map_err(|error| artifact_error(path.clone(), error))?;
        if !metadata.is_dir() || metadata.file_type().is_symlink() {
            continue;
        }
        let marker_path = path.join("attempt.json");
        let marker_metadata = match fs::symlink_metadata(&marker_path) {
            Ok(metadata) if metadata.is_file() && !metadata.file_type().is_symlink() => metadata,
            Ok(_) => {
                orphaned.push(format!(
                    "unreadable:{}",
                    native_path_report(Path::new(&entry.file_name())).hex
                ));
                continue;
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                orphaned.push(format!(
                    "unmarked:{}",
                    native_path_report(Path::new(&entry.file_name())).hex
                ));
                continue;
            }
            Err(error) => return Err(artifact_error(marker_path, error)),
        };
        let _ = marker_metadata;
        let bytes =
            fs::read(&marker_path).map_err(|error| artifact_error(marker_path.clone(), error))?;
        let marker: StageMarker = match serde_json::from_slice::<StageMarker>(&bytes) {
            Ok(marker) if marker.version == 1 && marker.operation == "install" => marker,
            _ => {
                orphaned.push(format!(
                    "unreadable:{}",
                    native_path_report(Path::new(&entry.file_name())).hex
                ));
                continue;
            }
        };
        if !committed.contains(&marker.attempt_id) {
            orphaned.push(marker.attempt_id);
        }
    }
    Ok(orphaned)
}

fn native_report_path(report: &NativePathReport) -> Option<PathBuf> {
    let mut bytes = Vec::with_capacity(report.hex.len() / 2);
    let raw = report.hex.as_bytes();
    if !raw.len().is_multiple_of(2) {
        return None;
    }
    for pair in raw.as_chunks::<2>().0 {
        let text = std::str::from_utf8(pair).ok()?;
        bytes.push(u8::from_str_radix(text, 16).ok()?);
    }
    native_path_from_bytes(&bytes).ok()
}

fn is_regular_nonsymlink(path: &Path) -> bool {
    fs::symlink_metadata(path)
        .is_ok_and(|metadata| metadata.is_file() && !metadata.file_type().is_symlink())
}

fn owned_member_files_present(root: &Path, connection: &Connection, pack_id: &str) -> bool {
    let Ok(members) = current_members(connection, root) else {
        return false;
    };
    let Some(member) = members.iter().find(|member| member.pack_id == pack_id) else {
        return false;
    };
    let Some(generation) = read_generation(connection, root).ok().flatten() else {
        return false;
    };
    let store_meta = fs::symlink_metadata(&generation.store_path);
    let store_present = store_meta
        .is_ok_and(|metadata| metadata.is_dir() && !metadata.file_type().is_symlink())
        && is_regular_nonsymlink(&generation.store_path.join("dictionary.sqlite3"));
    store_present
        && is_regular_nonsymlink(&root.join(&member.release_path))
        && is_regular_nonsymlink(&root.join(&member.seal_path))
        && member
            .report_path
            .as_ref()
            .is_none_or(|path| is_regular_nonsymlink(&root.join(path)))
}

fn read_generation(
    connection: &Connection,
    root: &Path,
) -> Result<Option<CollectionGeneration>, CollectionError> {
    type Row = (String, String, String, Vec<u8>, Vec<u8>);
    let row: Option<Row> = connection.query_row(
        "SELECT generation_id,content_id,canonical_sha256,store_path,members_json FROM generations ORDER BY local_revision DESC LIMIT 1",
        [],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
    ).optional().map_err(|error| db_error(root, error))?;
    row.map(
        |(generation_id, content_id, canonical_sha256, path_bytes, members_bytes)| {
            let store_path = native_path_from_bytes(&path_bytes)?;
            let members: Vec<InstalledMember> =
                serde_json::from_slice(&members_bytes).map_err(|error| {
                    CollectionError::UnsupportedVersion(format!(
                        "generation members are malformed: {error}"
                    ))
                })?;
            Ok(CollectionGeneration {
                generation_id,
                content_id,
                canonical_sha256,
                path: native_path_report(&store_path),
                store_path,
                members: members
                    .into_iter()
                    .map(|member| CollectionGenerationMember {
                        pack_id: member.pack_id,
                        release_sha256: member.release_sha256,
                        payload_kind: member.payload_kind,
                    })
                    .collect(),
            })
        },
    )
    .transpose()
}

fn descriptor_bytes(pack: &NormalizedPack) -> Result<Vec<u8>, CollectionError> {
    let dictionary = &pack.dictionary;
    let registry = dictionary.registry().as_document().clone();
    let metadata = dictionary.metadata();
    let mut entities = Vec::new();
    let mut body_digests = BTreeMap::new();
    let entry_sources: BTreeMap<_, _> = metadata
        .entries
        .iter()
        .map(|entry| (entry.id.to_string(), entry.source.to_string()))
        .collect();
    let assertion_entries: BTreeMap<_, _> = metadata
        .assertions
        .iter()
        .map(|assertion| (assertion.id.to_string(), assertion.entry.to_string()))
        .collect();
    for row in &metadata.entries {
        let digest = record_digest(row)?;
        let id = row.id.to_string();
        body_digests.insert(format!("entry:{id}"), digest);
        entities.push(CollectionEntityReference {
            role: "entry".into(),
            id,
            entry_ids: Vec::new(),
            assertion_ids: Vec::new(),
            source_ids: vec![row.source.to_string()],
            occurrence: None,
            evidence: None,
            spelling: Some(row.spelling.clone()),
        });
    }
    for row in &metadata.assertions {
        let digest = record_digest(row)?;
        let id = row.id.to_string();
        body_digests.insert(format!("assertion:{id}"), digest);
        entities.push(CollectionEntityReference {
            role: "assertion".into(),
            id,
            entry_ids: vec![row.entry.to_string()],
            assertion_ids: Vec::new(),
            source_ids: entry_sources
                .get(&row.entry.to_string())
                .cloned()
                .into_iter()
                .collect(),
            occurrence: Some(row.occurrence.clone()),
            evidence: None,
            spelling: None,
        });
    }
    for row in &metadata.forms {
        let digest = record_digest(row)?;
        let id = row.id.to_string();
        body_digests.insert(format!("form:{id}"), digest);
        entities.push(CollectionEntityReference {
            role: "form".into(),
            id,
            entry_ids: vec![row.entry.to_string()],
            assertion_ids: Vec::new(),
            source_ids: entry_sources
                .get(&row.entry.to_string())
                .cloned()
                .into_iter()
                .collect(),
            occurrence: row.evidence.as_ref().map(|item| item.occurrence.clone()),
            evidence: row.evidence.clone(),
            spelling: None,
        });
    }
    for row in &metadata.editorial_senses {
        let digest = record_digest(row)?;
        let id = row.id.to_string();
        let assertion_ids = row
            .assertions
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        let entry_ids = row
            .assertions
            .iter()
            .filter_map(|assertion| assertion_entries.get(&assertion.to_string()).cloned())
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let source_ids = entry_ids
            .iter()
            .filter_map(|entry| {
                metadata
                    .entries
                    .iter()
                    .find(|item| item.id.to_string() == *entry)
                    .map(|item| item.source.to_string())
            })
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();
        body_digests.insert(format!("editorial_sense:{id}"), digest);
        entities.push(CollectionEntityReference {
            role: "editorial_sense".into(),
            id,
            entry_ids,
            assertion_ids,
            source_ids,
            occurrence: None,
            evidence: None,
            spelling: None,
        });
    }
    macro_rules! add_keyed_rows {
        ($field:literal, $rows:expr) => {{
            for row in $rows {
                body_digests.insert(format!("{}:{}", $field, row.id), record_digest(row)?);
            }
        }};
    }
    add_keyed_rows!("context", &metadata.contexts);
    add_keyed_rows!("frame", &metadata.frames);
    add_keyed_rows!("etymology", &metadata.etymologies);
    add_keyed_rows!("relation", &metadata.relations);
    add_keyed_rows!("owned_revision", &metadata.owned_revisions);
    add_keyed_rows!("computation", &metadata.computations);
    add_keyed_rows!("annotation", &metadata.annotations);
    let mut extension_digests = Vec::with_capacity(metadata.extensions.len());
    for extension in &metadata.extensions {
        extension_digests.push(record_digest(extension)?);
    }
    for row in &metadata.owned_heads {
        body_digests.insert(format!("owned_head:{}", row.source), record_digest(row)?);
    }
    let captures = dictionary
        .captures()
        .iter()
        .map(|capture| CollectionCaptureReference {
            capture: capture.identity.clone(),
            metadata: capture.metadata.clone(),
            byte_length: capture.bytes.len(),
            occurrences: capture
                .occurrences
                .iter()
                .map(|occurrence| OccurrenceRef {
                    capture: capture.identity.clone(),
                    locator: occurrence.locator.clone(),
                })
                .collect(),
        })
        .collect();
    let descriptor = CollectionReferenceMetadata {
        version: 1,
        pack_id: pack.pack_id.as_str().into(),
        release_sha256: release_sha256(&pack.release),
        payload_kind: payload_kind_name(pack.seal.payload_kind).into(),
        dictionary_version: 1,
        registry_version: registry.version,
        upstream_catalogue_revision: metadata.catalogue_revision,
        registry,
        source_metadata: CollectionSourceMetadata {
            collections: metadata.collections.clone(),
            components: metadata.components.clone(),
            releases: metadata.releases.clone(),
            code_notices: metadata.code_notices.clone(),
        },
        captures,
        entities,
        body_sha256: body_digests,
        extension_row_sha256_ordered_multiset: extension_digests,
        category_counts: CollectionCategoryCounts {
            entries: metadata.entries.len(),
            assertions: metadata.assertions.len(),
            contexts: metadata.contexts.len(),
            frames: metadata.frames.len(),
            forms: metadata.forms.len(),
            etymologies: metadata.etymologies.len(),
            relations: metadata.relations.len(),
            owned_revisions: metadata.owned_revisions.len(),
            owned_heads: metadata.owned_heads.len(),
            editorial_senses: metadata.editorial_senses.len(),
            computations: metadata.computations.len(),
            annotations: metadata.annotations.len(),
            extensions: metadata.extensions.len(),
        },
    };
    serde_json::to_vec(&descriptor).map_err(|error| CollectionError::InputInvalid {
        pack: Some(pack.pack_id),
        detail: error.to_string(),
    })
}

fn record_digest<T: Serialize>(value: &T) -> Result<String, CollectionError> {
    let bytes = serde_json::to_vec(value)
        .map_err(|error| CollectionError::InvalidRequest(error.to_string()))?;
    Ok(release_sha256(&bytes))
}

fn disposition(pack: &NormalizedPack, disposition: &str) -> CollectionInputDisposition {
    CollectionInputDisposition {
        pack_id: pack.pack_id.as_str().into(),
        disposition: disposition.into(),
        payload_kind: Some(payload_kind_name(pack.seal.payload_kind).into()),
        release_sha256: Some(release_sha256(&pack.release)),
        report: pack.report.clone(),
    }
}

fn content_identity(members: &[InstalledMember]) -> Result<String, CollectionError> {
    let mut ordered = members.to_vec();
    ordered.sort_by_key(|member| pack_order(&member.pack_id));
    let pairs: Vec<_> = ordered
        .iter()
        .map(|member| (member.pack_id.as_str(), member.release_sha256.as_str()))
        .collect();
    let mut bytes = b"vlajmaji-collection-content-v1\0".to_vec();
    bytes.extend(
        serde_json::to_vec(&pairs)
            .map_err(|error| CollectionError::InvalidRequest(error.to_string()))?,
    );
    Ok(release_sha256(&bytes))
}

fn payload_kind_name(kind: PackPayloadKind) -> &'static str {
    match kind {
        PackPayloadKind::Full => "full",
        PackPayloadKind::Synthetic => "synthetic",
    }
}
fn pack_order(pack_id: &str) -> u8 {
    match pack_id {
        "lensisku-jbotci" => 0,
        "llg" => 1,
        "la-lojban" => 2,
        "sutysisku" => 3,
        _ => u8::MAX,
    }
}
fn allocate_stage(
    root: &Path,
    revision: u64,
    catalogue_id: &str,
    mut next_attempt: impl FnMut(&Path, &str, u64) -> String,
) -> Result<(String, String, PathBuf), CollectionError> {
    for _ in 0..8 {
        let attempt = next_attempt(root, "install", revision);
        let generation_id = format!("{catalogue_id}-{attempt}");
        let stage = root.join("generations").join(&attempt);
        match try_create_private_dir(&stage) {
            Ok(()) => return Ok((attempt, generation_id, stage)),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(artifact_error(stage, error)),
        }
    }
    Err(CollectionError::OccupiedDestination(
        root.join("generations"),
    ))
}

fn parse_pack_id(value: &str) -> Result<PackId, CollectionError> {
    match value {
        "lensisku-jbotci" => Ok(PackId::LensiskuJbotci),
        "llg" => Ok(PackId::Llg),
        "la-lojban" => Ok(PackId::LaLojban),
        "sutysisku" => Ok(PackId::Sutysisku),
        _ => Err(CollectionError::UnsupportedVersion(format!(
            "unknown pack id {value}"
        ))),
    }
}
fn attempt_id(root: &Path, operation: &str, revision: u64) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let mut digest = Sha256::new();
    digest.update(native_path_bytes(root));
    digest.update(operation.as_bytes());
    digest.update(revision.to_be_bytes());
    digest.update(nanos.to_be_bytes());
    digest.update(std::process::id().to_be_bytes());
    format!("{:x}", digest.finalize())[..32].to_owned()
}
fn native_path_bytes(path: &Path) -> Vec<u8> {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        path.as_os_str().as_bytes().to_vec()
    }
    #[cfg(not(unix))]
    {
        path.to_string_lossy().as_bytes().to_vec()
    }
}
fn native_path_from_bytes(bytes: &[u8]) -> Result<PathBuf, CollectionError> {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStringExt;
        Ok(PathBuf::from(std::ffi::OsString::from_vec(bytes.to_vec())))
    }
    #[cfg(not(unix))]
    {
        String::from_utf8(bytes.to_vec())
            .map(PathBuf::from)
            .map_err(|error| CollectionError::UnsupportedVersion(error.to_string()))
    }
}
fn write_new_file(path: &Path, bytes: &[u8]) -> Result<(), CollectionError> {
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    use std::io::Write as _;
    let mut file = options
        .open(path)
        .map_err(|error| artifact_error(path.to_path_buf(), error))?;
    file.write_all(bytes)
        .and_then(|_| file.sync_all())
        .map_err(|error| artifact_error(path.to_path_buf(), error))
}
fn create_private_dir(path: &Path) -> Result<(), CollectionError> {
    try_create_private_dir(path).map_err(|error| artifact_error(path.to_path_buf(), error))
}
fn try_create_private_dir(path: &Path) -> io::Result<()> {
    let mut builder = fs::DirBuilder::new();
    builder.recursive(false);
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    builder.create(path)
}
fn set_private_directory(path: &Path) -> Result<(), CollectionError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))
            .map_err(|error| artifact_error(path.to_path_buf(), error))?;
    }
    Ok(())
}
fn set_private_file(path: &Path) -> Result<(), CollectionError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))
            .map_err(|error| artifact_error(path.to_path_buf(), error))?;
    }
    Ok(())
}
struct TransactionGuard<'a> {
    connection: &'a Connection,
    root: &'a Path,
    active: bool,
}

impl<'a> TransactionGuard<'a> {
    fn begin(connection: &'a Connection, root: &'a Path) -> Result<Self, CollectionError> {
        connection
            .execute_batch("BEGIN IMMEDIATE")
            .map_err(|error| map_busy(root, error))?;
        Ok(Self {
            connection,
            root,
            active: true,
        })
    }

    fn rollback(&mut self) -> Result<(), CollectionError> {
        if self.active {
            #[cfg(test)]
            if TEST_FAIL_NEXT_ROLLBACK.with(|fail| fail.replace(false)) {
                return Err(CollectionError::IndeterminateSources {
                    detail: format!("injected rollback failure for {}", self.root.display()),
                    source: Some(Box::new(io::Error::other("injected rollback cause"))),
                    rollback_source: None,
                });
            }
            self.connection.execute_batch("ROLLBACK").map_err(|error| {
                CollectionError::IndeterminateSources {
                    detail: format!(
                        "rollback could not be confirmed for {}",
                        self.root.display()
                    ),
                    source: Some(Box::new(error)),
                    rollback_source: None,
                }
            })?;
            self.active = false;
        }
        Ok(())
    }

    fn commit(&mut self) -> Result<(), CollectionError> {
        match self.connection.execute_batch("COMMIT") {
            Ok(()) => {
                self.active = false;
                Ok(())
            }
            Err(error) if matches!(&error, rusqlite::Error::SqliteFailure(code, _) if matches!(code.code, rusqlite::ErrorCode::DatabaseBusy | rusqlite::ErrorCode::DatabaseLocked)) => {
                match self.connection.execute_batch("ROLLBACK") {
                    Ok(()) => {
                        self.active = false;
                        Err(CollectionError::BusySource(error))
                    }
                    Err(rollback) => {
                        self.active = false;
                        Err(CollectionError::IndeterminateSources {
                            detail: format!(
                                "COMMIT failed and rollback could not be confirmed for {}; inspect catalogue activation before retrying",
                                self.root.display()
                            ),
                            source: Some(Box::new(error)),
                            rollback_source: Some(Box::new(rollback)),
                        })
                    }
                }
            }
            Err(error) => {
                self.active = false;
                Err(CollectionError::IndeterminateSources {
                    detail: format!("commit result is uncertain for {}", self.root.display()),
                    source: Some(Box::new(error)),
                    rollback_source: None,
                })
            }
        }
    }
}

impl Drop for TransactionGuard<'_> {
    fn drop(&mut self) {
        if self.active {
            // Every early `?` after BEGIN explicitly attempts SQL rollback; the
            // initiating error remains primary if this best-effort attempt also fails.
            let _ = self.connection.execute_batch("ROLLBACK");
            self.active = false;
        }
    }
}
fn map_busy(root: &Path, error: rusqlite::Error) -> CollectionError {
    if let rusqlite::Error::SqliteFailure(code, _) = &error
        && matches!(
            code.code,
            rusqlite::ErrorCode::DatabaseBusy | rusqlite::ErrorCode::DatabaseLocked
        )
    {
        CollectionError::BusySource(error)
    } else {
        db_error(root, error)
    }
}
fn db_error(root: &Path, error: rusqlite::Error) -> CollectionError {
    CollectionError::ArtifactIoSource {
        path: Some(crate::collections_schema::database_path(root)),
        source: Box::new(error),
    }
}
fn verification_error(error: impl Error + Send + Sync + 'static) -> CollectionError {
    CollectionError::VerificationFailedSource {
        detail: error.to_string(),
        source: Some(Box::new(error)),
    }
}
fn composition_error(error: vlajmaji_lexicon::PackError) -> CollectionError {
    match &error {
        vlajmaji_lexicon::PackError::Conflict { category, key } => CollectionError::PackConflict {
            category: (*category).into(),
            key: key.clone(),
            source: Box::new(error),
        },
        _ => CollectionError::Pack(Box::new(error)),
    }
}
fn artifact_error(path: PathBuf, error: impl Error + Send + Sync + 'static) -> CollectionError {
    CollectionError::ArtifactIoSource {
        path: Some(path),
        source: Box::new(error),
    }
}

/// Read-only summary of a collection catalogue.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CollectionListReport {
    /// Report schema version.
    pub version: u32,
    /// Catalogue-local revision.
    pub local_revision: u64,
    /// Current installed generation, if any.
    pub generation: Option<CollectionGeneration>,
    /// Registered and installed pack summaries.
    pub packs: Vec<CollectionInspection>,
    /// Retained uncommitted attempts and controlled generation children that are unmarked or unreadable.
    pub staged_attempts: Vec<String>,
}

/// Read-only inspection summary for one pack.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CollectionInspection {
    /// Report schema version.
    pub version: u32,
    /// Stable pack ID.
    pub pack_id: String,
    /// Current installed release digest, if installed.
    pub installed_release_sha256: Option<String>,
    /// Immutable generations that contained this pack, newest first.
    pub installed_history: Vec<CollectionInstalledHistory>,
    /// Registration metadata summaries.
    pub registrations: Vec<CollectionRegistrationSummary>,
    /// Whether current owned release/seal/store paths are present.
    pub owned_files_present: Option<bool>,
    /// Recorded verification scope from the last install.
    pub last_verification_scope: Option<String>,
    /// Minimum derived-store header observation, not an All verification.
    pub derived_store_minimum: Option<CollectionDerivedStoreMinimum>,
    /// User selection is not configured by S02.
    pub policy_state: String,
}

/// One retained installation generation containing a pack.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CollectionInstalledHistory {
    /// Physical generation identity.
    pub generation_id: String,
    /// Data-derived content identity.
    pub content_id: String,
    /// Canonical composed dictionary digest.
    pub canonical_sha256: String,
    /// Catalogue-local revision that published the generation.
    pub local_revision: u64,
    /// Pack release digest represented in the generation.
    pub release_sha256: String,
    /// Declared Full or Synthetic scope.
    pub payload_kind: String,
    /// True only for the currently active generation.
    pub current: bool,
}

/// Metadata-only registration summary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CollectionRegistrationSummary {
    /// Sealed release digest.
    pub release_sha256: String,
    /// Declared payload scope.
    pub payload_kind: String,
    /// Whether every recorded original native input path currently resolves.
    pub inputs_present: bool,
    /// Frozen native input paths.
    pub inputs: Vec<NativePathReport>,
    /// Whether registered input bytes remain at caller-managed paths or were embedded/borrowed.
    pub input_ownership: String,
    /// Whether the derived generation contains a tool-owned copy of this pack's sealed inputs.
    pub installed_pack_copy_ownership: String,
    /// Whether a tool-owned derived store currently exists for this registration.
    pub derived_store_ownership: String,
    /// Compatibility is evaluated only against an installed set.
    pub compatibility: String,
    /// Exact registered source identities.
    pub source_ids: Vec<String>,
    /// Source names, abbreviations, evidence and standing.
    pub sources: Vec<SourceRecord>,
    /// Exact registered distributor identities.
    pub distributor_ids: Vec<String>,
    /// Distributor names retained by the registration.
    pub distributors: Vec<DistributorRecord>,
    /// Explicit known or unresolved source lineage.
    pub lineage: Vec<SourceLineage>,
    /// Capture pins and metadata without occurrence inventories or raw bytes.
    pub captures: Vec<CollectionCaptureSummary>,
    /// Collection identities represented by this source release.
    pub collection_ids: Vec<String>,
    /// Collection names/descriptions and component metadata, without lexical body rows.
    pub collections: Vec<CollectionRecord>,
    /// Source component metadata associated with the collection records.
    pub components: Vec<ComponentRecord>,
    /// Release provenance and terms without assertion identity arrays.
    pub releases: Vec<CollectionReleaseSummary>,
    /// Upstream revision, distinct from the catalogue-local revision.
    pub upstream_catalogue_revision: u64,
    /// Dictionary format version recorded by the release.
    pub dictionary_version: u32,
    /// Registry format version recorded by the release.
    pub registry_version: u32,
    /// Full reference counts, including empty categories.
    pub reference_counts: CollectionCategoryCounts,
    /// Terms and redistribution disposition retained for its releases.
    pub terms: Vec<CollectionTermsSummary>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct StoredCollectionRegistrationSummary {
    source_ids: Vec<String>,
    sources: Vec<SourceRecord>,
    distributor_ids: Vec<String>,
    distributors: Vec<DistributorRecord>,
    lineage: Vec<SourceLineage>,
    captures: Vec<CollectionCaptureSummary>,
    collection_ids: Vec<String>,
    collections: Vec<CollectionRecord>,
    components: Vec<ComponentRecord>,
    releases: Vec<CollectionReleaseSummary>,
    upstream_catalogue_revision: u64,
    dictionary_version: u32,
    registry_version: u32,
    reference_counts: CollectionCategoryCounts,
    terms: Vec<CollectionTermsSummary>,
}

fn registration_summary_bytes(descriptor_bytes: &[u8]) -> Result<Vec<u8>, CollectionError> {
    let descriptor: CollectionReferenceMetadata = serde_json::from_slice(descriptor_bytes)
        .map_err(|error| {
            CollectionError::InvalidRequest(format!(
                "encode registration summary from descriptor: {error}"
            ))
        })?;
    let summary = StoredCollectionRegistrationSummary {
        source_ids: descriptor
            .registry
            .sources
            .iter()
            .map(|source| source.id.to_string())
            .collect(),
        sources: descriptor.registry.sources.clone(),
        distributor_ids: descriptor
            .registry
            .distributors
            .iter()
            .map(|item| item.id.to_string())
            .collect(),
        distributors: descriptor.registry.distributors.clone(),
        lineage: descriptor.registry.lineage.clone(),
        captures: descriptor
            .captures
            .iter()
            .map(|capture| CollectionCaptureSummary {
                capture: capture.capture.clone(),
                metadata: capture.metadata.clone(),
                byte_length: capture.byte_length,
            })
            .collect(),
        collection_ids: descriptor
            .source_metadata
            .collections
            .iter()
            .map(|item| item.id.to_string())
            .collect(),
        collections: descriptor.source_metadata.collections.clone(),
        components: descriptor.source_metadata.components.clone(),
        releases: descriptor
            .source_metadata
            .releases
            .iter()
            .map(|release| CollectionReleaseSummary {
                id: release.id.to_string(),
                component_id: release.component.to_string(),
                distributor_id: release.distributor.as_ref().map(ToString::to_string),
                upstream_revision: release.upstream_revision.clone(),
                distributor_revision: release.distributor_revision.clone(),
                captured_at: release.captured_at.clone(),
                time_precision: release.time_precision,
                schema_version: release.schema_version,
                normalizer: release.normalizer.clone(),
                normalizer_version: release.normalizer_version.clone(),
                captures: release.captures.clone(),
                accounting: release.accounting.clone(),
                diagnostics_count: release.diagnostics.len(),
                selected: release.selected,
                availability: release.availability,
                content_notices_count: release.content_notices.len(),
                content_notice_licenses: release
                    .content_notices
                    .iter()
                    .map(|notice| notice.license.clone())
                    .collect(),
                content_notice_locators: release
                    .content_notices
                    .iter()
                    .map(|notice| notice.locator.clone())
                    .collect(),
                redistribution: format!("{:?}", release.redistribution).to_lowercase(),
                redistribution_evidence: release.redistribution_evidence.clone(),
            })
            .collect(),
        upstream_catalogue_revision: descriptor.upstream_catalogue_revision,
        dictionary_version: descriptor.dictionary_version,
        registry_version: descriptor.registry_version,
        reference_counts: descriptor.category_counts,
        terms: descriptor
            .source_metadata
            .releases
            .iter()
            .map(|release| CollectionTermsSummary {
                release_id: release.id.to_string(),
                redistribution: format!("{:?}", release.redistribution).to_lowercase(),
                evidence: release.redistribution_evidence.clone(),
                content_notices: release.content_notices.len(),
            })
            .collect(),
    };
    serde_json::to_vec(&summary).map_err(|error| {
        CollectionError::InvalidRequest(format!("encode registration summary: {error}"))
    })
}

/// Terms and redistribution summary for one source release.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CollectionTermsSummary {
    /// Stable release identity.
    pub release_id: String,
    /// Unknown, held, or permitted as explicitly recorded.
    pub redistribution: String,
    /// Exact evidence strings supplied by the source pack.
    pub evidence: Vec<String>,
    /// Number of captured content notices.
    pub content_notices: usize,
}

/// Capture identity, metadata and length retained by a registration summary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CollectionCaptureSummary {
    /// Full-byte capture identity.
    pub capture: CaptureRef,
    /// Capture-time source metadata.
    pub metadata: CaptureMetadata,
    /// Captured byte count without the captured payload.
    pub byte_length: usize,
}

/// Compact release provenance retained in list/inspect without assertion arrays.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CollectionReleaseSummary {
    /// Stable release ID.
    pub id: String,
    /// Associated source component ID.
    pub component_id: String,
    /// Distributor ID when explicitly recorded.
    pub distributor_id: Option<String>,
    /// Upstream revision presence state.
    pub upstream_revision: Presence<String>,
    /// Distributor revision presence state.
    pub distributor_revision: Presence<String>,
    /// Capture time presence state.
    pub captured_at: Presence<String>,
    /// Precision of the capture time.
    pub time_precision: TimePrecision,
    /// Release schema version.
    pub schema_version: u32,
    /// Normalizer name and version used for the release.
    pub normalizer: String,
    /// Normalizer version string.
    pub normalizer_version: String,
    /// Capture identities, without occurrence lists or source bytes.
    pub captures: Vec<CaptureRef>,
    /// Input, normalized and excluded count accounting.
    pub accounting: ReleaseAccounting,
    /// Number of diagnostics without their occurrence-level details.
    pub diagnostics_count: usize,
    /// Whether this release was selected by its source producer.
    pub selected: bool,
    /// Availability state.
    pub availability: Availability,
    /// Number of content notices.
    pub content_notices_count: usize,
    /// Notice license presence states without full captured notice text.
    pub content_notice_licenses: Vec<Presence<String>>,
    /// Notice locator presence states.
    pub content_notice_locators: Vec<Presence<String>>,
    /// Redistribution disposition.
    pub redistribution: String,
    /// Evidence retained for the redistribution disposition.
    pub redistribution_evidence: Vec<String>,
}

/// Minimum header observation for the currently derived store.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CollectionDerivedStoreMinimum {
    /// `supported` when the ordinary minimum header opens successfully.
    pub status: String,
    /// Read/format diagnostic when the minimum check cannot open it.
    pub diagnostic: Option<String>,
}

/// Result of a completed durable collection mutation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CollectionMutationReport {
    /// Report schema version.
    pub version: u32,
    /// True only for a completed durable operation.
    pub ok: bool,
    /// Operation name.
    pub operation: String,
    /// Registered, installed or idempotent.
    pub outcome: String,
    /// Catalogue-local revision after the operation.
    pub local_revision: u64,
    /// S02 does not configure user selection.
    pub policy_state: String,
    /// Mutation attempt identity, if one was needed.
    pub attempt_id: Option<String>,
    /// Current generation, if installed.
    pub generation: Option<CollectionGeneration>,
    /// Per-input dispositions.
    pub inputs: Vec<CollectionInputDisposition>,
}

/// Encodes a native path without using it as a lossy routing string.
pub fn native_path_report(path: &Path) -> NativePathReport {
    #[cfg(unix)]
    use std::os::unix::ffi::OsStrExt;
    #[cfg(unix)]
    let bytes = path.as_os_str().as_bytes();
    #[cfg(not(unix))]
    let owned = path.to_string_lossy().into_owned().into_bytes();
    #[cfg(not(unix))]
    let bytes = owned.as_slice();
    NativePathReport {
        encoding: if cfg!(unix) { "unix-bytes" } else { "utf8" }.into(),
        hex: bytes.iter().map(|byte| format!("{byte:02x}")).collect(),
        utf8: path.to_str().map(str::to_owned),
    }
}

/// A successfully normalized, strictly sealed pack ready for catalogue work.
#[derive(Debug)]
pub(crate) struct NormalizedPack {
    /// Stable pack ID.
    pub pack_id: PackId,
    /// Strict model loaded from the canonical release.
    pub dictionary: Dictionary,
    /// Exact release bytes to preserve.
    pub release: Vec<u8>,
    /// Exact seal bytes to preserve.
    pub seal_bytes: Vec<u8>,
    /// Strict parsed seal.
    pub seal: PackSeal,
    /// Input paths frozen by the caller before this operation.
    pub input_paths: Vec<PathBuf>,
    /// Normalizer accounting or None for already sealed input.
    pub report: Option<serde_json::Value>,
}

impl NormalizedPack {
    fn from_raw(
        pack_id: PackId,
        payload_kind: PackPayloadKind,
        dictionary: Dictionary,
        input_paths: Vec<PathBuf>,
        report: serde_json::Value,
    ) -> Result<Self, CollectionError> {
        let sealed = seal_release(pack_id, payload_kind, &dictionary)
            .map_err(|error| CollectionError::Pack(Box::new(error)))?;
        let dictionary = load_sealed_release(pack_id, &sealed.release, &sealed.seal)
            .map_err(|error| CollectionError::Pack(Box::new(error)))?;
        let seal = PackSeal::from_json_bytes(&sealed.seal)
            .map_err(|error| CollectionError::Pack(Box::new(error)))?;
        Ok(Self {
            pack_id,
            dictionary,
            release: sealed.release,
            seal_bytes: sealed.seal,
            seal,
            input_paths,
            report: Some(report),
        })
    }
}

/// Reads and normalizes one selected input using the existing source adapter.
///
/// This function performs no filesystem writes and no network access.
///
/// # Errors
///
/// Returns a typed input error when reading, adapter validation, sealing or
/// strict seal validation fails.
pub(crate) fn normalize_input(
    input: &CollectionInput<'_>,
    allow_synthetic: bool,
) -> Result<NormalizedPack, CollectionError> {
    match input {
        CollectionInput::SealedFiles {
            pack_id,
            release,
            seal,
        } => {
            let release_bytes = fs::read(release).map_err(|source| CollectionError::InputIo {
                path: release.clone(),
                source,
            })?;
            let seal_bytes = fs::read(seal).map_err(|source| CollectionError::InputIo {
                path: seal.clone(),
                source,
            })?;
            normalize_sealed(
                *pack_id,
                release_bytes,
                seal_bytes,
                vec![release.clone(), seal.clone()],
                allow_synthetic,
            )
        }
        CollectionInput::SealedBytes {
            pack_id,
            release,
            seal,
        } => normalize_sealed(
            *pack_id,
            release.to_vec(),
            seal.to_vec(),
            Vec::new(),
            allow_synthetic,
        ),
        CollectionInput::Jbotci {
            payload_kind,
            raw,
            metadata,
            options,
        } => {
            check_payload_kind(*payload_kind, allow_synthetic)?;
            let raw_bytes = read_input(raw)?;
            let metadata_bytes = read_input(metadata)?;
            let imported =
                import_jbotci_export(&raw_bytes, &metadata_bytes, options).map_err(|source| {
                    CollectionError::Adapter {
                        pack: PackId::LensiskuJbotci,
                        source: Box::new(source),
                    }
                })?;
            if !imported.report().definition_count_matches || !imported.report().entry_count_matches
            {
                return Err(CollectionError::InputInvalid {
                    pack: Some(PackId::LensiskuJbotci),
                    detail: "supplier counts do not match observed rows and eligible spellings"
                        .into(),
                });
            }
            let report = serde_json::to_value(imported.report()).map_err(|error| {
                CollectionError::InputInvalid {
                    pack: Some(PackId::LensiskuJbotci),
                    detail: error.to_string(),
                }
            })?;
            NormalizedPack::from_raw(
                PackId::LensiskuJbotci,
                *payload_kind,
                imported.into_dictionary(),
                vec![raw.clone(), metadata.clone()],
                report,
            )
        }
        CollectionInput::Llg {
            payload_kind,
            gismu,
            cmavo,
            rafsi,
            notice,
            options,
        } => {
            check_payload_kind(*payload_kind, allow_synthetic)?;
            let g = read_input(gismu)?;
            let c = read_input(cmavo)?;
            let r = read_input(rafsi)?;
            let n = read_input(notice)?;
            let imported = import_llg_lists(&g, &c, &r, &n, options).map_err(|source| {
                CollectionError::Adapter {
                    pack: PackId::Llg,
                    source: Box::new(source),
                }
            })?;
            let report = serde_json::to_value(imported.report()).map_err(|error| {
                CollectionError::InputInvalid {
                    pack: Some(PackId::Llg),
                    detail: error.to_string(),
                }
            })?;
            NormalizedPack::from_raw(
                PackId::Llg,
                *payload_kind,
                imported.into_dictionary(),
                vec![gismu.clone(), cmavo.clone(), rafsi.clone(), notice.clone()],
                report,
            )
        }
        CollectionInput::LaLojban {
            payload_kind,
            raw,
            options,
        } => {
            check_payload_kind(*payload_kind, allow_synthetic)?;
            let bytes = read_input(raw)?;
            let imported = import_la_lojban_export(&bytes, options).map_err(|source| {
                CollectionError::Adapter {
                    pack: PackId::LaLojban,
                    source: Box::new(source),
                }
            })?;
            let report = serde_json::to_value(imported.report()).map_err(|error| {
                CollectionError::InputInvalid {
                    pack: Some(PackId::LaLojban),
                    detail: error.to_string(),
                }
            })?;
            NormalizedPack::from_raw(
                PackId::LaLojban,
                *payload_kind,
                imported.into_dictionary(),
                vec![raw.clone()],
                report,
            )
        }
        CollectionInput::Sutysisku {
            payload_kind,
            raw,
            options,
        } => {
            check_payload_kind(*payload_kind, allow_synthetic)?;
            let bytes = read_input(raw)?;
            let imported = import_sutysisku_export(&bytes, options).map_err(|source| {
                CollectionError::Adapter {
                    pack: PackId::Sutysisku,
                    source: Box::new(source),
                }
            })?;
            let report = serde_json::to_value(imported.report()).map_err(|error| {
                CollectionError::InputInvalid {
                    pack: Some(PackId::Sutysisku),
                    detail: error.to_string(),
                }
            })?;
            NormalizedPack::from_raw(
                PackId::Sutysisku,
                *payload_kind,
                imported.into_dictionary(),
                vec![raw.clone()],
                report,
            )
        }
    }
}

fn normalize_sealed(
    pack_id: PackId,
    release: Vec<u8>,
    seal_bytes: Vec<u8>,
    paths: Vec<PathBuf>,
    allow_synthetic: bool,
) -> Result<NormalizedPack, CollectionError> {
    let seal = PackSeal::from_json_bytes(&seal_bytes).map_err(|source| {
        CollectionError::InputInvalidSource {
            pack: Some(pack_id),
            detail: "pack seal failed strict parsing".into(),
            source: Box::new(source),
        }
    })?;
    if seal.payload_kind == PackPayloadKind::Synthetic && !allow_synthetic {
        return Err(CollectionError::InvalidRequest(
            "synthetic inputs require explicit allow_synthetic opt-in".into(),
        ));
    }
    let dictionary = load_sealed_release(pack_id, &release, &seal_bytes).map_err(|source| {
        CollectionError::InputInvalidSource {
            pack: Some(pack_id),
            detail: "sealed release failed validation".into(),
            source: Box::new(source),
        }
    })?;
    Ok(NormalizedPack {
        pack_id,
        dictionary,
        release,
        seal_bytes,
        seal,
        input_paths: paths,
        report: None,
    })
}
fn check_payload_kind(kind: PackPayloadKind, allow_synthetic: bool) -> Result<(), CollectionError> {
    if kind == PackPayloadKind::Synthetic && !allow_synthetic {
        return Err(CollectionError::InvalidRequest(
            "synthetic inputs require explicit allow_synthetic opt-in".into(),
        ));
    }
    Ok(())
}
fn read_input(path: &Path) -> Result<Vec<u8>, CollectionError> {
    fs::read(path).map_err(|source| CollectionError::InputIo {
        path: path.to_path_buf(),
        source,
    })
}
fn input_pack_id(input: &CollectionInput<'_>) -> PackId {
    match input {
        CollectionInput::SealedFiles { pack_id, .. }
        | CollectionInput::SealedBytes { pack_id, .. } => *pack_id,
        CollectionInput::Jbotci { .. } => PackId::LensiskuJbotci,
        CollectionInput::Llg { .. } => PackId::Llg,
        CollectionInput::LaLojban { .. } => PackId::LaLojban,
        CollectionInput::Sutysisku { .. } => PackId::Sutysisku,
    }
}
fn input_has_relative_paths(input: &CollectionInput<'_>) -> bool {
    let relative = |path: &Path| !path.is_absolute();
    match input {
        CollectionInput::SealedFiles { release, seal, .. } => relative(release) || relative(seal),
        CollectionInput::SealedBytes { .. } => false,
        CollectionInput::Jbotci { raw, metadata, .. } => relative(raw) || relative(metadata),
        CollectionInput::Llg {
            gismu,
            cmavo,
            rafsi,
            notice,
            ..
        } => relative(gismu) || relative(cmavo) || relative(rafsi) || relative(notice),
        CollectionInput::LaLojban { raw, .. } | CollectionInput::Sutysisku { raw, .. } => {
            relative(raw)
        }
    }
}
fn validate_selection(
    inputs: &[CollectionInput<'_>],
    selection: &CollectionSelection,
) -> Result<(), CollectionError> {
    if inputs.is_empty() {
        return Err(CollectionError::InvalidRequest(
            "at least one input is required".into(),
        ));
    }
    if matches!(selection, CollectionSelection::Packs(ids) if ids.is_empty()) {
        return Err(CollectionError::InvalidRequest(
            "explicit pack selection must not be empty".into(),
        ));
    }
    let mut seen = std::collections::BTreeSet::new();
    for input in inputs {
        let id = input_pack_id(input);
        if !seen.insert(id) {
            return Err(CollectionError::InvalidRequest(format!(
                "duplicate input pack {id}"
            )));
        }
    }
    let selected = match selection {
        CollectionSelection::All => vec![
            PackId::LensiskuJbotci,
            PackId::Llg,
            PackId::LaLojban,
            PackId::Sutysisku,
        ],
        CollectionSelection::Packs(ids) => {
            let mut unique = std::collections::BTreeSet::new();
            for id in ids {
                if !unique.insert(*id) {
                    return Err(CollectionError::InvalidRequest(format!(
                        "duplicate selected pack {id}"
                    )));
                }
            }
            ids.clone()
        }
    };
    for id in selected {
        if !seen.contains(&id) {
            return Err(CollectionError::InvalidRequest(format!(
                "selected pack {id} has no input"
            )));
        }
    }
    Ok(())
}

/// Hashes exact canonical release bytes.
#[must_use]
pub(crate) fn release_sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
thread_local! {
    static TEST_INTERRUPT_AFTER_STAGE_ALLOCATION: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static TEST_FAIL_NEXT_ROLLBACK: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

#[cfg(test)]
mod manifest_tests {
    use super::*;

    #[test]
    fn test_allocate_stage_retries_occupied_candidate_without_overwriting_it() {
        let unique = format!(
            "vlajmaji-stage-allocation-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let root = std::env::temp_dir().join(unique);
        fs::create_dir(&root).unwrap();
        fs::create_dir(root.join("generations")).unwrap();
        let occupied = root.join("generations/occupied");
        fs::create_dir(&occupied).unwrap();
        fs::write(occupied.join("sentinel"), b"keep exactly").unwrap();
        let mut names = ["occupied", "available"].into_iter();
        let (attempt, generation, stage) = allocate_stage(&root, 3, "catalogue", |_, _, _| {
            names.next().unwrap().into()
        })
        .unwrap();
        assert_eq!(attempt, "available");
        assert_eq!(generation, "catalogue-available");
        assert!(stage.is_dir());
        assert_eq!(
            fs::read(occupied.join("sentinel")).unwrap(),
            b"keep exactly"
        );

        let exhausted =
            allocate_stage(&root, 3, "catalogue", |_, _, _| "occupied".into()).unwrap_err();
        assert!(matches!(exhausted, CollectionError::OccupiedDestination(_)));
        assert_eq!(
            fs::read(occupied.join("sentinel")).unwrap(),
            b"keep exactly"
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn test_interruption_after_stage_allocation_retains_prior_generation_and_allows_retry() {
        let unique = format!(
            "vlajmaji-stage-interruption-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let root = std::env::temp_dir().join(unique);
        let dictionary = Dictionary::from_document(Default::default()).unwrap();
        let first = seal_release(PackId::Llg, PackPayloadKind::Synthetic, &dictionary).unwrap();
        let first_request = CollectionRequest::new(
            vec![CollectionInput::SealedBytes {
                pack_id: PackId::Llg,
                release: Cow::Borrowed(&first.release),
                seal: Cow::Borrowed(&first.seal),
            }],
            CollectionSelection::Packs(vec![PackId::Llg]),
            true,
        )
        .unwrap();
        let first_generation = CollectionCatalogue::install(&root, first_request)
            .unwrap()
            .generation
            .unwrap();
        let first_store_bytes =
            fs::read(first_generation.store_path.join("dictionary.sqlite3")).unwrap();

        let sealed = seal_release(
            PackId::LensiskuJbotci,
            PackPayloadKind::Synthetic,
            &dictionary,
        )
        .unwrap();
        let request = CollectionRequest::new(
            vec![CollectionInput::SealedBytes {
                pack_id: PackId::LensiskuJbotci,
                release: Cow::Borrowed(&sealed.release),
                seal: Cow::Borrowed(&sealed.seal),
            }],
            CollectionSelection::Packs(vec![PackId::LensiskuJbotci]),
            true,
        )
        .unwrap();
        TEST_INTERRUPT_AFTER_STAGE_ALLOCATION.with(|interrupt| interrupt.set(true));
        let error = CollectionCatalogue::install(&root, request.clone()).unwrap_err();
        assert!(
            matches!(&error, CollectionError::MutationContext { source, .. }
            if matches!(source.as_ref(), CollectionError::InvalidRequest(detail) if detail.contains("injected interruption")))
        );
        let report = error.report("install");
        assert_eq!(report.activation.as_deref(), Some("not_activated"));
        assert_eq!(report.prior_local_revision, Some(1));
        let attempt = report.attempt_id.unwrap();
        assert!(report.proposed_generation.is_some());
        assert_eq!(
            report.observed_generation.as_deref(),
            Some(first_generation.generation_id.as_str())
        );
        let listing = CollectionCatalogue::open(&root).unwrap().list().unwrap();
        assert_eq!(listing.local_revision, 1);
        assert_eq!(
            listing.generation.as_ref().unwrap().generation_id,
            first_generation.generation_id
        );
        assert_eq!(
            fs::read(first_generation.store_path.join("dictionary.sqlite3")).unwrap(),
            first_store_bytes
        );
        assert!(
            listing.staged_attempts.iter().any(|stage| stage
                == &format!("unmarked:{}", native_path_report(Path::new(&attempt)).hex))
        );

        let retried = CollectionCatalogue::install(&root, request).unwrap();
        assert_eq!(retried.local_revision, 2);
        assert_ne!(
            retried.generation.as_ref().unwrap().generation_id,
            first_generation.generation_id
        );
        assert_eq!(
            fs::read(first_generation.store_path.join("dictionary.sqlite3")).unwrap(),
            first_store_bytes
        );
        assert_eq!(
            CollectionCatalogue::open(&root)
                .unwrap()
                .list()
                .unwrap()
                .staged_attempts
                .len(),
            1
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn test_injected_rollback_failure_retains_initiating_and_rollback_causes() {
        let unique = format!(
            "vlajmaji-rollback-failure-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let root = std::env::temp_dir().join(unique);
        let dictionary = Dictionary::from_document(Default::default()).unwrap();
        let sealed = seal_release(PackId::Llg, PackPayloadKind::Synthetic, &dictionary).unwrap();
        let request = CollectionRequest::new(
            vec![CollectionInput::SealedBytes {
                pack_id: PackId::Llg,
                release: Cow::Borrowed(&sealed.release),
                seal: Cow::Borrowed(&sealed.seal),
            }],
            CollectionSelection::Packs(vec![PackId::Llg]),
            true,
        )
        .unwrap();
        TEST_INTERRUPT_AFTER_STAGE_ALLOCATION.with(|interrupt| interrupt.set(true));
        TEST_FAIL_NEXT_ROLLBACK.with(|fail| fail.set(true));
        let error = CollectionCatalogue::install(&root, request).unwrap_err();
        assert!(
            matches!(&error, CollectionError::MutationContext { source, .. }
            if matches!(source.as_ref(), CollectionError::IndeterminateSources { source: Some(cause), rollback_source: Some(rollback), .. }
                if matches!(cause.downcast_ref::<CollectionError>(), Some(CollectionError::InvalidRequest(_)))
                    && rollback.source().and_then(|error| error.downcast_ref::<io::Error>()).is_some_and(|error| error.to_string().contains("injected rollback cause"))))
        );
        let report = error.report("install");
        assert_eq!(report.category, "indeterminate");
        assert_eq!(report.activation.as_deref(), Some("indeterminate"));
        let listing = CollectionCatalogue::open(&root).unwrap().list().unwrap();
        assert_eq!(listing.local_revision, 0);
        assert!(listing.generation.is_none());
        let _ = fs::remove_dir_all(root);
    }

    fn parse(options: &str) -> Result<CollectionRequest<'static>, CollectionError> {
        let bytes = format!(
            r#"{{"version":1,"inputs":[{{"kind":"la-lojban","pack_id":"la-lojban","payload_kind":"synthetic","raw":"raw.json","options":{{"revision":"rev","expected_raw_sha256":"hash"{options}}}}}]}}"#
        );
        CollectionRequest::from_manifest(
            bytes.as_bytes(),
            "/tmp/collections/batch.json",
            CollectionSelection::Packs(vec![PackId::LaLojban]),
            true,
        )
    }

    fn parse_sutysisku(options: &str) -> Result<CollectionRequest<'static>, CollectionError> {
        let bytes = format!(
            r#"{{"version":1,"inputs":[{{"kind":"sutysisku","pack_id":"sutysisku","payload_kind":"synthetic","raw":"raw.json","options":{{"revision":"rev","expected_raw_sha256":"hash"{options}}}}}]}}"#
        );
        CollectionRequest::from_manifest(
            bytes.as_bytes(),
            "/tmp/collections/batch.json",
            CollectionSelection::Packs(vec![PackId::Sutysisku]),
            true,
        )
    }

    #[test]
    fn test_selection_skips_unselected_missing_native_path_and_all_reports_failure() {
        let dictionary = Dictionary::from_document(Default::default()).unwrap();
        let packs = [
            PackId::LensiskuJbotci,
            PackId::Llg,
            PackId::LaLojban,
            PackId::Sutysisku,
        ]
        .into_iter()
        .map(|id| {
            (
                id,
                seal_release(id, PackPayloadKind::Synthetic, &dictionary).unwrap(),
            )
        })
        .collect::<BTreeMap<_, _>>();
        let missing = std::env::temp_dir().join(format!(
            "vlajmaji-missing-unselected-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let selected_only = CollectionRequest::new(
            vec![
                CollectionInput::SealedBytes {
                    pack_id: PackId::Llg,
                    release: Cow::Borrowed(&packs[&PackId::Llg].release),
                    seal: Cow::Borrowed(&packs[&PackId::Llg].seal),
                },
                CollectionInput::SealedFiles {
                    pack_id: PackId::LensiskuJbotci,
                    release: missing.clone(),
                    seal: missing.clone().with_extension("seal"),
                },
            ],
            CollectionSelection::Packs(vec![PackId::Llg]),
            true,
        )
        .unwrap();
        let normalized = normalize_selected(selected_only, None).unwrap();
        assert_eq!(normalized.len(), 1);
        assert_eq!(normalized[0].pack_id, PackId::Llg);

        let mut all_inputs = vec![CollectionInput::SealedFiles {
            pack_id: PackId::LensiskuJbotci,
            release: missing.clone(),
            seal: missing.clone().with_extension("seal"),
        }];
        for id in [PackId::Llg, PackId::LaLojban, PackId::Sutysisku] {
            all_inputs.push(CollectionInput::SealedBytes {
                pack_id: id,
                release: Cow::Borrowed(&packs[&id].release),
                seal: Cow::Borrowed(&packs[&id].seal),
            });
        }
        let all = CollectionRequest::new(all_inputs, CollectionSelection::All, true).unwrap();
        let error = normalize_selected(all, None).unwrap_err();
        let CollectionError::Batch { source, inputs } = error else {
            panic!("all-input failure must retain batch dispositions")
        };
        assert!(matches!(source.as_ref(), CollectionError::InputIo { .. }));
        let by_id = inputs
            .into_iter()
            .map(|input| (input.pack_id, input.disposition))
            .collect::<BTreeMap<_, _>>();
        assert_eq!(by_id["lensisku-jbotci"], "failed");
        assert_eq!(by_id["llg"], "unprocessed");
        assert_eq!(by_id["la-lojban"], "unprocessed");
        assert_eq!(by_id["sutysisku"], "unprocessed");
        assert!(
            !missing.exists(),
            "the selected-only success and failed read create no missing input"
        );
    }

    #[test]
    fn test_manifest_locator_absence_and_null_are_none_and_strings_are_retained() {
        for options in ["", ",\"raw_locator\":null,\"notice_locator\":null"] {
            let request = parse(options).unwrap();
            let CollectionInput::LaLojban { raw, options, .. } = &request.inputs[0] else {
                panic!("La-Lojban input expected")
            };
            assert_eq!(raw, Path::new("/tmp/collections/raw.json"));
            assert_eq!(options.raw_locator, None);
            assert_eq!(options.notice_locator, None);
        }
        let request = parse(r#","raw_locator":"https://example.invalid/raw","notice_locator":"https://example.invalid/notice""#).unwrap();
        let CollectionInput::LaLojban { options, .. } = &request.inputs[0] else {
            panic!("La-Lojban input expected")
        };
        assert_eq!(
            options.raw_locator.as_deref(),
            Some("https://example.invalid/raw")
        );
        assert_eq!(
            options.notice_locator.as_deref(),
            Some("https://example.invalid/notice")
        );
    }

    #[test]
    fn test_manifest_locator_wrong_type_is_rejected() {
        let error = parse(r#","raw_locator":17"#).unwrap_err();
        assert!(matches!(error, CollectionError::InvalidRequest(_)));
        let error = parse_sutysisku(r#","notice_locator":false"#).unwrap_err();
        assert!(matches!(error, CollectionError::InvalidRequest(_)));
    }

    #[test]
    fn test_sutysisku_manifest_optional_locators_accept_omission_and_null() {
        for options in ["", ",\"raw_locator\":null,\"notice_locator\":null"] {
            let request = parse_sutysisku(options).unwrap();
            let CollectionInput::Sutysisku { options, .. } = &request.inputs[0] else {
                panic!("Sutysisku input expected")
            };
            assert_eq!(options.raw_locator, None);
            assert_eq!(options.notice_locator, None);
        }
    }

    #[test]
    fn test_manifest_direct_api_rejects_duplicate_and_escaped_duplicate_keys() {
        let duplicate_top = br#"{"version":1,"version":1,"inputs":[]}"#;
        let duplicate_option = br#"{"version":1,"inputs":[{"kind":"la-lojban","pack_id":"la-lojban","payload_kind":"synthetic","raw":"raw","options":{"revision":"a","revis\u0069on":"b","expected_raw_sha256":"hash"}}]}"#;
        for bytes in [duplicate_top.as_slice(), duplicate_option.as_slice()] {
            let error = CollectionRequest::from_manifest(
                bytes,
                "/tmp/collections/batch.json",
                CollectionSelection::Packs(vec![PackId::LaLojban]),
                true,
            )
            .unwrap_err();
            assert!(matches!(error, CollectionError::InvalidRequest(_)));
        }
    }

    #[test]
    fn test_batch_failure_reports_validated_failed_unprocessed_and_skipped_inputs() {
        let dictionary = Dictionary::from_document(Default::default()).unwrap();
        let llg = seal_release(PackId::Llg, PackPayloadKind::Synthetic, &dictionary).unwrap();
        fn valid<'a>(pack_id: PackId, release: &'a [u8], seal: &'a [u8]) -> CollectionInput<'a> {
            CollectionInput::SealedBytes {
                pack_id,
                release: Cow::Borrowed(release),
                seal: Cow::Borrowed(seal),
            }
        }
        let request = CollectionRequest::new(
            vec![
                valid(PackId::Llg, &llg.release, &llg.seal),
                valid(PackId::LensiskuJbotci, &llg.release, &llg.seal),
                valid(PackId::Sutysisku, &llg.release, &llg.seal),
            ],
            CollectionSelection::Packs(vec![PackId::LensiskuJbotci, PackId::Llg]),
            true,
        )
        .unwrap();
        let error = normalize_selected(request, None).unwrap_err();
        let report = error.report("install");
        assert_eq!(report.version, 1);
        assert!(!report.ok);
        assert_eq!(report.operation, "install");
        assert_eq!(report.category, "input_invalid");
        assert_eq!(report.activation.as_deref(), Some("not_activated"));
        assert!(report.observed_generation.is_none());
        let dispositions = report
            .inputs
            .into_iter()
            .map(|input| (input.pack_id, input.disposition))
            .collect::<BTreeMap<_, _>>();
        assert_eq!(dispositions["lensisku-jbotci"], "failed");
        assert_eq!(dispositions["llg"], "unprocessed");
        assert_eq!(dispositions["sutysisku"], "skipped_without_io");
    }
}
