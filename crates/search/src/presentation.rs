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
    DictionaryHitDetailResponse, ReferenceStatus,
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
