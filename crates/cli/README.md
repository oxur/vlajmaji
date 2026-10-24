# vlajmaji-cli

The `vlajmaji` JSON command for [Vlajmaji](https://github.com/oxur/vlajmaji)'s dictionary workbench.

For the 0.2.0 development checkout, run these commands from the workspace root
with Bauske checked out alongside Vlajmaji:

```sh
make build
./bin/vlajmaji --help
```

Use `make build-release` for an optimized build, or override `INSTALL_DIR` to
choose where the binary is copied. The published 0.1.0 package still installs
the previous `relojban-search` command.

Requires Rust 1.98 or newer. Commands build a new SQLite store directory from a lexical archive, report corpus counts, and accept query JSON through an argument or standard input. Compound analysis and generation are enabled in the CLI.

The additive `dictionary-build INPUT.json FRESH_STORE` command builds a
validated grouped-dictionary store from versioned dictionary JSON.
`dictionary-query STORE [--request JSON]` accepts exact or typed-reference
requests on the argument or stdin. New-command runtime errors emit a structured
JSON object to stderr, no stdout, and a nonzero exit. See the
[grouped dictionary query guide](../../docs/dictionary-query.md).

Corpus data is not bundled. See the [data and replay guide](https://github.com/oxur/vlajmaji/blob/main/docs/migration.md) for the current research-input workflow. Licensed under Apache-2.0.
