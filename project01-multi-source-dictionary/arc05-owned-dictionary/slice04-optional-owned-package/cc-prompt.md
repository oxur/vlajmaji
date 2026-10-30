# Arc05 S04 — Optional owned history package

STATUS: PROSPECTIVE DRAFT, UNISSUED — accepted S03 source/pins, accepted error/API reconciliation and root readiness pending; prospective malformed INPUT buffers are preserved.

You are the retained CC, requested Luna/high. This is a future INITIAL assignment, not current source/target authority. Existing S03 INITIAL owns its fifteen source paths/index/ONE retained target. Do not implement, run Cargo, create data or claim intake acceptance from this draft. CDC will reconcile the accepted S03 source and exact expected buffers before root readiness/status-only issue. No backend/model attestation is available from a requested configuration alone.

## 1. Useful increment and binding boundary

A caller can prepare an explicitly selected owned snapshot pair, opt into `vlajmaji-data-owned/embedded`, and load its ENTIRE validated history before SQLite. The crate defaults empty, is unpublished, depends directly only on lexicon, and delegates to S03 strict loading. The umbrella exposes it only behind `data-owned`. Existing four upstream PackId/All inputs, normalized model/registry versions, store/query behavior and all other feature definitions remain unchanged. This is Arc05's final implementation increment, not Arc06 live managed integration or project completion.

Full current S04 slice-plan.md and ledger.md are binding, along with project cdc-directive04.md and current Arc05 arc-plan.md/ledger.md. Original eight S04 criteria/verifiers/owners/statuses remain unchanged OPEN. Five assignments maximum including initial; any failed hard gate stops for concrete CDC disposition, no automatic extra retry or sixth pass. Preserve actual failures/argv/env/source applicability in artifacts/execution.md. No separate command-export framework, acceptance prewriting, target reset, Full/default/ignored-source input probing, new cache/worktree, network/push/PR/hosted CI/publication or optimization.

## 2. Exact prospective source scope and ownership

After accepted intake/fresh pins and ONE explicit transfer ONLY, modify these TWELVE named paths (four NEW):

1. `.github/workflows/ci.yml`
2. `.gitignore`
3. `Cargo.lock`
4. `README.md`
5. `docs/owned-dictionary.md`
6. NEW `crates/data-owned/Cargo.toml`
7. NEW `crates/data-owned/README.md`
8. NEW `crates/data-owned/src/lib.rs`
9. NEW `crates/data-owned/tests/owned_package_contract.rs`
10. `crates/vlajmaji/Cargo.toml`
11. `crates/vlajmaji/src/lib.rs`
12. `crates/vlajmaji/README.md`

No root Cargo.toml, store/search/CLI/codec/upstream pack/model/registry/Bauske files. Workspace `members=["crates/*"]` and lexicon workspace dependency already suffice. No additional source fixtures, example files, dependency upgrades or broad formatting. Exact allowed output/report homes are this slice's artifacts/execution.md, relevant raw logs and eventual proposed closing report; do not precreate verifier/retrospective/success records.

Physical source checkout remains `.worktrees/project01-arc04/vlajmaji` on `codex/project01-owned-dictionary`, reused deliberately across arcs. Actual accepted S03 commit/tree/parent/clean status and unchanged guard are REQUIRED before issue/transfer, presently UNBOUND. Do not substitute accepted S02 854865a7 for future S03 applicability. Preserve old A04/S04/main refs. CDC/root remain source/index/target read-only during your exclusive author interval.

## 3. Concrete implementation spine

A. New Cargo manifest mirrors data-llg's workspace0.3/Rust1.98 layout: package name `vlajmaji-data-owned`, workspace version/edition/rust-version/license/repository, publish=false, readme, meaningful description, exclude=["data/**"], features default=[]/embedded=[], dependency ONLY vlajmaji-lexicon.workspace=true, workspace lints. No PackId/PACK_ID, store/search/sqlite/serde/hash direct dependency or build script.

B. New lib has complete public rustdoc and Result `# Errors` docs. Default code neither opens nor includes payload. `pub const fn is_embedded()->bool` returns cfg!(feature="embedded"). Under embedded, `snapshot_bytes()` and `seal_bytes()` return static slices via include_bytes!("../data/snapshot.json") and include_bytes!("../data/seal.json"). `load()->Result<OwnedSnapshot,OwnedSnapshotError>` calls accepted `load_owned_snapshot(snapshot_bytes(),seal_bytes())` directly. These names match current S03 WIP but MUST be reconciled to accepted S03 before issue. Do not parse a final Dictionary only, weaken whole-history/seal validation, cache results, invent runtime fallback or acquire external context. Pins/curation definitions survive without embedding foreign Dictionary bodies.

C. Umbrella adds one optional path/version0.3.0 dependency, feature `data-owned=["dep:vlajmaji-data-owned","vlajmaji-data-owned/embedded"]`, and cfg re-export `pub use vlajmaji_data_owned as data_owned`. Default[] and every existing feature/edge/export are retained. `.gitignore` adds exact `/crates/data-owned/data/`. No payload/database/generated files are committed.

D. Default new-package test asserts is_embedded false without any data directory; cfg embedded tests are GENERIC against the prepared input pair (load succeeds, strict roundtrip and bytes match) and do not assume owned-r4/a/b/counts. Existing Linux CI uses the valid S02 zero-owned-assertion curation example. The separate outside consumer below owns the rich five-state independent oracle. Add no compile-fail harness merely mirroring cfg.

E. Insert EXACT one Linux synthetic owned-preparation step immediately before existing all-features command, leaving triggers/jobs/backend/four original fixture routes and all-features invocation unchanged. Recipe in plan: exclusive mktemp under RUNNER_TEMP; existing public `owned_curation` example writes inputs/journal/view; mkdir crates/data-owned/data; existing CLI `owned export JOURNAL --output .../snapshot.json --seal .../seal.json`. Keep --locked --offline, create_new output behavior and explicit paths. Reproduce this body locally in the physically asserted disjoint archive; editing CI locally is authorized, hosted dispatch is not.

F. README/guide changes give runnable example→explicit export→data-owned/embedded load plus direct consumer/umbrella distinction. State owned notices/editor/source authority, no automatic foreign redistribution or copyright clearance, unpublished local package, missing context versus materialized view, no fifth All pack. Documentation uses actual accepted CLI positional/option grammar and fresh task-owned paths; execute its meaningful commands once.

## 4. One intentional lock bootstrap, then locked gates

Pin existing actual CARGO_HOME/RUSTUP_HOME/PATH BEFORE assigning fresh exclusive HOME/XDG_DATA_HOME/XDG_CONFIG_HOME/TMPDIR; preserve literal PATH unchanged. Use absolute Arc05 guard `/tmp/vlajmaji-p01-a05-cargo`, SHA426d1e49863551fe37252cecb8d36f8fb58410670fc550c6deedc89ddb92f53d, existing ONE retained A01 target and explicit +1.98.0. No downloads/new cache.

Save accepted old lock before authoring. After new default-empty package/umbrella edge are written, perform standalone safe stdin rustfmt and named copyback FIRST. The ONE explicit `guard +1.98.0 check -p vlajmaji-data-owned --offline` is the FIRST Cargo metadata/build operation and sole intended lock writer. Do not run Cargo fmt/metadata/Make fmt-check earlier; do not generate-lockfile or intentionally fail --locked first. Capture actual exit and compare parsed records before continuing.

For every old `(name,version,source)` package retain all fields/checksum/dependency/other records exactly, EXCEPT umbrella `vlajmaji` gains exactly `vlajmaji-data-owned`. Add exactly one local owned package0.3.0 depending only on lexicon with no registry source/checksum. No old record disappears or external version/edge changes. STOP on any discrepancy. After bootstrap every Cargo command is +1.98.0 --locked --offline; Make CARGO override/RUST_TOOLCHAIN=1.98.0. Consumer's task-owned new lock is separate: after explicit transfer and frozen INPUTs/manifest, exactly ONE `guard +1.98.0 generate-lockfile --offline --manifest-path CONSUMER/Cargo.toml` prepares that NEW private lock, never the product workspace. Record actual normal graph and overlapping versions versus workspace; stop on offline failure, no downloads/retry. Then the sole consumer build/run is locked/offline. This metadata preparation is not another product observation/qualifier.

## 5. One finite outside consumer and original fidelity controls

Create a new exclusive task-owned outside workspace source with `[workspace]` and DIRECT dependencies ONLY data-owned(path,embedded) and lexicon(path). It may use std; it CANNOT call transitive serde_json/sha2 or add them as deps. No copied crypto implementation. Use ONE actual build/invocation as proof, no separate qualifier/rebuild per malformed input. Freeze exact manifest/source/argv/binary and pre-input expected buffers before load.

Accepted S03 own-a/b schema2 snapshot+seal, all five whole base states/curation selections and literal24 rosters/cutoffs form the independent expected authority, NOT package.load output or zero-own demo. Accepted S03 artifact/source hashes remain PENDING; prospective malformed buffers and exact changes are in artifacts/consumer-inputs/oracles.json and the plan final handoff paragraph. Reconcile accepted public variants before issue; do not derive expected buffers from candidate output. Compare loaded wrapper canonical bytes, dictionary_at(r0…r4) whole bytes/hashes and curation_at(r0…r4) full definitions/selections/digests. Retained Link creation at r2, r3 tombstone and r4 reactivation stay historically distinct. Context bodies are absent; no materialization inferred.

Before compile prepare exact malformed payload/seal buffers with existing offline Python from pre-input authority: unsupported payload version2 and invalid unknown prefix section, each with matching outer SHA/length seal so typed/history errors are reached. Also tamper and unsupported/wrong-kind seal counterparts using copied buffers. Consumer uses ONLY public loader and fixed buffers, no own JSON/hash library. Preserve all changed INPUT bytes/hashes and positive restore. Final typed variant expectations must match accepted S03 public API before issue. This is the original finite strict-loader obligation, no new corpus or exhaustive matrix.

Inspect actual normal dependency closure (`cargo tree -e normal`, resolved private lock) contains intended lexicon/serde/hash closure and NO store/search/rusqlite/libsqlite3-sys/oxigraph. Show direct deps exactly owned+lexicon. Umbrella smoke is separate and may include store; do not infer before-SQLite from umbrella. Existing four PackId/All paths source-reviewed unchanged plus combined features exercised.

Continue actual existing owned export/check/import/old-history/context materialize→derived dictionary-build/query from accepted input, reusing S03 useful CLI increment and exact bodies/keys. Do not repeat Full corpus, all prior slice matrices or invent managed live editing. Preserve code/content notices separately.

## 6. Exact finite validation sequence and receipt limits

After one lock bootstrap/parity, current fmt-check→lint BEFORE behavior. New default package no-payload tests, meaningful affected/default Make check and isolated search no-default use source without preparing ignored embedded files. All feature preparation/embedded builds occur ONLY in ONE new disjoint Synthetic archive with sibling Bauske topology and existing target; assert physical cwd equals archive INSIDE EVERY command body before Cargo exec AND pass exact workdir. Compare all changed source/manifest/lock bodies source↔archive. No source ignored pack inputs read/stat/hash to determine whether present.

Archive order: exact future CI owned recipe→the ALREADY-declared generic data-owned/embedded and umbrella/data-owned selected-feature tests; inventory/hash the WHOLE actual newly generated data-owned/data directory, prove task-owned archive/provenance, then rename it to a NEW exclusive evidence destination OUTSIDE both crate and source archive, preserving every byte. No preliminary workspace allfeatures or retained/source/user data move. Make a NEW empty data directory; explicit frozen-rich import into a fresh journal→owned export create_new pair there→SOLE direct rich consumer build/run/graph inspection. Then established FOUR Synthetic fixture recipes jbotci/llg/la-lojban/sutysisku→umbrella combined owned+four+morphology→final ONE workspace allfeatures including existing graph. Preserve both distinct pair identities; reject stale embedded consumer output and keep moved pair outside package FILE inventory. Same ONE archive/target/matrix, no destructive replacement/second qualifier. No new Cartesian per-old-feature campaign. Existing graph backend/toolchain support is retained, no install/network allowed. Stop unexpected failure for concrete disposition, preserve raw/source/env/terminal; no hidden replay or unrelated baseline fixture repair.

Actual affected package FILE inventories are `cargo package -p vlajmaji-data-owned --list --allow-dirty --locked --offline` and umbrella equivalent with explicit+1.98.0/guard; test -- --list is not package evidence. Assert expected source/readme/manifest entries and excluded data/**/DB/scratch; not package --verify/publish/legal clearance. No feature rerun solely bookkeeping once source applicability is unchanged.

Before freeze settle ORIGINAL finite row census (actual positive/negative reach, unresolved platform/IO limitations), current exact twelve-path diff/old lock parity, complete commands/exits/raw locations and source applicability. Commit ONLY named12 paths with adjacent Codex/Billo trailers; return clean commit/tree/parent, exact hashes and explicit source/index/ONEtarget QUIET to CDC. Green author gates are proposed-done, not independent acceptance. Closing report may be written only after actual execution and must preserve failures/limits.

## 7. Required complete intake before any transfer

Read whole current S04 plan/ledger/this prompt and directive04; relevant binding current Arc05/project plan/design/ledger plus accepted S03 verifier/closure and declared source/API. Historical appendices and old Arc04 megabyte evidence are reference-only, not mandatory reload. Read complete new-package analog data-llg manifest/lib/test/README; umbrella manifest/lib/README; root workspace dependency/lint/feature policy/Make; actual workflow all feature prep; ignored path rules; accepted S03 pure owned snapshot/seal wrapper/codec/hash/history validation/error APIs and owned CLI export/example recipe; relevant existing data tests/owned guide. Exact source ranges/hashes and expected fixture query manifest are PENDING accepted S03 and must be frozen before issue.

Read actual named Rust guide AP18/AP19/AP33/AP67/API02/API03/API04/EH04/EH13/EH14 sections and applicable project-management prompt/verification/engineering guidance already used; report actual extents, not titles alone. Return source-cited coverage + readback in artifacts/execution.md: distinct snapshot vs PackId, whole old history/curation, default cfg boundaries, actual standalone graph, lock bootstrap first-writer and record parity, private-buffer negative construction, physical-cwd/owned env, file/stdout/error/authority boundaries, finite gates and ownership. Ask only consequential unresolved choices; routine within-contract implementation is yours. CDC acceptance/fresh source/branch/lock/guard/protected refs precede ONE explicit twelve-path/index/target transfer. No new intake ceremony once coverage is accepted.

After accepted S04, fresh Sol6.1/high reviewer must ACTUALLY execute original Arc05 composition and SAME context final canonical review/root consistency before mechanical close. All14 project rows remain OPEN; Arc06 unactivated. This unissued draft claims no ready source, gate result or acceptance.
