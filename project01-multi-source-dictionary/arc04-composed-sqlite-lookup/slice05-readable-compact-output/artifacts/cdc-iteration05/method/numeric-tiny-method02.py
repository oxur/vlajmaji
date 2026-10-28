"""Separate owned-Tiny entrypoint for SAME exact numerical runner orchestration.
SOURCE ONLY until frozen-current readable/Tiny caller proof and target handback.
This does not weaken or override numeric-runner02.py's fixed Full entrypoint.
"""
import importlib.util,json,sys
from pathlib import Path
HERE=Path(__file__).parent
s=importlib.util.spec_from_file_location('numeric_runner',HERE/'numeric-runner02.py')
r=importlib.util.module_from_spec(s);s.loader.exec_module(r)
if __name__=='__main__':
 assert len(sys.argv)==2
 cfg=json.loads(Path(sys.argv[1]).read_bytes())
 assert cfg['readiness_status']=='TINY_METHOD_FROZEN_HANDOFF'
 assert Path(cfg['root']).is_relative_to(HERE) and Path(cfg['config']).is_relative_to(HERE)
 assert not Path(cfg['root']).is_relative_to(r.ROOT)
 r.execute(cfg,tiny=True)
