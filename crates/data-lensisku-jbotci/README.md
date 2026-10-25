# `vlajmaji-data-lensisku-jbotci`

This unpublished optional crate embeds a locally prepared Lensisku pack at
compile time. A clean checkout intentionally has no dictionary payload, and
the default feature set does not require one. There is no runtime file lookup,
network request, or fallback fixture.

To prepare a clearly synthetic three-row fixture for local feature checks:

```sh
cargo run --locked --offline -p vlajmaji-cli -- \
  data-pack fixture jbotci crates/data-lensisku-jbotci/data
cargo check --locked --offline -p vlajmaji-data-lensisku-jbotci \
  --features embedded
```

To prepare a full pack from exact local pinned inputs, use
`data-pack materialize jbotci RAW METADATA crates/data-lensisku-jbotci/data`.
The importer checks the actual bytes and metadata before writing. The local
payload is ignored by Git and is never fetched during builds. `load()` verifies
the seal and complete dictionary model; the seal records integrity and
inventory only, not publisher authentication or redistribution permission.

This data package is not published. It is a source-tree preparation mechanism
for local research and consumers that explicitly choose the embedded feature.
