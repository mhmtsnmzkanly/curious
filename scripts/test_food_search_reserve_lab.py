#!/usr/bin/env python3
"""Python validation tests for food-search reserve registration and raw evidence."""
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

SCRIPT = Path(__file__).with_name("run_food_search_reserve_lab.py")
spec = importlib.util.spec_from_file_location("food_search_reserve_runner_test", SCRIPT)
runner = importlib.util.module_from_spec(spec)
assert spec.loader is not None
spec.loader.exec_module(runner)


class FoodReserveRunnerTests(unittest.TestCase):
    def registration(self, identity, fixtures):
        return {
            "status": "registered", "root_reviewed": True, "source": identity,
            "scenario_sha256": fixtures, "seed_roles": runner.expected_seed_roles(),
            "binary_sha256": "a" * 64,
            "ticks_per_world": 300, "arms": list(runner.ARMS),
            "habitats": list(runner.HABITATS), "world_count": 36,
            "phase_seconds": 300, "child_seconds": 60,
            "child_address_space_bytes": 1024**3, "aggregate_output_bytes": 256 * 1024**2,
            "max_world_output_bytes": 16 * 1024**2, "max_identities": 10000,
            "max_events_per_tick": 20000, "mutation_per_mille": 0,
            "genetics_enabled": True, "candidate_flag": "food_search_reserve_enabled",
        }

    def test_frozen_dimensions_and_seed_roles(self):
        roles = runner.expected_seed_roles()
        self.assertEqual(len(roles), 18)
        self.assertEqual(len(runner.ARMS), 2)
        self.assertEqual(len(roles) * len(runner.ARMS), 36)
        self.assertEqual(runner.TICKS * 36, 10800)
        self.assertEqual(runner.HABITATS["v1_unique_food"][0], "v1")
        self.assertEqual(runner.HABITATS["sparse_resource"][0], "scenarios/ecology-traits-lab.json")
        self.assertEqual(sorted({r["seed"] for r in roles}), list(range(62401, 62407)))

    def test_registration_accepts_exact_design_and_rejects_changed_horizon(self):
        identity = {"revision":"abc","clean":True,"file_sha256":{},"content_sha256":"def"}
        fixtures = {name: "0" * 64 if path != "v1" else None for name,(path,_) in runner.HABITATS.items()}
        registration = self.registration(identity, fixtures)
        runner.validate_registration(registration, identity, fixtures, "a" * 64)
        registration["world_count"] = 35
        with self.assertRaisesRegex(ValueError, "world_count"):
            runner.validate_registration(registration, identity, fixtures, "a" * 64)

    def test_registration_requires_clean_source_and_pinned_binary(self):
        identity = {"revision":"abc","clean":False,"file_sha256":{},"content_sha256":"def"}
        fixtures = {name: "0" * 64 if path != "v1" else None for name,(path,_) in runner.HABITATS.items()}
        registration = self.registration(identity, fixtures)
        with self.assertRaisesRegex(ValueError, "clean committed"):
            runner.validate_registration(registration, identity, fixtures, "a" * 64)
        identity["clean"] = True
        with self.assertRaisesRegex(ValueError, "binary SHA"):
            runner.validate_registration(registration, identity, fixtures, "b" * 64)

    def test_seed_scan_uses_unique_seed_set_and_large_line_bound(self):
        self.assertEqual(runner.SEED_SCAN_MAX_LINE_BYTES, 64 * 1024 * 1024)
        with tempfile.TemporaryDirectory() as temp:
            archive = Path(temp)
            roles = [{"habitat":"meadow","seed":62401},{"habitat":"sparse_resource","seed":62401}]
            self.assertEqual(runner.scan_seed_roles(archive, roles)["unseen"], [62401])
            (archive / "history.json").write_text('{"seed":62401}\n', encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "already occur"):
                runner.scan_seed_roles(archive, roles)

    def synthetic_rows(self):
        initial = {"policy":{},"entities":[{"id":1}]}
        header = {"type":"header","schema":"curious-food-search-reserve-v1-jsonl","habitat":"meadow","seed":62401,"arm":"baseline","initial_state":initial,"candidate_policy":{"food_search_reserve_enabled":False}}
        ticks = [{"type":"tick","tick":tick,"birth_lineage":[],"mating_events":[],"death_lineage":[],"focal_events":[],"reason_counts":{},"reserve_interventions":[],"focal_zero_energy_visible_food_approach_failures":0,"focal_underfunded_visible_food_approach_failures":0} for tick in range(1,runner.TICKS+1)]
        summary = {"type":"summary","ticks_completed":runner.TICKS,"checkpoint_restore_valid":True,"lineage_issues":0,"unknown_generation_births":0,"births_by_species":{},"deaths_by_species":{},"births_by_generation":{},"deaths_by_generation":{},"final_alive":{"Herbivore":1},"living_entity_ticks":runner.TICKS,"plant_food_gained_energy":0,"movement_energy_paid":0,"zero_energy_alive_tick_sum":0,"f1_births":0,"f2_births":0,"focal_approach_out_of_range_count":0,"focal_zero_energy_visible_food_approach_failures":0,"focal_underfunded_visible_food_approach_failures":0,"reserve_intervention_count":0}
        return [header,*ticks,summary]

    def test_raw_validator_requires_complete_ordered_ticks_and_lineage(self):
        rows = self.synthetic_rows()
        with tempfile.TemporaryDirectory() as temp:
            raw = Path(temp) / "run.jsonl"
            raw.write_text("".join(json.dumps(row)+"\n" for row in rows), encoding="utf-8")
            self.assertEqual(runner._read_run(raw,62401,"meadow","baseline")["summary"]["ticks_completed"],runner.TICKS)
            broken = list(rows)
            broken[3] = {**broken[3], "reason_counts":None}
            raw.write_text("".join(json.dumps(row)+"\n" for row in broken), encoding="utf-8")
            with self.assertRaisesRegex(ValueError,"required per-tick"):
                runner._read_run(raw,62401,"meadow","baseline")
            broken = list(rows)
            broken[1] = {**broken[1],"tick":2}
            raw.write_text("".join(json.dumps(row)+"\n" for row in broken), encoding="utf-8")
            with self.assertRaisesRegex(ValueError,"tick sequence"):
                runner._read_run(raw,62401,"meadow","baseline")

    def test_pair_validator_allows_only_registered_flag_difference(self):
        base = {"header":{"habitat":"meadow","seed":62401,"arm":"baseline","candidate_policy":{"food_search_reserve_enabled":False},"initial_state":{"policy":{},"entities":[{"id":1}]}},"summary":{"living_entity_ticks":10,"plant_food_gained_energy":3,"movement_energy_paid":4,"zero_energy_alive_tick_sum":2,"f1_births":0,"f2_births":0,"focal_approach_out_of_range_count":1,"focal_first_zero_energy_tick":None,"f1_breeders":[2],"f2_breeders":[],"living_f1_breeders":[2],"living_f2_breeders":[],"final_alive":{"Herbivore":2}},"rows":[]}
        candidate = {"header":{"habitat":"meadow","seed":62401,"arm":"candidate","candidate_policy":{"food_search_reserve_enabled":True},"initial_state":{"policy":{"food_search_reserve_enabled":True},"entities":[{"id":1}]}},"summary":{"living_entity_ticks":12,"plant_food_gained_energy":5,"movement_energy_paid":4,"zero_energy_alive_tick_sum":1,"f1_births":0,"f2_births":0,"focal_approach_out_of_range_count":0,"focal_first_zero_energy_tick":25,"f1_breeders":[2,3,4],"f2_breeders":[5],"living_f1_breeders":[2,3],"living_f2_breeders":[5],"final_alive":{"Herbivore":3}},"rows":[]}
        pair = runner.validate_pair(base,candidate,"meadow",62401)
        self.assertEqual(pair["candidate_minus_baseline"]["living_entity_ticks"],2)
        self.assertEqual(pair["candidate_minus_baseline"]["f1_breeders_count"],2)
        self.assertEqual(pair["candidate_minus_baseline"]["f2_breeders_count"],1)
        self.assertEqual(pair["candidate_minus_baseline"]["final_alive.Herbivore"],1)
        self.assertNotIn("focal_first_zero_energy_tick",pair["candidate_minus_baseline"])
        candidate["header"]["initial_state"]["entities"][0]["id"] = 99
        with self.assertRaisesRegex(ValueError,"initial State differs"):
            runner.validate_pair(base,candidate,"meadow",62401)


if __name__ == "__main__":
    unittest.main()
