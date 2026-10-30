# Vlajmaji

This checkout is on the 0.3.0 development cycle. The dependency and installation
examples below target the published 0.2.0 release.

An umbrella library for [Vlajmaji](https://github.com/oxur/vlajmaji), a
source-preserving (Re)Lojban dictionary workbench. It re-exports its libraries
as `vlajmaji::lexicon`, `vlajmaji::store`, and `vlajmaji::search`.

## Component crates

| Crate | Role | Documentation |
| --- | --- | --- |
| [vlajmaji-lexicon](https://crates.io/crates/vlajmaji-lexicon) | Source registry, versioned dictionary models, original bytes, provenance, imports, and archives. | [API docs](https://docs.rs/vlajmaji-lexicon) |
| [vlajmaji-store](https://crates.io/crates/vlajmaji-store) | SQLite dictionary persistence, rebuildable FTS5 indexes, and an optional Oxigraph/RocksDB backend. | [API docs](https://docs.rs/vlajmaji-store) |
| [vlajmaji-search](https://crates.io/crates/vlajmaji-search) | Source-grouped dictionary lookup, spelling distance, phonological/prosodic filters, and optional compound analysis. | [API docs](https://docs.rs/vlajmaji-search) |
| [vlajmaji-cli](https://crates.io/crates/vlajmaji-cli) | The separately installed `vlajmaji` JSON command-line tool. | [CLI guide](https://github.com/oxur/vlajmaji/tree/main/crates/cli) |

Use this crate for the umbrella namespace, or depend on component crates
directly. The CLI package is installed separately from the library.
The underlying phonology comes from
[bauske-phonology](https://crates.io/crates/bauske-phonology)
([API docs](https://docs.rs/bauske-phonology)).

## Use as a library

```toml
[dependencies]
vlajmaji = "0.2.0"
```

## Example

Open an existing dictionary store and retrieve exact, source-grouped assertions:

```rust,no_run
use vlajmaji::search::{DictionaryExactOptions, DictionarySearchEngine};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let engine = DictionarySearchEngine::open("dictionary-store")?;
    let result = engine.exact("farvi", DictionaryExactOptions::default())?;
    println!("{} source occurrences", result.totals.occurrences);
    Ok(())
}
```

Corpus data is not bundled. See the
[migration and replay guide](https://github.com/oxur/vlajmaji/blob/main/docs/migration.md)
for importing research inputs and building a store. Records keep competing source
assertions separate; computed scores and generated compounds do not establish
attested usage or semantic equivalence.

## Optional features

The umbrella library enables no optional features by default.

- `morphology` enables compound decomposition and generation through
  `vlajmaji-search` and `latkerlo-jvotci`.
- `graph` enables the Oxigraph/RocksDB backend through `vlajmaji-store`. It
  requires native C++ and Clang/libclang build tooling.
- `data-lensisku-jbotci` selects the unpublished optional embedded pack crate.
  It forwards its `embedded` feature and requires local pack preparation first;
  no dictionary data is included by default.
- `data-llg` selects the unpublished optional embedded gismu/cmavo/assigned
  rafsi pack crate. It forwards that crate's `embedded` feature and requires a
  caller-prepared local payload; no dictionary data is included by default.
- `data-la-lojban` selects the unpublished optional La-Lojban English
  Lensisku pack crate. It forwards `embedded` and requires local payload
  preparation; no dictionary data is included by default and redistribution is
  held pending verified content terms.
- `data-owned` selects and re-exports the unpublished `vlajmaji-data-owned`
  package as `vlajmaji::data_owned`. It validates a complete local snapshot
  and seal; it does not add a fifth upstream pack or implicitly materialize
  foreign context.
- `data-sutysisku` selects the unpublished optional Sutysisku English
  projection pack crate and re-exports it as `vlajmaji::data_sutysisku`. It
  forwards `embedded` and requires local payload preparation; no dictionary
  data is included by default, and redistribution remains Held while upstream
  lexical lineage and content terms are unverified.

These features can be enabled independently or together. The CLI package enables
morphology separately; that does not change this library's defaults for callers
that depend only on `vlajmaji`.

Prepare a synthetic local payload with the CLI's `data-pack fixture jbotci`
command before building the feature. Full pinned local materialization uses
`data-pack materialize jbotci RAW METADATA OUTPUT`. The optional data package
is unpublished, and its local seal records integrity and inventory rather than
publisher authorization or content licensing.

For LLG, prepare a synthetic payload with `data-pack fixture llg OUTPUT` or
materialize explicit pinned local inputs with
`data-pack materialize llg GISMU CMAVO RAFSI NOTICE OUTPUT` before enabling
`data-llg`. The public source notice is retained as source evidence; public
redistribution remains Held.

For La-Lojban, prepare a Synthetic local payload with
`data-pack fixture la-lojban OUTPUT` or materialize the locally pinned export
with `data-pack materialize la-lojban RAW_JSON OUTPUT` before enabling
`data-la-lojban`. Its seal records local integrity and inventory; it does not
establish permission to redistribute the dictionary content.

For Sutysisku, prepare a Synthetic local payload with
`data-pack fixture sutysisku OUTPUT` for an offline feature build, or
materialize caller-provided pinned bytes with
`data-pack materialize sutysisku RAW_JSON OUTPUT` for a Full local payload
before enabling `data-sutysisku`. The command reads only the supplied path and
does not fetch data or create a default store. The package is unpublished, and
content redistribution remains Held pending verified terms and upstream
lexical lineage.

Requires Rust 1.98 or newer. Licensed under Apache-2.0;
corpus content terms remain separate from the code license.

For the versioned dictionary APIs, see the
[dictionary model guide](https://github.com/oxur/vlajmaji/blob/main/docs/dictionary-model.md)
and [source-grouped lookup guide](https://github.com/oxur/vlajmaji/blob/main/docs/dictionary-query.md).
