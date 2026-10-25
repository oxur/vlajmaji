import hashlib,json,os,subprocess,sys,time
from pathlib import Path
source=Path(sys.argv[1]);cli=Path(sys.argv[2]);out=Path(sys.argv[3]);out.mkdir(exist_ok=False)
fixture=Path('/tmp/vlajmaji-p01-a02-cdc/api-fixtures03/duplicate-native-and-complete');raw=fixture/'raw.json';meta=fixture/'meta.toml';sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
pins=out/'valid.env';pins.write_text(f'version=1\nrevision={"0"*40}\nraw_sha256={sha(raw)}\nmetadata_sha256={sha(meta)}\n')
env={**os.environ,'VLAJMAJI_BIN':str(cli),'CURL':'/tmp/vlajmaji-p01-a02-cdc/s02-boundary01-cases/curl','CDC_CALLS':str(out/'calls'),'CDC_REV':'0'*40,'CDC_RAW':str(raw),'CDC_META':str(meta),'CARGO_TARGET_DIR':'/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc01/target','CARGO_NET_OFFLINE':'true'}
records=[]
def run(name,operation,cache):
 argv=['/bin/bash',str(source/'scripts/jbotci-import.sh'),operation,str(pins),str(cache)];start=time.monotonic();p=subprocess.run(argv,cwd=source,env=env,capture_output=True)
 (out/(name+'.stdout')).write_bytes(p.stdout);(out/(name+'.stderr')).write_bytes(p.stderr);records.append({'case':name,'argv':argv,'returncode':p.returncode,'elapsed_seconds':time.monotonic()-start,'expected_success':True,'contract_pass':p.returncode==0});return p
alias=Path('/tmp/vlajmaji-p01-a02-cdc-parent-alias01');actual=Path('/private/tmp/vlajmaji-p01-a02-cdc-parent-alias01');assert not alias.exists() and not actual.exists()
run('standard-macos-tmp-parent','acquire',alias);p=run('real-private-tmp-counterpart','acquire',actual);assert p.returncode==0,p.stderr
link=out/'cache-root-link';link.symlink_to(actual,target_is_directory=True);run('explicit-cache-root-alias','check',link)
result={'source_head':subprocess.check_output(['git','rev-parse','HEAD'],cwd=source,text=True).strip(),'tmp_link':os.readlink('/tmp'),'controls':3,'cases':records,'disagreements':[r['case'] for r in records if not r['contract_pass']]}
(out/'result.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result,indent=2))
