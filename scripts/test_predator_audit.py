import copy
import json
from pathlib import Path
import tempfile
import unittest
from analyze_predator_audit import analyze

class PredatorAuditTests(unittest.TestCase):
    def analyze_rows(self, rows):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'audit.jsonl'
            path.write_text(''.join(json.dumps(r) + '\n' for r in rows))
            return analyze(path)

    def row(self):
        return {'type': 'decision', 'blocker': 'InsufficientEnergy',
                'hidden_adjacent_corpses': [{'id': 9, 'burrow': True, 'amount': 60}],
                'trace': {'tick': 1, 'id': 1, 'species': 'Carnivore',
                          'before': {'energy': 100, 'max_energy': 270},
                          'decision': {'reason': 'HuntPrey', 'intent': {'Attack': {'target_id': 2}}},
                          'outcome': {'events': [{'ActionRejected': {'reason': 'NoPath'}}]}}}

    def test_repeated_opportunities_are_not_counted_as_distinct_corpses(self):
        a = self.row()
        b = copy.deepcopy(a)
        b['trace']['tick'] = 2
        b['trace']['outcome']['events'] = [{'EntityAttacked': {'damage': 20}}]
        r = self.analyze_rows([a, b])
        self.assertEqual(r['hidden_adjacent_decisions'], 2)
        self.assertEqual(r['unique_hidden_corpses'], 1)
        self.assertEqual(r['attacks'], {'intents': 2, 'NoPath': 1, 'landed': 1})

    def test_herbivore_audit_cannot_be_presented_as_predator_evidence(self):
        row = self.row()
        row['trace']['species'] = 'Herbivore'
        with self.assertRaisesRegex(ValueError, 'Carnivore'):
            self.analyze_rows([row])

if __name__ == '__main__':
    unittest.main()
