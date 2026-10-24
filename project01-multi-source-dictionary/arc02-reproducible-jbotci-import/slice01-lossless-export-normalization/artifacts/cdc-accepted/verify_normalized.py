#!/usr/bin/env python3
"""CDC scratch whole-population oracle; expectations derive from original bytes."""
import collections
import hashlib
import json
import sys
from pathlib import Path

checks = collections.Counter()
def require(condition, category, detail):
    checks[category] += 1
    if not condition:
        raise AssertionError(f'{category}: {detail}')

def digest(b): return hashlib.sha256(b).hexdigest()
def val(x): return {'Value': x}
def presence(row, key, transform=lambda x:x):
    if key not in row: return 'Missing'
    return 'Null' if row[key] is None else val(transform(row[key]))

def skip(s, i):
    while i < len(s) and s[i] in ' \t\r\n': i += 1
    return i

def raw_array(s):
    """Return decoded values and exact element lexemes; do not reserialize values."""
    d=json.JSONDecoder();i=skip(s,0)
    if s[i]!='[': raise ValueError('expected array')
    i=skip(s,i+1);out=[]
    while s[i]!=']':
        start=i;v,i=d.raw_decode(s,i);out.append((v,s[start:i]));i=skip(s,i)
        if s[i]==',':i=skip(s,i+1)
        elif s[i]!=']':raise ValueError('expected comma or array end')
    if skip(s,i+1)!=len(s):raise ValueError('trailing data')
    return out

def raw_object(s):
    d=json.JSONDecoder();i=skip(s,0)
    if s[i]!='{':raise ValueError('expected object')
    i=skip(s,i+1);out=[]
    while s[i]!='}':
        name,i=d.raw_decode(s,i);i=skip(s,i)
        if s[i]!=':':raise ValueError('expected colon')
        i=skip(s,i+1);start=i;v,i=d.raw_decode(s,i)
        out.append((name,s[start:i],v));i=skip(s,i)
        if s[i]==',':i=skip(s,i+1)
        elif s[i]!='}':raise ValueError('expected comma or object end')
    if skip(s,i+1)!=len(s):raise ValueError('trailing object data')
    return out

def main():
    rawpath,metapath,releasepath,reportpath=sys.argv[1:5]
    rb=Path(rawpath).read_bytes();mb=Path(metapath).read_bytes();rbsha=digest(rb)
    source=raw_array(rb.decode());out=json.loads(Path(releasepath).read_bytes())
    report=json.loads(Path(reportpath).read_bytes());report=report.get('report',report)
    meta=out['metadata'];reg=out['registry'];captures=out['captures']
    require(out['version']==1,'envelope','document version')
    require(out['legacy_archive'] is None,'envelope','explicit export is not old archive')
    require(len(captures)==2,'envelope','raw and supplier metadata captures')
    byns={c['identity']['namespace']:c for c in captures}
    rawcap=byns['captured:lensisku:jbotci:en'];metacap=byns['captured:lensisku:jbotci:en:metadata']
    for cap,b in [(rawcap,rb),(metacap,mb)]:
        require(bytes(cap['bytes'])==b,'raw_capture','exact bytes')
        require(cap['identity']['sha256']==digest(b),'raw_capture','full SHA')
        require(cap['metadata']['captured_at']=='Missing','provenance','unknown capture time')
        require(cap['metadata']['time_precision']=='Unknown','provenance','no fabricated time precision')
        require(cap['metadata']['revision']==val(report['revision']),'provenance','exact distributor revision')
    require(metacap['occurrences']==[],'raw_capture','metadata has zero lexical rows')
    require(len(rawcap['occurrences'])==len(source),'raw_population','every input row')
    require(len(meta['entries'])==len(source),'normalized_population','per-occurrence entries')
    require(len(meta['assertions'])==len(source),'normalized_population','every row has assertion')
    require(len(reg['occurrences'])==len(source),'normalized_population','every row attributed')
    require(len(reg['sources'])==1 and reg['sources'][0]['id']=='lensisku','provenance','one asserting source')
    require(reg['sources'][0]['abbreviation']=='LSK','provenance','assigned source abbreviation')
    require(reg['sources'][0]['abbreviation_kind']=='ProjectAssigned','provenance','abbreviation status')
    require([d['id'] for d in reg['distributors']]==['jbotci'],'provenance','separate distributor')
    require(len(reg['snapshots'])==2,'provenance','both captures bound')
    for b in reg['snapshots']:
        require(b['source']=='lensisku' and b['distributor']=='jbotci','provenance','capture binding')
    entries={x['id']:x for x in meta['entries']}
    assertions={x['occurrence']['locator']:x for x in meta['assertions']}
    attrs={x['occurrence']['locator']:x for x in reg['occurrences']}
    contrib={x['id']:x for x in reg['contributors']}
    expected_contributors={row['user']['username'] for row,_ in source}
    require(len(contrib)==len(expected_contributors),'contributor_population','one stable source-scoped identity per exact username')
    contexts=collections.defaultdict(list);forms=collections.defaultdict(list);etym=collections.defaultdict(list)
    ext=collections.defaultdict(list)
    for x in meta['contexts']:contexts[x['assertion']].append(x)
    for x in meta['forms']:forms[x['entry']].append(x)
    for x in meta['etymologies']:etym[x['assertion']].append(x)
    for x in meta['extensions']:ext[x['owner']].append(x)
    for absent in ['frames','relations','owned_revisions','owned_heads','editorial_senses','computations','annotations']:
        require(meta[absent]==[],'noninference',absent)
    require(len(meta['releases'])==1,'release','one normalized release')
    release=meta['releases'][0]
    require(all(release['accounting'][k]==v for k,v in {'input_occurrences':len(source),'normalized_occurrences':len(source),'query_excluded_occurrences':10}.items()) and bool(release['accounting']['unit'].strip()),'release','balanced labelled accounting')
    require(release['distributor']=='jbotci' and release['distributor_revision']==val(report['revision']),'release','distributor revision')
    require(release['upstream_revision']=='Missing','release','no invented upstream revision')
    require(release['redistribution']=='Held','terms','public redistribution held')
    require(release['normalizer']=='vlajmaji-jbotci-export' and release['normalizer_version']=='1','release','normalizer version')
    require(release['schema_version']==1 and release['availability']=='Available' and release['selected'],'release','availability/version')
    require(set(release['assertions'])==set(x['id'] for x in meta['assertions']),'release','all assertions listed')
    require(len(release['assertions'])==len(source),'release','no duplicate membership')
    require(any('Apache' in str(n) for n in meta['code_notices']),'terms','code notice separate')
    require(any('supplier' in str(n).lower() and 'public' in str(n).lower() for n in release['content_notices']),'terms','supplier content claim scoped')
    classes=collections.Counter();fieldcounts=collections.Counter();eligible=set();allwords=set();exclusions=0
    formcount=glosscount=notecount=domaincount=etymcount=0
    for i,((row,lexeme),gotraw) in enumerate(zip(source,rawcap['occurrences'])):
        loc=f'/{i}';rf=raw_object(lexeme)
        require(gotraw['locator']==loc,'raw_row','source ordinal')
        require(gotraw['raw_line'] is None,'raw_row','JSON row has no fabricated line')
        expected=[{'name':k,'encoding':'Json','raw_value':v} for k,v,_ in rf]
        require(gotraw['fields']==expected,'native_fields',loc)
        names={k:j for j,(k,_,_) in enumerate(rf)};lex={k:v for k,v,_ in rf}
        fieldcounts.update(k for k,_,_ in rf);classes[row['word_type']]+=1;allwords.add(row['word'])
        a=assertions[loc];e=entries[a['entry']];at=attrs[loc]
        require(a['occurrence']=={'capture':rawcap['identity'],'locator':loc},'occurrence',loc)
        require(e['source']=='lensisku' and e['spelling']==val(row['word']),'entry',loc)
        require(e['language']==val('jbo') and e['script']=='Missing' and e['mapped_pos']=='Missing','entry',loc+' language/noninference')
        require(e['native_word_class']==val(row['word_type']) and e['selmaho']==presence(row,'selmaho'),'entry',loc+' native class/selmaho')
        require(a['definition']==val({'text':row['definition'],'language':val('en'),'markup':'Missing'}),'assertion',loc+' exact definition')
        require(a['source_score']==val(lex['score']),'assertion',loc+' exact score lexeme')
        require(a['language']==val('en') and a['native_sense_id']=='Missing' and a['timestamps']==[],'assertion',loc+' no invented native fields')
        yes=row['word_type']!='wiki' and bool(row['definition'].strip()) and bool(row['word'])
        if yes:
            require(a['eligibility']=='Lexical','eligibility',loc);eligible.add(row['word'])
        else:
            exclusions+=1;require(bool(a['eligibility'].get('Excluded',{}).get('reason')),'eligibility',loc)
            require(any(d['occurrence']==a['occurrence'] for d in release['diagnostics']),'eligibility',loc+' diagnostic')
        require(at['native_entry_id'] is None and at['native_assertion_id']==lex['definition_id'],'native_identity',loc)
        require(at['occurrence']=={'snapshot':{'namespace':rawcap['identity']['namespace'],'sha256':rbsha},'locator':loc},'native_identity',loc+' full tuple')
        require(contrib[at['contributor']]['display_name']==row['user']['username'],'contributor',loc)
        cs=contexts[a['id']]
        for kind,key in [('Note','notes'),('Domain','jargon')]:
            got=[c for c in cs if c['kind']==kind]
            require(len(got)==int(key in row),'context_population',loc+' '+key)
            for c in got:
                require(c['value']['Value']['text']==row[key],'context_value',loc+' '+key)
                require(c['evidence']['occurrence']==a['occurrence'] and c['evidence']['field_index']==names[key],'context_evidence',loc+' '+key)
            if key=='notes':notecount+=len(got)
            else:domaincount+=len(got)
        gs=[c for c in cs if c['kind']=='Gloss'];graw=raw_array(lex.get('gloss_keywords','[]'))
        require(len(gs)==len(graw),'gloss_population',loc)
        for g,(item,itemraw) in zip(gs,graw):
            require(g['value']['Value']['text']==item['word'],'gloss_value',loc)
            require(g['evidence']['occurrence']==a['occurrence'] and g['evidence']['field_index']==names['gloss_keywords'],'gloss_evidence',loc)
            require(any(x['value']==itemraw for x in ext[g['id']]),'gloss_native_meaning',loc+' raw object/null/missing')
        glosscount+=len(gs)
        if 'place_keywords' in row:
            require(any(x['name']=='lensisku.place_keywords' and x['value']==lex['place_keywords'] for x in ext[a['id']]),'place_keywords',loc)
        expectedforms=[]
        for key,_,_ in rf:
            if key in ['rafsi','experimental_rafsi']:
                expectedforms.extend((s,key) for s in row[key].split())
        gotforms=forms[e['id']]
        require(len(gotforms)==len(expectedforms),'form_population',loc)
        for key in ['rafsi','experimental_rafsi']:
            group=[f for f in gotforms if f['standing']==val(key)]
            expectedgroup=[spelling for spelling,k in expectedforms if k==key]
            require([f['spelling'] for f in group]==expectedgroup,'form_value',loc+' '+key+' ordered tokens')
            for f in group:
                require(f['kind']=='SourceReportedForm','form_value',loc+' '+key+' source role')
                require(f['evidence']['occurrence']==a['occurrence'] and f['evidence']['field_index']==names[key],'form_evidence',loc)
                require(f['computation'] is None and f['components']==[],'form_noninference',loc)
        formcount+=len(gotforms)
        es=etym[a['id']]
        require(len(es)==int('etymology' in row),'etymology_population',loc)
        for e2 in es:
            require(e2['statement']['Value']['text']==row['etymology'] and e2['components']==[],'etymology_value',loc)
            require(e2['evidence']['occurrence']==a['occurrence'] and e2['evidence']['field_index']==names['etymology'],'etymology_evidence',loc)
        etymcount+=len(es)
    expected_report={'version':1,'revision':'3d9fd41d9ed32de26311ee27ec86b6b165985aa1','input_rows':len(source),'native_fields':sum(fieldcounts.values()),'normalized_assertions':len(source),'lexical_occurrences':len(source)-exclusions,'excluded_occurrences':exclusions,'eligible_spellings':len(eligible),'all_spellings':len(allwords),'contributors':len(expected_contributors),'supplier_definition_count':33526,'supplier_entry_count':31248,'raw_bytes':len(rb),'metadata_bytes':len(mb),'raw_sha256':digest(rb),'metadata_sha256':digest(mb),'word_types':dict(classes),'field_counts':dict(fieldcounts),'definition_count_matches':True,'entry_count_matches':True}
    for k,v in expected_report.items():require(report[k]==v,'report',k)
    result={'verdict':'PASS','raw_sha256':digest(rb),'metadata_sha256':digest(mb),'release_sha256':digest(Path(releasepath).read_bytes()),'rows':len(source),'native_fields':sum(fieldcounts.values()),'normalized_assertions':len(meta['assertions']),'eligible_occurrences':len(source)-exclusions,'exact_eligible_spellings':len(eligible),'contexts':{'gloss':glosscount,'note':notecount,'domain':domaincount},'forms':formcount,'etymologies':etymcount,'check_categories':dict(checks),'total_checks':sum(checks.values())}
    print(json.dumps(result,indent=2))

if __name__=='__main__':main()
