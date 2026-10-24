# CDC verification — Slice02

## Candidate07b96b4 — NEEDS REFINEMENT

Frozen source07b96b4fad76711d4e3cfd5c556cb90bd8545fe2, clean; initial prompt7e730d9 + refinement01/4225a97. CDC read the actual1344-line library,326-line contract suite, public example/consumers, manifests/reports/attempts and scoped workspace diff. All6ledger rows remain OPEN. This is independent executed review, not intake approval.

Fresh evidence in artifacts/cdc-review-07b96b4: gates.json + full logs reproduce Rust1.98.0 fmt/check/clippy/test/doc (64unit/integration+5doctests), E03manifest, E02(103/103), archive; all exit0. Isolated build target /tmp/p02a02-cdc-lexicon-target. Full-archive byte check independently compared all16 stored source bodies/hash identities with E02 inputs:16/16 exact. Protected archive/bibliography/P01 crates/E02/S01 source diff vs49e1d97 is empty. Pinned places body matched clean Git object3d9fd41. Separate public jbotci consumer rebuilt/run under explicit1.98 with preserved lock: exit0 in14.724s, same direct-source errors,2farvi records/optional1drop, metadata collapse/form projection and4/5computed markers/alias/unmapped outputs. No whole-app/model/production inference.

Current stable became1.99.1 during CC's recorded upstream stable-toolchain synchronization; CC's actual1.99 evidence is retained. CDC added explicit1.98.0 minimal+clippy/rustfmt without changing default, then used +1.98.0. rustup automatically reported manager self-update1.29.0→1.29.2 during installation; no source/default selection changed. Successful install/component tool receipts in task history; actual compiler versions in fresh logs. This resolves MSRV evidence, not other findings.

## Concrete findings and disposition

F01 — source-link parser: public import of notes "See also $x_{1}$ and {target}." returns targets1 and target; "Foresee also {target}." also creates a link. Same-clause TeX and marker boundaries violate the bounded explicit-link contract. Existing after-period control passed but did not test these cases. Other boundary controls (semicolon/outside clause) passed. Parent supplied TeX lead; CDC independently reproduced.

F02 — native text/definition views: actual cpana fixed CVC/CCV/CVV blank slots return Missing; known source blanks must remain distinguishable from absent fields. LLG text row "fav farvi null" reports gloss as JSON Null and definition=Some("null"), although this format supplies a form-row gloss, not a dictionary definition. Raw lines/bytes are intact, so this is a derived-view defect, not lost acquisitions. Make field presence format-aware, preserve empty fixed slots and distinguish gloss from definition. Counts must continue separating1545forms-only rows from19696definition-bearing source rows (not unique meanings).

F03 — proposal invariants/identity: public serde Deserialize creates Proposal namespace captured:fake/idforged; add_proposal accepts it but its own archive loader rejects it. Private constructor checks are bypassable through a public trait. Also two valid constructor tuples ("a\0b","c","d") and ("a","b\0c","d") yield the same ID due to ambiguous NUL concatenation. Use validated boundary and unambiguous tuple encoding; do not merge/distort proposals.

F04 — report/replay reconciliation: frozen summary/source code has1988source-reported assertions but source report still1417; closing row says16Lensisku farvi rows although13API/16all-source is actual. Evidence's "manifest indexes itself" is inaccurate (30evidence+9candidate+manifest itself =40paths, not self-hash). "resumed only after CDC confirmed corrected extents" must distinguish S01 correction from S02's own receipt/no new approval gate. Provide exact source-working-dir example, external-consumer and pinned jbotci reconstruction/replay commands; existing source report links are insufficient as an executable recipe. La corpus metadata should carry the known Git source URI/pin from accepted E02 rather than only a local payload label/hash; don't invent unknown actual request/redirect precision. These are current evidence corrections; prior candidate/failed attempts remain.

Counterexamples.log and preserved counterexample source/lock demonstrate F01–03 through public APIs. No source changes by CDC.

| Row | Independent disposition at07b96b4 |
| --- | --- |
| S02-01 | Runtime/source reuse result VERIFIED within its stated package-only scope; retain with unchanged jbotci inputs/code/lock guard. |
| S02-02 | Population/hash/raw-byte census VERIFIED; typed meaning/form view needs F02. |
| S02-03 | Core JSON/archive negatives pass; F02presence and F03public invariant/ID defects require repair. |
| S02-04 | Exact/form/proposal separation positive cases pass; F01/F02/F03 require repair. |
| S02-05 | Public API compiles on1.98; F02/F03 and F04replay/metadata need repair before acceptance. |
| S02-06 | Fresh gates/preservation pass; F04 current-record reconciliation/replay needed. |

Next preserved assignment cc-prompt-iteration02.md;2of5 refinements, same scope. Retain successful unchanged jbotci/acquisition evidence; rerun changed local library/contracts/example/archive/consumer/derived manifests and actual1.98 gates. No operator decision or acceptance yet.


## Final gate — ACCEPTED at cef3727

CDC independently accepts all six rows at source `cef37271414d02044c6cf5c0251aae19aecca114`. Source is clean. The substantive repaired source `389c0a9` was actually read and reproduced; the final commit changes only replay README and its manifest. Three final shell blocks pass syntax checks, use the actual int19h repository and fresh output paths, and preserve frozen logs. No additional source defect remains. Initial findings and all three numbered refinements remain historical evidence.

Fresh explicit Rust1.98.0 gates in [gates.json](artifacts/cdc-review-389c0a9/gates.json) pass fmt/check/Clippy/tests/rustdoc: **66 unit/integration +5 doctests =71**. E02 verifies103/103; original archive verifies. Independent corpus example and outside-workspace consumer import21241 records/16 snapshots; all16 original source bodies are byte-identical after reload. Population:19696 definition-bearing rows plus1545 forms-only gloss rows, not unique meanings. farvi16 all-source records include13 API records across languages; si'e2/sie0. Assigned3091/source-reported1988 form assertions and13521 bounded literal links remain distinct views.

[Changed-case controls](artifacts/cdc-review-389c0a9/counterexamples.log) now pass same-clause TeX, marker/clause boundaries, native text null/blank slots, gloss separation and proposal tuple identity. Expected compile-negative proves Proposal no longer exposes unchecked public Deserialize. The actual preserved v1 external archive was reconstructed byte-identically (SHA256 ed45bfc90520c482e5467782ccd2f8eb9f74b2f1aea2809ef061dcea40cc829c); its bounded read migration produces exactly the current v2 consumer archive bytes. This is tested compatibility for one unaccepted prior candidate, not a general migration commitment.

Example archive:28320417 bytes, SHA2564e5c30e978bde631349e1107599f3ef9738c1f63257ba280e146d2256c7cef5a. External consumer archive:28319503 bytes, SHA25667d5199acea5cc78146df0adff85af1c43d901a01b7dfad1c2079abde8197da9. Different explicit proposal inputs explain different archive identities. See full-archive.json, external-consumer.log, v1-reconstruction.json and legacy-check.log in the same review directory.

| Row | Final independent disposition |
| --- | --- |
| S02-01 | ACCEPTED: actual pinned public jbotci package runtime independently rebuilt/run on1.98 at07b96b4; final raw/probe/lock unchanged guard retains this result. Import normalization, optional-field collapse and computed-place limits are explicit; no production adoption. |
| S02-02 | ACCEPTED: complete population/identities/languages/locators and all raw bytes independently reconcile. No source/definition winner or silent drop. |
| S02-03 | ACCEPTED: same-import JSON/type/UTF8/hash/atomic/idempotent controls plus repaired format-aware presence and exact archive/proposal validation pass. |
| S02-04 | ACCEPTED: assigned/reported roles, exact phrase/apostrophe spelling, source-spanned bounded links and separate proposals pass positive and adversarial controls. No inferred semantics/slots. |
| S02-05 | ACCEPTED: real public outside-workspace consumer and example pass on1.98; metadata includes known pinned La source URI; replay is executable and preserves frozen evidence. |
| S02-06 | ACCEPTED: final guards and inventory pass; protected archive/bibliography/P01/E02/S01 source diff is empty.71-test census reconciles. |

[Final guards](artifacts/cdc-review-389c0a9/final-guards.json) retain successful unchanged code and upstream evidence rather than rebuild it for documentation-only corrections. Inventory59 evidence+10 candidate entries is not69 distinct files and does not self-hash its manifest. Final source/code/data terms remain separate; no standard conformance, semantic truth or production readiness inferred.

Recording deviations: CC committed only its delegated evidence/closing paths in planning0b3d33d despite the no-planning-commit instruction. History is preserved; it changed no ledger/acceptance. CDC retained ownership of all dispositions and reasserted the boundary. The S02 receipt's earlier CDC-confirmation wording referred to S01's correction, not a new S02 approval gate; S02's own completed intake preceded source work. No operator decision was required.
