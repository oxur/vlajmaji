# S04 — Optional owned snapshot package and offline consumer

| Field | Value |
| --- | --- |
| Status | PROSPECTIVE DRAFT ONLY, UNISSUED. S02 locally CLOSED; S03 INITIAL implementation active with exclusive CC15/index/ONEtarget ownership, no accepted S03 dependency yet |
| Authority | directive04 THROUGH ARC05 ONLY; retained Sol6.1/high CDC and Luna/high CC; five S04 passes including initial, no automatic sixth |
| Known source | Accepted S02 854865a7/treeef81ffbb; final accepted S03 source and exact snapshot APIs must replace the remaining dependency placeholder before issue |
| Useful result | Caller-selected owned history can be embedded and strictly loaded by an offline consumer whose direct dependency graph excludes store/search/SQLite; the existing CLI prepares the exact local pair |
| Version / date | 0.3 / 2026-10-29; first unissued prompt prepared, accepted S03/readset/consumer buffers/root readiness still pending |

## 1. Source-grounded package boundary

Existing data-llg/data-lensisku-jbotci manifests are workspace0.3/Rust1.98, publish=false, default features empty, embedded feature explicit, direct dependency lexicon only and exclude data/** from package files. Their lib uses compile-time include_bytes behind embedded, exposes exact bytes and delegates to the strict pure loader. Umbrella vlajmaji already depends on store/search independently of optional data; an umbrella consumer alone cannot prove a before-SQLite dependency boundary.

Add analogous vlajmaji-data-owned, retaining its DISTINCT owned snapshot kind and history API. Do not add PackId, PACK_ID, a fifth All installation input, hidden catalogue registration, context acquisition, a default journal or normalized-model version change. Owned bodies/editor/content notices remain caller-declared; code licensing does not grant redistribution permission for included content. This slice packages the accepted S03 pair; it does not invent a second export format or append foreign corpora to each owned edit.

## 2. Proposed exact crate and umbrella shape

New crates/data-owned/Cargo.toml: name vlajmaji-data-owned; workspace version/edition/rust-version/license/repository; publish=false; README; exclude=["data/**"]; features default=[] and embedded=[]; direct dependency ONLY vlajmaji-lexicon.workspace=true; workspace lints. Workspace members=["crates/*"] already discovers it, so do not add a redundant member or unrelated dependency entry. Cargo.lock will gain only the actual local package/optional umbrella edge and any necessary lock metadata from the approved locked/offline procedure; no dependency upgrades/network/backend changes. Before issue settle how the deliberate new workspace member's initial lock update is performed once, then all gates --locked --offline; do not prescribe a knowingly failing --locked update.

Public API in new src/lib.rs, with documentation and no payload access in default build:

```rust
pub const fn is_embedded() -> bool; // cfg!(feature="embedded")
#[cfg(feature="embedded")]
pub fn snapshot_bytes() -> &'static [u8]; // include_bytes!("../data/snapshot.json")
#[cfg(feature="embedded")]
pub fn seal_bytes() -> &'static [u8]; // include_bytes!("../data/seal.json")
#[cfg(feature="embedded")]
pub fn load() -> Result<OwnedSnapshot,OwnedSnapshotError>;
```

load calls accepted S03 load_owned_snapshot(snapshot_bytes(),seal_bytes()) directly, preserving its precise typed errors, seal kind/inventory, canonical-payload and WHOLE-history checks. It returns the validated history wrapper, not just the final Dictionary. No store/search dependency, runtime file lookup, OnceLock cache, fallback data or unverified serde decode. Reconcile exact S03 names before issue; these signatures are a prospective policy.

Umbrella Cargo adds optional path/version dependency vlajmaji-data-owned and data-owned=["dep:vlajmaji-data-owned","vlajmaji-data-owned/embedded"]. cfg(feature="data-owned") re-export as data_owned. Defaults remain []. Existing four data features, graph/morphology, PackId4/All4 behavior and all existing exports remain byte-compatible. Add /crates/data-owned/data/ to .gitignore. Do not commit snapshot/seal bytes, SQLite files, build output or machine paths.

## 3. Actual preparation and consumer increment

Use accepted owned CLI to create/edit/curate an explicit task-owned journal, then S03 owned export ROOT --output DATA_DIR/snapshot.json --seal DATA_DIR/seal.json. Create DATA_DIR explicitly beneath a NEW disjoint source archive for feature proof. Existing export's create_new outputs must reject replacement. No new data-pack owned command: existing owned export already prepares the exact two files; keep All-four fixture/install dispatch untouched. Do not prepare or stat ignored Full materializations in the reused source checkout. The data-owned README must give the real public example + CLI inputs and owned export recipe with fresh directories, not require users to reverse-engineer tests.

One direct outside-workspace Rust consumer declares only vlajmaji-data-owned (path, embedded) and vlajmaji-lexicon (path), uses the existing ONE target, locked/offline and pinned toolchain. It loads and compares the full canonical snapshot and EVERY old base/curation state/hash to the pre-input synthetic authority, plus strict tamper/unsupported-version/invalid-history counterparts using the same loader and copied byte buffers. Pin original expected history before loading; do not obtain the oracle from package.load. Exact loaded wrapper bytes must equal S03 export pair input. Missing external context remains retained curation/pins; loading is not materialization or evidence external bodies were embedded.

The consumer resolved normal dependency graph must contain the intended lexicon/serde/hash closure and contain NO vlajmaji-store/vlajmaji-search/rusqlite/libsqlite3-sys/oxigraph. Inspect the actual Cargo.lock/tree metadata, not only source imports. This is the before-SQLite proof. A separate umbrella feature smoke check proves re-export/load wiring; its store dependency is expected and must not be misrepresented as the isolated graph. Do not introduce a second independent qualifier campaign: this finite outside consumer is itself the actual proof.

Continue the accepted existing CLI workflow from the same synthetic owned input: export/load/import/history, explicit context materialize and explicit fresh derived DictionaryStore/query. Retain exact IDs/whole metadata/old assertions/detail references from inputs; no default-store fallback. These APIs/CLI already belong to S01–S03; reuse their meaningful workflow rather than invent a new live managed integration (Arc06).

## 4. Finite proof and packaging

- Clean/default data-owned compiles/tests with no data directory present, is_embedded=false; byte/load symbols remain cfg-gated. Optional umbrella default has no owned payload edge. No compile-fail harness is required solely to mirror cfg implementation.
- Direct embedded consumer and umbrella re-export load the same accepted schema1/schema2 Tiny history as applicable, preserving whole old states/curation, source/editor/notices/raw/Presence and deterministic pair bytes; use S03's existing full-history oracles, not a new fixture corpus.
- Actual pure loader negatives must discriminate sealed tamper/version/kind/history failure; package never bypasses strict loading. Unsupported/error input is tested through loader buffer calls, not by rebuilding the crate once per malformed embedded pair.
- Inspect actual package FILE inventory: no data/**, database, scratch or generated payload is distributable; publish=false remains. This is not a publication or legal clearance. README/code notice distinction stays explicit.
- Default/no-default and affected individual data-owned plus combined existing-four/data-owned/morphology/graph features retain existing contracts. All embedded/combined checks run ONLY in the reviewed disjoint Synthetic archive, with accepted upstream fixture commands jbotci/llg/la-lojban/sutysisku and owned export. Assert physical cwd equals the archive immediately BEFORE every Cargo exec and pass that workdir. Record actual source/archive equality for changed source/manifests; no source ignored-data probes.
- Existing PackId/All remains four, and owned export/load does not alter registrations/config/defaults. No copying/redistributing foreign context data by implication.

### Existing CI feature preparation compatibility

Actual .github/workflows/ci.yml Test job prepares exactly four upstream synthetic packs, then runs `cargo test --workspace --all-features --locked` on Linux. The new embedded owned feature cannot compile in that clean job without its own pair. Include EXACT `.github/workflows/ci.yml` in the later S04 source allowlist for ONE synthetic owned-preparation step immediately before the unchanged all-features test; no trigger/job/backend/four-pack fixture or hosted-delivery change. Directive04 permits this local compatibility edit, while external CI dispatch/delivery remains unauthorized.

Freeze this step against accepted S03 signatures before issue (the current source-guided prospective recipe is):

```yaml
      - name: Prepare synthetic owned snapshot for all-feature tests
        if: runner.os == 'Linux'
        run: |
          owned_tmp=$(mktemp -d "${RUNNER_TEMP}/vlajmaji-owned-XXXXXX")
          cargo run --locked --offline -p vlajmaji-store --example owned_curation -- "$owned_tmp/inputs" "$owned_tmp/journal" "$owned_tmp/materialized.json"
          mkdir -p crates/data-owned/data
          cargo run --locked --offline -p vlajmaji-cli -- owned export "$owned_tmp/journal" --output crates/data-owned/data/snapshot.json --seal crates/data-owned/data/seal.json
```

The example performs only its explicit synthetic normalization/journal/curation work; export creates the distinct pair exclusively. The external context remains a pin rather than an embedded foreign corpus. Existing output files must fail create_new rather than be overwritten. Preserve the existing four PackId/All fixture routes and all-features command. Reproduce this SAME command body locally once in the asserted disjoint Synthetic archive, with the same owned environment/target/guard and physical-cwd assertion before Cargo; no hosted action/network/default/retained-data probe follows this draft. This is original feature compatibility, not a new qualifier campaign or a change to S02's fourteen-path transfer.

Exact source allowlist and required-reading manifest wait accepted S03. Expected homes: new data-owned manifest/README/lib/test, umbrella manifest/lib/README, lock, .gitignore, root README/owned guide, the exact existing CI workflow preparation step above and existing public workflow example/test only if actual integration needs it. No normalized model/PackId/pack merge/provider/upstream loader/SQL/search changes. Use an exact finite list before issue, not open-ended directories. No productsource assignment or build follows this draft.

## 5. Gates and arc endpoint

Scoped stdin rustfmt→named copyback→ONE explicit offline default-empty lock bootstrap as the FIRST Cargo metadata/build operation→fmt-check→lint, focused new/affected proof and one outside consumer, actual CLI example, required Make/no-default, disjoint Synthetic feature checks and affected package FILE lists. Pin existing CARGO_HOME/RUSTUP_HOME/PATH before owned HOME/XDG/TMPDIR and use unchanged Arc05 guard/ONE retained target/+1.98.0/offline. New fixtures use exclusive creation; no target reset/download/new cache. Preserve actual failed attempts and applicability. No benchmark/Full/default/network/push/PR/publication or automatic sixth pass.

CDC independently reviews actual candidate and reproduces the affected public+CLI increment before child close. Arc05 then needs a fresh independent Sol/high context to execute the composed init/edit/conflict/curation/export/import/optional-load/explicit-query workflow, followed by SAME-context canonical review, root consistency and mechanical closure/retrospectives/project bubble-up. All fourteen project rows remain OPEN; Arc06 unactivated. This draft contains no success record or advance acceptance.

### Version0.2 — existing all-features preparation,2026-10-29

Root source review identified the clean CI embedded-input prerequisite before issue. The exact workflow path and one offline synthetic owned-preparation step are now prospective S04 scope; no blanket workflow exclusion applies to this compatibility step. No source/workflow edit, Cargo/hosted run or intake occurred. All eight original binding rows remain OPEN; accepted S03/signatures/allowlist/literal consumer and lock procedure remain pending.


### Prospective integration refinements —2026-10-29

Current dependency metadata: S02 is locally CLOSED at854865a7/planninge9b615ed; S03 INITIAL is issued atf51826d for complete intake only. This is not accepted S03 source or S04 issue authority. Exact API/source/body pins and final S04 readset/consumer buffers/feature matrix still wait accepted S03.

The normal package embedded tests must validate the supplied pair generically rather than assume fixed personal fixture IDs, assertion counts or r4: the existing CI preparation deliberately uses accepted self-contained owned_curation Tiny example, whereas the independent S04 consumer can embed the richer S03 own-a/b history as its pre-input authority. These are distinct explicit inputs to the SAME strict loader, not permission to weaken the consumer's whole historical oracle. No own assertions/history claim comes merely from the CI example.

A concrete later lock bootstrap may use ONE `guard +1.98.0 check -p vlajmaji-data-owned --offline` after the new default-empty crate/optional edge is authored and safe stdin-rustfmt/named copyback completed, as the FIRST Cargo metadata/build operation, with the sole intended Cargo.lock change being the new local package and umbrella edge. Do not use generate-lockfile to re-resolve the graph, or intentionally fail a locked command first. Compare every preexisting package name/version/source/checksum/dependency record to the accepted lock, except the one declared vlajmaji umbrella record gains exactly vlajmaji-data-owned. Stop if any unrelated record/edge changes. After this explicit bootstrap, all product commands are locked/offline. This is prospective local command design, NOT a Cargo run or a promise that an unexecuted resolution succeeded. The outside consumer's private new lock is separately task-owned and must prove its actual normal closure contains no store/search/SQLite; it cannot claim that boundary from umbrella imports.


### Bounded source/matrix draft for accepted-S03 reconciliation

Proposed exact TWELVE product paths (four new/eight existing), to be rechecked at actual accepted S03 before issue: `.github/workflows/ci.yml`, `.gitignore`, `Cargo.lock`, `README.md`, `docs/owned-dictionary.md`, NEW `crates/data-owned/Cargo.toml`, NEW `crates/data-owned/README.md`, NEW `crates/data-owned/src/lib.rs`, NEW `crates/data-owned/tests/owned_package_contract.rs`, `crates/vlajmaji/Cargo.toml`, `crates/vlajmaji/src/lib.rs`, `crates/vlajmaji/README.md`. Root workspace Cargo.toml already has crates/* membership and lexicon workspace dependency; no new workspace dependency is needed because the umbrella's existing optional packs use direct path/version dependencies. No store/search/CLI/codec/PackId source edit belongs to S04. This list is DRAFT, not transfer authority.

Lock parity uses parsed package records, not just total package counts: match each old `(name,version,source)` row with all checksum/dependency/other fields unchanged; the sole existing-record exception is package `vlajmaji` gaining EXACT `vlajmaji-data-owned`. Exactly one new local package row has workspace0.3.0 and dependency ONLY vlajmaji-lexicon, no registry source/checksum. No preexisting record disappears and no external package/version/edge changes. Validate the actual bootstrap diff before continuing locked gates, rather than treating a successful Cargo exit as this proof. Outside-consumer private lock is separate; inspect actual `cargo tree -e normal`/resolved lock for absence of store/search/rusqlite/libsqlite3-sys/oxigraph and show direct deps exactly owned+lexicon. No transitive crate use via undeclared serde_json/sha2.

The finite feature sequence will use pinned guard/+1.98/locked/offline throughout AFTER the one explicit default-empty lock bootstrap. Default source Make/check and new package default/no-data proof run without materializing any ignored source payload. All embedded commands run in one new physically asserted disjoint archive: prepare ONLY owned pair first→test data-owned/embedded→test umbrella/data-owned→ONE direct outside consumer execution; then the four original Synthetic upstream prep routes→umbrella combined owned+four upstream+morphology→existing workspace allfeatures (including graph). Isolated search no-default remains its actual separate gate. Existing old four feature definitions/PackId/All dispatch are source-reviewed unchanged, while combined/allfeatures actually exercise them; no new per-feature Cartesian campaign is implied. Package FILE inventories target data-owned and umbrella; they are not publication/verification. Existing allfeatures CI's synthetic zero-owned-assertion example remains generic valid loader input, separately attributed from the richer consumer oracle.

Outside-consumer positive/negative INPUT recipe, not runtime: reuse the accepted S03 mixed own-a/b schema2 complete history, its original five whole base states/curation selections, literal24 rosters and context cutoffs0/0→1/2. Save expected INPUT bytes before package load; compare package.load canonical snapshot and every dictionary_at/curation_at against that authority. Before compilation, prepare literal/copy byte buffers for two malformed payloads WITH matching seals using existing offline Python tools: one version2 payload, one invalid unknown prefix section in an otherwise valid final Dictionary/history envelope. Retain actual exact changed bytes/length/SHA and seal recomputation from the pre-input pair. Consumer embeds/reads only those fixed buffers via std plus the two declared crates; it does not implement SHA/JSON, add dependencies, or derive its expected state from candidate export. The public pure loader tests must reach unsupported payload/history predicates beyond outer length/hash; malformed/unsupported seal-only counterparts need no changed payload. Exact variant/source pins wait actual accepted S03, no success presumed. One consumer invocation is the proof, no extra Tiny qualifier/rebuild per negative case.


Prospective bootstrap ordering clarification: safe standalone stdin rustfmt/copyback precedes the ONE offline default-empty check. Before that check, do not run Cargo fmt/metadata/Make fmt-check or any other command that may implicitly write the new lock; otherwise the declared bootstrap would not be the actual first lock writer. Capture the old accepted lock before authoring, actual command/exit/new lock diff and exact permitted record exceptions, then require --locked --offline for all subsequent gates. This changes only unissued command ordering, not S03 ownership or S04 execution authority.


### Prospective prompt checkpoint —2026-10-29 CDT

S03 INITIAL is implementing under its original fifteen-path/index/ONEtarget transfer, not merely intake; no accepted S03 source is asserted. New cc-prompt.md is explicitly UNISSUED and carries the prospective exact twelve-path package/lock/CI/consumer/gate recipe. Accepted S03 source/API/pins, literal consumer buffers and exact source-read manifest remain mandatory reconciliation before root readiness or any S04 dispatch. This planning-only draft neither changes current S03 ownership nor closes any S04/project row.
