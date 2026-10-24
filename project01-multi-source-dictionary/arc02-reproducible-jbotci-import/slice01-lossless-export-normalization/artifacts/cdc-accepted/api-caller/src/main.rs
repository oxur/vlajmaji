use serde_json::Value;
use std::{error::Error, fs, io::Write, path::Path};
use vlajmaji_lexicon::{Dictionary, JbotciImportError, JbotciImportOptions, import_jbotci_export};

fn options(case: &Value) -> JbotciImportOptions {
    JbotciImportOptions {
        revision: case["revision"].as_str().unwrap().into(),
        expected_raw_sha256: case["raw_sha256"].as_str().unwrap().into(),
        expected_metadata_sha256: case["metadata_sha256"].as_str().unwrap().into(),
    }
}
fn category(e: &JbotciImportError) -> &'static str {
    match e {
        JbotciImportError::InvalidOptions(_) => "options",
        JbotciImportError::HashMismatch { .. } => "hash",
        JbotciImportError::InvalidUtf8 { .. } => "utf8",
        JbotciImportError::InvalidJson(_) => "json",
        JbotciImportError::InvalidMetadata(_) => "metadata",
        JbotciImportError::UnsupportedMetadata(_) => "unsupported_metadata",
        JbotciImportError::Registry(_) => "registry",
        JbotciImportError::Dictionary(_) => "dictionary",
        _ => "future_variant",
    }
}
fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args().collect();
    if args[1] == "--full" {
        let raw = fs::read(&args[2])?;
        let metadata = fs::read(&args[3])?;
        let result = import_jbotci_export(&raw, &metadata, &JbotciImportOptions::default())?;
        let dictionary = result.dictionary();
        assert_eq!(dictionary.captures().len(), 2);
        let rc = dictionary
            .captures()
            .iter()
            .find(|c| c.identity.namespace == "captured:lensisku:jbotci:en")
            .ok_or("raw capture missing")?;
        let mc = dictionary
            .captures()
            .iter()
            .find(|c| c.identity.namespace == "captured:lensisku:jbotci:en:metadata")
            .ok_or("metadata capture missing")?;
        assert!(rc.bytes == raw, "full raw capture bytes changed");
        assert!(mc.bytes == metadata, "full supplier metadata bytes changed");
        assert_eq!(rc.occurrences.len(), 33_526);
        assert!(mc.occurrences.is_empty());
        assert_eq!(
            rc.occurrences.iter().map(|r| r.fields.len()).sum::<usize>(),
            278_254
        );
        assert_eq!(dictionary.metadata().assertions.len(), 33_526);
        assert_eq!(result.report().input_rows, 33_526);
        assert_eq!(result.report().native_fields, 278_254);
        let canonical = dictionary.to_json_bytes()?;
        assert!(
            Dictionary::from_json_bytes(&canonical)? == *dictionary,
            "full canonical reload changed model"
        );
        let report = serde_json::to_vec_pretty(result.report())?;
        let mut f = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&args[4])?;
        f.write_all(&canonical)?;
        f.flush()?;
        let mut f = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&args[5])?;
        f.write_all(&report)?;
        f.flush()?;
        println!(
            "PASS full outside API: raw_bytes={} metadata_bytes={} rows={} fields={} canonical_bytes={}",
            raw.len(),
            metadata.len(),
            rc.occurrences.len(),
            result.report().native_fields,
            canonical.len()
        );
        return Ok(());
    }
    let root = Path::new(&args[1]);
    let manifest: Value = serde_json::from_slice(&fs::read(root.join("manifest.json"))?)?;
    let cases = manifest.as_array().unwrap();
    let mut negatives = 0;
    let mut disagreements = Vec::<String>::new();
    let mut positives = 0;
    let mut base: Option<Dictionary> = None;
    for case in cases {
        let name = case["name"].as_str().unwrap();
        let raw = fs::read(root.join(name).join("raw.json"))?;
        let meta = fs::read(root.join(name).join("meta.toml"))?;
        let result = import_jbotci_export(&raw, &meta, &options(case));
        if let Some(expected) = case["error"].as_str() {
            let e = match result {
                Err(e) => e,
                Ok(_) => panic!("{name}: unexpected success"),
            };
            if category(&e) != expected {
                eprintln!(
                    "CONTROL_FAILED {name}: expected {expected}, got {} ({e})",
                    category(&e)
                );
                disagreements.push(format!("{name}: expected {expected}, got {}", category(&e)));
            }
            match &e {
                JbotciImportError::InvalidUtf8 { input } => {
                    assert_eq!(
                        input,
                        if name == "raw-utf8" {
                            "raw"
                        } else {
                            "metadata"
                        },
                        "{name} UTF-8 input label"
                    );
                }
                JbotciImportError::HashMismatch {
                    input,
                    expected,
                    actual,
                } => {
                    let is_meta = name == "wrong-metadata-hash";
                    assert_eq!(
                        input,
                        if is_meta { "metadata" } else { "raw" },
                        "{name} hash input label"
                    );
                    assert_eq!(
                        expected,
                        "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
                        "{name} supplied mismatching digest"
                    );
                    assert_eq!(
                        actual,
                        case[if is_meta {
                            "actual_metadata_sha256"
                        } else {
                            "actual_raw_sha256"
                        }]
                        .as_str()
                        .unwrap(),
                        "{name} independently computed actual digest"
                    );
                }
                _ => {}
            }
            negatives += 1;
            continue;
        }
        let result = result?;
        assert_eq!(
            result.report().revision,
            options(case).revision,
            "{name} supplied revision in report"
        );
        assert_eq!(result.report().version, 1, "{name} report version");
        positives += 1;
        let dictionary = result.dictionary();
        assert_eq!(
            dictionary.metadata().assertions.len(),
            case["rows"].as_u64().unwrap() as usize,
            "{name}"
        );
        let projected = serde_json::to_value(dictionary)?;
        let pinned_notice = "https://github.com/int19h/jbotci/blob/3d9fd41d9ed32de26311ee27ec86b6b165985aa1/THIRD-PARTY-NOTICES.md#L242-L252";
        assert_eq!(projected["metadata"]["code_notices"][0]["locator"]["Value"], "LICENSE", "{name} truthful code locator");
        assert_eq!(projected["metadata"]["releases"][0]["content_notices"][0]["locator"]["Value"], pinned_notice, "{name} actual pinned supplier notice");
        assert_eq!(projected["metadata"]["releases"][0]["redistribution"], "Held", "{name} redistribution boundary");
        assert!(projected["registry"]["lineage"][0]["evidence"].to_string().contains(pinned_notice), "{name} lineage actual witness");
        let raw_capture = projected["captures"].as_array().unwrap().iter().find(|c|c["identity"]["namespace"]=="captured:lensisku:jbotci:en").unwrap();
        assert_eq!(raw_capture["metadata"]["notice_locator"]["Value"], pinned_notice, "{name} capture actual witness");
        let canonical = dictionary.to_json_bytes()?;
        if name == "duplicate-native-and-complete" {
            let a = dictionary.metadata();
            assert_ne!(
                a.entries[0].id, a.entries[1].id,
                "duplicate entries remain distinct"
            );
            assert_ne!(
                a.assertions[0].id, a.assertions[1].id,
                "duplicate assertions remain distinct"
            );
            assert_ne!(
                a.assertions[0].occurrence, a.assertions[1].occurrence,
                "duplicate occurrences remain distinct"
            );
            base = Some(dictionary.clone());
        }
        if name == "changed-whitespace" {
            let original = base.as_ref().unwrap();
            assert_ne!(
                original.captures()[0].identity,
                dictionary.captures()[0].identity,
                "byte change is a new capture"
            );
            for (a, b) in original
                .metadata()
                .entries
                .iter()
                .zip(&dictionary.metadata().entries)
            {
                assert_ne!(a.id, b.id, "new bytes produce new entry identity");
            }
            for (a, b) in original
                .metadata()
                .assertions
                .iter()
                .zip(&dictionary.metadata().assertions)
            {
                assert_ne!(a.id, b.id, "new bytes produce new assertion identity");
            }
        }
        assert_eq!(
            Dictionary::from_json_bytes(&canonical)?,
            *dictionary,
            "{name} canonical reload"
        );
        let again = import_jbotci_export(&raw, &meta, &options(case))?;
        assert_eq!(
            again.dictionary(),
            dictionary,
            "{name} deterministic complete model"
        );
        assert_eq!(
            again.dictionary().to_json_bytes()?,
            canonical,
            "{name} deterministic wire"
        );
        assert_eq!(
            serde_json::to_value(again.report())?,
            serde_json::to_value(result.report())?,
            "{name} deterministic report"
        );
        let mut other = options(case);
        other.revision = "1111111111111111111111111111111111111111".into();
        let revised = import_jbotci_export(&raw, &meta, &other)?;
        assert_eq!(
            revised.report().revision,
            other.revision,
            "{name} revised report provenance"
        );
        assert_ne!(
            dictionary.metadata().releases[0].id,
            revised.dictionary().metadata().releases[0].id,
            "{name} revision changes release"
        );
        assert_ne!(
            canonical,
            revised.dictionary().to_json_bytes()?,
            "{name} revision changes wire"
        );
        let ra = dictionary.registry().document();
        let rb = revised.dictionary().registry().document();
        assert_eq!(ra.sources, rb.sources, "{name} source identities");
        assert_eq!(
            ra.distributors, rb.distributors,
            "{name} distributor identities"
        );
        assert_eq!(
            ra.contributors, rb.contributors,
            "{name} contributor identities"
        );
        assert_eq!(ra.snapshots, rb.snapshots, "{name} snapshot attribution");
        assert_eq!(
            ra.occurrences, rb.occurrences,
            "{name} occurrence attribution"
        );
        for (a, b) in dictionary
            .captures()
            .iter()
            .zip(revised.dictionary().captures())
        {
            assert_eq!(a.identity, b.identity, "{name} same byte snapshot");
            assert_eq!(a.bytes, b.bytes, "{name} same source bytes");
            assert_eq!(a.occurrences, b.occurrences, "{name} same raw rows");
        }
        let a = dictionary.metadata();
        let b = revised.dictionary().metadata();
        assert_eq!(a.entries, b.entries, "{name} same entry identities");
        assert_eq!(
            a.assertions, b.assertions,
            "{name} same occurrence/assertion identities"
        );
        assert_eq!(a.contexts, b.contexts, "{name} same context projections");
        assert_eq!(a.forms, b.forms, "{name} same source-reported forms");
        assert_eq!(
            a.etymologies, b.etymologies,
            "{name} same etymology records"
        );
        let ea: Vec<_> = a
            .extensions
            .iter()
            .filter(|e| e.owner != a.releases[0].id)
            .collect();
        let eb: Vec<_> = b
            .extensions
            .iter()
            .filter(|e| e.owner != b.releases[0].id)
            .collect();
        assert_eq!(ea, eb, "{name} same per-row extension values/identities");
        if let Some(field) = case["check_field"].as_str() {
            let expected = case["diagnostic_expected"].as_bool().unwrap();
            for assertion in &a.assertions {
                assert!(
                    matches!(
                        assertion.eligibility,
                        vlajmaji_lexicon::Eligibility::Lexical
                    ),
                    "{name}: optional field excluded row"
                );
                if field == "selmaho" {
                    let raw_rows: Value = serde_json::from_slice(&raw)?;
                    let index: usize = assertion.occurrence.locator.strip_prefix('/').unwrap().parse()?;
                    let expected_projection = match raw_rows[index].get("selmaho") {
                        None => serde_json::json!("Missing"),
                        Some(Value::Null) => serde_json::json!("Null"),
                        Some(Value::String(text)) => serde_json::json!({"Value":text}),
                        Some(_) => serde_json::json!("Missing"),
                    };
                    let entry = a.entries.iter().find(|e| e.id == assertion.entry).unwrap();
                    assert_eq!(serde_json::to_value(&entry.selmaho)?,expected_projection,"{name} selmaho typed presence");
                }
                let diagnosed = a.releases[0].diagnostics.iter().any(|d| {
                    d.occurrence.as_ref() == Some(&assertion.occurrence)
                        && (d.code.contains(field) || d.message.contains(field))
                });
                if diagnosed != expected {
                    let issue = format!(
                        "{name} {}: field {field} expected diagnostic={expected}, got {diagnosed}",
                        assertion.occurrence.locator
                    );
                    eprintln!("CONTROL_FAILED {issue}");
                    disagreements.push(issue);
                }
            }
            if let Some(native) = case["expected_native_ids"].as_array() {
                let r = dictionary.registry().document();
                for (i, expected) in native.iter().enumerate() {
                    let at = r
                        .occurrences
                        .iter()
                        .find(|a| a.occurrence.locator() == format!("/{i}"))
                        .unwrap();
                    assert_eq!(
                        serde_json::to_value(&at.native_assertion_id)?,
                        *expected,
                        "{name}: native identity"
                    );
                }
            }
        }
        if let Some(key) = case["unknown_metadata_key"].as_str() {
            assert_eq!(result.report().diagnostics.get("metadata_unknown_key"), Some(&1), "{name} metadata diagnostic count");
            assert!(a.releases[0].diagnostics.iter().any(|d| d.code == "metadata_unknown_key" && d.message.contains(key) && d.occurrence.is_none()), "{name} metadata diagnostic scope");
        }
        if case["count_mismatch"] == true {
            assert!(
                !result.report().definition_count_matches && !result.report().entry_count_matches
            );
        }
        // Exercise ownership transfer as a public consumer, independently of the CLI.
        assert_eq!(
            again.into_dictionary(),
            *dictionary,
            "{name} owned result transfer"
        );
        println!("positive {name}: {} rows", a.assertions.len());
    }
    println!(
        "completed public API: {positives} positives, {negatives} exact error-variant controls, {} disagreements",
        disagreements.len()
    );
    if !disagreements.is_empty() {
        return Err(disagreements.join("; ").into());
    }
    println!("PASS public API controls");
    Ok(())
}
