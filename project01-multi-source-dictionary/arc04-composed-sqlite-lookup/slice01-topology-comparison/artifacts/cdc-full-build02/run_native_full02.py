from pathlib import Path
import hashlib,json,subprocess,sys,time
root=Path('/tmp/vlajmaji-p01-a04-s01-cdc');out=root/'native-full02';out.mkdir()
base=Path('/tmp/vlajmaji-p01-a04-s01-cc/full-comparison02');script=root/'oracle/native_topology.py';assert hashlib.sha256(script.read_bytes()).hexdigest()=='23fe1d157715bbf274380920ead96efff7376e1f663679e9dcb5913292920de8'
rows=[]
for name,packs in [('jbotci','lensisku-jbotci'),('llg','llg'),('la','la-lojban'),('sut','sutysisku'),('unified','lensisku-jbotci,llg,la-lojban,sutysisku')]:
 argv=[sys.executable,'-B',str(script),str(base/name/'dictionary.sqlite3'),str(out/name),packs];started=time.monotonic()
 with (out/(name+'.stdout')).open('xb')as fo,(out/(name+'.stderr')).open('xb')as fe:p=subprocess.run(argv,stdout=fo,stderr=fe,timeout=300)
 rows.append(dict(store=name,argv=argv,exit=p.returncode,wall_seconds=time.monotonic()-started));(out/'attempts.json').write_text(json.dumps(rows,indent=2)+'\n')
 assert p.returncode==0,rows[-1]
 v=json.loads((out/name/'native-sql01.json').read_text());assert v['verdict']=='PASS'and v['db_pre_sha256']==v['db_post_sha256'];print(json.dumps(dict(store=name,status='PASS',occurrences=v['occurrences'],native_fields=v['native_fields'],checks=v['total_checks'])),flush=True)
(out/'summary.json').write_text(json.dumps(dict(status='PASS',source='4b4158c8a8a489ead639b73706b01061de17cfe1',oracle_sha256=hashlib.sha256(script.read_bytes()).hexdigest(),stores=5,attempts=rows),indent=2)+'\n')
