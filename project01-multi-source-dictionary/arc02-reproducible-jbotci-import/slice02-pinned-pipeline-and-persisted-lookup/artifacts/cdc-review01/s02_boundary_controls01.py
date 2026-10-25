#!/usr/bin/env python3
import hashlib,json,os,shutil,subprocess,sys,time
from pathlib import Path
source=Path(sys.argv[1]);cli=Path(sys.argv[2]);out=Path(sys.argv[3]);out.mkdir(exist_ok=False)
fixture=Path('/tmp/vlajmaji-p01-a02-cdc/api-fixtures03/duplicate-native-and-complete');raw=fixture/'raw.json';meta=fixture/'meta.toml';sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();revision='0'*40
text=f'version=1\nrevision={revision}\nraw_sha256={sha(raw)}\nmetadata_sha256={sha(meta)}\n';pins=out/'valid.env';pins.write_text(text);calls=out/'curl-calls';sentinel=b'COMPETING CALLER DATA\n'
curl=out/'curl';curl.write_text('''#!/bin/bash
set -eu
printf '%s\\n' "$*" >> "$CDC_CALLS"
out=;url=
while [ "$#" -gt 0 ]; do case "$1" in --output|-o) out=$2;shift 2;; https://*) url=$1;shift;; *) shift;; esac;done
case "$url" in
 "https://raw.githubusercontent.com/int19h/jbotci/$CDC_REV/crates/jbotci-dictionary-data/data/dictionary-en.json") cp "$CDC_RAW" "$out";;
 "https://raw.githubusercontent.com/int19h/jbotci/$CDC_REV/crates/jbotci-dictionary-data/data/dictionary-en.metadata.toml") cp "$CDC_META" "$out";if [ -n "${CDC_DOWNLOAD_REPORT:-}" ];then (set -C;printf 'COMPETING CALLER DATA\\n' > "$CDC_DOWNLOAD_REPORT");fi;;
 *) exit 94;;
esac
''');curl.chmod(0o755)
wrapper=out/'real-cli-wrapper';wrapper.write_text('''#!/bin/bash
set -eu
"$CDC_REAL_CLI" "$@"
if [ "$1" = jbotci-check ] && [ -n "${CDC_CHECK_REPORT:-}" ];then
 case " $* " in *' --release '*) ;; *) (set -C;printf 'COMPETING CALLER DATA\\n' > "$CDC_CHECK_REPORT");;esac
fi
''');wrapper.chmod(0o755)
env={**os.environ,'VLAJMAJI_BIN':str(cli),'CURL':str(curl),'CDC_REAL_CLI':str(cli),'CDC_CALLS':str(calls),'CDC_REV':revision,'CDC_RAW':str(raw),'CDC_META':str(meta),'CARGO_TARGET_DIR':'/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc01/target','CARGO_NET_OFFLINE':'true','PYTHONDONTWRITEBYTECODE':'1'}
records=[]
def run(name,args,extra=None):
 e={**env,**(extra or {})};argv=['/bin/bash',str(source/'scripts/jbotci-import.sh'),*map(str,args)];start=time.monotonic();p=subprocess.run(argv,cwd=source,env=e,capture_output=True)
 (out/(name+'.stdout')).write_bytes(p.stdout);(out/(name+'.stderr')).write_bytes(p.stderr)
 r={'case':name,'argv':argv,'environment_overrides':extra or {},'returncode':p.returncode,'elapsed_seconds':time.monotonic()-start};records.append(r);return p,r
cache=out/'valid-cache';p,r=run('valid-regular-counterpart',['acquire',pins,cache]);assert p.returncode==0,p.stderr;json.loads(p.stdout);r['contract_pass']=True
for name,body,success in [('nul-forbidden',text.encode()+b'\0\n',False),('indented-assignment',text.replace('revision=',' revision=').encode(),False),('no-final-lf',text.rstrip('\n').encode(),True)]:
 pin=out/(name+'.env');pin.write_bytes(body);target=out/(name+'-cache') if not success else cache;before=len(calls.read_bytes())
 p,r=run(name,['acquire',pin,target]);r.update(expected_success=success,cache_created=target.exists(),transport_bytes_added=len(calls.read_bytes())-before)
 r['contract_pass']=(p.returncode==0)==success and (success or (not target.exists() and r['transport_bytes_added']==0))
link=out/'executable-link';link.symlink_to(cli);p,r=run('executable-symlink',['check',pins,cache],{'VLAJMAJI_BIN':str(link)});r.update(expected_success=True,contract_pass=p.returncode==0)
pinlink=out/'pin-link';pinlink.symlink_to(pins);p,r=run('pin-symlink',['check',pinlink,cache]);r.update(expected_success=True,contract_pass=p.returncode==0)
release=out/'replay.json';report=Path(str(release)+'.report.json');p,r=run('exclusive-replay-report',['replay',pins,cache,release],{'VLAJMAJI_BIN':str(wrapper),'CDC_CHECK_REPORT':str(report)})
r.update(expected_success=False,report_preserved=report.exists() and report.read_bytes()==sentinel,release_exists=release.exists());r['contract_pass']=p.returncode!=0 and r['report_preserved']
release=out/'update.json';report=Path(str(release)+'.report.json');p,r=run('exclusive-update-report',['update',pins,out/'update-cache',release],{'CDC_DOWNLOAD_REPORT':str(report)})
r.update(expected_success=False,report_preserved=report.exists() and report.read_bytes()==sentinel,release_exists=release.exists());r['contract_pass']=p.returncode!=0 and r['report_preserved']
result={'source_head':subprocess.check_output(['git','rev-parse','HEAD'],cwd=source,text=True).strip(),'script_sha256':sha(source/'scripts/jbotci-import.sh'),'cli_sha256':sha(cli),'bash':subprocess.check_output(['/bin/bash','--version'],text=True),'cases':records,'controls':len(records),'disagreements':[r['case'] for r in records if not r['contract_pass']],'note':'Real Rust validation/import/check throughout; fake curl only transports exact synthetic bytes and creates one caller-owned competing report at the prescribed boundary. Wrapper delegates actual CLI and creates a report only after successful raw validation.'}
(out/'result.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result,indent=2))
