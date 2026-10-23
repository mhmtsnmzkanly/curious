import copy
from pathlib import Path
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parent))
from run_lifetime_continuity_lab import validate_seed


def state(tick, age, update=True):
    return {
        "tick": tick,
        "entities": [
            {
                "id": 1,
                "species": "herbivore",
                "generation": 0,
                "life": {"age": age, "health": 100, "energy": 500},
                "learner": {"update_enabled": update, "values": []},
            }
        ],
        "rng": {"state": "fixed"},
    }


def tick_row(tick, age, update=True, direction=1):
    return {
        "tick": tick,
        "events": [],
        "decisions": [],
        "state": state(tick, age, update),
        "controller": {"direction": direction},
        "interventions_after": (
            ["reverse_water_before_53", "reverse_predator_route_before_53", "capture_checkpoint_before_53"]
            if tick == 52 else ["reverse_water_before_79"] if tick == 78 else []
        ),
    }


def valid_complete_seed(seed=44001):
    uninterrupted = [tick_row(t, t) for t in range(1, 105)]
    resumed = [tick_row(t, t, direction=-1) for t in range(53, 105)]
    # The uninterrupted branch also uses the reversed route after the split.
    for row in uninterrupted[52:]:
        row["controller"] = {"direction": -1}
    for row in resumed:
        if row["tick"] != 78:
            row["interventions_after"] = []
    checkpoint = state(52, 52)
    frozen_initial = state(52, 52, update=False)
    frozen = [tick_row(t, t, update=False, direction=-1) for t in range(53, 105)]
    for row in frozen:
        if row["tick"] != 78:
            row["interventions_after"] = []
    return {
        "schema": "curious-lifetime-lab-v1",
        "seed": seed,
        "status": "completed",
        "initial_state": state(0, 0),
        "uninterrupted": uninterrupted,
        "checkpoint_state": checkpoint,
        "checkpoint_controller": {"direction": -1},
        "resumed_initial_state": copy.deepcopy(checkpoint),
        "frozen_initial_state": frozen_initial,
        "resumed": resumed,
        "frozen": frozen,
        "physical_ticks": 208,
    }


class LifetimeContinuityValidationTests(unittest.TestCase):
    def test_valid_exact_replay_and_single_flag_frozen_branch(self):
        result = validate_seed(valid_complete_seed(), 44001)
        self.assertEqual(result["physical_ticks"], 208)
        self.assertEqual(result["final_age"], 104)

    def test_tampered_replay_state_is_rejected(self):
        row = valid_complete_seed()
        row["resumed"][10]["state"]["rng"]["state"] = "tampered"
        with self.assertRaisesRegex(ValueError, "differs from uninterrupted"):
            validate_seed(row, 44001)

    def test_frozen_branch_cannot_change_values_or_visits(self):
        row = valid_complete_seed()
        row["frozen"][0]["state"]["entities"][0]["learner"]["values"] = [
            {"pos": {"x": 1, "y": 3}, "value": 4, "visits": 1}
        ]
        with self.assertRaisesRegex(ValueError, "frozen branch changed learned values or visits"):
            validate_seed(row, 44001)

    def test_branch_report_uses_observed_resolver_cost_and_water_events(self):
        row = valid_complete_seed()
        row["uninterrupted"][0]["events"] = [
            {"EntityDrank": {"id": 1, "amount": 3}},
            {"EntityMoved": {"id": 1, "energy_cost": 2}},
            {"EntityAttacked": {"target": 1, "damage": 5}},
        ]
        result = validate_seed(row, 44001)
        first_window = result["branches"]["uninterrupted"][0]
        self.assertEqual(first_window["water_acquired"], 3)
        self.assertEqual(first_window["movement_energy"], 2)
        self.assertEqual(first_window["received_damage"], 5)

    def test_pre_split_natural_death_is_not_replaced_or_resumed(self):
        row = {
            "schema": "curious-lifetime-lab-v1",
            "seed": 44001,
            "status": "died_before_checkpoint",
            "initial_state": state(0, 0),
            "uninterrupted": [tick_row(t, t) for t in range(1, 52)],
            "checkpoint_state": None,
            "checkpoint_controller": None,
            "resumed_initial_state": None,
            "frozen_initial_state": None,
            "resumed": [],
            "frozen": [],
            "physical_ticks": 51,
        }
        row["uninterrupted"][-1]["state"]["entities"][0]["life"]["health"] = 0
        row["uninterrupted"][-1]["events"] = [{"EntityDied": {"id": 1}}]
        result = validate_seed(row, 44001)
        self.assertFalse(result["checkpoint_available"])
        self.assertEqual(result["final_age"], 51)

    def test_death_at_split_is_terminal_not_resumed(self):
        row = {
            "schema": "curious-lifetime-lab-v1",
            "seed": 44001,
            "status": "died_at_checkpoint",
            "initial_state": state(0, 0),
            "uninterrupted": [tick_row(t, t) for t in range(1, 53)],
            "checkpoint_state": None,
            "checkpoint_controller": None,
            "resumed_initial_state": None,
            "frozen_initial_state": None,
            "resumed": [],
            "frozen": [],
            "physical_ticks": 52,
        }
        row["uninterrupted"][-1]["state"]["entities"][0]["life"]["health"] = 0
        row["uninterrupted"][-1]["events"] = [{"EntityDied": {"id": 1}}]
        self.assertEqual(validate_seed(row, 44001)["final_age"], 52)

    def test_missing_checkpoint_must_not_claim_death_after_split(self):
        row = {
            "schema": "curious-lifetime-lab-v1",
            "seed": 44001,
            "status": "died_before_checkpoint",
            "initial_state": state(0, 0),
            "uninterrupted": [tick_row(t, t) for t in range(1, 53)],
            "checkpoint_state": None,
            "checkpoint_controller": None,
            "resumed_initial_state": None,
            "frozen_initial_state": None,
            "resumed": [],
            "frozen": [],
            "physical_ticks": 52,
        }
        row["uninterrupted"][-1]["state"]["entities"][0]["life"]["health"] = 0
        row["uninterrupted"][-1]["events"] = [{"EntityDied": {"id": 1}}]
        with self.assertRaisesRegex(ValueError, "pre-checkpoint death must occur"):
            validate_seed(row, 44001)


if __name__ == "__main__":
    unittest.main()
