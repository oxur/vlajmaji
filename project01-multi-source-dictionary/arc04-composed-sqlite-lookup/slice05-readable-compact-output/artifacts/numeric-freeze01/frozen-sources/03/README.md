# Vlajmaji

[![Build Status][gh-actions-badge]][gh-actions]
[![Tags][github-tags-badge]][github-tags]
[![][crate-badge]][crate]
[![][docs-badge]][docs]

[![Project Logo][logo]][logo-large]

*(Re)Lojban dictionaries, schemas, libraries, and command-line tools*

Vlajmaji is a growing dictionary workbench for exploring words, their sources,
and the possibilities of combining them. Its aim is to bring multiple
lexicons together without losing who said what, then make that material useful
for language study, word creation, poetry, and song.

## Current 0.3 development checkout

The capabilities below describe the current workspace checkout, not the published
0.2.0 packages. The latest published release is 0.2.0.

The Rust workspace already provides:

- A lexical archive that preserves source snapshots, raw records, provenance,
  and alternative assertions, keeping computed analyses and user proposals separate.
- A validated source registry for exact aliases, lineage, contributors, and
  attribution of preserved occurrences ([guide](docs/source-registry.md)).
- An immutable, versioned dictionary model with explicit source, owned,
  editorial, and computed evidence, plus an explicit legacy archive adapter
  ([model guide](docs/dictionary-model.md),
  [standards mapping limits](docs/dictionary-standards-map.md)).
- SQLite persistence for that dictionary model, with a minimum-header read-only
  open and separately requested schema, metadata, hash, model, foreign-key, and
  projection verification ([schema guide](docs/dictionary-schema.md)).
- Source-grouped exact lookup with source selection, complete pre-page totals,
  occurrence-based paging, and stable local/catalogue reference resolution
  ([query guide](docs/dictionary-query.md)).
- Rebuildable SQLite indexes and an optional Oxigraph backend for research comparisons.
- Exact and reverse-definition lookup, source-link traversal, spelling distance,
  and bounded phonological and prosodic filters.
- Optional compound decomposition and generation through `latkerlo-jvotci`.
- A JSON CLI, `vlajmaji`, including `dictionary-build` and `dictionary-query`
  alongside the existing archive/search commands. Grouped commands can use a
  default store under XDG data home or the standard HOME data directory, while
  the legacy archive commands remain explicit.
- An explicit local collection catalogue: `collections register` records
  source/reference metadata, while `collections install` builds one verified
  Unified generation. These commands do not configure the default store or
  select sources ([collection guide](docs/collections.md)).
- A lossless local jbotci export normalizer with `jbotci-import` and
  read-only `jbotci-check` commands and an explicit pinned acquire/replay/check/
  update workflow ([import guide](docs/jbotci-import.md)).
- An optional unpublished Lensisku/jbotci data package with strict local seals
  and consuming pure composition. `data-pack fixture jbotci OUTPUT` prepares
  synthetic CI/local data; `data-pack materialize jbotci RAW METADATA OUTPUT`
  prepares a full local pack from caller-provided pinned inputs. Neither command
  fetches data or writes a default store. The optional pack is absent from
  default builds, and its seal is an integrity inventory rather than publisher
  authentication or licensing evidence.
- A lossless adapter for the complete LLG gismu, cmavo, and assigned rafsi
  lists, plus an optional unpublished `vlajmaji-data-llg` embedded package.
  `data-pack fixture llg OUTPUT` prepares a Synthetic payload;
  `data-pack materialize llg GISMU CMAVO RAFSI NOTICE OUTPUT` normalizes
  caller-supplied local bytes against pinned hashes. No command fetches or
  redistributes source data automatically; terms remain Held.
- A lossless adapter for the pinned La-Lojban English Lensisku export, plus an
  optional unpublished embedded package. `data-pack fixture la-lojban OUTPUT`
  prepares Synthetic CI/local bytes; `data-pack materialize la-lojban RAW_JSON
  OUTPUT` validates the local full-byte pin and writes a Full pack. The package
  is absent from default builds, and content redistribution remains Held
  because software licensing does not establish data rights.
- A lossless adapter for the pinned Sutysisku English JSON projection and an
  optional unpublished embedded package. `data-pack fixture sutysisku OUTPUT`
  prepares Synthetic CI/local bytes; `data-pack materialize sutysisku RAW_JSON
  OUTPUT` verifies the complete caller-provided pin and writes a Full pack.
  The package is absent from default builds, and content redistribution remains
  Held pending verified terms and lexical lineage.
- All four local packs share the normalized model and strict seal, enabling
  pure consuming composition before a SQLite store is built.
- An umbrella library, `vlajmaji`, exposing `lexicon`, `store`, and `search`, with
  optional `morphology` and `graph` features.

The earlier archive/search PoC was exercised against 21,241 records from 16
source snapshots. The new grouped-dictionary path has a reproducible fixture
and public API/CLI examples. Four pinned local adapters now cover Lensisku/
jbotci, LLG, La-Lojban, and Sutysisku; adapters for other sources remain future
work.
Search scores and generated compounds are experimental aids; they do not establish
attested usage, semantic equivalence, or poetic merit.

| Package | Role |
| --- | --- |
| [`vlajmaji`](crates/vlajmaji) | Umbrella library re-exporting lexicon, store, and search, with opt-in morphology and graph features |
| [`vlajmaji-lexicon`](crates/lexicon) | Source-preserving lexical data and archives |
| [`vlajmaji-store`](crates/store) | Persistence and native indexes |
| [`vlajmaji-search`](crates/search) | Reusable queries and computed analyses |
| [`vlajmaji-data-lensisku-jbotci`](crates/data-lensisku-jbotci) | Optional unpublished Lensisku/jbotci embedded data package (local payload only) |
| [`vlajmaji-data-llg`](crates/data-llg) | Optional unpublished LLG embedded data package (local payload only) |
| [`vlajmaji-data-la-lojban`](crates/data-la-lojban) | Optional unpublished La-Lojban embedded data package (local payload only) |
| [`vlajmaji-data-sutysisku`](crates/data-sutysisku) | Optional unpublished Sutysisku embedded data package (local payload only) |
| [`vlajmaji-cli`](crates/cli) | The `vlajmaji` executable |

## Published 0.2.0 packages

Use Rust 1.98.0 or newer. Install the published 0.2.0 CLI with:

```sh
cargo +1.98.0 install vlajmaji-cli --version 0.2.0 --locked
vlajmaji --help
```

For a Rust library using the published release, add `vlajmaji = "0.2.0"` to
your dependencies and use `vlajmaji::{lexicon, store, search}`. The
[umbrella crate README](crates/vlajmaji/README.md) links the component crates and
describes the opt-in features.

## Build and try the current development checkout

For a source checkout, place [Bauske](https://github.com/oxur/bauske)
alongside this repository: phonology currently resolves through
`../bauske/crates/phonology`.

From this repository:

```sh
cargo +1.98.0 test --locked --workspace
cargo +1.98.0 run --locked -p vlajmaji-cli -- --help
```

Try the grouped-dictionary commands with a fresh fixture and store. The
subshell keeps the temporary data-home setting local to this example:

```sh
(
  set -eu
  run_dir=$(mktemp -d)
  export XDG_DATA_HOME="$run_dir/data"
  cargo +1.98.0 run --locked -p vlajmaji-lexicon --example dictionary_fixture -- "$run_dir/dictionary.json"
  cargo +1.98.0 run --locked -p vlajmaji-cli -- dictionary-build "$run_dir/dictionary.json"
  cargo +1.98.0 run --locked -p vlajmaji-cli -- dictionary-query \
    --request '{"operation":"exact","spelling":"farvi","options":{"page_size":2}}'
  cargo +1.98.0 run --locked -p vlajmaji-cli -- info
  cargo +1.98.0 run --locked -p vlajmaji-cli -- verify all
)
```

Grouped build, query, info, and verify share the optional explicit `STORE` or
default resolution: `$XDG_DATA_HOME/vlajmaji/stores/default` when
`XDG_DATA_HOME` is an absolute nonempty path, otherwise
`$HOME/.local/share/vlajmaji/stores/default`. This location is intended for a
future dictionary-of-dictionaries; the initial Lensisku-only store is
provisional. Build creates missing default parent directories and refuses to
replace an existing store. Query, info, and verify open only an existing store;
they never create or download one. Store creation completes full verification
before returning, while ordinary store open checks only the minimum readable
header. `info` reads counts and source summaries from SQL; query still decodes the
complete typed dictionary and builds its search indexes.

`verify` accepts six selective checks—`schema`, `metadata`, `hashes`, `model`,
`fks`, and `projection`—or `all` to attempt every check. The [schema
guide](docs/dictionary-schema.md) and [query guide](docs/dictionary-query.md)
cover the persisted structure, lookup behavior, default resolution, and errors.
The debug commands above demonstrate the fixture workflow; use the optimized
release CLI for representative runtime measurements, for example
`cargo +1.98.0 build --release --locked -p vlajmaji-cli`.

The response keeps competing definitions under their named sources, reports
complete totals before paging, and returns a continuation cursor when needed.
The [query guide](docs/dictionary-query.md) covers source filters, paging,
availability, stable references, and structured errors.

Compact exact/reference output is opt-in with `dictionary-query STORE
--format compact --request JSON`; configured lookup uses `lookup --format
compact --request JSON`. Both retain full output by default. Compact hits carry
strict detail keys that can be expanded with `dictionary-detail STORE
--reference JSON`, or `lookup-detail --reference JSON` for managed results.
The collection guide documents snapshot checking and the compact/detail fields.
Pass `--format readable` to `dictionary-query` or configured `lookup` for
deterministic source-grouped text, or to either detail command for a readable
complete-hit view. Full JSON remains the default; compact JSON keeps its
existing response shape.

Corpus data is not bundled. The [migration and replay guide](docs/migration.md)
describes importing research inputs, building a store, querying it, and running
the separately gated full-corpus test.

## Where this is going

Next come independently selectable corpus packages, multi-store composition,
and an editable dictionary of our own. The longer aim is a practical vocabulary
laboratory connected to Bauske's linguistic tools.

The workspace is on the **0.3.0 development cycle**; its APIs remain experimental.

## Development and releases

Run `make help` for build, test, lint, documentation, and publishing targets.
`make check` runs the core quality gates.
The Make helpers require Bash 3.2 or newer and Git; Python is not needed.
`make test-tools` runs the Bash tooling suite with disposable local Git remotes
and a Cargo stub, without building Rust or accessing the network.

`make build` builds and copies `vlajmaji` to `./bin`; `make build-release` does
the same for an optimized build. Set `INSTALL_DIR=./local-bin` to choose another
destination. To copy an existing build, run `make install` (or
`make install MODE=release` for release artifacts).

`make planning` sets up `.worktrees/planning` on the separate `planning` branch.
It fetches an existing branch from `origin`, or creates and pushes an empty
orphan branch if none exists. Later updates are fast-forward only; local
planning work is preserved.

Use `make publish-dry-run` to verify the workspace packages and `make publish`
to upload them to crates.io. Publish the Bauske dependency first. The packaged crates include their own READMEs and licenses.

## License

``` text
Apache License, Version 2.0
```

[//]: ---Named-Links---

[logo]: assets/images/v1-250.png
[logo-large]: assets/images/v1-1254.png
[gh-actions-badge]: https://github.com/oxur/vlajmaji/actions/workflows/ci.yml/badge.svg
[gh-actions]: https://github.com/oxur/vlajmaji/actions/workflows/ci.yml
[version]: https://github.com/oxur/vlajmaji/blob/main/.github/workflows/ci.yml
[github-tags]: https://github.com/oxur/vlajmaji/tags
[github-tags-badge]: https://img.shields.io/github/tag/oxur/vlajmaji.svg
[crate]: https://crates.io/crates/vlajmaji
[crate-badge]: https://img.shields.io/crates/v/vlajmaji.svg
[docs]: https://docs.rs/vlajmaji/
[docs-badge]: https://img.shields.io/badge/rust-documentation-blue.svg
