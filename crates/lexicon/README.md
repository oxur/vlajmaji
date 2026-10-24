# vlajmaji-lexicon

Source-preserving lexical records and archives for [Vlajmaji](https://github.com/oxur/vlajmaji).

The library retains raw source bytes, provenance, record identities, alternative assertions, and source-native form roles. It supports La-Lojban and Lensisku JSON plus LLG root, cmavo, and rafsi lists, keeping caller proposals separate from captured records.

The next development version is 0.2.0. Until it is published, use a local
path dependency on this crate; the version below describes the upcoming release.

```toml
[dependencies]
vlajmaji-lexicon = "0.2.0"
```

Requires Rust 1.98 or newer. This package contains code and synthetic fixtures; dictionary corpora are not bundled, and their content terms remain separate from the code license.

See the [API documentation](https://docs.rs/vlajmaji-lexicon) for examples. Licensed under Apache-2.0.

## Source registry

The additive `SourceRegistry` API binds stable source identities and optional
contributor/distributor records to captured occurrences without merging source
rows. See the [source registry guide](../../docs/source-registry.md) and run
`cargo +1.98.0 run --locked -p vlajmaji-lexicon --example source_registry` for
the five-occurrence synthetic example. The registry is a sidecar; it leaves
legacy source metadata, raw bytes, archive versions, and record identities
unchanged.
