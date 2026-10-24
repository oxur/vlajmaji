# vlajmaji-cli

The `relojban-search` JSON command for [Vlajmaji](https://github.com/oxur/vlajmaji)'s dictionary workbench.

```sh
cargo install vlajmaji-cli --version 0.1.0 --locked
relojban-search --help
```

Requires Rust 1.98 or newer. Commands build a new SQLite store directory from a lexical archive, report corpus counts, and accept query JSON through an argument or standard input. Compound analysis and generation are enabled in the CLI.

Corpus data is not bundled. See the [data and replay guide](https://github.com/oxur/vlajmaji/blob/main/docs/migration.md) for the current research-input workflow. Licensed under Apache-2.0.
