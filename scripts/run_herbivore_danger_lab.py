#!/usr/bin/env python3
"""Run the preregistered, bounded herbivore danger/refuge lab."""
from __future__ import annotations

import argparse
from collections import Counter, defaultdict
from datetime import datetime, timezone
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
PROTOCOL = ROOT / "docs/HERBIVORE_DANGER_PROTOCOL.md"
HARNESS = ROOT / "examples/herbivore_danger_lab.rs"
SCHEMA = "curious-herbivore-lab-run-v1"
RAW_SCHEMA = "curious-herbivore-danger-lab-v1"
SEEDS = list(range(45001, 45009))
SUBJECT_ID = 1
PREDATOR_ID = 2
MAX_TICKS = 1600
MAX_SECONDS = 300
MAX_OUTPUT_BYTES = 1024**3
MAX_CHILD_MEMORY_BYTES = 1024**3
MAX_RAW_LINE_BYTES = 64 * 1024**2
REGISTERED_OUTPUT = Path("/root/curious/data/experiments/herbivore-lab-v1-20261023")
FIXED_SOURCE_FILES = [
    "Cargo.toml", "Cargo.lock", "examples/herbivore_danger_lab.rs",
    "examples/support/refuge_episode_audit.rs",
    "docs/HERBIVORE_DANGER_PROTOCOL.md", "scripts/run_herbivore_danger_lab.py",
    "scripts/run_ecology_traits_lab.py",
]


def source_files() -> list[str]:
    tracked = subprocess.check_output(["git", "ls-files", "src"], cwd=ROOT, text=True).splitlines()
    rust_sources = [name for name in tracked if name.endswith(".rs")]
    return sorted(set(FIXED_SOURCE_FILES + rust_sources))


def sha(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            h.update(block)
    return h.hexdigest()


def source_hashes() -> dict[str, str]:
    return {name: sha(ROOT / name) for name in source_files()}


def declared_seed_audit(registry: Path) -> dict:
    """Use the maintained declared-seed scanner; numeric coincidence is not reuse."""
    helper_path = ROOT / "scripts/run_ecology_traits_lab.py"
    spec = importlib.util.spec_from_file_location("ecology_traits_seed_helper", helper_path)
    if spec is None or spec.loader is None:
        raise RuntimeError("cannot load maintained seed registry helper")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module.audit_seeds(registry, SEEDS)


def actor(state: dict, actor_id: int) -> dict | None:
    entities = state.get("entities") if isinstance(state, dict) else None
    if not isinstance(entities, list):
        return None
    return next((entity for entity in entities if entity.get("id") == actor_id), None)


def position(value):
    if not isinstance(value, dict) or type(value.get("x")) is not int or type(value.get("y")) is not int:
        raise ValueError("position must contain integer x/y")
    return {"x": value["x"], "y": value["y"]}


def event_items(value):
    if not isinstance(value, list):
        raise ValueError("timeline events must be a list")
    for event in value:
        if not isinstance(event, dict) or len(event) != 1:
            raise ValueError("simulation events must be externally tagged single-key objects")
        kind, payload = next(iter(event.items()))
        if not isinstance(payload, dict):
            raise ValueError(f"invalid {kind} payload")
        yield kind, payload


def field_num(value, field, label):
    number = value.get(field)
    if type(number) is not int or number < 0:
        raise ValueError(f"{label}: {field} must be a nonnegative integer")
    return number


def life_snapshot(state):
    subject = actor(state, SUBJECT_ID)
    if subject is None or not isinstance(subject.get("life"), dict):
        raise ValueError("subject state/life missing")
    life = subject["life"]
    return {
        "position": position(subject.get("pos")),
        "health": life.get("health"), "max_health": life.get("max_health"),
        "energy": life.get("energy"), "max_energy": life.get("max_energy"),
        "stamina": life.get("stamina"), "max_stamina": life.get("max_stamina"),
        "age": life.get("age"), "generation": subject.get("generation"),
        "species": subject.get("species"),
        "memory": subject.get("memory"),
    }


def validate_initial_pair(row):
    baseline = row.get("baseline", {}).get("initial_state")
    learned = row.get("refuge_memory", {}).get("initial_state")
    if not isinstance(baseline, dict) or not isinstance(learned, dict):
        raise ValueError("missing paired branch initial state")
    if row.get("paired_initial_state_equal_except_feature") is not True:
        raise ValueError("harness initial-state pair audit failed")
    altered = json.loads(json.dumps(learned))
    subject = actor(altered, SUBJECT_ID)
    if subject is None or not isinstance(subject.get("memory"), dict):
        raise ValueError("paired subject memory missing")
    subject["memory"]["refuge_outcomes_enabled"] = False
    if altered != baseline:
        raise ValueError("paired branch state differs beyond refuge feature flag")
    for state, wanted in ((baseline, False), (learned, True)):
        subject = actor(state, SUBJECT_ID)
        if subject.get("memory", {}).get("refuge_outcomes_enabled") is not wanted:
            raise ValueError("branch initial feature flag mismatch")
        if subject["species"] != "Herbivore" or subject["pos"] != {"x": 3, "y": 3}:
            raise ValueError("subject did not start at registered species/position")
    if baseline.get("tick") != 0 or learned.get("tick") != 0:
        raise ValueError("branch must start at tick zero")


def validate_checkpoint(branch, label):
    cp = branch.get("checkpoint_roundtrip")
    rows = branch["timeline"]
    tick_50 = next((r for r in rows if r.get("tick") == 50), None)
    if tick_50 is None:
        if cp is not None:
            raise ValueError(f"{label}: unexpected tick-50 roundtrip")
        return
    if not isinstance(cp, dict) or cp.get("tick") != 50:
        raise ValueError(f"{label}: missing tick-50 in-place roundtrip")
    if cp.get("state_equal") is not True or cp.get("serialized_state") != cp.get("restored_state"):
        raise ValueError(f"{label}: complete checkpoint state did not roundtrip")
    if cp.get("rng_before") != cp.get("rng_after"):
        raise ValueError(f"{label}: RNG changed during checkpoint roundtrip")
    if cp.get("patrol_index_before") != cp.get("patrol_index_after"):
        raise ValueError(f"{label}: external patrol cursor changed during checkpoint roundtrip")
    if cp["serialized_state"] != tick_50.get("state"):
        raise ValueError(f"{label}: roundtrip did not preserve the tick-50 world")


def subject_trace(row):
    decisions = row.get("decisions")
    if isinstance(decisions, list):
        return next((trace for trace in decisions if isinstance(trace, dict) and trace.get("id") == SUBJECT_ID), None)
    if isinstance(decisions, dict):
        return decisions if decisions.get("id") == SUBJECT_ID else None
    return None


AUDIT_PREDECISION_CLOSURES = {"threat_lost", "target_changed", "priority_interrupted", "options_invalid", "feature_disabled"}
DIRS = {"Up": (0, -1), "UpRight": (1, -1), "Right": (1, 0), "DownRight": (1, 1),
        "Down": (0, 1), "DownLeft": (-1, 1), "Left": (-1, 0), "UpLeft": (-1, -1)}


def closure_row_at(rows, tick):
    return rows[tick - 1] if 1 <= tick <= len(rows) else None


def visible_context(value):
    if not isinstance(value, dict):
        return None
    fields = ("threat_id", "threat_species", "bearing", "threat_distance",
              "energy_quartile", "stamina_quartile", "refuge_options")
    return {name: value.get(name) for name in fields}


def decode_steps(value):
    if isinstance(value, dict) and "0" in value:
        value = value["0"]
    if not isinstance(value, list) or any(direction not in DIRS for direction in value):
        raise ValueError("perceived/selected paths must be serialized Direction lists")
    return value


def walk_path(origin, steps):
    x, y = origin["x"], origin["y"]
    for step in steps:
        dx, dy = DIRS[step]
        x, y = x + dx, y + dy
    return {"x": x, "y": y}


def visible_refuges(trace):
    perception = trace.get("perception") or {}
    origin = position(trace.get("pos"))
    occupied = set()
    for entity in perception.get("entities", []):
        occupied.add((walk_path(origin, decode_steps(entity.get("steps")))["x"],
                      walk_path(origin, decode_steps(entity.get("steps")))["y"]))
    paths = []
    for encoded in perception.get("shelters", []):
        steps = decode_steps(encoded)
        if not steps:
            continue
        endpoint = walk_path(origin, steps)
        if (endpoint["x"], endpoint["y"]) not in occupied:
            paths.append((endpoint, steps))
    paths.sort(key=lambda pair: (pair[0]["y"], pair[0]["x"]))
    return paths


def validate_memory(memory, tick, label):
    if not isinstance(memory, dict):
        raise ValueError(f"{label}: memory snapshot missing")
    enabled = memory.get("refuge_outcomes_enabled")
    outcomes = memory.get("refuge_outcomes")
    pending = memory.get("pending_refuge_closures")
    episode = memory.get("refuge_episode")
    if type(enabled) is not bool or not isinstance(outcomes, list) or len(outcomes) > 8:
        raise ValueError(f"{label}: refuge memory feature flag/record cap invalid")
    if not isinstance(pending, list) or len(pending) > 1:
        raise ValueError(f"{label}: pending refuge closure cap exceeded")
    keys = set()
    def validate_key(key, item_label):
        if not isinstance(key, dict):
            raise ValueError(f"{item_label}: refuge key missing")
        if type(key.get("threat_id")) is not int or key.get("threat_species") not in {"Carnivore", "Omnivore"}:
            raise ValueError(f"{item_label}: invalid threat identity")
        if type(key.get("bearing")) is not int or key["bearing"] not in range(8):
            raise ValueError(f"{item_label}: invalid bearing bucket")
        if key.get("threat_distance") not in {0, 1, 2} or key.get("energy_quartile") not in {0, 1, 2, 3} or key.get("stamina_quartile") not in {0, 1, 2, 3}:
            raise ValueError(f"{item_label}: invalid threat/resource bucket")
        options = key.get("refuge_options")
        if not isinstance(options, list) or not options or len(options) > 8:
            raise ValueError(f"{item_label}: invalid refuge alternative set")
        normalized = [position(p) for p in options]
        if normalized != sorted(normalized, key=lambda p: (p["y"], p["x"])):
            raise ValueError(f"{item_label}: refuge endpoints are not stably sorted")
        endpoint = position(key.get("endpoint"))
        if endpoint not in normalized:
            raise ValueError(f"{item_label}: selected endpoint is outside the alternatives")
    for outcome in outcomes:
        if not isinstance(outcome, dict) or not isinstance(outcome.get("key"), dict):
            raise ValueError(f"{label}: malformed refuge outcome row")
        key = outcome["key"]
        validate_key(key, label)
        completed = field_num(outcome, "completed", label)
        safe = field_num(outcome, "safe_arrivals", label)
        if completed < 1 or completed > 1_000_000 or safe > completed:
            raise ValueError(f"{label}: invalid completed/safe-arrival counter")
        if field_num(outcome, "threat_damage", label) > 1_000_000_000_000 or field_num(outcome, "movement_energy", label) > 1_000_000_000_000:
            raise ValueError(f"{label}: refuge outcome aggregate exceeds counter cap")
        last_tick = field_num(outcome, "last_observed_tick", label)
        if last_tick > tick:
            raise ValueError(f"{label}: refuge observation is from a future tick")
        if tick - last_tick > 100:
            raise ValueError(f"{label}: expired refuge observation remained in memory")
        key_tuple = json.dumps(key, sort_keys=True)
        if key_tuple in keys:
            raise ValueError(f"{label}: duplicate refuge outcome key")
        keys.add(key_tuple)
    def validate_active_episode(episode, episode_label, ended_at=None):
        if not isinstance(episode, dict):
            raise ValueError(f"{episode_label}: refuge episode must be an object")
        validate_key(episode.get("key"), episode_label)
        if episode.get("threat_id") != episode["key"].get("threat_id") or episode.get("threat_species") != episode["key"].get("threat_species"):
            raise ValueError(f"{episode_label}: episode key/identity mismatch")
        if position(episode.get("target")) != position(episode["key"].get("endpoint")):
            raise ValueError(f"{episode_label}: episode target/key mismatch")
        for field in ("movement_energy", "threat_damage", "other_damage", "unattributed_health_loss"):
            if field_num(episode, field, label) > 1_000_000_000_000:
                raise ValueError(f"{episode_label}: episode counter cap exceeded")
        if field_num(episode, "elapsed_ticks", label) > 20 or field_num(episode, "unseen_ticks", label) > 20:
            raise ValueError(f"{episode_label}: episode exceeded elapsed/unseen bounds")
        start_tick = field_num(episode, "started_tick", episode_label)
        if start_tick > (ended_at if ended_at is not None else tick):
            raise ValueError(f"{episode_label}: episode starts in the future")
        if field_num(episode, "last_health", episode_label) > 1_000_000_000_000:
            raise ValueError(f"{episode_label}: episode health accumulator exceeds cap")
    if episode is not None:
        validate_active_episode(episode, label)
    for pending_row in pending:
        if not isinstance(pending_row, dict) or not isinstance(pending_row.get("episode"), dict):
            raise ValueError(f"{label}: malformed pending closure")
        end_tick = field_num(pending_row, "ended_tick", label)
        validate_active_episode(pending_row["episode"], label, end_tick)
        position(pending_row.get("final_pos"))
    if not enabled and (outcomes or episode is not None or pending):
        raise ValueError(f"{label}: disabled refuge-memory branch retained hidden refuge state")


def validate_branch(branch, seed, label):
    if not isinstance(branch, dict) or branch.get("status") not in {"natural_death", "completed_100_ticks"}:
        raise ValueError(f"{label}: invalid branch/status")
    initial = branch.get("initial_state")
    if not isinstance(initial, dict) or actor(initial, SUBJECT_ID) is None:
        raise ValueError(f"{label}: invalid initial state")
    expected_enabled = label == "refuge_memory"
    initial_memory = actor(initial, SUBJECT_ID).get("memory")
    validate_memory(initial_memory, 0, f"{label} initial state")
    if initial_memory.get("refuge_outcomes_enabled") is not expected_enabled:
        raise ValueError(f"{label}: initial refuge-memory flag does not match branch")
    rows = branch.get("timeline")
    if not isinstance(rows, list) or not rows or len(rows) > 100:
        raise ValueError(f"{label}: invalid or over-budget timeline")
    prev = 0
    previous_age = actor(initial, SUBJECT_ID)["life"].get("age")
    initial_subject = actor(initial, SUBJECT_ID)
    if type(previous_age) is not int:
        raise ValueError(f"{label}: initial subject age is missing")
    closures = []
    engine_closures = []
    death_events = []
    subject_movements = Counter()
    subject_threat_damage = Counter()
    subject_other_damage = Counter()
    subject_total_damage = Counter()
    for row in rows:
        tick = row.get("tick")
        if type(tick) is not int or tick != prev + 1:
            raise ValueError(f"{label}: non-contiguous physical tick timeline")
        prev = tick
        state = row.get("state")
        if not isinstance(state, dict) or state.get("tick") != tick:
            raise ValueError(f"{label}: saved state tick mismatch")
        subject = actor(state, SUBJECT_ID)
        if subject is None:
            raise ValueError(f"{label}: subject identity disappeared/replaced")
        pre_memory = row.get("memory_before")
        post_memory = row.get("memory_after")
        validate_memory(pre_memory, tick - 1, f"{label} tick {tick} memory_before")
        validate_memory(post_memory, tick, f"{label} tick {tick} memory_after")
        if pre_memory.get("refuge_outcomes_enabled") is not expected_enabled or post_memory.get("refuge_outcomes_enabled") is not expected_enabled:
            raise ValueError(f"{label}: refuge-memory enable flag changed during continuous branch")
        if pre_memory != (initial_memory if tick == 1 else rows[tick - 2].get("memory_after")):
            raise ValueError(f"{label}: memory state discontinuity between physical ticks")
        if post_memory != subject.get("memory"):
            raise ValueError(f"{label}: per-tick memory_after disagrees with complete saved state")
        age = subject.get("life", {}).get("age")
        if type(age) is not int or age != previous_age + 1:
            raise ValueError(f"{label}: subject age reset or failed to advance once per physical tick")
        if subject.get("species") != initial_subject.get("species") or subject.get("generation") != initial_subject.get("generation"):
            raise ValueError(f"{label}: same actor identity/species/generation was not retained")
        previous_age = age
        decisions = row.get("decisions")
        if not isinstance(decisions, (list, dict)):
            raise ValueError(f"{label}: per-tick decisions/perception missing")
        def reject_hidden_inputs(value):
            if isinstance(value, dict):
                for key, nested in value.items():
                    if str(key).lower() in {"patrol_index", "controller", "controller_state", "future_route", "future_positions", "hidden_route"}:
                        raise ValueError(f"{label}: controller/future state leaked into subject-facing decision record")
                    reject_hidden_inputs(nested)
            elif isinstance(value, list):
                for nested in value:
                    reject_hidden_inputs(nested)
        reject_hidden_inputs(decisions)
        for kind, payload in event_items(row.get("events")):
            if kind == "EntityDied" and payload.get("id") == SUBJECT_ID:
                death_events.append(tick)
            elif kind == "EntityMoved" and payload.get("id") == SUBJECT_ID:
                subject_movements[tick] += field_num(payload, "energy_cost", label)
            elif kind == "EntityAttacked" and payload.get("target") == SUBJECT_ID:
                attacker = payload.get("attacker")
                damage = field_num(payload, "damage", label)
                if type(attacker) is not int:
                    raise ValueError(f"{label}: attack attribution missing attacker id")
                subject_total_damage[tick] += damage
                (subject_threat_damage if attacker == PREDATOR_ID else subject_other_damage)[tick] += damage
            elif kind == "RefugeEpisodeEnded":
                if payload.get("id") != SUBJECT_ID:
                    raise ValueError(f"{label}: foreign subject refuge closure")
                engine_closures.append((tick, payload))
        audits = row.get("episode_audit")
        if not isinstance(audits, list):
            raise ValueError(f"{label}: symmetric episode_audit list missing")
        for audit in audits:
            if not isinstance(audit, dict) or audit.get("id") != SUBJECT_ID or audit.get("ended_tick") != tick:
                raise ValueError(f"{label}: invalid observer episode closure/tick")
            closures.append((tick, audit))
    if branch["status"] == "completed_100_ticks":
        if len(rows) != 100 or death_events:
            raise ValueError(f"{label}: completed branch must be alive at exactly 100 ticks")
        if life_snapshot(rows[-1]["state"])["health"] == 0:
            raise ValueError(f"{label}: zero-health subject labeled completed")
    else:
        if len(death_events) != 1 or death_events[0] != rows[-1]["tick"]:
            raise ValueError(f"{label}: natural death must stop on one terminal EntityDied event")
        if life_snapshot(rows[-1]["state"])["health"] != 0:
            raise ValueError(f"{label}: death event lacks terminal zero health")
    validate_checkpoint(branch, label)

    if label == "baseline" and engine_closures:
        raise ValueError("baseline unexpectedly emitted feature-owned RefugeEpisodeEnded events")

    episode_rows = []
    seen_closures = set()
    for tick, episode in closures:
        started = field_num(episode, "started_tick", label)
        ended = field_num(episode, "ended_tick", label)
        if ended != tick or started < 1 or started > ended:
            raise ValueError(f"{label}: invalid episode closure timing")
        if ended - started > 20:
            raise ValueError(f"{label}: episode exceeded registered 20-tick limit")
        marker = (started, ended, json.dumps(episode.get("context_key"), sort_keys=True), episode.get("closure"))
        if marker in seen_closures:
            raise ValueError(f"{label}: duplicate episode closure")
        seen_closures.add(marker)
        endpoint = position(episode.get("endpoint")) if episode.get("endpoint") is not None else None
        final_pos = position(episode.get("final_pos")) if episode.get("final_pos") is not None else None
        if not isinstance(episode.get("context_key"), dict):
            raise ValueError(f"{label}: closure lacks perceived context key")
        context = episode["context_key"]
        expected_context_fields = {"threat_id", "threat_species", "bearing", "threat_distance",
                                   "energy_quartile", "stamina_quartile", "refuge_options"}
        if set(context) != expected_context_fields or context.get("threat_id") != episode.get("threat_id"):
            raise ValueError(f"{label}: context key does not match the registered perceived-context schema")
        if context.get("threat_species") != episode.get("threat_species"):
            raise ValueError(f"{label}: context threat species does not match closure")
        if type(context.get("bearing")) is not int or context["bearing"] not in range(8):
            raise ValueError(f"{label}: invalid relative-bearing bucket")
        if type(context.get("threat_distance")) is not int or context["threat_distance"] not in {0, 1, 2}:
            raise ValueError(f"{label}: invalid threat-distance bucket")
        if context.get("energy_quartile") not in {0, 1, 2, 3} or context.get("stamina_quartile") not in {0, 1, 2, 3}:
            raise ValueError(f"{label}: invalid energy/stamina quartile")
        options = context.get("refuge_options")
        if not isinstance(options, list):
            raise ValueError(f"{label}: perceived refuge-option set missing")
        normalized_options = [position(option) for option in options]
        if normalized_options != sorted(normalized_options, key=lambda p: (p["y"], p["x"])):
            raise ValueError(f"{label}: refuge endpoints are not in stable sorted order")
        if len({(p["x"], p["y"]) for p in normalized_options}) != len(normalized_options):
            raise ValueError(f"{label}: duplicate perceived refuge endpoint")
        resolution = episode.get("endpoint_resolution")
        candidates = episode.get("endpoint_candidates")
        if resolution not in {"unique", "no_move_path", "unmatched", "ambiguous_prefix"}:
            raise ValueError(f"{label}: invalid endpoint-resolution status")
        if not isinstance(candidates, list):
            raise ValueError(f"{label}: endpoint candidates must be recorded")
        normalized_candidates = [position(candidate) for candidate in candidates]
        if normalized_candidates != normalized_options:
            raise ValueError(f"{label}: endpoint candidates disagree with visible refuge alternatives")
        if resolution == "unique":
            if endpoint is None or endpoint not in normalized_options:
                raise ValueError(f"{label}: unique endpoint resolution is not supported by visible choices")
        elif endpoint is not None:
            raise ValueError(f"{label}: unresolved endpoint must remain unknown")
        if type(episode.get("completed")) is not bool or type(episode.get("attribution_known")) is not bool:
            raise ValueError(f"{label}: completed/attribution flags must be booleans")
        normalized_closure = str(episode.get("closure", "")).lower()
        if (normalized_closure == "arrived") != episode["completed"]:
            raise ValueError(f"{label}: completed route must correspond exactly to arrived closure")
        if normalized_closure == "endpoint_ambiguous" and (episode["completed"] or episode.get("safe_arrival")):
            raise ValueError(f"{label}: ambiguous endpoint arrival must be censored")
        predecision = normalized_closure in AUDIT_PREDECISION_CLOSURES
        event_end = ended - 1 if predecision else ended
        expected_move = sum(subject_movements[t] for t in range(started, event_end + 1))
        expected_threat_damage = sum(subject_threat_damage[t] for t in range(started, event_end + 1))
        expected_other_damage = sum(subject_other_damage[t] for t in range(started, event_end + 1))
        for key in ("movement_energy", "threat_damage", "other_damage", "unattributed_health_loss"):
            field_num(episode, key, label)
        if episode["movement_energy"] != expected_move:
            raise ValueError(f"{label}: episode movement energy disagrees with EntityMoved resolver events")
        if episode["threat_damage"] != expected_threat_damage or episode["other_damage"] != expected_other_damage:
            raise ValueError(f"{label}: episode damage attribution disagrees with EntityAttacked events")
        if (episode["other_damage"] > 0 or episode["unattributed_health_loss"] > 0) and episode["attribution_known"]:
            raise ValueError(f"{label}: foreign/unattributed damage was not censored")
        end_row = closure_row_at(rows, ended)
        trace = subject_trace(end_row) if end_row is not None else None
        if predecision and (trace is None or not isinstance(trace.get("pos"), dict) or not isinstance(trace.get("before"), dict)):
            raise ValueError(f"{label}: predecision closure lacks same-tick full decision-trace origin/body")
        event_subject_pos = position(trace["pos"]) if predecision else life_snapshot(rows[ended - 1]["state"])["position"]
        if final_pos != event_subject_pos:
            raise ValueError(f"{label}: closure final position disagrees with saved world state")
        start_row = closure_row_at(rows, started)
        start_trace = subject_trace(start_row) if start_row is not None else None
        if start_trace is None or start_trace.get("tick") != started:
            raise ValueError(f"{label}: episode does not start on a recorded subject choice")
        if start_trace.get("decision", {}).get("reason") != episode.get("initial_reason"):
            raise ValueError(f"{label}: observer initial reason disagrees with DecisionTrace")
        if episode.get("initial_reason") != "SeekShelter":
            raise ValueError(f"{label}: episode did not start on an ordinary SeekShelter choice")
        if start_trace.get("perception") is None:
            raise ValueError(f"{label}: episode start lacks full current perception")
        visible_threat = next((e for e in start_trace["perception"].get("entities", [])
                               if e.get("id") == episode.get("threat_id")
                               and e.get("species") == episode.get("threat_species")), None)
        if visible_threat is None:
            raise ValueError(f"{label}: episode did not start with the threat visible in perception")
        threat_steps = decode_steps(visible_threat.get("steps"))
        if not threat_steps:
            raise ValueError(f"{label}: visible threat has no perceived relative direction")
        expected_bearing = list(DIRS).index(threat_steps[0])
        expected_distance = min(len(threat_steps) - 1, 2)
        if context["bearing"] != expected_bearing or context["threat_distance"] != expected_distance:
            raise ValueError(f"{label}: context bearing/distance disagrees with current threat perception")
        before = start_trace.get("before", {})
        energy_q = min(3, 4 * min(before.get("energy", 0), before.get("max_energy", 0)) // max(1, before.get("max_energy", 0)))
        stamina_q = min(3, 4 * min(before.get("stamina", 0), before.get("max_stamina", 0)) // max(1, before.get("max_stamina", 0)))
        if context["energy_quartile"] != energy_q or context["stamina_quartile"] != stamina_q:
            raise ValueError(f"{label}: context energy/stamina quartiles disagree with actual body")
        threat_position = walk_path(start_trace["pos"], threat_steps)
        if episode.get("visible_threat_position") != threat_position:
            raise ValueError(f"{label}: visible threat position was not reconstructed from perception")
        expected_refuges = [p for p, _ in visible_refuges(start_trace)]
        if normalized_options != expected_refuges:
            raise ValueError(f"{label}: refuge alternatives disagree with visible unoccupied shelter paths")
        if episode.get("starting_health") != start_trace.get("before", {}).get("health"):
            raise ValueError(f"{label}: observer starting health disagrees with DecisionTrace")
        expected_final_health = trace.get("before", {}).get("health") if predecision else life_snapshot(rows[ended - 1]["state"])["health"]
        if episode.get("final_health") != expected_final_health:
            raise ValueError(f"{label}: observer final health disagrees with resolver/decision state")
        previous_health = episode["starting_health"]
        expected_unattributed = 0
        for current_tick in range(started, ended + 1):
            current_row = closure_row_at(rows, current_tick)
            current_trace = subject_trace(current_row) if current_row is not None else None
            if current_trace is None:
                death_before_intent = (
                    current_tick == ended == len(rows)
                    and branch["status"] == "natural_death"
                    and normalized_closure == "died"
                    and current_tick in death_events
                    and life_snapshot(current_row["state"])["health"] == 0
                    and not predecision
                )
                if not death_before_intent:
                    raise ValueError(f"{label}: episode health audit lacks a decision trace outside terminal death-before-intent")
                after_health = life_snapshot(current_row["state"])["health"]
                expected_unattributed += max(0, previous_health - after_health - subject_total_damage[current_tick])
                previous_health = after_health
                continue
            if type(current_trace.get("before", {}).get("health")) is not int:
                raise ValueError(f"{label}: episode health audit has invalid trace health")
            before_health = current_trace["before"]["health"]
            expected_unattributed += max(0, previous_health - before_health)
            if not (predecision and current_tick == ended):
                after_health = life_snapshot(current_row["state"])["health"]
                expected_unattributed += max(0, before_health - after_health - subject_total_damage[current_tick])
                previous_health = after_health
            else:
                previous_health = before_health
        if episode["unattributed_health_loss"] != expected_unattributed:
            raise ValueError(f"{label}: unattributed health loss disagrees with per-tick body and resolver damage")
        selected_path = episode.get("initial_selected_path")
        decision_intent = start_trace.get("decision", {}).get("intent", {})
        selected_path = decode_steps(selected_path)
        if decision_intent.get("Move", {}).get("steps") != selected_path or episode.get("initial_intent") != decision_intent:
            raise ValueError(f"{label}: observer selected path disagrees with actual decision")
        endpoint_matches = [p for p, path in visible_refuges(start_trace)
                            if path[:len(selected_path)] == selected_path and selected_path]
        expected_resolution = ("no_move_path" if not selected_path else
                               "unique" if len(endpoint_matches) == 1 else
                               "ambiguous_prefix" if len(endpoint_matches) > 1 else "unmatched")
        if resolution != expected_resolution or (resolution == "unique" and endpoint != endpoint_matches[0]):
            raise ValueError(f"{label}: endpoint resolution disagrees with current visible shelter paths")
        if episode.get("safe_arrival") is True:
            if (normalized_closure != "arrived" or episode.get("completed") is not True
                    or episode.get("attribution_known") is not True
                    or episode["threat_damage"] != 0 or episode["other_damage"] != 0
                    or endpoint is None or final_pos != endpoint or resolution != "unique"):
                raise ValueError(f"{label}: unsafe or unverified route marked safe arrival")
        elif episode.get("safe_arrival") is not False:
            raise ValueError(f"{label}: safe-arrival field must be boolean")
        if episode.get("attribution_known") is False and episode.get("safe_arrival"):
            raise ValueError(f"{label}: censored attribution credited as safe")
        episode_rows.append({
            **episode, "branch": label, "seed": seed, "closed_tick": tick,
            "endpoint": endpoint, "final_pos": final_pos,
            "health_start": episode.get("starting_health"),
            "health_end": (trace.get("before", {}).get("health") if predecision else life_snapshot(rows[ended - 1]["state"])["health"]),
            "memory_before": closure_row_at(rows, started).get("memory_before"),
            "memory_after": closure_row_at(rows, ended).get("memory_after"),
            "choice_and_perception_rows": [
                {"tick": r["tick"], "subject_trace": subject_trace(r),
                 "subject_events": [event for event in r.get("events", [])
                                    if isinstance(event, dict) and any(
                                        isinstance(payload, dict) and
                                        (payload.get("id") == SUBJECT_ID or payload.get("target") == SUBJECT_ID)
                                        for payload in event.values())],
                 "post_position": life_snapshot(r["state"])["position"]}
                for r in rows if started <= r["tick"] <= ended
            ],
        })
    for tick in range(1, len(rows) + 1):
        count = sum(1 for closure_tick, _ in closures if closure_tick == tick)
        if count > 1 and not isinstance(rows[tick - 1].get("events"), list):
            raise ValueError(f"{label}: invalid same-tick closure collection")
    if len(episode_rows) != len(closures):
        raise ValueError(f"{label}: closure accounting mismatch")
    unmatched = list(episode_rows)
    unmapped_engine_events = []
    for ended_tick, event in engine_closures:
        event_context = visible_context(event.get("context_key"))
        candidate = next((row for row in unmatched
                          if row["ended_tick"] == ended_tick
                          and row["started_tick"] == event.get("started_tick")
                          and row["threat_id"] == event.get("threat_id")
                          and visible_context(row.get("context_key")) == event_context
                          and (row.get("endpoint") is None or row.get("endpoint") == event.get("endpoint"))), None)
        if candidate is None:
            raise ValueError(f"{label}: engine RefugeEpisodeEnded lacks any matching observer closure")
        if candidate.get("endpoint_resolution") != "unique":
            unmapped_engine_events.append({"event": event, "observer_reason": candidate.get("endpoint_resolution")})
            unmatched.remove(candidate)
            continue
        for field in ("movement_energy", "threat_damage", "other_damage", "unattributed_health_loss", "final_pos"):
            if candidate.get(field) != event.get(field):
                raise ValueError(f"{label}: observer and engine episode audit disagree on {field}")
        if candidate.get("endpoint_resolution") == "unique":
            if str(candidate.get("closure")).lower() != str(event.get("closure")).lower():
                raise ValueError(f"{label}: observer and engine closure reasons disagree")
            if candidate.get("safe_arrival") != event.get("safe_arrival"):
                raise ValueError(f"{label}: observer and engine safe-arrival audit disagree")
        unmatched.remove(candidate)
    subject_start = life_snapshot(initial)
    subject_end = life_snapshot(rows[-1]["state"])
    first_safe_tick = min((e["closed_tick"] for e in episode_rows if e["safe_arrival"]), default=None)
    post_escape_rows = [r for r in rows if first_safe_tick is not None and r["tick"] > first_safe_tick]
    post_escape_food = 0
    post_escape_water = 0
    resource_reasons = {"SeekFood", "SearchFood", "RecallFood", "SeekWater", "SearchWater", "RecallWater", "LearnedWater"}
    resource_ticks = []
    for row in post_escape_rows:
        trace = subject_trace(row)
        reason = trace.get("decision", {}).get("reason") if trace else None
        if reason in resource_reasons:
            resource_ticks.append({"tick": row["tick"], "reason": reason,
                                   "movement_energy_paid": subject_movements[row["tick"]]})
        for kind, payload in event_items(row.get("events")):
            if kind == "EntityAte" and payload.get("id") == SUBJECT_ID:
                post_escape_food += field_num(payload, "amount", label)
            elif kind == "EntityDrank" and payload.get("id") == SUBJECT_ID:
                post_escape_water += field_num(payload, "amount", label)
    return {
        "branch": label, "seed": seed, "status": branch["status"],
        "physical_ticks": len(rows), "initial_subject": subject_start,
        "final_subject": subject_end,
        "movement_energy": sum(subject_movements.values()),
        "threat_attributed_damage": sum(subject_threat_damage.values()),
        "other_attributed_damage": sum(subject_other_damage.values()),
        "body_and_table_change": {
            "health_delta": (subject_end["health"] or 0) - (subject_start["health"] or 0),
            "energy_delta": (subject_end["energy"] or 0) - (subject_start["energy"] or 0),
            "stamina_delta": (subject_end["stamina"] or 0) - (subject_start["stamina"] or 0),
            "refuge_outcomes_start": subject_start["memory"].get("refuge_outcomes", []),
            "refuge_outcomes_end": subject_end["memory"].get("refuge_outcomes", []),
        },
        "post_escape": {
            "first_safe_arrival_tick": first_safe_tick,
            "ticks_after_first_safe_arrival": len(post_escape_rows),
            "food_acquired": post_escape_food, "water_acquired": post_escape_water,
            "movement_energy_paid": sum(subject_movements[r["tick"]] for r in post_escape_rows),
            "resource_seeking_ticks": len(resource_ticks),
            "resource_seeking_movement_energy_paid": sum(row["movement_energy_paid"] for row in resource_ticks),
            "resource_seeking_decisions": resource_ticks,
        },
        "episodes": episode_rows,
        "closures": len(episode_rows),
        "safe_arrivals": sum(e["safe_arrival"] for e in episode_rows),
        "censored_closures": sum(not e["completed"] or not e["attribution_known"] for e in episode_rows),
        "terminal_memory": subject_end["memory"],
        "engine_events_unmapped_to_unique_observer_endpoint": unmapped_engine_events,
    }


def comparable_context(a, b):
    left, right = a.get("context_key"), b.get("context_key")
    if not isinstance(left, dict) or not isinstance(right, dict):
        return False
    return all(left.get(key) == right.get(key) for key in (
        "threat_id", "threat_species", "bearing", "threat_distance",
        "energy_quartile", "stamina_quartile", "refuge_options",
    ))


def first_last_same_context(episodes):
    result = []
    for seed in SEEDS:
        for branch in ("baseline", "refuge_memory"):
            subset = [e for e in episodes if e["seed"] == seed and e["branch"] == branch]
            groups = defaultdict(list)
            for episode in subset:
                key = episode.get("context_key")
                if isinstance(key, dict):
                    visible_context = {field: key.get(field) for field in (
                        "threat_id", "threat_species", "bearing", "threat_distance",
                        "energy_quartile", "stamina_quartile", "refuge_options",
                    )}
                    groups[json.dumps(visible_context, sort_keys=True)].append(episode)
            repeated = [group for group in groups.values() if len(group) > 1]
            if repeated:
                for group in repeated:
                    result.append({"seed": seed, "branch": branch, "available": True,
                                   "context": {field: group[0]["context_key"].get(field) for field in (
                                       "threat_id", "threat_species", "bearing", "threat_distance",
                                       "energy_quartile", "stamina_quartile", "refuge_options")},
                                   "first": context_episode(group[0]), "last": context_episode(group[-1]),
                                   "context_observations": len(group)})
            else:
                result.append({"seed": seed, "branch": branch, "available": False,
                               "reason": "no genuinely repeated visible context; death/unavailability is not replaced"})
    return result


def context_episode(episode):
    fields = ("closed_tick", "closure", "endpoint", "final_pos", "completed", "safe_arrival",
              "threat_damage", "other_damage", "unattributed_health_loss", "movement_energy",
              "health_start", "health_end", "initial_reason", "initial_selected_path")
    return {field: episode.get(field) for field in fields}


def summarize(raw_path: Path) -> dict:
    rows = []
    with raw_path.open("r", encoding="utf-8") as stream:
        for line_number in range(1, len(SEEDS) + 2):
            line = stream.readline(MAX_RAW_LINE_BYTES + 1)
            if not line:
                break
            if len(line.encode("utf-8")) > MAX_RAW_LINE_BYTES:
                raise ValueError(f"raw JSONL line {line_number} exceeded 64 MiB")
            try:
                rows.append(json.loads(line))
            except json.JSONDecodeError as error:
                raise ValueError(f"invalid raw JSONL line {line_number}: {error}") from error
        if stream.read(1):
            raise ValueError("unexpected extra raw lab output")
    if len(rows) != len(SEEDS):
        raise ValueError(f"expected exactly {len(SEEDS)} raw seed rows, received {len(rows)}")
    branches, seed_summaries = [], []
    total_ticks = 0
    for expected_seed, row in zip(SEEDS, rows, strict=True):
        if row.get("schema") != RAW_SCHEMA or row.get("seed") != expected_seed:
            raise ValueError(f"raw schema/seed mismatch at expected seed {expected_seed}")
        validate_initial_pair(row)
        seed_branches = []
        for name in ("baseline", "refuge_memory"):
            parsed = validate_branch(row.get(name), expected_seed, name)
            branches.append(parsed)
            total_ticks += parsed["physical_ticks"]
            seed_branches.append({
                "branch": parsed["branch"], "status": parsed["status"],
                "physical_ticks": parsed["physical_ticks"], "closures": parsed["closures"],
                "safe_arrivals": parsed["safe_arrivals"], "censored_closures": parsed["censored_closures"],
                "movement_energy": parsed["movement_energy"],
                "threat_attributed_damage": parsed["threat_attributed_damage"],
                "other_attributed_damage": parsed["other_attributed_damage"],
                "safe_arrival_rate_per_closure": (parsed["safe_arrivals"] / parsed["closures"] if parsed["closures"] else None),
                "threat_damage_per_closure": (parsed["threat_attributed_damage"] / parsed["closures"] if parsed["closures"] else None),
                "health_start": parsed["initial_subject"]["health"],
                "health_end": parsed["final_subject"]["health"],
            })
        expected_ticks = sum(x["physical_ticks"] for x in seed_branches)
        if row.get("physical_ticks") != expected_ticks:
            raise ValueError(f"seed {expected_seed}: physical tick total mismatch")
        seed_summaries.append({"seed": expected_seed, "physical_ticks": expected_ticks,
                               "branches": seed_branches})
    if total_ticks > MAX_TICKS:
        raise ValueError("physical ticks exceeded preregistered 1,600-tick ceiling")
    return {
        "schema": "curious-herbivore-lab-summary-v1", "protocol_version": 1,
        "ticks": total_ticks, "tick_ceiling": MAX_TICKS,
        "seeds": seed_summaries,
        "descriptives": branches,
        "first_last_same_visible_context": first_last_same_context(
            [episode for branch in branches for episode in branch["episodes"]]),
        "interpretation": "Descriptive, bounded individual escape outcomes only. Safe arrivals and attributed damage use eligible observed episodes; all censored closures remain in the denominator. Learning from completed arrivals cannot establish reduced mortality; unavailable/dead later matches are not imputed.",
    }


def render_report(summary, manifest):
    lines = [
        "# Herbivore danger and refuge lab — bounded descriptive report", "",
        f"Revision `{manifest['revision']}`; {summary['ticks']} / {MAX_TICKS} physical ticks.",
        f"Raw evidence SHA-256: `{manifest['raw_sha256']}`.", "",
        "This report covers eight registered seeds and paired continuous branches. A safe arrival requires resolver-verified refuge entry and no damage from the observed threat. Censored closures are retained; the laboratory cannot establish reduced mortality because learning updates only after completed arrivals.", "",
        "| Seed | Branch | Status | Ticks | Closures (denominator) | Safe arrivals / closures | Censored | Movement energy | Threat damage / closure | Other damage | Health start→end |",
        "|---:|---|---|---:|---:|---:|---:|---:|---:|---:|---|",
    ]
    for seed in summary["seeds"]:
        for b in seed["branches"]:
            rate = "—" if b["safe_arrival_rate_per_closure"] is None else f"{b['safe_arrival_rate_per_closure']:.3f}"
            damage_rate = "—" if b["threat_damage_per_closure"] is None else f"{b['threat_damage_per_closure']:.3f}"
            lines.append(f"| {seed['seed']} | {b['branch']} | {b['status']} | {b['physical_ticks']} | {b['closures']} | {b['safe_arrivals']} / {rate} | {b['censored_closures']} | {b['movement_energy']} | {b['threat_attributed_damage']} / {damage_rate} | {b['other_attributed_damage']} | {b['health_start']}→{b['health_end']} |")
    lines += ["", "First versus last genuinely matching visible context:", "",
              "| Seed | Branch | Available | First/last tick | First/last safe arrival | First/last damage |",
              "|---:|---|---|---|---|---|"]
    for pair in summary["first_last_same_visible_context"]:
        if pair["available"]:
            first, last = pair["first"], pair["last"]
            lines.append(f"| {pair['seed']} | {pair['branch']} | yes | {first['closed_tick']} / {last['closed_tick']} | {first['safe_arrival']} / {last['safe_arrival']} | {first['threat_damage']} / {last['threat_damage']} |")
        else:
            lines.append(f"| {pair['seed']} | {pair['branch']} | no | — | — | — |")
    lines += ["", "Per-choice perception, decision/path records, endpoint/final position, memory snapshots, costs, and every closure are retained in `summary.json` and `raw.jsonl`. Absence of observed threat damage is not a mortality result; dead or unmatched branches remain unavailable.", ""]
    return "\n".join(lines)


def capture_process(command, cwd, raw_path, stderr_path, timeout=MAX_SECONDS, byte_limit=MAX_OUTPUT_BYTES):
    """Bound both wall time and stdout bytes, including partial-line/closed-stdout cases."""
    start = __import__("time").monotonic()
    def child_limits():
        import resource
        resource.setrlimit(resource.RLIMIT_AS, (MAX_CHILD_MEMORY_BYTES, MAX_CHILD_MEMORY_BYTES))
    try:
        with raw_path.open("xb") as raw, stderr_path.open("xb") as stderr_file:
            process = subprocess.Popen(command, cwd=cwd, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                        start_new_session=True, preexec_fn=child_limits)
            assert process.stdout is not None and process.stderr is not None
            stdout_fd, stderr_fd = process.stdout.fileno(), process.stderr.fileno()
            os.set_blocking(stdout_fd, False)
            os.set_blocking(stderr_fd, False)
            import selectors
            with selectors.DefaultSelector() as selector:
                selector.register(stdout_fd, selectors.EVENT_READ, "stdout")
                selector.register(stderr_fd, selectors.EVENT_READ, "stderr")
                total = raw_bytes = stderr_bytes = 0
                timed_out = False
                too_large = False
                while True:
                    remaining = timeout - (__import__("time").monotonic() - start)
                    if remaining <= 0 and process.poll() is None:
                        timed_out = True
                        import signal
                        try:
                            os.killpg(process.pid, signal.SIGKILL)
                        except ProcessLookupError:
                            pass
                    if selector.get_map():
                        for key, _ in selector.select(timeout=min(0.1, max(0.0, remaining))):
                            try:
                                chunk = os.read(key.fd, 65536)
                            except BlockingIOError:
                                continue
                            if not chunk:
                                selector.unregister(key.fd)
                                continue
                            if total + len(chunk) > byte_limit:
                                too_large = True
                                import signal
                                try:
                                    os.killpg(process.pid, signal.SIGKILL)
                                except ProcessLookupError:
                                    pass
                                break
                            target = raw if key.data == "stdout" else stderr_file
                            target.write(chunk)
                            target.flush()
                            total += len(chunk)
                            if key.data == "stdout":
                                raw_bytes += len(chunk)
                            else:
                                stderr_bytes += len(chunk)
                    if process.poll() is not None and not selector.get_map():
                        break
                    if not selector.get_map() and process.poll() is None:
                        try:
                            process.wait(timeout=min(0.05, max(0.001, remaining)))
                        except subprocess.TimeoutExpired:
                            continue
                if process.poll() is None:
                    import signal
                    try:
                        os.killpg(process.pid, signal.SIGKILL)
                    except ProcessLookupError:
                        pass
                process.wait(timeout=5)
            process.stdout.close()
            process.stderr.close()
        return {"exit_code": process.returncode, "timed_out": timed_out,
                "output_limit_exceeded": too_large, "raw_bytes": raw_bytes,
                "stderr_bytes": stderr_bytes, "total_output_bytes": total,
                "status": "completed" if not timed_out and not too_large and process.returncode == 0 else "failed",
                "elapsed_seconds": __import__("time").monotonic() - start}
    except Exception as error:
        return {"status": "failed", "runner_error": f"{type(error).__name__}: {error}",
                "elapsed_seconds": __import__("time").monotonic() - start}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, default=ROOT / "target/release/examples/herbivore_danger_lab")
    parser.add_argument("--output", type=Path, default=REGISTERED_OUTPUT)
    parser.add_argument("--seed-registry", type=Path, default=Path("/root/curious/data/experiments"))
    args = parser.parse_args()
    output, registry = args.output.resolve(), args.seed_registry.resolve()
    if output != REGISTERED_OUTPUT.resolve():
        raise SystemExit(f"registered output path is fixed: {REGISTERED_OUTPUT}")
    if output.exists():
        raise SystemExit(f"refusing to overwrite existing run: {output}")
    status = subprocess.check_output(["git", "status", "--porcelain", "--untracked-files=all"], cwd=ROOT, text=True)
    if status.strip():
        raise SystemExit("registered run requires a clean committed source worktree")
    seed_audit = declared_seed_audit(registry)
    if seed_audit["unseen"] != SEEDS:
        raise SystemExit(f"registered seeds are not all unseen: {seed_audit['matches']}")
    binary = args.binary.resolve(strict=True)
    commit = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()
    hashes = source_hashes()
    source_digest = hashlib.sha256(json.dumps(hashes, sort_keys=True, separators=(",", ":")).encode()).hexdigest()
    output.mkdir(parents=True, exist_ok=False)
    probe = output / "lab-probe"
    shutil.copy2(binary, probe)
    probe.chmod(0o555)
    protocol_copy = output / "protocol.md"
    shutil.copy2(PROTOCOL, protocol_copy)
    protocol_copy.chmod(0o444)
    source_dir = output / "source"
    for name in source_files():
        dest = source_dir / name
        dest.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(ROOT / name, dest)
        dest.chmod(0o444)
    manifest = {
        "schema": SCHEMA, "protocol_version": 1, "run_name": "herbivore-danger-lab",
        "status": "running", "revision": commit, "git_dirty": False,
        "source_sha256": source_digest, "source_files_sha256": hashes,
        "binary_sha256": sha(probe), "artifact_name": "lab-probe",
        "protocol_sha256": sha(protocol_copy), "runner_sha256": sha(Path(__file__)),
        "harness_sha256": sha(HARNESS), "seeds": SEEDS, "seed_audit": seed_audit,
        "branches": ["baseline", "refuge_memory"], "ticks_per_branch_ceiling": 100,
        "tick_ceiling": MAX_TICKS, "command": ["./lab-probe"],
        "started_at": datetime.now(timezone.utc).isoformat(),
        "timeout_seconds": MAX_SECONDS, "max_output_bytes": MAX_OUTPUT_BYTES,
        "max_child_memory_bytes": MAX_CHILD_MEMORY_BYTES,
        "probe_file": "lab-probe", "protocol_file": "protocol.md",
        "source_directory": "source", "raw_file": "raw.jsonl", "summary_file": "summary.json",
    }
    manifest_path = output / "manifest.json"
    manifest_path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    result = capture_process([str(probe)], output, output / "raw.jsonl", output / "stderr.log")
    manifest.update(result)
    try:
        if result.get("status") != "completed":
            raise RuntimeError(result.get("runner_error") or "probe failed, timed out, or exceeded output budget")
        manifest["raw_sha256"] = sha(output / "raw.jsonl")
        summary = summarize(output / "raw.jsonl")
        summary_path = output / "summary.json"
        summary_path.write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
        summary_path.chmod(0o444)
        manifest["summary_sha256"] = sha(summary_path)
        report_path = output / "report.md"
        report_path.write_text(render_report(summary, manifest), encoding="utf-8")
        report_path.chmod(0o444)
        manifest["report_sha256"] = sha(report_path)
        (output / "raw.jsonl").chmod(0o444)
        (output / "stderr.log").chmod(0o444)
        manifest["status"] = "completed"
    except Exception as error:
        manifest["status"] = "failed"
        manifest["error"] = f"{type(error).__name__}: {error}"
        if (output / "raw.jsonl").exists():
            manifest["raw_sha256"] = sha(output / "raw.jsonl")
        if (output / "stderr.log").exists():
            manifest["stderr_sha256"] = sha(output / "stderr.log")
    manifest["finished_at"] = datetime.now(timezone.utc).isoformat()
    manifest_path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    if manifest["status"] != "completed":
        raise SystemExit(f"lab failed; partial evidence preserved in {output}")
    print(json.dumps({"status": "completed", "output": str(output), "raw_sha256": manifest["raw_sha256"]}))


if __name__ == "__main__":
    main()
