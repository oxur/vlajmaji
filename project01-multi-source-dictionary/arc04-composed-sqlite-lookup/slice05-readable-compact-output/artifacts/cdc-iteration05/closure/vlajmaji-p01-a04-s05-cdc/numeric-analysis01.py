"""Prospective v1.2 S05 numerical acceptance. No product IO; consumes retained receipts only.
Same identity+implementation across EXACT three rounds; no pooling/retry selection.
Source preparation only until actual frozen source/runner/Tiny/root readiness.
"""
import json,statistics
from pathlib import Path
NS=1_000_000_000
FRESH={'baseline-full':(6*NS,10*NS),'candidate-full':(6*NS,10*NS),'compact':(5_500_000_000,9*NS),'readable':(5_500_000_000,9*NS)}
WARM=(250_000_000,750_000_000)
STARTUP=(5*NS,8*NS)
FULL_PROJECTION=(500_000_000,NS)
SMALL_PROJECTION=(50_000_000,150_000_000)

def stats(values,limit):
    assert len(values)==3 and all(type(v) is int and v>=0 for v in values)
    median=sorted(values)[1];maximum=max(values)
    return {'observations_ns':values,'median_ns':median,'maximum_ns':maximum,'median_limit_ns':limit[0],'maximum_limit_ns':limit[1],'ok':median<=limit[0] and maximum<=limit[1]}

def analyze(stage_rows,spec):
    measured=[r for r in spec if not r.get('prime',False)];assert len(measured)==27
    ids=[r['id'] for r in measured];assert len(set(ids))==27
    operations={r['id']:r['operation'] for r in measured}
    assert len(stage_rows)==21 # 12 actual fresh CLI +9 warm observers; separate semantic stage22.
    out={'fresh':{},'startup':{},'warm':{},'projection_serialization':{},'failed':[]}
    for mode,limits in FRESH.items():
        rows=[r for r in stage_rows if r['kind']=='fresh' and r['mode']==mode]
        assert sorted(r['round'] for r in rows)==[1,2,3]
        rows.sort(key=lambda r:r['round'])
        result=stats([r['elapsed_ns'] for r in rows],limits);out['fresh'][mode]=result
        if not result['ok']:out['failed'].append('fresh:'+mode)
        if mode in ('compact','readable'):
            byte_limit=65536 if mode=='compact' else 131072
            assert all(type(r['stdout_bytes']) is int and r['stdout_bytes']<=byte_limit for r in rows)
    for mode in ('full','compact','readable'):
        stages=[r for r in stage_rows if r['kind']=='warm' and r['mode']==mode]
        assert sorted(r['round'] for r in stages)==[1,2,3];stages.sort(key=lambda r:r['round'])
        observed={i:[] for i in ids};startup=[]
        for stage in stages:
            rows=stage['rows'];assert len(rows)==56
            assert rows[0]['row_type']=='setup' and rows[-1]['row_type']=='complete'
            assert rows[-1]['prime']==rows[-1]['measured']==27
            requests=rows[1:-1];assert len(requests)==54
            assert [r['id'] for r in requests]==[r['id'] for r in spec]
            prime=[r for r in requests if r['prime']];actual=[r for r in requests if not r['prime']]
            assert len(prime)==len(actual)==27 and [r['identity'] for r in actual]==ids
            assert requests[0]['payload_setup_ns'] is not None
            assert all(r.get('payload_setup_ns') is None for r in requests[1:] if r['ok'])
            startup.append(rows[0]['config_and_capture_ns']+requests[0]['payload_setup_ns'])
            for r in actual:
                assert r['mode']==mode and r['round']==stage['round']
                if r['ok']:
                    assert r['payload_setup_ns'] is None
                    assert r['query_total_ns']==r['lookup_ns']+r['projection_ns']
                    assert r['total_ns']==r['query_total_ns'] and r['candidate_sql_ns']<=r['lookup_ns']
                else:
                    assert r['identity']=='wrong-role' and r['expected_error']=='invalid_reference' and r['profile'] is None
                observed[r['identity']].append(r)
        out['startup'][mode]=stats(startup,STARTUP)
        if not out['startup'][mode]['ok']:out['failed'].append('startup:'+mode)
        for identity,rows in observed.items():
            assert len(rows)==3
            # WHOLE Resolve is warm time, NEVER subtract its projection.
            # Wrong-role records actual whole call+typed report construction, no fake profile.
            field='whole_error_ns' if identity=='wrong-role' else 'query_total_ns' if operations[identity]=='reference' else 'lookup_ns'
            key=mode+':'+identity
            result=stats([r[field] for r in rows],WARM);result['boundary']=field
            out['warm'][key]=result
            if not result['ok']:out['failed'].append('warm:'+key)
            if mode=='full' and identity=='farvi-all':
                p=stats([r['projection_ns']+r['serialization_ns'] for r in rows],FULL_PROJECTION)
            elif mode in ('compact','readable') and identity!='wrong-role':
                p=stats([r['projection_ns']+r['serialization_ns'] for r in rows],SMALL_PROJECTION)
            else:continue
            p['reference_overlap_not_subtracted_from_warm']=operations[identity]=='reference'
            out['projection_serialization'][key]=p
            if not p['ok']:out['failed'].append('projection+serialization:'+key)
    out['ok']=not out['failed']
    out['scope']='actual 3 observations per identity/mode; priming excluded; no heterogeneous pooling, retries or detail cap'
    return out
