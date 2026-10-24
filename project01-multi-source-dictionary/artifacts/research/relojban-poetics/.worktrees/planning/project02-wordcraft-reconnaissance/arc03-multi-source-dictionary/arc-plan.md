# Arc03 — Multi-source dictionary

| Field | Value |
| --- | --- |
| project | project02-wordcraft-reconnaissance |
| arc | arc03-multi-source-dictionary |
| status | planned / all acceptance rows OPEN; planning authority only |
| depends-on | accepted Arc02 source7a05516 and planning1b9f741; existing source-preserving model/search and pinned E02/E03 inputs |
| blocks | reusable source-grouped dictionary with reproducible optional corpora and an editable owned source |
| version |1.0,2026-10-23|

## Outcome, decisions and authority

Duncan has adopted the SQLite relational direction, a thorough DMLex/OntoLex-informed schema, default definitions grouped by named source with stable abbreviation, preservation of every assertion, reproducible scripted jbotci import, an optional data crate/Cargo feature for each corpus, and our editable dictionary as a first-class source. This arc plans the implementation of that capability. It is not a separate design-only arc.

Current authority is **planning only**: prepare this arc/ledger and first slice/ledger in the confirmed orphan worktree. Execute no slice, create no CC prompt or new agent, change no source/code/accepted acquisition, run no importer/build/benchmark, schedule no refresh and publish nothing. Source remains7a05516 on feature/project02-lexical-search; planning baseline1b9f741. Parent/root is read-only while CDC owns these planning updates; candidate review precedes a scoped planning commit. Future CDC SolHigh/CC LunaHigh selections are retained intent, not renewed Autonomous execution authority or backend attestation. No model/substitution/CRC/layout decision is requested. A later execution instruction must establish its dedicated non-default source branch, author/readiness packet and independent review; none is issued here.

Accepted: SQLite technology, source-grouped assertion semantics and required corpus packages. **Not selected:** one unified database versus attached databases versus application federation, concrete migration/DDL, public data redistribution, projectwide license or deployment. Slice04 freezes the storage/dispatch decision before Slice05/06 integration. Standards inform the schema; conformance is not inferred. Project01Arc02 remains parked.

## Six useful implementation increments (future; none executed)

| Slice | Usable result and delivery | Dependencies / boundary |
| --- | --- | --- |
|01-source-registry-and-schema| Versioned source/lineage/abbreviation registry, thorough relational/Rust schema foundation and a grouped exact-lookup contract demonstrated on discriminating fixtures; schema/mapping documentation.| Accepted v2 raw/provenance model is the baseline. Small isolated SQLite fixtures do not select multi-corpus topology. Detailed plan below. |
|02-verbatim-jbotci-import| A reproducible script/manual-or-CI-capable check acquires the pinned vendored raw dictionary-en.json+metadata, retains every row/field/alternative, transforms to the accepted schema and emits a reconciled normalized release.| S01 schema/identities. Dedicated slice; do not substitute english() or best-definition output. Default acquisition is public pinned jbotci bytes, without Lensisku credentials. |
|03-optional-corpus-packages| Each required upstream collection has its own optional data crate and Cargo feature, immutable raw/snapshot metadata, schema/versioned normalized release, notices and runtime inventory; combined selection works offline, with network-free ordinary build/load and default/per-corpus/combined package tests.| S01/S02. Required collections: jbotci-delivered full Lensisku, LLG list family, La-Lojban and Sutysisku. This is not a one-pack experiment with others deferred. Physical SQLite layout remains unselected. |
|04-sqlite-composition-decision| Bounded same-record/query comparison of a source-keyed unified derived index, SQLite ATTACH and application fanout; a documented implementation decision with complete-set/pagination/availability oracles.| S01 contract+S03 releases. Evaluate all three candidates: ATTACH has inspected SQLite/Datasette precedent; a unified derived index extends the accepted local store; application fanout is a design candidate with no established Rust federation implementation inspected. No graph-DB rematch or ecosystem survey. Freeze decision before dependent integration. |
|05-owned-dictionary-revisions| The owned dictionary uses the same source/schema API, retains editorial revisions/history and explicit cross-source curated assertions, exports immutable snapshots and has its own optional data crate/feature.| S01 schema+accepted S04 decision. A mutable working source and sealed exported release are distinct. It is not an override that replaces other sources. |
|06-grouped-lookup-and-replay| Reusable grouped multi-source Rust lookup/CLI, source filters/aliases, totals/pagination/links, selected-pack removal/reload and owned edits integrated under the chosen topology, with offline replay/documented packages.| Accepted predecessors. Preserve all conflicting assertions and expose provenance, completeness/availability and score policy. Final composition/retrospectives/remaining decisions required if execution is later authorized. |

Only Slice01 is detailed now. Later slice directories and assignments wait for predecessor acceptance and execution authority. Split an oversized increment before issue rather than reducing the required corpus/schema obligations.

## Source identity, meaning and selection invariants

Author/contributor, asserting/upstream dictionary, lineage relationship, distributor, corpus collection, captured snapshot, normalized release and individual assertion occurrence are separate identities. Abbreviations are stable registry labels/aliases, never primary keys or proof of independence. Prefer a source's verified convention; otherwise label the abbreviation explicitly project-assigned. Do not silently reuse a retired alias or equate renamed source/distributor IDs.

Default lookup preserves all matching definitions/claims grouped by registered source. Each row retains author, upstream IDs when supplied, distribution/snapshot/raw locator and lineage. Multiple Lensisku projections do not become independent attestations; repeated observations remain available. Identical text alone does not prove identity or license deduplication. An explicit owned editorial sense-link/group may connect assertions across sources with editor/revision/rationale; it neither rewrites upstream senses nor silently creates a winner. Missing structured places/relations remain unknown; parsed/extracted morphology and prose analyses have tool/run/version provenance.

Required corpus collections are a finite priority set justified by accepted E02/E03 coverage and the new pinned jbotci inspection, **not an unqualified claim of best quality or present freshness**. Proposed package names are implementation candidates: relojban-data-lensisku-jbotci, relojban-data-llg, relojban-data-la-lojban, relojban-data-sutysisku and (S05) relojban-data-owned. Each has its own opt-in corpus feature; core code has no mandatory dictionary payload. LLG's gismu/cmavo/rafsi lists are separately identified components within its collection manifest; if packaged as separate corpora, each receives its own crate/feature. No all-corpora monolith substitutes for independent packages.

Each release records exact input bytes/hash, upstream/distributor revision and time precision, component schemas, normalizer version, counts/diagnostics and code/content notices. Raw captures are immutable; normalized/indexed material is reproducible derived data. Core schema and data-release versions are distinct. Unknown/unsupported schema fails explicitly; it is not guessed into an older type. Source-pack removal hides active rows without erasing catalog/history or breaking owned links: targets become explicitly unavailable and recover when the matching release returns.

## SQLite dispatch contract and bounded comparison

No database topology is selected by crate boundaries. S04 compares the same registered sources/records/queries through: (a) one derived SQLite database keyed by source/snapshot; (b) sealed per-pack databases attached to a connection with an appropriately mutable owned store; (c) application dispatch across stores with complete merging. ATTACH is built in; Datasette --crossdb is an inspected precedent, not an adopted dependency or a local result. Compare query sets, cross-source links, totals/pages/order, pack selection/removal, schema mismatch and owned revision consistency before costs. No local per-database FTS rank is assumed globally comparable; use a declared global policy or reject unsupported ranked merging rather than merge local top-k results.

References and SQLite constraints are in artifacts/planning-reconnaissance.md: connection attachment limits, qualified table names, URI activation, cross-file transaction/WAL conditions and immutable-file requirements. Readonly is not equivalent to immutable. Only sealed, never-changing packfiles may use immutable=1; the editable owned database must remain writable and use normal locking. Compare indexed results to complete reference sets, including apostrophe/y/diphthong distinctions. Query cursors bind source/release set and owned revision, so concurrent changes cannot silently skip/duplicate definitions. Runtime/version/build limits must be observed in S04, not assumed from documentation.

## Preservation, gates and research bound

Use existing accepted evidence by immutable reference; preserve every issued prompt/attempt if execution is later authorized. All rows start OPEN. Future slice acceptance requires source/public-caller review and appropriate Rust1.98/default/feature/no-data checks, full multiplicity/set/cursor/role oracles and real independent disposition. Fresh arc composition must replay raw→normalized release→selected SQLite dispatch→grouped API/CLI→owned snapshot. Methodology retrospectives and project carryover remain mandatory under any renewed Autonomous execution; planning documents alone prove no implementation.

Each source/release has a **conditional public-redistribution gate**: record the applicable upstream/distributor/contributor evidence and notice scope separately from code license; mark public publication held until resolved for that release. Supplier public-domain statements are evidence, not automatic clearance. This does not blanket-block authorized internal planning/evaluation or the internal optional packages. No public package/crate publication or license choice is authorized here.

Artifacts for this planning pass: one compact artifacts/planning-reconnaissance.md, preserving pinned facts/receipt fields/limitations needed for the raw-vs-compiled importer decision. No full corpus was acquired or imported now. The future slices use their own default artifacts/ homes; actual delivery code/docs belong in crates/* and the research checkout after authorization.

## Version history
- v1.0 —2026-10-23: planned the requested six implementation increments after SQLite/schema/source-grouping decisions; first-slice planning open set only, no execution/assignment or acceptance.

## Current source-home override — v1.1, 2026-10-23

The v1.0 source7a05516/R-checkout statements above describe the planning baseline and remain historical. Accepted bounded migrations now place all lexical/store/search/CLI implementation in `/Users/oubiwann/lab/oxur/vlajmaji` at e20e60b5e9043a6d5e8c6d712348f8094a4a3967, with `crates/{lexicon,store,search}` and singular `crate/cli`; public packages use vlajmaji names. Phonology is `/Users/oubiwann/lab/oxur/bauske` at9c409baefc027c158dec387d857e24a47a1075ab, consumed through sibling ../bauske/crates/phonology. R b9b76edaa206b2c89f0436059354c2943be7b6da retains research/planning/evidence and has no active Cargo workspace. See ../migration-bauske/cdc-verification.md and project-plan.md v4.1 for exact mapping/replay limits.

Any later Arc03 implementation assignment targets V's renamed libraries/new corpus packages, not removed R/crates paths, and establishes its own branch/readiness. Preserved P01/E02/E03 sources stay at their old identities/refs. All six increments/schema/corpus obligations and all8 OPEN rows remain unchanged. No Arc03 execution/prompt exists; this is source routing only. Future data-package names remain candidates, not implementation acceptance.
