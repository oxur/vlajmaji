"""PROSPECTIVE S04 ONE-root/four-publication campaign. Syntax-only until reviewed frozen preflight.
No retries, import, cache control, accepted-root access or inferred delivery authority.
"""
import hashlib, importlib.util, json, os, stat, subprocess, sys, time
from pathlib import Path
IDS = ['lensisku-jbotci', 'llg', 'la-lojban', 'sutysisku']
GIB = 1024**3
HERE=Path(__file__).parent
_spec=importlib.util.spec_from_file_location('receipt_check',HERE/'full-receipt-checker01.py')
check=importlib.util.module_from_spec(_spec);_spec.loader.exec_module(check)
FULL_PINS={'lensisku-jbotci':'0987ec84a8434573b5979ea854ca459ce2ea9c7fa03ebac6639574dd2b7aa460','llg':'d8c8be02b9888c1eee3d3daa053746e483aec995935c4d225234479d4215ce12','la-lojban':'e66951094935bd910c6a9ad12fa1bbb51f0578b53d44f22a73056811fe2bc64e','sutysisku':'67499447426d34c269577eb6947d7c7b22b71090e5a30fd653375819d590ffa2'}
FULL_CANON = 'd9d73cbb49cc5a9ddb0ec16cb6dbb4bb7260ad49f4f8f54c6e6d21f0779b7b19'
FULL_CONTENT = 'b4ee3e48edb910ce63b892f2a1d61342c26d3f2a1a46545f93ab15157192766e'
FULL_WIRE = '144b8b09b6e378243b36c8e01ab06353c3d573459668c8c158a86aebbea73556'

def sha(path):
    h = hashlib.sha256()
    with Path(path).open('rb') as f:
        for part in iter(lambda: f.read(8*1024*1024), b''): h.update(part)
    return h.hexdigest()

def new_json(path, value):
    with Path(path).open('x') as f:
        json.dump(value, f, indent=2); f.write('\n')

def child_error(path):
    return check.time_framed_error(Path(path).read_bytes())

def content(members):
    pairs = [(p, members[p]) for p in IDS if p in members]
    return hashlib.sha256(b'vlajmaji-collection-content-v1\0'+json.dumps(pairs,separators=(',',':')).encode()).hexdigest()

def check_lookup(value, subset=False, counts=None):
    r = value['result']['response']
    expected=counts or ({'lensisku':3,'sutysisku':1} if subset else {'lensisku':3,'llg':1,'sutysisku':1})
    assert r['totals']['occurrences'] == sum(expected.values())
    assert r['totals']['source_groups'] == len(expected)
    got = {s['source']['id']: s['occurrences'] for s in r['source_totals'] if s['occurrences']}
    assert got == expected

def inventory(paths):
    files, entries = [], []
    for root in paths:
        if not root.exists(): continue
        for p in [root,*sorted(root.rglob('*'))]:
            m=p.lstat();assert not stat.S_ISLNK(m.st_mode),'unexpected owned scratch symlink '+str(p)
            kind='directory' if stat.S_ISDIR(m.st_mode) else 'file' if stat.S_ISREG(m.st_mode) else 'unsupported'
            assert kind!='unsupported',p
            entries.append({'path':str(p),'kind':kind})
            if kind=='file': files.append({'path':str(p),'bytes':m.st_size})
    return {'entries':entries,'files':files,'bytes':sum(x['bytes'] for x in files)}

def main(preflight):
    cfg = json.loads(Path(preflight).read_text())
    assert cfg['pack_release_sha256']==FULL_PINS
    assert cfg['root']=='/tmp/vlajmaji-p01-a04-s04-cdc/full-operational01'
    assert cfg['output']=='/tmp/vlajmaji-p01-a04-s04-cdc/full-campaign01'
    assert cfg['config']=='/tmp/vlajmaji-p01-a04-s04-cdc/full-config01.toml'
    assert Path(cfg['batch'])==HERE/'full-batch-proposed01.json'
    assert sha(cfg['batch'])=='eaccc4fed4ef8834e0c6f1d7dced910aabe0b9aa914e15aa92a7fc88277af484'
    execute(cfg, tiny=False)

def execute(cfg, *, tiny):
    # Only the separately named Tiny method entrypoint may call this with tiny=True.
    # Full CLI entrypoint above fixes known pins, destinations, and expected constants.
    # Documentary readiness record, never a substitute for actual root review/tool authority.
    assert cfg['readiness_status'] == ('TINY_METHOD_FROZEN_HANDOFF' if tiny else 'ROOT_REVIEWED_EXACT_FROZEN_CAMPAIGN')
    root, out, config = map(Path, (cfg['root'],cfg['output'],cfg['config']))
    if tiny:
        assert root.name=='root' and out.name=='outputs' and config.name=='config.toml'
        assert root.parent==out.parent==config.parent and root.parent.name.startswith('full-runner-tiny-method')
    assert not root.exists() and not out.exists() and not config.exists()
    source, cli, observer, guard = map(Path,(cfg['source'],cfg['cli'],cfg['observer'],cfg['guard']))
    genv = {**os.environ,'GIT_OPTIONAL_LOCKS':'0'}
    assert subprocess.check_output(['git','--no-optional-locks','-C',str(source),'rev-parse','HEAD'],env=genv).decode().strip() == cfg['source_commit']
    assert not subprocess.check_output(['git','--no-optional-locks','-C',str(source),'status','--porcelain=v1'],env=genv)
    for entry in cfg['frozen_files']:
        assert sha(entry['path']) == entry['sha256'],entry['path']
    inputs_before = {e['path']:sha(e['path']) for e in cfg['sealed_input_files']}
    assert inputs_before == {e['path']:e['sha256'] for e in cfg['sealed_input_files']}
    out.mkdir(mode=0o700)
    isolated={k:str(out/n) for k,n in [('HOME','isolated-home'),('XDG_DATA_HOME','isolated-data'),('XDG_CONFIG_HOME','isolated-config')]}
    for value in isolated.values():Path(value).mkdir(mode=0o700)
    new_json(out/'child-environment.json',isolated)
    expected = out/'pre-input-expected.json'
    batch, witness, oracle = map(Path,(cfg['batch'],cfg['witness'],cfg['native_oracle']))
    digests = cfg['pack_release_sha256']; assert set(digests) == set(IDS)
    expected_content=content(digests)
    if not tiny:assert expected_content == FULL_CONTENT
    kind='synthetic' if tiny else 'full'
    all_counts={'lensisku':2,'llg':1,'sutysisku':2} if tiny else {'lensisku':3,'llg':1,'sutysisku':1}
    subset_counts={k:v for k,v in all_counts.items() if k!='llg'}
    allow=['--allow-synthetic'] if tiny else []
    kept = [p for p in IDS if p != 'llg']
    subset_content = content({p:digests[p] for p in kept})
    receipts, generations, immutable, reports = [], [], {}, []
    publications = 0
    def measure_resources(label):
        usage = inventory([root,out])
        # Config is a separate owned file; stages/locks also share its parent.
        # Actual publisher sibling(path,suffix) appends to FULL filename:
        # config.toml.lock and config.toml.stage-PID-NONCE (no leading-dot prefix).
        for p in config.parent.iterdir():
            if not (p.name==config.name or p.name.startswith(config.name+'.')):continue
            if p.is_symlink(): raise AssertionError('unexpected config sibling symlink '+str(p))
            if p.is_file(): usage['files'].append({'path':str(p),'bytes':p.stat().st_size})
        for p in out.rglob('*'):
            if p.is_file(): assert p.stat().st_size <= 128*1024**2, str(p)
        usage['bytes'] = sum(x['bytes'] for x in usage['files'])
        usage['accounting']='Observed bytes before writing this inventory; its own bytes and later summary/receipt bytes are disclosed post-inventory deltas. Final settled inventory is separately saved.'
        new_json(out/(label+'-data-inventory.json'),usage)
        assert usage['bytes'] <= 12*GIB
    def stage(name, argv, rss=8*GIB, expected_exit=0):
        args = [str(x) for x in argv]
        new_json(out/(name+'-argv.json'),args)
        stdout, stderr = out/(name+'.stdout'), out/(name+'.stderr')
        command = [sys.executable,str(guard),'--timeout','600','--max-rss-bytes',str(rss),'--log',str(stderr),'--stdout',str(stdout),'--',*args]
        started = time.monotonic()
        r = subprocess.run(command,env=isolated,capture_output=True,timeout=610)
        (out/(name+'-guard.stdout')).open('xb').write(r.stdout)
        (out/(name+'-guard.stderr')).open('xb').write(r.stderr)
        receipt = json.loads(r.stdout)
        receipt.update(guard_exit=r.returncode,expected_child_exit=expected_exit,outer_elapsed_s=time.monotonic()-started)
        new_json(out/(name+'-receipt.json'),receipt); receipts.append(receipt)
        assert not receipt['timed_out'] and receipt['rss_parse_ok'] and receipt['maximum_resident_set_size_bytes'] <= rss
        # Guard returns the child's nonzero exit for deliberate negative stages.
        assert r.returncode == expected_exit and receipt['exit_code'] == expected_exit
        assert stdout.stat().st_size <= 128*1024**2 and stderr.stat().st_size <= 128*1024**2
        measure_resources(name)
        return stdout
    def call(name,args,rss=8*GIB,expected_exit=0):
        p = stage(name,[cli,*args],rss,expected_exit)
        return child_error(out/(name+'.stderr')) if expected_exit else json.loads(p.read_text())
    def read_phase(name, generation, subset, revision, native=False):
        store=check.safe_store(generation['path'],root,generation['_attempt'])
        db = store/'dictionary.sqlite3'; cat = root/'catalogue.sqlite3'
        before = {str(db):sha(db),str(cat):sha(cat)}
        full = call(name+'-All',['verify','all',store],12*GIB); assert full['ok'] is True
        wire = out/(name+'-inner.json')
        stage(name+'-public',[observer,'observe-tiny' if tiny else 'observe',root,expected,witness,'subset' if subset else 'all',str(revision),wire])
        assert wire.stat().st_size <= 128*1024**2
        if not subset and not tiny: assert wire.stat().st_size == 7160366 and sha(wire) == FULL_WIRE
        if native and not tiny:
            packs = ','.join(kept if subset else IDS)
            stage(name+'-native',[sys.executable,oracle,db,out/(name+'-native-result'),packs])
        after = {p:sha(p) for p in before}; assert before == after
        new_json(out/(name+'-readonly-hashes.json'),{'before':before,'after':after})
        immutable[str(db)] = sha(db)
    def publication(name,args,revision,subset=False):
        nonlocal publications
        publications += 1; assert publications <= 4
        report = call(name,args,12*GIB)
        op,outcome={'G1-install':('install','installed'),'G2-rebuild':('rebuild','rebuilt'),'G3-remove-llg':('remove','removed'),'G4-exact-reinstall':('reinstall','reinstalled')}[name]
        if op in ('install','rebuild'): dispositions=[{'pack_id':p,'disposition':'installed','payload_kind':kind,'release_sha256':digests[p],'report':None} for p in IDS]
        elif op=='reinstall':dispositions=[{'pack_id':p,'disposition':'validated' if p=='llg' else 'skipped_without_io','payload_kind':kind if p=='llg' else None,'release_sha256':digests[p] if p=='llg' else None,'report':None} for p in IDS]
        else:dispositions=[]
        g=check.mutation(report,op,outcome,revision,kept if subset else IDS,digests,kind,dispositions)
        reports.append(report)
        assert [m['pack_id'] for m in g['members']] == (kept if subset else IDS)
        assert {m['pack_id']:m['release_sha256'] for m in g['members']} == {p:digests[p] for p in (kept if subset else IDS)}
        assert g['content_id'] == (subset_content if subset else expected_content)
        if not subset and not tiny: assert g['canonical_sha256'] == FULL_CANON
        check.safe_store(g['path'],root,report['attempt_id'])
        # Private runner-only association; never assert this as a serialized field.
        g=dict(g);g['_attempt']=report['attempt_id']
        assert g['generation_id'] not in {x['generation_id'] for x in generations}
        generations.append(g)
        report={**report,'generation':g}
        for path, old in immutable.items(): assert sha(path) == old
        return report
    try:
        stage('P0-input-expectations',[observer,'freeze-tiny' if tiny else 'freeze',batch,expected])
        g1 = publication('G1-install',['collections','install',root,'--manifest',batch,'--all',*allow],1)
        read_phase('G1',g1['generation'],False,1,True)
        profile = call('profile1-configure',['collections','configure',root,'--profile','full','--all','--make-default','--config',config]);check.configure(profile,config,root)
        check_lookup(call('profile1-lookup',['lookup','farvi','--config',config]),counts=all_counts)
        page1=call('G1-cursor-origin',['lookup','farvi','--page-size','1','--config',config])
        cursor=page1['result']['response']['next_cursor'];assert isinstance(cursor,str) and cursor
        new_json(out/'G1-managed-cursor.json',{'cursor':cursor,'scope':'G1 profile1 exact farvi page size1'})
        def capture_cursor(name,subset=False):
            q=call(name,['lookup','farvi','--page-size','1','--config',config])
            response=q['result']['response'];assert response['totals']['occurrences']==sum((subset_counts if subset else all_counts).values())
            saved=response['next_cursor'];assert isinstance(saved,str) and saved
            new_json(out/(name+'-saved.json'),{'cursor':saved,'scope':'fresh immediate-pre-transition managed page1 cursor'})
            return saved
        def old_cursor(name,stale,saved=cursor):
            q=call(name,['lookup','farvi','--page-size','1','--cursor',saved,'--config',config],expected_exit=1 if stale else 0)
            if stale:check.managed_error(q,'stale_cursor',config)
            else:
                r=q['result']['response'];assert r['totals']['occurrences']==sum(all_counts.values())
                full=json.loads((out/'G1-inner.json').read_bytes());ordered=[h['assertion']['id'] for group in full['groups'] for h in group['hits']]
                got=[h['assertion']['id'] for group in r['groups'] for h in group['hits']];assert got==ordered[1:2],(got,ordered)
        before = {str(root/'catalogue.sqlite3'):sha(root/'catalogue.sqlite3'),**immutable}
        paths_before=inventory([root])['entries']
        idem = call('update-identical',['collections','update',root,'--manifest',batch,'--all','--expected-revision','1',*allow],12*GIB)
        check.mutation(idem,'update','idempotent',1,IDS,digests,kind,[{'pack_id':p,'disposition':'validated','payload_kind':kind,'release_sha256':digests[p],'report':None} for p in IDS])
        assert inventory([root])['entries']==paths_before
        new_json(out/'U1-owned-paths-before-after.json',{'before':paths_before,'after':inventory([root])['entries']})
        assert idem['outcome']=='idempotent' and idem['local_revision']==1 and idem['generation']['generation_id']==g1['generation']['generation_id']
        assert before == {p:sha(p) for p in before}
        old_cursor('U1-cursor-fresh-process-valid',False)
        g2 = publication('G2-rebuild',['collections','rebuild',root,'--expected-revision','1'],2)
        read_phase('G2',g2['generation'],False,2)
        old_cursor('G2-prior-cursor-stale',True)
        old_store=check.safe_store(g1['generation']['path'],root,g1['attempt_id'])
        old_wire=stage('G2-fresh-oldG1-explicit-store',[cli,'dictionary-query',old_store,'--request',json.dumps({'operation':'exact','spelling':'farvi','options':{}})])
        assert old_wire.read_bytes()==(out/'G1-inner.json').read_bytes()
        g2_cursor=capture_cursor('G2-current-cursor-before-remove')
        g3 = publication('G3-remove-llg',['collections','remove',root,'--pack','llg','--expected-revision','2'],3,True)
        read_phase('G3',g3['generation'],True,3,True)
        old_cursor('G3-immediately-prior-cursor-stale-before-unavailable',True,g2_cursor)
        # Expected unavailable has no successful JSON stdout; error remains bounded stderr.
        stage('profile1-removed-unavailable',[cli,'lookup','farvi','--config',config],expected_exit=1)
        err = child_error(out/'profile1-removed-unavailable.stderr')
        assert 'removed' in check.managed_error(err,'unavailable_selection',config)['detail']
        off=call('profile2-disable',['collections','disable','--profile','full','--pack','llg','--config',config]);check.policy(off,'disable',2,config,root,IDS,['llg'],['llg'],3)
        check_lookup(call('profile2-subset',['lookup','farvi','--config',config]),True,subset_counts)
        g3_cursor=capture_cursor('G3-profile2-current-cursor-before-restore',True)
        g4 = publication('G4-exact-reinstall',['collections','reinstall',root,'--manifest',batch,'--pack','llg','--release-sha256',digests['llg'],'--expected-revision','3',*allow],4)
        read_phase('G4',g4['generation'],False,4)
        old_cursor('G4-immediately-prior-cursor-stale',True,g3_cursor)
        check_lookup(call('profile2-restored-disabled',['lookup','farvi','--config',config]),True,subset_counts)
        g4_cursor=capture_cursor('G4-profile2-current-cursor-before-enable',True)
        on=call('profile3-enable',['collections','enable','--profile','full','--pack','llg','--config',config]);check.policy(on,'enable',3,config,root,IDS,[],['llg'],4)
        old_cursor('profile3-prior-disabled-cursor-stale',True,g4_cursor)
        check_lookup(call('profile3-all',['lookup','farvi','--config',config]),counts=all_counts)
        clear=call('profile4-clear',['collections','default','--clear','--config',config]);check.policy(clear,'clear_default',4,config,root,[],[],[],None)
        stage('profile4-no-default',[cli,'lookup','farvi','--config',config],expected_exit=1)
        check.managed_error(child_error(out/'profile4-no-default.stderr'),'profile_default_missing',config)
        restore=call('profile5-restore',['collections','default','--profile','full','--config',config]);check.policy(restore,'set_default',5,config,root,IDS,[],[],4)
        check_lookup(call('profile5-all',['lookup','farvi','--config',config]),counts=all_counts)
        before=sha(root/'catalogue.sqlite3');reconcile=call('final-reconcile',['collections','reconcile',root]);check.reconcile(reconcile,root,generations,reports,digests)
        actual_stages=sorted(p.name for p in (root/'generations').iterdir())
        assert actual_stages==sorted(r['attempt_id'] for r in reports) and len(actual_stages)==4
        for r in reports:
            marker=json.loads((root/'generations'/r['attempt_id']/'attempt.json').read_bytes())
            assert marker=={'version':1,'attempt_id':r['attempt_id'],'operation':r['operation']}
        new_json(out/'actual-generations-and-attempts.json',{'stages':actual_stages,'reconciled':reconcile})
        assert sha(root/'catalogue.sqlite3')==before
        assert inputs_before == {p:sha(p) for p in inputs_before}
        for path, old in immutable.items(): assert sha(path)==old
        measure_resources('final')
        new_json(out/'summary.json',{'ok':True,'publications':publications,'generations':generations,'final_active':g4['generation']['generation_id'],'catalogue_revision':4,'profile_revision':5,'sealed_before_after':inputs_before,'immutable_derived_databases':immutable,'scope':'one root/four publications, final G4 active; no Full remove-all/changed-release/crash/held-reader scale claim; timeout active, RSS/disk/output thresholds postcompletion'})
        settled=inventory([root,out]);sidecars=[p for p in config.parent.iterdir() if p.name==config.name or p.name.startswith(config.name+'.')];assert sidecars==[config],sidecars
        settled['files'].append({'path':str(config),'bytes':config.stat().st_size});settled['bytes']+=config.stat().st_size
        assert settled['bytes']<=12*GIB
        new_json(out/'settled-inventory.json',{**settled,'unaccounted_later_bytes':'this settled inventory file only; exact size is separately available from filesystem metadata; no further product stages'})
        assert settled['bytes']+(out/'settled-inventory.json').stat().st_size<=12*GIB
        assert (out/'settled-inventory.json').stat().st_size<=128*1024**2
    except BaseException as error:
        new_json(out/'stopped.json',{'ok':False,'exception_type':type(error).__name__,'detail':str(error),'publication_attempts':publications,'retained':'all files/partial stages; no retry, deletion or reset'})
        raise

if __name__=='__main__':
    assert len(sys.argv)==2,'one concretely reviewed frozen preflight JSON is required'
    main(sys.argv[1])
