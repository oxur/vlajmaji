# Project01 — Multi-source dictionary

| Field | Value |
| --- | --- |
| project | project01-multi-source-dictionary |
| status | project OPEN; Arc01 and Arc02 CLOSED; Arc03 CLOSED; all14projectrowsOPEN; A04 active; S01 CLOSED/Unified selected; S02 design-before-issue; A05/A06 not activated |
| depends-on | Vlajmaji PoC at `4ee3babefa13fbde41c94ab9bfcf61bb029834fe`; bounded accepted research described below |
| blocks | Reusable source-grouped vocabulary workbench and source-backed wordcraft experiments |
| related | Bauske; Relojban-poetics Project02; Bō Gē's “the last snow”; Open Tone Harmony |
| source-home | `/Users/oubiwann/lab/oxur/vlajmaji` |
| planning-home | orphan `planning` branch, `.worktrees/planning/project01-multi-source-dictionary/` |
| version / date | 1.21 / 2026-10-25, America/Chicago |

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
[ledger.md](ledger.md). Arc01 has four independently accepted children and accepted separate technical/final packet composition; it is CLOSED. Arc02 is CLOSED after both accepted children and separate fresh technical/finalcanonical review. Arc03 all4children and fresh assembled technical composition are accepted at7b1c565; actual canonical packet is independently accepted at proposal a5226ee. A04 is now authorized/active under the current checkpoint below; A05/A06 remain unactivated.

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
| [02 — Reproducible full jbotci import](arc02-reproducible-jbotci-import/arc-plan.md) (CLOSED at4e37016) | Explicit acquire/replay/check/update path imports the pinned raw export with every alternative, original bytes and reconciled diagnostics; importer API plus Bash/Make entrypoints and normalized release. | Accepted A01 schema/identity/query contracts. | Two useful slices: lossless Rust/CLI normalization, then pinned pipeline/full persistence composition. |
| [03 — Independent corpus packages](arc03-independent-corpus-packages/arc-plan.md) (CLOSED at7b1c565; final canonical accepted) | Dedicated opt-in packages/features for jbotci/Lensisku, LLG components, La-Lojban and Sutysisku, usable individually and together without build/load network access. | A01 contracts and A02 full import; pinned source/component inputs. | Plan per collection; split packaging work rather than omit a collection. |
| [04 — Composed SQLite lookup](arc04-composed-sqlite-lookup/arc-plan.md) (active; S01 CLOSED/Unified selected) | Fair unified/ATTACH/fanout decision and selected integration **plus** local selected/all4 install, catalogue/list/inspect, configured/default one-command exact/ref, lifecycle/recovery and readable/compact/full-detail output. | Accepted A01–03 contracts and exact four Full packs; exact inquiry/effort/numeric approval and concrete S01 provider design recorded. | Five useful increments; S01 accepted at source4b4158c, selecting Unified for the frozen workload. S02 provisioning/catalogue preparation next; no install/default/lifecycle/output acceptance yet. |
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
| Full raw jbotci shape, observed counts, unknown/wiki-row eligibility | Resolved A02at4e37016:33526raw/278254fields/33516eligible/31248spellings; every row retained,9blank+1wiki explicitly excluded. Supplier counts reconciled separately; independent finalcanonicalpacket accepted5456b54. |
| Corpus component boundaries, common pack API and feature host | Resolved A03: common seal1/strict load/consuming composition,4unpublished optional embedded crates and umbrella features, exact component/provenance/held terms. All4children and fresh technical assembly accepted7b1c565; final packet accepted. |
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

## Historical Arc02 authority and contributor workflow — 2026-10-24 (superseded endpoint)

Duncan instructed: “okay ... I think we're ready to dive into arc02! Same Autonomous Mode, same model config/setup :-)”. This renews OpenAI GPT 6.1 Sol High CDC / GPT 6 Luna High CC for Arc02, including slice design/implementation/repairs/independent acceptance, composition, formal close/bubble-up and all retrospectives. Stop before A03. [cdc-directive02.md](cdc-directive02.md) is the preserved current assignment; directive01 and the preceding authority section are historical, not current corpus restrictions. Requested/dispatched settings are distinct from unavailable backend attestation.

Assigned CDC `/root/vlajmaji_p01_a02_cdc` acknowledged; root remains read-only. CDC owns planning/source setup; CC owns isolated source mutations only during issued intervals. Fresh source branch/worktree was created from verified clean main `d1e1c928f7999f2220b046cc5a5817f0529312ca`; no main mutation, push, PR, publication, integration, Bauske mutation or later-arc work is authorized. Current source has five `crates/*` packages, `crates/cli`, umbrella crate, Rust1.98 and Bauske0.3.0 pin `c5801ae17caaf86d2651b5ce34570199e873ad0f`; source AGENTS and CI were fully read. Arc01 original acceptance remains efef9c4; later merged corrections/release changes require their own current evidence.

Arc02 expressly authorizes necessary public acquisition/read of pinned jbotci raw `dictionary-en.json` plus metadata at `3d9fd41d9ed32de26311ee27ec86b6b165985aa1`; no live authenticated Lensisku, exhaustive discovery or silent repin. Full rows/fields/alternatives/native accounting, normalized output independent of SQLite, useful Rust/CLI/Bash/Make, offline replay/check/update and complete persist/reopen/grouped querying remain required. Data redistribution held; no massive raw payload in public code. Plan slices after actual source/data reconnaissance. No new dependency without demonstrated need.

Preflight: planning clean5531f604/ahead54 at takeover; main clean d1e1c928, old feature clean a7c250f. Data volume2.6GiB free. Prior designated A01target10GiB and source-local68MiB are protected pending justified cache ownership/handover; no blind cleanup. Every Cargo/Make command must set a task-owned target through fixed launcher, with one writer; serialize imports/snapshots/scans. Independent callers require unique package and binary identities/offline locks. Readback follows complete required bodies and whole-population data queries; preserve failed attempts. Fresh separate SolHigh context gates assembled technical/final packet composition; no CRC inferred. Local scoped commits authorized.

## Historical Arc02 research status and endpoint (superseded by current Arc03 checkpoint)

[Research guide and verification status](artifacts/research/README.md) distinguishes
preserved authored reports, source captures, authoritative historical gates and
new design choices. [copy-manifest.tsv](artifacts/research/copy-manifest.tsv) records
exact original paths, revisions, SHA-256 and byte sizes for every copied file.
Old source pins and measurements remain dated evidence, not current service or
performance claims. No full corpus, database or Cargo build cache was copied.

Current endpoint: Arc02 is CLOSED at4e37016 after independent child/technical/finalcanonical acceptance5456b54. Actual full import→standalonecanonical→ownfreshstore/drop/strictreopen→sourcegroupedRust+actualCLI/offlinecontrols composes; all8arcrows/15childrowsDONE. Oldwriter600s remains incomplete history. Return at this authorized endpoint; A03–A06remainunactivated and require a new explicit arc instruction. All14projectrowsremainOPEN; no push/PR/mainintegration/publication or project/Operator acceptance.

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

## Historical CDC preflight — 2026-10-24

Assigned CDC `/root/vlajmaji_p01_a01_cdc` acknowledged `cdc-directive01.md`. Requested OpenAI `gpt-6.1-sol` / high; tool-dispatched configuration is distinct from unknown backend attestation. Planning owner is CDC; source owner passes to the assigned CC only during issued intervals. Root remains read-only. Approved substrate commit: `35e368f`.

Original source baseline remains `4ee3babefa13fbde41c94ab9bfcf61bb029834fe`. During readiness Duncan's separate logo work advanced main through `08c6bdb` to `e0630d8279b9ea48ce5892e201a5c50c52c3a469`. CDC inspected the exact two commits: only `assets/images/v1-250.png` and `assets/images/v1-1254.png` changed. On root's explicit instruction the clean isolated feature branch was fast-forwarded to `e0630d8`; no main or asset mutation occurred. Main's untracked Emacs README buffers are protected. Source APIs remain the inspected `4ee3bab` code.

Task output owner: `CARGO_TARGET_DIR=/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc01/target`; serialize checks. Source integration destination is the local feature branch, independently accepted at slice and arc scale; main integration remains a later separately authorized action. Next current assignment is recorded at slice scale. No A02 activation or external action is authorized.

## Historical Arc01 progress — 2026-10-24

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

## Arc02 S01-to-S02 readiness —2026-10-24 (historical; superseded by current S02 close below)

Canonical arc02-reproducible-jbotci-import S01 is independently accepted at12fdd67 with all7rows closed after two executed refinements. Complete41API/36CLI/915570source checks, full shipped example+negative counterparts/fullCLI and current mandatory gates are in the child verifier/close/retro and hashed accepted artifacts. A02-01DONE/other7arcrowsOPEN; all14projectrowsOPEN. Complete raw33526rows/278254fields and every alternative remain, with33516eligible/31248spellings and10explicitexclusions. S02 must still deliver actual pinned acquisition/replay/check/update, fresh fullstore/reopen/groupedqueries and outside offline composition. CDC owns finite source-grounded readiness and target; CC idle. Main/Bauske clean and protected. Operator cache cleanup was assessed before rebuild: retained target14GiB/debug intact/release absent,449265967104B free at current handover. No cache deletion by contributors, push/publication, A03 or project acceptance.

- 1.8 —2026-10-24: source/data reconnaissance opened canonical Arc02 with two usable slices, eight open arc rows and seven open S01 rows. Actual export shape and full counts ground S01 importer/CLI contract; pinned pipeline/full persistence belongs S02. Source/worktree/cache ownership and abandoned pre-repair process recovery are recorded in owning readiness artifact. No implementation acceptance or project-row closure.

- 1.9 —2026-10-24: accepted full S01 normalization/API/CLI at12fdd67 with complete independent source-native population and current gates; child7rows closed, A02-01done,remaining7arc/14projectrowsopen. S02 actual acquisition/persisted/offline composition remains next; current resource/source ownership reconciled.

- 1.10 —2026-10-24: actual bounded full-input readiness exposed alphabetical deferred-FK load scans; old create stopped at600s with native/EXPLAIN/schema evidence and untouched incomplete artifacts. Opened S02useful pinnedworkflow/fullpersistedlookup plus narrow schema-preserving load/query corrections,8open childrows. Retained old binary/common-new-store comparison replaces prohibitively slow old writer prerequisite, not actual full persistence/query acceptance. All14projectrows remainopen; no A03.

## Arc02 historical S02-close checkpoint,2026-10-24

Both children accepted: S01at12fdd67/all7rows, S02at4e37016/all8rows. Full raw33526rows/278254fields,33516eligibleoccurrences/31248spellings,73226extensions compose with explicit pinned acquisition/offline replay/check/update and fresh complete relational/query/publicoutside behavior; current gates88/90Rust+19tools/onehistoricalresearchignore pass. A02-01/02DONE/6arcOPEN awaiting fresh independent technical/finalcanonical composition;14projectrowsOPEN. Freshengine strictopen25.059srelease (outside26.383s) and one-shotdebugCLI91.533s are practical costs owned P01-08/A04 operational comparison and P01-11/12/A06 integrated API/rebuild work. Long-lived engine amortization/response-ownedmetadata/occurrence scans remain explicit; noconstanttime/endtoendCLIratio/heapbyteclaim/newtopologydecision. Source/main/Bauske remain protected/frozen; noA03/push/publication.

-1.11 —2026-10-24: S02 supplied full useful pipeline/persistedlookup and all8independent rows, current nextstep is fresh separate assembled arc review. Updated currentstatus/latency carryovers while preserving historical readiness/attempts; header previouslystill1.8 despite historythrough1.10, nowreconciled. No structural roadmap/scope reduction or project-row closure.

## Arc02 historical technical/proposed-packet checkpoint,2026-10-24

Both roadmapA02capability slices delivered and fresh independent full-path technical composition PASS4e37016; actual canonical proposal is arc02-reproducible-jbotci-import/closing-report.md with CDCverification/retrospective. No promised A02runtime obligation deferred or dropped; detailed all14remaining-owner mapping is there. A03allfouroptionalcollections/features,A04unified/ATTACH/fanout,A05ownedediting/export,A06integratedworkbench and Duncanprojectgate remain. Cold strictopen~25srelease/actualreleaseCLI25.619s plus retaineddebug91.533s are practical carryovers to P01-08/A04 and P01-11/12/A06, no topology or newoptimization choice. Same-context finalcanonicalgate is stillpending; all14projectrowsOPEN/noA03.

-1.12 —2026-10-24: A02fresh full independent technical composition succeeded after bothchildcloses. Updated current shape/count decision and roadmap/currentnext with actual result; assembled canonical bubble-up/retro pending independent finalpacket. Original scope/decomposition retained; no projectrow closed or laterarc activated.

## Arc02 CLOSED and current project carryover —2026-10-24

Fresh technical and actual finalcanonicalpacket accepted at4e37016/5456b54; bothplannedchildren/all15rows and all8arcrowsDONE. RoadmapA02fullrawnormalization/API/CLI/pinnedworkflow/standalonerelease/fullpersistedgroupedlookup delivered without a deferredruntimeobligation. All14projectrowsstayOPEN. A03allfourrequiredoptionalcollections/features,A04fairunified/ATTACH/fanout,A05ownedediting/export,A06integratedprojectreplay and Duncanprojectgate retain their exact owners. Strictopen~25srelease/releaseCLI25.619s/debugCLI91.533s practical costs remain P01-08/A04 and P01-11/12/A06; source response-ownedmetadata/occurrence scans explicit, no topologyselection/constanttime/endtoendspeedclaim. Localfeature source4e37016 unchanged; protectedmain/Bauske clean. StopbeforeA03/noexternalpublication.

-1.13 —2026-10-24: Arc02actualfinalcanonicalpacket independently accepted5456b54, authorizing mechanicalstatus/evidence/framing close only. All8arcrowsclosed, all14projectrowsopen; currentroadmap/next/shape decision and fullremainingowners reconciled. Earlierpreparedcheckpoints remainexplicitlyhistorical, no source/runtime/newscope action.

## Current Arc03 authority and package design —2026-10-25

Duncan explicitly authorized all Arc03 in Autonomous Mode; unchanged requested Sol6.1/high CDC and Luna/high CC. [cdc-directive03.md](cdc-directive03.md) is preserved authority; [arc03-independent-corpus-packages/arc-plan.md](arc03-independent-corpus-packages/arc-plan.md) owns the current eight-row open set and four useful corpus increments. Source is isolated feature/project01-independent-corpus-packages from clean main52fd3c4, all five original crates retained; Bauske main/default installed store are protected. No push/PR/publication or A04 activation.

Common seal/load and consuming pure composition plus dedicated optional internal data crates are resolved in the arc/S01 plan: actual full normalized bytes embedded only after explicit offline local materialization, default core has no payload, Synthetic CI preparation retains clean all-features coverage. Held real data stays excluded from Git/package/upload. Internal path-dependency delivery is not public crates.io readiness; unpublished optional dependencies keep umbrella public release gated by later distribution work. All14projectrows remain OPEN. Historical accepted Arc02 and bounded follow-up endpoints above remain historical; PR3 was subsequently merged at current52fd3c4, not retroactive reacceptance of those snapshots.

## Historical Arc03 technical/proposed bubble-up checkpoint —2026-10-25

All4roadmap collections are actual complete internal offline Rust dependencies: jbotci/Lensisku, The Logical Language Group gismu/cmavo/rafsi family, La-Lojban distribution, Sutysisku projection. Child24rows accepted at04289ae/0d70c9f/62b27db/7b1c565; fresh author-independent actual assembly/store/nativeSQL/publicRust+releaseCLI technical review accepted current7b1c565. Actual canonical packet remains pending same-context final verdict; all8arcrows stayOPEN until it. No runtime delivery deferred from A03.

Pure normalized composition works before SQLite. Four locally materialized Full packages retain78361received assertions/76722entries,76803lexical/1558excluded,31375eligible spellings,566367native fields including23705Sutkeys,3sources/2distributors/4collections/6components/6releases/8captures. Shared Lensisku authority and La distribution copies stay distinct from independent attestations. Content notices/unknown lineage remain explicit; all releases Held. Internal include_bytes consumption is delivered, public crates.io-ready redistribution is not. Core/default builds need no payload; selected missing data fails compilation; explicit offline Synthetic preparation supports clean allfeatures without pretending to be Full.

Fresh reviewer created ONE independent full schema1 store from435519540B canonical/SHA d9d73cbb49cc5a9ddb0ec16cb6dbb4bb7260ad49f4f8f54c6e6d21f0779b7b19, dropped/reopened, passed all6explicit checks and375794raw-derived native/typedSQL controls. All47table inventories and complete eligible tuples match. Fullfarvi5occurrences/3groups with aliases/pages/references and actual releaseCLI equality compose. Evidence: arc03-independent-corpus-packages/artifacts/composition-review.md. Protected provisionalLensisku-only installeddefault remains unchanged; assembled review store is a separate owned witness, not a default replacement/topology decision.

Current operational framing supersedes only the present-tense historical strictopen carryover: mergedPR3 already moved comprehensive validation to explicit verify, ordinary open is minimum-header, info uses SQL, query decodes typed model and builds local safety/setup indexes. Historical A02~25s strict-open timings remain dated evidence. Current four-pack farvi wire is7160366B for5hits, canonical JSON excludingLF7160365B; releaseCLI singlequery3.027s is one local process observation, not a benchmark/cold-disk/general latency claim. Combined-stream capture/source-renderer inference is stated by reviewer. Payload/setup costs remain P01-08/A04 and P01-11/12/A06; no new threshold/optimization/topology is chosen.

All14projectrows remainOPEN. A04 owns fair unified/ATTACH/fanout comparison and selected integration; A05 owns production edits/history/conflicts/sealedownedpackage; A06 owns remaining all-pack+owned/removal/reload/query-family/end-to-end work and Duncan final project acceptance. Each release's public redistribution remains a separate Operator/source-rights gate. StopbeforeA04/no push/PR/publication/main integration.

-1.15 —2026-10-25: A03all4children/24rows and fresh actual full assembled technical composition deliver all4internal dependencies. Current package/identity/feature boundaries and project partial contributions reconciled; current startup language updated to mergedPR3 while preserving historical timings. Canonical final gate pending; all14projectrowsOPEN, unchanged A04–A06/Operator owners.

## Historical endpoint — Arc03 CLOSED,2026-10-25

Fresh technical and actual nine-body final canonical review independently ACCEPTED source7b1c565/proposala5226ee; all4children/24rows and all8arcrowsDONE. Final verdict arc03-independent-corpus-packages/artifacts/composition-review/final-canonical-review.md. Current packet retains all4internal dependencies, complete native/provenance/terms/query/feature proofs and all14projectrowsOPEN. Historical proposed/pending checkpoints above remain dated history. No A04 activation, publicdata publication, push/PR/main/default-store change. Return at authorized Arc03 endpoint; next substantive arc requires Duncan's explicit instruction.

-1.16 —2026-10-25: actual finalcanonical review independently accepted; mechanical8row/currentstatus/evidence/framing close only. No new capability/scope/terms/topology change or runtime rerun. RemainingA04/A05/A06/Operator/rights owners unchanged, project14OPEN.

## Historical Arc04 planning-only follow-up —2026-10-25

Duncan agreed to expand the formerly topology-focused A04 roadmap into explicit user workflow deliveries and asked for the planning work now. Root bounded the request to records+A04plan, stopping before implementation. Arc04 is PLANNED, not active/issued; A05/A06 remain unactivated and all14projectrowsOPEN. Earlier stop-beforeA04 statements are accepted historical execution endpoints; this instruction authorizes planning only. Source7b/main52fd/Bauskec580/default/Arc01–03 records remain immutable. No CC/source/target/store/network/benchmark/installation or new branch/worktree.

The mandatory expansion (was: three-way topology comparison+selected integration without explicit user installation/output deliverables) is [W1–W5](arc04-composed-sqlite-lookup/artifacts/user-workflows.md): local selected/all4 installation; register/list/inspect; configured/default one-command combined exact/ref with source filtering; lifecycle/recovery; readable plus compact/full/detail output. API/CLI/help/recipes and independent gates accompany each useful increment. These outcomes belong to A04 and cannot be deferred wholesale to A06. Existing Q01–Q06 compatibility remains required; broader Q01–Q06+owned final integration stays A06 and owned writable editor/package stays A05.

Names, flags and catalogue/config/migration/projection schemas remain design decisions before owning prompts. Current full JSON and explicitSTORE compatibility are retained. Actual userdefault must not be switched by implementation/review; isolated scratch/XDG fixtures prove install/migration. Public redistribution Held does not block authorized internal local installation; no automatic network. Comparison [protocol](arc04-composed-sqlite-lookup/artifacts/topology-comparison-protocol.md) gives a concrete proposed7hour/5store/12GiB/600s-stage envelope and correctness-first criteria; neither investigation nor that budget is approved. Numeric latency/bytes/setup budgets require explicit resolution before affected trials/code, without invented subsecond claims. Root consistency review precedes scoped planning commit; no execution prompt exists.

-1.17 —2026-10-25: after Arc03 closure exposed the remaining provisioning/default/output gap and Duncan explicitly agreed, added A04's five user workflows to roadmap/plan/ledger; retained three-candidate comparison and all A05/A06/Operator/rights owners. Planning-only authorization, no investigation/implementation or new acceptance claims.

## Historical Arc04 renewal / S01 HOLD —2026-10-25

Duncan's “let’s dive in!” renews Autonomous execution THROUGHARC04ONLY with unchanged Sol6.1/highCDC/Luna/highCC. Existing plan02c8063 remains preserved. General source setup/design/readiness is authorized; explicit proposed inquiry/effort/operationalbudget question is pending asynchronously via root, not approved by silence. No CCissue/trial/newFullstore before returned answer and concrete source recipe. S01DRAFT/HOLD open set: arc04-composed-sqlite-lookup/slice01-topology-comparison/.

Actual cleanisolated source fromaccepted7b: `.worktrees/project01-arc04/vlajmaji`, branch `codex/project01-composed-sqlite-lookup`, read-only Bauske sibling; unusedA04guard points to retainedtarget, no builds. Provider/API/trust/generation and exactread-onlyATTACH recipe holds are explicit in S01plan; source-guided coupling is identified, not falsely declared implementation-ready. All14projectrowsOPEN/A05–A06unactivated, acceptedArc01–03/source7b/main/default immutable, no push/publication.

-1.18 —2026-10-25: recorded explicitArc04executionrenewal after planningcommit, pending inquiry/numericanswer, isolatedcodexsetup and honestunissuedS01HOLD open set. No runtime observation/decision or scope reduction; already-spent preparation counts toward proposedfreezeallowance rather than restarting it.

## Historical Arc04 S01 approval and issue — 2026-10-25

Duncan explicitly approved the exact proposed inquiry, effort, workload and numeric sheet: “this looks good! Approved, and engage!” See protocol 1.2 and the issued S01 plan/ledger/prompt. Existing Sol6.1/high CDC and Luna/high CC remain; Autonomous endpoint is Arc04 only. All fourteen project rows remain OPEN. S01 compares independently loaded U/A/F state with common full semantics and actual setup/hydration costs; no unified metadata sidecar for A/F. Selected topology remains unresolved until actual correctness/budget evidence and independent acceptance. Compact/readable budgets apply to S05, not a premature S01 implementation.

Conservative accounting charges 30 minutes preparation and at most 60 further freeze minutes from retained 22:06:10 UTC; total seven working hours, five new Full stores, 12 GiB new scratch and three balanced timing rounds are binding. The isolated codex source starts at protected 7b; root is read-only, CC source/index/target only issued intervals. No actual user default/main/Bauske/accepted record mutation, push/publication or A05 authority.

- 1.19 — 2026-10-25: explicit inquiry approval and resolved source-guided S01 recipe; initial assignment issued with all child/arc/project rows still OPEN. Earlier HOLD checkpoints preserved. No runtime result claimed.

## Historical S01 bounded repair / specific HOLD — 2026-10-25

See Arc04 protocol1.3: the required pre-create comparator gate was violated before two retained store creates. No comparison/selection/acceptance. Remaining Full create/query/timing is held for an explicit Operator exception after a concrete tested reuse-two/finish-three proposal. General Arc04 authority persists for bounded readiness repair; all project rows OPEN, A05/A06 unactivated.

- 1.20 — 2026-10-25: actual prerequisite deviation and specific HOLD recorded without rewriting history, accepting a topology, resetting limits or changing project contracts.

Current re-entry recommendation update: protocol1.4 proposes preserving early two as historical and a clean fresh five-store run, requiring explicit count five→seven amendment. Other caps unchanged; no further Full operation or bespoke resume until response. This supersedes the interim reuse proposal, without changing its recorded chronology.

## Version 1.21 — S01 count amendment approved, remaining readiness HOLD

Duncan explicitly approved seven TOTAL candidate stores: retain the two historical/unaccepted stores and create five clean comparison stores only after the remaining prerequisites. Protocol 1.5 records the exact answer and timestamp semantics. Same seven working hours including preparation, 12 GiB including the retained two, 600-second bounds and three balanced rounds; no reset. COUNT gate cleared only. Full operations remain HOLD until CDC reviews the complete comparator/prefix/source/gates, batch/reference/page and Value-serialization harness, frozen category config, binaries and expanded argv. No topology decision or acceptance is implied; all rows remain OPEN.

## Version 1.22 — accepted Arc04 S01, 2026-10-26 UTC

S01 is CLOSED at source4b4158c; Unified selected through the complete finite three-candidate comparison and independent acceptance. See Arc04/S01 decision, verifier and retrospective. A04-01 DONE; nine arc rows and all fourteen project rows OPEN. This is a partial P01-08 contribution, not completed production integration. S02 retains local selected/all installation, catalogue/ownership and native-path design before issue; S03–S05 retain configured/default combined lookup, lifecycle and representation delivery. All prior authority/history checkpoints remain preserved.
