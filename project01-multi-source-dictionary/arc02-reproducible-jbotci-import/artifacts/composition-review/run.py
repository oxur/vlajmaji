from pathlib import Path
import subprocess,json,hashlib,datetime,time,sys,os
root=Path('/tmp/vlajmaji-p01-a02-arc-review/f20261024');label=sys.argv[1];seconds=int(sys.argv[2]);argv=sys.argv[3:];start=time.monotonic()
out={'argv':argv,'cwd':os.getcwd(),'start_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'environment':{k:os.environ[k] for k in ('CARGO_TARGET_DIR','CARGO_NET_OFFLINE','CURL','VLAJMAJI_BIN') if k in os.environ}}
with (root/(label+'.stdout')).open('xb') as so,(root/(label+'.stderr')).open('xb') as se:
 p=subprocess.Popen(argv,stdout=so,stderr=se);out['pid']=p.pid;(root/(label+'.running.json')).write_text(json.dumps(out,indent=2))
 try:out['exit']=p.wait(timeout=seconds)
 except subprocess.TimeoutExpired:
  out['timeout_s']=seconds;p.terminate()
  try:p.wait(timeout=10)
  except subprocess.TimeoutExpired:p.kill();p.wait()
  out['exit']=p.returncode
out['elapsed_s']=time.monotonic()-start;out['end_utc']=datetime.datetime.now(datetime.timezone.utc).isoformat();out['streams']={}
for suffix in ['stdout','stderr']:
 f=root/(label+'.'+suffix);b=f.read_bytes();out['streams'][suffix]={'bytes':len(b),'sha256':hashlib.sha256(b).hexdigest()}
(root/(label+'.json')).write_text(json.dumps(out,indent=2)+'\n');print(json.dumps(out,indent=2));sys.exit(0 if out['exit']==0 else 1)
