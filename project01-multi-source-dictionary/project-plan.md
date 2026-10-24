# Project01 — Multi-source dictionary

| Field | Value |
| --- | --- |
| project | project01-multi-source-dictionary |
| status | project OPEN; Arc01 CLOSED at original accepted revision; Arc02 ACTIVE/readiness; A03–A06 not activated |
| depends-on | Vlajmaji PoC at `4ee3babefa13fbde41c94ab9bfcf61bb029834fe`; bounded accepted research described below |
| blocks | Reusable source-grouped vocabulary workbench and source-backed wordcraft experiments |
| related | Bauske; Relojban-poetics Project02; Bō Gē's “the last snow”; Open Tone Harmony |
| source-home | `/Users/oubiwann/lab/oxur/vlajmaji` |
| planning-home | orphan `planning` branch, `.worktrees/planning/project01-multi-source-dictionary/` |
| version / date | 1.7 / 2026-10-24, America/Chicago |

## Purpose and first useful result

Give a Rust caller and the `vlajmaji` CLI a dictionary that can say **who says
what**, show competing definitions together under their named sources, and keep
source history available while the user explores words and compounds. The
motivating consumer remains the opening of “the last snow”: “on the old plum / the
snow     is opening.” This project supplies trustworthy lexical material for that
exploration; it does not settle the poem's translation or interpretation.

The first usable increment is a registry-backed, source-grouped exact lookup on a
small fixture, with source abbreviations, all assertions, locators, exact totals
and explicit availability. Subsequent arcs make it work with full, independently
selectable corpora and an editable dictionary of our own.

The design and remaining decisions are in [design.md](design.md). Acceptance is in
[ledger.md](ledger.md). Arc01 has four independently accepted children and accepted separate technical/final packet composition; it is CLOSED. Arc02 is now activated by the renewed authority below; A03–A06 remain unactivated.

## Decisions already made

- SQLite is the relational storage direction. Keep the canonical raw/normalized
  source material and make indexes rebuildable. The physical composition strategy
  is still open.
- Preserve each source assertion and occurrence. Default dictionary display groups
  definitions by named source and a stable abbreviation; it selects no winner.
- Separate authors, asserting dictionaries, upstream lineage, distributors,
  collections, snapshots, normalized releases and occurrences. Distribution copies
  are not independent linguistic attestations.
- Use a thorough DMLex/OntoLex-informed metadata model, with an explicit mapping
  and loss register. Neither names nor a relational schema establish conformance.
  ELEXIS is a possible later service/interchange boundary, not the internal schema.
- Give every required corpus its own optional data crate and consumer Cargo
  feature. Core libraries have no compulsory dictionary payload.
- Dedicate an increment to scripted import/transformation of the **raw**
  jbotci-vendored Lensisku export; its selected compiled dictionary is insufficient
  for the all-assertions contract.
- Treat our editable dictionary as another first-class source, with revisions,
  explicit editorial links and an optional sealed snapshot package.
- Automate reproducible replay/check/update commands. Ordinary builds and loads
  are offline. A scheduled refresh, unattended publication and live authenticated
  Lensisku access have not been requested.

These are recovered user decisions, not new recommendations. The original
[Arc03 draft](artifacts/research/relojban-poetics/.worktrees/planning/project02-wordcraft-reconnaissance/arc03-multi-source-dictionary/arc-plan.md)
is preserved separately; this plan supersedes its **future implementation routing
and decomposition** for Vlajmaji only. It does not rewrite or close that research
project's ledgers.

## Definition of done

A documented public Rust API and CLI can select the required independent corpus
packages and the owned source; perform source-grouped exact dictionary lookup;
retain all conflicting definitions and provenance; traverse explicit stable links;
and distinguish unavailable data from a genuine empty result. Existing search
families continue to work with their declared source/analysis policies.

The full jbotci raw import is reproducible with balanced row/field accounting.
The LLG list family, La-Lojban and Sutysisku also have independently selectable,
versioned normalized releases with source/component manifests and notices. The
owned dictionary supports append-preserving revisions and reproducible exports.
A complete reference result verifies counts, multiplicity, pages, source changes
and links across the selected SQLite topology.

Completion requires independent slice and composition acceptance plus Duncan's
project acceptance. Public crates.io/data publication is a separate decision:
internal packages and replay must work even where a release's public redistribution
status is explicitly held. A held corpus cannot silently disappear from scope.

## Boundaries and preservation

In scope: source registry and aliases; broad lexical metadata with raw-preserving
extensions; full import/replay; independent corpus packages/features; SQLite
composition; owned revisions; grouped Rust/CLI queries; migration/rebuild guidance;
and meaningful offline/public-caller checks at each usable increment.

Retain PoC raw bytes, v1/v2 archive interpretation, existing occurrence identity
encoding, source-native fields, form roles, proposal separation, query policies and
explicit errors. Any incompatible new format or API transition must be named,
versioned, documented and tested. Do not reinterpret historical IDs as new registry
IDs or silently rewrite an old capture. Bauske's phonology remains a separate
unchanged dependency unless a separately authorized change is needed.

Outside this project: a new linguistic theory or perceptual sound metric; a new
compound engine; automatic inference of etymology, senses or place structures;
semantic embeddings; a hosted dictionary service; a GUI; certified DMLex/OntoLex
export; exhaustive corpus discovery; and a complete poetic translation or singing
trial. Existing source-backed morphology/sound queries remain useful and tested;
new linguistic or artistic work stays with Bauske/the research project.

## Arc roadmap

Arc IDs are local to this new project; dependencies below, not numbering alone,
define execution order. Later arcs are roadmap entries, not issued assignments.

| Arc | Usable capability and destination | Required inputs | Detail now |
| --- | --- | --- | --- |
| [01 — Dictionary foundation](arc01-dictionary-foundation/arc-plan.md) | Registered sources, broad versioned metadata/schema and grouped exact lookup over fixtures; existing lexicon/store/search crates and user documentation. | Current PoC, pinned research and explicit source roles. | Four scope-preserving slices after S01 sizing; current detail in arc plan. |
| 02 — Reproducible full jbotci import (ACTIVE/readiness) | Explicit acquire/replay/check/update path imports the pinned raw export with every alternative, original bytes and reconciled diagnostics; importer API plus Bash/Make entrypoints and normalized release. | Accepted A01 schema/identity/query contracts. | Detailed source-grounded planning now; see current authority. |
| 03 — Independent corpus packages | Dedicated opt-in packages/features for jbotci/Lensisku, LLG components, La-Lojban and Sutysisku, usable individually and together without build/load network access. | A01 contracts and A02 full import; pinned source/component inputs. | Plan per collection; split packaging work rather than omit a collection. |
| 04 — Composed SQLite lookup | Compare unified derived index, ATTACH and application fanout on the same complete contract, then implement the selected composition and availability/query-snapshot policy. | A01 queries and A03 real pack releases. | One bounded decision followed by usable integration; no topology selected now. |
| 05 — Owned dictionary | First-class editable source, revision/history and explicit editorial relationships, immutable export/reimport and its own optional data crate. | A01 model plus accepted A04 composition. | Model is reserved in A01; production editing belongs here. |
| 06 — Integrated dictionary workbench | Rust/CLI source selection and grouped output across all packs and owned revisions; old query-family regression, removal/reload, paging, documentation and clean offline replay compose end to end. | Accepted predecessors. | Final integration adds remaining behavior; it must not postpone A01's usable lookup. |

Every implementation slice includes its relevant API/CLI entrypoint, documentation,
validation and independent review. Arc04's comparison is a real unresolved
engineering decision, not a repeated SQLite-versus-graph survey. Its precise
question, finite workload, effort bound and stop/re-entry owner must be agreed
before running it; the three required candidates remain in scope.

## Carryover without scope loss

| Preserved research Arc03 increment | New owner |
| --- | --- |
| S01 registry, thorough schema, grouped exact/count/page contract | A01 S01–S04 after explicit split; complete typed/schema coverage in S02/S03 |
| S02 raw jbotci import | A02 |
| S03 all four upstream optional corpus collections | A03; one collection cannot stand in for the others |
| S04 unified / ATTACH / fanout comparison and topology decision | A04, before dependent multi-store/owned integration |
| S05 owned revisions and its optional package | A05 |
| S06 grouped API/CLI, complete replay and composition | A06; early fixture lookup remains A01 |
| Cross-cutting source terms, provenance, references and review | Every affected slice plus project rows P01-12–P01-14 |

The research project's five original artistic/readiness obligations remain there,
open. This plan neither completes them nor restarts the parked phonology arc.

## Open decisions and who resolves them

| Decision | Status / owner / gate |
| --- | --- |
| Concrete registry IDs, alias validation and source-lineage records | Resolved and independently accepted A01S01at11ec701; exact aliases/history/attribution compose at efef9c4. Existing IDs preserved. |
| Concrete DDL, archive/normalized version transition and standards mapping | Resolved A01S02model37c3518+S03store01a9ceb3; explicit versions/legacy adapter/rebuild and standards loss register, independently composed. Production topology remainsA04. |
| Full raw jbotci shape, observed counts, unknown/wiki-row eligibility | A02 importer; supplier counts are expectations to reconcile, not an oracle to force. |
| Corpus component boundaries, common pack API and feature host | A03 before package assignments; separate source/distributor identities are already required. |
| Unified SQLite / ATTACH / fanout, ranking policy and query consistency | A04 decision after correctness comparison, before dependent integration. No merging local FTS top-k as if ranks were global. |
| Owned editor conflict/revision semantics | A05 before production editing; preserve prior assertion references. |
| Public redistribution for each release | Source/release-specific evidence and notices; public publication held until resolved. Current code remains Apache-2.0. |
| New service/interchange implementation, UI or artistic extension | Outside current scope; a later explicit decision. |

We have enough to plan; these are timed decisions with consumers, not reasons to
repeat discovery. If a required input remains unavailable, report the exact
blocked obligation and seek an explicit amendment. Do not substitute a smaller
corpus or weaker oracle without recording the scope change.

## Historical Arc01 authority and contributor workflow (superseded endpoint)

On 2026-10-24 Duncan approved starting the first slice in Autonomous Mode and
working straight through **Arc01 — Dictionary foundation**, explicitly selecting
OpenAI **GPT 6.1 Sol High** for CDC and **GPT 6 Luna High** for CC. This supersedes
the initial planning-only restriction for Arc01, not for the later project arcs.
Endpoint: complete Arc01 through its four scope-preserving children (originally three), corrections, independent reviews, formal arc
composition/close and CDC retrospectives; stop before opening Arc02.

The selected workflow is CDC + CC. Root `/root` performs preflight/dispatch and
then remains read-only over source and planning while assigned CDC owns the arc.
The preserved [CDC assignment](cdc-directive01.md) supplies the full operational
contract; CDC records its actual returned context identity and acknowledgement.
Requested configuration is distinct from unreported backend attestation.

| Resource | Active owner / boundary |
| --- | --- |
| Source | Assigned CC owns mutation/index after each preserved prompt; CDC reads/reviews against a stable candidate. |
| Source worktree | `/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc01/vlajmaji` |
| Source branch / start | `feature/project01-dictionary-foundation`, from `4ee3babefa13fbde41c94ab9bfcf61bb029834fe` |
| Planning | Assigned CDC owns this orphan `planning` worktree and all scope/ledger/prompt/close records after handoff. |
| Build output | Task-owned target directory under `.worktrees/project01-arc01/`; one writer at a time, preserved failures and explicit cleanup ownership. |
| Bauske | Read-only sibling symlink at `.worktrees/project01-arc01/bauske`, target `/Users/oubiwann/lab/oxur/bauske`, start `7e7818c3d5a86a67b4fb15519d8bf078fa032627`. |
| Main/research | Main checkout and untracked assets, old research and old planning remain outside implementation ownership. |

Local scoped commits on the feature and planning branches are authorized by
Autonomous/Expedited Mode. No push, PR, main integration, release, new corpus
acquisition, deployment, schedule or authenticated service action is part of this
arc instruction. No `.github/workflows` exists at the source baseline; recheck
external effects before any later separately authorized push. Use independent
`cdc-verification.md` records; a fresh selected-model context verifies higher-scale
composition when CDC assembled it. No CRC workflow has been selected.

Before each CC issue, CDC completes source/domain-guided design and prompt
readiness, records the assigned context/approved settings and stable input state,
and provides the full open set plus required-reading/intake manifest. Preserve
each issued prompt; at most five refinement iterations per slice. A substantive
repair by a reviewer needs another independent verifier. Keep all rows and
cross-scale bubble-up/retrospective duties; do not weaken evidence to advance.
Consequential architecture/scope changes remain Duncan's decision. Ordinary
engineering choices within this contract belong to CDC; no repeated permission
question is needed for the already approved roles or endpoint.

## Current Arc02 authority and contributor workflow — 2026-10-24

Duncan instructed: “okay ... I think we're ready to dive into arc02! Same Autonomous Mode, same model config/setup :-)”. This renews OpenAI GPT 6.1 Sol High CDC / GPT 6 Luna High CC for Arc02, including slice design/implementation/repairs/independent acceptance, composition, formal close/bubble-up and all retrospectives. Stop before A03. [cdc-directive02.md](cdc-directive02.md) is the preserved current assignment; directive01 and the preceding authority section are historical, not current corpus restrictions. Requested/dispatched settings are distinct from unavailable backend attestation.

Assigned CDC `/root/vlajmaji_p01_a02_cdc` acknowledged; root remains read-only. CDC owns planning/source setup; CC owns isolated source mutations only during issued intervals. Fresh source branch/worktree will start from verified clean main `d1e1c928f7999f2220b046cc5a5817f0529312ca`; no main mutation, push, PR, publication, integration, Bauske mutation or later-arc work is authorized. Current source has five `crates/*` packages, `crates/cli`, umbrella crate, Rust1.98 and Bauske0.3.0 pin `c5801ae17caaf86d2651b5ce34570199e873ad0f`; source AGENTS and CI were fully read. Arc01 original acceptance remains efef9c4; later merged corrections/release changes require their own current evidence.

Arc02 expressly authorizes necessary public acquisition/read of pinned jbotci raw `dictionary-en.json` plus metadata at `3d9fd41d9ed32de26311ee27ec86b6b165985aa1`; no live authenticated Lensisku, exhaustive discovery or silent repin. Full rows/fields/alternatives/native accounting, normalized output independent of SQLite, useful Rust/CLI/Bash/Make, offline replay/check/update and complete persist/reopen/grouped querying remain required. Data redistribution held; no massive raw payload in public code. Plan slices after actual source/data reconnaissance. No new dependency without demonstrated need.

Preflight: planning clean5531f604/ahead54 at takeover; main clean d1e1c928, old feature clean a7c250f. Data volume2.6GiB free. Prior designated A01target10GiB and source-local68MiB are protected pending justified cache ownership/handover; no blind cleanup. Every Cargo/Make command must set a task-owned target through fixed launcher, with one writer; serialize imports/snapshots/scans. Independent callers require unique package and binary identities/offline locks. Readback follows complete required bodies and whole-population data queries; preserve failed attempts. Fresh separate SolHigh context gates assembled technical/final packet composition; no CRC inferred. Local scoped commits authorized.

## Research status and next action

[Research guide and verification status](artifacts/research/README.md) distinguishes
preserved authored reports, source captures, authoritative historical gates and
new design choices. [copy-manifest.tsv](artifacts/research/copy-manifest.tsv) records
exact original paths, revisions, SHA-256 and byte sizes for every copied file.
Old source pins and measurements remain dated evidence, not current service or
performance claims. No full corpus, database or Cargo build cache was copied.

Next: complete Arc02 required intake, resource handover and current-source/raw-shape reconnaissance, then open the canonical arc/slice plan and implementation assignment. Arc01 remains closed at its original accepted source; all14projectrows remain open. Planning existence/copy fidelity alone are not implementation acceptance.

## Planning validation — initial draft checkpoint

The 2026-10-24 planning pass checked all 41 copied files against their originals
with byte comparisons, SHA-256, byte sizes and the two recorded original Git
revisions. New authored Markdown local links and whitespace checks passed; all
14 project, 6 arc and 7 slice implementation rows remain open. These are planning
and preservation checks, not runtime tests or independent implementation acceptance.

Original research and its planning worktree stayed clean at the recorded HEADs.
Vlajmaji's tracked source/index and HEAD stayed unchanged at `4ee3bab`. An unrelated
untracked `assets/` directory appeared in the source checkout during planning;
it is outside this writer's ownership and was left untouched. The new documents
were preserved as approved review candidates, then committed at `35e368f` before execution readiness.

## Version history

- 1.0 — 2026-10-24: recovered the unexecuted research Arc03 obligations into
  Vlajmaji Project01, expanded them into six usable arcs, split the broad first
  increment into three foundation slices, and preserved selected research under
  project-level artifacts. All implementation rows remain open.

- 1.1 — 2026-10-24: Duncan approved Autonomous execution through Arc01 and explicitly selected CDC GPT 6.1 Sol High / CC GPT 6 Luna High. Recorded isolated feature worktree, stable ownership and one-arc endpoint; later arcs remain unactivated.

## Active CDC preflight — 2026-10-24

Assigned CDC `/root/vlajmaji_p01_a01_cdc` acknowledged `cdc-directive01.md`. Requested OpenAI `gpt-6.1-sol` / high; tool-dispatched configuration is distinct from unknown backend attestation. Planning owner is CDC; source owner passes to the assigned CC only during issued intervals. Root remains read-only. Approved substrate commit: `35e368f`.

Original source baseline remains `4ee3babefa13fbde41c94ab9bfcf61bb029834fe`. During readiness Duncan's separate logo work advanced main through `08c6bdb` to `e0630d8279b9ea48ce5892e201a5c50c52c3a469`. CDC inspected the exact two commits: only `assets/images/v1-250.png` and `assets/images/v1-1254.png` changed. On root's explicit instruction the clean isolated feature branch was fast-forwarded to `e0630d8`; no main or asset mutation occurred. Main's untracked Emacs README buffers are protected. Source APIs remain the inspected `4ee3bab` code.

Task output owner: `CARGO_TARGET_DIR=/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc01/target`; serialize checks. Source integration destination is the local feature branch, independently accepted at slice and arc scale; main integration remains a later separately authorized action. Next current assignment is recorded at slice scale. No A02 activation or external action is authorized.

## Arc01 current progress — 2026-10-24

S01 is independently accepted at11ec701 (registry and borrowed attribution, one repair); its seven-row close/verification/retrospective are complete. S02 typed model/JSON/exact legacy adapter is independently accepted37c3518 after preserved initial/R1 failures and two refinements; S03 relational persistence is independently accepted01a9ceb3after two refinements; S04 grouped lookup is accepted efef9c4after one refinement and separate technical composition passed at the same exact source. Final canonical packet gate accepted at7ddd0db; Arc01formallyCLOSED. All project rows remain open: this first increment does not supply full corpora, final topology, production editor or project acceptance.

Arc01 decomposition update (2026-10-24): original S02 now has typed-model S02 and persistence S03; original grouped S03 moves intact to S04. Root acknowledged this authorized split. Endpoint remains the full Arc01 foundation; all later arcs remain unactivated. Historical three-slice directive describes preserved capability scope, not permission to omit the fourth child introduced to retain implementation/review headroom.

-1.2 —2026-10-24: reconciled current Arc01 progress with S02 failed candidates/refinement02 and preserved scope-complete four-slice endpoint; all later arcs remain unactivated.

-1.3 —2026-10-24: recorded independently accepted S02 model increment and retained complete Arc01 relational/query/composition endpoint; project/later-arc obligations remain open.

-1.4 —2026-10-24: recorded independently accepted S03 relational persistence, all12schema areas and actual public/legacy/negative/current-gate evidence; S04 and all-four composition remain before Arc01endpoint. All14project rows/later arcs remain open.

## Arc01 foundation bubble-up —2026-10-24

All4children deliver the roadmap's fixture foundation, all33childrowsdone, original7criteria/12schema areas retained. Separate independent composition at efef9c4passed265actualRust/CLIcommands/2227harness checks; final canonical close-packet gate also accepted at7ddd0db. Arc01CLOSED; project stillOPEN/all14rowsOPEN. No new arc/scope/topology/linguistic inference or corpus substitution is required. Complete remaining-owner mapping is arc01-dictionary-foundation/closing-report.md; later A02rawimport,A03alloptionalcollections,A04unified/ATTACH/fanout comparison,A05productionownededitor/export,A06fullintegration and Duncan's projectacceptance remain.

Feature source efef9c4includes inspected initial logo ancestry e0630d8but excludes later concurrent main ed64166README branding/b0777fcd25943d7887c40981e7f4896ec65b04ffLinux/macOSCI. Current main hasCI; no new push/workflow trigger/integration is authorized here. Main untrackedFinder files remain protected. Future integration must reconcile concurrent README/CI separately; no main/Bauske/publication action performed. Research41copybytes/hash/size remain verified as preservation evidence.

-1.5 —2026-10-24: Arc01all4children and independent technical composition delivered the full foundation; current body/registry-DDL decisions/remaining project owners/main drift reconciled from actual accepted evidence. Final packet gate pending; no later arc activated or project row closed.

-1.6 —2026-10-24: separate final canonical packet gate accepted at7ddd0db; Arc01formallyclosed at sourceefef9c4, all6arcrows/33childrowsdone and all required retrospectives/bubble-up complete. Project14rowsremainOPEN, noA02or main integration/publication activated; return at authorized endpoint.

- 1.7 — 2026-10-24: Duncan renewed Autonomous/Expedited execution and existing explicit model selection for Arc02. Recorded directive02, fresh current-main source baseline/five-crate CLI facts, corpus acquisition authority and held publication, resource guardrails and stop-before-A03 boundary. Previous Arc01 authority/preflight/progress are historical receipts; all14projectrows remain OPEN.
