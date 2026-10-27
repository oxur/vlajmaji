#!/usr/bin/env python3
"""Bounded S04 canonical/row verifier; no product/DB/build/network action."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

parser = argparse.ArgumentParser()
parser.add_argument("mode", choices=("proposal", "mechanical"))
parser.add_argument("receipt", type=Path)
args = parser.parse_args()
slice_root = Path(__file__).resolve().parents[1]
arc_root = slice_root.parent
project_root = arc_root.parent
planning_root = project_root.parent
source_root = Path("/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc04/vlajmaji")
head = "432d6321e3b2431e7fe2cc5f60e9317780e984b8"
base = "dcdf216ea5805b20f374ce74c302e42bef12c158"

def git(root, *parts):
    return subprocess.check_output(["git", "--no-optional-locks", *parts], cwd=root, text=True)

def rows(text, prefix):
    result = {}
    for line in text.splitlines():
        if line.startswith("| " + prefix):
            fields = [part.strip() for part in line.split("|")[1:-1]]
            result[fields[0]] = fields
    return result

def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

assert git(source_root, "rev-parse", "HEAD").strip() == head
assert git(source_root, "status", "--porcelain") == ""
assert git(source_root, "branch", "--show-current").strip() == "codex/project01-composed-sqlite-lookup"
assert git(source_root, "diff", "--check", base, head) == ""
inv = json.loads((slice_root / "artifacts/scope-inventory01.json").read_text())
changed = git(source_root, "diff", "--name-only", base, head).splitlines()
assert changed == list(inv["source_paths"])
assert len(changed) == 20
for name, expected in inv["source_paths"].items():
    path = source_root / name
    assert path.stat().st_size == expected["bytes"] and digest(path) == expected["sha256"], name

current = rows((slice_root / "ledger.md").read_text(), "A04-S04-")
rel = str((slice_root / "ledger.md").relative_to(planning_root))
original = rows(git(planning_root, "show", "dac75f9:" + rel), "A04-S04-")
assert current.keys() == original.keys() and len(current) == 6
for key in current:
    assert current[key][:4] == original[key][:4], key
    assert current[key][4] == ("done" if args.mode == "mechanical" else "open"), key
arc = rows((arc_root / "ledger.md").read_text(), "A04-")
assert len(arc) == 10
expected_done = {"A04-01", "A04-02", "A04-04"}
if args.mode == "mechanical":
    expected_done |= {"A04-03", "A04-05"}
assert {key for key, value in arc.items() if value[4] == "done"} == expected_done
assert all(value[4] == ("done" if key in expected_done else "open") for key, value in arc.items())
project = rows((project_root / "ledger.md").read_text(), "P01-")
assert len(project) == 14 and all(value[5] == "open" for value in project.values())
for path, prefix in [(arc_root / "ledger.md", "A04-"), (project_root / "ledger.md", "P01-")]:
    relative = str(path.relative_to(planning_root))
    old = rows(git(planning_root, "show", "b436e2c:" + relative), prefix)
    new = rows(path.read_text(), prefix)
    assert old.keys() == new.keys()
    assert all(old[key][:(6 if prefix == "P01-" else 4)] == new[key][:(6 if prefix == "P01-" else 4)] for key in old)

payload_count = 0
for item in inv["immutable_manifests"]:
    manifest = slice_root / item["path"]
    assert digest(manifest) == item["sha256"]
    records = json.loads(manifest.read_text())["payloads"]
    actual = {str(f.relative_to(manifest.parent)) for f in manifest.parent.rglob("*") if f.is_file() and f != manifest}
    assert actual == {row["path"] for row in records}
    for row in records:
        file = manifest.parent / row["path"]
        assert file.stat().st_size == row["bytes"] and digest(file) == row["sha256"]
    payload_count += len(records)
assert payload_count == 780 and len(inv["immutable_manifests"]) == 10
output = args.receipt.resolve()
actual = {str(f.relative_to(slice_root)) for f in (slice_root / "artifacts").rglob("*") if f.is_file() and f.resolve() != output}
expected = set(inv["artifact_paths"])
try:
    expected.discard(str(output.relative_to(slice_root)))
except ValueError:
    pass
assert actual == expected, (actual - expected, expected - actual)
for name in inv["first_class_documents"]:
    assert (slice_root / name).is_file(), name
for name in ["closing-report.md", "cdc-verification.md"]:
    body = (slice_root / name).read_text()
    assert all(key in body for key in current)
assert "ACCEPTED" in (slice_root / "cdc-verification.md").read_text()
receipt = {"version": 1, "mode": args.mode, "ok": True, "source": head,
           "source_paths": len(changed), "unchanged_contracts": True,
           "child_rows": 6, "child_done": 6 if args.mode == "mechanical" else 0,
           "arc_done": len(expected_done), "arc_open": 10-len(expected_done),
           "project_open": 14, "immutable_manifests": 10,
           "immutable_payloads_verified": payload_count,
           "artifact_paths_verified_excluding_output_receipt": len(actual),
           "runtime_reexecution": False, "full_operations": False}
with args.receipt.open("x") as handle:
    json.dump(receipt, handle, indent=2)
    handle.write("\n")
print(json.dumps(receipt, sort_keys=True))
