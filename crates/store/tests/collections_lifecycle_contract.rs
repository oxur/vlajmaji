//! Public collection lifecycle and schema migration contract tests.

use std::{
    borrow::Cow,
    collections::BTreeMap,
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use rusqlite::Connection;
use sha2::{Digest, Sha256};
use vlajmaji_lexicon::{
    Dictionary, DictionaryDocument, Eligibility, JbotciImportOptions, LaLojbanImportOptions,
    LlgImportOptions, PackId, PackPayloadKind, Presence, SourceRegistry, SutysiskuImportOptions,
    compose_dictionaries, import_jbotci_export, import_la_lojban_export, import_llg_lists,
    import_sutysisku_export, seal_release,
};
use vlajmaji_store::{
    CollectionCatalogue, CollectionErrorCategory, CollectionInput, CollectionRequest,
    CollectionSelection,
};

fn fresh_root(label: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "vlajmaji-s04-{label}-{}-{stamp}",
        std::process::id()
    ))
}

fn sealed(pack_id: PackId) -> (Vec<u8>, Vec<u8>) {
    let dictionary = Dictionary::from_document(DictionaryDocument::default()).unwrap();
    let release = seal_release(pack_id, PackPayloadKind::Synthetic, &dictionary).unwrap();
    (release.release, release.seal)
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

type FamilyInventory = BTreeMap<String, BTreeMap<String, usize>>;

fn row_inventory<T: serde::Serialize>(rows: &[T]) -> BTreeMap<String, usize> {
    let mut inventory = BTreeMap::new();
    for row in rows {
        let bytes = serde_json::to_vec(row).unwrap();
        *inventory.entry(sha256(&bytes)).or_insert(0) += 1;
    }
    inventory
}

/// Hashes complete typed records in every persisted family, including empty
/// families, without using the production dictionary composer.
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
    const LLG_EMPTY: &[u8] = b"";
    const LLG_NOTICE: &[u8] = b"synthetic notice\n";
    let lensisku = format!(
        r#"[{{"word":"farvi","word_type":"gismu","definition":"{jbotci_definition}","definition_id":333,"score":1.2500,"user":{{"username":"shared"}}}}]"#
    );
    let metadata = format!(
        "language_tag = \"en\"\nlanguage_realname = \"English\"\nsource_language_tag = \"jbo\"\nformat = \"json\"\npositive_scores_only = false\nfilename = \"dictionary-en.json\"\nmetadata_url = \"https://example.invalid/languages\"\ndownload_url = \"https://example.invalid/export\"\nlensisku_created_at = \"2026-10-04T03:07:08Z\"\nsha256 = \"{}\"\ndefinition_count = 1\nentry_count = 1\n",
        sha256(lensisku.as_bytes()),
    );
    let jbotci = import_jbotci_export(
        lensisku.as_bytes(),
        metadata.as_bytes(),
        &JbotciImportOptions {
            revision: "1111111111111111111111111111111111111111".into(),
            expected_raw_sha256: sha256(lensisku.as_bytes()),
            expected_metadata_sha256: sha256(metadata.as_bytes()),
        },
    )
    .unwrap()
    .into_dictionary();
    let la = import_la_lojban_export(
        lensisku.as_bytes(),
        &LaLojbanImportOptions {
            revision: "2222222222222222222222222222222222222222".into(),
            expected_raw_sha256: sha256(lensisku.as_bytes()),
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
            expected_raw_sha256: sha256(SUTYSISKU),
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
        LLG_EMPTY,
        LLG_EMPTY,
        LLG_NOTICE,
        &LlgImportOptions {
            expected_gismu_sha256: sha256(&gismu),
            expected_cmavo_sha256: sha256(LLG_EMPTY),
            expected_rafsi_sha256: sha256(LLG_EMPTY),
            expected_notice_sha256: sha256(LLG_NOTICE),
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

fn sealed_input<'a>(pack_id: PackId, release: &'a [u8], seal: &'a [u8]) -> CollectionInput<'a> {
    CollectionInput::SealedBytes {
        pack_id,
        release: Cow::Borrowed(release),
        seal: Cow::Borrowed(seal),
    }
}

#[test]
fn test_catalogue_migration_v1_preserves_generation_and_fences_legacy_names() {
    let root = fresh_root("migrate-v1");
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
    let installed = CollectionCatalogue::install(&root, request()).unwrap();
    let prior_generation = installed.generation.unwrap().generation_id;
    // Metadata-only registration advances catalogue revision without creating a
    // newer generation. Migration must preserve the latest generation pointer.
    let (la_release, la_seal) = sealed(PackId::LaLojban);
    CollectionCatalogue::register(
        &root,
        CollectionRequest::new(
            vec![sealed_input(PackId::LaLojban, &la_release, &la_seal)],
            CollectionSelection::Packs(vec![PackId::LaLojban]),
            true,
        )
        .unwrap(),
    )
    .unwrap();
    let db = root.join("catalogue.sqlite3");

    // Turn this owned v2 fixture into the exact old physical-name topology.
    // The test exercises public v1 reads and migration, not a captured old binary.
    let connection = Connection::open(&db).unwrap();
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

    let v1_bytes = fs::read(&db).unwrap();

    // Keep the active LLG registration and its location row in both clones.
    // The other registration is metadata-only; retaining it would make a
    // malformed clone for reasons unrelated to FK grouping.
    let llg_only_v1_clone = |label: &str| {
        let clone_root = fresh_root(label);
        fs::create_dir(&clone_root).unwrap();
        let clone_db = clone_root.join("catalogue.sqlite3");
        fs::copy(&db, &clone_db).unwrap();
        let clone_connection = Connection::open(&clone_db).unwrap();
        clone_connection
            .execute_batch(
                "BEGIN IMMEDIATE;
                 DELETE FROM registration_locations WHERE pack_id <> 'llg';
                 DELETE FROM registrations WHERE pack_id <> 'llg';
                 COMMIT;",
            )
            .unwrap();
        drop(clone_connection);
        clone_root
    };

    // The same pruned clone with the original composite constraint is valid
    // and migrates, proving that the head/member/registration state itself is
    // not the reason the negative clone below is rejected.
    let valid_root = llg_only_v1_clone("migrate-valid-fk");
    let valid_db = valid_root.join("catalogue.sqlite3");
    let valid_revision = CollectionCatalogue::open(&valid_root)
        .unwrap()
        .local_revision()
        .unwrap();
    let valid_migration = CollectionCatalogue::migrate(&valid_root, valid_revision).unwrap();
    assert_eq!(valid_migration.schema_version, 2);
    assert_eq!(valid_migration.local_revision, valid_revision + 1);
    assert_eq!(
        valid_migration.active_generation.as_deref(),
        Some(prior_generation.as_str())
    );
    assert_eq!(
        CollectionCatalogue::open(&valid_root)
            .unwrap()
            .generation()
            .unwrap()
            .unwrap()
            .generation_id,
        prior_generation
    );
    let pointer_connection = Connection::open(&valid_db).unwrap();
    pointer_connection
        .execute(
            "UPDATE catalogue_meta_v2 SET value='' WHERE key='active_generation_id'",
            [],
        )
        .unwrap();
    drop(pointer_connection);
    assert!(
        CollectionCatalogue::open(&valid_root)
            .unwrap()
            .generation()
            .unwrap()
            .is_none()
    );
    let history_connection = Connection::open(&valid_db).unwrap();
    let retained_history: i64 = history_connection
        .query_row("SELECT count(*) FROM generations_v2", [], |row| row.get(0))
        .unwrap();
    assert_eq!(
        retained_history, 1,
        "null active pointer must not select history"
    );
    let pointer_connection = Connection::open(&valid_db).unwrap();
    pointer_connection
        .execute(
            "UPDATE catalogue_meta_v2 SET value='missing-generation' WHERE key='active_generation_id'",
            [],
        )
        .unwrap();
    drop(pointer_connection);
    let missing_pointer_bytes = fs::read(&valid_db).unwrap();
    let missing_pointer = CollectionCatalogue::open(&valid_root)
        .unwrap()
        .generation()
        .unwrap_err();
    assert_eq!(
        missing_pointer.category(),
        vlajmaji_store::CollectionErrorCategory::UnsupportedVersion
    );
    assert_eq!(fs::read(&valid_db).unwrap(), missing_pointer_bytes);
    let restore_pointer = Connection::open(&valid_db).unwrap();
    restore_pointer
        .execute(
            "UPDATE catalogue_meta_v2 SET value=?1 WHERE key='active_generation_id'",
            [&prior_generation],
        )
        .unwrap();
    drop(restore_pointer);
    let restored_pointer_bytes = fs::read(&valid_db).unwrap();
    assert_eq!(
        CollectionCatalogue::open(&valid_root)
            .unwrap()
            .generation()
            .unwrap()
            .unwrap()
            .generation_id,
        prior_generation
    );
    assert_eq!(fs::read(&valid_db).unwrap(), restored_pointer_bytes);
    let _ = fs::remove_dir_all(valid_root);

    let bad_heads_root = llg_only_v1_clone("migrate-bad-heads");
    let bad_heads_db = bad_heads_root.join("catalogue.sqlite3");
    let bad_heads_connection = Connection::open(&bad_heads_db).unwrap();
    bad_heads_connection
        .execute("DELETE FROM installed_heads", [])
        .unwrap();
    drop(bad_heads_connection);
    let bad_heads_bytes = fs::read(&bad_heads_db).unwrap();
    let bad_heads_revision = CollectionCatalogue::open(&bad_heads_root)
        .unwrap()
        .local_revision()
        .unwrap();
    let bad_heads = CollectionCatalogue::migrate(&bad_heads_root, bad_heads_revision).unwrap_err();
    assert_eq!(
        bad_heads.category(),
        vlajmaji_store::CollectionErrorCategory::UnsupportedVersion
    );
    assert_eq!(fs::read(&bad_heads_db).unwrap(), bad_heads_bytes);
    let _ = fs::remove_dir_all(bad_heads_root);

    let bad_member_root = llg_only_v1_clone("migrate-bad-members");
    let bad_member_db = bad_member_root.join("catalogue.sqlite3");
    let bad_member_revision = CollectionCatalogue::open(&bad_member_root)
        .unwrap()
        .local_revision()
        .unwrap();
    let bad_member_connection = Connection::open(&bad_member_db).unwrap();
    bad_member_connection
        .execute("UPDATE generations SET members_json=x'7b'", [])
        .unwrap();
    drop(bad_member_connection);
    let bad_member_bytes = fs::read(&bad_member_db).unwrap();
    let bad_member =
        CollectionCatalogue::migrate(&bad_member_root, bad_member_revision).unwrap_err();
    assert_eq!(
        bad_member.category(),
        vlajmaji_store::CollectionErrorCategory::UnsupportedVersion
    );
    assert_eq!(fs::read(&bad_member_db).unwrap(), bad_member_bytes);
    let _ = fs::remove_dir_all(bad_member_root);

    // These valid-JSON metadata-only corruptions must all fail before ALTER;
    // the valid unmodified LLG clone above is their same-state counterpart.
    let mut accepted_corruptions = Vec::new();
    for field in [
        "pack_id",
        "release_sha256",
        "version",
        "dictionary_version",
        "registry_version",
    ] {
        let root = llg_only_v1_clone(&format!("migrate-bad-seal-{field}"));
        let path = root.join("catalogue.sqlite3");
        let connection = Connection::open(&path).unwrap();
        let seal_bytes: Vec<u8> = connection
            .query_row(
                "SELECT seal_json FROM registrations WHERE pack_id='llg'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let mut seal: serde_json::Value = serde_json::from_slice(&seal_bytes).unwrap();
        seal[field] = match field {
            "pack_id" => serde_json::Value::String("la-lojban".into()),
            "release_sha256" => serde_json::Value::String("0".repeat(64)),
            _ => serde_json::Value::from(999),
        };
        connection
            .execute(
                "UPDATE registrations SET seal_json=?1 WHERE pack_id='llg'",
                [serde_json::to_vec(&seal).unwrap()],
            )
            .unwrap();
        drop(connection);
        let before = fs::read(&path).unwrap();
        let revision = CollectionCatalogue::open(&root)
            .unwrap()
            .local_revision()
            .unwrap();
        match CollectionCatalogue::migrate(&root, revision) {
            Ok(_) => accepted_corruptions.push(format!("seal.{field}")),
            Err(error) => {
                assert!(
                    matches!(
                        error.category(),
                        vlajmaji_store::CollectionErrorCategory::UnsupportedVersion
                            | vlajmaji_store::CollectionErrorCategory::VerificationFailed
                    ),
                    "seal.{field} returned an unrelated category: {error:?}"
                );
                assert_eq!(fs::read(&path).unwrap(), before, "seal.{field}");
            }
        }
        let _ = fs::remove_dir_all(root);
    }

    let missing_registration_root = llg_only_v1_clone("migrate-missing-registration");
    let missing_registration_path = missing_registration_root.join("catalogue.sqlite3");
    let missing_registration_connection = Connection::open(&missing_registration_path).unwrap();
    missing_registration_connection
        .execute_batch(
            "DELETE FROM registration_locations WHERE pack_id='llg';
             DELETE FROM registrations WHERE pack_id='llg';",
        )
        .unwrap();
    drop(missing_registration_connection);
    let missing_registration_before = fs::read(&missing_registration_path).unwrap();
    let missing_registration_revision = CollectionCatalogue::open(&missing_registration_root)
        .unwrap()
        .local_revision()
        .unwrap();
    match CollectionCatalogue::migrate(&missing_registration_root, missing_registration_revision) {
        Ok(_) => accepted_corruptions.push("active member without registration".into()),
        Err(error) => {
            assert_eq!(
                error.category(),
                vlajmaji_store::CollectionErrorCategory::UnsupportedVersion
            );
            assert_eq!(
                fs::read(&missing_registration_path).unwrap(),
                missing_registration_before
            );
        }
    }
    let _ = fs::remove_dir_all(missing_registration_root);

    let duplicate_member_root = llg_only_v1_clone("migrate-duplicate-member");
    let duplicate_member_path = duplicate_member_root.join("catalogue.sqlite3");
    let duplicate_member_connection = Connection::open(&duplicate_member_path).unwrap();
    let (generation_id, members_bytes): (String, Vec<u8>) = duplicate_member_connection
        .query_row(
            "SELECT generation_id,members_json FROM generations ORDER BY local_revision DESC LIMIT 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    let mut members: Vec<serde_json::Value> = serde_json::from_slice(&members_bytes).unwrap();
    members.push(members[0].clone());
    duplicate_member_connection
        .execute(
            "UPDATE generations SET members_json=?1 WHERE generation_id=?2",
            rusqlite::params![serde_json::to_vec(&members).unwrap(), generation_id],
        )
        .unwrap();
    drop(duplicate_member_connection);
    let duplicate_member_before = fs::read(&duplicate_member_path).unwrap();
    let duplicate_member_revision = CollectionCatalogue::open(&duplicate_member_root)
        .unwrap()
        .local_revision()
        .unwrap();
    match CollectionCatalogue::migrate(&duplicate_member_root, duplicate_member_revision) {
        Ok(_) => accepted_corruptions.push("duplicate active member pack".into()),
        Err(error) => {
            assert_eq!(
                error.category(),
                vlajmaji_store::CollectionErrorCategory::UnsupportedVersion
            );
            assert_eq!(
                fs::read(&duplicate_member_path).unwrap(),
                duplicate_member_before
            );
        }
    }
    let _ = fs::remove_dir_all(duplicate_member_root);
    assert!(
        accepted_corruptions.is_empty(),
        "migration accepted inconsistent metadata: {accepted_corruptions:?}"
    );

    let bad_version_root = fresh_root("migrate-bad-version");
    fs::create_dir(&bad_version_root).unwrap();
    let bad_version_db = bad_version_root.join("catalogue.sqlite3");
    fs::copy(&db, &bad_version_db).unwrap();
    let bad_version_connection = Connection::open(&bad_version_db).unwrap();
    bad_version_connection
        .execute(
            "UPDATE catalogue_meta SET value='999' WHERE key='schema_version'",
            [],
        )
        .unwrap();
    drop(bad_version_connection);
    let bad_version_bytes = fs::read(&bad_version_db).unwrap();
    let bad_version = CollectionCatalogue::migrate(&bad_version_root, 0).unwrap_err();
    assert_eq!(
        bad_version.category(),
        vlajmaji_store::CollectionErrorCategory::UnsupportedVersion
    );
    assert_eq!(fs::read(&bad_version_db).unwrap(), bad_version_bytes);
    let _ = fs::remove_dir_all(bad_version_root);

    // Preserve the projected FK rows while splitting the registration identity
    // into two independent constraints. Migration must reject this shape even
    // though the flattened target/from/to tuples are unchanged.
    let split_root = llg_only_v1_clone("migrate-split-fk");
    let split_db = split_root.join("catalogue.sqlite3");
    let split_connection = Connection::open(&split_db).unwrap();
    split_connection
        .execute_batch(
            "BEGIN IMMEDIATE;
             CREATE UNIQUE INDEX registrations_pack_id_uq ON registrations(pack_id);
             CREATE UNIQUE INDEX registrations_release_sha256_uq
                 ON registrations(release_sha256);
             ALTER TABLE registration_locations RENAME TO registration_locations_composite;
             CREATE TABLE registration_locations(
                 pack_id TEXT NOT NULL,
                 release_sha256 TEXT NOT NULL,
                 input_paths_json BLOB NOT NULL,
                 PRIMARY KEY(pack_id, release_sha256),
                 FOREIGN KEY(pack_id) REFERENCES registrations(pack_id),
                 FOREIGN KEY(release_sha256) REFERENCES registrations(release_sha256)
             );
             INSERT INTO registration_locations(pack_id,release_sha256,input_paths_json)
                 SELECT pack_id,release_sha256,input_paths_json
                 FROM registration_locations_composite;
             DROP TABLE registration_locations_composite;
             COMMIT;",
        )
        .unwrap();
    drop(split_connection);
    let split_bytes = fs::read(&split_db).unwrap();
    let split_revision = CollectionCatalogue::open(&split_root)
        .unwrap()
        .local_revision()
        .unwrap();
    let split_error = CollectionCatalogue::migrate(&split_root, split_revision).unwrap_err();
    assert_eq!(
        split_error.category(),
        vlajmaji_store::CollectionErrorCategory::UnsupportedVersion
    );
    assert_eq!(fs::read(&split_db).unwrap(), split_bytes);
    assert_eq!(
        CollectionCatalogue::open(&split_root)
            .unwrap()
            .schema_version()
            .unwrap(),
        1
    );

    let legacy_register = CollectionCatalogue::register(&root, request()).unwrap_err();
    assert_eq!(
        legacy_register.category(),
        vlajmaji_store::CollectionErrorCategory::MigrationRequired
    );
    let legacy_install = CollectionCatalogue::install(&root, request()).unwrap_err();
    assert_eq!(
        legacy_install.category(),
        vlajmaji_store::CollectionErrorCategory::MigrationRequired
    );
    assert_eq!(fs::read(&db).unwrap(), v1_bytes);

    let old_reader = CollectionCatalogue::open(&root).unwrap();
    assert_eq!(old_reader.schema_version().unwrap(), 1);
    assert_eq!(
        old_reader.generation().unwrap().unwrap().generation_id,
        prior_generation
    );
    let expected_revision = old_reader.local_revision().unwrap();
    let busy = CollectionCatalogue::migrate(&root, expected_revision).unwrap_err();
    assert_eq!(
        busy.category(),
        vlajmaji_store::CollectionErrorCategory::Busy,
        "actual migration error: {busy:?}"
    );
    assert_eq!(
        CollectionCatalogue::open(&root)
            .unwrap()
            .schema_version()
            .unwrap(),
        1
    );
    drop(old_reader);
    let stale = CollectionCatalogue::migrate(&root, expected_revision + 1).unwrap_err();
    assert_eq!(
        stale.category(),
        vlajmaji_store::CollectionErrorCategory::RevisionConflict
    );
    assert_eq!(fs::read(&db).unwrap(), v1_bytes);
    assert_eq!(
        CollectionCatalogue::open(&root)
            .unwrap()
            .schema_version()
            .unwrap(),
        1
    );

    let migrated = CollectionCatalogue::migrate(&root, expected_revision).unwrap();
    assert_eq!(migrated.outcome, "migrated");
    assert_eq!(migrated.prior_schema_version, 1);
    assert_eq!(migrated.schema_version, 2);
    assert_eq!(migrated.local_revision, expected_revision + 1);
    assert_eq!(
        migrated.active_generation.as_deref(),
        Some(prior_generation.as_str())
    );
    let catalogue = CollectionCatalogue::open(&root).unwrap();
    assert_eq!(catalogue.schema_version().unwrap(), 2);
    assert_eq!(
        catalogue.generation().unwrap().unwrap().generation_id,
        prior_generation
    );
    let list = catalogue.list().unwrap();
    assert_eq!(list.catalogue_schema_version, 2);
    let connection = Connection::open(&db).unwrap();
    let old_names: i64 = connection
        .query_row("SELECT count(*) FROM sqlite_master WHERE type='table' AND name IN ('registrations','registration_locations','generations','installed_heads','attempts')", [], |row| row.get(0))
        .unwrap();
    assert_eq!(old_names, 0);
    let meta_view: String = connection
        .query_row(
            "SELECT type FROM sqlite_master WHERE name='catalogue_meta'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(meta_view, "view");
    assert!(
        connection
            .execute(
                "UPDATE catalogue_meta SET value='1' WHERE key='schema_version'",
                []
            )
            .is_err()
    );
    assert!(
        connection
            .query_row("SELECT count(*) FROM registrations", [], |row| row
                .get::<_, i64>(0))
            .is_err()
    );
    for table in [
        "catalogue_meta",
        "registrations",
        "registration_locations",
        "generations",
        "installed_heads",
        "attempts",
    ] {
        for statement in [
            format!("INSERT INTO {table} DEFAULT VALUES"),
            format!("UPDATE {table} SET rowid=rowid WHERE 0"),
            format!("DELETE FROM {table} WHERE 0"),
        ] {
            assert!(
                connection.execute(&statement, []).is_err(),
                "legacy write was accepted: {statement}"
            );
        }
    }
    let repeat = CollectionCatalogue::migrate(&root, migrated.local_revision).unwrap();
    assert_eq!(repeat.outcome, "idempotent");
    assert_eq!(repeat.local_revision, migrated.local_revision);
    let _ = fs::remove_dir_all(split_root);
    let _ = fs::remove_dir_all(root);
}

#[path = "../../lexicon/examples/support/dictionary_fixture.rs"]
mod fixture_support;

#[test]
fn test_update_replaces_selected_pack_and_rebuild_forces_new_generation() {
    let root = fresh_root("update-rebuild");
    let (initial_release, initial_seal) = sealed(PackId::Llg);
    let initial_request = || {
        CollectionRequest::new(
            vec![sealed_input(PackId::Llg, &initial_release, &initial_seal)],
            CollectionSelection::Packs(vec![PackId::Llg]),
            true,
        )
        .unwrap()
    };
    CollectionCatalogue::register(&root, initial_request()).unwrap();
    let initial = CollectionCatalogue::install(&root, initial_request()).unwrap();
    let initial_generation = initial.generation.unwrap();
    let initial_revision = initial.local_revision;

    let (la_release, la_seal) = sealed(PackId::LaLojban);
    let unavailable_update = CollectionRequest::new(
        vec![sealed_input(PackId::LaLojban, &la_release, &la_seal)],
        CollectionSelection::Packs(vec![PackId::LaLojban]),
        true,
    )
    .unwrap();
    let before_unavailable_update = fs::read(root.join("catalogue.sqlite3")).unwrap();
    let unavailable =
        CollectionCatalogue::update(&root, unavailable_update, initial_revision).unwrap_err();
    let unavailable_source = match &unavailable {
        vlajmaji_store::CollectionError::MutationContext { source, .. } => source.as_ref(),
        other => other,
    };
    assert!(matches!(
        unavailable_source,
        vlajmaji_store::CollectionError::NotInstalled {
            pack: Some(PackId::LaLojban)
        }
    ));
    assert_eq!(
        unavailable.category(),
        vlajmaji_store::CollectionErrorCategory::NotInstalled
    );
    assert_eq!(
        unavailable.report("update").pack_id.as_deref(),
        Some("la-lojban")
    );
    assert_eq!(
        fs::read(root.join("catalogue.sqlite3")).unwrap(),
        before_unavailable_update
    );

    let mut document = fixture_support::complete_fixture().unwrap().into_document();
    // Keep the valid normalized typed fixture while removing its legacy archive,
    // which is intentionally not a supported LLG release payload.
    document.legacy_archive = None;
    document.metadata.entries[0].spelling =
        vlajmaji_lexicon::Presence::Value("updated-spelling".into());
    let revised = Dictionary::from_document(document).unwrap();
    assert!(!revised.metadata().forms.is_empty());
    assert!(!revised.metadata().editorial_senses.is_empty());
    let expected_revised = revised.clone();
    let release = seal_release(PackId::Llg, PackPayloadKind::Synthetic, &revised).unwrap();
    let revised_request = || {
        CollectionRequest::new(
            vec![sealed_input(PackId::Llg, &release.release, &release.seal)],
            CollectionSelection::Packs(vec![PackId::Llg]),
            true,
        )
        .unwrap()
    };

    let stale =
        CollectionCatalogue::update(&root, revised_request(), initial_revision - 1).unwrap_err();
    assert_eq!(
        stale.category(),
        vlajmaji_store::CollectionErrorCategory::RevisionConflict
    );
    let before_update = CollectionCatalogue::open(&root).unwrap().list().unwrap();
    assert_eq!(before_update.local_revision, initial_revision);
    assert_eq!(
        before_update.generation.unwrap().generation_id,
        initial_generation.generation_id
    );

    let updated = CollectionCatalogue::update(&root, revised_request(), initial_revision).unwrap();
    assert_eq!(updated.outcome, "updated");
    assert_eq!(updated.local_revision, initial_revision + 1);
    let updated_generation = updated.generation.unwrap();
    assert_ne!(
        updated_generation.generation_id,
        initial_generation.generation_id
    );
    let updated_actual = vlajmaji_store::DictionaryStore::open(&updated_generation.store_path)
        .unwrap()
        .dictionary()
        .unwrap();
    assert_eq!(updated_actual, expected_revised);

    let exact_repeat =
        CollectionCatalogue::update(&root, revised_request(), updated.local_revision).unwrap();
    assert_eq!(exact_repeat.outcome, "idempotent");
    assert_eq!(exact_repeat.local_revision, updated.local_revision);
    assert_eq!(
        exact_repeat.generation.unwrap().generation_id,
        updated_generation.generation_id
    );

    let full_kind = seal_release(PackId::Llg, PackPayloadKind::Full, &expected_revised).unwrap();
    assert_eq!(full_kind.release, release.release);
    assert_ne!(full_kind.seal, release.seal);
    let changed_kind_request = CollectionRequest::new(
        vec![sealed_input(
            PackId::Llg,
            &full_kind.release,
            &full_kind.seal,
        )],
        CollectionSelection::Packs(vec![PackId::Llg]),
        true,
    )
    .unwrap();
    let before_changed_kind = fs::read(root.join("catalogue.sqlite3")).unwrap();
    let changed_kind_error =
        CollectionCatalogue::update(&root, changed_kind_request, updated.local_revision)
            .unwrap_err();
    assert_eq!(
        changed_kind_error.category(),
        vlajmaji_store::CollectionErrorCategory::IdentityConflict
    );
    assert_eq!(
        fs::read(root.join("catalogue.sqlite3")).unwrap(),
        before_changed_kind
    );

    let locations = CollectionCatalogue::open(&root)
        .unwrap()
        .installed_artifact_locations(PackId::Llg)
        .unwrap()
        .unwrap();
    let release_bytes = fs::read(&locations.release_path).unwrap();
    let before_bad_copy = fs::read(root.join("catalogue.sqlite3")).unwrap();
    fs::write(&locations.release_path, b"corrupt sealed release").unwrap();
    assert!(CollectionCatalogue::rebuild(&root, updated.local_revision).is_err());
    assert_eq!(
        fs::read(root.join("catalogue.sqlite3")).unwrap(),
        before_bad_copy
    );
    fs::write(&locations.release_path, &release_bytes).unwrap();

    // Rebuild repairs a damaged derived SQLite copy from the retained sealed input.
    fs::write(&locations.database_path, b"corrupt derived sqlite").unwrap();
    let rebuilt = CollectionCatalogue::rebuild(&root, updated.local_revision).unwrap();
    assert_eq!(rebuilt.outcome, "rebuilt");
    assert_eq!(rebuilt.local_revision, updated.local_revision + 1);
    let rebuilt_generation = rebuilt.generation.unwrap();
    assert_ne!(
        rebuilt_generation.generation_id,
        updated_generation.generation_id
    );
    assert_eq!(rebuilt_generation.content_id, updated_generation.content_id);
    assert_eq!(
        rebuilt_generation.canonical_sha256,
        updated_generation.canonical_sha256
    );
    let verification = vlajmaji_store::DictionaryStoreVerification::verify(
        &rebuilt_generation.store_path,
        vlajmaji_store::DictionaryStoreVerificationCheck::All,
    )
    .unwrap();
    assert!(verification.ok);
    let rebuilt_actual = vlajmaji_store::DictionaryStore::open(&rebuilt_generation.store_path)
        .unwrap()
        .dictionary()
        .unwrap();
    assert_eq!(rebuilt_actual, expected_revised);
    let list = CollectionCatalogue::open(&root).unwrap().list().unwrap();
    assert_eq!(
        list.generation.unwrap().generation_id,
        rebuilt_generation.generation_id
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn test_four_source_update_and_rebuild_preserve_complete_typed_inventory() {
    let seal_models = |models: Vec<(PackId, Dictionary)>| {
        models
            .into_iter()
            .map(|(pack_id, dictionary)| {
                let sealed =
                    seal_release(pack_id, PackPayloadKind::Synthetic, &dictionary).unwrap();
                (pack_id, dictionary, sealed.release, sealed.seal)
            })
            .collect::<Vec<_>>()
    };

    let initial_packs = readiness_four_pack_models("synthetic develops");
    assert_eq!(initial_packs.len(), 4);
    let initial_models = initial_packs
        .iter()
        .map(|(_, dictionary)| dictionary.clone())
        .collect::<Vec<_>>();
    let source_record = |dictionary: &Dictionary| {
        dictionary
            .registry()
            .as_document()
            .sources
            .iter()
            .find(|source| source.id.to_string() == "lensisku")
            .cloned()
            .expect("the source-literal imports share a Lensisku source record")
    };
    assert_eq!(
        source_record(&initial_models[0]),
        source_record(&initial_models[2])
    );
    assert_ne!(
        initial_models[0].metadata().entries[0].id,
        initial_models[2].metadata().entries[0].id,
        "the raw source bytes are shared, but occurrence-scoped entry IDs differ"
    );
    let initial_expected_inventory = max_family_inventory(&initial_models);
    assert_eq!(initial_expected_inventory.len(), 25);
    assert_eq!(
        initial_expected_inventory
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        [
            "aliases",
            "annotations",
            "assertions",
            "captures",
            "code_notices",
            "collections",
            "components",
            "computations",
            "contexts",
            "contributors",
            "distributors",
            "editorial_senses",
            "entries",
            "etymologies",
            "extensions",
            "forms",
            "frames",
            "lineage",
            "occurrences",
            "owned_heads",
            "owned_revisions",
            "relations",
            "releases",
            "snapshots",
            "sources",
        ]
    );
    assert!(initial_expected_inventory.contains_key("frames"));
    assert!(initial_expected_inventory["frames"].is_empty());
    let expected_raw_assertions = [1, 1, 1, 3];
    let expected_lexical_assertions = [1, 1, 1, 2];
    for (index, (_, dictionary)) in initial_packs.iter().enumerate() {
        assert_eq!(
            dictionary.metadata().assertions.len(),
            expected_raw_assertions[index]
        );
        assert_eq!(
            dictionary
                .metadata()
                .assertions
                .iter()
                .filter(|row| row.eligibility == Eligibility::Lexical)
                .count(),
            expected_lexical_assertions[index]
        );
    }
    assert_eq!(expected_raw_assertions.iter().sum::<usize>(), 6);
    assert_eq!(expected_lexical_assertions.iter().sum::<usize>(), 5);

    // The comparator is exact over all categories, including empty ones, and
    // treats rows as a multiset. Its expected map is built from the four
    // independently imported input models before any catalogue operation.
    let mut reordered_models = initial_models.clone();
    let mut reordered_document = initial_models[3].clone().into_document();
    reordered_document.metadata.assertions.reverse();
    reordered_models[3] = Dictionary::from_document(reordered_document).unwrap();
    assert_eq!(
        max_family_inventory(&reordered_models),
        initial_expected_inventory
    );
    let mut missing_empty = initial_expected_inventory.clone();
    missing_empty.remove("frames");
    assert_ne!(missing_empty, initial_expected_inventory);
    let mut extra_empty = initial_expected_inventory.clone();
    extra_empty.insert("unexpected_empty".to_owned(), BTreeMap::new());
    assert_ne!(extra_empty, initial_expected_inventory);
    let mut changed_multiplicity = initial_expected_inventory.clone();
    let (_, one_row) = changed_multiplicity
        .iter_mut()
        .find(|(_, rows)| !rows.is_empty())
        .unwrap();
    *one_row.values_mut().next().unwrap() += 1;
    assert_ne!(changed_multiplicity, initial_expected_inventory);
    let mut changed_body_document = initial_models[0].clone().into_document();
    changed_body_document.metadata.entries[0].language = Presence::Missing;
    let changed_body = Dictionary::from_document(changed_body_document).unwrap();
    assert_ne!(
        family_inventory(&changed_body),
        family_inventory(&initial_models[0])
    );

    let initial_expected = compose_dictionaries(initial_models).unwrap();
    let initial_sealed = seal_models(initial_packs);
    let root = fresh_root("four-source-update-rebuild");
    let installed =
        CollectionCatalogue::install(&root, readiness_four_pack_request(&initial_sealed)).unwrap();
    let initial_generation = installed.generation.unwrap();
    let initial_actual = vlajmaji_store::DictionaryStore::open(&initial_generation.store_path)
        .unwrap()
        .dictionary()
        .unwrap();
    assert_eq!(initial_actual, initial_expected);
    assert_eq!(
        family_inventory(&initial_actual),
        initial_expected_inventory
    );

    let updated_packs = readiness_four_pack_models("synthetic develops further");
    let mut conflicting_packs = readiness_four_pack_models("synthetic develops further");
    let mut conflicting_document = conflicting_packs[0].1.clone().into_document();
    let mut conflicting_registry = conflicting_document.registry.as_document().clone();
    let shared_source = conflicting_registry
        .sources
        .iter_mut()
        .find(|source| source.id.to_string() == "lensisku")
        .expect("the Jbotci literal declares the shared Lensisku source");
    shared_source.display_name.push_str(" (conflicting body)");
    conflicting_document.registry = SourceRegistry::from_document(conflicting_registry).unwrap();
    conflicting_packs[0].1 = Dictionary::from_document(conflicting_document).unwrap();
    let conflicting_sealed = seal_models(conflicting_packs);
    let before_conflict = fs::read(root.join("catalogue.sqlite3")).unwrap();
    let conflict = CollectionCatalogue::update(
        &root,
        readiness_four_pack_request(&conflicting_sealed),
        installed.local_revision,
    )
    .unwrap_err();
    assert_eq!(
        conflict.category(),
        CollectionErrorCategory::IdentityConflict
    );
    assert_eq!(
        fs::read(root.join("catalogue.sqlite3")).unwrap(),
        before_conflict
    );

    let updated_expected = compose_dictionaries(
        updated_packs
            .iter()
            .map(|(_, dictionary)| dictionary.clone())
            .collect(),
    )
    .unwrap();
    assert_ne!(updated_expected, initial_expected);
    let updated_expected_inventory = max_family_inventory(
        &updated_packs
            .iter()
            .map(|(_, dictionary)| dictionary.clone())
            .collect::<Vec<_>>(),
    );
    let updated_sealed = seal_models(updated_packs);
    let updated = CollectionCatalogue::update(
        &root,
        readiness_four_pack_request(&updated_sealed),
        installed.local_revision,
    )
    .unwrap();
    assert_eq!(updated.outcome, "updated");
    assert_eq!(updated.inputs.len(), 4);
    let updated_generation = updated.generation.unwrap();
    assert_ne!(
        updated_generation.generation_id,
        initial_generation.generation_id
    );
    let updated_actual = vlajmaji_store::DictionaryStore::open(&updated_generation.store_path)
        .unwrap()
        .dictionary()
        .unwrap();
    assert_eq!(updated_actual, updated_expected);
    assert_eq!(
        family_inventory(&updated_actual),
        updated_expected_inventory
    );

    let rebuilt = CollectionCatalogue::rebuild(&root, updated.local_revision).unwrap();
    assert_eq!(rebuilt.outcome, "rebuilt");
    assert_eq!(rebuilt.inputs.len(), 4);
    let rebuilt_generation = rebuilt.generation.unwrap();
    assert_ne!(
        rebuilt_generation.generation_id,
        updated_generation.generation_id
    );
    assert_eq!(
        rebuilt_generation.canonical_sha256,
        updated_generation.canonical_sha256
    );
    assert_eq!(rebuilt_generation.content_id, updated_generation.content_id);
    let rebuilt_actual = vlajmaji_store::DictionaryStore::open(&rebuilt_generation.store_path)
        .unwrap()
        .dictionary()
        .unwrap();
    assert_eq!(rebuilt_actual, updated_expected);
    assert_eq!(
        family_inventory(&rebuilt_actual),
        updated_expected_inventory
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn test_update_and_migrate_missing_catalogue_do_not_create_root() {
    let root = fresh_root("missing-maintenance");
    let (release, seal) = sealed(PackId::Llg);
    let request = CollectionRequest::new(
        vec![sealed_input(PackId::Llg, &release, &seal)],
        CollectionSelection::Packs(vec![PackId::Llg]),
        true,
    )
    .unwrap();
    let update = CollectionCatalogue::update(&root, request, 0).unwrap_err();
    assert!(matches!(
        update,
        vlajmaji_store::CollectionError::MutationContext { source, .. }
            if matches!(source.as_ref(), vlajmaji_store::CollectionError::MissingCatalogue(_))
    ));
    assert!(
        !root.exists(),
        "update of an absent catalogue must not initialize it"
    );
    let migration = CollectionCatalogue::migrate(&root, 0).unwrap_err();
    assert!(matches!(
        migration,
        vlajmaji_store::CollectionError::MissingCatalogue(_)
    ));
    assert!(
        !root.exists(),
        "explicit migration of an absent catalogue must not create it"
    );

    let registered_root = fresh_root("registered-only-rebuild");
    let registered_request = CollectionRequest::new(
        vec![sealed_input(PackId::Llg, &release, &seal)],
        CollectionSelection::Packs(vec![PackId::Llg]),
        true,
    )
    .unwrap();
    let registered = CollectionCatalogue::register(&registered_root, registered_request).unwrap();
    let no_active_generation =
        CollectionCatalogue::rebuild(&registered_root, registered.local_revision).unwrap_err();
    let no_active_source = match &no_active_generation {
        vlajmaji_store::CollectionError::MutationContext { source, .. } => source.as_ref(),
        other => other,
    };
    assert!(matches!(
        no_active_source,
        vlajmaji_store::CollectionError::NotInstalled { pack: None }
    ));
    assert_eq!(
        no_active_generation.category(),
        vlajmaji_store::CollectionErrorCategory::NotInstalled
    );
    assert_eq!(no_active_generation.report("rebuild").pack_id, None);
    let _ = fs::remove_dir_all(registered_root);
}
