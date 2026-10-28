"""Existing saved Tiny result reconciliation only; no Full data/model/query."""
from pathlib import Path
import importlib.util,json,hashlib
D=Path(__file__).parent;O=D/'completion-tiny-01'
s=importlib.util.spec_from_file_location('completion',D/'completion-runner01.py');c=importlib.util.module_from_spec(s);s.loader.exec_module(c);r=c.r
cfg=json.loads((D/'completion-tiny-preflight02.json').read_bytes());summary=json.loads((O/'summary.json').read_bytes())
assert summary['completion_checks_passed'] and summary['tiny_method'] and summary['attempted_children']==10 and summary['failure'] is None
stages=[json.loads(p.read_bytes()) for p in O.glob('*/checked-stage.json')];stages.sort(key=lambda row:row['number']);assert len(stages)==10
fresh,snapshots=c.reused_fresh(cfg,True);expected=r.expected_facts(cfg,True);spec=json.loads(Path(cfg['workload']).read_bytes());count=0
for number,(stage,want) in enumerate(zip(stages,cfg['expanded_stages']),1):
 assert stage['number']==number and stage['original_equivalent_stage']==number+12
 for k,v in want.items():assert stage[k]==v
 folder=O/(f'warm-{stage["round"]}-{stage["mode"]}' if stage['kind']=='warm' else 'semantic')
 c.validate_guard(stage,(folder/'guard.stderr').read_bytes())
 for name in ('stdout','stderr'):assert r.sha(folder/name)==stage[name+'_sha256'] and (folder/name).stat().st_size==stage[name+'_bytes']
 assert r.framed_jsonl(folder/'stdout')==stage['rows']
 if stage['kind']=='warm':
  c.a.validate_warm(stage,spec,tiny=True,workload_sha256=r.sha(cfg['workload']),snapshot=snapshots[stage['mode']]);c.validate_facts(stage,expected);count+=54
 else:assert stage['rows'][0]['row_type']=='semantic_complete' and stage['rows'][0]['ok'] is True and stage['rows'][0]['tiny'] is True and stage['rows'][0]['accepted_full_inner_sha_checked'] is False
assert count==486
verdict=c.a.analyze(fresh,stages[:9],spec,tiny=True,workload_sha256=r.sha(cfg['workload']),snapshots=snapshots)
assert verdict==json.loads((O/'completion-analysis.json').read_bytes()) and verdict['completion_checks_passed']
assert summary['database_before']==summary['database_after']==cfg['expected_database_sha256'] and summary['database_unchanged']
assert summary['target_cache_before_bytes']==summary['target_cache_after_bytes'] and summary['target_cache_delta_bytes']==0
for pin in cfg['frozen_files']:r.pin(pin)
answer={'source_commit':cfg['source_commit'],'source_tree':cfg['source_tree'],'actual_children':10,'new_fresh_children':0,'historical_fresh_reused':12,'prime':243,'measured':243,'request_facts':486,'frozen_preinput_facts':54,'semantic_completed':True,'same_saved_analysis_exact':True,'latency_exceptions':verdict['accepted_latency_exceptions'],'database_saved_pair_unchanged':True,'target_cache_pair_bytes':summary['target_cache_before_bytes'],'observed_scratch_bytes':summary['observed_scratch_bytes'],'all_pins_stable':len(cfg['frozen_files']),'history':'prior45actualTiny method children preserved; current outer01 output-prefix miss launchedZERO; correctedouter02 isSOLE10child qualifier (55totalactualTiny method children across history); no Full child in this reconciliation','scope':'Tiny method qualification only; no numerical Full acceptance'}
p=D/'completion-tiny-reconciliation01.json'
with p.open('x') as f:json.dump(answer,f,indent=2);f.write('\n')
print(json.dumps(answer))
