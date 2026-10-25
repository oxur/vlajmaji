from pathlib import Path
import os,json,subprocess,hashlib,time
r=Path('/tmp/vlajmaji-p01-a03-cdc/s03');p=r/'cli-controls01';p.mkdir();cli=r/'candidate-cli';env={k:v for k,v in os.environ.items() if k not in ['HOME','XDG_DATA_HOME']};rows=[]
def run(name,args,code=None):
 t=time.monotonic();q=subprocess.run([str(cli)]+list(map(str,args)),cwd=p,env=env,capture_output=True);item={'name':name,'argv':list(map(str,args)),'exit':q.returncode,'elapsed':time.monotonic()-t,'stdout':q.stdout.decode(),'stderr':q.stderr.decode()};rows.append(item);(r/'cli-controls01.json').write_text(json.dumps(rows,indent=2));
 if code is None:assert q.returncode==0,item
 else:assert q.returncode!=0 and json.loads(q.stderr)['error']['code']==code,item
 return json.loads(q.stdout) if q.returncode==0 and '--help' not in args else None
x=run('synthetic',['data-pack','fixture','la-lojban',p/'synthetic']);v=x['report'];assert x['payload_kind']=='synthetic';assert (v['input_rows'],v['native_fields'],v['normalized_assertions'],v['lexical_occurrences'],v['excluded_occurrences'],v['eligible_spellings'],v['all_spellings'],v['contributors'])==(3,12,3,2,1,1,1,0)
for kind in ['directory','file','symlink']:
 dest=p/('occupied-'+kind)
 if kind=='directory':dest.mkdir();sent=dest/'keep';sent.write_bytes(b'untouched')
 elif kind=='file':dest.write_bytes(b'untouched');sent=dest
 else:dest.symlink_to(p/'occupied-directory',target_is_directory=True);sent=p/'occupied-directory/keep'
 run('occupied-'+kind,['data-pack','fixture','la-lojban',dest],'io');assert sent.read_bytes()==b'untouched'
raw=b'[{"word":"farvi","word_type":"gismu","definition":"explicit synthetic definition","definition_id":1e9999,"score":1.2500,"rafsi":["fav","farv"]}]';f=p/'custom.json';f.write_bytes(raw)
def material(name,data,code=None,extra=()):
 src=p/(name+'.json');src.write_bytes(data);out=p/(name+'-output');args=['data-pack','materialize','la-lojban',src,out,'--raw-sha256',hashlib.sha256(data).hexdigest(),'--revision','1'*40]+list(extra);x=run(name,args,code)
 if code:assert not out.exists()
 return x,out
x,out=material('valid-full',raw);v=x['report'];assert x['payload_kind']=='full' and v['input_rows']==1 and v['native_fields']==6 and v['lexical_occurrences']==1
m=json.loads((out/'release.json').read_text());assert m['registry']['occurrences'][0]['native_assertion_id']=='1e9999';assert len(m['metadata']['forms'])==2;assert m['metadata']['assertions'][0]['source_score']=={'Value':'1.2500'};assert m['captures'][0]['metadata']['locator'].endswith('/'+'1'*40+'/assets/dumps/en.json');assert m['metadata']['releases'][0]['redistribution']=='Held';assert len(m['captures'])==1 and not m['metadata']['extensions'];seal=json.loads((out/'seal.json').read_text());assert seal['payload_kind']=='full' and seal['release_sha256']==hashlib.sha256((out/'release.json').read_bytes()).hexdigest()
run('wrong-pin',['data-pack','materialize','la-lojban',f,p/'wrong-pin-output'],'la_lojban_hash_mismatch');assert not (p/'wrong-pin-output').exists()
material('invalid-json',b'{}','la_lojban_json');material('invalid-utf8',b'\xff','la_lojban_utf8')
run('invalid-revision',['data-pack','materialize','la-lojban',f,p/'invalid-revision-output','--raw-sha256',hashlib.sha256(raw).hexdigest(),'--revision','bad'],'la_lojban_options');assert not (p/'invalid-revision-output').exists()
material('invalid-locator',raw,'la_lojban_options',['--notice-locator',''])
run('missing-input',['data-pack','materialize','la-lojban',p/'missing',p/'missing-output'],'io');assert not (p/'missing-output').exists()
for name,args in [('material-help',['data-pack','materialize','la-lojban','--help']),('fixture-help',['data-pack','fixture','la-lojban','--help'])]:run(name,args)
assert all(v in rows[-2]['stdout'] for v in ['--revision','--raw-sha256','--raw-locator','--notice-locator'])
summary={'status':'PASS','cases':len(rows),'source':'62b27db8b58f66d351550445d7312cfe10385a9c','profile':'debug','binary_sha256':hashlib.sha256(cli.read_bytes()).hexdigest(),'home_xdg_absent':True};(r/'cli-controls01-summary.json').write_text(json.dumps(summary,indent=2));print(json.dumps(summary))
