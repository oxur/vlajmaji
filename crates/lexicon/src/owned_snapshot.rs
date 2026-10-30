//! Pure, versioned snapshots of complete owned dictionary history.

use std::{error::Error, fmt};

use serde::de::Deserializer;
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
use sha2::{Digest, Sha256};

use crate::{
    Dictionary, DictionaryError, EntityId, OwnedContextPin, OwnedCurationData,
    OwnedCurationDocument, OwnedCurationError, OwnedCurationRecord, OwnedCurationRevision,
    OwnedHead, SourceId, SourceOrigin, SourceRegistry,
};

const SECTION_NAMES: [&str; 24] = [
    "registry.sources",
    "registry.contributors",
    "registry.distributors",
    "registry.aliases",
    "registry.lineage",
    "registry.snapshots",
    "registry.occurrences",
    "captures",
    "metadata.code_notices",
    "metadata.collections",
    "metadata.components",
    "metadata.releases",
    "metadata.entries",
    "metadata.assertions",
    "metadata.contexts",
    "metadata.frames",
    "metadata.forms",
    "metadata.etymologies",
    "metadata.relations",
    "metadata.owned_revisions",
    "metadata.editorial_senses",
    "metadata.computations",
    "metadata.annotations",
    "metadata.extensions",
];
const DICTIONARY_VERSION: u32 = 1;
const REGISTRY_VERSION: u32 = 1;

/// One ordered committed-prefix length for a typed dictionary section.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OwnedSectionPrefix {
    /// Exact versioned section name.
    pub section: String,
    /// Number of rows committed in this section at this revision.
    pub length: u64,
}

/// Complete immutable base state and all committed state hashes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OwnedSnapshotCommit {
    /// Revision represented by this commit, in genesis-first order.
    pub revision: EntityId,
    /// SHA-256 of the complete canonical Dictionary at this revision.
    pub state_sha256: String,
    /// Exact ordered 24-section committed-prefix roster.
    pub prefixes: Vec<OwnedSectionPrefix>,
}

/// One curation sidecar prefix at its corresponding base revision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OwnedSnapshotCurationPrefix {
    /// Corresponding owned-base revision.
    pub revision: EntityId,
    /// Number of context pins committed at this revision.
    pub contexts_length: u64,
    /// Number of curation definitions committed at this revision.
    pub records_length: u64,
    /// Stable curation state digest at this revision.
    pub curation_sha256: String,
}

/// Complete final curation rows and the actual historical sidecar cutoffs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OwnedSnapshotCuration {
    /// Curation schema version, currently one.
    pub version: u32,
    /// Immutable context pins in declaration order.
    pub contexts: Vec<OwnedContextPin>,
    /// Immutable definitions in creation order.
    pub records: Vec<OwnedCurationRecord>,
    /// Complete selection history, one row per base revision.
    pub revisions: Vec<OwnedCurationRevision>,
    /// Actual persisted sidecar prefixes and hashes.
    pub prefixes: Vec<OwnedSnapshotCurationPrefix>,
}

/// Unvalidated typed input to [`OwnedSnapshot::from_data`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OwnedSnapshotData {
    /// Snapshot wire version, currently one.
    pub version: u32,
    /// Original owned journal schema, one or two.
    pub journal_schema: u32,
    /// Sole owned source.
    pub source: SourceId,
    /// Final committed head.
    pub head: EntityId,
    /// Complete final base Dictionary, stored once.
    pub dictionary: Dictionary,
    /// Complete commit history with actual prefix rosters.
    pub commits: Vec<OwnedSnapshotCommit>,
    /// Present exactly for schema two, including empty migrated history.
    pub curation: Option<OwnedSnapshotCuration>,
}

/// A validated complete snapshot with invariant-bearing data kept private.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct OwnedSnapshot(OwnedSnapshotData);

#[derive(Deserialize)]
struct SnapshotHeader {
    version: u32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SnapshotWire {
    version: u32,
    journal_schema: u32,
    source: SourceId,
    head: EntityId,
    dictionary: Box<RawValue>,
    commits: Vec<OwnedSnapshotCommit>,
    curation: Option<OwnedSnapshotCuration>,
}

impl<'de> Deserialize<'de> for OwnedSnapshot {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let wire = SnapshotWire::deserialize(deserializer)?;
        let dictionary = Dictionary::from_json_bytes(wire.dictionary.get().as_bytes())
            .map_err(serde::de::Error::custom)?;
        Self::from_data(OwnedSnapshotData {
            version: wire.version,
            journal_schema: wire.journal_schema,
            source: wire.source,
            head: wire.head,
            dictionary,
            commits: wire.commits,
            curation: wire.curation,
        })
        .map_err(serde::de::Error::custom)
    }
}

/// Strict integrity inventory for one complete owned snapshot payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OwnedSnapshotSeal {
    /// Seal schema version, currently one.
    pub version: u32,
    /// Exact seal kind discriminator.
    pub kind: String,
    /// Snapshot payload version.
    pub snapshot_version: u32,
    /// Original journal schema, one or two.
    pub journal_schema: u32,
    /// Dictionary model version.
    pub dictionary_version: u32,
    /// Registry model version.
    pub registry_version: u32,
    /// Sole owned source.
    pub source: SourceId,
    /// Final committed head.
    pub head: EntityId,
    /// Complete revision inventory in original order.
    pub revision_ids: Vec<EntityId>,
    /// SHA-256 of the exact payload bytes.
    pub snapshot_sha256: String,
    /// Exact payload byte length.
    pub snapshot_bytes: u64,
}

impl OwnedSnapshotSeal {
    /// Strictly parse a seal JSON object, rejecting duplicate keys and invalid UTF-8.
    ///
    /// # Errors
    ///
    /// Returns an error for malformed JSON, duplicate keys, unsupported versions,
    /// or an invalid seal header or digest.
    pub fn from_json_bytes(bytes: &[u8]) -> Result<Self, OwnedSnapshotError> {
        let text = std::str::from_utf8(bytes)
            .map_err(|error| OwnedSnapshotError::InvalidDocument(error.to_string()))?;
        crate::check_duplicate_keys(text)
            .map_err(|error| OwnedSnapshotError::InvalidDocument(error.to_string()))?;
        let seal: Self = serde_json::from_str(text)
            .map_err(|error| OwnedSnapshotError::InvalidDocument(error.to_string()))?;
        seal.validate()?;
        Ok(seal)
    }

    /// Serialize the typed seal as compact JSON without a trailing line feed.
    ///
    /// # Errors
    ///
    /// Returns an error if the seal header, version, or digest is invalid, or
    /// serialization fails.
    pub fn to_json_bytes(&self) -> Result<Vec<u8>, OwnedSnapshotError> {
        self.validate()?;
        serde_json::to_vec(self)
            .map_err(|error| OwnedSnapshotError::InvalidDocument(error.to_string()))
    }

    fn validate(&self) -> Result<(), OwnedSnapshotError> {
        for (kind, version) in [
            ("owned snapshot seal", self.version),
            ("owned snapshot", self.snapshot_version),
            ("dictionary", self.dictionary_version),
            ("registry", self.registry_version),
        ] {
            if version != 1 {
                return Err(OwnedSnapshotError::UnsupportedVersion {
                    kind,
                    found: version,
                });
            }
        }
        if !matches!(self.journal_schema, 1 | 2) {
            return Err(OwnedSnapshotError::UnsupportedVersion {
                kind: "owned journal schema",
                found: self.journal_schema,
            });
        }
        if self.kind != "vlajmaji-owned-snapshot" {
            return Err(OwnedSnapshotError::InvalidDocument(
                "unexpected owned snapshot seal kind".into(),
            ));
        }
        if !valid_sha256(&self.snapshot_sha256) {
            return Err(OwnedSnapshotError::InvalidDocument(
                "seal snapshot_sha256 must be lowercase hexadecimal".into(),
            ));
        }
        if self.snapshot_bytes == 0
            || self.revision_ids.is_empty()
            || self.revision_ids.last() != Some(&self.head)
        {
            return Err(OwnedSnapshotError::InvalidDocument(
                "seal requires nonempty bytes and a revision inventory ending at head".into(),
            ));
        }
        let unique: std::collections::BTreeSet<_> = self.revision_ids.iter().collect();
        if unique.len() != self.revision_ids.len() {
            return Err(OwnedSnapshotError::InvalidDocument(
                "seal revision_ids must be unique".into(),
            ));
        }
        Ok(())
    }
}

/// Canonical snapshot payload paired with its canonical seal bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SealedOwnedSnapshot {
    /// Canonical complete snapshot JSON bytes.
    pub snapshot: Vec<u8>,
    /// Canonical strict seal JSON bytes.
    pub seal: Vec<u8>,
}

/// Typed failure while validating or sealing complete owned history.
#[derive(Debug)]
#[non_exhaustive]
pub enum OwnedSnapshotError {
    /// Malformed strict input or invalid snapshot structure.
    InvalidDocument(String),
    /// Unsupported snapshot, seal, dictionary, registry, or curation version.
    UnsupportedVersion {
        /// Versioned layer that was unsupported.
        kind: &'static str,
        /// Version present in the input.
        found: u32,
    },
    /// Requested revision is absent from the snapshot.
    MissingRevision(EntityId),
    /// A complete history invariant is invalid.
    History(String),
    /// Actual byte or roster length differs from the declared value.
    LengthMismatch {
        /// Declared length.
        expected: u64,
        /// Actual length.
        actual: u64,
    },
    /// A declared digest differs from the actual bytes/state.
    HashMismatch {
        /// Declared digest.
        expected: String,
        /// Computed digest.
        actual: String,
    },
    /// Seal inventory differs from the validated payload.
    InventoryMismatch {
        /// Inventory field that disagreed.
        field: &'static str,
    },
    /// A sealed payload is valid but not its canonical typed byte representation.
    NonCanonical,
    /// Base Dictionary validation failed.
    Dictionary(DictionaryError),
    /// Curation history validation failed.
    Curation(OwnedCurationError),
}

impl fmt::Display for OwnedSnapshotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDocument(detail) => write!(f, "invalid owned snapshot: {detail}"),
            Self::UnsupportedVersion { kind, found } => {
                write!(f, "unsupported {kind} version {found}")
            }
            Self::MissingRevision(revision) => write!(f, "snapshot revision not found: {revision}"),
            Self::History(detail) => write!(f, "invalid owned snapshot history: {detail}"),
            Self::LengthMismatch { expected, actual } => {
                write!(
                    f,
                    "snapshot length mismatch: expected {expected}, got {actual}"
                )
            }
            Self::HashMismatch { expected, actual } => {
                write!(
                    f,
                    "snapshot hash mismatch: expected {expected}, got {actual}"
                )
            }
            Self::InventoryMismatch { field } => {
                write!(f, "snapshot seal inventory mismatch: {field}")
            }
            Self::NonCanonical => f.write_str("sealed snapshot payload is not canonical JSON"),
            Self::Dictionary(error) => write!(f, "invalid snapshot dictionary: {error}"),
            Self::Curation(error) => write!(f, "invalid snapshot curation: {error}"),
        }
    }
}

impl Error for OwnedSnapshotError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Dictionary(error) => Some(error),
            Self::Curation(error) => Some(error),
            _ => None,
        }
    }
}

impl From<DictionaryError> for OwnedSnapshotError {
    fn from(error: DictionaryError) -> Self {
        Self::Dictionary(error)
    }
}

impl From<OwnedCurationError> for OwnedSnapshotError {
    fn from(error: OwnedCurationError) -> Self {
        Self::Curation(error)
    }
}

impl OwnedSnapshot {
    /// Validate complete typed snapshot data and every historical state.
    ///
    /// # Errors
    ///
    /// Returns an error if the snapshot does not satisfy its version, history,
    /// roster, hash, or curation invariants.
    pub fn from_data(data: OwnedSnapshotData) -> Result<Self, OwnedSnapshotError> {
        validate_snapshot(&data)?;
        Ok(Self(data))
    }

    /// Strictly parse a snapshot object and preserve nested Dictionary bytes.
    ///
    /// # Errors
    ///
    /// Returns an error for malformed or invalid nested Dictionary data,
    /// duplicate keys, unsupported versions, or an invalid historical state.
    pub fn from_json_bytes(bytes: &[u8]) -> Result<Self, OwnedSnapshotError> {
        let text = std::str::from_utf8(bytes)
            .map_err(|error| OwnedSnapshotError::InvalidDocument(error.to_string()))?;
        crate::check_duplicate_keys(text)
            .map_err(|error| OwnedSnapshotError::InvalidDocument(error.to_string()))?;
        let header: SnapshotHeader = serde_json::from_str(text)
            .map_err(|error| OwnedSnapshotError::InvalidDocument(error.to_string()))?;
        if header.version != 1 {
            return Err(OwnedSnapshotError::UnsupportedVersion {
                kind: "owned snapshot",
                found: header.version,
            });
        }
        let wire: SnapshotWire = serde_json::from_str(text)
            .map_err(|error| OwnedSnapshotError::InvalidDocument(error.to_string()))?;
        let dictionary = Dictionary::from_json_bytes(wire.dictionary.get().as_bytes())?;
        Self::from_data(OwnedSnapshotData {
            version: wire.version,
            journal_schema: wire.journal_schema,
            source: wire.source,
            head: wire.head,
            dictionary,
            commits: wire.commits,
            curation: wire.curation,
        })
    }

    /// Serialize this validated snapshot as compact typed JSON bytes.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails.
    pub fn to_json_bytes(&self) -> Result<Vec<u8>, OwnedSnapshotError> {
        serde_json::to_vec(self)
            .map_err(|error| OwnedSnapshotError::InvalidDocument(error.to_string()))
    }

    /// Borrow the validated typed snapshot data.
    pub fn as_data(&self) -> &OwnedSnapshotData {
        &self.0
    }

    /// Consume the validated wrapper and return its data.
    pub fn into_data(self) -> OwnedSnapshotData {
        self.0
    }

    /// Reconstruct one complete validated historical Dictionary.
    ///
    /// # Errors
    ///
    /// Returns [`OwnedSnapshotError::MissingRevision`] when the requested
    /// revision is not present, or an error if reconstruction fails.
    pub fn dictionary_at(&self, revision: &EntityId) -> Result<Dictionary, OwnedSnapshotError> {
        let index = self
            .0
            .commits
            .iter()
            .position(|commit| &commit.revision == revision)
            .ok_or_else(|| OwnedSnapshotError::MissingRevision(revision.clone()))?;
        build_state(&self.0, index)
    }

    /// Reconstruct curation state at one revision, if this is a schema-two snapshot.
    ///
    /// # Errors
    ///
    /// Returns [`OwnedSnapshotError::MissingRevision`] when the requested
    /// revision is not present, or an error if reconstruction fails.
    pub fn curation_at(
        &self,
        revision: &EntityId,
    ) -> Result<Option<OwnedCurationDocument>, OwnedSnapshotError> {
        let index = self
            .0
            .commits
            .iter()
            .position(|commit| &commit.revision == revision)
            .ok_or_else(|| OwnedSnapshotError::MissingRevision(revision.clone()))?;
        let Some(curation) = &self.0.curation else {
            return Ok(None);
        };
        let dictionary = build_state(&self.0, index)?;
        Ok(Some(build_curation(&self.0, curation, index, dictionary)?))
    }
}

/// Construct canonical snapshot and seal bytes for one validated snapshot.
///
/// # Errors
///
/// Returns an error if the snapshot or seal cannot be serialized.
pub fn seal_owned_snapshot(
    snapshot: &OwnedSnapshot,
) -> Result<SealedOwnedSnapshot, OwnedSnapshotError> {
    let payload = snapshot.to_json_bytes()?;
    let data = snapshot.as_data();
    let seal = OwnedSnapshotSeal {
        version: 1,
        kind: "vlajmaji-owned-snapshot".to_owned(),
        snapshot_version: data.version,
        journal_schema: data.journal_schema,
        dictionary_version: DICTIONARY_VERSION,
        registry_version: REGISTRY_VERSION,
        source: data.source.clone(),
        head: data.head.clone(),
        revision_ids: data
            .commits
            .iter()
            .map(|commit| commit.revision.clone())
            .collect(),
        snapshot_sha256: sha256(&payload),
        snapshot_bytes: payload.len() as u64,
    }
    .to_json_bytes()?;
    Ok(SealedOwnedSnapshot {
        snapshot: payload,
        seal,
    })
}

/// Load and fully validate exact sealed bytes, including canonical payload spelling.
///
/// # Errors
///
/// Returns an error if the seal, payload bytes, payload inventory, or any
/// reconstructed historical state is invalid.
pub fn load_owned_snapshot(
    snapshot_bytes: &[u8],
    seal_bytes: &[u8],
) -> Result<OwnedSnapshot, OwnedSnapshotError> {
    let seal = OwnedSnapshotSeal::from_json_bytes(seal_bytes)?;
    if seal.version != 1 {
        return Err(OwnedSnapshotError::UnsupportedVersion {
            kind: "owned snapshot seal",
            found: seal.version,
        });
    }
    if seal.kind != "vlajmaji-owned-snapshot" {
        return Err(OwnedSnapshotError::InvalidDocument(
            "unexpected owned snapshot seal kind".into(),
        ));
    }
    if !valid_sha256(&seal.snapshot_sha256) {
        return Err(OwnedSnapshotError::InvalidDocument(
            "seal snapshot_sha256 must be lowercase hexadecimal".into(),
        ));
    }
    let actual_length =
        u64::try_from(snapshot_bytes.len()).map_err(|_| OwnedSnapshotError::LengthMismatch {
            expected: seal.snapshot_bytes,
            actual: u64::MAX,
        })?;
    if actual_length != seal.snapshot_bytes {
        return Err(OwnedSnapshotError::LengthMismatch {
            expected: seal.snapshot_bytes,
            actual: actual_length,
        });
    }
    let actual_hash = sha256(snapshot_bytes);
    if actual_hash != seal.snapshot_sha256 {
        return Err(OwnedSnapshotError::HashMismatch {
            expected: seal.snapshot_sha256,
            actual: actual_hash,
        });
    }
    let snapshot = OwnedSnapshot::from_json_bytes(snapshot_bytes)?;
    let data = snapshot.as_data();
    let expected_ids: Vec<_> = data
        .commits
        .iter()
        .map(|commit| commit.revision.clone())
        .collect();
    let inventory_matches = seal.snapshot_version == data.version
        && seal.journal_schema == data.journal_schema
        && seal.dictionary_version == DICTIONARY_VERSION
        && seal.registry_version == REGISTRY_VERSION
        && seal.source == data.source
        && seal.head == data.head
        && seal.revision_ids == expected_ids;
    if !inventory_matches {
        let field = if seal.snapshot_version != data.version {
            "snapshot_version"
        } else if seal.journal_schema != data.journal_schema {
            "journal_schema"
        } else if seal.dictionary_version != DICTIONARY_VERSION {
            "dictionary_version"
        } else if seal.registry_version != REGISTRY_VERSION {
            "registry_version"
        } else if seal.source != data.source {
            "source"
        } else if seal.head != data.head {
            "head"
        } else {
            "revision_ids"
        };
        return Err(OwnedSnapshotError::InventoryMismatch { field });
    }
    if snapshot.to_json_bytes()? != snapshot_bytes {
        return Err(OwnedSnapshotError::NonCanonical);
    }
    Ok(snapshot)
}

fn validate_snapshot(data: &OwnedSnapshotData) -> Result<(), OwnedSnapshotError> {
    if data.version != 1 {
        return Err(OwnedSnapshotError::UnsupportedVersion {
            kind: "owned snapshot",
            found: data.version,
        });
    }
    if !matches!(data.journal_schema, 1 | 2) {
        return Err(OwnedSnapshotError::UnsupportedVersion {
            kind: "owned journal schema",
            found: data.journal_schema,
        });
    }
    let final_document = data.dictionary.document();
    let registry = data.dictionary.registry().as_document();
    if final_document.legacy_archive.is_some() {
        return Err(OwnedSnapshotError::History(
            "owned base cannot contain a legacy archive".into(),
        ));
    }
    drop(final_document);
    if registry.sources.len() != 1 || registry.sources[0].origin != SourceOrigin::OwnedDraft {
        return Err(OwnedSnapshotError::History(
            "base must have exactly one OwnedDraft source".into(),
        ));
    }
    if registry.sources[0].id != data.source {
        return Err(OwnedSnapshotError::InventoryMismatch { field: "source" });
    }
    if data.commits.is_empty() {
        return Err(OwnedSnapshotError::History("missing commits".into()));
    }
    let revisions = &data.dictionary.metadata().owned_revisions;
    if revisions.len() != data.commits.len() {
        return Err(OwnedSnapshotError::LengthMismatch {
            expected: revisions.len() as u64,
            actual: data.commits.len() as u64,
        });
    }
    if data.head != revisions.last().expect("nonempty").id {
        return Err(OwnedSnapshotError::InventoryMismatch { field: "head" });
    }
    let expected_catalogue_revision = u64::try_from(data.commits.len() - 1)
        .map_err(|_| OwnedSnapshotError::History("commit count exceeds catalogue range".into()))?;
    let final_metadata = data.dictionary.metadata();
    if final_metadata.catalogue_revision != expected_catalogue_revision
        || final_metadata.owned_heads.len() != 1
        || final_metadata.owned_heads[0].source != data.source
        || final_metadata.owned_heads[0].revision != data.head
    {
        return Err(OwnedSnapshotError::InventoryMismatch {
            field: "dictionary head or catalogue_revision",
        });
    }
    if data.journal_schema == 1 && data.curation.is_some() {
        return Err(OwnedSnapshotError::History(
            "schema-one snapshot cannot contain curation".into(),
        ));
    }
    if data.journal_schema == 2 && data.curation.is_none() {
        return Err(OwnedSnapshotError::History(
            "schema-two snapshot requires curation, including empty migrated history".into(),
        ));
    }
    if let Some(curation) = &data.curation {
        validate_curation_inventory(data, curation)?;
    }

    let mut previous_rosters: Option<Vec<u64>> = None;
    let mut previous_state: Option<Dictionary> = None;
    for (index, (commit, revision)) in data.commits.iter().zip(revisions).enumerate() {
        if commit.revision != revision.id {
            return Err(OwnedSnapshotError::History(format!(
                "commit {index} revision differs from the base revision chain"
            )));
        }
        if !valid_sha256(&commit.state_sha256) {
            return Err(OwnedSnapshotError::History(format!(
                "commit {index} state hash is not lowercase SHA-256"
            )));
        }
        validate_roster(
            commit,
            &data.dictionary,
            previous_rosters.as_deref(),
            index + 1 == data.commits.len(),
        )?;
        let mut state = build_state(data, index)?;
        let bytes = state.to_json_bytes()?;
        let actual_hash = sha256(&bytes);
        if actual_hash != commit.state_sha256 {
            return Err(OwnedSnapshotError::HashMismatch {
                expected: commit.state_sha256.clone(),
                actual: actual_hash,
            });
        }
        if index == 0 {
            validate_genesis(&state)?;
        } else if let Some(old) = &previous_state {
            validate_adjacent(old, &state)?;
        }
        if let Some(curation) = &data.curation {
            let validated = build_curation(data, curation, index, state)?;
            state = validated.into_data().dictionary;
        }
        previous_rosters = Some(commit.prefixes.iter().map(|prefix| prefix.length).collect());
        previous_state = Some(state);
    }
    if data.commits.last().map(|commit| &commit.revision) != Some(&data.head) {
        return Err(OwnedSnapshotError::InventoryMismatch { field: "head" });
    }
    Ok(())
}

fn validate_curation_inventory(
    snapshot: &OwnedSnapshotData,
    curation: &OwnedSnapshotCuration,
) -> Result<(), OwnedSnapshotError> {
    if curation.version != 1 {
        return Err(OwnedSnapshotError::UnsupportedVersion {
            kind: "owned curation",
            found: curation.version,
        });
    }
    let expected = snapshot.commits.len();
    for actual in [curation.prefixes.len(), curation.revisions.len()] {
        if actual != expected {
            return Err(OwnedSnapshotError::LengthMismatch {
                expected: expected as u64,
                actual: actual as u64,
            });
        }
    }
    let mut previous_contexts = 0_u64;
    let mut previous_records = 0_u64;
    for (index, ((prefix, revision), commit)) in curation
        .prefixes
        .iter()
        .zip(&curation.revisions)
        .zip(&snapshot.commits)
        .enumerate()
    {
        if prefix.revision != commit.revision || revision.revision != commit.revision {
            return Err(OwnedSnapshotError::History(format!(
                "curation revision {index} does not match the base commit"
            )));
        }
        if prefix.contexts_length > i64::MAX as u64
            || prefix.records_length > i64::MAX as u64
            || prefix.contexts_length > curation.contexts.len() as u64
            || prefix.records_length > curation.records.len() as u64
            || prefix.contexts_length < previous_contexts
            || prefix.records_length < previous_records
            || !valid_sha256(&prefix.curation_sha256)
        {
            return Err(OwnedSnapshotError::History(format!(
                "curation cutoff {index} is invalid"
            )));
        }
        previous_contexts = prefix.contexts_length;
        previous_records = prefix.records_length;
    }
    if previous_contexts != curation.contexts.len() as u64
        || previous_records != curation.records.len() as u64
    {
        return Err(OwnedSnapshotError::History(
            "final curation cutoff must equal every final row vector".into(),
        ));
    }
    Ok(())
}

fn validate_roster(
    commit: &OwnedSnapshotCommit,
    final_dictionary: &Dictionary,
    previous: Option<&[u64]>,
    is_final: bool,
) -> Result<(), OwnedSnapshotError> {
    if commit.prefixes.len() != SECTION_NAMES.len() {
        return Err(OwnedSnapshotError::LengthMismatch {
            expected: SECTION_NAMES.len() as u64,
            actual: commit.prefixes.len() as u64,
        });
    }
    let mut lengths = Vec::with_capacity(SECTION_NAMES.len());
    for (index, (prefix, section)) in commit.prefixes.iter().zip(SECTION_NAMES).enumerate() {
        if prefix.section != section {
            return Err(OwnedSnapshotError::History(format!(
                "commit roster section {index} must be {section}"
            )));
        }
        let length = usize::try_from(prefix.length).map_err(|_| {
            OwnedSnapshotError::History(format!("section {section} length does not fit memory"))
        })?;
        if prefix.length > i64::MAX as u64 {
            return Err(OwnedSnapshotError::History(format!(
                "section {section} length exceeds SQLite INTEGER"
            )));
        }
        let final_length = section_length(final_dictionary, section)?;
        if length > final_length {
            return Err(OwnedSnapshotError::History(format!(
                "section {section} prefix exceeds final rows"
            )));
        }
        if previous.is_some_and(|prior| prefix.length < prior[index]) {
            return Err(OwnedSnapshotError::History(format!(
                "section {section} prefix decreases"
            )));
        }
        lengths.push(prefix.length);
    }
    if is_final
        && lengths.iter().zip(SECTION_NAMES).any(|(length, section)| {
            *length as usize != section_length(final_dictionary, section).unwrap_or(usize::MAX)
        })
    {
        return Err(OwnedSnapshotError::History(
            "final commit roster must equal every final dictionary vector".into(),
        ));
    }
    Ok(())
}

fn section_length(dictionary: &Dictionary, section: &str) -> Result<usize, OwnedSnapshotError> {
    let registry = dictionary.registry().as_document();
    let metadata = dictionary.metadata();
    let length = match section {
        "registry.sources" => registry.sources.len(),
        "registry.contributors" => registry.contributors.len(),
        "registry.distributors" => registry.distributors.len(),
        "registry.aliases" => registry.aliases.len(),
        "registry.lineage" => registry.lineage.len(),
        "registry.snapshots" => registry.snapshots.len(),
        "registry.occurrences" => registry.occurrences.len(),
        "captures" => dictionary.captures().len(),
        "metadata.code_notices" => metadata.code_notices.len(),
        "metadata.collections" => metadata.collections.len(),
        "metadata.components" => metadata.components.len(),
        "metadata.releases" => metadata.releases.len(),
        "metadata.entries" => metadata.entries.len(),
        "metadata.assertions" => metadata.assertions.len(),
        "metadata.contexts" => metadata.contexts.len(),
        "metadata.frames" => metadata.frames.len(),
        "metadata.forms" => metadata.forms.len(),
        "metadata.etymologies" => metadata.etymologies.len(),
        "metadata.relations" => metadata.relations.len(),
        "metadata.owned_revisions" => metadata.owned_revisions.len(),
        "metadata.editorial_senses" => metadata.editorial_senses.len(),
        "metadata.computations" => metadata.computations.len(),
        "metadata.annotations" => metadata.annotations.len(),
        "metadata.extensions" => metadata.extensions.len(),
        _ => {
            return Err(OwnedSnapshotError::History(format!(
                "unknown section {section}"
            )));
        }
    };
    Ok(length)
}

fn build_state(data: &OwnedSnapshotData, index: usize) -> Result<Dictionary, OwnedSnapshotError> {
    let commit = data
        .commits
        .get(index)
        .ok_or_else(|| OwnedSnapshotError::History("commit index out of range".into()))?;
    let mut document = data.dictionary.document();
    let mut registry = document.registry.document();
    for prefix in &commit.prefixes {
        let length = usize::try_from(prefix.length)
            .map_err(|_| OwnedSnapshotError::History("section length exceeds memory".into()))?;
        match prefix.section.as_str() {
            "registry.sources" => registry.sources.truncate(length),
            "registry.contributors" => registry.contributors.truncate(length),
            "registry.distributors" => registry.distributors.truncate(length),
            "registry.aliases" => registry.aliases.truncate(length),
            "registry.lineage" => registry.lineage.truncate(length),
            "registry.snapshots" => registry.snapshots.truncate(length),
            "registry.occurrences" => registry.occurrences.truncate(length),
            "captures" => document.captures.truncate(length),
            "metadata.code_notices" => document.metadata.code_notices.truncate(length),
            "metadata.collections" => document.metadata.collections.truncate(length),
            "metadata.components" => document.metadata.components.truncate(length),
            "metadata.releases" => document.metadata.releases.truncate(length),
            "metadata.entries" => document.metadata.entries.truncate(length),
            "metadata.assertions" => document.metadata.assertions.truncate(length),
            "metadata.contexts" => document.metadata.contexts.truncate(length),
            "metadata.frames" => document.metadata.frames.truncate(length),
            "metadata.forms" => document.metadata.forms.truncate(length),
            "metadata.etymologies" => document.metadata.etymologies.truncate(length),
            "metadata.relations" => document.metadata.relations.truncate(length),
            "metadata.owned_revisions" => document.metadata.owned_revisions.truncate(length),
            "metadata.editorial_senses" => document.metadata.editorial_senses.truncate(length),
            "metadata.computations" => document.metadata.computations.truncate(length),
            "metadata.annotations" => document.metadata.annotations.truncate(length),
            "metadata.extensions" => document.metadata.extensions.truncate(length),
            _ => {
                return Err(OwnedSnapshotError::History(format!(
                    "unknown section {}",
                    prefix.section
                )));
            }
        }
    }
    document.registry = SourceRegistry::from_document(registry)
        .map_err(|error| OwnedSnapshotError::History(error.to_string()))?;
    document.metadata.catalogue_revision = index as u64;
    document.metadata.owned_heads = vec![OwnedHead {
        source: data.source.clone(),
        revision: commit.revision.clone(),
    }];
    Dictionary::from_document(document).map_err(OwnedSnapshotError::Dictionary)
}

fn validate_genesis(dictionary: &Dictionary) -> Result<(), OwnedSnapshotError> {
    let document = dictionary.document();
    let registry = dictionary.registry().as_document();
    let metadata = dictionary.metadata();
    if document.legacy_archive.is_some()
        || registry.sources.len() != 1
        || registry.sources[0].origin != SourceOrigin::OwnedDraft
        || metadata.owned_heads.len() != 1
        || metadata.owned_revisions.len() != 1
    {
        return Err(OwnedSnapshotError::History(
            "invalid owned genesis shape".into(),
        ));
    }
    drop(document);
    let source = &registry.sources[0].id;
    let head = &metadata.owned_heads[0];
    let revision = &metadata.owned_revisions[0];
    if &head.source != source
        || head.revision != revision.id
        || &revision.source != source
        || revision.parent.is_some()
        || !revision.tombstones.is_empty()
        || metadata.catalogue_revision != 0
    {
        return Err(OwnedSnapshotError::History(
            "invalid owned genesis head".into(),
        ));
    }
    if registry
        .contributors
        .iter()
        .all(|row| row.id != revision.editor)
    {
        return Err(OwnedSnapshotError::History(
            "owned genesis editor is absent".into(),
        ));
    }
    let collections: std::collections::BTreeSet<_> = metadata
        .collections
        .iter()
        .filter(|row| row.source == *source)
        .map(|row| row.id.clone())
        .collect();
    let components: std::collections::BTreeSet<_> = metadata
        .components
        .iter()
        .filter(|row| collections.contains(&row.collection))
        .map(|row| row.id.clone())
        .collect();
    let releases: Vec<_> = metadata
        .releases
        .iter()
        .filter(|row| components.contains(&row.component))
        .collect();
    if collections.is_empty()
        || components.is_empty()
        || releases.is_empty()
        || releases
            .iter()
            .any(|release| release.content_notices.is_empty())
    {
        return Err(OwnedSnapshotError::History(
            "owned genesis requires collection, component, release, and content notice".into(),
        ));
    }
    crate::owned_curation::validate_owned_base(dictionary).map_err(OwnedSnapshotError::Curation)
}

fn validate_adjacent(old: &Dictionary, new: &Dictionary) -> Result<(), OwnedSnapshotError> {
    let old_registry = old.registry().as_document();
    let new_registry = new.registry().as_document();
    let old_metadata = old.metadata();
    let new_metadata = new.metadata();
    let source = old_registry
        .sources
        .first()
        .ok_or_else(|| OwnedSnapshotError::History("missing prior source".into()))?
        .id
        .clone();
    if old_registry.sources.len() != 1
        || new_registry.sources.len() != 1
        || new_registry.sources[0].id != source
        || new_metadata.owned_revisions.len() != old_metadata.owned_revisions.len() + 1
        || new_metadata.catalogue_revision != old_metadata.catalogue_revision + 1
    {
        return Err(OwnedSnapshotError::History(
            "adjacent states must append one revision for the same source".into(),
        ));
    }
    let prior = old_metadata
        .owned_revisions
        .last()
        .ok_or_else(|| OwnedSnapshotError::History("missing prior revision".into()))?;
    let next = new_metadata.owned_revisions.last().expect("length checked");
    if next.source != source
        || next.parent.as_ref() != Some(&prior.id)
        || new_metadata.owned_heads.len() != 1
        || new_metadata.owned_heads[0].source != source
        || new_metadata.owned_heads[0].revision != next.id
        || new_metadata.owned_revisions[..old_metadata.owned_revisions.len()]
            != old_metadata.owned_revisions[..]
    {
        return Err(OwnedSnapshotError::History(
            "adjacent revision must parent and become the sole prior head".into(),
        ));
    }
    let old_active: Vec<_> = prior.assertions.iter().map(ToString::to_string).collect();
    let new_active: Vec<_> = next.assertions.iter().map(ToString::to_string).collect();
    let expected_tombstones: Vec<_> = old_active
        .iter()
        .filter(|id| !new_active.contains(id))
        .cloned()
        .collect();
    let actual_tombstones: Vec<_> = next.tombstones.iter().map(ToString::to_string).collect();
    if actual_tombstones != expected_tombstones {
        return Err(OwnedSnapshotError::History(
            "new tombstones must exactly list removed prior assertions in order".into(),
        ));
    }
    let prior_assertions = old_metadata.assertions.len();
    if new_metadata.assertions[prior_assertions..]
        .iter()
        .any(|assertion| !next.assertions.contains(&assertion.id))
    {
        return Err(OwnedSnapshotError::History(
            "new assertions must be active at their creation revision".into(),
        ));
    }
    crate::owned_curation::validate_owned_base(new).map_err(OwnedSnapshotError::Curation)
}

fn build_curation(
    snapshot: &OwnedSnapshotData,
    curation: &OwnedSnapshotCuration,
    index: usize,
    dictionary: Dictionary,
) -> Result<OwnedCurationDocument, OwnedSnapshotError> {
    let prefix = curation
        .prefixes
        .get(index)
        .ok_or_else(|| OwnedSnapshotError::History("missing curation prefix".into()))?;
    let commit = snapshot
        .commits
        .get(index)
        .ok_or_else(|| OwnedSnapshotError::History("missing matching base commit".into()))?;
    if prefix.revision != commit.revision {
        return Err(OwnedSnapshotError::History(
            "curation prefix revision differs from base history".into(),
        ));
    }
    let contexts_len = usize::try_from(prefix.contexts_length)
        .map_err(|_| OwnedSnapshotError::History("context cutoff exceeds memory".into()))?;
    let records_len = usize::try_from(prefix.records_length)
        .map_err(|_| OwnedSnapshotError::History("record cutoff exceeds memory".into()))?;
    if contexts_len > curation.contexts.len() || records_len > curation.records.len() {
        return Err(OwnedSnapshotError::History(
            "curation cutoff exceeds final rows".into(),
        ));
    }
    let document = OwnedCurationDocument::from_data(OwnedCurationData {
        version: curation.version,
        dictionary,
        contexts: curation.contexts[..contexts_len].to_vec(),
        records: curation.records[..records_len].to_vec(),
        revisions: curation.revisions[..=index].to_vec(),
    })?;
    let actual = document.state_sha256()?;
    if actual != prefix.curation_sha256 {
        return Err(OwnedSnapshotError::HashMismatch {
            expected: prefix.curation_sha256.clone(),
            actual,
        });
    }
    Ok(document)
}

fn sha256(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(64);
    for byte in digest {
        use fmt::Write as _;
        let _ = write!(output, "{byte:02x}");
    }
    output
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
