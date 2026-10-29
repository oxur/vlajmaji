from pathlib import Path
import os,stat,hashlib,json,sys
base=Path('/tmp/vlajmaji-p01-a04-arc-review02');tag=sys.argv[1];out=base/'receipts'/('inventory-'+tag+'.json');assert not out.exists();rows=[];stack=[base]
while stack:
 d=stack.pop()
 for entry in sorted(os.scandir(d),key=lambda e:os.fsencode(e.name)):
  p=Path(entry.path);s=entry.stat(follow_symlinks=False);r={'path':str(p.relative_to(base)),'bytes':s.st_size,'kind':None}
  if stat.S_ISDIR(s.st_mode):r['kind']='directory';stack.append(p);r['accounted_bytes']=0
  elif stat.S_ISLNK(s.st_mode):r['kind']='symlink';raw=os.fsencode(os.readlink(p));r['accounted_bytes']=len(raw);r['sha256_of_link_payload']=hashlib.sha256(raw).hexdigest()
  else:
   assert stat.S_ISREG(s.st_mode),(p,s.st_mode);r['kind']='file';raw=p.read_bytes();r['accounted_bytes']=len(raw);r['sha256']=hashlib.sha256(raw).hexdigest()
  rows.append(r)
total=sum(x['accounted_bytes'] for x in rows);assert total<=200*1024**2,total
result={'stage':tag,'scope':'review02 ALL extant owned paths, nofollow; shared target excluded; own inventory file excluded because written after observation','bytes':total,'file_count':sum(r['kind']=='file' for r in rows),'symlink_count':sum(r['kind']=='symlink' for r in rows),'directory_count':sum(r['kind']=='directory' for r in rows),'threshold':200*1024**2,'peak_or_cumulative_claim':False,'rows':rows}
with out.open('x') as f:json.dump(result,f,indent=2);f.write('\n')
print(json.dumps({k:v for k,v in result.items() if k!='rows'}))
