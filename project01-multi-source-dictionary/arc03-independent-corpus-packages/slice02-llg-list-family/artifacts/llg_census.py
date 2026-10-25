from pathlib import Path
from collections import Counter
import json,hashlib
p=Path('/tmp/vlajmaji-p01-a03-cdc/readiness');allheads=set();eligible=set();out={};totalfields=0;contexts=0;forms=0
for name in ['gismu','cmavo','rafsi']:
 b=(p/('llg-current-'+name+'.txt')).read_bytes();lines=b.decode('ascii').splitlines();heads=[];fields=Counter();ctx=Counter();fm=0
 for ordinal,line in enumerate(lines,1):
  if name!='rafsi' and ordinal==1:fields['$line']+=1;continue
  if name=='gismu':
   spec=[('word',1,6),('assigned_rafsi_cvc',7,10),('assigned_rafsi_ccv',11,14),('assigned_rafsi_cvv',15,19),('keyword',20,40),('hint',41,61),('definition',62,158),('teaching_code',159,161),('frequency_1993',161,165),('notes',165,None)]
   assert len(line)>=158
   for field,start,end in spec:
    if len(line)>start:fields[field]+=1
   head=line[1:6].strip();eligible.add(head)
   for field,start,end in spec[1:4]:fm+=bool(line[start:end].strip())
   for field,start,end in [spec[4],spec[5],spec[9]]:ctx[field]+=bool(line[start:end].strip())
  elif name=='cmavo':
   assert len(line)>=62;head=line[:10].strip();eligible.add(head)
   for field in ['word','selmaho','keyword','definition']:fields[field]+=1
   ctx['keyword']+=bool(line[20:61].strip())
  else:
   bits=line.split(None,2);assert len(bits)==3;head=bits[1];fm+=1;ctx['gloss']+=1
   fields.update(['rafsi','word','gloss'])
  assert head;heads.append(head);allheads.add(head)
 out[name]={'bytes':len(b),'sha256':hashlib.sha256(b).hexdigest(),'physical_lines':len(lines),'body_rows':len(heads),'distinct_heads':len(set(heads)),'fields':dict(fields),'native_fields':sum(fields.values()),'contexts':dict(ctx),'context_total':sum(ctx.values()),'assigned_forms':fm,'identical_complete_rows':len(lines)-len(set(lines)),'minimum_line_bytes':min(map(len,lines)),'maximum_line_bytes':max(map(len,lines)),'all_terminal_crlf':b.count(b'\r\n')==len(lines),'header_claimed_count':int(lines[0].strip().split()[0]) if name!='rafsi' else None}
 totalfields+=sum(fields.values());contexts+=sum(ctx.values());forms+=fm
out['totals']={'physical_lines':sum(x['physical_lines'] for x in out.values()),'native_fields':totalfields,'contexts':contexts,'assigned_forms':forms,'distinct_all_heads':len(allheads),'distinct_eligible_heads':len(eligible),'definition_assertions':1436+1091,'excluded_assertions':1545+2,'entries_with_header_placeholders':len(allheads)+2}
(p/'llg-census01.json').write_text(json.dumps(out,indent=2)+'\n');print(json.dumps(out,indent=2))
