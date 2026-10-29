"""UNISSUED source-only fixed post-journey completion: same Tiny ROOT only."""
import hashlib, importlib.util, json, os, stat, sys
from pathlib import Path
HERE=Path(__file__).parent
s=importlib.util.spec_from_file_location('readable',HERE/'readable-cli-checks01.py')
m=importlib.util.module_from_spec(s);s.loader.exec_module(m)

def owned_tree(root):
    assert root.is_absolute() and '..' not in root.parts
    assert stat.S_ISDIR(root.lstat().st_mode) and not root.is_symlink()
    stack=[root];out={}
    while stack:
        d=stack.pop()
        for p in sorted(d.iterdir()):
            mode=p.lstat().st_mode;rel=str(p.relative_to(root))
            assert not stat.S_ISLNK(mode),(p,'symlink')
            if stat.S_ISDIR(mode):out[rel]={'kind':'directory'};stack.append(p)
            else:
                assert stat.S_ISREG(mode),(p,'nonregular')
                data=p.read_bytes();out[rel]={'kind':'file','bytes':len(data),'sha256':hashlib.sha256(data).hexdigest()}
    return out

def main():
    binary,context_file,cli=map(Path,sys.argv[1:]);assert all(p.is_absolute() for p in (binary,context_file,cli))
    ctx=m.prior.strict_json(context_file.read_bytes());assert ctx['fixture_scope']=='original-four-pack-synthetic-readable'
    root=cli/'root';config=cli/'config.toml';manifest=cli/'manifest.json';out=cli/'post-journey';out.mkdir(mode=0o700)
    env={'PATH':os.defpath,'LANG':'C.UTF-8'}
    for key in ('HOME','XDG_CONFIG_HOME','XDG_DATA_HOME'):
        p=cli/key.lower();assert p.is_dir() and not p.is_symlink();env[key]=str(p)
    tmp=out/'tmp';tmp.mkdir(mode=0o700);env['TMPDIR']=str(tmp)
    def run(name,args):return m.prior.run(binary,args,env,out,name)
    def wire(name,args):
        r=run(name,args);assert not r.stderr;return m.prior.strict_json(r.stdout)
    assert stat.S_ISDIR(root.lstat().st_mode) and not root.is_symlink()
    install=m.prior.strict_json((cli/'W1-install.stdout').read_bytes());g1=m.path_report(install['generation']['path'])
    g1.relative_to(root);assert '..' not in g1.parts
    walk=root
    for part in g1.relative_to(root).parts:
        walk=walk/part;assert stat.S_ISDIR(walk.lstat().st_mode) and not walk.is_symlink()
    initial=wire('before-list',['collections','list',root]);assert initial['local_revision']==3
    p=wire('before-profiles',['collections','profiles','--config',config]);assert p['revision']==3 and p['default_profile']=='work'
    before=owned_tree(root);g1_before=owned_tree(g1);config_before=config.read_bytes()
    unchanged=wire('identical-update',['collections','update',root,'--manifest',manifest,'--all','--allow-synthetic','--expected-revision','3'])
    assert set(unchanged)==m.MUTATION and unchanged['version']==1 and unchanged['ok'] is True
    assert (unchanged['operation'],unchanged['outcome'],unchanged['local_revision'],unchanged['attempt_id'])==('update','idempotent',3,None)
    restored=m.prior.strict_json((cli/'W4-reinstall.stdout').read_bytes())
    assert unchanged['generation']==restored['generation']
    assert owned_tree(root)==before and config.read_bytes()==config_before
    old_args=['dictionary-query',g1,'--request',m.j({'operation':'exact','spelling':'farvi'})]
    old=run('old-g1-before',[*old_args,'--format','readable']);assert not old.stderr
    assert old.stdout==(cli/'W5-retained-old-store.stdout').read_bytes()
    rebuilt=m.mutation(wire('rebuild',['collections','rebuild',root,'--expected-revision','3']),'rebuild','rebuilt',4)
    assert rebuilt['generation']['generation_id'] not in (install['generation']['generation_id'],restored['generation']['generation_id'])
    now=m.path_report(rebuilt['generation']['path']);now.relative_to(root);assert '..' not in now.parts and now!=g1
    walk=root
    for part in now.relative_to(root).parts:
        walk=walk/part;assert stat.S_ISDIR(walk.lstat().st_mode) and not walk.is_symlink()
    assert owned_tree(g1)==g1_before and config.read_bytes()==config_before
    final=wire('after-list',['collections','list',root]);assert final['local_revision']==4 and final['staged_attempts']==[]
    p=wire('after-profiles',['collections','profiles','--config',config]);assert p['revision']==3 and p['default_profile']=='work'
    args=['lookup','farvi','--config',config,'--profile','work']
    compact=wire('rebuilt-compact',[*args,'--format','compact']);m.prior.compact_exact_wire(compact['result']['response'])
    snapshot=compact['snapshot'];assert snapshot['local_revision']==4 and snapshot['profile']['revision']==3 and snapshot['profile']['name']=='work'
    assert snapshot['generation_id']==rebuilt['generation']['generation_id'] and snapshot['packs']==m.PACKS and snapshot['disabled']==[]
    response=compact['result']['response'];ids=ctx['ordered_lexical_assertion_ids']
    assert response['totals']=={'occurrences':5,'distinct_spellings':1,'source_groups':3}
    assert [g['source']['id'] for g in response['groups']]==['lensisku','llg','sutysisku']
    assert [h['assertion']['id'] for g in response['groups'] for h in g['hits']]==ids
    for group in response['groups']:
        for hit in group['hits']:
            assert hit['detail_ref']==ctx['expected_keys'][hit['assertion']['id']]
            assert hit['assertion']==ctx['expected_assertions'][hit['assertion']['id']]
            assert hit['entry']==ctx['expected_entries'][hit['entry']['id']]
    text=run('rebuilt-readable',[*args,'--format','readable']);assert not text.stderr
    m.readable_exact(text.stdout,ctx,ctx['ordered_lexical_assertion_ids'],compact)
    old_after=run('old-g1-after',[*old_args,'--format','readable']);assert old_after.stdout==old.stdout and not old_after.stderr
    rec=wire('reconcile',['collections','reconcile',root]);assert rec['attention_required'] is False
    result={'ok':True,'scope':'same original CLI root update3/3 unchanged then rebuild4/3; oldG1 preserved','root_before':before,'g1_before_after':g1_before,'new_generation':rebuilt['generation']}
    with (out/'summary.json').open('x') as f:json.dump(result,f,indent=2);f.write('\n')
    print(json.dumps({'ok':True,'catalogue_revision':4,'profile_revision':3}))
if __name__=='__main__':main()
