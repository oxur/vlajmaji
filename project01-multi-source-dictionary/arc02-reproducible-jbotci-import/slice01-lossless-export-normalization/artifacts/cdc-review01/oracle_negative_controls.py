#!/usr/bin/env python3
"""Prove the CDC full-population oracle rejects specific source-fidelity losses."""
import copy,hashlib,json,subprocess,sys
from pathlib import Path

def main():
 raw,metadata,release,report=sys.argv[1:5]
 base=Path(__file__).parent;oracle=base/'verify_normalized.py'
 root=base/'oracle-discrimination';root.mkdir(exist_ok=False)
 args=[sys.executable,'-B',str(oracle),raw,metadata]
 good=subprocess.run(args+[release,report],capture_output=True)
 (root/'positive.stdout').write_bytes(good.stdout);(root/'positive.stderr').write_bytes(good.stderr)
 assert good.returncode==0,good.stderr.decode(errors='replace')
 original=json.loads(Path(release).read_bytes());results=[]
 for name,category in [('numeric-score-equivalent','assertion'),('native-lexeme-equivalent','native_fields'),('wrong-source-form-role','form_value'),('gloss-null-to-missing','gloss_native_meaning')]:
  doc=copy.deepcopy(original)
  if name=='numeric-score-equivalent':
   target=next(a for a in doc['metadata']['assertions'] if a['occurrence']['locator']=='/18910')
   assert target['source_score']=={'Value':'1.0'}
   target['source_score']={'Value':'1.00'}
  elif name=='native-lexeme-equivalent':
   cap=next(c for c in doc['captures'] if c['identity']['namespace']=='captured:lensisku:jbotci:en')
   f=next(f for f in cap['occurrences'][0]['fields'] if f['name']=='score')
   old=f['raw_value'];assert '.' in old and 'e' not in old.lower();f['raw_value']=old+'0'
   assert json.loads(old)==json.loads(f['raw_value'])
  elif name=='wrong-source-form-role':
   assert doc['metadata']['forms'][0]['kind']=='SourceReportedForm'
   doc['metadata']['forms'][0]['kind']='AssignedRafsi'
  else:
   gloss={c['id'] for c in doc['metadata']['contexts'] if c['kind']=='Gloss'}
   found=False
   for x in doc['metadata']['extensions']:
    if x['owner'] not in gloss:continue
    v=json.loads(x['value'])
    if isinstance(v,dict) and 'meaning' in v and v['meaning'] is None:
     del v['meaning'];x['value']=json.dumps(v,separators=(',',':'));found=True;break
   assert found
  path=root/(name+'.json');path.write_text(json.dumps(doc,ensure_ascii=False,separators=(',',':')))
  p=subprocess.run(args+[str(path),report],capture_output=True)
  (root/(name+'.stdout')).write_bytes(p.stdout);(root/(name+'.stderr')).write_bytes(p.stderr)
  message=p.stderr.decode(errors='replace')
  assert p.returncode!=0 and 'AssertionError: '+category+':' in message,(name,p.returncode,message)
  results.append({'name':name,'expected_category':category,'exit':p.returncode,'counterexample_sha256':hashlib.sha256(path.read_bytes()).hexdigest()})
  # Each file is an owned derived counterexample; retain hash/log, then reclaim it.
  path.unlink()
 out={'verdict':'PASS','positive_exit':good.returncode,'counterexamples':results,'oracle_sha256':hashlib.sha256(oracle.read_bytes()).hexdigest()}
 (root/'results.json').write_text(json.dumps(out,indent=2)+'\n');print(json.dumps(out,indent=2))
if __name__=='__main__':main()
