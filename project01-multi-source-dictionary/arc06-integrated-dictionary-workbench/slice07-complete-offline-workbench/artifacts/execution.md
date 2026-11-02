# S07 INITIAL1/5 — CC execution record (2026-11-01)

Source: `/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc04/vlajmaji`, starting at accepted `b5da0fe3f2967f24b26f614ced0d30fc3ba604c0` (`ac4e2fcefdb93112e76961dc27d227fc029a3344`). The current S07 v0.3 prompt authorized documentation changes only to `docs/integrated-workbench.md`, `README.md`, and `crates/search/README.md`.

Delivered a full offline four-pack plus owned-history recipe in `docs/integrated-workbench.md`, linked from the root and search-crate READMEs. It identifies the eight exact original inputs and pins; gives native materialization, owned init/edit/export, strict sealed manifest/install, historical r1 and configured Head refresh/query commands; and distinguishes the public pure `load_owned_snapshot` validation from explicit journal import. The guide also includes typed public Rust Exact and Links query calls, complete farvi expected counts and source-group counts, provenance/rights cautions, and the default Search versus CLI morphology feature distinction.

I inspected the relevant current command definitions, request types, `WorkbenchSearchEngine::open`, `execute`, and `execute_lexical` before publishing their spellings. `git diff --check` passed; committed only the three authorized paths. No build, test, corpus materialization, destination store, or runtime query was performed by this docs-only assignment. The checked-in recipe and Rust snippets are documentation, not an executed functional proof.

Commit: `ea049896089efcc5d74e8349a04f639f66614d8d`; parent: `b5da0fe3f2967f24b26f614ced0d30fc3ba604c0`; tree: `b0ebb3480bbedb8523c8d8d1d5a1977c53193249`. `git show --check` passed and the source checkout was clean after commit. The authorized changed paths were exactly `README.md`, `crates/search/README.md`, and `docs/integrated-workbench.md`.

All eight S07 ledger rows remain OPEN. The NEW independently authored public Rust consumer and assembled four-corpus/owned functional proof, same-context canonical review, ROOT consistency, later S07/Arc06 row dispositions, and separate Operator project acceptance remain pending. No product/API defect or plan change was found in this documentation pass. A separate S06 closing-report append from the superseded follow-up is preserved as existing history and was not included in this S07 source commit.

## 2026-11-01 narrow API documentation correction

After CDC's readback of the public `OwnedJournal::import_snapshot` signature (`root: &Path`), corrected the one Rust example call in `docs/integrated-workbench.md` to pass `&destination`. `git diff --check` passed. Commit `7d80155ba21953b702d39c61e8d159cb8097609a` has parent `ea049896089efcc5d74e8349a04f639f66614d8d`, tree `5bdaabb463f7547b0e9d71b006f0e534040f8684`, and changes only `docs/integrated-workbench.md`; `git show --check` passed and the source checkout is clean. The previous S07 docs commit remains in history. No build or runtime was run; the independent proof remains pending.

## 2026-11-01 CLI alternative clarification

A second docs-only readback found that the shell recipe later assumes the imported journal exists, while the guide had not shown the exact CLI alternative to the Rust import example. Added the source-verified `owned check --input ... --seal ...` and `owned import DEST --input ... --seal ...` commands, explicitly offering them instead of the Rust route. `git diff --check` and `git show --check` passed. Commit `27fc2be6081a3efd551c22be4a77bb871bfe46c7` changes only `docs/integrated-workbench.md`; parent `7d80155ba21953b702d39c61e8d159cb8097609a`; tree `7444b0798efc2dd3edfb2a278555177f5f241d70`. No product build or runtime was run.
