//! Computed lexical and caller-proposal query result types.
//!
//! Implementations share the guarded workbench providers and keep derived
//! analyses and caller-owned proposals separate from captured source rows.

use serde::{Deserialize, Serialize};
use std::str::FromStr;
use vlajmaji_lexicon::{
    self as lexicon, AliasScope, CaptureMetadata, CodeNotice, EntityId, Proposal, SourceId,
};

use crate::{ComputedValue, DictionaryCompactHit, DictionaryHit, DistancePosition, Nucleus};

/// Caller-authored proposal input, validated by the canonical lexicon constructor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkbenchProposalInput {
    /// Caller namespace; whitespace is preserved exactly.
    pub namespace: String,
    /// Exact proposed spelling; empty text is permitted by the constructor.
    pub spelling: String,
    /// Exact proposal text; empty text is permitted by the constructor.
    pub text: String,
}

/// Computed pre-page occurrence and spelling counts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkbenchComputedTotals {
    /// Always `assertion_occurrences` for source families.
    pub unit: String,
    /// Selected/current/release-and-head-eligible lexical assertion occurrences.
    pub candidate_assertions: usize,
    /// Distinct exact Value spellings in the candidate cohort.
    pub candidate_distinct_spellings: usize,
    /// Family matches before paging.
    pub records: usize,
    /// Distinct exact Value spellings among family matches.
    pub distinct_spellings: usize,
    /// Sources with at least one family match.
    pub source_groups: usize,
}

/// One matching source's complete pre-page computed counts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkbenchComputedSourceTotal {
    /// Complete retained source record.
    pub source: lexicon::SourceRecord,
    /// Matching assertion occurrences.
    pub records: usize,
    /// Distinct exact Value spellings among matches.
    pub distinct_spellings: usize,
}

/// Actual native class plus any narrowly permitted legacy LLG inference.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkbenchAnalysisClassEvidence {
    /// The exact native presence state from this assertion.
    pub native_word_class: lexicon::Presence<String>,
    /// Operational P01 kind selected for the analysis.
    pub mapped_word_kind: String,
    /// Exact direct-mapping or capture-fallback rule.
    pub rule: String,
    /// Actual loaded LLG capture identity, or required null.
    #[serde(deserialize_with = "super::deserialize_required_option")]
    pub capture: Option<lexicon::CaptureRef>,
    /// Actual capture format label for fallback, or required null.
    #[serde(deserialize_with = "super::deserialize_required_option")]
    pub format_label: Option<String>,
}

/// Phonological evidence derived from one exact spelling and mapped kind.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkbenchAnalysisEvidence {
    /// Operational P01 word kind.
    pub mapped_word_kind: String,
    /// Exact written nuclei from the unchanged P01 implementation.
    pub nuclei: Vec<Nucleus>,
    /// Resolved syllable count, or required null.
    #[serde(deserialize_with = "super::deserialize_required_option")]
    pub syllable_count: Option<usize>,
    /// Default primary nucleus, or required null.
    #[serde(deserialize_with = "super::deserialize_required_option")]
    pub primary_nucleus: Option<usize>,
    /// Actual assigned, known-no-primary, or error state.
    pub stress_status: String,
    /// Named analysis and grouping diagnostics.
    pub diagnostics: Vec<String>,
    /// Actual native presence and per-assertion mapping provenance.
    pub class_evidence: WorkbenchAnalysisClassEvidence,
}

/// One returned Full hit with complete source data and optional analysis.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkbenchComputedFullHit {
    /// Actual complete dictionary hit.
    pub hit: DictionaryHit,
    /// Spelling distance, or required null outside that family.
    #[serde(deserialize_with = "super::deserialize_required_option")]
    pub spelling_distance: Option<usize>,
    /// Analysis evidence, or required null when this family does not analyze.
    #[serde(deserialize_with = "super::deserialize_required_option")]
    pub analysis: Option<WorkbenchAnalysisEvidence>,
    /// Complete positional explanations, used by Contrast.
    pub distance_positions: Vec<DistancePosition>,
    /// Actual engine decomposition for Components, or required null.
    #[serde(deserialize_with = "super::deserialize_required_option")]
    pub computed_components: Option<Vec<String>>,
}

/// One returned Compact hit with actual same-format compact identity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkbenchComputedCompactHit {
    /// Actual compact dictionary hit.
    pub hit: DictionaryCompactHit,
    /// Spelling distance, or required null outside that family.
    #[serde(deserialize_with = "super::deserialize_required_option")]
    pub spelling_distance: Option<usize>,
    /// Analysis evidence, or required null when this family does not analyze.
    #[serde(deserialize_with = "super::deserialize_required_option")]
    pub analysis: Option<WorkbenchAnalysisEvidence>,
    /// Complete positional explanations, used by Contrast.
    pub distance_positions: Vec<DistancePosition>,
    /// Actual engine decomposition for Components, or required null.
    #[serde(deserialize_with = "super::deserialize_required_option")]
    pub computed_components: Option<Vec<String>>,
}

/// Full-format result for one of the six source-backed computed families.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkbenchComputedFull {
    /// Ordered family policy identifiers.
    pub policy_ids: Vec<String>,
    /// Actual dictionary state digest.
    pub state_sha256: String,
    /// The exact validated query.
    pub query: super::lexical::WorkbenchLexicalQuery,
    /// Selected sources in resolved caller order.
    pub resolved_sources: Vec<SourceId>,
    /// Counts computed before page slicing.
    pub totals: WorkbenchComputedTotals,
    /// Complete counts for sources with matches, sorted by SourceId.
    pub source_totals: Vec<WorkbenchComputedSourceTotal>,
    /// Full source availability from the same provider.
    pub availability: Vec<crate::DictionarySourceAvailability>,
    /// Globally ranked returned occurrences.
    pub hits: Vec<WorkbenchComputedFullHit>,
    /// Per-occurrence exclusions at their first applicable stage.
    pub exclusions: std::collections::BTreeMap<String, usize>,
    /// Opaque continuation or required null.
    #[serde(deserialize_with = "super::deserialize_required_option")]
    pub next_cursor: Option<String>,
    /// Complete actual dictionary code notices.
    pub code_notices: Vec<CodeNotice>,
}

/// Compact-format result for one of the six source-backed computed families.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkbenchComputedCompact {
    /// Ordered family policy identifiers.
    pub policy_ids: Vec<String>,
    /// Actual dictionary state digest.
    pub state_sha256: String,
    /// The exact validated query.
    pub query: super::lexical::WorkbenchLexicalQuery,
    /// Selected sources in resolved caller order.
    pub resolved_sources: Vec<SourceId>,
    /// Counts computed before page slicing.
    pub totals: WorkbenchComputedTotals,
    /// Complete counts for sources with matches, sorted by SourceId.
    pub source_totals: Vec<WorkbenchComputedSourceTotal>,
    /// Compact source availability from the same provider.
    pub availability: Vec<crate::DictionaryCompactSourceAvailability>,
    /// Globally ranked returned occurrences.
    pub hits: Vec<WorkbenchComputedCompactHit>,
    /// Per-occurrence exclusions at their first applicable stage.
    pub exclusions: std::collections::BTreeMap<String, usize>,
    /// Opaque continuation or required null.
    #[serde(deserialize_with = "super::deserialize_required_option")]
    pub next_cursor: Option<String>,
    /// Complete actual dictionary code notices.
    pub code_notices: Vec<CodeNotice>,
    /// Genuine snapshot digest of the cached Compact provider.
    pub snapshot_sha256: String,
}

/// Counts for caller-provided Proposal rows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkbenchProposalTotals {
    /// Always `caller_proposals`.
    pub unit: String,
    /// Every constructor-validated proposal input, including nonmatches.
    pub candidate_proposals: usize,
    /// Exact-spelling returned proposal rows, with duplicates retained.
    pub records: usize,
}

/// One actual constructor-created caller proposal with its input ordinal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkbenchProposalRecord {
    /// Actual Proposal constructor identity.
    pub id: String,
    /// Exact caller namespace.
    pub namespace: String,
    /// Exact caller spelling.
    pub spelling: String,
    /// Exact caller text.
    pub text: String,
    /// Zero-based caller position, checked to fit u64.
    pub input_ordinal: u64,
}

/// ProposalExact result, independent of dictionary-provider setup.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkbenchProposalResult {
    /// Ordered proposal policies.
    pub policy_ids: Vec<String>,
    /// The exact validated query.
    pub query: super::lexical::WorkbenchLexicalQuery,
    /// Pre-page proposal counts.
    pub totals: WorkbenchProposalTotals,
    /// Exact returned proposal rows, including duplicate IDs at distinct ordinals.
    pub proposals: Vec<WorkbenchProposalRecord>,
    /// Opaque continuation or required null.
    #[serde(deserialize_with = "super::deserialize_required_option")]
    pub next_cursor: Option<String>,
    /// Empty because proposals have no dictionary-source provenance.
    pub code_notices: Vec<CodeNotice>,
}

/// Generate result with one actual morphology-engine value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkbenchGeneratedResult {
    /// Ordered generation policy identifiers.
    pub policy_ids: Vec<String>,
    /// The exact validated query.
    pub query: super::lexical::WorkbenchLexicalQuery,
    /// One genuine engine result, separate from source membership.
    pub value: ComputedValue,
    /// Empty because a generated value has no dictionary-source provenance.
    pub code_notices: Vec<CodeNotice>,
}

/// A retained source occurrence used by the shared ranking pipeline.
#[derive(Debug, Clone, Copy)]
pub(crate) struct CandidateRef<'a> {
    pub source: &'a lexicon::SourceRecord,
    pub entry: &'a lexicon::EntryRecord,
    pub assertion: &'a lexicon::AssertionRecord,
    pub capture: &'a CaptureMetadata,
}

#[derive(Debug)]
pub(crate) struct ComputedRow<'a> {
    pub candidate: CandidateRef<'a>,
    pub spelling: String,
    pub occurrence_id: String,
    pub distance: Option<usize>,
    pub analysis: Option<WorkbenchAnalysisEvidence>,
    pub distance_positions: Vec<DistancePosition>,
    pub components: Option<Vec<String>>,
}

#[derive(Debug)]
pub(crate) struct ComputedScan<'a> {
    pub policy_ids: Vec<String>,
    pub totals: WorkbenchComputedTotals,
    pub source_totals: Vec<WorkbenchComputedSourceTotal>,
    pub exclusions: std::collections::BTreeMap<String, usize>,
    pub rows: Vec<ComputedRow<'a>>,
}

pub(crate) fn scan_source<'a>(
    query: &super::lexical::WorkbenchLexicalQuery,
    candidates: Vec<CandidateRef<'a>>,
    compose_target_ids: Option<&std::collections::BTreeSet<EntityId>>,
) -> Result<ComputedScan<'a>, super::lexical::WorkbenchLexicalError> {
    use super::lexical::{WorkbenchLexicalError, WorkbenchLexicalQuery};
    use std::collections::{BTreeMap, BTreeSet};

    let candidate_assertions = candidates.len();
    let candidate_distinct_spellings = candidates
        .iter()
        .filter_map(|candidate| match &candidate.entry.spelling {
            lexicon::Presence::Value(value) => Some(value.clone()),
            _ => None,
        })
        .collect::<BTreeSet<_>>()
        .len();
    let mut exclusions = BTreeMap::<String, usize>::new();
    let mut rows = Vec::new();
    let mut policy_ids = vec![
        COMPUTED_ASSERTIONS_POLICY.into(),
        ASSERTION_ORDER_POLICY.into(),
    ];

    let phonology = |spelling: &str, mapped: &str| -> Result<WorkbenchAnalysisEvidence, String> {
        let kind = crate::parse_kind(mapped).map_err(|error| error.to_string())?;
        let evidence = crate::analyze(spelling, kind)?;
        Ok(WorkbenchAnalysisEvidence {
            mapped_word_kind: mapped.to_owned(),
            nuclei: evidence.nuclei,
            syllable_count: evidence.syllables,
            primary_nucleus: evidence.primary,
            stress_status: evidence.stress_status,
            diagnostics: evidence.diagnostics,
            class_evidence: WorkbenchAnalysisClassEvidence {
                native_word_class: lexicon::Presence::Missing,
                mapped_word_kind: mapped.to_owned(),
                rule: String::new(),
                capture: None,
                format_label: None,
            },
        })
    };

    let mapped_for = |candidate: &CandidateRef<'_>| -> Option<(String, String, Option<lexicon::CaptureRef>, Option<String>)> {
        let native = &candidate.entry.native_word_class;
        let direct = match native {
            lexicon::Presence::Value(value) => map_native(value),
            lexicon::Presence::Null | lexicon::Presence::Missing => None,
        };
        if let Some((mapped, suffix)) = direct {
            return Some((
                mapped.into(),
                format!("dictionary-analysis-class-v1/native/{suffix}"),
                None,
                None,
            ));
        }
        if !matches!(native, lexicon::Presence::Missing) {
            return None;
        }
        let (mapped, suffix) = match candidate.capture.format_label.as_str() {
            "llg-gismu-fixed-columns" => map_native("gismu")?,
            "llg-cmavo-fixed-columns" => map_native("cmavo")?,
            _ => return None,
        };
        Some((
            mapped.into(),
            format!(
                "dictionary-analysis-class-v1/capture/{}/{suffix}",
                candidate.capture.format_label
            ),
            Some(candidate.assertion.occurrence.capture.clone()),
            Some(candidate.capture.format_label.clone()),
        ))
    };

    let mut contrast_anchor = None;
    match query {
        WorkbenchLexicalQuery::SpellingDistance { anchor, .. } => {
            if anchor.is_empty() {
                return Err(invalid("spelling-distance anchor must not be empty"));
            }
            policy_ids.push(crate::SPELLING_POLICY.into());
        }
        WorkbenchLexicalQuery::Prosody { kind, .. } => {
            if let Some(kind) = kind {
                crate::parse_kind(kind)
                    .map_err(|error| WorkbenchLexicalError::Computation(Box::new(error)))?;
            }
            policy_ids.extend([
                crate::PHONOLOGY_POLICY.into(),
                "dictionary-analysis-class-v1".into(),
            ]);
        }
        WorkbenchLexicalQuery::Nuclei { kind, .. } => {
            crate::parse_kind(kind)
                .map_err(|error| WorkbenchLexicalError::Computation(Box::new(error)))?;
            policy_ids.extend([
                crate::PHONOLOGY_POLICY.into(),
                "dictionary-analysis-class-v1".into(),
            ]);
        }
        WorkbenchLexicalQuery::Contrast { anchor, kind, .. } => {
            let word_kind = crate::parse_kind(kind)
                .map_err(|error| WorkbenchLexicalError::Computation(Box::new(error)))?;
            let anchor = crate::analyze(anchor, word_kind).map_err(|detail| {
                WorkbenchLexicalError::Computation(Box::new(crate::SearchError::Phonology(detail)))
            })?;
            if anchor.nuclei.iter().any(|nucleus| nucleus.kind != "vowel") {
                return Err(invalid("aligned contrast requires single-vowel nuclei"));
            }
            contrast_anchor = Some(
                anchor
                    .nuclei
                    .iter()
                    .map(|nucleus| nucleus.spelling.clone())
                    .collect::<Vec<_>>(),
            );
            policy_ids.extend([
                crate::PHONOLOGY_POLICY.into(),
                "dictionary-analysis-class-v1".into(),
                "aligned-single-vowel-contrast-v1".into(),
            ]);
        }
        WorkbenchLexicalQuery::Components { components } => {
            #[cfg(not(feature = "morphology"))]
            {
                let _ = components;
                return Err(WorkbenchLexicalError::Computation(Box::new(
                    crate::SearchError::FeatureDisabled("morphology"),
                )));
            }
            #[cfg(feature = "morphology")]
            {
                if components.is_empty()
                    || components.iter().any(|part| !crate::valid_component(part))
                {
                    return Err(invalid(
                        "components must be non-empty supported ASCII tokens",
                    ));
                }
                policy_ids.push(crate::COMPONENT_POLICY.into());
            }
        }
        WorkbenchLexicalQuery::Compose { links_from, .. } => {
            policy_ids.extend([
                crate::PHONOLOGY_POLICY.into(),
                "dictionary-analysis-class-v1".into(),
            ]);
            if links_from.is_some() {
                policy_ids.push("compose-links-target-hops-v1".into());
            }
        }
        _ => return Err(invalid("source scan received a non-source query")),
    }

    #[cfg(feature = "morphology")]
    let mut component_cache = BTreeMap::<String, Result<Vec<String>, String>>::new();
    let mut distance_cache = BTreeMap::<String, usize>::new();
    let mut phonology_cache =
        BTreeMap::<(String, String), Result<WorkbenchAnalysisEvidence, String>>::new();
    for candidate in candidates {
        if let WorkbenchLexicalQuery::Compose {
            definition_contains,
            ..
        } = query
        {
            let definition_matches = definition_contains.as_ref().is_none_or(|needle| {
                matches!(&candidate.assertion.definition, lexicon::Presence::Value(definition)
                    if definition.text.to_ascii_lowercase().contains(&needle.to_ascii_lowercase()))
            });
            let links_match =
                compose_target_ids.is_none_or(|ids| ids.contains(&candidate.assertion.id));
            if !definition_matches || !links_match {
                *exclusions.entry("candidate-filter".into()).or_default() += 1;
                continue;
            }
        }
        let lexicon::Presence::Value(spelling) = &candidate.entry.spelling else {
            *exclusions.entry("no-headword".into()).or_default() += 1;
            continue;
        };
        let mut row = ComputedRow {
            candidate,
            spelling: spelling.clone(),
            occurrence_id: occurrence_tuple_hex(&candidate.assertion.occurrence),
            distance: None,
            analysis: None,
            distance_positions: Vec::new(),
            components: None,
        };
        match query {
            WorkbenchLexicalQuery::SpellingDistance {
                anchor,
                max_distance,
            } => {
                let distance = *distance_cache
                    .entry(spelling.clone())
                    .or_insert_with(|| strsim::levenshtein(anchor, spelling));
                if max_distance.is_some_and(|bound| distance > bound) {
                    *exclusions.entry("distance-bound".into()).or_default() += 1;
                    continue;
                }
                row.distance = Some(distance);
            }
            WorkbenchLexicalQuery::Prosody {
                kind,
                syllables,
                nuclei,
                primary_nucleus,
            } => {
                let Some((mapped, rule, capture, format_label)) = mapped_for(&row.candidate) else {
                    *exclusions
                        .entry("unmapped-source-class".into())
                        .or_default() += 1;
                    continue;
                };
                if kind.as_ref().is_some_and(|requested| requested != &mapped) {
                    *exclusions
                        .entry("requested-kind-filter".into())
                        .or_default() += 1;
                    continue;
                }
                let mut evidence = match phonology_cache
                    .entry((spelling.clone(), mapped.clone()))
                    .or_insert_with(|| phonology(spelling, &mapped))
                {
                    Ok(value) => value.clone(),
                    Err(_) => {
                        *exclusions.entry("analysis-error".into()).or_default() += 1;
                        continue;
                    }
                };
                evidence.class_evidence =
                    class_evidence(&row.candidate, mapped.clone(), rule, capture, format_label);
                if syllables.is_some() && evidence.syllable_count.is_none() {
                    *exclusions
                        .entry("unresolved-syllable-grouping".into())
                        .or_default() += 1;
                    continue;
                }
                if syllables.is_some_and(|wanted| evidence.syllable_count != Some(wanted))
                    || nuclei.is_some_and(|wanted| evidence.nuclei.len() != wanted)
                    || primary_nucleus
                        .is_some_and(|wanted| evidence.primary_nucleus != Some(wanted))
                {
                    *exclusions.entry("filter-mismatch".into()).or_default() += 1;
                    continue;
                }
                row.analysis = Some(evidence);
            }
            WorkbenchLexicalQuery::Nuclei { kind, pattern } => {
                let Some((mapped, rule, capture, format_label)) = mapped_for(&row.candidate) else {
                    *exclusions
                        .entry("unmapped-source-class".into())
                        .or_default() += 1;
                    continue;
                };
                if mapped != *kind {
                    *exclusions
                        .entry("requested-kind-filter".into())
                        .or_default() += 1;
                    continue;
                }
                let mut evidence = match phonology_cache
                    .entry((spelling.clone(), mapped.clone()))
                    .or_insert_with(|| phonology(spelling, &mapped))
                {
                    Ok(value) => value.clone(),
                    Err(_) => {
                        *exclusions.entry("analysis-error".into()).or_default() += 1;
                        continue;
                    }
                };
                if !evidence
                    .nuclei
                    .iter()
                    .map(|nucleus| nucleus.spelling.as_str())
                    .eq(pattern.iter().map(String::as_str))
                {
                    *exclusions
                        .entry("nucleus-pattern-mismatch".into())
                        .or_default() += 1;
                    continue;
                }
                evidence.class_evidence =
                    class_evidence(&row.candidate, mapped, rule, capture, format_label);
                row.analysis = Some(evidence);
            }
            WorkbenchLexicalQuery::Contrast {
                kind,
                require_mismatch,
                ..
            } => {
                let Some((mapped, rule, capture, format_label)) = mapped_for(&row.candidate) else {
                    *exclusions
                        .entry("unmapped-source-class".into())
                        .or_default() += 1;
                    continue;
                };
                if mapped != *kind {
                    *exclusions
                        .entry("requested-kind-filter".into())
                        .or_default() += 1;
                    continue;
                }
                let mut evidence = match phonology_cache
                    .entry((spelling.clone(), mapped.clone()))
                    .or_insert_with(|| phonology(spelling, &mapped))
                {
                    Ok(value) => value.clone(),
                    Err(_) => {
                        *exclusions.entry("analysis-error".into()).or_default() += 1;
                        continue;
                    }
                };
                let Some(anchors) = contrast_anchor.as_ref() else {
                    continue;
                };
                if evidence.nuclei.len() != anchors.len()
                    || evidence
                        .nuclei
                        .iter()
                        .any(|nucleus| nucleus.kind != "vowel")
                {
                    *exclusions
                        .entry("unaligned-or-non-vowel-nucleus".into())
                        .or_default() += 1;
                    continue;
                }
                let mut mismatch = false;
                for (index, (anchor, nucleus)) in anchors.iter().zip(&evidence.nuclei).enumerate() {
                    let left = bauske_phonology::Segment::from_str(anchor)
                        .map_err(|error| phonology_error(error.to_string()))?;
                    let right = bauske_phonology::Segment::from_str(&nucleus.spelling)
                        .map_err(|error| phonology_error(error.to_string()))?;
                    let comparison = bauske_phonology::DistanceModel::default()
                        .compare(left, right)
                        .map_err(|error| phonology_error(error.to_string()))?;
                    mismatch |= comparison.total() > 0;
                    row.distance_positions.push(DistancePosition {
                        index,
                        anchor: anchor.clone(),
                        candidate: nucleus.spelling.clone(),
                        total: comparison.total(),
                        profile_id: comparison.profile_id().into(),
                        source_id: comparison.source_id().into(),
                        contributions: comparison
                            .contributions()
                            .iter()
                            .map(|part| crate::DistanceContribution {
                                dimension: part.dimension().label().into(),
                                left: part.left().label().into(),
                                right: part.right().label().into(),
                                cost: part.cost(),
                            })
                            .collect(),
                    });
                }
                if *require_mismatch && !mismatch {
                    *exclusions
                        .entry("no-within-vowel-mismatch".into())
                        .or_default() += 1;
                    continue;
                }
                evidence.class_evidence =
                    class_evidence(&row.candidate, mapped, rule, capture, format_label);
                row.analysis = Some(evidence);
            }
            #[cfg(feature = "morphology")]
            WorkbenchLexicalQuery::Components { components } => {
                let returned = component_cache
                    .entry(spelling.clone())
                    .or_insert_with(|| crate::guarded_veljvo(spelling));
                let returned = match returned {
                    Ok(value) => value,
                    Err(reason) => {
                        *exclusions.entry(reason.clone()).or_default() += 1;
                        continue;
                    }
                };
                if !crate::ordered_subsequence(returned, components) {
                    *exclusions
                        .entry("component-sequence-mismatch".into())
                        .or_default() += 1;
                    continue;
                }
                row.components = Some(returned.clone());
            }
            #[cfg(not(feature = "morphology"))]
            WorkbenchLexicalQuery::Components { .. } => {}
            WorkbenchLexicalQuery::Compose {
                min_nuclei, kind, ..
            } => {
                let Some((mapped, rule, capture, format_label)) = mapped_for(&row.candidate) else {
                    *exclusions
                        .entry("unmapped-source-class".into())
                        .or_default() += 1;
                    continue;
                };
                if kind.as_ref().is_some_and(|wanted| wanted != &mapped) {
                    *exclusions
                        .entry("requested-kind-filter".into())
                        .or_default() += 1;
                    continue;
                }
                let mut evidence = match phonology_cache
                    .entry((spelling.clone(), mapped.clone()))
                    .or_insert_with(|| phonology(spelling, &mapped))
                {
                    Ok(value) => value.clone(),
                    Err(_) => {
                        *exclusions.entry("analysis-error".into()).or_default() += 1;
                        continue;
                    }
                };
                if min_nuclei.is_some_and(|minimum| evidence.nuclei.len() < minimum) {
                    *exclusions.entry("nuclei-filter".into()).or_default() += 1;
                    continue;
                }
                evidence.class_evidence =
                    class_evidence(&row.candidate, mapped, rule, capture, format_label);
                row.analysis = Some(evidence);
            }
            _ => unreachable!("query kind was validated above"),
        }
        rows.push(row);
    }

    rows.sort_by(|left, right| {
        if matches!(query, WorkbenchLexicalQuery::SpellingDistance { .. }) {
            left.distance
                .cmp(&right.distance)
                .then_with(|| left.spelling.as_bytes().cmp(right.spelling.as_bytes()))
        } else {
            left.spelling.as_bytes().cmp(right.spelling.as_bytes())
        }
        .then_with(|| left.candidate.source.id.cmp(&right.candidate.source.id))
        .then_with(|| left.occurrence_id.cmp(&right.occurrence_id))
        .then_with(|| {
            left.candidate
                .assertion
                .id
                .cmp(&right.candidate.assertion.id)
        })
    });
    let mut per_source =
        BTreeMap::<SourceId, (lexicon::SourceRecord, usize, BTreeSet<String>)>::new();
    let mut all_spellings = BTreeSet::new();
    for row in &rows {
        all_spellings.insert(row.spelling.clone());
        let slot = per_source
            .entry(row.candidate.source.id.clone())
            .or_insert_with(|| (row.candidate.source.clone(), 0, BTreeSet::new()));
        slot.1 += 1;
        slot.2.insert(row.spelling.clone());
    }
    let source_totals = per_source
        .into_values()
        .map(
            |(source, records, spellings)| WorkbenchComputedSourceTotal {
                source,
                records,
                distinct_spellings: spellings.len(),
            },
        )
        .collect::<Vec<_>>();
    Ok(ComputedScan {
        policy_ids,
        totals: WorkbenchComputedTotals {
            unit: "assertion_occurrences".into(),
            candidate_assertions,
            candidate_distinct_spellings,
            records: rows.len(),
            distinct_spellings: all_spellings.len(),
            source_groups: source_totals.len(),
        },
        source_totals,
        exclusions,
        rows,
    })
}

fn invalid(detail: &str) -> super::lexical::WorkbenchLexicalError {
    super::lexical::WorkbenchLexicalError::Computation(Box::new(crate::SearchError::InvalidQuery(
        detail.into(),
    )))
}

pub(crate) fn occurrence_tuple_hex(reference: &lexicon::OccurrenceRef) -> String {
    let mut bytes = Vec::new();
    for part in [
        reference.capture.namespace.as_bytes(),
        reference.capture.sha256.as_bytes(),
        reference.locator.as_bytes(),
    ] {
        bytes.extend_from_slice(&(part.len() as u64).to_be_bytes());
        bytes.extend_from_slice(part);
    }
    let mut hex = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write as _;
        let _ = write!(hex, "{byte:02x}");
    }
    hex
}

pub(crate) fn family_name(query: &super::lexical::WorkbenchLexicalQuery) -> &'static str {
    use super::lexical::WorkbenchLexicalQuery;
    match query {
        WorkbenchLexicalQuery::SpellingDistance { .. } => "spelling_distance",
        WorkbenchLexicalQuery::Prosody { .. } => "prosody",
        WorkbenchLexicalQuery::Nuclei { .. } => "nuclei",
        WorkbenchLexicalQuery::Contrast { .. } => "contrast",
        WorkbenchLexicalQuery::Components { .. } => "components",
        WorkbenchLexicalQuery::Compose { .. } => "compose",
        _ => "",
    }
}

fn phonology_error(detail: String) -> super::lexical::WorkbenchLexicalError {
    super::lexical::WorkbenchLexicalError::Computation(Box::new(crate::SearchError::Phonology(
        detail,
    )))
}

fn map_native(value: &str) -> Option<(&'static str, &'static str)> {
    crate::map_native_class(value)
}

fn class_evidence(
    candidate: &CandidateRef<'_>,
    mapped: String,
    rule: String,
    capture: Option<lexicon::CaptureRef>,
    format_label: Option<String>,
) -> WorkbenchAnalysisClassEvidence {
    WorkbenchAnalysisClassEvidence {
        native_word_class: candidate.entry.native_word_class.clone(),
        mapped_word_kind: mapped,
        rule,
        capture,
        format_label,
    }
}

const COMPUTED_ASSERTIONS_POLICY: &str = "workbench-computed-assertions-v1";
const ASSERTION_ORDER_POLICY: &str = "computed-assertion-order-v1";
const PROPOSAL_POLICY: &str = "caller-proposals-v1";
const PROPOSAL_NAMESPACE_POLICY: &str = "proposal-namespace-separated-v1";

pub(crate) fn proposal_exact(
    request: &super::lexical::WorkbenchLexicalRequest,
    format: &str,
    binding_sha256: &str,
) -> Result<WorkbenchProposalResult, super::lexical::WorkbenchLexicalError> {
    use super::lexical::{WorkbenchLexicalError, WorkbenchLexicalPosition};

    let super::lexical::WorkbenchLexicalQuery::ProposalExact {
        spelling,
        proposals: inputs,
    } = &request.query
    else {
        return Err(WorkbenchLexicalError::InvalidRequest {
            detail: "ProposalExact helper received a different query".into(),
        });
    };
    if !request.options.sources.is_empty() || request.options.alias_scope != AliasScope::ActiveOnly
    {
        return Err(WorkbenchLexicalError::InvalidRequest {
            detail: "ProposalExact requires empty sources and active-only alias scope".into(),
        });
    }

    // Construct every row before filtering or page handling. A malformed
    // nonmatching proposal is still an invalid caller input.
    let rows = inputs
        .iter()
        .enumerate()
        .map(|(index, input)| {
            let proposal = Proposal::new(&input.namespace, &input.spelling, &input.text)
                .map_err(|error| WorkbenchLexicalError::Proposal(Box::new(error)))?;
            let input_ordinal =
                u64::try_from(index).map_err(|_| WorkbenchLexicalError::InvalidRequest {
                    detail: "proposal input ordinal does not fit in u64".into(),
                })?;
            Ok(WorkbenchProposalRecord {
                id: proposal.id().to_owned(),
                namespace: proposal.namespace().to_owned(),
                spelling: proposal.spelling().to_owned(),
                text: proposal.text().to_owned(),
                input_ordinal,
            })
        })
        .collect::<Result<Vec<_>, WorkbenchLexicalError>>()?;

    // The position is defined within the matching exact-spelling cohort, not
    // within every constructor-valid proposal. A valid nonmatching row cannot
    // be used as a continuation for this query.
    let mut matches = rows
        .into_iter()
        .filter(|row| row.spelling == *spelling)
        .collect::<Vec<_>>();
    matches.sort_by(|left, right| {
        left.spelling
            .as_bytes()
            .cmp(right.spelling.as_bytes())
            .then(left.id.as_bytes().cmp(right.id.as_bytes()))
            .then(left.input_ordinal.cmp(&right.input_ordinal))
    });
    let start = if let Some(token) = request.options.cursor.as_deref() {
        let position = super::lexical::decode_cursor(token, request, &[], format, binding_sha256)?;
        let WorkbenchLexicalPosition::ProposalExact {
            proposal_id,
            input_ordinal,
        } = position
        else {
            return Err(WorkbenchLexicalError::StaleCursor);
        };
        let row = matches
            .iter()
            .position(|row| row.id == proposal_id && row.input_ordinal == input_ordinal)
            .map(|index| index + 1);
        if row.is_none() {
            return Err(WorkbenchLexicalError::InvalidCursorPosition);
        }
        row.expect("checked proposal cursor position")
    } else {
        0
    };
    let records = matches.len();
    let end = request
        .options
        .page_size
        .map_or(records, |size| start.saturating_add(size).min(records));
    let next_cursor = if end < records && end > start {
        let row = &matches[end - 1];
        Some(super::lexical::encode_cursor(
            request,
            &[],
            format,
            binding_sha256,
            WorkbenchLexicalPosition::ProposalExact {
                proposal_id: row.id.clone(),
                input_ordinal: row.input_ordinal,
            },
        )?)
    } else {
        None
    };
    Ok(WorkbenchProposalResult {
        policy_ids: vec![PROPOSAL_POLICY.into(), PROPOSAL_NAMESPACE_POLICY.into()],
        query: request.query.clone(),
        totals: WorkbenchProposalTotals {
            unit: "caller_proposals".into(),
            candidate_proposals: inputs.len(),
            records,
        },
        proposals: matches.drain(start..end).collect(),
        next_cursor,
        code_notices: Vec::new(),
    })
}

pub(crate) fn generate(
    query: &super::lexical::WorkbenchLexicalQuery,
    request: &super::lexical::WorkbenchLexicalRequest,
) -> Result<WorkbenchGeneratedResult, super::lexical::WorkbenchLexicalError> {
    use super::lexical::{WorkbenchLexicalError, WorkbenchLexicalQuery};

    let WorkbenchLexicalQuery::Generate {
        components,
        setting,
    } = query
    else {
        return Err(WorkbenchLexicalError::InvalidRequest {
            detail: "Generate helper received a different query".into(),
        });
    };
    #[cfg(not(feature = "morphology"))]
    {
        let _ = (components, setting, request);
        return Err(WorkbenchLexicalError::Computation(Box::new(
            crate::SearchError::FeatureDisabled("morphology"),
        )));
    }
    #[cfg(feature = "morphology")]
    {
        if !request.options.sources.is_empty()
            || request.options.alias_scope != AliasScope::ActiveOnly
            || request.options.page_size.is_some()
            || request.options.cursor.is_some()
        {
            return Err(WorkbenchLexicalError::InvalidRequest {
                detail: "Generate requires neutral source, alias, page, and cursor options".into(),
            });
        }
        let value = crate::guarded_generate(components, *setting)
            .map_err(|error| WorkbenchLexicalError::Computation(Box::new(error)))?;
        Ok(WorkbenchGeneratedResult {
            policy_ids: vec![crate::COMPONENT_POLICY.into()],
            query: query.clone(),
            value,
            code_notices: Vec::new(),
        })
    }
}
