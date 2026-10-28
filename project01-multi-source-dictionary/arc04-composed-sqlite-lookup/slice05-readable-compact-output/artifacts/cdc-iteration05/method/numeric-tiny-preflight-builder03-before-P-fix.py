"""Current011 SAME-runner02 owned-Tiny22 qualifier; NO retained Full leaf/data access."""
from pathlib import Path
import importlib.util,json,shutil,subprocess,os,stat,math
D=Path('/tmp/vlajmaji-p01-a04-s05-cdc');S=Path('/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc04/vlajmaji')
sp=importlib.util.spec_from_file_location('runner',D/'numeric-runner02.py');r=importlib.util.module_from_spec(sp);sp.loader.exec_module(r)
g=lambda *a:subprocess.check_output(['git','--no-optional-locks','-C',str(S),*a]).decode().strip()
assert g('rev-parse','HEAD')=='011af09a44abc153b8b8253f70d84147e21a6bb0' and not g('status','--porcelain=v1')
cfg=json.loads((D/'numeric-tiny-preflight02.json').read_bytes())
cfg.update(source_commit=g('rev-parse','HEAD'),source_tree=g('rev-parse','HEAD^{tree}'),output=str(D/'numeric-tiny-orchestration03'),cli=str(D/'numeric-frozen-binaries02/vlajmaji'),observer=str(D/'numeric-frozen-binaries02/numeric-observer'))
assert not Path(cfg['output']).exists()
# Reuse EXACT accepted nofollow/birth-cutoff accounting source, no retained p01 traversal.
source=(D/'numeric-full-preflight-builder04.py').read_text();scope=source[source.index("explicit=("):source.index("paths={Path(row['path'])")];exec(compile(scope,'accepted-scratch-scope','exec'))
paths={Path(row['path']).resolve() for row in json.loads((D/'numeric-source-closure04.json').read_bytes())['files']}
paths.discard((D/'numeric-runner01.py').resolve());paths.discard((D/'numeric-tiny-method01.py').resolve())
paths|={D/'numeric-runner02.py',D/'numeric-tiny-method02.py',Path(__file__),S/'Cargo.lock',S/'Cargo.toml'}
paths|={Path(cfg[k]).resolve() for k in ['cli','observer','baseline','workload','config','tiny_context','python','guard']}
changed=g('diff','--name-only','32e99b4fb0bfdc24e66535952114b6497d6aabfa','HEAD').splitlines();assert len(changed)==21;paths|={(S/p).resolve() for p in changed}
ctx=json.loads(Path(cfg['tiny_context']).read_bytes())
for row in ctx['inputs']:
 for k in ['release','seal']:paths.add(Path(row[k]).resolve())
cfg['frozen_files']=[{'path':str(p),'bytes':p.stat().st_size,'sha256':r.sha(p)} for p in sorted(paths)]
cfg['expanded_stages']=r.expanded(cfg,True);assert len(cfg['expanded_stages'])==22 and len(r.expected_facts(cfg,True))==54
cfg['preflight_observed']={'free_bytes':shutil.disk_usage(D).free,'named_scratch_paths':len(scratch),'named_scratch_bytes':observed['bytes'],'scope':'ONE current011 Tiny qualifier, SAME runner02/observer source/expected facts and unchanged owned Tiny DB pair; not Full metrics; no retained Full model/config/DB leaf access','previous_method_populations':'prior1failed RSS+22passed at old1b source retained separately; these are22additional authorized current-source qualifier children','target_cache_pair':'runner quiet before/after; build changes precede campaign','limits':'active600s timeout; RSS/output/scratch postcompletion thresholds, no cleanup or automatic retry'}
p=D/'numeric-tiny-preflight03.json'
with p.open('x') as f:json.dump(cfg,f,indent=2);f.write('\n')
print(json.dumps({'path':str(p),'bytes':p.stat().st_size,'sha256':r.sha(p),'frozen_files':len(cfg['frozen_files']),'stages':len(cfg['expanded_stages']),'scratch_paths':len(scratch),'scratch_bytes':observed['bytes'],'free_bytes':shutil.disk_usage(D).free}))
