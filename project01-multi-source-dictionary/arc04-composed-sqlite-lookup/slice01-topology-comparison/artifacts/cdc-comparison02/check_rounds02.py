"""Finite full-output and individual-request analysis. No DB or product execution."""
from pathlib import Path
import hashlib,importlib.util,json,re,statistics,sys
base=Path('/tmp/vlajmaji-p01-a04-s01-cdc')
spec=importlib.util.spec_from_file_location('metrics',base/'metrics02.py');metrics=importlib.util.module_from_spec(spec);spec.loader.exec_module(metrics)
sha=lambda b:hashlib.sha256(b).hexdigest()

def validate_batch(summary,expect,chains):
 paths={x['id']:Path(x['path'])for x in summary['responses']};fullraw=paths['farvi-all'].read_bytes();assert len(fullraw)==expect['bytes']and sha(fullraw)==expect['sha256']
 full=json.loads(fullraw);ids=lambda v:[h['assertion']['id']for g in v['groups']for h in g['hits']]
 hits=lambda v:[h for g in v['groups']for h in g['hits']]
 fullids=ids(full);assert len(fullids)==5 and len(set(fullids))==5 and len(full['groups'])==3 and full['totals']['occurrences']==5 and full['native_candidate_occurrences']==5
 manifest={}
 for name,p in paths.items():
  raw=p.read_bytes();v=json.loads(raw);manifest[name]=dict(bytes=len(raw),sha256=sha(raw))
  if name.startswith('farvi-'):
   if name!='farvi-all':
    suffix=name.removeprefix('farvi-');assert v['totals']['occurrences']==expect['sources'][suffix]and v['native_candidate_occurrences']==5
    assert v['groups']==[g for g in full['groups']if g['source']['abbreviation'].lower()==suffix]
  elif name in ['absent','excluded-sut']:assert v['totals']['occurrences']==0 and v['groups']==[]
  elif name.startswith('page'):
   assert v['totals']==full['totals']and v['native_candidate_occurrences']==5
   if name=='page-0':assert not v['groups']and v['next_cursor']is None
  elif name in ['jbotci-local','jbotci-catalogue-native333','la-catalogue-native333','llg-component','sut-native-entry']:assert v['status']=='Resolved',name
  elif name=='wrong-role':assert v['expected_error']and v['error']['code']=='invalid_reference'and 'sources'in v['error']
  elif name in ['missing','unknown','unresolved']:assert v['status']=={'missing':'NoMatch','unknown':'Unknown','unresolved':'Unresolved'}[name]
  else:raise AssertionError(name)
 for chain in chains.values():
  actual=[]
  for name in chain:actual+=hits(json.loads(paths[name].read_bytes()))
  assert actual==hits(full),chain
  assert json.loads(paths[chain[-1]].read_bytes())['next_cursor']is None
 return manifest

def analyze(observation_paths,out):
 expectation=json.loads((base/'full-freeze02/expectations-full03.json').read_text());requests=json.loads((base/'full-freeze02/requests-full03.json').read_text());names=[r['id']for r in requests if not r.get('prime')];chains=expectation['pages']['concat_by_size']
 truth=dict(bytes=7160366,sha256=expectation['full_wire']['legacy_unpaged_full_sha256'],sources={'lsk':3,'llg':1,'sut':1})
 rows=[];stage_rows=[];wires=[]
 for path in observation_paths:
  doc=json.loads(Path(path).read_text())
  for row in doc['observations']:
   assert row['decision']=='passed'and row['exit']==0 and row['maximum_rss_bytes']>0
   raw=Path(row['stdout_path']).read_bytes();assert sha(raw)==row['stdout_sha256'];text=Path(row['stderr_path']).read_text();row=dict(row)
   if row['workload']=='fresh':
    assert len(raw)==truth['bytes']and sha(raw)==truth['sha256'];p=re.search(r'stage=open topology=\w+ elapsed_ms=\d+ profile=DictionarySnapshotOpenProfile \{[^\n]+total: ([^ }]+) \}',text);assert p
    row['startup_seconds']=metrics.duration_ns(p[1])/1e9
    p=re.search(r'stage=exact request=farvi-all profile=DictionaryExactProfile \{ total: ([^,]+), candidate_sql: ([^,]+), hydration: ([^,]+), lookup_order_paging: ([^ }]+) \}',text);assert p
    total,sql,hyd,res=map(metrics.duration_ns,p.groups());assert total==sql+hyd+res
    val=int(re.search(r'stage=value request=farvi-all elapsed_us=(\d+)',text)[1]);ser=int(re.search(r'stage=serialize request=farvi-all elapsed_us=(\d+)',text)[1]);row['fresh_hydration_value_serialization_us']=hyd/1000+val+ser
   else:
    summary=json.loads(raw);assert [r['id']for r in summary['responses']]==names
    wires.append(dict(round=row['round'],topology=row['topology'],outputs=validate_batch(summary,truth,chains)))
    for s in metrics.profiles(text,names):stage_rows.append(dict(round=row['round'],topology=row['topology'],**s))
   rows.append(row)
 reference=wires[0]['outputs'];assert all(w['outputs']==reference for w in wires),'complete per-request wire mismatch'
 budgets=[]
 def budget(label,vals,median,max0):
  result=dict(measure=label,values=vals,median=statistics.median(vals),max=max(vals),median_ceiling=median,max_ceiling=max0);result['pass']=result['median']<=median and result['max']<=max0;return result
 for t in ['unified','attached','fanout']:
  fresh=[r for r in rows if r['topology']==t and r['workload']=='fresh'];assert len(fresh)==3
  budgets +=[dict(topology=t,**budget('fresh startup seconds',[r['startup_seconds']for r in fresh],5,8)),dict(topology=t,**budget('fresh fullfarvi E2E seconds',[r['outer_wall_seconds']for r in fresh],6,10)),dict(topology=t,**budget('fresh fullfarvi hydration+Value+render microseconds',[r['fresh_hydration_value_serialization_us']for r in fresh],500000,1000000))]
  for name in names:
   ss=[s for s in stage_rows if s['topology']==t and s['id']==name];assert len(ss)==3
   budgets.append(dict(topology=t,request=name,**budget('warm lookup microseconds',[s['warm_lookup_us']for s in ss],250000,750000)))
   upper=[s['hydration_value_serialization_us']if s['kind']=='exact'else s['warm_lookup_us']+s['value_us']+s['serialize_us']for s in ss]
   budgets.append(dict(topology=t,request=name,reference_whole_call_upper_bound=ss[0]['kind']=='reference',**budget('hydration+Value+render microseconds',upper,500000,1000000)))
  rss=[r['maximum_rss_bytes']for r in rows if r['topology']==t];assert max(rss)<=8589934592
 output=dict(status='PASS finite correctness and budget evaluation completed; selection remains CDC disposition',processes=len(rows),warm_stage_rows=len(stage_rows),per_request_wire_sets=len(wires),complete_wire_parity=True,wire_manifest=reference,processes_detail=rows,stages=stage_rows,budgets=budgets,budget_failures=[b for b in budgets if not b['pass']],cache_state='uncontrolled filesystem cache, no purge; process-fresh does not mean disk cold')
 out.write_text(json.dumps(output,indent=2)+'\n');return output
if __name__=='__main__':
 output=analyze(sys.argv[1:-1],Path(sys.argv[-1]));print(json.dumps({k:output[k]for k in ['status','processes','warm_stage_rows','per_request_wire_sets','complete_wire_parity','budget_failures']}))
