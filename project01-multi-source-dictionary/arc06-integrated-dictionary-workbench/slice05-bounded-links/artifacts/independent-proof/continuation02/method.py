#!/usr/bin/env python3
"""S05 observer CONTINUATION02 DRAFT. Total finite31 product children: one existing
public Rust example binary and30 native CLI,23zero/8typedone. Includes EXACT
eight original guide roles under captured launcher/env/terminal. No Cargo,
new Rust consumer, source/index edits, Full corpus, acquisition or cleanup.
Complete DRAFT source is NOT runtime authority. Frozen source/binaries/
42INPUT/environment bindings +ROOT exact-method readiness precede any child.
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

def summary(r):
 return {'id':r['id'],'component_id':r['component'],'distributor_id':r['distributor'],**{k:r[k] for k in ['upstream_revision','distributor_revision','captured_at','time_precision','schema_version','normalizer','normalizer_version','captures','accounting']},'diagnostics_count':len(r['diagnostics']),'selected':r['selected'],'availability':r['availability'],'content_notices_count':len(r['content_notices']),'content_notice_licenses':[n['license'] for n in r['content_notices']],'content_notice_locators':[n['locator'] for n in r['content_notices']],'redistribution':r['redistribution'].lower(),'redistribution_evidence':r['redistribution_evidence']}

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

# Input-derived helpers. No query output is an expected-value authority.
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

def evidence_expected(d,e):
 if e is None:return None
 o=e['occurrence'];cap=optional(d['captures'],lambda c:c['identity']==o['capture'])
 snap=optional(d['registry']['snapshots'],lambda r:r['snapshot']==o['capture'])
 attr=optional(d['registry']['occurrences'],lambda r:r['occurrence']=={'snapshot':o['capture'],'locator':o['locator']})
 return {'evidence':e,'status':'Loaded' if cap else 'CatalogueOnly','reason':None if cap else 'missing_capture','capture':cap['metadata'] if cap else None,'raw':one(cap['occurrences'],lambda r:r['locator']==o['locator']) if cap else None,'snapshot_attribution':snap,'occurrence_attribution':attr,'contributor':one(d['registry']['contributors'],lambda r:r['id']==attr['contributor']) if attr and attr['contributor'] else None,'distributor':one(d['registry']['distributors'],lambda r:r['id']==snap['distributor']) if snap and snap['distributor'] else None}
def local_row(d,ref):
 role,id=next(iter(ref.items()));table={'Entry':'entries','Assertion':'assertions','Form':'forms','EditorialSense':'editorial_senses'}[role]
 return one(d['metadata'][table],lambda r:r['id']==id)
def local_source(d,ref):
 role,id=next(iter(ref.items()));r=local_row(d,ref)
 if role=='Entry':return r['source']
 if role in ['Assertion','Form']:return entry(d,r['entry'])['source']
 rev=one(d['metadata']['owned_revisions'],lambda x:x['id']==r['revision']);return rev['source']
def source_ids(d,ids):return [entry(d,assertion(d,id)['entry'])['source'] for id in ids]
def role_ids(d,ref):
 role,id=next(iter(ref.items()));r=local_row(d,ref)
 selected={a for rel in d['metadata']['releases'] if rel['selected'] and rel['availability']=='Available' for a in rel['assertions']}
 for h in d['metadata']['owned_heads']:
  rev=one(d['metadata']['owned_revisions'],lambda x:x['id']==h['revision'])
  owned={a['id'] for a in d['metadata']['assertions'] if entry(d,a['entry'])['source']==h['source']}
  selected-=owned-set(rev['assertions'])
 if role=='EditorialSense':return [a for a in r['assertions'] if a in selected]
 if role=='Assertion':return [id] if id in selected else []
 eid=id if role=='Entry' else r['entry']
 return [a['id'] for a in d['metadata']['assertions'] if a['entry']==eid and a['id'] in selected]

def record_expected(d,identity):
 if identity['kind']=='typed_relation':
  r=one(d['metadata']['relations'],lambda r:r['id']==identity['relation_id'])
  rev=one(d['metadata']['owned_revisions'],lambda v:v['id']==r['editorial_revision']) if r['editorial_revision'] else None
  origin=local_source(d,r['origin']);e=r['evidence']
  authority='source_evidence' if e else 'editorial_revision' if rev else 'unattributed_origin_source'
  publisher=origin if e or not rev else rev['source']
  return {'identity':identity,'publisher':source(d,publisher),'authority':authority,'origin_source':source(d,origin),'relation':r,'context':None,'token':None,'editorial_revision':rev,'evidence':evidence_expected(d,e)}
 c=one(d['metadata']['contexts'],lambda r:r['id']==identity['context_id']);sid=entry(d,assertion(d,c['assertion'])['entry'])['source']
 token={k:identity[k] for k in ['target_spelling','byte_start','byte_end','rule_version']}
 return {'identity':identity,'publisher':source(d,sid),'authority':'derived_source_clause','origin_source':source(d,sid),'relation':None,'context':c,'token':token,'editorial_revision':None,'evidence':evidence_expected(d,c['evidence'])}
def compact_record_expected(d,identity):
 full=record_expected(d,identity);r=full['relation'];origin=r['origin'] if r else {'Assertion':full['context']['assertion']}
 return {'identity':identity,'publisher':full['publisher']['id'],'authority':full['authority'],'origin':origin,'relation_kind':r['kind'] if r else None,'direction':r['direction'] if r else None,'scope':r['scope'] if r else None}
def link_key(d,identity,binding_sha):
 r=record_expected(d,identity);body=r['relation'] or r['context']
 return {'version':1,'digest_policy':'link-evidence-serde-json-v1','role':'link_evidence','binding_sha256':binding_sha,'publisher':r['publisher']['id'],'identity':identity,'record_sha256':sha(enc(body))}

def reference_check(res,d,compact,state,snapshot):
 keys(res,'target status operational_reason registrations catalogue_matches requested_release payload')
 target=res['target'];payload=res['payload']
 for reg in res['registrations']:
  keys(reg,'admitted disabled installation_state installed_release_sha256 operational_reason pack_id payload_kind release_sha256 requested_release selected sources')
  assert reg['pack_id']=='la-lojban' and reg['payload_kind']=='synthetic'
  assert reg['release_sha256']==sha(enc(json.loads(inputs['artifacts/links-inputs/mixed-pack-r1.json'])))
  assert reg['installed_release_sha256']==reg['release_sha256'] and reg['installation_state']=='installed'
  assert reg['selected'] and not reg['disabled'] and reg['admitted'] and reg['operational_reason'] is None
  for s in reg['sources']:assert s==source(d,s['id'])
 if payload is None:
  assert res['status'] in ['Unknown','Unresolved','NoMatch','CatalogueOnly'];return
 keys(payload,'version target status entity requested_release confirmed unverified availability state_sha256'+(' projection_policy snapshot_sha256' if compact else ''))
 assert payload['version']==1 and payload['target']==target and payload['status']==res['status'] and payload['state_sha256']==state
 assert payload['availability']==availability(d,compact)
 if compact:assert payload['projection_policy']=='compact-dictionary-v1' and payload['snapshot_sha256']==snapshot
 if res['requested_release'] is not None:
  rel=one(d['metadata']['releases'],lambda r:r['id']==res['requested_release']['id']);assert res['requested_release']==summary(rel)
 if payload['requested_release'] is not None:assert payload['requested_release']==one(d['metadata']['releases'],lambda r:r['id']==payload['requested_release']['id']) if not compact else payload['requested_release']==summary(one(d['metadata']['releases'],lambda r:r['id']==payload['requested_release']['id']))
 if isinstance(target,dict) and 'Local' in target:
  ref=target['Local'];role,id=next(iter(ref.items()));row=local_row(d,ref)
  if not compact:assert payload['entity']=={role:row}
  else:
   entity=payload['entity'];keys(entity,'role id source_ids entry_ids assertion_ids occurrence spelling evidence')
   assert entity['role']=={'Entry':'entry','Assertion':'assertion','Form':'form','EditorialSense':'editorial_sense'}[role] and entity['id']==id
   ids=role_ids(d,ref);assert entity['assertion_ids']==ids
   eids=[assertion(d,a)['entry'] for a in ids] if role=='EditorialSense' else [id if role=='Entry' else row['entry']]
   assert entity['entry_ids']==eids and entity['source_ids']==[entry(d,e)['source'] for e in eids]
   assert entity['spelling']==(row['spelling'] if role=='Entry' else None)
 for array in ['confirmed','unverified']:
  for v in payload[array]:
   a=assertion(d,v['assertion']['id']);e=entry(d,a['entry']);o=a['occurrence'];cap=one(d['captures'],lambda c:c['identity']==o['capture'])
   assert v['entry']==e and v['assertion']==a and v['occurrence_id']==occurrence_id(o) and v['capture_metadata']==cap['metadata']
   attr=one(d['registry']['occurrences'],lambda r:r['occurrence']=={'snapshot':o['capture'],'locator':o['locator']});assert v['occurrence_attribution']==attr
   if not compact:assert v['raw']==one(cap['occurrences'],lambda r:r['locator']==o['locator'])
   else:
    snap=one(d['registry']['snapshots'],lambda r:r['snapshot']==o['capture']);assert v['snapshot_attribution']==snap
   assert v['owned_revision_ids']==[r['id'] for r in d['metadata']['owned_revisions'] if a['id'] in r['assertions']]
   assert v['tombstoned_by']==[r['id'] for r in d['metadata']['owned_revisions'] if a['id'] in r['tombstones']]
   assert v['payload_present'] and v['current_eligible']==(a['id'] in role_ids(d,{'Assertion':a['id']}))

def row_projection(groups):return [{'publisher':g['source']['id'],'depth':h['depth'],'identity':h['identity'],'target_assertion_ids':h['target_assertion_ids']} for g in groups for h in g['hits']]
def fixed_case(name):
 if name in ['r1_depth2','r2_depth2']:return copy.deepcopy(fixed[name])
 f=copy.deepcopy(fixed[name]);f['rows']=[copy.deepcopy(fixed['r1_depth2']['rows'][i]) for i in f['row_indices_from_r1_depth2']]
 if 'targets_by_row' in f:
  for r,t in zip(f['rows'],f['targets_by_row']):r['target_assertion_ids']=t
 if 'source_totals' not in f:
  ids={r['publisher'] for r in f['rows']};f['source_totals']=[]
  for sid in sorted(ids):
   rows=[r for r in f['rows']if r['publisher']==sid];ev={enc(r['identity']['evidence'])for r in rows};asso={a for r in rows for a in [r['identity']['from_assertion_id'],*r['target_assertion_ids']]}
   f['source_totals'].append({'source':sid,'records':len(rows),'distinct_evidence':len(ev),'associated_assertions':len(asso)})
 return f

def observation_check(o,d,compact,state,snapshot,requested):
 keys(o,'entry resolution candidates traversal_status reason')
 if o['entry'] is not None:assert o['entry']==entry(d,o['entry']['id'])
 reference_check(o['resolution'],d,compact,state,snapshot)
 confirmed=[v['assertion']['id']for v in (o['resolution']['payload'] or {}).get('confirmed',[])];unverified=[v['assertion']['id']for v in (o['resolution']['payload'] or {}).get('unverified',[])]
 assert [v['assertion_id']for v in o['candidates']]==confirmed+unverified
 for c in o['candidates']:
  keys(c,'assertion_id source_id traversal_status reason');a=assertion(d,c['assertion_id']);sid=entry(d,a['entry'])['source'];assert c['source_id']==sid
  reason='unverified_identity' if a['id'] in unverified else 'no_current_eligible_assertion' if a['id'] not in role_ids(d,{'Assertion':a['id']}) or a['eligibility']!='Lexical' else 'requested_source' if sid not in requested else None
  assert c['reason']==reason and c['traversal_status']==('not_traversable' if reason else 'traversable')
 traversable=any(c['traversal_status']=='traversable'for c in o['candidates']);assert o['traversal_status']==('traversable'if traversable else'not_traversable')
 if traversable:assert o['reason'] is None
 elif o['resolution']['status'] in ['Unknown','Unresolved','NoMatch']:assert o['reason']=={'Unknown':'unknown','Unresolved':'unresolved','NoMatch':'no_match'}[o['resolution']['status']]
 elif not o['resolution']['payload']:assert o['reason']=='no_admitted_payload'
 elif not confirmed and unverified:assert o['reason']=='unverified_identity'
 elif confirmed and any(c['reason']=='requested_source' for c in o['candidates']):assert o['reason']=='requested_source'
 else:assert o['reason']=='no_current_eligible_assertion'

def links_check(out,b,revision,compact,case='r1_depth2',indices=None,profile='direct',request=None,terminal=True,config_revision=4):
 eff=outer(out,b,'compact'if compact else'full',profile,config_revision=config_revision);keys(out['result'],'operation response');assert out['result']['operation']=='links'
 r=out['result']['response'];keys(r,'policy state_sha256 query resolved_sources seed_assertion_ids totals source_totals availability groups next_cursor code_notices'+(' snapshot_sha256'if compact else''))
 d=models[revision];want=fixed_case(case);wantrows=want['rows'];page=wantrows if indices is None else [wantrows[i]for i in indices]
 assert r['policy']=='bounded-links-v1' and r['state_sha256']==b['state_sha256'] and r['query']==(request or depth2)['query']
 requested=['links-upstream']if case=='only_KU_depth2'else['links-owned']if case=='only_KO_depth2'else['links-owned','links-upstream']
 assert r['resolved_sources']==requested and r['seed_assertion_ids']==[a for a in fixed['seed_assertion_ids']if entry(d,assertion(d,a)['entry'])['source']in requested]
 assert r['totals']==want['totals'];assert r['source_totals']==[{**x,'source':source(d,x['source'])}for x in want['source_totals']]
 assert r['availability']==availability(d,compact) and r['code_notices']==[]
 snapshot=r.get('snapshot_sha256');
 if compact:digest(snapshot)
 assert row_projection(r['groups'])==page
 assert [g['source']['id']for g in r['groups']]==list(dict.fromkeys(h['publisher']for h in page))
 for g in r['groups']:
  keys(g,'source full_count returned_count partial hits');assert g['source']==source(d,g['source']['id'])
  full_count=one(want['source_totals'],lambda x:x['source']==g['source']['id'])['records'];assert g['full_count']==full_count and g['returned_count']==len(g['hits']) and g['partial']==(len(g['hits'])<full_count)
  for h in g['hits']:
   keys(h,'identity depth from record origin_resolution target_observations target_assertion_ids traversal_status'+(' detail_ref'if compact else''));identity=h['identity']['evidence'];a=assertion(d,h['identity']['from_assertion_id']);hit_check(h['from'],d,a,compact)
   assert h['record']==(compact_record_expected(d,identity)if compact else record_expected(d,identity))
   if compact:assert h['detail_ref']==link_key(d,identity,eff)
   reference_check(h['origin_resolution'],d,compact,r['state_sha256'],snapshot)
   for o in h['target_observations']:observation_check(o,d,compact,r['state_sha256'],snapshot,requested)
   assert h['traversal_status']==('traversable'if h['target_assertion_ids']else'observed_only')
   if identity['kind']=='source_clause' and identity['target_spelling']=='branch':assert [o['entry']['id']for o in h['target_observations']]==fixed['clause_branch_entry_order']
   if identity['kind']=='typed_relation' and identity['relation_id']in ['links-relation-07','links-relation-08']:
    tag='R07'if identity['relation_id'].endswith('07')else'R08';oracle=fixed['format_specific'][tag]['Compact'if compact else'Full'];res=h['target_observations'][0]['resolution'];p=res['payload']
    assert res['status']==oracle['status'];assert (p is not None)==oracle.get('payload',True)
    assert [v['assertion']['id']for v in (p or {}).get('confirmed',[])]==oracle['confirmed'];assert [v['assertion']['id']for v in (p or {}).get('unverified',[])]==oracle['unverified']
 assert (r['next_cursor'] is None)==terminal
 return r

def direct(root,revision,head=False):
 b=json.loads((root/'binding.json').read_bytes());keys(b,'version kind selection catalogue owned contexts resolutions selected_release_ids state_sha256 binding_sha256')
 assert b['version']==1 and b['kind']=='vlajmaji-workbench'
 assert b['selection']==json.loads(inputs['artifacts/links-inputs/selection-historical-r1.json']if revision==1 and not head else inputs['artifacts/links-inputs/selection-head.json'])
 assert b['contexts']==[] and b['resolutions']==[]
 c=b['catalogue'];assert c['root']==native(L/'catalogue')and c['local_revision']==1 and c['recorded_state_sha256']==sha(enc(json.loads(inputs['artifacts/links-inputs/mixed-pack-r1.json'])))
 assert c['members']==[{'pack_id':'la-lojban','release_sha256':sha(enc(json.loads(inputs['artifacts/links-inputs/mixed-pack-r1.json']))),'payload_kind':'synthetic'}]
 assert b['owned']['root']==native(L/'owned')and b['owned']['selection']==b['selection']['owned']
 assert b['owned']['identity']=={'journal_schema':1,'source':'links-owned','revision':'links-owned-r'+str(revision),'sequence':revision,'state_sha256':sha(enc(json.loads(inputs['artifacts/links-inputs/owned-r'+str(revision)+'.json']))),'curation_state_sha256':None}
 assert b['selected_release_ids']==[r['id']for r in models[revision]['metadata']['releases']if r['selected']and r['availability']=='Available']
 digest(b['state_sha256']);assert b['binding_sha256']==sha(enc({k:b[k]for k in ['version','kind','selection','catalogue','owned','contexts','resolutions','selected_release_ids','state_sha256']}))
 return b

def outer(out,b,fmt,profile='direct',detail=False,config_revision=4):
 keys(out,'version kind binding_sha256 selection '+('reference record origin_resolution target_observations'if detail else'format result')+(' configuration'if profile!='direct'else''))
 assert out['version']==1 and out['kind']==('vlajmaji-workbench-link-detail'if detail else'vlajmaji-workbench-lexical')
 if not detail:assert out['format']==fmt
 owned=b['owned']['identity'];c=b['catalogue']
 assert out['selection']=={'version':1,'packs':['la-lojban'],'disabled':[],'upstream':{'local_revision':1,'generation_id':c['generation_id'],'content_id':c['content_id'],'state_sha256':c['recorded_state_sha256']},'owned':{'mode':b['selection']['owned'],'source':'links-owned','revision':owned['revision'],'sequence':owned['sequence'],'journal_schema':1,'state_sha256':owned['state_sha256'],'curation_sha256':None},'context_counts':{'declared':0,'provided':0},'curation_counts':None}
 if profile=='direct':assert out['binding_sha256']==b['binding_sha256']
 else:
  # Protocol named field order, never the sorted CLI map order.
  assert config_revision in [3,4]
  config_sha=sha(cfg.read_bytes())
  if config_revision==3:
   wire=json.loads(cfg.read_bytes());assert wire['revision']==4 and len(wire['profiles'])==1 and [p['sequence']for p in wire['prepared']]==[0,1]
   wire['revision']=3;wire['profiles'][0]['active_generation']=0;wire['prepared']=wire['prepared'][:1]
   config_sha=sha(enc(wire)+b'\n')
  ident={'version':1,'kind':'vlajmaji-configured-workbench','config':native(cfg),'config_revision':config_revision,'config_sha256':config_sha,'requested_profile':profile,'profile':'links','prepared_sequence':0 if config_revision==3 else 1,'view_binding_sha256':b['binding_sha256']}
  ident['effective_binding_sha256']=sha(enc(ident));assert out['configuration']==ident and out['binding_sha256']==ident['effective_binding_sha256']
 return out['binding_sha256']

def detail_check(out,b,revision,key,profile='direct'):
 eff=outer(out,b,'full',profile,True);d=models[revision];assert out['reference']==key and key==link_key(d,key['identity'],eff)
 assert out['record']==record_expected(d,key['identity'])
 reference_check(out['origin_resolution'],d,True,b['state_sha256'],(out['origin_resolution']['payload']or{}).get('snapshot_sha256'))
 # Detail uses cached Compact reference results; their snapshot digest is genuine
 # provider identity, not fabricated from the binding or equal to state by fiat.
 for o in out['target_observations']:
  snap=(o['resolution']['payload']or{}).get('snapshot_sha256');observation_check(o,d,True,b['state_sha256'],snap,['links-owned','links-upstream'])
 return out

def readable_check(text,out,detail=False):
 # Parse semantic complete JSON after exact labels/indentation. Preserve arrays
 # and group order, do not prescribe an additional trailing newline.
 lines=text.splitlines();i=0
 def consume(label,value):
  nonlocal i
  assert i<len(lines)and lines[i].startswith(label+': '),(i,label)
  assert json.loads(lines[i][len(label)+2:])==value,(i,label);i+=1
 consume('workbench_binding',out['binding_sha256'])
 if 'configuration'in out:consume('workbench_configuration',out['configuration'])
 consume('workbench_selection',out['selection'])
 if detail:
  for label,key in [('link_reference','reference'),('record','record'),('origin_resolution','origin_resolution'),('target_observations','target_observations')]:consume(label,out[key])
  consume('link_policy','bounded-links-v1')
 else:
  r=out['result']['response'];consume('lexical_operation','links');consume('lexical_policy','bounded-links-v1')
  for label,key in [('lexical_totals','totals'),('lexical_source_totals','source_totals'),('lexical_next_cursor','next_cursor')]:consume(label,r[key])
  for g in r['groups']:
   consume('source',g['source']);consume('full_count',g['full_count']);consume('returned_count',g['returned_count']);consume('partial',g['partial'])
   for h in g['hits']:
    consume('  step',h['identity'])
    for field in ['depth','from','record','origin_resolution','target_observations','target_assertion_ids','traversal_status','detail_ref']:
     consume('    '+('evidence'if field=='record'else'link_detail'if field=='detail_ref'else field),h[field])
 assert i==len(lines),(i,len(lines),lines[i:i+2])

def cursor_check(token,binding_sha,request,last):
 protocol=inputs['artifacts/links-inputs/links-cursor-protocol.json'];oracle=json.loads(protocol)
 # Check immutable artificial byte oracles separately from a genuine live token.
 for raw,hashkey in [('query_payload_utf8','query_sha256'),('reversed_selectors_query_payload_utf8','reversed_selectors_query_sha256'),('body_utf8','checksum')]:assert sha(oracle[raw].encode())==oracle[hashkey]
 assert oracle['token']=='workbench-lexical-v1:'+oracle['envelope_utf8'].encode().hex()
 query={'query':{'operation':'links','spelling':request['query']['spelling'],'depth':request['query']['depth']},'selectors':request['options']['sources'],'resolved_sources':['links-owned','links-upstream'],'alias_scope':request['options']['alias_scope']}
 ev=last['identity']['evidence'];ev={k:ev[k]for k in (['kind','relation_id']if ev['kind']=='typed_relation'else['kind','context_id','target_spelling','byte_start','byte_end','rule_version'])}
 pos={'operation':'links','source':last['publisher'],'depth':last['depth'],'evidence_kind_rank':0 if ev['kind']=='typed_relation'else 1,'evidence':ev,'orientation_rank':0 if last['identity']['orientation']=='forward'else 1,'from_assertion_id':last['identity']['from_assertion_id']}
 body={'version':1,'domain':'workbench-lexical-cursor-v1','format':'compact','binding_sha256':binding_sha,'query_sha256':sha(enc(query)),'last':pos}
 envelope={'body':body,'checksum':sha(enc(body))}
 assert token=='workbench-lexical-v1:'+enc(envelope).hex()
 return envelope

# Concrete finite calls and predicates, declared before the first child.
PLAN=[
 ('public-example','existing compiled example;42 INPUT byte parity before child2;all11 Links result cases fixed rows/counts and input bodies'),
 ('guide-historical-full','guide1: historical r1 Full thirteen ordered steps, complete input-derived records and source bodies'),
 ('guide-historical-compact','guide2: same historical request Compact;format-local R07/R08;actual detail keys'),
 ('guide-historical-detail','guide3: first reported key+samebinding;actual R03 record/revision and all-selectable target closure'),
 ('guide-configured-readable','guide4: named links current r2 Readable equals complete labelled Compact JSON, exact effective identity'),
 ('guide-named-full','guide5: named links r2 Full thirteen rows;associated/visitedfive'),
 ('guide-default-full','guide6: default r2 Full, exact requested_profile null and result equal guide5'),
 ('guide-named-compact','guide7: named links r2 Compact and same-format native outcomes'),
 ('guide-named-detail','guide8: first current configured Compact key paired with effective binding;actual record'),
 ('historical-readable','complete labelled Readable equals child3 Compact;no extra LF assumption'),
 ('source-record-detail','R01 source-backed actual record/evidence/raw/attributions from fixed INPUT'),
 ('source-detail-readable','four complete detail JSON labels equal child11 plus exact outer labels'),
 ('current-direct-full','direct current r2 Full agrees fixed r2 input-derived graph, distinct owned head semantics'),
 ('current-direct-compact','direct current r2 Compact own binding/keys/native statuses'),
 ('depth-one','fixed depth1 seven rows and complete named totals/source totals'),
 ('KU-filter','five fixed KU identities AND independently named filtered target arrays/observations'),
 ('KO-filter','three fixed KO identities, empty hops, all named totals'),
 ('page-zero','legal page0 retains full totals/source totals, groups empty,next null'),
 ('page-first','actual issued Compact token after fixed first row;typed ordered hash'),
 ('page-next','consume same issued token with page size3;rows1..3,new token fixedLAST'),
 ('page-rest','consume next token with page size null;remaining9,terminalNone,group partial correctlytrue'),
 ('wrong-format-cursor','typed stale_cursor and exact3-field error envelope'),
 ('changed-query-cursor','typed stale_cursor;query differs but otherwisevalid'),
 ('bad-detail-version','typed malformed_link_detail for constructible wrongversion key'),
 ('foreign-form-key','strict LinkDetail FILE rejects old Form key with request_json BEFOREopen'),
 ('malformed-request','malformed JSON request_json BEFOREopen'),
 ('invalid-depth','depth3 invalid_lexical_request after guard'),
 ('old-key-current','genuine historical key+fresh current outer binding gives stale_link_detail'),
 ('forged-record-sha','validshaped actualidentity changedbodyhash gives stale_link_detail'),
 ('KU-compact','sameview filter-scoped traversal, clause9 key identical to unfiltered key'),
 ('filter-independent-detail','sameclause9 key all-selectable detail makes LO-b traversable despite KU query exclusion')]
assert len(PLAN)==31

if __name__=='__main__':
 assert len(sys.argv)==2,'frozen binding JSON required; draft is not runtime authority'
 binding=json.loads(Path(sys.argv[1]).read_bytes());assert binding['status']=='FROZEN_CDC_VERIFICATION'
 src=Path(binding['source']);cli=Path(binding['cli']);example=Path(binding['example']);env=os.environ.copy()
 assert sha(env['PATH'].encode())==binding['path_sha256']and env['RUSTUP_AUTO_INSTALL']=='0'
 for k in ['CARGO_HOME','RUSTUP_HOME']:assert env[k]==binding['toolhomes'][k]
 check_pins();assert len(binding['source_files'])==17
 for p in [cli,example,Path(__file__),*[src/r for r in binding['source_files']]]:assert str(p)in binding['files'],p
 inputs={}
 for emitted,row in binding['inputs'].items():
  p=Path(row['path']);assert pin(p)=={k:row[k]for k in ['bytes','sha256']}and str(p)in binding['files'];inputs[emitted]=p.read_bytes()
 assert len(inputs)==42 and len(set(inputs))==42
 pack=json.loads(inputs['artifacts/links-inputs/mixed-pack-r1.json']);r1=json.loads(inputs['artifacts/links-inputs/owned-r1.json']);r2=json.loads(inputs['artifacts/links-inputs/owned-r2.json'])
 fixed=json.loads(inputs['artifacts/links-inputs/links-expected.json']);spec=json.loads(inputs['artifacts/links-inputs/links-fixture-spec.json'])
 depth1=json.loads(inputs['artifacts/links-inputs/links-depth1.json']);depth2=json.loads(inputs['artifacts/links-inputs/links-depth2.json']);models={1:merge_models(pack,r1),2:merge_models(pack,r2)}
 assert len(pack['metadata']['relations'])==8 and len(pack['metadata']['contexts'])==1 and pack['metadata']['owned_heads']==[]
 assert r2['metadata']['owned_revisions'][-1]['assertions']==['links-owned-a','links-owned-b']and r2['metadata']['owned_revisions'][-1]['tombstones']==['links-owned-c']
 assert r2['metadata']['releases'][-1]['assertions']==['links-owned-a','links-owned-b','links-owned-c']
 for d in models.values():
  for a in spec['assertions']:
   actual=assertion(d,a['id']);assert actual['entry']==a['entry']and actual['definition']['Value']['text']==a['definition']and actual['eligibility']=='Lexical'
  for e in spec['entries']:assert entry(d,e['id'])['source']==e['source']and entry(d,e['id'])['spelling']=={'Value':e['spelling']}
  for cap in spec['captures']:
   actual=one(d['captures'],lambda c:c['identity']['namespace']==cap['namespace']);assert actual['identity']['sha256']==sha(cap['bytes_utf8'].encode())==cap['sha256']and actual['metadata']['format_label']==cap['format_label']
 # Persist exact complete record/evidence expectations from inputs before child1.
 expected={str(rev):{'records':[record_expected(d,r['identity']['evidence'])for r in fixed['r'+str(rev)+'_depth2']['rows']],'availability_full':availability(d,False),'availability_compact':availability(d,True)}for rev,d in models.items()}
 protocol=json.loads(inputs['artifacts/links-inputs/links-cursor-protocol.json']);protoquery=copy.deepcopy(depth2);protoquery['options']['sources']=['KU','KO']
 cursor_check(protocol['token'],'1'*64,protoquery,fixed['r1_depth2']['rows'][0])
 base=Path(binding['continuation']['base']);assert base==Path('/private/tmp/vlajmaji-a06-s05-cdc-ube57fdz')
 selected_env=json.loads((base/'selected-env.json').read_bytes())
 for k,leaf in [('HOME','home'),('XDG_DATA_HOME','xdg-data'),('XDG_CONFIG_HOME','xdg-config'),('TMPDIR','tmp')]:assert selected_env[k]==str(base/leaf)and(base/leaf).is_dir()
 env.update(selected_env)
 assert enc(expected)==(base/'expected-before-children.json').read_bytes()and enc(PLAN)==(base/'planned-children.json').read_bytes()
 with(base/'selected-env-continuation02.json').open('xb')as f:f.write(enc(selected_env))
 W=base/'workbench-example';L=W/'links-workbench';I=W/'artifacts/links-inputs';hist=L/'view-historical-r1';current=L/'view-head-r2';cfg=L/'links-workbench-profiles.json'
 requests={}
 def req(name,value):
  p=base/(name+' ü.json')
  if name in ['depth two','depth one','KU filter','KO filter','zero','first','invalid depth','old Form key','source evidence detail']:assert p.read_bytes()==enc(value)
  else:
   with p.open('xb')as f:f.write(enc(value))
  requests[name]={'path':str(p),**pin(p)};return p
 depth2file=req('depth two',depth2);depth1file=req('depth one',depth1)
 ku=copy.deepcopy(depth2);ku['options']['sources']=['KU'];kufile=req('KU filter',ku)
 ko=copy.deepcopy(depth2);ko['options']['sources']=['KO'];kofile=req('KO filter',ko)
 zero=copy.deepcopy(depth2);zero['options']['page_size']=0;zerofile=req('zero',zero)
 first=copy.deepcopy(depth2);first['options']['page_size']=1;firstfile=req('first',first)
 invalid=copy.deepcopy(depth2);invalid['query']['depth']=3;invalidfile=req('invalid depth',invalid)
 malformed=base/'malformed ü.json';assert malformed.read_bytes()==b'{'
 oldform={'version':1,'digest_policy':'form-entry-computation-serde-json-v1','role':'form','binding_sha256':'a'*64,'source':'links-upstream','entry_id':'links-upstream-entry-z','form_id':'links-upstream-form-z','entry_sha256':'b'*64,'form_sha256':'c'*64,'computation_sha256':None};oldformfile=req('old Form key',oldform)
 assert (base/'negative-buffers-before-children.json').read_bytes()==enc({str(p):pin(p)for p in [invalidfile,malformed,oldformfile]})
 prefix=json.loads((base/'commands-continuation01.json').read_bytes());assert len(prefix)==17
 for number,row in enumerate(prefix,1):
  assert row['n']==number and row['name']==PLAN[number-1][0]and row['cwd']==str(base)and row['expected_exit']==row['actual_exit']==0 and not row['timeout']
  for field in ['stdout','stderr']:assert pin(row[field])==row[field+'_pin']
 children=[prefix[0]]
 original=children[0];assert original['n']==1 and original['name']=='public-example'and original['argv']==[str(example),str(W)]and original['cwd']==str(base)and original['expected_exit']==original['actual_exit']==0 and not original['timeout']
 for field in ['stdout','stderr']:assert pin(original[field])==original[field+'_pin']
 assert Path(original['stderr']).read_bytes()==b''

 def run(name,argv,code=0,readable=False):
  n=len(children)+1;assert n<=31 and PLAN[n-1][0]==name
  retained=n<=17
  if retained:
   row=prefix[n-1];assert row['argv']==[str(a)for a in argv]and row['cwd']==str(base)and row['expected_exit']==code
   for field in ['stdout','stderr']:assert pin(row[field])==row[field+'_pin']
   out=Path(row['stdout']).read_bytes();err=Path(row['stderr']).read_bytes();actual=row['actual_exit'];timed=row['timeout']
  else:
   assert not(base/f'{n:02}-{name}.stdout').exists()and not(base/f'{n:02}-{name}.stderr').exists()
   started=time.time()
   try:
    p=subprocess.run([str(a)for a in argv],cwd=base,env=env,capture_output=True,timeout=300);out,err,actual,timed=p.stdout,p.stderr,p.returncode,False
   except subprocess.TimeoutExpired as e:out,err,actual,timed=e.stdout or b'',e.stderr or b'',None,True
   op=base/f'{n:02}-{name}.stdout';ep=base/f'{n:02}-{name}.stderr'
   with op.open('xb')as f:f.write(out)
   with ep.open('xb')as f:f.write(err)
   row={'n':n,'name':name,'predicate':PLAN[n-1][1],'argv':[str(a)for a in argv],'cwd':str(base),'expected_exit':code,'actual_exit':actual,'timeout':timed,'seconds':time.time()-started,'stdout':str(op),'stderr':str(ep),'stdout_pin':pin(op),'stderr_pin':pin(ep)}
  guide_files={'guide-historical-full':'links-direct-full.json','guide-historical-compact':'links-direct-compact.json','guide-historical-detail':'links-direct-detail.json','guide-configured-readable':'links-configured-readable.txt','guide-named-full':'links-named-full-r2.json','guide-default-full':'links-default-full-r2.json','guide-named-compact':'links-named-compact-r2.json','guide-named-detail':'links-named-detail-r2.json'}
  if name in guide_files:
   documented=W/guide_files[name];assert documented.read_bytes()==out and row['documented_output']==str(documented)and row['documented_output_pin']==pin(documented)
  children.append(row)
  if not retained:(base/'commands-continuation02.json').write_bytes(enc(children))
  assert not timed and actual==code,row
  if code:
   assert out==b''and err.endswith(b'\n')and err.count(b'\n')==1
   obj=json.loads(err);keys(obj,'error');keys(obj['error'],'code message sources');assert isinstance(obj['error']['message'],str)and obj['error']['sources']==[];return obj['error']
  assert err==b'',(name,err)
  if readable:return out.decode()
  assert out.endswith(b'\n')
  if name!='public-example':assert out.count(b'\n')==1
  return json.loads(out)
 def v(name,*args,**kw):return run(name,[cli,*args],**kw)
 def lexical(name,root,path,fmt='full',profile='direct',**kw):
  guide=name.startswith('guide-');path=Path(path).relative_to(base)if guide else path;config=cfg.relative_to(base)if guide else cfg
  if root is not None and guide:root=Path(root).relative_to(base)
  args=['workbench']
  if profile!='direct':args+=['profiles','lexical','--config',config]+(['--profile',profile]if profile else[])
  else:args+=['lexical',root]
  return v(name,*args,'--request',path,'--format',fmt,readable=fmt=='readable',**kw)
 def detail(name,root,path,expected_binding,fmt='full',profile='direct',**kw):
  guide=name.startswith('guide-');path=Path(path).relative_to(base)if guide else path;config=cfg.relative_to(base)if guide else cfg
  if root is not None and guide:root=Path(root).relative_to(base)
  args=['workbench']
  if profile!='direct':args+=['profiles','link-detail','--config',config]+(['--profile',profile]if profile else[])
  else:args+=['link-detail',root]
  return v(name,*args,'--reference',path,'--expected-binding',expected_binding,'--format',fmt,readable=fmt=='readable',**kw)
 public=json.loads(Path(original['stdout']).read_bytes())
 # Verify ALL PATH-qualified emitted authorities before ANY CLI product child.
 for emitted,raw in inputs.items():assert (W/emitted).read_bytes()==raw,emitted
 assert public['profile_revisions']==[1,2,3,4,5,6]
 b1=direct(hist,1);b2=direct(current,2);bc=direct(L/'view-configured-r2',2);bcr1=direct(L/'view-configured-r1',1,head=True)
 assert bcr1['selection']['owned']=='head'and b1['selection']['owned']=={'revision':'links-owned-r1'}and bcr1['binding_sha256']!=b1['binding_sha256']
 assert bcr1['owned']['identity']==b1['owned']['identity']and bcr1['state_sha256']==b1['state_sha256']
 assert b2['binding_sha256']==bc['binding_sha256']
 before={str(p):pin(p)for root in [hist,current,L/'view-configured-r2']for p in [root/'binding.json',root/'dictionary/dictionary.sqlite3']};before[str(cfg)]=pin(cfg)
 for emitted in inputs:before[str(W/emitted)]=pin(W/emitted)
 # The example's11 query bodies are observations, checked against preloaded inputs.
 pub=public['links'];assert pub['request']==depth2 and pub['depth1_request']==depth1
 for key,rev,compact,profile,case in [
  ('historical_full_r1',1,False,'direct','r1_depth2'),('historical_compact_r1',1,True,'direct','r1_depth2'),('historical_full_r1_depth1',1,False,'direct','depth1'),('historical_compact_r1_depth1',1,True,'direct','depth1'),('current_full_r2',2,False,'direct','r2_depth2'),('current_compact_r2',2,True,'direct','r2_depth2'),('configured_r2_full',2,False,'links','r2_depth2'),('configured_r2_compact',2,True,'links','r2_depth2'),('default_r2_full',2,False,None,'r2_depth2')]:
  links_check(pub[key],b1 if rev==1 else b2,rev,compact,case,profile=profile,request=depth1 if case=='depth1'else depth2)
 # Source run_links_journey configures1/refreshr1=>2/set_default=>3 BEFORE
 # retained configured-r1 queries. HEAD binding is distinct from historical r1.
 for key,compact in [('configured_r1_full',False),('configured_r1_compact',True)]:
  links_check(pub[key],bcr1,1,compact,profile='links',config_revision=3)
 for key,bview,rev,profile in [('historical_link_detail',b1,1,'direct'),('configured_r2_detail',b2,2,'links')]:detail_check(pub[key],bview,rev,pub[key]['reference'],profile)
 # Guide8 are retained actual prefix records, reread/asserted only below.
 # EXACT guide8 prefix observations are rechecked without executing any child.
 g1=lexical('guide-historical-full',hist,I/'links-depth2.json');links_check(g1,b1,1,False)
 g2=lexical('guide-historical-compact',hist,I/'links-depth2.json','compact');r=links_check(g2,b1,1,True)
 firstkey=r['groups'][0]['hits'][0]['detail_ref'];firstkeyfile=W/'links-direct-detail-ref.json'
 assert firstkeyfile.read_bytes()==enc(firstkey)
 requests['guide jq-equivalent first-key extraction']={'path':str(firstkeyfile),**pin(firstkeyfile)}
 g3=detail('guide-historical-detail',hist,firstkeyfile,g2['binding_sha256']);detail_check(g3,b1,1,firstkey)
 g4=lexical('guide-configured-readable',None,I/'links-depth2.json','readable','links');readable_check(g4,pub['configured_r2_compact'])
 g5=lexical('guide-named-full',None,I/'links-depth2.json','full','links');links_check(g5,b2,2,False,'r2_depth2',profile='links')
 g6=lexical('guide-default-full',None,I/'links-depth2.json','full',None);links_check(g6,b2,2,False,'r2_depth2',profile=None);assert g5['result']==g6['result']
 g7=lexical('guide-named-compact',None,I/'links-depth2.json','compact','links');r=links_check(g7,b2,2,True,'r2_depth2',profile='links')
 currentkey=r['groups'][0]['hits'][0]['detail_ref'];currentkeyfile=W/'links-named-detail-ref-r2.json'
 assert currentkeyfile.read_bytes()==enc(currentkey)
 requests['guide jq-equivalent named-key extraction']={'path':str(currentkeyfile),**pin(currentkeyfile)}
 g8=detail('guide-named-detail',None,currentkeyfile,g7['binding_sha256'],'full','links');detail_check(g8,b2,2,currentkey,'links')
 readable_check(lexical('historical-readable',hist,depth2file,'readable'),g2)
 r01=one([h for g in g2['result']['response']['groups']for h in g['hits']],lambda h:h['identity']['evidence']=={'kind':'typed_relation','relation_id':'links-relation-01'})['detail_ref'];r01file=req('source evidence detail',r01)
 sdetail=detail('source-record-detail',hist,r01file,g2['binding_sha256']);detail_check(sdetail,b1,1,r01)
 readable_check(detail('source-detail-readable',hist,r01file,g2['binding_sha256'],'readable'),sdetail,True)
 links_check(lexical('current-direct-full',current,depth2file),b2,2,False,'r2_depth2')
 links_check(lexical('current-direct-compact',current,depth2file,'compact'),b2,2,True,'r2_depth2')
 links_check(lexical('depth-one',hist,depth1file),b1,1,False,'depth1',request=depth1)
 kufull=lexical('KU-filter',hist,kufile);links_check(kufull,b1,1,False,'only_KU_depth2',request=ku)
 links_check(lexical('KO-filter',hist,kofile,'compact'),b1,1,True,'only_KO_depth2',request=ko)
 links_check(lexical('page-zero',hist,zerofile),b1,1,False,indices=[],request=zero)
 p1=lexical('page-first',hist,firstfile,'compact');a=links_check(p1,b1,1,True,indices=[0],request=first,terminal=False);cursor_check(a['next_cursor'],p1['binding_sha256'],first,fixed['r1_depth2']['rows'][0])
 follow=copy.deepcopy(first);follow['options'].update(cursor=a['next_cursor'],page_size=3);followfile=req('continuation three',follow)
 p2=lexical('page-next',hist,followfile,'compact');b=links_check(p2,b1,1,True,indices=[1,2,3],request=follow,terminal=False);cursor_check(b['next_cursor'],p2['binding_sha256'],follow,fixed['r1_depth2']['rows'][3])
 rest=copy.deepcopy(follow);rest['options'].update(cursor=b['next_cursor'],page_size=None);restfile=req('continuation remaining',rest)
 p3=lexical('page-rest',hist,restfile,'compact');links_check(p3,b1,1,True,indices=list(range(4,13)),request=rest)
 assert row_projection(p1['result']['response']['groups'])+row_projection(p2['result']['response']['groups'])+row_projection(p3['result']['response']['groups'])==fixed['r1_depth2']['rows']
 assert lexical('wrong-format-cursor',hist,followfile,code=1)['code']=='stale_cursor'
 changed=copy.deepcopy(follow);changed['query']['spelling']='branch';changedfile=req('changed cursor query',changed)
 assert lexical('changed-query-cursor',hist,changedfile,'compact',code=1)['code']=='stale_cursor'
 bad=copy.deepcopy(firstkey);bad['version']=2;badfile=req('bad detail version',bad)
 assert detail('bad-detail-version',hist,badfile,g2['binding_sha256'],code=1)['code']=='malformed_link_detail'
 assert detail('foreign-form-key',base/'ABSENT prepared root',oldformfile,'a'*64,code=1)['code']=='request_json'
 assert lexical('malformed-request',base/'ABSENT prepared root',malformed,code=1)['code']=='request_json'
 assert lexical('invalid-depth',hist,invalidfile,code=1)['code']=='invalid_lexical_request'
 assert detail('old-key-current',current,firstkeyfile,b2['binding_sha256'],code=1)['code']=='stale_link_detail'
 forged=copy.deepcopy(firstkey);forged['record_sha256']='0'*64;forgedfile=req('forged body SHA',forged)
 assert detail('forged-record-sha',hist,forgedfile,g2['binding_sha256'],code=1)['code']=='stale_link_detail'
 kuc=lexical('KU-compact',hist,kufile,'compact');links_check(kuc,b1,1,True,'only_KU_depth2',request=ku)
 identity=one([h for g in g2['result']['response']['groups']for h in g['hits']],lambda h:h['identity']['evidence']['kind']=='source_clause'and h['identity']['evidence']['byte_start']==9)['detail_ref']
 filtered=one([h for g in kuc['result']['response']['groups']for h in g['hits']],lambda h:h['identity']['evidence']==identity['identity']);assert filtered['detail_ref']==identity
 assert [(o['entry']['id'],o['reason'])for o in filtered['target_observations']]==[(o['entry'],o['reason'])for o in fixed['only_KU_depth2']['branch_observations']]
 clausefile=req('filter independent clause detail',identity);clause=detail('filter-independent-detail',hist,clausefile,g2['binding_sha256']);detail_check(clause,b1,1,identity)
 lob=one(clause['target_observations'],lambda o:o['entry']['id']=='links-owned-entry-b');assert lob['traversal_status']=='traversable'and lob['reason']is None
 assert len(children)==31 and sum(r['actual_exit']==0 for r in children)==23 and sum(r['actual_exit']==1 for r in children)==8
 for p,wanted in before.items():assert pin(p)==wanted,p
 for row in requests.values():assert pin(row['path'])=={k:row[k]for k in ['bytes','sha256']}
 for emitted,raw in inputs.items():assert(W/emitted).read_bytes()==raw
 check_pins()
 receipt={'status':'PASS','scope':'31 original finite children; no rebuild/new consumer/default data/real Full/extra controls','base':str(base),'binding_file':str(Path(sys.argv[1]).resolve()),'binding_pin':pin(sys.argv[1]),'children':children,'selected_env':selected_env,'requests':requests,'post_unchanged':before,'pre_input_pin_count':42,'original_frozen_pin_count':len(binding['files'])}
 (base/'execution-continuation02.json').write_bytes(enc(receipt));print('PASS '+str(base),flush=True)
