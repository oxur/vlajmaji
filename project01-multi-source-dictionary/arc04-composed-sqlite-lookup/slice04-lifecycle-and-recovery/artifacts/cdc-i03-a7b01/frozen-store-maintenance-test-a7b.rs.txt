//! Public removal, exact reinstall, and physical reconciliation contracts.

use std::{
    borrow::Cow,
    collections::BTreeMap,
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use sha2::{Digest, Sha256};
use vlajmaji_lexicon::{
    Dictionary, DictionaryDocument, Eligibility, JbotciImportOptions, LaLojbanImportOptions,
    LlgImportOptions, PackId, PackPayloadKind, Presence, SutysiskuImportOptions,
    compose_dictionaries, import_jbotci_export, import_la_lojban_export, import_llg_lists,
    import_sutysisku_export, seal_release,
};
use vlajmaji_store::{
    CollectionCatalogue, CollectionErrorCategory, CollectionInput, CollectionInstallationState,
    CollectionRequest, CollectionSelection,
};

fn fresh_root(label: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "vlajmaji-s04-maintenance-{label}-{}-{stamp}",
        std::process::id()
    ))
}

fn sealed_llg() -> (Vec<u8>, Vec<u8>, String) {
    sealed_llg_revision(0)
}

fn sealed_llg_revision(revision: u64) -> (Vec<u8>, Vec<u8>, String) {
    let mut document = DictionaryDocument::default();
    document.metadata.catalogue_revision = revision;
    let dictionary = Dictionary::from_document(document).unwrap();
    let sealed = seal_release(PackId::Llg, PackPayloadKind::Synthetic, &dictionary).unwrap();
    let digest = format!("{:x}", Sha256::digest(&sealed.release));
    (sealed.release, sealed.seal, digest)
}

fn sealed_pack(pack: PackId) -> (Vec<u8>, Vec<u8>, String) {
    let dictionary = Dictionary::from_document(DictionaryDocument::default()).unwrap();
    let sealed = seal_release(pack, PackPayloadKind::Synthetic, &dictionary).unwrap();
    let digest = format!("{:x}", Sha256::digest(&sealed.release));
    (sealed.release, sealed.seal, digest)
}

type FamilyInventory = BTreeMap<String, BTreeMap<String, usize>>;

fn row_inventory<T: serde::Serialize>(rows: &[T]) -> BTreeMap<String, usize> {
    let mut inventory = BTreeMap::new();
    for row in rows {
        let bytes = serde_json::to_vec(row).unwrap();
        *inventory
            .entry(format!("{:x}", Sha256::digest(bytes)))
            .or_insert(0) += 1;
    }
    inventory
}

fn family_inventory(dictionary: &Dictionary) -> FamilyInventory {
    let metadata = dictionary.metadata();
    let registry = dictionary.registry().as_document();
    let mut inventory = BTreeMap::new();
    macro_rules! metadata_families {
        ($($field:ident),+ $(,)?) => {$({
            inventory.insert(stringify!($field).to_owned(), row_inventory(&metadata.$field));
        })+};
    }
    metadata_families!(
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
    macro_rules! registry_families {
        ($($field:ident),+ $(,)?) => {$({
            inventory.insert(stringify!($field).to_owned(), row_inventory(&registry.$field));
        })+};
    }
    registry_families!(
        sources,
        contributors,
        distributors,
        aliases,
        lineage,
        snapshots,
        occurrences
    );
    inventory.insert("captures".to_owned(), row_inventory(dictionary.captures()));
    inventory
}

fn max_family_inventory(dictionaries: &[Dictionary]) -> FamilyInventory {
    let mut expected = FamilyInventory::new();
    for dictionary in dictionaries {
        for (family, rows) in family_inventory(dictionary) {
            let merged = expected.entry(family).or_default();
            for (record, count) in rows {
                let slot = merged.entry(record).or_default();
                *slot = (*slot).max(count);
            }
        }
    }
    expected
}

fn readiness_four_pack_models(jbotci_definition: &str) -> Vec<(PackId, Dictionary)> {
    const SUTYSISKU: &[u8] = br#"{"farvi":{"d":"synthetic one","t":"gismu","n":null},"farvi":{"d":"synthetic two","t":"gismu"},"zoizoizoi":{"d":"retained excluded","t":"unsupported"}}"#;
    const LLG_GISMU_HEX: &str = "206661727669202020202020202020202020202067726f772020202020202020202020202020202020202020202020202020202020202020202020202020642020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020";
    const EMPTY: &[u8] = b"";
    const NOTICE: &[u8] = b"synthetic notice\n";
    let lensisku = format!(
        r#"[{{"word":"farvi","word_type":"gismu","definition":"{jbotci_definition}","definition_id":333,"score":1.2500,"user":{{"username":"shared"}}}}]"#
    );
    let lensisku_sha256 = format!("{:x}", Sha256::digest(lensisku.as_bytes()));
    let metadata = format!(
        "language_tag = \"en\"\nlanguage_realname = \"English\"\nsource_language_tag = \"jbo\"\nformat = \"json\"\npositive_scores_only = false\nfilename = \"dictionary-en.json\"\nmetadata_url = \"https://example.invalid/languages\"\ndownload_url = \"https://example.invalid/export\"\nlensisku_created_at = \"2026-10-04T03:07:08Z\"\nsha256 = \"{}\"\ndefinition_count = 1\nentry_count = 1\n",
        lensisku_sha256,
    );
    let jbotci = import_jbotci_export(
        lensisku.as_bytes(),
        metadata.as_bytes(),
        &JbotciImportOptions {
            revision: "1111111111111111111111111111111111111111".into(),
            expected_raw_sha256: lensisku_sha256.clone(),
            expected_metadata_sha256: format!("{:x}", Sha256::digest(metadata.as_bytes())),
        },
    )
    .unwrap()
    .into_dictionary();
    let la = import_la_lojban_export(
        lensisku.as_bytes(),
        &LaLojbanImportOptions {
            revision: "2222222222222222222222222222222222222222".into(),
            expected_raw_sha256: format!("{:x}", Sha256::digest(lensisku.as_bytes())),
            raw_locator: None,
            notice_locator: None,
        },
    )
    .unwrap()
    .into_dictionary();
    let sut = import_sutysisku_export(
        SUTYSISKU,
        &SutysiskuImportOptions {
            revision: "3333333333333333333333333333333333333333".into(),
            expected_raw_sha256: format!("{:x}", Sha256::digest(SUTYSISKU)),
            raw_locator: None,
            notice_locator: None,
        },
    )
    .unwrap()
    .into_dictionary();
    let (gismu_bytes, remainder) = LLG_GISMU_HEX.as_bytes().as_chunks::<2>();
    assert!(remainder.is_empty());
    let gismu = gismu_bytes
        .iter()
        .map(|pair| {
            let digit = |byte: u8| (byte as char).to_digit(16).unwrap() as u8;
            (digit(pair[0]) << 4) | digit(pair[1])
        })
        .collect::<Vec<_>>();
    let llg = import_llg_lists(
        &gismu,
        EMPTY,
        EMPTY,
        NOTICE,
        &LlgImportOptions {
            expected_gismu_sha256: format!("{:x}", Sha256::digest(&gismu)),
            expected_cmavo_sha256: format!("{:x}", Sha256::digest(EMPTY)),
            expected_rafsi_sha256: format!("{:x}", Sha256::digest(EMPTY)),
            expected_notice_sha256: format!("{:x}", Sha256::digest(NOTICE)),
            base_locator: "https://example.invalid/llg".into(),
        },
    )
    .unwrap()
    .into_dictionary();
    vec![
        (PackId::LensiskuJbotci, jbotci),
        (PackId::Llg, llg),
        (PackId::LaLojban, la),
        (PackId::Sutysisku, sut),
    ]
}

fn readiness_four_pack_request<'a>(
    sealed: &'a [(PackId, Dictionary, Vec<u8>, Vec<u8>)],
) -> CollectionRequest<'a> {
    CollectionRequest::new(
        sealed
            .iter()
            .map(|(pack_id, _, release, seal)| CollectionInput::SealedBytes {
                pack_id: *pack_id,
                release: Cow::Borrowed(release.as_slice()),
                seal: Cow::Borrowed(seal.as_slice()),
            })
            .collect(),
        CollectionSelection::Packs(vec![
            PackId::LensiskuJbotci,
            PackId::Llg,
            PackId::LaLojban,
            PackId::Sutysisku,
        ]),
        true,
    )
    .unwrap()
}

fn readiness_single_pack_request<'a>(
    sealed: &'a [(PackId, Dictionary, Vec<u8>, Vec<u8>)],
    selected: PackId,
) -> CollectionRequest<'a> {
    let (_, _, release, seal) = sealed
        .iter()
        .find(|(pack, _, _, _)| *pack == selected)
        .expect("selected literal pack exists");
    CollectionRequest::new(
        vec![CollectionInput::SealedBytes {
            pack_id: selected,
            release: Cow::Borrowed(release),
            seal: Cow::Borrowed(seal),
        }],
        CollectionSelection::Packs(vec![selected]),
        true,
    )
    .unwrap()
}

#[test]
fn test_literal_four_pack_remove_reinstall_and_remove_all_preserve_inventory() {
    let models = readiness_four_pack_models("synthetic lifecycle baseline");
    let raw_assertions = [1, 1, 1, 3];
    let lexical_assertions = [1, 1, 1, 2];
    for (index, (_, model)) in models.iter().enumerate() {
        assert_eq!(model.metadata().assertions.len(), raw_assertions[index]);
        assert_eq!(
            model
                .metadata()
                .assertions
                .iter()
                .filter(|row| row.eligibility == Eligibility::Lexical)
                .count(),
            lexical_assertions[index]
        );
    }
    assert_eq!(raw_assertions.iter().sum::<usize>(), 6);
    assert_eq!(lexical_assertions.iter().sum::<usize>(), 5);
    let expected_full = max_family_inventory(
        &models
            .iter()
            .map(|(_, model)| model.clone())
            .collect::<Vec<_>>(),
    );
    assert_eq!(expected_full.len(), 25);
    assert!(expected_full.contains_key("captures"));
    assert!(expected_full.contains_key("frames"));
    assert!(expected_full["frames"].is_empty());

    // The independent multiset comparator must notice both absent and
    // unexpected empty families, along with a changed record body/count.
    let mut missing_empty = expected_full.clone();
    missing_empty.remove("frames");
    assert_ne!(missing_empty, expected_full);
    let mut extra_empty = expected_full.clone();
    extra_empty.insert("unexpected_empty".into(), BTreeMap::new());
    assert_ne!(extra_empty, expected_full);
    let mut changed_count = expected_full.clone();
    let (_, rows) = changed_count
        .iter_mut()
        .find(|(_, rows)| !rows.is_empty())
        .unwrap();
    *rows.values_mut().next().unwrap() += 1;
    assert_ne!(changed_count, expected_full);
    let mut changed_document = models[0].1.clone().into_document();
    changed_document.metadata.entries[0].language = Presence::Missing;
    let changed_model = Dictionary::from_document(changed_document).unwrap();
    assert_ne!(
        family_inventory(&changed_model),
        family_inventory(&models[0].1)
    );

    let complete_typed =
        compose_dictionaries(models.iter().map(|(_, model)| model.clone()).collect()).unwrap();
    let sealed = models
        .into_iter()
        .map(|(pack, model)| {
            let bytes = seal_release(pack, PackPayloadKind::Synthetic, &model).unwrap();
            (pack, model, bytes.release, bytes.seal)
        })
        .collect::<Vec<_>>();
    let root = fresh_root("literal-four-pack-remove-reinstall");
    let installed =
        CollectionCatalogue::install(&root, readiness_four_pack_request(&sealed)).unwrap();
    assert_eq!(installed.local_revision, 1);
    let initial_generation = installed.generation.unwrap();
    let initial_model = vlajmaji_store::DictionaryStore::open(&initial_generation.store_path)
        .unwrap()
        .dictionary()
        .unwrap();
    assert_eq!(initial_model, complete_typed);
    assert_eq!(family_inventory(&initial_model), expected_full);

    let removed_subset = CollectionCatalogue::remove(&root, &[PackId::Llg], 1).unwrap();
    assert_eq!(removed_subset.local_revision, 2);
    let subset_generation = removed_subset.generation.unwrap();
    assert_eq!(subset_generation.members.len(), 3);
    let expected_subset = max_family_inventory(
        &sealed
            .iter()
            .filter(|(pack, _, _, _)| *pack != PackId::Llg)
            .map(|(_, model, _, _)| model.clone())
            .collect::<Vec<_>>(),
    );
    let subset_model = vlajmaji_store::DictionaryStore::open(&subset_generation.store_path)
        .unwrap()
        .dictionary()
        .unwrap();
    assert_eq!(family_inventory(&subset_model), expected_subset);
    assert_eq!(
        subset_generation
            .members
            .iter()
            .map(|member| member.pack_id.as_str())
            .collect::<Vec<_>>(),
        ["lensisku-jbotci", "la-lojban", "sutysisku"]
    );

    let (_, llg_model, llg_release, _) = sealed
        .iter()
        .find(|(pack, _, _, _)| *pack == PackId::Llg)
        .unwrap();
    let llg_sha = format!("{:x}", Sha256::digest(llg_release));
    let reinstalled = CollectionCatalogue::reinstall(
        &root,
        readiness_single_pack_request(&sealed, PackId::Llg),
        &llg_sha,
        2,
    )
    .unwrap();
    assert_eq!(reinstalled.local_revision, 3);
    let restored_generation = reinstalled.generation.unwrap();
    assert_eq!(restored_generation.members.len(), 4);
    let restored_model = vlajmaji_store::DictionaryStore::open(&restored_generation.store_path)
        .unwrap()
        .dictionary()
        .unwrap();
    assert_eq!(family_inventory(&restored_model), expected_full);
    assert_eq!(restored_model, complete_typed);
    assert_eq!(
        fs::read(
            restored_generation
                .store_path
                .parent()
                .unwrap()
                .join("inputs/llg/release.json")
        )
        .unwrap(),
        *llg_release
    );
    assert_eq!(llg_model.metadata().assertions.len(), 1);

    let all = [
        PackId::LensiskuJbotci,
        PackId::Llg,
        PackId::LaLojban,
        PackId::Sutysisku,
    ];
    let removed_all = CollectionCatalogue::remove(&root, &all, 3).unwrap();
    assert_eq!(removed_all.local_revision, 4);
    assert!(removed_all.generation.is_none());
    let catalogue = CollectionCatalogue::open(&root).unwrap();
    assert!(catalogue.generation().unwrap().is_none());
    assert!(catalogue.list().unwrap().generation.is_none());
    for pack in all {
        assert_eq!(
            catalogue.inspect(pack).unwrap().installation_state,
            CollectionInstallationState::Removed
        );
    }
    drop(catalogue);

    let mut revision = 4;
    for pack in all {
        let release = sealed.iter().find(|(id, _, _, _)| *id == pack).unwrap();
        let expected_sha = format!("{:x}", Sha256::digest(&release.2));
        let restored = CollectionCatalogue::reinstall(
            &root,
            readiness_single_pack_request(&sealed, pack),
            &expected_sha,
            revision,
        )
        .unwrap();
        revision += 1;
        assert_eq!(restored.local_revision, revision);
    }
    let final_generation = CollectionCatalogue::open(&root)
        .unwrap()
        .generation()
        .unwrap()
        .unwrap();
    assert_eq!(final_generation.members.len(), 4);
    let final_model = vlajmaji_store::DictionaryStore::open(&final_generation.store_path)
        .unwrap()
        .dictionary()
        .unwrap();
    assert_eq!(family_inventory(&final_model), expected_full);
    assert_eq!(final_model, complete_typed);
    let _ = fs::remove_dir_all(root);
}

fn request<'a>(release: &'a [u8], seal: &'a [u8]) -> CollectionRequest<'a> {
    request_for(PackId::Llg, release, seal)
}

fn request_for<'a>(pack_id: PackId, release: &'a [u8], seal: &'a [u8]) -> CollectionRequest<'a> {
    CollectionRequest::new(
        vec![CollectionInput::SealedBytes {
            pack_id,
            release: Cow::Borrowed(release),
            seal: Cow::Borrowed(seal),
        }],
        CollectionSelection::Packs(vec![pack_id]),
        true,
    )
    .unwrap()
}

fn two_pack_request<'a>(
    llg_release: &'a [u8],
    llg_seal: &'a [u8],
    jbotci_release: &'a [u8],
    jbotci_seal: &'a [u8],
) -> CollectionRequest<'a> {
    CollectionRequest::new(
        vec![
            CollectionInput::SealedBytes {
                pack_id: PackId::Llg,
                release: Cow::Borrowed(llg_release),
                seal: Cow::Borrowed(llg_seal),
            },
            CollectionInput::SealedBytes {
                pack_id: PackId::LensiskuJbotci,
                release: Cow::Borrowed(jbotci_release),
                seal: Cow::Borrowed(jbotci_seal),
            },
        ],
        CollectionSelection::Packs(vec![PackId::Llg, PackId::LensiskuJbotci]),
        true,
    )
    .unwrap()
}

fn lifecycle_event(root: &std::path::Path, operation: &str) -> serde_json::Value {
    let attempt = CollectionCatalogue::reconcile(root)
        .unwrap()
        .attempts
        .into_iter()
        .find(|attempt| attempt.operation == operation)
        .expect("lifecycle attempt is recorded");
    serde_json::from_str(&attempt.detail).unwrap()
}

#[test]
fn test_remove_reinstall_retained_release_and_revision() {
    let root = fresh_root("roundtrip");
    let (release, seal, release_sha256) = sealed_llg();
    let installed = CollectionCatalogue::install(&root, request(&release, &seal)).unwrap();
    let first_generation = installed.generation.unwrap();

    let removed = CollectionCatalogue::remove(&root, &[PackId::Llg], 1).unwrap();
    assert_eq!(removed.outcome, "removed");
    assert_eq!(removed.local_revision, 2);
    assert!(removed.generation.is_none());
    let remove_event = lifecycle_event(&root, "remove");
    assert_eq!(remove_event["local_revision"], 2);
    assert_eq!(remove_event["generation_id"], serde_json::Value::Null);
    assert_eq!(remove_event["packs"][0]["pack_id"], "llg");
    assert_eq!(remove_event["packs"][0]["prior_state"], "installed");
    assert_eq!(remove_event["packs"][0]["release_sha256"], release_sha256);

    let catalogue = CollectionCatalogue::open(&root).unwrap();
    let inspection = catalogue.inspect(PackId::Llg).unwrap();
    assert_eq!(
        inspection.installation_state,
        CollectionInstallationState::Removed
    );
    assert_eq!(inspection.installed_history.len(), 1);
    assert_eq!(
        inspection.installed_history[0].generation_id,
        first_generation.generation_id
    );
    drop(catalogue);

    let implicit_restore =
        CollectionCatalogue::install(&root, request(&release, &seal)).unwrap_err();
    assert_eq!(
        implicit_restore.category(),
        CollectionErrorCategory::ReinstallRequired
    );

    // The caller's exact normalized bytes must be sufficient to restore the
    // pack even when a retained generation's owned copy has gone missing.
    let first_stage = first_generation.store_path.parent().unwrap();
    fs::remove_file(first_stage.join("inputs/llg/release.json")).unwrap();
    fs::remove_file(first_stage.join("inputs/llg/seal.json")).unwrap();

    let reinstalled =
        CollectionCatalogue::reinstall(&root, request(&release, &seal), &release_sha256, 2)
            .unwrap();
    assert_eq!(reinstalled.outcome, "reinstalled");
    assert_eq!(reinstalled.local_revision, 3);
    let restored = reinstalled.generation.unwrap();
    assert_ne!(restored.generation_id, first_generation.generation_id);
    assert_eq!(restored.content_id, first_generation.content_id);
    let restored_stage = restored.store_path.parent().unwrap();
    assert_ne!(restored_stage, first_stage);
    assert_eq!(
        fs::read(restored_stage.join("inputs/llg/release.json")).unwrap(),
        release
    );
    let attempts_before_idempotent = CollectionCatalogue::reconcile(&root)
        .unwrap()
        .attempts
        .len();
    let idempotent =
        CollectionCatalogue::reinstall(&root, request(&release, &seal), &release_sha256, 3)
            .unwrap();
    assert_eq!(idempotent.outcome, "idempotent");
    assert_eq!(idempotent.local_revision, 3);
    assert_eq!(
        idempotent
            .generation
            .as_ref()
            .map(|item| item.generation_id.as_str()),
        Some(restored.generation_id.as_str())
    );
    assert_eq!(
        CollectionCatalogue::reconcile(&root)
            .unwrap()
            .attempts
            .len(),
        attempts_before_idempotent
    );
    let reinstall_event = lifecycle_event(&root, "reinstall");
    assert_eq!(reinstall_event["local_revision"], 3);
    assert_eq!(reinstall_event["generation_id"], restored.generation_id);
    assert_eq!(reinstall_event["packs"][0]["pack_id"], "llg");
    assert_eq!(reinstall_event["packs"][0]["prior_state"], "removed");
    assert_eq!(
        reinstall_event["packs"][0]["release_sha256"],
        release_sha256
    );

    let catalogue = CollectionCatalogue::open(&root).unwrap();
    assert_eq!(
        catalogue.inspect(PackId::Llg).unwrap().installation_state,
        CollectionInstallationState::Installed
    );
    let reconcile = CollectionCatalogue::reconcile(&root).unwrap();
    assert_eq!(reconcile.local_revision, 3);
    assert_eq!(
        reconcile.active_generation.as_deref(),
        Some(restored.generation_id.as_str())
    );
    assert_eq!(reconcile.verification_scope, "minimum_header_only");
    assert_eq!(reconcile.retained_generation_count, 2);
    drop(catalogue);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn test_reinstall_digest_mismatch_preserves_removed_state() {
    let root = fresh_root("digest-mismatch");
    let (release, seal, registered_sha256) = sealed_llg();
    let (other_release, other_seal, other_sha256) = sealed_llg_revision(1);
    CollectionCatalogue::install(&root, request(&release, &seal)).unwrap();
    CollectionCatalogue::remove(&root, &[PackId::Llg], 1).unwrap();
    let unknown = CollectionCatalogue::reinstall(
        &root,
        request(&other_release, &other_seal),
        &other_sha256,
        2,
    )
    .unwrap_err();
    assert_eq!(
        unknown.category(),
        CollectionErrorCategory::UnknownRegisteredRelease
    );
    let unknown_report = unknown.report("reinstall");
    assert_eq!(unknown_report.pack_id.as_deref(), Some("llg"));
    assert_eq!(unknown_report.prior_local_revision, Some(2));
    assert!(unknown_report.attempt_id.is_none());
    assert!(unknown_report.proposed_generation.is_none());
    assert!(unknown_report.observed_generation.is_none());
    assert_eq!(unknown_report.inputs.len(), 1);
    assert_eq!(unknown_report.inputs[0].disposition, "validated");
    assert_eq!(
        unknown_report.inputs[0].release_sha256.as_deref(),
        Some(other_sha256.as_str())
    );
    let error = CollectionCatalogue::reinstall(
        &root,
        request(&other_release, &other_seal),
        &registered_sha256,
        2,
    )
    .unwrap_err();
    assert_eq!(error.category(), CollectionErrorCategory::IdentityConflict);
    let stale = CollectionCatalogue::reinstall(
        &root,
        request(&other_release, &other_seal),
        &registered_sha256,
        1,
    )
    .unwrap_err();
    assert_eq!(stale.category(), CollectionErrorCategory::RevisionConflict);
    let catalogue = CollectionCatalogue::open(&root).unwrap();
    assert_eq!(catalogue.local_revision().unwrap(), 2);
    assert_eq!(
        catalogue.inspect(PackId::Llg).unwrap().installation_state,
        CollectionInstallationState::Removed
    );
    drop(catalogue);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn test_reinstall_open_and_begin_busy_keep_validated_input_context() {
    let root = fresh_root("reinstall-open-busy-context");
    let (release, seal, digest) = sealed_pack(PackId::Llg);
    let (jb_release, jb_seal, _) = sealed_pack(PackId::LensiskuJbotci);
    let missing = CollectionCatalogue::reinstall(
        &root,
        request_for(PackId::Llg, &release, &seal),
        &digest,
        1,
    )
    .unwrap_err();
    assert_eq!(
        missing.category(),
        CollectionErrorCategory::MissingCatalogue
    );
    let missing_report = missing.report("reinstall");
    assert_eq!(missing_report.activation.as_deref(), Some("not_activated"));
    assert!(missing_report.prior_local_revision.is_none());
    assert_eq!(missing_report.inputs.len(), 1);
    assert_eq!(missing_report.inputs[0].disposition, "validated");

    let installed = CollectionCatalogue::install(
        &root,
        two_pack_request(&release, &seal, &jb_release, &jb_seal),
    )
    .unwrap();
    let original_generation = installed.generation.unwrap().generation_id;
    let writer = rusqlite::Connection::open(root.join("catalogue.sqlite3")).unwrap();
    writer.execute_batch("BEGIN IMMEDIATE").unwrap();
    let begin_busy = CollectionCatalogue::reinstall(
        &root,
        request_for(PackId::Llg, &release, &seal),
        &digest,
        1,
    )
    .unwrap_err();
    assert_eq!(begin_busy.category(), CollectionErrorCategory::Busy);
    let begin_report = begin_busy.report("reinstall");
    assert_eq!(begin_report.activation.as_deref(), Some("not_activated"));
    assert!(begin_report.prior_local_revision.is_none());
    assert!(begin_report.observed_generation.is_none());
    assert!(begin_report.attempt_id.is_none());
    assert!(begin_report.proposed_generation.is_none());
    assert_eq!(begin_report.inputs.len(), 1);
    assert_eq!(begin_report.inputs[0].disposition, "validated");
    writer.execute_batch("ROLLBACK").unwrap();
    drop(writer);

    let held_reader = rusqlite::Connection::open(root.join("catalogue.sqlite3")).unwrap();
    held_reader.execute_batch("BEGIN DEFERRED").unwrap();
    let _: String = held_reader
        .query_row(
            "SELECT value FROM catalogue_meta_v2 WHERE key='local_revision'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    let commit_busy = CollectionCatalogue::remove(&root, &[PackId::Llg], 1).unwrap_err();
    assert_eq!(commit_busy.category(), CollectionErrorCategory::Busy);
    let commit_report = commit_busy.report("remove");
    assert_eq!(commit_report.activation.as_deref(), Some("not_activated"));
    assert_eq!(commit_report.prior_local_revision, Some(1));
    assert_eq!(
        commit_report.observed_generation.as_deref(),
        Some(original_generation.as_str())
    );
    assert!(commit_report.attempt_id.is_some());
    assert!(commit_report.proposed_generation.is_some());
    assert!(commit_report.inputs.is_empty());
    held_reader.execute_batch("ROLLBACK").unwrap();
    drop(held_reader);

    let retry = CollectionCatalogue::remove(&root, &[PackId::Llg], 1).unwrap();
    assert_eq!(retry.outcome, "removed");
    assert_eq!(retry.local_revision, 2);
    assert_ne!(retry.generation.unwrap().generation_id, original_generation);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn test_remove_one_pack_publishes_survivor_generation_and_reinstall_restores_member() {
    let root = fresh_root("survivor");
    let (llg_release, llg_seal, llg_sha) = sealed_pack(PackId::Llg);
    let (jb_release, jb_seal, _) = sealed_pack(PackId::LensiskuJbotci);
    let installed = CollectionCatalogue::install(
        &root,
        two_pack_request(&llg_release, &llg_seal, &jb_release, &jb_seal),
    )
    .unwrap();
    let first_generation = installed.generation.unwrap().generation_id;
    let removed = CollectionCatalogue::remove(&root, &[PackId::Llg], 1).unwrap();
    assert_eq!(removed.local_revision, 2);
    let survivor = removed.generation.unwrap();
    assert_ne!(survivor.generation_id, first_generation);
    let remove_event = lifecycle_event(&root, "remove");
    assert_eq!(remove_event["generation_id"], survivor.generation_id);
    assert_eq!(remove_event["packs"][0]["prior_state"], "installed");
    assert_eq!(remove_event["packs"][0]["release_sha256"], llg_sha);
    assert_eq!(
        survivor
            .members
            .iter()
            .map(|member| member.pack_id.as_str())
            .collect::<Vec<_>>(),
        vec!["lensisku-jbotci"]
    );
    let catalogue = CollectionCatalogue::open(&root).unwrap();
    assert_eq!(
        catalogue.inspect(PackId::Llg).unwrap().installation_state,
        CollectionInstallationState::Removed
    );
    assert_eq!(
        catalogue
            .inspect(PackId::LensiskuJbotci)
            .unwrap()
            .installation_state,
        CollectionInstallationState::Installed
    );
    drop(catalogue);

    let restored =
        CollectionCatalogue::reinstall(&root, request(&llg_release, &llg_seal), &llg_sha, 2)
            .unwrap();
    assert_eq!(restored.local_revision, 3);
    assert_eq!(restored.generation.unwrap().members.len(), 2);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn test_remove_registered_only_pack_commits_without_creating_generation() {
    let root = fresh_root("registered-only");
    let (release, seal, _) = sealed_llg();
    CollectionCatalogue::register(&root, request(&release, &seal)).unwrap();
    let before = CollectionCatalogue::open(&root).unwrap();
    assert!(before.generation().unwrap().is_none());
    drop(before);

    let removed = CollectionCatalogue::remove(&root, &[PackId::Llg], 1).unwrap();
    assert_eq!(removed.local_revision, 2);
    assert!(removed.generation.is_none());
    let remove_event = lifecycle_event(&root, "remove");
    assert_eq!(remove_event["generation_id"], serde_json::Value::Null);
    assert_eq!(remove_event["packs"][0]["prior_state"], "registered_only");
    assert_eq!(
        remove_event["packs"][0]["release_sha256"],
        serde_json::Value::Null
    );
    assert_eq!(
        CollectionCatalogue::open(&root)
            .unwrap()
            .inspect(PackId::Llg)
            .unwrap()
            .installation_state,
        CollectionInstallationState::Removed
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn test_remove_registered_only_pack_keeps_active_generation() {
    let root = fresh_root("registered-only-keep");
    let (llg_release, llg_seal, _) = sealed_pack(PackId::Llg);
    let (la_release, la_seal, _) = sealed_pack(PackId::LaLojban);
    let installed = CollectionCatalogue::install(&root, request(&llg_release, &llg_seal)).unwrap();
    let active_generation = installed.generation.unwrap().generation_id;
    CollectionCatalogue::register(&root, request_for(PackId::LaLojban, &la_release, &la_seal))
        .unwrap();

    let removed = CollectionCatalogue::remove(&root, &[PackId::LaLojban], 2).unwrap();
    assert_eq!(removed.local_revision, 3);
    assert_eq!(
        removed
            .generation
            .as_ref()
            .map(|value| value.generation_id.as_str()),
        Some(active_generation.as_str())
    );
    let event = lifecycle_event(&root, "remove");
    assert_eq!(event["packs"][0]["prior_state"], "registered_only");
    assert_eq!(event["packs"][0]["release_sha256"], serde_json::Value::Null);
    assert_eq!(event["generation_id"], active_generation);

    let catalogue = CollectionCatalogue::open(&root).unwrap();
    assert_eq!(
        catalogue.inspect(PackId::Llg).unwrap().installation_state,
        CollectionInstallationState::Installed
    );
    assert_eq!(
        catalogue
            .inspect(PackId::LaLojban)
            .unwrap()
            .installation_state,
        CollectionInstallationState::Removed
    );
    drop(catalogue);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn test_reconcile_committed_stage_markers_and_unknown_operation() {
    let root = fresh_root("committed-stage-marker-operations");
    let (release, seal, release_sha256) = sealed_llg();
    let installed = CollectionCatalogue::install(&root, request(&release, &seal)).unwrap();
    assert_eq!(installed.local_revision, 1);
    let removed = CollectionCatalogue::remove(&root, &[PackId::Llg], 1).unwrap();
    assert_eq!(removed.local_revision, 2);
    assert!(removed.generation.is_none());
    let reinstalled =
        CollectionCatalogue::reinstall(&root, request(&release, &seal), &release_sha256, 2)
            .unwrap();
    assert_eq!(reinstalled.local_revision, 3);
    let active_generation = reinstalled.generation.unwrap().generation_id;

    let clean = CollectionCatalogue::reconcile(&root).unwrap();
    assert_eq!(clean.local_revision, 3);
    assert_eq!(
        clean.active_generation.as_deref(),
        Some(active_generation.as_str())
    );
    assert!(clean.staged_attempts.is_empty());
    let committed_install = clean
        .attempts
        .iter()
        .find(|attempt| attempt.operation == "install")
        .expect("install attempt is retained")
        .attempt_id
        .clone();
    assert!(
        clean
            .attempts
            .iter()
            .any(|attempt| { attempt.operation == "remove" && attempt.status == "committed" })
    );
    assert!(
        clean
            .attempts
            .iter()
            .any(|attempt| { attempt.operation == "reinstall" && attempt.status == "committed" })
    );

    let generation_root = root.join("generations");
    let orphan_id = "ffffffffffffffffffffffffffffffff";
    assert!(
        !clean
            .attempts
            .iter()
            .any(|attempt| attempt.attempt_id == orphan_id)
    );
    let orphan_dir = generation_root.join("valid-uncommitted-remove");
    fs::create_dir(&orphan_dir).unwrap();
    let orphan_marker = serde_json::to_vec(&serde_json::json!({
        "version": 1,
        "attempt_id": orphan_id,
        "operation": "remove"
    }))
    .unwrap();
    fs::write(orphan_dir.join("attempt.json"), &orphan_marker).unwrap();

    let unknown_dir = generation_root.join("unknown-operation-committed-id");
    fs::create_dir(&unknown_dir).unwrap();
    let unknown_marker = serde_json::to_vec(&serde_json::json!({
        "version": 1,
        "attempt_id": committed_install,
        "operation": "future-operation"
    }))
    .unwrap();
    fs::write(unknown_dir.join("attempt.json"), &unknown_marker).unwrap();

    let database_path = root.join("catalogue.sqlite3");
    let database_before = fs::read(&database_path).unwrap();
    let report = CollectionCatalogue::reconcile(&root).unwrap();
    let mut unknown_name_hex = String::new();
    for byte in b"unknown-operation-committed-id" {
        use std::fmt::Write as _;
        write!(unknown_name_hex, "{byte:02x}").unwrap();
    }
    let mut expected_staged = vec![
        orphan_id.to_owned(),
        format!("unreadable:{unknown_name_hex}"),
    ];
    expected_staged.sort();
    let mut observed_staged = report.staged_attempts.clone();
    observed_staged.sort();
    assert_eq!(observed_staged, expected_staged);
    assert!(report.attention_required);
    assert_eq!(report.local_revision, clean.local_revision);
    assert_eq!(report.active_generation, clean.active_generation);
    assert_eq!(report.attempts, clean.attempts);
    assert_eq!(fs::read(&database_path).unwrap(), database_before);
    assert_eq!(
        fs::read(orphan_dir.join("attempt.json")).unwrap(),
        orphan_marker
    );
    assert_eq!(
        fs::read(unknown_dir.join("attempt.json")).unwrap(),
        unknown_marker
    );
    fs::remove_dir_all(root).unwrap();
}
