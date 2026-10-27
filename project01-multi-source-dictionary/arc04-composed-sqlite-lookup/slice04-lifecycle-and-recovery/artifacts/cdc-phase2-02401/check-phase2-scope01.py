"""Read-only frozen-boundary checker. DO NOT RUN while CC owns a mutable source candidate."""
import hashlib,json,os,subprocess,sys
from pathlib import Path
SOURCE=Path('/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc04/vlajmaji')
BASE='8b5df7b3ebc06d14b880af59f223355736706260'
ALLOWED='''crates/store/src/collections.rs
crates/store/src/collections/lifecycle.rs
crates/store/src/collections/maintenance.rs
crates/store/src/collections_schema.rs
crates/store/src/collection_profiles.rs
crates/store/src/collection_profiles/policy.rs
crates/store/src/lib.rs
crates/store/tests/collections_maintenance_contract.rs
crates/store/tests/collection_policy_contract.rs
crates/search/src/collection_query.rs
crates/search/tests/collection_lifecycle_contract.rs
crates/cli/src/main.rs
crates/cli/tests/collections_maintenance_contract.rs
crates/store/README.md
crates/cli/README.md
docs/collections.md'''.splitlines()
ENV={**os.environ,'GIT_OPTIONAL_LOCKS':'0'}
def git(*args,root=SOURCE):return subprocess.check_output(['git','--no-optional-locks','-C',str(root),*args],env=ENV)
def digest(b):return hashlib.sha256(b).hexdigest()
def check(expected):
 head=git('rev-parse','HEAD').decode().strip();assert head==expected
 assert git('branch','--show-current').decode().strip()=='codex/project01-composed-sqlite-lookup'
 assert not git('status','--porcelain=v1')
 subprocess.run(['git','--no-optional-locks','-C',str(SOURCE),'merge-base','--is-ancestor',BASE,head],env=ENV,check=True)
 names=git('diff','--name-only',BASE,head).decode().splitlines();assert set(names)<=set(ALLOWED) and len(names)==len(set(names))
 assert not git('diff','--name-only','--diff-filter=D',BASE,head)
 subprocess.run(['git','--no-optional-locks','-C',str(SOURCE),'diff','--check',BASE,head],env=ENV,check=True)
 records=[]
 for name in names:
  disk=(SOURCE/name).read_bytes();blob=git('show',head+':'+name);assert disk==blob
  records.append({'path':name,'bytes':len(blob),'sha256':digest(blob),'git_blob':git('rev-parse',head+':'+name).decode().strip()})
 lock=(SOURCE/'Cargo.lock').read_bytes();assert lock==git('show',BASE+':Cargo.lock') and digest(lock)=='65cd7f39a88bdd2a3dab1fcd14e9055ff7b77cb22ec3af8c309ce49dbea78700'
 guard=Path('/tmp/vlajmaji-p01-a04-cargo').read_bytes();assert digest(guard)=='20fe7ba73311aa503870923af09f33f3b53262742ca4eeea2972092924cf5de8'
 body=git('show','-s','--format=%B',head)
 trailers=subprocess.check_output(['git','interpret-trailers','--parse'],input=body,env=ENV).decode().splitlines()
 assert trailers.count('Co-authored-by: Codex <noreply@openai.com>')==1 and trailers.count('Co-authored-by: Billo AI <ai-engineering@billo.systems>')==1
 protected=[]
 for root,expected_head in [(Path('/Users/oubiwann/lab/oxur/vlajmaji'),'52fd3c40a781d6945cfc74a2e79fa66b249e2a5d'),(Path('/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc03/vlajmaji'),'7b1c565ebf6c8a423e04211b599006b8c21d65d9'),(Path('/Users/oubiwann/lab/oxur/bauske'),'c5801ae17caaf86d2651b5ce34570199e873ad0f')]:
  observed=git('rev-parse','HEAD',root=root).decode().strip();status=git('status','--porcelain=v1',root=root).decode();assert observed==expected_head and status==''
  protected.append({'path':str(root),'head':observed,'status':status})
 return dict(ok=True,scope='read-only exact committed source/scope/lock/guard/trailers/protected Git boundaries; no behavioral/DB/default/target proof',head=head,parent=git('rev-parse','HEAD^').decode().strip(),tree=git('rev-parse','HEAD^{tree}').decode().strip(),base=BASE,changed_files=records,trailers=trailers,protected_git=protected)
if __name__=='__main__':
 result=check(sys.argv[1]);out=Path(sys.argv[2]);out.open('x').write(json.dumps(result,indent=2)+'\n');print(json.dumps({'ok':result['ok'],'head':result['head'],'changed_paths':len(result['changed_files'])}))
