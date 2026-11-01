//! Emit typed mixed-view inputs, then build a synthetic upstream + owned workbench.
//!
//! Usage: `cargo run -p vlajmaji-search --example integrated_workbench -- OUTPUT_ROOT`
//!
//! The fresh `OUTPUT_ROOT/inputs` directory is complete before either source
//! journal or workbench is created. It contains the tiny La-Lojban input,
//! schema-one owned genesis/r1/r2 dictionaries, selection, exact request, and
//! input-derived expected source/definition rosters. The two sources are
//! deliberately distinct: upstream `lensisku` contributes “advance”; the
//! owned journal contributes “grow” and “develop” at r1, then “flourish” at r2.

use std::{
    borrow::Cow,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

use sha2::{Digest, Sha256};
use vlajmaji_lexicon::{
    AbbreviationKind, AssertionRecord, Availability, CaptureMetadata, CaptureRef, CollectionRecord,
    ComponentRecord, ComputationRecord, ContentNotice, ContextKind, ContextRecord, ContributorId,
    ContributorRecord, Dictionary, DictionaryDocument, DictionaryMetadata, Direction,
    EditorialSenseRecord, Eligibility, EntityId, EntryRecord, FormKind, FormRecord, InputRef,
    LaLojbanImportOptions, LocalRef, NativeEncoding, NativeField, OccurrenceAttribution,
    OccurrenceRef, OwnedHead, OwnedRevisionRecord, PackId, PackPayloadKind, Presence, RawCapture,
    RawOccurrence, RedistributionStatus, RegistryDocument, RelationKind, RelationRecord,
    RelationScope, RelationTarget, ReleaseAccounting, ReleaseRecord, SnapshotAttribution,
    SourceEvidence, SourceId, SourceOrigin, SourceRecord, SourceRegistry, TextValue, TimePrecision,
    import_la_lojban_export, seal_release,
};
use vlajmaji_search::{
    ConfiguredWorkbenchQueryError, ConfiguredWorkbenchSearchEngine, DictionaryExactOptions,
    DictionaryRequest, ReferenceEntity, ReferenceStatus, WorkbenchFullResult,
    WorkbenchLexicalCompactResult, WorkbenchLexicalFullResult, WorkbenchLexicalRequest,
    WorkbenchLinksCompact, WorkbenchLinksFull, WorkbenchQueryError, WorkbenchSearchEngine,
};
use vlajmaji_store::{
    CollectionCatalogue, CollectionInput, CollectionRequest, CollectionSelection, OwnedJournal,
    WorkbenchBuildRequest, WorkbenchError, WorkbenchOwnedRevision,
    WorkbenchProfileConfigureRequest, WorkbenchProfileError, WorkbenchProfiles, WorkbenchSelection,
    WorkbenchView,
};

const UPSTREAM_JSON: &[u8] = br#"[
  {"word":"farvi","word_type":"gismu","definition":"advance","definition_id":1}
]"#;
const PROFILE_BETA_SELECTION_JSON: &[u8] =
    br#"{"version":1,"packs":[],"disabled":[],"owned":"head","contexts":[]}"#;
const PROFILES_EXPECTED_JSON: &[u8] = br#"{"version":1,"config_revisions":[1,2,3,4,5,6],"prepared_sequences":[0,1,2],"alpha_r1":{"occurrences":3,"source_groups":2},"beta_r1":{"occurrences":2,"source_groups":1},"beta_r2":{"occurrences":3,"source_groups":1}}"#;
const REFERENCE_OWNED_ENTRY_JSON: &[u8] = br#"{"Local":{"Entry":"owned-entry-farvi"}}"#;
const REFERENCE_OWNED_ASSERTION_JSON: &[u8] = br#"{"Local":{"Assertion":"owned-assertion-a"}}"#;
const REFERENCE_UNKNOWN_JSON: &[u8] = br#""Unknown""#;
const REFERENCE_UNRESOLVED_JSON: &[u8] = br#"{"Unresolved":"future editorial target"}"#;
const REFERENCE_DISABLED_SELECTION_JSON: &[u8] =
    br#"{"version":1,"packs":["la-lojban"],"disabled":["la-lojban"],"owned":"head","contexts":[]}"#;
const REFERENCE_EXPECTED_JSON: &[u8] = br#"{"version":1,"kind":"vlajmaji-workbench-reference","owned_source":"owned-example","owned_r1_assertions":["owned-assertion-a","owned-assertion-b"],"upstream_source":"lensisku","upstream_assertion":"la-lojban-06220fcd82ff067293787c5057b3a68d6c63400d80e29d02fe5a4a64be015db5-assertion-0","owned_status":"Resolved","upstream_status":"Resolved","disabled_status":"CatalogueOnly","disabled_reason":"disabled","unknown_status":"Unknown","unresolved_status":"Unresolved","profile_revisions":[1,2,3,4,5,6],"publication_sequences":[0,1,2]}"#;
const LEXICAL_SELECTION_JSON: &[u8] =
    br#"{"version":1,"packs":["la-lojban"],"disabled":[],"owned":"head","contexts":[]}"#;
const LEXICAL_REVERSE_REQUEST_JSON: &[u8] = br#"{"version":1,"query":{"operation":"reverse_definition","contains":"gRoW"},"options":{"sources":[],"alias_scope":"ActiveOnly","page_size":null,"cursor":null}}"#;
const LEXICAL_FORM_REQUEST_JSON: &[u8] = br#"{"version":1,"query":{"operation":"form","value":"farvi-form","role":null,"kind":null},"options":{"sources":[],"alias_scope":"ActiveOnly","page_size":null,"cursor":null}}"#;
const LEXICAL_EXPECTED_JSON: &[u8] = br#"{"version":1,"policy":"stored-form-kind-v1","source_order":["lexical-owned","lexical-upstream"],"form_value":"farvi-form","form_order":["lexical-owned-form-stem","lexical-owned-form-components","lexical-owned-form-candidate","lexical-upstream-form-variant","lexical-upstream-form-pronunciation","lexical-upstream-form-reported","lexical-upstream-form-rafsi"],"forms":[{"id":"lexical-owned-form-stem","kind":"ComputedStem","entry":"lexical-owned-entry","mapped_role":null,"computation":"lexical-owned-computation-stem","evidence_status":null},{"id":"lexical-owned-form-components","kind":"ComputedComponents","entry":"lexical-owned-empty-entry","mapped_role":null,"computation":"lexical-owned-computation-components","evidence_status":null},{"id":"lexical-owned-form-candidate","kind":"GeneratedCandidate","entry":"lexical-owned-empty-entry","mapped_role":null,"computation":"lexical-owned-computation-candidate","evidence_status":null},{"id":"lexical-upstream-form-variant","kind":"SourceVariant","entry":"lexical-upstream-entry","mapped_role":null,"computation":null,"evidence_status":"CatalogueOnly"},{"id":"lexical-upstream-form-pronunciation","kind":"SourcePronunciation","entry":"lexical-upstream-entry","mapped_role":null,"computation":null,"evidence_status":"Loaded"},{"id":"lexical-upstream-form-reported","kind":"SourceReportedForm","entry":"lexical-upstream-entry","mapped_role":"SourceReportedForm","computation":null,"evidence_status":"Loaded"},{"id":"lexical-upstream-form-rafsi","kind":"AssignedRafsi","entry":"lexical-upstream-entry","mapped_role":"AssignedRafsi","computation":null,"evidence_status":"Loaded"}],"r1":{"records":7,"source_groups":2,"associated_assertion_union":4,"upstream_associations":["lexical-upstream-a","lexical-upstream-b"],"owned_main_associations":["lexical-owned-a","lexical-owned-b"],"owned_empty_associations":[],"reverse_grow_order":["lexical-owned-a","lexical-upstream-a","lexical-upstream-b"],"sum_repeated_form_associations":10},"r2":{"records":7,"source_groups":2,"associated_assertion_union":3,"upstream_associations":["lexical-upstream-a","lexical-upstream-b"],"owned_main_associations":["lexical-owned-c"],"owned_empty_associations":[],"reverse_grow_order":["lexical-upstream-a","lexical-upstream-b"],"sum_repeated_form_associations":9},"definitions":{"lexical-upstream-a":"GROW <b>literal</b>","lexical-upstream-b":"grow","lexical-owned-a":"grow","lexical-owned-b":"develop","lexical-owned-c":"flourish"},"scope":"PRE-input literal identity/count/order authority; compare whole returned records directly to typed emitted inputs. No candidate export or same-engine oracle."}"#;

const LEXICAL_CURSOR_PROTOCOL_ORACLE_JSON: &[u8] = br##"{
  "scope": "PURE pre-input protocol oracle, fixed artificial identity only; no product run/golden livebinding claim",
  "query_payload_utf8": "{\"query\":{\"operation\":\"form\",\"value\":\"farvi-form\",\"role\":null,\"kind\":null},\"selectors\":[\"LU\",\"LO\"],\"resolved_sources\":[\"lexical-owned\",\"lexical-upstream\"],\"alias_scope\":\"ActiveOnly\"}",
  "query_sha256": "670ff975f8187867a8901d6b547472cef8d49363470fdcc70538a1cc03c550dd",
  "reversed_selectors_query_payload_utf8": "{\"query\":{\"operation\":\"form\",\"value\":\"farvi-form\",\"role\":null,\"kind\":null},\"selectors\":[\"LO\",\"LU\"],\"resolved_sources\":[\"lexical-owned\",\"lexical-upstream\"],\"alias_scope\":\"ActiveOnly\"}",
  "reversed_selectors_query_sha256": "426379cde0fe8cf91ef19bbb56fd7266f755c1db1f28be3830878226f1ae25d7",
  "body_utf8": "{\"version\":1,\"domain\":\"workbench-lexical-cursor-v1\",\"format\":\"compact\",\"binding_sha256\":\"1111111111111111111111111111111111111111111111111111111111111111\",\"query_sha256\":\"670ff975f8187867a8901d6b547472cef8d49363470fdcc70538a1cc03c550dd\",\"last\":{\"operation\":\"form\",\"source\":\"lexical-owned\",\"entry_spelling\":\"farvi\",\"spelling\":\"farvi-form\",\"kind\":\"ComputedStem\",\"form_id\":\"lexical-owned-form-stem\"}}",
  "checksum": "3b3f73718cdefcf6d7063d50bc1f69140152b865a867e1289a576c4ad9f206f2",
  "envelope_utf8": "{\"body\":{\"version\":1,\"domain\":\"workbench-lexical-cursor-v1\",\"format\":\"compact\",\"binding_sha256\":\"1111111111111111111111111111111111111111111111111111111111111111\",\"query_sha256\":\"670ff975f8187867a8901d6b547472cef8d49363470fdcc70538a1cc03c550dd\",\"last\":{\"operation\":\"form\",\"source\":\"lexical-owned\",\"entry_spelling\":\"farvi\",\"spelling\":\"farvi-form\",\"kind\":\"ComputedStem\",\"form_id\":\"lexical-owned-form-stem\"}},\"checksum\":\"3b3f73718cdefcf6d7063d50bc1f69140152b865a867e1289a576c4ad9f206f2\"}",
  "token": "workbench-lexical-v1:7b22626f6479223a7b2276657273696f6e223a312c22646f6d61696e223a22776f726b62656e63682d6c65786963616c2d637572736f722d7631222c22666f726d6174223a22636f6d70616374222c2262696e64696e675f736861323536223a2231313131313131313131313131313131313131313131313131313131313131313131313131313131313131313131313131313131313131313131313131313131222c2271756572795f736861323536223a2236373066663937356638313837383637613839303164366235343734373263656638643439333633343730666463633730353338613163633033633535306464222c226c617374223a7b226f7065726174696f6e223a22666f726d222c22736f75726365223a226c65786963616c2d6f776e6564222c22656e7472795f7370656c6c696e67223a226661727669222c227370656c6c696e67223a2266617276692d666f726d222c226b696e64223a22436f6d70757465645374656d222c22666f726d5f6964223a226c65786963616c2d6f776e65642d666f726d2d7374656d227d7d2c22636865636b73756d223a2233623366373337313863646566636636643730363364353062633166363931343031353262383635613836376531323839613537366334616439663230366632227d"
}
"##;

fn value(text: &str) -> TextValue {
    TextValue {
        text: text.into(),
        language: Presence::Missing,
        markup: Presence::Missing,
    }
}

fn owned_release(
    id: &str,
    component: &EntityId,
    captures: Vec<CaptureRef>,
    assertions: Vec<EntityId>,
) -> Result<ReleaseRecord, Box<dyn std::error::Error>> {
    let count = assertions.len() as u64;
    Ok(ReleaseRecord {
        id: EntityId::new(id)?,
        component: component.clone(),
        distributor: None,
        upstream_revision: Presence::Missing,
        distributor_revision: Presence::Missing,
        captured_at: Presence::Missing,
        time_precision: TimePrecision::Unknown,
        schema_version: 1,
        normalizer: "integrated-workbench-example".into(),
        normalizer_version: "1".into(),
        captures,
        assertions,
        accounting: ReleaseAccounting {
            input_occurrences: Some(count),
            normalized_occurrences: Some(count),
            query_excluded_occurrences: Some(0),
            unit: "occurrences".into(),
        },
        diagnostics: Vec::new(),
        selected: true,
        availability: Availability::Available,
        content_notices: vec![ContentNotice {
            text: Presence::Value("synthetic example data; no licence grant".into()),
            locator: Presence::Missing,
            license: Presence::Missing,
        }],
        redistribution: RedistributionStatus::Held,
        redistribution_evidence: Vec::new(),
    })
}

fn genesis() -> Result<Dictionary, Box<dyn std::error::Error>> {
    let source = SourceId::new("owned-example")?;
    let editor = ContributorId::new("example-editor")?;
    let collection = EntityId::new("owned-example-collection")?;
    let component = EntityId::new("owned-example-component")?;
    let revision = EntityId::new("owned-r0")?;
    let registry = SourceRegistry::from_document(RegistryDocument {
        sources: vec![SourceRecord {
            id: source.clone(),
            display_name: "Example owned dictionary".into(),
            abbreviation: "OWN".into(),
            abbreviation_kind: AbbreviationKind::ProjectAssigned,
            abbreviation_evidence: Some("Example-only selector".into()),
            origin: SourceOrigin::OwnedDraft,
        }],
        contributors: vec![ContributorRecord {
            id: editor.clone(),
            display_name: "Example editor".into(),
        }],
        ..RegistryDocument::default()
    })?;
    let mut metadata = DictionaryMetadata::default();
    metadata.collections.push(CollectionRecord {
        id: collection.clone(),
        source: source.clone(),
        display_name: "Example owned collection".into(),
        description: Presence::Value(value("Synthetic, explicitly owned example")),
    });
    metadata.components.push(ComponentRecord {
        id: component.clone(),
        collection,
        native_name: "example lexical entries".into(),
        languages: vec!["jbo".into()],
    });
    metadata.releases.push(owned_release(
        "owned-release-r0",
        &component,
        Vec::new(),
        Vec::new(),
    )?);
    metadata.owned_revisions.push(OwnedRevisionRecord {
        id: revision.clone(),
        source: source.clone(),
        parent: None,
        editor,
        timestamp: Presence::Missing,
        time_precision: TimePrecision::Unknown,
        assertions: Vec::new(),
        tombstones: Vec::new(),
        rationale: Presence::Value(value("Start with an empty active set")),
    });
    metadata.owned_heads.push(OwnedHead { source, revision });
    Ok(Dictionary::from_document(DictionaryDocument {
        registry,
        metadata,
        ..DictionaryDocument::default()
    })?)
}

fn edit(seed: &Dictionary) -> Result<Dictionary, Box<dyn std::error::Error>> {
    let mut document = seed.document();
    let source = SourceId::new("owned-example")?;
    let editor = ContributorId::new("example-editor")?;
    let component = document.metadata.components[0].id.clone();
    let capture_bytes = br#"{"word":"farvi","definitions":["grow","develop"]}"#.to_vec();
    let occurrences = [
        RawOccurrence {
            locator: "/a".into(),
            fields: vec![NativeField {
                name: "definition".into(),
                encoding: NativeEncoding::Text,
                raw_value: "grow".into(),
            }],
            raw_line: Some("farvi — grow".into()),
        },
        RawOccurrence {
            locator: "/b".into(),
            fields: vec![NativeField {
                name: "definition".into(),
                encoding: NativeEncoding::Text,
                raw_value: "develop".into(),
            }],
            raw_line: Some("farvi — develop".into()),
        },
    ];
    let capture = RawCapture::new(
        "owned-example-capture",
        CaptureMetadata {
            source_name: "Example author input".into(),
            format_label: "application/json".into(),
            locator: "integrated-workbench example".into(),
            revision: Presence::Missing,
            captured_at: Presence::Missing,
            time_precision: TimePrecision::Unknown,
            notice_locator: Presence::Missing,
        },
        capture_bytes,
        occurrences.to_vec(),
    );
    let capture_ref = capture.identity.clone();
    document.captures.push(capture);
    let refs = ["/a", "/b"].map(|locator| OccurrenceRef {
        capture: capture_ref.clone(),
        locator: locator.into(),
    });
    let mut registry = document.registry.document();
    registry.snapshots.push(SnapshotAttribution {
        snapshot: capture_ref.to_snapshot_identity()?,
        source: source.clone(),
        distributor: None,
    });
    for (index, occurrence) in refs.iter().enumerate() {
        registry.occurrences.push(OccurrenceAttribution {
            occurrence: occurrence.to_record_identity()?,
            contributor: Some(editor.clone()),
            native_entry_id: Some("farvi".into()),
            native_assertion_id: Some(format!("owned-{}", if index == 0 { "a" } else { "b" })),
        });
    }
    document.registry = SourceRegistry::from_document(registry)?;

    let entry = EntityId::new("owned-entry-farvi")?;
    let assertion_a = EntityId::new("owned-assertion-a")?;
    let assertion_b = EntityId::new("owned-assertion-b")?;
    document.metadata.entries.push(EntryRecord {
        id: entry.clone(),
        source: source.clone(),
        spelling: Presence::Value("farvi".into()),
        language: Presence::Value("jbo".into()),
        script: Presence::Missing,
        native_word_class: Presence::Missing,
        selmaho: Presence::Null,
        mapped_pos: Presence::Missing,
    });
    for (id, occurrence, definition) in [
        (assertion_a.clone(), refs[0].clone(), "grow"),
        (assertion_b.clone(), refs[1].clone(), "develop"),
    ] {
        document.metadata.assertions.push(AssertionRecord {
            id,
            entry: entry.clone(),
            occurrence,
            native_sense_id: Presence::Missing,
            language: Presence::Value("en".into()),
            definition: Presence::Value(value(definition)),
            source_score: Presence::Null,
            timestamps: Vec::new(),
            eligibility: Eligibility::Lexical,
        });
    }
    document.metadata.releases.push(owned_release(
        "owned-release-r1",
        &component,
        vec![capture_ref],
        vec![assertion_a.clone(), assertion_b.clone()],
    )?);
    let revision = EntityId::new("owned-r1")?;
    document.metadata.catalogue_revision = 1;
    document.metadata.owned_revisions.push(OwnedRevisionRecord {
        id: revision.clone(),
        source: SourceId::new("owned-example")?,
        parent: Some(EntityId::new("owned-r0")?),
        editor,
        timestamp: Presence::Missing,
        time_precision: TimePrecision::Unknown,
        assertions: vec![assertion_a, assertion_b],
        tombstones: Vec::new(),
        rationale: Presence::Value(value("Add two owned farvi definitions")),
    });
    document.metadata.owned_heads[0].revision = revision;
    Ok(Dictionary::from_document(document)?)
}

fn next_edit(previous: &Dictionary) -> Result<Dictionary, Box<dyn std::error::Error>> {
    let mut document = previous.document();
    let source = SourceId::new("owned-example")?;
    let editor = ContributorId::new("example-editor")?;
    let component = document.metadata.components[0].id.clone();
    let capture_bytes = br#"{"word":"farvi","definition":"flourish"}"#.to_vec();
    let occurrence = RawOccurrence {
        locator: "/c".into(),
        fields: vec![NativeField {
            name: "definition".into(),
            encoding: NativeEncoding::Text,
            raw_value: "flourish".into(),
        }],
        raw_line: Some("farvi — flourish".into()),
    };
    let capture = RawCapture::new(
        "owned-example-capture-r2",
        CaptureMetadata {
            source_name: "Example author input".into(),
            format_label: "application/json".into(),
            locator: "integrated-workbench example r2".into(),
            revision: Presence::Missing,
            captured_at: Presence::Missing,
            time_precision: TimePrecision::Unknown,
            notice_locator: Presence::Missing,
        },
        capture_bytes,
        vec![occurrence],
    );
    let capture_ref = capture.identity.clone();
    let occurrence_ref = OccurrenceRef {
        capture: capture_ref.clone(),
        locator: "/c".into(),
    };
    document.captures.push(capture);
    let mut registry = document.registry.document();
    registry.snapshots.push(SnapshotAttribution {
        snapshot: capture_ref.to_snapshot_identity()?,
        source: source.clone(),
        distributor: None,
    });
    registry.occurrences.push(OccurrenceAttribution {
        occurrence: occurrence_ref.to_record_identity()?,
        contributor: Some(editor.clone()),
        native_entry_id: Some("farvi".into()),
        native_assertion_id: Some("owned-c".into()),
    });
    document.registry = SourceRegistry::from_document(registry)?;

    let assertion_id = EntityId::new("owned-assertion-c")?;
    document.metadata.assertions.push(AssertionRecord {
        id: assertion_id.clone(),
        entry: EntityId::new("owned-entry-farvi")?,
        occurrence: occurrence_ref,
        native_sense_id: Presence::Missing,
        language: Presence::Value("en".into()),
        definition: Presence::Value(value("flourish")),
        source_score: Presence::Null,
        timestamps: Vec::new(),
        eligibility: Eligibility::Lexical,
    });
    document.metadata.releases.push(owned_release(
        "owned-release-r2",
        &component,
        vec![capture_ref],
        vec![assertion_id.clone()],
    )?);
    let revision = EntityId::new("owned-r2")?;
    let previous_revision = document.metadata.owned_revisions.last().unwrap();
    let mut active = previous_revision.assertions.clone();
    active.push(assertion_id);
    document.metadata.catalogue_revision = 2;
    document.metadata.owned_revisions.push(OwnedRevisionRecord {
        id: revision.clone(),
        source,
        parent: Some(previous_revision.id.clone()),
        editor,
        timestamp: Presence::Missing,
        time_precision: TimePrecision::Unknown,
        assertions: active,
        tombstones: Vec::new(),
        rationale: Presence::Value(value("Add a third competing farvi definition")),
    });
    document.metadata.owned_heads[0].revision = revision;
    Ok(Dictionary::from_document(document)?)
}

fn lexical_release(
    id: &str,
    component: &EntityId,
    captures: Vec<CaptureRef>,
    assertions: Vec<EntityId>,
) -> ReleaseRecord {
    let count = assertions.len() as u64;
    ReleaseRecord {
        id: EntityId::new(id).unwrap(),
        component: component.clone(),
        distributor: None,
        upstream_revision: Presence::Missing,
        distributor_revision: Presence::Missing,
        captured_at: Presence::Missing,
        time_precision: TimePrecision::Unknown,
        schema_version: 1,
        normalizer: "lexical-contract-fixture".into(),
        normalizer_version: "1".into(),
        captures,
        assertions,
        accounting: ReleaseAccounting {
            input_occurrences: Some(count),
            normalized_occurrences: Some(count),
            query_excluded_occurrences: Some(0),
            unit: "occurrences".into(),
        },
        diagnostics: Vec::new(),
        selected: true,
        availability: Availability::Available,
        content_notices: vec![ContentNotice {
            text: Presence::Value("synthetic lexical contract fixture".into()),
            locator: Presence::Missing,
            license: Presence::Missing,
        }],
        redistribution: RedistributionStatus::Held,
        redistribution_evidence: Vec::new(),
    }
}

fn lexical_upstream_dictionary() -> Dictionary {
    let source = SourceId::new("lexical-upstream").unwrap();
    let contributor = ContributorId::new("lexical-upstream-editor").unwrap();
    let collection = EntityId::new("lexical-upstream-collection").unwrap();
    let component = EntityId::new("lexical-upstream-component").unwrap();
    let entry = EntityId::new("lexical-upstream-entry").unwrap();
    let capture = RawCapture::new(
        "captured:lexical-upstream:r1",
        CaptureMetadata {
            source_name: "Lexical upstream fixture".into(),
            format_label: "text/plain".into(),
            locator: "upstream r1".into(),
            revision: Presence::Missing,
            captured_at: Presence::Missing,
            time_precision: TimePrecision::Unknown,
            notice_locator: Presence::Missing,
        },
        b"GROW <b>literal</b>\ngrow\n".to_vec(),
        vec![
            RawOccurrence {
                locator: "/0".into(),
                fields: Vec::new(),
                raw_line: Some("GROW <b>literal</b>".into()),
            },
            RawOccurrence {
                locator: "/1".into(),
                fields: Vec::new(),
                raw_line: Some("grow".into()),
            },
        ],
    );
    let capture_ref = capture.identity.clone();
    let occurrences = ["/0", "/1"].map(|locator| OccurrenceRef {
        capture: capture_ref.clone(),
        locator: locator.into(),
    });
    let old_capture = CaptureRef {
        namespace: "captured:lexical-upstream-old".into(),
        sha256: "11".repeat(32),
    };
    let registry = SourceRegistry::from_document(RegistryDocument {
        sources: vec![SourceRecord {
            id: source.clone(),
            display_name: "Lexical upstream fixture".into(),
            abbreviation: "LU".into(),
            abbreviation_kind: AbbreviationKind::ProjectAssigned,
            abbreviation_evidence: Some("S04 lexical fixture".into()),
            origin: SourceOrigin::Upstream,
        }],
        contributors: vec![ContributorRecord {
            id: contributor.clone(),
            display_name: "Lexical fixture editor".into(),
        }],
        snapshots: vec![SnapshotAttribution {
            snapshot: capture_ref.to_snapshot_identity().unwrap(),
            source: source.clone(),
            distributor: None,
        }],
        occurrences: occurrences
            .iter()
            .enumerate()
            .map(|(index, occurrence)| OccurrenceAttribution {
                occurrence: occurrence.to_record_identity().unwrap(),
                contributor: Some(contributor.clone()),
                native_entry_id: Some("lexical-upstream-entry".into()),
                native_assertion_id: Some(format!(
                    "lexical-upstream-{}",
                    if index == 0 { "a" } else { "b" }
                )),
            })
            .collect(),
        ..RegistryDocument::default()
    })
    .unwrap();

    let mut metadata = DictionaryMetadata::default();
    metadata.collections.push(CollectionRecord {
        id: collection.clone(),
        source: source.clone(),
        display_name: "Lexical upstream collection".into(),
        description: Presence::Missing,
    });
    metadata.components.push(ComponentRecord {
        id: component.clone(),
        collection,
        native_name: "farvi".into(),
        languages: vec!["jbo".into()],
    });
    let count = 2;
    let release = |id: &str,
                   captures: Vec<CaptureRef>,
                   assertions: Vec<EntityId>,
                   occurrence_count: u64,
                   availability| {
        ReleaseRecord {
            id: EntityId::new(id).unwrap(),
            component: component.clone(),
            distributor: None,
            upstream_revision: Presence::Missing,
            distributor_revision: Presence::Missing,
            captured_at: Presence::Missing,
            time_precision: TimePrecision::Unknown,
            schema_version: 1,
            normalizer: "lexical-contract-fixture".into(),
            normalizer_version: "1".into(),
            captures,
            assertions,
            accounting: ReleaseAccounting {
                input_occurrences: Some(occurrence_count),
                normalized_occurrences: Some(occurrence_count),
                query_excluded_occurrences: Some(0),
                unit: "occurrences".into(),
            },
            diagnostics: Vec::new(),
            selected: true,
            availability,
            content_notices: vec![ContentNotice {
                text: Presence::Value("synthetic lexical contract fixture".into()),
                locator: Presence::Missing,
                license: Presence::Missing,
            }],
            redistribution: RedistributionStatus::Held,
            redistribution_evidence: Vec::new(),
        }
    };
    let assertion_ids = [
        EntityId::new("lexical-upstream-a").unwrap(),
        EntityId::new("lexical-upstream-b").unwrap(),
    ];
    metadata.releases.push(release(
        "lexical-upstream-release-r1",
        vec![capture_ref.clone()],
        assertion_ids.to_vec(),
        count,
        Availability::Available,
    ));
    metadata.releases.push(release(
        "lexical-upstream-old-release",
        vec![old_capture.clone()],
        Vec::new(),
        0,
        Availability::Missing,
    ));
    metadata.entries.push(EntryRecord {
        id: entry.clone(),
        source: source.clone(),
        spelling: Presence::Value("farvi".into()),
        language: Presence::Value("jbo".into()),
        script: Presence::Missing,
        native_word_class: Presence::Missing,
        selmaho: Presence::Missing,
        mapped_pos: Presence::Missing,
    });
    metadata.assertions = vec![
        AssertionRecord {
            id: assertion_ids[0].clone(),
            entry: entry.clone(),
            occurrence: occurrences[0].clone(),
            native_sense_id: Presence::Missing,
            language: Presence::Value("en".into()),
            definition: Presence::Value(TextValue {
                text: "GROW <b>literal</b>".into(),
                language: Presence::Missing,
                markup: Presence::Missing,
            }),
            source_score: Presence::Missing,
            timestamps: Vec::new(),
            eligibility: Eligibility::Lexical,
        },
        AssertionRecord {
            id: assertion_ids[1].clone(),
            entry: entry.clone(),
            occurrence: occurrences[1].clone(),
            native_sense_id: Presence::Missing,
            language: Presence::Value("en".into()),
            definition: Presence::Value(TextValue {
                text: "grow".into(),
                language: Presence::Missing,
                markup: Presence::Missing,
            }),
            source_score: Presence::Missing,
            timestamps: Vec::new(),
            eligibility: Eligibility::Lexical,
        },
    ];
    let evidence = |occurrence: OccurrenceRef| {
        Some(SourceEvidence {
            occurrence,
            field_index: None,
            citation: Presence::Missing,
            offset: None,
        })
    };
    metadata.forms = vec![
        FormRecord {
            id: EntityId::new("lexical-upstream-form-variant").unwrap(),
            entry: entry.clone(),
            spelling: "farvi-form".into(),
            language: Presence::Value("jbo".into()),
            script: Presence::Missing,
            pronunciation: Presence::Missing,
            kind: FormKind::SourceVariant,
            standing: Presence::Missing,
            components: Vec::new(),
            evidence: evidence(OccurrenceRef {
                capture: old_capture,
                locator: "/old".into(),
            }),
            computation: None,
        },
        FormRecord {
            id: EntityId::new("lexical-upstream-form-pronunciation").unwrap(),
            entry: entry.clone(),
            spelling: "farvi-form".into(),
            language: Presence::Value("jbo".into()),
            script: Presence::Missing,
            pronunciation: Presence::Value(TextValue {
                text: "farvi".into(),
                language: Presence::Value("jbo".into()),
                markup: Presence::Missing,
            }),
            kind: FormKind::SourcePronunciation,
            standing: Presence::Missing,
            components: Vec::new(),
            evidence: evidence(occurrences[0].clone()),
            computation: None,
        },
        FormRecord {
            id: EntityId::new("lexical-upstream-form-reported").unwrap(),
            entry: entry.clone(),
            spelling: "farvi-form".into(),
            language: Presence::Value("jbo".into()),
            script: Presence::Missing,
            pronunciation: Presence::Missing,
            kind: FormKind::SourceReportedForm,
            standing: Presence::Missing,
            components: Vec::new(),
            evidence: evidence(occurrences[1].clone()),
            computation: None,
        },
        FormRecord {
            id: EntityId::new("lexical-upstream-form-rafsi").unwrap(),
            entry,
            spelling: "farvi-form".into(),
            language: Presence::Value("jbo".into()),
            script: Presence::Missing,
            pronunciation: Presence::Missing,
            kind: FormKind::AssignedRafsi,
            standing: Presence::Missing,
            components: Vec::new(),
            evidence: evidence(occurrences[0].clone()),
            computation: None,
        },
    ];
    Dictionary::from_document(DictionaryDocument {
        registry,
        captures: vec![capture],
        metadata,
        ..DictionaryDocument::default()
    })
    .expect("fixed LU typed records preserve loaded and unavailable evidence")
}

fn lexical_owned_genesis() -> Dictionary {
    let source = SourceId::new("lexical-owned").unwrap();
    let editor = ContributorId::new("lexical-editor").unwrap();
    let collection = EntityId::new("lexical-owned-collection").unwrap();
    let component = EntityId::new("lexical-owned-component").unwrap();
    let revision = EntityId::new("lexical-owned-r0").unwrap();
    let registry = SourceRegistry::from_document(RegistryDocument {
        sources: vec![SourceRecord {
            id: source.clone(),
            display_name: "Lexical owned fixture".into(),
            abbreviation: "LO".into(),
            abbreviation_kind: AbbreviationKind::ProjectAssigned,
            abbreviation_evidence: Some("test fixture".into()),
            origin: SourceOrigin::OwnedDraft,
        }],
        contributors: vec![ContributorRecord {
            id: editor.clone(),
            display_name: "Lexical fixture editor".into(),
        }],
        ..RegistryDocument::default()
    })
    .unwrap();
    let mut metadata = DictionaryMetadata::default();
    metadata.collections.push(CollectionRecord {
        id: collection,
        source: source.clone(),
        display_name: "Lexical owned collection".into(),
        description: Presence::Missing,
    });
    metadata.components.push(ComponentRecord {
        id: component.clone(),
        collection: EntityId::new("lexical-owned-collection").unwrap(),
        native_name: "farvi forms".into(),
        languages: vec!["jbo".into()],
    });
    metadata.releases.push(lexical_release(
        "lexical-owned-release-r0",
        &component,
        Vec::new(),
        Vec::new(),
    ));
    metadata.owned_revisions.push(OwnedRevisionRecord {
        id: revision.clone(),
        source: source.clone(),
        parent: None,
        editor,
        timestamp: Presence::Missing,
        time_precision: TimePrecision::Unknown,
        assertions: Vec::new(),
        tombstones: Vec::new(),
        rationale: Presence::Value(TextValue {
            text: "empty genesis".into(),
            language: Presence::Missing,
            markup: Presence::Missing,
        }),
    });
    metadata.owned_heads.push(OwnedHead { source, revision });
    Dictionary::from_document(DictionaryDocument {
        registry,
        metadata,
        ..DictionaryDocument::default()
    })
    .unwrap()
}

fn lexical_owned_revision_one(genesis: &Dictionary) -> Dictionary {
    let source = SourceId::new("lexical-owned").unwrap();
    let editor = ContributorId::new("lexical-editor").unwrap();
    let capture = RawCapture::new(
        "captured:lexical-owned:r1",
        CaptureMetadata {
            source_name: "Lexical owned fixture".into(),
            format_label: "text/plain".into(),
            locator: "owned r1".into(),
            revision: Presence::Missing,
            captured_at: Presence::Missing,
            time_precision: TimePrecision::Unknown,
            notice_locator: Presence::Missing,
        },
        b"farvi grow\nfarvi develop\n".to_vec(),
        vec![
            RawOccurrence {
                locator: "/a".into(),
                fields: Vec::new(),
                raw_line: Some("farvi grow".into()),
            },
            RawOccurrence {
                locator: "/b".into(),
                fields: Vec::new(),
                raw_line: Some("farvi develop".into()),
            },
        ],
    );
    let capture_ref = capture.identity.clone();
    let occurrences = ["/a", "/b"].map(|locator| OccurrenceRef {
        capture: capture_ref.clone(),
        locator: locator.into(),
    });
    let mut document = genesis.document();
    document.captures.push(capture);
    let mut registry = document.registry.document();
    registry.snapshots.push(SnapshotAttribution {
        snapshot: capture_ref.to_snapshot_identity().unwrap(),
        source: source.clone(),
        distributor: None,
    });
    for (index, occurrence) in occurrences.iter().enumerate() {
        registry.occurrences.push(OccurrenceAttribution {
            occurrence: occurrence.to_record_identity().unwrap(),
            contributor: Some(editor.clone()),
            native_entry_id: Some("lexical-owned-entry".into()),
            native_assertion_id: Some(format!(
                "lexical-owned-{}",
                if index == 0 { "a" } else { "b" }
            )),
        });
    }
    document.registry = SourceRegistry::from_document(registry).unwrap();
    let main_entry = EntityId::new("lexical-owned-entry").unwrap();
    let empty_entry = EntityId::new("lexical-owned-empty-entry").unwrap();
    document.metadata.entries = vec![
        EntryRecord {
            id: main_entry.clone(),
            source: source.clone(),
            spelling: Presence::Value("farvi".into()),
            language: Presence::Value("jbo".into()),
            script: Presence::Missing,
            native_word_class: Presence::Missing,
            selmaho: Presence::Missing,
            mapped_pos: Presence::Missing,
        },
        EntryRecord {
            id: empty_entry.clone(),
            source: source.clone(),
            spelling: Presence::Value("zero-associated".into()),
            language: Presence::Value("jbo".into()),
            script: Presence::Missing,
            native_word_class: Presence::Missing,
            selmaho: Presence::Missing,
            mapped_pos: Presence::Missing,
        },
    ];
    let assertion_a = EntityId::new("lexical-owned-a").unwrap();
    let assertion_b = EntityId::new("lexical-owned-b").unwrap();
    document.metadata.assertions = vec![
        AssertionRecord {
            id: assertion_a.clone(),
            entry: main_entry.clone(),
            occurrence: occurrences[0].clone(),
            native_sense_id: Presence::Missing,
            language: Presence::Value("en".into()),
            definition: Presence::Value(TextValue {
                text: "grow".into(),
                language: Presence::Missing,
                markup: Presence::Missing,
            }),
            source_score: Presence::Missing,
            timestamps: Vec::new(),
            eligibility: Eligibility::Lexical,
        },
        AssertionRecord {
            id: assertion_b.clone(),
            entry: main_entry.clone(),
            occurrence: occurrences[1].clone(),
            native_sense_id: Presence::Missing,
            language: Presence::Value("en".into()),
            definition: Presence::Value(TextValue {
                text: "develop".into(),
                language: Presence::Missing,
                markup: Presence::Missing,
            }),
            source_score: Presence::Missing,
            timestamps: Vec::new(),
            eligibility: Eligibility::Lexical,
        },
    ];
    let computation_ids = [
        EntityId::new("lexical-owned-computation-stem").unwrap(),
        EntityId::new("lexical-owned-computation-components").unwrap(),
        EntityId::new("lexical-owned-computation-candidate").unwrap(),
    ];
    document.metadata.computations = computation_ids
        .iter()
        .map(|id| ComputationRecord {
            id: id.clone(),
            tool: "fixture".into(),
            tool_version: "1".into(),
            inputs: vec![InputRef::Literal("farvi".into())],
            policy: "fixture-literal".into(),
            profile: Presence::Missing,
            explanation: Presence::Missing,
        })
        .collect();
    document.metadata.forms = vec![
        FormRecord {
            id: EntityId::new("lexical-owned-form-stem").unwrap(),
            entry: main_entry,
            spelling: "farvi-form".into(),
            language: Presence::Value("jbo".into()),
            script: Presence::Missing,
            pronunciation: Presence::Missing,
            kind: FormKind::ComputedStem,
            standing: Presence::Missing,
            components: Vec::new(),
            evidence: None,
            computation: Some(computation_ids[0].clone()),
        },
        FormRecord {
            id: EntityId::new("lexical-owned-form-components").unwrap(),
            entry: empty_entry.clone(),
            spelling: "farvi-form".into(),
            language: Presence::Value("jbo".into()),
            script: Presence::Missing,
            pronunciation: Presence::Missing,
            kind: FormKind::ComputedComponents,
            standing: Presence::Missing,
            components: Vec::new(),
            evidence: None,
            computation: Some(computation_ids[1].clone()),
        },
        FormRecord {
            id: EntityId::new("lexical-owned-form-candidate").unwrap(),
            entry: empty_entry,
            spelling: "farvi-form".into(),
            language: Presence::Value("jbo".into()),
            script: Presence::Missing,
            pronunciation: Presence::Missing,
            kind: FormKind::GeneratedCandidate,
            standing: Presence::Missing,
            components: Vec::new(),
            evidence: None,
            computation: Some(computation_ids[2].clone()),
        },
    ];
    let component = document.metadata.components[0].id.clone();
    document.metadata.releases.push(lexical_release(
        "lexical-owned-release-r1",
        &component,
        vec![capture_ref],
        vec![assertion_a.clone(), assertion_b.clone()],
    ));
    let revision = EntityId::new("lexical-owned-r1").unwrap();
    document.metadata.catalogue_revision = 1;
    document.metadata.owned_revisions.push(OwnedRevisionRecord {
        id: revision.clone(),
        source: source.clone(),
        parent: Some(EntityId::new("lexical-owned-r0").unwrap()),
        editor,
        timestamp: Presence::Missing,
        time_precision: TimePrecision::Unknown,
        assertions: vec![assertion_a, assertion_b],
        tombstones: Vec::new(),
        rationale: Presence::Value(TextValue {
            text: "add a and b".into(),
            language: Presence::Missing,
            markup: Presence::Missing,
        }),
    });
    document.metadata.owned_heads[0].revision = revision;
    Dictionary::from_document(document).unwrap()
}

fn lexical_owned_revision_two(previous: &Dictionary) -> Dictionary {
    let mut document = previous.document();
    let source = SourceId::new("lexical-owned").unwrap();
    let editor = ContributorId::new("lexical-editor").unwrap();
    let capture = RawCapture::new(
        "captured:lexical-owned:r2",
        CaptureMetadata {
            source_name: "Lexical owned fixture".into(),
            format_label: "text/plain".into(),
            locator: "owned r2".into(),
            revision: Presence::Missing,
            captured_at: Presence::Missing,
            time_precision: TimePrecision::Unknown,
            notice_locator: Presence::Missing,
        },
        b"farvi flourish\n".to_vec(),
        vec![RawOccurrence {
            locator: "/c".into(),
            fields: Vec::new(),
            raw_line: Some("farvi flourish".into()),
        }],
    );
    let capture_ref = capture.identity.clone();
    let occurrence = OccurrenceRef {
        capture: capture_ref.clone(),
        locator: "/c".into(),
    };
    document.captures.push(capture);
    let mut registry = document.registry.document();
    registry.snapshots.push(SnapshotAttribution {
        snapshot: capture_ref.to_snapshot_identity().unwrap(),
        source: source.clone(),
        distributor: None,
    });
    registry.occurrences.push(OccurrenceAttribution {
        occurrence: occurrence.to_record_identity().unwrap(),
        contributor: Some(editor.clone()),
        native_entry_id: Some("lexical-owned-entry".into()),
        native_assertion_id: Some("lexical-owned-c".into()),
    });
    document.registry = SourceRegistry::from_document(registry).unwrap();
    let assertion = EntityId::new("lexical-owned-c").unwrap();
    document.metadata.assertions.push(AssertionRecord {
        id: assertion.clone(),
        entry: EntityId::new("lexical-owned-entry").unwrap(),
        occurrence,
        native_sense_id: Presence::Missing,
        language: Presence::Value("en".into()),
        definition: Presence::Value(TextValue {
            text: "flourish".into(),
            language: Presence::Missing,
            markup: Presence::Missing,
        }),
        source_score: Presence::Missing,
        timestamps: Vec::new(),
        eligibility: Eligibility::Lexical,
    });
    let component = document.metadata.components[0].id.clone();
    document.metadata.releases.push(lexical_release(
        "lexical-owned-release-r2",
        &component,
        vec![capture_ref],
        vec![assertion.clone()],
    ));
    let previous_revision = document.metadata.owned_revisions.last().unwrap().clone();
    let revision = EntityId::new("lexical-owned-r2").unwrap();
    document.metadata.catalogue_revision = 2;
    document.metadata.owned_revisions.push(OwnedRevisionRecord {
        id: revision.clone(),
        source,
        parent: Some(previous_revision.id),
        editor,
        timestamp: Presence::Missing,
        time_precision: TimePrecision::Unknown,
        assertions: vec![assertion],
        tombstones: vec![
            EntityId::new("lexical-owned-a").unwrap(),
            EntityId::new("lexical-owned-b").unwrap(),
        ],
        rationale: Presence::Value(TextValue {
            text: "replace a and b with c".into(),
            language: Presence::Missing,
            markup: Presence::Missing,
        }),
    });
    document.metadata.owned_heads[0].revision = revision;
    Dictionary::from_document(document).unwrap()
}

fn build_lexical_view(
    root: &std::path::Path,
    upstream: &Dictionary,
    owned_root: &std::path::Path,
    owned_revision: WorkbenchOwnedRevision,
    view_name: &str,
) -> PathBuf {
    let catalogue = root.join("catalogue");
    if !catalogue.exists() {
        let sealed = seal_release(PackId::LaLojban, PackPayloadKind::Synthetic, upstream).unwrap();
        let request = CollectionRequest::new(
            vec![CollectionInput::SealedBytes {
                pack_id: PackId::LaLojban,
                release: Cow::Borrowed(&sealed.release),
                seal: Cow::Borrowed(&sealed.seal),
            }],
            CollectionSelection::Packs(vec![PackId::LaLojban]),
            true,
        )
        .unwrap();
        CollectionCatalogue::install(&catalogue, request).unwrap();
    }
    let view_root = root.join(view_name);
    WorkbenchView::build_new(
        &view_root,
        WorkbenchBuildRequest {
            catalogue_root: Some(catalogue),
            owned_root: Some(owned_root.to_path_buf()),
            selection: WorkbenchSelection {
                version: 1,
                packs: vec![PackId::LaLojban],
                disabled: Vec::new(),
                owned: Some(owned_revision),
                contexts: Vec::new(),
            },
        },
    )
    .unwrap();
    view_root
}

fn run_lexical_journey(
    output_root: &Path,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let root = output_root.join("lexical-workbench");
    fs::create_dir(&root)?;
    let artifacts = output_root.join("artifacts");
    let inputs = artifacts.join("lexical-inputs");
    fs::create_dir(&inputs)?;

    let upstream = lexical_upstream_dictionary();
    let seed = lexical_owned_genesis();
    let revision_one = lexical_owned_revision_one(&seed);
    let revision_two = lexical_owned_revision_two(&revision_one);
    let selection: WorkbenchSelection = serde_json::from_slice(LEXICAL_SELECTION_JSON)?;
    let mut historical_selection = selection.clone();
    historical_selection.owned = Some(WorkbenchOwnedRevision::Revision(EntityId::new(
        "lexical-owned-r1",
    )?));
    let form_request: WorkbenchLexicalRequest = serde_json::from_slice(LEXICAL_FORM_REQUEST_JSON)?;
    let reverse_request: WorkbenchLexicalRequest =
        serde_json::from_slice(LEXICAL_REVERSE_REQUEST_JSON)?;
    let sealed = seal_release(PackId::LaLojban, PackPayloadKind::Synthetic, &upstream)?;

    // The complete lexical input set is durable before installing its
    // catalogue, creating the journal, or building a workbench database.
    write_new(
        &inputs.join("lexical-upstream.json"),
        &upstream.to_json_bytes()?,
    )?;
    write_new(
        &inputs.join("lexical-owned-seed.json"),
        &seed.to_json_bytes()?,
    )?;
    write_new(
        &inputs.join("lexical-owned-r1.json"),
        &revision_one.to_json_bytes()?,
    )?;
    write_new(
        &inputs.join("lexical-owned-r2.json"),
        &revision_two.to_json_bytes()?,
    )?;
    write_new(
        &inputs.join("lexical-selection.json"),
        &serde_json::to_vec_pretty(&selection)?,
    )?;
    write_new(
        &inputs.join("lexical-historical-selection.json"),
        &serde_json::to_vec_pretty(&historical_selection)?,
    )?;
    write_new(
        &inputs.join("reverse-grow.json"),
        LEXICAL_REVERSE_REQUEST_JSON,
    )?;
    write_new(&inputs.join("form-all.json"), LEXICAL_FORM_REQUEST_JSON)?;
    write_new(&inputs.join("lexical-expected.json"), LEXICAL_EXPECTED_JSON)?;
    write_new(
        &inputs.join("cursor-protocol-oracle.json"),
        LEXICAL_CURSOR_PROTOCOL_ORACLE_JSON,
    )?;

    let catalogue_root = root.join("catalogue");
    let owned_root = root.join("owned");
    let request = CollectionRequest::new(
        vec![CollectionInput::SealedBytes {
            pack_id: PackId::LaLojban,
            release: Cow::Borrowed(&sealed.release),
            seal: Cow::Borrowed(&sealed.seal),
        }],
        CollectionSelection::Packs(vec![PackId::LaLojban]),
        true,
    )?;
    CollectionCatalogue::install(&catalogue_root, request)?;
    let (mut writer, created) = OwnedJournal::create(&owned_root, &seed)?;
    if created.head.sequence != 0 {
        return Err("the lexical owned journal must begin at sequence zero".into());
    }
    let committed_r1 = writer.commit(&created.head.revision.id, &revision_one)?;
    if committed_r1.head.sequence != 1 {
        return Err("the lexical r1 publication must be sequence one".into());
    }

    let view_r1 = build_lexical_view(
        &root,
        &upstream,
        &owned_root,
        WorkbenchOwnedRevision::Head,
        "view-r1",
    );
    let mut engine_r1 = WorkbenchSearchEngine::open(&view_r1)?;
    let form_r1 = engine_r1.execute_lexical(&form_request)?;
    let WorkbenchLexicalFullResult::Form(forms_r1) = &form_r1.result else {
        return Err("the fixed Form request did not return Form records".into());
    };
    if forms_r1.totals.records != 7
        || forms_r1.totals.source_groups != 2
        || forms_r1.totals.associated_assertions != 4
        || forms_r1
            .groups
            .iter()
            .map(|group| group.source.id.as_str())
            .collect::<Vec<_>>()
            != ["lexical-owned", "lexical-upstream"]
    {
        return Err("the r1 Form result differs from the fixed lexical counts/order".into());
    }
    let r1_input = revision_one.document();
    let upstream_input = upstream.document();
    let expected_form_ids = [
        "lexical-owned-form-stem",
        "lexical-owned-form-components",
        "lexical-owned-form-candidate",
        "lexical-upstream-form-variant",
        "lexical-upstream-form-pronunciation",
        "lexical-upstream-form-reported",
        "lexical-upstream-form-rafsi",
    ];
    let returned_form_ids = forms_r1
        .groups
        .iter()
        .flat_map(|group| group.hits.iter().map(|hit| hit.form.id.as_str()))
        .collect::<Vec<_>>();
    if returned_form_ids != expected_form_ids {
        return Err("the returned Forms differ from the input-derived fixed order".into());
    }
    for group in &forms_r1.groups {
        let input = if group.source.id.as_str() == "lexical-owned" {
            &r1_input
        } else {
            &upstream_input
        };
        for row in &group.hits {
            let expected = input
                .metadata
                .forms
                .iter()
                .find(|candidate| candidate.id == row.form.id)
                .ok_or("a returned Form is absent from its emitted input")?;
            if &row.form != expected {
                return Err("a returned complete Form differs from its emitted input".into());
            }
            let expected_entry = input
                .metadata
                .entries
                .iter()
                .find(|candidate| candidate.id == row.entry.id)
                .ok_or("a returned Form Entry is absent from its emitted input")?;
            if &row.entry != expected_entry {
                return Err("a returned complete Entry differs from its emitted input".into());
            }
            for associated in &row.associated_assertions {
                let expected_assertion = input
                    .metadata
                    .assertions
                    .iter()
                    .find(|candidate| candidate.id == associated.assertion.id)
                    .ok_or("a returned associated Assertion is absent from its emitted input")?;
                if &associated.assertion != expected_assertion {
                    return Err(
                        "a complete associated Assertion differs from its emitted input".into(),
                    );
                }
            }
            if let Some(computation) = &row.computation {
                let expected_computation = input
                    .metadata
                    .computations
                    .iter()
                    .find(|candidate| candidate.id == computation.id)
                    .ok_or("a returned Computation is absent from its emitted input")?;
                if computation != expected_computation {
                    return Err("a returned Computation differs from its emitted input".into());
                }
            }
            if let Some(evidence) = &row.evidence {
                let expected_evidence =
                    row.form.evidence.as_ref().ok_or(
                        "a Form with a returned evidence projection has no input evidence",
                    )?;
                if &evidence.evidence != expected_evidence {
                    return Err("returned Form evidence differs from its emitted input".into());
                }
                let absent_variant = row.form.id.as_str() == "lexical-upstream-form-variant";
                if (evidence.status == "CatalogueOnly") != absent_variant
                    || evidence.capture.is_some() == absent_variant
                    || evidence.raw.is_some() == absent_variant
                {
                    return Err("loaded and catalogue-only Form evidence states disagree".into());
                }
            }
        }
    }
    let compact_r1 = engine_r1.execute_lexical_compact(&form_request)?;
    let WorkbenchLexicalCompactResult::Form(compact_forms_r1) = &compact_r1.result else {
        return Err("the fixed Compact Form request returned another operation".into());
    };
    if compact_forms_r1.totals != forms_r1.totals {
        return Err("Full and Compact Form totals differ for the same r1 request".into());
    }
    let form_binding = compact_r1.binding_sha256.clone();
    let mut r1_details = Vec::new();
    for compact_group in &compact_forms_r1.groups {
        for compact_form in &compact_group.hits {
            let detail = engine_r1.form_detail(&compact_form.detail_ref, &form_binding)?;
            let expected = r1_input
                .metadata
                .forms
                .iter()
                .chain(upstream_input.metadata.forms.iter())
                .find(|candidate| candidate.id == compact_form.form_id)
                .ok_or("Compact returned a Form absent from both typed inputs")?;
            if &detail.form.form != expected {
                return Err("Form detail differs from the complete emitted Form row".into());
            }
            r1_details.push(detail);
        }
    }
    let reverse_r1 = engine_r1.execute_lexical(&reverse_request)?;
    let WorkbenchLexicalFullResult::ReverseDefinition(reverse_rows_r1) = &reverse_r1.result else {
        return Err("the fixed reverse-definition request returned another operation".into());
    };
    let reverse_ids = reverse_rows_r1
        .groups
        .iter()
        .flat_map(|group| group.hits.iter().map(|hit| hit.assertion.id.as_str()))
        .collect::<Vec<_>>();
    if reverse_rows_r1.totals.records != 3
        || reverse_ids
            != [
                "lexical-owned-a",
                "lexical-upstream-a",
                "lexical-upstream-b",
            ]
    {
        return Err("the r1 reverse-definition result differs from its typed inputs".into());
    }

    let profile_config = root.join("workbench-profiles.json");
    let configured = WorkbenchProfiles::configure(
        &profile_config,
        WorkbenchProfileConfigureRequest {
            name: "lexical".into(),
            catalogue_root: Some(catalogue_root.clone()),
            owned_root: Some(owned_root.clone()),
            selection: selection.clone(),
            make_default: true,
        },
        None,
    )?;
    let configured_refresh = WorkbenchProfiles::refresh(
        &profile_config,
        None,
        root.join("configured-r1"),
        configured.revision,
    )?;
    let mut named_engine = ConfiguredWorkbenchSearchEngine::open(&profile_config, Some("lexical"))?;
    let configured_form_r1 = named_engine.execute_lexical(&form_request)?;
    if serde_json::to_vec(&configured_form_r1.result)? != serde_json::to_vec(&form_r1.result)? {
        return Err(
            "configured r1 Form output differs from the direct input-derived output".into(),
        );
    }
    let configured_compact_r1 = named_engine.execute_lexical_compact(&form_request)?;
    let WorkbenchLexicalCompactResult::Form(configured_compact_forms_r1) =
        &configured_compact_r1.result
    else {
        return Err("configured Compact Form request returned another operation".into());
    };
    let configured_detail_r1 = named_engine.form_detail(
        &configured_compact_forms_r1.groups[0].hits[0].detail_ref,
        &configured_compact_r1.binding_sha256,
    )?;
    if configured_detail_r1.form.form.id.as_str() != "lexical-owned-form-stem" {
        return Err("configured detail did not resolve the first emitted Compact Form".into());
    }
    let mut default_engine = ConfiguredWorkbenchSearchEngine::open(&profile_config, None)?;
    let default_form_r1 = default_engine.execute_lexical(&form_request)?;
    if serde_json::to_vec(&default_form_r1.result)?
        != serde_json::to_vec(&configured_form_r1.result)?
        || default_form_r1.binding_sha256 == configured_form_r1.binding_sha256
        || default_form_r1.configuration.requested_profile.is_some()
    {
        return Err("named and default r1 outputs or effective identities disagree".into());
    }

    let committed_r2 = writer.commit(&committed_r1.head.revision.id, &revision_two)?;
    if committed_r2.head.sequence != 2 {
        return Err("the lexical r2 publication must be sequence two".into());
    }
    drop(writer);
    if default_engine.execute_lexical(&form_request).is_ok() {
        return Err("the retained default r1 engine remained current after the r2 append".into());
    }
    let refreshed_r2 = WorkbenchProfiles::refresh(
        &profile_config,
        Some("lexical"),
        root.join("configured-r2"),
        configured_refresh.publication.revision,
    )?;
    if refreshed_r2
        .publication
        .active
        .as_ref()
        .map(|prepared| prepared.sequence)
        != Some(1)
    {
        return Err("the configured r2 refresh did not append prepared sequence one".into());
    }
    let mut named_r2_engine =
        ConfiguredWorkbenchSearchEngine::open(&profile_config, Some("lexical"))?;
    let configured_form_r2 = named_r2_engine.execute_lexical(&form_request)?;
    let configured_compact_r2 = named_r2_engine.execute_lexical_compact(&form_request)?;
    let WorkbenchLexicalCompactResult::Form(configured_compact_forms_r2) =
        &configured_compact_r2.result
    else {
        return Err("configured r2 Compact Form request returned another operation".into());
    };
    let configured_detail_r2 = named_r2_engine.form_detail(
        &configured_compact_forms_r2.groups[0].hits[0].detail_ref,
        &configured_compact_r2.binding_sha256,
    )?;
    let configured_detail_input = revision_two.document();
    let configured_detail_expected = configured_detail_input
        .metadata
        .forms
        .iter()
        .find(|form| form.id == configured_detail_r2.form.form.id)
        .ok_or("configured r2 detail Form is absent from the emitted owned input")?;
    let configured_detail_entry = configured_detail_input
        .metadata
        .entries
        .iter()
        .find(|entry| entry.id == configured_detail_r2.form.entry.id)
        .ok_or("configured r2 detail Entry is absent from the emitted owned input")?;
    if &configured_detail_r2.form.form != configured_detail_expected
        || &configured_detail_r2.form.entry != configured_detail_entry
    {
        return Err("configured r2 detail differs from its complete emitted input rows".into());
    }
    let default_r2_engine = ConfiguredWorkbenchSearchEngine::open(&profile_config, None)?;
    let mut default_r2_engine = default_r2_engine;
    let default_form_r2 = default_r2_engine.execute_lexical(&form_request)?;
    let history = WorkbenchProfiles::read(&profile_config)?.history(None)?;
    if serde_json::to_vec(&default_form_r2.result)?
        != serde_json::to_vec(&configured_form_r2.result)?
        || default_form_r2.binding_sha256 == configured_form_r2.binding_sha256
        || default_form_r2.configuration.requested_profile.is_some()
        || configured_detail_r2.form.form.id.as_str() != "lexical-owned-form-stem"
        || history.revision != refreshed_r2.publication.revision
        || history
            .prepared
            .iter()
            .map(|row| row.sequence)
            .collect::<Vec<_>>()
            != [0, 1]
    {
        return Err("configured r2 named/default outputs or retained history disagree".into());
    }
    let view_r2 = build_lexical_view(
        &root,
        &upstream,
        &owned_root,
        WorkbenchOwnedRevision::Head,
        "view-r2",
    );
    let mut engine_r2 = WorkbenchSearchEngine::open(&view_r2)?;
    let form_r2 = engine_r2.execute_lexical(&form_request)?;
    let WorkbenchLexicalFullResult::Form(forms_r2) = &form_r2.result else {
        return Err("the r2 Form request did not return Form records".into());
    };
    if serde_json::to_vec(&configured_form_r2.result)? != serde_json::to_vec(&form_r2.result)? {
        return Err(
            "configured r2 Form output differs from the direct emitted-input result".into(),
        );
    }
    let owned_associations = forms_r2
        .groups
        .iter()
        .find(|group| group.source.id.as_str() == "lexical-owned")
        .ok_or("r2 Form result lacks its owned source group")?
        .hits
        .iter()
        .find(|row| row.form.id.as_str() == "lexical-owned-form-stem")
        .ok_or("r2 Form result lacks the owned stem")?
        .associated_assertions
        .iter()
        .map(|hit| hit.assertion.id.as_str())
        .collect::<Vec<_>>();
    if forms_r2.totals.records != 7
        || forms_r2.totals.source_groups != 2
        || forms_r2.totals.associated_assertions != 3
        || forms_r2
            .groups
            .iter()
            .flat_map(|group| group.hits.iter().map(|row| row.form.id.as_str()))
            .collect::<Vec<_>>()
            != expected_form_ids
        || owned_associations != ["lexical-owned-c"]
    {
        return Err("r2 must expose only assertion c on the owned stem".into());
    }
    for group in &forms_r2.groups {
        let input = if group.source.id.as_str() == "lexical-owned" {
            revision_two.document()
        } else {
            upstream.document()
        };
        for row in &group.hits {
            let expected_form = input
                .metadata
                .forms
                .iter()
                .find(|candidate| candidate.id == row.form.id)
                .ok_or("an r2 returned Form is absent from its emitted input")?;
            let expected_entry = input
                .metadata
                .entries
                .iter()
                .find(|candidate| candidate.id == row.entry.id)
                .ok_or("an r2 returned Entry is absent from its emitted input")?;
            if &row.form != expected_form || &row.entry != expected_entry {
                return Err("an r2 complete Form or Entry differs from its emitted input".into());
            }
            for associated in &row.associated_assertions {
                let expected_assertion = input
                    .metadata
                    .assertions
                    .iter()
                    .find(|candidate| candidate.id == associated.assertion.id)
                    .ok_or("an r2 associated Assertion is absent from its emitted input")?;
                if &associated.assertion != expected_assertion {
                    return Err("an r2 complete Assertion differs from its emitted input".into());
                }
            }
        }
    }
    let historical_view = build_lexical_view(
        &root,
        &upstream,
        &owned_root,
        WorkbenchOwnedRevision::Revision(EntityId::new("lexical-owned-r1")?),
        "view-historical-r1",
    );
    let mut historical_engine = WorkbenchSearchEngine::open(&historical_view)?;
    let historical_reverse = historical_engine.execute_lexical(&reverse_request)?;
    let WorkbenchLexicalFullResult::ReverseDefinition(historical_rows) = &historical_reverse.result
    else {
        return Err("the historical r1 query returned another operation".into());
    };
    if historical_rows.totals.records != 3
        || historical_rows
            .groups
            .iter()
            .flat_map(|group| group.hits.iter().map(|hit| hit.assertion.id.as_str()))
            .collect::<Vec<_>>()
            != [
                "lexical-owned-a",
                "lexical-upstream-a",
                "lexical-upstream-b",
            ]
    {
        return Err("the retained historical r1 view lost its original matches".into());
    }
    let reverse_r2 = engine_r2.execute_lexical(&reverse_request)?;
    let WorkbenchLexicalFullResult::ReverseDefinition(reverse_rows_r2) = &reverse_r2.result else {
        return Err("the r2 reverse-definition request returned another operation".into());
    };
    if reverse_rows_r2.totals.records != 2 {
        return Err("r2 reverse-definition must retain the two upstream matches".into());
    }

    Ok(serde_json::json!({
        "input_root": inputs,
        "selection": selection,
        "profile_revision": configured.revision,
        "publication_revision": configured_refresh.publication.revision,
        "refreshed_r2_profile_revision": refreshed_r2.publication.revision,
        "profile_history": history,
        "owned_publication_sequences": [created.head.sequence, committed_r1.head.sequence, committed_r2.head.sequence],
        "direct_r1_binding_sha256": form_r1.binding_sha256,
        "configured_r1": configured_form_r1,
        "configured_compact_r1": configured_compact_r1,
        "configured_detail_r1": configured_detail_r1,
        "default_r1": default_form_r1,
        "full_form_r1": form_r1,
        "compact_form_r1": compact_r1,
        "form_details_r1": r1_details,
        "reverse_grow_r1": reverse_r1,
        "configured_r2": configured_form_r2,
        "configured_compact_r2": configured_compact_r2,
        "configured_detail_r2": configured_detail_r2,
        "default_r2": default_form_r2,
        "full_form_r2": form_r2,
        "reverse_grow_r2": reverse_r2,
        "historical_reverse_grow_r1": historical_reverse,
        "fixed_expected": serde_json::from_slice::<serde_json::Value>(LEXICAL_EXPECTED_JSON)?,
    }))
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}

// Self-contained copy of the validated fixed Links typed-input constructor.
fn links_id(value: &str) -> EntityId {
    EntityId::new(value).unwrap()
}
fn links_source(value: &str) -> SourceId {
    SourceId::new(value).unwrap()
}
fn links_text(value: &str) -> Presence<TextValue> {
    Presence::Value(TextValue {
        text: value.into(),
        language: Presence::Missing,
        markup: Presence::Missing,
    })
}
fn links_evidence(occurrence: OccurrenceRef, field_index: Option<usize>) -> Option<SourceEvidence> {
    Some(SourceEvidence {
        occurrence,
        field_index,
        citation: Presence::Missing,
        offset: None,
    })
}
fn links_release(
    id_value: &str,
    component: &EntityId,
    captures: Vec<CaptureRef>,
    assertions: Vec<EntityId>,
    selected: bool,
) -> ReleaseRecord {
    let n = assertions.len() as u64;
    ReleaseRecord {
        id: links_id(id_value),
        component: component.clone(),
        distributor: None,
        upstream_revision: Presence::Missing,
        distributor_revision: Presence::Missing,
        captured_at: Presence::Missing,
        time_precision: TimePrecision::Unknown,
        schema_version: 1,
        normalizer: "links-fixed-v1".into(),
        normalizer_version: "1".into(),
        captures,
        assertions,
        accounting: ReleaseAccounting {
            input_occurrences: Some(n),
            normalized_occurrences: Some(n),
            query_excluded_occurrences: Some(0),
            unit: "occurrences".into(),
        },
        diagnostics: Vec::new(),
        selected,
        availability: Availability::Available,
        content_notices: vec![ContentNotice {
            text: Presence::Value("synthetic Links fixture".into()),
            locator: Presence::Missing,
            license: Presence::Missing,
        }],
        redistribution: RedistributionStatus::Held,
        redistribution_evidence: Vec::new(),
    }
}
fn links_capture(
    namespace: &str,
    source_name: &str,
    lines: &[&str],
    fields: Vec<Vec<NativeField>>,
) -> RawCapture {
    let bytes = lines.join("\n").into_bytes();
    RawCapture::new(
        namespace,
        CaptureMetadata {
            source_name: source_name.into(),
            format_label: "synthetic-links-rows-v1".into(),
            locator: namespace.into(),
            revision: Presence::Missing,
            captured_at: Presence::Missing,
            time_precision: TimePrecision::Unknown,
            notice_locator: Presence::Missing,
        },
        bytes,
        lines
            .iter()
            .zip(fields)
            .enumerate()
            .map(|(i, (line, fields))| RawOccurrence {
                locator: format!("/{i}"),
                fields,
                raw_line: Some((*line).into()),
            })
            .collect(),
    )
}
fn links_field(name: &str, value: &str) -> NativeField {
    NativeField {
        name: name.into(),
        encoding: NativeEncoding::Text,
        raw_value: value.into(),
    }
}
fn links_entry(id_value: &str, source_id: &SourceId, spelling: &str) -> EntryRecord {
    EntryRecord {
        id: links_id(id_value),
        source: source_id.clone(),
        spelling: Presence::Value(spelling.into()),
        language: Presence::Value("jbo".into()),
        script: Presence::Missing,
        native_word_class: Presence::Missing,
        selmaho: Presence::Missing,
        mapped_pos: Presence::Missing,
    }
}
fn links_assertion(
    id_value: &str,
    entry_id: &EntityId,
    occurrence: OccurrenceRef,
    definition: &str,
) -> AssertionRecord {
    AssertionRecord {
        id: links_id(id_value),
        entry: entry_id.clone(),
        occurrence,
        native_sense_id: Presence::Missing,
        language: Presence::Value("en".into()),
        definition: links_text(definition),
        source_score: Presence::Missing,
        timestamps: Vec::new(),
        eligibility: Eligibility::Lexical,
    }
}
fn links_source_row(id_value: &str, abbreviation: &str, origin: SourceOrigin) -> SourceRecord {
    SourceRecord {
        id: links_source(id_value),
        display_name: id_value.into(),
        abbreviation: abbreviation.into(),
        abbreviation_kind: AbbreviationKind::ProjectAssigned,
        abbreviation_evidence: Some("fixed Links input".into()),
        origin,
    }
}
fn links_collection_component(
    source_id: &SourceId,
    collection_id: &str,
    component_id: &str,
) -> (CollectionRecord, ComponentRecord) {
    (
        CollectionRecord {
            id: links_id(collection_id),
            source: source_id.clone(),
            display_name: collection_id.into(),
            description: Presence::Missing,
        },
        ComponentRecord {
            id: links_id(component_id),
            collection: links_id(collection_id),
            native_name: component_id.into(),
            languages: vec!["jbo".into()],
        },
    )
}

fn links_owned_r0() -> Dictionary {
    let lo = links_source("links-owned");
    let editor = ContributorId::new("links-editor").unwrap();
    let (collection, component) =
        links_collection_component(&lo, "links-owned-collection", "links-owned-component");
    let registry = SourceRegistry::from_document(RegistryDocument {
        sources: vec![links_source_row(
            "links-owned",
            "KO",
            SourceOrigin::OwnedDraft,
        )],
        contributors: vec![ContributorRecord {
            id: editor.clone(),
            display_name: "Links fixture editor".into(),
        }],
        ..RegistryDocument::default()
    })
    .unwrap();
    let revision = links_id("links-owned-r0");
    let mut metadata = DictionaryMetadata::default();
    metadata.collections.push(collection);
    metadata.components.push(component.clone());
    metadata.releases.push(links_release(
        "links-owned-release-r0",
        &component.id,
        Vec::new(),
        Vec::new(),
        true,
    ));
    metadata.owned_revisions.push(OwnedRevisionRecord {
        id: revision.clone(),
        source: lo.clone(),
        parent: None,
        editor,
        timestamp: Presence::Missing,
        time_precision: TimePrecision::Unknown,
        assertions: Vec::new(),
        tombstones: Vec::new(),
        rationale: links_text("genesis"),
    });
    metadata.owned_heads.push(OwnedHead {
        source: lo,
        revision,
    });
    Dictionary::from_document(DictionaryDocument {
        registry,
        metadata,
        ..DictionaryDocument::default()
    })
    .unwrap()
}

fn links_owned_r1(previous: &Dictionary) -> Dictionary {
    let lo = links_source("links-owned");
    let editor = ContributorId::new("links-editor").unwrap();
    let mut doc = previous.document();
    let lines = [
        "seed | Links fixture a",
        "branch | Links fixture b",
        "leaf | Links fixture c",
    ];
    let capture = links_capture(
        "links-owned-capture-r1",
        "Links owned",
        &lines,
        lines
            .iter()
            .map(|line| {
                let (word, definition) = line.split_once(" | ").unwrap();
                vec![
                    links_field("word", word),
                    links_field("definition", definition),
                ]
            })
            .collect(),
    );
    let cap = capture.identity.clone();
    let occurrences = ["/0", "/1", "/2"].map(|locator| OccurrenceRef {
        capture: cap.clone(),
        locator: locator.into(),
    });
    doc.captures.push(capture);
    let mut registry = doc.registry.document();
    registry.snapshots.push(SnapshotAttribution {
        snapshot: cap.to_snapshot_identity().unwrap(),
        source: lo.clone(),
        distributor: None,
    });
    for (i, occurrence) in occurrences.iter().enumerate() {
        registry.occurrences.push(OccurrenceAttribution {
            occurrence: occurrence.to_record_identity().unwrap(),
            contributor: Some(editor.clone()),
            native_entry_id: Some(
                [
                    "links-owned-entry-a",
                    "links-owned-entry-b",
                    "links-owned-entry-c",
                ][i]
                    .into(),
            ),
            native_assertion_id: Some(
                ["links-owned-a", "links-owned-b", "links-owned-c"][i].into(),
            ),
        });
    }
    doc.registry = SourceRegistry::from_document(registry).unwrap();
    let ea = links_id("links-owned-entry-a");
    let eb = links_id("links-owned-entry-b");
    let ec = links_id("links-owned-entry-c");
    doc.metadata.entries = vec![
        links_entry("links-owned-entry-a", &lo, "seed"),
        links_entry("links-owned-entry-b", &lo, "branch"),
        links_entry("links-owned-entry-c", &lo, "leaf"),
    ];
    doc.metadata.assertions = vec![
        links_assertion(
            "links-owned-a",
            &ea,
            occurrences[0].clone(),
            "Links fixture a",
        ),
        links_assertion(
            "links-owned-b",
            &eb,
            occurrences[1].clone(),
            "Links fixture b",
        ),
        links_assertion(
            "links-owned-c",
            &ec,
            occurrences[2].clone(),
            "Links fixture c",
        ),
    ];
    let component = links_id("links-owned-component");
    doc.metadata.releases.push(links_release(
        "links-owned-release-r1",
        &component,
        vec![cap],
        vec![
            links_id("links-owned-a"),
            links_id("links-owned-b"),
            links_id("links-owned-c"),
        ],
        true,
    ));
    let rev = links_id("links-owned-r1");
    doc.metadata.owned_revisions.push(OwnedRevisionRecord {
        id: rev.clone(),
        source: lo.clone(),
        parent: Some(links_id("links-owned-r0")),
        editor: editor.clone(),
        timestamp: Presence::Missing,
        time_precision: TimePrecision::Unknown,
        assertions: vec![
            links_id("links-owned-a"),
            links_id("links-owned-b"),
            links_id("links-owned-c"),
        ],
        tombstones: Vec::new(),
        rationale: links_text("append a, b, c"),
    });
    doc.metadata.owned_heads[0].revision = rev;
    doc.metadata.catalogue_revision = 1;
    Dictionary::from_document(doc).unwrap()
}

fn links_owned_r2(previous: &Dictionary) -> Dictionary {
    let mut doc = previous.document();
    let editor = ContributorId::new("links-editor").unwrap();
    let component = links_id("links-owned-component");
    let cap = doc
        .captures
        .iter()
        .find(|row| row.identity.namespace == "links-owned-capture-r1")
        .unwrap()
        .identity
        .clone();
    doc.metadata.releases.push(links_release(
        "links-owned-release-r2",
        &component,
        vec![cap],
        vec![
            links_id("links-owned-a"),
            links_id("links-owned-b"),
            links_id("links-owned-c"),
        ],
        true,
    ));
    let rev = links_id("links-owned-r2");
    doc.metadata.owned_revisions.push(OwnedRevisionRecord {
        id: rev.clone(),
        source: links_source("links-owned"),
        parent: Some(links_id("links-owned-r1")),
        editor,
        timestamp: Presence::Missing,
        time_precision: TimePrecision::Unknown,
        assertions: vec![links_id("links-owned-a"), links_id("links-owned-b")],
        tombstones: vec![links_id("links-owned-c")],
        rationale: links_text("retire c"),
    });
    doc.metadata.owned_heads[0].revision = rev;
    doc.metadata.catalogue_revision = 2;
    Dictionary::from_document(doc).unwrap()
}

fn links_upstream_dictionary() -> Dictionary {
    let lu = links_source("links-upstream");
    let contributor = ContributorId::new("links-upstream-editor").unwrap();
    let (collection, component) =
        links_collection_component(&lu, "links-upstream-collection", "links-upstream-component");
    let lines = [
        "seed | Links fixture x | See also {branch} {branch} {missing}.",
        "branch | Links fixture y",
        "leaf | Links fixture z",
    ];
    let raw = links_capture(
        "links-upstream-capture-r1",
        "Links upstream",
        &lines,
        vec![
            vec![
                links_field("word", "seed"),
                links_field("definition", "Links fixture x"),
                links_field("notes", "See also {branch} {branch} {missing}."),
            ],
            vec![
                links_field("word", "branch"),
                links_field("definition", "Links fixture y"),
            ],
            vec![
                links_field("word", "leaf"),
                links_field("definition", "Links fixture z"),
            ],
        ],
    );
    let cap = raw.identity.clone();
    let occ = ["/0", "/1", "/2"].map(|locator| OccurrenceRef {
        capture: cap.clone(),
        locator: locator.into(),
    });
    let registry = SourceRegistry::from_document(RegistryDocument {
        sources: vec![links_source_row(
            "links-upstream",
            "KU",
            SourceOrigin::Upstream,
        )],
        contributors: vec![ContributorRecord {
            id: contributor.clone(),
            display_name: "Links upstream editor".into(),
        }],
        snapshots: vec![SnapshotAttribution {
            snapshot: cap.to_snapshot_identity().unwrap(),
            source: lu.clone(),
            distributor: None,
        }],
        occurrences: occ
            .iter()
            .enumerate()
            .map(|(i, occurrence)| OccurrenceAttribution {
                occurrence: occurrence.to_record_identity().unwrap(),
                contributor: Some(contributor.clone()),
                native_entry_id: [Some("seed-native"), Some("N"), None][i].map(str::to_owned),
                native_assertion_id: Some(["x-native", "y-native", "z-native"][i].into()),
            })
            .collect(),
        ..RegistryDocument::default()
    })
    .unwrap();
    let ex = links_id("links-upstream-entry-x");
    let ey = links_id("links-upstream-entry-y");
    let ez = links_id("links-upstream-entry-z");
    let ax = links_id("links-upstream-x");
    let ay = links_id("links-upstream-y");
    let az = links_id("links-upstream-z");
    let mut metadata = DictionaryMetadata::default();
    metadata.collections.push(collection);
    metadata.components.push(component.clone());
    metadata.releases.push(links_release(
        "links-upstream-release-r1",
        &component.id,
        vec![cap.clone()],
        vec![ax.clone(), ay.clone(), az.clone()],
        true,
    ));
    metadata.entries = vec![
        links_entry("links-upstream-entry-x", &lu, "seed"),
        links_entry("links-upstream-entry-y", &lu, "branch"),
        links_entry("links-upstream-entry-z", &lu, "leaf"),
        links_entry("links-upstream-entry-q", &lu, "branch"),
    ];
    metadata.assertions = vec![
        links_assertion("links-upstream-x", &ex, occ[0].clone(), "Links fixture x"),
        links_assertion("links-upstream-y", &ey, occ[1].clone(), "Links fixture y"),
        links_assertion("links-upstream-z", &ez, occ[2].clone(), "Links fixture z"),
    ];
    metadata.forms.push(vlajmaji_lexicon::FormRecord {
        id: links_id("links-upstream-form-z"),
        entry: ez,
        spelling: "leaf-form".into(),
        language: Presence::Value("jbo".into()),
        script: Presence::Missing,
        pronunciation: Presence::Value(TextValue {
            text: "leaf".into(),
            language: Presence::Value("jbo".into()),
            markup: Presence::Missing,
        }),
        kind: vlajmaji_lexicon::FormKind::SourceReportedForm,
        standing: Presence::Missing,
        components: Vec::new(),
        evidence: links_evidence(occ[2].clone(), None),
        computation: None,
    });
    let context_id = links_id("links-note-x");
    metadata.contexts.push(ContextRecord {
        id: context_id,
        assertion: ax.clone(),
        kind: ContextKind::Note,
        value: links_text("See also {branch} {branch} {missing}."),
        translated_from: None,
        evidence: SourceEvidence {
            occurrence: occ[0].clone(),
            field_index: Some(2),
            citation: Presence::Missing,
            offset: None,
        },
    });
    Dictionary::from_document(DictionaryDocument {
        registry,
        captures: vec![raw],
        metadata,
        ..DictionaryDocument::default()
    })
    .unwrap()
}

fn links_fixed_dictionary() -> Dictionary {
    let owned = links_owned_r1(&links_owned_r0());
    let upstream = links_upstream_dictionary();
    let mut doc = owned.document();
    let upstream_doc = upstream.document();
    let owned_registry = doc.registry.document();
    let upstream_registry = upstream_doc.registry.document();
    doc.registry = SourceRegistry::from_document(RegistryDocument {
        sources: owned_registry
            .sources
            .into_iter()
            .chain(upstream_registry.sources.iter().cloned())
            .collect(),
        contributors: owned_registry
            .contributors
            .into_iter()
            .chain(upstream_registry.contributors.iter().cloned())
            .collect(),
        snapshots: owned_registry
            .snapshots
            .into_iter()
            .chain(upstream_registry.snapshots.iter().cloned())
            .collect(),
        occurrences: owned_registry
            .occurrences
            .into_iter()
            .chain(upstream_registry.occurrences.iter().cloned())
            .collect(),
        ..RegistryDocument::default()
    })
    .unwrap();
    doc.captures.extend(upstream_doc.captures);
    doc.metadata
        .collections
        .extend(upstream_doc.metadata.collections);
    doc.metadata
        .components
        .extend(upstream_doc.metadata.components);
    doc.metadata.releases.extend(upstream_doc.metadata.releases);
    doc.metadata.entries.extend(upstream_doc.metadata.entries);
    doc.metadata
        .assertions
        .extend(upstream_doc.metadata.assertions);
    doc.metadata.forms.extend(upstream_doc.metadata.forms);
    doc.metadata.contexts.extend(upstream_doc.metadata.contexts);
    doc.metadata
        .relations
        .extend(upstream_doc.metadata.relations);
    doc.metadata.editorial_senses.push(EditorialSenseRecord {
        id: links_id("links-owned-sense-ba"),
        assertions: vec![links_id("links-owned-b"), links_id("links-owned-a")],
        editor: ContributorId::new("links-editor").unwrap(),
        revision: links_id("links-owned-r1"),
        rationale: TextValue {
            text: "ordered b then a".into(),
            language: Presence::Missing,
            markup: Presence::Missing,
        },
    });
    let lu = links_source("links-upstream");
    let cap = doc
        .captures
        .iter()
        .find(|row| row.identity.namespace == "links-owned-capture-r1")
        .unwrap()
        .identity
        .clone();
    let occurrence = OccurrenceRef {
        capture: cap,
        locator: "/0".into(),
    };
    doc.metadata.relations.extend([
        RelationRecord {
            id: links_id("links-relation-01"),
            origin: LocalRef::Assertion(links_id("links-upstream-x")),
            kind: RelationKind::SourceSeeAlso,
            direction: Direction::Forward,
            scope: RelationScope::Assertion,
            target: RelationTarget::Local(LocalRef::Entry(links_id("links-owned-entry-b"))),
            evidence: links_evidence(
                OccurrenceRef {
                    capture: doc
                        .captures
                        .iter()
                        .find(|row| row.identity.namespace == "links-upstream-capture-r1")
                        .unwrap()
                        .identity
                        .clone(),
                    locator: "/0".into(),
                },
                None,
            ),
            editorial_revision: None,
        },
        RelationRecord {
            id: links_id("links-relation-02"),
            origin: LocalRef::Form(links_id("links-upstream-form-z")),
            kind: RelationKind::SourceEtymology,
            direction: Direction::Reverse,
            scope: RelationScope::Form,
            target: RelationTarget::Local(LocalRef::Assertion(links_id("links-owned-b"))),
            evidence: links_evidence(
                OccurrenceRef {
                    capture: doc
                        .captures
                        .iter()
                        .find(|row| row.identity.namespace == "links-upstream-capture-r1")
                        .unwrap()
                        .identity
                        .clone(),
                    locator: "/2".into(),
                },
                None,
            ),
            editorial_revision: None,
        },
        RelationRecord {
            id: links_id("links-relation-05"),
            origin: LocalRef::Assertion(links_id("links-upstream-y")),
            kind: RelationKind::SourceSeeAlso,
            direction: Direction::Forward,
            scope: RelationScope::Assertion,
            target: RelationTarget::Unresolved("missing".into()),
            evidence: links_evidence(
                OccurrenceRef {
                    capture: doc
                        .captures
                        .iter()
                        .find(|row| row.identity.namespace == "links-upstream-capture-r1")
                        .unwrap()
                        .identity
                        .clone(),
                    locator: "/1".into(),
                },
                None,
            ),
            editorial_revision: None,
        },
        RelationRecord {
            id: links_id("links-relation-03"),
            origin: LocalRef::EditorialSense(links_id("links-owned-sense-ba")),
            kind: RelationKind::EditorialLink,
            direction: Direction::Bidirectional,
            scope: RelationScope::Editorial,
            target: RelationTarget::Local(LocalRef::Assertion(links_id("links-upstream-y"))),
            evidence: None,
            editorial_revision: Some(links_id("links-owned-r1")),
        },
        RelationRecord {
            id: links_id("links-relation-04"),
            origin: LocalRef::Entry(links_id("links-owned-entry-b")),
            kind: RelationKind::Other("fixture-unknown".into()),
            direction: Direction::Unspecified,
            scope: RelationScope::Entry,
            target: RelationTarget::Unknown,
            evidence: None,
            editorial_revision: None,
        },
        RelationRecord {
            id: links_id("links-relation-06"),
            origin: LocalRef::Assertion(links_id("links-upstream-y")),
            kind: RelationKind::EditorialLink,
            direction: Direction::Forward,
            scope: RelationScope::Assertion,
            target: RelationTarget::Local(LocalRef::Assertion(links_id("links-owned-c"))),
            evidence: None,
            editorial_revision: Some(links_id("links-owned-r1")),
        },
        RelationRecord {
            id: links_id("links-relation-07"),
            origin: LocalRef::Assertion(links_id("links-owned-a")),
            kind: RelationKind::SourceSeeAlso,
            direction: Direction::Forward,
            scope: RelationScope::Assertion,
            target: RelationTarget::Catalog(vlajmaji_lexicon::CatalogTarget {
                source: lu.clone(),
                release: links_id("links-upstream-release-r1"),
                native_entry_id: Some("N".into()),
                native_assertion_id: None,
                occurrence: None,
            }),
            evidence: links_evidence(occurrence.clone(), None),
            editorial_revision: None,
        },
        RelationRecord {
            id: links_id("links-relation-08"),
            origin: LocalRef::Assertion(links_id("links-owned-a")),
            kind: RelationKind::SourceSeeAlso,
            direction: Direction::Forward,
            scope: RelationScope::Assertion,
            target: RelationTarget::Catalog(vlajmaji_lexicon::CatalogTarget {
                source: lu,
                release: links_id("links-upstream-release-r1"),
                native_entry_id: Some("absent".into()),
                native_assertion_id: None,
                occurrence: None,
            }),
            evidence: links_evidence(occurrence, None),
            editorial_revision: None,
        },
    ]);
    doc.metadata.owned_heads.clear();
    Dictionary::from_document(doc).unwrap()
}

const LINK_DEPTH1_JSON: &[u8] = br#"{
  "version": 1,
  "query": {
    "operation": "links",
    "spelling": "seed",
    "depth": 1
  },
  "options": {
    "sources": [],
    "alias_scope": "ActiveOnly",
    "page_size": null,
    "cursor": null
  }
}
"#;
const LINK_DEPTH2_JSON: &[u8] = br#"{
  "version": 1,
  "query": {
    "operation": "links",
    "spelling": "seed",
    "depth": 2
  },
  "options": {
    "sources": [],
    "alias_scope": "ActiveOnly",
    "page_size": null,
    "cursor": null
  }
}
"#;
const LINK_FIXTURE_SPEC_JSON: &[u8] = br#"{
  "scope": "PRE-INPUT constructor/semantic authority, not product JSON or a claim these Dictionary constructors have run. Actual complete typed models must validate before dependent writes; unspecified decoration follows existing typed constructors and is preserved from emitted preinputs.",
  "sources": [
    {
      "id": "links-owned",
      "origin": "OwnedDraft",
      "abbreviation": "KO"
    },
    {
      "id": "links-upstream",
      "origin": "Upstream",
      "abbreviation": "KU"
    }
  ],
  "entries": [
    {
      "id": "links-owned-entry-a",
      "source": "links-owned",
      "spelling": "seed",
      "assertions": [
        "links-owned-a"
      ]
    },
    {
      "id": "links-owned-entry-b",
      "source": "links-owned",
      "spelling": "branch",
      "assertions": [
        "links-owned-b"
      ]
    },
    {
      "id": "links-owned-entry-c",
      "source": "links-owned",
      "spelling": "leaf",
      "assertions": [
        "links-owned-c"
      ]
    },
    {
      "id": "links-upstream-entry-x",
      "source": "links-upstream",
      "spelling": "seed",
      "assertions": [
        "links-upstream-x"
      ]
    },
    {
      "id": "links-upstream-entry-y",
      "source": "links-upstream",
      "spelling": "branch",
      "assertions": [
        "links-upstream-y"
      ]
    },
    {
      "id": "links-upstream-entry-z",
      "source": "links-upstream",
      "spelling": "leaf",
      "assertions": [
        "links-upstream-z"
      ]
    },
    {
      "id": "links-upstream-entry-q",
      "source": "links-upstream",
      "spelling": "branch",
      "assertions": []
    }
  ],
  "assertions": [
    {
      "id": "links-owned-a",
      "entry": "links-owned-entry-a",
      "eligibility": "Lexical",
      "definition": "Links fixture a",
      "locator": "/0"
    },
    {
      "id": "links-owned-b",
      "entry": "links-owned-entry-b",
      "eligibility": "Lexical",
      "definition": "Links fixture b",
      "locator": "/1"
    },
    {
      "id": "links-owned-c",
      "entry": "links-owned-entry-c",
      "eligibility": "Lexical",
      "definition": "Links fixture c",
      "locator": "/2"
    },
    {
      "id": "links-upstream-x",
      "entry": "links-upstream-entry-x",
      "eligibility": "Lexical",
      "definition": "Links fixture x",
      "locator": "/0"
    },
    {
      "id": "links-upstream-y",
      "entry": "links-upstream-entry-y",
      "eligibility": "Lexical",
      "definition": "Links fixture y",
      "locator": "/1"
    },
    {
      "id": "links-upstream-z",
      "entry": "links-upstream-entry-z",
      "eligibility": "Lexical",
      "definition": "Links fixture z",
      "locator": "/2"
    }
  ],
  "captures": [
    {
      "namespace": "links-owned-capture-r1",
      "source": "links-owned",
      "bytes_utf8": "seed | Links fixture a\nbranch | Links fixture b\nleaf | Links fixture c",
      "sha256": "de37040263a712f73ef64d3b8449f9b84b8c7d5680120bd46e1d57724a233110",
      "raw_locators": [
        "/0",
        "/1",
        "/2"
      ],
      "format_label": "synthetic-links-rows-v1",
      "raw_lines": [
        "seed | Links fixture a",
        "branch | Links fixture b",
        "leaf | Links fixture c"
      ],
      "raw_fields": "Text word, definition; upstream /0 additionally notes with exact Context text. Preserve field order; Context evidence field_index=2, other source evidence field_index=null."
    },
    {
      "namespace": "links-upstream-capture-r1",
      "source": "links-upstream",
      "bytes_utf8": "seed | Links fixture x | See also {branch} {branch} {missing}.\nbranch | Links fixture y\nleaf | Links fixture z",
      "sha256": "5f6e19ad8012d9f7e83d0c12a56094d35034543dc0b9b1be64fbc742618cb273",
      "raw_locators": [
        "/0",
        "/1",
        "/2"
      ],
      "format_label": "synthetic-links-rows-v1",
      "raw_lines": [
        "seed | Links fixture x | See also {branch} {branch} {missing}.",
        "branch | Links fixture y",
        "leaf | Links fixture z"
      ],
      "raw_fields": "Text word, definition; upstream /0 additionally notes with exact Context text. Preserve field order; Context evidence field_index=2, other source evidence field_index=null."
    }
  ],
  "native_attribution": {
    "LU-x": {
      "native_entry_id": "seed-native",
      "native_assertion_id": "x-native"
    },
    "LU-y": {
      "native_entry_id": "N",
      "native_assertion_id": "y-native"
    },
    "LU-z": {
      "native_entry_id": null,
      "native_assertion_id": "z-native"
    }
  },
  "form": {
    "id": "links-upstream-form-z",
    "entry": "links-upstream-entry-z",
    "kind": "SourceReportedForm",
    "spelling": "leaf-form",
    "evidence_assertion": "links-upstream-z",
    "computation": null
  },
  "sense": {
    "id": "links-owned-sense-ba",
    "revision": "links-owned-r1",
    "assertions": [
      "links-owned-b",
      "links-owned-a"
    ]
  },
  "context": {
    "id": "links-note-x",
    "assertion": "links-upstream-x",
    "kind": "Note",
    "text": "See also {branch} {branch} {missing}.",
    "evidence_assertion": "links-upstream-x",
    "tokens": [
      {
        "byte_start": 9,
        "byte_end": 17,
        "target_spelling": "branch",
        "rule_version": "see-also-braces-v2"
      },
      {
        "byte_start": 18,
        "byte_end": 26,
        "target_spelling": "branch",
        "rule_version": "see-also-braces-v2"
      },
      {
        "byte_start": 27,
        "byte_end": 36,
        "target_spelling": "missing",
        "rule_version": "see-also-braces-v2"
      }
    ]
  },
  "relations": [
    {
      "id": "links-relation-01",
      "origin": {
        "Assertion": "links-upstream-x"
      },
      "kind": "SourceSeeAlso",
      "direction": "Forward",
      "scope": "Assertion",
      "target": {
        "Local": {
          "Entry": "links-owned-entry-b"
        }
      },
      "publisher": "links-upstream",
      "evidence_assertion": "links-upstream-x",
      "editorial_revision": null
    },
    {
      "id": "links-relation-02",
      "origin": {
        "Form": "links-upstream-form-z"
      },
      "kind": "SourceEtymology",
      "direction": "Reverse",
      "scope": "Form",
      "target": {
        "Local": {
          "Assertion": "links-owned-b"
        }
      },
      "publisher": "links-upstream",
      "evidence_assertion": "links-upstream-z",
      "editorial_revision": null
    },
    {
      "id": "links-relation-03",
      "origin": {
        "EditorialSense": "links-owned-sense-ba"
      },
      "kind": "EditorialLink",
      "direction": "Bidirectional",
      "scope": "Editorial",
      "target": {
        "Local": {
          "Assertion": "links-upstream-y"
        }
      },
      "publisher": "links-owned",
      "evidence_assertion": null,
      "editorial_revision": "links-owned-r1"
    },
    {
      "id": "links-relation-04",
      "origin": {
        "Entry": "links-owned-entry-b"
      },
      "kind": {
        "Other": "fixture-unknown"
      },
      "direction": "Unspecified",
      "scope": "Entry",
      "target": "Unknown",
      "publisher": "links-owned",
      "evidence_assertion": null,
      "editorial_revision": null
    },
    {
      "id": "links-relation-05",
      "origin": {
        "Assertion": "links-upstream-y"
      },
      "kind": "SourceSeeAlso",
      "direction": "Forward",
      "scope": "Assertion",
      "target": {
        "Unresolved": "missing"
      },
      "publisher": "links-upstream",
      "evidence_assertion": "links-upstream-y",
      "editorial_revision": null
    },
    {
      "id": "links-relation-06",
      "origin": {
        "Assertion": "links-upstream-y"
      },
      "kind": "EditorialLink",
      "direction": "Forward",
      "scope": "Assertion",
      "target": {
        "Local": {
          "Assertion": "links-owned-c"
        }
      },
      "publisher": "links-owned",
      "evidence_assertion": null,
      "editorial_revision": "links-owned-r1"
    },
    {
      "id": "links-relation-07",
      "origin": {
        "Assertion": "links-owned-a"
      },
      "kind": "SourceSeeAlso",
      "direction": "Forward",
      "scope": "Assertion",
      "target": {
        "Catalog": {
          "source": "links-upstream",
          "release": "links-upstream-release-r1",
          "native_entry_id": "N",
          "native_assertion_id": null,
          "occurrence": null
        }
      },
      "publisher": "links-owned",
      "evidence_assertion": "links-owned-a",
      "editorial_revision": null
    },
    {
      "id": "links-relation-08",
      "origin": {
        "Assertion": "links-owned-a"
      },
      "kind": "SourceSeeAlso",
      "direction": "Forward",
      "scope": "Assertion",
      "target": {
        "Catalog": {
          "source": "links-upstream",
          "release": "links-upstream-release-r1",
          "native_entry_id": "absent",
          "native_assertion_id": null,
          "occurrence": null
        }
      },
      "publisher": "links-owned",
      "evidence_assertion": "links-owned-a",
      "editorial_revision": null
    }
  ],
  "owned_history": [
    {
      "revision": "links-owned-r0",
      "catalogue_revision": 0,
      "parent": null,
      "active": [],
      "tombstones": [],
      "release": "links-owned-release-r0",
      "release_assertions": []
    },
    {
      "revision": "links-owned-r1",
      "catalogue_revision": 1,
      "parent": "links-owned-r0",
      "active": [
        "links-owned-a",
        "links-owned-b",
        "links-owned-c"
      ],
      "tombstones": [],
      "release": "links-owned-release-r1",
      "release_assertions": [
        "links-owned-a",
        "links-owned-b",
        "links-owned-c"
      ]
    },
    {
      "revision": "links-owned-r2",
      "catalogue_revision": 2,
      "parent": "links-owned-r1",
      "active": [
        "links-owned-a",
        "links-owned-b"
      ],
      "tombstones": [
        "links-owned-c"
      ],
      "release": "links-owned-release-r2",
      "release_assertions": [
        "links-owned-a",
        "links-owned-b",
        "links-owned-c"
      ],
      "release_accounting": {
        "input_occurrences": 3,
        "normalized_occurrences": 3,
        "query_excluded_occurrences": 0,
        "unit": "occurrences"
      }
    }
  ],
  "mixed_pack": {
    "pack_id": "la-lojban",
    "payload_kind": "synthetic",
    "owned_heads": [],
    "owned_revision_prefix": [
      "links-owned-r0",
      "links-owned-r1"
    ],
    "owned_release_prefix": [
      "links-owned-release-r0",
      "links-owned-release-r1"
    ],
    "upstream_release": "links-upstream-release-r1",
    "upstream_release_assertions": [
      "links-upstream-x",
      "links-upstream-y",
      "links-upstream-z"
    ],
    "all_shared_LO_rows_equal_typed_bytes": true,
    "no_r2_control_row": true
  },
  "journal": {
    "registry_sources": [
      "links-owned"
    ],
    "real_genesis_then_append_r1_then_r2": true,
    "pack_LO_shared_rows_are_exact_immutable_prefix": true,
    "owns_sole_composed_head": true,
    "all_retained_capture_and_raw_rows_preserved": true,
    "r2_selected_available_release": true
  },
  "metadata_only_counterpart": {
    "scope": "VALID additional variant of SAME R08; La graph remains admitted, Llg registered only/unselected, no ninth relation/no malformed descriptor",
    "replace_R08_target": {
      "Catalog": {
        "source": "links-upstream",
        "release": "links-upstream-remote-release",
        "native_entry_id": "remote-native",
        "native_assertion_id": null,
        "occurrence": null
      }
    },
    "La_placeholder_release": {
      "id": "links-upstream-remote-release",
      "source": "links-upstream",
      "availability": "Missing",
      "selected": false,
      "captures": [],
      "assertions": []
    },
    "Llg_registered_only_model": {
      "sources": [
        "links-upstream"
      ],
      "SourceRecord_byte_equal": true,
      "release": "links-upstream-remote-release",
      "availability": "Missing",
      "selected": false,
      "assertions": [
        "links-upstream-remote-assertion"
      ],
      "entry": "links-upstream-remote-entry",
      "spelling": "remote",
      "native_entry_id": "remote-native",
      "loaded_capture": "links-upstream-remote-capture",
      "occurrence": "/0",
      "pack_id": "llg"
    },
    "expected": {
      "Full": "CatalogueOnly",
      "Compact": "CatalogueOnly",
      "operational_reason": "not_selected",
      "payload": null,
      "target_hops": [],
      "unchanged_step_counts": true
    }
  }
}
"#;
const LINK_EXPECTED_JSON: &[u8] = br#"{
  "scope": "PRE-INPUT manually enumerated graph/order/count authority; body expectations come from complete emitted input rows BEFORE query; Full/Compact target lower arrays retain own format order, not these traversal-ID unions.",
  "seed_assertion_ids": [
    "links-owned-a",
    "links-upstream-x"
  ],
  "clause_branch_entry_order": [
    "links-owned-entry-b",
    "links-upstream-entry-q",
    "links-upstream-entry-y"
  ],
  "r1_depth2": {
    "rows": [
      {
        "publisher": "links-owned",
        "depth": 1,
        "identity": {
          "evidence": {
            "kind": "typed_relation",
            "relation_id": "links-relation-03"
          },
          "orientation": "forward",
          "from_assertion_id": "links-owned-a"
        },
        "target_assertion_ids": [
          "links-upstream-y"
        ]
      },
      {
        "publisher": "links-owned",
        "depth": 1,
        "identity": {
          "evidence": {
            "kind": "typed_relation",
            "relation_id": "links-relation-07"
          },
          "orientation": "forward",
          "from_assertion_id": "links-owned-a"
        },
        "target_assertion_ids": [
          "links-upstream-y"
        ]
      },
      {
        "publisher": "links-owned",
        "depth": 1,
        "identity": {
          "evidence": {
            "kind": "typed_relation",
            "relation_id": "links-relation-08"
          },
          "orientation": "forward",
          "from_assertion_id": "links-owned-a"
        },
        "target_assertion_ids": []
      },
      {
        "publisher": "links-owned",
        "depth": 2,
        "identity": {
          "evidence": {
            "kind": "typed_relation",
            "relation_id": "links-relation-03"
          },
          "orientation": "forward",
          "from_assertion_id": "links-owned-b"
        },
        "target_assertion_ids": [
          "links-upstream-y"
        ]
      },
      {
        "publisher": "links-owned",
        "depth": 2,
        "identity": {
          "evidence": {
            "kind": "typed_relation",
            "relation_id": "links-relation-03"
          },
          "orientation": "reverse",
          "from_assertion_id": "links-upstream-y"
        },
        "target_assertion_ids": [
          "links-owned-b",
          "links-owned-a"
        ]
      },
      {
        "publisher": "links-owned",
        "depth": 2,
        "identity": {
          "evidence": {
            "kind": "typed_relation",
            "relation_id": "links-relation-04"
          },
          "orientation": "forward",
          "from_assertion_id": "links-owned-b"
        },
        "target_assertion_ids": []
      },
      {
        "publisher": "links-owned",
        "depth": 2,
        "identity": {
          "evidence": {
            "kind": "typed_relation",
            "relation_id": "links-relation-06"
          },
          "orientation": "forward",
          "from_assertion_id": "links-upstream-y"
        },
        "target_assertion_ids": [
          "links-owned-c"
        ]
      },
      {
        "publisher": "links-upstream",
        "depth": 1,
        "identity": {
          "evidence": {
            "kind": "typed_relation",
            "relation_id": "links-relation-01"
          },
          "orientation": "forward",
          "from_assertion_id": "links-upstream-x"
        },
        "target_assertion_ids": [
          "links-owned-b"
        ]
      },
      {
        "publisher": "links-upstream",
        "depth": 1,
        "identity": {
          "evidence": {
            "kind": "source_clause",
            "context_id": "links-note-x",
            "byte_start": 9,
            "byte_end": 17,
            "rule_version": "see-also-braces-v2",
            "target_spelling": "branch"
          },
          "orientation": "forward",
          "from_assertion_id": "links-upstream-x"
        },
        "target_assertion_ids": [
          "links-owned-b",
          "links-upstream-y"
        ]
      },
      {
        "publisher": "links-upstream",
        "depth": 1,
        "identity": {
          "evidence": {
            "kind": "source_clause",
            "context_id": "links-note-x",
            "byte_start": 18,
            "byte_end": 26,
            "rule_version": "see-also-braces-v2",
            "target_spelling": "branch"
          },
          "orientation": "forward",
          "from_assertion_id": "links-upstream-x"
        },
        "target_assertion_ids": [
          "links-owned-b",
          "links-upstream-y"
        ]
      },
      {
        "publisher": "links-upstream",
        "depth": 1,
        "identity": {
          "evidence": {
            "kind": "source_clause",
            "context_id": "links-note-x",
            "byte_start": 27,
            "byte_end": 36,
            "rule_version": "see-also-braces-v2",
            "target_spelling": "missing"
          },
          "orientation": "forward",
          "from_assertion_id": "links-upstream-x"
        },
        "target_assertion_ids": []
      },
      {
        "publisher": "links-upstream",
        "depth": 2,
        "identity": {
          "evidence": {
            "kind": "typed_relation",
            "relation_id": "links-relation-02"
          },
          "orientation": "reverse",
          "from_assertion_id": "links-owned-b"
        },
        "target_assertion_ids": [
          "links-upstream-z"
        ]
      },
      {
        "publisher": "links-upstream",
        "depth": 2,
        "identity": {
          "evidence": {
            "kind": "typed_relation",
            "relation_id": "links-relation-05"
          },
          "orientation": "forward",
          "from_assertion_id": "links-upstream-y"
        },
        "target_assertion_ids": []
      }
    ],
    "totals": {
      "unit": "traversal_steps",
      "records": 13,
      "source_groups": 2,
      "distinct_evidence": 11,
      "associated_assertions": 6,
      "visited_assertions": 6
    },
    "source_totals": [
      {
        "source": "links-owned",
        "records": 7,
        "distinct_evidence": 5,
        "associated_assertions": 4
      },
      {
        "source": "links-upstream",
        "records": 6,
        "distinct_evidence": 6,
        "associated_assertions": 4
      }
    ]
  },
  "r2_depth2": {
    "rows": [
      {
        "publisher": "links-owned",
        "depth": 1,
        "identity": {
          "evidence": {
            "kind": "typed_relation",
            "relation_id": "links-relation-03"
          },
          "orientation": "forward",
          "from_assertion_id": "links-owned-a"
        },
        "target_assertion_ids": [
          "links-upstream-y"
        ]
      },
      {
        "publisher": "links-owned",
        "depth": 1,
        "identity": {
          "evidence": {
            "kind": "typed_relation",
            "relation_id": "links-relation-07"
          },
          "orientation": "forward",
          "from_assertion_id": "links-owned-a"
        },
        "target_assertion_ids": [
          "links-upstream-y"
        ]
      },
      {
        "publisher": "links-owned",
        "depth": 1,
        "identity": {
          "evidence": {
            "kind": "typed_relation",
            "relation_id": "links-relation-08"
          },
          "orientation": "forward",
          "from_assertion_id": "links-owned-a"
        },
        "target_assertion_ids": []
      },
      {
        "publisher": "links-owned",
        "depth": 2,
        "identity": {
          "evidence": {
            "kind": "typed_relation",
            "relation_id": "links-relation-03"
          },
          "orientation": "forward",
          "from_assertion_id": "links-owned-b"
        },
        "target_assertion_ids": [
          "links-upstream-y"
        ]
      },
      {
        "publisher": "links-owned",
        "depth": 2,
        "identity": {
          "evidence": {
            "kind": "typed_relation",
            "relation_id": "links-relation-03"
          },
          "orientation": "reverse",
          "from_assertion_id": "links-upstream-y"
        },
        "target_assertion_ids": [
          "links-owned-b",
          "links-owned-a"
        ]
      },
      {
        "publisher": "links-owned",
        "depth": 2,
        "identity": {
          "evidence": {
            "kind": "typed_relation",
            "relation_id": "links-relation-04"
          },
          "orientation": "forward",
          "from_assertion_id": "links-owned-b"
        },
        "target_assertion_ids": []
      },
      {
        "publisher": "links-owned",
        "depth": 2,
        "identity": {
          "evidence": {
            "kind": "typed_relation",
            "relation_id": "links-relation-06"
          },
          "orientation": "forward",
          "from_assertion_id": "links-upstream-y"
        },
        "target_assertion_ids": []
      },
      {
        "publisher": "links-upstream",
        "depth": 1,
        "identity": {
          "evidence": {
            "kind": "typed_relation",
            "relation_id": "links-relation-01"
          },
          "orientation": "forward",
          "from_assertion_id": "links-upstream-x"
        },
        "target_assertion_ids": [
          "links-owned-b"
        ]
      },
      {
        "publisher": "links-upstream",
        "depth": 1,
        "identity": {
          "evidence": {
            "kind": "source_clause",
            "context_id": "links-note-x",
            "byte_start": 9,
            "byte_end": 17,
            "rule_version": "see-also-braces-v2",
            "target_spelling": "branch"
          },
          "orientation": "forward",
          "from_assertion_id": "links-upstream-x"
        },
        "target_assertion_ids": [
          "links-owned-b",
          "links-upstream-y"
        ]
      },
      {
        "publisher": "links-upstream",
        "depth": 1,
        "identity": {
          "evidence": {
            "kind": "source_clause",
            "context_id": "links-note-x",
            "byte_start": 18,
            "byte_end": 26,
            "rule_version": "see-also-braces-v2",
            "target_spelling": "branch"
          },
          "orientation": "forward",
          "from_assertion_id": "links-upstream-x"
        },
        "target_assertion_ids": [
          "links-owned-b",
          "links-upstream-y"
        ]
      },
      {
        "publisher": "links-upstream",
        "depth": 1,
        "identity": {
          "evidence": {
            "kind": "source_clause",
            "context_id": "links-note-x",
            "byte_start": 27,
            "byte_end": 36,
            "rule_version": "see-also-braces-v2",
            "target_spelling": "missing"
          },
          "orientation": "forward",
          "from_assertion_id": "links-upstream-x"
        },
        "target_assertion_ids": []
      },
      {
        "publisher": "links-upstream",
        "depth": 2,
        "identity": {
          "evidence": {
            "kind": "typed_relation",
            "relation_id": "links-relation-02"
          },
          "orientation": "reverse",
          "from_assertion_id": "links-owned-b"
        },
        "target_assertion_ids": [
          "links-upstream-z"
        ]
      },
      {
        "publisher": "links-upstream",
        "depth": 2,
        "identity": {
          "evidence": {
            "kind": "typed_relation",
            "relation_id": "links-relation-05"
          },
          "orientation": "forward",
          "from_assertion_id": "links-upstream-y"
        },
        "target_assertion_ids": []
      }
    ],
    "totals": {
      "unit": "traversal_steps",
      "records": 13,
      "source_groups": 2,
      "distinct_evidence": 11,
      "associated_assertions": 5,
      "visited_assertions": 5
    },
    "source_totals": [
      {
        "source": "links-owned",
        "records": 7,
        "distinct_evidence": 5,
        "associated_assertions": 3
      },
      {
        "source": "links-upstream",
        "records": 6,
        "distinct_evidence": 6,
        "associated_assertions": 4
      }
    ]
  },
  "depth1": {
    "row_indices_from_r1_depth2": [
      0,
      1,
      2,
      7,
      8,
      9,
      10
    ],
    "totals": {
      "unit": "traversal_steps",
      "records": 7,
      "source_groups": 2,
      "distinct_evidence": 7,
      "associated_assertions": 4,
      "visited_assertions": 4
    },
    "source_totals": [
      {
        "source": "links-owned",
        "records": 3,
        "distinct_evidence": 3,
        "associated_assertions": 2
      },
      {
        "source": "links-upstream",
        "records": 4,
        "distinct_evidence": 4,
        "associated_assertions": 3
      }
    ]
  },
  "only_KU_depth2": {
    "row_indices_from_r1_depth2": [
      7,
      8,
      9,
      10,
      12
    ],
    "targets_by_row": [
      [],
      [
        "links-upstream-y"
      ],
      [
        "links-upstream-y"
      ],
      [],
      []
    ],
    "totals": {
      "unit": "traversal_steps",
      "records": 5,
      "source_groups": 1,
      "distinct_evidence": 5,
      "associated_assertions": 2,
      "visited_assertions": 2
    },
    "branch_observations": [
      {
        "entry": "links-owned-entry-b",
        "reason": "requested_source"
      },
      {
        "entry": "links-upstream-entry-q",
        "reason": "no_current_eligible_assertion"
      },
      {
        "entry": "links-upstream-entry-y",
        "reason": null
      }
    ]
  },
  "only_KO_depth2": {
    "row_indices_from_r1_depth2": [
      0,
      1,
      2
    ],
    "targets_by_row": [
      [],
      [],
      []
    ],
    "totals": {
      "unit": "traversal_steps",
      "records": 3,
      "source_groups": 1,
      "distinct_evidence": 3,
      "associated_assertions": 1,
      "visited_assertions": 1
    }
  },
  "format_specific": {
    "R07": {
      "Full": {
        "status": "Resolved",
        "confirmed": [
          "links-upstream-y"
        ],
        "unverified": [
          "links-upstream-z"
        ],
        "hops": [
          "links-upstream-y"
        ]
      },
      "Compact": {
        "status": "Unverified",
        "confirmed": [
          "links-upstream-y"
        ],
        "unverified": [
          "links-upstream-z"
        ],
        "hops": [
          "links-upstream-y"
        ]
      }
    },
    "R08": {
      "Full": {
        "status": "Unverified",
        "payload": true,
        "confirmed": [],
        "unverified": [
          "links-upstream-z"
        ],
        "hops": []
      },
      "Compact": {
        "status": "NoMatch",
        "payload": false,
        "confirmed": [],
        "unverified": [],
        "hops": []
      }
    }
  },
  "pages_r1_size1_then3": {
    "first_indices": [
      0
    ],
    "second_indices": [
      1,
      2,
      3
    ],
    "remaining_indices": [
      4,
      5,
      6,
      7,
      8,
      9,
      10,
      11,
      12
    ],
    "all_totals_invariant": true,
    "terminal_next_cursor": null
  },
  "empty_start": {
    "records": 0,
    "source_groups": 0,
    "distinct_evidence": 0,
    "associated_assertions": 0,
    "visited_assertions": 0
  },
  "metadata_only_counterpart_R08": {
    "status": "CatalogueOnly",
    "operational_reason": "not_selected",
    "payload": null,
    "target_assertion_ids": [],
    "same_depth2_counts_as_r1": true
  }
}
"#;
const LINK_CURSOR_PROTOCOL_JSON: &[u8] = br#"{
  "scope": "PURE pre-input typed serde field-order oracle; artificial binding, not a live composition golden hash",
  "query_payload_utf8": "{\"query\":{\"operation\":\"links\",\"spelling\":\"seed\",\"depth\":2},\"selectors\":[\"KU\",\"KO\"],\"resolved_sources\":[\"links-owned\",\"links-upstream\"],\"alias_scope\":\"ActiveOnly\"}",
  "query_sha256": "fb8da7420bfbcdf9f095b22f5ed64662f988a0fc93c4c70e7f0572ceed2b2bd8",
  "reversed_selectors_query_payload_utf8": "{\"query\":{\"operation\":\"links\",\"spelling\":\"seed\",\"depth\":2},\"selectors\":[\"KO\",\"KU\"],\"resolved_sources\":[\"links-owned\",\"links-upstream\"],\"alias_scope\":\"ActiveOnly\"}",
  "reversed_selectors_query_sha256": "82157d8d1149057f1d403d7bd44cc7321d07ea29e5b4e64460fd1a12ba2f3e8e",
  "body_utf8": "{\"version\":1,\"domain\":\"workbench-lexical-cursor-v1\",\"format\":\"compact\",\"binding_sha256\":\"1111111111111111111111111111111111111111111111111111111111111111\",\"query_sha256\":\"fb8da7420bfbcdf9f095b22f5ed64662f988a0fc93c4c70e7f0572ceed2b2bd8\",\"last\":{\"operation\":\"links\",\"source\":\"links-owned\",\"depth\":1,\"evidence_kind_rank\":0,\"evidence\":{\"kind\":\"typed_relation\",\"relation_id\":\"links-relation-03\"},\"orientation_rank\":0,\"from_assertion_id\":\"links-owned-a\"}}",
  "checksum": "449721121f681e73942061578e74cecbd297ea120f89268660d691271341cccd",
  "envelope_utf8": "{\"body\":{\"version\":1,\"domain\":\"workbench-lexical-cursor-v1\",\"format\":\"compact\",\"binding_sha256\":\"1111111111111111111111111111111111111111111111111111111111111111\",\"query_sha256\":\"fb8da7420bfbcdf9f095b22f5ed64662f988a0fc93c4c70e7f0572ceed2b2bd8\",\"last\":{\"operation\":\"links\",\"source\":\"links-owned\",\"depth\":1,\"evidence_kind_rank\":0,\"evidence\":{\"kind\":\"typed_relation\",\"relation_id\":\"links-relation-03\"},\"orientation_rank\":0,\"from_assertion_id\":\"links-owned-a\"}},\"checksum\":\"449721121f681e73942061578e74cecbd297ea120f89268660d691271341cccd\"}",
  "token": "workbench-lexical-v1:7b22626f6479223a7b2276657273696f6e223a312c22646f6d61696e223a22776f726b62656e63682d6c65786963616c2d637572736f722d7631222c22666f726d6174223a22636f6d70616374222c2262696e64696e675f736861323536223a2231313131313131313131313131313131313131313131313131313131313131313131313131313131313131313131313131313131313131313131313131313131222c2271756572795f736861323536223a2266623864613734323062666263646639663039356232326635656436343636326639383861306663393363346337306537663035373263656564326232626438222c226c617374223a7b226f7065726174696f6e223a226c696e6b73222c22736f75726365223a226c696e6b732d6f776e6564222c226465707468223a312c2265766964656e63655f6b696e645f72616e6b223a302c2265766964656e6365223a7b226b696e64223a2274797065645f72656c6174696f6e222c2272656c6174696f6e5f6964223a226c696e6b732d72656c6174696f6e2d3033227d2c226f7269656e746174696f6e5f72616e6b223a302c2266726f6d5f617373657274696f6e5f6964223a226c696e6b732d6f776e65642d61227d7d2c22636865636b73756d223a2234343937323131323166363831653733393432303631353738653734636563626432393765613132306638393236383636306436393132373133343163636364227d"
}
"#;

fn write_input(path: &Path, bytes: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    fs::write(path, bytes)?;
    Ok(())
}

fn links_request(depth: u8) -> WorkbenchLexicalRequest {
    let bytes = match depth {
        1 => LINK_DEPTH1_JSON,
        2 => LINK_DEPTH2_JSON,
        _ => unreachable!("the fixed example only emits depth one and two requests"),
    };
    serde_json::from_slice(bytes).expect("fixed Links request is a valid typed input")
}

fn links_full_rows(response: &WorkbenchLinksFull) -> Vec<serde_json::Value> {
    response
        .groups
        .iter()
        .flat_map(|group| {
            group.hits.iter().map(move |step| {
                serde_json::json!({
                    "publisher": group.source.id.as_str(),
                    "depth": step.depth,
                    "identity": {
                        "evidence": step.identity.evidence,
                        "orientation": step.identity.orientation,
                        "from_assertion_id": step.from.assertion.id.as_str(),
                    },
                    "target_assertion_ids": step.target_assertion_ids,
                })
            })
        })
        .collect()
}

fn links_compact_rows(response: &WorkbenchLinksCompact) -> Vec<serde_json::Value> {
    response
        .groups
        .iter()
        .flat_map(|group| {
            group.hits.iter().map(move |step| {
                serde_json::json!({
                    "publisher": group.source.id.as_str(),
                    "depth": step.depth,
                    "identity": {
                        "evidence": step.identity.evidence,
                        "orientation": step.identity.orientation,
                        "from_assertion_id": step.from.assertion.id.as_str(),
                    },
                    "target_assertion_ids": step.target_assertion_ids,
                })
            })
        })
        .collect()
}

fn assert_links_full_rows_match_input(
    response: &WorkbenchLinksFull,
    input: &Dictionary,
) -> Result<(), Box<dyn std::error::Error>> {
    let document = input.document();
    for group in &response.groups {
        for step in &group.hits {
            let assertion = document
                .metadata
                .assertions
                .iter()
                .find(|row| row.id == step.from.assertion.id)
                .ok_or("Links Full returned an assertion absent from its typed input")?;
            let entry = document
                .metadata
                .entries
                .iter()
                .find(|row| row.id == step.from.entry.id)
                .ok_or("Links Full returned an Entry absent from its typed input")?;
            if &step.from.assertion != assertion || &step.from.entry != entry {
                return Err(
                    "Links Full from-side rows differ from their complete typed inputs".into(),
                );
            }
            if let Some(actual) = &step.record.relation {
                let expected = document
                    .metadata
                    .relations
                    .iter()
                    .find(|row| row.id == actual.id)
                    .ok_or("Links Full returned a Relation absent from its typed input")?;
                if actual != expected {
                    return Err("Links Full Relation differs from its complete typed input".into());
                }
            }
            if let Some(actual) = &step.record.context {
                let expected = document
                    .metadata
                    .contexts
                    .iter()
                    .find(|row| row.id == actual.id)
                    .ok_or("Links Full returned a Context absent from its typed input")?;
                if actual != expected {
                    return Err("Links Full Context differs from its complete typed input".into());
                }
            }
        }
    }
    Ok(())
}

fn run_links_journey(output_root: &Path) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let inputs = output_root.join("artifacts/links-inputs");
    fs::create_dir_all(&inputs)?;

    // Freeze every complete pre-query input, including the literal authorities,
    // before creating the journal or catalogue below.
    let owned_r0 = links_owned_r0();
    let owned_r1 = links_owned_r1(&owned_r0);
    let owned_r2 = links_owned_r2(&owned_r1);
    let mixed_pack = links_fixed_dictionary();
    let head_selection = WorkbenchSelection {
        version: 1,
        packs: vec![PackId::LaLojban],
        disabled: Vec::new(),
        owned: Some(WorkbenchOwnedRevision::Head),
        contexts: Vec::new(),
    };
    let historical_selection = WorkbenchSelection {
        owned: Some(WorkbenchOwnedRevision::Revision(links_id("links-owned-r1"))),
        ..head_selection.clone()
    };
    for (name, dictionary) in [
        ("owned-r0.json", &owned_r0),
        ("owned-r1.json", &owned_r1),
        ("owned-r2.json", &owned_r2),
        ("mixed-pack-r1.json", &mixed_pack),
    ] {
        write_input(
            &inputs.join(name),
            &serde_json::to_vec_pretty(&dictionary.document())?,
        )?;
    }
    write_input(
        &inputs.join("selection-head.json"),
        &serde_json::to_vec_pretty(&head_selection)?,
    )?;
    write_input(
        &inputs.join("selection-historical-r1.json"),
        &serde_json::to_vec_pretty(&historical_selection)?,
    )?;
    for (name, bytes) in [
        ("links-depth1.json", LINK_DEPTH1_JSON),
        ("links-depth2.json", LINK_DEPTH2_JSON),
        ("links-fixture-spec.json", LINK_FIXTURE_SPEC_JSON),
        ("links-expected.json", LINK_EXPECTED_JSON),
        ("links-cursor-protocol.json", LINK_CURSOR_PROTOCOL_JSON),
    ] {
        write_input(&inputs.join(name), bytes)?;
    }

    let workspace = output_root.join("links-workbench");
    fs::create_dir_all(&workspace)?;
    let owned_root = workspace.join("owned");
    let (mut writer, created) = OwnedJournal::create(&owned_root, &owned_r0)?;
    let r1_publication = writer.commit(&links_id("links-owned-r0"), &owned_r1)?;
    if created.head.sequence != 0 || r1_publication.head.sequence != 1 {
        return Err("Links journal did not retain the fixed r0→r1 sequence".into());
    }

    let sealed = seal_release(PackId::LaLojban, PackPayloadKind::Synthetic, &mixed_pack)?;
    let catalogue_root = workspace.join("catalogue");
    let catalogue_request = CollectionRequest::new(
        vec![CollectionInput::SealedBytes {
            pack_id: PackId::LaLojban,
            release: Cow::Borrowed(&sealed.release),
            seal: Cow::Borrowed(&sealed.seal),
        }],
        CollectionSelection::Packs(vec![PackId::LaLojban]),
        true,
    )?;
    CollectionCatalogue::install(&catalogue_root, catalogue_request)?;

    let profile_config = workspace.join("links-workbench-profiles.json");
    let configured = WorkbenchProfiles::configure(
        &profile_config,
        WorkbenchProfileConfigureRequest {
            name: "links".into(),
            catalogue_root: Some(catalogue_root.clone()),
            owned_root: Some(owned_root.clone()),
            selection: head_selection.clone(),
            make_default: false,
        },
        None,
    )?;
    let configured_r1 = WorkbenchProfiles::refresh(
        &profile_config,
        Some("links"),
        workspace.join("view-configured-r1"),
        configured.revision,
    )?;
    let configured_default = WorkbenchProfiles::set_default(
        &profile_config,
        Some("links"),
        configured_r1.publication.revision,
    )?;

    let historical_root = workspace.join("view-historical-r1");
    let r1_view = WorkbenchView::build_new(
        &historical_root,
        WorkbenchBuildRequest {
            catalogue_root: Some(catalogue_root.clone()),
            owned_root: Some(owned_root.clone()),
            selection: historical_selection,
        },
    )?;
    let mut historical = WorkbenchSearchEngine::open(&historical_root)?;
    let depth1_request = links_request(1);
    let full_r1_depth1 = historical.execute_lexical(&depth1_request)?;
    let compact_r1_depth1 = historical.execute_lexical_compact(&depth1_request)?;
    let WorkbenchLexicalFullResult::Links(full_depth1) = &full_r1_depth1.result else {
        return Err("fixed depth-one request did not produce Full Links".into());
    };
    let WorkbenchLexicalCompactResult::Links(compact_depth1) = &compact_r1_depth1.result else {
        return Err("fixed depth-one request did not produce Compact Links".into());
    };
    if full_depth1.totals.records != 7 || compact_depth1.totals.records != 7 {
        return Err("the fixed depth-one Links request must retain seven rows".into());
    }
    let request = links_request(2);
    let expected: serde_json::Value = serde_json::from_slice(LINK_EXPECTED_JSON)?;
    let expected_r1_rows = expected["r1_depth2"]["rows"]
        .as_array()
        .ok_or("fixed expected rows are missing")?;
    let full_r1 = historical.execute_lexical(&request)?;
    let compact_r1 = historical.execute_lexical_compact(&request)?;
    let WorkbenchLexicalFullResult::Links(full_links) = &full_r1.result else {
        return Err("fixed Links request did not produce a Full Links result".into());
    };
    let WorkbenchLexicalCompactResult::Links(compact_links) = &compact_r1.result else {
        return Err("fixed Links request did not produce a Compact Links result".into());
    };
    if links_full_rows(full_links) != *expected_r1_rows
        || links_compact_rows(compact_links) != *expected_r1_rows
        || serde_json::to_value(&full_links.totals)? != expected["r1_depth2"]["totals"]
        || serde_json::to_value(&compact_links.totals)? != expected["r1_depth2"]["totals"]
    {
        return Err("r1 Full/Compact rows or totals differ from the fixed pre-input oracle".into());
    }
    assert_links_full_rows_match_input(full_links, &mixed_pack)?;
    let detail_ref = compact_links
        .groups
        .iter()
        .flat_map(|group| &group.hits)
        .find(|step| {
            matches!(&step.identity.evidence, vlajmaji_search::WorkbenchLinkEvidenceId::TypedRelation { relation_id } if relation_id.as_str() == "links-relation-01")
        })
        .ok_or("fixed Links query did not retain its source-evidence detail reference")?
        .detail_ref
        .clone();
    let detail = historical.link_detail(&detail_ref, &compact_r1.binding_sha256)?;
    let mut configured_r1_engine =
        ConfiguredWorkbenchSearchEngine::open(&profile_config, Some("links"))?;
    let configured_full_r1 = configured_r1_engine.execute_lexical(&request)?;
    let configured_compact_r1 = configured_r1_engine.execute_lexical_compact(&request)?;
    if serde_json::to_vec(&configured_full_r1.result)? != serde_json::to_vec(&full_r1.result)? {
        return Err("configured r1 Links body differs from its direct historical result".into());
    }
    drop(configured_r1_engine);

    // Append r2 only after the historical engine has exercised the r1 view.
    let r2_publication = writer.commit(&links_id("links-owned-r1"), &owned_r2)?;
    if r2_publication.head.sequence != 2 {
        return Err("Links journal did not retain the fixed r2 sequence".into());
    }
    let current_root = workspace.join("view-head-r2");
    let current_view = WorkbenchView::build_new(
        &current_root,
        WorkbenchBuildRequest {
            catalogue_root: Some(catalogue_root),
            owned_root: Some(owned_root),
            selection: head_selection,
        },
    )?;
    let mut current = WorkbenchSearchEngine::open(&current_root)?;
    let full_r2 = current.execute_lexical(&request)?;
    let compact_r2 = current.execute_lexical_compact(&request)?;
    let WorkbenchLexicalFullResult::Links(full_links_r2) = &full_r2.result else {
        return Err("current r2 request did not produce Full Links".into());
    };
    let WorkbenchLexicalCompactResult::Links(compact_links_r2) = &compact_r2.result else {
        return Err("current r2 request did not produce Compact Links".into());
    };
    let expected_r2_rows = expected["r2_depth2"]["rows"]
        .as_array()
        .ok_or("fixed r2 expected rows are missing")?
        .clone();
    if links_full_rows(full_links_r2) != expected_r2_rows
        || links_compact_rows(compact_links_r2) != expected_r2_rows
        || serde_json::to_value(&full_links_r2.totals)? != expected["r2_depth2"]["totals"]
        || serde_json::to_value(&compact_links_r2.totals)? != expected["r2_depth2"]["totals"]
    {
        return Err(
            "current r2 Full/Compact rows or totals differ from the fixed pre-input oracle".into(),
        );
    }
    assert_links_full_rows_match_input(full_links_r2, &mixed_pack)?;
    let configured_r2 = WorkbenchProfiles::refresh(
        &profile_config,
        None,
        workspace.join("view-configured-r2"),
        configured_default.revision,
    )?;
    let mut named_r2_engine =
        ConfiguredWorkbenchSearchEngine::open(&profile_config, Some("links"))?;
    let named_full_r2 = named_r2_engine.execute_lexical(&request)?;
    let named_compact_r2 = named_r2_engine.execute_lexical_compact(&request)?;
    let WorkbenchLexicalCompactResult::Links(named_compact_links) = &named_compact_r2.result else {
        return Err("configured Compact r2 did not return Links".into());
    };
    let named_detail_ref = named_compact_links
        .groups
        .iter()
        .flat_map(|group| &group.hits)
        .find(|step| {
            matches!(&step.identity.evidence, vlajmaji_search::WorkbenchLinkEvidenceId::TypedRelation { relation_id } if relation_id.as_str() == "links-relation-01")
        })
        .ok_or("configured r2 omitted the expected detail reference")?
        .detail_ref
        .clone();
    let named_detail =
        named_r2_engine.link_detail(&named_detail_ref, &named_compact_r2.binding_sha256)?;
    drop(named_r2_engine);
    let mut default_r2_engine = ConfiguredWorkbenchSearchEngine::open(&profile_config, None)?;
    let default_full_r2 = default_r2_engine.execute_lexical(&request)?;
    if serde_json::to_vec(&default_full_r2.result)? != serde_json::to_vec(&named_full_r2.result)?
        || serde_json::to_vec(&named_full_r2.result)? != serde_json::to_vec(&full_r2.result)?
    {
        return Err(
            "named/default configured r2 Links differ from the direct current result".into(),
        );
    }
    let profile_history = WorkbenchProfiles::read(&profile_config)?.history(None)?;
    if configured.revision != 1
        || configured_r1.publication.revision != 2
        || configured_default.revision != 3
        || configured_r2.publication.revision != 4
        || configured_r2
            .publication
            .active
            .as_ref()
            .map(|row| row.sequence)
            != Some(1)
        || profile_history
            .prepared
            .iter()
            .map(|row| row.sequence)
            .collect::<Vec<_>>()
            != [0, 1]
    {
        return Err(
            "Links profile publication history differs from the fixed r1/r2 sequence".into(),
        );
    }

    Ok(serde_json::json!({
        "inputs": inputs,
        "workspace": workspace,
        "historical_r1_binding": r1_view.binding,
        "current_r2_binding": current_view.binding,
        "depth1_request": depth1_request,
        "historical_full_r1_depth1": full_r1_depth1,
        "historical_compact_r1_depth1": compact_r1_depth1,
        "r1_publication": r1_publication,
        "r2_publication": r2_publication,
        "request": request,
        "historical_full_r1": full_r1,
        "historical_compact_r1": compact_r1,
        "historical_link_detail": detail,
        "configured_r1_full": configured_full_r1,
        "configured_r1_compact": configured_compact_r1,
        "current_full_r2": full_r2,
        "current_compact_r2": compact_r2,
        "configured_r2_publication": configured_r2,
        "configured_r2_full": named_full_r2,
        "configured_r2_compact": named_compact_r2,
        "configured_r2_detail": named_detail,
        "default_r2_full": default_full_r2,
        "profile_history": profile_history,
    }))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let output_root = args
        .next()
        .map(PathBuf::from)
        .ok_or("usage: integrated_workbench OUTPUT_ROOT")?;
    if args.next().is_some() {
        return Err("expected exactly one output root".into());
    }
    fs::create_dir(&output_root)?;
    let inputs = output_root.join("inputs");
    fs::create_dir(&inputs)?;

    let upstream_sha256 = format!("{:x}", Sha256::digest(UPSTREAM_JSON));
    let upstream = import_la_lojban_export(
        UPSTREAM_JSON,
        &LaLojbanImportOptions {
            revision: "0000000000000000000000000000000000000000".into(),
            expected_raw_sha256: upstream_sha256.clone(),
            raw_locator: Some("https://example.invalid/tiny/farvi.json".into()),
            notice_locator: Some("https://example.invalid/tiny/README.md".into()),
        },
    )?
    .into_dictionary();
    let seed = genesis()?;
    let revision_one = edit(&seed)?;
    let revision_two = next_edit(&revision_one)?;
    let owned_entry_target =
        RelationTarget::Local(LocalRef::Entry(EntityId::new("owned-entry-farvi")?));
    let owned_assertion_target =
        RelationTarget::Local(LocalRef::Assertion(EntityId::new("owned-assertion-a")?));
    let unknown_target = RelationTarget::Unknown;
    let unresolved_target = RelationTarget::Unresolved("future editorial target".into());
    let upstream_release = upstream
        .metadata()
        .releases
        .first()
        .ok_or("upstream input has no release")?;
    let upstream_assertion = upstream
        .metadata()
        .assertions
        .first()
        .ok_or("upstream input has no assertion")?;
    let upstream_registry = upstream.registry().document();
    let upstream_source = upstream_registry
        .sources
        .iter()
        .find(|row| row.id.as_str() == "lensisku")
        .ok_or("upstream input has no Lensisku source")?;
    let upstream_native = upstream_registry
        .occurrences
        .iter()
        .find(|row| row.occurrence == upstream_assertion.occurrence.to_record_identity().unwrap())
        .ok_or("upstream assertion has no retained native attribution")?;
    let upstream_target = RelationTarget::Catalog(vlajmaji_lexicon::CatalogTarget {
        source: upstream_source.id.clone(),
        release: upstream_release.id.clone(),
        native_entry_id: None,
        native_assertion_id: upstream_native.native_assertion_id.clone(),
        occurrence: Some(upstream_assertion.occurrence.clone()),
    });
    let disabled_selection: WorkbenchSelection =
        serde_json::from_slice(REFERENCE_DISABLED_SELECTION_JSON)?;
    let upstream_target_bytes = serde_json::to_vec(&upstream_target)?;
    if upstream_target_bytes
        != br#"{"Catalog":{"source":"lensisku","release":"la-lojban-release-116b8d1761763fa4341642a0f2ede5b45e508ac43b6ad79688bc5694e92ed438","native_entry_id":null,"native_assertion_id":"1","occurrence":{"capture":{"namespace":"captured:lensisku:la-lojban:en","sha256":"06220fcd82ff067293787c5057b3a68d6c63400d80e29d02fe5a4a64be015db5"},"locator":"/0"}}}"#
    {
        return Err("upstream reference target differs from its fixed typed input".into());
    }
    if serde_json::to_vec(&owned_entry_target)? != REFERENCE_OWNED_ENTRY_JSON
        || serde_json::to_vec(&owned_assertion_target)? != REFERENCE_OWNED_ASSERTION_JSON
        || serde_json::to_vec(&unknown_target)? != REFERENCE_UNKNOWN_JSON
        || serde_json::to_vec(&unresolved_target)? != REFERENCE_UNRESOLVED_JSON
    {
        return Err("owned and status reference targets differ from fixed typed inputs".into());
    }
    let selection = WorkbenchSelection {
        version: 1,
        packs: vec![vlajmaji_lexicon::PackId::LaLojban],
        disabled: Vec::new(),
        owned: Some(vlajmaji_store::WorkbenchOwnedRevision::Head),
        contexts: Vec::new(),
    };
    let mut historical_selection = selection.clone();
    historical_selection.owned = Some(WorkbenchOwnedRevision::Revision(EntityId::new("owned-r1")?));
    let request = DictionaryRequest::Exact {
        spelling: "farvi".into(),
        options: DictionaryExactOptions::default(),
    };
    let expected = serde_json::json!({
        "spelling": "farvi",
        "source_order": ["lensisku", "owned-example"],
        "definitions_by_source_r1": {
            "lensisku": ["advance"],
            "owned-example": ["grow", "develop"]
        },
        "definitions_by_source_r2": {
            "lensisku": ["advance"],
            "owned-example": ["grow", "develop", "flourish"]
        },
        "owned_head_r1": "owned-r1",
        "owned_head_r2": "owned-r2",
        "historical_owned_revision": "owned-r1",
        "upstream_raw_sha256": upstream_sha256,
        "selection": selection.clone(),
        "historical_selection": historical_selection.clone(),
        "request": request.clone(),
    });
    let sealed = seal_release(
        vlajmaji_lexicon::PackId::LaLojban,
        vlajmaji_lexicon::PackPayloadKind::Synthetic,
        &upstream,
    )?;
    // All typed request/selection/source inputs are durable before the first
    // catalogue, journal, or workbench database is created.
    write_new(&inputs.join("upstream.json"), UPSTREAM_JSON)?;
    write_new(
        &inputs.join("upstream-dictionary.json"),
        &upstream.to_json_bytes()?,
    )?;
    write_new(&inputs.join("upstream-release.json"), &sealed.release)?;
    write_new(&inputs.join("upstream-seal.json"), &sealed.seal)?;
    write_new(&inputs.join("owned-seed.json"), &seed.to_json_bytes()?)?;
    write_new(
        &inputs.join("owned-r1.json"),
        &revision_one.to_json_bytes()?,
    )?;
    write_new(
        &inputs.join("owned-r2.json"),
        &revision_two.to_json_bytes()?,
    )?;
    write_new(
        &inputs.join("selection.json"),
        &serde_json::to_vec_pretty(&selection)?,
    )?;
    let profile_alpha_selection = serde_json::to_vec_pretty(&selection)?;
    write_new(
        &inputs.join("profile-alpha-selection.json"),
        &profile_alpha_selection,
    )?;
    write_new(
        &inputs.join("profile-beta-selection.json"),
        PROFILE_BETA_SELECTION_JSON,
    )?;
    write_new(
        &inputs.join("profiles-expected.json"),
        PROFILES_EXPECTED_JSON,
    )?;
    write_new(
        &inputs.join("historical-selection.json"),
        &serde_json::to_vec_pretty(&historical_selection)?,
    )?;
    write_new(
        &inputs.join("request.json"),
        &serde_json::to_vec_pretty(&request)?,
    )?;
    write_new(
        &inputs.join("expected.json"),
        &serde_json::to_vec_pretty(&expected)?,
    )?;

    let reference_input_root = output_root.join("artifacts/reference-inputs");
    fs::create_dir_all(&reference_input_root)?;
    write_new(
        &reference_input_root.join("reference-owned-entry.json"),
        REFERENCE_OWNED_ENTRY_JSON,
    )?;
    write_new(
        &reference_input_root.join("reference-owned-assertion.json"),
        REFERENCE_OWNED_ASSERTION_JSON,
    )?;
    write_new(
        &reference_input_root.join("reference-upstream.json"),
        &upstream_target_bytes,
    )?;
    write_new(
        &reference_input_root.join("reference-unknown.json"),
        REFERENCE_UNKNOWN_JSON,
    )?;
    write_new(
        &reference_input_root.join("reference-unresolved.json"),
        REFERENCE_UNRESOLVED_JSON,
    )?;
    write_new(
        &reference_input_root.join("reference-disabled-selection.json"),
        REFERENCE_DISABLED_SELECTION_JSON,
    )?;
    write_new(
        &reference_input_root.join("reference-expected.json"),
        REFERENCE_EXPECTED_JSON,
    )?;

    let catalogue_root = output_root.join("catalogue");
    let owned_root = output_root.join("owned");
    let workbench_r1_root = output_root.join("workbench-r1");
    let workbench_r2_root = output_root.join("workbench-r2");
    let install = CollectionRequest::new(
        vec![CollectionInput::SealedBytes {
            pack_id: vlajmaji_lexicon::PackId::LaLojban,
            release: Cow::Borrowed(&sealed.release),
            seal: Cow::Borrowed(&sealed.seal),
        }],
        CollectionSelection::Packs(vec![vlajmaji_lexicon::PackId::LaLojban]),
        true,
    )?;
    CollectionCatalogue::install(&catalogue_root, install)?;
    let (mut writer, created) = OwnedJournal::create(&owned_root, &seed)?;
    let revision_one_result = writer.commit(&created.head.revision.id, &revision_one)?;
    drop(writer);

    let built_r1 = vlajmaji_store::WorkbenchView::build_new(
        &workbench_r1_root,
        WorkbenchBuildRequest {
            catalogue_root: Some(catalogue_root),
            owned_root: Some(owned_root.clone()),
            selection: selection.clone(),
        },
    )?;
    let mut search_r1 = WorkbenchSearchEngine::open(&workbench_r1_root)?;
    let full_r1 = search_r1.execute(request.clone())?;
    let WorkbenchFullResult::Exact(actual_r1) = &full_r1.result else {
        return Err("the emitted exact request did not produce an exact response".into());
    };
    if actual_r1.groups.len() != 2
        || actual_r1.groups[0].source.id.as_str() != "lensisku"
        || actual_r1.groups[1].source.id.as_str() != "owned-example"
        || actual_r1.groups[0].hits.len() != 1
        || actual_r1.groups[1].hits.len() != 2
        || actual_r1.totals.occurrences != 3
    {
        return Err("the r1 mixed farvi query must retain all three competing definitions".into());
    }
    let upstream_entry = upstream
        .metadata()
        .entries
        .iter()
        .find(|entry| {
            entry.source.as_str() == "lensisku" && entry.spelling == Presence::Value("farvi".into())
        })
        .ok_or("the normalized upstream input lacks the expected farvi entry")?;
    let upstream_assertion = upstream
        .metadata()
        .assertions
        .iter()
        .find(|assertion| assertion.entry == upstream_entry.id)
        .ok_or("the normalized upstream input lacks the expected farvi assertion")?;
    let upstream_capture = upstream
        .capture(&upstream_assertion.occurrence.capture)
        .ok_or("the normalized upstream input lacks the farvi capture")?;
    let upstream_raw = upstream
        .occurrence(&upstream_assertion.occurrence)
        .ok_or("the normalized upstream input lacks the farvi raw row")?;
    let upstream_registry = upstream.registry().document();
    let upstream_snapshot_identity = upstream_assertion
        .occurrence
        .capture
        .to_snapshot_identity()?;
    let expected_snapshot_attribution = upstream_registry
        .snapshots
        .iter()
        .find(|row| row.snapshot == upstream_snapshot_identity)
        .ok_or("the normalized upstream input lacks snapshot attribution")?;
    let upstream_record_identity = upstream_assertion.occurrence.to_record_identity()?;
    let expected_occurrence_attribution = upstream_registry
        .occurrences
        .iter()
        .find(|row| row.occurrence == upstream_record_identity)
        .ok_or("the normalized upstream input lacks occurrence attribution")?;
    let upstream_hit = &actual_r1.groups[0].hits[0];
    if &upstream_hit.entry != upstream_entry
        || &upstream_hit.assertion != upstream_assertion
        || upstream_hit.capture != upstream_capture.metadata
        || &upstream_hit.raw != upstream_raw
        || &upstream_hit.snapshot_attribution != expected_snapshot_attribution
        || upstream_hit.occurrence_attribution.as_ref() != Some(expected_occurrence_attribution)
    {
        return Err("the upstream Full hit differs from the emitted normalized input".into());
    }
    let owned_entry = revision_one
        .metadata()
        .entries
        .iter()
        .find(|entry| entry.id.as_str() == "owned-entry-farvi")
        .ok_or("the owned r1 input lacks the expected farvi entry")?;
    if actual_r1.groups[1]
        .hits
        .iter()
        .map(|hit| hit.entry.id.as_str())
        .any(|id| id != owned_entry.id.as_str())
        || actual_r1.groups[1]
            .hits
            .iter()
            .map(|hit| match &hit.assertion.definition {
                Presence::Value(text) => text.text.as_str(),
                Presence::Missing | Presence::Null => "<absent>",
            })
            .collect::<Vec<_>>()
            != ["grow", "develop"]
    {
        return Err("the owned Full hits differ from the emitted r1 input or order".into());
    }
    let expected_owned_r1_assertions = revision_one
        .metadata()
        .assertions
        .iter()
        .filter(|assertion| assertion.entry == owned_entry.id)
        .collect::<Vec<_>>();
    if actual_r1.groups[1].hits.len() != expected_owned_r1_assertions.len()
        || actual_r1.groups[1]
            .hits
            .iter()
            .zip(&expected_owned_r1_assertions)
            .any(|(hit, assertion)| hit.entry != *owned_entry || hit.assertion != **assertion)
    {
        return Err("the owned Full hits must match every emitted r1 Entry and Assertion".into());
    }

    let direct_upstream_reference = search_r1.resolve_reference(&upstream_target)?;
    let upstream_reference_payload = direct_upstream_reference
        .result
        .payload
        .as_ref()
        .ok_or("direct upstream reference must include its admitted payload")?;
    let upstream_reference_hit = upstream_reference_payload
        .confirmed
        .first()
        .ok_or("direct upstream reference must confirm its typed assertion")?;
    if direct_upstream_reference.result.status != ReferenceStatus::Resolved
        || upstream_reference_payload.confirmed.len() != 1
        || upstream_reference_hit.entry != *upstream_entry
        || upstream_reference_hit.assertion != *upstream_assertion
        || upstream_reference_hit.raw.as_ref() != Some(upstream_raw)
        || upstream_reference_hit.occurrence_attribution.as_ref()
            != Some(expected_occurrence_attribution)
        || upstream_reference_hit.release_ids != [upstream_release.id.clone()]
        || !upstream_reference_hit.owned_revision_ids.is_empty()
    {
        return Err("the direct upstream reference differs from its typed input".into());
    }
    let compact_r1_root = output_root.join("workbench-r1");
    let mut direct_compact_engine = WorkbenchSearchEngine::open(&compact_r1_root)?;
    let direct_compact_upstream =
        direct_compact_engine.resolve_reference_compact(&upstream_target)?;
    if direct_compact_upstream.result.status != ReferenceStatus::Resolved
        || direct_compact_upstream
            .result
            .payload
            .as_ref()
            .is_none_or(|payload| payload.confirmed.len() != 1)
    {
        return Err("the direct Compact upstream reference must retain its typed payload".into());
    }
    let direct_reference_detail_ref = direct_compact_upstream
        .result
        .payload
        .as_ref()
        .and_then(|payload| payload.confirmed.first())
        .and_then(|row| row.detail_ref.as_ref())
        .ok_or("the direct Compact reference must issue its existing detail key")?;
    let direct_reference_detail = direct_compact_engine.detail(
        direct_reference_detail_ref,
        &direct_compact_upstream.binding_sha256,
    )?;
    if direct_reference_detail.binding_sha256 != direct_compact_upstream.binding_sha256
        || direct_reference_detail.result.hit.is_none()
    {
        return Err("the direct Compact reference detail must keep its outer binding".into());
    }
    let expected_owned_entry = revision_one
        .metadata()
        .entries
        .iter()
        .find(|entry| entry.id.as_str() == "owned-entry-farvi")
        .ok_or("the owned r1 input lacks the fixed reference entry")?;
    let expected_owned_assertion = revision_one
        .metadata()
        .assertions
        .iter()
        .find(|assertion| assertion.id.as_str() == "owned-assertion-a")
        .ok_or("the owned r1 input lacks the fixed reference assertion")?;
    let direct_owned_entry = search_r1.resolve_reference(&owned_entry_target)?;
    if direct_owned_entry.result.status != ReferenceStatus::Resolved
        || !matches!(
            direct_owned_entry.result.payload.as_deref().and_then(|payload| payload.entity.as_ref()),
            Some(ReferenceEntity::Entry(entry)) if entry == expected_owned_entry
        )
        || !direct_owned_entry.result.registrations.is_empty()
    {
        return Err("the direct owned Entry reference differs from its r1 input".into());
    }
    let direct_owned_assertion = search_r1.resolve_reference(&owned_assertion_target)?;
    if direct_owned_assertion.result.status != ReferenceStatus::Resolved
        || !matches!(
            direct_owned_assertion.result.payload.as_deref().and_then(|payload| payload.entity.as_ref()),
            Some(ReferenceEntity::Assertion(assertion)) if assertion == expected_owned_assertion
        )
        || direct_owned_assertion
            .result
            .payload
            .as_ref()
            .and_then(|payload| payload.confirmed.first())
            .is_none_or(|row| row.owned_revision_ids != [EntityId::new("owned-r1").unwrap()])
    {
        return Err("the direct owned Assertion reference differs from its r1 input".into());
    }
    let direct_unknown = search_r1.resolve_reference(&unknown_target)?;
    let direct_unresolved = search_r1.resolve_reference(&unresolved_target)?;
    if direct_unknown.result.status != ReferenceStatus::Unknown
        || direct_unknown.result.payload.is_some()
        || direct_unresolved.result.status != ReferenceStatus::Unresolved
        || direct_unresolved.result.payload.is_some()
    {
        return Err(
            "Unknown and nonempty Unresolved targets must preserve their typed status".into(),
        );
    }

    let disabled_workbench_root = output_root.join("workbench-reference-disabled");
    vlajmaji_store::WorkbenchView::build_new(
        &disabled_workbench_root,
        WorkbenchBuildRequest {
            catalogue_root: Some(output_root.join("catalogue")),
            owned_root: Some(owned_root.clone()),
            selection: disabled_selection,
        },
    )?;
    let mut disabled_engine = WorkbenchSearchEngine::open(&disabled_workbench_root)?;
    let disabled_upstream = disabled_engine.resolve_reference(&upstream_target)?;
    let disabled_owned = disabled_engine.resolve_reference(&owned_entry_target)?;
    if disabled_upstream.result.status != ReferenceStatus::CatalogueOnly
        || disabled_upstream.result.operational_reason.as_deref() != Some("disabled")
        || disabled_upstream.result.payload.is_some()
        || disabled_owned.result.status != ReferenceStatus::Resolved
        || !matches!(
            disabled_owned.result.payload.as_deref().and_then(|payload| payload.entity.as_ref()),
            Some(ReferenceEntity::Entry(entry)) if entry == expected_owned_entry
        )
    {
        return Err("the disabled catalogue and active owned reference policies differ".into());
    }

    // Keep the configured-profile journal independent from the original S01
    // journal so its publication sequence is an input to this example alone.
    let configured_owned_root = output_root.join("configured-owned");
    let (mut configured_writer, configured_created) =
        OwnedJournal::create(&configured_owned_root, &seed)?;
    let configured_revision_one_result =
        configured_writer.commit(&configured_created.head.revision.id, &revision_one)?;
    drop(configured_writer);

    let profile_config = output_root.join("workbench-profiles.json");
    let profile_alpha_r1_root = output_root.join("configured-alpha-r1");
    let profile_beta_r1_root = output_root.join("configured-beta-r1");
    let alpha_configured = WorkbenchProfiles::configure(
        &profile_config,
        WorkbenchProfileConfigureRequest {
            name: "alpha".into(),
            catalogue_root: Some(output_root.join("catalogue")),
            owned_root: Some(configured_owned_root.clone()),
            selection: selection.clone(),
            make_default: true,
        },
        None,
    )?;
    if alpha_configured.revision != 1 || !alpha_configured.changed {
        return Err("alpha configuration must create profile revision one".into());
    }
    let alpha_r1 = WorkbenchProfiles::refresh(&profile_config, None, &profile_alpha_r1_root, 1)?;
    if alpha_r1.publication.revision != 2
        || alpha_r1.publication.active.as_ref().map(|row| row.sequence) != Some(0)
    {
        return Err("alpha r1 refresh must publish sequence zero at revision two".into());
    }
    let mut alpha_engine = ConfiguredWorkbenchSearchEngine::open(&profile_config, None)?;
    let configured_alpha_r1 = alpha_engine.execute(request.clone())?;
    let vlajmaji_search::WorkbenchFullResult::Exact(alpha_exact) = &configured_alpha_r1.result
    else {
        return Err("configured alpha must retain the Full Exact result tag".into());
    };
    if alpha_exact.totals.occurrences != 3
        || alpha_exact.groups.len() != 2
        || serde_json::to_vec(alpha_exact)? != serde_json::to_vec(actual_r1)?
    {
        return Err("configured alpha r1 must retain the exact mixed emitted-input result".into());
    }
    let configured_alpha_reference = alpha_engine.resolve_reference(&upstream_target)?;
    let configured_alpha_compact_reference =
        alpha_engine.resolve_reference_compact(&upstream_target)?;
    if configured_alpha_reference.result.status != ReferenceStatus::Resolved
        || serde_json::to_vec(&configured_alpha_reference.result)?
            != serde_json::to_vec(&direct_upstream_reference.result)?
        || configured_alpha_reference.configuration.config_revision != 2
        || configured_alpha_compact_reference.result.status != ReferenceStatus::Resolved
        || configured_alpha_compact_reference
            .configuration
            .config_revision
            != 2
    {
        return Err(
            "configured alpha r1 must preserve the direct reference body and revision".into(),
        );
    }

    let beta_selection = WorkbenchSelection::from_json_bytes(PROFILE_BETA_SELECTION_JSON)?;
    let beta_configured = WorkbenchProfiles::configure(
        &profile_config,
        WorkbenchProfileConfigureRequest {
            name: "beta".into(),
            catalogue_root: None,
            owned_root: Some(configured_owned_root.clone()),
            selection: beta_selection.clone(),
            make_default: false,
        },
        Some(2),
    )?;
    if beta_configured.revision != 3 || beta_configured.default_profile.as_deref() != Some("alpha")
    {
        return Err("beta configuration must preserve alpha as default at revision three".into());
    }
    let profile_bytes_before_noop = fs::read(&profile_config)?;
    let profile_sha_before_noop = Sha256::digest(&profile_bytes_before_noop);
    let beta_noop = WorkbenchProfiles::configure(
        &profile_config,
        WorkbenchProfileConfigureRequest {
            name: "beta".into(),
            catalogue_root: None,
            owned_root: Some(configured_owned_root.clone()),
            selection: beta_selection.clone(),
            make_default: false,
        },
        Some(3),
    )?;
    let profile_bytes_after_noop = fs::read(&profile_config)?;
    if beta_noop.changed
        || beta_noop.revision != 3
        || profile_bytes_after_noop != profile_bytes_before_noop
        || Sha256::digest(&profile_bytes_after_noop) != profile_sha_before_noop
    {
        return Err("the exact beta no-op must retain revision three and config bytes".into());
    }
    let mut named_alpha_engine =
        ConfiguredWorkbenchSearchEngine::open(&profile_config, Some("alpha"))?;
    let named_alpha_r1 = named_alpha_engine.execute(request.clone())?;
    if serde_json::to_vec(&named_alpha_r1.result)?
        != serde_json::to_vec(&configured_alpha_r1.result)?
        || named_alpha_r1.binding_sha256 == configured_alpha_r1.binding_sha256
    {
        return Err(
            "named alpha must match the default body with a distinct effective identity".into(),
        );
    }
    let named_alpha_reference = named_alpha_engine.resolve_reference(&upstream_target)?;
    if serde_json::to_vec(&named_alpha_reference.result)?
        != serde_json::to_vec(&configured_alpha_reference.result)?
        || named_alpha_reference.binding_sha256 == configured_alpha_reference.binding_sha256
        || named_alpha_reference.configuration.config_revision != 3
    {
        return Err(
            "named alpha must retain the same r1 reference with a distinct identity".into(),
        );
    }
    if !matches!(
        alpha_engine.execute(request.clone()),
        Err(ConfiguredWorkbenchQueryError::Profile(error))
            if matches!(error.as_ref(), WorkbenchProfileError::ConfigChanged { .. })
    ) {
        return Err("an alpha engine must reject the changed config capture".into());
    }
    if !matches!(
        alpha_engine.resolve_reference(&upstream_target),
        Err(ConfiguredWorkbenchQueryError::Profile(error))
            if matches!(error.as_ref(), WorkbenchProfileError::ConfigChanged { .. })
    ) {
        return Err("an alpha reference engine must reject the changed config capture".into());
    }
    let beta_r1 =
        WorkbenchProfiles::refresh(&profile_config, Some("beta"), &profile_beta_r1_root, 3)?;
    if beta_r1.publication.revision != 4
        || beta_r1.publication.active.as_ref().map(|row| row.sequence) != Some(1)
    {
        return Err("beta r1 refresh must publish sequence one at revision four".into());
    }
    let mut beta_engine_at_revision_four =
        ConfiguredWorkbenchSearchEngine::open(&profile_config, Some("beta"))?;
    let configured_beta_r1 = beta_engine_at_revision_four.execute(request.clone())?;
    let vlajmaji_search::WorkbenchFullResult::Exact(beta_r1_exact) = &configured_beta_r1.result
    else {
        return Err("configured beta must retain the Full Exact result tag".into());
    };
    if beta_r1_exact.totals.occurrences != 2
        || beta_r1_exact.groups.len() != 1
        || serde_json::to_vec(&beta_r1_exact.groups[0].hits)?
            != serde_json::to_vec(&actual_r1.groups[1].hits)?
    {
        return Err("configured beta r1 must retain the exact two emitted owned hits".into());
    }
    let configured_beta_owned =
        beta_engine_at_revision_four.resolve_reference(&owned_entry_target)?;
    let configured_beta_owned_compact =
        beta_engine_at_revision_four.resolve_reference_compact(&owned_entry_target)?;
    if configured_beta_owned.result.status != ReferenceStatus::Resolved
        || !configured_beta_owned.result.catalogue_matches.is_empty()
        || !configured_beta_owned.result.registrations.is_empty()
        || !matches!(
            configured_beta_owned.result.payload.as_deref().and_then(|payload| payload.entity.as_ref()),
            Some(ReferenceEntity::Entry(entry)) if entry == expected_owned_entry
        )
        || configured_beta_owned_compact.result.status != ReferenceStatus::Resolved
        || !configured_beta_owned_compact
            .result
            .catalogue_matches
            .is_empty()
    {
        return Err("configured beta r1 must resolve the owned Entry without a catalogue".into());
    }
    let default = WorkbenchProfiles::set_default(&profile_config, Some("beta"), 4)?;
    if default.revision != 5 || default.default_profile.as_deref() != Some("beta") {
        return Err("setting beta as default must produce config revision five".into());
    }
    if !matches!(
        beta_engine_at_revision_four.execute(request.clone()),
        Err(ConfiguredWorkbenchQueryError::Profile(error))
            if matches!(error.as_ref(), WorkbenchProfileError::ConfigChanged { .. })
    ) {
        return Err("changing the default must invalidate the captured beta engine".into());
    }
    if !matches!(
        beta_engine_at_revision_four.resolve_reference(&owned_entry_target),
        Err(ConfiguredWorkbenchQueryError::Profile(error))
            if matches!(error.as_ref(), WorkbenchProfileError::ConfigChanged { .. })
    ) {
        return Err("changing the default must invalidate beta reference capture".into());
    }
    let mut beta_engine = ConfiguredWorkbenchSearchEngine::open(&profile_config, Some("beta"))?;
    let configured_beta_r1 = beta_engine.execute(request.clone())?;
    let vlajmaji_search::WorkbenchFullResult::Exact(beta_exact) = &configured_beta_r1.result else {
        return Err("configured beta must retain the Full Exact result tag".into());
    };
    if beta_exact.totals.occurrences != 2
        || beta_exact.groups.len() != 1
        || serde_json::to_vec(&beta_exact.groups[0].hits)?
            != serde_json::to_vec(&actual_r1.groups[1].hits)?
    {
        return Err("configured beta r1 must retain the exact two emitted owned hits".into());
    }
    let configured_beta_reference = beta_engine.resolve_reference(&owned_entry_target)?;
    if configured_beta_reference.result.status != ReferenceStatus::Resolved
        || !configured_beta_reference
            .result
            .catalogue_matches
            .is_empty()
        || !configured_beta_reference.result.registrations.is_empty()
        || serde_json::to_vec(&configured_beta_reference.result)?
            != serde_json::to_vec(&configured_beta_owned.result)?
    {
        return Err("configured beta r1 must retain the catalogue-free owned reference".into());
    }

    let historical_r1_root = output_root.join("workbench-historical-r1");
    let historical_r1 = vlajmaji_store::WorkbenchView::build_new(
        &historical_r1_root,
        WorkbenchBuildRequest {
            catalogue_root: Some(output_root.join("catalogue")),
            owned_root: Some(owned_root.clone()),
            selection: historical_selection,
        },
    )?;
    if historical_r1
        .binding
        .owned()
        .map(|owned| owned.identity.revision.as_str())
        != Some("owned-r1")
    {
        return Err("the historical view must select owned-r1 before the r2 append".into());
    }
    let mut search_historical_r1 = WorkbenchSearchEngine::open(&historical_r1_root)?;
    let historical_full_r1 = search_historical_r1.execute(request.clone())?;
    if serde_json::to_vec(&historical_full_r1.result)? != serde_json::to_vec(&full_r1.result)? {
        return Err("the historical r1 result must match the emitted r1 input".into());
    }
    let historical_owned_reference = search_historical_r1.resolve_reference(&owned_entry_target)?;
    if historical_owned_reference.result.status != ReferenceStatus::Resolved
        || !matches!(
            historical_owned_reference
                .result
                .payload
                .as_deref()
                .and_then(|payload| payload.entity.as_ref()),
            Some(ReferenceEntity::Entry(entry)) if entry == expected_owned_entry
        )
    {
        return Err("the historical owned-r1 reference must match its typed input".into());
    }

    let mut default_beta_engine = ConfiguredWorkbenchSearchEngine::open(&profile_config, None)?;
    let default_beta_before_append = default_beta_engine.execute(request.clone())?;
    if default_beta_before_append
        .configuration
        .requested_profile
        .is_some()
        || serde_json::to_vec(&default_beta_before_append.result)?
            != serde_json::to_vec(&configured_beta_r1.result)?
    {
        return Err("default beta must use the beta r1 body with default-mode identity".into());
    }
    let default_beta_reference_before_append =
        default_beta_engine.resolve_reference(&owned_entry_target)?;
    if default_beta_reference_before_append.result.status != ReferenceStatus::Resolved
        || serde_json::to_vec(&default_beta_reference_before_append.result)?
            != serde_json::to_vec(&configured_beta_reference.result)?
        || default_beta_reference_before_append.binding_sha256
            == configured_beta_reference.binding_sha256
    {
        return Err(
            "named and default beta must share the r1 reference body with distinct identities"
                .into(),
        );
    }

    let mut configured_writer = OwnedJournal::open(&configured_owned_root)?;
    let configured_revision_two_result = configured_writer.commit(
        &configured_revision_one_result.head.revision.id,
        &revision_two,
    )?;
    drop(configured_writer);

    let mut writer = OwnedJournal::open(&owned_root)?;
    let revision_two_result =
        writer.commit(&revision_one_result.head.revision.id, &revision_two)?;
    drop(writer);
    if !matches!(
        default_beta_engine.execute(request.clone()),
        Err(ConfiguredWorkbenchQueryError::Profile(error))
            if matches!(error.as_ref(), WorkbenchProfileError::View(inner)
                if matches!(inner.as_ref(), WorkbenchError::StaleDependencies {
                    dependency: vlajmaji_store::WorkbenchDependency::Owned,
                    ..
                }))
    ) {
        return Err(
            "the retained default beta Head view must stale after the configured r2 append".into(),
        );
    }
    if !matches!(
        default_beta_engine.resolve_reference(&owned_entry_target),
        Err(ConfiguredWorkbenchQueryError::Profile(error))
            if matches!(error.as_ref(), WorkbenchProfileError::View(inner)
                if matches!(inner.as_ref(), WorkbenchError::StaleDependencies {
                    dependency: vlajmaji_store::WorkbenchDependency::Owned,
                    ..
                }))
    ) {
        return Err("the default beta reference Head must stale after the r2 append".into());
    }
    if !matches!(
        beta_engine.execute(request.clone()),
        Err(ConfiguredWorkbenchQueryError::Profile(error))
            if matches!(error.as_ref(), WorkbenchProfileError::View(inner)
                if matches!(inner.as_ref(), WorkbenchError::StaleDependencies {
                    dependency: vlajmaji_store::WorkbenchDependency::Owned,
                    ..
                }))
    ) {
        return Err(
            "the configured beta r1 Head view must become stale after the r2 append".into(),
        );
    }
    if !matches!(
        beta_engine.resolve_reference(&owned_entry_target),
        Err(ConfiguredWorkbenchQueryError::Profile(error))
            if matches!(error.as_ref(), WorkbenchProfileError::View(inner)
                if matches!(inner.as_ref(), WorkbenchError::StaleDependencies {
                    dependency: vlajmaji_store::WorkbenchDependency::Owned,
                    ..
                }))
    ) {
        return Err("the named beta reference Head must stale after the r2 append".into());
    }
    if !matches!(
        search_r1.execute(request.clone()),
        Err(WorkbenchQueryError::View(error))
            if matches!(error.as_ref(), WorkbenchError::StaleDependencies {
                dependency: vlajmaji_store::WorkbenchDependency::Owned,
                ..
            })
    ) {
        return Err("the retained r1 Head view must become stale after the r2 commit".into());
    }
    if !matches!(
        search_r1.resolve_reference(&owned_entry_target),
        Err(WorkbenchQueryError::View(error))
            if matches!(error.as_ref(), WorkbenchError::StaleDependencies {
                dependency: vlajmaji_store::WorkbenchDependency::Owned,
                ..
            })
    ) {
        return Err("the retained r1 reference Head must stale after the r2 commit".into());
    }
    if historical_r1
        .binding
        .owned()
        .map(|owned| owned.identity.revision.as_str())
        != Some("owned-r1")
        || serde_json::to_vec(&search_historical_r1.execute(request.clone())?.result)?
            != serde_json::to_vec(&historical_full_r1.result)?
        || serde_json::to_vec(
            &search_historical_r1
                .resolve_reference(&owned_entry_target)?
                .result,
        )? != serde_json::to_vec(&historical_owned_reference.result)?
        || serde_json::to_vec(&historical_full_r1.result)? != serde_json::to_vec(&full_r1.result)?
    {
        return Err(
            "the same retained historical r1 view must remain byte-equivalent after the r2 append"
                .into(),
        );
    }
    let built_r2 = vlajmaji_store::WorkbenchView::build_new(
        &workbench_r2_root,
        WorkbenchBuildRequest {
            catalogue_root: Some(output_root.join("catalogue")),
            owned_root: Some(owned_root),
            selection,
        },
    )?;
    let mut search_r2 = WorkbenchSearchEngine::open(&workbench_r2_root)?;
    let full_r2 = search_r2.execute(request.clone())?;
    let WorkbenchFullResult::Exact(actual_r2) = &full_r2.result else {
        return Err("the emitted exact request did not produce an exact r2 response".into());
    };
    let expected_owned_r2_assertions = revision_two
        .metadata()
        .assertions
        .iter()
        .filter(|assertion| {
            revision_two.metadata().entries.iter().any(|entry| {
                entry.id == assertion.entry && entry.spelling == Presence::Value("farvi".into())
            })
        })
        .collect::<Vec<_>>();
    if actual_r2.groups.len() != 2
        || actual_r2.groups[0].source.id.as_str() != "lensisku"
        || actual_r2.groups[1].source.id.as_str() != "owned-example"
        || actual_r2.groups[0].hits.len() != 1
        || actual_r2.groups[1].hits.len() != 3
        || actual_r2.totals.occurrences != 4
        || actual_r2.groups[1]
            .hits
            .iter()
            .map(|hit| match &hit.assertion.definition {
                Presence::Value(text) => text.text.as_str(),
                Presence::Missing | Presence::Null => "<absent>",
            })
            .collect::<Vec<_>>()
            != ["grow", "develop", "flourish"]
        || actual_r2.groups[1].hits.len() != expected_owned_r2_assertions.len()
        || actual_r2.groups[1]
            .hits
            .iter()
            .zip(&expected_owned_r2_assertions)
            .any(|(hit, assertion)| {
                hit.entry
                    != *revision_two
                        .metadata()
                        .entries
                        .iter()
                        .find(|entry| entry.id == assertion.entry)
                        .expect("fixture assertion has an entry")
                    || hit.assertion != **assertion
            })
    {
        return Err("the refreshed r2 query must retain all four competing definitions".into());
    }
    let profile_beta_r2_root = output_root.join("configured-beta-r2");
    let beta_r2 = WorkbenchProfiles::refresh(&profile_config, None, &profile_beta_r2_root, 5)?;
    if beta_r2.publication.revision != 6
        || beta_r2.publication.active.as_ref().map(|row| row.sequence) != Some(2)
    {
        return Err("default beta r2 refresh must publish sequence two at revision six".into());
    }
    let mut beta_r2_engine = ConfiguredWorkbenchSearchEngine::open(&profile_config, Some("beta"))?;
    let configured_beta_r2 = beta_r2_engine.execute(request.clone())?;
    let vlajmaji_search::WorkbenchFullResult::Exact(beta_r2_exact) = &configured_beta_r2.result
    else {
        return Err("configured beta r2 must retain the Full Exact result tag".into());
    };
    if beta_r2_exact.totals.occurrences != 3
        || beta_r2_exact.groups.len() != 1
        || serde_json::to_vec(&beta_r2_exact.groups[0].hits)?
            != serde_json::to_vec(&actual_r2.groups[1].hits)?
    {
        return Err("configured beta r2 must retain the exact three emitted owned hits".into());
    }
    let configured_beta_r2_reference = beta_r2_engine.resolve_reference(&owned_entry_target)?;
    if configured_beta_r2_reference.result.status != ReferenceStatus::Resolved
        || !matches!(
            configured_beta_r2_reference
                .result
                .payload
                .as_deref()
                .and_then(|payload| payload.entity.as_ref()),
            Some(ReferenceEntity::Entry(entry)) if entry == expected_owned_entry
        )
    {
        return Err("configured beta r2 must resolve the emitted owned Entry".into());
    }
    let history = WorkbenchProfiles::read(&profile_config)?.history(None)?;
    if default.revision != 5
        || default.default_profile.as_deref() != Some("beta")
        || history
            .prepared
            .iter()
            .map(|row| row.sequence)
            .collect::<Vec<_>>()
            != [0, 1, 2]
    {
        return Err("profile history must retain three rows through revision six".into());
    }
    let mut named_beta_engine =
        ConfiguredWorkbenchSearchEngine::open(&profile_config, Some("beta"))?;
    let named_beta_r2 = named_beta_engine.execute(request.clone())?;
    let mut refreshed_default_beta_engine =
        ConfiguredWorkbenchSearchEngine::open(&profile_config, None)?;
    let default_beta_r2 = refreshed_default_beta_engine.execute(request.clone())?;
    if serde_json::to_vec(&named_beta_r2.result)? != serde_json::to_vec(&default_beta_r2.result)?
        || named_beta_r2.binding_sha256 == default_beta_r2.binding_sha256
    {
        return Err("named and default beta must share the r2 body but differ in identity".into());
    }
    let lexical = run_lexical_journey(&output_root)?;
    let links = run_links_journey(&output_root)?;
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "inputs": inputs,
            "revision_one": revision_one_result,
            "revision_two": revision_two_result,
            "binding_sha256_r1": built_r1.binding.binding_sha256(),
            "state_sha256_r1": built_r1.binding.state_sha256(),
            "binding_sha256_r2": built_r2.binding.binding_sha256(),
            "state_sha256_r2": built_r2.binding.state_sha256(),
            "expected": expected,
            "full_r1": full_r1,
            "historical_full_r1": historical_full_r1,
            "reference_direct_upstream": direct_upstream_reference,
            "reference_direct_compact_upstream": direct_compact_upstream,
            "reference_direct_detail": direct_reference_detail,
            "reference_direct_owned_entry": direct_owned_entry,
            "reference_direct_owned_assertion": direct_owned_assertion,
            "reference_direct_unknown": direct_unknown,
            "reference_direct_unresolved": direct_unresolved,
            "reference_disabled_upstream": disabled_upstream,
            "reference_disabled_owned": disabled_owned,
            "historical_binding_sha256": historical_r1.binding.binding_sha256(),
            "full_r2": full_r2,
            "profile_revisions": [
                alpha_configured.revision,
                alpha_r1.publication.revision,
                beta_configured.revision,
                beta_r1.publication.revision,
                default.revision,
                beta_r2.publication.revision
            ],
            "profile_history": history,
            "configured_alpha_r1": configured_alpha_r1,
            "configured_alpha_reference": configured_alpha_reference,
            "configured_alpha_compact_reference": configured_alpha_compact_reference,
            "named_alpha_reference": named_alpha_reference,
            "configured_beta_r1": configured_beta_r1,
            "configured_beta_reference": configured_beta_reference,
            "configured_beta_owned_compact": configured_beta_owned_compact,
            "configured_beta_r2": configured_beta_r2,
            "configured_beta_r2_reference": configured_beta_r2_reference,
            "beta_noop": beta_noop,
            "configured_revision_two": configured_revision_two_result,
            "named_alpha_r1": named_alpha_r1,
            "default_beta_before_append": default_beta_before_append,
            "default_beta_reference_before_append": default_beta_reference_before_append,
            "historical_owned_reference": historical_owned_reference,
            "named_beta_r2": named_beta_r2,
            "lexical": lexical,
            "links": links,
        }))?
    );
    Ok(())
}
