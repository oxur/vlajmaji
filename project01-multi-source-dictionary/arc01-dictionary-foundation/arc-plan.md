# Arc01 — Dictionary foundation

| Field | Value |
| --- | --- |
| project / arc | project01-multi-source-dictionary / arc01-dictionary-foundation |
| status | active; S01 accepted11ec701; S02 accepted37c3518; S03 accepted01a9ceb3; S04 accepted efef9c4; separate composition pending |
| depends-on | Vlajmaji `4ee3bab`; preserved source/grouping/schema decisions and accepted PoC contracts |
| blocks | A02 full import, A03 optional releases, A04 composition and A05 owned editing |
| version / date | 1.14 / 2026-10-24 |

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

S01 and S02 are independently accepted. S03 persistence is independently accepted01a9ceb3after two preserved failed candidates and refinement02, all9rowsdone; S04 is prepared after accepted S03 interfaces and evidence.
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

For current S04 (original S03) compare the complete ordered occurrence reference to unpaged and all
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
four child verdicts for that composition demonstration. Retrospectives apply if
execution is activated in Autonomous Mode; no close records are pre-created.

## Historical first-slice criteria and their new owners

The original unexecuted research Arc03/S01 ledger remains byte-preserved in the
research bundle. Its seven criteria are redistributed, not discarded:

| Historical research Arc03/S01 row | New implementation owner |
| --- | --- |
| S01-01 — registry, attribution, aliases | A01 S01, especially S01-01/S01-02/S01-05. |
| S01-02 — thorough schema and raw-preserving extensions | Current A01 S02 typed model + S03 relational persistence; all twelve design.md coverage areas. |
| S01-03 — all-source grouped exact assertions | Current A01 S04 (original S03), including contributor/distribution locators and owned fixture. |
| S01-04 — filters, full counts, deterministic occurrence pages | Current A01 S04 (original S03); full unpaged reference and stale-query controls. |
| S01-05 — stable links/history through removal and owned fixture changes | Current A01 S02 catalogue/revision model + S03 persistence + S04 transition controls; A05 implements production editing. |
| S01-06 — source/computed/unknown distinctions | Current A01 S01 preservation, S02/S03 model/persistence, S04 query eligibility/explanations. |
| S01-07 — public caller, docs, Rust/schema and independent gates | Every affected slice plus A01-04's fresh composition demonstration. |

## Source-readiness and next action

S01, S02 and S03 independent acceptance are recorded in their close sets. S02 initial/refinement01 failed verdicts, repaired refinement02 acceptance and all issued prompts remain preserved in its child plan/verification/artifacts. S03's concrete DDL/lifecycle/integrity choices now bind its issued slice plan and cc-prompt.md; S04 query choices await actual accepted persistence interfaces. Do not delegate unresolved domain semantics to an implementer with a list of acceptance rows. No physical topology decision is required for the foundation fixture.

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

-1.4 —2026-10-24: S02 assignment issued at0b26224; reconciled current owner/status prose and original seven-row mapping with the four-slice split. Historical initial breakdown and S01-time bubble-up remain preserved.

-1.5 —2026-10-24: reconciled active status with independently failed S02 initial/refinement01 and current bounded refinement02. No criterion, architecture, decomposition or later-slice scope changed.

## S02 accepted bubble-up —2026-10-24

S02 broad immutable raw/metadata/JSON/exact archive adapter accepted37c3518 after preserved initial/R1 failed reviews and two refinements;9/9 child rows reproduced. Source/code/fixture/pin/error/role/history findings resolved within assigned contract. A01-02 remains OPEN until S03 concrete12-area relational persistence/strict reopen/tamper/rebuild is accepted. S04 grouped Rust/CLI/reference/pages/cursors/availability/link/owned transitions and independent all-four composition remain pending. No criterion/architecture/split changed; current S03 readiness consumes actual accepted model. See child close/verification/retrospective and artifacts/accepted-replay.json.

-1.6 —2026-10-24: recorded accepted S02 and complete relational/query carryover; arc remains active and composition unaccepted.

-1.7 —2026-10-24: issued source-grounded S03 concrete relational store assignment/9-row ledger after accepted S02; all12 areas and complete S04/composition obligations retained.

-1.8 —2026-10-24: reconciled stale intermediate S02 status prose with accepted37c3518 and active S03; preserved prospective independent SQL/public-control specimens. S03 first-save disk interruption and bounded known-cache recovery are recorded in its artifacts/s03-disk-recovery.md; same initial assignment resumed, no criterion or iteration changed.

-1.9 —2026-10-24: S03 independent initial source/gates/47-table/negative/positive/legacy review retained; F1–F7 bounded four-path repair issued. All9S03rows stay open; S04 and all-four composition/arc close remain pending.

-1.10 —2026-10-24: S03 R1 strict field/14positive/94+18+3+13negative/3legacy controls pass independently; two metadata query category checks and committed role regression remain open for bounded R2 API/tests repair. DDL/model/fixture preserved; no S04 or arc acceptance.

-1.11 —2026-10-24: S03 all9rows independently accepted01a9ceb3after final78/1check,10no-default,61/1all-feature,locked1.98and37public/SQL/legacy+metadata3proofs. A01-02closes with S02 model/S03 store. No structural scope change; S04 full query/CLI/reference transitions and separate composition remain pending.

-1.12 —2026-10-24: issued S04 concrete indexed candidate/search response/page/cursor/availability/reference/Rust+CLI contract at slice04-grouped-exact-lookup/cc-prompt.md,8open rows. Accepted S03 DDL/model/fixture and legacy query behavior unchanged. Separate all-four composition/arc close remain mandatory.

-1.13 —2026-10-24: S04initial ba580d0 independently reproduces275behavior controls and84/1check,15isolatedno-default,67/1allfeatures,locked1.98/example; narrow existing association-index/doc requirement remains. Issued bounded refinement01, preserving all8rows/scope and initial evidence. Separate composition/arc close still required.

## S04 accepted bubble-up —2026-10-24

S04accepted efef9c4after one narrow refinement; all8rows reproduced, complete275public/CLI value controls+4prior-cursor compatibility,84/1check,15isolatedno-default,67/1allfeatures,locked1.98/example. Original groupedS03scope and all12areas retained; no structural change. All4childrenaccepted, A01-03done. A01-04separate-context composition and final preservation/scope rows remain pending; accepted children alone do not close arc.

-1.14 —2026-10-24: recorded independent S04acceptance/close/retrospective and full scope retention; next mandatory independent composition at exact efef9c4, noA02activation.
