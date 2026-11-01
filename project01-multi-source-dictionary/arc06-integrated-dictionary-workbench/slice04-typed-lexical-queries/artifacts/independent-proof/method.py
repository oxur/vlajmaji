#!/usr/bin/env python3
"""UNEXECUTED S04 independent DRAFT: 31 children, 1 author-compiled public
example plus 30 native CLI; 23 expected zero and 8 typed one. No Cargo/build,
new Rust consumer, default data, Full corpus, acquisition, benchmark or cleanup.
Author rich/private/writer/IO controls remain separately graded evidence.
Requires frozen source/binary/31-input/env bindings and ROOT runtime clearance.
"""
import copy, hashlib, json, os, re, subprocess, sys, tempfile, time
from pathlib import Path

def enc(v): return json.dumps(v,ensure_ascii=False,separators=(',',':')).encode()
def sha(b): return hashlib.sha256(b).hexdigest()
def pin(p):
 b=Path(p).read_bytes(); return {'bytes':len(b),'sha256':sha(b)}
def keys(v,n): assert set(v)==set(n.split()),(set(v),n)
def one(rows,predicate):
 found=[r for r in rows if predicate(r)]; assert len(found)==1,found; return found[0]
def optional(rows,predicate):
 found=[r for r in rows if predicate(r)]; assert len(found)<=1,found; return found[0] if found else None
def native(p): return {'encoding':'unix-bytes','hex':os.fsencode(str(p)).hex(),'utf8':str(p)}
def digest(v): assert isinstance(v,str) and re.fullmatch('[0-9a-f]{64}',v),v

def occurrence_id(r):
 return b''.join(len(x.encode()).to_bytes(8,'big')+x.encode() for x in [r['capture']['namespace'],r['capture']['sha256'],r['locator']]).hex()

assert len(sys.argv)==2,'frozen binding JSON required; draft is not runtime authority'
binding=json.loads(Path(sys.argv[1]).read_bytes())
assert binding['status']=='FROZEN_CDC_VERIFICATION'
src=Path(binding['source']); cli=Path(binding['cli']); example=Path(binding['example'])
env=os.environ.copy()
assert sha(env['PATH'].encode())==binding['path_sha256']
assert env['RUSTUP_AUTO_INSTALL']=='0'
for k in ['CARGO_HOME','RUSTUP_HOME']: assert env[k]==binding['toolhomes'][k]
def git_at(root,*args):
 return subprocess.check_output(['git','--no-optional-locks','-C',str(root),*args],env=env,text=True).strip()
def check_pins():
 assert git_at(src,'rev-parse','HEAD')==binding['head']
 assert git_at(src,'rev-parse','HEAD^{tree}')==binding['tree']
 assert git_at(src,'rev-parse','HEAD^')==binding['parent']
 assert git_at(src,'branch','--show-current')=='codex/project01-integrated-workbench'
 assert git_at(src,'status','--porcelain','--untracked-files=all')==''
 for p,wanted in binding['files'].items(): assert pin(p)==wanted,p
 for ref,expected in binding['protected_refs'].items(): assert git_at(src,'rev-parse',ref)==expected,ref
 ba=Path(binding['bauske']['path'])
 assert git_at(ba,'rev-parse','HEAD')==binding['bauske']['head']
 assert git_at(ba,'rev-parse','HEAD^{tree}')==binding['bauske']['tree']
 assert git_at(ba,'status','--porcelain','--untracked-files=all').splitlines()==binding['bauske']['status']
check_pins()
assert len(binding['source_files'])==13
for p in [cli,example,Path(__file__),*[src/r for r in binding['source_files']]]: assert str(p) in binding['files'],p

# ALL31 pre-input authorities are loaded before creating product roots/child1.
inputs={}
for name,r in binding['inputs'].items():
 p=Path(r['path']); b=p.read_bytes(); assert pin(p)=={k:r[k] for k in ['bytes','sha256']}
 assert str(p) in binding['files']; inputs[name]=b
assert len(inputs)==31
lexical_names={'lexical-upstream.json','lexical-owned-seed.json','lexical-owned-r1.json','lexical-owned-r2.json','lexical-selection.json','lexical-historical-selection.json','form-all.json','reverse-grow.json','lexical-expected.json','cursor-protocol-oracle.json'}
assert lexical_names<=set(inputs)
up=json.loads(inputs['lexical-upstream.json']); r1=json.loads(inputs['lexical-owned-r1.json']); r2=json.loads(inputs['lexical-owned-r2.json'])
fixed=json.loads(inputs['lexical-expected.json']); form_query=json.loads(inputs['form-all.json']); reverse_query=json.loads(inputs['reverse-grow.json'])
assert fixed['source_order']==['lexical-owned','lexical-upstream'] and len(fixed['form_order'])==7
assert [r['id'] for r in up['metadata']['forms']]==[r['id'] for r in fixed['forms'][3:]]
assert [r['id'] for r in r1['metadata']['forms']]==[r['id'] for r in fixed['forms'][:3]]
assert r1['metadata']['owned_revisions'][-1]['assertions']==['lexical-owned-a','lexical-owned-b']
assert r2['metadata']['owned_revisions'][-1]['assertions']==['lexical-owned-c']
assert r2['metadata']['owned_revisions'][-1]['tombstones']==['lexical-owned-a','lexical-owned-b']
for d in [up,r1,r2]:
 assert d['version']==1 and d['legacy_archive'] is None
 for a in d['metadata']['assertions']: assert a['definition']['Value']['text']==fixed['definitions'][a['id']]
 for e in d['metadata']['entries']: assert list(e)==['id','source','spelling','language','script','native_word_class','selmaho','mapped_pos']
 for f in d['metadata']['forms']: assert list(f)==['id','entry','spelling','language','script','pronunciation','kind','standing','components','evidence','computation']
 for c in d['metadata']['computations']: assert list(c)==['id','tool','tool_version','inputs','policy','profile','explanation']

# Model-derived record expectations are persisted before child1. Full Hit extras
# outside this asserted core retain inherited/source-reviewed transport scope.
def source(d,id): return one(d['registry']['sources'],lambda r:r['id']==id)
def entry(d,id): return one(d['metadata']['entries'],lambda r:r['id']==id)
def evidence(d,f):
 e=f['evidence']
 if e is None: return None
 o=e['occurrence']; cap=optional(d['captures'],lambda r:r['identity']==o['capture'])
 snap=optional(d['registry']['snapshots'],lambda r:r['snapshot']==o['capture'])
 attr=optional(d['registry']['occurrences'],lambda r:r['occurrence']=={'snapshot':o['capture'],'locator':o['locator']})
 contributor=one(d['registry']['contributors'],lambda r:r['id']==attr['contributor']) if attr and attr['contributor'] is not None else None
 distributor=one(d['registry']['distributors'],lambda r:r['id']==snap['distributor']) if snap and snap['distributor'] is not None else None
 return {'evidence':e,'status':'Loaded' if cap else 'CatalogueOnly','reason':None if cap else 'missing_capture','capture':cap['metadata'] if cap else None,'raw':one(cap['occurrences'],lambda r:r['locator']==o['locator']) if cap else None,'snapshot_attribution':snap,'occurrence_attribution':attr,'contributor':contributor,'distributor':distributor}
def eligible(d):
 ids=set(d['metadata']['owned_revisions'][-1]['assertions']) if d['registry']['sources'][0]['origin']=='OwnedDraft' else {a['id'] for a in d['metadata']['assertions']}
 return [a for a in d['metadata']['assertions'] if a['id'] in ids and a['eligibility']=='Lexical']
def association_ids(d,f):
 return [a['id'] for a in sorted(eligible(d),key=lambda a:(entry(d,a['entry'])['spelling'].get('Value',''),occurrence_id(a['occurrence']),a['id'])) if a['entry']==f['entry']]
def form_expected(d,f):
 e=entry(d,f['entry']); c=one(d['metadata']['computations'],lambda c:c['id']==f['computation']) if f['computation'] else None
 ids=association_ids(d,f)
 return {'form':f,'entry':e,'source':source(d,e['source']),'mapped_role':f['kind'] if f['kind'] in ['SourceReportedForm','AssignedRafsi'] else None,'computation':c,'evidence':evidence(d,f),'association_status':'eligible_assertions' if ids else 'no_current_eligible_assertion','associated_assertion_ids':ids}
def summary(r):
 return {'id':r['id'],'component_id':r['component'],'distributor_id':r['distributor'],**{k:r[k] for k in ['upstream_revision','distributor_revision','captured_at','time_precision','schema_version','normalizer','normalizer_version','captures','accounting']},'diagnostics_count':len(r['diagnostics']),'selected':r['selected'],'availability':r['availability'],'content_notices_count':len(r['content_notices']),'content_notice_licenses':[n['license'] for n in r['content_notices']],'content_notice_locators':[n['locator'] for n in r['content_notices']],'redistribution':r['redistribution'].lower(),'redistribution_evidence':r['redistribution_evidence']}
def availability(d,compact):
 out=[]
 for s in sorted(d['registry']['sources'],key=lambda r:r['id']):
  collections=[r for r in d['metadata']['collections'] if r['source']==s['id']]; components=[r for r in d['metadata']['components'] if r['collection'] in [c['id'] for c in collections]]
  releases=[r for r in d['metadata']['releases'] if r['component'] in [c['id'] for c in components]]
  captures=[]
  for r in releases:
   for c in r['captures']:
    if c not in captures: captures.append(c)
  loaded=[c for c in captures if any(x['identity']==c for x in d['captures'])]
  heads=[h for h in d['metadata']['owned_heads'] if h['source']==s['id']]; assert len(heads)<=1
  out.append({'source':s,'aliases':[a for a in d['registry']['aliases'] if a['source']==s['id']],'lineage':[a for a in d['registry']['lineage'] if a['source']==s['id']],'collections':collections,'components':components,'releases':[summary(r) for r in releases] if compact else releases,'selected_available_release_ids':[r['id'] for r in releases if r['selected'] and r['availability']=='Available'],'loaded_captures':loaded,'missing_captures':[c for c in captures if c not in loaded],'owned_head':heads[0] if heads else None,'selectable':True,'reason':None})
 return out
expected={}
for revision,owned in [(1,r1),(2,r2)]:
 byid={f['id']:form_expected(d,f) for d in [owned,up] for f in d['metadata']['forms']}
 assert list(byid)==fixed['form_order']
 expected[str(revision)]={'forms':byid,'availability_full':availability(owned,False)+availability(up,False),'availability_compact':availability(owned,True)+availability(up,True)}
 assert sum(len(x['associated_assertion_ids']) for x in byid.values())==fixed['r'+str(revision)]['sum_repeated_form_associations']

base=Path(tempfile.mkdtemp(prefix='vlajmaji-a06-s04-cdc-',dir='/private/tmp'))
for k,leaf in [('HOME','home'),('XDG_DATA_HOME','xdg-data'),('XDG_CONFIG_HOME','xdg-config'),('TMPDIR','tmp')]:
 p=base/leaf; p.mkdir(); env[k]=str(p)
env.update(CARGO_TARGET_DIR=binding['target'],CARGO_NET_OFFLINE='true',RUSTUP_AUTO_INSTALL='0')
selected_env={k:env[k] for k in ['PATH','CARGO_HOME','RUSTUP_HOME','HOME','XDG_DATA_HOME','XDG_CONFIG_HOME','TMPDIR','CARGO_TARGET_DIR','CARGO_NET_OFFLINE','RUSTUP_AUTO_INSTALL']}
(base/'selected-env.json').write_bytes(enc(selected_env)); (base/'expected-before-children.json').write_bytes(enc(expected))
W=base/'workbench ü example'; L=W/'lexical-workbench'; I=W/'artifacts/lexical-inputs'; hist=L/'view-historical-r1'; current=L/'view-r2'; cfg=L/'workbench-profiles.json'
requests={}
def req(name,value):
 p=base/(name+' ü.json')
 with p.open('xb') as f:f.write(enc(value))
 requests[name]={'path':str(p),**pin(p)}; return p
form_file=req('form request',form_query); reverse_file=req('reverse request',reverse_query)
zero=copy.deepcopy(form_query); zero['options']['page_size']=0; zero_file=req('zero page',zero)
first=copy.deepcopy(form_query); first['options']['page_size']=1; first_file=req('one page',first)
empty=copy.deepcopy(reverse_query); empty['query']['contains']=''; empty_file=req('empty reverse',empty)
malformed=base/'malformed ü.json'; malformed.write_bytes(b'{')
old_key={'version':1,'digest_policy':'entry-assertion-serde-json-v1','role':'assertion','source':'lexical-owned','entry_id':'lexical-owned-entry','assertion_id':'lexical-owned-a','occurrence':r1['metadata']['assertions'][0]['occurrence'],'entry_sha256':'a'*64,'assertion_sha256':'b'*64,'native_entry_id':None,'native_assertion_id':None,'contributor_id':None,'distributor_id':None}
old_key_file=req('old assertion key',old_key)
(base/'negative-buffers-before-children.json').write_bytes(enc({str(p):pin(p) for p in [empty_file,malformed,old_key_file]}))
children=[]
def run(name,argv,code=0,readable=False):
 n=len(children)+1; assert n<=31
 started=time.time()
 try:
  p=subprocess.run([str(a) for a in argv],cwd=base,env=env,capture_output=True,timeout=240)
  out,err,actual,timed=p.stdout,p.stderr,p.returncode,False
 except subprocess.TimeoutExpired as e: out,err,actual,timed=e.stdout or b'',e.stderr or b'',None,True
 op=base/f'{n:02}-{name}.stdout'; ep=base/f'{n:02}-{name}.stderr'
 with op.open('xb') as f:f.write(out)
 with ep.open('xb') as f:f.write(err)
 row={'n':n,'name':name,'argv':[str(a) for a in argv],'cwd':str(base),'expected_exit':code,'actual_exit':actual,'timeout':timed,'seconds':time.time()-started,'stdout':str(op),'stderr':str(ep),'stdout_pin':pin(op),'stderr_pin':pin(ep)}
 children.append(row); (base/'commands.json').write_bytes(enc(children))
 assert not timed and actual==code,row
 if code:
  assert out==b'' and err.endswith(b'\n') and err.count(b'\n')==1
  obj=json.loads(err); keys(obj,'error'); keys(obj['error'],'code message sources'); assert obj['error']['sources']==[]; return obj['error']
 assert err==b'',(name,err)
 if readable:return out.decode()
 assert out.endswith(b'\n')
 if name!='public-example': assert out.count(b'\n')==1
 return json.loads(out)
def v(name,*args,**kw):return run(name,[cli,*args],**kw)
def lexical(name,root,path,fmt='full',profile='direct',**kw):
 args=['workbench']
 if profile!='direct':args+=['profiles','lexical','--config',cfg]+(['--profile',profile] if profile else [])
 else:args+=['lexical',root]
 return v(name,*args,'--request',path,'--format',fmt,readable=fmt=='readable',**kw)
def detail(name,root,path,expected_binding,fmt='full',profile='direct',**kw):
 args=['workbench']
 if profile!='direct':args+=['profiles','form-detail','--config',cfg]+(['--profile',profile] if profile else [])
 else:args+=['form-detail',root]
 return v(name,*args,'--reference',path,'--expected-binding',expected_binding,'--format',fmt,readable=fmt=='readable',**kw)

def direct(root,revision):
 b=json.loads((root/'binding.json').read_bytes()); owned=r1 if revision==1 else r2
 keys(b,'version kind selection catalogue owned contexts resolutions selected_release_ids state_sha256 binding_sha256')
 assert b['version']==1 and b['kind']=='vlajmaji-workbench'
 assert b['selection']==json.loads(inputs['lexical-historical-selection.json'] if revision==1 else inputs['lexical-selection.json'])
 assert b['contexts']==[] and b['resolutions']==[]
 assert b['catalogue']['root']==native(L/'catalogue') and b['catalogue']['local_revision']==1
 assert b['catalogue']['recorded_state_sha256']==sha(inputs['lexical-upstream.json'])
 assert b['catalogue']['members']==[{'pack_id':'la-lojban','release_sha256':sha(inputs['lexical-upstream.json']),'payload_kind':'synthetic'}]
 assert b['owned']['root']==native(L/'owned') and b['owned']['selection']==b['selection']['owned']
 assert b['owned']['identity']=={'journal_schema':1,'source':'lexical-owned','revision':'lexical-owned-r'+str(revision),'sequence':revision,'state_sha256':sha(inputs['lexical-owned-r'+str(revision)+'.json']),'curation_state_sha256':None}
 assert b['selected_release_ids']==[r['id'] for d in [up,owned] for r in d['metadata']['releases'] if r['selected'] and r['availability']=='Available']
 digest(b['state_sha256']); assert b['binding_sha256']==sha(enc({k:b[k] for k in ['version','kind','selection','catalogue','owned','contexts','resolutions','selected_release_ids','state_sha256']}))
 return b

def outer(out,b,fmt,profile='direct',kind='vlajmaji-workbench-lexical'):
 keys(out,'version kind binding_sha256 selection '+('format result' if kind.endswith('lexical') else 'reference form')+(' configuration' if profile!='direct' else ''))
 assert out['version']==1 and out['kind']==kind
 if kind.endswith('lexical'): assert out['format']==fmt
 owned=b['owned']['identity']; c=b['catalogue']
 assert out['selection']=={'version':1,'packs':['la-lojban'],'disabled':[],'upstream':{'local_revision':1,'generation_id':c['generation_id'],'content_id':c['content_id'],'state_sha256':c['recorded_state_sha256']},'owned':{'mode':b['selection']['owned'],'source':'lexical-owned','revision':owned['revision'],'sequence':owned['sequence'],'journal_schema':1,'state_sha256':owned['state_sha256'],'curation_sha256':None},'context_counts':{'declared':0,'provided':0},'curation_counts':None}
 if profile=='direct':assert out['binding_sha256']==b['binding_sha256']
 else:
  # Explicit protocol-ordered9 fields; NEVER hash sorted CLI JSON maps.
  identity={'version':1,'kind':'vlajmaji-configured-workbench','config':native(cfg),'config_revision':3,'config_sha256':sha(cfg.read_bytes()),'requested_profile':profile,'profile':'lexical','prepared_sequence':1,'view_binding_sha256':b['binding_sha256']}
  identity['effective_binding_sha256']=sha(enc(identity)); assert out['configuration']==identity and out['binding_sha256']==identity['effective_binding_sha256']
 return out['binding_sha256']

def hit_check(hit,d,a,compact):
 e=entry(d,a['entry']); o=a['occurrence']; cap=one(d['captures'],lambda c:c['identity']==o['capture']); snap=one(d['registry']['snapshots'],lambda s:s['snapshot']==o['capture']); attr=one(d['registry']['occurrences'],lambda r:r['occurrence']=={'snapshot':o['capture'],'locator':o['locator']})
 assert hit['entry']==e and hit['assertion']==a and hit['occurrence_id']==occurrence_id(o)
 assert hit['capture']==cap['metadata'] and hit['snapshot_attribution']==snap and hit['occurrence_attribution']==attr
 assert hit['contributor']==(one(d['registry']['contributors'],lambda r:r['id']==attr['contributor']) if attr['contributor'] else None)
 assert hit['distributor']==(one(d['registry']['distributors'],lambda r:r['id']==snap['distributor']) if snap['distributor'] else None)
 assert hit['release_ids']==[r['id'] for r in d['metadata']['releases'] if r['selected'] and r['availability']=='Available' and a['id'] in r['assertions']]
 if compact:
  keys(hit,'occurrence_id entry assertion capture snapshot_attribution occurrence_attribution contributor distributor release_ids detail_ref')
  assert hit['detail_ref']=={'version':1,'digest_policy':'entry-assertion-serde-json-v1','role':'assertion','source':e['source'],'entry_id':e['id'],'assertion_id':a['id'],'occurrence':o,'entry_sha256':sha(enc(e)),'assertion_sha256':sha(enc(a)),'native_entry_id':attr['native_entry_id'],'native_assertion_id':attr['native_assertion_id'],'contributor_id':attr['contributor'],'distributor_id':snap['distributor']}
 else:
  assert hit['raw']==one(cap['occurrences'],lambda r:r['locator']==o['locator'])
  assert hit['owned_revision_ids']==[r['id'] for r in d['metadata']['owned_revisions'] if a['id'] in r['assertions']]
  assert hit['tombstoned_by']==[r['id'] for r in d['metadata']['owned_revisions'] if a['id'] in r['tombstones']]

def form_key(expected_row,binding_sha):
 r=expected_row
 return {'version':1,'digest_policy':'form-entry-computation-serde-json-v1','role':'form','binding_sha256':binding_sha,'source':r['source']['id'],'entry_id':r['entry']['id'],'form_id':r['form']['id'],'entry_sha256':sha(enc(r['entry'])),'form_sha256':sha(enc(r['form'])),'computation_sha256':sha(enc(r['computation'])) if r['computation'] else None}
def form_check(row,revision,compact,binding_sha):
 id=row['form_id'] if compact else row['form']['id']; wanted=expected[str(revision)]['forms'][id]; d=up if wanted['source']['id']=='lexical-upstream' else r1 if revision==1 else r2
 if compact:
  keys(row,'form_id entry_id source_id entry_spelling spelling kind mapped_role evidence computation_id association_status associated_assertions detail_ref')
  assert {k:row[k] for k in ['form_id','entry_id','source_id','entry_spelling','spelling','kind','mapped_role','evidence','computation_id','association_status']}=={'form_id':id,'entry_id':wanted['entry']['id'],'source_id':wanted['source']['id'],'entry_spelling':wanted['entry']['spelling'],'spelling':wanted['form']['spelling'],'kind':wanted['form']['kind'],'mapped_role':wanted['mapped_role'],'evidence':wanted['form']['evidence'],'computation_id':wanted['form']['computation'],'association_status':wanted['association_status']}
  assert row['detail_ref']==form_key(wanted,binding_sha)
 else:
  keys(row,'form entry source mapped_role computation evidence association_status associated_assertions')
  for k in ['form','entry','source','mapped_role','computation','evidence','association_status']:assert row[k]==wanted[k],(id,k)
 assert [h['assertion']['id'] for h in row['associated_assertions']]==wanted['associated_assertion_ids']
 for h in row['associated_assertions']:hit_check(h,d,one(d['metadata']['assertions'],lambda a:a['id']==h['assertion']['id']),compact)
 return id

def lexical_check(out,b,revision,fmt,query,ids=None,more=False,profile='direct'):
 bound=outer(out,b,fmt,profile); keys(out['result'],'operation response'); operation=query['query']['operation']; assert out['result']['operation']==operation
 r=out['result']['response']; keys(r,'policy state_sha256 query resolved_sources totals source_totals availability groups next_cursor code_notices'+(' snapshot_sha256' if fmt=='compact' else ''))
 assert r['policy']==('stored-form-kind-v1' if operation=='form' else 'literal-definition-ascii-v1')
 assert r['state_sha256']==b['state_sha256'] and r['query']==query['query'] and r['resolved_sources']==fixed['source_order']
 assert r['availability']==expected[str(revision)]['availability_'+fmt]
 assert r['code_notices']==up['metadata']['code_notices']+(r1 if revision==1 else r2)['metadata']['code_notices']
 if fmt=='compact':digest(r['snapshot_sha256'])
 if operation=='form':
  roster=fixed['form_order']; union=fixed['r'+str(revision)]['associated_assertion_union']
  assert r['totals']=={'unit':'stored_forms','records':7,'source_groups':2,'associated_assertions':union}
  assert r['source_totals']==[{'source':source(r1 if revision==1 else r2,'lexical-owned'),'records':3,'associated_assertions':2 if revision==1 else 1},{'source':source(up,'lexical-upstream'),'records':4,'associated_assertions':2}]
 else:
  roster=fixed['r'+str(revision)]['reverse_grow_order']; assert r['totals']=={'unit':'assertion_occurrences','records':len(roster),'source_groups':2 if revision==1 else 1,'associated_assertions':len(roster)}
  assert r['source_totals']==([{'source':source(r1,'lexical-owned'),'records':1,'associated_assertions':1}] if revision==1 else [])+[{'source':source(up,'lexical-upstream'),'records':2,'associated_assertions':2}]
 wanted_ids=roster if ids is None else ids; got=[]
 group_sources=[]
 for group in r['groups']:
  keys(group,'source full_count returned_count partial hits'); sid=group['source']['id']; group_sources.append(sid); d=up if sid=='lexical-upstream' else r1 if revision==1 else r2
  assert group['source']==source(d,sid)
  full_count=(4 if sid=='lexical-upstream' else 3) if operation=='form' else 2 if sid=='lexical-upstream' else 1
  expected_group_ids=[id for id in wanted_ids if ('lexical-upstream' in id)==(sid=='lexical-upstream')]
  assert group['full_count']==full_count and group['returned_count']==len(expected_group_ids) and len(group['hits'])==len(expected_group_ids) and group['partial']==(len(expected_group_ids)<full_count)
  for row in group['hits']:
   if operation=='form':got.append(form_check(row,revision,fmt=='compact',bound))
   else:
    a=one(d['metadata']['assertions'],lambda a:a['id']==row['assertion']['id']); hit_check(row,d,a,fmt=='compact'); got.append(a['id'])
 assert got==wanted_ids
 assert group_sources==[sid for sid in fixed['source_order'] if any(('lexical-upstream' in id)==(sid=='lexical-upstream') for id in wanted_ids)]
 assert (r['next_cursor'] is not None)==more
 return r

def readable_check(text,compact,detail_row=False):
 lines=text.splitlines(); expected_lines=[('workbench_binding',compact['binding_sha256'])]
 if 'configuration'in compact:expected_lines.append(('workbench_configuration',compact['configuration']))
 expected_lines.append(('workbench_selection',compact['selection']))
 if detail_row:
  expected_lines.append(('form_reference',compact['reference'])); expected_lines += [(k,compact['form'][k]) for k in ['form','entry','source','mapped_role','computation','evidence','association_status','associated_assertions']]
 else:
  p=compact['result']['response']; expected_lines += [('lexical_operation',compact['result']['operation']),('lexical_policy',p['policy']),('lexical_totals',p['totals']),('lexical_source_totals',p['source_totals']),('lexical_next_cursor',p['next_cursor'])]
  for group in p['groups']:
   expected_lines += [(k,group[k]) for k in ['source','full_count','returned_count','partial']]
   for row in group['hits']:
    if compact['result']['operation']=='form':
     expected_lines += [('  form',row['form_id']),('    kind',row['kind']),('    mapped_role',row['mapped_role']),('    entry',{'id':row['entry_id'],'spelling':row['entry_spelling']}),('    source_id',row['source_id']),('    evidence',row['evidence']),('    computation_id',row['computation_id']),('    association_status',row['association_status']),('    associated_assertions',row['associated_assertions']),('    form_detail',row['detail_ref'])]
    else:expected_lines += [('  assertion',row['assertion']['id']),('    spelling',row['entry']['spelling']),('    definition',row['assertion']['definition']),('    detail',row['detail_ref'])]
 assert len(lines)==len(expected_lines),(len(lines),len(expected_lines))
 for line,(label,value) in zip(lines,expected_lines):
  prefix=label+': '; assert line.startswith(prefix),(line,label); assert json.loads(line[len(prefix):])==value,(label,line)
 # No final-LF assumption is added to Readable output.

def detail_check(out,b,revision,reference,profile='direct'):
 bound=outer(out,b,'full',profile,'vlajmaji-workbench-form-detail'); assert out['reference']==reference
 assert reference==form_key(expected[str(revision)]['forms'][reference['form_id']],bound)
 form_check(out['form'],revision,False,bound)
 return out

def cursor_check(token,query,b,format,last_id):
 prefix='workbench-lexical-v1:'; assert token.startswith(prefix)
 raw=bytes.fromhex(token[len(prefix):]); assert token[len(prefix):]==raw.hex()
 e=json.loads(raw); keys(e,'body checksum'); keys(e['body'],'version domain format binding_sha256 query_sha256 last')
 # Exact typed body order and query order; source selector order retained.
 typed_query={'query':query['query'],'selectors':query['options']['sources'],'resolved_sources':fixed['source_order'],'alias_scope':query['options']['alias_scope']}
 last=e['body']['last']; expected_last={'operation':'form','source':'lexical-owned' if 'lexical-owned' in last_id else 'lexical-upstream','entry_spelling':'zero-associated' if last_id in fixed['form_order'][1:3] else 'farvi','spelling':'farvi-form','kind':one(fixed['forms'],lambda f:f['id']==last_id)['kind'],'form_id':last_id}
 body={'version':1,'domain':'workbench-lexical-cursor-v1','format':format,'binding_sha256':b['binding_sha256'],'query_sha256':sha(enc(typed_query)),'last':expected_last}
 assert e['body']==body and e['checksum']==sha(enc(body)); assert raw==enc({'body':body,'checksum':sha(enc(body))})

# Static finite child sequence. The first producer is NEVER replayed to repair
# missing old author transcripts; this is a distinct frozen independent proof.
produced=run('public-example',[example,W])
assert produced['profile_revisions']==[1,2,3,4,5,6]
for name,b in inputs.items():
 p=I/name if name in lexical_names else W/'artifacts/reference-inputs'/name if name.startswith('reference-') else W/'inputs'/name
 assert p.read_bytes()==b,(name,str(p))
bh=direct(hist,1); bc=direct(current,2); bp=direct(L/'configured-r2',2)
config=json.loads(cfg.read_bytes()); assert config['revision']==3 and config['default_profile']=='lexical'
assert [p['sequence'] for p in config['prepared']]==[0,1] and config['profiles'][0]['active_generation']==1
before_files={str(p):pin(p) for p in [cfg,hist/'binding.json',hist/'dictionary/dictionary.sqlite3',current/'binding.json',current/'dictionary/dictionary.sqlite3',L/'configured-r2/binding.json',L/'configured-r2/dictionary/dictionary.sqlite3']}
full1=lexical('historical-form-full',hist,form_file); lexical_check(full1,bh,1,'full',form_query)
compact1=lexical('historical-form-compact',hist,form_file,'compact'); p1=lexical_check(compact1,bh,1,'compact',form_query)
readable_check(lexical('historical-form-readable',hist,form_file,'readable'),compact1)
stem_ref=p1['groups'][0]['hits'][0]['detail_ref']; stem_file=req('historical stem key',stem_ref)
detail1=detail('historical-stem-detail',hist,stem_file,compact1['binding_sha256']); detail_check(detail1,bh,1,stem_ref)
reported_ref=one(p1['groups'][1]['hits'],lambda x:x['form_id']=='lexical-upstream-form-reported')['detail_ref']; reported_file=req('historical reported key',reported_ref)
# Readable transport counterpart: this Full record was already checked against
# pre-input Form/Entry/Source/Computation/evidence/assertion core above; leftover
# inherited Hit metadata is parity scope, not an independent new body oracle.
reported_expected={'version':1,'kind':'vlajmaji-workbench-form-detail','binding_sha256':bh['binding_sha256'],'selection':full1['selection'],'reference':reported_ref,'form':one(full1['result']['response']['groups'][1]['hits'],lambda x:x['form']['id']==reported_ref['form_id'])}
detail_check(reported_expected,bh,1,reported_ref)
readable_check(detail('historical-reported-detail-readable',hist,reported_file,bh['binding_sha256'],'readable'),reported_expected,True)
rev1=lexical('historical-reverse-full',hist,reverse_file); lexical_check(rev1,bh,1,'full',reverse_query)
revc1=lexical('historical-reverse-compact',hist,reverse_file,'compact'); lexical_check(revc1,bh,1,'compact',reverse_query)
readable_check(lexical('historical-reverse-readable',hist,reverse_file,'readable'),revc1)
nfull=lexical('named-current-form-full',current,form_file,profile='lexical'); lexical_check(nfull,bp,2,'full',form_query,profile='lexical')
ncompact=lexical('named-current-form-compact',current,form_file,'compact',profile='lexical'); np=lexical_check(ncompact,bp,2,'compact',form_query,profile='lexical')
dfull=lexical('default-current-form-full',current,form_file,profile=None); lexical_check(dfull,bp,2,'full',form_query,profile=None)
dcompact=lexical('default-current-form-compact',current,form_file,'compact',profile=None); dp=lexical_check(dcompact,bp,2,'compact',form_query,profile=None)
assert nfull['result']==dfull['result'] and nfull['binding_sha256']!=dfull['binding_sha256']
readable_check(lexical('named-current-form-readable',current,form_file,'readable',profile='lexical'),ncompact)
default_ref=dp['groups'][0]['hits'][0]['detail_ref']; default_file=req('default current stem key',default_ref)
default_detail=detail('default-current-stem-detail',current,default_file,dcompact['binding_sha256'],profile=None); detail_check(default_detail,bp,2,default_ref,profile=None)
full2=lexical('current-direct-form-full',current,form_file); lexical_check(full2,bc,2,'full',form_query)
compact2=lexical('current-direct-form-compact',current,form_file,'compact'); lexical_check(compact2,bc,2,'compact',form_query)
zero_out=lexical('historical-form-zero',hist,zero_file); lexical_check(zero_out,bh,1,'full',zero,ids=[])
first_out=lexical('historical-form-page-one',hist,first_file,'compact'); first_rows=lexical_check(first_out,bh,1,'compact',first,ids=fixed['form_order'][:1],more=True)
token=first_rows['next_cursor']; cursor_check(token,first,bh,'compact',fixed['form_order'][0])
page2=copy.deepcopy(first); page2['options'].update(page_size=3,cursor=token); page2_file=req('page continuation two',page2)
second_out=lexical('historical-form-page-three',hist,page2_file,'compact'); second_rows=lexical_check(second_out,bh,1,'compact',page2,ids=fixed['form_order'][1:4],more=True)
cursor_check(second_rows['next_cursor'],page2,bh,'compact',fixed['form_order'][3])
page3=copy.deepcopy(page2); page3['options']['cursor']=second_rows['next_cursor']; page3_file=req('page continuation final',page3)
third_out=lexical('historical-form-page-final',hist,page3_file,'compact'); lexical_check(third_out,bh,1,'compact',page3,ids=fixed['form_order'][4:])
assert lexical('compact-cursor-on-full',hist,page2_file,'full',code=1)['code']=='stale_cursor'
changed=copy.deepcopy(page2); changed['query']['value']='different-form'; changed_file=req('changed query continuation',changed)
assert lexical('changed-query-cursor',hist,changed_file,'compact',code=1)['code']=='stale_cursor'
assert detail('old-assertion-key-file',hist,old_key_file,bh['binding_sha256'],code=1)['code']=='request_json'
assert lexical('malformed-request-file',hist,malformed,code=1)['code']=='request_json'
assert lexical('empty-reverse-request',hist,empty_file,code=1)['code']=='invalid_lexical_request'
assert lexical('head-r1-stale',L/'view-r1',form_file,code=1)['code']=='stale_workbench'
assert detail('historical-key-on-current',current,stem_file,bc['binding_sha256'],code=1)['code']=='stale_form_detail'
forged=copy.deepcopy(stem_ref); forged['form_sha256']='f'*64; assert forged['form_sha256']!=stem_ref['form_sha256']; forged_file=req('forged current shape key',forged)
assert detail('forged-form-digest',hist,forged_file,bh['binding_sha256'],code=1)['code']=='stale_form_detail'
# Last two positives remain the same finite original stored-record/detail scope.
component_ref=p1['groups'][0]['hits'][1]['detail_ref']; component_file=req('zero association component key',component_ref)
component=detail('zero-association-component-detail',hist,component_file,bh['binding_sha256']); detail_check(component,bh,1,component_ref)
# Current r2 reverse definition proves only upstream a/b remain eligible.
reverse2=lexical('current-direct-reverse-full',current,reverse_file); lexical_check(reverse2,bc,2,'full',reverse_query)
assert len(children)==31 and sum(r['expected_exit']==0 for r in children)==23 and sum(r['expected_exit']==1 for r in children)==8
assert all(r['actual_exit']==r['expected_exit'] and not r['timeout'] for r in children)
for path,wanted in before_files.items(): assert pin(path)==wanted,path
for row in requests.values(): assert pin(row['path'])=={k:row[k] for k in ['bytes','sha256']},row['path']
check_pins()
(base/'execution.json').write_bytes(enc({'status':'PASS','source_head':binding['head'],'source_tree':binding['tree'],'method':pin(__file__),'bindings':pin(sys.argv[1]),'selected_env':selected_env,'children':children,'product_children':31,'zero_exits':23,'typed_one_exits':8,'post_pins':'PASS','request_files':requests,'input_authorities':31,'emitted_input_parity':'31 WHOLE buffers PASS','protected_refs':'PASS','scope':'one author-compiled public Rust example plus 30 independently observed native CLI children; no new Rust consumer or Full corpus; rich private/writer/IO controls separately author/source graded'}))
print('PASS',str(base),'31 children (23 zero, 8 typed one); post-pins PASS')
