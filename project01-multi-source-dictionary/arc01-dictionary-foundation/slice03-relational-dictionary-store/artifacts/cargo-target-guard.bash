#!/usr/bin/env bash
# Task-owned execution guard; no source/Makefile changes.
set -euo pipefail
export CARGO_TARGET_DIR=/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc01/target
printf 'Vlajmaji Arc01 Cargo target: %s\n' "$CARGO_TARGET_DIR" >&2
exec cargo "$@"
