use serde::de::{Deserializer,MapAccess,Visitor,Error};
use serde_json::value::RawValue;
struct Root;
impl<'de> Visitor<'de> for Root {
 type Value=Vec<(String,String,String)>;
 fn expecting(&self,f:&mut std::fmt::Formatter)->std::fmt::Result {f.write_str("ordered root object")}
 fn visit_map<M:MapAccess<'de>>(self,mut m:M)->Result<Self::Value,M::Error>{
 let mut rows=Vec::new();
 while let Some(key)=m.next_key::<Box<RawValue>>()? {let word:String=serde_json::from_str(key.get()).map_err(M::Error::custom)?;let body=m.next_value::<Box<RawValue>>()?;rows.push((key.get().into(),word,body.get().into()));}
 Ok(rows)
 }
}
fn main(){
 let raw=r#"{ "\u0066arvi":{"d":"one","t":"gismu"}, "farvi" : { "d": "two", "t":"gismu" }, ".a":{"d":"control"} }"#;
 let mut d=serde_json::Deserializer::from_str(raw);let rows=d.deserialize_map(Root).unwrap();d.end().unwrap();
 assert_eq!(rows,vec![(r#""\u0066arvi""#.into(),"farvi".into(),r#"{"d":"one","t":"gismu"}"#.into()),(r#""farvi""#.into(),"farvi".into(),r#"{ "d": "two", "t":"gismu" }"#.into()),(r#"".a""#.into(),".a".into(),r#"{"d":"control"}"#.into())]);
 println!("PASS pinned serde MapKey/RawValue preserves3ordered root occurrences, exact escaped-key lexeme and both repeated decoded farvi keys, exact value whitespace; no map collapse");
}
