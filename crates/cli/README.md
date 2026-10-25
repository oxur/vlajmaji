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

The additive `dictionary-build INPUT.json [STORE]` command builds a
validated grouped-dictionary store from versioned dictionary JSON.
`dictionary-query [STORE] [--request JSON]` accepts exact or typed-reference
requests on the argument or stdin. `info [STORE]` reports counts for either the
grouped dictionary or the legacy archive store. When `STORE` is omitted, these
commands use `$XDG_DATA_HOME/vlajmaji/stores/default`; if XDG data home is unset,
empty, or relative, they fall back to
`$HOME/.local/share/vlajmaji/stores/default`. An explicit store always wins.
The default location is intended for a future dictionary-of-dictionaries; the
initial Lensisku-only store is provisional. Build creates missing default parent
directories privately but never replaces an existing store. Query and info do
not create or download a store. New-command runtime errors emit a structured
JSON object to stderr, no stdout, and a nonzero exit. See the
[grouped dictionary query guide](https://github.com/oxur/vlajmaji/blob/main/docs/dictionary-query.md).

`jbotci-import RAW METADATA FRESH.json` locally normalizes a hash-pinned export;
`jbotci-check RAW METADATA [--release RELEASE.json]` validates it without
writing. `make jbotci-acquire`, `jbotci-replay`, `jbotci-check`, and
`jbotci-update` provide the explicit pinned local workflow; each requires a
caller-chosen cache root and fresh release/store paths where applicable. See
the [jbotci import guide](https://github.com/oxur/vlajmaji/blob/main/docs/jbotci-import.md)
for pins, preservation, identity, metadata, and content-notice limits.

Corpus data is not bundled. See the [data and replay guide](https://github.com/oxur/vlajmaji/blob/main/docs/migration.md) for the current research-input workflow. Licensed under Apache-2.0.
