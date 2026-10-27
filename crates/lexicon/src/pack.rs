//! Versioned, locally sealed dictionary packs and lossless model composition.

use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::dictionary::{DictionaryDocument, UnverifiedDictionaryDocument};
use crate::registry::{RegistryDocument, SourceRegistry, validate_identity_prefix};
use crate::{
    CaptureRef, Dictionary, DictionaryError, DictionaryMetadata, EntityId, RegistryError, SourceId,
};

/// Stable identifier for a supported independent source pack.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PackId {
    /// Full Lensisku content distributed through the pinned jbotci export.
    LensiskuJbotci,
    /// LLG source material.
    Llg,
    /// La-Lojban source material.
    LaLojban,
    /// Sutysisku source material.
    Sutysisku,
}

impl PackId {
    /// Returns this pack's stable kebab-case identifier.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::LensiskuJbotci => "lensisku-jbotci",
            Self::Llg => "llg",
            Self::LaLojban => "la-lojban",
            Self::Sutysisku => "sutysisku",
        }
    }
}

impl fmt::Display for PackId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Describes whether a sealed pack contains synthetic or full source material.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PackPayloadKind {
    /// A small fixture created for tests and local preparation.
    Synthetic,
    /// The complete locally pinned source payload.
    Full,
}

/// Integrity inventory for one canonical dictionary release.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PackSeal {
    /// Seal schema version. Only version 1 is currently supported.
    pub version: u32,
    /// Pack this release belongs to.
    pub pack_id: PackId,
    /// Distinguishes synthetic preparation from a full local payload.
    pub payload_kind: PackPayloadKind,
    /// Dictionary model schema version.
    pub dictionary_version: u32,
    /// Registry model schema version.
    pub registry_version: u32,
    /// SHA-256 of the complete canonical dictionary JSON bytes.
    pub release_sha256: String,
    /// Exact canonical dictionary JSON byte length.
    pub release_bytes: u64,
    /// First-seen source IDs in the release document.
    pub source_ids: Vec<SourceId>,
    /// First-seen release IDs in the release document.
    pub release_ids: Vec<EntityId>,
    /// First-seen capture identities in the release document.
    pub captures: Vec<CaptureRef>,
}

/// Canonical release bytes paired with their strict version-1 seal bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SealedRelease {
    /// Canonical validated dictionary JSON.
    pub release: Vec<u8>,
    /// Canonical strict seal JSON.
    pub seal: Vec<u8>,
}

/// Typed failure while sealing, loading, or composing a dictionary pack.
#[derive(Debug)]
#[non_exhaustive]
pub enum PackError {
    /// Seal JSON is malformed, duplicated, or outside the strict wire shape.
    InvalidSeal(String),
    /// A seal or model schema version is unsupported.
    UnsupportedVersion {
        /// Model layer whose version was not supported.
        kind: &'static str,
        /// Unsupported wire version.
        version: u32,
    },
    /// The caller requested a different pack from the one named by the seal.
    PackMismatch {
        /// Pack requested by the caller.
        expected: PackId,
        /// Pack named in the seal.
        actual: PackId,
    },
    /// Canonical release bytes do not match the sealed length.
    LengthMismatch {
        /// Byte length recorded in the seal.
        expected: u64,
        /// Actual release byte length.
        actual: u64,
    },
    /// Canonical release bytes do not match the sealed digest.
    HashMismatch {
        /// SHA-256 recorded in the seal.
        expected: String,
        /// Actual release SHA-256.
        actual: String,
    },
    /// The strict source/release/capture inventory does not match the model.
    InventoryMismatch {
        /// Inventory field that differed.
        field: &'static str,
    },
    /// Validated dictionary model construction or serialization failed.
    Dictionary(DictionaryError),
    /// Registry construction or validation failed.
    Registry(RegistryError),
    /// Two rows claim one identity but have unequal complete bodies.
    Conflict {
        /// Model identity category.
        category: &'static str,
        /// Conflicting identity key.
        key: String,
    },
    /// A legacy archive is outside the sealed independent-pack composition model.
    LegacyArchiveUnsupported,
}

impl fmt::Display for PackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSeal(detail) => write!(f, "invalid pack seal: {detail}"),
            Self::UnsupportedVersion { kind, version } => {
                write!(f, "unsupported {kind} version {version}")
            }
            Self::PackMismatch { expected, actual } => {
                write!(f, "pack mismatch: expected {expected}, got {actual}")
            }
            Self::LengthMismatch { expected, actual } => {
                write!(
                    f,
                    "release byte length mismatch: expected {expected}, got {actual}"
                )
            }
            Self::HashMismatch { expected, actual } => {
                write!(
                    f,
                    "release SHA-256 mismatch: expected {expected}, got {actual}"
                )
            }
            Self::InventoryMismatch { field } => {
                write!(f, "release {field} inventory differs from its seal")
            }
            Self::Dictionary(error) => write!(f, "invalid dictionary: {error}"),
            Self::Registry(error) => write!(f, "invalid registry: {error}"),
            Self::Conflict { category, key } => {
                write!(f, "conflicting {category} identity {key}")
            }
            Self::LegacyArchiveUnsupported => {
                f.write_str("legacy archive cannot be combined as an independent pack")
            }
        }
    }
}

impl std::error::Error for PackError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Dictionary(error) => Some(error),
            Self::Registry(error) => Some(error),
            _ => None,
        }
    }
}

impl From<DictionaryError> for PackError {
    fn from(error: DictionaryError) -> Self {
        Self::Dictionary(error)
    }
}

impl From<RegistryError> for PackError {
    fn from(error: RegistryError) -> Self {
        Self::Registry(error)
    }
}

macro_rules! merge_rows {
    ($source:ident, $dest:ident, $indexes:ident, $( $field:ident, $category:literal );+ $(;)?) => {
        $(
            for row in $source.$field {
                push_keyed(
                    &mut $dest.$field,
                    &mut $indexes.$field,
                    row,
                    |row| row.id.to_string(),
                    $category,
                )?;
            }
        )+
    };
}

impl PackSeal {
    /// Parses strict versioned JSON seal bytes, rejecting duplicate keys.
    ///
    /// # Errors
    ///
    /// Returns [`PackError::InvalidSeal`] for malformed, duplicate-key, or
    /// unknown-field JSON and [`PackError::UnsupportedVersion`] for a version
    /// this implementation cannot read.
    pub fn from_json_bytes(bytes: &[u8]) -> Result<Self, PackError> {
        let text = std::str::from_utf8(bytes)
            .map_err(|error| PackError::InvalidSeal(error.to_string()))?;
        crate::check_duplicate_keys(text)
            .map_err(|error| PackError::InvalidSeal(error.to_string()))?;
        let seal: Self = serde_json::from_str(text)
            .map_err(|error| PackError::InvalidSeal(error.to_string()))?;
        if seal.version != 1 {
            return Err(PackError::UnsupportedVersion {
                kind: "pack seal",
                version: seal.version,
            });
        }
        if seal.dictionary_version != 1 {
            return Err(PackError::UnsupportedVersion {
                kind: "dictionary",
                version: seal.dictionary_version,
            });
        }
        if seal.registry_version != 1 {
            return Err(PackError::UnsupportedVersion {
                kind: "registry",
                version: seal.registry_version,
            });
        }
        if !valid_sha256(&seal.release_sha256) {
            return Err(PackError::InvalidSeal(
                "release_sha256 must be 64 lowercase hexadecimal characters".into(),
            ));
        }
        Ok(seal)
    }
}

/// Serializes a validated dictionary and records its integrity inventory.
///
/// The result is not a signature, publisher attestation, or license judgment.
///
/// # Errors
///
/// Returns a typed dictionary serialization or seal serialization error.
pub fn seal_release(
    pack_id: PackId,
    payload_kind: PackPayloadKind,
    dictionary: &Dictionary,
) -> Result<SealedRelease, PackError> {
    let release = dictionary.to_json_bytes()?;
    let registry = dictionary.registry().as_document();
    let seal = PackSeal {
        version: 1,
        pack_id,
        payload_kind,
        dictionary_version: 1,
        registry_version: registry.version,
        release_sha256: sha256(&release),
        release_bytes: release.len() as u64,
        source_ids: registry.sources.iter().map(|row| row.id.clone()).collect(),
        release_ids: dictionary
            .metadata()
            .releases
            .iter()
            .map(|row| row.id.clone())
            .collect(),
        captures: dictionary
            .captures()
            .iter()
            .map(|capture| capture.identity.clone())
            .collect(),
    };
    let seal_bytes =
        serde_json::to_vec(&seal).map_err(|error| PackError::InvalidSeal(error.to_string()))?;
    Ok(SealedRelease {
        release,
        seal: seal_bytes,
    })
}

/// Validates a seal and loads its strict, complete dictionary model.
///
/// # Errors
///
/// Returns a typed error for malformed seals, pack/version/length/hash/inventory
/// mismatches, or invalid registry/model data.
pub fn load_sealed_release(
    expected: PackId,
    release: &[u8],
    seal_bytes: &[u8],
) -> Result<Dictionary, PackError> {
    let seal = PackSeal::from_json_bytes(seal_bytes)?;
    if seal.pack_id != expected {
        return Err(PackError::PackMismatch {
            expected,
            actual: seal.pack_id,
        });
    }
    let actual_length = release.len() as u64;
    if seal.release_bytes != actual_length {
        return Err(PackError::LengthMismatch {
            expected: seal.release_bytes,
            actual: actual_length,
        });
    }
    let actual_hash = sha256(release);
    if seal.release_sha256 != actual_hash {
        return Err(PackError::HashMismatch {
            expected: seal.release_sha256,
            actual: actual_hash,
        });
    }
    let dictionary = Dictionary::from_json_bytes(release)?;
    let registry = dictionary.registry().as_document();
    if 1 != seal.dictionary_version {
        return Err(PackError::InventoryMismatch {
            field: "dictionary version",
        });
    }
    if registry.version != seal.registry_version {
        return Err(PackError::InventoryMismatch {
            field: "registry version",
        });
    }
    let sources: Vec<_> = registry.sources.iter().map(|row| row.id.clone()).collect();
    if sources != seal.source_ids {
        return Err(PackError::InventoryMismatch { field: "source" });
    }
    let releases: Vec<_> = dictionary
        .metadata()
        .releases
        .iter()
        .map(|row| row.id.clone())
        .collect();
    if releases != seal.release_ids {
        return Err(PackError::InventoryMismatch { field: "release" });
    }
    let captures: Vec<_> = dictionary
        .captures()
        .iter()
        .map(|capture| capture.identity.clone())
        .collect();
    if captures != seal.captures {
        return Err(PackError::InventoryMismatch { field: "capture" });
    }
    Ok(dictionary)
}

/// Consumes validated dictionaries into one strictly validated dictionary.
///
/// ID-bearing rows are exact-row deduplicated by identity in first-seen order;
/// same-key rows with unequal bodies are conflicts, and no winner is chosen.
/// Unkeyed extension rows instead use an ordered maximum-multiplicity union:
/// repetitions within an input are preserved, while combining inputs adds only
/// copies beyond the greatest multiplicity observed in any one input.
///
/// # Errors
///
/// Returns a typed identity conflict, unsupported legacy archive, registry
/// error, or final dictionary validation error.
pub fn compose_dictionaries(inputs: Vec<Dictionary>) -> Result<Dictionary, PackError> {
    let documents = inputs
        .into_iter()
        .map(|input| {
            let document = input.into_document();
            DictionaryDocument {
                version: document.version,
                legacy_archive: document.legacy_archive,
                registry: document.registry.into_document(),
                captures: document.captures,
                metadata: document.metadata,
            }
        })
        .collect();
    let document = compose_unverified_documents(documents)?;
    let registry = SourceRegistry::from_document(document.registry)?;
    let validated = DictionaryDocument {
        version: document.version,
        legacy_archive: document.legacy_archive,
        registry,
        captures: document.captures,
        metadata: document.metadata,
    };
    Dictionary::from_document(validated).map_err(PackError::Dictionary)
}

/// Compose typed, unverified dictionary documents without claiming that the
/// complete model graph has been validated.
///
/// This operation checks only format/version and merge identity invariants.
/// Call `Dictionary::from_document` when full graph validation is required.
///
/// # Errors
///
/// Returns an unsupported legacy/version or conflicting identity error.
pub fn compose_unverified_documents(
    inputs: Vec<UnverifiedDictionaryDocument>,
) -> Result<UnverifiedDictionaryDocument, PackError> {
    let mut output = DictionaryDocument {
        version: 1,
        legacy_archive: None,
        registry: RegistryDocument::default(),
        captures: Vec::new(),
        metadata: DictionaryMetadata::default(),
    };
    let mut registry = RegistryDocument::default();
    let mut metadata_output = DictionaryMetadata::default();
    let mut meta_indexes = MetadataIndexes::default();
    let mut capture_index = BTreeMap::new();
    let mut source_index = BTreeMap::new();
    let mut contributor_index = BTreeMap::new();
    let mut distributor_index = BTreeMap::new();
    let mut alias_index = BTreeMap::new();
    let mut snapshot_index = BTreeMap::new();
    let mut occurrence_index = BTreeMap::new();
    let mut extension_buckets = BTreeMap::<(EntityId, String), Vec<usize>>::new();
    let mut extension_multiplicities = BTreeMap::<usize, usize>::new();

    for document in inputs {
        if document.legacy_archive.is_some() {
            return Err(PackError::LegacyArchiveUnsupported);
        }
        if document.version != 1 {
            return Err(PackError::UnsupportedVersion {
                kind: "dictionary",
                version: document.version,
            });
        }
        metadata_output.catalogue_revision = metadata_output
            .catalogue_revision
            .max(document.metadata.catalogue_revision);
        let registry_doc = document.registry;
        if registry_doc.version != 1 {
            return Err(PackError::UnsupportedVersion {
                kind: "registry",
                version: registry_doc.version,
            });
        }
        ensure_unique(&registry_doc.sources, "source", |row| row.id.to_string())?;
        ensure_unique(&registry_doc.contributors, "contributor", |row| {
            row.id.to_string()
        })?;
        ensure_unique(&registry_doc.distributors, "distributor", |row| {
            row.id.to_string()
        })?;
        ensure_unique(&registry_doc.aliases, "alias", |row| {
            composite_key([row.source.as_str(), row.text.as_str()])
        })?;
        ensure_unique(&registry_doc.snapshots, "snapshot", |row| {
            composite_key([row.snapshot.namespace(), row.snapshot.sha256()])
        })?;
        ensure_unique(&registry_doc.occurrences, "occurrence attribution", |row| {
            occurrence_key(&row.occurrence)
        })?;
        for row in registry_doc.sources {
            push_keyed(
                &mut registry.sources,
                &mut source_index,
                row,
                |row| row.id.to_string(),
                "source",
            )?;
        }
        for row in registry_doc.contributors {
            push_keyed(
                &mut registry.contributors,
                &mut contributor_index,
                row,
                |row| row.id.to_string(),
                "contributor",
            )?;
        }
        for row in registry_doc.distributors {
            push_keyed(
                &mut registry.distributors,
                &mut distributor_index,
                row,
                |row| row.id.to_string(),
                "distributor",
            )?;
        }
        for row in registry_doc.aliases {
            push_keyed(
                &mut registry.aliases,
                &mut alias_index,
                row,
                |row| composite_key([row.source.as_str(), row.text.as_str()]),
                "alias",
            )?;
        }
        extend_exact(&mut registry.lineage, registry_doc.lineage);
        for row in registry_doc.snapshots {
            push_keyed(
                &mut registry.snapshots,
                &mut snapshot_index,
                row,
                |row| composite_key([row.snapshot.namespace(), row.snapshot.sha256()]),
                "snapshot",
            )?;
        }
        for row in registry_doc.occurrences {
            push_keyed(
                &mut registry.occurrences,
                &mut occurrence_index,
                row,
                |row| occurrence_key(&row.occurrence),
                "occurrence attribution",
            )?;
        }
        ensure_unique(&document.captures, "capture", |row| {
            composite_key([
                row.identity.namespace.as_str(),
                row.identity.sha256.as_str(),
            ])
        })?;
        for capture in document.captures {
            push_keyed(
                &mut output.captures,
                &mut capture_index,
                capture,
                |row| {
                    composite_key([
                        row.identity.namespace.as_str(),
                        row.identity.sha256.as_str(),
                    ])
                },
                "capture",
            )?;
        }
        let metadata = document.metadata;
        macro_rules! unique_ids { ($($field:ident => $category:literal),+ $(,)?) => { $(ensure_unique(&metadata.$field, $category, |row| row.id.to_string())?;)+ }; }
        unique_ids!(collections=>"collection",components=>"component",releases=>"release",entries=>"entry",assertions=>"assertion",contexts=>"context",frames=>"frame",forms=>"form",etymologies=>"etymology",relations=>"relation",owned_revisions=>"owned revision",editorial_senses=>"editorial sense",computations=>"computation",annotations=>"annotation");
        ensure_unique(&metadata.owned_heads, "owned head", |row| {
            row.source.to_string()
        })?;
        extend_exact(&mut metadata_output.code_notices, metadata.code_notices);
        merge_rows!(metadata, metadata_output, meta_indexes,
            collections, "collection";
            components, "component";
            releases, "release";
            entries, "entry";
            assertions, "assertion";
            contexts, "context";
            frames, "frame";
            forms, "form";
            etymologies, "etymology";
            relations, "relation";
            owned_revisions, "owned revision";
            editorial_senses, "editorial sense";
            computations, "computation";
            annotations, "annotation");
        for row in metadata.owned_heads {
            push_keyed(
                &mut metadata_output.owned_heads,
                &mut meta_indexes.owned_heads,
                row,
                |row| row.source.to_string(),
                "owned head",
            )?;
        }
        let mut input_extension_multiplicities = BTreeMap::<usize, usize>::new();
        for row in metadata.extensions {
            let bucket = (row.owner.clone(), row.name.clone());
            let existing = extension_buckets.get(&bucket).and_then(|indices| {
                indices
                    .iter()
                    .copied()
                    .find(|index| metadata_output.extensions[*index] == row)
            });
            if let Some(index) = existing {
                let seen = input_extension_multiplicities.entry(index).or_default();
                *seen += 1;
                if *seen > extension_multiplicities[&index] {
                    metadata_output.extensions.push(row);
                    extension_multiplicities.insert(index, *seen);
                }
            } else {
                let index = metadata_output.extensions.len();
                metadata_output.extensions.push(row);
                extension_buckets.entry(bucket).or_default().push(index);
                extension_multiplicities.insert(index, 1);
                input_extension_multiplicities.insert(index, 1);
            }
        }
    }
    validate_identity_prefix(&registry).map_err(PackError::Registry)?;
    output.metadata = metadata_output;
    output.registry = registry;
    Ok(output)
}

#[derive(Default)]
struct MetadataIndexes {
    collections: BTreeMap<String, usize>,
    components: BTreeMap<String, usize>,
    releases: BTreeMap<String, usize>,
    entries: BTreeMap<String, usize>,
    assertions: BTreeMap<String, usize>,
    contexts: BTreeMap<String, usize>,
    frames: BTreeMap<String, usize>,
    forms: BTreeMap<String, usize>,
    etymologies: BTreeMap<String, usize>,
    relations: BTreeMap<String, usize>,
    owned_revisions: BTreeMap<String, usize>,
    owned_heads: BTreeMap<String, usize>,
    editorial_senses: BTreeMap<String, usize>,
    computations: BTreeMap<String, usize>,
    annotations: BTreeMap<String, usize>,
}

fn push_keyed<T, F>(
    rows: &mut Vec<T>,
    indexes: &mut BTreeMap<String, usize>,
    row: T,
    key: F,
    category: &'static str,
) -> Result<(), PackError>
where
    T: PartialEq,
    F: FnOnce(&T) -> String,
{
    let identity = key(&row);
    if let Some(index) = indexes.get(&identity) {
        if rows[*index] != row {
            return Err(PackError::Conflict {
                category,
                key: identity,
            });
        }
        return Ok(());
    }
    indexes.insert(identity, rows.len());
    rows.push(row);
    Ok(())
}

fn ensure_unique<T, F>(rows: &[T], category: &'static str, mut key: F) -> Result<(), PackError>
where
    F: FnMut(&T) -> String,
{
    let mut seen = std::collections::BTreeSet::new();
    for row in rows {
        let identity = key(row);
        if !seen.insert(identity.clone()) {
            return Err(PackError::Conflict {
                category,
                key: identity,
            });
        }
    }
    Ok(())
}

fn extend_exact<T: PartialEq>(rows: &mut Vec<T>, incoming: Vec<T>) {
    for row in incoming {
        if !rows.contains(&row) {
            rows.push(row);
        }
    }
}

fn occurrence_key(identity: &crate::RecordIdentity) -> String {
    composite_key([
        identity.snapshot().namespace(),
        identity.snapshot().sha256(),
        identity.locator(),
    ])
}

fn composite_key<'a>(parts: impl IntoIterator<Item = &'a str>) -> String {
    let mut key = String::new();
    for part in parts {
        use fmt::Write as _;
        let _ = write!(key, "{}:", part.len());
        key.push_str(part);
    }
    key
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

#[cfg(test)]
mod tests {
    use super::composite_key;

    #[test]
    fn composite_identity_key_is_unambiguous_when_parts_contain_delimiters() {
        assert_ne!(composite_key(["a:b", "c"]), composite_key(["a", "b:c"]));
        assert_eq!(composite_key(["a:b", "c"]), composite_key(["a:b", "c"]));
    }
}
