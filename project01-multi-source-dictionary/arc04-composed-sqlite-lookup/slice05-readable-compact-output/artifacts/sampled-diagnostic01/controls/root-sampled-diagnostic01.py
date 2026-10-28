"""One authorized read-only sampled observer launch; never an acceptance run."""
import datetime, hashlib, importlib.util, json, os, re, shutil, signal
import subprocess, sys, time, traceback
from pathlib import Path

sys.dont_write_bytecode = True
BASE = Path('/tmp/vlajmaji-p01-a04-s05-cdc')
OUT = BASE / 'sampled-diagnostic01'
PACKET = BASE / 'numeric-full-preflight04.json'
PACKET_SHA = '1021d4cc6f8751c910f4079c4ee250afc1814d6f0b0d4e0a140893606100b9e9'
GIB = 1024 ** 3

def sha(path):
    h = hashlib.sha256()
    with Path(path).open('rb') as f:
        for part in iter(lambda: f.read(8 * 1024 * 1024), b''):
            h.update(part)
    return h.hexdigest()

def write(name, value):
    with (OUT / name).open('x') as f:
        json.dump(value, f, indent=2)
        f.write('\n')

def utc():
    return datetime.datetime.now(datetime.timezone.utc).isoformat()

def terminate_all(processes, deadline):
    processes = [p for p in processes if p is not None]
    # Signal both groups before waiting; all reaping shares one absolute bound.
    for process in processes:
        try:
            os.killpg(process.pid, signal.SIGTERM)
        except ProcessLookupError:
            pass
    for process in processes:
        try:
            process.wait(timeout=max(0, min(0.5, deadline - time.monotonic())))
        except subprocess.TimeoutExpired:
            pass
    # A direct parent can exit while a descendant remains in its group.
    for process in processes:
        try:
            os.killpg(process.pid, signal.SIGKILL)
        except ProcessLookupError:
            pass
    failures = []
    for process in processes:
        try:
            process.wait(timeout=max(0, deadline - time.monotonic()))
        except subprocess.TimeoutExpired:
            failures.append(process.pid)
    if failures:
        raise TimeoutError('direct-parent reaping incomplete at shared deadline: ' + repr(failures))

def main():
    assert sha(PACKET) == PACKET_SHA
    cfg = json.loads(PACKET.read_bytes())
    helper = BASE / 'numeric-runner02.py'
    helper_pin = next(p for p in cfg['frozen_files'] if p['path'] == '/private/tmp/vlajmaji-p01-a04-s05-cdc/numeric-runner02.py')
    assert sha(helper) == helper_pin['sha256']
    spec = importlib.util.spec_from_file_location('retained_numeric_helpers', helper)
    helpers = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(helpers)  # Pure helper definitions; main is guarded.
    assert not OUT.exists() and not OUT.is_symlink()
    helpers.nofollow(BASE)
    OUT.mkdir(mode=0o700)
    write('authority.json', {
        'user_message': 'I agree to the diagnostic run for data gathering',
        'scope': 'root-diagnostic-proposal01.md; one observer and one exact-PID sampler; no retry',
        'proposal_sha256': sha(BASE / 'root-diagnostic-proposal01.md'),
        'launcher_sha256': sha(Path(__file__)),
        'packet_sha256': PACKET_SHA,
        'recorded_utc': utc(),
        'acceptance': False,
    })
    state = {'started_utc': utc(), 'observer_launches': 0, 'sampler_launches': 0,
             'acceptance': False, 'source_changes': False, 'errors': []}
    timed = sampler = None
    db_before = None
    target_before = None
    started = None
    try:
        git = ['git', '--no-optional-locks', '-C', cfg['source']]
        assert subprocess.check_output(git + ['rev-parse', 'HEAD']).decode().strip() == cfg['source_commit']
        assert subprocess.check_output(git + ['rev-parse', 'HEAD^{tree}']).decode().strip() == cfg['source_tree']
        assert not subprocess.check_output(git + ['status', '--porcelain=v1'])
        # Recheck exactly the approved observer/config/workload identities.
        for key in ['observer', 'config', 'workload']:
            canonical = str(helpers.canonical_system_tmp(Path(cfg[key])))
            entry = next(p for p in cfg['frozen_files'] if p['path'] == canonical)
            helpers.pin(entry)
        ps = subprocess.check_output(['/bin/ps', '-axo', 'pid=,comm=']).decode()
        active = [line for line in ps.splitlines()
                  if Path(line.split(maxsplit=1)[1]).name in ['cargo', 'numeric-observer']]
        assert not active, active
        assert shutil.disk_usage(OUT).free >= cfg['minimum_free_bytes']
        scratch = list(map(Path, cfg['new_s05_scratch_paths']))
        assert BASE in scratch
        inv = helpers.inventory(scratch)
        write('scratch-before.json', inv)
        assert inv['bytes'] <= 12 * GIB
        target_before = helpers.target_cache_bytes(Path(cfg['target_cache']))
        write('target-before.json', {'bytes': target_before, 'path': cfg['target_cache']})
        dbs = [helpers.leaf(p, cfg['root']) for p in cfg['database_paths']]
        assert len(dbs) == 2 and set(map(str, dbs)) == set(cfg['expected_database_sha256'])
        db_before = {str(p): sha(p) for p in dbs}
        write('database-before.json', db_before)
        assert db_before == cfg['expected_database_sha256']
        env = {'PATH': '/usr/bin:/bin:/usr/sbin:/sbin', 'LANG': 'en_US.UTF-8', 'LC_ALL': 'en_US.UTF-8'}
        for key, name in [('HOME', 'isolated-home'), ('XDG_DATA_HOME', 'isolated-data'), ('XDG_CONFIG_HOME', 'isolated-config')]:
            (OUT / name).mkdir(mode=0o700)
            env[key] = str(OUT / name)
        write('environment.json', env)
        argv = [cfg['observer'], 'warm-full', 'full', '1', str(OUT)]
        # time(1) owns RSS measurement; its single shim publishes its own exact
        # PID, then execs the frozen observer without spawning another process.
        shim = 'import os,sys; f=open(sys.argv[1],"x"); f.write(str(os.getpid())); f.close(); os.execv(sys.argv[2],sys.argv[2:])'
        launch = ['/usr/bin/time', '-l', cfg['python'], '-B', '-c', shim, str(OUT / 'observer.pid'), *argv]
        write('launch.json', {'argv': argv, 'wrapper_argv': launch, 'timeout_seconds': 60,
                              'sampler_seconds': 10, 'sampling_interval_ms': 5})
        with (OUT / 'stdout').open('xb') as stdout, (OUT / 'stderr').open('xb') as stderr, (OUT / 'sample.stdout').open('xb') as sout, (OUT / 'sample.stderr').open('xb') as serr:
            started = time.monotonic()
            state['launch_utc'] = utc()
            timed = subprocess.Popen(launch, stdout=stdout, stderr=stderr, env=env, start_new_session=True)
            state['observer_launches'] = 1
            state['time_pid'] = timed.pid
            # Wait only for this launched shim to exec; never select by name.
            pid = None
            deadline = started + 2
            while time.monotonic() < deadline and timed.poll() is None:
                pidfile = OUT / 'observer.pid'
                if pidfile.exists() and pidfile.read_text().strip():
                    candidate = int(pidfile.read_text())
                    result = subprocess.run(['/bin/ps', '-p', str(candidate), '-o', 'comm='], capture_output=True, text=True, timeout=max(0.01, min(0.25, deadline - time.monotonic())))
                    if result.returncode == 0 and Path(result.stdout.strip()).name == 'numeric-observer':
                        pid = candidate
                        break
                time.sleep(0.01)
            assert pid is not None, 'observer exec/PID could not be confirmed; no sampling retry'
            state['observer_pid'] = pid
            sample_argv = ['/usr/bin/sample', str(pid), '10', '5', '-mayDie', '-file', str(OUT / 'sample.txt')]
            state['sample_argv'] = sample_argv
            state['sampler_launch_utc'] = utc()
            sampler = subprocess.Popen(sample_argv, stdout=sout, stderr=serr, env=env, start_new_session=True)
            state['sampler_launches'] = 1
            state['sampler_pid'] = sampler.pid
            while timed.poll() is None or sampler.poll() is None:
                if sampler.poll() is not None and sampler.returncode != 0:
                    raise RuntimeError('sampler failed/attachment denied; stop without retry')
                # Reserve five seconds for termination and reaping within 60s.
                if time.monotonic() - started >= 55:
                    state['timed_out'] = True
                    raise TimeoutError('60-second diagnostic limit')
                time.sleep(0.05)
            state.update(observer_exit=timed.wait(), sampler_exit=sampler.wait(), timed_out=False)
    except BaseException as error:
        state['errors'].append(repr(error))
        state['traceback'] = traceback.format_exc()
    finally:
        try:
            terminate_all([sampler, timed], started + 60 if started is not None else time.monotonic() + 1)
        except BaseException as error:
            state['errors'].append('reap: ' + repr(error))
        if timed is not None:
            state['observer_exit'] = timed.returncode
        if sampler is not None:
            state['sampler_exit'] = sampler.returncode
        if started is not None:
            state['elapsed_observer_and_sampler_seconds'] = time.monotonic() - started
        # Exactly the matching after hashes; no additional dataset traversal.
        if db_before is not None:
            try:
                after = {p: sha(p) for p in cfg['database_paths']}
                write('database-after.json', after)
                state['database_unchanged'] = after == db_before
            except BaseException as error:
                state['errors'].append('database-after: ' + repr(error))
        if target_before is not None:
            try:
                after = helpers.target_cache_bytes(Path(cfg['target_cache']))
                write('target-after.json', {'bytes': after, 'path': cfg['target_cache']})
                state['target_delta_bytes'] = after - target_before
            except BaseException as error:
                state['errors'].append('target-after: ' + repr(error))
        try:
            stderr = (OUT / 'stderr').read_bytes() if (OUT / 'stderr').exists() else b''
            matches = re.findall(rb'^\s*(\d+)\s+maximum resident set size\s*$', stderr, re.M)
            state['observer_maximum_rss_bytes'] = int(matches[-1]) if matches else None
            state['rss_within_8gib'] = bool(matches) and 0 < int(matches[-1]) <= 8 * GIB
            state['file_sizes_before_receipt'] = {str(p.relative_to(OUT)): p.stat().st_size for p in OUT.rglob('*') if p.is_file()}
            state['outputs_within_128mib'] = all(n <= 128 * 1024 ** 2 for n in state['file_sizes_before_receipt'].values())
            inv = helpers.inventory(list(map(Path, cfg['new_s05_scratch_paths'])))
            write('scratch-after.json', inv)
            state['observed_named_scratch_bytes'] = inv['bytes']
            state['scratch_within_12gib'] = inv['bytes'] <= 12 * GIB
        except BaseException as error:
            state['errors'].append('resource-after: ' + repr(error))
        state['completed_utc'] = utc()
        state['resource_scope'] = '60-second child-group timeout; RSS/disk limits checked after completion; scratch excludes subsequently written scratch-after and receipt bytes'
        state['numerical_acceptance'] = 'INVALID: sampling overhead; diagnostic only'
        write('receipt.json', state)
        print(json.dumps(state, sort_keys=True))
    return 0 if not state['errors'] and state.get('sampler_exit') == 0 and state.get('database_unchanged') and state.get('target_delta_bytes') == 0 and state.get('rss_within_8gib') and state.get('outputs_within_128mib') and state.get('scratch_within_12gib') else 1

if __name__ == '__main__':
    raise SystemExit(main())
