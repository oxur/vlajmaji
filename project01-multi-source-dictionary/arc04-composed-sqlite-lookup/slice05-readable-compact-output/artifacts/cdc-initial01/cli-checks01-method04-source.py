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

MANAGED = set("version ok category detail operation config_path root_path profile requested_profile snapshot known_pack_ids outcome writer".split())
STANDALONE = {"code", "message", "sources"}

def strict_json(raw):
    # Reject trailing material, including warnings. Whitespace/LF alone is framing.
    text = raw.decode("utf-8")
    obj, end = json.JSONDecoder().raw_decode(text)
    assert not text[end:].strip(), text[end:]
    return obj

def run(binary, args, env, out, name, expected=0, stdin=None, cwd=None, preexec_fn=None):
    result = subprocess.run([str(binary), *map(str, args)], input=stdin,
                            capture_output=True, env=env, timeout=60,
                            cwd=cwd, preexec_fn=preexec_fn)
    for suffix, data in [("stdout", result.stdout), ("stderr", result.stderr)]:
        with (out / f"{name}.{suffix}").open("xb") as handle:
            handle.write(data)
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
    compact = strict_json(projected.stdout)
    assert "response" not in compact and "result" not in compact
    assert compact["totals"]["occurrences"] == 5
    assert len([h for group in compact["groups"] for h in group["hits"]]) == 5
    configured = run(binary, ["collections", "configure", root, "--all", "--profile", "tiny", "--make-default", "--config", config], env, out, "configure")
    assert strict_json(configured.stdout)["outcome"] == "committed"
    managed = run(binary, ["lookup", "farvi", "--format", "compact", "--config", config], env, out, "managed-compact")
    assert strict_json(managed.stdout)["result"]["operation"] == "exact"
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
        body = complete["response"] if command == "lookup-detail" else complete
        assert body["status"] == "Resolved" and body["hit"] is not None
        assert body["detail_ref"] == first
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
            error(failure, command == "lookup-detail", "invalid_detail_reference", "lookup_detail")
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
            error(failure, command == "lookup-detail", "invalid_detail_reference", "lookup_detail")
        bad = [command, *prefix, "--reference", key, "--expected-snapshot", "é"]
        if command == "lookup-detail":
            bad += ["--config", out / "absent-config"]
        failure = run(binary, bad, env, out, f"{command}-malformed-snapshot", 1)
        error(failure, command == "lookup-detail", "malformed_snapshot", "lookup_detail")
    print(json.dumps({"ok": True, "scope": "prospective Tiny CLI; exact executed child receipts retained", "baseline_sha256": hashlib.sha256(old.stdout).hexdigest()}))

if __name__ == "__main__":
    main()
