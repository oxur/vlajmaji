"""Execute only exact helper definitions/control branch; never import Full modules."""
from pathlib import Path
from collections import Counter
import ast, hashlib, json, types
p = Path(__file__).with_name("native_topology.py")
b = p.read_bytes()
assert hashlib.sha256(b).hexdigest() == "23fe1d157715bbf274380920ead96efff7376e1f663679e9dcb5913292920de8"
tree = ast.parse(b)
body = []
for n in tree.body:
    if isinstance(n, ast.FunctionDef) and n.name in ("counter_equal", "context_tuple"):
        body.append(n)
    elif isinstance(n, ast.Assign) and any(isinstance(x, ast.Name) and x.id == "MISSING" for x in n.targets):
        body.append(n)
    elif isinstance(n, ast.If) and "--comparator-controls" in ast.unparse(n.test):
        body.append(n)
assert len(body) == 4
scope = {"Counter": Counter, "json": json, "sys": types.SimpleNamespace(argv=[str(p), "--comparator-controls"])}
try:
    exec(compile(ast.Module(body=body, type_ignores=[]), str(p)+":helpers-only", "exec"), scope)
except SystemExit as e:
    assert e.code == 0
else:
    raise AssertionError("exact control branch did not terminate")
