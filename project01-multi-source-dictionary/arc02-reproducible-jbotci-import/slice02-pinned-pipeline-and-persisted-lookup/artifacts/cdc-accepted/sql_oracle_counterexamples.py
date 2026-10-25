#!/usr/bin/env python3
"""Two finite, task-copy-only negative controls for the raw→SQL oracle.
Run only after the exact original full store passes; never opens it writable.
"""
import hashlib,json,os,shutil,sqlite3,subprocess,sys
from pathlib import Path
raw,meta,original,out=map(Path,sys.argv[1:5]);out.mkdir(exist_ok=False)
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
before=sha(original);results=[]
for name,category in [('whole-gloss-field','native_extensions'),('native-score-lexeme','all_native_fields')]:
 copy=out/(name+'.sqlite3');shutil.copyfile(original,copy);assert sha(copy)==before
 db=sqlite3.connect(copy);db.execute('PRAGMA foreign_keys=ON')
 if name=='whole-gloss-field':
  row=db.execute("SELECT ordinal,owner_id,name,value,evidence_id FROM extensions WHERE name='lensisku.gloss_keywords.raw' AND value<>'[]' ORDER BY ordinal LIMIT 1").fetchone();assert row
  db.execute('UPDATE extensions SET value=? WHERE ordinal=?',('[]',row[0]));changed={'table':'extensions','ordinal':row[0],'owner_id':row[1],'name':row[2],'before':row[3],'after':'[]','evidence_id':row[4]}
 else:
  row=db.execute("SELECT f.occurrence_id,f.ordinal,f.raw_value,o.locator FROM native_fields f JOIN occurrence_refs o ON o.id=f.occurrence_id WHERE f.name='score' ORDER BY o.locator COLLATE BINARY LIMIT 1").fetchone();assert row and row[2]!='99999999999999'
  db.execute('UPDATE native_fields SET raw_value=? WHERE occurrence_id=? AND ordinal=?',('99999999999999',row[0],row[1]));changed={'table':'native_fields','occurrence_id':row[0],'ordinal':row[1],'locator':row[3],'before':row[2],'after':'99999999999999'}
 db.commit();assert db.execute('PRAGMA foreign_key_check').fetchall()==[];assert db.execute('PRAGMA integrity_check').fetchall()==[('ok',)];db.close()
 command=[sys.executable,'-B',str(Path(__file__).parent/'verify_sqlite.py'),str(raw),str(meta),str(copy)]
 process=subprocess.run(command,capture_output=True,env={**os.environ,'PYTHONDONTWRITEBYTECODE':'1'})
 (out/(name+'.stdout')).write_bytes(process.stdout);(out/(name+'.stderr')).write_bytes(process.stderr)
 assert process.returncode!=0 and ('AssertionError: '+category+':').encode() in process.stderr,(name,process.returncode,process.stderr)
 assert sha(original)==before
 results.append({'case':name,'intentional_mutation':changed,'argv':command,'returncode':process.returncode,'expected_failure_category':category,'original_sha256':before,'mutated_copy_sha256':sha(copy),'original_unchanged':True})
(out/'result.json').write_text(json.dumps({'verdict':'PASS','negative_controls':2,'cases':results},indent=2)+'\n');print((out/'result.json').read_text())
