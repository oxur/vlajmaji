"""Independent S03 mapping oracle. Authored from binding plan/public JSON before candidate.
No production projection imports. Run INPUT.json DIRECTORY/dictionary.sqlite3.
Evidence integer IDs are private surrogate keys: compare their complete dereferenced rows.
"""
import collections, hashlib, json, pathlib, sqlite3, sys
TABLES = '''metadata canonical_document entity_ids sources contributors distributors source_aliases source_lineage capture_refs snapshot_bindings occurrence_refs occurrence_attributions raw_captures raw_occurrences native_fields collections components component_languages releases release_captures release_assertions release_diagnostics code_notices content_notices redistribution_evidence entries assertions assertion_timestamps source_evidence contexts argument_frames argument_slots forms form_components etymologies etymology_components relations owned_revisions revision_assertions revision_tombstones owned_heads editorial_senses editorial_members computations computation_inputs annotations extensions'''.split()

def p(name,v):
    state=next(iter(v)) if isinstance(v,dict) else v
    return {name+'_state':state.lower(),name+'_value':v['Value'] if state=='Value' else None}
def tv(name,v):
    return {name+'_text':v['text'],**p(name+'_language',v['language']),**p(name+'_markup',v['markup'])}
def tp(name,v):
    state=next(iter(v)) if isinstance(v,dict) else v
    return {name+'_state':state.lower(),**(tv(name,v['Value']) if state=='Value' else {name+s:None for s in ['_text','_language_state','_language_value','_markup_state','_markup_value']})}
def pm(name,v):
    state=next(iter(v)) if isinstance(v,dict) else v
    return {name+'_state':state.lower(),name+'_value':v['Value']['value'] if state=='Value' else None,name+'_rule':v['Value']['rule'] if state=='Value' else None}
def local(name,v):
    k=next(iter(v)) if v is not None else None
    return {name+'_kind':k,**{name+'_'+col+'_id':v[k] if k==role else None for role,col in [('Entry','entry'),('Assertion','assertion'),('Form','form'),('EditorialSense','editorial_sense')]}}
def cap(v):return (v['namespace'],v['sha256'])
def occ(v):
    c=v.get('capture',v.get('snapshot'))
    return (*cap(c),v['locator'])
def rid(v):
    return b''.join(len(s.encode()).to_bytes(8,'big')+s.encode() for s in occ(v)).hex()
def uint(v):return None if v is None else str(v)
def freeze(v):
    if isinstance(v,bytes):return ('blob',v.hex())
    if isinstance(v,dict):return tuple(sorted((k,freeze(x)) for k,x in v.items()))
    if isinstance(v,list):return tuple(map(freeze,v))
    return (type(v).__name__,v)
def build(d,raw):
    rows={t:[] for t in TABLES};m=d['metadata'];r=d['registry']
    def add(t,**row):rows[t].append(row)
    def evidence(v):
        if v is None:return None
        off=v['offset']
        row={'occurrence_id':rid(v['occurrence']),'field_index':uint(v['field_index']),**p('citation',v['citation']),'offset_start':uint(off['start']) if off else None,'offset_end':uint(off['end']) if off else None}
        rows['source_evidence'].append(row)
        return row
    for k,v in {'format':'vlajmaji-dictionary','schema_version':'1','complete':'true','canonical_sha256':hashlib.sha256(raw).hexdigest(),'dictionary_version':str(d['version']),'registry_version':str(r['version']),'catalogue_revision':str(m['catalogue_revision'])}.items():add('metadata',key=k,value=v)
    add('canonical_document',id=1,json=raw,legacy_archive=bytes(d['legacy_archive']) if d['legacy_archive'] is not None else None)
    roles={'collections':'collection','components':'component','releases':'release','entries':'entry','assertions':'assertion','contexts':'context','frames':'frame','forms':'form','etymologies':'etymology','relations':'relation','owned_revisions':'owned_revision','editorial_senses':'editorial_sense','computations':'computation','annotations':'annotation'}
    for key,role in roles.items():
        for i,x in enumerate(m[key]):add('entity_ids',id=x['id'],role=role,ordinal=i)
    for key in ['sources','contributors','distributors']:
        for i,x in enumerate(r[key]):add(key,ordinal=i,**x)
    for i,x in enumerate(r['aliases']):add('source_aliases',ordinal=i,source_id=x['source'],text=x['text'],status=x['status'],evidence=x['evidence'])
    for i,x in enumerate(r['lineage']):
        k,v=next(iter(x['upstream'].items()))
        add('source_lineage',ordinal=i,source_id=x['source'],target_kind=k,upstream_source_id=v if k=='Known' else None,unresolved_text=v if k=='Unresolved' else None,evidence=x['evidence'])
    captures=set();occurrences={}
    def identity_walk(x):
        if isinstance(x,dict):
            if set(x)=={'namespace','sha256'}:captures.add(cap(x))
            if 'locator' in x and ('capture' in x or 'snapshot' in x):occurrences[rid(x)]=occ(x)
            for v in x.values():identity_walk(v)
        elif isinstance(x,list):
            for v in x:identity_walk(v)
    identity_walk(d)
    for c in d['captures']:
        for o in c['occurrences']:
            v={'capture':c['identity'],'locator':o['locator']};occurrences[rid(v)]=occ(v)
    for ns,sha in sorted(captures):add('capture_refs',namespace=ns,sha256=sha)
    for id_,(ns,sha,loc) in sorted(occurrences.items()):add('occurrence_refs',id=id_,namespace=ns,sha256=sha,locator=loc)
    for i,x in enumerate(r['snapshots']):add('snapshot_bindings',ordinal=i,namespace=x['snapshot']['namespace'],sha256=x['snapshot']['sha256'],source_id=x['source'],distributor_id=x['distributor'])
    for i,x in enumerate(r['occurrences']):add('occurrence_attributions',ordinal=i,occurrence_id=rid(x['occurrence']),contributor_id=x['contributor'],native_entry_id=x['native_entry_id'],native_assertion_id=x['native_assertion_id'])
    for i,x in enumerate(d['captures']):
        ns,sha=cap(x['identity']);md=x['metadata']
        add('raw_captures',namespace=ns,sha256=sha,ordinal=i,bytes=bytes(x['bytes']),source_name=md['source_name'],format_label=md['format_label'],locator=md['locator'],**p('revision',md['revision']),**p('captured_at',md['captured_at']),time_precision=md['time_precision'],**p('notice_locator',md['notice_locator']))
        for j,o in enumerate(x['occurrences']):
            oid=rid({'capture':x['identity'],'locator':o['locator']})
            add('raw_occurrences',occurrence_id=oid,namespace=ns,sha256=sha,ordinal=j,raw_line=o['raw_line'])
            for n,f in enumerate(o['fields']):add('native_fields',occurrence_id=oid,ordinal=n,**f)
    for x in m['collections']:add('collections',id=x['id'],entity_role='collection',source_id=x['source'],display_name=x['display_name'],**tp('description',x['description']))
    for x in m['components']:
        add('components',id=x['id'],entity_role='component',collection_id=x['collection'],native_name=x['native_name'])
        for i,v in enumerate(x['languages']):add('component_languages',component_id=x['id'],ordinal=i,language=v)
    for x in m['releases']:
        add('releases',id=x['id'],entity_role='release',component_id=x['component'],distributor_id=x['distributor'],**p('upstream_revision',x['upstream_revision']),**p('distributor_revision',x['distributor_revision']),**p('captured_at',x['captured_at']),time_precision=x['time_precision'],schema_version=x['schema_version'],normalizer=x['normalizer'],normalizer_version=x['normalizer_version'],**{'accounting_'+k:uint(v) for k,v in x['accounting'].items() if k!='unit'},accounting_unit=x['accounting']['unit'],selected=int(x['selected']),availability=x['availability'],redistribution=x['redistribution'])
        for i,v in enumerate(x['captures']):add('release_captures',release_id=x['id'],ordinal=i,**v)
        for i,v in enumerate(x['assertions']):add('release_assertions',release_id=x['id'],ordinal=i,assertion_id=v)
        for i,v in enumerate(x['diagnostics']):add('release_diagnostics',release_id=x['id'],ordinal=i,occurrence_id=rid(v['occurrence']) if v['occurrence'] else None,code=v['code'],message=v['message'])
        for i,v in enumerate(x['content_notices']):add('content_notices',release_id=x['id'],ordinal=i,**p('text',v['text']),**p('locator',v['locator']),**p('license',v['license']))
        for i,v in enumerate(x['redistribution_evidence']):add('redistribution_evidence',release_id=x['id'],ordinal=i,evidence=v)
    for i,x in enumerate(m['code_notices']):add('code_notices',ordinal=i,text=x['text'],**p('locator',x['locator']),**p('license',x['license']))
    for x in m['entries']:add('entries',id=x['id'],entity_role='entry',source_id=x['source'],**{k:v for field in ['spelling','language','script','native_word_class','selmaho'] for k,v in p(field,x[field]).items()},**pm('mapped_pos',x['mapped_pos']))
    for x in m['assertions']:
        el=x['eligibility'];excluded=isinstance(el,dict)
        add('assertions',id=x['id'],entity_role='assertion',entry_id=x['entry'],occurrence_id=rid(x['occurrence']),**p('native_sense_id',x['native_sense_id']),**p('language',x['language']),**tp('definition',x['definition']),**p('source_score',x['source_score']),eligibility='Excluded' if excluded else el,exclusion_reason=el['Excluded']['reason'] if excluded else None)
        for i,v in enumerate(x['timestamps']):add('assertion_timestamps',assertion_id=x['id'],ordinal=i,**v)
    for x in m['contexts']:add('contexts',id=x['id'],entity_role='context',assertion_id=x['assertion'],kind=x['kind'],**tp('value',x['value']),translated_from=x['translated_from'],evidence_id=evidence(x['evidence']))
    for x in m['frames']:
        add('argument_frames',id=x['id'],entity_role='frame',assertion_id=x['assertion'],evidence_id=evidence(x['evidence']))
        for i,v in enumerate(x['slots']):add('argument_slots',frame_id=x['id'],ordinal=i,slot_order=v['order'],native_label=v['native_label'],**tp('description',v['description']),**p('grammatical_role',v['grammatical_role']))
    for x in m['forms']:
        add('forms',id=x['id'],entity_role='form',entry_id=x['entry'],spelling=x['spelling'],**p('language',x['language']),**p('script',x['script']),**tp('pronunciation',x['pronunciation']),kind=x['kind'],**p('standing',x['standing']),evidence_id=evidence(x['evidence']),computation_id=x['computation'])
        for i,v in enumerate(x['components']):add('form_components',form_id=x['id'],ordinal=i,component_order=v['order'],spelling=v['spelling'],**local('target',v['target']))
    for x in m['etymologies']:
        add('etymologies',id=x['id'],entity_role='etymology',assertion_id=x['assertion'],**tp('statement',x['statement']),**p('native_kind',x['native_kind']),evidence_id=evidence(x['evidence']))
        for i,v in enumerate(x['components']):add('etymology_components',etymology_id=x['id'],ordinal=i,component_order=v['order'],spelling=v['spelling'],**local('target',v['target']))
    for x in m['relations']:
        kind=x['kind'];other=isinstance(kind,dict);target=x['target'];tk=next(iter(target)) if isinstance(target,dict) else target;v=target[tk] if isinstance(target,dict) else None;catalog=v if tk=='Catalog' else {}
        add('relations',id=x['id'],entity_role='relation',**local('origin',x['origin']),kind='Other' if other else kind,other_kind=kind['Other'] if other else None,direction=x['direction'],scope=x['scope'],target_kind=tk,**local('local_target',v if tk=='Local' else None),catalog_source_id=catalog.get('source'),catalog_release_id=catalog.get('release'),catalog_native_entry_id=catalog.get('native_entry_id'),catalog_native_assertion_id=catalog.get('native_assertion_id'),catalog_occurrence_id=rid(catalog['occurrence']) if catalog.get('occurrence') else None,unresolved_text=v if tk=='Unresolved' else None,evidence_id=evidence(x['evidence']),editorial_revision_id=x['editorial_revision'])
    for x in m['owned_revisions']:
        add('owned_revisions',id=x['id'],entity_role='owned_revision',source_id=x['source'],parent_id=x['parent'],editor_id=x['editor'],**p('timestamp',x['timestamp']),time_precision=x['time_precision'],**tp('rationale',x['rationale']))
        for key,table in [('assertions','revision_assertions'),('tombstones','revision_tombstones')]:
            for i,v in enumerate(x[key]):add(table,revision_id=x['id'],ordinal=i,assertion_id=v)
    for i,x in enumerate(m['owned_heads']):add('owned_heads',source_id=x['source'],ordinal=i,revision_id=x['revision'])
    for x in m['editorial_senses']:
        add('editorial_senses',id=x['id'],entity_role='editorial_sense',editor_id=x['editor'],revision_id=x['revision'],**tv('rationale',x['rationale']))
        for i,v in enumerate(x['assertions']):add('editorial_members',editorial_sense_id=x['id'],ordinal=i,assertion_id=v)
    for x in m['computations']:
        add('computations',id=x['id'],entity_role='computation',tool=x['tool'],tool_version=x['tool_version'],policy=x['policy'],**p('profile',x['profile']),**tp('explanation',x['explanation']))
        for i,v in enumerate(x['inputs']):
            k,val=next(iter(v.items()))
            add('computation_inputs',computation_id=x['id'],ordinal=i,input_kind=k,occurrence_id=rid(val) if k=='Occurrence' else None,**local('entity',val if k=='Entity' else None),literal=val if k=='Literal' else None)
    for x in m['annotations']:add('annotations',id=x['id'],entity_role='annotation',computation_id=x['computation'],**local('target',x['target']),kind=x['kind'],value=x['value'],**tp('explanation',x['explanation']))
    for i,x in enumerate(m['extensions']):add('extensions',ordinal=i,owner_id=x['owner'],name=x['name'],value=x['value'],evidence_id=evidence(x['evidence']))
    return rows

def main():
    raw=pathlib.Path(sys.argv[1]).read_bytes();expected=build(json.loads(raw),raw)
    if len(sys.argv)==2:
        print(json.dumps({'prepared_tables':len(expected),'expected_rows':{k:len(v) for k,v in expected.items()},'fixture_sha256':hashlib.sha256(raw).hexdigest()},indent=2));return
    db=sqlite3.connect(pathlib.Path(sys.argv[2]).resolve().as_uri()+'?mode=ro',uri=True);db.row_factory=sqlite3.Row
    evidence={x['evidence_id']:{k:x[k] for k in x.keys() if k!='evidence_id'} for x in db.execute('SELECT * FROM source_evidence')}
    failures=[];cells=0
    actual_tables={x[0] for x in db.execute("SELECT name FROM sqlite_schema WHERE type='table' AND name NOT LIKE 'sqlite_%'")}
    if actual_tables!=set(TABLES):failures.append(('table_set',sorted(actual_tables^set(TABLES))))
    for table,want in expected.items():
        try:
            got=[dict(x) for x in db.execute('SELECT * FROM "'+table+'"')]
            if table=='source_evidence':
                for row in got:row.pop('evidence_id')
            else:
                for row in got:
                    if 'evidence_id' in row:row['evidence_id']=evidence[row['evidence_id']] if row['evidence_id'] is not None else None
            a=collections.Counter(map(freeze,got));b=collections.Counter(map(freeze,want));cells+=sum(len(x) for x in want)
            if a!=b:failures.append((table,{'actual_rows':len(got),'expected_rows':len(want),'missing':list((b-a).elements())[:2],'unexpected':list((a-b).elements())[:2]}))
        except Exception as e:failures.append((table,repr(e)))
    print(json.dumps({'tables':len(expected),'expected_cells':cells,'failures':len(failures),'details':failures},ensure_ascii=False,indent=2));sys.exit(bool(failures))
if __name__=='__main__':main()
