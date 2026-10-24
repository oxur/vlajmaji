use std::{env, fs, io, path::Path};
use vlajmaji_lexicon::{Dictionary, SourceRegistry};
use vlajmaji_store::{DictionaryStore, DictionaryStoreError, LexicalStore, SqliteStore};

fn outcome<T>(r: Result<T, DictionaryStoreError>) -> Option<T> {
    match r {
        Ok(v) => Some(v),
        Err(e) => {
            match &e {
                DictionaryStoreError::ExistingPath(_) => println!("ERR ExistingPath"),
                DictionaryStoreError::InvalidDictionary(_) => println!("ERR InvalidDictionary"),
                DictionaryStoreError::InvalidStore { reason, .. } => println!("ERR InvalidStore {reason:?}"),
                DictionaryStoreError::Backend(_) => println!("ERR Backend"),
                DictionaryStoreError::Io(_) => println!("ERR Io"),
                _ => println!("ERR FutureVariant"),
            }
            None
        }
    }
}
fn main() {
    let args: Vec<String> = env::args().collect();
    match args[1].as_str() {
        "create" => {
            let bytes=fs::read(&args[2]).unwrap();
            let dictionary=match Dictionary::from_json_bytes(&bytes) {
                Ok(v)=>v, Err(e)=>{println!("MODELERR {e:?}");return;}
            };
            let Some(store)=outcome(DictionaryStore::create(&args[3], &dictionary)) else {return;};
            assert_eq!(store.dictionary().document(),dictionary.document());
            assert_eq!(store.canonical_bytes(),dictionary.to_json_bytes().unwrap());
            assert_eq!(store.path(),Path::new(&args[3]));
            assert_eq!(store.database_path(),Path::new(&args[3]).join("dictionary.sqlite3"));
            drop(store);
            let Some(store)=outcome(DictionaryStore::open(&args[3])) else {return;};
            assert_eq!(store.dictionary().document(),dictionary.document());
            assert_eq!(store.canonical_bytes(),dictionary.to_json_bytes().unwrap());
            fs::write(Path::new(&args[3]).with_extension("canonical.json"),store.canonical_bytes()).unwrap();
            println!("OK CREATE {}",store.canonical_bytes().len());
        },
        "legacy" => {
            let dictionary=Dictionary::from_json_bytes(&fs::read(&args[2]).unwrap()).unwrap();
            let mut doc=dictionary.document().clone();
            let archive=doc.legacy_archive.clone().unwrap();
            let excluded=doc.metadata.assertions.iter().find(|a|a.id.as_str()=="assertion-19").unwrap().entry.clone();
            doc.captures.retain(|c|c.identity.namespace!="captured:a:opaque");
            let mut registry=doc.registry.document();
            registry.snapshots.retain(|s|s.snapshot.namespace()!="captured:a:opaque");
            registry.occurrences.retain(|o|o.occurrence.snapshot().namespace()!="captured:a:opaque");
            doc.registry=SourceRegistry::from_document(registry).unwrap();
            doc.metadata.assertions.retain(|a|a.id.as_str()!="assertion-19");
            doc.metadata.entries.retain(|e|e.id!=excluded);
            let release=doc.metadata.releases.iter_mut().find(|r|r.id.as_str()=="release-a").unwrap();
            release.captures.retain(|c|c.namespace!="captured:a:opaque");
            release.assertions.retain(|a|a.as_str()!="assertion-19");
            release.diagnostics.retain(|d|d.code!="opaque-excluded");
            release.accounting.input_occurrences=Some(16);
            release.accounting.normalized_occurrences=Some(16);
            release.accounting.query_excluded_occurrences=Some(0);
            let subset=Dictionary::from_document(doc.clone()).unwrap();
            let root=Path::new(&args[3]);fs::create_dir(root).unwrap();
            let old=SqliteStore::create(root.join("old-schema1"),&archive).unwrap();
            let extracted=old.archive_bytes().unwrap();assert_eq!(extracted,archive);drop(old);
            let old_file=root.join("old-schema1/index.sqlite3");let old_bytes=fs::read(&old_file).unwrap();
            let rebuilt=Dictionary::from_legacy_archive(&extracted,doc.registry,doc.metadata).unwrap();
            assert_eq!(rebuilt.document(),subset.document());
            assert_eq!(rebuilt.document().captures.len(),8);
            assert_eq!(rebuilt.document().metadata.assertions.len(),19);
            let store=DictionaryStore::create(root.join("new-dictionary"),&rebuilt).unwrap();drop(store);
            let store=DictionaryStore::open(root.join("new-dictionary")).unwrap();
            assert_eq!(store.dictionary().document(),rebuilt.document());
            assert_eq!(store.dictionary().document().legacy_archive.as_deref(),Some(archive.as_slice()));
            let old=SqliteStore::open(root.join("old-schema1")).unwrap();assert_eq!(old.archive_bytes().unwrap(),archive);drop(old);
            assert_eq!(fs::read(&old_file).unwrap(),old_bytes);
            println!("OK LEGACY 8 captures 19 assertions unchanged schema1/archive");
        },
        "open" => {
            if let Some(store)=outcome(DictionaryStore::open(&args[2])) {
                assert_eq!(store.canonical_bytes(),store.dictionary().to_json_bytes().unwrap());
                println!("OK OPEN {}",store.canonical_bytes().len());
            }
        },
        "hold" => {
            if let Some(store)=outcome(DictionaryStore::open(&args[2])) {
                println!("OK HOLD {}",store.canonical_bytes().len());
                use std::io::Write;
                io::stdout().flush().unwrap();
                let mut s=String::new();io::stdin().read_line(&mut s).unwrap();
                assert_eq!(store.canonical_bytes(),store.dictionary().to_json_bytes().unwrap());
            }
        },
        _=>panic!("unknown control mode"),
    }
}
