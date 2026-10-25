from pathlib import Path
from collections import Counter
import json,hashlib
R=Path('/tmp/vlajmaji-p01-a03-cdc/s04');raw=(R/'public-fixture.json').read_bytes()
class Pairs(list):pass
d=json.JSONDecoder(object_pairs_hook=Pairs)
root=d.decode(raw.decode());assert isinstance(root,Pairs)
rows=[];fields=Counter();semantic=Counter();rootcounts=Counter(k for k,v in root)
def duplicate(v):
 if isinstance(v,Pairs):return len({k for k,x in v})!=len(v) or any(duplicate(x) for k,x in v)
 if isinstance(v,list):return any(duplicate(x) for x in v)
 return False
for i,(key,row) in enumerate(root):
 fs=[k for k,v in row] if isinstance(row,Pairs) else []
 fields.update(fs);ambiguous=duplicate(row);valid=isinstance(row,Pairs) and not ambiguous
 values=dict(row) if valid else {}
 eligible=valid and bool(key) and values.get('t') in {'gismu','cmevla'} and bool(values.get('d','').strip())
 semantic['lexical' if eligible else 'excluded']+=1
 if valid:
  semantic['contexts']+=len(values.get('g',[]))+int(isinstance(values.get('n'),str))
  semantic['forms']+=len(values.get('r',[]))
 semantic['extensions']+=sum(k in {'g','b'} or k not in {'d','t','s','n','r'} for k in fs)
 rows.append({'ordinal':i,'decoded_key':key,'child_fields':fs,'child_field_count':len(fs),'duplicate_inside_row':ambiguous,'eligible':eligible,'nonobject':not isinstance(row,Pairs)})
nonobjects=sum(x['nonobject'] for x in rows)
out={'kind':'independent raw-literal census BEFORE numeric tuples and first product execution','fixture_sha256':hashlib.sha256(raw).hexdigest(),'bytes':len(raw),'root_members':len(root),'native_child_fields':sum(fields.values()),'raw_key_carriers':len(root),'raw_nonobject_carriers':nonobjects,'model_native_fields':len(root)+sum(fields.values())+nonobjects,'field_counts':dict(fields),'duplicate_decoded_root_groups':{k:v for k,v in rootcounts.items() if v>1},'declared_policy_populations':dict(semantic),'rows':rows}
(R/'public-fixture-census01.json').write_text(json.dumps(out,indent=2)+'\n');print(json.dumps(out,indent=2))
