#!/usr/bin/env python3
"""UNEXECUTED DRAFT. Frozen-source CDC point-of-use; no Cargo/network/default paths.
33 product children: same public example once + 32 CLI children. Inputs/expected hits
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
base=Path(tempfile.mkdtemp(prefix='vlajmaji-a06-s02-cdc-',dir='/private/tmp'))
for k,leaf in [('HOME','home'),('XDG_DATA_HOME','xdg-data'),('XDG_CONFIG_HOME','xdg-config'),('TMPDIR','tmp')]:
 p=base/leaf;p.mkdir();env[k]=str(p)
env['CARGO_NET_OFFLINE']='true';env['CARGO_TARGET_DIR']=binding['target'];env['RUSTUP_AUTO_INSTALL']='0'
(base/'selected-env.json').write_bytes(enc({k:env[k] for k in ['PATH','CARGO_HOME','RUSTUP_HOME','HOME','XDG_DATA_HOME','XDG_CONFIG_HOME','TMPDIR','CARGO_TARGET_DIR','CARGO_NET_OFFLINE','RUSTUP_AUTO_INSTALL']}))
assert git('rev-parse','HEAD')==binding['head'];assert git('rev-parse','HEAD^{tree}')==binding['tree']
assert git('branch','--show-current')=='codex/project01-integrated-workbench'
assert git('status','--porcelain','--untracked-files=all')==''
for path,pin in binding['files'].items(): assert filepin(path)==pin,path
assert all(str(p) in binding['files'] for p in [cli,example,Path(__file__)])
assert len(binding['source_files'])==15
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
# FIXTURE-SPECIFIC singleton composition identity, not a general compose claim.
# Immutable4fc pack.rs388-630 preserves first-seen keyed rows/max catalogue;
# only anonymous lineage/notices and extensions can normalize this one input.
# This fixed validated S01 authority has no lineage/extensions, one notice and
# unique typed keyed rows; dictionary.rs1063 serializes the typed model directly.
assert up['version']==1 and up['registry']['version']==1 and up['legacy_archive'] is None
assert up['registry']['lineage']==[] and len(up['metadata']['code_notices'])==1 and up['metadata']['extensions']==[] and up['metadata']['catalogue_revision']==0
for family in ['collections','components','releases','entries','assertions','contexts','frames','forms','etymologies','relations','owned_revisions','editorial_senses','computations','annotations']:
 rows=up['metadata'][family];assert len({r['id'] for r in rows})==len(rows),family

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
 n=len(children)+1;assert n<=33
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

def native(p):
 b=os.fsencode(str(p));return {'encoding':'unix-bytes','hex':b.hex(),'utf8':str(p)}
def config_body(rev,default,profiles,prepared):
 return {'version':1,'kind':'vlajmaji-workbench-profiles','revision':rev,'default_profile':default,'profiles':profiles,'prepared':prepared}
def capture_config(rev,default,profiles,prepared):
 wanted=config_body(rev,default,profiles,prepared);actual=cfg.read_bytes()
 assert actual==enc(wanted)+b'\n',('config exact typed order/whole bytes',wanted,actual)
 return sha(actual)
def write_check(v,operation,changed,prior,rev,profile,default,active):
 expect_keys(v,'version operation config changed prior_revision revision sha256 profile default_profile active')
 assert v=={'version':1,'operation':operation,'config':native(cfg),'changed':changed,'prior_revision':prior,'revision':rev,'sha256':sha(cfg.read_bytes()),'profile':profile,'default_profile':default,'active':active}
def identity_check(outer,rev,requested,profile,seq,root):
 expect_keys(outer,'version binding_sha256 configuration selection result');assert outer['version']==1
 i=outer['configuration'];expect_keys(i,'version kind config config_revision config_sha256 requested_profile profile prepared_sequence view_binding_sha256 effective_binding_sha256')
 direct=json.loads((root/'binding.json').read_bytes());expected={'version':1,'kind':'vlajmaji-configured-workbench','config':native(cfg),'config_revision':rev,'config_sha256':sha(cfg.read_bytes()),'requested_profile':requested,'profile':profile,'prepared_sequence':seq,'view_binding_sha256':direct['binding_sha256']}
 expected['effective_binding_sha256']=sha(enc(expected))
 assert i==expected and outer['binding_sha256']==expected['effective_binding_sha256']
 assert outer['binding_sha256']!=i['view_binding_sha256']
 return i

def full_check(outer,wanted,rev,requested,profile,seq,root,ownrev,mixed):
 identity_check(outer,rev,requested,profile,seq,root)
 s=outer['selection'];expect_keys(s,'version packs disabled upstream owned context_counts curation_counts')
 assert s['packs']==(['la-lojban'] if mixed else []) and s['disabled']==[]
 assert s['context_counts']=={'declared':0,'provided':0} and s['curation_counts'] is None
 own=s['owned'];expect_keys(own,'mode source revision sequence journal_schema state_sha256 curation_sha256')
 assert own=={'mode':'head','source':'owned-example','revision':ownrev,'sequence':int(ownrev[-1]),'journal_schema':1,'state_sha256':sha(inputs[ownrev+'.json']),'curation_sha256':None}
 if not mixed:assert s['upstream'] is None
 r=outer['result']['response'];assert outer['result']['operation']=='exact' and r['spelling']=='farvi'
 ss=sources if mixed else sources[1:];counts=[1,len(wanted)-1] if mixed else [len(wanted)]
 assert r['resolved_sources']==[x['id'] for x in ss]
 assert r['totals']=={'occurrences':len(wanted),'distinct_spellings':1,'source_groups':len(ss)}
 assert [g['source'] for g in r['groups']]==ss
 assert r['source_totals']==[{'source':source,'occurrences':count,'distinct_spellings':1} for source,count in zip(ss,counts)]
 assert [g['full_count'] for g in r['groups']]==counts
 assert all(g['returned_count']==len(g['hits']) and not g['partial'] for g in r['groups']) and r['next_cursor'] is None
 assert [h for g in r['groups'] for h in g['hits']]==wanted
 return r

def compact_check(outer,full,wanted):
 assert outer['binding_sha256']==full['binding_sha256'] and outer['configuration']==full['configuration'] and outer['selection']==full['selection']
 r=outer['result']['response'];assert outer['result']['operation']=='exact'
 assert r['totals']==full['result']['response']['totals'] and r['next_cursor'] is None
 fr=full['result']['response'];assert r['source_totals']==fr['source_totals'] and r['resolved_sources']==fr['resolved_sources']
 assert [{k:g[k] for k in ['source','full_count','returned_count','partial']} for g in r['groups']]==[{k:g[k] for k in ['source','full_count','returned_count','partial']} for g in fr['groups']]

 hs=[h for g in r['groups'] for h in g['hits']];assert len(hs)==len(wanted)
 for h,w in zip(hs,wanted):
  expect_keys(h,'occurrence_id entry assertion capture snapshot_attribution occurrence_attribution contributor distributor release_ids detail_ref')
  assert {k:x for k,x in h.items() if k!='detail_ref'}=={k:w[k] for k in h if k!='detail_ref'}
  ref=h['detail_ref'];expect_keys(ref,'version digest_policy role source entry_id assertion_id occurrence entry_sha256 assertion_sha256 native_entry_id native_assertion_id contributor_id distributor_id')
  assert ref['version']==1 and ref['digest_policy']=='entry-assertion-serde-json-v1' and ref['role']=='assertion'
  assert ref['entry_sha256']==sha(enc(w['entry'])) and ref['assertion_sha256']==sha(enc(w['assertion']))
  assert ref['source']==w['entry']['source'] and ref['entry_id']==w['entry']['id'] and ref['assertion_id']==w['assertion']['id'] and ref['occurrence']==w['assertion']['occurrence']
  attr=w['occurrence_attribution'];assert ref['native_entry_id']==attr['native_entry_id'] and ref['native_assertion_id']==attr['native_assertion_id'] and ref['contributor_id']==attr['contributor'] and ref['distributor_id']==w['snapshot_attribution']['distributor']
 return hs

def readable_check(text,full,wanted):
 lines=text.splitlines()
 for line,label,key in zip(lines[:3],['workbench_binding: ','workbench_configuration: ','workbench_selection: '],['binding_sha256','configuration','selection']):
  assert line.startswith(label) and json.loads(line[len(label):])==full[key]
 assert lines[3]==f'Exact "farvi": {len(wanted)} occurrences, 1 spellings, {len(full["result"]["response"]["groups"])} sources'
 rows=[x for x in lines if x.startswith('  ') and ' @ ' in x]
 assert rows==[f"  {h['assertion']['id']} @ {h['occurrence_id']}" for h in wanted]
 for label,key in [('    spelling: ','entry'),('    definition: ','assertion')]:
  actual=[json.loads(x[len(label):]) for x in lines if x.startswith(label)]
  assert actual==[h[key]['spelling' if key=='entry' else 'definition'] for h in wanted]

def owned_identity(ownrev):
 return {'source':'owned-example','journal_schema':1,'revision':ownrev,'sequence':int(ownrev[-1]),'state_sha256':sha(inputs[ownrev+'.json']),'curation_state_sha256':None}
def release_summary(r):
 return {'id':r['id'],'component_id':r['component'],'distributor_id':r['distributor'],**{k:r[k] for k in ['upstream_revision','distributor_revision','captured_at','time_precision','schema_version','normalizer','normalizer_version','captures','accounting']},'diagnostics_count':len(r['diagnostics']),'selected':r['selected'],'availability':r['availability'],'content_notices_count':len(r['content_notices']),'content_notice_licenses':[n['license'] for n in r['content_notices']],'content_notice_locators':[n['locator'] for n in r['content_notices']],'redistribution':r['redistribution'].lower(),'redistribution_evidence':r['redistribution_evidence']}
def expected_member():
 return {'pack_id':'la-lojban','release_sha256':sha(inputs['upstream-release.json']),'payload_kind':'synthetic'}
# Generation IDs/content IDs are produced opaque identities: verify presence,
# input-defined state/member contents and cross-report coherence, not guessed values.
catalogue_observation=None
def refresh_check(v,prior,rev,profile,default,sequence,root):
 expect_keys(v,'version operation publication view');assert v['version']==1 and v['operation']=='refresh'
 view=v['view'];expect_keys(view,'version operation root binding')
 assert view['version']==1 and view['operation']=='build_new' and view['root']==native(root)
 raw=(root/'binding.json').read_bytes();direct=json.loads(raw);assert direct==view['binding']
 mixed=profile=='alpha';ownrev='owned-r2' if sequence==2 else 'owned-r1'
 assert direct['selection']==json.loads(alpha if mixed else beta)
 assert direct['owned']=={'root':native(J),'selection':'head','identity':owned_identity(ownrev)}
 assert direct['contexts']==[] and direct['resolutions']==[]
 if mixed:
  assert catalogue_observation is not None and direct['catalogue']==catalogue_observation
 else:assert direct['catalogue'] is None
 names='version kind selection catalogue owned contexts resolutions selected_release_ids state_sha256'.split()
 assert direct['binding_sha256']==sha(enc({k:direct[k] for k in names}))
 prepared={'sequence':sequence,'profile':profile,'root':native(root),'binding_sha256':direct['binding_sha256']}
 write_check(v['publication'],'refresh',True,prior,rev,profile,default,prepared)
 return prepared

def metadata_check(v,rev,default,profiles,prepared):
 expect_keys(v,'version kind config revision sha256 default_profile profiles prepared')
 assert v=={'version':1,'kind':'vlajmaji-workbench-profiles','config':native(cfg),'revision':rev,'sha256':sha(cfg.read_bytes()),'default_profile':default,'profiles':profiles,'prepared':prepared}
def history_check(v,rev,profile,prepared):
 expect_keys(v,'version config revision sha256 profile prepared')
 assert v=={'version':1,'config':native(cfg),'revision':rev,'sha256':sha(cfg.read_bytes()),'profile':profile,'prepared':prepared}
def status_check(v,rev,requested,profile,prepared,mixed,ownrev):
 global catalogue_observation
 expect_keys(v,'version scope config config_revision config_sha256 requested_profile profile active catalogue owned')
 assert v['version']==1 and v['scope']=='configured-metadata-and-active-minimum-v1' and v['config']==native(cfg)
 assert v['config_revision']==rev and v['config_sha256']==sha(cfg.read_bytes()) and v['requested_profile']==requested and v['profile']==profile
 if prepared is None:assert v['active'] is None
 else:assert v['active']=={'prepared':prepared,'state':'current','error':None}
 if not mixed:assert v['catalogue'] is None
 else:
  c=v['catalogue'];expect_keys(c,'root local_revision generation packs');assert c['root']==native(C) and c['local_revision']==1
  assert [p['pack_id'] for p in c['packs']]==['la-lojban']
  p=c['packs'][0];expect_keys(p,'pack_id selected disabled installation_state installed_release_sha256 registrations')
  assert p['selected'] is True and p['disabled'] is False and p['installed_release_sha256']==sha(inputs['upstream-release.json'])
  assert p['installation_state']=='installed'
  assert p['registrations']==[{'release_sha256':sha(inputs['upstream-release.json']),'payload_kind':'synthetic','sources':up['registry']['sources'],'releases':[release_summary(r) for r in up['metadata']['releases']]}]
  g=c['generation'];expect_keys(g,'generation_id content_id state_sha256 members')
  assert isinstance(g['generation_id'],str) and g['generation_id'] and isinstance(g['content_id'],str) and g['content_id']
  assert g['state_sha256']==sha(inputs['upstream-dictionary.json']) and g['members']==[expected_member()]
  observed={'root':native(C),'local_revision':1,'generation_id':g['generation_id'],'content_id':g['content_id'],'recorded_state_sha256':g['state_sha256'],'members':g['members']}
  if catalogue_observation is None:catalogue_observation=observed
  else:assert observed==catalogue_observation
 own=v['owned'];expect_keys(own,'root selection identity');assert own['root']==native(J) and own['selection']=='head'
 assert own['identity']==owned_identity(ownrev)

# Literal new authority exists BEFORE any product child; the example must emit it unchanged.
alpha=inputs['selection.json'];beta=b'{"version":1,"packs":[],"disabled":[],"owned":"head","contexts":[]}'
profiles_expected=b'{"version":1,"config_revisions":[1,2,3,4,5,6],"prepared_sequences":[0,1,2],"alpha_r1":{"occurrences":3,"source_groups":2},"beta_r1":{"occurrences":2,"source_groups":1},"beta_r2":{"occurrences":3,"source_groups":1}}'
assert len(beta)==67 and len(profiles_expected)==214
newinputs={'profile-alpha-selection.json':alpha,'profile-beta-selection.json':beta,'profiles-expected.json':profiles_expected}
(base/'configured-input-authority-before-children.json').write_bytes(enc({n:{'bytes':len(b),'sha256':sha(b),'value':json.loads(b)} for n,b in newinputs.items()}))
only1=[h for h in wanted1 if h['entry']['source']=='owned-example'];only2=[h for h in wanted2 if h['entry']['source']=='owned-example']
assert len(only1)==2 and len(only2)==3
work=base/'workbench-example';cfg=base/'workbench-profiles.json';J=base/'cli-configured-owned'
C=work/'catalogue';I=work/'inputs';A=base/'alpha-r1';B=base/'beta-r1';B2=base/'beta-r2'
run('public-example',[example,work])
for name,b in {**inputs,**newinputs}.items():assert (I/name).read_bytes()==b,name
pa={'name':'alpha','catalogue_root':native(C),'owned_root':native(J),'selection':json.loads(alpha),'active_generation':None}
pb={'name':'beta','catalogue_root':None,'owned_root':native(J),'selection':json.loads(beta),'active_generation':None}
v('init','owned','init',J,'--input',I/'owned-seed.json')
v('edit-r1','owned','edit',J,'--expected-head','owned-r0','--input',I/'owned-r1.json')
w=v('configure-alpha','workbench','profiles','configure','--config',cfg,'--name','alpha','--selection',I/'profile-alpha-selection.json','--catalogue',C,'--owned',J,'--make-default')
write_check(w,'configure',True,None,1,'alpha','alpha',None);capture_config(1,'alpha',[pa],[])
s=v('unprepared-status','workbench','profiles','status','--config',cfg);status_check(s,1,None,pa,None,True,'owned-r1')
a0=refresh_check(v('refresh-alpha','workbench','profiles','refresh',A,'--config',cfg,'--expected-revision',1),1,2,'alpha','alpha',0,A)
pa['active_generation']=0;capture_config(2,'alpha',[pa],[a0])
f1=v('named-alpha','workbench','profiles','query','--config',cfg,'--profile','alpha','--request',I/'request.json','--format','full');full_check(f1,wanted1,2,'alpha','alpha',0,A,'owned-r1',True)
fd=v('default-alpha','workbench','profiles','query','--config',cfg,'--request',I/'request.json','--format','full');full_check(fd,wanted1,2,None,'alpha',0,A,'owned-r1',True)
assert f1['result']==fd['result'] and f1['binding_sha256']!=fd['binding_sha256']
c1=v('compact-alpha','workbench','profiles','query','--config',cfg,'--profile','alpha','--request',I/'request.json','--format','compact');keys=compact_check(c1,f1,wanted1)
readable_check(v('readable-alpha','workbench','profiles','query','--config',cfg,'--profile','alpha','--request',I/'request.json','--format','readable',json_output=False),f1,wanted1)
ref=base/'detail.json';ref.write_bytes(enc(keys[0]['detail_ref']))
d=v('detail-alpha','workbench','profiles','detail','--config',cfg,'--profile','alpha','--reference',ref,'--expected-binding',f1['binding_sha256'],'--format','full')
identity_check(d,2,'alpha','alpha',0,A);assert d['selection']==f1['selection'] and d['result']['hit']==wanted1[0] and d['result']['current_eligible'] is True
w=v('configure-beta','workbench','profiles','configure','--config',cfg,'--name','beta','--selection',I/'profile-beta-selection.json','--owned',J,'--expected-revision',2)
write_check(w,'configure',True,2,3,'beta','alpha',None);capture_config(3,'alpha',[pa,pb],[a0])
assert v('old-detail-binding','workbench','profiles','detail','--config',cfg,'--profile','alpha','--reference',ref,'--expected-binding',f1['binding_sha256'],'--format','full',code=1)['error']['code']=='stale_workbench_binding'
f3=v('alpha-reopened-rev3','workbench','profiles','query','--config',cfg,'--profile','alpha','--request',I/'request.json','--format','full');full_check(f3,wanted1,3,'alpha','alpha',0,A,'owned-r1',True)
before=cfg.read_bytes()
w=v('noop-beta','workbench','profiles','configure','--config',cfg,'--name','beta','--selection',I/'profile-beta-selection.json','--owned',J,'--expected-revision',3)
write_check(w,'configure',False,3,3,'beta','alpha',None);assert cfg.read_bytes()==before
b1=refresh_check(v('refresh-beta','workbench','profiles','refresh',B,'--config',cfg,'--profile','beta','--expected-revision',3),3,4,'beta','alpha',1,B)
pb['active_generation']=1;capture_config(4,'alpha',[pa,pb],[a0,b1])
fb=v('named-beta-rev4','workbench','profiles','query','--config',cfg,'--profile','beta','--request',I/'request.json','--format','full');full_check(fb,only1,4,'beta','beta',1,B,'owned-r1',False)
w=v('default-beta','workbench','profiles','default','--config',cfg,'--profile','beta','--expected-revision',4)
write_check(w,'set_default',True,4,5,'beta','beta',b1);capture_config(5,'beta',[pa,pb],[a0,b1])
f5=v('default-beta-rev5','workbench','profiles','query','--config',cfg,'--request',I/'request.json','--format','full');full_check(f5,only1,5,None,'beta',1,B,'owned-r1',False)
c5=v('compact-default-beta','workbench','profiles','query','--config',cfg,'--request',I/'request.json','--format','compact');compact_check(c5,f5,only1)
v('edit-r2','owned','edit',J,'--expected-head','owned-r1','--input',I/'owned-r2.json')
assert v('beta-source-stale','workbench','profiles','query','--config',cfg,'--request',I/'request.json','--format','full',code=1)['error']['code']=='stale_workbench'
b2=refresh_check(v('refresh-default-beta','workbench','profiles','refresh',B2,'--config',cfg,'--expected-revision',5),5,6,'beta','beta',2,B2)
pb['active_generation']=2;capture_config(6,'beta',[pa,pb],[a0,b1,b2])
f6=v('default-beta-rev6','workbench','profiles','query','--config',cfg,'--request',I/'request.json','--format','full');full_check(f6,only2,6,None,'beta',2,B2,'owned-r2',False)
history_check(v('history-all','workbench','profiles','history','--config',cfg),6,None,[a0,b1,b2])
metadata_check(v('info-rev6','workbench','profiles','info','--config',cfg),6,'beta',[pa,pb],[a0,b1,b2])
status_check(v('status-default','workbench','profiles','status','--config',cfg),6,None,pb,b2,False,'owned-r2')
assert v('alpha-source-stale','workbench','profiles','query','--config',cfg,'--profile','alpha','--request',I/'request.json','--format','full',code=1)['error']['code']=='stale_workbench'
before=cfg.read_bytes()
assert v('stale-config-cas','workbench','profiles','configure','--config',cfg,'--name','beta','--selection',I/'profile-beta-selection.json','--owned',J,'--expected-revision',5,code=1)['error']['code']=='workbench_config_conflict'
assert cfg.read_bytes()==before
w=v('remove-alpha','workbench','profiles','remove','--config',cfg,'--profile','alpha','--expected-revision',6)
write_check(w,'remove',True,6,7,'alpha','beta',None);capture_config(7,'beta',[pb],[a0,b1,b2])
history_check(v('removed-alpha-history','workbench','profiles','history','--config',cfg,'--profile','alpha'),7,'alpha',[a0])
assert v('removed-alpha-query','workbench','profiles','query','--config',cfg,'--profile','alpha','--request',I/'request.json','--format','full',code=1)['error']['code']=='unknown_workbench_profile'
metadata_check(v('final-info','workbench','profiles','info','--config',cfg),7,'beta',[pb],[a0,b1,b2])
assert len(children)==33 and sum(x['actual_exit']==0 for x in children)==28
for root in [A,B,B2]:assert (root/'binding.json').is_file() and (root/'dictionary').is_dir()
for name,b in {**inputs,**newinputs}.items():assert (I/name).read_bytes()==b
for p,pin in binding['files'].items():assert filepin(p)==pin,p
assert git('rev-parse','HEAD')==binding['head'] and git('rev-parse','HEAD^{tree}')==binding['tree'] and git('status','--porcelain','--untracked-files=all')==''
(base/'execution.json').write_bytes(enc({'status':'PASS','children':children,'source':binding['head'],'tree':binding['tree'],'method_sha256':filepin(__file__)['sha256'],'bindings_sha256':filepin(sys.argv[1])['sha256'],'scope':'33 bounded children; author-compiled public example replay plus independent input-derived configured CLI lifecycle/body/identity/detail/history/status/CAS checks; other finite mutation/curation/IO windows inherited source-reviewed author controls','base':str(base)}))
print(json.dumps({'status':'PASS','base':str(base),'children':len(children)}))
