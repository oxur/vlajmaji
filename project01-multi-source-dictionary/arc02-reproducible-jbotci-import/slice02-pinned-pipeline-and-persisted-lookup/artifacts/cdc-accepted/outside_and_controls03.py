import hashlib,json,os,subprocess,sys
from pathlib import Path
r=Path('/tmp/vlajmaji-p01-a02-cdc');src=Path('/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc02/vlajmaji');target=Path('/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc01/target');cli=target/'debug/vlajmaji';archive=r/'s02-clean-source-candidate03';store=r/'s02-full-store03';canonical=r/'s02-full-canonical02.json';leaves=list((r/'s02-full-cache02').iterdir());assert len(leaves)==1;raw=leaves[0]/'dictionary-en.json';meta=leaves[0]/'dictionary-en.metadata.toml';db=store/'dictionary.sqlite3';sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();before=sha(db)
def invoke(label,argv,extra=None):
 env=os.environ.copy();env.update(extra or {});p=subprocess.run([sys.executable,'-B',str(r/'run_process.py'),label,*map(str,argv)],env=env);assert p.returncode==0,(label,p.returncode)
invoke('s02-sql-counterexamples03',[sys.executable,'-B',r/'sql_oracle_counterexamples.py',raw,meta,db,r/'s02-sql-negative03'])
invoke('s02-full-cli-query03',[cli,'dictionary-query',store,'--request',json.dumps({'operation':'exact','spelling':'farvi'})])
a=json.loads((r/'s02-full-cli-query03.stdout').read_bytes());b=json.loads((r/'s02-new-finite23-03-results/farvi-default.json').read_bytes())['ok'];assert a==b and sha(db)==before;(r/'s02-full-cli-comparison03.json').write_text(json.dumps({'verdict':'PASS','complete_cli_vs_public_value_equality':True,'cli_sha256':sha(cli),'database_before_after_sha256':before,'query_output_bytes':(r/'s02-full-cli-query03.stdout').stat().st_size,'query_output_sha256':sha(r/'s02-full-cli-query03.stdout')},indent=2)+'\n')
# Real Cargo fallback executes archive code at an exact clean archive, offline and explicit target.
make=['make','--no-print-directory','-C',archive,'CARGO=/tmp/vlajmaji-p01-a02-cargo','RUST_TOOLCHAIN=1.98.0']
invoke('s02-outside-offline-replay03',make+['jbotci-replay','JBOTCI_CACHE='+str(r/'s02-full-cache02'),'JBOTCI_RELEASE='+str(r/'s02-outside-release03.json')],{'VLAJMAJI_BIN':'','CURL':'/bin/false'})
assert (r/'s02-outside-release03.json').read_bytes()==(r/'s02-full-release02.json').read_bytes()
invoke('s02-outside-build03',['/tmp/vlajmaji-p01-a02-cargo','+1.98.0','build','--locked','--offline','--release','--manifest-path',r/'outside-caller-candidate02/Cargo.toml','--bin','a02_cdc_composition_20261024d'])
invoke('s02-outside-public03',[target/'release/a02_cdc_composition_20261024d','outside',store,raw,meta,canonical])
for package in ['vlajmaji-lexicon','vlajmaji-cli']:
 invoke('s02-package-list-'+package+'-03',['/tmp/vlajmaji-p01-a02-cargo','+1.98.0','package','--locked','--offline','--list','-p',package])
assert sha(db)==before
print('PASS negative-oracle2/fullCLI/exactcleanoutsideMakeCargofallbackofflinefullreplay/uniqueoutsidepublicimport-reopen-query/packageinspection')
