//! Partial profile enablement/default write contracts.

use std::{
    borrow::Cow,
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use vlajmaji_lexicon::{Dictionary, PackId, PackPayloadKind, seal_release};
use vlajmaji_store::{
    CollectionCatalogue, CollectionInput, CollectionPolicySelection,
    CollectionProfileEnablementRequest, CollectionProfileWriteRequest, CollectionProfiles,
    CollectionRequest, CollectionSelection,
};

fn fresh_root(label: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "vlajmaji-s04-policy-{label}-{}-{stamp}",
        std::process::id()
    ));
    fs::create_dir(&path).unwrap();
    path
}

fn register(root: &Path, pack: PackId) {
    let dictionary = Dictionary::from_document(Default::default()).unwrap();
    let sealed = seal_release(pack, PackPayloadKind::Synthetic, &dictionary).unwrap();
    let request = CollectionRequest::new(
        vec![CollectionInput::SealedBytes {
            pack_id: pack,
            release: Cow::Borrowed(&sealed.release),
            seal: Cow::Borrowed(&sealed.seal),
        }],
        CollectionSelection::Packs(vec![pack]),
        true,
    )
    .unwrap();
    CollectionCatalogue::register(root, request).unwrap();
}

fn configured(path: &Path, root: &Path) {
    CollectionProfiles::configure(
        path,
        CollectionProfileWriteRequest {
            name: "research".into(),
            root: root.to_path_buf(),
            packs: vec![PackId::Llg, PackId::Sutysisku, PackId::LaLojban],
            disabled: vec![PackId::Sutysisku],
            make_default: true,
        },
    )
    .unwrap();
}

#[test]
fn test_partial_enablement_preserves_latest_disabled_order_and_reports_changed_members() {
    let root = fresh_root("partial-enable");
    let catalogue = root.join("catalogue");
    for pack in [PackId::Llg, PackId::Sutysisku, PackId::LaLojban] {
        register(&catalogue, pack);
    }
    let config = root.join("profiles.toml");
    configured(&config, &catalogue);

    // Reversed request order still reports changes in profile membership order.
    let report = CollectionProfiles::set_enabled(
        &config,
        CollectionProfileEnablementRequest {
            profile: None,
            selection: CollectionPolicySelection::Packs(vec![PackId::LaLojban, PackId::Llg]),
            enabled: false,
        },
    )
    .unwrap();
    assert_eq!(report.version, 1);
    assert!(report.ok);
    assert_eq!(report.operation, "disable");
    assert_eq!(report.outcome, "committed");
    assert_eq!(
        report.affected_pack_ids,
        vec![PackId::Llg, PackId::LaLojban]
    );
    assert_eq!(
        report.disabled,
        vec![PackId::Sutysisku, PackId::Llg, PackId::LaLojban]
    );
    let profiles = CollectionProfiles::read(&config).unwrap();
    assert_eq!(profiles.profile(None).unwrap().disabled, report.disabled);

    // Enabling one of the original disabled members removes only that ID;
    // the remaining old order is retained.
    let enabled = CollectionProfiles::set_enabled(
        &config,
        CollectionProfileEnablementRequest {
            profile: Some("research".into()),
            selection: CollectionPolicySelection::Packs(vec![PackId::Sutysisku]),
            enabled: true,
        },
    )
    .unwrap();
    assert_eq!(enabled.affected_pack_ids, vec![PackId::Sutysisku]);
    assert_eq!(enabled.disabled, vec![PackId::Llg, PackId::LaLojban]);

    let stable = fs::read(&config).unwrap();
    let revision = enabled.revision;
    let sha = enabled.sha256;
    let unchanged = CollectionProfiles::set_enabled(
        &config,
        CollectionProfileEnablementRequest {
            profile: Some("research".into()),
            selection: CollectionPolicySelection::Packs(vec![PackId::Sutysisku]),
            enabled: true,
        },
    )
    .unwrap();
    assert_eq!(unchanged.outcome, "unchanged");
    assert!(unchanged.affected_pack_ids.is_empty());
    assert_eq!(unchanged.revision, revision);
    assert_eq!(unchanged.sha256, sha);
    assert_eq!(fs::read(&config).unwrap(), stable);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn test_default_policy_set_clear_and_same_value_are_explicit_atomic_noops() {
    let root = fresh_root("default-policy");
    let catalogue = root.join("catalogue");
    for pack in [PackId::Llg, PackId::Sutysisku, PackId::LaLojban] {
        register(&catalogue, pack);
    }
    let config = root.join("profiles.toml");
    configured(&config, &catalogue);

    let set = CollectionProfiles::set_default(&config, Some("research")).unwrap();
    assert_eq!(set.operation, "set_default");
    assert_eq!(set.outcome, "unchanged");
    assert!(set.affected_pack_ids.is_empty());
    let bytes = fs::read(&config).unwrap();
    let unchanged = CollectionProfiles::set_default(&config, Some("research")).unwrap();
    assert_eq!(unchanged.outcome, "unchanged");
    assert_eq!(unchanged.revision, set.revision);
    assert_eq!(unchanged.sha256, set.sha256);
    assert_eq!(fs::read(&config).unwrap(), bytes);

    let clear = CollectionProfiles::set_default(&config, None).unwrap();
    assert_eq!(clear.operation, "clear_default");
    assert_eq!(clear.outcome, "committed");
    assert_eq!(clear.default_profile, None);
    assert!(clear.affected_pack_ids.is_empty());
    assert_eq!(
        CollectionProfiles::read(&config)
            .unwrap()
            .report()
            .default_profile,
        None
    );
    let restored = CollectionProfiles::set_default(&config, Some("research")).unwrap();
    assert_eq!(restored.outcome, "committed");
    assert_eq!(restored.default_profile.as_deref(), Some("research"));
    fs::remove_dir_all(root).unwrap();
}
