"""Additional controls derived from full frozen-source review; not replacements for initial specimens."""
import copy,json,pathlib,sys
x=json.loads(pathlib.Path(sys.argv[1]).read_bytes());out=pathlib.Path(sys.argv[2]);out.mkdir(exist_ok=True)
def save(name,d):(out/(name+'.json')).write_text(json.dumps(d,ensure_ascii=False,separators=(',',':')))
target=next(r for r in x['metadata']['relations'] if r['id']=='relation-missing-catalogue')['target']['Catalog']['occurrence']
d=copy.deepcopy(x);m=d['metadata'];e=copy.deepcopy(m['entries'][0]);e.update(id='entry-m-assertion-only',source='source-m',spelling={'Value':'missing-assertion-only'});m['entries'].append(e)
a=copy.deepcopy(m['assertions'][0]);o=copy.deepcopy(target);o['locator']='/assertion-only';a.update(id='assertion-m-only',entry=e['id'],occurrence=o,definition='Missing',source_score='Missing',timestamps=[]);m['assertions'].append(a);next(r for r in m['releases'] if r['id']=='release-m')['assertions'].append(a['id']);save('unavailable-assertion-only',d)
d=copy.deepcopy(x);o=copy.deepcopy(target);o['locator']='/computation-only';d['metadata']['computations'][0]['inputs'].append({'Occurrence':o});save('unavailable-computation-only',d)
d=copy.deepcopy(x);d['registry']['snapshots'].reverse();save('independent-registry-order',d)
d=copy.deepcopy(x);d['captures'].reverse();save('independent-raw-capture-order',d)
