//! PROSPECTIVE S05 numerical observer. SOURCE ONLY, no build/run authority.
//! Full command has fixed retained ROOT/config/workload; Tiny qualification separate.
//! No store mutation/import/create/All API; exactly one engine per process.
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fs::OpenOptions, io::Write, path::{Path,PathBuf}, time::{Duration,Instant}};
use vlajmaji_search::*;
use vlajmaji_store::CollectionProfiles;
const ROOT:&str="/tmp/vlajmaji-p01-a04-s04-cdc/full-operational01";
const CONFIG:&str="/tmp/vlajmaji-p01-a04-s04-cdc/full-config01.toml";
const STATE:&str="d9d73cbb49cc5a9ddb0ec16cb6dbb4bb7260ad49f4f8f54c6e6d21f0779b7b19";
const CONTENT:&str="b4ee3e48edb910ce63b892f2a1d61342c26d3f2a1a46545f93ab15157192766e";
const SPEC:&str=include_str!("../fixtures/requests-full03.json");
include!("semantic.rs");
#[derive(Clone,Deserialize)]
#[serde(deny_unknown_fields)]
struct Work {id:String,operation:String,#[serde(default)]spelling:String,#[serde(default)]sources:Vec<String>,page_size:Option<usize>,cursor_from:Option<String>,target:Option<vlajmaji_lexicon::RelationTarget>,expected_error:Option<String>,#[serde(default)]prime:bool}
enum Engine {Full(CollectionSearchEngine),Compact(CollectionCompactSearchEngine)}
enum Response {Full(CollectionQueryResponse),Compact(CollectionCompactQueryResponse)}
impl Engine {
 fn execute(&mut self,q:DictionaryRequest)->Result<(Response,CollectionProjectionProfile),CollectionQueryError>{match self{Self::Full(e)=>e.execute_profiled(q).map(|(r,p)|(Response::Full(r),p)),Self::Compact(e)=>e.execute_profiled(q).map(|(r,p)|(Response::Compact(r),p))}}
 fn snapshot(&self)->&CollectionSelectionSnapshot{match self{Self::Full(e)=>e.selection(),Self::Compact(e)=>e.selection()}}
}
fn ns(x:Duration)->u64{x.as_nanos().try_into().expect("bounded duration")}
fn sha(b:&[u8])->String{format!("{:x}",Sha256::digest(b))}
fn emit(v:&Value){let mut out=std::io::stdout().lock();serde_json::to_writer(&mut out,v).unwrap();out.write_all(b"\n").unwrap();out.flush().unwrap();}
fn json_wire<T:Serialize>(r:T)->Vec<u8>{
 // Actual existing CLI Full/Compact path: typed -> Value -> writer + LF.
 // Inner scope returns AFTER value and consumed typed-response temporaries drop.
 let value=serde_json::to_value(r).unwrap();let mut bytes=serde_json::to_vec(&value).unwrap();bytes.push(b'\n');drop(value);bytes
}
fn wire(r:Response,mode:&str)->Vec<u8>{match r{Response::Full(r)=>json_wire(r),Response::Compact(r)=>if mode=="readable"{render_collection_compact(&r).into_bytes()}else{json_wire(r)}}}
fn facts(r:&Response)->(Value,Option<String>){match r{
 Response::Full(r)=>match &r.result{
  CollectionQueryResult::Exact(x)=>(json!({"operation":"exact","totals":x.totals,"source_totals":x.source_totals.iter().map(|s|json!({"source":s.source.id,"occurrences":s.occurrences})).collect::<Vec<_>>(),"assertion_ids":x.groups.iter().flat_map(|g|g.hits.iter().map(|h|h.assertion.id.to_string())).collect::<Vec<_>>(),"groups":x.groups.len()}),x.next_cursor.clone()),
  CollectionQueryResult::Resolve(x)=>(json!({"operation":"resolve","status":x.status,"reason":x.operational_reason,"confirmed_ids":x.payload.as_ref().map(|p|p.confirmed.iter().map(|r|r.assertion.id.to_string()).collect::<Vec<_>>()).unwrap_or_default(),"unverified_ids":x.payload.as_ref().map(|p|p.unverified.iter().map(|r|r.assertion.id.to_string()).collect::<Vec<_>>()).unwrap_or_default()}),None)},
 Response::Compact(r)=>match &r.result{
  CollectionCompactQueryResult::Exact(x)=>(json!({"operation":"exact","totals":x.totals,"source_totals":x.source_totals.iter().map(|s|json!({"source":s.source.id,"occurrences":s.occurrences})).collect::<Vec<_>>(),"assertion_ids":x.groups.iter().flat_map(|g|g.hits.iter().map(|h|h.assertion.id.to_string())).collect::<Vec<_>>(),"groups":x.groups.len()}),x.next_cursor.clone()),
  CollectionCompactQueryResult::Resolve(x)=>(json!({"operation":"resolve","status":x.status,"reason":x.operational_reason,"confirmed_ids":x.payload.as_ref().map(|p|p.confirmed.iter().map(|r|r.assertion.id.to_string()).collect::<Vec<_>>()).unwrap_or_default(),"unverified_ids":x.payload.as_ref().map(|p|p.unverified.iter().map(|r|r.assertion.id.to_string()).collect::<Vec<_>>()).unwrap_or_default()}),None)}
}}
fn check_facts(w:&Work,f:&Value,tiny:bool){
 let id=w.id.strip_prefix("prime--").unwrap_or(&w.id);
 if w.operation=="exact" {
  let n=match id{"absent"|"excluded-sut"=>0,"farvi-lsk"=>if tiny{2}else{3},"farvi-llg"=>1,"farvi-sut"=>if tiny{2}else{1},_=>5};
  assert_eq!(f["totals"]["occurrences"].as_u64(),Some(n));
  let hits=f["assertion_ids"].as_array().unwrap();
  if w.page_size==Some(0){assert!(hits.is_empty());assert_eq!(f["groups"],0)}
  if w.page_size.is_none(){assert_eq!(hits.len() as u64,n)}
  if let Some(size)=w.page_size{assert!(hits.len()<=size)}
 }else{
  let status=match id{"unknown"=>"Unknown","unresolved"=>"Unresolved","missing"=>"NoMatch",_=>"Resolved"};
  assert_eq!(f["status"],status);
  let confirmed=f["confirmed_ids"].as_array().unwrap();let unverified=f["unverified_ids"].as_array().unwrap();
  assert!(unverified.is_empty());assert_eq!(confirmed.len(),usize::from(status=="Resolved"));
  if let Some(vlajmaji_lexicon::RelationTarget::Local(vlajmaji_lexicon::LocalRef::Assertion(a)))=&w.target {assert_eq!(confirmed[0],a.as_str())}
 }
}
fn request(w:&Work,cursors:&BTreeMap<String,Option<String>>)->DictionaryRequest{
 match w.operation.as_str(){
 "exact"=>DictionaryRequest::Exact{spelling:w.spelling.clone(),options:DictionaryExactOptions{sources:w.sources.clone(),page_size:w.page_size,cursor:match &w.cursor_from{None=>None,Some(id)=>Some(cursors.get(id).expect("declared earlier dependency").clone().expect("continuation exists"))},..Default::default()}},
 "reference"=>DictionaryRequest::Resolve{target:w.target.clone().expect("reference target")},_=>panic!("unrecognized frozen operation")}
}
fn main(){
 let process_started=Instant::now();
 let a:Vec<_>=std::env::args_os().skip(1).collect();
 if a.first().and_then(|x|x.to_str())==Some("qualify-tiny") {assert_eq!(a.len(),2);qualify_semantic_method(&PathBuf::from(&a[1]));return}
 if a.first().and_then(|x|x.to_str()).is_some_and(|s|s=="semantic-full"||s=="semantic-tiny") {
   let tiny=a[0]=="semantic-tiny";assert_eq!(a.len(),if tiny{4}else{2});let out=PathBuf::from(&a[1]);assert!(out.is_absolute()&&out.is_dir());
   let config=if tiny{PathBuf::from(&a[2])}else{PathBuf::from(CONFIG)};
   if tiny{assert!(config.is_absolute()&&config.starts_with("/tmp/vlajmaji-p01-a04-s05-cdc")&&!config.components().any(|c|c==std::path::Component::ParentDir));}
   let work=if tiny{std::fs::read(PathBuf::from(&a[3])).unwrap()}else{SPEC.as_bytes().to_vec()};semantic(&config,&work,&out,tiny);return
 }
 assert!(a.len()==4||a.len()==6,"warm-full MODE ROUND OUT or warm-tiny MODE ROUND OUT CONFIG SPEC");
 let command=a[0].to_str().unwrap();let mode=a[1].to_str().unwrap();assert!(["full","compact","readable"].contains(&mode));let round:u8=a[2].to_str().unwrap().parse().unwrap();assert!((1..=3).contains(&round));
 let out=PathBuf::from(&a[3]);assert!(out.is_absolute()&&out.is_dir());assert!(!std::fs::symlink_metadata(&out).unwrap().file_type().is_symlink());
 let tiny=command=="warm-tiny";assert!(tiny||command=="warm-full");
 let config=if tiny{assert_eq!(a.len(),6);let p=PathBuf::from(&a[4]);assert!(p.is_absolute()&&p.starts_with("/tmp/vlajmaji-p01-a04-s05-cdc")&&!p.components().any(|c|c==std::path::Component::ParentDir));p}else{assert_eq!(a.len(),4);PathBuf::from(CONFIG)};
 let workload=if tiny{std::fs::read(PathBuf::from(&a[5])).unwrap()}else{SPEC.as_bytes().to_vec()};
 let rows:Vec<Work>=serde_json::from_slice(&workload).unwrap();let template:Vec<Work>=serde_json::from_str(SPEC).unwrap();assert_eq!(rows.len(),54);
 for (w,t) in rows.iter().zip(&template){assert_eq!((&w.id,&w.operation,w.prime,w.page_size,&w.cursor_from,&w.expected_error),(&t.id,&t.operation,t.prime,t.page_size,&t.cursor_from,&t.expected_error));}
 let startup=process_started;let profiles=CollectionProfiles::read(&config).unwrap();let profile=profiles.profile(None).unwrap();
 assert_eq!(profile.root,if tiny{config.parent().unwrap().join("root")}else{PathBuf::from(ROOT)},"explicit ROOT checked BEFORE catalogue/provider access");
 let (mut engine,constructor)=if mode=="full"{let (e,t)=CollectionSearchEngine::open_configured_profiled(&profiles,None,None).unwrap();(Engine::Full(e),t)}else{let(e,t)=CollectionCompactSearchEngine::open_configured_profiled(&profiles,None,None).unwrap();(Engine::Compact(e),t)};
 let config_and_capture=startup.elapsed();
 if !tiny{let s=engine.snapshot();assert_eq!(s.local_revision,4);assert_eq!(s.profile.as_ref().unwrap().revision,5);assert_eq!(s.profile.as_ref().unwrap().name,"full");assert_eq!(s.content_id.as_deref(),Some(CONTENT));assert_eq!(s.recorded_state_sha256.as_deref(),Some(STATE));assert!(s.disabled.is_empty());assert!(s.generation_id.is_some());}
 emit(&json!({"row_type":"setup","sqlite_runtime_version":rusqlite::version(),"mode":mode,"round":round,"tiny":tiny,"config_and_capture_ns":ns(config_and_capture),"constructor_profile_ns":ns(constructor),"snapshot":engine.snapshot(),"request_spec_sha256":sha(&workload),"connection_proof":"one engine/one Unified provider by source; actual OS FD count not measured"}));
 let mut cursors=BTreeMap::new();let mut measured=0;let mut prime=0;
 for (index,w) in rows.iter().enumerate(){
  let q=request(w,&cursors);let error_started=Instant::now();
  match engine.execute(q){
   Ok((response,p))=>{
    assert!(w.expected_error.is_none());assert_eq!(p.query.total,p.total-p.payload_setup.map_or(Duration::ZERO,|s|s.total));assert_eq!(p.query.total,p.query.lookup+p.query.projection);assert!(p.query.candidate_sql<=p.query.lookup);
    assert_eq!(p.payload_setup.is_some(),index==0,"exactly first priming Exact opens payload");
    let (facts,cursor)=facts(&response);check_facts(w,&facts,tiny);cursors.insert(w.id.clone(),cursor);
    let serialize_started=Instant::now();let bytes=wire(response,mode);let serialization=serialize_started.elapsed();
    let row=json!({"row_type":"request","id":w.id,"identity":w.id.strip_prefix("prime--").unwrap_or(&w.id),"prime":w.prime,"mode":mode,"round":round,"operation":w.operation,"ok":true,"total_ns":ns(p.total),"payload_setup_ns":p.payload_setup.map(|s|ns(s.total)),"query_total_ns":ns(p.query.total),"lookup_ns":ns(p.query.lookup),"projection_ns":ns(p.query.projection),"candidate_sql_ns":ns(p.query.candidate_sql),"serialization_ns":ns(serialization),"wire_bytes":bytes.len(),"wire_sha256":sha(&bytes),"facts":facts});
    if w.id=="farvi-all"{let mut f=OpenOptions::new().write(true).create_new(true).open(out.join("farvi-all-wire.txt")).unwrap();f.write_all(&bytes).unwrap();f.sync_all().unwrap();}
    // Hash/representative-file IO are observer work outside query/serialization.
    // Drop response serialization buffer BEFORE next request, no multi-body retention.
    let wire_len=bytes.len();drop(bytes);emit(&row);
    if index==0 {assert!(config_and_capture+p.payload_setup.unwrap().total<=Duration::from_secs(8),"fresh typed startup maximum");}
    if !w.prime {
      let warm=if w.operation=="reference"{p.query.total}else{p.query.lookup};
      assert!(warm<=Duration::from_millis(750),"per-request warm maximum: {}",w.id);
      if mode=="full"&&w.id=="farvi-all" {assert!(p.query.projection+serialization<=Duration::from_secs(1),"full hydration+serialization maximum");}
      if mode!="full" {assert!(p.query.projection+serialization<=Duration::from_millis(150),"compact/readable projection+serialization maximum: {}",w.id);}
      if w.id=="farvi-all"&&mode!="full" {assert!(wire_len<=if mode=="compact"{65536}else{131072},"five-hit output byte maximum");}
    }
   },
   Err(error)=>{
    let report=error.report_for("lookup");drop(error);let whole_error=error_started.elapsed();assert_eq!(Some(report.category.as_str()),w.expected_error.as_deref());
    let serialization_started=Instant::now();let bytes=json_wire(json!({"error":report}));let serialization=serialization_started.elapsed();
    emit(&json!({"row_type":"request","id":w.id,"identity":w.id.strip_prefix("prime--").unwrap_or(&w.id),"prime":w.prime,"mode":mode,"round":round,"operation":w.operation,"ok":false,"expected_error":w.expected_error,"whole_error_ns":ns(whole_error),"serialization_ns":ns(serialization),"wire_bytes":bytes.len(),"wire_sha256":sha(&bytes),"profile":null}));drop(bytes);if !w.prime{assert!(whole_error<=Duration::from_millis(750),"whole wrong-role error+report maximum");}
   }
  }
  if w.prime{prime+=1}else{measured+=1}
 }
 assert_eq!((prime,measured),(27,27));drop(engine);drop(profiles);
 emit(&json!({"row_type":"complete","mode":mode,"round":round,"prime":prime,"measured":measured,"tiny":tiny,"semantic_observer":"separate finite process; this warm stdout is summary, not actual CLI E2E"}));
}
