# S04 initial assignment — catalogue2, explicit migration, update/rebuild

**ISSUED initial iteration after root readiness PASS.** Source/index/target transfer only after exact current-packet acknowledgment. Retained CC gpt-6-luna/high, CDC gpt-6.1-sol/high. All six S04 rows stay OPEN through initial implementation; follow-up lifecycle/policy work is mandatory.

## Exact authority and intake

After root review and ISSUED acknowledgment only: own existing isolated source branch `codex/project01-composed-sqlite-lookup` at clean dcdf216ea5805b20f374ce74c302e42bef12c158. Main b0106b has identical accepted tree but is protected; do not switch/edit main. No new branch/worktree/cache/model. CDC owns planning and independent review; root remains read-only. No push/network/PR/CI/default/Bauske/accepted data action. No Full create/import/query, benchmark or OS-cache action. Use exclusively owned tiny roots; retain failed artifacts and all prior stores/binaries/records.

Load completely: current source AGENTS.md; this slice-plan.md and ledger.md; relevant current Arc04 binding contracts/W4 from ../arc-plan.md and ../artifacts/user-workflows.md; source collections_schema.rs (all), collections.rs (public types, open/read paths1090–1465, mutation+transaction/stage helpers1465–2490 and report/minimum types2950–3280), store lib.rs exports, owning tests and CLI command/collection mutation/output branches. Recover actual line drift/truncation; read complete functions. Read S03 accepted profile/query contracts only where called; do not relaunch old probes/writers. Read the four-pack recipe/readiness census and its three literal fixture files under ../slice03-configured-combined-lookup/artifacts/readiness01; complete_fixture is supplemental nonempty Form/editorial proof. Preserve source-literal expected tuples BEFORE candidate execution.

Retain reliable prior complete framework/Rust intake; reload changed portions as needed. Apply actual Rust EH-01/EH-04 typed errors, OB-01/02 ownership, existing explicit TransactionGuard/indeterminate/output and source AGENTS test/docs conventions. Record bounded required-read coverage, source hashes and a source-cited readback of version/read/write fence, expected revision, active pointer, idempotent versus forced rebuild, DELETE Busy semantics and no deletion. Challenge any contradictory source before mutation. No broad guide/corpus intake restart.

## Implementation spine (binding behavior in plan; private names local)

1. Implement catalogue2 initialization and compatible1/2 reader. Rename all SIX tables inside explicit migration. Keep ONLY original catalogue_meta as a read-only key,value view; other five old names are absent, preventing old same-release no-op reads from falsely succeeding. All new v2 reads/writes use fixed physical names; compatible new v1 readers use original physical names through closed version-selected SQL. No writable view triggers. Recheck fence under BEGIN IMMEDIATE on every writer. v1 ordinary writes return MigrationRequired; explicit migrate checks expected revision and metadata consistency, validates FK targets/check, then atomically updates version/pointer/revision. Fresh roots2, no automatic migration/read mutation. Null pointer never falls back to latest history. Ordinary opens still minimum only.
2. Extract only the necessary common generation builder/publication logic from current install. Use the new child collections/lifecycle.rs, parent-private helper reuse and exact public signatures/types in plan. Preserve input normalization/seal/kind/native path/error dispositions and all old install tests. Current raw-SQL test corruptions need deliberate physicalv2 targets; preserve their original failure and positive oracles. All previous runtime CLI wire fields remain exact, including complete nullable contexts and actual native paths.
3. Add update: selected current members replaced, all others retained; stale expected revision rejects before stage; complete same input is verified idempotent, different kind/pin declaration not silently equal; current-set strict composition rejects unequal co-active identities while historical descriptors remain. Add forced rebuild: intact current sealed copies→new verified artifact with identical logical canonical/content IDs, new generation/revision. Missing/corrupt input remains typed failure; missing/corrupt derived store can be repaired explicitly without old-store fallback.
4. Preserve private modes, owned exclusive stages, verification-before-activation, explicit drops and fsync. Publish generation+heads+pointer+revision+attempt together; retain old bytes/orphans. Expand stage-operation recognition only for actual new operations. On failures preserve initiating/rollback typed causes and known input/activation contexts; receipt loss after confirmed commit remains committed. No false Drop guarantee.
5. Wire migration/update/rebuild CLI, the intentionally additive catalogue_schema_version field in the Serialize-only CollectionListReport version1 (all prior fields preserved; no exact whole-list-byte compatibility claim), help/guide/library docs and exports. All new runtime branches must use the existing managed collection failure/output path; no generic legacy envelope or lossy display-only native errors. Public #Errors docs and concrete usage: list revision→migrate ifneeded→update/rebuild→returned explicit STORE query/verify. No removal/policy/recovery implementation yet; document the still-future commands truthfully.

## Exact 12-path scope

- crates/store/src/collections.rs
- crates/store/src/collections_schema.rs
- crates/store/src/collections/lifecycle.rs (new)
- crates/store/src/lib.rs
- crates/store/tests/collections_lifecycle_contract.rs (new)
- crates/store/tests/collections_contract.rs
- crates/search/tests/collection_query_contract.rs
- crates/cli/src/main.rs
- crates/cli/tests/collections_lifecycle_contract.rs (new)
- crates/store/README.md
- crates/cli/README.md
- docs/collections.md

No Cargo/lock/workflow/adapter/model/dictionary.rs changes. Existing search test change is only physical-table corruption transport. Preserve all failed/bad/valid predicates; cannot bypass new view fence merely by dropping a test. Further path/policy need returns to CDC before edit.

## Cheap proof before gates

Implement actual public/CLI tests of every INITIAL proof in slice-plan. Start with metadata/view/write-fence+registered-only migration, then nonempty update/rebuild. Build expected inventory/source/native IDs from literal inputs before mutation. Check exact six physical table names/columns and actual pragma FK targets, the single meta-view shape, other old-name absence, all18 old-name INSERT/UPDATE/DELETE refusals, and old register/install same-release no-op queries failing before stage. Metadata remains unchanged after modeled writes; retain actual v1 CLI binary proof separately from modeled preopen SQL race. Pinned old read transaction must produce commit Busy+explicit rollback then released-lock successful migration. Same current-state valid counterpart accompanies each corruption/version/stale/path/fault negative.

Use private cfg(test) phase hooks or same common handler where necessary, never production failpoints. Test data/commit errors, precommit interruption, unknown activation/rollback uncertainty and postcommit receipt loss truthfully. No small baseline fake is called actual Full. Capture exact numeric counter/source/category/empty/native denominators mechanically; same comparator gets cheap positive/reorder and missing/extra-empty/multiplicity/Presence negatives before later expensive work. Retain original fixture/oracle/compiler/lint attempts.

## Gates, freeze and return

Use `/tmp/vlajmaji-p01-a04-cargo` with Rust+1.98.0, CARGO_NET_OFFLINE=true, CARGO_TARGET_DIR=/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc01/target explicitly even Make tooling children. No other writer. Complete current make format→make lint→make check in that order, full raw logs+actual exits in this slice artifacts/execution.md (bulk logs local plus canonical bounded receipts as appropriate). Scope additional isolated/all-feature gates to changed stable source and disclose actual platform. Do not infer a future hosted push or claim Linux proof from cfg.

Only after stable gates: git diff --check, exact12-path scoped source commit with adjacent parsed Co-authored-by Codex<noreply@openai.com> and Billo AI<ai-engineering@billo.systems> (normal trailer spacing), clean source, parent/tree/file hashes/current tests and complete remaining-six-row census. Explicitly return source/index/target ownership and all session exits. Proposed initial done is NOT independent acceptance or S04 closure. CDC reviews this finite useful format/update/rebuild increment, then authors numbered follow-up for the mandatory remove/reinstall/recovery/policy/migration integration. Do not create closing-report or mark any row DONE yet.
