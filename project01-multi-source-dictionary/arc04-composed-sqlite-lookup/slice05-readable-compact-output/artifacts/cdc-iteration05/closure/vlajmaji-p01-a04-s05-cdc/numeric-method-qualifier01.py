"""Pure method controls for SAME numeric-analysis01; no product/Full IO."""
import copy,importlib.util,json
from pathlib import Path
HERE=Path(__file__).parent
s=importlib.util.spec_from_file_location('numeric_analysis',HERE/'numeric-analysis01.py');a=importlib.util.module_from_spec(s);s.loader.exec_module(a)
spec=json.loads((HERE/'numeric-caller01/fixtures/requests-full03.json').read_bytes())
rows=[]
for mode in a.FRESH:
 for round_ in range(1,4):rows.append({'kind':'fresh','mode':mode,'round':round_,'elapsed_ns':1000,'stdout_bytes':100})
for mode in ('full','compact','readable'):
 for round_ in range(1,4):
  requests=[]
  for index,w in enumerate(spec):
   identity=w['id'].removeprefix('prime--');error=identity=='wrong-role'
   r={'row_type':'request','id':w['id'],'identity':identity,'prime':w.get('prime',False),'mode':mode,'round':round_,'operation':w['operation'],'ok':not error}
   if error:r.update(expected_error='invalid_reference',profile=None,whole_error_ns=1000)
   else:r.update(total_ns=1000,query_total_ns=1000,lookup_ns=600,projection_ns=400,candidate_sql_ns=100,serialization_ns=100,payload_setup_ns=100 if index==0 else None)
   requests.append(r)
  rows.append({'kind':'warm','mode':mode,'round':round_,'rows':[{'row_type':'setup','config_and_capture_ns':100},*requests,{'row_type':'complete','prime':27,'measured':27}]})
# The analysis warm TOTAL equality only applies measured requests, not first payload setup.
assert a.analyze(rows,spec)['ok'];print('PASS positive exact21 stages /3 observations each request/mode')

def mutate(mode,identity,changes,rounds=(1,2,3)):
 v=copy.deepcopy(rows)
 for stage in v:
  if stage['kind']=='warm' and stage['mode']==mode and stage['round'] in rounds:
   for r in stage['rows'][1:-1]:
    if not r['prime'] and r['identity']==identity:r.update(changes)
 return a.analyze(v,spec)

v=mutate('full','jbotci-local',{'query_total_ns':300_000_000,'total_ns':300_000_000,'lookup_ns':1,'projection_ns':299_999_999,'candidate_sql_ns':0})
assert 'warm:full:jbotci-local' in v['failed'];print('PASS whole Resolve construction gate catches slow projection even near-zero lookup')
v=mutate('full','farvi-all',{'query_total_ns':800_000_000,'total_ns':800_000_000,'lookup_ns':799_999_999,'projection_ns':1,'candidate_sql_ns':0},rounds=(2,))
assert 'warm:full:farvi-all' in v['failed'];print('PASS one request maximum cannot hide among fast different identities/rounds')
v=mutate('compact','unknown',{'serialization_ns':160_000_000})
assert 'projection+serialization:compact:unknown' in v['failed'];print('PASS compact reference projection+serialization cap includes Unknown')
v=mutate('readable','wrong-role',{'whole_error_ns':800_000_000},rounds=(3,))
assert 'warm:readable:wrong-role' in v['failed'];print('PASS real-error timing gate has no invented successful profile')
v=copy.deepcopy(rows);stage=next(r for r in v if r['kind']=='warm');stage['rows'][28]['identity']='different-request'
try:a.analyze(v,spec)
except AssertionError:print('PASS altered identity grouping rejected')
else:raise AssertionError('identity mutation accepted')
v=copy.deepcopy(rows);v[-1]['round']=2
try:a.analyze(v,spec)
except AssertionError:print('PASS duplicate round /missing third observation rejected')
else:raise AssertionError('duplicate round accepted')
print('COMPLETE 7 pure method controls; no product, database, model or metric trial')
