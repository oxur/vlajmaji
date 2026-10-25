from pathlib import Path
import subprocess,os,json,hashlib,shutil,sys,stat,time,errno
R=Path('/tmp/vlajmaji-cli-default-store-cdc');W=R/'controls02';W.mkdir();C=Path(sys.argv[1]);B=R/'accepted-arc02-cli';records=[];checks=0
Q='{"operation":"exact","spelling":"farvi"}'
base=os.environ.copy();base.pop('HOME',None);base.pop('XDG_DATA_HOME',None)
def need(c,label):
 global checks
 checks+=1
 if not c:raise AssertionError(label)
def digest(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def run(label,args,home=None,xdg=None,stdin=None,code=0,binary=C):
 env=base.copy()
 if home is not None:env['HOME']=os.fspath(home)
 if xdg is not None:env['XDG_DATA_HOME']=os.fspath(xdg)
 t=time.monotonic();p=subprocess.Popen([os.fspath(binary),*map(os.fspath,args)],env=env,cwd=W,stdin=subprocess.PIPE if stdin is not None else subprocess.DEVNULL,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
 out,err=p.communicate(None if stdin is None else stdin.encode(),timeout=60)
 (W/(label+'.stdout')).write_bytes(out);(W/(label+'.stderr')).write_bytes(err)
 records.append({'label':label,'argv':[os.fsdecode(x) for x in args],'argv_native_hex':[os.fsencode(x).hex() for x in args],'HOME':None if home is None else os.fsdecode(home),'XDG_DATA_HOME':None if xdg is None else os.fsdecode(xdg),'pid':p.pid,'exit':p.returncode,'expected':code,'elapsed_s':time.monotonic()-t,'stdout_sha256':hashlib.sha256(out).hexdigest(),'stderr_sha256':hashlib.sha256(err).hexdigest()})
 (R/'controls-progress02.json').write_text(json.dumps({'records':records,'checks_before_exit':checks},indent=2)+'\n')
 need(p.returncode==code,(label,p.returncode,err))
 if code:
  need(not out,label+' stdout empty')
  return err
 need(not err,label+' stderr empty')
 return out
def obj(b):return json.loads(b)
def error(b,c):need(obj(b)['error']['code']==c,('error',c,b))
def default(x):return Path(x)/'vlajmaji/stores/default'
def query(label,path=None,home=None,xdg=None,stdin=False):return obj(run(label,['dictionary-query',*([path] if path is not None else []),*([] if stdin else ['--request',Q])],home,xdg,Q if stdin else None))
def info(label,path=None,home=None,xdg=None):return obj(run(label,['info',*([path] if path is not None else [])],home,xdg))
# Real default and explicit override, whole response identity.
x=W/'xdg space $quote';h=W/'ignored-home';v=obj(run('xdg-build',['dictionary-build',R/'tiny-release.json'],h,x));need(v['store']==str(default(x)),'exact default path');need(v['assertions']==2,'tiny count')
a=query('xdg-query',xdg=x);b=query('stdin-query',xdg=x,stdin=True);c=query('explicit-query',default(x));need(a==b==c,'whole query equality');need(a['totals']['occurrences']==2,'duplicate alternatives')
i=info('default-info',xdg=x);j=info('explicit-info',default(x));need(i==j,'whole info equality');need(i['format']=='dictionary' and i['source_count']==1 and len(i['sources'])==1,'truthful single-source info');need(i['sources'][0]['id']=='lensisku','Lensisku source');need(i['assertions']==2,'info assertions');m=json.loads((R/'tiny-release.json').read_bytes());need(i['sources']==m['registry']['sources'],'whole source records');need((i['captures'],i['occurrences'],i['entries'])==(2,2,2),'complete info counts')
need(not h.exists(),'ignored home untouched')
# Pure environment fallback discriminators and successful builds.
for label,xdg in [('unset',None),('empty',''),('relative','relative/should-not-exist')]:
 home=W/('home-'+label);v=obj(run(label+'-fallback-build',['dictionary-build',R/'tiny-release.json'],home,xdg));p=home/'.local/share/vlajmaji/stores/default';need(v['store']==str(p),label+' fallback');need(info(label+'-fallback-info',home=home,xdg=xdg)['assertions']==2,label+' opened')
 need(stat.S_IMODE((home/'.local').stat().st_mode)&0o077==0,label+' private parent')
need(not (W/'relative').exists(),'relative XDG ignored')
# Explicit path bypasses all invalid env and retains missing-parent semantics.
e=W/'explicit';v=obj(run('explicit-build',['dictionary-build',R/'tiny-release.json',e],home='relative',xdg='also-relative'));need(v['store']==str(e),'explicit exact');need(query('explicit-invalid-env-query',e,home='relative',xdg='also-relative')==a,'override response')
error(run('missing-home-query',['dictionary-query','--request',Q],code=1),'store_path');error(run('relative-home-info',['info'],home='relative',xdg='relative',code=1),'store_path')
# Read commands never create. Input errors preserve leaf and existing data.
m=W/'missing-default';error(run('missing-default-query',['dictionary-query','--request',Q],xdg=m,code=1),'store');error(run('missing-default-info',['info'],xdg=m,code=1),'store');need(not m.exists(),'no mkdir for reads')
mi=W/'missing-input-default';error(run('missing-input-build',['dictionary-build',W/'absent.json'],xdg=mi,code=1),'io');need(not mi.exists(),'missing input no dirs')
invalid=W/'invalid.json';invalid.write_text('{');bad=W/'invalid-default';error(run('invalid-json-build',['dictionary-build',invalid],xdg=bad,code=1),'invalid_dictionary');need(not default(bad).exists(),'invalid input no store leaf')
error(run('explicit-parent-missing',['dictionary-build',R/'tiny-release.json',W/'absent-parent/store'],code=1),'store');need(not (W/'absent-parent').exists(),'explicit no parent creation')
old=digest(default(x)/'dictionary.sqlite3');error(run('occupied-default-dir',['dictionary-build',R/'tiny-release.json'],xdg=x,code=1),'store');need(digest(default(x)/'dictionary.sqlite3')==old,'no overwrite default DB')
for kind in ['file','symlink']:
 z=W/('occupied-'+kind);p=default(z);p.parent.mkdir(parents=True);sent=W/('sentinel-'+kind);sent.write_bytes(b'preserve exact bytes')
 if kind=='file':p.write_bytes(b'preserve exact bytes')
 else:p.symlink_to(sent)
 error(run('occupied-default-'+kind,['dictionary-build',R/'tiny-release.json'],xdg=z,code=1),'store');need(sent.read_bytes()==b'preserve exact bytes','referent preserved');need(p.read_bytes()==b'preserve exact bytes','occupied bytes preserved');need(p.is_symlink()==(kind=='symlink'),'kind preserved')
blocked=W/'blocked';blocked.write_bytes(b'parent file');error(run('blocked-parent',['dictionary-build',R/'tiny-release.json'],xdg=blocked,code=1),'io');need(blocked.read_bytes()==b'parent file','blocked bytes')
pre=W/'existing-mode';pre.mkdir();pre.chmod(0o755);run('existing-parent-mode',['dictionary-build',R/'tiny-release.json'],xdg=pre);need(stat.S_IMODE(pre.stat().st_mode)==0o755,'existing mode unchanged');need(stat.S_IMODE((pre/'vlajmaji').stat().st_mode)&0o077==0,'new private dir')
# Native-byte query/info positives and build JSON boundary, no lossy routing.
native=os.fsdecode(os.fsencode(W)+b'/native-\xff');np=default(native);native_fs={}
try:
 Path(native).mkdir()
except OSError as ex:
 need(ex.errno==errno.EILSEQ,('unexpected native filesystem error',ex.errno));native_fs={'supports_non_utf8_component':False,'errno':ex.errno,'error':str(ex)}
 # Deliberately valid lossy spelling makes forbidden lossy routing observably wrong.
 lossy=Path(os.fsencode(native).decode('utf-8','replace'));lp=default(lossy);lp.parent.mkdir(parents=True);shutil.copytree(default(x),lp)
 error(run('native-default-query-fs-boundary',['dictionary-query','--request',Q],xdg=native,code=1),'store')
 error(run('native-default-info-fs-boundary',['info'],xdg=native,code=1),'store')
 need(query('lossy-spelling-valid-counterpart',lp)==a,'lossy named store is a valid discriminating counterpart')
else:
 native_fs={'supports_non_utf8_component':True};np.parent.mkdir(parents=True);shutil.copytree(default(x),np)
 need(query('native-default-query',xdg=native)==a,'native default routing');need(info('native-default-info',xdg=native)==i,'native info routing');need(query('native-explicit-query',np)==a,'native explicit routing')
badnative=os.fsdecode(os.fsencode(W)+b'/build-\xfe');error(run('native-build-json-error',['dictionary-build',R/'tiny-release.json'],xdg=badnative,code=1),'store_path');need(not Path(badnative).exists(),'nonUTF8 build before mkdir')
need(query('utf8-override-native-env',e,home=native,xdg=badnative)==a,'explicit bypass native env')
# Exact unchanged legacy info wire and format-discriminating failures.
legacy=W/'legacy';run('baseline-legacy-build',['build',R/'legacy-empty-v2.json',legacy],binary=B);oldinfo=obj(run('baseline-legacy-info',['info',legacy],binary=B));newinfo=info('candidate-legacy-info',legacy);need(oldinfo==newinfo=={'records':0,'snapshots':0,'morphology_enabled':True},'legacy info unchanged')
for command in ['build','search']:
 out=run('legacy-'+command+'-still-required',[command],code=2);need(b'error:' in out,'Clap explicit legacy path')
corrupt=W/'corrupt';corrupt.mkdir();(corrupt/'dictionary.sqlite3').write_bytes(b'corrupt database');error(run('corrupt-grouped-info',['info',corrupt],code=1),'store')
shutil.copyfile(legacy/'index.sqlite3',corrupt/'index.sqlite3');error(run('ambiguous-format-info',['info',corrupt],code=1),'store_format')
need((corrupt/'dictionary.sqlite3').read_bytes()==b'corrupt database','corrupt not replaced')
ld=W/'default-legacy-corrupt';lp=default(ld);lp.mkdir(parents=True);(lp/'index.sqlite3').write_bytes(b'corrupt legacy data')
error(run('default-legacy-corrupt-info',['info'],xdg=ld,code=1),'store')
old_error=run('baseline-explicit-legacy-corrupt-info',['info',lp],code=1,binary=B);new_error=run('explicit-legacy-corrupt-info',['info',lp],code=1);need(old_error==new_error and not new_error.lstrip().startswith(b'{'),'legacy error wire identical');need((lp/'index.sqlite3').read_bytes()==b'corrupt legacy data','corrupt legacy unchanged')
for command in [None,'dictionary-build','dictionary-query','info']:
 text=run('help-'+str(command),([] if command is None else [command])+['--help']).decode();need('vlajmaji/stores/default' in text and 'XDG_DATA_HOME' in text,'help default '+str(command))
need(digest(default(x)/'dictionary.sqlite3')==old,'final original DB unchanged')
result={'verdict':'PASS','source':sys.argv[2],'binary_sha256':digest(C),'calls':len(records),'checks':checks,'records':records,'tiny_release_sha256':digest(R/'tiny-release.json'),'legacy_archive_sha256':digest(R/'legacy-empty-v2.json'),'default_store_sha256':old,'native_filesystem':native_fs}
(R/'controls-result02.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:v for k,v in result.items() if k!='records'},indent=2))
