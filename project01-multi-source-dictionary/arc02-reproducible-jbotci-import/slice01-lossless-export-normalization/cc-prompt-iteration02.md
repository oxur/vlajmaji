# CC assignment — A02 S01 iteration02: bounded normalization and delivery repairs

Code Contributor `/root/vlajmaji_p01_a02_cdc/vlajmaji_a02_s01_cc`, requested
OpenAI gpt-6-luna/high, same context and Autonomous/Expedited authority. This is
**first executed refinement of five**. Iteration01 was a pre-edit clarification;
preserve it and the initial prompt. S01 remains OPEN, all seven rows OPEN.
Candidate 23d057bf860c8e5137e4d4269fc9749ac8d449e6 is frozen, proposed-done,
**not accepted**. Repair findings below, then complete all original obligations.
This assignment supersedes iteration01 prospectively, not its historical evidence.

## Source, ownership and preserved boundaries

Source `/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc02/vlajmaji`,
feature/project01-full-jbotci-import, clean 23d057bf at handover. CC exclusively
owns source mutation/index and serialized target use from this handoff to frozen
return. CDC owns planning/design/review and will not execute source/target during
that interval. Main d1e1c928, old A01 checkout, Bauske c5801ae, research and all
issued prompts/accepted history remain read-only. No push/PR/main integration,
publication, full raw bundle, dependency/toolchain/framework change or S02 edits.

Exact repair source allowlist: `crates/lexicon/src/jbotci.rs`,
`crates/lexicon/tests/jbotci_contract.rs`, `crates/lexicon/examples/import_jbotci.rs`,
`crates/cli/tests/jbotci_contract.rs` (create by renaming the candidate's
`crates/cli/tests/jbotci_cli.rs`, explicitly authorized deletion of that old name),
`docs/jbotci-import.md`, `crates/lexicon/README.md`, `crates/cli/README.md`,
`README.md`, `CHANGELOG.md`. No other source edits expected; request a specific
consequential widening before affected edits. Preserve existing CLI implementation
unless a reproduced correction requires it. Sole planning write remains
`slice01-lossless-export-normalization/artifacts/cc-execution.md`: append repair
intake, original errors, exact attempts and row proposals; do not rewrite history.
Initial receipt's claim that all 13 paths matched the exact allowlist was wrong:
`jbotci_cli.rs` was outside the issued basename. Append that correction honestly.
CDC alone edits canonical plans/ledgers and commits planning. Local source commit
with the existing Codex/Billo AI trailers and named-path staging is authorized.

Every Cargo/Make invocation explicitly sets
`CARGO_TARGET_DIR=/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc01/target`
and uses `/tmp/vlajmaji-p01-a02-cargo`, +1.98.0/locked/offline; Make CARGO=that
launcher. Use login:false, scoped escalation for repository writes/builds. One
writer; serialize source edits/builds/imports/scans. No cache deletion/process kill.
Recheck free space before large outputs; last actual free space ~93GiB does not
relax ownership. Unique outside Cargo package AND binary names, exact source
hashes/offline lock. Preserve prior full outputs and failed attempts; use new
attempt paths. Never claim a default-target or stale binary run proves this commit.

## Required reading and coherent repair intake

Planning project root P is `/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/planning/project01-multi-source-dictionary`;
A=P/arc02-reproducible-jbotci-import; S=A/slice01-lossless-export-normalization.
Skill root K is `/Users/oubiwann/Library/Application Support/ChatGPT-Profiles/personal/codex-home/skills`.

1. Required-full/current issued planning: this prompt, S/slice-plan.md, S/ledger.md,
   S/artifacts/cdc-review-attempt01.md. Required-full retained contract: initial
   cc-prompt.md and iteration01, P/cdc-directive02.md. Previously fully received
   unchanged material need not be ritualistically reread if actually retained in
   this same context; record extents/state honestly. If lost/truncated, recover.
2. Required-section: source23d057b `jbotci.rs` parse_metadata; registry native ID
   construction; build_import row construction/early continue/unknown extension
   loop; lineage/capture/release/code notice constructors. Full example, both
   contract test files and full docs/jbotci-import.md. These are the changed
   contracts; inspect their actual complete relevant bodies, not grep hits.
3. Required-full K/engineering-methods/guides/07-implementation-prompt-authoring.md
   before dependent implementation; required-section testing03 Proportionate
   Proof and work-verification05 Evidence During Correction Review; Rust API
   API18/23/26–29, errors EH01–04/07–14, anti-patterns AP09/25 and CLI error/output
   guidance previously loaded in initial intake. Reuse actual retained receipt,
   reload missing normative bodies; their applied decisions are below.
4. Required-section witness: `/tmp/p02a02-jbotci/THIRD-PARTY-NOTICES.md`238–255 at
   3d9fd41d9ed32de26311ee27ec86b6b165985aa1; actual source `LICENSE`1–28. Required-data:
   inspect complete review01 API manifest/results, duplicate-unknown two-row input/
   results and CLI result inventory. No full corpus re-intake needed: input pins
   unchanged; fresh full normalization/byte checks still required after repair.
5. Reference-only: earlier exploratory candidate receipts and future S02 readiness.
   Original prompt's universal ownership/legacy/gates/full-population obligations
   still govern; no new investigation or evidence-only phase is commissioned.

Before dependent edits append source-cited readback connecting the seven repairs
below to their row, cause and falsifying positive/negative controls. Distinguish
text received from structured-data coverage; recover clipped output in bounded
ranges with sufficient aggregate and command output budgets. Do not invent
receipts. Once coherent, proceed without another approval request.

## Binding repairs, source-grounded route and concrete oracles

R1 — optional unsupported-shape diagnostics (rows02/03).
At23d057b presence_string around510 maps any non-string/non-null to Missing;
entry.selmaho around1126 and registry native ID around846 omit diagnostics.
CDC's 20-case outside caller completed eight positive imports plus twelve exact
error controls; selmaho object/array and definition_id object/array each preserve
raw/lexical eligibility but lack the required field-named diagnostic (four failed
cases, two issues). Add source-occurrence diagnostics for unsupported optional
shapes while preserving raw, field order, entry/assertion counts and lexical
eligibility. selmaho accepts missing/null/string (including empty); other JSON
shapes get `invalid_selmaho`. definition_id accepts missing/null/string/number
with exact number lexeme; boolean/object/array get `invalid_definition_id`, native
ID None. Prefer the existing add_diag/report+release route, checking unique
unambiguous row fields once during projection. No coercion, row exclusion or new
semantic authority. Add actual public API controls for each invalid shape versus
missing/null/empty/string and large numeric valid counterparts; diagnostics must
name original field and occurrence, counts balanced in report/release. Check
through actual CLI too. Invalid-shape tests cannot pass by merely checking raw.

R2 — malformed supported literals vs valid unsupported metadata (row01).
parse_metadata's final else around296 currently classifies `bad = true junk` as
UnsupportedMetadata; the current contract requires malformed/trailing junk to be
InvalidMetadata. Keep the deliberately bounded grammar; no TOML dependency or
broad parser. Recommended: identify supported boolean/nonnegative-integer token
followed by nonempty trailing text (including whitespace separator) and reject
as InvalidMetadata; string JSON parsing already rejects trailing text. Test
`true junk`, `false junk`, `12 junk`, `"ok" junk` -> InvalidMetadata, plus integer
overflow/missing/duplicate. Preserve valid unsupported language/format errors as
UnsupportedMetadata, and keep an actual syntactically valid outside-subset value
such as single-quoted TOML string as UnsupportedMetadata. Do not turn every
unsupported future syntax into malformed, or vice versa. Required key wrong
accepted-grammar type remains InvalidMetadata. All corresponding CLI failures
retain jbotci_metadata, no output created/mutated.

R3 — unknown raw extension survives ambiguous typed-row suppression (rows02/03).
CDC's exact two-row control repeats definition in row0 and has a valid row1 with
same opaque large-number object. Row0 correctly remains excluded/raw-retained,
but early continue around1160 skips unknown-field extension+diagnostic; row1 has
both. Unknown-field table at slice-plan121 has no duplicate-row exception.
Binding distinction: typed semantic projection stays suppressed for ambiguous
rows, but indexed native evidence is unambiguous. Move/reuse unknown-field
raw extension+diagnostic collection before the duplicate-row continue (object
rows only). Each unknown field occurrence, including repeated unknown names,
gets exact raw lexeme, correct assertion owner, occurrence and field_index;
no winner, deduplication or typed fact. Non-object $row remains raw pseudo-field,
not an invented unknown object key. Test duplicate-known-key+opaque row, its
valid counterpart and repeated unknown names: all raw retained, excluded row
still no typed lexical hit, expected distinct indexed extensions/diagnostics.

R4 — truthful, pinned notices (row03).
Generated CodeNotice.locator is LICENSE-APACHE, absent from candidate; actual
root LICENSE is Apache2. Set code notice locator `LICENSE`; no new license file.
Supplier claim source is independently read at the known pin, so use exactly
`https://github.com/int19h/jbotci/blob/3d9fd41d9ed32de26311ee27ec86b6b165985aa1/THIRD-PARTY-NOTICES.md#L242-L252`
as the notice locator in raw CaptureMetadata, ContentNotice and SourceLineage
supporting evidence (a private shared constant is recommended). Do not interpolate
arbitrary options.revision into an evidential claim about a notice never read at
that revision. options.revision still controls distributor provenance/releaseID/
report and capture revision exactly as before; notice evidence names the actual
witnessed supplier claim. Preserve exact quote words, supplier-claimed coverage
qualification, Held redistribution, unresolved upstream lineage, separate Apache
code/content status. Compare revision0/revision1 stable raw/row/attribution
identities plus distinct release provenance as before; assert notice locator is
fixed pinned witness and code locator exists in package/repository. No legal
conclusion or public redistribution is authorized.

R5 — public example actually writes useful normalized release (row05).
Current example accepts only two paths and reports stdout; original plan160
requires paths and fresh output. Change signature/usage to RAW METADATA OUTPUT
(default exact input pins remain), read+validate+serialize before touching output,
then OpenOptions.create_new(true).write(true) and write_all canonical JSON bytes.
Example file bytes are exactly Dictionary::to_json_bytes (no LF); CLI framing
continues canonical plus one LF. Retain useful capture/accounting assertions and
report stdout; document file/canonical distinction. I/O is fallible, no overwrite
of existing file/symlink/directory, partial freshly-created file on actual write
failure remains documented as in CLI. Run actual pinned full input with fresh
output; reload equal dictionary and capture bytes; rerun occupied valid input and
prove unchanged file/target; invalid pin/input must not create output. Update all
example invocations/docs. Do not add network or SQLite to this example.

R6 — nonvacuous CLI regression control and exact filename (rows04/06/07).
Rename cli/tests/jbotci_cli.rs to issued jbotci_contract.rs. Its occupied-file
invocation omits synthetic pins, so it currently fails hash validation before
create_new and never tests the claimed branch. Supply the same valid revision/
raw/meta pins used by the successful import; assert exact `io` structured code,
nonzero/empty stdout and unchanged occupied bytes. Add symlink (cfg(unix) with
unchanged referent) and directory valid-pin controls; fresh counterpart succeeds.
Keep hash-mismatch negative as separate typed failure. CDC independently ran
these branches successfully; this repairs the automatic regression oracle,
not a claim that source create_new is broken. Preserve original vacuous attempt.

R7 — evidence/scope correction (row07).
Append all actual changes, corrected file allowlist claim, candidate failures,
new test populations and exact source/target/binary/lock identities. Do not
rewrite original passing full-fidelity results or failed controls into a single
success. CDC review01 has 915570 passing checks but does not accept this candidate.

## Applied guidance, gates and return

Existing Rust API validation/typed-error guidance motivates keeping invalid vs
unsupported metadata distinct and optional malformed values explicit rather than
coercing them. AP09/AP25 prohibit recoverable input panic; use existing Result
and add_diag helpers. CLI error/output guidance plus testing Proportionate Proof
requires valid pins to reach the intended create_new branch; hash failure is not
an overwrite witness. Work-verification correction review keeps old exact
attempts and independently checks changed claims. Source-local mechanics/private
helper names are local discretion; binding observable choices above are resolved.

After fixes run focused public API/process/example tests, then required Make
format→lint→check, isolated search no-default/all-features and affected umbrella
features, all with explicit fixed target/launcher. Retain old archives/Q01–Q06.
Execute full pinned public importer and actual CLI import/check at new exact
candidate, complete raw/capture/native/field accounting, and outside caller with
unique package+binary/offline lock and revision/shape/metadata controls. Reuse
unchanged raw-source census and approved oracles with exact pins, but record new
canonical SHA because corrected notice metadata intentionally changes output.
All source field fidelity and identity obligations survive. Preserve old candidate
and full outputs; no duplicate enormous speculative runs. Full corpus execution
is required, not an ignored test or script promise. No S02 persistence yet.

Return frozen clean local source commit and complete diff/path inventory,
first-executed-refinement status, seven row proposals with commands/exits/
populations, remaining limitations, failed/retried attempts and exact ownership
handover. CDC independently reviews/reproduces before any acceptance. Report
material milestones (focused controls, full import, gate/freeze), whether any
finite full check is still running; don't silently leave long processes. Source
and target remain yours until explicit freeze, then no mutation while CDC reviews.

Author readiness: CDC received full1559-line importer, full API/CLI tests/example/
doc and source diff at23d057b; actual outside/API/full raw/CLI tests reproduce the
specified mismatches. Notice witnesses and actual LICENSE read directly. Duplicate
raw extensions have exact indexed evidence independent of typed semantics; no
policy weakening. Guide07 was reloaded in bounded recovery after an aggregate
truncation. No architecture question is delegated, no extra investigation or
external action required. Current readback and repair recipe fit the retained
same-context source/intake packet with implementation headroom.
