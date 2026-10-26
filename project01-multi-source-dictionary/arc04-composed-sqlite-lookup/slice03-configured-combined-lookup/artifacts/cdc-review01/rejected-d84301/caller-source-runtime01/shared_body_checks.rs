// Functional regression qualifier, not an actual-corpus population witness.
#[path = "/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc04/vlajmaji/crates/lexicon/examples/support/dictionary_fixture.rs"]
mod fixture;
use std::{borrow::Cow,path::Path};
use vlajmaji_lexicon::{Availability, LocalRef, PackId, PackPayloadKind, RelationTarget, seal_release};
use vlajmaji_store::{CollectionCatalogue,CollectionInput,CollectionRequest,CollectionSelection};
use vlajmaji_search::{CollectionSearchEngine,CollectionQuerySelection,CollectionQueryResult,DictionarySearchEngine,DictionaryRequest,ReferenceStatus};

pub fn run(base:&Path) {
    std::fs::create_dir(base).unwrap();
    let mut document=fixture::complete_fixture().unwrap().into_document();
    // Match the existing accepted public fixture recipe: packs reject the
    // optional legacy archive, while all typed fixture records stay intact.
    document.legacy_archive=None;
    let dictionary=vlajmaji_lexicon::Dictionary::from_document(document).unwrap();
    let release=dictionary.metadata().releases.iter().find(|r|r.selected && r.availability==Availability::Available && !r.assertions.is_empty()).unwrap();
    let assertion=dictionary.metadata().assertions.iter().find(|a|release.assertions.contains(&a.id)).unwrap().clone();
    let target=RelationTarget::Local(LocalRef::Assertion(assertion.id.clone()));
    let packs=[PackId::LaLojban,PackId::Llg];
    let seals=packs.iter().map(|p|seal_release(*p,PackPayloadKind::Synthetic,&dictionary).unwrap()).collect::<Vec<_>>();
    assert_eq!(seals[0].release,seals[1].release);
    let request=CollectionRequest::new(packs.iter().zip(&seals).map(|(p,s)|CollectionInput::SealedBytes{pack_id:*p,release:Cow::Borrowed(&s.release),seal:Cow::Borrowed(&s.seal)}).collect(),CollectionSelection::Packs(packs.to_vec()),true).unwrap();
    let root=base.join("R");
    CollectionCatalogue::install(&root,request).unwrap();
    let catalogue=CollectionCatalogue::open(&root).unwrap();
    let generation=catalogue.generation().unwrap().unwrap();
    let order=catalogue.list().unwrap().packs.into_iter().map(|p|match p.pack_id.as_str(){"la-lojban"=>PackId::LaLojban,"llg"=>PackId::Llg,_=>panic!("unexpected pack")}).collect::<Vec<_>>();
    drop(catalogue);
    assert_eq!(order.len(),2);
    let baseline=DictionarySearchEngine::open(&generation.store_path).unwrap().resolve(&target).unwrap();
    assert_eq!(baseline.status,ReferenceStatus::Resolved);
    assert!(baseline.confirmed.iter().any(|h|h.assertion==assertion));
    let cases=vec![
        ("all-selected",order.clone(),vec![],true),
        ("first-selected",vec![order[0]],vec![],true),
        ("second-selected",vec![order[1]],vec![],true),
        ("first-disabled-second-selected",order.clone(),vec![order[0]],true),
        ("second-disabled-first-selected",order.clone(),vec![order[1]],true),
        ("all-disabled",order.clone(),order.clone(),false),
    ];
    let mut rows=Vec::new();
    for (label,selected,disabled,available) in cases {
        let mut engine=CollectionSearchEngine::open(&root,CollectionQuerySelection{packs:selected.clone(),disabled:disabled.clone(),profile:None}).unwrap();
        let result=engine.execute(DictionaryRequest::Resolve{target:target.clone()}).unwrap();
        let CollectionQueryResult::Resolve(response)=result.result else{panic!("wrong operation")};
        let expected=if available {ReferenceStatus::Resolved}else{ReferenceStatus::CatalogueOnly};
        let body_matches=if available {response.payload.as_ref().is_some_and(|p|p.confirmed==baseline.confirmed && p.unverified==baseline.unverified)} else{response.payload.is_none()};
        let pass=response.status==expected && body_matches;
        rows.push(serde_json::json!({"label":label,"selected":selected,"disabled":disabled,"expected_status":expected,"actual_status":response.status,"payload_present":response.payload.is_some(),"body_matches":body_matches,"operational_reason":response.operational_reason,"matches":response.catalogue_matches,"pass":pass}));
    }
    let passed=rows.iter().filter(|r|r["pass"]==true).count();
    let summary=serde_json::json!({"ok":passed==rows.len(),"scope":"same strict Synthetic model installed under two packs; LocalRef selected contributor regression","catalogue_order":order,"target":target,"setup_install_succeeded":true,"baseline_status":baseline.status,"passed":passed,"failed":rows.len()-passed,"cases":rows});
    std::fs::write(base.join("summary.json"),serde_json::to_vec_pretty(&summary).unwrap()).unwrap();
    println!("{summary}");
    if passed!=rows.len(){std::process::exit(1)}
}
