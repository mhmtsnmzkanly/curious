import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from analyze_water_residual_tradeoffs import (
    best_candidate,
    closure_censor_reasons,
    choice_diagnostics,
    focal_event_totals,
    pareto_relation,
    realized_residual_sample,
    trunc_div,
)


class WaterResidualTradeoffTests(unittest.TestCase):
    def test_candidate_tie_keeps_first_perceived_source(self):
        candidates = [
            {"pos": {"x": 0, "y": 2}, "final_score": 100, "steps_len": 2},
            {"pos": {"x": 4, "y": 2}, "final_score": 100, "steps_len": 2},
        ]
        self.assertIs(best_candidate(candidates, "final_score"), candidates[0])

    def test_candidate_score_tie_prefers_shorter_route(self):
        candidates = [
            {"pos": {"x": 0, "y": 2}, "final_score": 100, "steps_len": 3},
            {"pos": {"x": 4, "y": 2}, "final_score": 100, "steps_len": 1},
        ]
        self.assertIs(best_candidate(candidates, "final_score"), candidates[1])

    def test_local_rank_counterfactual_strips_only_recorded_residual(self):
        tick = {
            "decisions": [{"id": 1, "before": {"water": 20, "energy": 30, "health": 100}}],
            "focal_choice": {
                "fresh": True,
                "selection_kind": "greedy",
                "selected_target": {"x": 4, "y": 2},
                "candidates": [
                    {"pos": {"x": 0, "y": 2}, "current_baseline": 100, "residual_mean": 0,
                     "residual_count": 2, "final_score": 100, "amount": 12, "steps_len": 3},
                    {"pos": {"x": 4, "y": 2}, "current_baseline": 90, "residual_mean": 20,
                     "residual_count": 1, "final_score": 110, "amount": 8, "steps_len": 1},
                ],
            },
        }
        result = choice_diagnostics(tick, "trained_online")
        self.assertTrue(result["residual_overrode_baseline"])
        self.assertEqual(result["baseline_only_target"], "0,2")
        self.assertEqual(result["current_greedy_target"], "4,2")
        self.assertEqual(result["body_water"], 20)

    def test_fresh_greedy_target_must_match_recomputed_final_rank(self):
        tick = {
            "tick": 17,
            "focal_choice": {
                "fresh": True,
                "selection_kind": "greedy",
                "selected_target": {"x": 0, "y": 2},
                "candidates": [
                    {"pos": {"x": 0, "y": 2}, "current_baseline": 100, "residual_mean": 0,
                     "residual_count": 0, "final_score": 100, "amount": 8, "steps_len": 2},
                    {"pos": {"x": 4, "y": 2}, "current_baseline": 90, "residual_mean": 20,
                     "residual_count": 1, "final_score": 110, "amount": 8, "steps_len": 1},
                ],
            },
        }
        with self.assertRaisesRegex(ValueError, "does not match"):
            choice_diagnostics(tick, "trained_online")

    def test_exploration_target_is_not_checked_against_greedy_rank(self):
        tick = {
            "focal_choice": {
                "fresh": True,
                "selection_kind": "explore",
                "selected_target": {"x": 0, "y": 2},
                "candidates": [
                    {"pos": {"x": 0, "y": 2}, "current_baseline": 100, "residual_mean": 0,
                     "residual_count": 0, "final_score": 100, "amount": 8, "steps_len": 2},
                    {"pos": {"x": 4, "y": 2}, "current_baseline": 90, "residual_mean": 20,
                     "residual_count": 1, "final_score": 110, "amount": 8, "steps_len": 1},
                ],
            },
        }
        self.assertEqual(choice_diagnostics(tick, "trained_online")["selected_target"], "0,2")

    def test_resolver_totals_include_only_focal_events(self):
        tick = {"events": [
            {"EntityDrank": {"id": 1, "amount": 7, "pos": {"x": 0, "y": 2}}},
            {"EntityMoved": {"id": 1, "energy_cost": 3}},
            {"EntityDrank": {"id": 2, "amount": 10, "pos": {"x": 4, "y": 2}}},
        ]}
        self.assertEqual(focal_event_totals(tick), {"water": 7, "movement_energy": 3, "drinks": 1, "moves": 1})

    def test_residual_reconstruction_matches_rust_truncation_toward_zero(self):
        self.assertEqual(trunc_div(-5, 2), -2)
        closure = {
            "end": "Reached",
            "water": 8,
            "attempt": {"choice_baseline": 1100, "movement_energy": 3, "ticks": 1},
        }
        self.assertEqual(realized_residual_sample(closure), 100)
        closure["end"] = "TimedOut"
        self.assertIsNone(realized_residual_sample(closure))

    def test_reached_outcome_with_censor_flags_is_not_a_residual_sample(self):
        closure = {
            "end": "Reached",
            "water": 10,
            "attempt": {"choice_baseline": 1700, "movement_energy": 2, "ticks": 1,
                        "outcome_overflowed": True},
        }
        self.assertIsNone(realized_residual_sample(closure))
        self.assertEqual(closure_censor_reasons(closure), ["outcome_overflowed"])
        closure["attempt"]["outcome_overflowed"] = False
        closure["attempt"]["candidate_censored"] = True
        self.assertIsNone(realized_residual_sample(closure))
        self.assertEqual(closure_censor_reasons(closure), ["candidate_censored"])

    def test_completed_failed_zero_water_episode_is_not_censored(self):
        closure = {"end": "Failed", "water": 0, "attempt": {"choice_baseline": 100}}
        self.assertEqual(closure_censor_reasons(closure), [])

    def test_pareto_relation_preserves_tradeoffs(self):
        baseline = {
            "water_acquired": 10, "movement_energy_paid": 10, "terminal_energy": 20,
            "terminal_water": 20, "terminal_health": 100, "observed_survival_ticks": 300,
        }
        less_water_less_cost = {**baseline, "water_acquired": 9, "movement_energy_paid": 8}
        self.assertEqual(pareto_relation(less_water_less_cost, baseline), "tradeoff")
        better = {**baseline, "water_acquired": 11}
        self.assertEqual(pareto_relation(better, baseline), "candidate_dominates")
        self.assertEqual(pareto_relation(baseline, baseline), "identical")


if __name__ == "__main__":
    unittest.main()
