# CC assignment — A02 S01 iteration03: precise unsupported numeric metadata

Same CC `/root/vlajmaji_p01_a02_cdc/vlajmaji_a02_s01_cc`, requested OpenAI
gpt-6-luna/high, Autonomous/Expedited. **Second executed refinement of five**.
Preserve initial/iteration01/iteration02 prompts and every prior attempt. S01
remains OPEN, all seven rows OPEN; frozen bbd6459510f7105fc6ad6b60eb8da3352ad9f874
is proposed-done and REPAIR REQUIRED, not accepted. No S02 implementation.

## Baseline, bounded ownership and resource handover

Source `/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc02/vlajmaji`,
feature/project01-full-jbotci-import, clean bbd6459510f7105fc6ad6b60eb8da3352ad9f874.
CC exclusively owns mutation/index and serialized retained-target execution from
explicit handover until clean frozen return. CDC/root are out of source/target
execution during that interval. Protected main d1e1c928, Bauske c5801ae, old A01,
research and issued records stay read-only. Local scoped source commit authorized;
no push/PR/merge/publication/data bundle/new dependency/feature/toolchain change.

Only source repair paths: `crates/lexicon/src/jbotci.rs`,
`crates/lexicon/tests/jbotci_contract.rs`, `crates/cli/tests/jbotci_contract.rs`,
`docs/jbotci-import.md` only if needed to clarify the bounded value/error grammar.
Do not change native-ID recognition, CLI implementation, example, model/store/
search, manifests or Cargo.lock. Sole planning write remains S/artifacts/cc-execution.md,
append actual intake/readback/attempts/gates/proposals, preserve old errors.
CDC alone owns canonical planning and planning commits.

Duncan reported broad cache cleanup during the preceding review. Before any CDC
build, actual assessment found retained target14GiB and debug CLI/example/callers
present, target/release absent;190GiB free, no matching task Cargo/rustc/Make/import
process in scoped snapshot. Source and planning intact. Guard unchanged SHA
`a19c3f1f83e40c97d21109f6b280969bc879c47ab587e529d1e28512b2a61430`.
Subsequent outside caller rebuild1.29s and CLI rebuild0.17s succeeded offline;
needed debug artifacts are ready. This does not prove every historical cache
file survived or invalidate old receipts. Do not delete caches or kill processes.
No speculative release build in this repair.

Every Cargo/Make explicitly sets
`CARGO_TARGET_DIR=/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc01/target`
and uses `/tmp/vlajmaji-p01-a02-cargo`, +1.98.0/locked/offline; Make CARGO=that
launcher; login:false/scoped repo-write escalation. Serialize builds/imports/
scans/edits, one target writer. Recheck resource paths before use. No transient
sha2/dev dependency: existing literal fixtures can use externally computed exact
hashes; retain failed fixture/command attempts instead of repeating that dependency
experiment. Outside package AND binary unique per independent caller, exact source/
lock hashes and rebuilt-current-binary attribution. All CDC command sessions exited.

## Required intake, retained contracts and actual review

P=`/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/planning/project01-multi-source-dictionary`;
A=P/arc02-reproducible-jbotci-import; S=A/slice01-lossless-export-normalization;
K=`/Users/oubiwann/Library/Application Support/ChatGPT-Profiles/personal/codex-home/skills`.

Required-full/current: this prompt, S/slice-plan.md, S/ledger.md,
S/artifacts/cdc-review-attempt02.md. Required-section: bbd6459 jbotci.rs
parse_metadata complete (roughly245–340), existing Number recognition and optional
shape diagnostic helpers; CLI test occupied regular-file/symlink/directory block;
API metadata classification test and its fixture helpers. Required-data: complete
33-case review02 manifest and API attempt02 log (13 positive fixtures/20 errors,
one disagreement), CLI36 inventory, and resource assessment JSON. Required-full
K/engineering-methods/guides/07-implementation-prompt-authoring.md before dependent
edits, with same-context actual retained prior reading permitted; read corrective
iterations and work-verification05 correction-review section before deciding reuse.
Retain actual initial/iteration01/02/directive02/Rust/error/testing intake where
still available, reload missing normative text in bounded complete chunks. No new
full-corpus intake or guide catalogue; original universal obligations remain.

Append concise source-cited readback of exact parser order, valid-versus-malformed
counterparts, integer-overflow preservation, no native-ID feature change, file io
assertion and mandatory current gates. Distinguish source reads/data queries, recover
truncations, do not invent receipts. Then implement without another approval gate.

## Binding correction A — valid JSON numeric value outside metadata subset

At bbd6459 parse_metadata around297–306 classifies every digit-leading value with
any nondigit as InvalidMetadata. Independent public outside caller executed all
33 cases; **sole disagreement** is appended valid assignment `bad = 1.5` returning
InvalidMetadata instead of UnsupportedMetadata. All repaired optional shapes,
exact Presence, fixed notices, revision controls and malformed literals passed.

Resolved contract: the supplier interface remains deliberately bounded to JSON-
compatible strings, booleans and unsigned decimal u64 integers. A complete valid
JSON numeric literal outside that unsigned-integer subset (fraction, exponent or
negative number) is an unsupported value grammar, not malformed trailing junk.
This classification does not make the parser general TOML or accept those values.
Valid JSON arrays/null/other existing outside-subset forms remain rejected as
unsupported; no new value home. Preserve explicit u64 overflow InvalidMetadata
for an all-digit integer token, empty key/RHS, duplicate/missing/required supported
wrong-type errors, and malformed supported literals/trailing junk as InvalidMetadata.
Single-quoted value, quoted key and valid unsupported language/format stay Unsupported.
Do not claim classification of arbitrary general-TOML syntax not implemented.

Recommended minimal recipe inside parse_metadata, using existing serde_json with
workspace arbitrary_precision (already enabled, no manifest change): retain JSON
string/boolean branches; handle all-ASCII-digit token via existing parse::<u64>()
first (so overflow remains InvalidMetadata); before the broad malformed numeric-
prefix branch, recognize a complete `serde_json::Number` parse and return
UnsupportedMetadata for the outside-subset numeric literal. Only after that
recognition reject malformed/trailing numeric/boolean text. Local helper/error
wording is discretion; concrete categories and precedence are binding.

Falsifying public API controls with otherwise complete valid metadata and exact
renewed metadata pin: `bad = 1.5`, `bad = 1e3`, `bad = 1e9999`, `bad = -1.5` ->
UnsupportedMetadata; `bad = 12` -> successful import with unknown-simple-key
retention/diagnostic. `bad = 12 junk`, `bad = 1.5 junk`, `bad = true junk`,
`bad = false junk`, `bad = "ok" junk`, empty key/RHS and
`bad = 18446744073709551616` -> InvalidMetadata. Keep valid single-quote/quoted-key/
unsupported-language/format counterparts. Current bbd6459 must fail the float/
exponent positives, so roundtrip or generic is_err is insufficient. CLI still
maps both metadata variants to jbotci_metadata/no output; public typed category
is the discriminating gate. No new dependency or broad parser investigation.

Root's separate native-number lead was **refuted**, not a defect: the same actual
33-case caller proves definition_id1e9999 and1e3 retain exact native/raw lexemes,
remain lexical and have no invalid_definition_id diagnostic under existing
arbitrary_precision. Preserve that positive/control and Number-based native-ID
recognition. Do not revert to f64 or change features to fix a nonexistent failure.

## Binding correction B — complete occupied-file regression assertion

The updated CLI test now reaches create_new with valid pins, and independent
actual CLI36 controls pass file/symlink/directory io branches. But the durable
regular-file overwrite block only checks failure/stdout/unchanged bytes, whereas
iteration02 R6 explicitly requires exact structured `error.code == "io"` for it.
Add serde_json decoding of overwrite.stderr and that exact assertion beside the
existing valid-pin call, matching the already-correct symlink/directory checks.
This is one missing automatic oracle, not a second product io defect. Keep the
fresh valid counterpart and independent hash-error negative. No CLI product change.

## Checks, preservation and frozen return

Run all cheap focused public API/process/outside controls first, read the entire
changed fixture and every dependent assertion before rerun. Preserve the two old
stale duplicate-fixture expectations, harness compile/filename failures, mistaken
Cargo-make invocation and transient dependency attempts. Do not rewrite them.

Then mandatory Make format→lint→check once after final source/test edits;
workspace allfeatures, isolated search no-default AND allfeatures, affected umbrella
no-default/allfeatures as original. Existing research-root ignore remains separate.
Complete exact current frozen-source gates; repeat only for changed source/tests,
failures or identified concerns, recording why. No new proof framework/profiler.

One current full pinned public import/reload and whole-release CLI check must
establish unchanged known valid-input behavior after parser precedence repair.
Do not regenerate redundant huge copies just to repeat unchanged evidence: the
previous full canonical215412497B/SHA0987ec84a8434573b5979ea854ca459ce2ea9c7fa03ebac6639574dd2b7aa460
is a prior CC witness, and new valid pinned output should match exactly because
only rejected metadata categories/test assertions change. Use a fresh output for
actual import, retain old outputs/receipts. Public example is unchanged; its prior
execution may be proposed for retention, with CDC deciding. CDC still performs
independent current full-field/example/gate acceptance after this correction.
No S02 work, store/full benchmark or publication in this interval.

Return clean scoped source commit (Codex/Billo AI trailers), exact diff/hashes/
executed populations and all seven row proposals; all rows remain OPEN pending
CDC independent disposition. Explicitly freeze source/index/target and cc-execution.
Report material focused/full/gate transitions and any unexpectedly long task-owned
process/finite scope. No self-acceptance or retrospective claimed by CC.

Author readiness: CDC received full bbd6459 repair diff, complete new intake after
bounded truncation recovery, exact source parser/CLI test blocks, and independently
reproduced33 API cases plus36 CLI invocations. One category defect and one missing
exact assertion are isolated; valid native-number lead is explicitly refuted.
07 authoring/corrective guide was loaded before this prompt. Rust concrete errors/
no coercion and testing nonvacuous real-branch oracles directly determine the recipe.
No unresolved consequential decision or new investigation is delegated to CC.
