# Slice01 closing report — accepted

## Initial CC candidate 40a94b4 — 2026-10-24

The following is CC `/root/cdc/p02a02_cc`'s attributed attestation, transferred from `artifacts/cc-execution.md` by CDC without upgrading its evidence strength. Independent review failed two controls; see `cdc-verification.md`. Initial prompt and candidate remain preserved.

## CC-proposed closing report

**Candidate:** commit `40a94b4c8abf44954244b01a273fba221fd1b916` on `feature/project01-dictionary-foundation`, message `feat(lexicon): add validated source registry and attribution`. The assigned source worktree was clean immediately after commit. The commit contains exactly these nine allowlisted source paths:

- `/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc01/vlajmaji/CHANGELOG.md`
- `/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc01/vlajmaji/README.md`
- `/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc01/vlajmaji/crates/lexicon/README.md`
- `/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc01/vlajmaji/crates/lexicon/examples/source_registry.rs`
- `/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc01/vlajmaji/crates/lexicon/src/attribution.rs`
- `/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc01/vlajmaji/crates/lexicon/src/lib.rs`
- `/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc01/vlajmaji/crates/lexicon/src/registry.rs`
- `/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc01/vlajmaji/crates/lexicon/tests/registry_contract.rs`
- `/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc01/vlajmaji/docs/source-registry.md`

The only delegated planning path written by CC is `/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/planning/project01-multi-source-dictionary/arc01-dictionary-foundation/slice01-source-registry-and-assertions/artifacts/cc-execution.md`; CDC retains planning commits and canonical close ownership.

| Ledger row | CC proposal | Evidence and limit |
| --- | --- | --- |
| S01-01 | Proposed done | Validated distinct ID roles, source standing, alias records, descriptive lineage, snapshot/occurrence attribution and borrowed views; public example and external caller retain A/B, D1/D2 and unknown attribution. |
| S01-02 | Proposed done | Exact selectors and rename/retire/reuse/collision/ambiguity/ID-shadow controls pass; identity remains stable across the exercised rename. |
| S01-03 | Proposed done | Six new contract tests and example prove 5 `farvi` occurrences (A=4, B=1), identical distinct locators and the same supplied assertion ID without deduplication. |
| S01-04 | Proposed done | Contract assertions compare raw bytes, full record identities, field Missing/Null/empty and nested large-number fields; archive bytes and archive roundtrip remain equal; separate lexical/assigned forms and proposals remain separate. Existing lexicon contract tests pass. |
| S01-05 | Proposed done | Constructor, JSON/serde, duplicate-key, reference, collision and history negative controls pass; failed updates preserve byte-identical registry JSON. |
| S01-06 | Proposed done | Shipped docs/example; offline external path-dependent caller; Rust 1.98 format, Clippy, default/no-default/all-feature tests, workspace check and docs gates pass as enumerated above. The one existing full-corpus test remains ignored; no full import is claimed. |
| S01-07 | Open for CDC | This is the attested return and carryover statement, not independent verification. CDC must reproduce/disposition rows and own the canonical close/bubble-up record. |

**Gate populations:** final `make check` passed with 50 passing tests and one ignored corpus test across CLI, legacy/new lexicon contracts, search, store, doctests and tooling; final no-default suite passed 10; final all-feature suite passed 33 with the same one ignored corpus test. Locked `cargo check --workspace`, shipped example and offline outside-workspace caller all exited 0. Full command populations and the chronology of failed attempts are in the preceding section. The outside caller is `/tmp/vlajmaji-p01-a01-s01-cc-caller.L9QA3A`; build outputs used the prescribed sibling Arc01 target directory. No external dataset query, corpus import, network operation, publication, push or integration was performed.

**Artifact inventory:** the nine committed source files above; this delegated execution/closing attestation; the temporary external caller directory; and build/test outputs under `/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc01/target`. No separate raw terminal log files were created; command outcomes and observed counts are recorded here. Existing preserved research and source inputs were read-only.

**Arc carryover:** S01 adds the validated in-memory registry/attribution sidecar and public use path. It does not persist this metadata or implement the broader raw-preserving schema, twelve coverage areas, standard mappings or archive adapter owned by A01 S02. It does not implement grouped exact source results, totals, pagination, availability, stable links, or pack removal/reinstallation and revision query transitions owned by A01 S03. Arc close still needs to compose accepted registry → persisted fixture → grouped Rust/CLI lookup, all occurrence pages, and removal/re-add/revision transitions. No database topology, editor, import or source-equivalence decision is made here. Independent acceptance and later-slice work remain with CDC under the canonical plan.

**CC status:** implementation, self-review, listed gates and explicit source commit are complete. The candidate is proposed done and frozen for CDC review; no ledger acceptance has been prefilled by CC.


## Current gate

The initial candidate required iteration01; its failed evidence remains above. The current candidate is accepted as recorded below.


## Refinement01 CC attestation

The following remains CC-authored attestation from `artifacts/cc-execution.md`, preserved by CDC; independent disposition is in `cdc-verification.md`.

## Refinement01 implementation, reproduction and frozen return — 2026-10-24

**Candidate:** `11ec7017a375fa62ba101cb51d85c36034621b81` (`fix(lexicon): harden registry identity validation`), parent `40a94b4c8abf44954244b01a273fba221fd1b916`. The source worktree is clean at return. `git diff --check` passed. The explicit commit contains only the four iteration01 source paths listed below; prior candidate `40a94b4` and CDC's initial failed-verification record remain unchanged.

**Findings:**

- **F1 — repaired.** Shared document validation now rejects any alias (active or retired) owned by A when its exact text equals another source's current abbreviation. It returns sorted `AliasCollision` source IDs. Constructor, `from_json_bytes`, direct `SourceRegistry` serde, and `apply_document` all reject active and retired `A:FX-B` against current `B:FX-B`; apply leaves serialized registry bytes identical. Existing lifecycle tests continue to show valid retired-A/active-B alias-to-alias `OLD` reuse (active selection resolves B; include-retired is sorted ambiguity). The CDC-owned negative caller now prints `retired alias vs other current abbreviation rejected: true` and exits 0.
- **F2 — repaired.** Added private strict snapshot/record and attribution wire DTOs in `registry.rs`; only the registry-side `SnapshotAttribution` / `OccurrenceAttribution` deserializers route through them. Snapshot identity fields remain the same strings and record tuple values; the public serialization shape is unchanged. Unknown nested fields at all three specified paths reject through both the helper and direct serde. Valid registry JSON roundtrips byte-identically. A direct legacy `SnapshotIdentity` serde negative control remains permissive, and the existing captured-byte/identity/archive v1/v2 regression passes, showing the strictness did not move into legacy identity/archive serde. CDC's caller prints `unknown nested registry identity field rejected: true` and exits 0.
- **F3 — repaired.** Rustdoc and the user guide now state that `validate_lexicon` checks bindings against its supplied Lexicon and cannot authenticate an arbitrary later record supplied to `attribute`. Direct use requires a record obtained from that validated Lexicon or an independently established identical capture. `attributed_records` remains the validated whole-collection join. New control creates a bound-snapshot phantom locator through the existing identity serde route: registry construction succeeds, while both `validate_lexicon` and `attributed_records` return the exact `OccurrenceNotFound` identity.

**Current-candidate gates (all used `CARGO_TARGET_DIR=/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc01/target`; Rust 1.98.0):**

- `cargo +1.98.0 fmt --all` — exit 0.
- `cargo +1.98.0 test --locked -p vlajmaji-lexicon --test registry_contract` — exit 0, 9 passed, 0 failed.
- Preserved CDC public negative caller: `cargo +1.98.0 run --offline --manifest-path /tmp/vlajmaji-p01-a01-s01-cdc-caller/Cargo.toml` — exit 0; both formerly false controls print `true`. The CDC specimen was not edited.
- `make check` — exit 0. Format and workspace Clippy passed. Test populations: CLI 2; legacy lexicon contract 7 passed/1 ignored; registry contract 9; search unit 1 and integration 8; store unit 1; fixture 0; public identity 1; doctests lexicon 4, search 1, store 0; Make/planning tooling 19. Total **53 passed, 1 ignored**. Documentation generation passed. The one ignored test is the existing `full_corpus_import_reconciles_all_input_populations`, requiring `RELOJBAN_RESEARCH_ROOT`; it was not run and no full import is claimed.
- `make test-no-default` — exit 0, search integration 9 plus one doctest: **10 passed**.
- `make test-all-features` — exit 0: CLI 2; legacy lexicon 7 passed/1 ignored; registry 9; search unit 1 and integration 8; store unit 1; fixture 2; public identity 1; lexicon doctests 4 and search doctest 1. Total **36 passed, 1 ignored**; same existing corpus test ignored.
- `cargo +1.98.0 check --workspace --locked` — exit 0.
- `cargo +1.98.0 run --locked -p vlajmaji-lexicon --example source_registry` — exit 0; `5 captured farvi occurrences across two asserting sources`.
- The prior evidence-named outside-workspace path `/tmp/vlajmaji-p01-a01-s01-cc-caller.L9QA3A` was rerun with `cargo +1.98.0 run --offline --manifest-path .../Cargo.toml`; it exits 0 and currently prints the two successful negative-control results above. It therefore provides the public path-dependent control outcome at this candidate; the shipped example separately confirms the five-row positive result. This is the actual observed output at rerun, not a relabeling of the older recorded five-row output.

**Seven-row CC proposal:** S01-01 proposed done (role/type contracts remain additive and full fixture still exercises the distinct roles); S01-02 proposed done (current-abbreviation shadowing now fails for either alias status, while allowed alias reuse/retirement controls pass); S01-03 proposed done (5 farvi occurrences remain 4 from A, 1 from B, including distinct duplicate locators); S01-04 proposed done (raw bytes, tuple identity, field presence, lexical/assigned form distinction, proposal separation and archive roundtrip remain covered and the legacy regressions pass); S01-05 proposed done (registry construction/apply/helper/direct serde invariants, sorted collision, atomicity, nested unknowns and exact phantom binding failure); S01-06 proposed done (external public callers, shipped example, Rust1.98 gates and docs pass; corpus test is explicitly ignored); S01-07 remains **open for CDC** to independently reproduce this new candidate and own close/bubble-up. These are CC proposals only; the seven ledger rows remain open until CDC's canonical disposition.

**Files and artifact inventory:** source commit paths are exactly `/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc01/vlajmaji/crates/lexicon/src/registry.rs`, `.../crates/lexicon/src/attribution.rs`, `.../crates/lexicon/tests/registry_contract.rs`, and `.../docs/source-registry.md`. CC's only planning write is this existing `.../slice01-source-registry-and-assertions/artifacts/cc-execution.md`; no planning commit was made. Preserved verification inputs include `/tmp/vlajmaji-p01-a01-s01-cdc-caller`, `/tmp/vlajmaji-p01-a01-s01-cc-caller.L9QA3A`, and the original failure logs `/tmp/vlajmaji-p01-a01-s01-cdc-negative-alias.log` and `/tmp/vlajmaji-p01-a01-s01-cdc-negative-controls.log`. Build/test artifacts remain under the assigned Arc01 target. No full corpus, external source, publication, push or integration was run.

**Scope comparison:** This repair only closes the initial S01 correctness and documentation findings within the original registry/attribution contract. It makes no change to the accepted PoC/archive identity model or slice scope. A01 S02 still owns persistence, broad schema coverage, raw-preserving extensions, standards mapping and archive migration; A01 S03 still owns grouped exact lookup, counts/pages, availability and transition behavior. A01/Arc01 composition, independent S01 acceptance and all higher-scale rows remain open.

**Status:** source frozen and candidate returned proposed-done for independent CDC review. No ledger acceptance or planning close is recorded by CC.


## Final close and bubble-up

All seven rows are done with reproduced evidence at `11ec7017a375fa62ba101cb51d85c36034621b81`; `cdc-verification.md` contains the complete independent row walk, gates and original failed controls. Scope delivered equals S01 scope assigned; no deferral/no-op/silent drop. Source delivery is accepted on the local feature branch, not main integration or publication. S02/S03 carryover remains complete as above, including all twelve schema areas and actual grouped Rust/CLI lookup/page/availability/revision controls. Arc status is updated before next-slice preparation.

Planning close commit is the commit containing this record (derive with Git history); source commits are `40a94b4` and `11ec701`. Planning paths are this slice's plan/ledger/close/verification/retrospective and existing CC evidence, plus parent arc plan/ledger and project plan/ledger status references. Issued prompts are preserved. Supporting artifact inventory is `artifacts/cc-execution.md`; task-owned /tmp callers/logs and target outputs are referenced by the independent record, with the initial wrong-target attempt and shared executable-name anomaly retained.

CDC methodology analysis: `cdc-retrospective.md`. No arc or project composition acceptance is claimed by closing S01.
