use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use vlajmaji_lexicon::*;
use vlajmaji_lexicon::pack::compose_unverified_documents;

fn sha(b: &[u8]) -> String { format!("{:x}", Sha256::digest(b)) }
const LENS: &[u8] = include_bytes!("../../fixtures/lensisku.json");
const SUT: &[u8] = include_bytes!("../../fixtures/sutysisku.json");
const LLG: &[u8] = include_bytes!("../../fixtures/llg-valid-row.txt");
fn models() -> Vec<Dictionary> {
    let revision = "0".repeat(40);
    let meta = format!("language_tag = \"en\"\nlanguage_realname = \"English\"\nsource_language_tag = \"jbo\"\nformat = \"json\"\npositive_scores_only = false\nfilename = \"dictionary-en.json\"\nmetadata_url = \"https://example.invalid/languages\"\ndownload_url = \"https://example.invalid/synthetic\"\nlensisku_created_at = \"2026-10-04T03:07:08Z\"\nsha256 = \"{}\"\ndefinition_count = 1\nentry_count = 1\n", sha(LENS));
    let jb = import_jbotci_export(LENS, meta.as_bytes(), &JbotciImportOptions { revision: revision.clone(), expected_raw_sha256: sha(LENS), expected_metadata_sha256: sha(meta.as_bytes()) }).unwrap().into_dictionary();
    let la = import_la_lojban_export(LENS, &LaLojbanImportOptions { revision: revision.clone(), expected_raw_sha256: sha(LENS), raw_locator: Some("https://example.invalid/la".into()), notice_locator: Some("https://example.invalid/code-notice".into()) }).unwrap().into_dictionary();
    let notice = b"Explicit synthetic notice";
    let llg = import_llg_lists(LLG,b"",b"",notice,&LlgImportOptions { expected_gismu_sha256: sha(LLG),expected_cmavo_sha256:sha(b""),expected_rafsi_sha256:sha(b""),expected_notice_sha256:sha(notice),base_locator:"https://example.invalid/llg".into() }).unwrap().into_dictionary();
    let sut = import_sutysisku_export(SUT, &SutysiskuImportOptions { revision, expected_raw_sha256:sha(SUT),raw_locator:Some("https://example.invalid/sut".into()),notice_locator:Some("https://example.invalid/code-notice".into()) }).unwrap().into_dictionary();
    vec![jb,llg,la,sut]
}
fn draft(d: Dictionary) -> UnverifiedDictionaryDocument {
    let x=d.into_document();
    UnverifiedDictionaryDocument {version:x.version,legacy_archive:x.legacy_archive,registry:x.registry.into_document(),captures:x.captures,metadata:x.metadata}
}
fn inventory(d:&UnverifiedDictionaryDocument)->BTreeMap<&'static str,BTreeMap<String,usize>> {
    fn rows<T:serde::Serialize>(xs:&[T])->BTreeMap<String,usize>{let mut out=BTreeMap::new(); for x in xs {*out.entry(sha(&serde_json::to_vec(x).unwrap())).or_default()+=1;}out}
    let mut out=BTreeMap::new();let m=&d.metadata;let r=&d.registry;
    macro_rules! metadata {($($k:ident),*)=>{$(out.insert(stringify!($k),rows(&m.$k));)*}}
    metadata!(collections,components,releases,code_notices,entries,assertions,contexts,forms,etymologies,frames,relations,owned_revisions,owned_heads,editorial_senses,computations,annotations,extensions);
    macro_rules! registry {($($k:ident),*)=>{$(out.insert(stringify!($k),rows(&r.$k));)*}}
    registry!(sources,contributors,distributors,aliases,lineage,snapshots,occurrences);
    out.insert("captures",rows(&d.captures));out
}
fn controls()->usize {
    let mut passed=0;
    macro_rules! ck {($name:literal,$v:expr)=>{{assert!($v,"{}",$name);println!("PASS {}",$name);passed+=1;}}}
    let ms=models();let ds:Vec<_>=ms.clone().into_iter().map(draft).collect();
    let strict=compose_dictionaries(ms).unwrap();let strict_draft=draft(strict.clone());
    let merged=compose_unverified_documents(ds.clone()).unwrap();
    ck!("all categories exact strict-unverified equality",merged==strict_draft);
    let actual=inventory(&merged);let mut expected:BTreeMap<&str,BTreeMap<String,usize>>=BTreeMap::new();
    for d in &ds {for(kind,rs)in inventory(d){let target=expected.entry(kind).or_default();for(k,n)in rs {let current=target.entry(k).or_default();*current=(*current).max(n);}}}
    ck!("full category inventory incl empty matches independent max union",actual==expected && actual["frames"].is_empty() && !actual["entries"].is_empty());
    let mut missing=expected.clone();missing.remove("frames");ck!("comparator missing empty category rejects",actual!=missing);
    let mut wrong=expected.clone();*wrong.get_mut("entries").unwrap().values_mut().next().unwrap()+=1;ck!("comparator changed multiplicity rejects",actual!=wrong);
    ck!("literal-census assertion total",merged.metadata.assertions.len()==6);
    ck!("literal-census lexical total",merged.metadata.assertions.iter().filter(|x|x.eligibility==Eligibility::Lexical).count()==5);
    ck!("literal-census source set",merged.registry.sources.iter().map(|x|x.id.as_str()).collect::<std::collections::BTreeSet<_>>()==["lensisku","llg","sutysisku"].into_iter().collect());
    ck!("repeated complete input idempotent",compose_unverified_documents(vec![merged.clone(),merged.clone()]).unwrap()==merged);
    let mut bad=merged.clone();bad.version=2;ck!("unsupported document version",matches!(compose_unverified_documents(vec![bad]),Err(PackError::UnsupportedVersion{version:2,..})));
    let mut bad=merged.clone();bad.registry.version=2;ck!("unsupported registry version",matches!(compose_unverified_documents(vec![bad]),Err(PackError::UnsupportedVersion{version:2,..})|Err(PackError::Registry(RegistryError::UnsupportedVersion(2)))));
    let mut bad=merged.clone();bad.legacy_archive=Some(vec![]);ck!("legacy rejected",matches!(compose_unverified_documents(vec![bad]),Err(PackError::LegacyArchiveUnsupported)));
    let mut bad=ds[0].clone();bad.registry.sources.push(bad.registry.sources[0].clone());ck!("within input identical source duplicate rejected",matches!(compose_unverified_documents(vec![bad]),Err(PackError::Conflict{..})|Err(PackError::Registry(RegistryError::DuplicateEntity{..}))));
    let mut bad=ds[0].clone();bad.metadata.entries.push(bad.metadata.entries[0].clone());ck!("within input identical entry duplicate rejected",matches!(compose_unverified_documents(vec![bad]),Err(PackError::Conflict{..})));
    let mut bad=ds[0].clone();bad.registry.sources[0].display_name="conflicting source body".into();ck!("cross input unequal same source rejects",matches!(compose_unverified_documents(vec![ds[0].clone(),bad]),Err(PackError::Conflict{category:"source",..})));
    let mut bad=ds[0].clone();bad.metadata.entries[0].spelling=Presence::Value("different".into());ck!("cross input unequal same entry rejects",matches!(compose_unverified_documents(vec![ds[0].clone(),bad]),Err(PackError::Conflict{category:"entry",..})));
    let mut ext=ds[0].clone();let mut a=ext.metadata.extensions[0].clone();a.name="cdc.repeated".into();a.value=RawJson::new("1").unwrap();let mut b=a.clone();b.value=RawJson::new("2").unwrap();ext.metadata.extensions=vec![a.clone(),a.clone(),b.clone(),b.clone()];
    let joined=compose_unverified_documents(vec![ext.clone(),ext.clone()]).unwrap();ck!("interleaved extension multiplicities and idempotence",joined.metadata.extensions==vec![a.clone(),a.clone(),b.clone(),b.clone()]);
    let mut more=ext.clone();more.metadata.extensions=vec![a.clone(),b.clone(),b.clone(),b.clone()];let joined=compose_unverified_documents(vec![ext,more]).unwrap();ck!("independent enumerated extension maxima",joined.metadata.extensions==vec![a.clone(),a,b.clone(),b.clone(),b] );
    let mut graph=merged.clone();graph.metadata.assertions[0].entry=EntityId::new("cdc-missing-entry").unwrap();let loaded=compose_unverified_documents(vec![graph]).unwrap();ck!("unverified graph remains unverified",Dictionary::from_json_bytes(&serde_json::to_vec(&loaded).unwrap()).is_err());
    passed
}
fn snapshot_controls(root: &std::path::Path) -> usize {
    use vlajmaji_search::{DictionaryExactOptions as Opt, DictionarySearchEngine as Engine, DictionaryQueryError as QE};
    use vlajmaji_store::{DictionaryReadTopology as T, DictionarySnapshotManifest as Manifest, DictionarySnapshotMember as Member, DictionaryStore as Store};
    let mut passed=0;
    macro_rules! ck {($name:literal,$v:expr)=>{{assert!($v,"{}",$name);println!("PASS {}",$name);passed+=1;}}}
    std::fs::create_dir(root).unwrap();
    let ms=models();
    let unified=compose_dictionaries(ms.clone()).unwrap();
    let digest=sha(&unified.to_json_bytes().unwrap());
    let mut members=Vec::new();
    for (n,d) in ms.iter().enumerate() {
        let path=root.join(format!("pack-{n}"));
        drop(Store::create(&path,d).unwrap());
        members.push(Member{id:format!("pack-{n}"),path,expected_state_sha256:sha(&d.to_json_bytes().unwrap())});
    }
    let u=root.join("unified");drop(Store::create(&u,&unified).unwrap());
    let manifest=|topology:T,generation:&str,selected:Option<Vec<EntityId>>|Manifest {
        version:1,generation:generation.into(),logical_state_sha256:digest.clone(),
        members:if topology==T::Unified {vec![Member{id:"unified".into(),path:u.clone(),expected_state_sha256:digest.clone()}]}else{members.clone()},
        selected_release_ids:selected,
    };
    let baseline=Engine::open_snapshot(T::Unified,manifest(T::Unified,"generation-one",None)).unwrap();
    let expected=baseline.exact("farvi",Opt::default()).unwrap();
    let mut expected_wire=serde_json::to_vec(&serde_json::to_value(&expected).unwrap()).unwrap();expected_wire.push(b'\n');{use std::io::Write;let mut f=std::fs::OpenOptions::new().write(true).create_new(true).open(root.join("expected-full-wire.json")).unwrap();f.write_all(&expected_wire).unwrap();}
    ck!("four-pack literal five hits three source groups",expected.totals.occurrences==5 && expected.totals.source_groups==3 && expected.native_candidate_occurrences==5);
    for topology in [T::Unified,T::Attached,T::Fanout] {
        let diagnostic=vlajmaji_store::DictionaryReadSnapshot::open(topology,manifest(topology,"generation-one",None)).unwrap();
        println!("{}",serde_json::json!({"scope":"tiny actual provider diagnostic","topology":format!("{topology:?}"),"live_attached_limit":diagnostic.runtime_attached_limit()}));drop(diagnostic);
        let current=Engine::open_snapshot(topology,manifest(topology,"generation-one",None)).unwrap();let e=&current;
        ck!("same complete full value across provider",e.exact("farvi",Opt::default()).unwrap()==expected);
        for (source,n) in [("LSK",2),("LLG",1),("SUT",2)] {
            let got=e.exact("farvi",Opt{sources:vec![source.into()],..Opt::default()}).unwrap();
            assert_eq!(got.totals.occurrences,n,"literal source set {source}");
            assert_eq!(got.totals.source_groups,1);
        }
        ck!("literal source filters retain global native count",e.exact("farvi",Opt{sources:vec!["SUT".into()],..Opt::default()}).unwrap().native_candidate_occurrences==5);
        ck!("source-excluded raw-positive yields query zero",e.exact("zoizoizoi",Opt{sources:vec!["SUT".into()],..Opt::default()}).unwrap().totals.occurrences==0);
        let zero=e.exact("farvi",Opt{page_size:Some(0),..Opt::default()}).unwrap();
        ck!("page zero keeps totals without hits or cursor",zero.totals==expected.totals&&zero.groups.is_empty()&&zero.next_cursor.is_none());
        let mut cursor=None;let mut actual=Vec::new();let mut page_count=0;
        loop {page_count+=1;assert!(page_count<=5,"page chain exceeds literal five-hit bound");let page=e.exact("farvi",Opt{page_size:Some(1),cursor,..Opt::default()}).unwrap();assert_eq!(page.totals,expected.totals);actual.extend(page.groups.into_iter().flat_map(|g|g.hits.into_iter().map(|h|h.assertion.id)));cursor=page.next_cursor;if cursor.is_none(){break}}
        let ids:Vec<_>=expected.groups.iter().flat_map(|g|g.hits.iter().map(|h|h.assertion.id.clone())).collect();
        ck!("global page concatenation exactly matches full order",actual==ids);
        for d in &ms {
            let id=d.metadata().assertions[0].id.clone();
            let target=RelationTarget::Local(LocalRef::Assertion(id.clone()));
            assert_eq!(e.resolve(&target).unwrap(),baseline.resolve(&target).unwrap());
            assert!(matches!(e.resolve(&RelationTarget::Local(LocalRef::Entry(id))),Err(QE::InvalidReference{..})));
        }
        ck!("all four local assertion refs and wrong role counterparts",true);
    }
    let first=baseline.exact("farvi",Opt{page_size:Some(1),..Opt::default()}).unwrap();
    let cursor=first.next_cursor.unwrap();
    for topology in [T::Attached,T::Fanout] {let e=Engine::open_snapshot(topology,manifest(topology,"generation-one",None)).unwrap();assert!(e.exact("farvi",Opt{page_size:Some(1),cursor:Some(cursor.clone()),..Opt::default()}).is_ok());}
    ck!("cursor crosses topology within same generation",true);
    let mut next_doc=unified.clone().into_document();next_doc.metadata.catalogue_revision+=1;
    let next_dictionary=Dictionary::from_document(next_doc).unwrap();let next_sha=sha(&next_dictionary.to_json_bytes().unwrap());
    let next_path=root.join("generation-two");drop(Store::create(&next_path,&next_dictionary).unwrap());
    let changed=Engine::open_snapshot(T::Unified,Manifest{version:1,generation:"generation-two".into(),logical_state_sha256:next_sha.clone(),members:vec![Member{id:"unified".into(),path:next_path,expected_state_sha256:next_sha}],selected_release_ids:None}).unwrap();
    ck!("old reader retains exact response across distinct new generation path",baseline.exact("farvi",Opt::default()).unwrap()==expected && changed.exact("farvi",Opt::default()).unwrap().catalogue_revision==expected.catalogue_revision+1);
    ck!("generation change rejects old cursor",matches!(changed.exact("farvi",Opt{page_size:Some(1),cursor:Some(cursor.clone()),..Opt::default()}),Err(QE::StaleCursor)));
    ck!("old reader still accepts its own cursor",baseline.exact("farvi",Opt{page_size:Some(1),cursor:Some(cursor.clone()),..Opt::default()}).is_ok());
    let disabled=Engine::open_snapshot(T::Unified,manifest(T::Unified,"generation-one",Some(vec![]))).unwrap();
    ck!("empty runtime selection returns zero without mutating immutable flags",disabled.exact("farvi",Opt::default()).unwrap().totals.occurrences==0&&baseline.exact("farvi",Opt::default()).unwrap()==expected);
    ck!("selection change rejects old cursor",matches!(disabled.exact("farvi",Opt{page_size:Some(1),cursor:Some(cursor),..Opt::default()}),Err(QE::StaleCursor)));
    drop(changed);drop(disabled);
    for topology in [T::Unified,T::Attached,T::Fanout] {
        let mut missing=manifest(topology,"generation-one",None);missing.members[0].path=root.join("never-created");
        ck!("missing member fails without creation",Engine::open_snapshot(topology,missing).is_err()&&!root.join("never-created").exists());
        let mut mismatch=manifest(topology,"generation-one",None);mismatch.members[0].expected_state_sha256="0".repeat(64);
        ck!("wrong expected recorded state fails",Engine::open_snapshot(topology,mismatch).is_err());
        ck!("unknown selected release fails",Engine::open_snapshot(topology,manifest(topology,"generation-one",Some(vec![EntityId::new("cdc-unknown-release").unwrap()]))).is_err());
    }
    drop(baseline);
    let unsupported=root.join("unsupported-version");drop(Store::create(&unsupported,&unified).unwrap());
    let status=std::process::Command::new("sqlite3").arg(unsupported.join("dictionary.sqlite3")).arg("UPDATE metadata SET value='2' WHERE key='dictionary_version';").status().unwrap();assert!(status.success());
    for topology in [T::Unified,T::Attached,T::Fanout] {
        let manifest=Manifest{version:1,generation:"unsupported-test".into(),logical_state_sha256:digest.clone(),members:vec![Member{id:"unified".into(),path:unsupported.clone(),expected_state_sha256:digest.clone()}],selected_release_ids:None};
        let error=match Engine::open_snapshot(topology,manifest){Err(e)=>e,Ok(_)=>panic!("unsupported header version opened")};
        ck!("all providers reject unsupported minimum header version",matches!(&error,QE::Snapshot(_))&&error.to_string().contains("unsupported"));
    }
    let corrupt=root.join("sql-extra-id");drop(Store::create(&corrupt,&unified).unwrap());
    let id=unified.metadata().assertions[0].id.as_str();
    let status=std::process::Command::new("sqlite3").arg(corrupt.join("dictionary.sqlite3")).arg(format!("UPDATE assertions SET id='cdc-absent-draft-assertion' WHERE id='{id}';")).status().unwrap();assert!(status.success());
    let opened=Engine::open(&corrupt).unwrap();
    ck!("returned SQL id absent from draft is typed integrity failure",matches!(opened.exact("farvi",Opt::default()),Err(QE::Integrity{..})));
    drop(opened);
    let status=std::process::Command::new("sqlite3").arg(corrupt.join("dictionary.sqlite3")).arg("DELETE FROM assertions WHERE id='cdc-absent-draft-assertion';").status().unwrap();assert!(status.success());
    let incomplete=Engine::open(&corrupt).unwrap();
    ck!("ordinary query does not claim missing SQL row detection",incomplete.exact("farvi",Opt::default()).unwrap().totals.occurrences==4);drop(incomplete);
    let verified=vlajmaji_store::DictionaryStoreVerification::verify(&corrupt,vlajmaji_store::DictionaryStoreVerificationCheck::All).unwrap();
    ck!("explicit verify rejects missing projection row",!verified.ok && verified.checks.len()==6 && verified.checks.iter().any(|c|c.check=="projection"&&c.status==vlajmaji_store::DictionaryStoreCheckStatus::Failed));
    passed
}

fn emit_fixture_inputs(root:&std::path::Path) {
    use std::io::Write;
    std::fs::create_dir(root).unwrap();
    let models=models();
    let ids=[PackId::LensiskuJbotci,PackId::Llg,PackId::LaLojban,PackId::Sutysisku];
    let write_new=|path:&std::path::Path,bytes:&[u8]|{let mut out=std::fs::OpenOptions::new().write(true).create_new(true).open(path).unwrap();out.write_all(bytes).unwrap();out.flush().unwrap();};
    for (id,dictionary) in ids.into_iter().zip(&models) {
        let pack=seal_release(id,PackPayloadKind::Synthetic,dictionary).unwrap();
        let dir=root.join(id.as_str());std::fs::create_dir(&dir).unwrap();
        write_new(&dir.join("release.json"),&pack.release);write_new(&dir.join("seal.json"),&pack.seal);
    }
    let unified=compose_dictionaries(models).unwrap();let bytes=unified.to_json_bytes().unwrap();
    write_new(&root.join("expected-unified.json"),&bytes);
    println!("{}",serde_json::json!({"status":"PASS","scope":"synthetic sealed input generation only; no store","expected_unified_sha256":sha(&bytes),"root":root}));
}

fn sqlite_diagnostics() {
    let c=rusqlite::Connection::open_in_memory().unwrap();
    let options:Vec<String>=c.prepare("PRAGMA compile_options").unwrap().query_map([],|r|r.get(0)).unwrap().collect::<Result<_,_>>().unwrap();
    let version:String=c.query_row("SELECT sqlite_version()",[],|r|r.get(0)).unwrap();
    let limit=c.limit(rusqlite::limits::Limit::SQLITE_LIMIT_ATTACHED).unwrap();
    println!("{}",serde_json::json!({"scope":"actual Rust-linked SQLite separate in-memory connection; closes before product controls","sqlite_version":version,"compile_options":options,"live_attached_limit":limit}));
}
fn main(){let args:Vec<_>=std::env::args_os().skip(1).collect();if args.first().is_some_and(|x|x=="emit-inputs"){assert_eq!(args.len(),2);emit_fixture_inputs(std::path::Path::new(&args[1]));return;}sqlite_diagnostics();let passed=controls();let snapshots=args.first().map(|p|snapshot_controls(std::path::Path::new(p)));println!("{}",serde_json::json!({"status":"PASS","independent_composition_controls":passed,"snapshot_controls":snapshots}));}

