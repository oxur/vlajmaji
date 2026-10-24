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
print(json.dumps({'prepared_variants':sorted(p.name for p in out.glob('*.json')),'product_results':None}))
