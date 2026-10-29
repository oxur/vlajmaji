# S04 — Optional owned snapshot package and offline consumer

| Field | Value |
| --- | --- |
| Status | PROSPECTIVE DRAFT ONLY, UNISSUED. S02 INITIAL/pass1 active; S03 also DRAFT, neither accepted dependency is invented |
| Authority | directive04 THROUGH ARC05 ONLY; retained Sol6.1/high CDC and Luna/high CC; five S04 passes including initial, no automatic sixth |
| Known source | Accepted S01 1b899ebb/tree745f; final accepted S02/S03 source and exact snapshot APIs must replace this placeholder before issue |
| Useful result | Caller-selected owned history can be embedded and strictly loaded by an offline consumer whose direct dependency graph excludes store/search/SQLite; the existing CLI prepares the exact local pair |
| Version / date | 0.2 / 2026-10-29; prospective existing-CI feature compatibility reconciled, accepted S03/exact allowlist/readset still pending |

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

Scoped stdin rustfmt→named copyback→fmt-check→lint, focused new/affected proof and one outside consumer, actual CLI example, required Make/no-default, disjoint Synthetic feature checks and affected package FILE lists. Pin existing CARGO_HOME/RUSTUP_HOME/PATH before owned HOME/XDG/TMPDIR and use unchanged Arc05 guard/ONE retained target/+1.98.0/offline. New fixtures use exclusive creation; no target reset/download/new cache. Preserve actual failed attempts and applicability. No benchmark/Full/default/network/push/PR/publication or automatic sixth pass.

CDC independently reviews actual candidate and reproduces the affected public+CLI increment before child close. Arc05 then needs a fresh independent Sol/high context to execute the composed init/edit/conflict/curation/export/import/optional-load/explicit-query workflow, followed by SAME-context canonical review, root consistency and mechanical closure/retrospectives/project bubble-up. All fourteen project rows remain OPEN; Arc06 unactivated. This draft contains no success record or advance acceptance.

### Version0.2 — existing all-features preparation,2026-10-29

Root source review identified the clean CI embedded-input prerequisite before issue. The exact workflow path and one offline synthetic owned-preparation step are now prospective S04 scope; no blanket workflow exclusion applies to this compatibility step. No source/workflow edit, Cargo/hosted run or intake occurred. All eight original binding rows remain OPEN; accepted S03/signatures/allowlist/literal consumer and lock procedure remain pending.
