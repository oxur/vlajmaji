#!/usr/bin/env python3
"""UNEXECUTED DRAFT. Frozen-source CDC point-of-use; no Cargo/network/default paths.
24 product children: same public example once + 23 CLI children. Inputs/expected hits
are derived before child1; no product output/export supplies their expected bodies.
Final source/binary/env bindings and explicit QUIET transfer required before launch.
"""
import hashlib,json,os,re,subprocess,sys,tempfile,time
from pathlib import Path


def sha(b): return hashlib.sha256(b).hexdigest()
def enc(v): return json.dumps(v,ensure_ascii=False,separators=(',',':')).encode()
def filepin(p):
 b=Path(p).read_bytes();return {'bytes':len(b),'sha256':sha(b)}
def git(*args):
 return subprocess.check_output(['git','--no-optional-locks','-C',str(src),*args],env=env,text=True).strip()
def expect_keys(v,names): assert set(v)==set(names.split()),(set(v),names)
def one(rows,pred):
 found=[x for x in rows if pred(x)];assert len(found)==1,found;return found[0]
def occurrence_id(ref):
 parts=[ref['capture']['namespace'],ref['capture']['sha256'],ref['locator']]
 return b''.join(len(x.encode()).to_bytes(8,'big')+x.encode() for x in parts).hex()

if len(sys.argv)!=2: raise SystemExit('binding JSON required; this draft is not issued')
binding=json.loads(Path(sys.argv[1]).read_bytes())
assert binding['status']=='FROZEN_CDC_VERIFICATION', 'no draft/runtime authority'
src=Path(binding['source']);cli=Path(binding['cli']);example=Path(binding['example'])
# Preserve literal incoming PATH and toolhomes before replacing ONLY owned HOME/XDG/TMPDIR.
env=os.environ.copy();assert sha(env['PATH'].encode())==binding['path_sha256']
for k in ['CARGO_HOME','RUSTUP_HOME']: assert env[k]==binding['toolhomes'][k]
base=Path(tempfile.mkdtemp(prefix='vlajmaji-a06-s01-cdc-',dir='/private/tmp'))
for k,leaf in [('HOME','home'),('XDG_DATA_HOME','xdg-data'),('XDG_CONFIG_HOME','xdg-config'),('TMPDIR','tmp')]:
 p=base/leaf;p.mkdir();env[k]=str(p)
env['CARGO_NET_OFFLINE']='true';env['CARGO_TARGET_DIR']=binding['target']
(base/'selected-env.json').write_bytes(enc({k:env[k] for k in ['PATH','CARGO_HOME','RUSTUP_HOME','HOME','XDG_DATA_HOME','XDG_CONFIG_HOME','TMPDIR','CARGO_TARGET_DIR','CARGO_NET_OFFLINE']}))
assert git('rev-parse','HEAD')==binding['head'];assert git('rev-parse','HEAD^{tree}')==binding['tree']
assert git('branch','--show-current')=='codex/project01-integrated-workbench'
assert git('status','--porcelain','--untracked-files=all')==''
for path,pin in binding['files'].items(): assert filepin(path)==pin,path
assert str(cli) in binding['files'] and str(example) in binding['files']
assert len(binding['source_files'])==14
for rel in binding['source_files']: assert str(src/rel) in binding['files'],rel

# Whole fixed pre-input authority: 11 original example INPUT files, before any child.
inputs={}
for name,pin in binding['inputs'].items():
 p=Path(binding['input_root'])/name;b=p.read_bytes();assert filepin(p)==pin
 inputs[name]=b
assert set(inputs)=={'expected.json','historical-selection.json','owned-r1.json','owned-r2.json','owned-seed.json','request.json','selection.json','upstream-dictionary.json','upstream-release.json','upstream-seal.json','upstream.json'}
up=json.loads(inputs['upstream-dictionary.json']);r1=json.loads(inputs['owned-r1.json']);r2=json.loads(inputs['owned-r2.json'])
request=json.loads(inputs['request.json']);assert request['operation']=='exact' and request['spelling']=='farvi'
assert [a['definition']['Value']['text'] for a in r1['metadata']['assertions']]==['grow','develop']
assert [a['definition']['Value']['text'] for a in r2['metadata']['assertions']]==['grow','develop','flourish']
assert up['metadata']['assertions'][0]['definition']['Value']=={'text':'advance','language':{'Value':'en'},'markup':'Missing'}
# Fixed input has no editorial/annotation families; rich-schema2 and rich typed
# transport remain independently source-reviewed, author-tested census controls.
empty=['contexts','frames','forms','etymologies','relations','editorial_senses','computations','annotations','extensions']
for d in [up,r1,r2]:
 for k in empty: assert d['metadata'][k]==[],k

def expected_hit(d,a):
 m=d['metadata'];reg=d['registry'];e=one(m['entries'],lambda e:e['id']==a['entry'])
 c=one(d['captures'],lambda c:c['identity']==a['occurrence']['capture'])
 raw=one(c['occurrences'],lambda r:r['locator']==a['occurrence']['locator'])
 snap=one(reg['snapshots'],lambda r:r['snapshot']==c['identity'])
 attr=one(reg['occurrences'],lambda r:r['occurrence']=={'snapshot':c['identity'],'locator':raw['locator']})
 contributor=next((x for x in reg['contributors'] if x['id']==attr['contributor']),None)
 distributor=next((x for x in reg['distributors'] if x['id']==snap['distributor']),None)
 return {'occurrence_id':occurrence_id(a['occurrence']),'entry':e,'assertion':a,'capture':c['metadata'],'raw':raw,
  'snapshot_attribution':snap,'occurrence_attribution':attr,'contributor':contributor,'distributor':distributor,
  'release_ids':[r['id'] for r in m['releases'] if r['selected'] and r['availability']=='Available' and a['id'] in r['assertions']],
  **{k:[] for k in empty},
  'owned_revision_ids':[r['id'] for r in m['owned_revisions'] if a['id'] in r['assertions']],
  'tombstoned_by':[r['id'] for r in m['owned_revisions'] if a['id'] in r['tombstones']]}
def roster(own):
 hits=[expected_hit(up,a) for a in up['metadata']['assertions']]+[expected_hit(own,a) for a in own['metadata']['assertions']]
 return sorted(hits,key=lambda h:(h['entry']['source'],h['entry']['spelling']['Value'],h['occurrence_id']))
wanted1=roster(r1);wanted2=roster(r2);assert len(wanted1)==3 and len(wanted2)==4
sources=[one(up['registry']['sources'],lambda row:row['id']=='lensisku'),one(r1['registry']['sources'],lambda row:row['id']=='owned-example')]
assert one(r2['registry']['sources'],lambda row:row['id']=='owned-example')==sources[1]
expected={'sources':sources,'r1_hits':wanted1,'r2_hits':wanted2,'r1_source_order':['lensisku','owned-example'],'r1_counts':[1,2],'r2_counts':[1,3]}
(base/'expected-before-children.json').write_bytes(enc(expected))
children=[]

def run(name,args,code=0,json_output=True):
 n=len(children)+1;assert n<=24
 started=time.time();argv=[str(x) for x in args]
 try:
  p=subprocess.run(argv,cwd=base,env=env,capture_output=True,timeout=180)
  out,err=p.stdout,p.stderr;actual=p.returncode;timedout=False
 except subprocess.TimeoutExpired as ex:
  out=ex.stdout or b'';err=ex.stderr or b'';actual=None;timedout=True
 outpath=base/f'{n:02}-{name}.stdout';errpath=base/f'{n:02}-{name}.stderr'
 with outpath.open('xb') as f:f.write(out)
 with errpath.open('xb') as f:f.write(err)
 row={'n':n,'name':name,'argv':argv,'cwd':str(base),'expected_exit':code,'actual_exit':actual,'timeout':timedout,'seconds':time.time()-started,'stdout':str(outpath),'stderr':str(errpath),'stdout_pin':filepin(outpath),'stderr_pin':filepin(errpath)}
 children.append(row);(base/'commands.json').write_bytes(enc(children))
 assert not timedout and actual==code,row
 if code:
  assert out==b'' and err.endswith(b'\n') and err.count(b'\n')==1
  error=json.loads(err);expect_keys(error,'error');expect_keys(error['error'],'code message sources');assert isinstance(error['error']['sources'],list);return error
 assert err==b'',(name,err)
 if json_output:
  assert out.endswith(b'\n')
  if name!='public-example': assert out.count(b'\n')==1
  return json.loads(out)
 return out.decode()

def v(name,*args,**kw):return run(name,[cli,*args],**kw)
def full_check(outer,wanted,rev,mode,schema=1):
 expect_keys(outer,'version binding_sha256 selection result');assert outer['version']==1 and re.fullmatch('[0-9a-f]{64}',outer['binding_sha256'])
 s=outer['selection'];expect_keys(s,'version packs disabled upstream owned context_counts curation_counts')
 assert s['packs']==['la-lojban'] and s['disabled']==[] and s['context_counts']=={'declared':0,'provided':0}
 own=s['owned'];expect_keys(own,'mode source revision sequence journal_schema state_sha256 curation_sha256')
 assert own['mode']==mode and own['source']=='owned-example' and own['revision']==rev and own['sequence']==int(rev[-1]) and own['journal_schema']==schema
 assert own['state_sha256']==sha(inputs[f'{rev}.json'])
 if schema==1: assert own['curation_sha256'] is None and s['curation_counts'] is None
 else:
  assert re.fullmatch('[0-9a-f]{64}',own['curation_sha256'])
  assert s['curation_counts']=={'records':0,'projected':0,'inactive':0,'unavailable':0,'unresolved':0,'resolved':0}
 assert outer['result']['operation']=='exact';r=outer['result']['response']
 assert r['spelling']=='farvi' and r['resolved_sources']==['lensisku','owned-example']
 assert r['totals']=={'occurrences':len(wanted),'distinct_spellings':1,'source_groups':2}
 assert [g['source'] for g in r['groups']]==sources
 assert r['source_totals']==[{'source':source,'occurrences':count,'distinct_spellings':1} for source,count in zip(sources,[1,len(wanted)-1])]
 assert [g['full_count'] for g in r['groups']]==[1,len(wanted)-1]
 assert all(g['returned_count']==len(g['hits']) and not g['partial'] for g in r['groups']) and r['next_cursor'] is None
 assert [h for g in r['groups'] for h in g['hits']]==wanted
 return r

def compact_check(outer,full,wanted):
 assert outer['binding_sha256']==full['binding_sha256'] and outer['selection']==full['selection']
 r=outer['result']['response'];assert outer['result']['operation']=='exact'
 assert r['totals']==full['result']['response']['totals'] and r['next_cursor'] is None
 hs=[h for g in r['groups'] for h in g['hits']];assert len(hs)==len(wanted)
 for h,w in zip(hs,wanted):
  expect_keys(h,'occurrence_id entry assertion capture snapshot_attribution occurrence_attribution contributor distributor release_ids detail_ref')
  assert {k:x for k,x in h.items() if k!='detail_ref'}=={k:w[k] for k in h if k!='detail_ref'}
  ref=h['detail_ref'];expect_keys(ref,'version digest_policy role source entry_id assertion_id occurrence entry_sha256 assertion_sha256 native_entry_id native_assertion_id contributor_id distributor_id')
  assert ref['entry_sha256']==sha(enc(w['entry'])) and ref['assertion_sha256']==sha(enc(w['assertion']))
  assert ref['source']==w['entry']['source'] and ref['entry_id']==w['entry']['id'] and ref['assertion_id']==w['assertion']['id'] and ref['occurrence']==w['assertion']['occurrence']
  attr=w['occurrence_attribution'];assert ref['native_entry_id']==attr['native_entry_id'] and ref['native_assertion_id']==attr['native_assertion_id'] and ref['contributor_id']==attr['contributor'] and ref['distributor_id']==w['snapshot_attribution']['distributor']
 return hs

def readable_check(text,full,wanted):
 lines=text.splitlines();assert lines[0].startswith('workbench_binding: ') and json.loads(lines[0][len('workbench_binding: '):])==full['binding_sha256']
 assert lines[1].startswith('workbench_selection: ') and json.loads(lines[1][len('workbench_selection: '):])==full['selection']
 assert lines[2]==f'Exact "farvi": {len(wanted)} occurrences, 1 spellings, 2 sources'
 rows=[x for x in lines if x.startswith('  ') and ' @ ' in x]
 assert rows==[f"  {h['assertion']['id']} @ {h['occurrence_id']}" for h in wanted]
 for label,key in [('    spelling: ','entry'),('    definition: ','assertion')]:
  actual=[json.loads(x[len(label):]) for x in lines if x.startswith(label)]
  assert actual==[h[key]['spelling' if key=='entry' else 'definition'] for h in wanted]

work=base/'workbench-example'
run('public-example',[example,work])
for name,b in inputs.items():assert (work/'inputs'/name).read_bytes()==b,name
I=work/'inputs';C=work/'catalogue';J=base/'cli-owned';W=base/'head-r1';H=base/'historical-r1';W2=base/'head-r2';M=base/'migrated-r2'
v('init','owned','init',J,'--input',I/'owned-seed.json')
v('edit-r1','owned','edit',J,'--expected-head','owned-r0','--input',I/'owned-r1.json')
v('build-head-r1','workbench','build',W,'--selection',I/'selection.json','--catalogue',C,'--owned',J)
info=v('info-r1','workbench','info',W);assert info['binding']['owned']['identity']['revision']=='owned-r1'
f1=v('full-r1','workbench','query',W,'--request',I/'request.json','--format','full');full_check(f1,wanted1,'owned-r1','head')
c1=v('compact-r1','workbench','query',W,'--request',I/'request.json','--format','compact');keys=compact_check(c1,f1,wanted1)
readable_check(v('readable-r1','workbench','query',W,'--request',I/'request.json','--format','readable',json_output=False),f1,wanted1)
ref=base/'detail.json';ref.write_bytes(enc(keys[0]['detail_ref']))
detail=v('detail','workbench','detail',W,'--reference',ref,'--expected-binding',f1['binding_sha256'],'--format','full')
assert detail['binding_sha256']==f1['binding_sha256'] and detail['selection']==f1['selection']
assert detail['result']['hit']==wanted1[0] and detail['result']['status']=='Resolved' and detail['result']['current_eligible'] is True
rd=v('detail-readable','workbench','detail',W,'--reference',ref,'--expected-binding',f1['binding_sha256'],'--format','readable',json_output=False)
assert rd.startswith('workbench_binding: '+json.dumps(f1['binding_sha256'])+'\nworkbench_selection: ')
v('build-historical','workbench','build',H,'--selection',I/'historical-selection.json','--catalogue',C,'--owned',J)
hbefore=v('historical-before','workbench','query',H,'--request',I/'request.json','--format','full');full_check(hbefore,wanted1,'owned-r1',{'revision':'owned-r1'})
v('edit-r2','owned','edit',J,'--expected-head','owned-r1','--input',I/'owned-r2.json')
assert v('head-stale','workbench','query',W,'--request',I/'request.json','--format','full',code=1)['error']['code']=='stale_workbench'
hafter=v('historical-after','workbench','query',H,'--request',I/'request.json','--format','full');full_check(hafter,wanted1,'owned-r1',{'revision':'owned-r1'});assert hafter==hbefore
v('build-r2','workbench','build',W2,'--selection',I/'selection.json','--catalogue',C,'--owned',J)
f2=v('full-r2','workbench','query',W2,'--request',I/'request.json','--format','full');full_check(f2,wanted2,'owned-r2','head')
c2=v('compact-r2','workbench','query',W2,'--request',I/'request.json','--format','compact');compact_check(c2,f2,wanted2)
assert v('wrong-outer-binding','workbench','detail',W2,'--reference',ref,'--expected-binding',f1['binding_sha256'],'--format','full',code=1)['error']['code']=='stale_workbench_binding'
v('historical-info','workbench','info',H)
v('migrate','owned','migrate-curation',J,'--expected-head','owned-r2')
assert v('migration-stale','workbench','query',W2,'--request',I/'request.json','--format','full',code=1)['error']['code']=='stale_workbench'
v('build-migrated','workbench','build',M,'--selection',I/'selection.json','--catalogue',C,'--owned',J)
fm=v('full-migrated','workbench','query',M,'--request',I/'request.json','--format','full');full_check(fm,wanted2,'owned-r2','head',2)
assert len(children)==24 and sum(x['actual_exit']==0 for x in children)==21
for name,b in inputs.items():assert (work/'inputs'/name).read_bytes()==b
for p,pin in binding['files'].items():assert filepin(p)==pin,p
assert git('rev-parse','HEAD')==binding['head'] and git('rev-parse','HEAD^{tree}')==binding['tree'] and git('status','--porcelain','--untracked-files=all')==''
(base/'execution.json').write_bytes(enc({'status':'PASS','children':children,'source':binding['head'],'tree':binding['tree'],'method_sha256':filepin(__file__)['sha256'],'bindings_sha256':filepin(sys.argv[1])['sha256'],'scope':'24 bounded children; schema1 mixed/historical/detail and explicit empty-schema2 migration; rich nonempty curation/other matrix inherited source+author evidence','base':str(base)}))
print(json.dumps({'status':'PASS','base':str(base),'children':len(children)}))
