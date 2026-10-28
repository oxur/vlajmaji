from pathlib import Path
import json,hashlib,importlib.util
D=Path('/tmp/vlajmaji-p01-a04-s05-cdc');O=D/'numeric-tiny-orchestration04';I=D/'i05-independent01'
spec=importlib.util.spec_from_file_location('runner',D/'numeric-runner02.py');r=importlib.util.module_from_spec(spec);spec.loader.exec_module(r)
cfg=json.loads((D/'numeric-tiny-preflight04.json').read_bytes());summary=json.loads((O/'summary.json').read_bytes());assert summary['ok'] and summary['tiny_method'] and summary['completed_or_attempted_stages']==22 and summary['failure'] is None
rows=sorted([json.loads(p.read_bytes()) for p in O.glob('*/checked-stage.json')],key=lambda x:x['number']);assert len(rows)==22;want=r.expected_facts(cfg,True);total=0
for row,expected in zip(rows,cfg['expanded_stages']):
 for k,v in expected.items():assert row[k]==v,(row['number'],k)
 assert row['guard_exit']==0 and row['receipt']['exit_code']==0 and row['receipt']['decision']=='passed' and not row['receipt']['timed_out'] and row['receipt']['rss_parse_ok']
 p=O/(f'warm-{row["round"]}-{row["mode"]}' if row['kind']=='warm' else 'semantic' if row['kind']=='semantic' else f'{row["number"]:02d}-{row["kind"]}-{row["round"] or 0}-{row["mode"] or "semantic"}')
 for f in ['stdout','stderr']:assert r.sha(p/f)==row[f+'_sha256'] and (p/f).stat().st_size==row[f+'_bytes']
 if row['kind']=='warm':
  raw=r.framed_jsonl(p/'stdout');assert len(raw)==56 and raw==row['rows'];assert [a['id'] for a in raw[1:-1]]==list(want)
  for a in raw[1:-1]:
   w=want[a['id']]
   if 'error' in w:assert not a['ok'] and a['expected_error']==w['error'] and a['profile'] is None
   else:assert a['ok'] and a['facts']==w
   total+=1
assert total==486
analysis=r.analysis.analyze(rows[:21],json.loads(Path(cfg['workload']).read_bytes()));assert analysis==json.loads((O/'numerical-analysis.json').read_bytes()) and analysis['ok']
assert r.fresh_phase_budget(rows[:12])==json.loads((O/'fresh-phase-budget.json').read_bytes())
assert summary['database_before']==summary['database_after']==cfg['expected_database_sha256'] and summary['database_unchanged']
assert summary['target_cache_before_bytes']==summary['target_cache_after_bytes'] and summary['target_cache_delta_bytes']==0
for f in cfg['frozen_files']:assert Path(f['path']).stat().st_size==f['bytes'] and r.sha(f['path'])==f['sha256']
result={'source_head':cfg['source_commit'],'source_tree':cfg['source_tree'],'checked_stages':22,'warm_actual_requests':total,'prime':243,'measured':243,'preinput_facts':54,'same_saved_analysis_exact':True,'phase12_exact':True,'all63_pins_stable':True,'database_pair_unchanged':True,'target_cache_pair_bytes':summary['target_cache_before_bytes'],'observed_scratch_bytes':summary['observed_inventory_bytes'],'scope':'current Tiny method only; no Full metric acceptance','retained_population':'prior source1b:1instrumentation-failed realbaseline+22PASS; current011 zero-stage outer pin-closure attempt then22PASS. Total45 realTiny method children, not one clean22 across history; no Full child in this record.'}
with (I/'tiny-reconciliation01.json').open('x') as f:json.dump(result,f,indent=2);f.write('\n')
print(json.dumps(result))
