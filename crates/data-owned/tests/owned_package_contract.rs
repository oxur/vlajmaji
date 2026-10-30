//! Contract checks for the optional owned-data package feature boundary.

use vlajmaji_data_owned::is_embedded;

#[cfg(not(feature = "embedded"))]
#[test]
fn test_default_feature_has_no_embedded_payload() {
    assert!(!is_embedded());
}

#[cfg(feature = "embedded")]
#[test]
fn test_embedded_feature_loads_complete_snapshot_and_roundtrips() {
    use vlajmaji_data_owned::{load, seal_bytes, snapshot_bytes};
    use vlajmaji_lexicon::load_owned_snapshot;

    assert!(is_embedded());
    let snapshot = load().expect("the prepared embedded pair must pass strict loading");
    assert_eq!(
        snapshot
            .to_json_bytes()
            .expect("the validated snapshot must serialize"),
        snapshot_bytes()
    );
    let reloaded = load_owned_snapshot(snapshot_bytes(), seal_bytes())
        .expect("the exact embedded pair must reload through the strict public loader");
    assert_eq!(
        reloaded
            .to_json_bytes()
            .expect("the reloaded snapshot must serialize"),
        snapshot_bytes()
    );
}
