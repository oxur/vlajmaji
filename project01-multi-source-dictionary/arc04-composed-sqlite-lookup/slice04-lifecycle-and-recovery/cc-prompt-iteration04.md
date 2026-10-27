# S04 iteration04 — complete the persistent committed-REMOVE regression

Status: ISSUED 2026-10-27 05:42:16 UTC after root concrete readiness PASS. CDC owns source/index/retained target until exact issued intake/readback and explicit transfer.

Baseline clean `a7b9baf67a6eea784cb90a2f388bf5e2537cbc72`, parenta9eeb440, tree7dcc6aa68b07abb90fea30b64d64d3dac2b4ab81 on the same isolated codex branch. This is the FIFTH and final implementation pass in S04 (initial,01,02,03,this04), with no reset or same03 relabel. Existing Duncan Autonomous THROUGH ARC04 and CDC Sol6.1/high + retained CC Luna/high continue. All six original rows remain OPEN; no Full/S05/default/network/delivery authority.

## Readback and ONE source path

Read this prompt, current full slice-plan/ledger and artifacts/review-i03-a7b01.md. Read cdc-i03-a7b01/summary.json and exact frozen-store-maintenance-test-a7b.rs.txt plus current independent public/CLI29/stage-pairs raw receipts and summary. Prior exact caller/fixtures are unchanged in cdc-i02-a9e01, referenced by source hashes; read/reference only, never execute historical writers. Read source AGENTS.md fully and applicable Rust EH-01/EH-04/EH-10 + OB-16 as needed. Inspect current test/helper bodies at sealed_pack, sealed_llg, request, two_pack_request and test_reconcile_committed_stage_markers_and_unknown_operation. Production collections.rs::orphan_stage_attempts is a read-only reference only.

ONLY `crates/store/tests/collections_maintenance_contract.rs` may change: complete that existing named test and at most a small private assertion helper in this test file. No production, other tests/files, docs, dependency/lock, schema, configuration, publisher/query/CLI, workflow or package changes. Source/index/target remain CDC-owned until your exact source-cited readback, clean a7/parent/tree/branch/session confirmation and explicit transfer. First append accurate intake/coverage erratum to existing artifacts/execution.md after transfer.

## Exact gap and correction

The production fix at a7 independently PASSES the original finite runtime scope, including three REAL CLI reconciliations after actual install/subset remove/exact reinstall. However, the saved author03 test installs only LLG, removes the LAST member (`generation=None`), then reinstalls. That history has only TWO physical generations; no committed remove marker is ever created. It also lacks explicit clean.attention_required:false and retained_generation_count3 assertions. This does not satisfy the persistent regression explicitly issued03. Record this as author test-coverage/attestation shortfall, not a production defect or correction to the original requirement. Preserve a7/test/logs/focused ordering error and narrower pass.

Complete the SAME existing regression with exact original preconditions:

```rust
let (release, seal, release_sha256) = sealed_llg();
let (jb_release, jb_seal, _) = sealed_pack(PackId::LensiskuJbotci);
let installed = CollectionCatalogue::install(
    &root, two_pack_request(&release, &seal, &jb_release, &jb_seal),
).unwrap();
```

Keep ROOT fresh/absent. Reuse that exact original LLG input/digest for reinstall; JB remains the survivor. Do not substitute empty remove-all or synthesize committed SQL rows/markers. Empty Synthetic bodies are adequate ONLY for marker transport, clearly separate from already retained original nonempty corpus/25-family proof. Do not hold a catalogue reader across mutations; returned report values and filesystem reads suffice.

1. After real install, assert revision1, a nonempty report attempt/generation, both expected member pack IDs, actual install marker operation/ID, and public reconcile clean (`staged_attempts=[]`, `attention_required=false`, retained_generation_count1, actual active generation and committed attempt). Preserve DB/marker bytes around this observation.
2. Remove LLG at expected1. It MUST return Some NEW generation with only JB member, revision2 and a real remove attempt. Read `generations/<removed.attempt_id>/attempt.json` and assert version1/that exact attempt/operation remove. Public reconcile must again be clean, retained_generation_count2 and active equal this survivor generation. Freeze DB/committed marker bytes before/after observation. Replace the old last-member None assertion because the deliberate input fixture now has a survivor; never weaken product behavior to force it.
3. Reinstall exact LLG at expected2. Assert revision3, Some distinct new physical generation, restored two members and actual reinstall marker; public reconcile must be clean with attention:false and EXACTLY THREE retained generations and three matching committed attempts. Assert the real committed REMOVE attempt/marker is among them. All old marker files/generations retained.
4. Preserve existing genuine-orphan and unknown-operation counterexamples. Supported remove marker with a fresh uncommitted ID remains orphan/attention. Unknown operation in a separate create-new owned directory deliberately names an ACTUALLY COMMITTED install attempt ID and remains unreadable/native-directory-hex, retaining the genuine orphan. Exact sorted-vector/set+cardinality comparison; no directory-order contract. Revision/current generation/attempt rows/DB/all original and controlled marker bytes remain unchanged. Retain controlled stages; no activation/repair/cleanup feature.

Use existing report/native/marker shapes, no new API. Keep assertions discriminating all three committed operation kinds, not only the existence of an operation row (remove-all also has a committed row). Preserve strict old assertions whose preconditions are unchanged. Failed owned roots remain; successful owned cleanup only after every assertion follows existing practice.

## Proportionate gates and exact retained applicability

After edits, current `make format` then `make lint` precede the ACTUAL revised named test:

`cargo +1.98.0 test --locked --offline -p vlajmaji-store --test collections_maintenance_contract test_reconcile_committed_stage_markers_and_unknown_operation -- --exact`

It must select/pass1 test, not zero. After that run one stable current `make check` (format/lint/tests/docs/tooling) and changed store --list inventory. No new feature/source matrix or unrelated refactor. Use the existing Rust1.98/offline/locked CARGO wrapper/retained target, CARGO_HOME untouched, exclusive unique `i04` raw logs; no overwrite/reset.

Production and every other tracked path MUST be byte-identical to a7. Prove exact one-file Git-tree delta and unchanged runtime/workflow/lock boundary. Retain actual a7 no-default39, all-features279/0/1 and unchanged search/CLI inventories ONLY as prior-source evidence with that exact test-only applicability. Revised test body is proven by the current targeted test/workspace check; do NOT claim it already ran all-features/Linux. Mandatory eventual CURRENT-HEAD Linux/default/all-features CI remains pending and will execute it there. No repeat of unchanged independent runtime/feature campaigns merely for bookkeeping; CDC may retain the already completed a7 public/CLI29/three-stage runtime proof after independently verifying the exact test-only delta. Later runner Tiny qualification remains required and distinct.

Observed200MiB Tiny accounting retains all earlier failed/retried roots and its disclosed non-peak/non-cumulative limitation; no cleanup to reset it, no Full data/stores/cache. Record actual commands, selected populations/exits/hashes and the explicit author03 scope erratum in existing execution. Commit ONLY this named test file with adjacent parsed Codex/Billo AI trailers; verify exact parenta7/tree/clean state, close every session and explicitly return source/index/retained target. No DONE rows or formal closure placeholders. If another substantive implementation correction is needed after this pass, surface exhausted five-pass budget to CDC/root; do not silently issue05 or request a model swap/reset. Ordinary caller-only method qualification errors remain honestly separate from product scope, never hidden.
