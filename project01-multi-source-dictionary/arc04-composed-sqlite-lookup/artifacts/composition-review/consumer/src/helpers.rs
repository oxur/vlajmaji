use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use vlajmaji_lexicon::*;

fn sha(b: &[u8]) -> String { format!("{:x}", Sha256::digest(b)) }
const LENS: &[u8] = include_bytes!("../fixtures/lensisku.json");
const SUT: &[u8] = include_bytes!("../fixtures/sutysisku.json");
const LLG: &[u8] = include_bytes!("../fixtures/llg-valid-row.txt");
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
