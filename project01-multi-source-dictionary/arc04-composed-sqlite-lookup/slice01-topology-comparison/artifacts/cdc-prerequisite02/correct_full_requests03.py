from pathlib import Path
import copy,hashlib,json,math
root=Path('/tmp/vlajmaji-p01-a04-s01-cdc/full-freeze02')
old_path=root/'requests-full02.json';old=old_path.read_bytes();assert hashlib.sha256(old).hexdigest()=='9cc8a4cda7d0711531657541abc00dd4cf3b75913e90b624c7b754a2f548d9e4'
measured=[copy.deepcopy(r)for r in json.loads(old)if not r.get('prime',False)]
insert_at=next(i for i,r in enumerate(measured)if r['operation']=='reference')
new=[dict(id='page2-concat-2',operation='exact',spelling='farvi',sources=[],page_size=2,cursor_from='page-2'),dict(id='page2-concat-3',operation='exact',spelling='farvi',sources=[],page_size=2,cursor_from='page2-concat-2'),dict(id='page3-concat-2',operation='exact',spelling='farvi',sources=[],page_size=3,cursor_from='page-3')]
measured[insert_at:insert_at]=new
for r in measured:
    if r['operation']=='reference':r['spelling']=''
prime=[]
for r in measured:
    p=copy.deepcopy(r);p['id']='prime--'+p['id'];p['prime']=True
    if 'cursor_from'in p:p['cursor_from']='prime--'+p['cursor_from']
    prime.append(p)
requests=prime+measured
assert len(measured)==27 and len(prime)==27 and len(requests)==54

def page_graph(rows,total=5):
    ids={r['id']:r for r in rows};assert len(ids)==len(rows),'duplicate request ids'
    earlier=set()
    for r in rows:
        assert isinstance(r.get('spelling'),str) or r['operation']=='reference','missing explicit exact spelling'
        if r.get('cursor_from'):assert r['cursor_from']in earlier,'cursor source absent or follows consumer'
        earlier.add(r['id'])
    chains={}
    for n in [1,2,3,8]:
        name=f'page-{n}';chain=[]
        for i in range(math.ceil(total/n)):
            assert name in ids,(n,'missing request',name)
            row=ids[name];assert row['operation']=='exact'and row['page_size']==n and row['spelling']=='farvi'and row['sources']==[],(n,'changed query')
            chain.append(name)
            following=[r['id']for r in rows if r.get('cursor_from')==name and r['page_size']==n]
            if i+1<math.ceil(total/n):assert len(following)==1,(n,'missing or forked continuation',name);name=following[0]
            else:assert not following,(n,'extra continuation')
        chains[str(n)]=chain
    return chains
chains=page_graph(measured)
prime_normalized=[]
for r in prime:
    q=copy.deepcopy(r);q['id']=q['id'].removeprefix('prime--')
    if q.get('cursor_from'):q['cursor_from']=q['cursor_from'].removeprefix('prime--')
    prime_normalized.append(q)
assert page_graph(prime_normalized)==chains
controls=[]
old_measured=[r for r in json.loads(old)if not r.get('prime',False)]
try:page_graph(old_measured)
except AssertionError as error:controls.append({'control':'old48case missing page2/3 continuations rejected','result':'PASS','diagnostic':str(error)})
else:raise AssertionError('old incomplete graph accepted')
for label,rows in [('remove required page2 tail',[r for r in measured if r['id']!='page2-concat-3']),('duplicate/fork continuation',measured+[dict(id='extra-page2',operation='exact',spelling='farvi',sources=[],page_size=2,cursor_from='page-2')])]:
    try:page_graph(rows)
    except AssertionError as error:controls.append({'control':label,'result':'PASS','diagnostic':str(error)})
    else:raise AssertionError(label+' accepted')
path=root/'requests-full03.json';path.write_text(json.dumps(requests,indent=2)+'\n')
expectations=json.loads((root/'expectations-full02.json').read_text());expectations['warm_request_count']=27;expectations['warm_prime_count']=27;expectations['requests_sha256']=hashlib.sha256(path.read_bytes()).hexdigest();expectations['pages']['concat_by_size']=chains;expectations['prior_requests_sha256']=hashlib.sha256(old).hexdigest()
(root/'expectations-full03.json').write_text(json.dumps(expectations,indent=2)+'\n')
report={'status':'PASS','scope':'pre-observation workload graph qualification; no product or Full data/store operation','positive':'all positive page sizes have exactly ceil(5/page_size) requests in both priming and measured phases','page_chains':chains,'negative_controls':controls,'request_count':54,'priming_count':27,'measured_count':27,'prior_sha256':hashlib.sha256(old).hexdigest(),'current_sha256':hashlib.sha256(path.read_bytes()).hexdigest(),'refuted_static_lead':'6fc740e Request.spelling already has serde(default); actual19case tiny whole-array run/batch passed with omitted reference spelling. Explicit empty spelling is clarity, not a deserialization repair.'}
(root/'requests-full03-graph-controls.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report))
