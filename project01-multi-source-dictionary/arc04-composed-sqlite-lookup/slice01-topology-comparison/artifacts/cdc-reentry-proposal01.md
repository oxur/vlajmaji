# S01 count-only clean re-entry proposal — 2026-10-25

**PROPOSED, NOT execution authority.** Root has presented the specific asynchronous choice to Duncan; explicit response pending. All Full create/query/timing remains HOLD. General Arc04 authority still permits current source/tiny/test/gate work with one CC source/index/target owner.

Request: amend the total candidate-store cap from **five to seven**: preserve the early Jbotci and LLG stores as historical/unaccepted, then create a clean five-store comparison only AFTER all prerequisites. Keep the same seven working hours including preparation, 12 GiB new data/scratch including the historical two, 600-second bounds and three balanced rounds. No reset; no timing round has been used. Alternative: retain cap five and stop for review. This is a specific plan amendment, not a new model/general-budget question.

The issued initial prompt already required same-comparator controls before Full. CC began without those controls; CDC/root caught the ordering violation. Historical Jbotci create+verify took 40.424 seconds; LLG took 3.383 seconds; interruption occurred during La load before its create. Original stage log is retained. Original per-file source hashes and creation peak RSS were not captured; neither is passed retrospectively. No candidate query or comparison timing ran. Old stores are not deleted, recreated or counted as accepted comparison data.

## Concrete facts and resource forecast

CDC cheap metadata corroboration is [cdc-reentry-facts01.json](cdc-reentry-facts01.json): historical database files 698281984 +58327040 =756609024 bytes; La/Sut/U absent. Fresh destination `/tmp/vlajmaji-p01-a04-s01-cc/full-comparison02` is absent. Four release files total 435545300 bytes; proposed unified canonical is 435519540 bytes. Conservative forecast is ten times the five logical JSON payloads, plus old files and 1 GiB other scratch: **10540999248 bytes (9.82 GiB)**, below 12 GiB. The factor is an estimate, not a measured guarantee; actual new sizes/RSS/time still govern. No Full payload scan/database open/hash was used for this CDC resource check.

The preserved original executable is `/tmp/vlajmaji-p01-a04-s01-cc/topology_probe-initial-preserved`, SHA `f9a8eefcc1949b32af6128e167063ce7d5923d75c0ef01c34c23aaefd99f080e`. Original config SHA remains `e940fc673e4b577d0467dc786892b3c9d68aea3709a11a4a1bd703c41d15689f`. Its actual logs/manifests remain in CC execution history. These are not current-source acceptance.

## Current method evidence and snapshots

- Exact Rust population comparator: `population-comparator-04.log`, three tests, including reordered/empty positive, both directions of missing/extra zero category, multiplicity rejection and occupied-path nonmutation. Category counts alone are not complete value fidelity; canonical pins/native tuples retain that separate obligation.
- Safe wrapper: `/tmp/vlajmaji-p01-a04-s01-cc/stage_guard_time.py`, current SHA 55cdca8ba3396d7912b3a3f22240940e2d5b7f04e01613124c029e807e3bb31a. OS `/usr/bin/time -l` highwater is used, not sampled ps. Whole-prepare 600 seconds is a conservative STRONGER bound than each-stage 600; all accepted creates would share that true process highwater.
- `time-positive.json`: passed, OS max RSS 1671168 bytes. `time-timeout.json`: rejected at 0.282 seconds, no complete RSS (timeout failure). `time-rss-reject.json`: 32 MiB child allocation observed 41467904 bytes, correctly rejected imposed one-byte cap. `time-rss-missing.json` is actually command-not-found/child-exit rejection WITH 884736-byte RSS; its filename is not missing-RSS proof. Earlier sampled-ps controls are superseded.
- Actual shared parser controls in `time-rss-parser-controls-03.log`: positive 123 accepted; missing, zero and malformed rejected. Missing/zero/malformed RSS never counts as a cap pass.
- Root independently extracted the actual `parse_rss` AST from wrapper SHA `55cdca8ba3396d7912b3a3f22240940e2d5b7f04e01613124c029e807e3bb31a` and reproduced five parser cases: positive 123; missing, zero and malformed return None; two valid lines select the last value 34. This is independent parser-method evidence only, without child, corpus, database or target execution.
- Current release-probe snapshot SHA a494f8974cc4bc988183f459736b334d7b52579d744a8a9f79b0b30810dcff44, built by `probe-release-build-02.log` in 60 seconds. Source is UNFROZEN and incomplete: remaining identity-prefix/recipe completion and final gates must finish; executable/config/argv must be re-frozen after later changes.

## Proposed command and remaining execution gates

The following is the correct planned argument vector, NOT an executed command:

```sh
python3 -B /tmp/vlajmaji-p01-a04-s01-cc/stage_guard_time.py --timeout 600 --max-rss-bytes 12884901888 --log /tmp/vlajmaji-p01-a04-s01-cc/full-prepare-02-child.log -- /Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc01/target/release/examples/topology_probe prepare --manifest /tmp/vlajmaji-p01-a04-s01-cc/prepare-full02.json
```

`prepare-full02.json` is NOT yet complete/frozen: accepted per-pack/unified category maps and its final hash remain prerequisites. Ordinary prepare must be exclusive/fresh; no implicit resume/cache reuse. Before any launch, require returned count amendment, actual completed cheap controls, final literal config/source/binary/wrapper hashes, expanded argv/fresh destination checks and current resource/ownership readiness. Count approval alone does not waive those gates. All six slice, ten arc and fourteen project rows OPEN; no topology decision or A05/publication/default action.
