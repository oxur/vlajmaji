"""Supplemental frozen-source SQL union/Presence controls; each fresh closed valid copy."""
import json,pathlib,shutil,sqlite3,subprocess,sys
binary=pathlib.Path(sys.argv[1]);fixture=pathlib.Path(sys.argv[2]);role_store=pathlib.Path(sys.argv[3]);root=pathlib.Path(sys.argv[4]);root.mkdir()
x=json.loads(fixture.read_bytes());refs=[{'Entry':'entry-a-si-e'},{'Assertion':'assertion-5'},{'Form':'form-variant'},{'EditorialSense':'editorial-farvi'}]
x['metadata']['computations'][0]['inputs'].extend({'Entity':r} for r in refs);inp=root/'all-entity-inputs.json';inp.write_text(json.dumps(x,ensure_ascii=False,separators=(',',':')))
p=subprocess.run([str(binary),'create',str(inp),str(root/'entity-store')],capture_output=True,text=True);assert p.returncode==0 and p.stdout.startswith('OK CREATE'),p.stdout+p.stderr
main=root/'entity-store';cases=[]
cases.extend([('collection-inner-null',main,"UPDATE collections SET description_language_state=NULL WHERE id='collection-a'"),('assertion-inner-null',main,"UPDATE assertions SET definition_language_state=NULL WHERE id='assertion-0'")])
for t,parent,key in [('form_components','form_id','form-variant'),('etymology_components','etymology_id','etymology-0')]:
 for n in range(4):cases.append((t+'-null-kind-'+str(n),role_store,f"UPDATE {t} SET target_kind=NULL WHERE {parent}='{key}' AND ordinal={n}"))
for n in range(2,6):cases.append(('entity-input-null-kind-'+str(n-2),main,f"UPDATE computation_inputs SET entity_kind=NULL WHERE computation_id='computation-1' AND ordinal={n}"))
c=sqlite3.connect(main/'dictionary.sqlite3');cols={r[1] for r in c.execute('PRAGMA table_info(relations)')};c.close();local='local_target_kind' if 'local_target_kind' in cols else 'local_kind'
cases.extend([('relation-null-local-kind',main,f"UPDATE relations SET {local}=NULL WHERE id='relation-editorial'"),('catalogue-empty-identity',main,"UPDATE relations SET catalog_native_entry_id=NULL,catalog_native_assertion_id=NULL,catalog_occurrence_id=NULL WHERE id='relation-missing-catalogue'"),('other-kind-null-payload',main,"UPDATE relations SET other_kind=NULL WHERE id='relation-unresolved'"),('source-kind-extra-other-payload',main,"UPDATE relations SET other_kind='unexpected' WHERE id='relation-see'")])
results=[]
for name,source,statement in cases:
 dest=root/name;shutil.copytree(source,dest);c=sqlite3.connect(dest/'dictionary.sqlite3');c.execute('PRAGMA foreign_keys=ON')
 try:
  cur=c.execute(statement);assert cur.rowcount==1,'setup selected wrong row count';c.commit();results.append({'name':name,'status':'FAIL','actual':'invalid shape accepted','sql':statement})
 except sqlite3.IntegrityError as e:results.append({'name':name,'status':'PASS' if e.sqlite_errorcode==275 else 'FAIL','expected_sqlite_code':275,'actual_sqlite_code':e.sqlite_errorcode,'detail':str(e),'sql':statement})
 except Exception as e:results.append({'name':name,'status':'SETUP_FAILED','detail':repr(e),'sql':statement})
 finally:c.close()
print(json.dumps({'controls':len(results),'passed':sum(r['status']=='PASS' for r in results),'relation_bound_name_present':local=='local_target_kind','results':results},indent=2));sys.exit(any(r['status']!='PASS' for r in results))
