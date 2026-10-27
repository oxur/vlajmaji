from pathlib import Path
import os,subprocess,hashlib,json,time,signal,shutil,sys
B=Path('/tmp/vlajmaji-p01-a04-s05-cdc');C=B/'public-caller01';T=Path('/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc01/target');HEAD=sys.argv.pop(1)
assert len(HEAD)==40 and all(c in '0123456789abcdef' for c in HEAD)
SOURCE=Path('/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc04/vlajmaji')
assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=SOURCE,text=True).strip()==HEAD
assert not subprocess.check_output(['git','status','--porcelain=v2'],cwd=SOURCE,text=True).strip()
def snapshot(label):
 out=B/(label+'-source');out.mkdir()
 for n in ('Cargo.toml','Cargo.lock'):shutil.copyfile(C/n,out/n)
 for n in ('src','fixtures'):shutil.copytree(C/n,out/n)
 rows=[{'path':str(p.relative_to(out)),'bytes':p.stat().st_size,'sha256':hashlib.sha256(p.read_bytes()).hexdigest()} for p in sorted(out.rglob('*')) if p.is_file()]
 with (B/(label+'-source-manifest.json')).open('x') as f:json.dump({'payloads':rows,'source_head':HEAD},f,indent=2);f.write('\n')
def command(label,argv,env,scope,binary=None):
 log=B/(label+'.log');start=time.monotonic();timeout=False
 with log.open('xb') as f:
  p=subprocess.Popen(argv,env=env,stdout=f,stderr=subprocess.STDOUT,start_new_session=True)
  try:code=p.wait(timeout=600)
  except subprocess.TimeoutExpired:timeout=True;os.killpg(p.pid,signal.SIGKILL);code=p.wait()
 data=log.read_bytes();r={'source_head':HEAD,'scope':scope,'argv':argv,'exit':code,'timeout':timeout,'elapsed_s':time.monotonic()-start,'log_bytes':len(data),'log_sha256':hashlib.sha256(data).hexdigest()}
 if binary:r.update(binary_bytes=binary.stat().st_size,binary_sha256=hashlib.sha256(binary.read_bytes()).hexdigest(),isolated_env={k:env[k] for k in ('HOME','XDG_DATA_HOME','XDG_CONFIG_HOME')})
 with (B/(label+'.json')).open('x') as f:json.dump(r,f,indent=2);f.write('\n')
 print(json.dumps(r),flush=True);print(data.decode()[-11000:],flush=True);return code
env=os.environ.copy();env.update(RUSTUP_TOOLCHAIN='1.98.0',CARGO='/tmp/vlajmaji-p01-a04-cargo',CARGO_TARGET_DIR=str(T),CARGO_NET_OFFLINE='true')
prefix=['/tmp/vlajmaji-p01-a04-cargo','+1.98.0'];suffix=['--manifest-path',str(C/'Cargo.toml'),'--offline','--locked']
mode=sys.argv[1]
if mode=='api':
 label=sys.argv[2];snapshot(label)
 sys.exit(command(label,prefix+['check','--bin','full-resolve-api']+suffix,env,'exact frozen public fullresolve_profiled compile contract; current corrected public API signature qualification, no product execution'))
if mode=='edges':
 number=sys.argv[2] if len(sys.argv)>2 else '01'
 build='edges-build'+number;run='edges-runtime'+number
 snapshot(build)
 code=command(build,prefix+['build','--bin','contract-edges']+suffix,env,'current corrected original-contract external caller build')
 if code:sys.exit(code)
 bi=B/'bin'/('contract-edges'+number);shutil.copyfile(T/'debug/contract-edges',bi);bi.chmod(0o700)
 e=B/(run+'-environment');e.mkdir(mode=0o700)
 for n in ('home','xdg-data','xdg-config'):(e/n).mkdir(mode=0o700)
 for k,n in [('HOME','home'),('XDG_DATA_HOME','xdg-data'),('XDG_CONFIG_HOME','xdg-config')]:env[k]=str(e/n)
 sys.exit(command(run,[str(bi),str(B/run),str(Path(sys.argv[3]))],env,'finite original-contract counterexamples with valid/restored counterparts',bi))
assert mode=='runtime';build_label=sys.argv[2];run_label=sys.argv[3]
snapshot(build_label)
code=command(build_label,prefix+['build','--bin','vlajmaji-a04-s05-cdc-public-01']+suffix,env,'current corrected original-contract external caller build, including exact full resolve API')
if code:sys.exit(code)
bi=B/'bin'/run_label;shutil.copyfile(T/'debug/vlajmaji-a04-s05-cdc-public-01',bi);bi.chmod(0o700)
e=B/(run_label+'-environment');e.mkdir(mode=0o700)
for n in ('home','xdg-data','xdg-config'):(e/n).mkdir(mode=0o700)
for k,n in [('HOME','home'),('XDG_DATA_HOME','xdg-data'),('XDG_CONFIG_HOME','xdg-config')]:env[k]=str(e/n)
code=command(run_label,[str(bi),str(B/run_label)],env,'original finite Tiny public controls plus current corrected API/profile predicates; exact previous failures retained',bi)
sys.exit(code)
