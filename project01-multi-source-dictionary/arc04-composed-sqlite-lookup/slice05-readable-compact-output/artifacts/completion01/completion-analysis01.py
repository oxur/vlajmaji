"""Completion-only retained-record analysis. No product/database IO.
Original latency targets remain visible; explicit operator exception is separate.
"""
import importlib.util
from pathlib import Path
HERE=Path(__file__).parent
s=importlib.util.spec_from_file_location('original_numeric_analysis',HERE/'numeric-analysis01.py')
original=importlib.util.module_from_spec(s);s.loader.exec_module(original)
SOURCE='011af09a44abc153b8b8253f70d84147e21a6bb0'
DECISION='operator accepted current timings; report only, not a historical target pass'

def checks_for(row,setup,index,mode):
    values=[]
    if row['ok']:
        if index==0:values.append(('fresh typed startup maximum',setup['config_and_capture_ns']+row['payload_setup_ns'],8_000_000_000))
        if not row['prime']:
            values.append(('per-request warm maximum',row['query_total_ns'] if row['operation']=='reference' else row['lookup_ns'],750_000_000))
            if mode=='full' and row['id']=='farvi-all':values.append(('full hydration+serialization maximum',row['projection_ns']+row['serialization_ns'],1_000_000_000))
            if mode!='full':values.append(('compact/readable projection+serialization maximum',row['projection_ns']+row['serialization_ns'],150_000_000))
    elif not row['prime']:values.append(('whole wrong-role error+report maximum',row['whole_error_ns'],750_000_000))
    return [{'boundary':name,'actual_ns':actual,'maximum_ns':limit,'within_historical_target':actual<=limit,'operator_exception':None if actual<=limit else {'source':SOURCE,'decision':DECISION}} for name,actual,limit in values]

def validate_warm(stage,spec,*,tiny,workload_sha256,snapshot):
    rows=stage['rows'];assert len(rows)==56
    assert rows[0]['row_type']=='setup' and rows[-1]['row_type']=='complete'
    assert rows[-1]['prime']==rows[-1]['measured']==27
    assert [r['id'] for r in rows[1:-1]]==[w['id'] for w in spec]
    assert rows[0]['mode']==stage['mode'] and rows[0]['round']==stage['round']
    assert rows[0]['tiny'] is tiny and rows[0]['request_spec_sha256']==workload_sha256 and rows[0]['snapshot']==snapshot
    assert rows[-1]['mode']==stage['mode'] and rows[-1]['round']==stage['round'] and rows[-1]['tiny'] is tiny
    for index,(row,want) in enumerate(zip(rows[1:-1],spec)):
        assert row['row_type']=='request' and row['mode']==stage['mode'] and row['round']==stage['round']
        assert row['identity']==want['id'].removeprefix('prime--') and row['prime']==want.get('prime',False) and row['operation']==want['operation']
        assert row['latency_checks']==checks_for(row,rows[0],index,stage['mode'])
        if row['ok']:
            setup=row['payload_setup_ns'] or 0
            assert row['query_total_ns']==row['lookup_ns']+row['projection_ns']
            assert row['total_ns']==row['query_total_ns']+setup and row['candidate_sql_ns']<=row['lookup_ns']
            assert (row['payload_setup_ns'] is not None)==(index==0)
            if not row['prime'] and row['id']=='farvi-all' and stage['mode']!='full':assert row['wire_bytes']<= (65536 if stage['mode']=='compact' else 131072)
        else:assert row['identity']=='wrong-role' and row['expected_error']=='invalid_reference' and row['profile'] is None

def analyze(fresh,warm,spec,*,tiny,workload_sha256,snapshots):
    assert len(fresh)==12 and len(warm)==9
    for stage in warm:validate_warm(stage,spec,tiny=tiny,workload_sha256=workload_sha256,snapshot=snapshots[stage['mode']])
    historical=original.analyze(fresh+warm,spec)
    assert all(row['ok'] for row in historical['fresh'].values()),'reused fresh gate was not passed'
    exceptions=historical['failed']
    assert all(key.startswith(('startup:','warm:','projection+serialization:')) for key in exceptions)
    observation_exceptions=[{'mode':stage['mode'],'round':stage['round'],'id':row['id'],**check} for stage in warm for row in stage['rows'][1:-1] for check in row['latency_checks'] if not check['within_historical_target']]
    return {'completion_checks_passed':True,'historical_targets_met':historical['ok'],'accepted_latency_exceptions':exceptions,'per_observation_latency_exceptions':observation_exceptions,'historical_analysis':historical,'scope':'original12 passed fresh reused at source011 +9 complete new warm processes; same-ID3round stats; latency is report-only under explicit decision, not historical target PASS; correctness/byte/resource failures remain fatal'}
