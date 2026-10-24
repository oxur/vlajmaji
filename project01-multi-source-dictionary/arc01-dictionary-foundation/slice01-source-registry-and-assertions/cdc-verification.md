# Slice01 CDC verification

## Initial candidate review — 2026-10-24

**Reviewer:** `/root/vlajmaji_p01_a01_cdc`, independent of implementer `/root/cdc/p02a02_cc`; requested OpenAI gpt-6.1-sol/high, backend attestation unreported. **Candidate:** `40a94b4c8abf44954244b01a273fba221fd1b916`, clean feature worktree. **Assignment:** initial `cc-prompt.md`, planning `ba6e6c0`; seven rows. **Verdict:** changes required; every row remains open pending the current candidate's complete independent disposition.

Inspected complete new registry (879 lines), attribution (209), tests (805), example (192), guide (99), and all nine-file source diff. Existing import/archive/identity behavior is additive except making the existing duplicate-key helper crate-visible. Intake/readback was checked; its tuple-encoding wording and incomplete recorded arc read range were corrected before edits, without treating receipts as proof of acceptance.

### Reproduced observations and failures

- First independent `make check` omitted `CARGO_TARGET_DIR`, created a new task-local source `target/`, and exited 0. Log `/tmp/vlajmaji-p01-a01-s01-cdc-check.log`. This setup mistake is preserved; no source edit occurred and that output directory has not been deleted.
- Corrected `CARGO_TARGET_DIR=/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc01/target make check` exited 0, including six registry tests, existing default tests/docs and 19 tooling tests. Log `/tmp/vlajmaji-p01-a01-s01-cdc-check-task-target.log`. This reproduces default gates only; no-default/all-feature/current repair checks remain pending.
- Outside-workspace independent caller `/tmp/vlajmaji-p01-a01-s01-cdc-caller` copies the actual shipped example and adds contract-derived negative controls. `CARGO_TARGET_DIR=.../target cargo +1.98.0 run --offline --manifest-path /tmp/vlajmaji-p01-a01-s01-cdc-caller/Cargo.toml` exited 101. `/tmp/vlajmaji-p01-a01-s01-cdc-negative-alias.log` first reproduces F1; `/tmp/vlajmaji-p01-a01-s01-cdc-negative-controls.log` reproduces F1 and F2. The normal five-occurrence example assertions pass before the controls. These are actual failed rejection controls, not tests of a simulated validator.

### Findings and exact repair

| Finding | Evidence / significance | Required disposition |
| --- | --- | --- |
| F1 | `registry.rs` selector validation checks only active alias/current abbreviation collisions. Adding A's **retired** alias `FX-B` while B's current abbreviation is `FX-B` is accepted; the issued contract forbids any alias shadowing another source's current abbreviation. Outside caller prints `...rejected: false`. S01-02/05; correctness-grade. | Reject this candidate through the same validator, preserving allowed retired+active *alias* reuse and current typed errors/atomicity. Add constructor, apply/unchanged-state and serde controls. |
| F2 | Existing legacy `SnapshotIdentity`/`RecordIdentity` serde ignores unknown keys. Nested `future:true` inside a registry snapshot identity passes `from_json_bytes`, contradicting strict registry-wire rules and guide. Outside caller prints `unknown nested registry identity field rejected: false`. S01-05; correctness-grade. | Strict local wire DTOs for new registry attribution JSON, including nested snapshot/record identities, then convert unchanged legacy values. Do not tighten legacy archive/identity deserialization. Check helper and direct SourceRegistry serde, including occurrence nesting. |
| F3 | `attribution.rs` attribute docs and guide imply validate_lexicon proves membership of an arbitrary separately deserialized CapturedRecord. It checks registry bindings against its supplied Lexicon, not a different record later passed to attribute. Source-level documentation finding affecting S01-06. | Explicitly require records from a validated Lexicon for direct attribute; explain validate_lexicon's two-direction scope and that attributed_records is the whole validated join. Add a phantom occurrence-binding control showing OccurrenceNotFound and keep empty legacy bindings supported. No new public signature or inference. |

### Seven-row walk and scope/bubble-up

S01-01/03/04: complete source review and default regression evidence support distinct roles, five occurrences, raw bytes/presence/identity and form/proposal preservation; remain open until repaired-candidate disposition. S01-02: F1 blocks. S01-05: F1/F2 block and phantom occurrence control is missing. S01-06: default Make reproduction passed, but F3 documentation and remaining feature/external gates block final acceptance. S01-07: initial independent review is performed with a failed verdict; final review, close and bubble-up still required. None is deferred or silently dropped.

S02/S03 obligations and all twelve schema areas are unchanged. These are within-contract implementation/documentation repairs, requiring no scope/architecture amendment or new user approval. No source change by CDC occurred; reviewer-authored /tmp controls are verification only. Source mutation returns to CC only under preserved iteration01; CDC retains planning ownership.

## Refinement01 independent review and acceptance — 2026-10-24

**Candidate:** `11ec7017a375fa62ba101cb51d85c36034621b81`, clean feature worktree. **Assignment:** preserved `cc-prompt-iteration01.md`, planning issue `66510ee` plus metadata correction `3b727c3`. Same independent reviewer/implementer attribution as above. **Current verdict: accepted**, all seven rows reproduced; prior failed verdict remains applicable to 40a94b4 only. No source repair by CDC.

Read the complete four-file repair diff: all aliases are now checked against other current abbreviations; private strict snapshot/record wire DTOs only affect new registry attribution deserialization; the legacy identity/archive serde is unchanged; direct-record trust docs and phantom locator oracle are corrected. Nine registry tests include the three nested unknown-field paths through both helper/direct serde and atomic failure of both alias statuses. Source diff/whitespace and clean branch check passed.

All commands below used `CARGO_TARGET_DIR=/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc01/target` under Rust1.98.0, serialized against the frozen candidate. Fresh independent logs are `/tmp/vlajmaji-p01-a01-s01-cdc-repair01-{check,no-default,all-features,workspace-check,example,cc-caller,negative-caller}.log`.

- `make check`: exit0, 53 passed / 1 existing corpus test ignored: CLI2; legacy lexicon7; new registry9; search unit1/integration8; store unit1/public identity1; lexicon/search doctests5; tooling19. Format, Clippy, docs also pass.
- `make test-no-default`: exit0, search integration9 + doctest1 =10.
- `make test-all-features`: exit0, 36 passed / 1 ignored: same Rust population plus two native graph fixture tests; no tooling count included.
- `cargo +1.98.0 check --workspace --locked`: exit0. Shipped `source_registry` example: exit0, exact `5 captured farvi occurrences across two asserting sources`.
- Independent negative caller: exit0, both original rejection controls now print true. Its package is now uniquely named `registry-independent-controls`; source remains the preserved CDC specimen, only its temporary manifest package name changed.
- **Caller anomaly preserved:** rerunning the original CC path with the shared target emitted the negative caller's output although its source still contained the five-occurrence println. Both temporary packages originally used `registry-public-caller`; distinct source files/inodes and one shared output executable were observed. Do not label those anomalous reruns as clean positive-caller evidence. A new independent package `registry-independent-positive`, `/tmp/vlajmaji-p01-a01-s01-cdc-positive`, copies the unchanged shipped example and compiles/runs offline: exit0 with the exact five-occurrence summary. Log `/tmp/vlajmaji-p01-a01-s01-cdc-repair01-positive-unique.log`. The names are now distinct. No product source or CC specimen was edited.

### Final seven-row walk

| Row | Disposition / evidence strength | Reproduced evidence |
| --- | --- | --- |
| S01-01 | done / reproduced | Public fixture and full source review establish separate source/contributor/distributor IDs, evidenced/unresolved lineage, snapshot bindings, unknown author/native IDs. |
| S01-02 | done / reproduced | Nine-test suite plus external F1 control: rename/retire/history/ambiguity and both active/retired abbreviation shadow rejection, stable IDs. |
| S01-03 | done / reproduced | Full five-occurrence reference: A4/B1, same text/native ID at distinct locators, D1/D2 preserved; independent positive caller and example. |
| S01-04 | done / reproduced | Complete source diff, new fixture and existing seven legacy contracts: original bytes/record IDs/presence/unknowns/forms/proposals and archive v1/v2 semantics preserved. |
| S01-05 | done / reproduced | Atomic constructor/apply/helper/direct-serde controls, nested wire rejects, exact phantom OccurrenceNotFound, unknown/duplicate/reference/history errors. |
| S01-06 | done / reproduced | Shipped example/docs and unique outside-workspace positive/negative callers; fresh full required Make/Cargo/feature matrix above. |
| S01-07 | done / reproduced | Independent actual diff/intake/gate review, complete row/scope walk, canonical close and bubble-up checked; retrospective linked below. |

### Bubble-up and limits

S01 delivers the assigned in-memory registry/attribution increment on the dedicated local feature branch. It selects no SQLite topology or linguistic winner. Source groups, broad metadata/schema, etymology representation, persistence/rebuild, query totals/pages/availability/links and owned-revision transitions remain explicitly owned by S02/S03; no criterion has been discarded or marked deferred. Arc-plan changes are status/readiness follow-through only, no structural amendment required. Project completion and all later arcs remain open. Main integration/publication remain separately unauthorized. Full-corpus test is intentionally not run at this fixture-only scope. The former local source target and temporary caller anomalies are retained as distinct setup observations.
