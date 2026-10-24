//! Build, reopen, page, and resolve against a validated dictionary store.

use std::{env, fs, process::ExitCode};

use vlajmaji_lexicon::{LocalRef, RelationTarget};
use vlajmaji_search::{DictionaryExactOptions, DictionarySearchEngine};

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = env::args_os().skip(1).collect();
    if args.len() != 2 {
        return Err("usage: dictionary_lookup INPUT_JSON FRESH_STORE".into());
    }
    let bytes = fs::read(&args[0])?;
    let built = DictionarySearchEngine::build(&args[1], &bytes)?;
    let (captures, occurrences, entries, assertions) = built.dictionary_counts();
    println!(
        "built {} captures, {} occurrences, {} entries, {} assertions; state {}",
        captures,
        occurrences,
        entries,
        assertions,
        built.state_sha256()
    );
    drop(built);

    let engine = DictionarySearchEngine::open(&args[1])?;
    let complete = engine.exact("farvi", DictionaryExactOptions::default())?;
    println!(
        "farvi: {} selected occurrences from {} native lexical candidates",
        complete.totals.occurrences, complete.native_candidate_occurrences
    );

    let mut cursor = None;
    loop {
        let page = engine.exact(
            "farvi",
            DictionaryExactOptions {
                page_size: Some(2),
                cursor,
                ..Default::default()
            },
        )?;
        for group in page.groups {
            for hit in group.hits {
                println!(
                    "{} {} {} {}",
                    group.source.id, hit.occurrence_id, hit.entry.id, hit.capture.source_name
                );
            }
        }
        cursor = page.next_cursor;
        if cursor.is_none() {
            break;
        }
    }

    if let Some(hit) = complete.groups.iter().flat_map(|group| &group.hits).next() {
        let resolved = engine.resolve(&RelationTarget::Local(LocalRef::Assertion(
            hit.assertion.id.clone(),
        )))?;
        println!(
            "local assertion reference: {:?}; confirmed={} unverified={}",
            resolved.status,
            resolved.confirmed.len(),
            resolved.unverified.len()
        );
    }
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("dictionary_lookup: {error}");
            ExitCode::FAILURE
        }
    }
}
