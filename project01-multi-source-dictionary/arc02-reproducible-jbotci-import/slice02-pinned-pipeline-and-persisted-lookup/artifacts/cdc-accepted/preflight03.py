import datetime,hashlib,json,subprocess
from pathlib import Path
r=Path('/tmp/vlajmaji-p01-a02-cdc');s=Path('/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc02/vlajmaji')
def cmd(args,cwd=s):return subprocess.check_output(args,cwd=cwd,text=True).strip()
def describe(p):
 p=Path(p);b=p.read_bytes();return {'path':str(p),'bytes':len(b),'sha256':hashlib.sha256(b).hexdigest()}
prior=json.loads((r/'s02-independent-preflight01.json').read_text());v={'time':datetime.datetime.now(datetime.timezone.utc).isoformat(),'source_head':cmd(['git','rev-parse','HEAD']),'source_status':cmd(['git','status','--porcelain=v1']),'diff_paths':cmd(['git','diff','--name-only','12fdd67..HEAD']).splitlines(),'repair_diff_paths':cmd(['git','diff','--name-only','8da776b..HEAD']).splitlines()}
v['protected']={name:{'head':cmd(['git','rev-parse','HEAD'],path),'status':cmd(['git','status','--porcelain=v1'],path)} for name,path in [('main','/Users/oubiwann/lab/oxur/vlajmaji'),('bauske','/Users/oubiwann/lab/oxur/bauske')]}
v['hashes']={name:describe(s/name)['sha256'] for name in prior['hashes']}
v['launcher']=describe('/tmp/vlajmaji-p01-a02-cargo');v['retained_old_binary']=describe(prior['retained_old_binary']['path']);v['cli']=describe(prior['cli']['path']);v['bash']=cmd(['/bin/bash','--version']);v['filesystem']=cmd(['df','-k',str(s)])
processes=cmd(['ps','-axo','pid=,ppid=,etime=,command=']).splitlines();v['matching_processes']=[x for x in processes if any(term in x for term in ['cargo +','rustc --','jbotci-import.sh replay','jbotci-import.sh update','vlajmaji dictionary-build','a02_cdc_composition_20261024']) and 'preflight02' not in x and 'retained-baseline-binary' not in x]
assert v['source_status']=='';assert all(x['status']=='' for x in v['protected'].values());assert v['retained_old_binary']['sha256']==prior['retained_old_binary']['sha256'];assert not v['matching_processes'],v['matching_processes']
(r/'s02-independent-preflight03.json').write_text(json.dumps(v,indent=2)+'\n');print(json.dumps(v,indent=2))
