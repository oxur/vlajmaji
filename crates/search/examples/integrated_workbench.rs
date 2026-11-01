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
    ComponentRecord, ContentNotice, ContributorId, ContributorRecord, Dictionary,
    DictionaryDocument, DictionaryMetadata, Eligibility, EntityId, EntryRecord,
    LaLojbanImportOptions, NativeEncoding, NativeField, OccurrenceAttribution, OccurrenceRef,
    OwnedHead, OwnedRevisionRecord, Presence, RawCapture, RawOccurrence, RedistributionStatus,
    RegistryDocument, ReleaseAccounting, ReleaseRecord, SnapshotAttribution, SourceId,
    SourceOrigin, SourceRecord, SourceRegistry, TextValue, TimePrecision, import_la_lojban_export,
    seal_release,
};
use vlajmaji_search::{
    ConfiguredWorkbenchQueryError, ConfiguredWorkbenchSearchEngine, DictionaryExactOptions,
    DictionaryRequest, WorkbenchFullResult, WorkbenchQueryError, WorkbenchSearchEngine,
};
use vlajmaji_store::{
    CollectionCatalogue, CollectionInput, CollectionRequest, CollectionSelection, OwnedJournal,
    WorkbenchBuildRequest, WorkbenchError, WorkbenchOwnedRevision,
    WorkbenchProfileConfigureRequest, WorkbenchProfileError, WorkbenchProfiles, WorkbenchSelection,
};

const UPSTREAM_JSON: &[u8] = br#"[
  {"word":"farvi","word_type":"gismu","definition":"advance","definition_id":1}
]"#;
const PROFILE_BETA_SELECTION_JSON: &[u8] =
    br#"{"version":1,"packs":[],"disabled":[],"owned":"head","contexts":[]}"#;
const PROFILES_EXPECTED_JSON: &[u8] = br#"{"version":1,"config_revisions":[1,2,3,4,5,6],"prepared_sequences":[0,1,2],"alpha_r1":{"occurrences":3,"source_groups":2},"beta_r1":{"occurrences":2,"source_groups":1},"beta_r2":{"occurrences":3,"source_groups":1}}"#;

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

fn write_new(path: &Path, bytes: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
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
    if !matches!(
        alpha_engine.execute(request.clone()),
        Err(ConfiguredWorkbenchQueryError::Profile(error))
            if matches!(error.as_ref(), WorkbenchProfileError::ConfigChanged { .. })
    ) {
        return Err("an alpha engine must reject the changed config capture".into());
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
        search_r1.execute(request.clone()),
        Err(WorkbenchQueryError::View(error))
            if matches!(error.as_ref(), WorkbenchError::StaleDependencies {
                dependency: vlajmaji_store::WorkbenchDependency::Owned,
                ..
            })
    ) {
        return Err("the retained r1 Head view must become stale after the r2 commit".into());
    }
    if historical_r1
        .binding
        .owned()
        .map(|owned| owned.identity.revision.as_str())
        != Some("owned-r1")
        || serde_json::to_vec(&search_historical_r1.execute(request.clone())?.result)?
            != serde_json::to_vec(&historical_full_r1.result)?
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
            "configured_beta_r1": configured_beta_r1,
            "configured_beta_r2": configured_beta_r2,
            "beta_noop": beta_noop,
            "configured_revision_two": configured_revision_two_result,
            "named_alpha_r1": named_alpha_r1,
            "default_beta_before_append": default_beta_before_append,
            "named_beta_r2": named_beta_r2,
        }))?
    );
    Ok(())
}
