# S01 iteration01 — nonUnix native-path inference correction

**STATUS: DRAFT / NOT ISSUED. PASS2/5,2026-10-31.** ROOT must read this concrete draft/readset/probe recipe before status-only issue. No source/index/target authority is granted now. All8 S01/all8arc/all14project rows OPEN/HOLD. Retained CC Luna/high is parked; CDC Sol6.1/high owns planning. This supersedes the INITIAL source-write assignment ONLY after later issue/bounded actual readback/fresh pins/explicit ONE-path transfer. No extra model/layout/Operator choice is needed.

## Defect, exact boundary and preserved evidence

Frozen INITIAL `cf0998f0d76321a2c8601fa3bc29bb44c69b7c8c`, tree `18b66ea71556382fe32d91c245b03a98f30676d1`, parent cb13d377 is clean on `codex/project01-integrated-workbench` in `/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc04/vlajmaji`. Original prompt25355B/SHA c15d5fba8195429e0f4889a3b236185c1433a952ddb0e583b7355d77e353c257 remains immutable. Keep directive05/initial behavior/quality/resource/authority rules; this iteration changes only the source allowance and affected validation explicitly below, not original criteria or five-pass budget. This is the SECOND of five total assignments, not another unfrozen pass1.

`crates/store/src/workbench.rs::decode_native_path`1550–1602 has `#[cfg(not(unix))] let path = return Err(...)` followed by `path.is_absolute()/to_str()/Ok(path)`. Native Linux/macOS compilation never typechecks the active nonUnix binding. ROOT noticed it after the native24-child independent proof; CDC had inspected the helper but missed this dormant-branch inference hazard. ONE approved std-only host rustc+1.98.0 `--edition=2024 --crate-type lib --emit metadata` probe reproduced E0282 at the untyped let plus unreachable warning, actual exit1. It proves the exact language shape, NOT a Windows crate build/runtime. No source correction or second probe was run. See artifacts/native-inference01/probe.rs806B/SHAaf3f6456a457dc0bb28d55fa16a9171243028ad1194c1057240937fb8f7a9514, stderr1328B/SHA3d1f8b0d3f7692e0de9c706b0e2887b12036efd4c14f91140ede57e34d024c03 and actual argv/env/receipt. Source/index/ONE target returned ROOT QUIET immediately afterward.

The previous uncommitted close proposal was WITHDRAWN BEFORE any acceptance commit. Twelve whole draft bodies are preserved unchanged under artifacts/close-proposal-withdrawn01; current HOLD prefixes supersede their proposed DONE/accepted wording. Original ledger cells are restored OPEN. Original authored native gates and CDC24 cf0998PASS are preserved, not reclassified failed Unix behavior or falsely applied to new source. CC's initial6883B proposed body is preserved separately. This iteration must not silently rewrite any of those records, issued prompt or preserved failures.

ROOT concrete disposition: repair complete platform cfg return blocks in ONE existing production path; keep common hex/NUL validation and ALL Unix operations/order/errors/return unchanged. Current affected language/native/store/build proof below, retain original source-qualified broad receipts, then one commit/QUIET and CDC repeats SAME24 method on fresh root. No new feature/archive/Full/consumer/dependency/cache/network campaign.

## Complete bounded reading and actual readback

Use existing artifacts/execution.md for this iteration's real loaded text/required-data results/source-cited readback, preserving earlier chronology. Required-full means whole actual content, not hash/headings/another context's read; required-section includes complete helper/type closure. Recover truncation before dependent edits. Retain actual applicable prior full-intake text only while available; recovery reloads genuinely missing consequential sections. No duplicate whole historical megabyte intake or new receipt framework.

| Order / material | Required scope and purpose |
| --- | --- |
| This iteration prompt, all8 current S01 ledger rows and slice-plan.md current HOLD + PASS2 portability-correction section | Required-full prompt/ledger; required-section current correction/validation rules. Exact source/pass/ownership/held acceptance. Original INITIAL prompt constraints remain governing; read its implementation spine4–7, validation/resource/return and initial intake record clarification as required sections if no longer available. This is not permission to resume original14-path writing. |
| project cdc-directive05.md; source AGENTS.md relevant workspace/style/commit/validation rules | Required-full directive; required-section actual source conventions. Arc-only/five passes/ONE target/no external/default/Full and explicit coauthors unchanged. |
| artifacts/native-inference01/probe.rs,stderr,argv.json,env.json,receipt.json,stdout | Required-full actual negative language probe/pins/actual exit; distinguish E0282 from a Windows run and do not infer compiler success from parser/source review. |
| source store/workbench.rs original decode_native_path1550–1602, NativePathReportWire1485–1505, WorkbenchError674–728 and necessary definitions; source store/lib.rs NativePathReport export; workspace Cargo.toml edition2024 and store inheritededition | Required-section exact old function/common/error/type/edition closure. Workbench84511B/SHA9bd9a7f05153367bef0845d841d0a3a9a0f3718adaa424730a387e31eaa4e423 is frozen original, not WIP. |
| artifacts/native-inference01/corrected-decode-native-path.rs | Required-full1739B/SHA5179fcc3799c9df6d3fbefefba1024281fa0be1e4ca7af06011240f9d69d8f3f. Proposed function ONLY, not applied/compiled at authoring. Compare every unchanged common/Unix statement before editing. |
| Existing store/tests/workbench_contract.rs11 tests, workbench.rs private binding/build tests and their fixture/helper closure; source Makefile format/fmtcheck/lint; CLI/search package defaults/example manifest | Required-section current actual test/helper/command definitions, not invented positive or new tests. Unit helper named test_binding_with_omitted_context_pin is NOT itself a #[test]; record actual selected test count. |
| Original execution current A–G + CDC qualifications + actual native/broad/freeze/independent24 records; artifacts/independent-proof/method.py + bindings.json | Required-section execution; required-full existing method/bindings. Unchanged24 method scope and original source/binary limitations; no new child/oracle. Do not reload unrelated old lint logs merely for history. |
| Existing engineering-methods prompt-authoring intake/correction sections; Rust error/ownership guidance actually applicable from INITIAL | Required-section retained actual text. Named Result/UnsupportedNativeEncoding/no panic/source changes, common Unix behavior preservation. No new domain or generic refactor. |
| Current HEAD/tree/branch/clean source; full14 original baseline-body pins, unchanged lock/guard and affected original function | Required-data complete named finite projection before transfer; actual commands/results in execution. All other13 source paths remain immutable during correction. No retained Full/default/DB/cache leaf probe. |

Readback before edit: explain why initial host checks missed active nonUnix inference; identify exact common checks and unchanged Unix order; describe complete nonUnix typed Err expression; scope ONE source path/no tests/API change; one language-only positive vs no Windows claim; source-qualified retained old gates vs current affected checks/default builds; exact commit/QUIET then SAME24 independent proof. CDC accepts this bounded actual readback/fresh pins before explicit transfer; no another ROOT permission ceremony after reviewed issue absent mismatch.

## Worked repair — ONE source path only

ONLY `crates/store/src/workbench.rs`, ONLY decode_native_path. No tests, public fields/serde/wire/errors/native path semantics, other function/source/lock/schema/dependency/docs edit. No public fault helper or framework. Existing common ASCII/even/lowercase hex and NUL checks stay before the cfg branches at the same order. Use the proposed complete function artifact:

```
common hex/decode/NUL statements unchanged
#[cfg(unix)]
{
    let path = { existing Unix OsStringExt/encoding check/from_vec statements };
    existing absolute/utf8 consistency check and InvalidBinding return
    Ok(path)
}
#[cfg(not(unix))]
{
    Err(WorkbenchError::UnsupportedNativeEncoding { encoding: report.encoding.clone() })
}
```

No diverging untyped let, no later receiver outside Unix block, no annotation merely hiding unreachable code, no lint suppression. This is a source-grounded implementation sketch; the artifact is the precise proposed function, uncompiled before this assignment. Private whitespace from prescribed rustfmt may change, but ANY semantic/statement/order/extra-function change stops for CDC disposition before further validation. Expected whole workbench.rs after precise replacement (one separating newline before next fn) is84588B/SHAcef8e1a5103ecf13bf2631bf57301f40cef40583c0ded9ac8481df2095a9f180 before any incidental formatter difference; do not silently waive a hash difference—inspect actual hunk and identify formatting versus contract change.

## ONE positive language counterpart and current affected sequence

After explicit transfer/edit, assert all prior commands terminal and exact ONE-path worktree delta; no parallel target owner. Reuse literal PATH from original-env.sh2287B/SHAae749d0dd537499b254fd4e9fc35980345eabf5e1164c41817e62eacc58b2c86 (PATH1917B/SHAf7af29286b33342886cb96026c90a8bfcbbb7e3cfaef920ed73086ab33d3a53d), pinned CARGO_HOME/RUSTUP_HOME before NEW exclusive pass2 HOME/XDG_DATA_HOME/XDG_CONFIG_HOME/TMPDIR under /private/tmp. Source once, override only four owned roots; no manual PATH reconstruction/dedup. Retain selected actual env/argv/cwd/raw/exit. Guard `/tmp/vlajmaji-p01-a05-cargo`330B/SHA426d1e49863551fe37252cecb8d36f8fb58410670fc550c6deedc89ddb92f53d; only same retained Arc01 target. No a06 guard or target Make argv.

Exactly ONE corrected std-only positive counterpart: create a NEW exclusive task scratch input under /private/tmp and NEW owned child under the retained target. Extract the COMPLETE corrected actual decode_native_path function text up to next `fn seal_payload_kind` (not an independently rewritten approximation), require its body match the proposed corrected function apart from trailing separator newline, and prepend only these standard-library dummy definitions:

```rust
use std::path::PathBuf;
struct NativePathReport { encoding:String, hex:String, utf8:Option<String> }
#[derive(Debug)]
enum WorkbenchError {
    UnsupportedNativeEncoding { encoding:String },
    InvalidBinding { detail:String },
}
```

To activate the SAME fallback on the host, substitute ONLY its exact `#[cfg(unix)]` with `#[cfg(any())]` (false) and `#[cfg(not(unix))]` with `#[cfg(not(any()))]` (true), once each. Preserve original extracted function + complete probe bytes/hash before invoking. Do NOT alter common validation or branch body. Invoke ONCE `/Users/oubiwann/.cargo/bin/rustc +1.98.0 --crate-name a06_native_inference_positive --crate-type lib --edition=2024 --emit metadata --out-dir EXACT_NEW_RETAINED_TARGET_CHILD EXACT_PROBE.rs`, from asserted task-owned cwd, RUSTUP_AUTO_INSTALL=0/CARGO_NET_OFFLINE=true, selected env/toolhomes. Expected actual exit0; unused dummy-definition warnings are recorded and not Windows/library-lint success. No Cargo/link/runtime/target installation/network or second probe/retry. Preserve full argv/env/stdout/stderr/actual terminal. Failure stops. This checks active fallback host language inference only, never certifies a Windows crate build or runtime.

Then current source physical cwd and existing prescribed guard/toolchain:

```
make format CARGO=/tmp/vlajmaji-p01-a05-cargo RUST_TOOLCHAIN=1.98.0
make fmt-check CARGO=/tmp/vlajmaji-p01-a05-cargo RUST_TOOLCHAIN=1.98.0
make lint CARGO=/tmp/vlajmaji-p01-a05-cargo RUST_TOOLCHAIN=1.98.0
/tmp/vlajmaji-p01-a05-cargo +1.98.0 test -p vlajmaji-store --test workbench_contract --locked --offline
/tmp/vlajmaji-p01-a05-cargo +1.98.0 test -p vlajmaji-store --lib workbench::tests --locked --offline
/tmp/vlajmaji-p01-a05-cargo +1.98.0 build -p vlajmaji-cli --bin vlajmaji --locked --offline
/tmp/vlajmaji-p01-a05-cargo +1.98.0 build -p vlajmaji-search --example integrated_workbench --locked --offline
```

Store integration expected existing11; private filter actual binding/build3 (zero selection is not pass). Set RUST_TEST_NOCAPTURE=1 for those actual test observations, preserve exact count. Explicit final two builds are DEFAULT-feature source-cwd builds, not cargo-run/CLI/example product invocations; record normal output paths/fullSHA and last writer flags. No author example/CLI journey rerun: CDC owns SAME bounded24 after frozen handback. No new archive, no Makecheck/no-default/allfeatures/package replay in this correction.

ROOT explicitly retains earlier source-qualified cf0998 Make422+tool19/no-default89/sole five-Synthetic archive425/package FILE43/27/24. Preservation rationale must compare actual diff: common validation unchanged, Unix decoder statements/errors/return unchanged, no Unix functional semantics or other13 source/Cargo/path-list/dependency changes. These remain actual predecessor receipts, NEVER relabelled as newly run current-source gates; final assembled arc owns full current validation. This is a reasoned affected revalidation of a cfg-only defect, not a quiet waiver, unknown repaired test or Windows support claim. If extra semantic edit would invalidate that rationale, STOP before extending the matrix.

## Freeze, return and independent disposition

Preserve all original native/24/probe failures and current outcomes chronologically in artifacts/execution.md. Append dated PASS2 proposed return to canonical closing-report.md without removing its current HOLD/prior as-of bodies; CDC updates current disposition only after actual review. No other planning/status/prompt edits. All8 rows stayOPEN during your return. Recheck EXACT one changed source path, others/lock/guard/protected refs unchanged, own source diff/context, current source applicability and all actual terminal receipts (unknown remains unknown). Commit ONLY `crates/store/src/workbench.rs` with `git commit --only`, adjacent trailers:

```
Co-authored-by: Codex <noreply@openai.com>
Co-authored-by: Billo AI <ai-engineering@billo.systems>
```

Return exact corrective commit/tree/parent cf0998, clean source, ONE named diff, original/new function statement comparison, positive-language probe literal/raw receipts and native/format/lint/tests/default CLI+example build pins. Explicit source/index/ONEtarget QUIET before CDC takes runtime. No push/PR/merge/network/source acquisition/default/Full/cleanup. No automatic third assignment or semantic extension after a hard failure.

CDC then repeats EXACT same15613B/SHA75472e8111741ee04fb430c4eed9a836d3a46acd7b866cdaa07df5eb144d38e6 method,24 children/unchanged original INPUT oracle on NEW owned root/current HEAD+tree/default binaries. New bindings list cumulative14 source files against original cb13, not just the one-file corrective commit. Archive cf0998 bodies stay pinned at their historical values; corrected source workbench.rs intentionally differs, no claim whole14 archive parity at new source. Preserve both actual24 attempts separately. No method expansion or extra campaign. Current local close/verifier/retro/parent bubble-up follows genuine final disposition/root consistency; later S02 readiness and final assembled/SAME canonical/Operator gates retain their original owners.

Draft readiness: actual negative and proposed cfg function source-grounded, exact one-path/resource/error/Unix preservation and executable finite positive/affected gates supplied; ROOT final whole draft/method review and later status-only issue remain pending. No source/target permission inferred from this DRAFT.
