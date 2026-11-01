# S04 — Typed literal definitions and stored forms

**DRAFT0.3 / UNISSUED / NOT READY,2026-11-01.** ROOT approved a source-grounded split after review of draft0.1 and source-backed sizing; prior0.1/0.2 are retained in Git7c429165/00368855: S04 delivers ReverseDefinition+stored Form; S05 owns bounded Links, its parser compatibility and relation/clause detail. No Links/edge request, response or detail stub is exposed by S04. Current source baseline8f373fa0/tree8f0d is accepted S02; S03 is WIP with CC exclusively owning its13paths/index/ONEtarget INITIAL/pass1. Final accepted S03/API reconciliation, exact source/wire/detail/oracle/readset/allowlist and ROOT readiness precede S04 issue. All8 child/eightarc/fourteenproject rows OPEN; through ALL Arc06 endpoint unchanged.

## Useful result and additive boundary

Public Rust and native CLI find mixed upstream/owned assertions by literal definition substring, or exact stored forms by their true kind/role, with complete source provenance and deterministic Full/Compact/Readable pages/detail. Proposed `execute_lexical(&WorkbenchLexicalRequest)` and `execute_lexical_compact` direct/configured methods, plus `workbench lexical ROOT --request FILE --format full|compact|readable` and configured equivalent. The new request/result enum contains ONLY ReverseDefinition and Form in this increment. A new `vlajmaji-workbench-lexical` response family preserves existing DictionaryRequest, old workbench query, S03 reference and managed/legacy wires. S05 can later add implemented Links variants to this additive family; it is not exposed as a placeholder now. S06 owns computation/generation/proposals; S07 owns identified functional Full/clean caller/fresh assembled/SAME canonical/ROOT. No Archive fabrication, linguistic inference, schema/model/dependency/default/source write or new performance claim.

## Actual stable source basis

- search/lib.rs78–177/536–567: ReverseDefinition rejects empty literal text; Form forwards exact value and optional FormRole without an empty/trim rejection. store/sqlite.rs248–331 and store/lib.rs279–300 preserve ASCII-lower literal substring and byte-exact forms, not global rank fusion.
- lexicon/dictionary.rs649–698: seven FormKind cases. Validator1688–1736 requires evidence/no computation for four source kinds and computation/no evidence for three computed kinds. Form owns an ENTRY, not an assertion/occurrence. A valid computed form can lack eligible associated assertions; a fake DictionaryHit/detail key would misrepresent it.
- lexicon/lib.rs401–408: only SourceReportedForm and AssignedRafsi legacy roles. Lensisku normalization880+ and Sut813 create SourceReportedForm; LLG1132/1156 creates AssignedRafsi. Do not relabel variants/pronunciations/computed material as these old roles.
- Dictionary Full exact967–1135 and Compact exact574–790 use selected source/release/owned-head assertion eligibility, complete totals and format-specific cursor/detail. Reuse retained Full maps/Compact ordinals and actual role rows; do not reload/validate/clone a whole model per query or create Full for Compact. Exact cursor/wire remains unchanged.

These were SOURCE reads, not runtime outcomes. Existing S01/S02 evidence is inherited only at its recorded scope; no new S04 case has executed.

## Concrete semantics

### ReverseDefinition: real assertion occurrences

Require nonempty `contains`. Match ONLY Presence::Value(TextValue).text after ASCII lowercase on both sides. Do not trim, Unicode-fold, strip markup/HTML, tokenize, interpret wildcard syntax, search notes/gloss instead, or compare FTS ranks. Missing/Null definitions do not match. Spaces, punctuation, quotes, NUL and non-ASCII remain literal. Return every selected eligible Lexical assertion under actual source/release/owned-head overlay and requested source/alias controls. Full existing DictionaryHit and its existing assertion-only Compact detail remain truthful for these actual assertions.

### Form: first-class stored records

Exact bytes on FormRecord.spelling; do not reject/trim empty values merely because reverse-definition rejects empty text. Proposed filters are required-null `role:Option<FormRole>` and `kind:Option<FormKind>`, with source/page options. Exact legacy mappings: SourceReportedForm→FormKind::SourceReportedForm; AssignedRafsi→AssignedRafsi; no mapping for the other five kinds. Both filters are conjunctive. Neither filter means all SEVEN ACTUALLY STORED kinds, explicitly a new typed stored-form policy rather than a claim the old corpus had all kinds. Never generate or infer a missing form.

A form match has its actual Form ID/body, complete owning EntryRecord/source, original evidence and optional computation identity. It is NOT an invented assertion, occurrence or DictionaryHit. Search stored forms PRESENT in the prepared model for resolved selectable sources; associated assertion roster independently uses actual selected release/head/Lexical policy. A zero-associated form remains explicit RETAINED STORED RECORD (no currently eligible assertion), not an attested/current-head occurrence merely because its source is selectable. Multiple matching forms remain distinct IDs even when their assertion associations overlap. Computed kinds remain labelled computed stored material with their real computation/no evidence; source kinds preserve evidence/no computation. S06 still owns running computations and generated proposals.

## Identity, response and paging direction

Dedicated Full/Compact response kind/version, direct/configured effective identity and existing SMALL selection summary. Request/result/Form-detail exact fieldsets and named errors must be frozen before issue. Full returns complete typed Form+Entry+Source/evidence/computation provenance. Compact retains kind/form/entry/source/computation/evidence identities and a new FORM-specific detail key resolved through the SAME cached format-local retained model. It must not use the old assertion-only key for a form with no assertion. Existing assertion detail is used only for real associated assertions. No public trusted body/digest setter or generic transport framework.

Counts BEFORE page slicing have explicit units: ReverseDefinition counts assertion occurrences; Form counts stored Form IDs, with separately counted actual associated assertions. Do not call form totals occurrences. Per-source full_count/returned_count/partial uses that result unit; duplicate associations never duplicate form rows and distinct form IDs never collapse. Final order: reverse source/exact entry spelling/occurrence tuple/assertionID; form source/entry spelling/form spelling/kind/formID, with exact kind ordering frozen near issue. Page0/1/several, terminal counts, complete bounded concatenation and same-binding issued-token continuation require pre-input oracles. New query/format/source/alias/effective-generation cursor binding and Form detail invalidation are explicit; old Exact/Resolve/reference tokens retain their contracts. No output-derived body/digest oracle.

Use common borrowed candidate policy with thin retained-provider adapters; hydrate/project only actual selected rows and matched provenance. Full and Compact use their OWN cached engine and existing selected sets/indexes. Pre→entire inner Result→drop readers→hook→NEW post retains error precedence, including setup failures. Final fieldsets/validation order for unknown sources, malformed/foreign tokens and Form body/computation/evidence key mismatch are readiness work, not CC discretion after issue.

## Finite original ownership (prospective)

A. Useful Rust/direct+configured CLI example and literal guide: distinct upstream/owned definitions and first-class source/computed forms, complete pre-input bodies/provenance/counts. Preserve all earlier emitted input authorities/journeys; no stub example.
B. Literal definition ASCII/non-ASCII case, punctuation/markup/NUL/empty/Missing/Null and source/alias/current-history/release selection controls, whole competing records and totals.
C. All seven valid stored FormKind shapes, exact two legacy mappings/conjunctive filters, source-vs-computation provenance, valid zero-associated form, repeated associations versus distinct form counts, complete Form detail/key.
D. Full/Compact/Readable explicit record units/order, bounded input-derived full-page concatenation at0/1/several and terminal counts; same-binding positive cursor/Form detail plus changed query/source/config/head/body-key counterparts. No fake occurrence or source-attestation claim.
E. Actual guard/window controls at declared scopes; reuse accepted shared guards/source-reviewed delegation honestly rather than duplicate a field matrix. Compact route must not hydrate Full or reconstruct the model per operation.
F. Native FILE/Unicode-space/deletedcwd/raw capability/error/Other/BrokenPipe, preserved inputs/config/roots and old query/reference/managed/legacy regressions/documentation.
G. Complete census BEFORE BROAD; frozen common current format/fmtcheck/lint/focused→example+literal→Make/no-default/ONE Synthetic archive/five categories/allfeatures/FILElists/final source-default builds; independent CDC proof, canonical/ROOT close. No evidence claim now.

Prior draft0.1/0.2 Links criterion/control ownership MOVES ENTIRELY to [S05 bounded Links](../slice05-bounded-links/slice-plan.md), including two evidence types, parser compatibility, four Directions, depth/cycle/dedup/order, edge pages/detail/native/admission and associated source selection. Original A06-03 requires BOTH useful increments before parent closure; no obligation is dropped or assigned to an unimplemented stub.

## Provisional destinations and remaining readiness

Potential13paths: search/src/{lib.rs,dictionary.rs,dictionary/projection.rs,workbench.rs,presentation.rs}; NEWsearch/src/workbench/lexical.rs; NEWsearch/tests/workbench_lexical_contract.rs; search/examples/integrated_workbench.rs; search/README.md; cli/src/main.rs; NEWcli/tests/workbench_lexical_contract.rs; docs/integrated-workbench.md; rootREADME.md. Paths under crates/ unless stated. This is sizing only, NOT a current allowance or invented Operator file cap. Lexicon parser extraction belongs S05; no collection_query/legacy store/schema/normalizer/model/manifest/dependency edits intended here.

Before issue: accepted S03 source/API/close; complete actual helper/constructor/guide closure; exact two-variant request, Full/Compact/Readable/Form detail fields/errors/strict nullable rules; first-class source/provenance and association/count policy, token namespace/validation/cursor/ordering; valid portable finite fixtures/pre-input buffers; exact allowance/readset/gates and full initial prompt; ROOT readiness. No source/Cargo/CC intake follows from this draft.

## Version history

- 0.1,2026-11-01,Git7c429165: initial combined lexical design, actual parser/typed-model source basis and first-class identity concerns.
- 0.2,2026-11-01,Git00368855: operational Direction/source scopes and retained zero-associated form labels; UNISSUED, no runtime.
- 0.3,2026-11-01: ROOT-approved split keeps useful ReverseDefinition+stored Form here, moves ALL Links obligations/prior draft Links criterion to S05, computed→S06 and final→S07. Only actual two-family request/results/Form-detail are exposed; prior drafts preserved in Git. Original8arc/14project raw cells, issued S03 and five-assignment rule unchanged.
