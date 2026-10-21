#!/usr/bin/env python3
"""Real HTTP/SQLite restart fidelity against the frozen qualified probe, in a temp run."""
import argparse
from collections import Counter
import json
import os
from pathlib import Path
import socket
import shutil
import subprocess
import tempfile
import time
import urllib.request


def run(args):
    token = 'isolated-observation-check-token'
    env = {**os.environ, 'CURIOUS_TOKEN': token}
    with tempfile.TemporaryDirectory(prefix='curious-observation-check-') as temporary:
        root=Path(temporary); data=root/'world'; reference=root/'reference';reference.mkdir()
        conditions=json.loads(Path(args.conditions).read_text())
        result=subprocess.run([args.initializer,args.scenario,args.conditions,str(data)],check=True,capture_output=True,text=True)
        with socket.socket() as sock:sock.bind(('127.0.0.1',0));port=sock.getsockname()[1]
        base=f'http://127.0.0.1:{port}'
        def request(path,body=None):
            headers={'Authorization':'Bearer '+token,'Content-Type':'application/json'}
            req=urllib.request.Request(base+path,headers=headers,data=None if body is None else json.dumps(body).encode())
            with urllib.request.urlopen(req,timeout=90) as response:return json.load(response)
        def start():
            child=subprocess.Popen([args.binary,'--serve','--resume','--paused','--data-dir',str(data),
                                    '--bind',f'127.0.0.1:{port}','--tps','0.1','--shard-mb','64','--max-history-mb','256'],
                                   env=env,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
            for _ in range(100):
                try:request('/status');return child
                except OSError:
                    if child.poll() is not None:raise RuntimeError('Observation server failed to start')
                    time.sleep(.05)
            child.kill();child.wait();raise RuntimeError('Observation server did not become ready')
        ref=subprocess.Popen([args.probe,args.scenario,'200',str(conditions['seed']),'both',
                              '--reserve-opportunities','--mate-rendezvous',
                              '--checkpoint-dir='+str(reference),'--checkpoint-at=100,200'],
                             stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
        child=None
        try:
            child=start();initial=request('/state')
            assert initial==json.loads((data/'initial-state.json').read_text())
            assert initial['policy']==conditions['policy'] and initial['genetics']==conditions['genetics']
            assert all(e['learner']['enabled'] and e['memory']['companions_enabled'] for e in initial['entities'])
            assert subprocess.run([args.initializer,args.scenario,args.conditions,str(data)],capture_output=True).returncode!=0
            assert request('/state')==initial
            started=time.monotonic();request('/control',{'action':'step','ticks':100});durable=request('/state')
            child.kill();child.wait();child=start()
            assert request('/state')==durable,'SIGKILL recovery changed state/RNG/settings'
            request('/control',{'action':'step','ticks':100});final=request('/state')
            ref.wait(timeout=120);assert ref.returncode==0
            for tick,state in [(100,durable),(200,final)]:
                expected=json.loads((reference/f'seed-{conditions["seed"]}'/f'checkpoint-{tick}.json').read_text())
                # Compare the accepted V12 biology; V13 adds inactive learning fields.
                comparable=json.loads(json.dumps(state))
                if state['version']==13 and expected['version']==12:
                    comparable['version']=12
                    for individual in comparable['entities']:
                        hunt=individual.pop('hunt_learner')
                        assert hunt=={'enabled':False,'update_enabled':True,'values':[],'episode':None,'last_episode':None}
                        assert individual['learner'].pop('update_enabled') is True
                assert comparable==expected,f'HTTP runtime differs from the frozen qualified biology at {tick}'
            inventory=request('/databases');sealed=[s for s in inventory if s['compressed']]
            ticks=0
            for shard in sealed:
                row=request('/query',{'shard':shard['shard'],'sql':"SELECT COUNT(*) FROM records WHERE kind='tick'"})
                ticks+=row['rows'][0][0]
            compressed=sum(s['bytes'] for s in sealed)
            if args.archive_sample and sealed:
                last=max(sealed,key=lambda s:s['shard'])
                candidates=list(data.glob(f'history-{last["shard"]:06}.sqlite.*'))
                assert len(candidates)==1
                shutil.copy2(candidates[0],args.archive_sample)
            evidence={'tick':final['tick'],'seed':conditions['seed'],'accepted_biological_state_and_rng_equal_at':[100,200],'new_learning_fields_validated_inactive':True,
                      'sigkill_recovery_equal':True,'existing_run_protected':True,
                      'policy':final['policy'],'genetics':final['genetics'],
                      'alive':dict(Counter(e['species'] for e in final['entities'] if e['life']['health']>0)),
                      'elapsed_recording_seconds':time.monotonic()-started,'closed_shards':len(sealed),
                      'closed_tick_records':ticks,'closed_archive_bytes':compressed,
                      'projected_archive_GiB_per_day_at_0_1_tps':compressed/ticks*8640/2**30 if ticks else None,
                      'limits':['Projection is from a 200-tick isolated sample, not a real-day capacity measurement.',
                                'Active shard, learner growth and future population changes add overhead.']}
            with Path(args.output).open('x') as stream:json.dump(evidence,stream,indent=2);stream.write('\n')
            print(json.dumps(evidence))
        finally:
            for p in [child,ref]:
                if p is not None and p.poll() is None:p.kill();p.wait()


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    for name in ['binary','initializer','probe','scenario','conditions','output']:p.add_argument('--'+name,required=True)
    p.add_argument('--archive-sample')
    a=p.parse_args()
    for name in ['binary','initializer','probe','scenario','conditions','output']:setattr(a,name,str(Path(getattr(a,name)).resolve()))
    run(a)
