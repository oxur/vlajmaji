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
