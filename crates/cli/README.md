# vlajmaji-cli

This checkout is on the 0.3.0 development cycle. The dependency and installation
examples below target the published 0.2.0 release.

The `vlajmaji` JSON command for [Vlajmaji](https://github.com/oxur/vlajmaji)'s dictionary workbench.

For a source checkout, run these commands from the workspace root
with Bauske checked out alongside Vlajmaji:

```sh
make build
./bin/vlajmaji --help
```

Use `make build-release` for an optimized build, or override `INSTALL_DIR` to
choose where the binary is copied. To install the published command, run
`cargo install vlajmaji-cli --version 0.2.0 --locked`. Version 0.1.0 used
the previous `relojban-search` command.

Requires Rust 1.98 or newer. Commands build a new SQLite store directory from a lexical archive, report corpus counts, and accept query JSON through an argument or standard input. Compound analysis and generation are enabled in the CLI.

The additive `dictionary-build INPUT.json FRESH_STORE` command builds a
validated grouped-dictionary store from versioned dictionary JSON.
`dictionary-query STORE [--request JSON]` accepts exact or typed-reference
requests on the argument or stdin. New-command runtime errors emit a structured
JSON object to stderr, no stdout, and a nonzero exit. See the
[grouped dictionary query guide](https://github.com/oxur/vlajmaji/blob/main/docs/dictionary-query.md).

`jbotci-import RAW METADATA FRESH.json` locally normalizes a hash-pinned export;
`jbotci-check RAW METADATA [--release RELEASE.json]` validates it without
writing. See the [jbotci import guide](https://github.com/oxur/vlajmaji/blob/main/docs/jbotci-import.md)
for preservation, identity, metadata, and content-notice limits.

Corpus data is not bundled. See the [data and replay guide](https://github.com/oxur/vlajmaji/blob/main/docs/migration.md) for the current research-input workflow. Licensed under Apache-2.0.
