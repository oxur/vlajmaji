# `vlajmaji-data-sutysisku`

This unpublished optional package embeds a locally materialized Sutysisku English projection at compile time. A clean checkout has no data payload. From the workspace root, prepare either the small CI fixture or a full local pack with the CLI:

```sh
cargo run --locked --offline -p vlajmaji-cli -- data-pack fixture sutysisku crates/data-sutysisku/data
# or
cargo run --locked --offline -p vlajmaji-cli -- data-pack materialize sutysisku PATH/TO/parsed-en.json crates/data-sutysisku/data
```

Then enable `vlajmaji-data-sutysisku/embedded` or the umbrella `data-sutysisku` feature. The package uses the normalized dictionary model and its strict seal, so it can participate in pure consuming composition with the other local packs before any SQLite store is built. No network access or runtime filesystem lookup is used by the embedded loader.

The pinned projection's upstream lexical lineage is not established. Its repository's MIT software license does not grant dictionary-content redistribution rights. The release therefore records content redistribution as **Held** until the lineage and data terms are verified.
