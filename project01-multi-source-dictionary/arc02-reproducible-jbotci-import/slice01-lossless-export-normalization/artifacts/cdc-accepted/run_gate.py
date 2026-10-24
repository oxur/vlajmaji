#!/usr/bin/env python3
"""Run one serialized CDC gate; retain exact command/environment/exit/elapsed/log."""
import json,os,subprocess,sys,time
from pathlib import Path
root=Path(__file__).resolve().parent
label=sys.argv[1];cwd=sys.argv[2];args=sys.argv[3:]
assert args and '/' not in label
log=root/(label+'.log');record=root/(label+'.json')
assert not log.exists() and not record.exists()
env=os.environ.copy();env.update(CARGO_TARGET_DIR='/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc01/target',CARGO_NET_OFFLINE='true',PYTHONDONTWRITEBYTECODE='1')
start=time.time();head=subprocess.check_output(['git','-C',cwd,'rev-parse','HEAD'],text=True).strip() if (Path(cwd)/'.git').exists() else None
r={'label':label,'cwd':cwd,'argv':args,'environment':{k:env[k] for k in ['CARGO_TARGET_DIR','CARGO_NET_OFFLINE','PYTHONDONTWRITEBYTECODE']},'head_before':head,'start_unix':start,'log':str(log)}
with log.open('xb') as f:
 p=subprocess.Popen(args,cwd=cwd,env=env,stdout=f,stderr=subprocess.STDOUT)
 r['pid']=p.pid;record.write_text(json.dumps(r,indent=2)+'\n')
 print(json.dumps({'started':label,'pid':p.pid,'log':str(log)}),flush=True)
 code=p.wait()
r.update(exit=code,elapsed_seconds=round(time.time()-start,3),log_bytes=log.stat().st_size)
record.write_text(json.dumps(r,indent=2)+'\n')
print(json.dumps(r,indent=2),flush=True)
print(log.read_text(errors='replace')[-6500:],flush=True)
raise SystemExit(code)
