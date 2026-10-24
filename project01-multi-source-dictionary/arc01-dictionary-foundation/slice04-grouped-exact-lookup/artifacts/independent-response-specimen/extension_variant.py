"""One selected-empty sibling release extension; positive sparse graph variant."""
import copy

def variant(d):
 x=copy.deepcopy(d)
 for selected in [True,False]:
  r=copy.deepcopy(next(r for r in x['metadata']['releases'] if r['id']=='release-a'));r.update(id='release-a-empty-'+str(selected).lower(),captures=[],assertions=[],diagnostics=[],selected=selected);r['accounting'].update(input_occurrences=0,normalized_occurrences=0,query_excluded_occurrences=0);x['metadata']['releases'].append(r)
  x['metadata']['extensions'].append(dict(owner=r['id'],name='sibling-extension-'+str(selected).lower(),value='{"label":"CDC sparse selected sibling fixture"}',evidence=None))
 return x
