# S06 COMPLETION03/5 — ISSUED after ROOT approval, 2026-11-01

Own path: `arc06-integrated-dictionary-workbench/slice06-computed-and-proposal-queries/cc-prompt-iteration02.md`.
Predecessor: [cc-prompt-iteration01.md](cc-prompt-iteration01.md), CORRECTION02/5. This is assignment THREE of FIVE, without reset. Operator approved this proportionate-proof replan through ROOT on 2026-11-01. Retained CC Luna/high and CDC Sol6.1/high continue THROUGH ALL Arc06.

Complete S06's remaining real gates and usable source handback. Product APIs, amended fixture expectations and all eight [ledger](ledger.md) criteria remain unchanged. This prompt supersedes conflicting agent-created proof procedures in older prompts/plans; preserve those files and all actual failures. Stop work on `artifacts/independent-method-draft01.py` and its DRAFT23 protocol: retained, superseded, never launched as product verification; prior pure checks keep their recorded scope. No new runner/validator/binding manifest, 54-file inventory, transcript reconstruction, per-command export apparatus or new test matrix.

## Resume the existing work

Read this prompt, the new CURRENT plan/ledger paragraphs and eight ledger rows. Retain the already-loaded product/API/fixture contract and recent execution census/errata; no repeated historical intake or duplicate read manifest. Inspect `git status --short` and the current diff for unexpected drift. A brief acknowledgment in existing execution.md is enough; no separate intake/readiness artifact.

SOURCE=`/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc04/vlajmaji`, branch `codex/project01-integrated-workbench`, HEAD `d8bd64e248b7193f9244cbe527e7fac1f7926ce7`, current dirty 15-path S06 implementation. The unchanged source already passed Make session91796 (496 Rust passes/0 failures/5 ignored +19 tooling) and isolated no-default session97728 (134/0/0). REUSE these results; do not replay for paperwork. The ignored research-input corpus test remains unrun. No archive or later gate has run.

ROOT has released this prompt; CC retains exclusive SOURCE/index/ONEtarget ownership until explicit return. CDC/ROOT remain read-only there. Source original ENV `/private/tmp/vlajmaji-p01-a06-s01-original-env.sh` FIRST, then fresh HOME/XDG_DATA_HOME/XDG_CONFIG_HOME/TMPDIR for gate work; fixed Cargo/Rustup homes, literal PATH, offline/auto-install0 and `CARGO_TARGET_DIR=/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc01/target`. Assert physical cwd; target is environment-only. Use program `make`, CARGO=`/tmp/vlajmaji-p01-a05-cargo`, RUST_TOOLCHAIN=1.98.0. Execution resumes after the named issue checkpoint and explicit CDC dispatch.

## Remaining commands, in order

1. **ONE isolated checkout for genuine all-features CI.** Use `git worktree add --detach /Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc04/vlajmaji-s06-allfeatures-archive01 d8bd64e248b7193f9244cbe527e7fac1f7926ce7`. Overlay the current 15 allowed files plus unchanged Cargo.lock using ordinary copy/tar from SOURCE, without caches/targets/data directories. Its only purpose is isolating required Synthetic fixtures. Occupied checkout/fixture destinations stop dependent work; no deletion/recreation.

   In that checkout, run the current CI recipes directly, with `/tmp/vlajmaji-p01-a05-cargo +1.98.0` as Cargo prefix:
   ```sh
   cargo run --locked --offline -p vlajmaji-cli -- data-pack fixture jbotci crates/data-lensisku-jbotci/data
   cargo run --locked --offline -p vlajmaji-cli -- data-pack fixture llg crates/data-llg/data
   cargo run --locked --offline -p vlajmaji-cli -- data-pack fixture la-lojban crates/data-la-lojban/data
   cargo run --locked --offline -p vlajmaji-cli -- data-pack fixture sutysisku crates/data-sutysisku/data
   scratch="$(mktemp -d "${TMPDIR}/vlajmaji-owned.XXXXXX")"
   mkdir crates/data-owned/data
   cargo run --locked --offline -p vlajmaji-store --example owned_curation -- "$scratch/inputs" "$scratch/journal" "$scratch/view.json"
   cargo run --locked --offline -p vlajmaji-cli -- owned export "$scratch/journal" --output crates/data-owned/data/snapshot.json --seal crates/data-owned/data/seal.json
   ```
   Then ONCE: `make test-all-features CARGO=/tmp/vlajmaji-p01-a05-cargo RUST_TOOLCHAIN=1.98.0`. Do not change SOURCE during it.
2. Return to physical SOURCE. Only affected package FILE checks:
   `/tmp/vlajmaji-p01-a05-cargo +1.98.0 package -p vlajmaji-search --list --allow-dirty --locked --offline`, then the same command for `vlajmaji-cli`. Store is unchanged; reuse its inherited package evidence. These are FILE listings, not publish/verification.
3. Build defaults once: `/tmp/vlajmaji-p01-a05-cargo +1.98.0 build -p vlajmaji-cli --locked --offline`; then `/tmp/vlajmaji-p01-a05-cargo +1.98.0 build -p vlajmaji-search --example integrated_workbench --locked --offline`. CLI default includes morphology; Search default example does not. Record executable paths and these actual feature commands for the independent caller proof. No further author producer/guide replay.

No product edit is anticipated. A real bug gets the smallest normal focused correction and affected check, not expanded machinery. A genuine unexpected gate failure stops dependent work for concrete review; never replay a successful gate to improve paperwork.

## Exact source allowance and return

Same15 paths, relative to SOURCE; no added allowance:
```text
crates/search/src/lib.rs
crates/search/src/dictionary.rs
crates/search/src/dictionary/projection.rs
crates/search/src/workbench.rs
crates/search/src/workbench/lexical.rs
crates/search/src/workbench/computed.rs
crates/search/src/workbench/links.rs
crates/search/src/presentation.rs
crates/search/examples/integrated_workbench.rs
crates/search/tests/workbench_computed_contract.rs
crates/search/README.md
crates/cli/src/main.rs
crates/cli/tests/workbench_computed_contract.rs
docs/integrated-workbench.md
README.md
```

Stage only the three new allowed files, then `git commit --only` these15 named paths with adjacent trailers:
```text
Co-authored-by: Codex <noreply@openai.com>
Co-authored-by: Billo AI <ai-engineering@billo.systems>
```
Confirm clean SOURCE/index, actual commit/tree/parent, protected refs unchanged, and final default executable paths. Append concise actual commands/results/log paths and known limits to existing execution.md; create/update slice-root closing-report.md as PROPOSED, identifying this assignment and all eight obligations. No new evidence apparatus. Return SOURCE/index/ONEtarget explicitly QUIET. Do not mark ledger rows done.

CDC then performs ONE independent actual diff/test review, reuses existing tests (one Search morphology package run plus CLI computed contract can cover the new paths), and uses a small direct default-example/documented-CLI journey: two genuine feature-off observations; morphology positives; current-r2 Compose0 versus historical1. Simple one-off assertions and existing fixed expectations suffice; record findings/results in canonical cdc-verification.md, not DRAFT23. Retain exact coverage limits. Independent disposition/normal closing/ROOT consistency precede child-only close.

S07 remains original four pinned corpora + owned/full functional composition, runnable guide, NEW independent public Rust consumer and SAME-context canonical review/ROOT consistency; one assembled proof may satisfy S07 and Arc06. No external delivery, default-store access, retained Full reuse, acquisition or cleanup. Material product changes return to ROOT/Operator.
