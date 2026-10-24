# Lexical workspace migration

The four lexical/search packages moved from the Relojban poetry source tree at accepted baseline [`7a05516b5d429f3184eaf6a2bde2fd9c5d12e9bf`](../.worktrees/planning/project02-wordcraft-reconnaissance/arc02-lexical-data-search/closing-report.md) to the separate checkout `/Users/oubiwann/lab/oxur/vlajmaji` (branch `feature/vlajmaji-migration`). The initial destination migration commit is `5436c2c56486e2fe026264aed0eecb2b76ffd54d`; the initial source retirement commit is `c2d7fdd66e23e57d4fc35a3c9a8841f601dd57b8`. Follow-up correction commits are linked from the planning artifact. The migration remains subject to independent disposition. The canonical Arc02 work itself is accepted; see the [Arc02 closure](../.worktrees/planning/project02-wordcraft-reconnaissance/arc02-lexical-data-search/closing-report.md). Older status wording in the Arc02 section of this README and earlier research notes is a historical snapshot.

| Historical R package/path | Current V package/path |
| --- | --- |
| `relojban-lexicon` at `crates/relojban-lexicon` | `vlajmaji-lexicon` at `crates/lexicon` |
| `relojban-lexicon-store` at `crates/relojban-lexicon-store` | `vlajmaji-store` at `crates/store` |
| `relojban-search` at `crates/relojban-search` | `vlajmaji-search` at `crates/search` |
| `relojban-search-cli` at `crates/relojban-search-cli` | `vlajmaji-cli` at `crate/cli` (binary remains `relojban-search`) |

In V, build the workspace with `cargo +1.98.0 build --locked --workspace`. Build a SQLite store from the accepted archive with `cargo +1.98.0 run --locked -p vlajmaji-cli -- build <ARCHIVE> <STORE>`, inspect counts with `cargo +1.98.0 run --locked -p vlajmaji-cli -- info <STORE>`, and run an exact spelling query with `cargo +1.98.0 run --locked -p vlajmaji-cli -- search <STORE> --query '{"operation":"exact","spelling":"farvi"}'`. The explicitly gated full-corpus test command and its required `RELOJBAN_RESEARCH_ROOT` are documented in V at `/Users/oubiwann/lab/oxur/vlajmaji/docs/migration.md`.

The prior E02/E03 probes, manifests, reports and run receipts remain unchanged and describe historical Arc02 execution against source baseline `7a05516`. To replay those records, check out that baseline in a separate R worktree; their original source paths were removed from the current R tree and those historical commands do not target V's renamed packages. Current destination commands are recorded in V's migration guide. The R archive verifier remains available at `../archive/verify.py`.


## Superseding phonology bridge status — 2026-10-23

The temporary absolute Relojban-research path described in the initial V migration was replaced by the sibling dependency `bauske-phonology = { path = "../bauske/crates/phonology" }`. V source imports use `bauske_phonology`; package identities and search policy/data remain unchanged. A standalone V checkout therefore requires its Bauske sibling at that path. The Bauske source migration is documented in [`bauske-migration.md`](bauske-migration.md); its independent disposition is tracked separately from this accepted V migration.
