//! Public exact-query parity for the three explicit read topologies.
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};
use vlajmaji_lexicon::{Dictionary, JbotciImportOptions, import_jbotci_export};
use vlajmaji_search::{DictionaryExactOptions, DictionarySearchEngine};
use vlajmaji_store::{
    DictionaryReadTopology, DictionarySnapshotManifest, DictionarySnapshotMember, DictionaryStore,
};

fn scratch() -> PathBuf {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!(
        "vlajmaji-search-topology-{}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&p).unwrap();
    p
}
fn sha(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn fixture() -> Dictionary {
    let raw=br#"[{"word":"farvi","word_type":"gismu","definition":"to grow"},{"word":"farvi","word_type":"gismu","definition":"to develop"}]"#;
    let metadata=format!("language_tag = \"en\"\nlanguage_realname = \"English\"\nsource_language_tag = \"jbo\"\nformat = \"json\"\npositive_scores_only = false\nfilename = \"dictionary-en.json\"\nmetadata_url = \"https://example.invalid/languages\"\ndownload_url = \"https://example.invalid/export\"\nlensisku_created_at = \"2026-10-04T03:07:08Z\"\nsha256 = \"{}\"\ndefinition_count = 2\nentry_count = 2\n",sha(raw)).into_bytes();
    import_jbotci_export(
        raw,
        &metadata,
        &JbotciImportOptions {
            revision: "0000000000000000000000000000000000000000".into(),
            expected_raw_sha256: sha(raw),
            expected_metadata_sha256: sha(&metadata),
        },
    )
    .unwrap()
    .into_dictionary()
}

#[test]
fn test_exact_profiled_matches_unprofiled_for_each_topology() {
    let root = scratch();
    let dictionary = fixture();
    let bytes = dictionary.to_json_bytes().unwrap();
    let digest = sha(&bytes);
    let mut members = Vec::new();
    for id in ["one", "two"] {
        let path = root.join(id);
        DictionaryStore::create(&path, &dictionary).unwrap();
        members.push(DictionarySnapshotMember {
            id: id.into(),
            path,
            expected_state_sha256: digest.clone(),
        });
    }
    let single = root.join("single");
    DictionaryStore::create(&single, &dictionary).unwrap();
    let manifest = |path: PathBuf| DictionarySnapshotManifest {
        version: 1,
        generation: "generation-a".into(),
        logical_state_sha256: digest.clone(),
        members: vec![DictionarySnapshotMember {
            id: "unified".into(),
            path,
            expected_state_sha256: digest.clone(),
        }],
        selected_release_ids: None,
    };
    let unified =
        DictionarySearchEngine::open_snapshot(DictionaryReadTopology::Unified, manifest(single))
            .unwrap();
    let fanout = DictionarySearchEngine::open_snapshot(
        DictionaryReadTopology::Fanout,
        DictionarySnapshotManifest {
            version: 1,
            generation: "generation-a".into(),
            logical_state_sha256: digest.clone(),
            members: members.clone(),
            selected_release_ids: None,
        },
    )
    .unwrap();
    let (attached, open_profile) = DictionarySearchEngine::open_snapshot_profiled(
        DictionaryReadTopology::Attached,
        DictionarySnapshotManifest {
            version: 1,
            generation: "generation-a".into(),
            logical_state_sha256: digest.clone(),
            members,
            selected_release_ids: None,
        },
    )
    .unwrap();
    assert!(open_profile.total >= open_profile.document_load_and_composition);
    let first_options = DictionaryExactOptions {
        page_size: Some(1),
        ..Default::default()
    };
    let baseline = unified.exact("farvi", first_options.clone()).unwrap();
    assert_eq!(baseline.totals.occurrences, 2);
    let cursor = baseline.next_cursor.clone().unwrap();
    for engine in [&fanout, &attached] {
        assert_eq!(
            engine.exact("farvi", first_options.clone()).unwrap(),
            baseline
        );
        let next = DictionaryExactOptions {
            page_size: Some(1),
            cursor: Some(cursor.clone()),
            ..Default::default()
        };
        let next_response = engine.exact("farvi", next.clone()).unwrap();
        assert_eq!(next_response.totals, baseline.totals);
        let (profiled, profile) = engine.exact_profiled("farvi", next).unwrap();
        assert_eq!(profiled, next_response);
        assert!(profile.total >= profile.candidate_sql);
    }
    let changed = DictionarySearchEngine::open_snapshot(
        DictionaryReadTopology::Unified,
        DictionarySnapshotManifest {
            version: 1,
            generation: "generation-b".into(),
            logical_state_sha256: digest.clone(),
            members: vec![DictionarySnapshotMember {
                id: "unified".into(),
                path: root.join("single"),
                expected_state_sha256: digest.clone(),
            }],
            selected_release_ids: None,
        },
    )
    .unwrap();
    let stale = DictionaryExactOptions {
        page_size: Some(1),
        cursor: Some(cursor),
        ..Default::default()
    };
    assert!(matches!(
        changed.exact("farvi", stale),
        Err(vlajmaji_search::DictionaryQueryError::StaleCursor)
    ));
    fs::remove_dir_all(root).unwrap();
}
