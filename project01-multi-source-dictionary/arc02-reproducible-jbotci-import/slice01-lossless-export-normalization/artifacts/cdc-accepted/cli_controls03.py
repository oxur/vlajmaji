#!/usr/bin/env python3
"""Independent CDC CLI behavioral controls; synthetic inputs, no build activity."""
import copy
import collections
import hashlib
import json
import subprocess
import runpy
import sys
from pathlib import Path

ROOT=Path(__file__).parent/'cli-control-cases03'
ROOT.mkdir(exist_ok=False)
LOGS=ROOT/'_logs';LOGS.mkdir()
BIN=str(Path(sys.argv[1]).resolve())
RESULTS=[]
PARSER=runpy.run_path(str(Path(__file__).parent/'verify_normalized.py'))
REV0='0'*40
REV1='1'*40
CLASSES=['bu-letteral','cmavo','cmavo-compound','cmevla','experimental cmavo',
         'experimental gismu',"fu'ivla",'gismu','lujvo','nalvla','obsolete cmavo',
         'obsolete cmevla',"obsolete fu'ivla",'obsolete zei-lujvo','phrase','zei-lujvo']

def sha(b):return hashlib.sha256(b).hexdigest()
def v(x):return {'Value':x}
def row(word='farvi',kind='gismu',definition='evolves',**extra):
    x={'word':word,'word_type':kind,'definition':definition,'definition_id':7,
       'score':-1,'user':{'username':'TestAuthor'}}
    x.update(extra);return x

def metadata(raw,count,words,**updates):
    x={'language_tag':'en','language_realname':'English','source_language_tag':'jbo',
       'format':'json','positive_scores_only':False,'filename':'dictionary-en.json',
       'metadata_url':'https://lensisku.lojban.org/api/language/languages',
       'download_url':'https://lensisku.lojban.org/api/export/dictionary/en?format=json&source_lang=jbo&positive_scores_only=false',
       'lensisku_created_at':'2026-10-04T03:07:08Z','sha256':sha(raw),
       'definition_count':count,'entry_count':words}
    x.update(updates)
    return ''.join(k+' = '+json.dumps(value,ensure_ascii=False)+'\n' for k,value in x.items()).encode()

def fixture(name,rows,eligible,raw=None,meta=None):
    d=ROOT/name;d.mkdir()
    raw=raw if raw is not None else json.dumps(rows,ensure_ascii=False,separators=(',',':')).encode()
    meta=meta if meta is not None else metadata(raw,len(rows),eligible)
    (d/'raw.json').write_bytes(raw);(d/'meta.toml').write_bytes(meta)
    return d,raw,meta

def invoke(d,raw,meta,operation='jbotci-import',revision=REV0,extra=(),expected=None,output='release.json',hashes=None):
    args=[BIN,operation,str(d/'raw.json'),str(d/'meta.toml')]
    if operation=='jbotci-import':args.append(str(d/output))
    rh,mh=hashes or (sha(raw),sha(meta))
    args+=['--revision',revision,'--raw-sha256',rh,'--metadata-sha256',mh,*map(str,extra)]
    occupied_before=(d/output).exists() or (d/output).is_symlink()
    names_before={p.name for p in d.iterdir()}
    before={p.name:sha(p.read_bytes()) for p in d.iterdir() if p.is_file() and not p.is_symlink()}
    p=subprocess.run(args,capture_output=True)
    (LOGS/(str(len(RESULTS))+'-'+d.name+'.stdout')).write_bytes(p.stdout);(LOGS/(str(len(RESULTS))+'-'+d.name+'.stderr')).write_bytes(p.stderr)
    evidence={'name':d.name,'operation':operation,'revision':revision,'expected':expected or 'success','exit':p.returncode}
    if expected:
        assert p.returncode!=0 and p.stdout==b'',(evidence,p.stdout,p.stderr)
        error=json.loads(p.stderr)['error'];assert error['code']==expected,(evidence,error)
        if operation=='jbotci-import':assert occupied_before or not ((d/output).exists() or (d/output).is_symlink()),(evidence,'unexpected output')
    else:
        assert p.returncode==0,(evidence,p.stderr)
        evidence['report']=json.loads(p.stdout)['report']
    for name,h in before.items():assert sha((d/name).read_bytes())==h,(evidence,'mutated '+name)
    names_after={p.name for p in d.iterdir()}
    assert names_after==(names_before if expected or operation=='jbotci-check' else names_before|{output}),(evidence,'unexpected directory mutation')
    RESULTS.append(evidence)
    if not expected and operation=='jbotci-import':
        doc=json.loads((d/output).read_bytes())
        caps={c['identity']['namespace']:c for c in doc['captures']}
        cap=caps['captured:lensisku:jbotci:en']
        assert bytes(cap['bytes'])==raw and bytes(caps['captured:lensisku:jbotci:en:metadata']['bytes'])==meta
        original=PARSER['raw_array'](raw.decode())
        assert len(original)==len(cap['occurrences'])
        gotrows={x['locator']:x for x in cap['occurrences']}
        assertions={x['occurrence']['locator']:x for x in doc['metadata']['assertions']}
        known={'word','word_type','definition','definition_id','score','user','notes','jargon','etymology','selmaho','rafsi','experimental_rafsi','gloss_keywords','place_keywords'}
        native_count=0;field_histogram=collections.Counter();class_histogram=collections.Counter()
        for i,(value,lexeme) in enumerate(original):
            fs=PARSER['raw_object'](lexeme) if isinstance(value,dict) else [('$row',lexeme,value)]
            native_count+=len(fs);field_histogram.update(k for k,_,_ in fs)
            class_histogram.update(value for key,_,value in fs if key=='word_type' and isinstance(value,str))
            loc='/'+str(i)
            assert gotrows[loc]['fields']==[{'name':k,'encoding':'Json','raw_value':lex} for k,lex,_ in fs]
            if isinstance(value,dict):
                a=assertions[loc]
                for j,(name,lex,_) in enumerate(fs):
                    if name not in known:
                        assert any(x['owner']==a['id'] and x['value']==lex and x['evidence']['field_index']==j and x['evidence']['occurrence']==a['occurrence'] for x in doc['metadata']['extensions'])
        assert evidence['report']['native_fields']==native_count
        assert evidence['report']['word_types']==dict(class_histogram)
        if all(isinstance(value,dict) for value,_ in original):
            assert evidence['report']['field_counts']==dict(field_histogram)
        return doc
    return evidence

def main():
    # All declared classes, negative score and optional/native preservation.
    rows=[row(word='word'+str(i),kind=k) for i,k in enumerate(CLASSES)]
    rows[0].update({'rafsi':'farv fav farv','experimental_rafsi':['farv','maz'],
        'notes':None,'jargon':'','etymology':None,'selmaho':None,
        'gloss_keywords':[{'word':'','meaning':None},{'word':'gloss'},{'word':'gloss','meaning':''}],
        'place_keywords':[{'word':'place','meaning':None}],
        'opaque':{'big':123456789012345678901234567890,'null':None}})
    # Exact duplicate row and native ID do not deduplicate.
    rows.append(copy.deepcopy(rows[0]))
    d,raw,meta=fixture('all-classes duplicates space',rows,16)
    a=invoke(d,raw,meta);b=invoke(d,raw,meta,output='repeat.json')
    c=invoke(d,raw,meta,revision=REV1,output='revision.json')
    assert a==b and (d/'release.json').read_bytes()==(d/'repeat.json').read_bytes()
    assert a['metadata']['releases'][0]['id']!=c['metadata']['releases'][0]['id']
    assert [x['identity'] for x in a['captures']]==[x['identity'] for x in c['captures']]
    for key in ['version','sources','contributors','distributors','aliases','snapshots','occurrences']:
        assert a['registry'][key]==c['registry'][key],('revision altered stable registry '+key)
    for key in a['metadata']:
        if key not in ['releases','extensions']:
            assert a['metadata'][key]==c['metadata'][key],('revision altered stable '+key)
    assert [x for x in a['metadata']['extensions'] if x['owner']!=a['metadata']['releases'][0]['id']]==[x for x in c['metadata']['extensions'] if x['owner']!=c['metadata']['releases'][0]['id']]
    for x,y in zip(a['captures'],c['captures']):
        assert x['bytes']==y['bytes'] and x['occurrences']==y['occurrences']
    assertions=a['metadata']['assertions'];entries=a['metadata']['entries']
    assert len(assertions)==len(entries)==17 and len({x['id'] for x in assertions})==17
    assert len({x['id'] for x in entries})==17
    assert all(x['eligibility']=='Lexical' for x in assertions)
    first=next(x for x in assertions if x['occurrence']['locator']=='/0')
    assert first['source_score']==v('-1')
    assert first['native_sense_id']=='Missing'
    assert next(e for e in entries if e['id']==first['entry'])['selmaho']=='Null'
    e0=first['entry'];forms=[f for f in a['metadata']['forms'] if f['entry']==e0]
    assert len(forms)==5
    assert [f['spelling'] for f in forms if f['standing']==v('rafsi')]==['farv','fav','farv']
    assert [f['spelling'] for f in forms if f['standing']==v('experimental_rafsi')]==['farv','maz']
    cs=[x for x in a['metadata']['contexts'] if x['assertion']==first['id']]
    assert [x['value'] for x in cs if x['kind']=='Note']==['Null']
    assert [x['value']['Value']['text'] for x in cs if x['kind']=='Gloss']==['','gloss','gloss']
    assert [x['value']['Value']['text'] for x in cs if x['kind']=='Domain']==['']
    assert len(a['metadata']['etymologies'])==2 and all(x['statement']=='Null' for x in a['metadata']['etymologies'])
    invoke(d,raw,meta,operation='jbotci-check',extra=['--release',d/'release.json'],output='check')
    invoke(d,raw,meta,operation='jbotci-check',revision=REV1,extra=['--release',d/'release.json'],expected='jbotci_release_mismatch',output='cross-check')
    # Existing output protected before final mutation, including symlink target.
    invoke(d,raw,meta,expected='io')
    (d/'protected').write_bytes(b'preserve me')
    (d/'link.json').symlink_to(d/'protected')
    invoke(d,raw,meta,expected='io',output='link.json')
    assert (d/'protected').read_bytes()==b'preserve me'
    (d/'directory.json').mkdir()
    invoke(d,raw,meta,expected='io',output='directory.json')

    # Invalid optional fields keep the lexical occurrence; invalid critical fields do not.
    mixed=[row('optional',notes=4,rafsi={'bad':1},gloss_keywords='bad'),
           row('wiki',kind='wiki',rafsi='wikr',selmaho='BY'),
           row('unknown',kind='future-kind'),row('undefined',definition='  '),
           row('null-definition',definition=None),row(word=None),
           {'word':'missing-definition','word_type':'gismu'},42,
           row('empty-definition',definition=''),row('valid-counterpart')]
    d2,r2,m2=fixture('critical-optional',mixed,2)
    doc=invoke(d2,r2,m2)
    byloc={x['occurrence']['locator']:x for x in doc['metadata']['assertions']}
    elig=[byloc['/'+str(i)]['eligibility'] for i in range(len(mixed))]
    assert [i for i,x in enumerate(elig) if x=='Lexical']==[0,9]
    assert len(doc['captures'][0]['occurrences'])==len(mixed)
    assert all(x['Excluded']['reason'] for x in elig[1:9])
    assert byloc['/4']['definition']=='Null' and byloc['/6']['definition']=='Missing'
    assert byloc['/8']['definition']['Value']['text']==''
    assert RESULTS[-1]['report']['excluded_occurrences']==8
    a0=byloc['/0']
    assert not [x for x in doc['metadata']['contexts'] if x['assertion']==a0['id']]
    assert not [x for x in doc['metadata']['forms'] if x['entry']==a0['entry']]
    scalar=byloc['/7']
    assert not [x for x in doc['metadata']['extensions'] if x['owner']==scalar['id']]
    assert not [x for x in doc['metadata']['releases'][0]['diagnostics'] if x['occurrence']==scalar['occurrence'] and x['code']=='unknown_native_field']
    assert doc['captures'][0]['occurrences'][7]['fields']==[{'name':'$row','encoding':'Json','raw_value':'42'}]
    invoke(d2,r2,m2,operation='jbotci-check',output='check')

    # Exact spelling/text and empty/null/missing are distinct from eligibility checks.
    spellings=[row(' farvi ',definition='  defined  '),row('farvi'),row('')]
    ds,rs,ms=fixture('spelling-preservation',spellings,2)
    sd=invoke(ds,rs,ms)
    sa={x['occurrence']['locator']:x for x in sd['metadata']['assertions']}
    se={x['id']:x for x in sd['metadata']['entries']}
    assert se[sa['/0']['entry']]['spelling']==v(' farvi ')
    assert sa['/0']['definition']['Value']['text']=='  defined  '
    assert sa['/0']['eligibility']=='Lexical' and sa['/1']['eligibility']=='Lexical'
    assert isinstance(sa['/2']['eligibility'],dict)
    assert RESULTS[-1]['report']['all_spellings']==3 and RESULTS[-1]['report']['eligible_spellings']==2

    # Repeated keys at top and nested depths remain raw, ambiguous rows excluded.
    r3=b'[{"word":"first","word":"second","word_type":"gismu","definition":"x"},'+b'{"word":"nested","word_type":"gismu","definition":"x","user":{"username":"one","username":"two"}},'+b'{"word":"valid","word_type":"gismu","definition":"x"}]'
    d3,r3,m3=fixture('duplicate-keys',[1,2,3],1,raw=r3)
    dd=invoke(d3,r3,m3)
    dx={x['occurrence']['locator']:x for x in dd['metadata']['assertions']}
    assert [dx['/'+str(i)]['eligibility']=='Lexical' for i in range(3)]==[False,False,True]
    assert [f['name'] for f in dd['captures'][0]['occurrences'][0]['fields']].count('word')==2

    # User attribution is optional, source-scoped and exact; invalid display names use a diagnosed fallback.
    actors=[row('actor'+str(i)) for i in range(7)]
    actors[0]['user']={'username':'Alice','realname':''}
    actors[1]['user']={'username':'alice'}
    actors[2]['user']={'username':' Bad\nName'}
    actors[3]['user']=None
    del actors[4]['user']
    actors[5]['user']={'realname':'Only real name'}
    actors[6]['user']='unsupported'
    da,ra,ma=fixture('attribution-presence',actors,7)
    ad=invoke(da,ra,ma)
    assert all(x['eligibility']=='Lexical' for x in ad['metadata']['assertions'])
    ac={x['id']:x['display_name'] for x in ad['registry']['contributors']}
    aa=sorted(ad['registry']['occurrences'],key=lambda x:int(x['occurrence']['locator'][1:]))
    assert ac[aa[0]['contributor']]=='Alice' and ac[aa[1]['contributor']]=='alice'
    assert aa[0]['contributor']!=aa[1]['contributor']
    assert aa[2]['contributor'] is not None and ac[aa[2]['contributor']]!=' Bad\nName'
    assert all(x['contributor'] is None for x in aa[3:])
    assert len(ac)==3

    # Exact input-number lexemes survive beyond floating point and integer ranges.
    r4=b'[{"word":"numbers","word_type":"gismu","definition":"x","definition_id":123456789012345678901234567890,"score":1.2300e+004,"opaque":-0.000}]'
    d4,r4,m4=fixture('number-lexemes',[1],1,raw=r4)
    n=invoke(d4,r4,m4)
    assert n['metadata']['assertions'][0]['source_score']==v('1.2300e+004')
    assert n['registry']['occurrences'][0]['native_assertion_id']=='123456789012345678901234567890'
    assert n['captures'][0]['occurrences'][0]['fields'][-1]['raw_value']=='-0.000'

    # A valid empty array is a complete zero-population release, not a panic/drop case.
    de,re,me=fixture('empty-export',[],0)
    empty=invoke(de,re,me)
    assert empty['metadata']['assertions']==[] and empty['metadata']['entries']==[]
    assert RESULTS[-1]['report']['input_rows']==0
    invoke(de,re,me,operation='jbotci-check',output='check')

    # One predicate at a time, with valid inputs above establishing counterparts.
    small=[row()]
    negative=[('root',b'{}',None,'jbotci_json'),('syntax',b'[',None,'jbotci_json'),
              ('raw-utf8',b'\xff',None,'jbotci_utf8'),
              ('meta-utf8',None,b'\xff','jbotci_utf8')]
    for name,rb,mb,code in negative:
        rb=rb if rb is not None else json.dumps(small).encode()
        mb=mb if mb is not None else metadata(rb,1,1)
        dx,rx,mx=fixture(name,small,1,raw=rb,meta=mb)
        invoke(dx,rx,mx,expected=code)
    for name,update,code in [('unsupported',{'language_tag':'fr'},'jbotci_metadata'),
            ('metadata-digest',{'sha256':'f'*64},'jbotci_metadata')]:
        rb=json.dumps(small).encode();mb=metadata(rb,1,1,**update)
        dx,rx,mx=fixture(name,small,1,raw=rb,meta=mb);invoke(dx,rx,mx,expected=code)
    for name,suffix in [('metadata-duplicate',b'format = "json"\n'),('metadata-trailing',b'bad = true junk\n')]:
        rb=json.dumps(small).encode();mb=metadata(rb,1,1)+suffix
        dx,rx,mx=fixture(name,small,1,raw=rb,meta=mb);invoke(dx,rx,mx,expected='jbotci_metadata')
    dx,rx,mx=fixture('valid-small',small,1)
    invoke(dx,rx,mx,revision='BAD',expected='jbotci_options')
    invoke(dx,rx,mx,hashes=('f'*64,sha(mx)),expected='jbotci_hash_mismatch')
    invoke(dx,rx,mx)
    baseline_diagnostics=collections.Counter(RESULTS[-1]['report']['diagnostics'])
    du,ru,mu=fixture('unknown-simple-metadata',small,1,raw=rx,meta=mx+b'extra_source_key = \"preserved\"\n')
    invoke(du,ru,mu)
    assert sum((collections.Counter(RESULTS[-1]['report']['diagnostics'])-baseline_diagnostics).values())>=1
    rb=json.dumps(small).encode();mb=metadata(rb,2,99)
    dc,rc,mc=fixture('count-mismatch',small,1,raw=rb,meta=mb)
    invoke(dc,rc,mc)
    assert not RESULTS[-1]['report']['definition_count_matches'] and not RESULTS[-1]['report']['entry_count_matches']
    invoke(dc,rc,mc,operation='jbotci-check',expected='jbotci_accounting',output='check')
    # Duplicate-key suppression must retain indexed native unknown evidence.
    dupraw=b'[{"word":"farvi","word_type":"gismu","definition":"a","definition":"b","opaque":{"n":123456789012345678901234567890},"opaque":null},{"word":"farvi","word_type":"gismu","definition":"a","opaque":{"n":123456789012345678901234567890}}]'
    dp,dr,dm=fixture('duplicate-unknown-fields',[1,2],1,raw=dupraw)
    du=invoke(dp,dr,dm)
    da={a['occurrence']['locator']:a for a in du['metadata']['assertions']}
    assert isinstance(da['/0']['eligibility'],dict) and da['/1']['eligibility']=='Lexical'
    de={e['id']:e for e in du['metadata']['entries']}[da['/0']['entry']]
    assert all(de[k]=='Missing' for k in ['spelling','native_word_class','selmaho'])
    assert da['/0']['definition']=='Missing' and da['/0']['source_score']=='Missing'
    assert not [x for x in du['metadata']['contexts'] if x['assertion']==da['/0']['id']]
    assert not [x for x in du['metadata']['forms'] if x['entry']==de['id']]

    ex=[x for x in du['metadata']['extensions'] if x['owner']==da['/0']['id'] and 'opaque' in x['name']]
    assert len(ex)==2 and [x['evidence']['field_index'] for x in ex]==[4,5]
    assert [x['value'] for x in ex]==['{"n":123456789012345678901234567890}','null']
    assert len([x for x in du['metadata']['releases'][0]['diagnostics'] if x['occurrence']==da['/0']['occurrence'] and x['code']=='unknown_native_field'])==2
    # Exact malformed metadata no-output counterpart controls.
    for idx,suffix in enumerate([b'= true\n',b'bad =\n',b'bad = false junk\n',b'bad = 12 junk\n',b'bad = "ok" junk\n']):
        p,r,m=fixture('malformed-metadata-extra-'+str(idx),small,1,raw=rx,meta=mx+suffix)
        invoke(p,r,m,expected='jbotci_metadata')
    result={'verdict':'PASS','cli':BIN,'cli_sha256':sha(Path(BIN).read_bytes()),'cases':RESULTS}
    (ROOT/'results.json').write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps({'verdict':'PASS','invocations':len(RESULTS),'results':str(ROOT/'results.json')}))

if __name__=='__main__':main()
