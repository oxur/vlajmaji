from pathlib import Path
import hashlib,json,gc
base=Path('/tmp/vlajmaji-p01-a04-s01-cdc')
src=Path('/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc03/vlajmaji/crates')
out=base/'full-freeze02';out.mkdir()
pins={'lensisku-jbotci':('0987ec84a8434573b5979ea854ca459ce2ea9c7fa03ebac6639574dd2b7aa460',215412497),'llg':('d8c8be02b9888c1eee3d3daa053746e483aec995935c4d225234479d4215ce12',14433039),'la-lojban':('e66951094935bd910c6a9ad12fa1bbb51f0578b53d44f22a73056811fe2bc64e',111187880),'sutysisku':('67499447426d34c269577eb6947d7c7b22b71090e5a30fd653375819d590ffa2',94511884)}
meta=['assertions','entries','contexts','frames','forms','etymologies','relations','collections','components','releases','owned_revisions','editorial_senses','computations','annotations']
registry=['sources','contributors','distributors','aliases','lineage','snapshots']
seen={k:{}for k in meta+registry};seen['code_notices']={};seen['owned_heads']={}
raw_totals={'captures':0,'occurrences':0,'native_fields':0,'extensions':0}
extension_owners=set();capture_ids=set();packs=[];notes={}
def body(row):return json.dumps(row,sort_keys=True,separators=(',',':'),ensure_ascii=False)
def insert(category,key,row):
    previous=seen[category].get(key)
    if previous is not None:assert previous==row,('unequal shared keyed record',category,key)
    else:seen[category][key]=row
for name,(expected,length) in pins.items():
    directory=src/('data-'+name)/'data';path=directory/'release.json'
    raw=path.read_bytes();actual=hashlib.sha256(raw).hexdigest();assert actual==expected and len(raw)==length
    d=json.loads(raw);del raw
    assert d['version']==1 and d['registry']['version']==1 and d['legacy_archive'] is None
    m=d['metadata'];r=d['registry'];counts={k:len(m[k])for k in meta+['code_notices','owned_heads','extensions']};counts.update({k:len(r[k])for k in registry})
    counts.update(captures=len(d['captures']),occurrences=sum(len(c['occurrences'])for c in d['captures']),native_fields=sum(len(o['fields'])for c in d['captures']for o in c['occurrences']))
    assert len(counts)==26
    for category in meta:
        ids=[x['id']for x in m[category]];assert len(ids)==len(set(ids)),(name,category,'within-input duplicate')
        for row in m[category]:insert(category,row['id'],row)
    for row in m['code_notices']:insert('code_notices',body(row),row)
    for row in m['owned_heads']:insert('owned_heads',row['source'],row)
    for category in registry:
        for row in r[category]:
            if category in ['sources','contributors','distributors']:key=row['id']
            elif category=='aliases':key=(row['source'],row['text'])
            else:key=body(row)
            insert(category,key,row)
    owners={row['owner']for row in m['extensions']};overlap=extension_owners&owners
    assert not overlap,('extension owners cross packs; sum cannot be assumed',name,sorted(overlap)[:5]);extension_owners.update(owners)
    for capture in d['captures']:
        identity=(capture['identity']['namespace'],capture['identity']['sha256']);assert identity not in capture_ids;capture_ids.add(identity)
    for category in raw_totals:raw_totals[category]+=counts[category]
    seal=directory/'seal.json';seal_bytes=seal.read_bytes();seal_doc=json.loads(seal_bytes);assert seal_doc['release_sha256']==expected and seal_doc['payload_kind']=='full'
    packs.append(dict(id=name,release=str(path),seal=str(seal),expected_sha256=expected,expected_population=counts,store={'lensisku-jbotci':'jbotci','llg':'llg','la-lojban':'la','sutysisku':'sut'}[name]))
    notes[name]={'canonical_sha256':actual,'canonical_bytes':length,'seal_sha256':hashlib.sha256(seal_bytes).hexdigest(),'counts':counts,'extension_owner_count':len(owners),'cross_pack_extension_owner_overlap':0}
    del d,m,r;gc.collect()
combined={k:len(v)for k,v in seen.items()};combined.update(raw_totals);assert len(combined)==26
assert combined['assertions']==78361 and combined['entries']==76722 and combined['native_fields']==566367 and combined['contexts']==177210 and combined['forms']==11598 and combined['etymologies']==3513 and combined['extensions']==138107 and combined['sources']==3 and combined['contributors']==238 and combined['distributors']==2 and combined['captures']==8
config=dict(generation='arc04-s01-full-comparison02',output_root='/tmp/vlajmaji-p01-a04-s01-cc/full-comparison02',unified_state_sha256='d9d73cbb49cc5a9ddb0ec16cb6dbb4bb7260ad49f4f8f54c6e6d21f0779b7b19',unified_expected_population=combined,packs=packs)
(out/'prepare-full02.json').write_text(json.dumps(config,indent=2)+'\n')
report={'scope':'pinned accepted normalized source-input census for predeclared maps; no candidate DB read or creation; counts not native tuple proof','input_notes':notes,'combined':combined,'union_method':'keyed metadata identities and complete-body conflict checks; registry full-body union; extension sums only after disjoint cross-pack owner proof; disjoint capture identities; all empty categories retained'}
(out/'input-census.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps({'status':'PASS','combined':combined,'config':str(out/'prepare-full02.json'),'config_sha256':hashlib.sha256((out/'prepare-full02.json').read_bytes()).hexdigest()}))
