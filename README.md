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

## What 0.2.0 provides

The Rust workspace already provides:

- A lexical archive that preserves source snapshots, raw records, provenance,
  and alternative assertions, keeping computed analyses and user proposals separate.
- A validated source registry for exact aliases, lineage, contributors, and
  attribution of preserved occurrences ([guide](docs/source-registry.md)).
- An immutable, versioned dictionary model with explicit source, owned,
  editorial, and computed evidence, plus an explicit legacy archive adapter
  ([model guide](docs/dictionary-model.md),
  [standards mapping limits](docs/dictionary-standards-map.md)).
- SQLite persistence for that dictionary model, with strict read-only reopen
  validation ([schema guide](docs/dictionary-schema.md)).
- Source-grouped exact lookup with source selection, complete pre-page totals,
  occurrence-based paging, and stable local/catalogue reference resolution
  ([query guide](docs/dictionary-query.md)).
- Rebuildable SQLite indexes and an optional Oxigraph backend for research comparisons.
- Exact and reverse-definition lookup, source-link traversal, spelling distance,
  and bounded phonological and prosodic filters.
- Optional compound decomposition and generation through `latkerlo-jvotci`.
- A JSON CLI, `vlajmaji`, including `dictionary-build` and `dictionary-query`
  alongside the existing archive/search commands.
- A lossless local jbotci export normalizer with `jbotci-import` and
  read-only `jbotci-check` commands and an explicit pinned acquire/replay/check/
  update workflow ([import guide](docs/jbotci-import.md)).
- An umbrella library, `vlajmaji`, exposing `lexicon`, `store`, and `search`, with
  optional `morphology` and `graph` features.

The earlier archive/search PoC was exercised against 21,241 records from 16
source snapshots. The new grouped-dictionary path has a reproducible fixture
and public API/CLI examples; the jbotci adapter has a pinned local replay.
Other upstream dictionary imports remain future work.
Search scores and generated compounds are experimental aids; they do not establish
attested usage, semantic equivalence, or poetic merit.

| Package | Role |
| --- | --- |
| [`vlajmaji`](crates/vlajmaji) | Umbrella library re-exporting lexicon, store, and search, with opt-in morphology and graph features |
| [`vlajmaji-lexicon`](crates/lexicon) | Source-preserving lexical data and archives |
| [`vlajmaji-store`](crates/store) | Persistence and native indexes |
| [`vlajmaji-search`](crates/search) | Reusable queries and computed analyses |
| [`vlajmaji-cli`](crates/cli) | The `vlajmaji` executable |

## Try it

Use Rust 1.98.0 or newer. Install the published CLI with:

```sh
cargo +1.98.0 install vlajmaji-cli --version 0.2.0 --locked
vlajmaji --help
```

For a Rust library, add `vlajmaji = "0.2.0"` to your dependencies and use
`vlajmaji::{lexicon, store, search}`. The
[umbrella crate README](crates/vlajmaji/README.md) links the component crates and
describes the opt-in features.

For a source checkout, place [Bauske](https://github.com/oxur/bauske)
alongside this repository: phonology currently resolves through
`../bauske/crates/phonology`.

From this repository:

```sh
cargo +1.98.0 test --locked --workspace
cargo +1.98.0 run --locked -p vlajmaji-cli -- --help
```

Try the grouped-dictionary commands with a fresh fixture and store:

```sh
run_dir=$(mktemp -d)
cargo +1.98.0 run --locked -p vlajmaji-lexicon --example dictionary_fixture -- "$run_dir/dictionary.json"
cargo +1.98.0 run --locked -p vlajmaji-cli -- dictionary-build "$run_dir/dictionary.json" "$run_dir/store"
cargo +1.98.0 run --locked -p vlajmaji-cli -- dictionary-query "$run_dir/store" \
  --request '{"operation":"exact","spelling":"farvi","options":{"page_size":2}}'
```

The response keeps competing definitions under their named sources, reports
complete totals before paging, and returns a continuation cursor when needed.
The [query guide](docs/dictionary-query.md) covers source filters, paging,
availability, stable references, and structured errors.

Corpus data is not bundled. The [migration and replay guide](docs/migration.md)
describes importing research inputs, building a store, querying it, and running
the separately gated full-corpus test.

## Where this is going

Next come independently selectable corpus packages, multi-store composition,
and an editable dictionary of our own. The longer aim is a practical vocabulary
laboratory connected to Bauske's linguistic tools.

The workspace is on the **0.3.0 development cycle**; its APIs remain experimental.
The latest published release is **0.2.0**.

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
