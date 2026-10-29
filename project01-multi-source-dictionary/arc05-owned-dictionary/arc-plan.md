# Arc05 — Owned dictionary

| Field | Value |
| --- | --- |
| Status | IMPLEMENTATION ACTIVE: S01 INITIAL/pass1, eight arc rows OPEN; no candidate acceptance yet |
| Authority | [directive04](../cdc-directive04.md), Duncan “merged! Let's continue :-D”; THROUGH ARC05 ONLY |
| Roles | Retained CDC Sol6.1/high; retained CC Luna/high; fresh independent Sol6.1/high assembled review and SAME-context canonical gate at close |
| Source | `codex/project01-owned-dictionary`, baseline `a55e1cc7c4d7b0df4a187ce3551bfce9141a39d3`, tree `836df880ca711c6813cb78ae1202973e618eb105` |
| Physical checkout | Reused `.worktrees/project01-arc04/vlajmaji`; old Arc04 branch9026 retained; no new worktree/cache |
| Ownership | Explicit df5bec2 handoff effective: retained CC exclusively owns eleven S01 paths/index/ONE target; root/CDC source-read-only, CDC planning/independent preparation |
| Version / date | 0.1 / 2026-10-28 |

## Useful result and boundaries

A caller can maintain an explicitly named dictionary of their own, preserve every prior assertion and editorial decision, export/reimport it without losing history, and load its optional snapshot before SQLite. The Rust API and CLI deliver each increment together. The source is caller-declared `OwnedDraft`, with a named editor and explicit content notices; it claims no community authority.

Arc04 is CLOSED and actually merged as a55e1cc7 (tree-equal accepted9026). Main CI40806671887 SUCCESS is baseline evidence only. Its accepted latency exception, failed campaigns, diagnostic, platform repairs and fixture limitations remain historical evidence; this arc does not optimise BL-0001 or run another Full/performance campaign. All fourteen project rows remain OPEN. Arc06 owns the broader all-pack/live managed workbench and final project/Operator acceptance.

## Architecture and choices

1. **Authority:** a new versioned SQLite owned journal, distinct from rebuildable read-only `DictionaryStore` and upstream collection catalogues. Writes use one transaction and persisted expected-head CAS under `BEGIN IMMEDIATE`. Two already-open writers cannot silently overwrite one another. No automatic last-writer-wins or undo-by-deleting-history.
2. **Model:** existing normalized Dictionary/registry version1 remains the complete typed model. S01 accepts whole validated genesis/candidate Dictionaries and appends new records; old typed bodies/order remain immutable. The current owned head and monotonically incremented catalogue revision are explicit control fields. An edit creates one same-source revision; active IDs describe the complete current set, and explicit tombstones retain removed assertion identities. Existing selected release flags stay immutable; existing owned-head selection already intersects release membership with current active IDs.
3. **Curation/context:** reviewed local senses use existing owned/editorial types. Cross-source curation persists in a distinct version1 owned envelope outside the base Dictionary, with typed `CatalogTarget`, direction, editor/revision/rationale and exact context provenance. No foreign placeholder ReleaseRecords and no copying whole upstream corpora per edit. Absent context is explicitly unavailable/unresolved. With caller-supplied exact validated context, compose the actual bodies FIRST, then materialize reviewed links/groupings with closure to actual releases/assertions. Reject unequal same-ID bodies; never infer identity from spelling or silently retarget. S02 must freeze the precise adapter APIs/wire/status policy before issue; this proposal does not pretend that detail is already executable.
4. **Export/reimport:** separate version1 owned snapshot/seal, preserving the complete ordered journal history and curation envelope. It is not a new `PackId`. Export validation and strict load precede derived indexing. Exact replay is idempotent only under S03's explicit import policy; conflicting same-ID bodies/head histories reject. S01 edit CAS itself has no idempotent stale-replay exception.
5. **Packaging:** optional `vlajmaji-data-owned` and umbrella `data-owned` feature, default empty and `publish=false`, using explicit local snapshot materialization. Existing four upstream `PackId` values/All-four installation stay unchanged. No fifth mandatory upstream input, automatic publication, network load or redistribution claim.
6. **Query boundary:** S01 provides actual init/edit/head/history/show. No assumed in-memory search constructor exists. S03 demonstrates snapshot→Dictionary→explicit fresh `DictionaryStore::create`→existing dictionary query/detail. Arc06 later integrates a live mixed managed view.

## Four usable increments

| Slice | Public Rust + CLI capability | Dependencies and owning acceptance |
| --- | --- | --- |
| [S01 — Owned journal](slice01-owned-journal/slice-plan.md) | Explicit ROOT init/edit/head/history/show, complete typed metadata, transactional CAS and immutable revision reads; shipped minimal seed/edit example and CLI recipe | Existing model/validators and rusqlite; no external context or corpus required. Initial full draft open set below |
| S02 — Reviewed curation/context adapter | Local reviewed senses plus persisted pinned external links/groupings; show unavailable context; materialize against exact explicit Tiny context and query existing reference routes | Accepted S01. Must settle full adapter record/schema/status/error closure and no-retarget controls before issue; journal/envelope migration is explicit, not an unversioned S01 schema change |
| S03 — Sealed history and explicit querying | Deterministic export/check/load/reimport; preserve all historical bodies/heads/curation; exact replay vs conflict; explicit derived index and full/compact/readable Rust+CLI query | Accepted S01–02. Distinct owned envelope/loader, no placeholder composition or default-store fallback |
| S04 — Optional owned package/offline consumer | Default-empty optional owned crate/feature; strict embedded snapshot load before SQLite; outside consumer and actual complete owned workflow | Accepted S03. Disjoint Synthetic materialization; no ignored Full payload overwrite. Existing four upstream/features remain compatible |

These are delivery increments, not investigation-only slices. Re-size only by an explicit preserved scope/owner amendment; do not silently defer curation or metadata preservation to Arc06. Each slice has five total implementation/refinement passes, including its initial assignment; no automatic sixth.

## Validation and evidence

Each child includes actual public Rust and actual CLI positive/negative sequences, exact persistent-state checks, source review, documentation, format→lint→focused tests→make check, affected no-default/feature/package checks and independent CDC disposition. Use existing synthetic fixtures and native-path subprocess infrastructure. Never put large JSON in argv; inputs are files/stdin. No new benchmark or per-command artifact framework.

Use one retained target with `/tmp/vlajmaji-p01-a05-cargo` (SHA256 `426d1e49863551fe37252cecb8d36f8fb58410670fc550c6deedc89ddb92f53d`), explicit +1.98.0/locked/offline. Source/workspace/Bauske sibling and accepted branches remain protected. Child HOME/XDG/TMPDIR are owned and disjoint; pin existing CARGO_HOME/RUSTUP_HOME/PATH before replacing HOME. Feature tests must use known Synthetic payloads or a reviewed disjoint source/materialization recipe; do not probe, overwrite or re-materialize ignored Full files in the reused checkout. Actual user default is never stat/read/changed.

At arc close a fresh independent context actually exercises init→edit→stale rejection→local/external curation→export→reimport→optional load→explicit derived query/history, including absent/conflicting context and old assertions. It reviews the four children and original compatibility boundaries, then SAME reviewer gates the actual canonical packet. CDC supplies child/arc retrospectives and project bubble-up; root checks consistency before mechanical closure. Produced-path inventory is proportionate, not a reconstruction of Arc04 megabytes. No success reports or row closures are prewritten.

## Current readiness

Root final readiness PASS at d75532d was followed by status-only issue e61cb65, complete retained intake/citation erratum acceptance and explicit handoff df5bec2. S01 implementation is ACTIVE, not an unissued proposal. Core/CLI format+lint milestones and the first narrow7/7 public store behavior tests exist; full24/edit-policy/raw/Presence/reactivation/example/CLI/broad/feature/package and independent acceptance remain pending. These author milestones are not a frozen candidate or acceptance. All8arc/7child/14project rows OPEN; one writer owns source/target. S02 now has a source-guided UNISSUED [policy/schema/API draft](slice02-reviewed-curation/slice-plan.md), whose final source reconciliation/CLI report/readiness remain later work. No new external authority.

### Historical draft readiness — version0.1

This is the concrete planning proposal, not an issued source assignment. S01's exact API/schema/append rules and finite regression oracles are in its full slice plan and DRAFT prompt. Root exact readiness remains required before status-only issue, bounded retained CC intake, fresh baseline/pins and explicit source/index/target transfer. S02's detailed adapter is deliberately pending its later readiness; its capability and evidence obligations are already owned here. No unresolved human product choice blocks S01. No push/PR/publication/Arc06 authority is inferred.

## Version history

- 0.1 — 2026-10-28: directive04 Arc05 activation proposal at merged a55; physical checkout/one cache reused; four usable increments and separate authoritative/context/export/package boundaries fixed. All rows OPEN; S01 DRAFT only.

### S01 issue checkpoint — 2026-10-28

Root final readiness PASS at proposal d75532d clears status-only initial issue and retained CC complete intake. Source/index/ONE target remain root QUIET until CDC accepts coverage/readback and explicitly transfers the eleven S01 paths; all8arc/7child/14project rows OPEN. Current S01 status supersedes the dated DRAFT readiness prose above; architecture and later-slice obligations unchanged.
