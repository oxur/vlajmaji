# S03 intake record — integrated reference admission

**State:** intake/readback only; no implementation, product execution, or proposed completion. All S03, Arc06, and P01 acceptance rows remain OPEN. Source, index, and the retained target remain ROOT-QUIET pending CDC acceptance, fresh pins, and an explicit transfer.

## Authority and baseline

Read the complete issued S03 prompt (29,032 bytes, SHA-256 `90364a84442df4176249c6d1c987953c07d2c64625c012c294466612822a512d`), active plan (35,281 bytes, `648f559bee002d45e736924061d46d837c13b45053b0cc567f472de1818fade7`), and ledger (3,593 bytes, `c0d56df7496228c2ed78725a9a42b0f52f6bb098284ec0171cabb1bc87aef708`). Also read directive 05 and the S03 manifest’s required Arc06/P01 planning sections, status, required data queries, and source readset. ROOT readiness was PASS for the reviewed draft; the issued planning revision is `c27d1f1239761d4b7c26bdf0257fc878dea3d876`. No planning text beyond this record was edited.

The source baseline is `/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc04/vlajmaji`, branch `codex/project01-integrated-workbench`, HEAD `8f373fa0298694b03d88f735a2ff11f1d1b9ca41`, tree `8f0d61298a061410c8a133df2107b73b4de4313e`, parent `4fc937dcbebd0e2f3171f495345d6ab2a6e29f62`; read-only Git inspection used `--no-optional-locks`. The tree was clean. The whole source `AGENTS.md` (8,404 bytes, SHA-256 `3775aeaad14a5c1eca7edc52d061f39130b1f24ba9fd6dc9f4f537a7282914d7`) was read. Rust toolchain policy is 1.98.0, edition 2024. No target, Cargo, Make, product, or runtime command was launched.

## Required source and guidance readback

All 28 identity-pinned paths in the issued manifest were checked against the source tree. Consequential source text was read, not inferred from hashes. This included:

- Store workbench DTOs, error and native-path boundaries, `WorkbenchView` open/currentness and catalogue dependency guard; configured profile implementation and identity/writer contracts.
- Catalogue `metadata_roster`, schema open/check-family, reference projection, query and compact projection. The roster uses one read-only connection and an already-deferred transaction, checks the family without starting a nested transaction, reads registration metadata in SQL order, computes logical installation state without payload-file inspection, and closes its statement/transaction. It is a metadata-only observation. Full hydration and payload verification belong to the actual reference query.
- Search workbench freshness, lazy Full/Compact/detail routes, borrowed dictionary indexes, Full reference resolution, Compact projection/resolution, and compact detail keys.
- The lexicon dictionary, registry, pack, owned-journal reader, relevant collection-query contracts, CLI profile/reference dispatch and shared output path, compact renderer, integrated-workbench example, its guide, relevant README sections, Make targets, and the complete CI workflow.
- Applicable project/Arc guidance and project-management, work-verification, testing, Rust-pattern, and CLI-pattern sections, including AP18/19/33, OB05/06/13/15, EH04/13/14, API20, and CLI05/06/10/21/26/27.

The planned reference endpoint is additive. It must preserve the existing managed Resolve and S01 routes and their wire formats. A reference request’s typed role-row digest is SHA-256 over that exact typed role row’s serialized bytes; it is distinct from a descriptor digest, source JSON digest, or query result. The descriptor and typed-row digest must agree before admission. Missing or malformed digests are integrity failures.

Full and Compact intentionally retain their different matching semantics. Full catalog matching makes an internal native-cleared metadata candidate, while retaining source, release, and occurrence attribution, then applies the original request’s native matching. A missing requested native value is the existing wildcard behavior; an entirely absent target identity is invalid. Compact uses the true-target RawANY metadata projection before its strict native check. No common rewrite should erase this distinction.

Status is also format-specific: Full reports Resolved when confirmed attribution is nonempty before considering unverified rows; Compact reports Unverified if any matching attribution is unverified. Unknown and nonempty Unresolved targets remain typed outcomes; a missing local role is invalid. The owned-only positive must be witnessed by an actual owned dictionary row, not an invented catalogue registration. Existing model validation rejects duplicate occurrence bindings; a raw manipulated descriptor is only a seam-level malformed-input observation, not a valid admitted-model duplicate.

The bounded metadata route retains logical observations per registration, including older matching registrations, and chooses the top-level reason from the first matching registration. It must not enumerate payload files or hydrate models. Both the route’s reader and the caller’s catalogue handle must be dropped before a real writer runs. Fresh post-write observation checks whether the old result is still current. S02 binding freshness is byte-exact; the Compact detail key is bound to the same cached effective query core, while outer binding identity remains distinct from the inner snapshot identity. Readable output remains the existing Compact renderer, with its source grouping and per-source incoming member order; Full, Compact, and Readable are not interchangeable projections.

The CLI boundary is a native FILE path, including a literal filename `-`; it is not stdin or JSON argv. Preserve the existing three-field error object and the established `response_output_io` behavior, including broken-pipe policy. A no-op must not be described as a publication. No new schema, model, PackId, dependency, cache, benchmark, or path is authorized.

## Structured input query

Read and parsed the S02 `bindings.json` map and all referenced JSON files. The original 11 inputs and the three `additional_fixed_inputs` (alpha selection, beta selection, expected profiles) all exist, are valid JSON, and match their recorded byte lengths and SHA-256 values. The old alpha selection equals the original selection byte-for-byte. The upstream dictionary and upstream release inputs are the same 4,469-byte buffer. The seven S03 inputs in `artifacts/reference-inputs/` all exist, parse as JSON, and match their recorded bindings. These are pre-input authorities, not product outputs.

| Input | Bytes | SHA-256 |
|---|---:|---|
| `expected.json` | 1093 | `91f63128c7801de531ddb014857cedbaa1536ab4af49a52f934405fd946a0765` |
| `historical-selection.json` | 133 | `7f40fbc8fbcab37660ec8e3498fe38a848316f54c140656c73e942f497617b85` |
| `owned-r1.json` | 5652 | `372fcce44cf1dd1f0d2d37b8b42927a25ac227f3d6f1e846856d8097896e4286` |
| `owned-r2.json` | 8282 | `b0a46586e5bba2817593f7cb0a5bdfb9a9897391fa1f4383ec4dd1b1217c69c7` |
| `owned-seed.json` | 2062 | `6d7e6c61892e1f5027e675ac8b32441044460cf1ee0359c179729196b087e0e8` |
| `request.json` | 163 | `e868dd2d63d21ca68e2d4ec6bf157993af269aaa3fe82c1d4ba89ac08af3470b` |
| `selection.json` | 107 | `edad0414598f2be7ed74bef8d611c3fd05dfacd215ff91a5909a2021f32796e5` |
| `upstream-dictionary.json` | 4469 | `9bb0f1071ca9152874bd43b3706f2c4a02964bdb69c3960e113ee6645d880279` |
| `upstream-release.json` | 4469 | `9bb0f1071ca9152874bd43b3706f2c4a02964bdb69c3960e113ee6645d880279` |
| `upstream-seal.json` | 474 | `0d43c55ce524c4c3d8d4f7f422dca7b0d4b0a4330b1e332fc12a83b2584dd9e9` |
| `upstream.json` | 83 | `06220fcd82ff067293787c5057b3a68d6c63400d80e29d02fe5a4a64be015db5` |
| `profile-alpha-selection.json` | 107 | `edad0414598f2be7ed74bef8d611c3fd05dfacd215ff91a5909a2021f32796e5` |
| `profile-beta-selection.json` | 67 | `beeda00574bd9e1642c0f6a8b1771e9bc6fa836b4db601e2f414148579f1fe37` |
| `profiles-expected.json` | 214 | `fca09ccd0d28cc2f12e5f1aeecdf61b2862a45f967d6fb39e62361e3a7940bb9` |

| S03 reference input | Bytes | SHA-256 |
|---|---:|---|
| `reference-disabled-selection.json` | 89 | `cc7756a611566ef3e82bb61f3e562cd3145ac6962937a4353e6c0b78eca014c2` |
| `reference-expected.json` | 529 | `43687f3379aa83c419d5915641992debd80d5ab43b663ad66208fc6fe35bc714` |
| `reference-owned-assertion.json` | 43 | `8538c3fe9250d7236b67184edd76ad3ab54e4b8664b9818d9923b4f805a70acf` |
| `reference-owned-entry.json` | 39 | `ea3efd655902cb677499047d2af35fa1fc8aad6c13d2a4ab876967732d0b5893` |
| `reference-unknown.json` | 9 | `14c4849191d018fc8b00b02dc1e49d9f8b36a152066023b3b4c609b2f62ba328` |
| `reference-unresolved.json` | 40 | `74bbd4c69ea0e90d3b71b4dbd6158f1018a56c85b3da7d956a41508a39be0fdc` |
| `reference-upstream.json` | 340 | `7434b3aa7c3416f597d9567e2783e6d2553beeefb94dfb96e7c22cbbfe378d70` |

The expected authority declares owner `owned-example`, owned r1 assertions `owned-assertion-a/b`, an upstream lensisku assertion tied to the emitted upstream release, a disabled catalogue-only selection, Unknown and Unresolved targets, and configured profile revision/sequence expectations. The upstream target carries a raw-native locator `/0`; the owned targets are Local assertion/entry references. The disabled-selection authority is version 1 and selects/disables `la-lojban`, with an owned head and no contexts. These values were queried from the inputs, not inferred from a future result.

## S02 carry-forward and limits

S02 is closed in its own records. Its actual independent33 consisted of one compiled public example plus 32 observed CLI children; it did not independently rebuild the Rust consumer or replay private hooks. The S02 binding’s three `additional_fixed_inputs` are included above. S02’s earlier toolhome auto-install deviation remains historical and grants no S03 acquisition authority. Retain the original environment file, pinned toolhomes and PATH, guard, offline/locked constraints, and `RUSTUP_AUTO_INSTALL=0` if and only if a later explicit S03 transfer authorizes commands.

No implementation choice or backend/model attestation is made here. No source/target custody transfer has occurred. I found no remaining consequential required read or source-contract contradiction; this record is ready for CDC’s intake review and fresh pin check.
## CDC intake clarification — 2026-11-01

This append clarifies wire strictness, reference-admission ordering, and the actual structured population projections read above. It does not change the issued prompt or claim implementation.

### Wire and admission rules made explicit

Every nullable field on the new DTO wire is still a required key whose value may be `null`. Plain derived `Option<T>` deserialization would incorrectly treat an omitted key as `None`; use the existing required-option deserialization pattern. Reject missing nullable keys, duplicate keys, and unknown keys at the new DTO boundary.

For a requested Catalog reference, inspect **every retained row matching source, release, role, requested identifier, and occurrence**, including older registrations, before deciding whether a row belongs to a currently admitted member. Each candidate must pass the descriptor plus valid typed-role-row SHA-256 comparison. If matching retained rows disagree, return Ambiguous; do not let current-member admission hide an older conflict. This is the raw-retained registration boundary, not permission to admit a release that is not currently a selected enabled member.

The metadata native wildcard is specifically an actual `native_entry_id = None` or `native_assertion_id = None` on the metadata row when the request expects `Some(...)`; it is not a request with a missing native target. After this metadata candidate step, the lower payload attribution index must be unique for the requested tuple. Do not add comprehensive full-model validation per reference: preserve the existing format-local lazy index/setup. Full build/verification remains a separate operation.

### Structured population readout

The complete S02 expected fixture authority was read from `slice02-configured-refresh/artifacts/independent-proof/inputs/expected.json` (1,093 bytes). It fixes spelling `farvi`, request `Exact` with `ActiveOnly`, no source restriction, no page-size or cursor, and source order `lensisku`, then `owned-example`. Its per-source `definitions_by_source_r1` is lensisku `[advance]`, owned-example `[grow, develop]`; r2 adds owned-example `[flourish]`. Thus its input-derived populations are alpha/r1: three occurrences in two source groups; beta/r1: two in one; beta/r2: three in one. The separate `profiles-expected.json` (214 bytes) fixes config revisions `[1,2,3,4,5,6]` and prepared sequences `[0,1,2]`; these are expected values, not observed outputs.

The upstream raw input is one JSON array row: `farvi`, word type `gismu`, definition `advance`, definition id `1`. Its normalized 4,469-byte dictionary/release authority has source `lensisku` (Upstream), one entry and one assertion whose definition is `advance` with language `en`, one selected Available release `la-lojban-release-116b8d1761763fa4341642a0f2ede5b45e508ac43b6ad79688bc5694e92ed438`, one capture `captured:lensisku:la-lojban:en` with raw SHA `06220fcd82ff067293787c5057b3a68d6c63400d80e29d02fe5a4a64be015db5`, and one occurrence `/0`. Its original typed role reference is Catalog source lensisku/release above, `native_entry_id: null`, `native_assertion_id: "1"`, and that capture/locator. The seal authority binds the same release bytes, source/release IDs, and capture. The selected enabled upstream row is therefore both metadata and payload evidence; the disabled authority separately fixes the same pack in `disabled`.

The owned seed authority is an explicit version-1 empty dictionary: source `owned-example` (OwnedDraft), selected Available `owned-release-r0` with no captures/assertions, no entries or assertions, and owned head `owned-r0` at the empty root revision. The r1 authority retains the same source and adds one entry `owned-entry-farvi` with spelling `farvi`; assertions `owned-assertion-a` (`grow`) and `owned-assertion-b` (`develop`); one selected Available `owned-release-r1` containing those two assertions and capture `owned-example-capture` with the pre-input raw digest, locators `/a` and `/b`; and head `owned-r1`, parent `owned-r0`, active list `[a,b]`, no tombstones. The r2 authority preserves those bodies and adds assertion `owned-assertion-c` (`flourish`), release `owned-release-r2`, and a distinct `owned-example-capture-r2` at locator `/c`; head `owned-r2`, parent `owned-r1`, active list `[a,b,c]`. Both owned captures and their metadata/raw occurrence rows were read from the complete typed input JSON, not inferred from expected counts.

The seven S03 authorities were read as complete JSON projections. They fix: an owned Local Assertion reference `owned-assertion-a`; an owned Local Entry reference `owned-entry-farvi`; the above upstream Catalog tuple; `Unknown`; `Unresolved("future editorial target")`; a version-1 disabled selection of `la-lojban` with `owned: "head"` and no contexts; and the expected resolution/status/profile oracle. That oracle names the two r1 owned assertions and upstream assertion ID, expects Resolved for both admitted references, CatalogueOnly/`disabled` for the disabled route, Unknown and Unresolved for their respective target classes, and the revision/sequence values above. These are fixed pre-query authorities; no runtime claim is attached to them.

### Project denominator

The P01 project ledger contains 14 status rows. The actual row query found all 14 OPEN. This read was a status count only; it does not close an Arc, slice, or project criterion.

No source, index, target, or runtime operation occurred during this clarification.
### CDC ledger-cell coverage clarification — 2026-11-01

I have now read the required whole-cell project ledger projection, not only queried statuses: `project01-multi-source-dictionary/ledger.md`, lines 29–42, 34,496 bytes, SHA-256 `1b1022573b7b8d7ee2414121a652be230b898de9f2f9b19dcf6de3850543b674`, at planning revision `c27d1f1239761d4b7c26bdf0257fc878dea3d876`. The 14 Markdown data rows contain the seven declared cells per row: row ID, criterion, verifier, grade/significance, owner, status, and Evidence/Notes. I read all 14 complete rows (P01-01 through P01-14), including the full Evidence/Notes text; every status cell is `open` (reported as OPEN). The preceding “status count only” phrase described the earlier denominator query and did not describe this full-row read. No criterion, verifier, owner, Evidence, or Notes cell was edited or summarized as a substitute for that read.
## CDC intake acceptance and conditional custody boundary —2026-11-01 06:18:46 UTC

CDC read the complete 10,235-byte original intake and both bounded appends (final CC body16,638B/SHA b71783ca30106f283fb51f9c7faf58d1aafdba707b3dd14a908de0d7df5a14c9). The readback is coherent on required-null serde keys, typed role-row SHA versus descriptor, ALL matching old/current registration comparisons BEFORE admission, Full native-cleared internal probe versus original target, metadata FIRST/RawANY versus lower UNIQUE indexes/status priority, owned-only actual role, logical observations/first-match reason, and dropped-reader successful writer/postcheck sequencing. Coverage is CC attestation plus recorded actual projections; it is not independent implementation acceptance. No consequential read or design contradiction remains.

Small recordkeeping erratum: CC's final paragraph says seven project columns and combines Evidence/Notes. The actual header has EIGHT separate cells: ID, Criterion, Verify, Significance, Origin, Status, Evidence, Notes. CDC inspected the complete14 raw rows and header directly; CC attests loading those whole rows. The seven-column label is inaccurate, not a missing criterion or permission to merge/drop Notes. Original paragraph is preserved, and every raw project/arc/child cell remains unchanged.

Fresh CDC read-only checks: source clean8f373fa0298694b03d88f735a2ff11f1d1b9ca41/tree8f0d61298a061410c8a133df2107b73b4de4313e/parent4fc937dcbebd0e2f3171f495345d6ab2a6e29f62 on codex/project01-integrated-workbench; ALL38 issued identity rows match actual whole files (28source+3S02close+7fixedbuffers), original14 inputs match BOTH frozen binding maps, three NEW allowed files are absent. Guard330B/426d1e49 and originalenv2287B/ae749d0d match; literal1917B PATH/fixed toolhomes retained, RUSTUP_AUTO_INSTALL=0 required. Protected maincb13d377/oldowned69a9f685/Bauskec5801ae1-treee5715eb8 unchanged; Bauske still has ONLY its two known untracked README autosaves. No source/index/target/runtime action occurred during intake.

EFFECTIVE ONLY upon CDC's explicit message AFTER this named planning checkpoint: retained Luna/high CC owns exactly the THIRTEEN source paths in issued plan§7, the source index and ONE Arc01 target for S03 INITIAL/pass1 of5. ROOT/CDC remain source/index/target READONLY until explicit frozen handback; CDC owns planning except CC's authorised execution append and actual proposed closing report. Preserve literal originalPATH/fixed CARGO_HOME/RUSTUP_HOME before fresh ONLY HOME/XDG_DATA_HOME/XDG_CONFIG_HOME/TMPDIR, auto-install0, physical SOURCEcwd, a05guard/+1.98/lockedoffline and target ENV ONLY. Complete originalA–G before BROAD gates; early coherent compiler/focused feedback remains authorised. No old assignment may resume. No child/arc/project acceptance, source build or product command is implied by this intake checkpoint.
