"""Independent exact envelope predicates. Method only until applied to frozen product outputs."""
import copy, hashlib, json
from pathlib import Path
FAILURE_KEYS=frozenset('version ok operation category detail path pack_id attempt_id prior_local_revision proposed_generation observed_generation activation inputs'.split())
POLICY_KEYS=frozenset('version ok operation outcome path profile root packs disabled affected_pack_ids default_profile prior_revision revision prior_sha256 sha256 observed_catalogue_revision observed_availability'.split())
def failure(value,operation,category,activation):
    assert set(value)=={'error'},set(value)
    error=value['error'];assert set(error)==FAILURE_KEYS,set(error)
    assert error['version']==1 and error['ok'] is False
    assert error['operation']==operation and error['category']==category
    assert error['activation']==activation
    assert isinstance(error['inputs'],list)
    return error

def policy(value,operation,outcome):
    assert set(value)==POLICY_KEYS,set(value)
    assert value['version']==1 and value['ok'] is True
    assert value['operation']==operation and value['outcome']==outcome
    for key in ('packs','disabled','affected_pack_ids'):assert isinstance(value[key],list)
    assert isinstance(value['prior_revision'],int) and isinstance(value['revision'],int)
    return value

def qualification():
    error=dict.fromkeys(FAILURE_KEYS,None);error.update(version=1,ok=False,operation='remove',category='revision_conflict',activation='not_activated',detail='literal method fixture',inputs=[])
    envelope={'error':error}
    p=dict.fromkeys(POLICY_KEYS,None);p.update(version=1,ok=True,operation='clear_default',outcome='committed',packs=[],disabled=[],affected_pack_ids=[],prior_revision=4,revision=5,prior_sha256='a'*64,sha256='b'*64)
    results=[]
    for label,fn,value,args in [('failure',failure,envelope,('remove','revision_conflict','not_activated')),('policy',policy,p,('clear_default','committed'))]:
        fn(value,*args);results.append({'name':label+'-literal-positive','pass':True})
        target=value['error'] if label=='failure' else value
        for key in sorted(target):
            changed=copy.deepcopy(value);del (changed['error'] if label=='failure' else changed)[key]
            try:fn(changed,*args)
            except AssertionError:results.append({'name':label+'-missing-'+key,'pass':True})
            else:raise AssertionError('accepted missing '+label+' '+key)
        changed=copy.deepcopy(value);(changed['error'] if label=='failure' else changed)['extra_empty']=[]
        try:fn(changed,*args)
        except AssertionError:results.append({'name':label+'-extra-empty-key','pass':True})
        else:raise AssertionError('accepted extra '+label)
    return results

if __name__=='__main__':
    import sys
    output=Path(sys.argv[1])
    controls=qualification()
    receipt={'ok':True,'scope':'pure exact-key envelope predicates only; no product/CLI/store proof','source_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),'controls':controls,'passed':len(controls)}
    with output.open('x') as file:json.dump(receipt,file,indent=2);file.write('\n')
    print(json.dumps({'ok':True,'passed':len(controls),'source_sha256':receipt['source_sha256']}))
