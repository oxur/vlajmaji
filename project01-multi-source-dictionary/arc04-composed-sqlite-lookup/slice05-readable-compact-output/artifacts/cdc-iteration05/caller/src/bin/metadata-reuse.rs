// SOURCE PREPARATION ONLY during retained CC six-path ownership.
// Compile/run only after clean frozen source/index/target handback.
include!("../helpers.rs");
include!("../owned_paths.rs");
include!("../profile_parity_checks.rs");
use rusqlite::{Connection, params};
use std::{borrow::Cow, path::Path};
use vlajmaji_search::*;
use vlajmaji_store::{
    CollectionCatalogue, CollectionInput, CollectionReferenceMetadataCache, CollectionRequest,
    CollectionSelection,
};
const PACKS: [PackId; 4] = [
    PackId::LensiskuJbotci,
    PackId::Llg,
    PackId::LaLojban,
    PackId::Sutysisku,
];
fn key(d: &UnverifiedDictionaryDocument, a: &AssertionRecord) -> DictionaryHitDetailRef {
    let e = d.metadata.entries.iter().find(|e| e.id == a.entry).unwrap();
    let snap = d
        .registry
        .snapshots
        .iter()
        .find(|s| {
            s.snapshot.namespace() == a.occurrence.capture.namespace
                && s.snapshot.sha256() == a.occurrence.capture.sha256
        })
        .unwrap();
    let native = d
        .registry
        .occurrences
        .iter()
        .find(|r| OccurrenceRef::from(&r.occurrence) == a.occurrence);
    DictionaryHitDetailRef {
        version: 1,
        digest_policy: "entry-assertion-serde-json-v1".into(),
        role: "assertion".into(),
        source: e.source.clone(),
        entry_id: e.id.clone(),
        assertion_id: a.id.clone(),
        occurrence: a.occurrence.clone(),
        entry_sha256: sha(&serde_json::to_vec(e).unwrap()),
        assertion_sha256: sha(&serde_json::to_vec(a).unwrap()),
        native_entry_id: native.and_then(|r| r.native_entry_id.clone()),
        native_assertion_id: native.and_then(|r| r.native_assertion_id.clone()),
        contributor_id: native.and_then(|r| r.contributor.clone()),
        distributor_id: snap.distributor.clone(),
    }
}
fn query(size: Option<usize>, cursor: Option<String>) -> DictionaryRequest {
    DictionaryRequest::Exact {
        spelling: "farvi".into(),
        options: DictionaryExactOptions {
            page_size: size,
            cursor,
            ..Default::default()
        },
    }
}
fn value<T: serde::Serialize>(x: &T) -> serde_json::Value {
    serde_json::to_value(x).unwrap()
}
fn write_descriptor(db: &Path, digest: &str, bytes: &[u8]) {
    let c = Connection::open(db).unwrap();
    assert_eq!(c.execute("UPDATE registrations_v2 SET descriptor_json=?1 WHERE pack_id='llg' AND release_sha256=?2",params![bytes,digest]).unwrap(),1);
}
fn write_kind(db: &Path, digest: &str, kind: &str) {
    let c = Connection::open(db).unwrap();
    assert_eq!(
        c.execute(
            "UPDATE registrations_v2 SET payload_kind=?1 WHERE pack_id='llg' AND release_sha256=?2",
            params![kind, digest]
        )
        .unwrap(),
        1
    );
}
fn db_leaf(path: &Path) {
    let m = std::fs::symlink_metadata(path).unwrap();
    assert!(
        m.file_type().is_file() && !m.file_type().is_symlink(),
        "strict owned SQLite leaf"
    );
}
fn unsupported(error: vlajmaji_store::CollectionError) -> String {
    match error {
        vlajmaji_store::CollectionError::UnsupportedVersion(detail) => detail,
        e => panic!("wrong typed error: {e:?}"),
    }
}
fn main() {
    fn assert_send<T: Send>() {}
    assert_send::<CollectionSearchEngine>();
    let base = std::path::PathBuf::from(
        std::env::args_os()
            .nth(1)
            .expect("exclusive owned absolute base"),
    );
    assert!(base.is_absolute());
    std::fs::create_dir(&base).unwrap();
    let root = base.join("registered");
    let compact_store = base.join("llg-constructed");
    let input = models();
    // All target/body/native expectations are fixed from individual inputs BEFORE registration.
    let llg = draft(input[1].clone());
    let assertion = llg
        .metadata
        .assertions
        .iter()
        .find(|a| matches!(a.eligibility, Eligibility::Lexical))
        .unwrap();
    let entry = llg
        .metadata
        .entries
        .iter()
        .find(|e| e.id == assertion.entry)
        .unwrap();
    let release = llg
        .metadata
        .releases
        .iter()
        .find(|r| r.assertions.contains(&assertion.id))
        .unwrap();
    let native = llg
        .registry
        .occurrences
        .iter()
        .find(|r| OccurrenceRef::from(&r.occurrence) == assertion.occurrence)
        .unwrap();
    let target = RelationTarget::Catalog(CatalogTarget {
        source: entry.source.clone(),
        release: release.id.clone(),
        native_entry_id: native.native_entry_id.clone(),
        native_assertion_id: native.native_assertion_id.clone(),
        occurrence: Some(assertion.occurrence.clone()),
    });
    assert!(
        native.native_entry_id.is_some() || native.native_assertion_id.is_some(),
        "positive must exercise supplied native predicate"
    );
    let local = RelationTarget::Local(LocalRef::Assertion(assertion.id.clone()));
    let packs: Vec<_> = PACKS
        .iter()
        .zip(&input)
        .map(|(p, d)| (*p, seal_release(*p, PackPayloadKind::Synthetic, d).unwrap()))
        .collect();
    for (p, s) in &packs {
        assert_eq!(
            load_sealed_release(*p, &s.release, &s.seal).unwrap(),
            input[PACKS.iter().position(|x| x == p).unwrap()]
        );
    }
    let request = CollectionRequest::new(
        packs
            .iter()
            .map(|(p, s)| CollectionInput::SealedBytes {
                pack_id: *p,
                release: Cow::Borrowed(&s.release),
                seal: Cow::Borrowed(&s.seal),
            })
            .collect(),
        CollectionSelection::Packs(PACKS.to_vec()),
        true,
    )
    .unwrap();
    CollectionCatalogue::register(&root, request).unwrap();
    let db = root.join("catalogue.sqlite3");
    db_leaf(&db);
    let (digest, kind, original): (String, String, Vec<u8>) = {
        let c = Connection::open(&db).unwrap();
        c.query_row("SELECT release_sha256,payload_kind,descriptor_json FROM registrations_v2 WHERE pack_id='llg'",[],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).unwrap()
    };
    let original_revision = CollectionCatalogue::open(&root)
        .unwrap()
        .local_revision()
        .unwrap();
    drop(vlajmaji_store::DictionaryStore::create(&compact_store, &input[1]).unwrap());
    db_leaf(&compact_store.join("dictionary.sqlite3"));
    {
        let compact = vlajmaji_search::DictionaryCompactSearchEngine::open(&compact_store).unwrap();
        let actual = compact.resolve(&target).unwrap();
        assert_eq!(actual.target, target);
        assert_eq!(actual.status, ReferenceStatus::Resolved);
        assert_eq!(actual.confirmed.len(), 1);
        assert!(actual.unverified.is_empty());
        assert_eq!(actual.confirmed[0].entry, *entry);
        assert_eq!(actual.confirmed[0].assertion, *assertion);
        assert_eq!(
            actual.confirmed[0].occurrence_attribution.as_ref(),
            Some(native)
        );
        let (profiled, p) = compact
            .execute_profiled(DictionaryRequest::Resolve {
                target: target.clone(),
            })
            .unwrap();
        let vlajmaji_search::DictionaryCompactQueryResult::Resolve(profiled) = profiled else {
            panic!("compact resolve")
        };
        assert_eq!(*profiled, actual);
        assert_eq!(p.lookup + p.projection, p.total);
    }
    println!(
        "PASS constructed Compact Catalog UNIQUE joins: pre-input exact body/native/Resolved and profiled whole parity"
    );

    let mut cache = CollectionReferenceMetadataCache::default();
    {
        let cat = CollectionCatalogue::open(&root).unwrap();
        let expected = cat.reference_metadata(PackId::Llg, &digest).unwrap();
        for _ in 0..2 {
            let got = cat
                .reference_metadata_reusing(PackId::Llg, &digest, &mut cache)
                .unwrap();
            assert_eq!(value(got.as_ref()), value(&expected));
        }
    }
    let mut engine = CollectionSearchEngine::open(
        &root,
        CollectionQuerySelection {
            packs: PACKS.to_vec(),
            disabled: vec![],
            profile: None,
        },
    )
    .unwrap();
    let baseline = engine
        .execute(DictionaryRequest::Resolve {
            target: target.clone(),
        })
        .unwrap();
    let CollectionQueryResult::Resolve(ref result) = baseline.result else {
        panic!("resolve result")
    };
    assert_eq!(result.status, ReferenceStatus::CatalogueOnly);
    assert_eq!(result.catalogue_matches.len(), 1);
    assert_eq!(
        result.catalogue_matches[0].native_entry_id,
        native.native_entry_id
    );
    let local_before = engine
        .execute(DictionaryRequest::Resolve {
            target: local.clone(),
        })
        .unwrap();
    let CollectionQueryResult::Resolve(ref result) = local_before.result else {
        panic!("local result")
    };
    assert_eq!(result.catalogue_matches.len(), 1);
    assert_eq!(result.catalogue_matches[0].native_entry_id, None);
    assert_eq!(result.catalogue_matches[0].native_assertion_id, None);
    println!(
        "PASS pre-input native Catalog and legacy Local nativeNone; cache/uncached WHOLE metadata parity"
    );
    // A valid same-key replacement must return NEW complete metadata, never stale cached success.
    let mut valid_change: serde_json::Value = serde_json::from_slice(&original).unwrap();
    let next = valid_change["upstream_catalogue_revision"]
        .as_u64()
        .unwrap()
        .checked_add(1)
        .unwrap();
    valid_change["upstream_catalogue_revision"] = serde_json::json!(next);
    write_descriptor(&db, &digest, &serde_json::to_vec(&valid_change).unwrap());
    {
        let cat = CollectionCatalogue::open(&root).unwrap();
        let uncached = cat.reference_metadata(PackId::Llg, &digest).unwrap();
        let got = cat
            .reference_metadata_reusing(PackId::Llg, &digest, &mut cache)
            .unwrap();
        assert_eq!(value(got.as_ref()), valid_change);
        assert_eq!(value(got.as_ref()), value(&uncached));
    }
    write_descriptor(&db, &digest, &original);
    {
        let cat = CollectionCatalogue::open(&root).unwrap();
        let got = cat
            .reference_metadata_reusing(PackId::Llg, &digest, &mut cache)
            .unwrap();
        assert_eq!(
            value(got.as_ref()),
            serde_json::from_slice::<serde_json::Value>(&original).unwrap()
        );
    }
    println!("PASS same-key VALID metadata replacement and exact restored cache/uncached parity");
    // Retain SAME cache and SAME engine; each mutation helper closes its writer.
    // Open/drop fresh Catalogue snapshots around cache calls; never imply pinned reader sees external writes.
    let mut malformed: serde_json::Value = serde_json::from_slice(&original).unwrap();
    malformed["version"] = serde_json::json!(999);
    write_descriptor(&db, &digest, &serde_json::to_vec(&malformed).unwrap());
    let corrupt_db = sha(&std::fs::read(&db).unwrap());
    {
        let cat = CollectionCatalogue::open(&root).unwrap();
        let old = unsupported(cat.reference_metadata(PackId::Llg, &digest).unwrap_err());
        let reused = unsupported(
            cat.reference_metadata_reusing(PackId::Llg, &digest, &mut cache)
                .unwrap_err(),
        );
        assert_eq!(reused, old, "uncached/cache typed error payload parity");
    }
    assert!(matches!(
        engine.execute(DictionaryRequest::Resolve {
            target: target.clone()
        }),
        Err(vlajmaji_search::CollectionQueryError::Catalogue(
            vlajmaji_store::CollectionError::UnsupportedVersion(_)
        ))
    ));
    assert!(matches!(
        engine.execute(DictionaryRequest::Resolve {
            target: local.clone()
        }),
        Err(vlajmaji_search::CollectionQueryError::Catalogue(
            vlajmaji_store::CollectionError::UnsupportedVersion(_)
        ))
    ));
    assert_eq!(
        sha(&std::fs::read(&db).unwrap()),
        corrupt_db,
        "read-only corruption failures do not write"
    );
    write_descriptor(&db, &digest, &original);
    {
        let cat = CollectionCatalogue::open(&root).unwrap();
        let got = cat
            .reference_metadata_reusing(PackId::Llg, &digest, &mut cache)
            .unwrap();
        assert_eq!(
            value(got.as_ref()),
            serde_json::from_slice::<serde_json::Value>(&original).unwrap()
        );
    }
    assert_eq!(
        value(
            &engine
                .execute(DictionaryRequest::Resolve {
                    target: target.clone()
                })
                .unwrap()
        ),
        value(&baseline)
    );
    assert_eq!(
        value(
            &engine
                .execute(DictionaryRequest::Resolve {
                    target: local.clone()
                })
                .unwrap()
        ),
        value(&local_before)
    );
    write_kind(&db, &digest, "legacy_archive");
    {
        let cat = CollectionCatalogue::open(&root).unwrap();
        let old = unsupported(cat.reference_metadata(PackId::Llg, &digest).unwrap_err());
        let reused = unsupported(
            cat.reference_metadata_reusing(PackId::Llg, &digest, &mut cache)
                .unwrap_err(),
        );
        assert_eq!(reused, old, "uncached/cache typed error payload parity");
    }
    assert!(matches!(
        engine.execute(DictionaryRequest::Resolve {
            target: target.clone()
        }),
        Err(vlajmaji_search::CollectionQueryError::Catalogue(
            vlajmaji_store::CollectionError::UnsupportedVersion(_)
        ))
    ));
    assert!(matches!(
        engine.execute(DictionaryRequest::Resolve {
            target: local.clone()
        }),
        Err(vlajmaji_search::CollectionQueryError::Catalogue(
            vlajmaji_store::CollectionError::UnsupportedVersion(_)
        ))
    ));
    write_kind(&db, &digest, &kind);
    let restored_db = sha(&std::fs::read(&db).unwrap());
    assert_eq!(
        value(
            &engine
                .execute(DictionaryRequest::Resolve {
                    target: target.clone()
                })
                .unwrap()
        ),
        value(&baseline)
    );
    assert_eq!(
        value(
            &engine
                .execute(DictionaryRequest::Resolve {
                    target: local.clone()
                })
                .unwrap()
        ),
        value(&local_before)
    );
    println!(
        "PASS SAME cache/engine current bytes and SQLkind corruption→error→exact restore; original Full fields"
    );
    {
        let cat = CollectionCatalogue::open(&root).unwrap();
        cache.retain_registered(&cat, &[]).unwrap();
        let got = cat
            .reference_metadata_reusing(PackId::Llg, &digest, &mut cache)
            .unwrap();
        assert_eq!(
            value(got.as_ref()),
            serde_json::from_slice::<serde_json::Value>(&original).unwrap()
        );
    }
    cache.clear();
    {
        let cat = CollectionCatalogue::open(&root).unwrap();
        let got = cat
            .reference_metadata_reusing(PackId::Llg, &digest, &mut cache)
            .unwrap();
        assert_eq!(
            value(got.as_ref()),
            serde_json::from_slice::<serde_json::Value>(&original).unwrap()
        );
    }
    assert_eq!(
        sha(&std::fs::read(&db).unwrap()),
        restored_db,
        "restore/prune/clear product calls are byte-exact read-only"
    );
    // Same public Raw projection: native success must not erase downstream phase errors.
    let raw_baseline = {
        let cat = CollectionCatalogue::open(&root).unwrap();
        cat.reference_projection(PackId::Llg, &digest, &target)
            .unwrap()
    };
    assert!(
        raw_baseline
            .entities
            .iter()
            .any(|e| e.id == assertion.id.as_str())
    );
    let mut raw_bad: serde_json::Value = serde_json::from_slice(&original).unwrap();
    raw_bad["registry"]["occurrences"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!({"later_bad_key":true}));
    let original_captures = raw_bad["captures"].clone();
    raw_bad["captures"][0] = serde_json::json!({"late_bad_capture":true});
    write_descriptor(&db, &digest, &serde_json::to_vec(&raw_bad).unwrap());
    let bad_capture_pair = sha(&std::fs::read(&db).unwrap());
    {
        let cat = CollectionCatalogue::open(&root).unwrap();
        match cat
            .reference_projection(PackId::Llg, &digest, &target)
            .unwrap_err()
        {
            vlajmaji_store::CollectionError::ReferenceProjection { source, .. } => {
                assert!(source.is_data());
                assert!(
                    source.to_string().contains("capture"),
                    "capture phase must precede later key error: {source}"
                );
            }
            e => panic!("wrong error: {e:?}"),
        }
    }
    assert_eq!(sha(&std::fs::read(&db).unwrap()), bad_capture_pair);
    raw_bad["captures"] = original_captures;
    write_descriptor(&db, &digest, &serde_json::to_vec(&raw_bad).unwrap());
    {
        let cat = CollectionCatalogue::open(&root).unwrap();
        match cat
            .reference_projection(PackId::Llg, &digest, &target)
            .unwrap_err()
        {
            vlajmaji_store::CollectionError::ReferenceProjection { source, .. } => {
                assert!(source.is_data());
                assert!(
                    source.to_string().contains("occurrence"),
                    "restored capture reaches later key error: {source}"
                );
            }
            e => panic!("wrong error: {e:?}"),
        }
    }
    write_descriptor(&db, &digest, &original);
    {
        let cat = CollectionCatalogue::open(&root).unwrap();
        assert_eq!(
            value(
                &cat.reference_projection(PackId::Llg, &digest, &target)
                    .unwrap()
            ),
            value(&raw_baseline)
        );
    }
    let restored_db = sha(&std::fs::read(&db).unwrap());
    println!(
        "PASS public Raw native phase vs late capture/registry typed errors and whole restored projection"
    );
    let (profiled, profile) = engine
        .execute_profiled(DictionaryRequest::Resolve { target })
        .unwrap();
    assert_eq!(value(&profiled), value(&baseline));
    assert_eq!(
        profile.query.lookup.as_nanos() + profile.query.projection.as_nanos(),
        profile.query.total.as_nanos()
    );
    assert_eq!(
        sha(&std::fs::read(&db).unwrap()),
        restored_db,
        "post-restore product calls remain byte-exact read-only"
    );
    assert_eq!(
        CollectionCatalogue::open(&root)
            .unwrap()
            .local_revision()
            .unwrap(),
        original_revision
    );
    {
        let c = Connection::open(&db).unwrap();
        let observed:(String,Vec<u8>)=c.query_row("SELECT payload_kind,descriptor_json FROM registrations_v2 WHERE pack_id='llg' AND release_sha256=?1",params![&digest],|r|Ok((r.get(0)?,r.get(1)?))).unwrap();
        assert_eq!(observed, (kind, original));
    }
    println!(
        "PASS prune/clear unchanged semantics and SAME-call Full Resolve profiled parity/equation; no performance acceptance"
    );
    // Original affected profile denominator on SAME restored four-pack Tiny ROOT.
    // This intentionally publishes ONE Tiny generation after retiring old snapshot engines.
    drop(engine);
    let request = CollectionRequest::new(
        packs
            .iter()
            .map(|(p, s)| CollectionInput::SealedBytes {
                pack_id: *p,
                release: Cow::Borrowed(&s.release),
                seal: Cow::Borrowed(&s.seal),
            })
            .collect(),
        CollectionSelection::Packs(PACKS.to_vec()),
        true,
    )
    .unwrap();
    let installed = CollectionCatalogue::install(&root, request).unwrap();
    assert_eq!(
        installed.local_revision,
        original_revision.checked_add(1).unwrap()
    );
    let generation = installed.generation.unwrap();
    let relative = generation
        .store_path
        .strip_prefix(&root)
        .expect("owned ROOT containment before candidate-path IO");
    assert!(!relative.as_os_str().is_empty());
    assert!(
        relative
            .components()
            .all(|p| matches!(p, std::path::Component::Normal(_)))
    );
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        assert_eq!(generation.path.encoding, "unix-bytes");
        assert_eq!(
            generation.path.hex,
            generation
                .store_path
                .as_os_str()
                .as_bytes()
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
        );
    }
    assert_eq!(
        generation.path.utf8.as_deref(),
        generation.store_path.to_str()
    );
    assert_owned_store_path(&root, &generation.store_path);
    let expected_members: BTreeMap<_, _> = packs
        .iter()
        .map(|(p, s)| (p.as_str().to_owned(), sha(&s.release)))
        .collect();
    assert_eq!(
        generation
            .members
            .iter()
            .map(|m| (m.pack_id.clone(), m.release_sha256.clone()))
            .collect::<BTreeMap<_, _>>(),
        expected_members
    );
    let store_db = generation.store_path.join("dictionary.sqlite3");
    db_leaf(&store_db);
    let before_store_profile = sha(&std::fs::read(&store_db).unwrap());
    let before_profile = sha(&std::fs::read(&db).unwrap());
    let docs: Vec<_> = input.iter().cloned().map(draft).collect();
    profile_parity_checks(&root, &generation.store_path, &docs);
    assert_eq!(
        sha(&std::fs::read(&store_db).unwrap()),
        before_store_profile,
        "payload store profile reads are byte-exact readonly"
    );
    assert_eq!(
        sha(&std::fs::read(&db).unwrap()),
        before_profile,
        "affected profile reads are byte-exact readonly"
    );
}
