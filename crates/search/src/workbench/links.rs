//! Bounded traversal across retained typed relations and source clauses.
//!
//! The request is a member of the existing lexical query route. This module
//! contains its Links-specific result and detail types.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use vlajmaji_lexicon::{
    ContextRecord, Direction, EntityId, EntryRecord, LocalRef, RelationKind, RelationRecord,
    RelationScope, SourceId, SourceRecord,
};

use crate::{
    DictionaryCompactHit, DictionaryCompactSourceAvailability, DictionaryHit,
    DictionarySourceAvailability,
};

use super::lexical::{
    self, WorkbenchLexicalError, WorkbenchLexicalPosition, WorkbenchLexicalQuery,
    WorkbenchLexicalRequest,
};
use super::{
    ConfiguredWorkbenchSearchEngine, WorkbenchFormEvidence, WorkbenchQueryError,
    WorkbenchQuerySelection, WorkbenchReferenceCompactResult, WorkbenchReferenceFullResult,
    WorkbenchSearchEngine,
};

/// Identity of the actual retained evidence used by a Links step.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum WorkbenchLinkEvidenceId {
    /// A typed relation retained in the dictionary document.
    TypedRelation {
        /// Original relation row identity.
        relation_id: EntityId,
    },
    /// A token parsed from an actual Note context row.
    SourceClause {
        /// Original context row identity.
        context_id: EntityId,
        /// Inclusive byte start, including the target's opening brace.
        byte_start: usize,
        /// Exclusive byte end, including the target's closing brace.
        byte_end: usize,
        /// Parser rule identity.
        rule_version: String,
        /// Exact target text between braces.
        target_spelling: String,
    },
}

/// Derived traversal-step identity; it is not a stored dictionary record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkbenchLinkStepId {
    /// Evidence from which this step was derived.
    pub evidence: WorkbenchLinkEvidenceId,
    /// Traversal orientation.
    pub orientation: String,
    /// Actual assertion at the from-side of the step.
    pub from_assertion_id: EntityId,
}

/// Full actual evidence record retained for one link.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkbenchLinkRecord {
    /// Typed evidence identity.
    pub identity: WorkbenchLinkEvidenceId,
    /// Publisher whose authority governs this evidence.
    pub publisher: SourceRecord,
    /// Literal authority branch used for this record.
    pub authority: String,
    /// Structural origin source.
    pub origin_source: SourceRecord,
    /// Actual typed relation, for relation evidence.
    #[serde(deserialize_with = "super::deserialize_required_option")]
    pub relation: Option<RelationRecord>,
    /// Actual Note context, for source-clause evidence.
    #[serde(deserialize_with = "super::deserialize_required_option")]
    pub context: Option<ContextRecord>,
    /// Parsed token, for source-clause evidence.
    #[serde(deserialize_with = "super::deserialize_required_option")]
    pub token: Option<vlajmaji_lexicon::SeeAlsoToken>,
    /// Actual owned revision named by the relation, when any.
    #[serde(deserialize_with = "super::deserialize_required_option")]
    pub editorial_revision: Option<vlajmaji_lexicon::OwnedRevisionRecord>,
    /// Original source evidence and payload status, when the evidence has one.
    #[serde(deserialize_with = "super::deserialize_required_option")]
    pub evidence: Option<WorkbenchFormEvidence>,
}

/// Compact retained evidence record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkbenchCompactLinkRecord {
    /// Typed evidence identity.
    pub identity: WorkbenchLinkEvidenceId,
    /// Publisher source identity.
    pub publisher: SourceId,
    /// Literal authority branch used for this record.
    pub authority: String,
    /// Original structural origin.
    pub origin: LocalRef,
    /// Actual relation kind, absent for source clauses.
    #[serde(deserialize_with = "super::deserialize_required_option")]
    pub relation_kind: Option<RelationKind>,
    /// Actual relation direction, absent for source clauses.
    #[serde(deserialize_with = "super::deserialize_required_option")]
    pub direction: Option<Direction>,
    /// Actual relation scope, absent for source clauses.
    #[serde(deserialize_with = "super::deserialize_required_option")]
    pub scope: Option<RelationScope>,
}

/// One candidate assertion considered at a target.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkbenchLinkCandidate {
    /// Actual candidate assertion identity.
    pub assertion_id: EntityId,
    /// Source owning the candidate.
    pub source_id: SourceId,
    /// Operational traversal disposition.
    pub traversal_status: String,
    /// Operational reason when this candidate cannot be traversed.
    #[serde(deserialize_with = "super::deserialize_required_option")]
    pub reason: Option<String>,
}

/// Full resolution and traversal disposition for one target observation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkbenchLinkTargetFull {
    /// Exact retained Entry for a source-clause spelling match.
    #[serde(deserialize_with = "super::deserialize_required_option")]
    pub entry: Option<EntryRecord>,
    /// Existing Full reference observation.
    pub resolution: WorkbenchReferenceFullResult,
    /// Actual retained candidate rows.
    pub candidates: Vec<WorkbenchLinkCandidate>,
    /// Operational target disposition.
    pub traversal_status: String,
    /// Operational reason when no candidate can be traversed.
    #[serde(deserialize_with = "super::deserialize_required_option")]
    pub reason: Option<String>,
}

/// Compact resolution and traversal disposition for one target observation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkbenchLinkTargetCompact {
    /// Exact retained Entry for a source-clause spelling match.
    #[serde(deserialize_with = "super::deserialize_required_option")]
    pub entry: Option<EntryRecord>,
    /// Existing Compact reference observation.
    pub resolution: WorkbenchReferenceCompactResult,
    /// Actual retained candidate rows.
    pub candidates: Vec<WorkbenchLinkCandidate>,
    /// Operational target disposition.
    pub traversal_status: String,
    /// Operational reason when no candidate can be traversed.
    #[serde(deserialize_with = "super::deserialize_required_option")]
    pub reason: Option<String>,
}

/// One complete Full traversal step.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkbenchLinkFullStep {
    /// Derived identity of this step.
    pub identity: WorkbenchLinkStepId,
    /// Minimum traversal distance from a seed.
    pub depth: u8,
    /// Actual Full from-side assertion hit.
    pub from: DictionaryHit,
    /// Actual retained evidence and provenance.
    pub record: WorkbenchLinkRecord,
    /// Resolution of the actual from-side target.
    pub origin_resolution: WorkbenchReferenceFullResult,
    /// Observations for the opposite side of this step.
    pub target_observations: Vec<WorkbenchLinkTargetFull>,
    /// Canonical union of eligible target assertion IDs.
    pub target_assertion_ids: Vec<EntityId>,
    /// Operational step disposition.
    pub traversal_status: String,
}

/// One compact traversal step with same-cache complete-detail reference.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkbenchLinkCompactStep {
    /// Derived identity of this step.
    pub identity: WorkbenchLinkStepId,
    /// Minimum traversal distance from a seed.
    pub depth: u8,
    /// Actual Compact from-side assertion hit.
    pub from: DictionaryCompactHit,
    /// Compact identity of actual retained evidence.
    pub record: WorkbenchCompactLinkRecord,
    /// Resolution of the actual from-side target.
    pub origin_resolution: WorkbenchReferenceCompactResult,
    /// Observations for the opposite side of this step.
    pub target_observations: Vec<WorkbenchLinkTargetCompact>,
    /// Canonical union of eligible target assertion IDs.
    pub target_assertion_ids: Vec<EntityId>,
    /// Operational step disposition.
    pub traversal_status: String,
    /// Same-view key for complete retained record detail.
    pub detail_ref: WorkbenchLinkDetailRef,
}

/// Counts over distinct pre-page traversal steps.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkbenchLinksTotals {
    /// Counted unit, always traversal_steps.
    pub unit: String,
    /// Number of distinct traversal steps before pagination.
    pub records: usize,
    /// Number of publishers with at least one step.
    pub source_groups: usize,
    /// Number of distinct evidence identities among steps.
    pub distinct_evidence: usize,
    /// Distinct union of from-side and eligible target assertions.
    pub associated_assertions: usize,
    /// Number of distinct seed and reached assertions, including terminal nodes.
    pub visited_assertions: usize,
}

/// Pre-page totals for one publisher.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkbenchLinksSourceTotal {
    /// Full publisher record.
    pub source: SourceRecord,
    /// Steps published by this source.
    pub records: usize,
    /// Distinct evidence identities published by this source.
    pub distinct_evidence: usize,
    /// Distinct union of from-side and eligible target assertions.
    pub associated_assertions: usize,
}

/// Full publisher group in one page.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkbenchLinksFullGroup {
    /// Publisher record.
    pub source: SourceRecord,
    /// Steps published by this source before paging.
    pub full_count: usize,
    /// Steps in this page.
    pub returned_count: usize,
    /// Whether this page omits publisher steps (`returned_count < full_count`), including on the final global page.
    pub partial: bool,
    /// Page steps in canonical traversal order.
    pub hits: Vec<WorkbenchLinkFullStep>,
}

/// Compact publisher group in one page.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkbenchLinksCompactGroup {
    /// Publisher record.
    pub source: SourceRecord,
    /// Steps published by this source before paging.
    pub full_count: usize,
    /// Steps in this page.
    pub returned_count: usize,
    /// Whether this page omits publisher steps (`returned_count < full_count`), including on the final global page.
    pub partial: bool,
    /// Page steps in canonical traversal order.
    pub hits: Vec<WorkbenchLinkCompactStep>,
}

/// Full Links traversal result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkbenchLinksFull {
    /// Traversal policy.
    pub policy: String,
    /// Prepared dictionary state.
    pub state_sha256: String,
    /// Validated query including exact spelling and depth.
    pub query: WorkbenchLexicalQuery,
    /// Resolved sources in canonical order.
    pub resolved_sources: Vec<SourceId>,
    /// Canonical seeds.
    pub seed_assertion_ids: Vec<EntityId>,
    /// Pre-page totals.
    pub totals: WorkbenchLinksTotals,
    /// Pre-page totals by publisher.
    pub source_totals: Vec<WorkbenchLinksSourceTotal>,
    /// Existing Full availability observations.
    pub availability: Vec<DictionarySourceAvailability>,
    /// Paged steps by publisher.
    pub groups: Vec<WorkbenchLinksFullGroup>,
    /// Continuation token when more steps remain.
    #[serde(deserialize_with = "super::deserialize_required_option")]
    pub next_cursor: Option<String>,
    /// Original code notices.
    pub code_notices: Vec<vlajmaji_lexicon::CodeNotice>,
}

/// Compact Links traversal result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkbenchLinksCompact {
    /// Traversal policy.
    pub policy: String,
    /// Prepared dictionary state.
    pub state_sha256: String,
    /// Validated query including exact spelling and depth.
    pub query: WorkbenchLexicalQuery,
    /// Resolved sources in canonical order.
    pub resolved_sources: Vec<SourceId>,
    /// Canonical seeds.
    pub seed_assertion_ids: Vec<EntityId>,
    /// Pre-page totals.
    pub totals: WorkbenchLinksTotals,
    /// Pre-page totals by publisher.
    pub source_totals: Vec<WorkbenchLinksSourceTotal>,
    /// Existing Compact availability observations.
    pub availability: Vec<DictionaryCompactSourceAvailability>,
    /// Paged steps by publisher.
    pub groups: Vec<WorkbenchLinksCompactGroup>,
    /// Continuation token when more steps remain.
    #[serde(deserialize_with = "super::deserialize_required_option")]
    pub next_cursor: Option<String>,
    /// Original code notices.
    pub code_notices: Vec<vlajmaji_lexicon::CodeNotice>,
    /// Compact snapshot binding.
    pub snapshot_sha256: String,
}

/// Strict relation-or-context detail identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkbenchLinkDetailRef {
    /// Detail-key version.
    pub version: u32,
    /// Typed-row digest policy.
    pub digest_policy: String,
    /// Detail role.
    pub role: String,
    /// Direct or configured effective binding.
    pub binding_sha256: String,
    /// Declared publisher.
    pub publisher: SourceId,
    /// Actual typed evidence identity.
    pub identity: WorkbenchLinkEvidenceId,
    /// SHA-256 of the actual typed relation or context row.
    pub record_sha256: String,
}

/// Direct link-evidence detail response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkbenchLinkDetailResponse {
    /// Response version.
    pub version: u32,
    /// Stable response kind.
    pub kind: String,
    /// Effective binding.
    pub binding_sha256: String,
    /// Captured selection.
    pub selection: WorkbenchQuerySelection,
    /// Validated evidence key.
    pub reference: WorkbenchLinkDetailRef,
    /// Complete retained evidence.
    pub record: WorkbenchLinkRecord,
    /// Original structural origin resolution.
    pub origin_resolution: WorkbenchReferenceCompactResult,
    /// Current all-source target observations.
    pub target_observations: Vec<WorkbenchLinkTargetCompact>,
}

/// Configured link-evidence detail response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfiguredWorkbenchLinkDetailResponse {
    /// Response version.
    pub version: u32,
    /// Stable response kind.
    pub kind: String,
    /// Effective binding.
    pub binding_sha256: String,
    /// Captured configured identity.
    pub configuration: vlajmaji_store::WorkbenchProfileIdentity,
    /// Captured selection.
    pub selection: WorkbenchQuerySelection,
    /// Validated evidence key.
    pub reference: WorkbenchLinkDetailRef,
    /// Complete retained evidence.
    pub record: WorkbenchLinkRecord,
    /// Original structural origin resolution.
    pub origin_resolution: WorkbenchReferenceCompactResult,
    /// Current all-source target observations.
    pub target_observations: Vec<WorkbenchLinkTargetCompact>,
}

#[derive(Clone)]
struct LinkEvidenceDraft {
    identity: WorkbenchLinkEvidenceId,
    publisher: SourceRecord,
    origin_source: SourceRecord,
    origin: LocalRef,
    target: vlajmaji_lexicon::RelationTarget,
    relation: Option<RelationRecord>,
    context_id: Option<EntityId>,
    token: Option<vlajmaji_lexicon::SeeAlsoToken>,
    revision_id: Option<EntityId>,
    record_sha256: String,
    source_evidence: Option<vlajmaji_lexicon::SourceEvidence>,
    authority: String,
}

#[derive(Clone)]
struct LinkEdgeDraft {
    evidence: usize,
    orientation: &'static str,
    target: vlajmaji_lexicon::RelationTarget,
    entry: Option<EntryRecord>,
}

#[derive(Clone)]
struct EntryLinkIdentity {
    source: SourceId,
    spelling: vlajmaji_lexicon::Presence<String>,
}

#[derive(Clone)]
struct AssertionLinkIdentity {
    id: EntityId,
    entry: EntityId,
    occurrence: vlajmaji_lexicon::OccurrenceRef,
    lexical: bool,
}

pub(super) struct LinkRows {
    // The graph retains identity/association indexes rather than a second copy
    // of the dictionary's model rows. Complete Entries are kept only when a
    // parsed source clause actually names their exact spelling.
    entries: BTreeMap<EntityId, EntryLinkIdentity>,
    clause_entries: BTreeMap<EntityId, EntryRecord>,
    assertions: BTreeMap<EntityId, AssertionLinkIdentity>,
    assertion_order: Vec<EntityId>,
    form_entries: BTreeMap<EntityId, EntityId>,
    sense_assertions: BTreeMap<EntityId, BTreeSet<EntityId>>,
    evidence: Vec<LinkEvidenceDraft>,
}

fn link_integrity(detail: impl Into<String>) -> crate::workbench::lexical::WorkbenchLexicalError {
    crate::workbench::lexical::WorkbenchLexicalError::Integrity {
        detail: detail.into(),
    }
}

fn validate_relation_provenance(
    kind: &RelationKind,
    has_source_evidence: bool,
    has_editorial_revision: bool,
    relation_id: &EntityId,
) -> Result<(), crate::workbench::lexical::WorkbenchLexicalError> {
    let valid = match kind {
        RelationKind::SourceSeeAlso | RelationKind::SourceEtymology => {
            has_source_evidence && !has_editorial_revision
        }
        RelationKind::EditorialLink => !has_source_evidence && has_editorial_revision,
        RelationKind::Other(_) => !(has_source_evidence && has_editorial_revision),
    };
    if valid {
        Ok(())
    } else {
        Err(link_integrity(format!(
            "relation {relation_id} has inconsistent evidence provenance"
        )))
    }
}

fn entry_spelling(entry: &EntryRecord) -> &str {
    match &entry.spelling {
        vlajmaji_lexicon::Presence::Value(value) => value,
        _ => "",
    }
}

fn entry_spelling_order(entry: &EntryLinkIdentity) -> &str {
    match &entry.spelling {
        vlajmaji_lexicon::Presence::Value(value) => value,
        vlajmaji_lexicon::Presence::Missing | vlajmaji_lexicon::Presence::Null => "",
    }
}

fn occurrence_tuple_hex(reference: &vlajmaji_lexicon::OccurrenceRef) -> String {
    let parts = [
        reference.capture.namespace.as_bytes(),
        reference.capture.sha256.as_bytes(),
        reference.locator.as_bytes(),
    ];
    let mut bytes = Vec::new();
    for part in parts {
        bytes.extend_from_slice(&(part.len() as u64).to_be_bytes());
        bytes.extend_from_slice(part);
    }
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write as _;
        let _ = write!(output, "{byte:02x}");
    }
    output
}

fn local_source<'a>(
    local: &LocalRef,
    metadata: &'a vlajmaji_lexicon::DictionaryMetadata,
    sources: &'a std::collections::BTreeMap<SourceId, SourceRecord>,
) -> Option<&'a SourceRecord> {
    let source = match local {
        LocalRef::Entry(id) => metadata
            .entries
            .iter()
            .find(|row| row.id == *id)?
            .source
            .clone(),
        LocalRef::Assertion(id) => {
            let assertion = metadata.assertions.iter().find(|row| row.id == *id)?;
            metadata
                .entries
                .iter()
                .find(|row| row.id == assertion.entry)?
                .source
                .clone()
        }
        LocalRef::Form(id) => {
            let form = metadata.forms.iter().find(|row| row.id == *id)?;
            metadata
                .entries
                .iter()
                .find(|row| row.id == form.entry)?
                .source
                .clone()
        }
        LocalRef::EditorialSense(id) => {
            let sense = metadata.editorial_senses.iter().find(|row| row.id == *id)?;
            metadata
                .owned_revisions
                .iter()
                .find(|row| row.id == sense.revision)?
                .source
                .clone()
        }
    };
    sources.get(&source)
}

fn occurrence_source<'a>(
    evidence: &vlajmaji_lexicon::SourceEvidence,
    registry: &'a vlajmaji_lexicon::RegistryDocument,
) -> Option<&'a SourceRecord> {
    let snapshot = registry.snapshots.iter().find(|row| {
        vlajmaji_lexicon::CaptureRef::from(&row.snapshot) == evidence.occurrence.capture
    })?;
    registry
        .sources
        .iter()
        .find(|row| row.id == snapshot.source)
}

fn build_link_rows(
    metadata: &vlajmaji_lexicon::DictionaryMetadata,
    registry: &vlajmaji_lexicon::RegistryDocument,
    selected: &std::collections::BTreeMap<SourceId, std::collections::BTreeSet<String>>,
    only_evidence: Option<&WorkbenchLinkEvidenceId>,
) -> Result<LinkRows, crate::workbench::lexical::WorkbenchLexicalError> {
    let sources = registry
        .sources
        .iter()
        .cloned()
        .map(|row| (row.id.clone(), row))
        .collect::<BTreeMap<_, _>>();
    let entry_identities = metadata
        .entries
        .iter()
        .map(|row| {
            (
                row.id.clone(),
                EntryLinkIdentity {
                    source: row.source.clone(),
                    spelling: row.spelling.clone(),
                },
            )
        })
        .collect::<BTreeMap<_, _>>();
    let form_entries = metadata
        .forms
        .iter()
        .map(|row| (row.id.clone(), row.entry.clone()))
        .collect();
    let sense_assertions = metadata
        .editorial_senses
        .iter()
        .map(|row| {
            (
                row.id.clone(),
                row.assertions.iter().cloned().collect::<BTreeSet<_>>(),
            )
        })
        .collect();
    let assertions_by_id = metadata
        .assertions
        .iter()
        .map(|row| (row.id.clone(), row))
        .collect::<BTreeMap<_, _>>();
    let mut assertions = std::collections::BTreeMap::new();
    for (source, ids) in selected {
        for assertion in &metadata.assertions {
            if ids.contains(assertion.id.as_str())
                && entry_identities
                    .get(&assertion.entry)
                    .is_some_and(|entry| &entry.source == source)
            {
                assertions.insert(
                    assertion.id.clone(),
                    AssertionLinkIdentity {
                        id: assertion.id.clone(),
                        entry: assertion.entry.clone(),
                        occurrence: assertion.occurrence.clone(),
                        lexical: assertion.eligibility == vlajmaji_lexicon::Eligibility::Lexical,
                    },
                );
            }
        }
        if !sources.contains_key(source) {
            return Err(link_integrity(format!(
                "selected Links source {source} is missing"
            )));
        }
    }
    let mut evidence = Vec::new();
    for relation in metadata.relations.iter().filter(|relation| {
        only_evidence.is_none_or(|identity| {
            matches!(identity, WorkbenchLinkEvidenceId::TypedRelation { relation_id } if relation_id == &relation.id)
        })
    }) {
        let origin_source = local_source(&relation.origin, metadata, &sources)
            .ok_or_else(|| {
                link_integrity(format!(
                    "relation {} has no structural origin source",
                    relation.id
                ))
            })?
            .clone();
        let source_evidence = relation.evidence.as_ref();
        let editorial_revision = relation.editorial_revision.as_ref();
        validate_relation_provenance(
            &relation.kind,
            source_evidence.is_some(),
            editorial_revision.is_some(),
            &relation.id,
        )?;
        let (publisher, authority, source_evidence, revision) =
            if let Some(source_evidence) = source_evidence {
                let publisher = occurrence_source(source_evidence, registry)
                    .ok_or_else(|| {
                        link_integrity(format!(
                            "relation {} evidence has no source owner",
                            relation.id
                        ))
                    })?
                    .clone();
                if publisher.id != origin_source.id || relation.editorial_revision.is_some() {
                    return Err(link_integrity(format!(
                        "source relation {} publisher differs from origin",
                        relation.id
                    )));
                }
                (
                    publisher,
                    "source_evidence",
                    Some(source_evidence.clone()),
                    None,
                )
            } else if let Some(revision_id) = editorial_revision {
                let revision = metadata
                    .owned_revisions
                    .iter()
                    .find(|row| row.id == *revision_id)
                    .ok_or_else(|| {
                        link_integrity(format!(
                            "relation {} names missing revision {revision_id}",
                            relation.id
                        ))
                    })?;
                let publisher = sources
                    .get(&revision.source)
                    .ok_or_else(|| {
                        link_integrity(format!(
                            "revision {} publisher source is missing",
                            revision.id
                        ))
                    })?
                    .clone();
                (
                    publisher,
                    "editorial_revision",
                    None,
                    Some(revision.id.clone()),
                )
            } else {
                (
                    origin_source.clone(),
                    "unattributed_origin_source",
                    None,
                    None,
                )
            };
        let _ = authority;
        evidence.push(LinkEvidenceDraft {
            identity: WorkbenchLinkEvidenceId::TypedRelation {
                relation_id: relation.id.clone(),
            },
            publisher,
            origin_source,
            origin: relation.origin.clone(),
            target: relation.target.clone(),
            relation: Some(relation.clone()),
            context_id: None,
            token: None,
            revision_id: revision,
            record_sha256: lexical::typed_sha256(relation)?,
            source_evidence,
            authority: authority.into(),
        });
    }
    let mut target_spellings = BTreeSet::new();
    for context in metadata.contexts.iter().filter(|context| {
        only_evidence.is_none_or(|identity| {
            matches!(identity, WorkbenchLinkEvidenceId::SourceClause { context_id, .. } if context_id == &context.id)
        })
    }) {
        if context.kind != vlajmaji_lexicon::ContextKind::Note {
            continue;
        }
        let vlajmaji_lexicon::Presence::Value(value) = &context.value else {
            continue;
        };
        let Some(publisher) = occurrence_source(&context.evidence, registry) else {
            return Err(link_integrity(format!(
                "Note context {} has no evidence source owner",
                context.id
            )));
        };
        let assertion = assertions_by_id.get(&context.assertion).ok_or_else(|| {
            link_integrity(format!(
                "Note context {} names a missing assertion",
                context.id
            ))
        })?;
        let entry = entry_identities.get(&assertion.entry).ok_or_else(|| {
            link_integrity(format!(
                "Note context {} assertion has no Entry",
                context.id
            ))
        })?;
        let origin = LocalRef::Assertion(assertion.id.clone());
        let context_sha256 = lexical::typed_sha256(context)?;
        for token in vlajmaji_lexicon::parse_see_also_tokens(&value.text) {
            target_spellings.insert(token.target_spelling.clone());
            evidence.push(LinkEvidenceDraft {
                identity: WorkbenchLinkEvidenceId::SourceClause {
                    context_id: context.id.clone(),
                    byte_start: token.byte_start,
                    byte_end: token.byte_end,
                    rule_version: token.rule_version.clone(),
                    target_spelling: token.target_spelling.clone(),
                },
                publisher: publisher.clone(),
                origin_source: sources.get(&entry.source).cloned().ok_or_else(|| {
                    link_integrity(format!("context {} Entry source is missing", context.id))
                })?,
                origin: origin.clone(),
                target: vlajmaji_lexicon::RelationTarget::Unresolved(token.target_spelling.clone()),
                relation: None,
                context_id: Some(context.id.clone()),
                token: Some(token),
                revision_id: None,
                record_sha256: context_sha256.clone(),
                source_evidence: Some(context.evidence.clone()),
                authority: "derived_source_clause".into(),
            });
        }
    }
    let clause_entries = metadata.entries.iter()
        .filter(|entry| matches!(&entry.spelling, vlajmaji_lexicon::Presence::Value(value) if target_spellings.contains(value)))
        .map(|entry| (entry.id.clone(), entry.clone()))
        .collect();
    Ok(LinkRows {
        entries: entry_identities,
        clause_entries,
        assertion_order: metadata
            .assertions
            .iter()
            .filter(|row| assertions.contains_key(&row.id))
            .map(|row| row.id.clone())
            .collect(),
        assertions,
        form_entries,
        sense_assertions,
        evidence,
    })
}

fn local_matches(local: &LocalRef, assertion: &AssertionLinkIdentity, rows: &LinkRows) -> bool {
    match local {
        LocalRef::Assertion(id) => id == &assertion.id,
        LocalRef::Entry(id) => id == &assertion.entry,
        LocalRef::Form(id) => rows
            .form_entries
            .get(id)
            .is_some_and(|entry| entry == &assertion.entry),
        LocalRef::EditorialSense(id) => rows
            .sense_assertions
            .get(id)
            .is_some_and(|ids| ids.contains(&assertion.id)),
    }
}

fn edges_from(
    rows: &LinkRows,
    node: &AssertionLinkIdentity,
    resolved: &[SourceId],
    mut target_hops: impl FnMut(
        &vlajmaji_lexicon::RelationTarget,
    ) -> Result<Vec<EntityId>, WorkbenchQueryError>,
) -> Result<Vec<LinkEdgeDraft>, WorkbenchQueryError> {
    let mut edges = Vec::new();
    for (index, evidence) in rows.evidence.iter().enumerate() {
        if !resolved.contains(&evidence.publisher.id) {
            continue;
        }
        if let Some(relation) = &evidence.relation {
            let forward = local_matches(&relation.origin, node, rows);
            let reverse = match &relation.target {
                vlajmaji_lexicon::RelationTarget::Local(local) => local_matches(local, node, rows),
                vlajmaji_lexicon::RelationTarget::Catalog(_)
                    if matches!(
                        relation.direction,
                        Direction::Reverse | Direction::Bidirectional
                    ) =>
                {
                    target_hops(&relation.target)?
                        .iter()
                        .any(|id| id == &node.id)
                }
                _ => false,
            };
            if relation.direction == Direction::Reverse {
                if reverse {
                    edges.push(LinkEdgeDraft {
                        evidence: index,
                        orientation: "reverse",
                        target: vlajmaji_lexicon::RelationTarget::Local(relation.origin.clone()),
                        entry: None,
                    });
                }
                continue;
            };
            if forward
                && matches!(
                    relation.direction,
                    Direction::Forward | Direction::Bidirectional | Direction::Unspecified
                )
            {
                edges.push(LinkEdgeDraft {
                    evidence: index,
                    orientation: "forward",
                    target: relation.target.clone(),
                    entry: None,
                });
            }
            if reverse && relation.direction == Direction::Bidirectional {
                edges.push(LinkEdgeDraft {
                    evidence: index,
                    orientation: "reverse",
                    target: vlajmaji_lexicon::RelationTarget::Local(relation.origin.clone()),
                    entry: None,
                });
            }
        } else if let (Some(_context_id), Some(token)) = (&evidence.context_id, &evidence.token)
            && evidence.origin == LocalRef::Assertion(node.id.clone())
        {
            let mut entries = rows.clause_entries.values()
                    .filter(|entry| matches!(&entry.spelling, vlajmaji_lexicon::Presence::Value(value) if value == &token.target_spelling))
                    .cloned()
                    .collect::<Vec<_>>();
            entries.sort_by(|left, right| {
                let left_source = rows.entries.get(&left.id).map(|row| &row.source);
                let right_source = rows.entries.get(&right.id).map(|row| &row.source);
                (left_source, entry_spelling(left), &left.id).cmp(&(
                    right_source,
                    entry_spelling(right),
                    &right.id,
                ))
            });
            if entries.is_empty() {
                edges.push(LinkEdgeDraft {
                    evidence: index,
                    orientation: "forward",
                    target: vlajmaji_lexicon::RelationTarget::Unresolved(
                        token.target_spelling.clone(),
                    ),
                    entry: None,
                });
            } else {
                for entry in entries {
                    edges.push(LinkEdgeDraft {
                        evidence: index,
                        orientation: "forward",
                        target: vlajmaji_lexicon::RelationTarget::Local(LocalRef::Entry(
                            entry.id.clone(),
                        )),
                        entry: Some(entry),
                    });
                }
            }
        }
    }
    Ok(edges)
}

#[derive(Clone)]
struct TargetObservationRef {
    cache_index: usize,
    entry: Option<EntryRecord>,
}

#[derive(Clone)]
struct RawLinkStep {
    evidence: usize,
    orientation: &'static str,
    depth: u8,
    from: AssertionLinkIdentity,
    observations: Vec<TargetObservationRef>,
    target_assertions: BTreeSet<EntityId>,
}

type EnumeratedLinkSteps = (Vec<EntityId>, Vec<RawLinkStep>, BTreeSet<EntityId>);

fn enumerate_link_steps(
    rows: &LinkRows,
    spelling: &str,
    max_depth: u8,
    resolved_sources: &[SourceId],
    mut observe: impl FnMut(
        &vlajmaji_lexicon::RelationTarget,
        Option<EntryRecord>,
    ) -> Result<
        (usize, Vec<vlajmaji_lexicon::AssertionRecord>),
        WorkbenchQueryError,
    >,
) -> Result<EnumeratedLinkSteps, WorkbenchQueryError> {
    let mut seeds = rows.assertion_order.iter().filter_map(|id| rows.assertions.get(id)).filter_map(|assertion| {
        let entry = rows.entries.get(&assertion.entry)?;
        (matches!(&entry.spelling, vlajmaji_lexicon::Presence::Value(value) if value == spelling)
            && assertion.lexical && resolved_sources.contains(&entry.source))
            .then_some(assertion.clone())
    }).collect::<Vec<_>>();
    seeds.sort_by(|left, right| {
        let left_entry = rows.entries.get(&left.entry);
        let right_entry = rows.entries.get(&right.entry);
        (
            left_entry.map(|entry| &entry.source),
            left_entry.map(entry_spelling_order),
            occurrence_tuple_hex(&left.occurrence),
            &left.id,
        )
            .cmp(&(
                right_entry.map(|entry| &entry.source),
                right_entry.map(entry_spelling_order),
                occurrence_tuple_hex(&right.occurrence),
                &right.id,
            ))
    });
    let seed_ids = seeds.iter().map(|row| row.id.clone()).collect::<Vec<_>>();
    let mut visited = seed_ids.iter().cloned().collect::<BTreeSet<_>>();
    let mut frontier = seeds.into_iter().map(|row| (row, 0u8)).collect::<Vec<_>>();
    let mut steps = Vec::new();
    let mut identities = Vec::<(WorkbenchLinkEvidenceId, String, EntityId)>::new();

    while !frontier.is_empty() {
        let (node, distance) = frontier.remove(0);
        if distance >= max_depth {
            continue;
        }
        let edges = edges_from(rows, &node, resolved_sources, |target| {
            observe(target, None).map(|(_, hits)| hits.into_iter().map(|row| row.id).collect())
        })?;
        for edge in edges {
            let (cache_index, resolved_target_rows) = observe(&edge.target, edge.entry.clone())?;
            let target_rows = resolved_target_rows
                .iter()
                .filter_map(|row| {
                    rows.assertions
                        .get(&row.id)
                        .filter(|current| current.lexical)
                        .cloned()
                })
                .collect::<Vec<_>>();
            let hop_ids = target_rows
                .iter()
                .map(|row| row.id.clone())
                .collect::<BTreeSet<_>>();
            let identity = (
                rows.evidence[edge.evidence].identity.clone(),
                edge.orientation.to_owned(),
                node.id.clone(),
            );
            if !identities.contains(&identity) {
                identities.push(identity);
                steps.push(RawLinkStep {
                    evidence: edge.evidence,
                    orientation: edge.orientation,
                    depth: distance + 1,
                    from: node.clone(),
                    observations: vec![TargetObservationRef {
                        cache_index,
                        entry: edge.entry.clone(),
                    }],
                    target_assertions: hop_ids,
                });
            } else if let Some(step) = steps.iter_mut().find(|step| {
                step.evidence == edge.evidence
                    && step.orientation == edge.orientation
                    && step.from.id == node.id
            }) {
                step.target_assertions.extend(hop_ids);
                step.observations.push(TargetObservationRef {
                    cache_index,
                    entry: edge.entry.clone(),
                });
            }
            if distance + 1 < max_depth {
                for target in target_rows {
                    if visited.insert(target.id.clone()) {
                        frontier.push((target, distance + 1));
                    }
                }
            } else {
                for target in target_rows {
                    visited.insert(target.id);
                }
            }
        }
    }
    Ok((seed_ids, steps, visited))
}

fn evidence_order(
    left: &WorkbenchLinkEvidenceId,
    right: &WorkbenchLinkEvidenceId,
) -> std::cmp::Ordering {
    match (left, right) {
        (
            WorkbenchLinkEvidenceId::TypedRelation { relation_id: a },
            WorkbenchLinkEvidenceId::TypedRelation { relation_id: b },
        ) => (0u8, a).cmp(&(0u8, b)),
        (
            WorkbenchLinkEvidenceId::TypedRelation { .. },
            WorkbenchLinkEvidenceId::SourceClause { .. },
        ) => std::cmp::Ordering::Less,
        (
            WorkbenchLinkEvidenceId::SourceClause { .. },
            WorkbenchLinkEvidenceId::TypedRelation { .. },
        ) => std::cmp::Ordering::Greater,
        (
            WorkbenchLinkEvidenceId::SourceClause {
                context_id: a,
                byte_start: as_,
                byte_end: ae,
                rule_version: ar,
                target_spelling: at,
            },
            WorkbenchLinkEvidenceId::SourceClause {
                context_id: b,
                byte_start: bs,
                byte_end: be,
                rule_version: br,
                target_spelling: bt,
            },
        ) => (a, as_, ae, ar, at).cmp(&(b, bs, be, br, bt)),
    }
}

fn compare_raw_steps(
    rows: &LinkRows,
    left: &RawLinkStep,
    right: &RawLinkStep,
) -> std::cmp::Ordering {
    let left_evidence = &rows.evidence[left.evidence];
    let right_evidence = &rows.evidence[right.evidence];
    left_evidence
        .publisher
        .id
        .cmp(&right_evidence.publisher.id)
        .then(left.depth.cmp(&right.depth))
        .then_with(|| evidence_order(&left_evidence.identity, &right_evidence.identity))
        .then_with(|| match (left.orientation, right.orientation) {
            ("forward", "reverse") => std::cmp::Ordering::Less,
            ("reverse", "forward") => std::cmp::Ordering::Greater,
            _ => std::cmp::Ordering::Equal,
        })
        .then_with(|| left.from.id.cmp(&right.from.id))
}

struct PreparedFullLinks {
    rows: LinkRows,
    resolved_sources: Vec<SourceId>,
    availability: Vec<DictionarySourceAvailability>,
    state_sha256: String,
    code_notices: Vec<vlajmaji_lexicon::CodeNotice>,
}
pub(super) struct PreparedCompactLinks {
    rows: LinkRows,
    resolved_sources: Vec<SourceId>,
    availability: Vec<DictionaryCompactSourceAvailability>,
    state_sha256: String,
    snapshot_sha256: String,
    code_notices: Vec<vlajmaji_lexicon::CodeNotice>,
}

trait FullLinksRuntime {
    fn prepare_full_links(
        &mut self,
        request: &WorkbenchLexicalRequest,
    ) -> Result<PreparedFullLinks, WorkbenchQueryError>;
    fn resolve_full_link_target(
        &mut self,
        target: &vlajmaji_lexicon::RelationTarget,
    ) -> Result<WorkbenchReferenceFullResult, WorkbenchQueryError>;
    fn full_link_hit(
        &mut self,
        assertion_id: &EntityId,
    ) -> Result<DictionaryHit, WorkbenchQueryError>;
    fn full_link_evidence(
        &mut self,
        evidence: &vlajmaji_lexicon::SourceEvidence,
        source: &SourceId,
    ) -> Result<WorkbenchFormEvidence, WorkbenchQueryError>;
    fn full_link_context(
        &mut self,
        context_id: &EntityId,
    ) -> Result<ContextRecord, WorkbenchQueryError>;
    fn full_link_revision(
        &mut self,
        revision_id: &EntityId,
    ) -> Result<vlajmaji_lexicon::OwnedRevisionRecord, WorkbenchQueryError>;
}

impl FullLinksRuntime for WorkbenchSearchEngine {
    fn prepare_full_links(
        &mut self,
        request: &WorkbenchLexicalRequest,
    ) -> Result<PreparedFullLinks, WorkbenchQueryError> {
        let engine = self
            .full_engine()
            .map_err(WorkbenchLexicalError::from)
            .map_err(WorkbenchQueryError::from)?;
        let (metadata, registry, availability, _captures, selected, state_sha256) =
            engine.lexical_link_rows();
        let source_state = availability
            .iter()
            .map(|row| (row.source.id.clone(), row.selectable, row.reason))
            .collect::<Vec<_>>();
        let resolved_sources = lexical::resolve_sources(registry, &request.options, &source_state)
            .map_err(WorkbenchQueryError::from)?;
        let rows = build_link_rows(metadata, registry, selected, None)
            .map_err(WorkbenchQueryError::from)?;
        Ok(PreparedFullLinks {
            rows,
            resolved_sources,
            availability: availability.to_vec(),
            state_sha256: state_sha256.to_owned(),
            code_notices: metadata.code_notices.clone(),
        })
    }
    fn resolve_full_link_target(
        &mut self,
        target: &vlajmaji_lexicon::RelationTarget,
    ) -> Result<WorkbenchReferenceFullResult, WorkbenchQueryError> {
        self.resolve_reference_full_inner(target)
    }
    fn full_link_hit(
        &mut self,
        assertion_id: &EntityId,
    ) -> Result<DictionaryHit, WorkbenchQueryError> {
        let engine = self.full_engine()?;
        let (metadata, ..) = engine.lexical_link_rows();
        let assertion = metadata
            .assertions
            .iter()
            .find(|row| &row.id == assertion_id)
            .ok_or_else(|| {
                WorkbenchQueryError::from(WorkbenchLexicalError::Integrity {
                    detail: format!("Links from assertion {assertion_id} disappeared"),
                })
            })?;
        engine
            .lexical_link_hit(assertion)
            .map_err(WorkbenchQueryError::from)
    }
    fn full_link_evidence(
        &mut self,
        evidence: &vlajmaji_lexicon::SourceEvidence,
        source: &SourceId,
    ) -> Result<WorkbenchFormEvidence, WorkbenchQueryError> {
        self.full_engine()?
            .lexical_link_evidence(evidence, source)
            .map_err(WorkbenchQueryError::from)
    }
    fn full_link_context(
        &mut self,
        context_id: &EntityId,
    ) -> Result<ContextRecord, WorkbenchQueryError> {
        self.full_engine()?
            .lexical_link_context(context_id)
            .map_err(WorkbenchQueryError::from)
    }
    fn full_link_revision(
        &mut self,
        revision_id: &EntityId,
    ) -> Result<vlajmaji_lexicon::OwnedRevisionRecord, WorkbenchQueryError> {
        self.full_engine()?
            .lexical_link_revision(revision_id)
            .map_err(WorkbenchQueryError::from)
    }
}

impl FullLinksRuntime for ConfiguredWorkbenchSearchEngine {
    fn prepare_full_links(
        &mut self,
        request: &WorkbenchLexicalRequest,
    ) -> Result<PreparedFullLinks, WorkbenchQueryError> {
        let engine = self.full_engine().map_err(WorkbenchQueryError::from)?;
        let (metadata, registry, availability, _captures, selected, state_sha256) =
            engine.lexical_link_rows();
        let source_state = availability
            .iter()
            .map(|row| (row.source.id.clone(), row.selectable, row.reason))
            .collect::<Vec<_>>();
        let resolved_sources = lexical::resolve_sources(registry, &request.options, &source_state)
            .map_err(WorkbenchQueryError::from)?;
        let rows = build_link_rows(metadata, registry, selected, None)
            .map_err(WorkbenchQueryError::from)?;
        Ok(PreparedFullLinks {
            rows,
            resolved_sources,
            availability: availability.to_vec(),
            state_sha256: state_sha256.to_owned(),
            code_notices: metadata.code_notices.clone(),
        })
    }
    fn resolve_full_link_target(
        &mut self,
        target: &vlajmaji_lexicon::RelationTarget,
    ) -> Result<WorkbenchReferenceFullResult, WorkbenchQueryError> {
        self.resolve_configured_reference_full_inner(target)
    }
    fn full_link_hit(
        &mut self,
        assertion_id: &EntityId,
    ) -> Result<DictionaryHit, WorkbenchQueryError> {
        let engine = self.full_engine()?;
        let (metadata, ..) = engine.lexical_link_rows();
        let assertion = metadata
            .assertions
            .iter()
            .find(|row| &row.id == assertion_id)
            .ok_or_else(|| {
                WorkbenchQueryError::from(WorkbenchLexicalError::Integrity {
                    detail: format!("Links from assertion {assertion_id} disappeared"),
                })
            })?;
        engine
            .lexical_link_hit(assertion)
            .map_err(WorkbenchQueryError::from)
    }
    fn full_link_evidence(
        &mut self,
        evidence: &vlajmaji_lexicon::SourceEvidence,
        source: &SourceId,
    ) -> Result<WorkbenchFormEvidence, WorkbenchQueryError> {
        self.full_engine()?
            .lexical_link_evidence(evidence, source)
            .map_err(WorkbenchQueryError::from)
    }
    fn full_link_context(
        &mut self,
        context_id: &EntityId,
    ) -> Result<ContextRecord, WorkbenchQueryError> {
        self.full_engine()?
            .lexical_link_context(context_id)
            .map_err(WorkbenchQueryError::from)
    }
    fn full_link_revision(
        &mut self,
        revision_id: &EntityId,
    ) -> Result<vlajmaji_lexicon::OwnedRevisionRecord, WorkbenchQueryError> {
        self.full_engine()?
            .lexical_link_revision(revision_id)
            .map_err(WorkbenchQueryError::from)
    }
}

pub(super) trait CompactLinksRuntime {
    fn prepare_link_detail(
        &mut self,
        identity: &WorkbenchLinkEvidenceId,
    ) -> Result<(LinkRows, Vec<SourceId>), WorkbenchQueryError>;
    fn prepare_compact_links(
        &mut self,
        request: &WorkbenchLexicalRequest,
    ) -> Result<PreparedCompactLinks, WorkbenchQueryError>;
    fn resolve_compact_link_target(
        &mut self,
        target: &vlajmaji_lexicon::RelationTarget,
    ) -> Result<WorkbenchReferenceCompactResult, WorkbenchQueryError>;
    fn compact_link_hit(
        &mut self,
        assertion_id: &EntityId,
    ) -> Result<DictionaryCompactHit, WorkbenchQueryError>;
    fn compact_link_evidence(
        &mut self,
        evidence: &vlajmaji_lexicon::SourceEvidence,
        source: &SourceId,
    ) -> Result<WorkbenchFormEvidence, WorkbenchQueryError>;
    fn compact_link_context(
        &mut self,
        context_id: &EntityId,
    ) -> Result<ContextRecord, WorkbenchQueryError>;
    fn compact_link_revision(
        &mut self,
        revision_id: &EntityId,
    ) -> Result<vlajmaji_lexicon::OwnedRevisionRecord, WorkbenchQueryError>;
}
impl CompactLinksRuntime for WorkbenchSearchEngine {
    fn prepare_link_detail(
        &mut self,
        identity: &WorkbenchLinkEvidenceId,
    ) -> Result<(LinkRows, Vec<SourceId>), WorkbenchQueryError> {
        prepare_compact_link_detail(self.compact_engine()?, identity)
    }
    fn prepare_compact_links(
        &mut self,
        request: &WorkbenchLexicalRequest,
    ) -> Result<PreparedCompactLinks, WorkbenchQueryError> {
        let engine = self
            .compact_engine()
            .map_err(WorkbenchLexicalError::from)
            .map_err(WorkbenchQueryError::from)?;
        let (metadata, registry, availability, _captures, selected, state_sha256, snapshot_sha256) =
            engine.lexical_link_rows();
        let source_state = availability
            .iter()
            .map(|row| (row.source.id.clone(), row.selectable, row.reason))
            .collect::<Vec<_>>();
        let resolved_sources = lexical::resolve_sources(registry, &request.options, &source_state)
            .map_err(WorkbenchQueryError::from)?;
        let rows = build_link_rows(metadata, registry, selected, None)
            .map_err(WorkbenchQueryError::from)?;
        Ok(PreparedCompactLinks {
            rows,
            resolved_sources,
            availability: availability.to_vec(),
            state_sha256: state_sha256.to_owned(),
            snapshot_sha256: snapshot_sha256.to_owned(),
            code_notices: metadata.code_notices.clone(),
        })
    }
    fn resolve_compact_link_target(
        &mut self,
        target: &vlajmaji_lexicon::RelationTarget,
    ) -> Result<WorkbenchReferenceCompactResult, WorkbenchQueryError> {
        self.resolve_reference_compact_inner(target)
    }
    fn compact_link_hit(
        &mut self,
        assertion_id: &EntityId,
    ) -> Result<DictionaryCompactHit, WorkbenchQueryError> {
        let engine = self.compact_engine()?;
        let (metadata, ..) = engine.lexical_link_rows();
        let assertion = metadata
            .assertions
            .iter()
            .find(|row| &row.id == assertion_id)
            .ok_or_else(|| {
                WorkbenchQueryError::from(WorkbenchLexicalError::Integrity {
                    detail: format!("Links from assertion {assertion_id} disappeared"),
                })
            })?;
        engine
            .lexical_link_hit(assertion)
            .map_err(WorkbenchQueryError::from)
    }
    fn compact_link_evidence(
        &mut self,
        evidence: &vlajmaji_lexicon::SourceEvidence,
        source: &SourceId,
    ) -> Result<WorkbenchFormEvidence, WorkbenchQueryError> {
        self.compact_engine()?
            .lexical_link_evidence(evidence, source)
            .map_err(WorkbenchQueryError::from)
    }
    fn compact_link_context(
        &mut self,
        context_id: &EntityId,
    ) -> Result<ContextRecord, WorkbenchQueryError> {
        self.compact_engine()?
            .lexical_link_context(context_id)
            .map_err(WorkbenchQueryError::from)
    }
    fn compact_link_revision(
        &mut self,
        revision_id: &EntityId,
    ) -> Result<vlajmaji_lexicon::OwnedRevisionRecord, WorkbenchQueryError> {
        self.compact_engine()?
            .lexical_link_revision(revision_id)
            .map_err(WorkbenchQueryError::from)
    }
}
impl CompactLinksRuntime for ConfiguredWorkbenchSearchEngine {
    fn prepare_link_detail(
        &mut self,
        identity: &WorkbenchLinkEvidenceId,
    ) -> Result<(LinkRows, Vec<SourceId>), WorkbenchQueryError> {
        prepare_compact_link_detail(
            self.compact_engine().map_err(WorkbenchQueryError::from)?,
            identity,
        )
    }
    fn prepare_compact_links(
        &mut self,
        request: &WorkbenchLexicalRequest,
    ) -> Result<PreparedCompactLinks, WorkbenchQueryError> {
        let engine = self.compact_engine().map_err(WorkbenchQueryError::from)?;
        let (metadata, registry, availability, _captures, selected, state_sha256, snapshot_sha256) =
            engine.lexical_link_rows();
        let source_state = availability
            .iter()
            .map(|row| (row.source.id.clone(), row.selectable, row.reason))
            .collect::<Vec<_>>();
        let resolved_sources = lexical::resolve_sources(registry, &request.options, &source_state)
            .map_err(WorkbenchQueryError::from)?;
        let rows = build_link_rows(metadata, registry, selected, None)
            .map_err(WorkbenchQueryError::from)?;
        Ok(PreparedCompactLinks {
            rows,
            resolved_sources,
            availability: availability.to_vec(),
            state_sha256: state_sha256.to_owned(),
            snapshot_sha256: snapshot_sha256.to_owned(),
            code_notices: metadata.code_notices.clone(),
        })
    }
    fn resolve_compact_link_target(
        &mut self,
        target: &vlajmaji_lexicon::RelationTarget,
    ) -> Result<WorkbenchReferenceCompactResult, WorkbenchQueryError> {
        self.resolve_configured_reference_compact_inner(target)
    }
    fn compact_link_hit(
        &mut self,
        assertion_id: &EntityId,
    ) -> Result<DictionaryCompactHit, WorkbenchQueryError> {
        let engine = self.compact_engine()?;
        let (metadata, ..) = engine.lexical_link_rows();
        let assertion = metadata
            .assertions
            .iter()
            .find(|row| &row.id == assertion_id)
            .ok_or_else(|| {
                WorkbenchQueryError::from(WorkbenchLexicalError::Integrity {
                    detail: format!("Links from assertion {assertion_id} disappeared"),
                })
            })?;
        engine
            .lexical_link_hit(assertion)
            .map_err(WorkbenchQueryError::from)
    }
    fn compact_link_evidence(
        &mut self,
        evidence: &vlajmaji_lexicon::SourceEvidence,
        source: &SourceId,
    ) -> Result<WorkbenchFormEvidence, WorkbenchQueryError> {
        self.compact_engine()?
            .lexical_link_evidence(evidence, source)
            .map_err(WorkbenchQueryError::from)
    }
    fn compact_link_context(
        &mut self,
        context_id: &EntityId,
    ) -> Result<ContextRecord, WorkbenchQueryError> {
        self.compact_engine()
            .map_err(WorkbenchQueryError::from)?
            .lexical_link_context(context_id)
            .map_err(WorkbenchQueryError::from)
    }
    fn compact_link_revision(
        &mut self,
        revision_id: &EntityId,
    ) -> Result<vlajmaji_lexicon::OwnedRevisionRecord, WorkbenchQueryError> {
        self.compact_engine()
            .map_err(WorkbenchQueryError::from)?
            .lexical_link_revision(revision_id)
            .map_err(WorkbenchQueryError::from)
    }
}

fn prepare_compact_link_detail(
    engine: &crate::DictionaryCompactSearchEngine,
    identity: &WorkbenchLinkEvidenceId,
) -> Result<(LinkRows, Vec<SourceId>), WorkbenchQueryError> {
    let (metadata, registry, availability, _captures, selected, ..) = engine.lexical_link_rows();
    match identity {
        WorkbenchLinkEvidenceId::TypedRelation { relation_id } => {
            let relation = metadata
                .relations
                .iter()
                .find(|row| &row.id == relation_id)
                .ok_or_else(|| WorkbenchQueryError::from(WorkbenchLexicalError::StaleLinkDetail))?;
            if let Some(evidence) = &relation.evidence {
                let publisher = occurrence_source(evidence, registry).ok_or_else(|| {
                    WorkbenchQueryError::from(WorkbenchLexicalError::Integrity {
                        detail: format!("relation {relation_id} evidence has no source owner"),
                    })
                })?;
                engine
                    .lexical_link_evidence(evidence, &publisher.id)
                    .map_err(WorkbenchQueryError::from)?;
            }
            if let Some(revision_id) = &relation.editorial_revision {
                engine
                    .lexical_link_revision(revision_id)
                    .map_err(WorkbenchQueryError::from)?;
            }
        }
        WorkbenchLinkEvidenceId::SourceClause { context_id, .. } => {
            let context = metadata
                .contexts
                .iter()
                .find(|row| &row.id == context_id)
                .ok_or_else(|| WorkbenchQueryError::from(WorkbenchLexicalError::StaleLinkDetail))?;
            let publisher = occurrence_source(&context.evidence, registry).ok_or_else(|| {
                WorkbenchQueryError::from(WorkbenchLexicalError::Integrity {
                    detail: format!("Note context {context_id} has no evidence source owner"),
                })
            })?;
            engine
                .lexical_link_context(context_id)
                .map_err(WorkbenchQueryError::from)?;
            engine
                .lexical_link_evidence(&context.evidence, &publisher.id)
                .map_err(WorkbenchQueryError::from)?;
        }
    }
    let identity_exists = match identity {
        WorkbenchLinkEvidenceId::TypedRelation { relation_id } => {
            metadata.relations.iter().any(|row| &row.id == relation_id)
        }
        WorkbenchLinkEvidenceId::SourceClause { context_id, .. } => {
            metadata.contexts.iter().any(|row| &row.id == context_id)
        }
    };
    if !identity_exists {
        return Err(WorkbenchQueryError::from(
            WorkbenchLexicalError::StaleLinkDetail,
        ));
    }
    let rows = build_link_rows(metadata, registry, selected, Some(identity))
        .map_err(WorkbenchQueryError::from)?;
    if !rows.evidence.iter().any(|row| &row.identity == identity) {
        return Err(WorkbenchQueryError::from(
            WorkbenchLexicalError::StaleLinkDetail,
        ));
    }
    let selectable = availability
        .iter()
        .filter(|row| row.selectable)
        .map(|row| row.source.id.clone())
        .collect();
    Ok((rows, selectable))
}

fn lowercase_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn detail_link_record(
    runtime: &mut impl CompactLinksRuntime,
    draft: &LinkEvidenceDraft,
) -> Result<WorkbenchLinkRecord, WorkbenchQueryError> {
    Ok(WorkbenchLinkRecord {
        identity: draft.identity.clone(),
        publisher: draft.publisher.clone(),
        authority: draft.authority.clone(),
        origin_source: draft.origin_source.clone(),
        relation: draft.relation.clone(),
        context: draft
            .context_id
            .as_ref()
            .map(|id| runtime.compact_link_context(id))
            .transpose()?,
        token: draft.token.clone(),
        editorial_revision: draft
            .revision_id
            .as_ref()
            .map(|id| runtime.compact_link_revision(id))
            .transpose()?,
        evidence: draft
            .source_evidence
            .as_ref()
            .map(|evidence| runtime.compact_link_evidence(evidence, &draft.publisher.id))
            .transpose()?,
    })
}

pub(super) fn build_compact_link_detail(
    runtime: &mut impl CompactLinksRuntime,
    reference: &WorkbenchLinkDetailRef,
    binding: &str,
) -> Result<
    (
        WorkbenchLinkRecord,
        WorkbenchReferenceCompactResult,
        Vec<WorkbenchLinkTargetCompact>,
    ),
    WorkbenchQueryError,
> {
    if let WorkbenchLinkEvidenceId::SourceClause {
        context_id,
        byte_start,
        byte_end,
        rule_version,
        target_spelling,
    } = &reference.identity
        && (context_id.as_str().is_empty()
            || byte_end.checked_sub(*byte_start) != target_spelling.len().checked_add(2)
            || rule_version != "see-also-braces-v2"
            || target_spelling.is_empty())
    {
        return Err(WorkbenchQueryError::from(
            WorkbenchLexicalError::MalformedLinkDetail {
                detail: "source-clause identity has invalid parser fields".into(),
            },
        ));
    }
    if reference.version != 1
        || reference.digest_policy != "link-evidence-serde-json-v1"
        || reference.role != "link_evidence"
        || !lowercase_sha256(&reference.binding_sha256)
        || !lowercase_sha256(&reference.record_sha256)
    {
        return Err(WorkbenchQueryError::from(
            WorkbenchLexicalError::MalformedLinkDetail {
                detail: "unsupported key version, policy, role, or digest syntax".into(),
            },
        ));
    }
    if reference.binding_sha256 != binding {
        return Err(WorkbenchQueryError::from(
            WorkbenchLexicalError::StaleLinkDetail,
        ));
    }
    let (rows, selectable) = runtime.prepare_link_detail(&reference.identity)?;
    let draft = rows
        .evidence
        .iter()
        .find(|row| row.identity == reference.identity)
        .ok_or_else(|| WorkbenchQueryError::from(WorkbenchLexicalError::StaleLinkDetail))?;
    let record = detail_link_record(runtime, draft)?;
    if draft.publisher.id != reference.publisher || draft.record_sha256 != reference.record_sha256 {
        return Err(WorkbenchQueryError::from(
            WorkbenchLexicalError::StaleLinkDetail,
        ));
    }
    if !selectable.contains(&draft.publisher.id) {
        return Err(WorkbenchQueryError::from(
            WorkbenchLexicalError::InvalidRequest {
                detail: format!(
                    "link evidence publisher {} is not selectable",
                    draft.publisher.id
                ),
            },
        ));
    }
    let origin_target = vlajmaji_lexicon::RelationTarget::Local(draft.origin.clone());
    let origin_resolution = runtime.resolve_compact_link_target(&origin_target)?;
    let mut target_observations = Vec::new();
    if let Some(token) = &draft.token {
        let mut entries = rows
            .clause_entries
            .values()
            .filter(|entry| {
                matches!(&entry.spelling, vlajmaji_lexicon::Presence::Value(value) if value == &token.target_spelling)
            })
            .cloned()
            .collect::<Vec<_>>();
        entries.sort_by(|left, right| {
            let left_source = rows.entries.get(&left.id).map(|row| &row.source);
            let right_source = rows.entries.get(&right.id).map(|row| &row.source);
            (left_source, entry_spelling(left), &left.id).cmp(&(
                right_source,
                entry_spelling(right),
                &right.id,
            ))
        });
        if entries.is_empty() {
            target_observations.push(
                compact_target_observation(runtime, &rows, &selectable, &draft.target, None)?.0,
            );
        } else {
            for entry in entries {
                let target =
                    vlajmaji_lexicon::RelationTarget::Local(LocalRef::Entry(entry.id.clone()));
                target_observations.push(
                    compact_target_observation(runtime, &rows, &selectable, &target, Some(entry))?
                        .0,
                );
            }
        }
    } else {
        target_observations
            .push(compact_target_observation(runtime, &rows, &selectable, &draft.target, None)?.0);
    }
    Ok((record, origin_resolution, target_observations))
}

fn operational_target_reason(
    target: &vlajmaji_lexicon::RelationTarget,
    status: crate::ReferenceStatus,
    has_payload: bool,
    candidates: &[WorkbenchLinkCandidate],
) -> Option<String> {
    if candidates
        .iter()
        .any(|row| row.traversal_status == "traversable")
    {
        return None;
    }
    match target {
        vlajmaji_lexicon::RelationTarget::Unknown => Some("unknown".into()),
        vlajmaji_lexicon::RelationTarget::Unresolved(_) => Some("unresolved".into()),
        _ if status == crate::ReferenceStatus::NoMatch => Some("no_match".into()),
        _ if !has_payload => Some("no_admitted_payload".into()),
        _ if candidates
            .iter()
            .all(|row| row.reason.as_deref() == Some("unverified_identity"))
            && candidates
                .iter()
                .any(|row| row.reason.as_deref() == Some("unverified_identity")) =>
        {
            Some("unverified_identity".into())
        }
        _ if candidates
            .iter()
            .any(|row| row.reason.as_deref() == Some("requested_source")) =>
        {
            Some("requested_source".into())
        }
        _ => Some("no_current_eligible_assertion".into()),
    }
}

fn candidate_from_row(
    assertion: &vlajmaji_lexicon::AssertionRecord,
    source: &SourceId,
    rows: &LinkRows,
    requested_sources: &[SourceId],
    confirmed: bool,
) -> WorkbenchLinkCandidate {
    let current = rows.assertions.contains_key(&assertion.id)
        && assertion.eligibility == vlajmaji_lexicon::Eligibility::Lexical;
    let (traversal_status, reason) = if !confirmed {
        ("not_traversable", Some("unverified_identity"))
    } else if !current {
        ("not_traversable", Some("no_current_eligible_assertion"))
    } else if !requested_sources.contains(source) {
        ("not_traversable", Some("requested_source"))
    } else {
        ("traversable", None)
    };
    WorkbenchLinkCandidate {
        assertion_id: assertion.id.clone(),
        source_id: source.clone(),
        traversal_status: traversal_status.into(),
        reason: reason.map(str::to_owned),
    }
}

fn full_target_observation(
    runtime: &mut impl FullLinksRuntime,
    rows: &LinkRows,
    requested_sources: &[SourceId],
    target: &vlajmaji_lexicon::RelationTarget,
    entry: Option<EntryRecord>,
) -> Result<
    (
        WorkbenchLinkTargetFull,
        Vec<vlajmaji_lexicon::AssertionRecord>,
    ),
    WorkbenchQueryError,
> {
    let resolution = runtime.resolve_full_link_target(target)?;
    let mut candidates = Vec::new();
    let mut hops = Vec::new();
    if let Some(payload) = &resolution.payload {
        for candidate in &payload.confirmed {
            let row = &candidate.assertion;
            let output =
                candidate_from_row(row, &candidate.entry.source, rows, requested_sources, true);
            if output.traversal_status == "traversable" {
                hops.push(row.clone());
            }
            candidates.push(output);
        }
        for candidate in &payload.unverified {
            candidates.push(candidate_from_row(
                &candidate.assertion,
                &candidate.entry.source,
                rows,
                requested_sources,
                false,
            ));
        }
    } else if matches!(target, vlajmaji_lexicon::RelationTarget::Local(_)) {
        // Raw retained identities are observations only. Traversal hops are
        // sourced exclusively from the format-local resolver's confirmed
        // payload above.
    }
    let reason = operational_target_reason(
        target,
        resolution.status,
        resolution.payload.is_some(),
        &candidates,
    );
    let traversal_status = if reason.is_none() {
        "traversable"
    } else {
        "not_traversable"
    };
    Ok((
        WorkbenchLinkTargetFull {
            entry,
            resolution,
            candidates,
            traversal_status: traversal_status.into(),
            reason,
        },
        hops,
    ))
}

fn compact_target_observation(
    runtime: &mut impl CompactLinksRuntime,
    rows: &LinkRows,
    requested_sources: &[SourceId],
    target: &vlajmaji_lexicon::RelationTarget,
    entry: Option<EntryRecord>,
) -> Result<
    (
        WorkbenchLinkTargetCompact,
        Vec<vlajmaji_lexicon::AssertionRecord>,
    ),
    WorkbenchQueryError,
> {
    let resolution = runtime.resolve_compact_link_target(target)?;
    let mut candidates = Vec::new();
    let mut hops = Vec::new();
    if let Some(payload) = &resolution.payload {
        for candidate in &payload.confirmed {
            let row = &candidate.assertion;
            let output =
                candidate_from_row(row, &candidate.entry.source, rows, requested_sources, true);
            if output.traversal_status == "traversable" {
                hops.push(row.clone());
            }
            candidates.push(output);
        }
        for candidate in &payload.unverified {
            candidates.push(candidate_from_row(
                &candidate.assertion,
                &candidate.entry.source,
                rows,
                requested_sources,
                false,
            ));
        }
    } else if matches!(target, vlajmaji_lexicon::RelationTarget::Local(_)) {
        // Metadata-only observations never become traversal hops.
    }
    let reason = operational_target_reason(
        target,
        resolution.status,
        resolution.payload.is_some(),
        &candidates,
    );
    let traversal_status = if reason.is_none() {
        "traversable"
    } else {
        "not_traversable"
    };
    Ok((
        WorkbenchLinkTargetCompact {
            entry,
            resolution,
            candidates,
            traversal_status: traversal_status.into(),
            reason,
        },
        hops,
    ))
}

fn from_target(row: &LinkEvidenceDraft, orientation: &str) -> vlajmaji_lexicon::RelationTarget {
    if orientation == "reverse" {
        return row.target.clone();
    }
    vlajmaji_lexicon::RelationTarget::Local(row.origin.clone())
}

fn full_link_record(
    runtime: &mut impl FullLinksRuntime,
    draft: &LinkEvidenceDraft,
) -> Result<WorkbenchLinkRecord, WorkbenchQueryError> {
    let evidence = draft
        .source_evidence
        .as_ref()
        .map(|row| runtime.full_link_evidence(row, &draft.publisher.id))
        .transpose()?;
    let context = draft
        .context_id
        .as_ref()
        .map(|id| runtime.full_link_context(id))
        .transpose()?;
    let revision = draft
        .revision_id
        .as_ref()
        .map(|id| runtime.full_link_revision(id))
        .transpose()?;
    Ok(WorkbenchLinkRecord {
        identity: draft.identity.clone(),
        publisher: draft.publisher.clone(),
        authority: draft.authority.clone(),
        origin_source: draft.origin_source.clone(),
        relation: draft.relation.clone(),
        context,
        token: draft.token.clone(),
        editorial_revision: revision,
        evidence,
    })
}

fn compact_link_record(draft: &LinkEvidenceDraft) -> WorkbenchCompactLinkRecord {
    let (relation_kind, direction, scope) =
        draft.relation.as_ref().map_or((None, None, None), |row| {
            (Some(row.kind.clone()), Some(row.direction), Some(row.scope))
        });
    WorkbenchCompactLinkRecord {
        identity: draft.identity.clone(),
        publisher: draft.publisher.id.clone(),
        authority: draft.authority.clone(),
        origin: draft.origin.clone(),
        relation_kind,
        direction,
        scope,
    }
}

fn detail_ref(
    binding: &str,
    draft: &LinkEvidenceDraft,
) -> Result<WorkbenchLinkDetailRef, WorkbenchQueryError> {
    let row_sha = &draft.record_sha256;
    Ok(WorkbenchLinkDetailRef {
        version: 1,
        digest_policy: "link-evidence-serde-json-v1".into(),
        role: "link_evidence".into(),
        binding_sha256: binding.into(),
        publisher: draft.publisher.id.clone(),
        identity: draft.identity.clone(),
        record_sha256: row_sha.clone(),
    })
}

fn evidence_kind_rank(identity: &WorkbenchLinkEvidenceId) -> u8 {
    match identity {
        WorkbenchLinkEvidenceId::TypedRelation { .. } => 0,
        WorkbenchLinkEvidenceId::SourceClause { .. } => 1,
    }
}

fn orientation_rank(orientation: &str) -> u8 {
    match orientation {
        "forward" => 0,
        _ => 1,
    }
}

fn position_for(rows: &LinkRows, step: &RawLinkStep) -> WorkbenchLexicalPosition {
    let evidence = &rows.evidence[step.evidence];
    WorkbenchLexicalPosition::Links {
        source: evidence.publisher.id.clone(),
        depth: step.depth,
        evidence_kind_rank: evidence_kind_rank(&evidence.identity),
        evidence: evidence.identity.clone(),
        orientation_rank: orientation_rank(step.orientation),
        from_assertion_id: step.from.id.clone(),
    }
}

fn position_after_cursor(
    rows: &LinkRows,
    steps: &[RawLinkStep],
    request: &WorkbenchLexicalRequest,
    resolved: &[SourceId],
    format: &str,
    binding: &str,
) -> Result<usize, WorkbenchQueryError> {
    let Some(cursor) = &request.options.cursor else {
        return Ok(0);
    };
    let position = lexical::decode_cursor(cursor, request, resolved, format, binding)
        .map_err(WorkbenchQueryError::from)?;
    if !matches!(position, WorkbenchLexicalPosition::Links { .. }) {
        return Err(WorkbenchQueryError::from(
            WorkbenchLexicalError::InvalidCursorPosition,
        ));
    }
    let Some(index) = steps
        .iter()
        .position(|step| position_for(rows, step) == position)
    else {
        return Err(WorkbenchQueryError::from(
            WorkbenchLexicalError::InvalidCursorPosition,
        ));
    };
    Ok(index + 1)
}

fn link_totals(
    rows: &LinkRows,
    steps: &[RawLinkStep],
    visited: &BTreeSet<EntityId>,
) -> (WorkbenchLinksTotals, Vec<WorkbenchLinksSourceTotal>) {
    let mut sources = BTreeMap::<
        SourceId,
        (
            SourceRecord,
            BTreeSet<WorkbenchLinkEvidenceId>,
            BTreeSet<EntityId>,
        ),
    >::new();
    let mut all_evidence = BTreeSet::new();
    let mut all_assertions = BTreeSet::new();
    for step in steps {
        let evidence = &rows.evidence[step.evidence];
        let row = sources
            .entry(evidence.publisher.id.clone())
            .or_insert_with(|| (evidence.publisher.clone(), BTreeSet::new(), BTreeSet::new()));
        row.1.insert(evidence.identity.clone());
        row.2.insert(step.from.id.clone());
        all_evidence.insert(evidence.identity.clone());
        all_assertions.insert(step.from.id.clone());
        for id in &step.target_assertions {
            row.2.insert(id.clone());
            all_assertions.insert(id.clone());
        }
    }
    let source_totals = sources
        .into_values()
        .map(|(source, evidence, assertions)| {
            let records = steps
                .iter()
                .filter(|step| rows.evidence[step.evidence].publisher.id == source.id)
                .count();
            WorkbenchLinksSourceTotal {
                source,
                records,
                distinct_evidence: evidence.len(),
                associated_assertions: assertions.len(),
            }
        })
        .collect::<Vec<_>>();
    let totals = WorkbenchLinksTotals {
        unit: "traversal_steps".into(),
        records: steps.len(),
        source_groups: source_totals.len(),
        distinct_evidence: all_evidence.len(),
        associated_assertions: all_assertions.len(),
        visited_assertions: visited.len(),
    };
    (totals, source_totals)
}

fn page_end(start: usize, count: Option<usize>, len: usize) -> usize {
    count.map_or(len, |count| start.saturating_add(count).min(len))
}

fn canonical_assertion_ids(rows: &LinkRows, ids: &BTreeSet<EntityId>) -> Vec<EntityId> {
    let mut ids = ids
        .iter()
        .filter_map(|id| rows.assertions.get(id))
        .collect::<Vec<_>>();
    ids.sort_by(|left, right| {
        let left_entry = rows.entries.get(&left.entry);
        let right_entry = rows.entries.get(&right.entry);
        (
            left_entry.map(|entry| &entry.source),
            left_entry.map(entry_spelling_order),
            occurrence_tuple_hex(&left.occurrence),
            &left.id,
        )
            .cmp(&(
                right_entry.map(|entry| &entry.source),
                right_entry.map(entry_spelling_order),
                occurrence_tuple_hex(&right.occurrence),
                &right.id,
            ))
    });
    ids.into_iter().map(|row| row.id.clone()).collect()
}

fn full_step(
    runtime: &mut impl FullLinksRuntime,
    rows: &LinkRows,
    requested_sources: &[SourceId],
    raw: &RawLinkStep,
    cache: &mut Vec<(
        vlajmaji_lexicon::RelationTarget,
        WorkbenchLinkTargetFull,
        Vec<vlajmaji_lexicon::AssertionRecord>,
    )>,
) -> Result<WorkbenchLinkFullStep, WorkbenchQueryError> {
    let draft = &rows.evidence[raw.evidence];
    let from = runtime.full_link_hit(&raw.from.id)?;
    let record = full_link_record(runtime, draft)?;
    let origin_target = from_target(draft, raw.orientation);
    let origin_resolution = if let Some((_, observation, _)) =
        cache.iter().find(|(target, _, _)| target == &origin_target)
    {
        observation.resolution.clone()
    } else {
        let (observation, hits) =
            full_target_observation(runtime, rows, requested_sources, &origin_target, None)?;
        let resolution = observation.resolution.clone();
        cache.push((origin_target, observation, hits));
        resolution
    };
    let traversal_status = if raw
        .observations
        .iter()
        .any(|reference| cache[reference.cache_index].1.traversal_status == "traversable")
    {
        "traversable"
    } else {
        "observed_only"
    };
    let target_observations = raw
        .observations
        .iter()
        .map(|reference| {
            let mut observation = cache[reference.cache_index].1.clone();
            observation.entry = reference.entry.clone();
            observation
        })
        .collect();
    Ok(WorkbenchLinkFullStep {
        identity: WorkbenchLinkStepId {
            evidence: draft.identity.clone(),
            orientation: raw.orientation.into(),
            from_assertion_id: raw.from.id.clone(),
        },
        depth: raw.depth,
        from,
        record,
        origin_resolution,
        target_observations,
        target_assertion_ids: canonical_assertion_ids(rows, &raw.target_assertions),
        traversal_status: traversal_status.into(),
    })
}

fn compact_step(
    runtime: &mut impl CompactLinksRuntime,
    rows: &LinkRows,
    requested_sources: &[SourceId],
    binding: &str,
    raw: &RawLinkStep,
    cache: &mut Vec<(
        vlajmaji_lexicon::RelationTarget,
        WorkbenchLinkTargetCompact,
        Vec<vlajmaji_lexicon::AssertionRecord>,
    )>,
) -> Result<WorkbenchLinkCompactStep, WorkbenchQueryError> {
    let draft = &rows.evidence[raw.evidence];
    let from = runtime.compact_link_hit(&raw.from.id)?;
    let record = compact_link_record(draft);
    let origin_target = from_target(draft, raw.orientation);
    let origin_resolution = if let Some((_, observation, _)) =
        cache.iter().find(|(target, _, _)| target == &origin_target)
    {
        observation.resolution.clone()
    } else {
        let (observation, hits) =
            compact_target_observation(runtime, rows, requested_sources, &origin_target, None)?;
        let resolution = observation.resolution.clone();
        cache.push((origin_target, observation, hits));
        resolution
    };
    let reference = detail_ref(binding, draft)?;
    let traversal_status = if raw
        .observations
        .iter()
        .any(|reference| cache[reference.cache_index].1.traversal_status == "traversable")
    {
        "traversable"
    } else {
        "observed_only"
    };
    let target_observations = raw
        .observations
        .iter()
        .map(|reference| {
            let mut observation = cache[reference.cache_index].1.clone();
            observation.entry = reference.entry.clone();
            observation
        })
        .collect();
    Ok(WorkbenchLinkCompactStep {
        identity: WorkbenchLinkStepId {
            evidence: draft.identity.clone(),
            orientation: raw.orientation.into(),
            from_assertion_id: raw.from.id.clone(),
        },
        depth: raw.depth,
        from,
        record,
        origin_resolution,
        target_observations,
        target_assertion_ids: canonical_assertion_ids(rows, &raw.target_assertions),
        traversal_status: traversal_status.into(),
        detail_ref: reference,
    })
}

fn build_full_links(
    runtime: &mut impl FullLinksRuntime,
    request: &WorkbenchLexicalRequest,
    binding: &str,
) -> Result<WorkbenchLinksFull, WorkbenchQueryError> {
    let (spelling, depth) = match &request.query {
        WorkbenchLexicalQuery::Links { spelling, depth } => (spelling.as_str(), *depth),
        _ => {
            return Err(WorkbenchQueryError::from(
                WorkbenchLexicalError::InvalidRequest {
                    detail: "Links query required".into(),
                },
            ));
        }
    };
    let prepared = runtime.prepare_full_links(request)?;
    let mut cached = Vec::<(
        vlajmaji_lexicon::RelationTarget,
        WorkbenchLinkTargetFull,
        Vec<vlajmaji_lexicon::AssertionRecord>,
    )>::new();
    let (seed_assertion_ids, mut steps, visited) = enumerate_link_steps(
        &prepared.rows,
        spelling,
        depth,
        &prepared.resolved_sources,
        |target, entry| {
            if let Some((index, (_, _, hits))) = cached
                .iter()
                .enumerate()
                .find(|(_, (known, _, _))| known == target)
            {
                return Ok((index, hits.clone()));
            }
            let (observation, hits) = full_target_observation(
                runtime,
                &prepared.rows,
                &prepared.resolved_sources,
                target,
                entry,
            )?;
            let index = cached.len();
            cached.push((target.clone(), observation, hits.clone()));
            Ok((index, hits))
        },
    )?;
    steps.sort_by(|a, b| compare_raw_steps(&prepared.rows, a, b));
    let (totals, source_totals) = link_totals(&prepared.rows, &steps, &visited);
    let start = position_after_cursor(
        &prepared.rows,
        &steps,
        request,
        &prepared.resolved_sources,
        "full",
        binding,
    )?;
    let end = page_end(start, request.options.page_size, steps.len());
    let mut grouped =
        BTreeMap::<SourceId, (SourceRecord, usize, Vec<WorkbenchLinkFullStep>)>::new();
    let full_counts = source_totals
        .iter()
        .map(|row| (row.source.id.clone(), row.records))
        .collect::<BTreeMap<_, _>>();
    for raw in &steps[start..end] {
        let source = prepared.rows.evidence[raw.evidence].publisher.clone();
        let full_count = *full_counts.get(&source.id).unwrap_or(&0);
        grouped
            .entry(source.id.clone())
            .or_insert_with(|| (source.clone(), full_count, Vec::new()))
            .2
            .push(full_step(
                runtime,
                &prepared.rows,
                &prepared.resolved_sources,
                raw,
                &mut cached,
            )?);
    }
    let groups = grouped
        .into_values()
        .map(|(source, full_count, hits)| WorkbenchLinksFullGroup {
            returned_count: hits.len(),
            partial: hits.len() < full_count,
            source,
            full_count,
            hits,
        })
        .collect();
    let next_cursor = if end < steps.len() && end > start {
        Some(
            lexical::encode_cursor(
                request,
                &prepared.resolved_sources,
                "full",
                binding,
                position_for(&prepared.rows, &steps[end - 1]),
            )
            .map_err(WorkbenchQueryError::from)?,
        )
    } else {
        None
    };
    Ok(WorkbenchLinksFull {
        policy: "bounded-links-v1".into(),
        state_sha256: prepared.state_sha256,
        query: request.query.clone(),
        resolved_sources: prepared.resolved_sources,
        seed_assertion_ids,
        totals,
        source_totals,
        availability: prepared.availability,
        groups,
        next_cursor,
        code_notices: prepared.code_notices,
    })
}

fn build_compact_links(
    runtime: &mut impl CompactLinksRuntime,
    request: &WorkbenchLexicalRequest,
    binding: &str,
) -> Result<WorkbenchLinksCompact, WorkbenchQueryError> {
    let (spelling, depth) = match &request.query {
        WorkbenchLexicalQuery::Links { spelling, depth } => (spelling.as_str(), *depth),
        _ => {
            return Err(WorkbenchQueryError::from(
                WorkbenchLexicalError::InvalidRequest {
                    detail: "Links query required".into(),
                },
            ));
        }
    };
    let prepared = runtime.prepare_compact_links(request)?;
    let mut cached = Vec::<(
        vlajmaji_lexicon::RelationTarget,
        WorkbenchLinkTargetCompact,
        Vec<vlajmaji_lexicon::AssertionRecord>,
    )>::new();
    let (seed_assertion_ids, mut steps, visited) = enumerate_link_steps(
        &prepared.rows,
        spelling,
        depth,
        &prepared.resolved_sources,
        |target, entry| {
            if let Some((index, (_, _, hits))) = cached
                .iter()
                .enumerate()
                .find(|(_, (known, _, _))| known == target)
            {
                return Ok((index, hits.clone()));
            }
            let (observation, hits) = compact_target_observation(
                runtime,
                &prepared.rows,
                &prepared.resolved_sources,
                target,
                entry,
            )?;
            let index = cached.len();
            cached.push((target.clone(), observation, hits.clone()));
            Ok((index, hits))
        },
    )?;
    steps.sort_by(|a, b| compare_raw_steps(&prepared.rows, a, b));
    let (totals, source_totals) = link_totals(&prepared.rows, &steps, &visited);
    let start = position_after_cursor(
        &prepared.rows,
        &steps,
        request,
        &prepared.resolved_sources,
        "compact",
        binding,
    )?;
    let end = page_end(start, request.options.page_size, steps.len());
    let mut grouped =
        BTreeMap::<SourceId, (SourceRecord, usize, Vec<WorkbenchLinkCompactStep>)>::new();
    let full_counts = source_totals
        .iter()
        .map(|row| (row.source.id.clone(), row.records))
        .collect::<BTreeMap<_, _>>();
    for raw in &steps[start..end] {
        let source = prepared.rows.evidence[raw.evidence].publisher.clone();
        let full_count = *full_counts.get(&source.id).unwrap_or(&0);
        grouped
            .entry(source.id.clone())
            .or_insert_with(|| (source.clone(), full_count, Vec::new()))
            .2
            .push(compact_step(
                runtime,
                &prepared.rows,
                &prepared.resolved_sources,
                binding,
                raw,
                &mut cached,
            )?);
    }
    let groups = grouped
        .into_values()
        .map(|(source, full_count, hits)| WorkbenchLinksCompactGroup {
            returned_count: hits.len(),
            partial: hits.len() < full_count,
            source,
            full_count,
            hits,
        })
        .collect();
    let next_cursor = if end < steps.len() && end > start {
        Some(
            lexical::encode_cursor(
                request,
                &prepared.resolved_sources,
                "compact",
                binding,
                position_for(&prepared.rows, &steps[end - 1]),
            )
            .map_err(WorkbenchQueryError::from)?,
        )
    } else {
        None
    };
    Ok(WorkbenchLinksCompact {
        policy: "bounded-links-v1".into(),
        state_sha256: prepared.state_sha256,
        query: request.query.clone(),
        resolved_sources: prepared.resolved_sources,
        seed_assertion_ids,
        totals,
        source_totals,
        availability: prepared.availability,
        groups,
        next_cursor,
        code_notices: prepared.code_notices,
        snapshot_sha256: prepared.snapshot_sha256,
    })
}

impl WorkbenchSearchEngine {
    pub(super) fn lexical_links_full_inner(
        &mut self,
        request: &WorkbenchLexicalRequest,
        binding: &str,
    ) -> Result<WorkbenchLinksFull, WorkbenchQueryError> {
        build_full_links(self, request, binding)
    }
    pub(super) fn lexical_links_compact_inner(
        &mut self,
        request: &WorkbenchLexicalRequest,
        binding: &str,
    ) -> Result<WorkbenchLinksCompact, WorkbenchQueryError> {
        build_compact_links(self, request, binding)
    }
}
impl ConfiguredWorkbenchSearchEngine {
    pub(super) fn lexical_links_full_inner(
        &mut self,
        request: &WorkbenchLexicalRequest,
        binding: &str,
    ) -> Result<WorkbenchLinksFull, WorkbenchQueryError> {
        build_full_links(self, request, binding)
    }
    pub(super) fn lexical_links_compact_inner(
        &mut self,
        request: &WorkbenchLexicalRequest,
        binding: &str,
    ) -> Result<WorkbenchLinksCompact, WorkbenchQueryError> {
        build_compact_links(self, request, binding)
    }
}

#[cfg(test)]
mod provenance_tests {
    use super::*;

    #[test]
    fn test_unverified_relations_reject_conflicting_provenance_authorities() {
        let relation_id = EntityId::new("unverified-provenance-control").unwrap();
        let other = RelationKind::Other("fixture-unknown".into());
        for (source_evidence, editorial_revision) in [(false, false), (true, false), (false, true)]
        {
            assert!(
                validate_relation_provenance(
                    &other,
                    source_evidence,
                    editorial_revision,
                    &relation_id,
                )
                .is_ok()
            );
        }
        assert!(matches!(
            validate_relation_provenance(&other, true, true, &relation_id),
            Err(crate::workbench::lexical::WorkbenchLexicalError::Integrity { .. })
        ));
    }
}
