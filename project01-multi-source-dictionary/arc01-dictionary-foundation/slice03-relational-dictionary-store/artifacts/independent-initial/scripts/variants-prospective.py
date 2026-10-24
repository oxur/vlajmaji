"""Prospective positive variants; immutable accepted main fixture remains unchanged."""
import copy,json,pathlib,sys
x=json.loads(pathlib.Path(sys.argv[1]).read_bytes());out=pathlib.Path(sys.argv[2]);out.mkdir(exist_ok=True)
def save(name,d):(out/(name+'.json')).write_text(json.dumps(d,ensure_ascii=False,separators=(',',':')))
d=copy.deepcopy(x);d['metadata']['catalogue_revision']=2**64-1
r=next(v for v in d['metadata']['releases'] if v['id']=='release-m')
r['accounting'].update(input_occurrences=2**64-1,normalized_occurrences=2**64-1,query_excluded_occurrences=2**64-1);save('u64-extremes',d)
d=copy.deepcopy(x);d['registry']['sources'][0]['display_name']='\0雪λ';e=d['metadata']['entries'][0]
e.update(spelling={'Value':''},language={'Value':'\0'},script={'Value':'雪'},native_word_class={'Value':''},selmaho={'Value':'\0'},mapped_pos={'Value':{'value':'','rule':'\0'}})
d['metadata']['contexts'][0]['value']={'Value':{'text':'\0雪','language':'Null','markup':{'Value':''}}}
d['metadata']['code_notices'][0].update(text='',locator={'Value':'\0'},license='Null');d['registry']['occurrences'][0].update(native_entry_id='',native_assertion_id='\0');save('text-presence-extremes',d)
d=copy.deepcopy(x);refs=[{'Entry':'entry-a-si-e'},{'Assertion':'assertion-5'},{'Form':'form-variant'},{'EditorialSense':'editorial-farvi'}]
c=[{'order':i+1,'spelling':['','\0','雪','cpana'][i],'target':v} for i,v in enumerate(refs)]
d['metadata']['forms'][0]['components']=copy.deepcopy(c);d['metadata']['etymologies'][0]['components']=copy.deepcopy(c);save('all-component-roles',d)
d=copy.deepcopy(x);d['registry']['snapshots']=[v for v in d['registry']['snapshots'] if v['snapshot']['namespace']!='catalogue:m:future'];save('unbound-missing-capture',d)
d=copy.deepcopy(x);d['legacy_archive']=None;d['captures']=[]
for key in d['registry']:
 if key!='version':d['registry'][key]=[]
d['registry']['sources']=[copy.deepcopy(x['registry']['sources'][0])]
for key in d['metadata']:d['metadata'][key]=0 if key=='catalogue_revision' else []
d['metadata']['collections']=[copy.deepcopy(x['metadata']['collections'][0])];d['metadata']['components']=[copy.deepcopy(x['metadata']['components'][0])]
r=copy.deepcopy(x['metadata']['releases'][0]);r.update(captures=[],assertions=[],diagnostics=[],accounting={'input_occurrences':0,'normalized_occurrences':0,'query_excluded_occurrences':0,'unit':'occurrence'});d['metadata']['releases']=[r];save('available-empty',d)
d=copy.deepcopy(d);d['registry']['sources']=[];d['metadata']['collections']=[];d['metadata']['components']=[];d['metadata']['releases']=[];save('all-empty',d)
# Legacy v1 changes only the accepted archive envelope version; no proposals exist.
d=copy.deepcopy(x);archive=json.loads(bytes(d['legacy_archive']));assert not archive['proposals'];archive['version']=1
d['legacy_archive']=list(json.dumps(archive,ensure_ascii=False,separators=(',',':')).encode());save('legacy-v1',d)
# Legacy identity and metadata fields deliberately permit empty strings; raw bytes/hashes stay exact.
d=copy.deepcopy(x)
def empty_namespace(v):
 if isinstance(v,dict):
  if v.get('namespace')=='captured:a:d1':v['namespace']=''
  for z in v.values():empty_namespace(z)
 elif isinstance(v,list):
  for z in v:empty_namespace(z)
empty_namespace(d)
c=next(c for c in d['captures'] if c['identity']['namespace']=='')
c['metadata'].update(source_name='',locator='',revision={'Value':''},captured_at={'Value':''},notice_locator={'Value':''})
archive=json.loads(bytes(d['legacy_archive']));empty_namespace(archive)
a=next(a for a in archive['sources'] if a['identity']['namespace']=='')
a['metadata'].update(source_name='',locator='',revision='',captured_at='',notice_locator='')
d['legacy_archive']=list(json.dumps(archive,ensure_ascii=False,separators=(',',':')).encode());save('legacy-empty-metadata',d)
# Known unavailable source evidence has order checks but no invented loaded byte/field bound.
d=copy.deepcopy(x);m=d['metadata'];target=next(r for r in m['relations'] if r['id']=='relation-missing-catalogue')['target']['Catalog']['occurrence']
e=copy.deepcopy(m['entries'][0]);e.update(id='entry-m-extra',source='source-m',spelling={'Value':'missing-word'});m['entries'].append(e)
a=copy.deepcopy(m['assertions'][0]);a.update(id='assertion-m-extra',entry=e['id'],occurrence=copy.deepcopy(target),definition='Missing',source_score='Missing',timestamps=[]);m['assertions'].append(a)
r=next(r for r in m['releases'] if r['id']=='release-m');r['assertions'].append(a['id'])
c=copy.deepcopy(m['contexts'][0]);c.update(id='context-m-extra',assertion=a['id'],value='Missing');c['evidence'].update(occurrence=copy.deepcopy(target),field_index=2**64-1,offset={'start':2**64-1,'end':2**64-1});m['contexts'].append(c);save('unavailable-usize-extremes',d)
# Reused historical aliases and Some(empty) catalogue-native identifiers remain representable.
d=copy.deepcopy(x);d['registry']['aliases'].append({'source':'source-b','text':'archive-a','status':'Retired','evidence':''})
c=next(r for r in d['metadata']['relations'] if r['id']=='relation-missing-catalogue')['target']['Catalog'];c.update(native_entry_id='',native_assertion_id='');save('alias-reuse-native-empty',d)
print(json.dumps({'prepared_variants':sorted(p.name for p in out.glob('*.json')),'product_results':None}))
