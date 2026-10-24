# Arc01 — Dictionary foundation

| Field | Value |
| --- | --- |
| project / arc | project01-multi-source-dictionary / arc01-dictionary-foundation |
| status | active; S01 accepted11ec701; S02 typed-model readiness active; S03 persistence/S04 grouped pending |
| depends-on | Vlajmaji `4ee3bab`; preserved source/grouping/schema decisions and accepted PoC contracts |
| blocks | A02 full import, A03 optional releases, A04 composition and A05 owned editing |
| version / date | 1.3 / 2026-10-24 |

## Capability

A Rust caller can register named lexical sources and their distribution lineage,
load a complete small dictionary fixture under a broad versioned schema, and get
all exact-match assertions grouped by source, with stable abbreviations, provenance,
complete counts, pages and explicit availability. The CLI exposes the fixture
lookup by arc close. This is a useful dictionary foundation, not a full-corpus
import or a decision about one-file versus multi-file SQLite.

Use [the project design](../design.md), [the project plan](../project-plan.md) and
[this arc's ledger](ledger.md). Current authority is Autonomous execution through Arc01 only under `../cdc-directive01.md`. Source branch is `feature/project01-dictionary-foundation`; current contributor assignments live in the child plans.

## Current slice breakdown — explicit split v1.3

The initial three-slice breakdown below is historical. Original S02 proved too broad to hold its complete raw/typed model, relational projection, full12-area fixtures and repair headroom in one selected CC context. The approved plan already permits a scope-preserving split; root explicitly acknowledged it on2026-10-24. Current endpoint is the same complete Arc01 capability, now four useful slices.

| Slice | Useful result | Preserved original owner |
| --- | --- | --- |
| S01 source registry and assertions | accepted11ec701 | original S01, all7 rows closed |
| [S02 versioned dictionary model](slice02-versioned-dictionary-model/slice-plan.md) | validated raw/typed model, full12-area fixture, JSON roundtrip and exact legacy adapter | first part of original S02 |
| S03 relational persistence | concrete keyed/FK schema, field-level persisted oracle, strict reopen/rebuild/mismatch path | remaining original S02, all12 areas persisted |
| S04 grouped exact lookup | full Rust/CLI grouping/totals/pages/availability/links/history and complete reference | original S03 intact |

## Historical initial slice breakdown (v1.0; superseded only in decomposition)

| Slice | Usable result and intended files | Dependency / boundary |
| --- | --- | --- |
| [01 — Source registry and assertions](slice01-source-registry-and-assertions/slice-plan.md) | Public validated source/alias/lineage registry plus assertion-attribution views over preserved PoC occurrences, documented through an external Rust caller; lexicon crate and schema/design documentation. | Current Lexicon identity/raw/form contracts. No new SQLite DDL or production editor. |
| 02 — Versioned relational schema | Concrete broad metadata tables/types, raw-preserving extension policy, standard mapping and old-archive adapter/rebuild path; a fixture can persist/reload without loss. Store/lexicon crates and shipped schema/migration docs. | Accepted S01 IDs/attribution. All twelve schema areas in design.md are accounted for; a single fixture database does not select final topology. |
| 03 — Grouped exact lookup | Public Rust API and CLI show every fixture assertion under its registered source; exact totals, default-unpaged output, optional occurrence pages, availability and stable links. Search/CLI and user documentation. | Accepted S01 registry and S02 store. Complete reference checks before any real corpus/topology comparison. |

Only S01 is detailed. Future slices are prepared after accepted prerequisites.
If any slice exceeds one context with repair headroom, split it explicitly while
retaining every original criterion and owner; do not reduce schema coverage.

## Composition and discriminating fixtures

Use labelled synthetic records and pinned small source witnesses, not a new
full-corpus acquisition. The final fixture must include:

- Two asserting sources, two distributors of one upstream source, author IDs and
  an unknown author; same-spelling competing definitions and exact duplicate
  observations. Group source assertions without manufacturing corroboration.
- Alias rename/retirement/collision; stable references despite changed names/paths;
  missing, disabled and removed/re-added packs; an owned draft and revision change.
- Definitions, separate glosses, form-only records, notes/examples/labels,
  missing/null/empty and unknown nested fields; optional structured frames versus
  prose, unresolved references, and explicit editorial links.
- `farvi` alternatives; `si'e` as a cmavo versus assigned `si'e → snime`; source
  reported `cpan` versus unassigned LLG `cpana`; predictable `snim` kept computed;
  `sie`, dot, curly apostrophe, `y`, case and diphthong distinctions; multiword
  `le jaica` intact. Do not resolve semantics or word class by spelling alone.

For S03 compare the complete ordered occurrence reference to unpaged and all
positive-size page concatenations, including sizes 1 and several; size 0 has real
totals and no rows. Count occurrence multiplicity and full source groups before
limits; disclose partial groups. Reject malformed/stale cursors and explicit
unknown/ambiguous/unavailable source selections. No-match and unavailable are
separate states. Preserve legacy query behavior through an explicit new API or
versioned adapter rather than silently reinterpreting its limit field.

Pack removal/reinstallation and owned revision changes are **fixture contract
controls** here. Production optional package loading is A03/A04 and production
editing is A05. The schema includes their identities/history now; it need not
implement the editor to demonstrate the contract.

## Technical and verification boundaries

The accepted archive and immutable input bytes stay canonical; do not rewrite
v1/v2 data or existing tuple IDs. Use the existing `crates/lexicon`, `crates/store`,
`crates/search` and singular `crate/cli`. Source is the 0.2.0 development workspace;
code remains Apache-2.0. Content provenance/terms and standard mapping remain
explicit even on fixtures. Source scores, generated forms and computed place
parsing are not source authority or semantic truth.

Each slice ships its public entrypoint, useful example and documentation with
meaningful positive/negative checks under Rust 1.98. Use relevant repository Make
gates, crate tests, public external caller and schema/identity roundtrips. Feature
checks follow actual affected surfaces; preserve graph compatibility if shared
formats change. No importer, full data fetch or database benchmark runs in A01.

An independent reviewer verifies each implementation; the author cannot accept
her own repair. Arc close additionally demonstrates registry → persisted fixture →
grouped Rust/CLI lookup → all pages → removal/re-add/revision transitions, then
walks the arc ledger and bubbles findings into the project. Do not substitute
three child verdicts for that composition demonstration. Retrospectives apply if
execution is activated in Autonomous Mode; no close records are pre-created.

## Historical first-slice criteria and their new owners

The original unexecuted research Arc03/S01 ledger remains byte-preserved in the
research bundle. Its seven criteria are redistributed, not discarded:

| Historical research Arc03/S01 row | New implementation owner |
| --- | --- |
| S01-01 — registry, attribution, aliases | A01 S01, especially S01-01/S01-02/S01-05. |
| S01-02 — thorough schema and raw-preserving extensions | A01 S02; all twelve design.md coverage areas. |
| S01-03 — all-source grouped exact assertions | A01 S03, including contributor/distribution locators and owned fixture. |
| S01-04 — filters, full counts, deterministic occurrence pages | A01 S03; full unpaged reference and stale-query controls. |
| S01-05 — stable links/history through removal and owned fixture changes | A01 S02 catalogue/revision model plus S03 transition controls; A05 implements production editing. |
| S01-06 — source/computed/unknown distinctions | A01 S01 preservation, S02 schema, S03 query eligibility/explanations. |
| S01-07 — public caller, docs, Rust/schema and independent gates | Every affected slice plus A01-04's fresh composition demonstration. |

## Source-readiness and next action

S01's bounded baseline inspection is recorded in its plan. Before its eventual CC
assignment, finish the concrete API/ID/serde/error decisions, applicable Rust guide
intake and source/test readiness. Do not delegate unresolved domain semantics to an
implementer with a list of acceptance rows. Later DDL/query choices are owned by
their respective slice readiness; no topology decision is required for S01.

## Version history

- 1.0 — 2026-10-24: split the historical broad first schema increment into registry,
  persistence/schema and grouped-query slices, retaining all coverage and fixture
  oracles. No implementation or acceptance claimed.

- 1.1 — 2026-10-24: activated the already approved Arc01 authority; reconciled current metadata with the later authorization and source logo-only fast-forward. Scope and every ledger row unchanged.

## S01 close bubble-up — 2026-10-24

S01 delivered registry/attribution at11ec701 after one refinement; all7 rows independently reproduced. No structural slice/architecture change is needed. S02 retains every twelve-area schema obligation, including explicit source-asserted etymology under morphology/relations; S03 retains grouped Rust/CLI/totals/pages/availability/history controls. Source-asserted structure must not be confused with computed parsing. The next fixture/persistence packet will use distinct public-caller package names after the S01 shared-target executable anomaly. Root's read-only41/41 byte/hash/size check of preserved research is preservation evidence only, not implementation acceptance.

- 1.2 — 2026-10-24: S01 acceptance and retrospective completed; recorded the retired-selector/strict-wire repairs and explicit S02 carryover before next-slice preparation.

## Scope-preserving split decision — 2026-10-24

Original S02 → new S02/S03; original S03 → new S04. No criterion, test family, schema area, full fixture/control, public caller or independent gate is dropped. A01-02 closes only after both typed model and relational persistence; A01-03's grouped lookup owner is now S04. A01-04 composition walks all four accepted children, still demonstrating registry→persisted fixture→grouped Rust/CLI→all pages→removal/re-add/owned revision. Opaque/unsupported raw rows need a typed preservation/eligibility home before A02; the existing strict Lexicon parser remains unchanged. Missing catalogue data is explicit and cannot masquerade as available zero matches.

-1.3 —2026-10-24: CDC split original S02 into model and persistence after actual source/model sizing; root acknowledged the preserved four-slice endpoint. Added explicit old/new owner map and retained historical breakdown.
