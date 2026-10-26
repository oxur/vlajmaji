"""Parse declared same-call stages; fail closed on missing request/stage evidence."""
import re
from decimal import Decimal

def duration_ns(token):
 m=re.fullmatch(r'(\d+(?:\.\d+)?)(ns|µs|ms|s)',token);assert m,token
 return int(Decimal(m[1])*{'ns':1,'µs':1000,'ms':1000000,'s':1000000000}[m[2]])

def profiles(raw,measured):
 rows=[]
 for line in raw.splitlines():
  if not line.startswith(('stage=measure request=','stage=measure-reference request=')):continue
  fields=dict(re.findall(r'(\w+)=([^\s]+)',line));name=fields['request']
  if line.startswith('stage=measure request='):
   p=re.search(r'DictionaryExactProfile \{ total: ([^,]+), candidate_sql: ([^,]+), hydration: ([^,]+), lookup_order_paging: ([^ }]+) \}',line);assert p,line
   total,sql,hydration,residual=map(duration_ns,p.groups());assert total==sql+hydration+residual
   value=int(fields['value_us']);render=int(fields['serialize_us']);lookup=int(fields['warm_lookup_us']);hydrate_output=int(fields['hydration_value_serialization_us'])
   assert lookup==sql//1000+residual//1000
   assert hydrate_output==hydration//1000+value+render
   rows.append(dict(id=name,kind='exact',total_ns=total,sql_ns=sql,residual_ns=residual,hydration_ns=hydration,warm_lookup_us=lookup,hydration_value_serialization_us=hydrate_output,value_us=value,serialize_us=render,write_us=int(fields['write_us']),bytes=int(fields['bytes']),sha256=fields['sha256']))
  else:
   rows.append(dict(id=name,kind='reference',warm_lookup_us=int(fields['lookup_resolve_us']),value_us=int(fields['value_us']),serialize_us=int(fields['serialize_us']),write_us=int(fields['write_us']),bytes=int(fields['bytes']),sha256=fields['sha256']))
 assert [r['id']for r in rows]==measured,([r['id']for r in rows],measured)
 return rows
