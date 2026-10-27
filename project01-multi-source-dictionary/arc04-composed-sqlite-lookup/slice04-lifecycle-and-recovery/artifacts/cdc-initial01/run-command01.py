import sys,os,json,time,hashlib,subprocess,signal
from pathlib import Path
base=Path(__file__).parent
label=sys.argv[1];argv=sys.argv[2:]
assert label and argv
log=base/(label+'.log');receipt=base/(label+'.json')
assert not log.exists() and not receipt.exists()
env=os.environ.copy();env.update(CARGO='/tmp/vlajmaji-p01-a04-cargo',RUST_TOOLCHAIN='1.98.0',CARGO_TARGET_DIR='/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc01/target',CARGO_NET_OFFLINE='true')
sources={}
for p in sorted((base/'public-caller01').rglob('*')):
 if p.is_file():sources[str(p.relative_to(base))]=hashlib.sha256(p.read_bytes()).hexdigest()
started=time.monotonic();timeout=False
with log.open('xb') as out:
 child=subprocess.Popen(argv,stdout=out,stderr=subprocess.STDOUT,env=env,start_new_session=True)
 try:code=child.wait(timeout=600)
 except subprocess.TimeoutExpired:
  timeout=True;os.killpg(child.pid,signal.SIGKILL);code=child.wait()
b=log.read_bytes();data=dict(argv=argv,cwd=os.getcwd(),source_commit='8b5df7b3ebc06d14b880af59f223355736706260',caller_sources_before=sources,exit_code=code,timed_out=timeout,elapsed_s=time.monotonic()-started,log_bytes=len(b),log_sha256=hashlib.sha256(b).hexdigest())
receipt.write_text(json.dumps(data,indent=2)+'\n')
print(json.dumps(data));print(b.decode(errors='replace')[-2800:]);sys.exit(1 if timeout else code)
