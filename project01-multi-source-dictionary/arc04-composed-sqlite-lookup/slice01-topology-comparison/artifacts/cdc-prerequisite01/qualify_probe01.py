from pathlib import Path
import hashlib,json,subprocess,sys
base=Path('/tmp/vlajmaji-p01-a04-s01-cdc')
root=base/'tiny-probe-qualification01';root.mkdir()
inputs=base/'probe-inputs01';probe=Path('/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc01/target/release/examples/topology_probe')
sha=lambda b:hashlib.sha256(b).hexdigest()
assert sha(probe.read_bytes())=='06ac76fd68c6df0e75089ecf193403307eb6dd2bc94c6461905aecc98ff813e4'
meta=['assertions','entries','contexts','frames','forms','etymologies','relations','extensions','collections','components','releases','code_notices','owned_revisions','owned_heads','editorial_senses','computations','annotations']
reg=['sources','contributors','distributors','aliases','lineage','snapshots']
def population(d):
    out={k:len(d['metadata'][k]) for k in meta}
    out.update({k:len(d['registry'][k]) for k in reg})
    out.update(captures=len(d['captures']),occurrences=sum(len(c['occurrences']) for c in d['captures']),native_fields=sum(len(o['fields']) for c in d['captures'] for o in c['occurrences']))
    assert len(out)==26
    return out
packs=[];docs={}
for name in ['lensisku-jbotci','llg','la-lojban','sutysisku']:
    raw=(inputs/name/'release.json').read_bytes();docs[name]=json.loads(raw)
    packs.append(dict(id=name,release=str(inputs/name/'release.json'),seal=str(inputs/name/'seal.json'),expected_sha256=sha(raw),expected_population=population(docs[name]),store=name))
unified_raw=(inputs/'expected-unified.json').read_bytes()
config=dict(generation='generation-one',output_root=str(root/'stores'),unified_state_sha256=sha(unified_raw),unified_expected_population=population(json.loads(unified_raw)),packs=packs)
config_path=root/'prepare.json';config_path.write_text(json.dumps(config,indent=2)+'\n')
jb=docs['lensisku-jbotci']['metadata']['assertions'][0]['id'];la=docs['la-lojban']['metadata']['assertions'][0]['id']
def exact(name,**kwargs):return dict(id=name,operation='exact',spelling='farvi',sources=[],page_size=None,**kwargs)
def ref(name,target,**kwargs):return dict(id=name,operation='reference',target=target,page_size=None,**kwargs)
requests=[exact('prime-exact',prime=True),ref('prime-reference',{'Local':{'Assertion':jb}},prime=True),exact('full'),dict(id='page0',operation='exact',spelling='farvi',sources=[],page_size=0)]
for n in range(5):
    r=dict(id=f'page{n+1}',operation='exact',spelling='farvi',sources=[],page_size=1)
    if n:r['cursor_from']=f'page{n}'
    requests.append(r)
for source in ['LSK','LLG','SUT']:requests.append(dict(id=source,operation='exact',spelling='farvi',sources=[source],page_size=None))
requests.extend([dict(id='absent',operation='exact',spelling='no-such-tiny-spelling',sources=[],page_size=None),dict(id='excluded',operation='exact',spelling='zoizoizoi',sources=['SUT'],page_size=None),ref('valid-ref',{'Local':{'Assertion':jb}}),ref('wrong-role',{'Local':{'Entry':jb}},expected_error='invalid_reference'),ref('following-ref',{'Local':{'Assertion':la}}),ref('unknown', 'Unknown'),ref('unresolved',{'Unresolved':'explicit tiny unresolved reference'})])
request_path=root/'requests.json';request_path.write_text(json.dumps(requests,indent=2)+'\n')
records=[]
def run(label,argv,timeout=60):
    p=subprocess.run(argv,capture_output=True,timeout=timeout)
    (root/(label+'-stdout.log')).write_bytes(p.stdout);(root/(label+'-stderr.log')).write_bytes(p.stderr)
    records.append(dict(label=label,argv=argv,exit_code=p.returncode,stdout_sha256=sha(p.stdout),stderr_sha256=sha(p.stderr)))
    (root/'execution-attempts.json').write_text(json.dumps(records,indent=2)+'\n')
    assert p.returncode==0,(label,p.returncode,p.stderr[-1500:]);return p
wrapper=base/'wrapper-controls02/stage_guard_time.py'
run('prepare',[sys.executable,'-B',str(wrapper),'--timeout','60','--max-rss-bytes','1073741824','--log',str(root/'prepare-child.log'),'--',str(probe),'prepare','--manifest',str(config_path)])
manifest=root/'stores/run-manifest.json'
dbs=sorted((root/'stores').glob('*/dictionary.sqlite3'));assert len(dbs)==5
before={str(p):sha(p.read_bytes())for p in dbs}
expected_wire=(base/'public-tiny01/expected-full-wire.json').read_bytes();expected=json.loads(expected_wire)
results={}
for topology in ['unified','attached','fanout']:
    p=run(topology+'-run',[str(probe),'run','--manifest',str(manifest),'--topology',topology,'--requests',str(request_path),'--request','full']);assert p.stdout==expected_wire,topology
    out=root/(topology+'-batch')
    p=run(topology+'-batch',[str(probe),'batch','--manifest',str(manifest),'--topology',topology,'--requests',str(request_path),'--output-dir',str(out)])
    summary=json.loads(p.stdout);values={r['id']:json.loads(Path(r['path']).read_bytes())for r in summary['responses']}
    assert len(values)==len(requests)-2
    assert values['full']==expected
    assert values['page0']['totals']==expected['totals'] and not values['page0']['groups'] and values['page0']['next_cursor'] is None
    ids=lambda value:[h['assertion']['id']for g in value['groups']for h in g['hits']]
    assert sum([ids(values[f'page{i}']) for i in range(1,6)],[])==ids(expected)
    assert values['page5']['next_cursor'] is None
    for source,count in [('LSK',2),('LLG',1),('SUT',2)]:assert values[source]['totals']['occurrences']==count and values[source]['native_candidate_occurrences']==5
    assert values['absent']['totals']['occurrences']==values['excluded']['totals']['occurrences']==0
    assert values['valid-ref']['status']==values['following-ref']['status']=='Resolved'
    assert values['wrong-role']['expected_error'] and values['wrong-role']['error']['code']=='invalid_reference' and 'sources' in values['wrong-role']['error']
    assert values['unknown']['status']=='Unknown' and values['unresolved']['status']=='Unresolved'
    results[topology]=dict(responses=len(values),full_wire_bytes=len(expected_wire),full_wire_sha256=sha(expected_wire),all_values=values)
reference=results['unified']['all_values']
assert all(v['all_values']==reference for v in results.values())
after={str(p):sha(p.read_bytes())for p in dbs};assert before==after
for value in results.values():del value['all_values']
report=dict(status='PASS',scope='independent actual Synthetic prepare-five-stores/run/batch qualification; not Full corpus, topology timing or selection',probe_sha256=sha(probe.read_bytes()),script_sha256=sha(Path(__file__).read_bytes()),config_sha256=sha(config_path.read_bytes()),requests_sha256=sha(request_path.read_bytes()),predeclared_category_count=26,request_count=len(requests),priming_count=2,providers=results,db_before=before,db_after=after,db_unchanged=True,attempts=records)
(root/'summary.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps({k:v for k,v in report.items()if k not in ['db_before','db_after','attempts']}))
