from pathlib import Path
import copy,hashlib,json,re,subprocess,time
base=Path('/tmp/vlajmaji-p01-a04-s01-cdc');root=base/'tiny-probe-qualification03';root.mkdir()
probe=Path('/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc01/target/release/examples/topology_probe')
sha=lambda b:hashlib.sha256(b).hexdigest()
assert sha(probe.read_bytes())=='e12ddf648d40f385a021591745b19c3e8b3ab8341ca88b780108bb83dbe1a215'
requests=json.loads((base/'full-freeze02/requests-full03.json').read_text())
docs={n:json.loads((base/'probe-inputs01'/n/'release.json').read_text())for n in ['lensisku-jbotci','llg','la-lojban','sutysisku']}
def witness(n):
 d=docs[n];a=d['metadata']['assertions'][0];e=next(e for e in d['metadata']['entries']if e['id']==a['entry']);r=next(r for r in d['metadata']['releases']if a['id']in r['assertions']);o=next(o for o in d['registry']['occurrences']if o['occurrence']['snapshot']==a['occurrence']['capture']and o['occurrence']['locator']==a['occurrence']['locator'])
 return {'Catalog':dict(source=e['source'],release=r['id'],native_entry_id=o['native_entry_id'],native_assertion_id=o['native_assertion_id'],occurrence=a['occurrence'])}
jb=docs['lensisku-jbotci']['metadata']['assertions'][0]['id']
for r in requests:
 name=r['id'].removeprefix('prime--')
 if name=='jbotci-local':r['target']={'Local':{'Assertion':jb}}
 elif name=='wrong-role':r['target']={'Local':{'Entry':jb}}
 elif name in ['jbotci-catalogue-native333','la-catalogue-native333','llg-component','sut-native-entry']:
  r['target']=witness({'jbotci-catalogue-native333':'lensisku-jbotci','la-catalogue-native333':'la-lojban','llg-component':'llg','sut-native-entry':'sutysisku'}[name])
 elif name=='missing':r['target']=witness('lensisku-jbotci');r['target']['Catalog']['native_assertion_id']='missing-native-assertion'
path=root/'requests-tiny03.json';path.write_text(json.dumps(requests,indent=2)+'\n')
manifest=base/'tiny-probe-qualification01/stores/run-manifest.json';dbs=sorted(manifest.parent.glob('*/dictionary.sqlite3'));assert len(dbs)==5
before={str(p):sha(p.read_bytes())for p in dbs};expected_wire=(base/'public-tiny01/expected-full-wire.json').read_bytes();expected=json.loads(expected_wire)
ids=lambda v:[h['assertion']['id']for g in v['groups']for h in g['hits']]
chains=json.loads((base/'full-freeze02/requests-full03-graph-controls.json').read_text())['page_chains']
records=[];results={}
for topology in ['unified','attached','fanout']:
 values=None
 for mode in ['run','batch']:
  argv=[str(probe),mode,'--manifest',str(manifest),'--topology',topology,'--requests',str(path)]
  if mode=='run':argv+=['--request','farvi-all']
  else:argv+=['--output-dir',str(root/(topology+'-batch'))]
  started=time.monotonic_ns();p=subprocess.run(argv,capture_output=True,timeout=60);elapsed=time.monotonic_ns()-started
  (root/(topology+'-'+mode+'.stdout')).write_bytes(p.stdout);(root/(topology+'-'+mode+'.stderr')).write_bytes(p.stderr)
  records.append(dict(topology=topology,mode=mode,argv=argv,exit=p.returncode,outer_ns=elapsed,stdout_sha256=sha(p.stdout),stderr_sha256=sha(p.stderr)))
  (root/'attempts.json').write_text(json.dumps(records,indent=2)+'\n');assert p.returncode==0,(topology,mode,p.stderr[-1000:])
  if mode=='run':assert p.stdout==expected_wire
  else:
   summary=json.loads(p.stdout);values={r['id']:json.loads(Path(r['path']).read_bytes())for r in summary['responses']};assert len(values)==27
   assert values['farvi-all']==expected
   for chain in chains.values():
    assert sum([ids(values[n])for n in chain],[])==ids(expected)
    assert values[chain[-1]]['next_cursor']is None
   assert values['page-0']['totals']==expected['totals']and values['page-0']['groups']==[]
   for source,count in [('lsk',2),('llg',1),('sut',2)]:assert values['farvi-'+source]['totals']['occurrences']==count and values['farvi-'+source]['native_candidate_occurrences']==5
   assert values['absent']['totals']['occurrences']==values['excluded-sut']['totals']['occurrences']==0
   for name in ['jbotci-local','jbotci-catalogue-native333','la-catalogue-native333','llg-component','sut-native-entry']:assert values[name]['status']=='Resolved',name
   assert values['wrong-role']['error']['code']=='invalid_reference'and values['missing']['status']=='NoMatch'
   assert values['unknown']['status']=='Unknown'and values['unresolved']['status']=='Unresolved'
 results[topology]=values
assert results['unified']==results['attached']==results['fanout']
after={str(p):sha(p.read_bytes())for p in dbs};assert before==after
report=dict(status='PASS',scope='current corrected profiler and full54 workload adapted to literal tiny inputs; no Full store operation or comparison round',source='4b4158c8a8a489ead639b73706b01061de17cfe1',probe_sha256=sha(probe.read_bytes()),requests_sha256=sha(path.read_bytes()),script_sha256=sha(Path(__file__).read_bytes()),requests=54,prime=27,measured=27,providers=3,positive_page_sizes=[1,2,3,8],full_wire_bytes=len(expected_wire),full_wire_sha256=sha(expected_wire),db_before=before,db_after=after,db_unchanged=True,attempts=records)
(root/'summary.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({k:v for k,v in report.items()if k not in ['db_before','db_after','attempts']}))
