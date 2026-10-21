import copy
import unittest

from measure_experience import ARMS, PAIRS, WINDOWS, summarize, window


def fixture():
    records = []
    trained = [{"pos": {"x": 0, "y": 3}, "value": 99, "visits": 400}]
    for training, evaluation in PAIRS:
        arms = []
        for name in ARMS:
            experienced = name.startswith("experienced")
            fixed = name in ("rule_based", "experienced_frozen")
            count = 400 if experienced else 0
            rows = []
            # Deliberately worse online learning: valid negative findings must pass.
            for i in range(400):
                water = 5 if name == "experienced_online" else 10
                rows.append({
                    "trial": i, "reversed": i >= 200,
                    "target": {"x": 0, "y": 3}, "water": water,
                    "movement_energy": 2, "incoming_damage": 0,
                    "rejected_actions": 0, "success": water == 10,
                    "partial": water < 10, "zero_water": False,
                    "unfinished_episode": False, "end": "Reached", "reward": 90,
                    "visits_before": count, "visits_after": count if fixed else count + 1,
                })
                if not fixed:
                    count += 1
            arms.append({
                "arm": name, "initial_values": copy.deepcopy(trained) if experienced else [],
                "initial_visits": 400 if experienced else 0,
                "reversal_values": copy.deepcopy(trained),
                "final_values": copy.deepcopy(trained), "trials": rows,
                "windows": {period: window(rows[i*100:(i+1)*100]) for i, period in enumerate(WINDOWS)},
            })
        records.append({
            "schema": "curious-experience-v1", "protocol": "water-competition-reversal-v1",
            "training_seed": training, "evaluation_seed": evaluation,
            "training_trials": 400, "evaluation_trials_per_arm": 400,
            "reversal_at": 200, "controlled_body_reset": True,
            "training_values": copy.deepcopy(trained), "arms": arms,
        })
    return records


class ExperienceSummaryTest(unittest.TestCase):
    def test_negative_finding_is_preserved_without_success_threshold(self):
        report = summarize(fixture())
        self.assertEqual(report["pairs"], 8)
        self.assertIsNone(report["superiority_claim"])
        self.assertEqual(report["per_seed"][0]["late_reversal_update_water_difference"], -500)
        self.assertEqual(report["mean_per_100_trials"]["experienced_online"]["after_last_100"]["partial_drinks"], 100)

    def test_rejects_missing_seed_and_falsified_summary(self):
        records = fixture()
        with self.assertRaises(ValueError):
            summarize(records[:-1])
        records[0]["arms"][0]["windows"]["after_last_100"]["water"] += 1
        with self.assertRaises(ValueError):
            summarize(records)

    def test_rejects_frozen_updates_and_changed_reversal(self):
        records = fixture()
        records[0]["arms"][3]["final_values"][0]["value"] += 1
        with self.assertRaises(ValueError):
            summarize(records)
        records = fixture()
        records[0]["arms"][1]["trials"][200]["reversed"] = False
        with self.assertRaises(ValueError):
            summarize(records)


if __name__ == "__main__":
    unittest.main()
