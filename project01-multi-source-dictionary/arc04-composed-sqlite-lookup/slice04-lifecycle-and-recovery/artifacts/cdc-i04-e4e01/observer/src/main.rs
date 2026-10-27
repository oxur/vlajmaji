// PROSPECTIVE independent Full observer. No build/run before frozen handback and readiness.
include!("helpers.rs");
include!("descriptor.rs");
use serde_json::{Value, json};
use std::{borrow::Cow, io::Write, path::Path};
use vlajmaji_search::{
    CollectionQueryResult, CollectionQuerySelection, CollectionSearchEngine,
    DictionaryExactOptions, DictionaryRequest, DictionarySearchEngine, ReferenceStatus,
};
use vlajmaji_store::{
    CollectionCatalogue, CollectionInput, CollectionRequest, CollectionSelection, DictionaryStore,
};
type Inventory = BTreeMap<String, BTreeMap<String, usize>>;
const IDS: [PackId; 4] = [
    PackId::LensiskuJbotci,
    PackId::Llg,
    PackId::LaLojban,
    PackId::Sutysisku,
];
const CANON: &str = "d9d73cbb49cc5a9ddb0ec16cb6dbb4bb7260ad49f4f8f54c6e6d21f0779b7b19";
const CONTENT: &str = "b4ee3e48edb910ce63b892f2a1d61342c26d3f2a1a46545f93ab15157192766e";
fn pack(x: &str) -> PackId {
    IDS.into_iter()
        .find(|p| p.as_str() == x)
        .expect("declared pack")
}
fn rows(d: &UnverifiedDictionaryDocument) -> Inventory {
    inventory(d)
        .into_iter()
        .map(|(k, v)| (k.to_owned(), v))
        .collect()
}
fn union(into: &mut Inventory, one: Inventory) {
    for (k, rs) in one {
        let dst = into.entry(k).or_default();
        for (k, n) in rs {
            let e = dst.entry(k).or_default();
            *e = (*e).max(n);
        }
    }
}
fn write_new(path: &Path, b: &[u8]) {
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .unwrap();
    f.write_all(b).unwrap();
    f.sync_all().unwrap();
}
fn selected_release_memberships(
    d: &UnverifiedDictionaryDocument,
    ids: &BTreeSet<String>,
) -> BTreeMap<String, Vec<EntityId>> {
    d.metadata
        .assertions
        .iter()
        .filter(|a| ids.contains(a.id.as_str()))
        .map(|a| {
            (
                a.id.to_string(),
                d.metadata
                    .releases
                    .iter()
                    .filter(|r| r.assertions.contains(&a.id))
                    .map(|r| r.id.clone())
                    .collect(),
            )
        })
        .collect()
}
fn qualify(root: &Path) {
    let ds = models();
    assert_eq!(
        ds.iter()
            .map(|d| d.metadata().assertions.len())
            .collect::<Vec<_>>(),
        [1, 1, 1, 3]
    );
    // Qualify the SAME membership builder used by freeze against direct literal-input fields.
    let jb = draft(ds[0].clone());
    let aid = jb.metadata.assertions[0].id.clone();
    let ids = BTreeSet::from([aid.to_string()]);
    let literal_release = jb
        .metadata
        .releases
        .iter()
        .find(|r| r.assertions == vec![aid.clone()])
        .expect("literal JB release owns its one assertion")
        .id
        .clone();
    let want_memberships = BTreeMap::from([(aid.to_string(), vec![literal_release.clone()])]);
    assert_eq!(selected_release_memberships(&jb, &ids), want_memberships);
    let mut changed_membership = draft(ds[0].clone());
    for r in &mut changed_membership.metadata.releases {
        r.assertions.retain(|a| a != &aid);
    }
    assert_ne!(
        selected_release_memberships(&changed_membership, &ids),
        want_memberships
    );
    let mut wrong_release = want_memberships.clone();
    wrong_release.get_mut(aid.as_str()).unwrap().clear();
    assert_ne!(selected_release_memberships(&jb, &ids), wrong_release);
    assert!(selected_release_memberships(&jb, &BTreeSet::new()).is_empty());
    drop(changed_membership);
    drop(jb);
    let mut all = Inventory::new();
    let mut subset = Inventory::new();
    for (i, d) in ds.iter().enumerate() {
        let x = rows(&draft(d.clone()));
        union(&mut all, x.clone());
        if i != 1 {
            union(&mut subset, x);
        }
    }
    assert_eq!(all.len(), 25);
    assert_eq!(subset.len(), 25);
    assert!(all["frames"].is_empty());
    let mut reverse = Inventory::new();
    for d in ds.iter().rev() {
        union(&mut reverse, rows(&draft(d.clone())));
    }
    assert_eq!(all, reverse);
    let mut bad = all.clone();
    bad.remove("frames");
    assert_ne!(bad, all);
    bad = all.clone();
    bad.insert("extra_empty".into(), BTreeMap::new());
    assert_ne!(bad, all);
    bad = all.clone();
    *bad.get_mut("assertions")
        .unwrap()
        .values_mut()
        .next()
        .unwrap() += 1;
    assert_ne!(bad, all);
    let mut changed = draft(ds[0].clone());
    changed.metadata.entries[0].spelling = Presence::Missing;
    assert_ne!(rows(&changed), rows(&draft(ds[0].clone())));
    let correct = expected(&draft(ds[0].clone()));
    changed = draft(ds[0].clone());
    changed.metadata.assertions[0].occurrence.locator = "changed reference".into();
    assert_ne!(expected(&changed), correct);
    changed = draft(ds[0].clone());
    changed.metadata.entries[0].spelling = Presence::Missing;
    assert_ne!(expected(&changed), correct);
    let mut missing = correct.clone();
    missing["category_counts"]
        .as_object_mut()
        .unwrap()
        .remove("frames");
    assert_ne!(missing, correct);
    let mut extra = correct.clone();
    extra["category_counts"]["extra_empty"] = json!(0);
    assert_ne!(extra, correct);
    let seals = IDS
        .iter()
        .zip(&ds)
        .map(|(p, d)| (*p, seal_release(*p, PackPayloadKind::Synthetic, d).unwrap()))
        .collect::<Vec<_>>();
    let request = CollectionRequest::new(
        seals
            .iter()
            .map(|(p, s)| CollectionInput::SealedBytes {
                pack_id: *p,
                release: Cow::Borrowed(&s.release),
                seal: Cow::Borrowed(&s.seal),
            })
            .collect(),
        CollectionSelection::All,
        true,
    )
    .unwrap();
    let report = CollectionCatalogue::install(root, request).unwrap();
    let g = report.generation.unwrap();
    {
        let s = DictionaryStore::open(&g.store_path).unwrap();
        let d = s.unverified_document().unwrap();
        assert_eq!(rows(&d), all);
        drop(d);
        drop(s);
    }
    {
        let c = CollectionCatalogue::open(root).unwrap();
        for ((p, d), (_, s)) in IDS.iter().zip(&ds).zip(&seals) {
            let desc = c.reference_metadata(*p, &sha(&s.release)).unwrap();
            assert_eq!(actual(&desc), expected(&draft(d.clone())));
            drop(desc);
        }
        drop(c);
    }
    let r = CollectionCatalogue::remove(root, &[PackId::Llg], 1).unwrap();
    {
        let s = DictionaryStore::open(&r.generation.unwrap().store_path).unwrap();
        let d = s.unverified_document().unwrap();
        assert_eq!(rows(&d), subset);
        drop(d);
        drop(s);
    }
    println!(
        "{}",
        json!({"ok":true,"scope":"SAME Full inventory/descriptor builders and comparator on literal Tiny all/subset; no Full access","raw":6,"lexical":5,"families":25,"controls":["all/subset actual store positive","input reorder positive","missing empty negative","extra empty negative","multiplicity negative","Presence negative","descriptor changed occurrence negative","descriptor Presence negative","descriptor missing empty negative","descriptor extra empty negative","all four descriptor positive","literal release membership positive","removed membership negative","wrong release negative","empty witness selection positive"]})
    );
}
fn freeze(batch: &Path, out: &Path) {
    let b: Value = serde_json::from_slice(&std::fs::read(batch).unwrap()).unwrap();
    let mut all = Inventory::new();
    let mut subset = Inventory::new();
    let mut metadata = Vec::new();
    let accepted: Value =
        serde_json::from_str(include_str!("../accepted-descriptors-and-refs02.json")).unwrap();
    let witness_ids = accepted["inputs"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|p| p["farvi"].as_array().unwrap())
        .map(|w| w["assertion"]["id"].as_str().unwrap().to_owned())
        .collect::<std::collections::BTreeSet<_>>();
    for item in b["inputs"].as_array().unwrap() {
        let p = pack(item["pack_id"].as_str().unwrap());
        let raw = std::fs::read(item["release"].as_str().unwrap()).unwrap();
        let sealbytes = std::fs::read(item["seal"].as_str().unwrap()).unwrap();
        let seal = PackSeal::from_json_bytes(&sealbytes).unwrap();
        assert_eq!(seal.pack_id, p);
        assert_eq!(seal.payload_kind, PackPayloadKind::Full);
        assert_eq!(sha(&raw), seal.release_sha256);
        assert_eq!(raw.len() as u64, seal.release_bytes);
        assert_eq!(seal.dictionary_version, 1);
        assert_eq!(seal.registry_version, 1);
        let d = UnverifiedDictionaryDocument::from_unverified_json_bytes(&raw).unwrap();
        drop(raw);
        assert_eq!(d.version, 1);
        assert_eq!(d.registry.version, 1);
        assert!(d.legacy_archive.is_none());
        let x = rows(&d);
        assert_eq!(x.len(), 25);
        union(&mut all, x.clone());
        if p != PackId::Llg {
            union(&mut subset, x);
        }
        let memberships = selected_release_memberships(&d, &witness_ids);
        metadata.push(json!({"pack_id":p,"release_sha256":seal.release_sha256,"seal_sha256":sha(&sealbytes),"payload_kind":"full","descriptor":expected(&d),"upstream_catalogue_revision":d.metadata.catalogue_revision,"farvi_memberships":memberships}));
        drop(d);
        drop(sealbytes);
    }
    assert_eq!(metadata.len(), 4);
    assert_eq!(all.len(), 25);
    assert_eq!(subset.len(), 25);
    let bytes =
        serde_json::to_vec(&json!({"version":1,"all":all,"subset":subset,"inputs":metadata}))
            .unwrap();
    write_new(out, &bytes);
    println!(
        "{}",
        json!({"ok":true,"scope":"pre-install independent input fingerprints, typed-unverified source read; not product validation","bytes":bytes.len(),"sha256":sha(&bytes)})
    );
}
fn observe(
    root: &Path,
    expected_file: &Path,
    witness_file: &Path,
    subset: bool,
    revision: u64,
    out: &Path,
) {
    let exp: Value = serde_json::from_slice(&std::fs::read(expected_file).unwrap()).unwrap();
    let want: Inventory =
        serde_json::from_value(exp[if subset { "subset" } else { "all" }].clone()).unwrap();
    let witnesses: Value = serde_json::from_slice(&std::fs::read(witness_file).unwrap()).unwrap();
    let c = CollectionCatalogue::open(root).unwrap();
    assert_eq!(c.local_revision().unwrap(), revision);
    let g = c.generation().unwrap().unwrap();
    assert_eq!(g.members.len(), if subset { 3 } else { 4 });
    assert_eq!(g.members.iter().any(|m| m.pack_id == PackId::Llg.as_str()), !subset);
    if !subset {
        assert_eq!(g.canonical_sha256, CANON);
        assert_eq!(g.content_id, CONTENT);
    }
    for item in exp["inputs"].as_array().unwrap() {
        let p = pack(item["pack_id"].as_str().unwrap());
        let desc = c
            .reference_metadata(p, item["release_sha256"].as_str().unwrap())
            .unwrap();
        assert_eq!(actual(&desc), item["descriptor"]);
        drop(desc);
    }
    drop(c);
    let expected_catalogue_revision = exp["inputs"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|p| !subset || p["pack_id"] != "llg")
        .map(|p| p["upstream_catalogue_revision"].as_u64().unwrap())
        .max()
        .unwrap();
    let memberships: BTreeMap<String, Vec<EntityId>> = exp["inputs"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|p| {
            serde_json::from_value::<BTreeMap<String, Vec<EntityId>>>(
                p["farvi_memberships"].clone(),
            )
            .unwrap()
        })
        .collect();
    assert_eq!(memberships.len(), 5);
    // Scope 1: ONE Store/model. No query engine or descriptor body survives here.
    {
        let s = DictionaryStore::open(&g.store_path).unwrap();
        let d = s.unverified_document().unwrap();
        assert_eq!(rows(&d), want);
        assert_eq!(d.metadata.catalogue_revision, expected_catalogue_revision);
        assert!(d.legacy_archive.is_none());
        assert_eq!(d.version, 1);
        assert_eq!(d.registry.version, 1);
        drop(d);
        drop(s);
    }
    drop(want);
    drop(exp);
    // Scope 2: ONE explicit-store engine. All is a SEPARATE serialized process outside this observer.
    {
        let e = DictionarySearchEngine::open(&g.store_path).unwrap();
        let full = e.exact("farvi", DictionaryExactOptions::default()).unwrap();
        assert_eq!(full.totals.occurrences, if subset { 4 } else { 5 });
        assert_eq!(full.totals.source_groups, if subset { 2 } else { 3 });
        let totals: BTreeMap<_, _> = full
            .source_totals
            .iter()
            .filter(|s| s.occurrences > 0)
            .map(|s| (s.source.id.to_string(), s.occurrences))
            .collect();
        let mut t = BTreeMap::from([("lensisku".into(), 3), ("sutysisku".into(), 1)]);
        if !subset {
            t.insert("llg".into(), 1);
        }
        assert_eq!(totals, t);
        let ids = full
            .groups
            .iter()
            .flat_map(|g| g.hits.iter().map(|h| h.assertion.id.clone()))
            .collect::<Vec<_>>();
        let bytes = [
            serde_json::to_vec(&serde_json::to_value(&full).unwrap()).unwrap(),
            vec![b'\n'],
        ]
        .concat();
        if !subset {
            assert_eq!(bytes.len(), 7160366);
            assert_eq!(
                sha(&bytes),
                "144b8b09b6e378243b36c8e01ab06353c3d573459668c8c158a86aebbea73556"
            );
        }
        write_new(out, &bytes);
        drop(bytes);
        drop(full);
        for size in [1, 2, 3] {
            let mut cursor = None;
            let mut got = Vec::new();
            let mut n = 0;
            loop {
                n += 1;
                assert!(n <= 5);
                let x = e
                    .exact(
                        "farvi",
                        DictionaryExactOptions {
                            page_size: Some(size),
                            cursor,
                            ..Default::default()
                        },
                    )
                    .unwrap();
                assert_eq!(x.totals.occurrences, ids.len());
                got.extend(
                    x.groups
                        .into_iter()
                        .flat_map(|g| g.hits.into_iter().map(|h| h.assertion.id)),
                );
                cursor = x.next_cursor;
                if cursor.is_none() {
                    break;
                }
            }
            assert_eq!(got, ids);
        }
        let p0 = e
            .exact(
                "farvi",
                DictionaryExactOptions {
                    page_size: Some(0),
                    ..Default::default()
                },
            )
            .unwrap();
        assert_eq!(p0.totals.occurrences, ids.len());
        assert!(p0.groups.is_empty());
        drop(p0);
        drop(e);
    }
    // Scope 3: ONE managed engine only after the explicit engine/model has dropped.
    {
        // Resolve admission is target-scoped. Keep removed LLG selected here so its
        // truthful reason is removed, rather than not_selected; exact uses subset above.
        let mut e = CollectionSearchEngine::open(
            root,
            CollectionQuerySelection {
                packs: IDS.to_vec(),
                disabled: vec![],
                profile: None,
            },
        )
        .unwrap();
        let mut refs = 0;
        for item in witnesses["inputs"].as_array().unwrap() {
            let removed = subset && item["pack_id"] == "llg";
            for w in item["farvi"].as_array().unwrap() {
                let target: RelationTarget = serde_json::from_value(w["target"].clone()).unwrap();
                let r = match e
                    .execute(DictionaryRequest::Resolve {
                        target: target.clone(),
                    })
                    .unwrap()
                    .result
                {
                    CollectionQueryResult::Resolve(r) => r,
                    _ => panic!("wrong response"),
                };
                assert_eq!(r.target, target);
                assert_eq!(
                    r.status,
                    if removed {
                        ReferenceStatus::CatalogueOnly
                    } else {
                        ReferenceStatus::Resolved
                    }
                );
                if !removed {
                    assert!(r.operational_reason.is_none());
                    let payload = r.payload.as_ref().unwrap();
                    assert_eq!(payload.confirmed.len(), 1);
                    assert_eq!(
                        serde_json::to_value(&payload.confirmed[0].assertion).unwrap(),
                        w["assertion"]
                    );
                    assert_eq!(
                        serde_json::to_value(&payload.confirmed[0].occurrence_attribution).unwrap(),
                        w["attribution"]
                    );
                    assert_eq!(
                        &payload.confirmed[0].release_ids,
                        &memberships[w["assertion"]["id"].as_str().unwrap()]
                    );
                    assert_eq!(
                        serde_json::to_value(&r.requested_release.as_ref().unwrap().id).unwrap(),
                        w["release_id"]
                    );
                } else {
                    assert!(r.payload.is_none());
                    assert_eq!(r.operational_reason.as_deref(), Some("removed"));
                    assert!(r.catalogue_matches.iter().any(|m| m.pack_id == "llg"
                        && serde_json::to_value(&m.model_release_id).unwrap() == w["release_id"]));
                }
                refs += 1;
                drop(r);
            }
        }
        assert_eq!(refs, 5);
        drop(e);
    }
    println!(
        "{}",
        json!({"ok":true,"scope":"complete typed25-family, all four retained descriptor fingerprints, full wire/pages and five scoped references","subset":subset,"revision":revision,"generation_id":g.generation_id,"store_path":g.store_path,"canonical_sha256":g.canonical_sha256,"wire_file":out})
    );
}
// Separate Synthetic method route. It cannot select Full input, root, pins, or
// weaken any Full-mode assertion. Inventory builders are shared with Full.
fn prepare_tiny_method(destination: &Path) {
    std::fs::create_dir(destination).unwrap();
    let ds = models();
    let mut inputs = Vec::new();
    for (p, d) in IDS.iter().zip(ds) {
        let sealed = seal_release(*p, PackPayloadKind::Synthetic, &d).unwrap();
        let dir = destination.join(p.as_str());
        std::fs::create_dir(&dir).unwrap();
        let release = dir.join("release.json");
        let seal = dir.join("seal.json");
        write_new(&release, &sealed.release);
        write_new(&seal, &sealed.seal);
        inputs.push(json!({"kind":"sealed","pack_id":p,"release":release,"seal":seal}));
    }
    write_new(&destination.join("manifest.json"), &serde_json::to_vec(&json!({"version":1,"inputs":inputs})).unwrap());
    println!("{}", json!({"ok":true,"scope":"original literal four-pack Synthetic inputs for actual CLI runner method qualification; no Full"}));
}
fn freeze_tiny_method(batch: &Path, output: &Path) {
    let b: Value = serde_json::from_slice(&std::fs::read(batch).unwrap()).unwrap();
    let mut all = Inventory::new();
    let mut subset = Inventory::new();
    let mut descriptors = Vec::new();
    for item in b["inputs"].as_array().unwrap() {
        let p = pack(item["pack_id"].as_str().unwrap());
        let release = std::fs::read(item["release"].as_str().unwrap()).unwrap();
        let sealbytes = std::fs::read(item["seal"].as_str().unwrap()).unwrap();
        let seal = PackSeal::from_json_bytes(&sealbytes).unwrap();
        assert_eq!(seal.payload_kind, PackPayloadKind::Synthetic);
        assert_eq!(seal.pack_id,p);
        assert_eq!(seal.release_sha256,sha(&release));
        let d = load_sealed_release(p,&release,&sealbytes).unwrap();
        let d = draft(d);
        let one = rows(&d);
        assert_eq!(one.len(),25);
        union(&mut all,one.clone());
        if p != PackId::Llg {union(&mut subset,one);}
        descriptors.push(json!({"pack_id":p,"release_sha256":seal.release_sha256,"descriptor":expected(&d)}));
    }
    write_new(output,&serde_json::to_vec(&json!({"version":1,"all":all,"subset":subset,"inputs":descriptors})).unwrap());
    println!("{}",json!({"ok":true,"scope":"Synthetic-only SAME 25-family/max-multiset and descriptor builders before installation"}));
}
fn observe_tiny_method(root: &Path, expected_file: &Path, subset: bool, revision: u64, out: &Path) {
    let exp: Value=serde_json::from_slice(&std::fs::read(expected_file).unwrap()).unwrap();
    let want: Inventory=serde_json::from_value(exp[if subset {"subset"} else {"all"}].clone()).unwrap();
    let c=CollectionCatalogue::open(root).unwrap();
    assert_eq!(c.local_revision().unwrap(),revision);
    let g=c.generation().unwrap().unwrap();
    for item in exp["inputs"].as_array().unwrap() {
        let desc=c.reference_metadata(pack(item["pack_id"].as_str().unwrap()),item["release_sha256"].as_str().unwrap()).unwrap();
        assert_eq!(actual(&desc),item["descriptor"]);
        drop(desc);
    }
    drop(c);
    {
        let store=DictionaryStore::open(&g.store_path).unwrap();
        let d=store.unverified_document().unwrap();
        assert_eq!(rows(&d),want);
        drop(d);drop(store);
    }
    drop(want);drop(exp);
    {
        let e=DictionarySearchEngine::open(&g.store_path).unwrap();
        let response=e.exact("farvi",DictionaryExactOptions::default()).unwrap();
        assert_eq!(response.totals.occurrences,if subset {4} else {5});
        let mut bytes=serde_json::to_vec(&serde_json::to_value(response).unwrap()).unwrap();
        bytes.push(b'\n');write_new(out,&bytes);drop(bytes);drop(e);
    }
    println!("{}",json!({"ok":true,"scope":"Synthetic-only actual CLI orchestration support; shared complete typed25/descriptor checks and full explicit wire"}));
}
fn main() {
    let a = std::env::args_os().collect::<Vec<_>>();
    match a[1].to_str().unwrap() {
        "qualify" => qualify(Path::new(&a[2])),
        "prepare-tiny" => prepare_tiny_method(Path::new(&a[2])),
        "freeze-tiny" => freeze_tiny_method(Path::new(&a[2]),Path::new(&a[3])),
        "observe-tiny" => observe_tiny_method(Path::new(&a[2]),Path::new(&a[3]),a[5]=="subset",a[6].to_str().unwrap().parse().unwrap(),Path::new(&a[7])),
        "freeze" => freeze(Path::new(&a[2]), Path::new(&a[3])),
        "observe" => observe(
            Path::new(&a[2]),
            Path::new(&a[3]),
            Path::new(&a[4]),
            a[5] == "subset",
            a[6].to_str().unwrap().parse().unwrap(),
            Path::new(&a[7]),
        ),
        _ => panic!("unsupported declared mode"),
    }
}
