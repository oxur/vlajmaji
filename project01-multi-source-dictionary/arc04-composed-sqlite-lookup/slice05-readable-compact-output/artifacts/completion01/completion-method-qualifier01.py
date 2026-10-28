"""Bounded synthetic SAME completion helpers; no product/DB/config/Full IO."""
import copy,importlib.util,json
from pathlib import Path
D=Path(__file__).parent
s=importlib.util.spec_from_file_location('completion_runner',D/'completion-runner01.py');r=importlib.util.module_from_spec(s);s.loader.exec_module(r)
a=r.a;spec=json.loads((D/'numeric-caller01/fixtures/requests-full03.json').read_bytes())
fresh=[{'kind':'fresh','mode':mode,'round':round_,'elapsed_ns':1000,'stdout_bytes':100} for mode in a.original.FRESH for round_ in range(1,4)]
warm=[]
snapshots={mode:{'synthetic_snapshot':mode if mode=='full' else 'compact'} for mode in ('full','compact','readable')}
kw={'tiny':False,'workload_sha256':'synthetic-sha','snapshots':snapshots}
for mode in ('full','compact','readable'):
 for round_ in range(1,4):
  setup={'row_type':'setup','mode':mode,'round':round_,'config_and_capture_ns':100,'tiny':False,'request_spec_sha256':'synthetic-sha','snapshot':snapshots[mode]}
  rows=[]
  for index,w in enumerate(spec):
   identity=w['id'].removeprefix('prime--');error=identity=='wrong-role'
   row={'row_type':'request','id':w['id'],'identity':identity,'prime':w.get('prime',False),'mode':mode,'round':round_,'operation':w['operation'],'ok':not error,'wire_bytes':100,'facts':{'synthetic':w['id']}}
   if error:row.update(expected_error='invalid_reference',profile=None,whole_error_ns=1000)
   else:row.update(total_ns=1100 if index==0 else 1000,query_total_ns=1000,lookup_ns=600,projection_ns=400,candidate_sql_ns=100,serialization_ns=100,payload_setup_ns=100 if index==0 else None)
   row['latency_checks']=a.checks_for(row,setup,index,mode);rows.append(row)
  warm.append({'kind':'warm','mode':mode,'round':round_,'rows':[setup,*rows,{'row_type':'complete','prime':27,'measured':27,'mode':mode,'round':round_,'tiny':False}]})
expected={w['id']:({'error':'invalid_reference'} if w['id'].removeprefix('prime--')=='wrong-role' else {'synthetic':w['id']}) for w in spec}
assert a.analyze(fresh,warm,spec,**kw)['historical_targets_met'];r.validate_facts(warm[0],expected);print('PASS positive exact3 peridentity/implementation; semantic facts validated')

def reject(name,call):
 try:call()
 except (AssertionError,KeyError):print('PASS reject '+name)
 else:raise AssertionError('accepted '+name)

def refresh(stages):
 for stage in stages:
  for index,row in enumerate(stage['rows'][1:-1]):row['latency_checks']=a.checks_for(row,stage['rows'][0],index,stage['mode'])

slow=copy.deepcopy(warm)
for stage in slow:
 stage['rows'][0]['config_and_capture_ns']=9_000_000_000
 for row in stage['rows'][1:-1]:
  if row['prime']:continue
  if not row['ok']:row['whole_error_ns']=900_000_000
  else:row.update(query_total_ns=1_300_000_000,total_ns=1_300_000_000,lookup_ns=1_000_000_000,projection_ns=300_000_000,serialization_ns=800_000_000,candidate_sql_ns=0)
refresh(slow);v=a.analyze(fresh,slow,spec,**kw)
assert v['completion_checks_passed'] and not v['historical_targets_met']
for prefix in ('startup:','warm:full:jbotci-local','warm:readable:wrong-role','projection+serialization:full:farvi-all','projection+serialization:compact:unknown'):assert any(k.startswith(prefix) for k in v['accepted_latency_exceptions'])
print('PASS report-only startup/wholeResolve/wrongrole/Full+smallprojection overruns retain originalactuals/targets')
changed=copy.deepcopy(warm);changed[0]['rows'][28]['identity']='different';reject('request identity',lambda:a.analyze(fresh,changed,spec,**kw))
changed=copy.deepcopy(warm);changed[-1]['round']=2;reject('duplicate round',lambda:a.analyze(fresh,changed,spec,**kw))
changed=copy.deepcopy(warm);changed[0]['rows'][28]['lookup_ns']+=1;reject('profile equation',lambda:a.analyze(fresh,changed,spec,**kw))
changed=copy.deepcopy(warm);changed[0]['rows'][1]['latency_checks'][0]['actual_ns']+=1;reject('fabricated timingexception',lambda:a.analyze(fresh,changed,spec,**kw))
changed=copy.deepcopy(warm);changed[0]['rows'][1]['facts']={};reject('semantic fact',lambda:r.validate_facts(changed[0],expected))
changed=copy.deepcopy(warm)
for stage in changed:
 if stage['mode']=='compact':
  row=next(row for row in stage['rows'][1:-1] if row['id']=='farvi-all');row['wire_bytes']=65537
reject('fivehit oversized wire',lambda:a.analyze(fresh,changed,spec,**kw))
guard={'guard_exit':0,'stdout_bytes':100,'stderr_bytes':0,'receipt':{'decision':'passed','exit_code':0,'timed_out':False,'rss_parse_ok':True,'maximum_resident_set_size_bytes':1024}}
r.validate_guard(guard,b'');print('PASS guard positive')
for field,value in [('decision','rejected child exit'),('timed_out',True),('maximum_resident_set_size_bytes',8*r.GIB+1),('rss_parse_ok',False)]:
 bad=copy.deepcopy(guard);bad['receipt'][field]=value;reject('guard '+field,lambda bad=bad:r.validate_guard(bad,b''))
bad=copy.deepcopy(guard);bad['stdout_bytes']=128*1024**2+1;reject('128MiB capture',lambda:r.validate_guard(bad,b''))
for where,field,value in [(0,'tiny',True),(0,'request_spec_sha256','wrong-sha'),(0,'snapshot',{}),(-1,'mode','wrong-mode')]:
 changed=copy.deepcopy(warm);changed[0]['rows'][where][field]=value;reject('setup/complete '+field,lambda changed=changed:a.analyze(fresh,changed,spec,**kw))
print('COMPLETE 18 pure completion-method controls; no product/database/config or metric observation')
