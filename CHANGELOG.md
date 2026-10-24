# Changelog

## 0.3.0 (unreleased)

- Start the next development cycle.

## 0.2.0 (2026-10-24)

- Add the `vlajmaji` umbrella library with opt-in morphology and graph features.
- Add Linux/macOS CI, dependency caching, and build-helper checks.
- Move the CLI package into `crates/cli` alongside the library crates.
- Add grouped exact lookup, stable dictionary references, and the
  `dictionary-build` / `dictionary-query` CLI commands.
- Add a versioned raw-preserving dictionary model, explicit legacy archive adapter, fixture, and standards mapping limits.

- Add a validated source/contributor/distributor registry and borrowed attribution
  views over immutable captured occurrences.
- Add immutable relational `DictionaryStore` persistence with schema-1 projection
  and strict read-only reopen validation; keep the legacy searchable `SqliteStore`
  schema and API unchanged.
- Rename the CLI command from `relojban-search` to `vlajmaji`.
- Install built binaries to configurable `INSTALL_DIR` (default `./bin`).
- Adopt Oxurack-style coloured build feedback and grouped help.

## 0.1.0

- Initial publication of the migrated research PoC workspace.
- Preserve existing CLI names, source provenance, and modelling limits.
- Add package metadata, crate READMEs, and repository development/release targets.
