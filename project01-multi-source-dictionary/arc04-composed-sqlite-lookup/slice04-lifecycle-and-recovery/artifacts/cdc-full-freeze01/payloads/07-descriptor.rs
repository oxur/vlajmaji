use std::collections::BTreeSet;
fn value<T:serde::Serialize>(v:&T)->Value {serde_json::to_value(v).unwrap()}
fn digest<T:serde::Serialize>(v:&T)->String {sha(&serde_json::to_vec(v).unwrap())}
fn fp(v:&Value)->String {digest(v)}
fn expected(d:&UnverifiedDictionaryDocument)->Value {
    let m=&d.metadata;
    let entries:BTreeMap<_,_>=m.entries.iter().map(|e|(&e.id,e)).collect();
    let assertions:BTreeMap<_,_>=m.assertions.iter().map(|a|(&a.id,a)).collect();
    let mut entities=BTreeMap::<String,Value>::new();let mut bodies=BTreeMap::<String,String>::new();
    for e in &m.entries {entities.insert(format!("entry:{}",e.id),json!({"role":"entry","id":e.id,"entry_ids":[],"assertion_ids":[],"source_ids":[e.source],"occurrence":null,"evidence":null,"spelling":e.spelling}));bodies.insert(format!("entry:{}",e.id),digest(e));}
    for a in &m.assertions {entities.insert(format!("assertion:{}",a.id),json!({"role":"assertion","id":a.id,"entry_ids":[a.entry],"assertion_ids":[],"source_ids":[entries[&a.entry].source],"occurrence":a.occurrence,"evidence":null,"spelling":null}));bodies.insert(format!("assertion:{}",a.id),digest(a));}
    for f in &m.forms {entities.insert(format!("form:{}",f.id),json!({"role":"form","id":f.id,"entry_ids":[f.entry],"assertion_ids":[],"source_ids":[entries[&f.entry].source],"occurrence":f.evidence.as_ref().map(|e|&e.occurrence),"evidence":f.evidence,"spelling":null}));bodies.insert(format!("form:{}",f.id),digest(f));}
    for s in &m.editorial_senses {let es:BTreeSet<_>=s.assertions.iter().map(|a|assertions[a].entry.clone()).collect();let ss:BTreeSet<_>=es.iter().map(|e|entries[e].source.clone()).collect();entities.insert(format!("editorial_sense:{}",s.id),json!({"role":"editorial_sense","id":s.id,"entry_ids":es,"assertion_ids":s.assertions,"source_ids":ss,"occurrence":null,"evidence":null,"spelling":null}));bodies.insert(format!("editorial_sense:{}",s.id),digest(s));}
    macro_rules! body {($tag:literal,$field:ident)=>{for r in &m.$field {assert!(bodies.insert(format!("{}:{}",$tag,r.id),digest(r)).is_none());}}}
    body!("context",contexts);body!("frame",frames);body!("etymology",etymologies);body!("relation",relations);body!("owned_revision",owned_revisions);body!("computation",computations);body!("annotation",annotations);
    for h in &m.owned_heads {bodies.insert(format!("owned_head:{}",h.source),digest(h));}
    let captures:Vec<_>=d.captures.iter().map(|c|{let occurrences:Vec<_>=c.occurrences.iter().map(|o|OccurrenceRef{capture:c.identity.clone(),locator:o.locator.clone()}).collect();json!({"capture":c.identity,"metadata":c.metadata,"byte_length":c.bytes.len(),"occurrences":occurrences})}).collect();
    let mut counts=BTreeMap::new();macro_rules! count {($($f:ident),*)=>{$(counts.insert(stringify!($f),m.$f.len());)*}}
    count!(entries,assertions,contexts,frames,forms,etymologies,relations,owned_revisions,owned_heads,editorial_senses,computations,annotations,extensions);
    json!({"registry":fp(&value(&d.registry)),"source_metadata":fp(&json!({"collections":m.collections,"components":m.components,"releases":m.releases,"code_notices":m.code_notices})),"captures":fp(&value(&captures)),"entities":fp(&value(&entities)),"body_sha256":fp(&value(&bodies)),"extension_row_sha256_ordered_multiset":fp(&value(&m.extensions.iter().map(digest).collect::<Vec<_>>())),"category_counts":counts,"dictionary_version":d.version,"registry_version":d.registry.version,"upstream_catalogue_revision":m.catalogue_revision})
}
fn actual(d:&vlajmaji_store::CollectionReferenceMetadata)->Value {
    let entities:BTreeMap<_,_>=d.entities.iter().map(|e|(format!("{}:{}",e.role,e.id),value(e))).collect();assert_eq!(entities.len(),d.entities.len());
    json!({"registry":fp(&value(&d.registry)),"source_metadata":fp(&value(&d.source_metadata)),"captures":fp(&value(&d.captures)),"entities":fp(&value(&entities)),"body_sha256":fp(&value(&d.body_sha256)),"extension_row_sha256_ordered_multiset":fp(&value(&d.extension_row_sha256_ordered_multiset)),"category_counts":d.category_counts,"dictionary_version":d.dictionary_version,"registry_version":d.registry_version,"upstream_catalogue_revision":d.upstream_catalogue_revision})
}
