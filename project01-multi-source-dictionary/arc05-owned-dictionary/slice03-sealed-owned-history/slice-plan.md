# S03 — Sealed owned history and explicit reimport

| Field | Value |
| --- | --- |
| Status | POLICY/API DRAFT ONLY; UNISSUED. S02 INITIAL/pass1 implementation is active; no frozen accepted S02 dependency yet |
| Authority | Arc05/directive04; retained CDC Sol6.1/high + Luna/high CC; five total S03 assignments including initial, no automatic sixth |
| Known source | Accepted S01 1b899ebb/tree745f; source branch codex/project01-owned-dictionary. Final S02 HEAD/schema/API/readset must replace this dependency placeholder before issue |
| Useful result | Export/check/load every owned revision and curation cutoff, import into a fresh authoritative journal, replay only exact history, and query an explicitly extracted/materialized derived model |
| Version / date | 0.4 / 2026-10-29; prospective report/base-golden policy fixed with actual CLI dependency boundary; accepted S02/API/context-oracle reconciliation and final readiness pending |

## 1. Representation and ownership boundary

Use a DISTINCT version1 owned snapshot and seal in lexicon, not PackSeal/PackId or a fifth All input. Existing normalized Dictionary/registry1 and existing four upstream packages remain unchanged. No store/search dependency in the codec; S04's optional owned package must be able to load it before SQLite.

The payload stores the final complete typed base Dictionary ONCE, and the ACTUAL ordered per-commit 24-section prefix rosters plus the existing state hashes. It does not infer old cutoffs from the final model or store a complete duplicate corpus for every revision. For schema2 it also stores each final context pin/reviewed record ONCE, every curation selection, and ACTUAL per-revision sidecar prefix lengths/hash. These rosters are authority-bearing input, not counts presented as body-integrity proof. This is canonical typed logical history, not SQLite file bytes/private layout or incidental JSON whitespace inside a hand-mutated SQL payload; source capture bytes and typed RawJson content still retain their exact declared values. Every historical model is reconstructed/validated/hashed during explicit codec validation and export/check/import. Existing journal open/head/history retain their cheap scopes.

Original context Dictionary bodies are NOT embedded merely because curation pins reference them; pins/targets/provenance remain exact, and caller-supplied context is still required for S02 materialization. The snapshot preserves the owned base captures, registry, full metadata and curation definitions/history. It does not claim to redistribute foreign context corpora or certify licence, authorship, authenticity or linguistic equivalence. The seal is an integrity declaration, not a signature.

## 2. Proposed pure lexicon shape

New owned_snapshot module and public exports. All DTO fields below are ordered for canonical typed serialization, Debug/Clone/Eq/PartialEq/Serialize/Deserialize with deny_unknown_fields; validated wrappers use MANUAL Deserialize→from_data, never a private-invariant bypass. Existing new enums use snake_case. Strict byte codecs retain whole-envelope duplicate-key/UTF8/syntax checks and original RawValue bytes for nested Dictionary; no Value round trip.

```rust
pub struct OwnedSectionPrefix { pub section: String, pub length: u64 }
pub struct OwnedSnapshotCommit {
    pub revision: EntityId,
    pub state_sha256: String,
    pub prefixes: Vec<OwnedSectionPrefix>,
}
pub struct OwnedSnapshotCurationPrefix {
    pub revision: EntityId,
    pub contexts_length: u64,
    pub records_length: u64,
    pub curation_sha256: String,
}
pub struct OwnedSnapshotCuration {
    pub version: u32, // 1
    pub contexts: Vec<OwnedContextPin>,
    pub records: Vec<OwnedCurationRecord>,
    pub revisions: Vec<OwnedCurationRevision>,
    pub prefixes: Vec<OwnedSnapshotCurationPrefix>,
}
pub struct OwnedSnapshotData {
    pub version: u32, // 1
    pub journal_schema: u32, // exactly1 or2
    pub source: SourceId,
    pub head: EntityId,
    pub dictionary: Dictionary,
    pub commits: Vec<OwnedSnapshotCommit>,
    pub curation: Option<OwnedSnapshotCuration>,
}
pub struct OwnedSnapshot(/* private OwnedSnapshotData */);
impl OwnedSnapshot {
    pub fn from_data(data: OwnedSnapshotData) -> Result<Self, OwnedSnapshotError>;
    pub fn from_json_bytes(bytes: &[u8]) -> Result<Self, OwnedSnapshotError>;
    pub fn to_json_bytes(&self) -> Result<Vec<u8>, OwnedSnapshotError>;
    pub fn as_data(&self) -> &OwnedSnapshotData;
    pub fn into_data(self) -> OwnedSnapshotData;
    pub fn dictionary_at(&self, revision: &EntityId) -> Result<Dictionary, OwnedSnapshotError>;
    pub fn curation_at(&self, revision: &EntityId)
        -> Result<Option<OwnedCurationDocument>, OwnedSnapshotError>;
}
```

The section whitelist/order is EXACT S01 SECTIONS: registry.sources, contributors, distributors, aliases, lineage, snapshots, occurrences; captures; metadata.code_notices, collections, components, releases, entries, assertions, contexts, frames, forms, etymologies, relations, owned_revisions, editorial_senses, computations, annotations, extensions (with their full registry./metadata. prefixes). Every commit has all24 once in that order; unknown/duplicate/missing/reordered roster entries reject. Lengths fit usize and SQLite i64 range, do not exceed final vector lengths, never decrease, and final lengths equal final vectors exactly. metadata.owned_revisions length at commit i is i+1. Commit order and revision IDs equal the complete base revision chain; implicit sequence is index i. Source/head equal final base source/head. Schema1 requires curationNone; schema2 requires Some, including genuinely empty migrated histories.

For each commit, build DictionaryDocument using ONLY those prefixes, model/registry1, legacyNone, catalogue_revision=i and owned_head={source,revision}. Validate via Dictionary and the owned-base policy, then compare lowercase SHA256 of exact Dictionary::to_json_bytes to the stored state_sha256. Validate genesis and every adjacent edit's S01 policy: one same-source revision, immutable old bodies/order (structurally provided by shared prefixes), unchanged old release flags, newly appended assertions active, exact ordered tombstones, explicit reactivation. Do not infer old provenance from today's head alone. Reject wrong hashes even if lower64 syntax is valid. Do not retain Vec<Dictionary> for all revisions during validation: consume one reconstructed state at a time, retaining only the bounded prior state needed for adjacent policy comparison.

For schema2, curation revisions/prefixes match every base commit in order. Lengths are nonnegative/in-range/monotonic, bounded by final vectors and final lengths exact. Reconstruct each historical OwnedCurationData using THAT base model and its exact sidecar prefixes, including curation revisions through i; validate via OwnedCurationDocument::from_data. Enforce S02 record-creation ordering and complete selections/tombstones, immutable definitions/pins and exact historical curation SHA. A future-created record cannot be selected in earlier history. Empty migration histories are retained, not replaced with an invented editorial review.

Expose the ALREADY DECLARED S02 curation digest protocol as a pure method `OwnedCurationDocument::state_sha256() -> Result<String,OwnedCurationError>` if no equivalent exists in accepted S02. It hashes exactly S02's ordered {version,source,revision,base_state_sha256,contexts,records,revisions} body; no new algorithm/version. Store export and pure snapshot codec must share that method instead of silently diverging hash maps. A crate-private owned-base validator may be shared by the two lexicon modules. Reconcile actual S02 implementation before issue; no assumption it already has this method.

## 3. Seal and canonical byte policy

```rust
pub struct OwnedSnapshotSeal {
    pub version: u32, //1
    pub kind: String, //EXACT "vlajmaji-owned-snapshot"
    pub snapshot_version: u32, //1
    pub journal_schema: u32, //1|2
    pub dictionary_version: u32, //1
    pub registry_version: u32, //1
    pub source: SourceId,
    pub head: EntityId,
    pub revision_ids: Vec<EntityId>, //all, actual chain order
    pub snapshot_sha256: String,
    pub snapshot_bytes: u64,
}
pub struct SealedOwnedSnapshot { pub snapshot: Vec<u8>, pub seal: Vec<u8> }
pub fn seal_owned_snapshot(snapshot: &OwnedSnapshot)
    -> Result<SealedOwnedSnapshot,OwnedSnapshotError>;
pub fn load_owned_snapshot(snapshot: &[u8], seal: &[u8])
    -> Result<OwnedSnapshot,OwnedSnapshotError>;
```

Provide strict seal from_json_bytes/to_json_bytes. Error precedence: strict seal shape/duplicate/version/kind/hash syntax, payload length/hash, strict payload UTF8/duplicates/version/shape+nested Dictionary, full historical validation, seal inventory comparison, then canonical payload equality. Canonical payload/seal exports are typed serde_json compact bytes with NO LF and no NEW exporter-generated timestamps, machine paths or random metadata. Original Dictionary/revision/capture timestamps, caller-supplied locators/native paths and all other typed source fields remain exact data; this rule never strips them. Repeated export of the same journal snapshot is byte-identical. For this NEW format, STRICT canonical payload-byte equality is a SEALED-LOAD rule: load_owned_snapshot validates the payload and then requires the received bytes to equal snapshot.to_json_bytes; a separately hashed pretty/noncanonical spelling rejects NonCanonical at that final comparison. Bare OwnedSnapshot::from_json_bytes still performs the full strict wire/history validation but accepts otherwise valid JSON whitespace/field ordering; it does not have a seal or assert canonical input spelling. from_data and manual Deserialize validate the same history invariants without a byte-spelling condition. to_json_bytes always emits the one typed compact representation. This does not change existing PackSeal loader behavior. Seal JSON itself may be strict noncanonical whitespace input; the writer always emits canonical seal bytes. Hash/length cover the EXACT payload file bytes, not stdout JSON.

OwnedSnapshotError is concrete/non_exhaustive with InvalidDocument(String), UnsupportedVersion{kind:&'static str,found:u32}, MissingRevision(EntityId), History(String), LengthMismatch{expected:u64,actual:u64}, HashMismatch{expected:String,actual:String}, InventoryMismatch{field:&'static str}, NonCanonical, Dictionary(DictionaryError), Curation(OwnedCurationError). Display/Error retains concrete source chains and Errors docs. Do not accept an outer digest as a substitute for typed history validation.

## 4. Journal export and import policy

Proposed additions, exact final signatures require accepted S02 reconciliation:

```rust
impl OwnedJournalReader {
    pub fn snapshot(&self) -> Result<OwnedSnapshot,OwnedJournalError>;
}
// snake_case serde variants; report is deny_unknown_fields.
pub enum OwnedImportOutcome { Imported, AlreadyPresent }
pub struct OwnedImportReport {
    pub version: u32,
    pub outcome: OwnedImportOutcome,
    pub source: SourceId,
    pub head: EntityId,
    pub journal_schema: u32,
    pub revision_count: u64,
    pub snapshot_sha256: String,
}
impl OwnedJournal {
    pub fn import_snapshot(root: &Path, snapshot: &OwnedSnapshot)
        -> Result<(Self,OwnedImportReport),OwnedJournalError>;
}
```

snapshot() is EXPLICIT whole-history export: use the reader's existing pinned connection, read ACTUAL committed 24 rosters and sidecar cutoffs, reconstruct/verify every committed state, then return a validated pure payload. Reject persisted corruption, orphan/extra commits or sidecar rows, unknown sections and rows beyond final prefixes; never filter away retained rows to manufacture a valid export. Do not invent cutoffs from vector length or silently export only head. No descriptor/context lookup or actual foreign corpus read. Export never edits/migrates a journal.

Fresh import: the parent exists and ROOT is absent; exclusive create_dir, same leaf/family/path protections and failed-partial-root retention as S01. Write the ENTIRE history in ONE BEGIN IMMEDIATE transaction with DELETE/FULL/FKON and busy0, same rollback/uncertainty/unwind guard. Do not implement repeated public create/commit/migrate calls that can publish a partial imported history. Reuse private validated insertion/schema helpers as needed without nested transactions. Write each final base/curation record once plus all exact commit/prefix rows, preserving original IDs/hashes/flags/editor/notice/raw values. Commit only complete valid schema1 or schema2. Schema2 import initializes genuine sidecar schema in that same transaction; it does not run public migration as a second phase.

Existing ROOT: EXPLICIT EXACT REPLAY ONLY. Open the actual family/schema, BEGIN IMMEDIATE, re-read persisted head/schema, export/fully verify its actual history on the SAME connection, compare canonical whole snapshot bytes/schema/source/history/curation. Equal returns AlreadyPresent with NO data/schema mutation. No comparison solely by current head/hash. Any different history, same-ID/body conflict, older prefix or newer extension returns ImportConflict and leaves logical state unchanged. Import NEVER appends to an existing journal and NEVER silently migrates/downgrades schema1↔2; normal edits and explicit S02 migration retain those responsibilities. Caller can choose a fresh destination. This policy deliberately separates S01 stale-edit Conflict from exact sealed replay.

New named OwnedJournalError/category InvalidSnapshot(OwnedSnapshotError) for invalid incoming sealed input and ImportConflict(String) for a valid unequal existing history; retain all old categories. Export's persisted malformed history remains Integrity/source-context error, not mislabelled invalid caller JSON. Reconcile exact accepted S02 error plumbing before issue. On stdout failure AFTER committed import, inspect head or repeat EXACT same sealed input to get AlreadyPresent; don't promise rollback or auto-retry.

## 5. CLI and one usable journey

Proposed explicit/native paths, no defaults/config/network/JSON argv:

```text
owned export ROOT --output PAYLOAD --seal SEAL
owned check --input PAYLOAD --seal SEAL
owned import ROOT --input PAYLOAD --seal SEAL
owned snapshot-model --input PAYLOAD --seal SEAL [--revision REV] --output MODEL
```

export uses a pinned current journal and includes its ENTIRE retained history, not an optional truncated --revision export. snapshot-model is PURE sealed loading plus exact requested BASE Dictionary file; it explicitly does not project the external curation adapter. To query materialized reviewed external relations, import then use S02 owned materialize with exact context. Schema1 base extraction needs no artificial curation migration. Existing owned show on the imported journal still returns complete current/historical wrappers.

Static ID/Clap validation comes before input/journal/output IO as appropriate; sealed bytes are strictly loaded before import creates/opens a journal. check and snapshot-model take NO ROOT and must not call default resolution. Existing owned ManagedOutputContext needs optional root handling for these rootless operations; reuse the common Json writer/flush/BrokenPipe/error lifecycle, no new emitter. Public reports retain exact typed fieldsets and existing Json(Value) sorted map-key stdout plus one LF. Files use native Path/OsStr reports, never lossy UTF8 substitution. Existing dictionary-build retains its inherited UTF8 STORE-response restriction; use an explicitly owned UTF8 derived destination rather than widen that API here.

Create_new each output, never replace existing files. Validate completely before output creation; for export preflight both explicit outputs, open each exclusively before writes, write/sync exact canonical bytes. No portable atomic-two-file or power-loss promise: any output open/write/flush/sync error may leave owned partial files; do not delete arbitrary paths or falsely roll back a completed file/import. Payload/seal/model files have NO added LF. Reports use the exact prospective fieldsets below; accepted S02 source reconciliation may change implementation wiring but must not leave output policy to the author. Seal/load errors use existing owned_<category> envelope; output errors remain response_output_io. Partial-file and postcommit inspection boundaries must be documented.

### Exact prospective report and small base-history oracle

All report versions are1; new public enums serialize snake_case. Public store/lexicon reports remain typed serde APIs; public OwnedImportReport above is also the CLI import result (outcome exactly imported/already_present). Private CLI-only reports use the existing serde_json::json! → QueryOutput::Json(Value) route with the exact fieldsets/types below, not a new serde dependency or an unused private input decoder. Their field lists are logical declarations, NOT stdout key order. Whole output-fieldset assertions reject extra/missing keys; deny_unknown_fields applies to actual public input DTOs, not this output-only Value construction:

- Export: `{version,source,head,journal_schema,revision_count,output:NativePathReport,output_bytes:u64,output_sha256:String,seal:NativePathReport,seal_bytes:u64,seal_sha256:String}`.
- Check: `{version,source,head,journal_schema,revision_count,snapshot_bytes:u64,snapshot_sha256:String,seal_bytes:u64,seal_sha256:String}`; no ROOT or output path invented.
- Snapshot-model: `{version,source,revision,journal_schema,output:NativePathReport,output_bytes:u64,output_sha256:String}`; revision is requested or snapshot head, bytes/hash cover the exact extracted Dictionary file.

SHA fields are lowercase64; counts fit u64, revision_count is ACTUAL complete commit count. Export/import snapshot hash is the canonical complete payload hash, not a final-model hash. Check hashes the actual verified payload/seal input files (seal may have accepted whitespace); export hashes exact canonical emitted files. NativePathReport uses existing encoding/hex/utf8 semantics; stdout maps retain existing sorted-key order plus exactly one LF, file bytes noLF. Assert whole fieldsets and values, not selected labels or report-derived expected hashes.

A small schema1 golden reuses the accepted S01 public example inputs, preserved at `../slice01-owned-journal/artifacts/cdc-proof/journey/inputs/{seed,edit}.json`, rather than constructing an expected result from the new exporter. Source owned-example, editor editor-example, head owned-r1, journal_schema1, revision_count2, curationNone. Ordered revision IDs `[owned-r0,owned-r1]`; active assertions r0[] and r1[owned-assertion-a,owned-assertion-b], both tombstones[]. Exact whole base bytes/hashes are: r0 2003B/SHA178cb4a48245db47a568bb740f5ec123575aae38eb961c9ce05f40c7f0526c9a; r1 5555B/SHAe96d7fdee27a1cf84b152799b51c7b6e541aeae714396665fa98985f5c81d05f. These are retained INPUT bytes checked against accepted S01 output, not a new journal query. Current/old extraction must equal the respective whole inputs, not only these hashes.

Freeze actual24 prefix roster lengths for this golden in §2 order: r0 `[1,1,0,0,0,0,0,0,0,1,1,1,0,0,0,0,0,0,0,1,0,0,0,0]`; r1 `[1,1,0,0,0,1,2,1,0,1,1,2,1,2,0,0,0,0,0,2,0,0,0,0]`. owned_heads is a control field, not a25th append vector. This small literal does NOT replace the separate rich all24-populated/schema2/context fidelity fixtures. Expected full snapshot/seal bytes are independently constructed from these known typed input models, literal rosters and fixed schema/source/head before product export; no expected field comes from exported/candidate output. The prospective exact compact pair is now authored under artifacts/schema1-golden/snapshot.json (8040B/SHAa16d6f1c3112bb62fa3dd091d6a972e17c0f084dc433f4f3110584faef090a05) and seal.json (319B/SHA9e87f3df1a18dffc1f4f3e4418d64f76fb56e3653ff213d895320702ce090d21), independently assembled from those retained input bytes and literal rosters before any S03 implementation. The Dictionary field embeds accepted edit.json bytes DIRECTLY, without a Value roundtrip. These are expected-format fixtures, not product-produced exports or loader/implementation proof; no S03 executable has run. The schema2 literal with actual S02 pin/record/selection hashes remains explicitly pending accepted S02, as do exact example argv/readset and final source pins.

One shipped public example and corresponding actual CLI journey should reuse the accepted S02 Tiny identities/context: create/edit/curate a multi-revision schema2 journal, export twice and compare exact pair bytes, pure load and inspect ALL old base/curation states, import fresh/replay, reject a valid unequal snapshot unchanged, and inspect old revisions. Separately cover schema1 fresh import/extraction without forced migration. Write a base dictionary via snapshot-model and build an explicit derived store; import+S02 exact-context materialization supplies the reviewed cross-source view for existing queries. Compare Full/Compact/Readable Exact/Local/Catalog/detail to pre-input IDs/whole bodies and curation provenance; no oracle selected from query output. No live managed all-pack workbench or Arc06 promise. Use files/stdin for any model-sized input; small existing query/key JSON interfaces remain unchanged.

## 6. Finite meaningful proof and gates

- Pure schema1 and schema2 histories across zero/one/multiple edits; complete24 vectors/raw/Presence/ordered duplicates and ALL old model/hash equality. Prefix missing/duplicate/unknown/out-of-order/too-large/decreasing/final-short, wrong revision/source/head, prior-body/hash, new-active/tombstone/release-flag and curation creation/cutoff/selection negatives have valid restore counterparts. Outer seal recomputation must not admit invalid history.
- Exact repeated canonical payload+seal; payload one-byte/length/hash/version/kind/inventory/duplicate/nested-duplicate/noncanonical sealed-load failures, with bare-codec acceptance of the same valid pretty payload and exact canonical reserialization; generic Deserialize cannot bypass wrapper invariants. No signature/licence inference.
- Fresh import schema1/schema2 preserves whole canonical snapshot and every history state. Existing exact replay is no-change; same-head unequal body/curation, older history, extension and schema mismatch all conflict unchanged. Wrong family/missing parent/existing nonjournal/symlink paths never replaced. Inject late-write abort and pinned-reader Busy/explicit retry within existing fixture/SQL mechanisms; preserve all logical rows/control, no physical-byte rollback claim.
- Pinned export and deliberate writer Busy are distinct from successful refresh: reader holds old snapshot, writer cannot commit until it is dropped; explicit retry/new reader shows newer full export. Do not prescribe impossible successful concurrent DELETE reader/writer snapshots.
- Actual public example and CLI export/check/import/model extraction/current+old show; file/stdin/native paths where applicable, structured errors before ROOT/output creation, existing outputs preserved, postcommit/stdout/partial-file boundary. Mac capability limits and later unexecuted Linux are recorded separately; reuse actual accepted IO preflights.
- Exact derived queries use existing APIs, pre-input full metadata/detail keys, and external context intentionally supplied; missing context stays absent, not an export loss. Original S01/S02 regression suites remain in the affected set.

Exact allowlist/readset and literal report/recipe oracles must be reconciled against accepted S02 before issue. Provisional15 paths: lexicon lib/new owned_snapshot/new test plus existing owned_curation (share private owned-base validation/add pure existing digest method); store lib/owned/owned_schema/new snapshot test/new example/README; CLI main/new snapshot test/README; owned guide/root README. No Cargo dependencies/lock/PackId/pack merge/search/provider/upstream data/workflow changes.

Use the SAME guard/ONE target and explicit+1.98/locked/offline, owned HOME/XDG/TMPDIR with existing CARGO_HOME/RUSTUP_HOME/PATH pinned. Scoped stdin rustfmt→named copyback→fmt-check→lint before behavior, focused new+affected suites/example/real CLI, Make check/no-default, disjoint Synthetic allfeatures with asserted physical archive cwd BEFORE exec and exact candidate/archive byte equality, actual affected package FILE inventories. Preserve actual failed attempts/source applicability; no automatic hard-gate rerun or sixth pass. No Full/default/new cache/benchmark/network/push/PR/publication. Independent CDC proof/close/retro and later fresh Arc05/SAME canonical gates remain. This DRAFT does not authorize intake/source/index/target activity and does not claim S02 accepted.

## 7. Draft history

- 2026-10-29, 0.1: initial prospective policy/API and eight-row open set; root read the whole plan/ledger and accepted the bounded exact-history replay direction, with accepted S02 reconciliation still required.
- 2026-10-29, 0.2: corrected current S02 phase to implementation active and fixed the public distinction between full validating bare JSON decoding and canonical-byte equality at sealed loading. No issue, intake, source assignment or accepted S02 claim.
- 2026-10-29, 0.3: fixed prospective report fieldsets and a retained-input schema1 whole-history/prefix golden; schema2 actual accepted context/history oracle remains pending. Parent A05-03 stays partial until actual S03 curation reimport, preserving its original verifier.
- 2026-10-29, 0.4: reflected actual CLI dependencies: private output-only reports use existing json!/Value with exact fieldsets, not an unavailable serde derive/input decoder; public typed store/lexicon APIs and file/hash/transport oracles unchanged.
- 2026-10-29: added the two small prospective schema1 expected-byte fixtures from retained S01 inputs, with direct nested Dictionary bytes and literal rosters. No product export/load or runtime acceptance inferred.

### Prospective deterministic-metadata clarification —2026-10-29

The no-nondeterministic-export-metadata rule excludes only newly generated machine/time/random fields. It preserves all original typed timestamps/locators/native paths. This clarifies the original whole-state fidelity requirement; all eight binding criteria/verifiers/owners/statuses remain unchanged and OPEN. Accepted S02/API reconciliation and final readiness remain pending; no S03 issue or runtime.
