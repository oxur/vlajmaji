# S03 closing report — sealed owned history

**CC proposed disposition, 2026-10-29 CDT.** The S03 implementation is frozen at source `2b7b6dda55dc00d62bfa4fcdc5d5a09742fcf528`, tree `83d81705cc1de3aa69f1d6d39e4c524bb1b6d9b3`, parent S02 `854865a7`. This report proposes evidence for the eight S03 child criteria; all eight remain OPEN pending CDC disposition. Parent status is separate: A05-01 and A05-02 are DONE; A05-03 through A05-08 remain OPEN. CDC/root own source, index, target, and final status. No external or Arc06 action follows.

The earlier [CC proposed report](artifacts/cc-closing-report.md) remains byte-unchanged at 5,960B/SHA-256 `9ee3b0b5dcf6087444dbdc09ee6ea158a9961fb25f13e1b0ed5a0cd4cd6eb8ef`. It is historical author evidence. CDC, not CC, ran the independent 58-child proof and the six-child CURRENT-empty schema2 counterpart; both passed with fixed inputs first. Those independent results are evidence in this proposal, not an author rerun or automatic row closure.

## Implemented candidate and actual integration

The implemented candidate provides pure lexicon OwnedSnapshot/OwnedSnapshotSeal, which preserve the complete schema1/2 base and curation history with actual ordered24-prefix rosters/cutoffs/hashes. Strict sealed loading checks canonical typed payload and inventory before SQLite, while the bare codec accepts valid pretty JSON and reserializes canonically. Checksums establish consistency rather than authenticity or redistribution rights. No PackId/All/model/registry version changes.

OwnedJournalReader exports its pinned complete history; fresh import publishes one complete transaction and exact canonical-history replay is idempotent. Same-head body/curation/old-cutoff, older/extension and cross-schema conflicts reject unchanged. Public Rust and native explicit CLI export/check/import/snapshot-model, the shipped self-contained example, and actual input-derived Full/Compact/Readable Exact/Resolve/detail use fresh explicit derived destinations. Missing context stays distinct from lost export data; foreign context corpora are not embedded.

[Verification](cdc-verification.md) supplies every original row predicate/disposition and scope. Actual current gates: pure7/store6/CLI3/private late1, author generated-input42children; Make392/0/1 plus19tooling; no-default83/0/0; sole disjoint Synthetic allfeatures394/0/1; package FILE inventories33/41/23. Independent fixed-input58children53zero5typed1 plus six allzero CURRENT-empty children preserve exact pairs, all old states, original-writer hash parity, whole materialized model and actual query/detail keys. No gate overlaps are summed as unique tests. Local Mac EILSEQ92 was observed; Linux positive/hosted CI and the ignored Full-corpus import were not executed.

## Produced paths and preserved evidence

The fourteen allowed source paths/commit/provenance are exhaustively named in [execution](artifacts/execution.md); allowed owned_schema.rs remained unchanged. [Evidence index](artifacts/evidence-index.json),SHAc249f59e, inventories339whole-original copies/1,220,756B with actual original paths/bytes/SHA, held Tiny-store/environment exclusions and zero missing requested prior logs. Raw EOF whitespace is preserved; authored-path diff-check passed. No SQLite/binary/default/retained Full payload is copied.

Canonical bodies are this report, [verification](cdc-verification.md), [retrospective](cdc-retrospective.md), [ledger](ledger.md) and updated parent records. Source-only author observer is artifacts/generated-query-method.py; actual author raws/inputs are under artifacts/author-current/ and author-prior/. Independent method/receipts/raws/inputs are cdc-proof01/ and cdc-empty-current01/. The preimplementation schema1-golden/, schema2-golden/, schema1-same-head-alternate/, build-history-goldens.py and history-golden-oracles.json remain separate immutable expected-format authorities, excluded from that original-copy index. The issued prompt remains db2b7877…90e4. No historical failures, unknown terminal or prior provenance limitation is erased.

## Proposed eight-row child disposition

The following maps the actual S03 child criteria in their issued order. These are proposed evidence summaries; every S03 row remains OPEN until CDC records disposition.

| Child row | Proposed evidence and scope | Status |
| --- | --- | --- |
| S03-01 | Pure and store whole-rich24 snapshot transport, historical rosters/state hashes, strict and manual parsing, and rehashed new-active/tombstone policy negatives with valid restores. | OPEN |
| S03-02 | Schema-two curation authority, historical selections/cutoffs/hashes, mixed own/external records, missing-context distinction, and current-empty migration counterpart. | OPEN |
| S03-03 | Exact original snapshot/seal bytes, repeat exports, strict malformed/version/kind/hash/length/inventory cases, bare pretty JSON acceptance, and restored valid inputs. | OPEN |
| S03-04 | Schema-one/schema-two/rich imports, exact replay, whole-state conflict and no-replacement controls, reader Busy/drop/retry, and TEMP-trigger rollback state. | OPEN |
| S03-05 | Shipped input-producing example, generated-input CLI lifecycle, native path reports and bytes, exclusive output opens, and post-commit stdout failure behavior. | OPEN |
| S03-06 | Input-derived Full/Compact/Readable Exact/Resolve/detail queries and the independent frozen-input whole-model/query proof. | OPEN |
| S03-07 | Scoped source compatibility, Make/no-default/Synthetic all-features and package-file inventories, with unchanged public model/registry/PackId boundaries. | OPEN |
| S03-08 | Intake, scoped implementation, retained failure history, evidence record, retrospective and proposed close report; final CDC/root disposition remains pending. | OPEN |

## Bubble-up

1. **Parent status and carryover:** A05-01 and A05-02 are DONE; A05-03 through A05-08 remain OPEN. S03 adds evidence for A05-03, but this proposed report does not close parent rows. P01-09 gains partial evidence only. Optional loader/package and fresh assembled/SAME canonical/root acceptance remain Arc05 owners.
2. **Unexpected findings and limits:** the initial schema-two import wrote inconsistent PRAGMA/meta versions, a real production bug corrected in the same unfrozen initial assignment. The first CURRENT-empty schema-two import/export check was initially pending; root later ran the six direct original-control children successfully. API/constructor/seal/IO/observer mistakes and lost lint terminal retain their own evidence classes. Mac capability, ignored corpus, incidental Cargo IO uncertainty and unchanged BL-0002 fixture fragility remain bounded limitations; no cause/fix/performance claim is added.
3. **Does the arc structural plan need change?** No. The original four usable increments remain coherent; S03 required no scope transfer or silent feature drop. S04's exact twelve-path optional-package/CI compatibility work remains its own slice. All eight S03 rows remain OPEN pending CDC disposition; Arc06/finalOperator/rights owners remain unchanged.

The [retrospective](cdc-retrospective.md) records process recommendations without turning them into adopted framework policy. This local close does not authorize network, push, PR, publication, a Full campaign or Arc06.
