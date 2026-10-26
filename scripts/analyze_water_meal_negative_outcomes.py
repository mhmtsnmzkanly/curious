#!/usr/bin/env python3
"""Bounded offline diagnosis of immutable water-meal V1 JSONL evidence."""
import argparse
import hashlib
import json
from collections import Counter
from pathlib import Path

ARMS = ("control", "food_reserve_only", "water_meal_only", "bundle")
FOCUS = {"meadow": (62502, 62503, 62505), "sparse_resource": (62503, 62505)}


def read_run(root, habitat, seed, arm, entry):
    path = root / f"{habitat}-seed{seed}-{arm}.jsonl"
    raw = path.read_bytes()
    digest = hashlib.sha256(raw).hexdigest()
    if len(raw) != entry["raw_bytes"] or digest != entry["raw_sha256"]:
        raise ValueError(f"raw file fails manifest size/hash: {path}")
    rows = [json.loads(line) for line in raw.splitlines()]
    if len(rows) < 3 or rows[0].get("type") != "header" or rows[-1].get("type") != "summary":
        raise ValueError(f"invalid or incomplete stream: {path}")
    ticks = rows[1:-1]
    if len(ticks) != 300 or any(row.get("tick") != i for i, row in enumerate(ticks, 1)):
        raise ValueError(f"unexpected tick sequence: {path}")
    if (rows[0].get("habitat"), rows[0].get("seed"), rows[0].get("arm")) != (habitat, seed, arm):
        raise ValueError(f"header identity mismatch: {path}")
    summary = rows[-1]
    observed = {
        "plant_food_gained_energy": sum(e["EntityAte"]["amount"] for tick in ticks for item in tick.get("actor_resolver_events", [])
            for e in [item["event"]] if "EntityAte" in e and e["EntityAte"].get("corpse_id") is None),
        "water_gained": sum(e["EntityDrank"]["amount"] for tick in ticks for item in tick.get("actor_resolver_events", [])
            for e in [item["event"]] if "EntityDrank" in e),
        "movement_energy_paid": sum(e["EntityMoved"]["energy_cost"] for tick in ticks for item in tick.get("actor_resolver_events", [])
            for e in [item["event"]] if "EntityMoved" in e),
        "water_meal_intervention_count": sum(len(tick.get("water_meal_interventions", [])) for tick in ticks),
        "living_entity_ticks": sum(sum(tick.get("alive", {}).values()) for tick in ticks),
    }
    for key, value in observed.items():
        if summary.get(key) != value:
            raise ValueError(f"resolver/tick accounting does not match summary {key}: {path}")
    return rows[0], ticks, summary, {"file": path.name, "bytes": len(raw), "sha256": digest}


def resolver_events(tick):
    return [(x.get("id"), x.get("event")) for x in tick.get("actor_resolver_events", [])]


def first_divergence(left, right, field):
    for tick, (a, b) in enumerate(zip(left, right), 1):
        if a.get(field) != b.get(field):
            return tick
    return None


def lineage_events(ticks, key):
    return [(row["tick"], event) for row in ticks for event in row.get(key, [])]


def alive_exposure(ticks, stop):
    return sum(sum(row.get("alive", {}).values()) for row in ticks[:stop])


def eat_outcomes(tick):
    meals, rejected = {}, Counter()
    for item in tick.get("actor_resolver_events", []):
        event = item.get("event", {})
        if "EntityAte" in event and event["EntityAte"].get("corpse_id") is None:
            meals[item["id"]] = meals.get(item["id"], 0) + event["EntityAte"].get("amount", 0)
        elif "ActionRejected" in event and event["ActionRejected"].get("action") == "Eat":
            rejected[event["ActionRejected"].get("reason", "Unknown")] += 1
    return {"plant_meal_energy_by_actor": meals, "rejected_eat_counts_by_reason": dict(sorted(rejected.items()))}


def first_lineage_mismatch(left, right):
    for a, b in zip(left, right):
        if a != b:
            return {"control": {"tick": a[0], "event": a[1]}, "water_meal_only": {"tick": b[0], "event": b[1]}}
    if len(left) != len(right):
        a = left[len(right)] if len(left) > len(right) else None
        b = right[len(left)] if len(right) > len(left) else None
        return {"control": {"tick": a[0], "event": a[1]} if a else None,
                "water_meal_only": {"tick": b[0], "event": b[1]} if b else None}
    return None


def death_causes(ticks):
    return dict(sorted(Counter(x.get("cause", "Unknown") for _, x in lineage_events(ticks, "death_lineage")).items()))


def intervention_outcomes(ticks):
    result = Counter()
    meal_energy = 0
    for tick in ticks:
        events = tick.get("actor_resolver_events", [])
        for intervention in tick.get("water_meal_interventions", []):
            actor = intervention["id"]
            actor_events = [item.get("event", {}) for item in events if item.get("id") == actor]
            ate = [event["EntityAte"] for event in actor_events if "EntityAte" in event and event["EntityAte"].get("corpse_id") is None]
            rejected = [event["ActionRejected"] for event in actor_events if "ActionRejected" in event and event["ActionRejected"].get("action") == "Eat"]
            if ate:
                result["accepted"] += 1
                meal_energy += sum(event.get("amount", 0) for event in ate)
            elif rejected:
                result["rejected_" + rejected[0].get("reason", "Unknown")] += 1
            else:
                result["unresolved"] += 1
    return {"attempts": sum(result.values()), "accepted": result.pop("accepted", 0),
            "accepted_meal_energy": meal_energy, **dict(sorted(result.items()))}


def summarize(root):
    manifest = json.loads((root / "manifest.json").read_text())
    entries = {(x["habitat"], x["seed"], x["arm"]): x for x in manifest["worlds"]}
    if len(entries) != 72:
        raise ValueError("manifest must contain 72 unique Worlds")
    all_runs = {}
    for habitat in ("v1_unique_food", "meadow", "sparse_resource"):
        for seed in range(62501, 62507):
            for arm in ARMS:
                key = habitat, seed, arm
                if key not in entries:
                    raise ValueError(f"manifest omits World {key}")
                all_runs[key] = read_run(root, habitat, seed, arm, entries[key])

    denominators = {}
    for habitat in ("v1_unique_food", "meadow", "sparse_resource"):
        denominators[habitat] = {}
        for arm in ARMS:
            records = [all_runs[(habitat, seed, arm)] for seed in range(62501, 62507)]
            totals = [sum(x[2][k] for x in records)
                      for k in ("living_entity_ticks", "plant_food_gained_energy", "water_gained", "movement_energy_paid", "water_meal_intervention_count")]
            causes = Counter()
            for _, ticks, _, _ in records:
                causes.update(death_causes(ticks))
            denominators[habitat][arm] = {
                "worlds": 6, "ticks_each": 300, "living_entity_ticks": totals[0],
                "plant_food_energy": totals[1], "water": totals[2], "movement_energy": totals[3],
                "interventions": totals[4], "intervention_resolver_outcomes": {
                    key: sum(intervention_outcomes(ticks).get(key, 0) for _, ticks, _, _ in records)
                    for key in sorted(set().union(*(intervention_outcomes(ticks).keys() for _, ticks, _, _ in records)))},
                "endpoint_alive_worlds": sum(sum(x[2]["final_alive"].values()) > 0 for x in records),
                "death_causes": dict(sorted(causes.items())),
                "per_seed": [{"seed": seed, "living_entity_ticks": run[2]["living_entity_ticks"],
                    "plant_food_energy": run[2]["plant_food_gained_energy"], "water": run[2]["water_gained"],
                    "movement_energy": run[2]["movement_energy_paid"],
                    "endpoint_alive": sum(run[2]["final_alive"].values())}
                    for seed, run in zip(range(62501, 62507), records)],
            }

    cases = []
    for habitat, seeds in FOCUS.items():
        for seed in seeds:
            _, control, csum, _ = all_runs[(habitat, seed, "control")]
            _, treated, tsum, _ = all_runs[(habitat, seed, "water_meal_only")]
            interventions = [(r["tick"], x) for r in treated for x in r.get("water_meal_interventions", [])]
            first_intervention = interventions[0][0] if interventions else None
            focal_divergence = first_divergence(control, treated, "focal_decision_trace")
            event_divergence = first_divergence(control, treated, "actor_resolver_events")
            birth_c, birth_t = lineage_events(control, "birth_lineage"), lineage_events(treated, "birth_lineage")
            death_c, death_t = lineage_events(control, "death_lineage"), lineage_events(treated, "death_lineage")
            birth_diff = first_lineage_mismatch(birth_c, birth_t)
            death_diff = first_lineage_mismatch(death_c, death_t)
            birth_divergence = min((x["tick"] for x in birth_diff.values() if x), default=None) if birth_diff else None
            death_divergence = min((x["tick"] for x in death_diff.values() if x), default=None) if death_diff else None
            first = first_intervention
            actor = interventions[0][1].get("id") if interventions else None
            rows_at_first = next((r for r in treated if r["tick"] == first), {})
            actor_actions = [x for x in rows_at_first.get("actor_resolver_events", []) if x.get("id") == actor]
            control_at_first = next((r for r in control if r["tick"] == first), {})
            first_death_diff = death_diff
            first_birth_diff = birth_diff
            before = max((first_intervention or 1) - 1, 0)
            direct_actor = {
                "events": actor_actions,
                "accepted_plant_meal_energy": sum(ev["EntityAte"].get("amount", 0) for item in actor_actions
                    for ev in [item.get("event", {})] if "EntityAte" in ev and ev["EntityAte"].get("corpse_id") is None),
                "rejected_eat": any("ActionRejected" in item.get("event", {}) and
                    item["event"]["ActionRejected"].get("action") == "Eat" for item in actor_actions),
            }
            cases.append({
                "habitat": habitat, "seed": seed, "comparison": "water_meal_only - control",
                "living_tick_delta": tsum.get("living_entity_ticks", 0) - csum.get("living_entity_ticks", 0),
                "endpoint_alive": {"control": csum.get("final_alive"), "water_meal_only": tsum.get("final_alive")},
                "first_intervention_tick": first_intervention,
                "first_intervention_actor": actor,
                "first_intervention": interventions[0][1] if interventions else None,
                "direct_actor_resolution": direct_actor,
                "all_actor_food_resolution_at_intervention_tick": {
                    "control": eat_outcomes(control_at_first), "water_meal_only": eat_outcomes(rows_at_first)},
                "population_survival_exposure": {
                    "living_individual_ticks_before_intervention": {"control": alive_exposure(control, before), "water_meal_only": alive_exposure(treated, before)},
                    "living_individual_ticks_from_intervention_through_horizon": {
                        "control": csum.get("living_entity_ticks", 0) - alive_exposure(control, before),
                        "water_meal_only": tsum.get("living_entity_ticks", 0) - alive_exposure(treated, before)},
                },
                "first_focal_trace_divergence_tick": focal_divergence,
                "first_resolver_event_list_divergence_tick": event_divergence,
                "first_birth_lineage_divergence_tick": birth_divergence,
                "first_death_lineage_divergence_tick": death_divergence,
                "first_birth_lineage_mismatch": first_birth_diff,
                "first_death_lineage_mismatch": first_death_diff,
                "control_focal_death": {"tick": csum.get("focal_death_tick"), "cause": csum.get("focal_death_cause")},
                "treated_focal_death": {"tick": tsum.get("focal_death_tick"), "cause": tsum.get("focal_death_cause")},
                "death_causes": {"control": death_causes(control), "water_meal_only": death_causes(treated)},
                "physical_totals": {arm: {k: all_runs[(habitat, seed, arm)][2][k] for k in
                    ("plant_food_gained_energy", "water_gained", "movement_energy_paid", "water_meal_intervention_count")}
                    for arm in ("control", "water_meal_only")},
                "lineage_signature_note": "Entity IDs are stable only while birth tick, generation, and parent IDs remain equal; post-divergence equal IDs are not presumed same individual.",
            })
    raw_inputs = [all_runs[key][3] for key in sorted(all_runs)]
    return {"source": str(root), "experiment_source_revision": manifest["source"].get("revision"),
            "experiment_source_content_sha256": manifest["source"].get("content_sha256"),
            "protocol_sha256": manifest["protocol_sha256"], "binary_sha256": manifest["binary_sha256"],
            "world_count": 72, "ticks_per_world": 300, "raw_inputs": raw_inputs,
            "analysis": "descriptive offline parsing; no simulation and no causal identification from aggregate association",
            "denominators": denominators, "focused_negative_seed_pairs": cases}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--input", type=Path, required=True, help="immutable V1 raw JSONL directory")
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    report = summarize(args.input)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n")
    print(f"wrote {args.output} ({args.output.stat().st_size} bytes)")


if __name__ == "__main__":
    main()
