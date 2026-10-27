import os,json,subprocess,hashlib,time
from pathlib import Path
b=Path(__file__).parent;o=b/'cli-runtime01';o.mkdir();cli=Path('/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc01/target/debug/vlajmaji');old=Path('/tmp/vlajmaji-p01-a04-s03-cdc/cli-45f2-release01');fixture=b/'public-runtime03-roots';root=fixture/'fresh-v2';manifest=fixture/'manifest.json';records=[]
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def run(name,args,code=0,binary=cli):
 argv=[str(binary),*map(str,args)];started=time.monotonic();r=subprocess.run(argv,env={},capture_output=True,timeout=60);(o/(name+'.stdout')).write_bytes(r.stdout);(o/(name+'.stderr')).write_bytes(r.stderr);records.append(dict(name=name,argv=argv,exit_code=r.returncode,elapsed_s=time.monotonic()-started,stdout_sha256=hashlib.sha256(r.stdout).hexdigest(),stderr_sha256=hashlib.sha256(r.stderr).hexdigest()));(o/'receipts.json').write_text(json.dumps(records,indent=2)+'\n');assert r.returncode==code,(name,r.returncode,r.stderr);return json.loads(r.stdout if code==0 else r.stderr)
keys={'version','ok','operation','category','detail','path','pack_id','attempt_id','prior_local_revision','proposed_generation','observed_generation','activation','inputs'}
def failure(x,op,category):
 e=x['error'];assert set(e)==keys;assert e['version']==1 and e['ok'] is False;assert e['operation']==op and e['category']==category;assert e['activation']=='not_activated';return e
# Frozen recipe: fresh-v2 is revision2 after initial install plus one caller child rebuild.
x=run('list',['collections','list',root]);assert x['local_revision']==2 and x['catalogue_schema_version']==2;g=x['generation'];before=sha(root/'catalogue.sqlite3')
failure(run('rebuild-stale',['collections','rebuild',root,'--expected-revision','1'],1),'rebuild','revision_conflict');assert sha(root/'catalogue.sqlite3')==before
x=run('update-idempotent',['collections','update',root,'--manifest',manifest,'--pack','sutysisku','--allow-synthetic','--expected-revision','2']);assert x['outcome']=='idempotent' and x['local_revision']==2 and x['generation']==g;assert sha(root/'catalogue.sqlite3')==before
x=run('rebuild',['collections','rebuild',root,'--expected-revision','2']);assert x['outcome']=='rebuilt' and x['local_revision']==3;n=x['generation'];assert n['generation_id']!=g['generation_id'] and n['content_id']==g['content_id'] and n['canonical_sha256']==g['canonical_sha256']
store=Path(os.fsdecode(bytes.fromhex(n['path']['hex'])));q=run('exact',['dictionary-query',store,'--request','{"operation":"exact","spelling":"farvi"}']);assert q['totals']['occurrences']==5;assert {x['source']['id']:x['occurrences'] for x in q['source_totals']}=={'lensisku':2,'llg':1,'sutysisku':2}
# Captured old executable initializes an independent metadata-only v1 ROOT.
m=o/'metadata v1 λ';run('old-register',['collections','register',m,'--manifest',manifest,'--all','--allow-synthetic'],binary=old)
x=run('migrate',['collections','migrate',m,'--expected-revision','1']);assert x['outcome']=='migrated' and x['schema_version']==2 and x['local_revision']==2 and x['active_generation'] is None
before=sha(m/'catalogue.sqlite3');failure(run('migrate-stale',['collections','migrate',m,'--expected-revision','1'],1),'migrate','revision_conflict');assert sha(m/'catalogue.sqlite3')==before
x=run('migrate-idempotent',['collections','migrate',m,'--expected-revision','2']);assert x['outcome']=='idempotent' and x['local_revision']==2 and x['active_generation'] is None;assert sha(m/'catalogue.sqlite3')==before
summary=dict(ok=True,source='8b5df7b3ebc06d14b880af59f223355736706260',cli_sha256=sha(cli),processes=len(records),scope='Tiny independent current CLI: stale+valid migrate/update/rebuild, exact failure keysets, explicit STORE query5occurrences/3groups; no Full/Linux/output-fault claim');(o/'summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps(summary))
