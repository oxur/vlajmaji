//! Managed compact queries reuse the captured collection policy while keeping
//! the Unified payload behind the compact dictionary projection boundary.

use std::{
    collections::BTreeSet,
    time::{Duration, Instant},
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use vlajmaji_lexicon::{LocalRef, PackId, RelationTarget};
use vlajmaji_store::{CollectionCatalogue, CollectionError, DictionaryReadTopology};

use crate::{
    DictionaryCompactExactResponse, DictionaryCompactReferenceResolution,
    DictionaryCompactSearchEngine, DictionaryHitDetailRef, DictionaryHitDetailResponse,
    DictionaryProjectionProfile, DictionaryQueryError, DictionarySnapshotOpenProfile,
};

use super::{
    CollectionQueryError, CollectionQuerySelection, CollectionReferenceMatch,
    CollectionSearchEngine, CollectionSelectionSnapshot,
};

/// Versioned response for an explicitly selected compact collection query.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CollectionCompactQueryResponse {
    /// Response schema version.
    pub version: u32,
    /// Captured operational identity.
    pub snapshot: CollectionSelectionSnapshot,
    /// Per-pack availability and selection facts.
    pub collections: Vec<super::CollectionPackAvailability>,
    /// Compact operation result.
    pub result: CollectionCompactQueryResult,
}

/// Compact managed exact or reference result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "operation", content = "response", rename_all = "snake_case")]
pub enum CollectionCompactQueryResult {
    /// Compact exact matches with lazy detail references.
    Exact(Box<DictionaryCompactExactResponse>),
    /// Compact typed-reference result.
    Resolve(Box<CollectionCompactReferenceResponse>),
}

/// Compact managed reference response. Full hit bodies are available through
/// `detail` only after validating a `DictionaryHitDetailRef`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CollectionCompactReferenceResponse {
    /// Caller target.
    pub target: RelationTarget,
    /// Resolution status.
    pub status: crate::ReferenceStatus,
    /// Operational reason when no selected payload was admitted.
    pub operational_reason: Option<String>,
    /// Compact reference projection when the target exists in the selected payload.
    pub payload: Option<DictionaryCompactReferenceResolution>,
    /// Registration-only matches.
    pub catalogue_matches: Vec<CollectionReferenceMatch>,
    /// Requested release summary when the target names a known release.
    pub requested_release: Option<vlajmaji_store::CollectionReleaseSummary>,
}

/// Full hit detail attached to the managed collection snapshot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CollectionHitDetailResponse {
    /// Response schema version.
    pub version: u32,
    /// Captured operational identity.
    pub snapshot: CollectionSelectionSnapshot,
    /// Per-pack availability and selection facts.
    pub collections: Vec<super::CollectionPackAvailability>,
    /// Validated complete hit response.
    pub response: DictionaryHitDetailResponse,
    /// Retained registration matches for the requested assertion.
    pub catalogue_matches: Vec<CollectionReferenceMatch>,
}

/// Same-call timing for managed compact operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CollectionProjectionProfile {
    /// Time for the complete operation.
    pub total: Duration,
    /// Time spent opening and hydrating the selected payload.
    pub payload_setup: Option<DictionarySnapshotOpenProfile>,
    /// Query timing for the exact or reference operation.
    pub query: DictionaryProjectionProfile,
}

/// Managed compact query state. The policy snapshot is captured before the
/// derived Dictionary document is opened, so metadata-only reference queries
/// remain available without loading the Unified payload.
pub struct CollectionCompactSearchEngine {
    base: CollectionSearchEngine,
    compact: Option<DictionaryCompactSearchEngine>,
}

impl CollectionCompactSearchEngine {
    /// Capture an explicit collection selection for compact operations.
    pub fn open(
        root: impl AsRef<std::path::Path>,
        selection: CollectionQuerySelection,
    ) -> Result<Self, CollectionQueryError> {
        Self::from_base(CollectionSearchEngine::open(root, selection)?)
    }

    /// Capture a named/default profile from an already parsed configuration.
    pub fn open_configured(
        profiles: &vlajmaji_store::CollectionProfiles,
        name: Option<&str>,
        override_packs: Option<&[PackId]>,
    ) -> Result<Self, CollectionQueryError> {
        Self::from_base(CollectionSearchEngine::open_configured(
            profiles,
            name,
            override_packs,
        )?)
    }

    /// Capture a named/default profile and report the time needed to read and
    /// validate its configuration and collection policy.
    pub fn open_configured_profiled(
        profiles: &vlajmaji_store::CollectionProfiles,
        name: Option<&str>,
        override_packs: Option<&[PackId]>,
    ) -> Result<(Self, Duration), CollectionQueryError> {
        let started = Instant::now();
        let engine = Self::open_configured(profiles, name, override_packs)?;
        Ok((engine, started.elapsed()))
    }

    fn from_base(mut base: CollectionSearchEngine) -> Result<Self, CollectionQueryError> {
        base.snapshot.projection_policy = "compact-collection-query-v1".into();
        base.snapshot.binding_sha256 = super::binding(&base.snapshot)?;
        if let Some(manifest) = &mut base.manifest {
            manifest.generation = base.snapshot.binding_sha256.clone();
        }
        Ok(Self {
            base,
            compact: None,
        })
    }

    /// Return the captured managed selection.
    #[must_use]
    pub fn selection(&self) -> &CollectionSelectionSnapshot {
        &self.base.snapshot
    }

    /// Execute a compact exact or typed-reference request.
    pub fn execute(
        &mut self,
        request: crate::DictionaryRequest,
    ) -> Result<CollectionCompactQueryResponse, CollectionQueryError> {
        self.execute_inner(request, None)
    }

    /// Execute a compact managed request and capture same-call timings.
    pub fn execute_profiled(
        &mut self,
        request: crate::DictionaryRequest,
    ) -> Result<(CollectionCompactQueryResponse, CollectionProjectionProfile), CollectionQueryError>
    {
        let started = Instant::now();
        let mut profile = CollectionProjectionProfile::default();
        let response = self.execute_inner(request, Some(&mut profile))?;
        profile.total = started.elapsed();
        profile.query.total = profile.total.saturating_sub(
            profile
                .payload_setup
                .map_or(Duration::ZERO, |setup| setup.total),
        );
        profile.query.lookup = profile.query.total.saturating_sub(profile.query.projection);
        Ok((response, profile))
    }

    fn execute_inner(
        &mut self,
        request: crate::DictionaryRequest,
        mut profile: Option<&mut CollectionProjectionProfile>,
    ) -> Result<CollectionCompactQueryResponse, CollectionQueryError> {
        let result = match request {
            crate::DictionaryRequest::Exact {
                spelling,
                mut options,
            } => {
                if let Some(cursor) = options.cursor.as_deref() {
                    options.cursor = Some(self.unwrap_managed_cursor(cursor)?);
                }
                self.base.admit_exact()?;
                if let Some(profile) = profile.as_deref_mut() {
                    self.ensure_compact(&mut profile.payload_setup)?;
                } else {
                    self.ensure_compact(&mut None)?;
                }
                let engine = self.compact.as_ref().ok_or_else(|| {
                    CollectionQueryError::Query(DictionaryQueryError::Integrity {
                        detail: "compact payload was not initialized".into(),
                    })
                })?;
                let (compact_result, query_profile) = if profile.is_some() {
                    engine
                        .execute_profiled(crate::DictionaryRequest::Exact { spelling, options })?
                } else {
                    (
                        engine.execute(crate::DictionaryRequest::Exact { spelling, options })?,
                        crate::DictionaryProjectionProfile::default(),
                    )
                };
                if let Some(profile) = profile.as_deref_mut() {
                    profile.query = query_profile;
                }
                let mut response = match compact_result {
                    crate::DictionaryCompactQueryResult::Exact(response) => *response,
                    crate::DictionaryCompactQueryResult::Resolve(_) => {
                        return Err(CollectionQueryError::Query(
                            DictionaryQueryError::Integrity {
                                detail: "exact compact execution returned a reference result"
                                    .into(),
                            },
                        ));
                    }
                };
                if let Some(cursor) = response.next_cursor.take() {
                    let projection_started = profile.as_ref().map(|_| Instant::now());
                    response.next_cursor = Some(self.wrap_managed_cursor(&cursor)?);
                    if let (Some(profile), Some(started)) =
                        (profile.as_deref_mut(), projection_started)
                    {
                        profile.query.projection += started.elapsed();
                    }
                }
                let response =
                    super::measure_collection_projection(&mut profile, || Box::new(response));
                CollectionCompactQueryResult::Exact(response)
            }
            crate::DictionaryRequest::Resolve { target } => {
                if let Some(profile) = profile.as_deref_mut() {
                    self.ensure_compact_if_selected(&target, &mut profile.payload_setup)?;
                } else {
                    let mut ignored_setup = None;
                    self.ensure_compact_if_selected(&target, &mut ignored_setup)?;
                }
                let response = self.resolve_compact(target, profile.as_deref_mut())?;
                let response =
                    super::measure_collection_projection(&mut profile, || Box::new(response));
                CollectionCompactQueryResult::Resolve(response)
            }
        };
        let projection_started = profile.as_ref().map(|_| Instant::now());
        let response = self.response(result);
        if let (Some(profile), Some(started)) = (profile, projection_started) {
            profile.query.projection += started.elapsed();
        }
        Ok(response)
    }

    /// Resolve one complete hit from a prior compact key.
    pub fn detail(
        &mut self,
        detail_ref: &DictionaryHitDetailRef,
        expected_snapshot: Option<&str>,
    ) -> Result<CollectionHitDetailResponse, CollectionQueryError> {
        // Reject malformed caller material before attempting any derived-store
        // I/O.  A missing or damaged payload must not mask a malformed key or
        // snapshot token at this public boundary.
        detail_ref
            .validate()
            .map_err(CollectionQueryError::Projection)?;
        if let Some(expected) = expected_snapshot {
            crate::validate_detail_snapshot_sha256(expected)
                .map_err(CollectionQueryError::Projection)?;
            if expected != self.base.snapshot.binding_sha256 {
                return Err(CollectionQueryError::Projection(
                    crate::DictionaryProjectionError::StaleSnapshot {
                        expected: expected.into(),
                        observed: self.base.snapshot.binding_sha256.clone(),
                    },
                ));
            }
        }
        let (matches, invalid_reference) = self.compact_detail_matches(detail_ref)?;
        if matches.is_empty() {
            if let Some(detail) = invalid_reference {
                return Err(CollectionQueryError::Query(
                    DictionaryQueryError::InvalidReference { detail },
                ));
            }
            let response = DictionaryHitDetailResponse {
                version: 1,
                projection_policy: "complete-dictionary-hit-v1".into(),
                detail_ref: detail_ref.clone(),
                status: crate::ReferenceStatus::NoMatch,
                operational_reason: None,
                state_sha256: self.base.snapshot.recorded_state_sha256.clone(),
                snapshot_sha256: self.base.snapshot.binding_sha256.clone(),
                hit: None,
                current_eligible: None,
            };
            return Ok(CollectionHitDetailResponse {
                version: 1,
                snapshot: self.base.snapshot.clone(),
                collections: self.base.snapshot.collections.clone(),
                response,
                catalogue_matches: Vec::new(),
            });
        }
        let reasons = matches
            .iter()
            .map(|matched| {
                self.base
                    .operational_reason(&matched.pack_id, &matched.release_sha256)
            })
            .collect::<Vec<_>>();
        let catalogue_matches = coalesce_detail_matches(&matches, self.base.generation.as_ref());
        let reason = reasons.iter().find_map(|reason| reason.as_deref());
        let metadata_only = reasons.iter().all(|reason| {
            reason.as_deref().is_some_and(|reason| {
                matches!(
                    reason,
                    "disabled" | "not_selected" | "removed" | "registered_only"
                )
            })
        });
        if !metadata_only {
            // A selected matching scope must open its real payload so missing,
            // corrupt, unsupported-version and integrity failures stay typed.
            self.ensure_compact(&mut None)?;
            let mut response = self
                .compact
                .as_ref()
                .expect("compact engine initialized")
                .detail(detail_ref, None)
                .map_err(CollectionQueryError::Projection)?;
            response.snapshot_sha256 = self.base.snapshot.binding_sha256.clone();
            return Ok(CollectionHitDetailResponse {
                version: 1,
                snapshot: self.base.snapshot.clone(),
                collections: self.base.snapshot.collections.clone(),
                response,
                catalogue_matches,
            });
        }
        let response = DictionaryHitDetailResponse {
            version: 1,
            projection_policy: "complete-dictionary-hit-v1".into(),
            detail_ref: detail_ref.clone(),
            status: crate::ReferenceStatus::CatalogueOnly,
            operational_reason: reason.map(str::to_owned),
            state_sha256: self.base.snapshot.recorded_state_sha256.clone(),
            snapshot_sha256: self.base.snapshot.binding_sha256.clone(),
            hit: None,
            current_eligible: None,
        };
        Ok(CollectionHitDetailResponse {
            version: 1,
            snapshot: self.base.snapshot.clone(),
            collections: self.base.snapshot.collections.clone(),
            response,
            catalogue_matches,
        })
    }

    fn compact_detail_matches(
        &self,
        reference: &DictionaryHitDetailRef,
    ) -> Result<(Vec<CollectionReferenceMatch>, Option<String>), CollectionQueryError> {
        let catalogue = CollectionCatalogue::open(&self.base.root)?;
        if catalogue.local_revision()? != self.base.snapshot.local_revision {
            return Err(CollectionQueryError::Catalogue(
                CollectionError::InvalidRequest(
                    "stale_snapshot: catalogue revision changed".into(),
                ),
            ));
        }
        let list = catalogue.list()?;
        let target = RelationTarget::Local(LocalRef::Assertion(reference.assertion_id.clone()));
        let mut matches = Vec::new();
        let mut invalid_reference = None;
        for pack in list.packs {
            let pack_id = super::parse_pack_id(&pack.pack_id).ok_or_else(|| {
                CollectionQueryError::InvalidSelection(format!(
                    "unsupported registered pack ID {}",
                    pack.pack_id
                ))
            })?;
            for registration in pack.registrations {
                let projection = catalogue.reference_projection(
                    pack_id,
                    &registration.release_sha256,
                    &target,
                )?;
                let assertion = projection.entities.iter().find(|entity| {
                    entity.role == "assertion" && entity.id == reference.assertion_id.as_str()
                });
                let entry = projection.entities.iter().find(|entity| {
                    entity.role == "entry" && entity.id == reference.entry_id.as_str()
                });
                let Some(assertion) = assertion else {
                    continue;
                };
                if entry.is_none()
                    || !assertion
                        .entry_ids
                        .contains(&reference.entry_id.to_string())
                {
                    invalid_reference.get_or_insert_with(|| {
                        "detail assertion does not link to the requested entry".into()
                    });
                    continue;
                }
                if !assertion.source_ids.contains(&reference.source.to_string())
                    || assertion.occurrence.as_ref() != Some(&reference.occurrence)
                {
                    invalid_reference.get_or_insert_with(|| {
                        "detail assertion source or occurrence does not match the requested key"
                            .into()
                    });
                    continue;
                }
                let body_matches = projection
                    .body_sha256
                    .get(&format!("assertion:{}", reference.assertion_id))
                    == Some(&reference.assertion_sha256)
                    && projection
                        .body_sha256
                        .get(&format!("entry:{}", reference.entry_id))
                        == Some(&reference.entry_sha256);
                if !body_matches {
                    continue;
                }
                let native = projection.registry.occurrences.iter().find(|row| {
                    row.occurrence.snapshot().namespace() == reference.occurrence.capture.namespace
                        && row.occurrence.snapshot().sha256() == reference.occurrence.capture.sha256
                        && row.occurrence.locator() == reference.occurrence.locator
                });
                let snapshot = projection.registry.snapshots.iter().find(|row| {
                    row.snapshot.namespace() == reference.occurrence.capture.namespace
                        && row.snapshot.sha256() == reference.occurrence.capture.sha256
                });
                let actual_native_entry = native.and_then(|row| row.native_entry_id.as_deref());
                let actual_native_assertion =
                    native.and_then(|row| row.native_assertion_id.as_deref());
                if reference
                    .native_entry_id
                    .as_deref()
                    .is_some_and(|requested| {
                        actual_native_entry != Some(requested)
                            && actual_native_assertion == Some(requested)
                    })
                    || reference
                        .native_assertion_id
                        .as_deref()
                        .is_some_and(|requested| {
                            actual_native_assertion != Some(requested)
                                && actual_native_entry == Some(requested)
                        })
                {
                    invalid_reference.get_or_insert_with(|| {
                        "native identity was supplied in the wrong role slot".into()
                    });
                    continue;
                }
                if native.is_none() || snapshot.is_none() {
                    invalid_reference.get_or_insert_with(|| {
                        "detail occurrence has no retained capture owner".into()
                    });
                    continue;
                }
                if actual_native_entry != reference.native_entry_id.as_deref()
                    || actual_native_assertion != reference.native_assertion_id.as_deref()
                    || native.and_then(|row| row.contributor.as_ref())
                        != reference.contributor_id.as_ref()
                    || snapshot.and_then(|row| row.distributor.as_ref())
                        != reference.distributor_id.as_ref()
                {
                    continue;
                }
                matches.push(CollectionReferenceMatch {
                    pack_id: pack.pack_id.clone(),
                    release_sha256: registration.release_sha256,
                    source_id: reference.source.to_string(),
                    model_release_id: String::new(),
                    role: Some("assertion".into()),
                    entity_id: Some(reference.assertion_id.to_string()),
                    occurrence: Some(reference.occurrence.locator.clone()),
                    capture: Some(reference.occurrence.capture.clone()),
                    native_entry_id: reference.native_entry_id.clone(),
                    native_assertion_id: reference.native_assertion_id.clone(),
                    contributing_pack_ids: vec![pack.pack_id.clone()],
                });
            }
        }
        Ok((matches, invalid_reference))
    }

    fn ensure_compact(
        &mut self,
        setup_profile: &mut Option<DictionarySnapshotOpenProfile>,
    ) -> Result<(), CollectionQueryError> {
        if self.compact.is_some() {
            return Ok(());
        }
        let manifest =
            self.base
                .manifest
                .clone()
                .ok_or_else(|| CollectionQueryError::UnavailablePack {
                    pack_id: "unified".into(),
                    reason: "no_generation".into(),
                })?;
        let (engine, profile) = DictionaryCompactSearchEngine::open_snapshot_profiled(
            DictionaryReadTopology::Unified,
            manifest,
        )?;
        self.compact = Some(engine);
        *setup_profile = Some(profile);
        Ok(())
    }

    fn ensure_compact_if_selected(
        &mut self,
        target: &RelationTarget,
        setup_profile: &mut Option<DictionarySnapshotOpenProfile>,
    ) -> Result<(), CollectionQueryError> {
        if matches!(
            target,
            RelationTarget::Unknown | RelationTarget::Unresolved(_)
        ) {
            return Ok(());
        }
        let matches = self.compact_metadata_matches(target, None)?;
        if self
            .base
            .selected_payload_for_target(target, &matches.admission)
            .is_some()
        {
            self.ensure_compact(setup_profile)?;
        }
        Ok(())
    }

    fn compact_metadata_matches(
        &self,
        target: &RelationTarget,
        mut profile: Option<&mut CollectionProjectionProfile>,
    ) -> Result<super::MetadataMatches, CollectionQueryError> {
        let RelationTarget::Catalog(requested) = target else {
            let RelationTarget::Local(local) = target else {
                return self.base.metadata_matches(target, profile.as_deref_mut());
            };
            return self.compact_local_metadata_matches(local, profile);
        };
        let catalogue = CollectionCatalogue::open(&self.base.root)?;
        if catalogue.local_revision()? != self.base.snapshot.local_revision {
            return Err(CollectionQueryError::Catalogue(
                CollectionError::InvalidRequest(
                    "stale_snapshot: catalogue revision changed".into(),
                ),
            ));
        }
        if requested.occurrence.is_none()
            && requested.native_entry_id.is_none()
            && requested.native_assertion_id.is_none()
        {
            return Err(CollectionQueryError::Query(
                DictionaryQueryError::InvalidReference {
                    detail: "catalogue target needs a native identity or occurrence".into(),
                },
            ));
        }
        let list = catalogue.list()?;
        let mut release_owner_found = false;
        let mut requested_release = None;
        let mut rows = Vec::new();
        for pack in &list.packs {
            let pack_id = super::parse_pack_id(&pack.pack_id).ok_or_else(|| {
                CollectionQueryError::InvalidSelection(format!(
                    "unsupported registered pack ID {}",
                    pack.pack_id
                ))
            })?;
            for registration in &pack.registrations {
                let projection = catalogue.reference_projection(
                    pack_id,
                    &registration.release_sha256,
                    target,
                )?;
                let Some(release) = projection
                    .releases
                    .iter()
                    .find(|release| release.summary.id == requested.release.as_str())
                else {
                    continue;
                };
                release_owner_found = true;
                if requested_release.is_none() {
                    requested_release =
                        Some(super::measure_collection_projection(&mut profile, || {
                            release.summary.clone()
                        }));
                }
                if requested.occurrence.as_ref().is_some_and(|occurrence| {
                    occurrence.locator.is_empty()
                        || !release.summary.captures.contains(&occurrence.capture)
                        || !projection.captures.iter().any(|capture| {
                            capture.capture == occurrence.capture
                                && capture.locator_presence.get(&occurrence.locator) == Some(&true)
                        })
                }) {
                    return Err(CollectionQueryError::Query(
                        DictionaryQueryError::InvalidReference {
                            detail: "catalogue occurrence is not in the requested release".into(),
                        },
                    ));
                }
                for entity in projection
                    .entities
                    .iter()
                    .filter(|entity| entity.role == "assertion")
                {
                    let Some(occurrence) = entity.occurrence.as_ref() else {
                        continue;
                    };
                    let native = projection.registry.occurrences.iter().find(|row| {
                        row.occurrence.snapshot().namespace() == occurrence.capture.namespace
                            && row.occurrence.snapshot().sha256() == occurrence.capture.sha256
                            && row.occurrence.locator() == occurrence.locator
                    });
                    let native_entry = native.and_then(|row| row.native_entry_id.clone());
                    let native_assertion = native.and_then(|row| row.native_assertion_id.clone());
                    if requested
                        .native_entry_id
                        .as_ref()
                        .is_some_and(|expected| native_entry.as_ref() != Some(expected))
                        || requested
                            .native_assertion_id
                            .as_ref()
                            .is_some_and(|expected| native_assertion.as_ref() != Some(expected))
                    {
                        continue;
                    }
                    rows.push(CollectionReferenceMatch {
                        pack_id: pack.pack_id.clone(),
                        release_sha256: registration.release_sha256.clone(),
                        source_id: requested.source.to_string(),
                        model_release_id: requested.release.to_string(),
                        role: Some("assertion".into()),
                        entity_id: Some(entity.id.clone()),
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
            return Err(CollectionQueryError::Query(
                DictionaryQueryError::InvalidReference {
                    detail:
                        "catalogue release is unknown or does not belong to the requested source"
                            .into(),
                },
            ));
        }
        let public = super::measure_collection_projection(&mut profile, || rows.clone());
        Ok(super::MetadataMatches {
            public,
            admission: rows,
            requested_release,
        })
    }

    fn compact_local_metadata_matches(
        &self,
        local: &LocalRef,
        mut profile: Option<&mut CollectionProjectionProfile>,
    ) -> Result<super::MetadataMatches, CollectionQueryError> {
        let catalogue = CollectionCatalogue::open(&self.base.root)?;
        if catalogue.local_revision()? != self.base.snapshot.local_revision {
            return Err(CollectionQueryError::Catalogue(
                CollectionError::InvalidRequest(
                    "stale_snapshot: catalogue revision changed".into(),
                ),
            ));
        }
        let list = catalogue.list()?;
        let (role, id) = super::local_role_id(local);
        let target = RelationTarget::Local(local.clone());
        let mut projections = Vec::new();
        for pack in &list.packs {
            let pack_id = super::parse_pack_id(&pack.pack_id).ok_or_else(|| {
                CollectionQueryError::InvalidSelection(format!(
                    "unsupported registered pack ID {}",
                    pack.pack_id
                ))
            })?;
            for registration in &pack.registrations {
                let projection = catalogue.reference_projection(
                    pack_id,
                    &registration.release_sha256,
                    &target,
                )?;
                if let Some(entity) = projection
                    .entities
                    .iter()
                    .find(|entity| entity.role == role && entity.id == id)
                    .cloned()
                {
                    projections.push((pack.pack_id.clone(), projection, entity));
                }
            }
        }
        let Some((_, first_projection, first_entity)) = projections
            .first()
            .map(|(pack, projection, entity)| (pack, projection, entity.clone()))
        else {
            return Err(CollectionQueryError::Query(
                DictionaryQueryError::InvalidReference {
                    detail: format!(
                        "local {role} identity {id} is not present in retained registrations"
                    ),
                },
            ));
        };
        let identity = format!("{role}:{id}");
        let first_body = first_projection.body_sha256.get(&identity).cloned();
        let mut rows = Vec::with_capacity(projections.len());
        for (pack_id, projection, entity) in projections {
            if entity != first_entity
                || projection.body_sha256.get(&identity).cloned() != first_body
            {
                return Err(CollectionQueryError::AmbiguousReference);
            }
            let native = entity.occurrence.as_ref().and_then(|occurrence| {
                projection.registry.occurrences.iter().find(|row| {
                    row.occurrence.snapshot().namespace() == occurrence.capture.namespace
                        && row.occurrence.snapshot().sha256() == occurrence.capture.sha256
                        && row.occurrence.locator() == occurrence.locator
                })
            });
            rows.push(CollectionReferenceMatch {
                pack_id: pack_id.clone(),
                release_sha256: projection.release_sha256,
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
                native_entry_id: native.and_then(|row| row.native_entry_id.clone()),
                native_assertion_id: native.and_then(|row| row.native_assertion_id.clone()),
                contributing_pack_ids: vec![pack_id],
            });
        }
        let mut groups: Vec<(usize, BTreeSet<String>, Option<usize>)> = Vec::new();
        for (index, row) in rows.iter().enumerate() {
            let row_is_current = self.base.generation.as_ref().is_some_and(|generation| {
                generation.members.iter().any(|member| {
                    member.pack_id == row.pack_id && member.release_sha256 == row.release_sha256
                })
            });
            let same_identity = |existing: &CollectionReferenceMatch| {
                existing.source_id == row.source_id
                    && existing.role == row.role
                    && existing.entity_id == row.entity_id
                    && existing.occurrence == row.occurrence
                    && existing.capture == row.capture
                    && existing.native_entry_id == row.native_entry_id
                    && existing.native_assertion_id == row.native_assertion_id
            };
            if let Some((_, contributors, current)) = groups
                .iter_mut()
                .find(|(first, _, _)| same_identity(&rows[*first]))
            {
                contributors.insert(row.pack_id.clone());
                if row_is_current {
                    *current = Some(index);
                }
            } else {
                groups.push((
                    index,
                    row.contributing_pack_ids.iter().cloned().collect(),
                    row_is_current.then_some(index),
                ));
            }
        }
        let public = super::measure_collection_projection(&mut profile, || {
            groups
                .iter()
                .map(|(first, contributors, current)| {
                    let mut row = rows[*first].clone();
                    if let Some(current) = current {
                        row.pack_id = rows[*current].pack_id.clone();
                        row.release_sha256 = rows[*current].release_sha256.clone();
                    }
                    row.contributing_pack_ids = contributors.iter().cloned().collect();
                    row
                })
                .collect()
        });
        Ok(super::MetadataMatches {
            public,
            admission: rows,
            requested_release: None,
        })
    }

    fn resolve_compact(
        &mut self,
        target: RelationTarget,
        mut profile: Option<&mut CollectionProjectionProfile>,
    ) -> Result<CollectionCompactReferenceResponse, CollectionQueryError> {
        if let RelationTarget::Unresolved(text) = &target
            && text.is_empty()
        {
            return Err(CollectionQueryError::Query(
                DictionaryQueryError::InvalidReference {
                    detail: "Unresolved target must be nonempty".into(),
                },
            ));
        }
        let metadata = match target {
            RelationTarget::Unknown | RelationTarget::Unresolved(_) => None,
            _ => Some(self.compact_metadata_matches(&target, profile.as_deref_mut())?),
        };
        let payload = if let Some(metadata) = &metadata {
            if self
                .base
                .selected_payload_for_target(&target, &metadata.admission)
                .is_some()
            {
                if let Some(profile) = profile.as_mut() {
                    self.ensure_compact(&mut profile.payload_setup)?;
                } else {
                    self.ensure_compact(&mut None)?;
                }
                let engine = self.compact.as_ref().expect("compact engine initialized");
                let result = if let Some(profile) = profile.as_mut() {
                    let (result, query_profile) =
                        engine.execute_profiled(crate::DictionaryRequest::Resolve {
                            target: target.clone(),
                        })?;
                    profile.query.candidate_sql += query_profile.candidate_sql;
                    profile.query.projection += query_profile.projection;
                    result
                } else {
                    engine.execute(crate::DictionaryRequest::Resolve {
                        target: target.clone(),
                    })?
                };
                match result {
                    crate::DictionaryCompactQueryResult::Resolve(response) => Some(*response),
                    crate::DictionaryCompactQueryResult::Exact(_) => {
                        return Err(CollectionQueryError::Query(
                            DictionaryQueryError::Integrity {
                                detail: "reference compact execution returned an exact result"
                                    .into(),
                            },
                        ));
                    }
                }
            } else {
                None
            }
        } else {
            None
        };
        let operational_reason = if payload.is_none() {
            metadata.as_ref().and_then(|matches| {
                matches.admission.iter().find_map(|matched| {
                    self.base
                        .operational_reason(&matched.pack_id, &matched.release_sha256)
                })
            })
        } else {
            None
        };
        let (catalogue_matches, requested_release) = metadata.map_or_else(
            || (Vec::new(), None),
            |value| (value.public, value.requested_release),
        );
        let status = payload.as_ref().map_or_else(
            || match &target {
                RelationTarget::Unknown => crate::ReferenceStatus::Unknown,
                RelationTarget::Unresolved(_) => crate::ReferenceStatus::Unresolved,
                _ if catalogue_matches.is_empty() => crate::ReferenceStatus::NoMatch,
                RelationTarget::Catalog(catalog)
                    if catalogue_matches.iter().any(|row| {
                        catalog
                            .native_entry_id
                            .as_ref()
                            .is_some_and(|id| row.native_entry_id.as_ref() != Some(id))
                            || catalog
                                .native_assertion_id
                                .as_ref()
                                .is_some_and(|id| row.native_assertion_id.as_ref() != Some(id))
                    }) =>
                {
                    crate::ReferenceStatus::Unverified
                }
                _ => crate::ReferenceStatus::CatalogueOnly,
            },
            |value| value.status,
        );
        Ok(CollectionCompactReferenceResponse {
            target,
            status,
            operational_reason,
            payload,
            catalogue_matches,
            requested_release,
        })
    }

    fn response(&self, result: CollectionCompactQueryResult) -> CollectionCompactQueryResponse {
        CollectionCompactQueryResponse {
            version: 1,
            snapshot: self.base.snapshot.clone(),
            collections: self.base.snapshot.collections.clone(),
            result,
        }
    }

    fn wrap_managed_cursor(&self, inner: &str) -> Result<String, CollectionQueryError> {
        #[derive(Serialize)]
        struct Cursor<'a> {
            version: u32,
            snapshot_binding: &'a str,
            inner: &'a str,
        }
        let bytes = serde_json::to_vec(&Cursor {
            version: 1,
            snapshot_binding: &self.base.snapshot.binding_sha256,
            inner,
        })
        .map_err(|error| {
            CollectionQueryError::Query(DictionaryQueryError::Integrity {
                detail: format!("compact cursor serialization failed: {error}"),
            })
        })?;
        let hex = bytes
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        let digest = Sha256::digest(&bytes);
        Ok(format!(
            "collection-compact-v1:{hex}:{}",
            digest
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>()
        ))
    }

    fn unwrap_managed_cursor(&self, cursor: &str) -> Result<String, CollectionQueryError> {
        let Some(rest) = cursor.strip_prefix("collection-compact-v1:") else {
            return Err(if is_valid_full_managed_cursor(cursor) {
                CollectionQueryError::StaleCursor
            } else {
                CollectionQueryError::MalformedCursor
            });
        };
        let Some((hex, digest)) = rest.rsplit_once(':') else {
            return Err(CollectionQueryError::MalformedCursor);
        };
        if hex.is_empty()
            || hex.len() % 2 != 0
            || !hex
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
            || digest.len() != 64
            || !digest
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        {
            return Err(CollectionQueryError::MalformedCursor);
        }
        let bytes = (0..hex.len())
            .step_by(2)
            .map(|index| u8::from_str_radix(&hex[index..index + 2], 16))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| CollectionQueryError::MalformedCursor)?;
        let actual = Sha256::digest(&bytes);
        let actual = actual
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        if actual != digest {
            return Err(CollectionQueryError::MalformedCursor);
        }
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Cursor {
            version: u32,
            snapshot_binding: String,
            inner: String,
        }
        let parsed: Cursor =
            serde_json::from_slice(&bytes).map_err(|_| CollectionQueryError::MalformedCursor)?;
        if parsed.version != 1
            || parsed.inner.is_empty()
            || !parsed.inner.starts_with("dictionary-compact-v1:")
        {
            return Err(CollectionQueryError::MalformedCursor);
        }
        if parsed.snapshot_binding != self.base.snapshot.binding_sha256 {
            return Err(CollectionQueryError::StaleCursor);
        }
        Ok(parsed.inner)
    }
}

pub(super) fn is_valid_compact_managed_cursor(cursor: &str) -> bool {
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct ManagedCursor {
        version: u32,
        snapshot_binding: String,
        inner: String,
    }
    #[derive(Deserialize, Serialize)]
    #[serde(deny_unknown_fields)]
    struct CompactBody {
        version: u32,
        binding: String,
        last: CompactKey,
    }
    #[derive(Deserialize, Serialize)]
    #[serde(deny_unknown_fields)]
    struct CompactKey {
        source: vlajmaji_lexicon::SourceId,
        spelling: String,
        occurrence_id: String,
        assertion_id: String,
    }
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct CompactEnvelope {
        body: CompactBody,
        checksum: String,
    }

    let Some(rest) = cursor.strip_prefix("collection-compact-v1:") else {
        return false;
    };
    let Some((encoded, checksum)) = rest.rsplit_once(':') else {
        return false;
    };
    if !lower_hex(encoded) || encoded.is_empty() || !valid_sha256(checksum) {
        return false;
    }
    let Some(bytes) = decode_compact_cursor_hex(encoded) else {
        return false;
    };
    if format!("{:x}", Sha256::digest(&bytes)) != checksum {
        return false;
    }
    let Ok(managed) = serde_json::from_slice::<ManagedCursor>(&bytes) else {
        return false;
    };
    if managed.version != 1
        || !valid_sha256(&managed.snapshot_binding)
        || !managed.inner.starts_with("dictionary-compact-v1:")
    {
        return false;
    }
    let Some(inner_hex) = managed.inner.strip_prefix("dictionary-compact-v1:") else {
        return false;
    };
    if !lower_hex(inner_hex) || inner_hex.is_empty() {
        return false;
    }
    let Some(inner_bytes) = decode_compact_cursor_hex(inner_hex) else {
        return false;
    };
    let Ok(compact) = serde_json::from_slice::<CompactEnvelope>(&inner_bytes) else {
        return false;
    };
    let Ok(body) = serde_json::to_vec(&compact.body) else {
        return false;
    };
    compact.body.version == 1
        && valid_sha256(&compact.body.binding)
        && valid_sha256(&compact.checksum)
        && !compact.body.last.spelling.is_empty()
        && !compact.body.last.assertion_id.is_empty()
        && valid_occurrence_hex(&compact.body.last.occurrence_id)
        && format!("{:x}", Sha256::digest(body)) == compact.checksum
}

fn decode_compact_cursor_hex(encoded: &str) -> Option<Vec<u8>> {
    if encoded.is_empty() || !encoded.len().is_multiple_of(2) || !lower_hex(encoded) {
        return None;
    }
    let (pairs, remainder) = encoded.as_bytes().as_chunks::<2>();
    if !remainder.is_empty() {
        return None;
    }
    pairs
        .iter()
        .map(|pair| {
            let high = (pair[0] as char).to_digit(16)? as u8;
            let low = (pair[1] as char).to_digit(16)? as u8;
            Some((high << 4) | low)
        })
        .collect()
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && lower_hex(value)
}

fn valid_occurrence_hex(value: &str) -> bool {
    !value.is_empty() && value.len().is_multiple_of(2) && lower_hex(value)
}

fn coalesce_detail_matches(
    matches: &[CollectionReferenceMatch],
    generation: Option<&vlajmaji_store::CollectionGeneration>,
) -> Vec<CollectionReferenceMatch> {
    let mut public: Vec<CollectionReferenceMatch> = Vec::new();
    for row in matches {
        let same_identity = |existing: &CollectionReferenceMatch| {
            existing.source_id == row.source_id
                && existing.role == row.role
                && existing.entity_id == row.entity_id
                && existing.occurrence == row.occurrence
                && existing.capture == row.capture
                && existing.native_entry_id == row.native_entry_id
                && existing.native_assertion_id == row.native_assertion_id
        };
        if let Some(existing) = public.iter_mut().find(|existing| same_identity(existing)) {
            for pack_id in &row.contributing_pack_ids {
                if !existing.contributing_pack_ids.contains(pack_id) {
                    existing.contributing_pack_ids.push(pack_id.clone());
                }
            }
            existing.contributing_pack_ids.sort();
            if generation.is_some_and(|generation| {
                generation.members.iter().any(|member| {
                    member.pack_id == row.pack_id && member.release_sha256 == row.release_sha256
                })
            }) {
                existing.pack_id = row.pack_id.clone();
                existing.release_sha256 = row.release_sha256.clone();
            }
        } else {
            public.push(row.clone());
        }
    }
    public
}

fn is_valid_full_managed_cursor(cursor: &str) -> bool {
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Managed {
        version: u32,
        snapshot_binding: String,
        inner: String,
    }
    #[derive(Deserialize, Serialize)]
    #[serde(deny_unknown_fields)]
    struct Body {
        version: u32,
        binding: String,
        last: Last,
    }
    #[derive(Deserialize, Serialize)]
    #[serde(deny_unknown_fields)]
    struct Last {
        source: String,
        spelling: String,
        occurrence_id: String,
    }
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Envelope {
        body: Body,
        checksum: String,
    }
    let Some(rest) = cursor.strip_prefix("collection-v1:") else {
        return false;
    };
    let Some((hex, digest)) = rest.split_once(':') else {
        return false;
    };
    if hex.is_empty()
        || hex.len() % 2 != 0
        || !lower_hex(hex)
        || digest.len() != 64
        || !lower_hex(digest)
    {
        return false;
    }
    let Ok(bytes) = decode_lower_hex(hex) else {
        return false;
    };
    if format!("{:x}", Sha256::digest(&bytes)) != digest {
        return false;
    }
    let Ok(managed) = serde_json::from_slice::<Managed>(&bytes) else {
        return false;
    };
    if managed.version != 1
        || managed.snapshot_binding.len() != 64
        || !lower_hex(&managed.snapshot_binding)
    {
        return false;
    }
    let Ok(inner) = decode_lower_hex(&managed.inner) else {
        return false;
    };
    let Ok(envelope) = serde_json::from_slice::<Envelope>(&inner) else {
        return false;
    };
    if envelope.body.version != 1
        || envelope.body.binding.len() != 64
        || !lower_hex(&envelope.body.binding)
        || envelope.checksum.len() != 64
        || !lower_hex(&envelope.checksum)
    {
        return false;
    }
    serde_json::to_vec(&envelope.body)
        .ok()
        .is_some_and(|body| format!("{:x}", Sha256::digest(body)) == envelope.checksum)
}

fn lower_hex(value: &str) -> bool {
    value
        .bytes()
        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn decode_lower_hex(value: &str) -> Result<Vec<u8>, ()> {
    if !value.len().is_multiple_of(2) || !lower_hex(value) {
        return Err(());
    }
    (0..value.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&value[index..index + 2], 16).map_err(|_| ()))
        .collect()
}
