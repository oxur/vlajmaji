//! Borrowed compact selection over the typed, unverified document.

use std::collections::{BTreeMap, BTreeSet};

use vlajmaji_lexicon as lexicon;
use vlajmaji_lexicon::{CaptureRef, EntityId, SourceId};

use super::{DictionaryQueryError, projection::DictionaryCompactSourceAvailability};

pub(super) type CompactSelection = BTreeMap<SourceId, BTreeSet<String>>;
pub(super) type CompactViews = (Vec<DictionaryCompactSourceAvailability>, CompactSelection);

/// Compact identity indexes retain ordinals, not cloned dictionary bodies.
pub(super) struct CompactIndices {
    pub captures: BTreeMap<CaptureRef, usize>,
    pub occurrences: BTreeMap<(CaptureRef, String), (usize, usize)>,
    pub sources: BTreeMap<SourceId, usize>,
    pub entries: BTreeMap<String, usize>,
    pub assertions: BTreeMap<String, usize>,
    pub forms: BTreeMap<String, usize>,
    pub editorial_senses: BTreeMap<String, usize>,
    pub snapshots: BTreeMap<String, usize>,
    pub occurrence_attributions: BTreeMap<String, usize>,
    pub contributors: BTreeMap<String, usize>,
    pub distributors: BTreeMap<String, usize>,
}

pub(super) fn index_document(
    document: &lexicon::UnverifiedDictionaryDocument,
    overlay: Option<&[EntityId]>,
) -> Result<(CompactIndices, CompactViews), DictionaryQueryError> {
    let captures = super::unique_map(
        document
            .captures
            .iter()
            .enumerate()
            .map(|(ordinal, row)| (row.identity.clone(), ordinal)),
        "capture",
    )?;
    let occurrences =
        super::unique_map(
            document
                .captures
                .iter()
                .enumerate()
                .flat_map(|(capture_index, capture)| {
                    capture.occurrences.iter().enumerate().map(
                        move |(occurrence_index, occurrence)| {
                            (
                                (capture.identity.clone(), occurrence.locator.clone()),
                                (capture_index, occurrence_index),
                            )
                        },
                    )
                }),
            "capture occurrence",
        )?;
    let entries = super::unique_map(
        document
            .metadata
            .entries
            .iter()
            .enumerate()
            .map(|(ordinal, row)| (row.id.to_string(), ordinal)),
        "entry",
    )?;
    let assertions = super::unique_map(
        document
            .metadata
            .assertions
            .iter()
            .enumerate()
            .map(|(ordinal, row)| (row.id.to_string(), ordinal)),
        "assertion",
    )?;
    let sources = super::unique_map(
        document
            .registry
            .sources
            .iter()
            .enumerate()
            .map(|(ordinal, row)| (row.id.clone(), ordinal)),
        "source",
    )?;

    if let Some(ids) = overlay {
        let known: BTreeSet<_> = document
            .metadata
            .releases
            .iter()
            .map(|row| row.id.clone())
            .collect();
        if let Some(unknown) = ids.iter().find(|id| !known.contains(*id)) {
            return Err(DictionaryQueryError::Integrity {
                detail: format!("snapshot selected unknown release {unknown}"),
            });
        }
    }

    // Selection, known-overlay validation and owned-revision validation occur
    // after the first five identity families and before these remaining indexes.
    let views = source_views(document, overlay)?;

    let forms = super::unique_map(
        document
            .metadata
            .forms
            .iter()
            .enumerate()
            .map(|(ordinal, row)| (row.id.to_string(), ordinal)),
        "form",
    )?;
    let editorial_senses = super::unique_map(
        document
            .metadata
            .editorial_senses
            .iter()
            .enumerate()
            .map(|(ordinal, row)| (row.id.to_string(), ordinal)),
        "editorial sense",
    )?;
    let _computations = super::unique_map(
        document
            .metadata
            .computations
            .iter()
            .enumerate()
            .map(|(ordinal, row)| (row.id.to_string(), ordinal)),
        "computation",
    )?;
    let snapshots = super::unique_map(
        document
            .registry
            .snapshots
            .iter()
            .enumerate()
            .map(|(ordinal, row)| (super::snapshot_key(row), ordinal)),
        "snapshot attribution",
    )?;
    let occurrence_attributions = super::unique_map(
        document
            .registry
            .occurrences
            .iter()
            .enumerate()
            .map(|(ordinal, row)| (super::record_key(&row.occurrence), ordinal)),
        "occurrence attribution",
    )?;
    let contributors = super::unique_map(
        document
            .registry
            .contributors
            .iter()
            .enumerate()
            .map(|(ordinal, row)| (row.id.to_string(), ordinal)),
        "contributor",
    )?;
    let distributors = super::unique_map(
        document
            .registry
            .distributors
            .iter()
            .enumerate()
            .map(|(ordinal, row)| (row.id.to_string(), ordinal)),
        "distributor",
    )?;

    Ok((
        CompactIndices {
            captures,
            occurrences,
            sources,
            entries,
            assertions,
            forms,
            editorial_senses,
            snapshots,
            occurrence_attributions,
            contributors,
            distributors,
        },
        views,
    ))
}

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

#[cfg(test)]
mod identity_tests {
    use super::super::{DictionaryQueryError, DictionaryQueryProvider, DictionarySearchEngine};
    use super::index_document;
    use std::{
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };
    use vlajmaji_lexicon::{EntityId, UnverifiedDictionaryDocument};
    use vlajmaji_store::DictionaryStore;

    mod fixture {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../lexicon/examples/support/dictionary_fixture.rs"
        ));
    }

    fn scratch() -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "vlajmaji-search-duplicate-index-{}-{stamp}",
            std::process::id()
        ));
        std::fs::create_dir(&path).unwrap();
        path
    }

    #[test]
    fn test_from_provider_rejects_each_duplicate_identity_family_in_order() {
        let dictionary = fixture::complete_fixture().expect("complete fixture is valid");
        let root = scratch();
        let store_path = root.join("dictionary.sqlite3");
        DictionaryStore::create(&store_path, &dictionary).unwrap();
        let bytes = dictionary.to_json_bytes().unwrap();
        let original = UnverifiedDictionaryDocument::from_unverified_json_bytes(&bytes).unwrap();

        let provider =
            || DictionaryQueryProvider::Store(DictionaryStore::open(&store_path).unwrap());
        DictionarySearchEngine::from_provider(provider(), original.clone())
            .expect("the unchanged document has unique indexes");

        macro_rules! rejects_duplicate {
            ($role:literal, $mutate:expr) => {{
                let mut document = original.clone();
                ($mutate)(&mut document);
                let error = DictionarySearchEngine::from_provider(provider(), document)
                    .expect_err("duplicate identity must be rejected");
                assert!(matches!(error, DictionaryQueryError::Integrity { ref detail }
                    if detail == &format!("duplicate {} identity in query index", $role)));
            }};
        }

        rejects_duplicate!("capture", |document: &mut UnverifiedDictionaryDocument| {
            document.captures.push(document.captures[0].clone());
        });
        rejects_duplicate!(
            "capture occurrence",
            |document: &mut UnverifiedDictionaryDocument| {
                let occurrence = document.captures[0].occurrences[0].clone();
                document.captures[0].occurrences.push(occurrence);
            }
        );
        rejects_duplicate!("entry", |document: &mut UnverifiedDictionaryDocument| {
            document
                .metadata
                .entries
                .push(document.metadata.entries[0].clone());
        });
        rejects_duplicate!(
            "assertion",
            |document: &mut UnverifiedDictionaryDocument| {
                document
                    .metadata
                    .assertions
                    .push(document.metadata.assertions[0].clone());
            }
        );
        rejects_duplicate!("source", |document: &mut UnverifiedDictionaryDocument| {
            document
                .registry
                .sources
                .push(document.registry.sources[0].clone());
        });
        rejects_duplicate!("form", |document: &mut UnverifiedDictionaryDocument| {
            document
                .metadata
                .forms
                .push(document.metadata.forms[0].clone());
        });
        rejects_duplicate!(
            "editorial sense",
            |document: &mut UnverifiedDictionaryDocument| {
                document
                    .metadata
                    .editorial_senses
                    .push(document.metadata.editorial_senses[0].clone());
            }
        );
        rejects_duplicate!(
            "computation",
            |document: &mut UnverifiedDictionaryDocument| {
                document
                    .metadata
                    .computations
                    .push(document.metadata.computations[0].clone());
            }
        );
        rejects_duplicate!(
            "snapshot attribution",
            |document: &mut UnverifiedDictionaryDocument| {
                document
                    .registry
                    .snapshots
                    .push(document.registry.snapshots[0].clone());
            }
        );
        rejects_duplicate!(
            "occurrence attribution",
            |document: &mut UnverifiedDictionaryDocument| {
                document
                    .registry
                    .occurrences
                    .push(document.registry.occurrences[0].clone());
            }
        );
        rejects_duplicate!(
            "contributor",
            |document: &mut UnverifiedDictionaryDocument| {
                document
                    .registry
                    .contributors
                    .push(document.registry.contributors[0].clone());
            }
        );
        rejects_duplicate!(
            "distributor",
            |document: &mut UnverifiedDictionaryDocument| {
                document
                    .registry
                    .distributors
                    .push(document.registry.distributors[0].clone());
            }
        );

        let mut capture_before_source = original.clone();
        capture_before_source
            .captures
            .push(capture_before_source.captures[0].clone());
        capture_before_source
            .registry
            .sources
            .push(capture_before_source.registry.sources[0].clone());
        let error = DictionarySearchEngine::from_provider(provider(), capture_before_source)
            .expect_err("capture identity precedes source identity");
        assert!(
            matches!(error, DictionaryQueryError::Integrity { ref detail }
            if detail == "duplicate capture identity in query index")
        );

        let mut source_before_form = original;
        source_before_form
            .registry
            .sources
            .push(source_before_form.registry.sources[0].clone());
        source_before_form
            .metadata
            .forms
            .push(source_before_form.metadata.forms[0].clone());
        let error = DictionarySearchEngine::from_provider(provider(), source_before_form)
            .expect_err("source identity precedes Form identity");
        assert!(
            matches!(error, DictionaryQueryError::Integrity { ref detail }
            if detail == "duplicate source identity in query index")
        );

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn test_index_document_preserves_duplicate_order_and_overlay_boundary() {
        let dictionary = fixture::complete_fixture().expect("complete fixture is valid");
        let bytes = dictionary.to_json_bytes().unwrap();
        let original = UnverifiedDictionaryDocument::from_unverified_json_bytes(&bytes).unwrap();
        index_document(&original, None).expect("unchanged compact input has unique identities");

        macro_rules! rejects_duplicate {
            ($role:literal, $mutate:expr) => {{
                let mut document = original.clone();
                ($mutate)(&mut document);
                let error = match index_document(&document, None) {
                    Err(error) => error,
                    Ok(_) => panic!("compact index accepted duplicate {} identity", $role),
                };
                assert!(matches!(error, DictionaryQueryError::Integrity { ref detail }
                    if detail == &format!("duplicate {} identity in query index", $role)));
            }};
        }

        rejects_duplicate!("capture", |document: &mut UnverifiedDictionaryDocument| {
            document.captures.push(document.captures[0].clone());
        });
        rejects_duplicate!(
            "capture occurrence",
            |document: &mut UnverifiedDictionaryDocument| {
                let occurrence = document.captures[0].occurrences[0].clone();
                document.captures[0].occurrences.push(occurrence);
            }
        );
        rejects_duplicate!("entry", |document: &mut UnverifiedDictionaryDocument| {
            document
                .metadata
                .entries
                .push(document.metadata.entries[0].clone());
        });
        rejects_duplicate!(
            "assertion",
            |document: &mut UnverifiedDictionaryDocument| {
                document
                    .metadata
                    .assertions
                    .push(document.metadata.assertions[0].clone());
            }
        );
        rejects_duplicate!("source", |document: &mut UnverifiedDictionaryDocument| {
            document
                .registry
                .sources
                .push(document.registry.sources[0].clone());
        });
        rejects_duplicate!("form", |document: &mut UnverifiedDictionaryDocument| {
            document
                .metadata
                .forms
                .push(document.metadata.forms[0].clone());
        });
        rejects_duplicate!(
            "editorial sense",
            |document: &mut UnverifiedDictionaryDocument| {
                document
                    .metadata
                    .editorial_senses
                    .push(document.metadata.editorial_senses[0].clone());
            }
        );
        rejects_duplicate!(
            "computation",
            |document: &mut UnverifiedDictionaryDocument| {
                document
                    .metadata
                    .computations
                    .push(document.metadata.computations[0].clone());
            }
        );
        rejects_duplicate!(
            "snapshot attribution",
            |document: &mut UnverifiedDictionaryDocument| {
                document
                    .registry
                    .snapshots
                    .push(document.registry.snapshots[0].clone());
            }
        );
        rejects_duplicate!(
            "occurrence attribution",
            |document: &mut UnverifiedDictionaryDocument| {
                document
                    .registry
                    .occurrences
                    .push(document.registry.occurrences[0].clone());
            }
        );
        rejects_duplicate!(
            "contributor",
            |document: &mut UnverifiedDictionaryDocument| {
                document
                    .registry
                    .contributors
                    .push(document.registry.contributors[0].clone());
            }
        );
        rejects_duplicate!(
            "distributor",
            |document: &mut UnverifiedDictionaryDocument| {
                document
                    .registry
                    .distributors
                    .push(document.registry.distributors[0].clone());
            }
        );
        rejects_duplicate!(
            "owned revision",
            |document: &mut UnverifiedDictionaryDocument| {
                document
                    .metadata
                    .owned_revisions
                    .push(document.metadata.owned_revisions[0].clone());
            }
        );

        let unknown = [EntityId::new("unknown-release").unwrap()];
        let mut capture_before_overlay = original.clone();
        capture_before_overlay
            .captures
            .push(capture_before_overlay.captures[0].clone());
        assert!(matches!(
            index_document(&capture_before_overlay, Some(&unknown)),
            Err(DictionaryQueryError::Integrity { ref detail })
                if detail == "duplicate capture identity in query index"
        ));

        assert!(matches!(
            index_document(&original, Some(&unknown)),
            Err(DictionaryQueryError::Integrity { ref detail })
                if detail == "snapshot selected unknown release unknown-release"
        ));

        let mut capture_before_source = original.clone();
        capture_before_source
            .captures
            .push(capture_before_source.captures[0].clone());
        capture_before_source
            .registry
            .sources
            .push(capture_before_source.registry.sources[0].clone());
        assert!(matches!(
            index_document(&capture_before_source, None),
            Err(DictionaryQueryError::Integrity { ref detail })
                if detail == "duplicate capture identity in query index"
        ));

        let mut source_before_form = original.clone();
        source_before_form
            .registry
            .sources
            .push(source_before_form.registry.sources[0].clone());
        source_before_form
            .metadata
            .forms
            .push(source_before_form.metadata.forms[0].clone());
        assert!(matches!(
            index_document(&source_before_form, None),
            Err(DictionaryQueryError::Integrity { ref detail })
                if detail == "duplicate source identity in query index"
        ));

        let known = original
            .metadata
            .releases
            .iter()
            .find(|release| {
                release.selected
                    && release.availability == vlajmaji_lexicon::Availability::Available
            })
            .expect("fixture has an available selected release")
            .id
            .clone();
        index_document(&original, Some(std::slice::from_ref(&known)))
            .expect("a known overlay must be accepted");

        let mut overlay_before_owned_revision = original;
        overlay_before_owned_revision
            .metadata
            .owned_revisions
            .push(overlay_before_owned_revision.metadata.owned_revisions[0].clone());
        assert!(matches!(
            index_document(&overlay_before_owned_revision, Some(&unknown)),
            Err(DictionaryQueryError::Integrity { ref detail })
                if detail == "snapshot selected unknown release unknown-release"
        ));
    }
}
