#!/usr/bin/env python3
"""CDC raw-source vs full persisted projection oracle; read-only SQLite snapshot."""
import collections,hashlib,json,runpy,sqlite3,struct,sys
from pathlib import Path

RAW_NS='captured:lensisku:jbotci:en'
META_NS=RAW_NS+':metadata'
checks=collections.Counter()
def need(condition,kind,where):
 checks[kind]+=1
 if not condition:raise AssertionError(f'{kind}: {where}')
def sha(b):return hashlib.sha256(b).hexdigest()
def tupleid(ns,digest,loc):
 return b''.join(struct.pack('>Q',len(x.encode()))+x.encode() for x in [ns,digest,loc]).hex()
def main():
 rawp,metap,dbp=sys.argv[1:4]
 raw=Path(rawp).read_bytes();metadata=Path(metap).read_bytes();digest=sha(raw)
 tokenizer=runpy.run_path(str(Path(__file__).parent/'verify_normalized.py'))
 source=tokenizer['raw_array'](raw.decode())
 dbpath=Path(dbp).resolve();before=sha(dbpath.read_bytes())
 db=sqlite3.connect(dbpath.as_uri()+'?mode=ro',uri=True)
 db.execute('PRAGMA query_only=ON');db.execute('BEGIN')
 need(db.execute('PRAGMA foreign_key_check').fetchall()==[],'integrity','foreign keys')
 need(db.execute('PRAGMA integrity_check').fetchall()==[('ok',)],'integrity','SQLite structural integrity')
 caps={ns:(h,b) for ns,h,b in db.execute('SELECT namespace,sha256,bytes FROM raw_captures')}
 need(caps=={RAW_NS:(digest,raw),META_NS:(sha(metadata),metadata)},'raw_captures','exact independently pinned bytes')
 fields=collections.defaultdict(list)
 for loc,ordinal,name,encoding,value in db.execute('SELECT o.locator,f.ordinal,f.name,f.encoding,f.raw_value FROM native_fields f JOIN occurrence_refs o ON o.id=f.occurrence_id ORDER BY o.locator COLLATE BINARY,f.ordinal'):
  fields[loc].append((ordinal,name,encoding,value))
 sql='''SELECT r.ordinal,o.id,o.namespace,o.sha256,o.locator,e.spelling_state,e.spelling_value,
 e.native_word_class_state,e.native_word_class_value,e.selmaho_state,e.selmaho_value,
 a.definition_state,a.definition_text,a.definition_language_state,a.definition_language_value,
 a.definition_markup_state,a.definition_markup_value,a.source_score_state,a.source_score_value,
 a.eligibility,a.exclusion_reason,oa.native_entry_id,oa.native_assertion_id,c.display_name,
 a.native_sense_id_state,a.native_sense_id_value,e.id,a.id
 FROM raw_occurrences r JOIN occurrence_refs o ON o.id=r.occurrence_id
 JOIN assertions a ON a.occurrence_id=o.id JOIN entries e ON e.id=a.entry_id
 JOIN occurrence_attributions oa ON oa.occurrence_id=o.id
 JOIN contributors c ON c.id=oa.contributor_id ORDER BY r.ordinal'''
 got=db.execute(sql).fetchall();need(len(got)==len(source),'population','joined assertion/entry/attribution per raw row')
 expected_contexts=[];expected_forms=[];expected_etym=[];expected_extensions=[];eligible=collections.Counter();expected_queries={}
 for i,((row,lexeme),s) in enumerate(zip(source,got)):
  fs=tokenizer['raw_object'](lexeme);loc='/'+str(i);nativelex={k:v for k,v,_ in fs};fieldindex={k:j for j,(k,_,_) in enumerate(fs)}
  need(s[:5]==(i,tupleid(RAW_NS,digest,loc),RAW_NS,digest,loc),'occurrence_tuple',loc)
  need(fields[loc]==[(j,k,'Json',v) for j,(k,v,_) in enumerate(fs)],'all_native_fields',loc)
  need(s[5:9]==('value',row['word'],'value',row['word_type']),'entry',loc)
  expected_sel=('value',row['selmaho']) if 'selmaho' in row else ('missing',None)
  need(s[9:11]==expected_sel,'selmaho',loc)
  need(s[11:17]==('value',row['definition'],'value','en','missing',None),'definition',loc)
  need(s[17:19]==('value',nativelex['score']),'score_lexeme',loc)
  lexical=row['word_type']!='wiki' and bool(row['definition'].strip()) and bool(row['word'])
  need(s[19]==('Lexical' if lexical else 'Excluded'),'eligibility',loc)
  need(s[20] is None if lexical else bool(s[20]),'eligibility_reason',loc)
  need(s[21:24]==(None,nativelex['definition_id'],row['user']['username']),'source_attribution',loc)
  need(s[24:26]==('missing',None),'noninference',loc+' native sense')
  if lexical:
   eligible[row['word']]+=1;expected_queries.setdefault(row['word'],[]).append((s[1],nativelex['definition_id']))
  for key,kind in [('notes','Note'),('jargon','Domain')]:
   if key in row:expected_contexts.append((loc,str(fieldindex[key]),kind,'value',row[key]))
  for g in row.get('gloss_keywords',[]):expected_contexts.append((loc,str(fieldindex['gloss_keywords']),'Gloss','value',g['word']))
  if 'gloss_keywords' in row:expected_extensions.append((loc,str(fieldindex['gloss_keywords']),'assertion','lensisku.gloss_keywords.raw',nativelex['gloss_keywords']))
  for g,glexeme in tokenizer['raw_array'](nativelex.get('gloss_keywords','[]')):
   expected_extensions.append((loc,str(fieldindex['gloss_keywords']),'Gloss','lensisku.gloss_keywords.object',glexeme))
  if 'place_keywords' in row:expected_extensions.append((loc,str(fieldindex['place_keywords']),'assertion','lensisku.place_keywords',nativelex['place_keywords']))
  for key in ['rafsi','experimental_rafsi']:
   for form in row.get(key,'').split():expected_forms.append((loc,str(fieldindex[key]),form,'SourceReportedForm','value',key))
  if 'etymology' in row:expected_etym.append((loc,str(fieldindex['etymology']),row['etymology']))
 actual_contexts=db.execute('SELECT o.locator,se.field_index,c.kind,c.value_state,c.value_text FROM contexts c JOIN source_evidence se ON se.evidence_id=c.evidence_id JOIN occurrence_refs o ON o.id=se.occurrence_id').fetchall()
 need(collections.Counter(actual_contexts)==collections.Counter(expected_contexts),'contexts','all source-derived gloss/note/domain rows and native field evidence')
 actual_forms=db.execute('SELECT o.locator,se.field_index,f.spelling,f.kind,f.standing_state,f.standing_value FROM forms f JOIN source_evidence se ON se.evidence_id=f.evidence_id JOIN occurrence_refs o ON o.id=se.occurrence_id').fetchall()
 need(collections.Counter(actual_forms)==collections.Counter(expected_forms),'forms','all source-derived forms, duplicates, roles and field evidence')
 actual_etym=db.execute('SELECT o.locator,se.field_index,e.statement_text FROM etymologies e JOIN source_evidence se ON se.evidence_id=e.evidence_id JOIN occurrence_refs o ON o.id=se.occurrence_id').fetchall()
 need(collections.Counter(actual_etym)==collections.Counter(expected_etym),'etymologies','all native statements and field evidence')
 actual_extensions=db.execute("SELECT o.locator,se.field_index,CASE WHEN c.id IS NOT NULL THEN c.kind WHEN a.id IS NOT NULL THEN 'assertion' ELSE 'other-owner' END,x.name,x.value FROM extensions x JOIN source_evidence se ON se.evidence_id=x.evidence_id JOIN occurrence_refs o ON o.id=se.occurrence_id LEFT JOIN contexts c ON c.id=x.owner_id LEFT JOIN assertions a ON a.id=x.owner_id").fetchall()
 need(collections.Counter(actual_extensions)==collections.Counter(expected_extensions),'native_extensions','all exact gloss objects including null/missing meaning, place fields, owner roles and field evidence')
 expected_extension_counts=dict(collections.Counter(x[3] for x in expected_extensions));expected_extension_counts['jbotci.metadata.raw_text']=1
 need(dict(db.execute('SELECT name,count(*) FROM extensions GROUP BY name'))==expected_extension_counts,'extensions_total','all native categories plus one metadata extension, with no silent extras/omissions')
 need(db.execute('SELECT source_id,count(*) FROM entries GROUP BY source_id').fetchall()==[('lensisku',len(source))],'provenance','every entry source')
 need(set(db.execute('SELECT language_state,language_value,script_state,script_value,mapped_pos_state,mapped_pos_value,mapped_pos_rule FROM entries'))=={('value','jbo','missing',None,'missing',None,None)},'noninference','entry language and no script/POS inference')
 need(set(db.execute('SELECT language_state,language_value FROM assertions'))=={('value','en')},'provenance','assertion language')
 usernames={r['user']['username'] for r,_ in source}
 need(collections.Counter(x[0] for x in db.execute('SELECT display_name FROM contributors'))==collections.Counter(usernames),'provenance','every exact distinct source username once')
 need(db.execute('SELECT id,abbreviation,abbreviation_kind FROM sources').fetchall()==[('lensisku','LSK','ProjectAssigned')],'provenance','one source, explicit assigned abbreviation')
 need(db.execute('SELECT id FROM distributors').fetchall()==[('jbotci',)],'provenance','one separate supplier distributor')
 need(set(db.execute('SELECT namespace,sha256,source_id,distributor_id FROM snapshot_bindings'))=={(RAW_NS,digest,'lensisku','jbotci'),(META_NS,sha(metadata),'lensisku','jbotci')},'provenance','both exact capture bindings')
 for table in ['assertion_timestamps','argument_frames','argument_slots','relations','owned_revisions','revision_assertions','revision_tombstones','owned_heads','editorial_senses','editorial_members','computations','computation_inputs','annotations','form_components','etymology_components']:
  need(db.execute('SELECT count(*) FROM '+table).fetchone()[0]==0,'noninference',table)
 header=dict(db.execute('SELECT key,value FROM metadata'))
 for k,v in [('complete','1'),('schema_version','1'),('dictionary_version','1'),('registry_version','1'),('catalogue_revision','0')]:need(header[k]==v,'versions',k)
 revision='3d9fd41d9ed32de26311ee27ec86b6b165985aa1'
 pinned_notice='https://github.com/int19h/jbotci/blob/'+revision+'/THIRD-PARTY-NOTICES.md#L242-L252'
 need(db.execute('SELECT distributor_id,upstream_revision_state,upstream_revision_value,distributor_revision_state,distributor_revision_value,captured_at_state,captured_at_value,time_precision,schema_version,normalizer,normalizer_version,accounting_input_occurrences,accounting_normalized_occurrences,accounting_query_excluded_occurrences,selected,availability,redistribution FROM releases').fetchall()==[('jbotci','missing',None,'value',revision,'missing',None,'Unknown',1,'vlajmaji-jbotci-export','1',str(len(source)),str(len(source)),'10',1,'Available','Held')],'release','full balanced release provenance/versions/terms')
 need(db.execute('SELECT locator_state,locator_value,license_state,license_value FROM code_notices').fetchall()==[('value','LICENSE','value','Apache-2.0')],'terms','truthful separate code notice')
 content=db.execute('SELECT text_state,text_value,locator_state,locator_value,license_state,license_value FROM content_notices').fetchall()
 need(len(content)==1 and content[0][0]=='value' and content[0][2:5]==('value',pinned_notice,'value') and 'supplier' in content[0][5] and 'claimed' in content[0][5],'terms','pinned supplier claim and coverage qualification')
 need(db.execute('SELECT x.name,x.value,x.evidence_id FROM extensions x JOIN releases r ON r.id=x.owner_id').fetchall()==[('jbotci.metadata.raw_text',json.dumps(metadata.decode(),ensure_ascii=False),None)],'metadata_capture','exact supplier metadata text also retained in release extension')
 need(set(db.execute('SELECT citation_state,citation_value,offset_start,offset_end FROM source_evidence'))=={('missing',None,None,None)},'noninference','no fabricated citation or offsets')
 captured=set(db.execute('SELECT namespace,revision_state,revision_value,captured_at_state,captured_at_value,time_precision,notice_locator_state,notice_locator_value FROM raw_captures'))
 need(captured=={(RAW_NS,'value',revision,'missing',None,'Unknown','value',pinned_notice),(META_NS,'value',revision,'missing',None,'Unknown','missing',None)},'provenance','exact capture revision, unknown time and pinned notice')
 lineage=db.execute('SELECT source_id,target_kind,upstream_source_id,unresolved_text,evidence FROM source_lineage').fetchall()
 need(len(lineage)==1 and lineage[0][:4]==('lensisku','Unresolved',None,'https://jbovlaste.lojban.org/') and pinned_notice in lineage[0][4],'provenance','unresolved upstream with actually witnessed supplier claim')
 need(db.execute('SELECT count(*) FROM release_assertions').fetchone()[0]==len(source),'release','all assertions remain listed')
 need(db.execute('SELECT count(*) FROM release_captures').fetchone()[0]==2,'release','both captures remain listed')
 # Every eligible spelling is compared to a native-source expectation, independently of product query code.
 sqlcounts=dict(db.execute("SELECT e.spelling_value,count(*) FROM entries e JOIN assertions a ON a.entry_id=e.id WHERE a.eligibility='Lexical' GROUP BY e.spelling_value COLLATE BINARY"))
 need(sqlcounts==dict(eligible),'exact_population','all eligible spelling counts')
 observed={}
 for spelling,oid,native in db.execute("SELECT e.spelling_value,a.occurrence_id,oa.native_assertion_id FROM entries e JOIN assertions a ON a.entry_id=e.id JOIN occurrence_attributions oa ON oa.occurrence_id=a.occurrence_id WHERE a.eligibility='Lexical' ORDER BY e.spelling_value COLLATE BINARY,a.occurrence_id COLLATE BINARY"):
  observed.setdefault(spelling,[]).append((oid,native))
 need(observed=={k:sorted(v) for k,v in expected_queries.items()},'exact_population','all eligible occurrence tuple/native assertion groups')
 db.close();need(before==sha(dbpath.read_bytes()),'no_write','database exact bytes after read-only oracle')
 out={'verdict':'PASS','raw_sha256':digest,'metadata_sha256':sha(metadata),'database_sha256':before,
 'rows':len(source),'fields':sum(map(len,fields.values())),'eligible_occurrences':sum(eligible.values()),'eligible_spellings':len(eligible),
 'contexts':len(actual_contexts),'source_extensions':len(actual_extensions),'extension_counts':expected_extension_counts,'total_extensions':sum(expected_extension_counts.values()),'forms':len(actual_forms),'etymologies':len(actual_etym),'check_categories':dict(checks),'total_checks':sum(checks.values())}
 print(json.dumps(out,indent=2))
if __name__=='__main__':main()
