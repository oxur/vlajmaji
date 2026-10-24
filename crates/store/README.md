# vlajmaji-store

Rebuildable indexes for [Vlajmaji](https://github.com/oxur/vlajmaji)'s source-preserving lexical archives.

SQLite and FTS5 provide local persistence and query indexes. The optional `graph` feature enables an Oxigraph/RocksDB backend for research comparisons and requires native C++/Clang build tooling. The lexical archive remains canonical; indexes do not merge competing source assertions.

The next development version is 0.2.0. Until it is published, use a local
path dependency on this crate; the version below describes the upcoming release.

```toml
[dependencies]
vlajmaji-store = "0.2.0"
```

Requires Rust 1.98 or newer. Store APIs use a store directory, rather than the internal SQLite file. Corpora are not bundled.

See the [API documentation](https://docs.rs/vlajmaji-store). Licensed under Apache-2.0.
