# PR3 hosted delivery receipt

Confirmed2026-10-25 by root's final `gh pr view` of the exact head and check rollup. [PR3](https://github.com/oxur/vlajmaji/pull/3) is OPEN, non-draft, base main, MERGEABLE at `f3551bc1a1e6be3d27f992f158310f8c7b3db4d3`. Root confirmed the source worktree clean and tracking origin with no ahead/behind. CDC records that fresh root confirmation here; no additional source/product/build/DB/benchmark execution was performed for this receipt.

All six final checks completed SUCCESS on the exact final head:

| Event/run | Check | Result | Duration |
| --- | --- | --- | --- |
| [Push40164318913](https://github.com/oxur/vlajmaji/actions/runs/40164318913) | [Check121082122486](https://github.com/oxur/vlajmaji/actions/runs/40164318913/job/121082122486) | COMPLETED SUCCESS | 34s |
| Push40164318913 | [macOS121082122784](https://github.com/oxur/vlajmaji/actions/runs/40164318913/job/121082122784) | COMPLETED SUCCESS | 1m9s |
| Push40164318913 | [Ubuntu121082122630](https://github.com/oxur/vlajmaji/actions/runs/40164318913/job/121082122630) | COMPLETED SUCCESS | 2m22s |
| [PR40164321769](https://github.com/oxur/vlajmaji/actions/runs/40164321769) | [Check121082130326](https://github.com/oxur/vlajmaji/actions/runs/40164321769/job/121082130326) | COMPLETED SUCCESS | 40s |
| PR40164321769 | [macOS121082130297](https://github.com/oxur/vlajmaji/actions/runs/40164321769/job/121082130297) | COMPLETED SUCCESS | 44s |
| PR40164321769 | [Ubuntu121082130147](https://github.com/oxur/vlajmaji/actions/runs/40164321769/job/121082130147) | COMPLETED SUCCESS | 2m25s |

This supplies the terminal hosted confirmation that was deliberately pending in both local repair verdicts. Repair01 (`c49b3c6`) replaced the NUL-sensitive manifest grep predicate with fail-closed forbidden-byte counting; its original Linux failure and local controls remain in pr3-ci-repair01-verification.md/evidence. The intermediate c49 Linux run already passed all7pipeline tests before encountering the separate E2BIG issue. Repair02 (`f3551bc`) moved full mutation SQL from argv into an owned file/stdin while preserving all4duplicate cases and the full fixture; its failure/local controls remain in pr3-ci-repair02-verification.md/evidence. Both final Ubuntu jobs now pass, alongside both Check and macOS jobs. Earlier failed runs and local-only statuses remain preserved as historical evidence.

Delivery is the pushed feature branch and open PR; no merge, release/package publication, deployment or main/Bauske mutation is claimed or authorized by this receipt. Accepted Arc02 and subsequent bounded acceptance/measurement/README/repair records are not reopened or rewritten. All14projectrows remain OPEN; A03 remains unopened. This is a planning-only receipt, using the existing artifact home and scoped local commit discipline.
