//! Search API behavior over a bounded fixture corpus.

use std::{
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

use vlajmaji_lexicon::{Lexicon, Proposal, SourceFormat, SourceMetadata, TimePrecision};
use vlajmaji_search::{Query, SearchEngine, SearchOptions};

static NEXT: AtomicUsize = AtomicUsize::new(0);

fn fixture_store() -> (PathBuf, SearchEngine) {
    let mut lexicon = Lexicon::new();
    lexicon
        .import_source(
            SourceMetadata::new(
                "fixture",
                "query fixture",
                SourceFormat::LaLojbanJson,
                "fixture.json",
                "v1",
                None,
                TimePrecision::Unknown,
            ),
            br#"[
            {"word":"farvi","word_type":"gismu","definition":"plum grows"},
            {"word":"farvi","word_type":"gismu","definition":"object evolves"},
            {"word":"snime","word_type":"gismu","definition":"snow and plum"}
        ]"#,
            None,
        )
        .unwrap();
    lexicon.add_proposal(Proposal::new("fixture-notes", "farvi", "proposed alternative").unwrap());
    lexicon.add_proposal(Proposal::new("fixture-notes", "proposalonly", "proposal only").unwrap());
    let archive = lexicon.to_archive_bytes().unwrap();
    let path = std::env::temp_dir().join(format!(
        "relojban-search-test-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let engine = SearchEngine::build(&path, &archive).unwrap();
    (path, engine)
}

#[test]
fn exact_proposal_option_is_separate_complete_and_limited_after_combination() {
    let (path, engine) = fixture_store();
    let source_only = engine
        .search(
            Query::Exact {
                spelling: "farvi".into(),
            },
            SearchOptions::default(),
        )
        .unwrap();
    assert_eq!(source_only.total_records, 2);
    assert!(
        source_only
            .hits
            .iter()
            .all(|hit| hit.source_format != "Proposal")
    );

    let combined = engine
        .search(
            Query::Exact {
                spelling: "farvi".into(),
            },
            SearchOptions {
                limit: 20,
                include_proposals: true,
            },
        )
        .unwrap();
    assert_eq!(combined.total_records, 3);
    assert_eq!(combined.distinct_spellings, 1);
    assert_eq!(
        combined
            .hits
            .iter()
            .filter(|hit| hit.source_format == "Proposal")
            .count(),
        1
    );
    assert!(
        combined
            .hits
            .last()
            .unwrap()
            .reason
            .contains("not captured source")
    );

    let zero = engine
        .search(
            Query::Exact {
                spelling: "farvi".into(),
            },
            SearchOptions {
                limit: 0,
                include_proposals: true,
            },
        )
        .unwrap();
    assert_eq!(zero.total_records, 3);
    assert!(zero.hits.is_empty());
    let prefix = engine
        .search(
            Query::Exact {
                spelling: "farvi".into(),
            },
            SearchOptions {
                limit: 2,
                include_proposals: true,
            },
        )
        .unwrap();
    assert_eq!(prefix.total_records, 3);
    assert_eq!(prefix.hits.len(), 2);
    assert!(
        prefix
            .hits
            .iter()
            .all(|hit| hit.source_format != "Proposal")
    );

    let proposal_only = engine
        .search(
            Query::Exact {
                spelling: "proposalonly".into(),
            },
            SearchOptions {
                limit: 20,
                include_proposals: true,
            },
        )
        .unwrap();
    assert_eq!(proposal_only.total_records, 1);
    assert_eq!(proposal_only.hits[0].source_format, "Proposal");
    assert!(
        engine
            .search(
                Query::ReverseDefinition {
                    contains: "plum".into()
                },
                SearchOptions {
                    limit: 20,
                    include_proposals: true
                }
            )
            .is_err()
    );
    drop(engine);
    std::fs::remove_dir_all(path).unwrap();
}

#[test]
fn compose_kind_is_validated_with_empty_and_nonempty_candidate_sets() {
    let (path, engine) = fixture_store();
    for term in ["plum", "absent-cdc-marker"] {
        let result = engine.search(
            Query::Compose {
                definition_contains: Some(term.into()),
                links_from: None,
                min_nuclei: None,
                kind: Some("not-a-kind".into()),
            },
            SearchOptions::default(),
        );
        assert!(
            matches!(result, Err(vlajmaji_search::SearchError::InvalidQuery(_))),
            "term={term}: {result:?}"
        );
    }
    drop(engine);
    std::fs::remove_dir_all(path).unwrap();
}

#[test]
fn native_candidate_counts_are_stage_labelled() {
    let (path, engine) = fixture_store();
    let exact = engine
        .search(
            Query::Exact {
                spelling: "farvi".into(),
            },
            SearchOptions::default(),
        )
        .unwrap();
    assert_eq!(exact.native_candidate_counts.len(), 1);
    assert_eq!(exact.native_candidate_counts[0].stage, "exact_spelling");
    let compose = engine
        .search(
            Query::Compose {
                definition_contains: Some("plum".into()),
                links_from: Some("farvi".into()),
                min_nuclei: None,
                kind: None,
            },
            SearchOptions::default(),
        )
        .unwrap();
    assert_eq!(
        compose
            .native_candidate_counts
            .iter()
            .map(|count| count.stage.as_str())
            .collect::<Vec<_>>(),
        vec!["definition_contains", "links_from"]
    );
    drop(engine);
    std::fs::remove_dir_all(path).unwrap();
}

#[test]
fn search_engine_has_safe_summary_debug_output() {
    fn assert_debug<T: std::fmt::Debug>() {}
    assert_debug::<SearchEngine>();
    let (path, engine) = fixture_store();
    let rendered = format!("{engine:?}");
    assert!(rendered.contains("SearchEngine"));
    assert!(rendered.contains("corpus_records"));
    assert!(!rendered.contains("proposed alternative"));
    drop(engine);
    std::fs::remove_dir_all(path).unwrap();
}

#[test]
fn native_exact_preserves_alternatives_and_limit_zero_keeps_total() {
    let (path, engine) = fixture_store();
    let result = engine
        .search(
            Query::Exact {
                spelling: "farvi".into(),
            },
            SearchOptions {
                limit: 0,
                include_proposals: false,
            },
        )
        .unwrap();
    assert_eq!(result.total_records, 2);
    assert_eq!(result.distinct_spellings, 1);
    assert!(result.hits.is_empty());
    assert!(result.native_path.is_some());
    drop(engine);
    std::fs::remove_dir_all(path).unwrap();
}

#[test]
fn scalar_spelling_distance_scans_and_preserves_record_multiplicity() {
    let (path, engine) = fixture_store();
    let result = engine
        .search(
            Query::SpellingDistance {
                anchor: "farvi".into(),
                max_distance: Some(1),
            },
            SearchOptions {
                limit: 10,
                include_proposals: false,
            },
        )
        .unwrap();
    assert_eq!(result.total_records, 2);
    assert_eq!(result.distinct_spellings, 1);
    assert!(
        result
            .hits
            .iter()
            .all(|hit| hit.spelling_distance == Some(0))
    );
    drop(engine);
    std::fs::remove_dir_all(path).unwrap();
}

#[test]
fn contrast_exposes_p01_coordinate_explanations_without_word_total() {
    let (path, engine) = fixture_store();
    let result = engine
        .search(
            Query::Contrast {
                anchor: "farvi".into(),
                kind: "brivla".into(),
                require_mismatch: true,
            },
            SearchOptions {
                limit: 10,
                include_proposals: false,
            },
        )
        .unwrap();
    assert_eq!(result.total_records, 1);
    let hit = &result.hits[0];
    assert_eq!(hit.spelling.as_deref(), Some("snime"));
    assert_eq!(hit.distance_positions.len(), 2);
    assert!(
        hit.distance_positions
            .iter()
            .all(|item| !item.contributions.is_empty())
    );
    drop(engine);
    std::fs::remove_dir_all(path).unwrap();
}

#[test]
fn source_native_class_mapping_is_format_aware_and_preserves_unknowns() {
    let mut lexicon = Lexicon::new();
    lexicon
        .import_source(
            SourceMetadata::new(
                "lensisku",
                "Lensisku fixture",
                SourceFormat::LensiskuJson,
                "lensisku.json",
                "v1",
                None,
                TimePrecision::Unknown,
            ),
            br#"[
                {"valsiword":"farvi","definition":"grow","type_name":"gismu"},
                {"valsiword":"mysteryword","definition":"unknown class","type_name":"future-native-class"}
            ]"#,
            None,
        )
        .unwrap();

    let mut gismu_line = vec![b' '; 158];
    gismu_line[1..6].copy_from_slice(b"klama");
    gismu_line[62..66].copy_from_slice(b"move");
    let mut gismu_source = b"header\n".to_vec();
    gismu_source.extend_from_slice(&gismu_line);
    gismu_source.push(b'\n');
    lexicon
        .import_source(
            SourceMetadata::new(
                "llg-gismu",
                "LLG gismu fixture",
                SourceFormat::LlgGismuFixed,
                "gismu.txt",
                "v1",
                None,
                TimePrecision::Unknown,
            ),
            &gismu_source,
            None,
        )
        .unwrap();

    let mut cmavo_line = vec![b' '; 80];
    cmavo_line[0..2].copy_from_slice(b"mi");
    cmavo_line[10..14].copy_from_slice(b"KOhA");
    cmavo_line[20..24].copy_from_slice(b"me/I");
    cmavo_line[61..69].copy_from_slice(b"pronoun ");
    let mut cmavo_source = b"header\n".to_vec();
    cmavo_source.extend_from_slice(&cmavo_line);
    cmavo_source.push(b'\n');
    lexicon
        .import_source(
            SourceMetadata::new(
                "llg-cmavo",
                "LLG cmavo fixture",
                SourceFormat::LlgCmavoFixed,
                "cmavo.txt",
                "v1",
                None,
                TimePrecision::Unknown,
            ),
            &cmavo_source,
            None,
        )
        .unwrap();

    lexicon
        .import_source(
            SourceMetadata::new(
                "llg-rafsi",
                "LLG assigned rafsi fixture",
                SourceFormat::LlgRafsiRows,
                "rafsi.txt",
                "v1",
                None,
                TimePrecision::Unknown,
            ),
            b"far snime assigned short form\n",
            None,
        )
        .unwrap();

    let archive = lexicon.to_archive_bytes().unwrap();
    let path = std::env::temp_dir().join(format!(
        "relojban-search-class-map-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let engine = SearchEngine::build(&path, &archive).unwrap();

    let hit = |spelling: &str| {
        engine
            .search(
                Query::Exact {
                    spelling: spelling.into(),
                },
                SearchOptions::default(),
            )
            .unwrap()
            .hits
            .into_iter()
            .next()
            .unwrap()
    };
    let lensisku = hit("farvi");
    assert_eq!(lensisku.source_class.as_deref(), Some("gismu"));
    assert_eq!(lensisku.mapped_word_kind.as_deref(), Some("brivla"));
    assert!(
        lensisku
            .mapping_rule
            .as_deref()
            .unwrap()
            .contains("lensisku-type_name")
    );

    let unknown = hit("mysteryword");
    assert_eq!(unknown.source_class.as_deref(), Some("future-native-class"));
    assert_eq!(unknown.mapped_word_kind, None);
    assert_eq!(unknown.mapping_rule, None);

    let llg_gismu = hit("klama");
    assert_eq!(llg_gismu.source_class.as_deref(), Some("gismu"));
    assert_eq!(llg_gismu.mapped_word_kind.as_deref(), Some("brivla"));
    assert!(
        llg_gismu
            .mapping_rule
            .as_deref()
            .unwrap()
            .contains("llg-gismu-fixed")
    );

    let llg_cmavo = hit("mi");
    assert_eq!(llg_cmavo.source_class.as_deref(), Some("cmavo"));
    assert_eq!(llg_cmavo.mapped_word_kind.as_deref(), Some("cmavo"));
    assert!(
        llg_cmavo
            .mapping_rule
            .as_deref()
            .unwrap()
            .contains("llg-cmavo-fixed")
    );

    let rafsi = hit("snime");
    assert_eq!(rafsi.source_class, None);
    assert_eq!(rafsi.mapped_word_kind, None);
    drop(engine);
    std::fs::remove_dir_all(path).unwrap();
}

#[cfg(not(feature = "morphology"))]
#[test]
fn disabled_morphology_is_a_typed_error_not_empty_success() {
    let (path, engine) = fixture_store();
    let components = engine.search(
        Query::Components {
            components: vec!["farvi".into()],
        },
        SearchOptions::default(),
    );
    assert!(matches!(
        components,
        Err(vlajmaji_search::SearchError::FeatureDisabled("morphology"))
    ));

    let generated = engine.search(
        Query::Generate {
            components: vec!["farvi".into(), "snime".into()],
            setting: vlajmaji_search::MorphologySetting::Standard,
        },
        SearchOptions::default(),
    );
    assert!(matches!(
        generated,
        Err(vlajmaji_search::SearchError::FeatureDisabled("morphology"))
    ));
    drop(engine);
    std::fs::remove_dir_all(path).unwrap();
}
