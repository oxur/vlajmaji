import hashlib,json,os,subprocess,sys
from pathlib import Path
source=Path(sys.argv[1]);cli=Path(sys.argv[2]);out=Path(sys.argv[3]);out.mkdir(exist_ok=False);root=Path('/tmp/vlajmaji-p01-a02-cdc/api-fixtures03');sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();records=[]
for name,fixture,success in [('mismatch','count-mismatch',False),('valid','duplicate-native-and-complete',True)]:
 raw=root/fixture/'raw.json';meta=root/fixture/'meta.toml';pins=out/(name+'.env');pins.write_text(f'version=1\nrevision={"0"*40}\nraw_sha256={sha(raw)}\nmetadata_sha256={sha(meta)}\n');cache=out/(name+'-cache');release=out/(name+'.json');report=Path(str(release)+'.report.json');store=out/(name+'-store')
 env={**os.environ,'VLAJMAJI_BIN':str(cli),'CURL':'/tmp/vlajmaji-p01-a02-cdc/s02-boundary01-cases/curl','CDC_CALLS':str(out/'calls'),'CDC_REV':'0'*40,'CDC_RAW':str(raw),'CDC_META':str(meta),'CARGO_TARGET_DIR':'/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc01/target','CARGO_NET_OFFLINE':'true'}
 argv=['/bin/bash',str(source/'scripts/jbotci-import.sh'),'update',str(pins),str(cache),str(release),str(store)];p=subprocess.run(argv,cwd=source,env=env,capture_output=True);(out/(name+'.stdout')).write_bytes(p.stdout);(out/(name+'.stderr')).write_bytes(p.stderr)
 leaf=cache/('0'*40+'-'+sha(raw)+'-'+sha(meta));record={'case':name,'argv':argv,'returncode':p.returncode,'expected_success':success,'release':release.exists(),'report':report.exists(),'store':store.exists(),'complete':(leaf/'complete').exists()}
 assert p.returncode==(0 if success else 1) and record['release'] and record['report'] and record['store']==success and record['complete']==success
 if not success:record['attempt_paths_in_stderr']={str(x):str(x).encode() in p.stderr for x in [leaf,release,report,store]};record['contract_pass']=all(record['attempt_paths_in_stderr'].values()) and b'jbotci_accounting' in p.stderr
 else:json.loads(p.stdout);record['contract_pass']=True
 records.append(record)
result={'controls':2,'cases':records,'disagreements':[x['case'] for x in records if not x['contract_pass']]};(out/'result.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result,indent=2))
