use std::{error::Error,fs::{self,OpenOptions},io::Write,path::Path,time::Instant};
use serde_json::{Value,json};
use vlajmaji::{lexicon as lex,store::{DictionaryStore,DictionaryStoreError},search::{DictionarySearchEngine,DictionaryExactOptions,DictionaryQueryError}};
type Result<T> = std::result::Result<T,Box<dyn Error>>;
fn write_new(p:impl AsRef<Path>,b:&[u8])->Result<()> {let mut f=OpenOptions::new().write(true).create_new(true).open(p)?;f.write_all(b)?;f.flush()?;Ok(())}
fn emit(root:&Path,name:&str,v:&Value)->Result<()> {write_new(root.join(format!("{name}.json")),&serde_json::to_vec(v)?) }
fn options(sources:&[&str],size:Option<usize>,cursor:Option<String>)->DictionaryExactOptions {DictionaryExactOptions{sources:sources.iter().map(|s|(*s).to_string()).collect(),page_size:size,cursor,..Default::default()}}
fn fail_code(e:DictionaryQueryError,expected:&str,errors:&mut Vec<Value>){let report=e.report();assert_eq!(report.code,expected);errors.push(serde_json::to_value(report).unwrap());}
fn main()->Result<()> {
 let args:Vec<_>=std::env::args().collect();assert_eq!(args.len(),4);
 let root=Path::new(&args[3]);let raw=fs::read(&args[1])?;let meta=fs::read(&args[2])?;let mut times=json!({});
 let t=Instant::now();eprintln!("stage full-public-import pid={}",std::process::id());
 let imported=lex::import_jbotci_export(&raw,&meta,&Default::default())?;
 let report=imported.report().clone();emit(root,"import-report",&serde_json::to_value(&report)?)?;
 assert_eq!((report.input_rows,report.native_fields,report.lexical_occurrences,report.excluded_occurrences,report.eligible_spellings),(33526,278254,33516,10,31248));
 assert!(report.definition_count_matches && report.entry_count_matches);assert_eq!(report.input_rows,report.normalized_assertions);
 let dictionary=imported.into_dictionary();times["import_s"]=json!(t.elapsed().as_secs_f64());
 let t=Instant::now();eprintln!("stage full-canonical-reload");let canonical=dictionary.to_json_bytes()?;assert_eq!(canonical.len(),215412497);
 write_new(root.join("canonical.json"),&canonical)?;let reload=lex::Dictionary::from_json_bytes(&canonical)?;assert!(reload==dictionary);drop(reload);
 assert_eq!(dictionary.captures().len(),2);assert_eq!(dictionary.captures()[0].bytes,raw);assert_eq!(dictionary.captures()[1].bytes,meta);assert_eq!(dictionary.captures()[0].occurrences.iter().map(|r|r.fields.len()).sum::<usize>(),278254);
 times["canonical_reload_s"]=json!(t.elapsed().as_secs_f64());let storepath=root.join("fresh-full-store");
 let t=Instant::now();eprintln!("stage fresh-full-create-bound-600s");let store=DictionaryStore::create(&storepath,&dictionary)?;
 assert!(store.dictionary()==&dictionary);assert_eq!(store.canonical_bytes(),canonical);times["create_internal_strict_reopen_s"]=json!(t.elapsed().as_secs_f64());drop(store);
 let t=Instant::now();eprintln!("stage independent-drop-strict-reopen");let reopened=DictionaryStore::open(&storepath)?;assert!(reopened.dictionary()==&dictionary);assert_eq!(reopened.canonical_bytes(),canonical);times["strict_reopen_s"]=json!(t.elapsed().as_secs_f64());
 assert_eq!(reopened.exact_assertion_ids("farvi")?.len(),2);drop(reopened);
 let release=dictionary.metadata().releases[0].id.clone();let aid=dictionary.metadata().assertions[13306].id.clone();
 drop(dictionary);drop(canonical);
 let t=Instant::now();eprintln!("stage public-engine-open");let engine=DictionarySearchEngine::open(&storepath)?;times["engine_open_s"]=json!(t.elapsed().as_secs_f64());
 eprintln!("stage finite-public-controls");let t=Instant::now();let mut errors=Vec::new();
 let full=engine.exact("farvi",Default::default())?;assert_eq!((full.totals.occurrences,full.totals.distinct_spellings,full.totals.source_groups),(2,1,1));assert_eq!(full.groups[0].source.abbreviation,"LSK");
 let fullval=serde_json::to_value(&full)?;emit(root,"farvi-public-full",&fullval)?;
 for selector in ["LSK","lensisku"]{let selected=engine.exact("farvi",options(&[selector],None,None))?;assert!(selected==full);}
 let zero=engine.exact("farvi",options(&[],Some(0),None))?;assert_eq!(zero.totals,full.totals);assert!(zero.groups.is_empty());assert!(zero.next_cursor.is_none());emit(root,"farvi-page0",&serde_json::to_value(zero)?)?;
 let first=engine.exact("farvi",options(&[],Some(1),None))?;let cursor=first.next_cursor.clone().expect("two results page1 needs cursor");assert_eq!(first.totals,full.totals);assert!(first.groups[0].partial);
 let second=engine.exact("farvi",options(&[],Some(1),Some(cursor.clone())))?;assert_eq!(second.totals,full.totals);assert!(second.next_cursor.is_none());
 let merged:Vec<_>=first.groups.iter().chain(second.groups.iter()).flat_map(|g|g.hits.clone()).collect();assert_eq!(merged,full.groups[0].hits);
 emit(root,"farvi-page1",&serde_json::to_value(first)?)?;emit(root,"farvi-page2",&serde_json::to_value(second)?)?;
 fail_code(engine.exact("ma",options(&[],Some(1),Some(cursor))).unwrap_err(),"stale_cursor",&mut errors);
 fail_code(engine.exact("farvi",options(&[],Some(1),Some("garbage".into()))).unwrap_err(),"malformed_cursor",&mut errors);
 fail_code(engine.exact("farvi",options(&["missing-source"],None,None)).unwrap_err(),"unknown_source",&mut errors);
 let ma=engine.exact("ma",Default::default())?;assert_eq!(ma.totals.occurrences,2);emit(root,"ma-full",&serde_json::to_value(ma)?)?;
 let absent=engine.exact("not-a-present-word",Default::default())?;assert_eq!(absent.totals.occurrences,0);emit(root,"no-match",&serde_json::to_value(absent)?)?;
 let wikiword="Periodic-table gismu assignment algorithm";let wiki=engine.exact(wikiword,Default::default())?;assert_eq!(wiki.totals.occurrences,0);emit(root,"wiki-exact",&serde_json::to_value(wiki)?)?;
 for (label,id) in [("native333","333"),("wiki-native88416","88416"),("native-no-match","no-such-native")]{
  let target=lex::RelationTarget::Catalog(lex::CatalogTarget{source:lex::SourceId::new("lensisku")?,release:release.clone(),native_entry_id:None,native_assertion_id:Some(id.into()),occurrence:None});
  let response=engine.resolve(&target)?;let v=serde_json::to_value(&response)?;
  if id=="333" {assert_eq!(v["status"],"Resolved");assert_eq!(v["confirmed"].as_array().unwrap().len(),1);}
  if id=="88416" {assert_eq!(v["status"],"Resolved");assert_eq!(v["confirmed"][0]["current_eligible"],false);}
  if id=="no-such-native" {assert_eq!(v["status"],"NoMatch");}
  emit(root,label,&v)?;
 }
 let local=engine.resolve(&lex::RelationTarget::Local(lex::LocalRef::Assertion(aid)))?;emit(root,"local13306",&serde_json::to_value(local)?)?;
 fail_code(engine.resolve(&lex::RelationTarget::Local(lex::LocalRef::Assertion(lex::EntityId::new("absent-id")?))).unwrap_err(),"invalid_reference",&mut errors);
 fail_code(engine.resolve(&lex::RelationTarget::Unresolved(String::new())).unwrap_err(),"invalid_reference",&mut errors);
 let unknown=engine.resolve(&lex::RelationTarget::Unknown)?;assert_eq!(serde_json::to_value(&unknown)?["status"],"Unknown");
 emit(root,"typed-errors",&json!(errors))?;times["finite_controls_s"]=json!(t.elapsed().as_secs_f64());drop(engine);
 eprintln!("stage synthetic-repeated-row-native-id");
 let fixture=fs::read(root.join("duplicate-raw.json"))?;let metadata=fs::read(root.join("duplicate-meta.toml"))?;let pins:Value=serde_json::from_slice(&fs::read(root.join("duplicate-pins.json"))?)?;
 let opts=lex::JbotciImportOptions{revision:lex::JBOTCI_PINNED_REVISION.into(),expected_raw_sha256:pins["raw"].as_str().unwrap().into(),expected_metadata_sha256:pins["metadata"].as_str().unwrap().into()};
 let imp=lex::import_jbotci_export(&fixture,&metadata,&opts)?;assert_eq!((imp.report().input_rows,imp.report().lexical_occurrences,imp.report().eligible_spellings),(2,2,1));
 let mut bad=opts.clone();bad.expected_raw_sha256="0".repeat(64);assert!(matches!(lex::import_jbotci_export(&fixture,&metadata,&bad),Err(lex::JbotciImportError::HashMismatch{input,..}) if input=="raw"));
 let tiny=root.join("duplicate-store");let st=DictionaryStore::create(&tiny,imp.dictionary())?;assert!(matches!(DictionaryStore::create(&tiny,imp.dictionary()),Err(DictionaryStoreError::ExistingPath(_))));drop(st);
 let en=DictionarySearchEngine::open(&tiny)?;let two=en.exact("farvi",Default::default())?;assert_eq!((two.totals.occurrences,two.totals.distinct_spellings),(2,1));assert_ne!(two.groups[0].hits[0].occurrence_id,two.groups[0].hits[1].occurrence_id);assert_eq!(two.groups[0].hits[0].occurrence_attribution.as_ref().unwrap().native_assertion_id,two.groups[0].hits[1].occurrence_attribution.as_ref().unwrap().native_assertion_id);emit(root,"duplicate-query",&serde_json::to_value(two)?)?;drop(en);
 emit(root,"rust-timing",&times)?;println!("{}",json!({"verdict":"PASS","pid":std::process::id(),"rows":33526,"fields":278254,"fresh_store":storepath,"full_model_equality":true,"capture_byte_equality":true,"canonical_bytes":215412497,"duplicate_occurrences":2,"timing":times}));Ok(())
}
