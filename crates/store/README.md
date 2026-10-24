# vlajmaji-store

Rebuildable indexes for [Vlajmaji](https://github.com/oxur/vlajmaji)'s source-preserving lexical archives.

SQLite and FTS5 provide local persistence and query indexes. The optional `graph` feature enables an Oxigraph/RocksDB backend for research comparisons and requires native C++/Clang build tooling. The lexical archive remains canonical; indexes do not merge competing source assertions.

This crate also exposes `DictionaryStore`, the immutable SQLite persistence
boundary for the versioned `vlajmaji-lexicon::Dictionary` model. It writes a
concrete schema-1 projection to a fresh directory and reopens it only after
checking canonical bytes, schema, foreign keys and every projected SQL value.
It is separate from the existing rebuildable `SqliteStore` search index.
Its `exact_assertion_ids` helper returns byte-exact lexical candidates in
persisted source/spelling/occurrence order. It does not normalize spelling or
apply catalogue selection; the grouped policy is implemented by
`vlajmaji-search` after the candidate query.

```rust,no_run
use vlajmaji_lexicon::Dictionary;
use vlajmaji_store::DictionaryStore;

let dictionary = Dictionary::from_json_bytes(&std::fs::read("dictionary.json")?)?;
let store = DictionaryStore::create("data/dictionary", &dictionary)?;
drop(store);
let reopened = DictionaryStore::open("data/dictionary")?;
assert_eq!(reopened.dictionary().document(), dictionary.document());
# Ok::<(), Box<dyn std::error::Error>>(())
```

The destination leaf must be fresh and its parent must already exist. Open is
read-only and never migrates or repairs. See the
[schema and trust-boundary guide](../../docs/dictionary-schema.md) and the
`dictionary_store` example.

The next development version is 0.2.0. Until it is published, use a local
path dependency on this crate; the version below describes the upcoming release.

```toml
[dependencies]
vlajmaji-store = "0.2.0"
```

Requires Rust 1.98 or newer. Store APIs use a store directory, rather than the internal SQLite file. Corpora are not bundled.

See the [API documentation](https://docs.rs/vlajmaji-store). Licensed under Apache-2.0.
