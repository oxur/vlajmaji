//! Literal and typed-value contracts for readable compact output.

#[path = "../../lexicon/examples/support/dictionary_fixture.rs"]
mod fixture;

use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use vlajmaji_lexicon::{
    CaptureRef, Eligibility, EntityId, LocalRef, OccurrenceRef, Presence, RelationTarget, SourceId,
    TextValue,
};
use vlajmaji_search::{
    CollectionCompactQueryResponse, CollectionCompactQueryResult,
    CollectionCompactReferenceResponse, CollectionSelectionSnapshot,
    DictionaryCompactExactResponse, DictionaryCompactGroup, DictionaryCompactQueryResult,
    DictionaryCompactReferenceResolution, DictionaryCompactSearchEngine, DictionaryExactOptions,
    DictionaryHit, DictionaryHitDetailRef, DictionaryHitDetailResponse, DictionarySearchEngine,
    DictionaryTotals, ReferenceStatus, render_collection_compact, render_dictionary_compact,
    render_dictionary_detail,
};

static NEXT_PATH: AtomicU64 = AtomicU64::new(0);
const DETAIL_KEY: &str = r#"{"version":1,"digest_policy":"entry-assertion-serde-json-v1","role":"assertion","source":"source-a","entry_id":"entry-a","assertion_id":"assertion-a","occurrence":{"capture":{"namespace":"fixture:readable","sha256":"0000000000000000000000000000000000000000000000000000000000000000"},"locator":"/0"},"entry_sha256":"1111111111111111111111111111111111111111111111111111111111111111","assertion_sha256":"2222222222222222222222222222222222222222222222222222222222222222","native_entry_id":null,"native_assertion_id":null,"contributor_id":null,"distributor_id":null}"#;

fn scratch() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock is after the Unix epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "vlajmaji-s05-readable-{}-{nonce}-{}",
        std::process::id(),
        NEXT_PATH.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).expect("create owned test root");
    root
}

fn compact_engine() -> (PathBuf, DictionaryCompactSearchEngine) {
    let root = scratch();
    let store = root.join("dictionary.sqlite3");
    let dictionary = fixture::complete_fixture().expect("complete fixture");
    let bytes = dictionary.to_json_bytes().expect("fixture JSON");
    DictionarySearchEngine::build(&store, &bytes).expect("build fixture store");
    let engine = DictionaryCompactSearchEngine::open(&store).expect("open compact engine");
    (root, engine)
}

#[test]
fn test_render_dictionary_compact_zero_exact_matches_literal_golden() {
    let response = DictionaryCompactExactResponse {
        version: 1,
        projection_policy: "compact-dictionary-v1".into(),
        policy: "exact".into(),
        store_schema: 2,
        catalogue_revision: 0,
        state_sha256: "0".repeat(64),
        snapshot_sha256: "1".repeat(64),
        spelling: "farvi".into(),
        resolved_sources: Vec::new(),
        native_candidate_occurrences: 0,
        totals: DictionaryTotals {
            occurrences: 0,
            distinct_spellings: 0,
            source_groups: 0,
        },
        source_totals: Vec::new(),
        availability: Vec::new(),
        groups: Vec::new(),
        next_cursor: None,
        code_notices: Vec::new(),
    };
    let rendered =
        render_dictionary_compact(&DictionaryCompactQueryResult::Exact(Box::new(response)));
    assert_eq!(
        rendered,
        "Exact \"farvi\": 0 occurrences, 0 spellings, 0 sources\nnext_cursor: null\n"
    );
    assert_eq!(
        rendered,
        render_dictionary_compact(&DictionaryCompactQueryResult::Exact(Box::new(
            DictionaryCompactExactResponse {
                version: 1,
                projection_policy: "compact-dictionary-v1".into(),
                policy: "exact".into(),
                store_schema: 2,
                catalogue_revision: 0,
                state_sha256: "0".repeat(64),
                snapshot_sha256: "1".repeat(64),
                spelling: "farvi".into(),
                resolved_sources: Vec::new(),
                native_candidate_occurrences: 0,
                totals: DictionaryTotals {
                    occurrences: 0,
                    distinct_spellings: 0,
                    source_groups: 0,
                },
                source_totals: Vec::new(),
                availability: Vec::new(),
                groups: Vec::new(),
                next_cursor: None,
                code_notices: Vec::new(),
            },
        )))
    );
}

#[test]
fn test_render_dictionary_compact_unknown_resolve_matches_literal_golden() {
    let response = DictionaryCompactReferenceResolution {
        version: 1,
        projection_policy: "compact-dictionary-v1".into(),
        state_sha256: "0".repeat(64),
        snapshot_sha256: "1".repeat(64),
        target: RelationTarget::Unknown,
        status: ReferenceStatus::Unknown,
        entity: None,
        confirmed: Vec::new(),
        unverified: Vec::new(),
        availability: Vec::new(),
        requested_release: None,
    };
    assert_eq!(
        render_dictionary_compact(&DictionaryCompactQueryResult::Resolve(Box::new(response))),
        "Reference \"Unknown\": Unknown\nentity: null\nconfirmed: 0\nunverified: 0\nrequested_release: null\n"
    );
}

#[test]
fn test_render_dictionary_compact_unresolved_atoms_use_json_string_escaping() {
    let response = DictionaryCompactReferenceResolution {
        version: 1,
        projection_policy: "compact-dictionary-v1".into(),
        state_sha256: "0".repeat(64),
        snapshot_sha256: "1".repeat(64),
        target: RelationTarget::Unresolved("esc\u{1b}cr\r tab\tline\nslash\\quote\"é".into()),
        status: ReferenceStatus::Unresolved,
        entity: None,
        confirmed: Vec::new(),
        unverified: Vec::new(),
        availability: Vec::new(),
        requested_release: None,
    };
    assert_eq!(
        render_dictionary_compact(&DictionaryCompactQueryResult::Resolve(Box::new(response))),
        "Reference {\"Unresolved\":\"esc\\u001bcr\\r tab\\tline\\nslash\\\\quote\\\"é\"}: Unresolved\nentity: null\nconfirmed: 0\nunverified: 0\nrequested_release: null\n"
    );
}

#[test]
fn test_render_dictionary_compact_nomatch_resolve_matches_literal_golden() {
    let response = DictionaryCompactReferenceResolution {
        version: 1,
        projection_policy: "compact-dictionary-v1".into(),
        state_sha256: "0".repeat(64),
        snapshot_sha256: "1".repeat(64),
        target: RelationTarget::Local(LocalRef::Assertion(EntityId::new("assertion-a").unwrap())),
        status: ReferenceStatus::NoMatch,
        entity: None,
        confirmed: Vec::new(),
        unverified: Vec::new(),
        availability: Vec::new(),
        requested_release: None,
    };
    assert_eq!(
        render_dictionary_compact(&DictionaryCompactQueryResult::Resolve(Box::new(response))),
        "Reference {\"Local\":{\"Assertion\":\"assertion-a\"}}: NoMatch\nentity: null\nconfirmed: 0\nunverified: 0\nrequested_release: null\n"
    );
}

#[test]
fn test_render_dictionary_compact_resolve_groups_confirmed_and_unverified() {
    let (root, engine) = compact_engine();
    let dictionary = fixture::complete_fixture().expect("complete fixture");
    let assertion = dictionary
        .metadata()
        .assertions
        .iter()
        .find(|row| row.eligibility == vlajmaji_lexicon::Eligibility::Lexical)
        .expect("fixture lexical assertion")
        .id
        .clone();
    let mut response = engine
        .resolve(&RelationTarget::Local(LocalRef::Assertion(assertion)))
        .expect("resolve fixture assertion");
    let available = response
        .availability
        .first_mut()
        .expect("fixture has source availability");
    available.source.abbreviation = "A".into();
    let available_source = available.source.id.clone();
    let mut confirmed = response
        .confirmed
        .first()
        .cloned()
        .expect("fixture has a confirmed assertion");
    confirmed.entry.source = available_source;
    let mut fallback = confirmed.clone();
    fallback.entry.source = SourceId::new("source-z").unwrap();
    response.confirmed = vec![confirmed];
    response.unverified = vec![fallback];
    let rendered =
        render_dictionary_compact(&DictionaryCompactQueryResult::Resolve(Box::new(response)));
    let expected = r#"Reference {"Local":{"Assertion":"assertion-0"}}: Resolved
entity: {"role":"assertion","id":"assertion-0","entry_ids":["entry-a-farvi"],"assertion_ids":["assertion-0"],"source_ids":["source-a"],"occurrence":{"capture":{"namespace":"captured:a:d1","sha256":"0ed0ed8d96a400d39d61527a76542ddc26b855739d0689c04d3fe848727a01e3"},"locator":"/0"},"evidence":null,"spelling":null}
confirmed: 1
A [source-a] 1 confirmed
  assertion-0 @ 000000000000000d63617074757265643a613a643100000000000000403065643065643864393661343030643339643631353237613736353432646463323662383535373339643036383963303464336665383438373237613031653300000000000000022f30
    spelling: {"Value":"farvi"}
    definition: {"Value":{"text":"$x_1$ (object) develops/evolves towards/into $x_2$ (state) from $x_3$ (state) for reason/because of $x_4$ (fact, event) through stages $x_5$","language":{"Value":"en"},"markup":"Missing"}}
    eligibility: "Lexical"
    payload_present: true
    current_eligible: true
    detail: {"version":1,"digest_policy":"entry-assertion-serde-json-v1","role":"assertion","source":"source-a","entry_id":"entry-a-farvi","assertion_id":"assertion-0","occurrence":{"capture":{"namespace":"captured:a:d1","sha256":"0ed0ed8d96a400d39d61527a76542ddc26b855739d0689c04d3fe848727a01e3"},"locator":"/0"},"entry_sha256":"8fcf293a117bddcb68dc6d9a330905a3fe257e129d81002c87e4bb47d3ceffbf","assertion_sha256":"4f4775ec379d731de7a2c8554332b7c063ce4be9d74d27604a8434c57c49bdda","native_entry_id":"333","native_assertion_id":"69002","contributor_id":"suzil","distributor_id":"distributor-a-d1"}
unverified: 1
? [source-z] 1 unverified
  assertion-0 @ 000000000000000d63617074757265643a613a643100000000000000403065643065643864393661343030643339643631353237613736353432646463323662383535373339643036383963303464336665383438373237613031653300000000000000022f30
    spelling: {"Value":"farvi"}
    definition: {"Value":{"text":"$x_1$ (object) develops/evolves towards/into $x_2$ (state) from $x_3$ (state) for reason/because of $x_4$ (fact, event) through stages $x_5$","language":{"Value":"en"},"markup":"Missing"}}
    eligibility: "Lexical"
    payload_present: true
    current_eligible: true
    detail: {"version":1,"digest_policy":"entry-assertion-serde-json-v1","role":"assertion","source":"source-a","entry_id":"entry-a-farvi","assertion_id":"assertion-0","occurrence":{"capture":{"namespace":"captured:a:d1","sha256":"0ed0ed8d96a400d39d61527a76542ddc26b855739d0689c04d3fe848727a01e3"},"locator":"/0"},"entry_sha256":"8fcf293a117bddcb68dc6d9a330905a3fe257e129d81002c87e4bb47d3ceffbf","assertion_sha256":"4f4775ec379d731de7a2c8554332b7c063ce4be9d74d27604a8434c57c49bdda","native_entry_id":"333","native_assertion_id":"69002","contributor_id":"suzil","distributor_id":"distributor-a-d1"}
requested_release: null
"#;
    assert_eq!(rendered, expected);
    drop(engine);
    fs::remove_dir_all(root).expect("remove owned test root");
}

fn literal_resolve_input() -> DictionaryCompactReferenceResolution {
    let (root, engine) = compact_engine();
    let dictionary = fixture::complete_fixture().expect("complete fixture");
    let seed_assertion_id = dictionary
        .metadata()
        .assertions
        .iter()
        .find(|row| row.eligibility == Eligibility::Lexical)
        .expect("fixture lexical assertion")
        .id
        .clone();
    let mut response = engine
        .resolve(&RelationTarget::Local(LocalRef::Assertion(
            seed_assertion_id,
        )))
        .expect("seed typed projection structs");
    let mut first_availability = response.availability[0].clone();
    first_availability.source.id = SourceId::new("source-a").unwrap();
    first_availability.source.abbreviation = "A".into();
    let mut later_availability = first_availability.clone();
    later_availability.source.abbreviation = "wrong".into();

    let mut confirmed = response.confirmed[0].clone();
    confirmed.entry.id = EntityId::new("entry-a").unwrap();
    confirmed.entry.source = SourceId::new("source-a").unwrap();
    confirmed.entry.spelling = Presence::Value("farvi".into());
    confirmed.assertion.id = EntityId::new("assertion-a").unwrap();
    confirmed.assertion.entry = confirmed.entry.id.clone();
    confirmed.assertion.occurrence = literal_detail_ref().occurrence;
    confirmed.assertion.definition = Presence::Value(TextValue {
        text: "é\n\"quoted\"".into(),
        language: Presence::Missing,
        markup: Presence::Null,
    });
    confirmed.assertion.eligibility = Eligibility::Lexical;
    confirmed.occurrence_id = "00".into();
    confirmed.payload_present = true;
    confirmed.current_eligible = true;
    confirmed.detail_ref = Some(literal_detail_ref());

    let mut excluded = confirmed.clone();
    excluded.entry.id = EntityId::new("entry-z").unwrap();
    excluded.entry.source = SourceId::new("source-z").unwrap();
    excluded.entry.spelling = Presence::Null;
    excluded.assertion.id = EntityId::new("assertion-z").unwrap();
    excluded.assertion.entry = excluded.entry.id.clone();
    excluded.assertion.definition = Presence::Missing;
    excluded.assertion.eligibility = Eligibility::Excluded {
        reason: "not lexical".into(),
    };
    excluded.occurrence_id = "01".into();
    excluded.payload_present = false;
    excluded.current_eligible = false;
    excluded.detail_ref = None;

    response.target =
        RelationTarget::Local(LocalRef::Assertion(EntityId::new("assertion-a").unwrap()));
    response.status = ReferenceStatus::Unverified;
    response.entity = None;
    response.confirmed = vec![confirmed];
    response.unverified = vec![excluded];
    response.availability = vec![first_availability, later_availability];
    response.requested_release = None;
    drop(engine);
    fs::remove_dir_all(root).expect("remove seed fixture root");
    response
}

#[test]
fn test_render_dictionary_compact_resolve_matches_mixed_literal_golden() {
    let response = literal_resolve_input();
    const EXPECTED: &str = "Reference {\"Local\":{\"Assertion\":\"assertion-a\"}}: Unverified\nentity: null\nconfirmed: 1\nA [source-a] 1 confirmed\n  assertion-a @ 00\n    spelling: {\"Value\":\"farvi\"}\n    definition: {\"Value\":{\"text\":\"é\\n\\\"quoted\\\"\",\"language\":\"Missing\",\"markup\":\"Null\"}}\n    eligibility: \"Lexical\"\n    payload_present: true\n    current_eligible: true\n    detail: {\"version\":1,\"digest_policy\":\"entry-assertion-serde-json-v1\",\"role\":\"assertion\",\"source\":\"source-a\",\"entry_id\":\"entry-a\",\"assertion_id\":\"assertion-a\",\"occurrence\":{\"capture\":{\"namespace\":\"fixture:readable\",\"sha256\":\"0000000000000000000000000000000000000000000000000000000000000000\"},\"locator\":\"/0\"},\"entry_sha256\":\"1111111111111111111111111111111111111111111111111111111111111111\",\"assertion_sha256\":\"2222222222222222222222222222222222222222222222222222222222222222\",\"native_entry_id\":null,\"native_assertion_id\":null,\"contributor_id\":null,\"distributor_id\":null}\nunverified: 1\n? [source-z] 1 unverified\n  assertion-z @ 01\n    spelling: \"Null\"\n    definition: \"Missing\"\n    eligibility: {\"Excluded\":{\"reason\":\"not lexical\"}}\n    payload_present: false\n    current_eligible: false\n    detail: null\nrequested_release: null\n";
    assert_eq!(
        render_dictionary_compact(&DictionaryCompactQueryResult::Resolve(Box::new(response))),
        EXPECTED
    );
}

#[test]
fn test_render_dictionary_compact_groups_sources_and_preserves_row_multiplicity() {
    let mut response = literal_resolve_input();
    let source_z = response.unverified[0].clone();
    let source_a = response.confirmed[0].clone();
    let mut source_z2 = source_z.clone();
    source_z2.assertion.id = EntityId::new("assertion-z2").unwrap();
    source_z2.occurrence_id = "02".into();
    let mut source_a2 = source_a.clone();
    source_a2.assertion.id = EntityId::new("assertion-a2").unwrap();
    source_a2.occurrence_id = "03".into();
    response.confirmed = vec![source_z, source_a.clone(), source_z2, source_a2, source_a];
    response.unverified.clear();

    let input_sources: Vec<_> = response
        .confirmed
        .iter()
        .map(|row| row.entry.source.as_str())
        .collect();
    assert_eq!(
        input_sources,
        ["source-z", "source-a", "source-z", "source-a", "source-a"]
    );
    let rendered =
        render_dictionary_compact(&DictionaryCompactQueryResult::Resolve(Box::new(response)));
    let headers: Vec<_> = rendered
        .lines()
        .filter(|line| line.ends_with(" confirmed") || line.ends_with(" unverified"))
        .collect();
    assert_eq!(
        headers,
        ["A [source-a] 3 confirmed", "? [source-z] 2 confirmed"]
    );
    let row_ids: Vec<_> = rendered
        .lines()
        .filter_map(|line| line.strip_prefix("  "))
        .filter_map(|line| line.split_once(" @ ").map(|(id, _)| id))
        .collect();
    assert_eq!(
        row_ids,
        [
            "assertion-a",
            "assertion-a2",
            "assertion-a",
            "assertion-z",
            "assertion-z2"
        ]
    );
}

#[test]
fn test_render_dictionary_compact_actual_nonempty_page_zero_keeps_totals_and_is_repeatable() {
    let (root, engine) = compact_engine();
    let unpaged = engine
        .exact("farvi", DictionaryExactOptions::default())
        .expect("nonempty fixture exact result");
    assert!(unpaged.totals.occurrences > 0);
    assert!(!unpaged.groups.is_empty());

    let page_zero = engine
        .exact(
            "farvi",
            DictionaryExactOptions {
                page_size: Some(0),
                ..DictionaryExactOptions::default()
            },
        )
        .expect("page-size-zero query result");
    assert_eq!(page_zero.totals, unpaged.totals);
    assert!(page_zero.groups.is_empty());
    assert!(page_zero.next_cursor.is_none());
    let expected = format!(
        "Exact \"farvi\": {} occurrences, {} spellings, {} sources\nnext_cursor: null\n",
        unpaged.totals.occurrences, unpaged.totals.distinct_spellings, unpaged.totals.source_groups,
    );
    let result = DictionaryCompactQueryResult::Exact(Box::new(page_zero));
    let rendered = render_dictionary_compact(&result);
    assert_eq!(rendered, expected);
    assert_eq!(render_dictionary_compact(&result), expected);
    drop(engine);
    fs::remove_dir_all(root).expect("remove owned test root");
}

#[test]
fn test_render_collection_compact_metadata_only_resolve_matches_literal_golden() {
    let snapshot = CollectionSelectionSnapshot {
        version: 1,
        profile: None,
        local_revision: 0,
        generation_id: None,
        content_id: None,
        recorded_state_sha256: None,
        packs: Vec::new(),
        disabled: Vec::new(),
        selected_releases: Default::default(),
        collections: Vec::new(),
        selection_policy: "selected-available-owned-head-v2".into(),
        order_policy: "source-release-occurrence-v1".into(),
        projection_policy: "compact-dictionary-v1".into(),
        binding_sha256: "3".repeat(64),
    };
    let result =
        CollectionCompactQueryResult::Resolve(Box::new(CollectionCompactReferenceResponse {
            target: RelationTarget::Local(LocalRef::Assertion(
                EntityId::new("assertion-a").unwrap(),
            )),
            status: ReferenceStatus::CatalogueOnly,
            operational_reason: Some("removed".into()),
            payload: None,
            catalogue_matches: Vec::new(),
            requested_release: None,
        }));
    let response = CollectionCompactQueryResponse {
        version: 1,
        snapshot,
        collections: Vec::new(),
        result,
    };
    assert_eq!(
        render_collection_compact(&response),
        "Reference {\"Local\":{\"Assertion\":\"assertion-a\"}}: CatalogueOnly\nreason: \"removed\"\npayload: false\nentity: null\nconfirmed: 0\nunverified: 0\nrequested_release: null\ncatalogue_matches: []\nselection: {\"version\":1,\"profile\":null,\"local_revision\":0,\"generation_id\":null,\"content_id\":null,\"recorded_state_sha256\":null,\"packs\":[],\"disabled\":[],\"selected_releases\":{},\"collections\":[],\"selection_policy\":\"selected-available-owned-head-v2\",\"order_policy\":\"source-release-occurrence-v1\",\"projection_policy\":\"compact-dictionary-v1\",\"binding_sha256\":\"3333333333333333333333333333333333333333333333333333333333333333\"}\ncollections: []\n"
    );
}

#[test]
fn test_render_dictionary_detail_nomatch_matches_literal_golden() {
    let detail_ref = literal_detail_ref();
    let response = DictionaryHitDetailResponse {
        version: 1,
        projection_policy: "complete-dictionary-hit-v1".into(),
        detail_ref,
        status: ReferenceStatus::NoMatch,
        operational_reason: Some("lexical_identity_changed".into()),
        state_sha256: None,
        snapshot_sha256: "3".repeat(64),
        hit: None,
        current_eligible: None,
    };
    assert_eq!(
        render_dictionary_detail(&response),
        format!(
            "Detail assertion-a: NoMatch\nsnapshot: \"{}\"\nreason: \"lexical_identity_changed\"\ncurrent_eligible: null\nreference: {}\nhit: null\n",
            "3".repeat(64),
            DETAIL_KEY,
        )
    );
}

#[test]
fn test_render_dictionary_detail_complete_typed_hit_matches_literal_golden() {
    const HIT_JSON: &str = r#"{"occurrence_id":"00","entry":{"id":"entry-a","source":"source-a","spelling":{"Value":"farvi"},"language":"Missing","script":"Null","native_word_class":"Missing","selmaho":"Missing","mapped_pos":"Missing"},"assertion":{"id":"assertion-a","entry":"entry-a","occurrence":{"capture":{"namespace":"fixture:readable","sha256":"0000000000000000000000000000000000000000000000000000000000000000"},"locator":"/0"},"native_sense_id":"Null","language":"Missing","definition":{"Value":{"text":"é\n\"quoted\"","language":"Missing","markup":"Null"}},"source_score":{"Value":"1e+9999"},"timestamps":[],"eligibility":"Lexical"},"capture":{"source_name":"Fixture é","format_label":"json","locator":"memory:readable","revision":"Missing","captured_at":"Null","time_precision":"Unknown","notice_locator":"Missing"},"raw":{"locator":"/0","fields":[{"name":"score","encoding":"Json","raw_value":"1e+9999"}],"raw_line":null},"snapshot_attribution":{"snapshot":{"namespace":"fixture:readable","sha256":"0000000000000000000000000000000000000000000000000000000000000000"},"source":"source-a","distributor":null},"occurrence_attribution":null,"contributor":null,"distributor":null,"release_ids":["release-a"],"contexts":[],"frames":[],"forms":[],"etymologies":[],"relations":[],"editorial_senses":[],"computations":[],"annotations":[],"extensions":[],"owned_revision_ids":[],"tombstoned_by":[]}"#;
    const HIT_PRETTY: &str = r#"{
  "occurrence_id": "00",
  "entry": {
    "id": "entry-a",
    "source": "source-a",
    "spelling": {
      "Value": "farvi"
    },
    "language": "Missing",
    "script": "Null",
    "native_word_class": "Missing",
    "selmaho": "Missing",
    "mapped_pos": "Missing"
  },
  "assertion": {
    "id": "assertion-a",
    "entry": "entry-a",
    "occurrence": {
      "capture": {
        "namespace": "fixture:readable",
        "sha256": "0000000000000000000000000000000000000000000000000000000000000000"
      },
      "locator": "/0"
    },
    "native_sense_id": "Null",
    "language": "Missing",
    "definition": {
      "Value": {
        "text": "é\n\"quoted\"",
        "language": "Missing",
        "markup": "Null"
      }
    },
    "source_score": {
      "Value": "1e+9999"
    },
    "timestamps": [],
    "eligibility": "Lexical"
  },
  "capture": {
    "source_name": "Fixture é",
    "format_label": "json",
    "locator": "memory:readable",
    "revision": "Missing",
    "captured_at": "Null",
    "time_precision": "Unknown",
    "notice_locator": "Missing"
  },
  "raw": {
    "locator": "/0",
    "fields": [
      {
        "name": "score",
        "encoding": "Json",
        "raw_value": "1e+9999"
      }
    ],
    "raw_line": null
  },
  "snapshot_attribution": {
    "snapshot": {
      "namespace": "fixture:readable",
      "sha256": "0000000000000000000000000000000000000000000000000000000000000000"
    },
    "source": "source-a",
    "distributor": null
  },
  "occurrence_attribution": null,
  "contributor": null,
  "distributor": null,
  "release_ids": [
    "release-a"
  ],
  "contexts": [],
  "frames": [],
  "forms": [],
  "etymologies": [],
  "relations": [],
  "editorial_senses": [],
  "computations": [],
  "annotations": [],
  "extensions": [],
  "owned_revision_ids": [],
  "tombstoned_by": []
}"#;
    let hit: DictionaryHit = serde_json::from_str(HIT_JSON).expect("typed literal hit");
    assert_eq!(serde_json::to_string_pretty(&hit).unwrap(), HIT_PRETTY);
    let response = DictionaryHitDetailResponse {
        version: 1,
        projection_policy: "complete-dictionary-hit-v1".into(),
        detail_ref: literal_detail_ref(),
        status: ReferenceStatus::Resolved,
        operational_reason: None,
        state_sha256: Some("0".repeat(64)),
        snapshot_sha256: "3".repeat(64),
        hit: Some(hit),
        current_eligible: Some(true),
    };
    let expected = format!(
        "Detail assertion-a: Resolved\nsnapshot: \"{}\"\nreason: null\ncurrent_eligible: true\nreference: {}\nhit: {HIT_PRETTY}\n",
        "3".repeat(64),
        DETAIL_KEY,
    );
    assert_eq!(render_dictionary_detail(&response), expected);
}

fn literal_detail_ref() -> DictionaryHitDetailRef {
    DictionaryHitDetailRef {
        version: 1,
        digest_policy: "entry-assertion-serde-json-v1".into(),
        role: "assertion".into(),
        source: SourceId::new("source-a").unwrap(),
        entry_id: EntityId::new("entry-a").unwrap(),
        assertion_id: EntityId::new("assertion-a").unwrap(),
        occurrence: OccurrenceRef {
            capture: CaptureRef {
                namespace: "fixture:readable".into(),
                sha256: "0".repeat(64),
            },
            locator: "/0".into(),
        },
        entry_sha256: "1".repeat(64),
        assertion_sha256: "2".repeat(64),
        native_entry_id: None,
        native_assertion_id: None,
        contributor_id: None,
        distributor_id: None,
    }
}

#[test]
fn test_render_dictionary_compact_nonempty_presence_and_duplicate_occurrences_matches_literal_golden()
 {
    const KEY: &str = DETAIL_KEY;
    let (root, engine) = compact_engine();
    let mut response = engine
        .exact("farvi", DictionaryExactOptions::default())
        .expect("complete fixture exact result");
    let mut base = response.groups[0].hits[0].clone();
    let source_id = SourceId::new("source-a").unwrap();
    base.entry.source = source_id.clone();
    base.entry.id = EntityId::new("entry-a").unwrap();
    base.assertion.entry = base.entry.id.clone();
    base.assertion.id = EntityId::new("assertion-a").unwrap();
    base.assertion.occurrence = literal_detail_ref().occurrence.clone();
    base.occurrence_id = "00".into();
    base.detail_ref = literal_detail_ref();

    let mut missing = base.clone();
    missing.entry.spelling = Presence::Missing;
    missing.assertion.definition = Presence::Missing;
    let mut null = base.clone();
    null.entry.spelling = Presence::Null;
    null.assertion.definition = Presence::Null;
    let mut value = base;
    value.entry.spelling = Presence::Value("farvi".into());
    value.assertion.definition = Presence::Value(TextValue {
        text: "é\n\"quoted\"".into(),
        language: Presence::Missing,
        markup: Presence::Null,
    });

    let mut source = response.groups[0].source.clone();
    source.id = source_id;
    source.abbreviation = "A".into();
    response.spelling = "farvi".into();
    response.totals = DictionaryTotals {
        occurrences: 4,
        distinct_spellings: 1,
        source_groups: 1,
    };
    response.groups = vec![DictionaryCompactGroup {
        source,
        full_count: 4,
        returned_count: 3,
        partial: true,
        hits: vec![missing, null, value],
    }];
    response.next_cursor = Some("next-page".into());
    let expected = format!(
        "Exact \"farvi\": 4 occurrences, 1 spellings, 1 sources\nA [source-a] 3/4 (partial)\n  assertion-a @ 00\n    spelling: \"Missing\"\n    definition: \"Missing\"\n    detail: {KEY}\n  assertion-a @ 00\n    spelling: \"Null\"\n    definition: \"Null\"\n    detail: {KEY}\n  assertion-a @ 00\n    spelling: {{\"Value\":\"farvi\"}}\n    definition: {{\"Value\":{{\"text\":\"é\\n\\\"quoted\\\"\",\"language\":\"Missing\",\"markup\":\"Null\"}}}}\n    detail: {KEY}\nnext_cursor: \"next-page\"\n"
    );
    assert_eq!(serde_json::to_string(&literal_detail_ref()).unwrap(), KEY);
    assert_eq!(
        render_dictionary_compact(&DictionaryCompactQueryResult::Exact(Box::new(response))),
        expected
    );
    drop(engine);
    fs::remove_dir_all(root).expect("remove owned test root");
}

#[test]
fn test_render_dictionary_compact_preserves_typed_presence_and_duplicate_order() {
    let root = scratch();
    let store = root.join("dictionary.sqlite3");
    let dictionary = fixture::complete_fixture().expect("complete fixture");
    let metadata = dictionary.metadata();
    let context_id = metadata
        .assertions
        .iter()
        .find(|row| {
            row.occurrence.capture.namespace == "captured:a:lexical"
                && row.occurrence.locator == "/0"
        })
        .expect("fixture context assertion")
        .id
        .clone();
    let editorial_id = metadata
        .assertions
        .iter()
        .find(|row| {
            row.occurrence.capture.namespace == "captured:a:d1" && row.occurrence.locator == "/0"
        })
        .expect("fixture editorial assertion")
        .id
        .clone();
    let editorial_sense_id = metadata
        .editorial_senses
        .iter()
        .find(|row| row.assertions.contains(&editorial_id))
        .expect("fixture editorial link")
        .id
        .clone();
    let bytes = dictionary.to_json_bytes().expect("fixture JSON");
    DictionarySearchEngine::build(&store, &bytes).expect("build fixture store");
    let engine = DictionaryCompactSearchEngine::open(&store).expect("open compact engine");

    let compact = DictionaryCompactQueryResult::Exact(Box::new(
        engine
            .exact("farvi", DictionaryExactOptions::default())
            .expect("fixture exact result"),
    ));
    let rendered = render_dictionary_compact(&compact);
    let exact = match &compact {
        DictionaryCompactQueryResult::Exact(response) => response,
        DictionaryCompactQueryResult::Resolve(_) => unreachable!(),
    };
    assert!(rendered.ends_with("\n"));
    assert!(rendered.contains("next_cursor: null\n"));
    assert_eq!(
        rendered.matches("    detail: {").count(),
        exact
            .groups
            .iter()
            .map(|group| group.hits.len())
            .sum::<usize>()
    );
    for group in &exact.groups {
        let projected = group
            .hits
            .iter()
            .map(|hit| {
                format!(
                    "  {} @ {}\n    spelling: {}\n    definition: {}\n    detail: {}\n",
                    serde_json::to_string(hit.assertion.id.as_str())
                        .unwrap()
                        .trim_matches('"'),
                    serde_json::to_string(&hit.occurrence_id)
                        .unwrap()
                        .trim_matches('"'),
                    serde_json::to_string(&hit.entry.spelling).unwrap(),
                    serde_json::to_string(&hit.assertion.definition).unwrap(),
                    serde_json::to_string(&hit.detail_ref).unwrap(),
                )
            })
            .collect::<String>();
        assert!(rendered.contains(&projected));
    }

    let context_key = engine
        .exact("si'e", DictionaryExactOptions::default())
        .expect("si'e exact result")
        .groups
        .into_iter()
        .flat_map(|group| group.hits)
        .find(|hit| hit.assertion.id == context_id)
        .expect("pre-input context assertion hit");
    let context_detail = engine
        .detail(&context_key.detail_ref, None)
        .expect("si'e complete detail");
    let context_hit = context_detail.hit.as_ref().expect("si'e full hit");
    assert!(!context_hit.forms.is_empty());
    assert!(!context_hit.contexts.is_empty());
    let context_pretty = serde_json::to_string_pretty(context_hit).unwrap();
    assert!(
        render_dictionary_detail(&context_detail).ends_with(&format!("hit: {context_pretty}\n"))
    );

    let editorial_key = exact
        .groups
        .iter()
        .flat_map(|group| &group.hits)
        .find(|hit| hit.assertion.id == editorial_id)
        .expect("pre-input editorial farvi assertion");
    let detail = engine
        .detail(&editorial_key.detail_ref, None)
        .expect("complete fixture detail");
    let hit = detail.hit.as_ref().expect("complete hit");
    assert_eq!(hit.assertion.occurrence.capture.namespace, "captured:a:d1");
    assert_eq!(hit.assertion.occurrence.locator, "/0");
    assert!(
        hit.editorial_senses
            .iter()
            .any(|row| row.id == editorial_sense_id)
    );
    let pretty = serde_json::to_string_pretty(hit).unwrap();
    assert!(render_dictionary_detail(&detail).ends_with(&format!("hit: {pretty}\n")));

    drop(engine);
    fs::remove_dir_all(root).expect("remove owned test root");
}
