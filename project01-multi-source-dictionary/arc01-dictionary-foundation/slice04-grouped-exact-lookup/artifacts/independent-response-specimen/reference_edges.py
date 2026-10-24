"""Reference edge positives and alias rename continuation checks."""
import json,pathlib,subprocess,sys,collections
import response_oracle as o
DOC,BASE,SEQUENCE_ROOT,CLI,PUBLIC,OUT=sys.argv[1:];d=json.loads(pathlib.Path(DOC).read_bytes());records=[]
def check(name,fn):
 try:fn();records.append(dict(name=name,status='PASS'))
 except Exception as e:records.append(dict(name=name,status='FAIL',detail=str(e)))
def call(s,req,i):
 text=json.dumps(req);r=subprocess.run([PUBLIC,'request',str(s),text] if i=='public' else [CLI,'dictionary-query',str(s),'--request',text],capture_output=True,text=True);return r,json.loads(r.stdout) if r.stdout.strip() else None
def eq(a,b):
 ds=o.diffs(a,b);assert not ds,'; '.join(ds[:10])
for i in ['public','cli']:
 for name,t in [('unresolved-space',{'Unresolved':' '}),('unresolved-nul',{'Unresolved':'\0'}),('opaque-entry',{'Local':{'Entry':'entry-opaque'}}),('opaque-assertion',{'Local':{'Assertion':'assertion-19'}})]:
  def edge(t=t,i=i):
   r,v=call(BASE,dict(operation='resolve',target=t),i);assert r.returncode==0,(r.returncode,v,r.stderr);v.pop('state_sha256');eq(v,o.resolved(d,t))
  check(i+':'+name,edge)
 store=pathlib.Path(SEQUENCE_ROOT)/'renamed-paths'
 def oldalias(i=i):
  r,v=call(store,dict(operation='exact',spelling='farvi',options=dict(sources=['a'])),i);assert r.returncode!=0
  if i=='cli':assert not r.stdout.strip();v=json.loads(r.stderr)
  eq(v['error']['code'],'unknown_source')
 check(i+':rename-old-alias-active-only',oldalias)
 for sel,scope in [('a','IncludeRetired'),('a2','ActiveOnly'),('source-a','ActiveOnly')]:
  def selector(sel=sel,scope=scope,i=i):
   r,v=call(store,dict(operation='exact',spelling='farvi',options=dict(sources=[sel],alias_scope=scope)),i);assert r.returncode==0,(r.returncode,v,r.stderr);eq(v['totals']['occurrences'],4);eq(v['resolved_sources'],['source-a']);eq([h['assertion']['id'] for g in v['groups'] for h in g['hits']],['assertion-0','assertion-1','assertion-2','assertion-3'])
  check(i+':rename-selector:'+sel,selector)
for name in ['disabled-a','removed-a']:
 store=pathlib.Path(SEQUENCE_ROOT)/name
 target={'Catalog':dict(source='source-a',release='release-a',native_entry_id='definitely-wrong',native_assertion_id='69002',occurrence=d['metadata']['assertions'][0]['occurrence'])}
 for interface in ['public','cli']:
  def mismatch(s=store,t=target,i=interface):
   r,v=call(s,dict(operation='resolve',target=t),i);assert r.returncode==0,(r.returncode,v,r.stderr);eq(v['status'],'NoMatch');eq(v['confirmed'],[]);eq(v['unverified'],[])
  check(interface+':unavailable-known-mismatch:'+name,mismatch)
pathlib.Path(OUT).write_text(json.dumps(dict(records=records,summary=dict(collections.Counter(r['status'] for r in records))),indent=2)+'\n');print(json.dumps(dict(collections.Counter(r['status'] for r in records))));sys.exit(any(r['status']!='PASS' for r in records))
