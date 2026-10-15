#!/usr/bin/env python3
"""Read-only individual and trophic accounting from tick history (including gz)."""
import argparse
from collections import Counter, deque
import json
from pathlib import Path

from measure_ecosystem import history_connection


def diagnose(directory):
    people = {}
    previous = {}
    expired_meat = Counter()
    samples = []
    last_tick = 0
    for path in sorted(Path(directory).glob('history-*')):
        if not path.name.endswith(('.sqlite', '.sqlite.gz')):
            continue
        with history_connection(path) as db:
            for record, tick, kind, encoded in db.execute(
                "SELECT id,tick,kind,state_json FROM records WHERE kind IN ('initial','tick') ORDER BY id"
            ):
                state = json.loads(encoded)
                current = {e['id']: e for e in state['entities']}
                last_tick = max(last_tick, tick)
                for ident, e in current.items():
                    row = people.setdefault(ident, {
                        'id': ident, 'species': e['species'], 'generation': e['generation'],
                        'parents': e['parents'], 'first_seen': tick, 'mature': False,
                        'births_parented': 0, 'ate': 0, 'drank': 0, 'attacks': 0,
                        'moves': 0, 'move_energy': 0, 'energy_spent': 0, 'energy_to_offspring': 0,
                        'decisions': Counter(), 'rejections': Counter(),
                        'ready_ticks': 0, 'visible_ready_mate_ticks': 0,
                        'adjacent_ready_mate_ticks': 0, 'last_decisions': deque(maxlen=10),
                    })
                    row['last_seen'] = tick
                    row['last_life'] = e['life']
                    life = e['life']
                    row['mature'] |= life['health'] > 0 and life['age'] >= life['maturity_age']
                    if life['health'] > 0:
                        row['last_alive'] = tick
                # Removed corpses have zero food when consumed, remaining food when expired.
                for ident, e in previous.items():
                    if isinstance(e['phase'], dict) and 'Corpse' in e['phase'] and e['corpse_energy'] and (ident not in current or current[ident]['phase'] == 'Removed'):
                        remaining = current.get(ident, e)['corpse_energy']
                        expired_meat[e['species']] += remaining
                if kind == 'tick':
                    intake = Counter()
                    transfers = Counter()
                    for event_kind, payload in db.execute(
                        'SELECT kind,payload FROM events WHERE record_id=? ORDER BY ordinal', (record,)
                    ):
                        event = json.loads(payload)[event_kind]
                        if event_kind == 'EntityMated':
                            for ident in [event['parent1'], event['parent2']]:
                                people[ident]['births_parented'] += 1
                                paid = current[ident]['life']['max_energy']//4
                                people[ident]['energy_to_offspring'] += paid
                                transfers[ident] += paid
                        elif event_kind == 'EntityDied':
                            people[event['id']]['death'] = {'tick': tick, 'cause': event.get('cause')}
                        elif event_kind in ('EntityAte', 'EntityDrank'):
                            people[event['id']]['ate' if event_kind == 'EntityAte' else 'drank'] += event['amount']
                            if event_kind == 'EntityAte':
                                intake[event['id']] += event['amount']
                        elif event_kind == 'EntityAttacked':
                            people[event['attacker']]['attacks'] += 1
                        elif event_kind == 'ActionRejected':
                            people[event['id']]['rejections'][event['reason']] += 1
                    for (payload,) in db.execute(
                        'SELECT trace_json FROM decision_traces WHERE record_id=?', (record,)
                    ):
                        t = json.loads(payload)
                        row = people[t['id']]
                        row['decisions'][t['decision']['reason']] += 1
                        life = t['before']
                        if t['outcome']:
                            moves = t['outcome']['life']['moves_used']
                            row['moves'] += moves
                            row['move_energy'] += moves * life['move_energy_cost']
                        ready = (life['age'] >= life['maturity_age'] and not life['reproduction_cooldown']
                                 and life['stamina'] >= 2 and life['energy'] >= life['max_energy']//2
                                 and life['water'] >= life['max_water']//2 and life['health'] >= life['max_health']//2)
                        mates = [e for e in t['seen']['entities'] if e['species'] == t['species'] and e['reproductive']]
                        row['ready_ticks'] += ready
                        row['visible_ready_mate_ticks'] += bool(ready and mates)
                        row['adjacent_ready_mate_ticks'] += bool(ready and any(e['distance'] <= 1 for e in mates))
                        row['last_decisions'].append({
                            'tick': tick, 'reason': t['decision']['reason'],
                            'energy': life['energy'], 'water': life['water'], 'health': life['health'],
                            'visible_prey': sum(e['species'] != 'Carnivore' for e in t['seen']['entities']) if t['species'] == 'Carnivore' else None,
                            'ready_mates': len(mates), 'events': t['outcome']['events'] if t['outcome'] else [],
                        })
                    for ident, before in previous.items():
                        if before['life']['health'] > 0 and ident in current:
                            # Actual energy removed, including saturated basal costs.
                            # Birth transfers are reported separately, not metabolism.
                            spent = before['life']['energy'] + intake[ident] - current[ident]['life']['energy'] - transfers[ident]
                            if spent < 0:
                                raise ValueError(f'Unexplained energy gain at tick {tick}, id {ident}')
                            people[ident]['energy_spent'] += spent
                if kind == 'tick' and (tick % 25 == 0 or tick == 1):
                    count = Counter(e['species'] for e in current.values() if e['life']['health'] > 0)
                    sheltered = sum(e['life']['health'] > 0 and any(p == e['pos'] and c == 'Burrow' for p, c in state['resources']) for e in current.values())
                    samples.append({'tick': tick, 'alive': dict(count), 'sheltered': sheltered})
                previous = current
    species = {}
    for name in ('Herbivore', 'Carnivore', 'Omnivore'):
        rows = [e for e in people.values() if e['species'] == name]
        if not rows:
            continue
        young = [e for e in rows if e['generation'] > 0]
        reasons = Counter()
        for e in rows:
            reasons.update(e['decisions'])
        species[name] = {
            'individuals': len(rows), 'births': len(young),
            'descendants_matured': sum(e['mature'] for e in young),
            'descendants_reproduced': sum(e['births_parented'] > 0 for e in young),
            'max_generation': max(e['generation'] for e in rows),
            'deaths': dict(Counter(e['death']['cause'] for e in rows if 'death' in e)),
            'last_alive': max(e.get('last_alive', 0) for e in rows),
            'ready_ticks': sum(e['ready_ticks'] for e in rows),
            'visible_ready_mate_ticks': sum(e['visible_ready_mate_ticks'] for e in rows),
            'adjacent_ready_mate_ticks': sum(e['adjacent_ready_mate_ticks'] for e in rows),
            'ate': sum(e['ate'] for e in rows), 'attacks': sum(e['attacks'] for e in rows),
            'energy_spent': sum(e['energy_spent'] for e in rows),
            'energy_to_offspring': sum(e['energy_to_offspring'] for e in rows),
            'move_energy': sum(e['move_energy'] for e in rows),
            'decisions': dict(reasons),
        }
    for e in people.values():
        e['last_decisions'] = list(e['last_decisions'])
    return {'last_tick': last_tick, 'species': species, 'expired_meat': dict(expired_meat),
            'samples': samples, 'individuals': list(people.values())}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('directory')
    parser.add_argument('--output', required=True)
    args = parser.parse_args()
    result = diagnose(args.directory)
    Path(args.output).write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps({k: v for k, v in result.items() if k not in ('individuals', 'samples')}, indent=2))
