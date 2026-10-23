"""Targeted contract and process-budget tests for the herbivore lab runner."""
import importlib.util
import json
from pathlib import Path
import sys
import tempfile
import time
import unittest

RUNNER_PATH = Path(__file__).resolve().parent / "run_herbivore_danger_lab.py"
SPEC = importlib.util.spec_from_file_location("herbivore_danger_runner", RUNNER_PATH)
runner = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(runner)


def state(tick, enabled=False, health=100):
    return {
        "version": 16, "tick": tick, "rng": 123, "entities": [
            {"id": 1, "pos": {"x": 3, "y": 3}, "species": "Herbivore", "generation": 0,
             "life": {"health": health, "max_health": 100, "energy": 100, "max_energy": 100,
                      "stamina": 100, "max_stamina": 100, "age": tick},
             "memory": {"refuge_outcomes_enabled": enabled, "refuge_outcomes": [],
                        "refuge_episode": None, "pending_refuge_closures": []}},
            {"id": 2, "pos": {"x": 3, "y": 0}, "species": "Carnivore", "life": {"health": 100}},
        ],
    }


def branch(enabled):
    initial = state(0, enabled)
    timeline = [{"tick": t, "events": [], "decisions": [], "episode_audit": [],
                 "state": state(t, enabled),
                 "memory_before": initial["entities"][0]["memory"] if t == 1 else state(t - 1, enabled)["entities"][0]["memory"],
                 "memory_after": state(t, enabled)["entities"][0]["memory"], "patrol_index": 0}
                for t in range(1, 101)]
    cp_state = timeline[49]["state"]
    cp = {"tick": 50, "serialized_state": cp_state, "restored_state": cp_state,
          "state_equal": True, "rng_before": 123, "rng_after": 123,
          "patrol_index_before": 0, "patrol_index_after": 0}
    return {"status": "completed_100_ticks", "initial_state": initial,
            "timeline": timeline, "checkpoint_roundtrip": cp}


def sample_row(seed=45001):
    return {"schema": runner.RAW_SCHEMA, "seed": seed, "baseline": branch(False),
            "refuge_memory": branch(True), "physical_ticks": 200,
            "paired_initial_state_equal_except_feature": True}


class ContractTests(unittest.TestCase):
    def test_valid_pair_and_full_ticks_validate(self):
        result = sample_row()
        runner.validate_initial_pair(result)
        parsed = runner.validate_branch(result["refuge_memory"], 45001, "refuge_memory")
        self.assertEqual(parsed["physical_ticks"], 100)
        self.assertEqual(parsed["closures"], 0)

    def test_initial_pair_rejects_any_change_beyond_feature_flag(self):
        result = sample_row()
        result["refuge_memory"]["initial_state"]["entities"][0]["life"]["energy"] = 99
        with self.assertRaisesRegex(ValueError, "beyond refuge feature flag"):
            runner.validate_initial_pair(result)

    def test_roundtrip_requires_complete_state_and_rng_equality(self):
        value = branch(False)
        value["checkpoint_roundtrip"]["rng_after"] = 124
        with self.assertRaisesRegex(ValueError, "RNG changed"):
            runner.validate_branch(value, 45001, "baseline")

    def test_movement_energy_must_match_resolver_event(self):
        value = branch(False)
        trace = {"tick": 1, "id": 1, "pos": {"x": 3, "y": 3},
                 "before": {"health": 100, "energy": 100, "max_energy": 100, "stamina": 100, "max_stamina": 100},
                 "perception": {"entities": [{"id": 2, "species": "Carnivore", "steps": ["Up"]}],
                                "shelters": [["Left", "Left"], ["Right", "Right"]]},
                 "decision": {"reason": "SeekShelter", "intent": {"Move": {"steps": ["Left"]}}}}
        value["timeline"][0]["decisions"] = [trace]
        value["timeline"][0]["events"] = [{"EntityMoved": {
            "id": 1, "energy_cost": 7, "from": {"x": 3, "y": 3}, "to": {"x": 2, "y": 3}}}]
        value["timeline"][0]["episode_audit"] = [{
            "id": 1, "threat_id": 2, "threat_species": "Carnivore",
            "context_key": {"threat_id": 2, "threat_species": "Carnivore", "bearing": 0,
                             "threat_distance": 0, "energy_quartile": 3, "stamina_quartile": 3,
                             "refuge_options": [{"x": 1, "y": 3}, {"x": 5, "y": 3}]},
            "started_tick": 1, "ended_tick": 1, "endpoint": {"x": 1, "y": 3},
            "endpoint_resolution": "unique", "endpoint_candidates": [{"x": 1, "y": 3}, {"x": 5, "y": 3}],
            "final_pos": {"x": 3, "y": 3}, "closure": "target_changed", "completed": False,
            "safe_arrival": False, "attribution_known": True, "threat_damage": 0,
            "other_damage": 0, "movement_energy": 6, "unattributed_health_loss": 0,
            "starting_health": 100, "final_health": 100, "initial_reason": "SeekShelter",
            "initial_intent": {"Move": {"steps": ["Left"]}}, "initial_selected_path": ["Left"],
            "visible_threat_position": {"x": 3, "y": 2}}]
        with self.assertRaisesRegex(ValueError, "movement energy disagrees"):
            runner.validate_branch(value, 45001, "baseline")

    def test_same_tick_predecision_and_death_closures_charge_different_moves(self):
        value = branch(False)
        value["status"] = "natural_death"
        value["timeline"] = value["timeline"][:2]
        value["checkpoint_roundtrip"] = None
        value["timeline"][0]["state"] = state(1, False, health=20)
        value["timeline"][0]["state"]["entities"][0]["pos"] = {"x": 3, "y": 2}
        value["timeline"][0]["events"] = [{"EntityMoved": {"id": 1, "energy_cost": 5,
            "from": {"x": 3, "y": 3}, "to": {"x": 3, "y": 2}}}]
        value["timeline"][0]["decisions"] = [{"tick": 1, "id": 1, "pos": {"x": 3, "y": 3},
            "before": {"health": 20, "energy": 100, "max_energy": 100, "stamina": 100, "max_stamina": 100}, "perception": {"entities": [{"id": 2, "species": "Carnivore", "steps": ["Up"]}],
                "shelters": [["Left", "Left"], ["Right", "Right"]]},
            "decision": {"reason": "SeekShelter", "intent": {"Move": {"steps": ["Right"]}}}}]
        value["timeline"][1]["state"] = state(2, False, health=0)
        value["timeline"][1]["state"]["entities"][0]["pos"] = {"x": 2, "y": 3}
        value["timeline"][1]["events"] = [
            {"EntityMoved": {"id": 1, "energy_cost": 7, "from": {"x": 3, "y": 2}, "to": {"x": 2, "y": 3}}},
            {"EntityAttacked": {"attacker": 2, "target": 1, "damage": 20}},
            {"EntityDied": {"id": 1}},
        ]
        value["timeline"][1]["decisions"] = [{"tick": 2, "id": 1, "pos": {"x": 3, "y": 2},
            "before": {"health": 20, "energy": 100, "max_energy": 100, "stamina": 100, "max_stamina": 100}, "perception": {"entities": [{"id": 2, "species": "Carnivore", "steps": ["Up"]}],
                "shelters": [["DownLeft", "Left"], ["DownRight", "Right"]]},
            "decision": {"reason": "SeekShelter", "intent": {"Move": {"steps": ["DownLeft"]}}}}]
        options = [{"x": 1, "y": 3}, {"x": 5, "y": 3}]
        context = {"threat_id": 2, "threat_species": "Carnivore", "bearing": 0,
                   "threat_distance": 0, "energy_quartile": 3, "stamina_quartile": 3,
                   "refuge_options": options}
        def episode(start, endpoint, path, closure, final, move, damage, start_health, final_health):
            return {"id": 1, "threat_id": 2, "threat_species": "Carnivore", "context_key": context,
                    "started_tick": start, "ended_tick": 2, "endpoint": endpoint,
                    "endpoint_resolution": "unique", "endpoint_candidates": options,
                    "final_pos": final, "closure": closure, "completed": False, "safe_arrival": False,
                    "attribution_known": True, "threat_damage": damage, "other_damage": 0,
                    "unattributed_health_loss": 0, "movement_energy": move,
                    "starting_health": start_health, "final_health": final_health,
                    "initial_reason": "SeekShelter", "initial_intent": {"Move": {"steps": path}},
                    "initial_selected_path": path, "visible_threat_position": {"x": 3, "y": 2} if start == 1 else {"x": 3, "y": 1}}
        value["timeline"][1]["episode_audit"] = [
            episode(1, options[1], ["Right"], "target_changed", {"x": 3, "y": 2}, 5, 0, 20, 20),
            episode(2, options[0], ["DownLeft"], "died", {"x": 2, "y": 3}, 7, 20, 20, 0),
        ]
        parsed = runner.validate_branch(value, 45001, "baseline")
        self.assertEqual([e["movement_energy"] for e in parsed["episodes"]], [5, 7])
        self.assertEqual([e["threat_damage"] for e in parsed["episodes"]], [0, 20])

    def test_terminal_death_before_intent_uses_death_event_without_fake_trace(self):
        value = branch(False)
        value["status"] = "natural_death"
        value["timeline"] = value["timeline"][:2]
        value["checkpoint_roundtrip"] = None
        options = [{"x": 1, "y": 3}, {"x": 5, "y": 3}]
        context = {"threat_id": 2, "threat_species": "Carnivore", "bearing": 0,
                   "threat_distance": 0, "energy_quartile": 3, "stamina_quartile": 3,
                   "refuge_options": options}
        trace = {"tick": 1, "id": 1, "pos": {"x": 3, "y": 3},
                 "before": {"health": 20, "energy": 100, "max_energy": 100, "stamina": 100, "max_stamina": 100},
                 "perception": {"entities": [{"id": 2, "species": "Carnivore", "steps": ["Up"]}],
                                "shelters": [["Left", "Left"], ["Right", "Right"]]},
                 "decision": {"reason": "SeekShelter", "intent": {"Move": {"steps": ["Left"]}}}}
        value["timeline"][0]["state"] = state(1, False, health=20)
        value["timeline"][0]["state"]["entities"][0]["pos"] = {"x": 3, "y": 2}
        value["timeline"][0]["decisions"] = [trace]
        value["timeline"][0]["events"] = [{"EntityMoved": {"id": 1, "energy_cost": 5,
            "from": {"x": 3, "y": 3}, "to": {"x": 3, "y": 2}}}]
        value["timeline"][1]["state"] = state(2, False, health=0)
        value["timeline"][1]["state"]["entities"][0]["pos"] = {"x": 3, "y": 2}
        value["timeline"][1]["decisions"] = []
        value["timeline"][1]["events"] = [
            {"EntityAttacked": {"attacker": 2, "target": 1, "damage": 20}},
            {"EntityDied": {"id": 1}},
        ]
        value["timeline"][1]["episode_audit"] = [{
            "id": 1, "threat_id": 2, "threat_species": "Carnivore", "context_key": context,
            "started_tick": 1, "ended_tick": 2, "endpoint": options[0],
            "endpoint_resolution": "unique", "endpoint_candidates": options,
            "final_pos": {"x": 3, "y": 2}, "closure": "died", "completed": False,
            "safe_arrival": False, "attribution_known": True, "threat_damage": 20,
            "other_damage": 0, "unattributed_health_loss": 0, "movement_energy": 5,
            "starting_health": 20, "final_health": 0, "initial_reason": "SeekShelter",
            "initial_intent": {"Move": {"steps": ["Left"]}}, "initial_selected_path": ["Left"],
            "visible_threat_position": {"x": 3, "y": 2}}]
        parsed = runner.validate_branch(value, 45001, "baseline")
        self.assertEqual(parsed["status"], "natural_death")
        self.assertEqual(parsed["episodes"][0]["threat_damage"], 20)

    def test_arrived_engine_event_with_ambiguous_observer_prefix_stays_unmapped_and_censored(self):
        value = branch(True)
        options = [{"x": 1, "y": 3}, {"x": 2, "y": 3}]
        context = {"threat_id": 2, "threat_species": "Carnivore", "bearing": 0,
                   "threat_distance": 0, "energy_quartile": 3, "stamina_quartile": 3,
                   "refuge_options": options}
        engine_key = {**context, "endpoint": options[0]}
        stored_memory = state(0, True)["entities"][0]["memory"]
        stored_memory["refuge_outcomes"].append({
            "key": engine_key, "last_observed_tick": 1, "completed": 1, "safe_arrivals": 1,
            "threat_damage": 0, "movement_energy": 5,
        })
        value["timeline"][0]["state"] = state(1, True)
        value["timeline"][0]["state"]["entities"][0]["pos"] = options[0]
        value["timeline"][0]["state"]["entities"][0]["memory"] = stored_memory
        value["timeline"][0]["memory_after"] = stored_memory
        value["timeline"][0]["events"] = [
            {"EntityMoved": {"id": 1, "energy_cost": 5, "from": {"x": 3, "y": 3}, "to": options[0]}},
            {"RefugeEpisodeEnded": {"id": 1, "threat_id": 2, "threat_species": "Carnivore",
                "context_key": engine_key, "started_tick": 1, "ended_tick": 1,
                "endpoint": options[0], "final_pos": options[0], "closure": "Arrived",
                "completed": True, "safe_arrival": True, "attribution_known": True,
                "threat_damage": 0, "other_damage": 0, "movement_energy": 5,
                "unattributed_health_loss": 0}},
        ]
        value["timeline"][0]["decisions"] = [{"tick": 1, "id": 1, "pos": {"x": 3, "y": 3},
            "before": {"health": 100, "energy": 100, "max_energy": 100, "stamina": 100, "max_stamina": 100},
            "perception": {"entities": [{"id": 2, "species": "Carnivore", "steps": ["Up"]}],
                           "shelters": [["Left"], ["Left", "Left"]]},
            "decision": {"reason": "SeekShelter", "intent": {"Move": {"steps": ["Left"]}}}}]
        value["timeline"][0]["episode_audit"] = [{
            "id": 1, "threat_id": 2, "threat_species": "Carnivore", "context_key": context,
            "started_tick": 1, "ended_tick": 1, "endpoint": None,
            "endpoint_resolution": "ambiguous_prefix", "endpoint_candidates": options,
            "final_pos": options[0], "closure": "endpoint_ambiguous", "completed": False,
            "safe_arrival": False, "attribution_known": True, "threat_damage": 0,
            "other_damage": 0, "unattributed_health_loss": 0, "movement_energy": 5,
            "starting_health": 100, "final_health": 100, "initial_reason": "SeekShelter",
            "initial_intent": {"Move": {"steps": ["Left"]}}, "initial_selected_path": ["Left"],
            "visible_threat_position": {"x": 3, "y": 2}}]
        for row in value["timeline"][1:]:
            row["state"]["entities"][0]["memory"] = stored_memory
            row["memory_before"] = stored_memory
            row["memory_after"] = stored_memory
        checkpoint_state = value["timeline"][49]["state"]
        value["checkpoint_roundtrip"] = {
            "tick": 50, "serialized_state": checkpoint_state, "restored_state": checkpoint_state,
            "state_equal": True, "rng_before": 123, "rng_after": 123,
            "patrol_index_before": 0, "patrol_index_after": 0,
        }
        parsed = runner.validate_branch(value, 45001, "refuge_memory")
        self.assertEqual(parsed["episodes"][0]["closure"], "endpoint_ambiguous")
        self.assertFalse(parsed["episodes"][0]["safe_arrival"])
        self.assertEqual(parsed["engine_events_unmapped_to_unique_observer_endpoint"][0]["engine_closure"], "Arrived")

    def test_unique_engine_closure_names_normalize_to_observer_snake_case(self):
        for audit_closure, engine_closure in (("target_changed", "TargetChanged"),
                                              ("threat_lost", "ThreatLost")):
            with self.subTest(closure=audit_closure):
                value = branch(True)
                options = [{"x": 1, "y": 3}, {"x": 5, "y": 3}]
                context = {"threat_id": 2, "threat_species": "Carnivore", "bearing": 0,
                           "threat_distance": 0, "energy_quartile": 3, "stamina_quartile": 3,
                           "refuge_options": options}
                key = {**context, "endpoint": options[0]}
                first_trace = {"tick": 1, "id": 1, "pos": {"x": 3, "y": 3},
                    "before": {"health": 100, "energy": 100, "max_energy": 100, "stamina": 100, "max_stamina": 100},
                    "perception": {"entities": [{"id": 2, "species": "Carnivore", "steps": ["Up"]}],
                                   "shelters": [["Left", "Left"], ["Right", "Right"]]},
                    "decision": {"reason": "SeekShelter", "intent": {"Move": {"steps": ["Left"]}}}}
                if audit_closure == "target_changed":
                    end_perception = {"entities": [{"id": 2, "species": "Carnivore", "steps": ["Up"]}],
                                      "shelters": [["DownLeft", "Left"], ["DownRight", "Right"]]}
                    end_decision = {"reason": "SeekShelter", "intent": {"Move": {"steps": ["DownRight"]}}}
                else:
                    end_perception = {"entities": [{"id": 2, "species": "Carnivore", "steps": ["Up", "Up", "Up"]}],
                                      "shelters": [["DownLeft", "Left"], ["DownRight", "Right"]]}
                    end_decision = {"reason": "SeekFood", "intent": {"Idle": {"duration": 1}}}
                end_trace = {"tick": 2, "id": 1, "pos": {"x": 3, "y": 2},
                    "before": {"health": 100, "energy": 100, "max_energy": 100, "stamina": 100, "max_stamina": 100},
                    "perception": end_perception, "decision": end_decision}
                value["timeline"][0]["state"] = state(1, True)
                value["timeline"][0]["state"]["entities"][0]["pos"] = {"x": 3, "y": 2}
                value["timeline"][0]["decisions"] = [first_trace]
                value["timeline"][0]["events"] = [{"EntityMoved": {"id": 1, "energy_cost": 5,
                    "from": {"x": 3, "y": 3}, "to": {"x": 3, "y": 2}}}]
                value["timeline"][1]["state"] = state(2, True)
                value["timeline"][1]["state"]["entities"][0]["pos"] = {"x": 3, "y": 2}
                value["timeline"][1]["decisions"] = [end_trace]
                value["timeline"][1]["events"] = [{"RefugeEpisodeEnded": {
                    "id": 1, "threat_id": 2, "threat_species": "Carnivore", "context_key": key,
                    "started_tick": 1, "ended_tick": 2, "endpoint": options[0],
                    "final_pos": {"x": 3, "y": 2}, "closure": engine_closure,
                    "completed": False, "safe_arrival": False, "attribution_known": True,
                    "threat_damage": 0, "other_damage": 0, "movement_energy": 5,
                    "unattributed_health_loss": 0}}]
                value["timeline"][1]["episode_audit"] = [{
                    "id": 1, "threat_id": 2, "threat_species": "Carnivore", "context_key": context,
                    "started_tick": 1, "ended_tick": 2, "endpoint": options[0],
                    "endpoint_resolution": "unique", "endpoint_candidates": options,
                    "final_pos": {"x": 3, "y": 2}, "closure": audit_closure, "completed": False,
                    "safe_arrival": False, "attribution_known": True, "threat_damage": 0,
                    "other_damage": 0, "unattributed_health_loss": 0, "movement_energy": 5,
                    "starting_health": 100, "final_health": 100, "initial_reason": "SeekShelter",
                    "initial_intent": {"Move": {"steps": ["Left"]}}, "initial_selected_path": ["Left"],
                    "visible_threat_position": {"x": 3, "y": 2}}]
                for row in value["timeline"][2:]:
                    row["state"]["entities"][0]["pos"] = {"x": 3, "y": 2}
                parsed = runner.validate_branch(value, 45001, "refuge_memory")
                self.assertEqual(parsed["episodes"][0]["closure"], audit_closure)
                self.assertEqual(parsed["engine_events_unmapped_to_unique_observer_endpoint"], [])

    def test_death_requires_terminal_zero_health_and_event(self):
        value = branch(False)
        value["status"] = "natural_death"
        value["timeline"] = value["timeline"][:2]
        value["timeline"][-1]["state"] = state(2, False, health=0)
        with self.assertRaisesRegex(ValueError, "EntityDied"):
            runner.validate_branch(value, 45001, "baseline")

    def test_hidden_patrol_cursor_is_rejected_inside_decision_records(self):
        value = branch(False)
        value["timeline"][0]["decisions"] = [{"subject": {"perception": {"patrol_index": 9}}}]
        with self.assertRaisesRegex(ValueError, "leaked"):
            runner.validate_branch(value, 45001, "baseline")


class BudgetTests(unittest.TestCase):
    def run_probe(self, code, timeout=0.25, byte_limit=64):
        temp = tempfile.TemporaryDirectory()
        root = Path(temp.name)
        raw, stderr = root / "raw.jsonl", root / "stderr.log"
        result = runner.capture_process([sys.executable, "-c", code], root, raw, stderr,
                                        timeout=timeout, byte_limit=byte_limit)
        return temp, result, raw

    def test_timeout_preserves_partial_stdout_and_fails(self):
        temp, result, raw = self.run_probe("import os,time; os.write(1,b'{partial'); time.sleep(3)")
        try:
            self.assertEqual(result["status"], "failed")
            self.assertTrue(result["timed_out"])
            self.assertEqual(raw.read_bytes(), b"{partial")
            self.assertLess(result["elapsed_seconds"], 1.5)
        finally:
            temp.cleanup()

    def test_output_budget_failure_preserves_only_bounded_prefix(self):
        temp, result, raw = self.run_probe("import os; os.write(1,b'x'*1000)", byte_limit=64)
        try:
            self.assertEqual(result["status"], "failed")
            self.assertTrue(result["output_limit_exceeded"])
            self.assertLessEqual(raw.stat().st_size, 64)
        finally:
            temp.cleanup()

    def test_timeout_still_applies_after_child_closes_stdout(self):
        temp, result, raw = self.run_probe("import os,time; os.close(1); time.sleep(3)", timeout=0.1)
        try:
            self.assertEqual(result["status"], "failed")
            self.assertTrue(result["timed_out"])
            self.assertEqual(raw.read_bytes(), b"")
        finally:
            temp.cleanup()


if __name__ == "__main__":
    unittest.main()
