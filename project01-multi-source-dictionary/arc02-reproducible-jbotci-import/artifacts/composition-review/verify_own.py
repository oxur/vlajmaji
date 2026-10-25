from pathlib import Path
import json,collections,hashlib,sqlite3,struct,re
from raw_reference import ROOT,LEAF,CLASSES,load_source,sha
checks=collections.Counter()
def need(condition,label):
 checks[label]+=1
 if not condition:raise AssertionError(label)
def main():
 rb,rows,fields,lexemes=load_source();digest=sha(rb);mb=(LEAF/'dictionary-en.metadata.toml').read_bytes()
 b=(ROOT/'canonical.json').read_bytes();need(len(b)==215412497 and sha(b)=='0987ec84a8434573b5979ea854ca459ce2ea9c7fa03ebac6639574dd2b7aa460','canonical_identity')
 model=json.loads(b);report=json.loads((ROOT/'import-report.json').read_text());meta=model['metadata'];reg=model['registry'];caps=model['captures']
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
 dbp=ROOT/'fresh-full-store/dictionary.sqlite3';before=sha(dbp.read_bytes());db=sqlite3.connect(dbp.resolve().as_uri()+'?mode=ro',uri=True);db.execute('PRAGMA query_only=ON');db.execute('BEGIN')
 need(dict(db.execute('SELECT key,value FROM metadata'))['complete']=='1','complete_marker')
 source=Path('/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc02/vlajmaji/crates/store/src/dictionary_schema.rs').read_text();ddl=source.split('const DDL: &str = r#"',1)[1].split('"#;',1)[0]
 memory=sqlite3.connect(':memory:');memory.execute('PRAGMA foreign_keys=ON');memory.executescript(ddl)
 query="SELECT type,name,tbl_name,sql FROM sqlite_schema WHERE name NOT LIKE 'sqlite_%' ORDER BY type,name"
 need(db.execute(query).fetchall()==memory.execute(query).fetchall(),'exact_ddl_indexes')
 need(db.execute('PRAGMA integrity_check').fetchall()==[('ok',)] and db.execute('PRAGMA foreign_key_check').fetchall()==[],'integrity_foreign_keys')
 try:
  memory.execute("INSERT INTO source_aliases VALUES(0,'nonexistent-source','bad-alias','Active',NULL)");memory.commit();raise AssertionError('missing FK failure')
 except sqlite3.IntegrityError:memory.rollback();need(True,'fk_enforced_valid_schema')
 actual_context=db.execute('SELECT c.id,c.assertion_id,c.kind,c.value_text,se.field_index FROM contexts c JOIN entity_ids e ON e.id=c.id JOIN source_evidence se ON c.evidence_id=se.evidence_id ORDER BY e.ordinal').fetchall()
 need(actual_context==expected_context,'all_context_ids_owners_order_values')
 actual_forms=db.execute('SELECT f.id,f.entry_id,f.spelling,f.kind,f.standing_value,se.field_index FROM forms f JOIN entity_ids e ON e.id=f.id JOIN source_evidence se ON f.evidence_id=se.evidence_id ORDER BY e.ordinal').fetchall()
 need(actual_forms==expected_forms,'all_form_ids_owners_order_values')
 actual_ety=db.execute('SELECT x.id,x.assertion_id,x.statement_text,se.field_index FROM etymologies x JOIN entity_ids e ON e.id=x.id JOIN source_evidence se ON x.evidence_id=se.evidence_id ORDER BY e.ordinal').fetchall()
 need(actual_ety==expected_ety,'all_etymology_ids_owners_order_values')
 actual_ext=db.execute('SELECT x.owner_id,x.name,x.value,o.locator,se.field_index FROM extensions x LEFT JOIN source_evidence se ON se.evidence_id=x.evidence_id LEFT JOIN occurrence_refs o ON o.id=se.occurrence_id ORDER BY x.ordinal').fetchall()
 need(actual_ext==expected_extensions,'all_exact_extension_owners_order_lexemes')
 actual_groups=collections.defaultdict(list)
 for w,aid,native in db.execute("SELECT e.spelling_value,a.id,oa.native_assertion_id FROM entries e JOIN assertions a ON a.entry_id=e.id JOIN occurrence_attributions oa ON oa.occurrence_id=a.occurrence_id WHERE a.eligibility='Lexical' ORDER BY e.spelling_value COLLATE BINARY,a.id COLLATE BINARY"):actual_groups[w].append((aid,native))
 need(dict(actual_groups)=={k:sorted(v) for k,v in expected_groups.items()},'all31248groups33516eligible_assertion_native_pairs')
 db.close();need(before==sha(dbp.read_bytes()),'no_store_mutation')
 # Consume complete public query values against raw-native expectations and ordered native payload.
 for name,word in [('farvi-public-full','farvi'),('ma-full','ma')]:
  v=json.loads((ROOT/(name+'.json')).read_text());hits=[h for g in v['groups'] for h in g['hits']];ri=[i for i,r in enumerate(rows) if r['word']==word and r['definition'].strip() and r['word_type'] in CLASSES]
  need(len(hits)==len(ri)==v['totals']['occurrences'],'public_complete_totals')
  for hit in hits:
   i=int(hit['assertion']['occurrence']['locator'][1:]);need(i in ri and hit['assertion']['definition']['Value']['text']==rows[i]['definition'],'public_all_competing_definitions')
   need(hit['raw']==caps[0]['occurrences'][i] and hit['occurrence_attribution']==reg['occurrences'][i],'public_whole_raw_native_attribution')
   need(hit['entry']==meta['entries'][i] and hit['assertion']==meta['assertions'][i],'public_complete_entry_assertion')
 for name,index in [('native333',13306),('wiki-native88416',22040),('local13306',13306)]:
  v=json.loads((ROOT/(name+'.json')).read_text());need(v['status']=='Resolved' and len(v['confirmed'])==1,'public_reference_status');hit=v['confirmed'][0]
  need(hit['raw']==caps[0]['occurrences'][index] and hit['assertion']==meta['assertions'][index] and hit['entry']==meta['entries'][index] and hit['payload_present'],'public_reference_complete_raw_native')
  need(hit['current_eligible']==(index!=22040),'public_excluded_resolves_retained')
 out={'verdict':'PASS','checks':dict(checks),'total_checks':sum(checks.values()),'raw_rows':len(rows),'native_fields':sum(fcounts.values()),'eligible_occurrences':sum(eligible.values()),'eligible_spellings':len(eligible),'ordered_contexts':len(expected_context),'ordered_forms':len(expected_forms),'ordered_etymologies':len(expected_ety),'ordered_extensions':len(expected_extensions),'store_sha256':before,'canonical_sha256':sha(b)}
 (ROOT/'own-reference-result.json').write_text(json.dumps(out,indent=2)+'\n');print(json.dumps(out,indent=2))
if __name__=='__main__':main()
