import json
from pathlib import Path
import tempfile
import unittest
from analyze_mating_audit import analyze


class MatingAuditTests(unittest.TestCase):
    def row(self, distance=3, ready=False, reason='Rest'):
        return {'type': 'decision', 'ready': ready, 'blocker': None if ready else 'InsufficientEnergy',
                'trace': {'id': 7, 'tick': 12001, 'species': 'Carnivore',
                          'before': {'max_energy': 270, 'energy': 220, 'max_meal': 60,
                                     'reproduction_reserve_per_mille': 885, 'health': 140, 'max_health': 140,
                                     'water': 60, 'max_water': 70},
                          'decision': {'reason': reason},
                          'perception': {'entities': [{'species': 'Carnivore', 'mature': True,
                                                       'reproductive': ready, 'steps': ['Right'] * distance}],
                                         'foods': [{'is_corpse': True, 'amount': 60, 'steps': ['Down']}]}}}

    def run_rows(self, rows):
        with tempfile.TemporaryDirectory() as d:
            path = Path(d)/'audit.jsonl'
            path.write_text(''.join(json.dumps(row)+'\n' for row in rows))
            return analyze(path)

    def test_distant_adult_meal_is_distinct_from_adjacent_and_ready_pair(self):
        result = self.run_rows([self.row(), self.row(distance=1), self.row(distance=1, ready=True, reason='Mate')])
        self.assertEqual(result['totals']['decisions'], 3)
        self.assertEqual(result['totals']['idle_energy_blocked_with_meal_and_distant_adult'], 1)
        self.assertEqual(result['totals']['ready_with_adjacent_ready'], 1)
        self.assertEqual(result['totals']['reserve_blocked_only_by_mate_distance'], 1)
        self.assertEqual(sum(w['decisions'] for w in result['windows'].values()), 3)

    def test_survival_and_incomplete_audits_are_not_mating_opportunities(self):
        result = self.run_rows([self.row(reason='SeekWater')])
        self.assertNotIn('idle_energy_blocked_with_meal_and_distant_adult', result['totals'])
        row = self.row(); row['trace']['perception'] = None
        with self.assertRaises(ValueError): self.run_rows([row])
