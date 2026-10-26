//! Prospective read-only functional caller; compile/run only after frozen handback
//! and the independent tiny controls. No store creation, verify or mutation API.
use std::{collections::BTreeMap, fs, io::Write, path::PathBuf};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use vlajmaji_lexicon::{AssertionRecord, EntityId, OccurrenceAttribution, PackId, RelationTarget};
use vlajmaji_search::{CollectionQueryError, CollectionQueryResult, CollectionSearchEngine,
    DictionaryExactOptions, DictionaryExactResponse, DictionaryQueryError, DictionaryRequest,
    ReferenceStatus};
use vlajmaji_store::CollectionProfiles;

#[derive(Deserialize)]
struct Expected { version:u32, inputs:Vec<ExpectedPack> }
#[derive(Deserialize)]
struct ExpectedPack { pack_id:String, farvi:Vec<Witness> }
#[derive(Deserialize)]
struct Witness { assertion:AssertionRecord, attribution:Option<OccurrenceAttribution>,
    release_id:EntityId, target:RelationTarget }

fn digest(bytes:&[u8])->String {format!("{:x}",Sha256::digest(bytes))}
fn private_dir(path:&std::path::Path) {
    let mut builder=fs::DirBuilder::new();
    #[cfg(unix)] {use std::os::unix::fs::DirBuilderExt;builder.mode(0o700);}
    builder.create(path).unwrap();
}
fn private_new(path:&std::path::Path,bytes:&[u8]) {
    let mut options=fs::OpenOptions::new();options.write(true).create_new(true);
    #[cfg(unix)] {use std::os::unix::fs::OpenOptionsExt;options.mode(0o600);}
    let mut file=options.open(path).unwrap();file.write_all(bytes).unwrap();file.flush().unwrap();
}
fn exact(engine:&mut CollectionSearchEngine, options:DictionaryExactOptions)->DictionaryExactResponse {
    match engine.execute(DictionaryRequest::Exact{spelling:"farvi".into(),options}).unwrap().result {
        CollectionQueryResult::Exact(response)=>*response,
        _=>panic!("wrong operation"),
    }
}
fn ids(response:&DictionaryExactResponse)->Vec<String> {
    response.groups.iter().flat_map(|group|&group.hits).map(|hit|hit.assertion.id.to_string()).collect()
}
fn validate_expected(expected:&Expected) {
    assert_eq!(expected.version,1);assert_eq!(expected.inputs.len(),4);
    let counts=expected.inputs.iter().map(|pack|(pack.pack_id.as_str(),pack.farvi.len()))
        .collect::<BTreeMap<_,_>>();
    assert_eq!(counts,BTreeMap::from([("lensisku-jbotci",2),("llg",1),("la-lojban",1),("sutysisku",1)]));
    for pack in &expected.inputs {for witness in &pack.farvi {
        let RelationTarget::Catalog(target)=&witness.target else {panic!("frozen witness is not catalogue scoped")};
        assert_eq!(target.release,witness.release_id);
        assert_eq!(target.occurrence.as_ref(),Some(&witness.assertion.occurrence));
        let attribution=witness.attribution.as_ref().unwrap();
        assert_eq!(target.native_entry_id,attribution.native_entry_id);
        assert_eq!(target.native_assertion_id,attribution.native_assertion_id);
        assert_eq!(target.source.as_str(),match pack.pack_id.as_str(){
            "lensisku-jbotci"|"la-lojban"=>"lensisku","llg"=>"llg","sutysisku"=>"sutysisku",_=>unreachable!(),
        });
        if pack.pack_id=="llg" || pack.pack_id=="sutysisku" {
            assert_eq!(target.native_entry_id.as_deref(),Some("farvi"));assert!(target.native_assertion_id.is_none());
        }
    }}
    let j=&expected.inputs.iter().find(|row|row.pack_id=="lensisku-jbotci").unwrap().farvi[0];
    let l=&expected.inputs.iter().find(|row|row.pack_id=="la-lojban").unwrap().farvi[0];
    assert_eq!(j.attribution.as_ref().unwrap().native_assertion_id.as_deref(),Some("333"));
    assert_eq!(l.attribution.as_ref().unwrap().native_assertion_id.as_deref(),Some("333"));
    assert_ne!(j.assertion.id,l.assertion.id);assert_ne!(j.release_id,l.release_id);
}
fn cli_inner_wire(bytes:&[u8])->Vec<u8> {
    let value:serde_json::Value=serde_json::from_slice(bytes).unwrap();
    assert_eq!(value["version"],1);assert_eq!(value["result"]["operation"],"exact");
    assert!(value["result"].as_object().unwrap().contains_key("response"));
    let mut wire=serde_json::to_vec(&value["result"]["response"]).unwrap();wire.push(b'\n');wire
}
fn check_cli_jb(bytes:&[u8],expected:&Expected)->serde_json::Value {
    let value:serde_json::Value=serde_json::from_slice(bytes).unwrap();
    assert_eq!(value["version"],1);assert_eq!(value["result"]["operation"],"exact");
    let response=&value["result"]["response"];
    let witnesses=&expected.inputs.iter().find(|p|p.pack_id=="lensisku-jbotci").unwrap().farvi;
    assert_eq!(witnesses.len(),2);
    assert_eq!(response["totals"]["occurrences"],witnesses.len());
    assert_eq!(response["totals"]["source_groups"],1);
    let totals=response["source_totals"].as_array().unwrap();assert_eq!(totals.len(),1);
    assert_eq!(totals[0]["source"]["id"],"lensisku");assert_eq!(totals[0]["occurrences"],2);
    let groups=response["groups"].as_array().unwrap();assert_eq!(groups.len(),1);
    assert_eq!(groups[0]["source"]["id"],"lensisku");
    let hits=groups[0]["hits"].as_array().unwrap();assert_eq!(hits.len(),2);
    let mut seen=std::collections::BTreeSet::new();
    let expected_releases=witnesses.iter().map(|w|w.release_id.clone()).collect::<std::collections::BTreeSet<_>>();
    for hit in hits {
        let assertion:AssertionRecord=serde_json::from_value(hit["assertion"].clone()).unwrap();
        assert!(seen.insert(assertion.id.clone()));
        let witness=witnesses.iter().find(|w|w.assertion.id==assertion.id).unwrap();
        assert_eq!(assertion,witness.assertion);
        assert!(hit.as_object().unwrap().contains_key("occurrence_attribution"));
        let attribution:Option<OccurrenceAttribution>=serde_json::from_value(hit["occurrence_attribution"].clone()).unwrap();
        assert_eq!(attribution,witness.attribution);
        let releases:Vec<EntityId>=serde_json::from_value(hit["release_ids"].clone()).unwrap();
        assert!(releases.contains(&witness.release_id));
        assert!(releases.iter().all(|r|expected_releases.contains(r)));
    }
    assert_eq!(seen,witnesses.iter().map(|w|w.assertion.id.clone()).collect());
    serde_json::json!({"ok":true,"scope":"JB override selected fields, not complete response-body equality","assertion_bodies_and_native_attributions":2,"source_groups":1,"lensisku_occurrences":2,"expected_release_memberships":true})
}
fn main() {
    let args=std::env::args_os().skip(1).map(PathBuf::from).collect::<Vec<_>>();
    if args.len()==3 && args[0]==std::path::Path::new("--check-cli-jb") {
        let bytes=fs::read(&args[1]).unwrap();let raw=fs::read(&args[2]).unwrap();
        let expected:Expected=serde_json::from_slice(&raw).unwrap();validate_expected(&expected);
        let mut result=check_cli_jb(&bytes,&expected);result["actual_cli_stdout_bytes"]=bytes.len().into();
        result["actual_cli_stdout_sha256"]=digest(&bytes).into();println!("{result}");return;
    }
    if args.len()==3 && args[0]==std::path::Path::new("--check-cli") {
        let stdout=fs::read(&args[1]).unwrap();let expected=fs::read(&args[2]).unwrap();
        let inner=cli_inner_wire(&stdout);assert_eq!(inner,expected);
        println!("{}",serde_json::json!({"ok":true,"scope":"complete inner Rust Value canonical bytes + LF equality",
            "actual_cli_stdout_bytes":stdout.len(),"actual_cli_stdout_sha256":digest(&stdout),
            "inner_bytes":inner.len(),"inner_sha256":digest(&inner)}));return;
    }
    if args.len()==2 && args[0]==std::path::Path::new("--qualify-expectations") {
        let bytes=fs::read(&args[1]).unwrap();
        let expected:Expected=serde_json::from_slice(&bytes).unwrap();validate_expected(&expected);
        println!("{}",serde_json::json!({"ok":true,"scope":"expected-witness typed parse only; no store access",
            "witnesses":5,"expected_source_sha256":digest(&bytes)}));return;
    }
    assert_eq!(args.len(),3,"CONFIG EXPECTED OUT (all explicit native paths)");
    let expected_bytes=fs::read(&args[1]).unwrap();
    let expected:Expected=serde_json::from_slice(&expected_bytes).unwrap();
    validate_expected(&expected);
    let expected_ids=expected.inputs.iter().flat_map(|pack|&pack.farvi)
        .map(|witness|witness.assertion.id.to_string()).collect::<std::collections::BTreeSet<_>>();
    assert_eq!(expected_ids.len(),5);
    private_dir(&args[2]); // Fresh, task-owned receipt leaf only.
    let profiles=CollectionProfiles::read(&args[0]).unwrap();
    let mut engine=CollectionSearchEngine::open_configured(&profiles,None,None).unwrap();
    assert_eq!(engine.selection().packs.len(),4);
    let full=exact(&mut engine,Default::default());
    assert_eq!(full.totals.occurrences,5); assert_eq!(full.totals.source_groups,3);
    assert_eq!(ids(&full).into_iter().collect::<std::collections::BTreeSet<_>>(),expected_ids);
    let totals:BTreeMap<_,_>=full.source_totals.iter()
        .map(|row|(row.source.id.to_string(),row.occurrences)).collect();
    assert_eq!(totals,BTreeMap::from([("lensisku".into(),3),("llg".into(),1),("sutysisku".into(),1)]));
    let mut wire=serde_json::to_vec(&serde_json::to_value(&full).unwrap()).unwrap();wire.push(b'\n');
    assert_eq!(wire.len(),7_160_366);
    assert_eq!(digest(&wire),"144b8b09b6e378243b36c8e01ab06353c3d573459668c8c158a86aebbea73556");
    private_new(&args[2].join("full-inner-wire.json"),&wire);drop(wire);
    println!("PASS accepted complete inner Value+LF wire, exact five frozen identities and 3/1/1 source totals");

    let lsk=exact(&mut engine,DictionaryExactOptions{sources:vec!["LSK".into()],..Default::default()});
    assert_eq!(lsk.totals.occurrences,3);assert_eq!(lsk.totals.source_groups,1);
    drop(lsk);
    let ordered=ids(&full);
    for size in [0,1,2,3] {
        let mut cursor=None;let mut all_ids=Vec::new();let mut count=0;
        loop {
            let page=exact(&mut engine,DictionaryExactOptions{page_size:Some(size),cursor,..Default::default()});
            assert_eq!(page.totals,full.totals);assert_eq!(page.source_totals,full.source_totals);
            all_ids.extend(ids(&page));count+=1;cursor=page.next_cursor;
            if cursor.is_none(){break;}assert!(count<=5);
        }
        assert_eq!(all_ids,if size==0 {Vec::new()} else {ordered.clone()});
    }
    drop(full);
    println!("PASS Lensisku alias and complete size 0/1/2/3 page chains/full totals/order");
    let mut refs=0;
    for pack in &expected.inputs {for witness in &pack.farvi {
        let response=match engine.execute(DictionaryRequest::Resolve{target:witness.target.clone()}).unwrap().result {
            CollectionQueryResult::Resolve(response)=>response,_=>panic!("wrong operation"),
        };
        assert_eq!(response.target,witness.target);assert_eq!(response.status,ReferenceStatus::Resolved);
        let payload=response.payload.unwrap();assert_eq!(payload.confirmed.len(),1);assert!(payload.unverified.is_empty());
        assert_eq!(payload.confirmed[0].assertion,witness.assertion);
        assert_eq!(payload.confirmed[0].occurrence_attribution,witness.attribution);
        assert!(payload.confirmed[0].release_ids.contains(&witness.release_id));
        refs+=1;
    }}
    assert_eq!(refs,5);
    // Check wrong-role against the already loaded all-pack engine; do not load
    // another complete model merely for this negative.
    let jb=&expected.inputs.iter().find(|row|row.pack_id=="lensisku-jbotci").unwrap().farvi[0];
    let target=RelationTarget::Local(vlajmaji_lexicon::LocalRef::Entry(jb.assertion.id.clone()));
    assert!(matches!(engine.execute(DictionaryRequest::Resolve{target}),
        Err(CollectionQueryError::Query(DictionaryQueryError::InvalidReference{..}))));
    drop(engine); // Complete all-pack model released before selected setup.
    println!("PASS five exact prior-input reference witnesses including separate JB/La native333");
    let mut selections=Vec::new();
    for (pack,total) in [(PackId::LensiskuJbotci,2),(PackId::LaLojban,1)] {
        let mut selected=CollectionSearchEngine::open_configured(&profiles,None,Some(&[pack])).unwrap();
        let response=exact(&mut selected,Default::default());
        assert_eq!(response.totals.occurrences,total);assert_eq!(response.totals.source_groups,1);
        let expected_set=expected.inputs.iter().find(|row|row.pack_id==pack.as_str()).unwrap()
            .farvi.iter().map(|row|row.assertion.id.to_string()).collect::<std::collections::BTreeSet<_>>();
        assert_eq!(ids(&response).into_iter().collect::<std::collections::BTreeSet<_>>(),expected_set);
        selections.push((pack.as_str(),total));
        drop(response);
        drop(selected); // Explicit provider/model lifetime boundary per pack.
    }
    println!("PASS JB-only/La-only pack selections retain separate received identities within Lensisku");
    let summary=serde_json::json!({"ok":true,"scope":"readonly Full functional proof, not timing",
        "expected_source_sha256":digest(&expected_bytes),"farvi_occurrences":5,"source_totals":totals,
        "inner_wire_bytes":7160366,"inner_wire_sha256":"144b8b09b6e378243b36c8e01ab06353c3d573459668c8c158a86aebbea73556",
        "reference_witnesses":refs,"positive_page_sizes":[1,2,3],"zero_page_size":0,"pack_selections":selections});
    private_new(&args[2].join("summary.json"),&serde_json::to_vec_pretty(&summary).unwrap());
    println!("{summary}");
}

#[cfg(test)]
mod tests {
    use super::*;
    fn frozen()->Expected {
        serde_json::from_slice(include_bytes!("../expected-from-s02.json")).unwrap()
    }
    #[test]
    fn test_expected_witnesses_positive_and_changed_reference_presence_pack_reject() {
        validate_expected(&frozen());
        let mut changed=frozen();
        let RelationTarget::Catalog(target)=&mut changed.inputs[0].farvi[0].target else{unreachable!()};
        target.release=EntityId::new("wrong-release").unwrap();
        assert!(std::panic::catch_unwind(||validate_expected(&changed)).is_err());
        let mut changed=frozen();
        let RelationTarget::Catalog(target)=&mut changed.inputs[0].farvi[0].target else{unreachable!()};
        target.occurrence=None;
        assert!(std::panic::catch_unwind(||validate_expected(&changed)).is_err());
        let mut changed=frozen();changed.inputs.pop();
        assert!(std::panic::catch_unwind(||validate_expected(&changed)).is_err());
        let mut changed=frozen();
        let RelationTarget::Catalog(target)=&mut changed.inputs[1].farvi[0].target else{unreachable!()};
        target.native_assertion_id=Some("invented-role".into());
        assert!(std::panic::catch_unwind(||validate_expected(&changed)).is_err());
    }
    #[test]
    fn test_cli_inner_wire_complete_value_and_presence_extra_number_lexeme() {
        let wire=cli_inner_wire(br#"{"version":1,"result":{"operation":"exact","response":{"optional":null,"number":1e9999}}}"#);
        assert_eq!(wire,b"{\"number\":1e9999,\"optional\":null}\n");
        let missing=cli_inner_wire(br#"{"version":1,"result":{"operation":"exact","response":{"number":1e9999}}}"#);
        let extra=cli_inner_wire(br#"{"version":1,"result":{"operation":"exact","response":{"optional":null,"number":1e9999,"extra":[]}}}"#);
        assert_ne!(wire,missing);assert_ne!(wire,extra);
    }
    #[test]
    fn test_cli_jb_projection_positive_changed_body_attribution_release_missing_and_duplicate_reject() {
        let expected=frozen();
        let witnesses=&expected.inputs.iter().find(|p|p.pack_id=="lensisku-jbotci").unwrap().farvi;
        // Minimal synthetic checker input: proves the declared selected-field
        // predicates, not a manufactured full CLI response or product result.
        let hits=witnesses.iter().map(|w|serde_json::json!({"assertion":w.assertion,"occurrence_attribution":w.attribution,"release_ids":[w.release_id]})).collect::<Vec<_>>();
        let value=serde_json::json!({"version":1,"result":{"operation":"exact","response":{"totals":{"occurrences":2,"source_groups":1},"source_totals":[{"source":{"id":"lensisku"},"occurrences":2}],"groups":[{"source":{"id":"lensisku"},"hits":hits}]}}});
        check_cli_jb(&serde_json::to_vec(&value).unwrap(),&expected);
        for change in 0..5 {
            let mut altered=value.clone();let hit=&mut altered["result"]["response"]["groups"][0]["hits"][0];
            match change {
                0=>hit["assertion"]["definition"]="Missing".into(),
                1=>hit["occurrence_attribution"]["native_assertion_id"]="wrong-native".into(),
                2=>hit["release_ids"]=serde_json::json!(["wrong-release"]),
                3=>{hit.as_object_mut().unwrap().remove("release_ids");},
                _=>{altered["result"]["response"]["groups"][0]["hits"][1]=altered["result"]["response"]["groups"][0]["hits"][0].clone();},
            }
            assert!(std::panic::catch_unwind(||check_cli_jb(&serde_json::to_vec(&altered).unwrap(),&expected)).is_err());
        }
    }
}
