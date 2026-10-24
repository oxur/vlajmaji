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
