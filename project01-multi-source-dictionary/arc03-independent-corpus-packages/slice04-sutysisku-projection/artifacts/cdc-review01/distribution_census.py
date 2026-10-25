from pathlib import Path
from collections import Counter,defaultdict
import json,hashlib
p=Path('/Users/oubiwann/Library/Application Support/ChatGPT-Profiles/personal/codex-home/.chatgpt-projects/g-p-6a8c72d334048191887d4e00472f8430/relojban-poetics/experiments/E02-resource-discovery/raw/lexical');outdir=Path('/tmp/vlajmaji-p01-a03-cdc/readiness')
def unique(pairs):
 out={}
 for k,v in pairs:
  assert k not in out,('duplicate source key',k)
  out[k]=v
 return out
dec=json.JSONDecoder(object_pairs_hook=unique)
def ws(t,i):
 while i<len(t) and t[i].isspace():i+=1
 return i
def fields(t):
 i=ws(t,0);assert t[i]=='{';i+=1;out=[]
 while True:
  i=ws(t,i)
  if t[i]=='}':assert not t[i+1:].strip();return out
  begin=i;k,i=dec.raw_decode(t,i);keylex=t[begin:i];assert isinstance(k,str);i=ws(t,i);assert t[i]==':';i=ws(t,i+1);begin=i;v,i=dec.raw_decode(t,i);out.append((k,t[begin:i],v,keylex));i=ws(t,i)
  if t[i]==',':i+=1
  else:assert t[i]=='}'
def la_source():
 b=(p/'la-lojban-en.json').read_bytes();t=b.decode();rows=json.loads(t,object_pairs_hook=unique);fs=[];raws=[];i=ws(t,0);assert t[i]=='[';i+=1
 for row in rows:
  i=ws(t,i);start=i;v,i=dec.raw_decode(t,i);assert v==row;raws.append(t[start:i]);fs.append(fields(raws[-1]));i=ws(t,i)
  if t[i]==',':i+=1
  else:assert t[i]==']'
 assert t[i:].strip()==']';return b,rows,fs,raws
classes={'bu-letteral','cmavo','cmavo-compound','cmevla','experimental cmavo','experimental gismu',"fu'ivla",'gismu','lujvo','nalvla','obsolete cmavo','obsolete cmevla',"obsolete fu'ivla",'obsolete zei-lujvo','phrase','zei-lujvo'}
def main():
 b,rows,fs,raws=la_source();counts=Counter(k for f in fs for k,_,_,_ in f);eligible=[x for x in rows if x['word_type'] in classes and x['word'] and x['definition'].strip()];gloss=sum(len(x.get('gloss_keywords',[])) for x in rows);users=Counter(x.get('user',{}).get('username') for x in rows)
 out={'raw_bytes':len(b),'raw_sha256':hashlib.sha256(b).hexdigest(),'rows':len(rows),'native_fields':sum(counts.values()),'field_counts':dict(counts),'whole_source_nested_keys_unique':True,'identical_complete_rows':len(raws)-len(set(raws)),'distinct_words':len({x['word'] for x in rows}),'distinct_definition_ids':len({x['definition_id'] for x in rows}),'lexical':len(eligible),'excluded':len(rows)-len(eligible),'eligible_spellings':len({x['word'] for x in eligible}),'classes':dict(Counter(x['word_type'] for x in rows)),'gloss_items':gloss,'contexts':gloss+counts['notes']+counts['jargon'],'forms':sum(len(x.get('rafsi','').split())+len(x.get('experimental_rafsi','').split()) for x in rows),'etymologies':counts['etymology'],'extensions_native':counts['gloss_keywords']+gloss+counts['place_keywords'],'contributors':len(users),'null_fields':dict(Counter(k for x in rows for k,v in x.items() if v is None)),'user_fields':dict(Counter(k for x in rows for k in x['user']))}
 (outdir/'la-native-census01.json').write_text(json.dumps(out,indent=2)+'\n');print('LA',json.dumps(out,indent=2))
 b=(p/'sutysisku-parsed-en.json').read_bytes();root=fields(b.decode());counts=Counter();tc=Counter();gloss=0;forms=0;eligible=[];excluded=[];optional=defaultdict(Counter)
 for ordinal,(word,raw,row,keylex) in enumerate(root):
  for k,lex,v,_ in fields(raw):counts[k]+=1;optional[k][type(v).__name__]+=1
  tc[row['t']]+=1;gloss+=len(row.get('g',[]));forms+=len(row.get('r',[]))
  yes=bool(word) and row['t'] in classes and bool(row['d'].strip())
  (eligible if yes else excluded).append((ordinal,word))
 out={'raw_bytes':len(b),'raw_sha256':hashlib.sha256(b).hexdigest(),'root_key_occurrences':len(root),'unique_root_spellings':len({x[0] for x in root}),'native_child_fields':sum(counts.values()),'field_counts':dict(counts),'field_types':{k:dict(v) for k,v in optional.items()},'whole_source_nested_keys_unique':True,'classes':dict(tc),'lexical':len(eligible),'eligible_spellings':len({x[1] for x in eligible}),'excluded':excluded,'gloss_items':gloss,'contexts':gloss+counts['n'],'reported_forms':forms,'form_lengths':dict(Counter(len(v) for _,_,row,_ in root for v in row.get('r',[]))),'opaque_b_arrays':counts['b'],'opaque_b_values':sum(len(row.get('b',[])) for _,_,row,_ in root)}
 (outdir/'sut-native-census01.json').write_text(json.dumps(out,indent=2)+'\n');print('SUT',json.dumps(out,indent=2))
if __name__=='__main__':main()
