# Slice01 — Source registry and assertions

| Field | Value |
| --- | --- |
| project / arc / slice | project01-multi-source-dictionary / arc01-dictionary-foundation / slice01-source-registry-and-assertions |
| status | active; initial candidate 40a94b4 reviewed; iteration01 active; acceptance pending |
| depends-on | Vlajmaji `4ee3bab`; Lexicon raw/occurrence/form/proposal contracts |
| blocks | A01 S02 schema persistence and S03 grouped queries |
| version / date | 1.2 / 2026-10-24 |

## Useful result and before/after

Today a captured row has a snapshot namespace, byte hash, locator and caller-supplied
source metadata. A consumer cannot reliably tell whether two distribution snapshots
represent one asserting dictionary, resolve a historical source abbreviation, or
attach an author without conflating those roles.

After this slice, a public Rust caller can register dictionaries, contributors and
distributors, record explicit evidenced lineage, resolve aliases, and obtain
source-attributed assertion/occurrence views over the **same preserved PoC data**.
Two distributors of one source group under that source without erasing either
observation. A competing source stays separate. This is a small usable library
increment with examples, not a schema-only note.

## In scope

1. A validated source registry with stable source identities, display names,
   standard-versus-project-assigned abbreviations, historical aliases/retirement,
   explicit ambiguity/collision errors and separate contributor/distributor IDs.
2. Evidence-bearing lineage and snapshot attribution records. Unknowns stay
   unknown; an unverified relationship cannot automatically merge namespaces.
3. Assertion-attribution views/envelopes that refer to existing immutable
   occurrences, optional supplied entry/definition IDs, author, source and
   distributor/snapshot. Preserve separate duplicate occurrences and unknown IDs.
4. A labelled finite fixture and an external public Rust caller that demonstrate
   these contracts, validation errors, stable references after rename, and the
   unchanged raw/proposal/form boundaries. Ship the example and API documentation.

Likely source home is `crates/lexicon/src/` with focused registry/attribution
modules exported from `lib.rs`; tests extend `crates/lexicon/tests/`. Final module
and type names, validation rules and serialization are resolved in readiness
before a CC prompt, not left as consequential implementer choices.

Out of scope: SQLite schema changes, pagination, full corpus download/import,
package feature wiring, database topology, a production editor, linguistic
inference or standard export. A01 S02 owns concrete broad metadata persistence;
S03 owns grouped lookup transport/count/page behavior. The complete project
schema contract remains required, not deferred away.

## Existing behavior to preserve

`Lexicon::import_source` accepts caller-owned bytes, checks an expected hash before
parsing and mutates only after validation. Same namespace/hash is idempotent;
new hash is a distinct snapshot. `lookup_exact` remains byte exact and returns all
retained source rows. Captured source records, assigned/source-reported forms,
explicit references and proposals remain separate.

Keep `SnapshotIdentity`, `RecordIdentity`, raw-field presence and existing
`vlajmaji-store::record_id` semantics. An enriched catalogue refers to these IDs;
it does not rename their components or use a display abbreviation as identity.
Any serialized registry has its own explicit version and validation path, including
untrusted deserialization; no public constructor invariant may be bypassed by
`Deserialize`. Do not silently alter v1/v2 archive interpretation in this slice.

Legacy `SourceMetadata::new` is currently infallible and retains supplied empty
values. Keep that archive/import behavior; validation of the new registry is a
separate contract. An unbound legacy namespace needs an explicit attribution
error/state in the enriched view, not invented provenance or a rewritten capture.

## Finite discriminating fixture

Use two synthetic asserting sources A/B; distributors D1/D2 both distributing A;
one named author plus unknown author; same spelling with competing definitions;
and two byte-identical observations at distinct locators. Include a rename and a
retired alias, an ambiguous/colliding alias attempt, an unresolved lineage edge,
and a malformed registry reference. Labels are explicitly synthetic, not invented
community-standard abbreviations or evidence of real lineage.

Retain separate `si'e` lexical and assigned-form facts, exact `sie`, and spelling
case/apostrophe distinctions through attribution. Missing/null/empty and unknown
nested raw fields must survive access and archive roundtrip. An owned draft is a
separate origin, not a new upstream assertion; computed analysis remains labelled.
The small fixture is enough for this slice; broad schema and cursor controls land
with their owning slices.

## Verification and exit

The [seven ledger rows](ledger.md) define acceptance. Demonstrate the public API
from outside the crate/workspace, with exact expected IDs, source/lineage roles and
occurrence counts—not merely that a command exited successfully. Alias rename
must preserve identity; failed registration/invalid deserialization must leave the
registry unchanged. An ambiguous alias is an error; unknown lineage is not guessed.

Compare original raw bytes, occurrence IDs, field presence and form/proposal roles
before and after attribution/archive roundtrip. Retain relevant existing lexicon
contract tests. Run Rust 1.98 format/check/tests/Clippy/docs for affected packages,
and workspace compatibility checks appropriate to the actual API change. Use
negative controls that distinguish conflation, silent deduplication and bypassed
validation. Do not run new full imports, comparison benchmarks or publication.

Implementation is proposed-done until independently reproduced and dispositioned.
A reviewable branch is not integrated delivery; activation must name the intended
integration branch/owner and its acceptance gate. Five refinement iterations
remain the cap; substantive scope/semantic changes return to Duncan and CDC.

Artifacts: `artifacts/` inside this slice when supporting evidence is produced.
Actual reusable code, examples and user documentation belong in the source repo.
No artifacts, close report, retrospective or success verdict is prefilled now.

## Author readiness and assignment history

Current bounded inspection covered Cargo/README/crate roles, the lexical public
identity/import/access API, store identity encoding and existing SQLite DDL at
`4ee3bab`. It establishes the gap and preservation targets; it is **not** completed
implementation-prompt readiness or a fresh runtime verification.

Before issue: revalidate branch/revision and dirty inputs; finish relevant source
and contract-test inspection; load framework prompt-authoring plus applicable Rust
API/type/serde/error/documentation guidance; resolve concrete ID/alias/error and
serialization choices; specify observable fixture outputs; establish implementer,
independent verifier, approved settings, source branch and authorized endpoint.

Current assignment: **`cc-prompt-iteration01.md`**, following failed initial candidate review. No executed refinement pass yet. Current source baseline is the clean isolated feature branch at `40a94b4`. Initial assignment started at `e0630d8`, whose only changes from original `4ee3bab` were the user's two logo commits; the registry implementation then landed at `40a94b4`.

## Version history

- 1.0 — 2026-10-24: first planned usable registry/attribution slice, split from
  the preserved broad schema increment without accepting or erasing its criteria.

## Completed author readiness — 2026-10-24

CDC inspected lexicon `lib.rs` lines 1–885 (identity, capture, forms, import, access and archive), complete `tests/contract.rs` (466 lines), workspace/lexicon manifests, Makefile, store identity encoding, and existing crate layout. Original parser details remain untouched in S01. Loaded framework project-management wayfinder/scales/layout/top-down, methodology role and Autonomous sections, full prompt-authoring guide, work-verification ledger/row/independence contracts; Rust scoped API/type/error/doc/anti-pattern/core sections underpin the decisions in the prompt. No runtime baseline success is claimed.

Readiness is source-grounded, design-complete for S01, executable and falsifiable: separate validated ID newtypes; explicit exact alias/retirement/ambiguity policy; versioned registry with constructor-equivalent deserialization; atomic document validation with preserved alias/binding history; attribution borrows preserved captures and errors on missing provenance. Full signatures, wire shapes, tests and applied guide rationale are in `cc-prompt.md`. No SQLite or archive change is authorized. Schema and grouped query obligations remain S02/S03. Packet is bounded to approximately 12–18k tokens of mandatory text and code before implementation, leaving repair headroom; no historical corpus scan is required.

Durable supporting artifact: `artifacts/cc-execution.md` is needed for intake/readback, exact gate observations and CC per-row proposal without giving CC planning/index ownership. CDC incorporates its attestation into canonical close records only after review. An outside-workspace caller under task-owned `/tmp` is retained by exact path in that log.

## Assignment history

| Assignment | Date | Predecessor | Reason | Disposition |
| --- | --- | --- | --- | --- |
| `cc-prompt.md` | 2026-10-24 | none | Initial source registry and assertion increment after CDC readiness | returned; candidate 40a94b4; changes required after F1/F2/F3 review |

CDC `/root/vlajmaji_p01_a01_cdc` independently reviews the frozen candidate; CC is explicitly dispatched OpenAI `gpt-6-luna` / high. All seven rows remain open until independently reproduced. Feature branch is the accepted local delivery destination; no main integration or publication.

- 1.1 — 2026-10-24: completed engineering readiness, preserved initial assignment and coherent current authority; original baseline and carryover remain explicit.

### Dispatch observations and ownership reset

Fresh `spawn_agent(model=gpt-6-luna, reasoning_effort=high, fork_turns=none)` returned `agent thread limit reached`; no new CC started. A local ephemeral CLI attempt then exited 2 before any contributor started: the installed `codex` launcher requested a profile selection. No implementation occurred in either attempt. Diagnostic logs remain `/tmp/vlajmaji-p01-a01-s01-cc-{events.jsonl,stderr.log}`.

Root established that completed `/root/cdc/p02a02_cc` was explicitly spawned in this conversation as OpenAI `gpt-6-luna` / high / `fork_turns:none` and instructed reuse without changing its model/settings. This is model continuity, not silent substitution; backend attestation remains unreported. It receives the preserved initial prompt with a complete current intake and explicit ownership reset. CDC `/root/vlajmaji_p01_a01_cdc` reviews independently; the former parent/root retains read-only boundaries. The context is not fresh: retain awareness of historical context load and use renewed assignment/source readings, not predecessor acceptance, for this work.

| `cc-prompt-iteration01.md` | 2026-10-24 | `cc-prompt.md` | Reproduced alias/strict nested wire defects and direct-attribution trust-boundary docs | issued; refinement01, source 40a94b4 |

Current source ownership returns to CC `/root/cdc/p02a02_cc` for iteration01 only; same approved model/settings, scope and five-refinement cap. CDC remains independent reviewer; no source repair authored by CDC. First candidate and failed controls remain in `closing-report.md`, `cdc-verification.md` and `artifacts/cc-execution.md`.

- 1.2 — 2026-10-24: independent initial review failed two discriminating controls and a documentation boundary; issued preserved refinement01 without changing scope or any criterion.

Metadata correction at refinement01 intake: the old current-source sentence still named the initial e0630d8 input; corrected to 40a94b4 while retaining initial baseline history. No issued prompt or source contract changed.
