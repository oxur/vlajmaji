//! Public S02 dictionary model and raw-preservation contract tests.
use vlajmaji_lexicon::{
    Availability, Dictionary, DictionaryDocument, DictionaryError, Eligibility, EntityId, FormKind,
    Presence, RawJson,
};
#[path = "../examples/support/dictionary_fixture.rs"]
mod fixture_support;

#[test]
fn empty_document_is_valid_and_serde_revalidates() {
    let value = Dictionary::default();
    let bytes = value.to_json_bytes().unwrap();
    assert_eq!(
        Dictionary::from_json_bytes(&bytes).unwrap().document(),
        value.document()
    );
    assert!(serde_json::from_str::<Dictionary>(r#"{"version":2,"legacy_archive":null,"registry":{"version":1,"sources":[],"contributors":[],"distributors":[],"aliases":[],"lineage":[],"snapshots":[],"occurrences":[]},"captures":[],"metadata":{"catalogue_revision":0,"code_notices":[],"collections":[],"components":[],"releases":[],"entries":[],"assertions":[],"contexts":[],"frames":[],"forms":[],"etymologies":[],"relations":[],"owned_revisions":[],"owned_heads":[],"editorial_senses":[],"computations":[],"annotations":[],"extensions":[]}}"#).is_err());
}
#[test]
fn raw_values_ids_and_duplicate_envelope_keys_are_checked() {
    assert!(EntityId::new("Bad").is_err());
    assert!(RawJson::new(r#"{"x":1,"x":2}"#).is_ok());
    assert!(RawJson::new("not-json").is_err());
    assert!(matches!(
        Dictionary::from_json_bytes(br#"{"version":1,"version":1}"#),
        Err(DictionaryError::InvalidJson(_))
    ));
}
#[test]
fn complete_public_fixture_roundtrips_all_source_and_metadata_rows() {
    let d = fixture_support::complete_fixture().unwrap();
    assert_eq!(d.captures().len(), 9);
    assert_eq!(
        d.captures()
            .iter()
            .map(|c| c.occurrences.len())
            .sum::<usize>(),
        20
    );
    assert_eq!(d.metadata().entries.len(), 14);
    assert_eq!(d.metadata().assertions.len(), 20);
    let farvi = d
        .metadata()
        .assertions
        .iter()
        .filter(|a| {
            d.metadata()
                .entries
                .iter()
                .any(|e| e.id == a.entry && e.spelling == Presence::Value("farvi".into()))
        })
        .collect::<Vec<_>>();
    assert_eq!(farvi.len(), 7);
    let five_place = "$x_1$ (object) develops/evolves towards/into $x_2$ (state) from $x_3$ (state) for reason/because of $x_4$ (fact, event) through stages $x_5$";
    let four_place =
        "$x_{1}$ develops/evolves towards/into $x_{2}$ from $x_{3}$ through stages $x_{4}$.";
    let a_d1 = d
        .captures()
        .iter()
        .find(|c| c.identity.namespace == "captured:a:d1")
        .unwrap();
    let a_d2 = d
        .captures()
        .iter()
        .find(|c| c.identity.namespace == "captured:a:d2")
        .unwrap();
    assert_eq!(a_d1.occurrences.len(), 3);
    assert_eq!(a_d2.occurrences.len(), 1);
    let five_rows = farvi
        .iter()
        .filter(|a| {
            a.occurrence.capture.namespace == "captured:a:d1"
                && matches!(a.occurrence.locator.as_str(), "/0" | "/1")
        })
        .collect::<Vec<_>>();
    assert_eq!(five_rows.len(), 2);
    assert!(five_rows.iter().all(|a| matches!(
        &a.definition,
        Presence::Value(t) if t.text == five_place
    )));
    assert!(five_rows.iter().all(|a| matches!(
        &a.source_score,
        Presence::Value(v) if v.as_str() == "1.0"
    )));
    let four_row = farvi
        .iter()
        .find(|a| a.occurrence.capture.namespace == "captured:a:d1" && a.occurrence.locator == "/2")
        .unwrap();
    assert_eq!(
        match &four_row.definition {
            Presence::Value(t) => t.text.as_str(),
            _ => "<missing>",
        },
        four_place
    );
    assert!(matches!(&four_row.source_score, Presence::Value(v) if v.as_str() == "100002.0"));
    let repeated = farvi
        .iter()
        .find(|a| a.occurrence.capture.namespace == "captured:a:d2" && a.occurrence.locator == "/0")
        .unwrap();
    assert!(matches!(&repeated.definition, Presence::Value(t) if t.text == five_place));
    assert!(matches!(&repeated.source_score, Presence::Value(v) if v.as_str() == "1.0"));
    for (row, time, created) in [
        (&five_rows[0], "1479570326", "\"2016-11-19T15:45:26Z\""),
        (&four_row, "1046655416", "\"2003-03-03T01:36:56Z\""),
    ] {
        assert!(
            row.timestamps
                .iter()
                .any(|t| t.field == "time" && t.value.as_str() == time)
        );
        assert!(
            row.timestamps
                .iter()
                .any(|t| t.field == "created_at" && t.value.as_str() == created)
        );
    }
    let a_rows = farvi
        .iter()
        .filter(|a| a.occurrence.capture.namespace.starts_with("captured:a:"))
        .collect::<Vec<_>>();
    assert_eq!(a_rows.len(), 4);
    assert!(
        a_rows
            .iter()
            .all(|a| a.native_sense_id == Presence::Missing)
    );
    let synthetic_sense = d
        .metadata()
        .assertions
        .iter()
        .find(|a| {
            a.occurrence.capture.namespace == "captured:a:lexical" && a.occurrence.locator == "/0"
        })
        .unwrap();
    assert_eq!(
        synthetic_sense.native_sense_id,
        Presence::Value("synthetic-sense-si-e".into())
    );
    assert_eq!(
        d.metadata()
            .releases
            .iter()
            .find(|r| r.id.as_str() == "release-a")
            .unwrap()
            .accounting,
        vlajmaji_lexicon::ReleaseAccounting {
            input_occurrences: Some(17),
            normalized_occurrences: Some(17),
            query_excluded_occurrences: Some(1),
            unit: "occurrence".into()
        }
    );
    let duplicate_rows = d
        .captures()
        .iter()
        .find(|c| c.identity.namespace == "captured:a:d1")
        .unwrap();
    assert_eq!(
        duplicate_rows.occurrences[0].fields,
        duplicate_rows.occurrences[1].fields
    );
    assert_ne!(
        duplicate_rows.occurrences[0].locator,
        duplicate_rows.occurrences[1].locator
    );
    let out = d.to_json_bytes().unwrap();
    let restored = Dictionary::from_json_bytes(&out).unwrap();
    assert_eq!(restored.document(), d.document());
    assert_eq!(restored.metadata().contexts.len(), 8);
    assert_eq!(restored.metadata().frames.len(), 1);
    assert_eq!(restored.metadata().forms.len(), 7);
    assert_eq!(restored.metadata().etymologies.len(), 3);
    assert_eq!(restored.metadata().relations.len(), 4);
    assert_eq!(restored.metadata().owned_revisions.len(), 2);
    assert_eq!(
        restored.document().legacy_archive,
        d.document().legacy_archive
    );
    let mut invalid = d.document();
    invalid.version = 99;
    assert!(matches!(
        Dictionary::from_document(invalid),
        Err(DictionaryError::UnsupportedVersion(99))
    ));
    let default = DictionaryDocument {
        version: 0,
        ..DictionaryDocument::default()
    };
    assert!(Dictionary::from_document(default).is_err());
}

#[test]
fn complete_fixture_keeps_native_presence_and_release_absence_distinct() {
    let d = fixture_support::complete_fixture().unwrap();
    let si = d
        .captures()
        .iter()
        .find(|c| c.identity.namespace == "captured:a:lexical")
        .unwrap();
    let row0 = si.occurrences.iter().find(|o| o.locator == "/0").unwrap();
    assert!(
        row0.fields
            .iter()
            .any(|f| f.name == "unknown" && f.raw_value.contains("12345678901234567890"))
    );
    assert_eq!(
        d.metadata()
            .entries
            .iter()
            .find(|e| e.spelling == Presence::Value("si'e".into()))
            .unwrap()
            .selmaho,
        Presence::Value("MOI".into())
    );
    assert_eq!(d.metadata().etymologies[0].statement, Presence::Null);
    assert_eq!(
        d.metadata().etymologies[1].statement,
        Presence::Value(vlajmaji_lexicon::TextValue {
            text: String::new(),
            language: Presence::Missing,
            markup: Presence::Missing
        })
    );
    let missing = d
        .metadata()
        .releases
        .iter()
        .find(|r| r.availability == Availability::Missing)
        .unwrap();
    assert_eq!(missing.accounting.input_occurrences, None);
    assert_eq!(missing.accounting.normalized_occurrences, None);
    assert_eq!(
        d.metadata()
            .assertions
            .iter()
            .filter(|a| matches!(a.eligibility, Eligibility::Excluded { .. }))
            .count(),
        1
    );
    assert_eq!(
        d.metadata()
            .forms
            .iter()
            .filter(|f| matches!(f.kind, FormKind::AssignedRafsi))
            .count(),
        1
    );
    let gismu = d
        .captures()
        .iter()
        .find(|c| c.identity.namespace == "captured:a:gismu")
        .unwrap();
    let cpana = gismu.occurrences.iter().find(|o| o.locator == "2").unwrap();
    assert!(
        cpana
            .fields
            .iter()
            .any(|f| f.name == "assigned_rafsi_cvc" && f.raw_value.is_empty())
    );
    assert_eq!(
        cpana.raw_line.as_deref(),
        Some(
            " cpana              upon                                      x1 is upon/atop/resting on/lying on [the upper surface of] x2 in frame of reference/gravity x3   1e  63    (x1 may be object or event); (cf. se vasru, jbini, zvati, nenri, vreta, ckana, diklo, jibni, lamji, zutse, punji for lay upon, sarji, zbepi)"
        )
    );
    let registry = d.registry().document();
    assert_eq!(registry.distributors.len(), 2);
    assert_eq!(registry.snapshots.len(), 10);
    assert_eq!(
        registry
            .snapshots
            .iter()
            .find(|s| s.snapshot.namespace() == "captured:a:d1")
            .unwrap()
            .distributor
            .as_ref()
            .unwrap()
            .as_str(),
        "distributor-a-d1"
    );
    assert_eq!(
        registry
            .snapshots
            .iter()
            .find(|s| s.snapshot.namespace() == "captured:a:d2")
            .unwrap()
            .distributor
            .as_ref()
            .unwrap()
            .as_str(),
        "distributor-a-d2"
    );
    let repeated_native = registry
        .occurrences
        .iter()
        .filter(|a| {
            a.native_entry_id.as_deref() == Some("333")
                && a.native_assertion_id.as_deref() == Some("69002")
        })
        .collect::<Vec<_>>();
    assert_eq!(repeated_native.len(), 3);
    let distinct_occurrences = repeated_native
        .iter()
        .map(|a| {
            (
                a.occurrence.snapshot().namespace().to_owned(),
                a.occurrence.snapshot().sha256().to_owned(),
                a.occurrence.locator().to_owned(),
            )
        })
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(distinct_occurrences.len(), 3);
    assert!(repeated_native.iter().all(|a| {
        a.contributor
            .as_ref()
            .is_some_and(|c| c.as_str() == "suzil")
    }));
    let unknown = registry
        .occurrences
        .iter()
        .find(|a| {
            a.occurrence.snapshot().namespace() == "captured:a:lexical"
                && a.occurrence.locator() == "/0"
        })
        .unwrap();
    assert!(
        unknown.contributor.is_none()
            && unknown.native_entry_id.is_none()
            && unknown.native_assertion_id.is_none()
    );
    assert!(
        registry
            .contributors
            .iter()
            .any(|c| c.id.as_str() == "officialdata")
    );
    assert!(
        registry
            .aliases
            .iter()
            .any(|a| a.text == "archive-a" && a.status == vlajmaji_lexicon::AliasStatus::Active)
    );
    assert!(
        registry
            .aliases
            .iter()
            .any(|a| a.text == "legacy-a" && a.status == vlajmaji_lexicon::AliasStatus::Retired)
    );
    assert!(registry.lineage.iter().any(|l| matches!(&l.upstream, vlajmaji_lexicon::LineageTarget::Known(s) if s.as_str() == "source-b")));
    assert!(registry.lineage.iter().any(|l| matches!(&l.upstream, vlajmaji_lexicon::LineageTarget::Unresolved(s) if s == "unresolved mirror lineage")));
    assert!(registry.sources.iter().any(|s| s.id.as_str() == "source-m"));
    assert!(
        registry
            .snapshots
            .iter()
            .any(|s| s.source.as_str() == "source-m"
                && s.snapshot.namespace() == "catalogue:m:future")
    );
    assert_eq!(
        d.metadata()
            .collections
            .iter()
            .find(|c| c.id.as_str() == "collection-m")
            .unwrap()
            .source
            .as_str(),
        "source-m"
    );
    assert_eq!(
        d.metadata()
            .components
            .iter()
            .find(|c| c.id.as_str() == "component-m")
            .unwrap()
            .collection
            .as_str(),
        "collection-m"
    );
    assert_eq!(
        d.metadata()
            .releases
            .iter()
            .find(|r| r.id.as_str() == "release-m")
            .unwrap()
            .availability,
        Availability::Missing
    );
    assert_eq!(d.metadata().owned_heads[0].revision.as_str(), "revision-r1");
    assert!(
        d.metadata()
            .releases
            .iter()
            .find(|r| r.id.as_str() == "release-o1")
            .unwrap()
            .selected
    );
    assert!(
        !d.metadata()
            .releases
            .iter()
            .find(|r| r.id.as_str() == "release-o2")
            .unwrap()
            .selected
    );
    let contexts = &d.metadata().contexts;
    assert_eq!(contexts.len(), 8);
    assert!(
        contexts
            .iter()
            .any(|c| c.kind == vlajmaji_lexicon::ContextKind::Gloss
                && matches!(&c.value, Presence::Value(v) if v.text == "short gloss"))
    );
    assert!(
        contexts
            .iter()
            .any(|c| c.kind == vlajmaji_lexicon::ContextKind::Note
                && matches!(&c.value, Presence::Value(v) if v.text == "synthetic note"))
    );
    assert!(
        contexts
            .iter()
            .any(|c| c.kind == vlajmaji_lexicon::ContextKind::Label && c.value == Presence::Null)
    );
    assert!(
        contexts
            .iter()
            .any(|c| c.kind == vlajmaji_lexicon::ContextKind::Register
                && matches!(&c.value, Presence::Value(v) if v.text.is_empty()))
    );
    for kind in [
        vlajmaji_lexicon::ContextKind::Domain,
        vlajmaji_lexicon::ContextKind::Example,
        vlajmaji_lexicon::ContextKind::Translation,
        vlajmaji_lexicon::ContextKind::Citation,
    ] {
        assert!(
            contexts
                .iter()
                .any(|c| c.kind == kind && matches!(&c.value, Presence::Value(_)))
        );
    }
    assert_eq!(d.metadata().frames[0].id.as_str(), "frame-si-e");
    assert_eq!(d.metadata().frames[0].slots[0].native_label, "x1");
    assert_eq!(d.metadata().frames[0].slots[0].order, 1);
    assert!(
        matches!(&d.metadata().frames[0].slots[0].description, Presence::Value(v) if v.text == "snow")
    );
    let frame_assertion = d.metadata().frames[0].assertion.clone();
    assert_eq!(
        d.metadata()
            .assertions
            .iter()
            .find(|a| a.id == frame_assertion)
            .unwrap()
            .occurrence
            .capture
            .namespace,
        "captured:a:lexical"
    );
    assert!(
        d.metadata().frames.iter().all(|f| !d
            .metadata()
            .assertions
            .iter()
            .find(|a| a.id == f.assertion)
            .unwrap()
            .occurrence
            .capture
            .namespace
            .starts_with("captured:a:d")),
        "farvi place prose is not a parsed frame"
    );
    let entry = |spelling: &str| {
        d.metadata()
            .entries
            .iter()
            .find(|e| e.spelling == Presence::Value(spelling.into()))
            .unwrap()
    };
    assert_eq!(entry("si'e").language, Presence::Value("jbo".into()));
    assert_eq!(
        entry("si'e").native_word_class,
        Presence::Value("cmavo".into())
    );
    assert_eq!(entry("si'e").selmaho, Presence::Value("MOI".into()));
    assert_eq!(
        entry("cpana").native_word_class,
        Presence::Value("gismu".into())
    );
    assert!(
        matches!(&entry("cpana").mapped_pos, Presence::Value(p) if p.rule == "explicit-fixture-map-v1")
    );
    assert!(
        d.metadata()
            .entries
            .iter()
            .any(|e| e.spelling == Presence::Missing)
    );
    let form = |id: &str| {
        d.metadata()
            .forms
            .iter()
            .find(|f| f.id.as_str() == id)
            .unwrap()
    };
    assert_eq!(form("form-variant").spelling, "siʼe");
    assert_eq!(
        form("form-variant").standing,
        Presence::Value("reported".into())
    );
    assert!(
        matches!(&form("form-pronunciation").pronunciation, Presence::Value(v) if v.text == "siʔe")
    );
    assert_eq!(form("form-assigned").kind, FormKind::AssignedRafsi);
    assert_eq!(form("form-reported").spelling, "cpan");
    assert_eq!(form("form-stem").spelling, "snim");
    assert_eq!(
        form("form-stem").computation.as_ref().unwrap().as_str(),
        "computation-1"
    );
    assert_eq!(form("form-generated").components[0].order, 1);
    assert_eq!(form("form-generated").components[0].spelling, "snime");
    assert_eq!(
        form("form-components").components[0].target,
        Some(vlajmaji_lexicon::LocalRef::Entry(entry("cpana").id.clone()))
    );
    assert_eq!(
        form("form-generated").components[0].target,
        Some(vlajmaji_lexicon::LocalRef::Entry(entry("snime").id.clone()))
    );
    assert_eq!(d.metadata().etymologies[0].statement, Presence::Null);
    assert!(
        matches!(&d.metadata().etymologies[1].statement, Presence::Value(v) if v.text.is_empty())
    );
    assert!(
        matches!(&d.metadata().etymologies[2].statement, Presence::Value(v) if v.text == "derived source text")
    );
    let relation_see = d
        .metadata()
        .relations
        .iter()
        .find(|r| r.id.as_str() == "relation-see")
        .unwrap();
    assert_eq!(relation_see.scope, vlajmaji_lexicon::RelationScope::Entry);
    assert!(matches!(
        &relation_see.target,
        vlajmaji_lexicon::RelationTarget::Unknown
    ));
    assert!(d.metadata().relations.iter().any(|r| r.id.as_str() == "relation-unresolved" && matches!(&r.target, vlajmaji_lexicon::RelationTarget::Unresolved(v) if v == "unresolved target")));
    assert!(d.metadata().relations.iter().any(|r| r.id.as_str() == "relation-missing-catalogue" && matches!(&r.target, vlajmaji_lexicon::RelationTarget::Catalog(t) if t.source.as_str() == "source-m" && t.release.as_str() == "release-m" && t.native_entry_id.as_deref() == Some("native-missing"))));
    let lexical = d
        .captures()
        .iter()
        .find(|c| c.identity.namespace == "captured:a:lexical")
        .unwrap();
    let target = lexical.occurrences[0]
        .fields
        .iter()
        .find(|f| f.name == "source_etymology_target")
        .unwrap();
    assert_eq!(
        target.raw_value,
        r#"{"source":"source-m","release":"release-m","native_entry_id":"native-missing","occurrence":{"capture":{"namespace":"catalogue:m:future","sha256":"0000000000000000000000000000000000000000000000000000000000000000"},"locator":"/0"}}"#
    );
    assert_eq!(
        d.metadata().owned_revisions[1]
            .parent
            .as_ref()
            .unwrap()
            .as_str(),
        "revision-r1"
    );
    assert_eq!(
        d.metadata().owned_revisions[1].tombstones[0],
        d.metadata().owned_revisions[0].assertions[0]
    );
    assert_eq!(d.metadata().editorial_senses[0].assertions.len(), 2);
    assert_ne!(
        d.metadata()
            .assertions
            .iter()
            .find(|a| a.id == d.metadata().editorial_senses[0].assertions[0])
            .unwrap()
            .occurrence
            .capture
            .namespace,
        d.metadata()
            .assertions
            .iter()
            .find(|a| a.id == d.metadata().editorial_senses[0].assertions[1])
            .unwrap()
            .occurrence
            .capture
            .namespace
    );
    assert_eq!(d.metadata().computations[0].tool, "fixture-tool");
    assert_eq!(d.metadata().computations[0].tool_version, "1");
    assert_eq!(d.metadata().computations[0].policy, "fixture-policy");
    assert_eq!(
        d.metadata().computations[0].profile,
        Presence::Value("fixture-profile".into())
    );
    assert_eq!(d.metadata().computations[0].inputs.len(), 2);
    assert!(
        matches!(&d.metadata().computations[0].inputs[0], vlajmaji_lexicon::InputRef::Occurrence(o) if o.capture.namespace == "captured:a:lexical" && o.locator == "/0")
    );
    assert!(
        matches!(&d.metadata().computations[0].inputs[1], vlajmaji_lexicon::InputRef::Literal(s) if s == "explicit input")
    );
    assert!(
        matches!(&d.metadata().computations[0].explanation, Presence::Value(v) if v.text == "labelled computation; no automatic analysis")
    );
    assert!(
        matches!(&d.metadata().annotations[0].explanation, Presence::Value(v) if v.text == "fixture output")
    );
    assert_eq!(
        d.metadata().annotations[0].value.as_str(),
        r#"{"score":0.5,"labels":["synthetic",true]}"#
    );
    assert!(
        d.metadata().extensions[0]
            .value
            .as_str()
            .contains("12345678901234567890")
    );
    assert_eq!(d.metadata().code_notices[0].text, "fixture code notice");
    assert_eq!(
        d.metadata().code_notices[0].locator,
        Presence::Value("docs/dictionary-model.md".into())
    );
    assert_eq!(
        d.metadata().code_notices[0].license,
        Presence::Value("project terms".into())
    );
    assert_eq!(
        d.metadata()
            .releases
            .iter()
            .find(|r| r.id.as_str() == "release-a")
            .unwrap()
            .content_notices[0]
            .text,
        Presence::Value("fixture notice".into())
    );
    assert_eq!(
        d.metadata()
            .releases
            .iter()
            .find(|r| r.id.as_str() == "release-a")
            .unwrap()
            .content_notices[0]
            .license,
        Presence::Missing
    );
    assert!(
        d.metadata()
            .releases
            .iter()
            .any(|r| r.id.as_str() == "release-a"
                && r.redistribution == vlajmaji_lexicon::RedistributionStatus::Held)
    );
    assert!(
        d.metadata()
            .releases
            .iter()
            .any(|r| r.id.as_str() == "release-b"
                && r.redistribution == vlajmaji_lexicon::RedistributionStatus::Unknown)
    );
    assert!(
        d.metadata()
            .releases
            .iter()
            .any(|r| r.id.as_str() == "release-o1"
                && r.redistribution == vlajmaji_lexicon::RedistributionStatus::Permitted
                && !r.redistribution_evidence.is_empty())
    );
}

#[test]
fn constructor_rejects_raw_reference_accounting_and_provenance_failures() {
    let original = fixture_support::complete_fixture().unwrap();
    let mut d = original.document();
    d.captures[0].bytes[0] ^= 1;
    assert!(matches!(
        Dictionary::from_document(d),
        Err(DictionaryError::CaptureHashMismatch { .. })
    ));
    let mut d = original.document();
    let duplicate = d.captures[0].occurrences[0].clone();
    d.captures[0].occurrences.push(duplicate);
    assert!(Dictionary::from_document(d).is_err());
    let mut d = original.document();
    d.metadata.assertions[0].occurrence.locator = "/phantom".into();
    assert!(Dictionary::from_document(d).is_err());
    let mut d = original.document();
    d.metadata
        .releases
        .iter_mut()
        .find(|r| r.id.as_str() == "release-a")
        .unwrap()
        .accounting
        .input_occurrences = Some(16);
    assert!(Dictionary::from_document(d).is_err());
    let mut d = original.document();
    d.metadata.contexts[0].evidence.field_index = Some(999);
    assert!(Dictionary::from_document(d).is_err());
    let mut d = original.document();
    d.metadata
        .forms
        .iter_mut()
        .find(|f| f.kind == FormKind::ComputedStem)
        .unwrap()
        .evidence = Some(d.metadata.contexts[0].evidence.clone());
    assert!(Dictionary::from_document(d).is_err());
    let mut d = original.document();
    d.metadata.owned_revisions[0].parent = Some(d.metadata.owned_revisions[1].id.clone());
    assert!(Dictionary::from_document(d).is_err());
    let mut d = original.document();
    d.metadata.owned_revisions[1].tombstones =
        vec![d.metadata.owned_revisions[1].assertions[0].clone()];
    assert!(Dictionary::from_document(d).is_err());
}

#[test]
fn available_empty_release_is_distinct_from_unavailable_catalogue_and_bad_refs_reject() {
    let original = fixture_support::complete_fixture().unwrap();
    let mut d = original.document();
    let mut empty = d
        .metadata
        .releases
        .iter()
        .find(|r| r.id.as_str() == "release-b")
        .unwrap()
        .clone();
    empty.id = EntityId::new("release-empty").unwrap();
    empty.captures.clear();
    empty.assertions.clear();
    empty.accounting = vlajmaji_lexicon::ReleaseAccounting {
        input_occurrences: Some(0),
        normalized_occurrences: Some(0),
        query_excluded_occurrences: Some(0),
        unit: "occurrence".into(),
    };
    d.metadata.releases.push(empty);
    assert!(Dictionary::from_document(d).is_ok());

    let mut d = original.document();
    let missing = d
        .metadata
        .releases
        .iter_mut()
        .find(|r| r.id.as_str() == "release-m")
        .unwrap();
    missing.availability = Availability::Available;
    missing.accounting = vlajmaji_lexicon::ReleaseAccounting {
        input_occurrences: Some(0),
        normalized_occurrences: Some(0),
        query_excluded_occurrences: Some(0),
        unit: "occurrence".into(),
    };
    assert!(Dictionary::from_document(d).is_err());

    let mut d = original.document();
    d.metadata.entries.push(d.metadata.entries[0].clone());
    assert!(Dictionary::from_document(d).is_err());
}

#[test]
fn direct_serde_is_strict_and_identity_adapters_are_exact() {
    let d = fixture_support::complete_fixture().unwrap();
    let json = String::from_utf8(d.to_json_bytes().unwrap()).unwrap();
    let unknown = json.replacen(
        "\"catalogue_revision\":1",
        "\"catalogue_revision\":1,\"surprise\":true",
        1,
    );
    assert!(serde_json::from_str::<Dictionary>(&unknown).is_err());
    let nested = json.replacen(
        "\"spelling\":{\"Value\":\"farvi\"}",
        "\"spelling\":{\"Value\":\"farvi\"},\"unknown_nested\":true",
        1,
    );
    assert_ne!(nested, json);
    assert!(serde_json::from_str::<Dictionary>(&nested).is_err());
    let mut enum_wire = serde_json::to_value(d.document()).unwrap();
    let excluded = d
        .metadata()
        .assertions
        .iter()
        .position(|a| matches!(a.eligibility, Eligibility::Excluded { .. }))
        .unwrap();
    enum_wire["metadata"]["assertions"][excluded]["eligibility"]["Excluded"]["unexpected"] =
        serde_json::json!(true);
    let enum_bytes = serde_json::to_vec(&enum_wire).unwrap();
    assert!(serde_json::from_slice::<Dictionary>(&enum_bytes).is_err());
    assert!(Dictionary::from_json_bytes(&enum_bytes).is_err());
    let capture = &d.captures()[0].identity;
    let legacy = capture.to_snapshot_identity().unwrap();
    assert_eq!(legacy.namespace(), capture.namespace);
    assert_eq!(legacy.sha256(), capture.sha256);
    let occurrence = &d.metadata().assertions[0].occurrence;
    let legacy = occurrence.to_record_identity().unwrap();
    assert_eq!(legacy.locator(), occurrence.locator);
    assert_eq!(legacy.snapshot().namespace(), occurrence.capture.namespace);
    assert_eq!(legacy.snapshot().sha256(), occurrence.capture.sha256);
}

#[test]
fn known_missing_capture_can_retain_unverifiable_projection_with_unknown_counts() {
    let base = fixture_support::complete_fixture().unwrap();
    let mut d = base.document();
    let release = d
        .metadata
        .releases
        .iter_mut()
        .find(|r| r.id.as_str() == "release-m")
        .unwrap();
    let capture = release.captures[0].clone();
    let occurrence = vlajmaji_lexicon::OccurrenceRef {
        capture,
        locator: "/known-row".into(),
    };
    let assertion = EntityId::new("missing-assertion").unwrap();
    let entry = EntityId::new("missing-entry").unwrap();
    d.metadata.entries.push(vlajmaji_lexicon::EntryRecord {
        id: entry.clone(),
        source: vlajmaji_lexicon::SourceId::new("source-m").unwrap(),
        spelling: Presence::Missing,
        language: Presence::Missing,
        script: Presence::Missing,
        native_word_class: Presence::Missing,
        selmaho: Presence::Missing,
        mapped_pos: Presence::Missing,
    });
    d.metadata
        .assertions
        .push(vlajmaji_lexicon::AssertionRecord {
            id: assertion.clone(),
            entry,
            occurrence: occurrence.clone(),
            native_sense_id: Presence::Value("remote-id".into()),
            language: Presence::Value("en".into()),
            definition: Presence::Value(vlajmaji_lexicon::TextValue {
                text: "retained projection".into(),
                language: Presence::Value("en".into()),
                markup: Presence::Missing,
            }),
            source_score: Presence::Missing,
            timestamps: vec![],
            eligibility: Eligibility::Excluded {
                reason: "source spelling unavailable".into(),
            },
        });
    release.assertions.push(assertion.clone());
    d.metadata.contexts.push(vlajmaji_lexicon::ContextRecord {
        id: EntityId::new("missing-context").unwrap(),
        assertion,
        kind: vlajmaji_lexicon::ContextKind::Citation,
        value: Presence::Value(vlajmaji_lexicon::TextValue {
            text: "citation retained".into(),
            language: Presence::Missing,
            markup: Presence::Missing,
        }),
        translated_from: None,
        evidence: vlajmaji_lexicon::SourceEvidence {
            occurrence,
            field_index: Some(99),
            citation: Presence::Value("catalogue-only reference".into()),
            offset: None,
        },
    });
    assert!(Dictionary::from_document(d).is_ok());
}

#[test]
fn declared_validation_categories_are_emitted_and_malformed_parent_paths_do_not_panic() {
    let original = fixture_support::complete_fixture().unwrap();

    let mut d = original.document();
    d.metadata.entries.push(d.metadata.entries[0].clone());
    assert!(matches!(
        Dictionary::from_document(d),
        Err(DictionaryError::DuplicateEntity(_))
    ));

    let mut d = original.document();
    d.metadata.contexts[0].assertion = EntityId::new("missing-assertion").unwrap();
    assert!(matches!(
        Dictionary::from_document(d),
        Err(DictionaryError::MissingReference(_))
    ));

    let mut d = original.document();
    d.metadata
        .releases
        .iter_mut()
        .find(|r| r.id.as_str() == "release-a")
        .unwrap()
        .accounting
        .input_occurrences = Some(999);
    assert!(matches!(
        Dictionary::from_document(d),
        Err(DictionaryError::InvalidAccounting(_))
    ));

    let mut d = original.document();
    let missing = d
        .metadata
        .releases
        .iter_mut()
        .find(|r| r.id.as_str() == "release-m")
        .unwrap();
    missing.availability = Availability::Available;
    missing.accounting.input_occurrences = Some(0);
    missing.accounting.normalized_occurrences = Some(0);
    missing.accounting.query_excluded_occurrences = Some(0);
    assert!(matches!(
        Dictionary::from_document(d),
        Err(DictionaryError::InvalidAvailability(_))
    ));

    let mut d = original.document();
    let computation = d.metadata.computations[0].id.clone();
    let form = d
        .metadata
        .forms
        .iter_mut()
        .find(|f| f.kind == FormKind::ComputedStem)
        .unwrap();
    form.evidence = Some(d.metadata.contexts[0].evidence.clone());
    form.computation = Some(computation);
    assert!(matches!(
        Dictionary::from_document(d),
        Err(DictionaryError::InvalidProvenance(_))
    ));

    let mut d = original.document();
    d.metadata.owned_revisions[0].parent = Some(d.metadata.owned_revisions[1].id.clone());
    assert!(matches!(
        Dictionary::from_document(d),
        Err(DictionaryError::InvalidHistory(_))
    ));

    for access in 0..3 {
        let mut malformed = original.document();
        let second = malformed.metadata.owned_revisions[1].id.clone();
        malformed.metadata.owned_revisions[0].parent = Some(second);
        malformed.metadata.owned_revisions[1].parent =
            Some(EntityId::new("missing-parent").unwrap());
        let bytes = serde_json::to_vec(&malformed).unwrap();
        match access {
            0 => assert!(matches!(
                Dictionary::from_document(malformed),
                Err(DictionaryError::MissingReference(_))
            )),
            1 => assert!(
                std::panic::catch_unwind(|| Dictionary::from_json_bytes(&bytes))
                    .unwrap()
                    .is_err()
            ),
            _ => assert!(
                std::panic::catch_unwind(|| serde_json::from_slice::<Dictionary>(&bytes))
                    .unwrap()
                    .is_err()
            ),
        }
    }
}

#[test]
fn etymology_components_accept_each_declared_local_reference_role() {
    let base = fixture_support::complete_fixture().unwrap().document();
    let targets = [
        vlajmaji_lexicon::LocalRef::Entry(base.metadata.entries[0].id.clone()),
        vlajmaji_lexicon::LocalRef::Assertion(base.metadata.assertions[0].id.clone()),
        vlajmaji_lexicon::LocalRef::Form(base.metadata.forms[0].id.clone()),
        vlajmaji_lexicon::LocalRef::EditorialSense(base.metadata.editorial_senses[0].id.clone()),
    ];
    for (order, target) in targets.into_iter().enumerate() {
        let mut draft = base.clone();
        draft.metadata.etymologies[0]
            .components
            .push(vlajmaji_lexicon::OrderedComponent {
                order: order as u32 + 1,
                spelling: format!("component-{order}"),
                target: Some(target),
            });
        let bytes = serde_json::to_vec(&draft).unwrap();
        assert!(Dictionary::from_document(draft).is_ok());
        assert!(Dictionary::from_json_bytes(&bytes).is_ok());
        assert!(serde_json::from_slice::<Dictionary>(&bytes).is_ok());
    }

    let mut wrong_role = base;
    wrong_role.metadata.etymologies[0]
        .components
        .push(vlajmaji_lexicon::OrderedComponent {
            order: 1,
            spelling: "wrong-role".into(),
            target: Some(vlajmaji_lexicon::LocalRef::Assertion(
                wrong_role.metadata.entries[0].id.clone(),
            )),
        });
    assert!(matches!(
        Dictionary::from_document(wrong_role),
        Err(DictionaryError::MissingReference(_))
    ));
}

#[test]
fn named_reference_and_provenance_failures_use_declared_categories() {
    let base = fixture_support::complete_fixture().unwrap().document();

    let mut form_entry = base.clone();
    form_entry.metadata.forms[0].entry = EntityId::new("missing-entry").unwrap();
    assert!(matches!(
        Dictionary::from_document(form_entry),
        Err(DictionaryError::MissingReference(_))
    ));

    let mut relation_origin = base.clone();
    relation_origin.metadata.relations[0].origin =
        vlajmaji_lexicon::LocalRef::Entry(EntityId::new("missing-entry").unwrap());
    assert!(matches!(
        Dictionary::from_document(relation_origin),
        Err(DictionaryError::MissingReference(_))
    ));

    let mut foreign_form_evidence = base.clone();
    let foreign = foreign_form_evidence
        .metadata
        .assertions
        .iter()
        .find(|a| a.occurrence.capture.namespace == "captured:b:d1")
        .unwrap()
        .occurrence
        .clone();
    let form = foreign_form_evidence
        .metadata
        .forms
        .iter_mut()
        .find(|f| {
            f.evidence.is_some()
                && foreign_form_evidence
                    .metadata
                    .entries
                    .iter()
                    .find(|entry| entry.id == f.entry)
                    .is_some_and(|entry| entry.source.as_str() == "source-a")
        })
        .unwrap();
    let evidence = form.evidence.as_mut().unwrap();
    evidence.occurrence = foreign;
    evidence.field_index = None;
    evidence.offset = None;
    let foreign_form_error = Dictionary::from_document(foreign_form_evidence);
    assert!(
        matches!(
            foreign_form_error,
            Err(DictionaryError::InvalidProvenance(_))
        ),
        "{foreign_form_error:?}"
    );

    let mut incomplete_computation = base.clone();
    incomplete_computation.metadata.computations[0].tool.clear();
    assert!(matches!(
        Dictionary::from_document(incomplete_computation),
        Err(DictionaryError::InvalidProvenance(_))
    ));

    let mut phantom_evidence = base.clone();
    let loaded_capture = phantom_evidence.metadata.assertions[0]
        .occurrence
        .capture
        .clone();
    phantom_evidence.metadata.extensions[0].evidence = Some(vlajmaji_lexicon::SourceEvidence {
        occurrence: vlajmaji_lexicon::OccurrenceRef {
            capture: loaded_capture,
            locator: "/missing-evidence".into(),
        },
        field_index: None,
        citation: Presence::Missing,
        offset: None,
    });
    assert!(matches!(
        Dictionary::from_document(phantom_evidence),
        Err(DictionaryError::MissingReference(_))
    ));

    let mut phantom_assertion = base.clone();
    phantom_assertion.metadata.assertions[0].occurrence.locator = "/missing-assertion".into();
    assert!(matches!(
        Dictionary::from_document(phantom_assertion),
        Err(DictionaryError::MissingReference(_))
    ));

    let mut unbound_capture = base;
    let mut registry = unbound_capture.registry.document();
    registry
        .snapshots
        .retain(|snapshot| snapshot.snapshot.namespace() != "captured:b:d1");
    unbound_capture.registry = vlajmaji_lexicon::SourceRegistry::from_document(registry).unwrap();
    assert!(matches!(
        Dictionary::from_document(unbound_capture),
        Err(DictionaryError::MissingReference(_))
    ));
}
