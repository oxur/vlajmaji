"""Pure current-runner wire comparator qualification using retained TINY outputs.
No product children/Full input/model/DB access. Run only after actual Tiny journey.
"""
import copy,hashlib,importlib.util,json,sys
from pathlib import Path
HERE=Path(__file__).parent
s=importlib.util.spec_from_file_location('runner',HERE/'numeric-runner01.py');r=importlib.util.module_from_spec(s);s.loader.exec_module(r)
def encoded(v):return (json.dumps(v,ensure_ascii=False,separators=(',',':'))+'\n').encode()
def reject(name,call):
 try:call()
 except (AssertionError,KeyError):print('PASS '+name)
 else:raise AssertionError('method accepted '+name)
def main(context,compact,full):
 for p in [context,compact,full]:r.leaf(p,HERE)
 ctx=json.loads(Path(context).read_bytes());assert ctx['fixture_scope']=='original-four-pack-synthetic-readable'
 ids=ctx['lexical_assertion_ids'];assert len(set(ids))==5
 raw=Path(compact).read_bytes();value=r.check.strict_json(raw)
 r.fresh_wire(raw,'compact',ids,None);print('PASS positive same current compact CLI schema/count/input5IDs')
 mutations=[]
 v=copy.deepcopy(value);del v['version'];mutations.append(('missing outer key',v))
 v=copy.deepcopy(value);v['extra']=None;mutations.append(('extra outer key',v))
 v=copy.deepcopy(value);del v['result']['response']['groups'][0]['hits'][0]['capture'];mutations.append(('missing mandatory compact capture key',v))
 v=copy.deepcopy(value);v['result']['response']['groups'][0]['hits'][0]['raw']=None;mutations.append(('forbidden raw extra field',v))
 v=copy.deepcopy(value);v['result']['response']['totals']['occurrences']=4;mutations.append(('wrong complete count',v))
 v=copy.deepcopy(value);v['result']['response']['groups'][0]['hits'][0]['assertion']['id']='wrong-id';mutations.append(('wrong pre-input assertion identity',v))
 for name,v in mutations:reject(name,lambda v=v:r.fresh_wire(encoded(v),'compact',ids,None))
 fullraw=Path(full).read_bytes();r.fresh_wire(fullraw,'candidate-full',ids,hashlib.sha256(fullraw).hexdigest());print('PASS positive actual Full CLI byte hash comparator')
 reject('wrong complete Full byte hash',lambda:r.fresh_wire(fullraw,'candidate-full',ids,'0'*64))
 print('COMPLETE 9 pure wire controls; complementary Rust SAME complete typed comparator covers Presence/native-role/key-hash/release/occurrence mutations')
if __name__=='__main__':
 assert len(sys.argv)==4
 main(*sys.argv[1:])
