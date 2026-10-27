# Arc04 S04 — CC proposed completion

S04 delivers the explicit lifecycle layer for the local collection catalogue: schema-2 migration and old-writer fencing; selected update and forced rebuild; remove, exact reinstall, reconciliation, and retained immutable generations; configuration enable/disable/default policy; and reversible scratch handling of the legacy/default route. The library and CLI expose the operations with typed outcomes and known context, preserve explicit STORE behavior, and document the compatibility and lifecycle boundaries. This is the author’s proposed completion report, not independent acceptance. The six S04 ledger rows remain OPEN until root’s canonical review and the planning mechanical close.

## Six-row evidence walk

**A04-S04-01 — versioned catalogue and migration.** The implementation validates schema/version and head/member/registration relationships, migrates explicitly under the writer transaction, uses the active-generation pointer without falling back to the newest historical generation, and fences the old table names while retaining the read-only compatibility metadata view. Author and independent tiny controls cover migration, no-write failures, old-name refusal, pinned-reader Busy/retry, and captured-v1 behavior at their recorded source identities. The terminal Linux CI adds current-head execution of the native and lifecycle controls. This is finite compatibility and metadata validation, not a full payload audit.

**A04-S04-02 — update and rebuild.** Selected updates preserve unmodified co-active packs and historical generations; exact repeats are verified idempotently, while forced rebuild publishes a new physical generation with the same logical body. The retained independent literal Tiny proof checks all 25 record families, source/native/reference identity, old bytes, and nonempty Form/editorial cases. The one reviewed Full campaign separately exercised identical update and forced rebuild (G1→G2), with complete typed All checks and unchanged full response bytes for G1/G2/G4. The Full evidence is attributed to its original execution, not a rerun at 432.

**A04-S04-03 — removal and exact reinstall.** Removal retains catalogue/reference history and publishes the surviving selection or a null active pointer; explicit reinstall restores the same release-bound references without retargeting. The corrected persistent test now uses two packs, verifies a real committed REMOVE marker, clean reconciliation at each revision, and exact restoration. Independent CLI and stage-reconciliation journeys cover removed availability, genuine orphan/unknown-operation attention, retained external/old-store bytes, and cursor/reference behavior. Full G3/G4 provide the larger subset/remove and exact-restore counterpart.

**A04-S04-04 — selection policy.** Enable, disable, default, and profile membership remain policy separate from immutable source records and installed membership. Same-parent alternate configuration, latest-file behavior, no-op revisions, selected/all behavior, disabled errors, and cursor invalidation have author and independent Tiny/CLI evidence. The Full sequence additionally covers disable, restore while still disabled, enable, clearing the default without fallback, and restoring the configured default. Existing configure wire behavior is retained.

**A04-S04-05 — recovery and legacy/default transition.** Stage recognition accepts the five committed generation-producing operation names while preserving genuine orphan and unknown-operation attention; reconciliation does not activate or delete retained stages. The two-pack marker regression and current Linux/macOS runs exercise committed install/remove/reinstall and clean count-three reconciliation. Explicit scratch legacy/default switching and reversal preserve the old STORE route and bytes. The Full campaign ends with four retained committed stages and a clean active G4. Returned-fault and controlled child-exit evidence is retained at its exact scope; this does not establish power-loss or hot-journal recovery.

**A04-S04-06 — integration, gates, and evidence.** Current author gates and the actual current-head hosted run are green. Run 40515705170 for commit 432d6321e3b2431e7fe2cc5f60e9317780e984b8 terminated SUCCESS. CDC’s independent census reports Linux workspace 282/0/1 across 51 result rows, isolated search 39/0/0 across 7 rows, Linux Synthetic all-features 284/0/1 across 51 rows, and macOS workspace 277/0/1 across 51 rows. The run directly executes the native ROOT/CONFIG journey and both distinct /dev/full targets in default and all-features, plus the corrected REMOVE-marker test. A stale aggregate snapshot still says Linux in progress; direct completed job status and all successful steps resolve that snapshot, which is retained unchanged. The complete run evidence and census are in artifacts/cdc-linux-final01/.

## Frozen source and validation

Current source is 432d6321e3b2431e7fe2cc5f60e9317780e984b8, parent e4e8bfc55c5071d9144fa72e7360525efa43fd56, tree dd2f378be98b6fe9c62d5f45055a70a351971bb1. The exact S04 delta from accepted dcdf216ea5805b20f374ce74c302e42bef12c158 is these 20 files:

- crates/cli/README.md
- crates/cli/src/main.rs
- crates/cli/tests/collections_lifecycle_contract.rs
- crates/cli/tests/collections_maintenance_contract.rs
- crates/search/src/collection_query.rs
- crates/search/tests/collection_lifecycle_contract.rs
- crates/search/tests/collection_query_contract.rs
- crates/store/README.md
- crates/store/src/collection_profiles.rs
- crates/store/src/collection_profiles/policy.rs
- crates/store/src/collections.rs
- crates/store/src/collections/lifecycle.rs
- crates/store/src/collections/maintenance.rs
- crates/store/src/collections_schema.rs
- crates/store/src/lib.rs
- crates/store/tests/collection_policy_contract.rs
- crates/store/tests/collections_contract.rs
- crates/store/tests/collections_lifecycle_contract.rs
- crates/store/tests/collections_maintenance_contract.rs
- docs/collections.md

The six implementation passes are exhausted: initial catalogue increment, numbered 01–04, then the human-approved sixth test-only pass. Initial 8b5df7b3 was accepted only for its bounded increment; 024de816 was rejected, with its corrections and all subsequent test-precondition failures preserved. The first hosted e4 run exposed two Linux test preconditions; the explicit one-pass extension repaired those tests without changing production. All raw attempts and canonical reviews remain in artifacts/execution.md and their referenced bundles.

The author’s final local make check at 432 exited 0: 51 Rust result rows, 277 passed, 0 failed, 1 ignored, plus 19 helper tests. The changed CLI target ran 4/4 macOS-visible tests; its two Linux-specific tests were excluded locally. The hosted run above supplies their actual Linux execution. The source-boundary manifest confirms only the named CLI test file differs from e4 and 121 other tracked entries are identical, including production code and lockfile. Local Full evidence therefore remains applicable by exact production-byte equivalence; it was not repeated at 432.

## Retained Full evidence and limits

The single reviewed Full campaign completed four publication attempts without retry, with 39 stages including six expected typed-rejection children. G1 install, identical update, forced rebuild G2, subset removal G3, and exact reinstall G4 passed their frozen checks. G1/G2/G4 full inner response-plus-LF each measured 7,160,366 bytes with SHA-256 144b8b09b6e378243b36c8e01ab06353c3d573459668c8c158a86aebbea73556. Existing-output analysis also records native tuples, release/reference identities, totals, cursors, policy transitions, old STORE output and final reconciliation; it did not rerun queries. These are functional observations, not a benchmark. The observed maximum RSS was 7,971,405,824 bytes; resource-size checks were post-run, not a live memory cap. Full remove-all, changed-release Full scale, held-reader Full scale, actual process crash, power loss, and hot-journal recovery are not claimed. Native coverage is the specific 47-table/15-empty scope recorded in the Full checkpoint, not an independent reconstruction of every native row.

The detailed immutable scope pointer is artifacts/scope-inventory01.json (SHA-256 59badf643a64ada17e50f00c0e8b66f6c40e084c4184940aa84a2c8d3eb42c79): it inventories the 20 source paths and retained supporting artifacts, with ten immutable bundle manifests and 780 payloads rehashed without mismatch. The inventory’s note reflects its creation-time state; this report is now authored. The Full result bundle and exact local output analysis are under artifacts/cdc-full-result01/; no bulk Full database or output bodies were copied into the report.

## Proposed arc disposition and delivery boundary

For the parent arc, propose A04-03 and A04-05 DONE after root’s canonical consistency review; A04-01, A04-02 and A04-04 were already DONE. The other five arc rows remain OPEN, all fourteen project rows remain OPEN, and S05 is not yet issued. S04’s six child ledger rows remain OPEN pending root review and the planning close; this report does not change the ledger.

Protected primary source 52fd was not changed. Earlier user-authorized PR4 was merged by Duncan to main b0106b with the accepted S01–S03 tree and terminal green CI. Current S04 commit 432 is a separate frozen delivery. Explicit permission now exists for root to create the S04 PR after canonical close; no S04 PR has yet been created and merge is not authorized here. S05 must receive its own branch and root transition after this close. No claim is made that 432 is already merged. The planning mechanical-close commit should include this report and the final records; its actual commit identity is returned after creation rather than embedded self-referentially.
