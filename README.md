# Vlajmaji

[![Build Status][gh-actions-badge]][gh-actions]
[![Tags][github-tags-badge]][github-tags]

[![Project Logo][logo]][logo-large]

*(Re)Lojban dictionaries, schemas, libraries, and command-line tools*

Vlajmaji is a growing dictionary workbench for exploring words, their sources,
and the possibilities of combining them. Its aim is to bring multiple
lexicons together without losing who said what, then make that material useful
for language study, word creation, poetry, and song.

## Current proof of concept

The Rust workspace already provides:

- A lexical archive that preserves source snapshots, raw records, provenance,
  and alternative assertions, keeping computed analyses and user proposals separate.
- Rebuildable SQLite indexes and an optional Oxigraph backend for research comparisons.
- Exact and reverse-definition lookup, source-link traversal, spelling distance,
  and bounded phonological and prosodic filters.
- Optional compound decomposition and generation through `latkerlo-jvotci`.
- A JSON CLI, `vlajmaji`, for building stores and running queries.

The PoC has been exercised against 21,241 records from 16 source snapshots.
Search scores and generated compounds are experimental aids; they do not establish
attested usage, semantic equivalence, or poetic merit.

| Package | Role |
| --- | --- |
| [`vlajmaji-lexicon`](crates/lexicon) | Source-preserving lexical data and archives |
| [`vlajmaji-store`](crates/store) | Persistence and native indexes |
| [`vlajmaji-search`](crates/search) | Reusable queries and computed analyses |
| [`vlajmaji-cli`](crate/cli) | The `vlajmaji` executable |

## Try it

Use Rust 1.98.0 or newer. Check out [Bauske](https://github.com/oxur/bauske)
alongside this repository: phonology currently resolves through
`../bauske/crates/phonology`.

From this repository:

```sh
cargo +1.98.0 test --locked --workspace
cargo +1.98.0 run --locked -p vlajmaji-cli -- --help
```

Corpus data is not bundled. The [migration and replay guide](docs/migration.md)
describes importing research inputs, building a store, querying it, and running
the separately gated full-corpus test.

## Where this is going

Next comes a richer source-grouped dictionary model, scripted imports, optional
per-corpus data crates, and an editable dictionary of our own. The longer aim is
a practical vocabulary laboratory connected to Bauske's linguistic tools.

The workspace is on the **0.2.0 development cycle**; its APIs are experimental.
The latest published release is 0.1.0, whose CLI command is still `relojban-search`.

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
