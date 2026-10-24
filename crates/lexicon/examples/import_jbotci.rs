//! Validate and report a raw jbotci export without writing an output file.
//!
//! Usage: `cargo run -p vlajmaji-lexicon --example import_jbotci -- RAW METADATA`.

use std::fs;

use vlajmaji_lexicon::{JBOTCI_PINNED_RAW_SHA256, JbotciImportOptions, import_jbotci_export};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let raw_path = args.next().ok_or("missing RAW path")?;
    let metadata_path = args.next().ok_or("missing METADATA path")?;
    if args.next().is_some() {
        return Err("expected exactly RAW and METADATA paths".into());
    }
    let raw = fs::read(raw_path)?;
    let metadata = fs::read(metadata_path)?;
    let imported = import_jbotci_export(&raw, &metadata, &JbotciImportOptions::default())?;

    let captures = imported.dictionary().captures();
    assert_eq!(captures.len(), 2, "raw and metadata byte captures");
    assert!(captures.iter().any(|capture| capture.bytes == raw));
    assert!(captures.iter().any(|capture| capture.bytes == metadata));
    let native_fields: u64 = captures
        .iter()
        .flat_map(|capture| &capture.occurrences)
        .map(|occurrence| occurrence.fields.len() as u64)
        .sum();
    assert_eq!(native_fields, imported.report().native_fields);

    if imported.report().raw_sha256 == JBOTCI_PINNED_RAW_SHA256 {
        assert_eq!(imported.report().input_rows, 33_526);
        assert_eq!(imported.report().native_fields, 278_254);
        assert_eq!(imported.report().normalized_assertions, 33_526);
        assert_eq!(imported.report().lexical_occurrences, 33_516);
        assert_eq!(imported.report().excluded_occurrences, 10);
    }
    println!("{}", serde_json::to_string_pretty(imported.report())?);
    Ok(())
}
