from pathlib import Path
import copy,hashlib,json
base=Path('/tmp/vlajmaji-p01-a04-s01-cdc');requests={r['id']:r for r in json.loads((base/'full-freeze02/requests-full03.json').read_text())if not r.get('prime')};names=['jbotci-local','jbotci-catalogue-native333','la-catalogue-native333','llg-component','sut-native-entry']
def hp(parts):
 h=hashlib.sha256()
 for part in parts:
  b=part.encode();h.update(len(b).to_bytes(8,'big'));h.update(b)
 return h.hexdigest()
def expected_id(name,target):
 c=target['Catalog'];raw=c['occurrence']['capture']['sha256'];loc=c['occurrence']['locator']
 if name.startswith('jbotci'):return 'jbotci-'+raw+'-assertion-'+loc[1:]
 if name.startswith('la-'):return 'la-lojban-'+raw+'-assertion-'+loc[1:]
 if name.startswith('sut-'):return 'sutysisku-'+raw+'-assertion-'+loc[1:]
 return 'llg-assertion-'+hp(['llg-assertion-v1',raw,'gismu',loc])
def check(name,d):
 target=requests[name]['target'];cat=requests['jbotci-catalogue-native333']['target']if name=='jbotci-local'else target;c=cat['Catalog'];aid=expected_id(name,cat)
 assert d['target']==target and d['status']=='Resolved'and d['version']==1 and not d['unverified']and len(d['confirmed'])==1
 assert d['state_sha256']=='d9d73cbb49cc5a9ddb0ec16cb6dbb4bb7260ad49f4f8f54c6e6d21f0779b7b19'
 if name=='jbotci-local':assert d['entity']['Assertion']['id']==target['Local']['Assertion']==aid and d['requested_release']is None
 else:assert d['entity']is None and d['requested_release']['id']==c['release']and aid in d['requested_release']['assertions']
 h=d['confirmed'][0];a=h['assertion'];e=h['entry'];o=h['occurrence_attribution'];occ=c['occurrence'];capture=occ['capture']
 assert a['id']==aid and a['entry']==e['id']and e['source']==c['source']and e['spelling']=={'Value':'farvi'}and a['occurrence']==occ
 assert h['release_ids']==[c['release']]and h['payload_present']and h['current_eligible']and not h['tombstoned_by']
 assert o['occurrence']==dict(snapshot=capture,locator=occ['locator'])and o['native_entry_id']==c['native_entry_id']and o['native_assertion_id']==c['native_assertion_id']
 assert h['raw']['locator']==occ['locator']
 tupleid=b''.join(len(x.encode()).to_bytes(8,'big')+x.encode()for x in [capture['namespace'],capture['sha256'],occ['locator']]).hex();assert h['occurrence_id']==tupleid
 return dict(request=name,source=e['source'],release=c['release'],assertion=aid,capture=capture,locator=occ['locator'],native_entry_id=o['native_entry_id'],native_assertion_id=o['native_assertion_id'],entity_role='Assertion'if name=='jbotci-local'else None)
records=[];sample=None
for obs in [Path('/tmp/vlajmaji-p01-a04-s01-cc/full-round01/observations.json'),base/'full-rounds02-03/observations.json']:
 for row in json.loads(obs.read_text())['observations']:
  if row['workload']!='warm':continue
  paths={r['id']:Path(r['path'])for r in json.loads(Path(row['stdout_path']).read_bytes())['responses']}
  absent=json.loads(paths['absent'].read_bytes());assert absent['native_candidate_occurrences']==0 and absent['totals']['occurrences']==0 and not absent['groups']
  for name in names:
   d=json.loads(paths[name].read_bytes());result=check(name,d);result.update(round=row['round'],topology=row['topology']);records.append(result)
   if name=='la-catalogue-native333'and sample is None:sample=d
controls=[]
for label in ['wrong native role','retargeted assertion','wrong requested release']:
 d=copy.deepcopy(sample)
 if label=='wrong native role':d['confirmed'][0]['occurrence_attribution']['native_entry_id']='333';d['confirmed'][0]['occurrence_attribution']['native_assertion_id']=None
 elif label=='retargeted assertion':d['confirmed'][0]['assertion']['id']=requests['jbotci-local']['target']['Local']['Assertion']
 else:d['requested_release']['id']=requests['jbotci-catalogue-native333']['target']['Catalog']['release']
 try:check('la-catalogue-native333',d)
 except AssertionError:controls.append(dict(case=label,status='rejected as expected'))
 else:raise AssertionError(label+' accepted')
report=dict(status='PASS',scope='independent accepted target/ID recipe reconciliation of recorded response bodies only; no product query',positive_reference_cases=len(records),absent_native_zero_cases=9,negative_method_controls=controls,records=records,script_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),frozen_requests_sha256=hashlib.sha256((base/'full-freeze02/requests-full03.json').read_bytes()).hexdigest(),inspection_limit='First diagnostic projection accidentally included large confirmed/requested_release bodies and tool output truncated; no full textual read claimed. This complete structured query received only selected identities and checks entire target/native-role population.')
(base/'reference-scope03.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({k:v for k,v in report.items()if k!='records'}))
