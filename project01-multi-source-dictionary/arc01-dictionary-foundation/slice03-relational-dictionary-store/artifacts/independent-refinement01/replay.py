import pathlib,json,subprocess,shutil,sqlite3,hashlib,sys
s=pathlib.Path('/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/planning/project01-multi-source-dictionary/arc01-dictionary-foundation/slice03-relational-dictionary-store');a=s/'artifacts';controls=a/'independent-public-controls';inputs=a/'independent-initial/inputs';root=pathlib.Path('/tmp/vlajmaji-p01-a01-s03-cdc-r01-replay');root.mkdir();binary='/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc01/target/debug/dictionary-store-s03-cdc-r01-controls';rows=[]
def run(name,args,expected=None):
 p=subprocess.run(list(map(str,args)),capture_output=True,text=True);(root/(name+'.log')).write_text(p.stdout+p.stderr);ok=p.returncode==0 and (expected is None or p.stdout.strip().startswith(expected));row={'name':name,'exit':p.returncode,'pass':ok,'stdout':p.stdout.strip() if expected is not None else None};rows.append(row)
 if expected is None:
  try:(root/(name+'.json')).write_text(json.dumps(json.loads(p.stdout),indent=2)+'\n')
  except json.JSONDecodeError:pass
 return ok
base=root/'main';assert run('main-create',[binary,'create',inputs/'accepted-dictionary.json',base],'OK CREATE 132946')
oracle=a/'independent-sql-oracle/oracle.py'
run('main-sql',['python3',oracle,base.with_extension('canonical.json') if False else base.with_suffix('.canonical.json'),base/'dictionary.sqlite3'])
run('tamper',['python3',controls/'tamper.py',binary,base,root/'tamper-cases'])
run('filesystem',['python3',controls/'filesystem.py',binary,inputs/'accepted-dictionary.json',base,root/'filesystem-cases'])
run('canonical-shapes',['python3',controls/'canonical-shapes.py',binary,base,root/'canonical-cases'])
variant_root=root/'variants';variant_root.mkdir()
for directory in ['positive-calibrated','additional']:
 for inp in sorted((inputs/directory).glob('*.json')):
  name=inp.stem;dest=variant_root/name
  if run('variant-'+name,[binary,'create',inp,dest],'OK CREATE '):run('sql-'+name,['python3',oracle,dest.with_suffix('.canonical.json'),dest/'dictionary.sqlite3'])
run('additional-shapes',['python3',controls/'additional-shapes.py',binary,inputs/'accepted-dictionary.json',variant_root/'all-component-roles',root/'shape-cases'])
for name,inp in [('v2',inputs/'accepted-dictionary.json'),('v1',inputs/'positive-calibrated/legacy-v1.json'),('empty-metadata',inputs/'positive-calibrated/legacy-empty-metadata.json')]:run('legacy-'+name,[binary,'legacy',inp,root/('legacy-'+name)],'OK LEGACY 8 captures 19 assertions unchanged schema1/archive')
extra=[]
for name,sql,expected in [('metadata-no-value-column','ALTER TABLE metadata RENAME COLUMN value TO other_value','ERR InvalidStore Format'),('metadata-no-key-column','ALTER TABLE metadata RENAME COLUMN key TO other_key','ERR InvalidStore Format'),('metadata-no-table','DROP TABLE metadata','ERR InvalidStore Format')]:
 dest=root/name;shutil.copytree(base,dest);db=dest/'dictionary.sqlite3';c=sqlite3.connect(db);c.execute(sql);c.commit();c.close();before=db.read_bytes();p=subprocess.run([binary,'open',str(dest)],capture_output=True,text=True);extra.append({'name':name,'sql':sql,'expected':expected,'actual':p.stdout.strip(),'exit':p.returncode,'unchanged':db.read_bytes()==before,'pass':p.returncode==0 and p.stdout.strip()==expected and db.read_bytes()==before})
(root/'metadata-query-cases.json').write_text(json.dumps({'controls':len(extra),'passed':sum(r['pass'] for r in extra),'rows':extra},indent=2)+'\n')
(root/'summary.json').write_text(json.dumps({'source':'b29ac8c7dfcf37c8eaa69f538532a7e44c07a872','runs':rows,'additional_metadata_query_controls':extra},indent=2)+'\n');print(json.dumps({'runs':len(rows),'passed':sum(r['pass'] for r in rows),'failed_runs':[r for r in rows if not r['pass']],'metadata_query_controls':extra,'root':str(root)},indent=2));sys.exit(any(not r['pass'] for r in rows) or any(not r['pass'] for r in extra))
