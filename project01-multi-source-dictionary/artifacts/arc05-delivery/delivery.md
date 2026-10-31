# Arc05 delivery — 2026-10-31

Duncan authorized “let's merge this work into main ... oh, and let's update the README, too!” and asked for a minimal README example of an Arc05 feature. This delivery follows the existing local Arc05 closure; it does not reopen the closed slices or activate Arc06.

[PR #7](https://github.com/oxur/vlajmaji/pull/7) was rebase-merged at 2026-10-31T20:37:54Z. Primary local `main` is clean and synced at `cb13d377aeba85c1b76a9173abb22bedd535ac69`, tree `c8a65825b49c718c90e5b6f370b1f22358a5131a`. That tree equals the validated branch candidate `69a9f68507e23cef9b20e4bcabc86840e224689e`; all seven rebased commits also retain their corresponding original trees. See [merge receipt](merge-receipt.json) and [mapping](rebase-tree-map.json).

The README describes delivered owned-history/curation/package behavior and includes a short self-contained synthetic example. Its four commands were executed unchanged: generate/export/import five revisions, list the imported history, read `owned-r1`, and check the sealed snapshot. The complete historical dictionary matched its original input. [Literal example](readme-example.sh), [stdout](readme-example.stdout.txt), [stderr](readme-example.stderr.txt), and [verification/observer qualification](example-verification.json) preserve the actual result. The observer initially searched the wrong temporary parent; it then used the actual stdout path for read-only comparison, without rerunning any product command.

## Validation and observed corrections

| Candidate | Push CI | PR CI | Disposition |
| --- | --- | --- | --- |
| `a3be69a` (README) | 41238152142 success | 41238168171 failure | macOS CLI maintenance fixture `create_dir` returned AlreadyExists before product assertions |
| `3877f7c` (CLI fixture repair) | 41238701485 failure | 41238705824 success | macOS store policy fixture encountered a separate AlreadyExists setup failure |
| `69a9f68` (both repairs) | 41239342795 success | 41239347366 success | all six checks passed before merge |

Both failed jobs are preserved byte for byte: [CLI failure, job124646232812](ci-first-macos-failure.log) and [policy failure, job124647822396](ci-second-macos-failure.log). No automatic job rerun was used. Each repair received retained CDC read-only acceptance; those reviews made no runtime claims. Root ran the required format/lint/tests/docs/tooling gates after each change. The final [local gate](local-check.log) exited zero: 395 Rust tests passed, zero failed, one corpus test intentionally ignored; 19 tooling checks passed separately. [Local receipt](local-receipt.json) records the exact candidate and source boundary.

After accepted implementation `d2a566c6`, the only changed paths are the README, the CLI integration-test file, and the store policy file's `#[cfg(test)]` module. The production prefix of that store file is identical. Both fixture allocators now use exclusive creation, an atomic per-process sequence, and bounded retries only for AlreadyExists. Deterministic controls verify occupied directory/file contents survive. The original collision cause and exact historical collided path remain unproven.

[Premerge receipt](premerge-receipt.json) confirms the exact head/base and both successful runs. [Merged-main CI41239808260](https://github.com/oxur/vlajmaji/actions/runs/41239808260) subsequently completed SUCCESS: Check, macOS tests, and Linux tests including isolated no-default and all-features gates. [Main CI receipt](main-ci-receipt.json) records actual job and step results.

All fourteen project rows remain OPEN; Arc05 remains CLOSED and Arc06 unactivated. No release, planning push, default-store access, full-corpus replay, or cleanup was performed. Historical closed review packets and original failure evidence remain unchanged. [Manifest](manifest.json) binds the preserved delivery files.

Preservation note: the initial planning whitespace check flagged whitespace already present in raw CI logs and a raw local log EOF. Those bytes remain unchanged. Authored Markdown/JSON is checked separately; the manifest verifies every preserved file.
