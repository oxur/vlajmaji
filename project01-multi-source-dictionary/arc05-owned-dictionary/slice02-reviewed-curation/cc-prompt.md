# CC assignment — Arc05 S02 reviewed curation

STATUS: DRAFT — UNISSUED; accepted S01 pinned below; root exact DRAFT readiness pending.

## Useful result and authority

Add reviewed owned links/senses with explicit immutable external context, persistence and migration, truthful absence, and actual public Rust/CLI materialization into an existing derived query. [slice-plan.md](slice-plan.md) §§1–6 is the binding API/wire/schema/status/algorithm/error contract; all eight [ledger](ledger.md) rows OPEN. This draft does not transfer ownership or permit execution. Directive04 authorizes local work THROUGH ARC05 ONLY, same retained Luna/high CC and Sol6.1/high CDC; initial plus corrections FIVE total S02 passes, no automatic sixth. S01 is locally accepted as recorded below; root exact readiness is still required before issue. No Full/default/network/external delivery/Arc06.

Baseline: accepted S01 `1b899ebb492d89ceb1561abb0befb17ff3ccc5a1`, tree `745f789e3ddef62a2c2977c95a1d76f6f12382db`, parent a55; lock SHA256 `65cd7f39a88bdd2a3dab1fcd14e9055ff7b77cb22ec3af8c309ce49dbea78700`, guard SHA256 `426d1e49863551fe37252cecb8d36f8fb58410670fc550c6deedc89ddb92f53d`. Whole S01 local acceptance and platform/process limits are in ../slice01-owned-journal/cdc-verification.md. The physical source checkout remains `/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc04/vlajmaji`, branch `codex/project01-owned-dictionary`. Its historical directory name is not the work scope. No source/index/target operation before complete intake and explicit transfer.

## Required intake — current contract, not accumulated history

Planning root `/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/planning`; skill root `/Users/oubiwann/Library/Application Support/ChatGPT-Profiles/personal/codex-home/skills`. Recover truncated required extents before readback. Immutable prior intake may support unchanged background, but cannot attest newly added contracts or code.

| Kind | Material / extent | Why |
| --- | --- | --- |
| Required-full | This prompt, S02 plan and ledger, project directive04; actual accepted S01 cdc-verification.md (FULL) | Exact new contract, inherited obligations and proven dependency |
| Required-section | Planning AGENTS current Arc05 authority/standing rules; project plan current Arc05 boundaries; project design owned authority/context adapter; arc plan useful result/increments/current authority and complete arc ledger | Scope without reloading historic Arc04 megabytes |
| Required-full | Source AGENTS; lexicon/store/CLI Cargo manifests and exports; accepted S01 owned.rs, owned_schema.rs, store contract tests and shipped example | Concrete journal/rollback/summary reconstruction and public compatibility, not an imagined API |
| Required-section | lexicon dictionary types LocalRef/CatalogTarget/Direction/RelationRecord/EditorialSenseRecord/ExtensionRecord/OwnedRevisionRecord, Dictionary strict parser/serialization/accessors, global ID closure and relation/sense/revision/extension validation with invoked helpers; registry source/attribution types and identity conversion/validation helpers | Real global identity, exact provenance and normalized constraints |
| Required-section | lexicon pack.rs PackError and compose_dictionaries plus complete invoked composition/metadata merge machinery; lexicon lib.rs duplicate-key checker and RawJson representation/serde | Actual consume/merge behavior and strict original nested bytes |
| Required-section | CLI owned command types/dispatch/errors and shared success/error emitter; store NativePathReport/native_path_report; dictionary-build/query existing routes and public reference entrypoints used in the example | Existing native/IO/query lifecycle; no invented wrapper |
| Required-section | Current Make and CI Synthetic/allfeatures routes; prompt-authoring guide required-reading/intake sections; Rust AP18/19/33/67, API02/03/04, EH04/13/14 | Borrowing, error/source/rollback truth, required gate applicability |
| Conditional | Complete typed Tiny fixture/helper bodies if reused, plus every dependent conversion/remapping; existing native IO helper/test bodies if reused | No partial copy or input oracle selected from product output |
| Reference-only | Historical Arc01–04 records and old S01 compile/fixture failures except actual disposition/evidence needed for a current claim | Preserve provenance without turning history into an active full-read contract |

Record actual coverage/source pins and source-cited readback in S02 `artifacts/execution.md` only when intake begins. Explain schema1 vs schema2 behavior/migration/no hidden read verification; exact base vs curation history boundary; persisted-schema/head CAS and rollback; whole-context canonical pin/inventory and stable target; all-or-nothing ordered sense/dependency; unknown native vs wildcard; materialized provenance extensions and actual context composition; output-file/stdout boundary; exact allowed paths/resource guards. Resolve contradictions before edits. No enumeration-only comprehension claim.

## Implementation spine

1. Add a PURE lexicon owned_curation module and re-exports exactly as plan§§2–3. Existing normalized model/registry versions and PackId/All-four remain unchanged. Public validated wrapper has private invariant-bearing data; strict whole-envelope duplicate/syntax checks precede version/full decoding, nested Dictionary uses ORIGINAL RawValue bytes through its public strict parser. Supply the public strict OwnedContextInput codec; CLI must not invoke a crate-private checker or roundtrip a Value. Reuse existing error/dependency facilities and concrete source chains/Errors docs.
2. Separate pure CURRENT owned-base checks from journal historical proof. The pure helper checks exact single-source ordered owned chain/head/catalogue/active/tombstones and immutable context/curation shapes; it cannot infer when old base records were appended from a final Dictionary alone. S01 stored rosters and migration's actual historical reconstruction retain that evidence. Do not import store into lexicon or weaken historical preservation.
3. Materialize ONLY supplied actual context bodies. Check roster first, then whole canonical SHA/inventory in pin order; reject unknown/duplicate/wrong/conflicting supplied inputs, while missing inputs are visible typed absence. Consume inputs in pin declaration order, clone owned base once for output composition, build borrowed primitive identity indexes once, and preserve record/member order. Adopt existing compose semantics: notice/lineage exact-value dedup, extension maximum-input multiplicity, keyed equal-ID collapse/unequal rejection, first-input role ordering and catalogue maximum. Original journal/context authorities stay exact; the derived view is not whole-vector concatenation. No competing merge/restoration algorithm or pack.rs edit. No foreign placeholders/corpus copy per journal edit, spelling inference, partial sense, request cache or Full wildcard-native admission. Resolve senses before dependent links; preserve exact normalized records plus declared provenance extension, then validate the complete returned Dictionary. Resolved is identity, not lexical selection/license/equivalence.
4. Keep S01 init/create schema1 and old signatures. Implement explicit schema1→2 migration and new curated operations exactly as plan§4: persisted expected-head/schema INSIDE BEGIN IMMEDIATE; old base hashes/24 rosters/body/order/head/catalogue unchanged; four new STRICT sidecar tables/empty histories atomically added only after named whole-history verification. AlreadyCurrent is minimum header/head/no-write, not repeated verification. No implicit open/read/edit migration, consumed-Transaction rollback fiction or relaxed outcome-unknown policy. Schema2 ordinary edits carry prior curation selection without claiming another review; curated edit commits base and sidecar atomically. Curation-show is explicit full base+sidecar verification; head/history stay summaries.
5. Add owned migrate-curation/curate/curation-show/materialize with plan§5 static-error precedence. Context files are strict `{id,dictionary}`, not ID=path or JSON argv. Reuse existing structured error/Json write+flush. Materialize validates semantically before create_new output, writes exact canonical Dictionary bytes WITHOUT LF and reports the exact declared DTO fieldset through existing QueryOutput::Json(Value), preserving its sorted map-key transport and one LF. Rust declaration order is NOT the stdout byte-order contract; no new transport variant. File/stdout failure cannot undo completed output or claim a journal edit. Preserve partial owned files honestly.
6. Ship one small typed curation example: input-derived own r2/link/three-member sense and Tiny real upstream context, explicit migration/commit/absence/materialization, then actual dictionary-build into a separate derived ROOT and existing reference query. Derive full IDs/occurrence/native expectations before queries; show missing context and complete counterpart, no candidate-picked oracle. This normalized output is not S03's sealed owned snapshot or reimport schema.
7. Complete the finite controls in plan§6 and eight ledger rows. Batch independent fixture edits; a valid-but-immutable candidate must actually reach journal policy, while constructor-invalid graphs are separately classified. Preserve original failures/argv/source extent. Core discriminators include historical migration equality/Busy/restore/two schema handles; pin/record/selection prefixes and reactivation; unknown/changed context/SHA/body conflicts; missing middle sense/no partial group; exact ordered projection/provenance/native negatives; unchanged duplicate-bearing authority inputs versus declared notice/lineage-dedup view and extension maximum multiplicity; strict duplicate/version wire; real native file/stdin/output and derived Tiny Rust+CLI. No new broad matrix/benchmark/backend/accessor.

## Exact source allowlist — reconciled against accepted S01

Only these FOURTEEN existing/new paths, checked against accepted S01 (existing paths remain; the named new modules/tests/example are absent at baseline):

```text
crates/lexicon/src/lib.rs
crates/lexicon/src/owned_curation.rs
crates/lexicon/tests/owned_curation_contract.rs
crates/store/src/lib.rs
crates/store/src/owned.rs
crates/store/src/owned_schema.rs
crates/store/tests/owned_curation_contract.rs
crates/store/examples/owned_curation.rs
crates/store/README.md
crates/cli/src/main.rs
crates/cli/tests/owned_curation_contract.rs
crates/cli/README.md
docs/owned-dictionary.md
README.md
```

No manifests/lock/dependencies, normalized dictionary/registry/pack model rewrite, upstream/search/provider/data/Bauske/workflow or upstream/derived schema, S01 test rewrite or unrelated formatting. If actual source shows a needed spill, stop with the concrete reason before editing outside this set. Do not add a test-only public accessor or invented migration hook.

## Gates / resource and return

ONE retained target `/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc01/target`; guard `/tmp/vlajmaji-p01-a05-cargo` SHA256 `426d1e49863551fe37252cecb8d36f8fb58410670fc550c6deedc89ddb92f53d`; Rust +1.98.0, explicit --locked --offline where supported, Make CARGO override, existing CARGO_HOME/RUSTUP_HOME/PATH pinned before owned HOME/XDG/TMPDIR. Use require_escalated from outset for known authorized source/planning/target writes outside writable roots; scratch default. No new target/download/default/Full/cleanup/reset or benchmark.

After explicit transfer: scoped stdin rustfmt→exact named copyback and read-only fmt-check; Clippy workspace/all-targets -D warnings BEFORE behavior; whole new lexicon/store/CLI curation suites plus affected S01 regression suite; actual example/public+CLI journey; make check and isolated no-default; allfeatures only in disjoint candidate source archive with protected ignored Full inputs excluded, Synthetic four fixtures in archive only and read-only Bauske sibling. Before every archive Cargo exec, assert physical cwd equals that exact archive and pass that explicit workdir; keep the frozen guard unchanged.  actual cargo package --list --allow-dirty --locked --offline for affected lexicon/store/CLI FILE inventories (not test --list). Retain existing gates only with explicit identical-source applicability; do not relabel old source receipts current. No automatic failed hard-gate replay or sixth pass.

Return clean HEAD/tree/parent/exact delta and body hashes/lock+guard, exact argv/env/exits/raw paths/counts, actual finite-control census with limits, public example/CLI result and eight-row proposed disposition, source/index/ONEtarget QUIET handback. Explicit named paths with git commit --only and adjacent trailers:

```text
Co-authored-by: Codex <noreply@openai.com>
Co-authored-by: Billo AI <ai-engineering@billo.systems>
```

Author closing report only after actual work; CDC independently reviews/proves relevant behavior and retrospective/bubble-up before closure. No prewritten verification/success, push/PR/publication or Arc06. This remains UNISSUED until root's exact readiness review; S01 acceptance is pinned above.
