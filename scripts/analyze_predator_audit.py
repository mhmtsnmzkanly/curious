#!/usr/bin/env python3
"""Read-only accounting of predator choices and pre-existing missed meals."""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path


def analyze(path):
    reasons, blockers, attacks, deaths = (Counter() for _ in range(4))
    hidden, hungry, burrow, examples, ids = 0, 0, 0, [], set()
    decisions = 0
    for line in path.open():
        row = json.loads(line)
        if row['type'] == 'event':
            death = row['event'].get('EntityDied')
            if death:
                deaths[death['cause']] += 1
            continue
        t = row['trace']
        if t['species'] != 'Carnivore':
            raise ValueError('Predator audit requires Carnivore decisions')
        decisions += 1
        reasons[t['decision']['reason']] += 1
        blockers[row['blocker'] or 'Ready'] += 1
        missed = row['hidden_adjacent_corpses']
        if missed:
            hidden += 1
            hungry += t['before']['energy'] < t['before']['max_energy'] * 3 // 4
            burrow += any(c['burrow'] for c in missed)
            ids.update(c['id'] for c in missed)
            if len(examples) < 4:
                examples.append({'tick': t['tick'], 'id': t['id'], 'reason': t['decision']['reason'],
                                 'energy': t['before']['energy'], 'corpses': missed})
        if 'Attack' in t['decision']['intent']:
            attacks['intents'] += 1
            for event in (t['outcome'] or {}).get('events', []):
                if 'EntityAttacked' in event:
                    attacks['landed'] += 1
                if 'ActionRejected' in event:
                    attacks[event['ActionRejected']['reason']] += 1
    with path.open('rb') as source:
        digest = hashlib.file_digest(source, 'sha256').hexdigest()
    return {'source_sha256': digest, 'decisions': decisions, 'reasons': dict(reasons),
            'physiological_blockers': dict(blockers), 'attacks': dict(attacks),
            'death_causes': dict(deaths), 'hidden_adjacent_decisions': hidden,
            'hidden_adjacent_burrow_decisions': burrow, 'hidden_below_hunger_target': hungry,
            'unique_hidden_corpses': len(ids), 'examples': examples,
            'limits': ['Hidden corpses are pre-existing observer knowledge, never policy input.',
                       'New deaths and near-expiry corpses are omitted: this is a conservative sample.',
                       'Another consumer may take a meal during resolution; availability is not guaranteed intake.']}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('audit', type=Path)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    result = analyze(args.audit)
    with args.output.open('x') as out:
        json.dump(result, out, indent=2)
        out.write('\n')
    print(json.dumps({k: result[k] for k in ['decisions', 'hidden_adjacent_decisions', 'unique_hidden_corpses']}))
