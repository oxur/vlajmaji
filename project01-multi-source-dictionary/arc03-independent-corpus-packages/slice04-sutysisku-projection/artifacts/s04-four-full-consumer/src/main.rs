use std::collections::{BTreeMap, BTreeSet};

use vlajmaji_lexicon::{Eligibility, Presence, compose_dictionaries};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let direct = [
        vlajmaji_data_lensisku_jbotci::load()?,
        vlajmaji_data_llg::load()?,
        vlajmaji_data_la_lojban::load()?,
        vlajmaji_data_sutysisku::load()?,
    ];
    assert_eq!(
        vlajmaji_data_lensisku_jbotci::release_bytes(),
        vlajmaji::data_lensisku_jbotci::release_bytes()
    );
    assert_eq!(
        vlajmaji_data_llg::release_bytes(),
        vlajmaji::data_llg::release_bytes()
    );
    assert_eq!(
        vlajmaji_data_la_lojban::release_bytes(),
        vlajmaji::data_la_lojban::release_bytes()
    );
    assert_eq!(
        vlajmaji_data_sutysisku::release_bytes(),
        vlajmaji::data_sutysisku::release_bytes()
    );
    for package in [
        vlajmaji_data_lensisku_jbotci::PACK_ID,
        vlajmaji_data_llg::PACK_ID,
        vlajmaji_data_la_lojban::PACK_ID,
        vlajmaji_data_sutysisku::PACK_ID,
    ] {
        assert!(package.as_str().len() > 2);
    }
    let composed = compose_dictionaries(direct.to_vec())?;
    let metadata = composed.metadata();
    assert_eq!(metadata.assertions.len(), 78_361);
    assert_eq!(metadata.contexts.len(), 177_210);
    assert_eq!(metadata.forms.len(), 11_598);
    assert_eq!(metadata.etymologies.len(), 3_513);
    assert_eq!(metadata.extensions.len(), 138_107);
    let native_fields: usize = composed
        .captures()
        .iter()
        .flat_map(|capture| &capture.occurrences)
        .map(|occurrence| occurrence.fields.len())
        .sum();
    assert_eq!(native_fields, 566_367);
    let entries: BTreeMap<_, _> = metadata.entries.iter().map(|entry| (entry.id.clone(), entry)).collect();
    let eligible_entry_ids: BTreeSet<_> = metadata
        .assertions
        .iter()
        .filter(|assertion| assertion.eligibility == Eligibility::Lexical)
        .map(|assertion| assertion.entry.clone())
        .collect();
    let eligible_spellings: BTreeSet<_> = eligible_entry_ids
        .iter()
        .filter_map(|id| entries.get(id))
        .filter_map(|entry| match &entry.spelling {
            Presence::Value(spelling) => Some(spelling.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(eligible_spellings.len(), 31_375);
    println!(
        "direct+umbrella bytes equal; loaded four full releases; composed assertions={} eligible_words={} native_fields={} contexts={} forms={} etymologies={} extensions={}",
        metadata.assertions.len(), eligible_spellings.len(), native_fields,
        metadata.contexts.len(), metadata.forms.len(), metadata.etymologies.len(),
        metadata.extensions.len()
    );
    Ok(())
}
