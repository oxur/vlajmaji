from pathlib import Path
import subprocess,json,time,hashlib,os,datetime,shutil
D=Path('/tmp/vlajmaji-p01-a04-s05-cdc');S=Path('/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc04/vlajmaji');O=D/'completion-build01';O.mkdir(mode=0o700)
env={}
for k,n in [('HOME','home'),('XDG_DATA_HOME','xdg-data'),('XDG_CONFIG_HOME','xdg-config'),('TMPDIR','tmp')]:
 p=O/n;p.mkdir(mode=0o700);env[k]=str(p)
env.update(CARGO_HOME='/Users/oubiwann/.cargo',RUSTUP_HOME='/Users/oubiwann/.rustup',PATH='/Users/oubiwann/.cargo/bin:/Library/Frameworks/Python.framework/Versions/3.11/bin:/opt/homebrew/bin:/usr/bin:/bin:/usr/sbin:/sbin',CARGO_TARGET_DIR='/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc01/target',RUSTUP_TOOLCHAIN='1.98.0')
argv=['/tmp/vlajmaji-p01-a04-cargo','+1.98.0','build','--release','--manifest-path',str(D/'completion-caller01/Cargo.toml'),'--locked','--offline']
head=subprocess.check_output(['git','--no-optional-locks','-C',str(S),'rev-parse','HEAD']).decode().strip();assert head=='011af09a44abc153b8b8253f70d84147e21a6bb0'
assert not subprocess.check_output(['git','--no-optional-locks','-C',str(S),'status','--porcelain'])
record={'source_head':head,'argv':argv,'env':env,'started_utc':datetime.datetime.now(datetime.timezone.utc).isoformat()};start=time.monotonic();timeout=False
with (O/'build.log').open('xb') as log:
 try:p=subprocess.run(argv,cwd=S,env=env,stdout=log,stderr=subprocess.STDOUT,timeout=600);code=p.returncode
 except subprocess.TimeoutExpired:code=None;timeout=True
record.update(exit=code,timeout=timeout,elapsed_seconds=time.monotonic()-start,log=str(O/'build.log'),log_bytes=(O/'build.log').stat().st_size,log_sha256=hashlib.sha256((O/'build.log').read_bytes()).hexdigest())
if code==0:
 out=D/'completion-frozen-binaries01';out.mkdir(mode=0o700)
 original=Path(env['CARGO_TARGET_DIR'])/'release/vlajmaji-a04-s05-completion-observer-01';dest=out/'completion-observer';assert not dest.exists();shutil.copyfile(original,dest);dest.chmod(0o700);record['binary']={'path':str(dest),'bytes':dest.stat().st_size,'sha256':hashlib.sha256(dest.read_bytes()).hexdigest()}
with (O/'receipt.json').open('x') as f:json.dump(record,f,indent=2);f.write('\n')
print(json.dumps(record));assert code==0 and not timeout
