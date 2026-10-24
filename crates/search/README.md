# vlajmaji-search

Reusable lexical queries for the [Vlajmaji](https://github.com/oxur/vlajmaji) dictionary workbench.

Queries include exact spelling, literal reverse-definition lookup, captured forms and links, spelling distance, bounded phonological contrasts, and prosodic filters. Results retain source identity and explanations. The optional `morphology` feature adds compound decomposition and generation through `latkerlo-jvotci`.

The next development version is 0.2.0. Until it is published, use a local
path dependency on this crate; the version below describes the upcoming release.

The grouped dictionary API builds or reopens a validated `DictionaryStore`,
performs byte-exact source-grouped spelling lookup with complete pre-page
counts, and resolves typed local/catalogue references. Its responses retain
source, release, occurrence, editorial, and computed evidence as separate
fields. See the [grouped lookup guide](../../docs/dictionary-query.md) and run
`cargo run --example dictionary_lookup -- INPUT_JSON FRESH_STORE` for a public
API walkthrough.

```toml
[dependencies]
vlajmaji-search = "0.2.0"
```

Requires Rust 1.98 or newer. Computed proposals and scores do not establish attested usage, semantic equivalence, or poetic merit. Corpora are not bundled.

See the [API documentation](https://docs.rs/vlajmaji-search). Licensed under Apache-2.0.
