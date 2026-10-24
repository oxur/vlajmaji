# AGENTS.md

This file provides guidance to LLM-based coding agents when working with code in this repository.

## Project Overview

Vlajmaji is a Rust dictionary workbench for (Re)Lojban lexical archives,
rebuildable indexes, source-preserving search, and a JSON CLI. Records retain
original source bytes, snapshot identities, locators, competing assertions, and
source-native form roles. Computed analyses and caller proposals remain separate
from captured lexical material.

The workspace is at version 0.2.0, uses Rust edition 2024, and
requires Rust 1.98 or newer. Development commands and CI select Rust 1.98.0.
Supported platforms are macOS and Linux. Corpus data is not bundled with the
current implementation; synthetic fixtures support ordinary offline tests.

Bauske must be checked out as a sibling: the workspace resolves
`bauske-phonology` through `../bauske/crates/phonology`. Bauske has its own
repository, instructions, and commits.

## Design and Project Planning Documents

- Start with `README.md`, the crate READMEs, and `docs/migration.md`. The migration
  guide preserves historical commands; its 0.2.0 section identifies the current
  `vlajmaji` binary, replacing the old `relojban-search` name.
- Current project authority lives on the separate orphan `planning` branch in
  `.worktrees/planning/`. Read that worktree's `AGENTS.md`, then
  `project01-multi-source-dictionary/project-plan.md`, `design.md`, and `ledger.md`.
  Follow the relevant arc/slice plan and issued assignment for ledgered work.
- Historical research under the planning project's `artifacts/research/` is
  preserved reference material. Use current plans for assignments and acceptance;
  do not reinterpret historical instructions as current authorization.
- Plans can describe implementation in an isolated feature worktree that is not
  yet on `main`. Check `git worktree list`, the selected branch, and actual source
  before assuming a planned or accepted API is available in this checkout.
- `make planning` fetches or creates the planning worktree and can push an empty
  orphan branch when none exists remotely. Reuse the existing worktree first.
- Keep plans, ledgers, prompts, and verification records in the planning worktree.
  Substantive project implementation follows the assigned non-default branch and
  worktree ownership. Green checks alone do not establish independent acceptance.

## Crate Architecture

| Package | Path | Responsibility |
| --- | --- | --- |
| `vlajmaji` | `crates/vlajmaji` | Umbrella library re-exporting `lexicon`, `store`, and `search`, with opt-in `morphology` and `graph` features; its README also supplies its rustdoc landing page. |
| `vlajmaji-lexicon` | `crates/lexicon` | Canonical source records, raw bytes, provenance, occurrence identities, proposals, imports, and lexical archives. |
| `vlajmaji-store` | `crates/store` | Rebuildable SQLite/FTS5 indexes and an optional Oxigraph/RocksDB backend. |
| `vlajmaji-search` | `crates/search` | Source-preserving queries, spelling distance, phonological/prosodic filters, and optional compound analysis. |
| `vlajmaji-cli` | `crates/cli` | The `vlajmaji` JSON binary, command parsing, store creation/info, and query input/output. |

All workspace packages, including the CLI, live under `crates/`.
Keep reusable data, persistence, and query behavior in the libraries.
`vlajmaji-search` uses the sibling Bauske phonology library; its optional
`morphology` feature enables `latkerlo-jvotci`. The CLI enables that feature.
`vlajmaji-store` has no default features; `graph` enables Oxigraph/RocksDB and
requires native C++ and Clang/libclang tooling.

Use absolute crates.io, docs.rs, and repository links in the umbrella README so
they work in the packaged README and generated rustdoc. The CLI binary has
`doc = false` to avoid overwriting the umbrella library's `vlajmaji` docs; link
to its package and CLI guide instead of expecting a library API page.

Integration tests live in each package's `tests/` directory. The repository-level
`tests/test-build-tools.sh` exercises Make and the planning helper with disposable
local Git fixtures and a Cargo stub.

## Build & Test Commands

Run `make` or `make help` from the repository root to discover available targets.
Make uses `RUST_TOOLCHAIN=1.98.0` by default; override it explicitly when validating
another supported toolchain. The helper scripts require Bash 3.2 or newer and Git.

Always run `make format` after changes, then `make lint` before testing.
Before committing, always run `make check`.

| Command | Purpose |
| --- | --- |
| `make build` | Build the workspace and copy the debug `vlajmaji` binary to `./bin`. |
| `make build-release` | Build and install the optimized binary. |
| `make format` / `make fmt-check` | Format Rust / check formatting without modifying files. |
| `make lint` | Run workspace Clippy across all targets with warnings denied. |
| `make test` | Run locked workspace tests with default features. |
| `make test-no-default` | Test `vlajmaji-search` alone with optional morphology disabled. |
| `make test-all-features` | Run locked workspace tests including the graph backend. |
| `make test-tools` | Run shell syntax checks and the isolated build-helper suite. |
| `make docs` | Build workspace API documentation without dependency docs. |
| `make check` | Check tools, formatting, Clippy, default tests, docs, and build helpers. |

`make check` does not include the isolated no-default or all-features gates.
Run those for changes affecting feature boundaries or backend behavior. A
workspace-wide no-default test does not isolate search: the CLI can re-enable
morphology through Cargo feature unification.

The full-corpus test is intentionally ignored unless `RELOJBAN_RESEARCH_ROOT`
points to the preserved research checkout. Its exact invocation and importer
commands are documented in `docs/migration.md`. Report that test as unrun when
the inputs are unavailable; fixture success is not full-corpus verification.

CI lives in `.github/workflows/ci.yml`. It tests macOS and Linux, with Linux-only
no-default and all-features checks, and checks out a pinned Bauske revision beside
Vlajmaji. Update both dependency-checkout pins together when changing that
baseline. Publication targets are separate from validation and require an explicit
release request; publish the Bauske dependency first when releasing these crates.

## Conventions

- Test naming: `test_<fn>_<scenario>_<expectation>` for new tests.
- Load `/rust-guidelines` before writing Rust code, including its anti-patterns guidance.
- Honor workspace lints: unsafe code is forbidden and missing documentation is
  warned about. Document public APIs and retain meaningful error distinctions.
- Preserve original bytes, record multiplicity, identities, source roles, and
  explicit absence/unknown states. Keep source assertions, normalized values,
  derived indexes, computed scores, and user proposals distinguishable.
- Version and document incompatible archive/API changes; retain explicit legacy
  interpretation instead of silently rewriting historical captures or IDs.
- Search scores and generated compounds do not establish attested usage,
  semantic equivalence, or poetic merit.
- Keep ordinary builds and loads offline. Corpus content terms and redistribution
  decisions remain separate from the code's Apache-2.0 license.
- Logging via the `twyg` crate, config via the `confyg` crate when those
  capabilities are introduced; neither is currently a workspace dependency.
- Keep shell helpers compatible with Bash 3.2 and test failures as well as success.
- Keep changes scoped to the task and preserve unrelated working-tree changes.

## Commit Message Guidance

Commit only the intended files, using explicit paths and `git commit --only`
when other work may be present. Every assistant-authored commit message must
include these trailers:

```text
Co-authored-by: Codex <noreply@openai.com>
Co-authored-by: Billo AI <ai-engineering@billo.systems>
```

## Workbench

`workbench/` is reserved for operator scratch and temporary files. Keep it
excluded from version control and verify exclusion with `git check-ignore`
before writing scratch files there; use a temporary directory outside the
repository when no ignore rule is available. Keep durable project evidence in
the planning worktree rather than relying on scratch files.
