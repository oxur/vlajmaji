from pathlib import Path
import os,sys,subprocess,json,time,hashlib,shutil
R=Path('/tmp/vlajmaji-pr3-byte-cdc01');S=Path('/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-store-verification/vlajmaji');SCRIPT=S/'scripts/jbotci-import.sh';HEAD=sys.argv[1]
assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=S,text=True).strip()==HEAD
assert not subprocess.check_output(['git','status','--porcelain'],cwd=S)
W=R/'cases';W.mkdir();records=[]
valid=b'version=1\nrevision='+b'0'*40+b'\nraw_sha256='+b'a'*64+b'\nmetadata_sha256='+b'b'*64+b'\n'
cases=[('original-0-unknown',valid+b'unknown=x\n',False),('original-1-missing',valid[:valid.index(b'metadata_sha256')],False),('original-2-uppercase',valid.replace(b'a'*64,b'A'*64),False),('original-3-crlf',valid.replace(b'version=1\n',b'version=1\r\n'),False),('original-4-trailing',valid+b'trailing = junk\n',False),('original-5-indent',valid.replace(b'revision=',b' revision='),False),('original-6-nul',valid+b'\0\n',False),('original-7-nonascii',valid+'# snowman: ☃\n'.encode(),False)]
for name,value in [('tab',b'\t'),('escape',b'\x1b'),('del',b'\x7f'),('high80',b'\x80'),('highff',b'\xff'),('embedded-nul',b'\0')]:cases.append(('byte-'+name,b'# comment'+value+b'\n'+valid,False))
cases += [('valid-ascii-lf',b'# '+bytes(range(32,127))+b'\n'+valid,True),('valid-ascii-unterminated',b'# '+bytes(range(32,127))+b'\n'+valid.rstrip(b'\n'),True),('valid-lf-empty-comments',b'\n  # blank/comment control\n'+valid,True)]
for failed in [None,'tr','wc']:
 chosen=cases if failed is None else [('failed-'+failed,valid,False)]
 for name,body,ok in chosen:
  d=W/name;d.mkdir();pins=d/'pins.env';pins.write_bytes(body);cache=d/'cache';transport=d/'transport.log';cli_log=d/'cli.log';curl=d/'curl';cli=d/'cli'
  curl.write_text('''#!/bin/bash
set -eu
out=
while [[ $# -gt 0 ]]; do
 if [[ $1 == --output ]]; then out=$2; shift 2; else shift; fi
done
[[ -n $out ]]
printf 'curl\\n' >> "$TRANSPORT_LOG"
printf 'synthetic byte-control payload\\n' > "$out"
''');cli.write_text('''#!/bin/bash
set -eu
printf '%s\\n' "$*" >> "$CLI_LOG"
printf '{"ok":true}\\n'
''');curl.chmod(0o700);cli.chmod(0o700)
  env=os.environ.copy();env.update(CURL=str(curl),VLAJMAJI_BIN=str(cli),TRANSPORT_LOG=str(transport),CLI_LOG=str(cli_log));env.pop('HOME',None);env.pop('XDG_DATA_HOME',None)
  if failed:
   tools=d/'tools';tools.mkdir();stub=tools/failed;stub.write_text('#!/bin/bash\nprintf "CDC_'+failed+'_FAIL\\n" >&2\nexit 7\n');stub.chmod(0o700);env['PATH']=str(tools)+':'+env['PATH']
  argv=['/bin/bash',str(SCRIPT),'acquire',str(pins),str(cache)];start=time.monotonic();p=subprocess.Popen(argv,cwd=S,env=env,stdin=subprocess.DEVNULL,stdout=subprocess.PIPE,stderr=subprocess.PIPE);out,err=p.communicate(timeout=20);elapsed=time.monotonic()-start;(d/'stdout').write_bytes(out);(d/'stderr').write_bytes(err)
  entry={'name':name,'manifest_hex':body.hex(),'argv':argv,'pid':p.pid,'exit':p.returncode,'elapsed_s':elapsed,'expected_valid':ok,'stdout_sha256':hashlib.sha256(out).hexdigest(),'stderr_sha256':hashlib.sha256(err).hexdigest(),'stderr':err.decode(errors='replace'),'cache_exists':cache.exists(),'transport_calls':len(transport.read_text().splitlines()) if transport.exists() else 0,'cli_calls':len(cli_log.read_text().splitlines()) if cli_log.exists() else 0};records.append(entry);(R/'progress.json').write_text(json.dumps(records,indent=2)+'\n')
  if ok:
   assert p.returncode==0 and not err and entry['transport_calls']==2 and entry['cli_calls']==1,entry
   leaf=list(cache.iterdir());assert len(leaf)==1 and (leaf[0]/'complete').read_bytes()==b'vlajmaji-jbotci-cache-v1\n',entry
  else:
   assert p.returncode!=0 and not out and not cache.exists() and not transport.exists() and not cli_log.exists(),entry
   if name.startswith('byte-') or name in ['original-3-crlf','original-6-nul','original-7-nonascii']:assert b'only printable ASCII and LF' in err,entry
   if failed:assert ('CDC_'+failed+'_FAIL').encode() in err,entry
result={'verdict':'PASS','source':HEAD,'script_sha256':hashlib.sha256(SCRIPT.read_bytes()).hexdigest(),'bash':subprocess.check_output(['/bin/bash','--version'],text=True).splitlines()[0],'original_invalid':8,'added_forbidden':6,'valid_printable_lf_eof':3,'failed_filters':2,'cases':len(records),'records':records,'limits':'Synthetic transport/CLI stubs exercise byte-preflight and no-sideeffect process boundary only; native cargo pipeline tests exercise real tiny importer. BSDmac Bash3.2 local proof, no GNU/Linux equivalence claim; hosted LinuxCI remains final platform proof.'};(R/'result.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:v for k,v in result.items() if k!='records'},indent=2))
