from pathlib import Path
import os,json,subprocess,hashlib,shutil
r=Path('/tmp/vlajmaji-p01-a03-cdc/s02');p=r/'header-controls03';p.mkdir();cli=r/'candidate-0d70-cli';shutil.copy2('/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc01/target/debug/vlajmaji',cli)
old=json.loads((r/'header-controls01.json').read_text());g=(r/'header-controls01/0-then-1/gismu.txt').read_bytes().splitlines(keepends=True)[2];rows=[]
env={k:v for k,v in os.environ.items() if k not in ['HOME','XDG_DATA_HOME']}
for counts in [(0,1),(1,0),(1,1),(0,0),(),(1,),(0,)]:
 name='then'.join(map(str,counts)) or 'absent';d=p/name;d.mkdir();raw=b''.join(f'{i:05d} Lojban synthetic\n'.encode() for i in counts)+g
 oldcase=r/'header-controls01'/('-then-'.join(map(str,counts)))
 if oldcase.exists():raw=(oldcase/'gismu.txt').read_bytes()
 values={'gismu':raw,'cmavo':b'','rafsi':b'','notice':b'Explicit synthetic source notice\n'};args=[str(cli),'data-pack','materialize','llg']
 for k,v in values.items():f=d/(k+'.txt');f.write_bytes(v);args.append(str(f))
 args.append(str(d/'output'))
 for k,v in values.items():args+=['--'+k+'-sha256',hashlib.sha256(v).hexdigest()]
 args+=['--base-locator','https://example.invalid/header-control'];q=subprocess.run(args,cwd=d,env=env,capture_output=True);item={'counts':counts,'argv':args,'exit':q.returncode,'stdout':q.stdout.decode(),'stderr':q.stderr.decode()};rows.append(item);(r/'header-controls03.json').write_text(json.dumps(rows,indent=2));assert q.returncode==0,item
 rep=json.loads(q.stdout)['report'];c=rep['components'][0];claims=c['header_claims'];assert [x['declared_body_count'] for x in claims]==list(counts);assert [x['count_matches'] for x in claims]==[i==1 for i in counts];assert [x['occurrence']['locator'] for x in claims]==[str(i+1) for i in range(len(counts))]
 agree=bool(counts) and len(set(counts))==1;assert c['declared_body_count']==(counts[0] if agree else None);assert c['declared_count_matches']==(counts[0]==1 if agree else None)
 assert c['input_lines']==len(counts)+1 and rep['occurrences']==len(counts)+1
 model=json.loads((d/'output/release.json').read_text());diags=[x for rel in model['metadata']['releases'] for x in rel['diagnostics']];m=[x for x in diags if x['code']=='header_count_mismatch'];assert [x['occurrence']['locator'] for x in m]==[str(i+1) for i,n in enumerate(counts) if n!=1],m
 assert sum(x['code']=='conflicting_header_counts' for x in diags)==(1 if len(set(counts))>1 else 0)
summary={'status':'PASS','cases':len(rows),'source':'0d70c9f2591574bc66d0420b8666ac5ed2351e2e','binary_sha256':hashlib.sha256(cli.read_bytes()).hexdigest(),'scope':'7 independent CLI materializations: differing/equal/no/single header claims, per-claim locators, summaries, raw population'};(r/'header-controls03-summary.json').write_text(json.dumps(summary,indent=2));print(json.dumps(summary))
