//! Fixed typed-input contract for bounded Links queries.

use std::{
    borrow::Cow,
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

use vlajmaji_lexicon::{
    AbbreviationKind, AliasScope, AssertionRecord, Availability, CaptureMetadata, CaptureRef,
    CollectionRecord, ComponentRecord, ContentNotice, ContextKind, ContextRecord, ContributorId,
    ContributorRecord, Dictionary, DictionaryDocument, DictionaryMetadata, Direction,
    EditorialSenseRecord, Eligibility, EntityId, EntryRecord, LocalRef, NativeEncoding,
    NativeField, OccurrenceAttribution, OccurrenceRef, OwnedHead, OwnedRevisionRecord, PackId,
    PackPayloadKind, Presence, RawCapture, RawOccurrence, RedistributionStatus, RegistryDocument,
    RelationKind, RelationRecord, RelationScope, RelationTarget, ReleaseAccounting, ReleaseRecord,
    SnapshotAttribution, SourceEvidence, SourceId, SourceOrigin, SourceRecord, SourceRegistry,
    TextValue, TimePrecision, seal_release,
};
use vlajmaji_search::{
    ReferenceStatus, WorkbenchLexicalCompactResult, WorkbenchLexicalError,
    WorkbenchLexicalFullResult, WorkbenchLexicalOptions, WorkbenchLexicalQuery,
    WorkbenchLexicalRequest, WorkbenchLinkEvidenceId, WorkbenchLinksCompact, WorkbenchLinksFull,
    WorkbenchQueryError, WorkbenchSearchEngine,
};
use vlajmaji_store::{
    CollectionCatalogue, CollectionInput, CollectionRequest, CollectionSelection, OwnedJournal,
    WorkbenchBuildRequest, WorkbenchOwnedRevision, WorkbenchSelection, WorkbenchView,
};

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

fn owned_r2(previous: &Dictionary) -> Dictionary {
    let mut doc = previous.document();
    let editor = ContributorId::new("links-editor").unwrap();
    let component = id("links-owned-component");
    let cap = doc
        .captures
        .iter()
        .find(|row| row.identity.namespace == "links-owned-capture-r1")
        .unwrap()
        .identity
        .clone();
    doc.metadata.releases.push(release(
        "links-owned-release-r2",
        &component,
        vec![cap],
        vec![
            id("links-owned-a"),
            id("links-owned-b"),
            id("links-owned-c"),
        ],
        true,
    ));
    let rev = id("links-owned-r2");
    doc.metadata.owned_revisions.push(OwnedRevisionRecord {
        id: rev.clone(),
        source: source("links-owned"),
        parent: Some(id("links-owned-r1")),
        editor,
        timestamp: Presence::Missing,
        time_precision: TimePrecision::Unknown,
        assertions: vec![id("links-owned-a"), id("links-owned-b")],
        tombstones: vec![id("links-owned-c")],
        rationale: text("retire c"),
    });
    doc.metadata.owned_heads[0].revision = rev;
    doc.metadata.catalogue_revision = 2;
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

fn metadata_only_remote_dictionary() -> Dictionary {
    let lu = source("links-upstream");
    let (collection, component) = collection_component(
        &lu,
        "links-upstream-remote-collection",
        "links-upstream-remote-component",
    );
    let contributor = ContributorId::new("links-upstream-remote-editor").unwrap();
    let raw = capture(
        "links-upstream-remote-capture",
        "Links remote metadata",
        &["remote | Registered-only remote assertion"],
        vec![vec![
            field("word", "remote"),
            field("definition", "Registered-only remote assertion"),
        ]],
    );
    let occurrence = OccurrenceRef {
        capture: raw.identity.clone(),
        locator: "/0".into(),
    };
    let registry = SourceRegistry::from_document(RegistryDocument {
        sources: vec![source_row("links-upstream", "KU", SourceOrigin::Upstream)],
        contributors: vec![ContributorRecord {
            id: contributor,
            display_name: "Links upstream remote editor".into(),
        }],
        snapshots: vec![SnapshotAttribution {
            snapshot: raw.identity.to_snapshot_identity().unwrap(),
            source: lu.clone(),
            distributor: None,
        }],
        occurrences: vec![OccurrenceAttribution {
            occurrence: occurrence.to_record_identity().unwrap(),
            contributor: Some(ContributorId::new("links-upstream-remote-editor").unwrap()),
            native_entry_id: Some("remote-native".into()),
            native_assertion_id: Some("remote-assertion-native".into()),
        }],
        ..RegistryDocument::default()
    })
    .unwrap();
    let entry_id = id("links-upstream-remote-entry");
    let assertion_id = id("links-upstream-remote-assertion");
    let mut missing_release = release(
        "links-upstream-remote-release",
        &component.id,
        vec![raw.identity.clone()],
        vec![assertion_id.clone()],
        false,
    );
    missing_release.availability = Availability::Missing;
    let mut metadata = DictionaryMetadata::default();
    metadata.collections.push(collection);
    metadata.components.push(component);
    metadata.releases.push(missing_release);
    metadata
        .entries
        .push(entry("links-upstream-remote-entry", &lu, "remote"));
    metadata.assertions.push(assertion(
        "links-upstream-remote-assertion",
        &entry_id,
        occurrence,
        "Registered-only remote assertion",
    ));
    Dictionary::from_document(DictionaryDocument {
        registry,
        captures: vec![raw],
        metadata,
        ..DictionaryDocument::default()
    })
    .unwrap()
}

fn metadata_only_links_dictionary() -> Dictionary {
    let mut doc = fixed_links_dictionary().document();
    let release_component = doc
        .metadata
        .releases
        .iter()
        .find(|row| row.id == id("links-upstream-release-r1"))
        .unwrap()
        .component
        .clone();
    let mut placeholder = release(
        "links-upstream-remote-release",
        &release_component,
        Vec::new(),
        Vec::new(),
        false,
    );
    placeholder.availability = Availability::Missing;
    doc.metadata.releases.push(placeholder);
    let remote_relation = doc
        .metadata
        .relations
        .iter_mut()
        .find(|row| row.id == id("links-relation-08"))
        .unwrap();
    remote_relation.target = RelationTarget::Catalog(vlajmaji_lexicon::CatalogTarget {
        source: source("links-upstream"),
        release: id("links-upstream-remote-release"),
        native_entry_id: Some("remote-native".into()),
        native_assertion_id: None,
        occurrence: None,
    });
    Dictionary::from_document(doc).unwrap()
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

fn request(depth: u8) -> WorkbenchLexicalRequest {
    WorkbenchLexicalRequest {
        version: 1,
        query: WorkbenchLexicalQuery::Links {
            spelling: "seed".into(),
            depth,
        },
        options: WorkbenchLexicalOptions {
            sources: vec!["KO".into(), "KU".into()],
            ..WorkbenchLexicalOptions::default()
        },
    }
}

fn evidence_key(identity: &WorkbenchLinkEvidenceId) -> String {
    match identity {
        WorkbenchLinkEvidenceId::TypedRelation { relation_id } => {
            format!("relation:{relation_id}")
        }
        WorkbenchLinkEvidenceId::SourceClause {
            context_id,
            byte_start,
            byte_end,
            rule_version,
            target_spelling,
        } => {
            format!("clause:{context_id}:{byte_start}:{byte_end}:{rule_version}:{target_spelling}")
        }
    }
}

fn full_rows(response: &WorkbenchLinksFull) -> Vec<String> {
    response
        .groups
        .iter()
        .flat_map(|group| {
            group.hits.iter().map(move |step| {
                let targets = step
                    .target_assertion_ids
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(",");
                format!(
                    "{}|{}|{}|{}|{}|{}",
                    group.source.id,
                    step.depth,
                    evidence_key(&step.identity.evidence),
                    step.identity.orientation,
                    step.from.assertion.id,
                    targets
                )
            })
        })
        .collect()
}

fn compact_rows(response: &WorkbenchLinksCompact) -> Vec<String> {
    response
        .groups
        .iter()
        .flat_map(|group| {
            group.hits.iter().map(move |step| {
                let targets = step
                    .target_assertion_ids
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(",");
                format!(
                    "{}|{}|{}|{}|{}|{}",
                    group.source.id,
                    step.depth,
                    evidence_key(&step.identity.evidence),
                    step.identity.orientation,
                    step.from.assertion.id,
                    targets
                )
            })
        })
        .collect()
}

fn context_presence_counterpart() -> Dictionary {
    let mut document = fixed_links_dictionary().document();
    let original = document
        .metadata
        .contexts
        .iter()
        .find(|row| row.id == id("links-note-x"))
        .unwrap()
        .clone();
    for (context_id, kind, value) in [
        (
            "links-gloss-not-a-note",
            ContextKind::Gloss,
            text("See also {branch}."),
        ),
        ("links-note-null-value", ContextKind::Note, Presence::Null),
        (
            "links-note-missing-value",
            ContextKind::Note,
            Presence::Missing,
        ),
    ] {
        document.metadata.contexts.push(ContextRecord {
            id: id(context_id),
            assertion: original.assertion.clone(),
            kind,
            value,
            translated_from: None,
            evidence: original.evidence.clone(),
        });
    }
    Dictionary::from_document(document).unwrap()
}

fn expected_depth2_rows() -> Vec<String> {
    [
        "links-owned|1|relation:links-relation-03|forward|links-owned-a|links-upstream-y",
        "links-owned|1|relation:links-relation-07|forward|links-owned-a|links-upstream-y",
        "links-owned|1|relation:links-relation-08|forward|links-owned-a|",
        "links-owned|2|relation:links-relation-03|forward|links-owned-b|links-upstream-y",
        "links-owned|2|relation:links-relation-03|reverse|links-upstream-y|links-owned-b,links-owned-a",
        "links-owned|2|relation:links-relation-04|forward|links-owned-b|",
        "links-owned|2|relation:links-relation-06|forward|links-upstream-y|links-owned-c",
        "links-upstream|1|relation:links-relation-01|forward|links-upstream-x|links-owned-b",
        "links-upstream|1|clause:links-note-x:9:17:see-also-braces-v2:branch|forward|links-upstream-x|links-owned-b,links-upstream-y",
        "links-upstream|1|clause:links-note-x:18:26:see-also-braces-v2:branch|forward|links-upstream-x|links-owned-b,links-upstream-y",
        "links-upstream|1|clause:links-note-x:27:36:see-also-braces-v2:missing|forward|links-upstream-x|",
        "links-upstream|2|relation:links-relation-02|reverse|links-owned-b|links-upstream-z",
        "links-upstream|2|relation:links-relation-05|forward|links-upstream-y|",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}

fn assert_link_evidence_matches_input(
    actual: &vlajmaji_search::WorkbenchFormEvidence,
    expected: &SourceEvidence,
    source_id: &SourceId,
    input: &DictionaryDocument,
) {
    assert_eq!(&actual.evidence, expected);
    let registry = input.registry.document();
    let snapshot = registry
        .snapshots
        .iter()
        .find(|row| CaptureRef::from(&row.snapshot) == expected.occurrence.capture);
    if let Some(snapshot) = snapshot {
        assert_eq!(&snapshot.source, source_id);
    }
    let occurrence = registry.occurrences.iter().find(|row| {
        row.occurrence.snapshot() == &expected.occurrence.capture.to_snapshot_identity().unwrap()
            && row.occurrence.locator() == expected.occurrence.locator
    });
    let contributor = occurrence
        .and_then(|row| row.contributor.as_ref())
        .and_then(|id| registry.contributors.iter().find(|row| &row.id == id));
    let distributor = snapshot
        .and_then(|row| row.distributor.as_ref())
        .and_then(|id| registry.distributors.iter().find(|row| &row.id == id));
    assert_eq!(actual.occurrence_attribution, occurrence.cloned());
    assert_eq!(actual.contributor, contributor.cloned());
    assert_eq!(actual.distributor, distributor.cloned());
    if let Some(capture) = input
        .captures
        .iter()
        .find(|row| row.identity == expected.occurrence.capture)
    {
        assert_eq!(actual.status, "Loaded");
        assert_eq!(actual.reason, None);
        assert_eq!(actual.capture.as_ref(), Some(&capture.metadata));
        let raw = capture
            .occurrences
            .iter()
            .find(|row| row.locator == expected.occurrence.locator)
            .unwrap();
        assert_eq!(actual.raw.as_ref(), Some(raw));
        assert_eq!(actual.snapshot_attribution, snapshot.cloned());
    } else {
        assert_eq!(actual.status, "CatalogueOnly");
        assert_eq!(actual.reason.as_deref(), Some("missing_capture"));
        assert_eq!(actual.capture, None);
        assert_eq!(actual.raw, None);
        assert_eq!(actual.snapshot_attribution, snapshot.cloned());
    }
}

#[test]
fn test_links_fixed_typed_constructor_and_bounded_query() {
    let dictionary = fixed_links_dictionary();
    let preinput = dictionary.document();
    assert_eq!(preinput.metadata.entries.len(), 7);
    assert_eq!(preinput.metadata.assertions.len(), 6);
    assert_eq!(preinput.metadata.relations.len(), 8);
    assert_eq!(preinput.metadata.contexts.len(), 1);
    assert_eq!(preinput.metadata.releases.len(), 3);
    let parser_dictionary = context_presence_counterpart();
    let auxiliary = parser_dictionary.document();
    assert_eq!(auxiliary.metadata.contexts.len(), 4);
    assert!(auxiliary.metadata.contexts.iter().any(|row| {
        row.id == id("links-gloss-not-a-note")
            && row.kind == ContextKind::Gloss
            && row.value == text("See also {branch}.")
    }));
    assert!(auxiliary.metadata.contexts.iter().any(|row| {
        row.id == id("links-note-null-value")
            && row.kind == ContextKind::Note
            && row.value == Presence::Null
    }));
    assert!(auxiliary.metadata.contexts.iter().any(|row| {
        row.id == id("links-note-missing-value")
            && row.kind == ContextKind::Note
            && row.value == Presence::Missing
    }));
    let unicode_token = vlajmaji_lexicon::parse_see_also_tokens("See also {ž}.")
        .into_iter()
        .next()
        .unwrap();
    assert_eq!(unicode_token.target_spelling, "ž");
    assert_eq!((unicode_token.byte_start, unicode_token.byte_end), (9, 13));
    let token_json = serde_json::to_string(&unicode_token).unwrap();
    assert_eq!(
        serde_json::from_str::<vlajmaji_lexicon::SeeAlsoToken>(&token_json).unwrap(),
        unicode_token
    );
    assert!(serde_json::from_str::<vlajmaji_lexicon::SeeAlsoToken>(
        r#"{"byte_start":9,"byte_end":13,"target_spelling":"ž","rule_version":"see-also-braces-v2","unexpected":true}"#
    )
    .is_err());
    assert!(serde_json::from_str::<vlajmaji_lexicon::SeeAlsoToken>(
        r#"{"byte_start":9,"byte_start":9,"byte_end":13,"target_spelling":"ž","rule_version":"see-also-braces-v2"}"#
    )
    .is_err());
    assert_eq!(preinput.captures.len(), 2);
    assert!(preinput.metadata.owned_heads.is_empty());
    let empty_entry = preinput
        .metadata
        .entries
        .iter()
        .find(|row| row.id == id("links-upstream-entry-q"))
        .unwrap();
    assert_eq!(empty_entry.source, source("links-upstream"));
    assert_eq!(empty_entry.spelling, Presence::Value("branch".into()));
    assert!(
        !preinput
            .metadata
            .assertions
            .iter()
            .any(|row| row.entry == empty_entry.id)
    );
    let registry = preinput.registry.document();
    for (namespace, source_id) in [
        ("links-owned-capture-r1", "links-owned"),
        ("links-upstream-capture-r1", "links-upstream"),
    ] {
        let capture = preinput
            .captures
            .iter()
            .find(|row| row.identity.namespace == namespace)
            .unwrap();
        assert_eq!(capture.metadata.format_label, "synthetic-links-rows-v1");
        let snapshot = capture.identity.to_snapshot_identity().unwrap();
        assert!(
            registry
                .snapshots
                .iter()
                .any(|row| { row.snapshot == snapshot && row.source == source(source_id) })
        );
        assert_eq!(capture.occurrences.len(), 3);
    }
    let root = fresh_root();
    let owned_root = root.join("owned");
    let genesis = owned_r0();
    let revision_one = owned_r1(&genesis);
    let revision_two = owned_r2(&revision_one);
    let (mut writer, created) = OwnedJournal::create(&owned_root, &genesis).unwrap();
    assert_eq!(created.head.sequence, 0);
    let appended = writer.commit(&id("links-owned-r0"), &revision_one).unwrap();
    assert_eq!(appended.head.sequence, 1);
    let sealed = seal_release(PackId::LaLojban, PackPayloadKind::Synthetic, &dictionary).unwrap();
    let catalogue = root.join("catalogue");
    let catalog_request = CollectionRequest::new(
        vec![CollectionInput::SealedBytes {
            pack_id: PackId::LaLojban,
            release: Cow::Borrowed(&sealed.release),
            seal: Cow::Borrowed(&sealed.seal),
        }],
        CollectionSelection::Packs(vec![PackId::LaLojban]),
        true,
    )
    .unwrap();
    CollectionCatalogue::install(&catalogue, catalog_request).unwrap();
    let view_root = root.join("view-r1");
    WorkbenchView::build_new(
        &view_root,
        WorkbenchBuildRequest {
            catalogue_root: Some(catalogue.clone()),
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
    let mut engine = WorkbenchSearchEngine::open(&view_root).unwrap();

    let parser_sealed = seal_release(
        PackId::LaLojban,
        PackPayloadKind::Synthetic,
        &parser_dictionary,
    )
    .unwrap();
    let parser_catalogue = root.join("parser-counterpart-catalogue");
    let parser_request = CollectionRequest::new(
        vec![CollectionInput::SealedBytes {
            pack_id: PackId::LaLojban,
            release: Cow::Borrowed(&parser_sealed.release),
            seal: Cow::Borrowed(&parser_sealed.seal),
        }],
        CollectionSelection::Packs(vec![PackId::LaLojban]),
        true,
    )
    .unwrap();
    CollectionCatalogue::install(&parser_catalogue, parser_request).unwrap();
    let parser_view = root.join("parser-counterpart-view");
    WorkbenchView::build_new(
        &parser_view,
        WorkbenchBuildRequest {
            catalogue_root: Some(parser_catalogue),
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
    let mut parser_engine = WorkbenchSearchEngine::open(&parser_view).unwrap();
    let WorkbenchLexicalFullResult::Links(parser_result) =
        parser_engine.execute_lexical(&request(2)).unwrap().result
    else {
        panic!("parser-only context counterpart returned another operation")
    };
    assert_eq!(full_rows(&parser_result), expected_depth2_rows());

    let mut empty_request = request(1);
    empty_request.query = WorkbenchLexicalQuery::Links {
        spelling: String::new(),
        depth: 1,
    };
    let WorkbenchLexicalFullResult::Links(empty) =
        engine.execute_lexical(&empty_request).unwrap().result
    else {
        panic!("expected empty-start Links result")
    };
    assert_eq!(empty.totals.records, 0);
    for depth in [0, 3] {
        let invalid = request(depth);
        assert!(engine.execute_lexical(&invalid).is_err());
    }
    let mut zero_sized_page = request(1);
    zero_sized_page.options.page_size = Some(0);
    let WorkbenchLexicalFullResult::Links(full_zero_page) =
        engine.execute_lexical(&zero_sized_page).unwrap().result
    else {
        panic!("zero-sized page returned another Full operation")
    };
    assert_eq!(full_zero_page.totals.source_groups, 2);
    assert_eq!(full_zero_page.totals.records, 7);
    assert_eq!(full_zero_page.totals.distinct_evidence, 7);
    assert_eq!(full_zero_page.totals.associated_assertions, 4);
    assert_eq!(full_zero_page.totals.visited_assertions, 4);
    assert_eq!(full_zero_page.groups, Vec::new());
    assert!(full_zero_page.next_cursor.is_none());
    let WorkbenchLexicalCompactResult::Links(compact_zero_page) = engine
        .execute_lexical_compact(&zero_sized_page)
        .unwrap()
        .result
    else {
        panic!("zero-sized page returned another Compact operation")
    };
    assert_eq!(compact_zero_page.totals.source_groups, 2);
    assert_eq!(compact_zero_page.totals.records, 7);
    assert_eq!(compact_zero_page.totals.distinct_evidence, 7);
    assert_eq!(compact_zero_page.totals.associated_assertions, 4);
    assert_eq!(compact_zero_page.totals.visited_assertions, 4);
    assert_eq!(compact_zero_page.groups, Vec::new());
    assert!(compact_zero_page.next_cursor.is_none());

    let WorkbenchLexicalFullResult::Links(full_depth1) =
        engine.execute_lexical(&request(1)).unwrap().result
    else {
        panic!("expected Links Full result")
    };
    assert_eq!(full_depth1.totals.records, 7);
    assert_eq!(full_depth1.totals.source_groups, 2);
    assert_eq!(full_depth1.totals.distinct_evidence, 7);
    assert_eq!(full_depth1.totals.associated_assertions, 4);
    assert_eq!(full_depth1.totals.visited_assertions, 4);
    let WorkbenchLexicalCompactResult::Links(compact_depth1) =
        engine.execute_lexical_compact(&request(1)).unwrap().result
    else {
        panic!("expected Links Compact result")
    };
    assert_eq!(compact_depth1.totals.records, 7);
    assert_eq!(compact_depth1.totals.distinct_evidence, 7);

    let WorkbenchLexicalFullResult::Links(full_r1) =
        engine.execute_lexical(&request(2)).unwrap().result
    else {
        panic!("expected Links Full result")
    };
    let expected_r1 = expected_depth2_rows();
    assert_eq!(full_rows(&full_r1), expected_r1);
    assert_eq!(full_r1.totals.records, 13);
    assert_eq!(full_r1.totals.source_groups, 2);
    assert_eq!(full_r1.totals.distinct_evidence, 11);
    assert_eq!(full_r1.totals.associated_assertions, 6);
    assert_eq!(full_r1.totals.visited_assertions, 6);
    assert_eq!(
        full_r1
            .source_totals
            .iter()
            .map(|row| (
                row.source.id.as_str(),
                row.records,
                row.distinct_evidence,
                row.associated_assertions
            ))
            .collect::<Vec<_>>(),
        vec![("links-owned", 7, 5, 4), ("links-upstream", 6, 6, 4)]
    );
    let compact_r1_response = engine.execute_lexical_compact(&request(2)).unwrap();
    let compact_r1_binding = compact_r1_response.binding_sha256.clone();
    let WorkbenchLexicalCompactResult::Links(compact_r1) = compact_r1_response.result else {
        panic!("expected Links Compact result")
    };
    assert_eq!(compact_rows(&compact_r1), expected_r1);
    assert_eq!(compact_r1.totals, full_r1.totals);
    assert_eq!(
        compact_r1
            .source_totals
            .iter()
            .map(|row| (
                row.source.id.as_str(),
                row.records,
                row.distinct_evidence,
                row.associated_assertions
            ))
            .collect::<Vec<_>>(),
        vec![("links-owned", 7, 5, 4), ("links-upstream", 6, 6, 4)]
    );

    let mut ku_only = request(2);
    ku_only.options.sources = vec!["KU".into()];
    let ku_expected = [
        "links-upstream|1|relation:links-relation-01|forward|links-upstream-x|",
        "links-upstream|1|clause:links-note-x:9:17:see-also-braces-v2:branch|forward|links-upstream-x|links-upstream-y",
        "links-upstream|1|clause:links-note-x:18:26:see-also-braces-v2:branch|forward|links-upstream-x|links-upstream-y",
        "links-upstream|1|clause:links-note-x:27:36:see-also-braces-v2:missing|forward|links-upstream-x|",
        "links-upstream|2|relation:links-relation-05|forward|links-upstream-y|",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();
    let WorkbenchLexicalFullResult::Links(ku_full) =
        engine.execute_lexical(&ku_only).unwrap().result
    else {
        panic!("expected KU-filtered Full Links result")
    };
    assert_eq!(ku_full.totals.records, 5);
    assert_eq!(ku_full.totals.source_groups, 1);
    assert_eq!(ku_full.totals.distinct_evidence, 5);
    assert_eq!(ku_full.totals.associated_assertions, 2);
    assert_eq!(ku_full.totals.visited_assertions, 2);
    assert_eq!(full_rows(&ku_full), ku_expected);
    let ku_clause = ku_full
        .groups
        .iter()
        .flat_map(|group| &group.hits)
        .find(|step| {
            matches!(
                &step.identity.evidence,
                WorkbenchLinkEvidenceId::SourceClause { context_id, byte_start: 9, .. }
                    if context_id == &id("links-note-x")
            )
        })
        .unwrap();
    assert_eq!(
        ku_clause
            .target_observations
            .iter()
            .map(|target| (
                target.entry.as_ref().unwrap().id.as_str(),
                target.reason.as_deref(),
                target.traversal_status.as_str()
            ))
            .collect::<Vec<_>>(),
        vec![
            (
                "links-owned-entry-b",
                Some("requested_source"),
                "not_traversable"
            ),
            (
                "links-upstream-entry-q",
                Some("no_current_eligible_assertion"),
                "not_traversable"
            ),
            ("links-upstream-entry-y", None, "traversable"),
        ]
    );
    let WorkbenchLexicalCompactResult::Links(ku_compact) =
        engine.execute_lexical_compact(&ku_only).unwrap().result
    else {
        panic!("expected KU-filtered Compact Links result")
    };
    assert_eq!(ku_compact.resolved_sources, vec![source("links-upstream")]);
    assert_eq!(ku_compact.totals, ku_full.totals);
    assert_eq!(compact_rows(&ku_compact), ku_expected);
    let mut ko_only = request(2);
    ko_only.options.sources = vec!["KO".into()];
    let WorkbenchLexicalFullResult::Links(ko_full) =
        engine.execute_lexical(&ko_only).unwrap().result
    else {
        panic!("expected KO-filtered Full Links result")
    };
    assert_eq!(ko_full.totals.records, 3);
    assert_eq!(ko_full.totals.source_groups, 1);
    assert_eq!(ko_full.totals.distinct_evidence, 3);
    assert_eq!(ko_full.totals.associated_assertions, 1);
    assert_eq!(ko_full.totals.visited_assertions, 1);
    let ko_expected = [
        "links-owned|1|relation:links-relation-03|forward|links-owned-a|",
        "links-owned|1|relation:links-relation-07|forward|links-owned-a|",
        "links-owned|1|relation:links-relation-08|forward|links-owned-a|",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();
    assert_eq!(full_rows(&ko_full), ko_expected);
    assert_eq!(
        ko_full
            .source_totals
            .iter()
            .map(|row| (
                row.source.id.as_str(),
                row.records,
                row.distinct_evidence,
                row.associated_assertions
            ))
            .collect::<Vec<_>>(),
        vec![("links-owned", 3, 3, 1)]
    );
    let WorkbenchLexicalCompactResult::Links(ko_compact) =
        engine.execute_lexical_compact(&ko_only).unwrap().result
    else {
        panic!("expected KO-filtered Compact Links result")
    };
    assert_eq!(ko_compact.totals, ko_full.totals);
    assert_eq!(compact_rows(&ko_compact), ko_expected);
    assert_eq!(
        ko_compact
            .source_totals
            .iter()
            .map(|row| (
                row.source.id.as_str(),
                row.records,
                row.distinct_evidence,
                row.associated_assertions
            ))
            .collect::<Vec<_>>(),
        vec![("links-owned", 3, 3, 1)]
    );
    let unfiltered_clause = compact_r1
        .groups
        .iter()
        .flat_map(|group| &group.hits)
        .find(|step| {
            matches!(
                &step.identity.evidence,
                WorkbenchLinkEvidenceId::SourceClause { context_id, byte_start: 9, .. }
                    if context_id == &id("links-note-x")
            )
        })
        .unwrap();
    let filtered_clause = ku_compact
        .groups
        .iter()
        .flat_map(|group| &group.hits)
        .find(|step| step.identity == unfiltered_clause.identity)
        .unwrap();
    assert_eq!(filtered_clause.detail_ref, unfiltered_clause.detail_ref);
    let filtered_detail = engine
        .link_detail(&unfiltered_clause.detail_ref, &compact_r1_binding)
        .unwrap();
    assert_eq!(
        filtered_detail.record.identity,
        unfiltered_clause.identity.evidence
    );
    assert_eq!(
        filtered_detail
            .target_observations
            .iter()
            .map(|target| (
                target.entry.as_ref().unwrap().id.as_str(),
                target.reason.as_deref(),
                target.traversal_status.as_str()
            ))
            .collect::<Vec<_>>(),
        vec![
            ("links-owned-entry-b", None, "traversable"),
            (
                "links-upstream-entry-q",
                Some("no_current_eligible_assertion"),
                "not_traversable"
            ),
            ("links-upstream-entry-y", None, "traversable"),
        ]
    );

    let input_registry = preinput.registry.document();
    for group in &full_r1.groups {
        let group_source = input_registry
            .sources
            .iter()
            .find(|row| row.id == group.source.id)
            .unwrap();
        assert_eq!(&group.source, group_source);
        for step in &group.hits {
            let publisher = input_registry
                .sources
                .iter()
                .find(|row| row.id == step.record.publisher.id)
                .unwrap();
            let origin_source = input_registry
                .sources
                .iter()
                .find(|row| row.id == step.record.origin_source.id)
                .unwrap();
            assert_eq!(&step.record.publisher, publisher);
            assert_eq!(&step.record.origin_source, origin_source);
            let assertion = preinput
                .metadata
                .assertions
                .iter()
                .find(|row| row.id == step.from.assertion.id)
                .unwrap();
            let entry = preinput
                .metadata
                .entries
                .iter()
                .find(|row| row.id == step.from.entry.id)
                .unwrap();
            assert_eq!(&step.from.assertion, assertion);
            assert_eq!(&step.from.entry, entry);
            if let Some(relation) = &step.record.relation {
                let input = preinput
                    .metadata
                    .relations
                    .iter()
                    .find(|row| row.id == relation.id)
                    .unwrap();
                assert_eq!(relation, input);
                if let Some(source_evidence) = &relation.evidence {
                    assert_link_evidence_matches_input(
                        step.record.evidence.as_ref().unwrap(),
                        source_evidence,
                        &step.record.publisher.id,
                        &preinput,
                    );
                }
                if let Some(revision_id) = &relation.editorial_revision {
                    let input_revision = preinput
                        .metadata
                        .owned_revisions
                        .iter()
                        .find(|row| &row.id == revision_id)
                        .unwrap();
                    assert_eq!(
                        step.record.editorial_revision.as_ref(),
                        Some(input_revision)
                    );
                }
            }
            if let Some(context) = &step.record.context {
                let input = preinput
                    .metadata
                    .contexts
                    .iter()
                    .find(|row| row.id == context.id)
                    .unwrap();
                assert_eq!(context, input);
                if context.kind == ContextKind::Note
                    && matches!(context.value, Presence::Value(_))
                    && !vlajmaji_lexicon::parse_see_also_tokens(match &context.value {
                        Presence::Value(value) => value.text.as_str(),
                        _ => unreachable!(),
                    })
                    .is_empty()
                {
                    assert_link_evidence_matches_input(
                        step.record.evidence.as_ref().unwrap(),
                        &context.evidence,
                        &step.record.publisher.id,
                        &preinput,
                    );
                }
            }
        }
    }
    for (relation_id, authority, publisher, origin) in [
        (
            "links-relation-01",
            "source_evidence",
            "links-upstream",
            "links-upstream",
        ),
        (
            "links-relation-03",
            "editorial_revision",
            "links-owned",
            "links-owned",
        ),
        (
            "links-relation-04",
            "unattributed_origin_source",
            "links-owned",
            "links-owned",
        ),
        (
            "links-relation-06",
            "editorial_revision",
            "links-owned",
            "links-upstream",
        ),
    ] {
        let step = full_r1
            .groups
            .iter()
            .flat_map(|group| &group.hits)
            .find(|step| {
                matches!(
                    &step.identity.evidence,
                    WorkbenchLinkEvidenceId::TypedRelation { relation_id: actual }
                        if actual == &id(relation_id)
                )
            })
            .unwrap();
        assert_eq!(step.record.authority, authority);
        assert_eq!(step.record.publisher.id, source(publisher));
        assert_eq!(step.record.origin_source.id, source(origin));
    }
    let source_clause = full_r1
        .groups
        .iter()
        .flat_map(|group| &group.hits)
        .find(|step| {
            matches!(
                &step.identity.evidence,
                WorkbenchLinkEvidenceId::SourceClause { context_id, byte_start: 9, .. }
                    if context_id == &id("links-note-x")
            )
        })
        .unwrap();
    assert_eq!(source_clause.record.authority, "derived_source_clause");
    assert_eq!(source_clause.record.publisher.id, source("links-upstream"));
    assert_eq!(
        source_clause.record.origin_source.id,
        source("links-upstream")
    );
    let compact_clause = compact_r1
        .groups
        .iter()
        .flat_map(|group| &group.hits)
        .find(|step| {
            matches!(
                &step.identity.evidence,
                WorkbenchLinkEvidenceId::SourceClause { context_id, byte_start: 9, .. }
                    if context_id == &id("links-note-x")
            )
        })
        .unwrap();
    let input_context = preinput
        .metadata
        .contexts
        .iter()
        .find(|row| row.id == id("links-note-x"))
        .unwrap();
    assert_eq!(
        compact_clause.record.origin,
        LocalRef::Assertion(input_context.assertion.clone())
    );

    let branch_clause = full_r1
        .groups
        .iter()
        .flat_map(|group| &group.hits)
        .find(|step| {
            matches!(
                &step.identity.evidence,
                WorkbenchLinkEvidenceId::SourceClause { byte_start: 9, .. }
            )
        })
        .unwrap();
    assert_eq!(
        branch_clause
            .target_observations
            .iter()
            .map(|row| row.entry.as_ref().unwrap().id.as_str())
            .collect::<Vec<_>>(),
        vec![
            "links-owned-entry-b",
            "links-upstream-entry-q",
            "links-upstream-entry-y"
        ]
    );

    let relation07_full = full_r1.groups.iter().flat_map(|group| &group.hits).find(|step| {
        matches!(&step.identity.evidence, WorkbenchLinkEvidenceId::TypedRelation { relation_id } if relation_id.as_str() == "links-relation-07")
    }).unwrap();
    let r07_full = &relation07_full.target_observations[0].resolution;
    assert_eq!(r07_full.status, ReferenceStatus::Resolved);
    let r07_payload = r07_full.payload.as_ref().unwrap();
    assert_eq!(
        r07_payload
            .confirmed
            .iter()
            .map(|row| row.assertion.id.as_str())
            .collect::<Vec<_>>(),
        vec!["links-upstream-y"]
    );
    assert_eq!(
        r07_payload
            .unverified
            .iter()
            .map(|row| row.assertion.id.as_str())
            .collect::<Vec<_>>(),
        vec!["links-upstream-z"]
    );
    assert_eq!(
        relation07_full.target_assertion_ids,
        vec![id("links-upstream-y")]
    );
    assert_eq!(relation07_full.target_observations[0].reason, None);
    assert_eq!(
        relation07_full.target_observations[0].traversal_status,
        "traversable"
    );
    let relation08_full = full_r1.groups.iter().flat_map(|group| &group.hits).find(|step| {
        matches!(&step.identity.evidence, WorkbenchLinkEvidenceId::TypedRelation { relation_id } if relation_id.as_str() == "links-relation-08")
    }).unwrap();
    let r08_full = &relation08_full.target_observations[0].resolution;
    assert_eq!(r08_full.status, ReferenceStatus::Unverified);
    assert!(r08_full.payload.as_ref().unwrap().confirmed.is_empty());
    assert_eq!(
        r08_full
            .payload
            .as_ref()
            .unwrap()
            .unverified
            .iter()
            .map(|row| row.assertion.id.as_str())
            .collect::<Vec<_>>(),
        vec!["links-upstream-z"]
    );
    let relation07_compact = compact_r1.groups.iter().flat_map(|group| &group.hits).find(|step| {
        matches!(&step.identity.evidence, WorkbenchLinkEvidenceId::TypedRelation { relation_id } if relation_id.as_str() == "links-relation-07")
    }).unwrap();
    let r07_compact = &relation07_compact.target_observations[0].resolution;
    assert_eq!(r07_compact.status, ReferenceStatus::Unverified);
    assert_eq!(
        r07_compact
            .payload
            .as_ref()
            .unwrap()
            .confirmed
            .iter()
            .map(|row| row.assertion.id.as_str())
            .collect::<Vec<_>>(),
        vec!["links-upstream-y"]
    );
    assert_eq!(
        r07_compact
            .payload
            .as_ref()
            .unwrap()
            .unverified
            .iter()
            .map(|row| row.assertion.id.as_str())
            .collect::<Vec<_>>(),
        vec!["links-upstream-z"]
    );
    assert_eq!(
        relation07_compact.target_assertion_ids,
        vec![id("links-upstream-y")]
    );
    assert_eq!(relation07_compact.target_observations[0].reason, None);
    assert_eq!(
        relation07_compact.target_observations[0].traversal_status,
        "traversable"
    );
    let relation08_compact = compact_r1.groups.iter().flat_map(|group| &group.hits).find(|step| {
        matches!(&step.identity.evidence, WorkbenchLinkEvidenceId::TypedRelation { relation_id } if relation_id.as_str() == "links-relation-08")
    }).unwrap();
    let r08_compact = &relation08_compact.target_observations[0].resolution;
    assert_eq!(r08_compact.status, ReferenceStatus::NoMatch);
    assert!(r08_compact.payload.is_none());

    for (relation_id, expected_status, expected_reason) in [
        ("links-relation-04", ReferenceStatus::Unknown, "unknown"),
        (
            "links-relation-05",
            ReferenceStatus::Unresolved,
            "unresolved",
        ),
    ] {
        let full_step = full_r1
            .groups
            .iter()
            .flat_map(|group| &group.hits)
            .find(|step| {
                matches!(
                    &step.identity.evidence,
                    WorkbenchLinkEvidenceId::TypedRelation { relation_id: actual }
                        if actual == &id(relation_id)
                )
            })
            .unwrap();
        let full_target = &full_step.target_observations[0];
        assert_eq!(full_target.resolution.status, expected_status);
        assert_eq!(full_target.reason.as_deref(), Some(expected_reason));
        assert_eq!(full_target.traversal_status, "not_traversable");

        let compact_step = compact_r1
            .groups
            .iter()
            .flat_map(|group| &group.hits)
            .find(|step| {
                matches!(
                    &step.identity.evidence,
                    WorkbenchLinkEvidenceId::TypedRelation { relation_id: actual }
                        if actual == &id(relation_id)
                )
            })
            .unwrap();
        let compact_target = &compact_step.target_observations[0];
        assert_eq!(compact_target.resolution.status, expected_status);
        assert_eq!(compact_target.reason.as_deref(), Some(expected_reason));
        assert_eq!(compact_target.traversal_status, "not_traversable");
    }

    let mut full_page_request = request(2);
    full_page_request.options.page_size = Some(1);
    let WorkbenchLexicalFullResult::Links(full_page1) =
        engine.execute_lexical(&full_page_request).unwrap().result
    else {
        panic!("expected first Full page")
    };
    assert_eq!(
        full_page1
            .groups
            .iter()
            .map(|group| group.returned_count)
            .sum::<usize>(),
        1
    );
    assert_eq!(
        full_page1
            .groups
            .iter()
            .map(|group| (
                group.source.id.as_str(),
                group.full_count,
                group.returned_count,
                group.partial
            ))
            .collect::<Vec<_>>(),
        vec![("links-owned", 7, 1, true)]
    );
    let full_cursor = full_page1.next_cursor.clone().unwrap();
    let mut full_continuation = request(2);
    full_continuation.options.page_size = Some(3);
    full_continuation.options.cursor = Some(full_cursor.clone());
    let mut stale_cursor = |request: &WorkbenchLexicalRequest| {
        matches!(
            engine.execute_lexical(request),
            Err(WorkbenchQueryError::Lexical(error))
                if matches!(error.as_ref(), WorkbenchLexicalError::StaleCursor)
        )
    };
    let mut changed_query = full_continuation.clone();
    changed_query.query = WorkbenchLexicalQuery::Links {
        spelling: "other".into(),
        depth: 2,
    };
    assert!(stale_cursor(&changed_query));
    let mut changed_selector_order = full_continuation.clone();
    changed_selector_order.options.sources = vec!["KU".into(), "KO".into()];
    assert!(stale_cursor(&changed_selector_order));
    let mut changed_alias_scope = full_continuation.clone();
    changed_alias_scope.options.alias_scope = AliasScope::IncludeRetired;
    assert!(stale_cursor(&changed_alias_scope));
    assert!(matches!(
        engine.execute_lexical_compact(&full_continuation),
        Err(WorkbenchQueryError::Lexical(error))
            if matches!(error.as_ref(), WorkbenchLexicalError::StaleCursor)
    ));
    let mut compact_page_request = request(2);
    compact_page_request.options.page_size = Some(1);
    let WorkbenchLexicalCompactResult::Links(compact_page1) = engine
        .execute_lexical_compact(&compact_page_request)
        .unwrap()
        .result
    else {
        panic!("expected first Compact page")
    };
    let compact_cursor = compact_page1.next_cursor.clone().unwrap();
    assert_eq!(compact_rows(&compact_page1), vec![expected_r1[0].clone()]);

    let r2 = writer.commit(&id("links-owned-r1"), &revision_two).unwrap();
    assert_eq!(r2.head.sequence, 2);
    full_page_request.options.page_size = Some(3);
    full_page_request.options.cursor = Some(full_cursor);
    let WorkbenchLexicalFullResult::Links(full_page2) =
        engine.execute_lexical(&full_page_request).unwrap().result
    else {
        panic!("expected Full continuation after owned append")
    };
    assert_eq!(full_rows(&full_page2), expected_r1[1..4]);
    assert_eq!(full_page2.totals, full_r1.totals);
    assert_eq!(
        full_page2
            .groups
            .iter()
            .map(|group| (
                group.source.id.as_str(),
                group.full_count,
                group.returned_count,
                group.partial
            ))
            .collect::<Vec<_>>(),
        vec![("links-owned", 7, 3, true)]
    );
    let full_next = full_page2.next_cursor.clone().unwrap();
    full_page_request.options.page_size = None;
    full_page_request.options.cursor = Some(full_next);
    let WorkbenchLexicalFullResult::Links(full_page3) =
        engine.execute_lexical(&full_page_request).unwrap().result
    else {
        panic!("expected final Full continuation")
    };
    assert_eq!(full_rows(&full_page3), expected_r1[4..]);
    assert!(full_page3.next_cursor.is_none());
    assert_eq!(full_page3.totals, full_r1.totals);
    assert_eq!(full_page3.source_totals, full_r1.source_totals);
    assert_eq!(
        full_page3
            .groups
            .iter()
            .map(|group| (
                group.source.id.as_str(),
                group.full_count,
                group.returned_count,
                group.partial
            ))
            .collect::<Vec<_>>(),
        vec![("links-owned", 7, 3, true), ("links-upstream", 6, 6, false)]
    );

    compact_page_request.options.page_size = Some(3);
    compact_page_request.options.cursor = Some(compact_cursor);
    let WorkbenchLexicalCompactResult::Links(compact_page2) = engine
        .execute_lexical_compact(&compact_page_request)
        .unwrap()
        .result
    else {
        panic!("expected Compact continuation after owned append")
    };
    assert_eq!(compact_rows(&compact_page2), expected_r1[1..4]);
    assert_eq!(compact_page2.totals, compact_r1.totals);
    assert_eq!(
        compact_page2
            .groups
            .iter()
            .map(|group| (
                group.source.id.as_str(),
                group.full_count,
                group.returned_count,
                group.partial
            ))
            .collect::<Vec<_>>(),
        vec![("links-owned", 7, 3, true)]
    );
    let compact_next = compact_page2.next_cursor.clone().unwrap();
    compact_page_request.options.page_size = None;
    compact_page_request.options.cursor = Some(compact_next);
    let WorkbenchLexicalCompactResult::Links(compact_page3) = engine
        .execute_lexical_compact(&compact_page_request)
        .unwrap()
        .result
    else {
        panic!("expected final Compact continuation")
    };
    assert_eq!(compact_rows(&compact_page3), expected_r1[4..]);
    assert!(compact_page3.next_cursor.is_none());
    assert_eq!(compact_page3.totals, compact_r1.totals);
    assert_eq!(compact_page3.source_totals, compact_r1.source_totals);
    assert_eq!(
        compact_page3
            .groups
            .iter()
            .map(|group| (
                group.source.id.as_str(),
                group.full_count,
                group.returned_count,
                group.partial
            ))
            .collect::<Vec<_>>(),
        vec![("links-owned", 7, 3, true), ("links-upstream", 6, 6, false)]
    );

    let current_root = root.join("view-r2-current");
    WorkbenchView::build_new(
        &current_root,
        WorkbenchBuildRequest {
            catalogue_root: Some(root.join("catalogue")),
            owned_root: Some(owned_root),
            selection: WorkbenchSelection {
                version: 1,
                packs: vec![PackId::LaLojban],
                disabled: Vec::new(),
                owned: Some(WorkbenchOwnedRevision::Head),
                contexts: Vec::new(),
            },
        },
    )
    .unwrap();
    let mut current_engine = WorkbenchSearchEngine::open(&current_root).unwrap();
    let WorkbenchLexicalFullResult::Links(full_r2) =
        current_engine.execute_lexical(&request(2)).unwrap().result
    else {
        panic!("expected current r2 Full result")
    };
    assert_eq!(full_r2.totals.records, 13);
    assert_eq!(full_r2.totals.distinct_evidence, 11);
    assert_eq!(full_r2.totals.associated_assertions, 5);
    assert_eq!(full_r2.totals.visited_assertions, 5);
    let r06_r2 = full_r2.groups.iter().flat_map(|group| &group.hits).find(|step| {
        matches!(&step.identity.evidence, WorkbenchLinkEvidenceId::TypedRelation { relation_id } if relation_id.as_str() == "links-relation-06")
    }).unwrap();
    assert!(r06_r2.target_assertion_ids.is_empty());
    let WorkbenchLexicalCompactResult::Links(compact_r2) = current_engine
        .execute_lexical_compact(&request(2))
        .unwrap()
        .result
    else {
        panic!("expected current r2 Compact result")
    };
    assert_eq!(compact_r2.totals, full_r2.totals);

    let detail_step = compact_r1.groups.iter().flat_map(|group| &group.hits).find(|step| {
        matches!(&step.identity.evidence, WorkbenchLinkEvidenceId::TypedRelation { relation_id } if relation_id.as_str() == "links-relation-01")
    }).unwrap();
    let detail = engine
        .link_detail(&detail_step.detail_ref, &compact_r1_binding)
        .unwrap();
    let input_relation = preinput
        .metadata
        .relations
        .iter()
        .find(|row| row.id == id("links-relation-01"))
        .unwrap();
    assert_eq!(detail.record.relation.as_ref(), Some(input_relation));
    assert_eq!(
        detail.reference.record_sha256,
        detail_step.detail_ref.record_sha256
    );

    let mut wrong_key_binding = detail_step.detail_ref.clone();
    wrong_key_binding.binding_sha256 = "f".repeat(64);
    assert_ne!(wrong_key_binding.binding_sha256, compact_r1_binding);
    assert!(matches!(
        engine.link_detail(&wrong_key_binding, &compact_r1_binding),
        Err(WorkbenchQueryError::Lexical(error))
            if matches!(error.as_ref(), WorkbenchLexicalError::StaleLinkDetail)
    ));

    let mut malformed_key = detail_step.detail_ref.clone();
    malformed_key.record_sha256 = "not-a-sha256".into();
    assert!(matches!(
        engine.link_detail(&malformed_key, &compact_r1_binding),
        Err(WorkbenchQueryError::Lexical(error))
            if matches!(error.as_ref(), WorkbenchLexicalError::MalformedLinkDetail { .. })
    ));

    let mut missing_identity = detail_step.detail_ref.clone();
    missing_identity.identity = WorkbenchLinkEvidenceId::TypedRelation {
        relation_id: id("links-missing-relation"),
    };
    assert!(matches!(
        engine.link_detail(&missing_identity, &compact_r1_binding),
        Err(WorkbenchQueryError::Lexical(error))
            if matches!(error.as_ref(), WorkbenchLexicalError::StaleLinkDetail)
    ));

    let mut wrong_publisher = detail_step.detail_ref.clone();
    wrong_publisher.publisher = source("links-wrong-publisher");
    assert!(matches!(
        engine.link_detail(&wrong_publisher, &compact_r1_binding),
        Err(WorkbenchQueryError::Lexical(error))
            if matches!(error.as_ref(), WorkbenchLexicalError::StaleLinkDetail)
    ));

    // Installed catalogue files are advisory once these engines have opened
    // their prepared view. Preserve the exact bytes while exercising the
    // source-evidence query and detail through those already-open engines.
    let installed = CollectionCatalogue::open(&catalogue)
        .unwrap()
        .installed_artifact_locations(PackId::LaLojban)
        .unwrap()
        .unwrap();
    let mut held = Vec::new();
    for path in [
        installed.release_path,
        installed.seal_path,
        installed.database_path,
    ] {
        let bytes = fs::read(&path).unwrap();
        let file_name = path.file_name().unwrap().to_string_lossy();
        let held_path = path.with_file_name(format!("{file_name}.s05-held"));
        assert!(!held_path.exists());
        fs::rename(&path, &held_path).unwrap();
        held.push((path, held_path, bytes));
    }
    let WorkbenchLexicalFullResult::Links(full_without_copies) =
        engine.execute_lexical(&request(2)).unwrap().result
    else {
        panic!("Full Links query without advisory copies returned another operation")
    };
    assert_eq!(full_rows(&full_without_copies), expected_depth2_rows());
    let WorkbenchLexicalCompactResult::Links(compact_without_copies) =
        engine.execute_lexical_compact(&request(2)).unwrap().result
    else {
        panic!("Compact Links query without advisory copies returned another operation")
    };
    assert_eq!(
        compact_rows(&compact_without_copies),
        expected_depth2_rows()
    );
    let detail_without_copies = engine
        .link_detail(&detail_step.detail_ref, &compact_r1_binding)
        .unwrap();
    assert_eq!(
        detail_without_copies.record.relation.as_ref(),
        Some(input_relation)
    );
    for (path, held_path, bytes) in held.into_iter().rev() {
        fs::rename(held_path, &path).unwrap();
        assert_eq!(fs::read(path).unwrap(), bytes);
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn test_links_metadata_only_remote_target_retains_registration_without_hop() {
    let dictionary = metadata_only_links_dictionary();
    let registered = metadata_only_remote_dictionary();
    let expected_source = dictionary
        .document()
        .registry
        .document()
        .sources
        .into_iter()
        .find(|row| row.id == source("links-upstream"))
        .unwrap();
    let registered_source = registered
        .document()
        .registry
        .document()
        .sources
        .into_iter()
        .find(|row| row.id == source("links-upstream"))
        .unwrap();
    assert_eq!(registered_source, expected_source);
    let remote_doc = registered.document();
    assert_eq!(remote_doc.metadata.assertions.len(), 1);
    assert_eq!(
        remote_doc.metadata.assertions[0].id,
        id("links-upstream-remote-assertion")
    );
    assert_eq!(
        remote_doc.metadata.releases[0].availability,
        Availability::Missing
    );
    assert!(!remote_doc.metadata.releases[0].selected);

    let root = fresh_root();
    let owned_root = root.join("owned");
    let genesis = owned_r0();
    let revision_one = owned_r1(&genesis);
    let (mut writer, _) = OwnedJournal::create(&owned_root, &genesis).unwrap();
    writer.commit(&id("links-owned-r0"), &revision_one).unwrap();

    let catalogue = root.join("catalogue");
    let la = seal_release(PackId::LaLojban, PackPayloadKind::Synthetic, &dictionary).unwrap();
    CollectionCatalogue::install(
        &catalogue,
        CollectionRequest::new(
            vec![CollectionInput::SealedBytes {
                pack_id: PackId::LaLojban,
                release: Cow::Borrowed(&la.release),
                seal: Cow::Borrowed(&la.seal),
            }],
            CollectionSelection::Packs(vec![PackId::LaLojban]),
            true,
        )
        .unwrap(),
    )
    .unwrap();
    let view_root = root.join("view");
    WorkbenchView::build_new(
        &view_root,
        WorkbenchBuildRequest {
            catalogue_root: Some(catalogue.clone()),
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
    let mut before = WorkbenchSearchEngine::open(&view_root).unwrap();
    let baseline = before.execute_lexical(&request(2)).unwrap();
    let WorkbenchLexicalFullResult::Links(baseline) = baseline.result else {
        panic!("expected baseline Full Links result")
    };
    assert_eq!(baseline.totals.records, 13);
    assert_eq!(baseline.totals.source_groups, 2);
    assert_eq!(baseline.totals.distinct_evidence, 11);
    assert_eq!(baseline.totals.associated_assertions, 6);
    assert_eq!(baseline.totals.visited_assertions, 6);
    assert_eq!(full_rows(&baseline), expected_depth2_rows());
    let baseline_totals = baseline.totals;
    drop(before);

    let llg = seal_release(PackId::Llg, PackPayloadKind::Synthetic, &registered).unwrap();
    CollectionCatalogue::register(
        &catalogue,
        CollectionRequest::new(
            vec![CollectionInput::SealedBytes {
                pack_id: PackId::Llg,
                release: Cow::Borrowed(&llg.release),
                seal: Cow::Borrowed(&llg.seal),
            }],
            CollectionSelection::Packs(vec![PackId::Llg]),
            true,
        )
        .unwrap(),
    )
    .unwrap();

    let registered_view_root = root.join("view-with-registered-remote");
    WorkbenchView::build_new(
        &registered_view_root,
        WorkbenchBuildRequest {
            catalogue_root: Some(catalogue.clone()),
            owned_root: Some(owned_root),
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
    let mut engine = WorkbenchSearchEngine::open(&registered_view_root).unwrap();
    let WorkbenchLexicalFullResult::Links(full) =
        engine.execute_lexical(&request(2)).unwrap().result
    else {
        panic!("expected Full Links result")
    };
    assert_eq!(full.totals, baseline_totals);
    assert_eq!(full_rows(&full), expected_depth2_rows());
    let full_step = full
        .groups
        .iter()
        .flat_map(|group| &group.hits)
        .find(|step| {
            matches!(
                &step.identity.evidence,
                WorkbenchLinkEvidenceId::TypedRelation { relation_id }
                    if relation_id == &id("links-relation-08")
            )
        })
        .unwrap();
    let full_target = &full_step.target_observations[0];
    assert_eq!(
        full_target.resolution.status,
        ReferenceStatus::CatalogueOnly
    );
    assert_eq!(
        full_target.resolution.operational_reason.as_deref(),
        Some("not_selected")
    );
    assert!(full_target.resolution.payload.is_none());
    assert_eq!(full_target.traversal_status, "not_traversable");
    assert_eq!(full_step.target_assertion_ids, Vec::<EntityId>::new());

    let WorkbenchLexicalCompactResult::Links(compact) =
        engine.execute_lexical_compact(&request(2)).unwrap().result
    else {
        panic!("expected Compact Links result")
    };
    assert_eq!(compact.totals, baseline_totals);
    assert_eq!(compact_rows(&compact), expected_depth2_rows());
    let compact_step = compact
        .groups
        .iter()
        .flat_map(|group| &group.hits)
        .find(|step| {
            matches!(
                &step.identity.evidence,
                WorkbenchLinkEvidenceId::TypedRelation { relation_id }
                    if relation_id == &id("links-relation-08")
            )
        })
        .unwrap();
    let compact_target = &compact_step.target_observations[0];
    assert_eq!(
        compact_target.resolution.status,
        ReferenceStatus::CatalogueOnly
    );
    assert_eq!(
        compact_target.resolution.operational_reason.as_deref(),
        Some("not_selected")
    );
    assert!(compact_target.resolution.payload.is_none());
    assert_eq!(compact_target.traversal_status, "not_traversable");
    assert_eq!(compact_step.target_assertion_ids, Vec::<EntityId>::new());
}
