//! Partial profile-policy operations using the shared locked config writer.

use super::*;
use std::fs::OpenOptions;

#[derive(Debug, Clone, Copy)]
pub(super) enum DisabledPolicy {
    Explicit,
    PreserveLatest,
}

/// Resolves a profile's disabled set after the caller has acquired the writer lock.
pub(super) fn resolve_disabled(
    policy: DisabledPolicy,
    explicit: &[PackId],
    prior_disabled: Option<&[String]>,
    path: &Path,
) -> Result<Vec<PackId>, CollectionProfileError> {
    match policy {
        DisabledPolicy::Explicit => Ok(explicit.to_vec()),
        DisabledPolicy::PreserveLatest => prior_disabled
            .map(|values| parse_ids(path, values, "disabled"))
            .transpose()
            .map(Option::unwrap_or_default),
    }
}

pub(super) fn set_enabled(
    supplied_path: impl AsRef<Path>,
    request: CollectionProfileEnablementRequest,
) -> Result<CollectionProfilePolicyReport, CollectionProfileError> {
    set_enabled_with_hook(supplied_path, request, || {})
}

fn set_enabled_with_hook<F>(
    supplied_path: impl AsRef<Path>,
    request: CollectionProfileEnablementRequest,
    before_lock: F,
) -> Result<CollectionProfilePolicyReport, CollectionProfileError>
where
    F: FnOnce(),
{
    let path = absolute(supplied_path.as_ref())?;
    if let Some(name) = request.profile.as_deref() {
        validate_name(name, &path)?;
    }
    if let CollectionPolicySelection::Packs(ids) = &request.selection {
        if ids.is_empty() {
            return Err(invalid(
                &path,
                "explicit enablement selection cannot be empty",
            ));
        }
        validate_ids(ids, "selection", &path)?;
    }
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    before_lock();
    let lock = acquire_lock(&path)?;
    let prior = read_required_config(&path)?;
    let profile_name = request
        .profile
        .clone()
        .or_else(|| prior.wire.default_profile.clone())
        .ok_or_else(|| CollectionProfileError::NoDefaultProfile { path: path.clone() })?;
    let profile = prior.profiles.get(&profile_name).ok_or_else(|| {
        CollectionProfileError::UnknownProfile {
            path: path.clone(),
            name: profile_name.clone(),
        }
    })?;
    let selected = match &request.selection {
        CollectionPolicySelection::All => profile.packs.clone(),
        CollectionPolicySelection::Packs(ids) => {
            for pack in ids {
                if !profile.packs.contains(pack) {
                    return Err(CollectionProfileError::NotProfileMember {
                        path: path.clone(),
                        profile: profile_name,
                        pack: *pack,
                    });
                }
            }
            ids.clone()
        }
    };
    let (catalogue_revision, availability) = validate_catalogue_membership(
        &path,
        &profile.root,
        &profile.packs,
        &profile.disabled,
        &selected,
    )?;
    let affected = profile
        .packs
        .iter()
        .copied()
        .filter(|pack| {
            selected.contains(pack) && (profile.disabled.contains(pack) == request.enabled)
        })
        .collect::<Vec<_>>();
    let mut disabled = profile.disabled.clone();
    if request.enabled {
        disabled.retain(|pack| !affected.contains(pack));
    } else {
        disabled.extend(affected.iter().copied());
    }
    let operation = if request.enabled { "enable" } else { "disable" };
    let mut wire = prior.wire.clone();
    if !affected.is_empty() {
        let profile_wire = wire
            .profiles
            .get_mut(&profile_name)
            .expect("profile parsed from the same configuration");
        profile_wire.disabled = disabled
            .iter()
            .map(|pack| pack.as_str().to_owned())
            .collect();
    }
    let changed = wire != prior.wire;
    if !changed {
        let report = policy_report(PolicyReportData {
            operation,
            outcome: "unchanged",
            path: &path,
            profile: Some(&profile_name),
            root: Some(&profile.root),
            packs: &profile.packs,
            disabled: &disabled,
            affected: &[],
            default_profile: prior.wire.default_profile.as_deref(),
            prior_revision: prior.wire.revision,
            revision: prior.wire.revision,
            prior_sha256: &prior.sha256,
            sha256: &prior.sha256,
            catalogue_revision: Some(catalogue_revision),
            availability: Some(&availability),
        });
        drop(lock);
        return Ok(report);
    }
    let publication = publish_config_wire(&path, parent, Some(&prior), &mut wire, |_, _| Ok(()))?;
    let report = policy_report(PolicyReportData {
        operation,
        outcome: "committed",
        path: &path,
        profile: Some(&profile_name),
        root: Some(&profile.root),
        packs: &profile.packs,
        disabled: &disabled,
        affected: &affected,
        default_profile: wire.default_profile.as_deref(),
        prior_revision: publication.prior_revision.unwrap_or_default(),
        revision: publication.revision,
        prior_sha256: publication.prior_sha256.as_deref().unwrap_or_default(),
        sha256: &publication.sha256,
        catalogue_revision: Some(catalogue_revision),
        availability: Some(&availability),
    });
    drop(lock);
    Ok(report)
}

pub(super) fn set_default(
    supplied_path: impl AsRef<Path>,
    requested_profile: Option<&str>,
) -> Result<CollectionProfilePolicyReport, CollectionProfileError> {
    set_default_with_hook(supplied_path, requested_profile, || {})
}

fn set_default_with_hook<F>(
    supplied_path: impl AsRef<Path>,
    requested_profile: Option<&str>,
    before_lock: F,
) -> Result<CollectionProfilePolicyReport, CollectionProfileError>
where
    F: FnOnce(),
{
    let path = absolute(supplied_path.as_ref())?;
    if let Some(name) = requested_profile {
        validate_name(name, &path)?;
    }
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    before_lock();
    let lock = acquire_lock(&path)?;
    let prior = read_required_config(&path)?;
    let target =
        if let Some(name) = requested_profile {
            Some(prior.profiles.get(name).ok_or_else(|| {
                CollectionProfileError::UnknownProfile {
                    path: path.clone(),
                    name: name.to_owned(),
                }
            })?)
        } else {
            None
        };
    let observation = target
        .map(|profile| {
            validate_catalogue_membership(
                &path,
                &profile.root,
                &profile.packs,
                &profile.disabled,
                &profile.packs,
            )
        })
        .transpose()?;
    let mut wire = prior.wire.clone();
    wire.default_profile = requested_profile.map(str::to_owned);
    let operation = if requested_profile.is_some() {
        "set_default"
    } else {
        "clear_default"
    };
    if wire == prior.wire {
        let profile = target;
        let report = policy_report(PolicyReportData {
            operation,
            outcome: "unchanged",
            path: &path,
            profile: requested_profile,
            root: profile.map(|value| value.root.as_path()),
            packs: profile.map_or(&[], |value| value.packs.as_slice()),
            disabled: profile.map_or(&[], |value| value.disabled.as_slice()),
            affected: &[],
            default_profile: wire.default_profile.as_deref(),
            prior_revision: prior.wire.revision,
            revision: prior.wire.revision,
            prior_sha256: &prior.sha256,
            sha256: &prior.sha256,
            catalogue_revision: observation.as_ref().map(|value| value.0),
            availability: observation.as_ref().map(|value| value.1.as_str()),
        });
        drop(lock);
        return Ok(report);
    }
    let publication = publish_config_wire(&path, parent, Some(&prior), &mut wire, |_, _| Ok(()))?;
    let profile = target;
    let report = policy_report(PolicyReportData {
        operation,
        outcome: "committed",
        path: &path,
        profile: requested_profile,
        root: profile.map(|value| value.root.as_path()),
        packs: profile.map_or(&[], |value| value.packs.as_slice()),
        disabled: profile.map_or(&[], |value| value.disabled.as_slice()),
        affected: &[],
        default_profile: wire.default_profile.as_deref(),
        prior_revision: publication.prior_revision.unwrap_or_default(),
        revision: publication.revision,
        prior_sha256: publication.prior_sha256.as_deref().unwrap_or_default(),
        sha256: &publication.sha256,
        catalogue_revision: observation.as_ref().map(|value| value.0),
        availability: observation.as_ref().map(|value| value.1.as_str()),
    });
    drop(lock);
    Ok(report)
}

fn validate_catalogue_membership(
    config_path: &Path,
    root: &Path,
    packs: &[PackId],
    disabled: &[PackId],
    requested: &[PackId],
) -> Result<(u64, String), CollectionProfileError> {
    let catalogue =
        CollectionCatalogue::open(root).map_err(|source| CollectionProfileError::Catalogue {
            source: Box::new(source),
        })?;
    let revision =
        catalogue
            .local_revision()
            .map_err(|source| CollectionProfileError::Catalogue {
                source: Box::new(source),
            })?;
    let report = catalogue
        .list()
        .map_err(|source| CollectionProfileError::Catalogue {
            source: Box::new(source),
        })?;
    let registered = report
        .packs
        .iter()
        .map(|item| item.pack_id.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    for pack in packs.iter().chain(disabled).chain(requested) {
        if !registered.contains(pack.as_str()) {
            return Err(CollectionProfileError::Invalid {
                path: config_path.to_path_buf(),
                detail: format!("pack {pack} is not registered in the configured catalogue"),
            });
        }
    }
    Ok((
        revision,
        if report.generation.is_some() {
            "generation_observed".into()
        } else {
            "registered_only".into()
        },
    ))
}

struct PolicyReportData<'a> {
    operation: &'a str,
    outcome: &'a str,
    path: &'a Path,
    profile: Option<&'a str>,
    root: Option<&'a Path>,
    packs: &'a [PackId],
    disabled: &'a [PackId],
    affected: &'a [PackId],
    default_profile: Option<&'a str>,
    prior_revision: u64,
    revision: u64,
    prior_sha256: &'a str,
    sha256: &'a str,
    catalogue_revision: Option<u64>,
    availability: Option<&'a str>,
}

fn policy_report(data: PolicyReportData<'_>) -> CollectionProfilePolicyReport {
    CollectionProfilePolicyReport {
        version: 1,
        ok: true,
        operation: data.operation.into(),
        outcome: data.outcome.into(),
        path: native_path_report(data.path),
        profile: data.profile.map(str::to_owned),
        root: data.root.map(native_path_report),
        packs: data.packs.to_vec(),
        disabled: data.disabled.to_vec(),
        affected_pack_ids: data.affected.to_vec(),
        default_profile: data.default_profile.map(str::to_owned),
        prior_revision: data.prior_revision,
        revision: data.revision,
        prior_sha256: data.prior_sha256.into(),
        sha256: data.sha256.into(),
        observed_catalogue_revision: data.catalogue_revision,
        observed_availability: data.availability.map(str::to_owned),
    }
}

fn read_required_config(path: &Path) -> Result<CollectionProfiles, CollectionProfileError> {
    match read_optional_config(path)? {
        Some(config) => Ok(config),
        None => Err(CollectionProfileError::MissingFile {
            path: path.to_path_buf(),
            source: io::Error::new(io::ErrorKind::NotFound, "configuration file is absent"),
        }),
    }
}

fn acquire_lock(path: &Path) -> Result<OwnedSibling, CollectionProfileError> {
    let lock_path = sibling(path, ".lock");
    let lock_token = format!("vlajmaji collection profile writer {}\n", unique_nonce());
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(&lock_path).map_err(|source| {
        if source.kind() == io::ErrorKind::AlreadyExists {
            CollectionProfileError::Busy(lock_path.clone())
        } else {
            CollectionProfileError::Io {
                path: lock_path.clone(),
                source,
            }
        }
    })?;
    let lock = OwnedSibling::new(lock_path.clone(), &file).map_err(|source| {
        CollectionProfileError::Io {
            path: lock_path.clone(),
            source,
        }
    })?;
    file.write_all(lock_token.as_bytes())
        .and_then(|_| file.sync_all())
        .map_err(|source| CollectionProfileError::Io {
            path: lock_path,
            source,
        })?;
    drop(file);
    Ok(lock)
}

#[cfg(test)]
mod tests {
    use std::{
        borrow::Cow,
        fs,
        path::{Path, PathBuf},
        process::Command,
        sync::atomic::{AtomicU64, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    use vlajmaji_lexicon::{Dictionary, PackPayloadKind, seal_release};

    use super::*;
    use crate::{
        CollectionInput, CollectionProfileWriteRequest, CollectionRequest, CollectionSelection,
    };

    static SCRATCH_SEQUENCE: AtomicU64 = AtomicU64::new(0);

    fn create_root_with(mut candidate: impl FnMut(usize) -> PathBuf) -> PathBuf {
        const MAX_ATTEMPTS: usize = 256;

        for attempt in 0..MAX_ATTEMPTS {
            let root = candidate(attempt);
            match fs::create_dir(&root) {
                Ok(()) => return root,
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => panic!(
                    "could not exclusively create test scratch directory {}: {error}",
                    root.display()
                ),
            }
        }
        panic!("could not allocate a test scratch directory after {MAX_ATTEMPTS} candidates");
    }

    fn fresh_root() -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let sequence = SCRATCH_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        create_root_with(|attempt| {
            std::env::temp_dir().join(format!(
                "vlajmaji-s04-policy-lock-{}-{stamp}-{sequence}-{attempt}",
                std::process::id()
            ))
        })
    }

    #[test]
    fn test_create_root_with_occupied_candidates_preserves_existing_contents() {
        let root = fresh_root();
        let occupied_directory = root.join("occupied-directory");
        fs::create_dir(&occupied_directory).unwrap();
        let sentinel = occupied_directory.join("sentinel");
        fs::write(&sentinel, b"preserve existing directory").unwrap();
        let occupied_file = root.join("occupied-file");
        fs::write(&occupied_file, b"preserve existing file").unwrap();
        let fresh = root.join("fresh");
        let candidates = [occupied_directory, occupied_file.clone(), fresh.clone()];

        assert_eq!(
            create_root_with(|attempt| candidates[attempt].clone()),
            fresh
        );
        assert!(fresh.is_dir());
        assert_eq!(fs::read(&sentinel).unwrap(), b"preserve existing directory");
        assert_eq!(fs::read(&occupied_file).unwrap(), b"preserve existing file");
        fs::remove_dir_all(root).unwrap();
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

    fn configure(
        path: &Path,
        name: &str,
        root: &Path,
        packs: Vec<PackId>,
        disabled: Vec<PackId>,
        make_default: bool,
    ) {
        CollectionProfiles::configure(
            path,
            CollectionProfileWriteRequest {
                name: name.into(),
                root: root.into(),
                packs,
                disabled,
                make_default,
            },
        )
        .unwrap();
    }

    #[test]
    fn test_enablement_reads_latest_default_root_membership_and_disabled_under_lock() {
        let scratch = fresh_root();
        let root_a = scratch.join("catalogue-a");
        let root_b = scratch.join("catalogue-b");
        for pack in [PackId::Llg, PackId::LaLojban, PackId::Sutysisku] {
            register(&root_a, pack);
            register(&root_b, pack);
        }
        let config = scratch.join("profiles.toml");
        configure(
            &config,
            "research",
            &root_a,
            vec![PackId::Llg],
            vec![],
            true,
        );
        configure(&config, "other", &root_a, vec![PackId::Llg], vec![], false);

        let report = set_enabled_with_hook(
            &config,
            CollectionProfileEnablementRequest {
                profile: None,
                selection: CollectionPolicySelection::Packs(vec![PackId::Llg]),
                enabled: false,
            },
            || {
                // A cooperating writer commits a complete newer profile before
                // this operation acquires its lock.
                configure(
                    &config,
                    "other",
                    &root_b,
                    vec![PackId::LaLojban, PackId::Llg, PackId::Sutysisku],
                    vec![PackId::LaLojban],
                    true,
                );
            },
        )
        .unwrap();

        assert_eq!(report.profile.as_deref(), Some("other"));
        assert_eq!(report.root, Some(native_path_report(&root_b)));
        assert_eq!(
            report.packs,
            vec![PackId::LaLojban, PackId::Llg, PackId::Sutysisku]
        );
        assert_eq!(report.disabled, vec![PackId::LaLojban, PackId::Llg]);
        assert_eq!(report.affected_pack_ids, vec![PackId::Llg]);
        assert_eq!(report.default_profile.as_deref(), Some("other"));
        let final_profiles = CollectionProfiles::read(&config).unwrap();
        let current = final_profiles.profile(None).unwrap();
        assert_eq!(current.root, root_b);
        assert_eq!(current.disabled, report.disabled);
        assert_eq!(
            final_profiles.profile(Some("research")).unwrap().packs,
            vec![PackId::Llg]
        );
        fs::remove_dir_all(scratch).unwrap();
    }

    #[test]
    fn test_set_default_resolves_profile_added_before_lock() {
        let scratch = fresh_root();
        let catalogue = scratch.join("catalogue");
        register(&catalogue, PackId::Llg);
        let config = scratch.join("profiles.toml");
        configure(
            &config,
            "research",
            &catalogue,
            vec![PackId::Llg],
            vec![],
            true,
        );
        let report = set_default_with_hook(&config, Some("new-profile"), || {
            configure(
                &config,
                "new-profile",
                &catalogue,
                vec![PackId::Llg],
                vec![],
                false,
            );
        })
        .unwrap();
        assert_eq!(report.outcome, "committed");
        assert_eq!(report.default_profile.as_deref(), Some("new-profile"));
        assert_eq!(
            CollectionProfiles::read(&config)
                .unwrap()
                .report()
                .default_profile
                .as_deref(),
            Some("new-profile")
        );
        fs::remove_dir_all(scratch).unwrap();
    }

    #[test]
    fn test_config_writer_child_exit_keeps_visible_snapshot_and_requires_alt_file() {
        const CHILD_PHASE: &str = "VLAJMAJI_TEST_PROFILE_EXIT_PHASE";
        const CHILD_CONFIG: &str = "VLAJMAJI_TEST_PROFILE_EXIT_CONFIG";
        if let Ok(phase) = std::env::var(CHILD_PHASE) {
            let config = PathBuf::from(std::env::var_os(CHILD_CONFIG).unwrap());
            let before = CollectionProfiles::read(&config).unwrap();
            let profile = before.profile(Some("research")).unwrap();
            let mut request = CollectionProfileWriteRequest {
                name: "research".into(),
                root: profile.root,
                packs: profile.packs,
                disabled: vec![PackId::Llg],
                make_default: true,
            };
            request.disabled.sort();
            let exit_phase = match phase.as_str() {
                "stage" => WriterPhase::StageCreated,
                "replace" => WriterPhase::AfterReplace,
                other => panic!("unknown test child phase {other}"),
            };
            let result = CollectionProfiles::configure_with_hooks(
                &config,
                request,
                DisabledPolicy::Explicit,
                || {},
                |observed, _| {
                    if observed == exit_phase {
                        // This deliberately skips Rust destructors, unlike a
                        // returned IO error handled by the normal rollback path.
                        std::process::exit(73);
                    }
                    Ok(())
                },
            );
            panic!("child operation unexpectedly returned: {result:?}");
        }

        for (phase, expected_revision, expected_disabled) in
            [("stage", 1, false), ("replace", 2, true)]
        {
            let scratch = fresh_root();
            let catalogue = scratch.join("catalogue");
            register(&catalogue, PackId::Llg);
            let config = scratch.join("profiles.toml");
            configure(
                &config,
                "research",
                &catalogue,
                vec![PackId::Llg],
                vec![],
                true,
            );
            let initial_bytes = fs::read(&config).unwrap();
            let lock = config.with_file_name("profiles.toml.lock");

            let status = Command::new(std::env::current_exe().unwrap())
                .arg("--exact")
                .arg("collection_profiles::policy::tests::test_config_writer_child_exit_keeps_visible_snapshot_and_requires_alt_file")
                .arg("--nocapture")
                .env(CHILD_PHASE, phase)
                .env(CHILD_CONFIG, &config)
                .status()
                .unwrap();
            assert_eq!(status.code(), Some(73));
            assert!(lock.exists());
            let visible = CollectionProfiles::read(&config).unwrap();
            assert_eq!(visible.revision(), expected_revision);
            assert_eq!(
                visible
                    .profile(Some("research"))
                    .unwrap()
                    .disabled
                    .contains(&PackId::Llg),
                expected_disabled
            );
            let visible_bytes = fs::read(&config).unwrap();
            if phase == "stage" {
                assert_eq!(visible_bytes, initial_bytes);
                assert!(fs::read_dir(&scratch).unwrap().any(|entry| {
                    entry
                        .unwrap()
                        .file_name()
                        .to_string_lossy()
                        .contains(".stage-")
                }));
            }
            // The retained sibling lock is intentionally not stolen. Continue
            // from the exact last visible snapshot in a create-new file beside
            // it, then prove writes against the original remain Busy.
            let alt = scratch.join("profiles-alt.toml");
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&alt)
                .unwrap();
            use std::io::Write;
            file.write_all(&visible_bytes).unwrap();
            file.sync_all().unwrap();
            drop(file);
            fs::File::open(&scratch).unwrap().sync_all().unwrap();
            let validated_alt = CollectionProfiles::read(&alt).unwrap();
            assert_eq!(validated_alt.revision(), expected_revision);
            let continued = set_enabled(
                &alt,
                CollectionProfileEnablementRequest {
                    profile: Some("research".into()),
                    selection: CollectionPolicySelection::Packs(vec![PackId::Llg]),
                    enabled: expected_disabled,
                },
            )
            .unwrap();
            assert_eq!(continued.outcome, "committed");
            let alt_after = CollectionProfiles::read(&alt).unwrap();
            assert_eq!(alt_after.revision(), expected_revision + 1);
            assert_eq!(
                alt_after
                    .profile(Some("research"))
                    .unwrap()
                    .disabled
                    .contains(&PackId::Llg),
                !expected_disabled
            );
            assert!(matches!(
                set_enabled(
                    &config,
                    CollectionProfileEnablementRequest {
                        profile: Some("research".into()),
                        selection: CollectionPolicySelection::Packs(vec![PackId::Llg]),
                        enabled: true,
                    },
                ),
                Err(CollectionProfileError::Busy(_))
            ));
            assert_eq!(fs::read(&config).unwrap(), visible_bytes);
            assert!(lock.exists());
            fs::remove_dir_all(scratch).unwrap();
        }
    }
}
