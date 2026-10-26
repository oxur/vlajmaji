// Independent query controls, prospective until the candidate freezes.
use std::{collections::BTreeMap, path::Path};
use vlajmaji_lexicon::PackId;
use vlajmaji_store::{CollectionCatalogue, CollectionProfileWriteRequest, CollectionProfiles};
use vlajmaji_search::{CollectionSearchEngine, CollectionQuerySelection, CollectionQueryResult, CollectionQueryError, DictionaryExactOptions, DictionaryExactResponse, DictionaryRequest, DictionarySearchEngine};

fn request(size:Option<usize>, cursor:Option<String>, sources:Vec<String>) -> DictionaryRequest {
    DictionaryRequest::Exact {spelling:"farvi".into(),options:DictionaryExactOptions {page_size:size,cursor,sources,..Default::default()}}
}
fn exact(engine:&mut CollectionSearchEngine, size:Option<usize>, cursor:Option<String>, sources:Vec<String>) -> DictionaryExactResponse {
    match engine.execute(request(size,cursor,sources)).unwrap().result {
        CollectionQueryResult::Exact(response) => *response,
        _ => panic!("exact request returned a different operation"),
    }
}
fn open(root:&Path,packs:&[PackId],disabled:&[PackId]) -> CollectionSearchEngine {
    CollectionSearchEngine::open(root,CollectionQuerySelection{packs:packs.to_vec(),disabled:disabled.to_vec(),profile:None}).unwrap()
}
fn ids(response:&DictionaryExactResponse)->Vec<String> {
    response.groups.iter().flat_map(|group| &group.hits).map(|hit| hit.assertion.id.to_string()).collect()
}

pub fn run(base:&Path,root:&Path,packs:&[PackId]) {
    let catalogue=CollectionCatalogue::open(root).unwrap();
    let generation=catalogue.generation().unwrap().unwrap();drop(catalogue);
    let legacy=DictionarySearchEngine::open(&generation.store_path).unwrap();
    let previous=legacy.exact("farvi",DictionaryExactOptions::default()).unwrap();
    let mut wire=serde_json::to_vec(&serde_json::to_value(&previous).unwrap()).unwrap();wire.push(b'\n');
    std::fs::write(base.join("expected-old-unpaged-wire.json"),wire).unwrap();
    let full=exact(&mut open(root,packs,&[]),None,None,Vec::new());
    assert_eq!(full,previous);
    assert_eq!(full.totals.occurrences,5);assert_eq!(full.totals.source_groups,3);
    let totals:BTreeMap<_,_>=full.source_totals.iter().map(|row|(row.source.id.to_string(),row.occurrences)).collect();
    assert_eq!(totals,BTreeMap::from([("lensisku".into(),2),("llg".into(),1),("sutysisku".into(),2)]));
    println!("PASS complete unpaged old-route compatibility and literal source-attributed totals");

    for size in [0,1,2,3] {
        let mut cursor=None;let mut accumulated=Vec::new();let mut pages=0;
        loop {
            // Reopen on every page: cursor validity must survive a fresh engine.
            let page=exact(&mut open(root,packs,&[]),Some(size),cursor,Vec::new());
            assert_eq!(page.totals,full.totals);
            assert_eq!(page.source_totals,full.source_totals);
            accumulated.extend(ids(&page));pages+=1;
            cursor=page.next_cursor;
            if cursor.is_none(){break;}assert!(pages<=5,"finite five-hit page chain");
        }
        if size==0 {assert!(accumulated.is_empty());}else{assert_eq!(accumulated,ids(&full));}
        println!("PASS page size {size} complete totals and fresh-engine ordered continuation");
    }
    let source=exact(&mut open(root,packs,&[]),None,None,vec!["LSK".into()]);
    assert_eq!(source.totals.occurrences,2);assert_eq!(source.totals.source_groups,1);
    assert!(source.groups.iter().all(|group|group.source.id.as_str()=="lensisku"));
    for pack in [PackId::LensiskuJbotci,PackId::LaLojban] {
        let response=exact(&mut open(root,&[pack],&[]),None,None,Vec::new());
        assert_eq!(response.totals.occurrences,1);assert_eq!(response.totals.source_groups,1);
        assert!(response.groups.iter().all(|group|group.source.id.as_str()=="lensisku"));
    }
    println!("PASS shared source alias and independently selectable JB versus La receipts");

    let file=base.join("profiles.toml");
    let profiles=CollectionProfiles::read(&file).unwrap();
    let mut old=CollectionSearchEngine::open_configured(&profiles,None,None).unwrap();
    let first=exact(&mut old,Some(1),None,Vec::new());let saved=first.next_cursor.clone().unwrap();
    let parts:Vec<_>=saved.split(':').collect();assert_eq!(parts.len(),3);
    let cursor_bytes:Vec<u8>=(0..parts[1].len()).step_by(2).map(|at|u8::from_str_radix(&parts[1][at..at+2],16).unwrap()).collect();
    let cursor_payload:serde_json::Value=serde_json::from_slice(&cursor_bytes).unwrap();
    let encode=|payload:&serde_json::Value|{
        let bytes=serde_json::to_vec(payload).unwrap();
        let hex:String=bytes.iter().map(|byte|format!("{byte:02x}")).collect();
        format!("collection-v1:{hex}:{}",crate::sha(&bytes))
    };
    let mut fresh=CollectionSearchEngine::open_configured(&profiles,None,None).unwrap();
    assert_eq!(exact(&mut fresh,Some(1),Some(saved.clone()),Vec::new()).totals,full.totals);
    assert_eq!(exact(&mut fresh,Some(1),Some(encode(&cursor_payload)),Vec::new()).totals,full.totals);
    println!("PASS unchanged configured cursor accepted by fresh engine");

    CollectionProfiles::configure(&file,CollectionProfileWriteRequest{name:"local".into(),root:root.to_owned(),packs:packs.to_vec(),disabled:packs.to_vec(),make_default:true}).unwrap();
    assert!(matches!(CollectionSearchEngine::open_configured(&profiles,None,None),Err(CollectionQueryError::ConfigChanged)));
    let disabled_profiles=CollectionProfiles::read(&file).unwrap();
    let mut disabled=CollectionSearchEngine::open_configured(&disabled_profiles,None,None).unwrap();
    assert_eq!(disabled.selection().collections.iter().filter(|row|row.default_member).count(),4);
    assert!(disabled.selection().collections.iter().filter(|row|row.default_member).all(|row|row.disabled));
    assert!(matches!(disabled.execute(request(Some(1),Some(saved.clone()),Vec::new())),Err(CollectionQueryError::StaleCursor)));
    assert!(matches!(disabled.execute(request(Some(1),None,Vec::new())),Err(CollectionQueryError::NoEnabledPacks)));
    assert!(CollectionSearchEngine::open_configured(&disabled_profiles,None,Some(&[packs[0]])).is_err());
    // An already captured engine is allowed to finish its original snapshot.
    assert_eq!(exact(&mut old,Some(1),Some(saved),Vec::new()).totals,full.totals);
    println!("PASS disabled defaults report, explicit override rejects, stale wins before unavailable, old snapshot finishes");

    for cursor in [format!("collection-v1:aéx:{}","0".repeat(64)),"not-a-managed-cursor".into()] {
        assert!(matches!(disabled.execute(request(Some(1),Some(cursor),Vec::new())),Err(CollectionQueryError::MalformedCursor)));
    }
    for (key,value) in [("version",serde_json::json!(2)),("snapshot_binding",serde_json::json!("A".repeat(64))),("inner",serde_json::json!("")),("unexpected",serde_json::json!(true))] {
        let mut payload=cursor_payload.clone();payload[key]=value;
        assert!(matches!(disabled.execute(request(Some(1),Some(encode(&payload)),Vec::new())),Err(CollectionQueryError::MalformedCursor)),"{key}");
    }
    let mut stale=cursor_payload.clone();stale["snapshot_binding"]=serde_json::json!("0".repeat(64));
    assert!(matches!(disabled.execute(request(Some(1),Some(encode(&stale)),Vec::new())),Err(CollectionQueryError::StaleCursor)));
    let old_page=legacy.exact("farvi",DictionaryExactOptions{page_size:Some(1),..Default::default()}).unwrap();
    let legacy_cursor=old_page.next_cursor.unwrap();
    assert!(matches!(disabled.execute(request(Some(1),Some(legacy_cursor.clone()),Vec::new())),Err(CollectionQueryError::MalformedCursor)));
    assert_eq!(legacy.exact("farvi",DictionaryExactOptions{page_size:Some(1),cursor:Some(legacy_cursor),..Default::default()}).unwrap().totals,full.totals);
    println!("PASS non-ASCII and wrong-magic cursors are typed malformed before no-enabled admission");
    println!("PASS independent wrapper positive, version/binding/inner/extra-key negatives, typed stale and unchanged legacy cursor route");
    CollectionProfiles::configure(&file,CollectionProfileWriteRequest{name:"local".into(),root:root.to_owned(),packs:packs.to_vec(),disabled:Vec::new(),make_default:true}).unwrap();

    let current_profiles=CollectionProfiles::read(&file).unwrap();
    let mut before=CollectionSearchEngine::open_configured(&current_profiles,None,None).unwrap();
    let binding=before.selection().binding_sha256.clone();
    let first=exact(&mut before,Some(1),None,Vec::new());let cookie=first.next_cursor.unwrap();
    let catalogue=CollectionCatalogue::open(root).unwrap();
    let locations=catalogue.installed_artifact_locations(PackId::LensiskuJbotci).unwrap().unwrap();drop(catalogue);
    let release_bytes=std::fs::read(&locations.release_path).unwrap();
    let held_release=locations.release_path.with_extension("cdc-held");assert!(!held_release.exists());
    std::fs::rename(&locations.release_path,&held_release).unwrap();
    let mut copy_missing=CollectionSearchEngine::open_configured(&current_profiles,None,None).unwrap();
    assert_eq!(copy_missing.selection().binding_sha256,binding);
    let status=copy_missing.selection().collections.iter().find(|row|row.pack_id=="lensisku-jbotci").unwrap();
    assert_eq!(status.owned_files_present,Some(false));assert!(status.owned_files_warning.is_some());
    assert_eq!(exact(&mut copy_missing,Some(1),Some(cookie.clone()),Vec::new()).totals,full.totals);
    std::fs::rename(&held_release,&locations.release_path).unwrap();
    assert_eq!(std::fs::read(&locations.release_path).unwrap(),release_bytes);
    println!("PASS missing owned copy warns without changing binding or invalidating continuation");

    let held_store=locations.store_path.with_file_name("store-cdc-held");assert!(!held_store.exists());
    std::fs::rename(&locations.store_path,&held_store).unwrap();
    let mut missing=CollectionSearchEngine::open_configured(&current_profiles,None,None).unwrap();
    assert!(matches!(missing.execute(request(Some(1),Some(cookie.clone()),Vec::new())),Err(CollectionQueryError::StaleCursor)));
    let minimum=vlajmaji_store::DictionaryStore::open(&locations.store_path).unwrap_err();
    let class=crate::reference_checks::store_error_class(&minimum);
    let fresh_error=missing.execute(request(Some(1),None,Vec::new())).unwrap_err();
    assert!(crate::reference_checks::retains_store_cause(&fresh_error,&class),"missing derived lost typed {class}: {fresh_error:?}");
    assert!(matches!(missing.execute(DictionaryRequest::Resolve{target:vlajmaji_lexicon::RelationTarget::Unknown}).unwrap().result,CollectionQueryResult::Resolve(_)));
    std::fs::rename(&held_store,&locations.store_path).unwrap();
    let mut restored=CollectionSearchEngine::open_configured(&current_profiles,None,None).unwrap();
    assert_eq!(restored.selection().binding_sha256,binding);
    assert_eq!(exact(&mut restored,Some(1),Some(cookie),Vec::new()).totals,full.totals);
    println!("PASS missing derived store stales before payload error and exact restoration accepts old cursor");
}
