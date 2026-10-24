use std::{env,fs,panic::catch_unwind};
use vlajmaji_lexicon::*;
fn main()->Result<(),Box<dyn std::error::Error>> {
 let mut base=Dictionary::from_json_bytes(&fs::read(env::args().nth(1).expect("fresh fixture path"))?)?.document();
 base.legacy_archive=None;
 let mut n=0;let mut failed=Vec::new();
 let mut check=|label:&str,ok:bool| {n+=1;println!("{label}={ok}");if !ok{failed.push(label.to_owned());}};
 // OrderedComponent permits the declared LocalRef roles. These fresh synthetic
 // drafts test structural role validity, not a new claim about source etymology.
 for (label,target) in [
  ("etymology_valid_assertion_component",LocalRef::Assertion(base.metadata.assertions[0].id.clone())),
  ("etymology_valid_form_component",LocalRef::Form(base.metadata.forms[0].id.clone())),
  ("etymology_valid_editorial_component",LocalRef::EditorialSense(base.metadata.editorial_senses[0].id.clone()))
 ] {
  let mut d=base.clone();d.metadata.etymologies[0].components.push(OrderedComponent{order:1,spelling:"explicit synthetic component".into(),target:Some(target)});
  let bytes=serde_json::to_vec(&d)?;
  let ctor=catch_unwind(||Dictionary::from_document(d.clone())).is_ok_and(|r|r.is_ok());
  let helper=catch_unwind(||Dictionary::from_json_bytes(&bytes)).is_ok_and(|r|r.is_ok());
  let direct=catch_unwind(||serde_json::from_slice::<Dictionary>(&bytes)).is_ok_and(|r|r.is_ok());
  check(label,ctor&&helper&&direct);
 }
 let mut typed=|label:&str,edit:&dyn Fn(&mut DictionaryDocument),category:&str| {
  let mut d=base.clone();edit(&mut d);let bytes=serde_json::to_vec(&d).unwrap();
  let ctor=Dictionary::from_document(d);let helper=Dictionary::from_json_bytes(&bytes);
  println!("{label}_constructor={ctor:?}; helper={helper:?}");
  let expected=|r:Result<Dictionary,DictionaryError>|match (category,r){
   ("reference",Err(DictionaryError::MissingReference(_)))=>true,
   ("provenance",Err(DictionaryError::InvalidProvenance(_)))=>true,
   _=>false};
  check(label,expected(ctor)&&expected(helper));
 };
 typed("form_entry_named_reference",&|d|d.metadata.forms[0].entry=EntityId::new("cdc:missing-entry").unwrap(),"reference");
 typed("entry_relation_origin_named_reference",&|d|d.metadata.relations[0].origin=LocalRef::Entry(EntityId::new("cdc:missing-entry").unwrap()),"reference");
 typed("source_form_foreign_evidence_named_provenance",&|d|{
  let other=d.metadata.assertions.iter().find(|a|a.occurrence.capture.namespace=="captured:b:d1").unwrap().occurrence.clone();
  let f=d.metadata.forms.iter_mut().find(|f|f.evidence.is_some()).unwrap();
  f.evidence=Some(SourceEvidence{occurrence:other,field_index:None,citation:Presence::Missing,offset:None});
 },"provenance");
 typed("computation_missing_tool_named_provenance",&|d|d.metadata.computations[0].tool.clear(),"provenance");
 typed("loaded_phantom_evidence_named_reference",&|d|d.metadata.extensions[0].evidence=Some(SourceEvidence{occurrence:OccurrenceRef{capture:d.metadata.assertions[0].occurrence.capture.clone(),locator:"/cdc-phantom".into()},field_index:None,citation:Presence::Missing,offset:None}),"reference");
 typed("loaded_phantom_assertion_named_reference",&|d|d.metadata.assertions[0].occurrence.locator="/cdc-phantom".into(),"reference");
 typed("loaded_capture_missing_binding_named_reference",&|d|{
  let mut r=d.registry.document();r.snapshots.retain(|s|s.snapshot.namespace()!="captured:b:d1");
  d.registry=SourceRegistry::from_document(r).unwrap();
 },"reference");
 drop(typed);
 println!("additional_controls={n} failures={} {failed:?}",failed.len());
 if failed.is_empty(){Ok(())}else{Err("additional controls failed".into())}
}
