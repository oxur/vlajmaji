import hashlib,json,os,signal,subprocess,sys,time,stat,re
from pathlib import Path
B=Path('/tmp/vlajmaji-p01-a04-arc-review02');os.umask(0o077)
name=sys.argv[1];assert name in ('build','public','cli','post')
PYTHON='/Library/Frameworks/Python.framework/Versions/3.11/bin/python3'
TARGET='/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc01/target'
CLI='/tmp/vlajmaji-p01-a04-s05-cdc/numeric-frozen-binaries02/vlajmaji'
env={'HOME':str(B/'home'),'XDG_CONFIG_HOME':str(B/'xdg-config'),'XDG_DATA_HOME':str(B/'xdg-data'),'TMPDIR':str(B/'tmp'),'PATH':'/Users/oubiwann/.cargo/bin:/Library/Frameworks/Python.framework/Versions/3.11/bin:/opt/homebrew/bin:/usr/bin:/bin:/usr/sbin:/sbin','CARGO_HOME':'/Users/oubiwann/.cargo','RUSTUP_HOME':'/Users/oubiwann/.rustup','CARGO_TARGET_DIR':TARGET,'CARGO_NET_OFFLINE':'true','LANG':'C.UTF-8'}
commands={
'build':['/tmp/vlajmaji-p01-a04-cargo','+1.98.0','build','--release','--locked','--offline','--manifest-path',str(B/'consumer/Cargo.toml'),'--bin','vlajmaji-arc04-independent-review-01'],
'public':[TARGET+'/release/vlajmaji-arc04-independent-review-01',str(B/'public')],
'cli':[PYTHON,'-B',str(B/'methods/readable-cli-checks01.py'),CLI,str(B/'public/readable-context.json'),str(B/'cli')],
'post':[PYTHON,'-B',str(B/'methods/arc04-review-post-journey01.py'),CLI,str(B/'public/readable-context.json'),str(B/'cli')]}
argv=['/usr/bin/time','-l',*commands[name]];started=time.monotonic();timeout=False
with (B/f'receipts/{name}.stdout').open('xb') as out,(B/f'receipts/{name}.stderr').open('xb') as err:
 p=subprocess.Popen(argv,cwd=B,env=env,stdout=out,stderr=err,start_new_session=True)
 try:code=p.wait(timeout=600)
 except subprocess.TimeoutExpired:
  timeout=True;os.killpg(p.pid,signal.SIGKILL);code=p.wait()
r={'stage':name,'argv':argv,'cwd':str(B),'environment':env,'unset':['RELOJBAN_RESEARCH_ROOT'],'exit':code,'timeout':timeout,'active_process_group_timeout_seconds':600,'elapsed_seconds':time.monotonic()-started,'pid':p.pid}
for suffix in ('stdout','stderr'):
 d=(B/f'receipts/{name}.{suffix}').read_bytes();r[suffix]={'bytes':len(d),'sha256':hashlib.sha256(d).hexdigest()}
stderr=(B/f'receipts/{name}.stderr').read_text(errors='replace');rss=re.findall(r'(\d+)\s+maximum resident set size',stderr);r['maximum_rss_bytes']=int(rss[-1]) if rss else None
if name=='build' and code==0:
 q=Path(TARGET+'/release/vlajmaji-arc04-independent-review-01');d=q.read_bytes();r['binary']={'path':str(q),'bytes':len(d),'sha256':hashlib.sha256(d).hexdigest()}
with (B/f'receipts/{name}.json').open('x') as f:json.dump(r,f,indent=2);f.write('\n')
print(json.dumps(r))
if code or timeout:sys.exit(1)
