//! Public local-collection contract tests using tiny synthetic sealed packs.

use std::{
    borrow::Cow,
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

#[path = "../../lexicon/examples/support/dictionary_fixture.rs"]
mod fixture_support;

use rusqlite::Connection;
use sha2::Digest;
use vlajmaji_lexicon::{
    AbbreviationKind, CollectionRecord, Dictionary, DictionaryDocument, EntityId, PackId,
    PackPayloadKind, Presence, RegistryDocument, SourceId, SourceOrigin, SourceRecord,
    SourceRegistry, seal_release,
};
use vlajmaji_store::{
    CollectionCatalogue, CollectionError, CollectionInput, CollectionRequest, CollectionSelection,
    DictionaryStoreVerification, DictionaryStoreVerificationCheck,
};

fn fresh_root(label: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "vlajmaji-s02-{label}-{}-{stamp}",
        std::process::id()
    ))
}

fn sealed(pack_id: PackId) -> (Vec<u8>, Vec<u8>) {
    let dictionary = Dictionary::from_document(DictionaryDocument::default()).unwrap();
    let release = seal_release(pack_id, PackPayloadKind::Synthetic, &dictionary).unwrap();
    (release.release, release.seal)
}

fn sealed_input<'a>(pack_id: PackId, release: &'a [u8], seal: &'a [u8]) -> CollectionInput<'a> {
    CollectionInput::SealedBytes {
        pack_id,
        release: Cow::Borrowed(release),
        seal: Cow::Borrowed(seal),
    }
}

#[test]
fn test_request_all_missing_pack_is_rejected() {
    let (release, seal) = sealed(PackId::Llg);
    let error = CollectionRequest::new(
        vec![sealed_input(PackId::Llg, &release, &seal)],
        CollectionSelection::All,
        true,
    )
    .unwrap_err();
    assert!(matches!(error, CollectionError::InvalidRequest(_)));
}

#[test]
fn test_install_revision_overflow_is_known_precommit_failure() {
    let root = fresh_root("revision-overflow");
    let (release, seal) = sealed(PackId::Llg);
    let request = || {
        CollectionRequest::new(
            vec![sealed_input(PackId::Llg, &release, &seal)],
            CollectionSelection::Packs(vec![PackId::Llg]),
            true,
        )
        .unwrap()
    };
    CollectionCatalogue::register(&root, request()).unwrap();
    let connection = Connection::open(root.join("catalogue.sqlite3")).unwrap();
    connection
        .execute(
            "UPDATE catalogue_meta SET value=?1 WHERE key='local_revision'",
            [u64::MAX.to_string()],
        )
        .unwrap();
    drop(connection);

    let error = CollectionCatalogue::install(&root, request()).unwrap_err();
    assert!(
        matches!(&error, CollectionError::MutationContext { source, .. }
        if matches!(source.as_ref(), CollectionError::UnsupportedVersion(_)))
    );
    let report = error.report("install");
    assert_eq!(report.category, "unsupported_version");
    assert_eq!(report.activation.as_deref(), Some("not_activated"));
    assert_eq!(report.prior_local_revision, Some(u64::MAX));
    assert_eq!(report.attempt_id, None);
    assert_eq!(report.proposed_generation, None);
    let inspection = CollectionCatalogue::open(&root).unwrap().list().unwrap();
    assert_eq!(inspection.local_revision, u64::MAX);
    assert!(
        inspection.staged_attempts.is_empty(),
        "overflow is rejected before allocating a stage"
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn test_catalogue_register_then_install_and_repeat_keeps_generation() {
    let root = fresh_root("register-install");
    let (release, seal) = sealed(PackId::Llg);
    let request = || {
        CollectionRequest::new(
            vec![sealed_input(PackId::Llg, &release, &seal)],
            CollectionSelection::Packs(vec![PackId::Llg]),
            true,
        )
        .unwrap()
    };

    let registered = CollectionCatalogue::register(&root, request()).unwrap();
    assert_eq!(registered.outcome, "registered");
    assert_eq!(registered.local_revision, 1);
    assert!(registered.generation.is_none());
    let release_sha256 = registered.inputs[0].release_sha256.clone().unwrap();

    let reader = CollectionCatalogue::open(&root).unwrap();
    let listed = reader.list().unwrap();
    assert_eq!(listed.local_revision, 1);
    assert!(listed.generation.is_none());
    assert_eq!(listed.packs.len(), 1);
    assert_eq!(
        listed.packs[0].registrations[0].input_ownership,
        "embedded_or_borrowed"
    );
    assert_eq!(
        listed.packs[0].registrations[0].installed_pack_copy_ownership,
        "not_installed"
    );
    let references = reader
        .reference_metadata(PackId::Llg, &release_sha256)
        .unwrap();
    assert_eq!(references.version, 1);
    assert_eq!(references.dictionary_version, 1);
    assert_eq!(references.registry_version, 1);
    assert_eq!(references.category_counts.entries, 0);
    assert_eq!(references.category_counts.assertions, 0);
    assert!(references.captures.is_empty());
    drop(reader);

    let install = CollectionCatalogue::install(&root, request()).unwrap();
    assert_eq!(install.outcome, "installed");
    assert_eq!(install.local_revision, 2);
    let first = install.generation.unwrap();
    let verify = DictionaryStoreVerification::verify(
        &first.store_path,
        DictionaryStoreVerificationCheck::All,
    )
    .unwrap();
    assert!(verify.ok);

    let mut formatted_seal = seal.clone();
    formatted_seal.push(b' ');
    let whitespace_request = CollectionRequest::new(
        vec![sealed_input(PackId::Llg, &release, &formatted_seal)],
        CollectionSelection::Packs(vec![PackId::Llg]),
        true,
    )
    .unwrap();
    let repeated = CollectionCatalogue::install(&root, whitespace_request).unwrap();
    assert_eq!(repeated.outcome, "idempotent");
    assert_eq!(repeated.local_revision, 2);
    let second = repeated.generation.unwrap();
    assert_eq!(first.generation_id, second.generation_id);
    assert_eq!(first.content_id, second.content_id);
    assert_eq!(first.canonical_sha256, second.canonical_sha256);
    assert!(first.store_path.is_dir());
    assert_eq!(
        fs::read(
            first
                .store_path
                .parent()
                .unwrap()
                .join("inputs/llg/seal.json")
        )
        .unwrap(),
        seal
    );
    let inspection = CollectionCatalogue::open(&root)
        .unwrap()
        .inspect(PackId::Llg)
        .unwrap();
    assert_eq!(inspection.installed_history.len(), 1);
    assert!(inspection.installed_history[0].current);
    assert_eq!(
        inspection.registrations[0].installed_pack_copy_ownership,
        "tool_owned"
    );
    assert_eq!(
        inspection.registrations[0].derived_store_ownership,
        "tool_owned"
    );

    let dictionary = Dictionary::from_document(Default::default()).unwrap();
    let full = seal_release(PackId::Llg, PackPayloadKind::Full, &dictionary).unwrap();
    assert_eq!(
        full.release, release,
        "payload kind is declared by the seal, not the canonical release bytes"
    );
    let kind_conflict = CollectionRequest::new(
        vec![sealed_input(PackId::Llg, &full.release, &full.seal)],
        CollectionSelection::Packs(vec![PackId::Llg]),
        true,
    )
    .unwrap();
    assert!(matches!(
        CollectionCatalogue::install(&root, kind_conflict),
        Err(CollectionError::MutationContext { source, .. })
            if matches!(source.as_ref(), CollectionError::IdentityConflict { .. })
    ));
    let orphan = root.join("generations").join("uncommitted-test");
    fs::create_dir(&orphan).unwrap();
    fs::write(
        orphan.join("attempt.json"),
        br#"{"version":1,"attempt_id":"orphan-test","operation":"install"}"#,
    )
    .unwrap();
    fs::create_dir(root.join("generations/unmarked-stage")).unwrap();
    let reader = CollectionCatalogue::open(&root).unwrap();
    let staged = reader.list().unwrap().staged_attempts;
    assert!(staged.iter().any(|attempt| attempt == "orphan-test"));
    assert!(staged.iter().any(|attempt| attempt
        == &format!(
            "unmarked:{}",
            vlajmaji_store::native_path_report(std::path::Path::new("unmarked-stage")).hex
        )));
    drop(reader);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            fs::metadata(&root).unwrap().permissions().mode() & 0o777,
            0o700
        );
        assert_eq!(
            fs::metadata(root.join("catalogue.sqlite3"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
        assert_eq!(
            fs::metadata(&first.store_path)
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o700
        );
        assert_eq!(
            fs::metadata(first.store_path.join("dictionary.sqlite3"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
        let stage = first.store_path.parent().unwrap();
        let input_dir = stage.join("inputs/llg");
        assert_eq!(
            fs::metadata(stage).unwrap().permissions().mode() & 0o777,
            0o700
        );
        assert_eq!(
            fs::metadata(&input_dir).unwrap().permissions().mode() & 0o777,
            0o700
        );
        assert_eq!(
            fs::metadata(input_dir.join("release.json"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
    }
    let _ = fs::remove_dir_all(root);
}

#[test]
fn test_new_release_registration_stays_uninstalled_and_install_requires_later_update_flow() {
    let root = fresh_root("new-release-update-required");
    let (release, seal) = sealed(PackId::Llg);
    let old_request = || {
        CollectionRequest::new(
            vec![sealed_input(PackId::Llg, &release, &seal)],
            CollectionSelection::Packs(vec![PackId::Llg]),
            true,
        )
        .unwrap()
    };
    CollectionCatalogue::register(&root, old_request()).unwrap();
    let installed = CollectionCatalogue::install(&root, old_request()).unwrap();
    let old_generation = installed.generation.unwrap();

    let newer_dictionary = fixture_support::complete_fixture().unwrap();
    let newer = seal_release(PackId::Llg, PackPayloadKind::Synthetic, &newer_dictionary).unwrap();
    let newer_request = || {
        CollectionRequest::new(
            vec![CollectionInput::SealedBytes {
                pack_id: PackId::Llg,
                release: Cow::Owned(newer.release.clone()),
                seal: Cow::Owned(newer.seal.clone()),
            }],
            CollectionSelection::Packs(vec![PackId::Llg]),
            true,
        )
        .unwrap()
    };
    CollectionCatalogue::register(&root, newer_request()).unwrap();

    let inspection = CollectionCatalogue::open(&root)
        .unwrap()
        .inspect(PackId::Llg)
        .unwrap();
    assert_eq!(inspection.registrations.len(), 2);
    let old = inspection
        .registrations
        .iter()
        .find(|item| item.release_sha256 == format!("{:x}", sha2::Sha256::digest(&release)))
        .unwrap();
    let new = inspection
        .registrations
        .iter()
        .find(|item| item.release_sha256 == format!("{:x}", sha2::Sha256::digest(&newer.release)))
        .unwrap();
    assert_eq!(old.installed_pack_copy_ownership, "tool_owned");
    assert_eq!(old.derived_store_ownership, "tool_owned");
    assert_eq!(new.installed_pack_copy_ownership, "not_installed");
    assert_eq!(new.derived_store_ownership, "not_installed");

    let error = CollectionCatalogue::install(&root, newer_request()).unwrap_err();
    let report = error.report("install");
    assert_eq!(report.category.as_str(), "update_required");
    assert_eq!(report.activation.as_deref(), Some("not_activated"));
    assert_eq!(
        report.observed_generation.as_deref(),
        Some(old_generation.generation_id.as_str())
    );
    let after = CollectionCatalogue::open(&root).unwrap().list().unwrap();
    assert_eq!(after.local_revision, 3);
    assert_eq!(
        after.generation.unwrap().generation_id,
        old_generation.generation_id
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn test_exact_repeat_rejects_corrupt_owned_store_without_activating_changes() {
    let root = fresh_root("corrupt-owned-repeat");
    let (release, seal) = sealed(PackId::Llg);
    let request = || {
        CollectionRequest::new(
            vec![sealed_input(PackId::Llg, &release, &seal)],
            CollectionSelection::Packs(vec![PackId::Llg]),
            true,
        )
        .unwrap()
    };
    let installed = CollectionCatalogue::install(&root, request()).unwrap();
    let generation = installed.generation.unwrap();
    let before = CollectionCatalogue::open(&root).unwrap().list().unwrap();
    let db = generation.store_path.join("dictionary.sqlite3");
    fs::write(&db, b"corrupt owned database").unwrap();
    let corrupt_bytes = fs::read(&db).unwrap();

    let error = CollectionCatalogue::install(&root, request()).unwrap_err();
    assert!(
        matches!(&error, CollectionError::MutationContext { source, .. }
        if matches!(source.as_ref(), CollectionError::VerificationFailedSource { .. }))
    );
    let failure = error.report("install");
    assert_eq!(failure.category, "verification_failed");
    assert_eq!(failure.activation.as_deref(), Some("not_activated"));
    assert_eq!(failure.prior_local_revision, Some(before.local_revision));
    assert_eq!(failure.proposed_generation, None);
    assert_eq!(
        failure.observed_generation.as_deref(),
        Some(generation.generation_id.as_str())
    );
    let after = CollectionCatalogue::open(&root).unwrap().list().unwrap();
    assert_eq!(after.local_revision, before.local_revision);
    assert_eq!(
        after.generation.unwrap().generation_id,
        generation.generation_id
    );
    assert_eq!(
        fs::read(&db).unwrap(),
        corrupt_bytes,
        "verification does not repair or overwrite owned bytes"
    );
    assert!(
        after.staged_attempts.is_empty(),
        "an exact-repeat verification failure allocates no stage"
    );
    let inspection = CollectionCatalogue::open(&root)
        .unwrap()
        .inspect(PackId::Llg)
        .unwrap();
    assert_eq!(
        inspection.derived_store_minimum.as_ref().unwrap().status,
        "unsupported_or_unreadable"
    );
    assert!(
        inspection
            .derived_store_minimum
            .unwrap()
            .diagnostic
            .is_some()
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn test_sealed_pack_mismatch_is_rejected_before_catalogue_creation() {
    let (release, seal) = sealed(PackId::Llg);
    let root = fresh_root("mismatch");
    let request = CollectionRequest::new(
        vec![sealed_input(PackId::LaLojban, &release, &seal)],
        CollectionSelection::Packs(vec![PackId::LaLojban]),
        true,
    )
    .unwrap();
    let error = CollectionCatalogue::register(&root, request).unwrap_err();
    assert!(
        matches!(&error, CollectionError::Batch { source, .. } if matches!(source.as_ref(), CollectionError::InputInvalidSource { .. }))
    );
    let failure = error.report("register");
    assert_eq!(failure.category, "input_invalid");
    assert_eq!(failure.pack_id.as_deref(), Some("la-lojban"));
    assert_eq!(failure.activation.as_deref(), Some("not_activated"));
    assert_eq!(failure.inputs.len(), 1);
    assert_eq!(failure.inputs[0].disposition, "failed");
    assert!(error_chain_contains_pack_error(&error));
    assert!(!root.exists());
}

fn error_chain_contains_pack_error(error: &(dyn std::error::Error + 'static)) -> bool {
    let mut current = error;
    loop {
        if current
            .downcast_ref::<vlajmaji_lexicon::PackError>()
            .is_some()
        {
            return true;
        }
        let Some(source) = current.source() else {
            return false;
        };
        current = source;
    }
}

#[test]
fn test_empty_existing_roots_register_install_expect_occupied_with_validated_input() {
    for operation in ["register", "install"] {
        let root = fresh_root(&format!("occupied-empty-{operation}"));
        fs::create_dir(&root).unwrap();
        let (release, seal) = sealed(PackId::Llg);
        let expected_release_sha256 = format!("{:x}", sha2::Sha256::digest(&release));
        let request = CollectionRequest::new(
            vec![sealed_input(PackId::Llg, &release, &seal)],
            CollectionSelection::Packs(vec![PackId::Llg]),
            true,
        )
        .unwrap();
        let error = if operation == "register" {
            CollectionCatalogue::register(&root, request).unwrap_err()
        } else {
            CollectionCatalogue::install(&root, request).unwrap_err()
        };
        let report = error.report(operation);
        assert_eq!(
            error.category(),
            vlajmaji_store::CollectionErrorCategory::OccupiedDestination
        );
        assert_eq!(report.category, "occupied_destination");
        assert_eq!(report.activation.as_deref(), Some("not_activated"));
        assert_eq!(report.path.as_ref().unwrap().utf8.as_deref(), root.to_str());
        assert_eq!(report.inputs.len(), 1);
        assert_eq!(report.inputs[0].pack_id, "llg");
        assert_eq!(report.inputs[0].disposition, "validated");
        assert_eq!(report.inputs[0].payload_kind.as_deref(), Some("synthetic"));
        assert_eq!(
            report.inputs[0].release_sha256.as_deref(),
            Some(expected_release_sha256.as_str())
        );
        assert!(report.attempt_id.is_none());
        assert!(report.prior_local_revision.is_none());
        assert!(report.proposed_generation.is_none());
        assert_eq!(fs::read_dir(&root).unwrap().count(), 0);
        fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn test_begin_busy_register_install_expect_validated_context_and_retry() {
    for operation in ["register", "install"] {
        let root = fresh_root(&format!("begin-busy-{operation}"));
        let (existing_release, existing_seal) = sealed(PackId::Llg);
        let existing = CollectionRequest::new(
            vec![sealed_input(PackId::Llg, &existing_release, &existing_seal)],
            CollectionSelection::Packs(vec![PackId::Llg]),
            true,
        )
        .unwrap();
        CollectionCatalogue::register(&root, existing).unwrap();

        let (release, seal) = sealed(PackId::LensiskuJbotci);
        let expected_release_sha256 = format!("{:x}", sha2::Sha256::digest(&release));
        let make_request = || {
            CollectionRequest::new(
                vec![sealed_input(PackId::LensiskuJbotci, &release, &seal)],
                CollectionSelection::Packs(vec![PackId::LensiskuJbotci]),
                true,
            )
            .unwrap()
        };
        let blocker = Connection::open(root.join("catalogue.sqlite3")).unwrap();
        blocker.execute_batch("BEGIN IMMEDIATE").unwrap();
        let error = if operation == "register" {
            CollectionCatalogue::register(&root, make_request()).unwrap_err()
        } else {
            CollectionCatalogue::install(&root, make_request()).unwrap_err()
        };
        let report = error.report(operation);
        assert_eq!(
            error.category(),
            vlajmaji_store::CollectionErrorCategory::Busy
        );
        assert_eq!(report.category, "busy");
        assert_eq!(report.activation.as_deref(), Some("not_activated"));
        assert_eq!(report.inputs.len(), 1);
        assert_eq!(report.inputs[0].pack_id, "lensisku-jbotci");
        assert_eq!(report.inputs[0].disposition, "validated");
        assert_eq!(report.inputs[0].payload_kind.as_deref(), Some("synthetic"));
        assert_eq!(
            report.inputs[0].release_sha256.as_deref(),
            Some(expected_release_sha256.as_str())
        );
        assert!(report.attempt_id.is_none());
        assert!(report.prior_local_revision.is_none());
        assert!(report.proposed_generation.is_none());
        assert!(report.observed_generation.is_none());
        blocker.execute_batch("ROLLBACK").unwrap();
        drop(blocker);
        let retry = if operation == "register" {
            CollectionCatalogue::register(&root, make_request()).unwrap()
        } else {
            CollectionCatalogue::install(&root, make_request()).unwrap()
        };
        assert_eq!(retry.local_revision, 2);
        fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn test_missing_sealed_input_expect_known_pack_and_typed_io_cause() {
    let root = fresh_root("missing-selected-pack");
    fs::create_dir(&root).unwrap();
    let missing = root.join("missing sutysisku release.json");
    let seal_path = root.join("missing sutysisku seal.json");
    let request = || {
        CollectionRequest::new(
            vec![CollectionInput::SealedFiles {
                pack_id: PackId::Sutysisku,
                release: missing.clone(),
                seal: seal_path.clone(),
            }],
            CollectionSelection::Packs(vec![PackId::Sutysisku]),
            true,
        )
        .unwrap()
    };
    let catalogue = root.join("catalogue");
    let error = CollectionCatalogue::install(&catalogue, request()).unwrap_err();
    let report = error.report("install");
    assert_eq!(
        error.category(),
        vlajmaji_store::CollectionErrorCategory::InputIo
    );
    assert_eq!(report.category, "input_io");
    assert_eq!(report.pack_id.as_deref(), Some("sutysisku"));
    assert_eq!(
        report.path.as_ref().unwrap().utf8.as_deref(),
        missing.to_str()
    );
    assert_eq!(report.inputs.len(), 1);
    assert_eq!(report.inputs[0].pack_id, "sutysisku");
    assert_eq!(report.inputs[0].disposition, "failed");
    assert!(error_chain_contains_io_error(&error));
    assert!(!catalogue.exists());
    let (release, seal) = sealed(PackId::Sutysisku);
    fs::write(&missing, release).unwrap();
    fs::write(&seal_path, seal).unwrap();
    let retry = CollectionCatalogue::install(&catalogue, request()).unwrap();
    assert_eq!(retry.outcome, "installed");
    assert_eq!(retry.local_revision, 1);
    if root.exists() {
        fs::remove_dir_all(root).unwrap();
    }
}

fn error_chain_contains_io_error(error: &(dyn std::error::Error + 'static)) -> bool {
    let mut current = error;
    loop {
        if current.downcast_ref::<std::io::Error>().is_some() {
            return true;
        }
        let Some(source) = current.source() else {
            return false;
        };
        current = source;
    }
}

#[cfg(target_os = "macos")]
fn error_chain_raw_os_error(error: &(dyn std::error::Error + 'static)) -> Option<i32> {
    let mut current = error;
    loop {
        if let Some(io_error) = current.downcast_ref::<std::io::Error>() {
            return io_error.raw_os_error();
        }
        current = current.source()?;
    }
}

#[test]
fn test_malformed_seal_and_release_mismatch_expect_typed_pack_errors() {
    let root = fresh_root("sealed-pack-causes");
    let malformed = CollectionRequest::new(
        vec![CollectionInput::SealedBytes {
            pack_id: PackId::Llg,
            release: Cow::Borrowed(b"{}"),
            seal: Cow::Borrowed(b"{not json"),
        }],
        CollectionSelection::Packs(vec![PackId::Llg]),
        true,
    )
    .unwrap();
    let malformed_error = CollectionCatalogue::install(&root, malformed).unwrap_err();
    assert_eq!(
        malformed_error.category(),
        vlajmaji_store::CollectionErrorCategory::InputInvalid
    );
    assert_eq!(
        malformed_error.report("install").pack_id.as_deref(),
        Some("llg")
    );
    assert!(error_chain_contains_pack_error(&malformed_error));
    assert!(!root.exists());

    let (release, seal) = sealed(PackId::Llg);
    let mut changed_release = release.clone();
    changed_release[0] ^= 1;
    let mismatch = CollectionRequest::new(
        vec![sealed_input(PackId::Llg, &changed_release, &seal)],
        CollectionSelection::Packs(vec![PackId::Llg]),
        true,
    )
    .unwrap();
    let mismatch_error = CollectionCatalogue::install(&root, mismatch).unwrap_err();
    assert_eq!(
        mismatch_error.category(),
        vlajmaji_store::CollectionErrorCategory::InputInvalid
    );
    assert_eq!(
        mismatch_error.report("install").pack_id.as_deref(),
        Some("llg")
    );
    assert!(error_chain_contains_pack_error(&mismatch_error));
    assert!(!root.exists());
}

#[test]
fn test_install_commit_busy_rolls_back_catalogue_and_retains_staged_attempt() {
    let root = fresh_root("commit-busy");
    let (llg_release, llg_seal) = sealed(PackId::Llg);
    let llg_request = || {
        CollectionRequest::new(
            vec![sealed_input(PackId::Llg, &llg_release, &llg_seal)],
            CollectionSelection::Packs(vec![PackId::Llg]),
            true,
        )
        .unwrap()
    };
    CollectionCatalogue::install(&root, llg_request()).unwrap();
    let before = CollectionCatalogue::open(&root).unwrap().list().unwrap();
    let prior_summary = before.generation.unwrap();
    let prior_generation = prior_summary.generation_id.clone();
    let prior_store = prior_summary.store_path.clone();
    let prior_store_bytes = fs::read(prior_store.join("dictionary.sqlite3")).unwrap();

    let reader = Connection::open(root.join("catalogue.sqlite3")).unwrap();
    reader.busy_timeout(std::time::Duration::ZERO).unwrap();
    reader.execute_batch("BEGIN").unwrap();
    let revision: String = reader
        .query_row(
            "SELECT value FROM catalogue_meta WHERE key='local_revision'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(revision, "1");

    let (jb_release, jb_seal) = sealed(PackId::LensiskuJbotci);
    let jb_request = || {
        CollectionRequest::new(
            vec![sealed_input(PackId::LensiskuJbotci, &jb_release, &jb_seal)],
            CollectionSelection::Packs(vec![PackId::LensiskuJbotci]),
            true,
        )
        .unwrap()
    };
    let error = CollectionCatalogue::install(&root, jb_request()).unwrap_err();
    assert!(
        matches!(&error, CollectionError::MutationContext { source, .. } if matches!(source.as_ref(), CollectionError::BusySource(_))),
        "{error}"
    );
    let report = error.report("install");
    assert_eq!(report.category, "busy");
    assert_eq!(report.activation.as_deref(), Some("not_activated"));
    assert!(report.attempt_id.is_some());
    assert_eq!(report.prior_local_revision, Some(1));
    assert!(report.proposed_generation.is_some());
    assert_eq!(
        report.observed_generation.as_deref(),
        Some(prior_generation.as_str())
    );
    assert_eq!(report.inputs.len(), 1);
    assert_eq!(report.inputs[0].pack_id, "lensisku-jbotci");
    assert_eq!(report.inputs[0].disposition, "validated");
    assert!(report.inputs[0].release_sha256.is_some());
    let mutation_source = std::error::Error::source(&error)
        .unwrap()
        .downcast_ref::<CollectionError>()
        .unwrap();
    assert!(
        std::error::Error::source(mutation_source)
            .unwrap()
            .downcast_ref::<rusqlite::Error>()
            .is_some()
    );
    reader.execute_batch("ROLLBACK").unwrap();
    drop(reader);

    let after = CollectionCatalogue::open(&root).unwrap().list().unwrap();
    assert_eq!(
        after.local_revision, 1,
        "confirmed rollback must preserve the prior revision"
    );
    assert_eq!(after.generation.unwrap().generation_id, prior_generation);
    assert_eq!(
        after
            .packs
            .iter()
            .map(|pack| pack.pack_id.as_str())
            .collect::<Vec<_>>(),
        vec!["llg"]
    );
    assert_eq!(
        after.staged_attempts.len(),
        1,
        "the completed uncommitted stage remains inspectable"
    );
    assert_eq!(
        fs::read(prior_store.join("dictionary.sqlite3")).unwrap(),
        prior_store_bytes
    );

    let valid_retry = CollectionCatalogue::install(&root, jb_request()).unwrap();
    assert_eq!(valid_retry.local_revision, 2);
    let next_generation = valid_retry.generation.unwrap();
    assert_ne!(next_generation.generation_id, prior_generation);
    assert!(
        prior_store.is_dir(),
        "the prior generation stays immutable and available"
    );
    assert_eq!(
        fs::read(prior_store.join("dictionary.sqlite3")).unwrap(),
        prior_store_bytes
    );
    let after_retry = CollectionCatalogue::open(&root).unwrap().list().unwrap();
    assert_eq!(after_retry.local_revision, 2);
    assert_eq!(
        after_retry
            .packs
            .iter()
            .map(|pack| pack.pack_id.as_str())
            .collect::<Vec<_>>(),
        vec!["lensisku-jbotci", "llg"]
    );
    assert_eq!(
        after_retry.staged_attempts.len(),
        1,
        "the failed precommit stage remains distinct from the committed retry"
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn test_install_conflicting_shared_source_is_typed_and_valid_counterpart_composes_once() {
    let root = fresh_root("shared-source-conflict");
    let mut registry_document = RegistryDocument::default();
    registry_document.sources.push(SourceRecord {
        id: SourceId::new("source-a").unwrap(),
        display_name: "Captured A".into(),
        abbreviation: "a".into(),
        abbreviation_kind: AbbreviationKind::ProjectAssigned,
        abbreviation_evidence: None,
        origin: SourceOrigin::Upstream,
    });
    let mut base_document = DictionaryDocument {
        registry: SourceRegistry::from_document(registry_document).unwrap(),
        ..DictionaryDocument::default()
    };
    base_document.metadata.collections.push(CollectionRecord {
        id: EntityId::new("collection-a").unwrap(),
        source: SourceId::new("source-a").unwrap(),
        display_name: "Collection A".into(),
        description: Presence::Missing,
    });
    let jbotci = Dictionary::from_document(base_document).unwrap();
    let jb = seal_release(PackId::LensiskuJbotci, PackPayloadKind::Synthetic, &jbotci).unwrap();
    let first = CollectionRequest::new(
        vec![CollectionInput::SealedBytes {
            pack_id: PackId::LensiskuJbotci,
            release: Cow::Owned(jb.release.clone()),
            seal: Cow::Owned(jb.seal.clone()),
        }],
        CollectionSelection::Packs(vec![PackId::LensiskuJbotci]),
        true,
    )
    .unwrap();
    let initial = CollectionCatalogue::install(&root, first).unwrap();
    let initial_generation = initial.generation.unwrap();

    let mut conflicting_document = jbotci.clone().into_document();
    let mut registry = conflicting_document.registry.into_document();
    registry
        .sources
        .iter_mut()
        .find(|row| row.id.as_str() == "source-a")
        .unwrap()
        .display_name = "Different source body".into();
    conflicting_document.registry = SourceRegistry::from_document(registry).unwrap();
    let conflicting = Dictionary::from_document(conflicting_document).unwrap();
    let la_bad = seal_release(PackId::LaLojban, PackPayloadKind::Synthetic, &conflicting).unwrap();
    let conflict_request = CollectionRequest::new(
        vec![CollectionInput::SealedBytes {
            pack_id: PackId::LaLojban,
            release: Cow::Owned(la_bad.release),
            seal: Cow::Owned(la_bad.seal),
        }],
        CollectionSelection::Packs(vec![PackId::LaLojban]),
        true,
    )
    .unwrap();
    let error = CollectionCatalogue::install(&root, conflict_request).unwrap_err();
    assert!(
        matches!(&error, CollectionError::MutationContext { source, .. }
        if matches!(source.as_ref(), CollectionError::PackConflict { category, key, .. }
            if category == "source" && key == "source-a"))
    );
    assert!(
        std::error::Error::source(&error)
            .unwrap()
            .source()
            .is_some(),
        "original typed PackError remains in the source chain"
    );
    let failure = error.report("install");
    assert_eq!(failure.category, "identity_conflict");
    assert_eq!(failure.activation.as_deref(), Some("not_activated"));
    assert_eq!(failure.prior_local_revision, Some(1));
    assert_eq!(
        failure.observed_generation.as_deref(),
        Some(initial_generation.generation_id.as_str())
    );
    assert!(failure.attempt_id.is_some());
    assert!(failure.proposed_generation.is_some());
    assert_eq!(failure.inputs.len(), 1);
    assert_eq!(failure.inputs[0].pack_id, "la-lojban");
    assert_eq!(failure.inputs[0].disposition, "validated");
    let after_conflict = CollectionCatalogue::open(&root).unwrap().list().unwrap();
    assert_eq!(after_conflict.local_revision, 1);
    assert_eq!(after_conflict.packs.len(), 1);

    let la_valid = seal_release(PackId::LaLojban, PackPayloadKind::Synthetic, &jbotci).unwrap();
    let valid_request = CollectionRequest::new(
        vec![CollectionInput::SealedBytes {
            pack_id: PackId::LaLojban,
            release: Cow::Owned(la_valid.release),
            seal: Cow::Owned(la_valid.seal),
        }],
        CollectionSelection::Packs(vec![PackId::LaLojban]),
        true,
    )
    .unwrap();
    let installed = CollectionCatalogue::install(&root, valid_request).unwrap();
    let generation = installed.generation.unwrap();
    let composed = vlajmaji_store::DictionaryStore::open(&generation.store_path)
        .unwrap()
        .dictionary()
        .unwrap();
    assert_eq!(
        composed
            .registry()
            .as_document()
            .sources
            .iter()
            .filter(|row| row.id.as_str() == "source-a")
            .count(),
        1
    );
    assert_eq!(
        composed.metadata().entries.len(),
        jbotci.metadata().entries.len()
    );

    let mut conflicting_collection_document = jbotci.clone().into_document();
    conflicting_collection_document.metadata.collections[0].display_name =
        "Different collection body".into();
    let conflicting_collection =
        Dictionary::from_document(conflicting_collection_document).unwrap();
    let sut_bad = seal_release(
        PackId::Sutysisku,
        PackPayloadKind::Synthetic,
        &conflicting_collection,
    )
    .unwrap();
    let conflict_request = CollectionRequest::new(
        vec![CollectionInput::SealedBytes {
            pack_id: PackId::Sutysisku,
            release: Cow::Owned(sut_bad.release),
            seal: Cow::Owned(sut_bad.seal),
        }],
        CollectionSelection::Packs(vec![PackId::Sutysisku]),
        true,
    )
    .unwrap();
    let error = CollectionCatalogue::install(&root, conflict_request).unwrap_err();
    assert!(
        matches!(&error, CollectionError::MutationContext { source, .. }
        if matches!(source.as_ref(), CollectionError::PackConflict { category, key, .. }
            if category == "collection" && key == "collection-a"))
    );
    let after_conflict = CollectionCatalogue::open(&root).unwrap().list().unwrap();
    assert_eq!(after_conflict.local_revision, 2);
    assert_eq!(after_conflict.packs.len(), 2);

    let sut_valid = seal_release(PackId::Sutysisku, PackPayloadKind::Synthetic, &jbotci).unwrap();
    let valid_request = CollectionRequest::new(
        vec![CollectionInput::SealedBytes {
            pack_id: PackId::Sutysisku,
            release: Cow::Owned(sut_valid.release),
            seal: Cow::Owned(sut_valid.seal),
        }],
        CollectionSelection::Packs(vec![PackId::Sutysisku]),
        true,
    )
    .unwrap();
    let installed = CollectionCatalogue::install(&root, valid_request).unwrap();
    let generation = installed.generation.unwrap();
    let composed = vlajmaji_store::DictionaryStore::open(&generation.store_path)
        .unwrap()
        .dictionary()
        .unwrap();
    assert_eq!(
        composed
            .metadata()
            .collections
            .iter()
            .filter(|row| row.id.as_str() == "collection-a")
            .count(),
        1
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn test_reference_metadata_preserves_nonempty_form_and_editorial_link_sets() {
    let root = fresh_root("reference-link-sets");
    let dictionary = fixture_support::complete_fixture().unwrap();
    let expected_forms = dictionary.metadata().forms.clone();
    let expected_editorial = dictionary.metadata().editorial_senses.clone();
    let sealed = seal_release(PackId::Llg, PackPayloadKind::Synthetic, &dictionary).unwrap();
    let request = CollectionRequest::new(
        vec![CollectionInput::SealedBytes {
            pack_id: PackId::Llg,
            release: Cow::Owned(sealed.release.clone()),
            seal: Cow::Owned(sealed.seal.clone()),
        }],
        CollectionSelection::Packs(vec![PackId::Llg]),
        true,
    )
    .unwrap();
    let report = CollectionCatalogue::register(&root, request).unwrap();
    let digest = report.inputs[0].release_sha256.as_ref().unwrap();
    let descriptor = CollectionCatalogue::open(&root)
        .unwrap()
        .reference_metadata(PackId::Llg, digest)
        .unwrap();
    assert_eq!(descriptor.category_counts.forms, expected_forms.len());
    assert_eq!(
        descriptor.category_counts.editorial_senses,
        expected_editorial.len()
    );
    for form in expected_forms {
        let link = descriptor
            .entities
            .iter()
            .find(|item| item.role == "form" && item.id == form.id.as_str())
            .unwrap();
        assert_eq!(link.entry_ids, vec![form.entry.as_str().to_owned()]);
        let source = dictionary
            .metadata()
            .entries
            .iter()
            .find(|entry| entry.id == form.entry)
            .unwrap()
            .source
            .as_str();
        assert_eq!(link.source_ids, vec![source.to_owned()]);
        assert_eq!(link.evidence, form.evidence);
    }
    let editorial = &expected_editorial[0];
    let link = descriptor
        .entities
        .iter()
        .find(|item| item.role == "editorial_sense" && item.id == editorial.id.as_str())
        .unwrap();
    let expected_assertions = editorial
        .assertions
        .iter()
        .map(|id| id.as_str().to_owned())
        .collect::<Vec<_>>();
    let mut expected_entries = editorial
        .assertions
        .iter()
        .filter_map(|assertion| {
            dictionary
                .metadata()
                .assertions
                .iter()
                .find(|row| row.id == *assertion)
                .map(|row| row.entry.as_str().to_owned())
        })
        .collect::<Vec<_>>();
    expected_entries.sort();
    expected_entries.dedup();
    assert_eq!(link.assertion_ids, expected_assertions);
    assert_eq!(link.entry_ids, expected_entries);
    assert_eq!(
        link.source_ids,
        vec!["source-a".to_owned(), "source-b".to_owned()]
    );
    assert_eq!(
        descriptor.body_sha256.get("owned_head:source-o"),
        Some(&format!(
            "{:x}",
            sha2::Sha256::digest(
                serde_json::to_vec(&dictionary.metadata().owned_heads[0]).unwrap()
            )
        ))
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn test_list_uses_compact_registration_summary_without_decoding_descriptor() {
    let root = fresh_root("summary-isolation");
    let dictionary = fixture_support::complete_fixture().unwrap();
    let sealed = seal_release(PackId::Llg, PackPayloadKind::Synthetic, &dictionary).unwrap();
    let request = CollectionRequest::new(
        vec![CollectionInput::SealedBytes {
            pack_id: PackId::Llg,
            release: Cow::Owned(sealed.release),
            seal: Cow::Owned(sealed.seal),
        }],
        CollectionSelection::Packs(vec![PackId::Llg]),
        true,
    )
    .unwrap();
    let registered = CollectionCatalogue::register(&root, request).unwrap();
    let digest = registered.inputs[0].release_sha256.as_ref().unwrap();

    let connection = Connection::open(root.join("catalogue.sqlite3")).unwrap();
    connection.execute(
        "UPDATE registrations SET descriptor_json=x'00' WHERE pack_id='llg' AND release_sha256=?1",
        [digest],
    ).unwrap();
    drop(connection);

    let catalogue = CollectionCatalogue::open(&root).unwrap();
    let list = catalogue.list().unwrap();
    let summary = &list.packs[0].registrations[0];
    assert_eq!(
        summary.reference_counts.forms,
        dictionary.metadata().forms.len()
    );
    assert_eq!(
        summary.reference_counts.editorial_senses,
        dictionary.metadata().editorial_senses.len()
    );
    assert_eq!(
        summary.source_ids,
        vec!["source-a", "source-b", "source-o", "source-m"]
    );
    assert_eq!(summary.sources, dictionary.registry().as_document().sources);
    assert_eq!(
        summary.distributors,
        dictionary.registry().as_document().distributors
    );
    assert_eq!(summary.lineage, dictionary.registry().as_document().lineage);
    assert_eq!(summary.captures.len(), dictionary.captures().len());
    assert!(
        summary
            .captures
            .iter()
            .all(|capture| capture.byte_length > 0)
    );
    assert_eq!(summary.collections, dictionary.metadata().collections);
    assert_eq!(summary.components, dictionary.metadata().components);
    assert_eq!(summary.releases.len(), dictionary.metadata().releases.len());
    for (release, expected) in summary.releases.iter().zip(&dictionary.metadata().releases) {
        assert_eq!(release.id, expected.id.as_str());
        assert_eq!(release.accounting, expected.accounting);
        assert_eq!(
            release.redistribution_evidence,
            expected.redistribution_evidence
        );
    }
    assert!(matches!(
        catalogue.reference_metadata(PackId::Llg, digest),
        Err(CollectionError::UnsupportedVersion(_))
    ));
    drop(catalogue);
    let _ = fs::remove_dir_all(root);
}

#[cfg(unix)]
#[test]
fn test_catalogue_root_symlink_is_not_followed_for_read_or_write() {
    use std::os::unix::fs::symlink;

    let root = fresh_root("symlink-target");
    let alias = fresh_root("symlink-alias");
    let (release, seal) = sealed(PackId::Llg);
    let request = || {
        CollectionRequest::new(
            vec![sealed_input(PackId::Llg, &release, &seal)],
            CollectionSelection::Packs(vec![PackId::Llg]),
            true,
        )
        .unwrap()
    };
    CollectionCatalogue::register(&root, request()).unwrap();
    let before = fs::read(root.join("catalogue.sqlite3")).unwrap();
    symlink(&root, &alias).unwrap();

    assert!(matches!(
        CollectionCatalogue::open(&alias),
        Err(CollectionError::MissingCatalogue(_))
    ));
    let error = CollectionCatalogue::register(&alias, request()).unwrap_err();
    assert_eq!(
        error.category(),
        vlajmaji_store::CollectionErrorCategory::OccupiedDestination
    );
    assert_eq!(
        error.report("register").path.unwrap().utf8.as_deref(),
        alias.to_str()
    );
    assert_eq!(fs::read(root.join("catalogue.sqlite3")).unwrap(), before);
    assert!(
        fs::symlink_metadata(&alias)
            .unwrap()
            .file_type()
            .is_symlink()
    );
    let _ = fs::remove_file(alias);
    let _ = fs::remove_dir_all(root);
}

#[cfg(unix)]
#[test]
fn test_catalogue_file_and_database_symlink_are_refused_without_modification() {
    use std::os::unix::fs::symlink;

    let file_root = fresh_root("root-file");
    fs::write(&file_root, b"root sentinel").unwrap();
    let file_before = fs::read(&file_root).unwrap();
    assert!(matches!(
        CollectionCatalogue::open(&file_root),
        Err(CollectionError::MissingCatalogue(_))
    ));
    let (release, seal) = sealed(PackId::Llg);
    let file_request = CollectionRequest::new(
        vec![sealed_input(PackId::Llg, &release, &seal)],
        CollectionSelection::Packs(vec![PackId::Llg]),
        true,
    )
    .unwrap();
    let error = CollectionCatalogue::register(&file_root, file_request).unwrap_err();
    assert_eq!(
        error.category(),
        vlajmaji_store::CollectionErrorCategory::OccupiedDestination
    );
    assert_eq!(
        error.report("register").path.unwrap().utf8.as_deref(),
        file_root.to_str()
    );
    assert_eq!(fs::read(&file_root).unwrap(), file_before);

    let root = fresh_root("database-symlink");
    let request = CollectionRequest::new(
        vec![sealed_input(PackId::Llg, &release, &seal)],
        CollectionSelection::Packs(vec![PackId::Llg]),
        true,
    )
    .unwrap();
    CollectionCatalogue::register(&root, request).unwrap();
    let db = root.join("catalogue.sqlite3");
    let moved_db = root.join("catalogue.sqlite3.retained");
    fs::rename(&db, &moved_db).unwrap();
    let target_bytes = fs::read(&moved_db).unwrap();
    symlink(&moved_db, &db).unwrap();
    assert!(matches!(
        CollectionCatalogue::open(&root),
        Err(CollectionError::MissingCatalogue(_))
    ));
    let write_request = CollectionRequest::new(
        vec![sealed_input(PackId::Llg, &release, &seal)],
        CollectionSelection::Packs(vec![PackId::Llg]),
        true,
    )
    .unwrap();
    let error = CollectionCatalogue::register(&root, write_request).unwrap_err();
    assert_eq!(
        error.category(),
        vlajmaji_store::CollectionErrorCategory::OccupiedDestination
    );
    assert_eq!(
        error.report("register").path.unwrap().utf8.as_deref(),
        db.to_str()
    );
    assert_eq!(fs::read(&moved_db).unwrap(), target_bytes);
    assert!(fs::symlink_metadata(&db).unwrap().file_type().is_symlink());
    let _ = fs::remove_file(&db);
    let _ = fs::remove_dir_all(root);
    let _ = fs::remove_file(file_root);
}

#[test]
fn test_catalogue_unsupported_schema_version_is_read_only() {
    let root = fresh_root("unsupported-version");
    let (release, seal) = sealed(PackId::Llg);
    let request = CollectionRequest::new(
        vec![sealed_input(PackId::Llg, &release, &seal)],
        CollectionSelection::Packs(vec![PackId::Llg]),
        true,
    )
    .unwrap();
    CollectionCatalogue::register(&root, request).unwrap();
    let db = root.join("catalogue.sqlite3");
    let connection = Connection::open(&db).unwrap();
    connection
        .execute(
            "UPDATE catalogue_meta SET value='999' WHERE key='schema_version'",
            [],
        )
        .unwrap();
    drop(connection);
    let before = fs::read(&db).unwrap();
    assert!(matches!(
        CollectionCatalogue::open(&root),
        Err(CollectionError::UnsupportedVersion(_))
    ));
    assert_eq!(fs::read(&db).unwrap(), before);
    let _ = fs::remove_dir_all(root);
}

#[cfg(unix)]
#[test]
fn test_native_non_utf8_catalogue_root_preserves_bytes_or_reports_host_io() {
    use std::{
        ffi::OsString,
        os::unix::ffi::{OsStrExt, OsStringExt},
    };

    let parent = fresh_root("native-root-parent");
    fs::create_dir(&parent).unwrap();
    let root = parent.join(OsString::from_vec(b"catalogue-\xff".to_vec()));
    let report = vlajmaji_store::native_path_report(&root);
    let expected_hex = root
        .as_os_str()
        .as_bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    assert_eq!(report.encoding, "unix-bytes");
    assert_eq!(report.hex, expected_hex);
    assert!(report.utf8.is_none());
    #[cfg(target_os = "macos")]
    let entries_before = fs::read_dir(&parent)
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect::<Vec<_>>();
    let direct_mkdir = fs::create_dir(&root);

    let (release, seal) = sealed(PackId::Llg);
    let request = CollectionRequest::new(
        vec![sealed_input(PackId::Llg, &release, &seal)],
        CollectionSelection::Packs(vec![PackId::Llg]),
        true,
    )
    .unwrap();
    #[cfg(target_os = "macos")]
    {
        let direct_error = direct_mkdir.unwrap_err();
        assert_eq!(direct_error.raw_os_error(), Some(92));
        let error = CollectionCatalogue::register(&root, request).unwrap_err();
        let report = error.report("register");
        assert_eq!(report.category, "artifact_io");
        assert_eq!(
            report.path.as_ref().map(|path| path.hex.as_str()),
            Some(expected_hex.as_str())
        );
        assert_eq!(
            error_chain_raw_os_error(&error),
            direct_error.raw_os_error()
        );
        assert!(!root.exists());
        let entries_after = fs::read_dir(&parent)
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect::<Vec<_>>();
        assert_eq!(entries_after, entries_before);
    }
    #[cfg(target_os = "linux")]
    {
        direct_mkdir.unwrap();
        fs::remove_dir(&root).unwrap();
        CollectionCatalogue::register(&root, request).unwrap();
        assert!(CollectionCatalogue::open(&root).is_ok());
        let _ = fs::remove_dir_all(&root);
    }
    let _ = fs::remove_dir_all(parent);
}

#[cfg(target_os = "linux")]
#[test]
fn test_sealed_input_native_non_utf8_path_is_preserved_in_inspection() {
    use std::{
        ffi::OsString,
        os::unix::ffi::{OsStrExt, OsStringExt},
    };

    let scratch = fresh_root("native-input");
    fs::create_dir(&scratch).unwrap();
    let input_dir = scratch.join("input files");
    fs::create_dir(&input_dir).unwrap();
    let release = input_dir.join(OsString::from_vec(b"release-\xff.json".to_vec()));
    let seal_path = input_dir.join("seal.json");
    let (release_bytes, seal_bytes) = sealed(PackId::Llg);
    fs::write(&release, &release_bytes).unwrap();
    fs::write(&seal_path, &seal_bytes).unwrap();

    let request = CollectionRequest::new(
        vec![CollectionInput::SealedFiles {
            pack_id: PackId::Llg,
            release: release.clone(),
            seal: seal_path,
        }],
        CollectionSelection::Packs(vec![PackId::Llg]),
        true,
    )
    .unwrap();
    CollectionCatalogue::register(scratch.join("catalogue"), request).unwrap();
    let inspection = CollectionCatalogue::open(scratch.join("catalogue"))
        .unwrap()
        .inspect(PackId::Llg)
        .unwrap();
    let recorded = inspection.registrations[0]
        .inputs
        .iter()
        .find(|path| path.utf8.is_none())
        .unwrap();
    let expected_hex = release
        .as_os_str()
        .as_bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    assert_eq!(recorded.encoding, "unix-bytes");
    assert_eq!(recorded.hex, expected_hex);
    assert!(inspection.registrations[0].inputs_present);
    let _ = fs::remove_dir_all(scratch);
}
