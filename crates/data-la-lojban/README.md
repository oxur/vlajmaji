# vlajmaji-data-la-lojban

This unpublished optional dependency exposes the locally embedded La-Lojban
English dictionary export through the versioned Vlajmaji model. A clean checkout
contains no payload. From the workspace root, prepare ignored local data with
`cargo run --locked --offline -p vlajmaji-cli -- data-pack fixture la-lojban crates/data-la-lojban/data`
for synthetic builds, or
`cargo run --locked --offline -p vlajmaji-cli -- data-pack materialize la-lojban RAW_JSON crates/data-la-lojban/data`
for the locally pinned full export. The output directory is the fixed location
used by this crate's compile-time `include_bytes!` inputs. Preparation reads
only caller-provided local files and does not create a dictionary store.

Enable the `embedded` feature before using `release_bytes`, `seal_bytes`, or
`load`. The default feature set contains no data and needs no corpus files. The
seal is a local integrity inventory, not publisher authentication or permission
to redistribute the dictionary content. Content redistribution remains held
pending verified terms; the workspace's software license does not grant data
rights.
