from pathlib import Path
import json,os,subprocess,hashlib,shutil,sqlite3,sys,time,copy
R=Path('/tmp/vlajmaji-store-verification-cdc');W=R/'tiny-controls01';W.mkdir();C=Path(sys.argv[1]);SOURCE=sys.argv[2];B=Path('/tmp/vlajmaji-cli-default-store-cdc/candidate02-release-cli');F=Path('/tmp/vlajmaji-cli-default-store-cdc/tiny-release.json');records=[];checks=0
Q='{"operation":"exact","spelling":"farvi"}';NAMES=['schema','metadata','hashes','model','fks','projection'];base=os.environ.copy();base.pop('HOME',None);base.pop('XDG_DATA_HOME',None)
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def need(ok,label):
 global checks
 checks+=1
 if not ok:raise AssertionError(label)
def run(label,args,code=0,xdg=None,home=None,binary=C,json_output=True):
 env=base.copy()
 if xdg is not None:env['XDG_DATA_HOME']=str(xdg)
 if home is not None:env['HOME']=str(home)
 t=time.monotonic();p=subprocess.Popen([str(binary),*map(str,args)],cwd=W,env=env,stdin=subprocess.DEVNULL,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
 out,err=p.communicate(timeout=60)
 (W/(label+'.stdout')).write_bytes(out);(W/(label+'.stderr')).write_bytes(err)
 records.append({'label':label,'argv':list(map(str,args)),'pid':p.pid,'exit':p.returncode,'expected_exit':code,'elapsed_s':time.monotonic()-t,'HOME':None if home is None else str(home),'XDG_DATA_HOME':None if xdg is None else str(xdg),'stdout_bytes':len(out),'stdout_sha256':hashlib.sha256(out).hexdigest(),'stderr_bytes':len(err),'stderr_sha256':hashlib.sha256(err).hexdigest()})
 (R/'tiny-controls-progress01.json').write_text(json.dumps({'records':records,'checks':checks},indent=2)+'\n')
 need(p.returncode==code,(label,p.returncode,err));need(not (err if code==0 else out),label+' correct output stream')
 return json.loads(out if code==0 else err) if json_output else (out if code==0 else err)
def result(label,check,path=None,ok=True,xdg=None,home=None):
 raw=run(label,['verify',check,*([] if path is None else [path])],0 if ok else 1,xdg,home)
 report=raw if ok else raw['verification']
 if not ok:need(raw['error']['code']=='verification_failed',label+' typed verification failure')
 need(report['operation']=='verify' and report['version']==1 and report['ok']==ok,label+' report identity')
 rows=report['checks'];got={r['check']:r['status'] for r in rows};need(len(got)==len(rows),label+' no duplicate result')
 need(list(got)==(NAMES if check=='all' else [check]),label+' complete ordered scope')
 need(all(s in ['passed','failed','blocked'] for s in got.values()),label+' status vocabulary')
 need((all(s=='passed' for s in got.values()))==ok,label+' ok exact implication')
 return got
def mutate(path,sql=None,change=None):
 db=path/'dictionary.sqlite3';c=sqlite3.connect(db)
 if sql:c.executescript(sql)
 if change:
  b=c.execute('SELECT json FROM canonical_document WHERE id=1').fetchone()[0];d=json.loads(b);change(d);b=json.dumps(d,separators=(',',':'),ensure_ascii=False).encode();c.execute('UPDATE canonical_document SET json=? WHERE id=1',(b,));c.execute("UPDATE metadata SET value=? WHERE key='canonical_sha256'",(hashlib.sha256(b).hexdigest(),))
 c.commit();c.close()
# One real complete tiny build. Same valid query and info bytes as the baseline.
p=W/'base';run('build',['dictionary-build',F,p]);db=p/'dictionary.sqlite3';original=sha(db)
for name in NAMES+['all']:result('valid-'+name,name,p)
newq=run('valid-query',['dictionary-query',p,'--request',Q]);oldq=run('old-query',['dictionary-query',p,'--request',Q],binary=B);need(newq==oldq,'whole valid query unchanged')
newi=run('valid-info',['info',p]);oldi=run('old-info',['info',p],binary=B);need(newi==oldi,'whole valid info unchanged')
need(sha(db)==original,'valid operations readonly')
# Independent one-defect and dependency controls; no full payload involved.
cases=[
 ('schema','CREATE TABLE cdc_extra(value TEXT);',None,'schema',False,{'schema':'failed','model':'passed','fks':'passed'}),
 ('metadata',"UPDATE metadata SET value='wrong-family' WHERE key='format';",None,'metadata',True,{'metadata':'failed','schema':'passed','model':'passed','fks':'passed'}),
 ('unsupported-dictionary-version',"UPDATE metadata SET value='99' WHERE key='dictionary_version';",None,'metadata',True,{'metadata':'failed','schema':'passed','model':'passed','fks':'passed'}),
 ('unsupported-registry-version',"UPDATE metadata SET value='99' WHERE key='registry_version';",None,'metadata',True,{'metadata':'failed','schema':'passed','model':'passed','fks':'passed'}),
 ('hashes',"UPDATE metadata SET value=printf('%064d',0) WHERE key='canonical_sha256';",None,'hashes',False,{'hashes':'failed','model':'passed','fks':'passed'}),
 ('fks',"PRAGMA foreign_keys=OFF;INSERT INTO source_aliases(ordinal,source_id,text,status) VALUES(900,'cdc-absent','cdc-alias','Active');",None,'fks',False,{'fks':'failed','model':'passed','projection':'failed'}),
 ('projection',"UPDATE entries SET spelling_value='cdc-tampered' WHERE id=(SELECT id FROM entries LIMIT 1);",None,'projection',False,{'projection':'failed','schema':'passed','hashes':'passed','model':'passed','fks':'passed'}),
 ('model',None,lambda d:d['metadata']['assertions'][0].update(entry='cdc-missing-entry'),'model',False,{'model':'failed','projection':'blocked','schema':'passed','fks':'passed'}),
 ('registry-model',None,lambda d:d['registry']['sources'][0].update(abbreviation_kind='Conventional',abbreviation_evidence=None),'model',False,{'model':'failed','projection':'blocked','schema':'passed','fks':'passed'}),
 ('capture-hash',None,lambda d:d['captures'][0]['bytes'].__setitem__(0,d['captures'][0]['bytes'][0]^1),'hashes',False,{'hashes':'failed','model':'failed','projection':'blocked','fks':'passed'}),
 ('duplicate-entry-id',None,lambda d:d['metadata']['entries'].append(copy.deepcopy(d['metadata']['entries'][0])),'model',False,{'model':'failed','projection':'blocked'}),
 ('duplicate-assertion-id',None,lambda d:d['metadata']['assertions'].append(copy.deepcopy(d['metadata']['assertions'][0])),'model',False,{'model':'failed','projection':'blocked'}),
 ('duplicate-source-id',None,lambda d:d['registry']['sources'].append(copy.deepcopy(d['registry']['sources'][0])),'model',False,{'model':'failed','projection':'blocked'}),
 ('duplicate-capture',None,lambda d:d['captures'].append(copy.deepcopy(next(c for c in d['captures'] if c['occurrences']))),'model',False,{'model':'failed','projection':'blocked'}),
 ('duplicate-occurrence',None,lambda d:next(c for c in d['captures'] if c['occurrences'])['occurrences'].append(copy.deepcopy(next(c for c in d['captures'] if c['occurrences'])['occurrences'][0])),'model',False,{'model':'failed','projection':'blocked'}),
 ('native-duplicate-preserved',None,lambda d:d['metadata']['extensions'][0].update(value='{"cdc_native":1,"cdc_native":2}'),'projection',False,{'model':'passed','hashes':'passed','projection':'failed','schema':'passed','fks':'passed'}),
 ('legacy-column',"UPDATE canonical_document SET legacy_archive=x'6e6f742d616e2d61726368697665' WHERE id=1;",None,'hashes',False,{'hashes':'failed','model':'passed','projection':'failed'}),
 ('missing-metadata','DROP TABLE metadata;',None,'schema',True,{'schema':'failed','metadata':'failed','model':'passed','fks':'passed'}),
]
for name,sql,change,target,fast_fails,wanted in cases:
 path=W/name;shutil.copytree(p,path);mutate(path,sql,change);before=sha(path/'dictionary.sqlite3')
 run(name+'-fast-info',['info',path],1 if fast_fails else 0)
 if name=='model':
  failure=run('model-fast-query-runtime',['dictionary-query',path,'--request',Q],1);need(failure['error']['code']=='integrity','query uses draft and selected-reference runtime error instead of strict startup model validation')
 if name in ['capture-hash','registry-model']:
  run(name+'-fast-query-unverified',['dictionary-query',path,'--request',Q]);need(True,name+' fast query does not perform full hash/registry audit')
 if name.startswith('duplicate-'):
  failure=run(name+'-fast-query-ambiguity',['dictionary-query',path,'--request',Q],1);need(isinstance(failure['error']['code'],str) and bool(failure['error']['code']),name+' typed ambiguity instead of silently choosing one')
 got=result(name+'-selective',target,path,False);need(got[target]=='failed',name+' direct failure')
 got=result(name+'-all','all',path,False)
 for check,status in wanted.items():need(got[check]==status,(name,check,status,got))
 need(sha(path/'dictionary.sqlite3')==before,name+' readonly')
# A true duplicate normalized-envelope key differs from duplicate native JSON inside a quoted string.
path=W/'duplicate-envelope';shutil.copytree(p,path);c=sqlite3.connect(path/'dictionary.sqlite3');b=c.execute('SELECT json FROM canonical_document WHERE id=1').fetchone()[0];need(b.startswith(b'{'),'canonical root object');b=b'{"version":1,'+b[1:];c.execute('UPDATE canonical_document SET json=? WHERE id=1',(b,));c.execute("UPDATE metadata SET value=? WHERE key='canonical_sha256'",(hashlib.sha256(b).hexdigest(),));c.commit();c.close();before=sha(path/'dictionary.sqlite3');run('duplicate-envelope-fast-info',['info',path]);got=result('duplicate-envelope-all','all',path,False);need(got['model']=='failed' and got['projection']=='blocked','true envelope duplicate rejected by strict model');need(sha(path/'dictionary.sqlite3')==before,'duplicate envelope readonly')
# Broken canonical bytes: normal info is SQL-only; model fails and projection is blocked.
path=W/'malformed-canonical';shutil.copytree(p,path);c=sqlite3.connect(path/'dictionary.sqlite3');b=b'{';c.execute('UPDATE canonical_document SET json=? WHERE id=1',(b,));c.execute("UPDATE metadata SET value=? WHERE key='canonical_sha256'",(hashlib.sha256(b).hexdigest(),));c.commit();c.close();before=sha(path/'dictionary.sqlite3');run('malformed-fast-info',['info',path]);got=result('malformed-all','all',path,False);need(got['model']=='failed' and got['projection']=='blocked' and got['schema']=='passed' and got['fks']=='passed','malformed truthful independence');need(sha(path/'dictionary.sqlite3')==before,'malformed readonly')
# Independent known failures win over a simultaneous unknown draft prerequisite.
combined_cases=[('broken-draft-bad-header',"UPDATE metadata SET value='wrong-family' WHERE key='format'",'metadata'),('broken-draft-bad-hash',"UPDATE metadata SET value=printf('%064d',0) WHERE key='canonical_sha256'",'hashes'),('broken-draft-unsupported-dictionary',"UPDATE metadata SET value='2' WHERE key='dictionary_version'",'metadata'),('broken-draft-unsupported-registry',"UPDATE metadata SET value='2' WHERE key='registry_version'",'metadata')]
for name,metadata_sql,known in combined_cases:
 path=W/name;shutil.copytree(p,path);c=sqlite3.connect(path/'dictionary.sqlite3');b=b'{';c.execute('UPDATE canonical_document SET json=? WHERE id=1',(b,));c.execute("UPDATE metadata SET value=? WHERE key='canonical_sha256'",(hashlib.sha256(b).hexdigest(),));c.execute(metadata_sql);c.commit();c.close();before=sha(path/'dictionary.sqlite3')
 got=result(name+'-all','all',path,False);need(got[known]=='failed' and got['model']=='failed' and got['projection']=='blocked' and got['schema']=='passed' and got['fks']=='passed',name+' known failure not masked as blocked');need(sha(path/'dictionary.sqlite3')==before,name+' readonly')
# Simultaneous independent errors must all surface, not stop after schema.
path=W/'simultaneous';shutil.copytree(p,path);mutate(path,"CREATE TABLE cdc_extra(value TEXT);PRAGMA foreign_keys=OFF;INSERT INTO source_aliases(ordinal,source_id,text,status) VALUES(900,'cdc-absent','cdc-alias','Active');",lambda d:d['metadata']['assertions'][0].update(entry='cdc-missing-entry'));before=sha(path/'dictionary.sqlite3');got=result('simultaneous-all','all',path,False);need(got['schema']=='failed' and got['model']=='failed' and got['fks']=='failed' and got['projection']=='blocked','all independent failures and blocked projection');need(sha(path/'dictionary.sqlite3')==before,'simultaneous readonly')
# Existing resolver and no-create boundary, explicit override and command help.
x=W/'xdg';default=x/'vlajmaji/stores/default';default.parent.mkdir(parents=True);shutil.copytree(p,default);result('default-all','all',xdg=x,home='relative');result('explicit-no-home','schema',p,home='relative',xdg='relative')
missing=W/'absent-default';raw=run('missing-default',['verify','all'],1,missing);need(raw['error']['code']!='verification_failed','missing path invocation error');need(not missing.exists(),'verify never creates default')
raw=run('missing-base',['verify','schema'],1);need(raw['error']['code']=='store_path','same no-base resolver')
for name in [None,*NAMES,'all']:
 args=['verify',*([] if name is None else [name]),'--help'];text=run('help-'+str(name),args,json_output=False).decode();need('XDG_DATA_HOME' in text and 'vlajmaji/stores/default' in text,'actual help defaults '+str(name))
need(sha(db)==original,'final tiny source unchanged')
res={'verdict':'PASS','source':SOURCE,'binary_sha256':sha(C),'baseline_binary_sha256':sha(B),'calls':len(records),'checks':checks,'tamper_cases':len(cases)+len(combined_cases)+3,'valid_check_names':NAMES,'base_db_sha256_before_after':original,'records':records};(R/'tiny-controls-result01.json').write_text(json.dumps(res,indent=2)+'\n');print(json.dumps({k:v for k,v in res.items() if k!='records'},indent=2))
