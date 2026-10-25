from pathlib import Path
import json,collections,hashlib,sys
import importlib.util
spec=importlib.util.spec_from_file_location("native_source", "/tmp/vlajmaji-p01-a03-cdc/readiness/distribution_census.py");source=importlib.util.module_from_spec(spec);spec.loader.exec_module(source)
ROOT=Path('/tmp/vlajmaji-p01-a03-cdc/s03');CLASSES=source.classes
sha=lambda b:hashlib.sha256(b).hexdigest()
def load_source():
 b,rows,fs,raw=source.la_source();return b,rows,[[(k,lex,v) for k,lex,v,keylex in f] for f in fs],raw

checks=collections.Counter()
def need(condition,label):
 checks[label]+=1
 if not condition:raise AssertionError(label)
def main():
 rb,rows,fields,lexemes=load_source();digest=sha(rb)
 pack=Path(sys.argv[1]);b=(pack/'release.json').read_bytes();need(sha(rb)=='861e71f66173b383298f6b1823e3ee1ed76db4b9c3fb97e7177de1fd7b667d35','raw_pin_identity')
 model=json.loads(b);report=json.loads((pack/'report.json').read_text());meta=model['metadata'];reg=model['registry'];caps=model['captures']
 need(len(caps)==1 and bytes(caps[0]['bytes'])==rb,'exact_capture_bytes')
 need(caps[0]['identity']=={'namespace':'captured:lensisku:la-lojban:en','sha256':digest},'capture_identity')
 need(caps[0]['metadata']['revision']=={'Value':'de47055c68141c283841b1035cd7425bb32bf6ca'} and caps[0]['metadata']['captured_at']=='Missing','capture_provenance')
 need((len(meta['assertions']),len(meta['entries']),len(reg['occurrences']),len(caps[0]['occurrences']))==(len(rows),)*4,'complete_normalized_population')
 eligible=collections.Counter();fcounts=collections.Counter();classes=collections.Counter();ids=set();expected_context=[];expected_forms=[];expected_extensions=[];expected_ety=[];expected_groups=collections.defaultdict(list)
 rootid=f'la-lojban-{digest}'
 for i,(row,fs,cap,entry,assertion,attr) in enumerate(zip(rows,fields,caps[0]['occurrences'],meta['entries'],meta['assertions'],reg['occurrences'])):
  loc=f'/{i}';a=f'{rootid}-assertion-{i}';e=f'{rootid}-entry-{i}';native={k:v for k,v,_ in fs};fi={k:j for j,(k,_,_) in enumerate(fs)}
  need(cap['raw_line'] is None,'structured_row_not_fabricated_line')
  need(cap['locator']==loc and cap['fields']==[{'name':k,'encoding':'Json','raw_value':v} for k,v,_ in fs],'ordered_native_fields')
  need(entry['id']==e and assertion['id']==a and assertion['entry']==e and assertion['occurrence']['locator']==loc,'per_row_identity')
  need(attr['native_assertion_id']==native['definition_id'] and attr['native_entry_id'] is None,'native_identity')
  need(attr['contributor']=='lensisku-user-'+sha(row['user']['username'].encode()),'source_scoped_contributor')
  need(attr['occurrence']==assertion['occurrence'] and assertion['occurrence']['capture']==caps[0]['identity'],'complete_occurrence_reference')
  need(entry['source']=='lensisku' and entry['spelling']=={'Value':row['word']} and entry['native_word_class']=={'Value':row['word_type']},'native_entry_values')
  need(entry['language']=={'Value':'jbo'} and entry['script']=='Missing' and entry['mapped_pos']=='Missing','noninvented_entry_metadata')
  need(entry['selmaho']==({'Value':row['selmaho']} if 'selmaho' in row else 'Missing'),'selmaho_absence')
  need(assertion['language']=={'Value':'en'} and assertion['native_sense_id']=='Missing' and not assertion['timestamps'],'assertion_metadata')
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
 need(report['diagnostics']=={},'truthful_diagnostics')
 release=meta['releases'][0]
 need(release['redistribution']=='Held' and meta['code_notices'][0]['locator']=={'Value':'LICENSE'},'terms_hold')
 need(all(meta[k]==[] for k in ['frames','relations','owned_revisions','owned_heads','editorial_senses','computations','annotations']),'noninference')
 need(release['upstream_revision']=='Missing' and release['distributor_revision']=={'Value':'de47055c68141c283841b1035cd7425bb32bf6ca'},'release_provenance')
 need(release['content_notices'][0]['text']=='Missing' and release['content_notices'][0]['license']=='Missing','no_software_as_data_grant')
 need(reg['sources']==[{'id':'lensisku','display_name':'Lensisku','abbreviation':'LSK','abbreviation_kind':'ProjectAssigned','abbreviation_evidence':None,'origin':'Upstream'}],'shared_source_authority')
 need(reg['distributors']==[{'id':'la-lojban','display_name':'La-Lojban'}] and not reg['lineage'],'truthful_distribution_lineage')
 need(len(reg['contributors'])==211 and len(reg['snapshots'])==1,'contributor_snapshot_population')
 need({x['id']:x['display_name'] for x in reg['contributors']}=={'lensisku-user-'+sha(x['user']['username'].encode()):x['user']['username'] for x in rows},'exact_contributor_bodies')
 need(release['accounting']=={'input_occurrences':17056,'normalized_occurrences':17056,'query_excluded_occurrences':0,'unit':'source-array-row'},'release_accounting')
 need(release['distributor']=='la-lojban' and release['selected'] and release['availability']=='Available' and release['normalizer']=='vlajmaji-la-lojban-export' and release['normalizer_version']=='1','release_semantics')
 need(not release['diagnostics'],'full_native_diagnostics')
 expected_release='la-lojban-release-'+sha(('vlajmaji-la-lojban-release-v1\0de47055c68141c283841b1035cd7425bb32bf6ca\0'+digest).encode())
 need(release['id']==expected_release and release['captures']==[caps[0]['identity']],'revision_based_release_identity')
 need(all(x['source']=='lensisku' and x['distributor']=='la-lojban' for x in reg['snapshots']),'snapshot_attribution')
 for group in ['contexts','forms','etymologies']:
  for x in meta[group]:
   ev=x['evidence'];need(ev is not None and ev['occurrence']['capture']==caps[0]['identity'],'typed_evidence_capture')
   owner=x.get('assertion') or x.get('entry');ordinal=int(owner.rsplit('-',1)[1]);need(ev['occurrence']['locator']==f'/{ordinal}','typed_evidence_ordinal')
 need(all(x['language']=={'Value':'jbo'} and x['script']=='Missing' and x['pronunciation']=='Missing' and not x['components'] and x['computation'] is None for x in meta['forms']),'form_noninference')
 need(all(x['value']['Value']['language']=={'Value':'en'} for x in meta['contexts']),'context_language')
 need(all(x['statement']['Value']['language']=={'Value':'en'} and x['native_kind']=='Missing' and not x['components'] for x in meta['etymologies']),'etymology_noninference')
 actual_context=[(x['id'],x['assertion'],x['kind'],x['value']['Value']['text'],str(x['evidence']['field_index'])) for x in meta['contexts']]
 need(actual_context==expected_context,'all_context_ids_owners_order_values')
 actual_forms=[(x['id'],x['entry'],x['spelling'],x['kind'],x['standing']['Value'],str(x['evidence']['field_index'])) for x in meta['forms']]
 need(actual_forms==expected_forms,'all_form_ids_owners_order_values')
 actual_ety=[(x['id'],x['assertion'],x['statement']['Value']['text'],str(x['evidence']['field_index'])) for x in meta['etymologies']]
 need(actual_ety==expected_ety,'all_etymology_ids_owners_order_values')
 actual_ext=[(x['owner'],x['name'],x['value'],x['evidence']['occurrence']['locator'] if x['evidence'] else None,str(x['evidence']['field_index']) if x['evidence'] else None) for x in meta['extensions']]
 need(actual_ext==expected_extensions,'all_exact_extension_owners_order_lexemes')
 seal=json.loads((pack/'seal.json').read_bytes())
 need(seal['payload_kind']=='full' and seal['pack_id']=='la-lojban' and seal['release_sha256']==sha(b) and seal['release_bytes']==len(b),'full_integrity_seal')
 need(seal['source_ids']==['lensisku'] and seal['release_ids']==[release['id']] and seal['captures']==[c['identity'] for c in caps],'seal_ordered_inventory')
 out={'verdict':'PASS','checks':dict(checks),'total_checks':sum(checks.values()),'raw_rows':len(rows),'entries':len(meta['entries']),'assertions':len(meta['assertions']),'native_fields':sum(fcounts.values()),'eligible_occurrences':sum(eligible.values()),'eligible_spellings':len(eligible),'all_spellings':len({r['word'] for r in rows}),'ordered_contexts':len(expected_context),'ordered_forms':len(expected_forms),'ordered_etymologies':len(expected_ety),'ordered_extensions':len(expected_extensions),'canonical_sha256':sha(b),'canonical_bytes':len(b),'raw_sha256':digest,'pack_path':str(pack)}
 need((len(rows),sum(fcounts.values()),len(meta['contexts']),len(meta['forms']),len(meta['etymologies']),len(meta['extensions']))==(17056,149596,47557,1548,1428,39974),'source_population_controls')
 (ROOT/'full-native-result01.json').write_text(json.dumps(out,indent=2)+'\n');print(json.dumps(out,indent=2))
if __name__=='__main__':main()
