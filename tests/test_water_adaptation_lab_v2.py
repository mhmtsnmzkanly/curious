import importlib.util
from pathlib import Path
import sys
import unittest

ROOT = Path(__file__).resolve().parents[1]
SCRIPTS = ROOT / "scripts"
sys.path.insert(0, str(SCRIPTS))
spec = importlib.util.spec_from_file_location("water_adaptation_lab_v2", SCRIPTS / "run_water_adaptation_lab_v2.py")
lab = importlib.util.module_from_spec(spec)
assert spec.loader is not None
spec.loader.exec_module(lab)


def row(tick, selected, *, amount=3, rng=44, learner_value=1, x=0):
    return {
        "tick": tick,
        "rng_before": rng,
        "state_before": {"entities": [{"id": 1, "x": x, "life": {"speed": 2, "move_energy_cost": 1},
                                         "learner": {"values": [], "context_entries": [{
                                             "pos": {"x": 4, "y": 0}, "speed": 2,
                                             "move_energy_cost": 1, "last_seen_tick": 211,
                                             "observations": [learner_value],
                                         }]}}]},
        "focal_choice": {
            "fresh_two_source_choice": True,
            "selected_target": selected,
            "candidates": [
                {"pos": {"x": -2, "y": 0}, "amount": amount, "steps_len": 2},
                {"pos": {"x": 4, "y": 0}, "amount": 12, "steps_len": 4},
            ],
        },
    }


class WaterAdaptationV2WitnessTests(unittest.TestCase):
    def test_witness_requires_different_valid_context_prediction(self):
        online = {"ticks": [row(211, {"x": 4, "y": 0}, learner_value=9)]}
        frozen = {"ticks": [row(211, {"x": -2, "y": 0}, learner_value=1)]}
        result = lab.first_online_frozen_choice_witness(online, frozen)
        self.assertEqual(result["status"], "supported")
        self.assertEqual(result["tick"], 211)
        self.assertEqual(result["differing_context_candidate_positions"], ['{"x":4,"y":0}'])

    def test_witness_not_established_when_rng_or_candidates_differ(self):
        online = {"ticks": [row(211, {"x": 4, "y": 0})]}
        frozen = {"ticks": [row(211, {"x": -2, "y": 0}, rng=45)]}
        self.assertEqual(lab.first_online_frozen_choice_witness(online, frozen)["status"], "not_established")

    def test_witness_not_established_when_valid_context_predictions_match(self):
        online = row(211, {"x": 4, "y": 0}, learner_value=1)
        frozen = row(211, {"x": -2, "y": 0}, learner_value=1)
        result = lab.first_online_frozen_choice_witness({"ticks": [online]}, {"ticks": [frozen]})
        self.assertEqual(result["status"], "not_established")
        online["state_before"]["entities"][0]["learner"]["context_entries"][0]["last_seen_tick"] = 100
        frozen["state_before"]["entities"][0]["learner"]["context_entries"][0]["last_seen_tick"] = 100
        result = lab.first_online_frozen_choice_witness({"ticks": [online]}, {"ticks": [frozen]})
        self.assertEqual(result["status"], "not_established")
        frozen = {"ticks": [row(211, {"x": -2, "y": 0}, amount=2)]}
        self.assertEqual(lab.first_online_frozen_choice_witness(online, frozen)["status"], "not_established")

    def test_missing_context_and_zero_prediction_have_same_effective_correction(self):
        online = row(211, {"x": 4, "y": 0}, learner_value=0)
        frozen = row(211, {"x": -2, "y": 0}, learner_value=1)
        frozen["state_before"]["entities"][0]["learner"]["context_entries"] = []
        result = lab.first_online_frozen_choice_witness({"ticks": [online]}, {"ticks": [frozen]})
        self.assertEqual(result["status"], "not_established")

    def test_nonfresh_choices_do_not_count_as_witness(self):
        a, b = row(211, {"x": 4, "y": 0}), row(211, {"x": -2, "y": 0})
        a["focal_choice"]["fresh_two_source_choice"] = False
        self.assertEqual(lab.first_online_frozen_choice_witness({"ticks": [a]}, {"ticks": [b]})["status"],
                         "not_established")

    def test_v2_seed_roles_are_fixed(self):
        self.assertEqual(lab.ROLES, {"calibration": [62299], "formal": [62201, 62202, 62203, 62204, 62205, 62206]})
        self.assertEqual(lab.PROTOCOL_SPEC["ttl_clock_frozen"], False)

    def test_replay_keeps_viewer_schema_and_projects_full_state(self):
        state = {"tick": 1, "bounds": [-1, 1, -1, 1], "resources": [], "entities": [],
                 "environment": {"large": "omitted"}}
        branch = {"ticks": [{
            "tick": 1,
            "state_after": state,
            "decisions": [{"id": 1, "pos": {"x": 0, "y": 0},
                           "decision": {"intent": "Wait"}, "extra": "omitted"}],
        }]}
        replay = lab.replay_for({"seed": 3, "forks": {name: branch for name in lab.ARMS}}, "rev")
        self.assertEqual(replay["schema"], "water-adaptation-replay-v1")
        self.assertEqual(replay["experiment"], "water-adaptation-v2")
        frame = replay["arms"][0]["frames"][0]
        self.assertEqual(set(frame["state"]), {"tick", "bounds", "resources", "entities"})
        self.assertEqual(set(frame["decisions"][0]), {"id", "pos", "decision"})

    def test_missing_control_branch_fails_validity(self):
        state = {"entities": [{"id": 1, "life": {"health": 100}, "learner": {
            "values": [], "residuals": [], "context_entries": [{"observations": [-2]}]}}]}
        branch = {"status": "ceiling", "ticks": [], "state_after_treatment_before_reversal": state,
                  "state_after_fork_interruption": state}
        record = {"seed": 2, "role": "calibration", "checkpoint_state": state,
                  "prefix": [], "forks": {name: branch for name in lab.ARMS if name != "rule_based"}}
        analysis = lab.analyze_record(record)
        self.assertFalse(analysis["validity"]["all_required_branches_present"])


if __name__ == "__main__":
    unittest.main()
