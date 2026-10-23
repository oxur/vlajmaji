//! Public record identity behavior from an external-crate perspective.

use vlajmaji_lexicon::{Lexicon, SourceFormat, SourceMetadata, TimePrecision};
use vlajmaji_store::record_id;

#[test]
fn external_consumer_can_hydrate_the_published_record_identity() {
    let mut lexicon = Lexicon::new();
    lexicon
        .import_source(
            SourceMetadata::new(
                "consumer-test",
                "fixture",
                SourceFormat::LaLojbanJson,
                "fixture.json",
                "v1",
                None,
                TimePrecision::Unknown,
            ),
            br#"[{"word":"farvi","definition":"grow"}]"#,
            None,
        )
        .unwrap();
    let record = &lexicon.records()[0];
    let id = record_id(record.identity());
    assert_eq!(id.len(), 2 * (8 * 3 + "consumer-test".len() + 64 + 2));
    assert!(id.bytes().all(|byte| byte.is_ascii_hexdigit()));
}
