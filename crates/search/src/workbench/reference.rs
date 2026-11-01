//! Typed references across an explicit mixed workbench and its retained catalogue.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use vlajmaji_lexicon::{
    AssertionRecord, EditorialSenseRecord, EntryRecord, FormRecord, PackId, PackPayloadKind,
    RelationTarget, SourceRecord,
};
use vlajmaji_store::{CollectionCatalogue, CollectionInstallationState, CollectionReleaseSummary};

use super::{ConfiguredWorkbenchSearchEngine, WorkbenchQuerySelection, WorkbenchSearchEngine};
use crate::{
    CollectionReferenceMatch, DictionaryCompactReferenceResolution, ReferenceResolution,
    ReferenceStatus,
};

fn is_nonmaterialized_target(target: &vlajmaji_lexicon::RelationTarget) -> bool {
    matches!(target, vlajmaji_lexicon::RelationTarget::Unknown)
        || matches!(target, vlajmaji_lexicon::RelationTarget::Unresolved(value) if !value.is_empty())
}

fn owned_release_summary(
    owned_source: Option<&vlajmaji_lexicon::SourceId>,
    target: &RelationTarget,
    summary: Option<CollectionReleaseSummary>,
) -> Option<CollectionReleaseSummary> {
    summary.filter(|release| match (owned_source, target) {
        (Some(source), RelationTarget::Catalog(catalog)) => {
            source == &catalog.source && release.id == catalog.release.as_str()
        }
        _ => false,
    })
}

fn empty_observation() -> CatalogueObservation {
    CatalogueObservation {
        matches: Vec::new(),
        registrations: Vec::new(),
        requested_release: None,
        witnesses: Vec::new(),
        top_reason: None,
    }
}

fn status_without_payload(
    target: &RelationTarget,
    matches: &[CollectionReferenceMatch],
    strict_native: bool,
) -> ReferenceStatus {
    match target {
        RelationTarget::Unknown => ReferenceStatus::Unknown,
        RelationTarget::Unresolved(_) => ReferenceStatus::Unresolved,
        RelationTarget::Local(_) => {
            if matches.is_empty() {
                ReferenceStatus::NoMatch
            } else {
                ReferenceStatus::CatalogueOnly
            }
        }
        RelationTarget::Catalog(catalog) if matches.is_empty() => ReferenceStatus::NoMatch,
        RelationTarget::Catalog(catalog)
            if !strict_native
                && ((catalog.native_entry_id.is_some()
                    && matches.iter().any(|row| row.native_entry_id.is_none()))
                    || (catalog.native_assertion_id.is_some()
                        && matches.iter().any(|row| row.native_assertion_id.is_none()))) =>
        {
            ReferenceStatus::Unverified
        }
        RelationTarget::Catalog(_) => ReferenceStatus::CatalogueOnly,
    }
}

/// One retained catalogue registration relevant to a typed reference.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkbenchReferenceRegistration {
    /// Stable catalogue pack.
    pub pack_id: PackId,
    /// Exact registered release digest.
    pub release_sha256: String,
    /// Declared registered payload scope.
    pub payload_kind: PackPayloadKind,
    /// Whether this pack was selected by the workbench.
    pub selected: bool,
    /// Whether this selected pack was deliberately disabled.
    pub disabled: bool,
    /// Logical installation state captured by the metadata roster.
    pub installation_state: CollectionInstallationState,
    /// Release installed for this pack, when any.
    #[serde(deserialize_with = "super::deserialize_required_option")]
    pub installed_release_sha256: Option<String>,
    /// Whether this exact registration is an admitted selected, enabled member.
    pub admitted: bool,
    /// Local explanation when this registration is not admitted.
    #[serde(deserialize_with = "super::deserialize_required_option")]
    pub operational_reason: Option<String>,
    /// Source rows relevant to the requested entity.
    pub sources: Vec<SourceRecord>,
    /// Matching owned release summary, when the request is a Catalog target.
    #[serde(deserialize_with = "super::deserialize_required_option")]
    pub requested_release: Option<CollectionReleaseSummary>,
}

/// Full-format reference outcome and its retained catalogue observations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkbenchReferenceFullResult {
    /// Exact typed target supplied by the caller.
    pub target: RelationTarget,
    /// Existing Full reference status vocabulary.
    pub status: ReferenceStatus,
    /// First matching registration's local operational reason.
    #[serde(deserialize_with = "super::deserialize_required_option")]
    pub operational_reason: Option<String>,
    /// Existing complete Full resolution when an admitted body is available.
    #[serde(deserialize_with = "super::deserialize_required_option")]
    pub payload: Option<Box<ReferenceResolution>>,
    /// Raw retained descriptor matches in their established order.
    pub catalogue_matches: Vec<CollectionReferenceMatch>,
    /// First known target release summary, when present.
    #[serde(deserialize_with = "super::deserialize_required_option")]
    pub requested_release: Option<CollectionReleaseSummary>,
    /// Per-registration logical observations.
    pub registrations: Vec<WorkbenchReferenceRegistration>,
}

/// Compact-format reference outcome and its retained catalogue observations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkbenchReferenceCompactResult {
    /// Exact typed target supplied by the caller.
    pub target: RelationTarget,
    /// Existing Compact reference status vocabulary.
    pub status: ReferenceStatus,
    /// First matching registration's local operational reason.
    #[serde(deserialize_with = "super::deserialize_required_option")]
    pub operational_reason: Option<String>,
    /// Existing Compact resolution when an admitted body is available.
    #[serde(deserialize_with = "super::deserialize_required_option")]
    pub payload: Option<Box<DictionaryCompactReferenceResolution>>,
    /// Raw retained descriptor matches in their established order.
    pub catalogue_matches: Vec<CollectionReferenceMatch>,
    /// First known target release summary, when present.
    #[serde(deserialize_with = "super::deserialize_required_option")]
    pub requested_release: Option<CollectionReleaseSummary>,
    /// Per-registration logical observations.
    pub registrations: Vec<WorkbenchReferenceRegistration>,
}

/// Direct Full reference response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkbenchReferenceFullResponse {
    /// Response version.
    pub version: u32,
    /// Stable result kind.
    pub kind: String,
    /// Full-format marker.
    pub format: String,
    /// Outer workbench binding identity.
    pub binding_sha256: String,
    /// Small cached query selection summary.
    pub selection: WorkbenchQuerySelection,
    /// Full reference outcome.
    pub result: WorkbenchReferenceFullResult,
}

/// Direct Compact reference response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkbenchReferenceCompactResponse {
    /// Response version.
    pub version: u32,
    /// Stable result kind.
    pub kind: String,
    /// Compact-format marker.
    pub format: String,
    /// Outer workbench binding identity.
    pub binding_sha256: String,
    /// Small cached query selection summary.
    pub selection: WorkbenchQuerySelection,
    /// Compact reference outcome.
    pub result: WorkbenchReferenceCompactResult,
}

/// Configured Full reference response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfiguredWorkbenchReferenceFullResponse {
    /// Response version.
    pub version: u32,
    /// Stable result kind.
    pub kind: String,
    /// Full-format marker.
    pub format: String,
    /// Effective configured workbench binding identity.
    pub binding_sha256: String,
    /// Captured configured profile identity.
    pub configuration: vlajmaji_store::WorkbenchProfileIdentity,
    /// Small cached query selection summary.
    pub selection: WorkbenchQuerySelection,
    /// Full reference outcome.
    pub result: WorkbenchReferenceFullResult,
}

/// Configured Compact reference response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfiguredWorkbenchReferenceCompactResponse {
    /// Response version.
    pub version: u32,
    /// Stable result kind.
    pub kind: String,
    /// Compact-format marker.
    pub format: String,
    /// Effective configured workbench binding identity.
    pub binding_sha256: String,
    /// Captured configured profile identity.
    pub configuration: vlajmaji_store::WorkbenchProfileIdentity,
    /// Small cached query selection summary.
    pub selection: WorkbenchQuerySelection,
    /// Compact reference outcome.
    pub result: WorkbenchReferenceCompactResult,
}

/// Borrowed source-model role identity and exact typed body digest.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct TypedRoleWitness {
    pub(crate) descriptor: vlajmaji_store::CollectionEntityReference,
    pub(crate) body_sha256: String,
}

struct CatalogueObservation {
    matches: Vec<CollectionReferenceMatch>,
    registrations: Vec<WorkbenchReferenceRegistration>,
    requested_release: Option<CollectionReleaseSummary>,
    witnesses: Vec<TypedRoleWitness>,
    top_reason: Option<String>,
}

pub(crate) fn typed_role_witness<T: Serialize>(
    descriptor: vlajmaji_store::CollectionEntityReference,
    row: &T,
) -> Result<TypedRoleWitness, crate::DictionaryQueryError> {
    let bytes = serde_json::to_vec(row).map_err(|error| {
        crate::DictionaryQueryError::Dictionary(vlajmaji_lexicon::DictionaryError::InvalidJson(
            error.to_string(),
        ))
    })?;
    Ok(TypedRoleWitness {
        descriptor,
        body_sha256: format!("{:x}", Sha256::digest(bytes)),
    })
}

pub(crate) fn entry_witness(
    row: &EntryRecord,
) -> Result<TypedRoleWitness, crate::DictionaryQueryError> {
    typed_role_witness(
        vlajmaji_store::CollectionEntityReference {
            role: "entry".into(),
            id: row.id.to_string(),
            entry_ids: Vec::new(),
            assertion_ids: Vec::new(),
            source_ids: vec![row.source.to_string()],
            occurrence: None,
            evidence: None,
            spelling: Some(row.spelling.clone()),
        },
        row,
    )
}

pub(crate) fn assertion_witness(
    row: &AssertionRecord,
    entry: &EntryRecord,
) -> Result<TypedRoleWitness, crate::DictionaryQueryError> {
    typed_role_witness(
        vlajmaji_store::CollectionEntityReference {
            role: "assertion".into(),
            id: row.id.to_string(),
            entry_ids: vec![row.entry.to_string()],
            assertion_ids: Vec::new(),
            source_ids: vec![entry.source.to_string()],
            occurrence: Some(row.occurrence.clone()),
            evidence: None,
            spelling: None,
        },
        row,
    )
}

pub(crate) fn form_witness(
    row: &FormRecord,
    entry: &EntryRecord,
) -> Result<TypedRoleWitness, crate::DictionaryQueryError> {
    typed_role_witness(
        vlajmaji_store::CollectionEntityReference {
            role: "form".into(),
            id: row.id.to_string(),
            entry_ids: vec![row.entry.to_string()],
            assertion_ids: Vec::new(),
            source_ids: vec![entry.source.to_string()],
            occurrence: row.evidence.as_ref().map(|value| value.occurrence.clone()),
            evidence: row.evidence.clone(),
            spelling: None,
        },
        row,
    )
}

pub(crate) fn sense_witness(
    row: &EditorialSenseRecord,
    linked: &[(&AssertionRecord, &EntryRecord)],
) -> Result<TypedRoleWitness, crate::DictionaryQueryError> {
    let mut entry_ids = std::collections::BTreeSet::new();
    let mut source_ids = std::collections::BTreeSet::new();
    if linked.len() != row.assertions.len() {
        return Err(crate::DictionaryQueryError::Integrity {
            detail: format!("editorial sense {} has incomplete linked rows", row.id),
        });
    }
    for (index, (assertion, entry)) in linked.iter().enumerate() {
        if row.assertions.get(index) != Some(&assertion.id) {
            return Err(crate::DictionaryQueryError::Integrity {
                detail: format!(
                    "editorial sense {} has mismatched linked assertion order",
                    row.id
                ),
            });
        }
        entry_ids.insert(entry.id.to_string());
        source_ids.insert(entry.source.to_string());
    }
    typed_role_witness(
        vlajmaji_store::CollectionEntityReference {
            role: "editorial_sense".into(),
            id: row.id.to_string(),
            entry_ids: entry_ids.into_iter().collect(),
            assertion_ids: row.assertions.iter().map(ToString::to_string).collect(),
            source_ids: source_ids.into_iter().collect(),
            occurrence: None,
            evidence: None,
            spelling: None,
        },
        row,
    )
}

fn catalogue_observation(
    view: &vlajmaji_store::WorkbenchView,
    target: &RelationTarget,
    strict_native: bool,
    owned_release: Option<&CollectionReleaseSummary>,
) -> Result<CatalogueObservation, super::WorkbenchQueryError> {
    let Some(root) = view.bound_catalogue_root()? else {
        return Ok(CatalogueObservation {
            matches: Vec::new(),
            registrations: Vec::new(),
            requested_release: owned_release.cloned(),
            witnesses: Vec::new(),
            top_reason: None,
        });
    };
    let catalogue = CollectionCatalogue::open(&root)?;
    let captured_revision = view
        .binding()
        .catalogue()
        .map(|binding| binding.local_revision)
        .ok_or_else(|| {
            super::WorkbenchQueryError::Query(Box::new(crate::DictionaryQueryError::Integrity {
                detail: "bound catalogue path has no captured catalogue identity".into(),
            }))
        })?;
    let local_revision = catalogue.local_revision()?;
    let roster = catalogue.metadata_roster()?;
    if local_revision != captured_revision || roster.local_revision != captured_revision {
        return Err(super::WorkbenchQueryError::Catalogue(Box::new(
            vlajmaji_store::CollectionError::InvalidRequest(
                "stale_snapshot: catalogue revision changed".into(),
            ),
        )));
    }

    let (projection_target, requested_catalogue) = match target {
        RelationTarget::Catalog(requested) if !strict_native => {
            let mut candidate = requested.clone();
            candidate.native_entry_id = None;
            candidate.native_assertion_id = None;
            (RelationTarget::Catalog(candidate), Some(requested))
        }
        _ => (
            target.clone(),
            match target {
                RelationTarget::Catalog(requested) => Some(requested),
                _ => None,
            },
        ),
    };
    let selected = &view.binding().selection().packs;
    let disabled = &view.binding().selection().disabled;
    let generation_members = roster
        .generation
        .as_ref()
        .map(|generation| generation.members.as_slice())
        .unwrap_or_default();
    let pack_order = [
        PackId::LensiskuJbotci,
        PackId::Llg,
        PackId::LaLojban,
        PackId::Sutysisku,
    ];
    let mut matches = Vec::new();
    let mut registrations = Vec::new();
    let mut requested_release = None;
    let mut retained_entities = std::collections::BTreeMap::new();
    let mut ambiguous = None;
    let mut matching_release_owned = false;
    let mut release_seen = false;
    let mut requested_occurrence_retained = false;

    for pack_id in pack_order {
        let Some(pack) = roster.packs.iter().find(|row| row.pack_id == pack_id) else {
            continue;
        };
        for registration in &pack.registrations {
            if let Some(wanted) = requested_catalogue {
                release_seen |= registration
                    .releases
                    .iter()
                    .any(|release| release.id == wanted.release.as_str());
            }
            let projection = catalogue.reference_projection(
                pack_id,
                &registration.release_sha256,
                &projection_target,
            )?;
            if let Some(requested) = requested_catalogue
                && let Some(wanted_occurrence) = requested.occurrence.as_ref()
            {
                let release_contains_capture = projection.releases.iter().any(|release| {
                    release.summary.id == requested.release.as_str()
                        && release
                            .summary
                            .captures
                            .contains(&wanted_occurrence.capture)
                });
                let capture_contains_locator = projection.captures.iter().any(|capture| {
                    capture.capture == wanted_occurrence.capture
                        && capture
                            .locator_presence
                            .get(&wanted_occurrence.locator)
                            .copied()
                            == Some(true)
                });
                requested_occurrence_retained |=
                    release_contains_capture && capture_contains_locator;
            }
            let target_release = requested_catalogue.and_then(|wanted| {
                projection
                    .releases
                    .iter()
                    .find(|release| release.summary.id == wanted.release.as_str())
                    .map(|release| release.summary.clone())
            });
            if let Some(requested) = requested_catalogue
                && (target_release.is_some() || owned_release.is_some())
            {
                if requested.native_entry_id.is_none()
                    && requested.native_assertion_id.is_none()
                    && requested.occurrence.is_none()
                {
                    return Err(invalid_reference(
                        "catalogue target supplies no native identity or occurrence",
                    ));
                }
                if requested
                    .occurrence
                    .as_ref()
                    .is_some_and(|occurrence| occurrence.locator.is_empty())
                {
                    return Err(invalid_reference("catalogue occurrence locator is empty"));
                }
            }
            matching_release_owned |= target_release.is_some();
            let source_ids: Vec<String> = projection
                .entities
                .iter()
                .filter(|entity| target_entity_matches(entity, requested_catalogue, target))
                .flat_map(|entity| entity.source_ids.iter().cloned())
                .collect();
            let target_role_retained = projection
                .entities
                .iter()
                .any(|entity| target_entity_matches(entity, requested_catalogue, target));
            let relevant =
                target_release.is_some() || target_role_retained || !source_ids.is_empty();
            if !relevant {
                continue;
            }

            let pack_name = pack_id.as_str();
            let is_selected = selected.contains(&pack_id);
            let is_disabled = disabled.contains(&pack_id);
            let admitted = is_selected
                && !is_disabled
                && generation_members.iter().any(|member| {
                    member.pack_id == pack_name
                        && member.release_sha256 == registration.release_sha256
                });
            let mut registration_matches = Vec::new();
            let mut matched_entry_ids = std::collections::BTreeSet::new();
            let mut first_local_entities = std::collections::BTreeSet::new();
            if let Some(requested) = requested_catalogue {
                for entity in &projection.entities {
                    if entity.role != "assertion"
                        || !target_entity_matches(entity, Some(requested), target)
                    {
                        continue;
                    }
                    let occurrence = entity.occurrence.as_ref();
                    let attribution = occurrence.and_then(|wanted| {
                        projection.registry.occurrences.iter().find(|row| {
                            row.occurrence.snapshot().namespace() == wanted.capture.namespace
                                && row.occurrence.snapshot().sha256() == wanted.capture.sha256
                                && row.occurrence.locator() == wanted.locator
                        })
                    });
                    let entry_matches = requested.native_entry_id.as_ref().is_none_or(|expected| {
                        let actual = attribution.and_then(|row| row.native_entry_id.as_ref());
                        if strict_native {
                            actual == Some(expected)
                        } else {
                            actual.is_none_or(|actual| actual == expected)
                        }
                    });
                    let assertion_matches =
                        requested
                            .native_assertion_id
                            .as_ref()
                            .is_none_or(|expected| {
                                let actual =
                                    attribution.and_then(|row| row.native_assertion_id.as_ref());
                                if strict_native {
                                    actual == Some(expected)
                                } else {
                                    actual.is_none_or(|actual| actual == expected)
                                }
                            });
                    if entry_matches && assertion_matches {
                        matched_entry_ids.extend(entity.entry_ids.iter().cloned());
                    }
                }
            }
            for entity in &projection.entities {
                let target_match = target_entity_matches(entity, requested_catalogue, target)
                    || (entity.role == "entry" && matched_entry_ids.contains(&entity.id));
                if !target_match {
                    continue;
                }
                let occurrence = entity.occurrence.as_ref();
                let attribution = occurrence.and_then(|wanted| {
                    projection.registry.occurrences.iter().find(|row| {
                        row.occurrence.snapshot().namespace() == wanted.capture.namespace
                            && row.occurrence.snapshot().sha256() == wanted.capture.sha256
                            && row.occurrence.locator() == wanted.locator
                    })
                });
                let native_entry_id = attribution.and_then(|row| row.native_entry_id.clone());
                let native_assertion_id =
                    attribution.and_then(|row| row.native_assertion_id.clone());
                if entity.role == "assertion"
                    && let Some(wanted) = requested_catalogue
                {
                    let entry_mismatch = wanted.native_entry_id.as_ref().is_some_and(|expected| {
                        if strict_native {
                            native_entry_id.as_ref() != Some(expected)
                        } else {
                            native_entry_id
                                .as_ref()
                                .is_some_and(|actual| actual != expected)
                        }
                    });
                    let assertion_mismatch =
                        wanted.native_assertion_id.as_ref().is_some_and(|expected| {
                            if strict_native {
                                native_assertion_id.as_ref() != Some(expected)
                            } else {
                                native_assertion_id
                                    .as_ref()
                                    .is_some_and(|actual| actual != expected)
                            }
                        });
                    if entry_mismatch || assertion_mismatch {
                        continue;
                    }
                }
                let digest_key = format!("{}:{}", entity.role, entity.id);
                let body_sha256 = projection.body_sha256.get(&digest_key).ok_or_else(|| {
                    super::WorkbenchQueryError::Query(Box::new(
                        crate::DictionaryQueryError::Integrity {
                            detail: format!(
                                "retained target role {}:{} has no body digest",
                                entity.role, entity.id
                            ),
                        },
                    ))
                })?;
                if !valid_sha256(body_sha256) {
                    return Err(super::WorkbenchQueryError::Query(Box::new(
                        crate::DictionaryQueryError::Integrity {
                            detail: format!(
                                "retained target role {}:{} has a malformed body digest",
                                entity.role, entity.id
                            ),
                        },
                    )));
                }
                let compare_entity = !matches!(target, RelationTarget::Local(_))
                    || first_local_entities.insert((entity.role.clone(), entity.id.clone()));
                if compare_entity {
                    if let Some((previous, previous_digest)) =
                        retained_entities.get(&(entity.role.clone(), entity.id.clone()))
                    {
                        if previous != entity || previous_digest != body_sha256 {
                            ambiguous
                                .get_or_insert_with(|| (entity.role.clone(), entity.id.clone()));
                        }
                    } else {
                        retained_entities.insert(
                            (entity.role.clone(), entity.id.clone()),
                            (entity.clone(), body_sha256.clone()),
                        );
                    }
                }
                if !(matches!(target, RelationTarget::Catalog(_)) && entity.role == "entry") {
                    registration_matches.push(CollectionReferenceMatch {
                        pack_id: pack_name.to_owned(),
                        release_sha256: registration.release_sha256.clone(),
                        source_id: entity.source_ids.first().cloned().unwrap_or_default(),
                        model_release_id: requested_catalogue
                            .map_or_else(String::new, |wanted| wanted.release.to_string()),
                        role: Some(entity.role.clone()),
                        entity_id: Some(entity.id.clone()),
                        occurrence: occurrence.map(|value| value.locator.clone()),
                        capture: occurrence.map(|value| value.capture.clone()),
                        native_entry_id,
                        native_assertion_id,
                        contributing_pack_ids: vec![pack_name.to_owned()],
                    });
                }
            }
            matches.extend(registration_matches);
            let requested_release_for_registration = target_release;
            if requested_release.is_none() {
                requested_release = requested_release_for_registration.clone();
            }
            registrations.push(WorkbenchReferenceRegistration {
                pack_id,
                release_sha256: registration.release_sha256.clone(),
                payload_kind: registration.payload_kind,
                selected: is_selected,
                disabled: is_disabled,
                installation_state: pack.installation_state,
                installed_release_sha256: pack.installed_release_sha256.clone(),
                admitted,
                operational_reason: if is_disabled {
                    Some("disabled".into())
                } else if !is_selected {
                    Some("not_selected".into())
                } else if pack.installation_state == CollectionInstallationState::Removed {
                    Some("removed".into())
                } else if pack.installed_release_sha256.as_deref()
                    != Some(registration.release_sha256.as_str())
                {
                    Some("registered_only".into())
                } else if !registration.releases.iter().any(|release| {
                    release.selected
                        && release.availability == vlajmaji_lexicon::Availability::Available
                }) || !admitted
                {
                    Some("no_selected_available_release".into())
                } else {
                    None
                },
                sources: registration
                    .sources
                    .iter()
                    .filter(|source| {
                        source_ids.contains(&source.id.to_string())
                            || requested_catalogue.is_some_and(|wanted| source.id == wanted.source)
                    })
                    .cloned()
                    .collect(),
                requested_release: requested_release_for_registration,
            });
        }
    }
    if requested_catalogue.is_some() && !matching_release_owned && owned_release.is_none() {
        let detail = if release_seen {
            "catalogue release is not owned by the requested source"
        } else {
            "catalogue release is unknown"
        };
        return Err(invalid_reference(detail));
    }
    if requested_release.is_none() && !matching_release_owned {
        requested_release = owned_release.cloned();
    }
    if requested_catalogue.is_some_and(|wanted| wanted.occurrence.is_some())
        && !requested_occurrence_retained
        && owned_release.is_none()
    {
        return Err(invalid_reference(
            "catalogue occurrence is not present in the requested release",
        ));
    }
    if let Some((role, id)) = ambiguous {
        return Err(super::WorkbenchQueryError::AmbiguousReference { role, id });
    }
    let first_reason = top_reason(&matches, &registrations);
    drop(catalogue);
    if matches!(target, RelationTarget::Local(_)) {
        matches = deduplicate_matches(matches, &registrations);
    }
    let witnesses = retained_entities
        .into_values()
        .map(|(descriptor, body_sha256)| TypedRoleWitness {
            descriptor,
            body_sha256,
        })
        .collect();
    Ok(CatalogueObservation {
        matches,
        registrations,
        requested_release,
        witnesses,
        top_reason: first_reason,
    })
}

fn invalid_reference(detail: &str) -> super::WorkbenchQueryError {
    super::WorkbenchQueryError::Query(Box::new(crate::DictionaryQueryError::InvalidReference {
        detail: detail.to_owned(),
    }))
}

fn deduplicate_matches(
    rows: Vec<CollectionReferenceMatch>,
    registrations: &[WorkbenchReferenceRegistration],
) -> Vec<CollectionReferenceMatch> {
    let mut groups: Vec<CollectionReferenceMatch> = Vec::new();
    for row in rows {
        let key = (
            row.source_id.as_str(),
            row.role.as_deref(),
            row.entity_id.as_deref(),
            row.occurrence.as_deref(),
            row.capture.as_ref(),
            row.native_entry_id.as_deref(),
            row.native_assertion_id.as_deref(),
        );
        if let Some(existing) = groups.iter_mut().find(|existing| {
            (
                existing.source_id.as_str(),
                existing.role.as_deref(),
                existing.entity_id.as_deref(),
                existing.occurrence.as_deref(),
                existing.capture.as_ref(),
                existing.native_entry_id.as_deref(),
                existing.native_assertion_id.as_deref(),
            ) == key
        }) {
            if !existing.contributing_pack_ids.contains(&row.pack_id) {
                existing.contributing_pack_ids.push(row.pack_id.clone());
                existing.contributing_pack_ids.sort();
            }
            if registrations.iter().any(|registration| {
                registration.pack_id.as_str() == row.pack_id
                    && registration.installed_release_sha256.as_deref()
                        == Some(row.release_sha256.as_str())
            }) {
                existing.pack_id = row.pack_id;
                existing.release_sha256 = row.release_sha256;
            }
        } else {
            groups.push(row);
        }
    }
    groups
}

fn top_reason(
    matches: &[CollectionReferenceMatch],
    registrations: &[WorkbenchReferenceRegistration],
) -> Option<String> {
    let first = matches.first()?;
    registrations
        .iter()
        .find(|registration| {
            registration.pack_id.as_str() == first.pack_id
                && registration.release_sha256 == first.release_sha256
        })
        .and_then(|registration| registration.operational_reason.clone())
}

fn has_admitted_match(observation: &CatalogueObservation) -> bool {
    observation.matches.iter().any(|matched| {
        observation.registrations.iter().any(|registration| {
            registration.admitted
                && registration.pack_id.as_str() == matched.pack_id
                && registration.release_sha256 == matched.release_sha256
        })
    })
}

fn require_admitted_source_witness(
    target: &RelationTarget,
    observation: &CatalogueObservation,
    witnesses: &[TypedRoleWitness],
) -> Result<(), super::WorkbenchQueryError> {
    if matches!(target, RelationTarget::Catalog(_))
        && has_admitted_match(observation)
        && !source_has_target_role(target, witnesses)
    {
        return Err(super::WorkbenchQueryError::Query(Box::new(
            crate::DictionaryQueryError::Integrity {
                detail: "admitted catalogue target is absent from the prepared dictionary".into(),
            },
        )));
    }
    Ok(())
}

fn source_has_target_role(target: &RelationTarget, witnesses: &[TypedRoleWitness]) -> bool {
    match target {
        RelationTarget::Local(local) => {
            let (role, id) = match local {
                vlajmaji_lexicon::LocalRef::Entry(id) => ("entry", id.as_str()),
                vlajmaji_lexicon::LocalRef::Assertion(id) => ("assertion", id.as_str()),
                vlajmaji_lexicon::LocalRef::Form(id) => ("form", id.as_str()),
                vlajmaji_lexicon::LocalRef::EditorialSense(id) => ("editorial_sense", id.as_str()),
            };
            witnesses
                .iter()
                .any(|row| row.descriptor.role == role && row.descriptor.id == id)
        }
        RelationTarget::Catalog(catalog) => witnesses.iter().any(|row| {
            row.descriptor.role == "assertion"
                && row
                    .descriptor
                    .source_ids
                    .contains(&catalog.source.to_string())
                && row
                    .descriptor
                    .occurrence
                    .as_ref()
                    .is_some_and(|occurrence| {
                        catalog
                            .occurrence
                            .as_ref()
                            .is_none_or(|expected| expected == occurrence)
                    })
        }),
        RelationTarget::Unknown | RelationTarget::Unresolved(_) => false,
    }
}

fn should_resolve_source(
    target: &RelationTarget,
    has_catalogue: bool,
    observation: &CatalogueObservation,
    witnesses: &[TypedRoleWitness],
    owned_target: bool,
) -> bool {
    match target {
        RelationTarget::Local(_) => {
            source_has_target_role(target, witnesses) || observation.matches.is_empty()
        }
        RelationTarget::Catalog(_) if !has_catalogue => true,
        RelationTarget::Catalog(requested) => {
            let malformed_caller = (requested.native_entry_id.is_none()
                && requested.native_assertion_id.is_none()
                && requested.occurrence.is_none())
                || requested
                    .occurrence
                    .as_ref()
                    .is_some_and(|occurrence| occurrence.locator.is_empty());
            owned_target
                || (malformed_caller && observation.requested_release.is_some())
                || (source_has_target_role(target, witnesses) && has_admitted_match(observation))
        }
        RelationTarget::Unknown => false,
        RelationTarget::Unresolved(value) => value.is_empty(),
    }
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn check_source_witnesses(
    source: &[TypedRoleWitness],
    catalogue: &[TypedRoleWitness],
) -> Result<(), super::WorkbenchQueryError> {
    for source_row in source {
        if let Some(retained) = catalogue.iter().find(|row| {
            row.descriptor.role == source_row.descriptor.role
                && row.descriptor.id == source_row.descriptor.id
        }) && (retained.descriptor != source_row.descriptor
            || retained.body_sha256 != source_row.body_sha256)
        {
            return Err(super::WorkbenchQueryError::AmbiguousReference {
                role: source_row.descriptor.role.clone(),
                id: source_row.descriptor.id.clone(),
            });
        }
    }
    Ok(())
}

fn target_entity_matches(
    entity: &vlajmaji_store::CollectionEntityReference,
    catalog: Option<&vlajmaji_lexicon::CatalogTarget>,
    target: &RelationTarget,
) -> bool {
    match target {
        RelationTarget::Local(local) => {
            let (role, id) = match local {
                vlajmaji_lexicon::LocalRef::Entry(id) => ("entry", id.as_str()),
                vlajmaji_lexicon::LocalRef::Assertion(id) => ("assertion", id.as_str()),
                vlajmaji_lexicon::LocalRef::Form(id) => ("form", id.as_str()),
                vlajmaji_lexicon::LocalRef::EditorialSense(id) => ("editorial_sense", id.as_str()),
            };
            entity.role == role && entity.id == id
        }
        RelationTarget::Catalog(_) => {
            let Some(wanted) = catalog else { return false };
            entity.role == "assertion"
                && entity.occurrence.as_ref().is_some_and(|occurrence| {
                    wanted
                        .occurrence
                        .as_ref()
                        .is_none_or(|requested| requested == occurrence)
                })
                && entity.source_ids.contains(&wanted.source.to_string())
        }
        RelationTarget::Unknown | RelationTarget::Unresolved(_) => false,
    }
}

impl WorkbenchSearchEngine {
    /// Resolve one target without entering or leaving the public freshness guard.
    pub(crate) fn resolve_reference_full_inner(
        &mut self,
        target: &RelationTarget,
    ) -> Result<WorkbenchReferenceFullResult, super::WorkbenchQueryError> {
        let has_catalogue = self.view.binding().catalogue().is_some();
        let owned_source = self
            .view
            .binding()
            .owned()
            .map(|owned| owned.identity.source.clone());
        let own_release = if let (RelationTarget::Catalog(catalog), Some(source)) =
            (target, owned_source.as_ref())
        {
            self.full_engine()?
                .owned_release_summary(&catalog.release, source)
        } else {
            None
        };
        let own_release = owned_release_summary(owned_source.as_ref(), target, own_release);
        let mut observation = if is_nonmaterialized_target(target) {
            empty_observation()
        } else {
            catalogue_observation(&self.view, target, false, own_release.as_ref())?
        };
        let resolved = if !is_nonmaterialized_target(target) {
            let engine = self.full_engine()?;
            let owned_target = own_release.is_some();
            if observation.requested_release.is_none() {
                observation.requested_release = own_release.clone();
            }
            let witnesses = engine.typed_reference_witnesses(target)?;
            require_admitted_source_witness(target, &observation, &witnesses)?;
            check_source_witnesses(&witnesses, &observation.witnesses)?;
            if should_resolve_source(
                target,
                has_catalogue,
                &observation,
                &witnesses,
                owned_target,
            ) {
                Some(engine.resolve(target)?)
            } else {
                None
            }
        } else {
            None
        };
        let status = resolved.as_ref().map_or_else(
            || status_without_payload(target, &observation.matches, false),
            |value| value.status,
        );
        Ok(WorkbenchReferenceFullResult {
            target: target.clone(),
            status,
            operational_reason: resolved
                .is_none()
                .then(|| observation.top_reason.clone())
                .flatten(),
            payload: resolved.map(Box::new),
            catalogue_matches: observation.matches,
            requested_release: observation.requested_release,
            registrations: observation.registrations,
        })
    }

    /// Compact counterpart to [`Self::resolve_reference_full_inner`].
    pub(crate) fn resolve_reference_compact_inner(
        &mut self,
        target: &RelationTarget,
    ) -> Result<WorkbenchReferenceCompactResult, super::WorkbenchQueryError> {
        if matches!(target, RelationTarget::Catalog(catalog)
            if catalog.native_entry_id.is_none()
                && catalog.native_assertion_id.is_none()
                && catalog.occurrence.is_none())
        {
            return Err(invalid_reference(
                "catalogue target supplies no native identity or occurrence",
            ));
        }
        let has_catalogue = self.view.binding().catalogue().is_some();
        let owned_source = self
            .view
            .binding()
            .owned()
            .map(|owned| owned.identity.source.clone());
        let own_release = if let (RelationTarget::Catalog(catalog), Some(source)) =
            (target, owned_source.as_ref())
        {
            self.compact_engine()?
                .owned_release_summary(&catalog.release, source)
        } else {
            None
        };
        let own_release = owned_release_summary(owned_source.as_ref(), target, own_release);
        let mut observation = if is_nonmaterialized_target(target) {
            empty_observation()
        } else {
            catalogue_observation(&self.view, target, true, own_release.as_ref())?
        };
        let resolved = if !is_nonmaterialized_target(target) {
            let engine = self.compact_engine()?;
            let owned_target = own_release.is_some();
            if observation.requested_release.is_none() {
                observation.requested_release = own_release.clone();
            }
            let witnesses = engine.typed_reference_witnesses(target)?;
            require_admitted_source_witness(target, &observation, &witnesses)?;
            check_source_witnesses(&witnesses, &observation.witnesses)?;
            if should_resolve_source(
                target,
                has_catalogue,
                &observation,
                &witnesses,
                owned_target,
            ) {
                Some(engine.resolve(target)?)
            } else {
                None
            }
        } else {
            None
        };
        let status = resolved.as_ref().map_or_else(
            || status_without_payload(target, &observation.matches, true),
            |value| value.status,
        );
        Ok(WorkbenchReferenceCompactResult {
            target: target.clone(),
            status,
            operational_reason: resolved
                .is_none()
                .then(|| observation.top_reason.clone())
                .flatten(),
            payload: resolved.map(Box::new),
            catalogue_matches: observation.matches,
            requested_release: observation.requested_release,
            registrations: observation.registrations,
        })
    }

    /// Resolve an explicit target across this mixed view in Full format.
    ///
    /// The result keeps the caller's target and binding identity alongside the
    /// Full payload (when an admitted source can resolve it) and the retained
    /// catalogue observations. A `Catalog` target can therefore be useful even
    /// when it is only metadata-visible and has no payload.
    ///
    /// # Errors
    ///
    /// Returns a typed query error when the view is stale, the target is
    /// malformed or unknown to the relevant source, retained role rows
    /// disagree, or the Full projection fails. The error preserves the
    /// underlying view, query, projection, or catalogue cause.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use vlajmaji_lexicon::RelationTarget;
    /// use vlajmaji_search::{WorkbenchQueryError, WorkbenchSearchEngine};
    ///
    /// fn lookup(
    ///     engine: &mut WorkbenchSearchEngine,
    ///     target: &RelationTarget,
    /// ) -> Result<(), WorkbenchQueryError> {
    ///     let response = engine.resolve_reference(target)?;
    ///     assert_eq!(response.format, "full");
    ///     println!("{:?}: {} retained matches", response.result.status,
    ///         response.result.catalogue_matches.len());
    ///     Ok(())
    /// }
    /// ```
    pub fn resolve_reference(
        &mut self,
        target: &vlajmaji_lexicon::RelationTarget,
    ) -> Result<WorkbenchReferenceFullResponse, super::WorkbenchQueryError> {
        self.resolve_reference_with_hook(target, || {})
    }

    pub(super) fn resolve_reference_with_hook<F>(
        &mut self,
        target: &vlajmaji_lexicon::RelationTarget,
        after_inner: F,
    ) -> Result<WorkbenchReferenceFullResponse, super::WorkbenchQueryError>
    where
        F: FnOnce(),
    {
        self.view.ensure_current()?;
        let operation = self.resolve_reference_full_inner(target);
        after_inner();
        let freshness = self.view.ensure_current();
        freshness?;
        let result = operation?;
        Ok(WorkbenchReferenceFullResponse {
            version: 1,
            kind: "vlajmaji-workbench-reference".into(),
            format: "full".into(),
            binding_sha256: self.view.binding().binding_sha256().to_owned(),
            selection: self.selection.clone(),
            result,
        })
    }

    /// Resolve an explicit target across this mixed view in Compact format.
    ///
    /// Compact resolution preserves the Compact projection's member and
    /// attribution ordering. Its response includes the exact caller target,
    /// binding identity, optional admitted payload, and raw retained matches.
    ///
    /// # Errors
    ///
    /// Returns a typed query error when the view is stale, the target lacks a
    /// required catalogue identity, the target is unknown to the relevant
    /// source, retained role rows disagree, or Compact projection fails.
    /// Underlying view, query, projection, and catalogue errors remain typed.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use vlajmaji_lexicon::RelationTarget;
    /// use vlajmaji_search::{WorkbenchQueryError, WorkbenchSearchEngine};
    ///
    /// fn lookup_compact(
    ///     engine: &mut WorkbenchSearchEngine,
    ///     target: &RelationTarget,
    /// ) -> Result<(), WorkbenchQueryError> {
    ///     let response = engine.resolve_reference_compact(target)?;
    ///     assert_eq!(response.format, "compact");
    ///     println!("{:?}: {} retained matches", response.result.status,
    ///         response.result.catalogue_matches.len());
    ///     Ok(())
    /// }
    /// ```
    pub fn resolve_reference_compact(
        &mut self,
        target: &vlajmaji_lexicon::RelationTarget,
    ) -> Result<WorkbenchReferenceCompactResponse, super::WorkbenchQueryError> {
        self.view.ensure_current()?;
        let operation = self.resolve_reference_compact_inner(target);
        let freshness = self.view.ensure_current();
        freshness?;
        let result = operation?;
        Ok(WorkbenchReferenceCompactResponse {
            version: 1,
            kind: "vlajmaji-workbench-reference".into(),
            format: "compact".into(),
            binding_sha256: self.view.binding().binding_sha256().to_owned(),
            selection: self.selection.clone(),
            result,
        })
    }
}

impl ConfiguredWorkbenchSearchEngine {
    /// Resolve one configured Full target inside the caller's outer guard.
    pub(crate) fn resolve_configured_reference_full_inner(
        &mut self,
        target: &RelationTarget,
    ) -> Result<WorkbenchReferenceFullResult, super::WorkbenchQueryError> {
        let has_catalogue = self.view.view().binding().catalogue().is_some();
        let owned_source = self
            .view
            .view()
            .binding()
            .owned()
            .map(|owned| owned.identity.source.clone());
        let own_release = if let (RelationTarget::Catalog(catalog), Some(source)) =
            (target, owned_source.as_ref())
        {
            self.full_engine()?
                .owned_release_summary(&catalog.release, source)
        } else {
            None
        };
        let own_release = owned_release_summary(owned_source.as_ref(), target, own_release);
        let mut observation = if is_nonmaterialized_target(target) {
            empty_observation()
        } else {
            catalogue_observation(self.view.view(), target, false, own_release.as_ref())?
        };
        let resolved = if !is_nonmaterialized_target(target) {
            let engine = self.full_engine()?;
            let owned_target = own_release.is_some();
            if observation.requested_release.is_none() {
                observation.requested_release = own_release.clone();
            }
            let witnesses = engine.typed_reference_witnesses(target)?;
            require_admitted_source_witness(target, &observation, &witnesses)?;
            check_source_witnesses(&witnesses, &observation.witnesses)?;
            if should_resolve_source(
                target,
                has_catalogue,
                &observation,
                &witnesses,
                owned_target,
            ) {
                Some(engine.resolve(target)?)
            } else {
                None
            }
        } else {
            None
        };
        let status = resolved.as_ref().map_or_else(
            || status_without_payload(target, &observation.matches, false),
            |value| value.status,
        );
        Ok(WorkbenchReferenceFullResult {
            target: target.clone(),
            status,
            operational_reason: resolved
                .is_none()
                .then(|| observation.top_reason.clone())
                .flatten(),
            payload: resolved.map(Box::new),
            catalogue_matches: observation.matches,
            requested_release: observation.requested_release,
            registrations: observation.registrations,
        })
    }

    /// Resolve one configured Compact target inside the caller's outer guard.
    pub(crate) fn resolve_configured_reference_compact_inner(
        &mut self,
        target: &RelationTarget,
    ) -> Result<WorkbenchReferenceCompactResult, super::WorkbenchQueryError> {
        if matches!(target, RelationTarget::Catalog(catalog)
            if catalog.native_entry_id.is_none()
                && catalog.native_assertion_id.is_none()
                && catalog.occurrence.is_none())
        {
            return Err(invalid_reference(
                "catalogue target supplies no native identity or occurrence",
            ));
        }
        let has_catalogue = self.view.view().binding().catalogue().is_some();
        let owned_source = self
            .view
            .view()
            .binding()
            .owned()
            .map(|owned| owned.identity.source.clone());
        let own_release = if let (RelationTarget::Catalog(catalog), Some(source)) =
            (target, owned_source.as_ref())
        {
            self.compact_engine()?
                .owned_release_summary(&catalog.release, source)
        } else {
            None
        };
        let own_release = owned_release_summary(owned_source.as_ref(), target, own_release);
        let mut observation = if is_nonmaterialized_target(target) {
            empty_observation()
        } else {
            catalogue_observation(self.view.view(), target, true, own_release.as_ref())?
        };
        let resolved = if !is_nonmaterialized_target(target) {
            let engine = self.compact_engine()?;
            let owned_target = own_release.is_some();
            if observation.requested_release.is_none() {
                observation.requested_release = own_release.clone();
            }
            let witnesses = engine.typed_reference_witnesses(target)?;
            require_admitted_source_witness(target, &observation, &witnesses)?;
            check_source_witnesses(&witnesses, &observation.witnesses)?;
            if should_resolve_source(
                target,
                has_catalogue,
                &observation,
                &witnesses,
                owned_target,
            ) {
                Some(engine.resolve(target)?)
            } else {
                None
            }
        } else {
            None
        };
        let status = resolved.as_ref().map_or_else(
            || status_without_payload(target, &observation.matches, true),
            |value| value.status,
        );
        Ok(WorkbenchReferenceCompactResult {
            target: target.clone(),
            status,
            operational_reason: resolved
                .is_none()
                .then(|| observation.top_reason.clone())
                .flatten(),
            payload: resolved.map(Box::new),
            catalogue_matches: observation.matches,
            requested_release: observation.requested_release,
            registrations: observation.registrations,
        })
    }

    /// Resolve a target against a configured profile in Full format.
    ///
    /// The response records the captured profile identity and effective
    /// binding used for the lookup, together with the same Full payload and
    /// retained-catalogue observations returned by an explicit workbench.
    ///
    /// # Errors
    ///
    /// Returns a profile error if the selected configuration is no longer
    /// current, or a typed query error for invalid/unknown targets, conflicting
    /// retained rows, and Full projection failures. The configured error keeps
    /// profile and query failures in separate variants.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use vlajmaji_lexicon::RelationTarget;
    /// use vlajmaji_search::{ConfiguredWorkbenchQueryError, ConfiguredWorkbenchSearchEngine};
    ///
    /// fn lookup_configured(
    ///     engine: &mut ConfiguredWorkbenchSearchEngine,
    ///     target: &RelationTarget,
    /// ) -> Result<(), ConfiguredWorkbenchQueryError> {
    ///     let response = engine.resolve_reference(target)?;
    ///     assert_eq!(response.format, "full");
    ///     println!("profile revision {}", response.configuration.config_revision);
    ///     Ok(())
    /// }
    /// ```
    pub fn resolve_reference(
        &mut self,
        target: &vlajmaji_lexicon::RelationTarget,
    ) -> Result<ConfiguredWorkbenchReferenceFullResponse, super::ConfiguredWorkbenchQueryError>
    {
        self.resolve_reference_with_hook(target, || {})
    }

    pub(super) fn resolve_reference_with_hook<F>(
        &mut self,
        target: &vlajmaji_lexicon::RelationTarget,
        after_inner: F,
    ) -> Result<ConfiguredWorkbenchReferenceFullResponse, super::ConfiguredWorkbenchQueryError>
    where
        F: FnOnce(),
    {
        self.view
            .ensure_current()
            .map_err(|error| super::ConfiguredWorkbenchQueryError::Profile(Box::new(error)))?;
        let operation = self.resolve_configured_reference_full_inner(target);
        after_inner();
        let freshness = self
            .view
            .ensure_current()
            .map_err(|error| super::ConfiguredWorkbenchQueryError::Profile(Box::new(error)));
        freshness?;
        let result = operation
            .map_err(|error| super::ConfiguredWorkbenchQueryError::Query(Box::new(error)))?;
        Ok(ConfiguredWorkbenchReferenceFullResponse {
            version: 1,
            kind: "vlajmaji-workbench-reference".into(),
            format: "full".into(),
            binding_sha256: self.view.identity().effective_binding_sha256.clone(),
            configuration: self.view.identity().clone(),
            selection: self.selection.clone(),
            result,
        })
    }

    /// Resolve a target against a configured profile in Compact format.
    ///
    /// The response includes the captured profile identity and effective
    /// binding, so callers can keep the Compact result tied to the exact
    /// configured selection that produced it.
    ///
    /// # Errors
    ///
    /// Returns a profile error if the configuration changed, or a typed query
    /// error for malformed/unknown targets, conflicting retained rows, and
    /// Compact projection failures. The configured error distinguishes those
    /// profile and query causes.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use vlajmaji_lexicon::RelationTarget;
    /// use vlajmaji_search::{ConfiguredWorkbenchQueryError, ConfiguredWorkbenchSearchEngine};
    ///
    /// fn lookup_configured_compact(
    ///     engine: &mut ConfiguredWorkbenchSearchEngine,
    ///     target: &RelationTarget,
    /// ) -> Result<(), ConfiguredWorkbenchQueryError> {
    ///     let response = engine.resolve_reference_compact(target)?;
    ///     assert_eq!(response.format, "compact");
    ///     println!("profile revision {}", response.configuration.config_revision);
    ///     Ok(())
    /// }
    /// ```
    pub fn resolve_reference_compact(
        &mut self,
        target: &vlajmaji_lexicon::RelationTarget,
    ) -> Result<ConfiguredWorkbenchReferenceCompactResponse, super::ConfiguredWorkbenchQueryError>
    {
        self.view
            .ensure_current()
            .map_err(|error| super::ConfiguredWorkbenchQueryError::Profile(Box::new(error)))?;
        let operation = self.resolve_configured_reference_compact_inner(target);
        let freshness = self
            .view
            .ensure_current()
            .map_err(|error| super::ConfiguredWorkbenchQueryError::Profile(Box::new(error)));
        freshness?;
        let result = operation
            .map_err(|error| super::ConfiguredWorkbenchQueryError::Query(Box::new(error)))?;
        Ok(ConfiguredWorkbenchReferenceCompactResponse {
            version: 1,
            kind: "vlajmaji-workbench-reference".into(),
            format: "compact".into(),
            binding_sha256: self.view.identity().effective_binding_sha256.clone(),
            configuration: self.view.identity().clone(),
            selection: self.selection.clone(),
            result,
        })
    }
}
