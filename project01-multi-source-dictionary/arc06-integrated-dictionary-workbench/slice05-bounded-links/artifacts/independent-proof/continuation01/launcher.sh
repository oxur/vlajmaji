#!/bin/bash
set -euo pipefail
source /private/tmp/vlajmaji-p01-a06-s01-original-env.sh
export CARGO_HOME=/Users/oubiwann/.cargo
export RUSTUP_HOME=/Users/oubiwann/.rustup
export RUSTUP_AUTO_INSTALL=0
export CARGO_TARGET_DIR=/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc01/target
export CARGO_NET_OFFLINE=true
RUN_RECEIPTS=/private/tmp/vlajmaji-s05-cdc-proof-method-aq8a7hgq
for leaf in launch02.stdout launch02.stderr launch02.status; do
  test ! -e "$RUN_RECEIPTS/$leaf"
done
(set -o noclobber; printf 'RUNNING\n' > "$RUN_RECEIPTS/launch02.status")
set +e
python3 /Users/oubiwann/lab/oxur/vlajmaji/.worktrees/planning/project01-multi-source-dictionary/arc06-integrated-dictionary-workbench/slice05-bounded-links/artifacts/independent-proof/continuation01/method.py /Users/oubiwann/lab/oxur/vlajmaji/.worktrees/planning/project01-multi-source-dictionary/arc06-integrated-dictionary-workbench/slice05-bounded-links/artifacts/independent-proof/continuation01/bindings.json > "$RUN_RECEIPTS/launch02.stdout" 2> "$RUN_RECEIPTS/launch02.stderr"
CHILD_STATUS=$?
set -e
printf 'EXIT %s\n' "$CHILD_STATUS" > "$RUN_RECEIPTS/launch02.status"
cat "$RUN_RECEIPTS/launch02.stdout"
cat "$RUN_RECEIPTS/launch02.stderr" >&2
exit "$CHILD_STATUS"
