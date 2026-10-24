"""Prospective independent public filesystem/error/snapshot controls."""
import hashlib,json,pathlib,sqlite3,subprocess,sys
binary=pathlib.Path(sys.argv[1]);fixture=pathlib.Path(sys.argv[2]);valid=pathlib.Path(sys.argv[3]);root=pathlib.Path(sys.argv[4]);root.mkdir(exist_ok=True)
results=[]
def tree(path):
 if path.is_symlink():return {'symlink':str(path.readlink())}
 if path.is_file():return {'file':hashlib.sha256(path.read_bytes()).hexdigest()}
 if not path.exists():return None
 return {str(p.relative_to(path)):hashlib.sha256(p.read_bytes()).hexdigest() for p in path.rglob('*') if p.is_file()}
def run(name,args,expected,path):
 before=tree(path);p=subprocess.run([str(binary),*map(str,args)],capture_output=True,text=True);after=tree(path)
 good=p.returncode==0 and p.stdout.strip()==expected and before==after
 results.append({'name':name,'status':'PASS' if good else 'FAIL','expected':expected,'exit':p.returncode,'stdout':p.stdout.strip(),'stderr':p.stderr.strip(),'unchanged':before==after})
initial_valid=tree(valid)
run('missing-open', ['open',root/'absent'],'ERR InvalidStore MissingDatabase',root/'absent')
(root/'empty-directory').mkdir();run('empty-directory-open',['open',root/'empty-directory'],'ERR InvalidStore MissingDatabase',root/'empty-directory')
(root/'existing-file').write_bytes(b'keep caller file')
run('existing-file-create',['create',fixture,root/'existing-file'],'ERR ExistingPath',root/'existing-file')
run('existing-directory-create',['create',fixture,root/'empty-directory'],'ERR ExistingPath',root/'empty-directory')
(root/'dangling-link').symlink_to(root/'missing-link-target')
run('dangling-link-create',['create',fixture,root/'dangling-link'],'ERR ExistingPath',root/'dangling-link')
(root/'valid-link').symlink_to(valid,target_is_directory=True)
run('existing-link-create',['create',fixture,root/'valid-link'],'ERR ExistingPath',root/'valid-link')
run('existing-valid-store-create',['create',fixture,valid],'ERR ExistingPath',valid)
run('missing-parent-create',['create',fixture,root/'missing-parent'/'leaf'],'ERR Io',root/'missing-parent')
(root/'malformed-file').mkdir();(root/'malformed-file'/'dictionary.sqlite3').write_bytes(b'this is not a SQLite database')
run('malformed-database-open',['open',root/'malformed-file'],'ERR Backend',root/'malformed-file')
(root/'foreign-family').mkdir();c=sqlite3.connect(root/'foreign-family'/'dictionary.sqlite3');c.execute('CREATE TABLE metadata(key TEXT PRIMARY KEY,value TEXT NOT NULL)');c.execute("INSERT INTO metadata VALUES('schema_version','1')");c.commit();c.close()
run('foreign-family-open',['open',root/'foreign-family'],'ERR InvalidStore Format',root/'foreign-family')
# A successful opener must retain the pinned read transaction, not only a parsed in-memory model.
# Rollback-journal EXCLUSIVE cannot acquire while that genuine read snapshot is alive.
p=subprocess.Popen([str(binary),'hold',str(valid)],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
ready=p.stdout.readline().strip();locked=False;detail=''
try:
 c=sqlite3.connect(valid/'dictionary.sqlite3',timeout=0)
 try:c.execute('BEGIN EXCLUSIVE');c.rollback()
 except sqlite3.OperationalError as e:locked=e.sqlite_errorcode==sqlite3.SQLITE_BUSY;detail=str(e)
 finally:c.close()
finally:
 p.stdin.write('\n');p.stdin.flush();stdout,stderr=p.communicate(timeout=30)
results.append({'name':'retained-read-snapshot','status':'PASS' if ready.startswith('OK HOLD ') and locked and p.returncode==0 else 'FAIL','ready':ready,'exclusive_busy':locked,'detail':detail,'exit':p.returncode,'stdout':stdout,'stderr':stderr})
# Releasing the caller must release the read lock; no writer mutation is needed.
c=sqlite3.connect(valid/'dictionary.sqlite3',timeout=0)
try:c.execute('BEGIN EXCLUSIVE');c.rollback();results.append({'name':'drop-releases-read-snapshot','status':'PASS'})
except Exception as e:results.append({'name':'drop-releases-read-snapshot','status':'FAIL','detail':repr(e)})
finally:c.close()
results.append({'name':'all-filesystem-controls-preserve-valid-store','status':'PASS' if tree(valid)==initial_valid else 'FAIL'})
print(json.dumps({'controls':len(results),'passed':sum(r['status']=='PASS' for r in results),'results':results},indent=2));sys.exit(any(r['status']!='PASS' for r in results))
