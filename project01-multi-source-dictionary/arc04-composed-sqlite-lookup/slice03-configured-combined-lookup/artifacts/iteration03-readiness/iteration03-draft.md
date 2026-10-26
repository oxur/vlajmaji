# S03 iteration03 — make Linux native CONFIG positive mandatory

Status: **DRAFT/HOLD — not issued; no source/index/target transfer.**

## Authority and reason

Autonomous THROUGH ARC04 only; unchanged requested Luna/high CC and Sol6.1/high CDC. Duncan approved S03 normal source-branch push and CI watch; root owns delivery. This is a bounded validation correction for that delivery, not a new feature or inferred future push. All six original criteria/proof/owners remain unchanged and OPEN.

Frozen baseline is clean `45f21dc3d81d8b248d801104274fa799a1ff8608`, parent rejected d843. Local functional controls and the one retained Full campaign pass, but the original actual existing-byte CONFIG positive is not yet proved. The existing Unix test can return successfully from a direct-IO-qualified error arm on Linux; a green test name does not expose which branch ran. The strict Linux native ROOT control proves host byte-path capability, not direct CONFIG readback. This is a control-strength gap, not an observed product failure. Preserve original run40336154928 and its actual results.

## Exact source scope and repair

Only `crates/store/tests/collection_profiles_contract.rs` may change. No production module, dependency, Cargo.lock, schema, fixture, CI workflow or other test file.

In `test_configure_preserves_native_config_path_when_platform_allows`, gate the existing match arm as follows:

```rust
        #[cfg(not(target_os = "linux"))]
        Err(CollectionProfileError::Io { path, source }) => {
```

This preserves the existing Mac direct-IO error qualifier. On Linux any error must reach the existing final `Err(error) => panic!(...)`; success must retain all existing exact native CONFIG encoding/hex/utf8:null and real `CollectionProfiles::read` revision1 assertions. No new fixture, skipped test, early return, weakened assertion, runtime branch or positive-count inflation. One short explanatory comment is allowed. No other source change.

## Required readback and ownership

Read this prompt, current slice-plan/ledger and `artifacts/iteration03-readiness/runtime-retention45f201.json` in full; read the complete owning test and adjacent imports/helper needed to compile. Retain reliable prior AGENTS/framework/Rust intake; do not repeat unrelated guides/corpus work or execute historical caller writers. Acknowledge exact baseline, single path, branch semantics, all still-pending original Linux gates and no-Full boundary before dependent edits. Then CC alone owns source/index/retained target; CDC/root remain read-only.

Source root: `/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc04/vlajmaji`; branch `codex/project01-composed-sqlite-lookup`. Fixed guard `/tmp/vlajmaji-p01-a04-cargo`, Rust1.98.0, `CARGO_NET_OFFLINE=true`, explicit ENV `CARGO_TARGET_DIR=/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc01/target`, one writer. No new cache/CARGO_HOME/network/tool installation.

## Gates, retention and handback

Run actual current `make format`→`make lint`→`make check` in order with the existing ENV Make/guard recipe and complete durable logs/actual exits. Mechanically sum current Rust result rows and tooling counters; do not copy prior counts. Record any failed attempt. A focused owning test may be used if needed, but Mac success is not the Linux positive. No repeat Full store/import/query/timing or unchanged feature matrix merely for this cfg test correction. Current-head hosted CI is the discriminating Linux proof; root owns any reviewed delivery after handback.

Retain45f2 product/library/CLI/Full evidence only by exact one-test-file diff plus unchanged retention hashes. Existing package membership and runtime are unchanged; distinguish retained evidence from newly executed gates. Commit only the one source path with adjacent actual parsed trailers:

```text
Co-authored-by: Codex <noreply@openai.com>
Co-authored-by: Billo AI <ai-engineering@billo.systems>
```

Append execution provenance/receipts to the existing S03 `artifacts/execution.md` only; do not create close/verifier/retro or mark any row DONE. Return clean commit/parent/tree, exact diff/test SHA, parsed trailers, all sessions ended and explicit source/index/target handback. No push, PR, merge, publication, actual default/main/Bauske/accepted store mutation, S04 issue or A05 work.
