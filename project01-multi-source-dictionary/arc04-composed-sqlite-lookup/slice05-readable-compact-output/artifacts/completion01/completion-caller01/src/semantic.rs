// Prospective22nd-maximum semantic process. No Full run before exact root preflight.
fn detail_key(hit:&DictionaryHit)->DictionaryHitDetailRef {
 DictionaryHitDetailRef{version:1,digest_policy:"entry-assertion-serde-json-v1".into(),role:"assertion".into(),source:hit.entry.source.clone(),entry_id:hit.entry.id.clone(),assertion_id:hit.assertion.id.clone(),occurrence:hit.assertion.occurrence.clone(),entry_sha256:sha(&serde_json::to_vec(&hit.entry).unwrap()),assertion_sha256:sha(&serde_json::to_vec(&hit.assertion).unwrap()),native_entry_id:hit.occurrence_attribution.as_ref().and_then(|r|r.native_entry_id.clone()),native_assertion_id:hit.occurrence_attribution.as_ref().and_then(|r|r.native_assertion_id.clone()),contributor_id:hit.occurrence_attribution.as_ref().and_then(|r|r.contributor.clone()),distributor_id:hit.snapshot_attribution.distributor.clone()}
}
fn expected_compact_hit(known:&DictionaryHit)->DictionaryCompactHit {
 DictionaryCompactHit{occurrence_id:known.occurrence_id.clone(),entry:known.entry.clone(),assertion:known.assertion.clone(),capture:known.capture.clone(),snapshot_attribution:known.snapshot_attribution.clone(),occurrence_attribution:known.occurrence_attribution.clone(),contributor:known.contributor.clone(),distributor:known.distributor.clone(),release_ids:known.release_ids.clone(),detail_ref:detail_key(known)}
}
fn qualify_semantic_method(context:&Path){
 assert!(context.is_absolute()&&context.starts_with("/tmp/vlajmaji-p01-a04-s05-cdc")&&!context.components().any(|p|p==std::path::Component::ParentDir));
 let ctx:Value=serde_json::from_slice(&std::fs::read(context).unwrap()).unwrap();assert_eq!(ctx["fixture_scope"],"original-four-pack-synthetic-readable");
 let hit:DictionaryHit=serde_json::from_str(ctx["known_llg_hit_pretty"].as_str().unwrap()).unwrap();let want=expected_compact_hit(&hit);
 assert_eq!(want.entry,hit.entry);assert_eq!(want.capture,hit.capture);assert_eq!(want.detail_ref,detail_key(&hit));
 let mut changed=want.clone();changed.entry.spelling=vlajmaji_lexicon::Presence::Null;assert_ne!(changed,want);
 let mut changed=want.clone();changed.assertion.definition=vlajmaji_lexicon::Presence::Missing;assert_ne!(changed,want);
 let mut changed=want.clone();changed.detail_ref.native_assertion_id=Some("wrong-native-role".into());assert_ne!(changed,want);
 let mut changed=want.clone();changed.detail_ref.entry_sha256="0".repeat(64);assert_ne!(changed,want);
 let mut changed=want.clone();changed.release_ids.clear();assert_ne!(changed,want);
 let mut changed=want.clone();changed.occurrence_id.push('0');assert_ne!(changed,want);
 emit(&json!({"row_type":"method_qualified","ok":true,"scope":"pure SAME expected_compact_hit complete typed comparator; Tiny saved direct-typed known hit only, no engine/model/DB/query/Full data","positive":1,"negative":6}));
}
fn semantic(config:&Path,workload:&[u8],out:&Path,tiny:bool){
 let profiles=CollectionProfiles::read(config).unwrap();let policy=profiles.profile(None).unwrap();
 assert_eq!(policy.root,if tiny{config.parent().unwrap().join("root")}else{PathBuf::from(ROOT)});
 let query=||DictionaryRequest::Exact{spelling:"farvi".into(),options:Default::default()};
 let (expected,order,totals)={
  let mut full=CollectionSearchEngine::open_configured(&profiles,None,None).unwrap();
  if !tiny{assert_eq!(full.selection().local_revision,4);assert_eq!(full.selection().profile.as_ref().unwrap().revision,5);}
  let response=full.execute(query()).unwrap();
  let CollectionQueryResult::Exact(inner)=response.result else{panic!("full exact")};
  assert_eq!(inner.totals.occurrences,5);assert_eq!(inner.totals.source_groups,3);
  let bytes=json_wire(&inner);
  if !tiny {assert_eq!(bytes.len(),7160366);assert_eq!(sha(&bytes),"144b8b09b6e378243b36c8e01ab06353c3d573459668c8c158a86aebbea73556");}
  drop(bytes);
  let order:Vec<_>=inner.groups.iter().flat_map(|g|g.hits.iter().map(|h|h.assertion.id.to_string())).collect();
  let expected:BTreeMap<_,_>=inner.groups.iter().flat_map(|g|g.hits.iter().map(|h|(h.assertion.id.to_string(),h.clone()))).collect();assert_eq!(expected.len(),5);
  let totals=inner.totals.clone();
  drop(inner);drop(full); // Full availability/engine/model gone BEFORE compact setup.
  (expected,order,totals)
 };
 let keys:BTreeMap<_,_>=expected.iter().map(|(id,h)|(id.clone(),detail_key(h))).collect();
 {
  let mut compact=CollectionCompactSearchEngine::open_configured(&profiles,None,None).unwrap();
  let all=compact.execute(query()).unwrap();let CollectionCompactQueryResult::Exact(inner)=&all.result else{panic!("compact exact")};
  assert_eq!(inner.totals,totals);let ids:Vec<_>=inner.groups.iter().flat_map(|g|g.hits.iter().map(|h|h.assertion.id.to_string())).collect();assert_eq!(ids,order);
  for g in &inner.groups {for h in &g.hits{let want=&expected[h.assertion.id.as_str()];assert_eq!(h,&expected_compact_hit(want));}}
  let readable=render_collection_compact(&all);assert!(readable.ends_with('\n')&&!readable.ends_with("\n\n"));assert!(readable.len()<=131072);
  let compact_wire=json_wire(all);assert!(compact_wire.len()<=65536);drop(compact_wire);drop(readable);
  for (alias,source) in [("LSK","lensisku"),("LLG","llg"),("SUT","sutysisku")] {
    let response=compact.execute(DictionaryRequest::Exact{spelling:"farvi".into(),options:DictionaryExactOptions{sources:vec![alias.into()],..Default::default()}}).unwrap();
    let CollectionCompactQueryResult::Exact(projected)=&response.result else{panic!("source override exact")};
    let wanted:Vec<_>=order.iter().filter(|id|expected[*id].entry.source.as_str()==source).cloned().collect();
    let ids:Vec<_>=projected.groups.iter().flat_map(|g|g.hits.iter().map(|h|h.assertion.id.to_string())).collect();assert_eq!(ids,wanted);assert_eq!(projected.totals.occurrences,wanted.len());assert_eq!(projected.totals.source_groups,1);
    for h in projected.groups.iter().flat_map(|g|&g.hits){let known=&expected[h.assertion.id.as_str()];assert_eq!(h,&expected_compact_hit(known));}
    let bytes=render_collection_compact(&response);assert!(bytes.ends_with('\n')&&!bytes.ends_with("\n\n"));drop(bytes);drop(response);
  }
  for size in [0,1,2,3] {
    let mut observed=Vec::new();let mut cursor=None;let mut completed=false;
    for _ in 0..6 {let response=compact.execute(DictionaryRequest::Exact{spelling:"farvi".into(),options:DictionaryExactOptions{page_size:Some(size),cursor,..Default::default()}}).unwrap();let CollectionCompactQueryResult::Exact(p)=response.result else{panic!("exact page")};assert_eq!(p.totals,totals);observed.extend(p.groups.iter().flat_map(|g|g.hits.iter().map(|h|h.assertion.id.to_string())));cursor=p.next_cursor;if cursor.is_none(){completed=true;break}}
    assert!(completed);if size==0{assert!(observed.is_empty())}else{assert_eq!(observed,order)}
  }
  for (id,k) in &keys {let detail=compact.detail(k,None).unwrap();assert_eq!(detail.response.status,ReferenceStatus::Resolved);assert_eq!(detail.response.hit.as_ref(),Some(&expected[id]));assert_eq!(detail.response.current_eligible,Some(true));
    let bytes=json_wire(detail);let mut f=OpenOptions::new().create_new(true).write(true).open(out.join(format!("detail-{}.json",keys.keys().position(|x|x==id).unwrap()))).unwrap();f.write_all(&bytes).unwrap();f.sync_all().unwrap();drop(bytes);
  }
  let rows:Vec<Work>=serde_json::from_slice(workload).unwrap();let mut count=0;
  for w in rows.iter().filter(|w|!w.prime&&["jbotci-local","jbotci-catalogue-native333","la-catalogue-native333","llg-component","sut-native-entry"].contains(&w.id.as_str())) {
    let target=w.target.clone().unwrap();let response=compact.execute(DictionaryRequest::Resolve{target:target.clone()}).unwrap();let CollectionCompactQueryResult::Resolve(r)=response.result else{panic!("reference")};assert_eq!(r.target,target);assert_eq!(r.status,ReferenceStatus::Resolved);let payload=r.payload.unwrap();assert_eq!(payload.target,target);assert_eq!(payload.confirmed.len(),1);assert!(payload.unverified.is_empty());let v=&payload.confirmed[0];
    match &target {
      vlajmaji_lexicon::RelationTarget::Local(vlajmaji_lexicon::LocalRef::Assertion(id))=>{assert_eq!(&v.assertion.id,id);assert_eq!(v.assertion,expected[id.as_str()].assertion);},
      vlajmaji_lexicon::RelationTarget::Catalog(c)=>{assert_eq!(v.entry.source,c.source);assert_eq!(Some(&v.assertion.occurrence),c.occurrence.as_ref());assert_eq!(v.release_ids,vec![c.release.clone()]);let want=expected.values().find(|h|h.assertion.occurrence==v.assertion.occurrence).unwrap();assert_eq!(v.assertion,want.assertion);assert_eq!(v.detail_ref.as_ref(),Some(&keys[want.assertion.id.as_str()]));let native=v.occurrence_attribution.as_ref().unwrap();if let Some(n)=&c.native_entry_id{assert_eq!(native.native_entry_id.as_ref(),Some(n))}if let Some(n)=&c.native_assertion_id{assert_eq!(native.native_assertion_id.as_ref(),Some(n))}},
      _=>panic!("frozen five reference role changed")
    }
    count+=1;
  }
  assert_eq!(count,5);
  let wrong=rows.iter().find(|w|!w.prime&&w.id=="wrong-role").unwrap();let error=compact.execute(DictionaryRequest::Resolve{target:wrong.target.clone().unwrap()}).err().expect("actual wrong-role error");assert_eq!(error.report_for("lookup").category,"invalid_reference");
  drop(compact);
 }
 drop(profiles);
 emit(&json!({"row_type":"semantic_complete","ok":true,"tiny":tiny,"accepted_full_inner_sha_checked":!tiny,"projected_occurrences":5,"source_groups":3,"complete_details":5,"stable_references":5,"source_overrides":["LSK","LLG","SUT"],"wrong_role":"invalid_reference","page_sizes":[0,1,2,3],"scope":"one full engine then dropped; one compact engine; no creation/All/import/raw-model reconstruction"}));
}
