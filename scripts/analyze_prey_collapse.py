#!/usr/bin/env python3
"""Read-only matched-candidate comparison and population-flow accounting."""
import argparse
import hashlib
import json
from pathlib import Path

SPECIES = ('Herbivore', 'Carnivore')

def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def final_point(result):
    return {k: result[k] for k in ('alive', 'births', 'death_causes')} | {'tick': result['ticks']}

def windows(result, ticks):
    points = {s['tick']: s for s in result['samples']}
    points[result['ticks']] = final_point(result)
    ticks = [t for t in ticks if t in points]
    rows = []
    for start, end in zip(ticks, ticks[1:]):
        a, b = points[start], points[end]
        by_species = {}
        for species in SPECIES:
            born = b['births'].get(species, 0) - a['births'].get(species, 0)
            before = a['death_causes'].get(species, {})
            after = b['death_causes'].get(species, {})
            deaths = {cause: after.get(cause, 0) - before.get(cause, 0)
                      for cause in sorted(set(before) | set(after))}
            initial, final = a['alive'].get(species, 0), b['alive'].get(species, 0)
            if initial + born - sum(deaths.values()) != final:
                raise ValueError(f'Population accounting failed: {species} {start}..{end}')
            by_species[species] = {'before': initial, 'after': final, 'births': born,
                                   'deaths': deaths, 'net': final - initial}
        rows.append({'from_exclusive': start, 'to_inclusive': end, 'species': by_species})
    return rows

def analyze(candidate, previous):
    cm = json.loads((candidate / 'manifest.json').read_text())
    pm = json.loads((previous / 'manifest.json').read_text())
    for key in ('scenario_sha256', 'seeds', 'mode', 'companions', 'reserve_opportunities', 'shock', 'ticks'):
        if cm[key] != pm[key]:
            raise ValueError('Unmatched experiment setting: ' + key)
    for root, manifest in ((candidate, cm), (previous, pm)):
        if digest(root / 'scenario.json') != manifest['scenario_sha256']:
            raise ValueError('Scenario digest mismatch')
    rows = []
    for seed in cm['seeds']:
        path, control = candidate / f'seed-{seed}.json', previous / f'seed-{seed}.json'
        c, p = json.loads(path.read_text()), json.loads(control.read_text())
        cs, ps = {s['tick']: s for s in c['samples']}, {s['tick']: s for s in p['samples']}
        shared = sorted(set(cs) & set(ps))
        rows.append({'seed': seed, 'candidate_end': c['alive'], 'candidate_extinction': c['first_extinction'],
                     'previous_end': p['alive'], 'previous_extinction': p['first_extinction'],
                     'first_different_sample': next((t for t in shared if cs[t] != ps[t]), None),
                     'source_sha256': digest(path), 'previous_sha256': digest(control),
                     'windows': windows(c, [16000, 16800, 17000, 17200, 17400, 17600, c['ticks']]),
                     'previous_windows': windows(p, [16000, 16800, 17000, 17200, 17400, 17600, 20000]),
                     'candidate_predator_preparations': c['predator_reasons'].get('PrepareReproduction', 0),
                     'previous_predator_preparations': p['predator_reasons'].get('PrepareReproduction', 0)})
    return {'candidate_revision': cm['revision'], 'previous_revision': pm['revision'],
            'scenario_sha256': cm['scenario_sha256'], 'rows': rows,
            'limits': ['Sample resolution is 100 ticks, except exact terminal totals.',
                       'Different trajectories are not matched individual counterfactuals.',
                       'Cause of death alone does not establish why reproduction failed.']}

if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--candidate', type=Path, default=Path('data/experiments/policy-mature-validation/normal'))
    parser.add_argument('--previous', type=Path, default=Path('data/experiments/policy-isolation-validation/normal'))
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    result = analyze(args.candidate, args.previous)
    with args.output.open('x') as file:
        json.dump(result, file, indent=2)
        file.write('\n')
    print(json.dumps({'seeds': len(result['rows']), 'output': str(args.output)}))
