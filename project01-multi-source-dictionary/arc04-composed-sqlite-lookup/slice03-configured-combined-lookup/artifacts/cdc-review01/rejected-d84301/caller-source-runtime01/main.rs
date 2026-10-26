// Independent S03 caller source. Prepared during author ownership; not executed
// until candidate freeze and an explicit transfer of the retained target.
include!("helpers.rs");
mod profile_checks;
mod query_checks;
mod reference_checks;
mod shared_body_checks;
use std::{borrow::Cow, path::PathBuf};
use vlajmaji_store::{CollectionCatalogue, CollectionInput, CollectionRequest, CollectionSelection};

fn main() {
    // Concrete public signatures must remain nameable outside the workspace.
    let _: Option<(vlajmaji_store::CollectionProfile,vlajmaji_store::CollectionProfileIdentity,vlajmaji_store::CollectionProfilesReport,vlajmaji_store::CollectionProfileWriteReport,vlajmaji_store::CollectionProfileWriteRequest,vlajmaji_store::CollectionProfileMembershipWriteRequest,vlajmaji_store::CollectionProfileError,vlajmaji_store::CollectionInstalledArtifactLocations)> = None;
    let _: Option<(vlajmaji_search::CollectionQuerySelection,vlajmaji_search::CollectionSelectionSnapshot,vlajmaji_search::CollectionQueryResponse,vlajmaji_search::CollectionQueryResult,vlajmaji_search::CollectionReferenceResponse,vlajmaji_search::CollectionQueryError)> = None;
    if let Some(mode) = std::env::args_os().nth(1) {
        if mode == "--shared-body" {
            shared_body_checks::run(&PathBuf::from(std::env::args_os().nth(2).unwrap()));
            return;
        }
        if mode == "--absolute-without-cwd" || mode == "--relative-without-cwd" {
            // Only this isolated subprocess changes cwd. No global process
            // environment is mutated, and the scratch directory is exclusive.
            let file = PathBuf::from(std::env::args_os().nth(2).unwrap());
            let cwd = PathBuf::from(std::env::args_os().nth(3).unwrap());
            std::fs::create_dir(&cwd).unwrap();
            std::env::set_current_dir(&cwd).unwrap();
            std::fs::remove_dir(&cwd).unwrap();
            let direct_cwd=std::env::current_dir().unwrap_err();
            if mode == "--absolute-without-cwd" {
                let profiles = vlajmaji_store::CollectionProfiles::read(&file).unwrap();
                let profile = profiles.profile(None).unwrap();
                let request = vlajmaji_store::CollectionProfileWriteRequest { name:profile.name, root:profile.root, packs:profile.packs, disabled:profile.disabled, make_default:true };
                assert_eq!(vlajmaji_store::CollectionProfiles::configure(&file, request).unwrap().outcome, "unchanged");
                println!("PASS absolute library read/configure needs neither cwd nor HOME");
            } else {
                let error = vlajmaji_store::CollectionProfiles::read("relative.toml").unwrap_err();
                assert!(matches!(&error,vlajmaji_store::CollectionProfileError::CurrentDirectory{path,source}
                    if path==std::path::Path::new("relative.toml") && source.kind()==direct_cwd.kind()
                    && source.raw_os_error()==direct_cwd.raw_os_error()));
                let mut current: &(dyn std::error::Error + 'static) = &error;
                let mut io = false;
                loop {
                    if current.downcast_ref::<std::io::Error>().is_some() { io = true; }
                    match current.source() { Some(next) => current = next, None => break }
                }
                assert!(io);
                let report=vlajmaji_search::CollectionQueryError::from(error).report_for("profiles");
                assert_eq!(report.category,"config_io");
                println!("PASS relative library path preserves typed missing-cwd IO");
            }
            return;
        }
    }
    let base = PathBuf::from(std::env::args_os().nth(1).expect("exclusive fresh scratch root"));
    std::fs::create_dir(&base).expect("never reuse or pre-delete an attempt");
    let models = models();
    let packs = [PackId::LensiskuJbotci, PackId::Llg, PackId::LaLojban, PackId::Sutysisku];
    let reference_witnesses=reference_checks::freeze(&models,&packs);
    std::fs::write(base.join("pre-install-reference-witnesses.json"),serde_json::to_vec_pretty(&reference_witnesses).unwrap()).unwrap();
    // Freeze source-literal identities and a complete 25-family multiset before
    // any catalogue operation. This is the unchanged accepted S02 oracle route.
    let drafts: Vec<_> = models.iter().cloned().map(draft).collect();
    let mut expected: BTreeMap<&str, BTreeMap<String, usize>> = BTreeMap::new();
    for model in &drafts {
        for (category, rows) in inventory(model) {
            let category_rows = expected.entry(category).or_default();
            for (identity, count) in rows {
                let current = category_rows.entry(identity).or_default();
                *current = (*current).max(count);
            }
        }
    }
    let composed = compose_dictionaries(models.clone()).unwrap();
    let composed_inventory = inventory(&draft(composed.clone()));
    assert_eq!(composed_inventory, expected);
    assert_eq!(expected.len(), 25);
    assert!(expected["frames"].is_empty());
    let mut changed = expected.clone();
    changed.remove("frames");
    assert_ne!(expected, changed);
    let mut changed = expected.clone();
    changed.insert("unexpected_empty", BTreeMap::new());
    assert_ne!(expected, changed);
    let mut changed = expected.clone();
    *changed.get_mut("assertions").unwrap().values_mut().next().unwrap() += 1;
    assert_ne!(expected, changed);
    let mut changed = drafts[0].clone();
    changed.metadata.entries[0].spelling = Presence::Missing;
    assert_ne!(inventory(&drafts[0]), inventory(&changed));
    assert_eq!(composed.metadata().assertions.len(), 6);
    assert_eq!(composed.metadata().assertions.iter().filter(|assertion| assertion.eligibility == Eligibility::Lexical).count(), 5);
    let expected_assertions: Vec<_> = composed.metadata().assertions.iter().map(|assertion| assertion.id.to_string()).collect();
    let expected_source_totals = BTreeMap::from([("lensisku", 2), ("llg", 1), ("sutysisku", 2)]);
    std::fs::write(base.join("pre-install-expected.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "assertions": expected_assertions,
        "farvi_source_totals": expected_source_totals,
        "categories": expected,
        "canonical_sha256": sha(&composed.to_json_bytes().unwrap()),
    })).unwrap()).unwrap();
    println!("PASS pre-install complete inventory and four discriminating comparator negatives");

    let seals: Vec<_> = packs.iter().zip(&models).map(|(pack, dictionary)| seal_release(*pack, PackPayloadKind::Synthetic, dictionary).unwrap()).collect();
    let root = base.join("R");
    let request = CollectionRequest::new(packs.iter().zip(&seals).map(|(pack, sealed)| CollectionInput::SealedBytes {
        pack_id: *pack, release: Cow::Borrowed(&sealed.release), seal: Cow::Borrowed(&sealed.seal),
    }).collect(), CollectionSelection::Packs(packs.to_vec()), true).unwrap();
    CollectionCatalogue::install(&root, request).unwrap();
    let catalogue = CollectionCatalogue::open(&root).unwrap();
    let generation = catalogue.generation().unwrap().unwrap();
    drop(catalogue);
    std::fs::write(base.join("post-install-generation.json"),serde_json::to_vec_pretty(&generation).unwrap()).unwrap();
    let store = vlajmaji_store::DictionaryStore::open(&generation.store_path).unwrap();
    assert_eq!(inventory(&store.unverified_document().unwrap()), composed_inventory);
    drop(store);
    println!("PASS tiny installation agrees with independently frozen complete body");
    profile_checks::run(&base, &root, &packs);
    for (mode, name) in [("--absolute-without-cwd", "absolute"), ("--relative-without-cwd", "relative")] {
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .arg(mode).arg(base.join("profiles.toml")).arg(base.join(format!("deleted-cwd-{name}")))
            .env_clear().output().unwrap();
        std::fs::write(base.join(format!("deleted-cwd-{name}.stdout")), &output.stdout).unwrap();
        std::fs::write(base.join(format!("deleted-cwd-{name}.stderr")), &output.stderr).unwrap();
        assert!(output.status.success(), "{name}: {}", String::from_utf8_lossy(&output.stderr));
        println!("PASS isolated {name} missing-cwd subprocess");
    }
    query_checks::run(&base,&root,&packs);
    let pairs:Vec<_>=seals.iter().map(|sealed|(sealed.release.clone(),sealed.seal.clone())).collect();
    reference_checks::run(&base,&root,&packs,&pairs,&reference_witnesses);
    println!("{}", serde_json::json!({"ok":true,"scope":"initial independent profile/exact/cursor/reference controls; CLI and remaining state-fault coverage are separate explicit stages","base":base}));
}
