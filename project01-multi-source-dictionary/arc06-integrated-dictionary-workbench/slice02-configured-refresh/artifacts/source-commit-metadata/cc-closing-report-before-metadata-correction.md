# S02 Proposed Closing Report — configured refresh

**Author proposal only.** The implementation and assigned local gates reached terminal results, but CDC has not independently dispositioned S02. All eight S02 rows, all eight Arc06 rows, and all fourteen project rows remain OPEN. This report does not accept a ledger row or close the slice.

## Delivered increment

The change adds a strict, explicit configuration store for named/default workbench profiles, refresh publication, logical status, and reusable queries bound to the exact configuration and effective source identity. It preserves the existing `CollectionProfiles` API and delegates unchanged dictionary, owned-history, query, and presentation semantics where specified. The emitted-input example and literal guide exercise profile configuration, refresh, default changes, no-op persistence, query/detail, and history. The exact source commit is `ab3528882cf6a05bfdbd7292bc3748bd3c01bfa9` (tree `8f0d61298a061410c8a133df2107b73b4de4313e`, parent `4fc937dcbebd0e2f3171f495345d6ab2a6e29f62`), with exactly the fifteen issued source paths and a clean worktree. Detailed commands, logs, hashes, source scopes, and preserved failures are in [execution.md](artifacts/execution.md).

## Proposed row walk

| S02 row | Proposed evidence and limits | Status |
| --- | --- | --- |
| S02-01 — strict explicit configuration and profile compatibility | Store and CLI contracts cover profile wire, named/default behavior, native paths, and unchanged legacy API scope. | OPEN |
| S02-02 — writer CAS, no-op, and error boundaries | Public store tests exercise latest-under-lock CAS, exact no-op bytes, overflow, and strict nullable wire; private tests cover writer ownership and sentinels. No hostile-race guarantee is claimed. | OPEN |
| S02-03 — refresh publication and source mutation | Example/guide and private hooks cover refresh, pre/post-visible failure, currentness checks, successful writer windows, and durable prior state. No joint SQLite/config atomicity is claimed. | OPEN |
| S02-04 — reusable configured query identity and tokens | Search tests cover Full/Compact input-derived rows, same-binding continuation, cross-binding rejection, detail, and exact config-byte invalidation. Full, Compact, and Readable order policies remain distinct. | OPEN |
| S02-05 — defaults, lifecycle, and availability | CLI/example cover profile replacement, remove/re-add, defaults, history and stale behavior. Store status distinguishes registered, disabled, removed, selected-unregistered, unavailable, and malformed minimum-header cases. Unchanged S01 history/migration/curation semantics are inherited only through delegated code; no extra configured-specific migration execution is claimed. | OPEN |
| S02-06 — CLI native files and output acknowledgement | CLI tests cover Unicode/space paths, deleted-cwd, capability-scoped raw-byte behavior, Other/BrokenPipe, and persisted state. The macOS raw-byte creation branch returned EILSEQ 92; Linux raw-byte success and Windows native behavior were not run. | OPEN |
| S02-07 — current checks, archive, and package inclusion | Current format/fmt-check/lint, affected tests, `make check`, isolated no-default, one Synthetic all-features archive, and three `cargo package --list` inventories passed at their recorded scopes. The inventories establish included files only; no package build, publication, or hosted CI is claimed. | OPEN |
| S02-08 — independent verification and closeout | The author returned the exact source/index/one-target state QUIET to CDC. Independent source/public/CLI review, final row disposition, retrospective, and parent bubble-up remain CDC work. | OPEN |

## Validation and limits

The final current-source `make check` passed with 73 Rust result groups (442 passed, 0 failed, 2 ignored) and separate tooling checks (19/19). The isolated no-default command passed 12 groups (91 passed). The sole all-features archive gate passed 73 groups (445 passed, 0 failed, 2 ignored) after five existing Synthetic preparation recipes. Package file inventories listed 46 store, 29 search, and 26 CLI files. Exact argv, environment, physical working directories, raw paths, hashes, earlier failures, and qualifications are recorded in [execution.md](artifacts/execution.md).

The all-features check used the one disjoint local Synthetic archive and exact fifteen-file overlay plus `Cargo.lock`; source/archive bytes were compared before and after. It does not establish hosted CI. No Full/default corpus, network or external acquisition, package build/publication, push, or PR occurred. Earlier setup/toolchain-home deviations and failed attempts remain in the execution history and are not reclassified as successful product evidence.

## Bubble-up

- **Delivered result:** a local configured-profile refresh and query increment with an emitted-input example, CLI journey, and package file inclusion evidence; no external delivery is claimed.
- **Unexpected findings:** Full, Compact, and Readable have different source/member ordering behavior; the macOS raw-byte branch is capability-limited, and Linux raw-byte success remains unobserved here.
- **Arc change proposed:** none. CDC should keep all eight S02 rows open until its independent review. S03/S04/S05/S06 and the final canonical/Operator project gates remain separate work.
