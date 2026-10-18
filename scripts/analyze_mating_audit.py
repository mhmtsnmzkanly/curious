#!/usr/bin/env python3
"""Account local mating opportunities without inferring hidden world knowledge."""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path

IDLE = {'Rest', 'Explore', 'NoEligibleMate', 'RecallMate', 'ApproachMate'}


def analyze(path):
    windows, individuals, examples = {}, {}, []
    with path.open() as lines:
        for line in lines:
            row = json.loads(line)
            if row['type'] != 'decision':
                continue
            t, life = row['trace'], row['trace']['before']
            if t['species'] != 'Carnivore' or t['perception'] is None:
                raise ValueError('Full Carnivore audit required')
            tick, reason = t['tick'], t['decision']['reason']
            window = windows.setdefault(str(tick // 100 * 100), Counter())
            actor = individuals.setdefault(str(t['id']), Counter())
            mates = [e for e in t['perception']['entities']
                     if e['species'] == 'Carnivore' and e['mature']]
            flags = {'decisions': True, 'ready': row['ready'],
                     'idle': reason in IDLE, 'visible_adult': bool(mates),
                     'visible_ready': any(e['reproductive'] for e in mates),
                     'adjacent_adult': any(len(e['steps']) <= 1 for e in mates),
                     'idle_with_visible_adult': reason in IDLE and bool(mates),
                     'idle_with_distant_adult': reason in IDLE and any(len(e['steps']) > 1 for e in mates),
                     'ready_with_visible_ready': row['ready'] and any(e['reproductive'] for e in mates),
                     'ready_with_adjacent_ready': row['ready'] and any(e['reproductive'] and len(e['steps']) <= 1 for e in mates),
                     'idle_energy_blocked_with_adult': reason in IDLE and row['blocker'] == 'InsufficientEnergy' and bool(mates)}
            target = life['max_energy'] * life['reproduction_reserve_per_mille'] // 1000 + 1
            sufficient = [f for f in t['perception']['foods'] if f['is_corpse'] and len(f['steps']) <= 1
                          and min(f['amount'], life['max_meal'], life['max_energy']-life['energy']) >= target-life['energy']]
            flags['idle_energy_blocked_with_meal_and_distant_adult'] = (
                flags['idle_energy_blocked_with_adult'] and not flags['adjacent_adult'] and bool(sufficient))
            flags['reserve_blocked_only_by_mate_distance'] = (
                flags['idle_energy_blocked_with_meal_and_distant_adult']
                and reason in {'Rest', 'Explore', 'NoEligibleMate'}
                and life['health'] == life['max_health']
                and life['water'] >= life['max_water'] * 3 // 4
                and life['energy'] < target <= life['max_energy'])
            for counts in [window, actor]:
                counts.update(k for k, present in flags.items() if present)
                counts['reason.' + reason] += 1
                counts['blocker.' + (row['blocker'] or 'Ready')] += 1
            if flags['idle_energy_blocked_with_meal_and_distant_adult'] and len(examples) < 8:
                examples.append({'tick': tick, 'id': t['id'], 'energy': life['energy'],
                                 'reason': reason, 'mate_distances': [len(e['steps']) for e in mates],
                                 'food': sufficient})
    with path.open('rb') as source:
        sha = hashlib.file_digest(source, 'sha256').hexdigest()
    totals = Counter()
    for counts in windows.values():
        totals.update(counts)
    return {'source_sha256': sha, 'totals': dict(totals), 'windows': windows,
            'individuals': individuals, 'examples': examples,
            'limits': ['Counts are decision opportunities, not guaranteed mating or consumption.',
                       'Visible maturity does not establish the partner cooldown, reserves or remaining lifespan.',
                       'Companion memory is not proof the remembered animal is still alive.']}


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('audit', type=Path)
    p.add_argument('--output', type=Path, required=True)
    a = p.parse_args()
    result = analyze(a.audit)
    with a.output.open('x') as out:
        json.dump(result, out, indent=2)
        out.write('\n')
    print(json.dumps(result['totals']))
