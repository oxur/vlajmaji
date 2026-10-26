"""Arc-scale complete raw-derived SQL tuples. Original inputs read-only; no model used as expected output."""
from pathlib import Path
from collections import Counter,defaultdict
import sqlite3,json,hashlib,sys,time
import raw_reference as jb
import distribution_census as dist
from verify_llg_recipe import raw_fields
def counter_equal(a,b):
 ca,cb=Counter(a),Counter(b);return set(ca)==set(cb) and ca==cb
MISSING=object()
def context_tuple(a,k,v,ev0):
 if v is MISSING:return(a,k,'missing')+(None,)*6+ev0
 if v is None:return(a,k,'null')+(None,)*6+ev0
 return(a,k,'value',v,'value','en','missing',None,None)+ev0
if len(sys.argv)>1 and sys.argv[1]=='--comparator-controls':
 cases=[('empty-positive',[],[],True),('reordered-positive',[('source','a'),('source','b')],[('source','b'),('source','a')],True),('duplicate-negative',[('same',None)]*2,[('same',None)],False),('missing-negative',[],[('present',None)],False),('wrong-source-negative',[('lensisku','farvi')],[('llg','farvi')],False),('zero-count-key-negative',Counter({'explicit-empty-category':0}),Counter(),False),('missing-category-marker-negative',[('category','frames')],[],False),('missing-vs-null-negative',[('state','missing',None)],[('state','null',None)],False),('null-vs-empty-value-negative',[('state','null',None)],[('state','value','')],False)]
 for name,a,b,want in cases:assert counter_equal(a,b)==want,name
 e=('ns','hash','/0','3','missing',None,None,None)
 assert context_tuple('a','Note',None,e)==('a','Note','null',None,None,None,None,None,None)+e
 assert context_tuple('a','Note',MISSING,e)==('a','Note','missing',None,None,None,None,None,None)+e
 assert context_tuple('a','Note','',e)==('a','Note','value','','value','en','missing',None,None)+e
 assert context_tuple('a','Note','literal',e)==('a','Note','value','literal','value','en','missing',None,None)+e
 print(json.dumps({'status':'PASS','same_counter_comparator_cases':len(cases),'same_context_helper_presence_cases':4,'cases':[x[0] for x in cases],'scope':'comparator only; no corpus/store/product read'}));raise SystemExit(0)
assert len(sys.argv)==4, 'usage: native_topology.py NEW_DB NEW_OUTPUT_DIR comma-separated-pack-ids'
R=Path(sys.argv[2]);R.mkdir(exist_ok=False)
PACKS=set(sys.argv[3].split(','));assert PACKS and PACKS<={'lensisku-jbotci','llg','la-lojban','sutysisku'}
pre=json.loads(Path('/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/planning/project01-multi-source-dictionary/arc03-independent-corpus-packages/artifacts/composition-review/preflight01.json').read_text());checks=Counter();stats={};contexts=[];forms=[];etys=[];ext=[];expected_keys=set();eligible=[];expected_fields=0
sha=lambda b:hashlib.sha256(b).hexdigest()
def ck(ok,name):
 checks[name]+=1
 if not ok:raise AssertionError(name)
def equal(a,b,name):
 ca,cb=Counter(a),Counter(b);ck(counter_equal(a,b),name)
 return {'rows':sum(cb.values()),'sha256':sha(json.dumps(sorted(cb.items(),key=repr),ensure_ascii=False,separators=(',',':')).encode())}
def hp(parts):
 h=hashlib.sha256()
 for s in parts:b=s.encode();h.update(len(b).to_bytes(8,'big'));h.update(b)
 return h.hexdigest()
def tupleid(ns,h,loc):return b''.join(len(x.encode()).to_bytes(8,'big')+x.encode() for x in [ns,h,loc]).hex()
def ev(ns,h,loc,i):return(ns,h,loc,str(i),'missing',None,None,None)
def context(a,k,v,ev0):contexts.append(context_tuple(a,k,v,ev0))
def form(e,s,k,standing,ev0):forms.append((e,s,k,'value',standing,'value','jbo','missing',None,'missing',None,None,None,None,None,None)+ev0)
def ety(a,s,ev0):etys.append((a,'value',s,'value','en','missing',None,'missing',None)+ev0)
DB=Path(sys.argv[1]);assert 'vlajmaji-p01-a04' in str(DB) and DB.name=='dictionary.sqlite3';before=sha(DB.read_bytes());con=sqlite3.connect(f'file:{DB}?mode=ro',uri=True);con.execute('PRAGMA query_only=ON');con.row_factory=sqlite3.Row
query='''SELECT o.namespace,o.sha256,o.locator,o.id AS oid,r.ordinal,r.raw_line,a.id AS aid,a.entry_id AS eid,e.source_id,e.spelling_state,e.spelling_value,e.language_state AS elangstate,e.language_value AS elang,e.script_state,e.native_word_class_state AS classstate,e.native_word_class_value AS class,e.selmaho_state,e.selmaho_value,e.mapped_pos_state,a.native_sense_id_state,a.language_state AS alangstate,a.language_value AS alang,a.definition_state,a.definition_text,a.definition_language_state,a.definition_language_value,a.definition_markup_state,a.source_score_state,a.source_score_value,a.eligibility,oa.native_entry_id,oa.native_assertion_id,oa.contributor_id,c.display_name,s.distributor_id FROM raw_occurrences r JOIN occurrence_refs o ON o.id=r.occurrence_id JOIN assertions a ON a.occurrence_id=o.id JOIN entries e ON e.id=a.entry_id LEFT JOIN occurrence_attributions oa ON oa.occurrence_id=o.id LEFT JOIN contributors c ON c.id=oa.contributor_id JOIN snapshot_bindings s ON s.namespace=o.namespace AND s.sha256=o.sha256'''
actual={}
for row in con.execute(query):
 k=(row['namespace'],row['sha256'],row['locator']);ck(k not in actual,'unique_sql_occurrence');actual[k]=dict(row)
fields=defaultdict(list)
for row in con.execute('SELECT occurrence_id,ordinal,name,encoding,raw_value FROM native_fields ORDER BY occurrence_id,ordinal'):
 fields[row[0]].append(tuple(row[1:]))
def occurrence(ns,h,loc,ordinal,line,fs,source,word,definition,yes,nentry,nassert,username,aid,eid,wordclass=None,selmaho=None,distributor=None,score=None):
 global expected_fields
 k=(ns,h,loc);expected_keys.add(k);a=actual[k];oid=tupleid(*k);ck(a['oid']==oid and a['ordinal']==ordinal and a['raw_line']==line,'exact_occurrence_tuple_ordinal_line');ck(fields[oid]==[(i,n,enc,v) for i,(n,enc,v) in enumerate(fs)],'all_native_field_ordinals_names_encodings_lexemes');expected_fields+=len(fs)
 expected={'aid':aid,'eid':eid,'source_id':source,'spelling_state':'missing' if word is None else 'value','spelling_value':word,'elangstate':'missing' if word is None else 'value','elang':None if word is None else 'jbo','script_state':'missing','classstate':'missing' if wordclass is None else 'value','class':wordclass,'selmaho_state':'missing' if selmaho is None else 'value','selmaho_value':selmaho,'mapped_pos_state':'missing','native_sense_id_state':'missing','alangstate':'missing' if definition is None else 'value','alang':None if definition is None else 'en','definition_state':'missing' if definition is None else 'value','definition_text':definition,'definition_language_state':None if definition is None else 'value','definition_language_value':None if definition is None else 'en','definition_markup_state':None if definition is None else 'missing','source_score_state':'missing' if score is None else 'value','source_score_value':score,'eligibility':'Lexical' if yes else 'Excluded','native_entry_id':nentry,'native_assertion_id':nassert,'contributor_id':None if username is None else 'lensisku-user-'+sha(username.encode()),'display_name':username,'distributor_id':distributor}
 ck(all(a[k0]==v for k0,v in expected.items()),'complete_native_assertion_entry_attribution_policy')
 if yes:eligible.append((source,word,oid,aid))
 return a
for pack in [x for x in ['lensisku-jbotci','la-lojban'] if x in PACKS]:
 if pack=='lensisku-jbotci':raw,rows,fs,lex=jb.load_source();ns='captured:lensisku:jbotci:en';base='jbotci-'+sha(raw);distributor='jbotci'
 else:
  raw,rows,fs0,lex=dist.la_source();fs=[[(k,s,v) for k,s,v,_ in f] for f in fs0];ns='captured:lensisku:la-lojban:en';base='la-lojban-'+sha(raw);distributor='la-lojban'
 h=sha(raw);yesn=0
 for i,(row,native) in enumerate(zip(rows,fs)):
  loc='/'+str(i);aid=base+'-assertion-'+str(i);eid=base+'-entry-'+str(i);fmap={k:s for k,s,v in native};positions={k:j for j,(k,s,v) in enumerate(native)};yes=bool(row['word']) and row['word_type'] in jb.CLASSES and bool(row['definition'].strip());yesn+=yes
  occurrence(ns,h,loc,i,None,[(k,'Json',s) for k,s,v in native],'lensisku',row['word'],row['definition'],yes,None,fmap['definition_id'],row['user']['username'],aid,eid,row['word_type'],row.get('selmaho'),distributor,fmap['score'])
  for k,kind in [('notes','Note'),('jargon','Domain')]:
   if k in row:context(aid,kind,row[k],ev(ns,h,loc,positions[k]))
  if 'gloss_keywords' in row:
   v=fmap['gloss_keywords'];ev0=ev(ns,h,loc,positions['gloss_keywords']);ext.append((aid,'lensisku.gloss_keywords.raw',v)+ev0);p=1;dec=json.JSONDecoder()
   for j,g in enumerate(row['gloss_keywords']):
    while v[p].isspace():p+=1
    start=p;obj,p=dec.raw_decode(v,p);ck(obj==g,'independent_exact_gloss_object');cid=base+'-context-gloss-'+str(i)+'-'+str(j);context(aid,'Gloss',g['word'],ev0);ext.append((cid,'lensisku.gloss_keywords.object',v[start:p])+ev0)
    while v[p].isspace():p+=1
    if v[p]==',':p+=1
  if 'place_keywords' in row:ext.append((aid,'lensisku.place_keywords',fmap['place_keywords'])+ev(ns,h,loc,positions['place_keywords']))
  for k in ['rafsi','experimental_rafsi']:
   for s in row.get(k,'').split():form(eid,s,'SourceReportedForm',k,ev(ns,h,loc,positions[k]))
  if 'etymology' in row:ety(aid,row['etymology'],ev(ns,h,loc,positions['etymology']))
 if pack=='lensisku-jbotci':
  meta=Path(pre['inputs']['captured:lensisku:jbotci:en:metadata']['path']).read_bytes();rid=pre['packs'][pack]['seal']['release_ids'][0];ext.append((rid,'jbotci.metadata.raw_text',json.dumps(meta.decode(),ensure_ascii=False))+(None,)*8)
 stats[pack]={'occurrences':len(rows),'native_fields':sum(map(len,fs)),'lexical':yesn}
if 'sutysisku' in PACKS:
 raw=Path(pre['inputs']['captured:sutysisku:parsed-en']['path']).read_bytes();h=sha(raw);ns='captured:sutysisku:parsed-en';rows=dist.fields(raw.decode());base='sutysisku-'+h;yesn=0;sfields=0
 for i,(word,lex,row,keylex) in enumerate(rows):
  fs=dist.fields(lex);loc='/'+str(i);aid=base+'-assertion-'+str(i);eid=base+'-entry-'+str(i);yes=bool(word) and row['t'] in jb.CLASSES and bool(row['d'].strip());yesn+=yes;sfields+=1+len(fs)
  occurrence(ns,h,loc,i,None,[('$key','Json',keylex)]+[(k,'Json',s) for k,s,v,_ in fs],'sutysisku',word,row['d'],yes,word,None,None,aid,eid,row['t'],row.get('s'),'la-lojban')
  for index,(k,s,v,_) in enumerate(fs,1):
   ev0=ev(ns,h,loc,index)
   if k=='n':context(aid,'Note',v,ev0)
   if k=='g':
    ext.append((aid,'sutysisku.g.raw',s)+ev0)
    for g in v:context(aid,'Gloss',g,ev0)
   if k=='r':
    for f in v:form(eid,f,'SourceReportedForm','r',ev0)
   if k=='b':ext.append((aid,'sutysisku.b',s)+ev0)
 stats['sutysisku']={'occurrences':len(rows),'native_fields':sfields,'lexical':yesn}
if 'llg' in PACKS:
 llg_rows={};classes=defaultdict(set)
 for kind in ['gismu','cmavo','rafsi']:
  ns='captured:llg:'+kind;p=Path(pre['inputs'][ns]['path']);raw=p.read_bytes();h=sha(raw);llg_rows[kind]=(ns,h,raw.decode().splitlines())
  if kind=='cmavo':
   for line in llg_rows[kind][2][1:]:f=dict(raw_fields(kind,line,2));classes[f['word'].strip()].add(f['selmaho'].strip())
 count=0;nf=0;lexn=0
 for kind,(ns,h,lines) in llg_rows.items():
  for i,line in enumerate(lines):
   loc=str(i+1);fs=raw_fields(kind,line,i+1);nf+=len(fs);count+=1;aid='llg-assertion-'+hp(['llg-assertion-v1',h,kind,loc]);f=dict(fs);header=kind!='rafsi' and i==0;word=None if header else f['word'].strip();eid='llg-placeholder-'+hp(['llg-placeholder-v1',ns,h,loc]) if header else 'llg-entry-'+hp(['llg-entry-v1',word]);yes=not header and kind!='rafsi';lexn+=yes;definition=f['definition'].strip() if yes else None;selmaho=next(iter(classes[word])) if word in classes and len(classes[word])==1 else None
   occurrence(ns,h,loc,i,line,[(k,'Text',v) for k,v in fs],'llg',word,definition,yes,word,None,None,aid,eid,None,selmaho)
   if header:continue
   for j,(k,v) in enumerate(fs):
    trimmed=v.strip();ev0=ev(ns,h,loc,j)
    if k in ['keyword','hint','notes','gloss'] and trimmed:context(aid,'Note' if k in ['hint','notes'] else 'Gloss',('Native hint: ' if k=='hint' else '')+trimmed,ev0)
    if (k.startswith('assigned_rafsi_') or kind=='rafsi' and k=='rafsi') and trimmed:form(eid,trimmed,'AssignedRafsi','LLG assigned rafsi list' if kind=='rafsi' else 'LLG assigned gismu slot',ev0)
    if k in ['teaching_code','frequency_1993']:ext.append((aid,'llg.'+k,json.dumps(v,ensure_ascii=False))+ev0)
 stats['llg']={'occurrences':count,'native_fields':nf,'lexical':lexn}
ck(set(actual)==expected_keys and len(actual)==sum(x['occurrences'] for x in stats.values()),'complete_all_occurrences_no_omissions');ck(len(fields)==len(expected_keys) and expected_fields==sum(x['native_fields'] for x in stats.values()) and sum(map(len,fields.values()))==expected_fields,'complete_all_native_fields_no_omissions')
expected_captures=[(c['namespace'],c['sha256']) for p in PACKS for c in pre['packs'][p]['seal']['captures']];actual_captures=[(c[0],c[1]) for c in con.execute('SELECT namespace,sha256 FROM raw_captures')];ck(counter_equal(actual_captures,expected_captures),'complete_capture_identity_multiset')
for c in con.execute('SELECT namespace,sha256,bytes FROM raw_captures'):
 p=pre['inputs'][c['namespace']];ck(c['sha256']==p['sha256'] and bytes(c['bytes'])==Path(p['path']).read_bytes(),'all_eight_exact_capture_bytes')
evsql='o.namespace,o.sha256,o.locator,se.field_index,se.citation_state,se.citation_value,se.offset_start,se.offset_end';join=' LEFT JOIN source_evidence se ON se.evidence_id=x.evidence_id LEFT JOIN occurrence_refs o ON o.id=se.occurrence_id'
summary={}
summary['contexts']=equal([tuple(x) for x in con.execute('SELECT x.assertion_id,x.kind,x.value_state,x.value_text,x.value_language_state,x.value_language_value,x.value_markup_state,x.value_markup_value,x.translated_from,'+evsql+' FROM contexts x'+join)],contexts,'complete_context_owner_value_evidence_multiset')
summary['forms']=equal([tuple(x) for x in con.execute('SELECT x.entry_id,x.spelling,x.kind,x.standing_state,x.standing_value,x.language_state,x.language_value,x.script_state,x.script_value,x.pronunciation_state,x.pronunciation_text,x.pronunciation_language_state,x.pronunciation_language_value,x.pronunciation_markup_state,x.pronunciation_markup_value,x.computation_id,'+evsql+' FROM forms x'+join)],forms,'complete_form_owner_standing_value_evidence_multiset')
summary['etymologies']=equal([tuple(x) for x in con.execute('SELECT x.assertion_id,x.statement_state,x.statement_text,x.statement_language_state,x.statement_language_value,x.statement_markup_state,x.statement_markup_value,x.native_kind_state,x.native_kind_value,'+evsql+' FROM etymologies x'+join)],etys,'complete_etymology_owner_value_evidence_multiset')
summary['extensions']=equal([tuple(x) for x in con.execute('SELECT x.owner_id,x.name,x.value,'+evsql+' FROM extensions x'+join)],ext,'complete_extension_owner_exact_lexeme_evidence_multiset')
summary['eligible']=equal([tuple(x) for x in con.execute("SELECT e.source_id,e.spelling_value,a.occurrence_id,a.id FROM assertions a JOIN entries e ON e.id=a.entry_id WHERE a.eligibility='Lexical'")],eligible,'complete_eligible_source_spelling_occurrence_assertion_tuples')
words=sorted({x[1] for x in eligible});wordhash=sha(json.dumps(words,ensure_ascii=False,separators=(',',':')).encode());ck(PACKS!={'lensisku-jbotci','llg','la-lojban','sutysisku'} or len(words)==31375 and wordhash=='8be3bdddb9c965b5993a979265f1f805cf1eb85d159a9b57c932fa92e681e8f5','source_derived_complete_word_union')
tables=[x[0] for x in con.execute("SELECT name FROM sqlite_schema WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name")];inventory={t:con.execute('SELECT count(*) FROM '+t).fetchone()[0] for t in tables};ck(tables==['annotations', 'argument_frames', 'argument_slots', 'assertion_timestamps', 'assertions', 'canonical_document', 'capture_refs', 'code_notices', 'collections', 'component_languages', 'components', 'computation_inputs', 'computations', 'content_notices', 'contexts', 'contributors', 'distributors', 'editorial_members', 'editorial_senses', 'entity_ids', 'entries', 'etymologies', 'etymology_components', 'extensions', 'form_components', 'forms', 'metadata', 'native_fields', 'occurrence_attributions', 'occurrence_refs', 'owned_heads', 'owned_revisions', 'raw_captures', 'raw_occurrences', 'redistribution_evidence', 'relations', 'release_assertions', 'release_captures', 'release_diagnostics', 'releases', 'revision_assertions', 'revision_tombstones', 'snapshot_bindings', 'source_aliases', 'source_evidence', 'source_lineage', 'sources'],'exact_47_table_name_set_including_empty_categories');
for t in ['argument_frames','argument_slots','etymology_components','form_components','relations','owned_revisions','revision_assertions','revision_tombstones','owned_heads','editorial_senses','editorial_members','computations','computation_inputs','annotations','assertion_timestamps']:ck(inventory[t]==0,'explicit_empty_category_'+t)
con.close();after=sha(DB.read_bytes());ck(before==after,'readonly_db_hash_unchanged');out={'selected_packs':sorted(PACKS),'database':str(DB),'verdict':'PASS','checks':dict(checks),'total_checks':sum(checks.values()),'source_policy_per_pack':stats,'native_fields':expected_fields,'occurrences':len(expected_keys),'typed_multiset_comparisons':summary,'eligible_words':len(words),'eligible_word_union_sha256':wordhash,'eligible_by_source':dict(Counter(x[0] for x in eligible)),'table_inventory_all47':inventory,'db_pre_sha256':before,'db_post_sha256':after};(R/'native-sql01.json').write_text(json.dumps(out,indent=2)+'\n');print(json.dumps(out,indent=2))
