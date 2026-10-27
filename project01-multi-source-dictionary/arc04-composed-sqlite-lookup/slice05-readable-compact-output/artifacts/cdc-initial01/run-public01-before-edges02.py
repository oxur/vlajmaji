from pathlib import Path
import os,subprocess,hashlib,json,time,signal,shutil,sys
B=Path('/tmp/vlajmaji-p01-a04-s05-cdc');C=B/'public-caller01';T=Path('/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc01/target');HEAD='5bf5fb13f3d89f969570dff6aae5736feebd55fd'
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
env=os.environ.copy();env.update(CARGO='/tmp/vlajmaji-p01-a04-cargo',CARGO_TARGET_DIR=str(T),CARGO_NET_OFFLINE='true')
prefix=['/tmp/vlajmaji-p01-a04-cargo','+1.98.0'];suffix=['--manifest-path',str(C/'Cargo.toml'),'--offline','--locked']
mode=sys.argv[1]
if mode=='api':
 snapshot('api-proof01')
 sys.exit(command('api-proof01',prefix+['check','--bin','full-resolve-api']+suffix,env,'exact frozen public fullresolve_profiled compile contract; expected diagnostic failure, no product execution'))
if mode=='edges':
 snapshot('edges-build01')
 code=command('edges-build01',prefix+['build','--bin','contract-edges']+suffix,env,'original-contract frozen source lead qualification only')
 if code:sys.exit(code)
 bi=B/'bin/contract-edges01';shutil.copyfile(T/'debug/contract-edges',bi);bi.chmod(0o700)
 e=B/'edges-runtime01-environment';e.mkdir(mode=0o700)
 for n in ('home','xdg-data','xdg-config'):(e/n).mkdir(mode=0o700)
 for k,n in [('HOME','home'),('XDG_DATA_HOME','xdg-data'),('XDG_CONFIG_HOME','xdg-config')]:env[k]=str(e/n)
 sys.exit(command('edges-runtime01',[str(bi),str(B/'edges-runtime01'),str(B/'public-runtime03/caller-context.json')],env,'finite original-contract counterexamples with valid/restored counterparts',bi))
assert mode=='runtime';build_label=sys.argv[2];run_label=sys.argv[3]
snapshot(build_label)
code=command(build_label,prefix+['build','--bin','vlajmaji-a04-s05-cdc-public-01']+suffix,env,'diagnostic caller build; absent fullresolve_profiled proof remains separately failed')
if code:sys.exit(code)
bi=B/'bin'/run_label;shutil.copyfile(T/'debug/vlajmaji-a04-s05-cdc-public-01',bi);bi.chmod(0o700)
e=B/(run_label+'-environment');e.mkdir(mode=0o700)
for n in ('home','xdg-data','xdg-config'):(e/n).mkdir(mode=0o700)
for k,n in [('HOME','home'),('XDG_DATA_HOME','xdg-data'),('XDG_CONFIG_HOME','xdg-config')]:env[k]=str(e/n)
code=command(run_label,[str(bi),str(B/run_label)],env,'original finite Tiny public diagnostic controls; API contract separately failed',bi)
sys.exit(code)
