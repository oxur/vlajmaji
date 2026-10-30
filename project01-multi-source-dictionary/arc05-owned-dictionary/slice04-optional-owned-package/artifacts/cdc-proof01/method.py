#!/usr/bin/env python3
"""UNEXECUTED S04 reproduction: same frozen consumer ONCE + original12CLI.

Requires explicit frozen author handback. No Cargo/build/target mutation/default/
Full/network. All expected outputs derive from preserved pre-input authorities.
"""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import tomllib


def encoded(value):
    return json.dumps(value, ensure_ascii=False, separators=(',', ':')).encode()


def sha(value):
    return hashlib.sha256(value).hexdigest()


def tuple_id(occurrence):
    values = (occurrence['capture']['namespace'], occurrence['capture']['sha256'], occurrence['locator'])
    return b''.join(len(s.encode()).to_bytes(8, 'big') + s.encode() for s in values).hex()


def native_path(path):
    return {'encoding': 'unix-bytes', 'hex': os.fsencode(path).hex(), 'utf8': str(path)}


def main():
    # Bind actual final author source, normal graph and compiled executable bodies.
    binding_path = Path(sys.argv[1])
    binding_raw = binding_path.read_bytes()
    binding = json.loads(binding_raw)
    source, arc = Path(binding['source_path']), Path(binding['arc_path'])
    cli, consumer = Path(binding['cli_path']), Path(binding['consumer_path'])
    def git(*args):
        return subprocess.check_output(['git', '--no-optional-locks', '-C', str(source), *args]).decode().strip()
    def check_pins():
        assert git('rev-parse', 'HEAD') == binding['head']
        assert git('rev-parse', 'HEAD^{tree}') == binding['tree']
        assert not git('status', '--porcelain', '--untracked-files=all')
        for p, digest in binding['file_sha256'].items():
            assert sha(Path(p).read_bytes()) == digest, p
        assert binding_path.read_bytes() == binding_raw
    assert str(cli) in binding['file_sha256'] and str(consumer) in binding['file_sha256']
    assert str(source/'Cargo.lock') in binding['file_sha256']
    assert str(Path(__file__)) in binding['file_sha256']
    check_pins()
    manifest = tomllib.loads(Path(binding['consumer_manifest']).read_text())
    assert manifest['package']['name'] == 'vlajmaji-a05-s04-owned-consumer-01'
    assert set(manifest['dependencies']) == {'vlajmaji-data-owned', 'vlajmaji-lexicon'}
    assert not manifest.get('dev-dependencies') and not manifest.get('build-dependencies')
    assert not manifest.get('target')
    lock = tomllib.loads(Path(binding['consumer_lock']).read_text())
    names = {p['name'] for p in lock['package']}
    assert not names & {'vlajmaji-store','vlajmaji-search','rusqlite','libsqlite3-sys','oxigraph'}
    root = next(p for p in lock['package'] if p['name'] == manifest['package']['name'])
    assert root['dependencies'] == ['vlajmaji-data-owned','vlajmaji-lexicon']
    # Honest source-derived entire-lock graph, not a fabricated Cargo-tree raw.
    graph = json.loads(Path(binding['consumer_lock_graph']).read_bytes())
    assert graph['kind']=='source-derived-private-lock-closure-not-original-cargo-tree-output'
    assert graph['packages']==lock['package']
    assert graph['manifest_sha256']==sha(Path(binding['consumer_manifest']).read_bytes())
    assert graph['lock_sha256']==sha(Path(binding['consumer_lock']).read_bytes())
    for p in (binding['consumer_source'],binding['consumer_manifest'],binding['consumer_lock'],binding['consumer_lock_graph']):
        assert p in binding['file_sha256']
    # Every original input SHA/length is checked before creating/running any child.
    d = arc/'slice04-optional-owned-package'/'artifacts'/'consumer-inputs'
    assert str(d/'oracles.json') in binding['file_sha256']
    oracles = json.loads((d/'oracles.json').read_bytes())
    input_authorities={}
    def original(row):
        path=Path(os.path.normpath(str(d/row['path'])))
        raw = path.read_bytes()
        assert len(raw) == row['bytes'] and sha(raw) == row['sha256'] and not raw.endswith(b'\n')
        input_authorities[str(path)]=raw
        return raw
    payload, seal = original(oracles['positive_snapshot']), original(oracles['positive_seal'])
    histories = [(original(row['dictionary']),original(row['curation'])) for row in oracles['history']]
    context_raw = original(oracles['cli_inputs']['context.json'])
    view_raw = original(oracles['cli_inputs']['expected-view.json'])
    for case in oracles['cases']:
        original(case['payload']);original(case['seal'])
    for case in oracles['seal_or_hash_cases']:original(case)
    assert len(input_authorities)==21
    cli_only={str(Path(os.path.normpath(str(d/row['path'])))) for row in oracles['cli_inputs'].values()}
    copied=set(binding['consumer_input_copies'])
    assert set(input_authorities)-cli_only <= copied <= set(input_authorities)
    assert all(p in binding['file_sha256'] for p in input_authorities)
    for original_path,copy_path in binding['consumer_input_copies'].items():
        assert copy_path in binding['file_sha256']
        assert Path(copy_path).read_bytes()==input_authorities[original_path]
    assert (len(view_raw),sha(view_raw)) == (17940,'bd662bd6734dda3a17d666fbb260163d17d84ee44df0906d84dd1d75c022bfb3')
    view = json.loads(view_raw)
    curation = json.loads(histories[2][1]); sense,link = curation['records']
    entries = {e['id']:e for e in view['metadata']['entries']}
    assertions = {a['id']:a for a in view['metadata']['assertions']}
    captures = {encoded(c['identity']):c for c in view['captures']}
    attribution = {encoded({'capture':r['occurrence']['snapshot'],'locator':r['occurrence']['locator']}):r for r in view['registry']['occurrences']}
    snapshots = {encoded(r['snapshot']):r for r in view['registry']['snapshots']}
    contributors = {r['id']:r for r in view['registry']['contributors']}
    distributors = {r['id']:r for r in view['registry']['distributors']}
    projected_sense = next(r for r in view['metadata']['editorial_senses'] if r['id'] == sense['id'])
    def key_for(a):
        e,o = entries[a['entry']],a['occurrence']; attr=attribution[encoded(o)]; sn=snapshots[encoded(o['capture'])]
        return {'version':1,'digest_policy':'entry-assertion-serde-json-v1','role':'assertion',
                'source':e['source'],'entry_id':e['id'],'assertion_id':a['id'],'occurrence':o,
                'entry_sha256':sha(encoded(e)),'assertion_sha256':sha(encoded(a)),
                'native_entry_id':attr['native_entry_id'],'native_assertion_id':attr['native_assertion_id'],
                'contributor_id':attr['contributor'],'distributor_id':sn['distributor']}
    def expected_hit(aid):
        a=assertions[aid];o=a['occurrence'];c=captures[encoded(o['capture'])]
        attr=attribution[encoded(o)];sn=snapshots[encoded(o['capture'])]
        result={'occurrence_id':tuple_id(o),'entry':entries[a['entry']],'assertion':a,'capture':c['metadata'],
                'raw':next(r for r in c['occurrences'] if r['locator']==o['locator']),
                'snapshot_attribution':sn,'occurrence_attribution':attr,'contributor':contributors.get(attr['contributor']),
                'distributor':distributors.get(sn['distributor']),
                'release_ids':[r['id'] for r in view['metadata']['releases'] if aid in r['assertions']],
                'contexts':[],'frames':[],'forms':[],'etymologies':[],'relations':view['metadata']['relations'],
                'editorial_senses':[projected_sense],'computations':[],'annotations':[],'extensions':view['metadata']['extensions'],
                'owned_revision_ids':[r['id'] for r in view['metadata']['owned_revisions'] if aid in r['assertions']],
                'tombstoned_by':[]}
        assert len(result)==21
        return result
    for field in ('contexts','frames','forms','etymologies','computations','annotations'):
        assert view['metadata'][field] == []
    assert [r['id'] for r in view['metadata']['relations']] == [link['id']]
    assert [r['owner'] for r in view['metadata']['extensions']] == [sense['id'],link['id']]
    hits = {aid:expected_hit(aid) for aid in ('owned-assertion-a','owned-assertion-b')}
    keys = {aid:key_for(assertions[aid]) for aid in hits}
    assert all(len(k)==13 for k in keys.values())
    parent_path = os.environ['PATH']
    base = Path(tempfile.mkdtemp(prefix='vlajmaji-a05-s04-cdc-'))
    print('owned proof root:',base,flush=True)
    env = os.environ.copy()
    for key in ('HOME','XDG_DATA_HOME','XDG_CONFIG_HOME','TMPDIR'):
        p=base/key.lower();p.mkdir();env[key]=str(p)
    env.pop('RELOJBAN_RESEARCH_ROOT',None)
    assert env['PATH']==parent_path
    def save(name,raw):
        with (base/name).open('xb') as f:f.write(raw)
    save('binding.json',binding_raw)
    save('actual-env.json',encoded({k:env.get(k) for k in ('PATH','CARGO_HOME','RUSTUP_HOME','HOME','XDG_DATA_HOME','XDG_CONFIG_HOME','TMPDIR')}))
    for name,raw in [('snapshot.json',payload),('seal.json',seal),('context.json',context_raw),('expected-view.json',view_raw)]:save(name,raw)
    # All expectations above are pre-input authorities, not returned materialization.
    records=[]
    def run(label,args):
        args=list(map(str,args))
        try:
            result=subprocess.run(args,cwd=base,env=env,stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=120)
        except subprocess.TimeoutExpired as error:
            out,err=error.stdout or b'',error.stderr or b''
            save(label+'.stdout',out);save(label+'.stderr',err)
            records.append({'label':label,'argv':args,'exit':None,'expected_exit':0,'timed_out':True,
                            'stdout_bytes':len(out),'stdout_sha256':sha(out),'stderr_bytes':len(err),'stderr_sha256':sha(err)})
            (base/'commands.json').write_bytes(encoded(records)+b'\n');raise
        save(label+'.stdout',result.stdout);save(label+'.stderr',result.stderr)
        records.append({'label':label,'argv':args,'exit':result.returncode,'expected_exit':0,'timed_out':False,
                        'stdout_bytes':len(result.stdout),'stdout_sha256':sha(result.stdout),
                        'stderr_bytes':len(result.stderr),'stderr_sha256':sha(result.stderr)})
        (base/'commands.json').write_bytes(encoded(records)+b'\n')
        assert result.returncode==0,(label,result.returncode,result.stderr)
        return result
    def json_cli(label,args):
        r=run(label,[cli,*args]);assert not r.stderr and r.stdout.endswith(b'\n') and not r.stdout.endswith(b'\n\n')
        return json.loads(r.stdout)
    # Exactly one execution of the SAME author-built consumer, no build/qualifier.
    run('frozen-consumer',[consumer,*binding.get('consumer_args',[])])
    pair=['--input',base/'snapshot.json','--seal',base/'seal.json']
    checked=json_cli('check',['owned','check',*pair])
    assert checked=={'version':1,'source':'owned-example','head':'owned-r4','journal_schema':2,'revision_count':5,
                     'snapshot_bytes':len(payload),'snapshot_sha256':sha(payload),'seal_bytes':len(seal),'seal_sha256':sha(seal)}
    journal=base/'journal'
    assert json_cli('import',['owned','import',journal,*pair])=={'version':1,'outcome':'imported','source':'owned-example',
                     'head':'owned-r4','journal_schema':2,'revision_count':5,'snapshot_sha256':sha(payload)}
    old=json.loads(histories[1][0])
    assert json_cli('show-r1',['owned','show',journal,'--revision','owned-r1'])=={
        'version':1,'source':'owned-example','commit':{'sequence':1,'revision':old['metadata']['owned_revisions'][-1],
                                                    'state_sha256':sha(histories[1][0])},'dictionary':old}
    assert json_cli('curation-r3',['owned','curation-show',journal,'--revision','owned-r3'])==json.loads(histories[3][1])
    materialized=base/'view.json'
    assert json_cli('materialize-r2',['owned','materialize',journal,'--revision','owned-r2','--context',base/'context.json','--output',materialized])=={
        'version':1,'source':'owned-example','revision':'owned-r2','output':native_path(materialized),'output_sha256':sha(view_raw),
        'resolutions':[{'record':r['id'],'status':'resolved','reason':None,'member':None,'projected':True} for r in (sense,link)]}
    assert materialized.read_bytes()==view_raw
    store=base/'derived';run('build',[cli,'dictionary-build',materialized,store])
    request='{"operation":"exact","spelling":"farvi"}'
    for fmt in ('full','compact'):
        body=json_cli('exact-'+fmt,['dictionary-query',store,'--request',request,'--format',fmt])
        assert body['totals']=={'occurrences':2,'distinct_spellings':1,'source_groups':1}
        assert len(body['groups'])==1
        group=body['groups'][0]
        assert group['source']==next(r for r in view['registry']['sources'] if r['id']=='owned-example')
        assert [h['assertion']['id'] for h in group['hits']]==list(hits)
        for actual in group['hits']:
            aid=actual['assertion']['id'];expected=hits[aid]
            if fmt=='full':assert actual==expected
            else:
                compact={k:expected[k] for k in ('occurrence_id','entry','assertion','capture','snapshot_attribution','occurrence_attribution','contributor','distributor','release_ids')}
                compact['detail_ref']=keys[aid]
                assert actual==compact
    r=run('exact-readable',[cli,'dictionary-query',store,'--request',request,'--format','readable'])
    assert not r.stderr and r.stdout.endswith(b'\n')
    lines=r.stdout.decode().splitlines();assert lines[0]=='Exact "farvi": 2 occurrences, 1 spellings, 1 sources'
    for aid in hits:
        a=assertions[aid];pos=lines.index('  '+aid+' @ '+tuple_id(a['occurrence']))
        for offset,(label,value) in enumerate((('spelling',entries[a['entry']]['spelling']),('definition',a['definition']),('detail',keys[aid])),1):
            prefix='    '+label+': ';assert lines[pos+offset].startswith(prefix)
            assert json.loads(lines[pos+offset][len(prefix):])==value
    relation={'Local':{'EditorialSense':sense['id']}}
    response=json_cli('sense-compact',['dictionary-query',store,'--request',encoded({'operation':'resolve','target':relation}).decode(),'--format','compact'])
    ids=projected_sense['assertions']
    assert response['target']==relation and response['status']=='Resolved' and response['unverified']==[] and response['requested_release'] is None
    assert response['entity']=={'role':'editorial_sense','id':sense['id'],'entry_ids':[assertions[x]['entry'] for x in ids],
                               'assertion_ids':ids,'source_ids':[entries[assertions[x]['entry']]['source'] for x in ids],
                               'occurrence':None,'evidence':None,'spelling':None}
    assert [r['assertion']['id'] for r in response['confirmed']]==ids
    for row,aid in zip(response['confirmed'],ids):
        a=assertions[aid];o=a['occurrence'];capture=captures[encoded(o['capture'])]
        assert row=={'entry':entries[a['entry']],'assertion':a,'occurrence_id':tuple_id(o),'occurrence_attribution':attribution[encoded(o)],
                     'capture_metadata':capture['metadata'],'snapshot_attribution':snapshots[encoded(o['capture'])],
                     'payload_present':True,'current_eligible':True,'release_ids':[r['id'] for r in view['metadata']['releases'] if aid in r['assertions']],
                     'owned_revision_ids':[r['id'] for r in view['metadata']['owned_revisions'] if aid in r['assertions']],
                     'tombstoned_by':[],'detail_ref':key_for(a)}
    aid='owned-assertion-a';reference=keys[aid]
    for fmt in ('full','readable'):
        r=run('detail-'+fmt,[cli,'dictionary-detail',store,'--reference',encoded(reference).decode(),'--format',fmt])
        assert not r.stderr and r.stdout.endswith(b'\n')
        if fmt=='full':
            body=json.loads(r.stdout)
            assert set(body)=={'version','projection_policy','detail_ref','status','operational_reason','state_sha256','snapshot_sha256','hit','current_eligible'}
            assert body['version']==1 and body['projection_policy']=='complete-dictionary-hit-v1'
            assert body['detail_ref']==reference and body['status']=='Resolved' and body['operational_reason'] is None
            assert body['current_eligible'] is True and body['hit']==hits[aid]
        else:
            fields,hit_json=r.stdout.decode().split('\nhit: ',1);lines=fields.splitlines()
            assert lines[0]=='Detail '+aid+': Resolved'
            for index,(label,value) in enumerate((('reason',None),('current_eligible',True),('reference',reference)),2):
                prefix=label+': ';assert lines[index].startswith(prefix)
                assert json.loads(lines[index][len(prefix):])==value
            assert json.loads(hit_json)==hits[aid]
    assert len(records)==13
    check_pins()
    save('execution.json',encoded({'ok':True,'source':binding['head'],'tree':binding['tree'],'file_sha256':binding['file_sha256'],
                                  'children':records,'literal_path':parent_path,'consumer_runs':1,'cli_children':12}))
    print('PASS S04 independent reproduction: same consumer ONCE, 12CLI children',flush=True)

if __name__=='__main__':main()
