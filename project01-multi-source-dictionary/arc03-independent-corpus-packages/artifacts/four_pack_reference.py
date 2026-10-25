from pathlib import Path
import json,hashlib,importlib.util
r=Path('/tmp/vlajmaji-p01-a03-cdc/readiness')
def module(n,p):
 s=importlib.util.spec_from_file_location(n,p);m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
j=module('jbotci_reference','/tmp/vlajmaji-p01-a03-cdc/s01/raw_reference.py');l=module('distribution_reference',str(r/'distribution_census.py'))
jb,jrows,jfields,_=j.load_source();lb,lrows,lfields,_=l.la_source();sb=(l.p/'sutysisku-parsed-en.json').read_bytes();srows=l.fields(sb.decode())
assert hashlib.sha256(jb).hexdigest()=='12e826751c3e50c10dd8df1adafb5f9eb32cdf542b22c4244f284b59d3824dde';assert hashlib.sha256(lb).hexdigest()=='861e71f66173b383298f6b1823e3ee1ed76db4b9c3fb97e7177de1fd7b667d35';assert hashlib.sha256(sb).hexdigest()=='8e80f6626d2e68a17025d52c3f0cf5ea8c32cbfeeb0c2ff1417fcba539bd043d'
def eligible(x):return bool(x['word']) and x['word_type'] in j.CLASSES and bool(x['definition'].strip())
sets={'jbotci':{x['word'] for x in jrows if eligible(x)},'la-lojban':{x['word'] for x in lrows if eligible(x)},'sutysisku':{w for w,_,x,_ in srows if w and x['t'] in j.CLASSES and x['d'].strip()}}
llg=set();llg_rows=[]
for name,a,b,d in [('gismu',1,6,62),('cmavo',0,10,61)]:
 raw=(r/('llg-current-'+name+'.txt')).read_bytes();lines=raw.decode().splitlines()
 for ordinal,line in enumerate(lines[1:],2):
  word=line[a:b].strip();assert word and line[d:(158 if name=='gismu' else None)].strip();llg.add(word);llg_rows.append((name,ordinal,word))
sets['llg']=llg;allwords=set().union(*sets.values());la_jb=sets['jbotci']|sets['la-lojban'];jb_users={x['user']['username'] for x in jrows};la_users={x['user']['username'] for x in lrows};assert la_users<=jb_users
w=[]
for name,rows,fs in [('jbotci',jrows,jfields),('la-lojban',lrows,lfields)]:
 for i,(x,f) in enumerate(zip(rows,fs)):
  if x['word']=='farvi' and eligible(x):w.append({'pack':name,'locator':f'/{i}','native_assertion_id':next(y[1] for y in f if y[0]=='definition_id')})
for name,i,word in llg_rows:
 if word=='farvi':w.append({'pack':'llg','component':name,'locator':str(i)})
for i,(word,_,x,_) in enumerate(srows):
 if word=='farvi' and x['d'].strip() and x['t'] in j.CLASSES:w.append({'pack':'sutysisku','locator':f'/{i}','native_entry_id':word})
def row_stats(rows,fields,wrapper_extensions):
 eligible_rows=[x for x in rows if eligible(x)]
 count=lambda k:sum(k in x for x in rows)
 return {'assertions':len(rows),'entries':len(rows),'lexical_assertions':len(eligible_rows),'excluded_assertions':len(rows)-len(eligible_rows),'native_fields_with_sut_keys':sum(len(f) for f in fields),'contexts':sum(len(x.get('gloss_keywords',[])) for x in rows)+count('notes')+count('jargon'),'forms':sum(len(x.get(k,'').split()) for x in rows for k in ['rafsi','experimental_rafsi']),'etymologies':count('etymology'),'extensions':count('gloss_keywords')+sum(len(x.get('gloss_keywords',[])) for x in rows)+count('place_keywords')+wrapper_extensions}
stats={'jbotci':row_stats(jrows,jfields,1),'la-lojban':row_stats(lrows,lfields,0)}
slex=sum(bool(w) and x['t'] in j.CLASSES and bool(x['d'].strip()) for w,_,x,_ in srows)
stats['sutysisku']={'assertions':len(srows),'entries':len(srows),'lexical_assertions':slex,'excluded_assertions':len(srows)-slex,'native_fields_with_sut_keys':len(srows)+sum(len(l.fields(raw)) for _,raw,_,_ in srows),'contexts':sum(len(x.get('g',[]))+('n' in x) for _,_,x,_ in srows),'forms':sum(len(x.get('r',[])) for _,_,x,_ in srows),'etymologies':0,'extensions':sum(('g' in x)+('b' in x) for _,_,x,_ in srows)}
lg=(r/'llg-current-gismu.txt').read_text().splitlines();lc=(r/'llg-current-cmavo.txt').read_text().splitlines();lr=(r/'llg-current-rafsi.txt').read_text().splitlines();gstarts=[1,7,11,15,20,41,62,159,161,165]
stats['llg']={'assertions':len(lg)+len(lc)+len(lr),'entries':len(llg)+2,'lexical_assertions':len(llg_rows),'excluded_assertions':len(lg)+len(lc)+len(lr)-len(llg_rows),'native_fields_with_sut_keys':2+sum(sum(len(x)>start for start in gstarts) for x in lg[1:])+4*len(lc[1:])+3*len(lr),'contexts':sum(bool(x[20:40].strip())+bool(x[41:61].strip())+bool(x[165:].strip()) for x in lg[1:])+sum(bool(x[20:61].strip()) for x in lc[1:])+sum(len(x.split(maxsplit=2))==3 and bool(x.split(maxsplit=2)[2].strip()) for x in lr),'forms':sum(bool(x[a:b].strip()) for x in lg[1:] for a,b in [(7,10),(11,14),(15,19)])+len(lr),'etymologies':0,'extensions':sum((len(x)>159)+(len(x)>161) for x in lg[1:])}
totals={k:sum(x[k] for x in stats.values()) for k in stats['jbotci']};totals.update({'sources':3,'distributors':2,'collections':4,'components':6,'releases':6,'captures':8})
out={'kind':'source-derived policy reference, not product acceptance','raw_pins':{'jbotci':hashlib.sha256(jb).hexdigest(),'la-lojban':hashlib.sha256(lb).hexdigest(),'sutysisku':hashlib.sha256(sb).hexdigest()},'eligible_words_per_pack':{k:len(v) for k,v in sets.items()},'eligible_words_la_plus_jbotci':len(la_jb),'eligible_words_four_pack_union':len(allwords),'eligible_word_union_sha256':hashlib.sha256(json.dumps(sorted(allwords),ensure_ascii=False,separators=(',',':')).encode()).hexdigest(),'jbotci_contributors':len(jb_users),'la_contributors':len(la_users),'shared_contributors':len(jb_users&la_users),'la_contributors_subset':la_users<=jb_users,'farvi_eligible_occurrences':w,'farvi_total_occurrences':len(w),'farvi_source_groups':3,'per_pack_source_policy_counts':stats,'four_pack_expected':totals}
(r/'four-pack-reference02.json').write_text(json.dumps(out,indent=2)+'\n');print(json.dumps(out,indent=2))
