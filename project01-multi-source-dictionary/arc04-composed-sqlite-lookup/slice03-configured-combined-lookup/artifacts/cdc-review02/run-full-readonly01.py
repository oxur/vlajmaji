#!/usr/bin/env python3
"""One frozen S03 read-only retained-store campaign; never a retry loop."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys


def sha(path):
    value = hashlib.sha256()
    with Path(path).open('rb') as source:
        for block in iter(lambda: source.read(1024 * 1024), b''):
            value.update(block)
    return value.hexdigest()


def write(path, value):
    with path.open('x') as output:
        json.dump(value, output, indent=2)
        output.write('\n')


def main():
    assert len(sys.argv) == 3 and sys.argv[1] == '--execute-reviewed'
    frozen_path = Path(sys.argv[2])
    frozen = json.loads(frozen_path.read_bytes())
    assert frozen['runner_sha256'] == sha(__file__)
    source = Path(frozen['source_root'])
    env = dict(frozen['environment'])
    env['GIT_OPTIONAL_LOCKS'] = '0'
    head = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=source, env=env).decode().strip()
    status = subprocess.check_output(['git', 'status', '--porcelain=v1'], cwd=source, env=env)
    assert head == frozen['candidate_source'] and not status
    for path, expected in frozen['frozen_file_sha256'].items():
        assert sha(path) == expected, path
    campaign = Path(frozen['fresh_campaign'])
    campaign.mkdir(mode=0o700)  # No reuse, pre-delete or automatic second attempt.
    paths = {'database': Path(frozen['database']), 'catalogue': Path(frozen['catalogue'])}
    before = {name: sha(path) for name, path in paths.items()}
    write(campaign / 'before.json', before)
    assert before == frozen['expected_existing_sha256']
    stages = []
    error = None
    try:
        for stage in frozen['expanded_stages']:
            label = stage['name']
            with (campaign / (label + '.receipt.json')).open('x') as output, (campaign / (label + '.wrapper.stderr')).open('xb') as stderr:
                child = subprocess.run(stage['wrapper_argv'], stdout=output, stderr=stderr,
                                       env=frozen['environment'], timeout=610)
            receipt = json.loads((campaign / (label + '.receipt.json')).read_bytes())
            stages.append({'name': label, 'wrapper_exit_code': child.returncode, 'receipt': receipt})
            assert child.returncode == 0 and receipt['decision'] == 'passed', label
            if label == 'configure':
                report = json.loads((campaign / 'configure.stdout').read_bytes())
                assert report['version'] == 1 and report['ok'] is True
                assert report['outcome'] == 'committed' and report['revision'] == 1
                assert report['profile'] == 'local' and report['default_profile'] == 'local'
            if stage.get('checker_argv'):
                with (campaign / (label + '.check.stdout')).open('xb') as output, (campaign / (label + '.check.stderr')).open('xb') as stderr:
                    checked = subprocess.run(stage['checker_argv'], stdout=output, stderr=stderr,
                                             env=frozen['environment'], timeout=30)
                assert checked.returncode == 0, label + ' checker'
    except BaseException as caught:
        error = repr(caught)
    finally:
        after = {name: sha(path) for name, path in paths.items()}
        write(campaign / 'after.json', after)
        files = {str(path.relative_to(campaign)): {'bytes': path.stat().st_size, 'sha256': sha(path)}
                 for path in campaign.rglob('*') if path.is_file()}
        total = sum(row['bytes'] for row in files.values())
        ok = error is None and before == after and total <= frozen['limits']['campaign_output_bytes_postinventory_ceiling']
        write(campaign / 'campaign-summary.json', {'ok': ok, 'candidate': head,
              'error': error, 'stages': stages, 'before': before, 'after': after,
              'database_and_catalogue_unchanged': before == after,
              'output_bytes_before_summary': total, 'files': files,
              'limits': frozen['limits'], 'scope': 'functional read-only reuse; no performance trial'})
    print(json.dumps({'ok': ok, 'campaign': str(campaign), 'error': error, 'stages': len(stages)}))
    return 0 if ok else 1


if __name__ == '__main__':
    raise SystemExit(main())
