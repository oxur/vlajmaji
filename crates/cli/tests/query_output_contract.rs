//! Compact query and detail CLI routes preserve full defaults and strict keys.

#[path = "../../lexicon/examples/support/dictionary_fixture.rs"]
mod fixture;

use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use vlajmaji_lexicon::{
    CatalogTarget, Dictionary, Eligibility, JbotciImportOptions, LaLojbanImportOptions,
    LlgImportOptions, LocalRef, PackId, PackPayloadKind, Presence, RelationTarget,
    SutysiskuImportOptions, import_jbotci_export, import_la_lojban_export, import_llg_lists,
    import_sutysisku_export, load_sealed_release, seal_release,
};
use vlajmaji_search::DictionaryRequest;
use vlajmaji_store::{
    CollectionCatalogue, CollectionInput, CollectionRequest, CollectionSelection, DictionaryStore,
};

static NEXT: AtomicUsize = AtomicUsize::new(0);

fn scratch() -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "vlajmaji-s05-query-output-{}-{stamp}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir(&root).unwrap();
    root
}

fn run(args: &[&Path]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
        .args(args)
        .env_clear()
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .unwrap()
}

fn run_strings(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
        .args(args)
        .env_clear()
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .unwrap()
}

fn run_isolated(args: &[&str], home: &Path) -> Output {
    let xdg = home.join("xdg");
    std::fs::create_dir_all(xdg.join("config")).unwrap();
    std::fs::create_dir_all(xdg.join("data")).unwrap();
    std::fs::create_dir_all(xdg.join("state")).unwrap();
    Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
        .args(args)
        .env_clear()
        .env("HOME", home)
        .env("XDG_CONFIG_HOME", xdg.join("config"))
        .env("XDG_DATA_HOME", xdg.join("data"))
        .env("XDG_STATE_HOME", xdg.join("state"))
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .unwrap()
}

const TINY_LENS: &[u8] = br#"[{"word":"farvi","word_type":"gismu","definition":"synthetic develops","definition_id":333,"score":1.2500,"user":{"username":"shared"}}]"#;
const TINY_SUT: &[u8] = br#"{"farvi":{"d":"synthetic one","t":"gismu","n":null},"farvi":{"d":"synthetic two","t":"gismu"},"zoizoizoi":{"d":"retained excluded","t":"unsupported"}}"#;
const TINY_LLG_HEX: &str = "206661727669202020202020202020202020202067726f772020202020202020202020202020202020202020202020202020202020202020202020202020642020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020";
const TINY_NOTICE: &[u8] = b"Explicit synthetic notice";

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn tiny_llg() -> Vec<u8> {
    TINY_LLG_HEX
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| {
            let digit = |byte: u8| match byte {
                b'0'..=b'9' => byte - b'0',
                b'a'..=b'f' => byte - b'a' + 10,
                _ => panic!("invalid fixture hex"),
            };
            (digit(pair[0]) << 4) | digit(pair[1])
        })
        .collect()
}

fn tiny_models() -> Vec<Dictionary> {
    let revision = "0".repeat(40);
    let metadata = format!(
        "language_tag = \"en\"\nlanguage_realname = \"English\"\nsource_language_tag = \"jbo\"\nformat = \"json\"\npositive_scores_only = false\nfilename = \"dictionary-en.json\"\nmetadata_url = \"https://example.invalid/languages\"\ndownload_url = \"https://example.invalid/synthetic\"\nlensisku_created_at = \"2026-10-04T03:07:08Z\"\nsha256 = \"{}\"\ndefinition_count = 1\nentry_count = 1\n",
        sha256(TINY_LENS)
    );
    let jbotci = import_jbotci_export(
        TINY_LENS,
        metadata.as_bytes(),
        &JbotciImportOptions {
            revision: revision.clone(),
            expected_raw_sha256: sha256(TINY_LENS),
            expected_metadata_sha256: sha256(metadata.as_bytes()),
        },
    )
    .unwrap()
    .into_dictionary();
    let la = import_la_lojban_export(
        TINY_LENS,
        &LaLojbanImportOptions {
            revision: revision.clone(),
            expected_raw_sha256: sha256(TINY_LENS),
            raw_locator: Some("https://example.invalid/la".into()),
            notice_locator: Some("https://example.invalid/code-notice".into()),
        },
    )
    .unwrap()
    .into_dictionary();
    let llg = tiny_llg();
    let llg = import_llg_lists(
        &llg,
        b"",
        b"",
        TINY_NOTICE,
        &LlgImportOptions {
            expected_gismu_sha256: sha256(&llg),
            expected_cmavo_sha256: sha256(b""),
            expected_rafsi_sha256: sha256(b""),
            expected_notice_sha256: sha256(TINY_NOTICE),
            base_locator: "https://example.invalid/llg".into(),
        },
    )
    .unwrap()
    .into_dictionary();
    let sut = import_sutysisku_export(
        TINY_SUT,
        &SutysiskuImportOptions {
            revision,
            expected_raw_sha256: sha256(TINY_SUT),
            raw_locator: Some("https://example.invalid/sut".into()),
            notice_locator: Some("https://example.invalid/code-notice".into()),
        },
    )
    .unwrap()
    .into_dictionary();
    vec![jbotci, llg, la, sut]
}

type TypedInventory = BTreeMap<String, BTreeMap<String, usize>>;

macro_rules! typed_rows {
    ($rows:expr) => {{
        let mut inventory = BTreeMap::new();
        for row in $rows {
            *inventory
                .entry(sha256(&serde_json::to_vec(row).unwrap()))
                .or_insert(0) += 1;
        }
        inventory
    }};
}

fn typed_inventory(dictionary: &Dictionary) -> TypedInventory {
    let document = dictionary.clone().into_document();
    let metadata = document.metadata;
    let registry = document.registry.into_document();
    let mut inventory = BTreeMap::new();
    macro_rules! add_metadata {
        ($($family:ident),+ $(,)?) => {$({
            inventory.insert(stringify!($family).to_owned(), typed_rows!(&metadata.$family));
        })+};
    }
    add_metadata!(
        collections,
        components,
        releases,
        code_notices,
        entries,
        assertions,
        contexts,
        forms,
        etymologies,
        frames,
        relations,
        owned_revisions,
        owned_heads,
        editorial_senses,
        computations,
        annotations,
        extensions,
    );
    macro_rules! add_registry {
        ($($family:ident),+ $(,)?) => {$({
            inventory.insert(stringify!($family).to_owned(), typed_rows!(&registry.$family));
        })+};
    }
    add_registry!(
        sources,
        contributors,
        distributors,
        aliases,
        lineage,
        snapshots,
        occurrences
    );
    inventory.insert("captures".to_owned(), typed_rows!(&document.captures));
    inventory
}

fn max_input_inventory(models: &[Dictionary]) -> TypedInventory {
    let mut expected = BTreeMap::new();
    for family in [
        "collections",
        "components",
        "releases",
        "code_notices",
        "entries",
        "assertions",
        "contexts",
        "forms",
        "etymologies",
        "frames",
        "relations",
        "owned_revisions",
        "owned_heads",
        "editorial_senses",
        "computations",
        "annotations",
        "extensions",
        "sources",
        "contributors",
        "distributors",
        "aliases",
        "lineage",
        "snapshots",
        "occurrences",
        "captures",
    ] {
        expected.insert(family.to_owned(), BTreeMap::new());
    }
    for model in models {
        for (family, rows) in typed_inventory(model) {
            let expected_rows = expected.get_mut(&family).unwrap();
            for (digest, count) in rows {
                let maximum = expected_rows.entry(digest).or_insert(0);
                *maximum = (*maximum).max(count);
            }
        }
    }
    expected
}

fn write_tiny_manifest(base: &Path, models: &[Dictionary]) -> (PathBuf, PathBuf, PathBuf, String) {
    assert_eq!(
        sha256(TINY_LENS),
        "63fb2e10e80f34e6d7f04fbe9bcfe1cd69c7ec0c4016045d02e8b3cfcd866429"
    );
    assert_eq!(
        sha256(&tiny_llg()),
        "0408fd8bf1adae9d0da1e7faabb35a6c2cdf64d012140bcadfdd814f061b2735"
    );
    assert_eq!(
        sha256(TINY_SUT),
        "3e7738c9412a4dfb6169e8492f8ecb7fd25003d2141dbb8fa11782e580d00b03"
    );
    let mut inputs = Vec::new();
    let mut llg_release_sha = None;
    let mut llg_release_path = None;
    let mut llg_seal_path = None;
    for (index, (pack, model)) in [
        (PackId::LensiskuJbotci, &models[0]),
        (PackId::Llg, &models[1]),
        (PackId::LaLojban, &models[2]),
        (PackId::Sutysisku, &models[3]),
    ]
    .into_iter()
    .enumerate()
    {
        let sealed = seal_release(pack, PackPayloadKind::Synthetic, model).unwrap();
        let release_path = base.join(format!("pack-{index}-release.json"));
        let seal_path = base.join(format!("pack-{index}-seal.json"));
        let release_sha = sha256(&sealed.release);
        assert_eq!(
            typed_inventory(&load_sealed_release(pack, &sealed.release, &sealed.seal).unwrap()),
            typed_inventory(model),
            "sealed pre-input model retains every typed family"
        );
        if pack == PackId::Llg {
            llg_release_sha = Some(release_sha);
            llg_release_path = Some(release_path.clone());
            llg_seal_path = Some(seal_path.clone());
        }
        std::fs::write(&release_path, &sealed.release).unwrap();
        std::fs::write(&seal_path, &sealed.seal).unwrap();
        inputs.push(json!({
            "kind": "sealed", "pack_id": pack.as_str(),
            "release": release_path, "seal": seal_path,
        }));
    }
    let manifest = json!({
        "version": 1,
        "inputs": inputs,
    });
    let path = base.join("manifest.json");
    std::fs::write(&path, serde_json::to_vec(&manifest).unwrap()).unwrap();
    (
        path,
        llg_release_path.unwrap(),
        llg_seal_path.unwrap(),
        llg_release_sha.unwrap(),
    )
}

#[test]
fn test_readable_cli_tiny_lifecycle_preserves_four_pack_state_and_projection() {
    let base = scratch();
    let root = base.join("catalogue");
    let config = base.join("config.toml");
    let models = tiny_models();
    let expected_inventory = max_input_inventory(&models);
    assert_eq!(
        expected_inventory.len(),
        25,
        "all typed families are accounted for"
    );
    assert_eq!(expected_inventory["assertions"].values().sum::<usize>(), 6);
    assert_eq!(expected_inventory["sources"].values().sum::<usize>(), 3);
    let mut lexical_farvi = BTreeMap::new();
    for model in &models {
        let document = model.clone().into_document();
        for assertion in &document.metadata.assertions {
            let is_lexical_farvi = assertion.eligibility == Eligibility::Lexical
                && document.metadata.entries.iter().any(|entry| {
                    entry.id == assertion.entry && entry.spelling == Presence::Value("farvi".into())
                });
            lexical_farvi.insert(
                sha256(&serde_json::to_vec(assertion).unwrap()),
                is_lexical_farvi,
            );
        }
    }
    assert_eq!(
        expected_inventory["assertions"]
            .iter()
            .filter(|(digest, _)| lexical_farvi.get(*digest) == Some(&true))
            .map(|(_, count)| count)
            .sum::<usize>(),
        5,
        "pre-input max-union lexical farvi assertions"
    );
    let manifest_and_release = write_tiny_manifest(&base, &models);
    let manifest = manifest_and_release.0;
    let llg_release_path = manifest_and_release.1;
    let llg_seal_path = manifest_and_release.2;
    let llg_release_sha = manifest_and_release.3;
    let llg = &models[1];
    let llg_entry = llg
        .metadata()
        .entries
        .iter()
        .find(|entry| entry.spelling == Presence::Value("farvi".into()))
        .expect("LLG farvi entry");
    let llg_assertion = llg
        .metadata()
        .assertions
        .iter()
        .find(|assertion| {
            assertion.entry == llg_entry.id && assertion.eligibility == Eligibility::Lexical
        })
        .expect("LLG lexical farvi assertion");
    let local_target = RelationTarget::Local(LocalRef::Assertion(llg_assertion.id.clone()));
    let llg_source = llg_entry.source.clone();
    let llg_release = llg.metadata().releases[0].id.clone();
    let catalog_target = RelationTarget::Catalog(CatalogTarget {
        source: llg_source,
        release: llg_release,
        native_entry_id: None,
        native_assertion_id: None,
        occurrence: Some(llg_assertion.occurrence.clone()),
    });

    let install = run_isolated(
        &[
            "collections",
            "install",
            root.to_str().unwrap(),
            "--manifest",
            manifest.to_str().unwrap(),
            "--all",
            "--allow-synthetic",
        ],
        &base,
    );
    assert!(
        install.status.success(),
        "{}",
        String::from_utf8_lossy(&install.stderr)
    );
    let installed: Value = serde_json::from_slice(&install.stdout).unwrap();
    assert_eq!(installed["local_revision"], 1);
    assert_eq!(
        installed["generation"]["members"].as_array().unwrap().len(),
        4
    );
    let installed_store = PathBuf::from(
        installed["generation"]["path"]["utf8"]
            .as_str()
            .expect("JSON-safe generation.path"),
    );
    let installed_model = DictionaryStore::open(&installed_store)
        .unwrap()
        .dictionary()
        .unwrap();
    assert_eq!(
        typed_inventory(&installed_model),
        expected_inventory,
        "the installed model equals the independent pre-input max multiset"
    );
    let old_generation_sha = sha256(
        &DictionaryStore::open(&installed_store)
            .unwrap()
            .canonical_bytes()
            .unwrap(),
    );
    assert_eq!(
        installed["inputs"]
            .as_array()
            .unwrap()
            .iter()
            .find(|input| input["pack_id"] == "llg")
            .and_then(|input| input["release_sha256"].as_str())
            .expect("registered LLG release digest"),
        llg_release_sha,
        "candidate registered the pre-input sealed LLG release"
    );
    let installed_release_sha = llg_release_sha.clone();

    let configured = run_isolated(
        &[
            "collections",
            "configure",
            root.to_str().unwrap(),
            "--profile",
            "four",
            "--all",
            "--make-default",
            "--config",
            config.to_str().unwrap(),
        ],
        &base,
    );
    assert!(
        configured.status.success(),
        "{}",
        String::from_utf8_lossy(&configured.stderr)
    );
    let configured: Value = serde_json::from_slice(&configured.stdout).unwrap();
    assert_eq!(configured["revision"], 1);

    let profiles = run_isolated(
        &[
            "collections",
            "profiles",
            "--config",
            config.to_str().unwrap(),
        ],
        &base,
    );
    assert!(profiles.status.success());
    assert_eq!(
        serde_json::from_slice::<Value>(&profiles.stdout).unwrap()["revision"],
        1
    );
    let selection = run_isolated(
        &[
            "collections",
            "selection",
            "--config",
            config.to_str().unwrap(),
            "--profile",
            "four",
        ],
        &base,
    );
    assert!(selection.status.success());
    let selected: Value = serde_json::from_slice(&selection.stdout).unwrap();
    assert_eq!(selected["packs"].as_array().unwrap().len(), 4);
    let inspect = run_isolated(
        &["collections", "inspect", root.to_str().unwrap(), "llg"],
        &base,
    );
    assert!(inspect.status.success());
    let inspect: Value = serde_json::from_slice(&inspect.stdout).unwrap();
    assert_eq!(inspect["pack_id"], "llg");
    assert_eq!(inspect["installation_state"], "installed");
    let initial_list = run_isolated(&["collections", "list", root.to_str().unwrap()], &base);
    assert!(initial_list.status.success());
    let initial_list: Value = serde_json::from_slice(&initial_list.stdout).unwrap();
    assert_eq!(initial_list["catalogue_schema_version"], 2);
    assert_eq!(initial_list["local_revision"], 1);

    let lookup = |format: Option<&str>, request: Option<&str>| {
        let mut args = vec![
            "lookup",
            "farvi",
            "--config",
            config.to_str().unwrap(),
            "--profile",
            "four",
        ];
        if let Some(request) = request {
            args = vec![
                "lookup",
                "--request",
                request,
                "--config",
                config.to_str().unwrap(),
                "--profile",
                "four",
            ];
        }
        if let Some(format) = format {
            args.extend(["--format", format]);
        }
        run_isolated(&args, &base)
    };
    let full = lookup(None, None);
    assert!(
        full.status.success(),
        "{}",
        String::from_utf8_lossy(&full.stderr)
    );
    let full_value: Value = serde_json::from_slice(&full.stdout).unwrap();
    assert_eq!(full_value["snapshot"]["local_revision"], 1);
    assert_eq!(full_value["result"]["response"]["totals"]["occurrences"], 5);
    assert_eq!(
        full_value["result"]["response"]["source_totals"]
            .as_array()
            .unwrap()
            .len(),
        3
    );
    let explicit_profile_full = lookup(Some("full"), None);
    assert!(explicit_profile_full.status.success());
    assert_eq!(explicit_profile_full.stdout, full.stdout);
    let default_full = run_isolated(
        &["lookup", "farvi", "--config", config.to_str().unwrap()],
        &base,
    );
    let default_full_explicit = run_isolated(
        &[
            "lookup",
            "farvi",
            "--config",
            config.to_str().unwrap(),
            "--format",
            "full",
        ],
        &base,
    );
    assert!(default_full.status.success());
    assert_eq!(default_full.stdout, default_full_explicit.stdout);

    let compact = lookup(Some("compact"), None);
    assert!(
        compact.status.success(),
        "{}",
        String::from_utf8_lossy(&compact.stderr)
    );
    let compact_value: Value = serde_json::from_slice(&compact.stdout).unwrap();
    let compact_response = &compact_value["result"]["response"];
    assert_eq!(compact_response["totals"]["occurrences"], 5);
    assert_eq!(
        compact_response["source_totals"],
        full_value["result"]["response"]["source_totals"]
    );
    let readable = lookup(Some("readable"), None);
    assert!(
        readable.status.success(),
        "{}",
        String::from_utf8_lossy(&readable.stderr)
    );
    let readable = String::from_utf8(readable.stdout).unwrap();
    assert!(readable.starts_with("Exact \"farvi\": 5 occurrences"));
    assert!(readable.ends_with("next_cursor: null\n"));

    for target in [local_target.clone(), catalog_target.clone()] {
        let request = serde_json::to_string(&DictionaryRequest::Resolve { target }).unwrap();
        let resolved = lookup(Some("compact"), Some(&request));
        assert!(
            resolved.status.success(),
            "{}",
            String::from_utf8_lossy(&resolved.stderr)
        );
        let resolved: Value = serde_json::from_slice(&resolved.stdout).unwrap();
        let response = &resolved["result"]["response"];
        assert_eq!(response["status"], "Resolved");
        assert_eq!(
            response["payload"]["confirmed"].as_array().unwrap().len(),
            1
        );
        assert_eq!(
            response["payload"]["unverified"].as_array().unwrap().len(),
            0
        );
    }
    let local_request = serde_json::to_string(&DictionaryRequest::Resolve {
        target: local_target.clone(),
    })
    .unwrap();
    let readable_reference = lookup(Some("readable"), Some(&local_request));
    assert!(
        readable_reference.status.success(),
        "{}",
        String::from_utf8_lossy(&readable_reference.stderr)
    );
    let readable_reference = String::from_utf8(readable_reference.stdout).unwrap();
    assert!(readable_reference.starts_with("Reference {\"Local\":{\"Assertion\":\""));
    assert!(readable_reference.contains(": Resolved\n"));

    let resolved_local = lookup(Some("compact"), Some(&local_request));
    let resolved_local: Value = serde_json::from_slice(&resolved_local.stdout).unwrap();
    let detail_ref =
        resolved_local["result"]["response"]["payload"]["confirmed"][0]["detail_ref"].clone();
    let catalog_request = serde_json::to_string(&DictionaryRequest::Resolve {
        target: catalog_target.clone(),
    })
    .unwrap();
    let resolved_catalog = lookup(Some("compact"), Some(&catalog_request));
    assert!(resolved_catalog.status.success());
    let resolved_catalog: Value = serde_json::from_slice(&resolved_catalog.stdout).unwrap();
    let catalog_detail_ref =
        resolved_catalog["result"]["response"]["payload"]["confirmed"][0]["detail_ref"].clone();
    let readable_catalog = lookup(Some("readable"), Some(&catalog_request));
    assert!(readable_catalog.status.success());
    let readable_catalog = String::from_utf8(readable_catalog.stdout).unwrap();
    assert!(readable_catalog.starts_with("Reference {\"Catalog\":"));
    assert!(readable_catalog.contains(": Resolved\n"));
    assert_eq!(detail_ref["source"], llg_entry.source.as_str());
    assert_eq!(detail_ref["entry_id"], llg_entry.id.as_str());
    assert_eq!(detail_ref["assertion_id"], llg_assertion.id.as_str());
    assert_eq!(
        detail_ref["occurrence"],
        serde_json::to_value(&llg_assertion.occurrence).unwrap()
    );
    assert_eq!(
        detail_ref["entry_sha256"],
        sha256(&serde_json::to_vec(llg_entry).unwrap()),
        "detail key binds the pre-input Entry body"
    );
    assert_eq!(
        detail_ref["assertion_sha256"],
        sha256(&serde_json::to_vec(llg_assertion).unwrap()),
        "detail key binds the pre-input Assertion body"
    );
    assert_eq!(detail_ref["native_entry_id"], "farvi");
    assert_eq!(
        catalog_detail_ref, detail_ref,
        "Local and Catalog compact hits carry the same pre-input K_LLG"
    );
    let snapshot = resolved_local["snapshot"]["binding_sha256"]
        .as_str()
        .unwrap();
    let detail = run_isolated(
        &[
            "lookup-detail",
            "--reference",
            &detail_ref.to_string(),
            "--expected-snapshot",
            snapshot,
            "--config",
            config.to_str().unwrap(),
            "--profile",
            "four",
        ],
        &base,
    );
    assert!(
        detail.status.success(),
        "{}",
        String::from_utf8_lossy(&detail.stderr)
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&detail.stdout).unwrap()["response"]["status"],
        "Resolved"
    );
    let readable_detail = run_isolated(
        &[
            "lookup-detail",
            "--reference",
            &detail_ref.to_string(),
            "--expected-snapshot",
            snapshot,
            "--config",
            config.to_str().unwrap(),
            "--profile",
            "four",
            "--format",
            "readable",
        ],
        &base,
    );
    assert!(
        readable_detail.status.success(),
        "{}",
        String::from_utf8_lossy(&readable_detail.stderr)
    );
    let readable_detail = String::from_utf8(readable_detail.stdout).unwrap();
    let detail_assertion = detail_ref["assertion_id"].as_str().unwrap();
    assert!(
        readable_detail.starts_with(&format!("Detail {detail_assertion}:")),
        "{readable_detail:?}"
    );
    assert!(readable_detail.contains(": Resolved\n"));
    let readable_catalog_detail = run_isolated(
        &[
            "lookup-detail",
            "--reference",
            &catalog_detail_ref.to_string(),
            "--expected-snapshot",
            snapshot,
            "--config",
            config.to_str().unwrap(),
            "--profile",
            "four",
            "--format",
            "readable",
        ],
        &base,
    );
    assert!(
        readable_catalog_detail.status.success(),
        "{}",
        String::from_utf8_lossy(&readable_catalog_detail.stderr)
    );
    let readable_catalog_detail = String::from_utf8(readable_catalog_detail.stdout).unwrap();
    assert!(readable_catalog_detail.starts_with("Detail "));
    assert!(readable_catalog_detail.contains(": Resolved\n"));

    let explicit_default = run_isolated(
        &[
            "dictionary-query",
            installed_store.to_str().unwrap(),
            "--request",
            r#"{"operation":"exact","spelling":"farvi"}"#,
        ],
        &base,
    );
    assert!(explicit_default.status.success());
    let explicit_full = run_isolated(
        &[
            "dictionary-query",
            installed_store.to_str().unwrap(),
            "--request",
            r#"{"operation":"exact","spelling":"farvi"}"#,
            "--format",
            "full",
        ],
        &base,
    );
    assert_eq!(explicit_default.stdout, explicit_full.stdout);

    let first_page = run_isolated(
        &[
            "lookup",
            "farvi",
            "--page-size",
            "1",
            "--config",
            config.to_str().unwrap(),
            "--profile",
            "four",
            "--format",
            "compact",
        ],
        &base,
    );
    let first_page: Value = serde_json::from_slice(&first_page.stdout).unwrap();
    let cursor = first_page["result"]["response"]["next_cursor"]
        .as_str()
        .unwrap()
        .to_owned();

    let disabled = run_isolated(
        &[
            "collections",
            "disable",
            "--profile",
            "four",
            "--pack",
            "llg",
            "--config",
            config.to_str().unwrap(),
        ],
        &base,
    );
    assert!(
        disabled.status.success(),
        "{}",
        String::from_utf8_lossy(&disabled.stderr)
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&disabled.stdout).unwrap()["revision"],
        2
    );
    let disabled_selection = run_isolated(
        &[
            "collections",
            "selection",
            "--config",
            config.to_str().unwrap(),
            "--profile",
            "four",
        ],
        &base,
    );
    assert!(disabled_selection.status.success());
    assert_eq!(
        serde_json::from_slice::<Value>(&disabled_selection.stdout).unwrap()["local_revision"],
        1,
        "profile-only policy updates do not change the catalogue revision"
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&disabled_selection.stdout).unwrap()["disabled"],
        json!(["llg"])
    );
    let disabled_lookup = lookup(None, None);
    let disabled_value: Value = serde_json::from_slice(&disabled_lookup.stdout).unwrap();
    assert_eq!(
        disabled_value["result"]["response"]["totals"]["occurrences"],
        4
    );
    assert_eq!(
        disabled_value["result"]["response"]["source_totals"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    let disabled_compact = lookup(Some("compact"), None);
    assert!(disabled_compact.status.success());
    let disabled_compact: Value = serde_json::from_slice(&disabled_compact.stdout).unwrap();
    assert_eq!(
        disabled_compact["result"]["response"]["totals"]["occurrences"],
        4
    );
    let disabled_readable = lookup(Some("readable"), None);
    assert!(disabled_readable.status.success());
    assert!(
        String::from_utf8(disabled_readable.stdout)
            .unwrap()
            .starts_with("Exact \"farvi\": 4 occurrences")
    );
    let stale_cursor = run_isolated(
        &[
            "lookup",
            "farvi",
            "--page-size",
            "1",
            "--cursor",
            &cursor,
            "--config",
            config.to_str().unwrap(),
            "--profile",
            "four",
            "--format",
            "compact",
        ],
        &base,
    );
    assert!(!stale_cursor.status.success());
    assert_eq!(
        serde_json::from_slice::<Value>(&stale_cursor.stderr).unwrap()["error"]["category"],
        "stale_cursor"
    );
    let disabled_snapshot = disabled_compact["snapshot"]["binding_sha256"]
        .as_str()
        .unwrap();
    for reference in [&detail_ref, &catalog_detail_ref] {
        let disabled_detail = run_isolated(
            &[
                "lookup-detail",
                "--reference",
                &reference.to_string(),
                "--expected-snapshot",
                disabled_snapshot,
                "--config",
                config.to_str().unwrap(),
                "--profile",
                "four",
            ],
            &base,
        );
        assert!(
            disabled_detail.status.success(),
            "{}",
            String::from_utf8_lossy(&disabled_detail.stderr)
        );
        let disabled_detail: Value = serde_json::from_slice(&disabled_detail.stdout).unwrap();
        assert_eq!(disabled_detail["response"]["status"], "CatalogueOnly");
        assert_eq!(disabled_detail["response"]["hit"], Value::Null);
    }

    let enabled = run_isolated(
        &[
            "collections",
            "enable",
            "--profile",
            "four",
            "--pack",
            "llg",
            "--config",
            config.to_str().unwrap(),
        ],
        &base,
    );
    assert!(
        enabled.status.success(),
        "{}",
        String::from_utf8_lossy(&enabled.stderr)
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&enabled.stdout).unwrap()["revision"],
        3
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&lookup(None, None).stdout).unwrap()["result"]["response"]
            ["totals"]["occurrences"],
        5
    );
    let enabled_compact = lookup(Some("compact"), None);
    assert!(enabled_compact.status.success());
    let enabled_compact: Value = serde_json::from_slice(&enabled_compact.stdout).unwrap();
    assert_eq!(
        enabled_compact["result"]["response"]["totals"]["occurrences"],
        5
    );
    let enabled_readable = lookup(Some("readable"), None);
    assert!(enabled_readable.status.success());
    assert!(
        String::from_utf8(enabled_readable.stdout)
            .unwrap()
            .starts_with("Exact \"farvi\": 5 occurrences")
    );

    let removed = run_isolated(
        &[
            "collections",
            "remove",
            root.to_str().unwrap(),
            "--pack",
            "llg",
            "--expected-revision",
            "1",
        ],
        &base,
    );
    assert!(
        removed.status.success(),
        "{}",
        String::from_utf8_lossy(&removed.stderr)
    );
    let removed: Value = serde_json::from_slice(&removed.stdout).unwrap();
    assert_eq!(removed["local_revision"], 2);
    let removed_default = lookup(None, None);
    assert!(!removed_default.status.success());
    let removed_default_error: Value = serde_json::from_slice(&removed_default.stderr).unwrap();
    assert_eq!(
        removed_default_error["error"]["category"], "unavailable_selection",
        "removing the configured default must not silently fall back to another pack"
    );

    let removed_local = lookup(Some("compact"), Some(&local_request));
    assert!(
        removed_local.status.success(),
        "{}",
        String::from_utf8_lossy(&removed_local.stderr)
    );
    let removed_local: Value = serde_json::from_slice(&removed_local.stdout).unwrap();
    assert_eq!(
        removed_local["result"]["response"]["status"],
        "CatalogueOnly"
    );
    assert_eq!(
        removed_local["result"]["response"]["operational_reason"],
        "removed"
    );
    assert_eq!(removed_local["result"]["response"]["payload"], Value::Null);
    let removed_catalog = lookup(Some("compact"), Some(&catalog_request));
    assert!(removed_catalog.status.success());
    let removed_catalog: Value = serde_json::from_slice(&removed_catalog.stdout).unwrap();
    assert_eq!(
        removed_catalog["result"]["response"]["status"],
        "CatalogueOnly"
    );
    assert_eq!(
        removed_catalog["result"]["response"]["operational_reason"],
        "removed"
    );
    assert_eq!(
        removed_catalog["result"]["response"]["payload"],
        Value::Null
    );
    let removed_snapshot = removed_catalog["snapshot"]["binding_sha256"]
        .as_str()
        .unwrap();
    let stale_old_snapshot = run_isolated(
        &[
            "lookup-detail",
            "--reference",
            &detail_ref.to_string(),
            "--expected-snapshot",
            snapshot,
            "--config",
            config.to_str().unwrap(),
            "--profile",
            "four",
        ],
        &base,
    );
    assert!(!stale_old_snapshot.status.success());
    assert_eq!(
        serde_json::from_slice::<Value>(&stale_old_snapshot.stderr).unwrap()["error"]["category"],
        "stale_snapshot"
    );
    for reference in [&detail_ref, &catalog_detail_ref] {
        let removed_detail = run_isolated(
            &[
                "lookup-detail",
                "--reference",
                &reference.to_string(),
                "--expected-snapshot",
                removed_snapshot,
                "--config",
                config.to_str().unwrap(),
                "--profile",
                "four",
            ],
            &base,
        );
        assert!(removed_detail.status.success());
        let removed_detail: Value = serde_json::from_slice(&removed_detail.stdout).unwrap();
        assert_eq!(removed_detail["response"]["status"], "CatalogueOnly");
        assert_eq!(removed_detail["response"]["hit"], Value::Null);
        let removed_detail_readable = run_isolated(
            &[
                "lookup-detail",
                "--reference",
                &reference.to_string(),
                "--expected-snapshot",
                removed_snapshot,
                "--config",
                config.to_str().unwrap(),
                "--profile",
                "four",
                "--format",
                "readable",
            ],
            &base,
        );
        assert!(removed_detail_readable.status.success());
        assert!(
            String::from_utf8(removed_detail_readable.stdout)
                .unwrap()
                .contains("CatalogueOnly")
        );
    }

    let reinstall = run_isolated(
        &[
            "collections",
            "reinstall",
            root.to_str().unwrap(),
            "--pack",
            "llg",
            "--release",
            llg_release_path.to_str().unwrap(),
            "--seal",
            llg_seal_path.to_str().unwrap(),
            "--allow-synthetic",
            "--release-sha256",
            &installed_release_sha,
            "--expected-revision",
            "2",
        ],
        &base,
    );
    assert!(
        reinstall.status.success(),
        "{}",
        String::from_utf8_lossy(&reinstall.stderr)
    );
    let reinstalled: Value = serde_json::from_slice(&reinstall.stdout).unwrap();
    assert_eq!(reinstalled["local_revision"], 3);
    let restored_local = lookup(Some("compact"), Some(&local_request));
    let restored_local: Value = serde_json::from_slice(&restored_local.stdout).unwrap();
    assert_eq!(restored_local["result"]["response"]["status"], "Resolved");
    assert_eq!(
        restored_local["result"]["response"]["payload"]["confirmed"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    let restored_catalog = lookup(Some("compact"), Some(&catalog_request));
    assert!(restored_catalog.status.success());
    let restored_catalog: Value = serde_json::from_slice(&restored_catalog.stdout).unwrap();
    assert_eq!(restored_catalog["result"]["response"]["status"], "Resolved");
    assert_eq!(
        restored_catalog["result"]["response"]["payload"]["confirmed"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    let restored_full = lookup(None, None);
    assert!(restored_full.status.success());
    let restored_full_value: Value = serde_json::from_slice(&restored_full.stdout).unwrap();
    assert_eq!(
        restored_full_value["result"]["response"]["totals"]["occurrences"],
        5
    );
    let restored_compact = lookup(Some("compact"), None);
    assert!(restored_compact.status.success());
    let restored_compact_value: Value = serde_json::from_slice(&restored_compact.stdout).unwrap();
    assert_eq!(
        restored_compact_value["result"]["response"]["totals"]["occurrences"],
        5
    );
    let restored_readable = lookup(Some("readable"), None);
    assert!(restored_readable.status.success());
    assert!(
        String::from_utf8(restored_readable.stdout)
            .unwrap()
            .starts_with("Exact \"farvi\": 5 occurrences")
    );
    let restored_snapshot = restored_compact_value["snapshot"]["binding_sha256"]
        .as_str()
        .unwrap();
    let restored_detail = run_isolated(
        &[
            "lookup-detail",
            "--reference",
            &detail_ref.to_string(),
            "--expected-snapshot",
            restored_snapshot,
            "--config",
            config.to_str().unwrap(),
            "--profile",
            "four",
        ],
        &base,
    );
    assert!(restored_detail.status.success());
    let restored_detail: Value = serde_json::from_slice(&restored_detail.stdout).unwrap();
    assert_eq!(restored_detail["response"]["status"], "Resolved");
    assert_eq!(
        restored_detail["response"]["hit"]["assertion"]["id"],
        detail_ref["assertion_id"]
    );
    let stale_restored_old_snapshot = run_isolated(
        &[
            "lookup-detail",
            "--reference",
            &detail_ref.to_string(),
            "--expected-snapshot",
            snapshot,
            "--config",
            config.to_str().unwrap(),
            "--profile",
            "four",
        ],
        &base,
    );
    assert!(!stale_restored_old_snapshot.status.success());
    assert_eq!(
        serde_json::from_slice::<Value>(&stale_restored_old_snapshot.stderr).unwrap()["error"]["category"],
        "stale_snapshot"
    );
    let stale_restored_cursor = run_isolated(
        &[
            "lookup",
            "farvi",
            "--page-size",
            "1",
            "--cursor",
            &cursor,
            "--config",
            config.to_str().unwrap(),
            "--profile",
            "four",
            "--format",
            "compact",
        ],
        &base,
    );
    assert!(!stale_restored_cursor.status.success());
    assert_eq!(
        serde_json::from_slice::<Value>(&stale_restored_cursor.stderr).unwrap()["error"]["category"],
        "stale_cursor"
    );
    let fresh_cursor_page1 = run_isolated(
        &[
            "lookup",
            "farvi",
            "--page-size",
            "1",
            "--config",
            config.to_str().unwrap(),
            "--profile",
            "four",
            "--format",
            "compact",
        ],
        &base,
    );
    assert!(fresh_cursor_page1.status.success());
    let fresh_cursor_page1: Value = serde_json::from_slice(&fresh_cursor_page1.stdout).unwrap();
    let fresh_cursor = fresh_cursor_page1["result"]["response"]["next_cursor"]
        .as_str()
        .unwrap();
    let fresh_cursor_page2 = run_isolated(
        &[
            "lookup",
            "farvi",
            "--page-size",
            "1",
            "--cursor",
            fresh_cursor,
            "--config",
            config.to_str().unwrap(),
            "--profile",
            "four",
            "--format",
            "compact",
        ],
        &base,
    );
    assert!(fresh_cursor_page2.status.success());
    assert_eq!(
        sha256(
            &DictionaryStore::open(&installed_store)
                .unwrap()
                .canonical_bytes()
                .unwrap()
        ),
        old_generation_sha,
        "retired G1 canonical bytes remain unchanged"
    );

    let reconciled = run_isolated(&["collections", "reconcile", root.to_str().unwrap()], &base);
    assert!(
        reconciled.status.success(),
        "{}",
        String::from_utf8_lossy(&reconciled.stderr)
    );
    let reconciled: Value = serde_json::from_slice(&reconciled.stdout).unwrap();
    assert_eq!(reconciled["retained_generation_count"], 3);
    assert_eq!(
        reconciled["active_generation"],
        reinstalled["generation"]["generation_id"]
    );
    assert_eq!(reconciled["attention_required"], false);
    assert_eq!(reconciled["local_revision"], 3);
    assert_eq!(reconciled["catalogue_schema_version"], 2);
    let final_profiles = run_isolated(
        &[
            "collections",
            "profiles",
            "--config",
            config.to_str().unwrap(),
        ],
        &base,
    );
    let final_profiles: Value = serde_json::from_slice(&final_profiles.stdout).unwrap();
    assert_eq!(final_profiles["revision"], 3);
    let final_list = run_isolated(&["collections", "list", root.to_str().unwrap()], &base);
    assert_eq!(
        serde_json::from_slice::<Value>(&final_list.stdout).unwrap()["local_revision"],
        3
    );
    let _ = std::fs::remove_dir_all(base);
}

#[cfg(target_os = "linux")]
fn run_os(args: &[std::ffi::OsString]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
        .args(args)
        .env_clear()
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .unwrap()
}

#[cfg(unix)]
fn run_after_removing_cwd(cwd: &Path, args: &[std::ffi::OsString]) -> Output {
    let mut command = Command::new("/bin/sh");
    command
        .current_dir(cwd)
        .args(["-c", "rmdir \"$1\" || exit; shift; exec \"$@\"", "sh"])
        .arg(cwd)
        .arg(env!("CARGO_BIN_EXE_vlajmaji"))
        .args(args)
        .env_clear()
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    command.output().unwrap()
}

fn assert_json_keys(value: &Value, expected: &[&str]) {
    let actual: BTreeSet<_> = value
        .as_object()
        .expect("response is a JSON object")
        .keys()
        .map(String::as_str)
        .collect();
    let expected: BTreeSet<_> = expected.iter().copied().collect();
    assert_eq!(actual, expected);
}

fn install(root: &Path) -> PathBuf {
    let mut document = fixture::complete_fixture().unwrap().into_document();
    document.legacy_archive = None;
    let dictionary = Dictionary::from_document(document).unwrap();
    let sealed = seal_release(PackId::Llg, PackPayloadKind::Synthetic, &dictionary).unwrap();
    let request = CollectionRequest::new(
        vec![CollectionInput::SealedBytes {
            pack_id: PackId::Llg,
            release: Cow::Borrowed(&sealed.release),
            seal: Cow::Borrowed(&sealed.seal),
        }],
        CollectionSelection::Packs(vec![PackId::Llg]),
        true,
    )
    .unwrap();
    let report = CollectionCatalogue::install(root, request).unwrap();
    report.generation.unwrap().store_path
}

#[test]
fn test_dictionary_compact_query_and_detail_keep_full_default() {
    let root = scratch();
    let input = root.join("dictionary.json");
    let store = root.join("dictionary-store");
    let bytes = fixture::complete_fixture()
        .unwrap()
        .to_json_bytes()
        .unwrap();
    std::fs::write(&input, bytes).unwrap();
    let build = run(&[Path::new("dictionary-build"), &input, &store]);
    assert!(
        build.status.success(),
        "{}",
        String::from_utf8_lossy(&build.stderr)
    );

    let full = run_strings(&[
        "dictionary-query",
        store.to_str().unwrap(),
        "--request",
        r#"{"operation":"exact","spelling":"farvi"}"#,
    ]);
    assert!(
        full.status.success(),
        "{}",
        String::from_utf8_lossy(&full.stderr)
    );
    let full: Value = serde_json::from_slice(&full.stdout).unwrap();
    assert!(full["groups"][0]["hits"][0].get("assertion").is_some());
    let explicit_full = run_strings(&[
        "dictionary-query",
        store.to_str().unwrap(),
        "--request",
        r#"{"operation":"exact","spelling":"farvi"}"#,
        "--format",
        "full",
    ]);
    assert!(explicit_full.status.success());
    assert_eq!(
        full,
        serde_json::from_slice::<Value>(&explicit_full.stdout).unwrap()
    );

    let readable = run_strings(&[
        "dictionary-query",
        store.to_str().unwrap(),
        "--request",
        r#"{"operation":"exact","spelling":"farvi"}"#,
        "--format",
        "readable",
    ]);
    assert!(
        readable.status.success(),
        "{}",
        String::from_utf8_lossy(&readable.stderr)
    );
    let readable = String::from_utf8(readable.stdout).unwrap();
    assert!(readable.starts_with("Exact \"farvi\": "));
    assert!(readable.contains("\n  "));
    assert!(readable.ends_with("next_cursor: null\n"));

    let compact = run_strings(&[
        "dictionary-query",
        store.to_str().unwrap(),
        "--request",
        r#"{"operation":"exact","spelling":"farvi"}"#,
        "--format",
        "compact",
    ]);
    assert!(
        compact.status.success(),
        "{}",
        String::from_utf8_lossy(&compact.stderr)
    );
    let compact: Value = serde_json::from_slice(&compact.stdout).unwrap();
    assert_json_keys(
        &compact,
        &[
            "version",
            "projection_policy",
            "policy",
            "store_schema",
            "catalogue_revision",
            "state_sha256",
            "snapshot_sha256",
            "spelling",
            "resolved_sources",
            "native_candidate_occurrences",
            "totals",
            "source_totals",
            "availability",
            "groups",
            "next_cursor",
            "code_notices",
        ],
    );
    assert!(compact.get("operation").is_none());
    assert!(compact.get("response").is_none());
    let detail_ref = compact["groups"][0]["hits"][0]["detail_ref"].clone();
    assert!(compact["groups"][0]["hits"][0].get("raw").is_none());
    assert!(compact["groups"][0]["hits"][0].get("contexts").is_none());
    let detail = run_strings(&[
        "dictionary-detail",
        store.to_str().unwrap(),
        "--reference",
        &detail_ref.to_string(),
        "--expected-snapshot",
        compact["snapshot_sha256"].as_str().unwrap(),
    ]);
    assert!(
        detail.status.success(),
        "{}",
        String::from_utf8_lossy(&detail.stderr)
    );
    let detail: Value = serde_json::from_slice(&detail.stdout).unwrap();
    assert_eq!(detail["hit"]["assertion"]["id"], detail_ref["assertion_id"]);

    let detail_readable = run_strings(&[
        "dictionary-detail",
        store.to_str().unwrap(),
        "--reference",
        &detail_ref.to_string(),
        "--expected-snapshot",
        compact["snapshot_sha256"].as_str().unwrap(),
        "--format",
        "readable",
    ]);
    assert!(detail_readable.status.success());
    let detail_readable = String::from_utf8(detail_readable.stdout).unwrap();
    assert!(detail_readable.starts_with("Detail "));
    assert!(detail_readable.contains("\"occurrence_id\""));
    assert!(detail_readable.ends_with("\n}\n"));

    let mut invalid_detail_cases = vec![
        {
            let mut value = detail_ref.clone();
            value["role"] = Value::String("entry".into());
            value
        },
        {
            let mut value = detail_ref.clone();
            value["version"] = Value::from(2);
            value
        },
        {
            let mut value = detail_ref.clone();
            value.as_object_mut().unwrap().remove("native_entry_id");
            value
        },
    ];
    for required_nullable in ["native_assertion_id", "contributor_id", "distributor_id"] {
        let mut missing = detail_ref.clone();
        missing.as_object_mut().unwrap().remove(required_nullable);
        invalid_detail_cases.push(missing);
        let mut wrong_type = detail_ref.clone();
        wrong_type[required_nullable] = Value::from(7);
        invalid_detail_cases.push(wrong_type);
    }
    for (field, value) in [
        ("source", Value::String("invalid source id".into())),
        ("entry_id", Value::String("invalid entry id".into())),
        ("assertion_id", Value::String("invalid assertion id".into())),
    ] {
        let mut invalid_id = detail_ref.clone();
        invalid_id[field] = value;
        invalid_detail_cases.push(invalid_id);
    }
    let mut empty_locator = detail_ref.clone();
    empty_locator["occurrence"]["locator"] = Value::String(String::new());
    invalid_detail_cases.push(empty_locator);
    let mut empty_capture_namespace = detail_ref.clone();
    empty_capture_namespace["occurrence"]["capture"]["namespace"] = Value::String(String::new());
    invalid_detail_cases.push(empty_capture_namespace);
    let mut invalid_capture_digest = detail_ref.clone();
    invalid_capture_digest["occurrence"]["capture"]["sha256"] = Value::String("A".repeat(64));
    invalid_detail_cases.push(invalid_capture_digest);
    for invalid_reference in invalid_detail_cases {
        let invalid = run_strings(&[
            "dictionary-detail",
            root.join("missing-store").to_str().unwrap(),
            "--reference",
            &invalid_reference.to_string(),
        ]);
        assert!(!invalid.status.success());
        let error: Value = serde_json::from_slice(&invalid.stderr).unwrap();
        assert_eq!(error["error"]["code"], "invalid_detail_reference");
    }

    let dict = fixture::complete_fixture().unwrap();
    let entry = dict.document().metadata.entries[0].id.clone();
    let resolve_request = serde_json::to_string(&DictionaryRequest::Resolve {
        target: RelationTarget::Local(LocalRef::Entry(entry)),
    })
    .unwrap();
    let resolved = run_strings(&[
        "dictionary-query",
        store.to_str().unwrap(),
        "--request",
        &resolve_request,
        "--format",
        "compact",
    ]);
    assert!(
        resolved.status.success(),
        "{}",
        String::from_utf8_lossy(&resolved.stderr)
    );
    let resolved: Value = serde_json::from_slice(&resolved.stdout).unwrap();
    assert_json_keys(
        &resolved,
        &[
            "version",
            "projection_policy",
            "state_sha256",
            "snapshot_sha256",
            "target",
            "status",
            "entity",
            "confirmed",
            "unverified",
            "availability",
            "requested_release",
        ],
    );
    assert!(resolved.get("operation").is_none());
    assert!(resolved.get("response").is_none());
    assert!(resolved["target"].is_object());
    assert!(resolved["status"].is_string());
    let full_local = run_strings(&[
        "dictionary-query",
        store.to_str().unwrap(),
        "--request",
        &resolve_request,
        "--format",
        "full",
    ]);
    assert!(full_local.status.success());
    assert_eq!(
        serde_json::from_slice::<Value>(&full_local.stdout).unwrap()["status"],
        "Resolved"
    );
    let readable_local = run_strings(&[
        "dictionary-query",
        store.to_str().unwrap(),
        "--request",
        &resolve_request,
        "--format",
        "readable",
    ]);
    assert!(readable_local.status.success());
    let readable_local = String::from_utf8(readable_local.stdout).unwrap();
    assert!(readable_local.starts_with("Reference {\"Local\":"));
    assert!(readable_local.contains(": Resolved\n"));

    let document = dict.document();
    let (catalog_release, catalog_assertion) = document
        .metadata
        .releases
        .iter()
        .filter(|release| {
            release.selected && release.availability == vlajmaji_lexicon::Availability::Available
        })
        .find_map(|release| {
            document
                .metadata
                .assertions
                .iter()
                .find(|assertion| release.assertions.contains(&assertion.id))
                .map(|assertion| (release, assertion))
        })
        .expect("fixture has a selected available release with an assertion");
    let catalog_entry = document
        .metadata
        .entries
        .iter()
        .find(|entry| entry.id == catalog_assertion.entry)
        .expect("catalogue assertion has its Entry");
    let catalog_request = serde_json::to_string(&DictionaryRequest::Resolve {
        target: RelationTarget::Catalog(CatalogTarget {
            source: catalog_entry.source.clone(),
            release: catalog_release.id.clone(),
            native_entry_id: None,
            native_assertion_id: None,
            occurrence: Some(catalog_assertion.occurrence.clone()),
        }),
    })
    .unwrap();
    for format in ["full", "compact", "readable"] {
        let catalog = run_strings(&[
            "dictionary-query",
            store.to_str().unwrap(),
            "--request",
            &catalog_request,
            "--format",
            format,
        ]);
        assert!(
            catalog.status.success(),
            "{format}: {}",
            String::from_utf8_lossy(&catalog.stderr)
        );
        match format {
            "readable" => {
                let text = String::from_utf8(catalog.stdout).unwrap();
                assert!(text.starts_with("Reference {\"Catalog\":"));
                assert!(text.contains(": Resolved\n"));
            }
            _ => {
                let value: Value = serde_json::from_slice(&catalog.stdout).unwrap();
                assert_eq!(value["status"], "Resolved");
                if format == "compact" {
                    assert!(value.get("operation").is_none());
                    assert!(value.get("response").is_none());
                }
            }
        }
    }

    let malformed = run_strings(&[
        "dictionary-detail",
        root.join("missing-store").to_str().unwrap(),
        "--reference",
        "{",
    ]);
    assert!(!malformed.status.success());
    let error: Value = serde_json::from_slice(&malformed.stderr).unwrap();
    assert_eq!(error["error"]["code"], "invalid_detail_reference");
    let trailing = run_strings(&[
        "dictionary-detail",
        root.join("missing-store").to_str().unwrap(),
        "--reference",
        "{} {}",
    ]);
    assert!(!trailing.status.success());
    let error: Value = serde_json::from_slice(&trailing.stderr).unwrap();
    assert_eq!(error["error"]["code"], "invalid_detail_reference");
    let managed_malformed = run_strings(&[
        "lookup-detail",
        "--reference",
        "{",
        "--config",
        root.join("missing-config.toml").to_str().unwrap(),
        "--profile",
        "requested",
        "--pack",
        "llg",
    ]);
    assert!(!managed_malformed.status.success());
    let managed_error: Value = serde_json::from_slice(&managed_malformed.stderr).unwrap();
    assert_eq!(
        managed_error["error"]["category"],
        "invalid_detail_reference"
    );
    assert_eq!(managed_error["error"]["operation"], "lookup_detail");
    assert_eq!(managed_error["error"]["outcome"], "not_applicable");
    assert!(managed_error["error"]["snapshot"].is_null());
    assert_eq!(managed_error["error"]["config_path"], Value::Null);
    assert_eq!(managed_error["error"]["root_path"], Value::Null);
    assert_eq!(managed_error["error"]["profile"], Value::Null);
    assert!(managed_error["error"]["requested_profile"].is_null());
    assert_eq!(
        managed_error["error"]["known_pack_ids"],
        serde_json::json!([])
    );
    assert!(managed_error["error"]["writer"].is_null());
    for key in [
        "version",
        "ok",
        "category",
        "detail",
        "operation",
        "config_path",
        "root_path",
        "profile",
        "requested_profile",
        "snapshot",
        "known_pack_ids",
        "outcome",
        "writer",
    ] {
        assert!(
            managed_error["error"].get(key).is_some(),
            "missing field {key}"
        );
    }
    let malformed_snapshot = run_strings(&[
        "lookup-detail",
        "--reference",
        &detail_ref.to_string(),
        "--expected-snapshot",
        "not-a-digest",
        "--config",
        root.join("missing-config.toml").to_str().unwrap(),
        "--profile",
        "requested",
        "--pack",
        "llg",
    ]);
    assert!(!malformed_snapshot.status.success());
    let malformed_snapshot: Value = serde_json::from_slice(&malformed_snapshot.stderr).unwrap();
    assert_eq!(
        malformed_snapshot["error"]["category"],
        "malformed_snapshot"
    );
    assert!(malformed_snapshot["error"]["config_path"].is_null());
    assert!(malformed_snapshot["error"]["root_path"].is_null());
    assert!(malformed_snapshot["error"]["profile"].is_null());
    assert!(malformed_snapshot["error"]["requested_profile"].is_null());
    assert!(malformed_snapshot["error"]["snapshot"].is_null());
    assert_eq!(
        malformed_snapshot["error"]["known_pack_ids"],
        serde_json::json!([])
    );
    assert!(malformed_snapshot["error"]["writer"].is_null());
    let mut opaque_unicode_namespace = detail_ref.clone();
    opaque_unicode_namespace["occurrence"]["capture"]["namespace"] =
        Value::String("opaque/λ/é".into());
    let unicode_static_reference = run_strings(&[
        "lookup-detail",
        "--reference",
        &opaque_unicode_namespace.to_string(),
        "--config",
        root.join("missing-config.toml").to_str().unwrap(),
        "--profile",
        "requested",
        "--pack",
        "llg",
    ]);
    assert!(!unicode_static_reference.status.success());
    let unicode_error: Value = serde_json::from_slice(&unicode_static_reference.stderr).unwrap();
    assert_eq!(unicode_error["error"]["category"], "config_missing");
    assert_eq!(unicode_error["error"]["operation"], "lookup_detail");
    assert_eq!(unicode_error["error"]["outcome"], "not_applicable");
    let wrong_role = {
        let mut value = detail_ref.clone();
        value["role"] = Value::String("entry".into());
        value
    };
    let managed_invalid = run_strings(&[
        "lookup-detail",
        "--reference",
        &wrong_role.to_string(),
        "--config",
        root.join("missing-config.toml").to_str().unwrap(),
        "--profile",
        "requested",
        "--pack",
        "llg",
    ]);
    assert!(!managed_invalid.status.success());
    let managed_invalid: Value = serde_json::from_slice(&managed_invalid.stderr).unwrap();
    assert_eq!(
        managed_invalid["error"]["category"],
        "invalid_detail_reference"
    );
    assert!(managed_invalid["error"]["config_path"].is_null());
    assert!(managed_invalid["error"]["root_path"].is_null());
    assert!(managed_invalid["error"]["profile"].is_null());
    assert!(managed_invalid["error"]["requested_profile"].is_null());
    assert!(managed_invalid["error"]["snapshot"].is_null());
    assert_eq!(
        managed_invalid["error"]["known_pack_ids"],
        serde_json::json!([])
    );
    assert_eq!(managed_invalid["error"]["outcome"], "not_applicable");
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn test_lookup_compact_and_detail_preserve_managed_envelope() {
    let base = scratch();
    let root = base.join("catalogue");
    let store_path = install(&root);
    let config = base.join("config.toml");
    let configure = run_strings(&[
        "collections",
        "configure",
        root.to_str().unwrap(),
        "--profile",
        "compact-test",
        "--pack",
        "llg",
        "--make-default",
        "--config",
        config.to_str().unwrap(),
    ]);
    assert!(
        configure.status.success(),
        "{}",
        String::from_utf8_lossy(&configure.stderr)
    );

    let lookup = run_strings(&[
        "lookup",
        "farvi",
        "--config",
        config.to_str().unwrap(),
        "--profile",
        "compact-test",
        "--format",
        "compact",
    ]);
    assert!(
        lookup.status.success(),
        "{}",
        String::from_utf8_lossy(&lookup.stderr)
    );
    let lookup: Value = serde_json::from_slice(&lookup.stdout).unwrap();
    assert_eq!(lookup["result"]["operation"], "exact");
    assert!(lookup["snapshot"]["binding_sha256"].is_string());
    let detail_ref = lookup["result"]["response"]["groups"][0]["hits"][0]["detail_ref"].clone();
    let detail = run_strings(&[
        "lookup-detail",
        "--reference",
        &detail_ref.to_string(),
        "--expected-snapshot",
        lookup["snapshot"]["binding_sha256"].as_str().unwrap(),
        "--config",
        config.to_str().unwrap(),
        "--profile",
        "compact-test",
    ]);
    assert!(
        detail.status.success(),
        "{}",
        String::from_utf8_lossy(&detail.stderr)
    );
    let detail: Value = serde_json::from_slice(&detail.stdout).unwrap();
    assert_eq!(
        detail["response"]["hit"]["assertion"]["id"],
        detail_ref["assertion_id"]
    );
    assert!(detail["snapshot"]["binding_sha256"].is_string());

    let readable_lookup = run_strings(&[
        "lookup",
        "farvi",
        "--config",
        config.to_str().unwrap(),
        "--profile",
        "compact-test",
        "--format",
        "readable",
    ]);
    assert!(
        readable_lookup.status.success(),
        "{}",
        String::from_utf8_lossy(&readable_lookup.stderr)
    );
    let readable_lookup = String::from_utf8(readable_lookup.stdout).unwrap();
    assert!(readable_lookup.starts_with("Exact \"farvi\": "));
    assert!(readable_lookup.contains("selection: {\"version\":1,"));
    assert!(readable_lookup.ends_with("\n"));

    let readable_detail = run_strings(&[
        "lookup-detail",
        "--reference",
        &detail_ref.to_string(),
        "--expected-snapshot",
        lookup["snapshot"]["binding_sha256"].as_str().unwrap(),
        "--config",
        config.to_str().unwrap(),
        "--profile",
        "compact-test",
        "--format",
        "readable",
    ]);
    assert!(
        readable_detail.status.success(),
        "{}",
        String::from_utf8_lossy(&readable_detail.stderr)
    );
    let readable_detail = String::from_utf8(readable_detail.stdout).unwrap();
    assert!(readable_detail.starts_with("Detail "));
    assert!(readable_detail.contains("catalogue_matches: ["));
    assert!(readable_detail.contains("\"occurrence_id\""));

    let invalid_detail_format = run_strings(&[
        "lookup-detail",
        "--reference",
        &detail_ref.to_string(),
        "--config",
        config.to_str().unwrap(),
        "--format",
        "compact",
    ]);
    assert_eq!(invalid_detail_format.status.code(), Some(2));
    assert!(store_path.exists());
    std::fs::remove_dir_all(base).unwrap();
}

#[cfg(unix)]
#[test]
fn test_managed_compact_lookup_and_detail_resolve_absolute_paths_after_deleted_cwd() {
    let base = scratch();
    let root = base.join("catalogue");
    install(&root);
    let config = base.join("config.toml");
    let configure = run_strings(&[
        "collections",
        "configure",
        root.to_str().unwrap(),
        "--profile",
        "compact-cwd",
        "--pack",
        "llg",
        "--make-default",
        "--config",
        config.to_str().unwrap(),
    ]);
    assert!(
        configure.status.success(),
        "{}",
        String::from_utf8_lossy(&configure.stderr)
    );

    let absolute_cwd = base.join("absolute-child-cwd");
    std::fs::create_dir(&absolute_cwd).unwrap();
    let absolute_lookup_args = vec![
        "lookup".into(),
        "farvi".into(),
        "--config".into(),
        config.as_os_str().to_owned(),
        "--profile".into(),
        "compact-cwd".into(),
        "--format".into(),
        "compact".into(),
    ];
    let lookup = run_after_removing_cwd(&absolute_cwd, &absolute_lookup_args);
    assert!(
        lookup.status.success(),
        "{}",
        String::from_utf8_lossy(&lookup.stderr)
    );
    assert!(!absolute_cwd.exists());
    let lookup: Value = serde_json::from_slice(&lookup.stdout).unwrap();
    let detail_ref = lookup["result"]["response"]["groups"][0]["hits"][0]["detail_ref"].clone();
    let expected_snapshot = lookup["snapshot"]["binding_sha256"].as_str().unwrap();

    let readable_cwd = base.join("readable-child-cwd");
    std::fs::create_dir(&readable_cwd).unwrap();
    let mut readable_lookup_args = absolute_lookup_args.clone();
    let last = readable_lookup_args.len() - 1;
    readable_lookup_args[last] = "readable".into();
    let readable_lookup = run_after_removing_cwd(&readable_cwd, &readable_lookup_args);
    assert!(
        readable_lookup.status.success(),
        "{}",
        String::from_utf8_lossy(&readable_lookup.stderr)
    );
    assert!(
        String::from_utf8(readable_lookup.stdout)
            .unwrap()
            .starts_with("Exact \"farvi\": ")
    );

    let detail_cwd = base.join("detail-child-cwd");
    std::fs::create_dir(&detail_cwd).unwrap();
    let absolute_detail_args = vec![
        "lookup-detail".into(),
        "--reference".into(),
        detail_ref.to_string().into(),
        "--expected-snapshot".into(),
        expected_snapshot.into(),
        "--config".into(),
        config.as_os_str().to_owned(),
        "--profile".into(),
        "compact-cwd".into(),
    ];
    let detail = run_after_removing_cwd(&detail_cwd, &absolute_detail_args);
    assert!(
        detail.status.success(),
        "{}",
        String::from_utf8_lossy(&detail.stderr)
    );
    assert!(!detail_cwd.exists());
    let detail: Value = serde_json::from_slice(&detail.stdout).unwrap();
    assert_eq!(
        detail["response"]["hit"]["assertion"]["id"],
        detail_ref["assertion_id"]
    );

    let readable_detail_cwd = base.join("readable-detail-child-cwd");
    std::fs::create_dir(&readable_detail_cwd).unwrap();
    let mut readable_detail_args = absolute_detail_args.clone();
    readable_detail_args.extend(["--format".into(), "readable".into()]);
    let readable_detail = run_after_removing_cwd(&readable_detail_cwd, &readable_detail_args);
    assert!(
        readable_detail.status.success(),
        "{}",
        String::from_utf8_lossy(&readable_detail.stderr)
    );
    assert!(
        String::from_utf8(readable_detail.stdout)
            .unwrap()
            .starts_with("Detail ")
    );

    let relative_cwd = base.join("relative-child-cwd");
    std::fs::create_dir(&relative_cwd).unwrap();
    let relative_lookup_args = vec![
        "lookup".into(),
        "farvi".into(),
        "--config".into(),
        "relative.toml".into(),
        "--profile".into(),
        "compact-cwd".into(),
        "--format".into(),
        "compact".into(),
    ];
    let relative = run_after_removing_cwd(&relative_cwd, &relative_lookup_args);
    assert!(!relative.status.success());
    assert!(!relative_cwd.exists());
    let error: Value = serde_json::from_slice(&relative.stderr).unwrap();
    assert_eq!(error["error"]["category"], "config_io");
    assert_eq!(error["error"]["operation"], "lookup");
    assert_eq!(error["error"]["config_path"]["utf8"], "relative.toml");
    assert_eq!(error["error"]["outcome"], "not_applicable");
    std::fs::remove_dir_all(base).unwrap();
}

#[cfg(target_os = "linux")]
#[test]
fn test_managed_compact_routes_roundtrip_native_root_and_config() {
    use std::os::unix::ffi::OsStringExt;

    let base = scratch();
    let native_root = base.join(std::ffi::OsString::from_vec(b"catalogue-\xff".to_vec()));
    install(&native_root);
    let native_config = base.join(std::ffi::OsString::from_vec(b"profiles-\xfe.toml".to_vec()));
    let configure_args = vec![
        "collections".into(),
        "configure".into(),
        native_root.as_os_str().to_owned(),
        "--profile".into(),
        "native-compact".into(),
        "--pack".into(),
        "llg".into(),
        "--make-default".into(),
        "--config".into(),
        native_config.as_os_str().to_owned(),
    ];
    let configure = run_os(&configure_args);
    assert!(
        configure.status.success(),
        "{}",
        String::from_utf8_lossy(&configure.stderr)
    );
    assert!(native_config.exists());

    let lookup_args = vec![
        "lookup".into(),
        "farvi".into(),
        "--config".into(),
        native_config.as_os_str().to_owned(),
        "--profile".into(),
        "native-compact".into(),
        "--format".into(),
        "compact".into(),
    ];
    let lookup = run_os(&lookup_args);
    assert!(
        lookup.status.success(),
        "{}",
        String::from_utf8_lossy(&lookup.stderr)
    );
    let lookup: Value = serde_json::from_slice(&lookup.stdout).unwrap();
    let detail_ref = lookup["result"]["response"]["groups"][0]["hits"][0]["detail_ref"].clone();
    let expected_snapshot = lookup["snapshot"]["binding_sha256"].as_str().unwrap();
    let detail_args = vec![
        "lookup-detail".into(),
        "--reference".into(),
        detail_ref.to_string().into(),
        "--expected-snapshot".into(),
        expected_snapshot.into(),
        "--config".into(),
        native_config.as_os_str().to_owned(),
        "--profile".into(),
        "native-compact".into(),
    ];
    let detail = run_os(&detail_args);
    assert!(
        detail.status.success(),
        "{}",
        String::from_utf8_lossy(&detail.stderr)
    );
    let detail: Value = serde_json::from_slice(&detail.stdout).unwrap();
    assert_eq!(
        detail["response"]["hit"]["assertion"]["id"],
        detail_ref["assertion_id"]
    );

    let readable_lookup_args = vec![
        "lookup".into(),
        "farvi".into(),
        "--config".into(),
        native_config.as_os_str().to_owned(),
        "--profile".into(),
        "native-compact".into(),
        "--format".into(),
        "readable".into(),
    ];
    let readable_lookup = run_os(&readable_lookup_args);
    assert!(
        readable_lookup.status.success(),
        "{}",
        String::from_utf8_lossy(&readable_lookup.stderr)
    );
    assert!(
        String::from_utf8(readable_lookup.stdout)
            .unwrap()
            .starts_with("Exact \"farvi\": ")
    );
    let readable_detail_args = vec![
        "lookup-detail".into(),
        "--reference".into(),
        detail_ref.to_string().into(),
        "--expected-snapshot".into(),
        expected_snapshot.into(),
        "--config".into(),
        native_config.as_os_str().to_owned(),
        "--profile".into(),
        "native-compact".into(),
        "--format".into(),
        "readable".into(),
    ];
    let readable_detail = run_os(&readable_detail_args);
    assert!(
        readable_detail.status.success(),
        "{}",
        String::from_utf8_lossy(&readable_detail.stderr)
    );
    assert!(
        String::from_utf8(readable_detail.stdout)
            .unwrap()
            .starts_with("Detail ")
    );

    std::fs::remove_dir_all(&native_root).unwrap();
    let missing_root = run_os(&lookup_args);
    assert!(!missing_root.status.success());
    let error: Value = serde_json::from_slice(&missing_root.stderr).unwrap();
    assert_eq!(
        error["error"]["root_path"],
        serde_json::to_value(vlajmaji_store::native_path_report(&native_root)).unwrap()
    );
    assert_eq!(
        error["error"]["config_path"],
        serde_json::to_value(vlajmaji_store::native_path_report(&native_config)).unwrap()
    );
    let lossy_root = std::path::PathBuf::from(native_root.to_string_lossy().into_owned());
    assert!(!lossy_root.exists());
    std::fs::remove_dir_all(base).unwrap();
}

#[test]
fn test_managed_compact_lookup_and_detail_tolerate_broken_pipe() {
    let base = scratch();
    let root = base.join("catalogue");
    install(&root);
    let config = base.join("config.toml");
    let configure = run_strings(&[
        "collections",
        "configure",
        root.to_str().unwrap(),
        "--profile",
        "compact-test",
        "--pack",
        "llg",
        "--make-default",
        "--config",
        config.to_str().unwrap(),
    ]);
    assert!(
        configure.status.success(),
        "{}",
        String::from_utf8_lossy(&configure.stderr)
    );
    let lookup_args = [
        "lookup",
        "farvi",
        "--config",
        config.to_str().unwrap(),
        "--profile",
        "compact-test",
        "--format",
        "compact",
    ];
    let lookup = run_strings(&lookup_args);
    assert!(
        lookup.status.success(),
        "{}",
        String::from_utf8_lossy(&lookup.stderr)
    );
    let response: Value = serde_json::from_slice(&lookup.stdout).unwrap();
    let detail_ref =
        response["result"]["response"]["groups"][0]["hits"][0]["detail_ref"].to_string();
    let detail_args = [
        "lookup-detail",
        "--reference",
        detail_ref.as_str(),
        "--expected-snapshot",
        response["snapshot"]["binding_sha256"].as_str().unwrap(),
        "--config",
        config.to_str().unwrap(),
        "--profile",
        "compact-test",
    ];
    let detail = run_strings(&detail_args);
    assert!(
        detail.status.success(),
        "{}",
        String::from_utf8_lossy(&detail.stderr)
    );

    let readable_lookup_args = [
        "lookup",
        "farvi",
        "--config",
        config.to_str().unwrap(),
        "--profile",
        "compact-test",
        "--format",
        "readable",
    ];
    let readable_detail_args = [
        "lookup-detail",
        "--reference",
        detail_ref.as_str(),
        "--expected-snapshot",
        response["snapshot"]["binding_sha256"].as_str().unwrap(),
        "--config",
        config.to_str().unwrap(),
        "--profile",
        "compact-test",
        "--format",
        "readable",
    ];
    for args in [
        &lookup_args[..],
        &detail_args[..],
        &readable_lookup_args[..],
        &readable_detail_args[..],
    ] {
        let mut child = Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
            .args(args)
            .env_clear()
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        drop(child.stdout.take());
        let result = child.wait_with_output().unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert!(result.stderr.is_empty());
    }
    std::fs::remove_dir_all(base).unwrap();
}

#[cfg(target_os = "linux")]
#[test]
fn test_dictionary_compact_query_reports_full_device_output_failure() {
    let base = scratch();
    let store = base.join("dictionary-store");
    let dictionary = fixture::complete_fixture().unwrap();
    vlajmaji_store::DictionaryStore::create(&store, &dictionary).unwrap();
    let request = r#"{"operation":"exact","spelling":"farvi"}"#;

    let ordinary = run_strings(&[
        "dictionary-query",
        store.to_str().unwrap(),
        "--request",
        request,
        "--format",
        "compact",
    ]);
    assert!(
        ordinary.status.success(),
        "{}",
        String::from_utf8_lossy(&ordinary.stderr)
    );
    let response: Value = serde_json::from_slice(&ordinary.stdout).unwrap();
    assert!(response.get("operation").is_none());
    assert!(response.get("response").is_none());
    assert!(!response["groups"].as_array().unwrap().is_empty());
    let detail_ref = response["groups"][0]["hits"][0]["detail_ref"].to_string();
    let detail_args = [
        "dictionary-detail",
        store.to_str().unwrap(),
        "--reference",
        &detail_ref,
    ];
    let ordinary_detail = run_strings(&detail_args);
    assert!(
        ordinary_detail.status.success(),
        "{}",
        String::from_utf8_lossy(&ordinary_detail.stderr)
    );
    let detail: Value = serde_json::from_slice(&ordinary_detail.stdout).unwrap();
    assert!(detail["hit"].is_object());

    let full = std::fs::OpenOptions::new()
        .write(true)
        .open("/dev/full")
        .unwrap();
    let failed = Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
        .args([
            "dictionary-query",
            store.to_str().unwrap(),
            "--request",
            request,
            "--format",
            "compact",
        ])
        .env_clear()
        .stdout(full)
        .stderr(std::process::Stdio::piped())
        .output()
        .unwrap();
    assert!(!failed.status.success());
    let stderr = String::from_utf8_lossy(&failed.stderr);
    assert!(stderr.contains("write JSON:"), "{stderr}");
    assert!(stderr.contains("No space left on device"), "{stderr}");

    let readable_query_args = [
        "dictionary-query",
        store.to_str().unwrap(),
        "--request",
        request,
        "--format",
        "readable",
    ];
    let readable_query = run_strings(&readable_query_args);
    assert!(
        readable_query.status.success(),
        "{}",
        String::from_utf8_lossy(&readable_query.stderr)
    );
    assert!(
        String::from_utf8(readable_query.stdout)
            .unwrap()
            .starts_with("Exact \"farvi\": ")
    );
    let full = std::fs::OpenOptions::new()
        .write(true)
        .open("/dev/full")
        .unwrap();
    let failed_readable_query = Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
        .args(readable_query_args)
        .env_clear()
        .stdout(full)
        .stderr(std::process::Stdio::piped())
        .output()
        .unwrap();
    assert!(!failed_readable_query.status.success());
    let readable_query_envelope: Value =
        serde_json::from_slice(&failed_readable_query.stderr).unwrap();
    let readable_query_report = readable_query_envelope["error"].as_object().unwrap();
    assert_eq!(readable_query_report.len(), 3);
    assert_eq!(readable_query_report["code"], "response_output_io");
    assert_eq!(readable_query_report["sources"], serde_json::json!([]));
    assert!(
        readable_query_report["message"]
            .as_str()
            .unwrap()
            .contains("No space left on device")
    );

    let full = std::fs::OpenOptions::new()
        .write(true)
        .open("/dev/full")
        .unwrap();
    let failed_detail = Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
        .args(detail_args)
        .env_clear()
        .stdout(full)
        .stderr(std::process::Stdio::piped())
        .output()
        .unwrap();
    assert!(!failed_detail.status.success());
    let envelope: Value = serde_json::from_slice(&failed_detail.stderr).unwrap();
    let report = envelope["error"].as_object().unwrap();
    assert_eq!(report.len(), 3);
    assert_eq!(report["code"], "response_output_io");
    assert_eq!(report["sources"], serde_json::json!([]));
    assert!(
        report["message"]
            .as_str()
            .unwrap()
            .contains("No space left on device")
    );

    let readable_detail_args = [
        "dictionary-detail",
        store.to_str().unwrap(),
        "--reference",
        &detail_ref,
        "--format",
        "readable",
    ];
    let readable_detail = run_strings(&readable_detail_args);
    assert!(
        readable_detail.status.success(),
        "{}",
        String::from_utf8_lossy(&readable_detail.stderr)
    );
    assert!(
        String::from_utf8(readable_detail.stdout)
            .unwrap()
            .starts_with("Detail ")
    );
    let full = std::fs::OpenOptions::new()
        .write(true)
        .open("/dev/full")
        .unwrap();
    let failed_readable_detail = Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
        .args(readable_detail_args)
        .env_clear()
        .stdout(full)
        .stderr(std::process::Stdio::piped())
        .output()
        .unwrap();
    assert!(!failed_readable_detail.status.success());
    let readable_detail_envelope: Value =
        serde_json::from_slice(&failed_readable_detail.stderr).unwrap();
    let readable_detail_report = readable_detail_envelope["error"].as_object().unwrap();
    assert_eq!(readable_detail_report.len(), 3);
    assert_eq!(readable_detail_report["code"], "response_output_io");
    assert_eq!(readable_detail_report["sources"], serde_json::json!([]));
    assert!(
        readable_detail_report["message"]
            .as_str()
            .unwrap()
            .contains("No space left on device")
    );
    std::fs::remove_dir_all(base).unwrap();
}

#[cfg(target_os = "linux")]
#[test]
fn test_managed_compact_lookup_and_detail_report_full_device_envelopes() {
    let base = scratch();
    let root = base.join("catalogue");
    let store_path = install(&root);
    let config = base.join("config.toml");
    let configure = run_strings(&[
        "collections",
        "configure",
        root.to_str().unwrap(),
        "--profile",
        "compact-test",
        "--pack",
        "llg",
        "--make-default",
        "--config",
        config.to_str().unwrap(),
    ]);
    assert!(
        configure.status.success(),
        "{}",
        String::from_utf8_lossy(&configure.stderr)
    );

    let lookup_args = [
        "lookup",
        "farvi",
        "--config",
        config.to_str().unwrap(),
        "--profile",
        "compact-test",
        "--format",
        "compact",
    ];
    let lookup = run_strings(&lookup_args);
    assert!(
        lookup.status.success(),
        "{}",
        String::from_utf8_lossy(&lookup.stderr)
    );
    let lookup: Value = serde_json::from_slice(&lookup.stdout).unwrap();
    assert_eq!(lookup["result"]["operation"], "exact");
    let detail_ref = lookup["result"]["response"]["groups"][0]["hits"][0]["detail_ref"].clone();
    let detail_ref_text = detail_ref.to_string();
    let detail_args = [
        "lookup-detail",
        "--reference",
        &detail_ref_text,
        "--expected-snapshot",
        lookup["snapshot"]["binding_sha256"].as_str().unwrap(),
        "--config",
        config.to_str().unwrap(),
        "--profile",
        "compact-test",
    ];
    let detail = run_strings(&detail_args);
    assert!(
        detail.status.success(),
        "{}",
        String::from_utf8_lossy(&detail.stderr)
    );
    let detail: Value = serde_json::from_slice(&detail.stdout).unwrap();
    assert_eq!(
        detail["response"]["hit"]["assertion"]["id"],
        detail_ref["assertion_id"]
    );

    let readable_lookup_args = [
        "lookup",
        "farvi",
        "--config",
        config.to_str().unwrap(),
        "--profile",
        "compact-test",
        "--format",
        "readable",
    ];
    let readable_lookup = run_strings(&readable_lookup_args);
    assert!(readable_lookup.status.success());
    assert!(
        String::from_utf8(readable_lookup.stdout)
            .unwrap()
            .starts_with("Exact \"farvi\": ")
    );
    let readable_detail_args = [
        "lookup-detail",
        "--reference",
        &detail_ref_text,
        "--expected-snapshot",
        lookup["snapshot"]["binding_sha256"].as_str().unwrap(),
        "--config",
        config.to_str().unwrap(),
        "--profile",
        "compact-test",
        "--format",
        "readable",
    ];
    let readable_detail = run_strings(&readable_detail_args);
    assert!(readable_detail.status.success());
    assert!(
        String::from_utf8(readable_detail.stdout)
            .unwrap()
            .starts_with("Detail ")
    );

    let native_path = |path: &Path| {
        let bytes = path.as_os_str().as_encoded_bytes();
        serde_json::json!({
            "encoding": "unix-bytes",
            "hex": bytes.iter().map(|byte| format!("{byte:02x}")).collect::<String>(),
            "utf8": path.to_str(),
        })
    };
    let assert_managed_error =
        |failed: &Output, operation: &str, expected_snapshot: &Value, expected_packs: &Value| {
            assert!(!failed.status.success());
            let envelope: Value = serde_json::from_slice(&failed.stderr).unwrap();
            let report = &envelope["error"];
            for key in [
                "version",
                "ok",
                "category",
                "detail",
                "operation",
                "config_path",
                "root_path",
                "profile",
                "requested_profile",
                "snapshot",
                "known_pack_ids",
                "outcome",
                "writer",
            ] {
                assert!(report.get(key).is_some(), "missing report key {key}");
            }
            assert_eq!(report["version"], 1);
            assert_eq!(report["ok"], false);
            assert_eq!(report["category"], "response_output_io");
            assert_eq!(report["operation"], operation);
            assert_eq!(report["config_path"], native_path(&config));
            assert_eq!(report["root_path"], native_path(&root));
            assert_eq!(report["requested_profile"], "compact-test");
            assert_eq!(report["known_pack_ids"], expected_packs);
            assert_eq!(report["outcome"], "not_applicable");
            assert_eq!(report["snapshot"], expected_snapshot);
            assert!(report["profile"].is_object());
            assert!(report["writer"].is_null());
            assert!(
                report["detail"]
                    .as_str()
                    .unwrap()
                    .contains("No space left on device")
            );
        };

    let full_device = || {
        std::fs::OpenOptions::new()
            .write(true)
            .open("/dev/full")
            .unwrap()
    };
    let failed_lookup = Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
        .args(&lookup_args)
        .env_clear()
        .stdout(full_device())
        .stderr(std::process::Stdio::piped())
        .output()
        .unwrap();
    assert_managed_error(
        &failed_lookup,
        "lookup",
        &lookup["snapshot"],
        &lookup["snapshot"]["collections"]
            .as_array()
            .unwrap()
            .iter()
            .map(|row| row["pack_id"].clone())
            .collect::<Value>(),
    );

    let failed_detail = Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
        .args(&detail_args)
        .env_clear()
        .stdout(full_device())
        .stderr(std::process::Stdio::piped())
        .output()
        .unwrap();
    assert_managed_error(
        &failed_detail,
        "lookup_detail",
        &detail["snapshot"],
        &detail["snapshot"]["collections"]
            .as_array()
            .unwrap()
            .iter()
            .map(|row| row["pack_id"].clone())
            .collect::<Value>(),
    );

    let failed_readable_lookup = Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
        .args(&readable_lookup_args)
        .env_clear()
        .stdout(full_device())
        .stderr(std::process::Stdio::piped())
        .output()
        .unwrap();
    assert_managed_error(
        &failed_readable_lookup,
        "lookup",
        &lookup["snapshot"],
        &lookup["snapshot"]["collections"]
            .as_array()
            .unwrap()
            .iter()
            .map(|row| row["pack_id"].clone())
            .collect::<Value>(),
    );

    let failed_readable_detail = Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
        .args(&readable_detail_args)
        .env_clear()
        .stdout(full_device())
        .stderr(std::process::Stdio::piped())
        .output()
        .unwrap();
    assert_managed_error(
        &failed_readable_detail,
        "lookup_detail",
        &detail["snapshot"],
        &detail["snapshot"]["collections"]
            .as_array()
            .unwrap()
            .iter()
            .map(|row| row["pack_id"].clone())
            .collect::<Value>(),
    );
    assert!(store_path.exists());
    std::fs::remove_dir_all(base).unwrap();
}
