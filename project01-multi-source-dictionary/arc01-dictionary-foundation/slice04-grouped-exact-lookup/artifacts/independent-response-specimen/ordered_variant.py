"""Preserve valid repeated editorial members and nonlexicographic computation order."""
import copy

def variant(d):
 x=copy.deepcopy(d);x['metadata']['editorial_senses'][0]['assertions']=['assertion-0','assertion-0','assertion-4']
 c=copy.deepcopy(x['metadata']['computations'][0]);c['id']='computation-0';x['metadata']['computations'].append(c)
 a=copy.deepcopy(x['metadata']['annotations'][0]);a.update(id='annotation-0',computation='computation-0');x['metadata']['annotations'].append(a)
 return x
