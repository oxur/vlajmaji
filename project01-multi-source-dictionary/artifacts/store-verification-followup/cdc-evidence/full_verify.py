from pathlib import Path
import hashlib,json,os,sqlite3,subprocess,time
R=Path('/tmp/vlajmaji-store-verification-cdc');DB=Path('/Users/oubiwann/.local/share/vlajmaji/stores/default/dictionary.sqlite3');B=R/'candidate01-release-cli';EXPECTED='fe684fd52ef09aea9cba41bbdc97e57529199d66fdd4c04932bbe05e7131bf7d'
def sha(p):
 h=hashlib.sha256()
 with p.open('rb') as f:
  for x in iter(lambda:f.read(1048576),b''):h.update(x)
 return h.hexdigest()
def state():
 con=sqlite3.connect(DB.as_uri()+'?mode=ro',uri=True)
 row=con.execute('select json from canonical_document where id=1').fetchone();m=dict(con.execute('select key,value from metadata'));con.close()
 st=DB.stat();return {'sha256':sha(DB),'bytes':st.st_size,'inode':st.st_ino,'device':st.st_dev,'nlink':st.st_nlink,'canonical_bytes':len(row[0]),'canonical_sha256':hashlib.sha256(row[0]).hexdigest(),'metadata':m,'directory_entries':sorted(x.name for x in DB.parent.iterdir())}
x=os.environ.get('XDG_DATA_HOME');h=os.environ.get('HOME');base=Path(x) if x and Path(x).is_absolute() else Path(h)/'.local/share' if h and Path(h).is_absolute() else None
assert base/'vlajmaji/stores/default/dictionary.sqlite3'==DB
before=state();assert before['sha256']==EXPECTED
argv=[str(B),'verify','all'];t=time.monotonic()
with (R/'full-verify01.stdout').open('xb') as out,(R/'full-verify01.stderr').open('xb') as err:
 p=subprocess.Popen(argv,stdout=out,stderr=err,stdin=subprocess.DEVNULL)
 (R/'full-verify01-active.json').write_text(json.dumps({'argv':argv,'pid':p.pid,'before':before,'HOME':h,'XDG_DATA_HOME':x},indent=2)+'\n')
 code=p.wait(timeout=180)
elapsed=time.monotonic()-t;after=state();report=json.loads((R/'full-verify01.stdout').read_bytes())
assert code==0 and not (R/'full-verify01.stderr').read_bytes() and report['ok'] and len(report['checks'])==6 and all(r['status']=='passed' for r in report['checks'])
assert before==after
result={'verdict':'PASS','source':'53f9eacece01bb5f31ff4b3c2bcf270d7c4162e6','binary_sha256':sha(B),'argv':argv,'pid':p.pid,'exit':code,'elapsed_s':elapsed,'before':before,'after':after,'report':report,'stdout_sha256':sha(R/'full-verify01.stdout'),'stderr_sha256':sha(R/'full-verify01.stderr')};(R/'full-verify01.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result,indent=2))
