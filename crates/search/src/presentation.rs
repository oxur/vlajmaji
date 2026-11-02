//! Pure, source-grouped terminal renderers for compact dictionary projections.
//!
//! These functions only format already projected values. They never open a
//! provider, load payload data, or change query selection.

use std::{collections::BTreeMap, fmt::Write as _};

use serde::Serialize;

use crate::{
    CollectionCompactQueryResponse, CollectionCompactQueryResult,
    CollectionCompactReferenceResponse, CollectionHitDetailResponse,
    DictionaryCompactExactResponse, DictionaryCompactQueryResult,
    DictionaryCompactReferenceAssertion, DictionaryCompactReferenceResolution,
    DictionaryHitDetailResponse, ReferenceStatus, WorkbenchCompactFormRecord, WorkbenchFormRecord,
    WorkbenchLexicalCompactResult, WorkbenchLinksCompact, WorkbenchReverseDefinitionCompact,
};

/// Render a standalone compact Exact or Resolve result as readable text.
#[must_use]
pub fn render_dictionary_compact(result: &DictionaryCompactQueryResult) -> String {
    match result {
        DictionaryCompactQueryResult::Exact(response) => render_exact(response, None),
        DictionaryCompactQueryResult::Resolve(response) => render_dictionary_resolve(response),
    }
}

/// Render a managed compact Exact or Resolve result as readable text.
#[must_use]
pub fn render_collection_compact(response: &CollectionCompactQueryResponse) -> String {
    match &response.result {
        CollectionCompactQueryResult::Exact(result) => {
            render_exact(result, Some((&response.snapshot, &response.collections)))
        }
        CollectionCompactQueryResult::Resolve(result) => {
            render_collection_resolve(result, &response.snapshot, &response.collections)
        }
    }
}

/// Render a standalone complete-hit detail response as readable text.
#[must_use]
pub fn render_dictionary_detail(response: &DictionaryHitDetailResponse) -> String {
    render_detail(response, None)
}

/// Render a managed complete-hit detail response as readable text.
#[must_use]
pub fn render_collection_detail(response: &CollectionHitDetailResponse) -> String {
    render_detail(
        &response.response,
        Some((
            &response.snapshot,
            &response.collections,
            &response.catalogue_matches,
        )),
    )
}

/// Render one already-projected compact lexical result without opening a view.
///
/// The output retains the query's source order, all pre-page counts, and every
/// row field needed to follow a returned assertion or Form to its complete
/// detail key. Complete Form evidence and computations remain available from
/// the separate Form-detail operation.
#[must_use]
pub fn render_workbench_lexical_compact(result: &WorkbenchLexicalCompactResult) -> String {
    let mut output = String::new();
    if let WorkbenchLexicalCompactResult::Links(response) = result {
        writeln!(output, "lexical_operation: {}", json(&"links")).unwrap();
        writeln!(output, "lexical_policy: {}", json(&response.policy)).unwrap();
        writeln!(output, "lexical_totals: {}", json(&response.totals)).unwrap();
        writeln!(
            output,
            "lexical_source_totals: {}",
            json(&response.source_totals)
        )
        .unwrap();
        writeln!(
            output,
            "lexical_next_cursor: {}",
            json(&response.next_cursor)
        )
        .unwrap();
        render_links_compact(&mut output, response);
        return output;
    }
    if let Some((operation, response)) = computed_compact_result(result) {
        writeln!(output, "lexical_operation: {}", json(&operation)).unwrap();
        writeln!(output, "lexical_policy_ids: {}", json(&response.policy_ids)).unwrap();
        writeln!(output, "lexical_totals: {}", json(&response.totals)).unwrap();
        writeln!(
            output,
            "lexical_source_totals: {}",
            json(&response.source_totals)
        )
        .unwrap();
        writeln!(output, "lexical_exclusions: {}", json(&response.exclusions)).unwrap();
        writeln!(
            output,
            "lexical_next_cursor: {}",
            json(&response.next_cursor)
        )
        .unwrap();
        for hit in &response.hits {
            writeln!(output, "computed_hit: {}", json(hit)).unwrap();
        }
        return output;
    }
    if let WorkbenchLexicalCompactResult::ProposalExact(response) = result {
        writeln!(output, "lexical_operation: {}", json(&"proposal_exact")).unwrap();
        writeln!(output, "lexical_policy_ids: {}", json(&response.policy_ids)).unwrap();
        writeln!(output, "lexical_totals: {}", json(&response.totals)).unwrap();
        writeln!(
            output,
            "lexical_next_cursor: {}",
            json(&response.next_cursor)
        )
        .unwrap();
        for proposal in &response.proposals {
            writeln!(output, "proposal: {}", json(proposal)).unwrap();
        }
        return output;
    }
    if let WorkbenchLexicalCompactResult::Generate(response) = result {
        writeln!(output, "lexical_operation: {}", json(&"generate")).unwrap();
        writeln!(output, "lexical_policy_ids: {}", json(&response.policy_ids)).unwrap();
        writeln!(output, "computed_value: {}", json(&response.value)).unwrap();
        return output;
    }
    let (operation, policy, totals, source_totals, next_cursor) = match result {
        WorkbenchLexicalCompactResult::ReverseDefinition(response) => (
            "reverse_definition",
            &response.policy,
            &response.totals,
            &response.source_totals,
            &response.next_cursor,
        ),
        WorkbenchLexicalCompactResult::Form(response) => (
            "form",
            &response.policy,
            &response.totals,
            &response.source_totals,
            &response.next_cursor,
        ),
        WorkbenchLexicalCompactResult::Links(_) => unreachable!("Links handled above"),
        _ => unreachable!("computed and pure results handled above"),
    };
    writeln!(output, "lexical_operation: {}", json(&operation)).unwrap();
    writeln!(output, "lexical_policy: {}", json(policy)).unwrap();
    writeln!(output, "lexical_totals: {}", json(totals)).unwrap();
    writeln!(output, "lexical_source_totals: {}", json(source_totals)).unwrap();
    writeln!(output, "lexical_next_cursor: {}", json(next_cursor)).unwrap();

    match result {
        WorkbenchLexicalCompactResult::ReverseDefinition(response) => {
            render_reverse_definition_groups(&mut output, response);
        }
        WorkbenchLexicalCompactResult::Form(response) => {
            for group in &response.groups {
                writeln!(output, "source: {}", json(&group.source)).unwrap();
                writeln!(output, "full_count: {}", json(&group.full_count)).unwrap();
                writeln!(output, "returned_count: {}", json(&group.returned_count)).unwrap();
                writeln!(output, "partial: {}", json(&group.partial)).unwrap();
                for row in &group.hits {
                    render_compact_form(&mut output, row);
                }
            }
        }
        WorkbenchLexicalCompactResult::Links(_) => unreachable!("Links handled above"),
        _ => unreachable!("computed and pure results handled above"),
    }
    output
}

fn computed_compact_result(
    result: &WorkbenchLexicalCompactResult,
) -> Option<(
    &'static str,
    &crate::workbench::computed::WorkbenchComputedCompact,
)> {
    use WorkbenchLexicalCompactResult as Result;
    match result {
        Result::SpellingDistance(response) => Some(("spelling_distance", response)),
        Result::Prosody(response) => Some(("prosody", response)),
        Result::Nuclei(response) => Some(("nuclei", response)),
        Result::Contrast(response) => Some(("contrast", response)),
        Result::Components(response) => Some(("components", response)),
        Result::Compose(response) => Some(("compose", response)),
        _ => None,
    }
}

fn render_links_compact(output: &mut String, response: &WorkbenchLinksCompact) {
    for group in &response.groups {
        writeln!(output, "source: {}", json(&group.source)).unwrap();
        writeln!(output, "full_count: {}", json(&group.full_count)).unwrap();
        writeln!(output, "returned_count: {}", json(&group.returned_count)).unwrap();
        writeln!(output, "partial: {}", json(&group.partial)).unwrap();
        for step in &group.hits {
            writeln!(output, "  step: {}", json(&step.identity)).unwrap();
            writeln!(output, "    depth: {}", json(&step.depth)).unwrap();
            writeln!(output, "    from: {}", json(&step.from)).unwrap();
            writeln!(output, "    evidence: {}", json(&step.record)).unwrap();
            writeln!(
                output,
                "    origin_resolution: {}",
                json(&step.origin_resolution)
            )
            .unwrap();
            writeln!(
                output,
                "    target_observations: {}",
                json(&step.target_observations)
            )
            .unwrap();
            writeln!(
                output,
                "    target_assertion_ids: {}",
                json(&step.target_assertion_ids)
            )
            .unwrap();
            writeln!(
                output,
                "    traversal_status: {}",
                json(&step.traversal_status)
            )
            .unwrap();
            writeln!(output, "    link_detail: {}", json(&step.detail_ref)).unwrap();
        }
    }
}

/// Render a direct typed Links detail without opening a provider.
#[must_use]
pub fn render_workbench_link_detail(response: &crate::WorkbenchLinkDetailResponse) -> String {
    render_link_detail_fields(
        &response.binding_sha256,
        None,
        &response.selection,
        &response.reference,
        &response.record,
        &response.origin_resolution,
        &response.target_observations,
    )
}

/// Render a configured typed Links detail without opening a provider.
#[must_use]
pub fn render_configured_workbench_link_detail(
    response: &crate::ConfiguredWorkbenchLinkDetailResponse,
) -> String {
    render_link_detail_fields(
        &response.binding_sha256,
        Some(&response.configuration),
        &response.selection,
        &response.reference,
        &response.record,
        &response.origin_resolution,
        &response.target_observations,
    )
}

fn render_link_detail_fields(
    binding: &str,
    configuration: Option<&vlajmaji_store::WorkbenchProfileIdentity>,
    selection: &crate::WorkbenchQuerySelection,
    reference: &crate::WorkbenchLinkDetailRef,
    record: &crate::WorkbenchLinkRecord,
    origin_resolution: &crate::WorkbenchReferenceCompactResult,
    target_observations: &[crate::WorkbenchLinkTargetCompact],
) -> String {
    let mut output = String::new();
    writeln!(output, "workbench_binding: {}", json(&binding)).unwrap();
    if let Some(configuration) = configuration {
        writeln!(output, "workbench_configuration: {}", json(configuration)).unwrap();
    }
    writeln!(output, "workbench_selection: {}", json(selection)).unwrap();
    writeln!(output, "link_reference: {}", json(reference)).unwrap();
    writeln!(output, "record: {}", json(record)).unwrap();
    writeln!(output, "origin_resolution: {}", json(origin_resolution)).unwrap();
    writeln!(output, "target_observations: {}", json(target_observations)).unwrap();
    write!(output, "link_policy: {}", json(&"bounded-links-v1")).unwrap();
    output
}

/// Render a complete Form detail that has already passed the provider's
/// freshness and typed-row checks.
#[must_use]
pub fn render_workbench_form_detail(row: &WorkbenchFormRecord) -> String {
    let mut output = String::new();
    writeln!(output, "form: {}", json(&row.form)).unwrap();
    writeln!(output, "entry: {}", json(&row.entry)).unwrap();
    writeln!(output, "source: {}", json(&row.source)).unwrap();
    writeln!(output, "mapped_role: {}", json(&row.mapped_role)).unwrap();
    writeln!(output, "computation: {}", json(&row.computation)).unwrap();
    writeln!(output, "evidence: {}", json(&row.evidence)).unwrap();
    writeln!(
        output,
        "association_status: {}",
        json(&row.association_status)
    )
    .unwrap();
    writeln!(
        output,
        "associated_assertions: {}",
        json(&row.associated_assertions)
    )
    .unwrap();
    output
}

fn render_reverse_definition_groups(
    output: &mut String,
    response: &WorkbenchReverseDefinitionCompact,
) {
    for group in &response.groups {
        writeln!(output, "source: {}", json(&group.source)).unwrap();
        writeln!(output, "full_count: {}", json(&group.full_count)).unwrap();
        writeln!(output, "returned_count: {}", json(&group.returned_count)).unwrap();
        writeln!(output, "partial: {}", json(&group.partial)).unwrap();
        for hit in &group.hits {
            writeln!(output, "  assertion: {}", json(&hit.assertion.id)).unwrap();
            writeln!(output, "    spelling: {}", json(&hit.entry.spelling)).unwrap();
            writeln!(
                output,
                "    definition: {}",
                json(&hit.assertion.definition)
            )
            .unwrap();
            writeln!(output, "    detail: {}", json(&hit.detail_ref)).unwrap();
        }
    }
}

fn render_compact_form(output: &mut String, row: &WorkbenchCompactFormRecord) {
    writeln!(output, "  form: {}", json(&row.form_id)).unwrap();
    writeln!(output, "    kind: {}", json(&row.kind)).unwrap();
    writeln!(output, "    mapped_role: {}", json(&row.mapped_role)).unwrap();
    writeln!(
        output,
        "    entry: {}",
        json(&serde_json::json!({"id": row.entry_id, "spelling": row.entry_spelling}))
    )
    .unwrap();
    writeln!(output, "    source_id: {}", json(&row.source_id)).unwrap();
    writeln!(output, "    evidence: {}", json(&row.evidence)).unwrap();
    writeln!(output, "    computation_id: {}", json(&row.computation_id)).unwrap();
    writeln!(
        output,
        "    association_status: {}",
        json(&row.association_status)
    )
    .unwrap();
    writeln!(
        output,
        "    associated_assertions: {}",
        json(&row.associated_assertions)
    )
    .unwrap();
    writeln!(output, "    form_detail: {}", json(&row.detail_ref)).unwrap();
}

fn render_exact(
    response: &DictionaryCompactExactResponse,
    managed: Option<(
        &crate::CollectionSelectionSnapshot,
        &[crate::CollectionPackAvailability],
    )>,
) -> String {
    let mut output = String::new();
    writeln!(
        output,
        "Exact {}: {} occurrences, {} spellings, {} sources",
        json(&response.spelling),
        response.totals.occurrences,
        response.totals.distinct_spellings,
        response.totals.source_groups,
    )
    .expect("writing to String cannot fail");

    for group in &response.groups {
        write!(
            output,
            "{} [{}] {}/{}",
            atom(&group.source.abbreviation),
            atom(group.source.id.as_str()),
            group.returned_count,
            group.full_count,
        )
        .expect("writing to String cannot fail");
        if group.partial {
            output.push_str(" (partial)");
        }
        output.push('\n');
        for hit in &group.hits {
            writeln!(
                output,
                "  {} @ {}",
                atom(hit.assertion.id.as_str()),
                atom(&hit.occurrence_id),
            )
            .expect("writing to String cannot fail");
            writeln!(output, "    spelling: {}", json(&hit.entry.spelling))
                .expect("writing to String cannot fail");
            writeln!(
                output,
                "    definition: {}",
                json(&hit.assertion.definition)
            )
            .expect("writing to String cannot fail");
            writeln!(output, "    detail: {}", json(&hit.detail_ref))
                .expect("writing to String cannot fail");
        }
    }

    if let Some((snapshot, collections)) = managed {
        writeln!(output, "selection: {}", json(snapshot)).expect("writing to String cannot fail");
        writeln!(output, "collections: {}", json(collections))
            .expect("writing to String cannot fail");
    }
    writeln!(output, "next_cursor: {}", json(&response.next_cursor))
        .expect("writing to String cannot fail");
    output
}

fn render_dictionary_resolve(response: &DictionaryCompactReferenceResolution) -> String {
    let mut output = String::new();
    write_reference_head(&mut output, &response.target, response.status);
    write_entity_and_views(
        &mut output,
        response.entity.as_ref(),
        &response.confirmed,
        &response.unverified,
        &response.availability,
    );
    writeln!(
        output,
        "requested_release: {}",
        json(&response.requested_release)
    )
    .expect("writing to String cannot fail");
    output
}

fn render_collection_resolve(
    response: &CollectionCompactReferenceResponse,
    snapshot: &crate::CollectionSelectionSnapshot,
    collections: &[crate::CollectionPackAvailability],
) -> String {
    let mut output = String::new();
    write_reference_head(&mut output, &response.target, response.status);
    writeln!(output, "reason: {}", json(&response.operational_reason))
        .expect("writing to String cannot fail");
    writeln!(output, "payload: {}", response.payload.is_some())
        .expect("writing to String cannot fail");

    if let Some(payload) = &response.payload {
        write_entity_and_views(
            &mut output,
            payload.entity.as_ref(),
            &payload.confirmed,
            &payload.unverified,
            &payload.availability,
        );
    } else {
        writeln!(output, "entity: null\nconfirmed: 0\nunverified: 0")
            .expect("writing to String cannot fail");
    }

    writeln!(
        output,
        "requested_release: {}",
        json(&response.requested_release)
    )
    .expect("writing to String cannot fail");
    writeln!(
        output,
        "catalogue_matches: {}",
        json(&response.catalogue_matches)
    )
    .expect("writing to String cannot fail");
    writeln!(output, "selection: {}", json(snapshot)).expect("writing to String cannot fail");
    writeln!(output, "collections: {}", json(collections)).expect("writing to String cannot fail");
    output
}

fn write_reference_head(
    output: &mut String,
    target: &vlajmaji_lexicon::RelationTarget,
    status: ReferenceStatus,
) {
    writeln!(output, "Reference {}: {status:?}", json(target))
        .expect("writing to String cannot fail");
}

fn write_entity_and_views(
    output: &mut String,
    entity: Option<&vlajmaji_store::CollectionEntityReference>,
    confirmed: &[DictionaryCompactReferenceAssertion],
    unverified: &[DictionaryCompactReferenceAssertion],
    availability: &[crate::DictionaryCompactSourceAvailability],
) {
    writeln!(output, "entity: {}", json(&entity)).expect("writing to String cannot fail");
    writeln!(output, "confirmed: {}", confirmed.len()).expect("writing to String cannot fail");
    write_classification(output, confirmed, availability, "confirmed");
    writeln!(output, "unverified: {}", unverified.len()).expect("writing to String cannot fail");
    write_classification(output, unverified, availability, "unverified");
}

fn write_classification(
    output: &mut String,
    rows: &[DictionaryCompactReferenceAssertion],
    availability: &[crate::DictionaryCompactSourceAvailability],
    label: &str,
) {
    let mut groups =
        BTreeMap::<&vlajmaji_lexicon::SourceId, Vec<&DictionaryCompactReferenceAssertion>>::new();
    for row in rows {
        groups.entry(&row.entry.source).or_default().push(row);
    }

    for (source_id, group) in groups {
        let abbreviation = availability
            .iter()
            .find(|source| &source.source.id == source_id)
            .map(|source| source.source.abbreviation.as_str())
            .unwrap_or("?");
        writeln!(
            output,
            "{} [{}] {} {label}",
            atom(abbreviation),
            atom(source_id.as_str()),
            group.len(),
        )
        .expect("writing to String cannot fail");
        for row in group {
            write_reference_row(output, row);
        }
    }
}

fn write_reference_row(output: &mut String, row: &DictionaryCompactReferenceAssertion) {
    writeln!(
        output,
        "  {} @ {}",
        atom(row.assertion.id.as_str()),
        atom(&row.occurrence_id),
    )
    .expect("writing to String cannot fail");
    writeln!(output, "    spelling: {}", json(&row.entry.spelling))
        .expect("writing to String cannot fail");
    writeln!(
        output,
        "    definition: {}",
        json(&row.assertion.definition)
    )
    .expect("writing to String cannot fail");
    writeln!(
        output,
        "    eligibility: {}",
        json(&row.assertion.eligibility)
    )
    .expect("writing to String cannot fail");
    writeln!(
        output,
        "    payload_present: {}",
        json(&row.payload_present)
    )
    .expect("writing to String cannot fail");
    writeln!(
        output,
        "    current_eligible: {}",
        json(&row.current_eligible)
    )
    .expect("writing to String cannot fail");
    writeln!(output, "    detail: {}", json(&row.detail_ref))
        .expect("writing to String cannot fail");
}

fn render_detail(
    response: &DictionaryHitDetailResponse,
    managed: Option<(
        &crate::CollectionSelectionSnapshot,
        &[crate::CollectionPackAvailability],
        &[crate::CollectionReferenceMatch],
    )>,
) -> String {
    let mut output = String::new();
    writeln!(
        output,
        "Detail {}: {:?}",
        atom(response.detail_ref.assertion_id.as_str()),
        response.status,
    )
    .expect("writing to String cannot fail");
    writeln!(output, "snapshot: {}", json(&response.snapshot_sha256))
        .expect("writing to String cannot fail");
    writeln!(output, "reason: {}", json(&response.operational_reason))
        .expect("writing to String cannot fail");
    writeln!(
        output,
        "current_eligible: {}",
        json(&response.current_eligible)
    )
    .expect("writing to String cannot fail");
    writeln!(output, "reference: {}", json(&response.detail_ref))
        .expect("writing to String cannot fail");
    if let Some((snapshot, collections, catalogue_matches)) = managed {
        writeln!(output, "selection: {}", json(snapshot)).expect("writing to String cannot fail");
        writeln!(output, "collections: {}", json(collections))
            .expect("writing to String cannot fail");
        writeln!(output, "catalogue_matches: {}", json(catalogue_matches))
            .expect("writing to String cannot fail");
    }
    if let Some(hit) = &response.hit {
        writeln!(output, "hit: {}", pretty_json(hit)).expect("writing to String cannot fail");
    } else {
        output.push_str("hit: null\n");
    }
    output
}

fn json<T: Serialize + ?Sized>(value: &T) -> String {
    serde_json::to_string(value).expect("public projection types are JSON serializable")
}

fn pretty_json<T: Serialize + ?Sized>(value: &T) -> String {
    serde_json::to_string_pretty(value).expect("public projection types are JSON serializable")
}

fn atom(value: &str) -> String {
    let encoded = json(&value);
    encoded
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .expect("JSON strings have enclosing quotes")
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::atom;

    #[test]
    fn header_atoms_use_json_content_escaping_without_normalization() {
        assert_eq!(atom("A\u{1b}\r\t\n\\\"é"), r#"A\u001b\r\t\n\\\"é"#);
    }
}
