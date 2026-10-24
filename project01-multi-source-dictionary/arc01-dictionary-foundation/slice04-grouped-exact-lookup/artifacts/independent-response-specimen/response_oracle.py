"""CDC preimplementation source-value oracle. No product/store/query imports."""
import copy,hashlib,json

def oid(o):
 c=o.get('capture',o.get('snapshot'))
 return b''.join(len(x.encode()).to_bytes(8,'big')+x.encode() for x in [c['namespace'],c['sha256'],o['locator']]).hex()
def ck(c):return c['namespace'],c['sha256']
def pick(d,q,source_ids=None):
 m=d['metadata']; es={e['id']:e for e in m['entries']}; owned={s['id'] for s in d['registry']['sources'] if s['origin']=='OwnedDraft'}
 rs={r['id']:r for r in m['owned_revisions']};heads={h['source']:set(rs[h['revision']]['assertions']) for h in m['owned_heads']}
 selected=set(a for r in m['releases'] if r['selected'] and r['availability']=='Available' for a in r['assertions'])
 result=[]
 for a in m['assertions']:
  e=es[a['entry']]; s=e['source']
  if a['id'] not in selected or a['eligibility']!='Lexical' or e['spelling']!={'Value':q}:continue
  if source_ids is not None and s not in source_ids:continue
  if s in owned and a['id'] not in heads.get(s,set()):continue
  result.append(a)
 return sorted(result,key=lambda a:(es[a['entry']]['source'].encode(),q.encode(),oid(a['occurrence']).encode()))
def availability(d):
 m=d['metadata'];loaded={ck(c['identity']) for c in d['captures']};answer=[]
 for s in sorted(d['registry']['sources'],key=lambda s:s['id'].encode()):
  co=[r for r in m['collections'] if r['source']==s['id']];cop={r['id'] for r in co}
  comp=[r for r in m['components'] if r['collection'] in cop];compids={r['id'] for r in comp}
  rel=[r for r in m['releases'] if r['component'] in compids]; active=[r['id'] for r in rel if r['selected'] and r['availability']=='Available']
  head=next((h for h in m['owned_heads'] if h['source']==s['id']),None)
  reason='NoReleases' if not rel else 'NoSelectedAvailableRelease' if not active else 'NoOwnedHead' if s['origin']=='OwnedDraft' and head is None else None
  caps=[]
  for r in rel:
   for c in r['captures']:
    if c not in caps:caps.append(c)
  answer.append(dict(source=s,aliases=[r for r in d['registry']['aliases'] if r['source']==s['id']],lineage=[r for r in d['registry']['lineage'] if r['source']==s['id']],collections=co,components=comp,releases=rel,selected_available_release_ids=active,loaded_captures=[c for c in caps if ck(c) in loaded],missing_captures=[c for c in caps if ck(c) not in loaded],owned_head=head,selectable=reason is None,reason=reason))
 return answer
def hit(d,a):
 m=d['metadata'];r=d['registry']; e=next(e for e in m['entries'] if e['id']==a['entry']);o=a['occurrence']
 c=next(c for c in d['captures'] if c['identity']==o['capture']);raw=next(x for x in c['occurrences'] if x['locator']==o['locator'])
 snap=next(x for x in r['snapshots'] if x['snapshot']==o['capture']); att=next((x for x in r['occurrences'] if oid(x['occurrence'])==oid(o)),None)
 forms=[x for x in m['forms'] if x['entry']==e['id']]; senses=[x for x in m['editorial_senses'] if a['id'] in x['assertions']]
 local=[{'Entry':e['id']},{'Assertion':a['id']}]+[{'Form':x['id']} for x in forms]+[{'EditorialSense':x['id']} for x in senses]
 related=dict(contexts=[x for x in m['contexts'] if x['assertion']==a['id']],frames=[x for x in m['frames'] if x['assertion']==a['id']],forms=forms,etymologies=[x for x in m['etymologies'] if x['assertion']==a['id']],relations=[x for x in m['relations'] if x['origin'] in local],editorial_senses=senses,annotations=[x for x in m['annotations'] if x['target'] in local])
 compids={x['computation'] for x in forms if x['computation'] is not None}|{x['computation'] for x in related['annotations']}
 related['computations']=[x for x in m['computations'] if x['id'] in compids]
 relids=[x['id'] for x in m['releases'] if a['id'] in x['assertions'] and x['selected'] and x['availability']=='Available']
 own={a['id'],e['id']}
 for values in related.values():own.update(x['id'] for x in values)
 for av in availability(d):
  if av['source']['id']==e['source']:
   own.update(x['id'] for x in av['collections']+av['components']);own.update(av['selected_available_release_ids'])
 related['extensions']=[x for x in m['extensions'] if x['owner'] in own]
 return dict(occurrence_id=oid(o),entry=e,assertion=a,capture=c['metadata'],raw=raw,snapshot_attribution=snap,occurrence_attribution=att,contributor=next((x for x in r['contributors'] if att and x['id']==att['contributor']),None),distributor=next((x for x in r['distributors'] if x['id']==snap['distributor']),None),release_ids=relids,owned_revision_ids=[x['id'] for x in m['owned_revisions'] if a['id'] in x['assertions']],tombstoned_by=[x['id'] for x in m['owned_revisions'] if a['id'] in x['tombstones']],**related)
def reference_view(d,a):
 m=d['metadata'];o=a['occurrence'];e=next(e for e in m['entries'] if e['id']==a['entry']);c=next((c for c in d['captures'] if c['identity']==o['capture']),None);raw=next((x for x in c['occurrences'] if x['locator']==o['locator']),None) if c else None
 current=a in pick(d,e['spelling'].get('Value','')) if isinstance(e['spelling'],dict) else False
 return dict(entry=e,assertion=a,occurrence_id=oid(o),occurrence_attribution=next((x for x in d['registry']['occurrences'] if oid(x['occurrence'])==oid(o)),None),raw=raw,capture_metadata=c['metadata'] if c and raw else None,payload_present=raw is not None,current_eligible=current,release_ids=[r['id'] for r in m['releases'] if a['id'] in r['assertions']],owned_revision_ids=[r['id'] for r in m['owned_revisions'] if a['id'] in r['assertions']],tombstoned_by=[r['id'] for r in m['owned_revisions'] if a['id'] in r['tombstones']])
def diffs(actual,expected,path='$'):
 if type(actual)!=type(expected):return [f'{path}: type {type(actual).__name__} != {type(expected).__name__}']
 if isinstance(expected,dict):
  result=[]
  if set(actual)!=set(expected):result.append(f'{path}: keys {sorted(set(actual)^set(expected))}')
  for key in expected.keys()&actual.keys():result+=diffs(actual[key],expected[key],path+'.'+key)
  return result
 if isinstance(expected,list):
  result=[] if len(actual)==len(expected) else [f'{path}: len {len(actual)} != {len(expected)}']
  for i,(x,y) in enumerate(zip(actual,expected)):result+=diffs(x,y,f'{path}[{i}]')
  return result
 return [] if actual==expected else [f'{path}: {actual!r} != {expected!r}']
def resolved(d,target):
 m=d['metadata']; entity=None; release=None; yes=[];maybe=[]
 if target=='Unknown':status='Unknown'
 elif isinstance(target,dict) and 'Unresolved' in target:status='Unresolved'
 elif 'Local' in target:
  role,ident=next(iter(target['Local'].items()));table={'Entry':'entries','Assertion':'assertions','Form':'forms','EditorialSense':'editorial_senses'}[role]
  row=next(x for x in m[table] if x['id']==ident);entity={role:row}
  if role=='Assertion':yes=[row]
  elif role=='EditorialSense':yes=[x for x in m['assertions'] if x['id'] in row['assertions']]
  else:yes=[x for x in m['assertions'] if x['entry']==(ident if role=='Entry' else row['entry'])]
  status='Resolved'
 else:
  t=target['Catalog']; release=next(r for r in m['releases'] if r['id']==t['release']);base=[a for a in m['assertions'] if a['id'] in release['assertions'] and (t['occurrence'] is None or a['occurrence']==t['occurrence'])]
  mismatch=0
  for a in base:
   at=next((r for r in d['registry']['occurrences'] if oid(r['occurrence'])==oid(a['occurrence'])),{})
   fields=[(t[k],at.get(k)) for k in ['native_entry_id','native_assertion_id'] if t[k] is not None]
   if any(actual is not None and wanted!=actual for wanted,actual in fields):mismatch+=1;continue
   (maybe if any(actual is None for wanted,actual in fields) else yes).append(a)
  status='Resolved' if yes else 'Unverified' if maybe else 'NoMatch' if mismatch or release['availability']=='Available' else 'CatalogueOnly'
 entries={e['id']:e for e in m['entries']}
 def order(a):
  e=entries[a['entry']];sp=e['spelling'].get('Value','') if isinstance(e['spelling'],dict) else ''
  return e['source'].encode(),sp.encode(),oid(a['occurrence']).encode()
 return dict(version=1,target=target,status=status,entity=entity,confirmed=[reference_view(d,a) for a in sorted(yes,key=order)],unverified=[reference_view(d,a) for a in sorted(maybe,key=order)],availability=availability(d),requested_release=release)
