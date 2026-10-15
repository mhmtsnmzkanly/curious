#!/usr/bin/env python3
"""Predeclare a bounded demographic calibration grid before touching holdout seeds."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess


def prepare(generator, output):
    source = Path(__file__).resolve().parents[1]
    generator = Path(generator).resolve()
    output = Path(output).resolve()
    output.mkdir(parents=True, exist_ok=False)
    base = source / 'scenarios/mixed-refuge.json'
    profile = json.loads((source / 'scenarios/profiles/slow-predator.json').read_text())
    candidates = []
    # Order is fixed in advance. Selection may use only training seeds 1..2026;
    # holdouts 10001..10030 are reserved for one candidate's final qualification.
    for age in ['steady', 'staggered']:
        for n in [3, 4, 2]:
            for reserve in [890, 885, 895, 880, 900, 875]:
                name = f'n{n}-{age}-q{reserve}'
                body = {**profile, 'reproduction_reserve_per_mille': reserve}
                profile_path = output / f'{name}-profile.json'
                profile_path.write_text(json.dumps(body, indent=2) + '\n')
                scenario = output / f'{name}.json'
                subprocess.run([str(generator), str(base), str(n), '1', '1', age,
                                str(scenario), str(profile_path)], check=True)
                candidates.append({'name': name, 'scenario': str(scenario),
                                   'sha256': hashlib.sha256(scenario.read_bytes()).hexdigest(),
                                   'recipe': {'patches': n, 'food_multiplier': 1, 'water_multiplier': 1,
                                              'age_mode': age, 'predator': body}})
    manifest = {'base_sha256': hashlib.sha256(base.read_bytes()).hexdigest(),
                'generator_sha256': hashlib.sha256(generator.read_bytes()).hexdigest(),
                'revision': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=source, text=True).strip(),
                'candidates': candidates}
    (output / 'candidates.json').write_text(json.dumps(manifest, indent=2) + '\n')
    print(json.dumps({'count': len(candidates), 'manifest': str(output / 'candidates.json')}))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--generator', default='target/release/examples/habitat_fixture')
    parser.add_argument('--output', required=True)
    args = parser.parse_args()
    prepare(args.generator, args.output)
