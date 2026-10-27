//! Public read-only topology checks on small owned stores.
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};
use vlajmaji_store::{
    DictionaryReadSnapshot, DictionaryReadTopology, DictionarySnapshotManifest,
    DictionarySnapshotMember, DictionaryStore,
};

fn scratch() -> PathBuf {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!(
        "vlajmaji-topology-{}-{}-{}",
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

#[test]
fn test_snapshot_topologies_open_read_only_and_deduplicate_same_logical_ids() {
    let root = scratch();
    let dictionary = vlajmaji_lexicon::Dictionary::new();
    let bytes = dictionary.to_json_bytes().unwrap();
    let digest = sha(&bytes);
    let mut members = Vec::new();
    for id in ["one", "two"] {
        let path = root.join(format!("pack-{id} #?%"));
        DictionaryStore::create(&path, &dictionary).unwrap();
        members.push(DictionarySnapshotMember {
            id: id.into(),
            path,
            expected_state_sha256: digest.clone(),
        });
    }
    let single = root.join("single");
    DictionaryStore::create(&single, &dictionary).unwrap();
    let expected_ids = DictionaryStore::open(&single)
        .unwrap()
        .exact_assertion_ids("farvi")
        .unwrap();
    for topology in [
        DictionaryReadTopology::Attached,
        DictionaryReadTopology::Fanout,
    ] {
        let mut snapshot = DictionaryReadSnapshot::open(
            topology,
            DictionarySnapshotManifest {
                version: 1,
                generation: "g1".into(),
                logical_state_sha256: digest.clone(),
                members: members.clone(),
                selected_release_ids: None,
            },
        )
        .unwrap();
        snapshot.load_document().unwrap();
        assert_eq!(snapshot.logical_state_sha256(), digest);
        assert!(snapshot.runtime_attached_limit() >= 1);
        let ids = snapshot.exact_assertion_ids("farvi").unwrap();
        assert_eq!(ids, expected_ids);
    }
    let unified = DictionaryReadSnapshot::open(
        DictionaryReadTopology::Unified,
        DictionarySnapshotManifest {
            version: 1,
            generation: "g1".into(),
            logical_state_sha256: digest.clone(),
            members: vec![DictionarySnapshotMember {
                id: "unified".into(),
                path: single,
                expected_state_sha256: digest,
            }],
            selected_release_ids: None,
        },
    )
    .unwrap();
    assert_eq!(unified.exact_assertion_ids("farvi").unwrap(), expected_ids);
    fs::remove_dir_all(root).unwrap();
}

#[cfg(unix)]
#[test]
fn test_attached_rejects_non_utf8_native_path_without_lossy_routing_or_creation() {
    use std::os::unix::ffi::OsStringExt;
    let root = scratch();
    let path = root.join(std::ffi::OsString::from_vec(b"bad-\xff".to_vec()));
    let manifest = DictionarySnapshotManifest {
        version: 1,
        generation: "g1".into(),
        logical_state_sha256: "0".repeat(64),
        members: vec![DictionarySnapshotMember {
            id: "bad".into(),
            path: path.clone(),
            expected_state_sha256: "0".repeat(64),
        }],
        selected_release_ids: None,
    };
    assert!(matches!(
        DictionaryReadSnapshot::open(DictionaryReadTopology::Attached, manifest.clone()),
        Err(vlajmaji_store::DictionarySnapshotError::UnsupportedAttachmentPath(_))
    ));
    assert!(!path.exists());
    assert!(matches!(
        DictionaryReadSnapshot::open(DictionaryReadTopology::Unified, manifest),
        Err(vlajmaji_store::DictionarySnapshotError::Store(_))
    ));
    assert!(!path.exists());
    fs::remove_dir_all(root).unwrap();
}
