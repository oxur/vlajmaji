#![doc = include_str!("../README.md")]
#![deny(rustdoc::broken_intra_doc_links)]

#[cfg(feature = "embedded")]
use vlajmaji_lexicon::{OwnedSnapshot, load_owned_snapshot};

/// Returns whether this build includes a local snapshot and seal.
#[must_use]
pub const fn is_embedded() -> bool {
    cfg!(feature = "embedded")
}

/// Returns the embedded snapshot bytes.
///
/// The file is included only when the `embedded` feature is enabled.
#[cfg(feature = "embedded")]
#[must_use]
pub fn snapshot_bytes() -> &'static [u8] {
    include_bytes!("../data/snapshot.json")
}

/// Returns the embedded seal bytes.
///
/// The file is included only when the `embedded` feature is enabled.
#[cfg(feature = "embedded")]
#[must_use]
pub fn seal_bytes() -> &'static [u8] {
    include_bytes!("../data/seal.json")
}

/// Loads the complete embedded owned history after strict seal validation.
///
/// # Errors
///
/// Returns the exact validation error from the strict snapshot and seal
/// loader if the bytes, seal, inventory, history, or any historical dictionary
/// or curation state is invalid.
#[cfg(feature = "embedded")]
pub fn load() -> Result<OwnedSnapshot, vlajmaji_lexicon::OwnedSnapshotError> {
    load_owned_snapshot(snapshot_bytes(), seal_bytes())
}
