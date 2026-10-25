#!/usr/bin/env python3
"""Serial independent process recorder with separate exact stdout/stderr streams."""
import datetime,hashlib,json,os,subprocess,sys,time
from pathlib import Path
root=Path('/tmp/vlajmaji-p01-a02-cdc');label=sys.argv[1];argv=sys.argv[2:]
assert argv and all(c.isalnum() or c in '-_' for c in label)
paths={kind:root/(label+'.'+kind) for kind in ['stdout','stderr','json']}
assert all(not p.exists() for p in paths.values()),'attempt label already used; choose a fresh label'
env=os.environ.copy();env.update({'CARGO_TARGET_DIR':'/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc01/target','CARGO_NET_OFFLINE':'true','PYTHONDONTWRITEBYTECODE':'1'})
cwd=env.pop('CDC_RUN_CWD','/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc02/vlajmaji')
def githead():
 p=subprocess.run(['git','rev-parse','HEAD'],cwd=cwd,env={**env,'GIT_OPTIONAL_LOCKS':'0'},capture_output=True,text=True)
 return p.stdout.strip() if p.returncode==0 else None
r={'label':label,'argv':argv,'cwd':cwd,'start_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'head_before':githead(),'environment':{k:env[k] for k in ['CARGO_TARGET_DIR','CARGO_NET_OFFLINE','VLAJMAJI_BIN','CURL','CARGO','RUST_TOOLCHAIN'] if k in env},'state':'starting'}
start=time.monotonic()
with paths['stdout'].open('xb') as stdout,paths['stderr'].open('xb') as stderr:
 p=subprocess.Popen(argv,cwd=cwd,env=env,stdout=stdout,stderr=stderr)
 r.update(pid=p.pid,state='running');paths['json'].write_text(json.dumps(r,indent=2)+'\n');print(json.dumps(r),flush=True)
 rc=p.wait()
r.update(state='completed',returncode=rc,elapsed_seconds=time.monotonic()-start,end_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),head_after=githead())
r['streams']={kind:{'path':str(paths[kind]),'bytes':paths[kind].stat().st_size,'sha256':hashlib.sha256(paths[kind].read_bytes()).hexdigest()} for kind in ['stdout','stderr']}
paths['json'].write_text(json.dumps(r,indent=2)+'\n');print(json.dumps(r),flush=True);sys.exit(rc)
