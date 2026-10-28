"""Prospective independent S05 readable03 actual CLI journey. SOURCE ONLY.
Only frozen-current binary/owned Tiny context may be supplied after handback.
No Full entrypoint, default fallback, retries, cleanup or artifact overwrites.
Library literal bytes are independently checked by readable-caller01 first.
"""
import collections, hashlib, importlib.util, json, os, re, shutil, stat, sys, traceback
from pathlib import Path
HERE=Path(__file__).parent
spec=importlib.util.spec_from_file_location('prior_cli',HERE/'cli-checks01.py')
prior=importlib.util.module_from_spec(spec);spec.loader.exec_module(prior)
PACKS=['lensisku-jbotci','llg','la-lojban','sutysisku']
MUTATION=set('version ok operation outcome local_revision policy_state attempt_id generation inputs'.split())

def j(v): return json.dumps(v,ensure_ascii=False,separators=(',',':'))
def atom(s): return j(s)[1:-1]
def nofollow_owned_file(path,base):
    assert path.is_absolute() and '..' not in path.parts
    rel=path.relative_to(base);walk=base
    assert stat.S_ISDIR(walk.lstat().st_mode)
    for part in rel.parts:
        walk=walk/part;m=walk.lstat();assert not stat.S_ISLNK(m.st_mode)
    assert stat.S_ISREG(path.lstat().st_mode)
    return path.read_bytes()
def path_report(v):
    assert set(v)=={'encoding','hex','utf8'} and v['encoding'] in ('unix-bytes','utf8')
    assert isinstance(v['hex'],str) and re.fullmatch(r'(?:[0-9a-f]{2})*',v['hex'])
    raw=bytes.fromhex(v['hex']);assert b'\0' not in raw
    if v['utf8'] is not None:assert v['utf8'].encode()==raw
    return Path(os.fsdecode(raw))
def mutation(v,operation,outcome,revision):
    assert set(v)==MUTATION and v['version']==1 and v['ok'] is True
    assert v['operation']==operation and v['outcome']==outcome and v['local_revision']==revision
    assert isinstance(v['attempt_id'],str) and v['attempt_id']
    assert v['generation'] is not None
    assert set(v['generation'])=={'generation_id','content_id','canonical_sha256','path','members'}
    assert [m['pack_id'] for m in v['generation']['members']]==[p for p in PACKS if not(operation=='remove' and p=='llg')]
    assert all(m['payload_kind']=='synthetic' for m in v['generation']['members'])
    return v

def readable_exact(raw,ctx,ids,managed=None):
    text=raw.decode('utf-8');assert text.endswith('\n') and not text.endswith('\n\n')
    assert '\x1b' not in text
    nsrc=2 if len(ids)==4 else 3
    lines=text.splitlines();assert lines[0]==f'Exact "farvi": {len(ids)} occurrences, 1 spellings, {nsrc} sources'
    got=[];groups=[];i=1
    while i<len(lines) and not lines[i].startswith(('selection:','next_cursor:')):
        head=lines[i];i+=1;source=next(s for s in ('lensisku','llg','sutysisku') if f'[{atom(s)}]' in head)
        n=2 if source in ('lensisku','sutysisku') else 1
        assert head==f'{atom(ctx["sources"][source]["abbreviation"])} [{atom(source)}] {n}/{n}'
        groups.append(source)
        for _ in range(n):
            row=lines[i];aid=next(a for a in ids if row.startswith('  '+atom(a)+' @ '))
            assert row=='  '+atom(aid)+' @ '+atom(tuple_id(ctx['expected_keys'][aid]['occurrence']))
            a=ctx['expected_assertions'][aid];e=ctx['expected_entries'][a['entry']]
            assert lines[i+1].startswith('    spelling: ') and json.loads(lines[i+1][14:])==e['spelling']
            assert lines[i+2].startswith('    definition: ') and json.loads(lines[i+2][16:])==a['definition']
            assert lines[i+3].startswith('    detail: ') and json.loads(lines[i+3][12:])==ctx['expected_keys'][aid]
            got.append(aid);i+=4
    assert got==ids and len(got)==len(set(got))
    assert groups==(['lensisku','sutysisku'] if len(ids)==4 else ['lensisku','llg','sutysisku'])
    if managed is not None:
        assert lines[i].startswith('selection: ') and json.loads(lines[i][11:])==managed['snapshot'];i+=1
        assert lines[i].startswith('collections: ') and json.loads(lines[i][13:])==managed['collections'];i+=1
    assert lines[i:]==['next_cursor: null']

def tuple_id(o):
    fields=[o['capture']['namespace'],o['capture']['sha256'],o['locator']]
    return b''.join(len(s.encode()).to_bytes(8,'big')+s.encode() for s in fields).hex()

def readonly_error(result,category,operation):
    assert result.stdout==b'';v=prior.strict_json(result.stderr)
    assert set(v)=={'error'} and set(v['error'])==prior.MANAGED
    e=v['error'];assert e['version']==1 and e['ok'] is False and e['category']==category and e['operation']==operation
    assert e['outcome']=='not_applicable' and e['writer'] is None
    return e

def readable_reference(raw,ctx,target,status,managed=None,reason=None,payload=True):
    text=raw.decode('utf-8');assert text.endswith('\n') and not text.endswith('\n\n') and '\x1b' not in text
    lines=text.splitlines();assert lines[0].startswith('Reference ')
    header=lines[0][10:];observed,end=json.JSONDecoder().raw_decode(header)
    assert observed==target and header[end:]==': '+status
    catalog='Catalog' in target;requested=ctx['expected_requested_release'] if catalog else None
    key=ctx['llg_key'];a=ctx['expected_assertions'][key['assertion_id']];e=ctx['expected_entries'][key['entry_id']]
    i=1
    if managed is not None:
        assert lines[i]=='reason: '+j(reason) and lines[i+1]=='payload: '+('true' if payload else 'false');i+=2
    assert lines[i].startswith('entity: ') and json.loads(lines[i][8:])==(ctx['expected_local_entity'] if payload and not catalog else None);i+=1
    assert lines[i]=='confirmed: '+('1' if payload else '0');i+=1
    if payload:
        assert lines[i]==f'{atom(ctx["sources"]["llg"]["abbreviation"])} [llg] 1 confirmed';i+=1
        assert lines[i]=='  '+atom(key['assertion_id'])+' @ '+atom(tuple_id(key['occurrence']));i+=1
        for label,value in [('spelling',e['spelling']),('definition',a['definition']),('eligibility',a['eligibility']),('payload_present',True),('current_eligible',True),('detail',key)]:
            prefix='    '+label+': ';assert lines[i].startswith(prefix) and json.loads(lines[i][len(prefix):])==value;i+=1
    assert lines[i]=='unverified: 0';i+=1
    assert lines[i].startswith('requested_release: ') and json.loads(lines[i][19:])==requested;i+=1
    if managed is not None:
        matches=ctx['expected_catalog_matches'] if catalog else ctx['expected_local_matches']
        for label,value in [('catalogue_matches',matches),('selection',managed['snapshot']),('collections',managed['collections'])]:
            prefix=label+': ';assert lines[i].startswith(prefix) and json.loads(lines[i][len(prefix):])==value;i+=1
    assert i==len(lines),lines[i:]

def readable_detail(raw,ctx,snapshot,managed=None,status='Resolved',reason=None,hit=True):
    text=raw.decode('utf-8');assert text.endswith('\n') and not text.endswith('\n\n') and '\x1b' not in text
    # Complete direct typed pretty body was obtained from unchanged full engine,
    # with the lexical assertion/key checked against original inputs beforehand.
    prefix,body=text.split('\nhit: ',1);lines=prefix.splitlines();k=ctx['llg_key']
    assert lines[0]=='Detail '+atom(k['assertion_id'])+': '+status
    assert lines[1]=='snapshot: '+j(snapshot)
    assert lines[2]=='reason: '+j(reason)
    assert lines[3]=='current_eligible: '+('true' if hit else 'null')
    assert lines[4].startswith('reference: ') and json.loads(lines[4][11:])==k
    i=5
    if managed is not None:
        for label,value in [('selection',managed['snapshot']),('collections',managed['collections']),('catalogue_matches',ctx['expected_local_matches'])]:
            pref=label+': ';assert lines[i].startswith(pref) and json.loads(lines[i][len(pref):])==value;i+=1
    assert i==len(lines)
    assert body==(ctx['known_llg_hit_pretty']+'\n' if hit else 'null\n')

def main():
    binary,context_file,out=map(Path,sys.argv[1:]);assert all(p.is_absolute() for p in (binary,context_file,out))
    ctx=prior.strict_json(context_file.read_bytes());assert ctx['fixture_scope']=='original-four-pack-synthetic-readable'
    base=context_file.parent;out.mkdir(mode=0o700)
    env={'PATH':os.defpath,'LANG':'C.UTF-8'}
    for key in ['HOME','XDG_CONFIG_HOME','XDG_DATA_HOME']:
        d=out/key.lower();d.mkdir(mode=0o700);env[key]=str(d)
    copied=out/'packs';copied.mkdir();records=[];llg=None
    for row in ctx['inputs']:
        pid=row['pack_id'];assert pid in PACKS
        d=copied/pid;d.mkdir()
        for key in ['release','seal']:
            b=nofollow_owned_file(Path(row[key]),base)
            assert hashlib.sha256(b).hexdigest()==row[key+'_sha256']
            with (d/(key+'.json')).open('xb') as f:f.write(b)
        records.append({'kind':'sealed','pack_id':pid,'release':f'packs/{pid}/release.json','seal':f'packs/{pid}/seal.json'})
        if pid=='llg':llg=row
    assert [r['pack_id'] for r in records]==PACKS
    manifest=out/'manifest.json';manifest.write_text(j({'version':1,'inputs':records})+'\n')
    root=out/'root';config=out/'config.toml';assert not root.exists() and not config.exists()
    def run(name,args,expected=0,**kw):return prior.run(binary,args,env,out,name,expected,**kw)
    def wire(name,args):
        r=run(name,args);assert r.stderr==b'';return prior.strict_json(r.stdout)
    install=mutation(wire('W1-install',['collections','install',root,'--manifest',manifest,'--all','--allow-synthetic']),'install','installed',1)
    store=path_report(install['generation']['path']);store.relative_to(root)
    assert '..' not in store.parts and store.is_absolute()
    # Owned generation leaf/path identity must be established before any explicit-store read.
    assert stat.S_ISDIR(root.lstat().st_mode) and not stat.S_ISLNK(root.lstat().st_mode)
    walk=root
    for part in store.relative_to(root).parts:
        walk=walk/part;assert stat.S_ISDIR(walk.lstat().st_mode) and not stat.S_ISLNK(walk.lstat().st_mode)
    def state(cat,profile,name):
        l=wire(name+'-list',['collections','list',root]);assert l['local_revision']==cat and l['catalogue_schema_version']==2
        if profile is not None:
            p=wire(name+'-profiles',['collections','profiles','--config',config]);assert p['revision']==profile and p['default_profile']=='work'
        return l
    state(1,None,'W2-initial')
    for p in PACKS:assert wire('W2-inspect-'+p,['collections','inspect',root,p])['pack_id']==p
    c=wire('W3-configure',['collections','configure',root,'--profile','work','--all','--make-default','--config',config])
    assert c['outcome']=='committed' and c['revision']==1 and c['prior_revision'] is None
    state(1,1,'W3-configured')
    full=run('W3-default',['lookup','farvi','--config',config]);explicit=run('W3-explicit-full',['lookup','farvi','--config',config,'--format','full'])
    assert full.stdout==explicit.stdout and full.stderr==explicit.stderr==b''
    args=['lookup','farvi','--config',config,'--profile','work']
    compact=wire('W5-compact',[*args,'--format','compact']);prior.compact_exact_wire(compact['result']['response'])
    readable=run('W5-readable',[*args,'--format','readable']);assert readable.stderr==b''
    ids=ctx['ordered_lexical_assertion_ids'];assert len(ids)==5
    readable_exact(readable.stdout,ctx,ids,compact)
    standalone_args=['dictionary-query',store,'--request',j({'operation':'exact','spelling':'farvi'})]
    standalone_full=run('W5-store-default',standalone_args);standalone_explicit=run('W5-store-full',[*standalone_args,'--format','full'])
    assert standalone_full.stdout==standalone_explicit.stdout and standalone_full.stderr==standalone_explicit.stderr==b''
    standalone_compact=wire('W5-store-compact',[*standalone_args,'--format','compact']);prior.compact_exact_wire(standalone_compact)
    projected_ids=[h['assertion']['id'] for g in standalone_compact['groups'] for h in g['hits']]
    assert projected_ids==ids and standalone_compact['totals']['occurrences']==5
    for g in standalone_compact['groups']:
        for h in g['hits']:
            assert h['detail_ref']==ctx['expected_keys'][h['assertion']['id']]
            assert h['assertion']==ctx['expected_assertions'][h['assertion']['id']]
            assert h['entry']==ctx['expected_entries'][h['entry']['id']]
    standalone=run('W5-store-readable',[*standalone_args,'--format','readable'])
    assert standalone.stderr==b'';readable_exact(standalone.stdout,ctx,ids)
    local=ctx['local_request'];catalog=ctx['catalog_request'];key=j(ctx['llg_key'])
    for kind,q in [('local',local),('catalog',catalog)]:
        ref_args=['lookup','--request',j(q),'--config',config,'--profile','work']
        counterpart=wire('W5-'+kind+'-compact',[*ref_args,'--format','compact'])
        r=run('W5-'+kind,[*ref_args,'--format','readable']);assert r.stderr==b'';readable_reference(r.stdout,ctx,q['target'],'Resolved',counterpart)
        sr=run('W5-store-'+kind,['dictionary-query',store,'--request',j(q),'--format','readable']);assert sr.stderr==b'';readable_reference(sr.stdout,ctx,q['target'],'Resolved')
    detail_args=['lookup-detail','--reference',key,'--config',config,'--profile','work','--format','readable']
    r=run('W5-detail',detail_args);assert r.stderr==b'';readable_detail(r.stdout,ctx,compact['snapshot']['binding_sha256'],compact)
    sd=run('W5-store-detail',['dictionary-detail',store,'--reference',key,'--format','readable']);assert sd.stderr==b'';readable_detail(sd.stdout,ctx,standalone_compact['snapshot_sha256'])
    page=wire('W5-cursor',[*args,'--page-size','1','--format','compact']);cursor=page['result']['response']['next_cursor'];assert isinstance(cursor,str)
    observed=compact['snapshot']['binding_sha256']
    pinned=run('W5-detail-pinned',[*detail_args,'--expected-snapshot',observed]);assert pinned.stdout==r.stdout and pinned.stderr==b''
    for name,read_args,expected_bytes in [('standalone',['dictionary-query',store,'--request',j({'operation':'exact','spelling':'farvi'}),'--format','readable'],standalone.stdout),('managed',[*args,'--format','readable'],readable.stdout),('detail',detail_args,r.stdout)]:
        cwd=out/('deleted-cwd-'+name);cwd.mkdir()
        d=run('W5-'+name+'-deleted-cwd',read_args,cwd=cwd,preexec_fn=lambda p=cwd:os.rmdir(p));assert d.stdout==expected_bytes and d.stderr==b'' and not cwd.exists()
    for command,prefix in [('lookup-detail',[]),('dictionary-detail',[store])]:
        bad=[command,*prefix,'--reference',key,'--format','compact']
        if command=='lookup-detail':bad+=['--config',config]
        x=run('W5-'+command+'-compact-clap',bad,2);assert x.stdout==b'' and x.stderr
    d=wire('W4-disable',['collections','disable','--profile','work','--pack','llg','--config',config]);assert d['outcome']=='committed' and d['revision']==2
    state(1,2,'W4-disabled')
    subset=wire('W4-disabled-compact',[*args,'--format','compact']);assert subset['result']['response']['totals']['occurrences']==4
    r=run('W4-disabled-readable',[*args,'--format','readable']);readable_exact(r.stdout,ctx,[i for i in ids if i!=ctx['llg_key']['assertion_id']],subset)
    readonly_error(run('W4-disabled-oldcursor',[*args,'--page-size','1','--cursor',cursor,'--format','compact'],1),'stale_cursor','lookup')
    e=wire('W4-enable',['collections','enable','--profile','work','--pack','llg','--config',config]);assert e['outcome']=='committed' and e['revision']==3
    state(1,3,'W4-enabled');r=run('W4-enabled-readable',[*args,'--format','readable']);c=wire('W4-enabled-compact',[*args,'--format','compact']);readable_exact(r.stdout,ctx,ids,c)
    mutation(wire('W4-remove',['collections','remove',root,'--pack','llg','--expected-revision','1']),'remove','removed',2);state(2,3,'W4-removed')
    er=readonly_error(run('W4-removed-exact',[*args,'--format','readable'],1),'unavailable_selection','lookup');assert 'removed' in er['detail']
    removed_compact=None
    for kind,q in [('local',local),('catalog',catalog)]:
        ref_args=['lookup','--request',j(q),'--config',config,'--profile','work']
        removed_compact=wire('W4-removed-'+kind+'-compact',[*ref_args,'--format','compact'])
        r=run('W4-removed-'+kind,[*ref_args,'--format','readable']);assert r.stderr==b'';readable_reference(r.stdout,ctx,q['target'],'CatalogueOnly',removed_compact,'removed',False)
    d=run('W4-removed-detail',detail_args);assert d.stderr==b'';readable_detail(d.stdout,ctx,removed_compact['snapshot']['binding_sha256'],removed_compact,'CatalogueOnly','removed',False)
    readonly_error(run('W4-removed-detail-oldsnapshot',[*detail_args,'--expected-snapshot',observed],1),'stale_snapshot','lookup_detail')
    assert llg is not None
    mutation(wire('W4-reinstall',['collections','reinstall',root,'--pack','llg','--release',copied/'llg/release.json','--seal',copied/'llg/seal.json','--allow-synthetic','--release-sha256',llg['release_sha256'],'--expected-revision','2']),'reinstall','reinstalled',3)
    state(3,3,'W4-restored');c=wire('W5-restored-compact',[*args,'--format','compact']);r=run('W5-restored-readable',[*args,'--format','readable']);readable_exact(r.stdout,ctx,ids,c)
    d=run('W5-restored-detail',detail_args);assert d.stderr==b'';readable_detail(d.stdout,ctx,c['snapshot']['binding_sha256'],c)
    readonly_error(run('W5-restored-detail-oldsnapshot',[*detail_args,'--expected-snapshot',observed],1),'stale_snapshot','lookup_detail')
    old=run('W5-retained-old-store',['dictionary-query',store,'--request',j({'operation':'exact','spelling':'farvi'}),'--format','readable']);assert old.stdout==standalone.stdout and old.stderr==b''
    rec=wire('W4-reconcile',['collections','reconcile',root]);assert rec['attention_required'] is False
    final=state(3,3,'W5-final');assert final['staged_attempts']==[]
    # Explicit current native/dev-full process proof is Linux external gate on macOS.
    summary={'ok':True,'scope':'independent actual original-four-literal Tiny W1-W5 readable CLI journey','catalogue_revision':3,'profile_revision':3,'full_unchanged':True,'linux_native_devfull':'pending current-head Linux CI; no local Mac branch claim'}
    with (out/'summary.json').open('x') as f:json.dump(summary,f,indent=2);f.write('\n')
    print(json.dumps(summary))

if __name__=='__main__':
    try:main()
    except BaseException as exc:
        print('READABLE_METHOD_OR_PRODUCT_FAILURE '+repr(exc),file=sys.stderr);traceback.print_exc();raise
