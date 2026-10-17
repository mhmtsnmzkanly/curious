import unittest
from analyze_prey_collapse import windows

class PopulationFlowTests(unittest.TestCase):
    def result(self):
        return {'ticks': 60, 'alive': {'Carnivore': 1}, 'births': {'Herbivore': 1},
                'death_causes': {'Herbivore': {'Combat': 2, 'OldAge': 1}},
                'samples': [{'tick': 50, 'alive': {'Herbivore': 2, 'Carnivore': 1},
                             'births': {}, 'death_causes': {}}]}

    def test_terminal_partial_window_and_missing_species(self):
        rows = windows(self.result(), [50, 60])
        self.assertEqual(rows[0]['species']['Herbivore'],
                         {'before': 2, 'after': 0, 'births': 1,
                          'deaths': {'Combat': 2, 'OldAge': 1}, 'net': -2})
        self.assertEqual(rows[0]['species']['Carnivore']['net'], 0)

    def test_rejects_unexplained_loss(self):
        result = self.result()
        result['death_causes']['Herbivore']['Combat'] = 1
        with self.assertRaisesRegex(ValueError, 'accounting failed'):
            windows(result, [50, 60])

    def test_does_not_invent_missing_samples(self):
        self.assertEqual(len(windows(self.result(), [49, 50, 55, 60])), 1)

if __name__ == '__main__':
    unittest.main()
