#!/usr/bin/env python3
"""Fixture-only tests for the pinned refuge first-resolution extractor."""
from __future__ import annotations

import copy
import unittest

from refuge_first_resolution import AuditError, audit_records


SCENARIO_SHA = "9641a7a60157edcd9b3d379210b26aead5ed57df56ec242dca5c604a74d478f2"


def start(owner: int, tick: int, threat: int) -> dict:
    route = {
        "endpoint": {"x": owner, "y": 2},
        "endpoint_kind": "burrow",
        "path": ["Up", "UpLeft"],
        "path_steps": 2,
        "turns": 1,
        "observed_route_cells": ["empty", "burrow"],
        "selected_threat": {
            "id": threat,
            "species": "Carnivore",
            "observed_health": 40,
            "observed_power": 60,
            "observed_steps": ["Down"],
            "static_contact_geometry": {"minimum_current_clearance": 1},
        },
        "candidate_secret": "must-not-become-a-feature",
    }
    return {
        "record_type": "episode_start",
        "owner_id": owner,
        "threat_id": threat,
        "started_tick": tick,
        "start_position": {"x": 0, "y": 0},
        "body": {
            "id": owner,
            "health": 90,
            "max_health": 100,
            "energy": 40,
            "max_energy": 50,
            "stamina": 8,
            "max_stamina": 10,
            "speed": 2,
            "move_energy_cost": 1,
            "vision_range": 5,
        },
        "decision_reason": "SeekShelter",
        "selected_intent": {"Move": {"steps": ["Up"]}},
        "selected_route_index": 0,
        "routes": [route, {"candidate_secret": "never serialize"}],
    }


def interval(owner: int, threat: int, tick: int, attacks: list[dict], elapsed: int = 1) -> dict:
    selected = sum(e["damage"] for e in attacks if e["attacker"] == threat)
    other = sum(e["damage"] for e in attacks if e["attacker"] != threat)
    return {
        "record_type": "episode_tick",
        "owner_id": owner,
        "threat_id": threat,
        "tick": tick,
        "elapsed_ticks": elapsed,
        "attack_events": [
            {"attacker_id": event["attacker"], "target_id": event["target"], "damage": event["damage"]}
            for event in attacks
        ],
        "cumulative_threat_damage": selected,
        "cumulative_other_damage": other,
        "cumulative_unattributed_health_loss": 3,
        "cumulative_paid_movement_energy": 1,
        "health_before_resolution": 90,
        "health_after_resolution": 90 - selected - other - 3,
        "movement_events": [],
        "position_before_resolution": {"x": 0, "y": 0},
        "position_after_resolution": {"x": 0, "y": 1},
    }


def closure_for(s: dict, end: int, closure: str = "arrived") -> dict:
    copied_start = {key: value for key, value in s.items() if key != "record_type"}
    return {
        "record_type": "episode_closure",
        "start": copied_start,
        "ended_tick": end,
        "closure": closure,
        "completed_arrival": closure == "arrived",
        "threat_damage": 0,
        "other_damage": 0,
        "unattributed_health_loss": 0,
    }


def fixture_records() -> list[dict]:
    s1 = start(11, 1, 71)
    s2 = start(12, 2, 72)
    global1 = [{"kind": "moved", "event_index": 0, "id": 11}]
    attacks2 = [
        {"kind": "attacked", "event_index": 0, "attacker": 71, "target": 11, "damage": 8},
        {"kind": "attacked", "event_index": 2, "attacker": 72, "target": 12, "damage": 7},
        {"kind": "moved", "event_index": 4, "id": 12},
    ]
    owner11_attack = [attacks2[0]]
    owner12_attack = [attacks2[1]]
    return [
        {"record_type": "header", "protocol": "refuge_risk_baseline_v1_engineering", "scenario_sha256": SCENARIO_SHA, "seed": 123, "physical_tick_limit": 1000, "refuge_feature_flags_changed": False},
        {"record_type": "tick", "tick": 1, "resolver_events": 1},
        {"record_type": "resolver_events", "tick": 1, "events": global1},
        {"record_type": "tick", "tick": 2, "resolver_events": 5},
        {"record_type": "resolver_events", "tick": 2, "events": attacks2},
        s1,
        s2,
        interval(11, 71, 1, []),
        interval(11, 71, 2, owner11_attack, elapsed=2),
        interval(12, 72, 2, owner12_attack),
        closure_for(s1, 2, "priority_interrupted"),
        closure_for(s2, 2, "arrived"),
        {"record_type": "summary", "started_attempts": 2, "closed_attempts": 2, "completed_world_ticks": 2},
    ]


class FirstResolutionTests(unittest.TestCase):
    def test_exact_first_interval_and_excludes_later_continuation(self) -> None:
        rows, summary = audit_records(fixture_records(), "fixture-hash")
        self.assertEqual(len(rows), 2)
        self.assertEqual(summary["later_continuation_rows_excluded"], 1)
        self.assertEqual([row["outcome"]["selected_threat_damage"] for row in rows], [0, 7])
        self.assertEqual(summary["damage_support"], {"0": 1, "7": 1})
        self.assertEqual(rows[0]["outcome"]["exposure_resolver_intervals"], 1)

    def test_features_are_choice_time_selected_route_only(self) -> None:
        rows, _ = audit_records(fixture_records(), "fixture-hash")
        row = rows[0]
        self.assertNotIn("owner_id", row["features"])
        self.assertNotIn("threat_id", row["features"])
        self.assertNotIn("id", row["features"]["body"])
        self.assertNotIn("candidate_secret", str(row["features"]))
        self.assertNotIn("closure", row["features"])
        self.assertIn("owner_id", row["provenance"])
        self.assertEqual(row["provenance"]["closure"], "priority_interrupted")

    def test_missing_first_interval_is_unknown_not_zero(self) -> None:
        records = fixture_records()
        records = [r for r in records if not (r.get("record_type") == "episode_tick" and r.get("owner_id") == 11 and r.get("tick") == 1)]
        rows, summary = audit_records(records, "fixture-hash")
        first = next(row for row in rows if row["provenance"]["owner_id"] == 11)
        self.assertEqual(first["outcome"]["status"], "Unknown")
        self.assertIsNone(first["outcome"]["selected_threat_damage"])
        self.assertEqual(first["outcome"]["reason"], "missing_first_interval")
        self.assertEqual(summary["unknown_rows"], 1)
        self.assertEqual(summary["per_owner_counts_metadata_only"]["11"]["exact_first_intervals"], 0)

    def test_missing_global_witness_is_unknown_not_zero(self) -> None:
        records = [r for r in fixture_records() if not (r.get("record_type") == "resolver_events" and r.get("tick") == 1)]
        rows, _ = audit_records(records, "fixture-hash")
        first = next(row for row in rows if row["provenance"]["owner_id"] == 11)
        self.assertEqual(first["outcome"]["status"], "Unknown")
        self.assertIsNone(first["outcome"]["selected_threat_damage"])

    def test_selected_other_and_unattributed_damage_stay_separate(self) -> None:
        records = fixture_records()
        s = start(13, 3, 73)
        own = {"kind": "attacked", "event_index": 0, "attacker": 73, "target": 13, "damage": 6}
        other = {"kind": "attacked", "event_index": 2, "attacker": 90, "target": 13, "damage": 4}
        records.extend([
            {"record_type": "tick", "tick": 3, "resolver_events": 3},
            {"record_type": "resolver_events", "tick": 3, "events": [own, other]},
            s,
            interval(13, 73, 3, [own, other]),
            closure_for(s, 3, "died"),
        ])
        summary = next(r for r in records if r.get("record_type") == "summary")
        summary["started_attempts"] = 3
        summary["closed_attempts"] = 3
        summary["completed_world_ticks"] = 3
        rows, _ = audit_records(records, "fixture-hash")
        row = next(row for row in rows if row["provenance"]["owner_id"] == 13)
        self.assertEqual(row["outcome"]["status"], "Exact")
        self.assertEqual(row["outcome"]["selected_threat_damage"], 6)
        self.assertEqual(row["outcome"]["other_damage"], 4)
        self.assertEqual(row["outcome"]["unattributed_health_loss"], 3)

    def test_duplicate_global_witness_is_rejected(self) -> None:
        records = fixture_records()
        event_record = next(r for r in records if r.get("record_type") == "resolver_events" and r.get("tick") == 1)
        records.append(copy.deepcopy(event_record))
        with self.assertRaises(AuditError):
            audit_records(records, "fixture-hash")

    def test_duplicate_global_event_index_is_rejected(self) -> None:
        records = fixture_records()
        event_record = next(r for r in records if r.get("record_type") == "resolver_events" and r.get("tick") == 2)
        event_record["events"].append(copy.deepcopy(event_record["events"][0]))
        with self.assertRaises(AuditError):
            audit_records(records, "fixture-hash")

    def test_same_owner_start_on_prior_closure_tick_has_distinct_join(self) -> None:
        first = start(11, 1, 71)
        second = start(11, 2, 71)
        no_attack = [{"kind": "moved", "event_index": 0, "id": 11}]
        records = [
            {"record_type": "header", "protocol": "refuge_risk_baseline_v1_engineering", "scenario_sha256": SCENARIO_SHA, "seed": 123, "physical_tick_limit": 1000, "refuge_feature_flags_changed": False},
            {"record_type": "tick", "tick": 1, "resolver_events": 1},
            {"record_type": "resolver_events", "tick": 1, "events": no_attack},
            {"record_type": "tick", "tick": 2, "resolver_events": 1},
            {"record_type": "resolver_events", "tick": 2, "events": no_attack},
            first,
            second,
            interval(11, 71, 1, []),
            interval(11, 71, 2, [], elapsed=1),
            closure_for(first, 2, "target_changed"),
            closure_for(second, 2, "arrived"),
            {"record_type": "summary", "started_attempts": 2, "closed_attempts": 2, "completed_world_ticks": 2},
        ]
        rows, summary = audit_records(records, "fixture-hash")
        self.assertEqual(len(rows), 2)
        self.assertEqual([r["provenance"]["start_tick"] for r in rows], [1, 2])
        self.assertEqual([r["provenance"]["resolver_tick"] for r in rows], [1, 2])
        self.assertEqual(summary["later_continuation_rows_excluded"], 0)

    def test_duplicate_owner_tick_interval_is_rejected_even_with_other_elapsed(self) -> None:
        records = fixture_records()
        row = next(r for r in records if r.get("record_type") == "episode_tick" and r.get("owner_id") == 11 and r.get("tick") == 1)
        duplicate = copy.deepcopy(row)
        duplicate["elapsed_ticks"] = 2
        duplicate["tick"] = 2
        records.append(duplicate)
        with self.assertRaises(AuditError):
            audit_records(records, "fixture-hash")

    def test_start_closure_join_normalizes_record_tag_only(self) -> None:
        rows, summary = audit_records(fixture_records(), "fixture-hash")
        self.assertEqual(summary["starts"], summary["closures"])
        self.assertTrue(all(row["outcome"]["status"] == "Exact" for row in rows))

    def test_start_selected_threat_mismatch_is_unknown(self) -> None:
        records = fixture_records()
        s = next(r for r in records if r.get("record_type") == "episode_start" and r.get("owner_id") == 11)
        s["routes"][0]["selected_threat"]["id"] = 999
        closure = next(r for r in records if r.get("record_type") == "episode_closure" and r["start"].get("owner_id") == 11)
        closure["start"]["routes"][0]["selected_threat"]["id"] = 999
        rows, _ = audit_records(records, "fixture-hash")
        row = next(r for r in rows if r["provenance"]["owner_id"] == 11)
        self.assertEqual(row["outcome"]["status"], "Unknown")
        self.assertIsNone(row["outcome"]["selected_threat_damage"])

    def test_closure_and_later_attack_changes_do_not_leak_into_first_target(self) -> None:
        baseline_rows, _ = audit_records(fixture_records(), "fixture-hash")
        changed_records = fixture_records()
        closure = next(r for r in changed_records if r.get("record_type") == "episode_closure" and r["start"].get("owner_id") == 11)
        closure["closure"] = "died"
        closure["completed_arrival"] = False
        closure["final_health"] = 0
        global_tick2 = next(r for r in changed_records if r.get("record_type") == "resolver_events" and r.get("tick") == 2)
        continuation_event = next(e for e in global_tick2["events"] if e.get("target") == 11)
        continuation_event["damage"] = 19
        continuation = next(r for r in changed_records if r.get("record_type") == "episode_tick" and r.get("owner_id") == 11 and r.get("tick") == 2)
        continuation["attack_events"][0]["damage"] = 19
        continuation["cumulative_threat_damage"] = 19
        continuation["health_after_resolution"] = 68
        changed_rows, _ = audit_records(changed_records, "fixture-hash")
        baseline = next(r for r in baseline_rows if r["provenance"]["owner_id"] == 11)
        changed = next(r for r in changed_rows if r["provenance"]["owner_id"] == 11)
        self.assertEqual(changed["features"], baseline["features"])
        self.assertEqual(changed["outcome"], baseline["outcome"])
        self.assertNotEqual(changed["provenance"]["closure"], baseline["provenance"]["closure"])

    def test_unknown_positive_row_is_excluded_from_exact_owner_support(self) -> None:
        records = fixture_records()
        records = [r for r in records if not (r.get("record_type") == "episode_tick" and r.get("owner_id") == 12 and r.get("tick") == 2)]
        rows, summary = audit_records(records, "fixture-hash")
        owner_row = next(r for r in rows if r["provenance"]["owner_id"] == 12)
        self.assertEqual(owner_row["outcome"]["status"], "Unknown")
        self.assertEqual(summary["per_owner_counts_metadata_only"]["12"]["episode_starts"], 1)
        self.assertEqual(summary["per_owner_counts_metadata_only"]["12"]["exact_first_intervals"], 0)
        self.assertEqual(summary["per_owner_counts_metadata_only"]["12"]["positive_first_intervals"], 0)
        self.assertEqual(summary["damage_support"], {"0": 1})

    def test_missing_features_do_not_change_known_exact_target(self) -> None:
        records = fixture_records()
        s = next(r for r in records if r.get("record_type") == "episode_start" and r.get("owner_id") == 11)
        del s["body"]["max_health"]
        closure = next(r for r in records if r.get("record_type") == "episode_closure" and r["start"].get("owner_id") == 11)
        closure["start"] = {key: value for key, value in s.items() if key != "record_type"}
        rows, _ = audit_records(records, "fixture-hash")
        row = next(r for r in rows if r["provenance"]["owner_id"] == 11)
        self.assertEqual(row["feature_status"], "unknown")
        self.assertEqual(row["outcome"]["status"], "Exact")
        self.assertEqual(row["outcome"]["selected_threat_damage"], 0)


if __name__ == "__main__":
    unittest.main()
