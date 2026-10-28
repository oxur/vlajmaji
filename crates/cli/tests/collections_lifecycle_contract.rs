//! CLI coverage for the explicit catalogue lifecycle commands.

use std::{
    borrow::Cow,
    fs,
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
    sync::atomic::{AtomicUsize, Ordering},
};

use serde_json::Value;
use sha2::{Digest, Sha256};
use vlajmaji_lexicon::{
    CatalogTarget, Dictionary, PackId, PackPayloadKind, RelationTarget, SutysiskuImportOptions,
    import_sutysisku_export, seal_release,
};
use vlajmaji_search::{
    CollectionQueryResult, CollectionQuerySelection, CollectionSearchEngine,
    DictionaryExactOptions, DictionaryRequest, DictionarySearchEngine,
};
use vlajmaji_store::{
    CollectionCatalogue, CollectionInput, CollectionRequest, CollectionSelection,
};

static NEXT_ROOT: AtomicUsize = AtomicUsize::new(0);

fn fresh_root() -> PathBuf {
    for _ in 0..100 {
        let root = std::env::temp_dir().join(format!(
            "vlajmaji-s04-cli-lifecycle-{}-{}",
            std::process::id(),
            NEXT_ROOT.fetch_add(1, Ordering::Relaxed)
        ));
        match fs::create_dir(&root) {
            Ok(()) => return root,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => panic!("create exclusive scratch directory: {error}"),
        }
    }
    panic!("could not allocate unique scratch directory after 100 attempts")
}

fn run(args: &[&std::ffi::OsStr]) -> std::process::Output {
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

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[test]
fn test_collections_lifecycle_commands_migrate_update_rebuild_reports() {
    let root = fresh_root();
    let release_data = Dictionary::from_document(Default::default()).unwrap();
    let sealed = seal_release(PackId::Llg, PackPayloadKind::Synthetic, &release_data).unwrap();
    let release_path = root.join("release.json");
    let seal_path = root.join("seal.json");
    fs::write(&release_path, sealed.release).unwrap();
    fs::write(&seal_path, sealed.seal).unwrap();
    let catalogue = root.join("catalogue");

    let install = run(&[
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
    let installed = json(&install);
    assert_eq!(installed["operation"], "install");
    assert_eq!(installed["local_revision"], 1);

    let stale_migration = run(&[
        "collections".as_ref(),
        "migrate".as_ref(),
        catalogue.as_os_str(),
        "--expected-revision".as_ref(),
        "0".as_ref(),
    ]);
    assert!(!stale_migration.status.success());
    assert!(stale_migration.stdout.is_empty());
    let stale_migration_report: Value = serde_json::from_slice(&stale_migration.stderr).unwrap();
    assert_eq!(
        stale_migration_report["error"]["category"],
        "revision_conflict"
    );
    assert_eq!(stale_migration_report["error"]["operation"], "migrate");
    assert_eq!(
        stale_migration_report["error"]["activation"],
        "not_activated"
    );

    let migrate = run(&[
        "collections".as_ref(),
        "migrate".as_ref(),
        catalogue.as_os_str(),
        "--expected-revision".as_ref(),
        "1".as_ref(),
    ]);
    let migrated = json(&migrate);
    assert_eq!(migrated["operation"], "migrate");
    assert_eq!(migrated["outcome"], "idempotent");
    assert_eq!(migrated["schema_version"], 2);

    let update = run(&[
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
        "1".as_ref(),
    ]);
    let updated = json(&update);
    assert_eq!(updated["operation"], "update");
    assert_eq!(updated["outcome"], "idempotent");
    assert_eq!(updated["local_revision"], 1);

    let stale_rebuild = run(&[
        "collections".as_ref(),
        "rebuild".as_ref(),
        catalogue.as_os_str(),
        "--expected-revision".as_ref(),
        "0".as_ref(),
    ]);
    assert!(!stale_rebuild.status.success());
    assert!(stale_rebuild.stdout.is_empty());
    let stale_rebuild_report: Value = serde_json::from_slice(&stale_rebuild.stderr).unwrap();
    assert_eq!(
        stale_rebuild_report["error"]["category"],
        "revision_conflict"
    );
    assert_eq!(stale_rebuild_report["error"]["operation"], "rebuild");
    assert_eq!(stale_rebuild_report["error"]["activation"], "not_activated");

    let rebuild = run(&[
        "collections".as_ref(),
        "rebuild".as_ref(),
        catalogue.as_os_str(),
        "--expected-revision".as_ref(),
        "1".as_ref(),
    ]);
    let rebuilt = json(&rebuild);
    assert_eq!(rebuilt["operation"], "rebuild");
    assert_eq!(rebuilt["outcome"], "rebuilt");
    assert_eq!(rebuilt["local_revision"], 2);

    // A committed lifecycle mutation remains successful when the caller closes
    // its response pipe after the child has started.
    let mut child = Command::new("/bin/sh")
        .arg("-c")
        .arg("IFS= read -r _ || exit 125; exec \"$@\"")
        .arg("collections-rebuild-output-loss")
        .arg(env!("CARGO_BIN_EXE_vlajmaji"))
        .arg("collections")
        .arg("rebuild")
        .arg(&catalogue)
        .arg("--expected-revision")
        .arg("2")
        .env_clear()
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("start gated rebuild CLI");
    drop(child.stdout.take().expect("rebuild stdout pipe"));
    child
        .stdin
        .take()
        .expect("rebuild stdin gate")
        .write_all(b"go\n")
        .unwrap();
    let lost_response = child.wait_with_output().unwrap();
    assert!(lost_response.status.success());
    assert!(lost_response.stdout.is_empty());
    assert!(lost_response.stderr.is_empty());

    let list = run(&[
        "collections".as_ref(),
        "list".as_ref(),
        catalogue.as_os_str(),
    ]);
    assert_eq!(json(&list)["catalogue_schema_version"], 2);
    assert_eq!(json(&list)["local_revision"], 3);
    let _ = fs::remove_dir_all(root);
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
    let _ = fs::remove_dir_all(root);
}

#[test]
fn test_sut_lifecycle_update_rebuild_preserves_old_store_and_stales_managed_cursor() {
    let root = fresh_root();
    let catalogue = root.join("catalogue");
    let raw = br#"{"farvi":{"d":"grow","t":"gismu"},"farvi":{"d":"develop","t":"gismu"},"zoizoizoi":{"d":"retained excluded","t":"unsupported"}}"#;
    let import = |revision: &str| {
        import_sutysisku_export(
            raw,
            &SutysiskuImportOptions {
                revision: revision.into(),
                expected_raw_sha256: sha256(raw),
                raw_locator: None,
                notice_locator: None,
            },
        )
        .unwrap()
        .into_dictionary()
    };
    let seal = |dictionary: &Dictionary| {
        seal_release(PackId::Sutysisku, PackPayloadKind::Synthetic, dictionary).unwrap()
    };
    let request = |release: &[u8], seal: &[u8]| {
        CollectionRequest::new(
            vec![CollectionInput::SealedBytes {
                pack_id: PackId::Sutysisku,
                release: Cow::Owned(release.to_vec()),
                seal: Cow::Owned(seal.to_vec()),
            }],
            CollectionSelection::Packs(vec![PackId::Sutysisku]),
            true,
        )
        .unwrap()
    };
    let query_selection = CollectionQuerySelection {
        packs: vec![PackId::Sutysisku],
        disabled: Vec::new(),
        profile: None,
    };

    let original = import("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
    let original_source = original.registry().as_document().sources[0].clone();
    let original_sealed = seal(&original);
    let installed = CollectionCatalogue::install(
        &catalogue,
        request(&original_sealed.release, &original_sealed.seal),
    )
    .unwrap();
    let original_generation = installed.generation.unwrap();
    let original_database = original_generation.store_path.join("dictionary.sqlite3");
    let original_database_bytes = fs::read(&original_database).unwrap();
    let config = root.join("profiles.toml");
    let configured = run(&[
        "collections".as_ref(),
        "configure".as_ref(),
        catalogue.as_os_str(),
        "--profile".as_ref(),
        "lifecycle".as_ref(),
        "--pack".as_ref(),
        "sutysisku".as_ref(),
        "--make-default".as_ref(),
        "--config".as_ref(),
        config.as_os_str(),
    ]);
    assert_eq!(json(&configured)["outcome"], "committed");
    let lookup = |request: &str| {
        run(&[
            "lookup".as_ref(),
            "--request".as_ref(),
            request.as_ref(),
            "--config".as_ref(),
            config.as_os_str(),
        ])
    };
    let first_lookup =
        lookup(r#"{"operation":"exact","spelling":"farvi","options":{"page_size":1}}"#);
    let first_lookup_json = json(&first_lookup);
    assert_eq!(
        first_lookup_json["result"]["response"]["totals"]["occurrences"],
        2
    );
    assert_eq!(
        first_lookup_json["result"]["response"]["source_totals"],
        serde_json::json!([{
            "source": original_source,
            "occurrences": 2,
            "distinct_spellings": 1,
        }])
    );
    let old_cursor = first_lookup_json["result"]["response"]["next_cursor"]
        .as_str()
        .unwrap()
        .to_owned();

    // The legacy explicit STORE route remains pinned to its old immutable
    // generation after catalogue update and rebuild.
    let explicit_store = DictionarySearchEngine::open(&original_generation.store_path).unwrap();
    let old_response = explicit_store
        .exact("farvi", DictionaryExactOptions::default())
        .unwrap();
    assert_eq!(old_response.totals.occurrences, 2);
    let old_wire = serde_json::to_vec(&old_response).unwrap();
    let old_wire_sha256 = sha256(&old_wire);
    let old_store_path = original_generation.store_path.clone();
    let old_store_cli = || {
        Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
            .args([
                "dictionary-query",
                old_store_path.to_str().unwrap(),
                "--request",
                r#"{"operation":"exact","spelling":"farvi"}"#,
            ])
            .env_clear()
            .output()
            .unwrap()
    };
    let old_store_wire = old_store_cli();
    assert!(
        old_store_wire.status.success(),
        "{}",
        String::from_utf8_lossy(&old_store_wire.stderr)
    );
    let old_store_cli_sha256 = sha256(&old_store_wire.stdout);

    let old_source = original_source.id.clone();
    let old_release = original.metadata().releases[0].id.clone();
    let old_target = RelationTarget::Catalog(CatalogTarget {
        source: old_source.clone(),
        release: old_release,
        native_entry_id: Some("farvi".into()),
        native_assertion_id: None,
        occurrence: None,
    });

    let revised = import("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb");
    let revised_sealed = seal(&revised);
    let update = CollectionCatalogue::update(
        &catalogue,
        request(&revised_sealed.release, &revised_sealed.seal),
        installed.local_revision,
    )
    .unwrap();
    let updated_generation = update.generation.unwrap();
    assert_ne!(
        updated_generation.generation_id,
        original_generation.generation_id
    );
    assert_eq!(
        fs::read(&original_database).unwrap(),
        original_database_bytes
    );
    let explicit_after_update =
        DictionarySearchEngine::open(&original_generation.store_path).unwrap();
    let old_after_update = explicit_after_update
        .exact("farvi", DictionaryExactOptions::default())
        .unwrap();
    assert_eq!(
        sha256(&serde_json::to_vec(&old_after_update).unwrap()),
        old_wire_sha256
    );
    let old_store_after_update = old_store_cli();
    assert!(old_store_after_update.status.success());
    assert_eq!(old_store_after_update.stdout, old_store_wire.stdout);
    assert_eq!(sha256(&old_store_after_update.stdout), old_store_cli_sha256);

    let stale_update = lookup(&format!(
        "{{\"operation\":\"exact\",\"spelling\":\"farvi\",\"options\":{{\"page_size\":1,\"cursor\":\"{old_cursor}\"}}}}"
    ));
    assert!(!stale_update.status.success());
    let stale_update_report: Value = serde_json::from_slice(&stale_update.stderr).unwrap();
    assert_eq!(stale_update_report["error"]["category"], "stale_cursor");
    let mut current = CollectionSearchEngine::open(&catalogue, query_selection.clone()).unwrap();
    let new_target = RelationTarget::Catalog(CatalogTarget {
        source: revised.registry().as_document().sources[0].id.clone(),
        release: revised.metadata().releases[0].id.clone(),
        native_entry_id: Some("farvi".into()),
        native_assertion_id: None,
        occurrence: None,
    });
    let old_reference = current
        .execute(DictionaryRequest::Resolve { target: old_target })
        .unwrap();
    let CollectionQueryResult::Resolve(old_reference) = old_reference.result else {
        panic!("expected old catalogue reference result");
    };
    assert_eq!(
        old_reference.status,
        vlajmaji_search::ReferenceStatus::CatalogueOnly
    );
    assert!(old_reference.payload.is_none());
    let new_reference = current
        .execute(DictionaryRequest::Resolve { target: new_target })
        .unwrap();
    let CollectionQueryResult::Resolve(new_reference) = new_reference.result else {
        panic!("expected new resolved reference result");
    };
    assert_eq!(
        new_reference.status,
        vlajmaji_search::ReferenceStatus::Resolved
    );
    assert!(new_reference.payload.is_some());

    let update_lookup =
        lookup(r#"{"operation":"exact","spelling":"farvi","options":{"page_size":1}}"#);
    let update_lookup_json = json(&update_lookup);
    assert_eq!(
        update_lookup_json["result"]["response"]["source_totals"],
        serde_json::json!([{
            "source": revised.registry().as_document().sources[0],
            "occurrences": 2,
            "distinct_spellings": 1,
        }])
    );
    let update_cursor = update_lookup_json["result"]["response"]["next_cursor"]
        .as_str()
        .unwrap()
        .to_owned();

    let rebuilt = CollectionCatalogue::rebuild(&catalogue, update.local_revision).unwrap();
    let rebuilt_generation = rebuilt.generation.unwrap();
    assert_ne!(
        rebuilt_generation.generation_id,
        updated_generation.generation_id
    );
    assert_eq!(
        fs::read(&original_database).unwrap(),
        original_database_bytes
    );
    let old_store_after_rebuild = old_store_cli();
    assert!(old_store_after_rebuild.status.success());
    assert_eq!(old_store_after_rebuild.stdout, old_store_wire.stdout);
    assert_eq!(
        sha256(&old_store_after_rebuild.stdout),
        old_store_cli_sha256
    );
    let stale_rebuild = lookup(&format!(
        "{{\"operation\":\"exact\",\"spelling\":\"farvi\",\"options\":{{\"page_size\":1,\"cursor\":\"{update_cursor}\"}}}}"
    ));
    assert!(!stale_rebuild.status.success());
    let stale_rebuild_report: Value = serde_json::from_slice(&stale_rebuild.stderr).unwrap();
    assert_eq!(stale_rebuild_report["error"]["category"], "stale_cursor");
    let _ = fs::remove_dir_all(root);
}
