//! Borrowed compact selection over the typed, unverified document.

use std::collections::{BTreeMap, BTreeSet};

use vlajmaji_lexicon as lexicon;
use vlajmaji_lexicon::{EntityId, SourceId};

use super::{DictionaryQueryError, projection::DictionaryCompactSourceAvailability};

pub(super) type CompactSelection = BTreeMap<SourceId, BTreeSet<String>>;
pub(super) type CompactViews = (Vec<DictionaryCompactSourceAvailability>, CompactSelection);

pub(super) fn source_views(
    document: &lexicon::UnverifiedDictionaryDocument,
    overlay: Option<&[EntityId]>,
) -> Result<CompactViews, DictionaryQueryError> {
    let registry = &document.registry;
    let mut sources = registry.sources.clone();
    sources.sort_by(|left, right| left.id.cmp(&right.id));
    let mut revisions = BTreeMap::new();
    for revision in &document.metadata.owned_revisions {
        if revisions.insert(revision.id.clone(), revision).is_some() {
            return Err(DictionaryQueryError::Integrity {
                detail: "duplicate owned revision identity in query index".into(),
            });
        }
    }
    let mut views = Vec::new();
    let mut selected = BTreeMap::new();
    for source in sources {
        let collections: Vec<_> = document
            .metadata
            .collections
            .iter()
            .filter(|row| row.source == source.id)
            .cloned()
            .collect();
        let collection_ids: BTreeSet<_> = collections.iter().map(|row| row.id.clone()).collect();
        let components: Vec<_> = document
            .metadata
            .components
            .iter()
            .filter(|row| collection_ids.contains(&row.collection))
            .cloned()
            .collect();
        let component_ids: BTreeSet<_> = components.iter().map(|row| row.id.clone()).collect();
        let releases: Vec<_> = document
            .metadata
            .releases
            .iter()
            .filter(|row| component_ids.contains(&row.component))
            .collect();
        let selected_release_ids: Vec<_> = releases
            .iter()
            .filter(|row| {
                row.selected
                    && row.availability == lexicon::Availability::Available
                    && overlay.is_none_or(|ids| ids.contains(&row.id))
            })
            .map(|row| row.id.clone())
            .collect();
        let release_set: BTreeSet<_> = selected_release_ids.iter().cloned().collect();
        let release_assertions: BTreeSet<String> = releases
            .iter()
            .filter(|release| release_set.contains(&release.id))
            .flat_map(|release| release.assertions.iter().map(ToString::to_string))
            .collect();
        let owned_head = document
            .metadata
            .owned_heads
            .iter()
            .find(|head| head.source == source.id)
            .cloned();
        let mut seen_captures = BTreeSet::new();
        let mut loaded_captures = Vec::new();
        let mut missing_captures = Vec::new();
        for capture in releases.iter().flat_map(|release| &release.captures) {
            if !seen_captures.insert(capture.clone()) {
                continue;
            }
            if document.captures.iter().any(|row| &row.identity == capture) {
                loaded_captures.push(capture.clone());
            } else {
                missing_captures.push(capture.clone());
            }
        }
        let reason = if releases.is_empty() {
            Some(super::SourceSelectionFailure::NoReleases)
        } else if selected_release_ids.is_empty() {
            Some(super::SourceSelectionFailure::NoSelectedAvailableRelease)
        } else if source.origin == lexicon::SourceOrigin::OwnedDraft && owned_head.is_none() {
            Some(super::SourceSelectionFailure::NoOwnedHead)
        } else {
            None
        };
        let ids = if source.origin == lexicon::SourceOrigin::OwnedDraft {
            owned_head
                .as_ref()
                .and_then(|head| revisions.get(&head.revision))
                .map(|revision| {
                    let revision_ids: BTreeSet<_> = revision
                        .assertions
                        .iter()
                        .map(ToString::to_string)
                        .collect();
                    release_assertions
                        .intersection(&revision_ids)
                        .cloned()
                        .collect()
                })
                .unwrap_or_default()
        } else {
            release_assertions
        };
        selected.insert(source.id.clone(), ids);
        let summary = releases
            .iter()
            .map(|release| vlajmaji_store::CollectionReleaseSummary {
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
            .collect();
        views.push(DictionaryCompactSourceAvailability {
            aliases: registry
                .aliases
                .iter()
                .filter(|row| row.source == source.id)
                .cloned()
                .collect(),
            lineage: registry
                .lineage
                .iter()
                .filter(|row| row.source == source.id)
                .cloned()
                .collect(),
            source,
            collections,
            components,
            releases: summary,
            selected_available_release_ids: selected_release_ids,
            loaded_captures,
            missing_captures,
            owned_head,
            selectable: reason.is_none(),
            reason,
        });
    }
    Ok((views, selected))
}
