"""PROSPECTIVE finite S04 CLI caller; no execution until frozen source/binary and handback."""
import hashlib, importlib.util, json, os, sqlite3, subprocess, sys, time
from pathlib import Path
HERE=Path(__file__).parent
spec=importlib.util.spec_from_file_location('wire',HERE/'phase2-wire-checker01.py');wire=importlib.util.module_from_spec(spec);spec.loader.exec_module(wire)
IDS=['lensisku-jbotci','llg','la-lojban','sutysisku']
def sha(path):return hashlib.sha256(Path(path).read_bytes()).hexdigest()
def main():
    cli=Path(sys.argv[1]);fixture=Path(sys.argv[2]);output=Path(sys.argv[3]);output.mkdir();root=output/'cli lifecycle λ $()';config=output/'profiles.toml';manifest=fixture/'manifest.json';records=[]
    isolated={k:str(output/v) for k,v in [('HOME','isolated-home'),('XDG_DATA_HOME','isolated-data'),('XDG_CONFIG_HOME','isolated-config')]}
    for value in isolated.values():Path(value).mkdir(mode=0o700)
    (output/'child-environment.json').write_text(json.dumps(isolated,indent=2)+'\n')
    binary_sha=sha(cli);input_shas={str(p):sha(p) for p in sorted((fixture/'inputs').rglob('*')) if p.is_file()}
    def run(name,args,code=0,env=None):
        argv=[str(cli),*map(str,args)];start=time.monotonic();r=subprocess.run(argv,env={**isolated,**(env or {})},capture_output=True,timeout=60)
        (output/(name+'.stdout')).write_bytes(r.stdout);(output/(name+'.stderr')).write_bytes(r.stderr)
        records.append(dict(name=name,argv=argv,expected_exit=code,actual_exit=r.returncode,elapsed_s=time.monotonic()-start,stdout_sha256=hashlib.sha256(r.stdout).hexdigest(),stderr_sha256=hashlib.sha256(r.stderr).hexdigest()))
        (output/'receipts.json').write_text(json.dumps(records,indent=2)+'\n');assert r.returncode==code,(name,r.returncode,r.stderr)
        return json.loads(r.stdout if code==0 else r.stderr)
    def lookup(name,occ,groups):
        q=run(name,['lookup','farvi','--config',config]);q=q['result']['response'];assert q['totals']['occurrences']==occ and len(q['groups'])==groups
        expected={'lensisku':2,'llg':1,'sutysisku':2} if occ==5 else {'lensisku':2,'sutysisku':2}
        actual={x['source']['id']:x['occurrences'] for x in q['source_totals'] if x['occurrences']}
        assert actual==expected,(name,actual)
    legacy=json.loads((fixture/'legacy-store.json').read_text());legacy_store=Path(legacy['store']);legacy_env={'XDG_DATA_HOME':legacy['xdg_data_home']};legacy_db=legacy_store/'dictionary.sqlite3';assert sha(legacy_db)==legacy['database_sha256']
    exact=json.dumps({'operation':'exact','spelling':'farvi','options':{}},separators=(',',':'))
    def legacy_pair(label):
        explicit=run('legacy-explicit-'+label,['dictionary-query',legacy_store,'--request',exact])
        default=run('legacy-xdg-'+label,['dictionary-query','--request',exact],env=legacy_env)
        assert explicit==default and explicit['totals']['occurrences']==1 and explicit['totals']['source_groups']==1
        assert (output/('legacy-explicit-'+label+'.stdout')).read_bytes()==(output/('legacy-xdg-'+label+'.stdout')).read_bytes()
        assert sha(legacy_db)==legacy['database_sha256']
        return (output/('legacy-explicit-'+label+'.stdout')).read_bytes()
    legacy_wire=legacy_pair('before')
    fixture_llg_digest=sha(fixture/'inputs/llg/release.json')
    missing=wire.failure(run('reinstall-missing-root',['collections','reinstall',output/'absent catalogue','--manifest',manifest,'--pack','llg','--release-sha256',fixture_llg_digest,'--allow-synthetic','--expected-revision','0'],1),'reinstall','missing_catalogue','not_activated');assert missing['prior_local_revision'] is None and missing['attempt_id'] is None and len(missing['inputs'])==4 and next(x for x in missing['inputs'] if x['pack_id']=='llg')['release_sha256']==fixture_llg_digest
    first=run('install',['collections','install',root,'--manifest',manifest,'--all','--allow-synthetic']);assert first['local_revision']==1
    lock=sqlite3.connect(root/'catalogue.sqlite3');lock.execute('BEGIN IMMEDIATE')
    try:
        busy=wire.failure(run('reinstall-begin-busy',['collections','reinstall',root,'--manifest',manifest,'--pack','llg','--release-sha256',fixture_llg_digest,'--allow-synthetic','--expected-revision','1'],1),'reinstall','busy','not_activated');assert busy['prior_local_revision'] is None and busy['attempt_id'] is None and len(busy['inputs'])==4 and next(x for x in busy['inputs'] if x['pack_id']=='llg')['release_sha256']==fixture_llg_digest
    finally:lock.rollback();lock.close()
    same=run('reinstall-after-lock',['collections','reinstall',root,'--manifest',manifest,'--pack','llg','--release-sha256',fixture_llg_digest,'--allow-synthetic','--expected-revision','1']);assert same['outcome']=='idempotent' and same['local_revision']==1 and same['generation']['generation_id']==first['generation']['generation_id']
    configure=run('configure',['collections','configure',root,'--profile','all','--all','--make-default','--config',config]);assert configure['revision']==1
    lookup('all-before',5,3)
    before=sha(root/'catalogue.sqlite3')
    stale=wire.failure(run('remove-stale',['collections','remove',root,'--pack','llg','--expected-revision','0'],1),'remove','revision_conflict','not_activated');assert stale['prior_local_revision']==1 and stale['attempt_id'] is None and stale['proposed_generation'] is None;assert sha(root/'catalogue.sqlite3')==before
    removed=run('remove-llg',['collections','remove',root,'--pack','llg','--expected-revision','1']);assert removed['local_revision']==2 and removed['generation'] is not None
    unavailable=run('lookup-removed',['lookup','farvi','--config',config],1);assert unavailable['error']['category']=='unavailable_selection'
    inspect=run('inspect-removed',['collections','inspect',root,'llg']);assert inspect['installation_state']=='removed' and inspect['installed_release_sha256'] is None
    off=wire.policy(run('disable-llg',['collections','disable','--profile','all','--pack','llg','--config',config]),'disable','committed');assert off['affected_pack_ids']==['llg'] and off['revision']==2
    lookup('disabled-remaining',4,2)
    release=fixture/'inputs/llg/release.json';seal=fixture/'inputs/llg/seal.json';digest=sha(release);before=sha(root/'catalogue.sqlite3')
    unknown=wire.failure(run('reinstall-unknown',['collections','reinstall',root,'--manifest',manifest,'--pack','llg','--release-sha256','0'*64,'--allow-synthetic','--expected-revision','2'],1),'reinstall','unknown_registered_release','not_activated');assert unknown['pack_id']=='llg' and unknown['prior_local_revision']==2 and unknown['observed_generation']==removed['generation']['generation_id'] and unknown['attempt_id'] is None and unknown['proposed_generation'] is None;assert len(unknown['inputs'])==4;llg_input=next(x for x in unknown['inputs'] if x['pack_id']=='llg');assert llg_input['disposition']=='validated' and llg_input['payload_kind']=='synthetic' and llg_input['release_sha256']==digest;assert {x['pack_id'] for x in unknown['inputs'] if x['disposition']=='skipped_without_io'}==set(IDS)-{'llg'};assert sha(root/'catalogue.sqlite3')==before
    restored=run('reinstall-llg',['collections','reinstall',root,'--manifest',manifest,'--pack','llg','--release-sha256',digest,'--allow-synthetic','--expected-revision','2']);assert restored['local_revision']==3 and restored['generation']['content_id']==first['generation']['content_id'] and restored['generation']['generation_id']!=first['generation']['generation_id']
    lookup('restored-still-disabled',4,2)
    on=wire.policy(run('enable-llg',['collections','enable','--profile','all','--pack','llg','--config',config]),'enable','committed');assert on['affected_pack_ids']==['llg'] and on['revision']==3
    lookup('restored-enabled',5,3)
    original=config.read_bytes();no=wire.policy(run('enable-noop',['collections','enable','--profile','all','--all','--config',config]),'enable','unchanged');assert no['affected_pack_ids']==[] and no['revision']==3 and config.read_bytes()==original
    clear=wire.policy(run('default-clear',['collections','default','--clear','--config',config]),'clear_default','committed');assert clear['profile'] is None and clear['root'] is None and clear['affected_pack_ids']==[]
    profiles=run('profiles-cleared',['collections','profiles','--config',config]);assert profiles['default_profile'] is None
    chosen=wire.policy(run('default-set',['collections','default','--profile','all','--config',config]),'set_default','committed');assert chosen['affected_pack_ids']==[] and chosen['default_profile']=='all'
    lookup('default-restored',5,3)
    # Actual receipt-loss transport: close pipe before response; inspect actual committed null state.
    argv=[str(cli),'collections','remove',str(root),'--expected-revision','3'];argv.extend(a for id in reversed(IDS) for a in ['--pack',id]);start=time.monotonic()
    stderr=(output/'remove-all-brokenpipe.stderr').open('xb');child=subprocess.Popen(argv,env=isolated,stdout=subprocess.PIPE,stderr=stderr);child.stdout.close();code=child.wait(timeout=60);stderr.close();assert code==0
    records.append(dict(name='remove-all-brokenpipe',argv=argv,actual_exit=code,elapsed_s=time.monotonic()-start,scope='actual closed stdout + committed-state readback; BrokenPipe intentionally success, not non-BrokenPipe error-envelope proof'));(output/'receipts.json').write_text(json.dumps(records,indent=2)+'\n')
    listed=run('list-null',['collections','list',root]);assert listed['local_revision']==4 and listed['generation'] is None
    before=sha(root/'catalogue.sqlite3');observation=run('reconcile',['collections','reconcile',root]);assert observation['active_generation'] is None and observation['local_revision']==4 and observation['verification_scope']=='minimum_header_only';assert sha(root/'catalogue.sqlite3')==before
    assert legacy_pair('after')==legacy_wire
    assert input_shas=={str(p):sha(p) for p in sorted((fixture/'inputs').rglob('*')) if p.is_file()};assert sha(cli)==binary_sha
    summary=dict(ok=True,scope='finite independent tiny CLI lifecycle/policy/null/readback + original standalone explicit/scratch-XDG default before/after; no real default/Full/Linux/process-crash or non-BrokenPipe proof',processes=len(records),cli_sha256=binary_sha,external_input_sha256=input_shas)
    (output/'summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps(summary))
if __name__=='__main__':main()
