//! Explicit retained-state removal, exact reinstall, and physical reconciliation.

use super::*;
use rusqlite::params;
use std::{fs, path::Path};

const PACK_ORDER: [&str; 4] = ["lensisku-jbotci", "llg", "la-lojban", "sutysisku"];
type RegisteredReleaseMap =
    std::collections::BTreeMap<(String, String), (String, Vec<u8>, Vec<u8>)>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct LifecycleEvent {
    version: u32,
    operation: String,
    local_revision: u64,
    packs: Vec<LifecycleEventPack>,
    generation_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct LifecycleEventPack {
    pack_id: String,
    prior_state: CollectionInstallationState,
    release_sha256: Option<String>,
}

fn metadata_error(detail: impl Into<String>) -> CollectionError {
    CollectionError::UnsupportedVersion(format!("lifecycle event metadata: {}", detail.into()))
}

fn validate_event(
    operation: &str,
    prior_revision: i64,
    bytes: &[u8],
) -> Result<LifecycleEvent, CollectionError> {
    let event: LifecycleEvent = serde_json::from_slice(bytes)
        .map_err(|error| metadata_error(format!("malformed event: {error}")))?;
    let revision = u64::try_from(prior_revision)
        .map_err(|_| metadata_error("event attempt has a negative prior revision"))?;
    let expected_revision = revision
        .checked_add(1)
        .ok_or_else(|| metadata_error("event revision overflowed"))?;
    i64::try_from(expected_revision)
        .map_err(|_| metadata_error("event revision exceeds SQLite INTEGER range"))?;
    if event.version != 1
        || event.operation != operation
        || event.local_revision != expected_revision
        || !matches!(operation, "remove" | "reinstall")
        || event.packs.is_empty()
    {
        return Err(metadata_error("event identity or version is inconsistent"));
    }
    let mut previous = None;
    for item in &event.packs {
        let order = PACK_ORDER
            .iter()
            .position(|candidate| *candidate == item.pack_id)
            .ok_or_else(|| metadata_error("event names an unknown pack"))?;
        if previous.is_some_and(|prior| prior >= order) {
            return Err(metadata_error(
                "event pack IDs are duplicated or out of order",
            ));
        }
        previous = Some(order);
        let _ = parse_pack_id(&item.pack_id)?;
        if let Some(digest) = &item.release_sha256
            && (digest.len() != 64
                || !digest
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)))
        {
            return Err(metadata_error(
                "event release digest is not lowercase SHA-256",
            ));
        }
        match (operation, item.prior_state, item.release_sha256.as_ref()) {
            ("remove", CollectionInstallationState::Installed, Some(_)) => {}
            ("remove", CollectionInstallationState::RegisteredOnly, None) => {}
            ("remove", _, _) => {
                return Err(metadata_error(
                    "remove event prior state and release digest are inconsistent",
                ));
            }
            ("reinstall", CollectionInstallationState::Removed, Some(_)) => {}
            ("reinstall", _, _) => {
                return Err(metadata_error(
                    "reinstall event prior state and release digest are inconsistent",
                ));
            }
            _ => return Err(metadata_error("unrecognized lifecycle operation")),
        }
    }
    if operation == "reinstall" && event.generation_id.is_none() {
        return Err(metadata_error(
            "event generation disposition is inconsistent",
        ));
    }
    Ok(event)
}

pub(super) fn installation_state(
    connection: &Connection,
    root: &Path,
    pack_id: &str,
    installed: bool,
) -> Result<CollectionInstallationState, CollectionError> {
    if installed {
        return Ok(CollectionInstallationState::Installed);
    }
    if crate::collections_schema::schema_version(
        connection,
        &crate::collections_schema::database_path(root),
    )? < crate::collections_schema::VERSION_2
    {
        return Ok(CollectionInstallationState::RegisteredOnly);
    }
    let tables = crate::collections_schema::tables_v2();
    let mut statement = connection
        .prepare(&format!(
            "SELECT operation,prior_revision,detail FROM {} WHERE status='committed' AND operation IN ('remove','reinstall') ORDER BY rowid DESC",
            tables.attempts
        ))
        .map_err(|error| db_error(root, error))?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, String>(2)?,
            ))
        })
        .map_err(|error| db_error(root, error))?;
    for row in rows {
        let (operation, prior_revision, detail) = row.map_err(|error| db_error(root, error))?;
        let event = validate_event(&operation, prior_revision, detail.as_bytes())?;
        if let Some(item) = event.packs.iter().find(|item| item.pack_id == pack_id) {
            return match operation.as_str() {
                "remove" => Ok(CollectionInstallationState::Removed),
                "reinstall" => Err(metadata_error(format!(
                    "reinstall event for {} has no active head",
                    item.pack_id
                ))),
                _ => Err(metadata_error("unrecognized lifecycle event")),
            };
        }
    }
    Ok(CollectionInstallationState::RegisteredOnly)
}

pub(super) fn remove(
    root: &Path,
    packs: &[PackId],
    expected_local_revision: u64,
) -> Result<CollectionMutationReport, CollectionError> {
    if packs.is_empty() {
        return Err(CollectionError::InvalidRequest(
            "remove requires at least one registered pack".into(),
        ));
    }
    let mut selected = packs.to_vec();
    selected.sort_by_key(|pack| pack_order(pack.as_str()));
    selected.dedup();
    if selected.len() != packs.len() {
        return Err(CollectionError::InvalidRequest(
            "remove pack IDs must be unique".into(),
        ));
    }
    remove_or_reinstall(root, selected, expected_local_revision, None)
}

pub(super) fn reinstall(
    root: &Path,
    request: CollectionRequest<'_>,
    registered_release_sha256: &str,
    expected_local_revision: u64,
) -> Result<CollectionMutationReport, CollectionError> {
    if registered_release_sha256.len() != 64
        || !registered_release_sha256
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(CollectionError::InvalidRequest(
            "registered_release_sha256 must be lowercase SHA-256".into(),
        ));
    }
    validate_selection(&request.inputs, &request.selection)?;
    let selected = selected_pack_ids(&request.inputs, &request.selection);
    if selected.len() != 1 {
        return Err(CollectionError::InvalidRequest(
            "reinstall requires exactly one selected pack".into(),
        ));
    }
    let pack = *selected.iter().next().expect("one selected pack checked");
    let cwd = operation_cwd(root, &request)?;
    let absolute_root = absolute_path_with_cwd(root, cwd.as_deref())?;
    let input_ids = request.inputs.iter().map(input_pack_id).collect::<Vec<_>>();
    let normalized = normalize_selected(request, cwd.as_deref())?;
    let dispositions = input_dispositions(&input_ids, &selected, &normalized);
    let input = normalized
        .into_iter()
        .next()
        .ok_or_else(|| CollectionError::InvalidRequest("selected input is absent".into()))?;
    if input.pack_id != pack {
        return Err(CollectionError::InvalidRequest(
            "reinstall input pack differs from the selected pack".into(),
        ));
    }
    remove_or_reinstall(
        &absolute_root,
        vec![pack],
        expected_local_revision,
        Some((input, registered_release_sha256.to_owned(), dispositions)),
    )
}

fn remove_or_reinstall(
    root: &Path,
    packs: Vec<PackId>,
    expected_revision: u64,
    reinstall: Option<(NormalizedPack, String, Vec<CollectionInputDisposition>)>,
) -> Result<CollectionMutationReport, CollectionError> {
    let root = absolute_path(root)?;
    let connection = crate::collections_schema::open_writable(&root)?;
    let mut transaction = TransactionGuard::begin(&connection, &root)?;
    let mut reinstall = reinstall;
    let result = (|| {
        let tables = crate::collections_schema::tables_for(&connection, &root)?;
        require_v2(&connection, &root, tables)?;
        let prior_revision = crate::collections_schema::local_revision(&connection, &root)?;
        if prior_revision != expected_revision {
            return Err(CollectionError::RevisionConflict {
                expected: expected_revision,
                observed: prior_revision,
            });
        }
        let revision = prior_revision.checked_add(1).ok_or_else(|| {
            CollectionError::UnsupportedVersion("catalogue local_revision overflowed u64".into())
        })?;
        i64::try_from(revision).map_err(|_| {
            CollectionError::UnsupportedVersion("catalogue revision exceeds SQLite range".into())
        })?;
        let prior_sql = i64::try_from(prior_revision).map_err(|_| {
            CollectionError::UnsupportedVersion("catalogue revision exceeds SQLite range".into())
        })?;
        let active = current_members(&connection, &root)?;
        let prior_generation = read_generation(&connection, &root)?;
        let registrations = registered_releases(&connection, &root, tables)?;
        if let Some((input, digest, _)) = &reinstall {
            if !registrations.contains_key(&(input.pack_id.as_str().to_owned(), digest.clone())) {
                return Err(CollectionError::UnknownRegisteredRelease {
                    pack: input.pack_id,
                    release_sha256: digest.clone(),
                });
            }
            let state = installation_state(
                &connection,
                &root,
                input.pack_id.as_str(),
                active
                    .iter()
                    .any(|member| member.pack_id == input.pack_id.as_str()),
            )?;
            if state == CollectionInstallationState::RegisteredOnly {
                return Err(CollectionError::NotInstalled {
                    pack: Some(input.pack_id),
                });
            }
            if let Some(current) = active
                .iter()
                .find(|member| member.pack_id == input.pack_id.as_str())
            {
                if current.release_sha256 != *digest {
                    return Err(CollectionError::UpdateRequired(input.pack_id));
                }
                if release_sha256(&input.release) != *digest {
                    return Err(CollectionError::IdentityConflict {
                        category: "reinstall input release".into(),
                        key: format!("{}:{digest}", input.pack_id.as_str()),
                    });
                }
                verify_registration_matches_input(&connection, &root, tables, input, digest)?;
                let generation = prior_generation
                    .as_ref()
                    .ok_or_else(|| metadata_error("installed pack has no active generation"))?;
                let inputs = reinstall
                    .as_ref()
                    .map_or_else(Vec::new, |(_, _, inputs)| inputs.clone());
                drop(reinstall.take());
                verify_current_generation(&root, &active, generation)?;
                return Ok(CollectionMutationReport {
                    version: 1,
                    ok: true,
                    operation: "reinstall".into(),
                    outcome: "idempotent".into(),
                    local_revision: prior_revision,
                    policy_state: "unconfigured".into(),
                    attempt_id: None,
                    generation: Some(generation.clone()),
                    inputs,
                });
            } else if state != CollectionInstallationState::Removed {
                return Err(CollectionError::NotInstalled {
                    pack: Some(input.pack_id),
                });
            }
            if release_sha256(&input.release) != *digest {
                return Err(CollectionError::IdentityConflict {
                    category: "reinstall input release".into(),
                    key: format!("{}:{digest}", input.pack_id.as_str()),
                });
            }
            verify_registration_matches_input(&connection, &root, tables, input, digest)?;
        } else {
            for pack in &packs {
                let is_registered = registrations.keys().any(|(id, _)| id == pack.as_str());
                if !is_registered {
                    return Err(CollectionError::InvalidRequest(format!(
                        "pack {} is not registered",
                        pack.as_str()
                    )));
                }
            }
        }

        let event_packs = if let Some((input, digest, _)) = &reinstall {
            vec![LifecycleEventPack {
                pack_id: input.pack_id.as_str().into(),
                prior_state: CollectionInstallationState::Removed,
                release_sha256: Some(digest.clone()),
            }]
        } else {
            let mut events = Vec::new();
            for pack in &packs {
                let is_active = active.iter().any(|member| member.pack_id == pack.as_str());
                let state = installation_state(&connection, &root, pack.as_str(), is_active)?;
                if state != CollectionInstallationState::Removed {
                    events.push(LifecycleEventPack {
                        pack_id: pack.as_str().into(),
                        prior_state: state,
                        release_sha256: active
                            .iter()
                            .find(|member| member.pack_id == pack.as_str())
                            .map(|member| member.release_sha256.clone()),
                    });
                }
            }
            events
        };
        if reinstall.is_none() && event_packs.is_empty() {
            return Ok(CollectionMutationReport {
                version: 1,
                ok: true,
                operation: "remove".into(),
                outcome: "idempotent".into(),
                local_revision: prior_revision,
                policy_state: "unconfigured".into(),
                attempt_id: None,
                generation: prior_generation,
                inputs: Vec::new(),
            });
        }

        let mut next = active.clone();
        let affected: Vec<PackId> = event_packs
            .iter()
            .map(|item| parse_pack_id(&item.pack_id))
            .collect::<Result<_, _>>()?;
        if let Some((input, digest, _)) = &reinstall {
            next.push(InstalledMember {
                pack_id: input.pack_id.as_str().into(),
                release_sha256: digest.clone(),
                payload_kind: payload_kind_name(input.seal.payload_kind).into(),
                release_path: String::new(),
                seal_path: String::new(),
                report_path: None,
            });
            next.sort_by_key(|member| pack_order(&member.pack_id));
        } else {
            next.retain(|member| !affected.iter().any(|pack| member.pack_id == pack.as_str()));
        }

        let reinstalling = reinstall.is_some();
        let reinstall_inputs = reinstall
            .as_ref()
            .map_or_else(Vec::new, |(_, _, inputs)| inputs.clone());
        let operation = if reinstalling { "reinstall" } else { "remove" };
        let mut attempt = attempt_id(&root, operation, revision);
        let generation = if next == active {
            prior_generation.clone()
        } else if next.is_empty() {
            None
        } else {
            let (published, used_attempt) = publish_generation(
                &connection,
                &root,
                tables,
                GenerationPublication {
                    members: &next,
                    supplied: reinstall.take().map(|(input, _, _)| input),
                    revision,
                    operation,
                    initial_attempt: &attempt,
                },
            )?;
            attempt = used_attempt;
            Some(published)
        };
        if next != active && generation.is_none() {
            connection
                .execute(&format!("DELETE FROM {}", tables.installed_heads), [])
                .map_err(|error| db_error(&root, error))?;
            crate::collections_schema::update_active_generation(&connection, tables, "")
                .map_err(|error| db_error(&root, error))?;
        }
        let event = LifecycleEvent {
            version: 1,
            operation: if reinstalling { "reinstall" } else { "remove" }.into(),
            local_revision: revision,
            packs: event_packs,
            generation_id: generation.as_ref().map(|value| value.generation_id.clone()),
        };
        let detail =
            serde_json::to_string(&event).map_err(|error| CollectionError::ArtifactIoSource {
                path: None,
                source: Box::new(error),
            })?;
        crate::collections_schema::update_revision(&connection, tables, revision)
            .map_err(|error| db_error(&root, error))?;
        connection.execute(
            &format!("INSERT INTO {}(attempt_id,operation,status,prior_revision,detail) VALUES(?1,?2,'committed',?3,?4)", tables.attempts),
            params![attempt, event.operation, prior_sql, detail],
        ).map_err(|error| db_error(&root, error))?;
        Ok(CollectionMutationReport {
            version: 1,
            ok: true,
            operation: event.operation,
            outcome: if reinstalling {
                "reinstalled"
            } else {
                "removed"
            }
            .into(),
            local_revision: revision,
            policy_state: "unconfigured".into(),
            attempt_id: Some(attempt),
            generation,
            inputs: reinstall_inputs,
        })
    })();
    match result {
        Ok(report) if report.outcome == "idempotent" => {
            transaction.rollback()?;
            Ok(report)
        }
        Ok(report) => {
            transaction.commit()?;
            Ok(report)
        }
        Err(error) => match transaction.rollback() {
            Ok(()) => Err(error),
            Err(rollback) => Err(CollectionError::IndeterminateSources {
                detail: "lifecycle operation failed and rollback is uncertain".into(),
                source: Some(Box::new(error)),
                rollback_source: Some(Box::new(rollback)),
            }),
        },
    }
}

fn registered_releases(
    connection: &Connection,
    root: &Path,
    tables: crate::collections_schema::CatalogueTables,
) -> Result<RegisteredReleaseMap, CollectionError> {
    let mut statement = connection
        .prepare(&format!(
            "SELECT pack_id,release_sha256,payload_kind,seal_json,descriptor_json FROM {}",
            tables.registrations
        ))
        .map_err(|error| db_error(root, error))?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
            ))
        })
        .map_err(|error| db_error(root, error))?;
    let mut releases = std::collections::BTreeMap::new();
    for row in rows {
        let (pack, digest, kind, seal, descriptor) = row.map_err(|error| db_error(root, error))?;
        releases.insert((pack, digest), (kind, seal, descriptor));
    }
    Ok(releases)
}

fn verify_registration_matches_input(
    connection: &Connection,
    root: &Path,
    tables: crate::collections_schema::CatalogueTables,
    input: &NormalizedPack,
    digest: &str,
) -> Result<(), CollectionError> {
    let (kind, seal, descriptor): (String, Vec<u8>, Vec<u8>) = connection.query_row(
        &format!("SELECT payload_kind,seal_json,descriptor_json FROM {} WHERE pack_id=?1 AND release_sha256=?2", tables.registrations),
        params![input.pack_id.as_str(), digest],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    ).map_err(|error| db_error(root, error))?;
    if kind != payload_kind_name(input.seal.payload_kind)
        || PackSeal::from_json_bytes(&seal).map_err(verification_error)? != input.seal
        || descriptor != descriptor_bytes(input)?
    {
        return Err(CollectionError::IdentityConflict {
            category: "registered release declaration".into(),
            key: format!("{}:{digest}", input.pack_id.as_str()),
        });
    }
    Ok(())
}

fn verify_current_generation(
    root: &Path,
    members: &[InstalledMember],
    generation: &CollectionGeneration,
) -> Result<(), CollectionError> {
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
    if release_sha256(&canonical) != generation.canonical_sha256 {
        return Err(CollectionError::VerificationFailed(
            "current generation canonical digest differs from retained inputs".into(),
        ));
    }
    drop(canonical);
    drop(composed);
    let store = crate::DictionaryStore::open(&generation.store_path).map_err(verification_error)?;
    if store.recorded_state_sha256() != generation.canonical_sha256 {
        return Err(CollectionError::VerificationFailed(
            "current store header digest differs from catalogue generation".into(),
        ));
    }
    drop(store);
    let verification = crate::DictionaryStoreVerification::verify(
        &generation.store_path,
        crate::DictionaryStoreVerificationCheck::All,
    )
    .map_err(verification_error)?;
    if !verification.ok {
        return Err(CollectionError::VerificationFailed(
            "current generation failed All verification".into(),
        ));
    }
    Ok(())
}

struct GenerationPublication<'a> {
    members: &'a [InstalledMember],
    supplied: Option<NormalizedPack>,
    revision: u64,
    operation: &'a str,
    initial_attempt: &'a str,
}

fn publish_generation(
    connection: &Connection,
    root: &Path,
    tables: crate::collections_schema::CatalogueTables,
    publication: GenerationPublication<'_>,
) -> Result<(CollectionGeneration, String), CollectionError> {
    let GenerationPublication {
        members,
        supplied,
        revision,
        operation,
        initial_attempt,
    } = publication;
    let catalogue_id = crate::collections_schema::catalogue_identity(connection, tables)
        .map_err(|error| db_error(root, error))?;
    let generations = root.join("generations");
    let mut allocated = None;
    let mut candidate = initial_attempt.to_owned();
    for _ in 0..8 {
        let stage = generations.join(&candidate);
        match try_create_private_dir(&stage) {
            Ok(()) => {
                allocated = Some((candidate.clone(), stage));
                break;
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                candidate = attempt_id(root, operation, revision);
                continue;
            }
            Err(error) => return Err(artifact_error(stage, error)),
        }
    }
    let (attempt, stage) =
        allocated.ok_or_else(|| CollectionError::OccupiedDestination(generations.clone()))?;
    let generation_id = format!("{catalogue_id}-{attempt}");
    let marker = StageMarker {
        version: 1,
        attempt_id: attempt.clone(),
        operation: operation.to_owned(),
    };
    let marker_bytes =
        serde_json::to_vec(&marker).map_err(|error| CollectionError::ArtifactIoSource {
            path: Some(stage.join("attempt.json")),
            source: Box::new(error),
        })?;
    write_new_file(&stage.join("attempt.json"), &marker_bytes)?;
    let mut staged_members = members.to_vec();
    if let Some(pack) = supplied {
        let inputs_dir = stage.join("inputs");
        create_private_dir(&inputs_dir)?;
        let input_dir = inputs_dir.join(pack.pack_id.as_str());
        create_private_dir(&input_dir)?;
        write_new_file(&input_dir.join("release.json"), &pack.release)?;
        write_new_file(&input_dir.join("seal.json"), &pack.seal_bytes)?;
        if let Some(report) = &pack.report {
            let report_bytes =
                serde_json::to_vec(report).map_err(|error| CollectionError::ArtifactIoSource {
                    path: Some(input_dir.join("report.json")),
                    source: Box::new(error),
                })?;
            write_new_file(&input_dir.join("report.json"), &report_bytes)?;
        }
        crate::collections_schema::sync_dir(&input_dir)?;
        crate::collections_schema::sync_dir(&inputs_dir)?;
        let member = staged_members
            .iter_mut()
            .find(|member| member.pack_id == pack.pack_id.as_str())
            .ok_or_else(|| metadata_error("reinstall member is absent from new generation"))?;
        let prefix = PathBuf::from("generations")
            .join(&attempt)
            .join("inputs")
            .join(pack.pack_id.as_str());
        member.release_path = prefix.join("release.json").to_string_lossy().into_owned();
        member.seal_path = prefix.join("seal.json").to_string_lossy().into_owned();
        member.report_path = pack
            .report
            .as_ref()
            .map(|_| prefix.join("report.json").to_string_lossy().into_owned());
        // The new stage now owns exact normalized bytes; do not retain typed
        // adapter models while loading the complete unified dictionary.
        drop(pack);
    }
    let built = lifecycle::build_unified_generation(root, &stage, &staged_members)?;
    let revision_sql = i64::try_from(revision).map_err(|_| {
        CollectionError::UnsupportedVersion(
            "catalogue revision exceeds SQLite INTEGER range".into(),
        )
    })?;
    let members_bytes =
        serde_json::to_vec(&staged_members).map_err(|error| CollectionError::ArtifactIoSource {
            path: Some(stage.join("members.json")),
            source: Box::new(error),
        })?;
    connection
        .execute(
            &format!("INSERT INTO {}(generation_id,content_id,canonical_sha256,store_path,members_json,local_revision) VALUES(?1,?2,?3,?4,?5,?6)", tables.generations),
            params![generation_id, built.content_id, built.canonical_sha256, native_path_bytes(&built.store_path), members_bytes, revision_sql],
        )
        .map_err(|error| db_error(root, error))?;
    connection
        .execute(&format!("DELETE FROM {}", tables.installed_heads), [])
        .map_err(|error| db_error(root, error))?;
    for member in &staged_members {
        connection
            .execute(
                &format!("INSERT INTO {}(pack_id,release_sha256,payload_kind,generation_id) VALUES(?1,?2,?3,?4)", tables.installed_heads),
                params![member.pack_id, member.release_sha256, member.payload_kind, generation_id],
            )
            .map_err(|error| db_error(root, error))?;
    }
    crate::collections_schema::update_active_generation(connection, tables, &generation_id)
        .map_err(|error| db_error(root, error))?;
    crate::collections_schema::sync_dir(&generations)?;
    let generation = read_generation(connection, root)?
        .ok_or_else(|| metadata_error("published generation pointer could not be read back"))?;
    Ok((generation, marker.attempt_id))
}

pub(super) fn reconcile(root: &Path) -> Result<CollectionReconciliationReport, CollectionError> {
    let root = absolute_path(root)?;
    let before = journal_observation(&root);
    let connection = crate::collections_schema::open_writable(&root)?;
    let mut transaction = TransactionGuard::begin(&connection, &root)?;
    let result = (|| {
        let schema = crate::collections_schema::schema_version(
            &connection,
            &crate::collections_schema::database_path(&root),
        )?;
        let tables = crate::collections_schema::tables_for(&connection, &root)?;
        let revision = crate::collections_schema::local_revision(&connection, &root)?;
        let generation = read_generation(&connection, &root)?;
        let mut attempts = Vec::new();
        let mut statement = connection
            .prepare(&format!(
                "SELECT attempt_id,operation,status,prior_revision,detail FROM {} ORDER BY rowid",
                tables.attempts
            ))
            .map_err(|error| db_error(&root, error))?;
        let rows = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, String>(4)?,
                ))
            })
            .map_err(|error| db_error(&root, error))?;
        for row in rows {
            let (attempt_id, operation, status, prior_revision, detail) =
                row.map_err(|error| db_error(&root, error))?;
            attempts.push(CollectionReconciliationAttempt {
                attempt_id,
                operation,
                status,
                prior_revision: u64::try_from(prior_revision)
                    .map_err(|_| metadata_error("negative attempt prior revision"))?,
                detail,
            });
        }
        let staged_attempts = orphan_stage_attempts(&root, &connection)?;
        let minimum = generation.as_ref().map(|generation| {
            match crate::DictionaryStore::open(&generation.store_path) {
                Ok(store) => {
                    drop(store);
                    CollectionDerivedStoreMinimum {
                        status: "supported".into(),
                        diagnostic: None,
                    }
                }
                Err(error) => CollectionDerivedStoreMinimum {
                    status: "unsupported_or_unreadable".into(),
                    diagnostic: Some(error.to_string()),
                },
            }
        });
        let attention = !staged_attempts.is_empty()
            || minimum
                .as_ref()
                .is_some_and(|value| value.status != "supported");
        Ok(CollectionReconciliationReport {
            version: 1,
            ok: true,
            operation: "reconcile".into(),
            root_path: native_path_report(&root),
            catalogue_schema_version: schema,
            local_revision: revision,
            active_generation: generation.map(|value| value.generation_id),
            retained_generation_count: connection
                .query_row(
                    &format!("SELECT count(*) FROM {}", tables.generations),
                    [],
                    |row| row.get::<_, i64>(0),
                )
                .map_err(|error| db_error(&root, error))?
                as usize,
            attempts,
            staged_attempts,
            journal_before: before.clone(),
            journal_after: journal_observation(&root),
            sqlite_recovery: "writable_open_completed".into(),
            verification_scope: "minimum_header_only".into(),
            active_store_minimum: minimum,
            attention_required: attention,
            migration_required_for_mutation: schema == crate::collections_schema::VERSION_1,
        })
    })();
    match result {
        Ok(report) => {
            transaction.rollback()?;
            Ok(report)
        }
        Err(error) => match transaction.rollback() {
            Ok(()) => Err(error),
            Err(rollback) => Err(CollectionError::IndeterminateSources {
                detail: "reconcile read transaction rollback is uncertain".into(),
                source: Some(Box::new(error)),
                rollback_source: Some(Box::new(rollback)),
            }),
        },
    }
}

fn journal_observation(root: &Path) -> CollectionJournalObservation {
    let path = Path::new(&format!(
        "{}-journal",
        crate::collections_schema::database_path(root).display()
    ))
    .to_path_buf();
    match std::fs::metadata(path) {
        Ok(metadata) => CollectionJournalObservation {
            present: true,
            byte_length: Some(metadata.len()),
        },
        Err(_) => CollectionJournalObservation {
            present: false,
            byte_length: None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::validate_event;
    use serde_json::{Value, json};

    fn remove_event(prior_state: &str, digest: Option<&str>, generation: Option<&str>) -> Value {
        json!({
            "version": 1,
            "operation": "remove",
            "local_revision": 2,
            "packs": [{
                "pack_id": "llg",
                "prior_state": prior_state,
                "release_sha256": digest
            }],
            "generation_id": generation
        })
    }

    #[test]
    fn test_validate_event_remove_accepts_installed_and_registered_only_dispositions() {
        let digest = "a".repeat(64);
        let installed = remove_event("installed", Some(&digest), Some("generation-2"));
        assert!(validate_event("remove", 1, installed.to_string().as_bytes()).is_ok());

        let registered = remove_event("registered_only", None, None);
        assert!(validate_event("remove", 1, registered.to_string().as_bytes()).is_ok());
    }

    #[test]
    fn test_validate_event_reinstall_requires_removed_digest_and_generation() {
        let digest = "b".repeat(64);
        let valid = json!({
            "version": 1,
            "operation": "reinstall",
            "local_revision": 2,
            "packs": [{
                "pack_id": "llg",
                "prior_state": "removed",
                "release_sha256": digest
            }],
            "generation_id": "generation-2"
        });
        assert!(validate_event("reinstall", 1, valid.to_string().as_bytes()).is_ok());

        let missing_digest = json!({
            "version": 1,
            "operation": "reinstall",
            "local_revision": 2,
            "packs": [{"pack_id": "llg", "prior_state": "removed", "release_sha256": null}],
            "generation_id": "generation-2"
        });
        assert!(validate_event("reinstall", 1, missing_digest.to_string().as_bytes()).is_err());

        let missing_generation = json!({
            "version": 1,
            "operation": "reinstall",
            "local_revision": 2,
            "packs": [{
                "pack_id": "llg",
                "prior_state": "removed",
                "release_sha256": "b".repeat(64)
            }],
            "generation_id": null
        });
        assert!(validate_event("reinstall", 1, missing_generation.to_string().as_bytes()).is_err());
    }

    #[test]
    fn test_validate_event_rejects_duplicate_order_version_and_revision_overflow() {
        let digest = "c".repeat(64);
        let mut duplicate = remove_event("registered_only", None, None);
        duplicate["packs"] = json!([
            {"pack_id":"llg", "prior_state":"registered_only", "release_sha256":null},
            {"pack_id":"llg", "prior_state":"registered_only", "release_sha256":null}
        ]);
        assert!(validate_event("remove", 1, duplicate.to_string().as_bytes()).is_err());

        let mut out_of_order = remove_event("registered_only", None, None);
        out_of_order["packs"] = json!([
            {"pack_id":"llg", "prior_state":"registered_only", "release_sha256":null},
            {"pack_id":"lensisku-jbotci", "prior_state":"registered_only", "release_sha256":null}
        ]);
        assert!(validate_event("remove", 1, out_of_order.to_string().as_bytes()).is_err());

        let mut unsupported = remove_event("registered_only", None, None);
        unsupported["version"] = json!(2);
        assert!(validate_event("remove", 1, unsupported.to_string().as_bytes()).is_err());

        let mut wrong_digest = remove_event("installed", Some(&digest), None);
        wrong_digest["packs"][0]["release_sha256"] = json!("C".repeat(64));
        assert!(validate_event("remove", 1, wrong_digest.to_string().as_bytes()).is_err());

        let overflow = remove_event("registered_only", None, None);
        assert!(validate_event("remove", i64::MAX, overflow.to_string().as_bytes()).is_err());
    }
}
