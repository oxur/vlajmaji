import json,subprocess,sys
from pathlib import Path
r=Path('/tmp/vlajmaji-p01-a02-cdc');source='/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc02/vlajmaji';cli='/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc01/target/debug/vlajmaji'
for label,helper in [('s02-boundary02','s02_boundary_controls02.py'),('s02-alias02','s02_alias_controls02.py'),('s02-failure-paths02','s02_failure_paths02.py'),('s02-pipeline02','pipeline_controls02.py')]:
 out=r/(label+'-cases');argv=[sys.executable,'-B',str(r/'run_process.py'),label,sys.executable,'-B',str(r/helper),source,cli,str(out)]
 result=subprocess.run(argv);assert result.returncode==0,(label,result.returncode)
 resultfile=out/'result.json'
 if resultfile.exists():
  body=json.loads(resultfile.read_text());assert body.get('disagreements',[])==[],(label,body['disagreements'])
print('PASS repaired targeted13 and broad pipeline controls')
