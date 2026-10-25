from pathlib import Path
import json,collections,hashlib,sys
from raw_reference import ROOT,LEAF,CLASSES,load_source,sha
checks=collections.Counter()
def need(condition,label):
 checks[label]+=1
 if not condition:raise AssertionError(label)
def main():
 rb,rows,fields,lexemes=load_source();digest=sha(rb);mb=(LEAF/'dictionary-en.metadata.toml').read_bytes()
 pack=Path(sys.argv[1]);b=(pack/'release.json').read_bytes();need(len(b)==215412497 and sha(b)=='0987ec84a8434573b5979ea854ca459ce2ea9c7fa03ebac6639574dd2b7aa460','canonical_identity')
 model=json.loads(b);report=json.loads((pack/'report.json').read_text());meta=model['metadata'];reg=model['registry'];caps=model['captures']
 need(bytes(caps[0]['bytes'])==rb and bytes(caps[1]['bytes'])==mb,'exact_capture_bytes')
 need(len(caps[1]['occurrences'])==0,'metadata_not_lexical')
 need((len(meta['assertions']),len(meta['entries']),len(reg['occurrences']))==(len(rows),)*3,'complete_normalized_population')
 eligible=collections.Counter();fcounts=collections.Counter();classes=collections.Counter();ids=set();expected_context=[];expected_forms=[];expected_extensions=[];expected_ety=[];expected_groups=collections.defaultdict(list)
 rootid=f'jbotci-{digest}'
 for i,(row,fs,cap,entry,assertion,attr) in enumerate(zip(rows,fields,caps[0]['occurrences'],meta['entries'],meta['assertions'],reg['occurrences'])):
  loc=f'/{i}';a=f'{rootid}-assertion-{i}';e=f'{rootid}-entry-{i}';native={k:v for k,v,_ in fs};fi={k:j for j,(k,_,_) in enumerate(fs)}
  need(cap['locator']==loc and cap['fields']==[{'name':k,'encoding':'Json','raw_value':v} for k,v,_ in fs],'ordered_native_fields')
  need(entry['id']==e and assertion['id']==a and assertion['entry']==e and assertion['occurrence']['locator']==loc,'per_row_identity')
  need(attr['native_assertion_id']==native['definition_id'] and attr['native_entry_id'] is None,'native_identity')
  need(assertion['source_score']=={'Value':native['score']} and assertion['definition']['Value']['text']==row['definition'],'exact_score_definition')
  yes=row['word_type'] in CLASSES and bool(row['word']) and bool(row['definition'].strip())
  need((assertion['eligibility']=='Lexical')==yes,'explicit_eligibility')
  if yes:eligible[row['word']]+=1;expected_groups[row['word']].append((a,native['definition_id']))
  fcounts.update(k for k,_,_ in fs);classes[row['word_type']]+=1;ids.add(row['definition_id'])
  for k,kind in [('notes','Note'),('jargon','Domain')]:
   if k in row:expected_context.append((f'{rootid}-context-{i}-{k}',a,kind,row[k],str(fi[k])))
  if 'gloss_keywords' in row:
   expected_extensions.append((a,'lensisku.gloss_keywords.raw',native['gloss_keywords'],loc,str(fi['gloss_keywords'])))
   d=json.JSONDecoder();s=native['gloss_keywords'];p=1
   for j,g in enumerate(row['gloss_keywords']):
    while s[p].isspace():p+=1
    start=p;obj,p=d.raw_decode(s,p);need(obj==g,'gloss_raw_pair')
    c=f'{rootid}-context-gloss-{i}-{j}';expected_context.append((c,a,'Gloss',g['word'],str(fi['gloss_keywords'])))
    expected_extensions.append((c,'lensisku.gloss_keywords.object',s[start:p],loc,str(fi['gloss_keywords'])))
    while s[p].isspace():p+=1
    if s[p]==',':p+=1
  if 'place_keywords' in row:expected_extensions.append((a,'lensisku.place_keywords',native['place_keywords'],loc,str(fi['place_keywords'])))
  for key in ('rafsi','experimental_rafsi'):
   for j,form in enumerate(row.get(key,'').split()):expected_forms.append((f'{rootid}-form-{i}-{key}-{j}',e,form,'SourceReportedForm',key,str(fi[key])))
  if 'etymology' in row:expected_ety.append((f'{rootid}-etymology-{i}',a,row['etymology'],str(fi['etymology'])))
 need(report['field_counts']==dict(fcounts) and report['word_types']==dict(classes),'report_categories')
 need((report['input_rows'],report['normalized_assertions'],report['lexical_occurrences'],report['excluded_occurrences'],report['eligible_spellings'])==(len(rows),len(rows),sum(eligible.values()),len(rows)-sum(eligible.values()),len(eligible)),'balanced_report')
 need(report['diagnostics']=={'invalid_definition':9,'unsupported_word_type':1},'truthful_diagnostics')
 release=meta['releases'][0]
 need(release['redistribution']=='Held' and meta['code_notices'][0]['locator']=={'Value':'LICENSE'},'terms_hold')
 need(all(meta[k]==[] for k in ['frames','relations','owned_revisions','owned_heads','editorial_senses','computations','annotations']),'noninference')
 expected_extensions.append((release['id'],'jbotci.metadata.raw_text',json.dumps(mb.decode(),ensure_ascii=False),None,None))
 actual_context=[(x['id'],x['assertion'],x['kind'],x['value']['Value']['text'],str(x['evidence']['field_index'])) for x in meta['contexts']]
 need(actual_context==expected_context,'all_context_ids_owners_order_values')
 actual_forms=[(x['id'],x['entry'],x['spelling'],x['kind'],x['standing']['Value'],str(x['evidence']['field_index'])) for x in meta['forms']]
 need(actual_forms==expected_forms,'all_form_ids_owners_order_values')
 actual_ety=[(x['id'],x['assertion'],x['statement']['Value']['text'],str(x['evidence']['field_index'])) for x in meta['etymologies']]
 need(actual_ety==expected_ety,'all_etymology_ids_owners_order_values')
 actual_ext=[(x['owner'],x['name'],x['value'],x['evidence']['occurrence']['locator'] if x['evidence'] else None,str(x['evidence']['field_index']) if x['evidence'] else None) for x in meta['extensions']]
 need(actual_ext==expected_extensions,'all_exact_extension_owners_order_lexemes')
 seal=json.loads((pack/'seal.json').read_bytes())
 need(seal['payload_kind']=='full' and seal['pack_id']=='lensisku-jbotci' and seal['release_sha256']==sha(b) and seal['release_bytes']==len(b),'full_integrity_seal')
 need(seal['source_ids']==['lensisku'] and seal['release_ids']==[release['id']] and seal['captures']==[c['identity'] for c in caps],'seal_ordered_inventory')
 out={'verdict':'PASS','checks':dict(checks),'total_checks':sum(checks.values()),'raw_rows':len(rows),'entries':len(meta['entries']),'assertions':len(meta['assertions']),'native_fields':sum(fcounts.values()),'eligible_occurrences':sum(eligible.values()),'eligible_spellings':len(eligible),'all_spellings':len({r['word'] for r in rows}),'ordered_contexts':len(expected_context),'ordered_forms':len(expected_forms),'ordered_etymologies':len(expected_ety),'ordered_extensions':len(expected_extensions),'canonical_sha256':sha(b),'canonical_bytes':len(b),'raw_sha256':digest,'metadata_sha256':sha(mb),'pack_path':str(pack)}
 (ROOT/'full-native-result01.json').write_text(json.dumps(out,indent=2)+'\n');print(json.dumps(out,indent=2))
if __name__=='__main__':main()
