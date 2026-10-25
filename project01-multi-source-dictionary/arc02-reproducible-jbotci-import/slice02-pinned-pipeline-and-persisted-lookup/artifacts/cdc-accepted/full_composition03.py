import hashlib,json,subprocess,sys
from pathlib import Path
r=Path('/tmp/vlajmaji-p01-a02-cdc');target=Path('/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc01/target');new=target/'release/a02_cdc_composition_20261024c';old=r/'retained-baseline-binary/a02_cdc_composition_20261024b';canonical=r/'s02-full-canonical02.json';store=r/'s02-full-store03';leaves=list((r/'s02-full-cache02').iterdir());assert len(leaves)==1;raw=leaves[0]/'dictionary-en.json';meta=leaves[0]/'dictionary-en.metadata.toml'
def invoke(label,argv):
 p=subprocess.run([sys.executable,'-B',str(r/'run_process.py'),label,*map(str,argv)]);assert p.returncode==0,(label,p.returncode)
invoke('s02-full-create03',[new,'create',store,canonical])
invoke('s02-full-sql03',[sys.executable,'-B',r/'verify_sqlite.py',raw,meta,store/'dictionary.sqlite3'])
invoke('s02-old-finite8-03',[old,'finite8',store,canonical,r/'s02-old-finite8-03-results'])
invoke('s02-new-finite8-03',[new,'finite8',store,canonical,r/'s02-new-finite8-03-results'])
oldout=r/'s02-old-finite8-03-results';newout=r/'s02-new-finite8-03-results';a={p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in oldout.glob('*.json') if p.name!='timing.json'};b={p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in newout.glob('*.json') if p.name!='timing.json'};assert len(a)==8 and a==b;comparison={'verdict':'PASS','cases':8,'complete_json_byte_hashes':a,'same_store':str(store),'old_binary_sha256':hashlib.sha256(old.read_bytes()).hexdigest(),'new_binary_sha256':hashlib.sha256(new.read_bytes()).hexdigest(),'old_then_new':True,'profile':'release,Rust1.98.0,default_features_false,samecallerSource05f1a0e,lockpackageonlyrename','timing_limit':'One local observation, old-before-new cache/order/load confounds; no universal ratio/constant-time/heap-byte claim.'};(r/'s02-complete-old-new-comparison03.json').write_text(json.dumps(comparison,indent=2)+'\n')
invoke('s02-verify-finite8-03',[sys.executable,'-B',r/'verify_finite.py',raw,newout])
invoke('s02-new-finite23-03',[new,'finite',store,canonical,r/'s02-new-finite23-03-results'])
invoke('s02-verify-finite23-03',[sys.executable,'-B',r/'verify_finite.py',raw,r/'s02-new-finite23-03-results'])
invoke('s02-new-fixture12-03',[new,'fixture',r/'s02-fixture-candidate03'])
a=json.loads((r/'s02-fixture-baseline/responses.json').read_bytes());b=json.loads((r/'s02-fixture-candidate03/responses.json').read_bytes());assert len(a)==12 and a==b;(r/'s02-fixture-comparison03.json').write_text(json.dumps({'verdict':'PASS','cases':12,'complete_value_equality':True,'new_sha256':hashlib.sha256((r/'s02-fixture-candidate03/responses.json').read_bytes()).hexdigest()},indent=2)+'\n')
print('PASS full create/nativeSQL/old-new8/fullfinite23/fixture12 composition')
