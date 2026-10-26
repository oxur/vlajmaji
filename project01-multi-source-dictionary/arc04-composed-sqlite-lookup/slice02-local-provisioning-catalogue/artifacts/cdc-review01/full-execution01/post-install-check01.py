"""Read-only/resource gate after the single install. No creation or retry path."""
from pathlib import Path
import json,hashlib,stat,sqlite3,time
D=Path('/tmp/vlajmaji-p01-a04-s02-cdc');F=D/'full-freeze01';root=D/'full-operational01'
sha=lambda p:hashlib.file_digest(p.open('rb'),'sha256').hexdigest()
receipt=json.loads((F/'install-process-receipt01.json').read_bytes())
assert receipt['decision']=='passed' and receipt['exit_code']==0 and not receipt['timed_out'],receipt
raw=(D/'cdc-logs/full-install01.log').read_text();report,end=json.JSONDecoder().raw_decode(raw.lstrip())
assert report['version']==1 and report['ok'] is True and report['operation']=='install' and report['outcome']=='installed' and report['local_revision']==1,report
assert report['policy_state']=='unconfigured'
g=report['generation'];assert g['canonical_sha256']=='d9d73cbb49cc5a9ddb0ec16cb6dbb4bb7260ad49f4f8f54c6e6d21f0779b7b19'
assert [m['pack_id'] for m in g['members']]==['lensisku-jbotci','llg','la-lojban','sutysisku']
assert all(m['payload_kind']=='full' for m in g['members'])
assert len(report['inputs'])==4 and all(x['disposition']=='installed' for x in report['inputs'])
files=[];directories=[]
for p in [root,*sorted(root.rglob('*'))]:
 s=p.lstat();assert not stat.S_ISLNK(s.st_mode),str(p)
 if stat.S_ISDIR(s.st_mode):
  directories.append({'path':str(p.relative_to(root)),'mode':oct(stat.S_IMODE(s.st_mode))});assert stat.S_IMODE(s.st_mode)==0o700,(p,oct(s.st_mode))
 else:
  assert stat.S_ISREG(s.st_mode);files.append({'path':str(p.relative_to(root)),'bytes':s.st_size,'mode':oct(stat.S_IMODE(s.st_mode))});assert stat.S_IMODE(s.st_mode)==0o600,(p,oct(s.st_mode))
total=sum(x['bytes'] for x in files);assert total<=2300000000,total
store=Path(g['path']['utf8']);assert store.is_relative_to(root);database=store/'dictionary.sqlite3';dbsha=sha(database)
con=sqlite3.connect(database.as_uri()+'?mode=ro',uri=True);con.execute('PRAGMA query_only=ON')
with con.blobopen('canonical_document','json',1,readonly=True) as blob:
 n=len(blob);h=hashlib.sha256()
 while chunk:=blob.read(1048576):h.update(chunk)
assert n==435519540 and h.hexdigest()==g['canonical_sha256'],(n,h.hexdigest())
con.close();assert sha(database)==dbsha
expected=json.loads((F/'preflight01.json').read_bytes())
for item in expected['inputs']:
 assert sha(Path(item['release']))==item['release_sha256'];assert sha(Path(item['seal']))==item['seal_sha256']
result={'ok':True,'utc':time.strftime('%Y-%m-%dT%H:%M:%SZ',time.gmtime()),'report':report,'artifact_bytes':total,'files':files,'directories':directories,'store':str(store),'database':str(database),'database_sha256_before_readonly_campaign':dbsha,'canonical_blob_bytes':n,'canonical_blob_sha256':h.hexdigest(),'source_pins_unchanged':True,'rss_and_disk_limit_semantics':'post-completion acceptance thresholds;600sec timeout active'}
with (F/'post-install-resource-canonical01.json').open('x') as f:json.dump(result,f,indent=2);f.write('\n')
print(json.dumps({k:v for k,v in result.items() if k not in ['report','files','directories']}))
