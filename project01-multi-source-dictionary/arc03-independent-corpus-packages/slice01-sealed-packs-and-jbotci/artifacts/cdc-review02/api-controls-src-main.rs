use vlajmaji_lexicon::*;
use vlajmaji_search::{DictionarySearchEngine,DictionaryExactOptions,ReferenceStatus};
use sha2::{Digest,Sha256};
fn hash(b:&[u8])->String {format!("{:x}",Sha256::digest(b))}
fn model(rev:char)->Dictionary {
 let raw0=br#"[{"word":"farvi","word_type":"gismu","definition":"synthetic definition"}]"#;
 let owned=String::from_utf8(raw0.to_vec()).unwrap().replace("synthetic definition",if rev=='2' {"second synthetic definition"}else{"synthetic definition"});let raw=owned.as_bytes();
 let meta=format!("language_tag = \"en\"\nlanguage_realname = \"English\"\nsource_language_tag = \"jbo\"\nformat = \"json\"\npositive_scores_only = false\nfilename = \"dictionary-en.json\"\nmetadata_url = \"https://example.invalid/languages\"\ndownload_url = \"https://example.invalid/export\"\nlensisku_created_at = \"2026-10-04T03:07:08Z\"\nsha256 = \"{}\"\ndefinition_count = 1\nentry_count = 1\n",hash(raw));
 let options=JbotciImportOptions{revision:rev.to_string().repeat(40),expected_raw_sha256:hash(raw),expected_metadata_sha256:hash(meta.as_bytes())};
 import_jbotci_export(raw,meta.as_bytes(),&options).expect("valid original fixture").into_dictionary()
}
fn main(){
let a=model('1');let b=model('2');let a_id=a.metadata().assertions[0].id.clone();
assert_ne!(a.metadata().releases[0].id,b.metadata().releases[0].id);
let combined=compose_dictionaries(vec![a.clone(),b]).unwrap();
assert_eq!(combined.registry().as_document().sources.len(),1);
assert_eq!(combined.metadata().releases.len(),2);
assert_eq!(combined.metadata().assertions.len(),2); // distinct received rows remain distinct assertions
let mut conflict=a.clone().into_document();conflict.metadata.releases[0].selected=!conflict.metadata.releases[0].selected;
let conflict=Dictionary::from_document(conflict).unwrap();assert!(matches!(compose_dictionaries(vec![a.clone(),conflict]),Err(PackError::Conflict{..})));
let seal=seal_release(PackId::LensiskuJbotci,PackPayloadKind::Synthetic,&combined).unwrap();
assert_eq!(load_sealed_release(PackId::LensiskuJbotci,&seal.release,&seal.seal).unwrap(),combined);
let base:serde_json::Value=serde_json::from_slice(&seal.seal).unwrap();let mut rejected=0;
for field in ["version","dictionary_version","registry_version","release_bytes","release_sha256","source_ids","release_ids","captures","unknown"] {
 let mut v=base.clone();v[field]=match field {"release_sha256"=>serde_json::json!("0".repeat(64)),"source_ids"|"release_ids"|"captures"=>serde_json::json!([]),"unknown"=>serde_json::json!(true),_=>serde_json::json!(999)};
 assert!(load_sealed_release(PackId::LensiskuJbotci,&seal.release,&serde_json::to_vec(&v).unwrap()).is_err(),"{field}");rejected+=1;
}
assert!(matches!(load_sealed_release(PackId::Llg,&seal.release,&seal.seal),Err(PackError::PackMismatch{..})));
let mut duplicate=seal.seal.clone();duplicate.pop();duplicate.extend_from_slice(b",\"version\":1}");assert!(load_sealed_release(PackId::LensiskuJbotci,&seal.release,&duplicate).is_err());
let mut invalid=base.clone();let bad=b"{}";invalid["release_bytes"]=serde_json::json!(bad.len());invalid["release_sha256"]=serde_json::json!(hash(bad));assert!(load_sealed_release(PackId::LensiskuJbotci,bad,&serde_json::to_vec(&invalid).unwrap()).is_err());
let path=std::path::Path::new("/tmp/vlajmaji-p01-a03-cdc/s01/tiny-composed-store01");
let engine=DictionarySearchEngine::build(path,&seal.release).unwrap();drop(engine);
let store=vlajmaji_store::DictionaryStore::open(path).unwrap();assert_eq!(store.dictionary().unwrap(),combined);drop(store);
let engine=DictionarySearchEngine::open(path).unwrap();let all=engine.exact("farvi",DictionaryExactOptions::default()).unwrap();
assert_eq!((all.totals.occurrences,all.totals.distinct_spellings,all.totals.source_groups),(2,1,1));
for selector in ["lensisku","LSK"] {let opt=DictionaryExactOptions{sources:vec![selector.into()],..Default::default()};assert_eq!(engine.exact("farvi",opt).unwrap(),all);}
assert!(engine.exact("farvi",DictionaryExactOptions{sources:vec!["missing-source".into()],..Default::default()}).is_err());
let page=engine.exact("farvi",DictionaryExactOptions{page_size:Some(1),..Default::default()}).unwrap();assert_eq!(page.totals,all.totals);let next=engine.exact("farvi",DictionaryExactOptions{page_size:Some(1),cursor:page.next_cursor.clone(),..Default::default()}).unwrap();assert!(page.next_cursor.is_some());assert!(next.next_cursor.is_none());let ids=|groups: &Vec<vlajmaji_search::DictionaryGroup>|groups.iter().flat_map(|g|g.hits.iter().map(|h|h.assertion.id.clone())).collect::<Vec<_>>();let mut paged=ids(&page.groups);paged.extend(ids(&next.groups));assert_eq!(paged,ids(&all.groups));
let local=engine.resolve(&RelationTarget::Local(LocalRef::Assertion(a_id))).unwrap();assert_eq!(local.status,ReferenceStatus::Resolved);assert!(local.entity.is_some());
assert_eq!(engine.resolve(&RelationTarget::Unknown).unwrap().status,ReferenceStatus::Unknown);
let absent=engine.exact("absent",DictionaryExactOptions::default()).unwrap();assert_eq!(absent.totals.occurrences,0);
println!("PASS seal_negative_cases={} plus wrong-pack/duplicate-key/invalid-model; same-ID selected conflict; distinct releases/shared assertion; tiny create/reopen/source ID+alias/unknown/totals/page/local+unknown reference/absent",rejected);
}
