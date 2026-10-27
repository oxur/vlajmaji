// Prospective independent S05 INITIAL caller. Source preparation ONLY during CC ownership.
// Do not compile/run before actual frozen handback and caller method/source review.
include!("helpers.rs");
include!("supplement.rs");
include!("reference_checks.rs");
include!("cursor_checks.rs");
include!("store_projection_checks.rs");
include!("metadata_only_checks.rs");
include!("owned_paths.rs");
include!("history_checks.rs");
include!("context_checks.rs");
include!("cause_checks.rs");
include!("derived_fault_checks.rs");
include!("detail_identity_checks.rs");
include!("targeted_sentinel_checks.rs");
include!("selection_checks.rs");
include!("eager_checks.rs");
include!("public_api_checks.rs");
include!("profile_parity_checks.rs");
mod shared_body_checks;
use std::{borrow::Cow, path::Path};
use vlajmaji_search::*;
use vlajmaji_store::{CollectionCatalogue,CollectionInput,CollectionRequest,CollectionSelection};
const PACKS:[PackId;4]=[PackId::LensiskuJbotci,PackId::Llg,PackId::LaLojban,PackId::Sutysisku];
fn expected_inventory(ds:&[Dictionary])->BTreeMap<&'static str,BTreeMap<String,usize>>{
 let mut all:BTreeMap<_,BTreeMap<_,usize>>=BTreeMap::new();
 for d in ds {for(k,rows)in inventory(&draft(d.clone())){let family=all.entry(k).or_default();for(key,n)in rows{let old=family.entry(key).or_default();*old=(*old).max(n);}}}all
}
fn qualify_inventory(x:&BTreeMap<&'static str,BTreeMap<String,usize>>){
 assert_eq!(x.len(),25);let same=x.clone();assert_eq!(&same,x);
 let empty=*x.iter().find(|(_,v)|v.is_empty()).unwrap().0;
 let mut missing=x.clone();missing.remove(empty);assert_ne!(&missing,x);
 let mut changed=x.clone();*changed.values_mut().find(|v|!v.is_empty()).unwrap().values_mut().next().unwrap()+=1;assert_ne!(&changed,x);
 println!("PASS inventory method positive/empty-category/multiplicity;25families");
}
fn key(d:&UnverifiedDictionaryDocument,a:&AssertionRecord)->DictionaryHitDetailRef{
 let e=d.metadata.entries.iter().find(|e|e.id==a.entry).unwrap();
 let snap=d.registry.snapshots.iter().find(|s|s.snapshot.namespace()==a.occurrence.capture.namespace&&s.snapshot.sha256()==a.occurrence.capture.sha256).unwrap();
 let native=d.registry.occurrences.iter().find(|r|OccurrenceRef::from(&r.occurrence)==a.occurrence);
 DictionaryHitDetailRef{version:1,digest_policy:"entry-assertion-serde-json-v1".into(),role:"assertion".into(),source:e.source.clone(),entry_id:e.id.clone(),assertion_id:a.id.clone(),occurrence:a.occurrence.clone(),entry_sha256:sha(&serde_json::to_vec(e).unwrap()),assertion_sha256:sha(&serde_json::to_vec(a).unwrap()),native_entry_id:native.and_then(|r|r.native_entry_id.clone()),native_assertion_id:native.and_then(|r|r.native_assertion_id.clone()),contributor_id:native.and_then(|r|r.contributor.clone()),distributor_id:snap.distributor.clone()}
}
fn tuple_id(o:&OccurrenceRef)->String{
 let mut bytes=vec![];for part in [o.capture.namespace.as_bytes(),o.capture.sha256.as_bytes(),o.locator.as_bytes()]{bytes.extend_from_slice(&(part.len() as u64).to_be_bytes());bytes.extend_from_slice(part);}bytes.iter().map(|b|format!("{b:02x}")).collect()
}
fn query(size:Option<usize>,cursor:Option<String>)->DictionaryRequest{
 DictionaryRequest::Exact{spelling:"farvi".into(),options:DictionaryExactOptions{page_size:size,cursor,..Default::default()}}
}
fn exact(r:CollectionCompactQueryResponse)->DictionaryCompactExactResponse{
 match r.result{CollectionCompactQueryResult::Exact(r)=>*r,_=>panic!("expected exact")}
}
fn selected(root:&Path)->CollectionCompactSearchEngine{
 CollectionCompactSearchEngine::open(root,CollectionQuerySelection{packs:PACKS.to_vec(),disabled:vec![],profile:None}).unwrap()
}
fn main(){
 public_api_checks();
 let base=std::env::args_os().nth(1).expect("explicit NEW owned caller directory");let base=std::path::PathBuf::from(base);
 assert!(base.is_absolute(),"caller requires an explicit absolute owned base");
 std::fs::create_dir(&base).expect("exclusive NEW base; never pre-delete or reuse");let root=base.join("catalogue");
 let input=models();let want=expected_inventory(&input);qualify_inventory(&want);
 let docs:Vec<_>=input.iter().cloned().map(draft).collect();
 let expected_keys:BTreeMap<_,_>=docs.iter().flat_map(|d|d.metadata.assertions.iter().filter(|a|matches!(a.eligibility,Eligibility::Lexical)).map(move|a|(a.id.to_string(),key(d,a)))).collect();assert_eq!(expected_keys.len(),5);
 let mut expected_order:Vec<_>=docs.iter().flat_map(|d|d.metadata.assertions.iter().filter(|a|matches!(a.eligibility,Eligibility::Lexical)).map(move|a|{let e=d.metadata.entries.iter().find(|e|e.id==a.entry).unwrap();let spelling=match &e.spelling{Presence::Value(x)=>x.clone(),_=>String::new()};(e.source.to_string(),spelling,tuple_id(&a.occurrence),a.id.to_string())})).collect();expected_order.sort();let expected_order:Vec<_>=expected_order.into_iter().map(|x|x.3).collect();
 let mut missing_presence=docs[0].clone();missing_presence.metadata.entries[0].spelling=Presence::Missing;assert_ne!(inventory(&missing_presence),inventory(&docs[0]));println!("PASS comparator Presence mutation differs before candidate publication");
 let sealed:Vec<_>=PACKS.iter().zip(&input).map(|(p,d)|{let s=seal_release(*p,PackPayloadKind::Synthetic,d).unwrap();(*p,s.release,s.seal)}).collect();
 let req=CollectionRequest::new(sealed.iter().map(|(pack_id,release,seal)|CollectionInput::SealedBytes{pack_id:*pack_id,release:Cow::Borrowed(release),seal:Cow::Borrowed(seal)}).collect(),CollectionSelection::Packs(PACKS.to_vec()),true).unwrap();
 let installed=CollectionCatalogue::install(&root,req).unwrap();assert_eq!(installed.local_revision,1);let generation=installed.generation.unwrap();assert_owned_store_path(&root,&generation.store_path);
 let db=root.join("catalogue.sqlite3");let before=sha(&std::fs::read(&db).unwrap());
 {let s=vlajmaji_store::DictionaryStore::open(&generation.store_path).unwrap();assert_eq!(inventory(&s.unverified_document().unwrap()),want);}
 println!("PASS original four-pack pre-input25family installed equality");
 profile_parity_checks(&root, &generation.store_path, &docs);
 let mut engine=selected(&root);let all=exact(engine.execute(query(None,None)).unwrap());
 assert_eq!(all.totals.occurrences,5);assert_eq!(all.totals.source_groups,3);
 let ordered:Vec<_>=all.groups.iter().flat_map(|g|g.hits.iter().map(|h|h.assertion.id.to_string())).collect();
 assert_eq!(ordered,expected_order);assert_eq!(ordered.iter().cloned().collect::<std::collections::BTreeSet<_>>(),expected_keys.keys().cloned().collect());
 let totals:BTreeMap<_,_>=all.source_totals.iter().map(|s|(s.source.id.to_string(),s.occurrences)).collect();assert_eq!(totals,BTreeMap::from([("lensisku".into(),2),("llg".into(),1),("sutysisku".into(),2)]));
 for h in all.groups.iter().flat_map(|g|&g.hits){assert_eq!(&h.detail_ref,&expected_keys[&h.assertion.id.to_string()]);let d=docs.iter().find(|d|d.metadata.assertions.iter().any(|a|a.id==h.assertion.id)).unwrap();assert_eq!(h.assertion,d.metadata.assertions.iter().find(|a|a.id==h.assertion.id).unwrap().clone());assert_eq!(h.entry,d.metadata.entries.iter().find(|e|e.id==h.entry.id).unwrap().clone());}
 println!("PASS projected fields/5original keys/native roles/totals/source2-1-2");
 let zero=exact(engine.execute(query(Some(0),None)).unwrap());assert_eq!(zero.totals,all.totals);assert!(zero.groups.is_empty());assert!(zero.next_cursor.is_none());
 for size in [1,2,3]{let mut ids=vec![];let mut cursor=None;loop{let page=exact(selected(&root).execute(query(Some(size),cursor)).unwrap());assert_eq!(page.totals,all.totals);ids.extend(page.groups.iter().flat_map(|g|g.hits.iter().map(|h|h.assertion.id.to_string())));cursor=page.next_cursor;if cursor.is_none(){break}}assert_eq!(ids,ordered);}
 println!("PASS page0 and every1/2/3 concatenation through FRESH engines");
 {let full=DictionarySearchEngine::open(&generation.store_path).unwrap();let full=full.exact("farvi",DictionaryExactOptions::default()).unwrap();let hits:BTreeMap<_,_>=full.groups.into_iter().flat_map(|g|g.hits.into_iter().map(|h|(h.assertion.id.to_string(),h))).collect();for(id,k)in &expected_keys{let result=engine.detail(k,None).unwrap();assert_eq!(result.response.status,ReferenceStatus::Resolved);assert_eq!(result.response.hit.as_ref(),Some(&hits[id]));assert_eq!(result.response.current_eligible,Some(true));}}
 println!("PASS all5 pre-input detail keys recover COMPLETE current full DictionaryHit");
 store_projection_checks(&root,&docs,&sealed);
 original_reference_checks(&root,&docs);
 detail_identity_checks(&root,&generation.store_path,&docs);
 selection_checks(&root,&docs);
 let first=expected_keys.values().next().unwrap();let bytes=serde_json::to_vec(first).unwrap();assert_eq!(DictionaryHitDetailRef::from_json_bytes(&bytes).unwrap(),*first);
 for name in ["native_entry_id","native_assertion_id","contributor_id","distributor_id"]{let mut v:serde_json::Value=serde_json::from_slice(&bytes).unwrap();v.as_object_mut().unwrap().remove(name);assert_eq!(DictionaryHitDetailRef::from_json_bytes(&serde_json::to_vec(&v).unwrap()).unwrap_err().report().code,"invalid_detail_reference");}
 let mut opaque=first.clone();opaque.occurrence.capture.namespace="capture/β🧬".into();opaque.occurrence.locator="opaque locator/β\n".into();opaque.validate().unwrap();assert_eq!(DictionaryHitDetailRef::from_json_bytes(&serde_json::to_vec(&opaque).unwrap()).unwrap(),opaque);
 let mut bad=first.clone();bad.role="entry".into();assert_eq!(bad.validate().unwrap_err().report().code,"invalid_detail_reference");
 assert_eq!(validate_detail_snapshot_sha256("é").unwrap_err().report().code,"malformed_snapshot");
 let stale="0".repeat(64);assert_eq!(engine.detail(first,Some(&stale)).unwrap_err().report_for("lookup_detail").category,"stale_snapshot");
 println!("PASS strict-nullable/key-role/snapshot negatives and valid counterpart");
 drop(engine);assert_eq!(sha(&std::fs::read(&db).unwrap()),before);
 {let reference_request=DictionaryRequest::Resolve{target:RelationTarget::Local(LocalRef::Assertion(expected_keys.values().next().unwrap().assertion_id.clone()))};
 let context=serde_json::json!({"fixture_scope":"original-four-pack-synthetic","root":root,"store":generation.store_path,"detail_keys":expected_keys.values().collect::<Vec<_>>(),"reference_request":reference_request});
 let file=std::fs::OpenOptions::new().write(true).create_new(true).open(base.join("caller-context.json")).unwrap();serde_json::to_writer(file,&context).unwrap();}

 cursor_checks(&generation.store_path);
 supplement(&base);
 context_checks(&base);
 eager_checks(&base);
 derived_fault_checks(&base, &docs, &sealed);
 targeted_sentinel_checks(&base, &docs, &sealed);
 remove_all_checks(&base, &docs, &sealed);
 metadata_only_checks(&base,&docs,&sealed);
 history_checks(&root,&docs,&sealed);
 shared_body_checks::run(&base.join("shared-assertion"));
 shared_body_checks::run_entry(&base.join("shared-entry"));
 // Baseline CLI wire comparison and supplemental linked detail/state controls are added at actual freeze,
 // using the preserved baseline binary and the final initial API, without invoking Full data.
 println!("CALLER INITIAL PROSPECTIVE CORE COMPLETE (actual run only after frozen review)");
}
