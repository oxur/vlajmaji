//! Direct and configured Links CLI routes over the fixed mixed typed fixture.

use std::{
    borrow::Cow,
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};

#[cfg(unix)]
use std::{
    io::Write,
    os::{
        fd::OwnedFd,
        unix::{
            ffi::OsStringExt,
            net::{UnixDatagram, UnixStream},
        },
    },
    process::Stdio,
};

use vlajmaji_lexicon::{
    AbbreviationKind, AssertionRecord, Availability, CaptureMetadata, CaptureRef, CollectionRecord,
    ComponentRecord, ContentNotice, ContextKind, ContextRecord, ContributorId, ContributorRecord,
    Dictionary, DictionaryDocument, DictionaryMetadata, Direction, EditorialSenseRecord,
    Eligibility, EntityId, EntryRecord, LocalRef, NativeEncoding, NativeField,
    OccurrenceAttribution, OccurrenceRef, OwnedHead, OwnedRevisionRecord, PackId, PackPayloadKind,
    Presence, RawCapture, RawOccurrence, RedistributionStatus, RegistryDocument, RelationKind,
    RelationRecord, RelationScope, RelationTarget, ReleaseAccounting, ReleaseRecord,
    SnapshotAttribution, SourceEvidence, SourceId, SourceOrigin, SourceRecord, SourceRegistry,
    TextValue, TimePrecision, seal_release,
};
use vlajmaji_store::{
    CollectionCatalogue, CollectionInput, CollectionRequest, CollectionSelection, OwnedJournal,
    WorkbenchBuildRequest, WorkbenchOwnedRevision, WorkbenchProfileConfigureRequest,
    WorkbenchProfiles, WorkbenchSelection, WorkbenchView,
};

fn cli(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
        .args(args)
        .output()
        .expect("launch vlajmaji")
}

fn write_json(path: &Path, value: &serde_json::Value) {
    fs::write(path, serde_json::to_vec(value).unwrap()).unwrap();
}

fn result_ok(output: Output) -> serde_json::Value {
    assert!(
        output.status.success(),
        "CLI failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("successful CLI response is JSON")
}

fn assert_error_envelope(output: &Output, code: &str) {
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert_eq!(output.stderr.last(), Some(&b'\n'));
    assert_eq!(
        output.stderr.iter().filter(|byte| **byte == b'\n').count(),
        1
    );
    let envelope: serde_json::Value = serde_json::from_slice(&output.stderr).unwrap();
    let mut outer = envelope
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect::<Vec<_>>();
    outer.sort_unstable();
    assert_eq!(outer, ["error"]);
    let mut fields = envelope["error"]
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect::<Vec<_>>();
    fields.sort_unstable();
    assert_eq!(fields, ["code", "message", "sources"]);
    assert_eq!(envelope["error"]["code"], code);
    assert_eq!(envelope["error"]["sources"], serde_json::json!([]));
}

#[cfg(unix)]
fn saturated_stdout() -> (std::fs::File, UnixDatagram) {
    let (sender, peer) = UnixDatagram::pair().unwrap();
    sender.set_nonblocking(true).unwrap();
    let mut file = std::fs::File::from(OwnedFd::from(sender));
    let mut writes = 0usize;
    loop {
        match file.write(b"x") {
            Ok(1) => {
                writes += 1;
                assert!(writes < 1_000_000, "datagram queue did not saturate");
            }
            Ok(_) => unreachable!("single byte write must be complete or fail"),
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => break,
            #[cfg(target_os = "macos")]
            Err(error) if error.raw_os_error() == Some(55) => break,
            Err(error) => panic!("unexpected saturation error: {error}"),
        }
    }
    assert!(writes > 0);
    (file, peer)
}

#[cfg(unix)]
fn broken_pipe_stdout() -> std::fs::File {
    let (peer, writer) = UnixStream::pair().unwrap();
    drop(peer);
    let mut file = std::fs::File::from(OwnedFd::from(writer));
    let error = file
        .write(b"x")
        .expect_err("closed peer must reject a write");
    assert_eq!(error.kind(), std::io::ErrorKind::BrokenPipe);
    file
}

#[cfg(unix)]
#[test]
#[ignore = "invoked as an isolated child after deleting its own working directory"]
fn test_links_cli_deleted_cwd_child() {
    let cwd = PathBuf::from(std::env::var_os("VLJ_LINKS_DELETED_CWD").unwrap());
    std::env::set_current_dir(&cwd).unwrap();
    fs::remove_dir(&cwd).unwrap();
    assert!(std::env::current_dir().is_err());

    let view = PathBuf::from(std::env::var_os("VLJ_LINKS_DELETED_VIEW").unwrap());
    let request = PathBuf::from(std::env::var_os("VLJ_LINKS_DELETED_REQUEST").unwrap());
    let direct_reference =
        PathBuf::from(std::env::var_os("VLJ_LINKS_DELETED_DIRECT_REFERENCE").unwrap());
    let direct_binding = std::env::var("VLJ_LINKS_DELETED_DIRECT_BINDING").unwrap();
    let config = PathBuf::from(std::env::var_os("VLJ_LINKS_DELETED_CONFIG").unwrap());
    let configured_reference =
        PathBuf::from(std::env::var_os("VLJ_LINKS_DELETED_CONFIG_REFERENCE").unwrap());
    let configured_binding = std::env::var("VLJ_LINKS_DELETED_CONFIG_BINDING").unwrap();

    let direct = Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
        .args(["workbench", "lexical"])
        .arg(&view)
        .args(["--request"])
        .arg(&request)
        .output()
        .unwrap();
    assert_links_totals(&result_ok(direct));
    let direct_detail = Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
        .args(["workbench", "link-detail"])
        .arg(&view)
        .args(["--reference"])
        .arg(&direct_reference)
        .args(["--expected-binding", &direct_binding])
        .output()
        .unwrap();
    assert_eq!(
        result_ok(direct_detail)["kind"],
        "vlajmaji-workbench-link-detail"
    );

    let configured = Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
        .args(["workbench", "profiles", "lexical", "--config"])
        .arg(&config)
        .args(["--profile", "links", "--request"])
        .arg(&request)
        .output()
        .unwrap();
    assert_links_totals(&result_ok(configured));
    let configured_detail = Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
        .args(["workbench", "profiles", "link-detail", "--config"])
        .arg(&config)
        .args(["--profile", "links", "--reference"])
        .arg(&configured_reference)
        .args(["--expected-binding", &configured_binding])
        .output()
        .unwrap();
    assert_eq!(
        result_ok(configured_detail)["kind"],
        "vlajmaji-workbench-link-detail"
    );

    let relative_config = Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
        .args([
            "workbench",
            "profiles",
            "lexical",
            "--config",
            "links-profiles.json",
        ])
        .args(["--profile", "links", "--request"])
        .arg(&request)
        .output()
        .unwrap();
    assert_error_envelope(&relative_config, "workbench_config_io");
    let relative_request = Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
        .args(["workbench", "lexical"])
        .arg(&view)
        .args(["--request", "links request ü.json"])
        .output()
        .unwrap();
    assert_error_envelope(&relative_request, "workbench_io");
}

#[cfg(unix)]
struct DeletedCwdLinksInputs<'a> {
    view: &'a Path,
    request: &'a Path,
    direct_reference: &'a Path,
    direct_binding: &'a str,
    config: &'a Path,
    configured_reference: &'a Path,
    configured_binding: &'a str,
}

#[cfg(unix)]
fn run_links_after_child_deletes_cwd(cwd: &Path, inputs: DeletedCwdLinksInputs<'_>) {
    let output = Command::new(std::env::current_exe().unwrap())
        .args([
            "--ignored",
            "--exact",
            "test_links_cli_deleted_cwd_child",
            "--nocapture",
        ])
        .env("VLJ_LINKS_DELETED_CWD", cwd)
        .env("VLJ_LINKS_DELETED_VIEW", inputs.view)
        .env("VLJ_LINKS_DELETED_REQUEST", inputs.request)
        .env(
            "VLJ_LINKS_DELETED_DIRECT_REFERENCE",
            inputs.direct_reference,
        )
        .env("VLJ_LINKS_DELETED_DIRECT_BINDING", inputs.direct_binding)
        .env("VLJ_LINKS_DELETED_CONFIG", inputs.config)
        .env(
            "VLJ_LINKS_DELETED_CONFIG_REFERENCE",
            inputs.configured_reference,
        )
        .env(
            "VLJ_LINKS_DELETED_CONFIG_BINDING",
            inputs.configured_binding,
        )
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "deleted-cwd Links child failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

static NEXT: AtomicUsize = AtomicUsize::new(0);

fn fresh_root() -> PathBuf {
    for _ in 0..64 {
        let root = std::env::temp_dir().join(format!(
            "vlajmaji-links-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed),
        ));
        match fs::create_dir(&root) {
            Ok(()) => return root,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => panic!("create exclusive Links fixture root: {error}"),
        }
    }
    panic!("could not allocate an exclusive Links root after 64 attempts")
}

fn id(value: &str) -> EntityId {
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
fn evidence(occurrence: OccurrenceRef, field_index: Option<usize>) -> Option<SourceEvidence> {
    Some(SourceEvidence {
        occurrence,
        field_index,
        citation: Presence::Missing,
        offset: None,
    })
}
fn release(
    id_value: &str,
    component: &EntityId,
    captures: Vec<CaptureRef>,
    assertions: Vec<EntityId>,
    selected: bool,
) -> ReleaseRecord {
    let n = assertions.len() as u64;
    ReleaseRecord {
        id: id(id_value),
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
fn capture(
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
fn field(name: &str, value: &str) -> NativeField {
    NativeField {
        name: name.into(),
        encoding: NativeEncoding::Text,
        raw_value: value.into(),
    }
}
fn entry(id_value: &str, source_id: &SourceId, spelling: &str) -> EntryRecord {
    EntryRecord {
        id: id(id_value),
        source: source_id.clone(),
        spelling: Presence::Value(spelling.into()),
        language: Presence::Value("jbo".into()),
        script: Presence::Missing,
        native_word_class: Presence::Missing,
        selmaho: Presence::Missing,
        mapped_pos: Presence::Missing,
    }
}
fn assertion(
    id_value: &str,
    entry_id: &EntityId,
    occurrence: OccurrenceRef,
    definition: &str,
) -> AssertionRecord {
    AssertionRecord {
        id: id(id_value),
        entry: entry_id.clone(),
        occurrence,
        native_sense_id: Presence::Missing,
        language: Presence::Value("en".into()),
        definition: text(definition),
        source_score: Presence::Missing,
        timestamps: Vec::new(),
        eligibility: Eligibility::Lexical,
    }
}
fn source_row(id_value: &str, abbreviation: &str, origin: SourceOrigin) -> SourceRecord {
    SourceRecord {
        id: source(id_value),
        display_name: id_value.into(),
        abbreviation: abbreviation.into(),
        abbreviation_kind: AbbreviationKind::ProjectAssigned,
        abbreviation_evidence: Some("fixed Links input".into()),
        origin,
    }
}
fn collection_component(
    source_id: &SourceId,
    collection_id: &str,
    component_id: &str,
) -> (CollectionRecord, ComponentRecord) {
    (
        CollectionRecord {
            id: id(collection_id),
            source: source_id.clone(),
            display_name: collection_id.into(),
            description: Presence::Missing,
        },
        ComponentRecord {
            id: id(component_id),
            collection: id(collection_id),
            native_name: component_id.into(),
            languages: vec!["jbo".into()],
        },
    )
}

fn owned_r0() -> Dictionary {
    let lo = source("links-owned");
    let editor = ContributorId::new("links-editor").unwrap();
    let (collection, component) =
        collection_component(&lo, "links-owned-collection", "links-owned-component");
    let registry = SourceRegistry::from_document(RegistryDocument {
        sources: vec![source_row("links-owned", "KO", SourceOrigin::OwnedDraft)],
        contributors: vec![ContributorRecord {
            id: editor.clone(),
            display_name: "Links fixture editor".into(),
        }],
        ..RegistryDocument::default()
    })
    .unwrap();
    let revision = id("links-owned-r0");
    let mut metadata = DictionaryMetadata::default();
    metadata.collections.push(collection);
    metadata.components.push(component.clone());
    metadata.releases.push(release(
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
        rationale: text("genesis"),
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

fn owned_r1(previous: &Dictionary) -> Dictionary {
    let lo = source("links-owned");
    let editor = ContributorId::new("links-editor").unwrap();
    let mut doc = previous.document();
    let lines = [
        "seed | Links fixture a",
        "branch | Links fixture b",
        "leaf | Links fixture c",
    ];
    let capture = capture(
        "links-owned-capture-r1",
        "Links owned",
        &lines,
        lines
            .iter()
            .map(|line| {
                let (word, definition) = line.split_once(" | ").unwrap();
                vec![field("word", word), field("definition", definition)]
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
    let ea = id("links-owned-entry-a");
    let eb = id("links-owned-entry-b");
    let ec = id("links-owned-entry-c");
    doc.metadata.entries = vec![
        entry("links-owned-entry-a", &lo, "seed"),
        entry("links-owned-entry-b", &lo, "branch"),
        entry("links-owned-entry-c", &lo, "leaf"),
    ];
    doc.metadata.assertions = vec![
        assertion(
            "links-owned-a",
            &ea,
            occurrences[0].clone(),
            "Links fixture a",
        ),
        assertion(
            "links-owned-b",
            &eb,
            occurrences[1].clone(),
            "Links fixture b",
        ),
        assertion(
            "links-owned-c",
            &ec,
            occurrences[2].clone(),
            "Links fixture c",
        ),
    ];
    let component = id("links-owned-component");
    doc.metadata.releases.push(release(
        "links-owned-release-r1",
        &component,
        vec![cap],
        vec![
            id("links-owned-a"),
            id("links-owned-b"),
            id("links-owned-c"),
        ],
        true,
    ));
    let rev = id("links-owned-r1");
    doc.metadata.owned_revisions.push(OwnedRevisionRecord {
        id: rev.clone(),
        source: lo.clone(),
        parent: Some(id("links-owned-r0")),
        editor: editor.clone(),
        timestamp: Presence::Missing,
        time_precision: TimePrecision::Unknown,
        assertions: vec![
            id("links-owned-a"),
            id("links-owned-b"),
            id("links-owned-c"),
        ],
        tombstones: Vec::new(),
        rationale: text("append a, b, c"),
    });
    doc.metadata.owned_heads[0].revision = rev;
    doc.metadata.catalogue_revision = 1;
    Dictionary::from_document(doc).unwrap()
}

fn upstream_dictionary() -> Dictionary {
    let lu = source("links-upstream");
    let contributor = ContributorId::new("links-upstream-editor").unwrap();
    let (collection, component) =
        collection_component(&lu, "links-upstream-collection", "links-upstream-component");
    let lines = [
        "seed | Links fixture x | See also {branch} {branch} {missing}.",
        "branch | Links fixture y",
        "leaf | Links fixture z",
    ];
    let raw = capture(
        "links-upstream-capture-r1",
        "Links upstream",
        &lines,
        vec![
            vec![
                field("word", "seed"),
                field("definition", "Links fixture x"),
                field("notes", "See also {branch} {branch} {missing}."),
            ],
            vec![
                field("word", "branch"),
                field("definition", "Links fixture y"),
            ],
            vec![
                field("word", "leaf"),
                field("definition", "Links fixture z"),
            ],
        ],
    );
    let cap = raw.identity.clone();
    let occ = ["/0", "/1", "/2"].map(|locator| OccurrenceRef {
        capture: cap.clone(),
        locator: locator.into(),
    });
    let registry = SourceRegistry::from_document(RegistryDocument {
        sources: vec![source_row("links-upstream", "KU", SourceOrigin::Upstream)],
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
    let ex = id("links-upstream-entry-x");
    let ey = id("links-upstream-entry-y");
    let ez = id("links-upstream-entry-z");
    let ax = id("links-upstream-x");
    let ay = id("links-upstream-y");
    let az = id("links-upstream-z");
    let mut metadata = DictionaryMetadata::default();
    metadata.collections.push(collection);
    metadata.components.push(component.clone());
    metadata.releases.push(release(
        "links-upstream-release-r1",
        &component.id,
        vec![cap.clone()],
        vec![ax.clone(), ay.clone(), az.clone()],
        true,
    ));
    metadata.entries = vec![
        entry("links-upstream-entry-x", &lu, "seed"),
        entry("links-upstream-entry-y", &lu, "branch"),
        entry("links-upstream-entry-z", &lu, "leaf"),
        entry("links-upstream-entry-q", &lu, "branch"),
    ];
    metadata.assertions = vec![
        assertion("links-upstream-x", &ex, occ[0].clone(), "Links fixture x"),
        assertion("links-upstream-y", &ey, occ[1].clone(), "Links fixture y"),
        assertion("links-upstream-z", &ez, occ[2].clone(), "Links fixture z"),
    ];
    metadata.forms.push(vlajmaji_lexicon::FormRecord {
        id: id("links-upstream-form-z"),
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
        evidence: evidence(occ[2].clone(), None),
        computation: None,
    });
    let context_id = id("links-note-x");
    metadata.contexts.push(ContextRecord {
        id: context_id,
        assertion: ax.clone(),
        kind: ContextKind::Note,
        value: text("See also {branch} {branch} {missing}."),
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

fn fixed_links_dictionary() -> Dictionary {
    let owned = owned_r1(&owned_r0());
    let upstream = upstream_dictionary();
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
        id: id("links-owned-sense-ba"),
        assertions: vec![id("links-owned-b"), id("links-owned-a")],
        editor: ContributorId::new("links-editor").unwrap(),
        revision: id("links-owned-r1"),
        rationale: TextValue {
            text: "ordered b then a".into(),
            language: Presence::Missing,
            markup: Presence::Missing,
        },
    });
    let lu = source("links-upstream");
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
            id: id("links-relation-01"),
            origin: LocalRef::Assertion(id("links-upstream-x")),
            kind: RelationKind::SourceSeeAlso,
            direction: Direction::Forward,
            scope: RelationScope::Assertion,
            target: RelationTarget::Local(LocalRef::Entry(id("links-owned-entry-b"))),
            evidence: evidence(
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
            id: id("links-relation-02"),
            origin: LocalRef::Form(id("links-upstream-form-z")),
            kind: RelationKind::SourceEtymology,
            direction: Direction::Reverse,
            scope: RelationScope::Form,
            target: RelationTarget::Local(LocalRef::Assertion(id("links-owned-b"))),
            evidence: evidence(
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
            id: id("links-relation-05"),
            origin: LocalRef::Assertion(id("links-upstream-y")),
            kind: RelationKind::SourceSeeAlso,
            direction: Direction::Forward,
            scope: RelationScope::Assertion,
            target: RelationTarget::Unresolved("missing".into()),
            evidence: evidence(
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
            id: id("links-relation-03"),
            origin: LocalRef::EditorialSense(id("links-owned-sense-ba")),
            kind: RelationKind::EditorialLink,
            direction: Direction::Bidirectional,
            scope: RelationScope::Editorial,
            target: RelationTarget::Local(LocalRef::Assertion(id("links-upstream-y"))),
            evidence: None,
            editorial_revision: Some(id("links-owned-r1")),
        },
        RelationRecord {
            id: id("links-relation-04"),
            origin: LocalRef::Entry(id("links-owned-entry-b")),
            kind: RelationKind::Other("fixture-unknown".into()),
            direction: Direction::Unspecified,
            scope: RelationScope::Entry,
            target: RelationTarget::Unknown,
            evidence: None,
            editorial_revision: None,
        },
        RelationRecord {
            id: id("links-relation-06"),
            origin: LocalRef::Assertion(id("links-upstream-y")),
            kind: RelationKind::EditorialLink,
            direction: Direction::Forward,
            scope: RelationScope::Assertion,
            target: RelationTarget::Local(LocalRef::Assertion(id("links-owned-c"))),
            evidence: None,
            editorial_revision: Some(id("links-owned-r1")),
        },
        RelationRecord {
            id: id("links-relation-07"),
            origin: LocalRef::Assertion(id("links-owned-a")),
            kind: RelationKind::SourceSeeAlso,
            direction: Direction::Forward,
            scope: RelationScope::Assertion,
            target: RelationTarget::Catalog(vlajmaji_lexicon::CatalogTarget {
                source: lu.clone(),
                release: id("links-upstream-release-r1"),
                native_entry_id: Some("N".into()),
                native_assertion_id: None,
                occurrence: None,
            }),
            evidence: evidence(occurrence.clone(), None),
            editorial_revision: None,
        },
        RelationRecord {
            id: id("links-relation-08"),
            origin: LocalRef::Assertion(id("links-owned-a")),
            kind: RelationKind::SourceSeeAlso,
            direction: Direction::Forward,
            scope: RelationScope::Assertion,
            target: RelationTarget::Catalog(vlajmaji_lexicon::CatalogTarget {
                source: lu,
                release: id("links-upstream-release-r1"),
                native_entry_id: Some("absent".into()),
                native_assertion_id: None,
                occurrence: None,
            }),
            evidence: evidence(occurrence, None),
            editorial_revision: None,
        },
    ]);
    doc.metadata.owned_heads.clear();
    Dictionary::from_document(doc).unwrap()
}

fn install_links_view(root: &Path, dictionary: &Dictionary) -> (PathBuf, PathBuf) {
    let owned_root = root.join("owned");
    let genesis = owned_r0();
    let revision_one = owned_r1(&genesis);
    let (mut writer, created) = OwnedJournal::create(&owned_root, &genesis).unwrap();
    assert_eq!(created.head.sequence, 0);
    let committed = writer.commit(&id("links-owned-r0"), &revision_one).unwrap();
    assert_eq!(committed.head.sequence, 1);

    let sealed = seal_release(PackId::LaLojban, PackPayloadKind::Synthetic, dictionary).unwrap();
    let catalogue = root.join("catalogue");
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

    let view = root.join("view");
    WorkbenchView::build_new(
        &view,
        WorkbenchBuildRequest {
            catalogue_root: Some(catalogue),
            owned_root: Some(owned_root.clone()),
            selection: WorkbenchSelection {
                version: 1,
                packs: vec![PackId::LaLojban],
                disabled: Vec::new(),
                owned: Some(WorkbenchOwnedRevision::Revision(id("links-owned-r1"))),
                contexts: Vec::new(),
            },
        },
    )
    .unwrap();
    (view, owned_root)
}

fn assert_links_totals(response: &serde_json::Value) {
    let body = &response["result"]["response"];
    assert_eq!(body["totals"]["records"], 13);
    assert_eq!(body["totals"]["source_groups"], 2);
    assert_eq!(body["totals"]["distinct_evidence"], 11);
    assert_eq!(body["totals"]["associated_assertions"], 6);
    assert_eq!(body["totals"]["visited_assertions"], 6);
    assert_eq!(body["groups"].as_array().unwrap().len(), 2);
    assert_eq!(body["groups"][0]["source"]["id"], "links-owned");
    assert_eq!(body["groups"][1]["source"]["id"], "links-upstream");
}

fn readable_link_steps(text: &str) -> Vec<serde_json::Value> {
    let mut steps = Vec::new();
    for line in text.lines() {
        if let Some(value) = line.strip_prefix("  step: ") {
            steps.push(serde_json::json!({
                "identity": serde_json::from_str::<serde_json::Value>(value).unwrap()
            }));
            continue;
        }
        let Some(step) = steps.last_mut().and_then(serde_json::Value::as_object_mut) else {
            continue;
        };
        let Some((label, value)) = line
            .strip_prefix("    ")
            .and_then(|line| line.split_once(": "))
        else {
            continue;
        };
        let field = match label {
            "depth"
            | "from"
            | "origin_resolution"
            | "target_observations"
            | "target_assertion_ids"
            | "traversal_status" => label,
            "evidence" => "record",
            "link_detail" => "detail_ref",
            _ => continue,
        };
        step.insert(
            field.to_owned(),
            serde_json::from_str(value).unwrap_or_else(|error| {
                panic!("Readable field {label} is not JSON: {error}: {value}")
            }),
        );
    }
    steps
}

fn readable_fields(text: &str) -> serde_json::Map<String, serde_json::Value> {
    text.lines()
        .filter_map(|line| line.split_once(": "))
        .filter_map(|(label, value)| {
            serde_json::from_str(value)
                .ok()
                .map(|value| (label.to_owned(), value))
        })
        .collect()
}

fn readable_group_values(text: &str, label: &str) -> Vec<serde_json::Value> {
    let prefix = format!("{label}: ");
    text.lines()
        .filter_map(|line| line.strip_prefix(&prefix))
        .map(|value| serde_json::from_str(value).unwrap())
        .collect()
}

#[test]
fn test_direct_and_configured_links_detail_cli_full_and_readable() {
    let root = fresh_root();
    let dictionary = fixed_links_dictionary();
    let preinput = dictionary.document();
    assert_eq!(preinput.metadata.relations.len(), 8);
    assert_eq!(preinput.metadata.assertions.len(), 6);
    let (view, owned_root) = install_links_view(&root, &dictionary);
    let prepared_binding_bytes = fs::read(view.join("binding.json")).unwrap();
    let prepared_dictionary_bytes = fs::read(view.join("dictionary/dictionary.sqlite3")).unwrap();
    let catalogue = CollectionCatalogue::open(root.join("catalogue")).unwrap();
    let installed = catalogue
        .installed_artifact_locations(PackId::LaLojban)
        .unwrap()
        .unwrap();
    let mut catalogue_paths = vec![
        installed.release_path,
        installed.seal_path,
        installed.database_path,
    ];
    if let Some(report_path) = installed.report_path {
        catalogue_paths.push(report_path);
    }
    let catalogue_bytes = catalogue_paths
        .iter()
        .map(|path| fs::read(path).unwrap())
        .collect::<Vec<_>>();
    let owned_journal_bytes = fs::read(owned_root.join("owned.sqlite")).unwrap();
    let request_path = root.join("links request ü.json");
    write_json(
        &request_path,
        &serde_json::json!({
            "version": 1,
            "query": {"operation":"links", "spelling":"seed", "depth":2},
            "options": {"sources":[], "alias_scope":"ActiveOnly", "page_size":null, "cursor":null}
        }),
    );

    let direct_full = result_ok(cli(&[
        "workbench",
        "lexical",
        view.to_str().unwrap(),
        "--request",
        request_path.to_str().unwrap(),
        "--format",
        "full",
    ]));
    let direct_compact = result_ok(cli(&[
        "workbench",
        "lexical",
        view.to_str().unwrap(),
        "--request",
        request_path.to_str().unwrap(),
        "--format",
        "compact",
    ]));
    assert_links_totals(&direct_full);
    assert_links_totals(&direct_compact);
    assert_eq!(
        direct_full["result"]["response"]["groups"][0]["hits"]
            .as_array()
            .unwrap()
            .len(),
        7
    );
    assert_eq!(
        direct_full["result"]["response"]["groups"][1]["hits"]
            .as_array()
            .unwrap()
            .len(),
        6
    );
    let full_first = &direct_full["result"]["response"]["groups"][0]["hits"][0];
    assert_eq!(
        full_first["identity"]["evidence"]["relation_id"],
        "links-relation-03"
    );
    assert_eq!(full_first["from"]["assertion"]["id"], "links-owned-a");
    assert_eq!(full_first["target_assertion_ids"][0], "links-upstream-y");

    let direct_readable = cli(&[
        "workbench",
        "lexical",
        view.to_str().unwrap(),
        "--request",
        request_path.to_str().unwrap(),
        "--format",
        "readable",
    ]);
    assert!(direct_readable.status.success());
    let direct_text = String::from_utf8(direct_readable.stdout).unwrap();
    for line in [
        "lexical_operation: \"links\"",
        "  step: ",
        "    depth: ",
        "    evidence: ",
        "    origin_resolution: ",
        "    target_observations: ",
        "    target_assertion_ids: ",
        "    traversal_status: ",
        "    link_detail: ",
    ] {
        assert!(direct_text.contains(line), "missing Readable field {line}");
    }
    assert!(direct_text.contains("links-relation-03"));
    let compact_body = &direct_compact["result"]["response"];
    let readable_header = readable_fields(&direct_text);
    for (label, value) in [
        (
            "workbench_binding",
            direct_compact["binding_sha256"].clone(),
        ),
        ("workbench_selection", direct_compact["selection"].clone()),
        ("lexical_operation", serde_json::json!("links")),
        ("lexical_policy", compact_body["policy"].clone()),
        ("lexical_totals", compact_body["totals"].clone()),
        (
            "lexical_source_totals",
            compact_body["source_totals"].clone(),
        ),
        ("lexical_next_cursor", compact_body["next_cursor"].clone()),
    ] {
        assert_eq!(readable_header.get(label), Some(&value), "Readable {label}");
    }
    let compact_groups = compact_body["groups"].as_array().unwrap();
    for (label, key) in [
        ("source", "source"),
        ("full_count", "full_count"),
        ("returned_count", "returned_count"),
        ("partial", "partial"),
    ] {
        let expected = compact_groups
            .iter()
            .map(|group| group[key].clone())
            .collect::<Vec<_>>();
        assert_eq!(
            readable_group_values(&direct_text, label),
            expected,
            "Readable grouped {label} values"
        );
    }
    assert_eq!(
        compact_groups
            .iter()
            .map(|group| group["hits"].as_array().unwrap().len())
            .collect::<Vec<_>>(),
        readable_group_values(&direct_text, "returned_count")
            .iter()
            .map(|value| value.as_u64().unwrap() as usize)
            .collect::<Vec<_>>()
    );
    let actual_readable_steps = readable_link_steps(&direct_text);
    let expected_readable_steps = compact_groups
        .iter()
        .flat_map(|group| group["hits"].as_array().unwrap())
        .map(|step| {
            serde_json::json!({
                "identity": step["identity"],
                "depth": step["depth"],
                "from": step["from"],
                "record": step["record"],
                "origin_resolution": step["origin_resolution"],
                "target_observations": step["target_observations"],
                "target_assertion_ids": step["target_assertion_ids"],
                "traversal_status": step["traversal_status"],
                "detail_ref": step["detail_ref"],
            })
        })
        .collect::<Vec<_>>();
    assert_eq!(actual_readable_steps, expected_readable_steps);

    let direct_ref =
        direct_compact["result"]["response"]["groups"][0]["hits"][0]["detail_ref"].clone();
    assert!(
        direct_ref.is_object(),
        "direct Links response: {direct_compact:#}"
    );
    let direct_ref_path = root.join("link detail ü.json");
    write_json(&direct_ref_path, &direct_ref);
    let direct_binding = direct_compact["binding_sha256"].as_str().unwrap();
    let direct_detail = result_ok(cli(&[
        "workbench",
        "link-detail",
        view.to_str().unwrap(),
        "--reference",
        direct_ref_path.to_str().unwrap(),
        "--expected-binding",
        direct_binding,
    ]));
    assert_eq!(direct_detail["kind"], "vlajmaji-workbench-link-detail");
    assert_eq!(
        direct_detail["record"]["relation"]["id"],
        "links-relation-03"
    );
    assert_eq!(
        direct_detail["record"]["relation"],
        preinput
            .metadata
            .relations
            .iter()
            .find(|row| row.id == id("links-relation-03"))
            .map(|row| serde_json::to_value(row).unwrap())
            .unwrap()
    );
    let direct_detail_readable = cli(&[
        "workbench",
        "link-detail",
        view.to_str().unwrap(),
        "--reference",
        direct_ref_path.to_str().unwrap(),
        "--expected-binding",
        direct_binding,
        "--format",
        "readable",
    ]);
    assert!(direct_detail_readable.status.success());
    let detail_text = String::from_utf8(direct_detail_readable.stdout).unwrap();
    for label in [
        "link_reference: ",
        "record: ",
        "origin_resolution: ",
        "target_observations: ",
        "link_policy: ",
    ] {
        assert!(
            detail_text.contains(label),
            "missing detail Readable field {label}"
        );
    }
    let detail_fields = readable_fields(&detail_text);
    for (label, value) in [
        ("workbench_binding", direct_detail["binding_sha256"].clone()),
        ("workbench_selection", direct_detail["selection"].clone()),
        ("link_reference", direct_detail["reference"].clone()),
        ("record", direct_detail["record"].clone()),
        (
            "origin_resolution",
            direct_detail["origin_resolution"].clone(),
        ),
        (
            "target_observations",
            direct_detail["target_observations"].clone(),
        ),
        ("link_policy", serde_json::json!("bounded-links-v1")),
    ] {
        assert_eq!(detail_fields.get(label), Some(&value), "Readable {label}");
    }

    // Malformed request and strict detail-key failures stay read-only and use
    // the existing structured CLI error envelope.
    let valid_request_bytes = fs::read(&request_path).unwrap();
    let malformed_request = b"{\"version\":1";
    fs::write(&request_path, malformed_request).unwrap();
    let malformed_query = cli(&[
        "workbench",
        "lexical",
        view.to_str().unwrap(),
        "--request",
        request_path.to_str().unwrap(),
        "--format",
        "full",
    ]);
    assert_error_envelope(&malformed_query, "request_json");
    let malformed_query_error: serde_json::Value =
        serde_json::from_slice(&malformed_query.stderr).unwrap();
    assert_eq!(malformed_query_error["error"]["code"], "request_json");
    assert_eq!(fs::read(&request_path).unwrap(), malformed_request);
    fs::write(&request_path, &valid_request_bytes).unwrap();

    let mut unknown_detail = direct_ref.clone();
    unknown_detail
        .as_object_mut()
        .unwrap()
        .insert("unrecognized".into(), serde_json::json!(true));
    let strict_ref_path = root.join("strict link detail ü.json");
    write_json(&strict_ref_path, &unknown_detail);
    let strict_ref_bytes = fs::read(&strict_ref_path).unwrap();
    let strict_detail = cli(&[
        "workbench",
        "link-detail",
        view.to_str().unwrap(),
        "--reference",
        strict_ref_path.to_str().unwrap(),
        "--expected-binding",
        direct_binding,
    ]);
    assert_error_envelope(&strict_detail, "request_json");
    let strict_detail_error: serde_json::Value =
        serde_json::from_slice(&strict_detail.stderr).unwrap();
    assert_eq!(strict_detail_error["error"]["code"], "request_json");
    assert_eq!(fs::read(&strict_ref_path).unwrap(), strict_ref_bytes);

    let stale_outer_binding = "0".repeat(64);
    let stale_outer = cli(&[
        "workbench",
        "link-detail",
        view.to_str().unwrap(),
        "--reference",
        direct_ref_path.to_str().unwrap(),
        "--expected-binding",
        &stale_outer_binding,
    ]);
    assert_error_envelope(&stale_outer, "stale_workbench_binding");
    let stale_outer_error: serde_json::Value = serde_json::from_slice(&stale_outer.stderr).unwrap();
    assert_eq!(
        stale_outer_error["error"]["code"],
        "stale_workbench_binding"
    );

    let mut stale_detail_ref = direct_ref.clone();
    stale_detail_ref["record_sha256"] = serde_json::json!("0".repeat(64));
    let stale_ref_path = root.join("stale link detail ü.json");
    write_json(&stale_ref_path, &stale_detail_ref);
    let stale_detail = cli(&[
        "workbench",
        "link-detail",
        view.to_str().unwrap(),
        "--reference",
        stale_ref_path.to_str().unwrap(),
        "--expected-binding",
        direct_binding,
    ]);
    assert_error_envelope(&stale_detail, "stale_link_detail");
    let stale_detail_error: serde_json::Value =
        serde_json::from_slice(&stale_detail.stderr).unwrap();
    assert_eq!(stale_detail_error["error"]["code"], "stale_link_detail");

    for (field, value) in [
        ("version", serde_json::json!(2)),
        ("role", serde_json::json!("form")),
        ("record_sha256", serde_json::json!("A".repeat(64))),
    ] {
        let mut malformed = direct_ref.clone();
        malformed[field] = value;
        let malformed_path = root.join(format!("malformed-{field}-link-detail.json"));
        write_json(&malformed_path, &malformed);
        let malformed_bytes = fs::read(&malformed_path).unwrap();
        let malformed_output = cli(&[
            "workbench",
            "link-detail",
            view.to_str().unwrap(),
            "--reference",
            malformed_path.to_str().unwrap(),
            "--expected-binding",
            direct_binding,
        ]);
        assert_error_envelope(&malformed_output, "malformed_link_detail");
        assert_eq!(
            fs::read(&malformed_path).unwrap(),
            malformed_bytes,
            "malformed {field} reference must remain unchanged"
        );
    }
    assert_eq!(
        fs::read(view.join("binding.json")).unwrap(),
        prepared_binding_bytes
    );

    #[cfg(unix)]
    {
        let (saturated, peer) = saturated_stdout();
        let saturated_query = Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
            .args(["workbench", "lexical"])
            .arg(&view)
            .args(["--request"])
            .arg(&request_path)
            .stdout(Stdio::from(saturated))
            .output()
            .unwrap();
        drop(peer);
        assert_error_envelope(&saturated_query, "response_output_io");

        let broken_query = Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
            .args(["workbench", "lexical"])
            .arg(&view)
            .args(["--request"])
            .arg(&request_path)
            .stdout(Stdio::from(broken_pipe_stdout()))
            .output()
            .unwrap();
        assert_eq!(broken_query.status.code(), Some(0));
        assert!(broken_query.stdout.is_empty());
        assert!(broken_query.stderr.is_empty());

        let (saturated, peer) = saturated_stdout();
        let saturated_detail = Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
            .args(["workbench", "link-detail"])
            .arg(&view)
            .args(["--reference"])
            .arg(&direct_ref_path)
            .args(["--expected-binding", direct_binding])
            .stdout(Stdio::from(saturated))
            .output()
            .unwrap();
        drop(peer);
        assert_error_envelope(&saturated_detail, "response_output_io");

        let broken_detail = Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
            .args(["workbench", "link-detail"])
            .arg(&view)
            .args(["--reference"])
            .arg(&direct_ref_path)
            .args(["--expected-binding", direct_binding])
            .stdout(Stdio::from(broken_pipe_stdout()))
            .output()
            .unwrap();
        assert_eq!(broken_detail.status.code(), Some(0));
        assert!(broken_detail.stdout.is_empty());
        assert!(broken_detail.stderr.is_empty());

        assert_eq!(
            fs::read(view.join("binding.json")).unwrap(),
            prepared_binding_bytes
        );
        assert_eq!(
            fs::read(view.join("dictionary/dictionary.sqlite3")).unwrap(),
            prepared_dictionary_bytes
        );
        assert_eq!(
            catalogue_paths
                .iter()
                .map(|path| fs::read(path).unwrap())
                .collect::<Vec<_>>(),
            catalogue_bytes
        );
        assert_eq!(
            fs::read(owned_root.join("owned.sqlite")).unwrap(),
            owned_journal_bytes
        );
    }

    let config = root.join("profiles.json");
    let configured = WorkbenchProfiles::configure(
        &config,
        WorkbenchProfileConfigureRequest {
            name: "links".into(),
            catalogue_root: Some(root.join("catalogue")),
            owned_root: Some(root.join("owned")),
            selection: WorkbenchSelection {
                version: 1,
                packs: vec![PackId::LaLojban],
                disabled: Vec::new(),
                owned: Some(WorkbenchOwnedRevision::Revision(id("links-owned-r1"))),
                contexts: Vec::new(),
            },
            make_default: true,
        },
        None,
    )
    .unwrap();
    WorkbenchProfiles::refresh(&config, None, root.join("configured"), configured.revision)
        .unwrap();
    let config_bytes = fs::read(&config).unwrap();
    let named = result_ok(cli(&[
        "workbench",
        "profiles",
        "lexical",
        "--config",
        config.to_str().unwrap(),
        "--profile",
        "links",
        "--request",
        request_path.to_str().unwrap(),
        "--format",
        "compact",
    ]));
    assert_links_totals(&named);
    assert_eq!(named["configuration"]["profile"], "links");
    let default = result_ok(cli(&[
        "workbench",
        "profiles",
        "lexical",
        "--config",
        config.to_str().unwrap(),
        "--request",
        request_path.to_str().unwrap(),
        "--format",
        "compact",
    ]));
    assert_links_totals(&default);
    assert_eq!(default["configuration"]["profile"], "links");
    let configured_ref = named["result"]["response"]["groups"][0]["hits"][0]["detail_ref"].clone();
    let configured_ref_path = root.join("configured link detail ü.json");
    write_json(&configured_ref_path, &configured_ref);
    let configured_binding = named["binding_sha256"].as_str().unwrap();
    let configured_detail = result_ok(cli(&[
        "workbench",
        "profiles",
        "link-detail",
        "--config",
        config.to_str().unwrap(),
        "--profile",
        "links",
        "--reference",
        configured_ref_path.to_str().unwrap(),
        "--expected-binding",
        configured_binding,
    ]));
    assert_eq!(configured_detail["kind"], "vlajmaji-workbench-link-detail");
    assert_eq!(configured_detail["configuration"]["profile"], "links");
    assert_eq!(
        configured_detail["record"]["relation"]["id"],
        "links-relation-03"
    );
    let configured_readable = cli(&[
        "workbench",
        "profiles",
        "link-detail",
        "--config",
        config.to_str().unwrap(),
        "--profile",
        "links",
        "--reference",
        configured_ref_path.to_str().unwrap(),
        "--expected-binding",
        configured_binding,
        "--format",
        "readable",
    ]);
    assert!(configured_readable.status.success());
    let configured_text = String::from_utf8(configured_readable.stdout).unwrap();
    assert!(configured_text.contains("workbench_configuration: "));
    assert!(configured_text.contains("link_policy: \"bounded-links-v1\""));
    let configured_fields = readable_fields(&configured_text);
    for (label, value) in [
        (
            "workbench_binding",
            configured_detail["binding_sha256"].clone(),
        ),
        (
            "workbench_configuration",
            configured_detail["configuration"].clone(),
        ),
        (
            "workbench_selection",
            configured_detail["selection"].clone(),
        ),
        ("link_reference", configured_detail["reference"].clone()),
        ("record", configured_detail["record"].clone()),
        (
            "origin_resolution",
            configured_detail["origin_resolution"].clone(),
        ),
        (
            "target_observations",
            configured_detail["target_observations"].clone(),
        ),
        ("link_policy", serde_json::json!("bounded-links-v1")),
    ] {
        assert_eq!(
            configured_fields.get(label),
            Some(&value),
            "configured Readable {label}"
        );
    }
    #[cfg(unix)]
    {
        let raw_request_path = root.join(std::ffi::OsString::from_vec(
            b"links-\xff-request.json".to_vec(),
        ));
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&raw_request_path)
        {
            Ok(file) => {
                drop(file);
                fs::write(&raw_request_path, &valid_request_bytes).unwrap();
                eprintln!("raw-byte Links request filename is supported");
                // The OsStr is deliberately non-UTF-8; pass it without a lossy
                // conversion so the operating system receives the raw name.
                let raw_path_query = Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
                    .args(["workbench", "lexical"])
                    .arg(&view)
                    .args(["--request"])
                    .arg(raw_request_path.as_os_str())
                    .args(["--format", "full"])
                    .output()
                    .unwrap();
                assert_links_totals(&result_ok(raw_path_query));
            }
            Err(error) if cfg!(target_os = "macos") && error.raw_os_error() == Some(92) => {
                eprintln!("raw-byte Links request filename rejected with macOS EILSEQ 92");
                let raw_path_query = Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
                    .args(["workbench", "lexical"])
                    .arg(&view)
                    .args(["--request"])
                    .arg(raw_request_path.as_os_str())
                    .args(["--format", "full"])
                    .output()
                    .unwrap();
                assert_error_envelope(&raw_path_query, "workbench_io");
            }
            Err(error) => panic!("raw-byte Links request path failed: {error}"),
        }

        let raw_detail_path = root.join(std::ffi::OsString::from_vec(
            b"links-\xff-detail.json".to_vec(),
        ));
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&raw_detail_path)
        {
            Ok(file) => {
                drop(file);
                write_json(&raw_detail_path, &direct_ref);
                eprintln!("raw-byte Links detail filename is supported");
                let raw_path_detail = Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
                    .args(["workbench", "link-detail"])
                    .arg(&view)
                    .args(["--reference"])
                    .arg(raw_detail_path.as_os_str())
                    .args(["--expected-binding", direct_binding])
                    .output()
                    .unwrap();
                assert_eq!(
                    result_ok(raw_path_detail)["record"]["relation"]["id"],
                    "links-relation-03"
                );
            }
            Err(error) if cfg!(target_os = "macos") && error.raw_os_error() == Some(92) => {
                eprintln!("raw-byte Links detail filename rejected with macOS EILSEQ 92");
                let raw_path_detail = Command::new(env!("CARGO_BIN_EXE_vlajmaji"))
                    .args(["workbench", "link-detail"])
                    .arg(&view)
                    .args(["--reference"])
                    .arg(raw_detail_path.as_os_str())
                    .args(["--expected-binding", direct_binding])
                    .output()
                    .unwrap();
                assert_error_envelope(&raw_path_detail, "workbench_io");
            }
            Err(error) => panic!("raw-byte Links detail path failed: {error}"),
        }

        let deleted_cwd = root.join("deleted-cli-cwd");
        fs::create_dir(&deleted_cwd).unwrap();
        run_links_after_child_deletes_cwd(
            &deleted_cwd,
            DeletedCwdLinksInputs {
                view: &view,
                request: &request_path,
                direct_reference: &direct_ref_path,
                direct_binding,
                config: &config,
                configured_reference: &configured_ref_path,
                configured_binding,
            },
        );
        assert!(!deleted_cwd.exists());
        assert_eq!(fs::read(&request_path).unwrap(), valid_request_bytes);
        assert_eq!(fs::read(&config).unwrap(), config_bytes);
        assert_eq!(
            fs::read(view.join("binding.json")).unwrap(),
            prepared_binding_bytes
        );
        assert_eq!(
            fs::read(view.join("dictionary/dictionary.sqlite3")).unwrap(),
            prepared_dictionary_bytes
        );
        assert_eq!(
            catalogue_paths
                .iter()
                .map(|path| fs::read(path).unwrap())
                .collect::<Vec<_>>(),
            catalogue_bytes
        );
        assert_eq!(
            fs::read(owned_root.join("owned.sqlite")).unwrap(),
            owned_journal_bytes
        );
    }
    fs::remove_dir_all(root).unwrap();
}
