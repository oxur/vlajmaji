//! Normalize and reload a raw jbotci export into a fresh release file.
//!
//! Usage: `cargo run -p vlajmaji-lexicon --example import_jbotci -- RAW METADATA FRESH_OUTPUT`.

use std::{fs, fs::OpenOptions, io::Write};

use vlajmaji_lexicon::{JBOTCI_PINNED_RAW_SHA256, JbotciImportOptions, import_jbotci_export};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let raw_path = args.next().ok_or("missing RAW path")?;
    let metadata_path = args.next().ok_or("missing METADATA path")?;
    let output_path = args.next().ok_or("missing FRESH_OUTPUT path")?;
    if args.next().is_some() {
        return Err("expected RAW, METADATA, and FRESH_OUTPUT paths".into());
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
    let canonical = imported.dictionary().to_json_bytes()?;
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&output_path)?;
    output.write_all(&canonical)?;
    output.sync_all()?;
    drop(output);
    let reloaded = vlajmaji_lexicon::Dictionary::from_json_bytes(&fs::read(output_path)?)?;
    assert_eq!(
        reloaded.to_json_bytes()?,
        canonical,
        "canonical release round-trip"
    );
    let reloaded_captures = reloaded.captures();
    assert_eq!(reloaded_captures.len(), 2);
    assert!(reloaded_captures.iter().any(|capture| capture.bytes == raw));
    assert!(
        reloaded_captures
            .iter()
            .any(|capture| capture.bytes == metadata)
    );
    let reloaded_native_fields: u64 = reloaded_captures
        .iter()
        .flat_map(|c| &c.occurrences)
        .map(|occurrence| occurrence.fields.len() as u64)
        .sum();
    assert_eq!(reloaded_native_fields, imported.report().native_fields);
    println!("{}", serde_json::to_string_pretty(imported.report())?);
    Ok(())
}
