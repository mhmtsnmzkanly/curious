import json
from pathlib import Path
import tempfile
import unittest
from analyze_rendezvous_audit import analyze, safe_opportunity

class AuditEvidenceTests(unittest.TestCase):
    def analyze_rows(self, rows):
        with tempfile.TemporaryDirectory() as directory:
            p = Path(directory) / 'audit.jsonl'
            p.write_text(''.join(json.dumps(row) + '\n' for row in rows))
            return analyze(p)

    def test_missing_full_perception_cannot_produce_safety_claims(self):
        with self.assertRaisesRegex(ValueError, 'Full perception'):
            self.analyze_rows([{'type': 'decision', 'trace': {'perception': None}}])

    def test_refuge_occupancy_uses_coordinates_not_equal_path_spelling(self):
        row = {'type': 'decision', 'ready': False, 'social': {'target': None}, 'blocker': None,
               'trace': {'id': 1, 'tick': 1, 'pos': {'x': 0, 'y': 0},
                         'decision': {'reason': 'SeekShelter'}, 'outcome': None,
                         'perception': {'shelters': [['DownRight'], ['Left', 'Left']],
                                        'entities': [{'steps': ['Right', 'Down']}]}}}
        result = self.analyze_rows([row])['counts']
        self.assertEqual(result['shelter_target_occupied'], 1)
        self.assertEqual(result['occupied_shelter_with_open_alternative'], 1)

    def test_safe_step_requires_affordability_and_respects_diagonal_danger(self):
        row = {'dangers': [[{'x': 2, 'y': 2}, 1]], 'trace': {
            'tick': 1, 'pos': {'x': 0, 'y': 0},
            'before': {'energy': 80, 'move_energy_cost': 1, 'stamina': 16, 'moves_used': 0, 'speed': 3},
            'perception': {'directions': {'Right': 3}, 'entities': [], 'cells': []}}}
        mate = {'steps': ['Right'] * 3}
        self.assertFalse(safe_opportunity(row, mate))
        row['trace']['perception']['cells'] = [[{'x': 1, 'y': 0}, 'Burrow']]
        self.assertTrue(safe_opportunity(row, mate))
        row['trace']['before']['stamina'] = 0
        self.assertFalse(safe_opportunity(row, mate))

if __name__ == '__main__':
    unittest.main()
