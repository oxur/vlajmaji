#!/usr/bin/env bash
# Real-Git fixtures use local bare remotes; Cargo is replaced with a stub.
# No network, Python, Bats, or Rust compiler is needed. Bash 3.2 compatible.
set -euo pipefail
repo=$(cd -- "$(dirname -- "$0")/.." && pwd -P)
make_command=${MAKE:-make}
binary=$(awk '$1 == "BINARIES" && $2 == ":=" { print $3; exit }' "$repo/Makefile")
case $binary in
    bauske) package=bauske-phonology; lean='test --workspace --no-default-features --locked' ;;
    vlajmaji) package=vlajmaji-search; lean='test --locked -p vlajmaji-search --no-default-features' ;;
    *) printf 'Unrecognized BINARIES in Makefile: %s\n' "$binary" >&2; exit 1 ;;
esac
run_root=$(mktemp -d "${TMPDIR:-/tmp}/build-tools.XXXXXX")
# Isolate fixtures from user Git configuration, hooks, signing and environment.
export GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null GIT_TERMINAL_PROMPT=0
unset GIT_DIR GIT_WORK_TREE GIT_INDEX_FILE GIT_COMMON_DIR GIT_OBJECT_DIRECTORY GIT_ALTERNATE_OBJECT_DIRECTORIES
unset GIT_CONFIG_COUNT GIT_CONFIG_PARAMETERS GIT_CONFIG GIT_NAMESPACE
unset GIT_AUTHOR_NAME GIT_AUTHOR_EMAIL GIT_COMMITTER_NAME GIT_COMMITTER_EMAIL
unset MAKEFLAGS MFLAGS MAKEOVERRIDES
export LC_ALL=C

fail() { printf 'FAIL: %s\n' "$*" >&2; exit 1; }
equal() { [[ $1 == "$2" ]] || fail "expected [$2], got [$1]"; }
contains() { grep -F -- "$2" "$1" >/dev/null || fail "$1 does not contain: $2"; }
excludes() { if grep -F -- "$2" "$1" >/dev/null; then fail "$1 unexpectedly contains: $2"; fi; }
# A failed command is expected only here; assertions themselves always exit.
reject() {
    local code
    if "$@" > "$case_root/rejected.log" 2>&1; then fail "unexpected success: $*"; else code=$?; fi
    [[ $code -ne 0 ]] || fail 'failure was not propagated'
}
g() { git -C "$source" "$@"; }
w() { git -C "$work" "$@"; }
plan() { "$make_command" -C "$source" --no-print-directory "BASH=$BASH" planning; }
identity() {
    git -C "$1" config user.name 'Fixture User'
    git -C "$1" config user.email fixture@example.invalid
    git -C "$1" config commit.gpgsign false
}
fixture() {
    local base=${1:-$case_root}
    mkdir -p "$base"
    source=$base/'source checkout with spaces'
    origin=$base/origin.git
    work=$source/.worktrees/planning
    mkdir -p "$source"
    git init -b main "$source"
    identity "$source"
    printf 'source base\n' > "$source/tracked.txt"
    g add tracked.txt
    g commit -m 'source base'
    git init --bare "$origin"
    git --git-dir="$origin" symbolic-ref HEAD refs/heads/main
    g remote add origin "$origin"
    g push -u origin main
    cp "$repo/Makefile" "$repo/.gitignore" "$source/"
    mkdir "$source/scripts"
    cp "$repo/scripts/planning-worktree.sh" "$source/scripts/"
    g add Makefile .gitignore scripts/planning-worktree.sh
    g commit -m 'fixture build tooling'
    cp "$source/.git/index" "$base/index-before"
    head_before=$(g rev-parse HEAD)
    index_before=$base/index-before
}
unchanged() {
    equal "$(g rev-parse HEAD)" "$head_before"
    # Compare before status, which may refresh the index's stat cache.
    cmp "$source/.git/index" "$index_before" || fail 'implementation index changed'
    equal "$(g status --porcelain)" ''
}
remote_head() { git --git-dir="$origin" rev-parse refs/heads/planning; }
empty_root() {
    local commit
    commit=$(w rev-parse HEAD)
    equal "$(w rev-list --parents -n 1 "$commit")" "$commit"
    equal "$(w ls-tree -r --name-only HEAD)" ''
}
clone_planning() {
    clone=$case_root/'remote update'
    git clone "$origin" "$clone"
    identity "$clone"
    git -C "$clone" switch planning
}
remote_update() {
    clone_planning
    printf 'remote advanced\n' > "$clone/later.txt"
    git -C "$clone" add later.txt
    git -C "$clone" commit -m 'remote update'
    git -C "$clone" push origin planning
}
local_update() {
    printf 'local\n' > "$work/local.txt"
    w add local.txt
    w commit -m 'local ahead'
}

make_targets_forward_toolchain_and_failures() {
    export CARGO_LOG=$case_root/cargo.log CARGO_TARGET_DIR=$case_root/'cargo artifacts' TEST_BINARY=$binary
    local fake=$case_root/fake-cargo target fragment expected before
    cat > "$fake" <<'STUB'
#!/bin/sh
set -eu
printf '%s\n' "$*" >> "$CARGO_LOG"
[ "${FAKE_FAIL:-}" != 1 ] || exit 23
case " $* " in
    *' build '*)
        mode=debug
        case " $* " in *' --release '*) mode=release ;; esac
        mkdir -p "$CARGO_TARGET_DIR/$mode"
        printf '%s\n' "$mode" > "$CARGO_TARGET_DIR/$mode/$TEST_BINARY" ;;
esac
STUB
    chmod +x "$fake"
    for target in build build-release test-default test-no-default test-all-features lint docs; do
        case $target in
            build) fragment='build --workspace --locked' ;;
            build-release) fragment='build --workspace --release --locked' ;;
            test-default) fragment='test --workspace --locked' ;;
            test-no-default) fragment=$lean ;;
            test-all-features) fragment='test --workspace --all-features --locked' ;;
            lint) fragment='clippy --workspace --all-targets --locked -- -D warnings' ;;
            docs) fragment='doc --workspace --locked --no-deps' ;;
        esac
        "$make_command" -C "$repo" --no-print-directory "CARGO=$fake" RUST_TOOLCHAIN=1.98.0 "INSTALL_DIR=$case_root/installed tools" "$target"
        contains "$CARGO_LOG" "+1.98.0 $fragment"
        case $target in
            build) equal "$(cat "$case_root/installed tools/$binary")" debug ;;
            build-release) equal "$(cat "$case_root/installed tools/$binary")" release ;;
        esac
    done
    before=$(wc -l < "$CARGO_LOG")
    reject "$make_command" -C "$repo" --no-print-directory "CARGO=$fake" publish-one
    equal "$(wc -l < "$CARGO_LOG")" "$before"
    for target in publish publish-dry-run publish-one; do
        case $target in
            publish) expected='publish --workspace --registry crates-io --locked' ;;
            publish-dry-run) expected='publish --workspace --registry crates-io --dry-run --locked' ;;
            publish-one) expected="publish -p $package --registry crates-io --locked" ;;
        esac
        "$make_command" -C "$repo" --no-print-directory "CARGO=$fake" RUST_TOOLCHAIN=1.98.0 "CRATE=$package" "$target"
        contains "$CARGO_LOG" "+1.98.0 $expected"
    done
    export FAKE_FAIL=1
    reject "$make_command" -C "$repo" --no-print-directory "CARGO=$fake" "INSTALL_DIR=$case_root/installed tools" build
    equal "$(cat "$case_root/installed tools/$binary")" release
    excludes "$case_root/rejected.log" 'Build complete'
}

install_copies_selected_mode_and_reports_missing_artifacts() {
    local mode target=$case_root/'cargo artifacts'
    cp "$repo/Makefile" "$case_root/"
    for mode in debug release; do
        mkdir -p "$target/$mode"
        printf '#!/bin/sh\nprintf "%%s\\n" %s\n' "$mode" > "$target/$mode/$binary"
        chmod +x "$target/$mode/$binary"
    done
    "$make_command" -C "$case_root" --no-print-directory "CARGO_TARGET_DIR=$target" install
    equal "$("$case_root/bin/$binary")" debug
    cmp "$case_root/bin/$binary" "$target/debug/$binary"
    "$make_command" -C "$case_root" --no-print-directory "CARGO_TARGET_DIR=$target" "INSTALL_DIR=$case_root/custom install" MODE=release install
    equal "$("$case_root/custom install/$binary")" release
    cmp "$case_root/custom install/$binary" "$target/release/$binary"
    rm "$target/debug/$binary"
    reject "$make_command" -C "$case_root" --no-print-directory "CARGO_TARGET_DIR=$target" install
    excludes "$case_root/rejected.log" 'Binaries available'
    reject "$make_command" -C "$case_root" --no-print-directory MODE=invalid build
}

help_does_not_require_build_tools() {
    "$make_command" -C "$repo" --no-print-directory CARGO=/absent/cargo BASH=/absent/bash GIT=/absent/git help > "$case_root/help.log"
    contains "$case_root/help.log" planning
    contains "$case_root/help.log" publish-dry-run
}

create_empty_root_push_idempotence_and_preserve_source() {
    fixture
    plan > "$case_root/first.log"
    local commit
    commit=$(w rev-parse HEAD)
    empty_root
    equal "$(remote_head)" "$commit"
    contains "$case_root/first.log" 'created empty planning root'
    plan
    equal "$(w rev-parse HEAD)" "$commit"
    unchanged
}
remote_branch_without_local_branch_attaches() {
    fixture
    clone=$case_root/seed
    git clone "$origin" "$clone"
    identity "$clone"
    git -C "$clone" switch --orphan planning
    git -C "$clone" commit --allow-empty -m 'remote plan'
    git -C "$clone" push origin planning
    plan
    equal "$(w branch --show-current)" planning
    equal "$(w rev-parse HEAD)" "$(remote_head)"
    unchanged
}
remote_update_fast_forwards_clean_planning_worktree() {
    fixture; plan; remote_update; plan
    equal "$(cat "$work/later.txt")" 'remote advanced'
    equal "$(w rev-parse HEAD)" "$(remote_head)"
    unchanged
}
local_ahead_preserved_without_push() {
    fixture; plan
    local remote before
    remote=$(remote_head)
    local_update
    before=$(w rev-parse HEAD)
    plan
    equal "$(w rev-parse HEAD)" "$before"
    equal "$(remote_head)" "$remote"
    unchanged
}
existing_local_branch_without_remote_is_attached_and_pushed() {
    fixture
    g branch planning HEAD
    local expected
    expected=$(g rev-parse refs/heads/planning)
    plan
    equal "$(w rev-parse HEAD)" "$expected"
    equal "$(remote_head)" "$expected"
    unchanged
}
dirty_planning_refused_without_data_loss() {
    fixture; plan
    printf 'preserve me' > "$work/untracked"
    reject plan
    contains "$case_root/rejected.log" 'is dirty'
    equal "$(cat "$work/untracked")" 'preserve me'
    unchanged
}
true_divergence_refused() {
    fixture; plan; remote_update; local_update
    local before remote
    before=$(w rev-parse HEAD)
    remote=$(remote_head)
    reject plan
    contains "$case_root/rejected.log" diverged
    equal "$(w rev-parse HEAD)" "$before"
    equal "$(remote_head)" "$remote"
    unchanged
}
path_collision_and_wrong_branch_refused() {
    fixture "$case_root/collision"
    mkdir -p "$work"
    printf keep > "$work/keep"
    reject plan
    equal "$(cat "$work/keep")" keep
    unchanged
    fixture "$case_root/wrong-branch"
    g worktree add -b other "$work" HEAD
    reject plan
    equal "$(w branch --show-current)" other
    [[ -f $work/tracked.txt ]] || fail 'wrong-branch worktree damaged'
    unchanged
}
unreachable_origin_is_not_treated_as_missing_branch() {
    fixture
    g remote set-url origin "$case_root/absent.git"
    reject plan
    contains "$case_root/rejected.log" 'could not query remote branch'
    [[ ! -e $work ]] || fail 'created worktree after remote error'
    reject g show-ref --verify --quiet refs/heads/planning
    unchanged
}
implementation_dirty_state_is_preserved() {
    fixture
    printf 'staged change\n' > "$source/tracked.txt"
    g add tracked.txt
    cp "$source/.git/index" "$case_root/staged-index"
    printf 'staged plus unstaged\n' > "$source/tracked.txt"
    printf 'preserve\n' > "$source/untracked.txt"
    plan
    equal "$(g rev-parse HEAD)" "$head_before"
    cmp "$source/.git/index" "$case_root/staged-index"
    equal "$(cat "$source/tracked.txt")" 'staged plus unstaged'
    equal "$(cat "$source/untracked.txt")" preserve
}
branch_checked_out_elsewhere_is_refused() {
    fixture
    g branch planning HEAD
    g worktree add "$case_root/already attached" planning
    reject plan
    contains "$case_root/rejected.log" 'already checked out elsewhere'
    [[ -f $case_root/'already attached'/tracked.txt && ! -e $work ]] || fail 'unexpected worktree change'
    unchanged
}
push_rejection_preserves_empty_local_root_and_worktree() {
    fixture
    printf '#!/bin/sh\necho fixture push rejection >&2\nexit 1\n' > "$origin/hooks/pre-receive"
    chmod +x "$origin/hooks/pre-receive"
    reject plan
    contains "$case_root/rejected.log" 'fixture push rejection'
    empty_root
    reject git --git-dir="$origin" show-ref --verify --quiet refs/heads/planning
    unchanged
}
symlink_path_is_refused() {
    fixture
    mkdir -p "$case_root/outside" "$source/.worktrees"
    ln -s "$case_root/outside" "$work"
    reject plan
    contains "$case_root/rejected.log" symlink
    [[ ! -e $case_root/outside/.git ]] || fail 'symlink destination was changed'
    rm "$work"
    plan
    local commit suffix
    commit=$(w rev-parse HEAD)
    ln -s .worktrees/planning "$source/planning-alias"
    cd "$source"
    for suffix in / /.; do
        reject "$BASH" scripts/planning-worktree.sh --path "planning-alias$suffix"
        contains "$case_root/rejected.log" symlink
        equal "$(w rev-parse HEAD)" "$commit"
    done
    rm "$source/planning-alias"
    unchanged
}
local_branch_behind_remote_without_worktree_fast_forwards_once() {
    fixture; plan
    local old
    old=$(w rev-parse HEAD)
    g worktree remove "$work"
    remote_update
    equal "$(g rev-parse refs/heads/planning)" "$old"
    plan
    equal "$(cat "$work/later.txt")" 'remote advanced'
    equal "$(w rev-parse HEAD)" "$(remote_head)"
    unchanged
}
helper_can_be_invoked_from_repository_subdirectory() {
    fixture
    mkdir "$source/nested"
    (cd "$source/nested" && "$BASH" ../scripts/planning-worktree.sh --path .worktrees/planning) > "$case_root/direct.log"
    contains "$case_root/direct.log" 'created empty planning root'
    equal "$(w branch --show-current)" planning
    unchanged
}
git_administration_path_is_refused() {
    fixture
    cd "$source"
    reject "$BASH" scripts/planning-worktree.sh --path .git/planning
    contains "$case_root/rejected.log" 'Git administration'
    [[ ! -e $source/.git/planning ]] || fail 'Git administration data was changed'
    ln -s .git "$source/admin-alias"
    g worktree list --porcelain -z > "$case_root/worktrees-before"
    reject "$BASH" scripts/planning-worktree.sh --path missing/../admin-alias/review-planning
    contains "$case_root/rejected.log" 'Git administration'
    if [[ -d $source/.GIT ]]; then
        reject "$BASH" scripts/planning-worktree.sh --path .GIT/casefold-planning
        contains "$case_root/rejected.log" 'Git administration'
        [[ ! -e $source/.git/casefold-planning ]] || fail 'case alias bypassed Git directory guard'
    fi
    [[ ! -e $source/.git/review-planning ]] || fail 'created a worktree in Git administration data'
    g worktree list --porcelain -z > "$case_root/worktrees-after"
    cmp "$case_root/worktrees-before" "$case_root/worktrees-after"
    reject g show-ref --verify --quiet refs/heads/planning
    reject git --git-dir="$origin" show-ref --verify --quiet refs/heads/planning
    rm "$source/admin-alias"
    unchanged
}

cases=(
    make_targets_forward_toolchain_and_failures
    install_copies_selected_mode_and_reports_missing_artifacts
    help_does_not_require_build_tools
    create_empty_root_push_idempotence_and_preserve_source
    remote_branch_without_local_branch_attaches
    remote_update_fast_forwards_clean_planning_worktree
    local_ahead_preserved_without_push
    existing_local_branch_without_remote_is_attached_and_pushed
    dirty_planning_refused_without_data_loss
    true_divergence_refused
    path_collision_and_wrong_branch_refused
    unreachable_origin_is_not_treated_as_missing_branch
    implementation_dirty_state_is_preserved
    branch_checked_out_elsewhere_is_refused
    push_rejection_preserves_empty_local_root_and_worktree
    symlink_path_is_refused
    local_branch_behind_remote_without_worktree_fast_forwards_once
    helper_can_be_invoked_from_repository_subdirectory
    git_administration_path_is_refused
)
failed=0
passed=0
for test_case in "${cases[@]}"; do
    case_root=$run_root/$test_case
    mkdir "$case_root"
    # Do not put the function in an if/|| condition: that disables errexit in
    # the whole function. Only the parent disables it to collect the exit code.
    set +e
    (set -euo pipefail; cd "$case_root"; "$test_case") > "$case_root/output.log" 2>&1
    status=$?
    set -e
    if [[ $status -eq 0 ]]; then
        printf 'ok - %s\n' "$test_case"
        passed=$((passed + 1))
    else
        printf 'not ok - %s (exit %s)\n' "$test_case" "$status" >&2
        cat "$case_root/output.log" >&2
        failed=$((failed + 1))
    fi
done
printf '%s passed; %s failed (%s total)\n' "$passed" "$failed" "${#cases[@]}"
if [[ $failed -gt 0 ]]; then
    printf 'Fixtures and logs retained at %s\n' "$run_root" >&2
    exit 1
fi
rm -rf -- "$run_root"
