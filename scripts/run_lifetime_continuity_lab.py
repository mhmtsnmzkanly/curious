#!/usr/bin/env python3
"""Run and validate the preregistered same-individual lifetime lab once."""
from copy import deepcopy
from datetime import datetime, timezone
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess

SEEDS = [44001, 44002, 44003, 44004, 44005, 44006, 44007, 44008]
TICK_CEILING = 1664
SCHEMA = "curious-lifetime-lab-run-v1"


def sha(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            h.update(block)
    return h.hexdigest()


def actor(state, actor_id):
    return next((x for x in state["entities"] if x["id"] == actor_id), None)


def assert_age_and_identity(rows, initial, label, initial_tick=0):
    subject = actor(initial, 1)
    if subject is None or type(subject["life"].get("age")) is not int:
        raise ValueError(f"{label}: subject must have a valid age")
    if initial_tick == 0 and subject["life"]["age"] != 0:
        raise ValueError(f"{label}: subject must start at age zero")
    previous_age = subject["life"]["age"]
    previous_tick = initial_tick
    for row in rows:
        tick = row.get("tick")
        if type(tick) is not int or tick != previous_tick + 1:
            raise ValueError(f"{label}: non-contiguous tick timeline")
        current_state = row.get("state")
        current = actor(current_state, 1)
        if current is None:
            raise ValueError(f"{label}: same actor ID disappeared from saved state")
        age = current["life"].get("age")
        if type(age) is not int or age != previous_age + 1:
            raise ValueError(f"{label}: age did not progress exactly once without reset")
        if current["species"] != subject["species"] or current["generation"] != subject["generation"]:
            raise ValueError(f"{label}: individual identity/species/generation changed")
        if row.get("controller") not in ({"direction": 1}, {"direction": -1}):
            raise ValueError(f"{label}: invalid external predator-route cursor")
        previous_age, previous_tick = age, tick


def update_flag_state(state, enabled):
    value = deepcopy(state)
    subject = actor(value, 1)
    if subject is None:
        raise ValueError("subject missing from branch-start state")
    subject["learner"]["update_enabled"] = enabled
    return value


def validate_interventions(rows, label, has_checkpoint=False):
    by_tick = {row.get("tick"): row for row in rows}
    expectations = {
        52: ["reverse_water_before_53", "reverse_predator_route_before_53", "capture_checkpoint_before_53"],
        78: ["reverse_water_before_79"],
    }
    for tick, expected in expectations.items():
        if tick == 52 and not has_checkpoint:
            continue
        row = by_tick.get(tick)
        if row is not None and row.get("interventions_after") != expected:
            raise ValueError(f"{label}: missing or reordered post-tick-{tick} intervention")


def event_payloads(row):
    events = row.get("events")
    if not isinstance(events, list):
        raise ValueError("tick row events must be a list")
    for event in events:
        if not isinstance(event, dict):
            raise ValueError("invalid serialized simulation event")
        yield from event.items()


def has_subject_death(rows):
    return any(
        kind == "EntityDied" and payload.get("id") == 1
        for row in rows for kind, payload in event_payloads(row)
    )


def validate_terminal_status(status, rows):
    if not rows:
        raise ValueError("subject timeline cannot be empty")
    last = rows[-1]
    subject = actor(last.get("state"), 1)
    if subject is None:
        raise ValueError("subject missing from terminal world state")
    dead = subject["life"].get("health") == 0
    died_event = has_subject_death(rows)
    tick = last["tick"]
    if status == "completed" and (tick != 104 or dead or died_event):
        raise ValueError("completed status requires a living subject at tick 104")
    if status in ("died_before_checkpoint", "died_at_checkpoint", "died_after_checkpoint"):
        if not dead or not died_event:
            raise ValueError("death status requires zero health and the subject's EntityDied event")
        if status == "died_before_checkpoint" and tick >= 52:
            raise ValueError("pre-checkpoint death must occur before tick 52")
        if status == "died_at_checkpoint" and tick != 52:
            raise ValueError("at-checkpoint death must end at tick 52")
        if status == "died_after_checkpoint" and not 52 <= tick < 104:
            raise ValueError("post-checkpoint death must occur after tick 52 and before completion")


def learner_table(state):
    subject = actor(state, 1)
    if subject is None:
        raise ValueError("learner snapshot missing subject")
    values = subject["learner"].get("values")
    if not isinstance(values, list):
        raise ValueError("learner table must be a list")
    return {
        (row["pos"]["x"], row["pos"]["y"]):
            {"value": row["value"], "visits": row["visits"]}
        for row in values
    }


def table_delta(before, after):
    left, right = learner_table(before), learner_table(after)
    result = []
    for x, y in sorted(set(left) | set(right)):
        old, new = left.get((x, y), {"value": 0, "visits": 0}), right.get((x, y), {"value": 0, "visits": 0})
        if old != new:
            result.append({
                "position": {"x": x, "y": y},
                "value_delta": new["value"] - old["value"],
                "visits_delta": new["visits"] - old["visits"],
            })
    return result


def branch_windows(rows, initial_state):
    by_tick = {row["tick"]: row for row in rows}
    ranges = [(1, 52), (53, 78), (79, 104)]
    result = []
    for start, end in ranges:
        selected = [row for row in rows if start <= row["tick"] <= end]
        if start == 1:
            prior = initial_state
        else:
            previous = [tick for tick in by_tick if tick <= start - 1]
            prior = by_tick[max(previous)]["state"] if previous else initial_state
        final = selected[-1]["state"] if selected else prior
        water = damage = movement_energy = 0
        for row in selected:
            for kind, payload in event_payloads(row):
                if kind == "EntityDrank" and payload.get("id") == 1:
                    water += payload.get("amount", 0)
                elif kind == "EntityMoved" and payload.get("id") == 1:
                    movement_energy += payload.get("energy_cost", 0)
                elif kind == "EntityAttacked" and payload.get("target") == 1:
                    damage += payload.get("damage", 0)
        first_actor, final_actor = actor(prior, 1), actor(final, 1)
        result.append({
            "ticks": [start, end],
            "observed_ticks": len(selected),
            "water_acquired": water,
            "movement_energy": movement_energy,
            "received_damage": damage,
            "health_start": first_actor["life"]["health"],
            "health_end": final_actor["life"]["health"],
            "energy_start": first_actor["life"]["energy"],
            "energy_end": final_actor["life"]["energy"],
            "learner_table_delta": table_delta(prior, final),
        })
    return result


def validate_seed(row, expected_seed):
    if row.get("schema") != "curious-lifetime-lab-v1" or row.get("seed") != expected_seed:
        raise ValueError("lifetime lab schema/seed mismatch")
    if row.get("status") not in (
        "completed", "died_before_checkpoint", "died_at_checkpoint", "died_after_checkpoint"
    ):
        raise ValueError("invalid terminal subject status")
    initial = row.get("initial_state")
    if not isinstance(initial, dict) or initial.get("tick") != 0:
        raise ValueError("missing initial state")
    uninterrupted = row.get("uninterrupted", [])
    assert_age_and_identity(uninterrupted, initial, "uninterrupted")
    if len(uninterrupted) > 104:
        raise ValueError("uninterrupted branch exceeded 104 ticks")
    validate_terminal_status(row["status"], uninterrupted)
    checkpoint = row.get("checkpoint_state")
    resumed_initial = row.get("resumed_initial_state")
    frozen_initial = row.get("frozen_initial_state")
    resumed, frozen = row.get("resumed", []), row.get("frozen", [])
    if checkpoint is None:
        final_tick = uninterrupted[-1]["tick"] if uninterrupted else 0
        expected_status = "died_at_checkpoint" if final_tick == 52 else "died_before_checkpoint"
        if row["status"] != expected_status or final_tick > 52:
            raise ValueError("missing tick-52 checkpoint without a pre-checkpoint death")
        if any((resumed_initial is not None, frozen_initial is not None, resumed, frozen)):
            raise ValueError("a dead pre-checkpoint subject must not be resumed/replaced")
        validate_interventions(uninterrupted, "uninterrupted")
    else:
        if row["status"] == "died_before_checkpoint" or checkpoint.get("tick") != 52:
            raise ValueError("invalid checkpoint boundary/status")
        cp_subject = actor(checkpoint, 1)
        if cp_subject is None or cp_subject["life"]["age"] != 52:
            raise ValueError("checkpoint lost the subject or reset age")
        if row.get("checkpoint_controller") != {"direction": -1}:
            raise ValueError("checkpoint must retain reversed predator route cursor")
        if resumed_initial != checkpoint:
            raise ValueError("restored online branch did not start from exact checkpoint state")
        if frozen_initial != update_flag_state(checkpoint, False):
            raise ValueError("frozen branch changed state beyond the registered learner update flag")
        assert_age_and_identity(resumed, checkpoint, "resumed", initial_tick=52)
        assert_age_and_identity(frozen, frozen_initial, "frozen", initial_tick=52)
        if len(resumed) > 52 or len(frozen) > 52:
            raise ValueError("post-checkpoint branch exceeded 52 ticks")
        if frozen_initial["entities"] != update_flag_state(checkpoint, False)["entities"]:
            raise ValueError("frozen branch did not retain checkpoint learner table/visits")
        initial_subject = actor(frozen_initial, 1)
        for frozen_row in frozen:
            frozen_subject = actor(frozen_row["state"], 1)
            if frozen_subject["learner"].get("values") != initial_subject["learner"].get("values"):
                raise ValueError("frozen branch changed learned values or visits")
        online_after = [x for x in row["uninterrupted"] if x["tick"] > 52]
        if online_after != resumed:
            raise ValueError("restored branch differs from uninterrupted events/decisions/state/RNG")
        if len(online_after) != len(resumed):
            raise ValueError("replay continuation length mismatch")
        if len(row["uninterrupted"]) < 52:
            raise ValueError("checkpoint recorded before 52 world ticks")
        validate_interventions(uninterrupted, "uninterrupted", has_checkpoint=True)
        validate_interventions(resumed, "resumed")
        validate_interventions(frozen, "frozen")
    total = sum(len(row[k]) for k in ("uninterrupted", "resumed", "frozen"))
    if row.get("physical_ticks") != total or total > 208:
        raise ValueError("physical tick audit/registered ceiling mismatch")
    return {
        "seed": expected_seed,
        "status": row["status"],
        "physical_ticks": total,
        "uninterrupted_ticks": len(row["uninterrupted"]),
        "resumed_ticks": len(resumed),
        "frozen_ticks": len(frozen),
        "checkpoint_available": checkpoint is not None,
        "final_age": actor(uninterrupted[-1]["state"], 1)["life"]["age"]
            if uninterrupted else 0,
        "branches": {
            "uninterrupted": branch_windows(uninterrupted, initial),
            "resumed": branch_windows(resumed, checkpoint) if checkpoint is not None else [],
            "frozen": branch_windows(frozen, frozen_initial) if checkpoint is not None else [],
        },
    }


def summarize(raw_path: Path):
    results = []
    with raw_path.open(encoding="utf-8") as stream:
        for seed in SEEDS:
            line = stream.readline()
            if not line:
                raise ValueError("missing fixed-seed lifetime result")
            results.append(validate_seed(json.loads(line), seed))
        if stream.read().strip():
            raise ValueError("unexpected extra lifetime lab output")
    physical_ticks = sum(x["physical_ticks"] for x in results)
    if physical_ticks > TICK_CEILING:
        raise ValueError("aggregate lifetime study exceeded 1,664 physical ticks")
    return {
        "schema": "curious-lifetime-lab-summary-v1",
        "seeds": results,
        "physical_ticks": physical_ticks,
        "tick_ceiling": TICK_CEILING,
        "interpretation": "Identity/state continuity and deterministic checkpoint replay are audited separately from descriptive online/frozen behavior; no lifespan-learning acceptance is implied.",
    }


def render_report(summary):
    lines = [
        "# Lifetime continuity and reversal — bounded report",
        "",
        "This report audits whether the same actor identity and body/age/learner state persist through the registered resource/predator reversal and exact checkpoint replay. The subject is never replaced after death.",
        "",
        "| Seed | Status | Ticks | Age at last observed tick | Checkpoint | Online | Resumed | Frozen |",
        "|---:|---|---:|---:|---|---:|---:|---:|",
    ]
    for row in summary["seeds"]:
        lines.append(
            f"| {row['seed']} | {row['status']} | {row['physical_ticks']} | {row['final_age']} | "
            f"{str(row['checkpoint_available']).lower()} | {row['uninterrupted_ticks']} | "
            f"{row['resumed_ticks']} | {row['frozen_ticks']} |"
        )
    lines += [
        "",
        "## Observed resource, cost, and learner windows",
        "",
        "Water and injury are read from resolver events; movement energy is the actor's recorded locomotion cost. Learner table changes are shown as observed value/visit deltas. Missing ticks after natural death are not treated as zero exposure.",
        "",
        "| Seed | Branch | Window | Ticks observed | Water | Damage received | Move energy | Health start→end | Energy start→end | Learner table delta |",
        "|---:|---|---:|---:|---:|---:|---:|---:|---:|---|",
    ]
    for row in summary["seeds"]:
        for branch, windows in row["branches"].items():
            for window in windows:
                lines.append(
                    f"| {row['seed']} | {branch} | {window['ticks'][0]}–{window['ticks'][1]} | "
                    f"{window['observed_ticks']} | {window['water_acquired']} | "
                    f"{window['received_damage']} | {window['movement_energy']} | "
                    f"{window['health_start']}→{window['health_end']} | "
                    f"{window['energy_start']}→{window['energy_end']} | "
                    f"{json.dumps(window['learner_table_delta'], sort_keys=True)} |"
                )
    lines += [
        "",
        f"Total physical ticks: {summary['physical_ticks']} / {TICK_CEILING}.",
        "",
        "Online and restored-online branches are required to match in events, decisions, complete serialized state, and RNG after every post-checkpoint tick. The frozen branch differs only in its water learner update flag at the checkpoint. Descriptive behavioral differences are not a deployment criterion.",
    ]
    return "\n".join(lines) + "\n"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    repo = Path(__file__).resolve().parents[1]
    binary = args.binary.resolve(strict=True)
    output = args.output.resolve()
    if subprocess.check_output(["git", "status", "--porcelain"], cwd=repo, text=True).strip():
        raise ValueError("commit clean source before registered run")
    output.mkdir(parents=True, exist_ok=False)
    probe = output / "lab-probe"
    shutil.copy2(binary, probe)
    protocol = repo / "docs/LIFETIME_CONTINUITY_PROTOCOL.md"
    shutil.copy2(protocol, output / "protocol.md")
    work = output / "work"
    work.mkdir()
    command = [str(probe)]
    sources = [
        repo / "examples/lifetime_continuity_lab.rs",
        repo / "scripts/run_lifetime_continuity_lab.py",
        repo / "Cargo.toml",
        repo / "Cargo.lock",
    ]
    manifest = {
        "schema": SCHEMA,
        "protocol_version": 1,
        "seeds": SEEDS,
        "command": command,
        "status": "running",
        "revision": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=repo, text=True).strip(),
        "git_dirty": False,
        "binary_sha256": sha(probe),
        "artifact_name": "lab-probe",
        "protocol_sha256": sha(protocol),
        "sources": {str(path.relative_to(repo)): sha(path) for path in sources},
        "started_at": datetime.now(timezone.utc).isoformat(),
        "timeout_seconds": 300,
        "tick_ceiling": TICK_CEILING,
    }
    manifest_path = output / "manifest.json"
    manifest_path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    try:
        with (output / "raw.jsonl").open("x", encoding="utf-8") as raw, (output / "stderr.log").open("x", encoding="utf-8") as err:
            subprocess.run(command, cwd=work, stdout=raw, stderr=err, check=True, timeout=300)
        summary = summarize(output / "raw.jsonl")
        (output / "summary.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
        (output / "report.md").write_text(render_report(summary), encoding="utf-8")
        manifest.update(
            status="completed",
            raw_sha256=sha(output / "raw.jsonl"),
            summary_sha256=sha(output / "summary.json"),
            report_sha256=sha(output / "report.md"),
        )
    except Exception as exc:
        manifest.update(status="failed", error=str(exc), finished_at=datetime.now(timezone.utc).isoformat())
        if (output / "raw.jsonl").exists():
            manifest["raw_sha256"] = sha(output / "raw.jsonl")
        if (output / "stderr.log").exists():
            manifest["stderr_sha256"] = sha(output / "stderr.log")
        manifest_path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
        raise
    manifest["finished_at"] = datetime.now(timezone.utc).isoformat()
    manifest_path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"status": manifest["status"], "output": str(output)}))


if __name__ == "__main__":
    main()
