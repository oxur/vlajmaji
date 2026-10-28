"""Pure phase12 stop qualification: extract SAME runner helper; no product/Full IO."""
import ast,copy,importlib.util
from pathlib import Path
HERE=Path(__file__).parent
spec=importlib.util.spec_from_file_location('numeric_analysis',HERE/'numeric-analysis01.py')
a=importlib.util.module_from_spec(spec);spec.loader.exec_module(a)
tree=ast.parse((HERE/'numeric-runner01.py').read_text())
helper=next(n for n in tree.body if isinstance(n,ast.FunctionDef) and n.name=='fresh_phase_budget')
namespace={'analysis':a};exec(compile(ast.fix_missing_locations(ast.Module(body=[helper],type_ignores=[])),'<same runner fresh_phase_budget>','exec'),namespace)
check=namespace['fresh_phase_budget']
rows=[{'kind':'fresh','mode':mode,'round':round_,'elapsed_ns':1_000_000} for mode in a.FRESH for round_ in (1,2,3)]
assert check(rows)['ok'];print('PASS exact12 retained fresh observations /four complete three-round modes')
bad=copy.deepcopy(rows)
for row in bad:
 if row['mode']=='compact' and row['round'] in (1,2):row['elapsed_ns']=5_600_000_000
verdict=check(bad)
assert not verdict['ok'] and verdict['failed']==['fresh:compact']
assert verdict['fresh']['compact']['maximum_ns']<a.FRESH['compact'][1]
print('PASS fresh median failure below maximum rejects before warm launch')
execute=next(n for n in tree.body if isinstance(n,ast.FunctionDef) and n.name=='execute')
loop=next(n for n in ast.walk(execute) if isinstance(n,ast.For) and ast.unparse(n.target)=='(number, stage)')
first=loop.body[0];assert isinstance(first,ast.If) and ast.unparse(first.test)=="stage['kind'] == 'warm' and number == 13"
assert any(isinstance(n,ast.Call) and isinstance(n.func,ast.Name) and n.func.id=='fresh_phase_budget' for n in ast.walk(first))
assert any(isinstance(n,ast.Assert) and ast.unparse(n.test)=="fresh_verdict['ok']" for n in first.body)
assert isinstance(loop.body[1],ast.If) and ast.unparse(loop.body[1].test)=="stage['kind'] == 'semantic'"
assert next(i for i,n in enumerate(loop.body) if isinstance(n,ast.Assign) and any(isinstance(t,ast.Name) and t.id=='workdir' for t in n.targets))>1
print('PASS actual runner phase12 assertion precedes warm workdir/guard/process; phase21 gate retained')
print('COMPLETE 3 pure fresh-phase method controls; no product process, database, model or timing trial')
