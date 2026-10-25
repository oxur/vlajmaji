from pathlib import Path
import os,json,subprocess,hashlib,time
r=Path('/tmp/vlajmaji-p01-a03-cdc/s02');p=r/'cli-controls01';p.mkdir();cli=r/'candidate-b36-cli';env={k:v for k,v in os.environ.items() if k not in ['HOME','XDG_DATA_HOME']};rows=[]
def run(name,args,code=None):
 t=time.monotonic();q=subprocess.run([str(cli)]+[str(a) for a in args],env=env,cwd=p,stdout=subprocess.PIPE,stderr=subprocess.PIPE);rows.append({'name':name,'argv':[str(a) for a in args],'exit':q.returncode,'elapsed':time.monotonic()-t,'stdout':q.stdout.decode(),'stderr':q.stderr.decode()});(r/'cli-controls01.json').write_text(json.dumps(rows,indent=2))
 if code is None:assert q.returncode==0,(name,q.stderr)
 else:assert q.returncode!=0 and json.loads(q.stderr)['error']['code']==code,(name,q.stderr)
 return json.loads(q.stdout) if code is None and '--help' not in args else None
x=run('fixture',['data-pack','fixture','llg',p/'fixture']);assert x['payload_kind']=='synthetic';rep=x['report'];assert (rep['occurrences'],rep['entries'],rep['lexical_assertions'],rep['excluded_assertions'],rep['assigned_forms'])==(5,4,2,3,3)
for kind in ['directory','file','symlink']:
 dest=p/('occupied-'+kind)
 if kind=='directory':dest.mkdir();sent=dest/'keep';sent.write_bytes(b'untouched')
 elif kind=='file':dest.write_bytes(b'untouched');sent=dest
 else:dest.symlink_to(p/'occupied-directory',target_is_directory=True);sent=p/'occupied-directory/keep'
 run('occupied-'+kind,['data-pack','fixture','llg',dest],'io');assert sent.read_bytes()==b'untouched'
g=bytearray(b' '*158)
for start,v in [(1,b'farvi'),(7,b'fav'),(20,b'growth'),(62,b'x1 develops into x2')]:g[start:start+len(v)]=v
inputs={'gismu':b'00099 Lojban synthetic\n'+g+b'\n','cmavo':b'00001 Lojban synthetic\n'+f'{".a":<10}{"A":<10}{"logical or":<41}sumti connective\n'.encode(),'rafsi':b'fav farvi growth\n','notice':b'Explicit synthetic notice\n'}
for n,b in inputs.items():(p/(n+'.txt')).write_bytes(b)
def material(name,expected=None,modify=None):
 values=dict(inputs)
 if modify:values.update(modify)
 args=['data-pack','materialize','llg'];pins=[]
 for n,b in values.items():
  f=p/(name+'-'+n+'.txt');f.write_bytes(b);args.append(f);pins += ['--'+n+'-sha256',hashlib.sha256(b).hexdigest()]
 dest=p/(name+'-output');args += [dest]+pins+['--base-locator','https://example.invalid/synthetic']
 result=run(name,args,expected)
 if expected:assert not dest.exists()
 return result
v=material('valid-custom-full');assert v['payload_kind']=='full';assert v['report']['occurrences']==5 and v['report']['assigned_forms']==2;assert v['report']['components'][0]['declared_count_matches'] is False
# Exact default pins reject the custom source before output creation.
args=['data-pack','materialize','llg']+[p/(n+'.txt') for n in inputs]+[p/'bad-pin-output'];run('bad-pin',args,'llg_hash_mismatch');assert not (p/'bad-pin-output').exists()
material('invalid-utf8','llg_utf8',{'notice':b'\xff'})
# Invalid options with otherwise correct byte pins must not create the output.
args=['data-pack','materialize','llg']+[p/(n+'.txt') for n in inputs]+[p/'bad-options-output']
for n,b in inputs.items():args+=['--'+n+'-sha256',hashlib.sha256(b).hexdigest()]
args+=['--base-locator',''];run('bad-options',args,'llg_options');assert not (p/'bad-options-output').exists()
run('missing-input',['data-pack','materialize','llg',p/'absent',p/'cmavo.txt',p/'rafsi.txt',p/'notice.txt',p/'absent-output'],'io');assert not (p/'absent-output').exists()
for name,args in [('material-help',['data-pack','materialize','llg','--help']),('fixture-help',['data-pack','fixture','llg','--help'])]:run(name,args)
assert '--notice-sha256' in rows[-2]['stdout'];assert 'OUTPUT' in rows[-1]['stdout']
summary={'status':'PASS','contract_cases':len(rows),'source':'b36adadb9aba21363957cd306001ab95a7d540bd','profile':'debug','binary_sha256':hashlib.sha256(cli.read_bytes()).hexdigest(),'home_xdg_absent':True};(r/'cli-controls01-summary.json').write_text(json.dumps(summary,indent=2));print(json.dumps(summary))
