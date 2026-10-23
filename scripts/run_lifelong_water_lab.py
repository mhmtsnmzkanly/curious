#!/usr/bin/env python3
"""Run and validate the registered continuous-lifetime water lab.

The runner has two explicit phases. It never builds the Rust probe, modifies
the fixture, or changes a seed list. Raw output is retained even when the
process or an audit fails.
"""
from __future__ import annotations

import argparse
from copy import deepcopy
from datetime import datetime, timezone
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import resource
import shutil
import subprocess
import sys
import time
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
CANONICAL_ARCHIVE = Path("/root/curious/data/experiments")
PROTOCOL = Path("docs/LIFELONG_WATER_PROTOCOL.md")
REANALYSIS_PROTOCOL = Path("docs/LIFELONG_WATER_REANALYSIS.md")
SCANNER = Path("scripts/run_ecology_traits_lab.py")
RUNS = {
    "calibration": (51099,),
    "formal": (51001, 51002, 51003, 51004, 51005, 51006, 51007, 51008),
}
OUTPUT_DIRS = {
    "calibration": "lifetime-lab-v2-calibration-20261023",
    "formal": "lifetime-lab-v2-20261023",
}
CALIBRATION_REANALYSIS_DIR = "lifetime-lab-v2-calibration-reanalysis2-20261023"
SEED_SCHEMA = "curious-lifetime-seed-v2"
SUMMARY_SCHEMA = "curious-lifetime-summary-v2"
MANIFEST_SCHEMA = "curious-lifetime-lab-run-v2"
PROTOCOL_VERSION = 2
MAX_SECONDS = 300
MAX_OUTPUT_BYTES = 1024**3
MAX_SEED_RECORD_BYTES = 64 * 1024**2
MAX_CHILD_MEMORY_BYTES = 1024**3
FOCAL_ID = 1
COMPETITOR_ID = 2
FORK_TICK = 200
FINAL_TICK = 500
WATER_POSITIONS = ((0, 2), (4, 2))
TREATMENTS = {
    "online": (True, True, "retain"),
    "trained_frozen": (True, False, "retain"),
    "ablated_frozen": (True, False, "clear"),
    "rule_based": (False, False, "retain"),
}
TREATMENT_DESCRIPTIONS = {
    "online": "enabled=true;update_enabled=true;retain-values",
    "trained_frozen": "enabled=true;update_enabled=false;retain-values",
    "ablated_frozen": "enabled=true;update_enabled=false;clear-values-only",
    "rule_based": "enabled=false;update_enabled=false;retain-unused-values",
}
DIRECTIONS = {
    "Up": (0, -1), "UpRight": (1, -1), "Right": (1, 0),
    "DownRight": (1, 1), "Down": (0, 1), "DownLeft": (-1, 1),
    "Left": (-1, 0), "UpLeft": (-1, -1),
}


def now() -> str:
    return datetime.now(timezone.utc).isoformat()


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def canonical_sha(value: Any) -> str:
    encoded = json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode()
    return hashlib.sha256(encoded).hexdigest()


def source_paths(source_root: Path) -> list[Path]:
    paths = [source_root / "Cargo.toml", source_root / "Cargo.lock", source_root / PROTOCOL,
             source_root / SCANNER, Path(__file__).resolve()]
    for parent in (source_root / "src", source_root / "examples"):
        paths.extend(parent.rglob("*.rs"))
        paths.extend(parent.rglob("*.json"))
    unique = {path.resolve() for path in paths}
    missing = [str(path) for path in unique if not path.is_file()]
    if missing:
        raise RuntimeError(f"registered source files are missing: {sorted(missing)}")
    return sorted(unique, key=lambda p: str(p.relative_to(source_root)))


def source_hashes(source_root: Path) -> dict[str, str]:
    return {
        str(path.relative_to(source_root)): sha256_file(path)
        for path in source_paths(source_root)
    }


def source_map_sha256(hashes: dict[str, str]) -> str:
    return hashlib.sha256(json.dumps(hashes, sort_keys=True, separators=(",", ":")).encode()).hexdigest()


def source_identity(source_root: Path) -> tuple[str, dict[str, str]]:
    status = subprocess.check_output(
        ["git", "status", "--porcelain", "--untracked-files=all"],
        cwd=source_root, text=True,
    )
    if status.strip():
        raise RuntimeError("registered run requires a clean committed source worktree")
    revision = subprocess.check_output(
        ["git", "rev-parse", "HEAD"], cwd=source_root, text=True,
    ).strip()
    return revision, source_hashes(source_root)


def scan_seeds(registry: Path, seeds: tuple[int, ...],
               max_line_bytes: int = MAX_SEED_RECORD_BYTES) -> dict[str, Any]:
    """Use the established declared-seed scanner on the actual experiment archive."""
    module_path = ROOT / SCANNER
    spec = importlib.util.spec_from_file_location("curious_seed_archive_scanner", module_path)
    if spec is None or spec.loader is None:
        raise RuntimeError("cannot load the archived experiment seed scanner")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module.audit_seeds(registry, list(seeds), max_line_bytes=max_line_bytes)


def actor(state: dict[str, Any], actor_id: int) -> dict[str, Any] | None:
    entities = state.get("entities")
    if not isinstance(entities, list):
        return None
    return next((entity for entity in entities if entity.get("id") == actor_id), None)


def life(entity: dict[str, Any] | None) -> dict[str, Any]:
    if entity is None or not isinstance(entity.get("life"), dict):
        raise ValueError("serialized actor is missing life state")
    return entity["life"]


def learner(state: dict[str, Any], actor_id: int = FOCAL_ID) -> dict[str, Any]:
    entity = actor(state, actor_id)
    value = None if entity is None else entity.get("learner")
    if not isinstance(value, dict):
        raise ValueError(f"actor {actor_id} has no serialized water learner")
    return value


def pos_tuple(value: Any) -> tuple[int, int]:
    if not isinstance(value, dict) or type(value.get("x")) is not int or type(value.get("y")) is not int:
        raise ValueError("invalid serialized position")
    return value["x"], value["y"]


def actor_position(entity: dict[str, Any]) -> tuple[int, int]:
    return pos_tuple(entity.get("pos"))


def candidate_positions(choice: dict[str, Any], decision: dict[str, Any] | None = None) -> list[tuple[int, int]]:
    if decision is not None:
        origin = actor_position(decision)
    else:
        origin = None
    candidates = choice.get("candidates")
    if not isinstance(candidates, list):
        raise ValueError("choice exposure candidates must be an array")
    out = []
    for candidate in candidates:
        pos = pos_tuple(candidate.get("pos"))
        if pos in out:
            raise ValueError("choice exposure repeats a water source")
        out.append(pos)
    return out


def folds_to(origin: tuple[int, int], directions: list[str]) -> tuple[int, int]:
    x, y = origin
    for direction in directions:
        if direction not in DIRECTIONS:
            raise ValueError(f"unrecognized movement direction: {direction}")
        dx, dy = DIRECTIONS[direction]
        x += dx
        y += dy
    return x, y


def find_focal_decision(row: dict[str, Any]) -> dict[str, Any] | None:
    decisions = row.get("decisions")
    if not isinstance(decisions, list):
        raise ValueError("tick decisions must be an array")
    matches = [decision for decision in decisions if decision.get("id") == FOCAL_ID]
    if len(matches) > 1:
        raise ValueError("more than one focal DecisionTrace in a tick")
    return matches[0] if matches else None


def event_items(row: dict[str, Any]):
    events = row.get("events")
    if not isinstance(events, list):
        raise ValueError("tick events must be an array")
    for event in events:
        if not isinstance(event, dict) or len(event) != 1:
            raise ValueError("invalid serialized SimulationEvent")
        yield next(iter(event.items()))


def state_entities(state: dict[str, Any]) -> list[dict[str, Any]]:
    entities = state.get("entities")
    if not isinstance(entities, list):
        raise ValueError("serialized world State has no entity array")
    ids = [entity.get("id") for entity in entities]
    if any(type(actor_id) is not int for actor_id in ids) or len(ids) != len(set(ids)):
        raise ValueError("world State has invalid or duplicate actor IDs")
    if not set(ids).issubset({FOCAL_ID, COMPETITOR_ID}):
        raise ValueError(f"unexpected actor creation in lab State: {ids}")
    return entities


def phase_kind(entity: dict[str, Any]) -> str:
    phase = entity.get("phase")
    if isinstance(phase, str):
        return phase
    if isinstance(phase, dict) and len(phase) == 1:
        return next(iter(phase))
    return "Unknown"


def check_actor_lifecycle(before: dict[str, Any], after: dict[str, Any], tick: int,
                         label: str, prior_death_events: set[int], events: list[tuple[str, dict[str, Any]]]) -> None:
    before_entities = {entity["id"]: entity for entity in state_entities(before)}
    after_entities = {entity["id"]: entity for entity in state_entities(after)}
    missing = set(before_entities) - set(after_entities)
    new = set(after_entities) - set(before_entities)
    if new:
        raise ValueError(f"{label} tick {tick}: actor created or replaced: {sorted(new)}")
    if FOCAL_ID in missing or missing - {COMPETITOR_ID}:
        raise ValueError(f"{label} tick {tick}: focal/unknown actor disappeared")
    death_events = prior_death_events | {
        payload["id"] for kind, payload in events
        if kind == "EntityDied" and type(payload.get("id")) is int
    }
    for actor_id in missing:
        old = before_entities[actor_id]
        if not competitor_removal_allowed(actor_id, old, death_events):
            raise ValueError(f"{label} tick {tick}: competitor disappeared without a prior recorded natural death")
    for actor_id, old in before_entities.items():
        if actor_id in missing:
            continue
        new = after_entities[actor_id]
        if old.get("species") != new.get("species") or old.get("generation") != new.get("generation"):
            raise ValueError(f"{label} tick {tick}: actor identity or species changed")
        if old.get("controller") != new.get("controller") or old.get("genome") != new.get("genome"):
            raise ValueError(f"{label} tick {tick}: controller/genome changed during an individual lifetime")
        for key in ("max_meal", "move_energy_cost", "max_stamina", "vision_metabolic_cost",
                    "stamina_metabolic_cost", "max_age", "max_health", "max_energy",
                    "max_water", "maturity_age", "vision_range", "reproduction_interval",
                    "reproduction_reserve_per_mille", "speed"):
            if life(old).get(key) != life(new).get(key):
                raise ValueError(f"{label} tick {tick}: static body field {key} changed")
        if old.get("controller") != new.get("controller") or old.get("genome") != new.get("genome"):
            raise ValueError(f"{label} tick {tick}: controller/genome changed during an individual lifetime")
        for key in ("max_meal", "move_energy_cost", "max_stamina", "vision_metabolic_cost",
                    "stamina_metabolic_cost", "max_age", "max_health", "max_energy",
                    "max_water", "maturity_age", "vision_range", "reproduction_interval",
                    "reproduction_reserve_per_mille", "speed"):
            if life(old).get(key) != life(new).get(key):
                raise ValueError(f"{label} tick {tick}: static body field {key} changed")
        old_age, new_age = life(old).get("age"), life(new).get("age")
        if type(old_age) is not int or type(new_age) is not int or new_age < old_age:
            raise ValueError(f"{label} tick {tick}: invalid or decreasing age")
        if phase_kind(old) != "Corpse" and life(old).get("health", 0) > 0 and new_age != old_age + 1:
            raise ValueError(f"{label} tick {tick}: living actor age did not advance exactly once")
        if phase_kind(old) == "Corpse" and new_age != old_age:
            raise ValueError(f"{label} tick {tick}: dead actor age changed")


def assert_tick_row(row: Any, expected_tick: int, previous_state: dict[str, Any], label: str,
                    death_seen: set[int] | None = None) -> dict[str, Any]:
    if not isinstance(row, dict) or row.get("tick") != expected_tick:
        raise ValueError(f"{label}: expected contiguous tick {expected_tick}")
    before, after = row.get("state_before"), row.get("state_after")
    if not isinstance(before, dict) or not isinstance(after, dict):
        raise ValueError(f"{label} tick {expected_tick}: missing full pre/post state")
    if before != previous_state:
        raise ValueError(f"{label} tick {expected_tick}: pre-state does not match prior state")
    if "rng_before" not in row or "rng_after" not in row:
        raise ValueError(f"{label} tick {expected_tick}: missing RNG capture")
    if before.get("rng") != row["rng_before"] or after.get("rng") != row["rng_after"]:
        raise ValueError(f"{label} tick {expected_tick}: separate RNG capture differs from State")
    if before.get("tick") != expected_tick - 1 or after.get("tick") != expected_tick:
        raise ValueError(f"{label} tick {expected_tick}: serialized State tick mismatch")
    events = list(event_items(row))
    if death_seen is None:
        death_seen = set()
    check_actor_lifecycle(before, after, expected_tick, label, death_seen, events)
    death_seen.update(payload["id"] for kind, payload in events
                      if kind == "EntityDied" and type(payload.get("id")) is int)
    trace = find_focal_decision(row)
    choice = row.get("focal_choice")
    if (trace is None) != (choice is None):
        raise ValueError(f"{label} tick {expected_tick}: choice/DecisionTrace presence mismatch")
    if trace is not None:
        validate_choice_exposure(choice, trace, before, label, expected_tick)
    return after


def normalize_learner(state: dict[str, Any], *, enabled: bool = True,
                      update_enabled: bool = True, clear_values: bool = True) -> dict[str, Any]:
    normalized = deepcopy(state)
    target = learner(normalized)
    target["enabled"] = enabled
    target["update_enabled"] = update_enabled
    if clear_values:
        target["values"] = []
    return normalized


def treat_state(checkpoint: dict[str, Any], branch: str) -> dict[str, Any]:
    if branch not in TREATMENTS:
        raise ValueError(f"unregistered learner branch: {branch}")
    enabled, updates, values = TREATMENTS[branch]
    treated = deepcopy(checkpoint)
    target = learner(treated)
    target["enabled"] = enabled
    target["update_enabled"] = updates
    if values == "clear":
        target["values"] = []
    return treated


def reverse_rates(state: dict[str, Any]) -> dict[str, Any]:
    reversed_state = deepcopy(state)
    environment = reversed_state.get("environment")
    sites = environment.get("sites") if isinstance(environment, dict) else None
    if not isinstance(sites, list):
        raise ValueError("State has no environment site schedule")
    wanted = {WATER_POSITIONS[0]: (1, 4), WATER_POSITIONS[1]: (4, 2)}
    found = set()
    for site in sites:
        try:
            pos = pos_tuple(site.get("pos"))
        except ValueError:
            continue
        if pos in wanted and site.get("kind") == "Water":
            site["replenish"], site["interval"] = wanted[pos]
            found.add(pos)
    if found != set(WATER_POSITIONS):
        raise ValueError("State is missing one or both registered water sites")
    return reversed_state


def expected_initial_rng(seed: int) -> int:
    """Match the RNG calls made while populating the registered 5x5 map.

    World::with_seed populates its empty map before the lab example replaces
    the map contents. The population pass makes one draw per cell and two
    extra draws when the 5% spawn roll succeeds.
    """
    state = seed
    mask = (1 << 64) - 1
    for _ in range(25):
        roll = state % 100  # Random::next_u64 returns the previous state.
        state = (state * 6364136223846793005 + 1) & mask
        if roll < 5:
            state = (state * 6364136223846793005 + 1) & mask
            state = (state * 6364136223846793005 + 1) & mask
    return state


def validate_initial_fixture(initial: dict[str, Any], seed: int, label: str) -> None:
    if initial.get("tick") != 0:
        raise ValueError(f"{label}: initial State tick is not zero")
    if initial.get("rng") != expected_initial_rng(seed):
        raise ValueError(f"{label}: initial RNG does not match seeded map-population draws")
    if initial.get("bounds") != [0, 4, 0, 4]:
        raise ValueError(f"{label}: initial world bounds differ from the registered fixture")
    entities = {entity["id"]: entity for entity in state_entities(initial)}
    if set(entities) != {FOCAL_ID, COMPETITOR_ID}:
        raise ValueError(f"{label}: initial world must contain exactly the two registered actors")
    expected_positions = {FOCAL_ID: (2, 2), COMPETITOR_ID: (2, 3)}
    for actor_id, pos in expected_positions.items():
        subject = entities[actor_id]
        if subject.get("species") != "Herbivore" or subject.get("controller") != "AI":
            raise ValueError(f"{label}: actor {actor_id} does not match fixture species/controller")
        if actor_position(subject) != pos or subject.get("phase") != "Active":
            raise ValueError(f"{label}: actor {actor_id} initial position/phase mismatch")
        subject_life = life(subject)
        if subject_life.get("age") != 0 or subject_life.get("max_age") != 512 or subject_life.get("maturity_age") != 501:
            raise ValueError(f"{label}: actor {actor_id} initial age/lifespan mismatch")
        expected_body = {
            "max_water": 60, "water": 60, "max_energy": 80, "energy": 80,
            "max_health": 120, "health": 120, "speed": 3,
            "move_energy_cost": 1, "max_stamina": 16, "stamina": 16,
            "vision_range": 6,
        }
        if any(subject_life.get(field) != value for field, value in expected_body.items()):
            raise ValueError(f"{label}: actor {actor_id} critical initial body fields differ from ordinary herbivore defaults")
        flags = (learner(initial, actor_id).get("enabled"), learner(initial, actor_id).get("update_enabled"))
        expected_flags = (True, True) if actor_id == FOCAL_ID else (False, False)
        if flags != expected_flags:
            raise ValueError(f"{label}: actor {actor_id} learner flags mismatch")
        initial_learner = learner(initial, actor_id)
        if initial_learner.get("values") != [] or initial_learner.get("episode") is not None or initial_learner.get("last_episode") is not None:
            raise ValueError(f"{label}: actor {actor_id} water learner has pre-seeded history")
    sites = initial.get("environment", {}).get("sites", [])
    if len(sites) != 6:
        raise ValueError(f"{label}: fixture must contain exactly six renewal sites")
    waters, foods = {}, {}
    for site in sites:
        if site.get("kind") == "Water":
            waters[pos_tuple(site["pos"])] = site
        elif site.get("kind") == "Food":
            foods[pos_tuple(site["pos"])] = site
    if set(waters) != set(WATER_POSITIONS):
        raise ValueError(f"{label}: initial registered water sites do not match fixture")
    food_positions = {(0, 0), (4, 0), (0, 4), (4, 4)}
    if set(foods) != food_positions:
        raise ValueError(f"{label}: initial renewable food sites do not match fixture")
    expected_water = {WATER_POSITIONS[0]: (12, 12, 4, 2), WATER_POSITIONS[1]: (12, 12, 1, 4)}
    for pos, expected in expected_water.items():
        site = waters[pos]
        if (initial_resource(initial, pos, "Water"), site.get("capacity"), site.get("replenish"), site.get("interval"), site.get("schedule")) != (*expected, None):
            raise ValueError(f"{label}: initial water amount/capacity/renewal mismatch at {pos}")
    for pos, site in foods.items():
        if (initial_resource(initial, pos, "Food"), site.get("capacity"), site.get("replenish"), site.get("interval"), site.get("schedule")) != (20, 20, 2, 2, None):
            raise ValueError(f"{label}: initial food amount/capacity/renewal mismatch at {pos}")


def initial_resource(state: dict[str, Any], pos: tuple[int, int], kind: str) -> int | None:
    for entry in state.get("resources", []):
        if isinstance(entry, list) and len(entry) == 2 and pos_tuple(entry[0]) == pos:
            payload = entry[1]
            if isinstance(payload, dict) and isinstance(payload.get(kind), dict):
                return payload[kind].get("amount")
    return None


def capacity_map(state: dict[str, Any]) -> dict[tuple[int, int], int]:
    sites = state.get("environment", {}).get("sites", [])
    return {
        pos_tuple(site["pos"]): site["capacity"]
        for site in sites if site.get("kind") == "Water"
    }


def reason_and_intent(trace: dict[str, Any]) -> tuple[str, Any]:
    decision = trace.get("decision")
    if not isinstance(decision, dict):
        raise ValueError("DecisionTrace has no decision")
    return decision.get("reason"), decision.get("intent")


def trace_water_candidates(trace: dict[str, Any]) -> list[dict[str, Any]]:
    perception = trace.get("perception")
    waters = perception.get("waters") if isinstance(perception, dict) else None
    if not isinstance(waters, list):
        return []
    origin = actor_position(trace)
    out = []
    for water in waters:
        steps = water.get("steps")
        if not isinstance(steps, list):
            raise ValueError("perceived water path is not a direction array")
        out.append({
            "pos": folds_to(origin, steps),
            "amount": water.get("amount"),
            "steps_len": len(steps),
        })
    return out


def validate_choice_exposure(choice: Any, trace: dict[str, Any], before: dict[str, Any], label: str, tick: int) -> None:
    if not isinstance(choice, dict):
        raise ValueError(f"{label} tick {tick}: focal choice exposure must be an object")
    final_reason, intent = reason_and_intent(trace)
    if choice.get("final_reason") != final_reason or choice.get("intent") != intent:
        raise ValueError(f"{label} tick {tick}: choice reason/intent disagrees with trace")
    base_reason = choice.get("base_reason")
    if final_reason == "LearnedWater" and base_reason != "SeekWater":
        raise ValueError(f"{label} tick {tick}: LearnedWater was not mapped to SeekWater")
    if base_reason == "SeekWater" and final_reason not in ("SeekWater", "LearnedWater"):
        raise ValueError(f"{label} tick {tick}: unexpected final reason for base SeekWater")
    raw_candidates = choice.get("candidates")
    if not isinstance(raw_candidates, list):
        raise ValueError(f"{label} tick {tick}: choice candidates are not an array")
    expected = trace_water_candidates(trace)
    if len(raw_candidates) != len(expected):
        raise ValueError(f"{label} tick {tick}: choice candidate count differs from perception")
    values = {
        pos_tuple(value["pos"]): value.get("value")
        for value in learner(before).get("values", [])
    }
    actual_positions = []
    for actual, observed in zip(raw_candidates, expected):
        pos = pos_tuple(actual.get("pos"))
        if pos != observed["pos"] or actual.get("amount") != observed["amount"] or actual.get("steps_len") != observed["steps_len"]:
            raise ValueError(f"{label} tick {tick}: candidate exposure differs from ordinary perception")
        if type(actual.get("amount")) is not int or actual["amount"] < 0 or type(actual.get("steps_len")) is not int:
            raise ValueError(f"{label} tick {tick}: invalid candidate amount/path length")
        actual_positions.append(pos)
        saved = values.get(pos)
        if actual.get("saved_value") != saved:
            raise ValueError(f"{label} tick {tick}: saved candidate value differs from pre-choice table")
    if len(actual_positions) != len(set(actual_positions)):
        raise ValueError(f"{label} tick {tick}: duplicate water candidate positions")
    episode = learner(before).get("episode")
    if choice.get("active_episode_before") != episode:
        raise ValueError(f"{label} tick {tick}: active learner episode differs from pre-choice State")
    expected_fresh = fresh_choice_expected(choice, learner(before).get("enabled") is True, final_reason)
    positive_positions = [pos_tuple(item["pos"]) for item in raw_candidates if item.get("amount", 0) > 0]
    candidate_has_active = episode is not None and pos_tuple(episode["target"]) in positive_positions
    ongoing = learner(before).get("enabled") is True and final_reason == "LearnedWater" and candidate_has_active
    if type(choice.get("fresh")) is not bool or choice["fresh"] != expected_fresh:
        raise ValueError(f"{label} tick {tick}: fresh-choice classification mismatch")
    selection_kind = choice.get("selection_kind")
    if selection_kind not in ("ongoing", "greedy", "explore", "base", "none", "unknown"):
        raise ValueError(f"{label} tick {tick}: invalid selection_kind")
    if ongoing and selection_kind != "ongoing":
        raise ValueError(f"{label} tick {tick}: active perceived water target is not classified ongoing")
    if selection_kind == "unknown":
        return
    selected = choice.get("selected_target")
    intent_value = intent if isinstance(intent, dict) else {}
    drink = intent_value.get("Drink") if isinstance(intent_value, dict) else None
    expected_target = None
    if isinstance(drink, dict) and isinstance(drink.get("at"), list):
        expected_target = folds_to(actor_position(trace), drink["at"])
    if (None if selected is None else pos_tuple(selected)) != expected_target:
        raise ValueError(f"{label} tick {tick}: selected target differs from captured Drink intent")
    if selected is not None and pos_tuple(selected) not in actual_positions:
        raise ValueError(f"{label} tick {tick}: selected target was not an exposed candidate")


def fresh_choice_expected(choice: dict[str, Any], learner_enabled: bool, final_reason: str) -> bool:
    """Apply the preregistered fresh rule to captured evidence, without policy execution."""
    if choice.get("base_reason") != "SeekWater":
        return False
    candidates = choice.get("candidates", [])
    positive = [candidate for candidate in candidates if candidate.get("amount", 0) > 0]
    if len(positive) < 2:
        return False
    episode = choice.get("active_episode_before")
    candidate_positions = {pos_tuple(candidate["pos"]) for candidate in positive}
    ongoing = (
        learner_enabled
        and final_reason == "LearnedWater"
        and episode is not None
        and pos_tuple(episode["target"]) in candidate_positions
    )
    return not ongoing


def validate_initial_state_pair(base: dict[str, Any], actual: dict[str, Any], branch: str, phase: str) -> None:
    expected = treat_state(base, branch)
    if actual != expected:
        raise ValueError(f"{phase}/{branch}: treatment changed unregistered checkpoint fields")


def is_alive(state: dict[str, Any], actor_id: int) -> bool:
    entity = actor(state, actor_id)
    return entity is not None and life(entity).get("health", 0) > 0 and phase_kind(entity) not in ("Corpse", "Removed")


def validate_seed(record: Any, expected_seed: int, phase: str) -> dict[str, Any]:
    label = f"{phase} seed {expected_seed}"
    if not isinstance(record, dict):
        raise ValueError(f"{label}: seed envelope is not an object")
    if record.get("schema") != SEED_SCHEMA or record.get("protocol_version") != PROTOCOL_VERSION:
        raise ValueError(f"{label}: seed schema/protocol version mismatch")
    if record.get("role") != phase or record.get("seed") != expected_seed:
        raise ValueError(f"{label}: role/seed mismatch")
    status = record.get("status")
    if status not in ("complete", "invalid_checkpoint_actor_dead"):
        raise ValueError(f"{label}: invalid probe status {status!r}")
    initial = record.get("initial_state")
    if not isinstance(initial, dict):
        raise ValueError(f"{label}: missing initial full State")
    validate_initial_fixture(initial, expected_seed, label)
    prefix = record.get("prefix")
    if not isinstance(prefix, list) or not 0 < len(prefix) <= FORK_TICK:
        raise ValueError(f"{label}: prefix must contain the initial life through tick 200")
    previous = initial
    death_seen: set[int] = set()
    for tick, row in enumerate(prefix, 1):
        previous = assert_tick_row(row, tick, previous, f"{label} prefix", death_seen)
    checkpoint = record.get("checkpoint_state")
    if not isinstance(checkpoint, dict) or checkpoint != previous:
        raise ValueError(f"{label}: checkpoint does not equal the final recorded prefix State")
    checkpoint_entities = {entity["id"]: entity for entity in state_entities(checkpoint)}
    can_fork = all(is_alive(checkpoint, actor_id) for actor_id in (FOCAL_ID, COMPETITOR_ID))
    if can_fork and len(prefix) != FORK_TICK:
        raise ValueError(f"{label}: a live checkpoint must follow exactly 200 prefix ticks")
    if not can_fork and len(prefix) > FORK_TICK:
        raise ValueError(f"{label}: invalid-checkpoint prefix exceeded tick 200")
    if status == "complete" and not can_fork:
        raise ValueError(f"{label}: marked complete though a checkpoint actor is dead")
    if status == "invalid_checkpoint_actor_dead" and can_fork:
        raise ValueError(f"{label}: invalid checkpoint status though both actors are alive")
    forks = record.get("forks")
    if not isinstance(forks, dict) or set(forks) != set(TREATMENTS):
        raise ValueError(f"{label}: fork set mismatch")
    branch_names = set(TREATMENTS)
    if not can_fork:
        if any(forks[name] is not None for name in branch_names):
            raise ValueError(f"{label}: dead checkpoint must not create counterfactual suffixes")
        if record.get("physical_ticks") != len(prefix):
            raise ValueError(f"{label}: invalid-checkpoint physical tick count mismatch")
        return summarize_seed(record, {}, prefix)
    if record.get("physical_ticks") != FORK_TICK + sum(len(forks[name].get("ticks", [])) for name in branch_names if isinstance(forks[name], dict)):
        raise ValueError(f"{label}: physical tick accounting mismatch")
    if record["physical_ticks"] > 1400:
        raise ValueError(f"{label}: physical tick ceiling exceeded")
    for branch_name in TREATMENTS:
        branch = forks.get(branch_name)
        if not isinstance(branch, dict):
            raise ValueError(f"{label}/{branch_name}: missing branch record")
        if branch.get("state_before_treatment") != checkpoint:
            raise ValueError(f"{label}/{branch_name}: branch treatment does not start from exact checkpoint")
        treated = branch.get("state_after_treatment_before_reversal")
        if not isinstance(treated, dict):
            raise ValueError(f"{label}/{branch_name}: missing post-treatment State")
        if branch.get("learner_treatment") != TREATMENT_DESCRIPTIONS[branch_name]:
            raise ValueError(f"{label}/{branch_name}: learner treatment descriptor mismatch")
        validate_initial_state_pair(checkpoint, treated, branch_name, label)
        at_201 = branch.get("state_at_tick_201_before_world_tick")
        if not isinstance(at_201, dict) or at_201 != reverse_rates(treated):
            raise ValueError(f"{label}/{branch_name}: tick-201 State includes an unregistered reversal change")
        rows = branch.get("ticks")
        if not isinstance(rows, list) or len(rows) > FINAL_TICK - FORK_TICK:
            raise ValueError(f"{label}/{branch_name}: invalid suffix timeline/cap")
        if rows and rows[0].get("tick") != FORK_TICK + 1:
            raise ValueError(f"{label}/{branch_name}: suffix does not begin at tick 201")
        previous = at_201
        death_seen = set()
        for expected_tick, row in enumerate(rows, FORK_TICK + 1):
            previous = assert_tick_row(row, expected_tick, previous, f"{label}/{branch_name}", death_seen)
        if rows and rows[-1]["tick"] != FORK_TICK + len(rows):
            raise ValueError(f"{label}/{branch_name}: suffix tick accounting mismatch")
        if branch.get("status") not in ("ceiling", "focal_died"):
            raise ValueError(f"{label}/{branch_name}: invalid branch terminal status")
        focal_alive = is_alive(previous, FOCAL_ID)
        if branch.get("status") == "ceiling" and (len(rows) != 300 or not focal_alive):
            raise ValueError(f"{label}/{branch_name}: ceiling status must reach tick 500 alive")
        if branch.get("status") == "focal_died" and (not rows or focal_alive):
            raise ValueError(f"{label}/{branch_name}: focal death status lacks a dead terminal actor")
    return summarize_seed(record, forks, prefix)


def current_water_values(state: dict[str, Any]) -> dict[tuple[int, int], int]:
    return {pos_tuple(item["pos"]): item.get("value", 200) for item in learner(state).get("values", [])}


def greedy_target(choice: dict[str, Any], values: dict[tuple[int, int], int]) -> tuple[int, int] | None:
    candidates = choice.get("candidates", [])
    if len(candidates) < 2:
        return None
    ranked = []
    for index, candidate in enumerate(candidates):
        pos = pos_tuple(candidate["pos"])
        saved = values.get(pos, 200)
        ranked.append(((saved, -candidate["steps_len"], -index), pos))
    return max(ranked, key=lambda item: item[0])[1]


def state_water_amount(state: dict[str, Any], pos: tuple[int, int]) -> int | None:
    for item in state.get("resources", []):
        if isinstance(item, list) and len(item) == 2 and pos_tuple(item[0]) == pos:
            payload = item[1]
            if isinstance(payload, dict) and isinstance(payload.get("Water"), dict):
                return payload["Water"].get("amount")
    return None


def row_water_events(row: dict[str, Any], actor_id: int = FOCAL_ID) -> list[dict[str, Any]]:
    result = []
    for kind, payload in event_items(row):
        if kind == "EntityDrank" and payload.get("id") == actor_id:
            result.append(payload)
    return result


def row_movement_energy(row: dict[str, Any], actor_id: int = FOCAL_ID) -> int:
    return sum(payload.get("energy_cost", 0) for kind, payload in event_items(row)
               if kind == "EntityMoved" and payload.get("id") == actor_id)


def compare_physical_totals(online_water: int, control_water: int,
                            online_movement: int, control_movement: int) -> dict[str, Any]:
    water_delta = online_water - control_water
    movement_delta = online_movement - control_movement
    return {
        "water_amount_delta": water_delta,
        "movement_energy_paid_delta": movement_delta,
        "physical_outcome_difference": water_delta != 0 or movement_delta != 0,
        "position_or_timing_difference_alone_counts": False,
    }


def choice_counts(rows: list[dict[str, Any]], state_for_capacity: bool = True) -> dict[str, Any]:
    counts = {"fresh_two_source_choices": 0, "nonfull_two_source_choices": 0,
              "selected_site_counts": {"A": 0, "B": 0, "other": 0},
              "amount_exposures": {"A": [], "B": []}, "unknown_selection_kind": 0}
    for row in rows:
        choice = row.get("focal_choice")
        if not isinstance(choice, dict) or not choice.get("fresh"):
            continue
        candidates = choice.get("candidates", [])
        if (choice.get("base_reason") != "SeekWater" or len(candidates) != 2
                or {pos_tuple(c["pos"]) for c in candidates} != set(WATER_POSITIONS)
                or any(c.get("amount", 0) <= 0 for c in candidates)):
            continue
        counts["fresh_two_source_choices"] += 1
        capacity = capacity_map(row["state_before"]) if state_for_capacity else {}
        if any(candidate["amount"] < capacity.get(pos_tuple(candidate["pos"]), 0) for candidate in candidates):
            counts["nonfull_two_source_choices"] += 1
        for candidate in candidates:
            pos = pos_tuple(candidate["pos"])
            name = "A" if pos == WATER_POSITIONS[0] else "B" if pos == WATER_POSITIONS[1] else None
            if name:
                counts["amount_exposures"][name].append(candidate["amount"])
        target = choice.get("selected_target")
        if target is not None:
            pos = pos_tuple(target)
            site = "A" if pos == WATER_POSITIONS[0] else "B" if pos == WATER_POSITIONS[1] else "other"
            counts["selected_site_counts"][site] += 1
        if choice.get("selection_kind") == "unknown":
            counts["unknown_selection_kind"] += 1
    counts["source_amount_range"] = {
        key: [min(values), max(values)] if values else None
        for key, values in counts.pop("amount_exposures").items()
    }
    return counts


def summarize_rows(rows: list[dict[str, Any]], start_state: dict[str, Any], end_state: dict[str, Any], label: str) -> dict[str, Any]:
    water_events = [payload for row in rows for payload in row_water_events(row)]
    movement = sum(row_movement_energy(row) for row in rows)
    death_events = [payload for row in rows for kind, payload in event_items(row) if kind == "EntityDied" and payload.get("id") == FOCAL_ID]
    end_actor = actor(end_state, FOCAL_ID)
    end_life = life(end_actor)
    competitor_start = actor(start_state, COMPETITOR_ID)
    competitor_end = actor(end_state, COMPETITOR_ID)
    competitor_deaths = [payload for row in rows for kind, payload in event_items(row)
                         if kind == "EntityDied" and payload.get("id") == COMPETITOR_ID]
    competitor_drinks = [payload for row in rows for payload in row_water_events(row, COMPETITOR_ID)]
    competitor_life = life(competitor_end) if competitor_end is not None else None
    counts = choice_counts(rows)
    return {
        "status": label,
        "ticks": len(rows),
        "survival_ticks": len(rows),
        "death_cause": end_life.get("death_cause"),
        "death_event": death_events[-1] if death_events else None,
        **counts,
        "resolver_drinks": len(water_events),
        "water_acquired": sum(event.get("amount", 0) for event in water_events),
        "movement_energy_paid": movement,
        "fork_state": {field: life(actor(start_state, FOCAL_ID)).get(field)
                       for field in ("health", "energy", "water")},
        "terminal_state": {field: end_life.get(field) for field in ("health", "energy", "water")},
        "competitor_context": {
            "status": "removed" if competitor_end is None else ("alive" if is_alive(end_state, COMPETITOR_ID) else "dead"),
            "death_cause": competitor_life.get("death_cause") if competitor_life is not None else (competitor_deaths[-1].get("cause") if competitor_deaths else "unknown_removed"),
            "resolver_drinks": len(competitor_drinks),
            "water_acquired": sum(event.get("amount", 0) for event in competitor_drinks),
            "movement_energy_paid": sum(row_movement_energy(row, COMPETITOR_ID) for row in rows),
            "fork_state": ({field: life(competitor_start).get(field) for field in ("health", "energy", "water")} if competitor_start else None),
            "terminal_state": ({field: competitor_life.get(field) for field in ("health", "energy", "water")} if competitor_life is not None else None),
        },
        "fork_state_sha256": canonical_sha(start_state),
        "terminal_state_sha256": canonical_sha(end_state),
    }


def summarize_seed(record: dict[str, Any], forks: dict[str, Any], prefix: list[dict[str, Any]]) -> dict[str, Any]:
    checkpoint = record.get("checkpoint_state")
    prefix_end = checkpoint or (prefix[-1]["state_after"] if prefix else record["initial_state"])
    prefix_summary = summarize_rows(prefix, record["initial_state"], prefix_end,
                                    "reached_fork" if is_alive(prefix_end, FOCAL_ID) else "focal_died_before_fork")
    branches = {}
    for name in TREATMENTS:
        branch = forks.get(name) if isinstance(forks, dict) else None
        if branch is None:
            branches[name] = None
            continue
        rows = branch["ticks"]
        start = branch["state_at_tick_201_before_world_tick"]
        end = rows[-1]["state_after"] if rows else start
        branches[name] = summarize_rows(rows, start, end, branch["status"])
        branches[name]["learner_treatment"] = branch["learner_treatment"]
        branches[name]["initial_suffix_state_sha256"] = canonical_sha(start)
    return {
        "seed": record["seed"],
        "role": record["role"],
        "status": record["status"],
        "physical_ticks": record["physical_ticks"],
        "prefix": prefix_summary,
        "checkpoint_alive": bool(checkpoint and is_alive(checkpoint, FOCAL_ID) and is_alive(checkpoint, COMPETITOR_ID)),
        "checkpoint_state_sha256": canonical_sha(checkpoint) if checkpoint is not None else None,
        "branches": branches,
    }


def _rows(branch: dict[str, Any] | None) -> list[dict[str, Any]]:
    return branch["ticks"] if isinstance(branch, dict) else []


def normalize_for_paired_choice(state: dict[str, Any]) -> dict[str, Any]:
    normalized = deepcopy(state)
    target = learner(normalized)
    target["values"] = []
    target["enabled"] = True
    target["update_enabled"] = True
    return normalized


def causal_audit(record: dict[str, Any]) -> dict[str, Any]:
    branches = record.get("forks", {})
    online, trained = branches.get("online"), branches.get("trained_frozen")
    if not isinstance(online, dict) or not isinstance(trained, dict):
        return {"paired_check": None, "same_exposure_shadow_check": None,
                "attributable_choice": False, "physical_outcome_difference_after_choice": False,
                "post_choice_physical_comparisons": []}
    online_rows = {row["tick"]: row for row in _rows(online)}
    trained_rows = {row["tick"]: row for row in _rows(trained)}
    paired = None
    for tick in sorted(set(online_rows) & set(trained_rows)):
        left, right = online_rows[tick], trained_rows[tick]
        lc, rc = left.get("focal_choice"), right.get("focal_choice")
        if not (isinstance(lc, dict) and isinstance(rc, dict) and lc.get("fresh") and rc.get("fresh")):
            continue
        if lc.get("selected_target") is None or rc.get("selected_target") is None or lc.get("selected_target") == rc.get("selected_target"):
            continue
        ltrace, rtrace = find_focal_decision(left), find_focal_decision(right)
        same_world = (
            normalize_for_paired_choice(left["state_before"]) == normalize_for_paired_choice(right["state_before"])
            and left.get("rng_before") == right.get("rng_before")
            and ltrace is not None and rtrace is not None
            and ltrace.get("before") == rtrace.get("before")
            and ltrace.get("perception") == rtrace.get("perception")
            and ltrace.get("pos") == rtrace.get("pos")
            and ltrace.get("species") == rtrace.get("species")
            and candidate_positions(lc) == candidate_positions(rc)
            and lc.get("selection_kind") != "unknown"
            and rc.get("selection_kind") != "unknown"
        )
        if same_world:
            paired = {"tick": tick, "selected_online": lc.get("selected_target"),
                      "selected_trained_frozen": rc.get("selected_target"),
                      "same_prechoice_world_rng_and_candidates": True}
            break
    checkpoint_values = current_water_values(record["checkpoint_state"])
    shadow = None
    for row in _rows(online):
        choice = row.get("focal_choice")
        if not isinstance(choice, dict) or not choice.get("fresh") or choice.get("selection_kind") != "greedy":
            continue
        current_values = {
            pos_tuple(candidate["pos"]): 200 if candidate.get("saved_value") is None else candidate["saved_value"]
            for candidate in choice.get("candidates", [])
        }
        online_greedy = greedy_target(choice, current_values)
        trained_greedy = greedy_target(choice, checkpoint_values)
        actual = choice.get("selected_target")
        if online_greedy is not None and trained_greedy is not None and online_greedy != trained_greedy and actual == {"x": online_greedy[0], "y": online_greedy[1]}:
            shadow = {"tick": row["tick"], "actual_selection_kind": "greedy",
                      "selected_online": choice.get("selected_target"),
                      "greedy_current_online_table": {"x": online_greedy[0], "y": online_greedy[1]},
                      "greedy_checkpoint_table": {"x": trained_greedy[0], "y": trained_greedy[1]},
                      "same_exposure": True}
            break
    attribution = paired is not None or shadow is not None
    physical_comparisons = []
    for source_name, source in (("paired_check", paired), ("same_exposure_shadow_check", shadow)):
        if source is None:
            continue
        diverge_tick = source["tick"]
        online_post = [row for row in _rows(online) if row["tick"] >= diverge_tick]
        trained_post = [row for row in _rows(trained) if row["tick"] >= diverge_tick]
        common_ticks = min(len(online_post), len(trained_post))
        online_post, trained_post = online_post[:common_ticks], trained_post[:common_ticks]
        online_water = sum(event.get("amount", 0) for row in online_post for event in row_water_events(row))
        trained_water = sum(event.get("amount", 0) for row in trained_post for event in row_water_events(row))
        online_movement = sum(row_movement_energy(row) for row in online_post)
        trained_movement = sum(row_movement_energy(row) for row in trained_post)
        deltas = compare_physical_totals(online_water, trained_water, online_movement, trained_movement)
        water_delta = deltas["water_amount_delta"]
        movement_delta = deltas["movement_energy_paid_delta"]
        physical_comparisons.append({
            "attribution_check": source_name,
            "first_compared_tick": diverge_tick,
            "matched_ticks": common_ticks,
            "online_water_amount": online_water,
            "trained_frozen_water_amount": trained_water,
            "water_amount_delta": water_delta,
            "online_movement_energy_paid": online_movement,
            "trained_frozen_movement_energy_paid": trained_movement,
            **deltas,
        })
    physical_difference = any(row["physical_outcome_difference"] for row in physical_comparisons)
    return {"paired_check": paired, "same_exposure_shadow_check": shadow,
            "attributable_choice": attribution,
            "physical_outcome_difference_after_choice": physical_difference,
            "post_choice_physical_comparisons": physical_comparisons}


def common_horizon(online: dict[str, Any] | None, control: dict[str, Any] | None) -> dict[str, Any] | None:
    if not isinstance(online, dict) or not isinstance(control, dict):
        return None
    o_rows, c_rows = _rows(online), _rows(control)
    horizon = min(len(o_rows), len(c_rows), FINAL_TICK - FORK_TICK)
    o_slice, c_slice = o_rows[:horizon], c_rows[:horizon]
    o_end = o_slice[-1]["state_after"] if o_slice else online["state_at_tick_201_before_world_tick"]
    c_end = c_slice[-1]["state_after"] if c_slice else control["state_at_tick_201_before_world_tick"]
    o = summarize_rows(o_slice, online["state_at_tick_201_before_world_tick"], o_end, online["status"])
    c = summarize_rows(c_slice, control["state_at_tick_201_before_world_tick"], c_end, control["status"])
    differences = {
        "water_acquired": o["water_acquired"] - c["water_acquired"],
        "movement_energy_paid": o["movement_energy_paid"] - c["movement_energy_paid"],
        "health_at_horizon": o["terminal_state"]["health"] - c["terminal_state"]["health"],
        "energy_at_horizon": o["terminal_state"]["energy"] - c["terminal_state"]["energy"],
    }
    survival_ok = len(o_rows) >= len(c_rows)
    no_negative_component = (
        differences["water_acquired"] >= 0
        and differences["movement_energy_paid"] <= 0
        and differences["health_at_horizon"] >= 0
        and differences["energy_at_horizon"] >= 0
        and survival_ok
    )
    strict_positive_component = (
        differences["water_acquired"] > 0
        or differences["movement_energy_paid"] < 0
        or differences["health_at_horizon"] > 0
        or differences["energy_at_horizon"] > 0
        or len(o_rows) > len(c_rows)
    )
    return {"ticks": horizon, "differences_online_minus_trained_frozen": differences,
            "online_survival_ticks": len(o_rows), "trained_frozen_survival_ticks": len(c_rows),
            "survival_nondeteriorating": survival_ok,
            "no_negative_component": no_negative_component,
            "strict_positive_component": strict_positive_component,
            "net_physical_benefit_vector": no_negative_component and strict_positive_component}


def rank_audit(record: dict[str, Any]) -> dict[str, Any]:
    checkpoint_values = current_water_values(record["checkpoint_state"])
    pre_rows = [row for row in record["prefix"]
                if isinstance(row.get("focal_choice"), dict)
                and row["focal_choice"].get("fresh")
                and len(row["focal_choice"].get("candidates", [])) == 2
                and {pos_tuple(candidate["pos"]) for candidate in row["focal_choice"]["candidates"]} == set(WATER_POSITIONS)
                and all(candidate.get("amount", 0) > 0 for candidate in row["focal_choice"]["candidates"])]
    pre_choices = [row["focal_choice"] for row in pre_rows]
    post_rows = _rows(record.get("forks", {}).get("online"))
    post_choices = [row["focal_choice"] for row in post_rows
                    if isinstance(row.get("focal_choice"), dict)
                    and row["focal_choice"].get("fresh")
                    and len(row["focal_choice"].get("candidates", [])) == 2
                    and {pos_tuple(candidate["pos"]) for candidate in row["focal_choice"]["candidates"]} == set(WATER_POSITIONS)
                    and all(candidate.get("amount", 0) > 0 for candidate in row["focal_choice"]["candidates"])]
    before = None
    if pre_choices:
        before = greedy_target(pre_choices[-1], checkpoint_values)
    reversals = []
    if before is not None:
        for choice in post_choices:
            current = greedy_target(choice, {
                pos_tuple(c["pos"]): 200 if c.get("saved_value") is None else c["saved_value"]
                for c in choice["candidates"]
            })
            if current is not None and current != before:
                reversals.append({"tick": next(row["tick"] for row in post_rows if row.get("focal_choice") is choice),
                                  "pre_reversal_argmax": {"x": before[0], "y": before[1]},
                                  "post_reversal_argmax": {"x": current[0], "y": current[1]}})
                break
    checkpoint_argmax = None
    for choice in pre_choices:
        target = greedy_target(choice, checkpoint_values)
        if target is not None:
            checkpoint_argmax = target
    return {
        "checkpoint_table_values": {f"{x},{y}": value for (x, y), value in sorted(checkpoint_values.items())},
        "last_pre_reversal_exposure_argmax": {"x": before[0], "y": before[1]} if before else None,
        "checkpoint_table_argmax_on_last_pre_reversal_exposure": ({"x": checkpoint_argmax[0], "y": checkpoint_argmax[1]} if checkpoint_argmax else None),
                "post_reversal_argmax_reversals": reversals,
        "reversal_observed": bool(reversals),
    }


def analyze_seed(record: dict[str, Any]) -> dict[str, Any]:
    result = validate_seed(record, record["seed"], record["role"])
    if record["status"] == "complete":
        result["causal_audit"] = causal_audit(record)
        result["rank_audit"] = rank_audit(record)
        result["online_vs_trained_frozen_common_horizon"] = common_horizon(
            record["forks"].get("online"), record["forks"].get("trained_frozen"))
        result["trained_vs_ablated_frozen_common_horizon"] = common_horizon(
            record["forks"].get("trained_frozen"), record["forks"].get("ablated_frozen"))
        causal = result["causal_audit"]
        vector = result["online_vs_trained_frozen_common_horizon"]
        result["existence_hypothesis_conditions_met"] = bool(
            causal["attributable_choice"]
            and causal["physical_outcome_difference_after_choice"]
            and vector and vector["net_physical_benefit_vector"]
        )
    else:
        result.update({"causal_audit": None, "rank_audit": None,
                       "online_vs_trained_frozen_common_horizon": None,
                       "trained_vs_ablated_frozen_common_horizon": None,
                       "existence_hypothesis_conditions_met": False})
    return result


def calibration_gate(seed_summary: dict[str, Any]) -> dict[str, Any]:
    prefix = seed_summary["prefix"]
    online = seed_summary["branches"].get("online")
    suffix_choices = 0 if online is None else online["fresh_two_source_choices"]
    suffix_nonfull = 0 if online is None else online["nonfull_two_source_choices"]
    suffix_drinks = 0 if online is None else online["resolver_drinks"]
    checks = {
        "both_actors_alive_at_fork": seed_summary["checkpoint_alive"],
        "prefix_fresh_two_source_choices_at_least_6": prefix["fresh_two_source_choices"] >= 6,
        "prefix_resolver_drinks_at_least_3": prefix["resolver_drinks"] >= 3,
        "prefix_nonfull_two_source_choices_at_least_3": prefix["nonfull_two_source_choices"] >= 3,
        "online_suffix_fresh_two_source_choices_at_least_6": suffix_choices >= 6,
        "online_suffix_resolver_drinks_at_least_3": suffix_drinks >= 3,
        "online_suffix_nonfull_two_source_choices_at_least_3": suffix_nonfull >= 3,
    }
    return {"passed": all(checks.values()), "checks": checks,
            "observed": {"prefix_fresh_two_source_choices": prefix["fresh_two_source_choices"],
                         "prefix_drinks": prefix["resolver_drinks"],
                         "prefix_nonfull_choices": prefix["nonfull_two_source_choices"],
                         "online_suffix_fresh_two_source_choices": suffix_choices,
                         "online_suffix_drinks": suffix_drinks,
                         "online_suffix_nonfull_choices": suffix_nonfull}}


def render_report(summary: dict[str, Any]) -> str:
    lines = [
        "# Continuous-lifetime water learning — bounded run", "",
        f"Phase: **{summary['role']}**. Status: **{summary['status']}**.",
        "", "No weighted score is used. Tick counts are accelerated simulation time, not real elapsed lifetime.", "",
        "| Seed | Status | Prefix choices/drinks/nonfull | Branch | Suffix ticks | Choices | Drinks | Water | Move energy | Terminal H/E/W |",
        "|---:|---|---:|---|---:|---:|---:|---:|---:|---|",
    ]
    for seed in summary["seeds"]:
        p = seed["prefix"]
        prefix_text = f"{p['fresh_two_source_choices']}/{p['resolver_drinks']}/{p['nonfull_two_source_choices']}"
        if not any(seed["branches"].values()):
            lines.append(f"| {seed['seed']} | {seed['status']} | {prefix_text} | — | — | — | — | — | — | — |")
            continue
        for name, branch in seed["branches"].items():
            if branch is None:
                continue
            terminal = branch["terminal_state"]
            lines.append(
                f"| {seed['seed']} | {seed['status']} | {prefix_text} | {name} | {branch['ticks']} | "
                f"{branch['fresh_two_source_choices']} | {branch['resolver_drinks']} | {branch['water_acquired']} | "
                f"{branch['movement_energy_paid']} | {terminal['health']}/{terminal['energy']}/{terminal['water']} |"
            )
    lines.extend(["", "## Interpretation", ""])
    if summary["role"] == "calibration":
        gate = summary.get("calibration_validity", {})
        lines.append("Calibration fixture gate: **" + ("passed" if gate.get("passed") else "failed / non-identifiable") + "**.")
        if not gate.get("passed"):
            lines.append("The formal phase must not run from this result.")
    else:
        lines.append("An observed choice change alone is not evidence of benefit. Review each seed's causal audit and unweighted common-horizon outcome vector.")
    lines.append("")
    return "\n".join(lines)


def atomic_json(path: Path, value: Any) -> None:
    temporary = path.with_suffix(path.suffix + ".tmp")
    temporary.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    temporary.replace(path)


def require_new_output(path: Path) -> None:
    if path.exists():
        raise FileExistsError(f"refusing to overwrite historical experiment output: {path}")


def terminal_manifest_status(probe_status: str, analysis_status: str) -> str:
    """Gate failure is a completed capture; probe/schema failure is infrastructure failure."""
    if probe_status != "completed" or analysis_status not in ("completed", "non_identifiable"):
        return "failed"
    return "completed"


def competitor_removal_allowed(actor_id: int, entity_before: dict[str, Any], death_seen: set[int]) -> bool:
    return (
        actor_id == COMPETITOR_ID
        and phase_kind(entity_before) in ("Corpse", "Removed")
        and life(entity_before).get("health") == 0
        and actor_id in death_seen
    )


def _child_limits() -> None:
    resource.setrlimit(resource.RLIMIT_AS, (MAX_CHILD_MEMORY_BYTES, MAX_CHILD_MEMORY_BYTES))
    resource.setrlimit(resource.RLIMIT_FSIZE, (MAX_OUTPUT_BYTES, MAX_OUTPUT_BYTES))
    cpu_limit = MAX_SECONDS + 5
    resource.setrlimit(resource.RLIMIT_CPU, (cpu_limit, cpu_limit))


def run_probe(command: list[str], work: Path, raw_path: Path, stderr_path: Path) -> dict[str, Any]:
    start = time.monotonic()
    try:
        with raw_path.open("xb") as stdout, stderr_path.open("xb") as stderr:
            completed = subprocess.run(command, cwd=work, stdout=stdout, stderr=stderr,
                                       timeout=MAX_SECONDS, check=False, preexec_fn=_child_limits)
        raw_bytes = raw_path.stat().st_size
        stderr_bytes = stderr_path.stat().st_size
        output_limit_exceeded = raw_bytes >= MAX_OUTPUT_BYTES
        return {"status": "completed" if completed.returncode == 0 and not output_limit_exceeded else "failed",
                "exit_code": completed.returncode, "raw_bytes": raw_bytes,
                "stderr_bytes": stderr_bytes, "output_limit_exceeded": output_limit_exceeded,
                "runner_error": "probe output reached the registered raw-output cap" if output_limit_exceeded else None,
                "elapsed_seconds": time.monotonic() - start}
    except subprocess.TimeoutExpired as error:
        return {"status": "failed", "exit_code": None, "timed_out": True,
                "elapsed_seconds": time.monotonic() - start, "runner_error": str(error)}
    except Exception as error:
        return {"status": "failed", "exit_code": None,
                "elapsed_seconds": time.monotonic() - start,
                "runner_error": f"{type(error).__name__}: {error}"}


def parse_raw(raw_path: Path, phase: str, seeds: tuple[int, ...],
              progress_path: Path | None = None) -> list[dict[str, Any]]:
    results: list[dict[str, Any]] = []
    with raw_path.open("r", encoding="utf-8") as stream:
        for index, seed in enumerate(seeds, 1):
            line = stream.readline(MAX_SEED_RECORD_BYTES + 1)
            if not line:
                raise ValueError(f"missing seed envelope {index}/{len(seeds)}")
            if len(line.encode("utf-8")) > MAX_SEED_RECORD_BYTES:
                raise ValueError(f"seed envelope {index} exceeds {MAX_SEED_RECORD_BYTES} bytes")
            if not line.endswith("\n"):
                raise ValueError(f"seed envelope {index} is truncated or missing its JSONL newline")
            try:
                record = json.loads(line)
            except json.JSONDecodeError as error:
                raise ValueError(f"malformed JSONL seed envelope {index}: {error}") from error
            results.append(analyze_seed(record))
            del record, line
            if progress_path is not None:
                atomic_json(progress_path, {"schema": SUMMARY_SCHEMA,
                    "protocol_version": PROTOCOL_VERSION, "role": phase,
                    "status": "in_progress", "seeds": results})
        if stream.read(1):
            raise ValueError("raw JSONL contains more records than the preregistered seed list")
    if [row["seed"] for row in results] != list(seeds):
        raise ValueError("raw seed order does not match preregistered order")
    if any(row["role"] != phase for row in results):
        raise ValueError("raw seed roles do not match requested phase")
    return results


def require_passing_reanalysis_gate(reanalysis: dict[str, Any], gate: dict[str, Any]) -> None:
    if not gate.get("passed") or gate != reanalysis.get("calibration_validity"):
        raise RuntimeError("corrected calibration gate does not reproduce from captured raw")
    validation = reanalysis.get("validation_result")
    if not isinstance(validation, dict) or validation.get("passed") is not True:
        raise RuntimeError("corrected calibration reanalysis validation did not pass")
    if validation.get("errors") != []:
        raise RuntimeError("corrected calibration reanalysis records validation errors")


def validate_calibration_parent(archive_root: Path, current: dict[str, Any],
                                source_root: Path = ROOT) -> dict[str, Any]:
    calibration = archive_root / OUTPUT_DIRS["calibration"]
    manifest_path = calibration / "manifest.json"
    if not manifest_path.is_file():
        raise RuntimeError("formal phase requires a canonical calibration run")
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    if manifest.get("schema") != MANIFEST_SCHEMA or manifest.get("protocol_version") != PROTOCOL_VERSION:
        raise RuntimeError("calibration manifest schema mismatch")
    raw_path = calibration / "raw.jsonl"
    if not raw_path.is_file() or manifest.get("raw_sha256") != sha256_file(raw_path):
        raise RuntimeError("calibration raw hash mismatch")

    # A failed capture may be re-analyzed after a validator bug is fixed, but
    # the original capture and probe remain immutable. This path accepts only
    # a root-reviewed candidate whose hashes match the original manifest and
    # whose gate is independently reproduced from the captured raw trace.
    if manifest.get("status") == "failed":
        reanalysis_dir = archive_root / CALIBRATION_REANALYSIS_DIR
        reanalysis_path = reanalysis_dir / "reanalysis.json"
        if not reanalysis_path.is_file():
            raise RuntimeError("failed calibration requires the canonical corrected reanalysis")
        reanalysis = json.loads(reanalysis_path.read_text(encoding="utf-8"))
        if manifest.get("role") != "calibration":
            raise RuntimeError("failed calibration manifest role mismatch")
        reanalysis_identity = validate_calibration_reanalysis(
            archive_root, current, manifest, manifest_path, raw_path, reanalysis, source_root,
        )
        recalculated = parse_raw(raw_path, "calibration", RUNS["calibration"])
        gate = calibration_gate(recalculated[0])
        require_passing_reanalysis_gate(reanalysis, gate)
        summary_path = reanalysis_dir / "summary.json"
        report_path = reanalysis_dir / "report.md"
        if (not summary_path.is_file() or not report_path.is_file()
                or reanalysis.get("summary_sha256") != sha256_file(summary_path)
                or reanalysis.get("report_sha256") != sha256_file(report_path)):
            raise RuntimeError("corrected calibration summary/report hash mismatch")
        summary = json.loads(summary_path.read_text(encoding="utf-8"))
        if (summary.get("schema") != SUMMARY_SCHEMA or summary.get("role") != "calibration"
                or summary.get("calibration_validity") != gate or summary.get("status") != "completed"):
            raise RuntimeError("corrected calibration summary does not reproduce the passing gate")
        return {**reanalysis_identity, "calibration_validity": gate}

    summary_path = calibration / "summary.json"
    if not summary_path.is_file():
        raise RuntimeError("formal phase requires a completed canonical calibration summary")
    summary = json.loads(summary_path.read_text(encoding="utf-8"))
    if manifest.get("status") != "completed" or manifest.get("role") != "calibration":
        raise RuntimeError("formal phase requires calibration manifest status=completed")
    if (manifest.get("analysis_status") != "completed"
            or not manifest.get("calibration_validity", {}).get("passed")):
        raise RuntimeError("calibration manifest does not record a completed passing fixture gate")
    if (summary.get("schema") != SUMMARY_SCHEMA or summary.get("status") != "completed"
            or not summary.get("calibration_validity", {}).get("passed")):
        raise RuntimeError("calibration fixture-validity gate did not pass")
    if (manifest.get("seeds") != list(RUNS["calibration"])
            or manifest.get("calibration_validity") != summary.get("calibration_validity")):
        raise RuntimeError("calibration manifest and summary do not agree on registered seed/gate")
    if manifest.get("summary_sha256") != sha256_file(summary_path):
        raise RuntimeError("calibration summary hash mismatch")
    for field in ("source_sha256", "source_files_sha256", "binary_sha256", "protocol_sha256", "runner_sha256"):
        if manifest.get(field) != current.get(field):
            raise RuntimeError(f"formal candidate differs from calibration in {field}")
    recalculated = parse_raw(raw_path, "calibration", RUNS["calibration"])
    gate = calibration_gate(recalculated[0])
    if not gate["passed"] or gate != summary.get("calibration_validity"):
        raise RuntimeError("calibration gate does not reproduce from the captured raw trace")
    return {"manifest_sha256": sha256_file(manifest_path),
            "revision": manifest["revision"],
            "summary_sha256": sha256_file(summary_path),
            "raw_sha256": sha256_file(raw_path),
            "probe_sha256": manifest["binary_sha256"],
            "protocol_sha256": manifest["protocol_sha256"],
            "source_sha256": manifest["source_sha256"],
            "source_files_sha256": manifest["source_files_sha256"],
            "runner_sha256": manifest["runner_sha256"]}


def biological_source_hashes(files: Any) -> dict[str, str]:
    if not isinstance(files, dict):
        raise RuntimeError("calibration source hash map is malformed")
    selected = {
        path: digest for path, digest in files.items()
        if path in ("Cargo.toml", "Cargo.lock") or
        (path.endswith(".rs") and (path.startswith("src/") or path.startswith("examples/")))
    }
    if "Cargo.toml" not in selected or "Cargo.lock" not in selected or not any(
        path.startswith("src/") for path in selected
    ) or not any(path.startswith("examples/") for path in selected):
        raise RuntimeError("calibration biological source hash map is incomplete")
    return selected


def validate_calibration_reanalysis(archive_root: Path, current: dict[str, Any],
                                    original: dict[str, Any], manifest_path: Path,
                                    raw_path: Path, reanalysis: dict[str, Any],
                                    source_root: Path = ROOT) -> dict[str, Any]:
    if reanalysis.get("schema") != "curious-lifetime-lab-reanalysis-v2":
        raise RuntimeError("calibration reanalysis schema mismatch")
    if (reanalysis.get("status") != "completed" or reanalysis.get("role") != "calibration"
            or reanalysis.get("seeds") != list(RUNS["calibration"])
            or reanalysis.get("ticks_replayed") != 0
            or reanalysis.get("zero_new_ticks") is not True
            or reanalysis.get("probe_launched") is not False):
        raise RuntimeError("calibration reanalysis is incomplete or has an unexpected run scope")
    if reanalysis.get("revision") != current.get("revision"):
        raise RuntimeError("calibration reanalysis revision differs from formal candidate")
    if (reanalysis.get("current_source_sha256") != current.get("source_sha256")
            or reanalysis.get("current_source_files_sha256") != current.get("source_files_sha256")):
        raise RuntimeError("calibration reanalysis source identity differs from formal candidate")
    if reanalysis.get("original_run_dir") != OUTPUT_DIRS["calibration"]:
        raise RuntimeError("calibration reanalysis names a different original run")
    if (original.get("exit_code") != 0 or original.get("timed_out") is True
            or original.get("output_limit_exceeded") is True
            or original.get("raw_bytes") != raw_path.stat().st_size):
        raise RuntimeError("original calibration probe did not complete a bounded successful capture")
    original_dir = archive_root / OUTPUT_DIRS["calibration"]
    probe_path = original_dir / "lab-probe"
    protocol_path = original_dir / "protocol.md"
    if not probe_path.is_file() or sha256_file(probe_path) != original.get("binary_sha256"):
        raise RuntimeError("original calibration probe artifact hash mismatch")
    if not protocol_path.is_file() or sha256_file(protocol_path) != original.get("protocol_sha256"):
        raise RuntimeError("original calibration protocol artifact hash mismatch")
    expected = {
        "original_manifest_sha256": sha256_file(manifest_path),
        "original_raw_sha256": sha256_file(raw_path),
        "original_probe_sha256": original.get("binary_sha256"),
        "protocol_sha256": original.get("protocol_sha256"),
        "original_source_sha256": original.get("source_sha256"),
        "original_source_files_sha256": original.get("source_files_sha256"),
        "original_runner_sha256": original.get("runner_sha256"),
    }
    if original.get("raw_sha256") != expected["original_raw_sha256"]:
        raise RuntimeError("original calibration raw artifact hash mismatch")
    for key, value in expected.items():
        if reanalysis.get(key) != value:
            raise RuntimeError(f"calibration reanalysis provenance mismatch: {key}")
    if (reanalysis.get("original_summary_present") is not False
            or reanalysis.get("original_summary_sha256") is not None):
        raise RuntimeError("calibration reanalysis misstates the absent original summary")
    for key, value in (("binary_sha256", original.get("binary_sha256")),
                       ("protocol_sha256", original.get("protocol_sha256"))):
        if current.get(key) != value:
            raise RuntimeError(f"formal candidate differs from original calibration in {key}")
    original_biology = biological_source_hashes(original.get("source_files_sha256"))
    current_biology = biological_source_hashes(current.get("source_files_sha256"))
    if original_biology != current_biology:
        raise RuntimeError("formal candidate biological source differs from original calibration")
    if reanalysis.get("validator_sha256") != current.get("runner_sha256"):
        raise RuntimeError("calibration reanalysis validator differs from formal candidate")
    reanalysis_protocol_path = source_root / REANALYSIS_PROTOCOL
    if (not reanalysis_protocol_path.is_file()
            or reanalysis.get("reanalysis_protocol_sha256") != sha256_file(reanalysis_protocol_path)):
        raise RuntimeError("calibration reanalysis protocol hash mismatch")
    copied_protocol = archive_root / CALIBRATION_REANALYSIS_DIR / "protocol.md"
    if not copied_protocol.is_file() or sha256_file(copied_protocol) != sha256_file(reanalysis_protocol_path):
        raise RuntimeError("calibration reanalysis protocol copy is missing or changed")
    expected_rng = {str(seed): expected_initial_rng(seed) for seed in RUNS["calibration"]}
    if reanalysis.get("expected_initial_rng_by_seed") != expected_rng:
        raise RuntimeError("calibration reanalysis expected RNG values mismatch")
    return {"manifest_sha256": sha256_file(manifest_path),
            "reanalysis_sha256": sha256_file(archive_root / CALIBRATION_REANALYSIS_DIR / "reanalysis.json"),
            "revision": original["revision"],
            "summary_sha256": reanalysis.get("summary_sha256"),
            "report_sha256": reanalysis.get("report_sha256"),
            "raw_sha256": expected["original_raw_sha256"],
            "probe_sha256": expected["original_probe_sha256"],
            "protocol_sha256": expected["protocol_sha256"],
            "source_sha256": expected["original_source_sha256"],
            "source_files_sha256": expected["original_source_files_sha256"],
            "runner_sha256": current["runner_sha256"]}


def execute_calibration_reanalysis(archive_root: Path, source_root: Path = ROOT) -> dict[str, Any]:
    """Revalidate the immutable failed capture without launching a probe."""
    archive_root = archive_root.resolve()
    source_root = source_root.resolve()
    output = archive_root / CALIBRATION_REANALYSIS_DIR
    require_new_output(output)
    original_dir = archive_root / OUTPUT_DIRS["calibration"]
    manifest_path, raw_path = original_dir / "manifest.json", original_dir / "raw.jsonl"
    probe_path = original_dir / "lab-probe"
    if not all(path.is_file() for path in (manifest_path, raw_path, probe_path)):
        raise RuntimeError("immutable failed calibration is missing its manifest, raw, or probe")
    original = json.loads(manifest_path.read_text(encoding="utf-8"))
    if original.get("schema") != MANIFEST_SCHEMA or original.get("role") != "calibration":
        raise RuntimeError("original calibration manifest schema/role mismatch")
    if original.get("status") != "failed":
        raise RuntimeError("reanalysis is only registered for the immutable failed capture")
    if (original.get("exit_code") != 0 or original.get("timed_out") is True
            or original.get("output_limit_exceeded") is True):
        raise RuntimeError("reanalysis cannot rescue an unsuccessful or incomplete probe capture")
    original_manifest_sha = sha256_file(manifest_path)
    original_raw_sha = sha256_file(raw_path)
    if original.get("raw_sha256") != original_raw_sha:
        raise RuntimeError("original calibration raw hash mismatch")
    original_probe_sha = sha256_file(probe_path)
    if original.get("binary_sha256") != original_probe_sha:
        raise RuntimeError("original calibration probe hash mismatch")
    revision, hashes = source_identity(source_root)
    protocol_path = source_root / PROTOCOL
    protocol_sha = sha256_file(protocol_path)
    if original.get("protocol_sha256") != protocol_sha:
        raise RuntimeError("candidate frozen protocol differs from original calibration")
    if sha256_file(original_dir / "protocol.md") != original.get("protocol_sha256"):
        raise RuntimeError("original calibration protocol artifact hash mismatch")
    original_bio = biological_source_hashes(original.get("source_files_sha256"))
    candidate_bio = biological_source_hashes(hashes)
    if original_bio != candidate_bio:
        raise RuntimeError("candidate biological source differs from original calibration")
    current_runner_sha = sha256_file(Path(__file__).resolve())
    if current_runner_sha != hashes.get(str(Path(__file__).resolve().relative_to(source_root))):
        raise RuntimeError("reanalysis runner differs from its clean source identity")
    if original.get("summary_sha256") is not None or (original_dir / "summary.json").exists():
        raise RuntimeError("expected the original failed capture to have no summary")

    errors: list[str] = []
    gate: dict[str, Any] | None = None
    physical_ticks = 0
    records: list[dict[str, Any]] = []
    try:
        records = parse_raw(raw_path, "calibration", RUNS["calibration"])
        physical_ticks = sum(record["physical_ticks"] for record in records)
        gate = calibration_gate(records[0])
        if not gate.get("passed"):
            errors.append("calibration fixture validity gate did not pass")
    except Exception as error:
        errors.append(f"{type(error).__name__}: {error}")
    analysis_protocol = source_root / REANALYSIS_PROTOCOL
    if not analysis_protocol.is_file():
        raise RuntimeError("registered calibration reanalysis protocol is missing")
    protocol_reanalysis_sha = sha256_file(analysis_protocol)
    result = {
        "schema": "curious-lifetime-lab-reanalysis-v2",
        "status": "completed" if gate is not None else "failed",
        "original_run_dir": OUTPUT_DIRS["calibration"],
        "original_manifest_sha256": original_manifest_sha,
        "original_raw_sha256": original_raw_sha,
        "original_probe_sha256": original_probe_sha,
        "protocol_sha256": protocol_sha,
        "original_source_sha256": original.get("source_sha256"),
        "original_source_files_sha256": original.get("source_files_sha256"),
        "original_runner_sha256": original.get("runner_sha256"),
        "original_summary_present": False,
        "original_summary_sha256": None,
        "revision": revision,
        "current_source_sha256": source_map_sha256(hashes),
        "current_source_files_sha256": hashes,
        "role": "calibration",
        "seeds": list(RUNS["calibration"]),
        "ticks_replayed": 0,
        "zero_new_ticks": True,
        "probe_launched": False,
        "physical_ticks_validated": physical_ticks,
        "expected_initial_rng_by_seed": {
            str(seed): expected_initial_rng(seed) for seed in RUNS["calibration"]
        },
        "validator_sha256": current_runner_sha,
        "reanalysis_protocol_sha256": protocol_reanalysis_sha,
        "calibration_validity": gate,
        "validation_result": {
            "passed": gate is not None and gate.get("passed") is True and not errors,
            "errors": errors,
            "checks": {"raw_sha256_verified": True,
                       "probe_sha256_verified": True,
                       "frozen_protocol_sha256_verified": True,
                       "biological_source_hashes_verified": True,
                       "physical_ticks": physical_ticks},
        },
        "created_at": now(),
    }
    output.mkdir(parents=True, exist_ok=False)
    shutil.copyfile(analysis_protocol, output / "protocol.md")
    if records:
        seed_audit = original.get("seed_audit", {"seeds": list(RUNS["calibration"])})
        probe_run = {key: original[key] for key in (
            "exit_code", "elapsed_seconds", "raw_bytes", "stderr_bytes", "status",
            "timed_out", "output_limit_exceeded", "max_output_bytes",
            "max_seed_record_bytes", "max_child_memory_bytes", "timeout_seconds",
        ) if key in original}
        probe_run["status"] = "completed"
        summary = write_summary_and_report(output, "calibration", records, seed_audit, probe_run)
        result["summary_sha256"] = summary["summary_sha256"]
        result["report_sha256"] = summary["report_sha256"]
    atomic_json(output / "reanalysis.json", result)
    return result


def write_summary_and_report(outdir: Path, phase: str, records: list[dict[str, Any]],
                             seed_audit: dict[str, Any], probe_run: dict[str, Any]) -> dict[str, Any]:
    validity = calibration_gate(records[0]) if phase == "calibration" else None
    analysis_status = "non_identifiable" if phase == "calibration" and not validity["passed"] else "completed"
    result = {
        "schema": SUMMARY_SCHEMA, "protocol_version": PROTOCOL_VERSION,
        "role": phase, "status": analysis_status, "seeds": records,
        "physical_ticks": sum(row["physical_ticks"] for row in records),
        "tick_ceiling": 1400 if phase == "calibration" else 11200,
        "seed_audit": seed_audit, "probe_run": probe_run,
    }
    if validity is not None:
        result["calibration_validity"] = validity
    if phase == "formal":
        result["existence_example_seeds"] = [row["seed"] for row in records if row.get("existence_hypothesis_conditions_met")]
        result["interpretation"] = "Individual existence evidence only; no population, evolutionary, or ecosystem inference."
    else:
        result["interpretation"] = "Fixture validity only. A passed gate permits root review; it is not a learning benefit result."
    atomic_json(outdir / "summary.json", result)
    (outdir / "report.md").write_text(render_report(result), encoding="utf-8")
    (outdir / "summary.partial.json").unlink(missing_ok=True)
    result["summary_sha256"] = sha256_file(outdir / "summary.json")
    result["report_sha256"] = sha256_file(outdir / "report.md")
    return result


def execute_phase(phase: str, binary: Path, archive_root: Path,
                  source_root: Path, output: Path | None = None) -> dict[str, Any]:
    if phase not in RUNS:
        raise ValueError("phase must be calibration or formal")
    seeds = RUNS[phase]
    outdir = (output or (archive_root / OUTPUT_DIRS[phase])).resolve()
    archive_root = archive_root.resolve()
    if outdir != (archive_root / OUTPUT_DIRS[phase]).resolve():
        raise ValueError("output must use the canonical phase directory under the experiment archive")
    require_new_output(outdir)
    binary = binary.resolve(strict=True)
    revision, hashes = source_identity(source_root)
    source_sha = source_map_sha256(hashes)
    protocol_path = source_root / PROTOCOL
    seed_audit = scan_seeds(archive_root, seeds, max_line_bytes=MAX_SEED_RECORD_BYTES)
    if seed_audit["unseen"] != list(seeds):
        raise RuntimeError(f"preregistered seeds are already present in the experiment archive: {seed_audit['matches']}")
    command = ["./lab-probe", "--calibration" if phase == "calibration" else "--formal"]
    probe_copy_sha = sha256_file(binary)
    manifest = {
        "schema": MANIFEST_SCHEMA, "protocol_version": PROTOCOL_VERSION,
        "role": phase, "seeds": list(seeds), "command": command,
        "status": "prepared", "revision": revision, "git_dirty": False,
        "source_sha256": source_sha, "source_files_sha256": hashes,
        "binary_sha256": probe_copy_sha, "artifact_name": "lab-probe",
        "protocol_sha256": sha256_file(protocol_path),
        "runner_sha256": sha256_file(Path(__file__).resolve()),
        "seed_audit": seed_audit,
        "started_at": now(), "timeout_seconds": MAX_SECONDS,
        "max_output_bytes": MAX_OUTPUT_BYTES,
        "max_seed_record_bytes": MAX_SEED_RECORD_BYTES,
        "max_child_memory_bytes": MAX_CHILD_MEMORY_BYTES,
        "raw_file": "raw.jsonl", "stderr_file": "stderr.log",
        "summary_file": "summary.json", "report_file": "report.md",
    }
    if phase == "formal":
        manifest["calibration_identity"] = validate_calibration_parent(archive_root, manifest, source_root)
    outdir.mkdir(parents=True, exist_ok=False)
    manifest_path = outdir / "manifest.json"
    atomic_json(manifest_path, manifest)
    try:
        probe_path = outdir / "lab-probe"
        shutil.copy2(binary, probe_path)
        probe_path.chmod(0o555)
        shutil.copy2(protocol_path, outdir / "protocol.md")
        (outdir / "protocol.md").chmod(0o444)
        snapshot = outdir / "source"
        for source_file in source_paths(source_root):
            relative = source_file.relative_to(source_root)
            destination = snapshot / relative
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(source_file, destination)
            destination.chmod(0o444)
        snapshot_hashes = {
            str(path.relative_to(snapshot)): sha256_file(path)
            for path in snapshot.rglob("*") if path.is_file()
        }
        if snapshot_hashes != hashes:
            raise RuntimeError("immutable source snapshot differs from the registered source hashes")
        if sha256_file(probe_path) != manifest["binary_sha256"]:
            raise RuntimeError("copied lab-probe does not match the registered binary hash")
        work = outdir / "work"
        work.mkdir()
        manifest["status"] = "running"
        manifest["command"] = [str(probe_path), *command[1:]]
        atomic_json(manifest_path, manifest)
        probe_run = run_probe(manifest["command"], work, outdir / "raw.jsonl", outdir / "stderr.log")
        manifest.update(probe_run)
        if probe_run["status"] != "completed":
            raise RuntimeError(probe_run.get("runner_error", f"lab-probe exit code {probe_run.get('exit_code')}"))
        if sha256_file(probe_path) != manifest["binary_sha256"]:
            raise RuntimeError("immutable probe artifact changed during the run")
        if source_hashes(source_root) != hashes:
            raise RuntimeError("source files changed during registered run")
        records = parse_raw(outdir / "raw.jsonl", phase, seeds, outdir / "summary.partial.json")
        summary = write_summary_and_report(outdir, phase, records, seed_audit, probe_run)
        manifest.update(status=terminal_manifest_status(probe_run["status"], summary["status"]),
                        analysis_status=summary["status"],
                        calibration_validity=summary.get("calibration_validity"),
                        raw_sha256=sha256_file(outdir / "raw.jsonl"),
                        summary_sha256=summary["summary_sha256"], report_sha256=summary["report_sha256"],
                        finished_at=now())
    except Exception as error:
        manifest.update(status="failed", runner_error=f"{type(error).__name__}: {error}", finished_at=now())
        if (outdir / "raw.jsonl").is_file():
            manifest["raw_sha256"] = sha256_file(outdir / "raw.jsonl")
        atomic_json(manifest_path, manifest)
        raise
    atomic_json(manifest_path, manifest)
    return manifest


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("phase", choices=("calibration", "formal"))
    parser.add_argument("--binary", type=Path, default=ROOT / "target/release/examples/lifelong_water_lab")
    parser.add_argument("--archive-root", type=Path, default=CANONICAL_ARCHIVE)
    parser.add_argument("--source-root", type=Path, default=ROOT)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args(argv)
    try:
        manifest = execute_phase(args.phase, args.binary, args.archive_root, args.source_root, args.output)
    except Exception as error:
        print(f"lifelong water lab runner failed: {type(error).__name__}: {error}", file=sys.stderr)
        return 1
    print(json.dumps({"status": manifest["status"], "role": manifest["role"],
                      "analysis_status": manifest.get("analysis_status"),
                      "output": str((args.output or args.archive_root / OUTPUT_DIRS[args.phase]).resolve())}))
    return 0 if manifest["status"] == "completed" else 1


if __name__ == "__main__":
    raise SystemExit(main())
