from pathlib import Path
import subprocess,time,json,os,hashlib,shutil
r=Path('/tmp/vlajmaji-p01-a03-cdc/s02');s=Path('/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc03/vlajmaji');g='/tmp/vlajmaji-p01-a03-cargo';rows=[];env=dict(os.environ,CARGO_TARGET_DIR='/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc01/target',CARGO_NET_OFFLINE='true')
assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=s,text=True).strip()=='b36adadb9aba21363957cd306001ab95a7d540bd';assert not subprocess.check_output(['git','status','--short'],cwd=s)
def run(name,args,cwd=r):
 t=time.monotonic();log=r/(name+'.log')
 with log.open('xb') as f:q=subprocess.run(args,cwd=cwd,env=env,stdout=f,stderr=subprocess.STDOUT)
 row=dict(name=name,argv=args,exit=q.returncode,elapsed=time.monotonic()-t,log_sha256=hashlib.sha256(log.read_bytes()).hexdigest());rows.append(row);(r/'review01-receipt.json').write_text(json.dumps(rows,indent=2));print(json.dumps(row),flush=True);return q.returncode
assert run('full-native01',['python3','-B','/tmp/vlajmaji-p01-a03-cdc/readiness/verify_llg.py',str(s/'crates/data-llg/data')])==0
assert run('public-lock01',[g,'+1.98.0','generate-lockfile','--manifest-path',str(r/'public-caller/Cargo.toml'),'--offline'])==0
assert run('public-build01',[g,'+1.98.0','build','--manifest-path',str(r/'public-caller/Cargo.toml'),'--offline','--locked'])==0
binary=Path(env['CARGO_TARGET_DIR'])/'debug/a03_s02_cdc_public_20261025d';copy=r/'public-candidate-b36';shutil.copy2(binary,copy);data=s/'crates/data-llg/data';hidden=r/'hidden-full-data01';before={n:hashlib.sha256((data/n).read_bytes()).hexdigest() for n in ['release.json','report.json','seal.json']};empty=r/'empty-runtime01';empty.mkdir();data.rename(hidden)
try:code=run('public-runtime01',[str(copy)],cwd=empty)
finally:hidden.rename(data)
after={n:hashlib.sha256((data/n).read_bytes()).hexdigest() for n in before};(r/'public-isolation01.json').write_text(json.dumps({'source':'b36adadb9aba21363957cd306001ab95a7d540bd','binary_sha256':hashlib.sha256(copy.read_bytes()).hexdigest(),'before':before,'after':after,'restored_equal':before==after,'runtime_exit':code},indent=2));assert before==after;assert code==0
print('PASS full/native/public integration and restored bytes',flush=True)
