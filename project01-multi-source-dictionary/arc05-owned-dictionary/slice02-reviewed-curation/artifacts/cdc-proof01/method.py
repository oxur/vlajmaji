#!/usr/bin/env python3
"""S02 prospective independent public/CLI proof. NO launch before frozen handback.

Uses accepted S01 input a/b assertions for the CLI workflow, independently of
S02's separate public generator (whose own base intentionally has no assertions).
No benchmark observer, Full/default lookup, network, implicit root or retry.
"""
import copy
import hashlib
import json
import os
from pathlib import Path
import sqlite3
import subprocess
import sys
import tempfile


def encoded(value):
    return json.dumps(value, ensure_ascii=False, separators=(',', ':')).encode()


def sha(data):
    return hashlib.sha256(data).hexdigest()


def tuple_id(occurrence):
    values = (occurrence['capture']['namespace'], occurrence['capture']['sha256'], occurrence['locator'])
    return b''.join(len(s.encode()).to_bytes(8, 'big') + s.encode() for s in values).hex()


def native_path(path):
    return {'encoding': 'unix-bytes', 'hex': os.fsencode(path).hex(), 'utf8': str(path)}


def summary(release):
    result = {'id': release['id'], 'component_id': release['component'],
              'distributor_id': release['distributor']}
    for key in ('upstream_revision', 'distributor_revision', 'captured_at', 'time_precision',
                'schema_version', 'normalizer', 'normalizer_version', 'captures', 'accounting'):
        result[key] = release[key]
    result.update(diagnostics_count=len(release['diagnostics']), selected=release['selected'],
                  availability=release['availability'], content_notices_count=len(release['content_notices']),
                  content_notice_licenses=[n['license'] for n in release['content_notices']],
                  content_notice_locators=[n['locator'] for n in release['content_notices']],
                  redistribution=release['redistribution'].lower(),
                  redistribution_evidence=release['redistribution_evidence'])
    return result


def main():
    example, cli, s01_inputs = [Path(p) for p in sys.argv[1:4]]
    binary_hashes = {str(p): sha(p.read_bytes()) for p in (example, cli)}
    base = Path(tempfile.mkdtemp(prefix='vlajmaji-a05-s02-cdc-'))
    print('owned proof root:', base, flush=True)
    env = os.environ.copy()
    for name in ('HOME', 'XDG_DATA_HOME', 'XDG_CONFIG_HOME', 'TMPDIR'):
        path = base / name.lower()
        path.mkdir()
        env[name] = str(path)
    env.pop('RELOJBAN_RESEARCH_ROOT', None)
    records = []

    def save(name, data):
        with (base / name).open('xb') as out:
            out.write(data)

    def run(label, args, expected=0, input_bytes=None, stdout=None):
        args = [str(a) for a in args]
        result = subprocess.run(args, env=env, cwd=base, input=input_bytes,
                                stdout=subprocess.PIPE if stdout is None else stdout,
                                stderr=subprocess.PIPE, timeout=120)
        if result.stdout is not None:
            save(label + '.stdout', result.stdout)
        save(label + '.stderr', result.stderr)
        records.append({'label': label, 'argv': args, 'exit': result.returncode,
                        'expected_exit': expected, 'stdout_sha256': sha(result.stdout) if result.stdout is not None else None,
                        'stderr_sha256': sha(result.stderr)})
        (base / 'commands.json').write_bytes(encoded(records) + b'\n')
        assert result.returncode == expected, (label, result.returncode, result.stderr)
        return result

    def owned(label, args, expected=0, input_bytes=None):
        result = run(label, [cli, 'owned', *args], expected, input_bytes)
        if not expected:
            assert not result.stderr and result.stdout.endswith(b'\n') and not result.stdout.endswith(b'\n\n'), label
            return json.loads(result.stdout)
        return json.loads(result.stderr)['error']

    def logical_snapshot(root):
        # Only this proof's freshly created explicit journal is inspectable.
        root.relative_to(base)
        database = root / 'owned.sqlite'
        assert root.is_dir() and not root.is_symlink()
        assert database.is_file() and not database.is_symlink()
        allowed = {'owned_meta', 'owned_records', 'owned_commits', 'owned_commit_sections',
                   'owned_context_pins', 'owned_curation_records',
                   'owned_curation_revisions', 'owned_curation_prefixes'}
        with sqlite3.connect('file:' + str(database) + '?mode=ro', uri=True) as db:
            schema = db.execute("SELECT type,name,tbl_name,sql FROM sqlite_master "
                                "WHERE name NOT LIKE 'sqlite_%' ORDER BY type,name").fetchall()
            tables = [row[1] for row in schema if row[0] == 'table']
            assert set(tables) <= allowed and {'owned_meta', 'owned_records', 'owned_commits', 'owned_commit_sections'} <= set(tables)
            rows = {name: sorted(db.execute('SELECT * FROM "' + name + '"').fetchall(), key=repr)
                    for name in tables}
            return {'application_id': db.execute('PRAGMA application_id').fetchone()[0],
                    'user_version': db.execute('PRAGMA user_version').fetchone()[0],
                    'schema': schema, 'tables': rows}

    def save_logical_snapshot(label, snapshot):
        def wire(value):
            if isinstance(value, bytes):
                return {'blob_hex': value.hex()}
            if isinstance(value, (tuple, list)):
                return [wire(v) for v in value]
            if isinstance(value, dict):
                return {k: wire(v) for k, v in value.items()}
            return value
        save(label + '.logical-sqlite.json', encoded(wire(snapshot)))

    public_inputs = base / 'public-inputs'
    public_view = base / 'public-view.json'
    run('public-example', [example, public_inputs, base / 'public-journal', public_view])
    context_input_bytes = (public_inputs / 'context.json').read_bytes()
    supplied = json.loads(context_input_bytes)
    context = supplied['dictionary']
    prefix = b'{"id":' + encoded(supplied['id']) + b',"dictionary":'
    assert context_input_bytes.startswith(prefix) and context_input_bytes.endswith(b'}')
    assert context_input_bytes[len(prefix):-1] == encoded(context)
    assert sorted(a['definition']['Value']['text'] for a in context['metadata']['assertions']) == ['good', 'greeting', 'salutation']
    assert len(context['metadata']['assertions']) == 3

    # Accepted retained INPUTS, not output-selected or Compact-selected oracles.
    seed_raw, r1_raw = [(s01_inputs / n).read_bytes() for n in ('seed.json', 'edit.json')]
    assert (len(seed_raw), sha(seed_raw)) == (2003, '178cb4a48245db47a568bb740f5ec123575aae38eb961c9ce05f40c7f0526c9a')
    assert (len(r1_raw), sha(r1_raw)) == (5555, 'e96d7fdee27a1cf84b152799b51c7b6e541aeae714396665fa98985f5c81d05f')
    seed, r1 = json.loads(seed_raw), json.loads(r1_raw)
    assert encoded(seed) == seed_raw and encoded(r1) == r1_raw
    r2 = copy.deepcopy(r1)
    revision = copy.deepcopy(r1['metadata']['owned_revisions'][-1])
    revision.update(id='owned-r2', parent='owned-r1', rationale={'Value': {
        'text': 'Review explicit own and external occurrences', 'language': 'Missing', 'markup': 'Missing'}})
    r2['metadata']['owned_revisions'].append(revision)
    r2['metadata']['owned_heads'][0]['revision'] = 'owned-r2'
    r2['metadata']['catalogue_revision'] = 2
    pin = {'id': 'cdc-context', 'model_sha256': sha(encoded(context)),
           'source_ids': sorted(r['id'] for r in context['registry']['sources']),
           'release_ids': sorted(r['id'] for r in context['metadata']['releases'])}
    first = next(a for a in context['metadata']['assertions'] if a['definition']['Value']['text'] == 'greeting')
    entry = next(e for e in context['metadata']['entries'] if e['id'] == first['entry'])
    assert entry['spelling'] == {'Value': 'coi'}
    release = next(r for r in context['metadata']['releases'] if first['id'] in r['assertions'])
    attr = next(r for r in context['registry']['occurrences']
                if r['occurrence'] == {'snapshot': first['occurrence']['capture'], 'locator': first['occurrence']['locator']})
    catalog = {'source': entry['source'], 'release': release['id'],
               'native_entry_id': attr['native_entry_id'], 'native_assertion_id': attr['native_assertion_id'],
               'occurrence': first['occurrence']}
    target = {'context': pin['id'], 'target': catalog, 'entry': entry['id'], 'assertion': first['id']}
    rationale = lambda text: {'text': text, 'language': 'Missing', 'markup': 'Missing'}
    sense = {'id': 'cdc-reviewed-sense', 'revision': 'owned-r2', 'editor': 'editor-example',
             'rationale': rationale('Review two retained owned assertions and one supplied occurrence'),
             'kind': {'sense': {'members': [{'owned_assertion': 'owned-assertion-a'},
                                            {'owned_assertion': 'owned-assertion-b'}, {'external': target}]}}}
    link = {'id': 'cdc-reviewed-link', 'revision': 'owned-r2', 'editor': 'editor-example',
            'rationale': rationale('Link the reviewed sense to the pinned external occurrence'),
            'kind': {'link': {'origin': {'EditorialSense': sense['id']}, 'direction': 'Forward', 'target': target}}}
    curation = {'version': 1, 'dictionary': r2, 'contexts': [pin], 'records': [sense, link],
                'revisions': [{'revision': 'owned-r0', 'active': [], 'tombstones': []},
                              {'revision': 'owned-r1', 'active': [], 'tombstones': []},
                              {'revision': 'owned-r2', 'active': [sense['id'], link['id']], 'tombstones': []}]}
    context_input = {'id': pin['id'], 'dictionary': context}
    save('seed.json', seed_raw)
    save('r1.json', r1_raw)
    save('curation.json', encoded(curation))
    save('context.json', encoded(context_input))

    # Known nonoverlapping Tiny inputs: explicit declared compose semantics.
    # These disjoint input identities make keyed collapse irrelevant here; notice/lineage
    # exact dedup and extension multiplicity remain the established compose policy.
    view = copy.deepcopy(context)
    for key, rows in view['registry'].items():
        if isinstance(rows, list):
            rows.extend(copy.deepcopy(r2['registry'][key]))
            if key == 'lineage':
                view['registry'][key] = [v for i, v in enumerate(rows) if v not in rows[:i]]
    view['captures'].extend(copy.deepcopy(r2['captures']))
    for key, rows in view['metadata'].items():
        if isinstance(rows, list):
            rows.extend(copy.deepcopy(r2['metadata'][key]))
            if key == 'code_notices':
                view['metadata'][key] = [v for i, v in enumerate(rows) if v not in rows[:i]]
    assert not r2['metadata']['extensions']
    view['metadata']['catalogue_revision'] = max(context['metadata']['catalogue_revision'], 2)
    projected_sense = {'id': sense['id'], 'assertions': ['owned-assertion-a', 'owned-assertion-b', first['id']],
                       'editor': sense['editor'], 'revision': 'owned-r2', 'rationale': sense['rationale']}
    view['metadata']['editorial_senses'].append(projected_sense)
    view['metadata']['relations'].append({'id': link['id'], 'origin': {'EditorialSense': sense['id']},
        'kind': 'EditorialLink', 'direction': 'Forward', 'scope': 'Editorial', 'target': {'Catalog': catalog},
        'evidence': None, 'editorial_revision': 'owned-r2'})
    for record in (sense, link):
        view['metadata']['extensions'].append({'owner': record['id'], 'name': 'vlajmaji.owned-curation.v1',
            'value': encoded({'version': 1, 'record': record, 'contexts': [pin]}).decode(), 'evidence': None})
    save('expected-view.json', encoded(view))

    root = base / 'cli-journal'
    owned('init', ['init', root, '--input', base / 'seed.json'])
    owned('edit', ['edit', root, '--expected-head', 'owned-r0'], input_bytes=r1_raw)
    before = owned('history-before', ['history', root])
    schema_one_before = logical_snapshot(root)
    assert schema_one_before['user_version'] == 1
    save_logical_snapshot('schema1-before-migration', schema_one_before)
    # Hold an actual read snapshot through the attempted schema migration.
    blocker = sqlite3.connect('file:' + str(root / 'owned.sqlite') + '?mode=ro', uri=True)
    blocker.execute('BEGIN DEFERRED')
    blocker.execute('SELECT head_revision FROM owned_meta').fetchone()
    busy = owned('migration-held-reader', ['migrate-curation', root, '--expected-head', 'owned-r1'], expected=1)
    assert busy['code'] == 'owned_busy'
    blocker.close()
    assert logical_snapshot(root) == schema_one_before
    migration = owned('migrate', ['migrate-curation', root, '--expected-head', 'owned-r1'])
    assert migration == {'version': 1, 'source': 'owned-example', 'head': 'owned-r1',
                         'from_schema': 1, 'to_schema': 2, 'outcome': 'migrated'}
    assert owned('history-after', ['history', root]) == before
    schema_two_migrated = logical_snapshot(root)
    assert schema_two_migrated['application_id'] == schema_one_before['application_id']
    assert schema_two_migrated['user_version'] == 2
    for name, rows in schema_one_before['tables'].items():
        expected = copy.deepcopy(rows)
        if name == 'owned_meta':
            assert len(expected) == 1 and expected[0][2] == 1
            header = list(expected[0])
            header[2] = 2
            expected = [tuple(header)]
        assert schema_two_migrated['tables'][name] == expected, name
    assert all(row in schema_two_migrated['schema'] for row in schema_one_before['schema'])
    assert set(schema_two_migrated['tables']) - set(schema_one_before['tables']) == {
        'owned_context_pins', 'owned_curation_records', 'owned_curation_revisions', 'owned_curation_prefixes'}
    save_logical_snapshot('schema2-after-migration', schema_two_migrated)
    repeated = owned('migration-already-current', ['migrate-curation', root, '--expected-head', 'owned-r1'])
    assert repeated == {**migration, 'from_schema': 2, 'outcome': 'already_current'}
    assert logical_snapshot(root) == schema_two_migrated
    # Original pure-policy negatives use valid base graphs and the SAME
    # registered identities; their complete positive counterpart is curation.
    reserved = copy.deepcopy(curation)
    reserved_id = r2['metadata']['entries'][0]['id']
    reserved['records'][0]['id'] = reserved_id
    reserved['records'][1]['kind']['link']['origin'] = {'EditorialSense': reserved_id}
    reserved['revisions'][-1]['active'][0] = reserved_id
    wrong_editor = copy.deepcopy(curation)
    wrong_editor['dictionary']['registry']['contributors'].append({
        'id': 'cdc-other-editor', 'display_name': 'Explicit alternate test editor'})
    wrong_editor['records'][0]['editor'] = 'cdc-other-editor'
    newly_inactive = copy.deepcopy(curation)
    newly_inactive['revisions'][-1]['active'].remove(link['id'])
    for label, invalid in [('reserved-record-id', reserved), ('creation-editor-mismatch', wrong_editor),
                           ('new-record-inactive', newly_inactive)]:
        save(label + '.json', encoded(invalid))
        error = owned(label, ['curate', root, '--expected-head', 'owned-r1'],
                      expected=1, input_bytes=encoded(invalid))
        assert error['code'] == 'owned_invalid_curation'
        assert logical_snapshot(root) == schema_two_migrated
    owned('curate-stdin', ['curate', root, '--expected-head', 'owned-r1'], input_bytes=encoded(curation))
    assert owned('curation-r2', ['curation-show', root]) == curation
    for index, model in enumerate((seed, r1)):
        expected = {'version': 1, 'dictionary': model, 'contexts': [], 'records': [],
                    'revisions': curation['revisions'][:index+1]}
        assert owned('curation-r' + str(index), ['curation-show', root, '--revision', 'owned-r' + str(index)]) == expected
    head = owned('head-r2', ['head', root])
    assert head['head']['revision'] == revision and head['head']['state_sha256'] == sha(encoded(r2))
    curated_snapshot = logical_snapshot(root)
    save_logical_snapshot('schema2-curated', curated_snapshot)
    error = owned('stale-curate', ['curate', root, '--expected-head', 'owned-r1'], expected=1, input_bytes=encoded(curation))
    assert error['code'] == 'owned_conflict'
    assert logical_snapshot(root) == curated_snapshot
    assert owned('head-after-replay', ['head', root]) == head
    absent_path, full_path = base / 'absent.json', base / 'complete.json'
    absent = owned('absent', ['materialize', root, '--output', absent_path])
    assert absent['resolutions'] == [
        {'record': sense['id'], 'status': 'unavailable', 'reason': 'missing_context', 'member': 2, 'projected': False},
        {'record': link['id'], 'status': 'unresolved', 'reason': 'dependency_unavailable', 'member': None, 'projected': False}]
    assert absent_path.read_bytes() == encoded(r2)
    expected_resolutions = [{'record': r['id'], 'status': 'resolved', 'reason': None, 'member': None, 'projected': True} for r in (sense, link)]
    complete = owned('complete', ['materialize', root, '--context', base / 'context.json', '--output', full_path])
    assert complete == {'version': 1, 'source': 'owned-example', 'revision': 'owned-r2',
                        'output': native_path(full_path), 'output_sha256': sha(encoded(view)),
                        'resolutions': expected_resolutions}
    assert full_path.read_bytes() == encoded(view)
    changed = copy.deepcopy(context_input)
    changed['dictionary']['metadata']['catalogue_revision'] += 1
    save('wrong-context.json', encoded(changed))
    wrong_output = base / 'must-not-exist.json'
    error = owned('wrong-context', ['materialize', root, '--context', base / 'wrong-context.json', '--output', wrong_output], expected=1)
    assert error['code'] == 'owned_invalid_curation' and not wrong_output.exists()
    error = owned('existing-output', ['materialize', root, '--context', base / 'context.json', '--output', full_path], expected=1)
    assert error['code'] == 'owned_io' and full_path.read_bytes() == encoded(view)
    assert owned('final-head', ['head', root]) == head
    assert logical_snapshot(root) == curated_snapshot

    # Actual existing derived routes; all model-sized input stays in files.
    store = base / 'derived'
    run('dictionary-build', [cli, 'dictionary-build', full_path, store])
    entries = {e['id']: e for e in view['metadata']['entries']}
    assertions = {a['id']: a for a in view['metadata']['assertions']}
    captures = {encoded(c['identity']): c for c in view['captures']}
    attribution = {encoded({'capture': r['occurrence']['snapshot'], 'locator': r['occurrence']['locator']}): r
                   for r in view['registry']['occurrences']}
    snapshots = {encoded(r['snapshot']): r for r in view['registry']['snapshots']}

    def key_for(assertion):
        e, o = entries[assertion['entry']], assertion['occurrence']
        a = attribution[encoded(o)]
        sn = snapshots[encoded(o['capture'])]
        return {'version': 1, 'digest_policy': 'entry-assertion-serde-json-v1', 'role': 'assertion',
                'source': e['source'], 'entry_id': e['id'], 'assertion_id': assertion['id'], 'occurrence': o,
                'entry_sha256': sha(encoded(e)), 'assertion_sha256': sha(encoded(assertion)),
                'native_entry_id': a['native_entry_id'], 'native_assertion_id': a['native_assertion_id'],
                'contributor_id': a['contributor'], 'distributor_id': sn['distributor']}

    def assert_view_row(row, expected_id, compact):
        a = assertions[expected_id]
        o = a['occurrence']
        assert row['assertion'] == a and row['entry'] == entries[a['entry']]
        assert row['occurrence_id'] == tuple_id(o)
        assert row['payload_present'] is True and row['current_eligible'] is True
        assert row['occurrence_attribution'] == attribution[encoded(o)]
        capture = captures[encoded(o['capture'])]
        assert row['capture_metadata'] == capture['metadata']
        if compact:
            assert row['detail_ref'] == key_for(a)
            assert row['snapshot_attribution'] == snapshots[encoded(o['capture'])]
        else:
            assert row['raw'] == next(r for r in capture['occurrences'] if r['locator'] == o['locator'])
        assert row['release_ids'] == [r['id'] for r in view['metadata']['releases'] if expected_id in r['assertions']]
        assert row['owned_revision_ids'] == [r['id'] for r in view['metadata']['owned_revisions'] if expected_id in r['assertions']]
        assert row['tombstoned_by'] == []

    for name, relation, ids in [('local-sense', {'Local': {'EditorialSense': sense['id']}}, projected_sense['assertions']),
                                ('catalog', {'Catalog': catalog}, [first['id']])]:
        expected_entity = None if name == 'catalog' else {
            'role': 'editorial_sense', 'id': sense['id'],
            'entry_ids': [assertions[aid]['entry'] for aid in ids], 'assertion_ids': ids,
            'source_ids': [entries[assertions[aid]['entry']]['source'] for aid in ids],
            'occurrence': None, 'evidence': None, 'spelling': None}
        for fmt in ('full', 'compact', 'readable'):
            result = run(name + '-' + fmt, [cli, 'dictionary-query', store, '--request',
                         encoded({'operation': 'resolve', 'target': relation}).decode(), '--format', fmt])
            assert not result.stderr and result.stdout.endswith(b'\n') and not result.stdout.endswith(b'\n\n')
            if fmt != 'readable':
                body = json.loads(result.stdout)
                assert body['target'] == relation and body['status'] == 'Resolved'
                assert body['unverified'] == [] and [r['assertion']['id'] for r in body['confirmed']] == ids
                for row, aid in zip(body['confirmed'], ids):
                    assert_view_row(row, aid, fmt == 'compact')
                if name == 'local-sense':
                    assert body['requested_release'] is None
                    if fmt == 'full':
                        assert body['entity'] == {'EditorialSense': projected_sense}
                    else:
                        assert body['entity'] == expected_entity
                else:
                    assert body['entity'] is None
                    assert body['requested_release'] == (release if fmt == 'full' else summary(release))
            else:
                text = result.stdout.decode()
                assert text.startswith('Reference ' + encoded(relation).decode() + ': Resolved\n')
                lines = text.splitlines()
                assert lines[1].startswith('entity: ')
                assert json.loads(lines[1][len('entity: '):]) == expected_entity
                assert 'confirmed: ' + str(len(ids)) in lines and 'unverified: 0' in lines
                assert lines[-1].startswith('requested_release: ')
                assert json.loads(lines[-1].removeprefix('requested_release: ')) == (None if name == 'local-sense' else summary(release))
                for aid in ids:
                    a = assertions[aid]
                    pos = lines.index('  ' + aid + ' @ ' + tuple_id(a['occurrence']))
                    expected = {'spelling': entries[a['entry']]['spelling'], 'definition': a['definition'],
                                'eligibility': a['eligibility'], 'payload_present': True,
                                'current_eligible': True, 'detail': key_for(a)}
                    for offset, (label, value) in enumerate(expected.items(), 1):
                        prefix = '    ' + label + ': '
                        assert lines[pos+offset].startswith(prefix)
                        assert json.loads(lines[pos+offset][len(prefix):]) == value
                assert sum(line.startswith('  ') and not line.startswith('    ') for line in lines) == len(ids)

    for fmt in ('full', 'compact'):
        result = run('own-exact-' + fmt, [cli, 'dictionary-query', store, '--request',
                     '{"operation":"exact","spelling":"farvi"}', '--format', fmt])
        response = json.loads(result.stdout)
        hits = [h for g in response['groups'] for h in g['hits']]
        assert response['totals'] == {'occurrences': 2, 'distinct_spellings': 1, 'source_groups': 1}
        assert {h['assertion']['id'] for h in hits} == {'owned-assertion-a', 'owned-assertion-b'}
        for hit in hits:
            assert hit['entry'] == entries[hit['assertion']['entry']] and hit['assertion'] == assertions[hit['assertion']['id']]
    # Original retrospective selection predicate: immutable definitions survive
    # deselection/reactivation; old reads keep THEIR base and selection after r4.
    def next_selection(previous, revision_id, active, tombstones):
        data = copy.deepcopy(previous)
        model = data['dictionary']
        prior = model['metadata']['owned_revisions'][-1]
        new_revision = copy.deepcopy(prior)
        new_revision.update(id=revision_id, parent=prior['id'], rationale={'Value': {
            'text': 'Explicit curation selection ' + revision_id,
            'language': 'Missing', 'markup': 'Missing'}})
        model['metadata']['owned_revisions'].append(new_revision)
        model['metadata']['owned_heads'][0]['revision'] = revision_id
        model['metadata']['catalogue_revision'] += 1
        data['revisions'].append({'revision': revision_id, 'active': active, 'tombstones': tombstones})
        return data

    r3_curation = next_selection(curation, 'owned-r3', [sense['id']], [link['id']])
    r4_curation = next_selection(r3_curation, 'owned-r4', [sense['id'], link['id']], [])
    for prior, candidate in [('owned-r2', r3_curation), ('owned-r3', r4_curation)]:
        revision_id = candidate['dictionary']['metadata']['owned_revisions'][-1]['id']
        save(revision_id + '-curation.json', encoded(candidate))
        owned(revision_id + '-curate', ['curate', root, '--expected-head', prior],
              input_bytes=encoded(candidate))
    for revision_id, expected in [('owned-r2', curation), ('owned-r3', r3_curation),
                                  ('owned-r4', r4_curation)]:
        assert owned(revision_id + '-retrospective', ['curation-show', root, '--revision', revision_id]) == expected
    assert owned('r4-current-document', ['curation-show', root]) == r4_curation
    selected_snapshot = logical_snapshot(root)
    save_logical_snapshot('schema2-reactivated', selected_snapshot)
    for revision_id, candidate, link_active in [('owned-r3', r3_curation, False),
                                                ('owned-r4', r4_curation, True)]:
        expected_view = copy.deepcopy(view)
        for key in ('owned_revisions', 'owned_heads'):
            expected_view['metadata'][key] = copy.deepcopy(candidate['dictionary']['metadata'][key])
        expected_view['metadata']['catalogue_revision'] = max(
            context['metadata']['catalogue_revision'], candidate['dictionary']['metadata']['catalogue_revision'])
        if not link_active:
            expected_view['metadata']['relations'] = [r for r in expected_view['metadata']['relations']
                                                        if r['id'] != link['id']]
            expected_view['metadata']['extensions'] = [r for r in expected_view['metadata']['extensions']
                                                         if r['owner'] != link['id']]
        output = base / (revision_id + '-historical-view.json')
        report = owned(revision_id + '-historical-materialize', ['materialize', root, '--revision',
                       revision_id, '--context', base / 'context.json', '--output', output])
        expected_statuses = [expected_resolutions[0], expected_resolutions[1] if link_active else {
            'record': link['id'], 'status': 'inactive', 'reason': None, 'member': None, 'projected': False}]
        assert report == {'version': 1, 'source': 'owned-example', 'revision': revision_id,
                          'output': native_path(output), 'output_sha256': sha(encoded(expected_view)),
                          'resolutions': expected_statuses}
        assert output.read_bytes() == encoded(expected_view)
        assert logical_snapshot(root) == selected_snapshot
    # Read ONLY the newly created explicit Tiny journal, never a retained/default DB.
    database = root / 'owned.sqlite'
    assert root.is_dir() and not root.is_symlink()
    # owned_schema::FILE is exactly owned.sqlite; reject links before this new Tiny read.
    assert database.is_file() and not database.is_symlink()
    connection = sqlite3.connect('file:' + str(database) + '?mode=ro', uri=True)
    assert connection.execute('PRAGMA user_version').fetchone()[0] == 2
    assert connection.execute('SELECT COUNT(*) FROM owned_curation_revisions').fetchone()[0] == 5
    assert connection.execute('SELECT COUNT(*) FROM owned_curation_records').fetchone()[0] == 2
    connection.close()
    assert {str(p): sha(p.read_bytes()) for p in (example, cli)} == binary_hashes
    save('receipt.json', encoded({'status': 'PASS', 'root': str(base), 'children': len(records),
        'input_owned_assertions': ['owned-assertion-a', 'owned-assertion-b'],
        'mixed_sense_members': projected_sense['assertions'], 'materialized_sha256': sha(encoded(view)),
        'retrospective_revisions': ['owned-r2', 'owned-r3', 'owned-r4'],
        'retained_creation_revision': 'owned-r2',
        'source_binary_sha256': binary_hashes,
        'scope': 'owned synthetic public+CLI; no Full/default/benchmark/network',
        'environment': {k: env[k] for k in ('HOME', 'XDG_DATA_HOME', 'XDG_CONFIG_HOME', 'TMPDIR')}}) + b'\n')
    print('PASS public example + retained-owned mixed curation + actual derived reference routes;', len(records), 'direct children', flush=True)


if __name__ == '__main__':
    main()
