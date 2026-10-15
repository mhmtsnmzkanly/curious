#!/usr/bin/env python3
"""HTTP and crash recovery integration check; uses an isolated temporary run."""
import json
import os
from pathlib import Path
import socket
import subprocess
import sys
import tempfile
import time
import urllib.error
import urllib.request

binary = str(Path(sys.argv[1] if len(sys.argv)>1 else 'target/debug/curious').resolve())
with socket.socket() as sock:
    sock.bind(('127.0.0.1',0))
    port=sock.getsockname()[1]
base=f'http://127.0.0.1:{port}'
token='integration-test-token-not-a-secret'
env=dict(os.environ,CURIOUS_TOKEN=token)

def request(path,body=None,authorized=True):
    headers={'Content-Type':'application/json'}
    if authorized: headers['Authorization']=f'Bearer {token}'
    req=urllib.request.Request(base+path, data=None if body is None else json.dumps(body).encode(),headers=headers)
    try:
        with urllib.request.urlopen(req,timeout=5) as response:
            return response.status,json.load(response)
    except urllib.error.HTTPError as error:
        return error.code,json.load(error)

def start(directory,resume=False):
    cmd=[binary,'--serve','--paused','--data-dir',directory,'--bind',f'127.0.0.1:{port}','--shard-mb','1']
    if resume: cmd+=['--resume']
    child=subprocess.Popen(cmd,env=env,stdout=subprocess.DEVNULL,stderr=subprocess.PIPE)
    for _ in range(100):
        if child.poll() is not None: raise AssertionError(child.stderr.read().decode())
        try:
            if request('/status')[0]==200:return child
        except OSError:pass
        time.sleep(.05)
    child.kill()
    raise AssertionError('server did not start')

with tempfile.TemporaryDirectory(prefix='curious-http-') as directory:
    child=start(directory)
    try:
        assert request('/state',authorized=False)[0]==401
        assert request('/control',{'action':'speed','ticks_per_second':0})[0]==400
        assert request('/control',{'action':'step','ticks':3})[0]==200
        assert request('/status')[1]['tick']==3
        assert request('/decisions',authorized=False)[0]==401
        assert request('/decisions')[1]
        assert request('/control',{'action':'observation','full_all':False,'full_for':[1]})[0]==200
        assert request('/control',{'action':'step','ticks':1})[0]==200
        traces=request('/decisions')[1]
        assert all((d['perception'] is not None)==(d['id']==1) for d in traces)
        assert all(d['outcome'] is not None for d in traces)
        assert request('/control',{'action':'spawn','species':'Herbivore','pos':{'x':0,'y':0}})[0]==200
        state=request('/state')[1]
        assert len(state['entities'])>=5
        assert request('/control',{'action':'set_cell','pos':{'x':0,'y':0},'cell':{'Food':{'amount':123}}})[0]==200
        assert request('/control',{'action':'intent','id':1,'intent':{'Idle':{'duration':1}}})[0]==200
        durable=request('/state')[1]
        assert durable['commands']
        # An OS crash must not lose acknowledged interventions or pending commands.
        child.kill(); child.wait(timeout=5)
        child=start(directory,True)
        assert request('/state')[1]==durable
        shard=request('/status')[1]['shard']
        data=request('/query',{'shard':shard,'sql':'SELECT COUNT(*) FROM interventions'})
        assert data[0]==200 and data[1]['rows'][0][0]>=4
        assert request('/query',{'shard':shard,'sql':'DELETE FROM records'})[0]==400
        # Force closed shards and inspect compressed history through the HTTP API.
        for _ in range(20):
            if any(row.get('compressed') for row in request('/databases')[1]):
                break
            assert request('/control',{'action':'step','ticks':20})[0]==200
        assert any(row.get('compressed') for row in request('/databases')[1])
        assert request('/query',{'shard':0,'sql':'SELECT COUNT(*) FROM records'})[1]['rows'][0][0]>0
        archived_state=request('/state')[1]
        child.kill(); child.wait(timeout=5)
        child=start(directory,True)
        assert request('/state')[1]==archived_state
        assert request('/control',{'action':'speed','ticks_per_second':100})[0]==200
        assert request('/control',{'action':'resume'})[0]==200
        time.sleep(.1)
        assert request('/control',{'action':'pause'})[0]==200
        paused=request('/status')[1]['tick']
        time.sleep(.05)
        assert request('/status')[1]['tick']==paused and paused>3
        # A second writer is refused by the OS lock, including after crash recovery.
        duplicate=subprocess.run([binary,'--serve','--resume','--data-dir',directory],env=env,capture_output=True,timeout=5)
        assert duplicate.returncode!=0
        assert request('/control',{'action':'shutdown'})[0]==200
        child.wait(timeout=5)
        assert child.returncode==0
    finally:
        if child.poll() is None:child.kill();child.wait()
print('HTTP controls, auth, pause/speed, audit, writer lock, SIGKILL recovery: OK')
