use vlajmaji_lexicon::*;
use sha2::{Digest,Sha256};
fn hash(b:&[u8])->String {format!("{:x}",Sha256::digest(b))}
fn model(pattern:&[u8])->Dictionary {
 let raw=br#"[{"word":"farvi","word_type":"gismu","definition":"synthetic definition"}]"#;
 let meta=format!("language_tag = \"en\"\nlanguage_realname = \"English\"\nsource_language_tag = \"jbo\"\nformat = \"json\"\npositive_scores_only = false\nfilename = \"dictionary-en.json\"\nmetadata_url = \"https://example.invalid/languages\"\ndownload_url = \"https://example.invalid/export\"\nlensisku_created_at = \"2026-10-04T03:07:08Z\"\nsha256 = \"{}\"\ndefinition_count = 1\nentry_count = 1\n",hash(raw));
 let options=JbotciImportOptions{revision:"0".repeat(40),expected_raw_sha256:hash(raw),expected_metadata_sha256:hash(meta.as_bytes())};
 let mut doc=import_jbotci_export(raw,meta.as_bytes(),&options).expect("valid original fixture").into_dictionary().into_document();
 let owner=doc.metadata.assertions[0].id.clone();
 doc.metadata.extensions=pattern.iter().map(|n|ExtensionRecord{owner:owner.clone(),name:"same-native-field".into(),value:RawJson::new(n.to_string()).unwrap(),evidence:None}).collect();
 Dictionary::from_document(doc).expect("complete repeated extension model is valid")
}
fn main(){
 let expect_panic=std::env::args().nth(1).as_deref()==Some("expect-panic");let mut panics=0;let mut errors=0;let mut passed=0;
 for (name,pattern,twice) in [("distinct",vec![1,2],false),("one-repeat",vec![1,1],false),("repeat-before-new-repeat",vec![1,1,2,2],false),("repeat-before-new-self",vec![1,1,2],true)] {
  let input=model(&pattern);let expected=input.metadata().extensions.clone();let inputs=if twice {vec![input.clone(),input]}else{vec![input]};
  match std::panic::catch_unwind(||compose_dictionaries(inputs)) {
   Ok(Ok(output))=>{assert_eq!(output.metadata().extensions,expected);passed+=1;println!("{name}: VALID_INPUT_COMPOSED_EXACTLY");},
   Ok(Err(e))=>{errors+=1;println!("{name}: UNEXPECTED_ERROR {e}");},
   Err(_)=>{panics+=1;println!("{name}: PANIC_ON_VALID_INPUT");}
  }
 }
 println!("passed={passed} panics={panics} unexpected_errors={errors}");
 if expect_panic {assert_eq!((passed,panics,errors),(2,2,0));println!("candidate_verdict=REJECTED");}else{assert_eq!((passed,panics,errors),(4,0,0));println!("candidate_verdict=PASS");}
}
