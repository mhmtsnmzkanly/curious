#!/usr/bin/env python3
"""Notify actual elapsed 1/7/30-day milestones, never accelerated ticks as real days."""
import argparse,json,time
from pathlib import Path
from monitor_runtime import save,dispatch

def summarize(directory):
 result={'count':0,'first':None,'last':None,'rss_min':None,'rss_max':0,'fd_max':0,'pids':set(),'binaries':set(),'coverage_bins':set()}
 for path in sorted(Path(directory).glob('????-??.jsonl')):
  with path.open() as f:
   for line in f:
    try:s=json.loads(line)
    except ValueError:continue
    if 'status' not in s or 'rss_kib' not in s:continue
    result['count']+=1
    result['coverage_bins'].add(int(s['at']//900))
    if result['first'] is None:result['first']=s
    result['last']=s;rss=s['rss_kib'];result['rss_min']=min(result['rss_min'] or rss,rss);result['rss_max']=max(result['rss_max'],rss);result['fd_max']=max(result['fd_max'],s['fd_count']);result['pids'].add(s['pid']);result['binaries'].add(s.get('binary','older-sample'))
 result['coverage_samples']=len(result.pop('coverage_bins'))
 result['pids']=sorted(result['pids']);result['binaries']=sorted(result['binaries']);return result

def evaluate(previous,summary,now,label='',key_prefix='curious-evolution',page_id='p_6c6647a23f563ace'):
 state=dict(previous);state['pending']=list(previous.get('pending',[]));state['reported_days']=list(previous.get('reported_days',[]))
 first,last=summary['first'],summary['last']
 if not first or not last or now-last['at']>2100:return state
 span=last['at']-first['at']
 for days in [1,7,30]:
  if days in state['reported_days'] or span<days*86400 or summary.get('coverage_samples',summary['count'])<days*96*.8:continue
  status=last['status'];state['reported_days'].append(days)
  state['pending'].append({'title':f'Curious {label} — {days} günlük gerçek gözlem','message':f"{summary['count']} örnek; tick {status['tick']}, yaşayan {status['alive']}. RSS {summary['rss_min']}–{summary['rss_max']} KiB, en yüksek fd {summary['fd_max']}, gözlenen süreç sayısı {len(summary['pids'])}. Geçmiş {status['history_bytes']} bayt. Bu gerçek zamanlı ölçümdür; karma denge veya sürekli evrim kanıtı değildir.",'type':'info' if status['alive'] and not status.get('storage_error') else 'warning','target':{'user':'u_e049630155fc'},'page_id':page_id,'idempotency_key':f'{key_prefix}-real-days-{int(first["at"])}-{days}'})
 state['last_check']=now;return state

def main():
 p=argparse.ArgumentParser();p.add_argument('--directory',default='/root/curious/data/soak-evolution');p.add_argument('--state',default='/root/curious/data/soak-report.json');p.add_argument('--label',default='');p.add_argument('--key-prefix',default='curious-evolution');p.add_argument('--page-id',default='p_6c6647a23f563ace');a=p.parse_args();path=Path(a.state);previous=json.loads(path.read_text()) if path.exists() else {};summary=summarize(a.directory);state=evaluate(previous,summary,int(time.time()),a.label,a.key_prefix,a.page_id);save(path,state);dispatch(state,path,'/root/notify/bin/notify.js','/root/notify/token.json');print(json.dumps({'reported_days':state['reported_days'],'samples':summary['count']}))
if __name__=='__main__':main()
