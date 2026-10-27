"""Read-only receipt/owned marker reconciliation after the finite fresh Tiny CLI journey."""
import hashlib,json,sys
from pathlib import Path
base=Path('/tmp/vlajmaji-p01-a04-s04-cdc/i02-cli-runtime01')
root=base/'cli lifecycle λ $()'
r=json.loads((base/'reconcile.stdout').read_bytes())
committed={a['attempt_id']:a for a in r['attempts'] if a['status']=='committed'}
markers=[]
for d in sorted((root/'generations').iterdir()):
    assert d.is_dir() and not d.is_symlink()
    m=json.loads((d/'attempt.json').read_bytes())
    assert set(m)=={'version','attempt_id','operation'} and m['version']==1
    assert m['attempt_id']==d.name and d.name in committed
    assert committed[d.name]['operation']==m['operation']
    markers.append(m)
assert [m['operation'] for m in markers].count('remove')==1
assert [m['operation'] for m in markers].count('reinstall')==1
expected={'staged_attempts':[],'attention_required':False}
actual={k:r[k] for k in expected}
report={'source_commit':'a9eeb440b029a4466f265cfc4eaf3e15fb773b31','preconditions':'fresh original four-pack Tiny CLI journey completed with only committed owned stages; no manual or failed-stage injection in this ROOT','markers':markers,'committed_attempts':r['attempts'],'expected':expected,'actual':actual,'ok':actual==expected,'script_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),'reconcile_stdout_sha256':hashlib.sha256((base/'reconcile.stdout').read_bytes()).hexdigest(),'scope':'same finite CLI outputs and exact marker files only; no new product invocation/DB open/mutation/Full access; one connected committed-stage classification finding'}
p=Path('/tmp/vlajmaji-p01-a04-s04-cdc/i02-stage-classifier-review01.json')
with p.open('x') as f:json.dump(report,f,indent=2);f.write('\n')
print(json.dumps(report,indent=2))
sys.exit(0 if report['ok'] else 1)
