#!/usr/bin/env python3
"""DRAFT independent S02 tiny CLI control; execute only after target handback."""
import argparse, hashlib, json, os, pathlib, subprocess

def sha(b): return hashlib.sha256(b).hexdigest()

def main():
    p=argparse.ArgumentParser();p.add_argument('--binary',required=True);p.add_argument('--inventory-binary',required=True);p.add_argument('--scratch',required=True);a=p.parse_args()
    root=pathlib.Path(a.scratch);root.mkdir(mode=0o700)
    inputs=root/'local inputs λ ; literal';inputs.mkdir(mode=0o700)
    fixtures=pathlib.Path(__file__).parent/'public-caller01/fixtures'
    lens=(fixtures/'lensisku.json').read_bytes();llg=(fixtures/'llg-valid-row.txt').read_bytes();sut=(fixtures/'sutysisku.json').read_bytes()
    assert sha(lens)=='63fb2e10e80f34e6d7f04fbe9bcfe1cd69c7ec0c4016045d02e8b3cfcd866429'
    assert sha(llg)=='0408fd8bf1adae9d0da1e7faabb35a6c2cdf64d012140bcadfdd814f061b2735'
    assert sha(sut)=='3e7738c9412a4dfb6169e8492f8ecb7fd25003d2141dbb8fa11782e580d00b03'
    notice=b'Explicit synthetic notice'
    meta=('language_tag = "en"\nlanguage_realname = "English"\nsource_language_tag = "jbo"\nformat = "json"\npositive_scores_only = false\nfilename = "dictionary-en.json"\nmetadata_url = "https://example.invalid/languages"\ndownload_url = "https://example.invalid/synthetic"\nlensisku_created_at = "2026-10-04T03:07:08Z"\nsha256 = "'+sha(lens)+'"\ndefinition_count = 1\nentry_count = 1\n').encode()
    for name,b in [('lensisku.json',lens),('llg.txt',llg),('empty.txt',b''),('notice.txt',notice),('sutysisku.json',sut),('metadata.toml',meta)]:
        with (inputs/name).open('xb') as f:f.write(b)
    revision='0'*40
    records=[
      {'kind':'jbotci','pack_id':'lensisku-jbotci','payload_kind':'synthetic','raw':'lensisku.json','metadata':'metadata.toml','options':{'revision':revision,'expected_raw_sha256':sha(lens),'expected_metadata_sha256':sha(meta)}},
      {'kind':'llg','pack_id':'llg','payload_kind':'synthetic','gismu':'llg.txt','cmavo':'empty.txt','rafsi':'empty.txt','notice':'notice.txt','options':{'expected_gismu_sha256':sha(llg),'expected_cmavo_sha256':sha(b''),'expected_rafsi_sha256':sha(b''),'expected_notice_sha256':sha(notice),'base_locator':'https://example.invalid/llg'}},
      {'kind':'la-lojban','pack_id':'la-lojban','payload_kind':'synthetic','raw':'lensisku.json','options':{'revision':revision,'expected_raw_sha256':sha(lens),'raw_locator':'https://example.invalid/la','notice_locator':'https://example.invalid/code-notice'}},
      {'kind':'sutysisku','pack_id':'sutysisku','payload_kind':'synthetic','raw':'sutysisku.json','options':{'revision':revision,'expected_raw_sha256':sha(sut),'raw_locator':'https://example.invalid/sut','notice_locator':'https://example.invalid/code-notice'}}]
    manifest=inputs/'batch.json';manifest.write_text(json.dumps({'version':1,'inputs':records},ensure_ascii=False)+'\n')
    environment=dict(os.environ,HOME=str(root/'isolated-home'),XDG_DATA_HOME=str(root/'isolated-data'))
    receipts=[]
    def run(label,args,ok=True,stdin=None):
        argv=[a.binary,*map(str,args)];r=subprocess.run(argv,input=stdin,stdout=subprocess.PIPE,stderr=subprocess.PIPE,env=environment,cwd=root,timeout=60)
        (root/(label+'.stdout')).write_bytes(r.stdout);(root/(label+'.stderr')).write_bytes(r.stderr)
        receipts.append({'label':label,'argv':argv,'exit':r.returncode,'stdout_sha256':sha(r.stdout),'stderr_sha256':sha(r.stderr)})
        (root/'attempt-receipts.json').write_text(json.dumps(receipts,indent=2)+'\n')
        assert (r.returncode==0)==ok,(label,r.returncode,r.stderr[:2000])
        payload=json.loads(r.stdout if ok else r.stderr)
        if not ok:
            assert set(payload)=={'error'},payload
            return payload['error']
        return payload
    catalogue=root/'all-four'
    installed=run('install-all',['collections','install',catalogue,'--manifest',manifest,'--all','--allow-synthetic'])
    assert installed['version']==1 and installed['ok'] is True and installed['local_revision']==1
    assert len(installed['generation']['members'])==4
    store=pathlib.Path(installed['generation']['path']['utf8'])
    query=run('farvi-all',['dictionary-query',store,'--request',json.dumps({'operation':'exact','spelling':'farvi'})])
    assert query['totals']=={'occurrences':5,'distinct_spellings':1,'source_groups':3},query.get('totals')
    assert {s['source']['id']:s['occurrences'] for s in query['source_totals']}=={'lensisku':2,'llg':1,'sutysisku':2}
    inv_argv=[a.inventory_binary,'--inspect-raw-catalogue',str(catalogue)]
    inv=subprocess.run(inv_argv,stdout=subprocess.PIPE,stderr=subprocess.PIPE,env=environment,cwd=root,timeout=60)
    (root/'raw-inventory.stdout').write_bytes(inv.stdout);(root/'raw-inventory.stderr').write_bytes(inv.stderr)
    receipts.append({'label':'raw-inventory','argv':inv_argv,'exit':inv.returncode,'stdout_sha256':sha(inv.stdout),'stderr_sha256':sha(inv.stderr)})
    (root/'attempt-receipts.json').write_text(json.dumps(receipts,indent=2)+'\n')
    assert inv.returncode==0,(inv.returncode,inv.stderr)
    inv_report=json.loads(inv.stdout);assert inv_report['ok'] is True and inv_report['categories']==25
    assert inv_report['db_sha256_before']==inv_report['db_sha256_after']
    # Actual importer accounting mismatch with BOTH input pins truthful: this
    # rejects supplier-count mismatch rather than merely failing a stale hash.
    bad_meta=meta.replace(b'definition_count = 1',b'definition_count = 2')
    (inputs/'supplier-metadata.toml').write_bytes(bad_meta)
    bad_records=json.loads(json.dumps(records));bad_records[0]['metadata']='supplier-metadata.toml';bad_records[0]['options']['expected_metadata_sha256']=sha(bad_meta)
    bad_manifest=inputs/'supplier-count.json';bad_manifest.write_text(json.dumps({'version':1,'inputs':bad_records})+'\n')
    prior_store_sha=sha((store/'dictionary.sqlite3').read_bytes())
    failed_count=run('supplier-count',['collections','install',catalogue,'--manifest',bad_manifest,'--all','--allow-synthetic'],False)
    assert failed_count['category']=='input_invalid' and failed_count['pack_id']=='lensisku-jbotci' and failed_count['activation']=='not_activated'
    assert 'supplier counts' in failed_count['detail']
    assert [x['pack_id'] for x in failed_count['inputs']]==['lensisku-jbotci','llg','la-lojban','sutysisku']
    assert [x['disposition'] for x in failed_count['inputs']]==['failed','unprocessed','unprocessed','unprocessed']
    assert sha((store/'dictionary.sqlite3').read_bytes())==prior_store_sha
    retained=run('supplier-count-retained',['collections','list',catalogue]);assert retained['local_revision']==1 and retained['generation']['generation_id']==installed['generation']['generation_id']
    listed=run('list',['collections','list',catalogue]);assert len(listed['packs'])==4
    for pack in ['lensisku-jbotci','llg','la-lojban','sutysisku']:
        run('inspect-'+pack,['collections','inspect',catalogue,pack])
    # Same structurally complete manifest, but one selected input missing. The
    # jbotci-only counterpart must not read that unselected path or validate its
    # intentionally bad semantic pin; all-four must fail before activation.
    records[-1]['raw']='absent-sut.json';records[-1]['options']['expected_raw_sha256']='bad-semantic-pin'
    missing=inputs/'selected-only.json';missing.write_text(json.dumps({'version':1,'inputs':records})+'\n')
    selected=root/'selected-only'
    selected_report=run('selected-no-io',['collections','install',selected,'--manifest',missing,'--pack','lensisku-jbotci','--allow-synthetic'])
    expected_packs=['lensisku-jbotci','llg','la-lojban','sutysisku']
    assert [r['pack_id'] for r in selected_report['inputs']]==expected_packs
    assert [r['disposition'] for r in selected_report['inputs']]==['installed','skipped_without_io','skipped_without_io','skipped_without_io']
    assert selected_report['inputs'][0]['release_sha256'] is not None
    failed=run('all-selected-missing',['collections','install',selected,'--manifest',missing,'--all','--allow-synthetic'],False)
    assert failed['version']==1 and failed['ok'] is False and failed['activation']=='not_activated'
    assert failed['category']=='input_io',failed
    assert [r['pack_id'] for r in failed['inputs']]==expected_packs
    assert [r['disposition'] for r in failed['inputs']]==['validated','validated','validated','failed']
    assert all(r['release_sha256'] is not None and r['payload_kind']=='synthetic' for r in failed['inputs'][:3])
    again=run('selected-still-one',['collections','list',selected]);assert again['local_revision']==1 and len(again['generation']['members'])==1
    # No successful mutation receipt is read. The pipe's read end is closed;
    # the separately reopened catalogue, not the lost stdout, proves commit.
    lost=root/'lost-output-catalogue'
    argv=[a.binary,'collections','install',str(lost),'--manifest',str(manifest),'--pack','llg','--allow-synthetic']
    child=subprocess.Popen(argv,stdout=subprocess.PIPE,stderr=subprocess.PIPE,env=environment,cwd=root)
    child.stdout.close();child.stdout=None
    try:
        _,stderr=child.communicate(timeout=30)
    except subprocess.TimeoutExpired:
        child.kill();_,stderr=child.communicate()
        (root/'lost-output-timeout.stderr').write_bytes(stderr)
        raise
    code=child.returncode
    (root/'lost-output.stderr').write_bytes(stderr)
    receipts.append({'label':'lost-output-install','argv':argv,'exit':code,'stdout_read_end_closed':True,'stderr_sha256':sha(stderr)})
    assert code==0,(code,stderr)
    committed=run('lost-output-committed',['collections','list',lost])
    assert committed['local_revision']==1 and len(committed['generation']['members'])==1
    restored=run('lost-output-query',['dictionary-query',committed['generation']['path']['utf8'],'--request',json.dumps({'operation':'exact','spelling':'farvi'})])
    assert restored['totals']['occurrences']==1 and restored['totals']['source_groups']==1
    absent=root/'not-created'
    error=run('missing-read-only',['collections','list',absent],False)
    assert error['version']==1 and error['ok'] is False and error['activation']=='not_applicable' and error['observed_generation'] is None
    assert not absent.exists()
    output={'ok':True,'scope':'independent tiny CLI raw all-four and selected-only controls','processes':len(receipts),'receipts':receipts,'full_installations':0}
    (root/'summary.json').write_text(json.dumps(output,indent=2)+'\n');print(json.dumps(output))
if __name__=='__main__':main()
