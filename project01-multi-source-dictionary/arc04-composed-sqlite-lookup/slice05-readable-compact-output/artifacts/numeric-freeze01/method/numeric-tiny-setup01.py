"""Owned original four-literal Tiny setup for SAME numeric runner qualification only."""
from pathlib import Path
import hashlib,importlib.util,json,os,shutil,stat,subprocess,time
D=Path('/tmp/vlajmaji-p01-a04-s05-cdc');B=D/'numeric-tiny-fixture01';B.mkdir(mode=0o700)
spec=importlib.util.spec_from_file_location('checks',D/'readable-cli-checks01.py');checks=importlib.util.module_from_spec(spec);spec.loader.exec_module(checks)
source=D/'readable-runtime01/readable-context.json';raw=source.read_bytes();ctx=json.loads(raw);assert ctx['fixture_scope']=='original-four-pack-synthetic-readable'
newrows=[];manifest=[]
for row in ctx['inputs']:
 pid=row['pack_id'];d=B/'packs'/pid;d.mkdir(parents=True);new=dict(row)
 for key in ('release','seal'):
  p=Path(row[key]);p.relative_to(source.parent);b=checks.nofollow_owned_file(p,source.parent);assert hashlib.sha256(b).hexdigest()==row[key+'_sha256']
  dest=d/(key+'.json');dest.open('xb').write(b);new[key]=str(dest)
 newrows.append(new);manifest.append({'kind':'sealed','pack_id':pid,'release':str(d.relative_to(B)/'release.json'),'seal':str(d.relative_to(B)/'seal.json')})
assert [r['pack_id'] for r in manifest]==checks.PACKS
ctx['inputs']=newrows;(B/'readable-context.json').open('x').write(json.dumps(ctx,indent=2,ensure_ascii=False)+'\n');(B/'manifest.json').open('x').write(json.dumps({'version':1,'inputs':manifest},indent=2)+'\n')
(B/'input-provenance.json').open('x').write(json.dumps({'source':str(source),'source_bytes':len(raw),'source_sha256':hashlib.sha256(raw).hexdigest(),'scope':'exact copied sealed bytes and input-derived context; no candidate lexical oracle'},indent=2)+'\n')
setup=B/'setup';setup.mkdir();env={'PATH':'/usr/bin:/bin:/usr/sbin:/sbin','LANG':'en_US.UTF-8','LC_ALL':'en_US.UTF-8'}
for name in ('HOME','XDG_DATA_HOME','XDG_CONFIG_HOME'):
 p=B/name.lower();p.mkdir(mode=0o700);env[name]=str(p)
cli=D/'numeric-frozen-binaries01/vlajmaji';root=B/'root';config=B/'config.toml';assert not root.exists() and not config.exists()
def run(name,args):
 argv=[str(cli),*map(str,args)];started=time.monotonic()
 with (setup/(name+'.stdout')).open('xb') as o,(setup/(name+'.stderr')).open('xb') as e:r=subprocess.run(argv,stdout=o,stderr=e,env=env,timeout=60)
 stdout=(setup/(name+'.stdout')).read_bytes();stderr=(setup/(name+'.stderr')).read_bytes();receipt={'argv':argv,'exit':r.returncode,'elapsed_seconds':time.monotonic()-started,'stdout_bytes':len(stdout),'stdout_sha256':hashlib.sha256(stdout).hexdigest(),'stderr_bytes':len(stderr),'stderr_sha256':hashlib.sha256(stderr).hexdigest()};(setup/(name+'.receipt.json')).open('x').write(json.dumps(receipt,indent=2)+'\n');assert r.returncode==0 and stderr==b'',receipt
 return checks.prior.strict_json(stdout)
i=run('install',['collections','install',root,'--manifest',B/'manifest.json','--all','--allow-synthetic']);assert i['local_revision']==1 and i['outcome']=='installed'
store=checks.path_report(i['generation']['path']);relative=store.relative_to(root);assert relative.parts and all(p not in ('..','.') for p in relative.parts)
walk=root
for p in (root,*[root.joinpath(*relative.parts[:n]) for n in range(1,len(relative.parts)+1)]):
 m=p.lstat();assert stat.S_ISDIR(m.st_mode) and not stat.S_ISLNK(m.st_mode)
for p in (root/'catalogue.sqlite3',store/'dictionary.sqlite3'):
 m=p.lstat();assert stat.S_ISREG(m.st_mode) and not stat.S_ISLNK(m.st_mode)
c=run('configure',['collections','configure',root,'--profile','work','--all','--make-default','--config',config]);assert c['outcome']=='committed' and c['revision']==1
l=run('list',['collections','list',root]);assert l['local_revision']==1 and l['catalogue_schema_version']==2
p=run('profiles',['collections','profiles','--config',config]);assert p['revision']==1 and p['default_profile']=='work'
summary={'ok':True,'fixture_scope':ctx['fixture_scope'],'root':str(root),'config':str(config),'store':str(store),'catalogue_revision':1,'profile_revision':1,'inputs':newrows,'no_full_access':True};(B/'setup-summary.json').open('x').write(json.dumps(summary,indent=2)+'\n');print(json.dumps(summary))
