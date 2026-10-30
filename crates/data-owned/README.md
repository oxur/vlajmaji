# `vlajmaji-data-owned`

This unpublished, optional package embeds an explicitly selected owned
snapshot and seal. A clean checkout has no `data/` directory and the default
feature set is empty. Build with `embedded` only after preparing both files in
`data/`; the files are ignored and excluded from package contents.

`load()` delegates directly to `vlajmaji-lexicon::load_owned_snapshot` and
returns the complete validated `OwnedSnapshot`, including its retained
historical dictionaries and curation states. The embedded seal checks exact
bytes and identity; it is not a licence grant or redistribution permission.
Pinned foreign context remains a reference until the caller explicitly
materializes it with the appropriate context.

The package depends directly only on `vlajmaji-lexicon`. Applications that
need the owned snapshot without SQLite should depend directly on this package
and lexicon. The umbrella exposes it behind the opt-in `data-owned` feature;
the umbrella also includes store and search and is not the before-SQLite
consumer boundary.

## Prepare a local pair from the workspace root

The repository's synthetic example and CLI can prepare a fresh pair without
fetching external data:

```sh
scratch="$(mktemp -d)"
cargo run --locked --offline -p vlajmaji-store --example owned_curation -- \
  "$scratch/inputs" "$scratch/journal" "$scratch/view.json"
mkdir crates/data-owned/data
cargo run --locked --offline -p vlajmaji-cli -- owned export "$scratch/journal" \
  --output crates/data-owned/data/snapshot.json \
  --seal crates/data-owned/data/seal.json
```

A direct consumer can select the package feature explicitly:

```toml
[dependencies]
vlajmaji-data-owned = { path = "../data-owned", features = ["embedded"] }
vlajmaji-lexicon = { path = "../lexicon" }
```

Then call `vlajmaji_data_owned::load()` and select a retained state with
`dictionary_at` or `curation_at`. Do not add serde or hashing crates merely to
load the bytes; strict validation is provided by lexicon.
