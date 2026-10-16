#!/usr/bin/env python3
"""Read saved shock/control evidence; never rerun or modify a simulation."""
import argparse
import collections
import hashlib
import json
from pathlib import Path


def analyze(root):
    state = json.loads((root / 'state.json').read_text())
    selected = state['selected']['name']
    shock_dir = root / 'resource-shock'
    control_dir = root / ('canary-' + selected)
    shock_manifest = json.loads((shock_dir / 'manifest.json').read_text())
    control_manifest = json.loads((control_dir / 'manifest.json').read_text())
    for key in ['revision', 'binary_sha256', 'scenario_sha256', 'mode', 'seeds']:
        if shock_manifest[key] != control_manifest[key]:
            raise ValueError('Controls are not matched: ' + key)
    scenario = json.loads((shock_dir / 'scenario.json').read_text())
    treatment = shock_manifest['shock']
    at = treatment['at']
    site_rates = collections.Counter((s['kind'], s['replenish'], s['interval']) for s in scenario['sites'])
    rates = [{'kind': kind, 'count': count, 'interval': interval,
              'before': rate, 'during': rate * (100 - treatment['percent']) // 100}
             for (kind, rate, interval), count in sorted(site_rates.items())]
    rows = []
    for seed in shock_manifest['seeds']:
        path = shock_dir / f'seed-{seed}.json'
        shock = json.loads(path.read_text())
        control = json.loads((control_dir / path.name).read_text())
        pre = lambda result: [s for s in result['samples'] if s['tick'] < at]
        if pre(shock) != pre(control):
            raise ValueError(f'Seed {seed}: pre-shock population/energy observations differ')
        def point(tick):
            return next((s for s in shock['samples'] if s['tick'] == tick), None)
        dead = shock['last_predator_deaths']
        decisions = [d for death in dead for d in death['last_decisions']]
        ready = [d for d in decisions if d['ready']]
        final = dead[-1]
        last_decision = final['last_decisions'][-1]
        sampled = [s for s in shock['samples'] if s['tick'] >= at]
        rows.append({'seed': seed, 'extinct': shock['first_extinction'], 'ticks': shock['ticks'],
            'control_final': control['alive'], 'shock_final': shock['alive'],
            'pre_shock_observations_equal': True,
            'observations': {str(t): point(t) for t in [9900, 10100, 10500, 11000, 12000, 13000, 20000]},
            'sampled_minimum': {species: min([s['alive'].get(species, 0) for s in sampled] + [shock['alive'].get(species, 0)])
                                for species in ['Carnivore', 'Herbivore']},
            'last_deaths': dict(collections.Counter(d['cause'] for d in dead)),
            'recent_decisions': len(decisions), 'ready_recent_decisions': len(ready),
            'ready_with_visible_ready_mate': sum(d['ready_mates'] > 0 for d in ready),
            'last_death': {'tick': final['tick'], 'cause': final['cause'],
                'last_decision_before_death': last_decision},
            'source_sha256': hashlib.sha256(path.read_bytes()).hexdigest()})
    return {'revision': shock_manifest['revision'], 'binary_sha256': shock_manifest['binary_sha256'],
        'scenario_sha256': shock_manifest['scenario_sha256'], 'treatment': treatment,
        'effective_site_rates': rates, 'rows': rows,
        'limitations': ['Samples are every 100 ticks; minima may miss intermediate lows.',
            'Only the last 10 predator deaths and last 10 decisions per death are retained.',
            'No phase-resolved births, age distributions, water stocks or deaths are available.',
            'OldAge is the terminal cause, not proof that the shock had no physiological impact.',
            'Visible ready-mate counts omit unready or unseen potential partners.']}


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--research', type=Path, default=Path('data/coexistence-research'))
    p.add_argument('--output', type=Path, required=True)
    args = p.parse_args()
    result = analyze(args.research)
    with args.output.open('x') as file:
        json.dump(result, file, indent=2, ensure_ascii=False)
        file.write('\n')
    print(json.dumps({'seeds': len(result['rows']), 'extinctions': sum(r['extinct'] is not None for r in result['rows']),
                      'output': str(args.output)}))
