//! Optional, compile-time embedded LLG list-family pack.
//!
//! This unpublished package contains no payload in a clean checkout. Prepare
//! its ignored `data/` directory with the offline CLI fixture or local
//! materializer before enabling `embedded`.

use vlajmaji_lexicon::PackId;
#[cfg(feature = "embedded")]
use vlajmaji_lexicon::{Dictionary, PackError, load_sealed_release};

/// Stable pack identifier for this package.
pub const PACK_ID: PackId = PackId::Llg;

/// Returns the compile-time included canonical release bytes.
#[cfg(feature = "embedded")]
#[must_use]
pub fn release_bytes() -> &'static [u8] {
    include_bytes!("../data/release.json")
}

/// Returns the compile-time included strict seal bytes.
#[cfg(feature = "embedded")]
#[must_use]
pub fn seal_bytes() -> &'static [u8] {
    include_bytes!("../data/seal.json")
}

/// Loads the embedded release through strict seal and dictionary checks.
///
/// # Errors
///
/// Returns a typed error if the seal, release bytes, inventory, registry, or
/// complete dictionary model fails validation.
#[cfg(feature = "embedded")]
pub fn load() -> Result<Dictionary, PackError> {
    load_sealed_release(PACK_ID, release_bytes(), seal_bytes())
}

/// Returns whether a local payload was embedded in this build.
#[must_use]
pub const fn is_embedded() -> bool {
    cfg!(feature = "embedded")
}
