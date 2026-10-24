#!/usr/bin/env python3
"""Create or attach a safe local planning worktree without discarding history."""
from __future__ import annotations

import argparse
import os
from pathlib import Path
import subprocess
import sys


def git(root: Path, *args: str, check: bool = True, capture: bool = True, stdin: str | None = None) -> subprocess.CompletedProcess[str]:
    result = subprocess.run(
        ["git", "-C", str(root), *args], text=True, input=stdin,
        stdout=subprocess.PIPE if capture else None,
        stderr=subprocess.PIPE if capture else None,
    )
    if check and result.returncode:
        detail = (result.stderr or "").strip()
        raise RuntimeError(f"git {' '.join(args)} failed ({result.returncode})" + (f": {detail}" if detail else ""))
    return result


def gitdir(root: Path) -> Path:
    path = Path(git(root, "rev-parse", "--git-common-dir").stdout.strip())
    return (path if path.is_absolute() else root / path).resolve()


def worktrees(root: Path) -> list[dict[str, str]]:
    raw = git(root, "worktree", "list", "--porcelain", "-z").stdout
    items: list[dict[str, str]] = []
    for record in raw.split("\0"):
        if not record:
            continue
        if record.startswith("worktree "):
            items.append({"path": record[9:]})
        elif items:
            key, _, value = record.partition(" ")
            items[-1][key] = value
    return items


def tracked_paths(root: Path) -> list[Path]:
    names = git(root, "ls-files", "-z").stdout.split("\0")
    return [root / name for name in names if name]


def validate(root: Path, branch: str, remote: str, target: Path) -> Path:
    if not branch or branch.startswith("-"):
        raise RuntimeError("branch must be a non-empty Git branch name")
    git(root, "check-ref-format", "--branch", branch)
    if not remote or remote.startswith("-"):
        raise RuntimeError("remote must be a non-empty name")
    if git(root, "remote", "get-url", remote, check=False).returncode:
        raise RuntimeError(f"remote does not exist: {remote}")
    if target.is_symlink():
        raise RuntimeError(f"target path is a symlink: {target}")
    target = target.resolve(strict=False)
    if target == root:
        raise RuntimeError("planning worktree path may not be the implementation checkout")
    administration_paths = {root / ".git", gitdir(root)}
    for administration in administration_paths:
        administration = administration.resolve(strict=False)
        if target == administration or administration in target.parents or target in administration.parents:
            raise RuntimeError("planning worktree path overlaps Git administration data")
    for tracked in tracked_paths(root):
        resolved = tracked.resolve(strict=False)
        if target == resolved or resolved in target.parents or target in resolved.parents:
            raise RuntimeError("planning worktree path overlaps tracked implementation content")
    if target.exists() and not any(Path(item["path"]).resolve() == target for item in worktrees(root)):
        raise RuntimeError(f"target path already exists and is not a registered worktree: {target}")
    return target


def branch_worktree(root: Path, branch: str) -> dict[str, str] | None:
    for item in worktrees(root):
        if item.get("branch") == f"refs/heads/{branch}":
            return item
    return None


def clean(path: Path) -> bool:
    return not git(path, "status", "--porcelain", "--untracked-files=all").stdout


def ensure_clean(path: Path) -> None:
    if not clean(path):
        raise RuntimeError(f"planning worktree is dirty; preserve and clean it manually: {path}")


def ensure_registered(path: Path, root: Path, branch: str) -> None:
    items = worktrees(root)
    matching = [item for item in items if Path(item["path"]).resolve() == path]
    if not matching:
        raise RuntimeError(f"planning path is not registered as a Git worktree: {path}")
    item = matching[0]
    if item.get("locked") is not None or "prunable" in item:
        raise RuntimeError("planning worktree is locked or prunable")
    if item.get("branch") != f"refs/heads/{branch}":
        raise RuntimeError(f"worktree at target is on the wrong branch: {item.get('branch', 'detached')}")
    if gitdir(path) != gitdir(root):
        raise RuntimeError("target belongs to a different Git repository")
    ensure_clean(path)


def ancestry(root: Path, older: str, newer: str) -> bool:
    return git(root, "merge-base", "--is-ancestor", older, newer, check=False).returncode == 0


def run(args: argparse.Namespace) -> None:
    root = Path(git(Path.cwd(), "rev-parse", "--show-toplevel").stdout.strip()).resolve()
    target = Path(args.path).expanduser()
    if not target.is_absolute():
        target = root / target
    if target.is_symlink():
        raise RuntimeError(f"target path is a symlink: {target}")
    target = validate(root, args.branch, args.remote, target)

    local = git(root, "show-ref", "--verify", "--hash", f"refs/heads/{args.branch}", check=False).stdout.strip() or None
    existing = branch_worktree(root, args.branch)
    if existing and Path(existing["path"]).resolve() != target:
        raise RuntimeError(f"planning branch is already checked out elsewhere: {existing['path']}")

    ls = git(root, "ls-remote", "--exit-code", "--heads", args.remote, f"refs/heads/{args.branch}", check=False)
    if ls.returncode not in (0, 2):
        detail = (ls.stderr or "").strip()
        raise RuntimeError(f"could not query remote branch (exit {ls.returncode}); refusing to treat this as missing" + (f": {detail}" if detail else ""))
    remote_exists = ls.returncode == 0 and bool(ls.stdout.strip())
    remote_oid = ls.stdout.split()[0] if remote_exists else None

    if remote_exists:
        git(root, "fetch", args.remote, f"refs/heads/{args.branch}:refs/remotes/{args.remote}/{args.branch}")
        remote_oid = git(root, "rev-parse", f"refs/remotes/{args.remote}/{args.branch}").stdout.strip()

    if existing:
        ensure_registered(target, root, args.branch)
    elif target.exists():
        raise RuntimeError(f"target path is occupied: {target}")

    if remote_exists and not local:
        git(root, "branch", "--track", args.branch, f"{args.remote}/{args.branch}")
        git(root, "worktree", "add", str(target), args.branch)
        ensure_registered(target, root, args.branch)
        print(f"attached planning worktree at {target} from {args.remote}/{args.branch}")
        return

    if local and remote_exists:
        added_worktree = False
        if ancestry(root, local, remote_oid):
            if local != remote_oid:
                if existing:
                    git(target, "merge", "--ff-only", remote_oid)
                else:
                    git(root, "worktree", "add", str(target), args.branch)
                    added_worktree = True
                    git(target, "merge", "--ff-only", remote_oid)
                local = remote_oid
        elif not ancestry(root, remote_oid, local):
            raise RuntimeError("local and remote planning branches have diverged; reconcile manually")
        if not existing and not added_worktree:
            git(root, "worktree", "add", str(target), args.branch)
        ensure_registered(target, root, args.branch)
        print(f"planning worktree ready at {target} (local branch preserved)")
        return

    if local:
        if not existing:
            git(root, "worktree", "add", str(target), args.branch)
        ensure_registered(target, root, args.branch)
        git(root, "push", "-u", args.remote, f"refs/heads/{args.branch}:refs/heads/{args.branch}", capture=False)
        print(f"attached and pushed existing local planning branch at {target}")
        return

    # Neither branch exists. Add a detached linked worktree, then create a truly empty orphan.
    git(root, "worktree", "add", "--detach", str(target), "HEAD")
    git(target, "switch", "--orphan", args.branch)
    git(target, "commit", "--allow-empty", "-m", "Initialize planning worktree", capture=False)
    commit = git(target, "rev-parse", "HEAD").stdout.strip()
    parents = git(target, "rev-list", "--parents", "-n", "1", commit).stdout.split()
    tree = git(target, "rev-parse", f"{commit}^{{tree}}").stdout.strip()
    empty_tree = git(root, "hash-object", "-t", "tree", "--stdin", stdin="").stdout.strip()
    ensure_registered(target, root, args.branch)
    if len(parents) != 1 or tree != empty_tree:
        raise RuntimeError("new planning root is not a zero-parent, empty-tree commit; preserving worktree for inspection")
    git(root, "push", "-u", args.remote, f"refs/heads/{args.branch}:refs/heads/{args.branch}", capture=False)
    print(f"created empty planning root and pushed {args.remote}/{args.branch} at {target}")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--remote", default="origin")
    parser.add_argument("--branch", default="planning")
    parser.add_argument("--path", default=".worktrees/planning")
    try:
        run(parser.parse_args())
    except (RuntimeError, OSError) as exc:
        print(f"planning-worktree: {exc}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
