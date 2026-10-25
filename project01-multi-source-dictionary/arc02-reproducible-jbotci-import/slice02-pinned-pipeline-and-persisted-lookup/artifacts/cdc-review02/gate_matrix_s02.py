from pathlib import Path
import json,subprocess,sys,time
r=Path(__file__).parent
src='/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc02/vlajmaji'
make=['make','--no-print-directory','CARGO=/tmp/vlajmaji-p01-a02-cargo','RUST_TOOLCHAIN=1.98.0']
cargo=['/tmp/vlajmaji-p01-a02-cargo','+1.98.0','test','--locked','--offline']
commands=[('s02-make-format02',make+['format']),('s02-make-lint02',make+['lint']),('s02-make-check02',make+['check']),('s02-workspace-allfeatures02',make+['test-all-features']),('s02-search-nodefault02',make+['test-no-default']),('s02-search-allfeatures02',cargo+['-p','vlajmaji-search','--all-features']),('s02-umbrella-nodefault02',cargo+['-p','vlajmaji','--no-default-features']),('s02-umbrella-allfeatures02',cargo+['-p','vlajmaji','--all-features'])]
for label,command in commands:
 print('starting '+label,flush=True)
 out=r/(label+'.driver.log');assert not out.exists()
 with out.open('xb') as f:
  p=subprocess.run([sys.executable,'-B',str(r/'run_gate.py'),label,src,*command],stdout=f,stderr=subprocess.STDOUT)
 print(json.dumps({'gate':label,'exit':p.returncode,'record':str(r/(label+'.json'))}),flush=True)
 if p.returncode:
  print(out.read_text()[-7000:],flush=True);raise SystemExit(p.returncode)
print('PASS all eight serialized independent repository/feature commands',flush=True)
