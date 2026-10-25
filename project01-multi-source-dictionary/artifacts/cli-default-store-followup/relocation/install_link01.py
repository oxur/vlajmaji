from pathlib import Path
import os,json,hashlib,stat,subprocess,time
R=Path('/tmp/vlajmaji-cli-default-store-cdc');p=json.loads((R/'relocation-preflight01.json').read_text());old=Path(p['old_directory']);src=old/'dictionary.sqlite3'
x=os.environ.get('XDG_DATA_HOME');home=os.environ.get('HOME');base=Path(x) if x and Path(x).is_absolute() else Path(home)/'.local/share' if home and Path(home).is_absolute() else None
assert base is not None
new=base/'vlajmaji/stores/default';assert str(new)==p['destination'] and not os.path.lexists(new)
assert sorted(f.name for f in old.iterdir())==['dictionary.sqlite3'];s=src.lstat();assert stat.S_ISREG(s.st_mode) and (s.st_dev,s.st_ino,s.st_size)==(p['old_device'],p['old_inode'],p['db_bytes'])
def sha(path):
 h=hashlib.sha256()
 with path.open('rb') as f:
  for b in iter(lambda:f.read(1048576),b''):h.update(b)
 return h.hexdigest()
assert sha(src)==p['sha256']
q=subprocess.run(['/usr/sbin/lsof','-F','pcfn',str(src)],capture_output=True);assert q.returncode==1 and not q.stdout and not q.stderr
missing=[];a=new.parent
while not a.exists():
 assert not a.is_symlink();missing.append(a);a=a.parent
existing={'path':str(a),'mode':oct(stat.S_IMODE(a.stat().st_mode))};assert a.is_dir() and a.stat().st_dev==s.st_dev
created=[]
for a in reversed(missing):
 a.mkdir(mode=0o700);assert stat.S_IMODE(a.stat().st_mode)&0o077==0;created.append({'path':str(a),'mode':oct(stat.S_IMODE(a.stat().st_mode))})
# Exclusive leaf followed by exclusive link: neither call replaces an occupied path.
new.mkdir(mode=0o700);os.link(src,new/'dictionary.sqlite3',follow_symlinks=False)
t=(new/'dictionary.sqlite3').lstat();assert (t.st_dev,t.st_ino)==(s.st_dev,s.st_ino) and t.st_nlink==2
assert sha(new/'dictionary.sqlite3')==p['sha256'];assert oct(stat.S_IMODE(Path(existing['path']).stat().st_mode))==existing['mode']
r={'phase':'exclusive-link-created-original-retained-pending-validation','old_directory':str(old),'new_directory':str(new),'method':'same-filesystem exclusive hardlink, then validation before selected original removal','XDG_DATA_HOME':x,'HOME':home,'created_parent_directories':created,'existing_ancestor':existing,'new_leaf_mode':oct(stat.S_IMODE(new.stat().st_mode)),'sha256':p['sha256'],'db_bytes':t.st_size,'device':t.st_dev,'inode':t.st_ino,'nlink':t.st_nlink,'lsof_exit':q.returncode,'time':time.time()}
(R/'relocation-link01.json').write_text(json.dumps(r,indent=2)+'\n');print(json.dumps(r,indent=2))
