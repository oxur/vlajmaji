"""Finite serialized process trials; separate raw streams, true OS highwater RSS."""
import argparse,hashlib,json,os,re,signal,subprocess,time
from pathlib import Path

def parse_rss(raw):
 m=re.findall(rb'^\s*(\d+)\s+maximum resident set size\s*$',raw,re.M)
 value=int(m[-1])if m else None
 return value if value and value>0 else None

def execute(argv,out,err,timeout=60,max_rss=8589934592):
 start=time.monotonic_ns();timed=False
 with out.open('xb')as fo,err.open('xb')as fe:
  child=subprocess.Popen(['/usr/bin/time','-l',*argv],stdout=fo,stderr=fe,start_new_session=True)
  try:code=child.wait(timeout=timeout)
  except subprocess.TimeoutExpired:
   timed=True;os.killpg(child.pid,signal.SIGTERM)
   try:code=child.wait(timeout=1)
   except subprocess.TimeoutExpired:os.killpg(child.pid,signal.SIGKILL);code=child.wait()
 elapsed=(time.monotonic_ns()-start)/1e9;eb=err.read_bytes();ob=out.read_bytes();rss=parse_rss(eb)
 reason='timeout'if timed else 'missing/zero/malformed RSS'if rss is None else 'RSS cap'if rss>max_rss else 'child exit'if code else None
 return dict(argv=argv,pid=child.pid,exit=code,timed_out=timed,outer_wall_seconds=elapsed,stdout_bytes=len(ob),stdout_sha256=hashlib.sha256(ob).hexdigest(),stderr_sha256=hashlib.sha256(eb).hexdigest(),maximum_rss_bytes=rss,rss_source='macOS /usr/bin/time -l (bytes)',stdout_path=str(out),stderr_path=str(err),decision='rejected: '+reason if reason else 'passed')

def main():
 p=argparse.ArgumentParser();p.add_argument('plan',type=Path);p.add_argument('output',type=Path);a=p.parse_args();plan=json.loads(a.plan.read_text());assert plan['version']==2
 orders=[r['order']for r in plan['rounds']];assert orders in [[['unified','attached','fanout']],[['attached','fanout','unified'],['fanout','unified','attached']]]
 a.output.mkdir();records=[]
 for round0 in plan['rounds']:
  assert len(round0['commands'])==3
  for topology,pair in zip(round0['order'],round0['commands']):
   assert set(pair)=={'fresh','warm'}
   for workload in ['fresh','warm']:
    tag=f"round{round0['number']}-{topology}-{workload}";record=execute(pair[workload],a.output/(tag+'.stdout'),a.output/(tag+'.stderr'))
    record.update(round=round0['number'],topology=topology,workload=workload);records.append(record)
    report=dict(status='in-progress',plan_sha256=hashlib.sha256(a.plan.read_bytes()).hexdigest(),observations=records)
    (a.output/'observations.json').write_text(json.dumps(report,indent=2)+'\n')
    assert record['decision']=='passed',(tag,record)
 report['status']='recorded; correctness and engineering budgets assessed separately';report['cache_state']='uncontrolled filesystem cache; fresh processes, no cache purge'
 (a.output/'observations.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report))
if __name__=='__main__':main()
