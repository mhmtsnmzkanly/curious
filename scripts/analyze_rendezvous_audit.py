#!/usr/bin/env python3
"""Stream a copied world's full-perception audit; never modify simulation data."""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path

DIRECTIONS = {'Up': (0, -1), 'Down': (0, 1), 'Left': (-1, 0), 'Right': (1, 0),
              'UpLeft': (-1, -1), 'UpRight': (1, -1), 'DownLeft': (-1, 1), 'DownRight': (1, 1)}

def endpoint(origin, steps):
    x, y = origin['x'], origin['y']
    for step in steps:
        dx, dy = DIRECTIONS[step]
        x, y = x + dx, y + dy
    return x, y

def safe_opportunity(row, mate):
    t = row['trace']
    p, life = t['perception'], t['before']
    if (not mate['steps'] or life['energy'] < life['move_energy_cost']
            or life['stamina'] < 1 or life['moves_used'] >= life['speed']):
        return False
    first = mate['steps'][0]
    dest = endpoint(t['pos'], [first])
    occupied = {endpoint(t['pos'], e['steps']) for e in p['entities']}
    if not p['directions'].get(first, 0) or dest in occupied:
        return False
    if any((pos['x'], pos['y']) == dest and cell == 'Burrow' for pos, cell in p['cells']):
        return True
    dangers = [endpoint(t['pos'], e['steps']) for e in p['entities'] if e['species'] == 'Carnivore']
    dangers += [(pos['x'], pos['y']) for pos, seen in row['dangers'] if max(0, t['tick'] - seen) <= 30]
    return not any(max(abs(dest[0] - x), abs(dest[1] - y)) <= 2 for x, y in dangers)

def analyze(path):
    counts, reasons, blockers, active, rejected, deaths = (Counter() for _ in range(6))
    refuge_examples, opportunity_examples = [], []
    for line in path.open():
        row = json.loads(line)
        if row['type'] == 'event':
            death = row['event'].get('EntityDied')
            if death:
                deaths[death['cause']] += 1
            continue
        t = row['trace']
        p = t['perception']
        if p is None:
            raise ValueError('Full perception is required; summaries cannot establish route safety')
        reason = t['decision']['reason']
        counts['decisions'] += 1
        reasons[reason] += 1
        blockers[row['blocker'] or 'Ready'] += 1
        if row['social']['target']:
            active[reason] += 1
        events = (t['outcome'] or {}).get('events', [])
        rejections = [e['ActionRejected'] for e in events if 'ActionRejected' in e]
        rejected.update(e['reason'] for e in rejections)
        if reason == 'SeekShelter':
            paths = [s for s in p['shelters'] if s]
            selected = min(paths, key=len)
            occupied = {endpoint(t['pos'], e['steps']) for e in p['entities']}
            blocked = endpoint(t['pos'], selected) in occupied
            alternatives = [s for s in paths if endpoint(t['pos'], s) not in occupied]
            counts['shelter_decisions'] += 1
            counts['shelter_target_occupied'] += blocked
            counts['occupied_shelter_with_open_alternative'] += blocked and bool(alternatives)
            counts['shelter_rejections'] += bool(rejections)
            counts['rejected_shelter_target_occupied'] += bool(rejections) and blocked
            if blocked and alternatives and rejections and len(refuge_examples) < 3:
                refuge_examples.append({'tick': t['tick'], 'id': t['id'], 'origin': t['pos'],
                                        'selected': selected, 'open_alternative': min(alternatives, key=len)})
        if reason == 'WaitForMate' and row['ready']:
            mates = [e for e in p['entities'] if e['species'] == 'Herbivore' and e['reproductive'] and e['mature']]
            safe = [m for m in mates if safe_opportunity(row, m)]
            counts['ready_waits_with_visible_ready_mate'] += bool(mates)
            counts['ready_waits_with_safe_opportunity'] += bool(safe)
            retry = t['tick'] < row['social']['retry_after']
            counts['safe_opportunity_waits_during_retry'] += bool(safe) and retry
            if safe and retry and len(opportunity_examples) < 3:
                opportunity_examples.append({'tick': t['tick'], 'id': t['id'], 'mate': safe[0]['id'],
                                             'distance': len(safe[0]['steps']), 'retry_after': row['social']['retry_after']})
    with path.open('rb') as source:
        digest = hashlib.file_digest(source, 'sha256').hexdigest()
    return {'source_sha256': digest, 'counts': dict(counts),
            'reasons': dict(reasons), 'physiological_blockers': dict(blockers),
            'decisions_with_active_goal': dict(active), 'rejections': dict(rejected),
            'death_causes': dict(deaths), 'occupied_refuge_examples': refuge_examples,
            'missed_opportunity_examples': opportunity_examples,
            'limits': ['Safety uses only visible/remembered danger; it does not prove an entire route safe.',
                       'Counts are entity-decisions, not distinct individuals or births.',
                       'Post-decision active targets omit goals that expired during the decision.']}

if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('audit', type=Path)
    parser.add_argument('--output', required=True, type=Path)
    args = parser.parse_args()
    result = analyze(args.audit)
    with args.output.open('x') as out:
        json.dump(result, out, indent=2)
        out.write('\n')
    print(json.dumps(result['counts']))
