use sha2::{Digest,Sha256};
use vlajmaji::lexicon::{PackSeal,PackPayloadKind,Eligibility,compose_dictionaries};
fn main()->Result<(),Box<dyn std::error::Error>> {
 let raw=vlajmaji_data_lensisku_jbotci::release_bytes();let seal=vlajmaji_data_lensisku_jbotci::seal_bytes();
 assert_eq!(raw,vlajmaji::data_lensisku_jbotci::release_bytes());assert_eq!(seal,vlajmaji::data_lensisku_jbotci::seal_bytes());
 let sha=format!("{:x}",Sha256::digest(raw));assert_eq!(raw.len(),215412497);assert_eq!(sha,"0987ec84a8434573b5979ea854ca459ce2ea9c7fa03ebac6639574dd2b7aa460");
 let parsed=PackSeal::from_json_bytes(seal)?;assert_eq!(parsed.payload_kind,PackPayloadKind::Full);println!("embedded_full_bytes_verified");
 let model=vlajmaji::data_lensisku_jbotci::load()?;let m=model.metadata();let lexical=m.assertions.iter().filter(|x|matches!(x.eligibility,Eligibility::Lexical)).count();
 assert_eq!((m.entries.len(),m.assertions.len(),lexical,m.contexts.len(),m.forms.len(),m.etymologies.len(),m.extensions.len()),(33526,33526,33516,76357,4008,2085,73226));
 assert_eq!(model.registry().as_document().sources.len(),1);assert_eq!(m.releases.len(),1);assert_eq!(model.captures().len(),2);assert_eq!(model.captures()[0].bytes.len(),17163482);assert_eq!(model.captures()[1].bytes.len(),514);println!("strict_umbrella_load_with_direct_same_embedded_route_verified");
 let composed=compose_dictionaries(vec![model])?;let result=composed.to_json_bytes()?;assert_eq!(result.as_slice(),raw);
 println!("{}",serde_json::json!({"verdict":"PASS","canonical_bytes":result.len(),"canonical_sha256":sha,"sources":1,"releases":1,"captures":2,"entries":33526,"assertions":33526,"lexical":33516,"contexts":76357,"forms":4008,"etymologies":2085,"extensions":73226,"current_consuming_composition_exact":true}));Ok(())
}
