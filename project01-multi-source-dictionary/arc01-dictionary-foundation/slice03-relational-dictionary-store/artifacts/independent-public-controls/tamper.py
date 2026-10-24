"""Prospective independent S03 tamper/constraint controls; no candidate run at authorship.
Each setup is committed on a fresh closed copy; failed setup is recorded separately.
"""
import hashlib,json,pathlib,shutil,sqlite3,subprocess,sys
BIN=pathlib.Path(sys.argv[1]); BASE=pathlib.Path(sys.argv[2]); ROOT=pathlib.Path(sys.argv[3]);ROOT.mkdir(exist_ok=True)
results=[]
def tree(p):return {str(x.relative_to(p)):hashlib.sha256(x.read_bytes()).hexdigest() for x in p.rglob('*') if x.is_file()}
def check(name,setup,expected):
    dest=ROOT/name;shutil.copytree(BASE,dest); db=dest/'dictionary.sqlite3'
    try:
        c=sqlite3.connect(db);c.execute('PRAGMA foreign_keys=OFF');setup(c);c.commit();c.close()
    except Exception as e:
        results.append({'name':name,'status':'SETUP_FAILED','detail':repr(e)});return
    before=tree(dest);p=subprocess.run([str(BIN),'open',str(dest)],capture_output=True,text=True);after=tree(dest)
    ok=p.returncode==0 and p.stdout.strip()==expected and before==after
    results.append({'name':name,'status':'PASS' if ok else 'FAIL','expected':expected,'exit':p.returncode,'stdout':p.stdout.strip(),'stderr':p.stderr.strip(),'disk_unchanged':before==after})
def sql(statement,params=()):return lambda c:c.execute(statement,params)
check('wrong-family',sql("UPDATE metadata SET value='other-family' WHERE key='format'"),'ERR InvalidStore Format')
check('wrong-version',sql("UPDATE metadata SET value='999' WHERE key='schema_version'"),'ERR InvalidStore SchemaVersion')
check('incomplete',sql("UPDATE metadata SET value='false' WHERE key='complete'"),'ERR InvalidStore Incomplete')
check('missing-family',sql("DELETE FROM metadata WHERE key='format'"),'ERR InvalidStore Format')
check('wrong-canonical-hash',sql("UPDATE metadata SET value=? WHERE key='canonical_sha256'",('0'*64,)),'ERR InvalidStore CanonicalBytes')
check('no-canonical-row',sql('DELETE FROM canonical_document'),'ERR InvalidStore CanonicalBytes')
def canonical(c,mutate):
    d=json.loads(c.execute('SELECT json FROM canonical_document').fetchone()[0]);mutate(d);raw=json.dumps(d,ensure_ascii=False,separators=(',',':')).encode();c.execute('UPDATE canonical_document SET json=?',(raw,));c.execute("UPDATE metadata SET value=? WHERE key='canonical_sha256'",(hashlib.sha256(raw).hexdigest(),))
check('malformed-canonical-model',lambda c:canonical(c,lambda d:d.update(version=999)),'ERR InvalidDictionary')
check('valid-canonical-stale-projection',lambda c:canonical(c,lambda d:d['metadata'].update(catalogue_revision=2)),'ERR InvalidStore Projection')
check('schema-extra-table',sql('CREATE TABLE unexpected_table(id INTEGER)'),'ERR InvalidStore SchemaShape')
check('schema-extra-column',sql('ALTER TABLE entries ADD COLUMN unexpected_column TEXT'),'ERR InvalidStore SchemaShape')
check('schema-extra-index',sql('CREATE INDEX unexpected_index ON sources(display_name)'),'ERR InvalidStore SchemaShape')
check('schema-extra-trigger',sql('CREATE TRIGGER unexpected_trigger AFTER UPDATE ON sources BEGIN SELECT 1; END'),'ERR InvalidStore SchemaShape')
check('schema-extra-view',sql('CREATE VIEW unexpected_view AS SELECT id FROM sources'),'ERR InvalidStore SchemaShape')
def drop_index(c):
    row=c.execute("SELECT name FROM sqlite_schema WHERE type='index' AND sql IS NOT NULL AND tbl_name='entries' ORDER BY name LIMIT 1").fetchone();assert row,'required spelling index absent';c.execute('DROP INDEX "'+row[0]+'"')
check('schema-missing-required-index',drop_index,'ERR InvalidStore SchemaShape')
check('foreign-key-violation',sql("UPDATE entries SET source_id='nonexistent-source' WHERE id='entry-a-farvi'"),'ERR InvalidStore ForeignKeys')
# Each valid-shaped scalar mutation preserves table populations, schema and canonical JSON.
# Tables without an unconstrained textual payload use a distinct valid ordinal/nullable shape.
mutations={
 'entity_ids':"UPDATE entity_ids SET ordinal=ordinal+1000 WHERE id='entry-a-farvi'",
 'sources':"UPDATE sources SET display_name=display_name||' changed' WHERE id='source-a'",
 'contributors':"UPDATE contributors SET display_name=display_name||' changed' WHERE id='editor-1'",
 'distributors':"UPDATE distributors SET display_name=display_name||' changed' WHERE id='distributor-a-d1'",
 'source_aliases':"UPDATE source_aliases SET evidence='changed' WHERE ordinal=0",
 'source_lineage':"UPDATE source_lineage SET evidence='changed' WHERE ordinal=0",
 'snapshot_bindings':"UPDATE snapshot_bindings SET ordinal=ordinal+1000 WHERE ordinal=0",
 'occurrence_attributions':"UPDATE occurrence_attributions SET native_assertion_id='changed' WHERE ordinal=0",
 'raw_captures':"UPDATE raw_captures SET source_name='changed' WHERE ordinal=0",
 'raw_occurrences':"UPDATE raw_occurrences SET raw_line='changed' WHERE ordinal=0 AND namespace='captured:a:d1'",
 'native_fields':"UPDATE native_fields SET raw_value='999' WHERE name='definitionid' AND occurrence_id=(SELECT occurrence_id FROM raw_occurrences WHERE namespace='captured:a:d1' AND ordinal=0)",
 'collections':"UPDATE collections SET display_name='changed' WHERE id='collection-a'",
 'components':"UPDATE components SET native_name='changed' WHERE id='component-a'",
 'component_languages':"UPDATE component_languages SET language='xx' WHERE component_id='component-a' AND ordinal=0",
 'releases':"UPDATE releases SET normalizer_version='changed' WHERE id='release-a'",
 'release_captures':"UPDATE release_captures SET ordinal=ordinal+1000 WHERE release_id='release-a' AND ordinal=0",
 'release_assertions':"UPDATE release_assertions SET ordinal=ordinal+1000 WHERE release_id='release-a' AND ordinal=0",
 'release_diagnostics':"UPDATE release_diagnostics SET message='changed' WHERE release_id='release-a' AND ordinal=0",
 'code_notices':"UPDATE code_notices SET text='changed' WHERE ordinal=0",
 'content_notices':"UPDATE content_notices SET text_value='changed' WHERE release_id='release-a' AND ordinal=0",
 'redistribution_evidence':"UPDATE redistribution_evidence SET evidence='changed' WHERE ordinal=0",
 'entries':"UPDATE entries SET spelling_value='changed' WHERE id='entry-a-farvi'",
 'assertions':"UPDATE assertions SET definition_text='changed' WHERE id='assertion-0'",
 'assertion_timestamps':"UPDATE assertion_timestamps SET value='999' WHERE assertion_id='assertion-0' AND ordinal=0",
 'source_evidence':"UPDATE source_evidence SET citation_value='changed' WHERE evidence_id=(SELECT min(evidence_id) FROM source_evidence)",
 'contexts':"UPDATE contexts SET value_text='changed' WHERE id='context-0'",
 'argument_frames':"UPDATE argument_frames SET assertion_id='assertion-0' WHERE id='frame-si-e'",
 'argument_slots':"UPDATE argument_slots SET native_label='changed' WHERE frame_id='frame-si-e' AND ordinal=0",
 'forms':"UPDATE forms SET spelling='changed' WHERE id='form-variant'",
 'form_components':"UPDATE form_components SET spelling='changed' WHERE ordinal=0",
 'etymologies':"UPDATE etymologies SET native_kind_value='changed' WHERE id='etymology-0'",
 'relations':"UPDATE relations SET direction='Reverse' WHERE id='relation-see'",
 'owned_revisions':"UPDATE owned_revisions SET timestamp_value='changed' WHERE id='revision-r1'",
 'revision_assertions':"UPDATE revision_assertions SET ordinal=ordinal+1000 WHERE revision_id='revision-r1' AND ordinal=0",
 'revision_tombstones':"UPDATE revision_tombstones SET ordinal=ordinal+1000 WHERE revision_id='revision-r2' AND ordinal=0",
 'owned_heads':"UPDATE owned_heads SET revision_id='revision-r2' WHERE source_id='source-o'",
 'editorial_senses':"UPDATE editorial_senses SET rationale_text='changed' WHERE id='editorial-farvi'",
 'editorial_members':"UPDATE editorial_members SET ordinal=ordinal+1000 WHERE editorial_sense_id='editorial-farvi' AND ordinal=0",
 'computations':"UPDATE computations SET tool_version='changed' WHERE id='computation-1'",
 'computation_inputs':"UPDATE computation_inputs SET literal='changed' WHERE computation_id='computation-1' AND input_kind='Literal'",
 'annotations':"UPDATE annotations SET value='null' WHERE id='annotation-1'",
 'extensions':"UPDATE extensions SET value='null' WHERE ordinal=0",
}
for table,statement in mutations.items():
    def mutate(c,s=statement):
        counts=dict(c.execute("SELECT name,0 FROM sqlite_schema WHERE type='table' AND name NOT LIKE 'sqlite_%'"))
        for t in counts:counts[t]=c.execute('SELECT count(*) FROM "'+t+'"').fetchone()[0]
        cursor=c.execute(s);assert cursor.rowcount>0,'mutation selected no rows'
        for t,n in counts.items():assert c.execute('SELECT count(*) FROM "'+t+'"').fetchone()[0]==n,'population changed'
    check('projection-'+table,mutate,'ERR InvalidStore Projection')
check('projection-extra-source',sql("INSERT INTO contributors(id,ordinal,display_name) VALUES('new-contributor',999,'extra')"),'ERR InvalidStore Projection')
check('projection-delete-attribution',sql('DELETE FROM occurrence_attributions WHERE ordinal=0'),'ERR InvalidStore Projection')
check('projection-extra-capture-ref',sql("INSERT INTO capture_refs(namespace,sha256) VALUES('catalogue-extra',?)",('1'*64,)),'ERR InvalidStore Projection')
# Genuine SQL constraint controls are independent from open failures. Fresh copy, foreign_keys ON.
constraints={
 'null-text-primary-key':("INSERT INTO contributors(id,ordinal,display_name) VALUES(NULL,999,'null-key')",1299),
 'null-capture-tuple-key':("INSERT INTO capture_refs(namespace,sha256) VALUES(NULL,'ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff')",1299),
 'nested-text-presence-null-state':("UPDATE contexts SET value_language_state=NULL WHERE id='context-0'",275),
 'outer-missing-extra-text':("UPDATE forms SET pronunciation_text='unexpected' WHERE id='form-variant'",275),
 'optional-local-extra-payload':("UPDATE form_components SET target_kind=NULL WHERE form_id='form-components' AND ordinal=0",275),
 'required-local-null-kind':("UPDATE relations SET origin_kind=NULL WHERE id='relation-see'",1299),
 'evidence-unpaired-range':("UPDATE source_evidence SET offset_start='0',offset_end=NULL WHERE evidence_id=(SELECT min(evidence_id) FROM source_evidence)",275),
 'unknown-native-encoding':("UPDATE native_fields SET encoding='Xml' WHERE ordinal=0",275),
 'excluded-null-reason':("UPDATE assertions SET exclusion_reason=NULL WHERE id='assertion-19'",275),
 'form-double-provenance':("UPDATE forms SET computation_id='computation-1' WHERE id='form-variant'",275),
 'relation-unexpected-local-target':("UPDATE relations SET local_target_kind='Entry',local_target_entry_id='entry-a-si-e' WHERE id='relation-see'",275),
 'source-fk':("UPDATE entries SET source_id='nonexistent-source' WHERE id='entry-a-farvi'",787),
 'contributor-fk':("UPDATE occurrence_attributions SET contributor_id='nonexistent-contributor' WHERE ordinal=0",787),
 'distributor-fk':("UPDATE snapshot_bindings SET distributor_id='nonexistent-distributor' WHERE ordinal=0",787),
 'capture-fk':("UPDATE snapshot_bindings SET sha256='"+'f'*64+"' WHERE ordinal=0",787),
 'occurrence-fk':("UPDATE assertions SET occurrence_id='unknown-occurrence' WHERE id='assertion-0'",787),
 'parent-fk':("UPDATE owned_revisions SET parent_id='unknown-revision' WHERE id='revision-r2'",787),
 'member-fk':("UPDATE editorial_members SET assertion_id='unknown-assertion' WHERE ordinal=0",787),
 'computation-fk':("UPDATE forms SET computation_id='unknown-computation' WHERE computation_id IS NOT NULL",787),
 'evidence-fk':("UPDATE contexts SET evidence_id=999999 WHERE id='context-0'",787),
 'entry-role-fk':("UPDATE entries SET id='assertion-0' WHERE id='entry-a-farvi'",787),
 'local-role-fk':("UPDATE relations SET origin_entry_id='assertion-0' WHERE id='relation-see'",787),
 'local-null-shape':("UPDATE relations SET origin_entry_id=NULL WHERE id='relation-see'",275),
 'presence-null-shape':("UPDATE entries SET spelling_value=NULL WHERE id='entry-a-farvi'",275),
 'presence-extra-payload':("UPDATE entries SET script_value='unexpected' WHERE id='entry-a-farvi'",275),
 'role-constant':("UPDATE entries SET entity_role='assertion' WHERE id='entry-a-farvi'",275),
 'duplicate-source-ordinal':("UPDATE sources SET ordinal=0 WHERE id='source-b'",2067),
 'duplicate-native-field-ordinal':("UPDATE native_fields SET ordinal=0 WHERE ordinal=1 AND occurrence_id=(SELECT occurrence_id FROM raw_occurrences WHERE namespace='captured:a:d1' AND ordinal=0)",1555),
 'negative-vector-ordinal':("UPDATE component_languages SET ordinal=-1 WHERE component_id='component-a' AND ordinal=0",275),
 'u32-overflow':("UPDATE releases SET schema_version=4294967296 WHERE id='release-a'",275),
 'target-exclusive-shape':("UPDATE relations SET unresolved_text='unexpected' WHERE id='relation-see'",275),
}
for name,(statement,code) in constraints.items():
    dest=ROOT/('constraint-'+name);shutil.copytree(BASE,dest);c=sqlite3.connect(dest/'dictionary.sqlite3');c.execute('PRAGMA foreign_keys=ON')
    try:
        cur=c.execute(statement);assert cur.rowcount>0,'control selected no rows';c.commit();results.append({'name':'constraint-'+name,'status':'FAIL','detail':'mutation accepted'})
    except sqlite3.IntegrityError as e:
        got=e.sqlite_errorcode;results.append({'name':'constraint-'+name,'status':'PASS' if got==code else 'FAIL','expected_sqlite_code':code,'actual_sqlite_code':got,'detail':str(e)})
    except Exception as e:results.append({'name':'constraint-'+name,'status':'SETUP_FAILED','detail':repr(e)})
    finally:c.close()
print(json.dumps({'controls':len(results),'passed':sum(r['status']=='PASS' for r in results),'results':results},ensure_ascii=False,indent=2));sys.exit(any(r['status']!='PASS' for r in results))
