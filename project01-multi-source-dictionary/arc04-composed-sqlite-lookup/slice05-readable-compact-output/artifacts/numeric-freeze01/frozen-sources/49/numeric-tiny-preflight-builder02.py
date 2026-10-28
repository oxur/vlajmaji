"""Construct exact Tiny-only preflight after the retained accounting method failure."""
from pathlib import Path
import importlib.util,json,shutil
D=Path('/tmp/vlajmaji-p01-a04-s05-cdc');B=D/'numeric-tiny-fixture01';summary=json.loads((B/'setup-summary.json').read_bytes());ctx=json.loads((B/'readable-context.json').read_bytes())
sp=importlib.util.spec_from_file_location('numeric_runner',D/'numeric-runner01.py');r=importlib.util.module_from_spec(sp);sp.loader.exec_module(r)
cfg={'readiness_status':'TINY_METHOD_FROZEN_HANDOFF','root':summary['root'],'config':summary['config'],'output':str(D/'numeric-tiny-orchestration01'),'cli':str(D/'numeric-frozen-binaries01/vlajmaji'),'observer':str(D/'numeric-frozen-binaries01/numeric-observer'),'baseline':str(r.BASELINE),'workload':str(B/'requests.json'),'tiny_context':str(B/'readable-context.json'),'guard':str(r.GUARD),'python':'/Library/Frameworks/Python.framework/Versions/3.11/bin/python3','source':'/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc04/vlajmaji','source_commit':'1b9038ddc6c45f3ea709a0bdfec64041221e3571','source_tree':'338daa526d9f1cd56813d0c68a41a3f258e8ec47','expected_assertion_ids':ctx['lexical_assertion_ids'],'catalogue_revision':1,'profile_revision':1,'minimum_free_bytes':512*1024**2,'target_cache':'/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc01/target','sqlite_runtime_version':'3.53.2','sqlite_version_expectation':'pinned bundled libsqlite3-sys0.38.2 header; actual SAME observer setup rows MUST match','database_paths':[str(B/'root/catalogue.sqlite3'),str(Path(summary['store'])/'dictionary.sqlite3')]}
cfg['expected_database_sha256']={p:r.sha(p) for p in cfg['database_paths']}
scratch=[D,*[p for p in Path('/tmp').iterdir() if p!=D and (p.name.startswith('s05-') or p.name.startswith('vlajmaji-s05-') or p.name.startswith('vlajmaji-p01-a04-s05'))]];cfg['new_s05_scratch_paths']=[str(p) for p in sorted(scratch)];observed=r.inventory(scratch);assert observed['bytes']<=12*1024**3
paths={Path(row['path']).resolve() for row in json.loads((D/'numeric-source-closure04.json').read_bytes())['files']}
paths|={Path(cfg[k]).resolve() for k in ['cli','observer','baseline','workload','config','tiny_context','python','guard']}
paths|={D.resolve()/'numeric-tiny-method01.py',D.resolve()/'readable-cli-checks01.py',D.resolve()/'numeric-tiny-setup01.py',D.resolve()/'numeric-semantic-qualifier01.log',Path(__file__).resolve()}
for row in ctx['inputs']:
 for k in ['release','seal']:paths.add(Path(row[k]).resolve())
cfg['frozen_files']=[{'path':str(p),'bytes':p.stat().st_size,'sha256':r.sha(p)} for p in sorted(paths)]
cfg['expanded_stages']=r.expanded(cfg,True);assert len(r.expected_facts(cfg,True))==54
cfg['preflight_observed']={'free_bytes':shutil.disk_usage(D).free,'named_scratch_paths':len(scratch),'named_scratch_bytes':observed['bytes'],'target_cache_pair':'runner takes quiet du-sk before/after','limits':'Tiny qualification only; no retained Full root/model/DB access; no metrics accepted as Full'}
p=D/'numeric-tiny-preflight01.json'
with p.open('x') as f:json.dump(cfg,f,indent=2);f.write('\n')
print(json.dumps({'path':str(p),'bytes':p.stat().st_size,'sha256':r.sha(p),'frozen_files':len(cfg['frozen_files']),'expanded_stages':len(cfg['expanded_stages']),'expected_facts':54,'scratch_paths':len(scratch),'scratch_bytes':observed['bytes'],'free_bytes':shutil.disk_usage(D).free,'tiny_only':True}))
