use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, path::PathBuf};
use vlajmaji_lexicon::{
    Dictionary, JbotciImportOptions, LaLojbanImportOptions, LlgImportOptions,
    SutysiskuImportOptions, import_jbotci_export, import_la_lojban_export,
    import_llg_lists, import_sutysisku_export,
};
use vlajmaji_lexicon::pack::compose_dictionaries;
use vlajmaji_lexicon::Presence;
use vlajmaji_lexicon::{compose_unverified_documents, UnverifiedDictionaryDocument};

fn sha(bytes: &[u8]) -> String { format!("{:x}", Sha256::digest(bytes)) }
fn draft(d: Dictionary) -> UnverifiedDictionaryDocument {
    let doc = d.into_document();
    UnverifiedDictionaryDocument {
        version: doc.version,
        legacy_archive: doc.legacy_archive,
        registry: doc.registry.into_document(),
        captures: doc.captures,
        metadata: doc.metadata,
    }
}
fn categories(doc: &UnverifiedDictionaryDocument) -> BTreeMap<String, BTreeMap<String, usize>> {
    let root = serde_json::to_value(doc).expect("serialize fixture model");
    let mut result = BTreeMap::new();
    for family in ["metadata", "registry"] {
        let object = root.get(family).and_then(Value::as_object).expect("family object");
        for (name, rows) in object {
            if let Some(array) = rows.as_array() {
                let counts = result.entry(format!("{family}.{name}")).or_insert_with(BTreeMap::new);
                for row in array {
                    let bytes = serde_json::to_vec(row).expect("serialize inventory row");
                    *counts.entry(sha(&bytes)).or_insert(0usize) += 1;
                }
            }
        }
    }
    let captures = root.get("captures").and_then(Value::as_array).expect("captures");
    let counts = result.entry("captures".to_owned()).or_insert_with(BTreeMap::new);
    for row in captures {
        let bytes = serde_json::to_vec(row).expect("serialize capture row");
        *counts.entry(sha(&bytes)).or_insert(0usize) += 1;
    }
    result
}
fn expected_max(inputs: &[BTreeMap<String, BTreeMap<String, usize>>]) -> BTreeMap<String, BTreeMap<String, usize>> {
    let mut expected = BTreeMap::new();
    for input in inputs {
        for (category, rows) in input {
            let target = expected.entry(category.clone()).or_insert_with(BTreeMap::new);
            for (digest, count) in rows {
                let value = target.entry(digest.clone()).or_insert(0usize);
                *value = (*value).max(*count);
            }
        }
    }
    expected
}
fn mutate_category(map: &mut BTreeMap<String, BTreeMap<String, usize>>, category: &str) {
    let rows = map.get_mut(category).expect("nonempty category");
    let first = rows.values_mut().next().expect("category row");
    *first += 1;
}
fn main() {
    let dir = PathBuf::from(std::env::args_os().nth(1).expect("fixture directory"));
    let lens = std::fs::read(dir.join("lensisku.json")).expect("lens fixture");
    let llg_raw = std::fs::read(dir.join("llg-valid-row.txt")).expect("llg fixture");
    let sut_raw = std::fs::read(dir.join("sutysisku.json")).expect("sut fixture");
    let revision = "0".repeat(40);
    let meta = format!(
        "language_tag = \"en\"\nlanguage_realname = \"English\"\nsource_language_tag = \"jbo\"\nformat = \"json\"\npositive_scores_only = false\nfilename = \"dictionary-en.json\"\nmetadata_url = \"https://example.invalid/languages\"\ndownload_url = \"https://example.invalid/synthetic\"\nlensisku_created_at = \"2026-10-04T03:07:08Z\"\nsha256 = \"{}\"\ndefinition_count = 1\nentry_count = 1\n",
        sha(&lens)
    );
    let jb = import_jbotci_export(&lens, meta.as_bytes(), &JbotciImportOptions {
        revision: revision.clone(),
        expected_raw_sha256: sha(&lens),
        expected_metadata_sha256: sha(meta.as_bytes()),
    }).expect("jbotci import").into_dictionary();
    let la = import_la_lojban_export(&lens, &LaLojbanImportOptions {
        revision: revision.clone(),
        expected_raw_sha256: sha(&lens),
        raw_locator: Some("https://example.invalid/la".into()),
        notice_locator: Some("https://example.invalid/code-notice".into()),
    }).expect("la import").into_dictionary();
    let notice = b"Explicit synthetic notice";
    let llg = import_llg_lists(&llg_raw, b"", b"", notice, &LlgImportOptions {
        expected_gismu_sha256: sha(&llg_raw),
        expected_cmavo_sha256: sha(b""),
        expected_rafsi_sha256: sha(b""),
        expected_notice_sha256: sha(notice),
        base_locator: "https://example.invalid/llg".into(),
    }).expect("llg import").into_dictionary();
    let sut = import_sutysisku_export(&sut_raw, &SutysiskuImportOptions {
        revision,
        expected_raw_sha256: sha(&sut_raw),
        raw_locator: Some("https://example.invalid/sut".into()),
        notice_locator: Some("https://example.invalid/code-notice".into()),
    }).expect("sut import").into_dictionary();

    let models = vec![jb, llg, la, sut];
    let drafts: Vec<_> = models.clone().into_iter().map(draft).collect();
    let strict = draft(compose_dictionaries(models).expect("strict composition"));
    let actual = compose_unverified_documents(drafts.clone()).expect("unverified composition");
    assert_eq!(actual, strict, "strict and unverified serialization must agree");
    let inventories: Vec<_> = drafts.iter().map(categories).collect();
    let expected = expected_max(&inventories);
    let actual_inventory = categories(&actual);
    assert_eq!(actual_inventory, expected, "complete category inventory, including empty categories");

    let mut reversed = drafts.clone();
    reversed.reverse();
    let reversed_result = compose_unverified_documents(reversed).expect("reverse composition");
    assert_eq!(categories(&reversed_result), expected, "input reorder preserves inventory");

    let mut missing_empty = expected.clone();
    let empty_category = expected.iter().find(|(_, rows)| rows.is_empty()).map(|(name, _)| name.clone()).expect("at least one empty category");
    missing_empty.remove(&empty_category);
    assert_ne!(actual_inventory, missing_empty, "comparator distinguishes absent from empty categories");
    let nonempty = expected.iter().find(|(_, rows)| !rows.is_empty()).map(|(name, _)| name.clone()).expect("nonempty category");
    let mut wrong_count = expected.clone();
    mutate_category(&mut wrong_count, &nonempty);
    assert_ne!(actual_inventory, wrong_count, "comparator detects changed multiplicity");
    let mut changed_presence = actual.clone();
    let entry = changed_presence.metadata.entries.iter_mut().find(|row| matches!(row.spelling, Presence::Value(_))).expect("entry with present spelling");
    entry.spelling = Presence::Missing;
    assert_ne!(actual_inventory, categories(&changed_presence), "comparator detects Missing vs Value presence state");

    let totals: BTreeMap<_, _> = inventories.iter().enumerate().map(|(i, map)| {
        (["jbotci", "llg", "la-lojban", "sutysisku"][i], map.iter().map(|(_, rows)| rows.values().sum::<usize>()).sum::<usize>())
    }).collect();
    let category_rows: BTreeMap<_, _> = actual_inventory.iter().map(|(name, rows)| (name.clone(), rows.values().sum::<usize>())).collect();
    let input_category_rows: Vec<_> = inventories.iter().map(|inventory| inventory.iter().map(|(name, rows)| (name.clone(), rows.values().sum::<usize>())).collect::<BTreeMap<_, _>>()).collect();
    let input_assertions: Vec<_> = drafts.iter().map(|doc| serde_json::json!({
        "assertions": doc.metadata.assertions.len(),
        "lexical": doc.metadata.assertions.iter().filter(|row| row.eligibility == vlajmaji_lexicon::Eligibility::Lexical).count(),
        "excluded": doc.metadata.assertions.iter().filter(|row| row.eligibility != vlajmaji_lexicon::Eligibility::Lexical).count(),
        "sources": doc.registry.sources.iter().map(|row| row.id.as_str()).collect::<Vec<_>>()
    })).collect();
    let report = serde_json::json!({
        "kind":"s02-tiny-census",
        "inputs":{
            "lensisku_bytes":lens.len(),
            "lensisku_sha256":sha(&lens),
            "llg_bytes":llg_raw.len(),
            "llg_sha256":sha(&llg_raw),
            "sutysisku_bytes":sut_raw.len(),
            "sutysisku_sha256":sha(&sut_raw)
        },
        "input_category_row_totals":totals,
        "input_category_row_counts":input_category_rows,
        "input_assertion_eligibility_and_sources":input_assertions,
        "composed_category_row_totals":category_rows,
        "composed_assertions":actual.metadata.assertions.len(),
        "composed_lexical_assertions":actual.metadata.assertions.iter().filter(|r| r.eligibility == vlajmaji_lexicon::Eligibility::Lexical).count(),
        "composed_sources":actual.registry.sources.iter().map(|r| r.id.as_str()).collect::<Vec<_>>(),
        "comparator_controls_passed":6,
        "all_categories_including_empty":true
    });
    println!("{}", serde_json::to_string_pretty(&report).expect("render report"));
}
