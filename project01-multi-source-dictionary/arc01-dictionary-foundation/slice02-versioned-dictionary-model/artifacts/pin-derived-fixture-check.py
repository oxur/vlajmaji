import collections, hashlib, json, subprocess, sys
from pathlib import Path
R=Path('/Users/oubiwann/Library/Application Support/ChatGPT-Profiles/personal/codex-home/.chatgpt-projects/g-p-6a8c72d334048191887d4e00472f8430/relojban-poetics')
PIN='b9b76edaa206b2c89f0436059354c2943be7b6da'
BASE='experiments/E02-resource-discovery/raw/lexical/'
fixture=json.loads(Path(sys.argv[1]).read_bytes())
raw=subprocess.check_output(['git','show',f'{PIN}:{BASE}lensisku-farvi.json'],cwd=R)
assert hashlib.sha256(raw).hexdigest()=='1117d272d84557d08734b9e8c6f7969f1e8d7afe586db59ead19b8ba800eea79'
rows=json.loads(raw);assert len(rows)==13
five,four=rows[0],rows[1]
assert (five['definitionid'],four['definitionid'])==(69002,333)
gismu=subprocess.check_output(['git','show',f'{PIN}:{BASE}llg-gismu.txt'],cwd=R)
assert hashlib.sha256(gismu).hexdigest()=='f2fc4cff4f265ad02717d56733b4627ffa78c616db78dcaa44caab694e1dd5b7'
cpana=[(i+1,line) for i,line in enumerate(gismu.decode().splitlines()) if line.split() and line.split()[0]=='cpana']
assert len(cpana)==1 and cpana[0][0]==226 and len(cpana[0][1].encode())==309
checks={};captures={c['identity']['namespace']:c for c in fixture['captures']}
for namespace,locator,expected,label in [('captured:a:d1','/0',five,'d1_first_five'),('captured:a:d1','/1',five,'d1_duplicate_five'),('captured:a:d1','/2',four,'d1_four'),('captured:a:d2','/0',five,'d2_same_five')]:
    occurrence=next(o for o in captures[namespace]['occurrences'] if o['locator']==locator)
    # Compare every supplied key/value, including fields not projected into normalized columns.
    actual={f['name']:json.loads(f['raw_value']) for f in occurrence['fields']}
    checks[f'{label}_full_object']=actual==expected
checks['all_raw_payload_sha256']=all(hashlib.sha256(bytes(c['bytes'])).hexdigest()==c['identity']['sha256'] for c in fixture['captures'])
checks['exact_cpana_raw_line']=sum(o['raw_line']==cpana[0][1] for c in fixture['captures'] for o in c['occurrences'])==1
checks['loaded_capture_denominator']=len(fixture['captures'])==9
checks['raw_occurrence_denominator']=sum(len(c['occurrences']) for c in fixture['captures'])==20
bindings={(s['snapshot']['namespace'],s['snapshot']['sha256']):s['source'] for s in fixture['registry']['snapshots']}
counts=collections.Counter()
for c in fixture['captures']: counts[bindings[c['identity']['namespace'],c['identity']['sha256']]]+=len(c['occurrences'])
checks['source_partition']=dict(counts)=={'source-a':17,'source-b':1,'source-o':2}
checks['normalized_population']=len(fixture['metadata']['entries'])==14 and len(fixture['metadata']['assertions'])==20
checks['pinned_native_sense_absent']=all(a['native_sense_id']=='Missing' for a in fixture['metadata']['assertions'] if a['occurrence']['capture']['namespace'] in ('captured:a:d1','captured:a:d2'))
print(json.dumps({'pin':PIN,'source_rows':13,'cpana_line':226,'checks':checks,'passed':sum(checks.values()),'total':len(checks)},indent=2))
if not all(checks.values()):sys.exit(1)
