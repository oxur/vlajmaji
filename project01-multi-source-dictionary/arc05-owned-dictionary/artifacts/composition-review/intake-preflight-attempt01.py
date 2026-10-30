import copy, hashlib, json, os, re, subprocess, sys, tomllib
from pathlib import Path
arc=Path('/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/planning/project01-multi-source-dictionary/arc05-owned-dictionary')
out=arc/'artifacts/composition-review'; ready=arc/'artifacts/review-readiness'
b=json.loads((ready/'binding.json').read_bytes()); rs=json.loads((ready/'source-readset.json').read_bytes()); source=Path(b['source_path'])
def enc(v): return json.dumps(v,ensure_ascii=False,separators=(',',':')).encode()
def sha(v): return hashlib.sha256(v).hexdigest()
def git(*args): return subprocess.check_output(['git','--no-optional-locks','-C',str(source),*args])
assert git('rev-parse','HEAD').decode().strip()==b['head']
assert git('rev-parse','HEAD^{tree}').decode().strip()==b['tree']
assert git('branch','--show-current').decode().strip()=='codex/project01-owned-dictionary'
assert not git('status','--porcelain','--untracked-files=all')
for p,h in b['file_sha256'].items(): assert sha(Path(p).read_bytes())==h,p
for f in rs['files']:
 raw=Path(f['path']).read_bytes();assert (len(raw),sha(raw))==(f['bytes'],f['sha256']),f['path']
old=tomllib.loads(git('show','HEAD^:Cargo.lock').decode()); new=tomllib.loads((source/'Cargo.lock').read_text())
key=lambda p:(p['name'],p['version'],p.get('source'))
oldrows={key(p):p for p in old['package']}; newrows={key(p):p for p in new['package']}
assert len(oldrows)==142 and len(newrows)==143
assert set(newrows)-set(oldrows)=={('vlajmaji-data-owned','0.3.0',None)} and not set(oldrows)-set(newrows)
for k,p in oldrows.items():
 expected=copy.deepcopy(p)
 if p['name']=='vlajmaji': expected['dependencies']=sorted(expected['dependencies']+['vlajmaji-data-owned'])
 assert newrows[k]==expected,k
assert newrows[('vlajmaji-data-owned','0.3.0',None)]=={'name':'vlajmaji-data-owned','version':'0.3.0','dependencies':['vlajmaji-lexicon']}
manifest=tomllib.loads(Path(b['consumer_manifest']).read_text()); lock=tomllib.loads(Path(b['consumer_lock']).read_text()); graph=json.loads(Path(b['consumer_lock_graph']).read_bytes())
assert set(manifest['dependencies'])=={'vlajmaji-data-owned','vlajmaji-lexicon'}
assert graph['packages']==lock['package'] and len(lock['package'])==24
assert graph['kind']=='source-derived-private-lock-closure-not-original-cargo-tree-output'
assert graph['manifest_sha256']==sha(Path(b['consumer_manifest']).read_bytes()) and graph['lock_sha256']==sha(Path(b['consumer_lock']).read_bytes())
names={p['name'] for p in lock['package']}; forbidden={'vlajmaji-store','vlajmaji-search','rusqlite','libsqlite3-sys','oxigraph'}
assert not names&forbidden
for p in lock['package']:
 assert all(d.split()[0] in names for d in p.get('dependencies',[]))
archive=Path(manifest['dependencies']['vlajmaji-data-owned']['path']).parents[1]
archive_pairs=[]
for f in rs['files']:
 p=Path(f['path'])
 if p.is_relative_to(source):
  ap=archive/p.relative_to(source)
  assert ap.read_bytes()==p.read_bytes(),str(ap)
  archive_pairs.append(str(p.relative_to(source)))
d=arc/'slice04-optional-owned-package/artifacts/consumer-inputs'; oracle=json.loads((d/'oracles.json').read_bytes()); originals={}
def original(row):
 p=Path(os.path.normpath(str(d/row['path'])));raw=p.read_bytes()
 assert (len(raw),sha(raw))==(row['bytes'],row['sha256']) and not raw.endswith(b'\n'),str(p)
 originals[str(p)]=raw;return raw
payload=original(oracle['positive_snapshot']);seal=original(oracle['positive_seal']); snap=json.loads(payload)
rows=[]; history=[]
for i,r in enumerate(oracle['history']):
 raw=original(r['dictionary']);cr=original(r['curation']);m=json.loads(raw);c=json.loads(cr)
 assert enc(m)==raw and enc(c)==cr and c['dictionary']==m
 assert sha(raw)==r['state_sha256'] and len(raw)==r['dictionary_bytes']
 body={'version':1,'source':'owned-example','revision':r['revision'],'base_state_sha256':sha(raw),'contexts':c['contexts'],'records':c['records'],'revisions':c['revisions']}
 ch=sha(enc(body));assert ch==r['curation_sha256'] and ch!=sha(cr) and ch!=sha(raw)
 assert len(c['contexts'])==r['contexts_length'] and len(c['records'])==r['records_length']
 assert [p['length'] for p in snap['commits'][i]['prefixes']]==r['prefix_lengths']
 assert snap['commits'][i]['state_sha256']==sha(raw)
 assert snap['curation']['prefixes'][i]=={'revision':r['revision'],'contexts_length':len(c['contexts']),'records_length':len(c['records']),'curation_sha256':ch}
 final=snap['dictionary'];reconstruction=copy.deepcopy(final)
 for p in snap['commits'][i]['prefixes']:
  if p['section']=='captures': reconstruction['captures']=reconstruction['captures'][:p['length']]
  else:
   group,name=p['section'].split('.');reconstruction[group][name]=reconstruction[group][name][:p['length']]
 reconstruction['metadata']['catalogue_revision']=i;reconstruction['metadata']['owned_heads']=[{'source':'owned-example','revision':r['revision']}]
 assert enc(reconstruction)==raw
 rows.append({'revision':r['revision'],'base_sha256':sha(raw),'curation_state_sha256':ch,'curation_file_sha256':sha(cr),'base_bytes':len(raw),'curation_file_bytes':len(cr),'prefix_lengths':r['prefix_lengths'],'selection':c['revisions'][-1]})
 history.append((m,c))
assert snap['dictionary']==history[-1][0]
assert snap['curation']['contexts']==history[-1][1]['contexts'] and snap['curation']['records']==history[-1][1]['records'] and snap['curation']['revisions']==history[-1][1]['revisions']
sealobj=json.loads(seal);assert sealobj['snapshot_sha256']==sha(payload) and sealobj['snapshot_bytes']==len(payload)
for case in oracle['cases']:
 raw=original(case['payload']);sr=original(case['seal']);ss=json.loads(sr)
 assert ss['snapshot_bytes']==len(raw) and ss['snapshot_sha256']==sha(raw)
 if case['name']=='unsupported-payload': assert raw==payload.replace(b'"version":1',b'"version":2',1)
 else: assert raw==payload.replace(b'registry.sources',b'registry.invalid_section',1)
for r in oracle['seal_or_hash_cases']:
 raw=original(r)
 if r['path']=='unsupported-seal.json': assert raw==seal.replace(b'"version":1',b'"version":2',1)
 elif r['path']=='wrong-kind.seal.json': assert raw==seal.replace(b'vlajmaji-owned-snapshot',b'vlajmaji-corpus-release')
 else:
  diff=[i for i,(a,c) in enumerate(zip(payload,raw)) if a!=c];assert len(raw)==len(payload) and len(diff)==1
context_raw=original(oracle['cli_inputs']['context.json']);view_raw=original(oracle['cli_inputs']['expected-view.json']);context=json.loads(context_raw);view=json.loads(view_raw)
assert len(originals)==21 and len(b['consumer_input_copies'])==19
for p,q in b['consumer_input_copies'].items():assert Path(q).read_bytes()==originals[p]
context_model=context['dictionary'];c=history[2][1];base=history[2][0]
assert sha(enc(context_model))==c['contexts'][0]['model_sha256']
assert sorted(x['id'] for x in context_model['registry']['sources'])==c['contexts'][0]['source_ids']
assert sorted(x['id'] for x in context_model['metadata']['releases'])==c['contexts'][0]['release_ids']
expected=copy.deepcopy(context_model)
for k,v in base['registry'].items():
 if k!='version':expected['registry'][k]+=v
expected['captures']+=base['captures']
for k,v in base['metadata'].items():
 if k=='catalogue_revision':expected['metadata'][k]=max(v,expected['metadata'][k])
 else:expected['metadata'][k]+=v
sense,link=c['records']; members=sense['kind']['sense']['members'];ids=[m.get('owned_assertion') or m['external']['assertion'] for m in members]
expected['metadata']['editorial_senses'].append({'id':sense['id'],'assertions':ids,'editor':sense['editor'],'revision':sense['revision'],'rationale':sense['rationale']})
l=link['kind']['link'];expected['metadata']['relations'].append({'id':link['id'],'origin':l['origin'],'kind':'EditorialLink','direction':l['direction'],'scope':'Editorial','target':{'Catalog':l['target']['target']},'evidence':None,'editorial_revision':link['revision']})
for record in (sense,link):expected['metadata']['extensions'].append({'owner':record['id'],'name':'vlajmaji.owned-curation.v1','value':enc({'version':1,'record':record,'contexts':c['contexts']}).decode(),'evidence':None})
assert enc(expected)==view_raw and len(view_raw)==17940
assert ids[:2]==['owned-assertion-a','owned-assertion-b']
assert history[3][1]['records']==history[4][1]['records']==history[2][1]['records']
checks={'bound_count':len(b['file_sha256']),'source_count':len(rs['files']),'originals_count':len(originals),'consumer_copies_count':len(b['consumer_input_copies']),'workspace_old_records':len(oldrows),'workspace_new_records':len(newrows),'private_packages':lock['package'],'private_graph_kind':graph['kind'],'archive_source_pairs':archive_pairs,'history':rows,'expected_view_bytes':len(view_raw),'expected_view_sha256':sha(view_raw),'context_model_sha256':sha(enc(context_model)),'expected_sense_members':ids,'source_readset':rs,'originals':[{'path':p,'bytes':len(raw),'sha256':sha(raw)} for p,raw in originals.items()],'current_source':{'head':b['head'],'tree':b['tree'],'parent':git('rev-parse','HEAD^').decode().strip(),'status':git('status','--porcelain','--untracked-files=all').decode()},'runtime_children_so_far':0,'python':sys.executable,'literal_path':os.environ['PATH']}
with (out/'intake-preflight.json').open('xb') as f:f.write(json.dumps(checks,indent=2).encode()+b'\n')
print('PASS independent pre-input checks:93 pins,32 source,21 originals,19 copies,142 old workspace records +1,24 private graph,whole expected view,5 base/curation/file hashes')
