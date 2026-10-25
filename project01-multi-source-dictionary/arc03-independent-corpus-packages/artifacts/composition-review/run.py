from pathlib import Path
import os,sys,subprocess,json,time,hashlib
ROOT=Path('/tmp/vlajmaji-p01-a03-arc-review');name=sys.argv[1];cmd=sys.argv[2:];env=os.environ.copy();env.update(CARGO_TARGET_DIR='/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc01/target',CARGO_NET_OFFLINE='true');env.pop('RELOJBAN_RESEARCH_ROOT',None)
log=ROOT/'logs'/(name+'.log');receipt=ROOT/(name+'-receipt.json');start=time.monotonic();stage='process_start';stage_start=start;seen=0
with log.open('xb') as f:
 p=subprocess.Popen(cmd,stdout=f,stderr=subprocess.STDOUT,env=env,cwd=str(ROOT/'caller'))
 receipt.write_text(json.dumps({'name':name,'cmd':cmd,'pid':p.pid,'profile':'release','stage':stage,'status':'running','env':{k:env[k] for k in ['CARGO_TARGET_DIR','CARGO_NET_OFFLINE']}},indent=2)+'\n')
 while p.poll() is None:
  lines=log.read_text(errors='replace').splitlines()
  for line in lines[seen:]:
   try:v=json.loads(line)
   except ValueError:continue
   if 'stage_start' in v:stage=v['stage_start'];stage_start=time.monotonic();print('STAGE',stage,flush=True)
  seen=len(lines)
  if time.monotonic()-stage_start>600:
   print('SAFETY BOUNDARY600s',stage,p.pid,flush=True);p.terminate()
   try:p.wait(timeout=10)
   except subprocess.TimeoutExpired:p.kill();p.wait()
   break
  time.sleep(1)
 code=p.wait()
b=log.read_bytes();out={'name':name,'cmd':cmd,'pid':p.pid,'profile':'release','stage':stage,'exit':code,'elapsed_s':time.monotonic()-start,'log_sha256':hashlib.sha256(b).hexdigest(),'log_bytes':len(b),'env':{k:env[k] for k in ['CARGO_TARGET_DIR','CARGO_NET_OFFLINE']}};receipt.write_text(json.dumps(out,indent=2)+'\n');print(json.dumps(out));sys.exit(code if code>=0 else 128-code)
