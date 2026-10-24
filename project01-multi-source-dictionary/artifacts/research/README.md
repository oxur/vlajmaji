# Research carried into Vlajmaji

**Copied:** 2026-10-24, America/Chicago. **Purpose:** make the decisions and evidence
needed for Vlajmaji Project01 accessible from its own planning worktree.
This is a selected research copy, not a migration of the entire research project,
all primary-source payloads, every chat, or its original artistic obligations.

The user requested copying applicable research and confirmed project/arc/slice
`artifacts/` homes. Historical bytes are unchanged. New synthesis belongs in
[the project plan](../../project-plan.md) and [design](../../design.md); copied
instructions and authorization statements are **historical reference material**.

## Exact provenance

Original research home:
`/Users/oubiwann/Library/Application Support/ChatGPT-Profiles/personal/codex-home/.chatgpt-projects/g-p-6a8c72d334048191887d4e00472f8430/relojban-poetics`.

- Research/source branch `feature/bauske-migration`, HEAD
  `b9b76edaa206b2c89f0436059354c2943be7b6da`.
- Original orphan planning HEAD `25620f7e095ed0fbbf733c44ad54be448b8fb407`.
- Current implementation baseline: Vlajmaji
  `4ee3babefa13fbde41c94ab9bfcf61bb029834fe`.
- [copy-manifest.tsv](copy-manifest.tsv) identifies each original relative path,
  copied relative path, SHA-256, byte size and applicable original Git HEAD.

The `relojban-poetics/` subtree retains the original relative layout, including
selected hidden `.worktrees/planning/` records. Each historical file was compared
byte for byte with its original on copying. Originals remain in place. There is
no full dictionary payload, SQLite/Oxigraph database, Cargo target cache or new
upstream fetch in this copy.

Historical links and commands are also unchanged. Some lead to uncopied primary
payloads, probes, old source paths or temporary runs. Resolve those through the
original home plus the original relative path in the manifest, or the recorded
source revision. They are not current Vlajmaji replay commands. Current replay
commands live in the source repo's `docs/migration.md`; its 0.2.0 addendum names
`vlajmaji`. Do not repair historical links by rewriting their source files.

## Reading path and status

| Material | Read for | Verification status / limits |
| --- | --- | --- |
| [Current findings](relojban-poetics/research/lexical-search-findings.md) | PoC queries, boundaries and architecture options. | Historical report; authoritative Arc02 [composition gate](relojban-poetics/.worktrees/planning/project02-wordcraft-reconnaissance/arc02-lexical-data-search/cdc-verification.md) and [close](relojban-poetics/.worktrees/planning/project02-wordcraft-reconnaissance/arc02-lexical-data-search/closing-report.md) establish its accepted scope. No fresh test/benchmark run here. |
| [Import/model report](relojban-poetics/research/lexical-model-import.md) and [its gate](relojban-poetics/.worktrees/planning/project02-wordcraft-reconnaissance/arc02-lexical-data-search/slice02-lexical-model-import/cdc-verification.md) | Raw preservation, IDs, field presence, form roles and jbotci probe. | The authored report retains its earlier proposed-done wording. The later independent gate controls acceptance; do not rewrite either. |
| [Services/standards](relojban-poetics/research/lexical-data-search.md) and [query contract](relojban-poetics/research/lexical-query-contract.md) | DMLex/OntoLex/ELEXIS, API/import loss and exact-source semantics. | Pinned primary evidence and source-level/runtime distinctions; no standards conformance or current service-availability claim. |
| [Storage comparison](relojban-poetics/research/lexical-storage-comparison.md) | What the SQLite/Oxigraph measurements actually covered. | Historical bounded mixed workload; not a new exact-lookup benchmark or a choice among unified/ATTACH/fanout. |
| [Resource findings](relojban-poetics/research/wordcraft-resources.md) and [initial workbench survey](relojban-poetics/research/word-workbench.md) | Corpus/tool roles and originating product questions. | Later pinned trials supersede earlier untested survey claims within their measured scope. No unqualified “best/current” ranking. |
| [Original Arc03 plan](relojban-poetics/.worktrees/planning/project02-wordcraft-reconnaissance/arc03-multi-source-dictionary/arc-plan.md), [first slice](relojban-poetics/.worktrees/planning/project02-wordcraft-reconnaissance/arc03-multi-source-dictionary/slice01-source-registry-and-schema/slice-plan.md) and ledgers | Exact inherited scope, identities, metadata coverage and count/page/availability oracles. | Unexecuted planning; every implementation row was open. New Project01 owns future Vlajmaji decomposition. |
| [Planning reconnaissance](relojban-poetics/.worktrees/planning/project02-wordcraft-reconnaissance/arc03-multi-source-dictionary/artifacts/planning-reconnaissance.md) | Pinned jbotci raw-vs-compiled distinction and established SQLite composition constraints. | Metadata/code recheck dated 2026-10-23. Full raw jbotci payload/counts were not independently validated there. |
| [Poem opening](relojban-poetics/research/last-snow-opening.md) | The consumer and source-backed ambiguity that motivated this toolkit. | Exploratory artistic candidate; no reader/singing/Relojban-idiomatic acceptance. It is not lexical attestation. |
| [Vlajmaji migration](relojban-poetics/research/vlajmaji-migration.md), [Bauske migration](relojban-poetics/research/bauske-migration.md) and preserved migration gates | Old/new package homes and replay limits. | Historical acceptance; current names/versions/paths come from today's inspected source, not old CLI wording. |

## Source bibliography and claim classes

Primary source URLs/pins and capture provenance remain in the copied
[bibliography](relojban-poetics/bibliography/README.md),
[E02 manifest](relojban-poetics/experiments/E02-resource-discovery/manifest.json),
[E03 capture manifest](relojban-poetics/experiments/E03-lexical-data-search/slice01/raw/capture-manifest.json)
and [storage primary-source URL map](relojban-poetics/experiments/E03-lexical-data-search/slice03/derived/source-url-map.md).
The manifests point to original payloads; copying a manifest does not copy or
freshly validate every payload it lists.

| Source family | Role / pinned evidence | Current treatment |
| --- | --- | --- |
| LLG gismu/cmavo/rafsi lists | Language-source assignment/component baseline; E02 captures. | Required corpus family; historical list content is not current universal authority. |
| La-Lojban and Sutysisku | Lexical distribution/projection inputs; pinned E02 inventory. | Both required optional collections; preserve upstream lineage and overlap. |
| Lensisku via jbotci | Public vendored raw export and metadata at `3d9fd41d9ed32de26311ee27ec86b6b165985aa1`. | Required dedicated importer. Supplier 33,526/31,248 counts are not freshly measured counts; compiled selected output is lossy for our purpose. |
| OASIS DMLex 1.0 OS, 2025-04-29 | Lexicographic representation and modules; E03 normative evidence. | Schema vocabulary/mapping with explicit gaps, not a conformance claim. |
| OntoLex-Lemon, 2016 Community Group specification | Entry/form/sense/ordered-component and relation vocabulary. | Informs modelling; not a W3C Standard or a mandate for RDF storage. |
| ELEXIS REST 1.2 and pinned reference `2bcb77ea7b755df51b3cb1e2f42f7c6ad9784d67` | Service/interchange protocol and inspected reference code. | Historical pinned build failed; runtime probes not run. No new service dependency proposed. |
| SQLite official docs and Datasette precedent | ATTACH, runtime limits, URI/immutable and cross-file transaction constraints. | Inputs to later bounded composition decision; no local comparison result yet. |
| Rust library/project source and our observed probes | Import/morphology/search implementation behavior at specific versions. | Role-specific evidence; code terms and data redistribution remain distinct. |

Authored research reports are **secondary synthesis**, despite linking primary
sources. Raw captures/code/specifications are primary material. Later independent
gates are evidence of the bounded PoC result. This project's design is a new
proposal grounded in them. Supplier metadata, historical observations and new
choices must keep these labels when quoted downstream.

`prior-citations.tsv` retains unresolved original conversation citation markers;
copying it does not resolve those citations or reproduce the full original reply.
No historical unknown, failed attempt, unrun probe or artistic decision is upgraded
by this planning pass. Fresh acquisition/version/source-rights checks belong to
the implementation slice that needs them, under its own authority.
