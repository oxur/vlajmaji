//! Public contract tests for sealed independent packs and pure composition.

#[path = "../examples/support/dictionary_fixture.rs"]
mod fixture_support;

use vlajmaji_lexicon::{
    AbbreviationKind, AliasStatus, Dictionary, DictionaryDocument, ExtensionRecord,
    JbotciImportOptions, LineageTarget, PackError, PackId, PackPayloadKind, PackSeal, Presence,
    RawJson, RegistryDocument, RegistryError, SourceAlias, SourceId, SourceLineage, SourceOrigin,
    SourceRecord, SourceRegistry, UnverifiedDictionaryDocument, compose_dictionaries,
    compose_unverified_documents, import_jbotci_export, load_sealed_release, seal_release,
};

fn digest(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn imported(raw: &[u8], revision: &str) -> Dictionary {
    let metadata = format!(
        "language_tag = \"en\"\nlanguage_realname = \"English\"\nsource_language_tag = \"jbo\"\nformat = \"json\"\npositive_scores_only = false\nfilename = \"dictionary-en.json\"\nmetadata_url = \"https://example.invalid/languages\"\ndownload_url = \"https://example.invalid/export\"\nlensisku_created_at = \"2026-10-04T03:07:08Z\"\nsha256 = \"{}\"\ndefinition_count = 1\nentry_count = 1\n",
        digest(raw)
    )
    .into_bytes();
    let options = JbotciImportOptions {
        revision: revision.into(),
        expected_raw_sha256: digest(raw),
        expected_metadata_sha256: digest(&metadata),
    };
    import_jbotci_export(raw, &metadata, &options)
        .expect("test jbotci input imports")
        .into_dictionary()
}

const REVISION_A: &str = "0000000000000000000000000000000000000000";
const REVISION_B: &str = "1111111111111111111111111111111111111111";
const RAW_A: &[u8] = br#"[{"word":"farvi","word_type":"gismu","definition":"to grow"}]"#;
const RAW_B: &[u8] =
    br#"[{"word":"farvi","word_type":"gismu","definition":"to grow and develop"}]"#;

fn extension_fixture(values: &[&str]) -> (Dictionary, Vec<ExtensionRecord>) {
    let mut document = imported(RAW_A, REVISION_A).into_document();
    let owner = document.metadata.assertions[0].id.clone();
    let extensions: Vec<_> = values
        .iter()
        .map(|value| ExtensionRecord {
            owner: owner.clone(),
            name: "same-native-field".into(),
            value: RawJson::new((*value).to_owned()).expect("literal JSON value is valid"),
            evidence: None,
        })
        .collect();
    document.metadata.extensions = extensions.clone();
    (
        Dictionary::from_document(document).expect("literal extension fixture is valid"),
        extensions,
    )
}

#[test]
fn seal_roundtrip_is_strict_and_inventory_ordered() {
    let dictionary = imported(RAW_A, REVISION_A);
    let sealed = seal_release(
        PackId::LensiskuJbotci,
        PackPayloadKind::Synthetic,
        &dictionary,
    )
    .expect("seal validated dictionary");
    let seal = PackSeal::from_json_bytes(&sealed.seal).expect("strict seal parses");
    assert_eq!(seal.source_ids[0].as_str(), "lensisku");
    assert_eq!(seal.release_ids.len(), 1);
    assert_eq!(seal.captures.len(), 2);
    assert_eq!(seal.release_bytes as usize, sealed.release.len());
    assert_eq!(
        load_sealed_release(PackId::LensiskuJbotci, &sealed.release, &sealed.seal)
            .expect("seal loads validated model")
            .to_json_bytes()
            .unwrap(),
        sealed.release
    );
}

#[test]
fn seal_rejects_wrong_pack_length_hash_version_inventory_and_bad_wire() {
    let dictionary = imported(RAW_A, REVISION_A);
    let sealed = seal_release(
        PackId::LensiskuJbotci,
        PackPayloadKind::Synthetic,
        &dictionary,
    )
    .unwrap();
    assert!(matches!(
        load_sealed_release(PackId::Llg, &sealed.release, &sealed.seal),
        Err(PackError::PackMismatch { .. })
    ));
    assert!(matches!(
        load_sealed_release(
            PackId::LensiskuJbotci,
            &sealed.release[..sealed.release.len() - 1],
            &sealed.seal
        ),
        Err(PackError::LengthMismatch { .. })
    ));
    let mut changed = sealed.release.clone();
    changed[0] ^= 1;
    assert!(matches!(
        load_sealed_release(PackId::LensiskuJbotci, &changed, &sealed.seal),
        Err(PackError::HashMismatch { .. })
    ));

    let mut wire: serde_json::Value = serde_json::from_slice(&sealed.seal).unwrap();
    wire["version"] = 2.into();
    assert!(matches!(
        PackSeal::from_json_bytes(&serde_json::to_vec(&wire).unwrap()),
        Err(PackError::UnsupportedVersion { .. })
    ));
    wire["version"] = 1.into();
    wire["registry_version"] = 2.into();
    assert!(matches!(
        PackSeal::from_json_bytes(&serde_json::to_vec(&wire).unwrap()),
        Err(PackError::UnsupportedVersion {
            kind: "registry",
            version: 2
        })
    ));
    wire["registry_version"] = 1.into();
    wire["source_ids"] = serde_json::json!([]);
    assert!(matches!(
        load_sealed_release(
            PackId::LensiskuJbotci,
            &sealed.release,
            &serde_json::to_vec(&wire).unwrap()
        ),
        Err(PackError::InventoryMismatch { field: "source" })
    ));
    assert!(matches!(
        PackSeal::from_json_bytes(br#"{"version":1,"version":1}"#),
        Err(PackError::InvalidSeal(_))
    ));
    wire["source_ids"] = serde_json::json!(["lensisku"]);
    wire["unexpected"] = true.into();
    assert!(matches!(
        PackSeal::from_json_bytes(&serde_json::to_vec(&wire).unwrap()),
        Err(PackError::InvalidSeal(_))
    ));
    let invalid_model = b"{}";
    let invalid_model_seal = serde_json::json!({
        "version":1,
        "pack_id":"lensisku-jbotci",
        "payload_kind":"synthetic",
        "dictionary_version":1,
        "registry_version":1,
        "release_sha256":digest(invalid_model),
        "release_bytes":invalid_model.len(),
        "source_ids":[],
        "release_ids":[],
        "captures":[]
    });
    assert!(matches!(
        load_sealed_release(
            PackId::LensiskuJbotci,
            invalid_model,
            &serde_json::to_vec(&invalid_model_seal).unwrap()
        ),
        Err(PackError::Dictionary(_))
    ));
}

#[test]
fn composition_preserves_distinct_distributions_and_exact_dedup() {
    let first = imported(RAW_A, REVISION_A);
    let second = imported(RAW_B, REVISION_B);
    let one = compose_dictionaries(vec![first.clone(), second.clone()]).expect("compose releases");
    let reversed = compose_dictionaries(vec![second, first.clone()]).expect("reverse compose");
    assert_eq!(one.registry().as_document().sources.len(), 1);
    assert_eq!(one.metadata().releases.len(), 2);
    assert_eq!(one.metadata().assertions.len(), 2);
    assert_eq!(one.registry().as_document().occurrences.len(), 2);
    assert_eq!(one.captures().len(), 4);
    assert_ne!(one.metadata().releases[0].id, one.metadata().releases[1].id);
    assert_eq!(
        one.metadata().releases[0].id,
        first.metadata().releases[0].id
    );
    assert_eq!(
        reversed.metadata().releases[0].id,
        one.metadata().releases[1].id
    );
    let repeated = compose_dictionaries(vec![first.clone(), first.clone()])
        .expect("exact repeated model deduplicates");
    assert_eq!(repeated, first);
}

#[test]
fn test_unverified_composition_keeps_graph_validation_explicit_and_deduplicates_inputs() {
    fn draft(dictionary: Dictionary) -> UnverifiedDictionaryDocument {
        let doc = dictionary.into_document();
        DictionaryDocument {
            version: doc.version,
            legacy_archive: doc.legacy_archive,
            registry: doc.registry.into_document(),
            captures: doc.captures,
            metadata: doc.metadata,
        }
    }
    let first = imported(RAW_A, REVISION_A);
    let expected = compose_dictionaries(vec![first.clone()])
        .unwrap()
        .to_json_bytes()
        .unwrap();
    let composed = compose_unverified_documents(vec![draft(first.clone()), draft(first)]).unwrap();
    assert_eq!(serde_json::to_vec(&composed).unwrap(), expected);
    let mut duplicate = draft(imported(RAW_A, REVISION_A));
    duplicate
        .metadata
        .entries
        .push(duplicate.metadata.entries[0].clone());
    assert!(matches!(
        compose_unverified_documents(vec![duplicate]),
        Err(PackError::Conflict {
            category: "entry",
            ..
        })
    ));

    let broken = imported(RAW_A, REVISION_A).into_document();
    let mut registry = broken.registry.into_document();
    registry.lineage.push(SourceLineage {
        source: registry.sources[0].id.clone(),
        upstream: LineageTarget::Known(SourceId::new("unknown-upstream").unwrap()),
        evidence: Some("tiny graph-only regression fixture".into()),
    });
    let broken = UnverifiedDictionaryDocument {
        version: broken.version,
        legacy_archive: broken.legacy_archive,
        registry,
        captures: broken.captures,
        metadata: broken.metadata,
    };
    let composed = compose_unverified_documents(vec![broken])
        .expect("unverified composition defers cross-record graph validation");
    assert!(SourceRegistry::from_document(composed.registry).is_err());
}

#[test]
fn test_unverified_composition_rejects_conflicting_alias_identity_and_deduplicates_equal_alias() {
    fn draft(dictionary: Dictionary) -> UnverifiedDictionaryDocument {
        let doc = dictionary.into_document();
        DictionaryDocument {
            version: doc.version,
            legacy_archive: doc.legacy_archive,
            registry: doc.registry.into_document(),
            captures: doc.captures,
            metadata: doc.metadata,
        }
    }
    let base = imported(RAW_A, REVISION_A);
    let source = base.registry().as_document().sources[0].id.clone();
    let with_alias = |evidence: &str| {
        let mut doc = draft(base.clone());
        doc.registry.aliases.push(SourceAlias {
            source: source.clone(),
            text: "shared selector".into(),
            status: AliasStatus::Active,
            evidence: Some(evidence.into()),
        });
        doc
    };
    let composed = compose_unverified_documents(vec![with_alias("same"), with_alias("same")])
        .expect("an exactly repeated alias identity and body is deduplicated");
    assert_eq!(composed.registry.aliases.len(), 1);
    assert!(matches!(
        compose_unverified_documents(vec![
            with_alias("first evidence"),
            with_alias("other evidence")
        ]),
        Err(PackError::Conflict {
            category: "alias",
            ..
        })
    ));
}

#[test]
fn test_unverified_composition_rejects_merged_active_selector_collision() {
    fn draft(dictionary: Dictionary) -> UnverifiedDictionaryDocument {
        let doc = dictionary.into_document();
        DictionaryDocument {
            version: doc.version,
            legacy_archive: doc.legacy_archive,
            registry: doc.registry.into_document(),
            captures: doc.captures,
            metadata: doc.metadata,
        }
    }
    let base = imported(RAW_A, REVISION_A);
    let first_source = base.registry().as_document().sources[0].id.clone();
    let second_source = SourceId::new("second-source").unwrap();
    let make_pair = |status| {
        let mut first = draft(base.clone());
        first.registry.aliases.push(SourceAlias {
            source: first_source.clone(),
            text: "shared selector".into(),
            status,
            evidence: Some("source-bounded evidence".into()),
        });
        let mut second = draft(Dictionary::new());
        second.registry.sources.push(SourceRecord {
            id: second_source.clone(),
            display_name: "Second source".into(),
            abbreviation: "SECOND".into(),
            abbreviation_kind: AbbreviationKind::ProjectAssigned,
            abbreviation_evidence: None,
            origin: SourceOrigin::Upstream,
        });
        second.registry.aliases.push(SourceAlias {
            source: second_source.clone(),
            text: "shared selector".into(),
            status,
            evidence: Some("source-bounded evidence".into()),
        });
        vec![first, second]
    };
    assert!(matches!(
        compose_unverified_documents(make_pair(AliasStatus::Active)),
        Err(PackError::Registry(RegistryError::AliasCollision { .. }))
    ));
    assert!(compose_unverified_documents(make_pair(AliasStatus::Retired)).is_ok());
}

#[test]
fn composition_preserves_ordered_extension_max_multiplicities() {
    let (distinct, distinct_rows) = extension_fixture(&["1", "2"]);
    assert_eq!(
        compose_dictionaries(vec![distinct.clone()])
            .unwrap()
            .metadata()
            .extensions,
        distinct_rows
    );
    assert_eq!(
        compose_dictionaries(vec![distinct.clone(), distinct])
            .unwrap()
            .metadata()
            .extensions,
        distinct_rows
    );

    let (one_repeat, one_repeat_rows) = extension_fixture(&["1", "1"]);
    assert_eq!(
        compose_dictionaries(vec![one_repeat.clone()])
            .unwrap()
            .metadata()
            .extensions,
        one_repeat_rows
    );
    assert_eq!(
        compose_dictionaries(vec![one_repeat.clone(), one_repeat])
            .unwrap()
            .metadata()
            .extensions,
        one_repeat_rows
    );

    let (repeat_before_new_repeat, repeated_rows) = extension_fixture(&["1", "1", "2", "2"]);
    assert_eq!(
        compose_dictionaries(vec![repeat_before_new_repeat.clone()])
            .unwrap()
            .metadata()
            .extensions,
        repeated_rows
    );
    assert_eq!(
        compose_dictionaries(vec![
            repeat_before_new_repeat.clone(),
            repeat_before_new_repeat
        ])
        .unwrap()
        .metadata()
        .extensions,
        repeated_rows
    );

    let (repeat_before_new_self, self_rows) = extension_fixture(&["1", "1", "2"]);
    assert_eq!(
        compose_dictionaries(vec![repeat_before_new_self.clone(), repeat_before_new_self])
            .unwrap()
            .metadata()
            .extensions,
        self_rows
    );

    let (first, first_rows) = extension_fixture(&["1", "1", "2"]);
    let (second, second_rows) = extension_fixture(&["1", "1", "1", "2", "2", "2"]);
    let expected_maxima = vec![
        first_rows[0].clone(),
        first_rows[1].clone(),
        first_rows[2].clone(),
        second_rows[2].clone(),
        second_rows[4].clone(),
        second_rows[5].clone(),
    ];
    assert_eq!(
        compose_dictionaries(vec![first, second])
            .unwrap()
            .metadata()
            .extensions,
        expected_maxima
    );
}

#[test]
fn composition_rejects_conflicting_source_and_release_identities() {
    let first = imported(RAW_A, REVISION_A);
    let mut source_conflict_doc = first.clone().into_document();
    let mut registry: RegistryDocument = source_conflict_doc.registry.into_document();
    registry.sources[0].display_name = "Different source body".into();
    source_conflict_doc.registry = SourceRegistry::from_document(registry).unwrap();
    let source_conflict = Dictionary::from_document(source_conflict_doc).unwrap();
    assert!(matches!(
        compose_dictionaries(vec![first.clone(), source_conflict]),
        Err(PackError::Conflict {
            category: "source",
            ..
        })
    ));

    let mut release_conflict_doc = first.clone().into_document();
    release_conflict_doc.metadata.releases[0].normalizer_version = "other".into();
    let release_conflict = Dictionary::from_document(release_conflict_doc).unwrap();
    assert!(matches!(
        compose_dictionaries(vec![first, release_conflict]),
        Err(PackError::Conflict {
            category: "release",
            ..
        })
    ));
}

#[test]
fn composition_rejects_conflicting_capture_contributor_and_assertion_bodies() {
    let first = imported(RAW_A, REVISION_A);
    let mut capture_doc = first.clone().into_document();
    capture_doc.captures[0].metadata.source_name = "changed capture metadata".into();
    let capture_conflict = Dictionary::from_document(capture_doc).unwrap();
    assert!(matches!(
        compose_dictionaries(vec![first.clone(), capture_conflict]),
        Err(PackError::Conflict {
            category: "capture",
            ..
        })
    ));

    let raw_with_user = br#"[{"word":"farvi","word_type":"gismu","definition":"to grow","user":{"username":"alice"}}]"#;
    let with_user = imported(raw_with_user, REVISION_A);
    let mut contributor_doc = with_user.clone().into_document();
    let mut registry = contributor_doc.registry.into_document();
    registry.contributors[0].display_name = "Different contributor name".into();
    contributor_doc.registry = SourceRegistry::from_document(registry).unwrap();
    let contributor_conflict = Dictionary::from_document(contributor_doc).unwrap();
    assert!(matches!(
        compose_dictionaries(vec![with_user, contributor_conflict]),
        Err(PackError::Conflict {
            category: "contributor",
            ..
        })
    ));

    let mut assertion_doc = first.clone().into_document();
    assertion_doc.metadata.assertions[0].native_sense_id = Presence::Value("other".into());
    let assertion_conflict = Dictionary::from_document(assertion_doc).unwrap();
    assert!(matches!(
        compose_dictionaries(vec![first, assertion_conflict]),
        Err(PackError::Conflict {
            category: "assertion",
            ..
        })
    ));
}

#[test]
fn empty_composition_is_the_empty_validated_dictionary() {
    assert_eq!(compose_dictionaries(Vec::new()).unwrap(), Dictionary::new());
}

#[test]
fn complete_typed_model_composes_without_dropping_any_metadata_class() {
    let fixture = fixture_support::complete_fixture().unwrap();
    assert!(matches!(
        compose_dictionaries(vec![fixture.clone()]),
        Err(PackError::LegacyArchiveUnsupported)
    ));
    let mut document = fixture.into_document();
    document.legacy_archive = None;
    let original_extension = document.metadata.extensions[0].clone();
    let alternative_extension = ExtensionRecord {
        value: RawJson::new(r#"{"alternative":true}"#).unwrap(),
        ..original_extension.clone()
    };
    document
        .metadata
        .extensions
        .push(alternative_extension.clone());
    let independent = Dictionary::from_document(document).unwrap();
    let idempotent = compose_dictionaries(vec![independent.clone(), independent.clone()])
        .expect("composing a model with itself is idempotent");
    assert_eq!(idempotent, independent);
    let mut evidence_alternative = alternative_extension.clone();
    evidence_alternative.evidence.as_mut().unwrap().field_index = Some(1);
    let mut second_document = independent.clone().into_document();
    second_document
        .metadata
        .extensions
        .extend([evidence_alternative.clone(), alternative_extension.clone()]);
    let second = Dictionary::from_document(second_document).unwrap();
    let combined = compose_dictionaries(vec![independent, second])
        .expect("complete non-archive model alternatives compose");
    assert_eq!(
        combined.metadata().extensions,
        vec![
            original_extension,
            alternative_extension.clone(),
            evidence_alternative,
            alternative_extension
        ]
    );
    assert_eq!(combined.metadata().code_notices.len(), 1);
    assert_eq!(combined.metadata().collections.len(), 4);
    assert_eq!(combined.metadata().components.len(), 4);
    assert_eq!(combined.metadata().releases.len(), 5);
    assert_eq!(combined.metadata().entries.len(), 14);
    assert_eq!(combined.metadata().assertions.len(), 20);
    assert_eq!(combined.metadata().contexts.len(), 8);
    assert_eq!(combined.metadata().frames.len(), 1);
    assert_eq!(combined.metadata().forms.len(), 7);
    assert_eq!(combined.metadata().etymologies.len(), 3);
    assert_eq!(combined.metadata().relations.len(), 4);
    assert_eq!(combined.metadata().owned_revisions.len(), 2);
    assert_eq!(combined.metadata().owned_heads.len(), 1);
    assert_eq!(combined.metadata().editorial_senses.len(), 1);
    assert_eq!(combined.metadata().computations.len(), 1);
    assert_eq!(combined.metadata().annotations.len(), 1);
    assert_eq!(combined.metadata().extensions.len(), 4);
}
