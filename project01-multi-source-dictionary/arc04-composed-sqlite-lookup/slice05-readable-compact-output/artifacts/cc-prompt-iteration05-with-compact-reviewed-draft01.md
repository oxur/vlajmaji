# Project01 / Arc04 / S05 — content-validated Full reference metadata reuse

Status: ISSUED — root final concrete readiness PASS2026-10-28 UTC; complete intake and explicit transfer still required.

This is iteration05, the SIXTH total S05 implementation pass: initial,01,02,03,04,05. Duncan explicitly answered “Approve one correction pass and campaign” to the verbatim question in artifacts/authority-s05-extra-pass01.json. It permits this ONE correction of at most FOUR EXISTING files, required local/Tiny validation and ONE additional <=22-stage Full campaign under unchanged thresholds. No automatic seventh pass or retry, and no external push/CI/publication authority. Same retained Luna/high CC and Sol6.1/high CDC; Autonomous THROUGH ARC04, all six criteria remain OPEN. The failed numerical campaign is immutable, not retroactively accepted. No Full execution follows from this author prompt; CDC/root must review a NEW exact frozen campaign packet after the code and Tiny proof.

## Outcome / frozen baseline / four-path boundary

Current source is clean1b9038ddc6c45f3ea709a0bdfec64041221e3571/tree338daa526d9f1cd56813d0c68a41a3f258e8ec47, parent5e19754. Branch codex/project01-readable-compact-output, base main32e99; old local/origin S04refs432 remain immutable. Keep every public Full/Compact/Readable response and error category/shape unchanged. Reduce repeated typed parsing in managed Full metadata matching while continuing to read and fingerprint CURRENT registration bytes, and preserving stale/corrupt metadata detection. This is a source-grounded correction candidate, not a measured exclusive descriptor bottleneck or a guarantee of passing750ms.

Only these FOUR EXISTING source paths may change:

1. crates/store/src/collections.rs — opaque bounded metadata cache, two small additive cache APIs below, shared exact old decoder/row reader, focused private cfg(test) cache controls inside existing file. No lifecycle/schema behavior change.
2. crates/store/src/lib.rs — export ONLY the new named opaque CollectionReferenceMetadataCache alongside existing exports.
3. crates/search/src/collection_query.rs — private RefCell cache ownership, current-registration pruning and Full matcher borrowed descriptor reuse; bounded intra-call target entity/body ownership; existing profile/error/output policies preserved. No Compact/renderer/full dictionary kernel rewrite.
4. crates/search/tests/collection_query_contract.rs — persistent public Full exact/reference parity, same-reader byte-change/corruption/restore and existing stale/shared/history counterparts as specified below; reuse existing fixture and sqlite3 substrate.

No new source path, deps/lock, schema/config wire, importer/model, workflow, CLI/renderer/dictionary/projection.rs or other tests. In particular projection.rs's existing metadata_matches(&self) caller must keep compiling unchanged. No actual-default/Bauske/accepted-root mutation, new store/worktree/target, cache cleanup/reset, network/push/PR/merge/publication. One retained target only. No retained Full/G4 leaf/config/DB read/hash/query/import/create/All/trial by CC. Owned Tiny fixture preparation and internal existing verification are permitted for the named tests.

## Required reading / intake / ownership

Required FULL: this active prompt; current slice-plan.md WHOLE §§1–17; ledger.md; review-numeric01.md; source and planning AGENTS.md at their actual absolute roots below; authority-s05-extra-pass01.json; canonical numeric-failed01/numeric-full-output-review01.json; numeric-failed01/numeric-failure-correction-recommendation01.md. Original prompts initial–04 and immutable numerical freeze/failure manifests remain history. Complete means all bodies loaded in executing context, recover truncation; earlier missing-plan-extent intake mistakes are not repeated or erased.

Required source sections: collections.rs CollectionCatalogue/root/open/list/reference_metadata and CollectionReferenceMetadata/CollectionEntityReference/errors; all current collection_query.rs engine fields/constructors, metadata_matches, resolve_target, selected_payload_for_target, response/profile wrappers and representative-dedup tail; projection.rs existing metadata_matches caller READ ONLY; relevant full dictionary resolve/reference_views READ ONLY for output/error parity; COMPLETE collection_query_contract.rs fixtures and stale-heavy-metadata/shared history tests. Read these actual bodies, not just symbols. Existing whole file sizes are not permission to skim affected control flow.

Paths: source /Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc04/vlajmaji; planning /Users/oubiwann/lab/oxur/vlajmaji/.worktrees/planning; slice project01-multi-source-dictionary/arc04-composed-sqlite-lookup/slice05-readable-compact-output. ONE target /Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc01/target; guard /tmp/vlajmaji-p01-a04-cargo SHA20fe7ba7…c5de8; Cargo.lock65cd7f39…78700. No root/branch guessing.

Required guides: testing skill guides/01-testing-discipline.md Failure Triage/behavior contracts and guides/03-validation-gates.md execution populations/proportionate proof; rust-guidelines guides/04-ownership-borrowing.md OB-02/05/06/14/16/17/25 and guides/03-error-handling.md EH-01/04/10; engineering-methods approved prompt implementation and binding-existing-limits guidance already governing the slice. Roots /Users/oubiwann/Library/Application Support/ChatGPT-Profiles/personal/codex-home/skills/. Apply them concretely: no deep whole-descriptor clone to solve borrowing, no errors/results cached, primitive cache accounting is not measured heap, no implicit budget override.

Required DATA identity: hash/check EVERY canonical numeric-failed01 manifest payload size/SHA/noextra; read actual12fresh verdict, warm47rows/stderr/receipt/summary and execution-copy/outer/authority records. Seven bulky wire outputs stay held; no need to read them for intake and no actual DB/binary/Full data read. Prefix/source provenance is explicit. Explain observed13attempted/12checked versus absent14–22, and why root source inference does not time descriptor work. Append exact read extent/source-cited readback to existing artifacts/execution.md only, then WAIT for CDC fresh baseline/lock/guard/refs check and explicit FOUR-path/index/ONEtarget transfer. Issue/approval alone transfers nothing.

## A. Exact store cache surface and bounds

Recommended concrete public surface (names/signatures binding; private representation flexible):

```rust
pub struct CollectionReferenceMetadataCache { /* PRIVATE fields, no Clone */ }
impl Default for CollectionReferenceMetadataCache { /* empty, initially unbound */ }
impl CollectionReferenceMetadataCache {
    pub fn retain_registered(
        &mut self,
        catalogue: &CollectionCatalogue,
        registered: &[(PackId, String)],
    ) -> Result<(), CollectionError>;
    pub fn clear(&mut self); // drops cached entries/weight, RETAINS native root binding
}
impl CollectionCatalogue {
    pub fn reference_metadata_reusing<'cache>(
        &self,
        pack: PackId,
        release_sha256: &str,
        cache: &'cache mut CollectionReferenceMetadataCache,
    ) -> Result<std::borrow::Cow<'cache, CollectionReferenceMetadata>, CollectionError>;
}
```

CollectionReferenceMetadata already Clone supports Cow, but do NOT call clone/to_owned/into_owned on a cache hit. Borrowed hit; owned decoded fallback when not admitted to cache. The existing reference_metadata(&self,PackId,&str)->Result<CollectionReferenceMetadata,CollectionError> remains exact old behavior and signature. Share private current-row read and exact decode/header validation rather than duplicate or strengthen its rules. Keep old malformed/identity UnsupportedVersion strings/categories and SQLite/missing-registration errors; reference_projection's stronger dictionary/registry checks are NOT silently imported.

Private cache entry key is (PackId, exact sealed digest String); value is immutable parsed CollectionReferenceMetadata, SHA256 fingerprint of EXACT descriptor_json bytes, and byte weight bytes.len(). Also bind cache to catalogue.root's exact native PathBuf on first retain/helper use; different root is typed CollectionError::InvalidRequest, no rebinding or descriptor return. clear retains bound root. No lossy/display path conversions, canonicalization IO, inode guessing or process-global cache. The fresh catalogue open/revision logic remains the authority, not cache.root.

retain_registered takes current listing's keys, rejects wrong root, drops entries whose exact key is absent, and subtracts weight with checked invariants. It must NOT deserialize descriptors or load payloads. Duplicate keys are set membership only; no new error policy for listing duplicates. It does not authorize an absent SQL row: every helper still does actual query. Do not retain a second whole listing/descriptor buffer. The key count is bounded regardless of requested target or hostile repeated query identities.

Hard cache admission bounds: AT MOST128 retained parsed entries AND AT MOST536870912 (=512MiB) aggregate serialized-byte weight. Use checked arithmetic. This byte sum is a **serialized weight proxy**, NOT an actual Rust heap/RSS bound; typed metadata/string allocation can exceed it. Actual campaign RSS8GiB stays mandatory and includes cache. On over-entry or over-weight/overflow/one oversized descriptor, validate+return Cow::Owned WITHOUT inserting, evicting unrelated valid entries or changing the query/error. Do not fail a legitimate query solely because optimization budget is full. A changed existing entry subtracts/drops its old value BEFORE replacement admission; oversized replacement must not leave stale old state eligible. No LRU/request-result caching/hidden unbounded overflow slot. Clear/release drops entries/weight; allowed-key prune drops stale entries. cfg(test)-only adjustable private smaller limit constructor is permitted for tiny over-limit tests, production constants stay exact above.

For EVERY call, including hits, in this exact order:

1. Check/bind native root (cache API misuse is typed InvalidRequest); resolve current physical tables via existing tables_for; execute SAME current SQL pack/digest query yielding current payload_kind + descriptor_json. Missing/DB errors retain existing route. Never skip SQL because a key is cached.
2. Compute current exact descriptor SHA256, BEFORE any cached return. Do not hash serialized parsed objects. Current bytes are read even for same catalogue revision. A prior parse/fingerprint does not freeze later content or payload_kind.
3. On matching key+fingerprint, validate cached header version==1/pack_id/digest and cached payload_kind against CURRENT SQLpayload_kind using SAME existing check/error. SQL payload_kind changes with identical descriptor bytes must reject, drop invalid cached entry, and never hit. Avoid broad new kind/registry validation that old Full did not perform.
4. On byte change/miss, invalidate old entry, decode CURRENT bytes with exact old typed decoder/header checks, and do not cache errors. Only successful decoded immutable metadata can be admitted. Drop the transient bytes after validation/hash before returned borrow; do not retain raw+typed duplicates. Return borrowed admitted value or owned uncached fallback. Restoration after corrupt/current mismatch must recover exact response, no poisoned negative entry.

No caller data-dependent eviction can skip validation of another retained registration that old Full visited. No cache equality is a public semantic identity/body decision: complete typed entity/body ambiguity remains separate below. Expose no cache counters/diagnostics in public wire. cfg(test)-only private counters can count SQLread/fingerprint/decode/hit and proxy weight; they are method controls, not latency proof or global metrics.

## B. Preserve &self / borrow lifetime / Full matcher semantics

Add private `reference_metadata_cache: RefCell<CollectionReferenceMetadataCache>` to CollectionSearchEngine, empty in existing constructor(s). Keep metadata_matches(&self,...) and ALL existing callers/signatures. Existing engine contains rusqlite-backed dictionary provider, is Send/non-Sync; RefCell<Send cache> preserves Send and does not claim new Sync. Add compile-time positive Send guard if feasible inside these allowed modules; no unsafe/Mutex poisoning/global/threadlocal workaround. Do not make cache Clone or clone a descriptor to escape a borrow.

metadata_matches continues open-current-catalogue → captured revision equality → current list BEFORE accessing cache. Prepare/prune with only keys whose PackIds successfully parse (filter_map, NOT an early ?/error pass). Then KEEP the original parse/error point inside EACH registration scan. An invalid later pack must not preempt an earlier corrupt descriptor merely because cache preparation happened first. Pruning is bookkeeping, not validation or permission to skip any scan. Borrow RefCell mutably ONCE in this synchronous metadata scope. No callback, recursive metadata/helper call or payload setup while guard is live. Borrowed Cow/descriptors remain only within their individual processing iteration. Return only existing owned MetadataMatches fields plus bounded target comparison state; cache guard and transient descriptors must drop BEFORE ensure_payload/full resolve. Reentrant borrow is an internal invariant; do not introduce a new normal public error/category or suppress a real error with fallback cached output.

Full Catalog: SAME source/release/occurrence/native filtering, complete descriptor validation of EVERY old visited registration, requested-release ownership/error precedence, public row construction and release disambiguation. Full Local: SAME role/id scan and raw row order. Capture an owned clone of ONLY the FIRST MATCHED CollectionEntityReference and Option<body digest> as the comparison baseline. During later loop iterations compare borrowed candidate entity+digest and ACCUMULATE a mismatch flag, never early-return AmbiguousReference. Continue EVERY original registration SQL/read/decode in original order; only AFTER the complete scan at the old post-scan ambiguity boundary return AmbiguousReference when flagged. A later corrupt descriptor must still win over an earlier conflicting pair. This target-only state is not a deep descriptor/list/registry clone. Preserve old missing digest None-v-Some semantics. Avoid rereading first/additional matched descriptors after the initial pass; no miss/error shortcut for unrelated registrations old matcher read.

Keep old full Local native_entry_id/native_assertion_id None. Do NOT call compact matcher as a replacement: Compact fills native values and has different projection/error behavior. Preserve public first-row representative except current pack_id/release_sha256 preference, sorted contributing IDs, ALL original private pack/release admission scopes, complete-body ambiguity, and inherited admitted-release filtering. Do not cache target-specific matches/availability/response/full-hit/complete dictionary or skip corruption checks after cache warming. Full Exact payload/admission route, standalone/full eager/error semantics, compact setup/output and readable renderers stay byte unchanged.

Profiling unchanged: row SQL/read/hash/cache lookup/parse/classification/temporary cleanup remain lookup residual. Actual returned public clone/conversion blocks retain existing optional projection accumulator; metadata bookkeeping is NOT output projection. Whole managed Resolve total includes all this and cleanup;750ms maximum/250ms three-round median remain unchanged. Ordinary None introduces no new mandatory Instants. No stop-watch relabelling, zero placeholder or cached-success timing trick. Public wrappers/snapshot/collections/cursor/error bytes remain exact.

## C. Finite persistent controls (owned Tiny only)

Reuse existing valid fixtures and source-only declarations; no new corpus or test backend/dependency. Search already uses Command::new("sqlite3")/Python mutation scaffolding: use it for deliberate metadata faults and restore bytes. Product reads must have their own before/after DB-byte pair to prove no write; restoring fixture SQL can legitimately change SQLite page/header bytes, so do NOT demand whole DB equals the pre-interposition file after fixture writes. Preserve first failing fixture/oracle/command receipts, never retry by weakening predicates.

Store private controls, one finite test table/pairs:

- Initial miss exact metadata equality to uncached reference_metadata, second unchanged read returns Cow::Borrowed; private decode count unchanged, SQLread/fingerprint count increases. No deep descriptor clone. clear→next read redecodes; clear keeps root binding.
- SAME root/key/current identical bytes but altered SQLpayload_kind: BOTH old uncached and reuse helper exact existing header error; stale cached success impossible. Restore SQLkind→exact metadata success. Syntactically valid altered descriptor header/digest also rejects with same old category.
- SAME key current bytes replaced by malformed x00: exact uncached/reuse decode error, no stale return/error cache; restore original bytes→same exact metadata. Also successful syntactically valid same-key metadata change must update observed returned value rather than hide behind digest key.
- Different native root same key is typed InvalidRequest without IO into wrong-root data or overwriting bound cache. Valid same-root counterpart still works. Allowed-key pruning removes held entry/weight; reload redecodes. Root leaf/native identity stays exact.
- Production count/proxy constants exact; private lowered-limit fixtures demonstrate count-full and weight/one-oversized fallback Cow::Owned, exact result/error parity, no overflow/insertion, and no stale old entry retained after oversized replacement; restore small original→normal borrowed cache. cfgtest counts/weights establish only these mechanics, not actual heap or latency. No giant allocation just to cross512MiB.

Search public controls, reuse the existing complete fixture/installed pair:

- Whole ordinary-versus-profiled Full Exact +Local/Catalog Resolve equality, first setupSome/warmNone; compare warmed reuse responses to the pre-input/first unchanged result byte-for-byte/whole typed equality, not just counts. Profile equations and actual source regions still apply; no timing threshold for Tiny.
- Warm one engine's Full Local then interpose same-revision descriptor corruption; the SAME engine must return existing Catalogue/UnsupportedVersion cause, not cached success. Uncached new reader must agree. Restore exact descriptor blob→same engine/full response equals original; assert operation reads leave current fixture DB unchanged around each query. Warm+SQLkind mismatch same-reader also discriminates cache hit checks, valid restoration counterpart required.
- Keep existing stale_snapshot-before-corrupt-heavy-metadata; make sure warmed cache does not move the revision check later. Existing shared equal Entry/Assertion contributors/admitted scopes, unequal-body ambiguity/history, disabled/removed/registered-only/catalog release cases remain meaningful and execute at current source where affected. One exact precedence discriminator: valid early conflicting local entity pair PLUS corrupt later registered descriptor must return the existing corruption error, not ambiguity; restore later descriptor -> AmbiguousReference; restore conflicting body -> valid equal-body response. Preserve original scan order and unknown-PackId failure position during pruning (an earlier corrupt descriptor cannot be hidden by eager parsing a later invalid pack). Add only these cache-relevant pairings, not another full matrix or new public accessors.

Use owned native-root counterpart for cache root binding where feasible in existing fixture helpers; no claimed hosted Linux from macOS. Do not duplicate unchanged CLI/readable/54/64/vector tests solely for bookkeeping; their production bytes are unchanged and exact applicability will be recorded. Actual current-head hosted Linux later remains mandatory, separate external authorization.

## D. Ordered gates / environment / freeze

After explicit transfer, create an exclusive pass-specific parent `/tmp/vlajmaji-p01-a04-s05-cdc/i05-local01` with PRIVATE HOME/XDG_DATA_HOME/XDG_CONFIG_HOME and TMPDIR subdirs; do not reuse/delete existing roots. All Cargo children inherit these exact owned dirs so new fixtures remain inside counted D5. BEFORE overriding HOME, pin EXISTING tool/cache homes explicitly: CARGO_HOME=/Users/oubiwann/.cargo and RUSTUP_HOME=/Users/oubiwann/.rustup (current originals were unset, inherited HOME=/Users/oubiwann). Pin PATH=/Users/oubiwann/.cargo/bin:/Library/Frameworks/Python.framework/Versions/3.11/bin:/opt/homebrew/bin:/usr/bin:/bin:/usr/sbin:/sbin; existing cargo/rustup resolve through the first directory, no new download/home/cache. Record these actual pins and resolved tool paths in the receipt before children. RUST_TOOLCHAIN alone does not select wrapper cargo: use explicit guard `+1.98.0`, --locked --offline, retained target and actual compiler version; Make recipes additionally RUST_TOOLCHAIN=1.98.0 as their convention. Record exact env/argv/source hashes/terminalexit/rawlog/populations; no inherited user-default access, package download or second cache. Prior fresh_root AlreadyExists cause remains unproven, not newly labelled solved.

Repository AGENTS requires format→lint BEFORE testing. Narrow stdin rustfmt route already qualified: `rustup run 1.98.0 rustfmt --edition 2024 --emit stdout` with allowed FILEBYTES on STDIN (NOpatharg), stdout to owned scratch then copy back ONLY the exact allowed path. No recursive module traversal (collections.rs has unlisted children). Read-only make fmt-check afterward; lint pinned same ONEtarget. Never run broad makeformat beyond4path scope or unsupported skip_children.

Then one affected focused store/cache+search collection contract batch, current pinned makecheck, isolated search no-default and synthetic allfeatures because changed Full/library/store code requires current applicability. Use existing guard and exact source Synthetic recipe; no retainedFull/import/network. List-only changed package inventories store/search (lib path/export changes), no verify/publish. No CLI package replay absent actual package-path change. If a stable gate fails retain and repair only scoped actual failure, restore required order before next behavioral batch; no repeated broad passes absent source change/failure. Complete actual terminal gates, not banners/zero-selected tests. No oversized local campaign; original200MiB owned Tiny observation/stop policy remains calibrated (observed extant/retained roots, not invented historical peak).

At stable completion freeze one clean scoped source commit with both ADJACENT parsed trailers; exact4subset/cumulative paths, parent1b/tree/branch/lock/guard/oldS04refs checks. Retain all raw attempts/argv/env/source applicability; report current populations without overlaps, same-current compiler and cfg scope. CC final handback explicitly relinquishes FOURpaths/index/ONEtarget; CDC independently inspects whole critical delta, borrowed/noescape/None/timing/legacy policy and executes existing affected Tiny/API/public controls. No Full/scale acceptance from author gates or private cache counts.

## E. Later numerical/CI boundary — NOT CC authority

After frozen proof, CDC builds pinned optimized current CLI+observer outside target, qualifies SAME-code bounded Tiny/wire/semantic/method predicates as meaningful changed-source preparation, then presents NEW exact Full packet to root. Retain failed campaign01 immutable. Additional actual Full campaign must have NEW exclusive output (for example numeric-full02), changed runner fixed-output constant reviewed explicitly, exact source/binary/pin closure/new DBpair/resources/owned env, unchanged balanced12fresh+9warm+semantic22 and budgets. Baseline unchanged, no pooled observations/retry/cache reset/new Full store. Any early failure stops; no stage22 after known21 failure or warm13 after known12 failure. New campaign is ONE disclosed additional campaign under amendment, not a continuation erasing13failed children. No automatic second retry or seventh implementation pass; nonconvergence returns explicit disposition.

All six rows remain OPEN until actual numerical/current Linux/selected fresh independent Arc04 W1–W5 execution/CDC retro/rootcanonical gates. Source pass approval does not authorize push/CI, PR/merge or publication. Do not precreate acceptance/closure artifacts or launch fresh reviewer against WIP.


## Binding Catalog join amendment — SAME iteration05 / pass6

Amendment status: ISSUED — root final readiness PASS2026-10-28 UTC; delta intake and explicit transfer still required.

The preceding issued05 base is preserved BYTE-EXACT at artifacts/cc-prompt-iteration05-issued-before-catalog-amendment01.md (24777B/SHA84ac13c987abffcb6bb8e9c002215dd8901844dd21c38f50c5aae89078dbeaea). Its cache surface/limits/freshness/error/lifetime/gates remain binding. This amendment adds the source-grounded Catalog join correction within EXACTLY THE SAME FOUR EXISTING FILES and SAME human-authorized SIXTH pass, no budget reset/seventh. No source edit/build or handoff has occurred. Completed original120-line intake is historical actual coverage, not advance reading of this amendment.

### Why cache-only is incomplete, from EXISTING output only

CDC/root examined ALL27 retained PRIME request rows already emitted by warm1Full in the failed campaign. This is existing-output analysis, not another Full child/metric observation. Eighteen Exact primes query.total span1,847,833–3,854,208ns. Nine reference primes are below; these are priming rows, NOT three-observation acceptance or extra measured failure counts:

| Original prime identity | Whole query.total ns (or whole error) |
| --- | ---: |
| jbotci-local |1245563917|
| jbotci-catalogue-native333 |19844226833|
| la-catalogue-native333 |2701522125|
| llg-component |870683833|
| sut-native-entry |6320514875|
| unknown |5792|
| unresolved |1542|
| wrong-role |whole_error_ns821476334, profile null|
| missing |20528113375|

Only subsequent measured jbotci-local1,214,126,625ns triggered the actual stop. Other measured native cases were NOT reached. Canonical original raw is numeric-failed01/numeric-full01/warm-1-full/stdout (47rows total, all46 printed facts matched pre-input). Metadata descriptor parse is not a separately timed proven cost. Source nevertheless shows Catalog loops `release.assertions`, repeatedly `descriptor.entities.iter().find`, then `find_native` repeatedly scans registry.occurrences. That quadratic join is a second source-grounded correction target; cache-only cannot honestly be called sufficient without addressing it. Do not weaken the WholeResolve250/750ms or whole-error budget. The existing dictionary kernel already has primitive indices; no dictionary.rs/projection.rs change is justified here.

### Exact private borrowed-index shape in collection_query.rs

Keep all original cache public APIs/opaque fields/Cow return and `metadata_matches(&self)` unchanged. Add a PRIVATE helper of this shape; names may follow local style, behavior/lifetimes are binding:

```rust
struct FullDescriptorLookup<'d> {
    descriptor: &'d CollectionReferenceMetadata,
    entity_first: HashMap<(&'d str, &'d str), usize>,
    attribution_first: HashMap<(&'d str, &'d str, &'d str), usize>,
}
impl<'d> FullDescriptorLookup<'d> {
    fn new(descriptor: &'d CollectionReferenceMetadata) -> Self;
    fn entity(&self, role: &str, id: &str) -> Option<&'d CollectionEntityReference>;
    fn native(&self, occurrence: &OccurrenceRef, role: &str) -> Option<String>;
}
```

No public type/export/accessor for these indices, no new persistent cache index, no Clone of descriptor/registry/entity bodies. `descriptor` is the currently validated Cow's borrow. `new` scans entities ONCE, inserts `(entity.role.as_str(),entity.id.as_str()) -> ordinal` via `entry(...).or_insert(ordinal)`, and scans registry.occurrences ONCE, inserts `(row.occurrence.snapshot().namespace(), row.occurrence.snapshot().sha256(), row.occurrence.locator()) -> ordinal` likewise. Preserve FIRST ordinal, including when first native values areNone. Keys borrow existing strings, values primitive ordinals. No validation, sorting, deduplication of source rows or invented uniqueness error; arbitrary roles/duplicates accepted by the old typed decoder remain accepted. HashMap iteration must NEVER drive wire/output order.

`entity` uses first ordinal to borrow that existing entity. `native` performs ONE first-attribution tuple lookup, then clones only the requested entry/assertion optional String at the SAME old `find_native` call positions; unknown role remainsNone. Exact current `find_native` predicates are namespace+SHA+locator only; do NOT add source/release filters. First matching occurrence row with nativeNone wins even if a later duplicate hasSome. Optional one-row helper returning a borrowed attribution and two existing clone calls is equivalent, if it has the same first-row/no-output-clone-before-oldpoint semantics.

Index size is bounded to at most descriptor.entities.len() +descriptor.registry.occurrences.len() entries, duplicate keys only reduce it. These are TRANSIENT per-descriptor lookup indices, not full body caches or another selected inventory. They are not included in512MiB serialized cache proxy as a false heap bound; actual resident memory includes cache+indices+payload and must pass8GiB. No body strings/digests are computed in setup. All maps/drop and current Cow drop BEFORE the next mutable cache-helper call; no borrowed index survives the iteration/payload admission. No whole-cache immutable reference escaped by deepclone. Row/hash/index build/lookup/drop are lookup/residual, NOT returned-output projection; retain optional None/no-new-Instant contract. No extra profiling partition/public counter/wire change.

### Original Catalog ordering/error/admission policy is authoritative

Use these maps ONLY in the existing Full Catalog branch when a requested release belongs to the requested source. Keep the current registration scan order, full decoder/SQL-byte validation, first release lookup, requested-release cloning and existing collection/component ownership code exactly; do not replace its nested first/any semantics with a new uniqueness interpretation. Construct the borrowed index AFTER the existing owner match, invalid-target and occurrence/release/capture validation guards, immediately before the original release.assertions loop. Thus error precedence for wrong-role/unknown release/invalid occurrence remains unchanged.

Continue iterating the ORIGINAL release.assertions VECTOR, including every duplicate and its order. For each ID, replace only `.entities.iter().find(role==assertion &&id==...)` by indexed FIRSTentity lookup; an absent assertion still produces the existing Integrity error at that original loop position. Do NOT prevalidate all release IDs or reverse-match a native key before that loop: it could hide a missing earlier entity or change order. Preserve entity source_ids filter, entity occurrenceNone skip, optional requested occurrence filter and existing native call sites.

CRITICAL: current native admission skips only when requestedSome and actualSome!=expected. An actualNone is a WILDCARD at this metadata stage, not mismatch; payload can later beUnverified. Do NOT intersect exact reverse native indexes, drop unknown-attribution rows, treat nativeNone as failure or retarget source/release. Clone fields/build intermediate/public responses at the existing points, retaining Full Local nativeNone and all public/inner wire/schema/status/presence fields exactly.

Local MUST retain its `.filter` over ALL matching entities in original order and multiplicity for raw/admission rows; existing public representative/dedup policy remains unchanged. Do not substitute `entity_first` for that emitting scan or collapse raw Local duplicates. CRITICAL clarification of the preceding base ambiguity sketch: legacy post-scan rereads `.entities.find(role,id)` for EACH result row, so it compares only each REGISTRATION'S FIRST matching entity/digest against the global first registration's FIRST entity/digest. A later differing duplicate WITHIN the SAME registration does not newly cause ambiguity; it still supplies its original raw row. Compare the per-registration FIRST once while available, not each raw duplicate, accumulate the mismatch flag, complete all original registration reads and raise ambiguity only at the old post-scan point. Later corruption still wins. Public first-row/currentpack+digest representative and private all-contributor admission scopes remain unchanged.

### Small differential persistent controls, no expanded matrix

Within existing allowed search contract/private helper tests, use existing Tiny retained descriptors/SQL mutation and restore to discriminate:

1. FIRST duplicate role/id entity differs from later duplicate in relevant source/occurrence fields: Full Catalog must follow FIRST as old `.find`, while Local retains ALL matching ordered raw/admission rows and original public dedup. Non-FIRST differing duplicate within one registration must NOT newly cause ambiguity; a genuinely different FIRST entity across registrations MUST still cause post-scan ambiguity (and later corrupt registration must take precedence). Paired original valid descriptors restore exact prior response.
2. FIRST occurrence attribution has nativeNone and later duplicate has a conflictingSome (and a counterpart firstSome/laterdifferentSome): indexed native admission/output must use FIRST, preserve unknownactual wildcard, and never use last-wins or a reverse native-only intersection. Preserve exact current FullNative fields/status behavior; expected rows come from pre-input/uncached descriptor and old literal predicates, not observed indexed result.
3. Release assertions ordered `[B,A,B]` (source-known valid IDs) retains that order and multiplicity in Full Catalog matches. Missing entity at an earlier vector position must still give the same Integrity error even if a later assertion would match the requested native key. Unknown native actual has a valid counterexample showing admission survives for later payload classification.
4. Compare indexed helper outcome to the literal legacy `.find`/first-attribution predicates on these bounded inputs, plus WHOLE public Full Local/Catalog/ordinary-profiled parity already required. No new public test accessor, backend/dependency or fixture corpus. Native333 JB-versus-La/release scoping and existing unknown/wrong-role/missing controls remain source-/input-grounded; later numerical protocol unchanged.

These controls test FIRST/order/unknown/error semantics, not timing or a new performance matrix. No Full model/import/DB/cache trial now. Both cache/linear-join repairs must complete in the SAME existing pass before final stable gates/freeze; root final amendment readiness and exact delta intake before ownership transfer. The later one additional frozen22-child campaign can still fail; retain that failure with no seventh/retry if it does.


## Binding cross-mode join amendment — SAME iteration05 / sixth total pass

Amendment status: DRAFT — original FOUR-path work may continue; two extra paths NOT transferred until human scope amendment, root readiness and explicit delta handoff.

This addendum addresses two source-obvious Compact/Readable quadratic joins before the ONE further numerical campaign; it does not claim an observed Compact/Readable warm failure. Failed campaign01 stopped in Full warm stage13. No extra product/Full timing was run to diagnose this. Preserve the current issued35432B/b8e83216 original+Catalog body byte-exact separately before appending. Same sixth implementation pass and ONE additional <=22-stage campaign, unchanged budgets/no retry/seventh; extra-file scope authority must be recorded explicitly, not inferred from that pass approval.

## Exact scope delta / distinct three policies

Proposed maximum SIX EXISTING paths: original four remain; add ONLY (5) crates/search/src/dictionary/projection.rs and (6) crates/store/src/collections/reference_projection.rs. No borrowed.rs/dictionary.rs/collection_compact.rs/CLI/renderer/model/schema/deps/test-backend/newfile changes. Existing search/tests/collection_query_contract.rs and private inline tests in these allowed source files hold the new finite controls. Original four-path cache/Full work can continue independently while this draft awaits authority. Do not run final stable whole-source gates until all actually authorized source changes stabilize.

Keep these DIFFERENT semantics explicit:

- Retained Full descriptors accept typed duplicate rows: Catalog entity/attribution FIRST wins, unknown actual native is metadata wildcard; Local ALL raw rows, per-registration FIRST ambiguity. Existing preceding addendum remains exact.
- Constructed standalone Compact engine ALREADY validates primitive identity uniqueness in borrowed::index_document. Its existing ordinal maps are equivalent to its later .find under this invariant; do not weaken/strengthen constructor validation or copy Full duplicate policy here.
- Raw retained store projection accepts ordered duplicate occurrence rows and uses ANY matching native attribution, with lazy parse visit/error order. FIRST-only Full or constructed-engine UNIQUE semantics cannot be substituted here. This path does NOT admit missing actual native as a positive match.

## A. Compact kernel: three substitutions, no new indexing or API

At frozen1b dictionary/projection.rs::resolve_catalog iterates the requested release.assertions, then scans metadata.assertions, metadata.entries and registry.occurrences for every ID; occurrence scan allocates record_key and tuple_hex repeatedly. Its constructor always calls borrowed::index_document, which builds UNIQUE entries/assertions/occurrence_attributions maps and rejects duplicates before this method runs. The engine's document and those ordinals remain immutable together.

Replace only the repeated joins with existing indices:

```rust
// Shape: keep ORIGINAL surrounding control flow, continue/errors and vectors.
let Some(assertion) = self.indices.assertions.get(id.as_str())
    .and_then(|&ordinal| metadata.assertions.get(ordinal)) else { continue; };
// ... original optional occurrence predicate ...
let Some(entry) = self.indices.entries.get(assertion.entry.as_str())
    .and_then(|&ordinal| metadata.entries.get(ordinal)) else { continue; };
// ... original source predicate ...
let occurrence_key = super::tuple_hex(&assertion.occurrence); // ONCE/candidate
let attribution = self.indices.occurrence_attributions.get(&occurrence_key)
    .and_then(|&ordinal| self.document.registry.occurrences.get(ordinal));
```

EntityId supports as_str as in existing code. Do not add unverified .unwrap, reconstruct keys with a stricter identity constructor, or clone assertion/entry bodies. Existing missing assertion/entry behavior is CONTINUE, not the Full metadata Integrity error. Keep original release.assertions vector order AND duplicates, pairs comparison, actualNone→unverified (when expectedSome), confirmed/unverified vector order, availability-driven NoMatch/CatalogueOnly, owner/component/collection/invalid-target/occurrence validation, requested-release summary and all status/error/materialization behavior. Existing uniqueness means .find first versus index is not a new duplicate policy. Do NOT edit borrowed.rs or introduce a new index; no broad optimization of detail/Exact/kernel code. Existing reference_assertion and optional projection timing regions stay exact: key/ordinal/classification remains lookup, actual returned fields/hash clones remain projection, ordinaryNone no newInstant. Readable shares the same typed Compact response and needs no separate renderer change.

## B. Raw store projection: lazy ordered-key index with deferred errors

At reference_projection.rs around572–640, each source/optional-occurrence-eligible assertion entity scans registry.occurrences RawValues, parses OccurrenceKey for EACH row, and parses complete OccurrenceAttribution ONLY when the tuple matches. It stops at FIRST native-success row, but can continue past a matching row with missing/different native values to a later duplicate. A malformed key before success errors; an unrelated tuple's malformed native body is ignored. This repeated parse join is quadratic. Later materialization ALSO reads occurrence keys/selected full rows; preserve that later code UNCHANGED.

A private per-call helper of the following shape is concrete; local type names may vary, lifetime/behavior is binding:

```rust
struct RawNativeLookup<'a> {
    rows: &'a [&'a RawValue],
    // Exact namespace/SHA/locator STRING tuple, ordered matching raw ordinals.
    by_tuple: BTreeMap<(String, String, String), Vec<usize>>,
    first_bad_key: Option<usize>,
    // These target native flags are CONSTANT for this one reference_projection.
    wanted_entry: Option<&'a str>,
    wanted_assertion: Option<&'a str>,
    result: BTreeMap<(String, String, String), bool>,
}
// Equivalent distinct lifetimes for target/rows are fine if borrowed lifetimes
// actually require it; never clone raw descriptor bodies to satisfy borrowing.
// build(): parse keys once until FIRST key error, record ordinal, DO NOT return it.
// matches(&mut self, &OccurrenceRef) -> Result<bool, serde_json::Error>
```

Helper initialization is LAZY at the FIRST entity that passed the original source/occurrence filters AND actually needs native matching. No initialization/registry validation when all entities are filtered or neither native flag is supplied. Keep original entity iteration and all later release/capture/source/registry code/order intact. Tuple construction uses the same plain strings the old comparisons use, not fallible new SourceId/RecordIdentity grammar or normalization.

Key prepass parses `OccurrenceKey` from each raw in order ONCE until first error; it stores successfully parsed tuple→ordered ordinal lists for the valid PREFIX only. Stop at the first invalid key. Defer that invalid key's ordinal without returning its error early. No full OccurrenceAttribution decode in prepass. An ordinal is enough: when legacy scan must reach that row, reparse the EXACT immutable raw as OccurrenceKey and propagate the real serde_json::Error through the ORIGINAL CollectionError::ReferenceProjection{pack,digest,source} mapping. This avoids cloning/stringifying errors or a persistent negative cache. The immutable bytes make the same parse deterministic. Returning a stored owned original error exactly once is also acceptable if ownership/lifetime is explicit and errors are not memoized.

For an eligible candidate occurrence:

1. If a bool result for this exact tuple exists, return that successful prior computation; target native flags and raw buffer are fixed for the whole helper. No result exists for an error.
2. Visit ONLY matching tuple ordinals from the valid prefix, in ORIGINAL raw order. Decode full OccurrenceAttribution at each such ordinal, propagating its ORIGINAL typed error immediately when visited. Nonmatching tuple rows' malformed native bodies must NEVER be newly decoded/error. Preserve native_match exactly: requested missing flag is unrestricted, requestedSome requires actualSome equal; actualNone for requestedSome is false, NOT Full's wildcard. Continue on false, break on the first true (ANY matching duplicate). Memoize true and return without replaying a later key error or parsing later matching rows.
3. If no true row was found, replay first_bad_key error if it exists: legacy would reach it before finding success. Do NOT memoize false or an error before that check. If there is no key error and no typed matching error, memoize false and return. All valid-prefix nonmatching keys have already been inspected; do not scan RawValues again per candidate.

An earlier matching typed-body error outranks a later key error; a success before either shields later rows DURING THIS NATIVE PHASE. A later candidate with another tuple may still reach the deferred key error, so one candidate's true result cannot suppress errors globally. Result memo keys are bounded to candidate occurrence tuples, ordinals total<=registry.occurrences.len(), valid key map<=raw row count. Memoize ONLYbool success/failure after full original relevant scan; NEVERerrors. All state is per-call and dropped before function returns, no cross-target/request cache, no raw-body clone/complete typed registry cache. Work/index/memo allocations/drop remain metadata lookup, actual memory included in8GiB; no fake512MiBheap claim or output field/schema change.

Downstream materialization lines951ff still parses ALL occurrence keys and selected complete rows. Therefore native early success does NOT imply whole reference_projection succeeds if later phases find bad keys/selected fields. Preserve all subsequent error precedence. Do not remove, duplicate, eagerly prevalidate or reorder those phases, and do not claim false whole-method success in a helper-only test. No raw error is converted to absent/NoMatch or generic text.

## C. Finite paired controls, original inputs and no new matrix

Use existing allowed search contract fixtures, RawValue helper unit tests inside reference_projection.rs, and already-required same-code semantic/Tiny/native witnesses. No new test file/backend/dependency/public accessor. Preserve complete existing Full/Compact wire, target-scoped native333 JBversusLa, unknown/missing/unverified/role controls. Add only these discriminators:

- Constructed Compact valid fixture: ordinal lookup versus literal old `.find` yields WHOLE same Exact/Local/Catalog response, page/order/source and ordinary/profiled parity; retain original duplicate-identity constructor failures. Releaseassertions `[B,A,B]` retains declared order/multiplicity; missing assertion and missing entry retain CONTINUE behavior. An absent attribution +requestedSome retains Unverified, not NoMatch/Integrity. Existing complete fixture supplies bodies; pre-input expected rows are independent of candidate result.
- Raw private helper differential against a literal legacy nested-loop implementation on bounded inline RawValue rows: duplicate tuple firstNone then laterSomeWanted yields true (rejects FIRST-only); firstdifferentSome/laterWanted yields true; absent/mismatching actual yields false when no other match/error. Malformed key before would-be success errors; key error after earlier success is NOT raised by helper. A second eligible tuple lacking an earlier success DOES raise that deferred key error. Matching tuple malformed complete row before success errors; malformed complete row of a NONmatching tuple is ignored; malformed matching row after earlier success is ignored in this phase. Keep original error category and concrete serde source, not string-only assertions.
- Public error-precedence counterpart: use an earlier native success, a later deferred occurrence-key error, and a deliberately malformed raw.captures CaptureKey/CaptureProjection field that the existing code reads AFTER native matching and BEFORE the final registry occurrence loop (951ff). The capture-phase error must win; restoring that exact capture field reaches the later materialization occurrence-key error. Do NOT use early RawSourceMetadata/component/collection/code_notices parsing as the supposedly intervening error. Conversely a malformed key before native success must win before the intervening phase. Actual complete public projection/response equality on the restored valid descriptor is required. Do NOT demand whole-method success just because private native phase bypassed later error. Restore exact fixture descriptors after intentional SQL changes and use fresh read-only byte pairs around product calls.

No new Full observation to investigate this lead. Later SAME one additional frozen22-stage campaign must actually cover all3modes/27prime+27measured and native/error/source/semantic expectations; no promise of passing from source alone. All current local/Tiny gates wait for stabilized complete authorized six-path source, while four-path focused work can proceed normally. If any finite fixture is invalid under actual public preconditions, preserve the setup failure and repair the harness only; do not weaken the required error/order oracle or label it a product defect.

## Authority and transfer boundary

The previous human scope ceiling is FOUR. Until affirmative explicit human scope amendment AND root concrete readiness, the two extra files remain READONLY. After approval/issue, retainedCC reads WHOLEthis addendum, current plan§17.6/ledger and exact critical source/index-builder bodies; returns source-cited delta differentiating FullFIRST/CompactUNIQUE/RawANY+error visits. CDC accepts delta, verifies current original-four WIP stays in scope and both extra baseline files/lock/guard/oldrefs unchanged, then explicitly extends SOLE author ownership to the extra two files. Existing four-file transfer/one shared index+target continues, not a new writer/pass/worktree/cache. All six criteria remain OPEN; no seventh/retry/extra campaign/push from this addendum. Later executable trial still needs exact root frozen review.
