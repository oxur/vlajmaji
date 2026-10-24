# Preserved attempt chronology

All times and detailed output remain in `probes/`; this index prevents corrected attempts from replacing earlier evidence.

- The initial dependency resolution/build attempts exposed the optional graph digest-family conflict. The initial resolver logs and lock-adjusted retry are retained. A graph-only `sha2` 0.11 alias/lock selection resolved spareval's digest edge while the core continues using exact SHA-256 0.10.9. A standalone consumer resolved its own graph lock successfully; this is not a universal fresh-resolver guarantee.
- Fixture runs with incorrect arguments and expected-output assumptions are retained (`fixture-test*.log`). The corrected fixture run exercises actual missing/null/empty native-field values, hostile-looking input and an explicitly hypothetical proposal.
- A dense graph representation materialized missing field rows and reached approximately 1.9 GiB before being stopped. It was replaced with a sparse representation. The dense attempt is not used for final size or performance claims.
- An early sparse SQLite run was interrupted before query completion; instrumentation and an index were added. The retained logs show that attempt and subsequent complete runs.
- The first graph query suite failed because one selected binding variable was not projected. The later 59-query probe then exposed a slow correlated absence query. A generic sparse `MINUS` formulation was tested against the same finite oracle and passed. No field-name-specific missing-query shortcut remains.
- A correct-set probe initially recorded `a` as link origin because the origin projection matched namespace text; source inspection established the literal origin as `a'a`. The source mapping was corrected, and the edge-origin sets were rechecked. Two additional Missing-field controls tested an existing `definition` name and arbitrary `uncaptured arbitrary field`.
- Store reopen first rejected the graph as incomplete because the completion marker was duplicated. The marker update was corrected and a separate-process reopen passed. Failed stores and corrected outputs remain separate.
- An earlier benchmark measured file size before close. Its JSON is retained as `native-bench-before-close.json`; the final comparison drops/closes the stores first and records the final logical sizes in `native-bench-final.json`.
- Final current-code release create and separate-process open each compared all 61 corpus queries with the independent full scan. Their setup/query timings are in `native-create-timing-final.json` and `native-open-timing-final.json`; they are correctness-run timings, not part of the five-trial benchmark.

Original-limit accounting correction: the exact cumulative active time of stopped attempts was not recorded. The original 30-minute cap compliance is UNKNOWN; the prior sentence that said it was kept below the cap is retained here as superseded, unsupported wording, not a result. Successful measured phase durations are in their JSON records, including the final create/open and the corrected post-close benchmark. The before-close and post-close arrays are both preserved; prior timing has no contemporaneous complete code/config fingerprint. No universal database or server-performance conclusion is inferred from these application-level observations.


## Iteration02 correction

Iteration02 fixes the reproduced cycle-closing/unresolved second-hop edge loss and raw-value state predicate mismatch; it adds bounded non-payload `Debug` for both stores, positive fixture/external consumer compile/run evidence, and a current-code controlled series. Earlier successful/failing arrays and raw attempts above remain historical evidence.


- Iteration02 initially carried a mistaken reflexive-root interpretation into a preserved separate-process attempt (`iteration02-open-pre-correction.log`). After correction, start IDs are included only when reached through a real source edge: no-edge depth1/2 returns empty, while a traversed cycle-closing edge may return the start. The final fixture and 61-query separate-process open pass under this rule; the old attempt is not final proof.
- Iteration02 added Missing/Null raw-value predicate corrections, bounded non-payload Debug implementations, a caller-root external-consumer run, and one current-code controlled series. Final Rust 1.98 fmt/check/test/clippy/doc gates and the unchanged input guards are recorded in adjacent `iteration02-*` probe files. The current-code series used the corrected query implementation and unchanged source/config identity before/after; no second benchmark or reingest was run.
- The newly authorized iteration02 native repair phase totals 211.300 seconds from preserved command elapsed outputs, including build overhead and failed/interim attempts, below its separate 360-second ceiling. This does not resolve exact compliance with the earlier 30-minute cap.
