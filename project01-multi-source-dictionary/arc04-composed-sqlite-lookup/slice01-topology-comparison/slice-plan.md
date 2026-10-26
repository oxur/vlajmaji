# S01 — finite topology comparison and justified decision

| Field | Current state |
| --- | --- |
| status | CLOSED; source 4b4158c independently accepted; all six rows DONE; Unified selected |
| authority | Duncan: “this looks good! Approved, and engage!” Protocol 1.1 inquiry, effort and numeric limits explicitly approved via root; applicability/accounting in protocol 1.2. Autonomous THROUGH ARC04 ONLY. |
| roles | Requested/configured Sol6.1/high CDC, retained Luna/high CC; backend attestation unavailable. CDC design/planning/independent acceptance; CC source/index/target during issued interval; root read-only. |
| current prompt | [cc-prompt-iteration02.md](cc-prompt-iteration02.md), bounded post-freeze whole-call profiler and README correction completed at 4b4158c; source parent 6fc740e; target returned to CDC |
| source | /Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc04/vlajmaji; codex/project01-composed-sqlite-lookup; starts clean 7b1c565ebf6c8a423e04211b599006b8c21d65d9 |
| target | /Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc01/target; guard /tmp/vlajmaji-p01-a04-cargo SHA 20fe7ba73311aa503870923af09f33f3b53262742ca4eeea2972092924cf5de8 |
| boundaries | No push/PR/publication/A05, main/Bauske/default/accepted checkout/old-store mutation. No correct selected topology means S01/A04-01 stay OPEN and S02 unissued. |

## Useful decision and fixed scope

Select a justified correct physical composition for local provisioning and later configured lookup/lifecycle. Compare unified derived SQLite (U), read-only ATTACH (A), and application fanout (F) against the SAME complete four-pack exact/reference/source/global-page/generation contract and existing FULL representation. This investigation-only increment delivers reusable library prototypes, an executable harness, docs and a justified decision. It is not a production installer or general database survey.

The [protocol](../artifacts/topology-comparison-protocol.md), versions 1.0–1.2, preserves proposal, pending checkpoint and approval. Compact/readable output and its approved targets remain S05; unimplemented compact is neither required nor passed here. No A/F unified metadata sidecar or oracle-supplied result is permitted.

## Source-guided provider and trust design

At 7b, search dictionary.rs:456–487 owns DictionaryStore and typed draft/maps. from_store:519–775 loads the complete draft and builds uniqueness/selection/reference indexes; its only later store call is exact_assertion_ids at 801. exact:794–904 clones availability and hydrates candidates BEFORE paging, including page zero. These shared costs remain measured, not silently optimized. Store dictionary.rs:380–408 provides SQL; minimum metadata/canonical readers are at 512–654. Lexicon pack.rs:388–548 implements keyed composition and ordered maximum-multiplicity extensions. registry.rs:678–810 checks entity/selector identity before later graph validation. The unverified document alias is distinct from validated Dictionary.

### Closed store snapshot API

Add public DictionaryReadTopology with Unified, Attached and Fanout variants, and a plain DictionarySnapshotManifest version 1 containing generation, logical_state_sha256, members, and optional selected_release_ids. Each DictionarySnapshotMember contains id, native PathBuf, expected_state_sha256. Validate/freeze the WHOLE manifest BEFORE per-database reads: nonempty/control-free generation and IDs, unique member IDs, lowercase 64-hex digests, U exactly one member, A/F one through four, no duplicate selected release IDs. Unknown selected IDs fail after load. None preserves original available selection. This is explicit API/probe input, not persistent user config.

DictionaryReadSnapshot::open(topology, manifest) returns Result. A compatibility from_store(DictionaryStore) preserves the old standalone behavior with NO generation/selection overlay. load_document returns an explicitly UnverifiedDictionaryDocument, logical digest and concrete setup durations. exact_assertion_ids returns distinct logical IDs. Use private runtime invariant fields and a typed DictionarySnapshotError distinguishing manifest, unsupported attachment path retaining PathBuf, store/backend/composition and state mismatch. Add DictionaryQueryError::Snapshot/report code snapshot without changing existing categories. Public Result methods need Errors docs.

U owns one minimally opened DictionaryStore. F owns its own vector of minimally opened stores. A owns ONE read-only connection on its first member and attaches remaining members under fixed internal names pack1/pack2/pack3. Use bound mode=ro UTF8 URI paths, URI/read-only flags, query_only and is_readonly assertions for EVERY schema; no immutable=1 and no input-derived identifiers. Canonicalize UTF8 paths and percent-encode URI bytes including percent/question/hash. Invalid native UTF8 ATTACH paths return the typed unsupported error without lossy routing. Preserve U/F native paths. Share schema-qualified minimum metadata/canonical helpers restricted to fixed names, with original main readers delegating unchanged. Pin read transactions, but do not claim cross-file ATTACH atomic publication.

Pinned support: rusqlite 0.40.2 lib.rs:338–349 native Unix OsStr bytes; is_readonly:1074–1077. libsqlite3-sys 0.38.2 sqlite3.c:4170–4298 specifies URI mode=ro/percent decoding and undefined results for invalid decoded UTF8. No new dependency is needed. Iteration01 enables only the existing rusqlite limits feature in crates/store/Cargo.toml for the required safe runtime-limit read.

A/F load and decode THEIR OWN canonical documents, compose them, then serialize/hash the resulting unverified logical document on actual startup. Compare it with manifest logical SHA; charge read/decode/composition/canonical serialization/hash. U checks its expected/logical recorded header state with minimum-open trust. A header reread is NOT authentication; a composed hash does not establish graph validity or native-index agreement. Full verification remains explicit and required on new builds. Do not restore comprehensive startup auditing or forge validated Dictionary.

Cursor generation binding comprises generation, logical state and normalized selected release IDs, independent of member paths/count/physical topology. Expected member states are separately checked. New generation or selection invalidates cursors. Compatibility wrapper takes EXACT old binding branch. Tiny transitions use distinct sealed paths: an old reader retains old generation; a new manifest opens the new one. Unexpected members fail rather than produce mixed success.

### Shared unverified composition

Add pack::compose_unverified_documents(Vec<UnverifiedDictionaryDocument>) -> Result<UnverifiedDictionaryDocument, PackError>. Share the row-merging core with strict compose_dictionaries. Reject unsupported document/registry versions and legacy_archive BEFORE setting output version 1. Strict compose_dictionaries still ends through SourceRegistry::from_document and Dictionary::from_document, preserving validation and canonical bytes.

Within ONE input reject duplicate keyed identities even when equal (entities/bindings, captures, keyed metadata). Across inputs deduplicate equal COMPLETE keyed rows and reject unequal same-ID rows. Alias identity is existing (source,text), not a guessed scope key; unequal bodies conflict. Factor existing registry entity/selector-validation prefix privately for strict and merged-draft reuse; preserve strict error ordering and later full validation. Do not run full snapshot/occurrence/graph audit on the unverified path. Preserve first-seen order, maximum catalogue revision, every category and Missing/Null/Value. Extensions retain ordered maximum multiplicity, including [A,A,B,B], with existing representative-index fix. Distinct capture/occurrence provenance stays distinct.

### Common search and actual timing boundaries

Replace the concrete engine store field with the closed provider; old build/open wrap standalone U and share typed setup/hit/reference/page implementation. Add open_snapshot and open_snapshot_profiled with durations separate from responses. selected_release_ids intersects derived runtime selected-available releases and recomputes reasons/assertion membership WITHOUT editing ReleaseRecord.selected/history; unknown IDs fail.

Deduplicate identical assertion IDs across A/F members BEFORE native_candidate_occurrences: public meaning is distinct logical lexical assertion IDs returned before source/release selection. Distinct received assertion IDs remain distinct. Physical SQL row count can be a separate diagnostic. Reuse global ordering, pre-page totals, reference resolution and cursor rules; each provider performs its own SQL.

Add exact_profiled over the SAME inner exact path. std::time::Instant accumulates actual availability clone, make_hit and returned-hit clone blocks as hydration. Remaining same-call elapsed is conservative lookup/selection/order/paging plus residual/overhead; candidate SQL separately recorded. Do not subtract a separate SQL-only trial. Actual response serialization is timed outside that call; hydration plus serialization is the full budget boundary. Reference whole-call elapsed conservatively includes lookup/resolution construction. Startup includes provider load/composition/hash and common index setup. No response-wire metrics/logger/new dependency. Keep page-zero prehydration and current clone costs in S01; S05 owns compact projection.

## Harness, inputs and finite oracle

Add crates/search/examples/topology_probe.rs with explicit local prepare/run/batch modes. Freeze exact argv/request manifest in execution.md BEFORE use. Prepare takes explicit four pack IDs/release/seal paths/digests, strictly loads sealed packs once and EXCLUSIVELY creates one unified plus four per-pack stores in fresh owned output, emitting stage markers and manifests. Run opens a declared topology/generation/selection, executes request, emits full JSON plus LF on stdout and diagnostics separately. Batch exercises finite controls and emits hashes/summary; bulk payloads remain local. This is a probe, not product config/CLI. Preserve stage timeout and failure evidence; no auto acquisition or default path.

Read-only Full inputs: protected .worktrees/project01-arc03/vlajmaji/crates/data-{lensisku-jbotci,llg,la-lojban,sutysisku}/data/{release.json,seal.json}. Canonical SHA / bytes:

- jbotci: 0987ec84a8434573b5979ea854ca459ce2ea9c7fa03ebac6639574dd2b7aa460 / 215412497
- LLG: d8c8be02b9888c1eee3d3daa053746e483aec995935c4d225234479d4215ce12 / 14433039
- La: e66951094935bd910c6a9ad12fa1bbb51f0578b53d44f22a73056811fe2bc64e / 111187880
- Sut: 67499447426d34c269577eb6947d7c7b22b71090e5a30fd653375819d590ffa2 / 94511884

Full seals/raw pins: Arc03 artifacts/composition-review/preflight01.json. Independent recipes: Arc03 artifacts/four_pack_reference.py, four-pack-reference02.json and composition-review/{native_sql.py,raw_reference.py,distribution_census.py,verify_llg_recipe.py}. Inspect/adapt scratch copies; never execute historical writer paths. Complete typed inventories from accepted S04 support child verification; do not repeat every fingerprint campaign without a defect. Current full per-store native/typed tuple proof is still required; counts alone cannot certify it. All 47 projected tables, exact captures/fields, eligibility and categories must agree; optional contributors need LEFT JOIN.

Cross-checks: composed canonical 435519540 bytes / d9d73cbb49cc5a9ddb0ec16cb6dbb4bb7260ad49f4f8f54c6e6d21f0779b7b19; 78361 assertions, 76722 entries, 76803 lexical/1558 excluded, 31375 eligible spellings. 566367 native fields INCLUDING 23705 Sut key carriers (542662 without keys); 177210 contexts, 11598 forms, 3513 etymologies, 138107 extensions. Three sources, two distributors, 238 contributors, four collections, six components/releases, eight captures. La contributors are a SUBSET of jbotci, not equal full sets.

Farvi: five hits/three groups: jbotci ordinals 13306/13307 native333/69002, La4095/native333, LLG gismu physical357, Sut11309/nativefarvi. Legacy full wire including LF: 7160366 bytes / 144b8b09b6e378243b36c8e01ab06353c3d573459668c8c158a86aebbea73556. Compare Values/canonical encodings with stated formatting; struct key order differs from CLI Value serialization. New paged cursors can bind declared generation; unpaged full response remains identical.

Freeze requests BEFORE Full results: farvi unfiltered and LSK/LLG/SUT; independently verified absent spelling; Sut-only zoizoizoi excluded/query-zero with retained raw/reference positive; pages 0/1/2/3/8 concatenating all positive pages; local/catalogue/native333 release-scoped references plus LLG/Sut/Unknown/Unresolved/wrong-role/missing controls; tiny disabled/uninstalled/corrupt/unsupported/member-mismatch transitions. Every topology independently exercises selection/pages/refs/generation. Production lifecycle recovery stays S02–S04.

## Cheap controls before Full

Mechanically census ALL fixture literals/model categories/source sets/prerequisites BEFORE expected numeric tuples, including LLG keyword columns and null Note contexts. Preserve mistakes. SAME comparator must pass tiny empty/nonempty/reordered categories and reject count/category mutations before Full.

Public controls include within-input duplicates and unequal cross-input source/metadata/capture IDs; self-composition idempotence; interleaved extension maxima; unsupported document/registry/legacy; all presence states; an unverified graph failure rejected by strict validation; alias/shared Lensisku; same-ID copied SQL candidates versus distinct attestations; selection overlay versus immutable flags; stale cursor/generation, pinned old reader/new manifest; UTF8 URI metacharacters and typed unsupported ATTACH native paths; all schemas read-only/no missing-file creation; corruption/state/SQL mismatch typed failures rather than empty success. Profiled/unprofiled values must match. Existing standalone/default/legacy/full JSON stays unchanged.

## Limits, gates and disposition

Protocol 1.2 charges 30 minutes preparation and at most 60 further freeze minutes from retained 2026-10-25 22:06:10 UTC; accounting allocation, not fabricated elapsed measurement. Seven working hours TOTAL including prep; adapter allocation three hours, Full stage 90 minutes, analysis 60 minutes are not reset on handoff. Record actual working checkpoints. Cap exhaustion yields incomplete/inconclusive rows OPEN, never shortcuts.

At most SEVEN TOTAL Full stores under protocol 1.5: retain the two early historical/unaccepted stores, then one clean U plus the SAME clean four shared A/F after prerequisites. At most 12 GiB new data/scratch INCLUDING the retained two, 600 seconds per create/verify. No copying old/default stores, no duplicate cache. CC runs ONE timing round U/A/F at stable source; CDC reserves A/F/U and F/U/A on SAME frozen binaries/stores. Maximum three balanced rounds. Independent tiny stores are separate. Record actual Rust/SQLite/options/runtime attachment limit, source/input/binary/profile, raw wall/RSS/disk/connections/bytes/exit; unavailable evidence stays unresolved. No cache purge/disk-cold claims.

Every Cargo/Make: login:false, explicit ENV CARGO_TARGET_DIR at retained target, CARGO_NET_OFFLINE=true, fixed A04 guard +1.98.0, locked/offline after initial offline lock creation. Target assignments must not leak through command-line MAKEFLAGS. One writer/scanner. Unique outside package AND binary names; scratch Python -B only for orchestration/reference, no shipped runtime requirement.

Current full durable logs: make format → lint → check, actual populations; affected store/search/lexicon public/no-default tests, clean all-features with explicit four-Synthetic preparation as CI, relevant package/offline caller proof. No repeated unrelated baseline/import. Scoped source commits/both trailers, no dependency/lock/DDL changes. End writer sessions and freeze clean candidate before CDC reproduction. CC is proposed-done, not acceptance. CDC requires all six rows, scope walk, independently reproduced decision and retrospective; no close records precreated.

## History

- 0.1 at planning45eaa07: honest DRAFT/HOLD with budget and provider/trust/generation design incomplete, no execution.
- 1.0, 2026-10-25: exact approval in protocol1.2 and source-guided recipe resolved; initial assignment issued. No result inferred. First planning-write setup attempt failed Python syntax before any file write; corrected direct scoped write, no product execution.

- 1.1, 2026-10-25: issued cc-prompt-iteration01.md adds only crates/store/Cargo.toml as fifteenth path and existing rusqlite limits feature for actual read-only runtime-limit observation. Initial prompt preserved bytewise. Unissued direction01 global-feature proposal is superseded; no source edit under it. No cap/reset/new store or trial.

- 1.2, 2026-10-25: protocol1.3 records actual pre-create comparator violation, two retained completed stores and specific Operator exception gate. Current CC interval continues only bounded comparator/tiny resume/gates and allowed read-only validation; no remaining Full create/query/timing until explicit response. Initial issued prompts preserved, no cap/reset/acceptance.

- 1.3, 2026-10-25: protocol1.4 supersedes interim reuse advice. Current recommendation is clean five-store rerun with retained two historical stores, proposed total cap seven; NOT approved. No bespoke resume/extra old-store Full validation now. Tiny-tested RSS/timeout wrapper and exact resource/time proposal precede root’s one Operator choice.

## Version 1.4 — count answer and remaining execution gate

Duncan explicitly approved seven TOTAL candidate stores: retain the two historical/unaccepted stores and create five clean comparison stores only after the remaining prerequisites. Protocol 1.5 records the exact answer and timestamp semantics. Same seven working hours including preparation, 12 GiB including the retained two, 600-second bounds and three balanced rounds; no reset. COUNT gate cleared only. Full operations remain HOLD until CDC reviews the complete comparator/prefix/source/gates, batch/reference/page and Value-serialization harness, frozen category config, binaries and expanded argv. No topology decision or acceptance is implied; all rows remain OPEN.

## Version 1.5 — whole-call profiling correction before Full observations

Root identified that frozen 6fc740e assigns exact total inside the inner function, before its large locals are dropped. Iteration02 moves the whole-call timer/residual to the outer profiled API after inner return and clarifies U header trust versus A/F computed digest in the search README. No Full store has been created under the amended clean plan; no timing round ran. Prior 18+40 independent semantic controls and actual tiny prepare/run/batch remain valid at their stated scope, not proof of complete-call timing. Full source-input census and pins are unchanged; corrected binary and read-only tiny parity are prerequisites before Full release. All rows remain OPEN; no count/time reset.

## Current prerequisite release — 2026-10-26

See artifacts/cdc-prerequisite02/receipt.md and manifest at planning 158a3cb. Current source/binary, corrected 54-request graph, actual tiny process controls, timing wrappers and stage parsing are qualified before clean Full execution. No Full native result or comparison round is yet claimed. Protocol 1.6 supersedes the preceding historical readiness HOLD without resetting any budget or attempt.

## S01 accepted endpoint — 2026-10-26 UTC

See closing-report.md, cdc-verification.md, cdc-retrospective.md and artifacts/topology-decision.md. Three rounds/eighteen processes and all hard correctness/numeric budgets passed; ancillary cache-delta/auxiliary-FD unknowns remain explicit. Root approved canonical consistency before mechanical close. S02 now owns selected Unified provisioning/catalogue design; no S02 prompt or actual default activation is issued by this close.
