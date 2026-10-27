#!/usr/bin/env python3
"""Mechanical S03 close check; reads source, writes only own receipt/inventory."""
from pathlib import Path
import hashlib,json,subprocess
PL=Path('/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/planning')
REL='project01-multi-source-dictionary/arc04-composed-sqlite-lookup/slice03-configured-combined-lookup'
S=PL/REL; A=S/'artifacts'; PROJECT=PL/'project01-multi-source-dictionary'; ARC=S.parent
SOURCE=Path('/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc04/vlajmaji')
HEAD='dcdf216ea5805b20f374ce74c302e42bef12c158'
PROPOSAL='cb246fd5884c9e0cb2544f4ac1229363b9815f22'
def git(repo,*args):return subprocess.check_output(['git','--no-optional-locks',*args],cwd=repo)
def rows(text,prefix):return [list(map(str.strip,line.split('|')[1:-1])) for line in text.splitlines() if line.startswith('| '+prefix)]
def old(path):return git(PL,'show',PROPOSAL+':'+str(path.relative_to(PL))).decode()
inv=json.loads((A/'scope-inventory01.json').read_bytes())
child=rows((S/'ledger.md').read_text(),'A04-S03-'); prior=rows(old(S/'ledger.md'),'A04-S03-')
assert len(child)==6 and [r[:4] for r in child]==[r[:4] for r in prior]
assert all(r[4]=='done' for r in child)
arc=rows((ARC/'ledger.md').read_text(),'A04-'); priorarc=rows(old(ARC/'ledger.md'),'A04-')
assert len(arc)==10 and [r[:4] for r in arc]==[r[:4] for r in priorarc]
assert {r[0] for r in arc if r[4]=='done'}=={'A04-01','A04-02','A04-04'}
assert all(r[4]=='open' for r in arc if r[0] not in {'A04-01','A04-02','A04-04'})
project=rows((PROJECT/'ledger.md').read_text(),'P01-'); priorproject=rows(old(PROJECT/'ledger.md'),'P01-')
assert len(project)==14 and [r[:6] for r in project]==[r[:6] for r in priorproject]
assert all(r[5]=='open' for r in project)
assert git(SOURCE,'rev-parse','HEAD').decode().strip()==HEAD
assert not git(SOURCE,'status','--porcelain=v1')
actual=git(SOURCE,'diff','--name-only',inv['source_baseline'],HEAD).decode().splitlines()
assert set(actual)==set(inv['source_paths']) and len(actual)==16
for name,row in inv['source_paths'].items():
 data=git(SOURCE,'show',HEAD+':'+name)
 assert len(data)==row['bytes'] and hashlib.sha256(data).hexdigest()==row['sha256'],name
payloads=0
for record in inv['manifest_records']:
 m=S/record['path']; raw=m.read_bytes();assert hashlib.sha256(raw).hexdigest()==record['sha256']
 j=json.loads(raw); table=j.get('files',j.get('payloads'))
 iterable=table.items() if isinstance(table,dict) else [(x['path'],x) for x in table]
 for name,row in iterable:
  data=(m.parent/name).read_bytes()
  assert len(data)==row['bytes'] and hashlib.sha256(data).hexdigest()==row['sha256'],(m,name)
  payloads+=1
assert payloads==219
receipt=A/'mechanical-close-verification01.json'
assert not receipt.exists()
paths=sorted([str(p.relative_to(S)) for p in A.rglob('*') if p.is_file()]+[str(receipt.relative_to(S))])
assert len(paths)==len(set(paths))
inv['artifact_paths']=paths;inv['artifact_path_count']=len(paths)
inv['scope']='S03 technical/canonical ACCEPTED; mechanical six-child/three-arc/14-open-project checks PASS'
inv['postapproval_artifacts_not_yet_claimed']=[]
inv['slice_contracts']=child
inv['planning_close_identity']='Local mechanical close commit follows; actual identity returned after commit, never self-embedded.'
(A/'scope-inventory01.json').write_text(json.dumps(inv,indent=2)+'\n')
result={'ok':True,'proposal':PROPOSAL,'source':HEAD,'source_clean':True,'source_files':16,'child_done':6,'arc_done':3,'arc_open':7,'project_open':14,'unchanged_contracts':True,'manifest_count':8,'payload_hashes_verified':payloads,'artifact_paths':len(paths),'no_product_runtime_or_Full':True,'pending_commit_identity':'returned after scoped commit','authorized_PR_merge':'root-owned separate actual integration receipt; no false unchanging-main claim'}
with receipt.open('x') as f:json.dump(result,f,indent=2);f.write('\n')
assert paths==sorted(str(p.relative_to(S)) for p in A.rglob('*') if p.is_file())
print(json.dumps(result))
