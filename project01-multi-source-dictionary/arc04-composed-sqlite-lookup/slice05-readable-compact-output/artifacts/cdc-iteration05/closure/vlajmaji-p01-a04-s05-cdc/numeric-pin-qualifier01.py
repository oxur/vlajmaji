"""Pure owned-path qualification of builder05 pin preparation; no product/Full data."""
from pathlib import Path
import ast,importlib.util,os,stat,json
D=Path('/tmp/vlajmaji-p01-a04-s05-cdc');s=importlib.util.spec_from_file_location('r',D/'numeric-runner02.py');r=importlib.util.module_from_spec(s);s.loader.exec_module(r)
tree=ast.parse((D/'numeric-full-preflight-builder05.py').read_text());fn=next(n for n in tree.body if isinstance(n,ast.FunctionDef) and n.name=='strict_pin_path');ns={'r':r,'Path':Path};exec(compile(ast.Module(body=[fn],type_ignores=[]),'strict-pin-path-only','exec'),ns);check=ns['strict_pin_path']
B=D/'pin-qualifier01';B.mkdir(mode=0o700);(B/'real').mkdir();(B/'real/leaf').write_text('owned fixture only');(B/'linked-parent').symlink_to('real',target_is_directory=True);(B/'linked-leaf').symlink_to('real/leaf')
assert check(B/'real/leaf')==r.canonical_system_tmp(B/'real/leaf');print('PASS known system /tmp alias plus strict owned regular leaf')
for name,p in [('arbitrary ancestor link',B/'linked-parent/leaf'),('arbitrary leaf link',B/'linked-leaf')]:
 try:check(p)
 except AssertionError:print('PASS reject '+name+' before pin payload read')
 else:raise AssertionError(name)
assert check('/Library/Frameworks/Python.framework/Versions/3.11/bin/python3')==Path('/Library/Frameworks/Python.framework/Versions/3.11/bin/python3.11');print('PASS sole exact Python alias payload plus strict target')
print('COMPLETE4 pure pin-preparation controls; no symlink referent traversal by negative checks')
