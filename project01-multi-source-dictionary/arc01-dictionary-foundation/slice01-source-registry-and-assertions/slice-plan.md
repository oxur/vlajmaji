# Slice01 — Source registry and assertions

| Field | Value |
| --- | --- |
| project / arc / slice | project01-multi-source-dictionary / arc01-dictionary-foundation / slice01-source-registry-and-assertions |
| status | planned draft; no CC assignment or implementation |
| depends-on | Vlajmaji `4ee3bab`; Lexicon raw/occurrence/form/proposal contracts |
| blocks | A01 S02 schema persistence and S03 grouped queries |
| version / date | 1.0 / 2026-10-24 |

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

Current assignment: **none**. No `cc-prompt.md` has been issued, no refinement pass
has run, and no source writer has been dispatched. The source is unchanged on main.

## Version history

- 1.0 — 2026-10-24: first planned usable registry/attribution slice, split from
  the preserved broad schema increment without accepting or erasing its criteria.
