# Slice01 — Source registry and schema foundation

| Field | Value |
| --- | --- |
| project / arc | project02-wordcraft-reconnaissance / arc03-multi-source-dictionary |
| slice | slice01-source-registry-and-schema |
| status | planned; seven rows OPEN; no CC assignment or execution |
| depends-on | accepted source7a05516, Arc02 model/SQLite/search and source evidence; new source-grouping/schema decisions |
| blocks | faithful full jbotci import, optional corpus releases, topology comparison and owned dictionary |
| version |1.0,2026-10-23|

## First usable result (future implementation)

A versioned source registry and thorough shared lexical model, relational-schema foundation and reusable grouped exact-lookup API, demonstrated on a finite multi-source fixture. This lets S02 ingest the full raw export without throwing away unselected definitions and lets later packs/owned edits use the same contracts. Delivery after authorization belongs in the existing crates/* libraries and schema/mapping documentation; no source file is changed by this planning open set.

Inspect the actual accepted relojban-lexicon/store/search API and tests before issuing implementation. Preserve its useful raw archive/identity, alternative definitions, role distinctions and explicit errors. Use an additive or explicitly versioned schema/adapter/rebuild transition; never rewrite accepted v2 captures/evidence or claim a new format was already validated. A small standalone SQLite schema fixture may validate relational constraints here without choosing whether the final application uses one or multiple files. S04 owns that decision.

This planning-only open set intentionally contains slice-plan.md and ledger.md, **no cc-prompt.md**. The Operator has not authorized implementation; normal complete assignment/readiness/intake rules apply before any later dispatch. No initial or corrective prompt has been issued; refinement count0. Future durable evidence home is this slice's artifacts/ (not created speculatively); actual code/docs live in the source checkout.

## Logical schema coverage and assertion boundaries

The schema must cover these areas with explicit typed keys/relationships and a documented raw-preserving extension path. S01 chooses concrete tables/types through actual source readiness, not an invented universal semantic ontology.

| Area | Required representation/invariant |
| --- | --- |
| Registry and lineage | Stable source ID distinct from display name/standard-or-project abbreviation; alias history/ambiguity rejection; contributor identity separate from dictionary/institution/distributor; evidence-backed lineage edges, unknowns preserved. |
| Corpus/snapshot/release | Collection and component IDs, immutable raw payloads and locators, upstream revisions/time precision, distributor capture, hashes/schema/normalizer versions, availability and release notice scope. File path/ATTACH alias is not identity. |
| Lexical entries/forms | Source-entry identity, exact written form and language/script; explicit variants/pronunciation when supplied; source native word class/selma'o distinct from UD/English POS mappings. Keep apostrophe/dot/y/case/diphthong bytes; normalization is a separately named view. |
| Definitions/senses | Every definition/assertion occurrence, optional upstream definition/sense IDs, language/markup, contributor, score/timestamps as source metadata. A repeated row still has its own locator. A definition is not automatically a newly inferred sense, and equal text does not establish shared identity. |
| Glosses and examples | Definition, gloss, notes, usage/domain/register/labels, examples, citations and translations remain separate with source-field/presence provenance. Unknown extension fields/JSON types survive. Missing/null/empty stay distinguishable in canonical data; projected API fields state their limits. |
| Arguments/grammar | Source-asserted structured slots/frames only when present; original definition prose stays prose. A jbotci place parse can be stored as a versioned computed analysis, never a supplied argument assertion. farvi4/5 remains unresolved. |
| Rafsi/morphology | Official/experimental/source-reported form claims attached to their actual definition/source; predictable4-letter stems, extracted rafsi and generated/decomposed components are distinct derivations. Preserve form-level standing and ordered components; no etymology inference. |
| Relations/links | Explicit entry/sense/form/assertion target identity and relation scope/direction, raw citation/offset or upstream link, unresolved and unavailable targets. Cross-source links resolve via catalog IDs, not physical database names. |
| Owned revisions | First-class owned source keys, editor/revision/provenance, append-preserving assertion changes/tombstones and snapshot exports. Removing a pack or editing a headword must not erase previous assertion references/history. |
| Optional editorial senses | Explicit editor-authored cross-source grouping/mapping with referenced assertions, revision and rationale; no automatic merge, preferred source or sense winner. Ordinary lookup does not apply editorial suppression. |
| Computed annotations | Tool/version/run/input/profile IDs for morphology, spelling or P01 analysis; no recomputed value overwrites upstream data or becomes dictionary attestation/perceptual truth. |
| Terms/interchange | Separate code/content notices and per-source release status; DMLex/OntoLex mapping table including unsupported/lossy fields and required sidecars/extensions. Do not claim conformance from naming tables after a standard. |

Use pinned DMLex1.0/OntoLex evidence already accepted in Arc02. DMLex Core's same-language restriction and Crosslingual headwordExplanations cardinality do not justify flattening competing English explanations for a Lojban headword. Preserve alternative assertion IDs; describe a faithful candidate mapping or explicit unmapped fields rather than inventing senses to pass a schema. OntoLex is a Community Group specification; an informed relational model does not require RDF storage.

## Source-grouped API contract to implement and demonstrate

Default exact lookup returns every matching definition/assertion reference grouped by registered named source with its stable abbreviation. Within each group show contributor, corpus/distributor/snapshot and raw/upstream locator; repeated observations of the same upstream assertion remain traceable and are not counted as independent corroboration. Default is no best-definition/source selection. Explicit caller editorial views may add curated links without changing source rows.

The selected source/release set and owned revision are part of query identity. No source filter means all enabled/selected available sources, including owned current data when present; an explicit missing/disabled/ambiguous source request is a typed error, not a misleading zero. Resolve abbreviations through the registry; keep stable IDs in results and links. Source-pack availability is disclosed separately from absence of a definition.

Return full matching assertion/occurrence totals, distinct spellings, source-group count and per-source totals before transport pagination. Keep upstream-identity/independent-lineage counts separately named if offered, never substitute them for occurrence counts. Nonlexical/unsupported raw rows remain in import accounting; query eligibility/exclusion reasons are explicit.

Exact lookup's default all-definitions view is unpaged. Optional explicit page_size pages **assertion occurrences**, ordered by stable source ID, exact spelling bytes, assertion/occurrence ID; grouped display marks partial groups and carries their full counts. page_size0 returns no assertion rows with actual totals. A cursor binds schema/query/filters/source-release set/owned revision and last ordering key; unsupported/stale cursors return typed errors. Concatenating all positive-size pages must equal the complete unpaged ordered result, without missing/duplicate rows. Renaming an abbreviation or pack pathname does not change identities; a source-set/revision change cannot silently reuse a cursor. Do not mix per-database FTS scores; later ranked search requires an explicit shared policy.

## Finite implementation/verification spine

1. Inspect accepted types/fixtures and the pinned native field shapes; produce the schema/standard mapping and registry invariants above. Pin newly applicable Rust/API/serde/SQLite guidance before any future CC prompt; reuse only genuinely complete unchanged receipts.
2. Implement versioned registry/model/relational constraints and public grouped exact-lookup foundation on a small labelled fixture through the existing source-preserving boundary. Complete schema coverage/extension policy is required, not just a single flat definitions table. No full-corpus acquisition or jbotci payload transformation in S01.
3. Demonstrate at least two upstream sources, two distributors of one source, same-spelling competing definitions, exact duplicate observations, author IDs and absent author, abbreviation rename/collision, missing/null/empty, separate gloss/form-only row, structured-vs-prose places, wiki/unknown metadata and an owned draft. Include si'e/sie/dot/curly-apostrophe distinctions, assigned si'e versus computed snim/cpan, and farvi alternatives. Fixtures are explicitly synthetic or pinned witnesses, not new attestation.
4. Verify raw preservation and referential constraints, lineage-aware grouping without auto-merge, default-all/explicit-source behavior, exact totals and full-vs-pages equality at page sizes0/1/several, empty matches and stale/invalid cursors. Remove/re-add a fixture pack and edit an owned fixture revision: stable references survive with explicit unavailable/history state. No production owned editor is implemented here.
5. Exercise a copied public Rust caller and exact/rebuild schema recipe; run appropriate Rust1.98 format/check/test/Clippy/docs with feature variants affected. Schema/library/API docs and meaningful negative controls are part of delivery. Reserve broad all-corpus feature checks for S03 and real topology/cost comparison for S04; do not run them merely to fill evidence.

Exit requires all seven rows independently dispositioned after actual implementation. Source-readiness/plan existence is not a delivered schema or acceptance. Later details depend on that accepted result. Stop/escalate a substantive schema/semantic/rights decision rather than silently dropping fields or widening to ontology inference. Internal planning is authorized; public redistribution remains a per-source release gate and does not block this foundation fixture.

## Assignment/readiness status

No current CC assignment. Planning baseline/source identities and current recheck limits are in ../artifacts/planning-reconnaissance.md. Usefulness: unblock lossless full import and all subsequent corpus/owned/source-grouped increments. Applicability: existing flat source projections lack the requested registered grouping/revisions/pack semantics. Bound: this schema/fixture foundation, not the full import, corpus packaging, topology benchmark or owned editor. No implementation effort budget or execution authority is inferred from this plan; record those when an actual assignment is authorized.

## Version history
- v1.0 —2026-10-23: first-slice implementation plan/ledger only; no prompt, execution, new agent or prefilled close.

## Source-home override — v1.1, 2026-10-23

The approved code migration changes the future implementation location only: use V `/Users/oubiwann/lab/oxur/vlajmaji` (e20e60b5e9043a6d5e8c6d712348f8094a4a3967), packages vlajmaji-lexicon/store/search and singular crate/cli. Linguistic dependency is B sibling ../bauske/crates/phonology, package bauske-phonology. R is research/planning only; source7a05516 remains the historical prototype/evidence baseline. Arc plan v1.1/project plan v4.1 and ../../migration-bauske/cdc-verification.md govern this routing. All7 rows remain OPEN, no schema/import implementation or CC prompt/dispatch; previous semantic/count/page/source-fidelity oracles are unchanged.
