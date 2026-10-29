# S02 — Reviewed curation with explicit immutable context

| Field | Value |
| --- | --- |
| Status | SOURCE-GUIDED DRAFT; UNISSUED. Depends on actual S01 acceptance; not a second concurrent implementation |
| Authority | Arc05/directive04 only, same retained CDC Sol6.1/high and CC Luna/high; five total passes for this later slice |
| Baseline | Existing normalized model at merged a55/tree836; S01 owned APIs/schema1 are currently author WIP. Replace with actual accepted S01 HEAD/pins before readiness/issue |
| Useful result | Persist reviewed own-source links/senses, inspect missing context truthfully, and materialize exact supplied Tiny context into existing validated Dictionary/reference queries |
| Version / date | 0.1 / 2026-10-28; concrete policy draft for root review, not executable readiness |

## 1. Boundary and source findings

Keep the owned base Dictionary self-contained. Do not add foreign placeholder releases or import upstream assertion bodies into every journal edit. New context/curation types and strict byte codec live in lexicon, without store/search dependencies; authoritative persistence remains in store. S03 owns the sealed full-history snapshot/loader; S04's optional data-owned can therefore depend on lexicon alone. This slice's version1 curation document is not an alias for a PackSeal or a completed full-history snapshot.

Actual model dependencies: Dictionary's global entity IDs span metadata roles (`dictionary.rs::validate` idrows), not just separate entry/assertion namespaces. CatalogTarget requires loaded/catalogued release→component→collection/source and occurrence/capture closure; EditorialSenseRecord requires actual loaded assertion IDs. RelationRecord carries an editorial_revision but no separate editor/rationale fields. Existing ExtensionRecord can own an existing relation/sense ID and retain typed RawJson provenance. `compose_dictionaries(Vec<Dictionary>)` consumes actual validated bodies and rejects unequal same-ID records; it does not accept invented placeholder/full duplicates. Existing reference views derive current_eligible from selected active assertion membership, not simply the curation creation revision. These facts constrain the following projection.

## 2. Pure lexicon contract

Proposed new module `lexicon/src/owned_curation.rs` and lib re-exports; no normalized metadata/schema/PackId change. All wire types below have exact ordered fields, Debug/Clone/PartialEq/Eq/Serialize/Deserialize, deny_unknown_fields on structs; enums use snake_case externally tagged representations. IDs use existing EntityId/SourceId/ContributorId, strings retain exact bytes. No spelling normalization/inference.

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

Strict from_json_bytes checks UTF-8/duplicate object keys for the WHOLE envelope before serde, then existing Dictionary and curation validation; reuse lexicon's actual crate-private duplicate-key checker. Never use unverified deserialization as validated state. Hash is canonical typed Dictionary JSON, not file whitespace, SQLite bytes or a lexical occurrence key. The pin intentionally freezes the WHOLE immutable context input, including ordering/unrelated fields; an updated context needs a newly declared pin/curation record. No claim that the pin identifies every future mutable context independently of snapshot.

Validation, without context bodies: base follows owned journal one-source/revision/head rules. Pin IDs and record IDs are unique/reserved against all base metadata IDs; lower64/hash syntax and sorted unique declared inventories are checked. A pin cannot declare the base owned SourceId. Each target names a declared pin/source/release, entry/assertion IDs and nonempty capture namespace/locator with lower64 capture digest; no broader new Unicode namespace grammar. A pin's declared inventories are verified against its body only at explicit materialize, not cheap head/history. Source/release/entry/assertion/occurrence/native constraints together identify the claimed external occurrence; never recover a missing ID by spelling.

Every curation record names a retained same-source owned revision; editor exists AND equals that revision's editor (normalized Link otherwise cannot attribute its editor truthfully). Rationale is explicit TextValue. Link origin is an existing own Entry/Assertion/Form/EditorialSense, or a declared Sense record; resolve a Sense origin only when that entire active Sense materializes. Sense members are ordered, at least two distinct actual assertion identities, each explicit owned assertion or pinned external target; no inferred grouping, duplicate membership or partial semantic grouping. Global ID collision rejects rather than generating a replacement ID.

One curation revision exists for every owned revision in the same order. Genesis selection empty. Each next active list is a complete unique set; tombstones are previous active IDs omitted from it, in prior active order. New records must name the corresponding new base revision and be active there; older definitions/pins are immutable ordered prefixes. Reactivation requires explicitly selecting the retained record. A record's creation revision/editor/rationale remain exact when later selection carries it forward; carrying it does not claim a new review. The document includes current base + full curation prefix, but is NOT sufficient alone to reconstruct every old base metadata cutoff; the journal's S01 prefix rosters remain authoritative for historical base states and S03 export.

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
2. Clone only the own base model once; consume supplied context Dictionaries in PIN declaration order (independent of caller argv order) into existing compose_dictionaries. Actual bodies come FIRST; unequal same-ID conflicts reject. Do not insert placeholder foreign releases or deep-clone contexts to escape borrows. Build primitive borrowed ID/occurrence indexes once for resolution; map iteration never determines output order.
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

Curate input is strict OwnedCurationDocument bytes; absent --input reads stdin. Each --context FILE is strict OwnedContextInput bytes (id plus actual Dictionary), so no native ID=path splitting. Parse static IDs/whole input before journal IO. Materialize reads a pinned requested base+curation state, calls pure function, validates result, then writes canonical Dictionary bytes to create_new OUTPUT only. Semantic error creates no output; output IO failure may leave an owned partial file, with journal unchanged and honest error. Success prints a small report version/source/revision/output native path/hash/resolutions; exact DTO still must be frozen before issue. Curation-show is explicitly a full curation/model read, not cheap head/history. Reuse existing structured errors/Json write+flush lifecycle; no new wrapper transport.

Extend the shipped minimal example or add one small public curation example to emit own r2 candidate, one Tiny immutable upstream context file and explicit record/pin input. Do not infer targets from candidate search results: derive IDs/occurrence/native fields from the supplied typed input before queries. Demonstrate migrate→curate→show with NO context (Unavailable), then materialize with exact context→explicit dictionary-build OUTPUT DERIVED_ROOT→existing full/compact/readable Local/Catalog/reference behavior. S03 sealing/history export remains separate; this normalized view file is not mislabeled as an owned snapshot or reimport format.

## 6. Finite proof, scope and remaining draft work

Required meaningful controls: schema1 baseline still works; explicit migration preserves old states/hashes and repeated no-write result; held-reader/stale migration rollback; two pre-opened schema handles read persisted schema; same-head curated CAS and ordinary schema2 commit carry state; whole pin/record/revision immutable-prefix/active/tombstone/reactivation; actual context canonical SHA/order/inventory mismatch; duplicate/conflicting IDs; absent context remains visible; three-member Sense with missing middle never projects the other two; correct full context projects exact ordered IDs and provenance extension; nativeNone/known mismatch and wrong occurrence do not retarget; inactive record behavior; Link depends on entire Sense; all original source metadata/Presence/raw/extensions preserved; strict codec/version/duplicate keys; actual public Rust+CLI with file/stdin/native paths and explicit derived Tiny reference query. No new Full data/trial or benchmark.

Provisional source subset: new pure lexicon module/re-export/tests; existing owned store/schema/lib + new curation integration test/example; existing CLI main + new curation contract test; owned guide and store/CLI README. No dependencies/lock, upstream model/PackId/search kernel/provider or original data rewrites. Pin final exact path allowlist after accepted S01. Standard ordered scoped formatting/lint/focused/current Make/no-default/disjointSyntheticallfeatures/affected package FILE gates and CDC independent reproduction remain. Higher arc fresh/SAME canonical and project/rights gates unchanged.

NOT READY TO ISSUE: reconcile accepted S01 actual APIs/schema/HEAD; freeze exact materialize CLI report fields/serialization/error precedence and all required read scopes; write the remaining open ledger/prompt only after root reviews this concrete draft. The choices above are prospective policy, not proof that the WIP already implements them. No S02 CC dispatch/source transfer/build/runtime has occurred. S01's full original frontier remains mandatory and is not displaced by this planning work.
