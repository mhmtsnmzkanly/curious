#!/usr/bin/env python3
"""Read-only resource samples for the actual long-running world. No Notify or intervention."""
import argparse,datetime,json,os,subprocess,time,urllib.request
from pathlib import Path
from monitor_runtime import save

def main():
 p=argparse.ArgumentParser();p.add_argument('--url',default='http://127.0.0.1:7879');p.add_argument('--unit',default='curious-meadow.service');p.add_argument('--output',default='/root/curious/data/soak');p.add_argument('--species',action='store_true');a=p.parse_args()
 now=time.time();sample={'at':now,'url':a.url,'unit':a.unit}
 try:
  with urllib.request.urlopen(a.url+('/observation' if a.species else '/status'),timeout=5) as r:payload=json.load(r)
  sample['status']=payload['status'] if a.species else payload
  if a.species:
   from collections import Counter
   sample['species']=dict(Counter(e['species'] for e in payload['state']['entities'] if e['life']['health']>0))
  sample['response_ms']=round((time.time()-now)*1000,3)
  pid=int(subprocess.check_output(['systemctl','show','-p','MainPID','--value',a.unit],text=True))
  status=Path(f'/proc/{pid}/status').read_text()
  sample['rss_kib']=int(next(line.split()[1] for line in status.splitlines() if line.startswith('VmRSS:')))
  sample['fd_count']=len(list(Path(f'/proc/{pid}/fd').iterdir()))
  sample['pid']=pid
  sample['binary']=os.readlink(f'/proc/{pid}/exe')
  sample['service_started']=subprocess.check_output(['systemctl','show','-p','ActiveEnterTimestamp','--value',a.unit],text=True).strip()
 except (OSError,ValueError,subprocess.SubprocessError) as e:sample['error']=type(e).__name__
 out=Path(a.output);out.mkdir(parents=True,exist_ok=True)
 month=datetime.datetime.now(datetime.timezone.utc).strftime('%Y-%m')
 with (out/f'{month}.jsonl').open('a') as f:f.write(json.dumps(sample)+'\n');f.flush();os.fsync(f.fileno())
 save(out/'latest.json',sample)
 print(json.dumps(sample))
if __name__=='__main__':main()
