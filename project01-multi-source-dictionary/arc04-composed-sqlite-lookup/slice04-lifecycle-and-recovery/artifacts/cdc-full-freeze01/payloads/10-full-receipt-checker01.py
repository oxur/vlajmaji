"""Exact existing S04 wire shapes for the finite runner; pure until applied to receipts."""
import importlib.util, os, re, stat
from pathlib import Path
HERE=Path(__file__).parent
s=importlib.util.spec_from_file_location('wire',HERE/'phase2-wire-checker01.py')
wire=importlib.util.module_from_spec(s);s.loader.exec_module(wire)
MUTATION_KEYS=frozenset('version ok operation outcome local_revision policy_state attempt_id generation inputs'.split())
GENERATION_KEYS=frozenset('generation_id content_id canonical_sha256 path members'.split())
INPUT_KEYS=frozenset('pack_id disposition payload_kind release_sha256 report'.split())
CONFIGURE_KEYS=frozenset('version ok operation outcome path profile default_profile prior_revision revision prior_sha256 sha256 observed_catalogue_revision observed_availability'.split())
RECONCILE_KEYS=frozenset('version ok operation root_path catalogue_schema_version local_revision active_generation retained_generation_count attempts staged_attempts journal_before journal_after sqlite_recovery verification_scope active_store_minimum attention_required migration_required_for_mutation'.split())
MANAGED_ERROR_KEYS=frozenset('version ok operation category detail config_path root_path requested_profile known_pack_ids profile snapshot writer outcome'.split())
TIME_LABELS=('maximum resident set size','average shared memory size','average unshared data size','average unshared stack size','page reclaims','page faults','swaps','block input operations','block output operations','messages sent','messages received','signals received','voluntary context switches','involuntary context switches','instructions retired','cycles elapsed','peak memory footprint')
def exact(v,keys):assert isinstance(v,dict) and set(v)==keys,(set(v) if isinstance(v,dict) else type(v),keys)
def digest(v):assert isinstance(v,str) and re.fullmatch('[0-9a-f]{64}',v)
def native(v):
    exact(v,{'encoding','hex','utf8'});assert v['encoding']=='unix-bytes'
    assert isinstance(v['hex'],str) and re.fullmatch('(?:[0-9a-f]{2})+',v['hex'])
    raw=bytes.fromhex(v['hex']);assert b'\0' not in raw
    try:text=raw.decode('utf-8')
    except UnicodeDecodeError:text=None
    assert v['utf8']==text
    return Path(os.fsdecode(raw))
def safe_store(v,root,attempt):
    """No traversal, symlink component, or descriptor/display disagreement before IO."""
    p=native(v);assert p.is_absolute() and '..' not in p.parts and '.' not in p.parts
    assert isinstance(attempt,str) and attempt not in ('','.','..') and '/' not in attempt
    assert p==root/'generations'/attempt/'store',(p,root,attempt)
    # /tmp itself is an inherited host alias on macOS; owned ROOT and every descendant
    # must be real no-follow directories. Never resolve away an owned symlink.
    for q in [root,root/'generations',root/'generations'/attempt,p]:
        m=q.lstat();assert stat.S_ISDIR(m.st_mode) and not stat.S_ISLNK(m.st_mode),q
    assert p.resolve(strict=True).is_relative_to(root.resolve(strict=True)/'generations')
    return p

def mutation(v,operation,outcome,revision,ids,digests,kind,dispositions):
    exact(v,MUTATION_KEYS);assert v['version']==1 and v['ok'] is True
    assert (v['operation'],v['outcome'],v['local_revision'],v['policy_state'])==(operation,outcome,revision,'unconfigured')
    g=v['generation'];exact(g,GENERATION_KEYS)
    for k in ('content_id','canonical_sha256'):digest(g[k])
    assert isinstance(g['generation_id'],str) and g['generation_id']
    assert [m['pack_id'] for m in g['members']]==ids
    for m in g['members']:
        exact(m,{'pack_id','release_sha256','payload_kind'})
        assert m['release_sha256']==digests[m['pack_id']] and m['payload_kind']==kind
    assert len(v['inputs'])==len(dispositions)
    for item,want in zip(v['inputs'],dispositions):
        exact(item,INPUT_KEYS);assert item==want,(item,want)
    if outcome=='idempotent':assert v['attempt_id'] is None
    else:assert isinstance(v['attempt_id'],str) and v['attempt_id']
    return g

def configure(v,path,root):
    exact(v,CONFIGURE_KEYS);assert v['version']==1 and v['ok'] is True
    assert (v['operation'],v['outcome'],v['profile'],v['default_profile'])==('configure','committed','full','full')
    assert native(v['path'])==path
    assert v['prior_revision'] is None and v['prior_sha256'] is None and v['revision']==1
    assert v['observed_catalogue_revision']==1 and v['observed_availability']=='generation_observed';digest(v['sha256'])

def policy(v,operation,revision,path,root,ids,disabled,affected,cat_revision):
    wire.policy(v,operation,'committed');assert native(v['path'])==path
    assert v['prior_revision']==revision-1 and v['revision']==revision
    for k in ('prior_sha256','sha256'):digest(v[k])
    assert v['packs']==ids and v['disabled']==disabled and v['affected_pack_ids']==affected
    if operation=='clear_default':
        assert all(v[k] is None for k in ('profile','root','default_profile','observed_catalogue_revision','observed_availability'))
    else:
        assert v['profile']=='full' and native(v['root'])==root and v['default_profile']=='full'
        assert v['observed_catalogue_revision']==cat_revision and v['observed_availability']=='generation_observed'
    return v

def managed_error(v,category,config,operation='exact'):
    exact(v,{'error'});e=v['error'];exact(e,MANAGED_ERROR_KEYS)
    assert e['version']==1 and e['ok'] is False and e['operation']==operation
    assert e['category']==category and e['outcome']=='not_applicable' and e['writer'] is None
    assert native(e['config_path'])==config and e['requested_profile'] is None
    assert isinstance(e['known_pack_ids'],list) and isinstance(e['detail'],str)
    return e

def time_framed_error(raw):
    import json
    text=raw.decode('utf-8');text=text.lstrip()
    v,end=json.JSONDecoder().raw_decode(text)
    # Permit ONLY time(1)'s declared numeric metric lines after one complete JSON
    # value. No arbitrary child warnings, extra JSON, or unknown text is ignored.
    tail=text[end:];assert tail.startswith('\n')
    lines=[l.strip() for l in tail.splitlines() if l.strip()]
    assert len(lines)>=2
    assert re.fullmatch(r'[0-9.]+ real\s+[0-9.]+ user\s+[0-9.]+ sys',lines[0]),lines
    seen=[]
    for line in lines[1:]:
        m=re.fullmatch(r'(\d+)\s+('+ '|'.join(map(re.escape,TIME_LABELS))+')',line)
        assert m,line;seen.append(m.group(2))
        if m.group(2)=='maximum resident set size':assert int(m.group(1))>0
    assert len(seen)==len(set(seen)) and 'maximum resident set size' in seen
    exact(v,{'error'});return v

def reconcile(v,root,gs,reports,digests):
    import json
    exact(v,RECONCILE_KEYS);assert v['version']==1 and v['ok'] is True and v['operation']=='reconcile'
    assert native(v['root_path'])==root and v['catalogue_schema_version']==2 and v['local_revision']==4
    assert v['active_generation']==gs[-1]['generation_id'] and v['retained_generation_count']==4
    assert v['staged_attempts']==[] and v['attention_required'] is False and v['migration_required_for_mutation'] is False
    assert v['sqlite_recovery']=='writable_open_completed' and v['verification_scope']=='minimum_header_only'
    exact(v['active_store_minimum'],{'status','diagnostic'});assert v['active_store_minimum']=={'status':'supported','diagnostic':None}
    for k in ('journal_before','journal_after'):
        exact(v[k],{'present','byte_length'});assert v[k]=={'present':False,'byte_length':None}
    assert len(v['attempts'])==4
    want={r['attempt_id']:(r,i) for i,r in enumerate(reports)}
    assert {a['attempt_id'] for a in v['attempts']}==set(want)
    for a in v['attempts']:
        exact(a,{'attempt_id','operation','status','prior_revision','detail'})
        r,i=want[a['attempt_id']];assert a['operation']==r['operation'] and a['status']=='committed' and a['prior_revision']==i
        if a['operation'] in ('remove','reinstall'):
            e=json.loads(a['detail']);exact(e,{'version','operation','local_revision','packs','generation_id'})
            assert e['version']==1 and e['operation']==a['operation'] and e['local_revision']==i+1 and e['generation_id']==r['generation']['generation_id']
            assert len(e['packs'])==1;item=e['packs'][0];exact(item,{'pack_id','prior_state','release_sha256'})
            assert item=={'pack_id':'llg','prior_state':'installed' if a['operation']=='remove' else 'removed','release_sha256':digests['llg']}
