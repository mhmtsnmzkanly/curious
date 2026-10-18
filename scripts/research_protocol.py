"""Declared research treatments and read-only reuse of verified training evidence."""
import hashlib
import json
import os
from pathlib import Path

from validate_coexistence import summary

TRAINING_SEEDS = [1, 2, 3, 4, 5, 42, 12345, 2026]


def policy_flags(config):
    flags = []
    for field in ('reserve_opportunities', 'mate_rendezvous'):
        value = config.get(field, False)
        if not isinstance(value, bool):
            raise ValueError(f'{field} must be boolean')
        if value:
            flags.append('--' + field.replace('_', '-'))
    return flags


def protocol(config):
    policy_flags(config)
    seeds = config.get('held_out_seeds', list(range(10001, 10031)))
    if (len(seeds) != 30 or len(set(seeds)) != 30
            or any(type(s) is not int or not 0 <= s < 2**64 for s in seeds)
            or set(seeds).intersection(TRAINING_SEEDS)):
        raise ValueError('Declare 30 distinct unsigned holdout seeds outside training')
    fields = ('revision', 'probe_sha256', 'scenario_sha256', 'engine_revision',
              'engine_source', 'candidates', 'prevalidated', 'deploy_after_pass')
    return {**{key: config.get(key) for key in fields},
            'reserve_opportunities': config.get('reserve_opportunities', False),
            'mate_rendezvous': config.get('mate_rendezvous', False),
            'held_out_seeds': seeds}


def freeze(path, declaration):
    if path.exists():
        if json.loads(path.read_text()) != declaration:
            raise ValueError('Research protocol changed: use a fresh campaign, not resume')
    else:
        with path.open('x') as stream:
            stream.write(json.dumps(declaration, indent=2) + '\n')
            stream.flush()
            os.fsync(stream.fileno())


def digest(path):
    with Path(path).open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def reused_stage(config, name, scenario, seeds, ticks, mode, shock):
    entry = config.get('prevalidated', {}).get(name)
    if entry is None:
        return None
    root = Path(entry['path'])
    for filename in ('manifest.json', 'summary.json'):
        if digest(root / filename) != entry[filename + '_sha256']:
            raise ValueError('Prevalidated evidence changed: ' + filename)
    manifest = json.loads((root / 'manifest.json').read_text())
    expected = {'revision': config.get('engine_revision', config['revision']),
                'binary_sha256': config['probe_sha256'], 'scenario_sha256': digest(scenario),
                'dirty': False, 'seeds': seeds, 'ticks': ticks, 'mode': mode, 'companions': True,
                'reserve_opportunities': config.get('reserve_opportunities', False),
                'shock': {'at': 10000 if shock else 0, 'duration': 1000 if shock else 500, 'percent': 50}}
    if (any(manifest.get(k) != v for k, v in expected.items())
            or bool(manifest.get('mate_rendezvous', False)) != config.get('mate_rendezvous', False)):
        raise ValueError('Prevalidated experiment does not match this treatment')
    if digest(root / 'probe') != config['probe_sha256'] or digest(root / 'scenario.json') != expected['scenario_sha256']:
        raise ValueError('Prevalidated frozen artifacts changed')
    rows, hashes = [], {}
    for seed in seeds:
        path = root / f'seed-{seed}.json'
        row = json.loads(path.read_text())
        if (row['seed'] != seed or row['requested_ticks'] != ticks or row['mode'] != mode
                or row.get('reserve_opportunities', False) != expected['reserve_opportunities']
                or row.get('mate_rendezvous', False) != config.get('mate_rendezvous', False)
                or row.get('companions', True) is not True or row['shock'] != expected['shock']):
            raise ValueError('Prevalidated seed belongs to another treatment')
        rows.append(row)
        hashes[path.name] = digest(path)
    result = summary(rows, manifest)
    if not result['accepted'] or result != json.loads((root / 'summary.json').read_text()):
        raise ValueError('Prevalidated summary is incomplete or disagrees with raw results')
    return result, {'source': str(root), 'manifest_sha256': digest(root / 'manifest.json'),
                    'summary_sha256': digest(root / 'summary.json'), 'seed_sha256': hashes}


def progress_text(root, state):
    stage = state.get('stage', 'preparing')
    text = f"Durum: {state.get('status', 'preparing')}; aşama: {stage}."
    path = root / stage / 'progress.json'
    if path.exists():
        result = json.loads(path.read_text())
        text += f"\n\nBiten: {result['finished']}/{result['planned']}; geçen: {result['passed']}."
        for seed in result.get('pending_seeds', []):
            log = path.parent / f'seed-{seed}.log'
            if log.exists():
                with log.open('rb') as stream:
                    stream.seek(max(0, log.stat().st_size - 4096))
                    lines = stream.read().decode(errors='replace').splitlines()
                if lines:
                    try:
                        row = json.loads(lines[-1])
                        text += f"\n\nSeed {seed}: {row['tick']} adım; nüfus {row['alive']}."
                    except (ValueError, KeyError):
                        pass
    return text


def progress_block(root, state):
    path = root / state.get('stage', 'preparing') / 'progress.json'
    value = 100 if state.get('status') == 'qualified' else 0
    if path.exists():
        result = json.loads(path.read_text())
        value = min(100, result['finished'] * 100 / max(1, result['planned']))
    return {'id': 'current-progress', 'type': 'progress',
            'label': 'Bu aşamada tamamlanan koşular', 'value': value}
