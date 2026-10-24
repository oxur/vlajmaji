"""CLI stream and unchanged legacy behavior controls, independent of new query helpers."""
import collections,json,pathlib,subprocess,sys
DOC,STORE,CLI,ROOT=sys.argv[1:];root=pathlib.Path(ROOT);root.mkdir();d=json.loads(pathlib.Path(DOC).read_bytes());records=[]
def check(name,fn):
 try:r=fn();records.append(dict(name=name,status='PASS',details=r))
 except Exception as e:records.append(dict(name=name,status='FAIL',detail=str(e)))
def run(args,stdin=None):
 r=subprocess.run([CLI,*[str(x) for x in args]],input=stdin,capture_output=True,text=True);assert r.returncode==0,(r.returncode,r.stdout,r.stderr);assert not r.stderr.strip();return json.loads(r.stdout)
archive=root/'legacy.archive';archive.write_bytes(bytes(d['legacy_archive']));legacy=root/'legacy-store'
check('legacy-build',lambda:run(['build',archive,legacy]))
check('legacy-info',lambda: {'records':run(['info',legacy])['records']})
for size in [None,0,1,1001]:
 def search(size=size):
  args=['search',legacy]+([] if size is None else ['--limit',str(size)])
  v=run(args,json.dumps(dict(operation='exact',spelling='farvi')));assert v['total_records']==7,v['total_records'];assert len(v['hits'])==(7 if size is None else min(size,7)),len(v['hits']);return dict(total=v['total_records'],returned=len(v['hits']))
 check('legacy-search:'+str(size),search)
def pipe():
 p=subprocess.Popen([CLI,'dictionary-query',STORE,'--request','{"operation":"exact","spelling":"farvi"}'],stdout=subprocess.PIPE,stderr=subprocess.PIPE)
 p.stdout.close();status=p.wait(timeout=30);stderr=p.stderr.read().decode();p.stderr.close();assert status==0 and stderr=='',(status,stderr);return dict(exit=status)
check('new-cli-broken-pipe',pipe)
pathlib.Path(root/'results.json').write_text(json.dumps(dict(records=records,summary=dict(collections.Counter(r['status'] for r in records))),indent=2)+'\n');print(json.dumps(dict(collections.Counter(r['status'] for r in records))));sys.exit(any(r['status']!='PASS' for r in records))
