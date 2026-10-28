"""Bounded SAME runner inventory qualification; owned synthetic metadata only."""
from pathlib import Path
import importlib.util,os,json
D=Path('/tmp/vlajmaji-p01-a04-s05-cdc');spec=importlib.util.spec_from_file_location('runner',D/'numeric-runner01.py');r=importlib.util.module_from_spec(spec);spec.loader.exec_module(r)
B=D/'numeric-inventory-method01';B.mkdir(mode=0o700);root=B/'scratch';root.mkdir();(root/'nested').mkdir();referent=B/'referent';referent.mkdir();(referent/'must-not-appear').write_bytes(b'owned sentinel only')
(root/'regular').write_bytes(b'1234567')
links={root/'nested/link':b'../../referent',root/'dangling':b'missing-\xff',root/'loop':b'loop'}
for p,target in links.items():os.symlink(target,os.fsencode(p))
result=r.inventory([root]);entries={row['path']:row for row in result['entries']}
assert len(entries)==6 and result['bytes']==7+sum(map(len,links.values()))
for p,target in links.items():assert entries[str(p)]=={'path':str(p),'kind':'symlink','bytes':len(target),'native_target_hex':target.hex()}
assert all('must-not-appear' not in name and not name.startswith(str(referent)) for name in entries)
print('PASS nested link payload counted exactly; owned referent directory/file absent from inventory')
assert entries[str(root/'dangling')]['native_target_hex']==b'missing-\xff'.hex() and entries[str(root/'loop')]['bytes']==4
print('PASS dangling native-byte and self-loop links observed without following or recursion')
try:r.inventory([root/'loop'])
except AssertionError:print('PASS named scratch root remains strict nofollow')
else:raise AssertionError('symlink named root accepted')
try:r.leaf(root/'nested/link',root)
except AssertionError:print('PASS frozen/data leaf remains strict nofollow')
else:raise AssertionError('symlink frozen/data leaf accepted')
(B/'observed-inventory.json').open('x').write(json.dumps(result,indent=2)+'\n')
print('COMPLETE 4 inventory method controls; no product/Full process, referent read or cleanup')
