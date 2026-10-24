"""Calibrated variant query controls, called separately after main driver."""
import collections,copy,importlib.util,json,pathlib,subprocess,sys,traceback
import response_oracle as o
from variants import variants
INPUT,ROOT,CLI,PUBLIC=sys.argv[1:];root=pathlib.Path(ROOT);root.mkdir();d=json.loads(pathlib.Path(INPUT).read_bytes());records=[]
def check(name,fn):
 try:r=fn();records.append(dict(name=name,status='PASS',details=r));print('PASS',name,flush=True)
 except Exception as e:records.append(dict(name=name,status='FAIL',detail=str(e),traceback=traceback.format_exc()));print('FAIL',name,str(e)[:200],flush=True)
def eq(a,b):
 diff=o.diffs(a,b);assert not diff,'; '.join(diff[:10])
def run(args):
 r=subprocess.run([str(x) for x in args],capture_output=True,text=True);return r,json.loads(r.stdout) if r.stdout.strip() else None
def req(store,op,interface):return run([PUBLIC,'request',store,json.dumps(op)] if interface=='public' else [CLI,'dictionary-query',store,'--request',json.dumps(op)])
for name,doc in variants(d).items():
 p=root/(name+'.json');p.write_text(json.dumps(doc,ensure_ascii=False,separators=(',',':')));store=root/name;r,v=run([PUBLIC,'create',p,store])
 if r.returncode or v is None or 'counts' not in v:records.append(dict(name='setup:'+name,status='SETUP_FAILED',exit=r.returncode,stdout=r.stdout,stderr=r.stderr));continue
 for interface in ['public','cli']:
  def default(i=interface,s=store,x=doc):
   r,v=req(s,dict(operation='exact',spelling='farvi'),i);assert r.returncode==0,(r.returncode,v,r.stderr)
   eq([h for g in v['groups'] for h in g['hits']],[o.hit(x,a) for a in o.pick(x,'farvi')]);eq(v['availability'],o.availability(x));return dict(rows=v['totals']['occurrences'])
  check(interface+':default:'+name,default)
  def unavailable(selector,reason,i=interface,s=store):
   r,v=req(s,dict(operation='exact',spelling='never-a-word',options=dict(sources=[selector])),i);assert r.returncode!=0
   if i=='cli':assert not r.stdout.strip();v=json.loads(r.stderr)
   else:eq(v['variant'],'UnavailableSource')
   eq(v['error']['code'],'unavailable_source');eq(v['error']['sources'],[selector]);return dict(code=v['error']['code'],reason_expected=reason)
  if name=='no-owned-head':check(interface+':nohead-unavailable',lambda:unavailable('source-o','NoOwnedHead'))
  if name=='no-releases':check(interface+':noreleases-unavailable',lambda:unavailable('source-n','NoReleases'))
  if name=='available-zero':
   def empty(i=interface,s=store):
    r,v=req(s,dict(operation='exact',spelling='farvi',options=dict(sources=['source-e'])),i);assert r.returncode==0,(v,r.stderr);eq(v['groups'],[]);eq(v['totals'],dict(occurrences=0,distinct_spellings=0,source_groups=0));eq(v['resolved_sources'],['source-e']);eq(v['source_totals'],[]);return dict(empty=True)
   check(interface+':available-zero-selected',empty)
  if name=='alias-ambiguity':
   for scope,code in [('ActiveOnly','unknown_source'),('IncludeRetired','ambiguous_alias')]:
    def amb(sc=scope,code=code,i=interface,s=store):
     r,v=req(s,dict(operation='exact',spelling='farvi',options=dict(sources=['historic-shared'],alias_scope=sc)),i);assert r.returncode!=0
     if i=='cli':assert not r.stdout.strip();v=json.loads(r.stderr)
     else:eq(v['variant'],'Registry')
     eq(v['error']['code'],code)
     if code=='ambiguous_alias':eq(v['error']['sources'],['source-a','source-b'])
     return dict(code=code)
    check(interface+':retired-collision:'+scope,amb)
(root/'results.json').write_text(json.dumps(dict(records=records,summary=dict(collections.Counter(x['status'] for x in records))),ensure_ascii=False,indent=2)+'\n')
print(dict(collections.Counter(x['status'] for x in records)));sys.exit(0 if all(x['status']=='PASS' for x in records) else 1)
