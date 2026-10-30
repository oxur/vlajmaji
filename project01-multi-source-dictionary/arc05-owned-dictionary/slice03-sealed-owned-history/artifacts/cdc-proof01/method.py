#!/usr/bin/env python3
"""UNEXECUTED S03 independent proof. Requires frozen source/quiet target handback.

Only explicit newly-created Tiny journals may be opened. Expected bytes are
pre-implementation INPUT authorities. No Cargo, default store, Full corpus,
network, benchmark observer, automatic retry or source mutation.
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


def sha(value):
    return hashlib.sha256(value).hexdigest()


def tuple_id(occurrence):
    values = (occurrence['capture']['namespace'], occurrence['capture']['sha256'], occurrence['locator'])
    return b''.join(len(s.encode()).to_bytes(8, 'big') + s.encode() for s in values).hex()


def native_path(path):
    return {'encoding': 'unix-bytes', 'hex': os.fsencode(path).hex(), 'utf8': str(path)}


def summary(release):
    result = {'id': release['id'], 'component_id': release['component'], 'distributor_id': release['distributor']}
    for key in ('upstream_revision', 'distributor_revision', 'captured_at', 'time_precision',
                'schema_version', 'normalizer', 'normalizer_version', 'captures', 'accounting'):
        result[key] = release[key]
    result.update(diagnostics_count=len(release['diagnostics']), selected=release['selected'],
                  availability=release['availability'], content_notices_count=len(release['content_notices']),
                  content_notice_licenses=[n['license'] for n in release['content_notices']],
                  content_notice_locators=[n['locator'] for n in release['content_notices']],
                  redistribution=release['redistribution'].lower(), redistribution_evidence=release['redistribution_evidence'])
    return result


def main():
    # All bindings must be supplied from the actual accepted source handback.
    cli, example, source, arc, expected_head, expected_tree, cli_sha, example_sha = sys.argv[1:]
    cli, example, source, arc = map(Path, (cli, example, source, arc))
    def git(*args):
        return subprocess.check_output(['git', '-C', str(source), *args]).decode().strip()
    assert git('rev-parse', 'HEAD') == expected_head
    assert git('rev-parse', 'HEAD^{tree}') == expected_tree
    assert not git('status', '--porcelain', '--untracked-files=all')
    assert sha(cli.read_bytes()) == cli_sha and sha(example.read_bytes()) == example_sha
    binary_pins = {str(cli): cli_sha, str(example): example_sha}
    parent_path = os.environ['PATH']  # Literal captured PATH; no dedup/reordering.
    base = Path(tempfile.mkdtemp(prefix='vlajmaji-a05-s03-cdc-'))
    print('owned proof root:', base, flush=True)
    env = os.environ.copy()
    for key in ('HOME', 'XDG_DATA_HOME', 'XDG_CONFIG_HOME', 'TMPDIR'):
        path = base / key.lower()
        path.mkdir()
        env[key] = str(path)
    env.pop('RELOJBAN_RESEARCH_ROOT', None)
    assert env['PATH'] == parent_path
    records = []
    def save(name, data):
        with (base / name).open('xb') as handle:
            handle.write(data)
    save('actual-env.json', encoded({k:env.get(k) for k in ('PATH','CARGO_HOME','RUSTUP_HOME','HOME','XDG_DATA_HOME','XDG_CONFIG_HOME','TMPDIR')}))
    def run(label, args, expected=0, input_bytes=None):
        args = list(map(str, args))
        try:
            result = subprocess.run(args, cwd=base, env=env, input=input_bytes,
                                    stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=120)
        except subprocess.TimeoutExpired as error:
            stdout, stderr = error.stdout or b'', error.stderr or b''
            save(label + '.stdout', stdout)
            save(label + '.stderr', stderr)
            records.append({'label': label, 'argv': args, 'exit': None,
                            'expected_exit': expected, 'timed_out': True,
                            'stdout_bytes': len(stdout), 'stdout_sha256': sha(stdout),
                            'stderr_bytes': len(stderr), 'stderr_sha256': sha(stderr)})
            (base / 'commands.json').write_bytes(encoded(records) + b'\n')
            raise
        save(label + '.stdout', result.stdout)
        save(label + '.stderr', result.stderr)
        records.append({'label': label, 'argv': args, 'exit': result.returncode,
                        'expected_exit': expected, 'stdout_bytes': len(result.stdout),
                        'stdout_sha256': sha(result.stdout), 'stderr_bytes': len(result.stderr),
                        'stderr_sha256': sha(result.stderr)})
        (base / 'commands.json').write_bytes(encoded(records) + b'\n')
        assert result.returncode == expected, (label, result.returncode, result.stderr)
        return result
    def owned(label, args, expected=0):
        result = run(label, [cli, 'owned', *args], expected)
        if expected:
            assert not result.stdout and result.stderr.endswith(b'\n'), label
            return json.loads(result.stderr)['error']
        assert not result.stderr and result.stdout.endswith(b'\n') and not result.stdout.endswith(b'\n\n'), label
        return json.loads(result.stdout)
    def sql_state(root):
        root.relative_to(base)
        database = root / 'owned.sqlite'
        assert root.is_dir() and not root.is_symlink()
        assert database.is_file() and not database.is_symlink()
        with sqlite3.connect('file:' + str(database) + '?mode=ro', uri=True) as db:
            schema = db.execute("SELECT type,name,tbl_name,sql FROM sqlite_master WHERE name NOT LIKE 'sqlite_%' ORDER BY type,name").fetchall()
            names = [row[1] for row in schema if row[0] == 'table']
            allowed = {'owned_meta', 'owned_records', 'owned_commits', 'owned_commit_sections',
                       'owned_context_pins', 'owned_curation_records', 'owned_curation_revisions', 'owned_curation_prefixes'}
            assert set(names) <= allowed
            return (db.execute('PRAGMA application_id').fetchone()[0],
                    db.execute('PRAGMA user_version').fetchone()[0], schema,
                    {name: sorted(db.execute('SELECT * FROM "' + name + '"').fetchall(), key=repr) for name in names})

    n = arc / 'slice03-sealed-owned-history' / 'artifacts'
    schema1 = (n / 'schema1-golden' / 'snapshot.json').read_bytes()
    seal1 = (n / 'schema1-golden' / 'seal.json').read_bytes()
    schema2 = (n / 'schema2-golden' / 'snapshot.json').read_bytes()
    seal2 = (n / 'schema2-golden' / 'seal.json').read_bytes()
    alternate = (n / 'schema1-same-head-alternate' / 'snapshot.json').read_bytes()
    alt_seal = (n / 'schema1-same-head-alternate' / 'seal.json').read_bytes()
    for raw, length, digest in (
        (schema1,8040,'a16d6f1c3112bb62fa3dd091d6a972e17c0f084dc433f4f3110584faef090a05'),
        (seal1,319,'9e87f3df1a18dffc1f4f3e4418d64f76fb56e3653ff213d895320702ce090d21'),
        (schema2,15705,'2d4925e6baf72044b71cae7a78f466412276b3d837765a6f0b1ed59fe58c3d8a'),
        (seal2,353,'412de31a528c5bd77a31cfeba3bfbdb2806c3f15d5adb363aca1a629ef82573e'),
        (alternate,8040,'a85fd0340209ebd49161aded45cceaa06f4773e9de7358cf14c1d3e707f236f7'),
        (alt_seal,319,'88e4fd5cd3939ad5b3533d2309b264e64ddbc115768fd8cd91362282b48cf6f5')):
        assert len(raw) == length and sha(raw) == digest and not raw.endswith(b'\n')
    for label, raw in [('schema1.json',schema1),('schema1.seal.json',seal1),('schema2.json',schema2),
                       ('schema2.seal.json',seal2),('alternate.json',alternate),('alternate.seal.json',alt_seal)]:
        save(label, raw)
    proof2 = arc / 'slice02-reviewed-curation' / 'artifacts' / 'cdc-proof02' / 'tmpdir' / 'vlajmaji-a05-s02-cdc-cgznzofy'
    s01 = arc / 'slice01-owned-journal' / 'artifacts' / 'cdc-proof' / 'journey' / 'inputs'
    inputs = [(s01/'seed.json').read_bytes(), (s01/'edit.json').read_bytes()]
    curated = [json.loads((proof2/name).read_bytes()) for name in ('curation.json','owned-r3-curation.json','owned-r4-curation.json')]
    inputs += [encoded(doc['dictionary']) for doc in curated]
    oracle = json.loads((n / 'history-golden-oracles.json').read_bytes())
    for index, (raw, row) in enumerate(zip(inputs, oracle['states'])):
        assert len(raw) == row['dictionary_bytes'] and sha(raw) == row['state_sha256']
    # Schema2 backfills empty r0/r1 curation histories as actual retained states.
    selected = json.loads(schema2)['curation']['revisions']
    old_curations = [{'version':1,'dictionary':json.loads(inputs[index]),'contexts':[],'records':[],
                      'revisions':selected[:index+1]} for index in range(2)] + curated
    view_raw = (proof2/'expected-view.json').read_bytes()
    assert (len(view_raw), sha(view_raw)) == (17940,'bd662bd6734dda3a17d666fbb260163d17d84ee44df0906d84dd1d75c022bfb3')
    view = json.loads(view_raw)
    context_raw = (proof2/'context.json').read_bytes()
    context_input = json.loads(context_raw)
    context = context_input['dictionary']
    curation = curated[0]
    sense, link = curation['records']
    first = next(a for a in context['metadata']['assertions'] if a['definition']['Value']['text'] == 'greeting')
    catalog = sense['kind']['sense']['members'][2]['external']['target']
    release = next(r for r in context['metadata']['releases'] if r['id'] == catalog['release'])
    projected_sense = next(r for r in view['metadata']['editorial_senses'] if r['id'] == sense['id'])
    save('context.json', context_raw)
    save('writer-seed.json', inputs[0])
    save('writer-r1.json', inputs[1])
    for index, candidate in enumerate(curated,2):
        save('writer-r'+str(index)+'-curation.json',encoded(candidate))
    # All expected authorities above are loaded BEFORE any product child.
    run('public-example', [example, base/'public-inputs', base/'public-root', base/'public-import'])

    # ORIGINAL writer path, independently of copied import rows: its old private
    # curation hasher must agree with all five pre-input pure/golden hashes.
    writer_root = base/'writer-root'
    owned('writer-init',['init',writer_root,'--input',base/'writer-seed.json'])
    owned('writer-edit',['edit',writer_root,'--expected-head','owned-r0','--input',base/'writer-r1.json'])
    owned('writer-migrate',['migrate-curation',writer_root,'--expected-head','owned-r1'])
    for index in range(2,5):
        owned('writer-curate-'+str(index),['curate',writer_root,'--expected-head','owned-r'+str(index-1),
                                        '--input',base/('writer-r'+str(index)+'-curation.json')])
    writer_before = sql_state(writer_root)
    with sqlite3.connect('file:'+str(writer_root/'owned.sqlite')+'?mode=ro',uri=True) as db:
        actual_base = db.execute('SELECT revision_id,state_sha256 FROM owned_commits ORDER BY sequence').fetchall()
        actual_curation = db.execute('SELECT p.revision_id,p.contexts_length,p.records_length,p.curation_sha256 '
                                     'FROM owned_curation_prefixes p JOIN owned_commits c ON c.revision_id=p.revision_id '
                                     'ORDER BY c.sequence').fetchall()
    assert actual_base == [(row['revision'],row['state_sha256']) for row in oracle['states']]
    assert actual_curation == [(row['revision'],row['contexts_length'],row['records_length'],row['curation_sha256'])
                               for row in oracle['states']]
    written_payload,written_seal=base/'writer-export.json',base/'writer-export.seal.json'
    owned('writer-export',['export',writer_root,'--output',written_payload,'--seal',written_seal])
    assert written_payload.read_bytes() == schema2 and written_seal.read_bytes() == seal2
    assert sql_state(writer_root) == writer_before

    def pair_args(label):
        return ['--input',base/(label+'.json'),'--seal',base/(label+'.seal.json')]
    for label, raw, seal, schema, count, head in [('schema1',schema1,seal1,1,2,'owned-r1'),
                                                ('schema2',schema2,seal2,2,5,'owned-r4'),
                                                ('alternate',alternate,alt_seal,1,2,'owned-r1')]:
        report = owned(label+'-check',['check',*pair_args(label)])
        assert report == {'version':1,'source':'owned-example','head':head,'journal_schema':schema,
                          'revision_count':count,'snapshot_bytes':len(raw),'snapshot_sha256':sha(raw),
                          'seal_bytes':len(seal),'seal_sha256':sha(seal)}
    for label, raw, schema, count, head in [('schema1',schema1,1,2,'owned-r1'),('schema2',schema2,2,5,'owned-r4')]:
        root = base/(label+'-root')
        for attempt in range(2):
            before = None if attempt == 0 else sql_state(root)
            report = owned(label+'-import-'+str(attempt),['import',root,*pair_args(label)])
            assert report == {'version':1,'outcome':'imported' if attempt == 0 else 'already_present',
                              'source':'owned-example','head':head,'journal_schema':schema,
                              'revision_count':count,'snapshot_sha256':sha(raw)}
            if before is not None:
                assert sql_state(root) == before
        assert sql_state(root)[1] == schema
        frozen = sql_state(root)
        for rep in range(2):
            output, seal_output = base/(label+'-export-'+str(rep)+'.json'),base/(label+'-export-'+str(rep)+'.seal.json')
            report = owned(label+'-export-'+str(rep),['export',root,'--output',output,'--seal',seal_output])
            expected_seal = seal1 if schema == 1 else seal2
            assert output.read_bytes() == raw and seal_output.read_bytes() == expected_seal
            assert report == {'version':1,'source':'owned-example','head':head,'journal_schema':schema,
                              'revision_count':count,'output':native_path(output),'output_bytes':len(raw),
                              'output_sha256':sha(raw),'seal':native_path(seal_output),'seal_bytes':len(expected_seal),
                              'seal_sha256':sha(expected_seal)}
        for index, expected in enumerate(inputs[:count]):
            revision = 'owned-r'+str(index)
            show = owned(label+'-show-'+str(index),['show',root,'--revision',revision])
            expected_model = json.loads(expected)
            assert show == {'version':1,'source':'owned-example',
                            'commit':{'sequence':index,'revision':expected_model['metadata']['owned_revisions'][-1],
                                      'state_sha256':sha(expected)},'dictionary':expected_model}
            output = base/(label+'-model-'+str(index)+'.json')
            report = owned(label+'-model-'+str(index),['snapshot-model',*pair_args(label),'--revision',revision,'--output',output])
            assert output.read_bytes() == expected
            assert report == {'version':1,'source':'owned-example','revision':revision,'journal_schema':schema,
                              'output':native_path(output),'output_bytes':len(expected),'output_sha256':sha(expected)}
        assert sql_state(root) == frozen
    root = base/'schema2-root'
    frozen = sql_state(root)
    for index, expected in enumerate(old_curations):
        assert owned('curation-old-'+str(index),['curation-show',root,'--revision','owned-r'+str(index)]) == expected
    for label, destination in [('alternate',base/'schema1-root'),('schema2',base/'schema1-root'),('schema1',root)]:
        before = sql_state(destination)
        error = owned('conflict-'+label+'-'+destination.name,['import',destination,*pair_args(label)],expected=1)
        assert error['code'] == 'owned_import_conflict'
        assert sql_state(destination) == before
    # Strict load occurs before ROOT IO and before output creation.
    damaged = bytearray(schema2); damaged[-2] ^= 1
    save('damaged.json',bytes(damaged))
    bad_root = base/'must-not-create'
    error = owned('damaged-before-root',['import',bad_root,'--input',base/'damaged.json','--seal',base/'schema2.seal.json'],expected=1)
    assert error['code'] == 'owned_invalid_snapshot' and not bad_root.exists()
    existing_output = base/'schema2-export-0.json'
    error = owned('existing-output',['export',root,'--output',existing_output,'--seal',base/'new-seal.json'],expected=1)
    assert error['code'] == 'owned_io' and existing_output.read_bytes() == schema2
    assert sql_state(root) == frozen
    full_path = base/'materialized-r2.json'
    materialized = owned('materialized-r2',['materialize',root,'--revision','owned-r2','--context',base/'context.json','--output',full_path])
    assert materialized == {'version':1,'source':'owned-example','revision':'owned-r2',
                            'output':native_path(full_path),'output_sha256':sha(view_raw),
                            'resolutions':[{'record':r['id'],'status':'resolved','reason':None,'member':None,'projected':True}
                                           for r in (sense,link)]}
    assert full_path.read_bytes() == view_raw

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
                row_ids = ids
                if fmt == 'full' and name == 'local-sense':
                    # Fixed INPUT-derived Full order; entity/member order stays ids.
                    row_ids = [first['id'], 'owned-assertion-a', 'owned-assertion-b']
                    assert set(row_ids) == set(ids)
                    foreign_entry = entries[assertions[first['id']]['entry']]
                    own_a = assertions['owned-assertion-a']
                    own_b = assertions['owned-assertion-b']
                    assert foreign_entry['source'] < entries[own_a['entry']]['source']
                    assert entries[own_a['entry']]['source'] == entries[own_b['entry']]['source']
                    assert entries[own_a['entry']]['spelling'] == entries[own_b['entry']]['spelling']
                    assert tuple_id(own_a['occurrence']) < tuple_id(own_b['occurrence'])
                assert body['unverified'] == [] and [r['assertion']['id'] for r in body['confirmed']] == row_ids
                for row, aid in zip(body['confirmed'], row_ids):
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

    readable = run('own-exact-readable',[cli,'dictionary-query',store,'--request',
                   '{"operation":"exact","spelling":"farvi"}','--format','readable'])
    text = readable.stdout.decode()
    assert text.startswith('Exact "farvi": 2 occurrences, 1 spellings, 1 sources\n')
    lines = text.splitlines()
    for aid in ('owned-assertion-a','owned-assertion-b'):
        assertion = assertions[aid]
        pos = lines.index('  '+aid+' @ '+tuple_id(assertion['occurrence']))
        for offset,(label,value) in enumerate((('spelling',entries[assertion['entry']]['spelling']),
                                               ('definition',assertion['definition']),('detail',key_for(assertion))),1):
            prefix = '    '+label+': '
            assert lines[pos+offset].startswith(prefix)
            assert json.loads(lines[pos+offset][len(prefix):]) == value

    # Whole21-field hits are derived from the fixed INPUT view before detail calls.
    contributors = {row['id']:row for row in view['registry']['contributors']}
    distributors = {row['id']:row for row in view['registry']['distributors']}
    for aid in ('owned-assertion-a','owned-assertion-b'):
        assertion = assertions[aid]
        occurrence = assertion['occurrence']
        capture = captures[encoded(occurrence['capture'])]
        attr = attribution[encoded(occurrence)]
        snap = snapshots[encoded(occurrence['capture'])]
        expected_hit = {'occurrence_id':tuple_id(occurrence),'entry':entries[assertion['entry']],
                        'assertion':assertion,'capture':capture['metadata'],
                        'raw':next(row for row in capture['occurrences'] if row['locator'] == occurrence['locator']),
                        'snapshot_attribution':snap,'occurrence_attribution':attr,
                        'contributor':contributors.get(attr['contributor']),
                        'distributor':distributors.get(snap['distributor']),
                        'release_ids':[row['id'] for row in view['metadata']['releases'] if aid in row['assertions']],
                        'contexts':[],'frames':[],'forms':[],'etymologies':[],
                        'relations':view['metadata']['relations'],
                        'editorial_senses':[projected_sense],'computations':[],'annotations':[],
                        'extensions':view['metadata']['extensions'],
                        'owned_revision_ids':[row['id'] for row in view['metadata']['owned_revisions'] if aid in row['assertions']],
                        'tombstoned_by':[]}
        assert len(expected_hit) == 21
        for field in ('contexts','frames','forms','etymologies','computations','annotations'):
            assert view['metadata'][field] == []
        assert [row['id'] for row in view['metadata']['relations']] == [link['id']]
        assert [row['owner'] for row in view['metadata']['extensions']] == [sense['id'],link['id']]
        reference = key_for(assertion)
        assert len(reference) == 13
        for fmt in ('full','readable'):
            result = run(aid+'-detail-'+fmt,[cli,'dictionary-detail',store,'--reference',encoded(reference).decode(),'--format',fmt])
            assert not result.stderr and result.stdout.endswith(b'\n')
            if fmt == 'full':
                response = json.loads(result.stdout)
                assert set(response) == {'version','projection_policy','detail_ref','status','operational_reason',
                                         'state_sha256','snapshot_sha256','hit','current_eligible'}
                assert response['version'] == 1 and response['projection_policy'] == 'complete-dictionary-hit-v1'
                assert response['detail_ref'] == reference and response['status'] == 'Resolved'
                assert response['operational_reason'] is None and response['current_eligible'] is True
                assert response['hit'] == expected_hit
            else:
                output = result.stdout.decode()
                assert output.startswith('Detail '+aid+': Resolved\n')
                field_lines,hit_json = output.split('\nhit: ',1)
                field_lines = field_lines.splitlines()
                for index,(label,expected) in enumerate((('reason',None),('current_eligible',True),('reference',reference)),2):
                    prefix = label+': '
                    assert field_lines[index].startswith(prefix)
                    assert json.loads(field_lines[index][len(prefix):]) == expected
                assert json.loads(hit_json) == expected_hit

    # All operations above are read-only after imports. Binding is checked again.
    assert sql_state(root) == frozen
    assert git('rev-parse','HEAD') == expected_head and git('rev-parse','HEAD^{tree}') == expected_tree
    assert not git('status','--porcelain','--untracked-files=all')
    assert {str(p):sha(p.read_bytes()) for p in (cli,example)} == binary_pins
    save('execution.json',encoded({'ok':True,'source':expected_head,'tree':expected_tree,
                                  'binary_pins':binary_pins,'children':records,'literal_path':parent_path}))
    print('PASS independent sealed-history proof:',len(records),'children',flush=True)


if __name__ == '__main__':
    main()
