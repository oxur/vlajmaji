# vlajmaji-search

This checkout is on the 0.3.0 development cycle. The dependency and installation
examples below target the published 0.2.0 release.

Reusable lexical queries for the [Vlajmaji](https://github.com/oxur/vlajmaji) dictionary workbench.

Queries include exact spelling, literal reverse-definition lookup, captured forms and links, spelling distance, bounded phonological contrasts, and prosodic filters. Results retain source identity and explanations. The optional `morphology` feature adds compound decomposition and generation through `latkerlo-jvotci`.

The grouped dictionary API builds a fully verified `DictionaryStore` or opens
its typed draft without claiming full-store verification. Query construction
checks lookup identities needed by the query engine but does not replace the
explicit store verification API,
performs byte-exact source-grouped spelling lookup with complete pre-page
counts, and resolves typed local/catalogue references. Its responses retain
source, release, occurrence, editorial, and computed evidence as separate
fields. See the [grouped lookup guide](https://github.com/oxur/vlajmaji/blob/main/docs/dictionary-query.md) and run
`cargo run --example dictionary_lookup -- INPUT_JSON FRESH_STORE` for a public
API walkthrough.

For an explicit composed view, `DictionarySearchEngine::open_snapshot` accepts
a `DictionaryReadTopology` and a frozen `DictionarySnapshotManifest` from
`vlajmaji-store`. The same engine can read one unified store, fan out across
member stores, or use read-only SQLite `ATTACH`; each topology loads its own
typed document and performs its own candidate queries. Snapshot opening checks
member headers and expected recorded states, then checks the composed logical
digest. That typed draft is intentionally unverified: ordinary open does not
audit every projected SQL value. Use the store's explicit `All` verification
before relying on full persisted projection integrity. The attach topology
requires UTF-8 paths and reports unsupported native paths as an error; separate
attached files do not gain atomic-publication semantics.

The manifest's generation and optional selected-release IDs bind runtime
selection and cursors without changing immutable release history. A changed
generation or selection represents a different query view. Profiled opening
separates provider setup, document load/composition/digest, and common search
index construction. Exact-query profiling measures SQL candidates and actual
hit hydration; remaining same-call time is a conservative lookup/order/page
residual. Serialization is outside the query call and must be added when
assessing complete response cost. A/F startup includes reading, composing,
serializing, and hashing its own logical document; those costs remain visible.

The `topology_probe` example is a local comparison harness, not user
configuration or an installer. `prepare --manifest FILE` strictly loads four
explicitly pinned sealed inputs and exclusively creates one unified and four
member stores under a fresh output root. `run` opens one declared topology and
emits one full response with a trailing newline. `batch` primes the same
long-lived provider, then measures a frozen request list, including exact
cursor pages and typed reference outcomes; each response is written to a fresh
file and reported with its byte count and SHA-256. Its manifest and request
files must be retained with the results so startup, query, hydration,
serialization, and write stages can be interpreted separately.

```toml
[dependencies]
vlajmaji-search = "0.2.0"
```

Requires Rust 1.98 or newer. Computed proposals and scores do not establish attested usage, semantic equivalence, or poetic merit. Corpora are not bundled.

See the [API documentation](https://docs.rs/vlajmaji-search). Licensed under Apache-2.0.
