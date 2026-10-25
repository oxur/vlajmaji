from pathlib import Path
import subprocess,os,time,json,hashlib,tarfile,shutil
import sys
head=sys.argv[1]
r=Path('/tmp/vlajmaji-p01-a03-cdc/s04');s=Path('/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc03/vlajmaji');p=r/('clean-ci-'+head[:8]+'-01');p.mkdir();c=p/'vlajmaji';c.mkdir();arc=p/'source.tar'
with arc.open('xb') as f:subprocess.run(['git','archive',head],cwd=s,stdout=f,check=True)
with tarfile.open(arc) as t:
 assert all(not m.name.startswith('/') and '..' not in Path(m.name).parts for m in t.getmembers())
 t.extractall(c)
(p/'bauske').symlink_to('/Users/oubiwann/lab/oxur/bauske',target_is_directory=True)
env=dict(os.environ,CARGO_TARGET_DIR='/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc01/target',CARGO_NET_OFFLINE='true');g='/tmp/vlajmaji-p01-a03-cargo';rows=[]
def run(name,args,expected=0):
 t=time.monotonic();log=r/(name+'.log')
 with log.open('xb') as f:q=subprocess.run(args,cwd=c,env=env,stdout=f,stderr=subprocess.STDOUT)
 row=dict(name=name,argv=args,exit=q.returncode,elapsed=time.monotonic()-t,log_sha256=hashlib.sha256(log.read_bytes()).hexdigest());rows.append(row);(r/'clean-ci01-receipt.json').write_text(json.dumps(rows,indent=2));print(json.dumps(row),flush=True);assert q.returncode==expected,log
run('clean-no-data-default01',[g,'+1.98.0','check','--workspace','--locked','--offline'])
run('clean-selected-missing01',[g,'+1.98.0','check','-p','vlajmaji','--features','data-sutysisku','--locked','--offline'],101)
x=(r/'clean-selected-missing01.log').read_text();assert 'include_bytes!' in x and ('release.json' in x and 'seal.json' in x)
run('clean-ci-synthetic-prep01',[g,'+1.98.0','run','--locked','--offline','-p','vlajmaji-cli','--','data-pack','fixture','jbotci','crates/data-lensisku-jbotci/data'])
run('clean-ci-llg-synthetic-prep01',[g,'+1.98.0','run','--locked','--offline','-p','vlajmaji-cli','--','data-pack','fixture','llg','crates/data-llg/data'])
run('clean-ci-la-synthetic-prep01',[g,'+1.98.0','run','--locked','--offline','-p','vlajmaji-cli','--','data-pack','fixture','la-lojban','crates/data-la-lojban/data'])
run('clean-ci-sut-synthetic-prep01',[g,'+1.98.0','run','--locked','--offline','-p','vlajmaji-cli','--','data-pack','fixture','sutysisku','crates/data-sutysisku/data'])
cli=r/'candidate-cli';shutil.copy2(Path(env['CARGO_TARGET_DIR'])/'debug/vlajmaji',cli)
(r/'candidate-cli.json').write_text(json.dumps({'source':head,'profile':'debug','sha256':hashlib.sha256(cli.read_bytes()).hexdigest(),'build_log':'clean-ci-synthetic-prep01.log'},indent=2))
run('clean-ci-all-features01',['make','test-all-features','CARGO='+g])
for name in ['vlajmaji-cli','vlajmaji-lexicon','vlajmaji','vlajmaji-data-lensisku-jbotci','vlajmaji-data-llg','vlajmaji-data-la-lojban','vlajmaji-data-sutysisku']:
 run('package-'+name+'01',[g,'+1.98.0','package','--list','--locked','--offline','--allow-dirty','-p',name])
 data=(r/('package-'+name+'01.log')).read_text().splitlines();assert not any(line.startswith('data/') for line in data)
print('PASS clean offline feature preparation and package exclusion',flush=True)
