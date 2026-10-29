#!/usr/bin/env python3
"""Prospective S01 proof; run only after actual CDC source/target handback."""
import hashlib
import json
import os
from pathlib import Path
import sqlite3
import subprocess
import sys
import tempfile


def digest(data):
    return hashlib.sha256(data).hexdigest()


def main():
    example, cli = (Path(p) for p in sys.argv[1:3])
    base = Path(tempfile.mkdtemp(prefix='vlajmaji-a05-s01-cdc-'))
    env = os.environ.copy()
    for name in ('HOME', 'XDG_DATA_HOME', 'XDG_CONFIG_HOME', 'TMPDIR'):
        value = base / name.lower()
        value.mkdir()
        env[name] = str(value)
    records = []

    def run(label, args, expected=0, input_bytes=None, output_fd=None):
        out = subprocess.run([str(x) for x in args], input=input_bytes,
                             stdout=output_fd if output_fd is not None else subprocess.PIPE,
                             stderr=subprocess.PIPE, env=env, cwd=base, timeout=120)
        for suffix, data in (('stdout', out.stdout), ('stderr', out.stderr)):
            if data is not None:
                (base / (label + '.' + suffix)).write_bytes(data)
        records.append({'label': label, 'argv': [str(x) for x in args],
                        'exit': out.returncode, 'expected_exit': expected,
                        'stdout_sha256': digest(out.stdout) if out.stdout is not None else None,
                        'stderr_sha256': digest(out.stderr)})
        (base / 'commands.json').write_text(json.dumps(records, indent=2) + '\n')
        assert out.returncode == expected, (label, out.returncode, out.stderr)
        return out

    def query(label, *args, expected=0, input_bytes=None):
        result = run(label, [cli, 'owned', *args], expected, input_bytes)
        data = result.stdout if expected == 0 else result.stderr
        if expected == 0:
            assert data.endswith(b'\n') and not data.endswith(b'\n\n'), label
            assert not result.stderr, (label, result.stderr)
        decoded = json.loads(data)
        return decoded if expected == 0 else decoded['error']

    def summary(model, sequence, raw):
        return {'sequence': sequence, 'revision': model['metadata']['owned_revisions'][-1],
                'state_sha256': digest(raw)}

    def view(model, sequence, raw):
        return {'version': 1, 'source': 'owned-example',
                'commit': summary(model, sequence, raw), 'dictionary': model}

    inputs, api_root, cli_root = base / 'inputs', base / 'public-journal', base / 'cli-journal'
    public = run('public-example', [example, inputs, api_root])
    seed_bytes, edit_bytes = ((inputs / n).read_bytes() for n in ('seed.json', 'edit.json'))
    seed, edit = json.loads(seed_bytes), json.loads(edit_bytes)
    # Literal pre-CLI oracle, checked against source-guided emitted input, never query-picked.
    assert seed['metadata']['owned_revisions'][0]['id'] == 'owned-r0'
    assert seed['metadata']['owned_revisions'][0]['assertions'] == []
    assert [x['id'] for x in edit['metadata']['assertions']] == ['owned-assertion-a', 'owned-assertion-b']
    assert [x['definition']['Value']['text'] for x in edit['metadata']['assertions']] == ['grow', 'develop']
    assert edit['metadata']['entries'][0]['spelling'] == {'Value': 'farvi'}
    assert edit['metadata']['owned_heads'] == [{'source': 'owned-example', 'revision': 'owned-r1'}]
    decoder, rest, public_values = json.JSONDecoder(), public.stdout.decode(), []
    while rest.strip():
        value, count = decoder.raw_decode(rest.lstrip())
        public_values.append(value)
        rest = rest.lstrip()[count:]
    assert len(public_values) == 6
    init_report = {'version': 1, 'operation': 'init', 'source': 'owned-example',
                   'previous_head': None, 'head': summary(seed, 0, seed_bytes)}
    edit_report = {'version': 1, 'operation': 'edit', 'source': 'owned-example',
                   'previous_head': 'owned-r0', 'head': summary(edit, 1, edit_bytes)}
    head_report = {'version': 1, 'source': 'owned-example', 'head': summary(edit, 1, edit_bytes)}
    history_report = {'version': 1, 'source': 'owned-example', 'head': 'owned-r1',
                      'commits': [summary(seed, 0, seed_bytes), summary(edit, 1, edit_bytes)]}
    assert public_values == [init_report, edit_report, head_report, history_report,
                             view(edit, 1, edit_bytes), view(seed, 0, seed_bytes)]
    assert query('init-file', 'init', cli_root, '--input', inputs / 'seed.json') == init_report
    assert query('head-r0', 'head', cli_root) == {'version': 1, 'source': 'owned-example', 'head': summary(seed, 0, seed_bytes)}
    assert query('edit-stdin', 'edit', cli_root, '--expected-head', 'owned-r0', input_bytes=edit_bytes) == edit_report
    assert query('head-r1', 'head', cli_root) == head_report
    assert query('history-r1', 'history', cli_root) == history_report
    assert query('show-r1', 'show', cli_root) == view(edit, 1, edit_bytes)
    assert query('show-r0', 'show', cli_root, '--revision', 'owned-r0') == view(seed, 0, seed_bytes)
    replay = query('stale-replay', 'edit', cli_root, '--expected-head', 'owned-r0', input_bytes=edit_bytes, expected=1)
    assert replay['code'] == 'owned_conflict' and replay['sources'] == []
    assert query('unchanged-after-replay', 'show', cli_root) == view(edit, 1, edit_bytes)
    # Independent read-only schema/body/prefix check of this newly owned Tiny journal.
    database = cli_root / 'owned.sqlite'
    connection = sqlite3.connect('file:' + str(database) + '?mode=ro', uri=True)
    try:
        assert connection.execute('PRAGMA application_id').fetchone()[0] == 0x564c4f57
        assert connection.execute('PRAGMA user_version').fetchone()[0] == 1
        assert connection.execute('PRAGMA journal_mode').fetchone()[0] == 'delete'
        strict = {row[1]: row[5] for row in connection.execute('PRAGMA table_list') if row[1].startswith('owned_')}
        assert strict == {name: 1 for name in ('owned_meta', 'owned_records', 'owned_commits', 'owned_commit_sections')}
        def sections(model):
            registry = {'registry.' + key: value for key, value in model['registry'].items() if key != 'version'}
            metadata = {'metadata.' + key: value for key, value in model['metadata'].items() if key not in ('catalogue_revision', 'owned_heads')}
            return {**registry, 'captures': model['captures'], **metadata}
        seed_sections, edit_sections = sections(seed), sections(edit)
        assert len(seed_sections) == len(edit_sections) == 24
        for name, expected_rows in edit_sections.items():
            rows = connection.execute('SELECT ordinal,payload FROM owned_records WHERE section=? ORDER BY ordinal', (name,)).fetchall()
            assert [ordinal for ordinal, _ in rows] == list(range(len(expected_rows)))
            assert [json.loads(payload) for _, payload in rows] == expected_rows
        for revision, expected in (('owned-r0', seed_sections), ('owned-r1', edit_sections)):
            prefixes = dict(connection.execute('SELECT section,length FROM owned_commit_sections WHERE revision_id=?', (revision,)))
            assert prefixes == {name: len(rows) for name, rows in expected.items()}
        assert connection.execute('SELECT COUNT(*) FROM owned_commits').fetchone()[0] == 2
    finally:
        connection.close()
    missing = query('unknown-revision', 'show', cli_root, '--revision', 'owned-missing', expected=1)
    assert missing['code'] == 'owned_not_found'
    absent_root = base / 'never-created'
    bad = query('strict-input-before-root', 'init', absent_root, input_bytes=b'{"version":1,"version":1}', expected=1)
    assert bad['code'] == 'owned_invalid_dictionary' and not absent_root.exists()
    print(json.dumps({'status': 'PASS', 'scratch': str(base), 'commands': len(records),
                      'scope': 'one public example plus explicit-root typed CLI/history/error proof; no Full/default/network',
                      'example_sha256': digest(example.read_bytes()), 'cli_sha256': digest(cli.read_bytes())}))


if __name__ == '__main__':
    main()
