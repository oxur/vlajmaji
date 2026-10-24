# Lexical storage comparison (Slice03)

This report compares the reusable embedded stores implemented in `crates/relojban-lexicon-store` against the accepted v2 import archive. The SQLite store is the measured baseline; the optional Oxigraph store uses its native disk/RocksDB path. These are application-bundle measurements, not isolated engine benchmarks or production-adoption decisions.

## Reusable surface and data contract

`SqliteStore` is the default backend. `GraphStore` is available behind the `graph` feature. Both expose the same finite typed query/result API and typed errors, take a caller-owned fresh path, and preserve the exact accepted archive with its SHA-256. Native projections hold every record identity and indexed field/link data; reopen checks schema version, input hash and completion state. The archive remains authoritative for complete source reconstruction and all 16 raw snapshots. Stable source identity uses length-prefixed tuple components; proposals retain separate origin identity. The only core API addition is a documented borrowed `Lexicon::proposals()` view.

The corpus used for this comparison contains 21,241 records from 16 snapshots: 19,696 definition-bearing rows, 1,545 forms-only rows, 3,091 assigned forms, 1,988 source-reported forms and 13,521 literal links. All captured languages remain present. Exact spelling compares source bytes; definition substring searches use an ASCII-lowercase derived view while preserving raw text. Gloss-only rows do not count as definitions. Empty definition queries return an empty set.

SQLite uses parameterized queries and FTS5 trigram `case_sensitive=1` with quoted MATCH text plus an exact substring postfilter. Queries shorter than three Unicode scalar values and NUL-containing/unsupported literals take the visible parameterized SQL fallback. Graph queries use typed literal substitutions and the same lowercase text projection; field absence is a generic sparse `MINUS` query. Cross-reference queries return IDs only through actual captured edges (no reflexive start row), while preserving every edge at each allowed path length, including a cycle-closing edge that reaches the start and unresolved second-hop edges. Depths outside one and two return empty results. Neither backend normalizes spelling or derives semantic links.

## Correctness and persistence

The release harness compares complete ID and edge-origin sets with an independent Rust scan oracle. Both backends passed all 61 fixed corpus queries in a separate process after reopen under the corrected non-reflexive traversal contract. A fixture with left→right, right→left and right→absent proves two returned IDs plus all three depth-two edges; a no-edge start returns empty IDs/edges. This includes exact `si'e` versus `sie`, short/punctuation definition terms, form roles, Missing/Null/Value fields (including an arbitrary field name), link depths one and two, and proposal controls. Both also passed the separately labelled fixture controls for apostrophe variants, phrases, source-field presence, hostile-looking literals and a hypothetical proposal. A preserved interim separate-process attempt had mistakenly treated traversal as reflexive; it is explicitly pre-correction evidence only. The final fixture and corpus open apply the non-reflexive rule: a no-edge start returns empty IDs/edges, and a cycle-closing start is returned only after an actual edge reaches it. Fixture records are test data, not dictionary attestations.

Creation writes a canonical archive/hash and a completion marker. Reopen reconstructs the exact archive and all 16 source-byte identities. Public API controls exercised here cover initialized-path refusal, invalid archive input, and rejection of absent/incomplete stores. Schema-version and archive-hash corruption behavior is implemented in open checks but was not separately induced in these slice tests. Both public concrete stores implement bounded Debug that reports their caller path/backend without dumping archive contents; a compile-bound fixture proves both Debug implementations. A standalone consumer outside the workspace, passed a fresh caller-selected STORE_ROOT, created, queried and reopened both public backends. These checks establish observed behavior for this implementation and environment, not crash/power-loss guarantees.

## Controlled observations

The iteration02 controlled series used Rust 1.98.0, the locked workspace, release mode, rusqlite 0.40.2 with bundled SQLite, and Oxigraph 0.5.13 with native RocksDB. It reused the same-schema stores without reingesting because the repair changed only query semantics. Archive SHA-256: `4e5c30e978bde631349e1107599f3ef9738c1f63257ba280e146d2256c7cef5a`; 21,241 records; fixed 61-query suite. The current code/config identity was recorded before and after the single series and matched: Git base `aeba87ab2f56433fa0a411729ba918198d655036`; scoped tracked source/config diff SHA-256 `b40f71262d3cdef79f24f6dd42cab2b1305ef597b6ee3a63ede2e4618c4367dd`; query-function SHA-256 `158ff323e76fbc4cf06787c165ee24fbb57730572de25094d755e2088667d712`; exact per-file hashes are in `iteration02-benchmark-identity-before.json` and `...-after.json`. The `native_probe.rs` harness SHA-256 was `177329f245b1564a57e1eaaa75a712833e141957b9511f433ea6f1885e473b0a`.

The corrected separate-process reopen completed all 61 queries: SQLite setup 1,152 ms and batch 4,217 ms; Oxigraph setup 1,306 ms and batch 16,008 ms. The one current-code controlled series used one warmup and five alternating paired batches; every batch checked full IDs/edges against the scan oracle. SQLite warmup was 4,763 ms and measured batches were 4,096, 4,749, 4,203, 5,786 and 4,091 ms (median 4,203 ms; range 4,091–5,786 ms). Oxigraph warmup was 15,743 ms and measured batches were 15,478, 15,884, 15,978, 15,385 and 15,343 ms (median 15,478 ms; range 15,343–15,978 ms). Reopen in the benchmark process took 1,189/1,314 ms. After flush and close, the SQLite directory measured 194,306,048 bytes and the Oxigraph directory 194,474,716 bytes. The command elapsed 128.086 seconds including warm execution overhead.

Earlier create/setup timings and both earlier trial arrays remain preserved as historical records. The pre-iteration02 arrays are not attributed to a contemporaneous complete code/config fingerprint; the post-close earlier array is not the current query-code measurement. Full previous raw observations are in `native-bench-before-close.json` and `native-bench-final.json`. Original 30-minute cumulative cap compliance is UNKNOWN because active durations for some stopped attempts were not captured. The separately authorized iteration02 repair phase accumulated 211.300 seconds across recorded fixture, consumer, open, benchmark and preserved interim/failing commands (including compilation overhead), below its 360-second bound. No reingest or second current benchmark was run.

These results recommend SQLite as the Slice04 prototype baseline for this bounded workload. They do not isolate engine causality, measure memory, model concurrent users, or establish a universal latency winner.

Slice04's composed search library reuses the archive-backed SQLite store for native retrieval and scans source-preserving records for spelling, P01 sound and ordered components. See [the current endpoint report](lexical-search-findings.md) and [Slice04 evidence inventory](../experiments/E03-lexical-data-search/slice04/README.md). The measured storage comparison remains bounded to its recorded 61-query suite and does not make a general engine-selection or deployment decision.
## Other options and terms

The captured SQLite FTS5 documentation describes trigram tokenization and MATCH syntax; PostgreSQL 18 `pg_trgm` documentation notes its treatment of non-word characters, so it would not supply the byte-exact spelling contract by itself. PostgreSQL text-search configuration is normalization-oriented. SQLx 0.8.6 source was inspected for shared-service integration context only. No PostgreSQL server, SQLx integration, pgvector workload or deployed service was tested here.

The historical benchmark arrays are retained in both `native-bench-before-close.json` and `native-bench-final.json`; only the latter has a post-close footprint. The earlier arrays have no contemporaneous full code/config fingerprint and are not attributed to the current harness. Current-code timing uses a separately hashed harness after the Slice03 iteration02 query fixes.

The pinned rusqlite package is MIT; Oxigraph 0.5.13 offers MIT or Apache-2.0 terms. Those dependency notices do not select or change the project's license. Captured package/source identities, hashes and bounded primary-document sections are in the slice03 evidence directory. The response headers contain no original request URI; `experiments/E03-lexical-data-search/slice03/derived/source-url-map.md` labels reconstructed canonical locators and leaves original request provenance unresolved.

## Replay

From the repository root, with Rust 1.98.0 installed and dependencies available in Cargo's cache:

```sh
cargo +1.98.0 fmt --all -- --check
cargo +1.98.0 check --workspace --all-targets --locked --offline
cargo +1.98.0 check --workspace --all-targets --all-features --locked --offline
cargo +1.98.0 test --workspace --locked --offline
cargo +1.98.0 test --workspace --all-features --locked --offline
cargo +1.98.0 clippy --workspace --all-targets --locked --offline -- -D warnings
cargo +1.98.0 clippy --workspace --all-targets --all-features --locked --offline -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo +1.98.0 doc --workspace --no-deps --locked --offline
RUSTDOCFLAGS='-D warnings' cargo +1.98.0 doc --workspace --all-features --no-deps --locked --offline
```

The native create/open/bench phases and standalone consumer commands, exact corpus archive identity, captured inputs, raw logs and manifest verification are recorded in `experiments/E03-lexical-data-search/slice03/README.md`. Cargo target and native database files are intentionally outside the source tree.
