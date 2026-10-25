from pathlib import Path
import subprocess,sys,json,os,time,hashlib,platform,statistics
R=Path('/tmp/vlajmaji-store-verification-cdc');S=Path('/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-store-verification/vlajmaji');OLD=Path('/tmp/vlajmaji-cli-default-store-cdc/candidate02-release-cli');NEW=Path(sys.argv[1]);HEAD=sys.argv[2]
DB=Path('/Users/oubiwann/.local/share/vlajmaji/stores/default/dictionary.sqlite3');EXPECTED='fe684fd52ef09aea9cba41bbdc97e57529199d66fdd4c04932bbe05e7131bf7d';FARVI='4a0c902b350162d7932169428713b53970d8aaba235c94da3cff313aa981054e';INFO='7ce64667b5a14bb0df0c24071371e6c8ceb054015fb76ad8a693e9cd30813ed3'
def sha(p):
 h=hashlib.sha256()
 with p.open('rb') as f:
  for b in iter(lambda:f.read(1048576),b''):h.update(b)
 return h.hexdigest()
assert sha(OLD)=='68250cb649464eecd299efef2eb70d23b6827e5d8c40355eebdd1a4ce8bb5675'
assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=S,text=True).strip()==HEAD
assert not subprocess.check_output(['git','status','--porcelain'],cwd=S)
x=os.environ.get('XDG_DATA_HOME');home=os.environ.get('HOME');base=Path(x) if x and Path(x).is_absolute() else Path(home)/'.local/share' if home and Path(home).is_absolute() else None
assert base and base/'vlajmaji/stores/default/dictionary.sqlite3'==DB
assert DB.stat().st_size==698281984 and sha(DB)==EXPECTED
identity={'old_source':'63da16f482de48632053d9892e58bd72de400255','new_source':HEAD,'old_binary':str(OLD),'new_binary':str(NEW),'old_binary_sha256':sha(OLD),'new_binary_sha256':sha(NEW),'profile':'release','host':platform.uname()._asdict(),'cwd':str(S),'HOME':home,'XDG_DATA_HOME':x,'database':str(DB),'db_bytes':DB.stat().st_size,'db_sha256_before':EXPECTED,'db_device':DB.stat().st_dev,'db_inode':DB.stat().st_ino,'protocol':'3pairs per operation old-new,new-old,old-new;12process calls. No cache purge; hashes/full verification warm filesystem caches; fresh processes only. Same stdout file capture for both conditions. Other host activity not controlled.'}
(R/'comparison-identity.json').write_text(json.dumps(identity,indent=2)+'\n');records=[]
for pair in range(3):
 for operation,args,expected,expected_bytes in [('info',['info'],INFO,304),('farvi',['dictionary-query','--request','{"operation":"exact","spelling":"farvi"}'],FARVI,3028230)]:
  for condition,binary in ([('old',OLD),('new',NEW)] if pair%2==0 else [('new',NEW),('old',OLD)]):
   label=f'pair{pair+1}-{operation}-{condition}';argv=[str(binary),*args]
   (R/'comparison-progress.json').write_text(json.dumps({'completed':records,'next':label,'argv':argv},indent=2)+'\n')
   t=time.monotonic_ns()
   with (R/(label+'.stdout')).open('xb') as out,(R/(label+'.stderr')).open('xb') as err:
    p=subprocess.Popen(argv,cwd=S,env=os.environ.copy(),stdin=subprocess.DEVNULL,stdout=out,stderr=err)
    (R/'comparison-active.json').write_text(json.dumps({'label':label,'pid':p.pid,'argv':argv,'time_ns':t},indent=2)+'\n')
    try:code=p.wait(timeout=120)
    except subprocess.TimeoutExpired:
     # Preserve identity/status for bounded manual diagnosis; never kill an unknown process.
     (R/'comparison-timeout.json').write_text(json.dumps({'label':label,'pid':p.pid,'argv':argv,'limit_s':120},indent=2)+'\n');raise
   elapsed=(time.monotonic_ns()-t)/1e9;stdout=R/(label+'.stdout');stderr=R/(label+'.stderr')
   (R/'comparison-active.json').write_text(json.dumps({'label':label,'pid':p.pid,'status':'completed','exit':code,'elapsed_s':elapsed},indent=2)+'\n')
   record={'pair':pair+1,'operation':operation,'condition':condition,'label':label,'argv':argv,'pid':p.pid,'exit':code,'elapsed_s':elapsed,'stdout_bytes':stdout.stat().st_size,'stdout_sha256':sha(stdout),'stderr_bytes':stderr.stat().st_size,'stderr_sha256':sha(stderr)};records.append(record)
   (R/'comparison-progress.json').write_text(json.dumps({'completed':records,'next':None},indent=2)+'\n')
   assert code==0 and record['stderr_bytes']==0 and record['stdout_bytes']==expected_bytes and record['stdout_sha256']==expected,record
   print(label,elapsed,'s PASS',flush=True)
assert len(records)==12 and sha(DB)==EXPECTED
summary={}
for op in ['info','farvi']:
 old=[r['elapsed_s'] for r in records if r['operation']==op and r['condition']=='old'];new=[r['elapsed_s'] for r in records if r['operation']==op and r['condition']=='new']
 summary[op]={'old_s':old,'new_s':new,'old_median_s':statistics.median(old),'new_median_s':statistics.median(new),'old_range_s':[min(old),max(old)],'new_range_s':[min(new),max(new)],'paired_old_over_new':[a/b for a,b in zip(old,new)],'median_old_over_new':statistics.median(old)/statistics.median(new),'all_whole_outputs_identical':True}
result={'verdict':'PASS','identity':identity,'records':records,'summary':summary,'db_sha256_after':sha(DB),'limitation':'Local finite same-input process-startup evidence, not disk-cold, constant-time, subsecond guarantee or population benchmark.'};(R/'comparison-result.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(summary,indent=2),flush=True)
