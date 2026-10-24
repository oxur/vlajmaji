//! Direct SQL and lifecycle contract for the versioned dictionary store.
use rusqlite::Connection;
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};
use vlajmaji_lexicon::Dictionary;
use vlajmaji_store::{DictionaryStore, DictionaryStoreError, LexicalStore};

#[path = "../../lexicon/examples/support/dictionary_fixture.rs"]
mod fixture_support;

type ReleaseOracleRow = (
    String,
    String,
    i64,
    String,
    String,
    String,
    String,
    Option<String>,
    Option<String>,
    Option<String>,
);
type FormOracleRow = (
    String,
    String,
    String,
    String,
    Option<String>,
    Option<String>,
);
type RelationOracleRow = (
    String,
    String,
    Option<String>,
    String,
    String,
    String,
    Option<String>,
    Option<String>,
    Option<String>,
);
type OwnedRevisionOracleRow = (
    String,
    Option<String>,
    String,
    String,
    String,
    Option<String>,
    String,
    String,
);

fn scratch() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let parent = std::env::temp_dir().join(format!(
        "vlajmaji-dictionary-store-{}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
        N.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&parent).unwrap();
    parent
}

#[test]
fn complete_dictionary_creates_typed_projection_and_reopens_read_only() {
    let parent = scratch();
    let dictionary = fixture_support::complete_fixture().unwrap();
    let canonical = dictionary.to_json_bytes().unwrap();
    let path = parent.join("candidate");
    let store = DictionaryStore::create(&path, &dictionary).unwrap();
    assert_eq!(store.canonical_bytes(), canonical);
    assert_eq!(store.dictionary().document(), dictionary.document());
    drop(store);
    let reopened = DictionaryStore::open(&path).unwrap();
    assert_eq!(reopened.dictionary().document(), dictionary.document());
    let db = Connection::open(reopened.database_path()).unwrap();
    for (table, count) in [
        ("capture_refs", 10),
        ("occurrence_refs", 21),
        ("raw_captures", 9),
        ("raw_occurrences", 20),
        ("entries", 14),
        ("assertions", 20),
        ("contexts", 8),
        ("argument_frames", 1),
        ("forms", 7),
        ("etymologies", 3),
        ("relations", 4),
        ("owned_revisions", 2),
        ("editorial_senses", 1),
        ("computations", 1),
        ("annotations", 1),
        ("extensions", 1),
    ] {
        let actual: i64 = db
            .query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0))
            .unwrap();
        assert_eq!(actual, count, "{table}");
    }
    assert_eq!(db.query_row::<(String,String,String,String),_,_>("SELECT spelling_state,spelling_value,selmaho_state,selmaho_value FROM entries WHERE id='entry-a-si-e'",[],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).unwrap(),("value".into(),"si'e".into(),"value".into(),"MOI".into()));
    assert_eq!(db.query_row::<(String,String),_,_>("SELECT definition_text,source_score_value FROM assertions WHERE id='assertion-0'",[],|r|Ok((r.get(0)?,r.get(1)?))).unwrap(),("$x_1$ (object) develops/evolves towards/into $x_2$ (state) from $x_3$ (state) for reason/because of $x_4$ (fact, event) through stages $x_5$".into(),"1.0".into()));
    assert_eq!(db.query_row::<(String,Option<String>),_,_>("SELECT captured_at_state,captured_at_value FROM raw_captures WHERE namespace='captured:a:opaque'",[],|r|Ok((r.get(0)?,r.get(1)?))).unwrap(),("missing".into(),None));
    assert_eq!(
        db.query_row::<Vec<u8>, _, _>(
            "SELECT bytes FROM raw_captures WHERE namespace='captured:a:opaque'",
            [],
            |r| r.get(0)
        )
        .unwrap(),
        b"opaque-source-row\0retained".to_vec()
    );
    assert_eq!(
        db.query_row::<String, _, _>("SELECT kind FROM forms WHERE id='form-assigned'", [], |r| r
            .get(0))
            .unwrap(),
        "AssignedRafsi"
    );
    assert_eq!(
        db.query_row::<(String, Option<String>), _, _>(
            "SELECT statement_state,statement_text FROM etymologies WHERE id='etymology-0'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?))
        )
        .unwrap(),
        ("null".into(), None)
    );
    assert_eq!(db.query_row::<(String,String),_,_>("SELECT target_kind,catalog_source_id FROM relations WHERE id='relation-missing-catalogue'",[],|r|Ok((r.get(0)?,r.get(1)?))).unwrap(),("Catalog".into(),"source-m".into()));
    assert_eq!(
        db.query_row::<(String, String), _, _>(
            "SELECT source_id,revision_id FROM owned_heads",
            [],
            |r| Ok((r.get(0)?, r.get(1)?))
        )
        .unwrap(),
        ("source-o".into(), "revision-r1".into())
    );
    assert_eq!(
        db.query_row::<String, _, _>(
            "SELECT value FROM annotations WHERE id='annotation-1'",
            [],
            |r| r.get(0)
        )
        .unwrap(),
        r#"{"score":0.5,"labels":["synthetic",true]}"#
    );
    assert_eq!(
        db.query_row::<String, _, _>(
            "SELECT value FROM extensions WHERE name='unknown-extension'",
            [],
            |r| r.get(0)
        )
        .unwrap(),
        r#"{"nested":[null,false,12345678901234567890]}"#
    );
    assert_eq!(db.query_row::<(String,String),_,_>("SELECT source_id,distributor_id FROM snapshot_bindings WHERE namespace='captured:a:d1'",[],|r|Ok((r.get(0)?,r.get(1)?))).unwrap(),("source-a".into(),"distributor-a-d1".into()));
    assert_eq!(
        db.query_row::<(String, String), _, _>(
            "SELECT text,status FROM source_aliases WHERE ordinal=0",
            [],
            |r| Ok((r.get(0)?, r.get(1)?))
        )
        .unwrap(),
        ("archive-a".into(), "Active".into())
    );
    assert_eq!(
        db.query_row::<(String, Option<String>), _, _>(
            "SELECT target_kind,unresolved_text FROM source_lineage WHERE ordinal=1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?))
        )
        .unwrap(),
        (
            "Unresolved".into(),
            Some("unresolved mirror lineage".into())
        )
    );
    assert_eq!(
        db.query_row::<i64, _, _>(
            "SELECT count(*) FROM occurrence_attributions WHERE native_assertion_id='69002'",
            [],
            |r| r.get(0)
        )
        .unwrap(),
        3
    );
    assert_eq!(
        db.query_row::<(String, String, String), _, _>(
            "SELECT availability,redistribution,accounting_unit FROM releases WHERE id='release-m'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?))
        )
        .unwrap(),
        ("Missing".into(), "Unknown".into(), "occurrence".into())
    );
    assert_eq!(db.query_row::<(String,Option<String>,Option<String>),_,_>("SELECT spelling_state,spelling_value,native_word_class_value FROM entries WHERE id='entry-a-sie'",[],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).unwrap(),("value".into(),Some("sie".into()),Some("cmavo".into())));
    assert_eq!(
        db.query_row::<String, _, _>(
            "SELECT mapped_pos_rule FROM entries WHERE id='entry-a-si-e'",
            [],
            |r| r.get(0)
        )
        .unwrap(),
        "explicit-fixture-map-v1"
    );
    assert_eq!(db.query_row::<(String,String),_,_>("SELECT field,value FROM assertion_timestamps WHERE assertion_id='assertion-0' ORDER BY ordinal LIMIT 1",[],|r|Ok((r.get(0)?,r.get(1)?))).unwrap(),("time".into(),"1479570326".into()));
    assert_eq!(
        db.query_row::<(String, Option<String>), _, _>(
            "SELECT value_state,value_text FROM contexts WHERE id='context-2'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?))
        )
        .unwrap(),
        ("null".into(), None)
    );
    assert_eq!(
        db.query_row::<(String, Option<String>), _, _>(
            "SELECT value_state,value_text FROM contexts WHERE id='context-3'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?))
        )
        .unwrap(),
        ("value".into(), Some(String::new()))
    );
    assert_eq!(db.query_row::<(i64,String,String),_,_>("SELECT slot_order,native_label,grammatical_role_value FROM argument_slots WHERE frame_id='frame-si-e'",[],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).unwrap(),(1,"x1".into(),"agent".into()));
    assert_eq!(
        db.query_row::<(String, Option<String>), _, _>(
            "SELECT spelling,standing_value FROM forms WHERE id='form-reported'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?))
        )
        .unwrap(),
        ("cpan".into(), Some("source-reported".into()))
    );
    assert_eq!(db.query_row::<(String,Option<String>,Option<String>),_,_>("SELECT statement_state,statement_text,native_kind_value FROM etymologies WHERE id='etymology-1'",[],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).unwrap(),("value".into(),Some(String::new()),Some("source etymology".into())));
    assert_eq!(
        db.query_row::<(String, String), _, _>(
            "SELECT kind,direction FROM relations WHERE id='relation-editorial'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?))
        )
        .unwrap(),
        ("EditorialLink".into(), "Forward".into())
    );
    assert_eq!(
        db.query_row::<i64, _, _>(
            "SELECT count(*) FROM revision_tombstones WHERE revision_id='revision-r2'",
            [],
            |r| r.get(0)
        )
        .unwrap(),
        1
    );
    assert_eq!(
        db.query_row::<(String, String), _, _>(
            "SELECT tool_version,policy FROM computations WHERE id='computation-1'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?))
        )
        .unwrap(),
        ("1".into(), "fixture-policy".into())
    );
    assert_eq!(
        db.query_row::<i64, _, _>(
            "SELECT count(*) FROM content_notices WHERE release_id='release-a'",
            [],
            |r| r.get(0)
        )
        .unwrap(),
        1
    );
    assert_eq!(
        db.query_row::<String, _, _>("SELECT text FROM code_notices WHERE ordinal=0", [], |r| r
            .get(0))
            .unwrap(),
        "fixture code notice"
    );
    assert!(matches!(
        DictionaryStore::create(&path, &dictionary),
        Err(DictionaryStoreError::ExistingPath(_))
    ));
    drop(reopened);
    fs::remove_dir_all(parent).unwrap();
}

#[test]
fn full_fixture_has_independent_direct_sql_value_oracles() {
    let parent = scratch();
    let dictionary = fixture_support::complete_fixture().unwrap();
    let path = parent.join("candidate");
    let store = DictionaryStore::create(&path, &dictionary).unwrap();
    let db = Connection::open(store.database_path()).unwrap();

    // Every captured BLOB is compared through SQL, not just the canonical JSON.
    for capture in dictionary.captures() {
        let bytes: Vec<u8> = db
            .query_row(
                "SELECT bytes FROM raw_captures WHERE namespace=?1 AND sha256=?2",
                (&capture.identity.namespace, &capture.identity.sha256),
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(
            bytes, capture.bytes,
            "raw capture {}",
            capture.identity.namespace
        );
    }

    let native_fields: Vec<(String, String, String)> = db
        .prepare("SELECT f.name,f.encoding,f.raw_value FROM native_fields f JOIN occurrence_refs o ON o.id=f.occurrence_id WHERE o.namespace='captured:a:lexical' AND o.locator='/0' ORDER BY f.ordinal")
        .unwrap()
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
        .unwrap()
        .collect::<rusqlite::Result<_>>()
        .unwrap();
    assert_eq!(native_fields, vec![
        ("word".into(), "Json".into(), r#""si'e""#.into()),
        ("definition".into(), "Json".into(), r#""cmavo, MOI""#.into()),
        ("word_type".into(), "Json".into(), r#""cmavo""#.into()),
        ("selmaho".into(), "Json".into(), r#""MOI""#.into()),
        ("notes".into(), "Json".into(), r#""synthetic note""#.into()),
        ("rafsi".into(), "Json".into(), r#"["sni"]"#.into()),
        ("variant".into(), "Json".into(), r#""siʼe""#.into()),
        ("pronunciation".into(), "Json".into(), r#""siʔe""#.into()),
        ("etymology".into(), "Json".into(), "null".into()),
        ("frame".into(), "Json".into(), r#"[{"order":1,"label":"x1","description":"snow"}]"#.into()),
        ("native_sense_id".into(), "Json".into(), r#""synthetic-sense-si-e""#.into()),
        ("source_see_also".into(), "Json".into(), "null".into()),
        ("source_etymology_target".into(), "Json".into(), r#"{"source":"source-m","release":"release-m","native_entry_id":"native-missing","occurrence":{"capture":{"namespace":"catalogue:m:future","sha256":"0000000000000000000000000000000000000000000000000000000000000000"},"locator":"/0"}}"#.into()),
        ("unknown".into(), "Json".into(), r#"{"array":[1,true,null],"huge":1234567890123456789012345678901234567890}"#.into()),
    ]);

    let releases: Vec<ReleaseOracleRow> = db
        .prepare("SELECT id,availability,selected,upstream_revision_state,upstream_revision_value,normalizer,normalizer_version,accounting_input_occurrences,accounting_normalized_occurrences,accounting_query_excluded_occurrences FROM releases JOIN entity_ids USING(id) ORDER BY entity_ids.ordinal")
        .unwrap()
        .query_map([], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?,r.get(6)?,r.get(7)?,r.get(8)?,r.get(9)?)))
        .unwrap().collect::<rusqlite::Result<_>>().unwrap();
    assert_eq!(
        releases,
        vec![
            (
                "release-a".into(),
                "Available".into(),
                1,
                "value".into(),
                "fixture-v1".into(),
                "explicit-import".into(),
                "1".into(),
                Some("17".into()),
                Some("17".into()),
                Some("1".into())
            ),
            (
                "release-b".into(),
                "Available".into(),
                1,
                "value".into(),
                "fixture-v1".into(),
                "explicit-import".into(),
                "1".into(),
                Some("1".into()),
                Some("1".into()),
                Some("0".into())
            ),
            (
                "release-o1".into(),
                "Available".into(),
                1,
                "value".into(),
                "fixture-v1".into(),
                "explicit-import".into(),
                "1".into(),
                Some("1".into()),
                Some("1".into()),
                Some("0".into())
            ),
            (
                "release-o2".into(),
                "Available".into(),
                0,
                "value".into(),
                "fixture-v1".into(),
                "explicit-import".into(),
                "1".into(),
                Some("1".into()),
                Some("1".into()),
                Some("0".into())
            ),
            (
                "release-m".into(),
                "Missing".into(),
                0,
                "value".into(),
                "fixture-v1".into(),
                "explicit-import".into(),
                "1".into(),
                None,
                None,
                None
            ),
        ]
    );

    let contexts: Vec<(String, String, String, Option<String>)> = db
        .prepare("SELECT id,kind,value_state,value_text FROM contexts JOIN entity_ids USING(id) ORDER BY entity_ids.ordinal")
        .unwrap()
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))
        .unwrap()
        .collect::<rusqlite::Result<_>>()
        .unwrap();
    assert_eq!(
        contexts,
        vec![
            (
                "context-0".into(),
                "Gloss".into(),
                "value".into(),
                Some("short gloss".into())
            ),
            (
                "context-1".into(),
                "Note".into(),
                "value".into(),
                Some("synthetic note".into())
            ),
            ("context-2".into(), "Label".into(), "null".into(), None),
            (
                "context-3".into(),
                "Register".into(),
                "value".into(),
                Some(String::new())
            ),
            (
                "context-4".into(),
                "Domain".into(),
                "value".into(),
                Some("context 4".into())
            ),
            (
                "context-5".into(),
                "Example".into(),
                "value".into(),
                Some("context 5".into())
            ),
            (
                "context-6".into(),
                "Translation".into(),
                "value".into(),
                Some("context 6".into())
            ),
            (
                "context-7".into(),
                "Citation".into(),
                "value".into(),
                Some("context 7".into())
            ),
        ]
    );

    let farvi_assertions: Vec<(String, Option<String>, Option<String>)> = db
        .prepare("SELECT id,definition_text,source_score_value FROM assertions WHERE entry_id='entry-a-farvi' ORDER BY id")
        .unwrap()
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
        .unwrap()
        .collect::<rusqlite::Result<_>>()
        .unwrap();
    assert_eq!(
        farvi_assertions,
        vec![
            ("assertion-0".into(), Some("$x_1$ (object) develops/evolves towards/into $x_2$ (state) from $x_3$ (state) for reason/because of $x_4$ (fact, event) through stages $x_5$".into()), Some("1.0".into())),
            ("assertion-1".into(), Some("$x_1$ (object) develops/evolves towards/into $x_2$ (state) from $x_3$ (state) for reason/because of $x_4$ (fact, event) through stages $x_5$".into()), Some("1.0".into())),
            ("assertion-2".into(), Some("$x_{1}$ develops/evolves towards/into $x_{2}$ from $x_{3}$ through stages $x_{4}$.".into()), Some("100002.0".into())),
            ("assertion-3".into(), Some("$x_1$ (object) develops/evolves towards/into $x_2$ (state) from $x_3$ (state) for reason/because of $x_4$ (fact, event) through stages $x_5$".into()), Some("1.0".into())),
        ]
    );

    let forms: Vec<FormOracleRow> = db
        .prepare("SELECT id,kind,spelling,standing_state,standing_value,computation_id FROM forms JOIN entity_ids USING(id) ORDER BY entity_ids.ordinal")
        .unwrap().query_map([], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?)))
        .unwrap().collect::<rusqlite::Result<_>>().unwrap();
    assert_eq!(
        forms,
        vec![
            (
                "form-variant".into(),
                "SourceVariant".into(),
                "siʼe".into(),
                "value".into(),
                Some("reported".into()),
                None
            ),
            (
                "form-pronunciation".into(),
                "SourcePronunciation".into(),
                "si'e".into(),
                "missing".into(),
                None,
                None
            ),
            (
                "form-assigned".into(),
                "AssignedRafsi".into(),
                "si'e".into(),
                "value".into(),
                Some("assigned".into()),
                None
            ),
            (
                "form-reported".into(),
                "SourceReportedForm".into(),
                "cpan".into(),
                "value".into(),
                Some("source-reported".into()),
                None
            ),
            (
                "form-stem".into(),
                "ComputedStem".into(),
                "snim".into(),
                "missing".into(),
                None,
                Some("computation-1".into())
            ),
            (
                "form-components".into(),
                "ComputedComponents".into(),
                "cpana".into(),
                "missing".into(),
                None,
                Some("computation-1".into())
            ),
            (
                "form-generated".into(),
                "GeneratedCandidate".into(),
                "snimra".into(),
                "missing".into(),
                None,
                Some("computation-1".into())
            ),
        ]
    );
    let form_components: Vec<(String,i64,String,Option<String>)> = db
        .prepare("SELECT form_id,component_order,spelling,target_entry_id FROM form_components ORDER BY form_id,ordinal")
        .unwrap().query_map([], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?)))
        .unwrap().collect::<rusqlite::Result<_>>().unwrap();
    assert_eq!(
        form_components,
        vec![
            (
                "form-components".into(),
                1,
                "cpana".into(),
                Some("entry-a-cpana".into())
            ),
            (
                "form-generated".into(),
                1,
                "snime".into(),
                Some("entry-a-snime".into())
            ),
        ]
    );

    assert_eq!(
        db.query_row::<(i64, String, String, String, String), _, _>(
            "SELECT slot_order,native_label,description_state,description_text,grammatical_role_value FROM argument_slots WHERE frame_id='frame-si-e'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
        ).unwrap(),
        (1, "x1".into(), "value".into(), "snow".into(), "agent".into())
    );
    assert_eq!(
        db.query_row::<(String, String, String, String, Option<String>), _, _>(
            "SELECT o.namespace,o.locator,e.field_index,e.citation_state,e.citation_value FROM argument_frames f JOIN source_evidence e ON e.evidence_id=f.evidence_id JOIN occurrence_refs o ON o.id=e.occurrence_id WHERE f.id='frame-si-e'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
        ).unwrap(),
        ("captured:a:lexical".into(), "/0".into(), "9".into(), "value".into(), Some("labelled synthetic fixture; pinned witnesses noted in documentation".into()))
    );
    assert_eq!(
        db.query_row::<(String, Option<String>, String), _, _>(
            "SELECT statement_state,statement_text,native_kind_value FROM etymologies WHERE id='etymology-2'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        ).unwrap(),
        ("value".into(), Some("derived source text".into()), "source etymology".into())
    );

    let relation_shapes: Vec<RelationOracleRow> = db
        .prepare("SELECT id,kind,other_kind,direction,scope,target_kind,catalog_source_id,unresolved_text,editorial_revision_id FROM relations JOIN entity_ids USING(id) ORDER BY entity_ids.ordinal")
        .unwrap().query_map([], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?,r.get(6)?,r.get(7)?,r.get(8)?)))
        .unwrap().collect::<rusqlite::Result<_>>().unwrap();
    assert_eq!(
        relation_shapes,
        vec![
            (
                "relation-see".into(),
                "SourceSeeAlso".into(),
                None,
                "Forward".into(),
                "Entry".into(),
                "Unknown".into(),
                None,
                None,
                None
            ),
            (
                "relation-missing-catalogue".into(),
                "SourceEtymology".into(),
                None,
                "Unspecified".into(),
                "Entry".into(),
                "Catalog".into(),
                Some("source-m".into()),
                None,
                None
            ),
            (
                "relation-unresolved".into(),
                "Other".into(),
                Some("fixture".into()),
                "Bidirectional".into(),
                "Entry".into(),
                "Unresolved".into(),
                None,
                Some("unresolved target".into()),
                None
            ),
            (
                "relation-editorial".into(),
                "EditorialLink".into(),
                None,
                "Forward".into(),
                "Editorial".into(),
                "Local".into(),
                None,
                None,
                Some("revision-r2".into())
            ),
        ]
    );

    let revisions: Vec<OwnedRevisionOracleRow> = db
        .prepare("SELECT id,parent_id,timestamp_state,timestamp_value,rationale_state,rationale_text,time_precision,editor_id FROM owned_revisions JOIN entity_ids USING(id) ORDER BY entity_ids.ordinal")
        .unwrap().query_map([], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?,r.get(6)?,r.get(7)?)))
        .unwrap().collect::<rusqlite::Result<_>>().unwrap();
    assert_eq!(
        revisions,
        vec![
            (
                "revision-r1".into(),
                None,
                "value".into(),
                "2026-10-24T00:00:00Z".into(),
                "value".into(),
                Some("initial owned wording".into()),
                "Second".into(),
                "editor-1".into()
            ),
            (
                "revision-r2".into(),
                Some("revision-r1".into()),
                "value".into(),
                "2026-10-24T01:00:00Z".into(),
                "value".into(),
                Some("revised owned wording".into()),
                "Second".into(),
                "editor-1".into()
            ),
        ]
    );
    let editorial_members: Vec<(String,i64,String)> = db
        .prepare("SELECT editorial_sense_id,ordinal,assertion_id FROM editorial_members ORDER BY ordinal")
        .unwrap().query_map([], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?)))
        .unwrap().collect::<rusqlite::Result<_>>().unwrap();
    assert_eq!(
        editorial_members,
        vec![
            ("editorial-farvi".into(), 0, "assertion-0".into()),
            ("editorial-farvi".into(), 1, "assertion-4".into()),
        ]
    );

    let inputs: Vec<(String, Option<String>, Option<String>)> = db
        .prepare("SELECT input_kind,literal,occurrence_id FROM computation_inputs ORDER BY ordinal")
        .unwrap()
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
        .unwrap()
        .collect::<rusqlite::Result<_>>()
        .unwrap();
    assert_eq!(
        inputs
            .iter()
            .map(|(kind, literal, occurrence)| (
                kind.as_str(),
                literal.as_deref(),
                occurrence.is_some()
            ))
            .collect::<Vec<_>>(),
        vec![
            ("Occurrence", None, true),
            ("Literal", Some("explicit input"), false)
        ]
    );
    assert_eq!(db.query_row::<(String,Option<String>,String),_,_>("SELECT profile_state,profile_value,explanation_text FROM computations WHERE id='computation-1'",[],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).unwrap(),("value".into(),Some("fixture-profile".into()),"labelled computation; no automatic analysis".into()));

    let content: (String,Option<String>,String,Option<String>,String,Option<String>) = db
        .query_row("SELECT text_state,text_value,locator_state,locator_value,license_state,license_value FROM content_notices WHERE release_id='release-a'",[],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?))).unwrap();
    assert_eq!(
        content,
        (
            "value".into(),
            Some("fixture notice".into()),
            "value".into(),
            Some("fixture://notice".into()),
            "missing".into(),
            None
        )
    );
    assert_eq!(
        db.query_row::<String, _, _>(
            "SELECT evidence FROM redistribution_evidence WHERE release_id='release-o1'",
            [],
            |r| r.get(0)
        )
        .unwrap(),
        "explicit fixture permission evidence"
    );

    drop(db);
    drop(store);
    let reopened = DictionaryStore::open(&path).unwrap();
    assert_eq!(reopened.dictionary(), &dictionary);
    drop(reopened);
    fs::remove_dir_all(parent).unwrap();
}

#[test]
fn open_rejects_same_count_scalar_edits_in_each_relational_family_without_repair() {
    let base = scratch();
    let dict = fixture_support::complete_fixture().unwrap();
    let source = base.join("source");
    let store = DictionaryStore::create(&source, &dict).unwrap();
    let db = store.database_path();
    drop(store);
    let cases = [
        (
            "registry",
            "UPDATE sources SET display_name=display_name||'x' WHERE id='source-a'",
        ),
        (
            "raw",
            "UPDATE raw_captures SET source_name=source_name||'x' WHERE namespace='captured:a:d1'",
        ),
        (
            "catalogue",
            "UPDATE components SET native_name=native_name||'x' WHERE id='component-a'",
        ),
        (
            "entries",
            "UPDATE entries SET spelling_value=spelling_value||'x' WHERE id='entry-a-si-e'",
        ),
        (
            "contexts",
            "UPDATE contexts SET kind='Note' WHERE id='context-0'",
        ),
        (
            "forms",
            "UPDATE forms SET spelling=spelling||'x' WHERE id='form-assigned'",
        ),
        (
            "etymologies",
            "UPDATE etymologies SET native_kind_value=native_kind_value||'x' WHERE id='etymology-0'",
        ),
        (
            "relations",
            "UPDATE relations SET direction='Reverse' WHERE id='relation-see'",
        ),
        (
            "owned",
            "UPDATE owned_revisions SET timestamp_value=timestamp_value||'x' WHERE id='revision-r1'",
        ),
        (
            "editorial",
            "UPDATE editorial_senses SET rationale_text=rationale_text||'x' WHERE id='editorial-farvi'",
        ),
        (
            "computation",
            "UPDATE computations SET tool=tool||'x' WHERE id='computation-1'",
        ),
        (
            "annotations",
            "UPDATE annotations SET kind=kind||'x' WHERE id='annotation-1'",
        ),
        (
            "extensions",
            "UPDATE extensions SET value=value||' ' WHERE name='unknown-extension'",
        ),
        (
            "notices",
            "UPDATE code_notices SET text=text||'x' WHERE ordinal=0",
        ),
    ];
    for (name, sql) in cases {
        let case = base.join(name);
        fs::create_dir(&case).unwrap();
        let dbpath = case.join("dictionary.sqlite3");
        fs::copy(&db, &dbpath).unwrap();
        let conn = Connection::open(&dbpath).unwrap();
        let changed = conn.execute(sql, []).unwrap();
        assert_eq!(changed, 1, "tamper setup for {name}");
        drop(conn);
        let before = fs::read(&dbpath).unwrap();
        let error = DictionaryStore::open(&case).unwrap_err();
        assert!(
            matches!(
                error,
                DictionaryStoreError::InvalidStore {
                    reason: vlajmaji_store::DictionaryStoreInvalidity::Projection,
                    ..
                }
            ),
            "{name}: {error}"
        );
        assert_eq!(
            fs::read(&dbpath).unwrap(),
            before,
            "open must not repair {name}"
        );
    }
    fs::remove_dir_all(base).unwrap();
}

#[test]
fn empty_valid_dictionary_has_a_real_empty_projection() {
    let parent = scratch();
    let path = parent.join("empty");
    let dictionary = Dictionary::new();
    let store = DictionaryStore::create(&path, &dictionary).unwrap();
    assert_eq!(store.dictionary(), &dictionary);
    drop(store);
    let reopened = DictionaryStore::open(&path).unwrap();
    assert_eq!(reopened.dictionary(), &dictionary);
    drop(reopened);
    fs::remove_dir_all(parent).unwrap();
}

#[test]
fn fresh_path_lifecycle_refuses_existing_leafs_and_missing_parents() {
    let base = scratch();
    let dictionary = Dictionary::new();
    let file = base.join("occupied-file");
    fs::write(&file, b"caller-owned").unwrap();
    assert!(matches!(
        DictionaryStore::create(&file, &dictionary),
        Err(DictionaryStoreError::ExistingPath(_))
    ));
    assert_eq!(fs::read(&file).unwrap(), b"caller-owned");
    assert!(matches!(
        DictionaryStore::open(&file),
        Err(DictionaryStoreError::InvalidStore {
            reason: vlajmaji_store::DictionaryStoreInvalidity::MissingDatabase,
            ..
        })
    ));

    let missing_parent = base.join("absent-parent").join("store");
    assert!(matches!(
        DictionaryStore::create(&missing_parent, &dictionary),
        Err(DictionaryStoreError::Io(_))
    ));
    assert!(!missing_parent.parent().unwrap().exists());
    assert!(matches!(
        DictionaryStore::open(&missing_parent),
        Err(DictionaryStoreError::InvalidStore {
            reason: vlajmaji_store::DictionaryStoreInvalidity::MissingDatabase,
            ..
        })
    ));

    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        let dangling = base.join("dangling");
        symlink(base.join("no-target"), &dangling).unwrap();
        assert!(matches!(
            DictionaryStore::create(&dangling, &dictionary),
            Err(DictionaryStoreError::ExistingPath(_))
        ));
        assert!(
            dangling
                .symlink_metadata()
                .unwrap()
                .file_type()
                .is_symlink()
        );
    }
    fs::remove_dir_all(base).unwrap();
}

#[test]
fn sql_role_presence_foreign_key_and_unique_constraints_are_real() {
    fn fresh_db(base: &std::path::Path, name: &str) -> (PathBuf, PathBuf) {
        let source = base.join(format!("source-{name}"));
        let store = DictionaryStore::create(&source, &fixture_support::complete_fixture().unwrap())
            .unwrap();
        let db = store.database_path();
        drop(store);
        let case = base.join(format!("case-{name}"));
        fs::create_dir(&case).unwrap();
        let copied = case.join("dictionary.sqlite3");
        fs::copy(db, &copied).unwrap();
        (case, copied)
    }
    fn code(result: rusqlite::Result<()>, wanted: i32, label: &str) {
        let error = result.expect_err(label);
        match error {
            rusqlite::Error::SqliteFailure(e, _) => {
                assert_eq!(e.extended_code, wanted, "{label}: {e:?}")
            }
            other => panic!("{label}: expected SQLite constraint code {wanted}, got {other:?}"),
        }
    }
    let base = scratch();

    let (_case, db) = fresh_db(&base, "check");
    let conn = Connection::open(&db).unwrap();
    code(conn.execute("INSERT INTO native_fields(occurrence_id,ordinal,name,encoding,raw_value) VALUES('missing-occurrence',999,'x','NotAnEncoding','x')", []).map(|_| ()), 275, "native encoding CHECK");
    drop(conn);

    let (_case, db) = fresh_db(&base, "unique");
    let conn = Connection::open(&db).unwrap();
    code(
        conn.execute(
            "INSERT INTO metadata(key,value) VALUES('format','duplicate')",
            [],
        )
        .map(|_| ()),
        1555,
        "metadata primary key",
    );
    drop(conn);

    let (_case, db) = fresh_db(&base, "presence");
    let conn = Connection::open(&db).unwrap();
    code(conn.execute("UPDATE entries SET spelling_state='missing',spelling_value='forbidden' WHERE id='entry-a-si-e'", []).map(|_| ()), 275, "Presence value/state CHECK");
    drop(conn);

    let (_case, db) = fresh_db(&base, "foreign-key");
    let conn = Connection::open(&db).unwrap();
    conn.execute_batch("PRAGMA foreign_keys=ON; BEGIN;")
        .unwrap();
    conn.execute("INSERT INTO source_aliases(ordinal,source_id,text,status) VALUES(900,'missing-source','alias','Active')", []).unwrap();
    code(
        conn.execute_batch("COMMIT;"),
        787,
        "deferred source foreign key",
    );
    drop(conn);

    let (_case, db) = fresh_db(&base, "role-positive");
    let conn = Connection::open(&db).unwrap();
    conn.execute_batch("PRAGMA foreign_keys=ON; BEGIN;")
        .unwrap();
    let targets = [
        ("Entry", "target_entry_id", "entry-a-si-e"),
        ("Assertion", "target_assertion_id", "assertion-0"),
        ("Form", "target_form_id", "form-assigned"),
        (
            "EditorialSense",
            "target_editorial_sense_id",
            "editorial-farvi",
        ),
    ];
    for (n, (kind, target_column, target_id)) in targets.iter().enumerate() {
        let columns =
            format!("etymology_id,ordinal,component_order,spelling,target_kind,{target_column}");
        let sql = format!(
            "INSERT INTO etymology_components({columns}) VALUES('etymology-0',{}, {}, 'role witness', ?, ?)",
            900 + n,
            900 + n
        );
        conn.execute(&sql, [kind, target_id])
            .unwrap_or_else(|error| panic!("valid {kind} etymology target rejected: {error}"));
    }
    for (n, (kind, target_column, target_id)) in targets.iter().enumerate() {
        let columns =
            format!("form_id,ordinal,component_order,spelling,target_kind,{target_column}");
        let sql = format!(
            "INSERT INTO form_components({columns}) VALUES('form-components',{}, {}, 'role witness', ?, ?)",
            900 + n,
            900 + n
        );
        conn.execute(&sql, [kind, target_id])
            .unwrap_or_else(|error| panic!("valid {kind} form target rejected: {error}"));
    }
    conn.execute_batch("ROLLBACK;").unwrap();
    drop(conn);

    let (_case, db) = fresh_db(&base, "role-negative");
    let conn = Connection::open(&db).unwrap();
    code(conn.execute("INSERT INTO form_components(form_id,ordinal,component_order,spelling,target_kind,target_entry_id) VALUES('form-assigned',900,900,'bad','Entry','form-assigned')", []).map(|_| ()), 787, "cross-role target foreign key");
    drop(conn);

    fs::remove_dir_all(base).unwrap();
}

#[test]
fn full_unsigned_text_empty_and_nul_values_survive_sql_and_reopen() {
    let original = fixture_support::complete_fixture().unwrap();
    let mut document = serde_json::to_value(original.document()).unwrap();
    document["metadata"]["catalogue_revision"] = serde_json::json!(u64::MAX);
    let releases = document["metadata"]["releases"].as_array_mut().unwrap();
    let missing = releases
        .iter_mut()
        .find(|release| release["availability"] == "Missing")
        .unwrap();
    missing["accounting"]["input_occurrences"] = serde_json::json!(u64::MAX);
    missing["accounting"]["normalized_occurrences"] = serde_json::json!(u64::MAX);
    missing["accounting"]["query_excluded_occurrences"] = serde_json::json!(u64::MAX);
    document["metadata"]["code_notices"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!({
            "text":"λ\u{0}notice", "locator":{"Value":""}, "license":{"Value":""}
        }));
    let dictionary = Dictionary::from_document(serde_json::from_value(document).unwrap()).unwrap();
    let base = scratch();
    let path = base.join("store");
    let store = DictionaryStore::create(&path, &dictionary).unwrap();
    let db = Connection::open(store.database_path()).unwrap();
    assert_eq!(
        db.query_row::<String, _, _>(
            "SELECT value FROM metadata WHERE key='catalogue_revision'",
            [],
            |r| r.get(0)
        )
        .unwrap(),
        u64::MAX.to_string()
    );
    assert_eq!(db.query_row::<(Option<String>,Option<String>,Option<String>),_,_>("SELECT accounting_input_occurrences,accounting_normalized_occurrences,accounting_query_excluded_occurrences FROM releases WHERE availability='Missing'", [], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?))).unwrap(), (Some(u64::MAX.to_string()), Some(u64::MAX.to_string()), Some(u64::MAX.to_string())));
    assert_eq!(
        db.query_row::<String, _, _>(
            "SELECT text FROM code_notices WHERE ordinal=(SELECT max(ordinal) FROM code_notices)",
            [],
            |r| r.get(0)
        )
        .unwrap(),
        "λ\0notice"
    );
    assert_eq!(db.query_row::<(String,String,String,String),_,_>("SELECT locator_state,locator_value,license_state,license_value FROM code_notices WHERE ordinal=(SELECT max(ordinal) FROM code_notices)", [], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).unwrap(), ("value".into(), "".into(), "value".into(), "".into()));
    drop(db);
    drop(store);
    let reopened = DictionaryStore::open(&path).unwrap();
    assert_eq!(reopened.dictionary(), &dictionary);
    drop(reopened);
    fs::remove_dir_all(base).unwrap();
}

#[test]
fn legacy_archive_rebuild_is_explicit_separate_and_byte_preserving() {
    use vlajmaji_lexicon::SourceRegistry;
    use vlajmaji_store::SqliteStore;

    let full = fixture_support::complete_fixture().unwrap();
    let full_document = full.document();
    let archive = full_document.legacy_archive.clone().unwrap();
    let base = scratch();
    let old_path = base.join("old-schema1");
    let old = SqliteStore::create(&old_path, &archive).unwrap();
    let old_archive = old.archive_bytes().unwrap();
    assert_eq!(old_archive, archive);
    drop(old);
    let old_reopened = SqliteStore::open(&old_path).unwrap();
    assert_eq!(old_reopened.archive_bytes().unwrap(), archive);
    drop(old_reopened);

    let mut registry_document = full.registry().document();
    registry_document
        .snapshots
        .retain(|binding| binding.snapshot.namespace() != "captured:a:opaque");
    let registry = SourceRegistry::from_document(registry_document).unwrap();
    let mut metadata = full.metadata().clone();
    metadata
        .entries
        .retain(|entry| entry.id.as_str() != "entry-opaque");
    let opaque_assertions = metadata
        .assertions
        .iter()
        .filter(|assertion| assertion.occurrence.capture.namespace == "captured:a:opaque")
        .map(|assertion| assertion.id.clone())
        .collect::<std::collections::BTreeSet<_>>();
    metadata
        .assertions
        .retain(|assertion| assertion.occurrence.capture.namespace != "captured:a:opaque");
    let release = metadata
        .releases
        .iter_mut()
        .find(|release| release.id.as_str() == "release-a")
        .unwrap();
    release
        .captures
        .retain(|capture| capture.namespace != "captured:a:opaque");
    release
        .assertions
        .retain(|id| !opaque_assertions.contains(id));
    release.accounting.input_occurrences = Some(16);
    release.accounting.normalized_occurrences = Some(16);
    release.accounting.query_excluded_occurrences = Some(0);
    release.diagnostics.clear();
    let rebuilt =
        Dictionary::from_legacy_archive(&archive, registry.clone(), metadata.clone()).unwrap();
    assert_eq!(
        rebuilt
            .captures()
            .iter()
            .map(|capture| capture.occurrences.len())
            .sum::<usize>(),
        19
    );
    assert_eq!(rebuilt.metadata().entries.len(), 13);
    assert_eq!(rebuilt.metadata().assertions.len(), 19);
    assert_eq!(
        rebuilt.document().legacy_archive.as_deref(),
        Some(archive.as_slice())
    );

    let new_path = base.join("new-dictionary");
    let store = DictionaryStore::create(&new_path, &rebuilt).unwrap();
    assert_ne!(store.path(), old_path.as_path());
    assert_eq!(store.dictionary(), &rebuilt);
    drop(store);
    let reopened = DictionaryStore::open(&new_path).unwrap();
    assert_eq!(reopened.dictionary(), &rebuilt);
    assert_eq!(
        reopened.dictionary().document().legacy_archive.as_deref(),
        Some(archive.as_slice())
    );
    assert_eq!(
        SqliteStore::open(&old_path)
            .unwrap()
            .archive_bytes()
            .unwrap(),
        archive
    );

    let mut legacy_v1_value: serde_json::Value = serde_json::from_slice(&archive).unwrap();
    legacy_v1_value["version"] = serde_json::json!(1);
    let legacy_v1 = serde_json::to_vec(&legacy_v1_value).unwrap();
    let old_v1_path = base.join("old-schema1-v1");
    let old_v1 = SqliteStore::create(&old_v1_path, &legacy_v1).unwrap();
    assert_eq!(old_v1.archive_bytes().unwrap(), legacy_v1);
    drop(old_v1);
    let rebuilt_v1 = Dictionary::from_legacy_archive(&legacy_v1, registry, metadata).unwrap();
    let new_v1_path = base.join("new-dictionary-v1");
    let store_v1 = DictionaryStore::create(&new_v1_path, &rebuilt_v1).unwrap();
    assert_eq!(store_v1.dictionary(), &rebuilt_v1);
    drop(store_v1);
    assert_eq!(
        DictionaryStore::open(&new_v1_path).unwrap().dictionary(),
        &rebuilt_v1
    );
    fs::remove_dir_all(base).unwrap();
}

#[test]
fn integrity_failures_are_classified_without_mutating_the_database() {
    use sha2::{Digest, Sha256};
    use vlajmaji_store::DictionaryStoreInvalidity as Invalidity;

    let base = scratch();
    let fixture = fixture_support::complete_fixture().unwrap();
    let cases: [(&str, &str, Option<Invalidity>); 9] = [
        (
            "family",
            "UPDATE metadata SET value='other-family' WHERE key='format'",
            Some(Invalidity::Format),
        ),
        (
            "version",
            "UPDATE metadata SET value='99' WHERE key='schema_version'",
            Some(Invalidity::SchemaVersion),
        ),
        (
            "incomplete",
            "UPDATE metadata SET value='0' WHERE key='complete'",
            Some(Invalidity::Incomplete),
        ),
        (
            "bad-hash",
            "UPDATE metadata SET value=printf('%064d',0) WHERE key='canonical_sha256'",
            Some(Invalidity::CanonicalBytes),
        ),
        (
            "extra-schema",
            "CREATE TABLE unexpected_extra(value TEXT)",
            Some(Invalidity::SchemaShape),
        ),
        (
            "foreign-key",
            "PRAGMA foreign_keys=OFF; INSERT INTO source_aliases(ordinal,source_id,text,status) VALUES(900,'absent-source','alias','Active')",
            Some(Invalidity::ForeignKeys),
        ),
        (
            "extra-row",
            "INSERT INTO source_aliases(ordinal,source_id,text,status) VALUES(900,'source-a','new alias','Active')",
            Some(Invalidity::Projection),
        ),
        (
            "deleted-row",
            "DELETE FROM extensions WHERE name='unknown-extension'",
            Some(Invalidity::Projection),
        ),
        (
            "malformed-canonical",
            "UPDATE canonical_document SET json=x'7b' WHERE id=1",
            None,
        ),
    ];
    for (name, setup, wanted) in cases {
        let original = base.join(format!("origin-{name}"));
        let store = DictionaryStore::create(&original, &fixture).unwrap();
        let source_db = store.database_path();
        drop(store);
        let path = base.join(format!("case-{name}"));
        fs::create_dir(&path).unwrap();
        let dbpath = path.join("dictionary.sqlite3");
        fs::copy(source_db, &dbpath).unwrap();
        let conn = Connection::open(&dbpath).unwrap();
        if name == "malformed-canonical" {
            let bad = b"{";
            let digest = format!("{:x}", Sha256::digest(bad));
            conn.execute("UPDATE canonical_document SET json=?1 WHERE id=1", [bad])
                .unwrap();
            conn.execute(
                "UPDATE metadata SET value=?1 WHERE key='canonical_sha256'",
                [&digest],
            )
            .unwrap();
        } else {
            conn.execute_batch(setup)
                .unwrap_or_else(|error| panic!("tamper setup {name} failed: {error}"));
        }
        drop(conn);
        let before = fs::read(&dbpath).unwrap();
        let error = DictionaryStore::open(&path).unwrap_err();
        if let Some(reason) = wanted {
            assert!(
                matches!(error, DictionaryStoreError::InvalidStore { reason: actual, .. } if actual == reason),
                "{name}: {error}"
            );
        } else {
            assert!(
                matches!(error, DictionaryStoreError::InvalidDictionary(_)),
                "{name}: {error}"
            );
        }
        assert_eq!(
            fs::read(&dbpath).unwrap(),
            before,
            "open must not repair {name}"
        );
    }

    let original = base.join("origin-stale-projection");
    let store = DictionaryStore::create(&original, &fixture).unwrap();
    let source_db = store.database_path();
    drop(store);
    let path = base.join("case-stale-projection");
    fs::create_dir(&path).unwrap();
    let dbpath = path.join("dictionary.sqlite3");
    fs::copy(source_db, &dbpath).unwrap();
    let empty = Dictionary::new().to_json_bytes().unwrap();
    let digest = format!("{:x}", Sha256::digest(&empty));
    let conn = Connection::open(&dbpath).unwrap();
    conn.execute(
        "UPDATE canonical_document SET json=?1,legacy_archive=NULL WHERE id=1",
        [&empty],
    )
    .unwrap();
    conn.execute(
        "UPDATE metadata SET value=?1 WHERE key='canonical_sha256'",
        [&digest],
    )
    .unwrap();
    drop(conn);
    let before = fs::read(&dbpath).unwrap();
    let error = DictionaryStore::open(&path).unwrap_err();
    assert!(
        matches!(
            error,
            DictionaryStoreError::InvalidStore {
                reason: Invalidity::Projection,
                ..
            }
        ),
        "{error}"
    );
    assert_eq!(fs::read(&dbpath).unwrap(), before);

    fs::remove_dir_all(base).unwrap();
}

#[test]
fn empty_available_release_keeps_explicit_zero_counts() {
    let mut document =
        serde_json::to_value(fixture_support::complete_fixture().unwrap().document()).unwrap();
    let releases = document["metadata"]["releases"].as_array_mut().unwrap();
    let release = releases
        .iter_mut()
        .find(|release| release["id"] == "release-b")
        .unwrap();
    release["captures"] = serde_json::json!([]);
    release["assertions"] = serde_json::json!([]);
    release["accounting"] = serde_json::json!({
        "input_occurrences": 0, "normalized_occurrences": 0,
        "query_excluded_occurrences": 0, "unit": "occurrence"
    });
    release["diagnostics"] = serde_json::json!([]);
    let dictionary = Dictionary::from_document(serde_json::from_value(document).unwrap()).unwrap();
    let base = scratch();
    let path = base.join("available-empty");
    let store = DictionaryStore::create(&path, &dictionary).unwrap();
    let db = Connection::open(store.database_path()).unwrap();
    assert_eq!(db.query_row::<(String,String,String),_,_>("SELECT availability,accounting_input_occurrences,accounting_normalized_occurrences FROM releases WHERE id='release-b'", [], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?))).unwrap(), ("Available".into(), "0".into(), "0".into()));
    drop(db);
    drop(store);
    let reopened = DictionaryStore::open(&path).unwrap();
    assert_eq!(reopened.dictionary(), &dictionary);
    drop(reopened);
    fs::remove_dir_all(base).unwrap();
}

#[test]
fn empty_source_metadata_values_survive_actual_sql_projection() {
    use vlajmaji_lexicon::{
        AbbreviationKind, AssertionRecord, DictionaryMetadata, Eligibility, EntityId, EntryRecord,
        Lexicon, OccurrenceRef, Presence, RegistryDocument, SnapshotAttribution, SourceFormat,
        SourceId, SourceMetadata, SourceOrigin, SourceRecord, SourceRegistry, TextValue,
        TimePrecision,
    };
    let mut lexicon = Lexicon::new();
    lexicon
        .import_source(
            SourceMetadata::new(
                "",
                "",
                SourceFormat::LaLojbanJson,
                "",
                "",
                Some(String::new()),
                TimePrecision::Unknown,
            )
            .with_notice_locator(""),
            br#"[{"word":"x","definition":"d"}]"#,
            None,
        )
        .unwrap();
    let source = SourceId::new("source-empty").unwrap();
    let mut registry_doc = RegistryDocument::default();
    registry_doc.sources.push(SourceRecord {
        id: source.clone(),
        display_name: "Empty metadata source".into(),
        abbreviation: "empty".into(),
        abbreviation_kind: AbbreviationKind::ProjectAssigned,
        abbreviation_evidence: None,
        origin: SourceOrigin::Upstream,
    });
    registry_doc.snapshots.push(SnapshotAttribution {
        snapshot: lexicon.snapshots()[0].clone(),
        source,
        distributor: None,
    });
    let registry = SourceRegistry::from_document(registry_doc).unwrap();
    let archive = lexicon.to_archive_bytes().unwrap();
    let mut metadata = DictionaryMetadata::default();
    let entry_id = EntityId::new("entry-empty-metadata").unwrap();
    metadata.entries.push(EntryRecord {
        id: entry_id.clone(),
        source: SourceId::new("source-empty").unwrap(),
        spelling: Presence::Value("x".into()),
        language: Presence::Missing,
        script: Presence::Missing,
        native_word_class: Presence::Missing,
        selmaho: Presence::Missing,
        mapped_pos: Presence::Missing,
    });
    let record = &lexicon.records()[0];
    metadata.assertions.push(AssertionRecord {
        id: EntityId::new("assertion-empty-metadata").unwrap(),
        entry: entry_id,
        occurrence: OccurrenceRef::from(record.identity()),
        native_sense_id: Presence::Missing,
        language: Presence::Missing,
        definition: Presence::Value(TextValue {
            text: "d".into(),
            language: Presence::Missing,
            markup: Presence::Missing,
        }),
        source_score: Presence::Missing,
        timestamps: Vec::new(),
        eligibility: Eligibility::Lexical,
    });
    let dictionary = Dictionary::from_legacy_archive(&archive, registry, metadata).unwrap();
    let base = scratch();
    let path = base.join("empty-source-metadata");
    let store = DictionaryStore::create(&path, &dictionary).unwrap();
    let db = Connection::open(store.database_path()).unwrap();
    assert_eq!(db.query_row::<(String,String,String,String,String,String,String),_,_>("SELECT source_name,locator,revision_state,revision_value,captured_at_state,captured_at_value,notice_locator_value FROM raw_captures", [], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?,r.get(6)?))).unwrap(), ("".into(),"".into(),"value".into(),"".into(),"value".into(),"".into(),"".into()));
    drop(db);
    drop(store);
    let reopened = DictionaryStore::open(&path).unwrap();
    assert_eq!(reopened.dictionary(), &dictionary);
    drop(reopened);
    fs::remove_dir_all(base).unwrap();
}
