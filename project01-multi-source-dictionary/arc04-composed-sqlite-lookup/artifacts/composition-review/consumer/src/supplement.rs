// Prospective SOURCE ONLY: original supplemental model is independent of the candidate.
// Compile/execute only after frozen handback and full caller review.
#[path = "complete_fixture.rs"]
mod fixture;

fn supplement(base: &Path) {
    let model = fixture::complete_fixture().expect("accepted complete fixture remains strictly valid");
    let input = draft(model.clone());
    assert_eq!(input.metadata.forms.len(), 7);
    assert_eq!(input.metadata.editorial_senses.len(), 1);
    assert_eq!(input.metadata.contexts.len(), 8);
    let store_path = base.join("complete-fixture-store");
    drop(vlajmaji_store::DictionaryStore::create(&store_path, &model).unwrap());
    drop(model);
    let compact = DictionaryCompactSearchEngine::open(&store_path).unwrap();
    let mut observed_forms = std::collections::BTreeSet::new();
    let mut observed_senses = std::collections::BTreeSet::new();
    let mut observed_contexts = std::collections::BTreeSet::new();
    for spelling in ["si'e", "farvi", "cpana", "snime", "sie"] {
        let response = compact.exact(spelling, DictionaryExactOptions::default()).unwrap();
        for projected in response.groups.into_iter().flat_map(|g| g.hits) {
            let expected_key = key(&input, &projected.assertion);
            assert_eq!(projected.detail_ref, expected_key);
            let detail = compact.detail(&expected_key, None).unwrap();
            assert_eq!(detail.status, ReferenceStatus::Resolved);
            let hit = detail.hit.expect("complete detail must carry its hit");
            let assertion = input.metadata.assertions.iter().find(|a| a.id == expected_key.assertion_id).unwrap();
            let entry = input.metadata.entries.iter().find(|e| e.id == assertion.entry).unwrap();
            assert_eq!(&hit.entry, entry);
            assert_eq!(&hit.assertion, assertion);
            assert_eq!(hit.occurrence_id, tuple_id(&assertion.occurrence));
            let contexts: Vec<_> = input.metadata.contexts.iter().filter(|c| c.assertion == assertion.id).cloned().collect();
            let forms: Vec<_> = input.metadata.forms.iter().filter(|f| f.entry == entry.id).cloned().collect();
            let frames: Vec<_> = input.metadata.frames.iter().filter(|f| f.assertion == assertion.id).cloned().collect();
            let etymologies: Vec<_> = input.metadata.etymologies.iter().filter(|e| e.assertion == assertion.id).cloned().collect();
            let senses: Vec<_> = input.metadata.editorial_senses.iter().filter(|s| s.assertions.contains(&assertion.id)).cloned().collect();
            assert_eq!(hit.contexts, contexts);
            assert_eq!(hit.forms, forms);
            assert_eq!(hit.frames, frames);
            assert_eq!(hit.etymologies, etymologies);
            assert_eq!(hit.editorial_senses, senses);
            observed_forms.extend(hit.forms.iter().map(|f| f.id.to_string()));
            observed_senses.extend(hit.editorial_senses.iter().map(|s| s.id.to_string()));
            observed_contexts.extend(hit.contexts.iter().map(|c| c.id.to_string()));
            let capture = input.captures.iter().find(|c| c.identity == assertion.occurrence.capture).unwrap();
            let raw = capture.occurrences.iter().find(|r| r.locator == assertion.occurrence.locator).unwrap();
            assert_eq!(hit.capture, capture.metadata);
            assert_eq!(&hit.raw, raw);
        }
    }
    assert_eq!(observed_forms.len(), 7);
    assert_eq!(observed_senses.len(), 1);
    assert_eq!(observed_contexts.len(), 8);
    println!("PASS supplemental original-input detail fields: all7 Forms/1 editorial sense/8 contexts, raw/presence/frame/etymology");
}
