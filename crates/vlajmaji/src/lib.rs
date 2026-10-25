#![doc = include_str!("../README.md")]
#![deny(rustdoc::broken_intra_doc_links)]

/// Source-preserving lexical records, imports, and archives.
#[doc(inline)]
pub use vlajmaji_lexicon as lexicon;

/// Lexical queries and computed spelling, sound, and compound analyses.
#[doc(inline)]
pub use vlajmaji_search as search;

/// Optional embedded full Lensisku pack distributed through jbotci.
#[cfg(feature = "data-lensisku-jbotci")]
pub use vlajmaji_data_lensisku_jbotci as data_lensisku_jbotci;
/// Optional embedded LLG list-family pack.
#[cfg(feature = "data-llg")]
pub use vlajmaji_data_llg as data_llg;
/// Rebuildable SQLite indexes and the optional graph backend.
#[doc(inline)]
pub use vlajmaji_store as store;
