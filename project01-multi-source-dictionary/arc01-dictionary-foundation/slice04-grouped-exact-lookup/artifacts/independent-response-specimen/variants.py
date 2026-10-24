"""Additional finite S04 graph variants, source JSON only."""
import copy

def variants(d):
 out={}
 x=copy.deepcopy(d)
 for s in ['source-a','source-b']:x['registry']['aliases'].append(dict(source=s,text='historic-shared',status='Retired',evidence='CDC labelled ambiguity fixture'))
 out['alias-ambiguity']=x
 x=copy.deepcopy(d);r=copy.deepcopy(next(r for r in x['metadata']['releases'] if r['id']=='release-a'));r['id']='release-a-overlap';x['metadata']['releases'].append(r);out['overlap']=x
 x=copy.deepcopy(d);r=copy.deepcopy(next(r for r in x['metadata']['releases'] if r['id']=='release-a'));r.update(id='release-a-disabled',availability='Disabled');x['metadata']['releases'].append(r);out['mixed']=x
 x=copy.deepcopy(d);x['metadata']['owned_heads']=[];out['no-owned-head']=x
 x=copy.deepcopy(d);x['registry']['sources'].append(dict(id='source-n',display_name='No releases fixture',abbreviation='n',abbreviation_kind='ProjectAssigned',abbreviation_evidence=None,origin='Upstream'));out['no-releases']=x
 x=copy.deepcopy(d);source=copy.deepcopy(x['registry']['sources'][0]);source.update(id='source-e',display_name='Available empty fixture',abbreviation='e');x['registry']['sources'].append(source)
 collection=copy.deepcopy(x['metadata']['collections'][0]);collection.update(id='collection-e',source='source-e');x['metadata']['collections'].append(collection)
 component=copy.deepcopy(x['metadata']['components'][0]);component.update(id='component-e',collection='collection-e');x['metadata']['components'].append(component)
 release=copy.deepcopy(next(r for r in x['metadata']['releases'] if r['id']=='release-b'));release.update(id='release-e',component='component-e',captures=[],assertions=[],diagnostics=[]);release['accounting'].update(input_occurrences=0,normalized_occurrences=0,query_excluded_occurrences=0);x['metadata']['releases'].append(release);out['available-zero']=x
 return out
