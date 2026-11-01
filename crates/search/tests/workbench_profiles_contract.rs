//! Public configured-profile query and freshness contracts.

use std::{
    borrow::Cow,
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
};

use sha2::{Digest, Sha256};
use vlajmaji_lexicon::{
    Dictionary, Eligibility, EntityId, LaLojbanImportOptions, LocalRef, PackId, PackPayloadKind,
    RelationTarget, import_la_lojban_export, seal_release,
};
use vlajmaji_search::{
    ConfiguredWorkbenchQueryError, ConfiguredWorkbenchSearchEngine, DictionaryExactOptions,
    DictionaryRequest, WorkbenchFullResult, WorkbenchQueryError,
};
use vlajmaji_store::{
    CollectionCatalogue, CollectionInput, CollectionRequest, CollectionSelection,
    WorkbenchProfileConfigureRequest, WorkbenchProfiles, WorkbenchSelection,
};

static NEXT_ROOT: AtomicUsize = AtomicUsize::new(0);

fn fresh_root() -> PathBuf {
    for _ in 0..64 {
        let root = std::env::temp_dir().join(format!(
            "vlajmaji-workbench-configured-search-{}-{}",
            std::process::id(),
            NEXT_ROOT.fetch_add(1, Ordering::Relaxed)
        ));
        match fs::create_dir(&root) {
            Ok(()) => return root,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => panic!("create exclusive configured-search fixture: {error}"),
        }
    }
    panic!("could not allocate an exclusive configured-search fixture")
}

fn input_dictionary() -> Dictionary {
    let raw = br#"[
        {"word":"farvi","definition":"advance","word_type":"gismu"},
        {"word":"farvi","definition":"advance","word_type":"gismu"}
    ]"#;
    let expected_raw_sha256 = format!("{:x}", Sha256::digest(raw));
    import_la_lojban_export(
        raw,
        &LaLojbanImportOptions {
            revision: "a".repeat(40),
            expected_raw_sha256,
            raw_locator: Some("https://example.invalid/search-contract.json".into()),
            notice_locator: Some("https://example.invalid/search-contract-notice".into()),
        },
    )
    .expect("self-contained two-occurrence search input validates")
    .into_dictionary()
}

fn input_assertion_for_locator<'a>(
    dictionary: &'a Dictionary,
    locator: &str,
) -> (
    &'a vlajmaji_lexicon::AssertionRecord,
    &'a vlajmaji_lexicon::EntryRecord,
) {
    let assertion = dictionary
        .metadata()
        .assertions
        .iter()
        .find(|assertion| assertion.occurrence.locator == locator)
        .expect("pre-input assertion for the requested capture locator");
    let entry = dictionary
        .metadata()
        .entries
        .iter()
        .find(|entry| entry.id == assertion.entry)
        .expect("pre-input entry for the requested assertion");
    (assertion, entry)
}

fn install(root: &Path, dictionary: &Dictionary) {
    let sealed = seal_release(PackId::Llg, PackPayloadKind::Synthetic, dictionary).unwrap();
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

fn selection() -> WorkbenchSelection {
    WorkbenchSelection {
        version: 1,
        packs: vec![PackId::Llg],
        disabled: Vec::new(),
        owned: None,
        contexts: Vec::new(),
    }
}

fn configure(name: &str, catalogue: &Path, make_default: bool) -> WorkbenchProfileConfigureRequest {
    WorkbenchProfileConfigureRequest {
        name: name.into(),
        catalogue_root: Some(catalogue.to_path_buf()),
        owned_root: None,
        selection: selection(),
        make_default,
    }
}

#[test]
fn test_configured_search_keeps_full_compact_detail_and_invalidates_on_config_change() {
    let root = fresh_root();
    let dictionary = input_dictionary();
    let input_locators: Vec<_> = dictionary
        .metadata()
        .assertions
        .iter()
        .map(|assertion| assertion.occurrence.locator.as_str())
        .collect();
    assert_eq!(input_locators, ["/0", "/1"]);
    assert_eq!(dictionary.metadata().assertions.len(), 2);
    assert!(
        dictionary
            .metadata()
            .assertions
            .iter()
            .all(|assertion| assertion.eligibility == Eligibility::Lexical)
    );
    let catalogue = root.join("catalogue");
    install(&catalogue, &dictionary);
    let config = root.join("profiles.json");
    let first =
        WorkbenchProfiles::configure(&config, configure("alpha", &catalogue, true), None).unwrap();
    assert_eq!(first.revision, 1);

    let destination = root.join("alpha-workbench");
    let refreshed = WorkbenchProfiles::refresh(&config, None, &destination, 1).unwrap();
    assert_eq!(refreshed.publication.revision, 2);
    let mut engine = ConfiguredWorkbenchSearchEngine::open(&config, None).unwrap();
    let identity = engine.identity().clone();
    assert_eq!(identity.requested_profile, None);
    assert_eq!(identity.profile, "alpha");
    let named_alpha = ConfiguredWorkbenchSearchEngine::open(&config, Some("alpha")).unwrap();
    assert_eq!(named_alpha.identity().profile, identity.profile);
    assert_eq!(named_alpha.identity().config_sha256, identity.config_sha256);
    assert_ne!(
        named_alpha.identity().effective_binding_sha256,
        identity.effective_binding_sha256,
        "default and explicitly named modes are distinct effective identities",
    );

    let request = DictionaryRequest::Exact {
        spelling: "farvi".into(),
        options: DictionaryExactOptions {
            page_size: Some(1),
            ..DictionaryExactOptions::default()
        },
    };
    let full = engine.execute(request.clone()).unwrap();
    assert_eq!(full.binding_sha256, identity.effective_binding_sha256);
    assert_eq!(full.configuration, identity);
    let WorkbenchFullResult::Exact(full_exact) = &full.result else {
        panic!("configured exact query must retain its Full Exact tag")
    };
    assert_eq!(full_exact.groups.len(), 1);
    assert_eq!(full_exact.groups[0].hits.len(), 1);
    assert_eq!(full_exact.totals.occurrences, 2);
    assert_eq!(full_exact.source_totals[0].occurrences, 2);
    assert_eq!(full_exact.groups[0].full_count, 2);
    assert_eq!(full_exact.groups[0].returned_count, 1);
    let full_cursor = full_exact
        .next_cursor
        .clone()
        .expect("page-size-one Full result must issue a cursor");
    let (first_input_assertion, first_input_entry) = input_assertion_for_locator(&dictionary, "/0");
    assert_eq!(
        full_exact.groups[0].hits[0].assertion,
        *first_input_assertion
    );
    assert_eq!(full_exact.groups[0].hits[0].entry, *first_input_entry);

    let compact = engine.execute_compact(request).unwrap();
    assert_eq!(compact.binding_sha256, full.binding_sha256);
    assert_eq!(compact.configuration, full.configuration);
    let vlajmaji_search::DictionaryCompactQueryResult::Exact(compact_exact) = &compact.result
    else {
        panic!("configured exact query must retain its Compact Exact tag")
    };
    assert_eq!(compact_exact.groups.len(), 1);
    assert_eq!(compact_exact.groups[0].hits.len(), 1);
    assert_eq!(compact_exact.totals.occurrences, 2);
    assert_eq!(compact_exact.source_totals[0].occurrences, 2);
    assert_eq!(compact_exact.groups[0].full_count, 2);
    assert_eq!(compact_exact.groups[0].returned_count, 1);
    let compact_cursor = compact_exact
        .next_cursor
        .clone()
        .expect("page-size-one Compact result must issue a cursor");
    assert_eq!(
        compact_exact.groups[0].hits[0].assertion,
        *first_input_assertion
    );
    assert_eq!(compact_exact.groups[0].hits[0].entry, *first_input_entry);

    let next_full = engine
        .execute(DictionaryRequest::Exact {
            spelling: "farvi".into(),
            options: DictionaryExactOptions {
                page_size: Some(1),
                cursor: Some(full_cursor.clone()),
                ..DictionaryExactOptions::default()
            },
        })
        .unwrap();
    assert_eq!(next_full.configuration, identity);
    assert_eq!(next_full.binding_sha256, identity.effective_binding_sha256);
    let WorkbenchFullResult::Exact(next_full_exact) = next_full.result else {
        panic!("same-binding Full continuation retains its Exact tag")
    };
    let (next_input_assertion, next_input_entry) = input_assertion_for_locator(&dictionary, "/1");
    assert_eq!(next_full_exact.groups.len(), 1);
    assert_eq!(next_full_exact.groups[0].hits.len(), 1);
    assert_eq!(next_full_exact.totals.occurrences, 2);
    assert_eq!(next_full_exact.source_totals[0].occurrences, 2);
    assert_eq!(next_full_exact.groups[0].full_count, 2);
    assert_eq!(next_full_exact.groups[0].returned_count, 1);
    assert_eq!(
        next_full_exact.groups[0].hits[0].assertion,
        *next_input_assertion
    );
    assert_eq!(next_full_exact.groups[0].hits[0].entry, *next_input_entry);
    assert!(next_full_exact.next_cursor.is_none());

    let next_compact = engine
        .execute_compact(DictionaryRequest::Exact {
            spelling: "farvi".into(),
            options: DictionaryExactOptions {
                page_size: Some(1),
                cursor: Some(compact_cursor.clone()),
                ..DictionaryExactOptions::default()
            },
        })
        .unwrap();
    assert_eq!(next_compact.configuration, identity);
    assert_eq!(
        next_compact.binding_sha256,
        identity.effective_binding_sha256
    );
    let vlajmaji_search::DictionaryCompactQueryResult::Exact(next_compact_exact) =
        next_compact.result
    else {
        panic!("same-binding Compact continuation retains its Exact tag")
    };
    assert_eq!(next_compact_exact.groups.len(), 1);
    assert_eq!(next_compact_exact.groups[0].hits.len(), 1);
    assert_eq!(next_compact_exact.totals.occurrences, 2);
    assert_eq!(next_compact_exact.source_totals[0].occurrences, 2);
    assert_eq!(next_compact_exact.groups[0].full_count, 2);
    assert_eq!(next_compact_exact.groups[0].returned_count, 1);
    assert_eq!(
        next_compact_exact.groups[0].hits[0].assertion,
        *next_input_assertion
    );
    assert_eq!(
        next_compact_exact.groups[0].hits[0].entry,
        *next_input_entry
    );
    assert!(next_compact_exact.next_cursor.is_none());
    let full_hit = &full_exact.groups[0].hits[0];
    let detail = engine
        .detail(
            &compact_exact.groups[0].hits[0].detail_ref,
            &identity.effective_binding_sha256,
        )
        .unwrap();
    assert_eq!(detail.binding_sha256, full.binding_sha256);
    assert_eq!(detail.result.hit.as_ref(), Some(full_hit));
    assert_eq!(detail.result.snapshot_sha256, compact_exact.snapshot_sha256);
    assert_ne!(detail.binding_sha256, detail.result.snapshot_sha256);
    assert!(matches!(
        engine.detail(&compact_exact.groups[0].hits[0].detail_ref, "not-a-digest"),
        Err(ConfiguredWorkbenchQueryError::Query(error))
            if matches!(error.as_ref(), WorkbenchQueryError::MalformedBinding { .. })
    ));
    let other_binding = "0".repeat(64);
    if other_binding != identity.effective_binding_sha256 {
        assert!(matches!(
            engine.detail(&compact_exact.groups[0].hits[0].detail_ref, &other_binding),
            Err(ConfiguredWorkbenchQueryError::Query(error))
                if matches!(error.as_ref(), WorkbenchQueryError::StaleBinding { .. })
        ));
    }

    let entry_id: EntityId = dictionary.metadata().entries[0].id.clone();
    let target = RelationTarget::Local(LocalRef::Entry(entry_id.clone()));
    let resolved_full = engine
        .execute(DictionaryRequest::Resolve {
            target: target.clone(),
        })
        .unwrap();
    let WorkbenchFullResult::Resolve(resolved_full) = resolved_full.result else {
        panic!("configured Resolve retains the Full operation tag")
    };
    assert_eq!(resolved_full.target, target);
    let Some(vlajmaji_search::ReferenceEntity::Entry(entity)) = resolved_full.entity.as_ref()
    else {
        panic!("configured Full Resolve returns the typed entry")
    };
    assert_eq!(entity.id, entry_id);
    let resolved_compact = engine
        .execute_compact(DictionaryRequest::Resolve {
            target: target.clone(),
        })
        .unwrap();
    let vlajmaji_search::DictionaryCompactQueryResult::Resolve(resolved_compact) =
        resolved_compact.result
    else {
        panic!("configured Resolve retains the Compact operation tag")
    };
    assert_eq!(resolved_compact.target, target);
    let compact_entity = resolved_compact.entity.as_ref().unwrap();
    assert_eq!(compact_entity.role, "entry");
    assert_eq!(compact_entity.id, entry_id.as_str());

    let installed = CollectionCatalogue::open(&catalogue)
        .unwrap()
        .installed_artifact_locations(PackId::Llg)
        .unwrap()
        .unwrap();
    let release_bytes = fs::read(&installed.release_path).unwrap();
    let seal_bytes = fs::read(&installed.seal_path).unwrap();
    let held_release = root.join("held-alpha-release");
    let held_seal = root.join("held-alpha-seal");
    fs::rename(&installed.release_path, &held_release).unwrap();
    fs::rename(&installed.seal_path, &held_seal).unwrap();
    assert!(
        engine
            .execute(DictionaryRequest::Exact {
                spelling: "farvi".into(),
                options: DictionaryExactOptions::default(),
            })
            .is_ok(),
        "cached configured query survives advisory-copy disappearance"
    );
    fs::rename(&held_release, &installed.release_path).unwrap();
    fs::rename(&held_seal, &installed.seal_path).unwrap();
    assert_eq!(fs::read(&installed.release_path).unwrap(), release_bytes);
    assert_eq!(fs::read(&installed.seal_path).unwrap(), seal_bytes);

    let second =
        WorkbenchProfiles::configure(&config, configure("beta", &catalogue, false), Some(2))
            .unwrap();
    assert_eq!(second.revision, 3);
    let beta_destination = root.join("beta-workbench");
    let beta_refreshed =
        WorkbenchProfiles::refresh(&config, Some("beta"), &beta_destination, second.revision)
            .unwrap();
    assert_eq!(beta_refreshed.publication.revision, 4);
    let mut beta = ConfiguredWorkbenchSearchEngine::open(&config, Some("beta")).unwrap();
    let beta_config_bytes = fs::read(&config).unwrap();
    assert!(
        beta.execute(DictionaryRequest::Exact {
            spelling: "farvi".into(),
            options: DictionaryExactOptions::default(),
        })
        .is_ok(),
        "new beta capture is current before a byte-only edit"
    );
    let mut whitespace_edit = beta_config_bytes.clone();
    whitespace_edit.push(b'\n');
    fs::write(&config, whitespace_edit).unwrap();
    assert!(matches!(
        beta.execute(DictionaryRequest::Exact {
            spelling: "farvi".into(),
            options: DictionaryExactOptions::default(),
        }),
        Err(ConfiguredWorkbenchQueryError::Profile(error))
            if matches!(error.as_ref(), vlajmaji_store::WorkbenchProfileError::ConfigChanged {
                expected_sha256: Some(expected), observed_sha256: Some(observed), ..
            } if expected == &beta.identity().config_sha256 && observed != expected)
    ));
    fs::write(&config, &beta_config_bytes).unwrap();
    assert!(
        beta.execute(DictionaryRequest::Exact {
            spelling: "farvi".into(),
            options: DictionaryExactOptions::default(),
        })
        .is_ok(),
        "restoring exact captured bytes restores the unchanged capture"
    );
    let alpha_cursor = full_exact.next_cursor.clone().unwrap();
    assert!(matches!(
        beta.execute(DictionaryRequest::Exact {
            spelling: "farvi".into(),
            options: DictionaryExactOptions {
                cursor: Some(alpha_cursor),
                ..DictionaryExactOptions::default()
            },
        }),
        Err(ConfiguredWorkbenchQueryError::Query(error))
            if matches!(error.as_ref(), WorkbenchQueryError::Query(inner)
                if matches!(inner.as_ref(), vlajmaji_search::DictionaryQueryError::StaleCursor))
    ));
    assert!(matches!(
        beta.execute_compact(DictionaryRequest::Exact {
            spelling: "farvi".into(),
            options: DictionaryExactOptions {
                cursor: Some(compact_cursor),
                ..DictionaryExactOptions::default()
            },
        }),
        Err(ConfiguredWorkbenchQueryError::Query(error))
            if matches!(error.as_ref(), WorkbenchQueryError::Query(inner)
                if matches!(inner.as_ref(), vlajmaji_search::DictionaryQueryError::StaleCursor))
    ));
    assert!(matches!(
        beta.detail(
            &compact_exact.groups[0].hits[0].detail_ref,
            &identity.effective_binding_sha256,
        ),
        Err(ConfiguredWorkbenchQueryError::Query(error))
            if matches!(error.as_ref(), WorkbenchQueryError::StaleBinding { .. })
    ));
    let beta_binding = beta.identity().effective_binding_sha256.clone();
    assert!(matches!(
        engine.execute(DictionaryRequest::Exact {
            spelling: "farvi".into(),
            options: DictionaryExactOptions::default(),
        }),
        Err(ConfiguredWorkbenchQueryError::Profile(error))
            if matches!(error.as_ref(), vlajmaji_store::WorkbenchProfileError::ConfigChanged { .. })
    ));
    assert!(matches!(
        engine.execute_compact(DictionaryRequest::Exact {
            spelling: "farvi".into(),
            options: DictionaryExactOptions::default(),
        }),
        Err(ConfiguredWorkbenchQueryError::Profile(error))
            if matches!(error.as_ref(), vlajmaji_store::WorkbenchProfileError::ConfigChanged { .. })
    ));
    assert!(matches!(
        engine.detail(
            &compact_exact.groups[0].hits[0].detail_ref,
            &identity.effective_binding_sha256,
        ),
        Err(ConfiguredWorkbenchQueryError::Profile(error))
            if matches!(error.as_ref(), vlajmaji_store::WorkbenchProfileError::ConfigChanged { .. })
    ));
    fs::remove_file(&config).unwrap();
    assert!(matches!(
        beta.execute(DictionaryRequest::Exact {
            spelling: "farvi".into(),
            options: DictionaryExactOptions::default(),
        }),
        Err(ConfiguredWorkbenchQueryError::Profile(error))
            if matches!(error.as_ref(), vlajmaji_store::WorkbenchProfileError::ConfigChanged { .. })
    ));
    assert!(matches!(
        beta.execute_compact(DictionaryRequest::Exact {
            spelling: "farvi".into(),
            options: DictionaryExactOptions::default(),
        }),
        Err(ConfiguredWorkbenchQueryError::Profile(error))
            if matches!(error.as_ref(), vlajmaji_store::WorkbenchProfileError::ConfigChanged { .. })
    ));
    assert!(matches!(
        beta.detail(
            &compact_exact.groups[0].hits[0].detail_ref,
            &beta_binding,
        ),
        Err(ConfiguredWorkbenchQueryError::Profile(error))
            if matches!(error.as_ref(), vlajmaji_store::WorkbenchProfileError::ConfigChanged { .. })
    ));
    fs::remove_dir_all(root).unwrap();
}
