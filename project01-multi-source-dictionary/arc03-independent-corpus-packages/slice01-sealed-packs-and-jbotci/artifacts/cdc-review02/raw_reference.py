from pathlib import Path
import json,hashlib,collections
ROOT=Path('/tmp/vlajmaji-p01-a03-cdc/s01')
LEAF=Path('/tmp/vlajmaji-p01-a02-cdc/s02-full-cache02/3d9fd41d9ed32de26311ee27ec86b6b165985aa1-12e826751c3e50c10dd8df1adafb5f9eb32cdf542b22c4244f284b59d3824dde-fe101eead51e0be1b885f3ee1b4555136e333e839baa250ddbd7b84c9af5fc98')
CLASSES={'bu-letteral','cmavo','cmavo-compound','cmevla','experimental cmavo','experimental gismu',"fu'ivla",'gismu','lujvo','nalvla','obsolete cmavo','obsolete cmevla',"obsolete fu'ivla",'obsolete zei-lujvo','phrase','zei-lujvo'}
def sha(b): return hashlib.sha256(b).hexdigest()
def unique(pairs):
 d={}
 for k,v in pairs:
  assert k not in d, ('duplicate JSON key',k)
  d[k]=v
 return d
def load_source():
 b=(LEAF/'dictionary-en.json').read_bytes();text=b.decode();rows=json.loads(text,object_pairs_hook=unique)
 dec=json.JSONDecoder(object_pairs_hook=unique);i=1;all_fields=[];raw_rows=[]
 def ws(i):
  while i<len(text) and text[i].isspace():i+=1
  return i
 for row in rows:
  i=ws(i);start=i;decoded,end=dec.raw_decode(text,i);assert decoded==row;raw_rows.append(text[start:end]);i=end
  body=raw_rows[-1];p=1;fs=[]
  while True:
   while body[p].isspace():p+=1
   if body[p]=='}':break
   key,p=dec.raw_decode(body,p)
   while body[p].isspace():p+=1
   assert body[p]==':';p+=1
   while body[p].isspace():p+=1
   begin=p;value,p=dec.raw_decode(body,p);fs.append((key,body[begin:p],value))
   while body[p].isspace():p+=1
   if body[p]==',':p+=1
   else:assert body[p]=='}'
  all_fields.append(fs);i=ws(i)
  if text[i]==',':i+=1
  else:assert text[i]==']'
 assert text[i:].strip()==']'
 return b,rows,all_fields,raw_rows
def main():
 b,rows,fields,lexemes=load_source();counts=collections.Counter();classes=collections.Counter();words=collections.Counter();eligible=collections.Counter();ids=collections.Counter();excluded=[];optional=collections.defaultdict(collections.Counter)
 for i,(r,fs) in enumerate(zip(rows,fields)):
  counts.update(k for k,_,_ in fs);classes[r['word_type']]+=1;words[r['word']]+=1;ids[str(r['definition_id'])]+=1
  yes=r['word_type'] in CLASSES and bool(r['word']) and bool(r['definition'].strip())
  if yes:eligible[r['word']]+=1
  else:excluded.append({'row':i,'word':r['word'],'class':r['word_type'],'native_id':r['definition_id'],'empty_definition':not bool(r['definition'].strip())})
  for k in ('selmaho','notes','jargon','gloss_keywords','place_keywords','rafsi','experimental_rafsi','etymology'):
   optional[k]['missing' if k not in r else 'null' if r[k] is None else 'empty' if not r[k] else 'nonempty']+=1
 out={'raw_sha256':sha(b),'rows':len(rows),'fields':sum(counts.values()),'field_counts':dict(counts),'classes':dict(classes),'all_words':len(words),'eligible_occurrences':sum(eligible.values()),'eligible_words':len(eligible),'distinct_native_ids':len(ids),'repeated_native_ids':{k:v for k,v in ids.items() if v>1},'duplicate_full_rows':len(lexemes)-len(set(lexemes)),'excluded':excluded,'excluded_only_words':sorted(set(words)-set(eligible)),'optional_populations':dict(optional),'witnesses':{w:[{'row':i,'native_id':r['definition_id'],'definition':r['definition']} for i,r in enumerate(rows) if r['word']==w] for w in ('farvi','ma','wiki')},'contexts':sum(len(r.get('gloss_keywords',[])) for r in rows)+counts['notes']+counts['jargon'],'gloss_objects':sum(len(r.get('gloss_keywords',[])) for r in rows),'forms':sum(len(r.get(k,'').split()) for r in rows for k in ('rafsi','experimental_rafsi')),'etymologies':counts['etymology'],'extensions':counts['gloss_keywords']+sum(len(r.get('gloss_keywords',[])) for r in rows)+counts['place_keywords']+1}
 (ROOT/'raw-census.json').write_text(json.dumps(out,indent=2,ensure_ascii=False)+'\n')
 assert (out['rows'],out['fields'],out['eligible_occurrences'],out['eligible_words'],out['extensions'])==(33526,278254,33516,31248,73226)
 print(json.dumps({k:v for k,v in out.items() if k not in ('witnesses','optional_populations','excluded')},indent=2))
if __name__=='__main__':main()
