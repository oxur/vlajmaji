# P01 phonology and CLI migration to Bauske

This bounded migration moves the two active P01 Rust packages from the Relojban poetry research checkout to `/Users/oubiwann/lab/oxur/bauske`, branch `feature/bauske-migration`. It changes only workspace location and Rust package/library identifiers; source behavior, phonology policies, CLI interface and persisted research data are retained. The binary remains named `relojban`.

| R package/path | B package/path | Public Rust identity |
| --- | --- | --- |
| `relojban-phonology` at `crates/relojban-phonology` | `bauske-phonology` at `crates/phonology` | `bauske_phonology` |
| `relojban-cli` at `crates/relojban-cli` | `bauske-cli` at `crates/cli` | `bauske_cli` |

The accepted lexical/search workspace remains in `/Users/oubiwann/lab/oxur/vlajmaji`; its workspace dependency resolves Bauske at `../bauske/crates/phonology`. Its binary remains `relojban-search`. V's initial source-to-destination record is [`vlajmaji-migration.md`](vlajmaji-migration.md).

Build and inspect phonology from Bauske:

```sh
cd /Users/oubiwann/lab/oxur/bauske
cargo +1.98.0 build --workspace --locked
cargo +1.98.0 test --workspace --locked
cargo +1.98.0 run --locked -p bauske-cli -- inventory
```

The migration preserves B/V's pre-existing LICENSE and `.gitignore` bytes and makes no project-wide license selection or source-license inference. Frozen local source refs are B `9c409ba` and V `e20e60b` on `feature/bauske-migration`; R retirement is committed separately below. R remains the research and immutable-evidence home; its `target/` directory is an ignored warmed build cache, not active Rust source. The migration's candidate evidence and independent disposition are recorded under `.worktrees/planning/project02-wordcraft-reconnaissance/migration-bauske/`.
