#![doc = include_str!("../README.md")]
#![deny(rustdoc::broken_intra_doc_links)]

/// Source-preserving lexical records, imports, and archives.
#[doc(inline)]
pub use vlajmaji_lexicon as lexicon;

/// Lexical queries and computed spelling, sound, and compound analyses.
#[doc(inline)]
pub use vlajmaji_search as search;

/// Optional embedded La-Lojban English dictionary pack.
#[cfg(feature = "data-la-lojban")]
pub use vlajmaji_data_la_lojban as data_la_lojban;
/// Optional embedded full Lensisku pack distributed through jbotci.
#[cfg(feature = "data-lensisku-jbotci")]
pub use vlajmaji_data_lensisku_jbotci as data_lensisku_jbotci;
/// Optional embedded LLG list-family pack.
#[cfg(feature = "data-llg")]
pub use vlajmaji_data_llg as data_llg;
/// Optional embedded Sutysisku English projection pack.
#[cfg(feature = "data-sutysisku")]
pub use vlajmaji_data_sutysisku as data_sutysisku;
/// Rebuildable SQLite indexes and the optional graph backend.
#[doc(inline)]
pub use vlajmaji_store as store;

/// Optional embedded, sealed owned dictionary history.
#[cfg(feature = "data-owned")]
pub use vlajmaji_data_owned as data_owned;

#[cfg(all(test, feature = "data-owned"))]
mod owned_data_smoke {
    #[test]
    fn test_data_owned_reexport_loads_complete_embedded_history() {
        assert!(super::data_owned::is_embedded());
        let snapshot = super::data_owned::load()
            .expect("the prepared owned snapshot must pass strict whole-history loading");
        assert_eq!(
            snapshot
                .to_json_bytes()
                .expect("the validated history must serialize")
                .as_slice(),
            super::data_owned::snapshot_bytes()
        );
    }
}
