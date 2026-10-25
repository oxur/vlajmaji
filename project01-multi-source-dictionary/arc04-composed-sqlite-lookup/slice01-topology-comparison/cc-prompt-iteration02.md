# CC iteration 02 — complete exact-call timing and digest-scope prose

**ISSUED, 2026-10-25.** Same Autonomous Arc04 authority and requested gpt-6-luna/high CC. CDC owns design/planning/independent acceptance. No model/budget permission question. This is a bounded correction to frozen candidate `6fc740ea9bb5058abd3ac9a7a3b74279a7dca978`, not a new slice or a restart. Preserve the initial prompt and iteration01 bytewise.

## Authority, ownership and fixed state

Source: `/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc04/vlajmaji`, branch `codex/project01-composed-sqlite-lookup`, clean parent 6fc740e. CDC transfers source/index/retained target only when dispatching this committed prompt. CC must acknowledge current readback before edits. Root is read-only. End sessions and explicitly return ownership when frozen.

Mutate only these source paths: `crates/search/src/dictionary.rs`, `crates/search/tests/dictionary_topology.rs`, `crates/search/README.md`. Append this slice's existing `artifacts/execution.md` with actual attempts, logs, component hashes and limits. No other source, dependency, lock, schema, corpus, store or accepted-record change. No push, PR, publication, default/main/Bauske mutation or A05. Do not run Full prepare, candidate verification/query, timing rounds or regenerate Full inputs. The seven-total count amendment is approved; its remaining prerequisites still HOLD.

Use retained target `/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc01/target`, guard `/tmp/vlajmaji-p01-a04-cargo`, explicit ENV `CARGO_TARGET_DIR` and `CARGO_NET_OFFLINE=true` on every Cargo/Make invocation, +1.98.0, locked/offline, login:false and one writer. Preserve old release probe SHA `06ac76fd68c6df0e75089ecf193403307eb6dd2bc94c6461905aecc98ff813e4` to a fresh named owned scratch copy BEFORE rebuilding; original first-attempt binary/stores/logs stay untouched.

## Source-grounded correction

Frozen `exact_profiled` wraps `exact_inner`, but `exact_inner` assigns `profile.total` before it returns and drops `all_hits`, candidate sets and other locals. Therefore its advertised whole-call total and residual omit inner cleanup. This is a real measurement boundary defect, not a response-value finding.

Move the whole-call `Instant` start and `profile.total` assignment into OUTER `exact_profiled`: start immediately before calling `exact_inner`; after it has fully returned, set total, then derive `lookup_order_paging = total - hydration - candidate_sql` with the existing conservative duration treatment. Remove the inner whole-call timer/total/residual assignment. Keep actual same-call internal SQL and availability/hit/group hydration blocks. Do not change exact selection, order, totals, cursor, errors, data wire, startup validation or native IDs. The warm lookup budget still uses SQL plus residual; full hydration/serialization still includes Value conversion and JSON/LF rendering. No separate SQL-only subtraction or response metric fields.

Strengthen the existing public topology/profile test to include Unified profiled parity as well as A/F, and basic public duration bounds against an outside stopwatch (SQL plus hydration cannot exceed total; total cannot exceed the externally observed call). Do not introduce a generic instrumentation framework, fabricated allocation cost, sleeps, or brittle performance threshold. The decisive cleanup-boundary proof is source placement around the completed inner call; tests also retain value equality.

The search README phrase “then checks the composed logical digest” must distinguish U's recorded-header/manifest comparison (unauthenticated until explicit verification) from A/F computing the composed canonical digest. Store README is already precise. No broader prose rewrite.

## Retained evidence and bounded verification

At parent 6fc740e, CDC actually ran 18 composition plus 40 snapshot/query controls, including explicit missing-SQL-row versus Projection failure; all passed. CDC also exercised four sealed Synthetic inputs through actual prepare to five tiny stores and U/A/F run/batch, with TWO priming and SEVENTEEN measured requests per provider (19 total), all values equal and DB hashes unchanged. Preserve those actual denominators. No Full observation or balanced round ran. These semantic/builder controls are retained, while the old exact total is not a complete-call measurement.

Run current `make format` then `make lint` then `make check`, capture complete fresh raw logs and exact populations/exit. Run the affected public `dictionary_topology` test and search no-default test surface. Existing all-features/package evidence may be retained for unchanged feature/dependency/file inventory; broaden only for a real changed condition or failure, not a duplicate matrix. Rebuild the release example once using the fixed guard/environment and `build --release -p vlajmaji-search --example topology_probe --locked --offline`.

Commit only the actual changed allowlisted source paths with both established Co-authored-by trailers. Return exact source commit, clean status, old/new binary SHA, source hashes, complete logs, sessions-ended statement and ownership. CDC will perform corrected read-only tiny run/batch/profile parity on the SAME five tiny stores before Full readiness release. No new Full data/store is authorized by this repair prompt. All six slice rows remain OPEN.
