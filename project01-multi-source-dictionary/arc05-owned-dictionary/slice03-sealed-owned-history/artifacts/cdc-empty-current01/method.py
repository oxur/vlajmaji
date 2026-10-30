#!/usr/bin/env python3
"""Original S03 finite empty-current-schema2 control; exactly six direct children."""
import ast, copy, hashlib, json, os, subprocess, tempfile
from pathlib import Path
S=Path('/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc04/vlajmaji')
A=Path('/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/planning/project01-multi-source-dictionary/arc05-owned-dictionary/slice03-sealed-owned-history/artifacts')
V=Path('/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc01/target/debug/vlajmaji')
HEAD='2b7b6dda55dc00d62bfa4fcdc5d5a09742fcf528';TREE='83d81705cc1de3aa69f1d6d39e4c524bb1b6d9b3'
BIN='1dd478822e8a83256ddb2c600235ab0acf90d7a37aee864392677b4d3d3be113'
def enc(x,sort=False):return json.dumps(x,ensure_ascii=False,separators=(',',':'),sort_keys=sort).encode()
def sha(x):return hashlib.sha256(x).hexdigest()
def git(*a):return subprocess.check_output(['git','-C',str(S),*a]).decode().strip()
def pins():
 assert git('rev-parse','HEAD')==HEAD and git('rev-parse','HEAD^{tree}')==TREE
 assert not git('status','--porcelain','--untracked-files=all')
 assert sha(V.read_bytes())==BIN
pins();method=Path(__file__).read_bytes()
b=Path(tempfile.mkdtemp(prefix='vlajmaji-a05-s03-empty-current-',dir='/private/tmp'))
print('owned proof root:',b,flush=True)
env=os.environ.copy();literal_path=env['PATH']
assert env['CARGO_HOME']=='/Users/oubiwann/.cargo' and env['RUSTUP_HOME']=='/Users/oubiwann/.rustup'
for k in ('HOME','XDG_DATA_HOME','XDG_CONFIG_HOME','TMPDIR'):
 p=b/k.lower();p.mkdir();env[k]=str(p)
env.pop('RELOJBAN_RESEARCH_ROOT',None);assert env['PATH']==literal_path
def save(n,d):
 with (b/n).open('xb') as f:f.write(d)
save('actual-env.json',enc({k:env.get(k) for k in ('PATH','CARGO_HOME','RUSTUP_HOME','HOME','XDG_DATA_HOME','XDG_CONFIG_HOME','TMPDIR')}))
raw=(A/'schema1-golden/snapshot.json').read_bytes();sealraw=(A/'schema1-golden/seal.json').read_bytes()
assert len(raw)==8040 and sha(raw)=='a16d6f1c3112bb62fa3dd091d6a972e17c0f084dc433f4f3110584faef090a05'
assert sha(sealraw)=='9e87f3df1a18dffc1f4f3e4418d64f76fb56e3653ff213d895320702ce090d21'
golden2=(A/'schema2-golden/snapshot.json').read_bytes();assert sha(golden2)=='2d4925e6baf72044b71cae7a78f466412276b3d837765a6f0b1ed59fe58c3d8a'
s1=json.loads(raw);s2=copy.deepcopy(s1);s2['journal_schema']=2
revisions=[{'revision':c['revision'],'active':[],'tombstones':[]} for c in s1['commits']]
prefixes=[];known=json.loads(golden2)['curation']['prefixes'][:2]
for i,c in enumerate(s1['commits']):
 body={'version':1,'source':s1['source'],'revision':c['revision'],'base_state_sha256':c['state_sha256'],'contexts':[],'records':[],'revisions':revisions[:i+1]}
 digest=sha(enc(body));assert digest==known[i]['curation_sha256']
 prefixes.append({'revision':c['revision'],'contexts_length':0,'records_length':0,'curation_sha256':digest})
s2['curation']={'version':1,'contexts':[],'records':[],'revisions':revisions,'prefixes':prefixes}
expected=enc(s2);seal2=json.loads(sealraw);seal2['journal_schema']=2;seal2['snapshot_sha256']=sha(expected);seal2['snapshot_bytes']=len(expected);expected_seal=enc(seal2)
for n,d in [('schema1.json',raw),('schema1.seal.json',sealraw),('expected-schema2.json',expected),('expected-schema2.seal.json',expected_seal)]:save(n,d)
save('pre-input-oracle.json',enc({'source':HEAD,'tree':TREE,'binary_sha256':BIN,'method_sha256':sha(method),'snapshot_bytes':len(expected),'snapshot_sha256':sha(expected),'seal_bytes':len(expected_seal),'seal_sha256':sha(expected_seal),'curation_hashes':[x['curation_sha256'] for x in prefixes]}))
records=[]
def run(label,args,want):
 argv=[str(V),'owned',*map(str,args)]
 try:r=subprocess.run(argv,cwd=b,env=env,capture_output=True,timeout=120)
 except subprocess.TimeoutExpired as e:
  out,err=e.stdout or b'',e.stderr or b'';save(label+'.stdout',out);save(label+'.stderr',err)
  records.append({'label':label,'argv':argv,'exit':None,'timed_out':True,'stdout_sha256':sha(out),'stderr_sha256':sha(err)})
  (b/'commands.json').write_bytes(enc(records));raise
 save(label+'.stdout',r.stdout);save(label+'.stderr',r.stderr)
 records.append({'label':label,'argv':argv,'exit':r.returncode,'timed_out':False,'stdout_bytes':len(r.stdout),'stderr_bytes':len(r.stderr),'stdout_sha256':sha(r.stdout),'stderr_sha256':sha(r.stderr)})
 (b/'commands.json').write_bytes(enc(records))
 assert r.returncode==0 and not r.stderr,(label,r.returncode,r.stderr)
 assert r.stdout==enc(want,sort=True)+b'\n',(label,r.stdout,want)
def imported(schema,digest):return {'version':1,'outcome':'imported','source':'owned-example','head':'owned-r1','journal_schema':schema,'revision_count':2,'snapshot_sha256':digest}
def native(p):return {'encoding':'unix-bytes','hex':os.fsencode(p).hex(),'utf8':str(p)}
def exported(out,seal):return {'version':1,'source':'owned-example','head':'owned-r1','journal_schema':2,'revision_count':2,'output':native(out),'output_bytes':len(expected),'output_sha256':sha(expected),'seal':native(seal),'seal_bytes':len(expected_seal),'seal_sha256':sha(expected_seal)}
r1=b/'migrated-root';r2=b/'reimported-root';o1=b/'first.json';z1=b/'first.seal.json';o2=b/'second.json';z2=b/'second.seal.json'
run('schema1-import',['import',r1,'--input',b/'schema1.json','--seal',b/'schema1.seal.json'],imported(1,sha(raw)))
run('migrate-empty',['migrate-curation',r1,'--expected-head','owned-r1'],{'version':1,'source':'owned-example','head':'owned-r1','from_schema':1,'to_schema':2,'outcome':'migrated'})
run('export-empty',['export',r1,'--output',o1,'--seal',z1],exported(o1,z1));assert o1.read_bytes()==expected and z1.read_bytes()==expected_seal
run('check-empty',['check','--input',o1,'--seal',z1],{'version':1,'source':'owned-example','head':'owned-r1','journal_schema':2,'revision_count':2,'snapshot_bytes':len(expected),'snapshot_sha256':sha(expected),'seal_bytes':len(expected_seal),'seal_sha256':sha(expected_seal)})
run('import-empty',['import',r2,'--input',o1,'--seal',z1],imported(2,sha(expected)))
run('reexport-empty',['export',r2,'--output',o2,'--seal',z2],exported(o2,z2));assert o2.read_bytes()==expected and z2.read_bytes()==expected_seal
assert len(records)==6;pins();save('execution.json',enc({'ok':True,'source':HEAD,'tree':TREE,'binary_sha256':BIN,'method_sha256':sha(method),'children':records,'snapshot_bytes':len(expected),'snapshot_sha256':sha(expected),'seal_bytes':len(expected_seal),'seal_sha256':sha(expected_seal)})+b'\n')
print('PASS original empty-current schema2 control: 6 children',flush=True)
