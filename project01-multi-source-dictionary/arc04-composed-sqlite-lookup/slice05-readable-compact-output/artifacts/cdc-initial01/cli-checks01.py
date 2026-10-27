"""Prospective S05 Tiny CLI proof. SOURCE ONLY until frozen caller review.

Inputs are explicit current/preserved binaries and an owned Tiny caller root.
This is not a Full entrypoint. No ambient HOME/default fallback is permitted.
"""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import stat
import sys
import time

MANAGED = set("version ok category detail operation config_path root_path profile requested_profile snapshot known_pack_ids outcome writer".split())
STANDALONE = {"code", "message", "sources"}
COMPACT_EXACT = set("version projection_policy policy store_schema catalogue_revision state_sha256 snapshot_sha256 spelling resolved_sources native_candidate_occurrences totals source_totals availability groups next_cursor code_notices".split())
COMPACT_HIT = set("occurrence_id entry assertion capture snapshot_attribution occurrence_attribution contributor distributor release_ids detail_ref".split())
COMPACT_AVAILABILITY = set("source aliases lineage collections components releases selected_available_release_ids loaded_captures missing_captures owned_head selectable reason".split())
DETAIL = set("version projection_policy detail_ref status operational_reason state_sha256 snapshot_sha256 hit current_eligible".split())
COMPACT_RESOLVE = set("version projection_policy state_sha256 snapshot_sha256 target status entity confirmed unverified availability requested_release".split())

def compact_resolve_wire(obj):
    assert isinstance(obj, dict) and set(obj) == COMPACT_RESOLVE
    assert type(obj["version"]) is int and obj["version"] == 1
    assert obj["projection_policy"] == "compact-dictionary-v1"
    for row in [*obj["confirmed"], *obj["unverified"]]:
        assert "fields" not in row and "raw" not in row
    for source in obj["availability"]:
        assert set(source) == COMPACT_AVAILABILITY
        for release in source["releases"]:
            assert "assertions" not in release
    return obj

def compact_exact_wire(obj):
    assert isinstance(obj, dict) and set(obj) == COMPACT_EXACT
    assert type(obj["version"]) is int and obj["version"] == 1
    assert obj["projection_policy"] == "compact-dictionary-v1"
    for group in obj["groups"]:
        assert set(group) == {"source", "full_count", "returned_count", "partial", "hits"}
        for hit in group["hits"]:
            assert set(hit) == COMPACT_HIT
    for source in obj["availability"]:
        assert set(source) == COMPACT_AVAILABILITY
        for release in source["releases"]:
            assert "assertions" not in release
    return obj

def detail_wire(obj):
    assert isinstance(obj, dict) and set(obj) == DETAIL
    assert type(obj["version"]) is int and obj["version"] == 1
    assert obj["projection_policy"] == "complete-dictionary-hit-v1"
    return obj

def strict_json(raw):
    # Reject trailing material, including warnings. Whitespace/LF alone is framing.
    text = raw.decode("utf-8")
    obj, end = json.JSONDecoder().raw_decode(text)
    assert not text[end:].strip(), text[end:]
    return obj

def run(binary, args, env, out, name, expected=0, stdin=None, cwd=None, preexec_fn=None):
    started = time.monotonic()
    argv = [str(binary), *map(str, args)]
    result = subprocess.run(argv, input=stdin,
                            capture_output=True, env=env, timeout=60,
                            cwd=cwd, preexec_fn=preexec_fn)
    for suffix, data in [("stdout", result.stdout), ("stderr", result.stderr)]:
        with (out / f"{name}.{suffix}").open("xb") as handle:
            handle.write(data)
    receipt = {"argv": argv, "exit": result.returncode, "expected_exit": expected,
               "elapsed_s": time.monotonic()-started,
               "stdout_bytes": len(result.stdout), "stdout_sha256": hashlib.sha256(result.stdout).hexdigest(),
               "stderr_bytes": len(result.stderr), "stderr_sha256": hashlib.sha256(result.stderr).hexdigest(),
               "cwd": None if cwd is None else str(cwd),
               "isolated_environment": {k:env[k] for k in ("HOME","XDG_CONFIG_HOME","XDG_DATA_HOME")}}
    with (out / f"{name}.receipt.json").open("x") as handle:
        json.dump(receipt, handle, indent=2); handle.write("\n")
    assert result.returncode == expected, (name, result.returncode, result.stderr)
    return result

def error(result, managed, category, operation=None):
    assert result.stdout == b""
    wire = strict_json(result.stderr)
    assert isinstance(wire, dict) and set(wire) == {"error"}
    obj = wire["error"]  # Existing framing; 3/13 keys describe the INNER report.
    assert isinstance(obj, dict)
    if managed:
        assert set(obj) == MANAGED
        assert type(obj["version"]) is int and obj["version"] == 1 and obj["ok"] is False
        assert isinstance(obj["detail"], str) and obj["detail"]
        assert obj["category"] == category
        assert obj["operation"] == operation
        assert obj["outcome"] == "not_applicable"
        for field in ["config_path", "root_path", "profile", "requested_profile", "snapshot", "writer"]:
            assert obj[field] is None, (field, obj[field])
        assert obj["known_pack_ids"] == []
    else:
        assert set(obj) == STANDALONE and obj["code"] == category
        assert isinstance(obj["message"], str) and obj["message"]
        assert obj["sources"] == []
    return obj

def owned_store(context_file, context):
    assert context["fixture_scope"] == "original-four-pack-synthetic"
    store, root = Path(context["store"]), Path(context["root"])
    assert store.is_absolute() and root.is_absolute()
    assert context_file.parent == root.parent
    assert ".." not in root.parts and ".." not in store.parts
    relative = store.relative_to(root)  # Reject escape BEFORE stat/read of receipt path.
    assert relative.parts and stat.S_ISDIR(root.lstat().st_mode)
    owned = root
    for part in relative.parts:
        owned = owned / part
        metadata = owned.lstat()
        assert not stat.S_ISLNK(metadata.st_mode) and stat.S_ISDIR(metadata.st_mode)
    return store, root

DIAGNOSTIC_FAILURES = []
def check_error(result, managed, category, operation=None):
    try:
        return error(result, managed, category, operation)
    except AssertionError as exc:
        actual = result.stderr.decode("utf-8", "replace")
        DIAGNOSTIC_FAILURES.append({"expected_category":category,"managed":managed,"actual_stderr":actual})
        print("FAIL declared error contract " + json.dumps(DIAGNOSTIC_FAILURES[-1]), flush=True)
        return None

def main():
    binary, baseline, context_file, out = map(Path, sys.argv[1:])
    assert all(path.is_absolute() for path in (binary, baseline, context_file, out))
    out.mkdir()  # Exclusive; preserve every failed attempt, never pre-delete.
    context = json.loads(context_file.read_text())
    store, root = owned_store(context_file, context)
    env = {"PATH": os.defpath, "LANG": "C.UTF-8"}
    for name in ["HOME", "XDG_CONFIG_HOME", "XDG_DATA_HOME"]:
        owned = out / name.lower()
        owned.mkdir(mode=0o700)
        env[name] = str(owned.resolve())
    config = out.resolve() / "config.toml"
    request = json.dumps({"operation": "exact", "spelling": "farvi"})
    old = run(baseline, ["dictionary-query", store, "--request", request], env, out, "baseline")
    current = run(binary, ["dictionary-query", store, "--request", request], env, out, "default-full")
    explicit = run(binary, ["dictionary-query", store, "--format", "full", "--request", request], env, out, "explicit-full")
    assert old.stderr == current.stderr == explicit.stderr == b""
    assert old.stdout == current.stdout == explicit.stdout
    projected = run(binary, ["dictionary-query", store, "--format", "compact", "--request", request], env, out, "compact")
    compact = compact_exact_wire(strict_json(projected.stdout))
    assert "response" not in compact and "result" not in compact
    assert compact["totals"]["occurrences"] == 5
    assert len([h for group in compact["groups"] for h in group["hits"]]) == 5
    configured = run(binary, ["collections", "configure", root, "--all", "--profile", "tiny", "--make-default", "--config", config], env, out, "configure")
    assert strict_json(configured.stdout)["outcome"] == "committed"
    managed = run(binary, ["lookup", "farvi", "--format", "compact", "--config", config], env, out, "managed-compact")
    managed_wire = strict_json(managed.stdout)
    assert set(managed_wire) == {"version", "snapshot", "collections", "result"}
    assert set(managed_wire["result"]) == {"operation", "response"}
    assert managed_wire["result"]["operation"] == "exact"
    compact_exact_wire(managed_wire["result"]["response"])
    # Serialize the pre-input typed target in the Rust caller; do not guess the
    # legacy RelationTarget JSON representation or choose a target from output.
    reference = json.dumps(context["reference_request"], separators=(",", ":"))
    baseline_ref = run(baseline, ["dictionary-query", store, "--request", reference], env, out, "baseline-reference")
    current_ref = run(binary, ["dictionary-query", store, "--request", reference], env, out, "full-reference")
    assert baseline_ref.stdout == current_ref.stdout and baseline_ref.stderr == current_ref.stderr == b""
    compact_ref = run(binary, ["dictionary-query", store, "--request", reference, "--format", "compact"], env, out, "compact-reference")
    standalone_ref = compact_resolve_wire(strict_json(compact_ref.stdout))
    assert standalone_ref["target"] == context["reference_request"]["target"]
    assert standalone_ref["status"] == strict_json(baseline_ref.stdout)["status"] == "Resolved"
    managed_ref = run(binary, ["lookup", "--request", reference, "--format", "compact", "--config", config], env, out, "managed-reference")
    managed_ref_wire = strict_json(managed_ref.stdout)
    assert set(managed_ref_wire) == {"version", "snapshot", "collections", "result"}
    assert set(managed_ref_wire["result"]) == {"operation", "response"}
    assert managed_ref_wire["result"]["operation"] == "resolve"
    resolved = managed_ref_wire["result"]["response"]
    assert set(resolved) == {"target", "status", "operational_reason", "payload", "catalogue_matches", "requested_release"}
    assert resolved["status"] == "Resolved"
    compact_resolve_wire(resolved["payload"])
    # Public all-absolute routes must not acquire cwd after its removal. These
    # are NEW owned empty directories; the child removes only its own cwd.
    for name, args, expected_wire in [
        ("standalone", ["dictionary-query", store, "--format", "compact", "--request", request], projected.stdout),
        ("managed", ["lookup", "farvi", "--format", "compact", "--config", config], managed.stdout),
    ]:
        child_cwd = out.resolve() / f"deleted-cwd-{name}"
        child_cwd.mkdir(mode=0o700)
        result = run(binary, args, env, out, f"{name}-deleted-cwd", cwd=child_cwd,
                     preexec_fn=lambda path=child_cwd: os.rmdir(path))
        assert result.stderr == b"" and result.stdout == expected_wire
        assert not child_cwd.exists()
    first = context["detail_keys"][0]
    for command, prefix in [("dictionary-detail", [store]), ("lookup-detail", [])]:
        key = json.dumps(first, separators=(",", ":"))
        args = [command, *prefix, "--reference", key]
        if command == "lookup-detail":
            args += ["--config", config]
        complete = strict_json(run(binary, args, env, out, command).stdout)
        if command == "lookup-detail":
            assert set(complete) == {"version", "snapshot", "collections", "response", "catalogue_matches"}
        body = complete["response"] if command == "lookup-detail" else complete
        detail_wire(body)
        assert body["status"] == "Resolved" and body["hit"] is not None
        assert body["detail_ref"] == first
        # The two APIs deliberately bind different captured observations:
        # standalone projection snapshot versus the OUTER managed selection.
        observed = (managed_wire["snapshot"]["binding_sha256"]
                    if command == "lookup-detail" else compact["snapshot_sha256"])
        pinned = run(binary, [*args, "--expected-snapshot", observed], env, out,
                     f"{command}-captured-snapshot")
        assert pinned.stderr == b"" and strict_json(pinned.stdout) == complete
        child_cwd = out.resolve() / f"deleted-cwd-{command}"
        child_cwd.mkdir(mode=0o700)
        deleted = run(binary, args, env, out, f"{command}-deleted-cwd", cwd=child_cwd,
                      preexec_fn=lambda path=child_cwd: os.rmdir(path))
        assert deleted.stderr == b"" and strict_json(deleted.stdout) == complete
        assert not child_cwd.exists()
        for field in ["native_entry_id", "native_assertion_id", "contributor_id", "distributor_id"]:
            changed = dict(first)
            del changed[field]
            bad = [command, *prefix, "--reference", json.dumps(changed)]
            # Absent config plus broken key must retain the static-key error.
            if command == "lookup-detail":
                bad += ["--config", out / "absent-config"]
                if field == "native_entry_id":
                    # Static key failure precedes supplied profile/pack selection,
                    # and must not leak those unobserved values into its contexts.
                    bad += ["--profile", "must-not-be-read", "--pack", "must-not-be-parsed"]
            failure = run(binary, bad, env, out, f"{command}-missing-{field}", 1)
            check_error(failure, command == "lookup-detail", "invalid_detail_reference", "lookup_detail")
        invalids = []
        for field, value in [("version", 2), ("digest_policy", "unknown-policy"),
                             ("role", "entry"), ("source", ""), ("entry_id", ""),
                             ("assertion_id", ""), ("entry_sha256", "G" * 64)]:
            changed = dict(first)
            changed[field] = value
            invalids.append((field, json.dumps(changed)))
        changed = json.loads(json.dumps(first))
        changed["occurrence"]["locator"] = ""
        invalids += [("empty-locator", json.dumps(changed)),
                     ("trailing-key-json", key + " false"),
                     ("malformed-key-json", "{")]
        for field, value in [("namespace", ""), ("sha256", "Z" * 64)]:
            changed = json.loads(json.dumps(first))
            changed["occurrence"]["capture"][field] = value
            invalids.append(("capture-" + field, json.dumps(changed)))
        changed = dict(first)
        changed["unknown_field"] = None
        invalids.append(("unknown-field", json.dumps(changed)))
        for name, invalid in invalids:
            bad = [command, *prefix, "--reference", invalid]
            if command == "lookup-detail":
                bad += ["--config", out / "absent-config"]
            failure = run(binary, bad, env, out, f"{command}-invalid-{name}", 1)
            check_error(failure, command == "lookup-detail", "invalid_detail_reference", "lookup_detail")
        bad = [command, *prefix, "--reference", key, "--expected-snapshot", "é"]
        if command == "lookup-detail":
            bad += ["--config", out / "absent-config"]
        failure = run(binary, bad, env, out, f"{command}-malformed-snapshot", 1)
        check_error(failure, command == "lookup-detail", "malformed_snapshot", "lookup_detail")
    print(json.dumps({"ok": not DIAGNOSTIC_FAILURES, "scope": "diagnostic Tiny CLI; every declared strict error predicate retained, failures are not accepted", "failed_predicates":DIAGNOSTIC_FAILURES,"baseline_sha256": hashlib.sha256(old.stdout).hexdigest()}));sys.exit(1 if DIAGNOSTIC_FAILURES else 0)

if __name__ == "__main__":
    main()
