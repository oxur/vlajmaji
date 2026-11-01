#!/usr/bin/env python3
"""UNEXECUTED S03 CDC continuation: reuse validated original five children;
execute ONLY remaining22 CLI (27 total across attempts). No example/CLI replay,
Cargo/consumer/default/Full/cleanup. Final source/binary/env bindings,
quiet custody and ROOT readiness are required before the sole execution.
Rich Form/Sense/native-patch/writer/IO controls remain author-source-reviewed
unless this explicit point-of-use actually reaches them. No full corpus claim.
"""
import copy, hashlib, json, os, subprocess, sys, tempfile, time
from pathlib import Path

def sha(b):return hashlib.sha256(b).hexdigest()
def enc(v):return json.dumps(v,ensure_ascii=False,separators=(',',':')).encode()
def pin(p):
 b=Path(p).read_bytes();return {'bytes':len(b),'sha256':sha(b)}
def keys(v,names):assert set(v)==set(names.split()),(set(v),names)
def one(rows,predicate):
 found=[r for r in rows if predicate(r)];assert len(found)==1,found;return found[0]
def occurrence_id(r):
 return b''.join(len(x.encode()).to_bytes(8,'big')+x.encode() for x in [r['capture']['namespace'],r['capture']['sha256'],r['locator']]).hex()
def native(p):return {'encoding':'unix-bytes','hex':os.fsencode(str(p)).hex(),'utf8':str(p)}

assert len(sys.argv)==2,'frozen binding JSON required; draft grants no runtime authority'
binding=json.loads(Path(sys.argv[1]).read_bytes())
assert binding['status']=='FROZEN_CDC_VERIFICATION'
src=Path(binding['source']);cli=Path(binding['cli']);example=Path(binding['example'])
env=os.environ.copy()
assert sha(env['PATH'].encode())==binding['path_sha256']
assert env['RUSTUP_AUTO_INSTALL']=='0'
for k in ['CARGO_HOME','RUSTUP_HOME']:assert env[k]==binding['toolhomes'][k]
def git(*args):return subprocess.check_output(['git','--no-optional-locks','-C',str(src),*args],env=env,text=True).strip()
def check_pins():
 assert git('rev-parse','HEAD')==binding['head'] and git('rev-parse','HEAD^{tree}')==binding['tree']
 assert git('branch','--show-current')=='codex/project01-integrated-workbench'
 assert git('status','--porcelain','--untracked-files=all')==''
 for path,wanted in binding['files'].items():assert pin(path)==wanted,path
check_pins()
assert len(binding['source_files'])==13
for p in [cli,example,Path(__file__),*[src/rel for rel in binding['source_files']]]:assert str(p) in binding['files'],p

# ALL21 original authorities are loaded/checked BEFORE creating product roots.
inputs={}
for name,record in binding['inputs'].items():
 path=Path(record['path']);b=path.read_bytes();assert pin(path)=={k:record[k] for k in ['bytes','sha256']}
 assert str(path) in binding['files'];inputs[name]=b
old={'expected.json','historical-selection.json','owned-r1.json','owned-r2.json','owned-seed.json','request.json','selection.json','upstream-dictionary.json','upstream-release.json','upstream-seal.json','upstream.json','profile-alpha-selection.json','profile-beta-selection.json','profiles-expected.json'}
new={'reference-owned-entry.json','reference-owned-assertion.json','reference-upstream.json','reference-unknown.json','reference-unresolved.json','reference-disabled-selection.json','reference-expected.json'}
assert set(inputs)==old|new
up=json.loads(inputs['upstream-dictionary.json']);r1=json.loads(inputs['owned-r1.json']);r2=json.loads(inputs['owned-r2.json'])
fixed=json.loads(inputs['reference-expected.json'])
assert fixed['kind']=='vlajmaji-workbench-reference' and fixed['owned_r1_assertions']==['owned-assertion-a','owned-assertion-b']
assert [a['definition']['Value']['text'] for a in r1['metadata']['assertions']]==['grow','develop']
assert [a['definition']['Value']['text'] for a in r2['metadata']['assertions']]==['grow','develop','flourish']
assert up['metadata']['assertions'][0]['definition']['Value']=={'text':'advance','language':{'Value':'en'},'markup':'Missing'}
for d in [up,r1,r2]:
 assert d['version']==1 and d['legacy_archive'] is None
 for family in ['contexts','frames','forms','etymologies','relations','editorial_senses','computations','annotations','extensions']:assert d['metadata'][family]==[]

def summary(r):
 return {'id':r['id'],'component_id':r['component'],'distributor_id':r['distributor'],**{k:r[k] for k in ['upstream_revision','distributor_revision','captured_at','time_precision','schema_version','normalizer','normalizer_version','captures','accounting']},'diagnostics_count':len(r['diagnostics']),'selected':r['selected'],'availability':r['availability'],'content_notices_count':len(r['content_notices']),'content_notice_licenses':[n['license'] for n in r['content_notices']],'content_notice_locators':[n['locator'] for n in r['content_notices']],'redistribution':r['redistribution'].lower(),'redistribution_evidence':r['redistribution_evidence']}
def records(d,a):
 e=one(d['metadata']['entries'],lambda e:e['id']==a['entry'])
 c=one(d['captures'],lambda c:c['identity']==a['occurrence']['capture'])
 raw=one(c['occurrences'],lambda raw:raw['locator']==a['occurrence']['locator'])
 snap=one(d['registry']['snapshots'],lambda s:s['snapshot']==c['identity'])
 attr=one(d['registry']['occurrences'],lambda row:row['occurrence']=={'snapshot':c['identity'],'locator':raw['locator']})
 return e,c,raw,snap,attr

def reference_row(d,a,compact=False):
 e,c,raw,snap,attr=records(d,a)
 row={'entry':e,'assertion':a,'occurrence_id':occurrence_id(a['occurrence']),'occurrence_attribution':attr}
 if not compact:row['raw']=raw
 row['capture_metadata']=c['metadata']
 if compact:row['snapshot_attribution']=snap
 row.update({'payload_present':True,'current_eligible':True,'release_ids':[r['id'] for r in d['metadata']['releases'] if a['id'] in r['assertions']],'owned_revision_ids':[r['id'] for r in d['metadata']['owned_revisions'] if a['id'] in r['assertions']],'tombstoned_by':[r['id'] for r in d['metadata']['owned_revisions'] if a['id'] in r['tombstones']]})
 if compact:
  row['detail_ref']={'version':1,'digest_policy':'entry-assertion-serde-json-v1','role':'assertion','source':e['source'],'entry_id':e['id'],'assertion_id':a['id'],'occurrence':a['occurrence'],'entry_sha256':sha(enc(e)),'assertion_sha256':sha(enc(a)),'native_entry_id':attr['native_entry_id'],'native_assertion_id':attr['native_assertion_id'],'contributor_id':attr['contributor'],'distributor_id':snap['distributor']}
 return row

def availability(d,compact):
 rows=[]
 for source in sorted(d['registry']['sources'],key=lambda x:x['id']):
  collections=[c for c in d['metadata']['collections'] if c['source']==source['id']]
  components=[c for c in d['metadata']['components'] if c['collection'] in [r['id'] for r in collections]]
  releases=[r for r in d['metadata']['releases'] if r['component'] in [c['id'] for c in components]]
  captures=[]
  for release in releases:
   for capture in release['captures']:
    if capture not in captures:captures.append(capture)
  loaded=[c for c in captures if any(r['identity']==c for r in d['captures'])]
  heads=[h for h in d['metadata']['owned_heads'] if h['source']==source['id']]
  assert len(heads)<=1 and releases and all(r['selected'] and r['availability']=='Available' for r in releases)
  rows.append({'source':source,'aliases':[a for a in d['registry']['aliases'] if a['source']==source['id']],'lineage':[l for l in d['registry']['lineage'] if l['source']==source['id']],'collections':collections,'components':components,'releases':[summary(r) for r in releases] if compact else releases,'selected_available_release_ids':[r['id'] for r in releases],'loaded_captures':loaded,'missing_captures':[c for c in captures if c not in loaded],'owned_head':heads[0] if heads else None,'selectable':True,'reason':None})
 return rows
def whole_hit(d,a):
 e,c,raw,snap,attr=records(d,a)
 return {'occurrence_id':occurrence_id(a['occurrence']),'entry':e,'assertion':a,'capture':c['metadata'],'raw':raw,'snapshot_attribution':snap,'occurrence_attribution':attr,'contributor':next((r for r in d['registry']['contributors'] if r['id']==attr['contributor']),None),'distributor':next((r for r in d['registry']['distributors'] if r['id']==snap['distributor']),None),'release_ids':[r['id'] for r in d['metadata']['releases'] if r['selected'] and r['availability']=='Available' and a['id'] in r['assertions']],**{k:[] for k in ['contexts','frames','forms','etymologies','relations','editorial_senses','computations','annotations','extensions']},'owned_revision_ids':[r['id'] for r in d['metadata']['owned_revisions'] if a['id'] in r['assertions']],'tombstoned_by':[r['id'] for r in d['metadata']['owned_revisions'] if a['id'] in r['tombstones']]}
up_a=one(up['metadata']['assertions'],lambda a:a['id']==fixed['upstream_assertion'])
owned_as=sorted(r2['metadata']['assertions'],key=lambda a:(one(r2['metadata']['entries'],lambda e:e['id']==a['entry'])['spelling']['Value'],occurrence_id(a['occurrence'])))
assert [a['id'] for a in owned_as]==['owned-assertion-a','owned-assertion-b','owned-assertion-c']
base=Path(binding['continuation']['base'])
assert base.is_dir() and str(base)=='/private/tmp/vlajmaji-a06-s03-cdc-345rn1y_'
retained_env=json.loads((base/'selected-env.json').read_bytes())
for k,leaf in [('HOME','home'),('XDG_DATA_HOME','xdg-data'),('XDG_CONFIG_HOME','xdg-config'),('TMPDIR','tmp')]:
 p=base/leaf;assert p.is_dir() and retained_env[k]==str(p);env[k]=str(p)
env['CARGO_NET_OFFLINE']='true';env['CARGO_TARGET_DIR']=binding['target'];env['RUSTUP_AUTO_INSTALL']='0'
assert (base/'selected-env.json').read_bytes()==enc({k:env[k] for k in ['PATH','CARGO_HOME','RUSTUP_HOME','HOME','XDG_DATA_HOME','XDG_CONFIG_HOME','TMPDIR','CARGO_TARGET_DIR','CARGO_NET_OFFLINE','RUSTUP_AUTO_INSTALL']})
# Expected complete body/native/provenance buffers are persisted BEFORE child1.
expected={'up_whole_hit':whole_hit(up,up_a),'up_full':reference_row(up,up_a),'up_compact':reference_row(up,up_a,True),'owned_full':[reference_row(r2,a) for a in owned_as],'owned_compact':[reference_row(r2,a,True) for a in owned_as],'availability_full':availability(up,False)+availability(r2,False),'availability_compact':availability(up,True)+availability(r2,True)}
assert (base/'expected-before-children.json').read_bytes()==enc(expected)
# Restore only the original successful child, never invoke its binary again.
children=json.loads(Path(binding['continuation']['prefix_commands']).read_bytes())
assert len(children)==5
expected_prefix_names=['public-example','build-mixed-r2','upstream-full','upstream-compact','upstream-readable']
for index,row in enumerate(children,1):
 assert row['n']==index and row['name']==expected_prefix_names[index-1] and row['expected_exit']==row['actual_exit']==0 and row['timeout'] is False
 assert row['cwd']==str(base) and row['argv']==binding['continuation']['expected_prefix_argv'][index-1]
 assert pin(row['stdout'])==row['stdout_pin'] and pin(row['stderr'])==row['stderr_pin']
assert (base/'commands-continuation01.json').read_bytes()==Path(binding['continuation']['prefix_commands']).read_bytes()
assert (base/'commands.json').read_bytes()==Path(binding['continuation']['child1_commands']).read_bytes()
json.loads(Path(children[0]['stdout']).read_bytes())
assert not (base/'commands-continuation02.json').exists() and not (base/'execution-continuation02.json').exists()
def run(name,argv,code=0,text=False):
 n=len(children)+1;assert n<=27
 started=time.time()
 try:
  p=subprocess.run([str(a) for a in argv],cwd=base,env=env,capture_output=True,timeout=180)
  out,err,actual,timeout=p.stdout,p.stderr,p.returncode,False
 except subprocess.TimeoutExpired as e:out,err,actual,timeout=e.stdout or b'',e.stderr or b'',None,True
 op=base/f'{n:02}-{name}.stdout';ep=base/f'{n:02}-{name}.stderr'
 with op.open('xb') as f:f.write(out)
 with ep.open('xb') as f:f.write(err)
 row={'n':n,'name':name,'argv':[str(a) for a in argv],'cwd':str(base),'expected_exit':code,'actual_exit':actual,'timeout':timeout,'seconds':time.time()-started,'stdout':str(op),'stderr':str(ep),'stdout_pin':pin(op),'stderr_pin':pin(ep)}
 children.append(row);(base/'commands-continuation02.json').write_bytes(enc(children))
 assert not timeout and actual==code,row
 if code:
  assert out==b'' and err.endswith(b'\n') and err.count(b'\n')==1
  v=json.loads(err);keys(v,'error');keys(v['error'],'code message sources');return v['error']
 assert err==b'',(name,err)
 if text:return out.decode()
 assert out.endswith(b'\n')
 if name!='public-example':assert out.count(b'\n')==1
 return json.loads(out)
def v(name,*argv,**kw):return run(name,[cli,*argv],**kw)
W=base/'workbench-example';I=W/'inputs';C=W/'catalogue';J=W/'owned';cfg=W/'workbench-profiles.json';mix=base/'mixed-r2';disabled=base/'disabled-r2'
R=W/'artifacts/reference-inputs'
def emitted_path(name):return (R if name in new else I)/name
assert mix.is_dir() and not disabled.exists()

# Output identities below are checked against direct persisted binding and original
# inputs; opaque generation/content IDs are coherence-only, not predicted values.
def direct(root,mixed=True,is_disabled=False):
 b=json.loads((root/'binding.json').read_bytes())
 keys(b,'version kind selection catalogue owned contexts resolutions selected_release_ids state_sha256 binding_sha256')
 wanted=json.loads(inputs['reference-disabled-selection.json'] if is_disabled else inputs['selection.json'] if mixed else inputs['profile-beta-selection.json'])
 assert b['selection']==wanted and b['contexts']==[] and b['resolutions']==[]
 assert b['owned']['root']==native(J if mixed else W/'configured-owned') and b['owned']['selection']=='head'
 assert b['owned']['identity']=={'source':'owned-example','journal_schema':1,'revision':'owned-r2','sequence':2,'state_sha256':sha(inputs['owned-r2.json']),'curation_state_sha256':None}
 assert b['binding_sha256']==sha(enc({k:b[k] for k in ['version','kind','selection','catalogue','owned','contexts','resolutions','selected_release_ids','state_sha256']}))
 if mixed:
  cat=b['catalogue'];assert cat['root']==native(C) and cat['local_revision']==1
  assert isinstance(cat['generation_id'],str) and isinstance(cat['content_id'],str)
 else:assert b['catalogue'] is None
 return b

def selection_check(s,b):
 keys(s,'version packs disabled upstream owned context_counts curation_counts')
 expected={'version':1,'packs':b['selection']['packs'],'disabled':b['selection']['disabled'],'upstream':None,'owned':{'mode':'head','source':'owned-example','revision':'owned-r2','sequence':2,'journal_schema':1,'state_sha256':sha(inputs['owned-r2.json']),'curation_sha256':None},'context_counts':{'declared':0,'provided':0},'curation_counts':None}
 if b['catalogue']:
  c=b['catalogue'];expected['upstream']={'local_revision':c['local_revision'],'generation_id':c['generation_id'],'content_id':c['content_id'],'state_sha256':c['recorded_state_sha256']}
 assert s==expected

def outer_check(out,target,b,fmt,requested=None,configured=False):
 names='version kind format binding_sha256 selection result'+(' configuration' if configured else '')
 keys(out,names);assert out['version']==1 and out['kind']=='vlajmaji-workbench-reference' and out['format']==fmt
 selection_check(out['selection'],b)
 if configured:
  ident={'version':1,'kind':'vlajmaji-configured-workbench','config':native(cfg),'config_revision':6,'config_sha256':sha(cfg.read_bytes()),'requested_profile':requested,'profile':'beta','prepared_sequence':2,'view_binding_sha256':b['binding_sha256']}
  ident['effective_binding_sha256']=sha(enc(ident));assert out['configuration']==ident and out['binding_sha256']==ident['effective_binding_sha256']
 else:assert out['binding_sha256']==b['binding_sha256']
 r=out['result'];keys(r,'target status operational_reason payload catalogue_matches requested_release registrations');assert r['target']==target
 return r

def registry_match():
 a=up_a;e,c,raw,snap,attr=records(up,a)
 return {'pack_id':'la-lojban','release_sha256':sha(inputs['upstream-release.json']),'source_id':e['source'],'model_release_id':json.loads(inputs['reference-upstream.json'])['Catalog']['release'],'role':'assertion','entity_id':a['id'],'occurrence':a['occurrence']['locator'],'capture':a['occurrence']['capture'],'native_entry_id':attr['native_entry_id'],'native_assertion_id':attr['native_assertion_id'],'contributing_pack_ids':['la-lojban']}
def registration(disabled=False):
 return {'pack_id':'la-lojban','release_sha256':sha(inputs['upstream-release.json']),'payload_kind':'synthetic','selected':True,'disabled':disabled,'installation_state':'installed','installed_release_sha256':sha(inputs['upstream-release.json']),'admitted':not disabled,'operational_reason':'disabled' if disabled else None,'sources':up['registry']['sources'],'requested_release':summary(up['metadata']['releases'][0])}

def resolved_check(out,target,b,fmt,which,configured=False,requested=None):
 r=outer_check(out,target,b,fmt,requested,configured)
 assert r['status']=='Resolved' and r['operational_reason'] is None
 if which=='up':assert r['catalogue_matches']==[registry_match()] and r['registrations']==[registration()] and r['requested_release']==summary(up['metadata']['releases'][0])
 else:assert r['catalogue_matches']==[] and r['registrations']==[] and r['requested_release'] is None
 p=r['payload'];assert p is not None
 common='version state_sha256 target status entity confirmed unverified availability requested_release'
 keys(p,common+(' projection_policy snapshot_sha256' if fmt=='compact' else ''))
 assert p['version']==1 and p['target']==target and p['status']=='Resolved' and p['unverified']==[] and p['state_sha256']==b['state_sha256']
 wanted=[expected['up_'+fmt]] if which=='up' else expected['owned_'+fmt][:1] if which=='assertion' else expected['owned_'+fmt]
 assert p['confirmed']==wanted
 if which=='up':assert p['entity'] is None and p['requested_release']==(summary(up['metadata']['releases'][0]) if fmt=='compact' else up['metadata']['releases'][0])
 elif fmt=='full':assert p['entity']==({'Assertion':owned_as[0]} if which=='assertion' else {'Entry':r2['metadata']['entries'][0]}) and p['requested_release'] is None
 else:
  entry=r2['metadata']['entries'][0]
  assert p['entity']=={'role':'entry','id':entry['id'],'entry_ids':[entry['id']],'assertion_ids':[a['id'] for a in owned_as],'source_ids':['owned-example'],'occurrence':None,'evidence':None,'spelling':entry['spelling']}
  assert p['requested_release'] is None
 av=expected['availability_'+fmt] if b['selection']['packs'] and not b['selection']['disabled'] else availability(r2,fmt=='compact')
 assert p['availability']==av
 if fmt=='compact':assert p['projection_policy']=='compact-dictionary-v1' and len(p['snapshot_sha256'])==64
 return r

def readable_check(text,compact,configured=False):
 labels=[('workbench_binding','binding_sha256')]
 if configured:labels.append(('workbench_configuration','configuration'))
 labels.append(('workbench_selection','selection'))
 for key in ['target','status','operational_reason','catalogue_matches','registrations','requested_release']:labels.append(('reference_'+key,None))
 lines=text.splitlines();assert len(lines)>=len(labels)
 for line,(label,key) in zip(lines,labels):
  prefix=label+': ';assert line.startswith(prefix)
  assert json.loads(line[len(prefix):])==(compact[key] if key else compact['result'][label[len('reference_'):]])
 remainder=lines[len(labels):]
 payload=compact['result']['payload']
 if payload is None:assert remainder==[]
 else:
  assert remainder[0].startswith('Reference ')
  target_json,status=remainder[0][len('Reference '):].rsplit(': ',1)
  assert json.loads(target_json)==payload['target'] and status==payload['status']
  entity_line=one(remainder,lambda line:line.startswith('entity: '))
  assert json.loads(entity_line[len('entity: '):])==payload['entity']
  assert 'confirmed: '+str(len(payload['confirmed'])) in remainder and 'unverified: 0' in remainder
  release_line=one(remainder,lambda line:line.startswith('requested_release: '))
  assert json.loads(release_line[len('requested_release: '):])==payload['requested_release']
  expected_rows=sorted(payload['confirmed'],key=lambda r:r['entry']['source'])
  # Stable source grouping preserves incoming order inside each source.
  assert [json.loads(line[len('    spelling: '):]) for line in remainder if line.startswith('    spelling: ')]==[r['entry']['spelling'] for r in expected_rows]
  assert [json.loads(line[len('    definition: '):]) for line in remainder if line.startswith('    definition: ')]==[r['assertion']['definition'] for r in expected_rows]
  assert [line for line in remainder if line.startswith('  ') and not line.startswith('    ')]==['  '+r['assertion']['id']+' @ '+r['occurrence_id'] for r in expected_rows]
  assert [json.loads(line[len('    detail: '):]) for line in remainder if line.startswith('    detail: ')]==[r['detail_ref'] for r in expected_rows]
 return remainder

# Derive every negative buffer from original authority BEFORE child1.
up_target=json.loads(inputs['reference-upstream.json']);entry_target=json.loads(inputs['reference-owned-entry.json']);assertion_target=json.loads(inputs['reference-owned-assertion.json'])
negative={'malformed':b'{','wrong-role':enc({'Local':{'Form':'owned-entry-farvi'}}),'unknown-release':enc({'Catalog':{**up_target['Catalog'],'release':'unknown-release'}}),'empty-unresolved':enc({'Unresolved':''}),'wrong-source':enc({'Catalog':{**up_target['Catalog'],'source':'owned-example'}})}
for name,b in negative.items():assert (base/f'{name}.json').read_bytes()==b
assert (base/'negative-input-authority-before-children.json').read_bytes()==enc({name:{'bytes':len(b),'sha256':sha(b),'value_hex':b.hex()} for name,b in negative.items()})
# Original child1 is restored above; validate ALL21 mapped outputs BEFORE child2.
for n,b in inputs.items():assert emitted_path(n).read_bytes()==b,n
before_cfg=cfg.read_bytes()
# Re-evaluate retained successful children2..5 OFFLINE, never invoke them again.
build=json.loads(Path(children[1]['stdout']).read_bytes())
assert build['operation']=='build_new' and build['root']==native(mix);bm=direct(mix);assert build['binding']==bm
fu=json.loads(Path(children[2]['stdout']).read_bytes());resolved_check(fu,up_target,bm,'full','up')
cu=json.loads(Path(children[3]['stdout']).read_bytes());resolved_check(cu,up_target,bm,'compact','up')
readable_check(Path(children[4]['stdout']).read_text(),cu)
fe=v('owned-entry-full','workbench','reference',mix,'--target',R/'reference-owned-entry.json','--format','full');resolved_check(fe,entry_target,bm,'full','entry')
ce=v('owned-entry-compact','workbench','reference',mix,'--target',R/'reference-owned-entry.json','--format','compact');resolved_check(ce,entry_target,bm,'compact','entry')
fa=v('owned-assertion-full','workbench','reference',mix,'--target',R/'reference-owned-assertion.json','--format','full');resolved_check(fa,assertion_target,bm,'full','assertion')
bbeta=direct(W/'configured-beta-r2',False)
fb=v('default-beta-full','workbench','profiles','reference','--config',cfg,'--target',R/'reference-owned-entry.json','--format','full');resolved_check(fb,entry_target,bbeta,'full','entry',True,None)
cb=v('default-beta-compact','workbench','profiles','reference','--config',cfg,'--target',R/'reference-owned-entry.json','--format','compact');resolved_check(cb,entry_target,bbeta,'compact','entry',True,None)
named=v('named-beta-readable','workbench','profiles','reference','--config',cfg,'--profile','beta','--target',R/'reference-owned-entry.json','--format','readable',text=True)
# Named/default effective identities differ: derive named expected envelope from
# frozen config+pre-input mode rather than compare its SHA to default response.
named_outer=copy.deepcopy(cb);named_outer['configuration']['requested_profile']='beta';ident=named_outer['configuration'];ident.pop('effective_binding_sha256');ident['effective_binding_sha256']=sha(enc(ident));named_outer['binding_sha256']=ident['effective_binding_sha256']
readable_check(named,named_outer,True)
build=v('build-disabled-r2','workbench','build',disabled,'--selection',R/'reference-disabled-selection.json','--catalogue',C,'--owned',J)
bd=direct(disabled,True,True);assert build['binding']==bd
fd=v('disabled-full','workbench','reference',disabled,'--target',R/'reference-upstream.json','--format','full')
cd=v('disabled-compact','workbench','reference',disabled,'--target',R/'reference-upstream.json','--format','compact')
for out,fmt in [(fd,'full'),(cd,'compact')]:
 r=outer_check(out,up_target,bd,fmt);assert r=={'target':up_target,'status':'CatalogueOnly','operational_reason':'disabled','payload':None,'catalogue_matches':[registry_match()],'requested_release':summary(up['metadata']['releases'][0]),'registrations':[registration(True)]}
readable_check(v('disabled-readable','workbench','reference',disabled,'--target',R/'reference-upstream.json','--format','readable',text=True),cd)
fo=v('disabled-owned-full','workbench','reference',disabled,'--target',R/'reference-owned-entry.json','--format','full');resolved_check(fo,entry_target,bd,'full','entry')
for name,fmt,status in [('unknown','full','Unknown'),('unresolved','compact','Unresolved')]:
 target=json.loads(inputs[f'reference-{name}.json']);out=v(name,'workbench','reference',mix,'--target',R/f'reference-{name}.json','--format',fmt)
 assert outer_check(out,target,bm,fmt)=={'target':target,'status':status,'operational_reason':None,'payload':None,'catalogue_matches':[],'requested_release':None,'registrations':[]}
# Reuse the already-frozen independent negative buffers; no product replay.
for name,b in negative.items():
 path=base/f'{name}.json';assert path.read_bytes()==b
 code='request_json' if name=='malformed' else 'invalid_reference'
 assert v(name,'workbench','reference',mix,'--target',path,'--format','full',code=1)['code']==code
keyfile=base/'detail-key.json';assert not keyfile.exists();keyfile.write_bytes(enc(expected['up_compact']['detail_ref']))
detail=v('reference-key-detail','workbench','detail',mix,'--reference',keyfile,'--expected-binding',cu['binding_sha256'],'--format','full')
keys(detail,'version binding_sha256 selection result');assert detail['version']==1 and detail['binding_sha256']==cu['binding_sha256'] and detail['selection']==cu['selection']
dr=detail['result'];keys(dr,'version projection_policy detail_ref status operational_reason state_sha256 snapshot_sha256 hit current_eligible')
assert dr['version']==1 and dr['projection_policy']=='complete-dictionary-hit-v1' and dr['detail_ref']==expected['up_compact']['detail_ref']
assert dr['status']=='Resolved' and dr['operational_reason'] is None and dr['current_eligible'] is True and dr['state_sha256']==bm['state_sha256'] and dr['snapshot_sha256']==cu['result']['payload']['snapshot_sha256']
assert dr['hit']==expected['up_whole_hit']
assert v('missing-target-file','workbench','reference',mix,'--target',base/'never-created-target.json','--format','full',code=1)['code']=='workbench_io'
rebuild=v('catalogue-rebuild','collections','rebuild',C,'--expected-revision','1');assert rebuild['ok'] is True and rebuild['operation']=='rebuild' and rebuild['local_revision']==2
assert v('stale-reference','workbench','reference',mix,'--target',R/'reference-upstream.json','--format','full',code=1)['code']=='stale_workbench'
assert len(children)==27 and sum(c['actual_exit']==0 for c in children)==20
assert cfg.read_bytes()==before_cfg
for n,b in inputs.items():assert emitted_path(n).read_bytes()==b,n
check_pins()
(base/'execution-continuation02.json').write_bytes(enc({'status':'PASS','children':children,'source':binding['head'],'tree':binding['tree'],'method_sha256':pin(__file__)['sha256'],'bindings_sha256':pin(sys.argv[1])['sha256'],'scope':'27 bounded children: author-compiled public example replay plus26 independent CLI; full/compact complete targeted records and availability input-derived, reported state coherent with persisted binding; rich Form/Sense/raw descriptor/writer/IO controls inherited author-source-reviewed, not independently replayed','base':str(base),'continuation_of_prefix_commands_sha256':pin(binding['continuation']['prefix_commands'])['sha256'],'new_children':22}))
print(json.dumps({'status':'PASS','base':str(base),'children':len(children),'new_children':22,'original_children':5}))
