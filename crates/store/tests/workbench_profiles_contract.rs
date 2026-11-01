//! Public contract for versioned configured workbench profiles.

use std::{
    borrow::Cow,
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
};

use vlajmaji_lexicon::{Dictionary, PackId, PackPayloadKind, seal_release};
use vlajmaji_store::{
    CollectionCatalogue, CollectionInput, CollectionRequest, CollectionSelection,
    ConfiguredWorkbenchView, WorkbenchBuildRequest, WorkbenchProfileConfigureRequest,
    WorkbenchProfileError, WorkbenchProfiles, WorkbenchSelection, WorkbenchView,
    native_path_report,
};

static NEXT_ROOT: AtomicUsize = AtomicUsize::new(0);

fn fresh_root(label: &str) -> PathBuf {
    for _ in 0..64 {
        let path = std::env::temp_dir().join(format!(
            "vlajmaji-workbench-profiles-{label}-{}-{}",
            std::process::id(),
            NEXT_ROOT.fetch_add(1, Ordering::Relaxed)
        ));
        match fs::create_dir(&path) {
            Ok(()) => return path,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => panic!("create exclusive test root: {error}"),
        }
    }
    panic!("could not allocate an exclusive test root after 64 attempts")
}

fn selection() -> WorkbenchSelection {
    WorkbenchSelection {
        version: 1,
        packs: vec![PackId::Llg],
        disabled: Vec::new(),
        owned: None,
        contexts: Vec::new(),
    }
}

fn configure_request(
    name: &str,
    catalogue_root: &Path,
    make_default: bool,
) -> WorkbenchProfileConfigureRequest {
    WorkbenchProfileConfigureRequest {
        name: name.to_owned(),
        catalogue_root: Some(catalogue_root.to_path_buf()),
        owned_root: None,
        selection: selection(),
        make_default,
    }
}

fn install_synthetic_catalogue(root: &Path) {
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
    CollectionCatalogue::install(root, request).unwrap();
}

fn register_synthetic_pack(root: &Path, pack_id: PackId) {
    let dictionary = Dictionary::from_document(Default::default()).unwrap();
    let sealed = seal_release(pack_id, PackPayloadKind::Synthetic, &dictionary).unwrap();
    let request = CollectionRequest::new(
        vec![CollectionInput::SealedBytes {
            pack_id,
            release: Cow::Borrowed(&sealed.release),
            seal: Cow::Borrowed(&sealed.seal),
        }],
        CollectionSelection::Packs(vec![pack_id]),
        true,
    )
    .unwrap();
    CollectionCatalogue::register(root, request).unwrap();
}

#[test]
fn test_configure_noop_cas_and_strict_nullable_wire_are_preserved() {
    let root = fresh_root("config-wire");
    let catalogue = root.join("catalogue");
    install_synthetic_catalogue(&catalogue);
    let config = root.join("profiles.json");

    let created =
        WorkbenchProfiles::configure(&config, configure_request("alpha", &catalogue, true), None)
            .unwrap();
    assert!(created.changed);
    assert_eq!(created.prior_revision, None);
    assert_eq!(created.revision, 1);
    assert_eq!(created.active, None);
    let captured = WorkbenchProfiles::read(&config).unwrap();
    assert_eq!(captured.profile(None).unwrap().name, "alpha");
    assert_eq!(captured.profile(None).unwrap().active_generation, None);

    let original_bytes = fs::read(&config).unwrap();
    let no_op = WorkbenchProfiles::configure(
        &config,
        configure_request("alpha", &catalogue, true),
        Some(1),
    )
    .unwrap();
    assert!(!no_op.changed);
    assert_eq!(no_op.revision, 1);
    assert_eq!(no_op.sha256, created.sha256);
    assert_eq!(fs::read(&config).unwrap(), original_bytes);

    let stale = WorkbenchProfiles::configure(
        &config,
        configure_request("alpha", &catalogue, false),
        Some(0),
    )
    .unwrap_err();
    assert!(matches!(stale, WorkbenchProfileError::Conflict { .. }));
    assert_eq!(fs::read(&config).unwrap(), original_bytes);

    let mut wire: serde_json::Value = serde_json::from_slice(&original_bytes).unwrap();
    wire["profiles"][0]
        .as_object_mut()
        .unwrap()
        .remove("active_generation");
    fs::write(&config, serde_json::to_vec(&wire).unwrap()).unwrap();
    assert!(matches!(
        WorkbenchProfiles::read(&config),
        Err(WorkbenchProfileError::Decode { .. })
    ));

    wire = serde_json::from_slice(&original_bytes).unwrap();
    wire["profiles"][0]
        .as_object_mut()
        .unwrap()
        .remove("catalogue_root");
    fs::write(&config, serde_json::to_vec(&wire).unwrap()).unwrap();
    assert!(matches!(
        WorkbenchProfiles::read(&config),
        Err(WorkbenchProfileError::Decode { .. })
    ));

    wire = serde_json::from_slice(&original_bytes).unwrap();
    wire["profiles"][0]
        .as_object_mut()
        .unwrap()
        .remove("owned_root");
    fs::write(&config, serde_json::to_vec(&wire).unwrap()).unwrap();
    assert!(matches!(
        WorkbenchProfiles::read(&config),
        Err(WorkbenchProfileError::Decode { .. })
    ));

    wire = serde_json::from_slice(&original_bytes).unwrap();
    wire["profiles"][0]["unknown"] = serde_json::json!(true);
    fs::write(&config, serde_json::to_vec(&wire).unwrap()).unwrap();
    assert!(matches!(
        WorkbenchProfiles::read(&config),
        Err(WorkbenchProfileError::Decode { .. })
    ));

    wire = serde_json::from_slice(&original_bytes).unwrap();
    wire["version"] = serde_json::json!(2);
    fs::write(&config, serde_json::to_vec(&wire).unwrap()).unwrap();
    assert!(matches!(
        WorkbenchProfiles::read(&config),
        Err(WorkbenchProfileError::UnsupportedVersion { found: 2, .. })
    ));

    let original_text = std::str::from_utf8(&original_bytes).unwrap();
    let duplicate_top_level = format!("{{\"version\":1,{}", &original_text[1..]);
    fs::write(&config, duplicate_top_level).unwrap();
    assert!(matches!(
        WorkbenchProfiles::read(&config),
        Err(WorkbenchProfileError::Decode { .. })
    ));

    let original_text = std::str::from_utf8(&original_bytes).unwrap();
    let duplicate_nested = original_text.replacen(
        "\"name\":\"alpha\"",
        "\"name\":\"alpha\",\"name\":\"alpha\"",
        1,
    );
    fs::write(&config, duplicate_nested).unwrap();
    assert!(matches!(
        WorkbenchProfiles::read(&config),
        Err(WorkbenchProfileError::Decode { .. })
    ));

    let original_text = std::str::from_utf8(&original_bytes).unwrap();
    let duplicate_selection_field = original_text.replacen(
        "\"selection\":{\"version\":1",
        "\"selection\":{\"version\":1,\"version\":1",
        1,
    );
    fs::write(&config, duplicate_selection_field).unwrap();
    assert!(matches!(
        WorkbenchProfiles::read(&config),
        Err(WorkbenchProfileError::Decode { .. })
    ));

    wire = serde_json::from_slice(&original_bytes).unwrap();
    wire["profiles"][0]["catalogue_root"]["hex"] = serde_json::json!("0g");
    fs::write(&config, serde_json::to_vec(&wire).unwrap()).unwrap();
    assert!(matches!(
        WorkbenchProfiles::read(&config),
        Err(WorkbenchProfileError::InvalidPolicy { .. })
    ));

    wire = serde_json::from_slice(&original_bytes).unwrap();
    wire["default_profile"] = serde_json::json!("missing");
    fs::write(&config, serde_json::to_vec(&wire).unwrap()).unwrap();
    assert!(matches!(
        WorkbenchProfiles::read(&config),
        Err(WorkbenchProfileError::InvalidPolicy { .. })
    ));

    wire = serde_json::from_slice(&original_bytes).unwrap();
    wire["profiles"][0]["active_generation"] = serde_json::json!(0);
    fs::write(&config, serde_json::to_vec(&wire).unwrap()).unwrap();
    assert!(matches!(
        WorkbenchProfiles::read(&config),
        Err(WorkbenchProfileError::InvalidPolicy { .. })
    ));

    wire = serde_json::from_slice(&original_bytes).unwrap();
    wire["profiles"][0]["name"] = serde_json::json!("");
    fs::write(&config, serde_json::to_vec(&wire).unwrap()).unwrap();
    assert!(matches!(
        WorkbenchProfiles::read(&config),
        Err(WorkbenchProfileError::InvalidPolicy { .. })
    ));

    fs::write(&config, &original_bytes).unwrap();
    let intervening = WorkbenchProfiles::configure(
        &config,
        configure_request("beta", &catalogue, false),
        Some(1),
    )
    .unwrap();
    assert_eq!(intervening.prior_revision, Some(1));
    assert_eq!(intervening.revision, 2);
    let latest_bytes = fs::read(&config).unwrap();
    let stale_after_writer = WorkbenchProfiles::set_default(&config, Some("beta"), 1).unwrap_err();
    assert!(matches!(
        stale_after_writer,
        WorkbenchProfileError::Conflict {
            expected: Some(1),
            observed: Some(2)
        }
    ));
    assert_eq!(fs::read(&config).unwrap(), latest_bytes);

    let mut overflow: serde_json::Value = serde_json::from_slice(&original_bytes).unwrap();
    overflow["revision"] = serde_json::json!(i64::MAX);
    let overflow_bytes = serde_json::to_vec(&overflow).unwrap();
    fs::write(&config, &overflow_bytes).unwrap();
    assert!(matches!(
        WorkbenchProfiles::set_default(&config, None, i64::MAX as u64),
        Err(WorkbenchProfileError::RevisionOverflow(value)) if value == i64::MAX as u64
    ));
    assert_eq!(fs::read(&config).unwrap(), overflow_bytes);
    let lock = config.with_file_name("profiles.json.lock");
    assert!(!lock.exists());
    assert_eq!(
        fs::read_dir(&root)
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_name().to_string_lossy().contains(".stage-"))
            .count(),
        0
    );
}

#[test]
fn test_configure_preserves_unicode_config_basename_in_lock_sibling() {
    let root = fresh_root("raw-config-lock");
    let catalogue = root.join("catalogue");
    install_synthetic_catalogue(&catalogue);
    let config_name = "profiles ü.json";
    let config = root.join(config_name);
    let mut lock_name = std::ffi::OsString::from(config_name);
    lock_name.push(".lock");
    let expected_lock = root.join(lock_name);
    fs::write(&expected_lock, b"owned sentinel").unwrap();

    assert!(matches!(
        WorkbenchProfiles::configure(
            &config,
            configure_request("alpha", &catalogue, true),
            None
        ),
        Err(WorkbenchProfileError::Busy { lock }) if lock == expected_lock
    ));
    assert_eq!(fs::read(&expected_lock).unwrap(), b"owned sentinel");
    fs::remove_file(&expected_lock).unwrap();
    fs::create_dir(&expected_lock).unwrap();
    assert!(matches!(
        WorkbenchProfiles::configure(
            &config,
            configure_request("alpha", &catalogue, true),
            None
        ),
        Err(WorkbenchProfileError::Busy { lock }) if lock == expected_lock
    ));
    assert!(expected_lock.is_dir());
    fs::remove_dir(&expected_lock).unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;

        let target = root.join("symlink-target.json");
        let target_bytes = b"target remains byte-exact";
        fs::write(&target, target_bytes).unwrap();
        let link = root.join("profiles-link.json");
        symlink(&target, &link).unwrap();
        assert!(matches!(
            WorkbenchProfiles::configure(&link, configure_request("alpha", &catalogue, true), None),
            Err(WorkbenchProfileError::InvalidPolicy { .. })
        ));
        assert!(
            fs::symlink_metadata(&link)
                .unwrap()
                .file_type()
                .is_symlink()
        );
        assert_eq!(fs::read(&target).unwrap(), target_bytes);
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn test_refresh_history_and_captured_view_invalidate_on_any_config_change() {
    let root = fresh_root("refresh-history");
    let catalogue = root.join("catalogue");
    install_synthetic_catalogue(&catalogue);
    let config = root.join("profiles.json");
    WorkbenchProfiles::configure(&config, configure_request("alpha", &catalogue, true), None)
        .unwrap();

    let alpha_root = root.join("alpha-root");
    let refreshed = WorkbenchProfiles::refresh(&config, None, &alpha_root, 1).unwrap();
    assert_eq!(refreshed.publication.revision, 2);
    assert_eq!(refreshed.publication.active.as_ref().unwrap().sequence, 0);
    assert_eq!(refreshed.view.root, native_path_report(&alpha_root));

    let valid_prepared_bytes = fs::read(&config).unwrap();
    let mut prepared_wire: serde_json::Value =
        serde_json::from_slice(&valid_prepared_bytes).unwrap();
    prepared_wire["prepared"][0]["unknown"] = serde_json::json!(true);
    fs::write(&config, serde_json::to_vec(&prepared_wire).unwrap()).unwrap();
    assert!(matches!(
        WorkbenchProfiles::read(&config),
        Err(WorkbenchProfileError::Decode { .. })
    ));

    let valid_prepared_text = std::str::from_utf8(&valid_prepared_bytes).unwrap();
    let duplicate_prepared_field =
        valid_prepared_text.replacen("\"sequence\":0", "\"sequence\":0,\"sequence\":0", 1);
    fs::write(&config, duplicate_prepared_field).unwrap();
    assert!(matches!(
        WorkbenchProfiles::read(&config),
        Err(WorkbenchProfileError::Decode { .. })
    ));

    prepared_wire = serde_json::from_slice(&valid_prepared_bytes).unwrap();
    let native_hex = prepared_wire["prepared"][0]["root"]["hex"]
        .as_str()
        .unwrap();
    let native_hex_field = format!("\"hex\":{}", serde_json::to_string(native_hex).unwrap());
    let duplicate_native_field = valid_prepared_text.replacen(
        &native_hex_field,
        &format!("{native_hex_field},{native_hex_field}"),
        1,
    );
    fs::write(&config, duplicate_native_field).unwrap();
    assert!(matches!(
        WorkbenchProfiles::read(&config),
        Err(WorkbenchProfileError::Decode { .. })
    ));

    let native_utf8 = serde_json::to_string(&prepared_wire["prepared"][0]["root"]["utf8"]).unwrap();
    let native_utf8_field = format!("\"utf8\":{native_utf8}");
    let unknown_native_field = valid_prepared_text.replacen(
        &native_utf8_field,
        &format!("{native_utf8_field},\"unknown\":true"),
        1,
    );
    fs::write(&config, unknown_native_field).unwrap();
    assert!(matches!(
        WorkbenchProfiles::read(&config),
        Err(WorkbenchProfileError::Decode { .. })
    ));

    prepared_wire["prepared"][0]["root"]
        .as_object_mut()
        .unwrap()
        .remove("utf8");
    fs::write(&config, serde_json::to_vec(&prepared_wire).unwrap()).unwrap();
    assert!(matches!(
        WorkbenchProfiles::read(&config),
        Err(WorkbenchProfileError::Decode { .. })
    ));

    prepared_wire = serde_json::from_slice(&valid_prepared_bytes).unwrap();
    prepared_wire["prepared"][0]["root"]["hex"] = serde_json::json!("0g");
    fs::write(&config, serde_json::to_vec(&prepared_wire).unwrap()).unwrap();
    assert!(matches!(
        WorkbenchProfiles::read(&config),
        Err(WorkbenchProfileError::InvalidPolicy { .. })
    ));

    prepared_wire = serde_json::from_slice(&valid_prepared_bytes).unwrap();
    prepared_wire["prepared"][0]["sequence"] = serde_json::json!(1);
    fs::write(&config, serde_json::to_vec(&prepared_wire).unwrap()).unwrap();
    assert!(matches!(
        WorkbenchProfiles::read(&config),
        Err(WorkbenchProfileError::InvalidPolicy { .. })
    ));

    prepared_wire = serde_json::from_slice(&valid_prepared_bytes).unwrap();
    let actual_binding = prepared_wire["prepared"][0]["binding_sha256"]
        .as_str()
        .unwrap();
    let wrong_binding = if actual_binding == "0".repeat(64) {
        "1".repeat(64)
    } else {
        "0".repeat(64)
    };
    prepared_wire["prepared"][0]["binding_sha256"] = serde_json::json!(wrong_binding);
    fs::write(&config, serde_json::to_vec(&prepared_wire).unwrap()).unwrap();
    assert!(matches!(
        ConfiguredWorkbenchView::open(&config, None),
        Err(WorkbenchProfileError::BindingMismatch { .. })
    ));
    fs::write(&config, valid_prepared_bytes).unwrap();

    let captured = ConfiguredWorkbenchView::open(&config, None).unwrap();
    let identity = captured.identity().clone();
    assert_eq!(identity.requested_profile, None);
    assert_eq!(identity.prepared_sequence, 0);
    assert_eq!(
        captured.snapshot_manifest().generation,
        identity.effective_binding_sha256
    );

    WorkbenchProfiles::configure(
        &config,
        configure_request("beta", &catalogue, false),
        Some(2),
    )
    .unwrap();
    assert!(matches!(
        captured.ensure_current(),
        Err(WorkbenchProfileError::ConfigChanged { .. })
    ));

    let reopened_alpha = ConfiguredWorkbenchView::open(&config, Some("alpha")).unwrap();
    assert_eq!(
        reopened_alpha.identity().requested_profile.as_deref(),
        Some("alpha")
    );
    let beta_root = root.join("beta-root");
    let beta = WorkbenchProfiles::refresh(&config, Some("beta"), &beta_root, 3).unwrap();
    assert_eq!(beta.publication.revision, 4);
    assert_eq!(beta.publication.active.as_ref().unwrap().sequence, 1);

    let all = WorkbenchProfiles::read(&config)
        .unwrap()
        .history(None)
        .unwrap();
    assert_eq!(
        all.prepared
            .iter()
            .map(|row| row.sequence)
            .collect::<Vec<_>>(),
        [0, 1]
    );
    assert_eq!(
        all.prepared[0].root.utf8.as_deref(),
        Some(alpha_root.to_str().unwrap())
    );
    assert_eq!(
        all.prepared[1].root.utf8.as_deref(),
        Some(beta_root.to_str().unwrap())
    );
}

#[test]
fn test_status_reports_logical_catalogue_and_prepared_state() {
    let root = fresh_root("status");
    let catalogue = root.join("catalogue");
    install_synthetic_catalogue(&catalogue);
    let config = root.join("profiles.json");
    WorkbenchProfiles::configure(&config, configure_request("alpha", &catalogue, true), None)
        .unwrap();
    let unprepared = WorkbenchProfiles::read(&config)
        .unwrap()
        .status(None)
        .unwrap();
    assert_eq!(
        unprepared.scope,
        "configured-metadata-and-active-minimum-v1"
    );
    assert!(unprepared.active.is_none());
    let catalogue_status = unprepared.catalogue.unwrap();
    assert_eq!(catalogue_status.packs.len(), 1);
    assert_eq!(catalogue_status.packs[0].pack_id, "llg");
    assert!(catalogue_status.packs[0].selected);

    let selected_unregistered_config = root.join("selected-unregistered.json");
    WorkbenchProfiles::configure(
        &selected_unregistered_config,
        WorkbenchProfileConfigureRequest {
            name: "selected-unregistered".into(),
            catalogue_root: Some(catalogue.clone()),
            owned_root: None,
            selection: WorkbenchSelection {
                version: 1,
                packs: vec![PackId::Llg, PackId::Sutysisku],
                disabled: Vec::new(),
                owned: None,
                contexts: Vec::new(),
            },
            make_default: true,
        },
        None,
    )
    .unwrap();
    let selected_unregistered = WorkbenchProfiles::read(&selected_unregistered_config)
        .unwrap()
        .status(None)
        .unwrap();
    assert!(selected_unregistered.active.is_none());
    let selected_rows = selected_unregistered.catalogue.unwrap().packs;
    assert_eq!(selected_rows.len(), 2);
    assert_eq!(selected_rows[0].pack_id, "llg");
    assert!(selected_rows[0].selected);
    assert_eq!(selected_rows[1].pack_id, "sutysisku");
    assert!(selected_rows[1].selected && !selected_rows[1].disabled);
    assert!(selected_rows[1].installation_state.is_none());
    assert!(selected_rows[1].installed_release_sha256.is_none());
    assert!(selected_rows[1].registrations.is_empty());

    let destination = root.join("prepared");
    WorkbenchProfiles::refresh(&config, None, &destination, 1).unwrap();
    let current = WorkbenchProfiles::read(&config)
        .unwrap()
        .status(None)
        .unwrap();
    let active = current.active.unwrap();
    assert_eq!(
        active.state,
        vlajmaji_store::WorkbenchPreparedState::Current
    );
    assert!(active.error.is_none());
    assert!(current.owned.is_none());

    let database = destination.join("dictionary/dictionary.sqlite3");
    let wrong_level_sentinel = destination.join("dictionary.sqlite3");
    fs::write(&wrong_level_sentinel, b"leave this unrelated file alone").unwrap();
    fs::remove_file(&database).unwrap();
    let missing = WorkbenchProfiles::read(&config)
        .unwrap()
        .status(None)
        .unwrap()
        .active
        .unwrap();
    assert_eq!(
        missing.state,
        vlajmaji_store::WorkbenchPreparedState::Unavailable
    );
    assert_eq!(missing.error.unwrap().code, "workbench_dependency");
    assert_eq!(
        fs::read(&wrong_level_sentinel).unwrap(),
        b"leave this unrelated file alone"
    );

    let connection = rusqlite::Connection::open(&database).unwrap();
    connection
        .execute_batch("CREATE TABLE metadata(key TEXT NOT NULL PRIMARY KEY, value TEXT NOT NULL);")
        .unwrap();
    drop(connection);
    let malformed = WorkbenchProfiles::read(&config)
        .unwrap()
        .status(None)
        .unwrap()
        .active
        .unwrap();
    assert_eq!(
        malformed.state,
        vlajmaji_store::WorkbenchPreparedState::Invalid
    );

    register_synthetic_pack(&catalogue, PackId::LaLojban);
    register_synthetic_pack(&catalogue, PackId::LensiskuJbotci);
    register_synthetic_pack(&catalogue, PackId::Sutysisku);
    let catalogue_reader = CollectionCatalogue::open(&catalogue).unwrap();
    let catalogue_revision = catalogue_reader.local_revision().unwrap();
    drop(catalogue_reader);
    CollectionCatalogue::remove(&catalogue, &[PackId::Sutysisku], catalogue_revision).unwrap();
    let logical_config = root.join("logical-status.json");
    WorkbenchProfiles::configure(
        &logical_config,
        WorkbenchProfileConfigureRequest {
            name: "logical".into(),
            catalogue_root: Some(catalogue.clone()),
            owned_root: None,
            selection: WorkbenchSelection {
                version: 1,
                packs: vec![PackId::Llg, PackId::LaLojban, PackId::Sutysisku],
                disabled: vec![PackId::LaLojban],
                owned: None,
                contexts: Vec::new(),
            },
            make_default: true,
        },
        None,
    )
    .unwrap();
    let logical_status = WorkbenchProfiles::read(&logical_config)
        .unwrap()
        .status(None)
        .unwrap();
    let logical_packs = logical_status.catalogue.unwrap().packs;
    assert_eq!(logical_packs.len(), 4);
    assert_eq!(logical_packs[0].pack_id, "llg");
    assert!(logical_packs[0].selected);
    assert_eq!(
        logical_packs[0].installation_state,
        Some(vlajmaji_store::CollectionInstallationState::Installed)
    );
    assert_eq!(logical_packs[1].pack_id, "la-lojban");
    assert!(logical_packs[1].selected && logical_packs[1].disabled);
    assert_eq!(
        logical_packs[1].installation_state,
        Some(vlajmaji_store::CollectionInstallationState::RegisteredOnly)
    );
    assert_eq!(logical_packs[2].pack_id, "sutysisku");
    assert!(logical_packs[2].selected && !logical_packs[2].disabled);
    assert_eq!(
        logical_packs[2].installation_state,
        Some(vlajmaji_store::CollectionInstallationState::Removed)
    );
    assert_eq!(logical_packs[3].pack_id, "lensisku-jbotci");
    assert!(!logical_packs[3].selected && !logical_packs[3].disabled);
    assert_eq!(
        logical_packs[3].installation_state,
        Some(vlajmaji_store::CollectionInstallationState::RegisteredOnly)
    );
}

#[test]
fn test_build_drops_catalogue_reader_before_successful_writer() {
    let root = fresh_root("reader-drop-writer");
    let catalogue = root.join("catalogue");
    install_synthetic_catalogue(&catalogue);
    let view_root = root.join("view");
    WorkbenchView::build_new(
        &view_root,
        WorkbenchBuildRequest {
            catalogue_root: Some(catalogue.clone()),
            owned_root: None,
            selection: selection(),
        },
    )
    .unwrap();

    let reader = CollectionCatalogue::open(&catalogue).unwrap();
    let before = reader.metadata_roster().unwrap();
    assert_eq!(before.local_revision, 1);
    assert!(before.generation.is_some());
    drop(reader);
    let writer = CollectionCatalogue::rebuild(&catalogue, before.local_revision).unwrap();
    assert_eq!(writer.local_revision, before.local_revision + 1);
    let refreshed_reader = CollectionCatalogue::open(&catalogue).unwrap();
    let after = refreshed_reader.metadata_roster().unwrap();
    assert_eq!(after.local_revision, before.local_revision + 1);
    assert!(after.generation.is_some());
    assert_ne!(after.generation, before.generation);
    drop(refreshed_reader);
    assert!(matches!(
        WorkbenchView::open(&view_root),
        Err(vlajmaji_store::WorkbenchError::StaleDependencies { .. })
    ));
    fs::remove_dir_all(root).unwrap();
}
