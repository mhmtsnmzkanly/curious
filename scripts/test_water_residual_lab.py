"""Non-simulation safeguards for the bounded V17 water runner."""
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

ROOT=Path(__file__).resolve().parents[1]
SPEC=importlib.util.spec_from_file_location("run_water_residual_lab",ROOT/"scripts/run_water_residual_lab.py")
runner=importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(runner)


class WaterResidualRunnerTests(unittest.TestCase):
    def test_seed_roles_are_fixed_and_phase_scoped(self):
        self.assertEqual(runner.RUNS["calibration"],(52099,))
        self.assertEqual(runner.RUNS["formal"],tuple(range(52001,52009)))

    def test_rust_integer_division_is_exact_and_truncates_toward_zero(self):
        self.assertEqual(runner.trunc_div(7,3),2)
        self.assertEqual(runner.trunc_div(-7,3),-2)
        self.assertEqual(runner.life_baseline(1,1,{"speed":3,"max_water":60,"water":60,"move_energy_cost":1}),-100)

    def test_treatments_use_global_v17_but_naive_clears_residuals_only(self):
        checkpoint={"policy":{"water_residual_enabled":True},"entities":[
            {"id":1,"learner":{"enabled":True,"update_enabled":True,
             "values":[{"pos":{"x":1,"y":2},"value":7,"visits":2}],
             "residuals":[{"pos":{"x":1,"y":2},"sum":15,"visits":3}],
             "episode":{"target":{"x":1,"y":2},"choice_baseline":45},"last_episode":None}},
            {"id":2,"learner":{"enabled":False,"update_enabled":False,"values":[],"residuals":[]}}]}
        naive=runner.treatment_state(checkpoint,"naive_online")
        self.assertEqual(naive["policy"],checkpoint["policy"])
        self.assertEqual(naive["entities"][0]["learner"]["values"],checkpoint["entities"][0]["learner"]["values"])
        self.assertEqual(naive["entities"][0]["learner"]["episode"],checkpoint["entities"][0]["learner"]["episode"])
        self.assertNotIn("residuals",naive["entities"][0]["learner"])
        rule=runner.treatment_state(checkpoint,"rule_based")
        self.assertFalse(rule["entities"][0]["learner"]["enabled"])

    def test_aggregate_gate_uses_trained_online_and_requires_all_four_full_horizons(self):
        def branch(water=1,move=1):
            return {"water_acquired":water,"movement_energy_paid":move,"observed_survival_ticks":300,
                    "terminal_state":{"energy":10,"water":10,"health":10},"terminal_state_known":True}
        seeds=[]
        for _ in range(8):
            branches={name:branch() for name in runner.TREATMENTS}
            branches["trained_online"]=branch(2,0)
            seeds.append({"branches":branches,"training_opportunity":True})
        result=runner.aggregate_formal(seeds)
        self.assertTrue(result["bounded_utility_evidence"])
        seeds[0]["branches"]["trained_online"]=branch(0,20)
        self.assertFalse(runner.aggregate_formal(seeds)["bounded_utility_evidence"])
        seeds[0]["branches"]["trained_online"]=branch(2,0)
        seeds[0]["branches"]["naive_online"]["terminal_state_known"]=False
        self.assertFalse(runner.aggregate_formal(seeds)["bounded_utility_evidence"])
        seeds[0]["branches"]["naive_online"]["terminal_state_known"]=True
        seeds[0]["training_opportunity"]=False
        self.assertFalse(runner.aggregate_formal(seeds)["bounded_utility_evidence"])

    def test_calibration_requires_post_update_to_change_selected_source_mean(self):
        def row(old_sum,new_sum,old_count=1,new_count=2):
            position={"x":0,"y":2}
            return {"state_before":{"entities":[{"id":1,"learner":{"update_enabled":True,
                        "residuals":[{"pos":position,"sum":old_sum,"visits":old_count}]}}]},
                    "state_after":{"entities":[{"id":1,"learner":{"residuals":[{"pos":position,"sum":new_sum,"visits":new_count}]}}]},
                    "learner_closures":{"postresolver_closures":[{"end":"Reached","attempt":{
                        "target":position,"choice_baseline":4,"outcome_overflowed":False,"candidate_censored":False}}]}}
        self.assertTrue(runner.completed_update_changes_selected_mean([row(0,2)]))
        self.assertFalse(runner.completed_update_changes_selected_mean([row(0,0)]))

    def test_validator_counts_predecision_and_postresolver_closures_in_one_tick(self):
        old={"target":{"x":4,"y":4},"ticks":3,"movement_energy":2,"incoming_damage":0,"choice_baseline":5}
        new={"target":{"x":0,"y":2},"ticks":0,"movement_energy":0,"incoming_damage":0,"choice_baseline":1900}
        interrupted={"attempt":old,"water":0,"end":"Interrupted","reward":None}
        reached={"attempt":new,"water":10,"end":"Reached","reward":90}
        before={"entities":[{"id":1,"learner":{"enabled":True,"update_enabled":True,
                 "episode":old,"last_episode":None,"values":[],"residuals":[]}}]}
        after={"entities":[{"id":1,"learner":{"enabled":True,"update_enabled":True,
                "episode":None,"last_episode":reached,"values":[],"residuals":[]}}]}
        trace={"pos":{"x":0,"y":2},"perception":{"waters":[{"steps":[]},{"steps":["Right"]}]}}
        row={"state_before":before,"state_after":after,"focal_choice":{"base_reason":"SeekWater",
             "selected_target":{"x":0,"y":2},"candidates":[{"pos":{"x":0,"y":2},"current_baseline":1900}]},
             "learner_closures":{"predecision_closures":[interrupted],"episode_after_choice":new,
             "postresolver_closures":[reached],"replayed_decision_matches_trace":True,
             "replayed_postdecision_rng_matches_world":True,"replayed_learner_matches_world":True}}
        runner.validate_learner_closures(row,trace,"synthetic-double-closure",1)
        counts,total=runner.censor_counts([row])
        self.assertEqual(total,2)
        self.assertEqual(counts,{"Interrupted":1,"Reached":1})

    def test_raw_reader_is_streaming_and_fails_on_truncation_or_extra_rows(self):
        with tempfile.TemporaryDirectory() as tmp:
            raw=Path(tmp)/"raw.jsonl"
            raw.write_text(json.dumps({"seed":52099,"role":"calibration"})+"\n",encoding="utf-8")
            rows=runner.parse_raw(raw,"calibration",(52099,))
            self.assertEqual(next(rows)["seed"],52099)
            with self.assertRaises(StopIteration): next(rows)
            raw.write_text('{"seed":52099}',encoding="utf-8")
            with self.assertRaisesRegex(ValueError,"truncated"):
                next(runner.parse_raw(raw,"calibration",(52099,)))

    def test_formal_preflight_rejects_corrupt_calibration_evidence_before_output(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp)
            archive=root/"archive"; cal=archive/runner.OUTPUTS["calibration"]
            cal.mkdir(parents=True)
            output=archive/runner.OUTPUTS["formal"]
            binary=root/"candidate"; binary.write_bytes(b"candidate")
            retained=cal/"water-residual-probe"; retained.write_bytes(b"candidate")
            protocol=cal/"protocol.md"; protocol.write_bytes((ROOT/runner.PROTOCOL).read_bytes())
            scanner=cal/"seed-scanner.py"; scanner.write_bytes((ROOT/runner.SCANNER).read_bytes())
            raw=cal/"raw.jsonl"; raw.write_bytes(b"not-valid-json\n")
            report=cal/"report.md"; report.write_text("report",encoding="utf-8")
            summary=cal/"summary.json"
            summary_data={"schema":runner.SUMMARY_SCHEMA,"protocol_version":1,"role":"calibration",
                          "status":"completed","calibration_validity":{"passed":True}}
            summary.write_text(json.dumps(summary_data),encoding="utf-8")
            hashes={"test-source":"bound"}
            current={"revision":"rev","source_files_sha256":hashes,
                     "source_sha256":runner.canonical_sha(hashes),"binary_sha256":runner.sha(binary),
                     "protocol_sha256":runner.sha(ROOT/runner.PROTOCOL),"runner_sha256":runner.sha(ROOT/"scripts/run_water_residual_lab.py"),
                     "scanner_sha256":runner.sha(ROOT/runner.SCANNER)}
            def write_manifest():
                manifest={"schema":runner.MANIFEST_SCHEMA,"protocol_version":1,"role":"calibration",
                          "seeds":[52099],"status":"completed","analysis_status":"completed",
                          **current,"raw_sha256":runner.sha(raw),"summary_sha256":runner.sha(summary),
                          "binary_sha256":runner.sha(retained),"report_sha256":runner.sha(report)}
                (cal/"manifest.json").write_text(json.dumps(manifest),encoding="utf-8")
            write_manifest()
            raw.write_bytes(b"tampered raw\n")
            with patch.object(runner,"ARCHIVE",archive), patch.object(runner,"source_identity",return_value=("rev",hashes)), \
                 patch.object(runner,"scan_seeds",return_value={"unseen":list(runner.RUNS["formal"]),"matches":{}}):
                with self.assertRaisesRegex(RuntimeError,"raw hash"):
                    runner.execute_phase("formal",binary,archive,ROOT)
                self.assertFalse(output.exists())
                raw.write_bytes(b"not-valid-json\n")
                summary.write_text("{\"corrupt\":true}",encoding="utf-8")
                with self.assertRaisesRegex(RuntimeError,"summary hash"):
                    runner.execute_phase("formal",binary,archive,ROOT)
                self.assertFalse(output.exists())
                summary_data["calibration_validity"]["passed"]=False
                summary.write_text(json.dumps(summary_data),encoding="utf-8")
                write_manifest()
                with self.assertRaisesRegex(RuntimeError,"passing fixture gate"):
                    runner.execute_phase("formal",binary,archive,ROOT)
                self.assertFalse(output.exists())


if __name__=="__main__":
    unittest.main()
