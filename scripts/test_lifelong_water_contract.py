"""Semantic regressions for the lifetime-water analysis contract.

These tests use handcrafted serialized rows only; they do not run the probe or
consume calibration/formal world ticks.
"""
from __future__ import annotations

import importlib.util
from pathlib import Path
import unittest


RUNNER_PATH = Path(__file__).with_name("run_lifelong_water_lab.py")
SPEC = importlib.util.spec_from_file_location("lifelong_water_runner", RUNNER_PATH)
assert SPEC is not None and SPEC.loader is not None
runner = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(runner)


def actor(actor_id: int, *, x: int, y: int, health: int = 100,
          energy: int = 80, water: int = 40) -> dict:
    return {
        "id": actor_id,
        "species": "Herbivore",
        "phase": "Active",
        "pos": {"x": x, "y": y},
        "life": {
            "health": health,
            "energy": energy,
            "water": water,
            "death_cause": None,
        },
    }


def state(*, health: int = 100, energy: int = 80, water: int = 40,
          x: int = 2, y: int = 2) -> dict:
    return {
        "tick": 201,
        "rng": 123,
        "entities": [
            actor(1, x=x, y=y, health=health, energy=energy, water=water),
            actor(2, x=2, y=3),
        ],
        "resources": [],
    }


def branch(*, water: int = 0, movement: int = 0,
           health: int = 100, energy: int = 80,
           selected: tuple[int, int] = (0, 2),
           survived_ticks: int = 1) -> dict:
    start = state()
    end = state(health=health, energy=energy, water=40)
    events = []
    if water:
        events.append({"EntityDrank": {"id": 1, "pos": {"x": selected[0], "y": selected[1]}, "amount": water}})
    if movement:
        events.append({"EntityMoved": {"id": 1, "from": {"x": 2, "y": 2}, "to": {"x": selected[0], "y": selected[1]}, "energy_cost": movement}})
    rows = []
    for tick in range(201, 201 + survived_ticks):
        tick_start = state()
        tick_end = state(health=health, energy=energy)
        rows.append({
            "tick": tick,
            "state_before": tick_start,
            "rng_before": 123,
            "events": events if tick == 201 else [],
            "decisions": [],
            # This deliberately varies by branch. It is not a physical outcome.
            "focal_choice": {"selected_target": {"x": selected[0], "y": selected[1]}},
            "state_after": tick_end,
            "rng_after": 123,
        })
    return {
        "status": "synthetic_fixture",
        "state_at_tick_201_before_world_tick": start,
        "ticks": rows,
    }


class LifetimeWaterContractTests(unittest.TestCase):
    def test_location_change_with_identical_physical_totals_is_not_a_benefit(self) -> None:
        online = branch(selected=(0, 2))
        trained = branch(selected=(4, 2))

        totals = runner.compare_physical_totals(0, 0, 0, 0)
        vector = runner.common_horizon(online, trained)

        self.assertFalse(totals["physical_outcome_difference"])
        self.assertFalse(totals["position_or_timing_difference_alone_counts"])
        self.assertTrue(vector["no_negative_component"])
        self.assertFalse(vector["strict_positive_component"])
        self.assertFalse(vector["net_physical_benefit_vector"])

    def test_same_water_with_lower_paid_movement_energy_is_a_strict_benefit(self) -> None:
        online = branch(water=10, movement=2, energy=77)
        trained = branch(water=10, movement=4, energy=75)

        totals = runner.compare_physical_totals(10, 10, 2, 4)
        vector = runner.common_horizon(online, trained)

        self.assertEqual(totals["water_amount_delta"], 0)
        self.assertEqual(totals["movement_energy_paid_delta"], -2)
        self.assertTrue(totals["physical_outcome_difference"])
        self.assertTrue(vector["no_negative_component"])
        self.assertTrue(vector["strict_positive_component"])
        self.assertTrue(vector["net_physical_benefit_vector"])

    def test_water_gain_with_health_harm_is_reported_as_tradeoff(self) -> None:
        online = branch(water=10, movement=2, health=80, energy=77)
        trained = branch(water=5, movement=2, health=100, energy=77)

        vector = runner.common_horizon(online, trained)

        self.assertEqual(vector["differences_online_minus_trained_frozen"]["water_acquired"], 5)
        self.assertEqual(vector["differences_online_minus_trained_frozen"]["health_at_horizon"], -20)
        self.assertFalse(vector["no_negative_component"])
        self.assertFalse(vector["net_physical_benefit_vector"])

    def test_only_ablated_arm_clears_the_focal_values(self) -> None:
        checkpoint = {
            "tick": 200,
            "rng": 987,
            "entities": [{
                "id": 1,
                "pos": {"x": 2, "y": 2},
                "life": {"age": 200, "health": 90, "energy": 70, "water": 41},
                "learner": {
                    "enabled": True,
                    "update_enabled": True,
                    "values": [
                        {"pos": {"x": 0, "y": 2}, "value": 123, "visits": 4},
                        {"pos": {"x": 4, "y": 2}, "value": 87, "visits": 2},
                    ],
                    "episode": {"target": {"x": 0, "y": 2}, "ticks": 1,
                                "movement_energy": 0, "incoming_damage": 0},
                    "last_episode": None,
                },
            }],
        }
        original = checkpoint["entities"][0]["learner"].copy()

        online = runner.treat_state(checkpoint, "online")["entities"][0]["learner"]
        trained = runner.treat_state(checkpoint, "trained_frozen")["entities"][0]["learner"]
        ablated = runner.treat_state(checkpoint, "ablated_frozen")["entities"][0]["learner"]
        rule_based = runner.treat_state(checkpoint, "rule_based")["entities"][0]["learner"]

        self.assertEqual(online["values"], original["values"])
        self.assertEqual(trained["values"], original["values"])
        self.assertEqual(rule_based["values"], original["values"])
        self.assertEqual(ablated["values"], [])
        self.assertEqual(trained["episode"], original["episode"])
        self.assertEqual(ablated["episode"], original["episode"])
        self.assertEqual(checkpoint["entities"][0]["learner"], original)

    def test_stale_episode_is_fresh_but_visible_target_continuation_is_not(self) -> None:
        choice = {
            "base_reason": "SeekWater",
            "active_episode_before": {"target": {"x": 1, "y": 1}},
            "candidates": [
                {"pos": {"x": 0, "y": 2}, "amount": 3, "steps_len": 2},
                {"pos": {"x": 4, "y": 2}, "amount": 4, "steps_len": 2},
            ],
        }
        self.assertTrue(runner.fresh_choice_expected(choice, True, "LearnedWater"))

        choice["active_episode_before"]["target"] = {"x": 0, "y": 2}
        self.assertFalse(runner.fresh_choice_expected(choice, True, "LearnedWater"))
        # The disabled rule-based arm ignores the inherited learner episode.
        self.assertTrue(runner.fresh_choice_expected(choice, False, "SeekWater"))

    def test_greedy_shadow_uses_values_not_the_logged_exploration_choice(self) -> None:
        choice = {
            "selection_kind": "explore",
            "selected_target": {"x": 4, "y": 2},
            "candidates": [
                {"pos": {"x": 0, "y": 2}, "amount": 8, "steps_len": 2},
                {"pos": {"x": 4, "y": 2}, "amount": 8, "steps_len": 2},
            ],
        }

        self.assertEqual(runner.greedy_target(choice, {(0, 2): 110, (4, 2): 90}), (0, 2))
        self.assertEqual(choice["selected_target"], {"x": 4, "y": 2})
        self.assertEqual(choice["selection_kind"], "explore")


if __name__ == "__main__":
    unittest.main()
