#!/usr/bin/env python3
"""Independent CDC pipeline process controls; prepared before S02 candidate freeze.
Only fake transport is used; all validation/import/store/query is the actual supplied CLI.
"""
import hashlib,json,os,shutil,subprocess,sys,time
from pathlib import Path

def digest(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def snapshot(p):
 p=Path(p)
 if not p.exists():return {}
 return {str(x.relative_to(p)):('link:'+os.readlink(x) if x.is_symlink() else digest(x)) for x in sorted(p.rglob('*')) if x.is_file() or x.is_symlink()}
def main():
 source,cli,out=map(Path,sys.argv[1:4]);source=source.resolve();cli=cli.resolve();out=out.resolve();out.mkdir(exist_ok=False)
 script=source/'scripts/jbotci-import.sh';records=[];revision='0'*40
 fixture=Path('/tmp/vlajmaji-p01-a02-cdc/api-fixtures03/duplicate-native-and-complete')
 raw=out/'transport raw.json';meta=out/'transport meta.toml';shutil.copyfile(fixture/'raw.json',raw);shutil.copyfile(fixture/'meta.toml',meta)
 pins=out/'valid pins.env'
 def pintext(rev=revision,metadata=meta):return f'version=1\nrevision={rev}\nraw_sha256={digest(raw)}\nmetadata_sha256={digest(metadata)}\n'
 pins.write_text(pintext());calls=out/'curl calls.txt'
 curl=out/'fake curl'
 curl.write_text('''#!/bin/bash
set -eu
printf '%s\\n' "$*" >> "$CDC_CURL_CALLS"
output=;url=
while [ "$#" -gt 0 ]; do
 case "$1" in -o|--output) output=$2;shift 2;; https://*) url=$1;shift;; *) shift;; esac
done
[ -n "$output" ] && [ -n "$url" ] || exit 91
[ "${CDC_CURL_FAIL:-}" != all ] || exit 92
case "$url" in
 "https://raw.githubusercontent.com/int19h/jbotci/$CDC_REV/crates/jbotci-dictionary-data/data/dictionary-en.json") [ "${CDC_CURL_FAIL:-}" != raw ] || exit 95;cp "$CDC_RAW" "$output";;
 "https://raw.githubusercontent.com/int19h/jbotci/$CDC_REV/crates/jbotci-dictionary-data/data/dictionary-en.metadata.toml") [ "${CDC_CURL_FAIL:-}" != metadata ] || exit 93;cp "$CDC_META" "$output";;
 *) exit 94;;
esac
''');curl.chmod(0o755)
 cargo_probe=out/'forbidden cargo probe';cargo_probe.write_text('#!/bin/bash\nprintf called > "$CDC_CARGO_FORBIDDEN"\nexit 99\n');cargo_probe.chmod(0o755)
 alias=out/'actual CLI $ ` ; quote \" \' ';alias.symlink_to(cli)
 env=os.environ.copy();env.update({'VLAJMAJI_BIN':str(alias),'CURL':str(curl),'CDC_CURL_CALLS':str(calls),'CDC_RAW':str(raw),'CDC_META':str(meta),'CDC_REV':revision,'CDC_CARGO_FORBIDDEN':str(out/'CARGO_MUST_NOT_RUN'),'CARGO_NET_OFFLINE':'true','PYTHONDONTWRITEBYTECODE':'1'})
 env['CARGO_TARGET_DIR']='/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc01/target'
 def run(name,args,ok=True,extra=None,make=False):
  e=env.copy();e.update(extra or {});e={k:v for k,v in e.items() if v is not None};start=time.monotonic()
  command=(['make','--no-print-directory','-C',str(source)]+args if make else ['/bin/bash',str(script)]+list(map(str,args)))
  p=subprocess.run(command,env=e,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
  (out/(name+'.stdout')).write_bytes(p.stdout);(out/(name+'.stderr')).write_bytes(p.stderr)
  r={'case':name,'argv':command,'returncode':p.returncode,'elapsed_seconds':time.monotonic()-start,'expected_success':ok};records.append(r)
  (out/'attempts.json').write_text(json.dumps(records,indent=2))
  assert (p.returncode==0)==ok,(name,p.returncode,p.stderr.decode(errors='replace'))
  return p
 def leaf(cache,rev=revision,metadata=meta):return cache/(rev+'-'+digest(raw)+'-'+digest(metadata))
 def countcalls():return len(calls.read_text().splitlines()) if calls.exists() else 0
 def freshname(name):return out/name
 # Help cannot build or fetch, even without the executable/target.
 e=env.copy();e.pop('VLAJMAJI_BIN',None);e.pop('CARGO_TARGET_DIR',None);e['CARGO']='/bin/false'
 start=time.monotonic();p=subprocess.run(['/bin/bash',str(script),'help'],env=e,capture_output=True)
 (out/'help-no-build.stdout').write_bytes(p.stdout);(out/'help-no-build.stderr').write_bytes(p.stderr)
 records.append({'case':'help-no-build','returncode':p.returncode,'elapsed_seconds':time.monotonic()-start});assert p.returncode==0 and countcalls()==0
 # Every malformed manifest case must fail before creating cache or invoking transport.
 valid=pintext().encode();bad={
 'unknown-key':valid+b'other=1\n','duplicate-key':valid+b'version=1\n','missing-key':b'\n'.join(valid.splitlines()[:-1])+b'\n',
 'bad-version':valid.replace(b'version=1',b'version=2'),'uppercase-revision':valid.replace(('revision='+revision).encode(),b'revision='+b'A'*40),
 'short-digest':valid.replace(digest(raw).encode(),b'a'*63),'quoted-value':valid.replace(b'version=1',b'version="1"'),
 'space-assignment':valid.replace(b'version=1',b'version =1'),'trailing-text':valid.replace(b'version=1',b'version=1 #comment'),
 'cr':valid+b'\r','tab':valid+b'\t','nul':valid+b'\0','nonascii':valid+'é'.encode(),
 'command-substitution':valid.replace(b'version=1',('version=$(touch '+str(out/'MANIFEST_SENTINEL')+')').encode()),
 }
 for name,body in bad.items():
  pin=out/(name+'.env');pin.write_bytes(body);cache=out/(name+' cache');before=countcalls()
  run('manifest-'+name,['acquire',pin,cache],False);assert not cache.exists() and countcalls()==before
 assert not (out/'MANIFEST_SENTINEL').exists()
 # Well-shaped but false digest pins reach real Rust validation and retain the failed input attempt.
 for field in ['raw_sha256','metadata_sha256']:
  pin=out/('false-'+field+'.env');body=pintext();true=digest(raw) if field=='raw_sha256' else digest(meta);pin.write_text(body.replace(field+'='+true,field+'='+'f'*64))
  cachebad=out/('false-'+field+' cache');run('acquire-false-'+field,['acquire',pin,cachebad],False)
  leaves=list(cachebad.iterdir());assert len(leaves)==1 and (leaves[0]/'dictionary-en.json').read_bytes()==raw.read_bytes() and not (leaves[0]/'complete').exists()
 # Actual valid counterpart acquires two exact fixed URLs and completes only after Rust check.
 cache=out/'valid cache';p=run('acquire-valid',['acquire',pins,cache]);json.loads(p.stdout)
 good=leaf(cache);assert (good/'complete').read_bytes()==b'vlajmaji-jbotci-cache-v1\n';assert digest(good/'dictionary-en.json')==digest(raw);assert digest(good/'dictionary-en.metadata.toml')==digest(meta)
 stable=snapshot(cache);nc=countcalls();run('acquire-reuse-offline',['acquire',pins,cache],extra={'CDC_CURL_FAIL':'all'});assert countcalls()==nc and snapshot(cache)==stable
 for name,body in [('comments-and-blanks','# comment\n   # indented comment\n\n   \n'+pintext()),('no-final-lf',pintext().rstrip('\n'))]:
  accepted=out/(name+'.env');accepted.write_text(body);before=countcalls();run('manifest-valid-'+name,['acquire',accepted,cache],extra={'CDC_CURL_FAIL':'all'});assert countcalls()==before and snapshot(cache)==stable
 release=out/'release.json';store=out/'store';p=run('replay-valid',['replay',pins,cache,release,store],extra={'CDC_CURL_FAIL':'all'})
 assert p.stdout==(out/'release.json.report.json').read_bytes();json.loads(p.stdout);assert countcalls()==nc and snapshot(cache)==stable
 request=json.dumps({'operation':'exact','spelling':'ma'})
 start=time.monotonic();query=subprocess.run([str(cli),'dictionary-query',str(store),'--request',request],env=env,capture_output=True)
 (out/'duplicate-query.stdout').write_bytes(query.stdout);(out/'duplicate-query.stderr').write_bytes(query.stderr)
 records.append({'case':'duplicate-query','returncode':query.returncode,'elapsed_seconds':time.monotonic()-start});assert query.returncode==0
 response=json.loads(query.stdout);hits=response['groups'][0]['hits'];assert len(hits)==2 and len({h['occurrence_id'] for h in hits})==2
 assert hits[0]['occurrence_attribution']['native_assertion_id']==hits[1]['occurrence_attribution']['native_assertion_id']
 releasehash=digest(release);storebefore=snapshot(store);run('check-valid',['check',pins,cache,release],extra={'CDC_CURL_FAIL':'all'});assert digest(release)==releasehash and snapshot(store)==storebefore and snapshot(cache)==stable and countcalls()==nc
 # Sidecar is evidence only: check must ignore its contents and use whole canonical regeneration.
 report=out/'release.json.report.json';saved=report.read_bytes();report.write_text('deliberately not JSON\n');run('check-report-not-authority',['check',pins,cache,release]);assert report.read_text()=='deliberately not JSON\n';report.write_bytes(saved)
 # A missing cache must remain absent on check.
 missing=out/'missing cache';run('check-missing-cache',['check',pins,missing],False);assert not missing.exists()
 # Occupied valid destinations, including dangling links/directories, remain byte exact.
 for kind in ['file','directory','symlink']:
  destination=out/('occupied-'+kind)
  if kind=='file':destination.write_bytes(b'caller bytes\n')
  elif kind=='directory':destination.mkdir();(destination/'witness').write_bytes(b'caller bytes\n')
  else:destination.symlink_to(out/'never-created-target')
  before=snapshot(out);run('occupied-'+kind,['replay',pins,cache,destination],False)
  if kind=='file':assert destination.read_bytes()==b'caller bytes\n'
  elif kind=='directory':assert (destination/'witness').read_bytes()==b'caller bytes\n'
  else:assert destination.is_symlink() and not destination.exists()
 # Sidecar and optional store occupancy must be detected before the fresh release is written.
 for kind in ['report','store']:
  fresh=out/('blocked-'+kind+' release.json');blocked=Path(str(fresh)+'.report.json') if kind=='report' else out/'blocked store'
  blocked.write_bytes(b'caller-owned destination\n');before=blocked.read_bytes()
  args=['replay',pins,cache,fresh]+([blocked] if kind=='store' else [])
  run('occupied-'+kind,args,False);assert not fresh.exists() and blocked.read_bytes()==before
 firstfail=out/'first-download failure cache';before=countcalls();p=run('acquire-first-download-only-failure',['acquire',pins,firstfail],False,{'CDC_CURL_FAIL':'raw'});assert p.returncode==95 and countcalls()==before+1
 firstleaf=leaf(firstfail);assert firstleaf.is_dir() and not (firstleaf/'complete').exists() and not (firstleaf/'dictionary-en.metadata.toml').exists() and str(firstleaf).encode() in p.stderr
 # Partial acquisition is retained and cannot later be silently adopted.
 partial=out/'partial cache';p=run('acquire-partial',['acquire',pins,partial],False,{'CDC_CURL_FAIL':'metadata'});pl=leaf(partial);assert p.returncode==93 and str(pl).encode() in p.stderr
 assert (pl/'dictionary-en.json').read_bytes()==raw.read_bytes() and not (pl/'complete').exists();before=snapshot(partial);nc=countcalls()
 run('partial-retry-rejected',['acquire',pins,partial],False);assert snapshot(partial)==before and countcalls()==nc
 # Corrupt marker/input and symlink are rejected without network or repair.
 for name in ['marker','raw','raw-link','marker-link','leaf-link']:
  target=out/('corrupt-'+name);target.mkdir();cl=leaf(target)
  if name=='leaf-link':cl.symlink_to(good,target_is_directory=True)
  else:
   shutil.copytree(good,cl)
   if name=='marker':(cl/'complete').write_bytes(b'vlajmaji-jbotci-cache-v1')
   elif name=='raw':(cl/'dictionary-en.json').write_bytes(b'[]')
   elif name=='raw-link':(cl/'dictionary-en.json').unlink();(cl/'dictionary-en.json').symlink_to(raw)
   else:(cl/'complete').unlink();(cl/'complete').symlink_to(good/'complete')
  before=snapshot(target);nc=countcalls();run('cache-'+name,['check',pins,target],False,{'CDC_CURL_FAIL':'all'});assert snapshot(target)==before and countcalls()==nc
  if name=='raw':
   invalidrelease=out/'corrupt-cache replay.json';run('replay-corrupt-cache-no-output',['replay',pins,target,invalidrelease],False,{'CDC_CURL_FAIL':'all'});assert not invalidrelease.exists() and not Path(str(invalidrelease)+'.report.json').exists() and snapshot(target)==before
 # A structurally importable but count-mismatched update retains release/report and no activation/store.
 mismatchmeta=out/'mismatch metadata.toml';text=meta.read_text();assert 'definition_count = 2' in text;mismatchmeta.write_text(text.replace('definition_count = 2','definition_count = 3'))
 mismatchpins=out/'mismatch pins.env';mismatchpins.write_text(pintext(metadata=mismatchmeta));badrelease=out/'mismatch release.json';badstore=out/'mismatch store';old=snapshot(cache)
 p=run('update-count-mismatch',['update',mismatchpins,cache,badrelease,badstore],False,{'CDC_META':str(mismatchmeta)})
 assert badrelease.is_file() and Path(str(badrelease)+'.report.json').is_file() and not badstore.exists() and not (leaf(cache,metadata=mismatchmeta)/'complete').exists()
 reportbody=json.loads(Path(str(badrelease)+'.report.json').read_bytes());assert reportbody
 for path,value in old.items():assert snapshot(cache)[path]==value
 # Valid update counterpart, then provenance-only revision update: occurrence identities are stable.
 updated=out/'valid update.json';run('update-valid',['update',pins,cache,updated]);assert updated.read_bytes()==release.read_bytes()
 nextrev='1'*40;nextpins=out/'new revision pins.env';nextpins.write_text(pintext(rev=nextrev));newrelease=out/'new revision release.json';run('update-provenance-only',['update',nextpins,cache,newrelease],extra={'CDC_REV':nextrev})
 olddoc=json.loads(release.read_bytes());newdoc=json.loads(newrelease.read_bytes())
 assert olddoc['metadata']['releases'][0]['id']!=newdoc['metadata']['releases'][0]['id']
 assert olddoc['metadata']['assertions']==newdoc['metadata']['assertions'] and [(c['identity'],c['bytes'],c['occurrences']) for c in olddoc['captures']]==[(c['identity'],c['bytes'],c['occurrences']) for c in newdoc['captures']]
 # Make literal values must survive both GNU Make and the shell without command execution.
 sentinel=out/'SHOULD_NOT_EXIST';literal=out/('literal $(shell touch '+str(sentinel)+') `touch '+str(sentinel)+'` ; \' " $ cache')
 run('make-literal-acquire',['jbotci-acquire'],extra={'JBOTCI_PINS':str(pins),'JBOTCI_CACHE':str(literal)},make=True);assert leaf(literal).is_dir() and not sentinel.exists() 
 literal2=out/('commandline $(shell touch '+str(sentinel)+') `touch '+str(sentinel)+'` ; \' \" $ cache')
 run('make-commandline-literal-acquire',['jbotci-acquire','JBOTCI_PINS='+str(pins),'JBOTCI_CACHE='+str(literal2)],make=True);assert leaf(literal2).is_dir() and not sentinel.exists()
 # Missing explicit target must fail, with valid pins/cache; existing Make's default target cannot authorize fallback.
 p=run('make-missing-target',['jbotci-check'],False,{'VLAJMAJI_BIN':'','CARGO_TARGET_DIR':None,'JBOTCI_PINS':str(pins),'JBOTCI_CACHE':str(cache),'CARGO':str(cargo_probe)},make=True)
 assert not (out/'CARGO_MUST_NOT_RUN').exists() and b'CARGO_TARGET_DIR' in p.stderr
 run('make-child-failure',['jbotci-check'],False,{'VLAJMAJI_BIN':'/bin/false','JBOTCI_PINS':str(pins),'JBOTCI_CACHE':str(cache)},make=True)
 assert snapshot(cache).keys()>=stable.keys()
 result={'verdict':'PASS','cases':len(records),'attempts':records,'raw_sha256':digest(raw),'metadata_sha256':digest(meta),'source_script_sha256':digest(script),'cli_sha256':digest(cli),'note':'Fake curl transports exact source fixtures only; real CLI validates/imports/persists. No Cargo fallback execution claimed.'}
 (out/'result.json').write_text(json.dumps(result,indent=2));print(json.dumps(result,indent=2))
if __name__=='__main__':main()
