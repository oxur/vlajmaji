use std::{error::Error, path::PathBuf};
use vlajmaji_lexicon::{Dictionary, DictionaryDocument, SourceRegistry, UnverifiedDictionaryDocument};
use vlajmaji_store::{DictionaryStore, DictionaryStoreVerification as Verify, DictionaryStoreVerificationCheck as Check, DictionaryStoreCheckStatus as Status};

fn main() -> Result<(), Box<dyn Error>> {
    let destination = PathBuf::from(std::env::args_os().nth(1).ok_or("owned destination argument required")?);
    let original = Dictionary::new();
    let default_document: DictionaryDocument = original.document();
    let _: &SourceRegistry = &default_document.registry;
    let bytes = original.to_json_bytes()?;
    let raw: UnverifiedDictionaryDocument = serde_json::from_slice(&bytes)?;
    assert_eq!(raw.version, 1);
    let store = DictionaryStore::create(&destination, &original)?;
    assert_eq!(store.dictionary()?, original);
    assert_eq!(store.canonical_bytes()?, bytes);
    assert_eq!(store.unverified_document()?.version, 1);
    drop(store);
    let report = Verify::verify(&destination, Check::All)?;
    assert!(report.ok);
    assert_eq!(report.checks.len(), 6);
    assert!(report.checks.iter().all(|row| row.status == Status::Passed));
    for check in [Check::Schema, Check::Metadata, Check::Hashes, Check::Model, Check::Fks, Check::Projection] {
        let selected = Verify::verify(&destination, check)?;
        assert!(selected.ok);
        assert_eq!(selected.checks.len(), 1);
        assert_eq!(selected.checks[0].status, Status::Passed);
    }
    let mut malformed: serde_json::Value = serde_json::from_slice(&bytes)?;
    malformed["registry"]["sources"] = serde_json::json!([{
        "id":"outside-source", "display_name":"Outside", "abbreviation":"OUT",
        "abbreviation_kind":"Conventional", "abbreviation_evidence":null, "origin":"Upstream"
    }]);
    let unverified: UnverifiedDictionaryDocument = serde_json::from_value(malformed.clone())?;
    assert_eq!(unverified.registry.sources.len(), 1);
    assert!(SourceRegistry::from_document(unverified.registry).is_err());
    assert!(Dictionary::from_json_bytes(&serde_json::to_vec(&malformed)?).is_err());
    println!("outside public caller PASS: default generic compatibility, fallible reads, six checks/all, strict versus unverified registry/model boundary");
    Ok(())
}
