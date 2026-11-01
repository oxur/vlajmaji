#!/usr/bin/env python3
"""PROSPECTIVE S06 DRAFT23. No runtime authority before final binding/ROOT readiness."""
import copy,hashlib,json,os,re,subprocess,sys,time
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

def summary(r):
 return {'id':r['id'],'component_id':r['component'],'distributor_id':r['distributor'],**{k:r[k] for k in ['upstream_revision','distributor_revision','captured_at','time_precision','schema_version','normalizer','normalizer_version','captures','accounting']},'diagnostics_count':len(r['diagnostics']),'selected':r['selected'],'availability':r['availability'],'content_notices_count':len(r['content_notices']),'content_notice_licenses':[n['license'] for n in r['content_notices']],'content_notice_locators':[n['locator'] for n in r['content_notices']],'redistribution':r['redistribution'].lower(),'redistribution_evidence':r['redistribution_evidence']}

def git_at(root,*args):
 return subprocess.check_output(['git','--no-optional-locks','-C',str(root),*args],env=env,text=True).strip()

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

def source(d,id):return one(d['registry']['sources'],lambda r:r['id']==id)

def entry(d,id):return one(d['metadata']['entries'],lambda r:r['id']==id)

def assertion(d,id):return one(d['metadata']['assertions'],lambda r:r['id']==id)

def merge_models(pack,owned):
 d=copy.deepcopy(pack)
 for section in ['registry','metadata']:
  for name,rows in owned[section].items():
   if not isinstance(rows,list):continue
   if name=='owned_heads':d[section][name]=copy.deepcopy(rows);continue
   for row in rows:
    if isinstance(row,dict):
     field='id' if 'id' in row else 'snapshot' if 'snapshot' in row else 'occurrence' if 'occurrence' in row else None
     same=[r for r in d[section][name] if field and r[field]==row[field]]
     if same:assert len(same)==1 and same[0]==row,(section,name,field)
     elif row not in d[section][name]:d[section][name].append(copy.deepcopy(row))
    elif row not in d[section][name]:d[section][name].append(copy.deepcopy(row))
 for cap in owned['captures']:
  previous=optional(d['captures'],lambda c:c['identity']==cap['identity'])
  if previous:assert previous==cap
  else:d['captures'].append(copy.deepcopy(cap))
 d['metadata']['catalogue_revision']=owned['metadata']['catalogue_revision']
 return d

# Source/schema constants read from the active S06 packet. Final SOURCE is pinned
# only after author QUIET. This file is not executable authority by itself.
PREFIX='workbench-lexical-v1:'
B='workbench-computed-assertions-v1';O='computed-assertion-order-v1'
PH='p01-default-canonical-unspecified-v1';CL='dictionary-analysis-class-v1'
CO='latkerlo-jvotci-2.10.2601-ordered-standard-v1'
POLICIES={
 'spelling_distance':[B,O,'scalar-levenshtein-v1'],
 'prosody':[B,O,PH,CL],'nuclei':[B,O,PH,CL],
 'contrast':[B,O,PH,CL,'aligned-single-vowel-contrast-v1'],
 'components':[B,O,CO],'compose':[B,O,PH,CL,'compose-links-target-hops-v1'],
 'proposal_exact':['caller-proposals-v1','proposal-namespace-separated-v1'],
 'generate':[CO]}
PLAN=[
 ('default-public-example',0),('guide01-distance-full',0),('guide02-prosody-compact',0),
 ('guide03-nuclei-readable',0),('guide04-contrast-full',0),('guide05-components-compact',0),
 ('guide06-compose-compact',0),('guide07-proposal-readable',0),('guide08-standard-full',0),
 ('guide09-allow-y-full',0),('guide10-force-y-readable',0),('guide11-named-prosody-r2',0),
 ('guide12-default-prosody-r2',0),('current-compose-compact',0),('source-page1',0),
 ('source-page3',0),('source-rest',0),('proposal-page1',0),('proposal-page2',0),
 ('proposal-rest',0),('wrong-format-cursor',1),('absent-source-position',1),
 ('invalid-nonmatching-proposal',1)]
assert len(PLAN)==23

def ordered_query(request,resolved):
 # Exact typed QueryBindingPayload order, page_size/cursor deliberately excluded.
 return {'query':request['query'],'selectors':request['options']['sources'],
         'resolved_sources':resolved,'alias_scope':request['options']['alias_scope']}

def decode_token(token):
 assert token.startswith(PREFIX)
 raw=bytes.fromhex(token[len(PREFIX):])
 ev=json.loads(raw);keys(ev,'body checksum')
 assert enc(ev)==raw and sha(enc(ev['body']))==ev['checksum']
 return ev

def source_position(d,a,distance):
 e=entry(d,a['entry'])
 return {'operation':'computed_source','family':'spelling_distance','distance':distance,
         'spelling':e['spelling']['Value'],'source':e['source'],
         'occurrence_id':occurrence_id(a['occurrence']),'assertion_id':a['id']}

def token_check(token,binding_sha,request,resolved,fmt,last):
 body={'version':1,'domain':'workbench-lexical-cursor-v1','format':fmt,
       'binding_sha256':binding_sha,'query_sha256':sha(enc(ordered_query(request,resolved))),
       'last':last}
 assert token==PREFIX+enc({'body':body,'checksum':sha(enc(body))}).hex()
 return body

def source_total(d,ids):
 out=[]
 for sid in sorted({entry(d,assertion(d,a)['entry'])['source'] for a in ids}):
  aa=[assertion(d,a) for a in ids if entry(d,assertion(d,a)['entry'])['source']==sid]
  out.append({'source':source(d,sid),'records':len(aa),
              'distinct_spellings':len({entry(d,a['entry'])['spelling']['Value'] for a in aa})})
 return out

def expected_case(family,revision):
 e=copy.deepcopy(fixed['r1'][family])
 ids=[r['assertion_id'] for r in e['rows']] if family=='spelling_distance' else e['ids']
 if revision==2:
  ids=[a for a in ids if a!='comp-owned-snime']
  if family=='compose':ids=[]
  e['totals'].update(candidate_assertions=7,candidate_distinct_spellings=5,
                     records=len(ids),distinct_spellings=len({entry(models[2],assertion(models[2],a)['entry'])['spelling']['Value'] for a in ids}),
                     source_groups=len({entry(models[2],assertion(models[2],a)['entry'])['source'] for a in ids}))
 return e,ids

def full_hit_check(hit,d,a,compact):
 hit_check(hit,d,a,compact)
 if not compact:
  # Complete bounded input-derived bodies; no new fields/fixture population.
  assert hit['contexts']==[r for r in d['metadata']['contexts'] if r['assertion']==a['id']]
  for field in ['frames','forms','etymologies','relations','editorial_senses','computations','annotations','extensions']:
   assert hit[field]==[],(field,a['id'])
  assert hit['tombstoned_by']==[r['id'] for r in d['metadata']['owned_revisions'] if a['id'] in r['tombstones']]

def analysis_check(value,d,a):
 keys(value,'mapped_word_kind nuclei syllable_count primary_nucleus stress_status diagnostics class_evidence')
 e=entry(d,a['entry']);sp=e['spelling']['Value'];core=fixed['analysis_core'][sp]
 assert {k:value[k] for k in core}==core
 native_class=e['native_word_class'];capture=None;fmt=None
 if native_class=={'Value':'gismu'}:rule=CL+'/native/gismu->brivla'
 else:
  assert native_class=='Missing'
  cap=one(d['captures'],lambda r:r['identity']==a['occurrence']['capture'])
  fmt=cap['metadata']['format_label'];assert fmt=='llg-gismu-fixed-columns'
  capture=cap['identity'];rule=CL+'/capture/'+fmt+'/gismu->brivla'
 assert value['class_evidence']=={'native_word_class':native_class,
   'mapped_word_kind':core['mapped_word_kind'],'rule':rule,'capture':capture,'format_label':fmt}

def selection_for(b):
 c=b['catalogue'];owned=b['owned']['identity']
 return {'version':1,'packs':['la-lojban'],'disabled':[],
   'upstream':{'local_revision':1,'generation_id':c['generation_id'],'content_id':c['content_id'],
               'state_sha256':c['recorded_state_sha256']},
   'owned':{'mode':b['selection']['owned'],'source':'comp-owned','revision':owned['revision'],
            'sequence':owned['sequence'],'journal_schema':1,'state_sha256':owned['state_sha256'],
            'curation_sha256':None},
   'context_counts':{'declared':0,'provided':0},'curation_counts':None}

def direct(root,revision,head=False):
 b=json.loads((root/'binding.json').read_bytes())
 keys(b,'version kind selection catalogue owned contexts resolutions selected_release_ids state_sha256 binding_sha256')
 assert b['version']==1 and b['kind']=='vlajmaji-workbench'
 assert b['selection']==json.loads(inputs['artifacts/computed-inputs/selection-head.json' if head else 'artifacts/computed-inputs/selection-historical-r1.json'])
 assert b['contexts']==[] and b['resolutions']==[]
 c=b['catalogue'];assert c['root']==native(CW/'catalogue') and c['local_revision']==1
 packsha=sha(enc(pack));assert c['recorded_state_sha256']==packsha
 assert c['members']==[{'pack_id':'la-lojban','release_sha256':packsha,'payload_kind':'synthetic'}]
 assert b['owned']['root']==native(CW/'owned-journal') and b['owned']['selection']==b['selection']['owned']
 assert b['owned']['identity']=={'journal_schema':1,'source':'comp-owned','revision':'comp-owned-r'+str(revision),
   'sequence':revision,'state_sha256':sha(enc(own[revision])),'curation_state_sha256':None}
 assert b['selected_release_ids']==[r['id']for r in models[revision]['metadata']['releases'] if r['selected']and r['availability']=='Available']
 digest(b['state_sha256'])
 assert b['binding_sha256']==sha(enc({k:b[k]for k in ['version','kind','selection','catalogue','owned','contexts','resolutions','selected_release_ids','state_sha256']}))
 return b

def configuration_identity(b,requested,revision):
 wire=json.loads(cfg.read_bytes());assert wire['revision']==4
 assert [p['sequence']for p in wire['prepared']]==[0,1]
 cfgsha=sha(cfg.read_bytes())
 if revision==2:
  wire['revision']=2;wire['default_profile']=None
  wire['profiles'][0]['active_generation']=0;wire['prepared']=wire['prepared'][:1]
  cfgsha=sha(enc(wire)+b'\n')
 ident={'version':1,'kind':'vlajmaji-configured-workbench','config':native(cfg),
   'config_revision':revision,'config_sha256':cfgsha,'requested_profile':requested,
   'profile':'computed','prepared_sequence':0 if revision==2 else 1,
   'view_binding_sha256':b['binding_sha256']}
 ident['effective_binding_sha256']=sha(enc(ident));return ident

def outer(out,b,fmt,requested='direct',config_revision=4):
 keys(out,'version kind binding_sha256 selection format result'+(' configuration'if requested!='direct'else''))
 assert out['version']==1 and out['kind']=='vlajmaji-workbench-lexical' and out['format']==fmt
 assert out['selection']==selection_for(b)
 expected=b['binding_sha256']
 if requested!='direct':
  ci=configuration_identity(b,requested,config_revision);assert out['configuration']==ci
  expected=ci['effective_binding_sha256']
 assert out['binding_sha256']==expected
 return expected

def source_check(out,b,revision,request,compact=False,indices=None,requested='direct',config_revision=4):
 digest_outer=outer(out,b,'compact'if compact else'full',requested,config_revision)
 family=request['query']['operation'];keys(out['result'],'operation response')
 assert out['result']['operation']==family
 r=out['result']['response'];d=models[revision];expect,ids=expected_case(family,revision)
 keys(r,'policy_ids state_sha256 query resolved_sources totals source_totals availability hits exclusions next_cursor code_notices'+(' snapshot_sha256'if compact else''))
 assert r['policy_ids']==POLICIES[family] and r['query']==request['query']
 assert r['resolved_sources']==['comp-owned','comp-upstream']
 assert r['state_sha256']==b['state_sha256'] and r['totals']==expect['totals']
 if 'exclusions' in expect:assert r['exclusions']==expect['exclusions']
 else:
  # Fixed Components authority does not freeze a per-engine rejection breakdown.
  # Preserve author/source grade; only schema + first-exclusion conservation is
  # independently derived here, never expected map values copied from a result.
  assert family=='components' and isinstance(r['exclusions'],dict)
  assert all(isinstance(k,str)and isinstance(v,int)and v>=0 for k,v in r['exclusions'].items())
  assert sum(r['exclusions'].values())==r['totals']['candidate_assertions']-r['totals']['records']
 assert r['source_totals']==source_total(d,ids)
 assert r['availability']==availability(d,compact)
 assert r['code_notices']==d['metadata']['code_notices']
 if compact:digest(r['snapshot_sha256'])
 returned=ids if indices is None else[ids[i]for i in indices]
 assert [x['hit']['assertion']['id']for x in r['hits']]==returned
 distances={x['assertion_id']:x['distance']for x in fixed['r1']['spelling_distance']['rows']}
 for v in r['hits']:
  keys(v,'hit spelling_distance analysis distance_positions computed_components')
  a=assertion(d,v['hit']['assertion']['id']);full_hit_check(v['hit'],d,a,compact)
  assert v['spelling_distance']==(distances[a['id']]if family=='spelling_distance'else None)
  if family in ['spelling_distance','components']:assert v['analysis'] is None
  else:analysis_check(v['analysis'],d,a)
  assert v['distance_positions']==(fixed['r1']['contrast']['distance_positions']if family=='contrast'else[])
  assert v['computed_components']==(fixed['r1']['components']['decomposition']if family=='components'else None)
 if request['options']['page_size'] in [None,0]:assert r['next_cursor'] is None
 elif indices is not None and indices and indices[-1]<len(ids)-1:assert isinstance(r['next_cursor'],str)
 if r['next_cursor'] is not None:
  a=assertion(d,returned[-1]);token_check(r['next_cursor'],digest_outer,request,r['resolved_sources'],
    'compact'if compact else'full',source_position(d,a,distances[a['id']]))
 return r

def proposal_check(out,b,request,compact=False,indices=None):
 digest_outer=outer(out,b,'compact'if compact else'full')
 assert out['result']['operation']=='proposal_exact'
 r=out['result']['response'];keys(r,'policy_ids query totals proposals next_cursor code_notices')
 assert r['policy_ids']==POLICIES['proposal_exact'] and r['query']==request['query']
 assert r['totals']==fixed['proposal']['totals'] and r['code_notices']==[]
 rr=fixed['proposal']['rows'];assert r['proposals']==(rr if indices is None else[rr[i]for i in indices])
 if request['options']['page_size'] in [None,0]:assert r['next_cursor'] is None
 elif indices is not None and indices and indices[-1]<len(rr)-1:assert isinstance(r['next_cursor'],str)
 if r['next_cursor']is not None:
  last=r['proposals'][-1]
  token_check(r['next_cursor'],digest_outer,request,[],'compact'if compact else'full',
    {'operation':'proposal_exact','proposal_id':last['id'],'input_ordinal':last['input_ordinal']})
 return r

def generate_check(out,b,request,compact=False):
 outer(out,b,'compact'if compact else'full');assert out['result']['operation']=='generate'
 r=out['result']['response'];keys(r,'policy_ids query value code_notices')
 assert r['policy_ids']==POLICIES['generate']and r['query']==request['query']and r['code_notices']==[]
 expect=one(fixed['generate'],lambda v:v['setting']==request['query']['setting'])
 assert r['value']==expect;return r

def readable_check(text,out,inner=False):
 rows=[]
 if not inner:
  rows=[('workbench_binding',out['binding_sha256'])]
  if 'configuration'in out:rows.append(('workbench_configuration',out['configuration']))
  rows.append(('workbench_selection',out['selection']))
 family=out['result']['operation'];r=out['result']['response']
 rows.extend([('lexical_operation',family),('lexical_policy_ids',r['policy_ids'])])
 if family=='generate':rows.append(('computed_value',r['value']))
 elif family=='proposal_exact':
  rows.extend([('lexical_totals',r['totals']),('lexical_next_cursor',r['next_cursor'])])
  rows.extend(('proposal',x)for x in r['proposals'])
 else:
  rows.extend((label,r[key])for label,key in [('lexical_totals','totals'),('lexical_source_totals','source_totals'),('lexical_exclusions','exclusions'),('lexical_next_cursor','next_cursor')])
  rows.extend(('computed_hit',x)for x in r['hits'])
 actual=[]
 for line in text.splitlines():
  label,value=line.split(': ',1);actual.append((label,json.loads(value)))
 assert actual==rows

def producer_check(out):
 c=out['computed'];assert c['profile_revision']==4 and c['workspace']=='computed-workbench'
 assert c['inputs']['directory']=='artifacts/computed-inputs'
 for family,obs in c['historical_r1'].items():
  q=source_request if family=='spelling_distance'else make_request(fixed['r1'][family]['query'])
  if family=='components':
   assert obs=={'full':{'status':'feature_disabled','feature':'morphology'}}
  else:
   assert obs['full']['status']=='ok'and obs['compact']['status']=='ok'
   source_check(obs['full']['response'],b_hist,1,q)
   source_check(obs['compact']['response'],b_hist,1,q,True)
   readable_check(obs['compact']['readable'],obs['compact']['response'],True)
 assert set(c['historical_r1'])==set(['spelling_distance','prosody','nuclei','contrast','components','compose'])
 assert c['generation_settings']==[{'status':'feature_disabled','feature':'morphology'}]
 p=c['proposal_exact'];assert p['full']['status']==p['compact']['status']=='ok'
 proposal_check(p['full']['response'],b_hist,proposal_request)
 proposal_check(p['compact']['response'],b_hist,proposal_request,True)
 readable_check(p['compact']['readable'],p['compact']['response'],True)
 source_check(c['configured_r1'],b_head1,1,source_request,requested='computed',config_revision=2)
 source_check(c['configured_r2'],b_head2,2,source_request,requested='computed')
 source_check(c['default_r2'],b_head2,2,source_request,requested=None)
 assert c['configured_r2']['result']==c['default_r2']['result']
 assert c['r2_comparison']=={'result_body_equal':True,
  'named_effective_binding_sha256':c['configured_r2']['binding_sha256'],
  'default_effective_binding_sha256':c['default_r2']['binding_sha256']}
 # The accidental author OFF invocation remains separate evidence. No replay
 # is used to amend its old observer or feature provenance.

def make_request(query):
 return {'version':1,'query':copy.deepcopy(query),'options':copy.deepcopy(source_request['options'])}

def preflight():
 assert sha(os.environ['PATH'].encode())==binding['path_sha256']
 assert os.environ['RUSTUP_AUTO_INSTALL']=='0'and os.environ['CARGO_NET_OFFLINE']=='true'
 assert os.environ['CARGO_TARGET_DIR']==binding['target']
 for k in ['CARGO_HOME','RUSTUP_HOME']:assert os.environ[k]==binding['toolhomes'][k]
 assert git_at(src,'rev-parse','HEAD')==binding['head']
 assert git_at(src,'rev-parse','HEAD^{tree}')==binding['tree']
 assert git_at(src,'rev-parse','HEAD^')==binding['parent']
 assert git_at(src,'status','--porcelain','--untracked-files=all')==''
 for p,w in binding['files'].items():assert pin(p)==w,p
 for ref,w in binding['protected_refs'].items():assert git_at(src,'rev-parse',ref)==w
 ba=binding['bauske'];assert git_at(ba['path'],'rev-parse','HEAD')==ba['head']
 assert git_at(ba['path'],'rev-parse','HEAD^{tree}')==ba['tree']
 assert git_at(ba['path'],'status','--porcelain','--untracked-files=all').splitlines()==ba['status']

def file_new(path,value):
 with path.open('xb')as f:f.write(value)
 return path

def run(name,argv,code=0,readable=False,documented=None):
 n=len(children)+1;assert PLAN[n-1]==(name,code)
 op=base/f'{n:02}-{name}.stdout';ep=base/f'{n:02}-{name}.stderr'
 assert not op.exists()and not ep.exists()
 start=time.time()
 try:
  p=subprocess.run([str(a)for a in argv],cwd=base,env=env,capture_output=True,timeout=300)
  out,err,actual,timed=p.stdout,p.stderr,p.returncode,False
 except subprocess.TimeoutExpired as e:out,err,actual,timed=e.stdout or b'',e.stderr or b'',None,True
 file_new(op,out);file_new(ep,err)
 row={'n':n,'name':name,'argv':[str(a)for a in argv],'native_argv':[native(a)for a in argv],
  'cwd':str(base),'expected_exit':code,'actual_exit':actual,'timeout':timed,
  'seconds':time.time()-start,'stdout':str(op),'stderr':str(ep),'stdout_pin':pin(op),'stderr_pin':pin(ep)}
 if documented is not None:
  file_new(documented,out);row['documented_output']=str(documented);row['documented_output_pin']=pin(documented)
 children.append(row);(base/'commands.json').write_bytes(enc(children))
 assert not timed and actual==code,row
 if code:
  assert out==b''and err.endswith(b'\n')and err.count(b'\n')==1
  e=json.loads(err);keys(e,'error');keys(e['error'],'code message sources')
  assert isinstance(e['error']['code'],str)and isinstance(e['error']['message'],str)and e['error']['sources']==[]
  return e['error']
 assert err==b'',(name,err)
 return out.decode()if readable else json.loads(out)

def cli_call(name,path,fmt='full',profile='direct',code=0,readable=False,documented=None):
 argv=[cli,'workbench']
 if profile=='direct':argv+=['lexical',H]
 else:
  argv+=['profiles','lexical','--config',CFG]
  if profile is not None:argv+=['--profile',profile]
 argv+=['--request',path,'--format',fmt]
 return run(name,argv,code,readable,documented)

if __name__=='__main__':
 assert len(sys.argv)==2,'exact ROOT-reviewed frozen bindings required'
 binding=json.loads(Path(sys.argv[1]).read_bytes());assert binding['status']=='FROZEN_CDC_VERIFICATION'
 src=Path(binding['source']);cli=Path(binding['cli']);example=Path(binding['example']);env=os.environ.copy()
 preflight();assert len(binding['source_files'])==15
 inputs={}
 for name,row in binding['inputs'].items():
  assert pin(row['path'])=={k:row[k]for k in ['bytes','sha256']}
  inputs[name]=Path(row['path']).read_bytes()
 assert len(inputs)==54
 pack=json.loads(inputs['artifacts/computed-inputs/mixed-pack-r1.json'])
 own={r:json.loads(inputs[f'artifacts/computed-inputs/owned-r{r}.json'])for r in [0,1,2]}
 models={r:merge_models(pack,own[r])for r in [1,2]}
 fixed=json.loads(inputs['artifacts/computed-inputs/computed-expected.json'])
 source_request=json.loads(inputs['artifacts/computed-inputs/computed-source-request.json'])
 proposal_request=json.loads(inputs['artifacts/computed-inputs/computed-proposal-request.json'])
 generate_request=json.loads(inputs['artifacts/computed-inputs/computed-generate-request.json'])
 protocol=json.loads(inputs['artifacts/computed-inputs/computed-cursor-protocol.json'])
 for family in ['computed','proposal']:
  q=protocol[family]
  assert sha(q['query_binding_utf8'].encode())==q['query_sha256']
  assert sha(q['body_utf8'].encode())==q['checksum']
  assert q['token']==PREFIX+q['envelope_utf8'].encode().hex()
  assert json.loads(q['envelope_utf8'])=={'body':json.loads(q['body_utf8']),'checksum':q['checksum']}
 planning_before={p:pin(p)for p in binding['four_frozen_planning_bodies']}
 base=Path(binding['base']);assert not base.exists();base.mkdir()
 selected_env={k:str(base/leaf)for k,leaf in [('HOME','home'),('XDG_DATA_HOME','xdg-data'),('XDG_CONFIG_HOME','xdg-config'),('TMPDIR','tmp')]}
 for p in selected_env.values():Path(p).mkdir()
 env.update(selected_env)
 file_new(base/'selected-env.json',enc({**selected_env,'PATH_sha256':binding['path_sha256'],
  'CARGO_HOME':env['CARGO_HOME'],'RUSTUP_HOME':env['RUSTUP_HOME'],'CARGO_TARGET_DIR':env['CARGO_TARGET_DIR'],
  'RUSTUP_AUTO_INSTALL':env['RUSTUP_AUTO_INSTALL'],'CARGO_NET_OFFLINE':env['CARGO_NET_OFFLINE']}))
 file_new(base/'planned-children.json',enc(PLAN))
 file_new(base/'expected-before-children.json',enc({'models':models,'fixed':fixed,'planning_pins':planning_before}))
 W=base/'workbench-example';CW=W/'computed-workbench';cfg=CW/'profiles.json'
 H='workbench-example/computed-workbench/view-historical-r1'
 CFG='workbench-example/computed-workbench/profiles.json';I='workbench-example/artifacts/computed-inputs'
 children=[]
 product=run('default-public-example',[example,W])
 for name,value in inputs.items():assert (W/name).read_bytes()==value,name
 b_hist=direct(CW/'view-historical-r1',1)
 b_head1=direct(CW/'configured-r1',1,True)
 b_head2=direct(CW/'configured-r2',2,True)
 producer_check(product)
 # Capture all task-only durable files after producer, before query-only stage.
 # Do not read any retained Full/default/cache leaf outside this new W.
 durable_before={str(p):pin(p)for p in sorted(W.rglob('*'))if p.is_file()}
 file_new(base/'durable-before-cli.json',enc(durable_before))
 requests={}
 for name,family in [('computed-prosody','prosody'),('computed-nuclei','nuclei'),
   ('computed-contrast','contrast'),('computed-components','components'),('computed-compose','compose')]:
  value=make_request(fixed['r1'][family]['query']);requests[family]=value
  file_new(W/(name+'.json'),enc(value)+b'\n')
 for setting in ['allow_y','force_y']:
  value=copy.deepcopy(generate_request);value['query']['setting']=setting;requests[setting]=value
  file_new(W/('computed-generate-'+setting.replace('_','-')+'.json'),enc(value)+b'\n')
 def gp(n):return W/f'computed-guide-{n:02}.{"txt"if n in [3,7,10]else"json"}'
 d=cli_call('guide01-distance-full',I+'/computed-source-request.json',documented=gp(1));source_check(d,b_hist,1,source_request)
 p=cli_call('guide02-prosody-compact','workbench-example/computed-prosody.json','compact',documented=gp(2));source_check(p,b_hist,1,requests['prosody'],True)
 nr=cli_call('guide03-nuclei-readable','workbench-example/computed-nuclei.json','readable',readable=True,documented=gp(3))
 # Readable source header/hits are checked against the independent input-derived
 # expected schema plus producer's already-input-checked Compact response.
 readable_check(nr,product['computed']['historical_r1']['nuclei']['compact']['response'])
 c=cli_call('guide04-contrast-full','workbench-example/computed-contrast.json',documented=gp(4));source_check(c,b_hist,1,requests['contrast'])
 c=cli_call('guide05-components-compact','workbench-example/computed-components.json','compact',documented=gp(5));source_check(c,b_hist,1,requests['components'],True)
 c=cli_call('guide06-compose-compact','workbench-example/computed-compose.json','compact',documented=gp(6));source_check(c,b_hist,1,requests['compose'],True)
 pr=cli_call('guide07-proposal-readable',I+'/computed-proposal-request.json','readable',readable=True,documented=gp(7))
 readable_check(pr,product['computed']['proposal_exact']['compact']['response'])
 c=cli_call('guide08-standard-full',I+'/computed-generate-request.json',documented=gp(8));generate_check(c,b_hist,generate_request)
 c=cli_call('guide09-allow-y-full','workbench-example/computed-generate-allow-y.json',documented=gp(9));generate_check(c,b_hist,requests['allow_y'])
 force=cli_call('guide10-force-y-readable','workbench-example/computed-generate-force-y.json','readable',readable=True,documented=gp(10))
 # No extra product call: build the truthful generated readable fields from fixed
 # seven-field value and actual typed outer binding/selection.
 force_expected={'binding_sha256':b_hist['binding_sha256'],'selection':selection_for(b_hist),
   'result':{'operation':'generate','response':{'policy_ids':POLICIES['generate'],
   'value':one(fixed['generate'],lambda x:x['setting']=='force_y')}}}
 readable_check(force,force_expected)
 named=cli_call('guide11-named-prosody-r2','workbench-example/computed-prosody.json',profile='computed',documented=gp(11));source_check(named,b_head2,2,requests['prosody'],requested='computed')
 default=cli_call('guide12-default-prosody-r2','workbench-example/computed-prosody.json',profile=None,documented=gp(12));source_check(default,b_head2,2,requests['prosody'],requested=None)
 assert named['result']==default['result']
 c=cli_call('current-compose-compact','workbench-example/computed-compose.json','compact',profile='computed');source_check(c,b_head2,2,requests['compose'],True,requested='computed')
 def rq(name,value):return file_new(base/(name+'.json'),enc(value)+b'\n')
 first=copy.deepcopy(source_request);first['options']['page_size']=1
 r=cli_call('source-page1',rq('source-page1',first),'compact');rows1=source_check(r,b_hist,1,first,True,[0])
 more=copy.deepcopy(source_request);more['options'].update(page_size=3,cursor=rows1['next_cursor'])
 r=cli_call('source-page3',rq('source-page3',more),'compact');rows2=source_check(r,b_hist,1,more,True,[1,2,3])
 rest=copy.deepcopy(source_request);rest['options']['cursor']=rows2['next_cursor']
 r=cli_call('source-rest',rq('source-rest',rest),'compact');rows3=source_check(r,b_hist,1,rest,True,[4,5,6,7]);assert rows3['next_cursor']is None
 assert rows1['hits']+rows2['hits']+rows3['hits']==product['computed']['historical_r1']['spelling_distance']['compact']['response']['result']['response']['hits']
 pp=copy.deepcopy(proposal_request);pp['options']['page_size']=1
 r=cli_call('proposal-page1',rq('proposal-page1',pp));p1=proposal_check(r,b_hist,pp,indices=[0])
 pp2=copy.deepcopy(pp);pp2['options']['cursor']=p1['next_cursor']
 r=cli_call('proposal-page2',rq('proposal-page2',pp2));p2=proposal_check(r,b_hist,pp2,indices=[1])
 pp3=copy.deepcopy(proposal_request);pp3['options']['cursor']=p2['next_cursor']
 r=cli_call('proposal-rest',rq('proposal-rest',pp3));p3=proposal_check(r,b_hist,pp3,indices=[2]);assert p3['next_cursor']is None
 wrong=copy.deepcopy(first);wrong['options']['cursor']=rows1['next_cursor']
 e=cli_call('wrong-format-cursor',rq('source-wrong-format',wrong),code=1);assert e['code']=='stale_cursor'
 ev=decode_token(rows1['next_cursor']);ev['body']['last']['assertion_id']='not-a-fixed-assertion';ev['checksum']=sha(enc(ev['body']))
 absent=copy.deepcopy(first);absent['options']['cursor']=PREFIX+enc(ev).hex()
 e=cli_call('absent-source-position',rq('source-absent-position',absent),'compact',code=1);assert e['code']=='invalid_cursor_position'
 invalid=copy.deepcopy(proposal_request);invalid['query']['proposals']=[fixed['proposal']['nonmatching_invalid_namespace_negative']];invalid['options']['page_size']=0
 e=cli_call('invalid-nonmatching-proposal',rq('proposal-invalid-nonmatching',invalid),code=1);assert e['code']=='invalid_lexical_request'
 assert len(children)==23
 for p,w in durable_before.items():assert pin(p)==w,p
 for p,w in planning_before.items():assert pin(p)==w,p
 for name,value in inputs.items():assert (W/name).read_bytes()==value,name
 preflight()
 (base/'execution.json').write_bytes(enc({'status':'PASS','children':children,
  'scope':'23 independent product children;54 inputs;original author limits preserved',
  'planning_pins':planning_before,'durable_before':durable_before}))
 print('PASS:23 original product children,20zero/3typedone;54 inputs and immutable postchecks',flush=True)
