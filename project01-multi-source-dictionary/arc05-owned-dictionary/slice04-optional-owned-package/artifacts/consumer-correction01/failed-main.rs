use std::error::Error;

use vlajmaji_lexicon::{EntityId, OwnedSnapshotError, load_owned_snapshot};

const POSITIVE_SNAPSHOT: &[u8] = include_bytes!("../inputs/snapshot.json");
const POSITIVE_SEAL: &[u8] = include_bytes!("../inputs/seal.json");
const HISTORIES: &[(&str, &[u8], &[u8], &str)] = &[
    (
        "owned-r0",
        include_bytes!("../inputs/history/owned-r0.json"),
        include_bytes!("../inputs/history/owned-r0.curation.json"),
        "178cb4a48245db47a568bb740f5ec123575aae38eb961c9ce05f40c7f0526c9a",
    ),
    (
        "owned-r1",
        include_bytes!("../inputs/history/owned-r1.json"),
        include_bytes!("../inputs/history/owned-r1.curation.json"),
        "e96d7fdee27a1cf84b152799b51c7b6e541aeae714396665fa98985f5c81d05f",
    ),
    (
        "owned-r2",
        include_bytes!("../inputs/history/owned-r2.json"),
        include_bytes!("../inputs/history/owned-r2.curation.json"),
        "1cdf04f2a5dfc078ebc7c55416821a2a62a5835865c21846a78d289b142cef3a",
    ),
    (
        "owned-r3",
        include_bytes!("../inputs/history/owned-r3.json"),
        include_bytes!("../inputs/history/owned-r3.curation.json"),
        "d5279857727404503d065e4760960e2665d0e3b617cd5d88249eba53fb4e3917",
    ),
    (
        "owned-r4",
        include_bytes!("../inputs/history/owned-r4.json"),
        include_bytes!("../inputs/history/owned-r4.curation.json"),
        "5d912d6f36fe0df64045a862ed4dd228fde1caf2d2371979e9e86f758b0ab1a5",
    ),
];

fn main() -> Result<(), Box<dyn Error>> {
    let embedded = vlajmaji_data_owned::load()?;
    assert_eq!(embedded.to_json_bytes()?.as_slice(), POSITIVE_SNAPSHOT);
    assert_eq!(embedded.as_data().commits.len(), 5);
    assert_eq!(embedded.as_data().commits[0].prefixes.len(), 24);

    let snapshot = load_owned_snapshot(POSITIVE_SNAPSHOT, POSITIVE_SEAL)?;
    assert_eq!(snapshot.to_json_bytes()?.as_slice(), POSITIVE_SNAPSHOT);
    assert_eq!(snapshot.as_data().commits.len(), 5);
    assert_eq!(snapshot.as_data().commits[0].prefixes.len(), 24);

    for (revision, expected_dictionary, expected_curation, expected_state_sha256) in HISTORIES {
        let revision = EntityId::new((*revision).to_owned())?;
        let dictionary = snapshot.dictionary_at(&revision)?;
        assert_eq!(dictionary.to_json_bytes()?.as_slice(), *expected_dictionary);

        let curation = snapshot
            .curation_at(&revision)?
            .expect("schema-two history has one curation state per base revision");
        assert_eq!(curation.to_json_bytes()?.as_slice(), *expected_curation);
        assert_eq!(curation.state_sha256()?, *expected_state_sha256);
    }

    assert!(matches!(
        load_owned_snapshot(
            include_bytes!("../inputs/tampered-payload.json"),
            POSITIVE_SEAL,
        ),
        Err(OwnedSnapshotError::HashMismatch { expected, actual })
            if expected == "2d4925e6baf72044b71cae7a78f466412276b3d837765a6f0b1ed59fe58c3d8a"
                && actual == "7c2ce4f8e77964c75a0712ba2415ea798271609fb8ea978f59863cb2d7c016a9"
    ));
    assert!(matches!(
        load_owned_snapshot(
            include_bytes!("../inputs/unsupported-payload.json"),
            include_bytes!("../inputs/unsupported-payload.seal.json"),
        ),
        Err(OwnedSnapshotError::UnsupportedVersion {
            kind: "owned snapshot",
            found: 2,
        })
    ));
    assert!(matches!(
        load_owned_snapshot(
            include_bytes!("../inputs/unknown-prefix.json"),
            include_bytes!("../inputs/unknown-prefix.seal.json"),
        ),
        Err(OwnedSnapshotError::History(detail))
            if detail == "commit roster section 0 must be registry.sources"
    ));
    assert!(matches!(
        load_owned_snapshot(
            POSITIVE_SNAPSHOT,
            include_bytes!("../inputs/unsupported-seal.json"),
        ),
        Err(OwnedSnapshotError::UnsupportedVersion {
            kind: "owned snapshot seal",
            found: 2,
        })
    ));
    assert!(matches!(
        load_owned_snapshot(
            POSITIVE_SNAPSHOT,
            include_bytes!("../inputs/wrong-kind.seal.json"),
        ),
        Err(OwnedSnapshotError::InvalidDocument(detail))
            if detail == "unexpected owned snapshot seal kind"
    ));

    let restored = load_owned_snapshot(POSITIVE_SNAPSHOT, POSITIVE_SEAL)?;
    assert_eq!(restored.to_json_bytes()?.as_slice(), POSITIVE_SNAPSHOT);
    println!("verified embedded complete history: 5 base states, 5 curation states, 24 typed section prefixes, and strict loader errors");
    Ok(())
}
