// Input-derived target witnesses are frozen before registration/installation.
use std::{borrow::Cow, path::Path};
use vlajmaji_lexicon::*;
use vlajmaji_store::{CollectionCatalogue, CollectionInput, CollectionRequest, CollectionSelection};
use vlajmaji_search::{CollectionSearchEngine, CollectionQuerySelection, CollectionQueryResult, CollectionReferenceResponse, CollectionQueryError, DictionaryQueryError, DictionaryRequest, ReferenceStatus};

#[derive(Clone,serde::Serialize)]
pub struct Witness {pub pack:PackId,pub target:RelationTarget,pub assertion:AssertionRecord,pub attribution:Option<OccurrenceAttribution>}
pub fn freeze(models:&[Dictionary],packs:&[PackId])->Vec<Witness> {
    models.iter().zip(packs).flat_map(|(dictionary,pack)|dictionary.metadata().assertions.iter().map(move|assertion|{
        let entry=dictionary.metadata().entries.iter().find(|entry|entry.id==assertion.entry).unwrap();
        let release=dictionary.metadata().releases.iter().find(|release|release.assertions.contains(&assertion.id)).unwrap();
        let attribution=dictionary.registry().as_document().occurrences.iter().find(|row|row.occurrence==assertion.occurrence.to_record_identity().unwrap()).cloned();
        Witness{pack:*pack,target:RelationTarget::Catalog(CatalogTarget{source:entry.source.clone(),release:release.id.clone(),native_entry_id:attribution.as_ref().and_then(|row|row.native_entry_id.clone()),native_assertion_id:attribution.as_ref().and_then(|row|row.native_assertion_id.clone()),occurrence:Some(assertion.occurrence.clone())}),assertion:assertion.clone(),attribution}
    })).collect()
}
fn engine(root:&Path,packs:&[PackId],disabled:&[PackId])->CollectionSearchEngine {
    CollectionSearchEngine::open(root,CollectionQuerySelection{packs:packs.to_vec(),disabled:disabled.to_vec(),profile:None}).unwrap()
}
fn resolve(engine:&mut CollectionSearchEngine,target:RelationTarget)->CollectionReferenceResponse {
    match engine.execute(DictionaryRequest::Resolve{target}).unwrap().result {CollectionQueryResult::Resolve(response)=>*response,_=>panic!("resolve returned another operation")}
}
fn invalid(engine:&mut CollectionSearchEngine,target:RelationTarget) {
    assert!(matches!(engine.execute(DictionaryRequest::Resolve{target}),Err(CollectionQueryError::Query(DictionaryQueryError::InvalidReference{..}))));
}
fn request<'a>(packs:&[PackId],pairs:&'a[(Vec<u8>,Vec<u8>)],indices:&[usize])->CollectionRequest<'a> {
    CollectionRequest::new(indices.iter().map(|&index|CollectionInput::SealedBytes{pack_id:packs[index],release:Cow::Borrowed(&pairs[index].0),seal:Cow::Borrowed(&pairs[index].1)}).collect(),CollectionSelection::Packs(indices.iter().map(|&index|packs[index]).collect()),true).unwrap()
}
pub(crate) fn store_error_class(error:&vlajmaji_store::DictionaryStoreError)->String {
    match error {
        vlajmaji_store::DictionaryStoreError::Io(_)=>"io".into(),
        vlajmaji_store::DictionaryStoreError::Backend(_)=>"backend".into(),
        vlajmaji_store::DictionaryStoreError::InvalidStore{reason,..}=>format!("invalid:{reason:?}"),
        other=>panic!("unexpected minimum failure: {other}"),
    }
}
pub(crate) fn retains_store_cause(error:&(dyn std::error::Error+'static),class:&str)->bool {
    let mut current=error;
    loop {
        if let Some(store)=current.downcast_ref::<vlajmaji_store::DictionaryStoreError>() {
            return store_error_class(store)==class;
        }
        match current.source(){Some(next)=>current=next,None=>return false}
    }
}
fn revision_model(digit:char)->Dictionary {
    let meta=format!("language_tag = \"en\"\nlanguage_realname = \"English\"\nsource_language_tag = \"jbo\"\nformat = \"json\"\npositive_scores_only = false\nfilename = \"dictionary-en.json\"\nmetadata_url = \"https://example.invalid/languages\"\ndownload_url = \"https://example.invalid/synthetic\"\nlensisku_created_at = \"2026-10-04T03:07:08Z\"\nsha256 = \"{}\"\ndefinition_count = 1\nentry_count = 1\n",crate::sha(crate::LENS));
    import_jbotci_export(crate::LENS,meta.as_bytes(),&JbotciImportOptions{revision:digit.to_string().repeat(40),expected_raw_sha256:crate::sha(crate::LENS),expected_metadata_sha256:crate::sha(meta.as_bytes())}).unwrap().into_dictionary()
}
pub fn run(base:&Path,root:&Path,packs:&[PackId],pairs:&[(Vec<u8>,Vec<u8>)],witnesses:&[Witness]) {
    assert_eq!(witnesses.len(),6);
    let mut all=engine(root,packs,&[]);
    for witness in witnesses {
        let response=resolve(&mut all,witness.target.clone());
        assert_eq!(response.target,witness.target);assert_eq!(response.status,ReferenceStatus::Resolved);
        let payload=response.payload.unwrap();assert_eq!(payload.confirmed.len(),1);assert!(payload.unverified.is_empty());
        assert_eq!(payload.confirmed[0].assertion,witness.assertion);
        assert_eq!(payload.confirmed[0].occurrence_attribution,witness.attribution);
        assert_eq!(payload.confirmed[0].current_eligible,witness.assertion.eligibility==Eligibility::Lexical);
        let RelationTarget::Catalog(target)=&witness.target else{unreachable!()};
        assert!(payload.confirmed[0].release_ids.contains(&target.release));
    }
    let jb=witnesses.iter().find(|row|row.pack==PackId::LensiskuJbotci).unwrap();
    let la=witnesses.iter().find(|row|row.pack==PackId::LaLojban).unwrap();
    assert_ne!(jb.assertion.id,la.assertion.id);
    assert_eq!(jb.attribution.as_ref().unwrap().native_assertion_id.as_deref(),Some("333"));
    assert_eq!(la.attribution.as_ref().unwrap().native_assertion_id.as_deref(),Some("333"));
    drop(all);
    println!("PASS six input-frozen raw references including excluded row and distinct JB/La native333 scopes");

    let registered=base.join("registered-only");
    CollectionCatalogue::register(&registered,request(packs,pairs,&[0,1,2,3])).unwrap();
    let mut metadata=engine(&registered,packs,&[]);
    assert!(metadata.selection().generation_id.is_none());
    for witness in witnesses {
        let response=resolve(&mut metadata,witness.target.clone());
        assert_eq!(response.status,ReferenceStatus::CatalogueOnly);assert!(response.payload.is_none());
        assert_eq!(response.target,witness.target);assert_eq!(response.operational_reason.as_deref(),Some("registered_only"));
        assert!(response.requested_release.is_some());
        let RelationTarget::Catalog(target)=&witness.target else{unreachable!()};
        assert!(response.catalogue_matches.iter().any(|matched|
            matched.pack_id==witness.pack.as_str() && matched.source_id==target.source.as_str()
            && matched.model_release_id==target.release.as_str()
            && matched.role.as_deref()==Some("assertion")
            && matched.entity_id.as_deref()==Some(witness.assertion.id.as_str())
            && matched.occurrence.as_deref()==Some(witness.assertion.occurrence.locator.as_str())));
    }
    println!("PASS registered-only metadata references retain requested releases with no invented payload");

    let RelationTarget::Catalog(jb_target)=&jb.target else{unreachable!()};
    let mut wrong=jb_target.clone();wrong.native_assertion_id=Some("not-333".into());
    assert_eq!(resolve(&mut metadata,RelationTarget::Catalog(wrong)).status,ReferenceStatus::NoMatch);
    let llg=witnesses.iter().find(|row|row.pack==PackId::Llg).unwrap();
    let RelationTarget::Catalog(llg_target)=&llg.target else{unreachable!()};
    assert!(llg.attribution.as_ref().unwrap().native_assertion_id.is_none());
    let mut missing=llg_target.clone();missing.native_assertion_id=Some("unsupplied".into());
    assert_eq!(resolve(&mut metadata,RelationTarget::Catalog(missing)).status,ReferenceStatus::Unverified);
    invalid(&mut metadata,RelationTarget::Local(LocalRef::Entry(jb.assertion.id.clone())));
    assert_eq!(resolve(&mut metadata,RelationTarget::Local(LocalRef::Assertion(jb.assertion.id.clone()))).status,ReferenceStatus::CatalogueOnly);
    let mut wrong=jb_target.clone();wrong.source=SourceId::new("llg").unwrap();invalid(&mut metadata,RelationTarget::Catalog(wrong));
    let mut wrong=jb_target.clone();wrong.release=EntityId::new("unknown-release").unwrap();invalid(&mut metadata,RelationTarget::Catalog(wrong));
    let mut wrong=jb_target.clone();wrong.occurrence.as_mut().unwrap().locator.clear();invalid(&mut metadata,RelationTarget::Catalog(wrong));
    let mut wrong=jb_target.clone();wrong.occurrence=None;wrong.native_entry_id=None;wrong.native_assertion_id=None;invalid(&mut metadata,RelationTarget::Catalog(wrong));
    assert_eq!(resolve(&mut metadata,jb.target.clone()).status,ReferenceStatus::CatalogueOnly);
    println!("PASS native mismatch versus absent slot and exact malformed-role/source/release/locator counterparts");

    for (disabled,selected,reason) in [(vec![jb.pack],packs.to_vec(),"disabled"),(Vec::new(),vec![PackId::Llg],"not_selected")] {
        let response=resolve(&mut engine(root,&selected,&disabled),jb.target.clone());
        assert_eq!(response.status,ReferenceStatus::CatalogueOnly);assert!(response.payload.is_none());
        assert_eq!(response.operational_reason.as_deref(),Some(reason));
    }
    let mut disabled=engine(&registered,packs,packs);
    assert_eq!(resolve(&mut disabled,RelationTarget::Unknown).status,ReferenceStatus::Unknown);
    assert_eq!(resolve(&mut disabled,RelationTarget::Unresolved("future".into())).status,ReferenceStatus::Unresolved);
    invalid(&mut disabled,RelationTarget::Unresolved(String::new()));
    println!("PASS disabled/unselected are policy reasons and Unknown/Unresolved do not require generation");

    // Installed JB/LLG plus registered-only peers: Exact cannot return partial data,
    // while a target-scoped available reference is independent of those peers.
    CollectionCatalogue::install(&registered,request(packs,pairs,&[0,1])).unwrap();
    let mut mixed=engine(&registered,packs,&[]);
    assert!(matches!(mixed.execute(DictionaryRequest::Exact{spelling:"farvi".into(),options:Default::default()}),Err(CollectionQueryError::UnavailablePack{..})));
    assert!(matches!(mixed.execute(DictionaryRequest::Exact{spelling:"farvi".into(),options:vlajmaji_search::DictionaryExactOptions{sources:vec!["LLG".into()],..Default::default()}}),Err(CollectionQueryError::UnavailablePack{..})));
    assert_eq!(resolve(&mut mixed,jb.target.clone()).status,ReferenceStatus::Resolved);
    assert_eq!(resolve(&mut mixed,la.target.clone()).status,ReferenceStatus::CatalogueOnly);
    println!("PASS exact fails aggregate yet usable target resolves despite unrelated registered-only members");

    let history=base.join("reference-history");
    let equal=revision_model('1');
    assert_eq!(equal.metadata().assertions[0],jb.assertion);
    let equal_witness=freeze(std::slice::from_ref(&equal),&[PackId::LensiskuJbotci]);
    assert_ne!(equal_witness[0].target,jb.target);
    let sealed=seal_release(PackId::LensiskuJbotci,PackPayloadKind::Synthetic,&equal).unwrap();
    let equal_pairs=vec![(sealed.release,sealed.seal)];
    let mut document=revision_model('2').into_document();
    document.metadata.assertions[0].native_sense_id=Presence::Value("explicit-synthetic-alternative".into());
    let unequal=Dictionary::from_document(document).unwrap();
    let unequal_witness=freeze(std::slice::from_ref(&unequal),&[PackId::LensiskuJbotci]);
    assert_eq!(unequal.metadata().assertions[0].id,jb.assertion.id);
    assert_ne!(unequal.metadata().assertions[0],jb.assertion);
    let sealed=seal_release(PackId::LensiskuJbotci,PackPayloadKind::Synthetic,&unequal).unwrap();
    let unequal_pairs=vec![(sealed.release,sealed.seal)];
    std::fs::write(base.join("pre-history-witnesses.json"),serde_json::to_vec_pretty(&serde_json::json!({"equal":equal_witness,"unequal":unequal_witness})).unwrap()).unwrap();
    CollectionCatalogue::register(&history,request(packs,pairs,&[0])).unwrap();
    CollectionCatalogue::register(&history,request(&[PackId::LensiskuJbotci],&equal_pairs,&[0])).unwrap();
    assert_eq!(resolve(&mut engine(&history,&[PackId::LensiskuJbotci],&[]),RelationTarget::Local(LocalRef::Assertion(jb.assertion.id.clone()))).status,ReferenceStatus::CatalogueOnly);
    println!("PASS revision-only history with identical complete assertion is not an ambiguous local reference");
    CollectionCatalogue::register(&history,request(&[PackId::LensiskuJbotci],&unequal_pairs,&[0])).unwrap();
    let error=engine(&history,&[PackId::LensiskuJbotci],&[]).execute(DictionaryRequest::Resolve{target:RelationTarget::Local(LocalRef::Assertion(jb.assertion.id.clone()))}).unwrap_err();
    assert_eq!(error.report().category,"ambiguous_reference");
    for target in [jb.target.clone(),equal_witness[0].target.clone(),unequal_witness[0].target.clone()] {
        assert_eq!(resolve(&mut engine(&history,&[PackId::LensiskuJbotci],&[]),target).status,ReferenceStatus::CatalogueOnly);
    }
    println!("PASS unequal historical body has typed ambiguity and exact release-qualified counterparts disambiguate");

    let catalogue=CollectionCatalogue::open(root).unwrap();let generation=catalogue.generation().unwrap().unwrap();drop(catalogue);
    let database=generation.store_path.join("dictionary.sqlite3");
    let original=std::fs::read(&database).unwrap();
    std::fs::write(base.join("pre-fault-dictionary.sqlite3"),&original).unwrap();
    for mode in ["missing","corrupt","version"] {
        let held=database.with_extension("cdc-held");assert!(!held.exists());
        match mode {
            "missing"=>std::fs::rename(&database,&held).unwrap(),
            "corrupt"=>std::fs::write(&database,b"not a SQLite database").unwrap(),
            "version"=>{
                let output=std::process::Command::new("sqlite3").arg(&database).arg("UPDATE metadata SET value='99' WHERE key='schema_version';").output().unwrap();
                assert!(output.status.success(),"{}",String::from_utf8_lossy(&output.stderr));
            },
            _=>unreachable!(),
        }
        let direct=vlajmaji_store::DictionaryStore::open(&generation.store_path).unwrap_err();
        let class=store_error_class(&direct);
        if mode=="version" {assert_eq!(class,"invalid:SchemaVersion");}
        {
            let mut current=engine(root,packs,&[]);
            let error=current.execute(DictionaryRequest::Resolve{target:jb.target.clone()}).unwrap_err();
            assert!(retains_store_cause(&error,&class),"{mode} reference lost typed {class}: {error:?}");
            let error=current.execute(DictionaryRequest::Exact{spelling:"farvi".into(),options:Default::default()}).unwrap_err();
            assert!(retains_store_cause(&error,&class),"{mode} exact lost typed {class}: {error:?}");
            assert_eq!(resolve(&mut current,RelationTarget::Unknown).status,ReferenceStatus::Unknown);
        }
        // Policy-excluded targets remain metadata-only even though this current
        // derived artifact is unusable. This is distinct from the selected case.
        let excluded=resolve(&mut engine(root,packs,&[PackId::LensiskuJbotci]),jb.target.clone());
        assert_eq!(excluded.status,ReferenceStatus::CatalogueOnly);assert_eq!(excluded.operational_reason.as_deref(),Some("disabled"));
        if mode=="missing" {std::fs::rename(&held,&database).unwrap();}else{std::fs::write(&database,&original).unwrap();}
        assert_eq!(std::fs::read(&database).unwrap(),original);
        assert_eq!(resolve(&mut engine(root,packs,&[]),jb.target.clone()).status,ReferenceStatus::Resolved);
        println!("PASS {mode} selected payload preserves typed minimum cause; disabled metadata and exact-byte restored counterparts");
    }
}
