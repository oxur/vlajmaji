# vlajmaji-lexicon

This checkout is on the 0.3.0 development cycle. The dependency and installation
examples below target the published 0.2.0 release.

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

The pure `import_jbotci_export` adapter retains both pinned captures and ordered
native fields while exposing a conservative typed projection. Its bounded
metadata grammar, field map, identity rules, and local replay example are in the
[jbotci import guide](https://github.com/oxur/vlajmaji/blob/main/docs/jbotci-import.md).

The additive `import_llg_lists` adapter accepts the complete gismu, cmavo,
assigned rafsi, and notice byte inputs with caller-supplied SHA-256 pins. It
preserves each physical list line as an occurrence and every padded source
field while projecting only supplied keywords, hints, definitions, and forms.
It does not infer part of speech or linguistic structure, and hash checks do
not authenticate a publisher. This is a 0.3.0 development-checkout API; it is
not present in the published 0.2.0 package. See the [current source module](src/llg.rs)
and the [CLI guide](https://github.com/oxur/vlajmaji/tree/main/crates/cli) for
the local fixture and materialization commands.

The `pack` module seals canonical dictionary bytes with a strict versioned
integrity inventory and loads them only through the normal registry and model
validators. `compose_dictionaries` consumes validated models, preserving
first-seen record order, deduplicating exact equal identities, and rejecting
conflicts rather than choosing a semantic winner. Seals are local integrity
records, not publisher authentication or redistribution permission.

The development checkout also exposes `import_la_lojban_export` for the pinned
La-Lojban English Lensisku JSON export. It retains the complete byte snapshot,
ordered native fields, source-row occurrences, and native assertion-ID lexemes
while sharing the conservative Lensisku projection with jbotci. Its report
contains observed counts only; no upstream count is inferred. The La-Lojban
distributor is recorded separately from Lensisku, and content redistribution
remains held because the software license does not establish dataset terms.
This API is not part of the published 0.2.0 package; see the current source
module [`la_lojban.rs`](src/la_lojban.rs).
