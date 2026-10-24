use std::{env,fs,error::Error};
use vlajmaji_search::{DictionarySearchEngine,DictionaryRequest,DictionaryQueryError};
fn error(e:DictionaryQueryError)->! {
 let variant=match &e {
  DictionaryQueryError::Store(inner)=>{assert_eq!(e.source().unwrap().to_string(),inner.to_string());"Store"},
  DictionaryQueryError::Dictionary(inner)=>{assert_eq!(e.source().unwrap().to_string(),inner.to_string());"Dictionary"},
  DictionaryQueryError::Registry(inner)=>{assert_eq!(e.source().unwrap().to_string(),inner.to_string());"Registry"},
  DictionaryQueryError::UnavailableSource{..}=>"UnavailableSource",
  DictionaryQueryError::MalformedCursor{..}=>"MalformedCursor",
  DictionaryQueryError::StaleCursor=>"StaleCursor",
  DictionaryQueryError::InvalidCursorPosition=>"InvalidCursorPosition",
  DictionaryQueryError::InvalidReference{..}=>"InvalidReference",
  DictionaryQueryError::Integrity{..}=>"Integrity",_=>"FutureVariant"
 };
 println!("{}",serde_json::json!({"variant":variant,"error":e.report()}));std::process::exit(2)
}
fn main(){
 let args:Vec<_>=env::args().collect();
 if args[1]=="create" {
  let bytes=fs::read(&args[2]).unwrap();let engine=DictionarySearchEngine::build(&args[3],&bytes).unwrap_or_else(|e|error(e));
  let first=engine.exact("farvi",Default::default()).unwrap_or_else(|e|error(e));let hash=engine.state_sha256().to_owned();let counts=engine.dictionary_counts();drop(engine);
  let second=DictionarySearchEngine::open(&args[3]).unwrap_or_else(|e|error(e));assert_eq!(first,second.exact("farvi",Default::default()).unwrap());assert_eq!(hash,second.state_sha256());assert_eq!(counts,second.dictionary_counts());
  println!("{}",serde_json::json!({"counts":[counts.0,counts.1,counts.2,counts.3],"state_sha256":hash}));return
 }
 let request:DictionaryRequest=serde_json::from_str(&args[3]).unwrap();let engine=DictionarySearchEngine::open(&args[2]).unwrap_or_else(|e|error(e));
 let value=match request {
  DictionaryRequest::Exact{spelling,options}=>serde_json::to_value(engine.exact(&spelling,options).unwrap_or_else(|e|error(e))).unwrap(),
  DictionaryRequest::Resolve{target}=>serde_json::to_value(engine.resolve(&target).unwrap_or_else(|e|error(e))).unwrap(),
  _=>panic!("unexpected future request")
 };println!("{}",value);
}
