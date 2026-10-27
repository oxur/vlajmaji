# Current canonical close — 2026-10-27 14:09:56 UTC

Root final canonical consistency PASS at planning8bc3f386/source432d632. Six child rows now DONE by mechanical close; newly A04-03/05 DONE only, five other arc/all14project OPEN. The author proposed-completion body below is retained as the dated pre-close attestation; references there to OPEN/pending describe that checkpoint. The actual close commit is pending until committed and will be appended through the recorded convention.

# Arc04 S04 — CC proposed completion

S04 delivers schema-2 migration/fencing; update, rebuild, remove and exact reinstall; reconciliation; selection policy; and reversible scratch handling of legacy/default state. Library, CLI, and guides provide typed outcomes and preserve explicit STORE behavior. This is the author’s proposed completion report, not independent acceptance. The six S04 ledger rows remain OPEN until root’s canonical review and the planning mechanical close.

## Six-row evidence walk

**A04-S04-01 — versioned catalogue and migration.** Migration validates version and head/member/registration metadata under the writer transaction, uses the active pointer without latest-history fallback, and fences old names while retaining the read-only metadata view. Author and independent Tiny controls cover no-write failures, old-name refusal, pinned-reader Busy/retry, and captured-v1 behavior. Current Linux CI runs the lifecycle controls. This is metadata validation, not a full payload audit.

**A04-S04-02 — update and rebuild.** Updates preserve other active packs and history; exact repeats are idempotent, while forced rebuild keeps the logical body and creates a physical generation. Independent Tiny proof covers 25 record families, source/native/reference identity, old bytes, and nonempty Form/editorial cases. The retained Full campaign exercised update/rebuild (G1→G2) and typed All checks; it was not rerun at 432.

**A04-S04-03 — removal and exact reinstall.** Removal retains references and publishes the surviving selection or a null pointer; reinstall restores the release-bound identity. The corrected two-pack regression verifies a committed REMOVE marker, clean reconciliation at each revision, and exact restoration. Independent CLI/stage journeys cover removed availability, orphan/unknown-operation attention, retained bytes, and cursor/reference behavior; Full G3/G4 supply the larger subset/restore case.

**A04-S04-04 — selection policy.** Policy stays separate from source records and installed membership. Author and independent Tiny/CLI evidence covers alternate configuration, latest-file behavior, no-op revisions, disabled errors, selection, and cursor invalidation. Full covers disable/re-enable and default clear/restore without fallback; configure wire is retained.

**A04-S04-05 — recovery and legacy/default transition.** Reconciliation recognizes the five committed generation operations while retaining orphan/unknown-operation attention and never activates or deletes stages. The two-pack regression and Linux/macOS runs cover install/remove/reinstall and clean count-three reconciliation. Scratch legacy/default switching preserves old STORE bytes; Full ends with four committed stages and active G4. Returned faults and child exits are not power-loss or hot-journal proof.

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

Six passes are exhausted: initial increment, numbered 01–04, and the human-approved sixth test-only pass. Initial 8b5df7b3 was accepted only for its bounded scope; 024de816 was rejected. The first e4 CI exposed two Linux test preconditions, repaired by the explicit one-pass extension without production changes. Failed attempts and reviews remain in execution.md and their bundles.

The author’s final local make check at 432 exited 0: 51 Rust result rows, 277 passed, 0 failed, 1 ignored, plus 19 helper tests. The changed CLI target ran 4/4 macOS-visible tests; its two Linux-specific tests were excluded locally. The hosted run above supplies their actual Linux execution. The source-boundary manifest confirms only the named CLI test file differs from e4 and 121 other tracked entries are identical, including production code and lockfile. Local Full evidence therefore remains applicable by exact production-byte equivalence; it was not repeated at 432.

## Retained Full evidence and limits

The single Full campaign completed four publication attempts without retry and 39 stages, including six expected rejections. Install G1, identical update, rebuild G2, subset removal G3, and exact reinstall G4 passed frozen checks. G1/G2/G4 full inner response-plus-LF each measured 7,160,366 bytes with SHA-256 144b8b09b6e378243b36c8e01ab06353c3d573459668c8c158a86aebbea73556. Existing-output analysis also records native tuples, release/reference identities, totals, cursors, policy transitions, old STORE output and final reconciliation; it did not rerun queries. These are functional observations, not a benchmark. The observed maximum RSS was 7,971,405,824 bytes; resource-size checks were post-run, not a live memory cap. Full-scale remove-all, changed-release, and held-reader transitions were not run. Separate actual Tiny test-subprocess abrupt-exit/restart controls are claimed only at their recorded scope; they are not Full-scale or power-loss/hot-journal evidence. Native coverage is the specific 47-table/15-empty scope recorded in the Full checkpoint, not an independent reconstruction of every native row.

The scope and planning-close pointer is artifacts/scope-inventory01.json. Its planning_commit_scope field enumerates the proposed planning records and close materials; CDC will refresh that field after the current verification, retrospective, and checker bodies are settled. The Full result bundle and exact local output analysis are under artifacts/cdc-full-result01/; no bulk Full database or output bodies were copied into the report.

## Bubble-up to the arc

S04 delivered the lifecycle work assigned to it; no S04 requirement is silently deferred or treated as a no-op. The distinct readable/compact/reference/detail output and numeric-budget work remains in S05 as specified. Propose all six S04 child rows transition to DONE only after independent acceptance and root canonical consistency; all six ledger rows remain OPEN today. Propose A04-03 and A04-05 DONE after that review; A04-01, A04-02 and A04-04 were already DONE. The other five arc rows and all fourteen project rows remain OPEN, and S05 is not yet issued.

Answering the findings-to-plan question: after acceptance, update the arc plan to record the current/versioned lifecycle closure and the remaining S05 output delivery, without creating a structural slice or changing criteria. The [CDC verification](cdc-verification.md) and [CDC retrospective](cdc-retrospective.md) provide the independent disposition and process analysis; the report and planning-file inventory are enumerated by artifacts/scope-inventory01.json.

Protected primary source 52fd was not changed. Earlier user-authorized PR4 was merged by Duncan to main b0106b with the accepted S01–S03 tree and terminal green CI. Current S04 commit 432 is a separate frozen delivery. Explicit permission now exists for root to create the S04 PR after canonical close; no S04 PR has yet been created and merge is not authorized here. S05 must receive its own branch and root transition after this close. No claim is made that 432 is already merged. The planning mechanical-close commit should include this report and the final records; its actual commit identity is returned after creation rather than embedded self-referentially.

## CDC canonical close addendum

All six author-proposed DONE rows independently accepted, no deferrals/no-ops; root final scope/evidence consistency PASS. Mechanical close and actual commit/receipt are recorded in execution.md and cdc-verification.md. Supporting artifact/planning inventory is refreshed in artifacts/scope-inventory01.json. S04PR creation and separateS05branch are root-owned next actions, no source/merge/publication action here.
