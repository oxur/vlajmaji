"""Separate owned Tiny10 entrypoint; no retained Full override."""
from pathlib import Path
import importlib.util,json,sys
D=Path(__file__).parent
s=importlib.util.spec_from_file_location('completion',D/'completion-runner01.py');c=importlib.util.module_from_spec(s);s.loader.exec_module(c)
assert len(sys.argv)==2
cfg=json.loads(Path(sys.argv[1]).read_bytes())
assert Path(cfg['root']).is_relative_to(D) and Path(cfg['config']).is_relative_to(D)
assert not Path(cfg['root']).is_relative_to(c.r.ROOT)
c.execute(cfg,tiny=True)
