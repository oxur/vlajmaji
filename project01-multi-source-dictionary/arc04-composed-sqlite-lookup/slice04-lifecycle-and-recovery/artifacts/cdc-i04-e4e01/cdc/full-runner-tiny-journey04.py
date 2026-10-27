"""Explicitly separate Synthetic CLI orchestration qualification; NEVER invokes Full entrypoint.
Run only after exact clean frozen handback and same-target observer/CLI builds.
"""
import importlib.util, json, os, subprocess, sys
from pathlib import Path
HERE=Path(__file__).parent
s=importlib.util.spec_from_file_location('runner02',HERE/'full-operational-runner05.py');r=importlib.util.module_from_spec(s);s.loader.exec_module(r)
def main():
    assert len(sys.argv)==4,'CLI OBSERVER EXACT_SOURCE_COMMIT'
    cli,observer=map(Path,sys.argv[1:3]);head=sys.argv[3]
    base=HERE/'full-runner-tiny-method03';base.mkdir(mode=0o700)
    prepare_env={k:str(base/v) for k,v in [('HOME','prepare-home'),('XDG_DATA_HOME','prepare-data'),('XDG_CONFIG_HOME','prepare-config')]}
    for v in prepare_env.values():Path(v).mkdir(mode=0o700)
    guard=HERE/'stage_guard_split01.py'
    argv=[sys.executable,str(guard),'--timeout','60','--max-rss-bytes',str(512*1024**2),'--log',str(base/'prepare.stderr'),'--stdout',str(base/'prepare.stdout'),'--',str(observer),'prepare-tiny',str(base/'inputs')]
    r.new_json(base/'prepare-argv.json',argv)
    prepared=subprocess.run(argv,env=prepare_env,capture_output=True,timeout=65)
    (base/'prepare-guard.stdout').write_bytes(prepared.stdout);(base/'prepare-guard.stderr').write_bytes(prepared.stderr)
    assert prepared.returncode==0,prepared.stderr
    prep=json.loads(prepared.stdout);assert prep['exit_code']==0 and prep['decision']=='passed'
    inputs=[p for p in sorted((base/'inputs').rglob('*')) if p.is_file() and p.name!='manifest.json']
    batch=json.loads((base/'inputs/manifest.json').read_bytes())
    digests={x['pack_id']:r.sha(x['release']) for x in batch['inputs']}
    files=[cli,observer,guard,HERE/'full-operational-runner05.py',HERE/'full-receipt-checker01.py',HERE/'phase2-wire-checker01.py']
    cfg={'readiness_status':'TINY_METHOD_FROZEN_HANDOFF','root':str(base/'root'),'output':str(base/'outputs'),'config':str(base/'config.toml'),'source':'/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc04/vlajmaji','source_commit':head,'cli':str(cli),'observer':str(observer),'guard':str(guard),'batch':str(base/'inputs/manifest.json'),'witness':str(HERE/'full-caller01/accepted-descriptors-and-refs02.json'),'native_oracle':'UNUSED_IN_EXPLICIT_TINY_METHOD','pack_release_sha256':digests,'frozen_files':[{'path':str(p),'sha256':r.sha(p)} for p in files],'sealed_input_files':[{'path':str(p),'sha256':r.sha(p)} for p in inputs]}
    r.new_json(base/'tiny-method-preflight.json',cfg)
    r.execute(cfg,tiny=True)
    print(json.dumps({'ok':True,'scope':'actual original four-pack Synthetic CLI receipt/native path/policy/cursor/old-store/4-publication orchestration via SAME runner execute function; Full constants/native SQL/proof remain separate','source_commit':head,'runner_sha256':r.sha(HERE/'full-operational-runner05.py'),'summary':str(base/'outputs/summary.json')}))
if __name__=='__main__':main()
