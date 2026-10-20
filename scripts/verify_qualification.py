#!/usr/bin/env python3
"""Read-only revalidation of every gate before a fresh observation deployment."""
import argparse
import json
from pathlib import Path

from research_protocol import TRAINING_SEEDS, digest, protocol, reused_stage


def verify(root):
    root = Path(root).resolve()
    config = json.loads((root/'config.json').read_text())
    state = json.loads((root/'state.json').read_text())
    declaration = json.loads((root/'protocol.json').read_text())
    if state.get('status') != 'qualified' or declaration != protocol(config):
        raise ValueError('Campaign is not qualified or its declared protocol changed')
    if config.get('candidates') or not config['reserve_opportunities'] or not config['mate_rendezvous']:
        raise ValueError('Observation deployment supports only the qualified reserve treatment')
    if digest(config['probe']) != config['probe_sha256'] or digest(config['scenario']) != config['scenario_sha256']:
        raise ValueError('Qualified probe or scenario changed')
    stages = [('canary-fixed', config['scenario'], TRAINING_SEEDS, 20000, 'fixed', False)]
    for field in ['max_energy', 'reproduction_interval']:
        for percent in [90, 110]:
            variant = json.loads(Path(config['scenario']).read_text())
            for founder in variant['founders']:
                if founder['species'] != 'Carnivore': continue
                life = founder['life']; capacity = life['max_energy']
                life[field] = life[field] * percent // 100
                if field == 'max_energy': life['energy'] = life['energy'] * life['max_energy'] // capacity
            path = root/f'fixed-{field}-{percent}.json'
            if json.loads(path.read_text()) != variant:
                raise ValueError('Sensitivity variant changed')
            stages.append((f'sensitivity-fixed-{field}-{percent}', path, TRAINING_SEEDS[:4], 20000, 'fixed', False))
    stages += [('qualification-fixed', config['scenario'], declaration['held_out_seeds'], 100000, 'fixed', False),
               ('resource-shock', config['scenario'], TRAINING_SEEDS, 20000, 'fixed', True)]
    stages += [('canary-'+mode, config['scenario'], TRAINING_SEEDS, 20000, mode, False)
               for mode in ['learning', 'mutation', 'both']]
    stages += [('qualification-both', config['scenario'], declaration['held_out_seeds'], 100000, 'both', False)]
    if set(state['completed']) != {s[0] for s in stages}:
        raise ValueError('Required research gates were not all completed')
    results, evidence = {}, {}
    for name, scenario, seeds, ticks, mode, shock in stages:
        check = config
        if name not in config['prevalidated']:
            p = root/name
            check = {**config, 'engine_revision': config['revision'], 'prevalidated': {
                name: {'path': str(p), **{f+'_sha256': digest(p/f) for f in ['manifest.json','summary.json']}}}}
        result, proof = reused_stage(check, name, scenario, seeds, ticks, mode, shock)
        if ticks == 100000 and not result['qualified']:
            raise ValueError('Long run does not meet formal qualification')
        results[name], evidence[name] = result, proof
    seed = next(row['seed'] for row in results['qualification-both']['seed_results'] if row['passed'])
    return {'engine_revision': config['engine_revision'], 'runner_revision': config['revision'],
            'probe_sha256': config['probe_sha256'], 'scenario_sha256': config['scenario_sha256'],
            'seed': seed, 'conditions': {'seed': seed,
                'policy': {'reserve_opportunities': True, 'mate_rendezvous': True},
                'genetics': {'enabled': True, 'mutation_per_mille': 20},
                'learning_enabled': True, 'companions_enabled': True},
            'phases': results, 'evidence': evidence}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('campaign', type=Path)
    args = parser.parse_args()
    print(json.dumps(verify(args.campaign), indent=2))
