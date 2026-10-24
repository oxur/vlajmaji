use std::{env, fs, panic::catch_unwind};
use serde_json::{Value, json};
use vlajmaji_lexicon::{Dictionary, DictionaryDocument};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input = fs::read(env::args().nth(1).expect("explicit repaired fixture path"))?;
    let dictionary = Dictionary::from_json_bytes(&input)?;
    let mut base = serde_json::to_value(dictionary.document())?;
    base["legacy_archive"] = Value::Null;
    let wrong = base["metadata"]["computations"][0]["id"].clone();
    let mut failures = Vec::new();
    let mut count = 0;
    let mut reject = |label: &str, mutate: &dyn Fn(&mut Value)| {
        let mut value = base.clone();
        mutate(&mut value);
        // Specimen shape must deserialize: otherwise a typo would fake product rejection.
        let document: DictionaryDocument = serde_json::from_value(value.clone()).unwrap();
        let bytes = serde_json::to_vec(&value).unwrap();
        let ctor = catch_unwind(|| Dictionary::from_document(document)).is_ok_and(|r| r.is_err());
        let helper = catch_unwind(|| Dictionary::from_json_bytes(&bytes)).is_ok_and(|r| r.is_err());
        let direct = catch_unwind(|| serde_json::from_slice::<Dictionary>(&bytes)).is_ok_and(|r| r.is_err());
        let pass = ctor && helper && direct;
        count += 1;
        println!("{label}={pass}");
        if !pass { failures.push(label.to_owned()); }
    };
    reject("unreferenced_component_wrong_collection_role", &|d| {
        let mut c = d["metadata"]["components"][0].clone();
        c["id"] = json!("cdc:supplement-component"); c["collection"] = wrong.clone();
        d["metadata"]["components"].as_array_mut().unwrap().push(c);
    });
    reject("release_component_wrong_role", &|d| d["metadata"]["releases"][0]["component"] = wrong.clone());
    reject("assertion_entry_wrong_role", &|d| d["metadata"]["assertions"][0]["entry"] = wrong.clone());
    reject("frame_assertion_wrong_role", &|d| d["metadata"]["frames"][0]["assertion"] = wrong.clone());
    reject("form_entry_wrong_role", &|d| d["metadata"]["forms"][0]["entry"] = wrong.clone());
    reject("form_computation_wrong_role", &|d| {
        let id = d["metadata"]["entries"][0]["id"].clone();
        let form = d["metadata"]["forms"].as_array_mut().unwrap().iter_mut().find(|f| !f["computation"].is_null()).unwrap();
        form["computation"] = id;
    });
    reject("etymology_assertion_wrong_role", &|d| d["metadata"]["etymologies"][0]["assertion"] = wrong.clone());
    reject("relation_origin_wrong_role", &|d| d["metadata"]["relations"][0]["origin"] = json!({"Assertion":wrong}));
    reject("relation_local_target_wrong_role", &|d| d["metadata"]["relations"][0]["target"] = json!({"Local":{"Entry":wrong}}));
    reject("editorial_relation_revision_wrong_role", &|d| {
        let r = d["metadata"]["relations"].as_array_mut().unwrap().iter_mut().find(|r| r["kind"]==json!("EditorialLink")).unwrap();
        r["editorial_revision"] = wrong.clone();
    });
    reject("annotation_computation_wrong_role", &|d| d["metadata"]["annotations"][0]["computation"] = d["metadata"]["entries"][0]["id"].clone());
    reject("extension_unknown_owner", &|d| d["metadata"]["extensions"][0]["owner"] = json!("cdc:missing-owner"));
    reject("extension_loaded_phantom_evidence", &|d| {
        let mut evidence=d["metadata"]["contexts"][0]["evidence"].clone();
        evidence["occurrence"]["locator"]=json!("/supplement-phantom");
        d["metadata"]["extensions"][0]["evidence"]=evidence;
    });
    reject("missing_release_diagnostic_loaded_phantom", &|d| {
        let mut occurrence=d["metadata"]["assertions"][0]["occurrence"].clone();
        occurrence["locator"]=json!("/supplement-diagnostic-phantom");
        let r=d["metadata"]["releases"].as_array_mut().unwrap().iter_mut().find(|r| r["availability"]==json!("Missing")).unwrap();
        r["diagnostics"].as_array_mut().unwrap().push(json!({"occurrence":occurrence,"code":"synthetic","message":"negative reference control"}));
    });
    reject("catalogue_wrong_source_release", &|d| {
        let r=d["metadata"]["relations"].as_array_mut().unwrap().iter_mut().find(|r| r["target"].get("Catalog").is_some()).unwrap();
        r["target"]["Catalog"]["source"]=json!("source-b");
    });
    reject("catalogue_unknown_release", &|d| {
        let r=d["metadata"]["relations"].as_array_mut().unwrap().iter_mut().find(|r| r["target"].get("Catalog").is_some()).unwrap();
        r["target"]["Catalog"]["release"]=json!("cdc:unknown-release");
    });
    reject("owned_head_wrong_source", &|d| d["metadata"]["owned_heads"][0]["source"]=json!("source-a"));
    reject("editorial_assertion_wrong_role", &|d| d["metadata"]["editorial_senses"][0]["assertions"][0]=wrong.clone());
    reject("editorial_revision_wrong_role", &|d| d["metadata"]["editorial_senses"][0]["revision"]=wrong.clone());
    reject("owned_assertion_wrong_role", &|d| d["metadata"]["owned_revisions"][0]["assertions"][0]=wrong.clone());
    reject("duplicate_frame_slot_order", &|d| {
        let s=d["metadata"]["frames"][0]["slots"][0].clone();
        d["metadata"]["frames"][0]["slots"].as_array_mut().unwrap().push(s);
    });
    reject("duplicate_etymology_component_order", &|d| {
        d["metadata"]["etymologies"][0]["components"]=json!([
            {"order":1,"spelling":"synthetic-a","target":null},
            {"order":1,"spelling":"synthetic-b","target":null}]);
    });
    drop(reject);
    // Optional absent snapshot binding is not required by the binding model contract.
    let mut absent=base.clone();
    absent["registry"]["snapshots"].as_array_mut().unwrap().retain(|s| s["source"]!=json!("source-m"));
    let absent_bytes=serde_json::to_vec(&absent)?;
    let absent_document:DictionaryDocument=serde_json::from_value(absent)?;
    let valid=Dictionary::from_document(absent_document).is_ok()
        && Dictionary::from_json_bytes(&absent_bytes).is_ok()
        && serde_json::from_slice::<Dictionary>(&absent_bytes).is_ok();
    count+=1;println!("known_unavailable_capture_without_optional_binding={valid}");
    if !valid { failures.push("known_unavailable_capture_without_optional_binding".into()); }
    println!("supplement_controls={count} failures={} {failures:?}",failures.len());
    if failures.is_empty() { Ok(()) } else { Err("supplementary reference controls failed".into()) }
}
