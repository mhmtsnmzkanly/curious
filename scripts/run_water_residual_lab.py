#!/usr/bin/env python3
"""Run the finite, source-bound V17 water residual comparison.

This runner never builds or edits the probe. It scans only the current phase's
registered seeds, retains raw output on failure, and rejects dirty or changed
source, protocol, runner, scanner, or binary identities.
"""
from __future__ import annotations

import argparse
from copy import deepcopy
from datetime import datetime, timezone
import hashlib
import importlib.util
import json
from pathlib import Path
import resource
import shutil
import subprocess
import sys
import time
from typing import Any
from collections.abc import Iterator

ROOT = Path(__file__).resolve().parents[1]
ARCHIVE = Path("/root/curious/data/experiments")
PROTOCOL = Path("docs/WATER_RESIDUAL_PROTOCOL.md")
SCANNER = Path("scripts/run_ecology_traits_lab.py")
RUNS = {
    "calibration": (52099,),
    "formal": (52001, 52002, 52003, 52004, 52005, 52006, 52007, 52008),
}
OUTPUTS = {
    "calibration": "water-residual-v1-calibration-20261023",
    "formal": "water-residual-v1-20261023",
}
SEED_SCHEMA = "curious-water-residual-seed-v1"
SUMMARY_SCHEMA = "curious-water-residual-summary-v1"
MANIFEST_SCHEMA = "curious-water-residual-run-v1"
PROTOCOL_VERSION = 1
MAX_SECONDS = 300
MAX_OUTPUT_BYTES = 1024**3
MAX_SEED_RECORD_BYTES = 64 * 1024**2
MAX_CHILD_MEMORY_BYTES = 1024**3
FOCAL, RIVAL = 1, 2
FORK_TICK, END_TICK = 200, 500
HORIZON = END_TICK - FORK_TICK
WATER_POSITIONS = ((0, 2), (4, 2))
TREATMENTS = {
    "rule_based": (False, False, "retain"),
    "naive_online": (True, True, "clear_residuals"),
    "trained_online": (True, True, "retain"),
    "trained_frozen": (True, False, "retain"),
}
TREATMENT_DESCRIPTIONS = {
    "rule_based": "global_residual=true;enabled=false;update_enabled=false;ordinary-policy",
    "naive_online": "global_residual=true;enabled=true;update_enabled=true;clear-residuals-only",
    "trained_online": "global_residual=true;enabled=true;update_enabled=true;retain-tables",
    "trained_frozen": "global_residual=true;enabled=true;update_enabled=false;retain-tables",
}
DIRECTIONS = {
    "Up": (0, -1), "UpRight": (1, -1), "Right": (1, 0),
    "DownRight": (1, 1), "Down": (0, 1), "DownLeft": (-1, 1),
    "Left": (-1, 0), "UpLeft": (-1, -1),
}


def now() -> str:
    return datetime.now(timezone.utc).isoformat()


def sha(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def canonical_sha(value: Any) -> str:
    data = json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode()
    return hashlib.sha256(data).hexdigest()


def source_paths(root: Path) -> list[Path]:
    paths = [root / "Cargo.toml", root / "Cargo.lock", root / PROTOCOL,
             root / SCANNER, root / Path(__file__).resolve().relative_to(ROOT)]
    paths.extend((root / "src").rglob("*.rs"))
    paths.append(root / "examples/water_residual_lab.rs")
    result = sorted({p.resolve() for p in paths}, key=str)
    missing = [str(p) for p in result if not p.is_file()]
    if missing:
        raise RuntimeError(f"bound source files missing: {missing}")
    return result


def source_hashes(root: Path) -> dict[str, str]:
    return {str(p.relative_to(root.resolve())): sha(p) for p in source_paths(root)}


def source_identity(root: Path) -> tuple[str, dict[str, str]]:
    status = subprocess.check_output(
        ["git", "status", "--porcelain", "--untracked-files=all"], cwd=root, text=True
    )
    if status.strip():
        raise RuntimeError("registered phase requires a clean committed source worktree")
    rev = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=root, text=True).strip()
    return rev, source_hashes(root)


def scan_seeds(registry: Path, seeds: tuple[int, ...]) -> dict[str, Any]:
    module_path = ROOT / SCANNER
    spec = importlib.util.spec_from_file_location("water_seed_scanner", module_path)
    if spec is None or spec.loader is None:
        raise RuntimeError("cannot load the shared complete seed scanner")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module.audit_seeds(registry, list(seeds), max_line_bytes=MAX_SEED_RECORD_BYTES)


def pos(value: Any) -> tuple[int, int]:
    if not isinstance(value, dict) or type(value.get("x")) is not int or type(value.get("y")) is not int:
        raise ValueError("invalid serialized position")
    return value["x"], value["y"]


def actor(state: dict[str, Any], actor_id: int) -> dict[str, Any] | None:
    actors = state.get("entities")
    if not isinstance(actors, list):
        raise ValueError("State has no entity array")
    return next((item for item in actors if item.get("id") == actor_id), None)


def life(item: dict[str, Any] | None) -> dict[str, Any]:
    if not isinstance(item, dict) or not isinstance(item.get("life"), dict):
        raise ValueError("State actor lacks life fields")
    return item["life"]


def learner(state: dict[str, Any], actor_id: int = FOCAL) -> dict[str, Any]:
    subject = actor(state, actor_id)
    result = subject.get("learner") if subject else None
    if not isinstance(result, dict):
        raise ValueError(f"actor {actor_id} lacks water learner")
    return result


def is_alive(state: dict[str, Any], actor_id: int) -> bool:
    subject = actor(state, actor_id)
    return bool(subject and life(subject).get("health", 0) > 0 and subject.get("phase") != "Corpse")


def fold(origin: tuple[int, int], steps: list[str]) -> tuple[int, int]:
    x, y = origin
    for name in steps:
        if name not in DIRECTIONS:
            raise ValueError(f"unknown movement direction {name}")
        dx, dy = DIRECTIONS[name]
        x, y = x + dx, y + dy
    return x, y


def event_items(row: dict[str, Any]):
    events = row.get("events")
    if not isinstance(events, list):
        raise ValueError("tick row has no resolver event array")
    for event in events:
        if not isinstance(event, dict) or len(event) != 1:
            raise ValueError("invalid serialized SimulationEvent")
        yield next(iter(event.items()))


def decision(row: dict[str, Any]) -> dict[str, Any] | None:
    matches = [d for d in row.get("decisions", []) if isinstance(d, dict) and d.get("id") == FOCAL]
    if len(matches) > 1:
        raise ValueError("duplicate focal decision trace")
    return matches[0] if matches else None


def life_baseline(amount: int, route: int, body: dict[str, Any]) -> int:
    travel = max(route - 1, 0)
    speed = max(int(body.get("speed", 1)), 1)
    ticks = max((travel + speed - 1) // speed, 1)
    capacity = max(int(body.get("max_water", 0)) - int(body.get("water", 0)), 0)
    meal = min(amount, capacity, 10)
    benefit = meal * 2
    cost = travel * int(body.get("move_energy_cost", 0)) + ticks
    numerator = (benefit - cost) * 100
    return (abs(numerator) // ticks) * (-1 if numerator < 0 else 1)


def trunc_div(numerator: int, denominator: int) -> int:
    return (abs(numerator) // denominator) * (-1 if numerator < 0 else 1)


def normalized_choice_state(state: dict[str, Any]) -> dict[str, Any]:
    result = deepcopy(state)
    l = learner(result)
    l["values"] = []
    l["residuals"] = []
    l["update_enabled"] = True
    return result


def fixture_initial_rng(seed: int) -> int:
    state = seed
    mask = (1 << 64) - 1
    for _ in range(25):
        roll = state % 100
        state = (state * 6364136223846793005 + 1) & mask
        if roll < 5:
            state = (state * 6364136223846793005 + 1) & mask
            state = (state * 6364136223846793005 + 1) & mask
    return state


def validate_initial(state: dict[str, Any], seed: int, label: str) -> None:
    if state.get("version")!=17 or state.get("tick") != 0 or state.get("rng") != fixture_initial_rng(seed):
        raise ValueError(f"{label}: initial tick/RNG mismatch")
    if state.get("bounds") != [0, 4, 0, 4] or state.get("policy", {}).get("water_residual_enabled") is not True:
        raise ValueError(f"{label}: bounds or global V17 water flag mismatch")
    actors = state.get("entities", [])
    if {x.get("id") for x in actors} != {FOCAL, RIVAL}:
        raise ValueError(f"{label}: expected exactly focal and ordinary rival")
    for actor_id, expected_pos in ((FOCAL, (2, 2)), (RIVAL, (2, 3))):
        entity = actor(state, actor_id)
        if pos(entity.get("pos")) != expected_pos or entity.get("species") != "Herbivore" or entity.get("controller") != "AI":
            raise ValueError(f"{label}: actor {actor_id} does not match fixture")
        body = life(entity)
        if body.get("age") != 0 or body.get("max_age") != 512 or body.get("maturity_age") != 501:
            raise ValueError(f"{label}: actor {actor_id} age/lifespan mismatch")
        for key, value in {"max_water":60,"water":60,"max_energy":80,"energy":80,"max_health":120,
                           "health":120,"speed":3,"move_energy_cost":1,"max_stamina":16,
                           "stamina":16,"vision_range":6}.items():
            if body.get(key) != value:
                raise ValueError(f"{label}: actor {actor_id} body field {key} mismatch")
        l = learner(state, actor_id)
        expected = (True, True) if actor_id == FOCAL else (False, False)
        if (l.get("enabled"), l.get("update_enabled")) != expected:
            raise ValueError(f"{label}: actor {actor_id} learner flags mismatch")
        if l.get("values") != [] or l.get("residuals", []) != [] or l.get("episode") is not None:
            raise ValueError(f"{label}: initial learner history is not empty")
    sites = state.get("environment", {}).get("sites", [])
    expected_sites = {
        (0,2): ("Water",12,4,2), (4,2): ("Water",12,1,4),
        (0,0): ("Food",20,2,2), (4,0): ("Food",20,2,2),
        (0,4): ("Food",20,2,2), (4,4): ("Food",20,2,2),
    }
    actual = {pos(s.get("pos")): (s.get("kind"),s.get("capacity"),s.get("replenish"),s.get("interval")) for s in sites}
    if actual != expected_sites:
        raise ValueError(f"{label}: initial resource layout/rates mismatch")


def check_state(state: dict[str, Any], label: str) -> None:
    if state.get("version")!=17:
        raise ValueError(f"{label}: checkpoint State version is not V17")
    if state.get("policy", {}).get("water_residual_enabled") is not True:
        raise ValueError(f"{label}: global V17 residual option is not enabled")
    if actor(state,RIVAL) is not None and learner(state, RIVAL).get("enabled") is not False:
        raise ValueError(f"{label}: rival learner must remain disabled")


def assert_tick(row: dict[str, Any], tick: int, previous: dict[str, Any], label: str,
                death_seen: set[int] | None = None) -> dict[str, Any]:
    if row.get("tick") != tick or row.get("state_before") != previous:
        raise ValueError(f"{label} tick {tick}: state continuity/tick mismatch")
    before, after = row.get("state_before"), row.get("state_after")
    if not isinstance(before, dict) or not isinstance(after, dict) or before.get("tick") != tick-1 or after.get("tick") != tick:
        raise ValueError(f"{label} tick {tick}: missing complete State pair")
    if before.get("rng") != row.get("rng_before") or after.get("rng") != row.get("rng_after"):
        raise ValueError(f"{label} tick {tick}: captured RNG differs from State")
    check_state(before, label)
    check_state(after, label)
    old, new = {a["id"]:a for a in before.get("entities",[])}, {a["id"]:a for a in after.get("entities",[])}
    if (len(old)!=len(before.get("entities",[])) or len(new)!=len(after.get("entities",[]))
            or not set(old).issubset({FOCAL,RIVAL}) or not set(new).issubset({FOCAL,RIVAL})):
        raise ValueError(f"{label} tick {tick}: duplicate or unregistered actor ID")
    if not set(new).issubset(old) or FOCAL not in new:
        raise ValueError(f"{label} tick {tick}: actor creation or focal removal is forbidden")
    if death_seen is None:
        death_seen=set()
    tick_events=list(event_items(row))
    deaths={payload.get("id") for kind,payload in tick_events if kind=="EntityDied"}
    for actor_id in set(old)-set(new):
        if actor_id != RIVAL or (actor_id not in death_seen and actor_id not in deaths):
            raise ValueError(f"{label} tick {tick}: rival removal lacks a prior natural death event")
    death_seen.update(deaths)
    for actor_id in set(old) & set(new):
        a, b = old[actor_id], new[actor_id]
        if a.get("controller") != b.get("controller") or a.get("species") != b.get("species") or a.get("genome") != b.get("genome"):
            raise ValueError(f"{label} tick {tick}: actor identity changed")
        la, lb = life(a), life(b)
        for key in ("max_age","maturity_age","max_water","max_energy","max_health","speed","move_energy_cost","max_stamina","vision_range"):
            if la.get(key) != lb.get(key):
                raise ValueError(f"{label} tick {tick}: body field {key} changed")
        expected_age = la.get("age") + (1 if la.get("health",0)>0 else 0)
        if lb.get("age") != expected_age:
            raise ValueError(f"{label} tick {tick}: age reset/decrement")
    trace = decision(row)
    if (trace is None) != (row.get("focal_choice") is None):
        raise ValueError(f"{label} tick {tick}: focal choice trace mismatch")
    if trace is not None:
        validate_choice(row["focal_choice"], trace, before, label, tick)
    validate_learner_closures(row,trace,label,tick)
    return after


def validate_learner_closures(row: dict[str,Any],trace: dict[str,Any]|None,label: str,tick: int) -> None:
    audit=row.get("learner_closures")
    if (not isinstance(audit,dict) or audit.get("replayed_decision_matches_trace") is not True
            or audit.get("replayed_postdecision_rng_matches_world") is not True
            or audit.get("replayed_learner_matches_world") is not True):
        raise ValueError(f"{label} tick {tick}: cloned learner/resolver replay mismatch")
    pre=audit.get("predecision_closures")
    post=audit.get("postresolver_closures")
    if not isinstance(pre,list) or not isinstance(post,list) or len(pre)>1 or len(post)>1:
        raise ValueError(f"{label} tick {tick}: invalid episode-closure audit shape")
    before=learner(row["state_before"])
    after=learner(row["state_after"])
    active=before.get("episode")
    expected_pre=[]
    mid=active
    choice=row.get("focal_choice") or {}
    if trace is not None:
        enabled=before.get("enabled") is True
        base_reason=choice.get("base_reason")
        waters=(trace.get("perception") or {}).get("waters",[])
        visible={fold(pos(trace["pos"]),w.get("steps",[])) for w in waters}
        active_visible=active is not None and pos(active["target"]) in visible
        close_reason=None
        if not enabled and active is not None:
            close_reason="Disabled"
        elif base_reason!="SeekWater" and active is not None:
            close_reason="Interrupted"
        elif base_reason=="SeekWater" and active is not None and not active_visible:
            close_reason="Interrupted"
        if close_reason is not None:
            expected_pre=[{"attempt":active,"water":0,"end":close_reason,"reward":None}]
            mid=None
        if enabled and base_reason=="SeekWater" and not active_visible:
            candidates=choice.get("candidates",[])
            selected=choice.get("selected_target")
            if len(waters)>=2 and selected is not None:
                target=pos(selected)
                candidate=next((c for c in candidates if pos(c["pos"])==target),None)
                if candidate is None:
                    raise ValueError(f"{label} tick {tick}: new episode target is not a perceived source")
                mid={"target":selected,"ticks":0,"movement_energy":0,"incoming_damage":0,
                     "choice_baseline":candidate["current_baseline"]}
    if pre!=expected_pre:
        raise ValueError(f"{label} tick {tick}: predecision episode closure was missed or duplicated")
    if audit.get("episode_after_choice")!=mid:
        raise ValueError(f"{label} tick {tick}: cloned learner episode after choice mismatch")
    actual_post=mid is not None and after.get("episode") is None
    if actual_post:
        if len(post)!=1 or post[0]!=after.get("last_episode"):
            raise ValueError(f"{label} tick {tick}: postresolver closure missing or differs from final learner result")
    elif post:
        raise ValueError(f"{label} tick {tick}: spurious postresolver closure")
    expected_last=(post[0] if post else (expected_pre[-1] if expected_pre else before.get("last_episode")))
    if after.get("last_episode")!=expected_last:
        raise ValueError(f"{label} tick {tick}: closure result was not retained exactly once")


def validate_choice(choice: Any, trace: dict[str, Any], before: dict[str, Any], label: str, tick: int) -> None:
    if not isinstance(choice, dict):
        raise ValueError(f"{label} tick {tick}: malformed focal choice")
    percep = trace.get("perception") or {}
    waters = percep.get("waters", [])
    body = trace.get("before")
    if not isinstance(body, dict):
        raise ValueError(f"{label} tick {tick}: trace lacks body snapshot")
    learner_before = learner(before)
    if choice.get("active_episode_before") != learner_before.get("episode"):
        raise ValueError(f"{label} tick {tick}: episode/baseline mismatch")
    if len(choice.get("candidates",[])) != len(waters):
        raise ValueError(f"{label} tick {tick}: candidates differ from perception")
    for actual, perceived in zip(choice["candidates"], waters):
        target = fold(pos(trace.get("pos")), perceived.get("steps",[]))
        if pos(actual.get("pos")) != target or actual.get("amount") != perceived.get("amount") or actual.get("steps_len") != len(perceived.get("steps",[])):
            raise ValueError(f"{label} tick {tick}: source evidence differs from perception")
        values = {pos(item["pos"]):item for item in learner_before.get("values",[])}
        residuals = {pos(item["pos"]):item for item in learner_before.get("residuals",[])}
        legacy = values.get(target)
        residual = residuals.get(target)
        if (actual.get("legacy_value"),actual.get("legacy_visits")) != ((legacy.get("value"),legacy.get("visits")) if legacy else (None,None)):
            raise ValueError(f"{label} tick {tick}: legacy value evidence mismatch")
        total = residual.get("sum",0) if residual else 0
        count = residual.get("visits",0) if residual else 0
        mean = trunc_div(total,count) if count else 0
        baseline = life_baseline(perceived["amount"], len(perceived.get("steps",[])), body)
        if (actual.get("residual_sum"),actual.get("residual_count"),actual.get("residual_mean"),actual.get("current_baseline"),actual.get("final_score")) != (total,count,mean,baseline,baseline+mean):
            raise ValueError(f"{label} tick {tick}: residual score components mismatch")
    if choice.get("selection_kind") not in ("base","ongoing","greedy","explore"):
        raise ValueError(f"{label} tick {tick}: invalid selection classification")
    intent=trace.get("decision",{}).get("intent")
    drink=intent.get("Drink") if isinstance(intent,dict) else None
    expected_target=fold(pos(trace.get("pos")),drink.get("at",[])) if isinstance(drink,dict) else None
    selected=choice.get("selected_target")
    if (None if selected is None else pos(selected))!=expected_target:
        raise ValueError(f"{label} tick {tick}: selected source differs from actual focal intent")


def treatment_state(checkpoint: dict[str, Any], arm: str) -> dict[str, Any]:
    result = deepcopy(checkpoint)
    l = learner(result)
    enabled, updates, treatment = TREATMENTS[arm]
    l["enabled"], l["update_enabled"] = enabled, updates
    if treatment == "clear_residuals":
        l["residuals"] = []
        # V17 serde omits an empty residual vector in captured State JSON.
        l.pop("residuals",None)
    return result


def reverse_rates(state: dict[str, Any]) -> dict[str, Any]:
    result = deepcopy(state)
    wanted = {(0,2):(1,4),(4,2):(4,2)}
    seen = set()
    for site in result.get("environment",{}).get("sites",[]):
        point = pos(site.get("pos"))
        if point in wanted and site.get("kind") == "Water":
            site["replenish"],site["interval"] = wanted[point]
            seen.add(point)
    if seen != set(WATER_POSITIONS):
        raise ValueError("reversal source layout mismatch")
    return result


def validate_record(record: dict[str, Any], seed: int, role: str) -> dict[str, Any]:
    label = f"{role}/{seed}"
    if record.get("schema") != SEED_SCHEMA or record.get("protocol_version") != PROTOCOL_VERSION:
        raise ValueError(f"{label}: seed schema/version mismatch")
    if record.get("seed") != seed or record.get("role") != role:
        raise ValueError(f"{label}: role/seed mismatch")
    initial = record.get("initial_state")
    validate_initial(initial, seed, label)
    prefix = record.get("prefix")
    if not isinstance(prefix,list) or not 0 < len(prefix) <= FORK_TICK:
        raise ValueError(f"{label}: invalid continuous prefix")
    previous = initial
    prefix_deaths: set[int]=set()
    for tick,row in enumerate(prefix,1):
        previous = assert_tick(row,tick,previous,label+" prefix",prefix_deaths)
    checkpoint = record.get("checkpoint_state")
    if checkpoint != previous:
        raise ValueError(f"{label}: checkpoint does not match continuous prefix")
    can_fork=is_alive(checkpoint,FOCAL) and is_alive(checkpoint,RIVAL)
    if can_fork and len(prefix)!=FORK_TICK:
        raise ValueError(f"{label}: live checkpoint did not reach tick 200")
    if not can_fork:
        if record.get("status")!="invalid_checkpoint_actor_dead" or any(v is not None for v in record.get("forks",{}).values()):
            raise ValueError(f"{label}: pre-fork death status/arms mismatch")
        if record.get("physical_ticks")!=len(prefix):
            raise ValueError(f"{label}: pre-fork biological tick total mismatch")
        return {"seed":seed,"role":role,"status":"biological_negative_before_fork","physical_ticks":record["physical_ticks"],
                "prefix":summarize_prefix(prefix,initial,checkpoint),
                "branches":{name:None for name in TREATMENTS},"checkpoint_residuals":learner(checkpoint).get("residuals",[]),
                "training_opportunity":False,
                "causal_audit":None}
    if record.get("status") != "complete":
        raise ValueError(f"{label}: registered seed did not reach its live fork")
    if record.get("physical_ticks") != FORK_TICK + sum(len(record.get("forks",{}).get(n,{}).get("ticks",[])) for n in TREATMENTS):
        raise ValueError(f"{label}: physical tick total mismatch")
    if record["physical_ticks"] > 1400:
        raise ValueError(f"{label}: physical tick budget exceeded")
    forks = record.get("forks")
    if not isinstance(forks,dict) or set(forks) != set(TREATMENTS):
        raise ValueError(f"{label}: arm set mismatch")
    for arm in TREATMENTS:
        branch = forks[arm]
        if not isinstance(branch,dict) or branch.get("state_before_treatment") != checkpoint:
            raise ValueError(f"{label}/{arm}: did not fork from exact checkpoint")
        expected_treated = treatment_state(checkpoint,arm)
        if branch.get("state_after_treatment_before_reversal") != expected_treated:
            raise ValueError(f"{label}/{arm}: treatment changed unregistered state")
        if branch.get("learner_treatment") != TREATMENT_DESCRIPTIONS[arm]:
            raise ValueError(f"{label}/{arm}: treatment descriptor mismatch")
        if arm == "naive_online":
            cl, tl = learner(checkpoint), learner(branch["state_after_treatment_before_reversal"])
            if tl.get("values") != cl.get("values") or tl.get("episode") != cl.get("episode"):
                raise ValueError(f"{label}: naive arm failed to preserve legacy values/active baseline")
        at201 = branch.get("state_at_tick_201_before_world_tick")
        if at201 != reverse_rates(expected_treated):
            raise ValueError(f"{label}/{arm}: unregistered reversal state")
        rows = branch.get("ticks")
        if not isinstance(rows,list) or not 0 <= len(rows) <= HORIZON:
            raise ValueError(f"{label}/{arm}: invalid suffix length")
        previous = at201
        branch_deaths:set[int]=set()
        frozen = learner(at201).get("values"),learner(at201).get("residuals")
        for i,row in enumerate(rows,FORK_TICK+1):
            previous = assert_tick(row,i,previous,label+"/"+arm,branch_deaths)
            if arm=="rule_based" and learner(previous).get("enabled") is not False:
                raise ValueError(f"{label}: rule-based arm re-enabled focal learning")
            if arm == "trained_frozen":
                after_tables = learner(previous).get("values"),learner(previous).get("residuals")
                if after_tables != frozen:
                    raise ValueError(f"{label}: trained-frozen learned tables changed")
        status = branch.get("status")
        if status == "ceiling":
            if len(rows) != HORIZON or not is_alive(previous,FOCAL):
                raise ValueError(f"{label}/{arm}: false full-horizon status")
        elif status == "focal_died":
            if not rows or is_alive(previous,FOCAL):
                raise ValueError(f"{label}/{arm}: death status lacks observed death")
        else:
            raise ValueError(f"{label}/{arm}: invalid terminal status")
    return summarize_seed(record)


def focal_events(row: dict[str, Any]) -> tuple[int,int]:
    water = move = 0
    for kind,payload in event_items(row):
        if kind == "EntityDrank" and payload.get("id") == FOCAL:
            if type(payload.get("amount")) is not int or payload["amount"] < 0:
                raise ValueError("invalid resolver drink amount")
            water += payload["amount"]
        elif kind == "EntityMoved" and payload.get("id") == FOCAL:
            if type(payload.get("energy_cost")) is not int or payload["energy_cost"] < 0:
                raise ValueError("invalid resolver movement cost")
            move += payload["energy_cost"]
    return water,move


def branch_metrics(branch: dict[str, Any]) -> dict[str, Any]:
    rows = branch["ticks"]
    water = move = 0
    for row in rows:
        w,m = focal_events(row); water += w; move += m
    terminal_state = rows[-1]["state_after"] if rows else branch["state_at_tick_201_before_world_tick"]
    focal = actor(terminal_state,FOCAL)
    terminal = {k:life(focal).get(k) for k in ("energy","water","health")} if focal else None
    death_events=[payload for row in rows for kind,payload in event_items(row) if kind=="EntityDied" and payload.get("id")==FOCAL]
    rival_deaths=[payload for row in rows for kind,payload in event_items(row) if kind=="EntityDied" and payload.get("id")==RIVAL]
    end_counts,closure_count=censor_counts(rows)
    choices=choice_exposure_counts(rows)
    ending_state=rows[-1]["state_after"] if rows else branch["state_at_tick_201_before_world_tick"]
    rival=actor(ending_state,RIVAL)
    return {"status":branch["status"],"ticks":len(rows),"water_acquired":water,
            "resolver_drinks":sum(1 for row in rows for kind,payload in event_items(row) if kind=="EntityDrank" and payload.get("id")==FOCAL),
            "movement_energy_paid":move,"observed_survival_ticks":len(rows),
            "terminal_state":terminal,"terminal_state_known":branch["status"]=="ceiling",
            "death_event":death_events[-1] if death_events else None,
            "death_cause":life(focal).get("death_cause") if focal else "unknown_removed",
            "episode_closure_count":closure_count,"episode_end_counts":end_counts,
            "censored_episode_closures":sum(n for end,n in end_counts.items() if end in ("Interrupted","Disabled","Died","TimedOut")),
            "fresh_two_positive_choices":choices["fresh_two_positive_choices"],
            "fresh_two_positive_body_nonfull_choices":choices["fresh_two_positive_body_nonfull_choices"],
            "rival_status":"removed" if rival is None else ("alive" if is_alive(ending_state,RIVAL) else "dead"),
            "rival_death_event":rival_deaths[-1] if rival_deaths else None}


def choice_exposure_counts(rows: list[dict[str,Any]]) -> dict[str,int]:
    total=body_nonfull=0
    for row in rows:
        choice=row.get("focal_choice")
        if not isinstance(choice,dict) or not choice.get("fresh"):
            continue
        candidates=choice.get("candidates",[])
        if choice.get("base_reason")!="SeekWater" or len(candidates)<2 or sum(c.get("amount",0)>0 for c in candidates)<2:
            continue
        total+=1
        if fresh_two_positive_body_nonfull(row): body_nonfull+=1
    return {"fresh_two_positive_choices":total,"fresh_two_positive_body_nonfull_choices":body_nonfull}


def summarize_seed(record: dict[str, Any]) -> dict[str, Any]:
    forks = record["forks"]
    checkpoint=record["checkpoint_state"]
    branches = {name:branch_metrics(forks[name]) for name in TREATMENTS}
    prefix_metrics=summarize_prefix(record["prefix"],record["initial_state"],checkpoint)
    return {"seed":record["seed"],"role":record["role"],"status":record["status"],
            "physical_ticks":record["physical_ticks"],"prefix":prefix_metrics,"branches":branches,
            "checkpoint_residuals":learner(checkpoint).get("residuals",[]),
            "training_opportunity":bool(learner(checkpoint).get("residuals"))}


def censor_counts(rows: list[dict[str,Any]]) -> tuple[dict[str,int],int]:
    counts:dict[str,int]={}
    total=0
    for row in rows:
        audit=row["learner_closures"]
        closures=audit["predecision_closures"]+audit["postresolver_closures"]
        total+=len(closures)
        for closure in closures:
            end=closure.get("end","unknown")
            counts[end]=counts.get(end,0)+1
    return counts,total


def summarize_prefix(rows: list[dict[str,Any]],start:dict[str,Any],end:dict[str,Any]) -> dict[str,Any]:
    water=sum(focal_events(row)[0] for row in rows)
    movement=sum(focal_events(row)[1] for row in rows)
    deaths={actor_id:[payload for row in rows for kind,payload in event_items(row)
                      if kind=="EntityDied" and payload.get("id")==actor_id]
            for actor_id in (FOCAL,RIVAL)}
    end_counts,closure_count=censor_counts(rows)
    choices=choice_exposure_counts(rows)
    actors={}
    for actor_id in (FOCAL,RIVAL):
        entity=actor(end,actor_id)
        actors[str(actor_id)]={"status":"removed" if entity is None else ("alive" if is_alive(end,actor_id) else "dead"),
            "death_event":deaths[actor_id][-1] if deaths[actor_id] else None,
            "terminal_state":({k:life(entity).get(k) for k in ("energy","water","health")} if entity else None)}
    return {"ticks":len(rows),"status":"reached_fork" if is_alive(end,FOCAL) and is_alive(end,RIVAL) else "biological_negative_before_fork",
            "water_acquired":water,"movement_energy_paid":movement,"actors":actors,
            "episode_closure_count":closure_count,"episode_end_counts":end_counts,
            "censored_episode_closures":sum(n for end,n in end_counts.items() if end in ("Interrupted","Disabled","Died","TimedOut")),
            **choices}


def choice_rows(branch: dict[str, Any]):
    for row in branch["ticks"]:
        choice=row.get("focal_choice")
        if isinstance(choice,dict):
            yield row,choice


def same_physical_prestate(left: dict[str, Any], right: dict[str, Any]) -> bool:
    return (normalized_choice_state(left["state_before"]) == normalized_choice_state(right["state_before"])
            and left.get("rng_before")==right.get("rng_before"))


def causal_audit(record: dict[str, Any]) -> dict[str, Any]:
    online=record["forks"]["trained_online"]
    frozen=record["forks"]["trained_frozen"]
    right={row["tick"]:(row,choice) for row,choice in choice_rows(frozen)}
    selected=None
    for row,choice in choice_rows(online):
        other=right.get(row["tick"])
        if other is None: continue
        rrow,rchoice=other
        if not choice.get("fresh") or not rchoice.get("fresh"): continue
        if choice.get("selection_kind")!="greedy" or rchoice.get("selection_kind")!="greedy": continue
        if pos(choice["selected_target"]) == pos(rchoice["selected_target"]): continue
        lt,rt=decision(row),decision(rrow)
        if (not same_physical_prestate(row,rrow) or lt is None or rt is None
                or lt.get("before")!=rt.get("before") or lt.get("perception")!=rt.get("perception")
                or lt.get("pos")!=rt.get("pos")):
            continue
        # Both scores use current visible foraging baseline; compare the actual
        # online and frozen sum/count means on this same exposure.
        same_candidates=[]
        for lc,rc in zip(choice["candidates"],rchoice["candidates"]):
            if lc["pos"]!=rc["pos"] or lc["current_baseline"]!=rc["current_baseline"]: break
            same_candidates.append((lc,rc))
        if len(same_candidates)!=len(choice["candidates"]): continue
        if not greedy_by_score(choice) or not greedy_by_score(rchoice): continue
        selected={"tick":row["tick"],"trained_online_target":choice["selected_target"],
                  "trained_frozen_target":rchoice["selected_target"],"same_physical_prestate_perception_rng":True,
                  "same_current_baselines":True,"online_residuals":[(c["residual_sum"],c["residual_count"]) for c in choice["candidates"]],
                  "frozen_residuals":[(c["residual_sum"],c["residual_count"]) for c in rchoice["candidates"]]}
        break
    physical=None
    if selected:
        online_rows={row["tick"]:row for row in online["ticks"]}
        frozen_rows={row["tick"]:row for row in frozen["ticks"]}
        for tick in sorted(t for t in online_rows if t>=selected["tick"] and t in frozen_rows):
            lo=own_physical_events(online_rows[tick]); lf=own_physical_events(frozen_rows[tick])
            if lo!=lf:
                physical={"tick":tick,"trained_online_events":lo,"trained_frozen_events":lf}
                break
    if selected is not None:
        selected["first_different_own_resolver_events_after_choice"]=physical
    return {"mechanism_pass":bool(selected and physical),
            "first_qualifying_greedy_divergence":selected}


def own_physical_events(row: dict[str, Any]) -> list[dict[str, Any]]:
    result=[]
    for kind,payload in event_items(row):
        if kind=="EntityDrank" and payload.get("id")==FOCAL:
            result.append({"kind":"drink","pos":payload.get("pos"),"amount":payload.get("amount")})
        elif kind=="EntityMoved" and payload.get("id")==FOCAL:
            result.append({"kind":"move","from":payload.get("from"),"to":payload.get("to"),"energy_cost":payload.get("energy_cost")})
    return result


def greedy_by_score(choice: dict[str, Any]) -> bool:
    candidates=choice.get("candidates",[])
    if len(candidates)<2: return False
    winner=max(range(len(candidates)),key=lambda i:(candidates[i]["final_score"],-candidates[i]["steps_len"],-i))
    return pos(choice.get("selected_target"))==pos(candidates[winner]["pos"])


def compare(a: dict[str,Any], b: dict[str,Any]) -> dict[str,Any]:
    keys=("water_acquired","movement_energy_paid","observed_survival_ticks")
    differences={k:a[k]-b[k] for k in keys}
    ta,tb=a.get("terminal_state"),b.get("terminal_state")
    if a["terminal_state_known"] and b["terminal_state_known"] and ta and tb:
        differences.update({f"terminal_{k}":ta[k]-tb[k] for k in ("energy","water","health")})
    else:
        differences.update({f"terminal_{k}":None for k in ("energy","water","health")})
    return {"differences":differences,"full_horizon_pair":a["terminal_state_known"] and b["terminal_state_known"]}


def pareto_relation(candidate: dict[str,Any], control: dict[str,Any]) -> str:
    comp=compare(candidate,control)
    if not comp["full_horizon_pair"]: return "insufficient_common_horizon"
    d=comp["differences"]
    favorable=[d["water_acquired"]>=0,d["movement_energy_paid"]<=0,d["observed_survival_ticks"]>=0,
               d["terminal_energy"]>=0,d["terminal_water"]>=0,d["terminal_health"]>=0]
    strict=[d["water_acquired"]>0,d["movement_energy_paid"]<0,d["observed_survival_ticks"]>0,
            d["terminal_energy"]>0,d["terminal_water"]>0,d["terminal_health"]>0]
    if all(favorable) and any(strict): return "dominates"
    if all(v<=0 for v in [d["water_acquired"],-d["movement_energy_paid"],d["observed_survival_ticks"],d["terminal_energy"],d["terminal_water"],d["terminal_health"]]) and any(v<0 for v in [d["water_acquired"],-d["movement_energy_paid"],d["observed_survival_ticks"],d["terminal_energy"],d["terminal_water"],d["terminal_health"]]): return "dominated"
    if all(v==0 for v in [d["water_acquired"],d["movement_energy_paid"],d["observed_survival_ticks"],d["terminal_energy"],d["terminal_water"],d["terminal_health"]]): return "identical"
    return "tradeoff"


def calibration_gate(record: dict[str,Any]) -> dict[str,Any]:
    if record.get("status")!="complete":
        checks={"both_actors_alive_at_fork":False,"trained_residual_nonempty":False,
                "prefix_at_least_two_fresh_nonfull_two_source_choices":False,
                "trained_online_suffix_at_least_two_fresh_nonfull_two_source_choices":False,
                "matched_completed_drink_in_prefix":False,"completed_drink_in_suffix":False,
                "suffix_residual_sum_count_update":False,
                "completed_update_changes_selected_source_mean":False}
        return {"passed":False,"checks":checks,"observed":{"status":"biological_negative_before_fork"}}
    prefix=[r for r in record["prefix"] if fresh_two_positive_body_nonfull(r)]
    online=record["forks"]["trained_online"]
    suffix=[r for r in online["ticks"] if fresh_two_positive_body_nonfull(r)]
    prefix_drinks=own_completed_drinks(record["prefix"])
    suffix_drinks=own_completed_drinks(online["ticks"])
    updates=any(learner(row["state_after"]).get("residuals") != learner(row["state_before"]).get("residuals") for row in online["ticks"])
    mean_changed=completed_update_changes_selected_mean(online["ticks"])
    checks={"both_actors_alive_at_fork":is_alive(record["checkpoint_state"],FOCAL) and is_alive(record["checkpoint_state"],RIVAL),
            "trained_residual_nonempty":bool(learner(record["checkpoint_state"]).get("residuals")),
            "prefix_at_least_two_fresh_nonfull_two_source_choices":len(prefix)>=2,
            "trained_online_suffix_at_least_two_fresh_nonfull_two_source_choices":len(suffix)>=2,
            "matched_completed_drink_in_prefix":prefix_drinks>0,
            "completed_drink_in_suffix":suffix_drinks>0,
            "suffix_residual_sum_count_update":updates,
            "completed_update_changes_selected_source_mean":mean_changed}
    return {"passed":all(checks.values()),"checks":checks,"observed":{"prefix_choices":len(prefix),"suffix_choices":len(suffix),"prefix_drinks":prefix_drinks,"suffix_drinks":suffix_drinks,"post_mean_change":mean_changed}}


def completed_update_changes_selected_mean(rows: list[dict[str,Any]]) -> bool:
    for row in rows:
        if learner(row["state_before"]).get("update_enabled") is not True:
            continue
        before={pos(v["pos"]):v for v in learner(row["state_before"]).get("residuals",[])}
        after={pos(v["pos"]):v for v in learner(row["state_after"]).get("residuals",[])}
        for closure in row["learner_closures"]["postresolver_closures"]:
            attempt=closure.get("attempt",{})
            if closure.get("end") not in ("Reached","Failed") or attempt.get("choice_baseline") is None:
                continue
            if attempt.get("outcome_overflowed",False) or attempt.get("candidate_censored",False):
                continue
            target=pos(attempt["target"])
            old,new=before.get(target),after.get(target)
            old_count=old.get("visits",0) if old else 0
            new_count=new.get("visits",0) if new else 0
            if new_count!=old_count+1:
                continue
            old_mean=trunc_div(old["sum"],old["visits"]) if old and old.get("visits",0)>0 else 0
            new_mean=trunc_div(new["sum"],new["visits"]) if new and new.get("visits",0)>0 else 0
            if old_mean!=new_mean:
                return True
    return False


def fresh_two_positive_body_nonfull(row: dict[str,Any]) -> bool:
    choice=row.get("focal_choice")
    if not isinstance(choice,dict) or not choice.get("fresh"): return False
    candidates=choice.get("candidates",[])
    if len(candidates)<2 or any(c.get("amount",0)<=0 for c in candidates): return False
    focal=actor(row["state_before"],FOCAL)
    body=life(focal)
    return body.get("water",0)<body.get("max_water",0)


def own_completed_drinks(rows: list[dict[str,Any]]) -> int:
    count=0
    for row in rows:
        before=learner(row["state_before"])
        after=learner(row["state_after"])
        choice=row.get("focal_choice") or {}
        fresh_target=choice.get("selected_target") if choice.get("fresh") else None
        active=before.get("episode")
        result=after.get("last_episode")
        if not isinstance(result,dict) or after.get("episode") is not None:
            continue
        attempt=result.get("attempt")
        if not isinstance(attempt,dict) or result.get("end")!="Reached" or result.get("water",0)<=0:
            continue
        target=pos(attempt.get("target"))
        if not ((active is not None and pos(active.get("target"))==target) or
                (fresh_target is not None and pos(fresh_target)==target)):
            continue
        count += sum(1 for kind,payload in event_items(row)
                     if kind=="EntityDrank" and payload.get("id")==FOCAL
                     and payload.get("amount",0)>0 and pos(payload.get("pos"))==target)
    return count


def aggregate_formal(seeds: list[dict[str,Any]]) -> dict[str,Any]:
    pairs={}
    for arm in ("naive_online","trained_online"):
        pairs[arm]={}
        for comparator in ("trained_frozen","rule_based"):
            all_four_full=all(isinstance(s["branches"].get(name),dict) and s["branches"][name]["terminal_state_known"]
                              for s in seeds for name in TREATMENTS)
            all_full=all_four_full and all(s["branches"][arm]["terminal_state_known"] and s["branches"][comparator]["terminal_state_known"] for s in seeds)
            diffs={}
            if all_full:
                for component in ("water_acquired","movement_energy_paid","observed_survival_ticks"):
                    diffs[component]=sum(s["branches"][arm][component]-s["branches"][comparator][component] for s in seeds)
                for component in ("energy","water","health"):
                    diffs[f"terminal_{component}"]=sum(s["branches"][arm]["terminal_state"][component]-s["branches"][comparator]["terminal_state"][component] for s in seeds)
            per_seed_no_harm=all(
                isinstance(s["branches"].get(arm),dict) and isinstance(s["branches"].get(comparator),dict)
                and
                s["branches"][arm]["terminal_state_known"] and s["branches"][comparator]["terminal_state_known"]
                and s["branches"][arm]["observed_survival_ticks"]>=s["branches"][comparator]["observed_survival_ticks"]
                and s["branches"][arm]["terminal_state"]["health"]>=s["branches"][comparator]["terminal_state"]["health"]
                for s in seeds)
            signs=(diffs.get("water_acquired",-1)>=0 and diffs.get("movement_energy_paid",1)<=0 and diffs.get("observed_survival_ticks",-1)>=0
                   and diffs.get("terminal_energy",-1)>=0 and diffs.get("terminal_water",-1)>=0 and diffs.get("terminal_health",-1)>=0)
            strict=any((diffs.get("water_acquired",0)>0,diffs.get("movement_energy_paid",0)<0,diffs.get("observed_survival_ticks",0)>0,
                        diffs.get("terminal_energy",0)>0,diffs.get("terminal_water",0)>0,diffs.get("terminal_health",0)>0))
            pairs[arm][comparator]={"all_four_arms_full_300_tick_all_seeds":all_four_full,
                                    "full_300_tick_pair_horizon_all_seeds":all_full,"aggregate_candidate_minus_control":diffs,
                                    "per_seed_health_and_survival_nonnegative":per_seed_no_harm,"aggregate_pareto_dominates":bool(all_full and per_seed_no_harm and signs and strict)}
    primary=pairs["trained_online"]
    all_training_opportunities=all(s.get("training_opportunity") is True for s in seeds)
    return {"arm_comparisons":pairs,"all_seeds_had_nonempty_trained_residuals_at_fork":all_training_opportunities,
            "bounded_utility_evidence":all_training_opportunities and all(primary[comp]["aggregate_pareto_dominates"] for comp in ("trained_frozen","rule_based")),
            "naive_online_is_descriptive_only":True}


def analyze(records: Iterator[dict[str,Any]],phase: str) -> dict[str,Any]:
    seeds=[]
    total_ticks=0
    for rec in records:
        total_ticks += rec.get("physical_ticks",0)
        if total_ticks > (1400 if phase=="calibration" else 11200):
            raise ValueError(f"{phase} invocation exceeds its physical tick ceiling")
        summary=validate_record(rec,rec["seed"],phase)
        summary["causal_audit"]=causal_audit(rec) if rec["status"]=="complete" else None
        if phase=="calibration": summary["calibration_gate"]=calibration_gate(rec)
        elif rec["status"]=="complete":
            summary["pareto_vs_trained_frozen"]={a:pareto_relation(summary["branches"][a],summary["branches"]["trained_frozen"]) for a in ("naive_online","trained_online","rule_based")}
            summary["pareto_vs_rule_based"]={a:pareto_relation(summary["branches"][a],summary["branches"]["rule_based"]) for a in ("naive_online","trained_online","trained_frozen")}
        else:
            summary["pareto_vs_trained_frozen"]={a:"insufficient_common_horizon" for a in TREATMENTS if a!="trained_frozen"}
            summary["pareto_vs_rule_based"]={a:"insufficient_common_horizon" for a in TREATMENTS if a!="rule_based"}
        seeds.append(summary)
    result={"schema":SUMMARY_SCHEMA,"protocol_version":PROTOCOL_VERSION,"role":phase,"status":"completed","seeds":seeds}
    if phase=="calibration":
        gate=seeds[0]["calibration_gate"]
        result["calibration_validity"]=gate
        result["status"]="completed" if gate["passed"] else "non_identifiable"
    else:
        result["aggregate_formal"]=aggregate_formal(seeds)
        result["mechanism_seed_count"]=sum(bool(s["causal_audit"] and s["causal_audit"]["mechanism_pass"]) for s in seeds)
        result["interpretation"]="Fixed eight-seed descriptive evidence; no general statistical or ecological claim."
    return result


def render_report(summary: dict[str,Any]) -> str:
    lines=["# V17 water residual lab", "",f"Phase: **{summary['role']}**. Status: **{summary['status']}**.","",
           "Physical outcomes come from resolver events. No weighted utility is calculated; accelerated ticks are not real elapsed lifetime.","",
           "| Seed | Arm | Ticks | Water | Movement energy | Survival ticks | Terminal E/W/H | Status |","|---:|---|---:|---:|---:|---:|---|---|"]
    for seed in summary["seeds"]:
        prefix=seed["prefix"]
        deaths=[]
        for actor_id,actor_summary in prefix.get("actors",{}).items():
            event=actor_summary.get("death_event")
            if event is not None:
                deaths.append(f"actor {actor_id} at tick {event.get('tick',prefix['ticks'])}: {event.get('cause','unknown cause')}")
        if deaths:
            lines.append(f"| {seed['seed']} | Prefix ended tick {prefix['ticks']}; {'; '.join(deaths)}; {prefix['episode_closure_count']} learner closures / {prefix['censored_episode_closures']} censored | — | — | — | — | — | biological negative |")
        else:
            lines.append(f"| {seed['seed']} | Prefix to tick {prefix['ticks']}; {prefix['fresh_two_positive_body_nonfull_choices']} fresh nonfull choices; {prefix['episode_closure_count']} learner closures / {prefix['censored_episode_closures']} censored | {prefix['ticks']} | {prefix['water_acquired']} | {prefix['movement_energy_paid']} | — | — | reached fork |")
        for arm,m in seed["branches"].items():
            if m is None:
                lines.append(f"| {seed['seed']} | {arm} | — | — | — | — | unknown | biological_negative_before_fork |")
                continue
            terminal=m["terminal_state"]
            term="unknown" if terminal is None else f"{terminal['energy']}/{terminal['water']}/{terminal['health']}"
            lines.append(f"| {seed['seed']} | {arm} | {m['ticks']} | {m['water_acquired']} | {m['movement_energy_paid']} | {m['observed_survival_ticks']} | {term} | {m['status']} |")
    if summary["role"]=="calibration":
        lines += ["","Calibration opportunity gate: **"+("passed" if summary["calibration_validity"]["passed"] else "failed")+"**."]
    else:
        lines += ["",f"Mechanism-qualified seeds: **{summary['mechanism_seed_count']}**.",
                  f"Bounded utility evidence: **{summary['aggregate_formal']['bounded_utility_evidence']}**.",
                  "Every per-seed relation and aggregate vector is retained in `summary.json`."]
    return "\n".join(lines)+"\n"


def atomic_json(path: Path,value: Any) -> None:
    tmp=path.with_suffix(path.suffix+".tmp")
    tmp.write_text(json.dumps(value,indent=2,sort_keys=True)+"\n",encoding="utf-8")
    tmp.replace(path)


def child_limits() -> None:
    resource.setrlimit(resource.RLIMIT_AS,(MAX_CHILD_MEMORY_BYTES,MAX_CHILD_MEMORY_BYTES))
    resource.setrlimit(resource.RLIMIT_FSIZE,(MAX_OUTPUT_BYTES,MAX_OUTPUT_BYTES))
    resource.setrlimit(resource.RLIMIT_CPU,(MAX_SECONDS+5,MAX_SECONDS+5))


def run_probe(command: list[str],cwd: Path,raw: Path,stderr: Path) -> dict[str,Any]:
    started=time.monotonic()
    try:
        with raw.open("xb") as out,stderr.open("xb") as err:
            result=subprocess.run(command,cwd=cwd,stdout=out,stderr=err,timeout=MAX_SECONDS,check=False,preexec_fn=child_limits)
        size=raw.stat().st_size
        return {"status":"completed" if result.returncode==0 and size<MAX_OUTPUT_BYTES else "failed",
                "exit_code":result.returncode,"raw_bytes":size,"stderr_bytes":stderr.stat().st_size,
                "output_limit_exceeded":size>=MAX_OUTPUT_BYTES,"elapsed_seconds":time.monotonic()-started}
    except Exception as error:
        return {"status":"failed","exit_code":None,"runner_error":f"{type(error).__name__}: {error}","elapsed_seconds":time.monotonic()-started}


def validate_calibration_parent(archive: Path,current: dict[str,Any]) -> dict[str,Any]:
    cal=(archive/OUTPUTS["calibration"]).resolve()
    manifest_path=cal/"manifest.json"
    raw_path=cal/"raw.jsonl"
    summary_path=cal/"summary.json"
    binary_path=cal/"water-residual-probe"
    protocol_copy=cal/"protocol.md"
    scanner_copy=cal/"seed-scanner.py"
    required=(manifest_path,raw_path,summary_path,binary_path,protocol_copy,scanner_copy)
    if any(not path.is_file() for path in required):
        raise RuntimeError("formal phase requires complete retained calibration evidence")
    cm=json.loads(manifest_path.read_text(encoding="utf-8"))
    cs=json.loads(summary_path.read_text(encoding="utf-8"))
    if cm.get("schema")!=MANIFEST_SCHEMA or cm.get("protocol_version")!=PROTOCOL_VERSION:
        raise RuntimeError("calibration manifest schema/version mismatch")
    if cm.get("role")!="calibration" or cm.get("seeds")!=list(RUNS["calibration"]):
        raise RuntimeError("calibration manifest phase/seed list mismatch")
    if cm.get("status")!="completed" or cm.get("analysis_status")!="completed":
        raise RuntimeError("formal phase requires a completed passing calibration")
    if cm.get("raw_sha256")!=sha(raw_path):
        raise RuntimeError("calibration raw hash mismatch")
    if cm.get("summary_sha256")!=sha(summary_path):
        raise RuntimeError("calibration summary hash mismatch")
    if cm.get("binary_sha256")!=sha(binary_path):
        raise RuntimeError("retained calibration probe hash mismatch")
    if cm.get("protocol_sha256")!=sha(protocol_copy):
        raise RuntimeError("retained calibration protocol hash mismatch")
    if cm.get("scanner_sha256")!=sha(scanner_copy):
        raise RuntimeError("retained calibration seed-scanner hash mismatch")
    if cm.get("report_sha256")!=(sha(cal/"report.md") if (cal/"report.md").is_file() else None):
        raise RuntimeError("retained calibration report hash mismatch")
    for key in ("revision","source_files_sha256","source_sha256","binary_sha256",
                "protocol_sha256","runner_sha256","scanner_sha256"):
        if cm.get(key)!=current.get(key):
            raise RuntimeError(f"formal candidate differs from calibration in {key}")
    if cs.get("schema")!=SUMMARY_SCHEMA or cs.get("protocol_version")!=PROTOCOL_VERSION or cs.get("role")!="calibration":
        raise RuntimeError("calibration summary schema/role mismatch")
    if cs.get("status")!="completed" or cs.get("calibration_validity",{}).get("passed") is not True:
        raise RuntimeError("calibration summary does not record a passing fixture gate")
    # Recompute every calibration metric/gate from the retained raw record.
    recalculated=analyze(parse_raw(raw_path,"calibration",RUNS["calibration"]),"calibration")
    if recalculated!=cs:
        raise RuntimeError("calibration summary/gate does not reproduce from raw evidence")
    return {"manifest_sha256":sha(manifest_path),"raw_sha256":cm["raw_sha256"],
            "summary_sha256":cm["summary_sha256"],"binary_sha256":cm["binary_sha256"],
            "protocol_sha256":cm["protocol_sha256"],"scanner_sha256":cm["scanner_sha256"],
            "calibration_validity":cs["calibration_validity"]}


def parse_raw(raw: Path,phase: str,seeds: tuple[int,...]) -> Iterator[dict[str,Any]]:
    with raw.open("rb") as stream:
        for i,seed in enumerate(seeds,1):
            line=stream.readline(MAX_SEED_RECORD_BYTES+1)
            if not line or len(line)>MAX_SEED_RECORD_BYTES or not line.endswith(b"\n"):
                raise ValueError(f"raw record {i} missing, oversized, or truncated")
            record=json.loads(line)
            if record.get("seed")!=seed or record.get("role")!=phase:
                raise ValueError("raw seed order/role mismatch")
            yield record
        if stream.read(1): raise ValueError("raw stream contains extra records")


def execute_phase(phase: str,binary: Path,archive: Path,source_root: Path,output: Path|None=None) -> dict[str,Any]:
    if phase not in RUNS: raise ValueError("phase must be calibration or formal")
    seeds=RUNS[phase]
    expected=(archive/OUTPUTS[phase]).resolve()
    out=(output or expected).resolve()
    if out!=expected: raise ValueError("output must be the fixed phase directory")
    if out.exists(): raise FileExistsError(f"refusing to overwrite existing experiment output: {out}")
    if archive.resolve()!=ARCHIVE.resolve(): raise ValueError("CLI must use the canonical experiment archive")
    if source_root.resolve()!=ROOT.resolve(): raise ValueError("CLI must use this source-bound checkout")
    binary=binary.resolve(strict=True)
    revision,hashes=source_identity(source_root)
    protocol=source_root/PROTOCOL
    scanner=source_root/SCANNER
    audit=scan_seeds(archive,seeds)
    if audit.get("unseen")!=list(seeds): raise RuntimeError(f"phase seed conflict; no replacement permitted: {audit.get('matches')}")
    command=["./water-residual-probe","--calibration" if phase=="calibration" else "--formal",",".join(map(str,seeds))]
    manifest={"schema":MANIFEST_SCHEMA,"protocol_version":PROTOCOL_VERSION,"role":phase,"seeds":list(seeds),
        "command":command,"status":"prepared","revision":revision,"git_dirty":False,
        "source_files_sha256":hashes,"source_sha256":canonical_sha(hashes),"binary_sha256":sha(binary),
        "protocol_sha256":sha(protocol),"runner_sha256":sha(Path(__file__).resolve()),"scanner_sha256":sha(scanner),
        "seed_audit":audit,"started_at":now(),"timeout_seconds":MAX_SECONDS,"max_output_bytes":MAX_OUTPUT_BYTES,
        "max_seed_record_bytes":MAX_SEED_RECORD_BYTES,"max_child_memory_bytes":MAX_CHILD_MEMORY_BYTES,
        "raw_file":"raw.jsonl","stderr_file":"stderr.log"}
    if phase=="formal":
        manifest["calibration_identity"]=validate_calibration_parent(archive,manifest)
    out.mkdir(parents=True,exist_ok=False)
    mp=out/"manifest.json"; atomic_json(mp,manifest)
    try:
        probe=out/"water-residual-probe"; shutil.copy2(binary,probe); probe.chmod(0o555)
        shutil.copy2(protocol,out/"protocol.md"); (out/"protocol.md").chmod(0o444)
        shutil.copy2(scanner,out/"seed-scanner.py"); (out/"seed-scanner.py").chmod(0o444)
        (out/"work").mkdir()
        manifest["status"]="running"; manifest["command"]=[str(probe),*command[1:]]; atomic_json(mp,manifest)
        run=run_probe(manifest["command"],out/"work",out/"raw.jsonl",out/"stderr.log"); manifest.update(run)
        if run["status"]!="completed": raise RuntimeError(run.get("runner_error",f"probe exit {run.get('exit_code')}"))
        if sha(probe)!=manifest["binary_sha256"] or source_hashes(source_root)!=hashes: raise RuntimeError("registered binary/source changed during run")
        records=parse_raw(out/"raw.jsonl",phase,seeds)
        summary=analyze(records,phase)
        atomic_json(out/"summary.json",summary); (out/"report.md").write_text(render_report(summary),encoding="utf-8")
        manifest.update(status="completed",analysis_status=summary["status"],raw_sha256=sha(out/"raw.jsonl"),
            summary_sha256=sha(out/"summary.json"),report_sha256=sha(out/"report.md"),finished_at=now())
    except Exception as error:
        manifest.update(status="failed",runner_error=f"{type(error).__name__}: {error}",finished_at=now())
        if (out/"raw.jsonl").is_file(): manifest["raw_sha256"]=sha(out/"raw.jsonl")
        atomic_json(mp,manifest)
        raise
    atomic_json(mp,manifest)
    return manifest


def main(argv: list[str]|None=None) -> int:
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("phase",choices=tuple(RUNS))
    parser.add_argument("--binary",type=Path,default=ROOT/"target/release/examples/water_residual_lab")
    parser.add_argument("--archive-root",type=Path,default=ARCHIVE)
    parser.add_argument("--source-root",type=Path,default=ROOT)
    parser.add_argument("--output",type=Path)
    args=parser.parse_args(argv)
    try:
        manifest=execute_phase(args.phase,args.binary,args.archive_root,args.source_root,args.output)
    except Exception as error:
        print(f"water residual runner failed: {type(error).__name__}: {error}",file=sys.stderr); return 1
    print(json.dumps({"status":manifest["status"],"role":manifest["role"],"output":str((args.output or args.archive_root/OUTPUTS[args.phase]).resolve())}))
    return 0 if manifest.get("analysis_status")=="completed" else 1


if __name__=="__main__":
    raise SystemExit(main())
