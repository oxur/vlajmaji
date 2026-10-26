// Independent readonly S02 descriptor/query observer. Never creates a store.
use std::{collections::{BTreeMap,BTreeSet},path::Path,io::Write};
use sha2::{Digest,Sha256};
use serde_json::{Value,json};
use vlajmaji_lexicon::*;
use vlajmaji_store::CollectionCatalogue;
use vlajmaji_search::{DictionarySearchEngine as Engine,DictionaryExactOptions as Opt,DictionaryQueryError as QE,ReferenceStatus};
fn sha(b:&[u8])->String {format!("{:x}",Sha256::digest(b))}
fn value<T:serde::Serialize>(v:&T)->Value {serde_json::to_value(v).unwrap()}
fn digest<T:serde::Serialize>(v:&T)->String {sha(&serde_json::to_vec(v).unwrap())}
fn fp(v:&Value)->String {digest(v)}
fn pack(s:&str)->PackId {match s {"lensisku-jbotci"=>PackId::LensiskuJbotci,"llg"=>PackId::Llg,"la-lojban"=>PackId::LaLojban,"sutysisku"=>PackId::Sutysisku,_=>panic!("unknown declared pack")}}
fn expected(d:&UnverifiedDictionaryDocument)->Value {
    let m=&d.metadata;
    let entries:BTreeMap<_,_>=m.entries.iter().map(|e|(&e.id,e)).collect();
    let assertions:BTreeMap<_,_>=m.assertions.iter().map(|a|(&a.id,a)).collect();
    let mut entities=BTreeMap::<String,Value>::new();let mut bodies=BTreeMap::<String,String>::new();
    for e in &m.entries {entities.insert(format!("entry:{}",e.id),json!({"role":"entry","id":e.id,"entry_ids":[],"assertion_ids":[],"source_ids":[e.source],"occurrence":null,"evidence":null,"spelling":e.spelling}));bodies.insert(format!("entry:{}",e.id),digest(e));}
    for a in &m.assertions {entities.insert(format!("assertion:{}",a.id),json!({"role":"assertion","id":a.id,"entry_ids":[a.entry],"assertion_ids":[],"source_ids":[entries[&a.entry].source],"occurrence":a.occurrence,"evidence":null,"spelling":null}));bodies.insert(format!("assertion:{}",a.id),digest(a));}
    for f in &m.forms {entities.insert(format!("form:{}",f.id),json!({"role":"form","id":f.id,"entry_ids":[f.entry],"assertion_ids":[],"source_ids":[entries[&f.entry].source],"occurrence":f.evidence.as_ref().map(|e|&e.occurrence),"evidence":f.evidence,"spelling":null}));bodies.insert(format!("form:{}",f.id),digest(f));}
    for s in &m.editorial_senses {let es:BTreeSet<_>=s.assertions.iter().map(|a|assertions[a].entry.clone()).collect();let ss:BTreeSet<_>=es.iter().map(|e|entries[e].source.clone()).collect();entities.insert(format!("editorial_sense:{}",s.id),json!({"role":"editorial_sense","id":s.id,"entry_ids":es,"assertion_ids":s.assertions,"source_ids":ss,"occurrence":null,"evidence":null,"spelling":null}));bodies.insert(format!("editorial_sense:{}",s.id),digest(s));}
    macro_rules! body {($tag:literal,$field:ident)=>{for r in &m.$field {assert!(bodies.insert(format!("{}:{}",$tag,r.id),digest(r)).is_none());}}}
    body!("context",contexts);body!("frame",frames);body!("etymology",etymologies);body!("relation",relations);body!("owned_revision",owned_revisions);body!("computation",computations);body!("annotation",annotations);
    for h in &m.owned_heads {bodies.insert(format!("owned_head:{}",h.source),digest(h));}
    let captures:Vec<_>=d.captures.iter().map(|c|{let occurrences:Vec<_>=c.occurrences.iter().map(|o|OccurrenceRef{capture:c.identity.clone(),locator:o.locator.clone()}).collect();json!({"capture":c.identity,"metadata":c.metadata,"byte_length":c.bytes.len(),"occurrences":occurrences})}).collect();
    let mut counts=BTreeMap::new();macro_rules! count {($($f:ident),*)=>{$(counts.insert(stringify!($f),m.$f.len());)*}}
    count!(entries,assertions,contexts,frames,forms,etymologies,relations,owned_revisions,owned_heads,editorial_senses,computations,annotations,extensions);
    json!({"registry":fp(&value(&d.registry)),"source_metadata":fp(&json!({"collections":m.collections,"components":m.components,"releases":m.releases,"code_notices":m.code_notices})),"captures":fp(&value(&captures)),"entities":fp(&value(&entities)),"body_sha256":fp(&value(&bodies)),"extension_row_sha256_ordered_multiset":fp(&value(&m.extensions.iter().map(digest).collect::<Vec<_>>())),"category_counts":counts,"dictionary_version":d.version,"registry_version":d.registry.version,"upstream_catalogue_revision":m.catalogue_revision})
}
fn actual(d:&vlajmaji_store::CollectionReferenceMetadata)->Value {
    let entities:BTreeMap<_,_>=d.entities.iter().map(|e|(format!("{}:{}",e.role,e.id),value(e))).collect();assert_eq!(entities.len(),d.entities.len());
    json!({"registry":fp(&value(&d.registry)),"source_metadata":fp(&value(&d.source_metadata)),"captures":fp(&value(&d.captures)),"entities":fp(&value(&entities)),"body_sha256":fp(&value(&d.body_sha256)),"extension_row_sha256_ordered_multiset":fp(&value(&d.extension_row_sha256_ordered_multiset)),"category_counts":d.category_counts,"dictionary_version":d.dictionary_version,"registry_version":d.registry_version,"upstream_catalogue_revision":d.upstream_catalogue_revision})
}
fn freeze(manifest:&Path,output:&Path){
    let batch:Value=serde_json::from_slice(&std::fs::read(manifest).unwrap()).unwrap();let mut result=Vec::new();
    for item in batch["inputs"].as_array().unwrap(){let p=item["pack_id"].as_str().unwrap();let bytes=std::fs::read(item["release"].as_str().unwrap()).unwrap();let seal=PackSeal::from_json_bytes(&std::fs::read(item["seal"].as_str().unwrap()).unwrap()).unwrap();let d=UnverifiedDictionaryDocument::from_unverified_json_bytes(&bytes).unwrap();
        let mut witnesses=Vec::new();for a in &d.metadata.assertions {if a.eligibility!=Eligibility::Lexical {continue}let e=d.metadata.entries.iter().find(|e|e.id==a.entry).unwrap();if e.spelling!=Presence::Value("farvi".into()){continue}let r=d.metadata.releases.iter().find(|r|r.assertions.contains(&a.id)).unwrap();let at=d.registry.occurrences.iter().find(|o|o.occurrence==a.occurrence.to_record_identity().unwrap()).unwrap();let target=RelationTarget::Catalog(CatalogTarget{source:e.source.clone(),release:r.id.clone(),native_entry_id:at.native_entry_id.clone(),native_assertion_id:at.native_assertion_id.clone(),occurrence:Some(a.occurrence.clone())});witnesses.push(json!({"target":target,"assertion":a,"attribution":at,"release_id":r.id}));}
        result.push(json!({"pack_id":p,"release_sha256":sha(&bytes),"payload_kind":match seal.payload_kind{PackPayloadKind::Full=>"full",PackPayloadKind::Synthetic=>"synthetic"},"descriptor":expected(&d),"farvi":witnesses}));
    }
    let bytes=serde_json::to_vec_pretty(&json!({"version":1,"inputs":result})).unwrap();let mut out=std::fs::OpenOptions::new().write(true).create_new(true).open(output).unwrap();out.write_all(&bytes).unwrap();out.sync_all().unwrap();println!("{}",json!({"ok":true,"mode":"freeze accepted-input expectations before install","file":output,"sha256":sha(&bytes),"packs":4}));
}
fn check(root:&Path,expected_path:&Path,queries:bool){
    let expected:Value=serde_json::from_slice(&std::fs::read(expected_path).unwrap()).unwrap();let cat=CollectionCatalogue::open(root).unwrap();let g=cat.generation().unwrap().unwrap();let listing=cat.list().unwrap();assert_eq!(listing.packs.len(),4);let mut controls=0;
    for item in expected["inputs"].as_array().unwrap(){let p=item["pack_id"].as_str().unwrap();let desc=cat.reference_metadata(pack(p),item["release_sha256"].as_str().unwrap()).unwrap();assert_eq!(desc.version,1);assert_eq!(desc.payload_kind,item["payload_kind"]);assert_eq!(actual(&desc),item["descriptor"],"descriptor mismatch {p}");controls+=1;}
    // Comparator negatives use actual independent expected block values, not only counts.
    let first=&expected["inputs"][0]["descriptor"];let mut changed=first.clone();changed["registry"]=json!("wrong digest");assert_ne!(first,&changed);changed=first.clone();changed.as_object_mut().unwrap().remove("body_sha256");assert_ne!(first,&changed);controls+=2;
    if !queries {println!("{}",json!({"ok":true,"mode":"descriptor-only qualification","controls":controls}));return}
    assert_eq!(g.canonical_sha256,"d9d73cbb49cc5a9ddb0ec16cb6dbb4bb7260ad49f4f8f54c6e6d21f0779b7b19");assert_eq!(listing.local_revision,1);assert!(listing.staged_attempts.is_empty());drop(cat);
    let engine=Engine::open_snapshot(vlajmaji_store::DictionaryReadTopology::Unified,g.snapshot_manifest()).unwrap();let full=engine.exact("farvi",Opt::default()).unwrap();assert_eq!(full.totals.occurrences,5);assert_eq!(full.totals.source_groups,3);
    let totals:BTreeMap<_,_>=full.source_totals.iter().map(|x|(x.source.id.to_string(),x.occurrences)).collect();assert_eq!(totals,[("lensisku".into(),3),("llg".into(),1),("sutysisku".into(),1)].into_iter().collect());
    let wire=[serde_json::to_vec(&value(&full)).unwrap(),vec![b'\n']].concat();assert_eq!(wire.len(),7160366);assert_eq!(sha(&wire),"144b8b09b6e378243b36c8e01ab06353c3d573459668c8c158a86aebbea73556");controls+=2;
    for (source,n) in [("LSK",3),("LLG",1),("sutysisku",1)] {assert_eq!(engine.exact("farvi",Opt{sources:vec![source.into()],..Opt::default()}).unwrap().totals.occurrences,n);controls+=1;}
    let ids=full.groups.iter().flat_map(|g|g.hits.iter().map(|h|h.assertion.id.clone())).collect::<Vec<_>>();for size in [1,2,3]{let mut cursor=None;let mut seen=Vec::new();let mut pages=0;loop{pages+=1;assert!(pages<=5);let page=engine.exact("farvi",Opt{page_size:Some(size),cursor,..Opt::default()}).unwrap();assert_eq!(page.totals,full.totals);seen.extend(page.groups.into_iter().flat_map(|g|g.hits.into_iter().map(|h|h.assertion.id)));cursor=page.next_cursor;if cursor.is_none(){break}}assert_eq!(seen,ids);controls+=1;}
    let mut refids=Vec::new();for item in expected["inputs"].as_array().unwrap(){for witness in item["farvi"].as_array().unwrap(){let target:RelationTarget=serde_json::from_value(witness["target"].clone()).unwrap();let answer=engine.resolve(&target).unwrap();assert_eq!(answer.status,ReferenceStatus::Resolved);assert_eq!(answer.target,target);assert_eq!(answer.confirmed.len(),1);assert!(answer.unverified.is_empty());let hit=&answer.confirmed[0];assert_eq!(value(&hit.assertion),witness["assertion"]);assert_eq!(value(&hit.occurrence_attribution),witness["attribution"]);assert!(hit.release_ids.iter().any(|r|value(r)==witness["release_id"]));assert_eq!(value(&answer.requested_release.as_ref().unwrap().id),witness["release_id"]);refids.push(hit.assertion.id.clone());controls+=1;}}
    assert_eq!(refids.len(),5);assert_eq!(refids.iter().collect::<BTreeSet<_>>().len(),5);assert!(matches!(engine.resolve(&RelationTarget::Local(LocalRef::Entry(refids[0].clone()))),Err(QE::InvalidReference{..})));assert!(engine.resolve(&RelationTarget::Local(LocalRef::Assertion(refids[0].clone()))).is_ok());controls+=2;
    println!("{}",json!({"ok":true,"mode":"readonly full catalogue public consumer","controls":controls,"source_totals":totals,"farvi_wire_bytes":wire.len(),"farvi_wire_sha256":sha(&wire),"refs":refids,"store_path":g.store_path,"generation":g.generation_id,"canonical_sha256":g.canonical_sha256}));
}
fn main(){let args=std::env::args_os().collect::<Vec<_>>();match args[1].to_str().unwrap(){"freeze"=>freeze(Path::new(&args[2]),Path::new(&args[3])),"check-descriptors"=>check(Path::new(&args[2]),Path::new(&args[3]),false),"check-full"=>check(Path::new(&args[2]),Path::new(&args[3]),true),_=>panic!("unsupported observer mode")}}
