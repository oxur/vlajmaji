use std::{env, fs};
use vlajmaji_lexicon::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = env::args().nth(1).expect("explicit fixture JSON path");
    let original = Dictionary::from_json_bytes(&fs::read(path)?)?;
    let mut base = original.document();
    assert_eq!(base.captures.len(), 9);
    assert_eq!(
        base.captures
            .iter()
            .map(|c| c.occurrences.len())
            .sum::<usize>(),
        20
    );
    assert_eq!(base.metadata.assertions.len(), 20);
    assert_eq!(base.metadata.entries.len(), 14);
    assert_eq!(base.registry.document().snapshots.len(), 10);
    let farvi = base
        .metadata
        .assertions
        .iter()
        .filter(|a| {
            base.metadata
                .entries
                .iter()
                .any(|e| e.id == a.entry && e.spelling == Presence::Value("farvi".into()))
        })
        .count();
    assert_eq!(farvi, 7);
    assert_eq!(
        Dictionary::from_json_bytes(&original.to_json_bytes()?)?.document(),
        base
    );
    base.legacy_archive = None; // Isolate new model validation from old-archive subset authentication.
    Dictionary::from_document(base.clone())?;
    let mut failures = vec![];
    let mut n = 0;
    let mut check = |label: &str, ok: bool| {
        n += 1;
        println!("{label}={ok}");
        if !ok {
            failures.push(label.to_string());
        }
    };
    check("digit_first_entity_id", EntityId::new("1:source").is_ok());
    check(
        "unbounded_registry_grammar",
        EntityId::new(format!("a{}", "b".repeat(256))).is_ok(),
    );
    check("invalid_raw_json", RawJson::new("[broken}").is_err());
    check(
        "valid_empty_document",
        Dictionary::from_document(DictionaryDocument::default()).is_ok(),
    );
    let witnesses: serde_json::Value =
        serde_json::from_str(include_str!("../expected-witnesses.json"))?;
    let five = witnesses[0]["definition"].as_str().unwrap();
    let four = witnesses[1]["definition"].as_str().unwrap();
    let raw_defs: Vec<String> = base
        .captures
        .iter()
        .flat_map(|c| &c.occurrences)
        .flat_map(|o| &o.fields)
        .filter(|f| f.name == "definition" && f.encoding == NativeEncoding::Json)
        .filter_map(|f| serde_json::from_str(&f.raw_value).ok())
        .collect();
    check(
        "exact_raw_five_place_witness_three_observations",
        raw_defs.iter().filter(|s| s.as_str() == five).count() == 3,
    );
    check(
        "exact_raw_four_place_witness_one_observation",
        raw_defs.iter().filter(|s| s.as_str() == four).count() == 1,
    );
    for (label, text, score, epoch, date, count) in [
        ("five", five, "1.0", "1479570326", "2016-11-19T15:45:26Z", 3),
        (
            "four",
            four,
            "100002.0",
            "1046655416",
            "2003-03-03T01:36:56Z",
            1,
        ),
    ] {
        let selected: Vec<_> = base
            .metadata
            .assertions
            .iter()
            .filter(|a| matches!(&a.definition,Presence::Value(t) if t.text==text))
            .collect();
        check(
            &format!("exact_normalized_{label}_witness_and_score_dates"),
            selected.len() == count
                && selected.iter().all(|a| {
                    matches!(&a.source_score,Presence::Value(v) if v.as_str()==score)
                        && a.timestamps
                            .iter()
                            .any(|t| t.field == "time" && t.value.as_str() == epoch)
                        && a.timestamps.iter().any(|t| {
                            t.field == "created_at"
                                && serde_json::from_str::<String>(t.value.as_str())
                                    .is_ok_and(|s| s == date)
                        })
                }),
        );
        check(
            &format!("prose_{label}_witness_has_no_supplied_frame"),
            selected
                .iter()
                .all(|a| !base.metadata.frames.iter().any(|f| f.assertion == a.id)),
        );
    }
    check(
        "pinned_definition_ids_are_not_invented_native_senses",
        base.metadata
            .assertions
            .iter()
            .filter(|a| matches!(&a.definition,Presence::Value(t) if t.text==five || t.text==four))
            .all(|a| a.native_sense_id == Presence::Missing),
    );
    let cpana = include_str!("../expected-cpana.txt");
    check(
        "exact_pinned_cpana_raw_line",
        base.captures
            .iter()
            .flat_map(|c| &c.occurrences)
            .filter(|o| o.raw_line.as_deref() == Some(cpana))
            .count()
            == 1,
    );
    let kinds = [
        FormKind::SourceVariant,
        FormKind::SourcePronunciation,
        FormKind::SourceReportedForm,
        FormKind::AssignedRafsi,
        FormKind::ComputedStem,
        FormKind::ComputedComponents,
        FormKind::GeneratedCandidate,
    ];
    check(
        "every_form_kind_has_real_value",
        kinds.iter().all(|k| {
            base.metadata
                .forms
                .iter()
                .any(|f| &f.kind == k && !f.spelling.is_empty())
        }),
    );
    check(
        "assigned_sihe_targets_snime",
        base.metadata.forms.iter().any(|f| {
            f.kind == FormKind::AssignedRafsi
                && f.spelling == "si'e"
                && base
                    .metadata
                    .entries
                    .iter()
                    .any(|e| e.id == f.entry && e.spelling == Presence::Value("snime".into()))
        }),
    );
    check(
        "reported_cpan_targets_cpana",
        base.metadata.forms.iter().any(|f| {
            f.kind == FormKind::SourceReportedForm
                && f.spelling == "cpan"
                && base
                    .metadata
                    .entries
                    .iter()
                    .any(|e| e.id == f.entry && e.spelling == Presence::Value("cpana".into()))
        }),
    );
    check(
        "computed_snim_has_only_computed_provenance",
        base.metadata.forms.iter().any(|f| {
            f.kind == FormKind::ComputedStem
                && f.spelling == "snim"
                && f.evidence.is_none()
                && f.computation.is_some()
        }),
    );
    let registry = base.registry.document();
    check("two_actual_distributors", registry.distributors.len() >= 2);
    let a_d1 = registry
        .snapshots
        .iter()
        .find(|s| s.snapshot.namespace() == "captured:a:d1");
    let a_d2 = registry
        .snapshots
        .iter()
        .find(|s| s.snapshot.namespace() == "captured:a:d2");
    check(
        "same_source_two_bound_distributors",
        a_d1.zip(a_d2).is_some_and(|(a, b)| {
            a.source == b.source
                && a.distributor.is_some()
                && b.distributor.is_some()
                && a.distributor != b.distributor
        }),
    );
    check(
        "native_id_repetition_retains_three_occurrences",
        registry
            .occurrences
            .iter()
            .filter(|a| a.native_assertion_id.as_deref() == Some("69002"))
            .count()
            == 3,
    );
    check(
        "native_id_four_place_333",
        registry
            .occurrences
            .iter()
            .filter(|a| a.native_assertion_id.as_deref() == Some("333"))
            .count()
            == 1,
    );
    check(
        "known_and_unknown_authors",
        registry.occurrences.iter().any(|a| a.contributor.is_some())
            && registry.occurrences.iter().any(|a| a.contributor.is_none()),
    );
    check(
        "active_and_retired_aliases",
        registry
            .aliases
            .iter()
            .any(|a| a.status == AliasStatus::Active)
            && registry
                .aliases
                .iter()
                .any(|a| a.status == AliasStatus::Retired),
    );
    check(
        "known_and_unresolved_lineage",
        registry
            .lineage
            .iter()
            .any(|l| matches!(l.upstream, LineageTarget::Known(_)))
            && registry
                .lineage
                .iter()
                .any(|l| matches!(l.upstream, LineageTarget::Unresolved(_))),
    );
    let missing_release = base
        .metadata
        .releases
        .iter()
        .find(|r| r.availability == Availability::Missing)
        .unwrap();
    let missing_component = base
        .metadata
        .components
        .iter()
        .find(|c| c.id == missing_release.component)
        .unwrap();
    let missing_source = &base
        .metadata
        .collections
        .iter()
        .find(|c| c.id == missing_component.collection)
        .unwrap()
        .source;
    check(
        "distinct_catalogue_only_missing_source",
        registry.sources.len() == 4
            && base.captures.iter().all(|c| {
                registry
                    .snapshots
                    .iter()
                    .find(|s| CaptureRef::from(&s.snapshot) == c.identity)
                    .is_some_and(|s| &s.source != missing_source)
            })
            && missing_release.accounting.input_occurrences.is_none()
            && missing_release.accounting.normalized_occurrences.is_none()
            && missing_release
                .accounting
                .query_excluded_occurrences
                .is_none(),
    );
    let mut reject = |label: &str, edit: &dyn Fn(&mut DictionaryDocument)| {
        let mut d = base.clone();
        edit(&mut d);
        let typed = std::panic::catch_unwind(|| Dictionary::from_document(d.clone()))
            .is_ok_and(|r| r.is_err());
        let bytes = serde_json::to_vec(&d).unwrap();
        let direct = std::panic::catch_unwind(|| serde_json::from_slice::<Dictionary>(&bytes))
            .is_ok_and(|r| r.is_err());
        let helper = std::panic::catch_unwind(|| Dictionary::from_json_bytes(&bytes))
            .is_ok_and(|r| r.is_err());
        check(label, typed && direct && helper);
    };
    reject("unsupported_version", &|d| d.version = 999);
    reject("capture_hash_mismatch", &|d| d.captures[0].bytes.push(0));
    reject("duplicate_capture", &|d| {
        d.captures.push(d.captures[0].clone())
    });
    reject("duplicate_locator", &|d| {
        let row = d.captures[0].occurrences[0].clone();
        d.captures[0].occurrences.push(row);
    });
    reject("wrong_role_context_assertion", &|d| {
        d.metadata.contexts[0].assertion = d.metadata.entries[0].id.clone()
    });
    reject("wrong_role_translation", &|d| {
        d.metadata.contexts[0].translated_from = Some(d.metadata.entries[0].id.clone())
    });
    reject("loaded_phantom_assertion", &|d| {
        d.metadata.assertions[0].occurrence.locator = "/definitely-not-an-occurrence".into()
    });
    reject("available_missing_payload", &|d| {
        let r = d
            .metadata
            .releases
            .iter_mut()
            .find(|r| r.availability == Availability::Missing)
            .unwrap();
        r.availability = Availability::Available;
        r.accounting.input_occurrences = Some(0);
        r.accounting.normalized_occurrences = Some(0);
        r.accounting.query_excluded_occurrences = Some(0);
    });
    reject("wrong_available_accounting", &|d| {
        let r = d
            .metadata
            .releases
            .iter_mut()
            .find(|r| r.availability == Availability::Available)
            .unwrap();
        r.accounting.input_occurrences = Some(r.accounting.input_occurrences.unwrap() + 1);
    });
    reject("duplicate_release_membership", &|d| {
        let r = d
            .metadata
            .releases
            .iter_mut()
            .find(|r| !r.assertions.is_empty())
            .unwrap();
        r.assertions.push(r.assertions[0].clone());
    });
    reject("context_evidence_wrong_occurrence", &|d| {
        let c = &d.metadata.contexts[0];
        let a = d
            .metadata
            .assertions
            .iter()
            .find(|a| a.id != c.assertion)
            .unwrap()
            .occurrence
            .clone();
        d.metadata.contexts[0].evidence.occurrence = a;
    });
    reject("etymology_evidence_wrong_occurrence", &|d| {
        let e = &d.metadata.etymologies[0];
        let a = d
            .metadata
            .assertions
            .iter()
            .find(|a| a.id != e.assertion)
            .unwrap()
            .occurrence
            .clone();
        d.metadata.etymologies[0].evidence.occurrence = a;
    });
    reject("source_computed_crossover", &|d| {
        let id = d.metadata.computations[0].id.clone();
        let f = d
            .metadata
            .forms
            .iter_mut()
            .find(|f| f.evidence.is_some())
            .unwrap();
        f.computation = Some(id);
    });
    reject("invalid_evidence_range", &|d| {
        let e = &mut d.metadata.contexts[0].evidence;
        e.field_index = Some(0);
        e.offset = Some(TextRange {
            start: 0,
            end: usize::MAX,
        });
    });
    reject("owned_revision_cycle", &|d| {
        let a = d.metadata.owned_revisions[0].id.clone();
        let b = d.metadata.owned_revisions[1].id.clone();
        d.metadata.owned_revisions[0].parent = Some(b);
        d.metadata.owned_revisions[1].parent = Some(a);
    });
    reject("owned_future_tombstone", &|d| {
        let future = d.metadata.owned_revisions[1].assertions[0].clone();
        d.metadata.owned_revisions[0].tombstones = vec![future];
    });
    reject("wrong_role_annotation_target", &|d| {
        d.metadata.annotations[0].target = LocalRef::Entry(d.metadata.computations[0].id.clone());
    });
    reject("phantom_registry_attribution", &|d| {
        let mut r = d.registry.document();
        let mut a = r
            .occurrences
            .first()
            .cloned()
            .unwrap_or_else(|| OccurrenceAttribution {
                occurrence: d.metadata.assertions[0]
                    .occurrence
                    .to_record_identity()
                    .unwrap(),
                contributor: None,
                native_entry_id: None,
                native_assertion_id: None,
            });
        a.occurrence = OccurrenceRef {
            capture: CaptureRef::from(a.occurrence.snapshot()),
            locator: "/phantom-registry-only".into(),
        }
        .to_record_identity()
        .unwrap();
        r.occurrences.push(a);
        d.registry = SourceRegistry::from_document(r).unwrap();
    });
    reject("global_cross_role_duplicate_id", &|d| {
        d.metadata.computations[0].id = d.metadata.entries[0].id.clone()
    });
    reject("wrong_role_computation_input", &|d| {
        let id = d.metadata.computations[0].id.clone();
        d.metadata.computations[0]
            .inputs
            .push(InputRef::Entity(LocalRef::Entry(id)));
    });
    reject("wrong_role_form_component", &|d| {
        let id = d.metadata.entries[0].id.clone();
        let f = d
            .metadata
            .forms
            .iter_mut()
            .find(|f| !f.components.is_empty())
            .unwrap();
        f.components[0].target = Some(LocalRef::Assertion(id));
    });
    reject("source_form_evidence_wrong_source", &|d| {
        let fi = d
            .metadata
            .forms
            .iter()
            .position(|f| f.evidence.is_some())
            .unwrap();
        let source = &d
            .metadata
            .entries
            .iter()
            .find(|e| e.id == d.metadata.forms[fi].entry)
            .unwrap()
            .source;
        let a = d
            .metadata
            .assertions
            .iter()
            .find(|a| {
                d.metadata
                    .entries
                    .iter()
                    .any(|e| e.id == a.entry && &e.source != source)
            })
            .unwrap()
            .occurrence
            .clone();
        d.metadata.forms[fi].evidence = Some(SourceEvidence {
            occurrence: a,
            field_index: None,
            citation: Presence::Missing,
            offset: None,
        });
    });
    reject("unknown_owned_editor", &|d| {
        d.metadata.owned_revisions[0].editor =
            ContributorId::new("cdc:unregistered-editor").unwrap()
    });
    reject("unknown_editorial_editor", &|d| {
        d.metadata.editorial_senses[0].editor =
            ContributorId::new("cdc:unregistered-editor").unwrap()
    });
    reject("unknown_release_distributor", &|d| {
        d.metadata.releases[0].distributor =
            Some(DistributorId::new("cdc:unregistered-distributor").unwrap())
    });
    reject("unknown_collection_source", &|d| {
        d.metadata.collections[0].source = SourceId::new("cdc:unregistered-source").unwrap()
    });
    reject("missing_release_wrong_role_assertion", &|d| {
        let id = d.metadata.computations[0].id.clone();
        d.metadata
            .releases
            .iter_mut()
            .find(|r| r.availability == Availability::Missing)
            .unwrap()
            .assertions
            .push(id);
    });
    reject("orphan_catalogue_binding", &|d| {
        let mut r = d.registry.document();
        r.snapshots.push(SnapshotAttribution {
            snapshot: CaptureRef {
                namespace: "cdc:orphan".into(),
                sha256: "0".repeat(64),
            }
            .to_snapshot_identity()
            .unwrap(),
            source: r.sources[0].id.clone(),
            distributor: None,
        });
        d.registry = SourceRegistry::from_document(r).unwrap();
    });
    reject(
        "indirect_missing_owned_parent_is_typed_error_not_panic",
        &|d| {
            let r2 = d.metadata.owned_revisions[1].id.clone();
            d.metadata.owned_revisions[0].parent = Some(r2);
            d.metadata.owned_revisions[1].parent =
                Some(EntityId::new("cdc:missing-parent").unwrap());
        },
    );
    reject("duplicate_owned_head", &|d| {
        let head = d.metadata.owned_heads[0].clone();
        d.metadata.owned_heads.push(head);
    });
    reject("lexical_assertion_requires_present_spelling", &|d| {
        d.metadata.entries[0].spelling = Presence::Missing
    });
    reject("unreferenced_collection_requires_known_source", &|d| {
        let mut c = d.metadata.collections[0].clone();
        c.id = EntityId::new("cdc:orphan-collection").unwrap();
        c.source = SourceId::new("cdc:unknown-source").unwrap();
        d.metadata.collections.push(c);
    });
    reject("unreferenced_entry_requires_known_source", &|d| {
        let mut e = d.metadata.entries[0].clone();
        e.id = EntityId::new("cdc:orphan-entry").unwrap();
        e.source = SourceId::new("cdc:unknown-source").unwrap();
        d.metadata.entries.push(e);
    });
    reject("etymology_component_role_validates", &|d| {
        let id = d.metadata.entries[0].id.clone();
        d.metadata.etymologies[0].components.push(OrderedComponent {
            order: 1,
            spelling: "fixture".into(),
            target: Some(LocalRef::Assertion(id)),
        });
    });
    reject("computation_occurrence_phantom", &|d| {
        let mut o = d.metadata.assertions[0].occurrence.clone();
        o.locator = "/phantom-computation-input".into();
        d.metadata.computations[0]
            .inputs
            .push(InputRef::Occurrence(o));
    });
    reject("loaded_phantom_catalogue_target_even_when_disabled", &|d| {
        let r = d
            .metadata
            .releases
            .iter_mut()
            .find(|r| r.id.as_str() == "release-a")
            .unwrap();
        r.availability = Availability::Disabled;
        let rid = r.id.clone();
        let cap = r.captures[0].clone();
        let source = d
            .metadata
            .entries
            .iter()
            .find(|e| e.source.as_str() == "source-a")
            .unwrap()
            .source
            .clone();
        let rel = d
            .metadata
            .relations
            .iter_mut()
            .find(|r| matches!(r.target, RelationTarget::Catalog(_)))
            .unwrap();
        rel.target = RelationTarget::Catalog(CatalogTarget {
            source,
            release: rid,
            native_entry_id: None,
            native_assertion_id: None,
            occurrence: Some(OccurrenceRef {
                capture: cap,
                locator: "/phantom-disabled-target".into(),
            }),
        });
    });
    drop(reject);
    for field in [
        "source_name",
        "format_label",
        "locator",
        "revision",
        "captured_at",
        "time_precision",
        "notice_locator",
    ] {
        let mut d = original.document();
        let c = &mut d.captures[0].metadata;
        match field {
            "source_name" => c.source_name.push_str("-changed"),
            "format_label" => c.format_label.push_str("-changed"),
            "locator" => c.locator.push_str("-changed"),
            "revision" => c.revision = Presence::Value("changed".into()),
            "captured_at" => c.captured_at = Presence::Value("changed".into()),
            "time_precision" => {
                c.time_precision = if c.time_precision == TimePrecision::Second {
                    TimePrecision::Day
                } else {
                    TimePrecision::Second
                }
            }
            "notice_locator" => c.notice_locator = Presence::Value("changed".into()),
            _ => unreachable!(),
        }
        // Some fixtures already have Unknown; use a genuinely changed precision value.
        if d == original.document() {
            continue;
        }
        let bytes = serde_json::to_vec(&d)?;
        check(
            &format!("legacy_subset_metadata_{field}"),
            Dictionary::from_document(d).is_err()
                && Dictionary::from_json_bytes(&bytes).is_err()
                && serde_json::from_slice::<Dictionary>(&bytes).is_err(),
        );
    }
    let mut empty_release = base.clone();
    let mut release = empty_release
        .metadata
        .releases
        .iter()
        .find(|r| r.availability == Availability::Available)
        .unwrap()
        .clone();
    release.id = EntityId::new("cdc:empty-available-release")?;
    release.captures.clear();
    release.assertions.clear();
    release.diagnostics.clear();
    release.selected = false;
    release.accounting.input_occurrences = Some(0);
    release.accounting.normalized_occurrences = Some(0);
    release.accounting.query_excluded_occurrences = Some(0);
    empty_release.metadata.releases.push(release);
    check(
        "genuinely_empty_available",
        Dictionary::from_document(empty_release).is_ok(),
    );
    let mut typed = base.clone();
    typed
        .metadata
        .entries
        .push(typed.metadata.entries[0].clone());
    check(
        "duplicate_entity_has_named_category",
        matches!(
            Dictionary::from_document(typed),
            Err(DictionaryError::DuplicateEntity(_))
        ),
    );
    let mut typed = base.clone();
    typed.metadata.contexts[0].assertion = EntityId::new("cdc:absent-assertion")?;
    check(
        "reference_has_named_category",
        matches!(
            Dictionary::from_document(typed),
            Err(DictionaryError::MissingReference(_))
        ),
    );
    let mut typed = base.clone();
    typed
        .metadata
        .releases
        .iter_mut()
        .find(|r| r.availability == Availability::Available)
        .unwrap()
        .accounting
        .input_occurrences = Some(999);
    check(
        "accounting_has_named_category",
        matches!(
            Dictionary::from_document(typed),
            Err(DictionaryError::InvalidAccounting(_))
        ),
    );
    let mut typed = base.clone();
    let r = typed
        .metadata
        .releases
        .iter_mut()
        .find(|r| r.availability == Availability::Missing)
        .unwrap();
    r.availability = Availability::Available;
    r.accounting.input_occurrences = Some(0);
    r.accounting.normalized_occurrences = Some(0);
    r.accounting.query_excluded_occurrences = Some(0);
    check(
        "availability_has_named_category",
        matches!(
            Dictionary::from_document(typed),
            Err(DictionaryError::InvalidAvailability(_))
        ),
    );
    let mut typed = base.clone();
    let ev = typed.metadata.contexts[0].evidence.clone();
    typed
        .metadata
        .forms
        .iter_mut()
        .find(|f| f.kind == FormKind::ComputedStem)
        .unwrap()
        .evidence = Some(ev);
    check(
        "provenance_has_named_category",
        matches!(
            Dictionary::from_document(typed),
            Err(DictionaryError::InvalidProvenance(_))
        ),
    );
    let mut typed = base.clone();
    typed.metadata.owned_revisions[0].parent = Some(typed.metadata.owned_revisions[1].id.clone());
    check(
        "history_has_named_category",
        matches!(
            Dictionary::from_document(typed),
            Err(DictionaryError::InvalidHistory(_))
        ),
    );
    let mut wire = serde_json::to_value(&base)?;
    wire["captures"][0]["identity"]["unexpected"] = serde_json::json!(true);
    let bytes = serde_json::to_vec(&wire)?;
    check(
        "strict_nested_wire",
        serde_json::from_slice::<Dictionary>(&bytes).is_err()
            && Dictionary::from_json_bytes(&bytes).is_err(),
    );
    let mut eligibility_wire = serde_json::to_value(&base)?;
    let excluded_index = base
        .metadata
        .assertions
        .iter()
        .position(|a| matches!(a.eligibility, Eligibility::Excluded { .. }))
        .unwrap();
    eligibility_wire["metadata"]["assertions"][excluded_index]["eligibility"]["Excluded"]["unexpected"] =
        serde_json::json!(true);
    let bytes = serde_json::to_vec(&eligibility_wire)?;
    check(
        "strict_struct_enum_variant_wire",
        serde_json::from_slice::<Dictionary>(&bytes).is_err()
            && Dictionary::from_json_bytes(&bytes).is_err(),
    );
    // A known unavailable projection may retain metadata/evidence without invented raw rows.
    let mut absent = base.clone();
    let release = absent
        .metadata
        .releases
        .iter()
        .find(|r| r.availability == Availability::Missing)
        .unwrap()
        .clone();
    let component = absent
        .metadata
        .components
        .iter()
        .find(|x| x.id == release.component)
        .unwrap();
    let source = absent
        .metadata
        .collections
        .iter()
        .find(|x| x.id == component.collection)
        .unwrap()
        .source
        .clone();
    let mut entry = absent.metadata.entries[0].clone();
    entry.id = EntityId::new("cdc:missing-entry")?;
    entry.source = source;
    let mut assertion = absent.metadata.assertions[0].clone();
    assertion.id = EntityId::new("cdc:missing-assertion")?;
    assertion.entry = entry.id.clone();
    assertion.occurrence = OccurrenceRef {
        capture: release.captures[0].clone(),
        locator: "/retained-catalogue-projection".into(),
    };
    let mut context = absent.metadata.contexts[0].clone();
    context.id = EntityId::new("cdc:missing-context")?;
    context.assertion = assertion.id.clone();
    context.translated_from = None;
    context.evidence = SourceEvidence {
        occurrence: assertion.occurrence.clone(),
        field_index: Some(99),
        citation: Presence::Value("supplied catalogue evidence; unavailable bounds".into()),
        offset: Some(TextRange { start: 1, end: 2 }),
    };
    absent
        .metadata
        .releases
        .iter_mut()
        .find(|r| r.id == release.id)
        .unwrap()
        .assertions
        .push(assertion.id.clone());
    absent.metadata.entries.push(entry);
    absent.metadata.assertions.push(assertion);
    absent.metadata.contexts.push(context);
    let good_absent = Dictionary::from_document(absent.clone()).is_ok();
    let absent_bytes = serde_json::to_vec(&absent)?;
    check(
        "known_missing_projection_and_evidence",
        good_absent
            && serde_json::from_slice::<Dictionary>(&absent_bytes).is_ok()
            && Dictionary::from_json_bytes(&absent_bytes).is_ok(),
    );
    absent.metadata.contexts.last_mut().unwrap().evidence.offset =
        Some(TextRange { start: 3, end: 2 });
    check(
        "unavailable_range_still_ordered",
        Dictionary::from_document(absent).is_err(),
    );
    let mut repeated = base.clone();
    let field = repeated.captures[0].occurrences[0].fields[0].clone();
    let field_name = field.name.clone();
    let before = repeated.captures[0].occurrences[0]
        .fields_named(&field_name)
        .len();
    repeated.captures[0].occurrences[0].fields.push(field);
    let repeated_ok = Dictionary::from_document(repeated.clone())
        .and_then(|d| Dictionary::from_json_bytes(&d.to_json_bytes()?))
        .is_ok_and(|d| {
            d.document() == repeated
                && d.captures()[0].occurrences[0]
                    .fields_named(&field_name)
                    .len()
                    == before + 1
        });
    check("ordered_repeated_native_names_survive", repeated_ok);
    let mut unicode = base.clone();
    let opaque = unicode
        .metadata
        .assertions
        .iter()
        .find(|a| matches!(a.eligibility, Eligibility::Excluded { .. }))
        .unwrap()
        .clone();
    let row = unicode
        .captures
        .iter_mut()
        .find(|c| c.identity == opaque.occurrence.capture)
        .unwrap()
        .occurrences
        .iter_mut()
        .find(|o| o.locator == opaque.occurrence.locator)
        .unwrap();
    let index = row.fields.len();
    row.fields.push(NativeField {
        name: "unicode-evidence".into(),
        encoding: NativeEncoding::Json,
        raw_value: r#""\u00e9""#.into(),
    });
    let mut context = unicode.metadata.contexts[0].clone();
    context.id = EntityId::new("cdc:decoded-unicode-range")?;
    context.assertion = opaque.id;
    context.translated_from = None;
    context.evidence = SourceEvidence {
        occurrence: opaque.occurrence,
        field_index: Some(index),
        citation: Presence::Missing,
        offset: Some(TextRange { start: 0, end: 2 }),
    };
    unicode.metadata.contexts.push(context);
    check(
        "decoded_utf8_range_two_bytes",
        Dictionary::from_document(unicode.clone()).is_ok(),
    );
    unicode
        .metadata
        .contexts
        .last_mut()
        .unwrap()
        .evidence
        .offset = Some(TextRange { start: 0, end: 1 });
    check(
        "decoded_utf8_split_boundary_rejects",
        Dictionary::from_document(unicode).is_err(),
    );
    let mut empty_lexicon = Lexicon::new();
    empty_lexicon.import_source(
        SourceMetadata::new(
            "",
            "",
            SourceFormat::LaLojbanJson,
            "",
            "",
            Some("".into()),
            TimePrecision::Unknown,
        )
        .with_notice_locator(""),
        b"[]",
        None,
    )?;
    let archive_v2 = empty_lexicon.to_archive_bytes()?;
    let identity = empty_lexicon.snapshots()[0].clone();
    let mut registry_doc = RegistryDocument::default();
    registry_doc
        .sources
        .push(base.registry.document().sources[0].clone());
    let legacy_source = registry_doc.sources[0].id.clone();
    registry_doc.snapshots.push(SnapshotAttribution {
        snapshot: identity.clone(),
        source: legacy_source.clone(),
        distributor: None,
    });
    let legacy_registry = SourceRegistry::from_document(registry_doc)?;
    let mut metadata = DictionaryMetadata::default();
    let col_id = EntityId::new("cdc:legacy-empty-collection")?;
    metadata.collections.push(CollectionRecord {
        id: col_id.clone(),
        source: legacy_source,
        display_name: "explicit empty legacy fixture".into(),
        description: Presence::Missing,
    });
    let comp_id = EntityId::new("cdc:legacy-empty-component")?;
    metadata.components.push(ComponentRecord {
        id: comp_id.clone(),
        collection: col_id,
        native_name: "empty".into(),
        languages: vec![],
    });
    let mut release = base
        .metadata
        .releases
        .iter()
        .find(|r| r.availability == Availability::Available)
        .unwrap()
        .clone();
    release.id = EntityId::new("cdc:legacy-empty-release")?;
    release.component = comp_id;
    release.distributor = None;
    release.captures = vec![CaptureRef::from(&identity)];
    release.assertions.clear();
    release.diagnostics.clear();
    release.accounting.input_occurrences = Some(0);
    release.accounting.normalized_occurrences = Some(0);
    release.accounting.query_excluded_occurrences = Some(0);
    metadata.releases.push(release);
    let mut v1: serde_json::Value = serde_json::from_slice(&archive_v2)?;
    v1["version"] = serde_json::json!(1);
    let archive_v1 = serde_json::to_vec(&v1)?;
    for (label, archive) in [("v1", archive_v1), ("v2", archive_v2)] {
        Lexicon::from_archive_bytes(&archive)?;
        let adapted =
            Dictionary::from_legacy_archive(&archive, legacy_registry.clone(), metadata.clone());
        check(
            &format!("legacy_{label}_all_empty_metadata_preserved"),
            adapted.is_ok_and(|d| {
                d.document().legacy_archive.as_deref() == Some(archive.as_slice())
                    && d.captures()[0].identity.namespace.is_empty()
                    && d.captures()[0].metadata.source_name.is_empty()
                    && d.captures()[0].metadata.locator.is_empty()
                    && d.captures()[0].metadata.revision == Presence::Value("".into())
                    && d.captures()[0].metadata.captured_at == Presence::Value("".into())
                    && d.captures()[0].metadata.notice_locator == Presence::Value("".into())
            }),
        );
    }
    let mut strict_old = Lexicon::new();
    check(
        "opaque_rejected_by_unchanged_old_parser",
        strict_old
            .import_source(
                SourceMetadata::new(
                    "cdc:old-opaque",
                    "opaque",
                    SourceFormat::LaLojbanJson,
                    "memory:opaque",
                    "",
                    None,
                    TimePrecision::Unknown,
                ),
                b"opaque-source-row\0retained",
                None,
            )
            .is_err(),
    );
    println!("independent_controls={n}; failures={}", failures.len());
    if !failures.is_empty() {
        return Err(format!("failed controls: {failures:?}").into());
    }
    Ok(())
}
