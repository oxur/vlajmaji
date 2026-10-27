// PROSPECTIVE independent S04 iteration01 caller. No compilation/execution before handback.
// Original exact literal/helper bytes copied from accepted initial caller; new scenarios are not yet proof.
include!("helpers.rs");
#[allow(dead_code)]
mod supplemental {
    include!("../fixtures/complete_fixture.rs");
}
use std::{
    borrow::Cow,
    path::{Path, PathBuf},
};
use vlajmaji_search::{
    CollectionQueryError, CollectionQueryResult, CollectionQuerySelection, CollectionSearchEngine,
    DictionaryExactOptions, DictionaryRequest, DictionarySearchEngine, ReferenceStatus,
};
use vlajmaji_store::{
    CollectionCatalogue, CollectionError, CollectionGeneration, CollectionInput,
    CollectionInstallationState, CollectionPolicySelection, CollectionProfileEnablementRequest,
    CollectionProfileWriteRequest, CollectionProfiles, CollectionRequest, CollectionSelection,
    DictionaryStore,
};
const IDS: [PackId; 4] = [
    PackId::LensiskuJbotci,
    PackId::Llg,
    PackId::LaLojban,
    PackId::Sutysisku,
];
type Seals = Vec<(PackId, Vec<u8>, Vec<u8>)>;
fn seals(ds: &[Dictionary]) -> Seals {
    IDS.iter()
        .zip(ds)
        .map(|(id, d)| {
            let x = seal_release(*id, PackPayloadKind::Synthetic, d).unwrap();
            (*id, x.release, x.seal)
        })
        .collect()
}
fn req(xs: &[(PackId, Vec<u8>, Vec<u8>)]) -> CollectionRequest<'_> {
    CollectionRequest::new(
        xs.iter()
            .map(|(id, r, s)| CollectionInput::SealedBytes {
                pack_id: *id,
                release: Cow::Borrowed(r),
                seal: Cow::Borrowed(s),
            })
            .collect(),
        CollectionSelection::Packs(xs.iter().map(|x| x.0).collect()),
        true,
    )
    .unwrap()
}
fn expected(ds: &[Dictionary]) -> BTreeMap<&'static str, BTreeMap<String, usize>> {
    let mut all: BTreeMap<_, BTreeMap<_, usize>> = BTreeMap::new();
    for d in ds {
        for (k, rs) in inventory(&draft(d.clone())) {
            let out = all.entry(k).or_default();
            for (k, n) in rs {
                let old = out.entry(k).or_default();
                *old = (*old).max(n);
            }
        }
    }
    all
}
fn qual(ds: &[Dictionary]) {
    assert_eq!(
        ds.iter()
            .map(|d| d.metadata().assertions.len())
            .collect::<Vec<_>>(),
        vec![1, 1, 1, 3]
    );
    assert_eq!(
        ds.iter()
            .map(|d| d
                .metadata()
                .assertions
                .iter()
                .filter(|x| x.eligibility == Eligibility::Lexical)
                .count())
            .collect::<Vec<_>>(),
        vec![1, 1, 1, 2]
    );
    let e = expected(ds);
    assert_eq!(e.len(), 25);
    assert!(e["frames"].is_empty());
    let mut x = e.clone();
    x.remove("frames");
    assert_ne!(x, e);
    let mut x = e.clone();
    x.insert("unknown_empty", BTreeMap::new());
    assert_ne!(x, e);
    let mut x = e.clone();
    *x.get_mut("assertions")
        .unwrap()
        .values_mut()
        .next()
        .unwrap() += 1;
    assert_ne!(x, e);
    let mut x = draft(ds[0].clone());
    x.metadata.entries[0].spelling = Presence::Missing;
    assert_ne!(inventory(&x), inventory(&draft(ds[0].clone())));
    let mut x = draft(ds[3].clone());
    let before = x.metadata.assertions.clone();
    x.metadata.assertions.reverse();
    assert_ne!(before, x.metadata.assertions);
    assert_eq!(inventory(&x), inventory(&draft(ds[3].clone())));
    println!(
        "PASS pre-input literal6/lexical5 and exact25-family comparator: reorder positive, missing/extra empty, multiplicity and Presence negatives"
    );
}
fn current(root: &Path) -> (u64, Option<CollectionGeneration>) {
    let c = CollectionCatalogue::open(root).unwrap();
    (c.local_revision().unwrap(), c.generation().unwrap())
}
fn db(root: &Path) -> String {
    sha(&std::fs::read(root.join("catalogue.sqlite3")).unwrap())
}
fn store_sha(g: &CollectionGeneration) -> String {
    sha(&std::fs::read(g.store_path.join("dictionary.sqlite3")).unwrap())
}
fn store_inventory(g: &CollectionGeneration) -> BTreeMap<&'static str, BTreeMap<String, usize>> {
    let s = DictionaryStore::open(&g.store_path).unwrap();
    inventory(&s.unverified_document().unwrap())
}
fn engine(root: &Path, ids: Vec<PackId>) -> CollectionSearchEngine {
    CollectionSearchEngine::open(
        root,
        CollectionQuerySelection {
            packs: ids,
            disabled: vec![],
            profile: None,
        },
    )
    .unwrap()
}
fn exact(size: Option<usize>, cursor: Option<String>) -> DictionaryRequest {
    DictionaryRequest::Exact {
        spelling: "farvi".into(),
        options: DictionaryExactOptions {
            page_size: size,
            cursor,
            ..Default::default()
        },
    }
}
fn counts(root: &Path, ids: Vec<PackId>, occ: usize, groups: usize) {
    let mut e = engine(root, ids);
    match e.execute(exact(None, None)).unwrap().result {
        CollectionQueryResult::Exact(x) => {
            assert_eq!(x.totals.occurrences, occ);
            assert_eq!(x.groups.len(), groups)
        }
        _ => panic!("wrong result"),
    }
}
fn native_target(d: &Dictionary) -> RelationTarget {
    let a = d
        .metadata()
        .assertions
        .iter()
        .find(|a| a.eligibility == Eligibility::Lexical)
        .unwrap();
    let e = d
        .metadata()
        .entries
        .iter()
        .find(|e| e.id == a.entry)
        .unwrap();
    let r = d
        .metadata()
        .releases
        .iter()
        .find(|r| r.assertions.contains(&a.id))
        .unwrap();
    let o = d
        .registry()
        .as_document()
        .occurrences
        .iter()
        .find(|o| o.occurrence == a.occurrence.to_record_identity().unwrap())
        .unwrap();
    RelationTarget::Catalog(CatalogTarget {
        source: e.source.clone(),
        release: r.id.clone(),
        native_entry_id: o.native_entry_id.clone(),
        native_assertion_id: o.native_assertion_id.clone(),
        occurrence: Some(a.occurrence.clone()),
    })
}
fn ref_status(root: &Path, ids: Vec<PackId>, target: RelationTarget, status: ReferenceStatus) {
    let mut e = engine(root, ids);
    match e
        .execute(DictionaryRequest::Resolve { target })
        .unwrap()
        .result
    {
        CollectionQueryResult::Resolve(r) => assert_eq!(r.status, status),
        _ => panic!("wrong result"),
    }
}
fn events(root: &Path) -> Vec<serde_json::Value> {
    let c = rusqlite::Connection::open_with_flags(
        root.join("catalogue.sqlite3"),
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .unwrap();
    let mut q=c.prepare("SELECT detail FROM attempts_v2 WHERE operation IN ('remove','reinstall') ORDER BY prior_revision").unwrap();
    q.query_map([], |r| r.get::<_, String>(0))
        .unwrap()
        .map(|r| serde_json::from_str(&r.unwrap()).unwrap())
        .collect()
}
fn core_error(e: &CollectionError) -> &CollectionError {
    match e {
        CollectionError::MutationContext { source, .. } => core_error(source),
        _ => e,
    }
}
fn lifecycle(base: &Path, ds: &[Dictionary], xs: &Seals, changed_sut: &Dictionary) {
    let root = base.join("four-pack");
    let before = expected(ds);
    let llg_target = native_target(&ds[1]);
    let llg_sha = sha(&xs[1].1);
    let initial = CollectionCatalogue::install(&root, req(xs)).unwrap();
    let g1 = initial.generation.unwrap();
    let g1hash = store_sha(&g1);
    let old_reader = DictionarySearchEngine::open(&g1.store_path).unwrap();
    let old_wire = serde_json::to_vec(
        &serde_json::to_value(
            old_reader
                .exact("farvi", DictionaryExactOptions::default())
                .unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(store_inventory(&g1), before);
    counts(&root, IDS.to_vec(), 5, 3);
    let initialdb = db(&root);
    let idem =
        CollectionCatalogue::reinstall(&root, req(&xs[1..2]), &llg_sha, initial.local_revision)
            .unwrap();
    assert_eq!(idem.outcome, "idempotent");
    assert_eq!(
        idem.generation.as_ref().unwrap().generation_id,
        g1.generation_id
    );
    assert_eq!(db(&root), initialdb);
    let bad = CollectionCatalogue::reinstall(
        &root,
        req(&xs[1..2]),
        &"0".repeat(64),
        initial.local_revision,
    )
    .unwrap_err();
    assert!(matches!(
        core_error(&bad),
        CollectionError::UnknownRegisteredRelease {
            pack: PackId::Llg,
            ..
        }
    ));
    assert_eq!(db(&root), initialdb);
    let old_cursor = {
        let mut e = engine(&root, IDS.to_vec());
        match e.execute(exact(Some(1), None)).unwrap().result {
            CollectionQueryResult::Exact(r) => r.next_cursor.unwrap(),
            _ => panic!(),
        }
    };
    let removed =
        CollectionCatalogue::remove(&root, &[PackId::Llg], initial.local_revision).unwrap();
    assert_eq!(removed.local_revision, initial.local_revision + 1);
    let g2 = removed.generation.unwrap();
    assert_ne!(g2.generation_id, g1.generation_id);
    let survivors = vec![ds[0].clone(), ds[2].clone(), ds[3].clone()];
    assert_eq!(store_inventory(&g2), expected(&survivors));
    assert_eq!(store_sha(&g1), g1hash);
    counts(&root, vec![IDS[0], IDS[2], IDS[3]], 4, 2);
    let i = CollectionCatalogue::open(&root)
        .unwrap()
        .inspect(PackId::Llg)
        .unwrap();
    assert_eq!(i.installation_state, CollectionInstallationState::Removed);
    assert!(i.installed_release_sha256.is_none());
    let mut e = engine(&root, IDS.to_vec());
    assert!(matches!(
        e.execute(exact(Some(1), Some(old_cursor))),
        Err(CollectionQueryError::StaleCursor)
    ));
    assert!(
        matches!(e.execute(exact(None,None)),Err(CollectionQueryError::UnavailablePack{reason,..}) if reason=="removed")
    );
    drop(e);
    ref_status(
        &root,
        IDS.to_vec(),
        llg_target.clone(),
        ReferenceStatus::CatalogueOnly,
    );
    let stable = db(&root);
    let err = CollectionCatalogue::install(&root, req(&xs[1..2])).unwrap_err();
    assert!(matches!(
        core_error(&err),
        CollectionError::ReinstallRequired { pack: PackId::Llg }
    ));
    assert_eq!(db(&root), stable);
    let restored =
        CollectionCatalogue::reinstall(&root, req(&xs[1..2]), &llg_sha, removed.local_revision)
            .unwrap();
    let g3 = restored.generation.unwrap();
    assert_ne!(g3.generation_id, g1.generation_id);
    assert_eq!(g3.content_id, g1.content_id);
    assert_eq!(g3.canonical_sha256, g1.canonical_sha256);
    assert_eq!(store_inventory(&g3), before);
    assert_eq!(store_sha(&g1), g1hash);
    counts(&root, IDS.to_vec(), 5, 3);
    ref_status(&root, IDS.to_vec(), llg_target, ReferenceStatus::Resolved);
    let gone = CollectionCatalogue::remove(
        &root,
        &[IDS[3], IDS[2], IDS[1], IDS[0]],
        restored.local_revision,
    )
    .unwrap();
    assert!(gone.generation.is_none());
    assert_eq!(current(&root).0, gone.local_revision);
    assert!(current(&root).1.is_none());
    let stable = db(&root);
    let again = CollectionCatalogue::remove(&root, &IDS, gone.local_revision).unwrap();
    assert_eq!(again.outcome, "idempotent");
    assert_eq!(db(&root), stable);
    assert_eq!(store_sha(&g1), g1hash);
    let last = events(&root).pop().unwrap();
    assert_eq!(
        last["packs"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| p["pack_id"].as_str().unwrap())
            .collect::<Vec<_>>(),
        IDS.iter().map(|p| p.as_str()).collect::<Vec<_>>()
    );
    assert!(last["generation_id"].is_null());
    let reconciliation = CollectionCatalogue::reconcile(&root).unwrap();
    assert!(reconciliation.active_generation.is_none());
    assert!(!reconciliation.migration_required_for_mutation);
    assert_eq!(db(&root), stable);
    // Restore all exact members one at a time; no install fallback may resurrect omitted history.
    let mut revision = gone.local_revision;
    for x in xs {
        let one = std::slice::from_ref(x);
        let r = CollectionCatalogue::reinstall(&root, req(one), &sha(&x.1), revision).unwrap();
        revision = r.local_revision;
        assert!(r.generation.is_some());
    }
    counts(&root, IDS.to_vec(), 5, 3);
    assert_eq!(store_inventory(&current(&root).1.unwrap()), before);
    assert_eq!(
        serde_json::to_vec(
            &serde_json::to_value(
                old_reader
                    .exact("farvi", DictionaryExactOptions::default())
                    .unwrap()
            )
            .unwrap()
        )
        .unwrap(),
        old_wire
    );
    drop(old_reader);
    let revision = current(&root).0;
    let current_generation = current(&root).1.unwrap();
    let unchanged_db = db(&root);
    let idem_update = CollectionCatalogue::update(&root, req(&xs[3..4]), revision).unwrap();
    assert_eq!(idem_update.outcome, "idempotent");
    assert_eq!(db(&root), unchanged_db);
    assert_eq!(
        idem_update.generation.unwrap().generation_id,
        current_generation.generation_id
    );
    let changed_seal =
        seal_release(PackId::Sutysisku, PackPayloadKind::Synthetic, changed_sut).unwrap();
    let one = vec![(PackId::Sutysisku, changed_seal.release, changed_seal.seal)];
    let changed_models = vec![
        ds[0].clone(),
        ds[1].clone(),
        ds[2].clone(),
        changed_sut.clone(),
    ];
    let changed_expected = expected(&changed_models);
    let updated = CollectionCatalogue::update(&root, req(&one), revision).unwrap();
    assert_eq!(
        store_inventory(updated.generation.as_ref().unwrap()),
        changed_expected
    );
    assert_eq!(store_sha(&g1), g1hash);
    counts(&root, IDS.to_vec(), 5, 3);
    let rebuilt = CollectionCatalogue::rebuild(&root, updated.local_revision).unwrap();
    assert_ne!(
        rebuilt.generation.as_ref().unwrap().generation_id,
        updated.generation.as_ref().unwrap().generation_id
    );
    assert_eq!(
        rebuilt.generation.as_ref().unwrap().content_id,
        updated.generation.as_ref().unwrap().content_id
    );
    assert_eq!(
        store_inventory(rebuilt.generation.as_ref().unwrap()),
        changed_expected
    );
    println!(
        "PASS current shared-publisher regression: exact update no-op, independently predeclared source-unique Sut revision and content-equal forced rebuild"
    );
    println!(
        "PASS complete25-family subset/null/repeated-remove/exact-all-reinstall with retained original bytes, removed refs/reasons, canonical identities, event order and stale-before-unavailable"
    );
}
fn registered_only(base: &Path, xs: &Seals) {
    let root = base.join("registered-plus-installed");
    let installed = CollectionCatalogue::install(&root, req(&xs[3..4])).unwrap();
    let generation = installed.generation.unwrap();
    let hash = store_sha(&generation);
    let registered = CollectionCatalogue::register(&root, req(&xs[1..2])).unwrap();
    let prior = registered.local_revision;
    let removed = CollectionCatalogue::remove(&root, &[PackId::Llg], prior).unwrap();
    assert_eq!(removed.local_revision, prior + 1);
    let g = removed.generation.unwrap();
    assert_eq!(g.generation_id, generation.generation_id);
    assert_eq!(g.content_id, generation.content_id);
    assert_eq!(store_sha(&g), hash);
    counts(&root, vec![PackId::Sutysisku], 2, 1);
    let event = events(&root).pop().unwrap();
    assert_eq!(event["generation_id"], generation.generation_id);
    assert!(event["packs"][0]["release_sha256"].is_null());
    let stable = db(&root);
    let repeat =
        CollectionCatalogue::remove(&root, &[PackId::Llg], removed.local_revision).unwrap();
    assert_eq!(repeat.outcome, "idempotent");
    assert_eq!(db(&root), stable);
    let root = base.join("never-installed");
    let reg = CollectionCatalogue::register(&root, req(&xs[1..2])).unwrap();
    let stable = db(&root);
    let err =
        CollectionCatalogue::reinstall(&root, req(&xs[1..2]), &sha(&xs[1].1), reg.local_revision)
            .unwrap_err();
    assert_eq!(format!("{:?}", err.category()), "NotInstalled");
    assert_eq!(db(&root), stable);
    let removed = CollectionCatalogue::remove(&root, &[PackId::Llg], reg.local_revision).unwrap();
    assert!(removed.generation.is_none());
    let r = CollectionCatalogue::reinstall(
        &root,
        req(&xs[1..2]),
        &sha(&xs[1].1),
        removed.local_revision,
    )
    .unwrap();
    assert!(r.generation.is_some());
    counts(&root, vec![PackId::Llg], 1, 1);
    println!(
        "PASS registered-only removal Keep vs null, exact-current retained generation, never-installed reinstall refusal and explicit remove/reinstall valid counterpart"
    );
}
fn configured_counts(file: &Path, occ: usize, groups: usize) {
    let p = CollectionProfiles::read(file).unwrap();
    let mut e = CollectionSearchEngine::open_configured(&p, None, None).unwrap();
    match e.execute(exact(None, None)).unwrap().result {
        CollectionQueryResult::Exact(x) => {
            assert_eq!(x.totals.occurrences, occ);
            assert_eq!(x.groups.len(), groups)
        }
        _ => panic!(),
    }
}
fn policy(base: &Path, xs: &Seals) {
    let root = base.join("policy-root");
    CollectionCatalogue::install(&root, req(xs)).unwrap();
    let file = base.join("profiles.toml");
    let c = CollectionProfiles::configure(
        &file,
        CollectionProfileWriteRequest {
            name: "all".into(),
            root: root.clone(),
            packs: IDS.to_vec(),
            disabled: vec![PackId::Llg],
            make_default: true,
        },
    )
    .unwrap();
    configured_counts(&file, 4, 2);
    let stale_cookie = {
        let profiles = CollectionProfiles::read(&file).unwrap();
        let mut e = CollectionSearchEngine::open_configured(&profiles, None, None).unwrap();
        match e.execute(exact(Some(1), None)).unwrap().result {
            CollectionQueryResult::Exact(r) => r.next_cursor.unwrap(),
            _ => panic!(),
        }
    };
    let on = CollectionProfiles::set_enabled(
        &file,
        CollectionProfileEnablementRequest {
            profile: None,
            selection: CollectionPolicySelection::Packs(vec![PackId::Llg]),
            enabled: true,
        },
    )
    .unwrap();
    assert_eq!(on.affected_pack_ids, vec![PackId::Llg]);
    assert_eq!(on.revision, c.revision + 1);
    configured_counts(&file, 5, 3);
    {
        let profiles = CollectionProfiles::read(&file).unwrap();
        let mut e = CollectionSearchEngine::open_configured(&profiles, None, None).unwrap();
        assert!(matches!(
            e.execute(exact(Some(1), Some(stale_cookie))),
            Err(CollectionQueryError::StaleCursor)
        ));
    }
    let mut commented = std::fs::read(&file).unwrap();
    commented.extend_from_slice(b"\n# must survive semantic no-op\n");
    std::fs::write(&file, &commented).unwrap();
    let valid_cookie = {
        let profiles = CollectionProfiles::read(&file).unwrap();
        let mut e = CollectionSearchEngine::open_configured(&profiles, None, None).unwrap();
        match e.execute(exact(Some(1), None)).unwrap().result {
            CollectionQueryResult::Exact(r) => r.next_cursor.unwrap(),
            _ => panic!(),
        }
    };
    let nop = CollectionProfiles::set_enabled(
        &file,
        CollectionProfileEnablementRequest {
            profile: None,
            selection: CollectionPolicySelection::All,
            enabled: true,
        },
    )
    .unwrap();
    assert_eq!(nop.outcome, "unchanged");
    assert!(nop.affected_pack_ids.is_empty());
    assert_eq!(nop.revision, on.revision);
    assert_eq!(std::fs::read(&file).unwrap(), commented);
    {
        let profiles = CollectionProfiles::read(&file).unwrap();
        let mut e = CollectionSearchEngine::open_configured(&profiles, None, None).unwrap();
        assert!(e.execute(exact(Some(1), Some(valid_cookie))).is_ok());
    }
    let off = CollectionProfiles::set_enabled(
        &file,
        CollectionProfileEnablementRequest {
            profile: None,
            selection: CollectionPolicySelection::Packs(vec![
                PackId::Sutysisku,
                PackId::LensiskuJbotci,
            ]),
            enabled: false,
        },
    )
    .unwrap();
    assert_eq!(
        off.affected_pack_ids,
        vec![PackId::LensiskuJbotci, PackId::Sutysisku]
    );
    assert_eq!(off.disabled, off.affected_pack_ids);
    configured_counts(&file, 2, 2);
    let mixed = CollectionProfiles::set_enabled(
        &file,
        CollectionProfileEnablementRequest {
            profile: None,
            selection: CollectionPolicySelection::Packs(vec![PackId::Sutysisku, PackId::Llg]),
            enabled: false,
        },
    )
    .unwrap();
    assert_eq!(mixed.affected_pack_ids, vec![PackId::Llg]);
    assert_eq!(
        mixed.disabled,
        vec![PackId::LensiskuJbotci, PackId::Sutysisku, PackId::Llg]
    );
    configured_counts(&file, 1, 1);
    let narrow = CollectionProfiles::configure(
        &file,
        CollectionProfileWriteRequest {
            name: "narrow".into(),
            root: root.clone(),
            packs: vec![PackId::Llg],
            disabled: vec![],
            make_default: false,
        },
    )
    .unwrap();
    let before_bad = std::fs::read(&file).unwrap();
    let error = CollectionProfiles::set_enabled(
        &file,
        CollectionProfileEnablementRequest {
            profile: Some("narrow".into()),
            selection: CollectionPolicySelection::Packs(vec![PackId::Sutysisku]),
            enabled: false,
        },
    )
    .unwrap_err();
    assert_eq!(
        CollectionQueryError::Profile(error)
            .report_for("disable")
            .category,
        "not_profile_member"
    );
    assert_eq!(std::fs::read(&file).unwrap(), before_bad);
    let error = CollectionProfiles::set_default(&file, Some("unknown-profile")).unwrap_err();
    assert_eq!(
        CollectionQueryError::Profile(error)
            .report_for("set_default")
            .category,
        "unknown_profile"
    );
    assert_eq!(std::fs::read(&file).unwrap(), before_bad);
    let choose = CollectionProfiles::set_default(&file, Some("narrow")).unwrap();
    assert!(choose.affected_pack_ids.is_empty());
    assert_eq!(choose.revision, narrow.revision + 1);
    configured_counts(&file, 1, 1);
    // Clear policy requires no usable catalogue. Temporarily move only this owned
    // root and retain all its exact bytes; restore before the positive set counterpart.
    let held_root = base.join("policy-root-held-for-clear");
    std::fs::rename(&root, &held_root).unwrap();
    let clear = CollectionProfiles::set_default(&file, None).unwrap();
    assert!(!root.exists());
    std::fs::rename(&held_root, &root).unwrap();
    assert!(clear.profile.is_none());
    assert!(clear.root.is_none());
    assert!(clear.affected_pack_ids.is_empty());
    assert!(matches!(
        CollectionProfiles::read(&file).unwrap().profile(None),
        Err(vlajmaji_store::CollectionProfileError::NoDefaultProfile { .. })
    ));
    CollectionProfiles::set_default(&file, Some("all")).unwrap();
    configured_counts(&file, 1, 1);
    println!(
        "PASS partial policy changed-only/member-order accounting, exact comment/no-op, unknown-profile/nonmember unchanged, default clear with missing owned ROOT and restored set counterpart"
    );
}
fn explicit_restore_missing_copy(base: &Path, xs: &Seals, ds: &[Dictionary]) {
    let root = base.join("explicit-restore-missing-owned-copy");
    let installed = CollectionCatalogue::install(&root, req(&xs[1..2])).unwrap();
    let prior = installed.generation.unwrap();
    let removed =
        CollectionCatalogue::remove(&root, &[PackId::Llg], installed.local_revision).unwrap();
    assert!(removed.generation.is_none());
    let original = prior
        .store_path
        .parent()
        .unwrap()
        .join("inputs/llg/release.json");
    let held = original.with_extension("held-original");
    let bytes = std::fs::read(&original).unwrap();
    assert_eq!(sha(&bytes), sha(&xs[1].1));
    std::fs::rename(&original, &held).unwrap();
    assert!(!original.exists());
    let old_store = store_sha(&prior);
    let restored = CollectionCatalogue::reinstall(
        &root,
        req(&xs[1..2]),
        &sha(&xs[1].1),
        removed.local_revision,
    )
    .unwrap();
    let g = restored.generation.unwrap();
    assert_ne!(g.generation_id, prior.generation_id);
    assert_eq!(store_inventory(&g), expected(&[ds[1].clone()]));
    assert!(!original.exists());
    assert_eq!(std::fs::read(&held).unwrap(), bytes);
    assert_eq!(store_sha(&prior), old_store);
    let copied = g
        .store_path
        .parent()
        .unwrap()
        .join("inputs/llg/release.json");
    assert_eq!(std::fs::read(&copied).unwrap(), xs[1].1);
    println!(
        "PASS removed pack restored from explicit valid input into a new owned stage despite missing old owned copy; old missing path, held bytes and old store remain unchanged"
    );
}
fn supplemental_preservation(base: &Path, sut: &Dictionary) {
    let mut document = supplemental::complete_fixture().unwrap().into_document();
    document.legacy_archive = None;
    let complete = Dictionary::from_document(document).unwrap();
    assert!(!complete.metadata().forms.is_empty());
    assert!(!complete.metadata().editorial_senses.is_empty());
    let one_expected = inventory(&draft(complete.clone()));
    let two_expected = expected(&[complete.clone(), sut.clone()]);
    println!(
        "PREINPUT supplemental forms={} editorial_senses={} captures={} families={}",
        complete.metadata().forms.len(),
        complete.metadata().editorial_senses.len(),
        complete.clone().into_document().captures.len(),
        one_expected.len()
    );
    let llg = seal_release(PackId::Llg, PackPayloadKind::Synthetic, &complete).unwrap();
    let sut = seal_release(PackId::Sutysisku, PackPayloadKind::Synthetic, sut).unwrap();
    let xs = vec![
        (PackId::Llg, llg.release, llg.seal),
        (PackId::Sutysisku, sut.release, sut.seal),
    ];
    let root = base.join("supplemental");
    let installed = CollectionCatalogue::install(&root, req(&xs)).unwrap();
    let first = installed.generation.unwrap();
    let before_hash = store_sha(&first);
    assert_eq!(store_inventory(&first), two_expected);
    let descriptor = CollectionCatalogue::open(&root)
        .unwrap()
        .reference_metadata(PackId::Llg, &sha(&xs[0].1))
        .unwrap();
    let subset =
        CollectionCatalogue::remove(&root, &[PackId::Sutysisku], installed.local_revision).unwrap();
    let subset_generation = subset.generation.unwrap();
    assert_eq!(store_inventory(&subset_generation), one_expected);
    {
        let store = DictionaryStore::open(&subset_generation.store_path).unwrap();
        let document = store.unverified_document().unwrap();
        assert_eq!(document.version, 1);
        assert_eq!(document.registry.version, 1);
        assert_eq!(
            document.metadata.catalogue_revision,
            complete.metadata().catalogue_revision
        );
        assert!(document.legacy_archive.is_none());
        assert_ne!(subset.local_revision, document.metadata.catalogue_revision);
    }
    assert_eq!(store_sha(&first), before_hash);
    let gone = CollectionCatalogue::remove(&root, &[PackId::Llg], subset.local_revision).unwrap();
    assert!(gone.generation.is_none());
    assert_eq!(
        CollectionCatalogue::open(&root)
            .unwrap()
            .reference_metadata(PackId::Llg, &sha(&xs[0].1))
            .unwrap(),
        descriptor
    );
    let back =
        CollectionCatalogue::reinstall(&root, req(&xs[..1]), &sha(&xs[0].1), gone.local_revision)
            .unwrap();
    assert_eq!(
        store_inventory(back.generation.as_ref().unwrap()),
        one_expected
    );
    assert_eq!(
        back.generation.as_ref().unwrap().content_id,
        subset_generation.content_id
    );
    assert_ne!(
        back.generation.as_ref().unwrap().generation_id,
        subset_generation.generation_id
    );
    assert_eq!(store_sha(&first), before_hash);
    assert_eq!(
        CollectionCatalogue::open(&root)
            .unwrap()
            .reference_metadata(PackId::Llg, &sha(&xs[0].1))
            .unwrap(),
        descriptor
    );
    println!(
        "PASS nonempty Form/editorial/link/capture25-family complete fixture through co-active subset removal, null, exact reinstall and exact retained descriptor/history bytes"
    );
}
fn alternate_file(base: &Path) {
    use std::io::Write;
    let original = base.join("relative-original.toml");
    let lock = base.join("relative-original.toml.lock");
    let alternate = base.join("relative-alternate.toml");
    let literal=b"version = 1\nrevision = 1\ndefault_profile = \"all\"\n[profiles.all]\nroot = \"policy-root\"\npacks = [\"sutysisku\"]\ndisabled = []\n";
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&original)
        .unwrap();
    file.write_all(literal).unwrap();
    file.sync_all().unwrap();
    drop(file);
    let prior = CollectionProfiles::read(&original).unwrap();
    assert_eq!(prior.profile(None).unwrap().root, base.join("policy-root"));
    let lockbytes =
        b"caller-owned preexisting lock transport fixture; not claimed process interruption";
    let mut locked = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&lock)
        .unwrap();
    locked.write_all(lockbytes).unwrap();
    locked.sync_all().unwrap();
    drop(locked);
    let request = || CollectionProfileEnablementRequest {
        profile: None,
        selection: CollectionPolicySelection::All,
        enabled: false,
    };
    assert!(
        matches!(CollectionProfiles::set_enabled(&original,request()),Err(vlajmaji_store::CollectionProfileError::Busy(p)) if p==lock)
    );
    let captured = std::fs::read(&original).unwrap();
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut alt = options.open(&alternate).unwrap();
    alt.write_all(&captured).unwrap();
    alt.sync_all().unwrap();
    drop(alt);
    std::fs::File::open(base).unwrap().sync_all().unwrap();
    assert_eq!(
        CollectionProfiles::read(&alternate)
            .unwrap()
            .profile(None)
            .unwrap()
            .root,
        base.join("policy-root")
    );
    assert!(
        std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&alternate)
            .is_err()
    );
    let changed = CollectionProfiles::set_enabled(&alternate, request()).unwrap();
    assert_eq!(changed.outcome, "committed");
    CollectionProfiles::set_enabled(
        &alternate,
        CollectionProfileEnablementRequest {
            profile: None,
            selection: CollectionPolicySelection::All,
            enabled: true,
        },
    )
    .unwrap();
    configured_counts(&alternate, 2, 1);
    assert!(
        matches!(CollectionProfiles::set_enabled(&original,request()),Err(vlajmaji_store::CollectionProfileError::Busy(p)) if p==lock)
    );
    assert_eq!(std::fs::read(&original).unwrap(), captured);
    assert_eq!(std::fs::read(&lock).unwrap(), lockbytes);
    println!(
        "PASS explicit same-parent exclusive ALT with relative ROOT, successful partial policy/lookup, occupied ALT refusal and unchanged still-Busy original; no process-crash claim"
    );
}
fn deleted_cwd_child(root: &Path, file: &Path, cwd: &Path) {
    assert!(root.is_absolute() && file.is_absolute() && cwd.is_absolute());
    std::env::set_current_dir(cwd).unwrap();
    std::fs::remove_dir(cwd).unwrap();
    assert!(std::env::current_dir().is_err());
    let xs = seals(&models());
    let prior = current(root).0;
    let observation = CollectionCatalogue::reconcile(root).unwrap();
    assert_eq!(observation.local_revision, prior);
    let removed = CollectionCatalogue::remove(root, &[PackId::Sutysisku], prior).unwrap();
    let restored = CollectionCatalogue::reinstall(
        root,
        req(&xs[3..4]),
        &sha(&xs[3].1),
        removed.local_revision,
    )
    .unwrap();
    assert!(restored.generation.is_some());
    CollectionProfiles::set_enabled(
        file,
        CollectionProfileEnablementRequest {
            profile: Some("all".into()),
            selection: CollectionPolicySelection::Packs(vec![PackId::Sutysisku]),
            enabled: true,
        },
    )
    .unwrap();
    CollectionProfiles::set_default(file, Some("all")).unwrap();
    println!(
        "PASS actual env-cleared deleted-cwd child with all-absolute remove/reinstall/reconcile/partial-policy/default paths"
    );
}
// Prospective new-operation native/direct-syscall control, unexecuted until handback.
#[cfg(unix)]
fn native_new_operations(base: &Path, xs: &Seals) {
    use std::os::unix::ffi::{OsStrExt, OsStringExt};
    fn hex(p: &Path) -> String {
        p.as_os_str()
            .as_bytes()
            .iter()
            .map(|x| format!("{x:02x}"))
            .collect()
    }
    fn io_signature(
        error: &(dyn std::error::Error + 'static),
    ) -> Option<(std::io::ErrorKind, Option<i32>)> {
        let mut at = Some(error);
        while let Some(e) = at {
            if let Some(io) = e.downcast_ref::<std::io::Error>() {
                return Some((io.kind(), io.raw_os_error()));
            }
            at = e.source();
        }
        None
    }
    let native = base.join(std::ffi::OsString::from_vec(b"phase2-root-\xff".to_vec()));
    let root = match std::fs::create_dir(&native) {
        Ok(()) => {
            std::fs::remove_dir(&native).unwrap();
            println!(
                "METHOD physical native-byte ROOT supported; actual phase2 install/remove/reinstall/reconcile follows"
            );
            native.clone()
        }
        Err(create) => {
            #[cfg(target_os = "linux")]
            panic!("Linux must support existing invalid-byte ROOT positive: {create:?}");
            #[cfg(not(target_os = "linux"))]
            {
                let direct = std::fs::symlink_metadata(&native).unwrap_err();
                println!(
                    "METHOD native ROOT setup create={create:?}, equivalent metadata={direct:?}"
                );
                for (op, error) in [
                    (
                        "remove",
                        CollectionCatalogue::remove(&native, &[PackId::Sutysisku], 0).unwrap_err(),
                    ),
                    (
                        "reinstall",
                        CollectionCatalogue::reinstall(&native, req(&xs[3..4]), &sha(&xs[3].1), 0)
                            .unwrap_err(),
                    ),
                    (
                        "reconcile",
                        CollectionCatalogue::reconcile(&native).unwrap_err(),
                    ),
                ] {
                    let report = error.report(op);
                    if direct.kind() == std::io::ErrorKind::NotFound {
                        assert_eq!(report.category, "missing_catalogue");
                    } else {
                        assert_eq!(
                            io_signature(&error),
                            Some((direct.kind(), direct.raw_os_error()))
                        );
                    }
                    let p = report.path.unwrap();
                    assert_eq!(p.hex, hex(&native));
                    assert!(p.utf8.is_none());
                }
                assert!(!PathBuf::from(native.to_string_lossy().into_owned()).exists());
                println!(
                    "PASS new remove/reinstall/reconcile native ROOT error fidelity at equivalent metadata syscall; no native physical success claim on this host"
                );
                base.join("phase2 native λ ;$ fallback")
            }
        }
    };
    let installed = CollectionCatalogue::install(&root, req(xs)).unwrap();
    let removed =
        CollectionCatalogue::remove(&root, &[PackId::Sutysisku], installed.local_revision).unwrap();
    let original = base.join(std::ffi::OsString::from_vec(
        b"phase2-release-\xfe.json".to_vec(),
    ));
    let seal = base.join("phase2-native-seal.json");
    std::fs::write(&seal, &xs[3].2).unwrap();
    let write = std::fs::write(&original, &xs[3].1);
    let direct = std::fs::read(&original);
    let before = db(&root);
    let request = CollectionRequest::new(
        vec![CollectionInput::SealedFiles {
            pack_id: PackId::Sutysisku,
            release: original.clone(),
            seal: seal.clone(),
        }],
        CollectionSelection::Packs(vec![PackId::Sutysisku]),
        true,
    )
    .unwrap();
    let outcome =
        CollectionCatalogue::reinstall(&root, request, &sha(&xs[3].1), removed.local_revision);
    match (write, direct, outcome) {
        (Ok(()), Ok(bytes), Ok(report)) => {
            assert_eq!(bytes, xs[3].1);
            assert_eq!(report.local_revision, removed.local_revision + 1);
            assert_eq!(std::fs::read(&original).unwrap(), xs[3].1);
            assert_ne!(db(&root), before);
            println!(
                "PASS actual existing invalid-byte sealed input through exact reinstall; ROOT native support qualified separately; external bytes unchanged"
            );
        }
        (Err(_), Err(direct), Err(error)) => {
            #[cfg(target_os = "linux")]
            panic!("Linux must force existing native sealed-input success: {direct:?}, {error:?}");
            #[cfg(not(target_os = "linux"))]
            {
                assert_eq!(
                    io_signature(&error),
                    Some((direct.kind(), direct.raw_os_error()))
                );
                let path = error.report("reinstall").path.unwrap();
                assert_eq!(path.hex, hex(&original));
                assert!(path.utf8.is_none());
                assert_eq!(db(&root), before);
                let good = CollectionCatalogue::reinstall(
                    &root,
                    req(&xs[3..4]),
                    &sha(&xs[3].1),
                    removed.local_revision,
                )
                .unwrap();
                assert_eq!(good.local_revision, removed.local_revision + 1);
                println!(
                    "PASS exact native input-read error/unchanged catalogue plus SAME original valid sealed-bytes restore; native creation unsupported here"
                );
            }
        }
        other => panic!("new native reinstall differs from equivalent read: {other:?}"),
    }
    assert!(!PathBuf::from(original.to_string_lossy().into_owned()).exists());
    let revision = current(&root).0;
    let snapshot = db(&root);
    let observed = CollectionCatalogue::reconcile(&root).unwrap();
    assert_eq!(observed.local_revision, revision);
    assert_eq!(db(&root), snapshot);
    counts(&root, IDS.to_vec(), 5, 3);
}

// Actual captured-v1 binary creates only a fresh caller-owned Tiny fixture.
// This is compatible-v1 reconcile/write-fence proof, NOT an interrupted-v1 migration claim.
fn reconcile_v1_compatibility(base: &Path) {
    let binary = Path::new("/tmp/vlajmaji-p01-a04-s03-cdc/cli-45f2-release01");
    assert_eq!(
        sha(&std::fs::read(binary).unwrap()),
        "4ecf51f1f8958097779a28a3fee97455e89fa5dcac149b1a977a04ecba05f244"
    );
    let root = base.join("compatible-v1-only");
    assert!(!root.exists());
    let output = std::process::Command::new(binary)
        .args([
            std::ffi::OsStr::new("collections"),
            std::ffi::OsStr::new("install"),
            root.as_os_str(),
            std::ffi::OsStr::new("--manifest"),
            base.join("manifest.json").as_os_str(),
            std::ffi::OsStr::new("--all"),
            std::ffi::OsStr::new("--allow-synthetic"),
        ])
        .env_clear()
        .output()
        .unwrap();
    std::fs::write(base.join("captured-v1-install.stdout"), &output.stdout).unwrap();
    std::fs::write(base.join("captured-v1-install.stderr"), &output.stderr).unwrap();
    assert!(
        output.status.success(),
        "captured v1 child status {:?}: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    let c = CollectionCatalogue::open(&root).unwrap();
    assert_eq!(c.schema_version().unwrap(), 1);
    let rev = c.local_revision().unwrap();
    let original = c.generation().unwrap().unwrap();
    drop(c);
    assert_eq!(rev, 1);
    let before = db(&root);
    let observed = CollectionCatalogue::reconcile(&root).unwrap();
    assert_eq!(observed.local_revision, rev);
    let c = CollectionCatalogue::open(&root).unwrap();
    assert_eq!(c.schema_version().unwrap(), 1);
    assert_eq!(
        c.generation().unwrap().unwrap().generation_id,
        original.generation_id
    );
    drop(c);
    assert_eq!(db(&root), before);
    let error = CollectionCatalogue::remove(&root, &[PackId::Llg], rev).unwrap_err();
    assert_eq!(error.report("remove").category, "migration_required");
    assert_eq!(db(&root), before);
    let migrated = CollectionCatalogue::migrate(&root, rev).unwrap();
    assert!(migrated.ok);
    let c = CollectionCatalogue::open(&root).unwrap();
    assert_eq!(c.schema_version().unwrap(), 2);
    assert_eq!(c.local_revision().unwrap(), rev + 1);
    assert_eq!(
        c.generation().unwrap().unwrap().generation_id,
        original.generation_id
    );
    drop(c);
    let after = db(&root);
    CollectionCatalogue::reconcile(&root).unwrap();
    assert_eq!(db(&root), after);
    counts(&root, IDS.to_vec(), 5, 3);
    println!(
        "PASS actual captured-v1 Tiny creation→reconcile no auto-migration/logical mutation→typed v1 write fence→EXPLICIT migrate valid counterpart; no v1 crash/journal claim"
    );
}

// Standalone Lensisku fixture is independently supplied, never converted into a seal.
fn legacy_store_fixture(base: &Path, dictionary: &Dictionary) {
    let xdg = base.join("isolated-legacy-data");
    let store = xdg.join("vlajmaji/stores/default");
    std::fs::create_dir_all(store.parent().unwrap()).unwrap();
    let created = DictionaryStore::create(&store, dictionary).unwrap();
    drop(created);
    let database = store.join("dictionary.sqlite3");
    std::fs::write(base.join("legacy-store.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "xdg_data_home":xdg,"store":store,"database_sha256":sha(&std::fs::read(database).unwrap()),
        "expected_farvi_occurrences":1,"expected_farvi_source_groups":1,
        "scope":"fresh original single-Lensisku standalone STORE; no sealed pack/provenance fabricated from it"
    })).unwrap()).unwrap();
    println!(
        "PREPARED original standalone Lensisku STORE for independent explicit/scratch-XDG CLI before/after controls; no real default"
    );
}

fn main() {
    if std::env::args_os().nth(1).as_deref() == Some(std::ffi::OsStr::new("__deleted-cwd")) {
        let root = PathBuf::from(std::env::args_os().nth(2).unwrap());
        let file = PathBuf::from(std::env::args_os().nth(3).unwrap());
        let cwd = PathBuf::from(std::env::args_os().nth(4).unwrap());
        deleted_cwd_child(&root, &file, &cwd);
        return;
    }
    let base = PathBuf::from(
        std::env::args_os()
            .nth(1)
            .expect("exclusive fresh caller scratch"),
    );
    std::fs::create_dir(&base).unwrap();
    let ds = models();
    qual(&ds);
    let changed_raw = std::str::from_utf8(SUT)
        .unwrap()
        .replace("synthetic one", "synthetic version two");
    assert_ne!(changed_raw.as_bytes(), SUT);
    let changed_sut = import_sutysisku_export(
        changed_raw.as_bytes(),
        &SutysiskuImportOptions {
            revision: "1".repeat(40),
            expected_raw_sha256: sha(changed_raw.as_bytes()),
            raw_locator: Some("https://example.invalid/sut".into()),
            notice_locator: Some("https://example.invalid/code-notice".into()),
        },
    )
    .unwrap()
    .into_dictionary();
    assert_eq!(changed_sut.metadata().assertions.len(), 3);
    assert_eq!(
        changed_sut
            .metadata()
            .assertions
            .iter()
            .filter(|a| a.eligibility == Eligibility::Lexical)
            .count(),
        2
    );
    assert_ne!(
        inventory(&draft(ds[3].clone())),
        inventory(&draft(changed_sut.clone()))
    );
    let xs = seals(&ds);
    legacy_store_fixture(&base, &ds[0]);
    let pre = expected(&ds);
    std::fs::write(
        base.join("pre-input-expected.json"),
        serde_json::to_vec_pretty(&pre).unwrap(),
    )
    .unwrap();
    let inputs = base.join("inputs");
    std::fs::create_dir(&inputs).unwrap();
    let mut manifest = Vec::new();
    for (id, r, seal) in &xs {
        let d = inputs.join(id.as_str());
        std::fs::create_dir(&d).unwrap();
        std::fs::write(d.join("release.json"), r).unwrap();
        std::fs::write(d.join("seal.json"), seal).unwrap();
        manifest.push(serde_json::json!({"kind":"sealed","pack_id":id.as_str(),"release":format!("inputs/{}/release.json",id.as_str()),"seal":format!("inputs/{}/seal.json",id.as_str())}));
    }
    std::fs::write(
        base.join("manifest.json"),
        serde_json::to_vec_pretty(&serde_json::json!({"version":1,"inputs":manifest})).unwrap(),
    )
    .unwrap();
    reconcile_v1_compatibility(&base);
    lifecycle(&base, &ds, &xs, &changed_sut);
    registered_only(&base, &xs);
    explicit_restore_missing_copy(&base, &xs, &ds);
    supplemental_preservation(&base, &ds[3]);
    policy(&base, &xs);
    alternate_file(&base);
    #[cfg(unix)]
    native_new_operations(&base, &xs);
    let cwd = base.join("deleted-cwd");
    std::fs::create_dir(&cwd).unwrap();
    let child = std::process::Command::new(std::env::current_exe().unwrap())
        .arg("__deleted-cwd")
        .arg(base.join("policy-root"))
        .arg(base.join("profiles.toml"))
        .arg(&cwd)
        .env_clear()
        .output()
        .unwrap();
    std::fs::write(base.join("deleted-cwd.stdout"), &child.stdout).unwrap();
    std::fs::write(base.join("deleted-cwd.stderr"), &child.stderr).unwrap();
    assert!(
        child.status.success(),
        "{}",
        String::from_utf8_lossy(&child.stderr)
    );
    println!("{}", String::from_utf8_lossy(&child.stdout));
    println!(
        "{}",
        serde_json::json!({"ok":true,"scope":"independent phase2 tiny lifecycle/policy caller; no Full or process interruption proof","base":base})
    );
}
