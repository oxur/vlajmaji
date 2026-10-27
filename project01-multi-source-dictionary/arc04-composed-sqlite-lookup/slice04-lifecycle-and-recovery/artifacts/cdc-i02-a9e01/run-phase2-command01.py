"""Phase2 bounded command capture. Execute only after explicit frozen CC handback."""
import sys, os, json, time, hashlib, subprocess, signal
from pathlib import Path
base = Path(__file__).parent
expected, label, caller_name, *argv = sys.argv[1:]
assert expected and label and argv
source = Path('/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc04/vlajmaji')
git_env = {**os.environ, 'GIT_OPTIONAL_LOCKS': '0'}
observed = subprocess.check_output(['git', '--no-optional-locks', '-C', str(source), 'rev-parse', 'HEAD'], env=git_env).decode().strip()
assert observed == expected
assert subprocess.check_output(['git', '--no-optional-locks', '-C', str(source), 'status', '--porcelain=v1'], env=git_env) == b''
log, receipt = base / (label + '.log'), base / (label + '.json')
assert not log.exists() and not receipt.exists()
caller = base / caller_name
assert caller.is_dir()
sources = {}
for path in sorted(caller.rglob('*')):
    if path.is_file() and 'target' not in path.relative_to(caller).parts:
        sources[str(path.relative_to(base))] = hashlib.sha256(path.read_bytes()).hexdigest()
env = os.environ.copy()
env.update(CARGO='/tmp/vlajmaji-p01-a04-cargo', RUST_TOOLCHAIN='1.98.0', CARGO_TARGET_DIR='/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc01/target', CARGO_NET_OFFLINE='true')
started = time.monotonic()
timed_out = False
with log.open('xb') as out:
    child = subprocess.Popen(argv, stdout=out, stderr=subprocess.STDOUT, env=env, start_new_session=True)
    try:
        code = child.wait(timeout=600)
    except subprocess.TimeoutExpired:
        timed_out = True
        os.killpg(child.pid, signal.SIGKILL)
        code = child.wait()
raw = log.read_bytes()
data = dict(argv=argv, cwd=os.getcwd(), source_commit=observed, caller_sources_before=sources, exit_code=code, timed_out=timed_out, elapsed_s=time.monotonic()-started, log_bytes=len(raw), log_sha256=hashlib.sha256(raw).hexdigest(), resource_scope='active600s process-group timeout; no RSS enforcement or observation')
with receipt.open('x') as out:
    json.dump(data, out, indent=2)
    out.write('\n')
print(json.dumps(data))
print(raw.decode(errors='replace')[-3200:])
sys.exit(1 if timed_out else code)
