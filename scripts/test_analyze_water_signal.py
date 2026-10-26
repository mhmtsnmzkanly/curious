import tempfile
import unittest
from pathlib import Path

import analyze_water_signal as analyzer


class WaterSignalAnalyzerTests(unittest.TestCase):
    def test_baseline_mirrors_foraging_integer_arithmetic(self):
        life = {"speed": 3, "max_water": 60, "water": 50, "move_energy_cost": 1}
        # Four travel steps take two occupied ticks: (2*10 - 4 - 2)*100/2.
        self.assertEqual(analyzer.baseline_value(12, 5, life), 700)
        # Rust truncates toward zero for negative scores.
        self.assertEqual(analyzer.trunc_div(-400, 3), -133)

    def test_context_requires_exact_body_key_and_live_ttl(self):
        entry = {"pos": {"x": 4, "y": 0}, "speed": 3,
                 "move_energy_cost": 1, "last_seen_tick": 100,
                 "observations": [0, 10, -1, 3]}
        self.assertEqual(analyzer.context_prediction([entry], (4, 0), 3, 1, 148), 3)
        self.assertIsNone(analyzer.context_prediction([entry], (4, 0), 3, 1, 149))
        self.assertIsNone(analyzer.context_prediction([entry], (4, 0), 2, 1, 110))

    def test_context_can_change_order_when_score_gap_is_small(self):
        candidates = [{"steps_len": 1}, {"steps_len": 1}]
        self.assertEqual(analyzer.rank(candidates, [100, 90])[0], 0)
        self.assertEqual(analyzer.rank(candidates, [100, 120])[0], 1)

    def test_formal_records_show_sparse_signal_and_no_ranking_flips(self):
        input_dir = Path(__file__).resolve().parents[1] / analyzer.INPUT
        with tempfile.TemporaryDirectory() as tmp:
            result = analyzer.run(input_dir, Path(tmp) / "analysis.json")
        trained = result["aggregate"]["trained_online"]
        self.assertEqual(result["seed_count"], 6)
        self.assertEqual(trained["context_updates"], 147)
        self.assertEqual(trained["context_updates_zero"], 146)
        self.assertEqual(trained["context_update_residual_distribution"], {0: 146, 67: 1})
        self.assertEqual(trained["two_candidate_observations"], 46)
        self.assertEqual(trained["fresh_choice_opportunities"], 43)
        self.assertEqual(trained["ongoing_continuation_observations"], 3)
        self.assertEqual(trained["context_changed_baseline_ranking"], 0)


if __name__ == "__main__":
    unittest.main()
