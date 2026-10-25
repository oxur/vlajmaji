from pathlib import Path
import subprocess,os,time,json,hashlib,shutil,sys
r=Path('/tmp/vlajmaji-p01-a03-cdc/s04');s=Path('/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc03/vlajmaji');target=Path('/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc01/target')
def sha(p):
 h=hashlib.sha256()
 with p.open('rb') as f:
  for b in iter(lambda:f.read(1048576),b''):h.update(b)
 return h.hexdigest()
source=subprocess.check_output(['git','rev-parse','HEAD'],cwd=s,text=True).strip();assert source==sys.argv[1];assert not subprocess.check_output(['git','status','--porcelain'],cwd=s)
binary=r/'public-candidate03';assert not binary.exists();shutil.copy2(target/'debug/a03_s04_cdc_public_20261025g',binary)
data=s/'crates/data-sutysisku/data';hidden=r/'hidden-full-data03';empty=r/'empty-runtime03';empty.mkdir();files=['release.json','seal.json','report.json'];before={n:sha(data/n) for n in files};assert not hidden.exists();assert not (r/'tiny-four-pack-store03').exists();env={k:v for k,v in os.environ.items() if k not in ['HOME','XDG_DATA_HOME']};data.rename(hidden);t=time.monotonic();code=None;timed_out=False
try:
 with (r/'public-runtime03.log').open('xb') as f:
  child=subprocess.Popen([str(binary),"--full-only"],cwd=empty,env=env,stdout=f,stderr=subprocess.STDOUT)
  (r/'public-process03.json').write_text(json.dumps({'pid':child.pid,'argv':[str(binary),'--full-only'],'cwd':str(empty),'source':source,'finite_timeout_seconds':600},indent=2)+'\n')
  try:code=child.wait(timeout=600)
  except subprocess.TimeoutExpired:
   timed_out=True;child.terminate()
   try:code=child.wait(timeout=10)
   except subprocess.TimeoutExpired:child.kill();code=child.wait()
finally:hidden.rename(data)
after={n:sha(data/n) for n in files};receipt={'source':source,'profile':'debug','binary_sha256':sha(binary),'source_sha256':sha(r/'public-caller/src/main.rs'),'manifest_sha256':sha(r/'public-caller/Cargo.toml'),'lock_sha256':sha(r/'public-caller/Cargo.lock'),'before':before,'after':after,'restored_equal':before==after,'home_xdg_absent':True,'empty_cwd':str(empty),'elapsed':time.monotonic()-t,'runtime_exit':code,'timed_out':timed_out};(r/'public-isolation03.json').write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps(receipt),flush=True);assert before==after;assert code==0 and not timed_out
