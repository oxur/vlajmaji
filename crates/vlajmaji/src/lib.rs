#![doc = include_str!("../README.md")]
#![deny(rustdoc::broken_intra_doc_links)]

/// Source-preserving lexical records, imports, and archives.
#[doc(inline)]
pub use vlajmaji_lexicon as lexicon;

/// Lexical queries and computed spelling, sound, and compound analyses.
#[doc(inline)]
pub use vlajmaji_search as search;

/// Rebuildable SQLite indexes and the optional graph backend.
#[doc(inline)]
pub use vlajmaji_store as store;
