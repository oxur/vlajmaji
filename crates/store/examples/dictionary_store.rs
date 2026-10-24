//! Build and reopen one validated versioned dictionary from canonical JSON.
//!
//! Usage: `cargo run -p vlajmaji-store --example dictionary_store -- INPUT.json STORE_DIR`
use std::{env, fs};
use vlajmaji_lexicon::Dictionary;
use vlajmaji_store::DictionaryStore;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args_os().skip(1);
    let input = args.next().ok_or("expected INPUT.json and STORE_DIR")?;
    let destination = args.next().ok_or("expected STORE_DIR")?;
    if args.next().is_some() {
        return Err("too many arguments".into());
    }
    let bytes = fs::read(input)?;
    let dictionary = Dictionary::from_json_bytes(&bytes)?;
    let expected_document = dictionary.document();
    let expected = dictionary.to_json_bytes()?;
    let destination = std::path::PathBuf::from(destination);
    let store = DictionaryStore::create(&destination, &dictionary)?;
    assert_eq!(store.canonical_bytes(), expected);
    drop(store);
    let reopened = DictionaryStore::open(&destination)?;
    assert_eq!(reopened.canonical_bytes(), expected);
    assert_eq!(reopened.dictionary().document(), expected_document);
    let model = reopened.dictionary();
    println!(
        "validated {} captures, {} occurrences, {} entries, {} assertions: {}",
        model.captures().len(),
        model
            .captures()
            .iter()
            .map(|c| c.occurrences.len())
            .sum::<usize>(),
        model.metadata().entries.len(),
        model.metadata().assertions.len(),
        reopened.path().display()
    );
    Ok(())
}
