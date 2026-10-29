//! Public wire and invariant checks for reviewed owned curation.

use vlajmaji_lexicon::{
    AbbreviationKind, CatalogTarget, ContributorId, Dictionary, DictionaryDocument,
    DictionaryMetadata, Direction, EntityId, EntryRecord, LocalRef, OwnedContextInput,
    OwnedContextPin, OwnedCurationData, OwnedCurationDocument, OwnedCurationKind,
    OwnedCurationRecord, OwnedCurationRevision, OwnedHead, OwnedPinnedTarget, OwnedRevisionRecord,
    OwnedSenseMember, Presence, RegistryDocument, SourceId, SourceOrigin, SourceRecord,
    SourceRegistry, TextValue, TimePrecision,
};

#[path = "../examples/support/dictionary_fixture.rs"]
mod fixture_support;

fn owned_base() -> Dictionary {
    let source = SourceId::new("owner").unwrap();
    let editor = ContributorId::new("editor").unwrap();
    let revision = EntityId::new("r0").unwrap();
    let registry = SourceRegistry::from_document(RegistryDocument {
        sources: vec![SourceRecord {
            id: source.clone(),
            display_name: "Owned dictionary".into(),
            abbreviation: "OWN".into(),
            abbreviation_kind: AbbreviationKind::ProjectAssigned,
            abbreviation_evidence: None,
            origin: SourceOrigin::OwnedDraft,
        }],
        contributors: vec![vlajmaji_lexicon::ContributorRecord {
            id: editor.clone(),
            display_name: "Editor".into(),
        }],
        ..RegistryDocument::default()
    })
    .unwrap();
    let mut metadata = DictionaryMetadata::default();
    metadata.entries.push(EntryRecord {
        id: EntityId::new("owned-entry").unwrap(),
        source: source.clone(),
        spelling: Presence::Value("owned".into()),
        language: Presence::Missing,
        script: Presence::Missing,
        native_word_class: Presence::Missing,
        selmaho: Presence::Missing,
        mapped_pos: Presence::Missing,
    });
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
            text: "genesis".into(),
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

fn fixture_target_at(assertion_index: usize) -> (Dictionary, OwnedContextPin, OwnedPinnedTarget) {
    let mut context_document = fixture_support::complete_fixture().unwrap().document();
    context_document.legacy_archive = None;
    let context = Dictionary::from_document(context_document).unwrap();
    let assertion = &context.metadata().assertions[assertion_index];
    let entry = context
        .metadata()
        .entries
        .iter()
        .find(|row| row.id == assertion.entry)
        .unwrap();
    let release = context
        .metadata()
        .releases
        .iter()
        .find(|row| row.assertions.contains(&assertion.id))
        .unwrap();
    let component = context
        .metadata()
        .components
        .iter()
        .find(|row| row.id == release.component)
        .unwrap();
    let collection = context
        .metadata()
        .collections
        .iter()
        .find(|row| row.id == component.collection)
        .unwrap();
    let occurrence_identity = assertion.occurrence.to_record_identity().unwrap();
    let attribution = context
        .registry()
        .as_document()
        .occurrences
        .iter()
        .find(|row| row.occurrence == occurrence_identity)
        .unwrap();
    let pin =
        OwnedContextPin::from_dictionary(EntityId::new("context-a").unwrap(), &context).unwrap();
    let target = OwnedPinnedTarget {
        context: pin.id.clone(),
        target: CatalogTarget {
            source: collection.source.clone(),
            release: release.id.clone(),
            native_entry_id: attribution.native_entry_id.clone(),
            native_assertion_id: attribution.native_assertion_id.clone(),
            occurrence: Some(assertion.occurrence.clone()),
        },
        entry: entry.id.clone(),
        assertion: assertion.id.clone(),
    };
    (context, pin, target)
}

fn fixture_target() -> (Dictionary, OwnedContextPin, OwnedPinnedTarget) {
    fixture_target_at(0)
}

fn link_record(revision: &EntityId, target: OwnedPinnedTarget) -> OwnedCurationRecord {
    OwnedCurationRecord {
        id: EntityId::new("curation-link").unwrap(),
        revision: revision.clone(),
        editor: ContributorId::new("editor").unwrap(),
        rationale: TextValue {
            text: "reviewed fixture target".into(),
            language: Presence::Missing,
            markup: Presence::Missing,
        },
        kind: OwnedCurationKind::Link {
            origin: LocalRef::Entry(EntityId::new("owned-entry").unwrap()),
            direction: Direction::Forward,
            target,
        },
    }
}

fn curation(
    mut base: Dictionary,
    contexts: Vec<OwnedContextPin>,
    records: Vec<OwnedCurationRecord>,
) -> OwnedCurationData {
    if !records.is_empty() {
        let mut document = base.document();
        let previous = document.metadata.owned_revisions[0].clone();
        let revision = EntityId::new("r1").unwrap();
        document.metadata.catalogue_revision = 1;
        document
            .metadata
            .owned_revisions
            .push(vlajmaji_lexicon::OwnedRevisionRecord {
                id: revision.clone(),
                source: previous.source.clone(),
                parent: Some(previous.id),
                editor: previous.editor,
                timestamp: Presence::Missing,
                time_precision: TimePrecision::Unknown,
                assertions: Vec::new(),
                tombstones: Vec::new(),
                rationale: Presence::Value(TextValue {
                    text: "curation fixture revision".into(),
                    language: Presence::Missing,
                    markup: Presence::Missing,
                }),
            });
        document.metadata.owned_heads[0].revision = revision;
        base = Dictionary::from_document(document).unwrap();
    }
    OwnedCurationData {
        version: 1,
        revisions: if records.is_empty() {
            vec![OwnedCurationRevision {
                revision: base.metadata().owned_revisions[0].id.clone(),
                active: Vec::new(),
                tombstones: Vec::new(),
            }]
        } else {
            vec![
                OwnedCurationRevision {
                    revision: base.metadata().owned_revisions[0].id.clone(),
                    active: Vec::new(),
                    tombstones: Vec::new(),
                },
                OwnedCurationRevision {
                    revision: base.metadata().owned_revisions[1].id.clone(),
                    active: records.iter().map(|record| record.id.clone()).collect(),
                    tombstones: Vec::new(),
                },
            ]
        },
        dictionary: base,
        contexts,
        records,
    }
}

fn empty_curation() -> OwnedCurationData {
    let base = owned_base();
    OwnedCurationData {
        version: 1,
        revisions: vec![OwnedCurationRevision {
            revision: base.metadata().owned_revisions[0].id.clone(),
            active: Vec::new(),
            tombstones: Vec::new(),
        }],
        dictionary: base,
        contexts: Vec::new(),
        records: Vec::new(),
    }
}

#[test]
fn typed_document_deserialization_revalidates_and_preserves_dictionary_bytes() {
    let data = empty_curation();
    let expected_dictionary = data.dictionary.to_json_bytes().unwrap();
    let document = OwnedCurationDocument::from_data(data).unwrap();
    let bytes = document.to_json_bytes().unwrap();
    let restored = OwnedCurationDocument::from_json_bytes(&bytes).unwrap();
    let serde_restored: OwnedCurationDocument = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(restored, document);
    assert_eq!(serde_restored, document);
    assert_eq!(
        restored.as_data().dictionary.to_json_bytes().unwrap(),
        expected_dictionary
    );
    assert_eq!(
        restored.materialize(Vec::new()).unwrap().dictionary,
        restored.as_data().dictionary
    );
}

#[test]
fn materialize_composes_the_exact_supplied_context_before_the_owned_base() {
    let mut context_document = fixture_support::complete_fixture().unwrap().document();
    context_document.legacy_archive = None;
    let mut registry = context_document.registry.into_document();
    let duplicate_lineage = registry.lineage[0].clone();
    registry.lineage.push(duplicate_lineage);
    context_document.registry = SourceRegistry::from_document(registry).unwrap();
    let duplicate_notice = context_document.metadata.code_notices[0].clone();
    context_document
        .metadata
        .code_notices
        .push(duplicate_notice.clone());
    let duplicate_extension = context_document.metadata.extensions[0].clone();
    context_document
        .metadata
        .extensions
        .extend([duplicate_extension.clone(), duplicate_extension.clone()]);
    let context_a = Dictionary::from_document(context_document).unwrap();
    let context_a_bytes = context_a.to_json_bytes().unwrap();
    let mut context_b_document = context_a.document();
    let marker = context_b_document.metadata.extensions[0].clone();
    context_b_document.metadata.extensions.insert(
        0,
        vlajmaji_lexicon::ExtensionRecord {
            name: "second-context-marker".into(),
            ..marker
        },
    );
    let context_b = Dictionary::from_document(context_b_document).unwrap();
    let context_b_bytes = context_b.to_json_bytes().unwrap();
    let pin_a =
        OwnedContextPin::from_dictionary(EntityId::new("context-a").unwrap(), &context_a).unwrap();
    let pin_b =
        OwnedContextPin::from_dictionary(EntityId::new("context-b").unwrap(), &context_b).unwrap();
    let base = owned_base();
    let base_bytes = base.to_json_bytes().unwrap();
    let expected = vlajmaji_lexicon::compose_dictionaries(vec![
        context_a.clone(),
        context_b.clone(),
        base.clone(),
    ])
    .unwrap();
    let document = OwnedCurationDocument::from_data(OwnedCurationData {
        version: 1,
        dictionary: base,
        contexts: vec![pin_a.clone(), pin_b.clone()],
        records: Vec::new(),
        revisions: vec![OwnedCurationRevision {
            revision: EntityId::new("r0").unwrap(),
            active: Vec::new(),
            tombstones: Vec::new(),
        }],
    })
    .unwrap();

    let output = document
        .materialize(vec![
            OwnedContextInput {
                id: pin_b.id,
                dictionary: context_b.clone(),
            },
            OwnedContextInput {
                id: pin_a.id,
                dictionary: context_a.clone(),
            },
        ])
        .unwrap();

    assert_eq!(output.dictionary, expected);
    assert!(output.resolutions.is_empty());
    assert_eq!(context_a.to_json_bytes().unwrap(), context_a_bytes);
    assert_eq!(context_b.to_json_bytes().unwrap(), context_b_bytes);
    assert_eq!(
        document.as_data().dictionary.to_json_bytes().unwrap(),
        base_bytes
    );
    let mut unique_lineage = Vec::new();
    for row in &context_a.registry().as_document().lineage {
        if !unique_lineage.contains(row) {
            unique_lineage.push(row.clone());
        }
    }
    assert_eq!(
        context_a.registry().as_document().lineage.len(),
        unique_lineage.len() + 1
    );
    assert_eq!(
        output.dictionary.registry().as_document().lineage,
        unique_lineage
    );
    assert_eq!(context_a.metadata().code_notices.len(), 2);
    assert_eq!(
        output.dictionary.metadata().code_notices,
        vec![duplicate_notice]
    );
    let input_extension_multiplicity = context_a
        .metadata()
        .extensions
        .iter()
        .filter(|row| {
            row.owner == duplicate_extension.owner && row.name == duplicate_extension.name
        })
        .count();
    let output_extension_multiplicity = output
        .dictionary
        .metadata()
        .extensions
        .iter()
        .filter(|row| {
            row.owner == duplicate_extension.owner && row.name == duplicate_extension.name
        })
        .count();
    assert_eq!(input_extension_multiplicity, 3);
    assert_eq!(output_extension_multiplicity, input_extension_multiplicity);
    let marker_position = output
        .dictionary
        .metadata()
        .extensions
        .iter()
        .position(|row| row.name == "second-context-marker")
        .unwrap();
    assert_eq!(marker_position, context_a.metadata().extensions.len());
}

#[test]
fn unequal_same_id_context_rows_are_a_composition_conflict() {
    let (context_a, pin_a, _) = fixture_target();
    let mut context_b_document = context_a.document();
    let mut registry = context_b_document.registry.into_document();
    registry.sources[0].display_name.push_str(" changed");
    context_b_document.registry = SourceRegistry::from_document(registry).unwrap();
    let context_b = Dictionary::from_document(context_b_document).unwrap();
    let pin_b =
        OwnedContextPin::from_dictionary(EntityId::new("context-b").unwrap(), &context_b).unwrap();
    let document = OwnedCurationDocument::from_data(OwnedCurationData {
        version: 1,
        dictionary: owned_base(),
        contexts: vec![pin_a.clone(), pin_b.clone()],
        records: Vec::new(),
        revisions: vec![OwnedCurationRevision {
            revision: EntityId::new("r0").unwrap(),
            active: Vec::new(),
            tombstones: Vec::new(),
        }],
    })
    .unwrap();

    let error = document
        .materialize(vec![
            OwnedContextInput {
                id: pin_a.id,
                dictionary: context_a,
            },
            OwnedContextInput {
                id: pin_b.id,
                dictionary: context_b,
            },
        ])
        .unwrap_err();
    assert!(matches!(
        error,
        vlajmaji_lexicon::OwnedCurationError::Composition(vlajmaji_lexicon::PackError::Conflict {
            category: "source",
            ..
        })
    ));
}

#[test]
fn from_data_and_manual_deserializer_reject_invalid_history() {
    let mut data = empty_curation();
    data.revisions[0]
        .active
        .push(vlajmaji_lexicon::EntityId::new("missing").unwrap());
    assert!(OwnedCurationDocument::from_data(data.clone()).is_err());
    let invalid = serde_json::to_vec(&data).unwrap();
    assert!(serde_json::from_slice::<OwnedCurationDocument>(&invalid).is_err());

    let (_, pin, target) = fixture_target();
    let mut record = link_record(&EntityId::new("r2").unwrap(), target);
    let mut historical = curation(owned_base(), vec![pin], vec![record.clone()]);
    let mut dictionary = historical.dictionary.document();
    let previous = dictionary.metadata.owned_revisions.last().unwrap().clone();
    let r2 = EntityId::new("r2").unwrap();
    dictionary.metadata.catalogue_revision = 2;
    dictionary
        .metadata
        .owned_revisions
        .push(OwnedRevisionRecord {
            id: r2.clone(),
            source: previous.source,
            parent: Some(previous.id),
            editor: previous.editor,
            timestamp: Presence::Missing,
            time_precision: TimePrecision::Unknown,
            assertions: Vec::new(),
            tombstones: Vec::new(),
            rationale: Presence::Value(TextValue {
                text: "later record revision".into(),
                language: Presence::Missing,
                markup: Presence::Missing,
            }),
        });
    dictionary.metadata.owned_heads[0].revision = r2;
    historical.dictionary = Dictionary::from_document(dictionary).unwrap();
    record.revision = EntityId::new("r2").unwrap();
    historical.records[0] = record.clone();
    historical.revisions.push(OwnedCurationRevision {
        revision: record.revision.clone(),
        active: Vec::new(),
        tombstones: Vec::new(),
    });
    historical.revisions[1].active.push(record.id);
    assert!(OwnedCurationDocument::from_data(historical.clone()).is_err());
    let invalid = serde_json::to_vec(&historical).unwrap();
    assert!(serde_json::from_slice::<OwnedCurationDocument>(&invalid).is_err());
    assert!(OwnedCurationDocument::from_json_bytes(&invalid).is_err());
}

#[test]
fn curation_operation_enums_use_snake_case_externally_tagged_names() {
    let owned =
        OwnedSenseMember::OwnedAssertion(vlajmaji_lexicon::EntityId::new("assertion-a").unwrap());
    let encoded = serde_json::to_value(owned).unwrap();
    assert_eq!(encoded["owned_assertion"], "assertion-a");
    let kind = OwnedCurationKind::Sense {
        members: vec![OwnedSenseMember::OwnedAssertion(
            vlajmaji_lexicon::EntityId::new("assertion-a").unwrap(),
        )],
    };
    assert!(serde_json::to_value(kind).unwrap().get("sense").is_some());
}

#[test]
fn strict_curation_decoders_reject_unknown_link_and_sense_payload_fields() {
    let (_, pin, target) = fixture_target();
    let link_document = OwnedCurationDocument::from_data(curation(
        owned_base(),
        vec![pin.clone()],
        vec![link_record(&EntityId::new("r1").unwrap(), target.clone())],
    ))
    .unwrap();
    let link_bytes = link_document.to_json_bytes().unwrap();
    assert_eq!(
        OwnedCurationDocument::from_json_bytes(&link_bytes).unwrap(),
        link_document
    );
    let mut link_value: serde_json::Value = serde_json::from_slice(&link_bytes).unwrap();
    link_value["records"][0]["kind"]["link"]
        .as_object_mut()
        .unwrap()
        .insert("unexpected".into(), serde_json::Value::Bool(true));
    let link_with_unknown = serde_json::to_vec(&link_value).unwrap();
    assert!(OwnedCurationDocument::from_json_bytes(&link_with_unknown).is_err());
    assert!(serde_json::from_slice::<OwnedCurationDocument>(&link_with_unknown).is_err());

    let (_, _, second_target) = fixture_target_at(1);
    let sense = OwnedCurationRecord {
        id: EntityId::new("curation-sense").unwrap(),
        revision: EntityId::new("r1").unwrap(),
        editor: ContributorId::new("editor").unwrap(),
        rationale: TextValue {
            text: "reviewed member".into(),
            language: Presence::Missing,
            markup: Presence::Missing,
        },
        kind: OwnedCurationKind::Sense {
            members: vec![
                OwnedSenseMember::External(target),
                OwnedSenseMember::External(second_target),
            ],
        },
    };
    let sense_document =
        OwnedCurationDocument::from_data(curation(owned_base(), vec![pin], vec![sense])).unwrap();
    let sense_bytes = sense_document.to_json_bytes().unwrap();
    assert_eq!(
        OwnedCurationDocument::from_json_bytes(&sense_bytes).unwrap(),
        sense_document
    );
    let mut sense_value: serde_json::Value = serde_json::from_slice(&sense_bytes).unwrap();
    sense_value["records"][0]["kind"]["sense"]
        .as_object_mut()
        .unwrap()
        .insert("unexpected".into(), serde_json::Value::Bool(true));
    let sense_with_unknown = serde_json::to_vec(&sense_value).unwrap();
    assert!(OwnedCurationDocument::from_json_bytes(&sense_with_unknown).is_err());
    assert!(serde_json::from_slice::<OwnedCurationDocument>(&sense_with_unknown).is_err());
}

#[test]
fn missing_context_is_visible_and_exact_context_projects_link_and_provenance() {
    let (context, pin, target) = fixture_target();
    let base = owned_base();
    let record = link_record(&EntityId::new("r1").unwrap(), target.clone());
    let document =
        OwnedCurationDocument::from_data(curation(base, vec![pin.clone()], vec![record.clone()]))
            .unwrap();

    let absent = document.materialize(Vec::new()).unwrap();
    assert_eq!(
        absent.resolutions[0].status,
        vlajmaji_lexicon::OwnedCurationStatus::Unavailable
    );
    assert_eq!(
        absent.resolutions[0].reason,
        Some(vlajmaji_lexicon::OwnedCurationReason::MissingContext)
    );
    assert!(
        !absent
            .dictionary
            .metadata()
            .relations
            .iter()
            .any(|row| row.id.as_str() == "curation-link")
    );

    let input = OwnedContextInput {
        id: pin.id.clone(),
        dictionary: context,
    };
    let bytes = input.to_json_bytes().unwrap();
    let restored_input = OwnedContextInput::from_json_bytes(&bytes).unwrap();
    assert_eq!(restored_input, input);
    assert_eq!(restored_input.to_json_bytes().unwrap(), bytes);
    let resolved = document.materialize(vec![restored_input]).unwrap();
    assert_eq!(
        resolved.resolutions[0].status,
        vlajmaji_lexicon::OwnedCurationStatus::Resolved
    );
    assert!(
        resolved
            .dictionary
            .metadata()
            .relations
            .iter()
            .any(|row| row.id.as_str() == "curation-link")
    );
    let extension = resolved
        .dictionary
        .metadata()
        .extensions
        .iter()
        .find(|row| row.owner.as_str() == "curation-link")
        .unwrap();
    assert_eq!(extension.name, "vlajmaji.owned-curation.v1");
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(extension.value.as_str()).unwrap(),
        serde_json::json!({
            "version": 1,
            "record": record,
            "contexts": [pin],
        })
    );

    // A valid metadata-only context retains the declared release and occurrence
    // identity while omitting the raw capture needed to project the Link.
    let mut metadata_only_document = fixture_target().0.document();
    for release in &mut metadata_only_document.metadata.releases {
        release.availability = vlajmaji_lexicon::Availability::Missing;
    }
    metadata_only_document.captures.clear();
    let metadata_only = Dictionary::from_document(metadata_only_document).unwrap();
    let metadata_pin = OwnedContextPin::from_dictionary(pin.id.clone(), &metadata_only).unwrap();
    let metadata_record = link_record(&EntityId::new("r1").unwrap(), target);
    let metadata_document = OwnedCurationDocument::from_data(curation(
        owned_base(),
        vec![metadata_pin.clone()],
        vec![metadata_record],
    ))
    .unwrap();
    let unavailable = metadata_document
        .materialize(vec![OwnedContextInput {
            id: metadata_pin.id,
            dictionary: metadata_only,
        }])
        .unwrap();
    assert_eq!(
        unavailable.resolutions[0].status,
        vlajmaji_lexicon::OwnedCurationStatus::Unresolved
    );
    assert_eq!(
        unavailable.resolutions[0].reason,
        Some(vlajmaji_lexicon::OwnedCurationReason::PayloadUnavailable)
    );
}

#[test]
fn a_three_member_sense_is_not_partially_projected() {
    let (context, pin, valid) = fixture_target();
    let (_, _, valid_third) = fixture_target_at(1);
    let revision = EntityId::new("r1").unwrap();
    let missing = OwnedPinnedTarget {
        assertion: EntityId::new("missing-assertion").unwrap(),
        ..valid.clone()
    };
    let record = OwnedCurationRecord {
        id: EntityId::new("curation-sense").unwrap(),
        revision,
        editor: ContributorId::new("editor").unwrap(),
        rationale: TextValue {
            text: "three member group".into(),
            language: Presence::Missing,
            markup: Presence::Missing,
        },
        kind: OwnedCurationKind::Sense {
            members: vec![
                OwnedSenseMember::External(valid.clone()),
                OwnedSenseMember::External(missing),
                OwnedSenseMember::External(valid_third),
            ],
        },
    };
    let document =
        OwnedCurationDocument::from_data(curation(owned_base(), vec![pin.clone()], vec![record]))
            .unwrap();
    let output = document
        .materialize(vec![OwnedContextInput {
            id: pin.id,
            dictionary: context.clone(),
        }])
        .unwrap();
    assert_eq!(
        output.resolutions[0].status,
        vlajmaji_lexicon::OwnedCurationStatus::Unresolved
    );
    assert_eq!(output.resolutions[0].member, Some(1));
    assert!(
        !output
            .dictionary
            .metadata()
            .editorial_senses
            .iter()
            .any(|row| row.id.as_str() == "curation-sense")
    );
}

#[test]
fn a_complete_three_member_sense_projects_all_members_in_declared_order() {
    let (context, pin, first) = fixture_target_at(0);
    let (_, _, second) = fixture_target_at(1);
    let (_, _, third) = fixture_target_at(2);
    let revision = EntityId::new("r1").unwrap();
    let expected = vec![
        first.assertion.clone(),
        second.assertion.clone(),
        third.assertion.clone(),
    ];
    let record = OwnedCurationRecord {
        id: EntityId::new("curation-complete-sense").unwrap(),
        revision,
        editor: ContributorId::new("editor").unwrap(),
        rationale: TextValue {
            text: "all three reviewed members".into(),
            language: Presence::Missing,
            markup: Presence::Missing,
        },
        kind: OwnedCurationKind::Sense {
            members: vec![
                OwnedSenseMember::External(first),
                OwnedSenseMember::External(second),
                OwnedSenseMember::External(third),
            ],
        },
    };
    let document =
        OwnedCurationDocument::from_data(curation(owned_base(), vec![pin.clone()], vec![record]))
            .unwrap();

    let output = document
        .materialize(vec![OwnedContextInput {
            id: pin.id.clone(),
            dictionary: context.clone(),
        }])
        .unwrap();

    assert_eq!(
        output.resolutions[0].status,
        vlajmaji_lexicon::OwnedCurationStatus::Resolved
    );
    assert!(output.resolutions[0].projected);
    let sense = output
        .dictionary
        .metadata()
        .editorial_senses
        .iter()
        .find(|row| row.id.as_str() == "curation-complete-sense")
        .unwrap();
    assert_eq!(sense.assertions, expected);
    let extension = output
        .dictionary
        .metadata()
        .extensions
        .iter()
        .find(|row| row.owner.as_str() == "curation-complete-sense")
        .unwrap();
    assert_eq!(extension.name, "vlajmaji.owned-curation.v1");
}

#[test]
fn link_to_a_sense_reports_dependency_absence_before_its_missing_context() {
    let (context, pin, first) = fixture_target_at(0);
    let (_, _, second) = fixture_target_at(1);
    let revision = EntityId::new("r1").unwrap();
    let sense_id = EntityId::new("curation-sense-dependency").unwrap();
    let sense = OwnedCurationRecord {
        id: sense_id.clone(),
        revision: revision.clone(),
        editor: ContributorId::new("editor").unwrap(),
        rationale: TextValue {
            text: "two external members".into(),
            language: Presence::Missing,
            markup: Presence::Missing,
        },
        kind: OwnedCurationKind::Sense {
            members: vec![
                OwnedSenseMember::External(first.clone()),
                OwnedSenseMember::External(second),
            ],
        },
    };
    let link = OwnedCurationRecord {
        id: EntityId::new("curation-dependent-link").unwrap(),
        revision: revision.clone(),
        editor: ContributorId::new("editor").unwrap(),
        rationale: TextValue {
            text: "depends on the reviewed sense".into(),
            language: Presence::Missing,
            markup: Presence::Missing,
        },
        kind: OwnedCurationKind::Link {
            origin: LocalRef::EditorialSense(sense_id.clone()),
            direction: Direction::Forward,
            target: first,
        },
    };
    let document = OwnedCurationDocument::from_data(curation(
        owned_base(),
        vec![pin.clone()],
        vec![sense, link],
    ))
    .unwrap();

    let without_context = document.materialize(Vec::new()).unwrap();
    assert_eq!(
        without_context.resolutions[0].reason,
        Some(vlajmaji_lexicon::OwnedCurationReason::MissingContext)
    );
    assert_eq!(
        without_context.resolutions[1].status,
        vlajmaji_lexicon::OwnedCurationStatus::Unresolved
    );
    assert_eq!(
        without_context.resolutions[1].reason,
        Some(vlajmaji_lexicon::OwnedCurationReason::DependencyUnavailable)
    );
    assert!(without_context.dictionary.metadata().relations.is_empty());

    let with_context = document
        .materialize(vec![OwnedContextInput {
            id: pin.id,
            dictionary: context,
        }])
        .unwrap();
    assert!(with_context.resolutions.iter().all(|row| {
        row.status == vlajmaji_lexicon::OwnedCurationStatus::Resolved && row.projected
    }));
    assert!(
        with_context
            .dictionary
            .metadata()
            .editorial_senses
            .iter()
            .any(|row| row.id.as_str() == "curation-sense-dependency")
    );
    assert!(
        with_context
            .dictionary
            .metadata()
            .relations
            .iter()
            .any(|row| row.id.as_str() == "curation-dependent-link")
    );
}

#[test]
fn supplied_context_identity_and_target_checks_are_exact_and_non_retargeting() {
    let (context, pin, target) = fixture_target_at(0);
    let record = link_record(&EntityId::new("r1").unwrap(), target.clone());
    let document =
        OwnedCurationDocument::from_data(curation(owned_base(), vec![pin.clone()], vec![record]))
            .unwrap();

    let unknown_id = OwnedContextInput {
        id: EntityId::new("other-context").unwrap(),
        dictionary: context.clone(),
    };
    assert!(document.materialize(vec![unknown_id]).is_err());
    let duplicated = OwnedContextInput {
        id: pin.id.clone(),
        dictionary: context.clone(),
    };
    assert!(
        document
            .materialize(vec![duplicated.clone(), duplicated])
            .is_err()
    );

    let mut changed_document = context.document();
    changed_document.metadata.releases[0]
        .normalizer_version
        .push_str("-changed");
    let changed = Dictionary::from_document(changed_document).unwrap();
    assert!(matches!(
        document.materialize(vec![OwnedContextInput {
            id: pin.id.clone(),
            dictionary: changed,
        }]),
        Err(vlajmaji_lexicon::OwnedCurationError::ContextMismatch { .. })
    ));

    for wrong_inventory in [
        OwnedContextPin {
            source_ids: vec![SourceId::new("other-source").unwrap()],
            ..pin.clone()
        },
        OwnedContextPin {
            release_ids: vec![EntityId::new("other-release").unwrap()],
            ..pin.clone()
        },
    ] {
        let inventory_document = OwnedCurationDocument::from_data(OwnedCurationData {
            version: 1,
            dictionary: owned_base(),
            contexts: vec![wrong_inventory.clone()],
            records: Vec::new(),
            revisions: vec![OwnedCurationRevision {
                revision: EntityId::new("r0").unwrap(),
                active: Vec::new(),
                tombstones: Vec::new(),
            }],
        })
        .unwrap();
        assert!(matches!(
            inventory_document.materialize(vec![OwnedContextInput {
                id: wrong_inventory.id.clone(),
                dictionary: context.clone(),
            }]),
            Err(vlajmaji_lexicon::OwnedCurationError::ContextMismatch { .. })
        ));
    }

    let (_, _, other_occurrence) = fixture_target_at(1);
    let mut wrong_occurrence = target.clone();
    wrong_occurrence.target.occurrence = other_occurrence.target.occurrence;
    let wrong_occurrence_document = OwnedCurationDocument::from_data(curation(
        owned_base(),
        vec![pin.clone()],
        vec![link_record(&EntityId::new("r1").unwrap(), wrong_occurrence)],
    ))
    .unwrap();
    let result = wrong_occurrence_document
        .materialize(vec![OwnedContextInput {
            id: pin.id.clone(),
            dictionary: context.clone(),
        }])
        .unwrap();
    assert_eq!(
        result.resolutions[0].reason,
        Some(vlajmaji_lexicon::OwnedCurationReason::OccurrenceMismatch)
    );

    let mut missing_entry = target.clone();
    missing_entry.entry = EntityId::new("unknown-entry").unwrap();
    let missing_entry_document = OwnedCurationDocument::from_data(curation(
        owned_base(),
        vec![pin.clone()],
        vec![link_record(&EntityId::new("r1").unwrap(), missing_entry)],
    ))
    .unwrap();
    let missing_entry_result = missing_entry_document
        .materialize(vec![OwnedContextInput {
            id: pin.id.clone(),
            dictionary: context.clone(),
        }])
        .unwrap();
    assert_eq!(
        missing_entry_result.resolutions[0].reason,
        Some(vlajmaji_lexicon::OwnedCurationReason::MissingEntity)
    );

    let mut unknown_assertion = target.clone();
    unknown_assertion.assertion = EntityId::new("unknown-assertion").unwrap();
    let unknown_assertion_document = OwnedCurationDocument::from_data(curation(
        owned_base(),
        vec![pin.clone()],
        vec![link_record(
            &EntityId::new("r1").unwrap(),
            unknown_assertion,
        )],
    ))
    .unwrap();
    let unknown_assertion_result = unknown_assertion_document
        .materialize(vec![OwnedContextInput {
            id: pin.id.clone(),
            dictionary: context.clone(),
        }])
        .unwrap();
    assert_eq!(
        unknown_assertion_result.resolutions[0].reason,
        Some(vlajmaji_lexicon::OwnedCurationReason::AssertionNotInRelease)
    );

    if let Some(other_source) = pin
        .source_ids
        .iter()
        .find(|source| **source != target.target.source)
    {
        let mut wrong_source = target.clone();
        wrong_source.target.source = other_source.clone();
        let wrong_source_document = OwnedCurationDocument::from_data(curation(
            owned_base(),
            vec![pin.clone()],
            vec![link_record(&EntityId::new("r1").unwrap(), wrong_source)],
        ))
        .unwrap();
        let wrong_source_result = wrong_source_document
            .materialize(vec![OwnedContextInput {
                id: pin.id.clone(),
                dictionary: context.clone(),
            }])
            .unwrap();
        assert_eq!(
            wrong_source_result.resolutions[0].reason,
            Some(vlajmaji_lexicon::OwnedCurationReason::WrongSource)
        );
    }

    if let Some(other_release) = pin.release_ids.iter().find(|release| {
        **release != target.target.release
            && !context
                .metadata()
                .releases
                .iter()
                .find(|row| row.id == **release)
                .is_some_and(|row| row.assertions.contains(&target.assertion))
    }) {
        let mut wrong_release = target.clone();
        wrong_release.target.release = other_release.clone();
        let wrong_release_document = OwnedCurationDocument::from_data(curation(
            owned_base(),
            vec![pin.clone()],
            vec![link_record(&EntityId::new("r1").unwrap(), wrong_release)],
        ))
        .unwrap();
        let wrong_release_result = wrong_release_document
            .materialize(vec![OwnedContextInput {
                id: pin.id.clone(),
                dictionary: context.clone(),
            }])
            .unwrap();
        assert_eq!(
            wrong_release_result.resolutions[0].reason,
            Some(vlajmaji_lexicon::OwnedCurationReason::WrongSource)
        );
    }

    let mut wrong_native = target.clone();
    assert!(wrong_native.target.native_entry_id.is_some());
    wrong_native.target.native_entry_id = Some("not-the-captured-entry".into());
    let wrong_native_document = OwnedCurationDocument::from_data(curation(
        owned_base(),
        vec![pin.clone()],
        vec![link_record(&EntityId::new("r1").unwrap(), wrong_native)],
    ))
    .unwrap();
    let result = wrong_native_document
        .materialize(vec![OwnedContextInput {
            id: pin.id,
            dictionary: context.clone(),
        }])
        .unwrap();
    assert_eq!(
        result.resolutions[0].reason,
        Some(vlajmaji_lexicon::OwnedCurationReason::NativeIdentityMismatch)
    );

    let mut unverified = context.document();
    let mut registry = unverified.registry.as_document().clone();
    let identity = target
        .target
        .occurrence
        .as_ref()
        .unwrap()
        .to_record_identity()
        .unwrap();
    let attribution = registry
        .occurrences
        .iter_mut()
        .find(|row| row.occurrence == identity)
        .unwrap();
    attribution.native_entry_id = None;
    attribution.native_assertion_id = None;
    unverified.registry = SourceRegistry::from_document(registry).unwrap();
    let without_native = Dictionary::from_document(unverified).unwrap();
    let no_native_pin = OwnedContextPin::from_dictionary(
        EntityId::new("context-without-native").unwrap(),
        &without_native,
    )
    .unwrap();
    let mut requested_native = target.clone();
    requested_native.context = no_native_pin.id.clone();
    requested_native.target.native_entry_id = Some("requested-but-unrecorded".into());
    requested_native.target.native_assertion_id = None;
    let unknown_native_document = OwnedCurationDocument::from_data(curation(
        owned_base(),
        vec![no_native_pin.clone()],
        vec![link_record(&EntityId::new("r1").unwrap(), requested_native)],
    ))
    .unwrap();
    let result = unknown_native_document
        .materialize(vec![OwnedContextInput {
            id: no_native_pin.id.clone(),
            dictionary: without_native.clone(),
        }])
        .unwrap();
    assert_eq!(
        result.resolutions[0].reason,
        Some(vlajmaji_lexicon::OwnedCurationReason::NativeIdentityUnknown)
    );

    let mut no_native_claim = target;
    no_native_claim.context = no_native_pin.id.clone();
    no_native_claim.target.native_entry_id = None;
    no_native_claim.target.native_assertion_id = None;
    let no_native_claim_document = OwnedCurationDocument::from_data(curation(
        owned_base(),
        vec![no_native_pin.clone()],
        vec![link_record(&EntityId::new("r1").unwrap(), no_native_claim)],
    ))
    .unwrap();
    let result = no_native_claim_document
        .materialize(vec![OwnedContextInput {
            id: no_native_pin.id,
            dictionary: without_native,
        }])
        .unwrap();
    assert_eq!(
        result.resolutions[0].status,
        vlajmaji_lexicon::OwnedCurationStatus::Resolved
    );
}

#[test]
fn inactive_records_remain_visible_without_resolving_their_target() {
    let (_, pin, target) = fixture_target();
    let record = link_record(&EntityId::new("r1").unwrap(), target);
    let record_id = record.id.clone();
    let mut data = curation(owned_base(), vec![pin], vec![record]);
    let mut base = data.dictionary.document();
    let previous = base.metadata.owned_revisions[1].clone();
    let revision = EntityId::new("r2").unwrap();
    base.metadata.catalogue_revision = 2;
    base.metadata.owned_revisions.push(OwnedRevisionRecord {
        id: revision.clone(),
        source: previous.source.clone(),
        parent: Some(previous.id),
        editor: previous.editor,
        timestamp: Presence::Missing,
        time_precision: TimePrecision::Unknown,
        assertions: Vec::new(),
        tombstones: Vec::new(),
        rationale: Presence::Value(TextValue {
            text: "retire the reviewed link".into(),
            language: Presence::Missing,
            markup: Presence::Missing,
        }),
    });
    base.metadata.owned_heads[0].revision = revision.clone();
    data.dictionary = Dictionary::from_document(base).unwrap();
    data.revisions.push(OwnedCurationRevision {
        revision,
        active: Vec::new(),
        tombstones: vec![record_id],
    });
    let document = OwnedCurationDocument::from_data(data).unwrap();
    let output = document.materialize(Vec::new()).unwrap();
    assert_eq!(
        output.resolutions[0].status,
        vlajmaji_lexicon::OwnedCurationStatus::Inactive
    );
    assert_eq!(output.resolutions[0].reason, None);
    assert!(!output.resolutions[0].projected);
}

#[test]
fn strict_byte_codec_rejects_duplicate_keys_and_unsupported_version() {
    let duplicate = br#"{"version":1,"version":1}"#;
    assert!(OwnedCurationDocument::from_json_bytes(duplicate).is_err());

    let mut data = empty_curation();
    data.version = 2;
    let bytes = serde_json::to_vec(&data).unwrap();
    assert!(matches!(
        OwnedCurationDocument::from_json_bytes(&bytes),
        Err(vlajmaji_lexicon::OwnedCurationError::UnsupportedVersion { found: 2 })
    ));

    let duplicate_context = br#"{"id":"context-a","id":"context-b","dictionary":{}}"#;
    assert!(OwnedContextInput::from_json_bytes(duplicate_context).is_err());
}
