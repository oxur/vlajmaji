"""Build review-only Full execution packet; NEVER stat/read/hash Full ROOT/config/DB."""
from pathlib import Path
import importlib.util,json,shutil,subprocess
D=Path('/tmp/vlajmaji-p01-a04-s05-cdc');P=Path('/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/planning');S=Path('/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc04/vlajmaji')
sp=importlib.util.spec_from_file_location('runner',D/'numeric-runner01.py');r=importlib.util.module_from_spec(sp);sp.loader.exec_module(r)
g=lambda *a:subprocess.check_output(['git','--no-optional-locks','-C',str(S),*a]).decode().strip()
assert g('rev-parse','HEAD')=='1b9038ddc6c45f3ea709a0bdfec64041221e3571' and g('status','--porcelain=v1')==''
assert not r.OUTPUT.exists()
recorded=json.loads(Path('/tmp/vlajmaji-p01-a04-s04-cdc/full-campaign01/G4-readonly-hashes.json').read_bytes());assert recorded['before']==recorded['after'];expected=recorded['after']
expected_paths={str(r.ROOT/'catalogue.sqlite3'),str(r.ROOT/'generations/22138258fb19a4b58ae1bb67d230a49e/store/dictionary.sqlite3')};assert set(expected)==expected_paths
# Config pin is from accepted last writer report+settled inventory, not a current Full leaf read.
old_report=json.loads(Path('/tmp/vlajmaji-p01-a04-s04-cdc/full-campaign01/profile5-restore.stdout').read_bytes());assert old_report['revision']==5 and old_report['sha256']=='51a6cb10ad3c7a0dd27a5bc03f207cd1d81774d6cf4cbeb21ee64c75fd17108c'
cfg={'readiness_status':'FULL_EXACT_PACKET_FOR_ROOT_REVIEW_NOT_EXECUTION_CLEARANCE','root':str(r.ROOT),'config':str(r.CONFIG),'output':str(r.OUTPUT),'cli':str(D/'numeric-frozen-binaries01/vlajmaji'),'observer':str(D/'numeric-frozen-binaries01/numeric-observer'),'baseline':str(r.BASELINE),'workload':str(r.WORK),'guard':str(r.GUARD),'python':'/Library/Frameworks/Python.framework/Versions/3.11/bin/python3','source':str(S),'source_commit':g('rev-parse','HEAD'),'source_tree':g('rev-parse','HEAD^{tree}'),'catalogue_revision':4,'profile_revision':5,'minimum_free_bytes':16*1024**3,'target_cache':'/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc01/target','sqlite_runtime_version':'3.53.2','database_paths':sorted(expected),'expected_database_sha256':expected}
witness=json.loads(r.WITNESS.read_bytes());cfg['expected_assertion_ids']=[a['assertion']['id'] for inp in witness['inputs'] for a in inp['farvi']];assert len(set(cfg['expected_assertion_ids']))==5
# Source-listed fixture families from current cli/search/store tests. Including older
# extant fixtures is conservative historical OVERCOUNT, not proof of S05 ownership.
explicit=('s05-','vlajmaji-s05-','vlajmaji-p01-a04-s05')
generic=('vlajmaji-reference-projection-','vlajmaji-query-output-','vlajmaji-s04-',
 'vlajmaji-s03-profile-','vlajmaji-s03-cli-','vlajmaji-s03-managed-','vlajmaji-s02-',
 'vlajmaji-verification-','vlajmaji-store-fixture-','vlajmaji-dictionary-store-',
 'vlajmaji-data-pack-','vlajmaji-cli-default-store-','vlajmaji-topology-',
 'vlajmaji-cli-verify-','vlajmaji-search-topology-','vlajmaji-cli-marker-unit-','vlajmaji-manifest-')
import os,stat
inherited=Path(os.environ['TMPDIR']);assert inherited.is_absolute() and '..' not in inherited.parts
if inherited.parts[1:2]==('var',):
 assert stat.S_ISLNK(Path('/var').lstat().st_mode) and os.readlink('/var')=='private/var'
 inherited=Path('/private/var').joinpath(*inherited.parts[2:])
assert inherited==Path('/private/var/folders/dt/kc8ndn6d3kx8c1jnxr05drsh0000gn/T')
r.nofollow(inherited) # qualify system alias first, then strict owned descendant paths
scratch=[D]
for base in (Path('/tmp'),inherited):
 for p in base.iterdir():
  if p!=D and p.name.startswith(explicit+generic):
   assert not p.name.startswith('vlajmaji-p01-') or p.name.startswith('vlajmaji-p01-a04-s05')
   scratch.append(p)
assert len({str(r.canonical_system_tmp(p)) for p in scratch})==len(scratch)
cfg['new_s05_scratch_paths']=[str(p) for p in sorted(scratch)];observed=r.inventory(scratch);assert observed['bytes']<=12*1024**3
cfg['scratch_scope_provenance']={'temporary_bases':['/tmp',str(inherited)],'inherited_TMPDIR_original':os.environ['TMPDIR'],'source_prefixes_explicit':explicit,'source_fixture_prefixes_conservative':generic,'caveat':'Includes older relevant generic fixture roots conservatively; not S05-only/cumulative/peak attribution. Does not select retained p01 roots except explicit S05, actual user default or arbitrary directory names. No cleanup/reset.','source_calibration':'Actual query scratch prefix already vlajmaji-s05-query-output; confirmed old omission was inherited TMPDIR plus generic reference fixture coverage, not invented prefix drift.'}
paths={Path(row['path']).resolve() for row in json.loads((D/'numeric-source-closure04.json').read_bytes())['files']}
paths|={Path(cfg[k]).resolve() for k in ['cli','observer','baseline','workload','python','guard']};paths.add(r.WITNESS.resolve());paths.add(Path(__file__).resolve());paths.add((S/'Cargo.lock').resolve());paths.add((S/'Cargo.toml').resolve())
changed=g('diff','--name-only','32e99b4fb0bfdc24e66535952114b6497d6aabfa','HEAD').splitlines();assert len(changed)==21
paths|={(S/p).resolve() for p in changed}
cfg['frozen_files']=[{'path':str(p),'bytes':p.stat().st_size,'sha256':r.sha(p)} for p in sorted(paths)]
config_pin={'path':str(r.canonical_system_tmp(r.CONFIG)),'bytes':219,'sha256':old_report['sha256']};cfg['frozen_files'].append(config_pin)
cfg['deferred_full_metadata_pin']={'pin':config_pin,'observation':'Expected from accepted S04 profile5 report+settled size; NOT currently stat/read/hash-verified. Runner strict leaf/hash only after root launch clearance.'}
cfg['db_hash_boundary']='DB paths/hashes from accepted G4 receipt only. DBs excluded from frozen_files; ONE actual before/after pair is inside the single campaign after clearance, prehash warms cache.'
cfg['expanded_stages']=r.expanded(cfg,False);assert len(r.expected_facts(cfg,False))==54
cfg['preflight_observed']={'free_bytes':shutil.disk_usage(D).free,'minimum_free_bytes':cfg['minimum_free_bytes'],'new_scratch_named_paths':len(scratch),'new_scratch_logical_bytes':observed['bytes'],'target_cache_bytes':r.target_cache_bytes(Path(cfg['target_cache'])),'mechanisms':'600s active per-stage process-group timeout;8GiB RSS/128MiB capture/12GiB ALL named S05 scratch are POSTCOMPLETION checks, not live enforcement; no cache reset/cleanup','finite_children':22,'phase_boundaries':'12fresh verdict before warm13; SAME21 analysis beforesemantic22; stop on first failure/no automatic retry','private_env':'runner creates0700 isolated HOME/XDG_DATA_HOME/XDG_CONFIG_HOME under fresh output; allchildren use explicit config; no actualdefault access','population':'12fresh4modesx3 balancedrounds +9warm3modesx3rounds54requests(27prime/27measured)each +ONEsemantic; sameID3obs med/max, no pooling/retries','connections':'one Unified provider/engine perprocess bysource; actual OS fdcount unmeasured','disk_arithmetic':'current named scratch <12GiB; up to22*128MiB capture plus named representative/detail files all counted against12GiB after each child; no new Full store/generation, retainedROOT outside quota, sharedtarget excluded with quiet before/after allocation pair','output_accounting':'final inventory/summary own bytes explicitly named post-inventory delta, not false exact all-files total'}
cfg['method_evidence']={'tiny_attempt01':'one REAL baseline CLI observation, failed sandboxRSS instrumentation; preserved unchanged','tiny_retry02':'approved elevated METHOD retry22actualPASS, DBpairunchanged/targetquietdelta0; not Fullmetrics','pure_controls':{'analysis':7,'fresh_stop':3,'symlink_inventory':4,'wire':9,'typed_compact_comparator_positive':1,'typed_compact_comparator_negative':6},'whole_scope':'No retainedFull root/config/DB current stat/read/hash/query in packet preparation; only accepted small priorreports/witness/baselinebinary and current sources/Tiny data.'}
p=D/'numeric-full-preflight02.json'
with p.open('x') as f:json.dump(cfg,f,indent=2);f.write('\n')
print(json.dumps({'path':str(p),'bytes':p.stat().st_size,'sha256':r.sha(p),'frozen_files':len(cfg['frozen_files']),'current_rehashed_pins':len(cfg['frozen_files'])-1,'deferred_config_pin':1,'expanded_stages':len(cfg['expanded_stages']),'expected_database_pair':'recorded only, no current DB reads','new_scratch_bytes':observed['bytes'],'free_bytes':shutil.disk_usage(D).free,'review_only_no_launch':True}))
