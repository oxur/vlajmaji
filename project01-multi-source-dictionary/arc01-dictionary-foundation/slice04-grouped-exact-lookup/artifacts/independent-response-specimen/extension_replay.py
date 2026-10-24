"""Check selected sibling-release extension ownership without manufacturing membership."""
import json,pathlib,subprocess,sys,collections
import response_oracle as o
from extension_variant import variant
DOC,ROOT,CLI,PUBLIC=sys.argv[1:];root=pathlib.Path(ROOT);root.mkdir();d=variant(json.loads(pathlib.Path(DOC).read_bytes()));p=root/'input.json';p.write_text(json.dumps(d,ensure_ascii=False,separators=(',',':')));store=root/'store';records=[]
r=subprocess.run([PUBLIC,'create',str(p),str(store)],capture_output=True,text=True)
if r.returncode or not json.loads(r.stdout).get('counts'):raise AssertionError((r.returncode,r.stdout,r.stderr))
for interface in ['public','cli']:
 try:
  req=json.dumps(dict(operation='exact',spelling='farvi'));r=subprocess.run([PUBLIC,'request',str(store),req] if interface=='public' else [CLI,'dictionary-query',str(store),'--request',req],capture_output=True,text=True);assert r.returncode==0,(r.returncode,r.stdout,r.stderr);v=json.loads(r.stdout)
  actual=[h for g in v['groups'] for h in g['hits']];expected=[o.hit(d,a) for a in o.pick(d,'farvi')];diff=o.diffs(actual,expected);assert not diff,'; '.join(diff[:10]);eq=o.diffs(v['availability'],o.availability(d));assert not eq,'; '.join(eq[:10])
  for h in actual:
   if h['entry']['source']=='source-a':
    names={x['name'] for x in h['extensions']};assert 'sibling-extension-true' in names and 'sibling-extension-false' not in names,names;assert 'release-a-empty-true' not in h['release_ids']
  records.append(dict(name=interface+':selected-sibling-extension',status='PASS'))
 except Exception as e:records.append(dict(name=interface+':selected-sibling-extension',status='FAIL',detail=str(e)))
(root/'results.json').write_text(json.dumps(dict(records=records,summary=dict(collections.Counter(r['status'] for r in records))),indent=2)+'\n');print(json.dumps(dict(collections.Counter(r['status'] for r in records))));sys.exit(any(r['status']!='PASS' for r in records))
