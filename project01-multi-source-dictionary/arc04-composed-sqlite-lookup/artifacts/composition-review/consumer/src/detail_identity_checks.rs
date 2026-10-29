// Prospective SOURCE ONLY: negatives retain the exact original valid counterpart.
fn detail_identity_checks(root: &Path, store: &Path, docs: &[UnverifiedDictionaryDocument]) {
    let original = key(&docs[0], &docs[0].metadata.assertions[0]);
    assert!(original.native_entry_id.is_none());
    assert_eq!(original.native_assertion_id.as_deref(), Some("333"));
    let absent_assertion = EntityId::new("independent-absent-assertion").unwrap();
    assert!(docs.iter().all(|d| d.metadata.assertions.iter().all(|a| a.id != absent_assertion)));
    let mut engine = selected(root);
    let standalone = DictionaryCompactSearchEngine::open(store).unwrap();
    let valid = engine.detail(&original, None).unwrap();
    let standalone_valid = standalone.detail(&original, None).unwrap();
    assert_eq!(standalone_valid.hit, valid.response.hit);
    assert_eq!(valid.response.status, ReferenceStatus::Resolved);
    assert_eq!(valid.response.hit.as_ref().unwrap().assertion, docs[0].metadata.assertions[0]);
    for field in ["entry", "assertion", "native-slot", "absent-assertion"] {
        let mut changed = original.clone();
        match field {
            "entry" => changed.entry_sha256 = "0".repeat(64),
            "assertion" => changed.assertion_sha256 = "0".repeat(64),
            "native-slot" => changed.native_assertion_id = Some("334".into()),
            "absent-assertion" => changed.assertion_id = absent_assertion.clone(),
            _ => unreachable!(),
        }
        let response = engine.detail(&changed, None).unwrap();
        assert_eq!(response.response.status, ReferenceStatus::NoMatch, "{field}");
        assert!(response.response.hit.is_none());
        assert!(response.response.current_eligible.is_none());
        let standalone_response = standalone.detail(&changed, None).unwrap();
        assert_eq!(standalone_response.status, ReferenceStatus::NoMatch, "standalone {field}");
        assert!(standalone_response.hit.is_none() && standalone_response.current_eligible.is_none());
    }
    let other = &docs[1].metadata.assertions[0];
    for field in ["known-source", "known-entry", "known-occurrence", "native-role"] {
        let mut changed = original.clone();
        match field {
            "known-source" => changed.source = docs[1].metadata.entries.iter().find(|e| e.id == other.entry).unwrap().source.clone(),
            "known-entry" => changed.entry_id = other.entry.clone(),
            "known-occurrence" => changed.occurrence = other.occurrence.clone(),
            "native-role" => {
                changed.native_entry_id = changed.native_assertion_id.take();
            },
            _ => unreachable!(),
        }
        assert_eq!(engine.detail(&changed, None).unwrap_err().report_for("lookup_detail").category, "invalid_reference", "{field}");
        assert_eq!(standalone.detail(&changed, None).unwrap_err().report().code, "invalid_reference", "standalone {field}");
    }
    assert_eq!(engine.detail(&original, None).unwrap().response, valid.response);
    assert_eq!(standalone.detail(&original, None).unwrap(), standalone_valid);
    println!("PASS BOTH routes original native333 detail: body/native-value/absent-target NoMatch; known source/entry/occurrence/wrong-role InvalidReference; complete valid counterpart retained");
}
