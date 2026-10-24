use std::collections::BTreeMap;
use std::error::Error;
use vlajmaji_lexicon::{
    AbbreviationKind, AliasStatus, ArgumentFrameRecord, ArgumentSlot, AssertionRecord,
    Availability, CaptureMetadata, CaptureRef, CatalogTarget, CodeNotice, CollectionRecord,
    ComponentRecord, ComputationRecord, ContentNotice, ContextKind, ContextRecord as Context,
    ContributorId, Dictionary, DictionaryMetadata, DistributorId, DistributorRecord, Eligibility,
    EntityId, EtymologyRecord, ExtensionRecord, FormKind, FormRecord, InputRef, LineageTarget,
    LocalRef, NamedTimestamp, NativeEncoding, NativeField, OccurrenceAttribution, OccurrenceRef,
    OrderedComponent, OwnedHead, OwnedRevisionRecord, PosMapping, Presence, RawCapture, RawJson,
    RawOccurrence, RedistributionStatus, RegistryDocument, RelationKind, RelationRecord,
    RelationScope, RelationTarget, ReleaseAccounting, ReleaseDiagnostic, ReleaseRecord,
    SnapshotAttribution, SourceAlias, SourceEvidence, SourceFormat, SourceId, SourceLineage,
    SourceMetadata, SourceOrigin, SourceRecord, SourceRegistry, TextValue, TimePrecision,
};

fn eid(s: &str) -> EntityId {
    EntityId::new(s).expect("fixture entity ID")
}
fn source(s: &str) -> SourceId {
    SourceId::new(s).expect("fixture source ID")
}
fn tv(s: &str) -> TextValue {
    TextValue {
        text: s.into(),
        language: Presence::Value("en".into()),
        markup: Presence::Missing,
    }
}
fn meta(namespace: &str, name: &str, format: SourceFormat, loc: &str) -> SourceMetadata {
    SourceMetadata::new(
        namespace,
        name,
        format,
        loc,
        "fixture-v1",
        None,
        TimePrecision::Unknown,
    )
}
fn evidence(occurrence: OccurrenceRef, field_index: Option<usize>) -> SourceEvidence {
    SourceEvidence {
        occurrence,
        field_index,
        citation: Presence::Value(
            "labelled synthetic fixture; pinned witnesses noted in documentation".into(),
        ),
        offset: None,
    }
}
fn native_text(name: &str, value: &str) -> NativeField {
    NativeField {
        name: name.into(),
        encoding: NativeEncoding::Text,
        raw_value: value.into(),
    }
}
fn native_json(name: &str, value: &str) -> NativeField {
    NativeField {
        name: name.into(),
        encoding: NativeEncoding::Json,
        raw_value: value.into(),
    }
}

/// Decode the plain quoted scalar fields used by this pinned fixture. This is
/// intentionally not a general JSON parser; the exact raw lexeme stays in the
/// capture and unsupported escapes remain unprojected.
fn fixture_string(raw: &str) -> Option<String> {
    let inner = raw.strip_prefix('"')?.strip_suffix('"')?;
    (!inner.contains('\\')).then(|| inner.to_owned())
}

pub fn complete_fixture() -> Result<Dictionary, Box<dyn Error>> {
    let farvi_five = r#"{"definitionid":69002,"valsiword":"farvi","valsiid":333,"langid":2,"definition":"$x_1$ (object) develops/evolves towards/into $x_2$ (state) from $x_3$ (state) for reason/because of $x_4$ (fact, event) through stages $x_5$","notes":"","etymology":null,"selmaho":null,"jargon":"","definitionnum":69002,"langrealname":"English","username":"suzil","time":1479570326,"created_at":"2016-11-19T15:45:26Z","type_name":"gismu","official_rafsi":"fav farv","experimental_rafsi":null,"rafsi":"fav farv","score":1.0,"comment_count":null,"gloss_keywords":null,"place_keywords":null,"user_vote":null,"owner_only":false,"can_edit":false,"has_image":false,"sound_url":"/api/jbovlaste/valsi/farvi/sound","embedding":null,"similarity":null}"#;
    let farvi_four = r#"{"definitionid":333,"valsiword":"farvi","valsiid":333,"langid":2,"definition":"$x_{1}$ develops/evolves towards/into $x_{2}$ from $x_{3}$ through stages $x_{4}.","notes":"See also {pruce}, {banro}, {makcu}, {ciste}, {cupra}, {ferti}.","etymology":null,"selmaho":null,"jargon":null,"definitionnum":1,"langrealname":"English","username":"officialdata","time":1046655416,"created_at":"2003-03-03T01:36:56Z","type_name":"gismu","official_rafsi":"fav farv","experimental_rafsi":null,"rafsi":"fav farv","score":100002.0,"comment_count":null,"gloss_keywords":null,"place_keywords":null,"user_vote":null,"owner_only":true,"can_edit":false,"has_image":false,"sound_url":"/api/jbovlaste/valsi/farvi/sound","embedding":null,"similarity":null}"#;
    let a_d1 = format!("[{farvi_five},{farvi_five},{farvi_four}]");
    let a_d2 = format!("[{farvi_five}]");
    let b_d1 = br#"[{"word":"farvi","definition":"a competing synthetic definition","word_type":"brivla"}]"#;
    let a_words = r#"[{"word":"si'e","definition":"cmavo, MOI","word_type":"cmavo","selmaho":"MOI","notes":"synthetic note","rafsi":["sni"],"variant":"siʼe","pronunciation":"siʔe","etymology":null,"frame":[{"order":1,"label":"x1","description":"snow"}],"unknown":{"array":[1,true,null],"huge":1234567890123456789012345678901234567890}},{"word":"sie","definition":"distinct spelling","word_type":"cmavo","etymology":""},{"word":"Si'e","definition":"case retained","word_type":"cmavo"},{"word":"si’e","definition":"curly apostrophe retained","word_type":"cmavo"},{"word":".si'e","definition":"leading dot retained","word_type":"cmavo"},{"word":"sy","definition":"y spelling retained","word_type":"cmavo"},{"word":"bai","definition":"with; by means of","word_type":"cmavo","selmaho":"BAI"},{"word":"le jaica","definition":"a multiword spelling","word_type":"cmavo"},{"word":"cpana","definition":"above","word_type":"gismu","etymology":"derived source text","rafsi":"cpan"},{"word":"snime","definition":"snow","word_type":"gismu","rafsi":["snim"]}]"#;
    let cpana_line = format!("\n{}\n", " cpana              upon                                      x1 is upon/atop/resting on/lying on [the upper surface of] x2 in frame of reference/gravity x3   1e  63    (x1 may be object or event); (cf. se vasru, jbini, zvati, nenri, vreta, ckana, diklo, jibni, lamji, zutse, punji for lay upon, sarji, zbepi)").into_bytes();
    let o1 = br#"[{"word":"farvi","definition":"owned first wording","word_type":"brivla"}]"#;
    let o2 = br#"[{"word":"farvi","definition":"owned revised wording","word_type":"brivla"}]"#;
    let mut lex = vlajmaji_lexicon::Lexicon::new();
    for (m, b) in [
        (
            meta(
                "captured:a:d1",
                "A dictionary v1",
                SourceFormat::LensiskuJson,
                "memory:a:d1",
            ),
            a_d1.as_bytes(),
        ),
        (
            meta(
                "captured:a:d2",
                "A mirror",
                SourceFormat::LensiskuJson,
                "memory:a:d2",
            ),
            a_d2.as_bytes(),
        ),
        (
            meta(
                "captured:b:d1",
                "B dictionary",
                SourceFormat::LaLojbanJson,
                "memory:b:d1",
            ),
            b_d1.as_slice(),
        ),
        (
            meta(
                "captured:a:lexical",
                "A lexical rows",
                SourceFormat::LaLojbanJson,
                "memory:a:lexical",
            ),
            a_words.as_bytes(),
        ),
        (
            meta(
                "captured:a:rafsi",
                "A assigned rafsi",
                SourceFormat::LlgRafsiRows,
                "memory:a:rafsi",
            ),
            b"si'e snime snow\n".as_slice(),
        ),
        (
            meta(
                "captured:a:gismu",
                "A gismu",
                SourceFormat::LlgGismuFixed,
                "memory:a:gismu",
            ),
            cpana_line.as_slice(),
        ),
        (
            meta(
                "owned:o:r1",
                "Owned r1",
                SourceFormat::LaLojbanJson,
                "memory:o:r1",
            ),
            o1.as_slice(),
        ),
        (
            meta(
                "owned:o:r2",
                "Owned r2",
                SourceFormat::LaLojbanJson,
                "memory:o:r2",
            ),
            o2.as_slice(),
        ),
    ] {
        lex.import_source(m, b, None)?;
    }
    let archive = lex.to_archive_bytes()?;
    let mut rd = RegistryDocument::default();
    for (id, name, abbr, origin) in [
        ("source-a", "Captured A", "a", SourceOrigin::Upstream),
        ("source-b", "Captured B", "b", SourceOrigin::Upstream),
        ("source-o", "Owned O", "o", SourceOrigin::OwnedDraft),
        (
            "source-m",
            "Catalogue source M",
            "m",
            SourceOrigin::Upstream,
        ),
    ] {
        rd.sources.push(SourceRecord {
            id: source(id),
            display_name: name.into(),
            abbreviation: abbr.into(),
            abbreviation_kind: AbbreviationKind::ProjectAssigned,
            abbreviation_evidence: None,
            origin,
        });
    }
    rd.contributors.extend([
        vlajmaji_lexicon::ContributorRecord {
            id: ContributorId::new("editor-1")?,
            display_name: "Fixture editor".into(),
        },
        vlajmaji_lexicon::ContributorRecord {
            id: ContributorId::new("suzil")?,
            display_name: "Source author suzil".into(),
        },
        vlajmaji_lexicon::ContributorRecord {
            id: ContributorId::new("officialdata")?,
            display_name: "Official data contributor".into(),
        },
    ]);
    rd.distributors.extend([
        DistributorRecord {
            id: DistributorId::new("distributor-a-d1")?,
            display_name: "Captured A D1 distributor".into(),
        },
        DistributorRecord {
            id: DistributorId::new("distributor-a-d2")?,
            display_name: "Captured A D2 distributor".into(),
        },
    ]);
    rd.aliases.extend([
        SourceAlias {
            source: source("source-a"),
            text: "archive-a".into(),
            status: AliasStatus::Active,
            evidence: Some("fixture selector alias".into()),
        },
        SourceAlias {
            source: source("source-a"),
            text: "legacy-a".into(),
            status: AliasStatus::Retired,
            evidence: Some("fixture retired alias".into()),
        },
    ]);
    rd.lineage.extend([
        SourceLineage {
            source: source("source-a"),
            upstream: LineageTarget::Known(source("source-b")),
            evidence: Some("fixture documented relationship".into()),
        },
        SourceLineage {
            source: source("source-a"),
            upstream: LineageTarget::Unresolved("unresolved mirror lineage".into()),
            evidence: Some("fixture unresolved locator".into()),
        },
    ]);
    let source_for_namespace = |n: &str| {
        if n.starts_with("owned:o:") {
            source("source-o")
        } else if n.starts_with("captured:b:") {
            source("source-b")
        } else {
            source("source-a")
        }
    };
    for snap in lex.snapshots() {
        let distributor = match snap.namespace() {
            "captured:a:d1" => Some(DistributorId::new("distributor-a-d1")?),
            "captured:a:d2" => Some(DistributorId::new("distributor-a-d2")?),
            _ => None,
        };
        rd.snapshots.push(SnapshotAttribution {
            snapshot: snap.clone(),
            source: source_for_namespace(snap.namespace()),
            distributor,
        });
    }
    for row in lex.records().iter().filter(|row| {
        matches!(
            row.identity().snapshot().namespace(),
            "captured:a:d1" | "captured:a:d2" | "captured:a:lexical"
        )
    }) {
        let json_string = |field: &str| match row.field(field) {
            vlajmaji_lexicon::FieldPresence::Value(raw) => fixture_string(raw),
            _ => None,
        };
        let raw_scalar = |field: &str| match row.field(field) {
            vlajmaji_lexicon::FieldPresence::Value(raw) => Some(raw.to_owned()),
            _ => None,
        };
        let username = json_string("username");
        let contributor = username.as_deref().and_then(|name| match name {
            "suzil" => ContributorId::new("suzil").ok(),
            "officialdata" => ContributorId::new("officialdata").ok(),
            _ => None,
        });
        rd.occurrences.push(OccurrenceAttribution {
            occurrence: row.identity().clone(),
            contributor,
            native_entry_id: raw_scalar("valsiid"),
            native_assertion_id: raw_scalar("definitionid"),
        });
    }
    let registry = SourceRegistry::from_document(rd)?;
    let mut md = DictionaryMetadata {
        catalogue_revision: 1,
        ..DictionaryMetadata::default()
    };
    md.code_notices.push(CodeNotice {
        text: "fixture code notice".into(),
        locator: Presence::Value("docs/dictionary-model.md".into()),
        license: Presence::Value("project terms".into()),
    });
    for (id, src, name) in [
        ("collection-a", "source-a", "A source"),
        ("collection-b", "source-b", "B source"),
        ("collection-o", "source-o", "Owned source"),
        ("collection-m", "source-m", "Missing catalogue source"),
    ] {
        md.collections.push(CollectionRecord {
            id: eid(id),
            source: source(src),
            display_name: name.into(),
            description: Presence::Value(tv("fixture collection")),
        });
    }
    for (id, col, name) in [
        ("component-a", "collection-a", "A components"),
        ("component-b", "collection-b", "B component"),
        ("component-o", "collection-o", "Owned revisions"),
        ("component-m", "collection-m", "Missing component"),
    ] {
        md.components.push(ComponentRecord {
            id: eid(id),
            collection: eid(col),
            native_name: name.into(),
            languages: vec!["jbo".into(), "en".into()],
        });
    }
    let mut source_by_capture = BTreeMap::<CaptureRef, SourceId>::new();
    for snap in lex.snapshots() {
        source_by_capture.insert(
            CaptureRef::from(snap),
            source_for_namespace(snap.namespace()),
        );
    }
    let mut caprows = Vec::<(CaptureRef, Vec<vlajmaji_lexicon::CapturedRecord>)>::new();
    for snap in lex.snapshots() {
        let cref = CaptureRef::from(snap);
        let rows = lex
            .records()
            .iter()
            .filter(|r| r.identity().snapshot() == snap)
            .cloned()
            .collect::<Vec<_>>();
        caprows.push((cref, rows));
    }
    let mut assertion_for_ref = BTreeMap::<OccurrenceRef, EntityId>::new();
    let mut entry_ids = BTreeMap::<(String, String), EntityId>::new();
    let mut entry_rows = Vec::new();
    let mut assertion_rows = Vec::new();
    let mut ix = 0usize;
    for (cref, rows) in &caprows {
        for row in rows {
            let oref = OccurrenceRef::from(row.identity());
            let src = source_by_capture[cref].clone();
            let word = row.word().unwrap_or("");
            let entry_id = if word.is_empty() {
                eid("entry-opaque")
            } else {
                let key = (src.to_string(), word.to_string());
                if let Some(id) = entry_ids.get(&key) {
                    id.clone()
                } else {
                    let id = eid(&format!("entry-{}", entry_ids.len() + 1));
                    entry_ids.insert(key, id.clone());
                    id
                }
            };
            if !entry_rows
                .iter()
                .any(|e: &vlajmaji_lexicon::EntryRecord| e.id == entry_id)
            {
                let class = row.field("word_type");
                let class2 = row.field("type_name");
                let native = match class {
                    vlajmaji_lexicon::FieldPresence::Value(v) => fixture_string(v),
                    _ => match class2 {
                        vlajmaji_lexicon::FieldPresence::Value(v) => fixture_string(v),
                        _ => None,
                    },
                };
                let sel = row.field("selmaho");
                let sel = match sel {
                    vlajmaji_lexicon::FieldPresence::Value(v) => {
                        fixture_string(v).map(Presence::Value)
                    }
                    vlajmaji_lexicon::FieldPresence::Null => Some(Presence::Null),
                    _ => None,
                }
                .unwrap_or(Presence::Missing);
                entry_rows.push(vlajmaji_lexicon::EntryRecord {
                    id: entry_id.clone(),
                    source: src.clone(),
                    spelling: if word.is_empty() {
                        Presence::Missing
                    } else {
                        Presence::Value(word.into())
                    },
                    language: Presence::Value("jbo".into()),
                    script: Presence::Missing,
                    native_word_class: native.map_or(Presence::Missing, Presence::Value),
                    selmaho: sel,
                    mapped_pos: Presence::Value(PosMapping {
                        value: "part-of-speech".into(),
                        rule: "explicit-fixture-map-v1".into(),
                    }),
                });
            }
            let aid = eid(&format!("assertion-{ix}"));
            assertion_for_ref.insert(oref.clone(), aid.clone());
            let def = row
                .definition()
                .map(|s| {
                    Presence::Value(TextValue {
                        text: s.into(),
                        language: Presence::Value("en".into()),
                        markup: Presence::Missing,
                    })
                })
                .unwrap_or(Presence::Missing);
            let elig = if word.is_empty() {
                Eligibility::Excluded {
                    reason: "unsupported opaque nonlexical row".into(),
                }
            } else {
                Eligibility::Lexical
            };
            assertion_rows.push(AssertionRecord {
                id: aid,
                entry: entry_id,
                occurrence: oref,
                native_sense_id: match row.field("definition_id") {
                    vlajmaji_lexicon::FieldPresence::Value(v) => Some(Presence::Value(v.into())),
                    _ => match row.field("definitionid") {
                        vlajmaji_lexicon::FieldPresence::Value(v) => {
                            Some(Presence::Value(v.into()))
                        }
                        _ => None,
                    },
                }
                .unwrap_or(Presence::Missing),
                language: Presence::Value("en".into()),
                definition: def,
                source_score: match row.field("score") {
                    vlajmaji_lexicon::FieldPresence::Value(v) => {
                        RawJson::new(v).ok().map(Presence::Value)
                    }
                    _ => None,
                }
                .unwrap_or(Presence::Missing),
                timestamps: ["time", "created_at"]
                    .into_iter()
                    .filter_map(|name| match row.field(name) {
                        vlajmaji_lexicon::FieldPresence::Value(v) => {
                            RawJson::new(v).ok().map(|value| NamedTimestamp {
                                field: name.into(),
                                value,
                                precision: TimePrecision::Unknown,
                            })
                        }
                        _ => None,
                    })
                    .collect(),
                eligibility: elig,
            });
            ix += 1;
        }
    }
    // The opaque row is a sixth A capture and a seventeenth A occurrence.
    let opaque_bytes = b"opaque-source-row\0retained".to_vec();
    let opaque_capture = RawCapture::new(
        "captured:a:opaque",
        CaptureMetadata {
            source_name: "A opaque capture".into(),
            format_label: "opaque-fixture".into(),
            locator: "memory:a:opaque".into(),
            revision: Presence::Value("fixture-v1".into()),
            captured_at: Presence::Missing,
            time_precision: TimePrecision::Unknown,
            notice_locator: Presence::Missing,
        },
        opaque_bytes,
        vec![RawOccurrence {
            locator: "row:1".into(),
            fields: vec![
                native_text("source-kind", "opaque"),
                native_json("unknown", r#"{"x":[null,true,12345678901234567890]}"#),
            ],
            raw_line: Some("opaque/nonlexical retained bytes".into()),
        }],
    );
    let oref = OccurrenceRef {
        capture: opaque_capture.identity.clone(),
        locator: "row:1".into(),
    };
    let opaque_entry = eid("entry-opaque");
    entry_rows.push(vlajmaji_lexicon::EntryRecord {
        id: opaque_entry.clone(),
        source: source("source-a"),
        spelling: Presence::Missing,
        language: Presence::Missing,
        script: Presence::Missing,
        native_word_class: Presence::Missing,
        selmaho: Presence::Missing,
        mapped_pos: Presence::Missing,
    });
    let opaque_aid = eid(&format!("assertion-{ix}"));
    assertion_rows.push(AssertionRecord {
        id: opaque_aid.clone(),
        entry: opaque_entry,
        occurrence: oref.clone(),
        native_sense_id: Presence::Missing,
        language: Presence::Missing,
        definition: Presence::Missing,
        source_score: Presence::Missing,
        timestamps: Vec::new(),
        eligibility: Eligibility::Excluded {
            reason: "unsupported opaque nonlexical row".into(),
        },
    });
    md.entries = entry_rows;
    md.assertions = assertion_rows;
    // Available release membership is explicit, preserving every raw row.
    let mut a_caps = Vec::new();
    let mut b_caps = Vec::new();
    let mut o_caps = Vec::new();
    let mut o2_caps = Vec::new();
    let mut by_src = BTreeMap::<String, Vec<EntityId>>::new();
    for a in &md.assertions {
        let src = source_by_capture
            .get(&a.occurrence.capture)
            .cloned()
            .unwrap_or_else(|| source("source-a"));
        by_src
            .entry(src.to_string())
            .or_default()
            .push(a.id.clone());
        match src.as_str() {
            "source-a" => a_caps.push(a.occurrence.capture.clone()),
            "source-b" => b_caps.push(a.occurrence.capture.clone()),
            _ => {
                if a.occurrence.capture.namespace.ends_with("r1") {
                    o_caps.push(a.occurrence.capture.clone())
                } else {
                    o2_caps.push(a.occurrence.capture.clone())
                }
            }
        }
    }
    a_caps.sort();
    a_caps.dedup();
    b_caps.sort();
    b_caps.dedup();
    o_caps.sort();
    o_caps.dedup();
    o2_caps.sort();
    o2_caps.dedup();
    let mut add_release = |id: &str,
                           component: &str,
                           captures: Vec<CaptureRef>,
                           assertions: Vec<EntityId>,
                           avail: Availability,
                           excluded: u64| {
        let n = assertions.len() as u64;
        md.releases.push(ReleaseRecord {
            id: eid(id),
            component: eid(component),
            distributor: None,
            upstream_revision: Presence::Value("fixture-v1".into()),
            distributor_revision: Presence::Missing,
            captured_at: Presence::Missing,
            time_precision: TimePrecision::Unknown,
            schema_version: 1,
            normalizer: "explicit-import".into(),
            normalizer_version: "1".into(),
            captures,
            assertions,
            accounting: ReleaseAccounting {
                input_occurrences: Some(n),
                normalized_occurrences: Some(n),
                query_excluded_occurrences: Some(excluded),
                unit: "occurrence".into(),
            },
            diagnostics: if excluded > 0 {
                vec![ReleaseDiagnostic {
                    occurrence: Some(oref.clone()),
                    code: "opaque-excluded".into(),
                    message: "retained but not lexical".into(),
                }]
            } else {
                Vec::new()
            },
            selected: id != "release-o2",
            availability: avail,
            content_notices: vec![ContentNotice {
                text: Presence::Value("fixture notice".into()),
                locator: Presence::Value("fixture://notice".into()),
                license: Presence::Missing,
            }],
            redistribution: if id == "release-a" {
                RedistributionStatus::Held
            } else if id == "release-b" || id == "release-m" {
                RedistributionStatus::Unknown
            } else {
                RedistributionStatus::Permitted
            },
            redistribution_evidence: if id == "release-o1" || id == "release-o2" {
                vec!["explicit fixture permission evidence".into()]
            } else {
                Vec::new()
            },
        });
    };
    let a_ids = by_src.remove("source-a").unwrap_or_default();
    let b_ids = by_src.remove("source-b").unwrap_or_default();
    let o_ids = by_src.remove("source-o").unwrap_or_default();
    let (o1_ids, o2_ids): (Vec<_>, Vec<_>) = o_ids.into_iter().partition(|id| {
        let a = md.assertions.iter().find(|a| &a.id == id).unwrap();
        a.occurrence.capture.namespace.ends_with("r1")
    });
    add_release(
        "release-a",
        "component-a",
        a_caps,
        a_ids,
        Availability::Available,
        1,
    );
    add_release(
        "release-b",
        "component-b",
        b_caps,
        b_ids,
        Availability::Available,
        0,
    );
    add_release(
        "release-o1",
        "component-o",
        o_caps,
        o1_ids,
        Availability::Available,
        0,
    );
    add_release(
        "release-o2",
        "component-o",
        o2_caps,
        o2_ids,
        Availability::Available,
        0,
    );
    let missing = CaptureRef {
        namespace: "catalogue:m:future".into(),
        sha256: "0".repeat(64),
    };
    add_release(
        "release-m",
        "component-m",
        vec![missing.clone()],
        Vec::new(),
        Availability::Missing,
        0,
    );
    if let Some(r) = md.releases.last_mut() {
        r.accounting = ReleaseAccounting {
            input_occurrences: None,
            normalized_occurrences: None,
            query_excluded_occurrences: None,
            unit: "occurrence".into(),
        };
        r.selected = false;
        r.diagnostics.push(ReleaseDiagnostic {
            occurrence: None,
            code: "not-captured".into(),
            message: "catalogue presence is not payload availability".into(),
        });
    }
    // One owned assertion per immutable revision; both remain addressable.
    let owned1 = md
        .assertions
        .iter()
        .find(|a| a.occurrence.capture.namespace.ends_with("r1"))
        .unwrap()
        .id
        .clone();
    let owned2 = md
        .assertions
        .iter()
        .find(|a| a.occurrence.capture.namespace.ends_with("r2"))
        .unwrap()
        .id
        .clone();
    let editor = vlajmaji_lexicon::ContributorId::new("editor-1")?;
    md.owned_revisions = vec![
        OwnedRevisionRecord {
            id: eid("revision-r1"),
            source: source("source-o"),
            parent: None,
            editor: editor.clone(),
            timestamp: Presence::Value("2026-10-24T00:00:00Z".into()),
            time_precision: TimePrecision::Second,
            assertions: vec![owned1.clone()],
            tombstones: Vec::new(),
            rationale: Presence::Value(tv("initial owned wording")),
        },
        OwnedRevisionRecord {
            id: eid("revision-r2"),
            source: source("source-o"),
            parent: Some(eid("revision-r1")),
            editor: editor.clone(),
            timestamp: Presence::Value("2026-10-24T01:00:00Z".into()),
            time_precision: TimePrecision::Second,
            assertions: vec![owned2.clone()],
            tombstones: vec![owned1.clone()],
            rationale: Presence::Value(tv("revised owned wording")),
        },
    ];
    md.owned_heads.push(OwnedHead {
        source: source("source-o"),
        revision: eid("revision-r1"),
    });
    md.editorial_senses
        .push(vlajmaji_lexicon::EditorialSenseRecord {
            id: eid("editorial-farvi"),
            assertions: vec![
                md.assertions
                    .iter()
                    .find(|a| {
                        a.occurrence.capture.namespace == "captured:a:d1"
                            && a.occurrence.locator == "/0"
                    })
                    .unwrap()
                    .id
                    .clone(),
                md.assertions
                    .iter()
                    .find(|a| a.occurrence.capture.namespace == "captured:b:d1")
                    .unwrap()
                    .id
                    .clone(),
            ],
            editor: editor.clone(),
            revision: eid("revision-r2"),
            rationale: tv("explicit editorial comparison; no default grouping"),
        });
    // Populate all eight context kinds using evidence on a real lexical assertion.
    let target = md
        .assertions
        .iter()
        .find(|a| {
            a.occurrence.capture.namespace == "captured:a:lexical" && a.occurrence.locator == "/0"
        })
        .unwrap()
        .clone();
    let rawrow = lex
        .records()
        .iter()
        .find(|r| {
            r.identity().snapshot().namespace() == "captured:a:lexical"
                && r.identity().locator() == "/0"
        })
        .unwrap();
    let field_idx = rawrow.fields().iter().position(|f| f.name() == "notes");
    let ev = evidence(target.occurrence.clone(), field_idx);
    let kinds = [
        ContextKind::Gloss,
        ContextKind::Note,
        ContextKind::Label,
        ContextKind::Register,
        ContextKind::Domain,
        ContextKind::Example,
        ContextKind::Translation,
        ContextKind::Citation,
    ];
    for (i, k) in kinds.into_iter().enumerate() {
        md.contexts.push(Context {
            id: eid(&format!("context-{i}")),
            assertion: target.id.clone(),
            kind: k,
            value: if i == 0 {
                Presence::Value(tv("short gloss"))
            } else if i == 1 {
                Presence::Value(tv("synthetic note"))
            } else if i == 2 {
                Presence::Null
            } else if i == 3 {
                Presence::Value(tv(""))
            } else {
                Presence::Value(tv(&format!("context {i}")))
            },
            translated_from: if i == 6 { Some(eid("context-4")) } else { None },
            evidence: ev.clone(),
        });
    }
    md.frames.push(ArgumentFrameRecord {
        id: eid("frame-si-e"),
        assertion: target.id.clone(),
        slots: vec![ArgumentSlot {
            order: 1,
            native_label: "x1".into(),
            description: Presence::Value(tv("snow")),
            grammatical_role: Presence::Value("agent".into()),
        }],
        evidence: evidence(
            target.occurrence.clone(),
            rawrow.fields().iter().position(|f| f.name() == "frame"),
        ),
    });
    let si_entry = md
        .entries
        .iter()
        .find(|e| e.spelling == Presence::Value("si'e".into()))
        .unwrap()
        .id
        .clone();
    let snime_entry = md
        .entries
        .iter()
        .find(|e| e.spelling == Presence::Value("snime".into()))
        .unwrap()
        .id
        .clone();
    let cpana_entry = md
        .entries
        .iter()
        .find(|e| e.spelling == Presence::Value("cpana".into()))
        .unwrap()
        .id
        .clone();
    let si_evidence = evidence(
        target.occurrence.clone(),
        rawrow.fields().iter().position(|f| f.name() == "variant"),
    );
    md.forms = vec![
        FormRecord {
            id: eid("form-variant"),
            entry: si_entry.clone(),
            spelling: "siʼe".into(),
            language: Presence::Value("jbo".into()),
            script: Presence::Missing,
            pronunciation: Presence::Missing,
            kind: FormKind::SourceVariant,
            standing: Presence::Value("reported".into()),
            components: Vec::new(),
            evidence: Some(si_evidence.clone()),
            computation: None,
        },
        FormRecord {
            id: eid("form-pronunciation"),
            entry: si_entry.clone(),
            spelling: "si'e".into(),
            language: Presence::Value("jbo".into()),
            script: Presence::Missing,
            pronunciation: Presence::Value(tv("siʔe")),
            kind: FormKind::SourcePronunciation,
            standing: Presence::Missing,
            components: Vec::new(),
            evidence: Some(evidence(
                target.occurrence.clone(),
                rawrow
                    .fields()
                    .iter()
                    .position(|f| f.name() == "pronunciation"),
            )),
            computation: None,
        },
        FormRecord {
            id: eid("form-assigned"),
            entry: snime_entry.clone(),
            spelling: "si'e".into(),
            language: Presence::Value("jbo".into()),
            script: Presence::Missing,
            pronunciation: Presence::Missing,
            kind: FormKind::AssignedRafsi,
            standing: Presence::Value("assigned".into()),
            components: Vec::new(),
            evidence: Some(evidence(
                OccurrenceRef::from(
                    lex.records()
                        .iter()
                        .find(|r| r.format() == SourceFormat::LlgRafsiRows)
                        .unwrap()
                        .identity(),
                ),
                Some(0),
            )),
            computation: None,
        },
        FormRecord {
            id: eid("form-reported"),
            entry: cpana_entry.clone(),
            spelling: "cpan".into(),
            language: Presence::Value("jbo".into()),
            script: Presence::Missing,
            pronunciation: Presence::Missing,
            kind: FormKind::SourceReportedForm,
            standing: Presence::Value("source-reported".into()),
            components: Vec::new(),
            evidence: Some(evidence(
                OccurrenceRef::from(
                    lex.records()
                        .iter()
                        .find(|r| {
                            r.word() == Some("cpana") && r.format() == SourceFormat::LaLojbanJson
                        })
                        .unwrap()
                        .identity(),
                ),
                Some(4),
            )),
            computation: None,
        },
        FormRecord {
            id: eid("form-stem"),
            entry: snime_entry.clone(),
            spelling: "snim".into(),
            language: Presence::Value("jbo".into()),
            script: Presence::Missing,
            pronunciation: Presence::Missing,
            kind: FormKind::ComputedStem,
            standing: Presence::Missing,
            components: Vec::new(),
            evidence: None,
            computation: Some(eid("computation-1")),
        },
        FormRecord {
            id: eid("form-components"),
            entry: cpana_entry.clone(),
            spelling: "cpana".into(),
            language: Presence::Value("jbo".into()),
            script: Presence::Missing,
            pronunciation: Presence::Missing,
            kind: FormKind::ComputedComponents,
            standing: Presence::Missing,
            components: vec![OrderedComponent {
                order: 1,
                spelling: "cpana".into(),
                target: Some(LocalRef::Entry(cpana_entry.clone())),
            }],
            evidence: None,
            computation: Some(eid("computation-1")),
        },
        FormRecord {
            id: eid("form-generated"),
            entry: snime_entry.clone(),
            spelling: "snimra".into(),
            language: Presence::Value("jbo".into()),
            script: Presence::Missing,
            pronunciation: Presence::Missing,
            kind: FormKind::GeneratedCandidate,
            standing: Presence::Missing,
            components: vec![OrderedComponent {
                order: 1,
                spelling: "snime".into(),
                target: Some(LocalRef::Entry(snime_entry.clone())),
            }],
            evidence: None,
            computation: Some(eid("computation-1")),
        },
    ];
    for (i, (locator, state)) in [
        ("/0", Presence::Null),
        (
            "/1",
            Presence::Value(TextValue {
                text: String::new(),
                language: Presence::Missing,
                markup: Presence::Missing,
            }),
        ),
        ("/8", Presence::Value(tv("derived source text"))),
    ]
    .into_iter()
    .enumerate()
    {
        let a = md
            .assertions
            .iter()
            .find(|a| {
                a.occurrence.capture.namespace == "captured:a:lexical"
                    && a.occurrence.locator == locator
            })
            .unwrap();
        let row = lex
            .records()
            .iter()
            .find(|r| {
                r.identity().snapshot().namespace() == "captured:a:lexical"
                    && r.identity().locator() == locator
            })
            .unwrap();
        let idx = row
            .fields()
            .iter()
            .position(|f| f.name() == "etymology")
            .unwrap();
        md.etymologies.push(EtymologyRecord {
            id: eid(&format!("etymology-{i}")),
            assertion: a.id.clone(),
            statement: state,
            native_kind: Presence::Value("source etymology".into()),
            components: Vec::new(),
            evidence: evidence(a.occurrence.clone(), Some(idx)),
        });
    }
    let a_farvi = md
        .assertions
        .iter()
        .find(|a| a.occurrence.capture.namespace == "captured:a:d1" && a.occurrence.locator == "/0")
        .unwrap()
        .clone();
    md.relations = vec![
        RelationRecord {
            id: eid("relation-see"),
            origin: LocalRef::Assertion(a_farvi.id.clone()),
            kind: RelationKind::SourceSeeAlso,
            direction: vlajmaji_lexicon::Direction::Forward,
            scope: RelationScope::Assertion,
            target: RelationTarget::Unknown,
            evidence: Some(evidence(a_farvi.occurrence.clone(), Some(0))),
            editorial_revision: None,
        },
        RelationRecord {
            id: eid("relation-missing-catalogue"),
            origin: LocalRef::Entry(si_entry.clone()),
            kind: RelationKind::SourceEtymology,
            direction: vlajmaji_lexicon::Direction::Unspecified,
            scope: RelationScope::Entry,
            target: RelationTarget::Catalog(CatalogTarget {
                source: source("source-m"),
                release: eid("release-m"),
                native_entry_id: Some("native-missing".into()),
                native_assertion_id: None,
                occurrence: Some(OccurrenceRef {
                    capture: missing.clone(),
                    locator: "/0".into(),
                }),
            }),
            evidence: Some(evidence(target.occurrence.clone(), Some(0))),
            editorial_revision: None,
        },
        RelationRecord {
            id: eid("relation-unresolved"),
            origin: LocalRef::Entry(si_entry.clone()),
            kind: RelationKind::Other("fixture".into()),
            direction: vlajmaji_lexicon::Direction::Bidirectional,
            scope: RelationScope::Entry,
            target: RelationTarget::Unresolved("unresolved target".into()),
            evidence: None,
            editorial_revision: None,
        },
        RelationRecord {
            id: eid("relation-editorial"),
            origin: LocalRef::Entry(si_entry),
            kind: RelationKind::EditorialLink,
            direction: vlajmaji_lexicon::Direction::Forward,
            scope: RelationScope::Editorial,
            target: RelationTarget::Local(LocalRef::Entry(snime_entry)),
            evidence: None,
            editorial_revision: Some(eid("revision-r2")),
        },
    ];
    md.computations.push(ComputationRecord {
        id: eid("computation-1"),
        tool: "fixture-tool".into(),
        tool_version: "1".into(),
        inputs: vec![
            InputRef::Occurrence(target.occurrence.clone()),
            InputRef::Literal("explicit input".into()),
        ],
        policy: "fixture-policy".into(),
        profile: Presence::Value("fixture-profile".into()),
        explanation: Presence::Value(tv("labelled computation; no automatic analysis")),
    });
    md.annotations.push(vlajmaji_lexicon::ComputedAnnotation {
        id: eid("annotation-1"),
        computation: eid("computation-1"),
        target: LocalRef::Entry(cpana_entry),
        kind: "candidate".into(),
        value: RawJson::new(r#"{"score":0.5,"labels":["synthetic",true]}"#)?,
        explanation: Presence::Value(tv("fixture output")),
    });
    md.extensions.push(ExtensionRecord {
        owner: target.id.clone(),
        name: "unknown-extension".into(),
        value: RawJson::new(r#"{"nested":[null,false,12345678901234567890]}"#)?,
        evidence: Some(evidence(target.occurrence.clone(), Some(0))),
    });
    // Preserve the legacy registry subset for adapter validation; add the opaque capture binding after adaptation.
    let mut base_md = md.clone();
    base_md.entries.retain(|e| e.id != eid("entry-opaque"));
    base_md.assertions.retain(|a| a.id != opaque_aid);
    if let Some(a) = base_md
        .releases
        .iter_mut()
        .find(|r| r.id == eid("release-a"))
    {
        a.captures.retain(|c| c != &opaque_capture.identity);
        a.assertions.retain(|id| id != &opaque_aid);
        a.accounting = ReleaseAccounting {
            input_occurrences: Some(16),
            normalized_occurrences: Some(16),
            query_excluded_occurrences: Some(0),
            unit: "occurrence".into(),
        };
        a.diagnostics.clear();
    }
    let mut doc = Dictionary::from_legacy_archive(&archive, registry.clone(), base_md)?.document();
    let mut rd = registry.document();
    rd.snapshots.push(SnapshotAttribution {
        snapshot: opaque_capture.identity.to_snapshot_identity()?,
        source: source("source-a"),
        distributor: None,
    });
    rd.snapshots.push(SnapshotAttribution {
        snapshot: missing.to_snapshot_identity()?,
        source: source("source-m"),
        distributor: None,
    });
    doc.registry = SourceRegistry::from_document(rd)?;
    doc.captures.push(opaque_capture);
    doc.metadata = md;
    let dict = Dictionary::from_document(doc)?;
    Ok(dict)
}

#[allow(dead_code)]
pub fn write_and_reload(path: &std::path::Path) -> Result<(), Box<dyn Error>> {
    if path.exists() {
        return Err("refusing to overwrite existing output".into());
    }
    let dict = complete_fixture()?;
    let bytes = dict.to_json_bytes()?;
    std::fs::write(path, &bytes)?;
    let reload = Dictionary::from_json_bytes(&std::fs::read(path)?)?;
    assert_eq!(reload.document(), dict.document());
    assert_eq!(
        reload
            .captures()
            .iter()
            .map(|c| c.occurrences.len())
            .sum::<usize>(),
        20
    );
    assert_eq!(reload.metadata().entries.len(), 14);
    assert_eq!(reload.metadata().assertions.len(), 20);
    assert_eq!(
        reload
            .metadata()
            .releases
            .iter()
            .find(|r| r.id == eid("release-a"))
            .unwrap()
            .accounting
            .input_occurrences,
        Some(17)
    );
    assert_eq!(reload.metadata().contexts.len(), 8);
    assert_eq!(reload.metadata().frames.len(), 1);
    assert_eq!(reload.metadata().forms.len(), 7);
    assert_eq!(reload.metadata().etymologies.len(), 3);
    assert_eq!(reload.metadata().relations.len(), 4);
    assert_eq!(reload.metadata().owned_revisions.len(), 2);
    assert_eq!(reload.metadata().editorial_senses.len(), 1);
    assert_eq!(reload.metadata().computations.len(), 1);
    assert_eq!(reload.metadata().annotations.len(), 1);
    assert_eq!(reload.metadata().extensions.len(), 1);
    Ok(())
}
