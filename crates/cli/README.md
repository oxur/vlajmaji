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

`verify CHECK [STORE]` runs one explicit read-only store check (`schema`,
`metadata`, `hashes`, `model`, `fks`, `projection`, or `all`). Ordinary open,
query, and info check only the minimum header; they do not establish full store
integrity. `all` reports six ordered result rows. A failed check exits nonzero,
leaves stdout empty, and writes both its structured error and complete report
to stderr.

`collections register|install|list|inspect ROOT` manages an explicit local
source catalogue. Registration stores typed source and reference metadata
without copying release payloads. Installation creates one verified Unified
generation; it never changes the default store or source selection. Manifest
paths resolve beside the manifest, and selected inputs are read locally with
the existing pack adapters. Synthetic packs require `--allow-synthetic`.
See the [collection guide](../../docs/collections.md) for commands, strict
manifest fields, state semantics, and structured failures.

`jbotci-import RAW METADATA FRESH.json` locally normalizes a hash-pinned export;
`jbotci-check RAW METADATA [--release RELEASE.json]` validates it without
writing. `make jbotci-acquire`, `jbotci-replay`, `jbotci-check`, and
`jbotci-update` provide the explicit pinned local workflow; each requires a
caller-chosen cache root and fresh release/store paths where applicable. See
the [jbotci import guide](https://github.com/oxur/vlajmaji/blob/main/docs/jbotci-import.md)
for pins, preservation, identity, metadata, and content-notice limits.

Corpus data is not bundled. See the [data and replay guide](https://github.com/oxur/vlajmaji/blob/main/docs/migration.md) for the current research-input workflow. Licensed under Apache-2.0.

`data-pack fixture jbotci OUTPUT` prepares an explicitly synthetic three-row
pack through the real importer for local feature checks and CI.
`data-pack materialize jbotci RAW METADATA OUTPUT` verifies caller-supplied
pins and supplier accounting, then writes canonical release, seal, and report
files into a fresh directory. It only reads the paths supplied by the caller;
it does not fetch data, overwrite an output, or build a store. The `embedded`
data crate has no default payload and is unpublished; its ignored local data
must be prepared before selecting that feature. A synthetic fixture is not the
full source distribution.

The optional LLG list-family package uses the same local-only workflow:
`data-pack fixture llg OUTPUT` creates a Synthetic fixed-column example, while
`data-pack materialize llg GISMU CMAVO RAFSI NOTICE OUTPUT` reads four files
supplied by the caller and verifies their complete bytes against the default
published-pin hashes. `--gismu-sha256`, `--cmavo-sha256`, `--rafsi-sha256`,
`--notice-sha256`, and `--base-locator` override those defaults for a different
explicitly identified input set. The command never downloads files, replaces
an existing output directory, or writes a dictionary store. The full LLG
payload is not committed or published; its notice and redistribution status
remain separate from the Apache-2.0 code license.

The optional La-Lojban pack uses the same local-only workflow:
`data-pack fixture la-lojban OUTPUT` prepares a clearly Synthetic example, and
`data-pack materialize la-lojban RAW_JSON OUTPUT` checks the pinned complete
input before writing a Full local release, report, and seal. `--revision`,
`--raw-sha256`, `--raw-locator`, and `--notice-locator` identify the input and
its provenance. The new data package is unpublished, has no default payload,
and keeps content redistribution Held until its terms are verified.

The optional Sutysisku projection pack is prepared the same way:
`data-pack fixture sutysisku OUTPUT` creates a Synthetic example, while
`data-pack materialize sutysisku RAW_JSON OUTPUT` checks the caller-provided
complete-byte pin before writing a Full local release, report, and seal.
`--revision`, `--raw-sha256`, `--raw-locator`, and `--notice-locator` identify
the input and its provenance. The package is unpublished and has no default
payload; content redistribution remains Held because the upstream lexical
lineage and data terms are unresolved.
