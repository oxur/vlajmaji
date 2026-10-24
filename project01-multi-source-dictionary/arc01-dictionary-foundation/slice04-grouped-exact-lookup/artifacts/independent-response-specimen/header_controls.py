"""Supplemental response/header/transport controls; not a production oracle."""
import hashlib,json,pathlib,sqlite3,subprocess,sys,collections
import response_oracle as o
DOC,STORE,CLI,PUBLIC,OUT=sys.argv[1:];d=json.loads(pathlib.Path(DOC).read_bytes());records=[]
def check(name,fn):
 try:fn();records.append(dict(name=name,status='PASS'))
 except Exception as e:records.append(dict(name=name,status='FAIL',detail=str(e)))
def request(req,interface='public'):
 text=json.dumps(req,separators=(',',':'));r=subprocess.run([PUBLIC,'request',STORE,text] if interface=='public' else [CLI,'dictionary-query',STORE,'--request',text],text=True,capture_output=True);assert r.returncode==0,(r.returncode,r.stdout,r.stderr);return json.loads(r.stdout)
def eq(a,b):assert a==b,(a,b)
c=sqlite3.connect(f'file:{STORE}/dictionary.sqlite3?mode=ro',uri=True);raw=c.execute('SELECT json FROM canonical_document').fetchone()[0];c.close();state=hashlib.sha256(raw).hexdigest();eq(state,'c8f4cf31fcf571a7de887db524aa6c9bfe5e73e4bb2f480fb6bb49e25845ce55')
for interface in ['public','cli']:
 def head(i=interface):
  v=request(dict(operation='exact',spelling='farvi'),i)
  eq(set(v),set('version policy store_schema_version catalogue_revision state_sha256 spelling resolved_sources native_candidate_occurrences totals source_totals availability groups next_cursor code_notices'.split()));eq(v['state_sha256'],state);eq(v['resolved_sources'],['source-a','source-b','source-o'])
 check(interface+':exact-headers',head)
 def cur(i=interface):
  v=request(dict(operation='exact',spelling='farvi',options=dict(page_size=1)),i);cursor=v['next_cursor'];eq(cursor,cursor.lower());w=json.loads(bytes.fromhex(cursor));eq(set(w),{'body','checksum'});eq(set(w['body']),{'version','binding','last'});eq(set(w['body']['last']),{'source','spelling','occurrence_id'});eq(w['body']['version'],1);eq(w['checksum'],hashlib.sha256(json.dumps(w['body'],ensure_ascii=False,separators=(',',':')).encode()).hexdigest());eq(w['body']['last'],dict(source='source-a',spelling='farvi',occurrence_id=o.oid(d['metadata']['assertions'][0]['occurrence'])))
  z=request(dict(operation='exact',spelling='farvi',options=dict(page_size=0,cursor=cursor)),i);eq(z['groups'],[]);eq(z['next_cursor'],None);eq(z['totals']['occurrences'],6)
  resized=request(dict(operation='exact',spelling='farvi',options=dict(page_size=3,cursor=cursor)),i);eq([h['assertion']['id'] for g in resized['groups'] for h in g['hits']],['assertion-1','assertion-2','assertion-3'])
 check(interface+':cursor-wire-and-resize',cur)
 for size in [0,1,2**64-1]:
  def empty(i=interface,size=size):
   v=request(dict(operation='exact',spelling='never-a-word',options=dict(page_size=size)),i);eq(v['totals'],dict(occurrences=0,distinct_spellings=0,source_groups=0));eq(v['groups'],[]);eq(v['source_totals'],[]);eq(v['next_cursor'],None);eq(v['availability'],o.availability(d))
  check(interface+':empty-page:'+str(size),empty)
 def refhead(i=interface):
  v=request(dict(operation='resolve',target='Unknown'),i);eq(set(v),set('version state_sha256 target status entity confirmed unverified availability requested_release'.split()));eq(v['state_sha256'],state)
 check(interface+':reference-headers',refhead)
pathlib.Path(OUT).write_text(json.dumps(dict(records=records,summary=dict(collections.Counter(r['status'] for r in records))),indent=2)+'\n');print(json.dumps(dict(collections.Counter(r['status'] for r in records))));sys.exit(any(r['status']!='PASS' for r in records))
