#!/usr/bin/env python3
"""Measure isolated HTTP worlds without modifying the live service. Standard library only."""
import argparse
import contextlib
import gzip
import tempfile
import shutil
from collections import Counter
import json
import os
from pathlib import Path
import socket
import sqlite3
import subprocess
import time
import urllib.request


@contextlib.contextmanager
def history_connection(path):
    with tempfile.TemporaryDirectory(prefix='curious-history-') as scratch:
        if path.suffix == '.gz':
            unpacked = Path(scratch)/'history.sqlite'
            with gzip.open(path,'rb') as source, unpacked.open('wb') as target:
                shutil.copyfileobj(source,target)
            path = unpacked
        with sqlite3.connect(f'file:{path}?mode=ro', uri=True) as connection:
            yield connection


def measure(binary, directory, seed, ticks, scenario=None, shard_mb=16):
    directory.mkdir(parents=True, exist_ok=False)
    with socket.socket() as sock:
        sock.bind(('127.0.0.1', 0))
        port = sock.getsockname()[1]
    base = f'http://127.0.0.1:{port}'
    token = 'isolated-ecosystem-measurement'
    env = dict(os.environ, CURIOUS_TOKEN=token)

    def request(path, body=None):
        req = urllib.request.Request(base + path, data=None if body is None else json.dumps(body).encode(),
                                     headers={'Content-Type': 'application/json', 'Authorization': f'Bearer {token}'})
        with urllib.request.urlopen(req, timeout=60) as response:
            return json.load(response)

    command = [binary, '--serve', '--paused', '--data-dir', str(directory), '--seed', str(seed),
               '--bind', f'127.0.0.1:{port}', '--shard-mb', str(shard_mb)]
    if scenario:
        command += ['--scenario', scenario]
    child = subprocess.Popen(command, env=env, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE)
    started = time.monotonic()
    peak_rss_kib=0
    peak_fds=0
    max_batch_seconds=0.0
    try:
        for _ in range(100):
            if child.poll() is not None:
                raise RuntimeError(child.stderr.read().decode())
            try:
                request('/status')
                break
            except OSError:
                time.sleep(.02)
        else:
            raise RuntimeError('server did not become ready')
        for start in range(0, ticks, 100):
            batch_start=time.monotonic()
            request('/control', {'action': 'step', 'ticks': min(100, ticks - start)})
            max_batch_seconds=max(max_batch_seconds,time.monotonic()-batch_start)
            status_text=Path(f'/proc/{child.pid}/status').read_text()
            peak_rss_kib=max(peak_rss_kib,int(next(line.split()[1] for line in status_text.splitlines() if line.startswith('VmRSS:'))))
            peak_fds=max(peak_fds,len(list(Path(f'/proc/{child.pid}/fd').iterdir())))
        request('/control', {'action': 'shutdown'})
        child.wait(timeout=5)
    finally:
        if child.poll() is None:
            child.kill()
            child.wait()
    elapsed = time.monotonic() - started
    events, intents, rejected, death_causes = Counter(), Counter(), Counter(), Counter()
    generations = {}
    samples = []
    first_extinction = None
    starvation_floor_ticks = 0
    histories=sorted(p for p in directory.glob('history-*') if p.name.endswith(('.sqlite','.sqlite.gz')))
    for db in histories:
        with history_connection(db) as connection:
            for tick, kind, encoded, metrics in connection.execute('SELECT tick,kind,state_json,metrics_json FROM records ORDER BY id'):
                state, metric = json.loads(encoded), json.loads(metrics)
                if kind == 'initial':
                    for entity in state['entities']:
                        generations[entity['id']] = 0
                if kind != 'tick':
                    continue
                if not metric['alive'] and first_extinction is None:
                    first_extinction = tick
                if any(e['life']['health'] > 0 and e['life']['health'] <= e['life']['max_health']//4
                       and not e['life']['water'] and e['phase'] == 'Active' for e in state['entities']):
                    starvation_floor_ticks += 1
                if tick == 1 or tick % 10 == 0:
                    samples.append({'tick': tick, **metric})
            for kind, payload in connection.execute('SELECT kind,payload FROM events ORDER BY record_id,ordinal'):
                events[kind] += 1
                event = json.loads(payload)[kind]
                if kind == 'EntityMated':
                    generations[event['child']] = max(generations.get(event['parent1'], 0), generations.get(event['parent2'], 0)) + 1
                if kind == 'EntityDied' and 'cause' in event:
                    death_causes[event['cause']] += 1
                if kind == 'ActionRejected':
                    rejected[event['reason']] += 1
            for (intent,) in connection.execute('SELECT intent_json FROM decisions'):
                intents[next(iter(json.loads(intent)))] += 1
    return {'seed': seed, 'ticks': ticks, 'wall_seconds': round(elapsed, 3), 'first_extinction_tick': first_extinction,
            'max_generation': max(generations.values(), default=0), 'events': dict(events), 'intents': dict(intents),
            'rejected_actions': dict(rejected), 'death_causes': dict(death_causes),
            'ticks_with_active_low_health_and_zero_water': starvation_floor_ticks, 'samples': samples,
            'peak_sampled_rss_kib':peak_rss_kib,'peak_sampled_fds':peak_fds,'max_100_tick_batch_seconds':round(max_batch_seconds,4),
            'history_bytes': sum(p.stat().st_size for p in histories)}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--binary', default='target/release/curious')
    parser.add_argument('--output', required=True)
    parser.add_argument('--scenario')
    parser.add_argument('--shard-mb', type=int, default=16)
    parser.add_argument('--seeds', default='1,2,3,4,5,42,12345,2026')
    parser.add_argument('--ticks', type=int, default=500)
    args = parser.parse_args()
    output = Path(args.output)
    output.mkdir(parents=True, exist_ok=False)
    report = {'revision': subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip(),
              'dirty': bool(subprocess.check_output(['git','status','--porcelain'],text=True).strip()),
              'scenario': args.scenario or 'default four founders; no interventions except step/shutdown', 'runs': []}
    for seed in map(int, args.seeds.split(',')):
        result = measure(str(Path(args.binary).resolve()), output / f'seed-{seed}', seed, args.ticks, args.scenario, args.shard_mb)
        report['runs'].append(result)
        (output / 'summary.json').write_text(json.dumps(report, indent=2) + '\n')
        print(json.dumps({k: v for k, v in result.items() if k != 'samples'}), flush=True)


if __name__ == '__main__':
    main()
