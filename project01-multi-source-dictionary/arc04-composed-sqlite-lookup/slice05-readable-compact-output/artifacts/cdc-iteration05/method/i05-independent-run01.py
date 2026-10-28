from pathlib import Path
import os,sys,json,hashlib,subprocess,time,signal,shutil,re
D=Path('/tmp/vlajmaji-p01-a04-s05-cdc'); O=D/'i05-independent01'; S=Path('/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc04/vlajmaji'); T=Path('/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc01/target')
HEAD='011af09a44abc153b8b8253f70d84147e21a6bb0'
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def git(*a):return subprocess.check_output(['git','--no-optional-locks',*a],cwd=S,text=True).strip()
assert git('rev-parse','HEAD')==HEAD and not git('status','--porcelain=v1')
assert sha(S/'Cargo.lock')=='65cd7f39a88bdd2a3dab1fcd14e9055ff7b77cb22ec3af8c309ce49dbea78700'
assert sha('/tmp/vlajmaji-p01-a04-cargo')=='20fe7ba73311aa503870923af09f33f3b53262742ca4eeea2972092924cf5de8'
mode=sys.argv[1]
if mode=='verify':
 O.mkdir(mode=0o700,exist_ok=True)
 files=git('diff-tree','--no-commit-id','--name-only','-r','HEAD').splitlines()
 assert set(files)==set(['crates/store/src/collections.rs','crates/store/src/lib.rs','crates/search/src/collection_query.rs','crates/search/tests/collection_query_contract.rs','crates/search/src/dictionary/projection.rs','crates/store/src/collections/reference_projection.rs'])
 assert git('rev-parse','HEAD^')=='1b9038ddc6c45f3ea709a0bdfec64041221e3571'
 assert git('rev-parse','HEAD^{tree}')=='a8c0fdcb3e39fb260602bb48b837b25f4d2380ac'
 subprocess.run(['git','diff','--check','HEAD^','HEAD'],cwd=S,check=True)
 receipts=[]
 for name,expected,pop in [('check','2c0e8cc35f4d203c19404a42a98230511b02ddda6db97c5fb0a716a02f7aa714',(338,0,1)),('no-default','375271d74c576d9a8b39333b37dd37d5d9bd378b4a09a30b145e61a6177b4e38',(83,0,0)),('all-features','eb6d4612e03c6d6a3219e65119d9e6c3a54f451d0ebbe7d953804c5723816cfe',(340,0,1))]:
  p=Path('/private/tmp')/('s05-i05-'+name+'-final01.log');assert sha(p)==expected
  raw=p.read_text();rows=re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;',raw);counts=tuple(sum(int(r[i]) for r in rows) for i in range(3));assert counts==pop
  receipts.append({'path':str(p),'bytes':p.stat().st_size,'sha256':expected,'rust_groups':len(rows),'rust_totals':counts})
 for n,e in [('store','f4dd7d566dd5689d3e6ab958eb424f75d08e75aaf5695086bad9bcb67bbae1a6'),('search','a41689f57a61ca2a0df77145ab4610fc381fe004fcfc7dbcc35d227222a238d1')]:
  p=Path('/private/tmp')/('s05-i05-'+n+'-list-final01.log');assert sha(p)==e;receipts.append({'path':str(p),'bytes':p.stat().st_size,'sha256':e,'scope':'list-only'})
 for ref in ['refs/heads/codex/project01-composed-sqlite-lookup','refs/remotes/origin/codex/project01-composed-sqlite-lookup']:assert git('rev-parse',ref)=='432d6321e3b2431e7fe2cc5f60e9317780e984b8'
 for n in ['home','xdg-data','xdg-config','tmp']:(O/n).mkdir(mode=0o700)
 result={'source':str(S),'head':HEAD,'tree':git('rev-parse','HEAD^{tree}'),'parent':git('rev-parse','HEAD^'),'branch':git('branch','--show-current'),'clean':True,'files':[{'path':f,'bytes':(S/f).stat().st_size,'sha256':sha(S/f)} for f in files],'author_raw_reconciliation':receipts,'review':'CDC read whole six-path delta and critical caller context; no blocking source-contract issue; runtime acceptance pending'}
 with (O/'frozen-review01.json').open('x') as f:json.dump(result,f,indent=2);f.write('\n')
 print(json.dumps(result));sys.exit()
env=os.environ.copy();env.update(HOME=str(O/'home'),XDG_DATA_HOME=str(O/'xdg-data'),XDG_CONFIG_HOME=str(O/'xdg-config'),TMPDIR=str(O/'tmp'),CARGO_HOME='/Users/oubiwann/.cargo',RUSTUP_HOME='/Users/oubiwann/.rustup',PATH='/Users/oubiwann/.cargo/bin:/Library/Frameworks/Python.framework/Versions/3.11/bin:/opt/homebrew/bin:/usr/bin:/bin:/usr/sbin:/sbin',CARGO_NET_OFFLINE='true',CARGO_TARGET_DIR=str(T),RUSTUP_TOOLCHAIN='1.98.0')
label=sys.argv[2]; argv=sys.argv[3:];assert argv
log=O/(label+'.log'); started=time.monotonic();timeout=False
with log.open('xb') as f:
 p=subprocess.Popen(argv,cwd=S,env=env,stdout=f,stderr=subprocess.STDOUT,start_new_session=True)
 try:code=p.wait(timeout=600)
 except subprocess.TimeoutExpired:timeout=True;os.killpg(p.pid,signal.SIGKILL);code=p.wait()
r={'source_head':HEAD,'source_tree':git('rev-parse','HEAD^{tree}'),'argv':argv,'env':{k:env[k] for k in ['HOME','XDG_DATA_HOME','XDG_CONFIG_HOME','TMPDIR','CARGO_HOME','RUSTUP_HOME','PATH','CARGO_TARGET_DIR','RUSTUP_TOOLCHAIN']},'exit':code,'timeout':timeout,'elapsed_s':time.monotonic()-started,'log':str(log),'bytes':log.stat().st_size,'sha256':sha(log)}
if mode=='runtime':r['binary']={'path':argv[0],'bytes':Path(argv[0]).stat().st_size,'sha256':sha(argv[0])}
with (O/(label+'.json')).open('x') as f:json.dump(r,f,indent=2);f.write('\n')
print(json.dumps(r),flush=True);print(log.read_text()[-7000:],flush=True);sys.exit(code)
