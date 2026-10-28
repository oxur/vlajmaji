"""Prospective S05 v1.2 finite numerical runner. SOURCE ONLY, NOT launch clearance.
Full constants are fixed here; only separately named Tiny method entrypoint uses a
Tiny config/workload. 12 fresh CLI +9 warm observer +1 semantic observer maximum.
No retries, mutations, cache reset, import/create/All, cleanup or artifact overwrite.
"""
import hashlib, importlib.util, json, os, re, shutil, stat, subprocess, sys, time, traceback
from pathlib import Path
HERE=Path(__file__).parent
ROOT=Path('/tmp/vlajmaji-p01-a04-s04-cdc/full-operational01')
CONFIG=Path('/tmp/vlajmaji-p01-a04-s04-cdc/full-config01.toml')
OUTPUT=HERE/'numeric-full01'
WORK=HERE/'numeric-caller01/fixtures/requests-full03.json'
WORK_SHA='6fe407b2411263370524a44bd101c271fa17a3c3f4e92ee65425025f76d24c21'
WITNESS=Path('/tmp/vlajmaji-p01-a04-s04-cdc/full-caller01/accepted-descriptors-and-refs02.json')
WITNESS_SHA='7f332ad5f3e58f0d4a99cf3c12e82400e85923840430db76291c54951ccef867'
BASELINE=Path('/tmp/vlajmaji-p01-a04-s04-cdc/full-frozen-binaries01/vlajmaji')
BASELINE_SHA='20b21d4e27310437d643b246ecbaf652b5a4c6e53c407abf6035dca261d1b79c'
GUARD=Path('/tmp/vlajmaji-p01-a04-s04-cdc/stage_guard_split01.py')
GIB=1024**3
FRESH=[['baseline-full','candidate-full','compact','readable'],['readable','compact','candidate-full','baseline-full'],['compact','baseline-full','readable','candidate-full']]
WARM=[['full','compact','readable'],['readable','full','compact'],['compact','readable','full']]

def module(name,path):
 s=importlib.util.spec_from_file_location(name,path);m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
check=module('numeric_wire',HERE/'cli-checks01.py')
analysis=module('numeric_analysis',HERE/'numeric-analysis01.py')

def new_json(path,value):
 with Path(path).open('x') as f:json.dump(value,f,indent=2,ensure_ascii=False);f.write('\n')

def fresh_phase_budget(stage_rows):
 assert len(stage_rows)==12 and all(row['kind']=='fresh' for row in stage_rows)
 out={'fresh':{},'failed':[]}
 for mode,limits in analysis.FRESH.items():
  rows=[row for row in stage_rows if row['mode']==mode]
  assert sorted(row['round'] for row in rows)==[1,2,3]
  rows.sort(key=lambda row:row['round'])
  result=analysis.stats([row['elapsed_ns'] for row in rows],limits)
  out['fresh'][mode]=result
  if not result['ok']:out['failed'].append('fresh:'+mode)
 out['ok']=not out['failed']
 out['scope']='same retained12 fresh observations; exact3 per mode; no new process or trial'
 return out

def sha(path):
 h=hashlib.sha256()
 with Path(path).open('rb') as f:
  for part in iter(lambda:f.read(8*1024*1024),b''):h.update(part)
 return h.hexdigest()

def canonical_system_tmp(path):
 p=Path(path);assert p.is_absolute() and '..' not in p.parts
 if p.parts[1:2]==('tmp',):
  assert stat.S_ISLNK(Path('/tmp').lstat().st_mode) and os.readlink('/tmp')=='private/tmp'
  return Path('/private/tmp').joinpath(*p.parts[2:])
 return p

def nofollow(path,base=None):
 original=Path(path);p=canonical_system_tmp(original)
 if base is not None:p.relative_to(canonical_system_tmp(base)) # lexical containment BEFORE filesystem traversal
 walk=Path(p.anchor)
 for atom in p.parts[1:]:
  walk/=atom;assert not stat.S_ISLNK(walk.lstat().st_mode),walk
 return original

def leaf(path,base=None):
 p=nofollow(path,base);assert stat.S_ISREG(p.lstat().st_mode),p
 return p

def inventory(paths):
 rows=[];canonical=[canonical_system_tmp(p) for p in paths]
 assert len(set(canonical))==len(canonical)
 assert not any(a!=b and a.is_relative_to(b) for a in canonical for b in canonical)
 for root in map(Path,paths):
  nofollow(root) # Named roots/ancestors remain strict; retained negative links are descendants only.
  pending=[root]
  while pending:
   p=pending.pop();m=p.lstat()
   if stat.S_ISLNK(m.st_mode):
    target=os.readlink(os.fsencode(p)) # Native link payload ONLY; never stat/read/recurse into referent.
    rows.append({'path':str(p),'kind':'symlink','bytes':len(target),'native_target_hex':target.hex()})
   elif stat.S_ISDIR(m.st_mode):
    rows.append({'path':str(p),'kind':'directory','bytes':0})
    with os.scandir(p) as entries:children=sorted((Path(e.path) for e in entries),key=lambda q:os.fsencode(q.name))
    pending.extend(reversed(children)) # Recursion only after lstat-confirmed actual directory.
   else:
    assert stat.S_ISREG(m.st_mode),p
    rows.append({'path':str(p),'kind':'file','bytes':m.st_size})
 return {'entries':rows,'bytes':sum(r['bytes'] for r in rows),'scope':'exact named S05 scratch including retained negative-fixture link payload bytes via lstat/readlink without referent traversal; logical retained bytes, not historical cumulative/peak'}

def target_cache_bytes(target):
 p=nofollow(target);assert stat.S_ISDIR(p.lstat().st_mode)
 raw=subprocess.check_output(['/usr/bin/du','-sk',str(p)]);count=raw.split(None,1)[0]
 assert count.isdigit();return int(count)*1024


def pin(entry):
 p=leaf(entry['path']);assert p.stat().st_size==entry['bytes'] and sha(p)==entry['sha256'],p

def expanded(cfg,tiny):
 cli,observer,config=str(cfg['cli']),str(cfg['observer']),str(cfg['config']);rows=[]
 for round_,modes in enumerate(FRESH,1):
  for mode in modes:
   binary=str(cfg['baseline']) if mode=='baseline-full' else cli
   args=[binary,'lookup','farvi','--config',config]
   if mode!='baseline-full':args+=['--format','full' if mode=='candidate-full' else mode]
   rows.append({'kind':'fresh','round':round_,'mode':mode,'argv':args})
 for round_,modes in enumerate(WARM,1):
  for mode in modes:
   name=f'warm-{round_}-{mode}';workdir=Path(cfg['output'])/name
   args=[observer,'warm-tiny' if tiny else 'warm-full',mode,str(round_),str(workdir)]
   if tiny:args += [config,str(cfg['workload'])]
   rows.append({'kind':'warm','round':round_,'mode':mode,'argv':args})
 args=[observer,'semantic-tiny' if tiny else 'semantic-full',str(Path(cfg['output'])/'semantic')]
 if tiny:args += [config,str(cfg['workload'])]
 rows.append({'kind':'semantic','round':None,'mode':None,'argv':args})
 assert len(rows)==22;return rows

def fresh_wire(raw,mode,expected,full_hash):
 assert raw.endswith(b'\n') and not raw.endswith(b'\n\n')
 if mode=='readable':
  text=raw.decode();assert text.startswith('Exact "farvi": 5 occurrences, 1 spellings, 3 sources\n')
  assert b'\x1b' not in raw and len(raw)<=131072
  observed=[]
  for line in text.splitlines():
   if line.startswith('    detail: '):observed.append(json.loads(line[12:])['assertion_id'])
  assert set(observed)==set(expected) and len(observed)==5
  assert text.splitlines()[-1]=='next_cursor: null'
  return None
 value=check.strict_json(raw)
 assert isinstance(value,dict) and set(value)=={'version','snapshot','collections','result'} and value['version']==1
 assert set(value['result'])=={'operation','response'} and value['result']['operation']=='exact'
 inner=value['result']['response'];assert inner['totals']=={'occurrences':5,'distinct_spellings':1,'source_groups':3}
 hits=[h for group in inner['groups'] for h in group['hits']]
 assert len(hits)==5 and {h['assertion']['id'] for h in hits}==set(expected)
 assert inner['next_cursor'] is None
 if mode=='compact':check.compact_exact_wire(inner);assert len(raw)<=65536
 else:
  digest=hashlib.sha256(raw).hexdigest()
  if full_hash is not None:assert digest==full_hash,'actual baseline/candidate Full CLI bytes changed'
  return digest
 return None

def framed_jsonl(path):
 raw=Path(path).read_bytes();assert raw.endswith(b'\n') and b'\r' not in raw
 lines=raw.splitlines();assert all(lines)
 return [check.strict_json(line) for line in lines]

def tuple_hex(occurrence):
 fields=[occurrence['capture']['namespace'],occurrence['capture']['sha256'],occurrence['locator']]
 return b''.join(len(v.encode()).to_bytes(8,'big')+v.encode() for v in fields).hex()

def expected_facts(cfg,tiny):
 spec=json.loads(leaf(cfg['workload']).read_bytes())
 if tiny:
  ctx=json.loads(leaf(cfg['tiny_context'],HERE).read_bytes());assert ctx['fixture_scope']=='original-four-pack-synthetic-readable'
  keys=ctx['expected_keys'];rows=[(k['source'],tuple_hex(k['occurrence']),aid) for aid,k in keys.items()]
 else:
  witness=json.loads(leaf(WITNESS).read_bytes())
  rows=[(a['target']['Catalog']['source'],tuple_hex(a['assertion']['occurrence']),a['assertion']['id']) for inp in witness['inputs'] for a in inp['farvi']]
 rows.sort();assert len(rows)==len({r[2] for r in rows})==5
 assert set(cfg['expected_assertion_ids'])=={r[2] for r in rows}
 answer={};positions={};aliases={'LSK':'lensisku','LLG':'llg','SUT':'sutysisku'}
 for request in spec:
  if request['operation']=='exact':
   sources=[aliases[v] for v in request.get('sources',[])]
   selected=[r for r in rows if (not sources or r[0] in sources)] if request['spelling']=='farvi' else []
   start=positions[request['cursor_from']] if request.get('cursor_from') else 0
   n=request.get('page_size');page=selected[start:] if n is None else selected[start:start+n]
   positions[request['id']]=start+len(page)
   totals={source:sum(r[0]==source for r in selected) for source in sorted({r[0] for r in selected})}
   facts={'operation':'exact','totals':{'occurrences':len(selected),'distinct_spellings':int(bool(selected)),'source_groups':len(totals)},'source_totals':[{'source':k,'occurrences':v} for k,v in totals.items()],'assertion_ids':[r[2] for r in page],'groups':len({r[0] for r in page})}
  else:
   identity=request['id'].removeprefix('prime--')
   if identity=='wrong-role':answer[request['id']]={'error':'invalid_reference'};continue
   if identity in ('unknown','unresolved','missing'):
    status={'unknown':'Unknown','unresolved':'Unresolved','missing':'NoMatch'}[identity];ids=[]
   else:
    target=request['target'];status='Resolved'
    if 'Local' in target:ids=[target['Local']['Assertion']]
    else:
     c=target['Catalog'];ids=[r[2] for r in rows if r[0]==c['source'] and r[1]==tuple_hex(c['occurrence'])]
    assert len(ids)==1 and ids[0] in cfg['expected_assertion_ids']
   facts={'operation':'resolve','status':status,'reason':None,'confirmed_ids':ids,'unverified_ids':[]}
  answer[request['id']]=facts
 assert len(answer)==54;return answer

def execute(cfg,*,tiny):
 assert cfg['readiness_status']==('TINY_METHOD_FROZEN_HANDOFF' if tiny else 'ROOT_REVIEWED_EXACT_FROZEN_NUMERICAL_CAMPAIGN')
 root,config,out=map(Path,(cfg['root'],cfg['config'],cfg['output']))
 if tiny:
  assert root==config.parent/'root' and config.is_relative_to(HERE) and Path(cfg['workload']).is_relative_to(HERE)
  assert out.is_relative_to(HERE) and out.name.startswith('numeric-tiny-')
 else:
  assert (root,config,out)==(ROOT,CONFIG,OUTPUT)
  assert Path(cfg['baseline'])==BASELINE and sha(BASELINE)==BASELINE_SHA
  assert Path(cfg['workload'])==WORK and sha(WORK)==WORK_SHA
  assert sha(WITNESS)==WITNESS_SHA
  expected=[a['assertion']['id'] for r in json.loads(WITNESS.read_bytes())['inputs'] for a in r['farvi']]
  assert cfg['expected_assertion_ids']==expected
  assert cfg['catalogue_revision']==4 and cfg['profile_revision']==5
 assert root.is_absolute() and config.is_absolute() and out.is_absolute() and '..' not in out.parts
 assert not out.exists() and not out.is_symlink()
 assert Path(cfg['guard'])==GUARD
 source=Path(cfg['source']);env_git={**os.environ,'GIT_OPTIONAL_LOCKS':'0'}
 assert subprocess.check_output(['git','--no-optional-locks','-C',str(source),'rev-parse','HEAD'],env=env_git).decode().strip()==cfg['source_commit']
 assert subprocess.check_output(['git','--no-optional-locks','-C',str(source),'rev-parse','HEAD^{tree}'],env=env_git).decode().strip()==cfg['source_tree']
 assert not subprocess.check_output(['git','--no-optional-locks','-C',str(source),'status','--porcelain=v1'],env=env_git)
 for f in cfg['frozen_files']:pin(f)
 assert len({f['path'] for f in cfg['frozen_files']})==len(cfg['frozen_files'])
 closure={f['path'] for f in cfg['frozen_files']}
 for path in [cfg['cli'],cfg['observer'],cfg['baseline'],str(GUARD),__file__,str(HERE/'numeric-analysis01.py'),str(HERE/'cli-checks01.py'),str(cfg['workload']),str(config)]:assert str(Path(path).resolve()) in closure,path
 # Known G4 store is fixed by final exact preflight; NO receipt-derived unvalidated traversal.
 databases=[leaf(p,root) for p in cfg['database_paths']]
 assert len(databases)==2 and len(set(databases))==2
 assert root/'catalogue.sqlite3' in databases
 assert next(p for p in databases if p.name!='catalogue.sqlite3').name=='dictionary.sqlite3'
 assert all(str(p) in cfg['expected_database_sha256'] for p in databases)
 assert shutil.disk_usage(out.parent).free>=cfg['minimum_free_bytes']
 scratch=list(map(Path,cfg['new_s05_scratch_paths']));assert HERE in scratch
 target=Path(cfg['target_cache']);assert target==Path('/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc01/target')
 assert not any(canonical_system_tmp(target).is_relative_to(canonical_system_tmp(p)) or canonical_system_tmp(p).is_relative_to(canonical_system_tmp(target)) for p in scratch)
 assert not any(canonical_system_tmp(ROOT).is_relative_to(canonical_system_tmp(p)) or canonical_system_tmp(p).is_relative_to(canonical_system_tmp(ROOT)) for p in scratch)
 before_scratch=inventory(scratch);assert before_scratch['bytes']<=12*GIB
 cache_before=target_cache_bytes(target)
 expected=expected_facts(cfg,tiny)
 stages=expanded(cfg,tiny);assert stages==cfg['expanded_stages']
 out.mkdir(mode=0o700)
 isolated={k:str(out/n) for k,n in [('HOME','isolated-home'),('XDG_DATA_HOME','isolated-data'),('XDG_CONFIG_HOME','isolated-config')]}
 for p in isolated.values():Path(p).mkdir(mode=0o700)
 childenv={'PATH':'/usr/bin:/bin:/usr/sbin:/sbin','LANG':'en_US.UTF-8','LC_ALL':'en_US.UTF-8',**isolated}
 new_json(out/'environment.json',childenv);new_json(out/'expanded-stages.json',stages)
 new_json(out/'s05-scratch-before.json',before_scratch);new_json(out/'target-cache-before.json',{'path':str(target),'bytes':cache_before,'mechanism':'du -sk logical allocated KiB observation'})
 new_json(out/'pre-input-normalized-facts.json',expected)
 before={str(p):sha(p) for p in databases} # ONE whole-campaign pair; this warms OS cache, NOT a cold-cache trial.
 new_json(out/'database-before.json',before)
 assert before==cfg['expected_database_sha256']
 actual=[];failed=None;full_hash=None;started=time.monotonic()
 try:
  for number,stage in enumerate(stages,1):
   if stage['kind']=='warm' and number==13:
    assert len(actual)==12
    fresh_verdict=fresh_phase_budget(actual);new_json(out/'fresh-phase-budget.json',fresh_verdict)
    assert fresh_verdict['ok'],fresh_verdict['failed']
   if stage['kind']=='semantic':
    assert number==22 and len(actual)==21
    spec=json.loads(Path(cfg['workload']).read_bytes());verdict=analysis.analyze(actual,spec);new_json(out/'numerical-analysis.json',verdict)
    assert verdict['ok'],verdict['failed']
   name=f'{number:02d}-{stage["kind"]}-{stage["round"] or 0}-{stage["mode"] or "semantic"}'
   workdir=out/(f'warm-{stage["round"]}-{stage["mode"]}' if stage['kind']=='warm' else 'semantic' if stage['kind']=='semantic' else name)
   workdir.mkdir(mode=0o700)
   stdout=workdir/'stdout';stderr=workdir/'stderr';guardout=workdir/'guard.stdout';guarderr=workdir/'guard.stderr'
   argv=[cfg['python'],str(GUARD),'--timeout','600','--max-rss-bytes',str(8*GIB),'--log',str(stderr),'--stdout',str(stdout),'--',*stage['argv']]
   with guardout.open('xb') as o,guarderr.open('xb') as e:
    result=subprocess.run(argv,stdout=o,stderr=e,env=childenv)
   receipt=check.strict_json(guardout.read_bytes());new_json(workdir/'receipt.json',receipt)
   record={**stage,'number':number,'guard_argv':argv,'guard_exit':result.returncode,'receipt':receipt,'stdout_bytes':stdout.stat().st_size,'stdout_sha256':sha(stdout),'stderr_bytes':stderr.stat().st_size,'stderr_sha256':sha(stderr)}
   actual.append(record);new_json(workdir/'stage.json',record)
   assert result.returncode==0 and receipt['decision']=='passed' and receipt['exit_code']==0
   assert receipt['timed_out'] is False and receipt['rss_parse_ok'] is True and 0<receipt['maximum_resident_set_size_bytes']<=8*GIB
   assert guarderr.read_bytes()==b''
   assert record['stdout_bytes']+record['stderr_bytes']<=128*1024**2
   if stage['kind']=='fresh':
    record['elapsed_ns']=round(receipt['elapsed_seconds']*1_000_000_000)
    record['elapsed_precision']='guard monotonic seconds, ~50ms polling; converted ns does not imply ns precision'
    limit=10 if stage['mode'] in ('baseline-full','candidate-full') else 9
    assert receipt['elapsed_seconds']<=limit,'fresh maximum'
    digest=fresh_wire(stdout.read_bytes(),stage['mode'],cfg['expected_assertion_ids'],full_hash)
    if digest is not None:full_hash=digest
   elif stage['kind']=='warm':
    record['rows']=framed_jsonl(stdout)
    # Full validation + per-identity medians/maxima runs after all nine stages;
    # per-observation maximum gates are inside SAME observer and stop on failure.
    assert len(record['rows'])==56 and record['rows'][-1]['row_type']=='complete'
    assert record['rows'][0]['sqlite_runtime_version']==cfg['sqlite_runtime_version']
    requests=record['rows'][1:-1]
    assert [r['id'] for r in requests]==list(expected)
    for r in requests:
     want=expected[r['id']]
     if 'error' in want:assert r['ok'] is False and r['expected_error']==want['error'] and r['profile'] is None
     else:assert r['ok'] is True and r['facts']==want,(stage['mode'],stage['round'],r['id'],r.get('facts'),want)
   else:
    rows=framed_jsonl(stdout);assert len(rows)==1 and rows[0]['row_type']=='semantic_complete' and rows[0]['ok'] is True
    assert rows[0]['tiny'] is tiny and rows[0]['accepted_full_inner_sha_checked'] is (not tiny)
    record['rows']=rows
   new_json(workdir/'checked-stage.json',record)
   observed=inventory(scratch);assert observed['bytes']<=12*GIB,'new numerical scratch observed threshold'
   print(json.dumps({'stage':number,'kind':stage['kind'],'mode':stage['mode'],'round':stage['round'],'decision':'passed','elapsed_seconds':receipt['elapsed_seconds'],'rss_bytes':receipt['maximum_resident_set_size_bytes']}),flush=True)
 except BaseException as e:
  failed={'type':type(e).__name__,'detail':str(e),'traceback':traceback.format_exc()};raise
 finally:
  after={str(p):sha(p) for p in databases};new_json(out/'database-after.json',after)
  inventory_=inventory(scratch);new_json(out/'inventory-before-summary.json',inventory_)
  cache_after=target_cache_bytes(target);new_json(out/'target-cache-after.json',{'path':str(target),'bytes':cache_after,'delta_bytes':cache_after-cache_before})
  summary={'ok':failed is None and len(actual)==22 and before==after and cache_after==cache_before and inventory_['bytes']<=12*GIB,'tiny_method':tiny,'completed_or_attempted_stages':len(actual),'elapsed_seconds':time.monotonic()-started,'failure':failed,'database_before':before,'database_after':after,'database_unchanged':before==after,'observed_inventory_bytes':inventory_['bytes'],'target_cache_before_bytes':cache_before,'target_cache_after_bytes':cache_after,'target_cache_delta_bytes':cache_after-cache_before,'settled_delta':'inventory-before-summary.json, target-cache-after.json and summary.json are authored AFTER this observation and excluded from its exact bytes; final external manifest will include both','resource_mechanism':'600s process-group timeout active; RSS8GiB/output128MiB/new scratch12GiB checked AFTER each process, not live enforcement','fresh_semantics':'fresh PROCESS only; one prehash pair warmed OS cache; no cache controls','proof_scope':'same-call timings + exact3 observations per identity; no pooling/retry; one semantic observer; errors retained; no detail latency cap'}
  new_json(out/'summary.json',summary)
  if before!=after:raise AssertionError('retained DB/catalogue bytes changed')
  if cache_before!=cache_after:raise AssertionError('quiet retained target cache changed')
  if inventory_['bytes']>12*GIB:raise AssertionError('final named S05 scratch exceeds observed threshold')

if __name__=='__main__':
 assert len(sys.argv)==2,'numeric-runner01.py EXACT_FROZEN_PREFLIGHT.json'
 cfg=json.loads(Path(sys.argv[1]).read_bytes());execute(cfg,tiny=False)
