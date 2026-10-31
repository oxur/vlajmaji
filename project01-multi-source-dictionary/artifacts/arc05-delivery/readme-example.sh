(
  set -eu
  run_dir="$(mktemp -d)"
  cargo +1.98.0 run --locked --offline -p vlajmaji-store --example owned_snapshot -- \
    "$run_dir/inputs" "$run_dir/journal" "$run_dir/imported"
  cargo +1.98.0 run --locked --offline -p vlajmaji-cli -- owned history "$run_dir/imported"
  cargo +1.98.0 run --locked --offline -p vlajmaji-cli -- owned show "$run_dir/imported" \
    --revision owned-r1
  cargo +1.98.0 run --locked --offline -p vlajmaji-cli -- owned check \
    --input "$run_dir/inputs/snapshot.json" --seal "$run_dir/inputs/seal.json"
)
