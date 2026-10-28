// Prospective outside public proof for S05 readable03. SOURCE ONLY until clean handback.
// Seven whole expected strings are byte-preserved from root-reviewed plan §14.2.
// These are presentation-data inputs, not claimed validated dictionaries/body digests.
fn readable_public_checks(root: &Path, store: &Path, docs: &[UnverifiedDictionaryDocument], sealed: &[(PackId, Vec<u8>, Vec<u8>)]) {
    use vlajmaji_search::{render_dictionary_compact, render_collection_compact, render_dictionary_detail, render_collection_detail};
    let _: fn(&DictionaryCompactQueryResult) -> String = render_dictionary_compact;
    let _: fn(&CollectionCompactQueryResponse) -> String = render_collection_compact;
    let _: fn(&DictionaryHitDetailResponse) -> String = render_dictionary_detail;
    let _: fn(&CollectionHitDetailResponse) -> String = render_collection_detail;
    let key: DictionaryHitDetailRef = serde_json::from_str(include_str!("../fixtures/readable-key.json")).unwrap();
    let hit: DictionaryHit = serde_json::from_str(include_str!("../fixtures/readable-hit.json")).unwrap();
    let policy: CollectionSelectionSnapshot = serde_json::from_str(include_str!("../fixtures/readable-snapshot.json")).unwrap();
    let engine = DictionaryCompactSearchEngine::open(store).unwrap();
    let mut exact = engine.exact("farvi", DictionaryExactOptions::default()).unwrap();
    assert_eq!(exact.totals.occurrences, 5);
    exact.totals.occurrences = 0; exact.totals.distinct_spellings = 0; exact.totals.source_groups = 0;
    exact.groups.clear(); exact.next_cursor = None;
    assert_eq!(render_dictionary_compact(&DictionaryCompactQueryResult::Exact(Box::new(exact))), include_str!("../fixtures/readable-golden-0.txt"));
    let mut exact = engine.exact("farvi", DictionaryExactOptions::default()).unwrap();
    let mut group = exact.groups[0].clone();
    group.source.id = SourceId::new("source-a").unwrap(); group.source.abbreviation = "A".into();
    group.full_count = 4; group.returned_count = 3; group.partial = true;
    let mut row = group.hits[0].clone(); row.entry = hit.entry.clone(); row.assertion = hit.assertion.clone();
    row.occurrence_id = "00".into(); row.detail_ref = key.clone();
    group.hits = [
        (Presence::Missing, Presence::Missing),
        (Presence::Null, Presence::Null),
        (Presence::Value("farvi".to_string()), hit.assertion.definition.clone()),
    ].into_iter().map(|(spelling, definition)| {let mut r=row.clone();r.entry.spelling=spelling;r.assertion.definition=definition;r}).collect();
    exact.groups = vec![group]; exact.totals.occurrences=4; exact.totals.distinct_spellings=1; exact.totals.source_groups=1; exact.next_cursor=Some("next-page".into());
    let exact = DictionaryCompactQueryResult::Exact(Box::new(exact));
    assert_eq!(render_dictionary_compact(&exact), include_str!("../fixtures/readable-golden-1.txt"));
    assert_eq!(render_dictionary_compact(&exact), render_dictionary_compact(&exact));
    println!("PASS independent literal Exact empty/nonempty/threePresence/duplicate/partial/cursor");

    // Caller-constructible string atoms; typed IDs remain in their actual ASCII grammar.
    let mut escaped=engine.exact("farvi",DictionaryExactOptions::default()).unwrap();
    let first_input=&docs[0].metadata.assertions.iter().find(|a|matches!(a.eligibility,Eligibility::Lexical)).unwrap().id;
    escaped.groups[0].source.abbreviation="A\u{1b}\r\t\\\"\nβ".into();
    escaped.groups[0].hits[0].occurrence_id="A\u{1b}\r\t\\\"\nβ".into();
    let escaped=render_dictionary_compact(&DictionaryCompactQueryResult::Exact(Box::new(escaped)));
    let lines:Vec<_>=escaped.lines().collect();
    assert_eq!(lines[1],r#"A\u001b\r\t\\\"\nβ [lensisku] 2/2"#);
    assert_eq!(lines[2],format!(r#"  {} @ A\u001b\r\t\\\"\nβ"#,first_input));
    assert!(!escaped.contains('\u{1b}')&&!escaped.contains('\r')&&!escaped.contains('\t'));
    println!("PASS independent exact literal header ESC/CR/tab/backslash/quote/LF/Unicode atom escaping");

    let doc=&docs[1]; // Original input order JB, LLG, La, Sut.
    let a=doc.metadata.assertions.iter().find(|a|matches!(a.eligibility,Eligibility::Lexical)).unwrap();
    let actual_key=key_for_readable(doc,a);
    let target=RelationTarget::Local(LocalRef::Assertion(a.id.clone()));
    let mut resolved=engine.resolve(&target).unwrap();
    assert_eq!(resolved.status,ReferenceStatus::Resolved);assert_eq!(resolved.confirmed.len(),1);assert!(resolved.unverified.is_empty());
    let mut view=resolved.confirmed[0].clone();view.entry=hit.entry.clone();view.assertion=hit.assertion.clone();view.occurrence_id="00".into();view.payload_present=true;view.current_eligible=true;view.detail_ref=Some(key.clone());
    let mut unknown=view.clone();unknown.entry.source=SourceId::new("source-z").unwrap();unknown.entry.spelling=Presence::Null;unknown.assertion.id=EntityId::new("assertion-z").unwrap();unknown.assertion.definition=Presence::Missing;unknown.assertion.eligibility=Eligibility::Excluded{reason:"not lexical".into()};unknown.occurrence_id="01".into();unknown.payload_present=false;unknown.current_eligible=false;unknown.detail_ref=None;
    let mut av=resolved.availability[0].clone();av.source.id=SourceId::new("source-a").unwrap();av.source.abbreviation="A".into();let mut later=av.clone();later.source.abbreviation="wrong".into();
    resolved.target=RelationTarget::Local(LocalRef::Assertion(EntityId::new("assertion-a").unwrap()));resolved.status=ReferenceStatus::Unverified;resolved.entity=None;resolved.confirmed=vec![view];resolved.unverified=vec![unknown];resolved.availability=vec![av,later];resolved.requested_release=None;
    assert_eq!(render_dictionary_compact(&DictionaryCompactQueryResult::Resolve(Box::new(resolved.clone()))),include_str!("../fixtures/readable-golden-2.txt"));
    let mut unknown=resolved;unknown.target=RelationTarget::Unknown;unknown.status=ReferenceStatus::Unknown;unknown.confirmed.clear();unknown.unverified.clear();
    assert_eq!(render_dictionary_compact(&DictionaryCompactQueryResult::Resolve(Box::new(unknown.clone()))),include_str!("../fixtures/readable-golden-3.txt"));
    let mut unresolved=unknown.clone();unresolved.target=RelationTarget::Unresolved("unknown\nβ".into());unresolved.status=ReferenceStatus::Unresolved;
    assert_eq!(render_dictionary_compact(&DictionaryCompactQueryResult::Resolve(Box::new(unresolved))),"Reference {\"Unresolved\":\"unknown\\nβ\"}: Unresolved\nentity: null\nconfirmed: 0\nunverified: 0\nrequested_release: null\n");
    let mut no_match=unknown;no_match.status=ReferenceStatus::NoMatch;
    assert_eq!(render_dictionary_compact(&DictionaryCompactQueryResult::Resolve(Box::new(no_match))),"Reference \"Unknown\": NoMatch\nentity: null\nconfirmed: 0\nunverified: 0\nrequested_release: null\n");
    let mut managed=selected(root).execute(DictionaryRequest::Resolve{target:target.clone()}).unwrap();
    managed.snapshot=policy.clone();managed.collections.clear();managed.result=CollectionCompactQueryResult::Resolve(Box::new(CollectionCompactReferenceResponse{target:RelationTarget::Local(LocalRef::Assertion(key.assertion_id.clone())),status:ReferenceStatus::CatalogueOnly,operational_reason:Some("removed".into()),payload:None,catalogue_matches:vec![],requested_release:None}));
    assert_eq!(render_collection_compact(&managed),include_str!("../fixtures/readable-golden-4.txt"));
    println!("PASS independent literal Resolve confirmed/unverified/excluded/fallback/firstlabel/Unknown and managedCatalogueOnly");

    let detail=DictionaryHitDetailResponse{version:1,projection_policy:"complete-dictionary-hit-v1".into(),detail_ref:key.clone(),status:ReferenceStatus::NoMatch,operational_reason:Some("lexical_identity_changed".into()),state_sha256:None,snapshot_sha256:"3".repeat(64),hit:None,current_eligible:None};
    assert_eq!(render_dictionary_detail(&detail),include_str!("../fixtures/readable-golden-5.txt"));
    let mut complete=detail.clone();complete.status=ReferenceStatus::Resolved;complete.operational_reason=None;complete.current_eligible=Some(true);complete.hit=Some(hit);
    assert_eq!(render_dictionary_detail(&complete),include_str!("../fixtures/readable-golden-6.txt"));
    let md=CollectionHitDetailResponse{version:1,snapshot:policy,collections:vec![],response:detail,catalogue_matches:vec![]};
    let plain=include_str!("../fixtures/readable-golden-5.txt");let prefix=plain.strip_suffix("hit: null\n").unwrap();
    let snapshot=include_str!("../fixtures/readable-snapshot.json").trim_end_matches('\n');
    let want=format!("{prefix}selection: {snapshot}\ncollections: []\ncatalogue_matches: []\nhit: null\n");
    assert_eq!(render_collection_detail(&md),want);
    println!("PASS independent literal complete21field typedPrettyHit/RawJson and null/managed detail");

    // Actual query output uses the same installed original input, not another store.
    let live=engine.detail(&actual_key,None).unwrap();
    assert_eq!(live.status,ReferenceStatus::Resolved);assert_eq!(live.hit.as_ref().unwrap().assertion,*a);
    let text=render_dictionary_detail(&live);assert!(text.ends_with('\n')&&!text.ends_with("\n\n"));assert!(text.ends_with(&format!("hit: {}\n",serde_json::to_string_pretty(live.hit.as_ref().unwrap()).unwrap())));
    drop(engine);
    write_readable_journey_inputs(root.parent().unwrap(),store,docs,sealed);
}
fn key_for_readable(d:&UnverifiedDictionaryDocument,a:&AssertionRecord)->DictionaryHitDetailRef {key(d,a)}
fn write_readable_journey_inputs(base:&Path, store:&Path, docs:&[UnverifiedDictionaryDocument], sealed:&[(PackId,Vec<u8>,Vec<u8>)]) {
    let dir=base.join("readable-inputs");std::fs::create_dir(&dir).unwrap();
    let mut inputs=Vec::new();
    for (pack,release,seal) in sealed {let p=dir.join(pack.as_str());std::fs::create_dir(&p).unwrap();let rp=p.join("release.json");let sp=p.join("seal.json");
        std::fs::OpenOptions::new().create_new(true).write(true).open(&rp).and_then(|mut f|std::io::Write::write_all(&mut f,release)).unwrap();
        std::fs::OpenOptions::new().create_new(true).write(true).open(&sp).and_then(|mut f|std::io::Write::write_all(&mut f,seal)).unwrap();
        inputs.push(serde_json::json!({"pack_id":pack.as_str(),"release":rp,"seal":sp,"release_sha256":sha(release),"seal_sha256":sha(seal)}));}
    let d=&docs[1];let a=d.metadata.assertions.iter().find(|a|matches!(a.eligibility,Eligibility::Lexical)).unwrap();let k=key(d,a);let r=d.metadata.releases.iter().find(|r|r.assertions.contains(&a.id)).unwrap();
    let local=DictionaryRequest::Resolve{target:RelationTarget::Local(LocalRef::Assertion(a.id.clone()))};
    let cat=DictionaryRequest::Resolve{target:RelationTarget::Catalog(CatalogTarget{source:k.source.clone(),release:r.id.clone(),native_entry_id:k.native_entry_id.clone(),native_assertion_id:k.native_assertion_id.clone(),occurrence:Some(a.occurrence.clone())})};
    let mut order:Vec<_>=docs.iter().flat_map(|d|d.metadata.assertions.iter().filter(|a|matches!(a.eligibility,Eligibility::Lexical)).map(move|a|{let e=d.metadata.entries.iter().find(|e|e.id==a.entry).unwrap();(e.source.to_string(),match &e.spelling{Presence::Value(s)=>s.clone(),_=>String::new()},tuple_id(&a.occurrence),a.id.to_string())})).collect();order.sort();
    let expected_keys:BTreeMap<_,_>=docs.iter().flat_map(|d|d.metadata.assertions.iter().filter(|a|matches!(a.eligibility,Eligibility::Lexical)).map(move|a|(a.id.to_string(),key(d,a)))).collect();
    let expected_assertions:BTreeMap<_,_>=docs.iter().flat_map(|d|d.metadata.assertions.iter().filter(|a|matches!(a.eligibility,Eligibility::Lexical)).map(|a|(a.id.to_string(),a))).collect();
    let expected_entries:BTreeMap<_,_>=docs.iter().flat_map(|d|d.metadata.entries.iter().map(|e|(e.id.to_string(),e))).collect();
    let sources:BTreeMap<_,_>=docs.iter().flat_map(|d|d.registry.sources.iter().map(|s|(s.id.to_string(),s))).collect();
    let known_hit_pretty={let full=DictionarySearchEngine::open(store).unwrap();let response=full.exact("farvi",Default::default()).unwrap();let known=response.groups.iter().flat_map(|g|&g.hits).find(|h|h.assertion.id==a.id).unwrap();assert_eq!(&known.assertion,a);assert_eq!(known.entry.id,k.entry_id);serde_json::to_string_pretty(known).unwrap()};
    let entity=serde_json::json!({"role":"assertion","id":a.id,"entry_ids":[a.entry],"assertion_ids":[a.id],"source_ids":[k.source],"occurrence":a.occurrence,"evidence":null,"spelling":null});
    let summary=vlajmaji_store::CollectionReleaseSummary::from(r);
    let release_digest=sealed.iter().find(|p|p.0==PackId::Llg).map(|p|sha(&p.1)).unwrap();
    let local_match=serde_json::json!({"pack_id":"llg","release_sha256":release_digest,"source_id":k.source,"model_release_id":"","role":"assertion","entity_id":a.id,"occurrence":a.occurrence.locator,"capture":a.occurrence.capture,"native_entry_id":k.native_entry_id,"native_assertion_id":k.native_assertion_id,"contributing_pack_ids":["llg"]});
    let mut catalog_match=local_match.clone();catalog_match["model_release_id"]=serde_json::to_value(&r.id).unwrap();
    let context=serde_json::json!({"fixture_scope":"original-four-pack-synthetic-readable","known_llg_hit_pretty":known_hit_pretty,"expected_local_entity":entity,"expected_requested_release":summary,"expected_local_matches":[local_match],"expected_catalog_matches":[catalog_match],"ordered_lexical_assertion_ids":order.into_iter().map(|x|x.3).collect::<Vec<_>>(),"expected_keys":expected_keys,"expected_assertions":expected_assertions,"expected_entries":expected_entries,"sources":sources,"inputs":inputs,"local_request":local,"catalog_request":cat,"llg_key":k,"lexical_assertion_ids":docs.iter().flat_map(|d|d.metadata.assertions.iter().filter(|a|matches!(a.eligibility,Eligibility::Lexical)).map(|a|a.id.to_string())).collect::<Vec<_>>()});
    let f=std::fs::OpenOptions::new().write(true).create_new(true).open(base.join("readable-context.json")).unwrap();serde_json::to_writer(f,&context).unwrap();
    println!("PASS source-derived sealed Tiny inputs/context prepared for actual CLI install journey");
}
