#!/usr/bin/env python3
"""Task-owned bounded process wrapper: total elapsed <= limit, macOS time(1) RSS."""
import argparse, json, os, re, signal, subprocess, sys, time
from contextlib import ExitStack

def parse_rss(raw):
    matches=re.findall(rb'^\s*(\d+)\s+maximum resident set size\s*$',raw,re.M)
    value=int(matches[-1]) if matches else None
    return value if value is not None and value > 0 else None

def run(argv, log_path, timeout, max_rss_bytes=None, stdout_path=None):
    if not argv or timeout <= 0:
        raise ValueError('argv and positive timeout are required')
    started=time.monotonic()
    with open(log_path,'xb') as log, ExitStack() as outputs:
        stdout = outputs.enter_context(open(stdout_path,'xb')) if stdout_path else log
        child=subprocess.Popen(['/usr/bin/time','-l',*argv],stdout=stdout,stderr=log,start_new_session=True)
        pid=child.pid
        timed_out=False
        while child.poll() is None:
            if time.monotonic()-started >= timeout:
                timed_out=True
                os.killpg(child.pid,signal.SIGTERM)
                try: child.wait(timeout=1)
                except subprocess.TimeoutExpired:
                    os.killpg(child.pid,signal.SIGKILL); child.wait()
                break
            time.sleep(0.05)
        code=child.wait()
    elapsed=time.monotonic()-started
    raw=open(log_path,'rb').read()
    rss=parse_rss(raw)
    result={'pid':pid,'argv':argv,'log':log_path,'elapsed_seconds':elapsed,'exit_code':code,'timed_out':timed_out,'maximum_resident_set_size_bytes':rss,'rss_source':'/usr/bin/time -l maximum resident set size','rss_parse_ok':rss is not None,'stdout_path':stdout_path}
    if timed_out: result['decision']='rejected elapsed limit'
    elif rss is None: result['decision']='rejected missing/zero/unparseable RSS receipt'
    elif max_rss_bytes is not None and rss>max_rss_bytes: result['decision']='rejected RSS limit'
    elif code: result['decision']='rejected child exit'
    else: result['decision']='passed'
    print(json.dumps(result,sort_keys=True))
    return 124 if timed_out else 125 if rss is None else 126 if max_rss_bytes is not None and rss>max_rss_bytes else code

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--timeout',type=float,default=600.0);p.add_argument('--max-rss-bytes',type=int);p.add_argument('--log',required=True);p.add_argument('--stdout');p.add_argument('argv',nargs=argparse.REMAINDER);a=p.parse_args()
    if a.argv and a.argv[0]=='--':a.argv=a.argv[1:]
    try: raise SystemExit(run(a.argv,a.log,a.timeout,a.max_rss_bytes,a.stdout))
    except Exception as e: print('stage_guard_time error: '+str(e),file=sys.stderr);raise SystemExit(125)
