"""Read-only final S05 test-pass boundary; run ONLY after explicit CC handback."""
import hashlib,json,re,subprocess,sys
from pathlib import Path
SOURCE=Path("/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc04/vlajmaji")
BASE="5e19754ce7ccad0ddea788c2e344df116711814c"
OLD="432d6321e3b2431e7fe2cc5f60e9317780e984b8"
ALLOWED={"crates/search/tests/presentation_contract.rs","crates/cli/src/main.rs","crates/cli/tests/query_output_contract.rs"}
def git(*args,stdin=None):
 return subprocess.run(["git","--no-optional-locks","-C",str(SOURCE),*args],input=stdin,capture_output=True,check=True).stdout
def sha(b): return hashlib.sha256(b).hexdigest()
def main():
 expected,destination=sys.argv[1:]
 assert git("rev-parse","HEAD").decode().strip()==expected
 assert git("rev-parse","HEAD^").decode().strip()==BASE
 assert git("branch","--show-current").decode().strip()=="codex/project01-readable-compact-output"
 assert git("status","--porcelain=v2")==b""
 changed=git("diff","--name-only",BASE,expected).decode().splitlines()
 assert set(changed)==ALLOWED and len(changed)==3
 assert git("diff","--check",BASE,expected)==b""
 marker=rb"(?m)^#\[cfg\(test\)\]\nmod managed_output_tests \{"
 before=git("show",BASE+":crates/cli/src/main.rs")
 after=git("show",expected+":crates/cli/src/main.rs")
 oldmarks=list(re.finditer(marker,before));newmarks=list(re.finditer(marker,after))
 assert len(oldmarks)==len(newmarks)==1
 oldprefix=before[:oldmarks[0].start()];newprefix=after[:newmarks[0].start()]
 assert oldprefix==newprefix
 for ref in ("refs/heads/codex/project01-composed-sqlite-lookup","refs/remotes/origin/codex/project01-composed-sqlite-lookup"):
  assert git("rev-parse",ref).decode().strip()==OLD
 lock=sha((SOURCE/"Cargo.lock").read_bytes());guard=sha(Path("/tmp/vlajmaji-p01-a04-cargo").read_bytes())
 assert lock=="65cd7f39a88bdd2a3dab1fcd14e9055ff7b77cb22ec3af8c309ce49dbea78700"
 assert guard=="20fe7ba73311aa503870923af09f33f3b53262742ca4eeea2972092924cf5de8"
 message=git("show","-s","--format=%B",expected)
 parsed=git("interpret-trailers","--parse",stdin=message).decode().splitlines()
 trailers=["Co-authored-by: Codex <noreply@openai.com>","Co-authored-by: Billo AI <ai-engineering@billo.systems>"]
 assert parsed==trailers and ("\n".join(trailers)).encode() in message
 result={"ok":True,"scope_only":True,"head":expected,"parent":BASE,"tree":git("rev-parse","HEAD^{tree}").decode().strip(),"changed_files":[{"path":rel,"sha256":sha((SOURCE/rel).read_bytes()),"blob":git("rev-parse",expected+":"+rel).decode().strip()} for rel in sorted(changed)],"production_prefix_bytes":len(newprefix),"production_prefix_sha256":sha(newprefix),"all_other_git_paths_unchanged":True,"lock_sha256":lock,"guard_sha256":guard,"parsed_trailers":parsed,"limits":"Read-only Git/source boundary only; no Cargo, target, database, default, Full or network actions; not behavioral acceptance"}
 with Path(destination).open("x") as f: json.dump(result,f,indent=2);f.write("\n")
 print(json.dumps(result))
if __name__=="__main__": main()
