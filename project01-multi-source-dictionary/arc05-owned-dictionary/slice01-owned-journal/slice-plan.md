# S01 — Authoritative owned journal

| Field | Value |
| --- | --- |
| Status | CLOSED locally: seven rows DONE after independent CDC source/public/CLI verification at1b899ebb/tree745f |
| Parent | [Arc05](../arc-plan.md), [directive04](../../cdc-directive04.md) |
| Baseline | `a55e1cc7c4d7b0df4a187ce3551bfce9141a39d3`, tree `836df880ca711c6813cb78ae1202973e618eb105`; branch `codex/project01-owned-dictionary` |
| Source home | `/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc04/vlajmaji` (physical checkout reused) |
| Ownership | Author and CDC verification sessions terminal; source/index/ONEtarget QUIET, returned to root/CDC. No S02 transfer yet |
| Roles / budget | Retained Sol6.1/high CDC + Luna/high CC; initial + corrections at most FIVE total passes, no automatic sixth |
| Version / date | 0.2 / 2026-10-28 |

## 1. First usable result

A Rust caller and actual CLI can initialize an explicitly named personal dictionary, append a complete typed edit, inspect its head/history and retrieve any historical state. A shipped small example produces valid seed/edit JSON; callers need not reverse-engineer the complete test fixture. No upstream data, default store, live search wrapper, export/package, external context or network is required. The journal is authoritative; it never mutates an upstream/derived DictionaryStore.

This slice deliberately accepts a whole validated Dictionary rather than a strings-only editor or JSON Patch. Full model constructors/validators already exist in lexicon. A changed assertion/body receives a new ID; old bodies remain. S02 owns the separate external-curation envelope; S03 owns sealed export/reimport and explicit derived queries; S04 owns the optional package. Those are obligations, not promises that S01 already implements them.

## 2. Public Rust surface and wire

Add `crates/store/src/owned.rs` and private `owned_schema.rs`, re-export through store/lib.rs. No lexicon/search/schema/model/PackId change, new dependency or Cargo.lock change. Existing Dictionary and registry version1 are unchanged. All new report structs derive Debug, Clone, PartialEq, Eq, Serialize, Deserialize with deny_unknown_fields; fields appear in the order below. Enum wire names are snake_case. Handles are not Clone/Serialize; no unsafe Send/Sync implementation.

```rust
pub struct OwnedCommitSummary {
    pub sequence: u64,
    pub revision: OwnedRevisionRecord,
    pub state_sha256: String,
}
pub struct OwnedHeadReport {
    pub version: u32, // 1
    pub source: SourceId,
    pub head: OwnedCommitSummary,
}
pub struct OwnedHistoryReport {
    pub version: u32,
    pub source: SourceId,
    pub head: EntityId,
    pub commits: Vec<OwnedCommitSummary>, // genesis first, sequence ascending
}
pub struct OwnedRevisionView {
    pub version: u32,
    pub source: SourceId,
    pub commit: OwnedCommitSummary,
    pub dictionary: Dictionary,
}
pub enum OwnedWriteOperation { Init, Edit }
pub struct OwnedWriteReport {
    pub version: u32,
    pub operation: OwnedWriteOperation,
    pub source: SourceId,
    pub previous_head: Option<EntityId>,
    pub head: OwnedCommitSummary,
}
impl OwnedJournal {
    pub fn create(root: &Path, genesis: &Dictionary)
        -> Result<(Self, OwnedWriteReport), OwnedJournalError>;
    pub fn open(root: &Path) -> Result<Self, OwnedJournalError>;
    pub fn commit(&mut self, expected_head: &EntityId, candidate: &Dictionary)
        -> Result<OwnedWriteReport, OwnedJournalError>;
}
impl OwnedJournalReader {
    pub fn open(root: &Path) -> Result<Self, OwnedJournalError>;
    pub fn head(&self) -> Result<OwnedHeadReport, OwnedJournalError>;
    pub fn history(&self) -> Result<OwnedHistoryReport, OwnedJournalError>;
    pub fn show(&self, revision: &EntityId)
        -> Result<OwnedRevisionView, OwnedJournalError>;
    pub fn show_head(&self) -> Result<OwnedRevisionView, OwnedJournalError>;
}
```

OwnedJournal owns one writable connection without a retained transaction or cached authoritative head; reader owns one read-only pinned transaction, so one reader's head/history/show are from the same observed state. Open a new reader for refresh. Drop readers before expecting a writer commit under DELETE journalling. Neither open creates a missing root/database. `Dictionary::from_json_bytes` is the strict CLI input parser (UTF-8, duplicate keys and whole graph); do not substitute unverified parsing or a Value roundtrip for canonical model bytes.

`OwnedJournalError` is a public Debug/Display/Error, non_exhaustive enum with variants `InvalidRequest(String)`, `InvalidDictionary(DictionaryError)`, `UnsupportedVersion { found: u32 }`, `WrongFamily`, `Conflict { expected: EntityId, actual: EntityId }`, `ImmutableRecord { section: String, ordinal: u64 }`, `NotFound(EntityId)`, `Integrity(String)`, `Busy(String)`, `Io(std::io::Error)`, `Backend(rusqlite::Error)`, `CommitOutcomeUnknown(String)`. No Eq/serde requirement for backend errors. Public `category(&self) -> OwnedJournalErrorCategory` returns matching snake_case serializable vocabulary. Keep inner sources via Error::source. Busy/locked is distinguished from corrupt/malformed data and ordinary IO. CommitOutcomeUnknown is reserved for a failed COMMIT followed by inability to establish rollback; document inspection of the requested revision before retrying. Do not claim power-loss guarantees beyond SQLite's configured transaction/synchronous contract.

## 3. Owned input and append policy

**Genesis:** a whole validated Dictionary with `legacy_archive=None`, exactly one registered source of origin OwnedDraft, exactly one owned head and one owned revision for that source, revision.parent=None, revision.tombstones empty, metadata.catalogue_revision=0. The head points at that revision. Its editor exists in registry; timestamp/time_precision/rationale retain caller Presence exactly (unknown is permitted, never invented). Require at least one own collection/component/release and at least one content notice on each release; unknown licensing remains explicit Presence/RedistributionStatus, never an automatic grant. Registry source ID/display name/abbreviation/evidence are caller-declared, not generated linguistic authority. A valid zero-assertion initial release is supported.

**Each edit:** current source/header versions/legacy absence remain the same. Candidate has one head for the same source; exactly ONE newly appended same-source OwnedRevisionRecord, parent exactly the persisted current head, and that new revision is the candidate head. Catalogue revision increments by exactly one; journal sequences are restricted to 0..=i64::MAX for SQLite INTEGER, with checked conversion and rejection before mutation at that limit. All existing selected/availability release flags, memberships, raw bytes, metadata and registry bodies are immutable. Append a new release/record when its body changes. New assertions must appear in the new revision's active set; the active set is complete, not a delta. Its explicit tombstones are the previous active IDs absent from the new set, in previous active-set order. Reactivation of a retained old assertion is allowed only by explicitly including it in the new complete active set. No implicit resurrection or deletion. Existing normalized ancestry/reference/uniqueness/eligibility rules also apply.

Compare every old element/body and its order, not merely array lengths or record counts. For the 24 stored vectors below, the old typed vector must be an exact prefix of the candidate typed vector. Equal existing IDs with changed bytes/body reject, including metadata fields a query does not render. No removal/reordering, deduplication, normalisation, hidden value replacement or loss of duplicate anonymous rows. Existing SourceRecord cannot be renamed in place; caller appends appropriately identified provenance rather than rewriting history. The candidate registry still has exactly one source; own-source metadata must validate without foreign placeholder releases/assertions.

Strict edit replay: after parsing/ordinary Dictionary validation and opening the journal, compare expected head to persisted head before owned append-policy checks. A replay using an old expected head is Conflict even when its candidate equals a past successful edit. S03 import will own a separate explicit exact-replay policy; no result/error cache here.

## 4. Journal schema and reconstruction

ROOT is explicit, directory-based; database leaf is `owned.sqlite`. New SQLite family `vlajmaji-owned`, application_id `0x564c4f57`, user_version/schema1, owned format1; normalized model1/registry1. Reject a foreign family before interpreting its tables; unsupported owned versions return UnsupportedVersion. No migration of upstream/derived schemas. Open roots/database leaves reject symlinks, non-directory roots/non-regular DB files; all path handling uses native Path/OsStr. Do not promise a portable race-free filesystem sandbox. Create uses exclusive create_dir on a previously absent ROOT under an existing parent, never replaces an existing ROOT; preserve a failed partial creation for diagnosis rather than deleting arbitrary contents.

Use rusqlite already in store. DELETE journal mode, synchronous=FULL, foreign_keys=ON, busy_timeout=0 on writers; read-only connection never changes journal settings. No multi-file atomicity/WAL/immutable-flag claim. Tables (STRICT; fixed trusted identifiers; parameters for values):

- `owned_meta(singleton INTEGER PRIMARY KEY CHECK(singleton=1), family TEXT, schema_version INTEGER, model_version INTEGER, registry_version INTEGER, source_id TEXT, head_revision TEXT, catalogue_revision INTEGER)`; all non-PK columns NOT NULL; source/header fixed, final two are current control state.
- `owned_records(section TEXT, ordinal INTEGER CHECK(ordinal>=0), payload BLOB, PRIMARY KEY(section,ordinal))`, all NOT NULL; payload is exact typed serde_json row bytes. No updates/deletes through the API.
- `owned_commits(revision_id TEXT PRIMARY KEY, parent_revision TEXT, sequence INTEGER UNIQUE CHECK(sequence>=0), state_sha256 TEXT NOT NULL)`; revision_id/sequence NOT NULL, parent nullable. The matching typed revision row carries editor/time/rationale/active/tombstones, not a second independently mutable copy.
- `owned_commit_sections(revision_id TEXT REFERENCES owned_commits(revision_id), section TEXT, length INTEGER CHECK(length>=0), PRIMARY KEY(revision_id,section))`, all NOT NULL; exactly 24 section counts for each commit.

The 24 section names are exactly: `registry.sources`, `registry.contributors`, `registry.distributors`, `registry.aliases`, `registry.lineage`, `registry.snapshots`, `registry.occurrences`, `captures`, `metadata.code_notices`, `metadata.collections`, `metadata.components`, `metadata.releases`, `metadata.entries`, `metadata.assertions`, `metadata.contexts`, `metadata.frames`, `metadata.forms`, `metadata.etymologies`, `metadata.relations`, `metadata.owned_revisions`, `metadata.editorial_senses`, `metadata.computations`, `metadata.annotations`, `metadata.extensions`.

`metadata.owned_heads` is reconstructed as the one selected revision; catalogue_revision is that commit's sequence. Model/registry version1 and legacy_archive=None are fixed. Each historical state is its 24 typed ordered prefixes, not a mutable view over current records. Validate section roster/counts/contiguous ordinals/SQL types and all model relationships; reject missing/excess/unknown sections, invalid parent/sequence/head, malformed payload or wrong hash. Prefix counts alone are not body integrity proof: lowercase SHA256 of `Dictionary::to_json_bytes()` of the reconstructed validated state must equal each requested commit's stored state_sha256. `open` checks only family/version/minimum header, without rebuilding a model. `head` reads a checked current commit/revision summary; `history` validates the contiguous commit chain and typed revision summaries (source, parent, sequence, head linkage and hash syntax), not every historical Dictionary. Neither summary command promises full body/hash verification. `show`/`show_head` explicitly reconstruct and fully validate/hash the requested state; later export/check owns whole-history verification. Tests may explicitly show each state. Do not hide expensive whole-model checking in summary/read-open paths. Never silently repair corruption. Private typed section adapters can reduce repetition, but no generic parser framework or arbitrary SQL identifiers from input.

This stores each appended record body once plus 24 small prefix counts per revision. Whole owned candidate validation still costs its own size; no performance bound or Full-corpus copy is promised. Captures/native lexemes/Presence/ordered extension multiplicity remain full fidelity. The authoritative row bytes/hash are not redefined as a new normalized model version.

## 5. Transaction and failure recipe

Create validates genesis/owned policy before making ROOT, then creates family/schema/genesis rows/head in one transaction. On any schema/data failure return an honest error and leave no logically committed journal; preserve any owned partial ROOT. Existing ROOT fails without replacement. Open read-write uses no CREATE; recheck family/version before writes.

Commit sequence is consequential:

1. BEGIN IMMEDIATE; read persisted header/head and reconstruct/validate current state INSIDE this transaction. Never trust a head cached at open. If expected differs, return exact Conflict and rollback; do not compare candidate against a stale in-memory base.
2. Validate owned control changes and all 24 typed old-body prefixes; require one new revision and exact active/tombstone policy. Compute validated candidate canonical SHA. No DB mutation yet.
3. Insert only suffix record rows, immutable commit row and 24 prefix lengths. UPDATE owned_meta head/catalogue_revision using the old head in the predicate; exactly one affected row. The transaction already serializes the decision; retain the conditional update as a defensive invariant.
4. COMMIT through the still-borrowed Connection, not a consumed rusqlite Transaction. On earlier failure rollback all inserted/control rows. Use a private transaction guard over `&Connection`: execute_batch("BEGIN IMMEDIATE"), run the fallible body, then execute_batch("COMMIT"). After COMMIT error, if `!is_autocommit()`, execute_batch("ROLLBACK") and verify is_autocommit(); only a successful established rollback returns the original typed Busy/backend error. If COMMIT errored with autocommit already true or rollback cannot be established, return CommitOutcomeUnknown and poison/close this writer handle for further commits. Reopen and inspect requested head/revision before another action. Before COMMIT was attempted, an already-ended transaction is an automatic rollback, not a commit. A private guard has best-effort rollback in Drop only for unwind protection; ordinary Result paths explicitly observe rollback failures. Do not call tx.rollback() after consuming tx.commit(). Busy/locked with successful rollback reports Busy. Never automatically retry a stale edit or change expected head.

Read-only snapshots consistently expose old state while held. A held reader can make a DELETE-mode writer commit Busy; after rollback the writer can be invoked deliberately again after reader drop with the same expected head. This is an explicit new caller action, not hidden retry. No callbacks or external effects inside a transaction.

## 6. CLI and usable example

Add one nested `owned` command to current `Command`/`execute` in `crates/cli/src/main.rs`, using explicit native PathBuf ROOT for EVERY operation:

```text
vlajmaji owned init ROOT [--input FILE]
vlajmaji owned edit ROOT --expected-head REVISION [--input FILE]
vlajmaji owned head ROOT
vlajmaji owned history ROOT
vlajmaji owned show ROOT [--revision REVISION]
```

For init/edit, --input reads the exact file bytes; absent --input reads stdin. Never a JSON argv value, automatic default ROOT, configuration lookup, import, download or silent create on read/edit. Parse EntityId and strict Dictionary input before any journal IO; invalid JSON/ID exits1 using existing structured-error path, except Clap usage errors exit2. After parsing, domain Conflict precedes append-policy errors as §3. Show without --revision is current pinned head. Successful stdout is the direct typed report converted at the existing Json transport boundary (one compact JSON object +LF), no new wrapper protocol or readable option. Errors reuse `CliError::Structured` envelope: `{"error":{"code":"owned_<category>","message":"...","sources":[]}}`; exact Conflict expected/actual IDs remain available in message and Rust variant. No changes to existing managed/query wire/opcodes.

Reuse shared QueryOutput::Json/emit_success_response write/flush handling. BrokenPipe remains existing success policy; other output IO exits1. A write can be durably committed before stdout acknowledgment fails; **output failure does not roll back or prove edit failure**. Inspect known requested revision/head before retry; stale replay remains Conflict. Existing structured errors and emitter tests are reused, with an owned edit output-failure test checking committed head rather than promising rollback. No second writer/output protocol.

Ship `crates/store/examples/owned_journal.rs`: construct a minimal explicitly named OwnedDraft source/editor/collection/component/notice/release and genesis r0, then a capture with two same-spelling assertions with distinct IDs/locators and metadata (Missing/Null/Value examples) in r1. Use existing typed constructors, strict validate both, write seed.json/edit.json with create_new under an exclusive supplied directory; demonstrate public create/commit/head/history/show in a DIFFERENT explicit ROOT, or a clearly separated example mode. Use explicit IDs `owned-example`, `editor-example`, `owned-collection`, `owned-component`, `owned-release-r0`, `owned-release-r1`, `owned-r0`, `owned-r1`, `owned-entry-farvi`, `owned-assertion-a`, `owned-assertion-b`; project-assigned abbreviation OWN, personal display name, unknown time precision/Presence where not supplied, Held redistribution with explicit example-only/no-licence-grant notice. Genesis has one zero-row available/selected release and r0 active[]; edit appends one raw capture with `/a` and `/b`, one entry spelling farvi, assertions a/b with distinct retained definitions “grow”/“develop”, new release membership[a,b], r1 active[a,b]/tombstones[]. Keep both old release/header and r0 exact. The example accepts `INPUTS_DIR JOURNAL_ROOT`, creates INPUTS_DIR exclusively, emits seed.json/edit.json, then executes public create/commit/read/show on JOURNAL_ROOT and prints typed reports. The documented CLI recipe uses a separate CLI_ROOT so it is genuinely fresh. No complete_fixture import/private helper dependency. Document exact CLI use of these emitted files, expected r0→r1/history[0,1]/old r0 bodies. Keep the example small and source-backed; no new export schema or per-command manifest.

## 7. Finite regression controls

Use actual public integration tests and existing SQLite/native-path patterns; helpers create ROOTs exclusively with bounded counter retries on AlreadyExists, not PID/time-only names. Existing baseline fixture fragility remains a risk if observed, not something this slice claims to repair globally.

| Control | Setup/action and exact oracle; rejected implementation |
| --- | --- |
| Genesis and errors | Shipped small seed round-trips; zero assertions allowed; explicit source/editor/notices retained. Wrong Upstream source, unknown editor, missing notice/release, non-genesis parent/multiple heads/version/duplicate JSON keys reject; no new ROOT for parsed-invalid input. Wrong existing family/version and existing ROOT never replaced |
| Two handles / replay | Open writers A+B at r0 BEFORE A commits r1. B's expected r0→candidate r2 yields Conflict{r0,r1}; current Dictionary/history/records/control unchanged. Same successful r1 candidate replay with expected r0 also Conflict. No pinned reader exists during this successful CAS sequence. Reopen/new reader sees r1. A separate held-reader case belongs to the Busy row; do not require an old pinned r0 reader to coexist with successful r1 commit in DELETE mode. Reject cached-self-head/LWW |
| Complete fidelity | Adapt existing portable complete_fixture for own-only data or build an own fixture spanning ALL 24 vectors, full RawCapture/native fields, every Presence state and ordered duplicate extension rows. Append valid records and one revision; compare entire typed candidate and every historical state. Mutate/reorder/remove one existing row in each applicable section (source/relationship/control constraints kept valid where possible) and assert ImmutableRecord with unchanged state. Core validators still reject invalid graphs before IO; distinguish that route from valid-but-immutable mutation |
| Active/tombstones | r1 active[a,b]; r2 creates c, active[b,c], tombstones[a]. Old a remains exact and r1 remains[a,b]. Wrong omitted/extra tombstone, changed prior body and multiple new revisions reject; explicit later reactivation tests policy. Selected release flags unchanged, no hidden unselecting |
| Atomic failure / busy | Install a test-only SQLite abort trigger on final head UPDATE, after valid suffix inserts would occur; commit fails, then public show/history plus SQL record/commit/count roster equals prewrite logical state. Drop trigger, same candidate succeeds. Separately hold reader to force Busy at commit; no partial head/rows, drop reader and explicit caller retry succeeds. No physical SQLite-byte rollback claim |
| Corruption / historical | Tamper one persisted historical row/body or prefix/hash/head via test SQLite connection; show/requested full verification rejects affected corrupt states; head/history reject malformed summary/chain/header state within their declared scope, without pretending to detect arbitrary assertion-body corruption. Never silently substitute latest. Missing revision gives NotFound. Valid restore counterpart works. No test-only public accessors |
| Actual CLI | Example files→init/edit/head/history/show default+explicit revision, stdin/file, stale writer/replay/error no mutation; exact typed reports match pre-input model. Unix native/non-UTF8 ROOT and INPUT path, deleted cwd with absolute paths, actual output write+flush/BrokenPipe policy where supported; IO failure after edit leaves r1 committed. No actual default access; owned HOME/XDG/TMPDIR set per child |

Existing complete_fixture is `crates/lexicon/examples/support/dictionary_fixture.rs::complete_fixture`; a portable #[path] import already exists in search tests. Do not assume its upstream/editorial relationships are valid after merely deleting foreign rows; construct a closed own model or explicitly remap every dependent reference. All-family vector coverage is coverage of the claimed metadata frontier, not a demand for a new test framework. Report actual Rust tests and assertion/control coverage separately.

## 8. Source allowlist and gates

Initial source allowlist (ELEVEN paths; no others without explicit CDC/root disposition):

```text
crates/store/src/lib.rs
crates/store/src/owned.rs
crates/store/src/owned_schema.rs
crates/store/tests/owned_journal_contract.rs
crates/store/examples/owned_journal.rs
crates/store/README.md
crates/cli/src/main.rs
crates/cli/tests/owned_journal_contract.rs
crates/cli/README.md
docs/owned-dictionary.md
README.md
```

No Cargo.toml/Cargo.lock, lexicon/model/search/PackId/data paths, existing upstream/derived schema, Bauske, workflow or unrelated formatting edits. Use rustfmt1.98 on allowed file bytes through stdin→scratch→copy back ONLY the named path for module roots; no recursive formatting spill. `make fmt-check` is read-only. Root/CDC review actual delta before freeze. Source commit uses named --only paths and adjacent Codex/Billo trailers. Current root readiness BEFORE issue and actual complete CC intake BEFORE transfer remain gates.

Future gate environment is an exclusive `/tmp/vlajmaji-p01-a05-s01-<unique>/` with owned HOME/XDG_CONFIG_HOME/XDG_DATA_HOME/TMPDIR. Before substituting HOME bind existing CARGO_HOME=/Users/oubiwann/.cargo, RUSTUP_HOME=/Users/oubiwann/.rustup, unchanged PATH including installed tools; verify actual homes/tool paths at intake without downloads. Guard sets ONE target/offline. Use `make CARGO=/tmp/vlajmaji-p01-a05-cargo RUST_TOOLCHAIN=1.98.0 ...`; direct commands use `/tmp/vlajmaji-p01-a05-cargo +1.98.0 ... --locked --offline`. Gate order:

1. Scoped rustfmt, `make fmt-check`, then `make lint` BEFORE behavior tests.
2. Guarded `test -p vlajmaji-store --test owned_journal_contract` and `test -p vlajmaji-cli --test owned_journal_contract`; run shipped example and documented real CLI sequence, capture actual argv/exits/source identity.
3. `make check` (default Rust/doc/tooling), `make test-no-default`; store/CLI changed APIs do not waive existing search isolation.
4. All-features uses a DISJOINT task-owned source archive: archive current HEAD then overlay ONLY allowed current candidate files (including new files); record candidate delta identity/equality, keep ignored files excluded. Place under owned feature-root/vlajmaji with the known read-only Bauske sibling link. Use the SAME target/guard. In that archive only, existing `data-pack fixture jbotci|llg|la-lojban|sutysisku` commands from CI materialize the four Synthetic crates/data destinations, then `test --workspace --all-features --locked --offline`. Do not run materialization in reused source checkout or inspect ignored Full materializations. Record archive-to-candidate Rust/manifest equality; synthetic sources/fixture counts are independent of Full acceptance.
5. Actual `cargo package -p vlajmaji-store --list --allow-dirty --locked --offline` and corresponding vlajmaji-cli command (PACKAGE FILE inventory, not test --list). No publish/registry/verify claim. Review docs/CLI help and diff scope.

No RELOJBAN_RESEARCH_ROOT/Full import/model/store/benchmark/default read, cache reset/new target, external action or automatic failed-gate retry. Preserve failure logs and return concrete disposition; baseline setup fragility does not authorise an unbounded rerun. Actual Linux execution is a later hosted gate only when separately authorised; macOS source/test proof is not Linux execution. Mark absent proof honestly.

## 9. Intake, disposition and return

DRAFT is not executable. After root readiness/status-only issue, retained CC loads prompt/this plan/ledger/directive04 FULL, current parent sections and exact source bodies in prompt, records loaded ranges/truncation recovery and source-cited readback in artifacts/execution.md, then CDC verifies coverage and fresh source/pins before explicit eleven-path/index/one-target transfer. No new agent/model question. Keep old arc megabytes reference-only.

Return clean branch HEAD/tree/parent, exact paths/body hashes/lock+guard, real gate argv/exits/raw locations and scope, public API/CLI sequence, row walk proposed only, limitations and target QUIET. CC closing report, CDC independent source/public proof and child retrospective are written only after actual work. No prewritten success record, silent status closure or speculative speed claim. Root still gates current issue readiness and final arc consistency; existing user authority covers ordinary local work only through Arc05.

Durable intake/log evidence lives in this slice’s `artifacts/execution.md` and relevant logs under `artifacts/`; do not create a root execution.md override. Create closing-report.md/CDC verifier/retro only after actual corresponding work, under existing scale conventions. No routine per-command exports or new proof framework.

## 10. Proposal readiness / history

CDC source inspection at accepted tree836 (now merged baseline a55) establishes existing complete model/owned validator, read-only derived store, absent in-memory query constructor, actual CLI/Make/feature/package paths. Relevant Rust guidance: AP18/API02 support borrowing input and storing appended bodies once; AP19/EH04/EH13 support the named non_exhaustive library error; AP33 requires explicit rollback/commit-error handling rather than discarded failures; EH14 requires Errors docs; API04 supports reusing strict byte parsing for file/stdin. API03 is SHOULD: the concrete borrowed &Path signatures deliberately follow existing store APIs rather than changing handle types. AP67 supports typed revision/head IDs and operation enums. No new error dependency or unsafe Send/Sync promise is introduced. The complete draft resolves S01 input/schema/CAS/mutability/read/failure/CLI policies; remaining private implementation choices cannot change those outcomes. S02 exact external-context adapter is later work with binding arc obligations; it does not block standalone S01. Root initial full-draft review identified and this draft corrects F01 (separate no-reader CAS vs reader Busy) and F02 (summary commands avoid whole-history model verification), with Connection-level rollback handling checked against local rusqlite0.40.2 transaction.rs190–230/lib.rs548/is_autocommit1059. F03 evidence-home/registry-helper intake closure is corrected. Root final exact draft review remains PENDING; no acceptance or source edits claimed.

- 0.1 — 2026-10-28: concrete DRAFT open set; whole typed candidates and append-prefix journal selected, explicit example/CLI route and finite negative controls included. Five total passes, no Full campaign.

### Initial issue / assignment history — 2026-10-28

Root final DRAFT READINESS PASS reviewed exact proposal d75532d660bf5b14232d684528165c80057e719c/tree9b57bc9266f8fc12a4536c2386da9da39ca34447: whole S01 plan/prompt/ledger, parent bodies/deltas, F01–F03 and viable Connection rollback recipe. Root independently verified ten paths, unchanged14projectrows,8arc/7childOPEN, clean a55/tree836/lock65cd/guard426d. Original substantive reviewed prompt/plan are preserved in d75532d; active prompt differs ONLY its STATUS line. This metadata records readiness, not product acceptance.

Retained Luna/high CC is assigned REQUIRED COMPLETE INTAKE ONLY now. Intake coverage/source-cited readback belongs in artifacts/execution.md. CDC accepts actual coverage and rechecks fresh branch/source/lock/guard before EXPLICIT eleven-path/index/ONE-target transfer. Until then root owns source/index/target QUIET; no source/Cargo/fixture action. No further root ceremony is required absent an actual mismatch. Five-total-pass budget and all rows OPEN unchanged.

### Initial intake acceptance / handoff — 2026-10-28

Actual bounded intake/readback plus citation erratum in artifacts/execution.md accepted by CDC; fresh clean a55/tree836/branch/lock/guard/protected refs verified. After this metadata commit and explicit CDC dispatch, retained Luna/high owns exactly plan§8 eleven paths/index/ONE target for INITIAL/pass1of5. Root/CDC source-read-only, all rows OPEN; no root approval ceremony repeats. Substantive issued prompt remains byte-exact.

## Version0.2 — actual S01 local close,2026-10-28

CDC accepted1b899ebb/tree745f (same source tree as originalf35, adjacent-trailer metadata correction only) after complete source review, independent15store+3CLI and12-child public/CLI/new-owned-schema proof. Seven rows DONE; author and CDC raw failures/current gates retained in artifacts. cdc-verification.md defines exact scopes and unknown incidental ignored-input I/O deviation; cdc-retrospective.md supplies follow-through. Earlier readiness/intake/partial checkpoints retain their original time cutoffs. No hosted Linux/Full/export/curation/package/Arc05 or project acceptance inferred. S02 separate draft follows root readiness.
