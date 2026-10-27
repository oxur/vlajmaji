"""Pure finite runner02 predicates. No product, corpus, SQLite, target, or child access."""
import copy, hashlib, importlib.util, json, sys
from pathlib import Path
HERE=Path(__file__).parent
s=importlib.util.spec_from_file_location('runner02',HERE/'full-operational-runner05.py');r=importlib.util.module_from_spec(s);s.loader.exec_module(r)
c=r.check
checks=[]
def good(name,fn):fn();checks.append({'name':name,'pass':True})
def bad(name,fn):
    try:fn()
    except (AssertionError,ValueError,KeyError,UnicodeDecodeError):checks.append({'name':name,'pass':True})
    else:raise AssertionError('accepted negative '+name)
def path(p):
    b=str(p).encode();return {'encoding':'unix-bytes','hex':b.hex(),'utf8':str(p)}
root=HERE/'full-runner-pure-method05';root.mkdir(mode=0o700)
(root/'generations').mkdir();(root/'generations'/'literal-attempt').mkdir();(root/'generations'/'literal-attempt'/'store').mkdir()
p=root/'generations/literal-attempt/store'
good('native supported descriptor and exact owned store',lambda:c.safe_store(path(p),root,'literal-attempt'))
for label,v in [('traversal',path(root/'generations/literal-attempt/../literal-attempt/store')),('lossy display disagreement',{**path(p),'utf8':'different'}),('native NUL',{'encoding':'unix-bytes','hex':'00','utf8':'\0'}),('uppercase hex',{'encoding':'unix-bytes','hex':'2F','utf8':'/'})]:
    bad(label,lambda v=v:c.safe_store(v,root,'literal-attempt'))
(root/'generations'/'linked').symlink_to(root/'generations/literal-attempt',target_is_directory=True)
bad('owned symlink refused',lambda:c.safe_store(path(root/'generations/linked/store'),root,'linked'))
# Exact source-declared mutation/key/member/input schema, independent literal values.
digests={p:'a'*64 for p in r.IDS}
inputs=[{'pack_id':p,'disposition':'installed','payload_kind':'full','release_sha256':digests[p],'report':None} for p in r.IDS]
g={'generation_id':'literal-generation','content_id':'b'*64,'canonical_sha256':'c'*64,'path':path(p),'members':[{'pack_id':p,'release_sha256':digests[p],'payload_kind':'full'} for p in r.IDS]}
v={'version':1,'ok':True,'operation':'install','outcome':'installed','local_revision':1,'policy_state':'unconfigured','attempt_id':'literal-attempt','generation':g,'inputs':inputs}
def mutation(x):c.mutation(x,'install','installed',1,r.IDS,digests,'full',inputs)
good('literal complete mutation positive',lambda:mutation(v))
for container in ('outer','generation','member','input'):
    keys={'outer':v,'generation':g,'member':g['members'][0],'input':inputs[0]}[container]
    for key in keys:
        x=copy.deepcopy(v);target={'outer':x,'generation':x['generation'],'member':x['generation']['members'][0],'input':x['inputs'][0]}[container];del target[key]
        bad(container+' missing '+key,lambda x=x:mutation(x))
    x=copy.deepcopy(v);target={'outer':x,'generation':x['generation'],'member':x['generation']['members'][0],'input':x['inputs'][0]}[container];target['extra_empty']=[]
    bad(container+' extra empty',lambda x=x:mutation(x))
for label,mutate in [('wrong member kind',lambda x:x['generation']['members'][0].update(payload_kind='synthetic')),('wrong operation',lambda x:x.update(operation='remove')),('wrong disposition',lambda x:x['inputs'][0].update(disposition='validated')),('missing null input report',lambda x:x['inputs'][0].pop('report'))]:
    x=copy.deepcopy(v);mutate(x);bad(label,lambda x=x:mutation(x))
# Shared already-qualified policy/failure helper is reused, not replaced.
checks.extend({'name':'reused '+x['name'],'pass':x['pass']} for x in c.wire.qualification())
frame=json.dumps({'error':{'category':'literal'}},indent=2)+'\n0.01 real 0.00 user 0.00 sys\n123 maximum resident set size\n0 swaps\n'
good('complete pretty child JSON and declared time lines',lambda:c.time_framed_error(frame.encode()))
for name,text in [('arbitrary trailing child text',frame+'unrecognized warning\n'),('second JSON',frame+'{}\n'),('zero RSS',frame.replace('123 maximum','0 maximum')),('missing RSS',frame.replace('123 maximum resident set size\n','')),('unknown metric',frame+'0 invented metric\n')]:bad(name,lambda text=text:c.time_framed_error(text.encode()))
good('Full constants independent of input ordering',lambda: (_ for _ in ()).throw(AssertionError()) if r.content(dict(reversed(list(r.FULL_PINS.items()))))!=r.FULL_CONTENT else None)
receipt={'ok':True,'scope':__doc__,'runner_sha256':hashlib.sha256((HERE/'full-operational-runner05.py').read_bytes()).hexdigest(),'checker_sha256':hashlib.sha256((HERE/'full-receipt-checker01.py').read_bytes()).hexdigest(),'checks':checks,'passed':len(checks),'actual_Tiny_cli_journey':'PENDING frozen handback; these pure controls are not product/orchestration execution'}
with Path(sys.argv[1]).open('x') as f:json.dump(receipt,f,indent=2);f.write('\n')
print(json.dumps({'ok':True,'passed':len(checks),'runner_sha256':receipt['runner_sha256'],'checker_sha256':receipt['checker_sha256']}))
