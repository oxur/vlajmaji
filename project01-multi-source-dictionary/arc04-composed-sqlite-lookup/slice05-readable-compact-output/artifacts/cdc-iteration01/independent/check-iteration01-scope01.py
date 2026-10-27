"""Read-only S05 iteration01 frozen-boundary checker. Do not run during CC ownership."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys

SOURCE = Path("/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc04/vlajmaji")
BASE = "432d6321e3b2431e7fe2cc5f60e9317780e984b8"
BRANCH = "codex/project01-readable-compact-output"
OLD = "codex/project01-composed-sqlite-lookup"
ALLOWED = """
crates/search/src/dictionary.rs
crates/search/src/dictionary/borrowed.rs
crates/search/src/dictionary/projection.rs
crates/search/src/collection_query.rs
crates/search/src/collection_query/projection.rs
crates/search/src/lib.rs
crates/search/README.md
crates/search/tests/dictionary_projection_contract.rs
crates/search/tests/collection_projection_contract.rs
crates/store/src/collections.rs
crates/store/src/collections/reference_projection.rs
crates/store/src/lib.rs
crates/store/tests/reference_projection_contract.rs
crates/cli/src/main.rs
crates/cli/src/query_output.rs
crates/cli/tests/query_output_contract.rs
crates/cli/README.md
docs/collections.md
README.md
""".split()

def git(*args, stdin=None):
    result = subprocess.run(["git", "--no-optional-locks", "-C", str(SOURCE), *args],
                            input=stdin, capture_output=True, check=True)
    return result.stdout

def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

CORRECTION_BASE = "5bf5fb13f3d89f969570dff6aae5736feebd55fd"
CORRECTION_ALLOWED = """
crates/search/src/dictionary.rs
crates/search/src/dictionary/borrowed.rs
crates/search/src/dictionary/projection.rs
crates/search/src/collection_query.rs
crates/search/src/collection_query/projection.rs
crates/search/tests/dictionary_projection_contract.rs
crates/search/tests/collection_projection_contract.rs
crates/cli/src/main.rs
crates/cli/tests/query_output_contract.rs
""".split()

def main():
    expected, destination = sys.argv[1:]
    head = git("rev-parse", "HEAD").decode().strip()
    assert head == expected
    assert git("branch", "--show-current").decode().strip() == BRANCH
    assert git("status", "--porcelain=v2") == b""
    assert git("rev-parse", f"refs/heads/{OLD}").decode().strip() == BASE
    assert git("rev-parse", f"refs/remotes/origin/{OLD}").decode().strip() == BASE
    changed = git("diff", "--name-only", f"{BASE}..{head}").decode().splitlines()
    assert changed and len(ALLOWED) == 19 and set(changed) <= set(ALLOWED)
    assert git("diff", "--check", f"{BASE}..{head}") == b""
    git("merge-base", "--is-ancestor", CORRECTION_BASE, head)
    correction = git("diff", "--name-only", f"{CORRECTION_BASE}..{head}").decode().splitlines()
    assert correction and len(CORRECTION_ALLOWED)==9 and set(correction)<=set(CORRECTION_ALLOWED)
    assert git("diff", "--check", f"{CORRECTION_BASE}..{head}")==b""
    lock = sha(SOURCE / "Cargo.lock")
    guard = sha(Path("/tmp/vlajmaji-p01-a04-cargo"))
    assert lock == "65cd7f39a88bdd2a3dab1fcd14e9055ff7b77cb22ec3af8c309ce49dbea78700"
    assert guard == "20fe7ba73311aa503870923af09f33f3b53262742ca4eeea2972092924cf5de8"
    message = git("show", "-s", "--format=%B", head)
    parsed = git("interpret-trailers", "--parse", stdin=message).decode().splitlines()
    assert parsed == ["Co-authored-by: Codex <noreply@openai.com>",
                      "Co-authored-by: Billo AI <ai-engineering@billo.systems>"]
    result = {"scope_only": True, "ok": True, "correction_base": CORRECTION_BASE, "correction_allowed_count":9,"correction_changed_paths":correction, "head": head, "base": BASE,
              "tree": git("rev-parse", "HEAD^{tree}").decode().strip(),
              "branch": BRANCH, "allowed_count": len(ALLOWED),
              "changed_count": len(changed), "lock_sha256": lock,
              "guard_sha256": guard, "parsed_trailers": parsed,
              "changed_files": [{"path": path,
                                 "git_blob": git("rev-parse", f"HEAD:{path}").decode().strip(),
                                 "sha256": sha(SOURCE / path)} for path in changed],
              "limits": "No Cargo/target/database/default/network actions; scope is not functional acceptance"}
    with Path(destination).open("x") as handle:
        json.dump(result, handle, indent=2)
        handle.write("\n")
    print(json.dumps({k: v for k, v in result.items() if k != "changed_files"}))

if __name__ == "__main__":
    main()
