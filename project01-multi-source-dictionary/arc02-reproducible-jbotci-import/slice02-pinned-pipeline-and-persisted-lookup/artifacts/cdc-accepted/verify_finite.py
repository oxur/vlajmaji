"""Independent finite public-result checks derived from the pinned raw rows."""
from pathlib import Path
import hashlib,json,runpy,struct,sys
rawp,outp=sys.argv[1:3];raw=Path(rawp).read_bytes();digest=hashlib.sha256(raw).hexdigest();out=Path(outp)
tok=runpy.run_path(str(Path(__file__).parent/'verify_normalized.py'));rows=tok['raw_array'](raw.decode());checks=0

def need(ok,where):
 global checks
 checks+=1
 assert ok,where

def load(name):
 v=json.loads((out/(name+'.json')).read_bytes());need(set(v)=={'ok'},name+' success');return v['ok']

def tupleid(loc):
 return b''.join(struct.pack('>Q',len(x.encode()))+x.encode() for x in ['captured:lensisku:jbotci:en',digest,loc]).hex()

def check_hit(h):
 i=int(h['raw']['locator'][1:]);row,lex=rows[i];fields=tok['raw_object'](lex);native={k:v for k,v,_ in fields}
 need(h['raw']['raw_line'] is None,'no fabricated raw line '+str(i))
 need(h['raw']['fields']==[{'name':k,'encoding':'Json','raw_value':v} for k,v,_ in fields],'ordered raw fields '+str(i))
 need(h['occurrence_id']==tupleid('/'+str(i)),'tuple identity '+str(i))
 need(h['entry']['id']==f'jbotci-{digest}-entry-{i}','entry id '+str(i))
 need(h['entry']['spelling']=={'Value':row['word']} and h['entry']['native_word_class']=={'Value':row['word_type']},'entry '+str(i))
 need(h['assertion']['id']==f'jbotci-{digest}-assertion-{i}','assertion id '+str(i))
 need(h['assertion']['definition']=={'Value':{'text':row['definition'],'language':{'Value':'en'},'markup':'Missing'}},'definition '+str(i))
 need(h['assertion']['source_score']=={'Value':native['score']},'score lexeme '+str(i))
 need(h['occurrence_attribution']['native_assertion_id']==native['definition_id'],'native id '+str(i))
 return i

all_expected={w:[i for i,(r,_) in enumerate(rows) if r['word']==w and r['word_type']!='wiki' and r['definition'].strip()] for w in ['farvi','ma','Periodic-table gismu assignment algorithm']}
responses={}
for name,word in [('farvi-default','farvi'),('farvi-abbreviation','farvi'),('ma','ma'),('wiki-excluded','Periodic-table gismu assignment algorithm')]:
 v=load(name);responses[name]=v;expected=all_expected[word];need(v['totals']=={'occurrences':len(expected),'distinct_spellings':int(bool(expected)),'source_groups':int(bool(expected))},name+' totals')
 need(v['native_candidate_occurrences']==len(expected),name+' SQL candidates')
 need(v['resolved_sources']==['lensisku'],name+' source selector')
 need(v['policy']=='exact-entry-v1' and v['store_schema_version']==1 and v['catalogue_revision']==0,name+' policy/versions')
 hits=[h for g in v['groups'] for h in g['hits']];need(sorted(check_hit(h) for h in hits)==expected,name+' all source occurrences')
 for g in v['groups']:
  need(g['source']['id']=='lensisku' and g['source']['abbreviation']=='LSK',name+' source group')
  need(g['full_count']==len(expected) and g['returned_count']==len(expected) and not g['partial'],name+' unpaged group')
need(responses['farvi-default']==responses['farvi-abbreviation'],'complete default/abbreviation equality')
page_indices=[]
for name in ['page-first','page-next']:
 v=load(name);need(v['totals']==responses['farvi-default']['totals'],name+' full totals');hits=[h for g in v['groups'] for h in g['hits']];need(len(hits)==1,name+' one returned');page_indices.extend(check_hit(h) for h in hits)
 need(v['groups'][0]['full_count']==2 and v['groups'][0]['returned_count']==1 and v['groups'][0]['partial'],name+' partial group')
 need(bool(v['next_cursor'])==(name=='page-first'),name+' cursor ending')
need(sorted(page_indices)==all_expected['farvi'],'complete two-page occurrence union')
for name in ['resolve-local','catalog-native']:
 v=load(name);need(v['status']=='Resolved',name+' status');need(len(v['confirmed'])==1 and v['unverified']==[],name+' confidence')
 h=v['confirmed'][0];need(h['payload_present'] and h['current_eligible'],name+' availability');need(check_hit(h)==13306,name+' exact raw/native target')
 need(h['occurrence_attribution']['native_assertion_id']=='333',name+' native identity')
cases=8
if (out/'farvi-source-id.json').exists():
 need(load('farvi-source-id')==responses['farvi-default'],'complete explicit source-id counterpart')
 for name,word in [('cpana','cpana'),('sie',"si'e"),('unknown-spelling','__a02_nonexistent__'),('undefined-excluded',"sekoi'ai")]:
  v=load(name);expected=[i for i,(r,_) in enumerate(rows) if r['word']==word and r['word_type']!='wiki' and r['definition'].strip()]
  need(v['totals']=={'occurrences':len(expected),'distinct_spellings':int(bool(expected)),'source_groups':int(bool(expected))},name+' totals')
  need(v['native_candidate_occurrences']==len(expected),name+' candidates')
  hits=[h for g in v['groups'] for h in g['hits']];need(sorted(check_hit(h) for h in hits)==expected,name+' exact raw-source population')
 for name,code in [('unknown-selector','unknown_source'),('bad-cursor','malformed_cursor'),('resolve-empty','invalid_reference'),('catalog-bad-release','invalid_reference'),('cursor-other-spelling','stale_cursor')]:
  value=json.loads((out/(name+'.json')).read_bytes());need(set(value)=={'error'} and value['error']['code']==code,name+' exact error category')
 zero=load('zero-page');need(zero['totals']==responses['farvi-default']['totals'] and zero['groups']==[] and zero['next_cursor'] is None,'zero page retains full counts without results/cursor')
 for name,status in [('resolve-unknown','Unknown'),('resolve-unresolved','Unresolved'),('catalog-mismatch','NoMatch')]:
  v=load(name);need(v['status']==status and v['confirmed']==[] and v['unverified']==[],name+' exact status/empty payloads')
 wiki=load('catalog-excluded-wiki');need(wiki['status']=='Resolved' and len(wiki['confirmed'])==1 and wiki['unverified']==[],'excluded wiki catalog raw resolution')
 view=wiki['confirmed'][0];need(view['payload_present'] and not view['current_eligible'],'excluded wiki retained payload and explicit ineligibility');need(check_hit(view)==22040,'excluded wiki exact source row')
 cases=23
print(json.dumps({'verdict':'PASS','cases':cases,'checks':checks,'raw_sha256':digest,'source_farvi_ordinals':all_expected['farvi'],'source_ma_ordinals':all_expected['ma'],'wiki_exact_eligible':all_expected['Periodic-table gismu assignment algorithm']},indent=2))
