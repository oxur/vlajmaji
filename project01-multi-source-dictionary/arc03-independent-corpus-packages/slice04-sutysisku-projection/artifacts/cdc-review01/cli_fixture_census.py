from pathlib import Path
from collections import Counter
import re,json,hashlib,importlib.util
r=Path('/tmp/vlajmaji-p01-a03-cdc/s04');s=Path('/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc03/vlajmaji/crates/cli/src/main.rs');body=s.read_bytes();text=body.decode();start=text.index('pack: FixturePack::Sutysisku { output }');raw=re.search(r'let raw\s*=\s*br#"(.*?)"#;',text[start:],re.S).group(1).encode();spec=importlib.util.spec_from_file_location('native_source','/tmp/vlajmaji-p01-a03-cdc/readiness/distribution_census.py');src=importlib.util.module_from_spec(spec);spec.loader.exec_module(src);rows=src.fields(raw.decode());counts=Counter();types=Counter();eligible=[];contexts=forms=extensions=0
for i,(word,lex,row,keylex) in enumerate(rows):
 counts.update(k for k,_,_,_ in src.fields(lex));types[row['t']]+=1
 if word and row['t'] in src.classes and row['d'].strip():eligible.append(word)
 contexts+=len(row.get('g',[])) if row.get('g') is not None else 0
 contexts+=int('n' in row and (row['n'] is None or isinstance(row['n'],str)))
 forms+=len(row.get('r',[])) if row.get('r') is not None else 0
 extensions+=int('g' in row)+int('b' in row)
(r/'cli-sut-fixture.json').write_bytes(raw)
out={'kind':'frozen CLI literal census BEFORE independent expected tuple or fixture execution','source_sha256':hashlib.sha256(body).hexdigest(),'raw_sha256':hashlib.sha256(raw).hexdigest(),'raw_bytes':len(raw),'root_keys':len(rows),'native_child_fields':sum(counts.values()),'native_fields_with_keys':len(rows)+sum(counts.values()),'lexical_occurrences':len(eligible),'excluded_occurrences':len(rows)-len(eligible),'eligible_spellings':len(set(eligible)),'all_spellings':len({x[0] for x in rows}),'contexts':contexts,'forms':forms,'extensions':extensions,'field_counts':dict(counts),'word_types':dict(types)};(r/'cli-fixture-source-census01.json').write_text(json.dumps(out,indent=2)+'\n');print(json.dumps(out,indent=2))
