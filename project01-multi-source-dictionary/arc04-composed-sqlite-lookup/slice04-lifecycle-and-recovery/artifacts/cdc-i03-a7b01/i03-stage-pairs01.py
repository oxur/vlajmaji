"""Finite independent stage-classifier counterparts on the SAME fresh Tiny CLI journey.
Execute only after frozen i03 handback; owned test stages are retained, never activated.
"""
import hashlib,json,subprocess,sys,time
from pathlib import Path
cli,base,out=map(Path,sys.argv[1:4]);out.mkdir(mode=0o700)
root=base/'cli lifecycle λ $()'
env=json.loads((base/'child-environment.json').read_bytes())
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def new(p,v):
 with p.open('x') as f:json.dump(v,f,indent=2);f.write('\n')
records=[]
def reconcile(name):
 argv=[str(cli),'collections','reconcile',str(root)];started=time.monotonic();r=subprocess.run(argv,env=env,capture_output=True,timeout=60)
 (out/(name+'.stdout')).write_bytes(r.stdout);(out/(name+'.stderr')).write_bytes(r.stderr)
 records.append({'name':name,'argv':argv,'exit_code':r.returncode,'elapsed_s':time.monotonic()-started,'stdout_sha256':hashlib.sha256(r.stdout).hexdigest(),'stderr_sha256':hashlib.sha256(r.stderr).hexdigest()});assert r.returncode==0,(name,r.stderr)
 return json.loads(r.stdout)
db=root/'catalogue.sqlite3';db_before=sha(db)
initial=reconcile('committed-clean')
assert initial['local_revision']==4 and initial['active_generation'] is None
assert initial['retained_generation_count']==3 and len(initial['attempts'])==4
assert initial['staged_attempts']==[] and initial['attention_required'] is False
committed={a['attempt_id']:a for a in initial['attempts'] if a['status']=='committed'}
marker_before={str(p):sha(p) for p in (root/'generations').glob('*/attempt.json')}
assert len(marker_before)==3
orphan='cdc-uncommitted-supported-remove';assert orphan not in committed
p=root/'generations'/orphan;p.mkdir(mode=0o700)
new(p/'attempt.json',{'version':1,'attempt_id':orphan,'operation':'remove'})
orphan_bytes=(p/'attempt.json').read_bytes()
second=reconcile('supported-genuine-orphan')
assert second['staged_attempts']==[orphan] and second['attention_required'] is True
unknown='cdc-unknown-operation-committed-id';p=root/'generations'/unknown;p.mkdir(mode=0o700)
committed_id=next(a for a in committed if committed[a]['operation']=='install')
new(p/'attempt.json',{'version':1,'attempt_id':committed_id,'operation':'unknown-operation'})
unknown_bytes=(p/'attempt.json').read_bytes()
third=reconcile('unknown-operation-actually-committed-id')
assert set(third['staged_attempts'])=={orphan,'unreadable:'+unknown.encode().hex()} and len(third['staged_attempts'])==2
assert third['attention_required'] is True
for r in [second,third]:
 assert r['local_revision']==initial['local_revision'] and r['active_generation']==initial['active_generation']
 assert r['retained_generation_count']==initial['retained_generation_count'] and r['attempts']==initial['attempts']
assert sha(db)==db_before and all(sha(Path(p))==v for p,v in marker_before.items())
assert (root/'generations'/orphan/'attempt.json').read_bytes()==orphan_bytes
assert (root/'generations'/unknown/'attempt.json').read_bytes()==unknown_bytes
summary={'ok':True,'scope':'three actual public CLI reconciliations on same original four-pack Tiny journey: committed remove/reinstall clean, supported genuine orphan attention, unknown operation with actually committed ID unreadable; no Full/cleanup/activation/schema mutation','processes':records,'cli_sha256':sha(cli),'script_sha256':sha(Path(__file__)),'root':str(root),'database_before_after':db_before,'owned_original_markers_before_after':marker_before,'retained_controlled_stages':[orphan,unknown],'observed_retained_generation_count':3,'actual_physical_stage_directories':5}
new(out/'summary.json',summary);print(json.dumps(summary))
