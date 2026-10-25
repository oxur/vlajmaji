from pathlib import Path
import subprocess,os,json,hashlib,time
r=Path('/tmp/vlajmaji-p01-a03-cdc/s01');p=r/'cli-controls01';p.mkdir();cli=r/'candidate042-cli';rows=[];env={k:v for k,v in os.environ.items() if k not in ['HOME','XDG_DATA_HOME']}
def run(name,args,code=None):
 t=time.monotonic();q=subprocess.run([os.fsencode(cli)]+[os.fsencode(a) for a in args],cwd=p,env=env,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
 (p/(name+'.stdout')).write_bytes(q.stdout);(p/(name+'.stderr')).write_bytes(q.stderr)
 row={'name':name,'argv_fshex':[os.fsencode(a).hex() for a in args],'exit':q.returncode,'elapsed':time.monotonic()-t};rows.append(row)
 if code is None:assert q.returncode==0,(name,q.stderr)
 else:
  assert q.returncode!=0,name
  error=json.loads(q.stderr);assert error['error']['code']==code,(name,error)
 return json.loads(q.stdout) if q.stdout and code is None and '--help' not in args else None
q=run('fixture',['data-pack','fixture','jbotci',str(p/'fixture')]);assert q['payload_kind']=='synthetic';assert (q['report']['input_rows'],q['report']['lexical_occurrences'],q['report']['excluded_occurrences'])==(3,2,1)
for kind in ['directory','file','symlink']:
 dest=p/('occupied-'+kind)
 if kind=='directory':dest.mkdir();sent=dest/'keep';sent.write_bytes(b'untouched')
 elif kind=='file':dest.write_bytes(b'untouched');sent=dest
 else:dest.symlink_to(p/'occupied-directory',target_is_directory=True);sent=p/'occupied-directory/keep'
 run('occupied-'+kind,['data-pack','fixture','jbotci',str(dest)],'io');assert sent.read_bytes()==b'untouched'
raw=b'[{"word":"farvi","word_type":"gismu","definition":"first"},{"word":"farvi","word_type":"gismu","definition":"second"}]'
def digest(b):return hashlib.sha256(b).hexdigest()
def meta(raw,count=2):return ('language_tag = "en"\nlanguage_realname = "English"\nsource_language_tag = "jbo"\nformat = "json"\npositive_scores_only = false\nfilename = "dictionary-en.json"\nmetadata_url = "https://example.invalid/languages"\ndownload_url = "https://example.invalid/synthetic-export"\nlensisku_created_at = "2026-10-04T03:07:08Z"\nsha256 = "'+digest(raw)+'"\ndefinition_count = '+str(count)+'\nentry_count = 1\n').encode()
def material(name,rb,mb,code=None,override=None):
 rp=p/(name+'.raw');mp=p/(name+'.toml');rp.write_bytes(rb);mp.write_bytes(mb);out=p/(name+'-output')
 args=['data-pack','materialize','jbotci',str(rp),str(mp),str(out),'--revision','0'*40,'--raw-sha256',override or digest(rb),'--metadata-sha256',digest(mb)]
 value=run(name,args,code)
 if code:assert not out.exists(),name
 return value
v=material('valid-full',raw,meta(raw));assert v['payload_kind']=='full';assert v['report']['input_rows']==2
material('bad-raw-pin',raw,meta(raw),'jbotci_hash_mismatch','0'*64)
material('bad-json',b'[',meta(b'['),'jbotci_json')
material('bad-metadata',raw,b'broken = [','jbotci_metadata')
material('accounting-mismatch',raw,meta(raw,99),'jbotci_accounting')
run('missing-input',['data-pack','materialize','jbotci',str(p/'missing'),str(p/'missing2'),str(p/'missing-output')],'io');assert not (p/'missing-output').exists()
native=os.fsencode(p)+b'/native-\xff';q=run('native-path',['data-pack','fixture','jbotci',native]);assert q['payload_kind']=='synthetic';assert os.path.isfile(native+b'/seal.json')
for name,args in [('root-help',['--help']),('pack-help',['data-pack','--help']),('material-help',['data-pack','materialize','jbotci','--help']),('fixture-help',['data-pack','fixture','jbotci','--help'])]:run(name,args)
assert 'data-pack' in (p/'root-help.stdout').read_text();assert '--raw-sha256' in (p/'material-help.stdout').read_text()
receipt={'source':'04289ae65d3529c1c52141c1c9b207fee63a8f28','binary_sha256':digest(cli.read_bytes()),'environment_home_and_xdg_absent':True,'cases':rows,'passed':len(rows)};(r/'cli-controls01.json').write_text(json.dumps(receipt,indent=2));print(json.dumps({'status':'PASS','cases':len(rows),'binary_sha256':receipt['binary_sha256']}))
