"""Analyze existing finite campaign outputs ONLY after actual runner termination."""
from pathlib import Path
import json,hashlib,subprocess,time
b=Path(__file__).parent;p=b/'full-campaign01'
def sha(path):
 h=hashlib.sha256()
 with Path(path).open('rb') as f:
  for x in iter(lambda:f.read(8*1024**2),b''):h.update(x)
 return h.hexdigest()
summary=json.loads((p/'summary.json').read_text());assert summary['ok'] and summary['publications']==4 and summary['catalogue_revision']==4 and summary['profile_revision']==5
receipts=[]
for f in sorted(p.glob('*-receipt.json'),key=lambda x:x.stat().st_mtime_ns):
 x=json.loads(f.read_text());assert x['exit_code']==x['expected_child_exit'] and x['guard_exit']==x['expected_child_exit'] and not x['timed_out'] and x['rss_parse_ok'] and x['maximum_resident_set_size_bytes']>0
 receipts.append({'stage':f.name[:-13],'exit_code':x['exit_code'],'expected_exit':x['expected_child_exit'],'elapsed_seconds':x['elapsed_seconds'],'maximum_resident_set_size_bytes':x['maximum_resident_set_size_bytes'],'receipt_sha256':sha(f)})
assert len(receipts)==39
for stage in ['G1','G2','G3','G4']:
 x=json.loads((p/(stage+'-readonly-hashes.json')).read_text());assert x['before']==x['after']
wire=[]
for stage in ['G1','G2','G3','G4']:
 f=p/(stage+'-inner.json');r={'stage':stage,'bytes':f.stat().st_size,'sha256':sha(f)}
 if stage!='G3':assert r['bytes']==7160366 and r['sha256']=='144b8b09b6e378243b36c8e01ab06353c3d573459668c8c158a86aebbea73556'
 wire.append(r)
assert (p/'G2-fresh-oldG1-explicit-store.stdout').read_bytes()==(p/'G1-inner.json').read_bytes()
actual=json.loads((p/'actual-generations-and-attempts.json').read_text());assert len(actual['stages'])==4 and actual['reconciled']['attention_required'] is False
settled=json.loads((p/'settled-inventory.json').read_text());own=settled['bytes']+(p/'settled-inventory.json').stat().st_size
# Add current preserved binary/readiness/source overhead outside operational paths.
static_paths=[b/'full-frozen-binaries01',b/'full-executable-preflight02.json',b/'full-final-source-closure02.json',b/'full-expanded-argv01.json',b/'full-resource-preflight01.json',b/'full-freeze-summary02.json',b/'full-readiness-frozen01.md',b/'full-current-input-pins01.json',b/'full-current-raw-pins01.json']
static=sum(q.stat().st_size for x in static_paths for q in (x.rglob('*') if x.is_dir() else [x]) if q.is_file());assert own+static<12*1024**3
r={'ok':True,'utc':time.strftime('%Y-%m-%dT%H:%M:%SZ',time.gmtime()),'source_commit':'e4e8bfc55c5071d9144fa72e7360525efa43fd56','scope':'Existing outputs only; no DB reopen/query or additional Full operation','stages':receipts,'expected_negative_child_count':sum(x['expected_exit']!=0 for x in receipts),'full_wire':wire,'active_final':summary['final_active'],'catalogue_revision':4,'profile_revision':5,'new_data_bytes':{'operational_settled_including_inventory':own,'preserved_binary_readiness_overhead':static,'combined_observed':own+static,'threshold':12*1024**3,'scope':'One campaign root/output/config/sidecars plus named frozen binary/readiness files. Excludes preexisting accepted inputs/target/historical roots; later small analysis/canonical records separately disclosed.'},'limits':'timeout active; RSS/output/data acceptance thresholds post-completion; no live cap claim','full_coverage':'all/subset pre-input25-family + every retained descriptor, All each4stores, native47table/15empty+declaredtuples G1/G3, complete accepted wire G1/G2/G4,5refs incl removed, pages0/1/2/3, immediate cursor transitions, oldG1 explicit query, U1 no new path/rev, policy1..5, final clean4stages','not_claimed':'Full remove-all/changed release/crash/held-reader scale; actual Linux/current-head allfeatures; no all47native rows reconstructed','all_six_rows':'OPEN'}
with (b/'full-output-analysis01.json').open('x') as f:json.dump(r,f,indent=2);f.write('\n')
print(json.dumps({'ok':True,'stages':len(receipts),'expected_negative':r['expected_negative_child_count'],'observed_new_bytes':own+static,'wire':wire}))
