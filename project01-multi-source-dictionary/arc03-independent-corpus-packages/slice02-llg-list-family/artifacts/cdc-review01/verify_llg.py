"""CDC raw-derived complete LLG oracle, prepared before candidate output exists."""
from pathlib import Path
from collections import Counter,defaultdict
import json,hashlib,sys
INPUT=Path('/tmp/vlajmaji-p01-a03-cdc/readiness')
checks=0
def ck(ok,what):
 global checks
 checks+=1
 if not ok:raise AssertionError(what)
def value(x):return x['Value'] if isinstance(x,dict) and 'Value' in x else None
def text(x):
 v=value(x);return v['text'] if v is not None else None
def occ_key(o):return(o['capture']['namespace'],o['capture']['sha256'],o['locator'])
def evidence(e):return(occ_key(e['occurrence']),e['field_index'])
def raw_fields(component,line,ordinal):
 if component!='rafsi' and ordinal==1:return[('$line',line)]
 if component=='gismu':
  spec=[('word',1,6),('assigned_rafsi_cvc',7,10),('assigned_rafsi_ccv',11,14),('assigned_rafsi_cvv',15,19),('keyword',20,40),('hint',41,61),('definition',62,158),('teaching_code',159,161),('frequency_1993',161,165),('notes',165,None)]
  return[(n,line[a:b]) for n,a,b in spec if len(line)>a]
 if component=='cmavo':return[(n,line[a:b]) for n,a,b in [('word',0,10),('selmaho',10,20),('keyword',20,61),('definition',61,None)]]
 # Keep original separator prefix on the native gloss remainder, per issued contract.
 import re
 m=re.match(r'\s*(\S+)\s+(\S+)(.*)$',line);ck(m is not None,'rafsi grammar');return[('rafsi',m[1]),('word',m[2]),('gloss',m[3])]
def main():
 root=Path(sys.argv[1]);d=json.loads((root/'release.json').read_bytes());report=json.loads((root/'report.json').read_bytes());seal=json.loads((root/'seal.json').read_bytes());m=d['metadata'];registry=d['registry'];entries={e['id']:e for e in m['entries']};assertions={occ_key(a['occurrence']):a for a in m['assertions']};byword={value(e['spelling']):e for e in entries.values() if value(e['spelling']) is not None};
 ck(len(byword)==2433,'distinct heads');ck(len(entries)==2435,'entries with two headers');ck(len(m['assertions'])==4074,'all physical assertions');ck(len(assertions)==4074,'unique occurrence-backed assertions');ck(len(d['captures'])==4,'four captures');ck(len(m['releases'])==3,'three releases');ck(len(m['collections'])==1 and len(m['components'])==3,'collection/components');ck(len(registry['sources'])==1 and registry['sources'][0]['id']=='llg' and registry['sources'][0]['display_name']=='The Logical Language Group, Inc.','exact source');ck(not registry['contributors'] and not registry['distributors'] and not registry['lineage'],'no invented provenance')
 all_contexts=[];all_forms=[];all_extensions=[];all_fields=0;class_by_word={};noteb=(INPUT/'llg-current-public_domain.txt').read_bytes();expected_assertions=set();snapshots=set()
 for component in ['gismu','cmavo','rafsi']:
  raw=(INPUT/('llg-current-'+component+'.txt')).read_bytes();sha=hashlib.sha256(raw).hexdigest();ns='captured:llg:'+component;captures=[c for c in d['captures'] if c['identity']=={'namespace':ns,'sha256':sha}];ck(len(captures)==1,'exact capture identity');c=captures[0];ck(bytes(c['bytes'])==raw,'all raw bytes');ck(c['metadata']['revision']=='Missing' and c['metadata']['captured_at']=='Missing','unknown source clock/revision');snapshots.add((ns,sha));lines=raw.decode().splitlines();ck(len(c['occurrences'])==len(lines),'every physical line')
  for ordinal,(line,o) in enumerate(zip(lines,c['occurrences']),1):
   key=(ns,sha,str(ordinal));expected_assertions.add(key);ck(o['locator']==str(ordinal),'one-based locator');ck(o['raw_line']==line,'exact native source line');fs=raw_fields(component,line,ordinal);all_fields+=len(fs);ck(o['fields']==[{'name':n,'encoding':'Text','raw_value':v} for n,v in fs],'all ordered native fields');a=assertions[key];e=entries[a['entry']];ck(e['source']=='llg','source-owned entry');ck(e['native_word_class']=='Missing' and e['mapped_pos']=='Missing' and e['script']=='Missing','no filename POS/class');ck(a['native_sense_id']=='Missing' and a['source_score']=='Missing' and not a['timestamps'],'no inferred sense/score/time')
   if component!='rafsi' and ordinal==1:
    ck(a['definition']=='Missing' and isinstance(a['eligibility'],dict) and 'Excluded' in a['eligibility'],'header excluded');ck(e['spelling']=='Missing','header no invented word');continue
   f=dict(fs);head=f['word'].strip();ck(value(e['spelling'])==head and byword[head]['id']==e['id'],'exact shared complete word');ck(value(e['language'])=='jbo','entry language');ck(a['language']==('Missing' if component=='rafsi' else {'Value':'en'}),'definition-bearing assertion language');ck(e['id']==byword[head]['id'],'head identity');
   if component=='rafsi':ck(a['definition']=='Missing' and 'Excluded' in a['eligibility'],'form only excluded')
   else:ck(a['eligibility']=='Lexical' and text(a['definition'])==f['definition'].strip(),'literal definition lexical')
   if component=='cmavo':class_by_word.setdefault(head,set()).add(f['selmaho'].strip())
   for index,(name,v) in enumerate(fs):
    trimmed=v.strip();ev=(key,index)
    if name in ['keyword','hint','notes','gloss'] and trimmed:
     all_contexts.append((a['id'],'Note' if name in ['hint','notes'] else 'Gloss',trimmed,ev,name))
    if (name.startswith('assigned_rafsi_') or component=='rafsi' and name=='rafsi') and trimmed:all_forms.append((e['id'],trimmed,'AssignedRafsi',ev))
    if name in ['teaching_code','frequency_1993']:all_extensions.append((a['id'],'llg.'+name,json.dumps(v,ensure_ascii=False),ev))
 ck(set(assertions)==expected_assertions,'no extra occurrences');ck(all_fields==23298,'full native denominator')
 for head,e in byword.items():ck(e['selmaho']==({'Value':next(iter(class_by_word[head]))} if head in class_by_word and len(class_by_word[head])==1 else 'Missing'),'native selmaho agreement')
 notice=[c for c in d['captures'] if c['identity']['namespace']=='captured:llg:notice'];ck(len(notice)==1 and bytes(notice[0]['bytes'])==noteb and not notice[0]['occurrences'],'exact shared notice bytes');snapshots.add(('captured:llg:notice',hashlib.sha256(noteb).hexdigest()));ck(len(registry['snapshots'])==4,'snapshot attributions');ck(all(x['source']=='llg' and x['distributor'] is None for x in registry['snapshots']),'snapshot source');ck(len(registry['occurrences'])==4074,'native attribution count')
 for release in m['releases']:
  ck(release['selected'] and release['availability']=='Available' and release['redistribution']=='Held','selected held release');ck(release['upstream_revision']=='Missing' and release['distributor_revision']=='Missing' and release['captured_at']=='Missing','release unknown clocks');ck(release['content_notices'] and any(text0.get('text',{}).get('Value')==noteb.decode() for text0 in release['content_notices']),'exact native notice on each release');ck(len(release['captures'])==2,'component plus notice')
 # Compare every typed record with independent source references, ignoring generated IDs/order.
 actual_contexts=[]
 for x in m['contexts']:
  ev=evidence(x['evidence']);v=text(x['value']);source=next(y for y in all_contexts if y[0]==x['assertion'] and y[3]==ev)
  if source[4]=='hint':ck(v.endswith(source[2]) and 'hint' in v[:-len(source[2])].lower(),'hint is explicitly labelled');v=source[2]
  actual_contexts.append((x['assertion'],x['kind'],v,ev,source[4]))
 ck(Counter(actual_contexts)==Counter(all_contexts) and len(all_contexts)==5804,'complete context multiset')
 ck(Counter((f['entry'],f['spelling'],f['kind'],evidence(f['evidence'])) for f in m['forms'])==Counter(all_forms) and len(all_forms)==3091,'complete assigned forms/evidence');ck(all(value(f['standing']) and f['computation'] is None for f in m['forms']),'native standing/no computation')
 ck(Counter((x['owner'],x['name'],json.dumps(json.loads(x['value']),ensure_ascii=False),evidence(x['evidence'])) for x in m['extensions'])==Counter(all_extensions) and len(all_extensions)==2865,'all native historical extensions')
 for field in ['frames','etymologies','relations','owned_revisions','owned_heads','editorial_senses','computations','annotations']:ck(not m[field],'no inference '+field)
 ck(sum(a['eligibility']=='Lexical' for a in m['assertions'])==2527,'lexical denominator');ck(sum(isinstance(a['eligibility'],dict) and 'Excluded' in a['eligibility'] for a in m['assertions'])==1547,'excluded denominator');raw=(root/'release.json').read_bytes();ck(seal['payload_kind']=='full' and seal['pack_id']=='llg','Full seal');ck(seal['release_bytes']==len(raw) and seal['release_sha256']==hashlib.sha256(raw).hexdigest(),'seal exact canonical');print(json.dumps({'status':'PASS','checks':checks,'assertions':4074,'fields':all_fields,'entries':2435,'contexts':len(all_contexts),'forms':len(all_forms),'extensions':len(all_extensions),'canonical_bytes':len(raw),'canonical_sha256':hashlib.sha256(raw).hexdigest()}))
if __name__=='__main__':main()
