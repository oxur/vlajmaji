# vlajmaji-search

Reusable lexical queries for the [Vlajmaji](https://github.com/oxur/vlajmaji) dictionary workbench.

Queries include exact spelling, literal reverse-definition lookup, captured forms and links, spelling distance, bounded phonological contrasts, and prosodic filters. Results retain source identity and explanations. The optional `morphology` feature adds compound decomposition and generation through `latkerlo-jvotci`.

```toml
[dependencies]
vlajmaji-search = "0.1.0"
```

Requires Rust 1.98 or newer. Computed proposals and scores do not establish attested usage, semantic equivalence, or poetic merit. Corpora are not bundled.

See the [API documentation](https://docs.rs/vlajmaji-search). Licensed under Apache-2.0.
