#!/usr/bin/env python3
"""Tests for the registered water-meal priority runner and evidence checks."""
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

SCRIPT = Path(__file__).with_name("run_water_meal_priority_lab.py")
spec = importlib.util.spec_from_file_location("water_meal_runner_test", SCRIPT)
runner = importlib.util.module_from_spec(spec)
assert spec.loader is not None
spec.loader.exec_module(runner)


class WaterMealRunnerTests(unittest.TestCase):
    def test_frozen_factorial_dimensions_and_heldout_roles(self):
        roles = runner.expected_seed_roles()
        self.assertEqual(len(roles), 18)
        self.assertEqual(runner.ARMS, ("control", "food_reserve_only", "water_meal_only", "bundle"))
        self.assertEqual(len(roles) * len(runner.ARMS), 72)
        self.assertEqual(runner.TICKS * 72, 21600)
        self.assertEqual(sorted({r["seed"] for r in roles}), list(range(62501, 62507)))

    def test_registration_binds_design_and_flags(self):
        identity = {"revision":"abc","clean":True,"file_sha256":{},"content_sha256":"def"}
        fixtures = {name: "0" * 64 if path != "v1" else None for name,(path,_) in runner.HABITATS.items()}
        registration = {
            "status":"registered","root_reviewed":True,"source":identity,
            "scenario_sha256":fixtures,"seed_roles":runner.expected_seed_roles(),"binary_sha256":"a"*64,
            "ticks_per_world":300,"arms":list(runner.ARMS),"habitats":list(runner.HABITATS),"world_count":72,
            "phase_seconds":300,"child_seconds":60,"child_address_space_bytes":1024**3,
            "aggregate_output_bytes":256*1024**2,"max_world_output_bytes":16*1024**2,
            "max_identities":10000,"max_events_per_tick":20000,"mutation_per_mille":0,
            "genetics_enabled":True,"candidate_flags":{"food_search_reserve_enabled":"factorial_existing_flag","water_meal_priority_enabled":"approved_candidate"}}
        runner.validate_registration(registration,identity,fixtures,"a"*64)
        registration["world_count"] = 36
        with self.assertRaisesRegex(ValueError,"world_count"):
            runner.validate_registration(registration,identity,fixtures,"a"*64)

    def test_seed_scan_unique_seed_set_and_large_line_bound(self):
        self.assertEqual(runner.SEED_SCAN_MAX_LINE_BYTES,64*1024*1024)
        with tempfile.TemporaryDirectory() as temp:
            roles=[{"habitat":"meadow","seed":62501},{"habitat":"sparse_resource","seed":62501}]
            self.assertEqual(runner.scan_seed_roles(Path(temp),roles)["unseen"],[62501])

    def test_verified_archive_index_preserves_and_checks_declared_seed_roles(self):
        index={"seeds":[3,8],"seed_roles":{"seed":[3],"seeds":[8]}}
        seeds,roles=runner._verified_index_seed_metadata(index,"example.jsonl.archive.json")
        self.assertEqual(seeds,[3,8])
        self.assertEqual(roles,index["seed_roles"])
        with self.assertRaisesRegex(RuntimeError,"differ from its seed_roles"):
            runner._verified_index_seed_metadata({"seeds":[3,8],"seed_roles":{"seed":[3]}},"example.jsonl.archive.json")

    def test_intervention_validator_requires_original_matched_water_and_actual_meal(self):
        record={"pre_choice":{"energy":1},"perception":{"waters":[{"amount":4,"steps":{"0":["Left","Left"]}}],"foods":[{"amount":3,"is_corpse":False,"corpse_id":None,"steps":{"0":["Up"]}}]},
                "original_selected_decision":{"reason":"LearnedWater","intent":{"Drink":{"at":{"0":["Left","Left"]}}}},
                "original_reason":"LearnedWater","original_selected_water_path":["Left","Left"],
                "original_water_amount":4,"actual_selected_food_amount":3,
                "actual_selected_food":{"steps":["Up"],"corpse_id":None},
                "actual_decision":{"reason":"EatBeforeWater","intent":{"Eat":{"at":{"0":["Up"]},"corpse_id":None}}},
                "movement_energy_cost":2,"effective_speed":2,"stamina":1,"moves_used":0,"remaining_moves":2}
        runner._validate_water_meal_intervention(record,"synthetic")
        record["original_reason"]="SearchWater"
        with self.assertRaisesRegex(ValueError,"unapproved"):
            runner._validate_water_meal_intervention(record,"synthetic")

    def test_factorial_metrics_keep_numeric_counts_and_censor_nulls(self):
        summary={"living_entity_ticks":9,"focal_ticks_from_first_zero_to_death":None,"f1_breeders":[1,2],"death_causes":{"Starvation":1},"is_flag":True}
        metrics=runner._numeric_metrics(summary)
        self.assertEqual(metrics["living_entity_ticks"],9)
        self.assertEqual(metrics["f1_breeders_count"],2)
        self.assertEqual(metrics["death_causes.Starvation"],1)
        self.assertNotIn("focal_ticks_from_first_zero_to_death",metrics)
        self.assertNotIn("is_flag",metrics)
        self.assertNotIn("final_state",runner._compact_summary({"final_state":{"entities":[1]},"living_entity_ticks":9}))

    def test_factorial_interaction_and_unknown_support_are_separate(self):
        flags={"control":(False,False),"food_reserve_only":(True,False),"water_meal_only":(False,True),"bundle":(True,True)}
        values={"control":10,"food_reserve_only":11,"water_meal_only":13,"bundle":15}
        worlds={}
        for arm,(food,water) in flags.items():
            worlds[arm]={"header":{"habitat":"meadow","seed":62501,"arm":arm,
                "candidate_policy":{"food_search_reserve_enabled":food,"water_meal_priority_enabled":water}},
                "initial":{"policy":{"food_search_reserve_enabled":food,"water_meal_priority_enabled":water}},
                "rows":[{"reserve_interventions":[],"water_meal_interventions":[]}],
                "summary":{"living_entity_ticks":values[arm],
                    "focal_ticks_from_first_zero_to_death":None if arm=="water_meal_only" else 4,
                    "births_by_generation":{"3":1} if arm=="bundle" else {}}}
        block=runner.validate_block(worlds,"meadow",62501)
        self.assertEqual(block["factorial_interaction"]["living_entity_ticks"],1)
        self.assertEqual(block["paired_deltas_vs_control"]["bundle"]["births_by_generation.3"],1)
        self.assertNotIn("focal_ticks_from_first_zero_to_death",block["factorial_interaction"])
        aggregate=runner._paired_aggregate([block])
        self.assertEqual(aggregate["living_entity_ticks"]["interaction"]["n_supported_pairs"],1)
        self.assertEqual(aggregate["focal_ticks_from_first_zero_to_death"]["interaction"]["n_supported_pairs"],0)

    def test_manifest_size_includes_final_serialized_manifest(self):
        with tempfile.TemporaryDirectory() as temp:
            out=Path(temp)
            (out/"raw.jsonl").write_text("raw\n",encoding="utf-8")
            manifest={"status":"complete","aggregate_output_bytes":0}
            size=runner._finalize_manifest(out,manifest)
            self.assertEqual(size,sum(p.stat().st_size for p in out.rglob("*") if p.is_file()))
            self.assertEqual(json.loads((out/"manifest.json").read_text())["aggregate_output_bytes"],size)


if __name__ == "__main__":
    unittest.main()
