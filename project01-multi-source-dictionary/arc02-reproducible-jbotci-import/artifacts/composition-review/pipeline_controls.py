from pathlib import Path
import json,hashlib,subprocess,os,time
R=Path('/tmp/vlajmaji-p01-a02-arc-review/f20261024');S=Path('/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc02/vlajmaji');D=R/'pipeline-tiny';D.mkdir();records=[]
def sha(b):return hashlib.sha256(b).hexdigest()
def tree(path):return {str(p.relative_to(path)):{'bytes':len(p.read_bytes()),'sha256':sha(p.read_bytes())} for p in path.rglob('*') if p.is_file()}
raw=(R/'duplicate-raw.json').read_bytes();valid=(R/'duplicate-meta.toml').read_bytes();bad=valid.replace(b'definition_count = 2',b'definition_count = 99');(D/'raw').write_bytes(raw)
for key,meta in [('mismatch',bad),('valid',valid)]:
 (D/(key+'.meta')).write_bytes(meta);(D/(key+'.env')).write_text('version=1\nrevision='+'3d9fd41d9ed32de26311ee27ec86b6b165985aa1'+'\nraw_sha256='+sha(raw)+'\nmetadata_sha256='+sha(meta)+'\n')
curl=D/'fake-curl';curl.write_text('#!/bin/bash\nset -eu\nout=\nwhile [ "$#" -gt 0 ]; do\n case "$1" in --output) out=$2; shift 2 ;; https://*) url=$1; shift ;; *) shift ;; esac\ndone\ncase "$url" in */dictionary-en.json) cp "$FIXTURE_RAW" "$out" ;; */dictionary-en.metadata.toml) cp "$FIXTURE_META" "$out" ;; *) exit 91 ;; esac\n');curl.chmod(0o755)
env=os.environ.copy();env.update(VLAJMAJI_BIN=str(R/'vlajmaji-release-cli'),CURL=str(curl),CARGO_TARGET_DIR='/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc01/target',CARGO_NET_OFFLINE='true',FIXTURE_RAW=str(D/'raw'))
def run(name,args,expected,meta=None):
 if meta:env['FIXTURE_META']=str(D/(meta+'.meta'))
 start=time.monotonic();p=subprocess.Popen(args,stdout=subprocess.PIPE,stderr=subprocess.PIPE,env=env,cwd=S);so,se=p.communicate(timeout=60);(D/(name+'.stdout')).write_bytes(so);(D/(name+'.stderr')).write_bytes(se);records.append({'name':name,'argv':args,'pid':p.pid,'exit':p.returncode,'expected_exit':expected,'elapsed_s':time.monotonic()-start,'stdout_sha256':sha(so),'stderr_sha256':sha(se)});assert p.returncode==expected,(name,p.returncode,se.decode());return so,se
script=['/bin/bash',str(S/'scripts/jbotci-import.sh')]
so,se=run('update-mismatch',script+['update',str(D/'mismatch.env'),str(D/'mismatch-cache'),str(D/'mismatch-release.json'),str(D/'mismatch-store')],1,'mismatch')
assert b'jbotci_accounting' in se and (D/'mismatch-release.json').is_file() and (D/'mismatch-release.json.report.json').is_file() and not (D/'mismatch-store').exists()
leaf=next((D/'mismatch-cache').iterdir());assert not (leaf/'complete').exists();mreport=json.loads((D/'mismatch-release.json.report.json').read_text());assert not mreport['report']['definition_count_matches'] and mreport['report']['input_rows']==2
before=tree(D/'mismatch-cache');candidate=sha((D/'mismatch-release.json').read_bytes())
so,se=run('update-valid',script+['update',str(D/'valid.env'),str(D/'valid-cache'),str(D/'valid-release.json'),str(D/'valid-store')],0,'valid')
assert json.loads(so)['report']['definition_count_matches'];assert (D/'valid-store/dictionary.sqlite3').is_file();assert next((D/'valid-cache').glob('*/complete')).is_file()
assert tree(D/'mismatch-cache')==before and sha((D/'mismatch-release.json').read_bytes())==candidate
beforevalid=tree(D/'valid-cache');relbefore=sha((D/'valid-release.json').read_bytes())
so,se=run('check-no-write',script+['check',str(D/'valid.env'),str(D/'valid-cache'),str(D/'valid-release.json')],0)
assert tree(D/'valid-cache')==beforevalid and sha((D/'valid-release.json').read_bytes())==relbefore
so,se=run('occupied-valid-pin-replay',script+['replay',str(D/'valid.env'),str(D/'valid-cache'),str(D/'valid-release.json')],2)
assert b'release output already exists' in se and sha((D/'valid-release.json').read_bytes())==relbefore and tree(D/'valid-cache')==beforevalid
so,se=run('actual-cli-valid-structure-bad-raw-pin',[str(R/'vlajmaji-release-cli'),'jbotci-import',str(D/'raw'),str(D/'valid.meta'),str(D/'hash-failure.json'),'--raw-sha256','0'*64,'--metadata-sha256',sha(valid)],1)
assert so==b'' and json.loads(se)['error']['code']=='jbotci_hash_mismatch' and not (D/'hash-failure.json').exists()
assert sha((D/'valid-release.json').read_bytes())==relbefore
out={'verdict':'PASS','external_invocations':len(records),'records':records,'mismatch_retained_candidate_no_marker_no_store':True,'prior_history_unchanged':True,'check_no_mutation':True,'occupied_output_preserved':True,'exact_hash_error_no_output':True,'valid_counterpart':True};(R/'pipeline-controls-result.json').write_text(json.dumps(out,indent=2)+'\n');print(json.dumps(out,indent=2))
