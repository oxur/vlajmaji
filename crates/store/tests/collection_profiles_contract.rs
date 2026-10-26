//! Explicit-file collection profile parsing and persistence contracts.

use std::{
    borrow::Cow,
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use vlajmaji_lexicon::{Dictionary, PackId, PackPayloadKind, seal_release};
use vlajmaji_store::{
    CollectionCatalogue, CollectionInput, CollectionProfile, CollectionProfileError,
    CollectionProfileMembershipWriteRequest, CollectionProfileWriteRequest, CollectionProfiles,
    CollectionRequest, CollectionSelection,
};

fn scratch(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "vlajmaji-s03-profile-{label}-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir(&path).unwrap();
    path
}

fn register_empty(root: &std::path::Path) {
    register_pack(root, PackId::Llg);
}

fn register_pack(root: &std::path::Path, pack_id: PackId) {
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
fn test_membership_writer_preserves_disabled_policy_and_explicit_writer_can_clear_it() {
    let root = scratch("membership-preserves-disabled");
    let catalogue_root = root.join("catalogue");
    register_empty(&catalogue_root);
    let config_path = root.join("config.toml");
    let initial = format!(
        "version = 1\nrevision = 7\ndefault_profile = 'other'\n\n[profiles.research]\nroot = {:?}\npacks = []\ndisabled = ['llg']\n\n[profiles.other]\nroot = {:?}\npacks = []\ndisabled = []\n",
        catalogue_root.to_string_lossy(),
        catalogue_root.to_string_lossy()
    );
    fs::write(&config_path, &initial).unwrap();

    let changed = CollectionProfiles::configure_membership(
        &config_path,
        CollectionProfileMembershipWriteRequest {
            name: "research".into(),
            root: catalogue_root.clone(),
            packs: vec![PackId::Llg],
            make_default: true,
        },
    )
    .unwrap();
    assert_eq!(changed.outcome, "committed");
    let configured = CollectionProfiles::read(&config_path).unwrap();
    let research = configured.profile(Some("research")).unwrap();
    assert_eq!(research.packs, vec![PackId::Llg]);
    assert_eq!(research.disabled, vec![PackId::Llg]);
    assert_eq!(
        configured.report().default_profile.as_deref(),
        Some("research")
    );
    let other = configured.profile(Some("other")).unwrap();
    assert_eq!(other.root, catalogue_root);
    assert!(other.packs.is_empty());
    assert!(other.disabled.is_empty());

    let stable_bytes = fs::read(&config_path).unwrap();
    let stable_sha = configured.sha256().to_owned();
    let stable_revision = configured.revision();
    let noop = CollectionProfiles::configure_membership(
        &config_path,
        CollectionProfileMembershipWriteRequest {
            name: "research".into(),
            root: catalogue_root.clone(),
            packs: vec![PackId::Llg],
            make_default: true,
        },
    )
    .unwrap();
    assert_eq!(noop.outcome, "unchanged");
    assert_eq!(noop.sha256, stable_sha);
    assert_eq!(noop.revision, stable_revision);
    assert_eq!(fs::read(&config_path).unwrap(), stable_bytes);

    let explicit = CollectionProfiles::configure(
        &config_path,
        CollectionProfileWriteRequest {
            name: "research".into(),
            root: catalogue_root,
            packs: vec![PackId::Llg],
            disabled: Vec::new(),
            make_default: true,
        },
    )
    .unwrap();
    assert_eq!(explicit.outcome, "committed");
    assert_eq!(
        CollectionProfiles::read(&config_path)
            .unwrap()
            .profile(Some("research"))
            .unwrap()
            .disabled,
        Vec::<PackId>::new()
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn test_membership_writer_new_profile_starts_empty_and_incompatible_root_preserves_old_file() {
    let root = scratch("membership-new-root");
    let catalogue_root = root.join("catalogue");
    register_empty(&catalogue_root);
    let sut_root = root.join("sut-catalogue");
    register_pack(&sut_root, PackId::Sutysisku);
    let config_path = root.join("config.toml");
    let initial = format!(
        "version = 1\nrevision = 3\n\n[profiles.research]\nroot = {:?}\npacks = []\ndisabled = ['llg']\n",
        catalogue_root.to_string_lossy()
    );
    fs::write(&config_path, &initial).unwrap();

    let old = fs::read(&config_path).unwrap();
    let old_state = CollectionProfiles::read(&config_path).unwrap();
    let error = CollectionProfiles::configure_membership(
        &config_path,
        CollectionProfileMembershipWriteRequest {
            name: "research".into(),
            root: sut_root.clone(),
            packs: vec![PackId::Sutysisku],
            make_default: false,
        },
    )
    .unwrap_err();
    assert!(matches!(error, CollectionProfileError::Invalid { .. }));
    assert_eq!(fs::read(&config_path).unwrap(), old);
    let unchanged = CollectionProfiles::read(&config_path).unwrap();
    assert_eq!(unchanged.revision(), old_state.revision());
    assert_eq!(unchanged.sha256(), old_state.sha256());

    let retry = CollectionProfiles::configure_membership(
        &config_path,
        CollectionProfileMembershipWriteRequest {
            name: "research".into(),
            root: catalogue_root,
            packs: vec![PackId::Llg],
            make_default: false,
        },
    )
    .unwrap();
    assert_eq!(retry.outcome, "committed");
    let current = CollectionProfiles::read(&config_path).unwrap();
    assert_eq!(
        current.profile(Some("research")).unwrap().disabled,
        vec![PackId::Llg]
    );

    let new_profile = CollectionProfiles::configure_membership(
        &config_path,
        CollectionProfileMembershipWriteRequest {
            name: "new-profile".into(),
            root: current.profile(Some("research")).unwrap().root,
            packs: vec![PackId::Llg],
            make_default: false,
        },
    )
    .unwrap();
    assert_eq!(new_profile.outcome, "committed");
    let final_state = CollectionProfiles::read(&config_path).unwrap();
    assert_eq!(
        final_state.profile(Some("new-profile")).unwrap().disabled,
        Vec::<PackId>::new()
    );
    assert_eq!(
        final_state.profile(Some("research")).unwrap().disabled,
        vec![PackId::Llg]
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn test_profile_read_preserves_false_zero_empty_and_short_lists() {
    let root = scratch("literal");
    let path = root.join("config.toml");
    fs::write(&path, format!("version = 1\nrevision = 0\ndefault_profile = 'default'\n\n[profiles.default]\nroot = {:?}\npacks = []\ndisabled = []\n", root.to_string_lossy())).unwrap();
    let profiles = CollectionProfiles::read(&path).unwrap();
    assert_eq!(profiles.revision(), 0);
    assert_eq!(profiles.profile(None).unwrap().packs, Vec::<PackId>::new());
    assert_eq!(profiles.report().profiles.len(), 1);
    profiles.verify_unchanged().unwrap();
    fs::remove_dir_all(root).unwrap();
}

#[cfg(unix)]
#[test]
fn test_root_native_roundtrips_exact_bytes_and_rejects_malformed_descriptors() {
    use std::os::unix::ffi::OsStringExt;

    let root = scratch("root-native-wire");
    let config = root.join("profiles.toml");
    let native_root = root.join(std::ffi::OsString::from_vec(b"catalogue-\xff".to_vec()));
    let hex = native_root
        .as_os_str()
        .as_encoded_bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let valid = format!(
        "version = 1\nrevision = 0\n\n[profiles.research]\nroot_native = {{ encoding = \"unix-bytes\", hex = \"{hex}\" }}\npacks = []\ndisabled = []\n"
    );
    fs::write(&config, valid).unwrap();
    let profiles = CollectionProfiles::read(&config).unwrap();
    assert_eq!(
        profiles.profile(Some("research")).unwrap().root,
        native_root
    );

    for descriptor in [
        "{ encoding = \"other\", hex = \"2f746d702f78\" }",
        "{ encoding = \"unix-bytes\", hex = \"2F746d702f78\" }",
        "{ encoding = \"unix-bytes\", hex = \"7a\" }",
        "{ encoding = \"unix-bytes\", hex = \"2f746d702f00\" }",
    ] {
        let malformed = format!(
            "version = 1\nrevision = 0\n\n[profiles.research]\nroot_native = {descriptor}\npacks = []\ndisabled = []\n"
        );
        fs::write(&config, malformed).unwrap();
        assert!(
            matches!(
                CollectionProfiles::read(&config),
                Err(CollectionProfileError::Invalid { .. })
            ),
            "accepted root_native descriptor {descriptor}"
        );
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn test_profile_rejects_unknown_duplicate_and_incomplete_shapes() {
    let root = scratch("invalid");
    let path = root.join("config.toml");
    for text in [
        "version=1\nrevision=0\nunknown=true\n[profiles.default]\nroot='/tmp'\npacks=[]",
        "version=1\nrevision=0\n[profiles.default]\nroot='/tmp'\npacks=[]\npacks=['llg']",
        "version=1\nrevision=0\n[profiles.default]\npacks=[]",
        "version=2\nrevision=0\n[profiles.default]\nroot='/tmp'\npacks=[]",
    ] {
        fs::write(&path, text).unwrap();
        assert!(
            CollectionProfiles::read(&path).is_err(),
            "accepted {text:?}"
        );
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn test_configure_is_idempotent_and_keeps_unrelated_profiles() {
    let root = scratch("configure");
    let catalogue_root = root.join("catalogue");
    register_empty(&catalogue_root);
    let config_path = root.join("settings").join("config.toml");
    fs::create_dir(config_path.parent().unwrap()).unwrap();
    let initial = "version = 1\nrevision = 4\n\n[profiles.other]\nroot = \"/tmp\"\npacks = []\ndisabled = []\n";
    fs::write(&config_path, initial).unwrap();
    let request = CollectionProfileWriteRequest {
        name: "default".into(),
        root: catalogue_root.clone(),
        packs: vec![PackId::Llg],
        disabled: vec![],
        make_default: true,
    };
    let report = CollectionProfiles::configure(&config_path, request.clone()).unwrap();
    assert_eq!(report.outcome, "committed");
    assert_eq!(report.prior_revision, Some(4));
    assert_eq!(report.revision, 5);
    let parsed = CollectionProfiles::read(&config_path).unwrap();
    assert_eq!(
        parsed.profile(None).unwrap(),
        CollectionProfile {
            name: "default".into(),
            root: catalogue_root,
            packs: vec![PackId::Llg],
            disabled: vec![]
        }
    );
    assert!(
        parsed
            .report()
            .profiles
            .iter()
            .any(|profile| profile.name == "other")
    );
    let before = fs::read(&config_path).unwrap();
    let repeat = CollectionProfiles::configure(&config_path, request).unwrap();
    assert_eq!(repeat.outcome, "unchanged");
    assert_eq!(repeat.revision, 5);
    assert_eq!(fs::read(&config_path).unwrap(), before);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn test_configure_requires_registered_pack_and_keeps_old_bytes() {
    let root = scratch("missing-pack");
    let catalogue_root = root.join("catalogue");
    register_empty(&catalogue_root);
    let config_path = root.join("config.toml");
    let old = b"version=1\nrevision=0\n[profiles.old]\nroot='/tmp'\npacks=[]\n";
    fs::write(&config_path, old).unwrap();
    let error = CollectionProfiles::configure(
        &config_path,
        CollectionProfileWriteRequest {
            name: "new".into(),
            root: catalogue_root,
            packs: vec![PackId::Sutysisku],
            disabled: vec![],
            make_default: true,
        },
    )
    .unwrap_err();
    assert!(matches!(error, CollectionProfileError::Invalid { .. }));
    assert_eq!(fs::read(&config_path).unwrap(), old);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn test_configure_busy_sibling_preserves_foreign_file_and_writes_nothing() {
    let root = scratch("busy");
    let catalogue_root = root.join("catalogue");
    register_empty(&catalogue_root);
    let config_path = root.join("config.toml");
    let lock_path = root.join("config.toml.lock");
    let lock_bytes = b"another writer owns this lock\n";
    fs::write(&lock_path, lock_bytes).unwrap();
    let error = CollectionProfiles::configure(
        &config_path,
        CollectionProfileWriteRequest {
            name: "default".into(),
            root: catalogue_root,
            packs: vec![PackId::Llg],
            disabled: vec![],
            make_default: true,
        },
    )
    .unwrap_err();
    assert!(matches!(error, CollectionProfileError::Busy(_)));
    assert!(!config_path.exists());
    assert_eq!(fs::read(&lock_path).unwrap(), lock_bytes);
    fs::remove_dir_all(root).unwrap();
}

#[cfg(unix)]
#[test]
fn test_configure_creates_private_directories_and_preserves_existing_file_mode() {
    use std::os::unix::fs::{PermissionsExt, symlink};

    let root = scratch("modes");
    let catalogue_root = root.join("catalogue");
    register_empty(&catalogue_root);
    let config_path = root.join("new").join("private").join("config.toml");
    CollectionProfiles::configure(
        &config_path,
        CollectionProfileWriteRequest {
            name: "default".into(),
            root: catalogue_root.clone(),
            packs: vec![PackId::Llg],
            disabled: vec![],
            make_default: true,
        },
    )
    .unwrap();
    assert_eq!(
        fs::metadata(config_path.parent().unwrap())
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o700
    );
    assert_eq!(
        fs::metadata(config_path.parent().unwrap().parent().unwrap())
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o700
    );
    assert_eq!(
        fs::metadata(&config_path).unwrap().permissions().mode() & 0o777,
        0o600
    );

    fs::set_permissions(&config_path, fs::Permissions::from_mode(0o640)).unwrap();
    CollectionProfiles::configure(
        &config_path,
        CollectionProfileWriteRequest {
            name: "other".into(),
            root: catalogue_root,
            packs: vec![PackId::Llg],
            disabled: vec![],
            make_default: false,
        },
    )
    .unwrap();
    assert_eq!(
        fs::metadata(&config_path).unwrap().permissions().mode() & 0o777,
        0o640
    );

    let target = root.join("real-config.toml");
    let target_bytes = b"version=1\nrevision=0\nprofiles={}\n";
    fs::write(&target, target_bytes).unwrap();
    let alias = root.join("alias.toml");
    symlink(&target, &alias).unwrap();
    let error = CollectionProfiles::configure(
        &alias,
        CollectionProfileWriteRequest {
            name: "default".into(),
            root: root.join("catalogue"),
            packs: vec![PackId::Llg],
            disabled: vec![],
            make_default: true,
        },
    )
    .unwrap_err();
    assert!(matches!(
        error,
        CollectionProfileError::Invalid { .. } | CollectionProfileError::ConfigDecode { .. }
    ));
    assert_eq!(fs::read(&target).unwrap(), target_bytes);

    let real_parent = root.join("real-parent");
    fs::create_dir(&real_parent).unwrap();
    let ancestor_alias = root.join("ancestor-alias");
    symlink(&real_parent, &ancestor_alias).unwrap();
    let aliased_config = ancestor_alias.join("nested").join("config.toml");
    CollectionProfiles::configure(
        &aliased_config,
        CollectionProfileWriteRequest {
            name: "default".into(),
            root: root.join("catalogue"),
            packs: vec![PackId::Llg],
            disabled: vec![],
            make_default: true,
        },
    )
    .unwrap();
    assert!(real_parent.join("nested/config.toml").is_file());

    let non_directory = root.join("not-a-directory");
    fs::write(&non_directory, b"occupied").unwrap();
    let non_directory_config = non_directory.join("profiles.toml");
    let error = CollectionProfiles::configure(
        &non_directory_config,
        CollectionProfileWriteRequest {
            name: "default".into(),
            root: root.join("catalogue"),
            packs: vec![PackId::Llg],
            disabled: vec![],
            make_default: true,
        },
    )
    .unwrap_err();
    assert!(matches!(
        error,
        CollectionProfileError::Io { source, .. }
            if source.kind() == std::io::ErrorKind::NotADirectory
    ));
    assert!(!non_directory_config.exists());
    fs::remove_dir_all(root).unwrap();
}

#[cfg(unix)]
#[test]
fn test_configure_preserves_native_config_path_when_platform_allows() {
    use std::os::unix::ffi::OsStringExt;

    let root = scratch("native-path-overflow");
    let catalogue_root = root.join("catalogue");
    register_empty(&catalogue_root);
    let config_path = root.join(std::ffi::OsString::from_vec(b"profile-\xff.toml".to_vec()));

    let configured = CollectionProfiles::configure(
        &config_path,
        CollectionProfileWriteRequest {
            name: "default".into(),
            root: catalogue_root.clone(),
            packs: vec![],
            disabled: vec![],
            make_default: true,
        },
    );
    let configured = match configured {
        Ok(report) => report,
        #[cfg(not(target_os = "linux"))]
        Err(CollectionProfileError::Io { path, source }) => {
            // Some macOS volumes reject non-UTF-8 path components before
            // creating the lock. Compare the product result to the direct
            // native filesystem operation rather than treating the platform
            // limitation as a successful-path proof.
            let lock_path = root.join(std::ffi::OsString::from_vec(
                b"profile-\xff.toml.lock".to_vec(),
            ));
            let direct = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&lock_path)
                .unwrap_err();
            assert_eq!(source.kind(), direct.kind());
            assert_eq!(source.raw_os_error(), direct.raw_os_error());
            assert_eq!(path, lock_path);
            assert!(!config_path.exists());
            assert!(!root.join("profile-\u{fffd}.toml").exists());
            fs::remove_dir_all(root).unwrap();
            return;
        }
        Err(error) => panic!("unexpected native path failure: {error}"),
    };
    assert_eq!(configured.path.encoding, "unix-bytes");
    assert_eq!(
        configured.path.hex,
        vlajmaji_store::native_path_report(&config_path).hex
    );
    assert_eq!(configured.path.utf8, None);
    assert_eq!(
        CollectionProfiles::read(&config_path).unwrap().revision(),
        1
    );

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn test_configure_revision_overflow_preserves_existing_bytes() {
    let root = scratch("revision-overflow");
    let catalogue_root = root.join("catalogue");
    register_empty(&catalogue_root);
    let config_path = root.join("profile.toml");
    let overflow_bytes = format!(
        "version = 1\nrevision = {}\ndefault_profile = 'default'\n\n[profiles.default]\nroot = {:?}\npacks = []\ndisabled = []\n",
        i64::MAX,
        catalogue_root.to_string_lossy()
    )
    .into_bytes();
    fs::write(&config_path, &overflow_bytes).unwrap();
    let error = CollectionProfiles::configure(
        &config_path,
        CollectionProfileWriteRequest {
            name: "default".into(),
            root: catalogue_root,
            packs: vec![PackId::Llg],
            disabled: vec![],
            make_default: true,
        },
    )
    .unwrap_err();
    assert!(matches!(
        error,
        CollectionProfileError::Invalid { ref detail, .. }
            if detail.contains("revision overflow")
    ));
    assert_eq!(fs::read(&config_path).unwrap(), overflow_bytes);
    fs::remove_dir_all(root).unwrap();
}
