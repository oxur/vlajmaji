# Slice02 Rust library probes

This is an isolated, `publish = false` binary package for Project02 Arc01 Slice02. Its own workspace and lockfile pin `latkerlo-jvotci 2.10.2601`, `vlazba 1.0.11`, and the JSON projection utilities; it is not part of the production workspace. The package default feature explicitly forwards `vlazba/default`; lean mode uses `--no-default-features`. Rust and Cargo 1.98.0 were used.

Run from the repository root. Every Cargo command uses ignored root `target/p02-probes/`; the prosody replay invokes `target/p02-probes/debug/relojban` directly, so keep this target location when building the CLI for that replay. The package-local `rust-probes/target` path is not used.

```sh
CARGO_TARGET_DIR=target/p02-probes cargo fmt --manifest-path experiments/E02-resource-discovery/rust-probes/Cargo.toml -- --check
CARGO_TARGET_DIR=target/p02-probes cargo check --manifest-path experiments/E02-resource-discovery/rust-probes/Cargo.toml --locked --offline
CARGO_TARGET_DIR=target/p02-probes cargo clippy --manifest-path experiments/E02-resource-discovery/rust-probes/Cargo.toml --locked --offline -- -D warnings
CARGO_TARGET_DIR=target/p02-probes cargo test --manifest-path experiments/E02-resource-discovery/rust-probes/Cargo.toml --locked --offline
CARGO_TARGET_DIR=target/p02-probes cargo check --manifest-path experiments/E02-resource-discovery/rust-probes/Cargo.toml --locked --offline --no-default-features
CARGO_TARGET_DIR=target/p02-probes cargo clippy --manifest-path experiments/E02-resource-discovery/rust-probes/Cargo.toml --locked --offline --no-default-features -- -D warnings
CARGO_TARGET_DIR=target/p02-probes cargo test --manifest-path experiments/E02-resource-discovery/rust-probes/Cargo.toml --locked --offline --no-default-features
CARGO_TARGET_DIR=target/p02-probes cargo run --manifest-path experiments/E02-resource-discovery/rust-probes/Cargo.toml --locked --offline > experiments/E02-resource-discovery/probes/runs/default.jsonl 2> experiments/E02-resource-discovery/probes/runs/default.stderr
CARGO_TARGET_DIR=target/p02-probes cargo run --manifest-path experiments/E02-resource-discovery/rust-probes/Cargo.toml --locked --offline --no-default-features > experiments/E02-resource-discovery/probes/runs/no-default-features.jsonl 2> experiments/E02-resource-discovery/probes/runs/no-default-features.stderr
CARGO_TARGET_DIR=target/p02-probes cargo build --locked --offline -p relojban-cli --bin relojban
python3 experiments/E02-resource-discovery/probes/recipe/join-candidates.py
python3 experiments/E02-resource-discovery/probes/recipe/run-prosody.py > experiments/E02-resource-discovery/probes/runs/prosody.jsonl
```

The join consumes all 113 records from the captured Lensisku API response files for exact matching; it embeds the 16 English records in the derived artifact. Generation cases, feature modes, caller-declared kinds, limits and interpretation boundaries are enumerated in the Slice02 plan and captured in the JSONL rows. The join reads frozen Slice01 captures and its derived inventory; it does not fetch, change, or reproject raw data. Probe results are observations about these package versions and supplied inputs, not legality, semantic equivalence, or Relojban conformance.
