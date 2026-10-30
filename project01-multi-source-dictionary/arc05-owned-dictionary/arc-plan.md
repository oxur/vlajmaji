# Arc05 — Owned dictionary

| Field | Value |
| --- | --- |
| Status | S01–S03 CLOSED locally;23child and A05-01…05 DONE,3arc/all14project OPEN; S04 INITIALintakeaccepted; conditionalCC12/index/target aftercheckpoint+dispatch, its8rowsOPEN through finalfresh/canonical/root gate |
| Authority | [directive04](../cdc-directive04.md), Duncan “merged! Let's continue :-D”; THROUGH ARC05 ONLY |
| Roles | Retained CDC Sol6.1/high; retained CC Luna/high; fresh independent Sol6.1/high assembled review and SAME-context canonical gate at close |
| Source | `codex/project01-owned-dictionary`, accepted S03 `2b7b6dda55dc00d62bfa4fcdc5d5a09742fcf528`, tree `83d81705cc1de3aa69f1d6d39e4c524bb1b6d9b3`; accepted S01/S02/base history retained |
| Physical checkout | Reused `.worktrees/project01-arc04/vlajmaji`; old Arc04 branch9026 retained; no new worktree/cache |
| Ownership | S04 acceptedintake/freshpins; AFTERcommittedcheckpoint+explicitdispatch CCexclusive12/index/ONEtarget INITIAL, root/CDC readonly; governing planning CDC |
| Version / date | 0.4 / 2026-10-29 |

## Useful result and boundaries

A caller can maintain an explicitly named dictionary of their own, preserve every prior assertion and editorial decision, export/reimport it without losing history, and load its optional snapshot before SQLite. The Rust API and CLI deliver each increment together. The source is caller-declared `OwnedDraft`, with a named editor and explicit content notices; it claims no community authority.

Arc04 is CLOSED and actually merged as a55e1cc7 (tree-equal accepted9026). Main CI40806671887 SUCCESS is baseline evidence only. Its accepted latency exception, failed campaigns, diagnostic, platform repairs and fixture limitations remain historical evidence; this arc does not optimise BL-0001 or run another Full/performance campaign. All fourteen project rows remain OPEN. Arc06 owns the broader all-pack/live managed workbench and final project/Operator acceptance.

## Architecture and choices

1. **Authority:** a new versioned SQLite owned journal, distinct from rebuildable read-only `DictionaryStore` and upstream collection catalogues. Writes use one transaction and persisted expected-head CAS under `BEGIN IMMEDIATE`. Two already-open writers cannot silently overwrite one another. No automatic last-writer-wins or undo-by-deleting-history.
2. **Model:** existing normalized Dictionary/registry version1 remains the complete typed model. S01 accepts whole validated genesis/candidate Dictionaries and appends new records; old typed bodies/order remain immutable. The current owned head and monotonically incremented catalogue revision are explicit control fields. An edit creates one same-source revision; active IDs describe the complete current set, and explicit tombstones retain removed assertion identities. Existing selected release flags stay immutable; existing owned-head selection already intersects release membership with current active IDs.
3. **Curation/context:** reviewed local senses use existing owned/editorial types. Cross-source curation persists in a distinct version1 owned envelope outside the base Dictionary, with typed `CatalogTarget`, direction, editor/revision/rationale and exact context provenance. No foreign placeholder ReleaseRecords and no copying whole upstream corpora per edit. Absent context is explicitly unavailable/unresolved. With caller-supplied exact validated context, compose the actual bodies FIRST, then materialize reviewed links/groupings with closure to actual releases/assertions. Reject unequal same-ID bodies; never infer identity from spelling or silently retarget. S02 now delivers the precise strict version1 adapter and explicit schema2 migration at854865a7, independently accepted locally. Sealed history/reimport remains S03; this does not claim that later capability.
4. **Export/reimport:** separate version1 owned snapshot/seal, preserving the complete ordered journal history and curation envelope. It is not a new `PackId`. Export validation and strict load precede derived indexing. Exact replay is idempotent only under S03's explicit import policy; conflicting same-ID bodies/head histories reject. S01 edit CAS itself has no idempotent stale-replay exception.
5. **Packaging:** optional `vlajmaji-data-owned` and umbrella `data-owned` feature, default empty and `publish=false`, using explicit local snapshot materialization. Existing four upstream `PackId` values/All-four installation stay unchanged. No fifth mandatory upstream input, automatic publication, network load or redistribution claim.
6. **Query boundary:** S01 provides actual init/edit/head/history/show. No assumed in-memory search constructor exists. S03 demonstrates snapshot→Dictionary→explicit fresh `DictionaryStore::create`→existing dictionary query/detail. Arc06 later integrates a live mixed managed view.

## Four usable increments

| Slice | Public Rust + CLI capability | Dependencies and owning acceptance |
| --- | --- | --- |
| [S01 — Owned journal](slice01-owned-journal/slice-plan.md) | Explicit ROOT init/edit/head/history/show, complete typed metadata, transactional CAS and immutable revision reads; shipped minimal seed/edit example and CLI recipe | Existing model/validators and rusqlite; no external context or corpus required. Initial full draft open set below |
| S02 — Reviewed curation/context adapter | Local reviewed senses plus persisted pinned external links/groupings; show unavailable context; materialize against exact explicit Tiny context and query existing reference routes | Accepted S01; S02 locally CLOSED at854865a7 with exact adapter/migration/status/identity controls. Curation reimport remains S03 and A05-03 stays partialOPEN |
| S03 — Sealed history and explicit querying | Deterministic export/check/load/reimport; preserve all historical bodies/heads/curation; exact replay vs conflict; explicit derived index and full/compact/readable Rust+CLI query | Accepted S01–02. Distinct owned envelope/loader, no placeholder composition or default-store fallback |
| S04 — Optional owned package/offline consumer | Default-empty optional owned crate/feature; strict embedded snapshot load before SQLite; outside consumer and actual complete owned workflow | Accepted S03. Disjoint Synthetic materialization; no ignored Full payload overwrite. Existing four upstream/features remain compatible |

These are delivery increments, not investigation-only slices. Re-size only by an explicit preserved scope/owner amendment; do not silently defer curation or metadata preservation to Arc06. Each slice has five total implementation/refinement passes, including its initial assignment; no automatic sixth.

## Validation and evidence

Each child includes actual public Rust and actual CLI positive/negative sequences, exact persistent-state checks, source review, documentation, format→lint→focused tests→make check, affected no-default/feature/package checks and independent CDC disposition. Use existing synthetic fixtures and native-path subprocess infrastructure. Never put large JSON in argv; inputs are files/stdin. No new benchmark or per-command artifact framework.

Use one retained target with `/tmp/vlajmaji-p01-a05-cargo` (SHA256 `426d1e49863551fe37252cecb8d36f8fb58410670fc550c6deedc89ddb92f53d`), explicit +1.98.0/locked/offline. Source/workspace/Bauske sibling and accepted branches remain protected. Child HOME/XDG/TMPDIR are owned and disjoint; pin existing CARGO_HOME/RUSTUP_HOME/PATH before replacing HOME. Feature tests must use known Synthetic payloads or a reviewed disjoint source/materialization recipe; do not probe, overwrite or re-materialize ignored Full files in the reused checkout. Actual user default is never stat/read/changed.

At arc close a fresh independent context actually exercises init→edit→stale rejection→local/external curation→export→reimport→optional load→explicit derived query/history, including absent/conflicting context and old assertions. It reviews the four children and original compatibility boundaries, then SAME reviewer gates the actual canonical packet. CDC supplies child/arc retrospectives and project bubble-up; root checks consistency before mechanical closure. Produced-path inventory is proportionate, not a reconstruction of Arc04 megabytes. No success reports or row closures are prewritten.

## Current readiness

S01–S03 are locally CLOSED at2b7b6dda/tree83d81705;23child and five arc rows DONE, three arc/all14project OPEN. S03 curation reimport discharges original A05-03 alongside S02; complete owned seal/history/import/query predicates discharge A05-04/05. Its [verification](slice03-sealed-owned-history/cdc-verification.md) and [retrospective](slice03-sealed-owned-history/cdc-retrospective.md) retain actual58+6 direct verification children, original product/method failures, source/gate/platform scopes and exact evidence copies. S04 exact118c86e root readiness→e561 status-only issue→completeactualintake/addendum accepted and fresh2b7pinsPASS. AFTERcheckpointcommit+explicitdispatch CCowns12/index/ONEtarget INITIAL; operational proof alone cannot close its eight rows, which retain original fresh assembled/SAME canonical/root gate. Source/index/ONEtarget ROOT QUIET. No Arc06/external advance.

### Historical draft readiness — version0.1

This is the concrete planning proposal, not an issued source assignment. S01's exact API/schema/append rules and finite regression oracles are in its full slice plan and DRAFT prompt. Root exact readiness remains required before status-only issue, bounded retained CC intake, fresh baseline/pins and explicit source/index/target transfer. S02's detailed adapter is deliberately pending its later readiness; its capability and evidence obligations are already owned here. No unresolved human product choice blocks S01. No push/PR/publication/Arc06 authority is inferred.

## Version history

- 0.1 — 2026-10-28: directive04 Arc05 activation proposal at merged a55; physical checkout/one cache reused; four usable increments and separate authoritative/context/export/package boundaries fixed. All rows OPEN; S01 DRAFT only.

### S01 issue checkpoint — 2026-10-28

Root final readiness PASS at proposal d75532d clears status-only initial issue and retained CC complete intake. Source/index/ONE target remain root QUIET until CDC accepts coverage/readback and explicitly transfers the eleven S01 paths; all8arc/7child/14project rows OPEN. Current S01 status supersedes the dated DRAFT readiness prose above; architecture and later-slice obligations unchanged.

- 0.2 —2026-10-28: actual S01 local close/source1b899ebb, independent proof/retrospective and partial parent rows01/02; S02 current draft reconciled, still UNISSUED;6arc/all14project OPEN.

- 0.3 —2026-10-29: accepted S02 source854865a7, independent mixed-own migration/materialization/history proof and local eight-row close; actual body/current ownership updated. No arc structural re-sequencing; A05-03 reimport remains S03. Unchanged fixture fragility is BL-0002, not shipped repair; S03/S04/fresh assembled/canonical gates remain.


### S03 initial intake/transfer —2026-10-29 CDT

Root exact bd3b685 readiness→status-only issue f51826d→complete source-cited intake/query addendum accepted→fresh854/treeef81/branch/lock/guard/protectedrefs verified. After current checkpoint commit+explicit dispatch, retained Luna/high CC owns exact15/index/ONEtarget for S03 INITIAL/pass1; root/CDC source read-only. S03 eight rows remain OPEN; S04 DRAFT/unissued, parent six rows/project14 OPEN. Current detailed ownership is S03 artifacts/execution.md; earlier Current readiness DRAFT sentence is historical pre-issue, not acceptance. No external/Arc06.

- 0.4 —2026-10-29: actual S03 local close2b7/tree83d and original curation reimport/seal/history/query proof;23child/fivearcDONE, three arc/all14project OPEN. S04 remains UNISSUED pending final current reconciliation/readiness; its8rows OPEN through final assembled/canonical/root gate.
