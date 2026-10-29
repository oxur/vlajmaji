//! Pure, explicit reviewed links and senses over an owned dictionary base.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt;

use serde::de::Deserializer;
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
use sha2::{Digest, Sha256};

use crate::{
    AssertionRecord, CaptureRef, CatalogTarget, CollectionRecord, ComponentRecord, ContributorId,
    Dictionary, DictionaryError, Direction, EditorialSenseRecord, EntityId, EntryRecord,
    ExtensionRecord, LocalRef, OccurrenceAttribution, OccurrenceRef, PackError, RawCapture,
    RawJson, RawOccurrence, RelationKind, RelationRecord, RelationScope, RelationTarget as Target,
    ReleaseRecord, SourceId, SourceOrigin, TextValue,
};

/// A canonical immutable context body named by the curation document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OwnedContextPin {
    /// Stable identity for this declared context.
    pub id: EntityId,
    /// SHA-256 of the complete canonical Dictionary JSON bytes.
    pub model_sha256: String,
    /// Complete source inventory in sorted unique order.
    pub source_ids: Vec<SourceId>,
    /// Complete release inventory in sorted unique order.
    pub release_ids: Vec<EntityId>,
}

/// A caller-supplied real dictionary body for one declared pin.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OwnedContextInput {
    /// Declared context identity.
    pub id: EntityId,
    /// Validated immutable model supplied by the caller.
    pub dictionary: Dictionary,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ContextInputWire {
    id: EntityId,
    dictionary: Box<RawValue>,
}

impl<'de> Deserialize<'de> for OwnedContextInput {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let wire = ContextInputWire::deserialize(deserializer)?;
        let dictionary = Dictionary::from_json_bytes(wire.dictionary.get().as_bytes())
            .map_err(serde::de::Error::custom)?;
        Ok(Self {
            id: wire.id,
            dictionary,
        })
    }
}

impl OwnedContextInput {
    /// Parse a strict `{id,dictionary}` object, preserving original nested bytes.
    pub fn from_json_bytes(bytes: &[u8]) -> Result<Self, OwnedCurationError> {
        let text = std::str::from_utf8(bytes)
            .map_err(|e| OwnedCurationError::InvalidDocument(e.to_string()))?;
        crate::check_duplicate_keys(text)
            .map_err(|e| OwnedCurationError::InvalidDocument(e.to_string()))?;
        serde_json::from_str(text).map_err(|e| OwnedCurationError::InvalidDocument(e.to_string()))
    }

    /// Serialize this typed input as JSON bytes.
    pub fn to_json_bytes(&self) -> Result<Vec<u8>, OwnedCurationError> {
        serde_json::to_vec(self).map_err(|e| OwnedCurationError::InvalidDocument(e.to_string()))
    }
}

/// One exact source-owned or pinned external target.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OwnedPinnedTarget {
    /// Declared immutable context that supplies the target.
    pub context: EntityId,
    /// Exact normalized catalogue target; an occurrence is mandatory.
    pub target: CatalogTarget,
    /// Exact target entry identity.
    pub entry: EntityId,
    /// Exact target assertion identity.
    pub assertion: EntityId,
}

/// One member in an ordered reviewed sense.
// Preserve the issued public wire shape rather than boxing one variant.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum OwnedSenseMember {
    /// An assertion in the owned base.
    OwnedAssertion(EntityId),
    /// An exact assertion in one pinned context.
    External(OwnedPinnedTarget),
}

/// Normalized reviewed operation.
// Preserve the issued public wire shape rather than boxing one variant.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum OwnedCurationKind {
    /// Link one owned reference to a pinned target.
    Link {
        /// Existing owned origin.
        origin: LocalRef,
        /// Declared relation direction.
        direction: Direction,
        /// Exact pinned external target.
        target: OwnedPinnedTarget,
    },
    /// Retain an ordered, all-or-nothing editorial sense group.
    Sense {
        /// Ordered owned or pinned assertions.
        members: Vec<OwnedSenseMember>,
    },
}

/// Immutable reviewed definition with its actual authoring revision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OwnedCurationRecord {
    /// Stable curation identity.
    pub id: EntityId,
    /// Owned base revision at which this definition was created.
    pub revision: EntityId,
    /// Registered contributor who authored the definition.
    pub editor: ContributorId,
    /// Exact review rationale.
    pub rationale: TextValue,
    /// Reviewed operation.
    pub kind: OwnedCurationKind,
}

/// Complete selection at one corresponding owned revision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OwnedCurationRevision {
    /// Matching owned-base revision ID.
    pub revision: EntityId,
    /// Complete active curation selection in declared order.
    pub active: Vec<EntityId>,
    /// Previously active definitions removed at this revision, in prior order.
    pub tombstones: Vec<EntityId>,
}

/// Unvalidated typed input; callers validate through [`OwnedCurationDocument::from_data`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OwnedCurationData {
    /// Curation wire schema version.
    pub version: u32,
    /// Current self-contained owned dictionary base.
    pub dictionary: Dictionary,
    /// Immutable context pins in declaration order.
    pub contexts: Vec<OwnedContextPin>,
    /// Immutable reviewed definitions in creation order.
    pub records: Vec<OwnedCurationRecord>,
    /// Complete selection history, one row per owned revision.
    pub revisions: Vec<OwnedCurationRevision>,
}

/// Validated curation document with invariant-bearing data kept private.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct OwnedCurationDocument(OwnedCurationData);

impl<'de> Deserialize<'de> for OwnedCurationDocument {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::from_data(OwnedCurationData::deserialize(deserializer)?)
            .map_err(serde::de::Error::custom)
    }
}

/// Typed status for one reviewed definition in an explicit materialization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OwnedCurationStatus {
    /// Definition is not selected at the requested revision.
    Inactive,
    /// A required pinned context body was not supplied.
    Unavailable,
    /// Supplied bodies did not establish the exact reviewed target.
    Unresolved,
    /// Exact reviewed identities were established and projected.
    Resolved,
}

/// Typed explanation for absence or mismatch during materialization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OwnedCurationReason {
    /// Required context body is absent.
    MissingContext,
    /// The owned link origin is absent.
    MissingOrigin,
    /// A referenced model entity is absent.
    MissingEntity,
    /// The occurrence belongs to a different source.
    WrongSource,
    /// The pinned release is absent.
    MissingRelease,
    /// The assertion is not a member of the pinned release.
    AssertionNotInRelease,
    /// The exact occurrence identity differs.
    OccurrenceMismatch,
    /// The capture or raw occurrence is unavailable.
    PayloadUnavailable,
    /// A requested native value has no captured attribution.
    NativeIdentityUnknown,
    /// A requested native value conflicts with its attribution.
    NativeIdentityMismatch,
    /// A required Sense member did not resolve.
    MemberUnavailable,
    /// A Link depends on a Sense that did not project.
    DependencyUnavailable,
}

/// One ordered status row retained for every curation record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OwnedCurationResolution {
    /// Definition identity.
    pub record: EntityId,
    /// Materialization status.
    pub status: OwnedCurationStatus,
    /// Exact reason when no projection was made.
    pub reason: Option<OwnedCurationReason>,
    /// First unavailable Sense member, if applicable.
    pub member: Option<u64>,
    /// Whether the normalized row was projected.
    pub projected: bool,
}

/// Validated normalized Dictionary view plus per-record outcomes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnedMaterialization {
    /// Materialization report version.
    pub version: u32,
    /// Sole owned source.
    pub source: SourceId,
    /// Selected owned revision.
    pub revision: EntityId,
    /// Explicit composed output model.
    pub dictionary: Dictionary,
    /// Outcomes in original curation-record order.
    pub resolutions: Vec<OwnedCurationResolution>,
}

/// A curation wire, identity, context, composition, or model failure.
#[derive(Debug)]
#[non_exhaustive]
pub enum OwnedCurationError {
    /// Malformed strict input or an invalid curation invariant.
    InvalidDocument(String),
    /// Unsupported curation envelope version.
    UnsupportedVersion {
        /// Version number present in the input.
        found: u32,
    },
    /// Reused globally reserved identity.
    DuplicateId(EntityId),
    /// Missing or contradictory referenced identity.
    MissingReference(String),
    /// Supplied context differs from its declared immutable pin.
    ContextMismatch {
        /// Pin whose actual body differed.
        context: EntityId,
        /// Declared canonical digest.
        expected: String,
        /// Digest of the supplied canonical body.
        actual: String,
    },
    /// Existing composition rejected conflicting source bodies.
    Composition(crate::PackError),
    /// Final normalized model validation failed.
    Dictionary(DictionaryError),
}

/// Borrowed lookup tables over one already validated, immutable context body.
struct ContextIndex<'a> {
    releases: HashMap<EntityId, &'a ReleaseRecord>,
    components: HashMap<EntityId, &'a ComponentRecord>,
    collections: HashMap<EntityId, &'a CollectionRecord>,
    entries: HashMap<EntityId, &'a EntryRecord>,
    assertions: HashMap<EntityId, &'a AssertionRecord>,
    captures: HashMap<CaptureRef, &'a RawCapture>,
    occurrences: HashMap<OccurrenceRef, &'a RawOccurrence>,
    attributions: HashMap<OccurrenceRef, &'a OccurrenceAttribution>,
    release_assertions: HashMap<EntityId, BTreeSet<EntityId>>,
}

impl<'a> ContextIndex<'a> {
    fn new(dictionary: &'a Dictionary) -> Self {
        let metadata = dictionary.metadata();
        let registry = dictionary.registry().as_document();
        let mut occurrences = HashMap::new();
        for capture in dictionary.captures() {
            for raw in &capture.occurrences {
                occurrences.insert(
                    OccurrenceRef {
                        capture: capture.identity.clone(),
                        locator: raw.locator.clone(),
                    },
                    raw,
                );
            }
        }
        Self {
            releases: metadata
                .releases
                .iter()
                .map(|r| (r.id.clone(), r))
                .collect(),
            components: metadata
                .components
                .iter()
                .map(|r| (r.id.clone(), r))
                .collect(),
            collections: metadata
                .collections
                .iter()
                .map(|r| (r.id.clone(), r))
                .collect(),
            entries: metadata.entries.iter().map(|r| (r.id.clone(), r)).collect(),
            assertions: metadata
                .assertions
                .iter()
                .map(|r| (r.id.clone(), r))
                .collect(),
            captures: dictionary
                .captures()
                .iter()
                .map(|capture| (capture.identity.clone(), capture))
                .collect(),
            occurrences,
            attributions: registry
                .occurrences
                .iter()
                .map(|row| (OccurrenceRef::from(&row.occurrence), row))
                .collect(),
            release_assertions: metadata
                .releases
                .iter()
                .map(|release| {
                    (
                        release.id.clone(),
                        release.assertions.iter().cloned().collect(),
                    )
                })
                .collect(),
        }
    }
}

impl fmt::Display for OwnedCurationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDocument(s) => write!(f, "invalid owned curation: {s}"),
            Self::UnsupportedVersion { found } => {
                write!(f, "unsupported owned curation version {found}")
            }
            Self::DuplicateId(id) => write!(f, "duplicate owned curation identity {id}"),
            Self::MissingReference(s) => write!(f, "invalid owned curation reference: {s}"),
            Self::ContextMismatch {
                context,
                expected,
                actual,
            } => write!(
                f,
                "context {context} mismatch: expected {expected}, got {actual}"
            ),
            Self::Composition(e) => write!(f, "cannot compose curation context: {e}"),
            Self::Dictionary(e) => write!(f, "invalid materialized dictionary: {e}"),
        }
    }
}

impl std::error::Error for OwnedCurationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Composition(e) => Some(e),
            Self::Dictionary(e) => Some(e),
            _ => None,
        }
    }
}

impl From<DictionaryError> for OwnedCurationError {
    fn from(e: DictionaryError) -> Self {
        Self::Dictionary(e)
    }
}
impl From<PackError> for OwnedCurationError {
    fn from(e: PackError) -> Self {
        Self::Composition(e)
    }
}

#[derive(Deserialize)]
struct VersionHeader {
    version: u32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StrictWire {
    version: u32,
    dictionary: Box<RawValue>,
    contexts: Vec<OwnedContextPin>,
    records: Vec<OwnedCurationRecord>,
    revisions: Vec<OwnedCurationRevision>,
}

impl OwnedCurationDocument {
    /// Validate complete typed data and all current/history invariants.
    pub fn from_data(data: OwnedCurationData) -> Result<Self, OwnedCurationError> {
        validate(&data)?;
        Ok(Self(data))
    }

    /// Parse strict whole-envelope JSON, preserving original nested Dictionary bytes.
    pub fn from_json_bytes(bytes: &[u8]) -> Result<Self, OwnedCurationError> {
        let text = std::str::from_utf8(bytes)
            .map_err(|e| OwnedCurationError::InvalidDocument(e.to_string()))?;
        crate::check_duplicate_keys(text)
            .map_err(|e| OwnedCurationError::InvalidDocument(e.to_string()))?;
        let header: VersionHeader = serde_json::from_str(text)
            .map_err(|e| OwnedCurationError::InvalidDocument(e.to_string()))?;
        if header.version != 1 {
            return Err(OwnedCurationError::UnsupportedVersion {
                found: header.version,
            });
        }
        let wire: StrictWire = serde_json::from_str(text)
            .map_err(|e| OwnedCurationError::InvalidDocument(e.to_string()))?;
        let dictionary = Dictionary::from_json_bytes(wire.dictionary.get().as_bytes())?;
        Self::from_data(OwnedCurationData {
            version: wire.version,
            dictionary,
            contexts: wire.contexts,
            records: wire.records,
            revisions: wire.revisions,
        })
    }

    /// Serialize the validated envelope as typed JSON bytes.
    pub fn to_json_bytes(&self) -> Result<Vec<u8>, OwnedCurationError> {
        serde_json::to_vec(self).map_err(|e| OwnedCurationError::InvalidDocument(e.to_string()))
    }

    /// Borrow the validated typed data.
    pub fn as_data(&self) -> &OwnedCurationData {
        &self.0
    }

    /// Consume the wrapper and return its validated typed data.
    pub fn into_data(self) -> OwnedCurationData {
        self.0
    }

    /// Resolve only supplied pinned bodies and return an explicit normalized view.
    pub fn materialize(
        &self,
        inputs: Vec<OwnedContextInput>,
    ) -> Result<OwnedMaterialization, OwnedCurationError> {
        let data = &self.0;
        let (source, head) = base_source_head(&data.dictionary)?;
        let selection = data
            .revisions
            .last()
            .ok_or_else(|| OwnedCurationError::InvalidDocument("missing curation head".into()))?;
        let pin_map: BTreeMap<_, _> = data.contexts.iter().map(|p| (p.id.clone(), p)).collect();
        let mut supplied = BTreeMap::<EntityId, OwnedContextInput>::new();
        for input in inputs {
            if !pin_map.contains_key(&input.id) {
                return Err(OwnedCurationError::MissingReference(format!(
                    "unknown context {}",
                    input.id
                )));
            }
            if supplied.insert(input.id.clone(), input).is_some() {
                return Err(OwnedCurationError::MissingReference(
                    "duplicate supplied context".into(),
                ));
            }
        }
        for pin in &data.contexts {
            if let Some(input) = supplied.get(&pin.id) {
                let model = &input.dictionary;
                let bytes = model.to_json_bytes()?;
                let actual = sha256(&bytes);
                let sources: Vec<_> = model
                    .registry()
                    .as_document()
                    .sources
                    .iter()
                    .map(|r| r.id.clone())
                    .collect();
                let mut sorted_sources = sources.clone();
                sorted_sources.sort();
                let mut sorted_releases: Vec<_> = model
                    .metadata()
                    .releases
                    .iter()
                    .map(|r| r.id.clone())
                    .collect();
                sorted_releases.sort();
                if actual != pin.model_sha256
                    || sorted_sources != pin.source_ids
                    || sorted_releases != pin.release_ids
                {
                    return Err(OwnedCurationError::ContextMismatch {
                        context: pin.id.clone(),
                        expected: pin.model_sha256.clone(),
                        actual,
                    });
                }
            }
        }
        let indexes: BTreeMap<_, _> = supplied
            .iter()
            .map(|(id, input)| (id.clone(), ContextIndex::new(&input.dictionary)))
            .collect();
        let active: BTreeSet<_> = selection.active.iter().cloned().collect();
        let owned_assertions: BTreeSet<_> = data
            .dictionary
            .metadata()
            .assertions
            .iter()
            .map(|assertion| assertion.id.clone())
            .collect();
        let record_map: BTreeMap<_, _> = data.records.iter().map(|r| (r.id.clone(), r)).collect();
        let mut outcomes = Vec::with_capacity(data.records.len());
        let mut resolved_senses = Vec::new();
        let mut resolved_links = Vec::new();
        for record in &data.records {
            let result = if !active.contains(&record.id) {
                resolution(record, OwnedCurationStatus::Inactive, None, None, false)
            } else {
                match &record.kind {
                    OwnedCurationKind::Sense { members } => {
                        match resolve_members(members, &owned_assertions, &indexes, &pin_map) {
                            Ok(ids) => {
                                resolved_senses.push((record, ids));
                                resolution(record, OwnedCurationStatus::Resolved, None, None, true)
                            }
                            Err((reason, member)) => {
                                resolution(record, status_for(reason), Some(reason), member, false)
                            }
                        }
                    }
                    OwnedCurationKind::Link { .. } => resolution(
                        record,
                        OwnedCurationStatus::Unresolved,
                        Some(OwnedCurationReason::MissingContext),
                        None,
                        false,
                    ),
                }
            };
            outcomes.push(result);
        }
        let available_senses: BTreeSet<_> = resolved_senses
            .iter()
            .map(|(record, _)| record.id.clone())
            .collect();
        for (index, record) in data.records.iter().enumerate() {
            if !active.contains(&record.id) {
                continue;
            }
            let OwnedCurationKind::Link {
                origin,
                direction,
                target,
            } = &record.kind
            else {
                continue;
            };
            let depends_on_unavailable_sense = match origin {
                LocalRef::EditorialSense(id) => record_map.get(id).is_some_and(|record| {
                    matches!(record.kind, OwnedCurationKind::Sense { .. })
                        && !available_senses.contains(id)
                }),
                _ => false,
            };
            if depends_on_unavailable_sense {
                outcomes[index] = resolution(
                    record,
                    OwnedCurationStatus::Unresolved,
                    Some(OwnedCurationReason::DependencyUnavailable),
                    None,
                    false,
                );
                continue;
            }
            outcomes[index] = match resolve_target(target, &indexes, &pin_map) {
                Ok(()) => {
                    match origin_available(origin, &data.dictionary, &record_map, &available_senses)
                    {
                        Ok(()) => {
                            resolved_links.push((record, origin, direction, target));
                            resolution(record, OwnedCurationStatus::Resolved, None, None, true)
                        }
                        Err(reason) => resolution(
                            record,
                            OwnedCurationStatus::Unresolved,
                            Some(reason),
                            None,
                            false,
                        ),
                    }
                }
                Err(reason) => resolution(record, status_for(reason), Some(reason), None, false),
            };
        }
        let resolutions = outcomes;
        drop(indexes);
        // Contexts are consumed in declaration order; the owned base is the
        // sole clone and is appended after all supplied bodies.
        let mut models = Vec::with_capacity(data.contexts.len() + 1);
        for pin in &data.contexts {
            if let Some(input) = supplied.remove(&pin.id) {
                models.push(input.dictionary);
            }
        }
        models.push(data.dictionary.clone());
        let view = crate::compose_dictionaries(models)?;
        let mut output = view.into_document();
        for (record, ids) in resolved_senses {
            output.metadata.editorial_senses.push(EditorialSenseRecord {
                id: record.id.clone(),
                assertions: ids,
                editor: record.editor.clone(),
                revision: record.revision.clone(),
                rationale: record.rationale.clone(),
            });
            output
                .metadata
                .extensions
                .push(extension(record, &data.contexts)?);
        }
        for (record, origin, direction, target) in resolved_links {
            output.metadata.relations.push(link_row(
                record,
                origin.clone(),
                *direction,
                target.clone(),
            ));
            output
                .metadata
                .extensions
                .push(extension(record, &data.contexts)?);
        }
        let dictionary =
            Dictionary::from_document(output).map_err(OwnedCurationError::Dictionary)?;
        Ok(OwnedMaterialization {
            version: 1,
            source,
            revision: head,
            dictionary,
            resolutions,
        })
    }
}

impl OwnedContextPin {
    /// Derive a canonical immutable pin from a validated dictionary.
    pub fn from_dictionary(id: EntityId, context: &Dictionary) -> Result<Self, OwnedCurationError> {
        let bytes = context.to_json_bytes()?;
        let mut source_ids: Vec<_> = context
            .registry()
            .as_document()
            .sources
            .iter()
            .map(|r| r.id.clone())
            .collect();
        source_ids.sort();
        let mut release_ids: Vec<_> = context
            .metadata()
            .releases
            .iter()
            .map(|r| r.id.clone())
            .collect();
        release_ids.sort();
        Ok(Self {
            id,
            model_sha256: sha256(&bytes),
            source_ids,
            release_ids,
        })
    }
}

fn validate(data: &OwnedCurationData) -> Result<(), OwnedCurationError> {
    if data.version != 1 {
        return Err(OwnedCurationError::UnsupportedVersion {
            found: data.version,
        });
    }
    validate_owned_base(&data.dictionary)?;
    let (source, _) = base_source_head(&data.dictionary)?;
    let ids = all_ids(&data.dictionary);
    let mut seen = ids;
    for pin in &data.contexts {
        if !seen.insert(pin.id.clone()) {
            return Err(OwnedCurationError::DuplicateId(pin.id.clone()));
        }
        if !valid_hash(&pin.model_sha256)
            || !strict_sorted_unique(&pin.source_ids)
            || !strict_sorted_unique(&pin.release_ids)
            || pin.source_ids.contains(&source)
        {
            return Err(OwnedCurationError::InvalidDocument(format!(
                "invalid context pin {}",
                pin.id
            )));
        }
    }
    let pins: BTreeMap<_, _> = data.contexts.iter().map(|p| (p.id.clone(), p)).collect();
    let mut record_index = BTreeMap::new();
    for (index, record) in data.records.iter().enumerate() {
        if !seen.insert(record.id.clone()) {
            return Err(OwnedCurationError::DuplicateId(record.id.clone()));
        }
        record_index.insert(record.id.clone(), (index, &record.kind));
    }
    let revision_indices: BTreeMap<_, _> = data
        .dictionary
        .metadata()
        .owned_revisions
        .iter()
        .enumerate()
        .map(|(index, revision)| (revision.id.clone(), index))
        .collect();
    for record in &data.records {
        let revision = revision_indices
            .get(&record.revision)
            .copied()
            .ok_or_else(|| OwnedCurationError::MissingReference(record.revision.to_string()))?;
        let rev = &data.dictionary.metadata().owned_revisions[revision];
        if rev.source != source || rev.editor != record.editor {
            return Err(OwnedCurationError::MissingReference(format!(
                "record {} editor/revision",
                record.id
            )));
        }
        match &record.kind {
            OwnedCurationKind::Link { origin, target, .. } => {
                validate_origin(origin, &data.dictionary, &record_index)?;
                validate_target_shape(target, &pins)?;
            }
            OwnedCurationKind::Sense { members } => {
                if members.len() < 2 {
                    return Err(OwnedCurationError::InvalidDocument(format!(
                        "sense {} needs at least two members",
                        record.id
                    )));
                }
                let mut member_ids = BTreeSet::new();
                for member in members {
                    match member {
                        OwnedSenseMember::OwnedAssertion(id) => {
                            if !member_ids.insert(id.clone()) {
                                return Err(OwnedCurationError::InvalidDocument(
                                    "duplicate sense member".into(),
                                ));
                            }
                        }
                        OwnedSenseMember::External(target) => {
                            validate_target_shape(target, &pins)?;
                            if !member_ids.insert(target.assertion.clone()) {
                                return Err(OwnedCurationError::InvalidDocument(
                                    "duplicate sense member".into(),
                                ));
                            }
                        }
                    }
                }
            }
        }
    }
    if data.revisions.len() != data.dictionary.metadata().owned_revisions.len() {
        return Err(OwnedCurationError::InvalidDocument(
            "curation/base revision lengths differ".into(),
        ));
    }
    let mut previous = Vec::<EntityId>::new();
    for (i, (revision, base)) in data
        .revisions
        .iter()
        .zip(&data.dictionary.metadata().owned_revisions)
        .enumerate()
    {
        if revision.revision != base.id {
            return Err(OwnedCurationError::InvalidDocument(
                "curation revision order differs from base".into(),
            ));
        }
        if i == 0 && (!revision.active.is_empty() || !revision.tombstones.is_empty()) {
            return Err(OwnedCurationError::InvalidDocument(
                "curation genesis selection must be empty".into(),
            ));
        }
        unique_ids(&revision.active)?;
        unique_ids(&revision.tombstones)?;
        let active: BTreeSet<_> = revision.active.iter().collect();
        let expected_removed: Vec<_> = previous
            .iter()
            .filter(|id| !active.contains(id))
            .cloned()
            .collect();
        if expected_removed != revision.tombstones {
            return Err(OwnedCurationError::InvalidDocument(
                "curation tombstones differ from prior active removals".into(),
            ));
        }
        for id in revision.active.iter().chain(&revision.tombstones) {
            let Some((created, _)) = record_index.get(id) else {
                return Err(OwnedCurationError::MissingReference(id.to_string()));
            };
            let record = &data.records[*created];
            let creation = revision_indices
                .get(&record.revision)
                .copied()
                .unwrap_or(usize::MAX);
            if creation > i {
                return Err(OwnedCurationError::InvalidDocument(format!(
                    "record {id} selected before creation"
                )));
            }
        }
        let created_here: BTreeSet<_> = data
            .records
            .iter()
            .filter(|r| r.revision == base.id)
            .map(|r| r.id.clone())
            .collect();
        if !created_here.iter().all(|id| revision.active.contains(id)) {
            return Err(OwnedCurationError::InvalidDocument(
                "new definitions must be active at creation".into(),
            ));
        }
        previous = revision.active.clone();
    }
    Ok(())
}

fn validate_owned_base(dictionary: &Dictionary) -> Result<(), OwnedCurationError> {
    let doc = dictionary.document();
    if doc.version != 1 || doc.registry.as_document().version != 1 || doc.legacy_archive.is_some() {
        return Err(OwnedCurationError::InvalidDocument(
            "base model/registry version or legacy archive".into(),
        ));
    }
    let registry = doc.registry.as_document();
    if registry.sources.len() != 1 || registry.sources[0].origin != SourceOrigin::OwnedDraft {
        return Err(OwnedCurationError::InvalidDocument(
            "base must have one OwnedDraft source".into(),
        ));
    }
    let source = &registry.sources[0].id;
    let revisions = &doc.metadata.owned_revisions;
    if revisions.is_empty()
        || doc.metadata.owned_heads.len() != 1
        || doc.metadata.owned_heads[0].source != *source
        || doc.metadata.owned_heads[0].revision
            != revisions
                .last()
                .map(|r| r.id.clone())
                .unwrap_or_else(|| EntityId::new("invalid").expect("static id"))
        || doc.metadata.catalogue_revision != (revisions.len() - 1) as u64
        || doc.metadata.catalogue_revision > i64::MAX as u64
    {
        return Err(OwnedCurationError::InvalidDocument(
            "base owned head/revision sequence".into(),
        ));
    }
    let mut previous: Option<&EntityId> = None;
    for revision in revisions {
        if revision.source != *source || revision.parent.as_ref() != previous {
            return Err(OwnedCurationError::InvalidDocument(
                "base revision chain".into(),
            ));
        }
        previous = Some(&revision.id);
    }
    let first = &revisions[0];
    if !first.tombstones.is_empty() {
        return Err(OwnedCurationError::InvalidDocument(
            "base genesis tombstones".into(),
        ));
    }
    for pair in revisions.windows(2) {
        let old: BTreeSet<_> = pair[0].assertions.iter().collect();
        let active: BTreeSet<_> = pair[1].assertions.iter().collect();
        let removed: Vec<_> = pair[0]
            .assertions
            .iter()
            .filter(|id| !active.contains(id))
            .cloned()
            .collect();
        if removed != pair[1].tombstones {
            return Err(OwnedCurationError::InvalidDocument(
                "base tombstones differ from prior active removals".into(),
            ));
        }
        let _ = old;
    }
    Ok(())
}

fn base_source_head(dictionary: &Dictionary) -> Result<(SourceId, EntityId), OwnedCurationError> {
    let source = dictionary
        .registry()
        .as_document()
        .sources
        .first()
        .ok_or_else(|| OwnedCurationError::InvalidDocument("missing source".into()))?
        .id
        .clone();
    let head = dictionary
        .metadata()
        .owned_heads
        .first()
        .ok_or_else(|| OwnedCurationError::InvalidDocument("missing owned head".into()))?
        .revision
        .clone();
    Ok((source, head))
}

fn all_ids(dictionary: &Dictionary) -> BTreeSet<EntityId> {
    let m = dictionary.metadata();
    let mut ids = BTreeSet::new();
    macro_rules! add { ($($field:ident),*) => { $(ids.extend(m.$field.iter().map(|r| r.id.clone()));)* }; }
    add!(
        collections,
        components,
        releases,
        entries,
        assertions,
        contexts,
        frames,
        forms,
        etymologies,
        relations,
        owned_revisions,
        editorial_senses,
        computations,
        annotations
    );
    ids.extend(m.owned_heads.iter().map(|r| r.revision.clone()));
    ids
}

fn validate_target_shape(
    target: &OwnedPinnedTarget,
    pins: &BTreeMap<EntityId, &OwnedContextPin>,
) -> Result<(), OwnedCurationError> {
    let pin = pins
        .get(&target.context)
        .ok_or_else(|| OwnedCurationError::MissingReference(target.context.to_string()))?;
    if !pin.source_ids.contains(&target.target.source)
        || !pin.release_ids.contains(&target.target.release)
        || target.target.occurrence.as_ref().is_none_or(|o| {
            o.locator.is_empty() || o.capture.namespace.is_empty() || !valid_hash(&o.capture.sha256)
        })
    {
        return Err(OwnedCurationError::MissingReference(format!(
            "target {}",
            target.assertion
        )));
    }
    Ok(())
}

fn validate_origin(
    origin: &LocalRef,
    dictionary: &Dictionary,
    records: &BTreeMap<EntityId, (usize, &OwnedCurationKind)>,
) -> Result<(), OwnedCurationError> {
    let m = dictionary.metadata();
    let ok = match origin {
        LocalRef::Entry(id) => m.entries.iter().any(|x| &x.id == id),
        LocalRef::Assertion(id) => m.assertions.iter().any(|x| &x.id == id),
        LocalRef::Form(id) => m.forms.iter().any(|x| &x.id == id),
        LocalRef::EditorialSense(id) => {
            m.editorial_senses.iter().any(|x| &x.id == id)
                || records
                    .get(id)
                    .is_some_and(|(_, kind)| matches!(kind, OwnedCurationKind::Sense { .. }))
        }
    };
    if ok {
        Ok(())
    } else {
        Err(OwnedCurationError::MissingReference("link origin".into()))
    }
}

fn resolve_members(
    members: &[OwnedSenseMember],
    owned_assertions: &BTreeSet<EntityId>,
    indexes: &BTreeMap<EntityId, ContextIndex<'_>>,
    pins: &BTreeMap<EntityId, &OwnedContextPin>,
) -> Result<Vec<EntityId>, (OwnedCurationReason, Option<u64>)> {
    let mut ids = Vec::new();
    for (i, member) in members.iter().enumerate() {
        match member {
            OwnedSenseMember::OwnedAssertion(id) if owned_assertions.contains(id) => {
                ids.push(id.clone())
            }
            OwnedSenseMember::OwnedAssertion(_) => {
                return Err((OwnedCurationReason::MissingEntity, Some(i as u64)));
            }
            OwnedSenseMember::External(target) => match resolve_target(target, indexes, pins) {
                Ok(()) => ids.push(target.assertion.clone()),
                Err(reason) => return Err((reason, Some(i as u64))),
            },
        }
    }
    Ok(ids)
}

fn resolve_target(
    target: &OwnedPinnedTarget,
    indexes: &BTreeMap<EntityId, ContextIndex<'_>>,
    pins: &BTreeMap<EntityId, &OwnedContextPin>,
) -> Result<(), OwnedCurationReason> {
    let Some(index) = indexes.get(&target.context) else {
        return Err(OwnedCurationReason::MissingContext);
    };
    let Some(pin) = pins.get(&target.context) else {
        return Err(OwnedCurationReason::MissingContext);
    };
    if !pin.source_ids.contains(&target.target.source)
        || !pin.release_ids.contains(&target.target.release)
    {
        return Err(OwnedCurationReason::WrongSource);
    }
    let Some(release) = index.releases.get(&target.target.release) else {
        return Err(OwnedCurationReason::MissingRelease);
    };
    let Some(component) = index.components.get(&release.component) else {
        return Err(OwnedCurationReason::MissingRelease);
    };
    let Some(collection) = index.collections.get(&component.collection) else {
        return Err(OwnedCurationReason::MissingRelease);
    };
    if collection.source != target.target.source {
        return Err(OwnedCurationReason::WrongSource);
    }
    let Some(entry) = index.entries.get(&target.entry) else {
        return Err(OwnedCurationReason::MissingEntity);
    };
    if entry.source != target.target.source {
        return Err(OwnedCurationReason::WrongSource);
    }
    if !index
        .release_assertions
        .get(&target.target.release)
        .is_some_and(|assertions| assertions.contains(&target.assertion))
    {
        return Err(OwnedCurationReason::AssertionNotInRelease);
    }
    let Some(assertion) = index.assertions.get(&target.assertion) else {
        return Err(OwnedCurationReason::MissingEntity);
    };
    if assertion.entry != target.entry {
        return Err(OwnedCurationReason::OccurrenceMismatch);
    }
    let Some(occurrence) = target.target.occurrence.as_ref() else {
        return Err(OwnedCurationReason::OccurrenceMismatch);
    };
    if assertion.occurrence != *occurrence {
        return Err(OwnedCurationReason::OccurrenceMismatch);
    }
    let Some(capture) = index.captures.get(&occurrence.capture) else {
        return Err(OwnedCurationReason::PayloadUnavailable);
    };
    let Some(_raw) = index.occurrences.get(occurrence) else {
        return Err(OwnedCurationReason::PayloadUnavailable);
    };
    let attribution = index.attributions.get(occurrence);
    for (requested, actual) in [
        (
            target.target.native_entry_id.as_ref(),
            attribution.and_then(|row| row.native_entry_id.as_ref()),
        ),
        (
            target.target.native_assertion_id.as_ref(),
            attribution.and_then(|row| row.native_assertion_id.as_ref()),
        ),
    ] {
        if let Some(requested) = requested {
            match actual {
                None => return Err(OwnedCurationReason::NativeIdentityUnknown),
                Some(actual) if actual != requested => {
                    return Err(OwnedCurationReason::NativeIdentityMismatch);
                }
                Some(_) => {}
            }
        }
    }
    let _ = capture;
    Ok(())
}

fn origin_available(
    origin: &LocalRef,
    base: &Dictionary,
    records: &BTreeMap<EntityId, &OwnedCurationRecord>,
    resolved_senses: &BTreeSet<EntityId>,
) -> Result<(), OwnedCurationReason> {
    match origin {
        LocalRef::Entry(id) if base.metadata().entries.iter().any(|r| &r.id == id) => Ok(()),
        LocalRef::Assertion(id) if base.metadata().assertions.iter().any(|r| &r.id == id) => Ok(()),
        LocalRef::Form(id) if base.metadata().forms.iter().any(|r| &r.id == id) => Ok(()),
        LocalRef::EditorialSense(id)
            if base.metadata().editorial_senses.iter().any(|r| &r.id == id) =>
        {
            Ok(())
        }
        LocalRef::EditorialSense(id) => match records.get(id) {
            Some(OwnedCurationRecord {
                kind: OwnedCurationKind::Sense { .. },
                ..
            }) if resolved_senses.contains(id) => Ok(()),
            Some(OwnedCurationRecord {
                kind: OwnedCurationKind::Sense { .. },
                ..
            }) => Err(OwnedCurationReason::DependencyUnavailable),
            _ => Err(OwnedCurationReason::MissingOrigin),
        },
        _ => Err(OwnedCurationReason::MissingOrigin),
    }
}

fn link_row(
    record: &OwnedCurationRecord,
    origin: LocalRef,
    direction: Direction,
    target: OwnedPinnedTarget,
) -> RelationRecord {
    let scope = match origin {
        LocalRef::Entry(_) => RelationScope::Entry,
        LocalRef::Assertion(_) => RelationScope::Assertion,
        LocalRef::Form(_) => RelationScope::Form,
        LocalRef::EditorialSense(_) => RelationScope::Editorial,
    };
    RelationRecord {
        id: record.id.clone(),
        origin,
        kind: RelationKind::EditorialLink,
        direction,
        scope,
        target: Target::Catalog(target.target),
        evidence: None,
        editorial_revision: Some(record.revision.clone()),
    }
}

fn extension(
    record: &OwnedCurationRecord,
    pins: &[OwnedContextPin],
) -> Result<ExtensionRecord, OwnedCurationError> {
    let referenced: BTreeSet<_> = match &record.kind {
        OwnedCurationKind::Link { target, .. } => [target.context.clone()].into_iter().collect(),
        OwnedCurationKind::Sense { members } => members
            .iter()
            .filter_map(|m| match m {
                OwnedSenseMember::External(t) => Some(t.context.clone()),
                _ => None,
            })
            .collect(),
    };
    let contexts: Vec<_> = pins
        .iter()
        .filter(|p| referenced.contains(&p.id))
        .cloned()
        .collect();
    #[derive(Serialize)]
    struct ExtensionPayload<'a> {
        version: u32,
        record: &'a OwnedCurationRecord,
        contexts: Vec<OwnedContextPin>,
    }
    let payload = ExtensionPayload {
        version: 1,
        record,
        contexts,
    };
    let bytes = serde_json::to_vec(&payload)
        .map_err(|e| OwnedCurationError::InvalidDocument(e.to_string()))?;
    let value = RawJson::new(
        String::from_utf8(bytes).map_err(|e| OwnedCurationError::InvalidDocument(e.to_string()))?,
    )
    .map_err(OwnedCurationError::Dictionary)?;
    Ok(ExtensionRecord {
        owner: record.id.clone(),
        name: "vlajmaji.owned-curation.v1".into(),
        value,
        evidence: None,
    })
}

fn resolution(
    record: &OwnedCurationRecord,
    status: OwnedCurationStatus,
    reason: Option<OwnedCurationReason>,
    member: Option<u64>,
    projected: bool,
) -> OwnedCurationResolution {
    OwnedCurationResolution {
        record: record.id.clone(),
        status,
        reason,
        member,
        projected,
    }
}
fn status_for(reason: OwnedCurationReason) -> OwnedCurationStatus {
    if reason == OwnedCurationReason::MissingContext {
        OwnedCurationStatus::Unavailable
    } else {
        OwnedCurationStatus::Unresolved
    }
}
fn unique_ids(ids: &[EntityId]) -> Result<(), OwnedCurationError> {
    let mut seen = BTreeSet::new();
    if ids.iter().all(|id| seen.insert(id.clone())) {
        Ok(())
    } else {
        Err(OwnedCurationError::InvalidDocument(
            "duplicate selection ID".into(),
        ))
    }
}
fn strict_sorted_unique<T: Ord>(values: &[T]) -> bool {
    values.windows(2).all(|w| w[0] < w[1])
}
fn valid_hash(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn sha256(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(64);
    for b in Sha256::digest(bytes) {
        use fmt::Write;
        let _ = write!(s, "{b:02x}");
    }
    s
}
