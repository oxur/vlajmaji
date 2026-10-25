from pathlib import Path
import os,json,subprocess,hashlib,time
r=Path('/tmp/vlajmaji-p01-a03-cdc/s04');p=r/'cli-controls02';p.mkdir();cli=r/'candidate-cli';env={k:v for k,v in os.environ.items() if k not in ['HOME','XDG_DATA_HOME']};rows=[];census=json.loads((r/'cli-fixture-source-census01.json').read_bytes())
def run(name,args,code=None):
 args=list(map(str,args));t=time.monotonic();q=subprocess.run([str(cli)]+args,cwd=p,env=env,capture_output=True);item={'name':name,'argv':args,'exit':q.returncode,'elapsed':time.monotonic()-t,'stdout':q.stdout.decode(),'stderr':q.stderr.decode()};rows.append(item);(r/'cli-controls02.json').write_text(json.dumps(rows,indent=2)+'\n')
 if code is None:assert q.returncode==0,item
 else:assert q.returncode!=0 and json.loads(q.stderr)['error']['code']==code,item
 return json.loads(q.stdout) if q.returncode==0 and '--help' not in args else None
x=run('synthetic',['data-pack','fixture','sutysisku',p/'synthetic']);v=x['report'];assert x['payload_kind']=='synthetic'
for key,source in [('input_rows','root_keys'),('root_keys','root_keys'),('native_child_fields','native_child_fields'),('native_fields','native_fields_with_keys'),('lexical_occurrences','lexical_occurrences'),('excluded_occurrences','excluded_occurrences'),('eligible_spellings','eligible_spellings'),('all_spellings','all_spellings'),('contexts','contexts'),('reported_forms','forms'),('extensions','extensions')]:assert v[key]==census[source],(key,v,census)
assert v['field_counts']==dict(census['field_counts'],**{'$key':census['root_keys']});assert v['word_types']==census['word_types'];m=json.loads((p/'synthetic/release.json').read_bytes());assert m['metadata']['contexts'][0]['value']=='Null';assert m['metadata']['releases'][0]['redistribution']=='Held'
for kind in ['directory','file','symlink']:
 dest=p/('occupied-'+kind)
 if kind=='directory':dest.mkdir();sent=dest/'keep';sent.write_bytes(b'untouched')
 elif kind=='file':dest.write_bytes(b'untouched');sent=dest
 else:dest.symlink_to(p/'occupied-directory',target_is_directory=True);sent=p/'occupied-directory/keep'
 run('occupied-'+kind,['data-pack','fixture','sutysisku',dest],'io');assert sent.read_bytes()==b'untouched'
def material(name,data,code=None,extra=(),dest=None):
 src=p/(name+'.json');src.write_bytes(data);out=dest if dest is not None else p/(name+'-output');args=['data-pack','materialize','sutysisku',src,out,'--raw-sha256',hashlib.sha256(data).hexdigest(),'--revision','1'*40]+list(extra);x=run(name,args,code)
 if code:assert not out.exists() and not out.is_symlink()
 return x,out
raw=(r/'public-fixture.json').read_bytes();bad_native=p/os.fsdecode(b'native-\xff-output');material('invalid-native-byte-path',raw,'io',dest=bad_native);assert 'os error 92' in json.loads(rows[-1]['stderr'])['error']['message'];native=p/'native-�-output';x,out=material('valid-full-native-path',raw,dest=native);assert os.fsencode(out.name)==b'native-\xef\xbf\xbd-output';v=x['report'];f=json.loads((r/'public-fixture-census01.json').read_bytes());assert (v['input_rows'],v['native_fields'],v['lexical_occurrences'],v['excluded_occurrences'],v['contexts'],v['reported_forms'],v['extensions'])==(f['root_members'],f['model_native_fields'],f['declared_policy_populations']['lexical'],f['declared_policy_populations']['excluded'],f['declared_policy_populations']['contexts'],f['declared_policy_populations']['forms'],f['declared_policy_populations']['extensions'])
b=(out/'release.json').read_bytes();model=json.loads(b);seal=json.loads((out/'seal.json').read_bytes());assert x['payload_kind']=='full' and seal['payload_kind']=='full';assert bytes(model['captures'][0]['bytes'])==raw;assert model['captures'][0]['occurrences'][0]['fields'][0]['raw_value']==r'"\u0066arvi"';assert x['release_sha256']==seal['release_sha256']==hashlib.sha256(b).hexdigest();assert model['captures'][0]['metadata']['locator'].endswith('/'+'1'*40+'/data/parsed-en.json');assert model['metadata']['releases'][0]['redistribution']=='Held';assert seal['pack_id']=='sutysisku'
run('wrong-pin',['data-pack','materialize','sutysisku',p/'valid-full-native-path.json',p/'wrong-pin-output'],'sutysisku_hash_mismatch');assert not (p/'wrong-pin-output').exists()
material('invalid-json',b'[]','sutysisku_json');material('invalid-utf8',b'\xff','sutysisku_utf8')
run('invalid-revision',['data-pack','materialize','sutysisku',p/'valid-full-native-path.json',p/'invalid-revision-output','--raw-sha256',hashlib.sha256(raw).hexdigest(),'--revision','bad'],'sutysisku_options');assert not (p/'invalid-revision-output').exists()
material('invalid-locator',raw,'sutysisku_options',['--notice-locator',''])
run('missing-input',['data-pack','materialize','sutysisku',p/'missing',p/'missing-output'],'io');assert not (p/'missing-output').exists()
for name,args in [('material-help',['data-pack','materialize','sutysisku','--help']),('fixture-help',['data-pack','fixture','sutysisku','--help'])]:run(name,args)
assert all(v in rows[-2]['stdout'] for v in ['--revision','--raw-sha256','--raw-locator','--notice-locator'])
summary={'status':'PASS','cases':len(rows),'source':'7b1c565ebf6c8a423e04211b599006b8c21d65d9','profile':'debug','binary_sha256':hashlib.sha256(cli.read_bytes()).hexdigest(),'home_xdg_absent':True,'native_non_utf8_io92_matches_host':True,'valid_utf8_replacement_path_created_exactly':True,'fixture_census_precedes_assertions_and_run':True};(r/'cli-controls02-summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps(summary))
