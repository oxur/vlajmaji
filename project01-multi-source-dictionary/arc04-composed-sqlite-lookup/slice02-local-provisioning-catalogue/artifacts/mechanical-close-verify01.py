from pathlib import Path
import json,subprocess,hashlib
pl=Path('/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/planning'); p=pl/'project01-multi-source-dictionary';a=p/'arc04-composed-sqlite-lookup';s=a/'slice02-local-provisioning-catalogue'
def gitblob(rev,path):return subprocess.check_output(['git','show',rev+':'+str(path.relative_to(pl))],cwd=pl,text=True)
def rows(text,prefix):return [[x.strip() for x in l.strip('|').split('|')] for l in text.splitlines() if l.startswith('| '+prefix)]
r0=rows(gitblob('372db78227ff151c926c7dcd92db87053a69d7f5',s/'ledger.md'),'A04-S02-');r1=rows((s/'ledger.md').read_text(),'A04-S02-');assert len(r0)==len(r1)==6 and all(x[:4]==y[:4] and y[4]=='done' for x,y in zip(r0,r1))
p0=rows(gitblob('HEAD',p/'ledger.md'),'P01-');p1=rows((p/'ledger.md').read_text(),'P01-');assert p0==p1 and len(p1)==14 and all(x[5]=='open' for x in p1)
a0=rows(gitblob('HEAD',a/'ledger.md'),'A04-');a1=rows((a/'ledger.md').read_text(),'A04-');assert len(a0)==len(a1)==10 and all(x[:4]==y[:4] for x,y in zip(a0,a1));assert [x[0] for x in a1 if x[4]=='done']==['A04-01','A04-02']
report=(s/'closing-report.md').read_text();assert 'metadata-only replacement of unpublished' in report and 'byte-identical dictionary-query response' in report and 'existing store Linux-only test bool comparison' in report
# This checker proves only mechanical state/scope/hash fidelity, no runtime or corpus result.
source=Path('/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc04/vlajmaji')
head=subprocess.check_output(['git','rev-parse','HEAD'],cwd=source,text=True).strip();assert head=='ce1bd238d4529f5f5b27877569e4b44fa39fe12f';assert subprocess.check_output(['git','--no-optional-locks','status','--porcelain'],cwd=source)==b''
paths=subprocess.check_output(['git','diff','--name-only','4b4158c8a8a489ead639b73706b01061de17cfe1',head],cwd=source,text=True).splitlines();inv=json.loads((s/'artifacts/scope-inventory01.json').read_text());assert paths==inv['source_files'] and len(paths)==10
host=s/'artifacts/cdc-review01/hosted-run02';m=json.loads((host/'manifest.json').read_text())
for e in m['files']:
 b=(host/e['path']).read_bytes();assert len(b)==e['bytes'] and hashlib.sha256(b).hexdigest()==e['sha256']
subprocess.run(['git','diff','--check'],cwd=pl,check=True)
result={'version':1,'scope':'static close verifier only; no product/build/DB/network operation','source':head,'six_child_contracts_unchanged':True,'six_child_done':True,'arc_contracts_unchanged':True,'arc_done':['A04-01','A04-02'],'arc_open':8,'fourteen_project_rows_exact_unchanged_and_open':True,'ten_source_paths_exact':True,'six_hosted_payload_hashes_match':True,'whitespace_check':'pass','planning_close_commit':'the scoped commit containing this receipt; actual identity returned afterward'}
(s/'artifacts/mechanical-close-verification01.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result))
