import subprocess,sys,json,time,hashlib,os
from pathlib import Path
R=Path('/tmp/vlajmaji-cli-default-store-cdc');S=Path('/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-cli-default-store/vlajmaji')
label=sys.argv[1];args=sys.argv[2:];env=os.environ.copy();env.update(CARGO_TARGET_DIR='/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc01/target',CARGO_NET_OFFLINE='true')
head=subprocess.check_output(['git','rev-parse','HEAD'],cwd=S,text=True).strip();status=subprocess.check_output(['git','status','--porcelain'],cwd=S,text=True);start=time.monotonic()
with (R/(label+'.stdout')).open('xb') as out,(R/(label+'.stderr')).open('xb') as err:
 p=subprocess.Popen(args,cwd=S,env=env,stdout=out,stderr=err);code=p.wait(timeout=300)
r={'argv':args,'cwd':str(S),'pid':p.pid,'exit':code,'elapsed_s':time.monotonic()-start,'source_head':head,'source_status_before':status,'source_head_after':subprocess.check_output(['git','rev-parse','HEAD'],cwd=S,text=True).strip(),'target':env['CARGO_TARGET_DIR'],'offline':True,'profile':'release' if '--release' in args else 'command-specific','streams':{}}
for stream in ['stdout','stderr']:
 b=(R/(label+'.'+stream)).read_bytes();r['streams'][stream]={'bytes':len(b),'sha256':hashlib.sha256(b).hexdigest()}
(R/(label+'.json')).write_text(json.dumps(r,indent=2)+'\n');print(json.dumps(r,indent=2));sys.exit(code)
