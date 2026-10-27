//! Explicit v1 migration and current-generation update/rebuild operations.

use super::*;
use rusqlite::{OptionalExtension, params};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Clone, Copy)]
pub(super) enum InstallMode {
    Install,
    Update(u64),
    Rebuild(u64),
}

#[cfg(test)]
#[derive(Clone, Copy, PartialEq, Eq)]
enum LifecycleFailurePoint {
    MigrateBeforeDdl,
    MigrateAfterDdl,
    MigrateExitAfterDdl,
    UpdateBeforeCommit,
    RebuildBeforeCommit,
    RebuildExitBeforeCommit,
}

#[cfg(test)]
thread_local! {
    static TEST_LIFECYCLE_FAILURE_POINT: std::cell::Cell<Option<LifecycleFailurePoint>> = const { std::cell::Cell::new(None) };
}

#[cfg(test)]
fn maybe_inject_lifecycle_failure(
    point: LifecycleFailurePoint,
    root: &Path,
) -> Result<(), CollectionError> {
    let selected_point = TEST_LIFECYCLE_FAILURE_POINT.with(|selected| {
        let current = selected.get();
        let process_exit = point == LifecycleFailurePoint::RebuildBeforeCommit
            && current == Some(LifecycleFailurePoint::RebuildExitBeforeCommit);
        let migration_exit = point == LifecycleFailurePoint::MigrateAfterDdl
            && current == Some(LifecycleFailurePoint::MigrateExitAfterDdl);
        if current == Some(point) || process_exit || migration_exit {
            selected.set(None);
            Some(process_exit || migration_exit)
        } else {
            None
        }
    });
    if let Some(process_exit) = selected_point {
        if process_exit {
            std::process::exit(73);
        }
        return Err(CollectionError::ArtifactIoSource {
            path: Some(root.join("injected-lifecycle-phase")),
            source: Box::new(std::io::Error::other("injected returned phase failure")),
        });
    }
    Ok(())
}

fn foreign_key_layout_matches(
    observed: &[(i64, i64, String, String, String)],
    expected: &[(i64, i64, &str, &str, &str)],
) -> bool {
    observed
        .iter()
        .map(|(id, sequence, target, from, to)| {
            (*id, *sequence, target.as_str(), from.as_str(), to.as_str())
        })
        .eq(expected.iter().copied())
}

pub(super) fn migrate_catalogue(
    root: &Path,
    expected_local_revision: u64,
) -> Result<CollectionCatalogueMigrationReport, CollectionError> {
    let root = absolute_path(root)?;
    // Explicit migration operates only on an existing catalogue; it never creates
    // a fresh v2 root as a side effect of being asked to migrate.
    let connection = crate::collections_schema::open_writable(&root)?;
    let mut transaction = TransactionGuard::begin(&connection, &root)?;
    let mut error_context = MutationErrorContext::default();
    let result = (|| {
        let prior_schema = crate::collections_schema::schema_version(
            &connection,
            &crate::collections_schema::database_path(&root),
        )?;
        let tables = crate::collections_schema::tables_for(
            &connection,
            &crate::collections_schema::database_path(&root),
        )?;
        let prior_revision = crate::collections_schema::local_revision(&connection, &root)?;
        error_context.prior_local_revision = Some(prior_revision);
        if prior_revision != expected_local_revision {
            return Err(CollectionError::RevisionConflict {
                expected: expected_local_revision,
                observed: prior_revision,
            });
        }
        if prior_schema == crate::collections_schema::VERSION_2 {
            let generation = read_generation(&connection, &root)?;
            return Ok(CollectionCatalogueMigrationReport {
                version: 1,
                ok: true,
                operation: "migrate".into(),
                outcome: "idempotent".into(),
                root_path: native_path_report(&root),
                prior_schema_version: prior_schema,
                schema_version: prior_schema,
                prior_local_revision: prior_revision,
                local_revision: prior_revision,
                active_generation: generation.map(|generation| generation.generation_id),
            });
        }
        validate_v1_catalogue(&connection, &root, tables)?;
        let next_revision = prior_revision.checked_add(1).ok_or_else(|| {
            CollectionError::UnsupportedVersion("catalogue local_revision overflowed u64".into())
        })?;
        let _ = i64::try_from(next_revision).map_err(|_| {
            CollectionError::UnsupportedVersion(
                "next local_revision exceeds SQLite INTEGER range".into(),
            )
        })?;
        let current = read_generation(&connection, &root)?;
        let active_generation = current
            .as_ref()
            .map(|generation| generation.generation_id.clone());
        error_context.observed_generation = active_generation.clone();
        let attempt = attempt_id(&root, "migrate", next_revision);
        error_context.attempt_id = Some(attempt.clone());
        #[cfg(test)]
        maybe_inject_lifecycle_failure(LifecycleFailurePoint::MigrateBeforeDdl, &root)?;
        for (old, new) in [
            ("catalogue_meta", "catalogue_meta_v2"),
            ("registrations", "registrations_v2"),
            ("registration_locations", "registration_locations_v2"),
            ("generations", "generations_v2"),
            ("installed_heads", "installed_heads_v2"),
            ("attempts", "attempts_v2"),
        ] {
            let exists: bool = connection
                .query_row(
                    "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE name=?1)",
                    [new],
                    |row| row.get(0),
                )
                .map_err(|error| db_error(&root, error))?;
            if exists {
                return Err(CollectionError::UnsupportedVersion(format!(
                    "migration destination object {new} already exists"
                )));
            }
            connection
                .execute_batch(&format!("ALTER TABLE {old} RENAME TO {new}"))
                .map_err(|error| db_error(&root, error))?;
        }
        connection
            .execute_batch("CREATE VIEW catalogue_meta AS SELECT key,value FROM catalogue_meta_v2")
            .map_err(|error| db_error(&root, error))?;
        // The compatibility view still reports schema_version=1 until this
        // transaction updates the physical metadata table.
        let v2 = crate::collections_schema::tables_v2();
        connection
            .execute(
                &format!(
                    "UPDATE {} SET value='2' WHERE key='schema_version'",
                    v2.meta
                ),
                [],
            )
            .map_err(|error| db_error(&root, error))?;
        connection
            .execute(
                &format!(
                    "INSERT INTO {}(key,value) VALUES('active_generation_id',?1)",
                    v2.meta
                ),
                [active_generation.as_deref().unwrap_or("")],
            )
            .map_err(|error| db_error(&root, error))?;
        crate::collections_schema::update_revision(&connection, v2, next_revision)
            .map_err(|error| db_error(&root, error))?;
        let prior_sql = i64::try_from(prior_revision).map_err(|_| {
            CollectionError::UnsupportedVersion(
                "catalogue local_revision exceeds SQLite INTEGER range".into(),
            )
        })?;
        connection.execute(
            &format!("INSERT INTO {}(attempt_id,operation,status,prior_revision,detail) VALUES(?1,'migrate','committed',?2,'catalogue schema migrated')", v2.attempts),
            params![attempt, prior_sql],
        ).map_err(|error| db_error(&root, error))?;
        let mut foreign_keys = connection
            .prepare("PRAGMA foreign_key_check")
            .map_err(|error| db_error(&root, error))?;
        if foreign_keys
            .query([])
            .map_err(|error| db_error(&root, error))?
            .next()
            .map_err(|error| db_error(&root, error))?
            .is_some()
        {
            return Err(CollectionError::UnsupportedVersion(
                "catalogue foreign_key_check failed after migration".into(),
            ));
        }
        drop(foreign_keys);
        #[cfg(test)]
        maybe_inject_lifecycle_failure(LifecycleFailurePoint::MigrateAfterDdl, &root)?;
        Ok(CollectionCatalogueMigrationReport {
            version: 1,
            ok: true,
            operation: "migrate".into(),
            outcome: "migrated".into(),
            root_path: native_path_report(&root),
            prior_schema_version: prior_schema,
            schema_version: crate::collections_schema::VERSION_2,
            prior_local_revision: prior_revision,
            local_revision: next_revision,
            active_generation,
        })
    })();
    match result {
        Ok(report) if report.outcome == "idempotent" => match transaction.rollback() {
            Ok(()) => Ok(report),
            Err(error) => Err(mutation_context(error, error_context)),
        },
        Ok(report) => match transaction.commit() {
            Ok(()) => Ok(report),
            Err(error) => {
                if matches!(
                    error,
                    CollectionError::Indeterminate(_)
                        | CollectionError::IndeterminateSources { .. }
                ) {
                    error_context.observed_generation = None;
                }
                Err(mutation_context(error, error_context))
            }
        },
        Err(error) => match transaction.rollback() {
            Ok(()) => Err(mutation_context(error, error_context)),
            Err(rollback) => Err(mutation_context(
                CollectionError::IndeterminateSources {
                    detail: "catalogue migration failed and rollback could not be confirmed".into(),
                    source: Some(Box::new(error)),
                    rollback_source: Some(Box::new(rollback)),
                },
                MutationErrorContext {
                    observed_generation: None,
                    ..error_context
                },
            )),
        },
    }
}

fn validate_v1_catalogue(
    connection: &Connection,
    root: &Path,
    tables: crate::collections_schema::CatalogueTables,
) -> Result<(), CollectionError> {
    use std::collections::{BTreeMap, BTreeSet};
    let actual = {
        let mut statement = connection
            .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name")
            .map_err(|error| db_error(root, error))?;
        statement
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|error| db_error(root, error))?
            .collect::<Result<BTreeSet<_>, _>>()
            .map_err(|error| db_error(root, error))?
    };
    let expected = [
        "catalogue_meta",
        "registrations",
        "registration_locations",
        "generations",
        "installed_heads",
        "attempts",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<BTreeSet<_>>();
    if actual != expected {
        return Err(CollectionError::UnsupportedVersion(
            "version-1 catalogue does not have the exact six-table layout".into(),
        ));
    }
    let expected_columns = [
        ("catalogue_meta", &["key", "value"][..]),
        (
            "registrations",
            &[
                "pack_id",
                "release_sha256",
                "payload_kind",
                "seal_json",
                "descriptor_json",
                "summary_json",
                "input_paths_json",
            ][..],
        ),
        (
            "registration_locations",
            &["pack_id", "release_sha256", "input_paths_json"][..],
        ),
        (
            "generations",
            &[
                "generation_id",
                "content_id",
                "canonical_sha256",
                "store_path",
                "members_json",
                "local_revision",
            ][..],
        ),
        (
            "installed_heads",
            &["pack_id", "release_sha256", "payload_kind", "generation_id"][..],
        ),
        (
            "attempts",
            &[
                "attempt_id",
                "operation",
                "status",
                "prior_revision",
                "detail",
            ][..],
        ),
    ];
    for (table, expected) in expected_columns {
        let mut statement = connection
            .prepare(&format!("PRAGMA table_info({table})"))
            .map_err(|error| db_error(root, error))?;
        let observed = statement
            .query_map([], |row| row.get::<_, String>(1))
            .map_err(|error| db_error(root, error))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| db_error(root, error))?;
        if observed.iter().map(String::as_str).collect::<Vec<_>>() != expected {
            return Err(CollectionError::UnsupportedVersion(format!(
                "version-1 table {table} has an incompatible column layout"
            )));
        }
    }
    for (table, expected) in [
        (
            "registration_locations",
            &[
                (0, 0, "registrations", "pack_id", "pack_id"),
                (0, 1, "registrations", "release_sha256", "release_sha256"),
            ][..],
        ),
        (
            "installed_heads",
            &[(0, 0, "generations", "generation_id", "generation_id")][..],
        ),
        ("catalogue_meta", &[][..]),
        ("registrations", &[][..]),
        ("generations", &[][..]),
        ("attempts", &[][..]),
    ] {
        let mut statement = connection
            .prepare(&format!("PRAGMA foreign_key_list({table})"))
            .map_err(|error| db_error(root, error))?;
        let observed = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                ))
            })
            .map_err(|error| db_error(root, error))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| db_error(root, error))?;
        if !foreign_key_layout_matches(&observed, expected) {
            return Err(CollectionError::UnsupportedVersion(format!(
                "version-1 table {table} has incompatible foreign-key targets"
            )));
        }
    }
    let mut foreign_keys = connection
        .prepare("PRAGMA foreign_key_check")
        .map_err(|error| db_error(root, error))?;
    if foreign_keys
        .query([])
        .map_err(|error| db_error(root, error))?
        .next()
        .map_err(|error| db_error(root, error))?
        .is_some()
    {
        return Err(CollectionError::UnsupportedVersion(
            "version-1 catalogue contains a foreign-key violation".into(),
        ));
    }
    for suffix in [
        "catalogue_meta_v2",
        "registrations_v2",
        "registration_locations_v2",
        "generations_v2",
        "installed_heads_v2",
        "attempts_v2",
    ] {
        let exists: bool = connection
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE name=?1)",
                [suffix],
                |row| row.get(0),
            )
            .map_err(|error| db_error(root, error))?;
        if exists {
            return Err(CollectionError::UnsupportedVersion(format!(
                "version-1 migration destination {suffix} already exists"
            )));
        }
    }
    let registrations = {
        let mut statement = connection
            .prepare(&format!("SELECT pack_id,release_sha256,payload_kind,seal_json,descriptor_json,summary_json FROM {}", tables.registrations))
            .map_err(|error| db_error(root, error))?;
        statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Vec<u8>>(3)?,
                    row.get::<_, Vec<u8>>(4)?,
                    row.get::<_, Vec<u8>>(5)?,
                ))
            })
            .map_err(|error| db_error(root, error))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| db_error(root, error))?
    };
    let mut registered_members = BTreeSet::new();
    for (pack, digest, kind, seal_bytes, descriptor_bytes, summary_bytes) in registrations {
        let seal = PackSeal::from_json_bytes(&seal_bytes).map_err(verification_error)?;
        let descriptor: CollectionReferenceMetadata = serde_json::from_slice(&descriptor_bytes)
            .map_err(|error| {
                CollectionError::UnsupportedVersion(format!(
                    "registration descriptor is malformed: {error}"
                ))
            })?;
        let _: StoredCollectionRegistrationSummary = serde_json::from_slice(&summary_bytes)
            .map_err(|error| {
                CollectionError::UnsupportedVersion(format!(
                    "registration summary is malformed: {error}"
                ))
            })?;
        if pack != descriptor.pack_id
            || digest != descriptor.release_sha256
            || kind != descriptor.payload_kind
            || kind != payload_kind_name(seal.payload_kind)
            || pack != seal.pack_id.as_str()
            || digest != seal.release_sha256
            || descriptor.dictionary_version != seal.dictionary_version
            || descriptor.registry_version != seal.registry_version
        {
            return Err(CollectionError::UnsupportedVersion(
                "registration identity is inconsistent".into(),
            ));
        }
        registered_members.insert((pack, digest, kind));
    }
    let active = read_generation(connection, root)?;
    let mut expected_heads = BTreeMap::new();
    if let Some(generation) = &active {
        for member in &generation.members {
            if !registered_members.contains(&(
                member.pack_id.clone(),
                member.release_sha256.clone(),
                member.payload_kind.clone(),
            )) {
                return Err(CollectionError::UnsupportedVersion(
                    "active generation member has no matching registration".into(),
                ));
            }
            let head = (
                member.release_sha256.clone(),
                member.payload_kind.clone(),
                generation.generation_id.clone(),
            );
            if expected_heads
                .insert(member.pack_id.clone(), head)
                .is_some()
            {
                return Err(CollectionError::UnsupportedVersion(
                    "active generation contains a duplicate pack member".into(),
                ));
            }
        }
    }
    let observed_heads = {
        let mut statement = connection
            .prepare(&format!(
                "SELECT pack_id,release_sha256,payload_kind,generation_id FROM {}",
                tables.installed_heads
            ))
            .map_err(|error| db_error(root, error))?;
        statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    (
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                    ),
                ))
            })
            .map_err(|error| db_error(root, error))?
            .collect::<Result<BTreeMap<_, _>, _>>()
            .map_err(|error| db_error(root, error))?
    };
    if expected_heads != observed_heads {
        return Err(CollectionError::UnsupportedVersion(
            "installed heads do not exactly match latest generation members".into(),
        ));
    }
    Ok(())
}

pub(super) fn mutate_install_mode(
    root: &Path,
    request: CollectionRequest<'_>,
    mode: InstallMode,
) -> Result<CollectionMutationReport, CollectionError> {
    let operation = match mode {
        InstallMode::Install => "install",
        InstallMode::Update(_) => "update",
        InstallMode::Rebuild(_) => "rebuild",
    };
    validate_selection(&request.inputs, &request.selection)?;
    let cwd = operation_cwd(root, &request)?;
    let root = absolute_path_with_cwd(root, cwd.as_deref())?;
    let input_ids = request.inputs.iter().map(input_pack_id).collect::<Vec<_>>();
    let selected_ids = selected_pack_ids(&request.inputs, &request.selection);
    let normalized = normalize_selected(request, cwd.as_deref())?;
    let mut error_context = MutationErrorContext {
        inputs: input_dispositions(&input_ids, &selected_ids, &normalized),
        ..MutationErrorContext::default()
    };
    let (connection, newly_created) = match mode {
        InstallMode::Install => writer_connection(&root),
        InstallMode::Update(_) | InstallMode::Rebuild(_) => {
            crate::collections_schema::open_writable(&root).map(|connection| (connection, false))
        }
    }
    .map_err(|error| mutation_context(error, error_context.clone()))?;
    if newly_created {
        create_private_dir(&root.join("generations"))
            .map_err(|error| mutation_context(error, error_context.clone()))?;
        crate::collections_schema::sync_dir(&root)
            .map_err(|error| mutation_context(error, error_context.clone()))?;
    }
    let mut transaction = TransactionGuard::begin(&connection, &root)
        .map_err(|error| mutation_context(error, error_context.clone()))?;
    let result = (|| -> Result<CollectionMutationReport, CollectionError> {
        let tables = crate::collections_schema::tables_for(&connection, &root)?;
        require_v2(&connection, &root, tables)?;
        let prior_revision = crate::collections_schema::local_revision(&connection, &root)?;
        error_context.prior_local_revision = Some(prior_revision);
        let expected_revision = match mode {
            InstallMode::Install => None,
            InstallMode::Update(expected) | InstallMode::Rebuild(expected) => Some(expected),
        };
        if let Some(expected) = expected_revision
            && expected != prior_revision
        {
            return Err(CollectionError::RevisionConflict {
                expected,
                observed: prior_revision,
            });
        }
        let revision = prior_revision.checked_add(1).ok_or_else(|| {
            CollectionError::UnsupportedVersion("catalogue local_revision overflowed u64".into())
        })?;
        let prior_sql = i64::try_from(prior_revision).map_err(|_| {
            CollectionError::UnsupportedVersion(
                "catalogue local_revision exceeds SQLite INTEGER range".into(),
            )
        })?;
        let revision_sql = i64::try_from(revision).map_err(|_| {
            CollectionError::UnsupportedVersion(
                "next local_revision exceeds SQLite INTEGER range".into(),
            )
        })?;
        let previous_generation = current_members(&connection, &root)?;
        error_context.observed_generation =
            read_generation(&connection, &root)?.map(|item| item.generation_id);
        if matches!(mode, InstallMode::Install) {
            for pack in &normalized {
                let active = previous_generation
                    .iter()
                    .any(|member| member.pack_id == pack.pack_id.as_str());
                if maintenance::installation_state(
                    &connection,
                    &root,
                    pack.pack_id.as_str(),
                    active,
                )? == CollectionInstallationState::Removed
                {
                    return Err(CollectionError::ReinstallRequired { pack: pack.pack_id });
                }
            }
        }
        let mut members = if matches!(mode, InstallMode::Rebuild(_)) {
            Vec::new()
        } else {
            previous_generation.clone()
        };
        let mut new_packs = std::collections::BTreeMap::new();
        let mut dispositions = Vec::new();
        for pack in normalized {
            let digest = release_sha256(&pack.release);
            if let Some(existing) = members
                .iter()
                .find(|member| member.pack_id == pack.pack_id.as_str())
            {
                if existing.release_sha256 != digest {
                    if matches!(mode, InstallMode::Update(_)) {
                        members.retain(|member| member.pack_id != pack.pack_id.as_str());
                        new_packs.insert(pack.pack_id.as_str().to_owned(), pack);
                        continue;
                    }
                    return Err(CollectionError::UpdateRequired(pack.pack_id));
                }
                if existing.payload_kind != payload_kind_name(pack.seal.payload_kind) {
                    return Err(CollectionError::IdentityConflict {
                        category: "payload kind".into(),
                        key: format!("{}:{digest}", pack.pack_id),
                    });
                }
                let existing_release = fs::read(root.join(&existing.release_path))
                    .map_err(|error| artifact_error(root.join(&existing.release_path), error))?;
                let existing_seal = fs::read(root.join(&existing.seal_path))
                    .map_err(|error| artifact_error(root.join(&existing.seal_path), error))?;
                let old_seal =
                    PackSeal::from_json_bytes(&existing_seal).map_err(verification_error)?;
                if existing_release != pack.release || old_seal != pack.seal {
                    return Err(CollectionError::IdentityConflict {
                        category: "installed release/seal".into(),
                        key: format!("{}:{digest}", pack.pack_id),
                    });
                }
                dispositions.push(disposition(&pack, "validated"));
            } else {
                if matches!(mode, InstallMode::Update(_))
                    && !previous_generation
                        .iter()
                        .any(|member| member.pack_id == pack.pack_id.as_str())
                {
                    return Err(CollectionError::NotInstalled {
                        pack: Some(pack.pack_id),
                    });
                }
                new_packs.insert(pack.pack_id.as_str().to_owned(), pack);
            }
        }
        if new_packs.is_empty() && !members.is_empty() && !matches!(mode, InstallMode::Rebuild(_)) {
            let generation = read_generation(&connection, &root)?;
            let store_path = generation
                .as_ref()
                .map(|item| item.store_path.clone())
                .ok_or_else(|| {
                    CollectionError::VerificationFailed("installed heads have no generation".into())
                })?;
            let mut dictionaries = Vec::new();
            for member in &members {
                let release_path = root.join(&member.release_path);
                let seal_path = root.join(&member.seal_path);
                let release = fs::read(&release_path)
                    .map_err(|error| artifact_error(release_path.clone(), error))?;
                let seal = fs::read(&seal_path)
                    .map_err(|error| artifact_error(seal_path.clone(), error))?;
                dictionaries.push(
                    load_sealed_release(parse_pack_id(&member.pack_id)?, &release, &seal)
                        .map_err(verification_error)?,
                );
            }
            let composed =
                vlajmaji_lexicon::compose_dictionaries(dictionaries).map_err(composition_error)?;
            let canonical = composed.to_json_bytes().map_err(verification_error)?;
            if generation
                .as_ref()
                .is_none_or(|item| release_sha256(&canonical) != item.canonical_sha256)
            {
                return Err(CollectionError::VerificationFailed(
                    "existing generation canonical digest changed".into(),
                ));
            }
            drop(canonical);
            drop(composed);
            let opened = crate::DictionaryStore::open(&store_path).map_err(verification_error)?;
            if opened.recorded_state_sha256()
                != generation
                    .as_ref()
                    .map_or("", |item| item.canonical_sha256.as_str())
            {
                return Err(CollectionError::VerificationFailed(
                    "existing store header digest differs from catalogue canonical digest".into(),
                ));
            }
            drop(opened);
            let verification = crate::DictionaryStoreVerification::verify(
                &store_path,
                crate::DictionaryStoreVerificationCheck::All,
            )
            .map_err(verification_error)?;
            if !verification.ok {
                return Err(CollectionError::VerificationFailed(
                    "existing generation failed All verification".into(),
                ));
            }
            return Ok(CollectionMutationReport {
                version: 1,
                ok: true,
                operation: operation.into(),
                outcome: "idempotent".into(),
                local_revision: prior_revision,
                policy_state: "unconfigured".into(),
                attempt_id: None,
                generation,
                inputs: error_context.inputs.clone(),
            });
        }

        let catalogue_id = crate::collections_schema::catalogue_identity(&connection, tables)
            .map_err(|error| db_error(&root, error))?;
        let (attempt, generation_id, stage) =
            allocate_stage(&root, revision, &catalogue_id, attempt_id)?;
        error_context.attempt_id = Some(attempt.clone());
        error_context.proposed_generation = Some(generation_id.clone());
        #[cfg(test)]
        if TEST_INTERRUPT_AFTER_STAGE_ALLOCATION.with(|interrupt| interrupt.replace(false)) {
            return Err(CollectionError::InvalidRequest(
                "injected interruption after stage allocation".into(),
            ));
        }
        let relative_stage = PathBuf::from("generations").join(&attempt);
        write_new_file(
            &stage.join("attempt.json"),
            &serde_json::to_vec(&StageMarker {
                version: 1,
                attempt_id: attempt.clone(),
                operation: operation.into(),
            })
            .map_err(|error| CollectionError::InvalidRequest(error.to_string()))?,
        )?;
        create_private_dir(&stage.join("inputs"))?;
        for (pack_id, pack) in &new_packs {
            let input_dir = stage.join("inputs").join(pack_id);
            create_private_dir(&input_dir)?;
            write_new_file(&input_dir.join("release.json"), &pack.release)?;
            write_new_file(&input_dir.join("seal.json"), &pack.seal_bytes)?;
            if let Some(report) = &pack.report {
                let bytes = serde_json::to_vec(report).map_err(|error| {
                    CollectionError::ArtifactIoSource {
                        path: Some(input_dir.join("report.json")),
                        source: Box::new(error),
                    }
                })?;
                write_new_file(&input_dir.join("report.json"), &bytes)?;
            }
            crate::collections_schema::sync_dir(&input_dir)?;
            let prefix = relative_stage.join("inputs").join(pack_id);
            members.push(InstalledMember {
                pack_id: pack_id.clone(),
                release_sha256: release_sha256(&pack.release),
                payload_kind: payload_kind_name(pack.seal.payload_kind).into(),
                release_path: prefix.join("release.json").to_string_lossy().into_owned(),
                seal_path: prefix.join("seal.json").to_string_lossy().into_owned(),
                report_path: pack
                    .report
                    .as_ref()
                    .map(|_| prefix.join("report.json").to_string_lossy().into_owned()),
            });
            let descriptor = descriptor_bytes(pack)?;
            let summary = registration_summary_bytes(&descriptor)?;
            let locations = serde_json::to_vec(
                &pack
                    .input_paths
                    .iter()
                    .map(|path| native_path_report(path))
                    .collect::<Vec<_>>(),
            )
            .map_err(|error| CollectionError::InvalidRequest(error.to_string()))?;
            let existing: Option<(String, Vec<u8>, Vec<u8>)> = connection.query_row(
            &format!("SELECT payload_kind,seal_json,descriptor_json FROM {} WHERE pack_id=?1 AND release_sha256=?2", tables.registrations),
            params![pack_id, release_sha256(&pack.release)],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        ).optional().map_err(|error| db_error(&root, error))?;
            match existing {
                Some((kind, seal, old_descriptor))
                    if kind == payload_kind_name(pack.seal.payload_kind)
                        && PackSeal::from_json_bytes(&seal).ok().as_ref() == Some(&pack.seal)
                        && old_descriptor == descriptor =>
                {
                    connection.execute(&format!("INSERT OR IGNORE INTO {}(pack_id,release_sha256,input_paths_json) VALUES(?1,?2,?3)", tables.registration_locations), params![pack_id, release_sha256(&pack.release), locations]).map_err(|error| db_error(&root, error))?;
                }
                Some(_) => {
                    return Err(CollectionError::IdentityConflict {
                        category: "registration".into(),
                        key: format!("{pack_id}:{}", release_sha256(&pack.release)),
                    });
                }
                None => crate::collections_schema::insert_registration(
                    &connection,
                    tables,
                    crate::collections_schema::RegistrationInsert {
                        pack_id,
                        release_sha256: &release_sha256(&pack.release),
                        payload_kind: payload_kind_name(pack.seal.payload_kind),
                        seal_json: &pack.seal_bytes,
                        descriptor_json: &descriptor,
                        summary_json: &summary,
                        input_paths_json: &locations,
                    },
                )
                .map_err(|error| db_error(&root, error))?,
            };
        }
        members.sort_by_key(|member| pack_order(&member.pack_id));
        let new_dispositions = new_packs
            .values()
            .map(|pack| disposition(pack, "installed"))
            .collect::<Vec<_>>();
        // Inputs have been sealed and copied into the owned stage. Release the
        // adapter's typed models before loading the complete composing set.
        drop(new_packs);
        let built = build_unified_generation(&root, &stage, &members)?;
        let content_id = built.content_id;
        let canonical_sha256 = built.canonical_sha256;
        let store_path = built.store_path;
        let members_bytes =
            serde_json::to_vec(&members).map_err(|error| CollectionError::ArtifactIoSource {
                path: None,
                source: Box::new(error),
            })?;
        connection.execute(
        &format!("INSERT INTO {}(generation_id,content_id,canonical_sha256,store_path,members_json,local_revision) VALUES(?1,?2,?3,?4,?5,?6)", tables.generations),
        params![generation_id, content_id, canonical_sha256, native_path_bytes(&store_path), members_bytes, revision_sql],
    ).map_err(|error| db_error(&root, error))?;
        connection
            .execute(&format!("DELETE FROM {}", tables.installed_heads), [])
            .map_err(|error| db_error(&root, error))?;
        for member in &members {
            connection.execute(&format!("INSERT INTO {}(pack_id,release_sha256,payload_kind,generation_id) VALUES(?1,?2,?3,?4)", tables.installed_heads), params![member.pack_id, member.release_sha256, member.payload_kind, generation_id])
            .map_err(|error| db_error(&root, error))?;
        }
        crate::collections_schema::update_revision(&connection, tables, revision)
            .map_err(|error| db_error(&root, error))?;
        crate::collections_schema::update_active_generation(&connection, tables, &generation_id)
            .map_err(|error| db_error(&root, error))?;
        connection.execute(&format!("INSERT INTO {}(attempt_id,operation,status,prior_revision,detail) VALUES(?1,?2,'committed',?3,'Unified store passed All verification')", tables.attempts), params![attempt, operation, prior_sql])
        .map_err(|error| db_error(&root, error))?;
        // Make all owned directory entries durable before the SQLite commit. The
        // catalogue commit itself publishes the already-complete stage.
        crate::collections_schema::sync_dir(&root.join("generations"))?;
        let generation = Some(CollectionGeneration {
            generation_id: generation_id.clone(),
            content_id: content_id.clone(),
            canonical_sha256: canonical_sha256.clone(),
            path: native_path_report(&store_path),
            store_path,
            members: members
                .iter()
                .map(|member| CollectionGenerationMember {
                    pack_id: member.pack_id.clone(),
                    release_sha256: member.release_sha256.clone(),
                    payload_kind: member.payload_kind.clone(),
                })
                .collect(),
        });
        let completed_by_id = new_dispositions
            .iter()
            .chain(&dispositions)
            .map(|input| (input.pack_id.as_str(), input))
            .collect::<std::collections::BTreeMap<_, _>>();
        let inputs = error_context
            .inputs
            .iter()
            .map(|input| {
                completed_by_id.get(input.pack_id.as_str()).map_or_else(
                    || input.clone(),
                    |completed| CollectionInputDisposition {
                        disposition: completed.disposition.clone(),
                        ..input.clone()
                    },
                )
            })
            .collect();
        #[cfg(test)]
        match mode {
            InstallMode::Update(_) => {
                maybe_inject_lifecycle_failure(LifecycleFailurePoint::UpdateBeforeCommit, &root)?;
            }
            InstallMode::Rebuild(_) => {
                maybe_inject_lifecycle_failure(LifecycleFailurePoint::RebuildBeforeCommit, &root)?;
            }
            InstallMode::Install => {}
        }
        Ok(CollectionMutationReport {
            version: 1,
            ok: true,
            operation: operation.into(),
            outcome: match mode {
                InstallMode::Install => "installed",
                InstallMode::Update(_) => "updated",
                InstallMode::Rebuild(_) => "rebuilt",
            }
            .into(),
            local_revision: revision,
            policy_state: "unconfigured".into(),
            attempt_id: Some(attempt),
            generation,
            inputs,
        })
    })();
    match result {
        Ok(report) if report.outcome == "idempotent" => match transaction.rollback() {
            Ok(()) => Ok(report),
            Err(error) => Err(mutation_context(error, error_context)),
        },
        Ok(report) => match transaction.commit() {
            Ok(()) => Ok(report),
            Err(error) => {
                if matches!(
                    error,
                    CollectionError::Indeterminate(_)
                        | CollectionError::IndeterminateSources { .. }
                ) {
                    error_context.observed_generation = None;
                }
                Err(mutation_context(error, error_context))
            }
        },
        Err(error) => match transaction.rollback() {
            Ok(()) => Err(mutation_context(error, error_context)),
            Err(rollback) => Err(mutation_context(
                CollectionError::IndeterminateSources {
                    detail: "operation failed and rollback could not be confirmed".into(),
                    source: Some(Box::new(error)),
                    rollback_source: Some(Box::new(rollback)),
                },
                MutationErrorContext {
                    observed_generation: None,
                    ..error_context
                },
            )),
        },
    }
}

/// Metadata-only result from constructing and fully checking a complete member set.
pub(super) struct BuiltUnifiedGeneration {
    pub(super) content_id: String,
    pub(super) canonical_sha256: String,
    pub(super) store_path: PathBuf,
}

/// Builds one complete Unified store from retained owned member inputs.
///
/// The returned value contains no Dictionary, canonical buffer, or live Store.
pub(super) fn build_unified_generation(
    root: &Path,
    stage: &Path,
    members: &[InstalledMember],
) -> Result<BuiltUnifiedGeneration, CollectionError> {
    let mut dictionaries = Vec::with_capacity(members.len());
    for member in members {
        let release_path = root.join(&member.release_path);
        let seal_path = root.join(&member.seal_path);
        let release =
            fs::read(&release_path).map_err(|error| artifact_error(release_path.clone(), error))?;
        let seal =
            fs::read(&seal_path).map_err(|error| artifact_error(seal_path.clone(), error))?;
        dictionaries.push(
            load_sealed_release(parse_pack_id(&member.pack_id)?, &release, &seal)
                .map_err(verification_error)?,
        );
    }
    let composed =
        vlajmaji_lexicon::compose_dictionaries(dictionaries).map_err(composition_error)?;
    let canonical = composed.to_json_bytes().map_err(verification_error)?;
    let canonical_sha256 = release_sha256(&canonical);
    drop(canonical);
    let content_id = content_identity(members)?;
    let store_path = stage.join("store");
    let created_store =
        crate::DictionaryStore::create(&store_path, &composed).map_err(verification_error)?;
    let recorded_digest = created_store.recorded_state_sha256().to_owned();
    drop(created_store);
    drop(composed);
    if recorded_digest != canonical_sha256 {
        return Err(CollectionError::VerificationFailed(
            "created Unified store digest differs from composed release".into(),
        ));
    }
    set_private_file(&store_path.join("dictionary.sqlite3"))?;
    set_private_directory(&store_path)?;
    crate::collections_schema::sync_file(&store_path.join("dictionary.sqlite3"))?;
    crate::collections_schema::sync_dir(&store_path)?;
    let verification = crate::DictionaryStoreVerification::verify(
        &store_path,
        crate::DictionaryStoreVerificationCheck::All,
    )
    .map_err(verification_error)?;
    if !verification.ok {
        return Err(CollectionError::VerificationFailed(
            "new Unified store did not pass All verification".into(),
        ));
    }
    write_new_file(
        &stage.join("verified.json"),
        &serde_json::to_vec(&verification).map_err(|error| CollectionError::ArtifactIoSource {
            path: Some(stage.join("verified.json")),
            source: Box::new(error),
        })?,
    )?;
    if stage.join("inputs").is_dir() {
        crate::collections_schema::sync_dir(&stage.join("inputs"))?;
    }
    crate::collections_schema::sync_dir(stage)?;
    Ok(BuiltUnifiedGeneration {
        content_id,
        canonical_sha256,
        store_path,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        sync::atomic::{AtomicUsize, Ordering},
        time::SystemTime,
    };
    use vlajmaji_lexicon::DictionaryDocument;

    static NEXT_ROOT: AtomicUsize = AtomicUsize::new(0);

    fn tiny_root(label: &str) -> PathBuf {
        for _ in 0..64 {
            let root = std::env::temp_dir().join(format!(
                "vlajmaji-s04-lifecycle-{label}-{}-{}-{}",
                std::process::id(),
                SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos(),
                NEXT_ROOT.fetch_add(1, Ordering::Relaxed)
            ));
            if !root.exists() {
                return root;
            }
        }
        panic!("exhausted bounded fresh-root names for lifecycle fixture")
    }

    fn tiny_request(revision: u64) -> CollectionRequest<'static> {
        let mut document = DictionaryDocument::default();
        document.metadata.catalogue_revision = revision;
        let dictionary = Dictionary::from_document(document).unwrap();
        let release = seal_release(PackId::Llg, PackPayloadKind::Synthetic, &dictionary).unwrap();
        CollectionRequest::new(
            vec![CollectionInput::SealedBytes {
                pack_id: PackId::Llg,
                release: std::borrow::Cow::Owned(release.release),
                seal: std::borrow::Cow::Owned(release.seal),
            }],
            CollectionSelection::Packs(vec![PackId::Llg]),
            true,
        )
        .unwrap()
    }

    fn migrate_to_v1(root: &Path) -> (u64, String) {
        let request = tiny_request(0);
        CollectionCatalogue::register(root, request.clone()).unwrap();
        let installed = CollectionCatalogue::install(root, request).unwrap();
        let generation = installed.generation.unwrap().generation_id;
        let database = crate::collections_schema::database_path(root);
        let connection = Connection::open(database).unwrap();
        connection
            .execute_batch(
                "BEGIN IMMEDIATE;
                 DROP VIEW catalogue_meta;
                 ALTER TABLE catalogue_meta_v2 RENAME TO catalogue_meta;
                 ALTER TABLE registrations_v2 RENAME TO registrations;
                 ALTER TABLE registration_locations_v2 RENAME TO registration_locations;
                 ALTER TABLE generations_v2 RENAME TO generations;
                 ALTER TABLE installed_heads_v2 RENAME TO installed_heads;
                 ALTER TABLE attempts_v2 RENAME TO attempts;
                 DELETE FROM catalogue_meta WHERE key='active_generation_id';
                 UPDATE catalogue_meta SET value='1' WHERE key='schema_version';
                 COMMIT;",
            )
            .unwrap();
        drop(connection);
        let catalogue = CollectionCatalogue::open(root).unwrap();
        (catalogue.local_revision().unwrap(), generation)
    }

    #[test]
    fn test_foreign_key_layout_requires_one_composite_constraint() {
        let expected = [
            (0, 0, "registrations", "pack_id", "pack_id"),
            (0, 1, "registrations", "release_sha256", "release_sha256"),
        ];
        let valid = vec![
            (
                0,
                0,
                "registrations".into(),
                "pack_id".into(),
                "pack_id".into(),
            ),
            (
                0,
                1,
                "registrations".into(),
                "release_sha256".into(),
                "release_sha256".into(),
            ),
        ];
        assert!(foreign_key_layout_matches(&valid, &expected));
        let incorrectly_split = vec![
            (
                0,
                0,
                "registrations".into(),
                "pack_id".into(),
                "pack_id".into(),
            ),
            (
                1,
                0,
                "registrations".into(),
                "release_sha256".into(),
                "release_sha256".into(),
            ),
        ];
        assert!(!foreign_key_layout_matches(&incorrectly_split, &expected));
    }

    #[test]
    fn test_migration_returned_faults_roll_back_before_and_after_ddl_with_context() {
        for point in [
            LifecycleFailurePoint::MigrateBeforeDdl,
            LifecycleFailurePoint::MigrateAfterDdl,
        ] {
            let root = tiny_root("migration-returned-fault");
            let (revision, generation) = migrate_to_v1(&root);
            let before = fs::read(crate::collections_schema::database_path(&root)).unwrap();
            TEST_LIFECYCLE_FAILURE_POINT.with(|selected| selected.set(Some(point)));
            let error = CollectionCatalogue::migrate(&root, revision).unwrap_err();
            let CollectionError::MutationContext {
                source,
                attempt_id: Some(_),
                prior_local_revision: Some(prior),
                proposed_generation: None,
                observed_generation: Some(observed),
                ..
            } = error
            else {
                panic!("migration failure omitted known precommit context: {error:?}");
            };
            assert_eq!(prior, revision);
            assert_eq!(observed, generation);
            assert!(matches!(
                source.as_ref(),
                CollectionError::ArtifactIoSource { source, .. }
                    if source.to_string() == "injected returned phase failure"
            ));
            assert_eq!(
                fs::read(crate::collections_schema::database_path(&root)).unwrap(),
                before
            );
            let catalogue = CollectionCatalogue::open(&root).unwrap();
            assert_eq!(catalogue.schema_version().unwrap(), 1);
            assert_eq!(
                catalogue.generation().unwrap().unwrap().generation_id,
                generation
            );
            let _ = fs::remove_dir_all(root);
        }
    }

    #[test]
    fn test_reconcile_v1_is_read_only_and_does_not_migrate() {
        let root = tiny_root("reconcile-v1-read-only");
        let (revision, _) = migrate_to_v1(&root);
        let database = crate::collections_schema::database_path(&root);
        let before = fs::read(&database).unwrap();

        let report = CollectionCatalogue::reconcile(&root).unwrap();
        assert_eq!(report.catalogue_schema_version, 1);
        assert_eq!(report.local_revision, revision);
        assert!(report.migration_required_for_mutation);
        assert_eq!(fs::read(&database).unwrap(), before);
        let catalogue = CollectionCatalogue::open(&root).unwrap();
        assert_eq!(catalogue.schema_version().unwrap(), 1);
        drop(catalogue);

        let migrated = CollectionCatalogue::migrate(&root, revision).unwrap();
        assert_eq!(migrated.schema_version, 2);
        assert_eq!(migrated.local_revision, revision + 1);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn test_update_returned_fault_rolls_back_and_dual_fault_retains_both_causes() {
        for inject_rollback_failure in [false, true] {
            let root = tiny_root("update-returned-fault");
            let initial_request = tiny_request(0);
            CollectionCatalogue::register(&root, initial_request.clone()).unwrap();
            let installed = CollectionCatalogue::install(&root, initial_request).unwrap();
            let prior_revision = installed.local_revision;
            let prior_generation = installed.generation.unwrap().generation_id;
            let before = fs::read(crate::collections_schema::database_path(&root)).unwrap();
            TEST_LIFECYCLE_FAILURE_POINT
                .with(|selected| selected.set(Some(LifecycleFailurePoint::UpdateBeforeCommit)));
            if inject_rollback_failure {
                super::super::TEST_FAIL_NEXT_ROLLBACK.with(|fail| fail.set(true));
            }
            let error =
                CollectionCatalogue::update(&root, tiny_request(1), prior_revision).unwrap_err();
            let CollectionError::MutationContext {
                source,
                attempt_id: Some(_),
                prior_local_revision: Some(revision),
                proposed_generation: Some(_),
                observed_generation,
                ..
            } = error
            else {
                panic!("update failure omitted transaction context: {error:?}");
            };
            assert_eq!(revision, prior_revision);
            if inject_rollback_failure {
                assert_eq!(observed_generation, None);
                assert!(matches!(
                    source.as_ref(),
                    CollectionError::IndeterminateSources {
                        source: Some(initiating),
                        rollback_source: Some(rollback),
                        ..
                    } if matches!(initiating.downcast_ref::<CollectionError>(), Some(CollectionError::ArtifactIoSource { source, .. }) if source.to_string() == "injected returned phase failure")
                        && matches!(rollback.downcast_ref::<CollectionError>(), Some(CollectionError::IndeterminateSources { source: Some(inner), .. }) if inner.to_string().contains("injected rollback cause"))
                ));
            } else {
                assert_eq!(
                    observed_generation.as_deref(),
                    Some(prior_generation.as_str())
                );
                assert!(matches!(
                    source.as_ref(),
                    CollectionError::ArtifactIoSource { source, .. }
                        if source.to_string() == "injected returned phase failure"
                ));
                assert_eq!(
                    fs::read(crate::collections_schema::database_path(&root)).unwrap(),
                    before
                );
                let catalogue = CollectionCatalogue::open(&root).unwrap();
                assert_eq!(catalogue.local_revision().unwrap(), prior_revision);
                assert_eq!(
                    catalogue.generation().unwrap().unwrap().generation_id,
                    prior_generation
                );
            }
            let _ = fs::remove_dir_all(root);
        }
    }

    #[test]
    fn test_rebuild_before_commit_returned_fault_preserves_active_generation() {
        let root = tiny_root("rebuild-returned-fault");
        let request = tiny_request(0);
        CollectionCatalogue::register(&root, request.clone()).unwrap();
        let installed = CollectionCatalogue::install(&root, request).unwrap();
        let prior_revision = installed.local_revision;
        let prior_generation = installed.generation.unwrap().generation_id;
        let before = fs::read(crate::collections_schema::database_path(&root)).unwrap();

        TEST_LIFECYCLE_FAILURE_POINT
            .with(|selected| selected.set(Some(LifecycleFailurePoint::RebuildBeforeCommit)));
        let error = CollectionCatalogue::rebuild(&root, prior_revision).unwrap_err();
        let CollectionError::MutationContext {
            source,
            attempt_id: Some(_),
            prior_local_revision: Some(revision),
            proposed_generation: Some(_),
            observed_generation: Some(observed),
            ..
        } = error
        else {
            panic!("rebuild failure omitted transaction context: {error:?}");
        };
        assert_eq!(revision, prior_revision);
        assert_eq!(observed, prior_generation);
        assert!(matches!(
            source.as_ref(),
            CollectionError::ArtifactIoSource { source, .. }
                if source.to_string() == "injected returned phase failure"
        ));
        assert_eq!(
            fs::read(crate::collections_schema::database_path(&root)).unwrap(),
            before
        );
        let catalogue = CollectionCatalogue::open(&root).unwrap();
        assert_eq!(catalogue.local_revision().unwrap(), prior_revision);
        assert_eq!(
            catalogue.generation().unwrap().unwrap().generation_id,
            prior_generation
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn test_catalogue_child_exit_before_commit_reconciles_to_prior_generation() {
        const ROOT_ENV: &str = "VLAJMAJI_S04_CATALOGUE_EXIT_ROOT";
        if let Some(root) = std::env::var_os(ROOT_ENV) {
            let root = PathBuf::from(root);
            TEST_LIFECYCLE_FAILURE_POINT.with(|selected| {
                selected.set(Some(LifecycleFailurePoint::RebuildExitBeforeCommit))
            });
            let revision = CollectionCatalogue::open(&root)
                .unwrap()
                .local_revision()
                .unwrap();
            let _ = CollectionCatalogue::rebuild(&root, revision);
            panic!("test-only child exit hook did not terminate rebuild");
        }

        let root = tiny_root("catalogue-child-exit");
        let request = tiny_request(0);
        CollectionCatalogue::register(&root, request.clone()).unwrap();
        let installed = CollectionCatalogue::install(&root, request).unwrap();
        let prior_revision = installed.local_revision;
        let prior_generation = installed.generation.unwrap().generation_id;
        let database = crate::collections_schema::database_path(&root);
        let before = fs::read(&database).unwrap();
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .arg("--exact")
            .arg("collections::lifecycle::tests::test_catalogue_child_exit_before_commit_reconciles_to_prior_generation")
            .arg("--nocapture")
            .env(ROOT_ENV, &root)
            .output()
            .unwrap();
        assert_eq!(
            output.status.code(),
            Some(73),
            "child stdout={} stderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );

        let catalogue = CollectionCatalogue::open(&root).unwrap();
        assert_eq!(catalogue.local_revision().unwrap(), prior_revision);
        assert_eq!(
            catalogue.generation().unwrap().unwrap().generation_id,
            prior_generation
        );
        let report = CollectionCatalogue::reconcile(&root).unwrap();
        assert_eq!(report.local_revision, prior_revision);
        assert_eq!(
            report.active_generation.as_deref(),
            Some(prior_generation.as_str())
        );
        assert!(report.attention_required);
        assert!(!report.staged_attempts.is_empty());
        assert_eq!(fs::read(&database).unwrap(), before);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn test_migration_child_exit_after_ddl_reconciles_as_v1_then_migrates() {
        const ROOT_ENV: &str = "VLAJMAJI_S04_MIGRATE_EXIT_ROOT";
        const REVISION_ENV: &str = "VLAJMAJI_S04_MIGRATE_EXIT_REVISION";
        if let Some(root) = std::env::var_os(ROOT_ENV) {
            let root = PathBuf::from(root);
            let revision = std::env::var(REVISION_ENV).unwrap().parse().unwrap();
            TEST_LIFECYCLE_FAILURE_POINT
                .with(|selected| selected.set(Some(LifecycleFailurePoint::MigrateExitAfterDdl)));
            let _ = CollectionCatalogue::migrate(&root, revision);
            panic!("test-only child exit hook did not terminate migration");
        }

        let root = tiny_root("migration-child-exit-after-ddl");
        let (revision, generation) = migrate_to_v1(&root);
        let database = crate::collections_schema::database_path(&root);
        let before = fs::read(&database).unwrap();
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .arg("--exact")
            .arg("collections::lifecycle::tests::test_migration_child_exit_after_ddl_reconciles_as_v1_then_migrates")
            .arg("--nocapture")
            .env(ROOT_ENV, &root)
            .env(REVISION_ENV, revision.to_string())
            .output()
            .unwrap();
        assert_eq!(
            output.status.code(),
            Some(73),
            "child stdout={} stderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );

        let report = CollectionCatalogue::reconcile(&root).unwrap();
        assert_eq!(report.catalogue_schema_version, 1);
        assert_eq!(report.local_revision, revision);
        assert!(report.migration_required_for_mutation);
        assert_eq!(
            report.active_generation.as_deref(),
            Some(generation.as_str())
        );
        assert_eq!(fs::read(&database).unwrap(), before);
        let migrated = CollectionCatalogue::migrate(&root, revision).unwrap();
        assert_eq!(migrated.schema_version, 2);
        assert_eq!(migrated.local_revision, revision + 1);
        assert_eq!(
            migrated.active_generation.as_deref(),
            Some(generation.as_str())
        );
        let _ = fs::remove_dir_all(root);
    }
}
