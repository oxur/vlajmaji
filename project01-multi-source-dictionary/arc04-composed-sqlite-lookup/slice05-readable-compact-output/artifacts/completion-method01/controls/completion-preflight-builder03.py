"""Review packet builder. Full mode NEVER stats/reads/hashes retained config/DB.
Separate Tiny mode prepares the single owned Tiny10 qualification.
"""
from pathlib import Path
import importlib.util,json,shutil,subprocess,os,stat,math,sys
D=Path('/tmp/vlajmaji-p01-a04-s05-cdc');P=Path('/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/planning');S=Path('/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc04/vlajmaji')
s=importlib.util.spec_from_file_location('completion_runner',D/'completion-runner01.py');c=importlib.util.module_from_spec(s);s.loader.exec_module(c);r=c.r
assert len(sys.argv)==2 and sys.argv[1] in ('tiny','full');tiny=sys.argv[1]=='tiny'
old=json.loads((D/('numeric-tiny-preflight04.json' if tiny else 'numeric-full-preflight04.json')).read_bytes())
cfg=dict(old);cfg.update(readiness_status='TINY_COMPLETION_METHOD_FROZEN' if tiny else 'FULL_COMPLETION_PACKET_REVIEW_ONLY',output=str(D/('completion-tiny-01' if tiny else 'completion-full01')),observer=str(D/'completion-frozen-binaries01/completion-observer'))
assert not Path(cfg['output']).exists()
assert subprocess.check_output(['git','--no-optional-locks','-C',str(S),'rev-parse','HEAD']).decode().strip()==c.SOURCE
assert not subprocess.check_output(['git','--no-optional-locks','-C',str(S),'status','--porcelain'])
previous=D/('numeric-tiny-orchestration04' if tiny else 'numeric-full02')
pins=[]
for number,stage in enumerate(r.expanded(old,tiny)[:12],1):
 work=previous/f'{number:02d}-fresh-{stage["round"]}-{stage["mode"]}'
 p=r.leaf(work/'checked-stage.json',D);pins.append({'path':str(p),'bytes':p.stat().st_size,'sha256':r.sha(p)})
cfg['reused_fresh']={'source_commit':c.SOURCE,'source_tree':c.TREE,'original_config':old,'checked_stage_pins':pins,'scope':'exact12 originally checked receipts, no fresh child replay'}
# Source-only original accepted scratch selector. It skips protected retained p01 roots
# before lstat and accounts link payloads without following referents.
source=(D/'numeric-full-preflight-builder05.py').read_text();start=source.index('explicit=(');end=source.index('paths={strict_pin_path(')
exec(compile(source[start:end],'accepted-scratch-selector','exec'))
# selector updates cfg scratch lists/provenance and observed bytes, no Full leaf IO.
def strict_pin_path(path):
 p=r.canonical_system_tmp(Path(path));alias=Path('/Library/Frameworks/Python.framework/Versions/3.11/bin/python3')
 if p==alias:
  r.nofollow(p.parent);assert stat.S_ISLNK(p.lstat().st_mode) and os.readlink(p)=='python3.11';p=p.parent/'python3.11'
 r.leaf(p);return p
config_canonical=r.canonical_system_tmp(cfg['config'])
paths={strict_pin_path(f['path']) for f in old['frozen_files'] if tiny or Path(f['path'])!=config_canonical}
paths|={strict_pin_path(f['path']) for f in json.loads((D/'completion-source-closure02.json').read_bytes())['files']}
paths|={strict_pin_path(p) for p in [__file__,D/'completion-source-closure02.json',D/'completion-build01.py',D/'completion-build01/receipt.json',cfg['observer']]}
for pin in pins:
 p=Path(pin['path']);paths|={strict_pin_path(p),strict_pin_path(p.parent/'stdout'),strict_pin_path(p.parent/'stderr')}
if not tiny:
 qualification=json.loads((D/'completion-tiny-reconciliation01.json').read_bytes());assert qualification['actual_children']==10 and qualification['request_facts']==486 and qualification['source_commit']==c.SOURCE and qualification['all_pins_stable']==122
 cfg['completion_qualification']=qualification
 for name in ['completion-tiny-reconciliation01.json','completion-reconcile-tiny01.py','completion-tiny-execution02-receipt.json','completion-tiny-execution02.log','completion-tiny-preflight02.json','completion-tiny-01/summary.json','completion-tiny-01/completion-analysis.json']:paths.add(strict_pin_path(D/name))
cfg['frozen_files']=[{'path':str(p),'bytes':p.stat().st_size,'sha256':r.sha(p)} for p in sorted(paths)]
if not tiny:
 expected=[f for f in old['frozen_files'] if Path(f['path'])==config_canonical];assert len(expected)==1 and expected[0]['bytes']==219
 cfg['frozen_files']+=expected;cfg['deferred_full_metadata_pin']={'pin':expected[0],'observation':'expected219B/hash from accepted recorded writer; NOT currently read/stat/hash; deferred to root-cleared completion invocation'}
cfg['expanded_stages']=c.expanded(cfg,tiny);assert len(cfg['expanded_stages'])==10
fresh,snapshots=c.reused_fresh(cfg,tiny);assert len(fresh)==12 and len(snapshots)==3
cfg['preflight_observed']={'free_bytes':shutil.disk_usage(D).free,'minimum_free_bytes':cfg['minimum_free_bytes'],'named_scratch_paths':len(scratch),'observed_named_scratch_bytes':observed['bytes'],'target_cache_bytes':r.target_cache_bytes(Path(cfg['target_cache'])),'scope':'completion-only10 children max; historical12checkedfresh reused; no fresh replay; no Full model/store/import/All/reverification','limits':'600s active child process-group timeout;8GiB RSS/128MiB capture/12GiB ALL observed named scratch postcompletion thresholds; quiet own target pair; ONE DB pair INSIDE invocation','latency':'explicit report-only exception, original per-ID3round values/targets retained; correctness/byte/resource checks still fatal','full_data_boundary':'Full DB/config excluded from current actual data/leaf/hash access; expected pair and deferred config from accepted recorded receipts only' if not tiny else 'owned existing Tiny pair only inside qualification'}
cfg['method_evidence']={'derivation':str(D/'completion-caller-derivation01.json'),'pure_current18':str(D/'completion-method-qualifier02.log'),'semantic_helper':'byte-identical to prior qualified semantic.rs; source/input anchors unchanged','legacy_methods':'prior45actualTiny method children preserved; this ONE completion Tiny10 is separate, not a new numerical performance campaign','no_automatic_retry':True}
p=D/('completion-tiny-preflight02.json' if tiny else 'completion-full-preflight01.json')
with p.open('x') as f:json.dump(cfg,f,indent=2);f.write('\n')
print(json.dumps({'path':str(p),'bytes':p.stat().st_size,'sha256':r.sha(p),'pins':len(cfg['frozen_files']),'expanded_children':10,'reused_fresh':12,'review_only':not tiny,'observed_named_scratch_bytes':observed['bytes']}))
