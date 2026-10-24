# E02 — Project02 resource discovery

This is the reproducible evidence home for Project02 Arc01. Slice01 acquired and inspected the exact packages and bounded dictionary captures; Slice02 then executed isolated library probes and the unchanged prosody CLI. Neither slice certifies word legality or Relojban conformance, chooses a project license, or adopts a production dependency.

## Run and verify

From the repository root, using Python 3 standard library only:

```sh
python3 experiments/E02-resource-discovery/scripts/verify-manifest.py
python3 experiments/E02-resource-discovery/scripts/inspect.py
python3 archive/verify.py
shasum -a 256 "/Users/oubiwann/Library/CloudStorage/Dropbox/cowboys.n.beans/Books/Bo Ge's Poetry/poems/004-the-last-snow.md"
```

The first command checks all captured raw files against `manifest.json`, resolves the latest stable non-yanked package row from the saved sparse index, validates each crate SHA256 and archive path, and checks pinned Git blob identities. The offline inspector uses source-specific minimum structure checks for La-Lojban, Sutysisku and Lensisku API JSON, rejects duplicate JSON keys before they can be collapsed, and preserves per-row missing/empty quality witnesses. It parses the captured cmavo column layout and rejects an all-malformed short-row payload. The inspector writes `derived/inventory.json`, `derived/population-summary.json`, and `derived/negative-controls.json`; all complete matching records and definition bodies are in the inventory. No acquisition command is needed for replay.

## Inputs and identities

- `raw/registry/` contains the actual sparse-index responses and headers.
- `raw/crates/` contains the exact registry archives plus response headers.
- `raw/lexical/` contains LLG lists, pinned English distributions, twelve bounded Lensisku word responses, the status-aware absent sentinel, and the unusable Jbovlaste challenge response.
- `raw/notices/` contains pinned repository trees, source/API documents, LLG's notice, response headers, and failed primary-route evidence.
- `manifest.json` records bytes, hashes, revisions, resolvable source locators, selected registry checksums, and Git blob identities. It records the known acquisition day but not per-request local times; adjacent HTTP `Date` headers are response/cache dates. `attempts.md` distinguishes recorded request URLs from canonical routes reconstructed from pinned source and marks final redirects unknown where they were not retained.

The offline query set is exactly `snime`, `xrula`, `tricu`, `slabu`, `citno`, `cpana`, `kalri`, `banro`, `farvi`, `ricrprunu`, `flaume`, `rutrprunu`, plus `PROJECT02_ABSENT_SENTINEL`. The Lensisku API responses remain mixed-language because `langid=2` is a preference. The whole-dataset counts derive from the complete captured list/export payloads; API counts are explicitly limited to the twelve captured responses.

See [the resource map](../../research/wordcraft-resources.md) for what each source actually represents, field boundaries, notices, the twelve-entry comparison, Slice02 findings and limits. Raw captures are immutable; a new fetch must be a new attempt, not an overwrite.

## Slice02 library probes and CLI bridge

The isolated [Rust probe package](rust-probes/README.md) resolves the preserved archives at their exact versions and records a separate lockfile. Its `default` feature explicitly forwards `vlazba/default`; its lean mode uses `--no-default-features`. Runs and stderr are stored in `probes/runs/`; the offline exact candidate-to-dictionary join is reproducible with `python3 experiments/E02-resource-discovery/probes/recipe/join-candidates.py` from the repository root. The derived join preserves occurrence provenance, whole-record source matches, and capture-scoped unmatched statuses. The unchanged CLI specimen is replayed with `python3 experiments/E02-resource-discovery/probes/recipe/run-prosody.py` after its documented build. These outputs test package APIs and spelling-level phonology only; see Slice02 CC evidence and closing report for actual outcomes and limits.
