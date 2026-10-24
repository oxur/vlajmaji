"""Frozen S03 metadata query/category controls; each setup succeeds on a fresh closed copy."""
import json,pathlib,shutil,sqlite3,subprocess,sys
binary=pathlib.Path(sys.argv[1]);base=pathlib.Path(sys.argv[2]);root=pathlib.Path(sys.argv[3]);root.mkdir();rows=[]
for name,sql in [('metadata-no-value-column','ALTER TABLE metadata RENAME COLUMN value TO other_value'),('metadata-no-key-column','ALTER TABLE metadata RENAME COLUMN key TO other_key'),('metadata-no-table','DROP TABLE metadata')]:
 dest=root/name;shutil.copytree(base,dest);db=dest/'dictionary.sqlite3';c=sqlite3.connect(db);c.execute(sql);c.commit();c.close();before=db.read_bytes();p=subprocess.run([str(binary),'open',str(dest)],capture_output=True,text=True);actual=p.stdout.strip();unchanged=db.read_bytes()==before;rows.append({'name':name,'sql':sql,'expected':'ERR InvalidStore Format','actual':actual,'exit':p.returncode,'unchanged':unchanged,'pass':p.returncode==0 and actual=='ERR InvalidStore Format' and unchanged})
print(json.dumps({'controls':len(rows),'passed':sum(r['pass'] for r in rows),'rows':rows},indent=2));sys.exit(any(not r['pass'] for r in rows))
