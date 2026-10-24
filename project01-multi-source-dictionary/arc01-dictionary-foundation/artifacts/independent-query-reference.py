"""Independent model-scan reference for inherited A01 query policy, not an issued API.
Source JSON only; no SQLite/search production imports. Does not infer word identity.
"""
import collections,copy,json,pathlib,sys

def occurrence_id(o):
    c=o['capture'];parts=[c['namespace'],c['sha256'],o['locator']]
    return b''.join(len(v.encode()).to_bytes(8,'big')+v.encode() for v in parts).hex()
def scan(d,query,selected_sources=None):
    m=d['metadata'];entries={e['id']:e for e in m['entries']};assertions={a['id']:a for a in m['assertions']}
    revisions={r['id']:r for r in m['owned_revisions']};origins={s['id']:s['origin'] for s in d['registry']['sources']}
    heads={h['source']:set(revisions[h['revision']]['assertions']) for h in m['owned_heads']}
    # Only explicitly selected Available releases participate. Occurrence identity deduplicates
    # overlapping release membership, never duplicate text/native IDs at distinct locators.
    memberships=collections.defaultdict(list)
    for r in m['releases']:
        if r['selected'] and r['availability']=='Available':
            for a in r['assertions']:memberships[a].append(r['id'])
    rows=[]
    for aid,releases in memberships.items():
        a=assertions[aid];e=entries[a['entry']];source=e['source']
        if selected_sources is not None and source not in selected_sources:continue
        if origins[source]=='OwnedDraft' and aid not in heads.get(source,set()):continue
        if a['eligibility']!='Lexical' or e['spelling']!={'Value':query}:continue
        rows.append({'source':source,'spelling':query,'assertion_id':aid,'occurrence_id':occurrence_id(a['occurrence']),'occurrence':a['occurrence'],'releases':sorted(releases),'definition':a['definition'],'native_sense_id':a['native_sense_id'],'source_score':a['source_score']})
    rows.sort(key=lambda r:(r['source'].encode(),r['spelling'].encode(),r['occurrence_id'].encode()))
    ids=[r['occurrence_id'] for r in rows];assert len(ids)==len(set(ids))
    return {'query':query,'total_occurrences':len(rows),'distinct_spellings':len({r['spelling'] for r in rows}),'source_totals':dict(sorted(collections.Counter(r['source'] for r in rows).items())),'rows':rows}
def transitions(d):
    variants={}
    x=copy.deepcopy(d);next(r for r in x['metadata']['releases'] if r['id']=='release-a')['availability']='Disabled';variants['disabled-a']=x
    x=copy.deepcopy(d);x['legacy_archive']=None
    release=next(r for r in x['metadata']['releases'] if r['id']=='release-a');ns={c['namespace'] for c in release['captures']}
    x['captures']=[c for c in x['captures'] if c['identity']['namespace'] not in ns]
    release['availability']='Removed';release['accounting'].update(input_occurrences=None,normalized_occurrences=None,query_excluded_occurrences=None);variants['removed-a']=x
    x=copy.deepcopy(d);next(h for h in x['metadata']['owned_heads'] if h['source']=='source-o')['revision']='revision-r2'
    for r in x['metadata']['releases']:
        if r['id'] in ['release-o1','release-o2']:r['selected']=r['id']=='release-o2'
    variants['owned-r2']=x
    x=copy.deepcopy(d);x['legacy_archive']=None;s=next(s for s in x['registry']['sources'] if s['id']=='source-a');s['display_name']='Renamed captured A';s['abbreviation']='a2'
    x['registry']['aliases'].append({'source':'source-a','text':'a','status':'Retired','evidence':'labelled rename fixture'})
    for c in x['captures']:c['metadata']['locator']='renamed-path:'+c['metadata']['locator']
    variants['renamed-paths']=x
    return variants

def main():
    d=json.loads(pathlib.Path(sys.argv[1]).read_bytes());root=pathlib.Path(sys.argv[2]);root.mkdir(exist_ok=True)
    queries=['farvi',"si'e",'sie',"Si'e",'si’e',".si'e",'sy','bai','le jaica','cpana','cpan','snime','snim','never-a-word']
    refs={q:scan(d,q) for q in queries};assert refs['farvi']['total_occurrences']==6;assert refs['farvi']['source_totals']=={'source-a':4,'source-b':1,'source-o':1};assert refs["si'e"]['total_occurrences']==1;assert refs['snime']['total_occurrences']==2;assert refs['cpana']['total_occurrences']==2
    (root/'baseline-reference.json').write_text(json.dumps(refs,ensure_ascii=False,indent=2))
    for name,v in transitions(d).items():
        (root/(name+'.json')).write_text(json.dumps(v,ensure_ascii=False,separators=(',',':')))
        (root/(name+'-reference.json')).write_text(json.dumps(scan(v,'farvi'),ensure_ascii=False,indent=2))
    print(json.dumps({'baseline_reference_counts':{q:r['total_occurrences'] for q,r in refs.items()},'transition_reference_counts':{k:scan(v,'farvi')['total_occurrences'] for k,v in transitions(d).items()},'model_validation':'pending public-model caller; no product/query result claimed'},ensure_ascii=False))
if __name__=='__main__':main()
