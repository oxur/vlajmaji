import hashlib,json,subprocess
from pathlib import Path
p=Path('/tmp/vlajmaji-p01-a02-cdc/duplicate-unknown-attempt01');p.mkdir()
raw=b'[{"word":"farvi","word_type":"gismu","definition":"a","definition":"b","opaque":{"n":123456789012345678901234567890}},{"word":"farvi","word_type":"gismu","definition":"a","opaque":{"n":123456789012345678901234567890}}]'
sha=lambda b:hashlib.sha256(b).hexdigest()
meta=('language_tag = "en"\nlanguage_realname = "English"\nsource_language_tag = "jbo"\nformat = "json"\npositive_scores_only = false\nfilename = "dictionary-en.json"\nmetadata_url = "https://example.invalid/languages"\ndownload_url = "https://example.invalid/export"\nlensisku_created_at = "2026-10-04T03:07:08Z"\nsha256 = "'+sha(raw)+'"\ndefinition_count = 2\nentry_count = 1\n').encode()
(p/'raw.json').write_bytes(raw);(p/'metadata.toml').write_bytes(meta)
bin='/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc01/target/debug/vlajmaji'
cmd=[bin,'jbotci-import',str(p/'raw.json'),str(p/'metadata.toml'),str(p/'release.json'),'--revision','0'*40,'--raw-sha256',sha(raw),'--metadata-sha256',sha(meta)]
r=subprocess.run(cmd,capture_output=True);(p/'stdout').write_bytes(r.stdout);(p/'stderr').write_bytes(r.stderr)
assert r.returncode==0,(r.returncode,r.stderr)
d=json.loads((p/'release.json').read_bytes()); m=d['metadata']; results=[]
for a in m['assertions']:
    ex=[x for x in m['extensions'] if x['owner']==a['id']]
    results.append({'row':a['occurrence']['locator'],'eligibility':a['eligibility'],'unknown_extensions':[x for x in ex if 'opaque' in x['namespace']], 'diagnostics':[x for x in m['releases'][0]['diagnostics'] if x.get('occurrence')==a['occurrence']]})
result={'command':cmd,'exit':r.returncode,'raw_sha256':sha(raw),'metadata_sha256':sha(meta),'rows':results}
(p/'results.json').write_text(json.dumps(result,indent=2));print(json.dumps(result,indent=2))
