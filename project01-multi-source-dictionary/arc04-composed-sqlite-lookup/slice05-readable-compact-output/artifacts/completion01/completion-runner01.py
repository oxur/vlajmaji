"""Completion ONLY: nine balanced warm observers + unchanged semantic observer.
SOURCE/PREFLIGHT only until exact root clearance. No fresh children or retries.
"""
import importlib.util,json,os,shutil,subprocess,time,traceback
from pathlib import Path
HERE=Path(__file__).parent

def module(name,path):
 s=importlib.util.spec_from_file_location(name,path);m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
r=module('original_numeric_runner',HERE/'numeric-runner02.py')
a=module('completion_analysis',HERE/'completion-analysis01.py')
OUTPUT=HERE/'completion-full01'
GIB=1024**3
SOURCE='011af09a44abc153b8b8253f70d84147e21a6bb0'
TREE='a8c0fdcb3e39fb260602bb48b837b25f4d2380ac'

def expanded(cfg,tiny):
 rows=r.expanded(cfg,tiny)[12:];assert len(rows)==10
 assert [(s['round'],s['mode']) for s in rows[:9]]==[(round_,mode) for round_,modes in enumerate(r.WARM,1) for mode in modes]
 return rows

def reused_fresh(cfg,tiny):
 bundle=cfg['reused_fresh'];assert bundle['source_commit']==SOURCE and bundle['source_tree']==TREE
 assert bundle['original_config']['source_commit']==SOURCE and bundle['original_config']['source_tree']==TREE
 records=[];full_hash=None;snapshots={}
 for number,pin in enumerate(bundle['checked_stage_pins'],1):
  r.pin(pin);path=r.leaf(pin['path'],HERE);record=json.loads(path.read_bytes())
  assert record['number']==number and record['kind']=='fresh'
  expected=r.expanded(bundle['original_config'],tiny)[number-1]
  for k,v in expected.items():assert record[k]==v,(number,k)
  assert record['guard_exit']==0 and record['receipt']['exit_code']==0 and record['receipt']['decision']=='passed'
  assert not record['receipt']['timed_out'] and record['receipt']['rss_parse_ok'] and 0<record['receipt']['maximum_resident_set_size_bytes']<=8*GIB
  for name in ('stdout','stderr'):
   f=r.leaf(path.parent/name,HERE);assert f.stat().st_size==record[name+'_bytes'] and r.sha(f)==record[name+'_sha256']
  digest=r.fresh_wire((path.parent/'stdout').read_bytes(),record['mode'],cfg['expected_assertion_ids'],full_hash)
  if digest is not None:full_hash=digest
  if record['mode'] in ('candidate-full','compact'):
   mode='full' if record['mode']=='candidate-full' else 'compact';observed=r.check.strict_json((path.parent/'stdout').read_bytes())['snapshot']
   if mode in snapshots:assert snapshots[mode]==observed
   else:snapshots[mode]=observed
  records.append(record)
 assert len(records)==12 and r.fresh_phase_budget(records)['ok']
 snapshots['readable']=snapshots['compact']
 return records,snapshots

def validate_facts(stage,expected):
 for row in stage['rows'][1:-1]:
  want=expected[row['id']]
  if 'error' in want:assert row['ok'] is False and row['expected_error']==want['error'] and row['profile'] is None
  else:assert row['ok'] is True and row['facts']==want,(stage['mode'],stage['round'],row['id'])

def validate_guard(record,guard_stderr):
 receipt=record['receipt']
 assert record['guard_exit']==0 and receipt['decision']=='passed' and receipt['exit_code']==0
 assert receipt['timed_out'] is False and receipt['rss_parse_ok'] is True and 0<receipt['maximum_resident_set_size_bytes']<=8*GIB
 assert guard_stderr==b'' and record['stdout_bytes']+record['stderr_bytes']<=128*1024**2

def execute(cfg,*,tiny):
 assert cfg['readiness_status']==('TINY_COMPLETION_METHOD_FROZEN' if tiny else 'ROOT_REVIEWED_EXACT_COMPLETION_ONLY')
 root,config,out=map(Path,(cfg['root'],cfg['config'],cfg['output']))
 if tiny:
  assert root==config.parent/'root' and config.is_relative_to(HERE) and Path(cfg['workload']).is_relative_to(HERE)
  assert out.is_relative_to(HERE) and out.name.startswith('completion-tiny-')
 else:
  assert (root,config,out)==(r.ROOT,r.CONFIG,OUTPUT)
  assert Path(cfg['workload'])==r.WORK and r.sha(r.WORK)==r.WORK_SHA and r.sha(r.WITNESS)==r.WITNESS_SHA
  assert cfg['catalogue_revision']==4 and cfg['profile_revision']==5
 assert root.is_absolute() and config.is_absolute() and out.is_absolute() and '..' not in out.parts
 assert not out.exists() and not out.is_symlink() and Path(cfg['guard'])==r.GUARD
 source=Path(cfg['source']);env_git={**os.environ,'GIT_OPTIONAL_LOCKS':'0'}
 for suffix,value in [('HEAD',SOURCE),('HEAD^{tree}',TREE)]:assert subprocess.check_output(['git','--no-optional-locks','-C',str(source),'rev-parse',suffix],env=env_git).decode().strip()==value
 assert cfg['source_commit']==SOURCE and cfg['source_tree']==TREE
 assert not subprocess.check_output(['git','--no-optional-locks','-C',str(source),'status','--porcelain=v1'],env=env_git)
 for f in cfg['frozen_files']:r.pin(f)
 assert len({f['path'] for f in cfg['frozen_files']})==len(cfg['frozen_files'])
 closure={str(r.canonical_system_tmp(f['path'])) for f in cfg['frozen_files']}
 for p in [cfg['observer'],cfg['workload'],str(config),str(r.GUARD),__file__,str(HERE/'completion-analysis01.py'),str(HERE/'numeric-runner02.py'),str(HERE/'numeric-analysis01.py'),str(HERE/'cli-checks01.py')]:assert str(r.canonical_system_tmp(p)) in closure,p
 fresh,snapshots=reused_fresh(cfg,tiny);expected=r.expected_facts(cfg,tiny);workload_sha256=r.sha(cfg['workload'])
 databases=[r.leaf(p,root) for p in cfg['database_paths']]
 assert len(databases)==2 and len(set(databases))==2 and root/'catalogue.sqlite3' in databases
 assert next(p for p in databases if p.name!='catalogue.sqlite3').name=='dictionary.sqlite3'
 assert set(map(str,databases))==set(cfg['expected_database_sha256'])
 assert shutil.disk_usage(out.parent).free>=cfg['minimum_free_bytes']
 scratch=list(map(Path,cfg['new_s05_scratch_paths']));assert HERE in scratch
 target=Path(cfg['target_cache']);assert target==Path('/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc01/target')
 for protected in (target,r.ROOT):assert not any(r.canonical_system_tmp(protected).is_relative_to(r.canonical_system_tmp(p)) or r.canonical_system_tmp(p).is_relative_to(r.canonical_system_tmp(protected)) for p in scratch)
 before_scratch=r.inventory(scratch);assert before_scratch['bytes']<=12*GIB
 cache_before=r.target_cache_bytes(target)
 stages=expanded(cfg,tiny);assert stages==cfg['expanded_stages'];out.mkdir(mode=0o700)
 isolated={k:str(out/n) for k,n in [('HOME','isolated-home'),('XDG_DATA_HOME','isolated-data'),('XDG_CONFIG_HOME','isolated-config')]}
 for p in isolated.values():Path(p).mkdir(mode=0o700)
 childenv={'PATH':'/usr/bin:/bin:/usr/sbin:/sbin','LANG':'en_US.UTF-8','LC_ALL':'en_US.UTF-8',**isolated}
 r.new_json(out/'environment.json',childenv);r.new_json(out/'expanded-stages.json',stages);r.new_json(out/'reused-fresh.json',fresh)
 r.new_json(out/'s05-scratch-before.json',before_scratch);r.new_json(out/'target-cache-before.json',{'path':str(target),'bytes':cache_before})
 r.new_json(out/'pre-input-normalized-facts.json',expected)
 before={str(p):r.sha(p) for p in databases};r.new_json(out/'database-before.json',before)
 assert before==cfg['expected_database_sha256']
 actual=[];failed=None;started=time.monotonic()
 try:
  for number,stage in enumerate(stages,1):
   if stage['kind']=='semantic':
    assert number==10 and len(actual)==9
    verdict=a.analyze(fresh,actual,json.loads(Path(cfg['workload']).read_bytes()),tiny=tiny,workload_sha256=workload_sha256,snapshots=snapshots);r.new_json(out/'completion-analysis.json',verdict)
    assert verdict['completion_checks_passed'] # latency targets remain reported, not enforced
   workdir=out/(f'warm-{stage["round"]}-{stage["mode"]}' if stage['kind']=='warm' else 'semantic');workdir.mkdir(mode=0o700)
   stdout,stderr,guardout,guarderr=[workdir/n for n in ('stdout','stderr','guard.stdout','guard.stderr')]
   argv=[cfg['python'],str(r.GUARD),'--timeout','600','--max-rss-bytes',str(8*GIB),'--log',str(stderr),'--stdout',str(stdout),'--',*stage['argv']]
   with guardout.open('xb') as o,guarderr.open('xb') as e:result=subprocess.run(argv,stdout=o,stderr=e,env=childenv)
   receipt=r.check.strict_json(guardout.read_bytes());r.new_json(workdir/'receipt.json',receipt)
   record={**stage,'number':number,'original_equivalent_stage':number+12,'guard_argv':argv,'guard_exit':result.returncode,'receipt':receipt,'stdout_bytes':stdout.stat().st_size,'stdout_sha256':r.sha(stdout),'stderr_bytes':stderr.stat().st_size,'stderr_sha256':r.sha(stderr)}
   actual.append(record);r.new_json(workdir/'stage.json',record)
   validate_guard(record,guarderr.read_bytes())
   record['rows']=r.framed_jsonl(stdout)
   if stage['kind']=='warm':
    a.validate_warm(record,json.loads(Path(cfg['workload']).read_bytes()),tiny=tiny,workload_sha256=workload_sha256,snapshot=snapshots[stage['mode']])
    assert record['rows'][0]['sqlite_runtime_version']==cfg['sqlite_runtime_version']
    validate_facts(record,expected)
   else:
    rows=record['rows'];assert len(rows)==1 and rows[0]['row_type']=='semantic_complete' and rows[0]['ok'] is True
    assert rows[0]['tiny'] is tiny and rows[0]['accepted_full_inner_sha_checked'] is (not tiny)
   r.new_json(workdir/'checked-stage.json',record)
   observed=r.inventory(scratch);assert observed['bytes']<=12*GIB
   print(json.dumps({'completion_stage':number,'equivalent_original_stage':number+12,'kind':stage['kind'],'mode':stage['mode'],'round':stage['round'],'completion_decision':'passed','elapsed_seconds':receipt['elapsed_seconds'],'rss_bytes':receipt['maximum_resident_set_size_bytes']}),flush=True)
 except BaseException as error:
  failed={'type':type(error).__name__,'detail':str(error),'traceback':traceback.format_exc()};raise
 finally:
  after={str(p):r.sha(p) for p in databases};r.new_json(out/'database-after.json',after)
  observed=r.inventory(scratch);r.new_json(out/'inventory-before-summary.json',observed)
  cache_after=r.target_cache_bytes(target);r.new_json(out/'target-cache-after.json',{'path':str(target),'bytes':cache_after,'delta_bytes':cache_after-cache_before})
  summary={'completion_checks_passed':failed is None and len(actual)==10 and before==after and cache_after==cache_before and observed['bytes']<=12*GIB,'latency_targets':'report-only explicit exception; see completion-analysis.json; never a historical numerical PASS','tiny_method':tiny,'attempted_children':len(actual),'reused_fresh_children':12,'elapsed_seconds':time.monotonic()-started,'failure':failed,'database_before':before,'database_after':after,'database_unchanged':before==after,'target_cache_before_bytes':cache_before,'target_cache_after_bytes':cache_after,'target_cache_delta_bytes':cache_after-cache_before,'observed_scratch_bytes':observed['bytes'],'resource_mechanism':'600s process-group timeout active;8GiB RSS/128MiB captured output/12GiB observed scratch checked after each process; not live enforcement','settled_delta':'inventory-before-summary.json,target-cache-after.json,summary.json written after inventory observation; include external final inventory own-bytes separately','scope':'one nine-warm+semantic completion invocation;12passed historical fresh reused; failed campaigns immutable; fresh process is not cold OS cache'}
  r.new_json(out/'summary.json',summary)
  assert before==after and cache_before==cache_after and observed['bytes']<=12*GIB

if __name__=='__main__':
 import sys
 assert len(sys.argv)==2
 execute(json.loads(Path(sys.argv[1]).read_bytes()),tiny=False)
