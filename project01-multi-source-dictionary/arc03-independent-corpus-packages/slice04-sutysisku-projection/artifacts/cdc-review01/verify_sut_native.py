from pathlib import Path
from collections import Counter
import json,hashlib,sys,importlib.util
spec=importlib.util.spec_from_file_location('native_source','/tmp/vlajmaji-p01-a03-cdc/readiness/distribution_census.py');src=importlib.util.module_from_spec(spec);spec.loader.exec_module(src)
ROOT=Path('/tmp/vlajmaji-p01-a03-cdc/s04');REV='2c71205747143ea965749ffc3ac3b48ef5c0a517';PIN='8e80f6626d2e68a17025d52c3f0cf5ea8c32cbfeeb0c2ff1417fcba539bd043d'
sha=lambda b:hashlib.sha256(b).hexdigest()
checks=Counter()
def need(ok,label):
 checks[label]+=1
 if not ok:raise AssertionError(label)
def main():
 raw=(src.p/'sutysisku-parsed-en.json').read_bytes();rows=src.fields(raw.decode());need(sha(raw)==PIN,'independent_source_pin')
 pack=Path(sys.argv[1]);payload=(pack/'release.json').read_bytes();model=json.loads(payload);report=json.loads((pack/'report.json').read_bytes());seal=json.loads((pack/'seal.json').read_bytes())
 meta=model['metadata'];reg=model['registry'];caps=model['captures'];need(len(caps)==1 and bytes(caps[0]['bytes'])==raw,'exact_complete_capture')
 cap=caps[0];need(cap['identity']=={'namespace':'captured:sutysisku:parsed-en','sha256':PIN},'capture_identity')
 need(cap['metadata']['revision']=={'Value':REV} and cap['metadata']['captured_at']=='Missing','captured_provenance')
 need(len(rows)==len(cap['occurrences'])==len(meta['entries'])==len(meta['assertions'])==len(reg['occurrences']),'all_received_members_accounted')
 base='sutysisku-'+PIN;counts=Counter({'$key':len(rows)});types=Counter();expected_context=[];expected_forms=[];expected_ext=[];eligible=[];excluded=[]
 for i,((word,lex,row,keylex),occ,entry,a,attr) in enumerate(zip(rows,cap['occurrences'],meta['entries'],meta['assertions'],reg['occurrences'])):
  fs=src.fields(lex);counts.update(k for k,_,_,_ in fs);types[row['t']]+=1;ai=f'{base}-assertion-{i}';ei=f'{base}-entry-{i}';loc=f'/{i}';positions={k:j+1 for j,(k,_,_,_) in enumerate(fs)}
  expected=[{'name':'$key','encoding':'Json','raw_value':keylex}]+[{'name':k,'encoding':'Json','raw_value':v} for k,v,_,_ in fs]
  need(occ['locator']==loc and occ['raw_line'] is None and occ['fields']==expected,'ordered_keys_fields_exact_lexemes')
  need(entry['id']==ei and a['id']==ai and a['entry']==ei,'ordinal_entity_identity')
  need(entry['source']=='sutysisku' and entry['spelling']=={'Value':word} and entry['native_word_class']=={'Value':row['t']},'native_spelling_class')
  need(entry['language']=={'Value':'jbo'} and entry['script']=='Missing' and entry['mapped_pos']=='Missing','entry_noninference')
  need(entry['selmaho']==({'Value':row['s']} if 's' in row else 'Missing'),'native_selmaho_absence')
  need(a['definition']['Value']['text']==row['d'] and a['definition']['Value']['language']=={'Value':'en'},'exact_definition')
  need(a['language']=={'Value':'en'} and a['source_score']=='Missing' and a['native_sense_id']=='Missing' and not a['timestamps'],'assertion_noninference')
  need(a['occurrence']=={'capture':cap['identity'],'locator':loc},'complete_assertion_reference')
  need(attr['occurrence']=={'snapshot':cap['identity'],'locator':loc} and attr['native_entry_id']==word and attr['native_assertion_id'] is None and attr['contributor'] is None,'native_identity_attribution')
  yes=bool(word) and row['t'] in src.classes and bool(row['d'].strip());need((a['eligibility']=='Lexical')==yes,'source_policy_eligibility');(eligible if yes else excluded).append(word)
  # Required body invalidity does not discard supplied optional evidence/contexts.
  for k,rawvalue,value,_ in fs:
   field=str(positions[k])
   if k=='n':expected_context.append((f'{base}-context-note-{i}',ai,'Note',value,field))
   if k=='g':
    expected_ext.append((ai,'sutysisku.g.raw',rawvalue,loc,field))
    for j,g in enumerate(value):expected_context.append((f'{base}-context-gloss-{i}-{j}',ai,'Gloss',g,field))
   if k=='r':
    for j,form in enumerate(value):expected_forms.append((f'{base}-form-{i}-r-{j}',ei,form,'SourceReportedForm','r',field))
   if k=='b':expected_ext.append((ai,'sutysisku.b',rawvalue,loc,field))
 need(report['field_counts']==dict(counts) and report['word_types']==dict(types),'full_report_category_census')
 need((report['root_keys'],report['native_child_fields'],report['native_fields'],report['non_object_rows'])==(23705,91514,115219,0),'separate_key_child_denominators')
 need((report['input_rows'],report['normalized_assertions'],report['lexical_occurrences'],report['excluded_occurrences'],report['eligible_spellings'],report['all_spellings'])==(23705,23705,23704,1,23704,23705),'balanced_report')
 need(excluded==['zoizoizoi'],'actual_excluded_member')
 need(report['diagnostics']=={'invalid_definition':1},'truthful_invalid_definition_report')
 need(all(meta[k]==[] for k in ['frames','relations','owned_revisions','owned_heads','editorial_senses','computations','annotations','etymologies']),'no_inferred_model_facts')
 for group in ['contexts','forms']:
  for x in meta[group]:
   ev=x['evidence'];owner=x.get('assertion') or x.get('entry');ordinal=int(owner.rsplit('-',1)[1]);need(ev is not None and ev['occurrence']=={'capture':cap['identity'],'locator':f'/{ordinal}'},'typed_evidence_full_reference')
 need(all(x['language']=={'Value':'jbo'} and x['script']=='Missing' and x['pronunciation']=='Missing' and not x['components'] and x['computation'] is None for x in meta['forms']),'source_reported_forms_noninference')
 need(all(x['value']['Value']['language']=={'Value':'en'} for x in meta['contexts']),'context_language')
 actual_context=[(x['id'],x['assertion'],x['kind'],x['value']['Value']['text'],str(x['evidence']['field_index'])) for x in meta['contexts']]
 actual_forms=[(x['id'],x['entry'],x['spelling'],x['kind'],x['standing']['Value'],str(x['evidence']['field_index'])) for x in meta['forms']]
 actual_ext=[(x['owner'],x['name'],x['value'],x['evidence']['occurrence']['locator'],str(x['evidence']['field_index'])) for x in meta['extensions']]
 need(Counter(actual_context)==Counter(expected_context),'all_context_ids_owners_values_evidence')
 need(Counter(actual_forms)==Counter(expected_forms),'all_form_ids_owners_values_standing_evidence')
 need(actual_ext==expected_ext,'all_extension_owners_order_raw_lexemes_evidence')
 need(reg['sources']==[{'id':'sutysisku','display_name':'Sutysisku','abbreviation':'SUT','abbreviation_kind':'ProjectAssigned','abbreviation_evidence':None,'origin':'Upstream'}],'truthful_projection_source')
 need(reg['distributors']==[{'id':'la-lojban','display_name':'La-Lojban'}] and reg['contributors']==[],'verified_publisher_no_invented_contributors')
 need(reg['lineage']==[{'source':'sutysisku','upstream':{'Unresolved':'Exact upstream lexical dictionary lineage is unestablished for the pinned Sutysisku projection.'},'evidence':None}],'explicit_unresolved_lineage')
 need(all(x['source']=='sutysisku' and x['distributor']=='la-lojban' for x in reg['snapshots']),'source_distribution_distinction')
 release=meta['releases'][0];need(release['id']=='sutysisku-release-'+sha(('vlajmaji-sutysisku-release-v1\0'+REV+'\0'+PIN).encode()),'release_revision_identity')
 need(release['redistribution']=='Held' and release['upstream_revision']=='Missing' and release['distributor_revision']=={'Value':REV},'truthful_provenance_terms')
 need(release['content_notices'][0]['text']=='Missing' and release['content_notices'][0]['license']=='Missing' and meta['code_notices'][0]['locator']=={'Value':'LICENSE'},'code_terms_separate_no_data_grant')
 need(release['captures']==[cap['identity']] and release['assertions']==[a['id'] for a in meta['assertions']],'complete_release_membership')
 need(len(release['diagnostics'])==1 and release['diagnostics'][0]['code']=='invalid_definition' and release['diagnostics'][0]['occurrence']=={'capture':cap['identity'],'locator':'/23602'},'located_excluded_definition_diagnostic')
 need(meta['collections'][0]['id']=='sutysisku-en-collection' and meta['components'][0]['id']=='sutysisku-parsed-en-component' and release['component']==meta['components'][0]['id'],'collection_component_release_identity')
 need(release['selected'] and release['availability']=='Available' and release['normalizer']=='vlajmaji-sutysisku-export' and release['normalizer_version']=='1','release_semantics')
 need(release['accounting']=={'input_occurrences':23705,'normalized_occurrences':23705,'query_excluded_occurrences':1,'unit':'source-object-member'},'release_population_disposition')
 need((len(meta['contexts']),len(meta['forms']),len(meta['extensions']))==(47492,2951,22042),'source_complete_typed_population_controls')
 need(seal['payload_kind']=='full' and seal['pack_id']=='sutysisku' and seal['release_sha256']==sha(payload) and seal['release_bytes']==len(payload),'full_sealed_payload_integrity')
 need(seal['source_ids']==['sutysisku'] and seal['release_ids']==[release['id']] and seal['captures']==[cap['identity']],'complete_sealed_inventory')
 out={'verdict':'PASS','total_checks':sum(checks.values()),'checks':dict(checks),'raw_sha256':PIN,'raw_bytes':len(raw),'root_keys':len(rows),'native_child_fields':sum(counts.values())-len(rows),'native_fields_with_keys':sum(counts.values()),'lexical':len(eligible),'excluded':len(excluded),'contexts':len(actual_context),'forms':len(actual_forms),'extensions':len(actual_ext),'canonical_bytes':len(payload),'canonical_sha256':sha(payload),'pack_path':str(pack)}
 (ROOT/'full-native-result01.json').write_text(json.dumps(out,indent=2)+'\n');print(json.dumps(out,indent=2))
if __name__=='__main__':main()
