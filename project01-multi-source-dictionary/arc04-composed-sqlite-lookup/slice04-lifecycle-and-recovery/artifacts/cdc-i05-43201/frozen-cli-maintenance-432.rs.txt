//! CLI lifecycle maintenance controls for explicit removal and restoration.

use std::{
    borrow::Cow,
    fs,
    fs::OpenOptions,
    io::Write,
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use serde_json::Value;
use sha2::{Digest, Sha256};
use vlajmaji_lexicon::{Dictionary, PackId, PackPayloadKind, seal_release};
use vlajmaji_store::{
    CollectionCatalogue, CollectionInput, CollectionProfileWriteRequest, CollectionProfiles,
    CollectionRequest, CollectionSelection, DictionaryStore,
};

fn fresh_root() -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "vlajmaji-s04-cli-maintenance-{}-{stamp}",
        std::process::id()
    ));
    fs::create_dir(&root).unwrap();
    root
}

fn run(args: &[&std::ffi::OsStr]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
        .args(args)
        .env_clear()
        .output()
        .expect("run lifecycle command")
}

fn run_owned(args: &[std::ffi::OsString]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
        .args(args)
        .env_clear()
        .output()
        .expect("run lifecycle command")
}

fn json(output: &std::process::Output) -> Value {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

fn json_owned(output: &std::process::Output) -> Value {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[test]
fn test_collections_remove_reinstall_reconcile_cli_preserves_explicit_state() {
    let root = fresh_root();
    let dictionary = Dictionary::from_document(Default::default()).unwrap();
    let sealed = seal_release(PackId::Llg, PackPayloadKind::Synthetic, &dictionary).unwrap();
    let release_path = root.join("release.json");
    let seal_path = root.join("seal.json");
    fs::write(&release_path, &sealed.release).unwrap();
    fs::write(&seal_path, &sealed.seal).unwrap();
    let release_sha256 = sha256(&sealed.release);
    let catalogue = root.join("catalogue");

    let installed = json(&run(&[
        "collections".as_ref(),
        "install".as_ref(),
        catalogue.as_os_str(),
        "--pack".as_ref(),
        "llg".as_ref(),
        "--release".as_ref(),
        release_path.as_os_str(),
        "--seal".as_ref(),
        seal_path.as_os_str(),
        "--allow-synthetic".as_ref(),
    ]));
    let first_generation = installed["generation"]["generation_id"].clone();

    let removed = json(&run(&[
        "collections".as_ref(),
        "remove".as_ref(),
        catalogue.as_os_str(),
        "--pack".as_ref(),
        "llg".as_ref(),
        "--expected-revision".as_ref(),
        "1".as_ref(),
    ]));
    assert_eq!(removed["operation"], "remove");
    assert_eq!(removed["outcome"], "removed");
    assert_eq!(removed["local_revision"], 2);
    assert!(removed["generation"].is_null());

    let implicit_install = run(&[
        "collections".as_ref(),
        "install".as_ref(),
        catalogue.as_os_str(),
        "--pack".as_ref(),
        "llg".as_ref(),
        "--release".as_ref(),
        release_path.as_os_str(),
        "--seal".as_ref(),
        seal_path.as_os_str(),
        "--allow-synthetic".as_ref(),
    ]);
    let failure = serde_json::from_slice::<Value>(&implicit_install.stderr).unwrap();
    assert_eq!(failure["error"]["category"], "reinstall_required");
    assert_eq!(failure["error"]["operation"], "install");
    assert_eq!(failure["error"]["activation"], "not_activated");

    let unknown_release = "0".repeat(64);
    let unknown_reinstall = run(&[
        "collections".as_ref(),
        "reinstall".as_ref(),
        catalogue.as_os_str(),
        "--pack".as_ref(),
        "llg".as_ref(),
        "--release".as_ref(),
        release_path.as_os_str(),
        "--seal".as_ref(),
        seal_path.as_os_str(),
        "--allow-synthetic".as_ref(),
        "--release-sha256".as_ref(),
        unknown_release.as_str().as_ref(),
        "--expected-revision".as_ref(),
        "2".as_ref(),
    ]);
    assert!(!unknown_reinstall.status.success());
    let unknown_report: Value = serde_json::from_slice(&unknown_reinstall.stderr).unwrap();
    let unknown_report = &unknown_report["error"];
    assert_eq!(unknown_report["operation"], "reinstall");
    assert_eq!(unknown_report["category"], "unknown_registered_release");
    assert_eq!(unknown_report["pack_id"], "llg");
    assert_eq!(unknown_report["prior_local_revision"], 2);
    assert!(unknown_report["attempt_id"].is_null());
    assert!(unknown_report["proposed_generation"].is_null());
    assert!(unknown_report["observed_generation"].is_null());
    assert_eq!(unknown_report["activation"], "not_activated");
    assert_eq!(unknown_report["inputs"].as_array().unwrap().len(), 1);
    assert_eq!(unknown_report["inputs"][0]["disposition"], "validated");

    let reinstalled = json(&run(&[
        "collections".as_ref(),
        "reinstall".as_ref(),
        catalogue.as_os_str(),
        "--pack".as_ref(),
        "llg".as_ref(),
        "--release".as_ref(),
        release_path.as_os_str(),
        "--seal".as_ref(),
        seal_path.as_os_str(),
        "--allow-synthetic".as_ref(),
        "--release-sha256".as_ref(),
        release_sha256.as_str().as_ref(),
        "--expected-revision".as_ref(),
        "2".as_ref(),
    ]));
    assert_eq!(reinstalled["operation"], "reinstall");
    assert_eq!(reinstalled["outcome"], "reinstalled");
    assert_eq!(reinstalled["local_revision"], 3);
    assert_ne!(reinstalled["generation"]["generation_id"], first_generation);

    let reconciled = json(&run(&[
        "collections".as_ref(),
        "reconcile".as_ref(),
        catalogue.as_os_str(),
    ]));
    assert_eq!(reconciled["operation"], "reconcile");
    assert_eq!(reconciled["local_revision"], 3);
    assert_eq!(
        reconciled["active_generation"],
        reinstalled["generation"]["generation_id"]
    );
    assert_eq!(reconciled["sqlite_recovery"], "writable_open_completed");
    let _ = fs::remove_dir_all(root);
}

#[test]
fn test_cli_policy_commands_preserve_context_and_apply_partial_changes() {
    let root = fresh_root();
    let catalogue = root.join("catalogue");
    let dictionary = Dictionary::from_document(Default::default()).unwrap();
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
    CollectionCatalogue::register(&catalogue, request).unwrap();
    let config = root.join("profiles.toml");
    CollectionProfiles::configure(
        &config,
        CollectionProfileWriteRequest {
            name: "research".into(),
            root: catalogue.clone(),
            packs: vec![PackId::Llg],
            disabled: Vec::new(),
            make_default: true,
        },
    )
    .unwrap();
    let disabled = json(&run(&[
        "collections".as_ref(),
        "disable".as_ref(),
        "--config".as_ref(),
        config.as_os_str(),
        "--profile".as_ref(),
        "research".as_ref(),
        "--pack".as_ref(),
        "llg".as_ref(),
    ]));
    assert_eq!(disabled["version"], 1);
    assert!(disabled["ok"].as_bool().unwrap());
    assert_eq!(disabled["operation"], "disable");
    assert_eq!(disabled["outcome"], "committed");
    assert_eq!(disabled["affected_pack_ids"], serde_json::json!(["llg"]));
    assert_eq!(
        disabled["root"],
        serde_json::to_value(vlajmaji_store::native_path_report(&catalogue)).unwrap()
    );

    let enabled = json(&run(&[
        "collections".as_ref(),
        "enable".as_ref(),
        "--config".as_ref(),
        config.as_os_str(),
        "--profile".as_ref(),
        "research".as_ref(),
        "--all".as_ref(),
    ]));
    assert_eq!(enabled["operation"], "enable");
    assert_eq!(enabled["outcome"], "committed");
    assert_eq!(enabled["disabled"], serde_json::json!([]));

    let nonmember = run(&[
        "collections".as_ref(),
        "enable".as_ref(),
        "--config".as_ref(),
        config.as_os_str(),
        "--profile".as_ref(),
        "research".as_ref(),
        "--pack".as_ref(),
        "sutysisku".as_ref(),
    ]);
    assert!(!nonmember.status.success());
    let envelope: Value = serde_json::from_slice(&nonmember.stderr).unwrap();
    let report = &envelope["error"];
    assert_eq!(report["version"], 1);
    assert!(!report["ok"].as_bool().unwrap());
    assert_eq!(report["category"], "not_profile_member");
    assert_eq!(report["operation"], "enable");
    assert_eq!(report["requested_profile"], "research");
    assert_eq!(
        report["config_path"],
        serde_json::to_value(vlajmaji_store::native_path_report(&config)).unwrap()
    );

    let cleared = json(&run(&[
        "collections".as_ref(),
        "default".as_ref(),
        "--config".as_ref(),
        config.as_os_str(),
        "--clear".as_ref(),
    ]));
    assert_eq!(cleared["operation"], "clear_default");
    assert_eq!(cleared["outcome"], "committed");
    assert_eq!(cleared["default_profile"], Value::Null);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn test_managed_default_scratch_transition_preserves_legacy_default_store() {
    let root = fresh_root();
    let home = root.join("home");
    let xdg = root.join("xdg");
    fs::create_dir(&home).unwrap();
    fs::create_dir(&xdg).unwrap();
    let legacy_path = xdg.join("vlajmaji/stores/default");
    fs::create_dir_all(legacy_path.parent().unwrap()).unwrap();
    let legacy_input = root.join("lensisku-fixture");
    let materialized = Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
        .args([
            "data-pack",
            "fixture",
            "jbotci",
            legacy_input.to_str().unwrap(),
        ])
        .env_clear()
        .output()
        .unwrap();
    assert!(
        materialized.status.success(),
        "{}",
        String::from_utf8_lossy(&materialized.stderr)
    );
    let lensisku_release = fs::read(legacy_input.join("release.json")).unwrap();
    let dictionary = Dictionary::from_json_bytes(&lensisku_release).unwrap();
    DictionaryStore::create(&legacy_path, &dictionary).unwrap();
    let legacy_db = legacy_path.join("dictionary.sqlite3");
    let legacy_bytes = fs::read(&legacy_db).unwrap();
    let query = r#"{"operation":"exact","spelling":"farvi"}"#;

    let legacy_before = Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
        .args(["dictionary-query", "--request", query])
        .env_clear()
        .env("HOME", &home)
        .env("XDG_DATA_HOME", &xdg)
        .output()
        .unwrap();
    assert!(legacy_before.status.success());
    let legacy_json: Value = serde_json::from_slice(&legacy_before.stdout).unwrap();
    assert!(legacy_json["totals"]["occurrences"].as_u64().unwrap() > 0);

    let managed_root = root.join("managed-catalogue");
    let managed_pack = PackId::LensiskuJbotci;
    let sealed = seal_release(managed_pack, PackPayloadKind::Synthetic, &dictionary).unwrap();
    let registration = CollectionRequest::new(
        vec![CollectionInput::SealedBytes {
            pack_id: managed_pack,
            release: Cow::Borrowed(&sealed.release),
            seal: Cow::Borrowed(&sealed.seal),
        }],
        CollectionSelection::Packs(vec![managed_pack]),
        true,
    )
    .unwrap();
    CollectionCatalogue::register(&managed_root, registration.clone()).unwrap();
    CollectionCatalogue::install(&managed_root, registration).unwrap();
    let config = root.join("managed-config.toml");
    CollectionProfiles::configure(
        &config,
        CollectionProfileWriteRequest {
            name: "scratch".into(),
            root: managed_root.clone(),
            packs: vec![managed_pack],
            disabled: Vec::new(),
            make_default: false,
        },
    )
    .unwrap();
    let config_text = fs::read_to_string(&config).unwrap();
    let relative_config = config_text.replace(managed_root.to_str().unwrap(), "managed-catalogue");
    assert_ne!(relative_config, config_text);
    fs::write(&config, relative_config).unwrap();
    assert_eq!(
        CollectionProfiles::read(&config)
            .unwrap()
            .profile(Some("scratch"))
            .unwrap()
            .root,
        managed_root
    );

    let set_managed_default = Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
        .args([
            "collections",
            "default",
            "--profile",
            "scratch",
            "--config",
            config.to_str().unwrap(),
        ])
        .env_clear()
        .output()
        .unwrap();
    assert!(set_managed_default.status.success());
    let managed_lookup = Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
        .args(["lookup", "farvi", "--config", config.to_str().unwrap()])
        .env_clear()
        .output()
        .unwrap();
    assert!(
        managed_lookup.status.success(),
        "{}",
        String::from_utf8_lossy(&managed_lookup.stderr)
    );
    let managed_json: Value = serde_json::from_slice(&managed_lookup.stdout).unwrap();
    assert!(
        managed_json["result"]["response"]["totals"]["occurrences"]
            .as_u64()
            .unwrap()
            > 0
    );

    let clear_managed_default = Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
        .args([
            "collections",
            "default",
            "--clear",
            "--config",
            config.to_str().unwrap(),
        ])
        .env_clear()
        .output()
        .unwrap();
    assert!(clear_managed_default.status.success());
    let visible_before_crashed_writer = fs::read(&config).unwrap();
    let original_lock = config.with_file_name("managed-config.toml.lock");
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&original_lock)
        .unwrap()
        .write_all(b"retained interrupted-writer lock")
        .unwrap();
    let occupied_alt = root.join("occupied-config-alt.toml");
    fs::write(&occupied_alt, b"preserve occupied ALT").unwrap();
    assert!(matches!(
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&occupied_alt)
            .map(|_| ()),
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists
    ));
    assert_eq!(fs::read(&occupied_alt).unwrap(), b"preserve occupied ALT");
    let alt_config = root.join("managed-config-alt.toml");
    let mut alt = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&alt_config)
        .unwrap();
    alt.write_all(&visible_before_crashed_writer).unwrap();
    alt.sync_all().unwrap();
    drop(alt);
    fs::File::open(&root).unwrap().sync_all().unwrap();
    assert_eq!(
        CollectionProfiles::read(&alt_config)
            .unwrap()
            .profile(Some("scratch"))
            .unwrap()
            .root,
        managed_root
    );
    let original_write = Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
        .args([
            "collections",
            "default",
            "--profile",
            "scratch",
            "--config",
            config.to_str().unwrap(),
        ])
        .env_clear()
        .output()
        .unwrap();
    assert!(!original_write.status.success());
    assert!(original_write.stdout.is_empty());
    let original_error: Value = serde_json::from_slice(&original_write.stderr).unwrap();
    assert_eq!(original_error["error"]["category"], "config_busy");
    assert_eq!(fs::read(&config).unwrap(), visible_before_crashed_writer);
    let alt_default = Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
        .args([
            "collections",
            "default",
            "--profile",
            "scratch",
            "--config",
            alt_config.to_str().unwrap(),
        ])
        .env_clear()
        .output()
        .unwrap();
    assert!(alt_default.status.success());
    let alt_lookup = Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
        .args([
            "lookup",
            "farvi",
            "--profile",
            "scratch",
            "--config",
            alt_config.to_str().unwrap(),
        ])
        .env_clear()
        .output()
        .unwrap();
    assert!(
        alt_lookup.status.success(),
        "{}",
        String::from_utf8_lossy(&alt_lookup.stderr)
    );
    let alt_result: Value = serde_json::from_slice(&alt_lookup.stdout).unwrap();
    assert!(
        alt_result["result"]["response"]["totals"]["occurrences"]
            .as_u64()
            .unwrap()
            > 0
    );
    assert!(original_lock.exists());
    let legacy_after = Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
        .args(["dictionary-query", "--request", query])
        .env_clear()
        .env("HOME", &home)
        .env("XDG_DATA_HOME", &xdg)
        .output()
        .unwrap();
    assert!(legacy_after.status.success());
    assert_eq!(legacy_after.stdout, legacy_before.stdout);
    assert_eq!(fs::read(&legacy_db).unwrap(), legacy_bytes);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn test_four_pack_synthetic_update_and_rebuild_preserve_all_members() {
    let root = fresh_root();
    let catalogue = root.join("catalogue");
    let mut records = Vec::new();
    for (fixture, pack, label) in [
        ("jbotci", "lensisku-jbotci", "jb"),
        ("llg", "llg", "llg"),
        ("la-lojban", "la-lojban", "la"),
        ("sutysisku", "sutysisku", "sut"),
    ] {
        let output = root.join(format!("{label}-fixture"));
        let args = vec![
            "data-pack".into(),
            "fixture".into(),
            fixture.into(),
            output.as_os_str().to_owned(),
        ];
        let materialized = run_owned(&args);
        assert!(
            materialized.status.success(),
            "{fixture}: {}",
            String::from_utf8_lossy(&materialized.stderr)
        );
        records.push(serde_json::json!({
            "kind": "sealed",
            "pack_id": pack,
            "release": output.join("release.json"),
            "seal": output.join("seal.json"),
        }));
    }
    let manifest = root.join("packs.json");
    fs::write(
        &manifest,
        serde_json::to_vec(&serde_json::json!({"version": 1, "inputs": records})).unwrap(),
    )
    .unwrap();
    let register = run_owned(&[
        "collections".into(),
        "register".into(),
        catalogue.as_os_str().to_owned(),
        "--manifest".into(),
        manifest.as_os_str().to_owned(),
        "--all".into(),
        "--allow-synthetic".into(),
    ]);
    assert!(
        register.status.success(),
        "{}",
        String::from_utf8_lossy(&register.stderr)
    );
    let install = run_owned(&[
        "collections".into(),
        "install".into(),
        catalogue.as_os_str().to_owned(),
        "--manifest".into(),
        manifest.as_os_str().to_owned(),
        "--all".into(),
        "--allow-synthetic".into(),
    ]);
    let installed = json_owned(&install);
    assert_eq!(
        installed["generation"]["members"].as_array().unwrap().len(),
        4
    );
    assert_eq!(installed["inputs"].as_array().unwrap().len(), 4);

    let sut_dir = root.join("sut-fixture");
    let sut_release_path = sut_dir.join("release.json");
    let sut_seal_path = sut_dir.join("seal.json");
    let initial_release = fs::read(&sut_release_path).unwrap();
    let initial_dictionary = Dictionary::from_json_bytes(&initial_release).unwrap();
    let mut changed_document = initial_dictionary.into_document();
    changed_document.metadata.catalogue_revision += 1;
    let changed_dictionary = Dictionary::from_document(changed_document).unwrap();
    let changed_sealed = seal_release(
        PackId::Sutysisku,
        PackPayloadKind::Synthetic,
        &changed_dictionary,
    )
    .unwrap();
    fs::write(&sut_release_path, &changed_sealed.release).unwrap();
    fs::write(&sut_seal_path, &changed_sealed.seal).unwrap();
    let updated_manifest = root.join("updated-packs.json");
    let mut updated_records = records;
    let sut_record = updated_records
        .iter_mut()
        .find(|record| record["pack_id"] == "sutysisku")
        .unwrap();
    sut_record["release"] = serde_json::json!(sut_release_path);
    sut_record["seal"] = serde_json::json!(sut_seal_path);
    fs::write(
        &updated_manifest,
        serde_json::to_vec(&serde_json::json!({"version": 1, "inputs": updated_records})).unwrap(),
    )
    .unwrap();
    let update = run_owned(&[
        "collections".into(),
        "update".into(),
        catalogue.as_os_str().to_owned(),
        "--manifest".into(),
        updated_manifest.as_os_str().to_owned(),
        "--pack".into(),
        "sutysisku".into(),
        "--allow-synthetic".into(),
        "--expected-revision".into(),
        "2".into(),
    ]);
    let updated = json_owned(&update);
    assert_eq!(updated["outcome"], "updated");
    assert_eq!(updated["local_revision"], 3);
    assert_eq!(
        updated["generation"]["members"].as_array().unwrap().len(),
        4
    );

    let rebuild = run_owned(&[
        "collections".into(),
        "rebuild".into(),
        catalogue.as_os_str().to_owned(),
        "--expected-revision".into(),
        "3".into(),
    ]);
    let rebuilt = json_owned(&rebuild);
    assert_eq!(rebuilt["outcome"], "rebuilt");
    assert_eq!(rebuilt["local_revision"], 4);
    assert_eq!(
        rebuilt["generation"]["members"].as_array().unwrap().len(),
        4
    );
    assert_eq!(
        rebuilt["generation"]["members"]
            .as_array()
            .unwrap()
            .iter()
            .map(|member| member["pack_id"].as_str().unwrap())
            .collect::<Vec<_>>(),
        vec!["lensisku-jbotci", "llg", "la-lojban", "sutysisku"]
    );
    let _ = fs::remove_dir_all(root);
}

#[cfg(target_os = "linux")]
#[test]
fn test_linux_native_root_and_config_roundtrip_through_cli() {
    use std::os::unix::ffi::OsStringExt;

    let base = fresh_root();
    let catalogue = base.join(std::ffi::OsString::from_vec(b"catalogue-\xff".to_vec()));
    let config = base.join(std::ffi::OsString::from_vec(b"profiles-\xfe.toml".to_vec()));
    let release = base.join(std::ffi::OsString::from_vec(b"release-\xfd.json".to_vec()));
    let seal = base.join(std::ffi::OsString::from_vec(b"seal-\xfc.json".to_vec()));
    let fixture = base.join("llg-fixture");
    let materialized = run_owned(&[
        "data-pack".into(),
        "fixture".into(),
        "llg".into(),
        fixture.as_os_str().to_owned(),
    ]);
    assert!(
        materialized.status.success(),
        "{}",
        String::from_utf8_lossy(&materialized.stderr)
    );
    let release_bytes = fs::read(fixture.join("release.json")).unwrap();
    let seal_bytes = fs::read(fixture.join("seal.json")).unwrap();
    let checked =
        vlajmaji_lexicon::load_sealed_release(PackId::Llg, &release_bytes, &seal_bytes).unwrap();
    let selected_available_releases: Vec<_> = checked
        .metadata()
        .releases
        .iter()
        .filter(|release| {
            release.selected && release.availability == vlajmaji_lexicon::Availability::Available
        })
        .collect();
    assert!(!selected_available_releases.is_empty());
    let farvi_entries: Vec<_> = checked
        .metadata()
        .entries
        .iter()
        .filter(|entry| {
            matches!(
                &entry.spelling,
                vlajmaji_lexicon::Presence::Value(spelling) if spelling == "farvi"
            )
        })
        .collect();
    assert_eq!(farvi_entries.len(), 1);
    let farvi_entry = farvi_entries[0];
    let farvi_assertions: Vec<_> = checked
        .metadata()
        .assertions
        .iter()
        .filter(|assertion| {
            assertion.entry == farvi_entry.id
                && assertion.eligibility == vlajmaji_lexicon::Eligibility::Lexical
        })
        .collect();
    assert_eq!(farvi_assertions.len(), 1);
    let expected_assertion_id = serde_json::to_value(&farvi_assertions[0].id).unwrap();
    assert!(
        selected_available_releases
            .iter()
            .any(|release| release.assertions.contains(&farvi_assertions[0].id))
    );
    drop(checked);
    fs::write(&release, &release_bytes).unwrap();
    fs::write(&seal, &seal_bytes).unwrap();
    assert_eq!(fs::read(&release).unwrap(), release_bytes);
    assert_eq!(fs::read(&seal).unwrap(), seal_bytes);

    let register = run_owned(&[
        "collections".into(),
        "register".into(),
        catalogue.as_os_str().to_owned(),
        "--pack".into(),
        "llg".into(),
        "--release".into(),
        release.as_os_str().to_owned(),
        "--seal".into(),
        seal.as_os_str().to_owned(),
        "--allow-synthetic".into(),
    ]);
    assert!(
        register.status.success(),
        "{}",
        String::from_utf8_lossy(&register.stderr)
    );
    assert!(catalogue.join("catalogue.sqlite3").is_file());

    let install = run_owned(&[
        "collections".into(),
        "install".into(),
        catalogue.as_os_str().to_owned(),
        "--pack".into(),
        "llg".into(),
        "--release".into(),
        release.as_os_str().to_owned(),
        "--seal".into(),
        seal.as_os_str().to_owned(),
        "--allow-synthetic".into(),
    ]);
    assert_eq!(
        json_owned(&install)["generation"]["members"]
            .as_array()
            .unwrap()
            .len(),
        1
    );

    let configure = run_owned(&[
        "collections".into(),
        "configure".into(),
        catalogue.as_os_str().to_owned(),
        "--profile".into(),
        "native".into(),
        "--pack".into(),
        "llg".into(),
        "--make-default".into(),
        "--config".into(),
        config.as_os_str().to_owned(),
    ]);
    assert!(
        configure.status.success(),
        "{}",
        String::from_utf8_lossy(&configure.stderr)
    );
    let profiles = run_owned(&[
        "collections".into(),
        "profiles".into(),
        "--config".into(),
        config.as_os_str().to_owned(),
    ]);
    let profiles = json_owned(&profiles);
    assert_eq!(
        profiles["path"],
        serde_json::to_value(vlajmaji_store::native_path_report(&config)).unwrap()
    );
    assert!(profiles["path"]["utf8"].is_null());
    assert_eq!(
        profiles["profiles"][0]["root"],
        serde_json::to_value(vlajmaji_store::native_path_report(&catalogue)).unwrap()
    );
    assert!(profiles["profiles"][0]["root"]["utf8"].is_null());

    let lookup = run_owned(&[
        "lookup".into(),
        "farvi".into(),
        "--config".into(),
        config.as_os_str().to_owned(),
        "--profile".into(),
        "native".into(),
    ]);
    let lookup = json_owned(&lookup);
    let response = &lookup["result"]["response"];
    assert_eq!(response["totals"]["occurrences"], 1);
    assert_eq!(response["totals"]["source_groups"], 1);
    assert_eq!(response["source_totals"].as_array().unwrap().len(), 1);
    assert_eq!(response["groups"].as_array().unwrap().len(), 1);
    assert_eq!(response["groups"][0]["source"]["id"], "llg");
    assert_eq!(response["groups"][0]["hits"].as_array().unwrap().len(), 1);
    assert_eq!(
        response["groups"][0]["hits"][0]["assertion"]["id"],
        expected_assertion_id
    );

    let disabled = run_owned(&[
        "collections".into(),
        "disable".into(),
        "--config".into(),
        config.as_os_str().to_owned(),
        "--profile".into(),
        "native".into(),
        "--pack".into(),
        "llg".into(),
    ]);
    assert_eq!(json_owned(&disabled)["outcome"], "committed");
    let enabled = run_owned(&[
        "collections".into(),
        "enable".into(),
        "--config".into(),
        config.as_os_str().to_owned(),
        "--profile".into(),
        "native".into(),
        "--pack".into(),
        "llg".into(),
    ]);
    assert_eq!(json_owned(&enabled)["outcome"], "committed");
    let cleared = run_owned(&[
        "collections".into(),
        "default".into(),
        "--config".into(),
        config.as_os_str().to_owned(),
        "--clear".into(),
    ]);
    assert_eq!(json_owned(&cleared)["operation"], "clear_default");
    let set_default = run_owned(&[
        "collections".into(),
        "default".into(),
        "--config".into(),
        config.as_os_str().to_owned(),
        "--profile".into(),
        "native".into(),
    ]);
    assert_eq!(json_owned(&set_default)["operation"], "set_default");

    let removed = run_owned(&[
        "collections".into(),
        "remove".into(),
        catalogue.as_os_str().to_owned(),
        "--pack".into(),
        "llg".into(),
        "--expected-revision".into(),
        "2".into(),
    ]);
    assert_eq!(json_owned(&removed)["outcome"], "removed");
    let registered_sha = sha256(&release_bytes);
    let reinstalled = run_owned(&[
        "collections".into(),
        "reinstall".into(),
        catalogue.as_os_str().to_owned(),
        "--pack".into(),
        "llg".into(),
        "--release".into(),
        release.as_os_str().to_owned(),
        "--seal".into(),
        seal.as_os_str().to_owned(),
        "--allow-synthetic".into(),
        "--release-sha256".into(),
        registered_sha.into(),
        "--expected-revision".into(),
        "3".into(),
    ]);
    assert_eq!(json_owned(&reinstalled)["outcome"], "reinstalled");
    let reconciled = run_owned(&[
        "collections".into(),
        "reconcile".into(),
        catalogue.as_os_str().to_owned(),
    ]);
    let reconciled = json_owned(&reconciled);
    assert_eq!(reconciled["catalogue_schema_version"], 2);
    assert_eq!(
        reconciled["root_path"],
        serde_json::to_value(vlajmaji_store::native_path_report(&catalogue)).unwrap()
    );
    assert!(reconciled["root_path"]["utf8"].is_null());
    let rebuilt = run_owned(&[
        "collections".into(),
        "rebuild".into(),
        catalogue.as_os_str().to_owned(),
        "--expected-revision".into(),
        "4".into(),
    ]);
    assert_eq!(json_owned(&rebuilt)["outcome"], "rebuilt");
    let updated = run_owned(&[
        "collections".into(),
        "update".into(),
        catalogue.as_os_str().to_owned(),
        "--pack".into(),
        "llg".into(),
        "--release".into(),
        release.as_os_str().to_owned(),
        "--seal".into(),
        seal.as_os_str().to_owned(),
        "--allow-synthetic".into(),
        "--expected-revision".into(),
        "5".into(),
    ]);
    assert_eq!(json_owned(&updated)["outcome"], "idempotent");
    let migrated = run_owned(&[
        "collections".into(),
        "migrate".into(),
        catalogue.as_os_str().to_owned(),
        "--expected-revision".into(),
        "5".into(),
    ]);
    assert_eq!(json_owned(&migrated)["outcome"], "idempotent");
    assert_eq!(fs::read(&release).unwrap(), release_bytes);
    assert_eq!(fs::read(&seal).unwrap(), seal_bytes);
    let _ = fs::remove_dir_all(base);
}

#[cfg(target_os = "linux")]
#[test]
fn test_lifecycle_linux_dev_full_reports_commit_and_idempotent_outcomes() {
    use std::fs::OpenOptions;

    let root = fresh_root();
    let dictionary = Dictionary::from_document(Default::default()).unwrap();
    let sealed = seal_release(PackId::Llg, PackPayloadKind::Synthetic, &dictionary).unwrap();
    let release_path = root.join("release.json");
    let seal_path = root.join("seal.json");
    fs::write(&release_path, &sealed.release).unwrap();
    fs::write(&seal_path, &sealed.seal).unwrap();
    let catalogue = root.join("catalogue");

    let installed = Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
        .args([
            "collections",
            "install",
            catalogue.to_str().unwrap(),
            "--pack",
            "llg",
            "--release",
            release_path.to_str().unwrap(),
            "--seal",
            seal_path.to_str().unwrap(),
            "--allow-synthetic",
        ])
        .env_clear()
        .output()
        .unwrap();
    let installed: Value = json(&installed);
    assert_eq!(installed["local_revision"], 1);
    let generation_before = installed["generation"]["generation_id"].clone();

    let full = OpenOptions::new().write(true).open("/dev/full").unwrap();
    let failed_output = Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
        .args([
            "collections",
            "rebuild",
            catalogue.to_str().unwrap(),
            "--expected-revision",
            "1",
        ])
        .env_clear()
        .stdout(std::process::Stdio::from(full))
        .stderr(std::process::Stdio::piped())
        .output()
        .unwrap();
    assert!(!failed_output.status.success());
    assert!(failed_output.stdout.is_empty());
    let envelope: Value = serde_json::from_slice(&failed_output.stderr).unwrap();
    let error = &envelope["error"];
    for field in [
        "version",
        "ok",
        "operation",
        "category",
        "detail",
        "path",
        "pack_id",
        "attempt_id",
        "prior_local_revision",
        "proposed_generation",
        "observed_generation",
        "activation",
        "inputs",
    ] {
        assert!(error.get(field).is_some(), "missing report field {field}");
    }
    assert_eq!(error["version"], 1);
    assert_eq!(error["ok"], false);
    assert_eq!(error["operation"], "rebuild");
    assert_eq!(error["category"], "response_output_io");
    assert_eq!(error["activation"], "committed");
    assert_eq!(error["prior_local_revision"], 1);
    assert_ne!(error["proposed_generation"], generation_before);
    assert_eq!(error["observed_generation"], error["proposed_generation"]);
    assert!(
        error["detail"]
            .as_str()
            .unwrap()
            .contains("local_revision=2")
    );

    let generation_after: Value = json(&run(&[
        "collections".as_ref(),
        "list".as_ref(),
        catalogue.as_os_str(),
    ]));
    assert_eq!(generation_after["local_revision"], 2);
    assert_eq!(
        generation_after["generation"]["generation_id"],
        error["observed_generation"]
    );
    let db = catalogue.join("catalogue.sqlite3");
    let before_idempotent_migration = fs::read(&db).unwrap();

    let full = OpenOptions::new().write(true).open("/dev/full").unwrap();
    let idempotent_migration_output = Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
        .args([
            "collections",
            "migrate",
            catalogue.to_str().unwrap(),
            "--expected-revision",
            "2",
        ])
        .env_clear()
        .stdout(std::process::Stdio::from(full))
        .stderr(std::process::Stdio::piped())
        .output()
        .unwrap();
    assert!(!idempotent_migration_output.status.success());
    assert!(idempotent_migration_output.stdout.is_empty());
    let migration_envelope: Value =
        serde_json::from_slice(&idempotent_migration_output.stderr).unwrap();
    let migration_error = &migration_envelope["error"];
    for field in [
        "version",
        "ok",
        "operation",
        "category",
        "detail",
        "path",
        "pack_id",
        "attempt_id",
        "prior_local_revision",
        "proposed_generation",
        "observed_generation",
        "activation",
        "inputs",
    ] {
        assert!(
            migration_error.get(field).is_some(),
            "missing report field {field}"
        );
    }
    assert_eq!(migration_error["version"], 1);
    assert_eq!(migration_error["ok"], false);
    assert_eq!(migration_error["operation"], "migrate");
    assert_eq!(migration_error["category"], "response_output_io");
    assert_eq!(migration_error["activation"], "not_applicable");
    assert_eq!(migration_error["prior_local_revision"], 2);
    assert!(migration_error["attempt_id"].is_null());
    assert!(migration_error["proposed_generation"].is_null());
    assert_eq!(
        migration_error["observed_generation"],
        generation_after["generation"]["generation_id"]
    );
    assert!(
        migration_error["detail"]
            .as_str()
            .unwrap()
            .contains("local_revision=2")
    );
    assert_eq!(fs::read(&db).unwrap(), before_idempotent_migration);

    let idempotent_bytes = fs::read(&db).unwrap();

    let full = OpenOptions::new().write(true).open("/dev/full").unwrap();
    let idempotent_output = Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
        .args([
            "collections",
            "update",
            catalogue.to_str().unwrap(),
            "--pack",
            "llg",
            "--release",
            release_path.to_str().unwrap(),
            "--seal",
            seal_path.to_str().unwrap(),
            "--allow-synthetic",
            "--expected-revision",
            "2",
        ])
        .env_clear()
        .stdout(std::process::Stdio::from(full))
        .stderr(std::process::Stdio::piped())
        .output()
        .unwrap();
    assert!(!idempotent_output.status.success());
    let idempotent_envelope: Value = serde_json::from_slice(&idempotent_output.stderr).unwrap();
    assert_eq!(idempotent_envelope["error"]["operation"], "update");
    assert_eq!(idempotent_envelope["error"]["activation"], "not_applicable");
    assert_eq!(idempotent_envelope["error"]["prior_local_revision"], 2);
    assert!(idempotent_envelope["error"]["proposed_generation"].is_null());
    assert_eq!(
        idempotent_envelope["error"]["observed_generation"],
        generation_after["generation"]["generation_id"]
    );
    assert_eq!(fs::read(&db).unwrap(), idempotent_bytes);

    let ordinary = run(&[
        "collections".as_ref(),
        "update".as_ref(),
        catalogue.as_os_str(),
        "--pack".as_ref(),
        "llg".as_ref(),
        "--release".as_ref(),
        release_path.as_os_str(),
        "--seal".as_ref(),
        seal_path.as_os_str(),
        "--allow-synthetic".as_ref(),
        "--expected-revision".as_ref(),
        "2".as_ref(),
    ]);
    assert_eq!(json(&ordinary)["outcome"], "idempotent");
    assert_eq!(fs::read(&db).unwrap(), idempotent_bytes);

    let before_remove: Value = json(&run(&[
        "collections".as_ref(),
        "list".as_ref(),
        catalogue.as_os_str(),
    ]));
    assert_eq!(before_remove["local_revision"], 2);
    assert_eq!(
        before_remove["generation"]["generation_id"],
        generation_after["generation"]["generation_id"]
    );

    let full = OpenOptions::new().write(true).open("/dev/full").unwrap();
    let remove_output = Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
        .args([
            "collections",
            "remove",
            catalogue.to_str().unwrap(),
            "--pack",
            "llg",
            "--expected-revision",
            "2",
        ])
        .env_clear()
        .stdout(std::process::Stdio::from(full))
        .stderr(std::process::Stdio::piped())
        .output()
        .unwrap();
    assert!(!remove_output.status.success());
    assert!(remove_output.stdout.is_empty());
    let remove_error: Value = serde_json::from_slice(&remove_output.stderr).unwrap();
    let remove_error = &remove_error["error"];
    for field in [
        "version",
        "ok",
        "operation",
        "category",
        "detail",
        "path",
        "pack_id",
        "attempt_id",
        "prior_local_revision",
        "proposed_generation",
        "observed_generation",
        "activation",
        "inputs",
    ] {
        assert!(remove_error.get(field).is_some(), "missing {field}");
    }
    assert_eq!(remove_error["operation"], "remove");
    assert_eq!(remove_error["category"], "response_output_io");
    assert_eq!(remove_error["activation"], "committed");
    assert_eq!(remove_error["prior_local_revision"], 2);
    assert!(remove_error["proposed_generation"].is_null());
    assert!(remove_error["observed_generation"].is_null());
    let after_remove: Value = json(&run(&[
        "collections".as_ref(),
        "list".as_ref(),
        catalogue.as_os_str(),
    ]));
    assert_eq!(after_remove["local_revision"], 3);
    assert!(after_remove["generation"].is_null());

    let full = OpenOptions::new().write(true).open("/dev/full").unwrap();
    let release_sha256 = sha256(&sealed.release);
    let reinstall_output = Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
        .args([
            "collections",
            "reinstall",
            catalogue.to_str().unwrap(),
            "--pack",
            "llg",
            "--release",
            release_path.to_str().unwrap(),
            "--seal",
            seal_path.to_str().unwrap(),
            "--allow-synthetic",
            "--release-sha256",
            &release_sha256,
            "--expected-revision",
            "3",
        ])
        .env_clear()
        .stdout(std::process::Stdio::from(full))
        .stderr(std::process::Stdio::piped())
        .output()
        .unwrap();
    assert!(!reinstall_output.status.success());
    assert!(reinstall_output.stdout.is_empty());
    let reinstall_error: Value = serde_json::from_slice(&reinstall_output.stderr).unwrap();
    let reinstall_error = &reinstall_error["error"];
    for field in [
        "version",
        "ok",
        "operation",
        "category",
        "detail",
        "path",
        "pack_id",
        "attempt_id",
        "prior_local_revision",
        "proposed_generation",
        "observed_generation",
        "activation",
        "inputs",
    ] {
        assert!(reinstall_error.get(field).is_some(), "missing {field}");
    }
    assert_eq!(reinstall_error["operation"], "reinstall");
    assert_eq!(reinstall_error["category"], "response_output_io");
    assert_eq!(reinstall_error["activation"], "committed");
    assert_eq!(reinstall_error["prior_local_revision"], 3);
    assert!(reinstall_error["observed_generation"].is_string());
    assert_eq!(
        reinstall_error["proposed_generation"],
        reinstall_error["observed_generation"]
    );
    let after_reinstall: Value = json(&run(&[
        "collections".as_ref(),
        "list".as_ref(),
        catalogue.as_os_str(),
    ]));
    assert_eq!(after_reinstall["local_revision"], 4);
    assert_eq!(
        after_reinstall["generation"]["generation_id"],
        reinstall_error["observed_generation"]
    );
    let _ = fs::remove_dir_all(root);
}
