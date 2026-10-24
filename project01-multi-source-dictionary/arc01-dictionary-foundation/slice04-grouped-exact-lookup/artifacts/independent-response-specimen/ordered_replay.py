"""Source-record association ordering and duplicate membership are independent of hit order."""
import json,pathlib,subprocess,sys,collections
import response_oracle as o
from ordered_variant import variant
DOC,ROOT,CLI,PUBLIC=sys.argv[1:];root=pathlib.Path(ROOT);root.mkdir();d=variant(json.loads(pathlib.Path(DOC).read_bytes()));p=root/'input.json';p.write_text(json.dumps(d,ensure_ascii=False,separators=(',',':')));store=root/'store';records=[]
r=subprocess.run([PUBLIC,'create',str(p),str(store)],capture_output=True,text=True);assert r.returncode==0 and 'counts' in json.loads(r.stdout),(r.returncode,r.stdout,r.stderr)
for i in ['public','cli']:
 for q in ['farvi','cpana']:
  try:
   req=json.dumps(dict(operation='exact',spelling=q));r=subprocess.run([PUBLIC,'request',str(store),req] if i=='public' else [CLI,'dictionary-query',str(store),'--request',req],capture_output=True,text=True);assert r.returncode==0,(r.returncode,r.stdout,r.stderr);v=json.loads(r.stdout);diff=o.diffs([h for g in v['groups'] for h in g['hits']],[o.hit(d,a) for a in o.pick(d,q)]);assert not diff,'; '.join(diff[:10]);records.append(dict(name=i+':ordered:'+q,status='PASS'))
  except Exception as e:records.append(dict(name=i+':ordered:'+q,status='FAIL',detail=str(e)))
 try:
  req=json.dumps(dict(operation='resolve',target={'Local':{'EditorialSense':'editorial-farvi'}}));r=subprocess.run([PUBLIC,'request',str(store),req] if i=='public' else [CLI,'dictionary-query',str(store),'--request',req],capture_output=True,text=True);assert r.returncode==0;(v:=json.loads(r.stdout)).pop('state_sha256');diff=o.diffs(v,o.resolved(d,{'Local':{'EditorialSense':'editorial-farvi'}}));assert not diff,'; '.join(diff[:10]);records.append(dict(name=i+':duplicate-editorial-resolve',status='PASS'))
 except Exception as e:records.append(dict(name=i+':duplicate-editorial-resolve',status='FAIL',detail=str(e)))
(root/'results.json').write_text(json.dumps(dict(records=records,summary=dict(collections.Counter(r['status'] for r in records))),indent=2)+'\n');print(json.dumps(dict(collections.Counter(r['status'] for r in records))));sys.exit(any(r['status']!='PASS' for r in records))
