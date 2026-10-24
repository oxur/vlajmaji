use std::{env, fs, io, path::Path};
use vlajmaji_lexicon::Dictionary;
use vlajmaji_store::{DictionaryStore, DictionaryStoreError};

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
