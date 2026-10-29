# S02 — Reviewed curation with explicit immutable context

| Field | Value |
| --- | --- |
| Status | ISSUED INITIAL/pass1: complete intake ACCEPTED; fourteen-path transfer effective only after metadata commit + explicit CDC dispatch |
| Authority | Arc05/directive04 only, same retained CDC Sol6.1/high and CC Luna/high; five total passes for this later slice |
| Baseline | Accepted S01 1b899ebb492d89ceb1561abb0befb17ff3ccc5a1/tree745f789e3ddef62a2c2977c95a1d76f6f12382db, parenta55; owned APIs/schema1 locally verified; normalized model1 unchanged |
| Useful result | Persist reviewed own-source links/senses, inspect missing context truthfully, and materialize exact supplied Tiny context into existing validated Dictionary/reference queries |
| Version / date | 0.3 / 2026-10-28; status/readiness metadata only; substantive reviewed contract unchanged |

## 1. Boundary and source findings

Keep the owned base Dictionary self-contained. Do not add foreign placeholder releases or import upstream assertion bodies into every journal edit. New context/curation types and strict byte codec live in lexicon, without store/search dependencies; authoritative persistence remains in store. S03 owns the sealed full-history snapshot/loader; S04's optional data-owned can therefore depend on lexicon alone. This slice's version1 curation document is not an alias for a PackSeal or a completed full-history snapshot.

Actual model dependencies: Dictionary's global entity IDs span metadata roles (`dictionary.rs::validate` idrows), not just separate entry/assertion namespaces. CatalogTarget requires loaded/catalogued release→component→collection/source and occurrence/capture closure; EditorialSenseRecord requires actual loaded assertion IDs. RelationRecord carries an editorial_revision but no separate editor/rationale fields. Existing ExtensionRecord can own an existing relation/sense ID and retain typed RawJson provenance. `compose_dictionaries(Vec<Dictionary>)` consumes actual validated bodies and rejects unequal same-ID records; it does not accept invented placeholder/full duplicates. Existing reference views derive current_eligible from selected active assertion membership, not simply the curation creation revision. Existing composition is a declared view, not an exact concatenation of every input vector: anonymous code_notices and registry.lineage use exact-value dedup even within one input; extensions retain the maximum identical-row multiplicity of any input; keyed equal-ID rows collapse and unequal bodies reject; each role follows first-input ordering, and catalogue_revision is the maximum input revision. The owned journal and original pinned context files remain exact authorities. This adapter adopts those existing semantics without a second merge/restoration algorithm or pack.rs change. These facts constrain the following projection.

## 2. Pure lexicon contract

Proposed new module `lexicon/src/owned_curation.rs` and lib re-exports; no normalized metadata/schema/PackId change. All wire types below have exact ordered fields, Debug/Clone/PartialEq/Eq/Serialize/Deserialize, deny_unknown_fields on structs; enums use snake_case externally tagged representations. OwnedCurationDocument is the validated private-data exception to derived Deserialize: implement manual Deserialize as below; never derive an invariant-bypassing wrapper decoder. IDs use existing EntityId/SourceId/ContributorId, strings retain exact bytes. No spelling normalization/inference.

```rust
pub struct OwnedContextPin {
    pub id: EntityId,
    pub model_sha256: String, // lower64, SHA256(Dictionary::to_json_bytes())
    pub source_ids: Vec<SourceId>, // entire context inventory, sorted unique
    pub release_ids: Vec<EntityId>, // entire context inventory, sorted unique
}
pub struct OwnedPinnedTarget {
    pub context: EntityId,
    pub target: CatalogTarget, // occurrence MUST be Some
    pub entry: EntityId,
    pub assertion: EntityId,
}
pub enum OwnedSenseMember {
    OwnedAssertion(EntityId),
    External(OwnedPinnedTarget),
}
pub enum OwnedCurationKind {
    Link { origin: LocalRef, direction: Direction, target: OwnedPinnedTarget },
    Sense { members: Vec<OwnedSenseMember> },
}
pub struct OwnedCurationRecord {
    pub id: EntityId,
    pub revision: EntityId,
    pub editor: ContributorId,
    pub rationale: TextValue,
    pub kind: OwnedCurationKind,
}
pub struct OwnedCurationRevision {
    pub revision: EntityId,
    pub active: Vec<EntityId>,
    pub tombstones: Vec<EntityId>,
}
pub struct OwnedCurationData {
    pub version: u32, // 1
    pub dictionary: Dictionary, // current self-contained owned base
    pub contexts: Vec<OwnedContextPin>,
    pub records: Vec<OwnedCurationRecord>,
    pub revisions: Vec<OwnedCurationRevision>,
}
pub struct OwnedContextInput { pub id: EntityId, pub dictionary: Dictionary }
impl OwnedContextInput {
    pub fn from_json_bytes(bytes: &[u8]) -> Result<Self, OwnedCurationError>;
    pub fn to_json_bytes(&self) -> Result<Vec<u8>, OwnedCurationError>;
}
// Validated wrapper with private data; not a public mutable invariant-bearing field:
pub struct OwnedCurationDocument(/* private OwnedCurationData */);
impl OwnedContextPin {
    pub fn from_dictionary(id: EntityId, context: &Dictionary)
        -> Result<Self, OwnedCurationError>;
}
impl OwnedCurationDocument {
    pub fn from_data(data: OwnedCurationData) -> Result<Self, OwnedCurationError>;
    pub fn from_json_bytes(bytes: &[u8]) -> Result<Self, OwnedCurationError>;
    pub fn to_json_bytes(&self) -> Result<Vec<u8>, OwnedCurationError>;
    pub fn as_data(&self) -> &OwnedCurationData;
    pub fn into_data(self) -> OwnedCurationData;
    pub fn materialize(&self, inputs: Vec<OwnedContextInput>)
        -> Result<OwnedMaterialization, OwnedCurationError>;
}
```

OwnedCurationDocument implements manual `Deserialize` like existing Dictionary: deserialize the deny_unknown_fields OwnedCurationData DTO, then call `OwnedCurationDocument::from_data(data)` and map its validation error with `serde::de::Error::custom`. All ordinary deserialization paths therefore enforce the wrapper invariants; do not construct the private wrapper directly from decoded data. Serialize remains direct typed data. The strict public byte codec below additionally owns UTF-8/syntax/duplicate-key/original-nested-byte guarantees, which generic Deserialize alone does not promise.

Strict OwnedCurationDocument::from_json_bytes checks UTF-8/syntax/duplicate object keys for the WHOLE envelope, then decodes a version-only header (other fields ignored at this stage) and rejects unsupported version, then its full deny_unknown_fields wire with dictionary held as RawValue, then Dictionary::from_json_bytes on the ORIGINAL nested bytes, then curation validation. No Value reserialization. OwnedContextInput::from_json_bytes likewise checks UTF-8/syntax/duplicate keys for the WHOLE `{id,dictionary}` object, decodes deny_unknown_fields/id with dictionary as RawValue, then calls Dictionary::from_json_bytes on original nested bytes. This Input codec is inside lexicon, where the actual crate-private duplicate-key checker is accessible; CLI calls this public strict codec rather than trying to invoke that private checker. OwnedContextInput has no version field of its own; Dictionary carries its model version. Its to_json_bytes is direct typed serde. Never use unverified deserialization as validated state. Hash is canonical typed Dictionary JSON, not file whitespace, SQLite bytes or a lexical occurrence key. The pin intentionally freezes the WHOLE immutable context input, including ordering/unrelated fields; an updated context needs a newly declared pin/curation record. No claim that the pin identifies every future mutable context independently of snapshot.

Validation without context bodies calls a private PURE `validate_owned_base(&Dictionary) -> Result<(),OwnedCurationError>` in owned_curation.rs, with no store imports. Its input is already a validated Dictionary. Exact further checks: model/registry1, legacy_archiveNone, exactly one source with originOwnedDraft, exactly one matching head, nonempty owned_revisions all of that source in one insertion-ordered chain (first parentNone, each subsequent parent previous ID, last ID is head), catalogue_revision equals revisions.len()-1 and fits SQLite's 0..=i64::MAX journal sequence; every revision's complete active set/tombstones satisfy the normalized validator and its tombstones additionally equal the previous active-set removals in previous order. The first tombstones vector is empty. All Entry/Collection source ownership is already closed by Dictionary against the sole registry source; no foreign source can hide behind a Catalog placeholder. Registered editors are required by Dictionary. This CURRENT-data helper does not claim to prove historical record append cutoffs, when assertions were first appended, or old genesis notices from a final Dictionary alone. S01 store create/commit and explicit migration's actual per-revision prefix reconstruction retain those stronger historical/creation checks; S03 pure history envelope must carry the actual prefix rosters before it can verify them. Do not import store policy into lexicon or pretend current metadata alone proves the old histories. Pin IDs and record IDs are unique/reserved against all base metadata IDs; lower64/hash syntax and sorted unique declared inventories are checked. A pin cannot declare the base owned SourceId. Each target names a declared pin/source/release, entry/assertion IDs and nonempty capture namespace/locator with lower64 capture digest; no broader new Unicode namespace grammar. A pin's declared inventories are verified against its body only at explicit materialize, not cheap head/history. Source/release/entry/assertion/occurrence/native constraints together identify the claimed external occurrence; never recover a missing ID by spelling.

Every curation record names a retained same-source owned revision; editor exists AND equals that revision's editor (normalized Link otherwise cannot attribute its editor truthfully). Rationale is explicit TextValue. Link origin is an existing own Entry/Assertion/Form/EditorialSense, or a declared Sense record; resolve a Sense origin only when that entire active Sense materializes. Sense members are ordered, at least two distinct actual assertion identities, each explicit owned assertion or pinned external target; no inferred grouping, duplicate membership or partial semantic grouping. Global ID collision rejects rather than generating a replacement ID.

One curation revision exists for every owned revision in the same order. Genesis selection empty. Each next active list is a complete unique set; tombstones are previous active IDs omitted from it, in prior active order. New records must name the corresponding new base revision and be active there; older definitions/pins are immutable ordered prefixes. Build the base revision-ID→insertion-index mapping once. For EVERY historical curation selection at index i, each active and tombstone ID must reference a retained record whose creation revision index is <=i. A record retained in the final document must never be selected or tombstoned in earlier history before its creation; require it active at its own creation selection. This is part of existing invalid-history validation, including pure from_data/manual Deserialize and strict byte codec, not merely commit-time checking. Reactivation requires explicitly selecting the retained record. A record's creation revision/editor/rationale remain exact when later selection carries it forward; carrying it does not claim a new review. The document includes current base + full curation prefix, but is NOT sufficient alone to reconstruct every old base metadata cutoff; the journal's S01 prefix rosters remain authoritative for historical base states and S03 export.

## 3. Materialization, statuses and exact projection

```rust
pub enum OwnedCurationStatus { Inactive, Unavailable, Unresolved, Resolved }
pub enum OwnedCurationReason {
    MissingContext, MissingOrigin, MissingEntity, WrongSource, MissingRelease,
    AssertionNotInRelease, OccurrenceMismatch, PayloadUnavailable,
    NativeIdentityUnknown, NativeIdentityMismatch, MemberUnavailable,
    DependencyUnavailable,
}
pub struct OwnedCurationResolution {
    pub record: EntityId,
    pub status: OwnedCurationStatus,
    pub reason: Option<OwnedCurationReason>,
    pub member: Option<u64>, // first failing Sense member index; None for Link
    pub projected: bool,
}
pub struct OwnedMaterialization {
    pub version: u32, // 1
    pub source: SourceId,
    pub revision: EntityId,
    pub dictionary: Dictionary,
    pub resolutions: Vec<OwnedCurationResolution>, // original record order, incl inactive
}
```

Error policy is distinct from absence: invalid envelope/version/IDs/history, unknown/duplicate supplied context ID, wrong supplied canonical SHA/inventory and conflicting same-ID bodies are typed errors with NO materialized result; no silent fallback. Proposed non_exhaustive OwnedCurationError has InvalidDocument(String), UnsupportedVersion{found:u32}, DuplicateId(EntityId), MissingReference(String), ContextMismatch{context:EntityId,expected:String,actual:String}, Composition(PackError), Dictionary(DictionaryError); concrete Display/Error sources and Errors docs, no new dependency. Unknown reasons/versions are not accepted as arbitrary strings.

Algorithm:

1. Validate this document. Match supplied input IDs to declared pins; hash/check EVERY supplied Dictionary and exact source/release inventory, reject duplicates/unknown/mismatch before projection. Missing models are allowed and visible. Do not read any file or network in this pure function.
2. Clone only the own base model once; consume supplied context Dictionaries in PIN declaration order (independent of caller argv order) into existing compose_dictionaries. Actual bodies come FIRST; unequal same-ID conflicts reject. Adopt the existing declared composition semantics above, including notice/lineage dedup, extension maximum multiplicity, first-input role ordering and catalogue maximum; do not claim whole per-input-vector equality of the resulting view or restore anonymous rows with a competing merge. Do not insert placeholder foreign releases or deep-clone contexts to escape borrows. Build primitive borrowed ID/occurrence indexes once for resolution; map iteration never determines output order.
3. Inactive records return Inactive/reasonNone/projectedfalse without target resolution. For active records, source/release/component/collection ownership, exact assertion ID/entry ID/release assertion membership and exact occurrence must all match supplied pinned context. Raw capture and occurrence must actually be loaded. Requested Some native fields must match actual registry occurrence attribution; actualNone is Unresolved/NativeIdentityUnknown, wrongSome is Unresolved/NativeIdentityMismatch; target None makes no native claim. Do not import Full's wildcard matcher or retarget via lexical text.
4. Missing model gives Unavailable/MissingContext; supplied model with missing/mismatched target gives Unresolved and the precise reason. Owned members resolve from the base; explicit tombstoned historical assertions can still be identity-resolved, with ordinary query current_eligible computed later. For Sense use the FIRST failing member in declared order; if any member is absent/unresolved, project NONE of that Sense. The record and member index remain visible. No two-member subset of a three-member sense is ever silently projected.
5. Resolve all active Sense records first for dependency availability; then Links, preserving original order WITHIN each normalized role. A Link to a nonmaterialized Sense is Unresolved/DependencyUnavailable. No recursion or inferred dependency cycle resolution.
6. For a resolved Link, append RelationRecord with same id, exact origin/direction, kind EditorialLink, scope mapped from LocalRef role, target Catalog(exact supplied target), evidenceNone, editorial_revisionSome(record.revision). For a resolved Sense append EditorialSenseRecord with same id, exact ordered resolved assertion IDs/editor/revision/rationale. Current active selection does not rewrite creation provenance.
7. Preserve complete adapter metadata in an ExtensionRecord owned by each projected ID: name exactly `vlajmaji.owned-curation.v1`, evidenceNone, RawJson containing typed ordered `{version:1,record:OwnedCurationRecord,contexts:Vec<OwnedContextPin>}` for the pins referenced by that record in declaration order. This retains Link's per-record rationale/editor and exact context SHA, which RelationRecord alone cannot hold. It is a declared adapter, not certified interchange; no normalized schema change. Reject normalized/global-ID or reserved extension conflicts; do not overwrite existing rows. Validate final Dictionary via from_document before returning. Original base and contexts remain separate historical authorities; returned Dictionary is an explicit view, not an editable journal replacement.

A missing unrelated context may coexist with resolved independent records, but every unprojected record stays in resolutions and retained original envelope. A group is all-or-nothing. Output status Resolved means exact identity/context was established, not that an assertion is currently selected, lexically eligible, licensed, or linguistically equivalent. Existing query APIs keep those distinct flags and all source bodies. No new provider/cache/performance claim.

## 4. Explicit journal schema migration and write compatibility

Keep S01 create/init default schema1, and schema1 head/history/show/edit behavior. No automatic migration on open/read/edit. New library supports schemas1 and2; pure normalized/registry/curation-document versions remain1. Schema2 adds curation support; old binaries truthfully reject version2 rather than reinterpret it. Writer operations determine persisted schema INSIDE their transaction, not cached at handle open.

Proposed store additions (retain all S01 public signatures):

```rust
pub enum OwnedMigrationOutcome { Migrated, AlreadyCurrent }
pub struct OwnedCurationMigrationReport {
    pub version: u32, pub source: SourceId, pub head: EntityId,
    pub from_schema: u32, pub to_schema: u32, pub outcome: OwnedMigrationOutcome,
}
impl OwnedJournal {
    pub fn migrate_curation(&mut self, expected_head: &EntityId)
        -> Result<OwnedCurationMigrationReport, OwnedJournalError>;
    pub fn commit_curated(&mut self, expected_head: &EntityId,
        candidate: &OwnedCurationDocument) -> Result<OwnedWriteReport, OwnedJournalError>;
}
impl OwnedJournalReader {
    pub fn curation_show(&self, revision: &EntityId)
        -> Result<OwnedCurationDocument, OwnedJournalError>;
    pub fn curation_show_head(&self)
        -> Result<OwnedCurationDocument, OwnedJournalError>;
}
```

Add named MigrationRequired and InvalidCuration(OwnedCurationError) error/category variants without changing old categories. New curated operations on schema1 return MigrationRequired; baseline S01 operations remain usable. Schema2 ordinary S01 commit preserves all existing curation definitions/pins and appends an unchanged active-selection record for the new base revision, without claiming a new editorial review. It validates the inherited curation document against the new base, so it cannot accidentally take a reserved curation ID. New commit_curated includes base edit and curation append/selection in the SAME persisted-head CAS/transaction. No curation-only operation without an explicit new base revision.

New STRICT tables: `owned_context_pins(ordinal INTEGER PRIMARY KEY,id TEXT UNIQUE NOT NULL,payload BLOB NOT NULL)`; `owned_curation_records` same shape; `owned_curation_revisions(revision_id TEXT PRIMARY KEY NOT NULL REFERENCES owned_commits(revision_id),payload BLOB NOT NULL)`; `owned_curation_prefixes(revision_id TEXT PRIMARY KEY NOT NULL REFERENCES owned_commits(revision_id),contexts_length INTEGER NOT NULL,records_length INTEGER NOT NULL,curation_sha256 TEXT NOT NULL)` with nonnegative lengths/ordinals. All old schema1 tables/24 base sections/commit state hashes remain exact logical records. Prefixes and sidecar revision rows are immutable.

Curation SHA uses canonical typed bytes of ordered body `{version:1,source,revision,base_state_sha256,contexts,records,revisions}`: own SourceId/selected base revision+unchanged S01 state SHA, then typed prefixes. Reconstruct/full curation-show verifies this hash; cheap head/history do not hash context payloads or materialize models. No hidden verification on open.

Explicit migrate: BEGIN IMMEDIATE, persisted expected-head check, explicitly verify old historical base states (migration is a named whole-history operation), create four tables, write empty curation prefix/selection/hash for EVERY old base revision, set user_version and owned_meta.schema_version to2, COMMIT atomically. Empty curation history does NOT invent reviews or convert/drop existing normalized editorial rows. Old base hashes/head/catalogue_revision and all24 record bodies/prefixes stay exact. Repeated migration on valid schema2 is AlreadyCurrent/minimum-header+head check/no write; does not claim a fresh full history verification. Unsupported versions and stale head reject. Held readers cause Busy with rollback; no partial new tables/version bump. Use the S01 explicit rollback/uncertainty/unwind guard; no deferred foreign-key shortcut.

## 5. CLI and usable Tiny demonstration

Proposed additions under existing owned commands (all explicit ROOT, native file paths, no JSON argv/default/network):

```text
owned migrate-curation ROOT --expected-head REVISION
owned curate ROOT --expected-head REVISION [--input FILE]
owned curation-show ROOT [--revision REVISION]
owned materialize ROOT [--revision REVISION] [--context FILE ...] --output FILE
```

Curate input is strict OwnedCurationDocument bytes; absent --input reads stdin. Each --context FILE is strict OwnedContextInput bytes (id plus actual Dictionary), so no native ID=path splitting. Parse static IDs/whole input before journal IO. Materialize reads a pinned requested base+curation state, calls pure function, validates result, then writes canonical Dictionary bytes to create_new OUTPUT only. Semantic error creates no output; output IO failure may leave an owned partial file, with journal unchanged and honest error. Success prints the exact declared fieldset for private CLI DTO `OwnedMaterializeReport { version:u32, source:SourceId, revision:EntityId, output:NativePathReport, output_sha256:String, resolutions:Vec<OwnedCurationResolution> }`, version1. The existing QueryOutput::Json(Value) emitter determines stdout sorted map-key order plus one LF; Rust DTO declaration order is not a stdout byte-order promise, and no transport change is allowed. `output` uses existing native_path_report with encoding/hex/utf8, unix-bytes/lowercase native hex/Option UTF8 on supported Unix; no lossy UTF8 path substitution. OUTPUT file is EXACT Dictionary::to_json_bytes with no added LF, so output_sha256 is lowercase SHA256 of those file bytes; stdout report itself gets the existing single LF. This file is an explicit normalized view, not a sealed snapshot. Journal/base state is never mutated by materialize; stdout failure can follow a successfully written view and is not a rollback claim. Curation-show is explicitly a full curation/model read, not cheap head/history. Reuse existing structured errors/Json write+flush lifecycle; no new wrapper transport.

Extend the shipped minimal example or add one small public curation example to emit own r2 candidate, one Tiny immutable upstream context file and explicit record/pin input. Do not infer targets from candidate search results: derive IDs/occurrence/native fields from the supplied typed input before queries. Demonstrate migrate→curate→curation-show of the exact retained definition/document, with NO resolution computation in show. Then invoke materialize with ZERO context inputs to return visible Unavailable/MissingContext resolutions, followed by materialize with the exact context to return resolved records and an explicit normalized view→dictionary-build OUTPUT DERIVED_ROOT→existing full/compact/readable Local/Catalog/reference behavior. S03 sealing/history export remains separate; this normalized view file is not mislabeled as an owned snapshot or reimport format.

### Error/absence precedence and limits

- CLI Clap shape errors exit2. Parse expected/requested revision ID first; for curate decode the strict whole candidate bytes next, before ROOT. For materialize, read/decode every context FILE in argv order via OwnedContextInput::from_json_bytes before ROOT. File/strict JSON/ID errors use existing owned structured-error route, no configuration/default lookup.
- Then ROOT family/version/minimum header is checked. Curated commands require schema2 (schema1→MigrationRequired before revision lookup); curation-show checks the requested revision and explicitly reconstructs/verifies base+curation state. Ordinary head/history stay summary operations and never resolve/hash supplied context because they accept none.
- Writes check persisted expected head INSIDE transaction before append-policy comparisons, after static typed-input validation. Curated commit checks base append policy before sidecar append/selection policy. Migration compares head before schema2 AlreadyCurrent or schema1 full-history work. No current context bodies are required to commit a pinned curation definition.
- Pure materialize first checks ALL supplied context IDs for known/unique roster in input order, then validates supplied SHA/inventories in pin declaration order, then composes actual models, then resolves records/members, then validates the projected normalized model. No partial Dictionary on a typed error. Invalid shape/version/duplicate input, context SHA mismatch and same-ID conflict do not get relabelled Unavailable. Absence/unresolved member is instead a successful typed resolution state with no partial Sense projection.
- Create_new OUTPUT happens ONLY after semantic materialization succeeds. Existing target/write failure yields ordinary owned IO error, journal unchanged; partial owned output is retained honestly on mid-write failure. CLI pure-codec/materialization failures use `owned_invalid_curation`, journal/IO failures retain their matching owned_<category>, envelope/sources[] and shared emitter policy as S01. Detailed pure Rust error variants retain context/expected/actual and inner source; no new output handler or swallowed category conversion.

## 6. Finite proof, scope and remaining draft work

Required meaningful controls: schema1 baseline still works; explicit migration preserves old states/hashes and repeated no-write result; held-reader/stale migration rollback; two pre-opened schema handles read persisted schema; same-head curated CAS and ordinary schema2 commit carry state; whole pin/record/revision immutable-prefix/active/tombstone/reactivation and historical selection before creation rejected through from_data/manual Deserialize/byte codec; actual context canonical SHA/order/inventory mismatch; duplicate/conflicting IDs; absent context remains visible; three-member Sense with missing middle never projects the other two; correct full context projects exact ordered IDs and provenance extension; nativeNone/known mismatch and wrong occurrence do not retarget; inactive record behavior; Link depends on entire Sense; original authority documents/pinned context files remain unchanged with full metadata/Presence/raw fidelity, while the derived view follows declared existing composition: demonstrate duplicated notice/lineage retained in authority but deduplicated in view and extension maximum-input multiplicity separately; strict codec/version/duplicate keys; actual public Rust+CLI with file/stdin/native paths and explicit derived Tiny reference query. No new Full data/trial or benchmark.

Provisional source subset: new pure lexicon module/re-export/tests; existing owned store/schema/lib + new curation integration test/example; existing CLI main + new curation contract test; owned guide and store/CLI README. No dependencies/lock, upstream model/PackId/search kernel/provider or original data rewrites. The exact fourteen-path prompt allowlist is reconciled against accepted S01. Before archive Cargo exec assert physical cwd equals the exact disjoint archive and pass that workdir; no frozen guard/runner change. Standard ordered scoped formatting/lint/focused/current Make/no-default/disjointSyntheticallfeatures/affected package FILE gates and CDC independent reproduction remain. Higher arc fresh/SAME canonical and project/rights gates unchanged.

ROOT EXACT DRAFT READINESS PENDING ONLY: accepted S01 1b899ebb/tree745f, APIs/schema1, strict Input/base-helper/output/error contracts and exact fourteen-path readset/allowlist are reconciled above. The eight-row ledger and prompt remain UNISSUED; these are prospective S02 policies, not implementation proof. No S02 CC intake/source transfer/build/runtime has occurred. After root exact readiness, status-only issue, complete intake/source-cited readback and fresh pins precede explicit transfer. S01's actual disposition and limitations remain preserved.

## Version0.2 dependency reconciliation —2026-10-28

S01 is locally accepted at1b899ebb/tree745f with exact original controls and explicit limitations in its verifier. Schema1 remains current; its version negative changes pragma user_version2 while owned_meta remains1, so genuine matching schema2 after explicit migration does not require rewriting that S01 inconsistent-header test. Keep its UnsupportedVersion behavior/category. Exact fourteen source paths/readset are reconciled. Report transport follows existing Json(Value) sorted keys/singleLF; output FILE remains typed canonical Dictionary bytes/noLF/exactSHA. Existing compose-view dedup policy remains fixed. S02 still UNISSUED; no source/index/target transfer or acceptance claims.

## Version0.3 — exact readiness and intake-only issue,2026-10-28

Root read whole current bodies and complete F01–F03 delta and granted final readiness at planning28821fea/tree8aa8d531: plan30171B/918a47a1…, prompt13993B/0ff4c0bb…, ledger4382B/aa2d5736…. Eight original rows remainOPEN. The prompt issue changes ONLY its STATUS line, inverse-byte-exact to that reviewed draft. Accepted source1b899ebb/tree745f, lock65cd/guard426d are unchanged. Retained Luna/high CC receives complete required intake ONLY; source/index/ONEtarget stay QUIET until CDC accepts actual coverage/source-cited readback and fresh pins then explicitly transfers the14paths. No implementation, runtime or external authority merely from issue.

### Intake/handoff checkpoint —2026-10-29

Complete actual intake plus F01 clarification accepted by CDC after source-citation/status-scope calibration and fresh clean1b899ebb/tree745f/lock/guard/protected-ref checks. Actual record: artifacts/execution.md. AFTER this scoped metadata commit and separate explicit dispatch, CC owns exact14paths/index/ONEtarget; root/CDC readonly. No enumeration-only comprehension or product acceptance claim. Eight rows OPEN; technical issued contract unchanged.
