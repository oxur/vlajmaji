# Slice02 replay recipe

Run local candidate checks from the source repository root. Rust 1.98.0 is the declared MSRV and is used explicitly for the refinement replay:

```sh
cargo +1.98.0 fmt --all --check
cargo +1.98.0 check --workspace --all-targets --locked
cargo +1.98.0 clippy --workspace --all-targets --locked -- -D warnings
cargo +1.98.0 test --workspace --locked
RUSTDOCFLAGS=-Dwarnings cargo +1.98.0 doc --workspace --no-deps --locked
RUN_DIR="$(mktemp -d "${TMPDIR:-/tmp}/p02a02-slice02-corpus-replay.XXXXXX")"
cargo +1.98.0 run --locked --example import_corpus -- "$PWD" "$RUN_DIR/slice02-corpus-example.archive" > "$RUN_DIR/corpus-example.log"
```

The example reads all 16 entries in `corpus-inputs.json`, verifies each raw input SHA-256, imports the full E02 bytes, builds and reloads the versioned archive, and prints the derived summary. The summary distinguishes 19,696 definition-bearing source rows from 1,545 forms-only assigned-rafsi gloss rows; it does not count unique meanings or senses. The replay archive and log go in the fresh temporary directory; captured inputs and recorded probe logs remain untouched. `derived/import-summary.json` is the preserved summary with its archive path made repository-relative.

Replay the external public-API consumer from a separate temporary directory (it has its own `[workspace]` root and uses no private types or serde):

```sh
ROOT="$PWD"
TMP="$(mktemp -d "${TMPDIR:-/tmp}/p02a02-slice02-external-consumer.XXXXXX")"
mkdir -p "$TMP/consumer"
cp -R experiments/E03-lexical-data-search/slice02/probes/external-consumer/src "$TMP/consumer/"
cp experiments/E03-lexical-data-search/slice02/probes/external-consumer/Cargo.lock "$TMP/consumer/"
cp experiments/E03-lexical-data-search/slice02/probes/external-consumer/Cargo.toml "$TMP/consumer/"
python3 - "$TMP/consumer/Cargo.toml" "$ROOT/crates/relojban-lexicon" <<'PY'
from pathlib import Path
import sys
manifest = Path(sys.argv[1])
text = manifest.read_text()
text = text.replace("../../../../../crates/relojban-lexicon", sys.argv[2])
manifest.write_text(text)
PY
CARGO_TARGET_DIR="$TMP/target" cargo +1.98.0 run --locked --manifest-path "$TMP/consumer/Cargo.toml" -- "$ROOT" "$TMP/slice02.archive"
```

The consumer imports the same 16 inputs and checks record/snapshot counts, byte-equality of all 16 raw sources after archive reload, exact `farvi` across all languages, the 13 Lensisku `farvi` rows, `si'e`/`sie`, proposals, form/presence access, idempotence and archive reload. Its lockfile is retained beside its source.

For the optional upstream probe, reconstruct jbotci from the recorded public commit and replay the saved public consumer. The captured source pin is `3d9fd41d9ed32de26311ee27ec86b6b165985aa1`; the saved lockfiles and fixture JSON are under `probes/jbotci/`. Use ordinary TLS dependency resolution if a new Cargo cache is needed; sandbox DNS denials are not source/build results.

```sh
ROOT="$PWD"
TMP="$(mktemp -d "${TMPDIR:-/tmp}/p02a02-slice02-jbotci.XXXXXX")"
JBOTCI="$TMP/jbotci"
git clone --no-checkout https://github.com/int19h/jbotci.git "$JBOTCI"
git -C "$JBOTCI" checkout --detach 3d9fd41d9ed32de26311ee27ec86b6b165985aa1
cp experiments/E03-lexical-data-search/slice02/probes/jbotci/Cargo.lock "$JBOTCI/Cargo.lock"
cargo +1.98.0 build --locked --manifest-path "$JBOTCI/Cargo.toml" -p jbotci-dictionary --features import --target-dir "$TMP/jbotci-target"
cp -R experiments/E03-lexical-data-search/slice02/probes/jbotci/consumer "$TMP/consumer"
python3 - "$TMP/consumer/Cargo.toml" "$JBOTCI/crates/jbotci-dictionary" <<'PY'
from pathlib import Path
import sys
manifest = Path(sys.argv[1])
text = manifest.read_text()
text = text.replace("/tmp/p02a02-jbotci-cc-s02-20261023/crates/jbotci-dictionary", sys.argv[2])
manifest.write_text(text)
PY
CARGO_TARGET_DIR="$TMP/consumer-target" cargo +1.98.0 run --locked --manifest-path "$TMP/consumer/Cargo.toml" -- \
  "$ROOT/experiments/E02-resource-discovery/raw/lexical/la-lojban-en.json" \
  "$ROOT/experiments/E02-resource-discovery/raw/lexical/lensisku-farvi.json" \
  "$ROOT/experiments/E03-lexical-data-search/slice02/probes/jbotci/farvi-multiple.json" \
  "$ROOT/experiments/E03-lexical-data-search/slice02/probes/jbotci/farvi-missing.json" \
  "$ROOT/experiments/E03-lexical-data-search/slice02/probes/jbotci/farvi-null.json" \
  "$ROOT/experiments/E03-lexical-data-search/slice02/probes/jbotci/farvi-empty.json" \
  "$ROOT/experiments/E03-lexical-data-search/slice02/probes/jbotci/farvi-unknown-kind.json" \
  "$ROOT/experiments/E03-lexical-data-search/slice02/probes/jbotci/farvi-extra-field.json"
```

The recorded runtime result is `probes/jbotci/runtime-corrected.log`; `runtime.log` remains the original wrong-argument attempt. `attempts.md` records the correction sequence and distinguishes sandbox-only DNS/cache restrictions from actual package/runtime outcomes. The refinement02 CC replay did not rebuild or alter jbotci source, lockfiles, fixtures, or raw captures.
