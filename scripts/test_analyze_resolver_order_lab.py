import tempfile
import unittest
from pathlib import Path

import analyze_resolver_order_lab as analysis


class ResolverOrderAnalysisTests(unittest.TestCase):
    def test_artifact_hash_mismatch_fails_closed(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "artifact.json"
            path.write_bytes(b"registered bytes")
            with self.assertRaisesRegex(analysis.AnalysisError, "integrity mismatch"):
                analysis.checked_file(path, "0" * 64, path.stat().st_size, "result")

    def test_event_totals_mismatch_fails_closed(self):
        events = [
            {"EntityAte": {"id": 4, "amount": 3, "corpse_id": None, "pos": {"x": 1, "y": 0}}},
            {"EntityDrank": {"id": 5, "amount": 2, "pos": {"x": 0, "y": 0}}},
            {"EntityMoved": {"id": 4, "energy_cost": 1}},
            {"ActionRejected": {"id": 5, "action": "Eat", "reason": "MissingResource"}},
            {"EntityDied": {"id": 5}},
        ]
        derived = analysis.derive_event_metrics(events)
        branch = {"events": events, "event_totals": {**derived["totals"], "by_actor": derived["by_actor"]}}
        self.assertEqual(analysis.reconcile_event_totals(branch)["totals"]["plant_food_energy"], 3)
        branch["event_totals"]["water"] += 1
        with self.assertRaisesRegex(analysis.AnalysisError, "event total mismatch water"):
            analysis.reconcile_event_totals(branch)


if __name__ == "__main__":
    unittest.main()
