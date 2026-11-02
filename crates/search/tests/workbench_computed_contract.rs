//! Fixed, typed source inputs for computed lexical and proposal queries.

use std::{
    borrow::Cow,
    error::Error,
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

use vlajmaji_lexicon::{
    AbbreviationKind, AssertionRecord, Availability, CaptureMetadata, CodeNotice, CollectionRecord,
    ComponentRecord, ContentNotice, ContextKind, ContextRecord, ContributorId, ContributorRecord,
    Dictionary, DictionaryDocument, DictionaryMetadata, Eligibility, EntityId, EntryRecord,
    NativeEncoding, NativeField, OccurrenceAttribution, OccurrenceRef, OwnedHead,
    OwnedRevisionRecord, PackId, PackPayloadKind, Presence, RawCapture, RawOccurrence,
    RedistributionStatus, RegistryDocument, ReleaseAccounting, ReleaseRecord, SnapshotAttribution,
    SourceId, SourceOrigin, SourceRecord, SourceRegistry, TextValue, TimePrecision, seal_release,
};
use vlajmaji_search::{
    WorkbenchLexicalCompactResult, WorkbenchLexicalFullResult, WorkbenchLexicalOptions,
    WorkbenchLexicalQuery, WorkbenchLexicalRequest, WorkbenchQueryError, WorkbenchSearchEngine,
    render_workbench_lexical_compact,
};
use vlajmaji_store::{
    CollectionCatalogue, CollectionInput, CollectionRequest, CollectionSelection, OwnedJournal,
    WorkbenchBuildRequest, WorkbenchOwnedRevision, WorkbenchSelection, WorkbenchView,
};

static NEXT_ROOT: AtomicUsize = AtomicUsize::new(0);

fn fresh_root() -> PathBuf {
    for _ in 0..64 {
        let root = std::env::temp_dir().join(format!(
            "vlajmaji-computed-contract-{}-{}",
            std::process::id(),
            NEXT_ROOT.fetch_add(1, Ordering::Relaxed),
        ));
        match fs::create_dir(&root) {
            Ok(()) => return root,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => panic!("create exclusive computed-query root: {error}"),
        }
    }
    panic!("could not allocate an exclusive computed-query root after 64 attempts")
}

fn entity(value: &str) -> EntityId {
    EntityId::new(value).unwrap()
}

fn source(value: &str) -> SourceId {
    SourceId::new(value).unwrap()
}

fn text(value: &str) -> Presence<TextValue> {
    Presence::Value(TextValue {
        text: value.into(),
        language: Presence::Missing,
        markup: Presence::Missing,
    })
}

fn source_record(id: &str, abbreviation: &str, origin: SourceOrigin) -> SourceRecord {
    SourceRecord {
        id: source(id),
        display_name: match id {
            "comp-owned" => "Computed owned fixture",
            "comp-upstream" => "Computed upstream fixture",
            _ => unreachable!("fixed fixture source"),
        }
        .into(),
        abbreviation: abbreviation.into(),
        abbreviation_kind: AbbreviationKind::ProjectAssigned,
        abbreviation_evidence: Some("fixed S06 computed fixture".into()),
        origin,
    }
}

fn capture(
    namespace: &str,
    format_label: &str,
    raw_bytes: &[u8],
    occurrences: Vec<RawOccurrence>,
) -> RawCapture {
    RawCapture::new(
        namespace,
        CaptureMetadata {
            source_name: "S06 computed fixed input".into(),
            format_label: format_label.into(),
            locator: namespace.into(),
            revision: Presence::Missing,
            captured_at: Presence::Missing,
            time_precision: TimePrecision::Unknown,
            notice_locator: Presence::Missing,
        },
        raw_bytes.to_vec(),
        occurrences,
    )
}

fn field(name: &str, value: &str, encoding: NativeEncoding) -> NativeField {
    NativeField {
        name: name.into(),
        encoding,
        raw_value: value.into(),
    }
}

fn release(
    id: &str,
    component: &EntityId,
    captures: Vec<vlajmaji_lexicon::CaptureRef>,
    assertions: Vec<EntityId>,
    accounting: ReleaseAccounting,
) -> ReleaseRecord {
    ReleaseRecord {
        id: entity(id),
        component: component.clone(),
        distributor: None,
        upstream_revision: Presence::Missing,
        distributor_revision: Presence::Missing,
        captured_at: Presence::Missing,
        time_precision: TimePrecision::Unknown,
        schema_version: 1,
        normalizer: "computed-fixed-input-v1".into(),
        normalizer_version: "1".into(),
        captures,
        assertions,
        accounting,
        diagnostics: Vec::new(),
        selected: true,
        availability: Availability::Available,
        content_notices: vec![ContentNotice {
            text: Presence::Value("fixed computed source fixture".into()),
            locator: Presence::Missing,
            license: Presence::Missing,
        }],
        redistribution: RedistributionStatus::Held,
        redistribution_evidence: Vec::new(),
    }
}

fn accounting(input: u64, normalized: u64, excluded: u64) -> ReleaseAccounting {
    ReleaseAccounting {
        input_occurrences: Some(input),
        normalized_occurrences: Some(normalized),
        query_excluded_occurrences: Some(excluded),
        unit: "occurrences".into(),
    }
}

fn entry(
    id: &str,
    source_id: &SourceId,
    spelling: Presence<String>,
    class: Presence<String>,
) -> EntryRecord {
    EntryRecord {
        id: entity(id),
        source: source_id.clone(),
        spelling,
        language: Presence::Value("jbo".into()),
        script: Presence::Missing,
        native_word_class: class,
        selmaho: Presence::Missing,
        mapped_pos: Presence::Missing,
    }
}

fn assertion(
    id: &str,
    entry: &str,
    occurrence: OccurrenceRef,
    definition: &str,
    eligibility: Eligibility,
) -> AssertionRecord {
    AssertionRecord {
        id: entity(id),
        entry: entity(entry),
        occurrence,
        native_sense_id: Presence::Missing,
        language: Presence::Value("en".into()),
        definition: text(definition),
        source_score: Presence::Missing,
        timestamps: Vec::new(),
        eligibility,
    }
}

fn upstream_dictionary() -> Dictionary {
    let source_id = source("comp-upstream");
    let contributor = ContributorId::new("comp-upstream-editor").unwrap();
    let collection_id = entity("comp-upstream-collection");
    let component_id = entity("comp-upstream-component");

    let gismu_raw = b" farvi              grow                                      object evolves                                                                                  ";
    let cmavo_raw =
        b"ba        PU        future                                   future             ";
    let generic_raw = br#"[{"word":"snime","definition":"snow and plum","native_word_class":null},{"word":"farvi","definition":"future class","native_word_class":"future-native-class"},{"word":"klama","definition":"go"},{"definition":"missing word","native_word_class":"gismu"},{"word":null,"definition":"null word","native_word_class":"gismu"},{"word":"latkerlo","definition":"cat ear","native_word_class":"lujvo"},{"word":"farvi","definition":"excluded control","native_word_class":"gismu"}]"#;
    let gismu = capture(
        "computed-gismu-capture-r1",
        "llg-gismu-fixed-columns",
        gismu_raw,
        vec![RawOccurrence {
            locator: "line:1".into(),
            fields: vec![
                field("word", "farvi", NativeEncoding::Text),
                field("definition", "object evolves", NativeEncoding::Text),
            ],
            raw_line: Some(String::from_utf8(gismu_raw.to_vec()).unwrap()),
        }],
    );
    let cmavo = capture(
        "computed-cmavo-capture-r1",
        "llg-cmavo-fixed-columns",
        cmavo_raw,
        vec![RawOccurrence {
            locator: "line:1".into(),
            fields: vec![
                field("word", "ba", NativeEncoding::Text),
                field("definition", "future", NativeEncoding::Text),
            ],
            raw_line: Some(String::from_utf8(cmavo_raw.to_vec()).unwrap()),
        }],
    );
    let generic_rows: [(&str, &str, Option<&str>); 7] = [
        ("snime", "snow and plum", None),
        ("farvi", "future class", Some("future-native-class")),
        ("klama", "go", None),
        ("", "missing word", Some("gismu")),
        ("", "null word", Some("gismu")),
        ("latkerlo", "cat ear", Some("lujvo")),
        ("farvi", "excluded control", Some("gismu")),
    ];
    let generic_occurrences = generic_rows
        .iter()
        .enumerate()
        .map(|(index, (word, definition, class))| {
            let mut fields = vec![field(
                "definition",
                &serde_json::to_string(definition).unwrap(),
                NativeEncoding::Json,
            )];
            match index {
                3 => {}
                4 => fields.insert(0, field("word", "null", NativeEncoding::Json)),
                _ => fields.insert(
                    0,
                    field(
                        "word",
                        &serde_json::to_string(word).unwrap(),
                        NativeEncoding::Json,
                    ),
                ),
            }
            if let Some(class) = class {
                fields.push(field(
                    "native_word_class",
                    &serde_json::to_string(class).unwrap(),
                    NativeEncoding::Json,
                ));
            } else if index == 0 {
                fields.push(field("native_word_class", "null", NativeEncoding::Json));
            }
            RawOccurrence {
                locator: format!("/{index}"),
                fields,
                raw_line: None,
            }
        })
        .collect();
    let generic = capture(
        "computed-generic-capture-r1",
        "synthetic-computed-json-v1",
        generic_raw,
        generic_occurrences,
    );
    let captures = vec![gismu, cmavo, generic];
    let capture_ref = |namespace: &str| {
        captures
            .iter()
            .find(|capture| capture.identity.namespace == namespace)
            .unwrap()
            .identity
            .clone()
    };

    let ids = [
        "comp-upstream-farvi",
        "comp-upstream-ba",
        "comp-upstream-null-class",
        "comp-upstream-unsupported",
        "comp-upstream-missing-class",
        "comp-upstream-missing-word",
        "comp-upstream-null-word",
        "comp-upstream-lujvo",
        "comp-upstream-excluded",
    ];
    let occurrence = |namespace: &str, locator: &str| OccurrenceRef {
        capture: capture_ref(namespace),
        locator: locator.into(),
    };
    let source_rows = vec![source_record("comp-upstream", "CU", SourceOrigin::Upstream)];
    let snapshots = captures
        .iter()
        .map(|capture| SnapshotAttribution {
            snapshot: capture.identity.to_snapshot_identity().unwrap(),
            source: source_id.clone(),
            distributor: None,
        })
        .collect();
    let occurrence_refs = vec![
        occurrence("computed-gismu-capture-r1", "line:1"),
        occurrence("computed-cmavo-capture-r1", "line:1"),
        occurrence("computed-generic-capture-r1", "/0"),
        occurrence("computed-generic-capture-r1", "/1"),
        occurrence("computed-generic-capture-r1", "/2"),
        occurrence("computed-generic-capture-r1", "/3"),
        occurrence("computed-generic-capture-r1", "/4"),
        occurrence("computed-generic-capture-r1", "/5"),
        occurrence("computed-generic-capture-r1", "/6"),
    ];
    let registry = SourceRegistry::from_document(RegistryDocument {
        sources: source_rows,
        contributors: vec![ContributorRecord {
            id: contributor.clone(),
            display_name: "Computed fixture editor".into(),
        }],
        snapshots,
        occurrences: occurrence_refs
            .iter()
            .enumerate()
            .map(|(index, occurrence)| OccurrenceAttribution {
                occurrence: occurrence.to_record_identity().unwrap(),
                contributor: Some(contributor.clone()),
                native_entry_id: Some(format!("{}-entry", ids[index])),
                native_assertion_id: Some(ids[index].into()),
            })
            .collect(),
        ..RegistryDocument::default()
    })
    .unwrap();

    let spellings = [
        Presence::Value("farvi".into()),
        Presence::Value("ba".into()),
        Presence::Value("snime".into()),
        Presence::Value("farvi".into()),
        Presence::Value("klama".into()),
        Presence::Missing,
        Presence::Null,
        Presence::Value("latkerlo".into()),
        Presence::Value("farvi".into()),
    ];
    let classes = [
        Presence::Missing,
        Presence::Missing,
        Presence::Null,
        Presence::Value("future-native-class".into()),
        Presence::Missing,
        Presence::Value("gismu".into()),
        Presence::Value("gismu".into()),
        Presence::Value("lujvo".into()),
        Presence::Value("gismu".into()),
    ];
    let definitions = [
        "object evolves",
        "future",
        "snow and plum",
        "future class",
        "go",
        "missing word",
        "null word",
        "cat ear",
        "excluded control",
    ];
    let eligibility = [
        Eligibility::Lexical,
        Eligibility::Lexical,
        Eligibility::Lexical,
        Eligibility::Lexical,
        Eligibility::Lexical,
        Eligibility::Excluded {
            reason: "fixture-missing-headword".into(),
        },
        Eligibility::Excluded {
            reason: "fixture-null-headword".into(),
        },
        Eligibility::Lexical,
        Eligibility::Excluded {
            reason: "fixture-control".into(),
        },
    ];
    let mut metadata = DictionaryMetadata::default();
    metadata.collections.push(CollectionRecord {
        id: collection_id,
        source: source_id.clone(),
        display_name: "Computed upstream collection".into(),
        description: Presence::Missing,
    });
    metadata.components.push(ComponentRecord {
        id: component_id.clone(),
        collection: entity("comp-upstream-collection"),
        native_name: "computed upstream".into(),
        languages: vec!["jbo".into()],
    });
    metadata.code_notices.push(CodeNotice {
        text: "Synthetic fixed S06 test fixture; no rights grant".into(),
        locator: Presence::Missing,
        license: Presence::Missing,
    });
    for index in 0..ids.len() {
        metadata.entries.push(entry(
            &format!("{}-entry", ids[index]),
            &source_id,
            spellings[index].clone(),
            classes[index].clone(),
        ));
        metadata.assertions.push(assertion(
            ids[index],
            &format!("{}-entry", ids[index]),
            occurrence_refs[index].clone(),
            definitions[index],
            eligibility[index].clone(),
        ));
    }
    metadata.releases.push(release(
        "comp-upstream-release-r1",
        &component_id,
        captures.iter().map(|row| row.identity.clone()).collect(),
        ids.iter().map(|id| entity(id)).collect(),
        accounting(9, 9, 3),
    ));
    Dictionary::from_document(DictionaryDocument {
        registry,
        captures,
        metadata,
        ..DictionaryDocument::default()
    })
    .expect("fixed CU registry, captures, and amended release accounting validate")
}

fn owned_genesis() -> Dictionary {
    let owned_source = source("comp-owned");
    let editor = ContributorId::new("comp-owned-editor").unwrap();
    let collection_id = entity("comp-owned-collection");
    let component_id = entity("comp-owned-component");
    let revision_id = entity("comp-owned-r0");
    let registry = SourceRegistry::from_document(RegistryDocument {
        sources: vec![source_record("comp-owned", "CO", SourceOrigin::OwnedDraft)],
        contributors: vec![ContributorRecord {
            id: editor.clone(),
            display_name: "Computed owned editor".into(),
        }],
        ..RegistryDocument::default()
    })
    .unwrap();
    let mut metadata = DictionaryMetadata::default();
    metadata.collections.push(CollectionRecord {
        id: collection_id,
        source: owned_source.clone(),
        display_name: "Computed owned collection".into(),
        description: Presence::Missing,
    });
    metadata.components.push(ComponentRecord {
        id: component_id.clone(),
        collection: entity("comp-owned-collection"),
        native_name: "computed owned".into(),
        languages: vec!["jbo".into()],
    });
    metadata.releases.push(release(
        "comp-owned-release-r0",
        &component_id,
        Vec::new(),
        Vec::new(),
        accounting(0, 0, 0),
    ));
    metadata.owned_revisions.push(OwnedRevisionRecord {
        id: revision_id.clone(),
        source: owned_source.clone(),
        parent: None,
        editor,
        timestamp: Presence::Missing,
        time_precision: TimePrecision::Unknown,
        assertions: Vec::new(),
        tombstones: Vec::new(),
        rationale: text("empty genesis"),
    });
    metadata.owned_heads.push(OwnedHead {
        source: owned_source,
        revision: revision_id,
    });
    Dictionary::from_document(DictionaryDocument {
        registry,
        metadata,
        ..DictionaryDocument::default()
    })
    .expect("owned genesis is a valid zero-row prefix")
}

fn owned_r1(genesis: &Dictionary) -> Dictionary {
    let owned_source = source("comp-owned");
    let editor = ContributorId::new("comp-owned-editor").unwrap();
    let capture_raw = br#"[{"word":"farvi","definition":"plum grows","native_word_class":"gismu","notes":"See also {snime}."},{"word":"snime","definition":"snow and plum","native_word_class":"gismu"}]"#;
    let capture = capture(
        "computed-owned-capture-r1",
        "owned-draft-json-v1",
        capture_raw,
        vec![
            RawOccurrence {
                locator: "/0".into(),
                fields: vec![
                    field("word", r#""farvi""#, NativeEncoding::Json),
                    field("definition", r#""plum grows""#, NativeEncoding::Json),
                    field("native_word_class", r#""gismu""#, NativeEncoding::Json),
                    field("notes", r#""See also {snime}.""#, NativeEncoding::Json),
                ],
                raw_line: None,
            },
            RawOccurrence {
                locator: "/1".into(),
                fields: vec![
                    field("word", r#""snime""#, NativeEncoding::Json),
                    field("definition", r#""snow and plum""#, NativeEncoding::Json),
                    field("native_word_class", r#""gismu""#, NativeEncoding::Json),
                ],
                raw_line: None,
            },
        ],
    );
    let capture_ref = capture.identity.clone();
    let occurrences = ["/0", "/1"].map(|locator| OccurrenceRef {
        capture: capture_ref.clone(),
        locator: locator.into(),
    });
    let mut document = genesis.document();
    document.captures.push(capture);
    let mut registry = document.registry.document();
    registry.snapshots.push(SnapshotAttribution {
        snapshot: capture_ref.to_snapshot_identity().unwrap(),
        source: owned_source.clone(),
        distributor: None,
    });
    for (index, occurrence) in occurrences.iter().enumerate() {
        registry.occurrences.push(OccurrenceAttribution {
            occurrence: occurrence.to_record_identity().unwrap(),
            contributor: Some(editor.clone()),
            native_entry_id: Some(
                ["comp-owned-farvi-entry", "comp-owned-snime-entry"][index].into(),
            ),
            native_assertion_id: Some(["comp-owned-farvi", "comp-owned-snime"][index].into()),
        });
    }
    document.registry = SourceRegistry::from_document(registry).unwrap();
    let farvi = entity("comp-owned-farvi-entry");
    let snime = entity("comp-owned-snime-entry");
    document.metadata.entries = vec![
        entry(
            "comp-owned-farvi-entry",
            &owned_source,
            Presence::Value("farvi".into()),
            Presence::Value("gismu".into()),
        ),
        entry(
            "comp-owned-snime-entry",
            &owned_source,
            Presence::Value("snime".into()),
            Presence::Value("gismu".into()),
        ),
    ];
    document.metadata.assertions = vec![
        assertion(
            "comp-owned-farvi",
            farvi.as_str(),
            occurrences[0].clone(),
            "plum grows",
            Eligibility::Lexical,
        ),
        assertion(
            "comp-owned-snime",
            snime.as_str(),
            occurrences[1].clone(),
            "snow and plum",
            Eligibility::Lexical,
        ),
    ];
    document.metadata.contexts.push(ContextRecord {
        id: entity("comp-owned-note"),
        assertion: entity("comp-owned-farvi"),
        kind: ContextKind::Note,
        value: text("See also {snime}."),
        translated_from: None,
        evidence: vlajmaji_lexicon::SourceEvidence {
            occurrence: occurrences[0].clone(),
            field_index: Some(3),
            citation: Presence::Missing,
            offset: None,
        },
    });
    let component_id = entity("comp-owned-component");
    document.metadata.releases.push(release(
        "comp-owned-release-r1",
        &component_id,
        vec![capture_ref],
        vec![entity("comp-owned-farvi"), entity("comp-owned-snime")],
        accounting(2, 2, 0),
    ));
    let revision_id = entity("comp-owned-r1");
    document.metadata.catalogue_revision = 1;
    document.metadata.owned_revisions.push(OwnedRevisionRecord {
        id: revision_id.clone(),
        source: owned_source.clone(),
        parent: Some(entity("comp-owned-r0")),
        editor,
        timestamp: Presence::Missing,
        time_precision: TimePrecision::Unknown,
        assertions: vec![entity("comp-owned-farvi"), entity("comp-owned-snime")],
        tombstones: Vec::new(),
        rationale: text("append computed a and b"),
    });
    document.metadata.owned_heads[0].revision = revision_id;
    Dictionary::from_document(document).expect("amended owned r1 records validate")
}

fn mixed_dictionary(upstream: &Dictionary, owned_r1: &Dictionary) -> Dictionary {
    let mut document = upstream.document();
    let owned_document = owned_r1.document();

    document.captures.extend(owned_document.captures);
    let mut registry = document.registry.document();
    let owned_registry = owned_document.registry.document();
    registry.sources.extend(owned_registry.sources);
    registry.contributors.extend(owned_registry.contributors);
    registry.snapshots.extend(owned_registry.snapshots);
    registry.occurrences.extend(owned_registry.occurrences);
    document.registry = SourceRegistry::from_document(registry)
        .expect("fixed mixed pack registry preserves both source families");

    document
        .metadata
        .code_notices
        .extend(owned_document.metadata.code_notices);
    document
        .metadata
        .collections
        .extend(owned_document.metadata.collections);
    document
        .metadata
        .components
        .extend(owned_document.metadata.components);
    document
        .metadata
        .releases
        .extend(owned_document.metadata.releases);
    document
        .metadata
        .entries
        .extend(owned_document.metadata.entries);
    document
        .metadata
        .assertions
        .extend(owned_document.metadata.assertions);
    document
        .metadata
        .contexts
        .extend(owned_document.metadata.contexts);
    document
        .metadata
        .computations
        .extend(owned_document.metadata.computations);
    document
        .metadata
        .owned_revisions
        .extend(owned_document.metadata.owned_revisions);
    document.metadata.catalogue_revision = owned_document.metadata.catalogue_revision;
    // A retained mixed pack carries the byte-equal owned prefix, but no live head.
    assert!(document.metadata.owned_heads.is_empty());
    assert!(owned_document.metadata.owned_heads.len() == 1);
    Dictionary::from_document(document).expect("fixed mixed CU/owned r0-r1 pack validates")
}

fn prepared_r1(root: &std::path::Path) -> (PathBuf, PathBuf) {
    let upstream = upstream_dictionary();
    let genesis = owned_genesis();
    let revision_one = owned_r1(&genesis);
    let mixed = mixed_dictionary(&upstream, &revision_one);
    let owned_root = root.join("owned-journal");
    let (mut writer, created) = OwnedJournal::create(&owned_root, &genesis).unwrap();
    assert_eq!(created.head.sequence, 0);
    writer
        .commit(&entity("comp-owned-r0"), &revision_one)
        .unwrap();

    let catalogue_root = root.join("catalogue");
    let sealed = seal_release(PackId::LaLojban, PackPayloadKind::Synthetic, &mixed).unwrap();
    let catalogue_request = CollectionRequest::new(
        vec![CollectionInput::SealedBytes {
            pack_id: PackId::LaLojban,
            release: Cow::Borrowed(&sealed.release),
            seal: Cow::Borrowed(&sealed.seal),
        }],
        CollectionSelection::Packs(vec![PackId::LaLojban]),
        true,
    )
    .unwrap();
    CollectionCatalogue::install(&catalogue_root, catalogue_request).unwrap();
    let view_root = root.join("view-r1");
    WorkbenchView::build_new(
        &view_root,
        WorkbenchBuildRequest {
            catalogue_root: Some(catalogue_root),
            owned_root: Some(owned_root),
            selection: WorkbenchSelection {
                version: 1,
                packs: vec![PackId::LaLojban],
                disabled: Vec::new(),
                owned: Some(WorkbenchOwnedRevision::Revision(entity("comp-owned-r1"))),
                contexts: Vec::new(),
            },
        },
    )
    .unwrap();
    (view_root, root.to_path_buf())
}

fn request(query: WorkbenchLexicalQuery) -> WorkbenchLexicalRequest {
    WorkbenchLexicalRequest {
        version: 1,
        query,
        options: WorkbenchLexicalOptions {
            sources: vec!["comp-owned".into(), "comp-upstream".into()],
            ..WorkbenchLexicalOptions::default()
        },
    }
}

fn neutral_request(query: WorkbenchLexicalQuery) -> WorkbenchLexicalRequest {
    WorkbenchLexicalRequest {
        version: 1,
        query,
        options: WorkbenchLexicalOptions::default(),
    }
}

fn assert_full_hit_matches_fixed_input(
    hit: &vlajmaji_search::DictionaryHit,
    dictionary: &Dictionary,
) {
    let document = dictionary.document();
    let assertion = document
        .metadata
        .assertions
        .iter()
        .find(|row| row.id == hit.assertion.id)
        .expect("returned assertion is present in the fixed input");
    let tuple_parts = [
        assertion.occurrence.capture.namespace.as_bytes(),
        assertion.occurrence.capture.sha256.as_bytes(),
        assertion.occurrence.locator.as_bytes(),
    ];
    let mut tuple_bytes = Vec::new();
    for part in tuple_parts {
        tuple_bytes.extend_from_slice(&(part.len() as u64).to_be_bytes());
        tuple_bytes.extend_from_slice(part);
    }
    let occurrence_id = tuple_bytes
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    assert_eq!(hit.occurrence_id, occurrence_id);
    let entry = document
        .metadata
        .entries
        .iter()
        .find(|row| row.id == assertion.entry)
        .expect("fixed input assertion has its Entry");
    assert_eq!(&hit.entry, entry);
    assert_eq!(&hit.assertion, assertion);

    let capture = document
        .captures
        .iter()
        .find(|row| row.identity == assertion.occurrence.capture)
        .expect("fixed input assertion has its capture");
    assert_eq!(&hit.capture, &capture.metadata);
    let raw = capture
        .occurrences
        .iter()
        .find(|row| row.locator == assertion.occurrence.locator)
        .expect("fixed input assertion has its raw occurrence");
    assert_eq!(&hit.raw, raw);

    let registry = document.registry.document();
    let snapshot = assertion.occurrence.capture.to_snapshot_identity().unwrap();
    let snapshot_attribution = registry
        .snapshots
        .iter()
        .find(|row| row.snapshot == snapshot)
        .expect("fixed input capture has its snapshot attribution");
    assert_eq!(&hit.snapshot_attribution, snapshot_attribution);
    let occurrence_key = assertion.occurrence.to_record_identity().unwrap();
    let occurrence_attribution = registry
        .occurrences
        .iter()
        .find(|row| row.occurrence == occurrence_key);
    assert_eq!(hit.occurrence_attribution.as_ref(), occurrence_attribution);
    let contributor = occurrence_attribution
        .and_then(|row| row.contributor.as_ref())
        .and_then(|id| registry.contributors.iter().find(|row| row.id == *id));
    assert_eq!(hit.contributor.as_ref(), contributor);
    assert!(hit.distributor.is_none());

    let selected_release_ids = document
        .metadata
        .releases
        .iter()
        .filter(|release| {
            release.selected
                && release.availability == Availability::Available
                && release.assertions.contains(&assertion.id)
        })
        .map(|release| release.id.clone())
        .collect::<Vec<_>>();
    assert_eq!(hit.release_ids, selected_release_ids);
    assert_eq!(
        hit.contexts,
        document
            .metadata
            .contexts
            .iter()
            .filter(|row| row.assertion == assertion.id)
            .cloned()
            .collect::<Vec<_>>()
    );
    assert!(hit.frames.is_empty());
    assert!(hit.forms.is_empty());
    assert!(hit.etymologies.is_empty());
    assert!(hit.relations.is_empty());
    assert!(hit.editorial_senses.is_empty());
    assert!(hit.computations.is_empty());
    assert!(hit.annotations.is_empty());
    assert!(hit.extensions.is_empty());
    assert_eq!(
        hit.owned_revision_ids,
        document
            .metadata
            .owned_revisions
            .iter()
            .filter(|revision| revision.assertions.contains(&assertion.id))
            .map(|revision| revision.id.clone())
            .collect::<Vec<_>>()
    );
    assert!(hit.tombstoned_by.is_empty());
}

fn assert_computed_readable_matches_compact(result: &WorkbenchLexicalCompactResult) {
    let rendered = render_workbench_lexical_compact(result)
        .lines()
        .map(|line| {
            let (label, json) = line.split_once(": ").expect("label and JSON value");
            (
                label.to_owned(),
                serde_json::from_str(json).expect("complete readable JSON value"),
            )
        })
        .collect::<Vec<_>>();

    let mut expected = Vec::<(&str, serde_json::Value)>::new();
    macro_rules! source_body {
        ($operation:literal, $body:expr) => {{
            expected.extend([
                ("lexical_operation", serde_json::json!($operation)),
                (
                    "lexical_policy_ids",
                    serde_json::to_value(&$body.policy_ids).unwrap(),
                ),
                (
                    "lexical_totals",
                    serde_json::to_value(&$body.totals).unwrap(),
                ),
                (
                    "lexical_source_totals",
                    serde_json::to_value(&$body.source_totals).unwrap(),
                ),
                (
                    "lexical_exclusions",
                    serde_json::to_value(&$body.exclusions).unwrap(),
                ),
                (
                    "lexical_next_cursor",
                    serde_json::to_value(&$body.next_cursor).unwrap(),
                ),
            ]);
            expected.extend(
                $body
                    .hits
                    .iter()
                    .map(|hit| ("computed_hit", serde_json::to_value(hit).unwrap())),
            );
        }};
    }
    match result {
        WorkbenchLexicalCompactResult::SpellingDistance(body) => {
            source_body!("spelling_distance", body)
        }
        WorkbenchLexicalCompactResult::Prosody(body) => source_body!("prosody", body),
        WorkbenchLexicalCompactResult::Nuclei(body) => source_body!("nuclei", body),
        WorkbenchLexicalCompactResult::Contrast(body) => source_body!("contrast", body),
        WorkbenchLexicalCompactResult::Components(body) => source_body!("components", body),
        WorkbenchLexicalCompactResult::Compose(body) => source_body!("compose", body),
        WorkbenchLexicalCompactResult::ProposalExact(body) => {
            expected.extend([
                ("lexical_operation", serde_json::json!("proposal_exact")),
                (
                    "lexical_policy_ids",
                    serde_json::to_value(&body.policy_ids).unwrap(),
                ),
                (
                    "lexical_totals",
                    serde_json::to_value(&body.totals).unwrap(),
                ),
                (
                    "lexical_next_cursor",
                    serde_json::to_value(&body.next_cursor).unwrap(),
                ),
            ]);
            expected.extend(
                body.proposals
                    .iter()
                    .map(|proposal| ("proposal", serde_json::to_value(proposal).unwrap())),
            );
        }
        WorkbenchLexicalCompactResult::Generate(body) => expected.extend([
            ("lexical_operation", serde_json::json!("generate")),
            (
                "lexical_policy_ids",
                serde_json::to_value(&body.policy_ids).unwrap(),
            ),
            ("computed_value", serde_json::to_value(&body.value).unwrap()),
        ]),
        _ => panic!("expected an S06 computed Compact result"),
    }

    assert_eq!(
        rendered,
        expected
            .into_iter()
            .map(|(label, value)| (label.to_owned(), value))
            .collect::<Vec<_>>(),
        "complete Readable values and global header/record order"
    );
}

#[test]
fn test_fixed_computed_candidates_and_depth_one_compose_hops() {
    let root = fresh_root();
    let (view, _root) = prepared_r1(&root);
    let mut engine = WorkbenchSearchEngine::open(&view).unwrap();
    let upstream_input = upstream_dictionary();
    let owned_input = owned_r1(&owned_genesis());
    let result = engine
        .execute_lexical(&request(WorkbenchLexicalQuery::SpellingDistance {
            anchor: "farvi".into(),
            max_distance: None,
        }))
        .unwrap();
    let valid_response_json = serde_json::to_value(&result).unwrap();
    let decoded_response = serde_json::from_value::<
        vlajmaji_search::workbench::WorkbenchLexicalFullResponse,
    >(valid_response_json)
    .expect("the unchanged complete response is a valid strict wire value");
    assert_eq!(decoded_response, result);
    let mut response_with_unknown = serde_json::to_value(&result).unwrap();
    response_with_unknown["unexpected"] = serde_json::json!(true);
    assert!(
        serde_json::from_value::<vlajmaji_search::workbench::WorkbenchLexicalFullResponse>(
            response_with_unknown
        )
        .is_err()
    );
    let mut operation_with_unknown = serde_json::to_value(&result.result).unwrap();
    operation_with_unknown["response"]["unexpected"] = serde_json::json!(true);
    assert!(serde_json::from_value::<WorkbenchLexicalFullResult>(operation_with_unknown).is_err());

    let mut missing_required_nullable = serde_json::to_value(&result).unwrap();
    missing_required_nullable["result"]["response"]
        .as_object_mut()
        .unwrap()
        .remove("next_cursor");
    assert!(
        serde_json::from_value::<vlajmaji_search::workbench::WorkbenchLexicalFullResponse>(
            missing_required_nullable
        )
        .is_err()
    );
    let duplicate_required_nullable = serde_json::to_string(&result).unwrap().replacen(
        "\"next_cursor\":null",
        "\"next_cursor\":null,\"next_cursor\":null",
        1,
    );
    assert_ne!(
        duplicate_required_nullable,
        serde_json::to_string(&result).unwrap()
    );
    assert!(
        serde_json::from_str::<vlajmaji_search::workbench::WorkbenchLexicalFullResponse>(
            &duplicate_required_nullable
        )
        .is_err()
    );

    let WorkbenchLexicalFullResult::SpellingDistance(result) = result.result else {
        panic!("expected computed Full response")
    };
    assert_eq!(result.totals.candidate_assertions, 8);
    assert_eq!(result.totals.candidate_distinct_spellings, 5);
    assert_eq!(result.totals.records, 8);
    assert_eq!(result.code_notices, upstream_input.metadata().code_notices);
    for computed_hit in &result.hits {
        let input = match computed_hit.hit.entry.source.as_str() {
            "comp-owned" => &owned_input,
            "comp-upstream" => &upstream_input,
            other => panic!("unexpected fixed source {other}"),
        };
        assert_full_hit_matches_fixed_input(&computed_hit.hit, input);
    }
    assert_eq!(
        result
            .hits
            .iter()
            .map(|hit| hit.hit.assertion.id.as_str())
            .collect::<Vec<_>>(),
        [
            "comp-owned-farvi",
            "comp-upstream-farvi",
            "comp-upstream-unsupported",
            "comp-upstream-ba",
            "comp-upstream-missing-class",
            "comp-owned-snime",
            "comp-upstream-null-class",
            "comp-upstream-lujvo",
        ]
    );
    assert!(
        !result
            .hits
            .iter()
            .any(|hit| { matches!(hit.hit.entry.spelling, Presence::Missing | Presence::Null) })
    );

    let compose = engine
        .execute_lexical_compact(&request(WorkbenchLexicalQuery::Compose {
            definition_contains: Some("plum".into()),
            links_from: Some("farvi".into()),
            min_nuclei: Some(2),
            kind: Some("brivla".into()),
        }))
        .unwrap();
    let WorkbenchLexicalCompactResult::Compose(compose) = compose.result else {
        panic!("expected Compose Compact response")
    };
    assert_eq!(compose.totals.candidate_assertions, 8);
    assert_eq!(compose.totals.records, 1);
    assert_eq!(
        compose.hits[0].hit.assertion.id.as_str(),
        "comp-owned-snime"
    );
    assert_eq!(compose.exclusions.get("candidate-filter"), Some(&6));
    assert_eq!(compose.exclusions.get("unmapped-source-class"), Some(&1));

    let mut paged_request = request(WorkbenchLexicalQuery::SpellingDistance {
        anchor: "farvi".into(),
        max_distance: None,
    });
    paged_request.options.page_size = Some(1);
    let first_page = engine.execute_lexical(&paged_request).unwrap();
    let WorkbenchLexicalFullResult::SpellingDistance(first_page) = first_page.result else {
        panic!("expected first spelling-distance page")
    };
    assert_eq!(first_page.totals.records, 8);
    assert_eq!(first_page.hits.len(), 1);
    assert_eq!(
        first_page.hits[0].hit.assertion.id.as_str(),
        "comp-owned-farvi"
    );
    let first_cursor = first_page.next_cursor.clone().expect("more matches remain");

    paged_request.options.cursor = Some(first_cursor);
    // Page size is intentionally absent from the query identity.
    paged_request.options.page_size = Some(3);
    let second_page = engine.execute_lexical(&paged_request).unwrap();
    let WorkbenchLexicalFullResult::SpellingDistance(second_page) = second_page.result else {
        panic!("expected second spelling-distance page")
    };
    assert_eq!(second_page.hits.len(), 3);
    assert_eq!(second_page.totals.records, 8);
    let second_cursor = second_page
        .next_cursor
        .clone()
        .expect("four matches remain");
    paged_request.options.cursor = Some(second_cursor);
    paged_request.options.page_size = None;
    let final_page = engine.execute_lexical(&paged_request).unwrap();
    let WorkbenchLexicalFullResult::SpellingDistance(final_page) = final_page.result else {
        panic!("expected final spelling-distance page")
    };
    assert_eq!(final_page.hits.len(), 4);
    assert_eq!(final_page.next_cursor, None);
    let joined_ids = first_page
        .hits
        .iter()
        .chain(second_page.hits.iter())
        .chain(final_page.hits.iter())
        .map(|hit| hit.hit.assertion.id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        joined_ids,
        [
            "comp-owned-farvi",
            "comp-upstream-farvi",
            "comp-upstream-unsupported",
            "comp-upstream-ba",
            "comp-upstream-missing-class",
            "comp-owned-snime",
            "comp-upstream-null-class",
            "comp-upstream-lujvo",
        ]
    );

    let mut zero_request = request(WorkbenchLexicalQuery::SpellingDistance {
        anchor: "farvi".into(),
        max_distance: None,
    });
    zero_request.options.page_size = Some(0);
    let zero_full = engine.execute_lexical(&zero_request).unwrap();
    let WorkbenchLexicalFullResult::SpellingDistance(zero_full) = zero_full.result else {
        panic!("expected zero-size Full response")
    };
    assert_eq!(zero_full.totals.records, 8);
    assert!(zero_full.hits.is_empty());
    assert_eq!(zero_full.next_cursor, None);
    let zero_compact = engine.execute_lexical_compact(&zero_request).unwrap();
    let WorkbenchLexicalCompactResult::SpellingDistance(zero_compact) = zero_compact.result else {
        panic!("expected zero-size Compact response")
    };
    assert_eq!(zero_compact.totals, zero_full.totals);
    assert_eq!(zero_compact.source_totals, zero_full.source_totals);
    assert!(zero_compact.hits.is_empty());
    assert_eq!(zero_compact.next_cursor, None);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn test_fixed_prosody_nuclei_and_contrast_oracles() {
    let root = fresh_root();
    let (view, _root) = prepared_r1(&root);
    let mut engine = WorkbenchSearchEngine::open(&view).unwrap();

    let prosody = engine
        .execute_lexical(&request(WorkbenchLexicalQuery::Prosody {
            kind: Some("brivla".into()),
            syllables: Some(2),
            nuclei: Some(2),
            primary_nucleus: Some(0),
        }))
        .unwrap();
    let WorkbenchLexicalFullResult::Prosody(prosody) = prosody.result else {
        panic!("expected Prosody Full response")
    };
    assert_eq!(prosody.totals.candidate_assertions, 8);
    assert_eq!(prosody.totals.candidate_distinct_spellings, 5);
    assert_eq!(prosody.totals.records, 3);
    assert_eq!(prosody.totals.distinct_spellings, 2);
    assert_eq!(prosody.totals.source_groups, 2);
    assert_eq!(prosody.exclusions.get("unmapped-source-class"), Some(&3));
    assert_eq!(prosody.exclusions.get("requested-kind-filter"), Some(&2));
    assert_eq!(
        prosody
            .hits
            .iter()
            .map(|hit| hit.hit.assertion.id.as_str())
            .collect::<Vec<_>>(),
        [
            "comp-owned-farvi",
            "comp-upstream-farvi",
            "comp-owned-snime"
        ]
    );
    let farvi = prosody.hits[0].analysis.as_ref().unwrap();
    assert_eq!(farvi.mapped_word_kind, "brivla");
    assert_eq!(
        farvi
            .nuclei
            .iter()
            .map(|nucleus| nucleus.spelling.as_str())
            .collect::<Vec<_>>(),
        ["a", "i"]
    );
    assert_eq!(farvi.syllable_count, Some(2));
    assert_eq!(farvi.primary_nucleus, Some(0));
    assert_eq!(farvi.stress_status, "assigned-primary");
    assert!(farvi.diagnostics.is_empty());
    assert_eq!(
        farvi.class_evidence.native_word_class,
        Presence::Value("gismu".into())
    );
    assert_eq!(farvi.class_evidence.capture, None);
    assert_eq!(farvi.class_evidence.format_label, None);

    let nuclei = engine
        .execute_lexical_compact(&request(WorkbenchLexicalQuery::Nuclei {
            kind: "brivla".into(),
            pattern: vec!["a".into(), "i".into()],
        }))
        .unwrap();
    let WorkbenchLexicalCompactResult::Nuclei(nuclei) = nuclei.result else {
        panic!("expected Nuclei Compact response")
    };
    assert_eq!(nuclei.totals.candidate_assertions, 8);
    assert_eq!(nuclei.totals.records, 2);
    assert_eq!(nuclei.totals.distinct_spellings, 1);
    assert_eq!(nuclei.totals.source_groups, 2);
    assert_eq!(nuclei.exclusions.get("unmapped-source-class"), Some(&3));
    assert_eq!(nuclei.exclusions.get("requested-kind-filter"), Some(&2));
    assert_eq!(nuclei.exclusions.get("nucleus-pattern-mismatch"), Some(&1));
    assert_eq!(
        nuclei
            .hits
            .iter()
            .map(|hit| hit.hit.assertion.id.as_str())
            .collect::<Vec<_>>(),
        ["comp-owned-farvi", "comp-upstream-farvi"]
    );
    assert_computed_readable_matches_compact(&WorkbenchLexicalCompactResult::Nuclei(
        nuclei.clone(),
    ));

    let contrast = engine
        .execute_lexical(&request(WorkbenchLexicalQuery::Contrast {
            anchor: "farvi".into(),
            kind: "brivla".into(),
            require_mismatch: true,
        }))
        .unwrap();
    let WorkbenchLexicalFullResult::Contrast(contrast) = contrast.result else {
        panic!("expected Contrast Full response")
    };
    assert_eq!(contrast.totals.candidate_assertions, 8);
    assert_eq!(contrast.totals.records, 1);
    assert_eq!(contrast.totals.distinct_spellings, 1);
    assert_eq!(contrast.totals.source_groups, 1);
    assert_eq!(contrast.exclusions.get("unmapped-source-class"), Some(&3));
    assert_eq!(contrast.exclusions.get("requested-kind-filter"), Some(&2));
    assert_eq!(
        contrast.exclusions.get("no-within-vowel-mismatch"),
        Some(&2)
    );
    assert_eq!(
        contrast.hits[0].hit.assertion.id.as_str(),
        "comp-owned-snime"
    );
    assert_eq!(
        contrast.hits[0]
            .distance_positions
            .iter()
            .map(|position| (
                position.index,
                position.anchor.as_str(),
                position.candidate.as_str(),
                position.total,
            ))
            .collect::<Vec<_>>(),
        [(0, "a", "i", 1), (1, "i", "e", 1)]
    );
    assert_eq!(contrast.hits[0].spelling_distance, None);

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn test_proposal_exact_constructor_identity_and_ordinals() {
    let root = fresh_root();
    let (view, _root) = prepared_r1(&root);
    let mut engine = WorkbenchSearchEngine::open(&view).unwrap();
    let proposals = vec![
        vlajmaji_search::workbench::WorkbenchProposalInput {
            namespace: "experiment-one".into(),
            spelling: "farvi".into(),
            text: "first reading".into(),
        },
        vlajmaji_search::workbench::WorkbenchProposalInput {
            namespace: "experiment-two".into(),
            spelling: "farvi".into(),
            text: "second reading".into(),
        },
        vlajmaji_search::workbench::WorkbenchProposalInput {
            namespace: "experiment-one".into(),
            spelling: "farvi".into(),
            text: "first reading".into(),
        },
        vlajmaji_search::workbench::WorkbenchProposalInput {
            namespace: "experiment-one".into(),
            spelling: "snime".into(),
            text: "unmatched".into(),
        },
        vlajmaji_search::workbench::WorkbenchProposalInput {
            namespace: " ".into(),
            spelling: "".into(),
            text: "".into(),
        },
    ];
    let result = engine
        .execute_lexical(&neutral_request(WorkbenchLexicalQuery::ProposalExact {
            spelling: "farvi".into(),
            proposals: proposals.clone(),
        }))
        .unwrap();
    let WorkbenchLexicalFullResult::ProposalExact(result) = result.result else {
        panic!("expected ProposalExact Full response")
    };
    assert_eq!(result.totals.unit, "caller_proposals");
    assert_eq!(result.totals.candidate_proposals, 5);
    assert_eq!(result.totals.records, 3);
    assert_eq!(result.next_cursor, None);
    assert!(result.code_notices.is_empty());
    assert_eq!(
        result
            .proposals
            .iter()
            .map(|proposal| (proposal.id.as_str(), proposal.input_ordinal))
            .collect::<Vec<_>>(),
        [
            (
                "561410100f6a75f60b8c1d15ffaa94b110a9a05ef32f6bf22c65a6a1aea57d21",
                1,
            ),
            (
                "e665ac3f2e96bf14d5809d81fda5b1c9f369364d9a0ae275af20512e54d01258",
                0,
            ),
            (
                "e665ac3f2e96bf14d5809d81fda5b1c9f369364d9a0ae275af20512e54d01258",
                2,
            ),
        ]
    );
    let compact = engine
        .execute_lexical_compact(&neutral_request(WorkbenchLexicalQuery::ProposalExact {
            spelling: "farvi".into(),
            proposals: proposals.clone(),
        }))
        .unwrap();
    let WorkbenchLexicalCompactResult::ProposalExact(compact) = compact.result else {
        panic!("expected ProposalExact Compact response")
    };
    assert_computed_readable_matches_compact(&WorkbenchLexicalCompactResult::ProposalExact(
        compact,
    ));
    let empty = engine
        .execute_lexical(&neutral_request(WorkbenchLexicalQuery::ProposalExact {
            spelling: String::new(),
            proposals: proposals.clone(),
        }))
        .unwrap();
    let WorkbenchLexicalFullResult::ProposalExact(empty) = empty.result else {
        panic!("expected empty-spelling ProposalExact response")
    };
    assert_eq!(empty.totals.candidate_proposals, 5);
    assert_eq!(empty.totals.records, 1);
    assert_eq!(empty.proposals[0].input_ordinal, 4);
    assert_eq!(
        empty.proposals[0].id,
        "2b7206453a4ba8fa60da58e995d0e95224e55aefa653674edcfa9bd9dfc85d0b"
    );

    let mut paged = neutral_request(WorkbenchLexicalQuery::ProposalExact {
        spelling: "farvi".into(),
        proposals: proposals.clone(),
    });
    paged.options.page_size = Some(1);
    let first = engine.execute_lexical(&paged).unwrap();
    let WorkbenchLexicalFullResult::ProposalExact(first) = first.result else {
        panic!("expected first ProposalExact page")
    };
    assert_eq!(first.totals.candidate_proposals, 5);
    assert_eq!(first.totals.records, 3);
    assert_eq!(first.proposals[0].input_ordinal, 1);
    let cursor = first.next_cursor.expect("two exact matches remain");
    paged.options.cursor = Some(cursor);
    let second = engine.execute_lexical(&paged).unwrap();
    let WorkbenchLexicalFullResult::ProposalExact(second) = second.result else {
        panic!("expected second ProposalExact page")
    };
    assert_eq!(second.proposals[0].input_ordinal, 0);
    let cursor = second.next_cursor.expect("duplicate ordinal remains");
    paged.options.cursor = Some(cursor);
    paged.options.page_size = None;
    let final_page = engine.execute_lexical(&paged).unwrap();
    let WorkbenchLexicalFullResult::ProposalExact(final_page) = final_page.result else {
        panic!("expected final ProposalExact page")
    };
    assert_eq!(final_page.proposals.len(), 1);
    assert_eq!(final_page.proposals[0].input_ordinal, 2);
    assert_eq!(final_page.next_cursor, None);

    let invalid_namespace = neutral_request(WorkbenchLexicalQuery::ProposalExact {
        spelling: "different".into(),
        proposals: vec![vlajmaji_search::workbench::WorkbenchProposalInput {
            namespace: "captured:forbidden".into(),
            spelling: "farvi".into(),
            text: "invalid caller proposal".into(),
        }],
    });
    let mut invalid_namespace = invalid_namespace;
    invalid_namespace.options.page_size = Some(0);
    let proposal_error = engine.execute_lexical(&invalid_namespace).unwrap_err();
    let WorkbenchQueryError::Lexical(proposal_error) = proposal_error else {
        panic!("proposal constructor failure must retain its lexical error wrapper")
    };
    let proposal_cause = proposal_error.source().expect("proposal import cause");
    assert!(matches!(
        proposal_cause.downcast_ref::<vlajmaji_lexicon::ImportError>(),
        Some(vlajmaji_lexicon::ImportError::InvalidProposalNamespace)
    ));

    fs::remove_dir_all(root).unwrap();
}

#[cfg(feature = "morphology")]
#[test]
fn test_components_and_generate_use_actual_engine_values() {
    let root = fresh_root();
    let (view, _root) = prepared_r1(&root);
    let mut engine = WorkbenchSearchEngine::open(&view).unwrap();
    let components = engine
        .execute_lexical(&request(WorkbenchLexicalQuery::Components {
            components: vec!["mlatu".into()],
        }))
        .unwrap();
    let WorkbenchLexicalFullResult::Components(components) = components.result else {
        panic!("expected Components Full response")
    };
    assert_eq!(components.totals.candidate_assertions, 8);
    assert_eq!(components.totals.records, 1);
    assert_eq!(components.totals.distinct_spellings, 1);
    assert_eq!(components.totals.source_groups, 1);
    assert_eq!(
        components.hits[0].hit.assertion.id.as_str(),
        "comp-upstream-lujvo"
    );
    assert_eq!(
        components.hits[0].computed_components.as_ref().unwrap(),
        &vec!["mlatu".to_owned(), "kerlo".to_owned()]
    );

    for (setting, expected_id) in [
        (vlajmaji_search::MorphologySetting::Standard, "Standard"),
        (vlajmaji_search::MorphologySetting::AllowY, "AllowY"),
        (vlajmaji_search::MorphologySetting::ForceY, "ForceY"),
    ] {
        let generated = engine
            .execute_lexical(&neutral_request(WorkbenchLexicalQuery::Generate {
                components: vec!["mlatu".into(), "kerlo".into()],
                setting,
            }))
            .unwrap();
        let WorkbenchLexicalFullResult::Generate(generated) = generated.result else {
            panic!("expected Generate Full response")
        };
        assert_eq!(generated.value.spelling, "latkerlo");
        assert_eq!(generated.value.engine_score, 7937);
        assert_eq!(generated.value.component_indices, [[0, 3], [3, 8]]);
        assert_eq!(
            generated.value.engine_id,
            format!("latkerlo-jvotci-2.10.2601/{expected_id}")
        );
        assert!(generated.code_notices.is_empty());
    }
    let compact = engine
        .execute_lexical_compact(&neutral_request(WorkbenchLexicalQuery::Generate {
            components: vec!["mlatu".into(), "kerlo".into()],
            setting: vlajmaji_search::MorphologySetting::ForceY,
        }))
        .unwrap();
    let WorkbenchLexicalCompactResult::Generate(compact) = compact.result else {
        panic!("expected Generate Compact response")
    };
    assert_computed_readable_matches_compact(&WorkbenchLexicalCompactResult::Generate(compact));

    let empty_anchor = request(WorkbenchLexicalQuery::SpellingDistance {
        anchor: String::new(),
        max_distance: None,
    });
    let computation_error = engine.execute_lexical(&empty_anchor).unwrap_err();
    let WorkbenchQueryError::Lexical(computation_error) = computation_error else {
        panic!("computed validation failure must retain its lexical error wrapper")
    };
    let computation_cause = computation_error.source().expect("SearchError cause");
    assert!(matches!(
        computation_cause.downcast_ref::<vlajmaji_search::SearchError>(),
        Some(vlajmaji_search::SearchError::InvalidQuery(_))
    ));

    assert!(serde_json::from_str::<WorkbenchLexicalRequest>(
        r#"{"version":1,"query":{"operation":"spelling_distance","anchor":"farvi"},"options":{"sources":[],"alias_scope":"ActiveOnly","page_size":null,"cursor":null}}"#
    )
    .is_err());
    assert!(serde_json::from_str::<WorkbenchLexicalRequest>(
        r#"{"version":1,"version":1,"query":{"operation":"spelling_distance","anchor":"farvi","max_distance":null},"options":{"sources":[],"alias_scope":"ActiveOnly","page_size":null,"cursor":null}}"#
    )
    .is_err());
    assert!(
        serde_json::from_str::<vlajmaji_search::workbench::WorkbenchProposalInput>(
            r#"{"namespace":"proposal","spelling":"farvi","text":"text","unexpected":true}"#
        )
        .is_err()
    );
    assert!(
        serde_json::from_str::<vlajmaji_search::workbench::WorkbenchProposalInput>(
            r#"{"namespace":"proposal","namespace":"proposal","spelling":"farvi","text":"text"}"#
        )
        .is_err()
    );

    fs::remove_dir_all(root).unwrap();
}
