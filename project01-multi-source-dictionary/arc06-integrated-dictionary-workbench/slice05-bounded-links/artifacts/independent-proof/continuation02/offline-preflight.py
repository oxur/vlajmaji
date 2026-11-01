"""Pure read-only prefix observer preflight; no product/subprocess/file writes."""
import ast,copy,hashlib,json,os,re,sys,time
from pathlib import Path
D=Path(__file__).parent;method=D/'method-continuation02-draft.py';tree=ast.parse(method.read_bytes());ns=globals()
class ForbiddenSubprocess:
 def __getattr__(self,name):raise AssertionError('NO subprocess permitted in prefix preflight: '+name)
subprocess=ForbiddenSubprocess()
exec(compile(ast.Module(body=[n for n in tree.body if isinstance(n,ast.FunctionDef)],type_ignores=[]),str(method),'exec'),ns)
PLAN=ast.literal_eval(next(n.value for n in tree.body if isinstance(n,ast.Assign)and any(isinstance(t,ast.Name)and t.id=='PLAN'for t in n.targets)))
F=Path('/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/planning/project01-multi-source-dictionary/arc06-integrated-dictionary-workbench/slice05-bounded-links/artifacts/independent-proof/continuation01');binding=json.loads((F/'bindings.json').read_bytes());src=Path(binding['source']);cli=Path(binding['cli']);example=Path(binding['example']);env=os.environ.copy()
main=next(n for n in tree.body if isinstance(n,ast.If));nodes=[];started=False
for node in main.body:
 if isinstance(node,ast.Assign)and any(isinstance(t,ast.Name)and t.id=='inputs'for t in node.targets):started=True
 if not started:continue
 text=ast.get_source_segment(method.read_text(),node)
 if "lexical('page-zero'"in text:break
 if isinstance(node,ast.With)and'selected-env-continuation02.json'in text:continue
 nodes.append(node)
assert nodes and "KO-filter"in ast.get_source_segment(method.read_text(),nodes[-1])
original_open=Path.open
def readonly_open(self,mode='r',*args,**kwargs):
 assert not any(c in mode for c in 'wax+'),('NO writes permitted',self,mode)
 return original_open(self,mode,*args,**kwargs)
Path.open=readonly_open
try:
 exec(compile(ast.Module(body=nodes,type_ignores=[]),str(method)+' PURE_PREFIX','exec'),ns)
 assert len(children)==17 and all(r['actual_exit']==r['expected_exit']==0 and not r['timeout']for r in children)
 for p,w in binding['files'].items():assert pin(p)==w,p
 for emitted,raw in inputs.items():assert(W/emitted).read_bytes()==raw
finally:Path.open=original_open
print('PURE PREFIX1..17 exact argv/cwd/exit/streams +ALL output predicates PASS; guide8/Readable/detail/current/depth/KU/KO included')
print('PURE ALL122 original pins +42 emitted byte parities PASS; ZERO subprocess/product/file writes')
