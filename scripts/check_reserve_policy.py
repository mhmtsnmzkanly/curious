#!/usr/bin/env python3
"""Bounded opt-in normal/shock comparison; never deploys or changes parameters."""
import argparse
import fcntl
import json
from pathlib import Path
import subprocess
import time

from validate_coexistence import write_json


def run(args):
    source = Path(__file__).resolve().parents[1]
    root = Path(args.output).resolve()
    root.mkdir(parents=True, exist_ok=True)
    with (root / '.lock').open('a') as guard:
        fcntl.flock(guard, fcntl.LOCK_EX | fcntl.LOCK_NB)
        for stage in ['normal', 'shock']:
            output = root / stage
            state = {'status': 'running', 'stage': stage}
            command = ['python3', '-u', str(source / 'scripts/validate_coexistence.py'),
                '--probe', args.probe, '--scenario', args.scenario, '--output', str(output),
                '--seeds', '1,2,3,4,5,42,12345,2026', '--ticks', '20000',
                '--mode', 'fixed', '--reserve-opportunities', '--jobs', '2', '--timeout', '14400', '--notify']
            if output.exists(): command.append('--resume')
            if stage == 'shock': command += ['--shock-at', '10000', '--shock-duration', '1000', '--shock-percent', '50']
            with (root / (stage + '.log')).open('a') as log:
                child = subprocess.Popen(command, stdout=log, stderr=subprocess.STDOUT)
                try:
                    while True:
                        state['last_check'] = int(time.time())
                        write_json(root / 'state.json', state)
                        try:
                            code = child.wait(timeout=20)
                            break
                        except subprocess.TimeoutExpired:
                            pass
                    if code:
                        write_json(root / 'state.json', {**state, 'status': 'error', 'exit_code': code})
                        raise RuntimeError(f'{stage}: validator failed with exit {code}')
                finally:
                    if child.poll() is None:
                        child.terminate()
                        child.wait(timeout=30)
            result = json.loads((output / 'summary.json').read_text())
            if not result['complete']:
                write_json(root / 'state.json', {**state, 'status': 'error', 'reason': 'incomplete'})
                raise RuntimeError('Incomplete results cannot open the next gate')
            if not result['accepted']:
                write_json(root / 'state.json', {**state, 'status': 'rejected', 'result': result})
                return
        write_json(root / 'state.json', {**state, 'status': 'passed-training-comparison', 'result': result})


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--probe', required=True)
    p.add_argument('--scenario', required=True)
    p.add_argument('--output', required=True)
    run(p.parse_args())
