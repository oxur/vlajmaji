//! Public controls for the LLG list-family import boundary.

use sha2::{Digest, Sha256};
use vlajmaji_lexicon::{
    ContextKind, Eligibility, FormKind, LlgImportError, LlgImportOptions, Presence,
    import_llg_lists,
};

fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn options(gismu: &[u8], cmavo: &[u8], rafsi: &[u8], notice: &[u8]) -> LlgImportOptions {
    LlgImportOptions {
        expected_gismu_sha256: digest(gismu),
        expected_cmavo_sha256: digest(cmavo),
        expected_rafsi_sha256: digest(rafsi),
        expected_notice_sha256: digest(notice),
        base_locator: "https://example.invalid/llg".into(),
    }
}

fn put(line: &mut [u8], start: usize, end: usize, value: &str) {
    let bytes = value.as_bytes();
    assert!(bytes.len() <= end - start);
    line[start..start + bytes.len()].copy_from_slice(bytes);
}

fn gismu_row(word: &str, optional: bool) -> Vec<u8> {
    let mut line = vec![b' '; if optional { 170 } else { 158 }];
    put(&mut line, 1, 6, word);
    put(&mut line, 7, 10, "far");
    put(&mut line, 11, 14, "fav");
    put(&mut line, 15, 19, "farv");
    put(&mut line, 20, 40, "grow");
    put(&mut line, 41, 61, "growth");
    put(&mut line, 62, 158, "to grow");
    if optional {
        put(&mut line, 159, 161, "1A");
        put(&mut line, 161, 165, "0010");
        put(&mut line, 165, 170, " note");
    }
    line
}

fn lines(lines: &[Vec<u8>], ending: &[u8], terminal: bool) -> Vec<u8> {
    let mut output = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        if index > 0 {
            output.extend_from_slice(ending);
        }
        output.extend_from_slice(line);
    }
    if terminal {
        output.extend_from_slice(ending);
    }
    output
}

#[test]
fn test_import_llg_lists_preserves_captures_rows_and_evidence() {
    let gismu = lines(
        &[b"00001 Lojban gismu".to_vec(), gismu_row("farvi", true)],
        b"\r\n",
        true,
    );
    let cmavo = b"00001 Lojban cmavo\r\ndo        BAI       because                                 because it is so\r\n".to_vec();
    let rafsi = b"far farvi growth\r\nfar farvi develop\r\n".to_vec();
    let notice = b"LLG notice\r\n".to_vec();
    let result = import_llg_lists(
        &gismu,
        &cmavo,
        &rafsi,
        &notice,
        &options(&gismu, &cmavo, &rafsi, &notice),
    )
    .unwrap();
    let dictionary = result.dictionary();
    let report = result.report();

    assert_eq!(report.captures, 4);
    assert_eq!(report.occurrences, 6);
    assert_eq!(report.lexical_assertions, 2);
    assert_eq!(report.excluded_assertions, 4);
    assert_eq!(report.entries, 4);
    assert_eq!(report.assigned_forms, 5);
    assert_eq!(report.contexts, 6);
    assert_eq!(report.extensions, 2);
    assert_eq!(report.teaching_code_extensions, 1);
    assert_eq!(report.frequency_extensions, 1);
    assert_eq!(report.components[0].native_fields, 11);
    assert_eq!(report.components[1].native_fields, 5);
    assert_eq!(report.components[2].native_fields, 6);

    let gismu_capture = dictionary
        .captures()
        .iter()
        .find(|capture| capture.identity.namespace == "captured:llg:gismu")
        .unwrap();
    assert_eq!(gismu_capture.bytes, gismu);
    assert_eq!(
        gismu_capture.occurrences[1].raw_line.as_deref(),
        Some(std::str::from_utf8(&gismu_row("farvi", true)).unwrap())
    );
    assert_eq!(gismu_capture.occurrences[1].fields[0].raw_value, "farvi");
    assert_eq!(gismu_capture.occurrences[1].fields[7].name, "teaching_code");
    assert_eq!(gismu_capture.occurrences[1].fields[7].raw_value, "1A");
    assert_eq!(
        dictionary.registry().as_document().sources[0].display_name,
        "The Logical Language Group, Inc."
    );

    let do_entry = dictionary
        .metadata()
        .entries
        .iter()
        .find(|entry| entry.spelling == Presence::Value("do".into()))
        .unwrap();
    assert_eq!(do_entry.selmaho, Presence::Value("BAI".into()));
    assert!(
        dictionary
            .metadata()
            .forms
            .iter()
            .all(|form| form.kind == FormKind::AssignedRafsi)
    );
    assert_eq!(
        dictionary
            .metadata()
            .contexts
            .iter()
            .filter(|context| context.kind == ContextKind::Gloss)
            .count(),
        4
    );
    assert_eq!(
        dictionary
            .metadata()
            .assertions
            .iter()
            .filter(|assertion| matches!(assertion.eligibility, Eligibility::Excluded { .. }))
            .count(),
        4
    );
}

#[test]
fn test_import_llg_lists_reports_count_mismatch_without_dropping_rows() {
    let gismu = lines(
        &[b"00099 Lojban gismu".to_vec(), gismu_row("farvi", false)],
        b"\n",
        true,
    );
    let cmavo = b"00001 Lojban cmavo\ndo        BAI       because                                 because it is so\n".to_vec();
    let rafsi = b"far farvi growth\n".to_vec();
    let notice = b"notice".to_vec();
    let result = import_llg_lists(
        &gismu,
        &cmavo,
        &rafsi,
        &notice,
        &options(&gismu, &cmavo, &rafsi, &notice),
    )
    .unwrap();
    assert_eq!(result.report().components[0].declared_body_count, Some(99));
    assert_eq!(result.report().components[0].observed_body_count, 1);
    assert_eq!(
        result.report().components[0].declared_count_matches,
        Some(false)
    );
    assert_eq!(result.report().occurrences, 5);
    assert_eq!(
        result.dictionary().metadata().releases[0].diagnostics[0].code,
        "source_header"
    );
    assert!(
        result.dictionary().metadata().releases[0]
            .diagnostics
            .iter()
            .any(|row| row.code == "header_count_mismatch")
    );
}

#[test]
fn test_import_llg_lists_retains_unsupported_rows_and_terminal_cr() {
    let mut gismu = lines(
        &[
            b"00001 Lojban gismu".to_vec(),
            gismu_row("farvi", false),
            b"separator".to_vec(),
            "nonascii é".as_bytes().to_vec(),
            b"bare-cr\r".to_vec(),
        ],
        b"\r\n",
        false,
    );
    let cmavo = b"00000 Lojban cmavo".to_vec();
    let rafsi = Vec::new();
    let notice = b"notice".to_vec();
    let result = import_llg_lists(
        &gismu,
        &cmavo,
        &rafsi,
        &notice,
        &options(&gismu, &cmavo, &rafsi, &notice),
    )
    .unwrap();
    let capture = result
        .dictionary()
        .captures()
        .iter()
        .find(|capture| capture.identity.namespace == "captured:llg:gismu")
        .unwrap();
    assert_eq!(capture.occurrences.len(), 5);
    assert_eq!(capture.occurrences[2].fields[0].name, "$line");
    assert_eq!(capture.occurrences[3].fields[0].name, "$line");
    assert_eq!(
        capture.occurrences[4].raw_line.as_deref(),
        Some("bare-cr\r")
    );
    assert_eq!(result.report().components[0].unsupported_rows, 3);
    assert_eq!(result.report().components[0].form_only_assertions, 0);
    assert_eq!(result.report().components[2].form_only_assertions, 0);
    assert_eq!(result.report().occurrences, 6);
    assert_eq!(result.report().excluded_assertions, 5);
    gismu.extend_from_slice(b"\r\n");
    let ended = import_llg_lists(
        &gismu,
        &cmavo,
        &rafsi,
        &notice,
        &options(&gismu, &cmavo, &rafsi, &notice),
    )
    .unwrap();
    assert_eq!(ended.report().components[0].input_lines, 5);
}

#[test]
fn test_import_llg_lists_rejects_bad_pins_and_invalid_utf8() {
    let gismu = b"not a list".to_vec();
    let cmavo = Vec::new();
    let rafsi = Vec::new();
    let notice = b"notice".to_vec();
    let mut supplied = options(&gismu, &cmavo, &rafsi, &notice);
    supplied.expected_gismu_sha256 = "BAD".into();
    assert!(matches!(
        import_llg_lists(&gismu, &cmavo, &rafsi, &notice, &supplied),
        Err(LlgImportError::InvalidOptions(_))
    ));
    let supplied = options(&gismu, &cmavo, &rafsi, &notice);
    assert!(
        matches!(import_llg_lists(b"\xff", &cmavo, &rafsi, &notice, &options(b"\xff", &cmavo, &rafsi, &notice)), Err(LlgImportError::InvalidUtf8 { input }) if input == "gismu")
    );
    assert!(
        matches!(import_llg_lists(b"different", &cmavo, &rafsi, &notice, &supplied), Err(LlgImportError::HashMismatch { input, .. }) if input == "gismu")
    );
}

#[test]
fn test_import_llg_lists_keeps_duplicate_rows_and_excludes_conflicting_classes() {
    let gismu = lines(
        &[
            b"00002 Lojban gismu".to_vec(),
            gismu_row("farvi", false),
            gismu_row("farvi", false),
        ],
        b"\r\n",
        true,
    );
    let cmavo_row = |class: &str| {
        format!(
            "{:<10}{:<10}{:<41}{}",
            "do", class, "because", "because it is so"
        )
        .into_bytes()
    };
    let cmavo = lines(
        &[
            b"00002 Lojban cmavo".to_vec(),
            cmavo_row("BAI"),
            cmavo_row("UI"),
        ],
        b"\r\n",
        true,
    );
    let rafsi = lines(
        &[b"far farvi growth".to_vec(), b"far farvi growth".to_vec()],
        b"\r\n",
        true,
    );
    let notice = b"notice\r\n".to_vec();
    let result = import_llg_lists(
        &gismu,
        &cmavo,
        &rafsi,
        &notice,
        &options(&gismu, &cmavo, &rafsi, &notice),
    )
    .unwrap();
    let dictionary = result.dictionary();
    assert_eq!(result.report().occurrences, 8);
    assert_eq!(result.report().lexical_assertions, 4);
    assert_eq!(result.report().excluded_assertions, 4);
    assert_eq!(result.report().entries, 4);
    assert_eq!(result.report().assigned_forms, 8);
    assert_eq!(result.report().components[2].form_only_assertions, 2);
    assert_eq!(dictionary.captures()[1].occurrences.len(), 3);
    assert_eq!(dictionary.captures()[2].occurrences.len(), 3);
    assert_eq!(dictionary.captures()[3].occurrences.len(), 2);
    let do_entry = dictionary
        .metadata()
        .entries
        .iter()
        .find(|entry| entry.spelling == Presence::Value("do".into()))
        .unwrap();
    assert_eq!(do_entry.selmaho, Presence::Missing);
    assert_eq!(
        dictionary.metadata().releases[1]
            .diagnostics
            .iter()
            .filter(|row| row.code == "conflicting_cmavo_classes")
            .count(),
        1
    );
}

#[test]
fn test_import_llg_lists_excludes_malformed_rows_and_keeps_optional_tail_absent() {
    let gismu = lines(
        &[
            b"00001 Lojban gismu".to_vec(),
            gismu_row("farvi", false),
            b"short".to_vec(),
            Vec::new(),
        ],
        b"\n",
        true,
    );
    let cmavo = b"00000 Lojban cmavo\ndo\n".to_vec();
    let rafsi = b"incomplete\n".to_vec();
    let notice = b"notice".to_vec();
    let result = import_llg_lists(
        &gismu,
        &cmavo,
        &rafsi,
        &notice,
        &options(&gismu, &cmavo, &rafsi, &notice),
    )
    .unwrap();
    let report = result.report();
    assert_eq!(report.occurrences, 7);
    assert_eq!(report.lexical_assertions, 1);
    assert_eq!(report.excluded_assertions, 6);
    assert_eq!(report.components[0].unsupported_rows, 2);
    assert_eq!(report.components[0].native_fields, 10);
    assert_eq!(report.components[1].unsupported_rows, 1);
    assert_eq!(report.components[2].unsupported_rows, 1);
    assert_eq!(report.components[2].form_only_assertions, 0);
    let gismu_capture = result
        .dictionary()
        .captures()
        .iter()
        .find(|capture| capture.identity.namespace == "captured:llg:gismu")
        .unwrap();
    let valid_fields = &gismu_capture.occurrences[1].fields;
    assert_eq!(valid_fields.len(), 7);
    assert!(valid_fields.iter().all(|native| {
        !matches!(
            native.name.as_str(),
            "teaching_code" | "frequency_1993" | "notes"
        )
    }));
}
