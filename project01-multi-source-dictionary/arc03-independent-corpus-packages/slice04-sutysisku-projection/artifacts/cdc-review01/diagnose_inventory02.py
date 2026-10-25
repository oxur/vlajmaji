from pathlib import Path
import json,hashlib
r=Path('/tmp/vlajmaji-p01-a03-cdc/s04');p=r/'public-runtime02.log';maps={}
with p.open() as f:
 for line in f:
  if line.startswith('  left: '):maps['actual']=json.loads(line[len('  left: '):])
  if line.startswith(' right: '):maps['expected']=json.loads(line[len(' right: '):])
a=maps['actual'];e=maps['expected'];diff={k:{'actual_distinct':len(a.get(k,{})),'actual_rows':sum(a.get(k,{}).values()),'expected_distinct':len(e.get(k,{})),'expected_rows':sum(e.get(k,{}).values())} for k in a.keys()|e.keys() if a.get(k,{})!=e.get(k,{})};only_actual=sorted(a.keys()-e.keys());out={'actual_categories':len(a),'expected_categories':len(e),'only_actual_categories':only_actual,'only_actual_are_empty':all(not a[k] for k in only_actual),'only_expected_categories':sorted(e.keys()-a.keys()),'nonempty_or_content_differences':diff,'actual_category_counts':{k:sum(v.values()) for k,v in a.items()},'all_categories_equal_when_explicit_empty_inventories_retained':not diff,'log_sha256':hashlib.sha256(p.read_bytes()).hexdigest()};assert out==json.loads((r/'public-runtime02-inventory-diagnosis.json').read_bytes());print(json.dumps(out,indent=2))
