// Prospective S05 SOURCE ONLY, adapted from accepted S03 shared-body controls.
// Not compiled/executed; original four-literal population is checked separately.
// Original source SHA256: 2375528d3d5c0639a2a394b38c58410ec8c04f945cfeac86e504cd47abf6d4d2
#[path = "complete_fixture.rs"]
mod fixture;
use std::{borrow::Cow,path::Path};
use vlajmaji_lexicon::{Availability, LocalRef, PackId, PackPayloadKind, RelationTarget, seal_release};
use vlajmaji_store::{CollectionCatalogue,CollectionInput,CollectionRequest,CollectionSelection};
use vlajmaji_search::{CollectionCompactSearchEngine,CollectionQuerySelection,CollectionCompactQueryResult,DictionarySearchEngine,DictionaryRequest,ReferenceStatus};

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
    crate::assert_owned_store_path(&root,&generation.store_path);
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
        let mut engine=CollectionCompactSearchEngine::open(&root,CollectionQuerySelection{packs:selected.clone(),disabled:disabled.clone(),profile:None}).unwrap();
        let result=engine.execute(DictionaryRequest::Resolve{target:target.clone()}).unwrap();
        let CollectionCompactQueryResult::Resolve(response)=result.result else{panic!("wrong operation")};
        let expected=if available {ReferenceStatus::Resolved}else{ReferenceStatus::CatalogueOnly};
        let body_matches=if available {response.payload.as_ref().is_some_and(|p| {
            let actual=p.confirmed.iter().map(|h|(&h.entry,&h.assertion,&h.occurrence_id,&h.occurrence_attribution,&h.release_ids)).collect::<Vec<_>>();
            let expected=baseline.confirmed.iter().map(|h|(&h.entry,&h.assertion,&h.occurrence_id,&h.occurrence_attribution,&h.release_ids)).collect::<Vec<_>>();
            actual==expected && p.unverified.is_empty() && baseline.unverified.is_empty()
        })} else{response.payload.is_none()};
        let pass=response.status==expected && body_matches;
        rows.push(serde_json::json!({"label":label,"selected":selected,"disabled":disabled,"expected_status":expected,"actual_status":response.status,"payload_present":response.payload.is_some(),"body_matches":body_matches,"operational_reason":response.operational_reason,"matches":response.catalogue_matches,"pass":pass}));
    }
    let passed=rows.iter().filter(|r|r["pass"]==true).count();
    let summary=serde_json::json!({"ok":passed==rows.len(),"scope":"same strict Synthetic model installed under two packs; LocalRef selected contributor regression","catalogue_order":order,"target":target,"setup_install_succeeded":true,"baseline_status":baseline.status,"passed":passed,"failed":rows.len()-passed,"cases":rows});
    std::fs::write(base.join("summary.json"),serde_json::to_vec_pretty(&summary).unwrap()).unwrap();
    println!("{summary}");
    if passed!=rows.len(){std::process::exit(1)}
}

pub fn run_entry(base:&Path) {
    use std::collections::BTreeMap;
    std::fs::create_dir(base).unwrap();
    let first=crate::models().remove(1); // Exact named single-row LLG literal.
    let mut value=serde_json::to_value(first.clone().into_document()).unwrap();
    let mut renamed=BTreeMap::new();
    for category in ["releases","assertions","contexts","frames","forms","etymologies","relations","owned_revisions","editorial_senses","computations","annotations"] {
        for row in value["metadata"][category].as_array().unwrap() {
            if let Some(id)=row.get("id").and_then(serde_json::Value::as_str){renamed.insert(id.to_owned(),format!("second-{id}"));}
        }
    }
    // Distinct received assertions require distinct captured occurrences; keep
    // exact input bytes/digests while naming a second Synthetic capture scope.
    for capture in first.captures() {
        renamed.insert(capture.identity.namespace.clone(),format!("captured:second:{}",capture.identity.namespace.strip_prefix("captured:").unwrap()));
    }
    fn rename(v:&mut serde_json::Value, map:&BTreeMap<String,String>){match v {
        serde_json::Value::String(s)=>if let Some(next)=map.get(s){*s=next.clone()},
        serde_json::Value::Array(a)=>for x in a{rename(x,map)},
        serde_json::Value::Object(o)=>for x in o.values_mut(){rename(x,map)},_=>{}
    }}
    rename(&mut value,&renamed);
    let second=vlajmaji_lexicon::Dictionary::from_json_bytes(&serde_json::to_vec(&value).unwrap()).unwrap();
    assert_eq!(first.metadata().entries,second.metadata().entries);
    let _qualified_composition=vlajmaji_lexicon::compose_dictionaries(vec![first.clone(),second.clone()]).unwrap();
    let entry=first.metadata().entries[0].clone();
    let expected=[&first,&second].map(|d|d.metadata().assertions.iter().filter(|a|a.entry==entry.id).map(|a|(a.id.to_string(),a.clone())).collect::<BTreeMap<_,_>>());
    assert!(!expected[0].is_empty());assert!(!expected[1].is_empty());assert!(expected[0].keys().all(|id|!expected[1].contains_key(id)));
    let releases=[&first,&second].map(|d|d.metadata().releases.iter().filter(|r|r.selected&&r.availability==Availability::Available).map(|r|r.id.to_string()).collect::<std::collections::BTreeSet<_>>());
    assert!(releases[0].is_disjoint(&releases[1]));
    let memberships=[&first,&second].map(|d|d.metadata().assertions.iter().map(|a| {
        let ids=d.metadata().releases.iter().filter(|r|r.selected&&r.availability==Availability::Available&&r.assertions.contains(&a.id)).map(|r|r.id.to_string()).collect::<std::collections::BTreeSet<_>>();
        (a.id.to_string(),ids)
    }).collect::<BTreeMap<_,_>>());
    for i in 0..2 {for id in expected[i].keys(){assert!(!memberships[i][id].is_empty());}}

    std::fs::write(base.join("pre-install-expectations.json"),serde_json::to_vec_pretty(&serde_json::json!({"entry":entry,"assertions":expected,"selected_releases":releases,"per_assertion_memberships":memberships})).unwrap()).unwrap();
    let packs=[PackId::LaLojban,PackId::Llg];
    let seals=packs.iter().zip([&first,&second]).map(|(p,d)|seal_release(*p,PackPayloadKind::Synthetic,d).unwrap()).collect::<Vec<_>>();
    let request=CollectionRequest::new(packs.iter().zip(&seals).map(|(p,s)|CollectionInput::SealedBytes{pack_id:*p,release:Cow::Borrowed(&s.release),seal:Cow::Borrowed(&s.seal)}).collect(),CollectionSelection::Packs(packs.to_vec()),true).unwrap();
    let root=base.join("R");CollectionCatalogue::install(&root,request).unwrap();
    let target=RelationTarget::Local(LocalRef::Entry(entry.id));
    let cases=[("all-selected",vec![0,1],vec![]),("first-selected",vec![0],vec![]),("second-selected",vec![1],vec![]),("first-disabled",vec![0,1],vec![0]),("second-disabled",vec![0,1],vec![1])];
    let mut rows=Vec::new();
    for (label,selected,disabled) in cases {
        let admitted=selected.iter().copied().filter(|i|!disabled.contains(i)).collect::<Vec<_>>();
        let wanted=admitted.iter().flat_map(|i|expected[*i].clone()).collect::<BTreeMap<_,_>>();
        let mut engine=CollectionCompactSearchEngine::open(&root,CollectionQuerySelection{packs:selected.iter().map(|i|packs[*i]).collect(),disabled:disabled.iter().map(|i|packs[*i]).collect(),profile:None}).unwrap();
        let CollectionCompactQueryResult::Resolve(response)=engine.execute(DictionaryRequest::Resolve{target:target.clone()}).unwrap().result else{panic!("wrong operation")};
        let actual=response.payload.as_ref().map(|p|p.confirmed.iter().map(|h|(h.assertion.id.to_string(),h.assertion.clone())).collect::<BTreeMap<_,_>>()).unwrap_or_default();
        let release_scope_ok=response.payload.as_ref().is_some_and(|p|p.confirmed.iter().all(|h| {
            let wanted=admitted.iter().filter_map(|i|memberships[*i].get(h.assertion.id.as_str())).flat_map(|ids|ids.iter().cloned()).collect::<std::collections::BTreeSet<_>>();
            let actual=h.release_ids.iter().map(ToString::to_string).collect::<std::collections::BTreeSet<_>>();
            actual==wanted && actual.len()==h.release_ids.len()
        }));
        let pass=response.status==ReferenceStatus::Resolved && actual==wanted && release_scope_ok;
        rows.push(serde_json::json!({"label":label,"pass":pass,"status":response.status,"expected_assertion_ids":wanted.keys().collect::<Vec<_>>(),"actual_assertion_ids":actual.keys().collect::<Vec<_>>(),"release_scope_ok":release_scope_ok,"actual_release_ids":response.payload.as_ref().map(|p|p.confirmed.iter().map(|h|(&h.assertion.id,&h.release_ids)).collect::<Vec<_>>()),"operational_reason":response.operational_reason,"matches":response.catalogue_matches}));
    }
    let passed=rows.iter().filter(|r|r["pass"]==true).count();let summary=serde_json::json!({"ok":passed==rows.len(),"scope":"strict equal Entry body with distinct predeclared assertions and releases across two packs","setup_install_succeeded":true,"passed":passed,"failed":rows.len()-passed,"cases":rows});
    std::fs::write(base.join("summary.json"),serde_json::to_vec_pretty(&summary).unwrap()).unwrap();println!("{summary}");if passed!=rows.len(){std::process::exit(1)}
}
