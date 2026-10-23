//! One-open full-population query and source-scan comparison probe.

use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    str::FromStr,
};

use relojban_phonology::{
    DistanceModel, NucleusKind, Segment, SyllabificationMode, WordKind, analyze_word, syllabify,
};
use serde_json::json;
use vlajmaji_lexicon::{CapturedRecord, FieldPresence, Lexicon, SourceFormat};
use vlajmaji_search::{MorphologySetting, Query, SearchEngine, SearchOptions};
use vlajmaji_store::{
    LexicalStore, SearchQuery as NativeQuery, SqliteStore, record_id, scan_reference,
    validate_archive,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args_os().skip(1);
    let archive_path = args
        .next()
        .ok_or("usage: full_population_probe ARCHIVE STORE")?;
    let store_path = std::path::PathBuf::from(
        args.next()
            .ok_or("usage: full_population_probe ARCHIVE STORE")?,
    );
    let archive = fs::read(archive_path)?;
    let lexicon = validate_archive(&archive)?;
    let native = SqliteStore::open(&store_path)?;
    let engine = SearchEngine::open(&store_path)?;
    let (corpus_records, snapshots) = engine.corpus_counts();
    let distinct_spellings = lexicon
        .records()
        .iter()
        .filter_map(|record| record.word())
        .collect::<BTreeSet<_>>()
        .len();
    println!(
        "{}",
        json!({
            "event":"corpus",
            "records":corpus_records,
            "snapshots":snapshots,
            "distinct_spellings":distinct_spellings,
            "archive_bytes":archive.len()
        })
    );
    let mut source_class_counts = BTreeMap::<String, usize>::new();
    for record in lexicon.records() {
        if let Some(class) = independent_native_class(record) {
            let format = format!("{:?}", record.format());
            *source_class_counts
                .entry(format!("{format}:{class}"))
                .or_default() += 1;
        }
    }
    println!(
        "{}",
        json!({"event":"native_class_census","counts":source_class_counts})
    );

    let native_cases = [
        (
            "exact-farvi",
            NativeQuery::ExactSpelling("farvi".into()),
            Query::Exact {
                spelling: "farvi".into(),
            },
        ),
        (
            "reverse-snow",
            NativeQuery::DefinitionContains("snow".into()),
            Query::ReverseDefinition {
                contains: "snow".into(),
            },
        ),
        (
            "links-farvi-depth2",
            NativeQuery::CrossReferences {
                spelling: "farvi".into(),
                depth: 2,
            },
            Query::Links {
                spelling: "farvi".into(),
                depth: 2,
            },
        ),
    ];
    for (name, query, api_query) in native_cases {
        let expected = scan_reference(&lexicon, &query);
        let actual = native.search(&query)?;
        assert_eq!(actual.record_ids, expected.record_ids, "{name} native IDs");
        assert_eq!(actual.edges, expected.edges, "{name} native edges");
        let complete = engine.search(
            api_query,
            SearchOptions {
                limit: 1000,
                include_proposals: false,
            },
        )?;
        assert_eq!(
            complete
                .hits
                .iter()
                .map(|hit| hit.record_id.clone())
                .collect::<BTreeSet<_>>(),
            expected.record_ids,
            "{name} search API IDs"
        );
        if name == "links-farvi-depth2" {
            assert_eq!(
                complete.links.len(),
                expected.edges.len(),
                "{name} edge explanations"
            );
        }
        println!(
            "{}",
            json!({
                "event":"native_oracle",
                "name":name,
                "native_path":actual.path,
                "fallback":actual.fallback_reason,
                "record_ids":actual.record_ids.len(),
                "edges":actual.edges.len(),
                "api_returned":complete.hits.len()
            })
        );
    }
    if let Some(form) = lexicon.forms().first() {
        let value = form.form().to_owned();
        let role = form.role();
        let query = NativeQuery::Form {
            value: value.clone(),
            role: Some(role),
        };
        let expected = scan_reference(&lexicon, &query);
        let actual = native.search(&query)?;
        assert_eq!(actual.record_ids, expected.record_ids, "form native IDs");
        let response = engine.search(
            Query::Form {
                value,
                role: Some(role),
            },
            SearchOptions {
                limit: 1000,
                include_proposals: false,
            },
        )?;
        assert_eq!(
            response
                .hits
                .iter()
                .map(|hit| hit.record_id.clone())
                .collect::<BTreeSet<_>>(),
            expected.record_ids,
            "form search API IDs"
        );
        println!(
            "{}",
            json!({
                "event":"native_oracle",
                "name":"first-captured-form",
                "role":format!("{role:?}"),
                "record_ids":actual.record_ids.len()
            })
        );
    }

    let anchor = "farvi";
    let expected_distance = lexicon
        .records()
        .iter()
        .filter_map(|record| record.word().map(|word| (record, word)))
        .filter(|(_, word)| strsim::levenshtein(anchor, word) <= 1)
        .map(|(record, _)| record_id(record.identity()))
        .collect::<BTreeSet<_>>();
    let response = engine.search(
        Query::SpellingDistance {
            anchor: anchor.into(),
            max_distance: Some(1),
        },
        SearchOptions {
            limit: 1000,
            include_proposals: false,
        },
    )?;
    let actual_distance = response
        .hits
        .iter()
        .map(|hit| hit.record_id.clone())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        actual_distance, expected_distance,
        "full-scan spelling distance IDs"
    );
    println!(
        "{}",
        json!({
            "event":"scan_oracle",
            "name":"spelling_distance_farvi_le_1",
            "record_ids":actual_distance.len(),
            "distinct_spellings":response.distinct_spellings,
            "exclusions":response.exclusions
        })
    );

    let scan_cases = [
        (
            "prosody-two-syllable-brivla",
            Query::Prosody {
                kind: Some("brivla".into()),
                syllables: Some(2),
                nuclei: None,
                primary_nucleus: None,
            },
        ),
        (
            "nuclei-ai-brivla",
            Query::Nuclei {
                kind: "brivla".into(),
                pattern: vec!["a".into(), "i".into()],
            },
        ),
        (
            "contrast-farvi-brivla",
            Query::Contrast {
                anchor: "farvi".into(),
                kind: "brivla".into(),
                require_mismatch: true,
            },
        ),
        (
            "ordered-component-farvi",
            Query::Components {
                components: vec!["farvi".into()],
            },
        ),
    ];
    for (name, query) in scan_cases {
        let expected = independent_scan(name, &lexicon)?;
        let response = assert_complete_and_limits(&engine, &lexicon, query, &expected, name)?;
        println!(
            "{}",
            json!({
                "event":"scan_family",
                "name":name,
                "total_records":response.total_records,
                "distinct_spellings":response.distinct_spellings,
                "returned":response.hits.len(),
                "exclusions":response.exclusions,
                "policy_ids":response.policy_ids
            })
        );
    }

    let definition_candidates =
        scan_reference(&lexicon, &NativeQuery::DefinitionContains("grow".into()));
    let mut expected_compose = BTreeSet::new();
    for record in lexicon.records() {
        let id = record_id(record.identity());
        if !definition_candidates.record_ids.contains(&id) {
            continue;
        }
        let Some(spelling) = record.word() else {
            continue;
        };
        if independent_kind(record) != Some(WordKind::Brivla) {
            continue;
        }
        if let Ok(analysis) = analyze_word(spelling, WordKind::Brivla, &[])
            && analysis.nuclei().len() >= 2
        {
            expected_compose.insert(id);
        }
    }
    let compose_query = Query::Compose {
        definition_contains: Some("grow".into()),
        links_from: None,
        min_nuclei: Some(2),
        kind: Some("brivla".into()),
    };
    let expected_compose_scan = ExpectedScan {
        ids: expected_compose,
        exclusions: independent_compose_exclusions(&lexicon, &definition_candidates.record_ids),
    };
    let composed = assert_complete_and_limits(
        &engine,
        &lexicon,
        compose_query,
        &expected_compose_scan,
        "definition-grow-brivla-two-nuclei",
    )?;
    println!(
        "{}",
        json!({
            "event":"composed_query",
            "name":"definition-grow-brivla-two-nuclei",
            "definition_candidates":definition_candidates.record_ids.len(),
            "total_records":composed.total_records,
            "returned":composed.hits.len(),
            "exclusions":composed.exclusions
        })
    );

    #[cfg(feature = "morphology")]
    {
        let inputs = vec!["farvi".to_owned(), "snime".to_owned()];
        for setting in [
            MorphologySetting::Standard,
            MorphologySetting::AllowY,
            MorphologySetting::ForceY,
        ] {
            let generated = engine.search(
                Query::Generate {
                    components: inputs.clone(),
                    setting,
                },
                SearchOptions::default(),
            )?;
            let reverse = engine.search(
                Query::Generate {
                    components: inputs.iter().rev().cloned().collect(),
                    setting,
                },
                SearchOptions::default(),
            )?;
            let direct_settings = latkerlo_jvotci::Settings {
                y_hyphens: match setting {
                    MorphologySetting::Standard => latkerlo_jvotci::YHyphenSetting::Standard,
                    MorphologySetting::AllowY => latkerlo_jvotci::YHyphenSetting::AllowY,
                    MorphologySetting::ForceY => latkerlo_jvotci::YHyphenSetting::ForceY,
                },
                ..latkerlo_jvotci::Settings::default()
            };
            let direct = latkerlo_jvotci::get_lujvo_with_analytics("farvi snime", &direct_settings)
                .map_err(|error| error.to_string())?;
            assert_eq!(
                generated.computed[0].spelling, direct.0,
                "setting {setting:?}"
            );
            assert_eq!(
                generated.computed[0].engine_score, direct.1,
                "setting {setting:?}"
            );
            assert_eq!(
                generated.computed[0].score_function,
                "latkerlo_jvotci::get_lujvo_with_analytics"
            );
            let reverse_direct =
                latkerlo_jvotci::get_lujvo_with_analytics("snime farvi", &direct_settings)
                    .map_err(|error| error.to_string())?;
            assert_eq!(reverse.computed[0].spelling, reverse_direct.0);
            assert_eq!(reverse.computed[0].engine_score, reverse_direct.1);
            let mut joins = Vec::new();
            for spelling in ["farvi", "snime", generated.computed[0].spelling.as_str()] {
                joins.push(source_membership_join(&engine, &lexicon, spelling)?);
            }
            joins.push(source_membership_join(
                &engine,
                &lexicon,
                reverse.computed[0].spelling.as_str(),
            )?);
            println!(
                "{}",
                json!({"event":"generation_source_joins","setting":setting,"forward":generated.computed,"reverse":reverse.computed,"direct_score_function":"latkerlo_jvotci::get_lujvo_with_analytics","joins":joins})
            );
        }
    }

    let expected_proposals = lexicon
        .lookup_proposals("farvi")
        .into_iter()
        .map(|proposal| proposal.id().to_owned())
        .collect::<BTreeSet<_>>();
    let proposals = engine.search(
        Query::ProposalExact {
            spelling: "farvi".into(),
        },
        SearchOptions {
            limit: 1000,
            include_proposals: false,
        },
    )?;
    assert_eq!(
        proposals
            .hits
            .iter()
            .map(|hit| hit.record_id.clone())
            .collect::<BTreeSet<_>>(),
        expected_proposals,
        "proposal namespace exact IDs"
    );
    println!(
        "{}",
        json!({
            "event":"proposal_query",
            "name":"proposal-exact-farvi",
            "total_records":proposals.total_records,
            "returned":proposals.hits.len()
        })
    );

    let generated = engine.search(
        Query::Generate {
            components: vec!["farvi".into(), "snime".into()],
            setting: MorphologySetting::Standard,
        },
        SearchOptions::default(),
    )?;
    println!(
        "{}",
        json!({"event":"generation","computed":generated.computed})
    );
    Ok(())
}

fn independent_native_class(record: &CapturedRecord) -> Option<String> {
    match record.format() {
        SourceFormat::LaLojbanJson => json_string(record, "word_type"),
        SourceFormat::LensiskuJson => json_string(record, "type_name"),
        SourceFormat::LlgGismuFixed => Some("gismu".into()),
        SourceFormat::LlgCmavoFixed => Some("cmavo".into()),
        SourceFormat::LlgRafsiRows => None,
        _ => None,
    }
}

fn json_string(record: &CapturedRecord, field: &str) -> Option<String> {
    let FieldPresence::Value(raw) = record.field(field) else {
        return None;
    };
    serde_json::from_str(raw).ok()
}

fn independent_kind(record: &CapturedRecord) -> Option<WordKind> {
    match independent_native_class(record)?.as_str() {
        "gismu" | "experimental gismu" => Some(WordKind::Brivla),
        "lujvo" => Some(WordKind::GismuBasedLujvo),
        "fu'ivla" | "obsolete fu'ivla" => Some(WordKind::Zihevla),
        "cmavo" | "experimental cmavo" | "obsolete cmavo" => Some(WordKind::Cmavo),
        "cmevla" | "obsolete cmevla" => Some(WordKind::Cmevla),
        _ => None,
    }
}

struct ExpectedScan {
    ids: BTreeSet<String>,
    exclusions: BTreeMap<String, usize>,
}

fn assert_complete_and_limits(
    engine: &SearchEngine,
    lexicon: &Lexicon,
    query: Query,
    expected: &ExpectedScan,
    name: &str,
) -> Result<vlajmaji_search::SearchResponse, Box<dyn std::error::Error>> {
    let mut expected_rows = lexicon
        .records()
        .iter()
        .filter_map(|record| {
            let id = record_id(record.identity());
            expected
                .ids
                .contains(&id)
                .then(|| (record.word().unwrap_or("").as_bytes().to_vec(), id))
        })
        .collect::<Vec<_>>();
    expected_rows.sort();
    let ordered_ids = expected_rows
        .iter()
        .map(|(_, id)| id.clone())
        .collect::<Vec<_>>();
    let distinct = expected_rows
        .iter()
        .map(|(spelling, _)| spelling.clone())
        .collect::<BTreeSet<_>>()
        .len();
    let full = engine.search(
        query.clone(),
        SearchOptions {
            limit: usize::MAX,
            include_proposals: false,
        },
    )?;
    let actual_ids = full
        .hits
        .iter()
        .map(|hit| hit.record_id.clone())
        .collect::<Vec<_>>();
    assert_eq!(full.total_records, expected.ids.len(), "{name} total");
    assert_eq!(
        actual_ids.len(),
        expected.ids.len(),
        "{name} full cardinality"
    );
    assert_eq!(
        actual_ids.iter().collect::<BTreeSet<_>>().len(),
        actual_ids.len(),
        "{name} no duplicate ids"
    );
    assert_eq!(actual_ids, ordered_ids, "{name} complete sorted ids");
    assert_eq!(
        full.distinct_spellings, distinct,
        "{name} distinct spelling count"
    );
    assert_eq!(full.exclusions, expected.exclusions, "{name} exclusions");
    for limit in [0, 7] {
        let limited = engine.search(
            query.clone(),
            SearchOptions {
                limit,
                include_proposals: false,
            },
        )?;
        assert_eq!(
            limited.total_records,
            expected.ids.len(),
            "{name} limit {limit} total"
        );
        assert_eq!(
            limited
                .hits
                .iter()
                .map(|hit| hit.record_id.clone())
                .collect::<Vec<_>>(),
            ordered_ids.iter().take(limit).cloned().collect::<Vec<_>>(),
            "{name} limit {limit} prefix"
        );
        assert_eq!(
            limited.distinct_spellings, distinct,
            "{name} limit {limit} distinct"
        );
        assert_eq!(
            limited.exclusions, expected.exclusions,
            "{name} limit {limit} exclusions"
        );
    }
    Ok(full)
}

fn source_membership_join(
    engine: &SearchEngine,
    lexicon: &Lexicon,
    spelling: &str,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let response = engine.search(
        Query::Exact {
            spelling: spelling.into(),
        },
        SearchOptions {
            limit: usize::MAX,
            include_proposals: false,
        },
    )?;
    let expected = lexicon
        .records()
        .iter()
        .filter(|record| record.word() == Some(spelling))
        .map(|record| record_id(record.identity()))
        .collect::<BTreeSet<_>>();
    let actual = response
        .hits
        .iter()
        .map(|hit| hit.record_id.clone())
        .collect::<BTreeSet<_>>();
    assert_eq!(actual, expected, "generation Exact join for {spelling}");
    let forms = engine.search(
        Query::Form {
            value: spelling.into(),
            role: None,
        },
        SearchOptions {
            limit: usize::MAX,
            include_proposals: false,
        },
    )?;
    let expected_forms = lexicon
        .forms()
        .iter()
        .filter(|form| form.form() == spelling)
        .map(|form| record_id(form.source_record()))
        .collect::<BTreeSet<_>>();
    let actual_forms = forms
        .hits
        .iter()
        .map(|hit| hit.record_id.clone())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        actual_forms, expected_forms,
        "generation form join for {spelling}"
    );
    let form_roles = lexicon
        .forms()
        .iter()
        .filter(|form| form.form() == spelling)
        .map(|form| json!({"record_id":record_id(form.source_record()),"role":format!("{:?}", form.role())}))
        .collect::<Vec<_>>();
    Ok(
        json!({"spelling":spelling,"exact_record_ids":actual,"records":response.hits.iter().map(|hit| json!({"record_id":hit.record_id,"definition_present":hit.definition.is_some(),"gloss_present":hit.gloss.is_some()})).collect::<Vec<_>>(),"source_form_record_ids":actual_forms,"source_form_roles":form_roles,"zero_source_matches":expected.is_empty() && expected_forms.is_empty()}),
    )
}

fn independent_compose_exclusions(
    lexicon: &Lexicon,
    candidates: &BTreeSet<String>,
) -> BTreeMap<String, usize> {
    let mut exclusions = BTreeMap::new();
    for record in lexicon.records() {
        if !candidates.contains(&record_id(record.identity())) {
            continue;
        }
        let Some(spelling) = record.word() else {
            bump(&mut exclusions, "no-headword");
            continue;
        };
        let Some(kind) = independent_kind(record) else {
            bump(&mut exclusions, "unmapped-source-class");
            continue;
        };
        if kind != WordKind::Brivla {
            bump(&mut exclusions, "requested-kind-filter");
            continue;
        }
        match analyze_word(spelling, kind, &[]) {
            Ok(analysis) if analysis.nuclei().len() >= 2 => {}
            Ok(_) => bump(&mut exclusions, "nuclei-filter"),
            Err(_) => bump(&mut exclusions, "analysis-error"),
        }
    }
    exclusions
}

fn independent_scan(
    name: &str,
    lexicon: &Lexicon,
) -> Result<ExpectedScan, Box<dyn std::error::Error>> {
    let mut ids = BTreeSet::new();
    let mut exclusions = BTreeMap::new();
    match name {
        "prosody-two-syllable-brivla" => {
            for record in lexicon.records() {
                let Some(spelling) = record.word() else {
                    bump(&mut exclusions, "no-headword");
                    continue;
                };
                let Some(kind) = independent_kind(record) else {
                    bump(&mut exclusions, "unmapped-source-class");
                    continue;
                };
                if kind != WordKind::Brivla {
                    bump(&mut exclusions, "requested-kind-filter");
                    continue;
                }
                match analyze_word(spelling, kind, &[]) {
                    Ok(analysis) => {
                        let grouped = syllabify(&analysis, SyllabificationMode::Canonical);
                        if !grouped.is_resolved() {
                            bump(&mut exclusions, "unresolved-syllable-grouping");
                        } else if grouped.syllables().len() == 2 {
                            ids.insert(record_id(record.identity()));
                        } else {
                            bump(&mut exclusions, "filter-mismatch");
                        }
                    }
                    Err(_) => bump(&mut exclusions, "analysis-error"),
                }
            }
        }
        "nuclei-ai-brivla" => {
            for record in lexicon.records() {
                let Some(spelling) = record.word() else {
                    bump(&mut exclusions, "no-headword");
                    continue;
                };
                let Some(kind) = independent_kind(record) else {
                    bump(&mut exclusions, "unmapped-source-class");
                    continue;
                };
                if kind != WordKind::Brivla {
                    bump(&mut exclusions, "requested-kind-filter");
                    continue;
                }
                match analyze_word(spelling, kind, &[]) {
                    Ok(analysis)
                        if analysis
                            .nuclei()
                            .iter()
                            .map(|n| n.spelling())
                            .eq(["a", "i"]) =>
                    {
                        ids.insert(record_id(record.identity()));
                    }
                    Ok(_) => bump(&mut exclusions, "nucleus-pattern-mismatch"),
                    Err(_) => bump(&mut exclusions, "analysis-error"),
                }
            }
        }
        "contrast-farvi-brivla" => {
            let anchor = analyze_word("farvi", WordKind::Brivla, &[])?;
            let anchor_nuclei: Vec<_> = anchor.nuclei().iter().collect();
            for record in lexicon.records() {
                let Some(spelling) = record.word() else {
                    bump(&mut exclusions, "no-headword");
                    continue;
                };
                let Some(kind) = independent_kind(record) else {
                    bump(&mut exclusions, "unmapped-source-class");
                    continue;
                };
                if kind != WordKind::Brivla {
                    bump(&mut exclusions, "requested-kind-filter");
                    continue;
                }
                match analyze_word(spelling, kind, &[]) {
                    Ok(analysis)
                        if analysis.nuclei().len() == anchor_nuclei.len()
                            && analysis
                                .nuclei()
                                .iter()
                                .all(|n| matches!(n.kind(), NucleusKind::Vowel(_))) =>
                    {
                        let mut mismatch = false;
                        for (left, right) in anchor_nuclei.iter().zip(analysis.nuclei()) {
                            let left = Segment::from_str(left.spelling())?;
                            let right = Segment::from_str(right.spelling())?;
                            mismatch |= DistanceModel::default().compare(left, right)?.total() > 0;
                        }
                        if mismatch {
                            ids.insert(record_id(record.identity()));
                        } else {
                            bump(&mut exclusions, "no-within-vowel-mismatch");
                        }
                    }
                    Ok(_) => bump(&mut exclusions, "unaligned-or-non-vowel-nucleus"),
                    Err(_) => bump(&mut exclusions, "analysis-error"),
                }
            }
        }
        "ordered-component-farvi" => {
            let settings = latkerlo_jvotci::Settings::default();
            let mut cache = BTreeMap::<String, Result<Vec<String>, String>>::new();
            for record in lexicon.records() {
                let Some(spelling) = record.word() else {
                    bump(&mut exclusions, "no-headword");
                    continue;
                };
                if !spelling.is_ascii() {
                    bump(&mut exclusions, "unsupported-candidate-non-ascii");
                    continue;
                }
                let value = cache.entry(spelling.into()).or_insert_with(|| {
                    std::panic::catch_unwind(|| latkerlo_jvotci::get_veljvo(spelling, &settings))
                        .map_err(|_| "morphology-panic-caught-unwind-only".to_owned())
                        .and_then(|result| {
                            result.map_err(|_| "morphology-parse-rejected".to_owned())
                        })
                });
                match value {
                    Ok(parts) if ordered_subsequence(parts, &["farvi".into()]) => {
                        ids.insert(record_id(record.identity()));
                    }
                    Ok(_) => bump(&mut exclusions, "component-sequence-mismatch"),
                    Err(reason) => bump(&mut exclusions, reason),
                }
            }
        }
        _ => return Err(format!("unknown oracle case {name}").into()),
    }
    Ok(ExpectedScan { ids, exclusions })
}

fn ordered_subsequence(returned: &[String], requested: &[String]) -> bool {
    if requested.is_empty() {
        return false;
    }
    let mut next = 0;
    for component in returned {
        if component == &requested[next] {
            next += 1;
            if next == requested.len() {
                return true;
            }
        }
    }
    false
}

fn bump(counts: &mut BTreeMap<String, usize>, label: &str) {
    *counts.entry(label.to_owned()).or_default() += 1;
}
