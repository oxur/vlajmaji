# Slice03 persistence comparison evidence

This directory holds raw pinned primary inputs, bounded derived extracts, source code for replay, exact logs and negative attempts. `raw/` captures are immutable evidence. The accepted corpus archive is generated outside the repository from the pinned Slice02 source; its SHA-256 is `4e5c30e978bde631349e1107599f3ef9738c1f63257ba280e146d2256c7cef5a` (28,320,417 bytes). It contains 21,241 records and 16 snapshots. The complete field/query/control contract and outcomes are summarized in `research/lexical-storage-comparison.md`; failed attempts and chronology are in `attempts.md`.

Response headers do not record the original request URI. The original request-URL mapping is unavailable; `derived/source-url-map.md` labels the known canonical source URLs reconstructed from captured identity, not attested request URIs. Captures were made 2026-10-23 UTC; the exact per-file start/end times are in `probes/docs-capture-times.txt`. Raw bodies are preserved byte-for-byte. `extract_primary_sections.py` reads the captured SQLite FTS5 and PostgreSQL 18 documentation to produce `derived/primary-source-sections.txt`. Package source archives, manifests and notices are pinned under `raw/packages/` and `raw/docs/`. SHA-256 and byte inventories are in `manifest.json`; run `python3 verify_manifest.py` from this directory.

## Native replay

Rust 1.98.0, the workspace lockfile, release mode and the accepted archive are required. The command below uses caller-selected fresh paths and never removes them:

```sh
cargo +1.98.0 run --release --locked --offline -p relojban-lexicon-store --features graph --example native_probe -- create CORPUS_ARCHIVE SQLITE_PATH GRAPH_PATH
cargo +1.98.0 run --release --locked --offline -p relojban-lexicon-store --features graph --example native_probe -- open CORPUS_ARCHIVE SQLITE_PATH GRAPH_PATH
cargo +1.98.0 run --release --locked --offline -p relojban-lexicon-store --features graph --example native_probe -- bench CORPUS_ARCHIVE SQLITE_PATH GRAPH_PATH
```

`create` builds each backend and compares the complete fixed query results to a separately computed scan oracle. `open` performs the same comparison from a separate process after validating persisted state. `bench` performs one warmup and five alternating backend batches; it checks all 61 result/edge sets against the precomputed oracle and measures logical file size only after both stores are flushed and dropped. The current-code corrected open and single controlled timing/footprint series are `probes/iteration02-native-open-final.json` and `probes/iteration02-bench-final.json`; stderr, source/config identity guards, and cumulative phase accounting are adjacent files. Earlier create/open and benchmark results remain historical, not retroactively fingerprinted.

The standalone public consumer has its own `consumer/Cargo.toml` and `consumer/Cargo.lock`. Choose a fresh caller-owned parent directory and pass it explicitly:

```sh
STORE_ROOT="$(mktemp -d /tmp/p02-s03-consumer.XXXXXX)"
cargo +1.98.0 run --manifest-path experiments/E03-lexical-data-search/slice03/consumer/Cargo.toml --locked --offline -- "$STORE_ROOT"
```

The program creates its own uniquely named child under that path and does not remove existing data. Captured lock and logs distinguish consumer resolution from workspace resolution.

The source is research code. It implements finite synchronous query/store APIs and is not a network service or production CLI. Native DBs, extracted corpus archives, build targets and temporary store paths are excluded from this tree.
