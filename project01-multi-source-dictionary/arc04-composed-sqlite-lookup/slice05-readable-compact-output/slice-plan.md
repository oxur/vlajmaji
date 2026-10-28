# S05 — Readable and compact output

| Field | Value |
| --- | --- |
| Status / version | READABLE SEMANTIC/API/finite Tiny PASS; final test-only04 ISSUED /1.10 /2026-10-28 UTC after root final readiness PASS. Frozen5e strictF03 closed; all6OPEN/noFullnumericLinuxfreshARC acceptance. |
| Authority | Duncan's Autonomous THROUGH ARC04 ONLY, explicit next-slice continuation and separate S05 branch direction; Sol6.1/high CDC, retained Luna/high CC. Root reviews this concrete packet before issue. |
| Source / dependency | Clean5e19754ce7ccad0ddea788c2e344df116711814c, parent7399563/tree4c91c02a9d66d58ec7e26c9323289f40a730ccde on codex/project01-readable-compact-output; main32e99 base/accepted432 and old S04 local/origin432 frozen. |
| Ownership | Complete final04 intake/readback accepted at verifiedclean5e. AFTER scoped metadata commit and explicitCDCdispatch, retainedLuna/high CC owns ONLY THREEtestpaths/index/ONEtarget; CDC/root then read-only until frozenhandback. CDC owns planning; CC appends execution only. |
| Delivery | Additive compact Exact/Resolve and complete occurrence detail, then source-grouped readable Exact/Resolve/detail. Full default JSON stays compatible. Public libraries, CLI, documentation and actual five-workflow composition accompany output, not verification alone. |
| Sizing | FIVE TOTAL: initial/pass1,01/pass2,02/pass3,03/pass4 returned;04 ISSUED finalpass5 exactTHREEtestpaths. No automatic sixth/reset; unchanged six criteria/all later gates. |
| Limits | No dependency/lock/schema/importer/model/workflow changes, topology redo, new Full store/generation, cleanup/cache reset, actual-default access, S04 branch writes, A05 activation, network/push/PR/merge/publication. Later finite retained-G4 trial needs exact executable root readiness. |

## 1. Source findings and preserved behavior

At432, `dictionary.rs::from_provider` clones complete release availability, entry/assertion rows and detail group maps. `exact_inner` clones availability and hydrates every admitted hit before stable sorting and slicing; even page0 pays that cost. `ReferenceAssertionView` omits DictionaryHit contexts/forms/frames/editorial/computations/etc. `collection_query.rs` lazily opens its full engine after managed cursor validation, but retained reference lookup reads whole descriptors. CLI `execute` returns Value and output context; formatting that full Value would not satisfy projection.

Historical immutable8b analysis was CDC's; root accepted its implications then. Root's later NEW current-e4 source-only reads independently corroborated those boundaries and owned-head selection. e4 production equals432. These are source observations, not measured performance or new experiments. Initial proof consumes existing APIs/fixtures and approved representation work; no new investigation is commissioned.

Keep existing full constructors, DictionaryRequest/ExactOptions wire and construction, full response fields/bytes, source aliases, minimal unverified ordinary open, SQL candidate count, complete eager-hit error behavior and three-key stable order/cursor. No All, canonical full-model digest, owned-pack hash audit or comprehensive registry validation returns to ordinary startup. Existing standalone/legacy optional STORE/default resolution, all Q01–Q06 and profile/catalogue/lifecycle behavior survive. No per-profile materialization.

Valid canonical models have exactly one assertion per loaded OccurrenceRef (`lexicon/dictionary.rs1523–1583`); typed-unverified open can expose malformed alternatives. Full keeps BTreeSet assertion-ID iteration followed by stable `(source, spelling, occurrence tuple)` sort. Compact declares an explicit four-key policy `(source ID, spelling value-or-empty, tuple_hex occurrence, assertion ID)`; on valid data selection/order matches full, while unverified ties have deterministic distinct positions. This is a NEW compact codec, not a silent full-order change.

## 2. Two useful passes and exact source scope

Initial: a caller can choose compact before provider setup, Exact/Resolve in compact JSON, follow a stable detail reference to the complete DictionaryHit, and use all through CLI, with current tests/docs. Readable is deliberately unavailable until numbered01; do not advertise/accept a format whose renderer is unfinished. Initial contribution to all six rows is recorded; all remain OPEN.

Initial exact NINETEEN allowed paths (new means absent at432):

1. `crates/search/src/dictionary.rs`
2. NEW `crates/search/src/dictionary/borrowed.rs`
3. NEW `crates/search/src/dictionary/projection.rs`
4. `crates/search/src/collection_query.rs`
5. NEW `crates/search/src/collection_query/projection.rs`
6. `crates/search/src/lib.rs`
7. `crates/search/README.md`
8. NEW `crates/search/tests/dictionary_projection_contract.rs`
9. NEW `crates/search/tests/collection_projection_contract.rs`
10. `crates/store/src/collections.rs`
11. NEW `crates/store/src/collections/reference_projection.rs`
12. `crates/store/src/lib.rs`
13. NEW `crates/store/tests/reference_projection_contract.rs`
14. `crates/cli/src/main.rs`
15. NEW `crates/cli/src/query_output.rs`
16. NEW `crates/cli/tests/query_output_contract.rs`
17. `crates/cli/README.md`
18. `docs/collections.md`
19. `README.md`

Store parent changes are only targeted read projection/error/export wiring; no mutation algorithm/schema change. Existing tests stay byte-identical; new test homes own this increment. No adding public mutable index internals, Deref engine façade, dependency or feature to rescue a helper. Private names may vary with an explained equivalent; public API/wire/state/error choices are binding.

Planned numbered01 may touch those same paths plus NEW `crates/search/src/presentation.rs` and NEW `crates/search/tests/presentation_contract.rs`; union21. Its exact subset and recipe are reviewed against actual accepted initial source before issue. Supporting evidence lives in this slice artifacts/; preserve failures/receipts and the two prior prospective notes here without inventing close-set placeholders. It adds the renderer/format, numeric profiling proof and final runnable W1–W5 guide/tests; no second shadow full engine. Current plan already freezes its observable formatting and numerical policy below, but is not an advance assignment.

## 3. Initial public API and wire

All new response records below derive Debug/Clone/PartialEq/Serialize and have public documented fields; they are projection data, not falsely described as validated immutable domain models. New key/request structs also Deserialize with deny_unknown_fields and no silently ignored required key. Version1 is distinct from existing full v1 because public types and `projection_policy` differ. Existing response derives/keys remain untouched. Types exported from search/lib.rs; store projection types from store/lib.rs.

`DictionaryCompactSearchEngine` owns exactly one existing provider and its typed-unverified DictionaryDocument plus ordinal indexes and selected membership; it NEVER owns/constructs DictionarySearchEngine, DictionarySourceAvailability or DictionaryExactResponse. APIs:

- `open(path: impl AsRef<Path>) -> Result<Self, DictionaryQueryError>`.
- `open_snapshot(topology: DictionaryReadTopology, manifest: DictionarySnapshotManifest) -> Result<Self, DictionaryQueryError>` and matching `open_snapshot_profiled(...) -> Result<(Self, DictionarySnapshotOpenProfile), DictionaryQueryError>`.
- `state_sha256(&self) -> &str`, `snapshot_sha256(&self) -> &str` (stable operational binding, not authenticated model digest).
- `execute(&self, request: DictionaryRequest) -> Result<DictionaryCompactQueryResult, DictionaryQueryError>`; `exact(&self, spelling: &str, options: DictionaryExactOptions) -> Result<DictionaryCompactExactResponse, DictionaryQueryError>`; `resolve(&self, target: &RelationTarget) -> Result<DictionaryCompactReferenceResolution, DictionaryQueryError>`.
- `detail(&self, reference: &DictionaryHitDetailRef, expected_snapshot: Option<&str>) -> Result<DictionaryHitDetailResponse, DictionaryProjectionError>`.
- `execute_profiled` has the same input/result plus `DictionaryProjectionProfile` on success; ordinary execute delegates to the same inner operation without mandatory profiling.

`DictionaryCompactQueryResult` uses exactly `{operation:"exact"|"resolve",response:...}`. Exact top-level keys: version=1, projection_policy=`compact-dictionary-v1`, policy (existing exact policy), store_schema, catalogue_revision, state_sha256, snapshot_sha256, spelling, resolved_sources, native_candidate_occurrences, totals, source_totals, availability, groups, next_cursor, code_notices. Existing totals/source-total/group count meanings and zero behavior remain. Group fields source/full_count/returned_count/partial/hits have their existing meanings.

`DictionaryCompactHit` keys: occurrence_id, entry (complete EntryRecord), assertion (complete AssertionRecord), capture (CaptureMetadata), snapshot_attribution (complete SnapshotAttribution), occurrence_attribution (Option<OccurrenceAttribution>), contributor (Option<ContributorRecord>), distributor (Option<DistributorRecord>), release_ids (selected available memberships in original metadata order), detail_ref. Complete assertion includes definition, language, timestamps, native_sense_id, source_score and eligibility; complete entry retains all Presence fields. Do not replace Missing/Null/empty Value with strings or omit them. No winner/dedup by definition/native ID.

Compact availability contains source, aliases, lineage, collections, components, releases, selected_available_release_ids, loaded_captures, missing_captures, owned_head, selectable, reason. All are the existing small records except releases are `CollectionReleaseSummary` (NO assertions; normalizer/pins/terms/counts remain) and captures are CaptureRef only. Add `impl From<&ReleaseRecord> for CollectionReleaseSummary` by extracting the exact existing registration_summary_bytes field mapping; use that conversion for both existing summary and compact projection without changing old fields/values. Do not clone a ReleaseRecord to build this conversion. No unrelated whole release assertion-ID, capture occurrence/native-field, context/form/editorial/etc vector appears. Code notices are retained; documented omissions are retrieved through detail and existing explicit metadata/verify APIs.

Compact Resolve keys: version=1, projection_policy, state_sha256, snapshot_sha256, target, status, entity (Option<CollectionEntityReference>, lightweight role/IDs/link/spelling descriptor), confirmed, unverified, availability, requested_release (Option<CollectionReleaseSummary>). Each reference assertion view retains entry, assertion, occurrence_id, occurrence_attribution, capture_metadata (Option), snapshot_attribution (Option), payload_present, current_eligible, release_ids (all scoped memberships, metadata order), owned_revision_ids, tombstoned_by and detail_ref (Option, present only when exact lexical body+attribution are known). NO raw/native fields array. This is explicit projected Resolve, not an assertion that existing Resolve is complete DictionaryHit detail. Nonlexical/excluded targets remain resolvable with their exact Eligibility and current_eligible=false.

`CollectionCompactSearchEngine` exposes `open(root: impl AsRef<Path>, selection: CollectionQuerySelection)`, `open_configured(profiles: &CollectionProfiles, name: Option<&str>, override_packs: Option<&[PackId]>)`, `selection(&self) -> &CollectionSelectionSnapshot`, `execute(&mut self, DictionaryRequest)`, `detail(&mut self,&DictionaryHitDetailRef,Option<&str>)`, all Result with existing CollectionQueryError. Add only typed `CollectionQueryError::Projection(DictionaryProjectionError)` for the new detail categories; preserve source() and report mapping. It also has profiled configured constructor `(Self, Duration)` and profiled execute `(CollectionCompactQueryResponse, CollectionProjectionProfile)`.

Managed compact response keys version/snapshot/collections/result mirror the managed envelope with the NEW compact result. Managed Resolve result keys target/status/operational_reason/payload/catalogue_matches/requested_release mirror current managed semantics; payload is compact reference resolution, requested_release small summary. Managed detail is a separate `CollectionHitDetailResponse` with version/snapshot/collections/response/catalogue_matches. No new variant/required field is added to old DictionaryRequest or old full result/config/mutation schemas.

## 4. Detail identity, admission and errors (binding)

`DictionaryHitDetailRef` exact keys: version=1; digest_policy=`entry-assertion-serde-json-v1`; role:String, semantic value exactly `assertion`; source:SourceId; entry_id:EntityId; assertion_id:EntityId; occurrence:OccurrenceRef; entry_sha256:String; assertion_sha256:String; native_entry_id:Option<String>; native_assertion_id:Option<String>; contributor_id:Option<ContributorId>; distributor_id:Option<DistributorId>. All nullable attribution keys are required (use a strict wire visitor/required-nullable wrapper; derive Option alone accepts a missing key) and emitted as null when absent; Some("") remains present-empty. Native assertion and entry slots never interchange. Flat attribution field values are pinned; absence versus presence of an otherwise all-null OccurrenceAttribution container is not independently encoded in this key and can differ in snapshot-free current detail. The optional observed snapshot detects that legitimate metadata-state change. Source/capture namespace+lowercase64SHA/locator and role bind the exact captured occurrence, independent of spelling/physical path/preferred pack/release.

Digest v1 is SHA-256 of `serde_json::to_vec` of the COMPLETE current typed EntryRecord / AssertionRecord respectively, no LF, no domain prefix, lowercase64hex, matching store record_digest convention. Pinned declaration order is Entry id,source,spelling,language,script,native_word_class,selmaho,mapped_pos; Assertion id,entry,occurrence,native_sense_id,language,definition,source_score,timestamps,eligibility; nested existing serde forms and arbitrary-precision RawJson serializer behavior are part of v1. Changing this encoding/order needs a NEW digest_policy, not silent rebinding. Compute digests ONLY for returned compact hits/reference views and detail admission, never all entries/assertions during setup or page0. Reuse same body-hash helper for record types, not raw input JSON token spelling.

The key freezes lexical bodies and exact recorded attribution fields, NOT all subsequently attached context/form/editorial/computation/extension content. Snapshot-free detail returns current complete attached payload for that same lexical identity. The standalone compact-family snapshot hashes a versioned typed tuple of recorded state SHA, existing provider generation/overlay and stable lightweight availability/owned-head fields; managed detail uses the SAME captured compact snapshot binding as compact Exact/Resolve, although its payload projection label is complete. No path/clock/self-hash or maintenance-only warning enters either. Direct STORE has no invented generation ID: identical recorded state/availability without an explicit generation manifest is equivalent at that boundary. Fast-open header identity is observed, not a comprehensive authenticated/tamper audit. Optional expected_snapshot (lower64hex) is a separate optimistic observed-state precondition. Stable identity survives rebuild/removal/exact restore; physical generation/ROOT/clock/selected preferred release are absent from the key. An explicit current detail request does not silently open an old historical store.

Detail response exact keys version=1, projection_policy=`complete-dictionary-hit-v1`, detail_ref, status, operational_reason (Option<String>), state_sha256 (Option<String>), snapshot_sha256, hit (Option<DictionaryHit>), current_eligible (Option<bool>). Full hit is the EXISTING complete DictionaryHit, no field truncation or unrelated global availability. Its selected release IDs include all admitted contributing scopes. Excluded assertion detail can be Resolved with current_eligible=false; it is not promoted into lexical exact.

Precedence:

| Stage / condition | Result before later work |
| --- | --- |
| CLI request JSON, strict key shape/type, key version/policy/role, invalid IDs/SHA/capture/empty locator | Typed invalid_detail_reference; parse before config/HOME/ROOT. No panic on Unicode cursor/hex. |
| Supplied expected_snapshot malformed | malformed_snapshot, before payload. |
| Supplied expected_snapshot differs from captured compact-family operational binding | stale_snapshot BEFORE NoEnabled/removed/missing derived/payload admission; no payload load. |
| Valid target absent in all retained metadata | NoMatch with hit/current_eligible null. No invented InvalidReference merely for unknown digest. |
| Existing IDs but source/entry/occurrence relation contradicts known model or capture ownership / wrong native role | existing InvalidReference via typed Query; never retarget. Exact native-slot value mismatch or body digest absent among known candidates is NoMatch. |
| More than one retained unequal body matches ordinary unqualified LocalRef | Existing AmbiguousReference. Detail ref's explicit body digests disambiguate its matching historical scopes; equal bodies retain ALL pack/release contributors privately. |
| Key matches historical registration but no matching selected+enabled active release | CatalogueOnly + disabled/not_selected/removed/registered_only precedence exactly current managed semantics; hit/current_eligible null; retain matching metadata scopes. |
| Selected matching release, missing/corrupt/version/IO derived store | Original concrete minimum-open cause; no CatalogueOnly/zero fallback. |
| Matching available payload | Resolved + complete hit; require exact body/attribution match before projection. current_eligible is actual selected lexical eligibility. |

`DictionaryProjectionError` non_exhaustive: Query(DictionaryQueryError), ReferenceJson { source:serde_json::Error }, InvalidDetailReference { reason: DictionaryDetailReferenceInvalidity }, MalformedSnapshot { value:String }, StaleSnapshot { expected:String, observed:String }. Invalidity enum: UnsupportedVersion, UnsupportedDigestPolicy, WrongRole, InvalidDigest, EmptyLocator, InvalidCapture. Typed identifier/shape serde errors are retained as ReferenceJson and mapped invalid_detail_reference at CLI, per §10. No stringified Query/Store/Registry/Profile causes. Retain actual inherited typed enum causes; DictionarySnapshotError has an inherited Error::source traversal stop, so use public variant matching to prove its concrete Store cause rather than expand that unrelated module or claim a traversable chain beyond it. report() returns existing DictionaryErrorReport shape; codes invalid_detail_reference/malformed_snapshot/stale_snapshot, Query delegates exactly. Managed report retains complete current CollectionQueryErrorReport keys, truthful readonly not_applicable, native path/profile/snapshot context, source() causes. No runtime stdout/text-warning pollution.

## 5. Private implementation recipe / ownership

RECOMMENDED route, public policies above BINDING. Do not replace the whole full engine with a new generalized engine. New borrowed.rs owns small reusable functions and ordinal views; full struct/public constructors stay recognizably intact.

1. `SourceSelectionView<'a>` borrows SourceRecord, releases, owned-head and registry aliases; derives selected release IDs and assertion-ID membership using original ReleaseRecord selected+Available, overlay and OwnedDraft head intersection. Extract a shared selected-membership function from build_selected_assertions taking borrowed release iterators plus owned revisions. Full still constructs its original full availability in its original order; compact uses borrowed source views and small summaries. Keep full unknown-overlay check and duplicate-owned-revision timing/messages.
2. `DictionaryOrdinals` owns ID→usize / occurrence→(capture,row) and ID→Vec<usize> indexes, never cloned body/detail rows. Build compact fallible uniqueness checks in EXACT inherited order: capture, capture occurrence, entry, assertion, source, overlay-known-release, owned revision, form, editorial sense, computation, snapshot attribution, occurrence attribution, contributor, distributor. Same labels/errors; no full semantic audit. Group indexes for detail attach rows are primitive ordinals only. Do not precompute lexical-body digests.
3. Share a LAZY per-candidate admission helper taking an assertion lookup and selected membership; require each SQL candidate exists BEFORE selection skip. Full loops in its existing BTreeSet order, calls make_hit eagerly at the same point, and retains original sort/page. Do NOT first validate every candidate entry then hydrate all: that can change first-error precedence. Compact collects only admitted lightweight (source,entry/assertion ordinal,occurrence/key) rows, counts/orders/pages without make_hit. A missing selected entry needed for order is Integrity; absent raw/detail outside the returned page is not eagerly projected by compact. This intentional new projection scope never weakens full errors.
4. Small `HitParts<'a>` / `assemble_hit` clones the exact existing DictionaryHit fields. Existing full make_hit prepares its current borrowed parts from existing indexes; compact detail prepares corresponding parts via ordinal indexes on demand. It shares payload assembly, not an entire full engine. Keep full eager missing capture/raw/snapshot error sequence and release order. Compact Exact directly assembles compact fields for returned rows; detail alone assembles full hit. No clone of the entire DictionaryDocument or provider, no full-engine promotion retaining two complete models.
5. New store `CollectionCatalogue::reference_projection(pack:PackId, release_sha256:&str, target:&RelationTarget) -> Result<CollectionReferenceProjection,CollectionError>` reads retained descriptor bytes on the existing pinned readonly catalogue connection. Implements targeted multi-pass serde DeserializeSeed/Visitor, not full CollectionReferenceMetadata deserialization and trimming. Pass1 target entity/relation and requested release ownership; pass2 associated assertion entity IDs/occurrence scopes; pass3 only those body_sha256/registry attributions/capture locator presence/release membership IDs. Skip unrelated values with IgnoredAny, never allocate whole ReleaseRecord.assertions, raw capture occurrences or every entity body. Full JSON syntax/trailing/duplicate top-level header identity and selected field validation remain required; this is a targeted projection, not a new full descriptor audit. Read byte buffer may be whole descriptor; SQL/lazy/out-of-core claims are NOT made.
6. Projection exact fields: version, pack_id, release_sha256, payload_kind, dictionary_version, registry_version, upstream_catalogue_revision, entities (target + associated assertions + linked Entry descriptors), body_sha256 (matching role:id keys only), registry (only source/aliases/lineage and matched snapshot/occurrence/contributor/distributor records), releases (small summary plus matching_assertion_ids only), captures (CaptureRef/metadata/byte_length and only requested locator-presence observations), code_notices. Unknown LocalRef returns no matches; Catalog requires full known release/source/capture/locator validation. Preserve Entry/Form association by matching assertion.entry_ids (Entry descriptor itself has empty assertion_ids); Editorial uses exact stored assertion links, preserving distinct membership semantics. New CollectionError::ReferenceProjection {pack:PackId,release_sha256:String,source:serde_json::Error} preserves typed parse cause, maps unsupported_version. Full reference_metadata remains unchanged, including inherited error behavior.
7. Share pure target validation/equivalence/all-contributor admission/release filtering functions between full and compact managed paths. Full feeds its existing full descriptor/materialization; compact feeds targeted projection. Keep private admission scopes separate from concise public dedup, S03 cross-pack shared Entry/Assertion fixes and S04 removed-vs-registered state. No .ok()? swallowing errors. Every new catalogue read checks captured local revision; no stale reread bypass. Extract existing capture-selection routine into a private context parameterized ONLY by projection policy; old full uses exact prior strings. Compact uses `compact-dictionary-v1`, same selection v2, explicit compact four-key order; snapshot binding excludes maintenance-only warning fields and never wallclock/self-hash.
8. Managed outer compact cursor binding is validated before admission/payload setup exactly as full. Compact inner cursor prefix `dictionary-compact-v1:` with lowercasehex serde envelope version1/binding/last{source,spelling,occurrence_id,assertion_id}/SHA256 checksum; managed prefix `collection-compact-v1:` binds managed snapshot plus unchanged inner. Validate ASCII lowerhex before slicing, strict/trailing JSON, known version/checksum. ONLY fully well-formed/checksummed foreign full↔compact cursors are StaleCursor; malformed foreign/unknown syntax is MalformedCursor (exact grammar/sites §10); changed spelling/source/page policy mismatches stale, nonexistent last tuple InvalidCursorPosition. Page size itself may change as existing full allows. Fresh process same binding accepts. Page0 groups=[] next=null counts complete and ZERO compact-hit/body-digest/full-hit projections. Readable later shares compact cursors unchanged.
9. Profiling is opt-in same-code, not a user flow. `DictionaryProjectionProfile {total:Duration,lookup:Duration,projection:Duration,candidate_sql:Duration}`: lookup includes SQL+selection/order/cleanup, candidate_sql is diagnostic subset, total=lookup+projection. Outer timer ends AFTER inner owned locals drop; projection sums actual response field clones/detail hashes/hit assembly; rendering is caller-measured. `CollectionProjectionProfile {total:Duration,payload_setup:Option<DictionarySnapshotOpenProfile>,query:DictionaryProjectionProfile}` separates first lazy provider setup; query.total excludes measured payload setup and includes wrapper selection/metadata/cloning/cleanup. Add matching full managed execute_profiled using unchanged full operation and existing exact_profiled; profile full Resolve's actual availability/view materialization. Existing full profiling API remains compatible; no source counters substituted for latency.

## 6. CLI, readable follow-up and exact output boundary

Initial adds `--format full|compact` to existing dictionary-query and lookup, default FULL for both TTY/pipes. No request schema alteration. Parse request before paths/setup as current managed route; dispatch chosen constructor BEFORE provider setup. Add separate `dictionary-detail [STORE] --reference JSON [--expected-snapshot SHA]` and `lookup-detail --reference JSON [--expected-snapshot SHA] [--config FILE] [--profile NAME] [--pack ID]...`. No stdin detail ambiguity: --reference is required strict JSON; all native STORE/config paths are PathBuf; optional STORE resolution identical to dictionary-query, explicit paths override defaults. Lookup detail preserves config-location/profile/disabled typing and no real-default access in tests. Detail output is complete JSON+LF in initial.

query_output.rs owns private `QueryOutput::Json(Value)` and later `Readable(String)` plus typed execution/output-origin context. This is a CLI transport boundary, not reusable search logic. Json full goes through exact existing serde Value+LF path; compact serializes its typed projected result. New details use existing structured standalone error shape / complete managed envelope and truthful readonly activation. No format is selected by inspecting response JSON keys. Preserve graceful BrokenPipe; non-BrokenPipe output/flush failures use typed current managed readonly envelope and original IO source; legacy full standalone errors remain compatible.

Numbered01 adds format readable (same compact constructor/result) and reusable search presentation module APIs `render_dictionary_compact(&DictionaryCompactQueryResult)->String`, `render_collection_compact(&CollectionCompactQueryResponse)->String`, `render_dictionary_detail(&DictionaryHitDetailResponse)->String`, `render_collection_detail(&CollectionHitDetailResponse)->String`. Detail commands then accept `--format full|readable`, full default; compact is rejected at Clap because detail is already an explicit complete projection. Exact/Resolve readable is source-grouped, including nonlexical/excluded/catalogue-only targets. No full response-to-readable shortcut.

Readable v1 is deterministic uncoloured UTF-8, no terminal-width probing, truncation or automatic hard wrap. Literal LF structures lines; corpus control characters including ESC/CR/tab/newline render via JSON string escaping, retaining visible exact content rather than executing terminal controls. Header `Exact <JSON spelling>: <N> occurrences, <D> spellings, <G> sources`; source line `<abbreviation> [<source_id>] <returned>/<full>` plus ` (partial)`; hit line two spaces + `<assertion_id> @ <occurrence_id>`; following indented labelled `spelling: <Presence JSON>`, `definition: <Presence JSON>`, `detail: <compact key JSON>`. Missing, Null and Value forms use exact serde Presence representations. Overall `next_cursor: <JSON string-or-null>` always printed, LF terminated. Resolve header `Reference <target JSON>: <PascalCase status>` plus `reason: <JSON string-or-null>` for managed; group confirmed/unverified by source using same order; each projected assertion prints eligibility/payload/current_eligible and detail key; metadata-only entity/matches and requested release summary remain explicit. Unknown/Unresolved still print target+status, never fake zero exact. Detail header `Detail <assertion_id>: <status>`, snapshot/reason and complete DictionaryHit as deterministic pretty JSON when present, explicit `hit: null` otherwise. Detail readable is intentionally complete and has no compact five-hit byte cap. These exact labels/lines become golden fixtures before numbered01 code.

## 7. Persistent author controls / independent proof

Use inline portable copies of the THREE S03 readiness01 fixtures (lensisku.json used by JB+La, llg-valid-row.txt, sutysisku.json) via normal source-literal helper code; NO absolute planning include paths in product tests. Strict adapters/seals with original pins. Before installation compute independent 25-family max-multiset/capture/source/native inventory, qualify comparator with positive/reorder/missing empty family/multiplicity/Presence mutation. Named fixture: six raw received rows, five lexical farvi occurrences, three source groups, per source lensisku2/llg1/sutysisku2, per pack JB1/LLG1/La1/Sut2. Full retained corpus differs (LSK3/LLG1/Sut1). Assert identities from pre-input models, not candidate-picked expected hits. Supplemental accepted complete_fixture has nonempty Form/editorial/Presence/links; use it alongside all4, not as substitute.

Initial persistent tests in named new files must prove:

- Full baseline exact complete Value+LF, pages0/1/2/3/all, source IDs/aliases/retired ambiguity/errors; existing full eager out-of-page missing capture/raw error preserved via actual fast-open corrupted fixture, with valid counterpart. Compact has same eligible IDs/totals/order on valid fixtures; all positive page concatenations, fresh-engine continuation, changed spelling/source/membership/policy/rebuild/remove-all→stale before unavailable, malformed Unicode/uppercase cursor and cross-projection error. Page0 actual projection counters=0, full availability builder/make_hit never reached by compact, AND public nonempty output/size/provenance tests. Counters supplement source inspection/behavior, not replace them.
- Compact field-by-field pre-input entry/assertion/native attribution/terms projection and complete-hit detail equal existing full DictionaryHit for all five named hits, plus nonempty supplemental contexts/forms/editorial/duplicate link ordering/RawJson/Presence. Changed digest/source/native role/capture/locator negatives and valid counterparts; digest wire literal fixture expected before candidate. Rebuild keeps stable key valid; explicit snapshot stale; removal metadata-only; exact reinstall restores; changed body old key never retargets; changed attached context shows snapshot-free-current detail vs stale observed snapshot.
- Targeted retained metadata projection: known Local Entry/Assertion/Form/Editorial and source+release+native333 JB versus La, capture/locator/wrong-slot/absent-native/Unknown/Unresolved/excluded controls; compare intended small fields to accepted full metadata, preserve all shared-body contributors incl distinct assertion scopes under each sole-selected/all/disabled and unequal history ambiguity. Huge unrelated assertion-inventory sentinel is skipped, never returned/cloned; corrupt/trailing target descriptor fails typed cause, not success. Ordinary full metadata error behavior unchanged.
- CLI real compact/detail noHOME explicit-config, request-before-config error precedence, explicit STORE legacy/full bytes, native path and all-absolute deleted-cwd, strict error exact key/null contexts, BrokenPipe and Linux actual /dev/full readonly envelope. Nonempty Linux native CONFIG+ROOT positives forced, Mac equivalent-IO failure qualified; no zero-test Linux pass or generic fallback branch relabelled success. New tests own these routes; inherited controls retained with precise applicability.

Numbered01 adds exact readable golden/Unicode/control escaping/multiplicity/pages/ref/status/detail tests, real stdout failure counterparts, full W1→configure→query→lifecycle→readable/compact/detail journey with isolated HOME/XDG. All commands use explicit config/owned ROOT; no actual default stat/read. Fresh independently assigned Sol assembled ARC review after W5 and current Linux must EXECUTE selected combined workflows and review all five children; root receipt reconciliation alone is not that higher-scale execution. CDC/root retain independent six-row disposition, current platform gate, retro and inventory/mechanical close duties.

Before any initial author tests: make format, then make lint. Final stable make format→make lint→make check; isolated search no-default, Synthetic-prepared all-features, list-only package inventories for store/search/CLI with offline pinned guard and one target. No publish/verify-package/network. Preserve every failing source/command/log/exits/population; root handoff only clean scoped commit with two parsed trailers. No broad repeat once same-source gate passes absent real change/failure. CC proposed-done != independent acceptance.

## 8. Approved numerical protocol and final retained-Full proof

Authority is topology-comparison-protocol v1.2 explicit “this looks good! Approved, and engage!”, applying v1.1 numeric sheet to affected S05. Original workflow1.0 'no threshold' is historical. S01 comparison7stores/7h is closed, not a new S05 store/effort allowance. This is output implementation validation, not reopening topology selection.

| Boundary | Median / maximum of three observations |
| --- | --- |
| Fresh typed startup/setup before query/projection/output | 5s / 8s |
| Fresh full farvi process E2E captured stdout | 6s / 10s |
| Fresh compact (and same-projection readable) farvi E2E | 5.5s / 9s |
| Warm Exact lookup excluding hydration; Resolve WHOLE construction excluding render | 250ms / 750ms |
| Full farvi hydration+serialization incl LF | 500ms / 1000ms |
| Compact/readable projection+serialization incl LF | 50ms / 150ms |
| Five-hit compact / readable complete UTF8 bytes | 65536 / 131072 maximum, respectively |

No detail-body byte cap is invented; complete DictionaryHit must remain complete. Detail latency is recorded separately without falsely passing an unapproved exact/ref target. Read/query OS highwater <=8GiB; S05 NEW scratch <=12GiB excluding shared target (record before/after cache delta). <=5SQLite connections and5logicalDBs total; structural proof is labelled proven, actual FD/auxiliary observations measured or explicitly unavailable, never fabricated. Full/compact serialization uses actual typed serde/Value conversion +LF; render work includes escaping/string construction. No silent truncation/drop/projection change to meet thresholds. Failed gate retains evidence and returns for explicit disposition; no automatic extra trial or budget relaxation.

After initial+readable independent Tiny success and current source freeze, one reviewed finite read-only campaign reuses S04 final ACTIVE G4 at `/tmp/vlajmaji-p01-a04-s04-cdc/full-operational01` and config `/tmp/vlajmaji-p01-a04-s04-cdc/full-config01.toml` (profile revision5, catalogue4); no writes/migration/rebuild/import/create/All. Caller/CLI must explicitly drop each provider/engine/model before next. Protect original S04 outputs,4generations and preserved binaries. Baseline optimized CLI outside target `.../full-frozen-binaries01/vlajmaji` SHA20b21d4e27310437d643b246ecbaf652b5a4c6e53c407abf6035dca261d1b79c remains untouched. Baseline e4 production==432; don't call baseline S05 code. All-pack accepted inner Value+LF7160366B/SHA144b8b09b6e378243b36c8e01ab06353c3d573459668c8c158a86aebbea73556.

Freeze source/lock/runtime SQLite/binary/guard/oracle/fixture closure, actual argv/native paths/env and current free space/cache before launch. One whole-campaign DB+catalogue SHA pair; private scratch HOME/XDG_DATA_HOME/XDG_CONFIG_HOME prevents fallback to real default even on negative routes. Active600s processgroup timeout per child;8GiB RSS/128MiB individual stage output/new12GiB inventory are POST-COMPLETION thresholds unless a real enforcement wrapper says otherwise. Include dot sidecars/dirs/expected fingerprints and named small postinventory self-byte delta. Stop first unexpected exit/semantic/budget failure; no reset/retry/cold-cache claim. No OS cache purges or concurrent build/heavy reader.

Finite performance schedule: THREE balanced rounds. Fresh actual CLI order round1 baseline-full,candidate-full,compact,readable; round2 readable,compact,candidate-full,baseline-full; round3 compact,baseline-full,readable,candidate-full (12processes). Each runs one unpaged all4 farvi. Candidate warm order round1 full,compact,readable; round2 readable,full,compact; round3 compact,readable,full (9processes). Each warm process opens one engine and executes SAME27 fixed S01 requests, once priming then once measured: requests-full03.json54records (27prime/27measured), exact18/ref9; regenerate cursor tokens per projection and preserve graph, all page sizes1/2/3 concatenate fully. This frozen workload file is a query specification only, not old cursor bytes. Full source subset/native refs remain original known targets. Profile each same call under §10 per-ID/whole-Resolve acceptance; serialize actual complete output into a bounded buffer+LF then hash/size and drop before next request. Retain all summary rows and representative body; warm process stdout is summary, not 27 huge full bodies. Do NOT claim that summary is actual CLI E2E.

One additional semantic observer process (22nd maximum) checks candidate full accepted wire, projected five IDs/native fields/detail complete payloads and five stable refs + wrong-role, small source overrides and page concatenation with separate model/engine scopes; no measured extra rounds. New compact/readable expected fields derive prior original witnesses/Tiny-qualified comparator before reading candidate. Method qualification includes missing/extra key, Presence, native role, count, hash mismatches and guard expected exits. Baseline fresh CLI is paired E2E only; no invented old-library warm benchmark or All/body reauthentication. Static no-heavy-path evidence, actual bytes/process/RSS/timers and semantic agreement are separate obligations.

This is a concrete protocol ceiling, not current execution authority: final exact runner/current helper qualification/Tiny same-code journey/source and binary identity/expanded argv still require root executable readiness. Linux current-head CI requires separate candidate-specific external authorization; no S04 approval reused. S05 close and assembled arc follow actual semantic/numeric/platform evidence and root canonical consistency; A05 remains outside authority.

## 9. Applied guidance / author readiness

| Verified guide | Applied choice and proof |
| --- | --- |
| Rust EH-04 MUST; AP-54 AVOID | Named ProjectionError/typed concrete sources; actual invalid key, native IO and missing/corrupt/version matchable tests. Existing inherited errors not broadly rewritten. |
| Rust PF-12/13 SHOULD; AP-18 AVOID | Borrowed release rows/ordinal indexes and page-before-clone; actual compact no-heavy/page0 discriminator plus real bytes/latency. Not a speculative engine-wide optimization. |
| Rust OB-16 MUST | Explicit ownership/drop scopes in serialized Full observations; no claim Drop establishes recovery/durability. No source mutation here. |
| Engineering-methods07 §§1–6 | Author inspected432 symbols/consumers, fixes public consequences here, exact19paths and concrete positive/negative oracles; CC challenges actual mismatch before dependent edits. |
| PM03 usable increments/iteration and PM02 preservation | Initial API+CLI+docs usable, readable01 explicit and counted, issued prompt bytes retained, normal five-total limit; no premature close artifacts. |

Readiness author state: actual source/branch clean432; source-guided methods/data shapes above are UNCOMPILED design, not a patch or measured guarantee. No unresolved policy is delegated to CC; any source/type contradiction pauses affected work for CDC/root, not an invented public fallback. Root final readiness PASS received at the exact reviewed hashes; issued initial now requires exact full prompt/plan/ledger and bounded normative sections/data intake, then explicit sole19path/index/target transfer. Execution record created when actual intake/work occurs, not prefilled success. Closure requires normal CC report, CDC independent verifier/retro, scope inventory and fresh ARC gate at its proper scale.

## History

- 2026-10-27 DRAFT1.0: after S04 closed be873/metadata7b329 and root-created PR5/new branch, source-only S05 open set prepared. PR5 CI40518812275 terminal success at432 is predecessor-tree proof, not S05 acceptance. Preserve its three actual receipts in artifacts/readiness01. Root exact S05 review pending; no implementation.

## 10. Exact type/signature/precedence appendix (draft review refinement)

This appendix resolves abbreviated descriptions above; it governs exact Rust names/types where prose used “mirror” or “matching”. `lexicon` means vlajmaji_lexicon and `store` vlajmaji_store. Imports below are existing types. New response/projection records derive **Debug,Clone,PartialEq,Serialize**, NOT blanket Eq (existing CollectionEntityReference/CollectionReferenceMatch are PartialEq-only). Key additionally has strict Deserialize; ordinary Deserialize is unvalidated data, while from_json_bytes and every detail entrypoint validate semantics. Error enums implement Debug,Display,Error and typed source(), not Clone/Eq. Profile structs derive Debug,Clone,Default and are NOT serialized into existing responses.

Complete public response shapes (declaration/wire order is the order below; no flattened/omitted/null keys). These are UNCOMPILED design declarations. Rust field types are binding, private representation is not:

```rust
pub struct DictionaryCompactSourceAvailability {
    pub source: lexicon::SourceRecord,
    pub aliases: Vec<lexicon::SourceAlias>,
    pub lineage: Vec<lexicon::SourceLineage>,
    pub collections: Vec<lexicon::CollectionRecord>,
    pub components: Vec<lexicon::ComponentRecord>,
    pub releases: Vec<store::CollectionReleaseSummary>,
    pub selected_available_release_ids: Vec<lexicon::EntityId>,
    pub loaded_captures: Vec<lexicon::CaptureRef>,
    pub missing_captures: Vec<lexicon::CaptureRef>,
    pub owned_head: Option<lexicon::OwnedHead>,
    pub selectable: bool,
    pub reason: Option<SourceSelectionFailure>,
}
pub struct DictionaryCompactHit {
    pub occurrence_id: String,
    pub entry: lexicon::EntryRecord,
    pub assertion: lexicon::AssertionRecord,
    pub capture: lexicon::CaptureMetadata,
    pub snapshot_attribution: lexicon::SnapshotAttribution,
    pub occurrence_attribution: Option<lexicon::OccurrenceAttribution>,
    pub contributor: Option<lexicon::ContributorRecord>,
    pub distributor: Option<lexicon::DistributorRecord>,
    pub release_ids: Vec<lexicon::EntityId>,
    pub detail_ref: DictionaryHitDetailRef,
}
pub struct DictionaryCompactGroup {
    pub source: lexicon::SourceRecord,
    pub full_count: usize,
    pub returned_count: usize,
    pub partial: bool,
    pub hits: Vec<DictionaryCompactHit>,
}
pub struct DictionaryCompactExactResponse {
    pub version: u32,
    pub projection_policy: String,
    pub policy: String,
    pub store_schema: u32,
    pub catalogue_revision: u64,
    pub state_sha256: String,
    pub snapshot_sha256: String,
    pub spelling: String,
    pub resolved_sources: Vec<lexicon::SourceId>,
    pub native_candidate_occurrences: usize,
    pub totals: DictionaryTotals,
    pub source_totals: Vec<DictionarySourceTotal>,
    pub availability: Vec<DictionaryCompactSourceAvailability>,
    pub groups: Vec<DictionaryCompactGroup>,
    pub next_cursor: Option<String>,
    pub code_notices: Vec<lexicon::CodeNotice>,
}
pub struct DictionaryCompactReferenceAssertion {
    pub entry: lexicon::EntryRecord,
    pub assertion: lexicon::AssertionRecord,
    pub occurrence_id: String,
    pub occurrence_attribution: Option<lexicon::OccurrenceAttribution>,
    pub capture_metadata: Option<lexicon::CaptureMetadata>,
    pub snapshot_attribution: Option<lexicon::SnapshotAttribution>,
    pub payload_present: bool,
    pub current_eligible: bool,
    pub release_ids: Vec<lexicon::EntityId>,
    pub owned_revision_ids: Vec<lexicon::EntityId>,
    pub tombstoned_by: Vec<lexicon::EntityId>,
    pub detail_ref: Option<DictionaryHitDetailRef>,
}
pub struct DictionaryCompactReferenceResolution {
    pub version: u32,
    pub projection_policy: String,
    pub state_sha256: String,
    pub snapshot_sha256: String,
    pub target: lexicon::RelationTarget,
    pub status: ReferenceStatus,
    pub entity: Option<store::CollectionEntityReference>,
    pub confirmed: Vec<DictionaryCompactReferenceAssertion>,
    pub unverified: Vec<DictionaryCompactReferenceAssertion>,
    pub availability: Vec<DictionaryCompactSourceAvailability>,
    pub requested_release: Option<store::CollectionReleaseSummary>,
}
// Both result enums: serde(tag="operation",content="response",rename_all="snake_case").
pub enum DictionaryCompactQueryResult {
    Exact(Box<DictionaryCompactExactResponse>),
    Resolve(Box<DictionaryCompactReferenceResolution>),
}
pub struct CollectionCompactReferenceResponse {
    pub target: lexicon::RelationTarget,
    pub status: ReferenceStatus,
    pub operational_reason: Option<String>,
    pub payload: Option<DictionaryCompactReferenceResolution>,
    pub catalogue_matches: Vec<CollectionReferenceMatch>,
    pub requested_release: Option<store::CollectionReleaseSummary>,
}
pub enum CollectionCompactQueryResult {
    Exact(Box<DictionaryCompactExactResponse>),
    Resolve(Box<CollectionCompactReferenceResponse>),
}
pub struct CollectionCompactQueryResponse {
    pub version: u32,
    pub snapshot: CollectionSelectionSnapshot,
    pub collections: Vec<CollectionPackAvailability>,
    pub result: CollectionCompactQueryResult,
}
pub struct DictionaryHitDetailRef {
    pub version: u32,
    pub digest_policy: String,
    pub role: String, // semantic value EXACTLY "assertion"; unknown role is typed invalidity
    pub source: lexicon::SourceId,
    pub entry_id: lexicon::EntityId,
    pub assertion_id: lexicon::EntityId,
    pub occurrence: lexicon::OccurrenceRef,
    pub entry_sha256: String,
    pub assertion_sha256: String,
    pub native_entry_id: Option<String>,
    pub native_assertion_id: Option<String>,
    pub contributor_id: Option<lexicon::ContributorId>,
    pub distributor_id: Option<lexicon::DistributorId>,
}
pub struct DictionaryHitDetailResponse {
    pub version: u32,
    pub projection_policy: String,
    pub detail_ref: DictionaryHitDetailRef,
    pub status: ReferenceStatus,
    pub operational_reason: Option<String>,
    pub state_sha256: Option<String>,
    pub snapshot_sha256: String,
    pub hit: Option<DictionaryHit>,
    pub current_eligible: Option<bool>,
}
pub struct CollectionHitDetailResponse {
    pub version: u32,
    pub snapshot: CollectionSelectionSnapshot,
    pub collections: Vec<CollectionPackAvailability>,
    pub response: DictionaryHitDetailResponse,
    pub catalogue_matches: Vec<CollectionReferenceMatch>,
}
pub struct DictionaryProjectionProfile {
    pub total: std::time::Duration,
    pub lookup: std::time::Duration,
    pub projection: std::time::Duration,
    pub candidate_sql: std::time::Duration,
}
pub struct CollectionProjectionProfile {
    pub total: std::time::Duration,
    pub payload_setup: Option<DictionarySnapshotOpenProfile>,
    pub query: DictionaryProjectionProfile,
}
// These three public types live in store (never depend on search's detail key).
pub struct CollectionProjectedRelease {
    pub summary: CollectionReleaseSummary,
    pub matching_assertion_ids: Vec<lexicon::EntityId>,
}
pub struct CollectionProjectedCapture {
    pub capture: lexicon::CaptureRef,
    pub metadata: lexicon::CaptureMetadata,
    pub byte_length: usize,
    pub locator_presence: std::collections::BTreeMap<String, bool>,
}
pub struct CollectionReferenceProjection {
    pub version: u32,
    pub pack_id: String,
    pub release_sha256: String,
    pub payload_kind: String,
    pub dictionary_version: u32,
    pub registry_version: u32,
    pub upstream_catalogue_revision: u64,
    pub entities: Vec<CollectionEntityReference>,
    pub body_sha256: std::collections::BTreeMap<String, String>,
    pub registry: lexicon::RegistryDocument,
    pub releases: Vec<CollectionProjectedRelease>,
    pub captures: Vec<CollectionProjectedCapture>,
    pub code_notices: Vec<lexicon::CodeNotice>,
}
```

RegistryDocument above is an explicitly FILTERED unvalidated projection, not a purported validated SourceRegistry; version remains original, only related source/alias/lineage/snapshot/occurrence/contributor/distributor records retained in original order. Release/capture/entity vectors retain original relative order; maps use BTreeMap key order. `locator_presence` contains ONLY tested candidate/requested locators within a retained loaded capture; false means known retained capture lacks that locator. No capture record is invented for unavailable/nonretained captures; release.captures establishes named absence as existing resolver does. Matching assertion IDs are the actual target-projected set (which can legitimately be many for an Entry), never unrelated whole release inventories. A full summary includes existing exact CollectionReleaseSummary keys/types, no new serialized field there.

Exact method signatures supplement §3. Opaque engine structs have private fields, no Clone/Deref/full-engine ownership. `DictionaryCompactSearchEngine::{open,open_snapshot,open_snapshot_profiled,state_sha256,snapshot_sha256,exact,resolve,detail}` are exactly the signatures in §3; no shorthand applies below:

```rust
impl DictionaryHitDetailRef {
    pub fn from_json_bytes(bytes: &[u8]) -> Result<Self, DictionaryProjectionError>;
    pub fn validate(&self) -> Result<(), DictionaryProjectionError>;
}
pub fn validate_detail_snapshot_sha256(value: &str) -> Result<(), DictionaryProjectionError>;
impl DictionaryCompactSearchEngine {
    pub fn open(path: impl AsRef<std::path::Path>) -> Result<Self, DictionaryQueryError>;
    pub fn open_snapshot(topology: store::DictionaryReadTopology, manifest: store::DictionarySnapshotManifest) -> Result<Self, DictionaryQueryError>;
    pub fn open_snapshot_profiled(topology: store::DictionaryReadTopology, manifest: store::DictionarySnapshotManifest) -> Result<(Self, DictionarySnapshotOpenProfile), DictionaryQueryError>;
    pub fn state_sha256(&self) -> &str;
    pub fn snapshot_sha256(&self) -> &str;
    pub fn exact(&self, spelling: &str, options: DictionaryExactOptions) -> Result<DictionaryCompactExactResponse, DictionaryQueryError>;
    pub fn resolve(&self, target: &lexicon::RelationTarget) -> Result<DictionaryCompactReferenceResolution, DictionaryQueryError>;
    pub fn detail(&self, reference: &DictionaryHitDetailRef, expected_snapshot: Option<&str>) -> Result<DictionaryHitDetailResponse, DictionaryProjectionError>;
    pub fn execute(&self, request: DictionaryRequest) -> Result<DictionaryCompactQueryResult, DictionaryQueryError>;
    pub fn execute_profiled(&self, request: DictionaryRequest) -> Result<(DictionaryCompactQueryResult, DictionaryProjectionProfile), DictionaryQueryError>;
}
impl CollectionCompactSearchEngine {
    pub fn open(root: impl AsRef<std::path::Path>, selection: CollectionQuerySelection) -> Result<Self, CollectionQueryError>;
    pub fn open_configured(profiles: &store::CollectionProfiles, name: Option<&str>, override_packs: Option<&[lexicon::PackId]>) -> Result<Self, CollectionQueryError>;
    pub fn open_configured_profiled(profiles: &store::CollectionProfiles, name: Option<&str>, override_packs: Option<&[lexicon::PackId]>) -> Result<(Self, std::time::Duration), CollectionQueryError>;
    pub fn selection(&self) -> &CollectionSelectionSnapshot;
    pub fn execute(&mut self, request: DictionaryRequest) -> Result<CollectionCompactQueryResponse, CollectionQueryError>;
    pub fn execute_profiled(&mut self, request: DictionaryRequest) -> Result<(CollectionCompactQueryResponse, CollectionProjectionProfile), CollectionQueryError>;
    pub fn detail(&mut self, reference: &DictionaryHitDetailRef, expected_snapshot: Option<&str>) -> Result<CollectionHitDetailResponse, CollectionQueryError>;
}
// Additive profiling only; existing full APIs/wire remain.
impl DictionarySearchEngine {
    pub fn resolve_profiled(&self, target: &lexicon::RelationTarget) -> Result<(ReferenceResolution, DictionaryProjectionProfile), DictionaryQueryError>;
}
impl CollectionSearchEngine {
    pub fn open_configured_profiled(profiles: &store::CollectionProfiles, name: Option<&str>, override_packs: Option<&[lexicon::PackId]>) -> Result<(Self, std::time::Duration), CollectionQueryError>;
    pub fn execute_profiled(&mut self, request: DictionaryRequest) -> Result<(CollectionQueryResponse, CollectionProjectionProfile), CollectionQueryError>;
}
impl store::CollectionCatalogue {
    pub fn reference_projection(&self, pack: lexicon::PackId, release_sha256: &str, target: &lexicon::RelationTarget) -> Result<store::CollectionReferenceProjection, store::CollectionError>;
}
```

Actual432 lexicon/src/lib.rs76 reexports PackId at crate root, so lexicon::PackId above is the verified public path. No new model re-export or dependency is required.

New DictionaryProjectionError also exposes `report(&self) -> DictionaryErrorReport`. New dictionary error exact variants:

```rust
pub enum DictionaryDetailReferenceInvalidity {
    UnsupportedVersion, UnsupportedDigestPolicy, WrongRole,
    InvalidDigest, EmptyLocator, InvalidCapture,
}
#[non_exhaustive]
pub enum DictionaryProjectionError {
    Query(DictionaryQueryError),
    ReferenceJson { source: serde_json::Error },
    InvalidDetailReference { reason: DictionaryDetailReferenceInvalidity },
    MalformedSnapshot { value: String },
    StaleSnapshot { expected: String, observed: String },
}
```

`ReferenceJson` retains the actual serde source; Query retains existing concrete chain. Report code for BOTH ReferenceJson and semantic InvalidDetailReference is invalid_detail_reference. MalformedSnapshot code malformed_snapshot, StaleSnapshot stale_snapshot. New CollectionQueryError::Projection delegates category/detail and source; no old category changes. Existing standalone DictionaryErrorReport is exactly code:String,message:String,sources:Vec<SourceId>. Managed CollectionQueryErrorReport stays exact13fields version,ok,category,detail,operation,config_path,root_path,profile,requested_profile,snapshot,known_pack_ids,outcome,writer.

Static CLI flow: parse key bytes through from_json_bytes (unknown/missing keys, nullable key ABSENCE, wrong JSON type, invalid typed IDs, malformed JSON/trailing bytes → ReferenceJson); then validate key semantic version/policy/role/SHA/capture/locator (the invalidity variants); then validate supplied expected snapshot lower64ASCII with validate_detail_snapshot_sha256; ONLY THEN resolve STORE/config/HOME or open anything. role is String rather than a closed serde enum, so wrong role valid JSON produces WrongRole; unknown policy produces UnsupportedDigestPolicy. Missing native_entry_id/native_assertion_id/contributor_id/distributor_id MUST error rather than Option default. Syntactic typed identifier failure is now explicitly invalid_detail_reference, NOT generic request_json; this resolves the earlier draft ambiguity.

For those pre-open errors dictionary-detail emits the exact3key standalone report/sources=[]/exit1; lookup-detail emits the exact13key managed report, operation=`lookup_detail`, version1/okfalse/category as above, all path/profile/requested_profile/snapshot/writer contexts null, known_pack_ids=[],outcome=not_applicable, exit1. No raw input secret/context guesses or Clap exit2 for runtime key JSON. Clap still handles absent required --reference/unknown --format. After selection capture, enrich errors with actual observed config/root/profile/snapshot as existing managed route does; compare valid supplied expected snapshot before no-enabled/unavailable/payload. A constructor failing to capture a profile/ROOT naturally precedes comparison to an unavailable binding. Standalone engine open already owns its typed document: its detail() validates before new projection, not a false claim of undoing prior provider load. CLI static methods establish pre-open behavior separately.

Standalone snapshot body EXACT declaration order/encoding:

```rust
// private Serialize records, not public response fields
struct CompactSnapshotBody {
    version: u32, // 1
    policy: &'static str, // "compact-dictionary-v1"
    recorded_state_sha256: String,
    generation: Option<String>, // existing provider generation, no invented STORE path ID
    selected_release_ids: Option<Vec<lexicon::EntityId>>, // overlay IDs sorted ascending
    availability: Vec<CompactSnapshotSource>, // source_id ascending
}
struct CompactSnapshotSource {
    source_id: lexicon::SourceId,
    selected_available_release_ids: Vec<lexicon::EntityId>, // original release order
    loaded_captures: Vec<lexicon::CaptureRef>, // existing first-seen release order
    missing_captures: Vec<lexicon::CaptureRef>,
    owned_head: Option<lexicon::OwnedHead>,
    selectable: bool,
    reason: Option<SourceSelectionFailure>,
}
```

SHA-256 of serde_json::to_vec(body), no LF, lowercase64hex. Header recorded state is observed/minimum-open trust, not authenticated canonical hash. None overlay and Some(empty) remain distinct; generation None for standalone STORE can make a rebuild with identical state/availability equivalent, while a supplied snapshot manifest's changed generation differs. Excludes file path/stat/time, warning-only original input/owned-copy presence, full release assertion arrays, linked payload body clones and digest of own hash. Managed uses existing full snapshot binding helper over its unchanged explicit fields (projection/order set to compact policies, selection-v2), not a second subtly different config hash; detail returns that SAME compact-family binding. Full-route binding is unchanged. A legitimate new complete dictionary state with same Entry/Assertion but changed attached ContextRecord changes recorded SHA through normal create/update and stales supplied snapshot; do NOT use unauthenticated SQL tamper to claim context freshness. Snapshot-free key then returns new attached context while lexical key stays equal.

Descriptor traversal closure/algorithm precisions: target Local Assertion includes assertion AND linked Entry descriptor+entry digest; Entry includes itself/all associated assertions; Form includes form/linked Entry/all its assertions; Editorial includes sense/its assertion list/each linked Entry. If the requested Local ID exists only under another role, retain that same-ID role witness in entities so the search layer returns InvalidReference rather than losing it as unknown. A wholly absent ID has no such witness. Keep target role digest for ordinary Local equivalence exactly as existing full resolver; linked body digests are extra admission evidence for detail, not a silent stronger ordinary full ambiguity rule. Catalog includes matching assertion+linked Entry closures. For historical detail pass RelationTarget::Local(LocalRef::Assertion(key.assertion_id)) to store, then compare BOTH key digests/source/occurrence/native slots across every retained projection; never depend on a source→search import or just assertion digest. Equal matches all contribute; a matching historical body absent from current active set is CatalogueOnly.

Fixed multi-pass route: first seed gathers header/small collection/component/release ownership and target entity links; for Catalog it also classifies source/release/capture/native slot attribution using primitive occurrence keys, retaining related attributes only. Native comparison keeps Match/Mismatch/Unknown distinct, including absent occurrence attribution; no raw field fallback for missing native IDs. Second seed finds associated assertion/Entry descriptors in ORIGINAL entity order (for Catalog include source/capture candidate assertions not disproved by native slots; missing actual attribution remains unverified), plus related form/editorial target closure. Third seed streams requested release assertions and retains ONLY candidate-ID intersection, gathers linked role digests, related source/snapshot/occurrence/contributor/distributor records and exact candidate/requested locator-presence observations. No complete registry rows clone just to trim; primitive classification keys are allowed, whole unrelated registry payload is not. Source/release ownership checks precede native matching. JB333 and La333 must be compared against their DISTINCT requested release, namespace/hash and locator from the original frozen query, even though they share Lensisku and native assertion333. Known capture with missing locator is InvalidReference, empty locator/capture outside release invalid, known native slot mismatch NoMatch, absent slot Unverified. Needed target association may require replaying descriptor bytes within the same operation; no extra persistent schema/model/file is introduced.

CLI envelope exactness: existing standalone dictionary-query full remains UNWRAPPED DictionaryExactResponse or ReferenceResolution. New standalone `--format compact` is likewise UNWRAPPED DictionaryCompactExactResponse or DictionaryCompactReferenceResolution; match the library enum privately before serialization. The library execute result retains operation/response tags as declared, but those tags are NOT an extra standalone stdout wrapper. Managed lookup uses CollectionCompactQueryResponse with nested result operation/response; managed compact Resolve response/payload stays its declared nested shape. dictionary-detail emits unwrapped DictionaryHitDetailResponse; lookup-detail emits CollectionHitDetailResponse. One JSON value+LF, no new diagnostic/metrics keys. Initial formats reject readable; numbered01 formats select typed renderers before setup, then emit exactly their String+final LF. Snapshot/key provenance remains visible in the compact response/detail_ref, not inferred from a filename.

Numeric boundary correction and populations (supersedes any abbreviated Exact/Resolve label above): v1.2 explicitly says **References report whole lookup/resolution construction as their conservative warm boundary.** For each of the SAME27 IDs obtain3 measured observations (one per round) and compute median/max PER ID AND implementation; never pool81 or priming rows. Exact warm uses query.lookup (actual SQL+residual excluding measured hydration/projection); Resolve warm uses query.total including availability/targeted descriptor/reference views/wrapper construction and inner cleanup, excluding serialization/render ONLY. Compact reference projection+render is a separately reported/capped overlapping subset, never subtracted from warm Resolve acceptance. Wrong-role is an expected invalid_reference in both prime/measure sets: wrap the actual Err call and error-report construction with a conservative outer same-call timer; no success profile is fabricated. It remains a per-ID warm reference gate, with serialization recorded separately.

Full hydration+serialization500/1000ms applies ONLY primary measured farvi-all (one observation per round, each candidate full; historical baseline CLI provides E2E/bytes only). Other full projection timings are recorded, not passed against an invented full-response cap. Compact/readable projection+serialization50/150ms applies to every SUCCESSFUL fixed measured request (26 IDs each3 observations), including primary five-hit farvi-all, filtered/absent/excluded/pages and successful reference statuses. Wrong-role error has no returned projection profile, so it is explicitly outside that projection cap, not labelled zero/passed; its warm whole-call ceiling still applies. Primary unpaged all4 farvi-all alone supplies five-hit64KiB/128KiB caps; every other response still retains all declared fields with size recorded. Fresh E2E distributions are separately baseline-full/candidate-full/compact/readable, three each. Typed setup distributions separately candidate full/compact/readable three each from process main-entry→pre-query work plus actual lazy payload_setup; process-main startup before constructor and capture read are charged. S01 timers/rounds are not reused as fresh candidate observations.

### Exact cursor grammar / foreign precedence

Compact standalone cursor is `dictionary-compact-v1:<HEX>`, where HEX is lowercase even-length hex of strict serde JSON `{body:{version:1,binding:<lower64hex>,last:{source:<SourceId>,spelling:<String>,occurrence_id:<String>,assertion_id:<String>}},checksum:<lower64hex>}` in that declaration order. Checksum is SHA-256 of serde_json::to_vec(body), no LF. occurrence_id must be the valid tuple-hex shape, assertion_id a valid EntityId string. Managed compact is `collection-compact-v1:<HEX>:<SHA>`; HEX encodes strict `{version:1,snapshot_binding:<lower64hex>,inner:<nonempty compact cursor>}`, SHA is lowercase SHA-256 of those exact decoded bytes. No extra colon/trailing JSON, unsupported version, uppercase/nonASCII/odd hex or invalid digest is accepted.

Recognized foreign legacy standalone is ONLY the actual full naked-lowerhex CursorEnvelope with strict version1/full3key-body/binding64/checksum-valid schema; foreign legacy managed is ONLY actual collection-v1:HEX:SHA strict envelope AND a valid legacy inner CursorEnvelope. Similarly full recognizes new compact prefixes only after strict complete compact grammar/checksums (both layers for managed). A fully valid OTHER projection is typed StaleCursor; malformed foreign-prefix/body/hex/version/checksum/inner is MalformedCursor. Prefix alone never proves staleness. Unknown prefixes are Malformed. No need current binding equality to classify an otherwise valid foreign projection as stale.

Place additive full foreign recognizer exactly at existing `cursor_position` entry (AFTER full eager hydration/source/error position), and at existing managed `unwrap_cursor` entry (BEFORE admission/payload). Do not move/rewrite the existing own-full validation/checksum/binding/position order, generated bytes or opaque-inner behavior. For NEW compact, validate complete own grammar/checksums before current binding comparison, then last-key lookup; malformed wins its stale-binding test, a valid mismatched binding is stale, and a valid matching binding with absent last tuple is InvalidCursorPosition. Managed compact can validate syntax of both new layers metadata-only before admission; actual matching last-position work waits for candidate lookup. Readable uses the exact compact cursor, no separate renderer binding. Persistent paired tests include valid foreign cursor→stale and same foreign cursor with one checksum/hex/trailing/inner mutation→malformed, plus full out-of-page integrity precedence unchanged.

## Draft refinement history

- Root first concrete draft review, 2026-10-27: requested exact public types/signatures, supported derives, explicit snapshot encoding, key parse/report precedence, descriptor linked-entry closure/native333 route, standalone envelopes and whole-reference warm boundary/per-ID populations. This appendix and consistent edits resolve those authoring gaps BEFORE issue; no failed product/test or implementation pass is implied. Root final readiness remains pending.

## Initial issue / readiness history —2026-10-27 UTC

Root FINAL S05 INITIAL READINESS PASS: complete plan69700B/767d64fc…6ad28, prompt15694B/fe0d6d27…4316, ledger4473B/1d1aa1e4…1be76; source-critical API/order/error/descriptor/cursor guards crosschecked, not all-source semantic acceptance. Five provenance/PR5payloads550313B exact originals/hash/size/noextras;19initialpaths, sixOPEN, parent5DONE5OPEN/project14OPEN and source432/newbranch/oldS04refs verified. Prior draft checkpoint pending statements are history. Reviewed draft prompt is byte-preserved at artifacts/cc-prompt-reviewed-draft01.md; issuance changes ONLY its status line, all functional instructions unchanged. Initial is pass1of5total, readable planned pass2; remaining corrections count. Local scoped planning issuecommit precedes dispatch; CC must complete exact required manifest/readback, then CDC explicitly transfers ONLY19paths/index/retainedtarget after fresh identity check. Until then CDC owns quiet source and CC none. Root readonly; no source/Full/Linux/external/readable01/arc acceptance implied.

### Actual initial intake/transfer checkpoint

Complete CC intake recorded at artifacts/execution.md, original SHA8890ff2a…08a15; CDC read/accepted it and fresh-rechecked source432/tree dd2/newbranch/oldS04refs/lock/guard. Parser correction retained, reading attestation not product verification. Scoped metadata commit precedes explicit19path/index/target transfer message; CC then sole author, CDC/root readonly. No required input truncation or consequential mismatch reported; sixOPEN and all future gates persist.

## Current author-state clarification —2026-10-27 UTC

The issue-time1.0 status at949dde and intake transfer atcae2b76d remain historical provenance. The source/dependency432 identity above is the accepted BASELINE, not the uncommitted WIP tree. Earlier author receipts include the four-literal installed25-family result (test15) and initial CLI scaffold (test04); later source edits mean neither is an independent current-source gate. The original finite control census is active, including the newly reported compact page0 empty-group repair. No scope/API/criterion/proof/owner/status change, broad final gate, frozen handback, Full execution, Linux result, readable assignment or acceptance follows.

## 11. Frozen INITIAL disposition and correction iteration01 — DRAFT 2026-10-27

Root user-authorized rebase preserves identical9eb→5bf tree at current clean5bf. Original INITIAL is NOT ACCEPTED: independent frozen review and source corroboration confirm lightweight paging/admission, inherited initialization validation/order, actual same-call profiling/missing full API, exact standalone snapshot body and detail static validation/context gaps. The finite public/edge/CLI diagnostics and all method mistakes are scoped in artifacts/review-initial01.md; exact cdc-initial01 closure is complete (963payloads/2,987,932B, manifestSHA827d275c…4e766). No new source action or gate follows from this DRAFT.

Proposed exact nine-path source subset and complete implementation spine/oracles/readback are in cc-prompt-iteration01.md. It governs only after root readiness and status-only issue/intake/transfer. This is proposed implementation pass2of5 TOTAL, not an extension/reset. Useful readable delivery moves to numbered02/plannedpass3; all numerical/platform/fresh assembled ARC and five-workflow obligations remain. Original issued prompt bytes and original six row cells unchanged.

Binding refinements resolve original omissions, not new architecture: ordinal/borrowed rows until actual page; SQL Assertion existence before membership then Entry admission; selectable zero-assertion source preserved; full actual uniqueness order through primitive compact indices; full and compact same-code optional profile accumulators with outer-local cleanup, query.total excluding measured payload setup and lookup=total-projection (candidateSQLsubset); exact private snapshot key availability; detail capture namespace onlynonempty opaque Unicode, SHA lower64, locator onlynonempty; static lookup-detail ALLcontextsnull/known_pack_ids[] before pack/profile/config enrichment. Do not change ordinary full eager/error/Exact profile semantics, persisted state, store projection, public wire or old Lookup errors.

Current ownership remains CDC exclusive for independent preservation/planning; retained CC/root source/index/target read-only. All six rows OPEN. Prospective prompt source before issue is an uncompiled recommended recipe; actual counterexamples are already executed as described, not a new experiment or benchmark. Required current explicit Rust1.98 gates follow actual repair, not rebase bookkeeping.

### Iteration01 author readiness (unissued; root final review pending)

Source-grounded: current5bf functions/types/call order were read and finite counterexamples completed; actual full constructor/resolve/reference_views and both managed execution/profile bodies inspected. Design-complete: exact nine-path recipe, public API, row/index/validation order, profile ownership/subset accounting, snapshot encoding and static contexts fixed in prompt. Guideline-applied: PF12/13 borrowed ordinal bodies, EH04 typed errors, OB16 explicit normal cleanup scope; no crash/performance claim. Executable: existing fixture/helpers/functions and explicit1.98 offline gate commands fit current19path home. Falsifiable: eight edge/five CLI/missing API plus valid/restored/unchanged-full counterparts reject frozen5bf; method errors stay separate. Coherent: corrected absolute planningAGENTS pointer, controls are slice-plan§7 (original prompt unnumbered), literal ANY-resolved-source BTreeSet predicate before Entry; original prompt/rows preserved. Six criteria are author readiness only, not product acceptance or permission to skip root final review/intake/transfer. Historical §6 readable numbered01 is now deferred numbered02 by this amendment; useful readable obligations unchanged.

### Iteration01 ISSUED / root final readiness PASS — 2026-10-27

Root completely read final draft55ad497b/plan5fa27e0b/ledger1a4d23a3/reviewe5b093bc; source critical API/order/profile/static boundaries rechecked. Verified963payloads/2,987,932B under manifest827d275c:961 exact real task-original copies, exact111880B CC canonical-prefix reconstruction, and one newly authored held inventory (the last two are not falsely counted original-file copies). Six original criterion/proof/owner/status cells and initialpromptSHAa370…f679 unchanged. Exact reviewed-final draft preserved separately; active prompt differs ONLY statusline. Root authorizes ordinary pass2issue/intake then explicit transfer; no new human gate. Product/Full/Linux/readable/ARC acceptance pending. This does not reopen/extend S04. DRAFT descriptions above remain history; this checkpoint governs issue status.

## 12. Frozen iteration01 review and proposed iteration02 — 2026-10-27 UTC

Current c364 has independent API/general/edges/54CLI semantic success at exact stated populations; evidence and partial fixed findings are in artifacts/review-iteration01.md. F02 still needs known-overlay check moved AFTER source uniqueness and actual compact-targeted regression (new unit only tested full); F03 still needs true optional per-region instrumentation/outer cleanup, not equation-only acceptance. Original criteria/proof/owners/statuses remain unchanged.

DRAFT cc-prompt-iteration02.md is proposed pass3of5 and freezes SIX paths/private signatures/timer inventory/conditional ordinary calls/safe formatter route. Root review before issue; no extra diagnosis or numeric/Full campaign. Retain already-built primitive capture/occurrence/attribution/contributor/distributor ordinal maps and use them for input lookups; no body/digest cache or full engine. Output/body/selection/error semantics remain fixed. Public CLI/store/export/doc bytes stay unchanged in this repair, supporting explicit current-evidence retention rather than relabelled descendant tests.

Readable is now numbered03/plannedpass4, not abandoned or reset; actual usable Exact AND reference/detail format plus approved numerical/Linux/fresh assembled ARC gates still required before all-six/arc close. Earlier numbered01/02/readable references are historical issue-time sizing; original issued prompts stay byte-preserved. No external delivery permission inferred.

Historical source checkpoint retained explicitly: iteration01 began at5bf/tree7e78 after root mapped2209→d52b30cf and9eb→5bf with identical trees/range-diff equality onto merged main32e99 (tree432). Backup codex/archive/project01-s05-pre-main-9eb5c698 and artifacts/rebase01 retain originals. Original9eb author gates remain attributed there, never relabelled fresh5bf. Iteration01 then returned c364/treebcaf after the exact eight-path delta.

### Iteration02 final readiness and status-only issue —2026-10-27 UTC

Root FINAL ITERATION02 READINESS PASS, 2026-10-27 UTC: exact reviewed draft20159B/SHAa1a96b35fe2cefe56a674278263d78772eec146e35643e1af94307289b00b2f9, plan c073d688…a2fd, ledger b1bc3d97…df4a, review7469f9f2…a511. Root read complete current bodies and source-critical recipes; independently verified all369payloads/2,302,475B under manifest1ce3b61a…f6c34, comprising368 whole original-file matches plus137039B exact live execution prefix, no extras. The prior wording '369exact task-original payloads' is qualified by this whole-file versus prefix distinction. Six binding criterion/proof/owner/status cells remain unchanged OPEN; quiet c364 parent5bf/treebcaf, two parsed trailers. Exact reviewed draft is preserved separately; active prompt changes ONLY line3 status. Corrective pass3of5, readable03/plannedpass4; all product/Full/Linux/ARC/external acceptance remains pending. Complete retained CC intake must explicitly cover WHOLE slice-plan §§1–12, including prior omitted §§1–2/6 history, before fresh source/lock/guard/refs check and CDC explicit SIX-path/index/ONE-target transfer. No new human approval gate or source work at issuance.
Issued prompt SHA256 `50cc14c778603405f3031abfb868f330f72a1dac17a639e6b8dd060261bd62f7`. Earlier DRAFT/pending descriptions are preserved historical checkpoints.

### Iteration02 complete intake accepted / authorized handoff —2026-10-27 UTC

CDC accepts the complete iteration02 intake/readback after reading the exact appended original and factual erratum. CC explicitly loaded WHOLE current plan §§1–12 and all issued required bodies/source functions, verified369payloads/2,302,475B; prior omitted sections remain historical and are not retroactively relabelled read. Correct c364 capture/static CLI negatives remain PASS, not a new repair. A further wording calibration: the erratum's 'retains primitive ordinal maps' describes the ISSUED target design; current c364 builds then discards several maps. Retain/use them is required implementation, not already completed source evidence. Source timing regions/None routing/inner cleanup remain substantive acceptance discriminators, never equations alone.

Immediately before transfer CDC independently rechecked clean c3643f4915b7876313dc3a58d8c56c87cf38fa5c/treebcaf63c9ec11b18ea23049ca10921b5d8e06da32/S05 branch, lock65cd7f39…8700/guard20fe7ba7…de8 and BOTH old S04refs432d632 unchanged. Upon the explicit CDC transfer message following this scoped metadata commit, retained Luna/high CC exclusively owns ONLY the SIX issued source paths, source index and ONE retained Arc01 target. CDC/root become source/index/target read-only until frozen handback; CDC continues planning/independent preparation only. CC may append existing execution.md only outside source. Pass3of5/all6rowsOPEN/readable03plannedpass4/FullnumericLinuxfreshARCexternal holds unchanged. No source operation occurred during intake.

## 13. Iteration02 bounded disposition and mandatory readable03 carryover —2026-10-27 UTC

Frozen739 compact/detail semantic/API/finiteTiny contribution PASS; strictF03 ONE OPEN pure output String clone in compact_hit. Root explicitly makes its exact move-or-real-timed-clone repair FIRST readable03/pass4 item within original21path union/five-total budget. INITIAL not unconditionally accepted, all6OPEN/noFullmetriceligibility. Current independent API0/public29namedPASS+shared6/5/edges11namedPASS+shared5/privateindex1/1; WHOLE ordinary/profile Exact+Local+Catalog parity across four modes and managedfirstSome/warmNone PASS. CDC owns quiet739/source/index/target. Existing review-iteration02 and immutable cdc-iteration02 preserve exact scope; no new runtime/external authority.

Historical review offered move-or-real-timed-clone. Current source-grounded03 recipe is definite: after final occurrence_attributions.get(&occurrence_key), MOVE existing String into occurrence_id; it has no later key use. Remove unnecessary clone; no new timer. No key/output bytes change or silent739edit; next frozen source must close it before numeric trials. Readable03 concrete goldens/typed transport first returns to root.


## 14. Readable03/pass4 concrete contract — DRAFT 2026-10-27 UTC

This section and cc-prompt-iteration03.md form the proposed ELEVEN-path readable increment at quiet7399563/treee184. Root accepted the initial engineering spine as policy, not final readiness or issue. Six original criterion/proof/owner/status cells remain OPEN and unchanged; pass4ofFIVE TOTAL, no reset. Sections1–13 and all issued prompt bytes remain historical/binding except these explicit current sequencing/format refinements. INITIAL semantic/API/Tiny contribution is bounded PASS; strictF03 remains OPEN until the first source change below is frozen and inspected. No Full metric eligibility yet.

### 14.1 Exact carryover and typed transport

In dictionary/projection.rs::compact_hit, remove `let occurrence_id = occurrence_key.clone();`. Perform the existing final `self.indices.occurrence_attributions.get(&occurrence_key)` lookup, then `let occurrence_id = occurrence_key;`. That tuple String has no later key use; move it into the existing timed returned-hit construction. No clone/timer fallback, representation/key change or other engine edit. The remaining TEN paths implement the four pure typed renderers, additive readable routing, tests and owning docs exactly as the draft prompt lists. Current source/ownership is739/CDC quiet, not c364/CC. The inherited full path and strict compact wire stay unchanged.

The CLI chooses Full/Compact/Readable before constructor/setup. Detail has Full/Readable only. Pure renderers take existing typed compact/detail data; direct typed serde for Presence, reference/catalogue values and complete DictionaryHit, no Value roundtrip. The private QueryOutput Json/Readable boundary writes readable String bytes once and flushes through the existing shared BrokenPipe/nonBroken handler. Managed context is captured from the typed result first. The NEW readable standalone dictionary-query origin uses the existing three-key dictionary error report on response-output IO; existing full/compact standalone compatibility and all managed envelopes remain intact. Explicit literal syntax and grouping in the prompt are binding.

### 14.2 Complete literal presentation fixtures and expected bytes

These are PURE PRESENTATION DATA, not accepted canonical dictionaries, Engine results or authenticated key/body digests. Public projection structs are caller-constructible. K deliberately supplies format-valid fixed digests, so a formatter test must not query/admit/recompute them. Construct the typed input structs corresponding to the values below, independently predeclare the exact UTF-8 expected strings INLINE in presentation_contract.rs, and compare renderer output byte-for-byte. Do not generate expected strings by rendering, Value roundtrip, trimming or newline normalization. Every fenced text body below includes its displayed final LF and no blank footer. Structural LF is separate from literal backslash-n inside JSON. The zero Exact golden contains TWO content lines, each ending LF.

K is the following exact one-line serde-shaped key, in declared field order; all nullable keys are present. No ellipses or substituted source-specific IDs in these literal tests:

```json
{"version":1,"digest_policy":"entry-assertion-serde-json-v1","role":"assertion","source":"source-a","entry_id":"entry-a","assertion_id":"assertion-a","occurrence":{"capture":{"namespace":"fixture:readable","sha256":"0000000000000000000000000000000000000000000000000000000000000000"},"locator":"/0"},"entry_sha256":"1111111111111111111111111111111111111111111111111111111111111111","assertion_sha256":"2222222222222222222222222222222222222222222222222222222222222222","native_entry_id":null,"native_assertion_id":null,"contributor_id":null,"distributor_id":null}
```

For nonempty Exact, input totals are4/1/1, one source id source-a abbreviation A, full_count4/returned_count3/partial=true, cursor Some("next-page"). The three hit records have SAME assertion ID/occurrence ID/key and the spelling/definition Presence states shown; duplicate occurrence display is deliberate, never deduplicated. SourceRecord's other fields and unused compact fields may use harmless typed defaults; they are not emitted. The third definition is complete TextValue {text: é plus LF plus quoted text, language:Missing, markup:Null}, never flattened. These intentionally independent presentation records do not purport to satisfy canonical per-occurrence uniqueness/body-hash invariants.

#### Literal Exact zero

```text
Exact "farvi": 0 occurrences, 0 spellings, 0 sources
next_cursor: null
```

#### Literal Exact nonempty / all Presence states / duplicate occurrences

```text
Exact "farvi": 4 occurrences, 1 spellings, 1 sources
A [source-a] 3/4 (partial)
  assertion-a @ 00
    spelling: "Missing"
    definition: "Missing"
    detail: {"version":1,"digest_policy":"entry-assertion-serde-json-v1","role":"assertion","source":"source-a","entry_id":"entry-a","assertion_id":"assertion-a","occurrence":{"capture":{"namespace":"fixture:readable","sha256":"0000000000000000000000000000000000000000000000000000000000000000"},"locator":"/0"},"entry_sha256":"1111111111111111111111111111111111111111111111111111111111111111","assertion_sha256":"2222222222222222222222222222222222222222222222222222222222222222","native_entry_id":null,"native_assertion_id":null,"contributor_id":null,"distributor_id":null}
  assertion-a @ 00
    spelling: "Null"
    definition: "Null"
    detail: {"version":1,"digest_policy":"entry-assertion-serde-json-v1","role":"assertion","source":"source-a","entry_id":"entry-a","assertion_id":"assertion-a","occurrence":{"capture":{"namespace":"fixture:readable","sha256":"0000000000000000000000000000000000000000000000000000000000000000"},"locator":"/0"},"entry_sha256":"1111111111111111111111111111111111111111111111111111111111111111","assertion_sha256":"2222222222222222222222222222222222222222222222222222222222222222","native_entry_id":null,"native_assertion_id":null,"contributor_id":null,"distributor_id":null}
  assertion-a @ 00
    spelling: {"Value":"farvi"}
    definition: {"Value":{"text":"é\n\"quoted\"","language":"Missing","markup":"Null"}}
    detail: {"version":1,"digest_policy":"entry-assertion-serde-json-v1","role":"assertion","source":"source-a","entry_id":"entry-a","assertion_id":"assertion-a","occurrence":{"capture":{"namespace":"fixture:readable","sha256":"0000000000000000000000000000000000000000000000000000000000000000"},"locator":"/0"},"entry_sha256":"1111111111111111111111111111111111111111111111111111111111111111","assertion_sha256":"2222222222222222222222222222222222222222222222222222222222222222","native_entry_id":null,"native_assertion_id":null,"contributor_id":null,"distributor_id":null}
next_cursor: "next-page"
```

The Resolve input below has target Local Assertion(assertion-a), status Unverified, entity None; one confirmed source-a/A row with the shown lexical body/K, and one unverified source-z row absent from availability with the shown Excluded/Null/Missing state and detail None. First matching availability abbreviation for source-a is A; include a SECOND matching source-a availability with label "wrong" to prove first-match policy. No source-z availability exists: use literal ? fallback, keep the row. Independently add an input with two source IDs in reverse order and repeated rows to assert ascending SourceId grouping and stable within-group multiplicity separately from the literal single-row groups. Both classification counts always appear.

#### Literal Resolve confirmed + unverified/excluded/fallback

```text
Reference {"Local":{"Assertion":"assertion-a"}}: Unverified
entity: null
confirmed: 1
A [source-a] 1 confirmed
  assertion-a @ 00
    spelling: {"Value":"farvi"}
    definition: {"Value":{"text":"é\n\"quoted\"","language":"Missing","markup":"Null"}}
    eligibility: "Lexical"
    payload_present: true
    current_eligible: true
    detail: {"version":1,"digest_policy":"entry-assertion-serde-json-v1","role":"assertion","source":"source-a","entry_id":"entry-a","assertion_id":"assertion-a","occurrence":{"capture":{"namespace":"fixture:readable","sha256":"0000000000000000000000000000000000000000000000000000000000000000"},"locator":"/0"},"entry_sha256":"1111111111111111111111111111111111111111111111111111111111111111","assertion_sha256":"2222222222222222222222222222222222222222222222222222222222222222","native_entry_id":null,"native_assertion_id":null,"contributor_id":null,"distributor_id":null}
unverified: 1
? [source-z] 1 unverified
  assertion-z @ 01
    spelling: "Null"
    definition: "Missing"
    eligibility: {"Excluded":{"reason":"not lexical"}}
    payload_present: false
    current_eligible: false
    detail: null
requested_release: null
```

#### Literal Resolve Unknown

```text
Reference "Unknown": Unknown
entity: null
confirmed: 0
unverified: 0
requested_release: null
```

The following managed CatalogueOnly fixture is also pure presentation data (empty captured policy snapshot rather than a claim about actual removal). Outer target/status/reason/requested_release are authoritative; payload=None. Its input snapshot is exactly the literal line; collections/catalogue_matches empty. A separate paired payload=Some fixture deliberately disagrees on nested target/status/requested_release and must render OUTER values with all nested views retained; expected fields are direct typed serde, not fallback or JSON-shape inspection. Actual removed/current selection semantics belong the process journey in14.3.

#### Literal managed metadata-only Resolve

```text
Reference {"Local":{"Assertion":"assertion-a"}}: CatalogueOnly
reason: "removed"
payload: false
entity: null
confirmed: 0
unverified: 0
requested_release: null
catalogue_matches: []
selection: {"version":1,"profile":null,"local_revision":0,"generation_id":null,"content_id":null,"recorded_state_sha256":null,"packs":[],"disabled":[],"selected_releases":{},"collections":[],"selection_policy":"selected-available-owned-head-v2","order_policy":"source-release-occurrence-v1","projection_policy":"compact-dictionary-v1","binding_sha256":"3333333333333333333333333333333333333333333333333333333333333333"}
collections: []
```

#### Literal detail NoMatch / hit null

```text
Detail assertion-a: NoMatch
snapshot: "3333333333333333333333333333333333333333333333333333333333333333"
reason: "lexical_identity_changed"
current_eligible: null
reference: {"version":1,"digest_policy":"entry-assertion-serde-json-v1","role":"assertion","source":"source-a","entry_id":"entry-a","assertion_id":"assertion-a","occurrence":{"capture":{"namespace":"fixture:readable","sha256":"0000000000000000000000000000000000000000000000000000000000000000"},"locator":"/0"},"entry_sha256":"1111111111111111111111111111111111111111111111111111111111111111","assertion_sha256":"2222222222222222222222222222222222222222222222222222222222222222","native_entry_id":null,"native_assertion_id":null,"contributor_id":null,"distributor_id":null}
hit: null
```

The complete detail golden uses K and fixed snapshot333… as above, status Resolved/reasonNone/current_eligibleSome(true). The supplied DictionaryHit is exactly the following nonempty lexical/capture/raw typed body: no renderer constructs it. Its source_score is Presence::Value(RawJson::new("1e+9999")); RawJson's transparent serializer emits the retained lexeme as a JSON STRING, not a number or Value normalization. This deliberately small full-hit literal pins ALL existing DictionaryHit fields, their declared order, null/empty distinction and typed pretty indentation. It is NOT a substitute for the separate accepted complete_fixture Forms/editorial/contexts test below.

#### Literal complete detail / direct typed pretty DictionaryHit

```text
Detail assertion-a: Resolved
snapshot: "3333333333333333333333333333333333333333333333333333333333333333"
reason: null
current_eligible: true
reference: {"version":1,"digest_policy":"entry-assertion-serde-json-v1","role":"assertion","source":"source-a","entry_id":"entry-a","assertion_id":"assertion-a","occurrence":{"capture":{"namespace":"fixture:readable","sha256":"0000000000000000000000000000000000000000000000000000000000000000"},"locator":"/0"},"entry_sha256":"1111111111111111111111111111111111111111111111111111111111111111","assertion_sha256":"2222222222222222222222222222222222222222222222222222222222222222","native_entry_id":null,"native_assertion_id":null,"contributor_id":null,"distributor_id":null}
hit: {
  "occurrence_id": "00",
  "entry": {
    "id": "entry-a",
    "source": "source-a",
    "spelling": {
      "Value": "farvi"
    },
    "language": "Missing",
    "script": "Null",
    "native_word_class": "Missing",
    "selmaho": "Missing",
    "mapped_pos": "Missing"
  },
  "assertion": {
    "id": "assertion-a",
    "entry": "entry-a",
    "occurrence": {
      "capture": {
        "namespace": "fixture:readable",
        "sha256": "0000000000000000000000000000000000000000000000000000000000000000"
      },
      "locator": "/0"
    },
    "native_sense_id": "Null",
    "language": "Missing",
    "definition": {
      "Value": {
        "text": "é\n\"quoted\"",
        "language": "Missing",
        "markup": "Null"
      }
    },
    "source_score": {
      "Value": "1e+9999"
    },
    "timestamps": [],
    "eligibility": "Lexical"
  },
  "capture": {
    "source_name": "Fixture é",
    "format_label": "json",
    "locator": "memory:readable",
    "revision": "Missing",
    "captured_at": "Null",
    "time_precision": "Unknown",
    "notice_locator": "Missing"
  },
  "raw": {
    "locator": "/0",
    "fields": [
      {
        "name": "score",
        "encoding": "Json",
        "raw_value": "1e+9999"
      }
    ],
    "raw_line": null
  },
  "snapshot_attribution": {
    "snapshot": {
      "namespace": "fixture:readable",
      "sha256": "0000000000000000000000000000000000000000000000000000000000000000"
    },
    "source": "source-a",
    "distributor": null
  },
  "occurrence_attribution": null,
  "contributor": null,
  "distributor": null,
  "release_ids": [
    "release-a"
  ],
  "contexts": [],
  "frames": [],
  "forms": [],
  "etymologies": [],
  "relations": [],
  "editorial_senses": [],
  "computations": [],
  "annotations": [],
  "extensions": [],
  "owned_revision_ids": [],
  "tombstoned_by": []
}
```

Supplementary complete_fixture test is mandatory: use crates/lexicon/examples/support/dictionary_fixture.rs::complete_fixture, preserving its actual nonempty Forms/editorial_senses/contexts and all complete DictionaryHit fields through existing strict construction/detail. Use the actual si'e assertion hit for nonempty Forms+contexts and the captured:a:d1 /0 farvi assertion hit for nonempty editorial_senses, derived from fixture input IDs BEFORE candidate detail; these are DIFFERENT hits. Assert their respective expected collections nonempty BEFORE formatter comparison. Do not demand all three on one hit or attach editorial data to si'e merely for the test. Expected body is independently DIRECT serde_json::to_string_pretty(&known_hit), compared whole with the literal prefix/reference metadata and one LF. This is a typed-serializer body comparator, NOT falsely described as an all-literal nested-fixture golden; the complete literal small hit above remains separate. Add direct typed Presence/TextValue/RawJson equality assertions to distinguish struct key order, retained lexical strings and duplicate Vec rows from Value-roundtrip behavior.

All header atoms use A = direct JSON-string CONTENT, stripping only the enclosing quotes; all labelled values use complete J. In a dedicated input put ESC (U+001B), CR, tab, LF, backslash, quote and visible é in source abbreviation, source ID where typed constructor permits opaque content, assertion/occurrence IDs and spelling/definition. Expected literal escapes are respectively backslash-u001b, backslash-r, backslash-t, backslash-n, double-backslash and backslash-quote; é stays UTF-8. Never emit a literal ESC/CR/tab from corpus fields. If a typed ID rejects one control, preserve the rejection and exercise that control in the remaining unconstrained header String fields; do not weaken ID validation. No trimming, ANSI, wrapping, width probe, singularization or extra footer LF.

### 14.3 Actual W1–W5 Tiny process journey / frozen argv and revisions

This is ONE fresh owned Tiny four-literal journey, not Full or a new comparative trial. It uses all four accepted source-literal models BEFORE installation: exact portable source recipe from S03 slice03-configured-combined-lookup/artifacts/cdc-review02/public-caller01/src/helpers.rs::models and ::inventory, with its three literal fixtures copied INLINE into the allowed existing CLI test (no planning include path). The original recipe emits models [JB,LLG,La,Sut]. Strictly reconstruct/seal each Synthetic input with existing seal_release and load_sealed_release, retain exact release/seal bytes, input IDs/digests and source-native attribution BEFORE invoking candidate CLI. Derive max-multiset25-family/capture expected inventory directly from individual inputs; six raw occurrences/five lexical farvi and three logical sources with2/1/2 hits, pack contributions1/1/1/2. Candidate composition/order may not manufacture expected preservation. Stock data-pack fixture's six farvi hits is supplementary, not a replacement.

Let B be a uniquely created private test base, R=B/root initially ABSENT, F=B/config.toml initially ABSENT, H=B/home, D=B/xdg-data, C=B/xdg-config (owned private existing directories). Every child clears inherited environment and sets HOME=H, XDG_DATA_HOME=D, XDG_CONFIG_HOME=C, with absolute paths and explicit ROOT/STORE or --config F on managed commands. No real default access even read. Read-only negative missing-HOME tests may remove the tested variable while retaining an owned XDG_DATA_HOME fallback and explicit paths; do not let env_clear-only accidentally reach platform passwd home. Mutation CLI commands such as install/remove do NOT accept --config; their positional R and owned HOME/XDG provide isolation. Lookup/config/profile routes explicitly carry --config F. B's current executable is env!(CARGO_BIN_EXE_vlajmaji), not a shell/bare PATH binary. Each argv cell below is an exact argument VECTOR template (angle-symbols replaced by the owned native OsString/value specified); do not shell-split JSON/paths. No invented --root or --set-default flag.

Write M=B/manifest.json with EXACT shape {"version":1,"inputs":[{"kind":"sealed","pack_id":"lensisku-jbotci","release":"packs/lensisku-jbotci/release.json","seal":"packs/lensisku-jbotci/seal.json"},{"kind":"sealed","pack_id":"llg","release":"packs/llg/release.json","seal":"packs/llg/seal.json"},{"kind":"sealed","pack_id":"la-lojban","release":"packs/la-lojban/release.json","seal":"packs/la-lojban/seal.json"},{"kind":"sealed","pack_id":"sutysisku","release":"packs/sutysisku/release.json","seal":"packs/sutysisku/seal.json"}]}. All release/seal paths relative to M's parent. Do not substitute a packs map or omit Synthetic opt-in.

Precompute A_LLG from the LLG input's single lexical farvi Assertion ID, Q_L={"operation":"resolve","target":{"Local":{"Assertion":A_LLG}}}, and Q_C={"operation":"resolve","target":<typed CatalogTarget serialized directly from that input's source/release/capture/locator/native attribution>}. Q_C is built from the pre-input exact roles, not selected from candidate response; serialize the existing RelationTarget directly without hand-invented Catalog keys. E={"operation":"exact","spelling":"farvi"}; E1 adds options:{page_size:1}; default remaining options via strict parser. D_LLG=pre-input seal.release_sha256 (exact sealed canonical release SHA; crosscheck load_sealed_release). L=B/packs/llg/release.json and T=B/packs/llg/seal.json. K_LLG is the CLI returned compact key checked against the PRE-INPUT entry/assertion/capture/native identity and complete typed body digest; obtaining its opaque binding/digest encoding from the response is not the expected lexical oracle. Save O=observed managed compact snapshot.binding_sha256 (expected_snapshot for managed detail), and CUR1=compact page1 cursor. Both are opaque observed tokens used only for declared stale/positive tests. G1_STORE is decoded from the successful install JSON generation.path NativePathReport, NOT generation.store_path (the Rust store_path field is serde(skip)). For this normal UTF-8 B use generation.path.utf8 as the exact absolute STORE; when a separate native-path control needs it, decode that report's declared encoding/hex using the existing native adapter and compare original native bytes. Check native containment/absence-before-create/expected generation and pin G1_STORE for explicit standalone comparisons; do not guess layout or select a default.

| Step / workflow | EXACT argv after executable | Expected exit / content | Catalogue revision / profile-file revision AFTER |
| --- | --- | --- | --- |
| W1 install | ["collections","install",R,"--manifest",M,"--all","--allow-synthetic"] |0; one all-four committed batch, G1 all4; exact pre-input inventory/pins and five-hit oracle. Fresh schema seeds0 then batch increments ONCE. |1 / absent |
| W2 list | ["collections","list",R] |0; schema2, four registered/installed, G1; do not mistake producer source for PackId. |1 / absent |
| W2 inspect each | ["collections","inspect",R,P] for P in [lensisku-jbotci,llg,la-lojban,sutysisku] |0 each; actual release/seal/ownership/verification scope per record. |1 / absent |
| W3 configure | ["collections","configure",R,"--profile","work","--all","--make-default","--config",F] |0 committed; all4 member/default enabled; fresh file revision1. |1 /1 |
| W3 profiles / selection | ["collections","profiles","--config",F]; ["collections","selection","--config",F,"--profile","work"] |0; default=work/all4; actual catalogue1/profile1 and projection/policy fields. |1 /1 |
| W3/W5 default compatibility | ["lookup","farvi","--config",F]; ["lookup","farvi","--config",F,"--format","full"] |0; default and explicitFull complete JSON byte-identical at unchanged captured state; five occurrences/three sources/2-1-2. |1 /1 |
| W5 compact/readable | ["lookup","farvi","--config",F,"--profile","work","--format","compact"]; same vector final "readable" |0; complete projected typed wire vs exact readable grammar, same five occurrences/all alternatives/totals/provenance; no full engine for readable. |1 /1 |
| W5 standalone compatibility | ["dictionary-query",G1_STORE,"--request",E]; same vector with ["--format","full"], ["--format","compact"], ["--format","readable"] |0; default/explicitFull bytes equal; full compact semantic identity and readable typed expected; explicit immutable STORE only, no config lookup. |1 /1 |
| W5 Local/Catalog reference | ["lookup","--request",Q_L,"--config",F,"--profile","work","--format","readable"]; same with Q_C; ["dictionary-query",G1_STORE,"--request",Q_L,"--format","readable"] and Q_C |0; BOTH live pre-remove Q_L and Q_C MUST be Resolved with EXACTLY one confirmed/zero unverified: the pre-input single lexical LLG assertion, every supplied identity known. Compare its exact assertion/entry/capture/locator/native slots and requested release; never accept candidate-observed status as oracle. Readable is reference output, not fake Exact. Keep full/compact paired current routes where original fixture comparator applies. |1 /1 |
| W5 detail / page token | ["lookup-detail","--reference",K_LLG,"--config",F,"--profile","work","--format","readable"]; ["dictionary-detail",G1_STORE,"--reference",K_LLG,"--format","readable"]; ["lookup","farvi","--config",F,"--profile","work","--page-size","1","--format","compact"] |0; complete hit fields, key identity, managed optional observed O positive; capture CUR1 and managed snapshot token O. No automatic detail hydration in renderer. |1 /1 |
| W4 disable | ["collections","disable","--profile","work","--pack","llg","--config",F] |0 committed profile change only; installed bytes unchanged. |1 /2 |
| W4 disabled query/cursor | ["lookup","farvi","--config",F,"--profile","work","--format","readable"]; compact counterpart; ["lookup","farvi","--config",F,"--profile","work","--page-size","1","--cursor",CUR1,"--format","compact"] |0 fresh: four hits/two sources LSK2+Sut2;1 cursor: typed stale before any unavailable/payload result. Preserve all current readonly context. |1 /2 |
| W4 enable | ["collections","enable","--profile","work","--pack","llg","--config",F] |0 committed; restored five hits/three sources on fresh readable+compact argv above, original release IDs unchanged. |1 /3 |
| W4 remove | ["collections","remove",R,"--pack","llg","--expected-revision","1"] |0 committed; catalogue2/G2 three-pack active subset, all immutable old inputs/G1 retained; file policy still includes enabled LLG. |2 /3 |
| W4 removed Exact | ["lookup","farvi","--config",F,"--profile","work","--format","readable"] |1 typed unavailable/removed, no successful four-hit/zero fallback because enabled requested LLG is now unavailable. |2 /3 |
| W4 removed reference/detail | ["lookup","--request",Q_L,"--config",F,"--profile","work","--format","readable"]; same with Q_C; ["lookup-detail","--reference",K_LLG,"--config",F,"--profile","work","--format","readable"] |0 metadata-only CatalogueOnly/removed and hit:null, exact source/release/role/key retained. ["--expected-snapshot",O] paired on detail returns1 typed stale BEFORE admission. |2 /3 |
| W4 exact reinstall | ["collections","reinstall",R,"--pack","llg","--release",L,"--seal",T,"--allow-synthetic","--release-sha256",D_LLG,"--expected-revision","2"] |0 committed; exact pre-input release restored in G3 all4, no wrong-release retarget. |3 /3 |
| W4/W5 restored readback | Repeat fresh all-four readable/compact/reference/detail vectors, plus expected_snapshot O negative |0 fresh: five hits/three sources and complete stable-key detail;1 oldO: typed stale. Old G1 explicit STORE still gives its original read result. |3 /3 |
| W2/W4 reconcile | ["collections","reconcile",R]; list/profiles/selection above |0; logical revision unchanged, G3active/3retained generations, supported committed install/remove/reinstall stages not false attention; config remains revision3. No cleanup. |3 /3 |

No extra mutating setup/noops precede this table. Assert each expected revision BEFORE next dependent mutation and after every listed transition; do not derive next expected value from returned candidate revision. Read-only commands MUST leave revisions/owned state unchanged. The exact count above is ONE batch install + remove + reinstall = THREE Tiny publications, not three Full generations, and configure/disable/enable = THREE config revisions. No register step or rebuild is silently inserted. If an existing API's exact wire differs from this recipe, preserve failing setup receipt and resolve it source-first before proceeding; do not relax oracle from observed output.

### 14.4 Applicability, outputs and readiness hold

The eleven-path scope is exactly the prompt's list, within approved21 union; only dictionary/projection.rs changes engine code and only the occurrence String move. Existing library projection/selection/state/profile tests verify output identity and actual source timer boundary; no extra kernel audit. The readable/presentation library and real CLI proof includes reference as well as Exact and complete detail, all statuses, direct typed serde, no second LF, common output IO and path/context precedence. Author gates use pinned Rust1.98.0, one retained target, unique labels, safe stdin formatter, lint-before-behavior, ordered final fmt-check→lint→check, isolated no-default, Syntheticallfeatures and search/CLI list-only package inventories. Actual Linux-specific nonUTF8 CONFIG/ROOT and write-open /dev/full remain external until separately authorized current-head CI; macOS conditional controls do not prove those branches.

The prohibition is on RETAINED FULL/G4 reads/models/import/create/reverification/All and ANY numeric campaign during this issue, not on the explicitly approved owned Tiny fixture preparation/install/remove/reinstall and their inherent existing verification (including Tiny internal All). No new Full ROOT/generation, resource experiment/cache action/actual-default access/network/external delivery. Later exact retained-G4 numeric/read-only/fresh assembled ARC gates remain mandatory with separate concrete readiness; this pass does not launch or waive them. Root concrete final packet review is pending; no source transfer, issue or runtime from DRAFT metadata. Once root accepts, preserve reviewed draft/status-only issue/scoped commit/full intake then explicit eleven-path/index/ONE-target handoff. All six rows remain OPEN.

### Readable03 final readiness and STATUS-ONLY issue —2026-10-27 UTC

Root FINAL READINESS PASS at exact reviewed prompt20032B/SHA2cd8dcddce30f96e036318d2329ca186c2117b15f7a2b3fb25736d37b719cda4, plan111856B/SHA6715efadd5e7bee52a7f14ec258f5365e7eadc2d136728384afd1b685f61774a and ledger11355B/SHA34acf432a188ebbd736fa3844f04b436bd28a1735775483541567f5ad468c36e. Root read complete prompt/§14 literal+workflow appendix/ledger and source-checked typed hit/RawJson/native path/Clap/seal/config+catalogue revision boundaries; six original cells unchangedOPEN, source739 clean. Accepted exact source-grounded clarification replaces historical move-or-time possibility with definite String MOVE; no criterion/policy change.

CDC author readiness: source-grounded actual739 APIs/wire/lifetime/Clap and fixture attachments inspected; design-complete pure typed Exact/Resolve/detail grammar/goldens/IO+revision transitions settled; PF12/13 borrowed projection/direct typed serde, EH04 existing error contexts and OB16 explicit output/engine ownership applied; executable eleven-path recipe/current fixtures+one target/pinned1.98 fit existing homes; falsifiable literal full-wire/Presence/control/LF/source-group/strict IO/native/stale/readback controls with pre-input expected IDs; coherent complete plan1–14 and exact prior evidence manifest/current-source distinctions, all six rowsOPEN. Sketches/literal authored data are not compiled product proof.

Current assignment is `cc-prompt-iteration03.md` (Project01/Arc04/S05), issued2026-10-27 UTC; predecessor `cc-prompt-iteration02.md` is returned for review, bounded semantic/API/Tiny contribution PASS at739 with ONE strictF03 carryover. Initial/01/02 prompts stay byte-preserved. Exact reviewed draft copied create-new to artifacts/cc-prompt-iteration03-reviewed-draft01.md; active prompt SHA05cc094c1444d96a9ff37eab904400ddd6f97f0dced4f9e14b7bfc22268640c7 differs ONLY line3STATUS. Pass4ofFIVE TOTAL, not a reset. Retained Luna/high CC must now load COMPLETE required prompt/plan§§1–14/ledger/evidence/source extent, append exact coverage+source-cited readback to existing execution.md, then wait for CDC review/fresh source739/treee184/lock65cd…8700/guard20fe…de8/oldS04refs432 checks and explicit ELEVEN-path/source-index/ONE-target transfer. Issue metadata alone transfers nothing. No new user gate for routine authorized intake/transfer; no source/Full/numeric/Linux/external acceptance or action from this issue. Root/CC read-only while CDC owns quiet source/index/target.

### Readable03 complete intake accepted / explicit ownership handoff —2026-10-27 UTC

CDC read the COMPLETE new CC intake/readback in execution.md lines529–EOF, full-file SHAdcbfe9f6296735552dd537b0858e5ae8c3e8069892de85a42b6ec004d511166f. The record and direct CC messages attest actual complete prompt140lines/plan§§1–14 including recovered truncated tail/ledger/review/predecessor history, named parent/guide/source/API/CLI/docs/fixture extents and required-data114payloads/1,110,977B zero hash/size discrepancies. Intake identity matches actual issued20060B/SHA05cc094c…640c7, current plan SHAd6fa05b4…7f9dc and ledger b6667a31…4d2f3. Reading coverage/readback are CC attestation, not independent product acceptance. CDC source-cited review found the readback coherent with typed grammar/directserde/IO/state/pre-input oracle/Full holds and exact eleven-path scope. One initial sentence inverted CURRENT clone order and TARGET move order; CC explicitly corrected it in the actual record BEFORE any source edit. Current739 clones before get; target removes clone and moves String after final lookup. Prior intake omissions stay historical and are not retroactively relabelled read.

CDC fresh read-only boundary immediately before this metadata handoff reverified clean7399563f98b41d26bd5099d04873e1a8cc7e23fe/treee1840807bba597e779a1b28d02937ceff9ffc4ab/correct S05branch, unchanged Cargo.lock65cd7f39…8700/guard20fe7ba7…de8, BOTH local+origin oldS04refs432d632. No source/index/target/build/store/default action occurred during intake. Root final readiness already authorizes this routine transfer; no further root or human gate.

EFFECTIVE ONLY on CDC's explicit transfer message after this scoped planning commit, retained Luna/high CC owns exactly the ELEVEN issued source paths, source index and ONE retained Arc01 target. CDC/root become SOURCE/INDEX/TARGET READ-ONLY until actual clean frozen handback; CDC retains planning/design/independent preparation, CC may append only existing execution.md outside source. Source739 is the accepted author BASELINE, not a future claim that the author's WIP is clean. First exact occurrence String MOVE, then concrete pure renderer/typedCLI/realTiny W1–W5 delivery and required current ordered gates. Pass4ofFIVE TOTAL/all6OPEN/noFullmetriceligibility until strict carryover closes; no Full/numeric/Linux/external action from transfer. Existing protected data/default/refs/one-target boundaries unchanged.

## 15. Readable03 frozen disposition and final persistent-test completion —2026-10-28 UTC

See review-iteration03.md and canonical artifacts/cdc-iteration03/manifest.json for exact scopes/denominators/history. Frozen5e is clean/exact11/cumulative21/two trailers; source/CLI/projection/renderer semantic/API/finite Tiny PASS. Original strictF03 pure occurrence clone is CLOSED by source-exact String MOVE after last lookup. Root independently reviewed frozen critical source and found no further production/hosted-source compile issue; no actualLinux claim follows. Independent current public34named+shared6/5, edges11named+shared5,54strictCLI,64child original readable journey pass; two38child reviewer operation-label-method failures remain retained. Author current325/0/1+19, no-default76/0/0 transcript-only and first unchanged-fixture allfeaturefail +single327/0/1 retry remain calibrated with exact source/log identity. Prior pre-strengthening separatecheck claim is UNPROVEN, not another run.

### 15.1 Draft04/pass5 design and source scope

The SAME original §14.2/14.3 obligations require saved persistent regressions in THREE paths only: search/tests/presentation_contract.rs (exact mixed excluded literal, first matching A despite later wrong availability, reverse-source/multiplicity, actual nonempty page_size0 deterministic rendering); CLI main.rs ONLY existing managed_output_tests cfgtest body (successful body write→flush BrokenPipe for Json+Readable, preserved Err5/other counterparts); CLI tests/query_output_contract.rs ONLY existing original-four-literal journey+bounded helpers/imports (whole PRE-INPUT THIRTEEN-field K, returned NativePathReport+strict owned containment/no-follow BEFORE IO, missing originalG1 standalone mode/ref/detail and removed readable vectors). cc-prompt-iteration04.md contains exact source-compatible types/getters/control flow/oracles and recommended same fixture/no-new-publication route. No production bytes/API/wire/docs/deps/paths change, no general matrix. main's entire production prefix must stay EXACT5e. Borrow pre-input registry with as_document(); native key attributions match to_record_identity/to_snapshot_identity; use public CollectionReleaseSummary::from(&ReleaseRecord). No custom field projection or candidate-picked expectations.

Existing ten saved test names do not equal seven prescribed literals; existing write-BrokenPipe does not equal flush-BrokenPipe; later different complete_fixture commands do not equal missing original installedG1 vectors. Earlier completion attestation is explicitly corrected, not rewritten. Exact source path was observed owned; no product vulnerability/default access asserted. Independent64-child behavior is retained separately and does not substitute persistent obligations. Root's earlier key count14 was a counting mistake; named fields correct, actual13 (9+4nullable).

### 15.2 Proof/applicability/authority

Finalpass5 requires pinned scoped stdin-format→readonlyfmtcheck→lint BEFORE tests; actual updated presentation and managed_output tests plus CLI original-literal journey, stable makecheck and new presentation under no-default. Unique logs/actual exits/counts. Source-only production-prefix/delta equivalence supports retaining5e unrelated semantic/54/64/no-default/allfeature/package proof; actual descendant CURRENT-HEAD Linux/default/isolated/allfeatures remains later mandatory and not relabelled. No local blanketallfeature replay solely bookkeeping, no extra trial. Persistent original tests still NOTaccepted until executed and independently reviewed; all6rowsOPEN. Full/numeric, external delivery and fresh assembled Arc04 gates held. This remains finalpass5ofFIVE, no sixth automatic authority.

### 15.3 Assignment history / author readiness

Predecessor cc-prompt-iteration03.md is returned/frozen5e with bounded semantic/API success and stated persistent omissions, never rewritten. Current active source assignment NONE; CDC quiet ownership after actual handback. cc-prompt-iteration04.md is DRAFT ONLY, not issued; root final readiness pending. Source-grounded actual structs/getters/test boundaries and strict literal/real-emitter oracles are complete within original21 union; testing Proportionate Proof and CLI-36/37 support the bounded method. No new architecture or human decision required for this ordinary final test completion. Root reviews exact complete prompt/review/plan/ledger/manifest before status-only issue, retained full intake then explicit three-path ownership transfer. Numeric preparation stays source-only; no Full launch from this plan update.

### 15.4 Root FINAL readiness / STATUS-ONLY issue —2026-10-28 UTC

Root completely read exact20931B draft38c85297…0738, review e4f8ab17…14ce0, appended§15/fullledger; confirmed metadata-only prior-body delta/unchanged bindingcells, actual APIs/getters and all659payloads3,185,566B noextras/mismatch. Canonical manifest7d38f53f…e6dcf unchanged;656whole originals +187806B exact handback execution prefix +generated immutable Gitdiff/newsummary correctly distinguished. Initial root whole-file assumption for declaredprefix failed then corrected; guessed review artifacts path ENOENT corrected, no product/artifact defect. Source5e clean. Readiness PASS for FINALpass5 test-onlyTHREEpaths, no new human gate.

Reviewed draft byte-preserved create-new artifacts/cc-prompt-iteration04-reviewed-draft01.md; active cc-prompt-iteration04.md differs ONLYline3STATUS. Current assignment04 ISSUED, predecessor03 returned/frozen5e with scoped semantic success/test omissions. Earlier§15.3 DRAFT checkpoint is historical. Retained Luna/high must load COMPLETE required prompt/wholeplan§§1–15/ledger/review/evidence/source, record actual extent/readback in execution.md, then CDC freshidentity+readback review and explicitTHREEpath/index/ONEtarget transfer. Root read-only. All6rowsOPEN; source beforecfgtest prefixbitexact/noProductionDocsKernelMatrix change. No Full/numeric/CI/network/delivery from issue.

### Final04 complete intake accepted / conditional ownership handoff —2026-10-28 UTC

CDC read COMPLETE new CC intake/readback in execution.md (wholefile197680B/SHA c76d1f3231366681491e4dc072edb8a1de34d5deac70b1252dee1b780d7d8a1f before this append). Actual required coverage is CC attestation: issued20955B/73511b1d…bb15; WHOLE833lineplan§§1–15, ledger/review/bothAGENTS, required source bodies and testing/Rust guide sections;659payloads3,185,566B with exact187806B handbackprefix/provenance. CDC compared source-cited readback to actual13fieldkey/registrygetter/nativecontainment-beforeIO/mixedliteral/order/page0/realflush semantics and found it coherent. One shorthand named defaultFull among missing vectors; actual originalG1 default+Full ALREADY exists, preserved; missing Exact modes are Compact/Readable. CDC explicitly corrected this before transfer; prompt already correct. No completed existing control is reclassified as missing.

Fresh CDC read-only source check clean5e19754ce7ccad0ddea788c2e344df116711814c/tree4c91c02a9d66d58ec7e26c9323289f40a730ccde/S05branch; lock65cd…8700/guard20fe…de8 and both old S04refs432 unchanged. No CC edit/build/runtime during intake and all prior CDC tool sessions terminal. AFTER this scoped metadata commit, explicit CDC dispatch transfers ONLY THREE test paths (presentation_contract.rs, main.rs managed_output_tests cfgbody ONLY, query_output_contract.rs original journey+bounded helpers/imports), source index and ONE retained target to Luna/high CC. CDC/root then source/index/target read-only until clean frozen handback. CDC retains planning except CC append-only execution. FinalFIFTHpass, no automatic sixth; all6OPEN/no Fullnumericplatformexternal acceptance.
