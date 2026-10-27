//! Public configured-query contract against a tiny synthetic local collection.

#[path = "../../lexicon/examples/support/dictionary_fixture.rs"]
mod fixture;

use std::{
    borrow::Cow,
    fs,
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use sha2::{Digest, Sha256};
use vlajmaji_lexicon::{
    CaptureRef, CatalogTarget, Dictionary, JbotciImportOptions, LaLojbanImportOptions,
    LlgImportOptions, LocalRef, OccurrenceRef, PackId, PackPayloadKind, Presence, RawCapture,
    RelationTarget, SnapshotAttribution, SourceId, SourceRegistry, SutysiskuImportOptions,
    import_jbotci_export, import_la_lojban_export, import_llg_lists, import_sutysisku_export,
    seal_release,
};
use vlajmaji_search::{
    CollectionQueryError, CollectionQueryResult, CollectionQuerySelection, CollectionSearchEngine,
    DictionaryExactOptions, DictionaryRequest, DictionarySearchEngine,
};
use vlajmaji_store::{
    CollectionCatalogue, CollectionInput, CollectionProfileWriteRequest, CollectionProfiles,
    CollectionRequest, CollectionSelection, DictionaryReadTopology,
};

fn fresh_root(label: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "vlajmaji-s03-managed-{label}-{}-{stamp}",
        std::process::id()
    ))
}

fn install_fixture(root: &std::path::Path) -> vlajmaji_store::CollectionGeneration {
    let mut document = fixture::complete_fixture().unwrap().into_document();
    // The fixture includes a derived legacy archive, which pack installation
    // intentionally rejects; this test exercises the preserved typed model.
    document.legacy_archive = None;
    let dictionary = Dictionary::from_document(document).unwrap();
    let sealed = seal_release(PackId::Llg, PackPayloadKind::Synthetic, &dictionary).unwrap();
    let input = CollectionInput::SealedBytes {
        pack_id: PackId::Llg,
        release: Cow::Borrowed(&sealed.release),
        seal: Cow::Borrowed(&sealed.seal),
    };
    let request = CollectionRequest::new(
        vec![input],
        CollectionSelection::Packs(vec![PackId::Llg]),
        true,
    )
    .unwrap();
    CollectionCatalogue::install(root, request)
        .unwrap()
        .generation
        .unwrap()
}

fn seal_input(pack_id: PackId, dictionary: &Dictionary) -> (Vec<u8>, Vec<u8>) {
    let sealed = seal_release(pack_id, PackPayloadKind::Synthetic, dictionary).unwrap();
    (sealed.release, sealed.seal)
}

fn install_sealed_pair(
    root: &std::path::Path,
    first: (PackId, &Dictionary),
    second: (PackId, &Dictionary),
) {
    let (first_release, first_seal) = seal_input(first.0, first.1);
    let (second_release, second_seal) = seal_input(second.0, second.1);
    let request = CollectionRequest::new(
        vec![
            CollectionInput::SealedBytes {
                pack_id: first.0,
                release: Cow::Owned(first_release),
                seal: Cow::Owned(first_seal),
            },
            CollectionInput::SealedBytes {
                pack_id: second.0,
                release: Cow::Owned(second_release),
                seal: Cow::Owned(second_seal),
            },
        ],
        CollectionSelection::Packs(vec![first.0, second.0]),
        true,
    )
    .unwrap();
    CollectionCatalogue::install(root, request).unwrap();
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

fn selection() -> CollectionQuerySelection {
    CollectionQuerySelection {
        packs: vec![PackId::Llg],
        disabled: vec![],
        profile: None,
    }
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[test]
fn test_public_profile_lookup_errors_keep_typed_categories_and_context() {
    let root = fresh_root("profile-error-categories");
    register_pack(&root, PackId::Llg);
    let config = root.join("profiles.toml");
    fs::write(
        &config,
        format!(
            "version = 1\nrevision = 1\ndefault_profile = 'research'\n\n[profiles.research]\nroot = {:?}\npacks = []\ndisabled = ['llg']\n",
            root.to_string_lossy()
        ),
    )
    .unwrap();
    let profiles = CollectionProfiles::read(&config).unwrap();

    let unknown = profiles.profile(Some("missing")).unwrap_err();
    let report = CollectionQueryError::Profile(unknown).report_for("lookup");
    assert_eq!(report.category, "unknown_profile");
    assert_eq!(
        report.config_path.unwrap().utf8.as_deref(),
        Some(config.to_str().unwrap())
    );
    assert_eq!(report.requested_profile.as_deref(), Some("missing"));
    assert!(report.root_path.is_none());

    let disabled =
        match CollectionSearchEngine::open_configured(&profiles, None, Some(&[PackId::Llg])) {
            Ok(_) => panic!("disabled pack unexpectedly opened"),
            Err(error) => error,
        };
    let CollectionQueryError::DisabledSelection { pack_id } = &disabled else {
        panic!("expected typed disabled-selection error, got {disabled}");
    };
    assert_eq!(pack_id, "llg");
    let report = disabled.report_for("lookup");
    assert_eq!(report.category, "disabled_selection");
    assert_eq!(report.known_pack_ids, vec!["llg"]);
    assert!(
        report.root_path.is_none(),
        "direct library report has no CLI root context"
    );
    fs::remove_dir_all(root).unwrap();
}

fn install_two_release_history(
    root: &std::path::Path,
    conflicting_body: bool,
) -> (
    vlajmaji_lexicon::EntityId,
    vlajmaji_lexicon::EntityId,
    vlajmaji_lexicon::EntityId,
    vlajmaji_lexicon::OccurrenceRef,
    usize,
    usize,
) {
    let mut first_document = fixture::complete_fixture().unwrap().into_document();
    first_document.legacy_archive = None;
    let assertion_id = first_document.metadata.assertions[0].id.clone();
    let occurrence = first_document.metadata.assertions[0].occurrence.clone();
    let current_release_id = first_document.metadata.releases[0].id.clone();
    let source_a_entries: std::collections::BTreeSet<_> = first_document
        .metadata
        .entries
        .iter()
        .filter(|entry| entry.source.as_str() == "source-a")
        .map(|entry| entry.id.clone())
        .collect();
    let current_expected_matches = first_document
        .metadata
        .assertions
        .iter()
        .filter(|assertion| {
            assertion.occurrence == occurrence && source_a_entries.contains(&assertion.entry)
        })
        .count();
    let first = Dictionary::from_document(first_document.clone()).unwrap();

    let mut historical_document = first_document;
    historical_document.metadata.catalogue_revision += 1;
    historical_document.metadata.releases[0].id =
        vlajmaji_lexicon::EntityId::new("release-history").unwrap();
    if conflicting_body {
        historical_document.metadata.assertions[0].native_sense_id =
            vlajmaji_lexicon::Presence::Value("different-native-sense".into());
    }
    let historical = Dictionary::from_document(historical_document).unwrap();
    let historical_release_id = historical.document().metadata.releases[0].id.clone();
    let historical_expected_matches = historical
        .document()
        .metadata
        .assertions
        .iter()
        .filter(|assertion| {
            assertion.occurrence == occurrence && source_a_entries.contains(&assertion.entry)
        })
        .count();
    let current_sealed = seal_release(PackId::Llg, PackPayloadKind::Synthetic, &first).unwrap();
    let historical_sealed =
        seal_release(PackId::Llg, PackPayloadKind::Synthetic, &historical).unwrap();
    let current_request = CollectionRequest::new(
        vec![CollectionInput::SealedBytes {
            pack_id: PackId::Llg,
            release: Cow::Borrowed(&current_sealed.release),
            seal: Cow::Borrowed(&current_sealed.seal),
        }],
        CollectionSelection::Packs(vec![PackId::Llg]),
        true,
    )
    .unwrap();
    CollectionCatalogue::install(root, current_request).unwrap();
    let history_request = CollectionRequest::new(
        vec![CollectionInput::SealedBytes {
            pack_id: PackId::Llg,
            release: Cow::Borrowed(&historical_sealed.release),
            seal: Cow::Borrowed(&historical_sealed.seal),
        }],
        CollectionSelection::Packs(vec![PackId::Llg]),
        true,
    )
    .unwrap();
    CollectionCatalogue::register(root, history_request).unwrap();
    (
        assertion_id,
        current_release_id,
        historical_release_id,
        occurrence,
        current_expected_matches,
        historical_expected_matches,
    )
}

#[test]
fn test_historical_local_identity_deduplicates_equal_body_and_rejects_conflict() {
    let equal_root = fresh_root("local-history-equal");
    let (
        equal_id,
        current_release_id,
        historical_release_id,
        occurrence,
        current_expected_matches,
        historical_expected_matches,
    ) = install_two_release_history(&equal_root, false);
    let mut engine = CollectionSearchEngine::open(&equal_root, selection()).unwrap();
    let response = engine
        .execute(DictionaryRequest::Resolve {
            target: RelationTarget::Local(LocalRef::Assertion(equal_id)),
        })
        .unwrap();
    let CollectionQueryResult::Resolve(response) = response.result else {
        panic!("wrong operation result");
    };
    assert!(response.payload.is_some());
    assert_eq!(
        response.catalogue_matches.len(),
        1,
        "equal historical identities are deduplicated"
    );
    let current_target = RelationTarget::Catalog(vlajmaji_lexicon::CatalogTarget {
        source: vlajmaji_lexicon::SourceId::new("source-a").unwrap(),
        release: current_release_id.clone(),
        native_entry_id: None,
        native_assertion_id: None,
        occurrence: Some(occurrence.clone()),
    });
    let current_resolved = engine
        .execute(DictionaryRequest::Resolve {
            target: current_target,
        })
        .unwrap();
    let CollectionQueryResult::Resolve(current_resolved) = current_resolved.result else {
        panic!("wrong reference result");
    };
    assert!(current_resolved.payload.is_some());
    assert_eq!(
        current_resolved.catalogue_matches.len(),
        current_expected_matches
    );
    assert!(
        current_resolved
            .catalogue_matches
            .iter()
            .all(|matched| { matched.model_release_id == current_release_id.as_str() })
    );
    let historical_resolved = engine
        .execute(DictionaryRequest::Resolve {
            target: RelationTarget::Catalog(vlajmaji_lexicon::CatalogTarget {
                source: vlajmaji_lexicon::SourceId::new("source-a").unwrap(),
                release: historical_release_id.clone(),
                native_entry_id: None,
                native_assertion_id: None,
                occurrence: Some(occurrence),
            }),
        })
        .unwrap();
    let CollectionQueryResult::Resolve(historical_resolved) = historical_resolved.result else {
        panic!("wrong reference result");
    };
    assert!(historical_resolved.payload.is_none());
    assert_eq!(
        historical_resolved.catalogue_matches.len(),
        historical_expected_matches
    );
    assert_eq!(
        historical_resolved.catalogue_matches[0].model_release_id,
        historical_release_id.as_str()
    );
    fs::remove_dir_all(equal_root).unwrap();

    let conflict_root = fresh_root("local-history-conflict");
    let (conflict_id, _, _, _, _, _) = install_two_release_history(&conflict_root, true);
    let mut engine = CollectionSearchEngine::open(&conflict_root, selection()).unwrap();
    let error = engine
        .execute(DictionaryRequest::Resolve {
            target: RelationTarget::Local(LocalRef::Assertion(conflict_id)),
        })
        .unwrap_err();
    assert!(matches!(error, CollectionQueryError::AmbiguousReference));
    assert_eq!(error.report_for("resolve").category, "ambiguous_reference");
    fs::remove_dir_all(conflict_root).unwrap();
}

#[test]
fn test_equal_local_contributors_admit_selected_enabled_pack_payload() {
    let mut document = fixture::complete_fixture().unwrap().into_document();
    document.legacy_archive = None;
    let assertion_id = document.metadata.assertions[0].id.clone();
    let dictionary = Dictionary::from_document(document).unwrap();
    let root = fresh_root("equal-local-contributors");
    install_sealed_pair(
        &root,
        (PackId::LaLojban, &dictionary),
        (PackId::Llg, &dictionary),
    );

    let cases = [
        (vec![PackId::LaLojban, PackId::Llg], vec![], true),
        (vec![PackId::LaLojban], vec![], true),
        (vec![PackId::Llg], vec![], true),
        (
            vec![PackId::LaLojban, PackId::Llg],
            vec![PackId::LaLojban],
            true,
        ),
        (vec![PackId::LaLojban, PackId::Llg], vec![PackId::Llg], true),
        (
            vec![PackId::LaLojban, PackId::Llg],
            vec![PackId::LaLojban, PackId::Llg],
            false,
        ),
    ];
    for (packs, disabled, should_admit) in cases {
        let query_selection = CollectionQuerySelection {
            packs,
            disabled,
            profile: None,
        };
        let mut engine = CollectionSearchEngine::open(&root, query_selection).unwrap();
        let result = engine
            .execute(DictionaryRequest::Resolve {
                target: RelationTarget::Local(LocalRef::Assertion(assertion_id.clone())),
            })
            .unwrap();
        let CollectionQueryResult::Resolve(response) = result.result else {
            panic!("expected a resolve response");
        };
        assert_eq!(response.payload.is_some(), should_admit);
        assert_eq!(response.catalogue_matches.len(), 1);
        assert_eq!(
            response.catalogue_matches[0].contributing_pack_ids,
            ["la-lojban", "llg"]
        );
    }
    fs::remove_dir_all(root).unwrap();
}

fn dictionary_with_scoped_assertion() -> (Dictionary, vlajmaji_lexicon::EntityId) {
    let mut document = fixture::complete_fixture().unwrap().into_document();
    document.legacy_archive = None;
    let source_assertion = document
        .metadata
        .assertions
        .iter()
        .find(|assertion| {
            matches!(
                assertion.eligibility,
                vlajmaji_lexicon::Eligibility::Lexical
            )
        })
        .unwrap()
        .clone();
    let source_release = document
        .metadata
        .releases
        .iter()
        .find(|release| release.assertions.contains(&source_assertion.id))
        .unwrap()
        .clone();
    let source_capture = document
        .captures
        .iter()
        .find(|capture| capture.identity == source_assertion.occurrence.capture)
        .unwrap()
        .clone();
    let occurrence = source_capture
        .occurrences
        .iter()
        .find(|occurrence| occurrence.locator == source_assertion.occurrence.locator)
        .unwrap()
        .clone();
    let new_capture = RawCapture::new(
        format!("{}:contributor-two", source_capture.identity.namespace),
        source_capture.metadata.clone(),
        source_capture.bytes.clone(),
        vec![occurrence],
    );
    let new_capture_ref = new_capture.identity.clone();
    let new_assertion_id = vlajmaji_lexicon::EntityId::new("assertion-contributor-two").unwrap();
    let mut new_assertion = source_assertion;
    new_assertion.id = new_assertion_id.clone();
    new_assertion.occurrence.capture = new_capture_ref.clone();
    new_assertion.native_sense_id = Presence::Value("contributor-two-sense".into());

    let mut new_release = source_release;
    new_release.id = vlajmaji_lexicon::EntityId::new("release-contributor-two").unwrap();
    new_release.captures = vec![new_capture_ref.clone()];
    new_release.assertions = vec![new_assertion_id.clone()];
    new_release.accounting.input_occurrences = Some(1);
    new_release.accounting.normalized_occurrences = Some(1);
    new_release.accounting.query_excluded_occurrences = Some(0);
    new_release.diagnostics.clear();

    let mut registry = document.registry.document();
    let source_binding = registry
        .snapshots
        .iter()
        .find(|binding| {
            binding.snapshot.namespace() == source_capture.identity.namespace
                && binding.snapshot.sha256() == source_capture.identity.sha256
        })
        .unwrap()
        .clone();
    registry.snapshots.push(SnapshotAttribution {
        snapshot: new_capture_ref.to_snapshot_identity().unwrap(),
        source: source_binding.source,
        distributor: source_binding.distributor,
    });
    document.registry = SourceRegistry::from_document(registry).unwrap();
    document.captures.push(new_capture);
    document.metadata.assertions.push(new_assertion);
    document.metadata.releases.push(new_release);
    (
        Dictionary::from_document(document).unwrap(),
        new_assertion_id,
    )
}

#[test]
fn test_shared_entry_retains_each_selected_contributor_release_scope() {
    let mut first_document = fixture::complete_fixture().unwrap().into_document();
    first_document.legacy_archive = None;
    let first_dictionary = Dictionary::from_document(first_document).unwrap();
    let (second_dictionary, contributor_assertion) = dictionary_with_scoped_assertion();
    let entry_id = first_dictionary
        .document()
        .metadata
        .assertions
        .iter()
        .find(|assertion| {
            matches!(
                assertion.eligibility,
                vlajmaji_lexicon::Eligibility::Lexical
            )
        })
        .unwrap()
        .entry
        .clone();
    let selected_assertions_for = |dictionary: &Dictionary| {
        let document = dictionary.document();
        document
            .metadata
            .assertions
            .iter()
            .filter(|assertion| {
                assertion.entry == entry_id
                    && matches!(
                        assertion.eligibility,
                        vlajmaji_lexicon::Eligibility::Lexical
                    )
                    && document.metadata.releases.iter().any(|release| {
                        release.selected
                            && release.availability == vlajmaji_lexicon::Availability::Available
                            && release.assertions.contains(&assertion.id)
                    })
            })
            .map(|assertion| assertion.id.to_string())
            .collect::<std::collections::BTreeSet<_>>()
    };
    let first_expected = selected_assertions_for(&first_dictionary);
    let second_expected = selected_assertions_for(&second_dictionary);
    let union_expected = first_expected
        .union(&second_expected)
        .cloned()
        .collect::<std::collections::BTreeSet<_>>();
    let root = fresh_root("shared-entry-release-scopes");
    install_sealed_pair(
        &root,
        (PackId::LaLojban, &first_dictionary),
        (PackId::Llg, &second_dictionary),
    );

    let cases = [
        (
            vec![PackId::LaLojban, PackId::Llg],
            vec![],
            true,
            Some(union_expected),
        ),
        (
            vec![PackId::LaLojban],
            vec![],
            false,
            Some(first_expected.clone()),
        ),
        (
            vec![PackId::Llg],
            vec![],
            true,
            Some(second_expected.clone()),
        ),
        (
            vec![PackId::LaLojban, PackId::Llg],
            vec![PackId::LaLojban],
            true,
            Some(second_expected.clone()),
        ),
        (
            vec![PackId::LaLojban, PackId::Llg],
            vec![PackId::Llg],
            false,
            Some(first_expected),
        ),
        (
            vec![PackId::LaLojban, PackId::Llg],
            vec![PackId::LaLojban, PackId::Llg],
            false,
            None,
        ),
    ];
    for (packs, disabled, include_second_contributor, expected_ids) in cases {
        let mut engine = CollectionSearchEngine::open(
            &root,
            CollectionQuerySelection {
                packs,
                disabled,
                profile: None,
            },
        )
        .unwrap();
        let result = engine
            .execute(DictionaryRequest::Resolve {
                target: RelationTarget::Local(LocalRef::Entry(entry_id.clone())),
            })
            .unwrap();
        let CollectionQueryResult::Resolve(response) = result.result else {
            panic!("expected a resolve response");
        };
        assert_eq!(response.payload.is_some(), expected_ids.is_some());
        if let Some(expected_ids) = expected_ids {
            let payload = response.payload.expect("selected contributor has payload");
            let actual_ids = payload
                .confirmed
                .iter()
                .map(|assertion| assertion.assertion.id.to_string())
                .collect::<std::collections::BTreeSet<_>>();
            assert_eq!(actual_ids, expected_ids);
            assert_eq!(
                actual_ids.contains(contributor_assertion.as_str()),
                include_second_contributor
            );
        }
        assert_eq!(response.catalogue_matches.len(), 1);
        assert_eq!(
            response.catalogue_matches[0].contributing_pack_ids,
            ["la-lojban", "llg"]
        );
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn test_readiness_four_source_literals_keep_independent_exact_counts() {
    // These are the readiness01 source literals, copied into this test so the
    // source checkout does not depend on a sibling planning worktree.
    const LENSISKU: &[u8] = br#"[{"word":"farvi","word_type":"gismu","definition":"synthetic develops","definition_id":333,"score":1.2500,"user":{"username":"shared"}}]"#;
    const SUTYSISKU: &[u8] = br#"{"farvi":{"d":"synthetic one","t":"gismu","n":null},"farvi":{"d":"synthetic two","t":"gismu"},"zoizoizoi":{"d":"retained excluded","t":"unsupported"}}"#;
    const LLG_GISMU_HEX: &str = "206661727669202020202020202020202020202067726f772020202020202020202020202020202020202020202020202020202020202020202020202020642020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020202020";
    const LLG_EMPTY: &[u8] = b"";
    const LLG_NOTICE: &[u8] = b"synthetic notice\n";
    fn unhex(value: &str) -> Vec<u8> {
        let (pairs, remainder) = value.as_bytes().as_chunks::<2>();
        assert!(remainder.is_empty());
        pairs
            .iter()
            .map(|pair| {
                let digit = |byte: u8| (byte as char).to_digit(16).unwrap() as u8;
                (digit(pair[0]) << 4) | digit(pair[1])
            })
            .collect()
    }
    let llg_gismu = unhex(LLG_GISMU_HEX);
    assert_eq!(LENSISKU.len(), 136);
    assert_eq!(
        sha256(LENSISKU),
        "63fb2e10e80f34e6d7f04fbe9bcfe1cd69c7ec0c4016045d02e8b3cfcd866429"
    );
    assert_eq!(SUTYSISKU.len(), 150);
    assert_eq!(
        sha256(SUTYSISKU),
        "3e7738c9412a4dfb6169e8492f8ecb7fd25003d2141dbb8fa11782e580d00b03"
    );
    assert_eq!(llg_gismu.len(), 158);
    assert_eq!(
        sha256(&llg_gismu),
        "0408fd8bf1adae9d0da1e7faabb35a6c2cdf64d012140bcadfdd814f061b2735"
    );

    let metadata = format!(
        "language_tag = \"en\"\nlanguage_realname = \"English\"\nsource_language_tag = \"jbo\"\nformat = \"json\"\npositive_scores_only = false\nfilename = \"dictionary-en.json\"\nmetadata_url = \"https://example.invalid/languages\"\ndownload_url = \"https://example.invalid/export\"\nlensisku_created_at = \"2026-10-04T03:07:08Z\"\nsha256 = \"{}\"\ndefinition_count = 1\nentry_count = 1\n",
        sha256(LENSISKU),
    );
    let jb = import_jbotci_export(
        LENSISKU,
        metadata.as_bytes(),
        &JbotciImportOptions {
            revision: "1111111111111111111111111111111111111111".into(),
            expected_raw_sha256: sha256(LENSISKU),
            expected_metadata_sha256: sha256(metadata.as_bytes()),
        },
    )
    .unwrap();
    let la = import_la_lojban_export(
        LENSISKU,
        &LaLojbanImportOptions {
            revision: "2222222222222222222222222222222222222222".into(),
            expected_raw_sha256: sha256(LENSISKU),
            raw_locator: None,
            notice_locator: None,
        },
    )
    .unwrap();
    let sut = import_sutysisku_export(
        SUTYSISKU,
        &SutysiskuImportOptions {
            revision: "3333333333333333333333333333333333333333".into(),
            expected_raw_sha256: sha256(SUTYSISKU),
            raw_locator: None,
            notice_locator: None,
        },
    )
    .unwrap();
    let llg = import_llg_lists(
        &llg_gismu,
        LLG_EMPTY,
        LLG_EMPTY,
        LLG_NOTICE,
        &LlgImportOptions {
            expected_gismu_sha256: sha256(&llg_gismu),
            expected_cmavo_sha256: sha256(LLG_EMPTY),
            expected_rafsi_sha256: sha256(LLG_EMPTY),
            expected_notice_sha256: sha256(LLG_NOTICE),
            base_locator: "https://example.invalid/llg".into(),
        },
    )
    .unwrap();
    assert_eq!(jb.report().input_rows, 1);
    assert_eq!(la.report().input_rows, 1);
    assert_eq!(sut.report().input_rows, 3);
    assert_eq!(
        llg.report()
            .components
            .iter()
            .map(|row| row.input_lines)
            .sum::<u64>(),
        1
    );

    let packs = [
        (PackId::LensiskuJbotci, jb.into_dictionary()),
        (PackId::Llg, llg.into_dictionary()),
        (PackId::LaLojban, la.into_dictionary()),
        (PackId::Sutysisku, sut.into_dictionary()),
    ];
    let sealed: Vec<_> = packs
        .into_iter()
        .map(|(pack, dictionary)| {
            (
                pack,
                seal_release(pack, PackPayloadKind::Synthetic, &dictionary).unwrap(),
            )
        })
        .collect();
    let inputs = sealed
        .iter()
        .map(|(pack, release)| CollectionInput::SealedBytes {
            pack_id: *pack,
            release: Cow::Borrowed(release.release.as_slice()),
            seal: Cow::Borrowed(release.seal.as_slice()),
        })
        .collect();
    let selected = vec![
        PackId::LensiskuJbotci,
        PackId::Llg,
        PackId::LaLojban,
        PackId::Sutysisku,
    ];
    let request =
        CollectionRequest::new(inputs, CollectionSelection::Packs(selected.clone()), true).unwrap();
    let root = fresh_root("readiness-four-source");
    CollectionCatalogue::install(&root, request).unwrap();
    let mut engine = CollectionSearchEngine::open(
        &root,
        CollectionQuerySelection {
            packs: selected,
            disabled: vec![],
            profile: None,
        },
    )
    .unwrap();
    let result = engine
        .execute(DictionaryRequest::Exact {
            spelling: "farvi".into(),
            options: DictionaryExactOptions::default(),
        })
        .unwrap();
    let CollectionQueryResult::Exact(result) = result.result else {
        panic!("wrong result kind");
    };
    assert_eq!(result.totals.occurrences, 5);
    let by_source: std::collections::BTreeMap<_, _> = result
        .source_totals
        .iter()
        .map(|row| (row.source.id.to_string(), row.occurrences))
        .collect();
    assert_eq!(
        by_source,
        [
            ("lensisku".to_string(), 2),
            ("llg".to_string(), 1),
            ("sutysisku".to_string(), 2)
        ]
        .into()
    );

    // Expected release/capture/native identity comes from the literal and
    // jbotci's source-defined identity formula, not a candidate query response.
    let jbotci_revision = "1111111111111111111111111111111111111111";
    let metadata_sha256 = sha256(metadata.as_bytes());
    let release_material = format!(
        "vlajmaji-jbotci-release-v1\0{jbotci_revision}\0{}\0{metadata_sha256}",
        sha256(LENSISKU),
    );
    let jbotci_release = format!("jbotci-release-{}", sha256(release_material.as_bytes()));
    let capture = CaptureRef {
        namespace: "captured:lensisku:jbotci:en".into(),
        sha256: sha256(LENSISKU),
    };
    let catalog_target = |native_entry_id: Option<&str>, native_assertion_id: Option<&str>| {
        RelationTarget::Catalog(CatalogTarget {
            source: SourceId::new("lensisku").unwrap(),
            release: vlajmaji_lexicon::EntityId::new(jbotci_release.clone()).unwrap(),
            native_entry_id: native_entry_id.map(str::to_owned),
            native_assertion_id: native_assertion_id.map(str::to_owned),
            occurrence: Some(OccurrenceRef {
                capture: capture.clone(),
                locator: "/0".into(),
            }),
        })
    };
    let resolved = engine
        .execute(DictionaryRequest::Resolve {
            target: catalog_target(None, Some("333")),
        })
        .unwrap();
    let CollectionQueryResult::Resolve(resolved) = resolved.result else {
        panic!("wrong reference result");
    };
    assert_eq!(resolved.status, vlajmaji_search::ReferenceStatus::Resolved);
    assert_eq!(resolved.catalogue_matches.len(), 1);
    assert_eq!(
        resolved.catalogue_matches[0].model_release_id,
        jbotci_release
    );
    assert!(resolved.payload.is_some());

    let no_match = engine
        .execute(DictionaryRequest::Resolve {
            target: catalog_target(None, Some("334")),
        })
        .unwrap();
    let CollectionQueryResult::Resolve(no_match) = no_match.result else {
        panic!("wrong reference result");
    };
    assert_eq!(no_match.status, vlajmaji_search::ReferenceStatus::NoMatch);
    assert!(no_match.payload.is_none());
    assert!(no_match.requested_release.is_some());

    let unverified = engine
        .execute(DictionaryRequest::Resolve {
            target: catalog_target(Some("333"), None),
        })
        .unwrap();
    let CollectionQueryResult::Resolve(unverified) = unverified.result else {
        panic!("wrong reference result");
    };
    assert_eq!(
        unverified.status,
        vlajmaji_search::ReferenceStatus::Unverified
    );

    let assertion_id =
        vlajmaji_lexicon::EntityId::new(format!("jbotci-{}-assertion-0", sha256(LENSISKU)))
            .unwrap();
    let wrong_local_role = engine
        .execute(DictionaryRequest::Resolve {
            target: RelationTarget::Local(LocalRef::Entry(assertion_id.clone())),
        })
        .unwrap_err();
    assert!(matches!(
        wrong_local_role,
        vlajmaji_search::CollectionQueryError::Query(
            vlajmaji_search::DictionaryQueryError::InvalidReference { .. }
        )
    ));
    let valid_local_role = engine
        .execute(DictionaryRequest::Resolve {
            target: RelationTarget::Local(LocalRef::Assertion(assertion_id)),
        })
        .unwrap();
    let CollectionQueryResult::Resolve(valid_local_role) = valid_local_role.result else {
        panic!("wrong reference result");
    };
    assert!(valid_local_role.payload.is_some());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn test_managed_exact_matches_existing_unpaged_wire_and_wraps_page_cursor() {
    let root = fresh_root("exact");
    let generation = install_fixture(&root);
    let manifest = generation.snapshot_manifest();
    let mut managed = CollectionSearchEngine::open(&root, selection()).unwrap();
    let response = managed
        .execute(DictionaryRequest::Exact {
            spelling: "farvi".into(),
            options: DictionaryExactOptions::default(),
        })
        .unwrap();
    let CollectionQueryResult::Exact(managed_exact) = response.result else {
        panic!("wrong operation result");
    };
    let baseline = DictionarySearchEngine::open_snapshot(DictionaryReadTopology::Unified, manifest)
        .unwrap()
        .exact("farvi", DictionaryExactOptions::default())
        .unwrap();
    assert_eq!(*managed_exact, baseline);

    let first = managed
        .execute(DictionaryRequest::Exact {
            spelling: "farvi".into(),
            options: DictionaryExactOptions {
                page_size: Some(1),
                ..Default::default()
            },
        })
        .unwrap();
    let CollectionQueryResult::Exact(first_page) = first.result else {
        panic!("wrong operation result");
    };
    assert!(
        first_page
            .next_cursor
            .as_deref()
            .is_some_and(|cursor| cursor.starts_with("collection-v1:"))
    );
    let mut next = CollectionSearchEngine::open(&root, selection()).unwrap();
    let second = next
        .execute(DictionaryRequest::Exact {
            spelling: "farvi".into(),
            options: DictionaryExactOptions {
                page_size: Some(1),
                cursor: first_page.next_cursor.clone(),
                ..Default::default()
            },
        })
        .unwrap();
    let CollectionQueryResult::Exact(second_page) = second.result else {
        panic!("wrong operation result");
    };
    assert_eq!(first_page.totals, second_page.totals);
    assert_ne!(first_page.groups, second_page.groups);
    let valid_local = managed
        .execute(DictionaryRequest::Resolve {
            target: RelationTarget::Local(LocalRef::Entry(
                vlajmaji_lexicon::EntityId::new("entry-a-farvi").unwrap(),
            )),
        })
        .unwrap();
    assert!(matches!(
        valid_local.result,
        CollectionQueryResult::Resolve(_)
    ));
    let absent_local = managed
        .execute(DictionaryRequest::Resolve {
            target: RelationTarget::Local(LocalRef::Entry(
                vlajmaji_lexicon::EntityId::new("missing-entry").unwrap(),
            )),
        })
        .unwrap_err();
    assert!(matches!(
        absent_local,
        vlajmaji_search::CollectionQueryError::Query(
            vlajmaji_search::DictionaryQueryError::InvalidReference { .. }
        )
    ));
    register_pack(&root, PackId::LaLojban);
    let retained_reader = managed
        .execute(DictionaryRequest::Exact {
            spelling: "farvi".into(),
            options: DictionaryExactOptions::default(),
        })
        .unwrap();
    let CollectionQueryResult::Exact(retained_exact) = retained_reader.result else {
        panic!("wrong operation result");
    };
    assert_eq!(
        *retained_exact, baseline,
        "open reader keeps its captured generation across a catalogue writer"
    );
    let mut changed_catalogue = CollectionSearchEngine::open(&root, selection()).unwrap();
    let stale_catalogue_cursor = changed_catalogue.execute(DictionaryRequest::Exact {
        spelling: "farvi".into(),
        options: DictionaryExactOptions {
            page_size: Some(1),
            cursor: first_page.next_cursor,
            ..Default::default()
        },
    });
    assert!(matches!(
        stale_catalogue_cursor,
        Err(vlajmaji_search::CollectionQueryError::StaleCursor)
    ));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn test_config_change_stales_cursor_before_disabled_selection_admission() {
    let root = fresh_root("config-stale-cursor");
    install_fixture(&root);
    let config = root.join("profiles.toml");
    CollectionProfiles::configure(
        &config,
        CollectionProfileWriteRequest {
            name: "research".into(),
            root: root.clone(),
            packs: vec![PackId::Llg],
            disabled: vec![],
            make_default: true,
        },
    )
    .unwrap();
    let first_snapshot = CollectionProfiles::read(&config).unwrap();
    let mut first_engine =
        CollectionSearchEngine::open_configured(&first_snapshot, None, None).unwrap();
    let first_response = first_engine
        .execute(DictionaryRequest::Exact {
            spelling: "farvi".into(),
            options: DictionaryExactOptions {
                page_size: Some(1),
                ..Default::default()
            },
        })
        .unwrap();
    let CollectionQueryResult::Exact(first_page) = first_response.result else {
        panic!("wrong operation result");
    };
    let cursor = first_page
        .next_cursor
        .expect("fixture should have another farvi page");

    let original_config = fs::read(&config).unwrap();
    let mut comment_only_change = original_config.clone();
    comment_only_change.extend_from_slice(b"\n# same policy, different exact config bytes\n");
    fs::write(&config, &comment_only_change).unwrap();
    let byte_changed_snapshot = CollectionProfiles::read(&config).unwrap();
    let mut byte_changed_engine =
        CollectionSearchEngine::open_configured(&byte_changed_snapshot, None, None).unwrap();
    let byte_changed_cursor = byte_changed_engine.execute(DictionaryRequest::Exact {
        spelling: "farvi".into(),
        options: DictionaryExactOptions {
            page_size: Some(1),
            cursor: Some(cursor.clone()),
            ..Default::default()
        },
    });
    assert!(matches!(
        byte_changed_cursor,
        Err(vlajmaji_search::CollectionQueryError::StaleCursor)
    ));

    fs::write(&config, &original_config).unwrap();
    let restored_snapshot = CollectionProfiles::read(&config).unwrap();
    let mut restored_engine =
        CollectionSearchEngine::open_configured(&restored_snapshot, None, None).unwrap();
    let restored_page = restored_engine
        .execute(DictionaryRequest::Exact {
            spelling: "farvi".into(),
            options: DictionaryExactOptions {
                page_size: Some(1),
                cursor: Some(cursor.clone()),
                ..Default::default()
            },
        })
        .unwrap();
    let CollectionQueryResult::Exact(restored_page) = restored_page.result else {
        panic!("wrong operation result");
    };
    assert!(!restored_page.groups.is_empty());

    // Change the disabled policy so a fresh engine would have no enabled
    // packs. The stale cursor must be rejected before that admission result.
    CollectionProfiles::configure(
        &config,
        CollectionProfileWriteRequest {
            name: "research".into(),
            root: root.clone(),
            packs: vec![PackId::Llg],
            disabled: vec![PackId::Llg],
            make_default: true,
        },
    )
    .unwrap();
    let changed_snapshot = CollectionProfiles::read(&config).unwrap();
    let mut changed_engine =
        CollectionSearchEngine::open_configured(&changed_snapshot, None, None).unwrap();
    let error = changed_engine
        .execute(DictionaryRequest::Exact {
            spelling: "farvi".into(),
            options: DictionaryExactOptions {
                page_size: Some(1),
                cursor: Some(cursor),
                ..Default::default()
            },
        })
        .unwrap_err();
    assert!(matches!(
        error,
        vlajmaji_search::CollectionQueryError::StaleCursor
    ));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn test_missing_derived_state_stales_cursor_and_exact_restore_accepts_it() {
    use vlajmaji_store::{DictionaryStoreError, DictionaryStoreInvalidity};

    let root = fresh_root("derived-stale-cursor");
    let generation = install_fixture(&root);
    let config = root.join("profiles.toml");
    CollectionProfiles::configure(
        &config,
        CollectionProfileWriteRequest {
            name: "research".into(),
            root: root.clone(),
            packs: vec![PackId::Llg],
            disabled: vec![],
            make_default: true,
        },
    )
    .unwrap();
    let original_database = fs::read(generation.store_path.join("dictionary.sqlite3")).unwrap();

    let original_profiles = CollectionProfiles::read(&config).unwrap();
    let mut original_engine =
        CollectionSearchEngine::open_configured(&original_profiles, None, None).unwrap();
    let first = original_engine
        .execute(DictionaryRequest::Exact {
            spelling: "farvi".into(),
            options: DictionaryExactOptions {
                page_size: Some(1),
                ..Default::default()
            },
        })
        .unwrap();
    let CollectionQueryResult::Exact(first_page) = first.result else {
        panic!("wrong result kind");
    };
    let cursor = first_page
        .next_cursor
        .expect("fixture should have another farvi hit");

    fs::remove_file(generation.store_path.join("dictionary.sqlite3")).unwrap();
    let missing_profiles = CollectionProfiles::read(&config).unwrap();
    let mut missing_engine =
        CollectionSearchEngine::open_configured(&missing_profiles, None, None).unwrap();
    let stale_before_payload = missing_engine.execute(DictionaryRequest::Exact {
        spelling: "farvi".into(),
        options: DictionaryExactOptions {
            page_size: Some(1),
            cursor: Some(cursor.clone()),
            ..Default::default()
        },
    });
    assert!(matches!(
        stale_before_payload,
        Err(CollectionQueryError::StaleCursor)
    ));
    let no_cursor = missing_engine
        .execute(DictionaryRequest::Exact {
            spelling: "farvi".into(),
            options: DictionaryExactOptions::default(),
        })
        .unwrap_err();
    assert!(matches!(
        no_cursor,
        CollectionQueryError::Query(vlajmaji_search::DictionaryQueryError::Snapshot(
            vlajmaji_store::DictionarySnapshotError::Store(DictionaryStoreError::InvalidStore {
                reason: DictionaryStoreInvalidity::MissingDatabase,
                ..
            })
        ))
    ));

    fs::write(
        generation.store_path.join("dictionary.sqlite3"),
        &original_database,
    )
    .unwrap();
    let restored_profiles = CollectionProfiles::read(&config).unwrap();
    let mut restored_engine =
        CollectionSearchEngine::open_configured(&restored_profiles, None, None).unwrap();
    let restored = restored_engine
        .execute(DictionaryRequest::Exact {
            spelling: "farvi".into(),
            options: DictionaryExactOptions {
                page_size: Some(1),
                cursor: Some(cursor),
                ..Default::default()
            },
        })
        .unwrap();
    assert!(matches!(restored.result, CollectionQueryResult::Exact(_)));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn test_metadata_only_unknown_and_unresolved_do_not_require_payload() {
    let root = fresh_root("metadata-only");
    // A registered-only pack has no Unified payload; non-target references remain cheap.
    let mut document = fixture::complete_fixture().unwrap().into_document();
    document.legacy_archive = None;
    let dictionary = Dictionary::from_document(document).unwrap();
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
    CollectionCatalogue::register(&root, request).unwrap();
    let mut engine = CollectionSearchEngine::open(&root, selection()).unwrap();
    assert_eq!(
        engine.selection().collections[0].derived_state,
        "registered_only"
    );
    for (target, expected) in [
        (vlajmaji_lexicon::RelationTarget::Unknown, "Unknown"),
        (
            vlajmaji_lexicon::RelationTarget::Unresolved("external label".into()),
            "Unresolved",
        ),
    ] {
        let response = engine
            .execute(DictionaryRequest::Resolve { target })
            .unwrap();
        let CollectionQueryResult::Resolve(resolution) = response.result else {
            panic!("wrong operation result");
        };
        assert_eq!(format!("{:?}", resolution.status), expected);
        assert!(resolution.payload.is_none());
    }
    let registered_only = engine
        .execute(DictionaryRequest::Exact {
            spelling: "farvi".into(),
            options: DictionaryExactOptions::default(),
        })
        .unwrap_err();
    assert!(matches!(
        registered_only,
        vlajmaji_search::CollectionQueryError::UnavailablePack { ref reason, .. }
            if reason == "registered_only"
    ));

    let mut disabled = CollectionSearchEngine::open(
        &root,
        CollectionQuerySelection {
            packs: vec![PackId::Llg],
            disabled: vec![PackId::Llg],
            profile: None,
        },
    )
    .unwrap();
    assert!(disabled.selection().collections[0].disabled);
    assert!(matches!(
        disabled.execute(DictionaryRequest::Exact {
            spelling: "farvi".into(),
            options: DictionaryExactOptions::default(),
        }),
        Err(vlajmaji_search::CollectionQueryError::NoEnabledPacks)
    ));

    let mut unselected = CollectionSearchEngine::open(
        &root,
        CollectionQuerySelection {
            packs: vec![],
            disabled: vec![],
            profile: None,
        },
    )
    .unwrap();
    assert!(!unselected.selection().collections[0].request_selected);
    assert!(matches!(
        unselected.execute(DictionaryRequest::Exact {
            spelling: "farvi".into(),
            options: DictionaryExactOptions::default(),
        }),
        Err(vlajmaji_search::CollectionQueryError::NoEnabledPacks)
    ));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn test_resolve_uses_selected_payload_when_an_unrelated_pack_is_registered_only() {
    let root = fresh_root("selected-reference-unrelated-unavailable");
    install_fixture(&root);
    register_pack(&root, PackId::LaLojban);
    let mut engine = CollectionSearchEngine::open(
        &root,
        CollectionQuerySelection {
            packs: vec![PackId::Llg, PackId::LaLojban],
            disabled: vec![],
            profile: None,
        },
    )
    .unwrap();

    let exact_error = engine
        .execute(DictionaryRequest::Exact {
            spelling: "farvi".into(),
            options: DictionaryExactOptions::default(),
        })
        .unwrap_err();
    assert!(matches!(
        exact_error,
        CollectionQueryError::UnavailablePack { ref pack_id, ref reason }
            if pack_id == "la-lojban" && reason == "registered_only"
    ));

    let filtered_error = engine
        .execute(DictionaryRequest::Exact {
            spelling: "farvi".into(),
            options: DictionaryExactOptions {
                sources: vec!["llg".into()],
                ..DictionaryExactOptions::default()
            },
        })
        .unwrap_err();
    assert!(matches!(
        filtered_error,
        CollectionQueryError::UnavailablePack { ref pack_id, ref reason }
            if pack_id == "la-lojban" && reason == "registered_only"
    ));

    let resolved = engine
        .execute(DictionaryRequest::Resolve {
            target: RelationTarget::Local(LocalRef::Entry(
                vlajmaji_lexicon::EntityId::new("entry-a-farvi").unwrap(),
            )),
        })
        .unwrap();
    let CollectionQueryResult::Resolve(resolved) = resolved.result else {
        panic!("wrong operation result");
    };
    assert!(resolved.payload.is_some());
    assert_eq!(resolved.catalogue_matches.len(), 1);
    assert_eq!(resolved.catalogue_matches[0].pack_id, "llg");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn test_missing_installed_copy_is_advisory_and_accessor_stays_metadata_only() {
    let root = fresh_root("missing-copy");
    let generation = install_fixture(&root);
    let manifest = generation.snapshot_manifest();
    let catalogue = CollectionCatalogue::open(&root).unwrap();
    let locations = catalogue
        .installed_artifact_locations(PackId::Llg)
        .unwrap()
        .unwrap();
    fs::remove_file(&locations.release_path).unwrap();

    // The path is returned from catalogue metadata even though the release file is absent.
    let locations_after_removal = CollectionCatalogue::open(&root)
        .unwrap()
        .installed_artifact_locations(PackId::Llg)
        .unwrap()
        .unwrap();
    assert_eq!(locations_after_removal.release_path, locations.release_path);

    let mut managed = CollectionSearchEngine::open(&root, selection()).unwrap();
    let pack = managed
        .selection()
        .collections
        .iter()
        .find(|row| row.pack_id == "llg")
        .unwrap();
    assert_eq!(pack.owned_files_present, Some(false));
    assert_eq!(
        pack.owned_files_warning.as_deref(),
        Some("installed_copy_missing")
    );
    let response = managed
        .execute(DictionaryRequest::Exact {
            spelling: "farvi".into(),
            options: DictionaryExactOptions::default(),
        })
        .unwrap();
    let CollectionQueryResult::Exact(exact) = response.result else {
        panic!("wrong operation result");
    };

    // File maintenance observations must not stale the cursor or make the healthy DB unusable.
    let baseline = DictionarySearchEngine::open_snapshot(DictionaryReadTopology::Unified, manifest)
        .unwrap()
        .exact("farvi", DictionaryExactOptions::default())
        .unwrap();
    assert_eq!(*exact, baseline);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn test_selected_installed_catalog_reference_reports_typed_payload_failure() {
    use vlajmaji_store::{
        DictionarySnapshotError, DictionaryStoreError, DictionaryStoreInvalidity,
    };
    let root = fresh_root("selected-payload-failure");
    let generation = install_fixture(&root);
    let model = fixture::complete_fixture().unwrap();
    let capture = model
        .captures()
        .iter()
        .find(|capture| capture.identity.namespace == "captured:a:d1")
        .unwrap();
    let target = RelationTarget::Catalog(CatalogTarget {
        source: SourceId::new("source-a").unwrap(),
        release: vlajmaji_lexicon::EntityId::new("release-a").unwrap(),
        native_entry_id: None,
        native_assertion_id: None,
        occurrence: Some(OccurrenceRef {
            capture: CaptureRef {
                namespace: capture.identity.namespace.clone(),
                sha256: capture.identity.sha256.clone(),
            },
            locator: "/0".into(),
        }),
    });
    fs::remove_dir_all(&generation.store_path).unwrap();
    let mut managed = CollectionSearchEngine::open(&root, selection()).unwrap();
    let pack = managed
        .selection()
        .collections
        .iter()
        .find(|row| row.pack_id == "llg")
        .unwrap();
    assert_eq!(pack.derived_state, "minimum_missing");
    let error = managed
        .execute(DictionaryRequest::Resolve { target })
        .unwrap_err();
    assert!(
        matches!(
            &error,
            vlajmaji_search::CollectionQueryError::Query(
                vlajmaji_search::DictionaryQueryError::Snapshot(DictionarySnapshotError::Store(
                    DictionaryStoreError::InvalidStore {
                        reason: DictionaryStoreInvalidity::MissingDatabase,
                        ..
                    }
                ))
            )
        ),
        "{error}"
    );
    let exact_error = managed
        .execute(DictionaryRequest::Exact {
            spelling: "farvi".into(),
            options: DictionaryExactOptions::default(),
        })
        .unwrap_err();
    assert!(matches!(
        exact_error,
        vlajmaji_search::CollectionQueryError::Query(
            vlajmaji_search::DictionaryQueryError::Snapshot(DictionarySnapshotError::Store(
                DictionaryStoreError::InvalidStore {
                    reason: DictionaryStoreInvalidity::MissingDatabase,
                    ..
                }
            ))
        )
    ));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn test_selected_corrupt_derived_store_is_not_reported_as_catalogue_only() {
    use vlajmaji_store::DictionarySnapshotError;
    let root = fresh_root("corrupt-derived-payload");
    let generation = install_fixture(&root);
    let model = fixture::complete_fixture().unwrap();
    let capture = model
        .captures()
        .iter()
        .find(|capture| capture.identity.namespace == "captured:a:d1")
        .unwrap();
    let target = RelationTarget::Catalog(CatalogTarget {
        source: SourceId::new("source-a").unwrap(),
        release: vlajmaji_lexicon::EntityId::new("release-a").unwrap(),
        native_entry_id: None,
        native_assertion_id: None,
        occurrence: Some(OccurrenceRef {
            capture: CaptureRef {
                namespace: capture.identity.namespace.clone(),
                sha256: capture.identity.sha256.clone(),
            },
            locator: "/0".into(),
        }),
    });
    fs::write(
        generation.store_path.join("dictionary.sqlite3"),
        b"not a sqlite database",
    )
    .unwrap();
    let mut managed = CollectionSearchEngine::open(&root, selection()).unwrap();
    assert_eq!(
        managed.selection().collections[0].derived_state,
        "minimum_invalid"
    );
    let error = managed
        .execute(DictionaryRequest::Resolve { target })
        .unwrap_err();
    assert!(matches!(
        error,
        vlajmaji_search::CollectionQueryError::Query(
            vlajmaji_search::DictionaryQueryError::Snapshot(DictionarySnapshotError::Store(_))
        )
    ));
    let exact_error = managed
        .execute(DictionaryRequest::Exact {
            spelling: "farvi".into(),
            options: DictionaryExactOptions::default(),
        })
        .unwrap_err();
    assert!(matches!(
        exact_error,
        vlajmaji_search::CollectionQueryError::Query(
            vlajmaji_search::DictionaryQueryError::Snapshot(DictionarySnapshotError::Store(_))
        )
    ));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn test_selected_unsupported_derived_version_is_typed_for_exact_and_resolve() {
    use vlajmaji_store::{DictionaryStoreError, DictionaryStoreInvalidity};

    let root = fresh_root("unsupported-derived-version");
    let generation = install_fixture(&root);
    let model = fixture::complete_fixture().unwrap();
    let capture = model
        .captures()
        .iter()
        .find(|capture| capture.identity.namespace == "captured:a:d1")
        .unwrap();
    let target = RelationTarget::Catalog(CatalogTarget {
        source: SourceId::new("source-a").unwrap(),
        release: vlajmaji_lexicon::EntityId::new("release-a").unwrap(),
        native_entry_id: None,
        native_assertion_id: None,
        occurrence: Some(OccurrenceRef {
            capture: CaptureRef {
                namespace: capture.identity.namespace.clone(),
                sha256: capture.identity.sha256.clone(),
            },
            locator: "/0".into(),
        }),
    });

    // The unchanged fixture is a valid counterpart for both operations.
    let mut valid = CollectionSearchEngine::open(&root, selection()).unwrap();
    assert!(
        valid
            .execute(DictionaryRequest::Exact {
                spelling: "farvi".into(),
                options: DictionaryExactOptions::default(),
            })
            .is_ok()
    );
    assert!(
        valid
            .execute(DictionaryRequest::Resolve {
                target: target.clone(),
            })
            .is_ok()
    );
    drop(valid);

    let database = generation.store_path.join("dictionary.sqlite3");
    let mutation = Command::new("sqlite3")
        .arg(&database)
        .arg("UPDATE metadata SET value='999' WHERE key='schema_version';")
        .output()
        .unwrap();
    assert!(
        mutation.status.success(),
        "sqlite3 version mutation failed: {}",
        String::from_utf8_lossy(&mutation.stderr)
    );

    let mut managed = CollectionSearchEngine::open(&root, selection()).unwrap();
    assert_eq!(
        managed.selection().collections[0].derived_state,
        "minimum_unsupported_version"
    );
    let resolve_error = managed
        .execute(DictionaryRequest::Resolve { target })
        .unwrap_err();
    assert!(
        matches!(
            &resolve_error,
            CollectionQueryError::Query(vlajmaji_search::DictionaryQueryError::Snapshot(
                vlajmaji_store::DictionarySnapshotError::Store(
                    DictionaryStoreError::InvalidStore {
                        reason: DictionaryStoreInvalidity::SchemaVersion,
                        ..
                    }
                )
            ))
        ),
        "unsupported version should retain its typed cause: {resolve_error:?}"
    );
    let exact_error = managed
        .execute(DictionaryRequest::Exact {
            spelling: "farvi".into(),
            options: DictionaryExactOptions::default(),
        })
        .unwrap_err();
    assert!(matches!(
        exact_error,
        CollectionQueryError::Query(vlajmaji_search::DictionaryQueryError::Snapshot(
            vlajmaji_store::DictionarySnapshotError::Store(DictionaryStoreError::InvalidStore {
                reason: DictionaryStoreInvalidity::SchemaVersion,
                ..
            })
        ))
    ));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn test_stale_reference_snapshot_wins_before_corrupt_heavy_metadata() {
    use vlajmaji_store::CollectionError;

    let root = fresh_root("stale-heavy-metadata");
    let model = fixture::complete_fixture().unwrap();
    let capture = model
        .captures()
        .iter()
        .find(|capture| capture.identity.namespace == "captured:a:d1")
        .unwrap();
    install_fixture(&root);
    let mut old_reader = CollectionSearchEngine::open(&root, selection()).unwrap();
    let target = RelationTarget::Catalog(CatalogTarget {
        source: SourceId::new("source-a").unwrap(),
        release: vlajmaji_lexicon::EntityId::new("release-a").unwrap(),
        native_entry_id: None,
        native_assertion_id: None,
        occurrence: Some(OccurrenceRef {
            capture: CaptureRef {
                namespace: capture.identity.namespace.clone(),
                sha256: capture.identity.sha256.clone(),
            },
            locator: "/0".into(),
        }),
    });

    let mutation = Command::new("sqlite3")
        .arg(root.join("catalogue.sqlite3"))
        .arg("UPDATE catalogue_meta_v2 SET value='2' WHERE key='local_revision'; UPDATE registrations_v2 SET descriptor_json=x'00' WHERE pack_id='llg';")
        .output()
        .unwrap();
    assert!(
        mutation.status.success(),
        "sqlite3 stale-metadata mutation failed: {}",
        String::from_utf8_lossy(&mutation.stderr)
    );

    let stale = old_reader
        .execute(DictionaryRequest::Resolve {
            target: target.clone(),
        })
        .unwrap_err();
    assert!(matches!(
        stale,
        CollectionQueryError::Catalogue(CollectionError::InvalidRequest(ref detail))
            if detail == "stale_snapshot: catalogue revision changed"
    ));

    let mut current = CollectionSearchEngine::open(&root, selection()).unwrap();
    let corrupt_current = current.execute(DictionaryRequest::Resolve { target });
    assert!(
        matches!(corrupt_current, Err(CollectionQueryError::Catalogue(_))),
        "fresh snapshot must observe the corrupted heavy descriptor instead of hiding it"
    );
    fs::remove_dir_all(root).unwrap();
}
