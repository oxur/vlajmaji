#!/usr/bin/env python3
"""Prospective S03 child-process controls; not run before frozen handback.

All fixtures are owned tiny Synthetic data. The caller must have completed its
independent public run; no real HOME/default or accepted Full store is used.
"""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys

CLI, PUBLIC, OUT = map(Path, sys.argv[1:4])
OUT.mkdir(mode=0o700)  # Exclusive attempt; never pre-delete or reuse a failure.
ROOT = PUBLIC / "R"
CONFIG = PUBLIC / "profiles.toml"
EXPECTED = json.loads((PUBLIC / "expected-old-unpaged-wire.json").read_bytes())
BASE_ENV = {"PATH": "/usr/bin:/bin", "LC_ALL": "C"}
ERROR_KEYS = {"version", "ok", "category", "detail", "operation", "config_path",
              "root_path", "profile", "requested_profile", "snapshot", "known_pack_ids", "outcome", "writer"}
receipts = []


def validate_error(error, outcome="not_applicable"):
    assert ERROR_KEYS == set(error), (ERROR_KEYS - set(error), set(error) - ERROR_KEYS)
    assert error["version"] == 1 and error["ok"] is False
    assert isinstance(error["category"], str) and error["category"]
    assert isinstance(error["operation"], str) and error["operation"]
    assert isinstance(error["detail"], str) and error["detail"]
    assert isinstance(error["known_pack_ids"], list)
    assert error["outcome"] == outcome


def unwrap_error(envelope, outcome="not_applicable"):
    assert set(envelope) == {"error"}
    error = envelope["error"]
    validate_error(error, outcome)
    return error


sample = dict.fromkeys(ERROR_KEYS)
sample.update(version=1, ok=False, category="request_invalid", operation="lookup",
              detail="method fixture", known_pack_ids=[], outcome="not_applicable")
validate_error(sample)
method_cases = 1
for field in ["config_path", "root_path", "profile", "requested_profile", "snapshot", "writer"]:
    changed = sample.copy()
    del changed[field]
    try:
        validate_error(changed)
    except AssertionError:
        method_cases += 1
    else:
        raise AssertionError(f"method accepted missing nullable {field}")


def digest(data):
    return hashlib.sha256(data).hexdigest()


def child(label, arguments, *, env=None, stdin=None, code=0):
    effective = BASE_ENV | (env or {})
    command = [os.fspath(CLI)] + [os.fspath(arg) for arg in arguments]
    result = subprocess.run(command, env=effective, cwd=OUT, input=stdin,
                            stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                            timeout=30, check=False)
    (OUT / f"{label}.stdout").write_bytes(result.stdout)
    (OUT / f"{label}.stderr").write_bytes(result.stderr)
    receipt = {"label": label, "argv_hex": [os.fsencode(arg).hex() for arg in command],
               "environment": effective, "exit_code": result.returncode,
               "stdout_bytes": len(result.stdout), "stdout_sha256": digest(result.stdout),
               "stderr_bytes": len(result.stderr), "stderr_sha256": digest(result.stderr)}
    receipts.append(receipt)
    (OUT / "attempts.json").write_text(json.dumps(receipts, indent=2) + "\n")
    assert result.returncode == code, (label, receipt, result.stderr[:2048])
    if code == 0:
        assert not result.stderr, (label, result.stderr[:2048])
        assert result.stdout.endswith(b"\n"), label
        return json.loads(result.stdout)
    assert not result.stdout, (label, result.stdout[:2048])
    return unwrap_error(json.loads(result.stderr))


def inner(envelope):
    assert envelope["version"] == 1
    assert envelope["result"]["operation"] == "exact"
    return envelope["result"]["response"]


def copy_config(path):
    path.parent.mkdir(mode=0o700, parents=True, exist_ok=True)
    with path.open("xb") as stream:
        stream.write(CONFIG.read_bytes())
    return path


def failed_stdout(label, arguments, *, outcome):
    """Linux /dev/full fails output without blocking configuration writes."""
    assert sys.platform.startswith("linux"), "actual Linux process requirement"
    command = [os.fspath(CLI)] + [os.fspath(arg) for arg in arguments]
    with open("/dev/full", "wb") as sink:
        result = subprocess.run(command, env=BASE_ENV, cwd=OUT, stdout=sink,
                                stderr=subprocess.PIPE, timeout=30, check=False)
    (OUT / f"{label}.stderr").write_bytes(result.stderr)
    receipts.append({"label": label, "argv_hex": [os.fsencode(arg).hex() for arg in command],
        "environment": BASE_ENV, "exit_code": result.returncode,
        "stdout_sink": "/dev/full; output not captured",
        "stderr_bytes": len(result.stderr), "stderr_sha256": digest(result.stderr)})
    (OUT / "attempts.json").write_text(json.dumps(receipts, indent=2) + "\n")
    assert result.returncode == 1, (label, result.returncode, result.stderr[:2048])
    error = unwrap_error(json.loads(result.stderr), outcome)
    assert error["category"] == "response_output_io"
    return error


profile = child("explicit-profiles-no-home", ["collections", "profiles", "--config", CONFIG])
assert profile["version"] == 1 and profile["default_profile"] == "local"
assert profile["path"]["hex"] == os.fsencode(CONFIG).hex()
selection = child("explicit-selection-no-home", ["collections", "selection", "--config", CONFIG])
assert len(selection["packs"]) == 4
unknown = child("unknown-profile", ["lookup", "farvi", "--config", CONFIG,
               "--profile", "unknown-profile"], code=1)
assert unknown["category"] == "unknown_profile" and unknown["requested_profile"] == "unknown-profile"
assert unknown["profile"] is None
assert unknown["config_path"]["hex"] == os.fsencode(CONFIG).hex()
assert inner(child("known-profile-counterpart", ["lookup", "farvi", "--config", CONFIG,
                   "--profile", "local"])) == EXPECTED

# Literal policies use the already-installed tiny root and never modify it.
disabled_config = OUT / "disabled-profile.toml"
disabled_config.write_text("version = 1\nrevision = 1\ndefault_profile = \"local\"\n"
    "[profiles.local]\nroot = " + json.dumps(os.fspath(ROOT)) + "\n"
    'packs = ["lensisku-jbotci", "llg", "la-lojban", "sutysisku"]\n'
    'disabled = ["lensisku-jbotci"]\n')
disabled_config.chmod(0o600)
disabled = child("known-disabled-override", ["lookup", "farvi", "--config", disabled_config,
                 "--pack", "lensisku-jbotci"], code=1)
assert disabled["category"] == "disabled_selection"
assert disabled["config_path"]["hex"] == os.fsencode(disabled_config).hex()
assert inner(child("enabled-override-counterpart", ["lookup", "farvi", "--config", disabled_config,
                   "--pack", "la-lojban"]))["totals"]["occurrences"] == 1
assert inner(child("word-all", ["lookup", "farvi", "--config", CONFIG])) == EXPECTED
non_broken_pipe_state = "pending_actual_linux"
if sys.platform.startswith("linux"):
    sink_error = failed_stdout("read-output-failure", ["lookup", "farvi", "--config", CONFIG],
                              outcome="not_applicable")
    assert sink_error["config_path"]["hex"] == os.fsencode(CONFIG).hex()
    assert sink_error["snapshot"] is not None
    assert inner(child("read-output-valid-counterpart", ["lookup", "farvi", "--config", CONFIG])) == EXPECTED
    sink_config = OUT / "failed-output/config.toml"
    sink_error = failed_stdout("committed-output-failure", ["collections", "configure", ROOT,
        "--profile", "local", "--all", "--make-default", "--config", sink_config], outcome="committed")
    assert sink_error["config_path"]["hex"] == os.fsencode(sink_config).hex()
    readback = child("committed-output-readback", ["collections", "profiles", "--config", sink_config])
    assert readback["revision"] == 1
    assert sorted(next(profile for profile in readback["profiles"] if profile["name"] == "local")["packs"]) == [
        "la-lojban", "lensisku-jbotci", "llg", "sutysisku"]
    assert inner(child("committed-output-lookup", ["lookup", "farvi", "--config", sink_config])) == EXPECTED
    non_broken_pipe_state = "actual_linux_pass"
request = json.dumps({"operation": "exact", "spelling": "farvi"})
assert inner(child("stdin-all", ["lookup", "--config", CONFIG], stdin=request.encode())) == EXPECTED

missing = OUT / "must-not-create/absent.toml"
error = child("parse-before-config", ["lookup", "--request", "{", "--config", missing], code=1)
assert error["category"] == "request_invalid" and error["operation"] == "lookup"
assert error["snapshot"] is None
assert not missing.parent.exists()
error = child("missing-config", ["collections", "profiles", "--config", missing], code=1)
assert error["category"] == "config_missing"
assert error["config_path"]["hex"] == os.fsencode(missing).hex()
assert not missing.parent.exists()
error = child("missing-home", ["collections", "profiles"], code=1)
assert error["category"] == "config_path"

home = OUT / "isolated-home"
fallback = copy_config(home / ".config/vlajmaji/config.toml")
for tag, xdg in [("relative", "relative-must-not-create"), ("empty", "")]:
    report = child(f"xdg-{tag}-fallback", ["collections", "profiles"],
                   env={"HOME": os.fspath(home), "XDG_CONFIG_HOME": xdg})
    assert report["path"]["hex"] == os.fsencode(fallback).hex()
assert not (OUT / "relative-must-not-create").exists()
xdg = OUT / "isolated-xdg"
chosen = copy_config(xdg / "vlajmaji/config.toml")
report = child("absolute-xdg-wins", ["collections", "profiles"],
               env={"HOME": "relative-unusable-home", "XDG_CONFIG_HOME": os.fspath(xdg)})
assert report["path"]["hex"] == os.fsencode(chosen).hex()

assert inner(child("explicit-jb", ["lookup", "farvi", "--config", CONFIG, "--pack", "lensisku-jbotci"]))["totals"]["occurrences"] == 1
assert inner(child("explicit-la", ["lookup", "farvi", "--config", CONFIG, "--pack", "la-lojban"]))["totals"]["occurrences"] == 1
source = inner(child("source-alias", ["lookup", "farvi", "--config", CONFIG, "--source", "LSK"]))
assert source["totals"]["occurrences"] == 2 and source["totals"]["source_groups"] == 1
assert source["groups"][0]["source"]["id"] == "lensisku"

expected_ids = [hit["assertion"]["id"] for group in EXPECTED["groups"] for hit in group["hits"]]
for size in [0, 1, 2, 3]:
    cursor = None
    accumulated = []
    for page in range(6):
        arguments = ["lookup", "farvi", "--config", CONFIG, "--page-size", str(size)]
        if cursor is not None:
            arguments += ["--cursor", cursor]
        response = inner(child(f"page-{size}-{page}", arguments))
        assert response["totals"] == EXPECTED["totals"]
        assert response["source_totals"] == EXPECTED["source_totals"]
        accumulated += [hit["assertion"]["id"] for group in response["groups"] for hit in group["hits"]]
        cursor = response["next_cursor"]
        if cursor is None:
            break
    assert cursor is None, "finite five-hit page chain"
    assert accumulated == ([] if size == 0 else expected_ids)

generation = json.loads((PUBLIC / "post-install-generation.json").read_bytes())
legacy = child("legacy-explicit-no-home", ["dictionary-query", generation["store_path"], "--request", request])
assert legacy == EXPECTED

# A real lost-output channel after mutation; no shipped failpoint or empty data.
lost_config = OUT / "lost-output/config.toml"
arguments = ["collections", "configure", os.fspath(ROOT), "--profile", "local", "--all", "--make-default", "--config", os.fspath(lost_config)]
process = subprocess.Popen([os.fspath(CLI), *arguments], env=BASE_ENV, cwd=OUT,
                           stdout=subprocess.PIPE, stderr=subprocess.PIPE)
process.stdout.close()
process.stdout = None
_, stderr = process.communicate(timeout=30)
(OUT / "lost-output.stderr").write_bytes(stderr)
receipts.append({"label": "lost-output", "argv_hex": [os.fsencode(arg).hex() for arg in [os.fspath(CLI), *arguments]],
                 "environment": BASE_ENV, "exit_code": process.returncode,
                 "stdout_capture": "reader deliberately closed; output bytes unavailable",
                 "stderr_bytes": len(stderr), "stderr_sha256": digest(stderr)})
(OUT / "attempts.json").write_text(json.dumps(receipts, indent=2) + "\n")
assert process.returncode == 0 and not stderr
committed = child("lost-output-reconcile", ["collections", "profiles", "--config", lost_config])
assert committed["revision"] == 1 and committed["default_profile"] == "local"
assert sorted(committed["profiles"][0]["packs"]) == ["la-lojban", "lensisku-jbotci", "llg", "sutysisku"]
assert inner(child("lost-output-lookup", ["lookup", "farvi", "--config", lost_config])) == EXPECTED
result = {"ok": True, "scope": "isolated process/default/old-route/page and nonempty lost-output controls; native/remaining fault cases are separate", "non_broken_pipe_process_state":non_broken_pipe_state,"processes": len(receipts), "error_method_cases": method_cases, "cli_sha256": digest(CLI.read_bytes()), "public_root": os.fspath(PUBLIC)}
(OUT / "summary.json").write_text(json.dumps(result, indent=2) + "\n")
print(json.dumps(result))
