#!/usr/bin/env python3
"""Resumable isolated ecology validation; a successful process is not a passing ecosystem."""
import argparse
from concurrent.futures import ThreadPoolExecutor, as_completed
import hashlib
import fcntl
import json
import os
from pathlib import Path
import shutil
import subprocess
import time


def write_json(path, value):
    temporary = path.with_suffix(path.suffix + '.tmp')
    with temporary.open('w') as stream:
        stream.write(json.dumps(value, indent=2, ensure_ascii=False) + '\n')
        stream.flush()
        os.fsync(stream.fileno())
    temporary.replace(path)


def passed(row, ticks):
    names = ('Herbivore', 'Carnivore')
    return (row['ticks'] == ticks and row['first_extinction'] is None
            and all(row['alive'].get(s, 0) >= 2
                    and row['late_births'].get(s, 0) > 0
                    and row['late_descendant_breeders'].get(s, 0) >= 2 for s in names))


def summary(rows, manifest):
    if len({r['seed'] for r in rows}) != len(rows) or any(r['seed'] not in manifest['seeds'] for r in rows):
        raise ValueError('Results must belong to distinct planned seeds')
    rows = sorted(rows, key=lambda r: manifest['seeds'].index(r['seed']))
    successful = sum(passed(r, manifest['ticks']) for r in rows)
    required = (9 * len(manifest['seeds']) + 9) // 10
    accepted = len(rows) == len(manifest['seeds']) and successful >= required
    return {'finished': len(rows), 'planned': len(manifest['seeds']), 'passed': successful,
            'required': required, 'accepted': len(rows) == len(manifest['seeds']) and successful >= required,
            'qualified': accepted and manifest['ticks'] >= 100000 and len(manifest['seeds']) >= 30 and not manifest['dirty'],
            'complete': len(rows) == len(manifest['seeds']),
            'seed_results': [{'seed': r['seed'], 'ticks': r['ticks'], 'first_extinction': r['first_extinction'],
                              'alive': r['alive'], 'passed': passed(r, manifest['ticks'])} for r in rows]}


def run(args):
    probe = Path(args.probe).resolve()
    scenario = Path(args.scenario).resolve()
    output = Path(args.output).resolve()
    seeds = [int(s) for s in args.seeds.split(',')]
    if not seeds or len(set(seeds)) != len(seeds) or any(s < 0 or s >= 2**64 for s in seeds):
        raise ValueError('Seeds must be unique unsigned 64-bit integers')
    if args.ticks < 10 or args.timeout <= 0:
        raise ValueError('At least ten ticks are required to define a final 10% window')
    if not 1 <= args.jobs <= 2:
        raise ValueError('Use one or two isolated worker processes')
    if args.shock_at < 0 or args.shock_duration <= 0 or not 0 <= args.shock_percent <= 100:
        raise ValueError('Invalid shock parameters')
    if args.shock_at and args.shock_at + args.shock_duration >= args.ticks * 9 // 10:
        raise ValueError('Shock must end before the final recovery window')
    repo = Path(__file__).resolve().parents[1]
    revision = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=repo, text=True).strip()
    dirty = bool(subprocess.check_output(['git', 'status', '--porcelain'], cwd=repo, text=True).strip())
    if dirty and not args.allow_dirty:
        raise ValueError('Commit source first, or explicitly label the run --allow-dirty as exploratory')
    manifest = {'revision': revision, 'dirty': dirty, 'binary_sha256': hashlib.sha256(probe.read_bytes()).hexdigest(),
                'scenario_sha256': hashlib.sha256(scenario.read_bytes()).hexdigest(),
                'seeds': seeds, 'ticks': args.ticks, 'mode': args.mode, 'companions': not args.no_companions,
                'shock': {'at': args.shock_at, 'duration': args.shock_duration, 'percent': args.shock_percent},
                'acceptance': '90% complete; no species extinction; >=2 alive and >=2 descendant breeders per species in final 10%, with births'}
    if output.exists():
        if not args.resume:
            raise ValueError('Output already exists; use a fresh directory or --resume')
        if json.loads((output / 'manifest.json').read_text()) != manifest:
            raise ValueError('Refusing resume: source, binary, scenario or experiment settings changed')
    else:
        output.mkdir(parents=True)
        (output / 'scenario.json').write_bytes(scenario.read_bytes())
        shutil.copy2(probe, output / 'probe')
        write_json(output / 'manifest.json', manifest)
    if (hashlib.sha256((output / 'probe').read_bytes()).hexdigest() != manifest['binary_sha256']
            or hashlib.sha256((output / 'scenario.json').read_bytes()).hexdigest() != manifest['scenario_sha256']):
        raise ValueError('Frozen experiment artifacts changed')
    with (output / '.lock').open('a') as guard:
        fcntl.flock(guard.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
        return execute(args, output, manifest)


def execute(args, output, manifest):
    seeds = manifest['seeds']
    # Per-seed atomic files allow recovery even if killed midway through a seed.
    rows = []
    pending = []
    for seed in seeds:
        saved = output / f'seed-{seed}.json'
        if saved.exists():
            row = json.loads(saved.read_text())
            if row['seed'] != seed or row['requested_ticks'] != args.ticks:
                raise ValueError('Invalid cached seed result')
            rows.append(row)
            continue
        pending.append(seed)

    def one_seed(seed):
        command = [str(output / 'probe'), str(output / 'scenario.json'), str(args.ticks), str(seed), args.mode, '--stop-on-extinction']
        command.append('--progress-every=1000')
        if args.shock_at:
            command += [f'--shock-at={args.shock_at}', f'--shock-duration={args.shock_duration}', f'--shock-percent={args.shock_percent}']
        if args.no_companions:
            command.append('--no-companions')
        started = time.monotonic()
        with (output / f'seed-{seed}.log').open('w') as log:
            child = subprocess.run(command, text=True, stdout=subprocess.PIPE, stderr=log, timeout=args.timeout, check=True)
        row = json.loads(child.stdout)
        if row['seed'] != seed or row['requested_ticks'] != args.ticks or row['mode'] != args.mode:
            raise ValueError('Probe returned a different experiment')
        row['process_seconds'] = time.monotonic() - started
        return row

    write_json(output / 'progress.json', {**summary(rows, manifest), 'pending_seeds': pending, 'workers': args.jobs})
    with ThreadPoolExecutor(max_workers=args.jobs) as pool:
        futures = {pool.submit(one_seed, seed): seed for seed in pending}
        for future in as_completed(futures):
            seed = futures[future]
            try:
                row = future.result()
            except Exception as error:
                for other in futures:
                    other.cancel()
                write_json(output / 'progress.json', {**summary(rows, manifest), 'failed_seed': seed, 'error': str(error)[:1000]})
                raise
            write_json(output / f'seed-{seed}.json', row)
            rows.append(row)
            remaining = [s for s in pending if s not in {r['seed'] for r in rows}]
            write_json(output / 'progress.json', {**summary(rows, manifest), 'pending_seeds': remaining, 'workers': args.jobs})
            print(json.dumps({'seed': seed, 'ticks': row['ticks'], 'alive': row['alive'], 'passed': passed(row, args.ticks)}), flush=True)
    result = summary(rows, manifest)
    write_json(output / 'summary.json', result)
    write_json(output / 'progress.json', result)
    if args.notify:
        identity = hashlib.sha256(json.dumps(manifest, sort_keys=True).encode()).hexdigest()[:24]
        message = {
            'title': 'Curious: karma ekosistem doğrulaması',
            'message': f"{result['passed']}/{result['planned']} koşu kabul ölçütünü geçti; gereken {result['required']}. "
                       + ('Belirtilen deney eşiği geçti.' if result['accepted'] else 'Denge hedefi geçilmedi; yeni dünya dağıtılmadı.'),
            'type': 'success' if result['accepted'] else 'info',
            'target': {'user': 'u_e049630155fc'}, 'idempotency_key': 'curious-coexistence-validation-' + identity,
            'panel': {'title': 'Av–avcı deney sonucu', 'mode': 'static', 'manifest': {'version': 1, 'blocks': [
                {'id': 'result', 'type': 'markdown', 'content': f"{args.ticks:,} tick × {len(seeds)} seed. Mod: {args.mode}. "
                 f"Geçen: {result['passed']}; gereken: {result['required']}.\n\nBu sonuç gerçek gün/hafta dayanıklılığı veya sonsuz süre garantisi değildir."},
                {'id': 'evidence', 'type': 'markdown', 'content': '```json\n' + json.dumps({'manifest': manifest, 'summary': result}, indent=2) + '\n```'}]}}}
        write_json(output / 'notice.json', message)
        subprocess.run(['node', '/root/notify/bin/notify.js', '-t', '/root/notify/token.json', 'send', str(output / 'notice.json'), '--json'], check=True)
    return result


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--probe', default='target/release/examples/ecology_probe')
    p.add_argument('--scenario', required=True)
    p.add_argument('--output', required=True)
    p.add_argument('--seeds', default=','.join(str(s) for s in range(10001, 10031)))
    p.add_argument('--ticks', type=int, default=100000)
    p.add_argument('--mode', choices=['fixed', 'learning', 'mutation', 'both'], default='fixed')
    p.add_argument('--timeout', type=int, default=3600)
    p.add_argument('--jobs', type=int, default=1)
    p.add_argument('--no-companions', action='store_true')
    p.add_argument('--shock-at', type=int, default=0)
    p.add_argument('--shock-duration', type=int, default=500)
    p.add_argument('--shock-percent', type=int, default=50)
    p.add_argument('--resume', action='store_true')
    p.add_argument('--allow-dirty', action='store_true')
    p.add_argument('--notify', action='store_true')
    print(json.dumps(run(p.parse_args()), indent=2))
