"""Check the pinned canonical candidate changes only the four prescribed notice homes."""
import hashlib,json,sys
from pathlib import Path
oldp,newp=map(Path,sys.argv[1:3]);old=json.loads(oldp.read_bytes());new=json.loads(newp.read_bytes())
changes=[]
def walk(a,b,p=()):
 if type(a) is not type(b):changes.append(p);return
 if isinstance(a,dict):
  assert a.keys()==b.keys(),('changed keys',p)
  for k in a:walk(a[k],b[k],p+(k,))
 elif isinstance(a,list):
  assert len(a)==len(b),('changed list size',p)
  for i,(x,y) in enumerate(zip(a,b)):walk(x,y,p+(i,))
 elif a!=b:changes.append(p)
walk(old,new)
expected={('registry','lineage',0,'evidence'),('captures',0,'metadata','notice_locator','Value'),('metadata','code_notices',0,'locator','Value'),('metadata','releases',0,'content_notices',0,'locator','Value')}
assert set(changes)==expected,changes
print(json.dumps({'verdict':'PASS','old_sha256':hashlib.sha256(oldp.read_bytes()).hexdigest(),'new_sha256':hashlib.sha256(newp.read_bytes()).hexdigest(),'exact_changed_paths':[list(x) for x in changes],'other_fields':'all equal, including captures bytes/native fields/IDs/accounting/projections'}))
