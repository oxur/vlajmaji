# Arc06 delivery — 2026-11-01 (America/Chicago)

**Delivery complete. PR merged; local main clean and synced; merged-main CI successful.**

Duncan authorized “let’s go ahead and merge to main.” [PR #8](https://github.com/oxur/vlajmaji/pull/8) targets main from `codex/project01-integrated-workbench`. GitHub permits rebase merges only. The fetched main baseline was `cb13d377aeba85c1b76a9173abb22bedd535ac69`.

## Accepted implementation and delivery corrections

Arc06 was independently accepted and closed locally at source `27fc2be6081a3efd551c22be4a77bb871bfe46c7`, tree `7444b0798efc2dd3edfb2a278555177f5f241d70`; see the [shared independent composition and canonical review](../../arc06-integrated-dictionary-workbench/slice07-complete-offline-workbench/artifacts/composition-review.md). This delivery preserves that closed evidence.

Three bounded follow-up commits prepare delivery:

- `a479ba6c5f10ad6ae768103cb5d956e0ecd730c5`: correct the guide’s jbotci native assertion 333 wording; no executable change.
- `3a1b9977be2f7e947d9f66c7ea257d8b0e7db67f`: correct a CLI test’s `unix_bytes` expectation to the established `unix-bytes` wire spelling, and assert serialization before the filesystem capability probe so macOS checks it too. Correct the same spelling in the manually constructed display fixture. No production change.
- `7df2081c9ebcdfcb7fda7d89642aefa18f6ddfd0`, tree `af0f8c63fb9e6ba4a84adc7a2613eee086bd5a59`: set CI-only Cargo development/test debug information to zero after observed runner disk exhaustion. All tests, feature checks, and debug assertions remain enabled.

The final candidate differs from the accepted source in exactly three paths: the guide, one CLI integration-test file, and CI configuration. The README already describes the Arc06 capabilities and links the integrated guide and offline four-pack/owned-history recipe.

## Validation and original failures

| Candidate | Push CI | PR CI | Observation |
| --- | --- | --- | --- |
| `a479ba6` | [41380869163](https://github.com/oxur/vlajmaji/actions/runs/41380869163): failure | [41380916923](https://github.com/oxur/vlajmaji/actions/runs/41380916923): failure | Both Linux workspace jobs exposed the incorrect test expectation; both Check and macOS jobs passed. |
| `3a1b997` | [41381431189](https://github.com/oxur/vlajmaji/actions/runs/41381431189): failure | [41381433694](https://github.com/oxur/vlajmaji/actions/runs/41381433694): success | Push Linux passed workspace/no-default suites, then ran out of disk compiling all features. The separate PR Linux job passed the entire matrix. |
| `7df2081` | [41382245384](https://github.com/oxur/vlajmaji/actions/runs/41382245384): success | [41382248530](https://github.com/oxur/vlajmaji/actions/runs/41382248530): success | All six jobs passed before merge, including both Linux all-feature suites. |

Representative complete raw failed jobs are preserved as [native encoding failure](native-encoding-failure.log) and [runner disk failure](runner-disk-failure.log). These remain failed attempts, not successful reruns or product-defect claims.

After each delivery correction, the required offline local format, lint, and Make check passed. Each Make check completed 496 Rust tests, zero failures, five explicitly ignored tests; all 19 tooling checks passed, along with documentation generation. The test repair also passed the focused profile suite (six passed, one ignored child invoked by its parent). Workflow validation with actionlint passed. [Final local raw check](local-check.log) preserves the current result. Local checks used the existing retained Arc01 target; no full-corpus replay or new embedded payload was needed.

The five ignored tests retain their stated limits; hosted synthetic feature checks are not new independent full-corpus proof. The real-data 909,970,491-byte Full Links response remains a known limitation, with [BL-0003](../../../backlog.md#bl-0003-investigate-excessive-full-links-response-expansion) committed on planning at `bf12d90`. That investigation remains unscheduled.

The initial commit-and-push approval review timed out before execution; its permitted retry succeeded. No unsafe-action rejection or duplicate push was inferred.

## Scope boundary

All fourteen Project acceptance rows remain open for separate Operator acceptance. This delivery does not publish packages, change corpus rights, access the default store, replay full-corpus work, or open another arc. The planning branch is local; a source/main merge does not imply a planning push.

## Merge receipt

PR #8 was rebase-merged at `2026-11-02T03:31:24Z` to `2724a79daf9a61010685a953708e9f07d643eeda`. The primary local main checkout was fast-forwarded and observed clean at that commit. Its tree `af0f8c63fb9e6ba4a84adc7a2613eee086bd5a59` exactly equals the validated `7df2081` candidate. An actual ordered comparison confirmed all thirteen corresponding intermediate trees match across the rebase, with zero merge commits.

[Post-merge main CI41383563108](https://github.com/oxur/vlajmaji/actions/runs/41383563108) completed SUCCESS on the actual merged commit. All three jobs passed, including Linux workspace, isolated no-default, all five synthetic data preparations and all-feature tests. The [API receipt](main-ci.json) preserves actual job and step dispositions. A final live GitHub branch read matched the clean local main at `2724a79daf9a61010685a953708e9f07d643eeda`.
