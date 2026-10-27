//! Optional, compile-time embedded Lensisku dictionary pack.
//!
//! This package is unpublished and has no payload in a clean checkout. Prepare
//! its ignored `data/` directory with the offline CLI fixture or a locally
//! pinned full import before enabling the `embedded` feature.

use vlajmaji_lexicon::PackId;
#[cfg(feature = "embedded")]
use vlajmaji_lexicon::{Dictionary, PackError, load_sealed_release};

/// Stable pack identifier for this package.
pub const PACK_ID: PackId = PackId::LensiskuJbotci;

/// Returns the compile-time included canonical release bytes.
///
/// # Panics
///
/// No panic is possible when this crate compiles; the bytes are included at
/// build time and exposed as a static slice.
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

/// Loads the embedded release through the lexicon's strict seal and model checks.
///
/// # Errors
///
/// Returns a typed error if the seal, release bytes, inventory, registry, or
/// complete dictionary model fails validation.
#[cfg(feature = "embedded")]
pub fn load() -> Result<Dictionary, PackError> {
    load_sealed_release(PACK_ID, release_bytes(), seal_bytes())
}

/// Returns whether a local payload was included in this build.
#[must_use]
pub const fn is_embedded() -> bool {
    cfg!(feature = "embedded")
}
