from pathlib import Path
import json,hashlib,os,stat
B=Path('/tmp/vlajmaji-p01-a04-arc-review02')
A=Path('/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/planning/project01-multi-source-dictionary/arc04-composed-sqlite-lookup/artifacts')
E=A/'composition-review';R=A/'composition-review.md';assert not R.exists() and not E.exists()
os.umask(0o077);E.mkdir(mode=0o700)
items=[]
def copy(src,rel):
 assert not src.is_symlink() and src.is_file();data=src.read_bytes();dst=E/rel;dst.parent.mkdir(parents=True,exist_ok=True)
 with dst.open('xb') as f:f.write(data)
 items.append({'path':str(rel),'original_path':str(src),'bytes':len(data),'sha256':hashlib.sha256(data).hexdigest(),'kind':'whole-review-original'})
for directory in ('consumer','methods','receipts'):
 for p in sorted((B/directory).rglob('*')):
  if p.is_file():copy(p,p.relative_to(B))
for d in (B/'cli',B/'cli/post-journey'):
 for p in sorted(d.iterdir()):
  if p.is_file():copy(p,p.relative_to(B))
for p in sorted((B/'cli/packs').rglob('*')):
 if p.is_file():copy(p,p.relative_to(B))
for p in (B/'public/readable-context.json',B/'public/caller-context.json',B/'public/shared-assertion/summary.json',B/'public/shared-entry/summary.json',B/'public/shared-entry/pre-install-expectations.json'):
 copy(p,p.relative_to(B))
for p in sorted((B/'public/readable-inputs').rglob('*')):
 if p.is_file():copy(p,Path('inputs')/p.relative_to(B/'public/readable-inputs'))
data=(B/'technical-report.md').read_bytes()
with R.open('xb') as f:f.write(data)
manifest={'schema':'arc04-independent-review02-v1','scope':'fresh selected composition/source/saved-evidence review; product source/index unchanged; not canonical closure','source':'9026c51791b5a356567f78dd9cd3b8c0eb510fa2','technical_report':{'path':str(R),'bytes':len(data),'sha256':hashlib.sha256(data).hexdigest()},'payload_count':len(items),'payload_bytes':sum(x['bytes'] for x in items),'files':items,'exclusions':'Shared target binary and newly owned Tiny DBs remain local; hashes/paths in receipts/inventories. No retained Full data copied/read except permitted saved-output comparisons. Manifest excludes its own later bytes.'}
with (E/'manifest.json').open('x') as f:json.dump(manifest,f,indent=2);f.write('\n')
print(json.dumps({k:v for k,v in manifest.items() if k!='files'},indent=2))
