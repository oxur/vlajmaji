#[path = "/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc02/vlajmaji/crates/lexicon/examples/support/dictionary_fixture.rs"]
mod fixture;

use serde_json::{Value, json};
use std::{
    error::Error,
    fs,
    io::{BufWriter, Write},
    path::Path,
    time::Instant,
};
use vlajmaji::{
    lexicon::Dictionary,
    search::{DictionaryExactOptions, DictionaryRequest, DictionarySearchEngine},
    store::DictionaryStore,
};

fn execute(engine: &DictionarySearchEngine, request: Value) -> Result<Value, Box<dyn Error>> {
    let req: DictionaryRequest = serde_json::from_value(request)?;
    Ok(match req {
        DictionaryRequest::Exact { spelling, options } => match engine.exact(&spelling, options) {
            Ok(v) => json!({"ok":v}),
            Err(e) => json!({"error":e.report()}),
        },
        DictionaryRequest::Resolve { target } => match engine.resolve(&target) {
            Ok(v) => json!({"ok":v}),
            Err(e) => json!({"error":e.report()}),
        },
    })
}
fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args().collect();
    let mode = &args[1];
    let store = &args[2];
    match mode.as_str() {
        "fixture" => {
            let root = Path::new(store);
            fs::create_dir(root)?;
            let base = fixture::complete_fixture()?;
            let e = DictionarySearchEngine::build(root.join("base"), &base.to_json_bytes()?)?;
            let mut responses = serde_json::Map::new();
            let request = |sources: Vec<&str>, alias_scope: &str| json!({"operation":"exact","spelling":"farvi","options":{"sources":sources,"alias_scope":alias_scope}});
            let default = execute(&e, request(vec![], "ActiveOnly"))?;
            assert_eq!(default["ok"]["native_candidate_occurrences"], 7);
            assert_eq!(default["ok"]["totals"]["occurrences"], 6);
            assert_eq!(default["ok"]["totals"]["source_groups"], 3);
            responses.insert("default".into(), default);
            let selected = execute(&e, request(vec!["source-a"], "ActiveOnly"))?;
            assert_eq!(selected["ok"]["totals"]["occurrences"], 4);
            responses.insert("source-id".into(), selected.clone());
            for (name, selector, scope) in [("abbreviation","a","ActiveOnly"),("active-alias","archive-a","ActiveOnly"),("retired-included","legacy-a","IncludeRetired")] {
                let r = execute(&e, request(vec![selector],scope))?;
                assert_eq!(r,selected,"{name} complete source selector counterpart");
                responses.insert(name.into(),r);
            }
            let retired = execute(&e, request(vec!["legacy-a"],"ActiveOnly"))?;
            assert_eq!(retired["error"]["code"],"unknown_source");
            responses.insert("retired-rejected".into(),retired);
            let missing = execute(&e, request(vec!["source-m"],"ActiveOnly"))?;
            assert_eq!(missing["error"]["code"],"unavailable_source");
            assert!(missing["error"]["message"].as_str().unwrap().contains("NoSelectedAvailableRelease"));
            responses.insert("missing-release".into(),missing);
            let owned = execute(&e, request(vec!["source-o"],"ActiveOnly"))?;
            assert_eq!(owned["ok"]["totals"]["occurrences"],1);
            responses.insert("owned-head-valid".into(),owned);
            drop(e);
            let mut doc = base.document();
            doc.metadata.owned_heads.clear();
            let d = Dictionary::from_document(doc)?;
            let e = DictionarySearchEngine::build(root.join("no-head"), &d.to_json_bytes()?)?;
            let missing = execute(&e, request(vec!["source-o"],"ActiveOnly"))?;
            assert_eq!(missing["error"]["code"],"unavailable_source");
            assert!(missing["error"]["message"].as_str().unwrap().contains("NoOwnedHead"));
            responses.insert("owned-head-missing".into(),missing);
            let without = execute(&e, request(vec![],"ActiveOnly"))?;
            assert_eq!(without["ok"]["totals"]["occurrences"],5);
            assert_eq!(without["ok"]["totals"]["source_groups"],2);
            responses.insert("default-without-head".into(),without);
            drop(e);
            let mut doc = base.document();
            let mut registry = doc.registry.document();
            registry.sources.push(serde_json::from_value(json!({"id":"source-empty","display_name":"Registered with no release","abbreviation":"emp","abbreviation_kind":"ProjectAssigned","abbreviation_evidence":null,"origin":"Upstream"}))?);
            doc.registry = vlajmaji_lexicon::SourceRegistry::from_document(registry)?;
            let d = Dictionary::from_document(doc)?;
            let e = DictionarySearchEngine::build(root.join("no-release"),&d.to_json_bytes()?)?;
            let missing = execute(&e,request(vec!["source-empty"],"ActiveOnly"))?;
            assert_eq!(missing["error"]["code"],"unavailable_source");
            assert!(missing["error"]["message"].as_str().unwrap().contains("NoReleases"));
            responses.insert("no-releases".into(),missing);
            let unchanged = execute(&e, request(vec![],"ActiveOnly"))?;
            assert_eq!(unchanged["ok"]["totals"]["occurrences"],6);
            assert_eq!(unchanged["ok"]["totals"]["source_groups"],3);
            responses.insert("default-unused-source".into(),unchanged);
            fs::write(root.join("responses.json"),serde_json::to_vec_pretty(&responses)?)?;
            println!("PASS fixture {} complete selector/availability responses",responses.len());
        }
        "create" => {
            let bytes = fs::read(&args[3])?;
            let start = Instant::now();
            let d = Dictionary::from_json_bytes(&bytes)?;
            println!("parse_ms={}", start.elapsed().as_millis());
            let start = Instant::now();
            let s = DictionaryStore::create(store, &d)?;
            assert_eq!(s.dictionary(), &d);
            assert_eq!(s.canonical_bytes(), bytes);
            println!(
                "create_reopen_ms={} database_bytes={}",
                start.elapsed().as_millis(),
                fs::metadata(s.database_path())?.len()
            );
        }
        "finite" | "finite8" => {
            let d = Dictionary::from_json_bytes(&fs::read(&args[3])?)?;
            let output = Path::new(&args[4]);
            fs::create_dir(output)?;
            let start = Instant::now();
            let e = DictionarySearchEngine::open(store)?;
            println!("strict_open_ms={}", start.elapsed().as_millis());
            let a = d
                .metadata()
                .assertions
                .iter()
                .find(|a| a.occurrence.locator == "/13306")
                .ok_or("farvi assertion missing")?;
            let release = &d.metadata().releases[0];
            let mut requests: Vec<(&str, Value)> = vec![
                (
                    "farvi-default",
                    json!({"operation":"exact","spelling":"farvi"}),
                ),
                (
                    "farvi-abbreviation",
                    json!({"operation":"exact","spelling":"farvi","options":{"sources":["LSK"]}}),
                ),
                (
                    "farvi-source-id",
                    json!({"operation":"exact","spelling":"farvi","options":{"sources":["lensisku"]}}),
                ),
                ("cpana", json!({"operation":"exact","spelling":"cpana"})),
                ("sie", json!({"operation":"exact","spelling":"si'e"})),
                ("ma", json!({"operation":"exact","spelling":"ma"})),
                (
                    "unknown-spelling",
                    json!({"operation":"exact","spelling":"__a02_nonexistent__"}),
                ),
                (
                    "wiki-excluded",
                    json!({"operation":"exact","spelling":"Periodic-table gismu assignment algorithm"}),
                ),
                (
                    "undefined-excluded",
                    json!({"operation":"exact","spelling":"sekoi'ai"}),
                ),
                (
                    "unknown-selector",
                    json!({"operation":"exact","spelling":"farvi","options":{"sources":["unknown-source"]}}),
                ),
                (
                    "zero-page",
                    json!({"operation":"exact","spelling":"farvi","options":{"page_size":0}}),
                ),
                (
                    "bad-cursor",
                    json!({"operation":"exact","spelling":"farvi","options":{"cursor":"zz"}}),
                ),
                (
                    "resolve-local",
                    json!({"operation":"resolve","target":{"Local":{"Assertion":a.id}}}),
                ),
                (
                    "resolve-unknown",
                    json!({"operation":"resolve","target":"Unknown"}),
                ),
                (
                    "resolve-unresolved",
                    json!({"operation":"resolve","target":{"Unresolved":"pending native reference"}}),
                ),
                (
                    "resolve-empty",
                    json!({"operation":"resolve","target":{"Unresolved":""}}),
                ),
                (
                    "catalog-native",
                    json!({"operation":"resolve","target":{"Catalog":{"source":"lensisku","release":release.id,"native_assertion_id":"333"}}}),
                ),
                (
                    "catalog-excluded-wiki",
                    json!({"operation":"resolve","target":{"Catalog":{"source":"lensisku","release":release.id,"native_assertion_id":"88416"}}}),
                ),
                (
                    "catalog-mismatch",
                    json!({"operation":"resolve","target":{"Catalog":{"source":"lensisku","release":release.id,"native_assertion_id":"__missing__"}}}),
                ),
                (
                    "catalog-bad-release",
                    json!({"operation":"resolve","target":{"Catalog":{"source":"lensisku","release":"missing-release","native_assertion_id":"333"}}}),
                ),
            ];
            let page_start = Instant::now();
            let page = execute(
                &e,
                json!({"operation":"exact","spelling":"farvi","options":{"page_size":1}}),
            )?;
            let page_elapsed = page_start.elapsed().as_micros();
            let cursor = page["ok"]["next_cursor"]
                .as_str()
                .ok_or("farvi cursor missing")?
                .to_owned();
            fs::write(output.join("page-first.json"), serde_json::to_vec(&page)?)?;
            requests.push(("page-next",json!({"operation":"exact","spelling":"farvi","options":{"page_size":1,"cursor":cursor}})));
            requests.push(("cursor-other-spelling",json!({"operation":"exact","spelling":"ma","options":{"page_size":1,"cursor":cursor}})));
            if mode == "finite8" {
                requests.retain(|(name, _)| {
                    matches!(
                        *name,
                        "farvi-default"
                            | "ma"
                            | "farvi-abbreviation"
                            | "wiki-excluded"
                            | "resolve-local"
                            | "catalog-native"
                            | "page-next"
                    )
                });
            }
            let mut timing = vec![
                json!({"case":"page-first","elapsed_us":page_elapsed,"request":{"operation":"exact","spelling":"farvi","options":{"page_size":1}},"success":true}),
            ];
            for (name, request) in requests {
                let start = Instant::now();
                let response = execute(&e, request.clone())?;
                let elapsed = start.elapsed().as_micros();
                fs::write(
                    output.join(format!("{name}.json")),
                    serde_json::to_vec(&response)?,
                )?;
                timing.push(json!({"case":name,"elapsed_us":elapsed,"request":request,"success":response.get("ok").is_some()}));
                println!("case={name} elapsed_us={elapsed}");
            }
            fs::write(
                output.join("timing.json"),
                serde_json::to_vec_pretty(&timing)?,
            )?;
            println!("PASS finite {} complete response cases", timing.len());
        }
        "population" => {
            let words: Vec<String> = serde_json::from_slice(&fs::read(&args[3])?)?;
            let output = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&args[4])?;
            let mut output = BufWriter::new(output);
            let start = Instant::now();
            let e = DictionarySearchEngine::open(store)?;
            let mut total = 0;
            for (i, word) in words.iter().enumerate() {
                let response = e.exact(word, DictionaryExactOptions::default())?;
                let mut hits = Vec::new();
                for group in &response.groups {
                    for h in &group.hits {
                        hits.push(json!({"source":group.source.id,"occurrence_id":h.occurrence_id,"native_assertion_id":h.occurrence_attribution.as_ref().and_then(|a|a.native_assertion_id.as_ref()),"definition":h.assertion.definition,"score":h.assertion.source_score}));
                    }
                }
                total += hits.len();
                let line = json!({"spelling":word,"native_candidates":response.native_candidate_occurrences,"occurrences":response.totals.occurrences,"distinct_spellings":response.totals.distinct_spellings,"groups":response.totals.source_groups,"hits":hits});
                serde_json::to_writer(&mut output, &line)?;
                output.write_all(b"\n")?;
                if (i + 1) % 1000 == 0 {
                    println!(
                        "population_queries={} occurrences={} elapsed_ms={}",
                        i + 1,
                        total,
                        start.elapsed().as_millis()
                    );
                    std::io::stdout().flush()?;
                }
            }
            output.flush()?;
            println!(
                "PASS population queries={} occurrences={} elapsed_ms={}",
                words.len(),
                total,
                start.elapsed().as_millis()
            );
        }
        _ => return Err("expected create, finite or population".into()),
    }
    Ok(())
}
