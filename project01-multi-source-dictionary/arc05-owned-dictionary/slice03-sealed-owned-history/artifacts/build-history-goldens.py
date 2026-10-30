#!/usr/bin/env python3
"""Expected S03 format INPUTS from accepted S01/S02 INPUT authorities; no product/API/SQLite calls."""
import copy
import hashlib
import json
from pathlib import Path

here = Path(__file__).resolve().parent
arc = here.parent.parent
s01 = arc / 'slice01-owned-journal/artifacts/cdc-proof/journey/inputs'
s02 = arc / 'slice02-reviewed-curation/artifacts/cdc-proof02/tmpdir/vlajmaji-a05-s02-cdc-cgznzofy'
enc = lambda x: json.dumps(x, ensure_ascii=False, separators=(',', ':')).encode()
sha = lambda x: hashlib.sha256(x).hexdigest()
sections = ['registry.sources','registry.contributors','registry.distributors','registry.aliases','registry.lineage','registry.snapshots','registry.occurrences','captures','metadata.code_notices','metadata.collections','metadata.components','metadata.releases','metadata.entries','metadata.assertions','metadata.contexts','metadata.frames','metadata.forms','metadata.etymologies','metadata.relations','metadata.owned_revisions','metadata.editorial_senses','metadata.computations','metadata.annotations','metadata.extensions']
r0 = [1,1,0,0,0,0,0,0,0,1,1,1,0,0,0,0,0,0,0,1,0,0,0,0]
r1 = [1,1,0,0,0,1,2,1,0,1,1,2,1,2,0,0,0,0,0,2,0,0,0,0]
rosters = [r0, r1, r1[:19]+[3]+r1[20:], r1[:19]+[4]+r1[20:], r1[:19]+[5]+r1[20:]]

def write(name, data):
    p = here / name
    p.parent.mkdir(parents=True, exist_ok=True)
    if p.exists():
        assert p.read_bytes() == data, str(p)
    else:
        with p.open('xb') as f:
            f.write(data)
    return {'path': name, 'bytes': len(data), 'sha256': sha(data)}

def rows(model, section):
    if section == 'captures':
        return model['captures']
    a,b = section.split('.')
    return model[a][b]

def raw_dictionary(data):
    text = data.decode()
    prefix = '{"version":1,"dictionary":'
    assert text.startswith(prefix)
    parsed, end = json.JSONDecoder().raw_decode(text, len(prefix))
    raw = text[len(prefix):end].encode()
    assert enc(parsed) == raw
    return parsed, raw

def payload(schema, head, dictionary_raw, commits, curation):
    # Embed the ORIGINAL nested Dictionary INPUT bytes directly.
    return (b'{"version":1,"journal_schema":'+str(schema).encode()+
            b',"source":"owned-example","head":'+enc(head)+b',"dictionary":'+dictionary_raw+
            b',"commits":'+enc(commits)+b',"curation":'+enc(curation)+b'}')

def seal(schema, head, ids, data):
    return enc({'version':1,'kind':'vlajmaji-owned-snapshot','snapshot_version':1,
                'journal_schema':schema,'dictionary_version':1,'registry_version':1,
                'source':'owned-example','head':head,'revision_ids':ids,
                'snapshot_sha256':sha(data),'snapshot_bytes':len(data)})

seed_raw = (s01/'seed.json').read_bytes()
edit_raw = (s01/'edit.json').read_bytes()
assert (len(seed_raw),sha(seed_raw)) == (2003,'178cb4a48245db47a568bb740f5ec123575aae38eb961c9ce05f40c7f0526c9a')
assert (len(edit_raw),sha(edit_raw)) == (5555,'e96d7fdee27a1cf84b152799b51c7b6e541aeae714396665fa98985f5c81d05f')
models = [json.loads(seed_raw),json.loads(edit_raw)]
raws = [seed_raw,edit_raw]
curation_inputs = []
for name in ['curation.json','owned-r3-curation.json','owned-r4-curation.json']:
    data = (s02/name).read_bytes()
    curation_inputs.append(json.loads(data))
    model,raw = raw_dictionary(data)
    models.append(model)
    raws.append(raw)
final = curation_inputs[-1]
assert [m['metadata']['owned_heads'][0]['revision'] for m in models] == ['owned-r0','owned-r1','owned-r2','owned-r3','owned-r4']
assert [r['id'] for r in final['records']] == ['cdc-reviewed-sense','cdc-reviewed-link']
assert [r['revision'] for r in final['records']] == ['owned-r2','owned-r2']
assert [r['active'] for r in final['revisions']] == [[],[],['cdc-reviewed-sense','cdc-reviewed-link'],['cdc-reviewed-sense'],['cdc-reviewed-sense','cdc-reviewed-link']]
assert [r['tombstones'] for r in final['revisions']] == [[],[],[],['cdc-reviewed-link'],[]]
assert len(final['contexts']) == 1 and len(final['records']) == 2
assert len(models[-1]['registry']['sources']) == 1
assert {a['id'] for a in models[-1]['metadata']['assertions']} == {'owned-assertion-a','owned-assertion-b'}
commits = []
cp = []
state_oracles = []
for i,(model,raw,roster) in enumerate(zip(models,raws,rosters)):
    assert len(roster) == len(sections) == 24
    assert [len(rows(model,s)) for s in sections] == roster
    for s,n in zip(sections,roster):
        assert rows(model,s) == rows(models[-1],s)[:n]
    rev = 'owned-r'+str(i)
    base_hash = sha(raw)
    commits.append({'revision':rev,'state_sha256':base_hash,
                    'prefixes':[{'section':s,'length':n} for s,n in zip(sections,roster)]})
    nc,nr = (0,0) if i < 2 else (1,2)
    body = {'version':1,'source':'owned-example','revision':rev,'base_state_sha256':base_hash,
            'contexts':final['contexts'][:nc],'records':final['records'][:nr],
            'revisions':final['revisions'][:i+1]}
    curation_hash = sha(enc(body))
    cp.append({'revision':rev,'contexts_length':nc,'records_length':nr,'curation_sha256':curation_hash})
    state_oracles.append({'revision':rev,'dictionary_bytes':len(raw),'state_sha256':base_hash,
                          'prefix_lengths':roster,'contexts_length':nc,'records_length':nr,
                          'curation_sha256':curation_hash})
curation = {'version':1,'contexts':final['contexts'],'records':final['records'],
            'revisions':final['revisions'],'prefixes':cp}
full = payload(2,'owned-r4',raws[-1],commits,curation)
outputs = [write('schema2-golden/snapshot.json',full),
           write('schema2-golden/seal.json',seal(2,'owned-r4',[r['revision'] for r in commits],full))]
# Same final model/head/hash but a different VALID old entry cutoff.
# Existing Entry has only source/spelling/native fields; its declared source exists at r0.
entry = models[1]['metadata']['entries'][0]
assert set(entry) == {'id','source','spelling','language','script','native_word_class','selmaho','mapped_pos'}
assert entry['source'] in [r['id'] for r in models[0]['registry']['sources']]
alt_genesis = copy.deepcopy(models[0])
alt_genesis['metadata']['entries'] = [copy.deepcopy(entry)]
alt_raw = enc(alt_genesis)
alt_commits = copy.deepcopy(commits[:2])
alt_commits[0]['prefixes'][12]['length'] = 1
alt_commits[0]['state_sha256'] = sha(alt_raw)
alt = payload(1,'owned-r1',edit_raw,alt_commits,None)
assert alt_commits[1] == commits[1]
outputs += [write('schema1-same-head-alternate/genesis.json',alt_raw),
            write('schema1-same-head-alternate/snapshot.json',alt),
            write('schema1-same-head-alternate/seal.json',seal(1,'owned-r1',['owned-r0','owned-r1'],alt))]
# These are expected INPUTs, not an export/loader result or authenticity proof.
inputs = {str(p.relative_to(arc)):{'bytes':len(p.read_bytes()),'sha256':sha(p.read_bytes())}
          for p in [s01/'seed.json',s01/'edit.json',s02/'curation.json',s02/'owned-r3-curation.json',s02/'owned-r4-curation.json']}
oracle = {'version':1,'kind':'source-guided-expected-input-not-product-proof',
          'source_dependency':'854865a72c4ca4971e1b93fd25a2e69ee14b03a8','inputs':inputs,
          'states':state_oracles,'outputs':outputs,'same_head_alternate':{
          'unchanged_final_revision':'owned-r1','unchanged_final_state_sha256':sha(edit_raw),
          'changed_old_section':'metadata.entries','old_cutoff':0,'alternate_cutoff':1,
          'alternate_genesis_sha256':sha(alt_raw),
          'validity_basis':'known Entry source exists in r0; no occurrence/release/assertion dependency; current genesis policy permits entries with zero assertions; actual from_data/load positive required by S03 tests'}}
write('history-golden-oracles.json',enc(oracle))
print(json.dumps({'outputs':outputs,'states':state_oracles},indent=2))
