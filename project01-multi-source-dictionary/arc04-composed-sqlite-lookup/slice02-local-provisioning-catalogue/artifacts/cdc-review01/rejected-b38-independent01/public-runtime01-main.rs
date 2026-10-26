// Independent S02 caller, prepared while the author owns the target.
// No invocation is authorized until the exact candidate handback.
include!("helpers.rs");
use std::{borrow::Cow, path::{Path,PathBuf}};
#[path = "/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc04/vlajmaji/crates/lexicon/examples/support/dictionary_fixture.rs"]
mod complete_fixture;
use vlajmaji_store::{CollectionCatalogue as Cat,CollectionInput as Input,CollectionRequest as Request,CollectionSelection as Selection,CollectionErrorCategory as Category};
use vlajmaji_search::{DictionarySearchEngine as Engine,DictionaryExactOptions as Opt,DictionaryQueryError as QE};

fn request<'a>(pairs:&'a [(PackId,Vec<u8>,Vec<u8>)], indices:&[usize])->Request<'a> {
    Request::new(indices.iter().map(|&i|Input::SealedBytes{pack_id:pairs[i].0,release:Cow::Borrowed(&pairs[i].1),seal:Cow::Borrowed(&pairs[i].2)}).collect(),Selection::Packs(indices.iter().map(|&i|pairs[i].0).collect()),true).unwrap()
}
fn generation(root:&Path)->vlajmaji_store::CollectionGeneration {
    let c=Cat::open(root).unwrap();c.generation().unwrap().unwrap()
}
fn db_sha(path:&Path)->String {sha(&std::fs::read(path.join("dictionary.sqlite3")).unwrap())}
fn main() {
    // Public report leaf types must be nameable by an independent caller.
    let _: Option<(vlajmaji_store::CollectionInstalledHistory,vlajmaji_store::CollectionCaptureSummary,vlajmaji_store::CollectionReleaseSummary)>=None;
    let base=PathBuf::from(std::env::args_os().nth(1).expect("fresh scratch destination argument"));
    std::fs::create_dir(&base).expect("exclusive scratch root; do not reuse an attempt");
    let ms=models();
    let packs=[PackId::LensiskuJbotci,PackId::Llg,PackId::LaLojban,PackId::Sutysisku];
    let pairs:Vec<_>=packs.into_iter().zip(&ms).map(|(p,d)|{let s=seal_release(p,PackPayloadKind::Synthetic,d).unwrap();(p,s.release,s.seal)}).collect();
    let mut passed=0usize;
    macro_rules! ck {($label:literal,$condition:expr)=>{{assert!($condition,"{}",$label);passed+=1;println!("PASS {}",$label);}}}
    // All model inventories are derived before any catalogue install. The
    // accepted literal recipe supplies six assertions/five lexical, not guessed
    // constructor counts from the catalogue under review.
    let drafts:Vec<_>=ms.clone().into_iter().map(draft).collect();
    let combined=compose_dictionaries(ms.clone()).unwrap();let inv=inventory(&draft(combined.clone()));
    let mut expected:BTreeMap<&str,BTreeMap<String,usize>>=BTreeMap::new();
    for d in &drafts {for(k,rows)in inventory(d){let out=expected.entry(k).or_default();for(id,n)in rows {let old=out.entry(id).or_default();*old=(*old).max(n);}}}
    ck!("all 25 categories match max-multiset recipe including empty categories",inv==expected&&inv.len()==25&&inv["frames"].is_empty());
    let mut missing=expected.clone();missing.remove("frames");ck!("comparator rejects missing empty category",inv!=missing);
    let mut extra=expected.clone();extra.insert("not-a-category",BTreeMap::new());ck!("comparator rejects unexpected empty category",inv!=extra);
    let mut mult=expected.clone();*mult.get_mut("assertions").unwrap().values_mut().next().unwrap()+=1;ck!("comparator rejects changed multiplicity",inv!=mult);
    let mut presence=drafts[0].clone();presence.metadata.entries[0].spelling=Presence::Missing;ck!("comparator rejects value versus missing",inventory(&presence)!=inventory(&drafts[0]));
    ck!("literal assertions and eligibility census",combined.metadata().assertions.len()==6&&combined.metadata().assertions.iter().filter(|a|a.eligibility==Eligibility::Lexical).count()==5);
    // No writes may occur through ordinary missing-catalogue readers.
    let r=base.join("R");ck!("missing reader creates nothing",Cat::open(&r).is_err()&&!r.exists());
    let occupied=base.join("occupied-empty");std::fs::create_dir(&occupied).unwrap();
    let error=Cat::register(&occupied,request(&pairs,&[0])).unwrap_err();
    ck!("preexisting empty directory is occupied and unchanged",error.category()==Category::OccupiedDestination&&std::fs::read_dir(&occupied).unwrap().next().is_none());
    let occupied_file=base.join("occupied-file");std::fs::write(&occupied_file,b"keep exact sentinel").unwrap();
    let error=Cat::install(&occupied_file,request(&pairs,&[0])).unwrap_err();
    ck!("preexisting file is occupied with exact original bytes",error.category()==Category::OccupiedDestination&&std::fs::read(&occupied_file).unwrap()==b"keep exact sentinel");
    let unsupported=base.join("unsupported-catalogue");Cat::register(&unsupported,request(&pairs,&[0])).unwrap();
    let database=unsupported.join("catalogue.sqlite3");
    {let connection=rusqlite::Connection::open(&database).unwrap();connection.execute("UPDATE catalogue_meta SET value='999' WHERE key='schema_version'",[]).unwrap();}
    let original=std::fs::read(&database).unwrap();
    let read_error=Cat::open(&unsupported).unwrap_err();let write_error=Cat::register(&unsupported,request(&pairs,&[0])).unwrap_err();
    ck!("unknown catalogue version rejected by readers and mutations without changing bytes",read_error.category()==Category::UnsupportedVersion&&write_error.category()==Category::UnsupportedVersion&&std::fs::read(&database).unwrap()==original);
    let registration=Cat::register(&r,request(&pairs,&[0])).unwrap();
    ck!("register only revision one no active generation",registration.local_revision==1&&registration.generation.is_none());
    let registered=Cat::open(&r).unwrap();
    let descriptor=registered.reference_metadata(PackId::LensiskuJbotci,&sha(&pairs[0].1)).unwrap();
    ck!("metadata descriptor preserves exact registry and immutable release metadata",descriptor.registry==*ms[0].registry().as_document()&&descriptor.source_metadata.releases==ms[0].metadata().releases);
    ck!("descriptor capture identity metadata lengths retained without payload",descriptor.captures.len()==ms[0].captures().len()&&descriptor.captures.iter().zip(ms[0].captures()).all(|(a,b)|a.capture==b.identity&&a.metadata==b.metadata&&a.byte_length==b.bytes.len()));
    let rows=&descriptor.registry.occurrences;
    ck!("native 333 has assertion role not entry role",rows.len()==1&&rows[0].native_entry_id.is_none()&&rows[0].native_assertion_id.as_deref()==Some("333"));
    let encoded_descriptor=serde_json::to_vec(&descriptor).unwrap();
    ck!("registration does not retain fixture lexical definition or copy normalized payload files",!encoded_descriptor.windows(b"synthetic develops".len()).any(|w|w==b"synthetic develops")&&!r.join("release.json").exists()&&!r.join("seal.json").exists());
    drop(registered);
    let repeated=Cat::register(&r,request(&pairs,&[0])).unwrap();ck!("exact register repeat keeps revision and no attempt",repeated.local_revision==1&&repeated.attempt_id.is_none());
    let only_llg=Cat::install(&r,request(&pairs,&[1])).unwrap();ck!("install LLG preserves earlier JB registration",only_llg.local_revision==2&&Cat::open(&r).unwrap().reference_metadata(PackId::LensiskuJbotci,&sha(&pairs[0].1)).is_ok());
    let before=generation(&r);let before_hash=db_sha(&before.store_path);
    let mut corrupted=pairs[0].1.clone();corrupted[0]^=1;
    let invalid=Request::new(vec![Input::SealedBytes{pack_id:PackId::LensiskuJbotci,release:Cow::Borrowed(&corrupted),seal:Cow::Borrowed(&pairs[0].2)}],Selection::Packs(vec![PackId::LensiskuJbotci]),true).unwrap();
    let error=Cat::install(&r,invalid).unwrap_err();let unchanged=generation(&r);
    ck!("bad seal payload fails without changing prior generation bytes",error.category()==Category::InputInvalid&&unchanged.generation_id==before.generation_id&&db_sha(&before.store_path)==before_hash);
    let added=Cat::install(&r,request(&pairs,&[0])).unwrap();ck!("additive install advances one revision and retains two members",added.local_revision==3&&added.generation.as_ref().unwrap().members.len()==2);
    let final_r=added.generation.unwrap();let final_hash=db_sha(&final_r.store_path);
    let repeat=Cat::install(&r,request(&pairs,&[0])).unwrap();let repeat_generation=repeat.generation.unwrap();
    ck!("exact install repeat preserves content physical IDs revision and DB bytes",repeat.local_revision==3&&repeat.attempt_id.is_none()&&repeat_generation.content_id==final_r.content_id&&repeat_generation.generation_id==final_r.generation_id&&db_sha(&final_r.store_path)==final_hash);
    let q=base.join("Q");Cat::install(&q,request(&pairs,&[0])).unwrap();let reverse=Cat::install(&q,request(&pairs,&[1])).unwrap();let final_q=reverse.generation.unwrap();
    ck!("opposite order gives same content canonical but distinct physical generation",reverse.local_revision==2&&final_q.content_id==final_r.content_id&&final_q.canonical_sha256==final_r.canonical_sha256&&final_q.generation_id!=final_r.generation_id);
    ck!("earlier immutable generation remains retained",before.store_path.is_dir()&&db_sha(&before.store_path)==before_hash);
    let engine=Engine::open_snapshot(vlajmaji_store::DictionaryReadTopology::Unified,final_r.snapshot_manifest()).unwrap();
    let full=engine.exact("farvi",Opt::default()).unwrap();
    ck!("two-pack query has two occurrences two independent source groups",full.totals.occurrences==2&&full.totals.source_groups==2);
    for (source,id) in [("LSK","lensisku"),("LLG","llg")] {let got=engine.exact("farvi",Opt{sources:vec![source.into()],..Opt::default()}).unwrap();assert_eq!(got.totals.occurrences,1);assert_eq!(got.groups.len(),1);assert_eq!(got.groups[0].source.id.as_str(),id);}
    ck!("both source filters preserve one received assertion",true);
    let mut cursor=None;let mut ids=Vec::new();let mut pages=0;
    loop {pages+=1;assert!(pages<=2);let page=engine.exact("farvi",Opt{page_size:Some(1),cursor,..Opt::default()}).unwrap();assert_eq!(page.totals,full.totals);ids.extend(page.groups.into_iter().flat_map(|g|g.hits.into_iter().map(|h|h.assertion.id)));cursor=page.next_cursor;if cursor.is_none(){break}}
    ck!("global pages concatenate exact full assertion order",ids==full.groups.iter().flat_map(|g|g.hits.iter().map(|h|h.assertion.id.clone())).collect::<Vec<_>>());
    for index in [0usize,1] {let id=ms[index].metadata().assertions[0].id.clone();assert!(engine.resolve(&RelationTarget::Local(LocalRef::Assertion(id.clone()))).is_ok());assert!(matches!(engine.resolve(&RelationTarget::Local(LocalRef::Entry(id))),Err(QE::InvalidReference{..})));}
    ck!("local assertion positive and typed wrong-role controls both sources",true);
    ck!("local mutation revision does not replace upstream query catalogue revision",full.catalogue_revision==ms[0].metadata().catalogue_revision);
    let mut complete=complete_fixture::complete_fixture().unwrap().into_document();
    let first=complete.metadata.editorial_senses[0].assertions[0].clone();
    complete.metadata.editorial_senses[0].assertions.push(first);
    let complete=Dictionary::from_document(complete).unwrap();
    // The fixture literal/source recipe has seven forms and one editorial sense;
    // three assertion memberships include an intentional repeated first item.
    assert_eq!(complete.metadata().forms.len(),7);assert_eq!(complete.metadata().editorial_senses.len(),1);
    assert_eq!(complete.metadata().editorial_senses[0].assertions.len(),3);
    let sealed=seal_release(PackId::LensiskuJbotci,PackPayloadKind::Synthetic,&complete).unwrap();
    let complete_pair=vec![(PackId::LensiskuJbotci,sealed.release,sealed.seal)];
    let refs_root=base.join("nonempty-references");Cat::register(&refs_root,request(&complete_pair,&[0])).unwrap();
    let refs=Cat::open(&refs_root).unwrap().reference_metadata(PackId::LensiskuJbotci,&sha(&complete_pair[0].1)).unwrap();
    for form in &complete.metadata().forms {
        let found=refs.entities.iter().find(|e|e.role=="form"&&e.id==form.id.as_str()).unwrap();
        let source=complete.metadata().entries.iter().find(|e|e.id==form.entry).unwrap().source.to_string();
        assert_eq!(found.entry_ids,vec![form.entry.to_string()]);assert!(found.assertion_ids.is_empty());
        assert_eq!(found.source_ids,vec![source]);assert_eq!(found.evidence,form.evidence);
        assert_eq!(found.occurrence,form.evidence.as_ref().map(|x|x.occurrence.clone()));
    }
    ck!("all seven nonempty forms retain exact entry source and evidence links",true);
    for sense in &complete.metadata().editorial_senses {
        let found=refs.entities.iter().find(|e|e.role=="editorial_sense"&&e.id==sense.id.as_str()).unwrap();
        let assertions=sense.assertions.iter().map(ToString::to_string).collect::<Vec<_>>();
        let entries=sense.assertions.iter().map(|id|complete.metadata().assertions.iter().find(|a|&a.id==id).unwrap().entry.to_string()).collect::<std::collections::BTreeSet<_>>().into_iter().collect::<Vec<_>>();
        let sources=entries.iter().map(|id|complete.metadata().entries.iter().find(|e|e.id.as_str()==id).unwrap().source.to_string()).collect::<std::collections::BTreeSet<_>>().into_iter().collect::<Vec<_>>();
        assert_eq!(found.assertion_ids,assertions);assert_eq!(found.entry_ids,entries);assert_eq!(found.source_ids,sources);
        assert_eq!(found.source_ids,vec!["source-a".to_owned(),"source-b".to_owned()]);
    }
    ck!("editorial descriptor retains multi-source links and repeated assertion memberships",true);
    let head=&complete.metadata().owned_heads[0];let key=format!("owned_head:{}",head.source);
    ck!("owned head body digest uses source identity key",refs.body_sha256.get(&key)==Some(&sha(&serde_json::to_vec(head).unwrap())));
    // Historical registrations are not co-active replacements.
    let mut next=ms[0].clone().into_document();next.metadata.catalogue_revision+=1;
    let next=Dictionary::from_document(next).unwrap();
    let changed=seal_release(PackId::LensiskuJbotci,PackPayloadKind::Synthetic,&next).unwrap();
    let changed_pair=vec![(PackId::LensiskuJbotci,changed.release,changed.seal)];
    let reg=Cat::register(&r,request(&changed_pair,&[0])).unwrap();
    ck!("historical different release may register without replacing installed head",reg.local_revision==4&&generation(&r).generation_id==final_r.generation_id);
    let update=Cat::install(&r,request(&changed_pair,&[0])).unwrap_err();
    ck!("different installed revision is update_required and leaves head unchanged",update.category()==Category::UpdateRequired&&generation(&r).generation_id==final_r.generation_id);
    let previous_hash=db_sha(&final_r.store_path);
    let mut conflicting=ms[2].clone().into_document();let mut registry=conflicting.registry.into_document();registry.sources[0].display_name="explicit unequal same-ID source fixture".into();conflicting.registry=SourceRegistry::from_document(registry).unwrap();
    let conflicting=Dictionary::from_document(conflicting).unwrap();let sealed=seal_release(PackId::LaLojban,PackPayloadKind::Synthetic,&conflicting).unwrap();let conflict_pair=vec![(PackId::LaLojban,sealed.release,sealed.seal)];
    let conflict=Cat::install(&r,request(&conflict_pair,&[0])).unwrap_err();
    ck!("co-active unequal shared source is typed conflict without activation",conflict.category()==Category::IdentityConflict&&generation(&r).generation_id==final_r.generation_id&&db_sha(&final_r.store_path)==previous_hash);
    let mut conflicting=ms[2].clone().into_document();let old=conflicting.metadata.collections[0].id.clone();let shared=ms[0].metadata().collections[0].id.clone();conflicting.metadata.collections[0].id=shared.clone();conflicting.metadata.collections[0].display_name="explicit unequal collection fixture".into();for component in &mut conflicting.metadata.components{if component.collection==old{component.collection=shared.clone();}}
    let conflicting=Dictionary::from_document(conflicting).unwrap();let sealed=seal_release(PackId::LaLojban,PackPayloadKind::Synthetic,&conflicting).unwrap();let conflict_pair=vec![(PackId::LaLojban,sealed.release,sealed.seal)];
    let conflict=Cat::install(&r,request(&conflict_pair,&[0])).unwrap_err();
    ck!("co-active unequal same-ID collection is typed conflict without activation",conflict.category()==Category::IdentityConflict&&generation(&r).generation_id==final_r.generation_id);
    let added_la=Cat::install(&r,request(&pairs,&[2])).unwrap();
    let latest=Engine::open_snapshot(vlajmaji_store::DictionaryReadTopology::Unified,added_la.generation.unwrap().snapshot_manifest()).unwrap();
    let result=latest.exact("farvi",Opt::default()).unwrap();
    let sources=result.source_totals.iter().map(|x|(x.source.id.to_string(),x.occurrences)).collect::<BTreeMap<_,_>>();
    ck!("equal shared Lensisku source composes received assertions without new authority",result.totals.occurrences==3&&result.totals.source_groups==2&&sources==[("lensisku".to_owned(),2),("llg".to_owned(),1)].into_iter().collect());
    // Targets are derived from the pinned input adapters, never selected from
    // the catalogue/query candidate under review. Both suppliers carry native
    // assertion 333, but their capture/release scope must remain distinct.
    let mut resolved_ids=Vec::new();
    for index in [0usize,2] {
        let expected_assertion=&ms[index].metadata().assertions[0];
        let release=ms[index].metadata().releases.iter().find(|r|r.assertions.contains(&expected_assertion.id)).unwrap();
        let attribution=ms[index].registry().as_document().occurrences.iter().find(|o|o.occurrence==expected_assertion.occurrence.to_record_identity().unwrap()).cloned().unwrap();
        assert_eq!(attribution.native_assertion_id.as_deref(),Some("333"));
        let target=RelationTarget::Catalog(CatalogTarget {
            source:SourceId::new("lensisku").unwrap(),release:release.id.clone(),
            native_entry_id:None,native_assertion_id:Some("333".into()),
            occurrence:Some(expected_assertion.occurrence.clone()),
        });
        let answer=latest.resolve(&target).unwrap();
        assert_eq!(answer.status,vlajmaji_search::ReferenceStatus::Resolved);
        assert_eq!(answer.target,target);assert!(answer.unverified.is_empty());
        assert_eq!(answer.confirmed.len(),1);
        let hit=&answer.confirmed[0];
        assert_eq!(hit.assertion,*expected_assertion);
        assert_eq!(hit.occurrence_attribution.as_ref(),Some(&attribution));
        assert!(hit.release_ids.contains(&release.id));
        assert_eq!(answer.requested_release.as_ref().map(|r|&r.id),Some(&release.id));
        resolved_ids.push(hit.assertion.id.clone());
    }
    ck!("same native333 preserves independent JB and La release capture and assertion scope",resolved_ids[0]!=resolved_ids[1]);
    ck!("already pinned engine remains on prior immutable installed generation",engine.exact("farvi",Opt::default()).unwrap()==full&&db_sha(&final_r.store_path)==previous_hash);
    let before=generation(&r);let protected=db_sha(&before.store_path);
    let pinned=Cat::open(&r).unwrap();let prior=pinned.list().unwrap();
    let blocked=Cat::install(&r,request(&pairs,&[3])).unwrap_err();let report=blocked.report("install");
    ck!("pinned public catalogue reader forces known commit nonactivation with context",blocked.category()==Category::Busy&&report.version==1&&!report.ok&&report.activation.as_deref()==Some("not_activated")&&report.prior_local_revision==Some(prior.local_revision)&&report.attempt_id.is_some()&&report.proposed_generation.is_some()&&report.observed_generation.as_deref()==Some(before.generation_id.as_str()));
    let mut cause:&(dyn std::error::Error+'static)=&blocked;let mut sqlite=false;
    loop{if cause.downcast_ref::<rusqlite::Error>().is_some(){sqlite=true;}match cause.source(){Some(next)=>cause=next,None=>break}}
    ck!("commit error retains typed SQLite cause through context wrappers",sqlite);
    drop(pinned);
    let after=Cat::open(&r).unwrap().list().unwrap();
    ck!("failed commit retains revision old generation and exact original DB bytes",after.local_revision==prior.local_revision&&after.generation.unwrap().generation_id==before.generation_id&&db_sha(&before.store_path)==protected&&!after.staged_attempts.is_empty());
    let final_install=Cat::install(&r,request(&pairs,&[3])).unwrap();let all=final_install.generation.unwrap();
    ck!("reader-release counterpart installs complete four-pack canonical content",all.members.len()==4&&all.canonical_sha256==sha(&combined.to_json_bytes().unwrap()));
    let loaded=vlajmaji_store::DictionaryStore::open(&all.store_path).unwrap().unverified_document().unwrap();
    ck!("actual installed four-pack model matches all predeclared category multisets",inventory(&loaded)==expected);
    let complete_engine=Engine::open_snapshot(vlajmaji_store::DictionaryReadTopology::Unified,all.snapshot_manifest()).unwrap();
    let all_query=complete_engine.exact("farvi",Opt::default()).unwrap();
    let sources=all_query.source_totals.iter().map(|x|(x.source.id.to_string(),x.occurrences)).collect::<BTreeMap<_,_>>();
    ck!("four received packs retain three authorities and five eligible farvi occurrences",all_query.totals.occurrences==5&&all_query.totals.source_groups==3&&sources==[("lensisku".to_owned(),2),("llg".to_owned(),1),("sutysisku".to_owned(),2)].into_iter().collect());
    println!("{}",serde_json::json!({"ok":true,"scope":"independent tiny S02 outside caller","passed":passed,"all_category_count":inv.len(),"base":base}));
}
