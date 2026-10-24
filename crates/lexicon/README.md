# vlajmaji-lexicon

Source-preserving lexical records and archives for [Vlajmaji](https://github.com/oxur/vlajmaji).

The library retains raw source bytes, provenance, record identities, alternative assertions, and source-native form roles. It supports La-Lojban and Lensisku JSON plus LLG root, cmavo, and rafsi lists, keeping caller proposals separate from captured records.

```toml
[dependencies]
vlajmaji-lexicon = "0.2.0"
```

Requires Rust 1.98 or newer. This package contains code and synthetic fixtures; dictionary corpora are not bundled, and their content terms remain separate from the code license.

See the [API documentation](https://docs.rs/vlajmaji-lexicon) for examples. Licensed under Apache-2.0.

## Source registry

The additive `SourceRegistry` API binds stable source identities and optional
contributor/distributor records to captured occurrences without merging source
rows. See the [source registry guide](https://github.com/oxur/vlajmaji/blob/main/docs/source-registry.md) and run
`cargo +1.98.0 run --locked -p vlajmaji-lexicon --example source_registry` for
the five-occurrence synthetic example. The registry is a sidecar; it leaves
legacy source metadata, raw bytes, archive versions, and record identities
unchanged.

## Versioned dictionary model

The additive `Dictionary` API stores exact raw captures and explicit typed metadata, validates the complete versioned JSON document, and adapts legacy archives without rewriting them. The adapter requires callers to provide the source registry and normalized metadata; it does not infer entries, senses, morphology, frames, etymology, or POS. Read the [dictionary model guide](https://github.com/oxur/vlajmaji/blob/main/docs/dictionary-model.md) and [standards mapping limits](https://github.com/oxur/vlajmaji/blob/main/docs/dictionary-standards-map.md). Run `cargo +1.98.0 run --locked -p vlajmaji-lexicon --example dictionary_fixture -- /tmp/dictionary-fixture.json` to write and reload the complete synthetic fixture at a fresh path.
