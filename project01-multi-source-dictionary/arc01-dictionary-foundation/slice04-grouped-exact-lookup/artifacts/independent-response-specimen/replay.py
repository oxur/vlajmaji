"""Real CLI + unique public Rust caller driver; source-derived expected values, no query imports.
Usage: replay.py ACCEPTED_JSON ROOT CLI_BIN PUBLIC_BIN TRANSITION_ORACLE_PY
"""
import collections,copy,hashlib,importlib.util,json,pathlib,sqlite3,subprocess,sys,traceback
import response_oracle as o
INPUT,ROOT,CLI,PUBLIC,TRANS=sys.argv[1:];root=pathlib.Path(ROOT);root.mkdir();d=json.loads(pathlib.Path(INPUT).read_bytes());records=[]
spec=importlib.util.spec_from_file_location('transitions',TRANS);trans=importlib.util.module_from_spec(spec);spec.loader.exec_module(trans)
def check(name,fn):
 try:
  details=fn();records.append(dict(name=name,status='PASS',details=details));print('PASS',name,flush=True);return details
 except Exception as e:
  records.append(dict(name=name,status='FAIL',detail=str(e),traceback=traceback.format_exc()));print('FAIL',name,str(e)[:240],flush=True)
def eq(actual,expected):
 delta=o.diffs(actual,expected)
 assert not delta,'; '.join(delta[:12])
 return dict(compared=True)
def call(argv,stdin=None):
 r=subprocess.run([str(v) for v in argv],input=stdin,text=True,capture_output=True)
 try:value=json.loads(r.stdout) if r.stdout.strip() else None
 except Exception as e:raise AssertionError(f'bad stdoutJSON {r.returncode} {r.stdout[:500]} {r.stderr[:500]}') from e
 return r,value
def request(store,req,interface='public',stdin=False):
 text=json.dumps(req,ensure_ascii=False,separators=(',',':'))
 if interface=='public':return call([PUBLIC,'request',store,text])
 return call([CLI,'dictionary-query',store]+([] if stdin else ['--request',text]),text if stdin else None)
def success(store,req,interface='public',stdin=False):
 r,v=request(store,req,interface,stdin);assert r.returncode==0 and v is not None,(r.returncode,v,r.stderr);assert not r.stderr.strip(),r.stderr
 return v
def error(store,req,variant,code,interface='public'):
 r,v=request(store,req,interface)
 if interface=='public':assert r.returncode!=0 and v['variant']==variant and v['error']['code']==code,(r.returncode,v,r.stderr)
 else:
  assert r.returncode!=0 and not r.stdout.strip(),(r.returncode,r.stdout,r.stderr)
  v=json.loads(r.stderr);assert v['error']['code']==code,v
 return dict(exit=r.returncode,code=v['error']['code'])
def create(name,doc,interface='public',input_path=None):
 p=root/(name+'.json');p.write_text(json.dumps(doc,ensure_ascii=False,separators=(',',':')));p=pathlib.Path(input_path) if input_path else p
 store=root/name
 r,v=call([PUBLIC,'create',p,store] if interface=='public' else [CLI,'dictionary-build',p,store]);assert r.returncode==0,(r.returncode,v,r.stderr)
 assert isinstance(v,dict) and 'state_sha256' in v,v
 db=sqlite3.connect(f'file:{store}/dictionary.sqlite3?mode=ro',uri=True)
 canonical=db.execute('select json from canonical_document where id=1').fetchone()[0];db.close();eq(json.loads(canonical),doc);assert hashlib.sha256(canonical).hexdigest()==v['state_sha256']
 if interface=='public':eq(v['counts'],[len(doc['captures']),sum(len(c['occurrences']) for c in doc['captures']),len(doc['metadata']['entries']),len(doc['metadata']['assertions'])])
 return store
base=create('baseline',d,input_path=INPUT);cli_base=create('cli-baseline',d,'cli',INPUT)
def exact_check(store,doc,q,options=None,interface='public',stdin=False):
 options=options or {};v=success(store,dict(operation='exact',spelling=q,options=options),interface,stdin)
 ids=options.get('_expected_sources');rows=o.pick(doc,q,ids)
 eq(v['version'],1);eq(v['policy'],'exact-entry-v1');eq(v['store_schema_version'],1);eq(v['catalogue_revision'],doc['metadata']['catalogue_revision']);eq(v['spelling'],q)
 eq(v['totals'],dict(occurrences=len(rows),distinct_spellings=int(bool(rows)),source_groups=len({next(e['source'] for e in doc['metadata']['entries'] if e['id']==a['entry']) for a in rows})))
 eq(v['availability'],o.availability(doc));eq(v['code_notices'],doc['metadata']['code_notices'])
 candidates=sum(a['eligibility']=='Lexical' and next(e['spelling'] for e in doc['metadata']['entries'] if e['id']==a['entry'])=={'Value':q} for a in doc['metadata']['assertions']);eq(v['native_candidate_occurrences'],candidates)
 actualhits=[h for g in v['groups'] for h in g['hits']]
 expected=[o.hit(doc,a) for a in rows]
 if options.get('page_size')==0:expected=[];eq(v['next_cursor'],None)
 elif options.get('page_size') is not None:expected=expected[:options['page_size']]
 eq(actualhits,expected)
 count=collections.Counter(h['entry']['source'] for h in [o.hit(doc,a) for a in rows]);sources={s['id']:s for s in doc['registry']['sources']}
 eq(v['source_totals'],[dict(source=sources[s],occurrences=n,distinct_spellings=1) for s,n in sorted(count.items())])
 for g in v['groups']:
  s=g['source']['id'];eq(g['source'],sources[s]);eq(g['full_count'],count[s]);eq(g['returned_count'],len(g['hits']));eq(g['partial'],len(g['hits'])<count[s])
 if options.get('page_size') is None:eq(v['next_cursor'],None)
 return {'occurrences':v['totals']['occurrences'],'value_rows':len(actualhits)}
queries=['farvi',"si'e",'sie',"Si'e",'si’e',".si'e",'sy','bai','le jaica','cpana','cpan','snime','snim','never-a-word','siʼe','snimra',"SI'E",'farvi ']
for interface,store in [('public',base),('cli',cli_base)]:
 for q in queries:check(f'{interface}:exact-values:{q}',lambda q=q,i=interface,s=store:exact_check(s,d,q,interface=i,stdin=i=='cli' and q=="si'e"))
 for size in [0,1,2,3,4,6,7,2**64-1]:
  def pages(size=size,i=interface,s=store):
   expected=[o.oid(a['occurrence']) for a in o.pick(d,'farvi')];seen=[];cursor=None;page=0
   while True:
    v=success(s,dict(operation='exact',spelling='farvi',options=dict(page_size=size,cursor=cursor)),i)
    eq(v['totals'],dict(occurrences=6,distinct_spellings=1,source_groups=3));eq(v['availability'],o.availability(d))
    eq(v['source_totals'],[dict(source=next(x for x in d['registry']['sources'] if x['id']==src),occurrences=n,distinct_spellings=1) for src,n in [('source-a',4),('source-b',1),('source-o',1)]])
    hits=[h for g in v['groups'] for h in g['hits']]
    for g in v['groups']:
     n={'source-a':4,'source-b':1,'source-o':1}[g['source']['id']];eq(g['full_count'],n);eq(g['returned_count'],len(g['hits']));eq(g['partial'],len(g['hits'])<n)
    seen.extend(h['occurrence_id'] for h in hits);page+=1
    if v['next_cursor'] is None:break
    assert size>0 and v['next_cursor']!=cursor and page<20
    cursor=v['next_cursor']
   eq(seen,expected if size else []);return dict(pages=page,rows=len(seen))
  check(f'{interface}:all-pages:{size}',pages)
# Resolve all concrete baseline entity roles/targets and native confidence; compare every returned field.
targets=[{'Local':{'Entry':'entry-a-farvi'}},{'Local':{'Assertion':'assertion-18'}},{'Local':{'Form':'form-assigned'}},{'Local':{'EditorialSense':'editorial-farvi'}}]+[r['target'] for r in d['metadata']['relations']]
cat=dict(source='source-a',release='release-a',native_entry_id='333',native_assertion_id='69002',occurrence=None)
for native_entry,native_assertion,occ in [('333','69002',None),('333','333',None),('',None,None),(None,None,d['metadata']['assertions'][0]['occurrence']),('333','69002',d['metadata']['assertions'][5]['occurrence']),('wrong','69002',d['metadata']['assertions'][0]['occurrence'])]:
 t=copy.deepcopy(cat);t.update(native_entry_id=native_entry,native_assertion_id=native_assertion,occurrence=occ);targets.append({'Catalog':t})
for i,t in enumerate(targets):
 for interface,store in [('public',base),('cli',cli_base)]:
  def ref(t=t,s=store,intf=interface):
   v=success(s,dict(operation='resolve',target=t),intf);expected=o.resolved(d,t);v.pop('state_sha256');eq(v,expected);return dict(status=v['status'],confirmed=len(v['confirmed']),unverified=len(v['unverified']))
  check(f'{interface}:resolve-values:{i}',ref)
# Direct resolver requests crossing the accepted input validator boundary.
invalid=[{'Local':{'Assertion':'entry-a-farvi'}},{'Local':{'Entry':'assertion-0'}},{'Local':{'Form':'assertion-0'}},{'Local':{'EditorialSense':'form-assigned'}},{'Local':{'Entry':'unknown-entry'}},{'Unresolved':''}]
for changes in [dict(source='source-z'),dict(release='release-z'),dict(release='release-b'),dict(native_entry_id=None,native_assertion_id=None),dict(occurrence={**d['metadata']['assertions'][0]['occurrence'],'locator':''}),dict(occurrence={**d['metadata']['assertions'][0]['occurrence'],'locator':'/phantom'}),dict(occurrence=d['metadata']['assertions'][4]['occurrence']),dict(occurrence={'capture':{'namespace':'unknown','sha256':'f'*64},'locator':'/0'})]:
 t=copy.deepcopy(cat);t.update(changes);invalid.append({'Catalog':t})
for n,t in enumerate(invalid):
 for interface,store in [('public',base),('cli',cli_base)]:check(f'{interface}:invalid-reference:{n}',lambda t=t,s=store,i=interface:error(s,dict(operation='resolve',target=t),'InvalidReference','invalid_reference',i))
# Alias selection and availability errors happen before no-match, and duplicates resolve once.
for selector in ['source-a','a','archive-a']:
 for interface,store in [('public',base),('cli',cli_base)]:
  def filtered(sel=selector,i=interface,s=store):
   v=success(s,dict(operation='exact',spelling='farvi',options={'sources':[sel,sel]}),i);eq(v['resolved_sources'],['source-a']);eq([h for g in v['groups'] for h in g['hits']],[o.hit(d,a) for a in o.pick(d,'farvi',{'source-a'})]);eq(v['availability'],o.availability(d));return dict(occurrences=v['totals']['occurrences'])
  check(f'{interface}:filter:{selector}',filtered)
for interface,store in [('public',base),('cli',cli_base)]:
 for sel,code,variant in [('legacy-a','unknown_source','Registry'),('source-z','unknown_source','Registry'),('m','unavailable_source','UnavailableSource')]:
  check(f'{interface}:selector-error:{sel}',lambda sel=sel,c=code,v=variant,i=interface,s=store:error(s,dict(operation='exact',spelling='never-a-word',options={'sources':[sel]}),v,c,i))
 check(f'{interface}:retired-positive',lambda i=interface,s=store:eq(success(s,dict(operation='exact',spelling='farvi',options={'sources':['legacy-a'],'alias_scope':'IncludeRetired'}),i)['totals']['occurrences'],4))
# Strict cursor wire, not a product-generated expected answer.
first=success(base,dict(operation='exact',spelling='farvi',options={'page_size':1}));cursor=first['next_cursor'];assert cursor
wire=json.loads(bytes.fromhex(cursor));
def encode(w):return json.dumps(w,ensure_ascii=False,separators=(',',':')).encode().hex()
def resign(w):w=copy.deepcopy(w);w['checksum']=hashlib.sha256(json.dumps(w['body'],ensure_ascii=False,separators=(',',':')).encode()).hexdigest();return encode(w)
bad=[('nonhex','zz'),('odd','a'),('upper',cursor.upper()),('nonutf8','ff'),('json','7b'),('empty','')]
x=copy.deepcopy(wire);x['extra']=1;bad.append(('unknown-outer',encode(x)))
x=copy.deepcopy(wire);x['body']['extra']=1;bad.append(('unknown-body',resign(x)))
x=copy.deepcopy(wire);x['body']['last']['extra']=1;bad.append(('unknown-last',resign(x)))
x=copy.deepcopy(wire);x['body']['version']=2;bad.append(('version',resign(x)))
x=copy.deepcopy(wire);x['checksum']='0'*64;bad.append(('checksum',encode(x)))
x=copy.deepcopy(wire);x['body']['binding']='g'*64;bad.append(('digest-shape',resign(x)))
for label,c in bad:
 for interface,store in [('public',base),('cli',cli_base)]:check(f'{interface}:cursor-malformed:{label}',lambda c=c,s=store,i=interface:error(s,dict(operation='exact',spelling='farvi',options={'cursor':c}),'MalformedCursor','malformed_cursor',i))
for label,query,opts in [('spelling','cpana',{}),('sources','farvi',{'sources':['source-a']}),('alias-policy','farvi',{'alias_scope':'IncludeRetired'})]:
 for interface,store in [('public',base),('cli',cli_base)]:check(f'{interface}:cursor-stale:{label}',lambda q=query,op=opts,s=store,i=interface:error(s,dict(operation='exact',spelling=q,options={**op,'cursor':cursor}),'StaleCursor','stale_cursor',i))
x=copy.deepcopy(wire);x['body']['last']['occurrence_id']='not-a-member';nonmember=resign(x)
for interface,store in [('public',base),('cli',cli_base)]:
 check(f'{interface}:cursor-position',lambda i=interface,s=store:error(s,dict(operation='exact',spelling='farvi',options={'cursor':nonmember}),'InvalidCursorPosition','invalid_cursor_position',i))
 check(f'{interface}:cursor-resize-suffix',lambda i=interface,s=store:eq([h['occurrence_id'] for g in success(s,dict(operation='exact',spelling='farvi',options={'cursor':cursor}),i)['groups'] for h in g['hits']],[o.oid(a['occurrence']) for a in o.pick(d,'farvi')][1:]))
# Coherent catalogue revisions: metadata/payload/current identity checked separately.
for name,doc in trans.transition_sequence(d).items():
 try:store=create(name,doc)
 except Exception as e:records.append(dict(name='transition-setup:'+name,status='SETUP_FAILED',detail=str(e)));continue
 for interface in ['public','cli']:
  check(f'{interface}:transition-exact:{name}',lambda s=store,x=doc,i=interface:exact_check(s,x,'farvi',interface=i))
  check(f'{interface}:transition-cursor:{name}',lambda s=store,i=interface:error(s,dict(operation='exact',spelling='farvi',options={'cursor':cursor}),'StaleCursor','stale_cursor',i))
  for label,t in [('native',{'Catalog':cat}),('historic',{'Local':{'Assertion':'assertion-17'}})]:
   def tr(s=store,x=doc,i=interface,t=t):
    v=success(s,dict(operation='resolve',target=t),i);v.pop('state_sha256');eq(v,o.resolved(x,t));return dict(status=v['status'])
   check(f'{interface}:transition-ref:{name}:{label}',tr)
# CLI request/usage/IO contracts; syntax failures keep clap diagnostics.
for name,text in [('unknown','{"operation":"exact","spelling":"farvi","extra":1}'),('options','{"operation":"exact","spelling":"farvi","options":{"extra":1}}'),('decode','{')]:
 def reqerr(t=text):
  r,v=call([CLI,'dictionary-query',cli_base,'--request',t]);assert r.returncode!=0 and not r.stdout.strip();eq(json.loads(r.stderr)['error']['code'],'request_json');return dict(exit=r.returncode)
 check('cli:request-error:'+name,reqerr)
for args in [[],['dictionary-query'],['dictionary-build'],['dictionary-query',str(cli_base),'--bogus']]:
 def usage(args=args):
  r,v=call([CLI,*args]);assert r.returncode!=0 and not r.stdout.strip();return dict(exit=r.returncode)
 check('cli:usage:'+repr(args),usage)
check('cli:missing-input',lambda: error(root/'missing-store',dict(operation='exact',spelling='farvi'),'Store','store','cli'))
for flag in ['--help','--version']:
 def helpflag(flag=flag):
  r=subprocess.run([CLI,flag],capture_output=True,text=True);assert r.returncode==0 and r.stdout and not r.stderr;return dict(exit=0)
 check('cli:'+flag,helpflag)
(root/'results.json').write_text(json.dumps({'baseline_sha256':hashlib.sha256(pathlib.Path(INPUT).read_bytes()).hexdigest(),'records':records,'summary':dict(collections.Counter(r['status'] for r in records))},ensure_ascii=False,indent=2)+'\n')
print(json.dumps(dict(collections.Counter(r['status'] for r in records))));sys.exit(0 if all(r['status']=='PASS' for r in records) else 1)
