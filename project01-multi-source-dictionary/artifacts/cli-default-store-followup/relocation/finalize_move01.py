from pathlib import Path
import json,hashlib,os,sqlite3,subprocess,stat,time
R=Path('/tmp/vlajmaji-cli-default-store-cdc');p=json.loads((R/'relocation-preflight01.json').read_text());link=json.loads((R/'relocation-link01.json').read_text());old=Path(p['old_directory']);src=old/'dictionary.sqlite3';new=Path(p['destination']);dst=new/'dictionary.sqlite3'
def sha(path):
 h=hashlib.sha256()
 with path.open('rb') as f:
  for b in iter(lambda:f.read(1048576),b''):h.update(b)
 return h.hexdigest()
for label in ['relocation-old-info','relocation-default-info','relocation-default-farvi']:
 record=json.loads((R/(label+'.json')).read_text());assert record['exit']==0 and record['source_head']==p['source'] and record['source_head_after']==p['source'];assert not (R/(label+'.stderr')).read_bytes()
assert (R/'relocation-old-info.stdout').read_bytes()==(R/'relocation-default-info.stdout').read_bytes()
i=json.loads((R/'relocation-default-info.stdout').read_bytes());assert (i['captures'],i['occurrences'],i['entries'],i['assertions'],i['source_count'])==(2,33526,33526,33526,1) and i['sources'][0]['id']=='lensisku'
response=R/'relocation-default-farvi.stdout';assert response.stat().st_size==3028230;response_sha=sha(response);assert response_sha=='4a0c902b350162d7932169428713b53970d8aaba235c94da3cff313aa981054e'
q=json.loads(response.read_bytes());assert q['totals']['occurrences']==2
assert sorted(f.name for f in old.iterdir())==['dictionary.sqlite3'] and sorted(f.name for f in new.iterdir())==['dictionary.sqlite3']
s=src.lstat();t=dst.lstat();assert stat.S_ISREG(s.st_mode) and stat.S_ISREG(t.st_mode) and (s.st_dev,s.st_ino,s.st_size)==(t.st_dev,t.st_ino,t.st_size)==(p['old_device'],p['old_inode'],p['db_bytes']) and s.st_nlink==2
for path in [src,dst]:
 proc=subprocess.run(['/usr/sbin/lsof','-F','pcfn',str(path)],capture_output=True);assert proc.returncode==1 and not proc.stdout and not proc.stderr;assert sha(path)==p['sha256']
c=sqlite3.connect(dst.as_uri()+'?mode=ro',uri=True);meta=dict(c.execute('SELECT key,value FROM metadata'));canonical=hashlib.sha256(c.execute('SELECT json FROM canonical_document WHERE id=1').fetchone()[0]).hexdigest();c.close();assert meta==p['metadata'] and canonical==p['canonical_sha256']
# No query/store process remains. Remove only the identity-rechecked selected original.
s=src.lstat();assert (s.st_dev,s.st_ino,s.st_size)==(p['old_device'],p['old_inode'],p['db_bytes']);src.unlink();old.rmdir()
assert not os.path.lexists(src) and not os.path.lexists(old) and dst.is_file();after=dst.stat();assert after.st_nlink==1 and sha(dst)==p['sha256']
planning=Path('/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/planning');a='project01-multi-source-dictionary/arc02-reproducible-jbotci-import';tree=subprocess.check_output(['git','rev-parse','HEAD:'+a],cwd=planning,text=True).strip();assert tree==(R/'arc02-tree-before.txt').read_text().strip();assert not subprocess.check_output(['git','diff','--name-only','--',a],cwd=planning,text=True)
res={'verdict':'PASS','source':p['source'],'binary_sha256':'68250cb649464eecd299efef2eb70d23b6827e5d8c40355eebdd1a4ce8bb5675','profile':'release','old_directory':str(old),'new_directory':str(new),'old_file_absent':not os.path.lexists(src),'old_directory_absent':not os.path.lexists(old),'new_file_exists':dst.is_file(),'new_file_set':['dictionary.sqlite3'],'method':link['method'],'db_bytes':after.st_size,'db_sha256_before_after':p['sha256'],'canonical_sha256_before_after':canonical,'schema_version':meta['schema_version'],'complete':meta['complete'],'device':after.st_dev,'inode':after.st_ino,'nlink_final':after.st_nlink,'default_info':i,'whole_farvi_response_bytes':response.stat().st_size,'whole_farvi_response_sha256':response_sha,'farvi_occurrences':q['totals']['occurrences'],'old_explicit_vs_new_noarg_info_bytes_equal':True,'read_only_lsof_no_open_handles':True,'protected_arc02_tree_before_after':tree,'original_manifests_unchanged':True,'created_parent_directories':link['created_parent_directories'],'existing_ancestor':link['existing_ancestor'],'final_time':time.time(),'limits':'Single-source Lensisku provisional default; future aggregate not built. Other accepted stores/raw/releases retained. Full response stays local; receipt carries exact hash.'}
(R/'relocation-final01.json').write_text(json.dumps(res,indent=2)+'\n');print(json.dumps(res,indent=2))
