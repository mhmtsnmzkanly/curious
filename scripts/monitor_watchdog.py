#!/usr/bin/env python3
"""Independent timer checks that monitors/samplers are still making progress."""
import argparse,json,time
from pathlib import Path
from monitor_runtime import save,dispatch

def evaluate(previous,observations,now):
 state=dict(previous);state['pending']=list(previous.get('pending',[]));state['incidents']=dict(previous.get('incidents',{}));state['pending_since']=dict(previous.get('pending_since',{}))
 for name,observation in observations.items():
  stamp=observation.get('last_check',0);limit=observation['limit']
  if observation.get('pending'):
   state['pending_since'].setdefault(name,now)
  else:state['pending_since'].pop(name,None)
  stale=now-stamp>limit or stamp>now+60
  stuck=name in state['pending_since'] and now-state['pending_since'][name]>900
  incident='stale' if stale else 'outbox' if stuck else None
  old=state['incidents'].get(name)
  if old!=incident:
   state['incidents'][name]=incident
   message=(f'{name}: izleme kaydı güncellenmiyor.' if stale else f'{name}: bildirim kuyruğu 15 dakikadır boşalmadı.' if stuck else f'{name}: izleme tekrar çalışıyor.')
   state['pending'].append({'title':'Curious — izleyici gözetimi','message':message,'type':'error' if incident else 'success','target':{'user':'u_e049630155fc'},'idempotency_key':f'curious-watchdog-{name}-{incident or "recovery"}-{now}'})
 state['last_check']=now;return state

def main():
 p=argparse.ArgumentParser();p.add_argument('--root',default='/root/curious/data');p.add_argument('--token-file',default='/root/notify/token.json');p.add_argument('--notify-cli',default='/root/notify/bin/notify.js');a=p.parse_args();root=Path(a.root);path=root/'watchdog.json';now=int(time.time());observations={}
 for name,filename,limit,key in [('meadow','meadow-monitor.json',180,'last_check'),('evolution','evolution-monitor.json',180,'last_check'),('sample-meadow','soak/latest.json',2100,'at'),('sample-evolution','soak-evolution/latest.json',2100,'at')]:
  try:data=json.loads((root/filename).read_text())
  except (OSError,ValueError):data={}
  observations[name]={'last_check':data.get(key,0),'limit':limit,'pending':bool(data.get('pending'))}
 try:previous=json.loads(path.read_text())
 except FileNotFoundError:previous={}
 state=evaluate(previous,observations,now);save(path,state);dispatch(state,path,a.notify_cli,a.token_file)
 print(json.dumps({'incidents':state['incidents'],'pending':len(state['pending'])}))
if __name__=='__main__':main()
