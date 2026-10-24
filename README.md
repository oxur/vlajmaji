# Vlajmaji

(Re)Lojban dictionaries, schemas, libraries, and command-line tools.

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
- A JSON CLI, `relojban-search`, for building stores and running queries.

The PoC has been exercised against 21,241 records from 16 source snapshots.
Search scores and generated compounds are experimental aids; they do not establish
attested usage, semantic equivalence, or poetic merit.

| Package | Role |
| --- | --- |
| [`vlajmaji-lexicon`](crates/lexicon) | Source-preserving lexical data and archives |
| [`vlajmaji-store`](crates/store) | Persistence and native indexes |
| [`vlajmaji-search`](crates/search) | Reusable queries and computed analyses |
| [`vlajmaji-cli`](crate/cli) | The `relojban-search` executable |

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

The 0.1.0 APIs are experimental and will evolve as the research develops.

## Development and releases

Run `make help` for build, test, lint, documentation, and publishing targets.
`make check` runs the core quality gates.

`make planning` sets up `.worktrees/planning` on the separate `planning` branch.
It fetches an existing branch from `origin`, or creates and pushes an empty
orphan branch if none exists. Later updates are fast-forward only; local
planning work is preserved.

Use `make publish-dry-run` to verify the workspace packages and `make publish`
to upload them to crates.io. Publish the Bauske dependency first. The packaged crates include their own READMEs and licenses.
