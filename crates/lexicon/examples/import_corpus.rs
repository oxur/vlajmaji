//! Imports the explicit accepted E02 corpus manifest and writes an archive.

use std::collections::BTreeMap;
use std::error::Error;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use sha2::{Digest, Sha256};
use vlajmaji_lexicon::{Lexicon, SourceFormat, SourceMetadata, TimePrecision};

#[derive(Debug, Deserialize)]
struct InputManifest {
    schema_version: u32,
    inputs: Vec<Input>,
}

#[derive(Debug, Deserialize)]
struct Input {
    path: String,
    namespace: String,
    format: SourceFormat,
    source_name: String,
    locator: String,
    revision: String,
    sha256: String,
    captured_at: Option<String>,
    time_precision: TimePrecision,
    notice_locator: Option<String>,
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn import_corpus(root: &Path, archive_path: &Path) -> Result<serde_json::Value, Box<dyn Error>> {
    let manifest_path = root.join("experiments/E03-lexical-data-search/slice02/corpus-inputs.json");
    let manifest_bytes = std::fs::read(&manifest_path)?;
    let manifest: InputManifest = serde_json::from_slice(&manifest_bytes)?;
    if manifest.schema_version != 1 {
        return Err(format!(
            "unsupported corpus manifest version {}",
            manifest.schema_version
        )
        .into());
    }

    let mut lexicon = Lexicon::new();
    let mut per_namespace = BTreeMap::new();
    for input in manifest.inputs {
        let bytes = std::fs::read(root.join(&input.path))?;
        let actual = sha256(&bytes);
        if actual != input.sha256 {
            return Err(format!("input hash mismatch for {}: {}", input.path, actual).into());
        }
        let mut metadata = SourceMetadata::new(
            input.namespace.clone(),
            input.source_name,
            input.format,
            input.locator,
            input.revision,
            input.captured_at,
            input.time_precision,
        );
        if let Some(notice) = input.notice_locator {
            metadata = metadata.with_notice_locator(notice);
        }
        let before = lexicon.records().len();
        let inserted = lexicon.import_source(metadata, &bytes, Some(&input.sha256))?;
        if !inserted {
            return Err(format!("duplicate input snapshot {}", input.path).into());
        }
        per_namespace.insert(input.namespace, lexicon.records().len() - before);
    }

    let archive = lexicon.to_archive_bytes()?;
    if let Some(parent) = archive_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(archive_path, &archive)?;
    let restored = Lexicon::from_archive_bytes(&archive)?;
    if restored.records().len() != lexicon.records().len() {
        return Err("archive reload changed the record count".into());
    }

    let farvi: Vec<_> = lexicon
        .lookup_exact("farvi")
        .iter()
        .map(|record| {
            serde_json::json!({
                "namespace": record.identity().snapshot().namespace(),
                "locator": record.identity().locator(),
                "definition_id": format!("{:?}", record.field("definitionid")),
                "langid": format!("{:?}", record.field("langid")),
            })
        })
        .collect();
    let sihe: Vec<_> = lexicon
        .lookup_exact("si'e")
        .iter()
        .map(|record| {
            serde_json::json!({
                "namespace": record.identity().snapshot().namespace(),
                "locator": record.identity().locator(),
                "selmaho": format!("{:?}", record.field("selmaho")),
            })
        })
        .collect();
    let assigned_forms = lexicon
        .forms()
        .iter()
        .filter(|form| form.role() == vlajmaji_lexicon::FormRole::AssignedRafsi)
        .count();
    let source_reported_forms = lexicon
        .forms()
        .iter()
        .filter(|form| form.role() == vlajmaji_lexicon::FormRole::SourceReportedForm)
        .count();
    let definition_bearing_source_rows = lexicon
        .records()
        .iter()
        .filter(|record| record.definition().is_some())
        .count();
    let forms_only_assigned_rafsi_rows = lexicon
        .records()
        .iter()
        .filter(|record| {
            record.format() == SourceFormat::LlgRafsiRows
                && record.definition().is_none()
                && record.gloss().is_some()
        })
        .count();
    Ok(serde_json::json!({
        "schema_version": 1,
        "input_manifest": manifest_path.strip_prefix(root)?.to_string_lossy(),
        "snapshot_count": lexicon.snapshots().len(),
        "record_count": lexicon.records().len(),
        "records_by_namespace": per_namespace,
        "form_assertions": {
            "assigned_rafsi": assigned_forms,
            "source_reported_form": source_reported_forms,
        },
        "definition_bearing_source_rows": definition_bearing_source_rows,
        "forms_only_assigned_rafsi_rows": forms_only_assigned_rafsi_rows,
        "cross_reference_count": lexicon.cross_references().len(),
        "farvi_exact_records": farvi,
        "sihe_exact_records": sihe,
        "sie_exact_records": lexicon.lookup_exact("sie").len(),
        "archive_path": archive_path,
        "archive_sha256": sha256(&archive),
        "archive_reload_record_count": restored.records().len(),
    }))
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args_os().skip(1);
    let root = args
        .next()
        .map(PathBuf::from)
        .unwrap_or(std::env::current_dir()?);
    let archive = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("target/slice02-lexicon-archive.json"));
    let summary = import_corpus(&root, &archive)?;
    println!("{}", serde_json::to_string_pretty(&summary)?);
    Ok(())
}
