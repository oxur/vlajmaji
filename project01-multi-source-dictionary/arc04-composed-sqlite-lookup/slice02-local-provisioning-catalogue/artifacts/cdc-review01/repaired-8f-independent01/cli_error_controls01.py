#!/usr/bin/env python3
"""Prospective repaired CLI boundary controls; no execution on rejected b38."""
import argparse,hashlib,json,os,pathlib,subprocess,time

def sha(b):return hashlib.sha256(b).hexdigest()
def main():
 p=argparse.ArgumentParser();p.add_argument('--binary',required=True);p.add_argument('--scratch',required=True);a=p.parse_args()
 root=pathlib.Path(a.scratch);root.mkdir(mode=0o700)
 env=dict(os.environ,HOME=str(root/'isolated-home'),XDG_DATA_HOME=str(root/'isolated-data'))
 receipts=[];fields={'version','ok','operation','category','detail','path','pack_id','attempt_id','prior_local_revision','proposed_generation','observed_generation','activation','inputs'}
 def run(label,args,operation,category,path=None,activation='not_activated',json_error=True):
  argv=[os.fsencode(a.binary),*map(os.fsencode,args)];started=time.monotonic()
  r=subprocess.run(argv,stdout=subprocess.PIPE,stderr=subprocess.PIPE,env=env,cwd=root,timeout=30)
  (root/(label+'.stdout')).write_bytes(r.stdout);(root/(label+'.stderr')).write_bytes(r.stderr)
  rec={'label':label,'argv_native':[{'hex':x.hex(),'utf8':os.fsdecode(x) if all(ord(c)<0xD800 or ord(c)>0xDFFF for c in os.fsdecode(x)) else None} for x in argv],'exit':r.returncode,'elapsed_seconds':time.monotonic()-started,'stdout_sha256':sha(r.stdout),'stderr_sha256':sha(r.stderr)}
  receipts.append(rec);(root/'attempts.json').write_text(json.dumps(receipts,indent=2)+'\n')
  assert r.returncode!=0 and not r.stdout,(label,r.returncode,r.stdout,r.stderr)
  if not json_error:
   assert b'error:' in r.stderr and b'cannot be used with' in r.stderr,r.stderr
   return
  outer=json.loads(r.stderr);assert set(outer)=={'error'},outer
  v=outer['error'];assert set(v)==fields,v
  assert v['version']==1 and v['ok'] is False and v['category']==category and v['operation']==operation and v['activation']==activation,v
  assert v['attempt_id'] is None and v['prior_local_revision'] is None and v['proposed_generation'] is None and v['observed_generation'] is None and v['inputs']==[],v
  if path is not None:
   b=os.fsencode(path);assert v['path']['encoding']=='unix-bytes' and v['path']['hex']==b.hex(),v
   try:u=b.decode('utf8')
   except UnicodeDecodeError:u=None
   assert v['path']['utf8']==u,v
  return v
 absent=root/'no-catalogue';missing=root/'absent λ ; literal.json'
 run('missing-unicode-manifest',['collections','install',absent,'--manifest',missing,'--all'],'install','input_io',missing)
 native=os.fsencode(root)+b'/missing-\xff.json'
 try:os.open(native,os.O_RDONLY)
 except OSError as e:direct_errno=e.errno
 else:raise AssertionError('unexpected existing native missing input')
 v=run('missing-native-manifest',['collections','register',absent,'--manifest',native,'--all'],'register','input_io',native)
 assert direct_errno==2 and 'os error 2' in v['detail'],v
 run('manifest-no-selector',['collections','install',absent,'--manifest',missing],'install','invalid_request')
 run('direct-no-files',['collections','register',absent,'--pack','llg'],'register','invalid_request')
 run('unknown-pack',['collections','inspect',absent,'unknown-pack'],'inspect','invalid_request',activation='not_applicable')
 run('missing-reader',['collections','list',absent],'list','missing_catalogue',absent,activation='not_applicable')
 run('clap-conflict',['collections','install',absent,'--manifest',missing,'--release',root/'unused'],'install',None,json_error=False)
 assert not absent.exists() and not (root/'unused').exists()
 summary={'ok':True,'scope':'independent repaired runtime-v1/native receipt plus plain Clap counterpart','processes':len(receipts),'direct_native_missing_errno':direct_errno,'destinations_created':[],'receipts':receipts}
 (root/'summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps(summary))
if __name__=='__main__':main()
