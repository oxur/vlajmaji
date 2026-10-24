# Slice04 composed-search prototype evidence

Date: 2026-10-23. CC candidate; independent acceptance is pending.

The implementation adds `crates/relojban-search` and `crates/relojban-search-cli`, plus only the documented store identity helper/test exception. The endpoint report is [research/lexical-search-findings.md](../../../research/lexical-search-findings.md). Archive/import, captured source and immutable S01/S02/S03 evidence remain in their existing locations.

The `raw/strsim-0.11.1/` folder preserves the exact already-cached package archive, Cargo manifest and MIT notice with SHA-256 checksums. Its README records the copy time and unknown original request time; it is not represented as a fresh network capture.

## Full-population input and replay

The accepted v2 archive identity is 28,320,417 bytes, SHA-256 `4e5c30e978bde631349e1107599f3ef9738c1f63257ba280e146d2256c7cef5a`; it contains 21,241 records, 16 snapshots and 17,181 distinct spellings. The first refinement02 replay rebuilt that exact archive from the immutable S02 corpus input manifest and input bytes, built SQLite at a fresh path, and ran the full oracle. Its importer, build, JSONL output and stderr are captured as `probes/import-corpus-iteration02.json`, `probes/build-store-iteration02.json`, `probes/full-population-iteration02.jsonl`, and `probes/full-population-iteration02.stderr`. The earlier prefix-only run `probes/full-population-final-current.log` remains unchanged as historical evidence.

Run these commands from the repository root. They write only to a newly created temporary directory; the copied consumer has its own manifest, lockfile and workspace boundary, and the commands run it from that consumer directory:

```sh
RUN_DIR="$(mktemp -d "${TMPDIR:-/tmp}/p02a02-s04-replay.XXXXXX")"
cargo +1.98.0 run --locked -p relojban-lexicon --example import_corpus -- "$PWD" "$RUN_DIR/corpus.archive" > "$RUN_DIR/import.json"
shasum -a 256 "$RUN_DIR/corpus.archive"
cargo +1.98.0 run --locked -p relojban-search-cli -- build "$RUN_DIR/corpus.archive" "$RUN_DIR/search.sqlite" > "$RUN_DIR/build.json"
cargo +1.98.0 run --locked -p relojban-search --features morphology --example full_population_probe -- "$RUN_DIR/corpus.archive" "$RUN_DIR/search.sqlite" > "$RUN_DIR/full-population.jsonl" 2> "$RUN_DIR/full-population.stderr"
(
  cd experiments/E03-lexical-data-search/slice04/probes/consumer
  CARGO_TARGET_DIR="$RUN_DIR/consumer-target-default" cargo +1.98.0 run --locked --manifest-path "$PWD/Cargo.toml" -- "$RUN_DIR/search.sqlite"
  CARGO_TARGET_DIR="$RUN_DIR/consumer-target-morphology" cargo +1.98.0 run --locked --features morphology --manifest-path "$PWD/Cargo.toml" -- "$RUN_DIR/search.sqlite"
)
```

The recorded importer result confirms the archive hash above. The probe compares native IDs and edge sets against `scan_reference`; sound/component/composed operations compare complete independently constructed sorted ID vectors, cardinality, duplicates, distinct spellings, exclusion partitions and limit-zero/seven projections. Expected source classes are read directly from source-format fields and P01/Latkerlo APIs, not the search mapper. Generation separately joins every input and output spelling through public Exact and Form queries and compares forward/reversed output and score to direct `get_lujvo_with_analytics` calls under all three settings. Consumer CWD is `experiments/E03-lexical-data-search/slice04/probes/consumer`; its `Cargo.lock` is used with `--locked`. The initial malformed manifest-path attempt is preserved at `probes/consumer/consumer-manifest-path-error-iteration02.log` and is not treated as a consumer result.

## Preserved attempts

- `probes/full-population-run.*` records the original invocation rejected by the then-present 1,000-result cap; the API cap has since been removed, and the iteration02 replay requests the complete source sets.
- `probes/full-population-precompact.*` records the first successful but oversized per-record morphology diagnostic output.
- `probes/full-population-prior-class-map.*` records the compact run before format-aware source-class reconciliation. Its La-subset counts are superseded, not deleted.
- `probes/full-population-class-map.*` records the first successful full class census and earlier full scans; `full-population-final-current.log` is the final rerun after adding the composed/proposal oracles.
- `probes/full-corpus-queries.jsonl` preserves the interrupted per-invocation CLI sweep. Reopening and revalidating the full corpus per process took about 17.5 seconds, so later CLI contract coverage uses the bounded integration suite plus a one-open full API batch.
- `probes/workspace-doc-default.log` records Cargo's failed combined rustdoc output for a library and same-named binary. Final docs succeeded with the CLI package documented separately.
- The first current-probe invocation omitted the required morphology feature; its exact observed stderr is retranscribed in `probes/full-population-no-feature.attempt.md`, then corrected with `--features morphology`. The later successful run remains in `probes/full-population-final-current.log`.

## Final validation commands

All used Rust 1.98.0. The commands below completed successfully after the final source-class mapping and CLI/consumer test changes:

```sh
cargo +1.98.0 fmt --all --check
cargo +1.98.0 check --workspace
cargo +1.98.0 check --workspace --all-features
cargo +1.98.0 test --workspace
cargo +1.98.0 test --workspace --all-features
cargo +1.98.0 clippy --workspace --all-targets -- -D warnings
cargo +1.98.0 clippy --workspace --all-targets --all-features -- -D warnings
CARGO_TARGET_DIR=/tmp/relojban-s04-workspace-doc-default RUSTDOCFLAGS='-D warnings' cargo +1.98.0 doc --workspace --no-deps --exclude relojban-search-cli
CARGO_TARGET_DIR=/tmp/relojban-s04-workspace-doc-all RUSTDOCFLAGS='-D warnings' cargo +1.98.0 doc --workspace --all-features --no-deps --exclude relojban-search-cli
CARGO_TARGET_DIR=/tmp/relojban-s04-cli-doc-default RUSTDOCFLAGS='-D warnings' cargo +1.98.0 doc -p relojban-search-cli --no-deps
CARGO_TARGET_DIR=/tmp/relojban-s04-cli-doc-target RUSTDOCFLAGS='-D warnings' cargo +1.98.0 doc -p relojban-search-cli --all-features --no-deps
cargo +1.98.0 test -p relojban-search --no-default-features
cargo +1.98.0 clippy -p relojban-search --no-default-features --all-targets -- -D warnings
```

The standalone consumer in `probes/consumer` has its own lockfile and `[workspace]` boundary. Its iteration02 logs (`consumer-default-iteration02b.log`, `consumer-morphology-iteration02b.log`) show fresh-store success with an external `Debug` bound assertion. The initial doubled manifest-path error is retained at `probes/consumer/consumer-manifest-path-error-iteration02.log`. CLI behavior is covered by `crates/relojban-search-cli/tests/cli_contract.rs`.

Iteration02 Rust1.98 gate logs are named `*-iteration02.log`: format, workspace check/test/Clippy default and all-features, isolated no-default search test/Clippy, and split rustdoc with isolated target directories. Default/all-feature workspace tests passed 86/22 and 88/22 suites, respectively; isolated no-default search passed 10/3 suites.

`manifest.json` records SHA-256 and byte sizes for the scoped source files and probe evidence. Run `python3 experiments/E03-lexical-data-search/slice04/verify_manifest.py` from the repository root to verify those bytes. The archive identity is recorded separately in the manifest and remained in `/tmp`.

No archive, database, Cargo target, or generated binary is part of the source commit. See the delegated planning receipt for the six-row proposed disposition and current source commit.
