"""Initial-candidate scalar diagnostic ONLY: known bound schema deviations remain failures."""
import contextlib,importlib.util,io,json,pathlib,sys
spec=importlib.util.spec_from_file_location('oracle', '/tmp/vlajmaji-p01-a01-s03-cdc-sql-oracle.py');o=importlib.util.module_from_spec(spec);spec.loader.exec_module(o)
original=o.build
def diagnostic_build(d,raw):
 rows=original(d,raw);ordinals={r['id']:r['ordinal'] for r in rows['entity_ids']}
 for table,rr in rows.items():
  for r in rr:
   if 'entity_role' in r:r['ordinal']=ordinals[r['id']]
   if table=='raw_occurrences':r['id']=r.pop('occurrence_id')
   if table=='relations':
    for key in list(r):
     if key.startswith('local_target_'):r['local_'+key[len('local_target_'):]]=r.pop(key)
 return rows
o.build=diagnostic_build;buffer=io.StringIO();code=0
with contextlib.redirect_stdout(buffer):
 try:o.main()
 except SystemExit as e:code=e.code
r=json.loads(buffer.getvalue());r['diagnostic_only']=True;r['acceptance_eligible']=False;r['structural_shims']=['14 extra role ordinals','raw_occurrences occurrence_id renamed id','relations local_target prefix renamed local']
print(json.dumps(r,ensure_ascii=False,indent=2));sys.exit(code)
