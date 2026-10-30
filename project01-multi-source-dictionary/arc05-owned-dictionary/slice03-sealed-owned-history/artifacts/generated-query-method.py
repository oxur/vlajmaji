#!/usr/bin/env python3
"""Scratch S03 generated-input query/detail check; run after owned_snapshot example.
Usage: python3 METHOD.py CLI INPUTS_DIR JOURNAL_ROOT IMPORTED_ROOT SCRATCH_DIR
All request and hit/detail expectations are assembled from the example's input
files before this script invokes materialize or any dictionary query.
"""
import hashlib, json, os, subprocess, sys
from pathlib import Path

cli, inputs, root, imported_root, scratch = map(Path, sys.argv[1:])
assert not scratch.exists()
scratch.mkdir(parents=True)
children = scratch/'children'
children.mkdir()
receipt_path = scratch/'children.jsonl'
root.parent.mkdir(parents=True, exist_ok=True)
imported_root.parent.mkdir(parents=True, exist_ok=True)
assert not root.exists() and not imported_root.exists()
def compact(v): return json.dumps(v, ensure_ascii=False, separators=(',', ':')).encode()
def sha(b): return hashlib.sha256(b).hexdigest()
def run(label, argv):
    command = [str(cli), *map(str, argv)]
    stdout_path, stderr_path = children/f'{label}.stdout', children/f'{label}.stderr'
    receipt = {'label':label, 'argv':command, 'timeout_seconds':120,
               'stdout_path':str(stdout_path), 'stderr_path':str(stderr_path)}
    try:
        p = subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                           check=False, timeout=120)
    except subprocess.TimeoutExpired as error:
        stdout = error.stdout or b''
        stderr = error.stderr or b''
        stdout_path.write_bytes(stdout)
        stderr_path.write_bytes(stderr)
        receipt.update(exit_code=None, timed_out=True, stdout_bytes=len(stdout),
                       stderr_bytes=len(stderr))
        with receipt_path.open('a', encoding='utf-8') as stream:
            stream.write(json.dumps(receipt, ensure_ascii=False, sort_keys=True)+'\n')
            stream.flush()
            os.fsync(stream.fileno())
        raise RuntimeError(f'{label}: timed out after 120 seconds; exit is unknown') from error
    stdout_path.write_bytes(p.stdout)
    stderr_path.write_bytes(p.stderr)
    receipt.update(exit_code=p.returncode, timed_out=False, stdout_bytes=len(p.stdout),
                   stderr_bytes=len(p.stderr))
    with receipt_path.open('a', encoding='utf-8') as stream:
        stream.write(json.dumps(receipt, ensure_ascii=False, sort_keys=True)+'\n')
        stream.flush()
        os.fsync(stream.fileno())
    if p.returncode or p.stderr:
        raise RuntimeError(f'{label}: exit={p.returncode}, stderr={p.stderr!r}')
    if not p.stdout.endswith(b'\n') or p.stdout.endswith(b'\n\n'):
        raise RuntimeError(f'{label}: expected one final LF')
    return p.stdout

def read(name): return json.loads((inputs/name).read_bytes())
def owned(label, *args): return json.loads(run(label,['owned',*args]))
base = read('base.json')
seed = read('seed.json')
r1 = read('r1.json')
r2 = read('r2.json')
r3 = read('r3.json')
context_input = read('context.json')
context = context_input['dictionary']
doc_r2 = read('r2-curation.json')
doc_r3 = read('r3-curation.json')
curation = read('r4-curation.json')
records = curation['records']
sense_record = next(r for r in records if 'sense' in r['kind'])
link_record = next(r for r in records if 'link' in r['kind'])
sense_kind = sense_record['kind']['sense']
link_kind = link_record['kind']['link']
context_member = next(m['external'] for m in sense_kind['members'] if 'external' in m)
foreign_id = context_member['assertion']
owned_ids = [m['owned_assertion'] for m in sense_kind['members'] if 'owned_assertion' in m]
assert len(owned_ids) == 2 and len({*owned_ids, foreign_id}) == 3
# Use only pre-query INPUTS to identify every lexical row, key and expected order.
all_docs = [base, context]
def rows(doc, section, field): return doc[section][field]
def by_id(section, field):
    return {row['id']: row for doc in all_docs for row in rows(doc, section, field)}
entries = by_id('metadata', 'entries')
assertions = by_id('metadata', 'assertions')
releases = [row for doc in all_docs for row in rows(doc, 'metadata', 'releases')]
owned_revisions = rows(base, 'metadata', 'owned_revisions')
registry = {field: [row for doc in all_docs for row in rows(doc, 'registry', field)]
            for field in ('sources','contributors','distributors','snapshots','occurrences')}
captures = [row for doc in all_docs for row in doc['captures']]

def occ_key(o): return compact(o)
def tuple_id(o):
    parts = (o['capture']['namespace'], o['capture']['sha256'], o['locator'])
    return b''.join(len(x.encode()).to_bytes(8,'big') + x.encode() for x in parts).hex()
def lookup_occ(o):
    identity = {'snapshot':o['capture'], 'locator':o['locator']}
    return next(row for row in registry['occurrences'] if occ_key(row['occurrence']) == occ_key(identity))
def lookup_snapshot(o): return next(row for row in registry['snapshots'] if row['snapshot'] == o['capture'])
def lookup_capture(o): return next(row for row in captures if row['identity'] == o['capture'])
def detail_key(a):
    e, o = entries[a['entry']], a['occurrence']
    attr, snap = lookup_occ(o), lookup_snapshot(o)
    return {'version':1,'digest_policy':'entry-assertion-serde-json-v1','role':'assertion',
            'source':e['source'],'entry_id':e['id'],'assertion_id':a['id'],'occurrence':o,
            'entry_sha256':sha(compact(e)),'assertion_sha256':sha(compact(a)),
            'native_entry_id':attr['native_entry_id'],'native_assertion_id':attr['native_assertion_id'],
            'contributor_id':attr['contributor'],'distributor_id':snap['distributor']}
def raw_for(a):
    cap = lookup_capture(a['occurrence'])
    return next(r for r in cap['occurrences'] if r['locator'] == a['occurrence']['locator'])

def summary(release):
    value = {k:release[k] for k in ('id','component','distributor','upstream_revision','distributor_revision',
        'captured_at','time_precision','schema_version','normalizer','normalizer_version','captures','accounting')}
    value['component_id'] = value.pop('component')
    value['distributor_id'] = value.pop('distributor')
    value.update(diagnostics_count=len(release['diagnostics']), selected=release['selected'],
        availability=release['availability'], content_notices_count=len(release['content_notices']),
        content_notice_licenses=[n['license'] for n in release['content_notices']],
        content_notice_locators=[n['locator'] for n in release['content_notices']],
        redistribution=release['redistribution'].lower(), redistribution_evidence=release['redistribution_evidence'])
    return value

def merged_hit(a):
    o = a['occurrence']; entry = entries[a['entry']]
    attr, snap, cap = lookup_occ(o), lookup_snapshot(o), lookup_capture(o)
    # This example has no contexts/forms/frames/etymologies/computations/annotations;
    # assert those source INPUT vectors before treating the expected vectors as empty.
    for doc in all_docs:
        for field in ('contexts','frames','forms','etymologies','computations','annotations'):
            assert not doc['metadata'][field], field
    sense_ids = [*owned_ids, foreign_id]
    editorial = {'id':sense_record['id'],'assertions':sense_ids,'editor':sense_record['editor'],
                 'revision':sense_record['revision'],'rationale':sense_record['rationale']}
    relation = {'id':link_record['id'],'origin':link_kind['origin'],'kind':'EditorialLink',
                'direction':link_kind['direction'],'scope':'Editorial',
                'target':{'Catalog':link_kind['target']['target']},'evidence':None,
                'editorial_revision':link_record['revision']}
    pin = curation['contexts'][0]
    for doc in all_docs:
        for field in ('relations','editorial_senses','extensions'):
            assert not doc['metadata'][field], field
    extensions = []
    for rec in (sense_record, link_record):
        payload = {'version':1,'record':rec,'contexts':[pin]}
        extensions.append({'owner':rec['id'],'name':'vlajmaji.owned-curation.v1',
                           'value':compact(payload).decode(), 'evidence':None})
    return {'occurrence_id':tuple_id(o),'entry':entry,'assertion':a,'capture':cap['metadata'],
        'raw':raw_for(a),'snapshot_attribution':snap,'occurrence_attribution':attr,
        'contributor':next((x for x in registry['contributors'] if x['id']==attr['contributor']),None),
        'distributor':next((x for x in registry['distributors'] if x['id']==snap['distributor']),None),
        'release_ids':[r['id'] for r in releases if a['id'] in r['assertions']],
        'contexts':[],'frames':[],'forms':[],'etymologies':[],'relations':[relation],
        'editorial_senses':[editorial],'computations':[],'annotations':[],'extensions':extensions,
        'owned_revision_ids':[r['id'] for r in owned_revisions if a['id'] in r['assertions']],
        'tombstoned_by':[r['id'] for r in owned_revisions if a['id'] in r['tombstones']]}

# Exact inputs and source/spelling/tuple order are bound before any CLI materialization/query.
expected_hit = {aid:merged_hit(assertions[aid]) for aid in [*owned_ids, foreign_id]}
assert all(len(hit) == 21 for hit in expected_hit.values())
expected_detail = {aid:detail_key(assertions[aid]) for aid in expected_hit}
assert all(len(key) == 13 for key in expected_detail.values())
def spelling_key(aid):
    spelling = entries[assertions[aid]['entry']]['spelling']
    return spelling.get('Value', '')
full_order = sorted(expected_hit, key=lambda aid: (
    entries[assertions[aid]['entry']]['source'], spelling_key(aid),
    tuple_id(assertions[aid]['occurrence']), aid))
assert full_order == [foreign_id, *owned_ids]
compact_order = [*owned_ids, foreign_id]
local_target = {'Local':{'EditorialSense':sense_record['id']}}
catalog_target = {'Catalog':link_kind['target']['target']}
# Full and compact resolution requests are written from the pre-input curation.
def query(label, target, fmt):
    request = compact({'operation':'resolve','target':target}).decode()
    return run(label,['dictionary-query',scratch/'derived-store','--request',request,'--format',fmt])

# The separate CLI root consumes the example's emitted INPUTS. Build all five
# base states and curation cutoffs from those files before any query/materialize.
states = [seed,r1,r2,r3,base]
revision_ids = [d['metadata']['owned_revisions'][-1]['id'] for d in states]
empty0 = {'version':1,'dictionary':seed,'contexts':[],'records':[],
          'revisions':[{'revision':revision_ids[0],'active':[],'tombstones':[]}]}
empty1 = {'version':1,'dictionary':r1,'contexts':[],'records':[],
          'revisions':[{'revision':revision_ids[0],'active':[],'tombstones':[]},
                       {'revision':revision_ids[1],'active':[],'tombstones':[]}]}
curations = [empty0,empty1,doc_r2,doc_r3,curation]
owned('cli-init','init',root,'--input',inputs/'seed.json')
owned('cli-edit','edit',root,'--expected-head',revision_ids[0],'--input',inputs/'r1.json')
owned('cli-migrate','migrate-curation',root,'--expected-head',revision_ids[1])
for index, doc in enumerate((doc_r2,doc_r3,curation),2):
    owned('cli-curate-'+str(index),'curate',root,'--expected-head',revision_ids[index-1],
          '--input',inputs/f'r{index}-curation.json')
for index,(state,doc) in enumerate(zip(states,curations)):
    show = owned('cli-show-'+str(index),'show',root,'--revision',revision_ids[index])
    assert show['dictionary'] == state
    assert show['commit']['sequence'] == index
    assert show['commit']['revision']['id'] == revision_ids[index]
    assert show['commit']['state_sha256'] == sha(compact(state))
    assert owned('cli-curation-show-'+str(index),'curation-show',root,'--revision',revision_ids[index]) == doc
history = owned('cli-history','history',root)
assert history['head'] == revision_ids[-1]
assert [(row['sequence'],row['revision']['id'],row['state_sha256']) for row in history['commits']] == [
    (index,revision_ids[index],sha(compact(state))) for index,state in enumerate(states)]

# CLI lifecycle is deliberately after all query targets, hit bodies and detail
# keys above are fixed from base/context/curation inputs.
export_json, export_seal = scratch/'export.json', scratch/'export.seal.json'
owned('cli-export','export',root,'--output',export_json,'--seal',export_seal)
repeat_json, repeat_seal = scratch/'repeat.json', scratch/'repeat.seal.json'
owned('cli-export-repeat','export',root,'--output',repeat_json,'--seal',repeat_seal)
assert export_json.read_bytes() == repeat_json.read_bytes()
assert export_seal.read_bytes() == repeat_seal.read_bytes()
assert export_json.read_bytes() == (inputs/'snapshot.json').read_bytes()
assert export_seal.read_bytes() == (inputs/'seal.json').read_bytes()
owned('cli-check','check','--input',export_json,'--seal',export_seal)
owned('cli-import','import',imported_root,'--input',export_json,'--seal',export_seal)
owned('cli-import-replay','import',imported_root,'--input',export_json,'--seal',export_seal)
for revision, expected in ((revision_ids[0],seed),(revision_ids[-1],base)):
    output = scratch/f'model-{revision}.json'
    owned('cli-snapshot-model-'+revision,'snapshot-model','--input',export_json,'--seal',export_seal,
          '--revision',revision,'--output',output)
    assert output.read_bytes() == compact(expected)
zero_context = scratch/'zero-context.json'
zero_report = owned('cli-materialize-zero','materialize',root,'--revision',revision_ids[-1],
                    '--output',zero_context)
assert zero_context.read_bytes() == compact(base)
assert zero_report['resolutions'] == [
    {'record':sense_record['id'],'status':'unavailable','reason':'missing_context','member':2,'projected':False},
    {'record':link_record['id'],'status':'unresolved','reason':'dependency_unavailable','member':None,'projected':False}]
materialized = scratch/'materialized-r4.json'
resolved_report = json.loads(run('materialize',['owned','materialize',root,'--revision',revision_ids[-1],
                   '--context',inputs/'context.json','--output',materialized]))
assert resolved_report['resolutions'] == [
    {'record':sense_record['id'],'status':'resolved','reason':None,'member':None,'projected':True},
    {'record':link_record['id'],'status':'resolved','reason':None,'member':None,'projected':True}]
run('dictionary-build',['dictionary-build',materialized,scratch/'derived-store'])
for name, target, ids in (('local',local_target,full_order),('catalog',catalog_target,[foreign_id])):
    for fmt in ('full','compact','readable'):
        raw = query(name+'-'+fmt,target,fmt)
        if fmt == 'readable':
            text = raw.decode()
            assert text.startswith('Reference '+compact(target).decode()+': Resolved\n')
            lines = text.splitlines()
            assert 'confirmed: '+str(len(ids)) in lines and 'unverified: 0' in lines
            assert lines[1].startswith('entity: ')
            if name == 'local':
                entity_expected = {'role':'editorial_sense','id':sense_record['id'],
                    'entry_ids':[assertions[aid]['entry'] for aid in compact_order],
                    'assertion_ids':compact_order,
                    'source_ids':[entries[assertions[aid]['entry']]['source'] for aid in compact_order],
                    'occurrence':None,'evidence':None,'spelling':None}
            else:
                entity_expected = None
            assert json.loads(lines[1].removeprefix('entity: ')) == entity_expected
            seen = [line.split(' @ ',1)[0].removeprefix('  ') for line in lines
                    if line.startswith('  ') and ' @ ' in line and not line.startswith('    ')]
            assert seen == ids
            for aid in ids:
                hit = expected_hit[aid]
                row = '  '+aid+' @ '+hit['occurrence_id']
                pos = lines.index(row)
                expected_labels = (('spelling',hit['entry']['spelling']),
                    ('definition',hit['assertion']['definition']),('eligibility',hit['assertion']['eligibility']),
                    ('payload_present',True),('current_eligible',True),('detail',expected_detail[aid]))
                for offset,(label,value) in enumerate(expected_labels,1):
                    prefix='    '+label+': '
                    assert lines[pos+offset].startswith(prefix)
                    assert json.loads(lines[pos+offset][len(prefix):]) == value
        else:
            body = json.loads(raw)
            assert body['target'] == target and body['status'] == 'Resolved' and not body['unverified']
            order = compact_order if fmt == 'compact' and name == 'local' else ids
            rows = body['confirmed']
            assert [r['assertion']['id'] for r in rows] == order
            for row, aid in zip(rows, order):
                if fmt == 'full':
                    hit = expected_hit[aid]
                    full_expected = {
                      'entry':hit['entry'], 'assertion':hit['assertion'],
                      'occurrence_id':hit['occurrence_id'],
                      'occurrence_attribution':hit['occurrence_attribution'], 'raw':hit['raw'],
                      'capture_metadata':hit['capture'], 'payload_present':True,
                      'current_eligible':True, 'release_ids':hit['release_ids'],
                      'owned_revision_ids':hit['owned_revision_ids'],
                      'tombstoned_by':hit['tombstoned_by']}
                    assert row == full_expected
                else:
                    compact_expected = {k:expected_hit[aid][k] for k in (
                      'entry','assertion','occurrence_id','occurrence_attribution','snapshot_attribution',
                      'release_ids','owned_revision_ids','tombstoned_by')}
                    compact_expected.update(capture_metadata=expected_hit[aid]['capture'],payload_present=True,
                        current_eligible=True,detail_ref=expected_detail[aid])
                    # Compact rows use `capture_metadata` and not full raw bytes.
                    assert row == compact_expected
            if name == 'local':
                assert body['requested_release'] is None
                if fmt == 'full':
                    assert body['entity'] == {'EditorialSense': {
                        'id':sense_record['id'], 'assertions':[*owned_ids,foreign_id],
                        'editor':sense_record['editor'], 'revision':sense_record['revision'],
                        'rationale':sense_record['rationale']}}
                else:
                    assert body['entity'] == {
                        'role':'editorial_sense','id':sense_record['id'],
                        'entry_ids':[assertions[aid]['entry'] for aid in compact_order],
                        'assertion_ids':compact_order,
                        'source_ids':[entries[assertions[aid]['entry']]['source'] for aid in compact_order],
                        'occurrence':None,'evidence':None,'spelling':None}
            else:
                assert body['entity'] is None
                release_id = catalog_target['Catalog']['release']
                release = next(r for r in releases if r['id']==release_id)
                assert body['requested_release'] == (release if fmt=='full' else summary(release))
        if fmt == 'readable':
            request_release = next(line for line in lines if line.startswith('requested_release: '))
            if name == 'local':
                requested_expected = None
            else:
                release = next(r for r in releases if r['id']==catalog_target['Catalog']['release'])
                requested_expected = summary(release)
            assert json.loads(request_release.removeprefix('requested_release: ')) == requested_expected

# Exact lookup confirms the two owned INPUT assertions, and detail returns the complete 21 fields.
for fmt in ('full','compact','readable'):
    raw = run('exact-'+fmt,['dictionary-query',scratch/'derived-store','--request',
        '{"operation":"exact","spelling":"farvi"}','--format',fmt])
    if fmt == 'readable':
        text=raw.decode(); assert text.startswith('Exact "farvi": 2 occurrences, 1 spellings, 1 sources\n')
        lines=text.splitlines()
        seen = [line.split(' @ ',1)[0].removeprefix('  ') for line in lines
                if line.startswith('  ') and ' @ ' in line and not line.startswith('    ')]
        assert seen == owned_ids
        for aid in owned_ids:
            pos=lines.index('  '+aid+' @ '+expected_hit[aid]['occurrence_id'])
            for offset, label, value in (
                (1,'spelling',expected_hit[aid]['entry']['spelling']),
                (2,'definition',expected_hit[aid]['assertion']['definition']),
                (3,'detail',expected_detail[aid])):
                prefix = '    '+label+': '
                assert lines[pos+offset].startswith(prefix)
                assert json.loads(lines[pos+offset][len(prefix):]) == value
    else:
        body=json.loads(raw); hits=[h for g in body['groups'] for h in g['hits']]
        assert body['totals']=={'occurrences':2,'distinct_spellings':1,'source_groups':1}
        assert {h['assertion']['id'] for h in hits}==set(owned_ids)
        assert [h['assertion']['id'] for h in hits] == owned_ids
        for h in hits:
            aid=h['assertion']['id']
            if fmt=='full':
                assert h == expected_hit[aid]
            else:
                compact_expected = {k:expected_hit[aid][k] for k in (
                    'occurrence_id','entry','assertion','capture','snapshot_attribution',
                    'occurrence_attribution','contributor','distributor','release_ids')}
                compact_expected['detail_ref'] = expected_detail[aid]
                assert h == compact_expected
for aid in expected_hit:
    for fmt in ('full','readable'):
        raw=run('detail-'+aid+'-'+fmt,['dictionary-detail',scratch/'derived-store','--reference',
            compact(expected_detail[aid]).decode(),'--format',fmt])
        if fmt=='full':
            body=json.loads(raw)
            assert set(body)=={'version','projection_policy','detail_ref','status','operational_reason',
                'state_sha256','snapshot_sha256','hit','current_eligible'}
            assert body['version']==1 and body['projection_policy']=='complete-dictionary-hit-v1'
            assert body['detail_ref']==expected_detail[aid] and body['status']=='Resolved'
            assert body['operational_reason'] is None and body['current_eligible'] is True
            assert len(body['state_sha256'])==64 and len(body['snapshot_sha256'])==64
            assert body['hit']==expected_hit[aid]
        else:
            text=raw.decode(); assert text.startswith('Detail '+aid+': Resolved\n')
            field_lines,hit_json=text.split('\nhit: ',1)
            field_lines=field_lines.splitlines()
            for offset,(label,value) in enumerate((('reason',None),('current_eligible',True),
                    ('reference',expected_detail[aid])),2):
                prefix=label+': '
                assert field_lines[offset].startswith(prefix)
                assert json.loads(field_lines[offset][len(prefix):]) == value
            assert json.loads(hit_json)==expected_hit[aid]
print('PASS generated-input dictionary query/detail checks')
