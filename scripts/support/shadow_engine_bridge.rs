//! Standalone, version-neutral checkpoint probe for comparing engine revisions.
//!
//! Compile this exact source in an external crate against either engine source.
//! It intentionally uses only the stable checkpoint/World API shared by V12/V19.

use curious::{simulation::checkpoint::State, simulation::SimulationEvent};
use serde_json::{json, Value};
use std::{collections::{BTreeMap, BTreeSet}, env, fs::{self, File}, io::{BufWriter, Write}, path::PathBuf};
use std::time::Instant;

fn fail(message: impl std::fmt::Display) -> ! {
    eprintln!("shadow_engine_bridge: {message}");
    std::process::exit(2)
}

fn event_name(event: &Value) -> &str {
    event.as_object().and_then(|object| object.keys().next()).map(String::as_str).unwrap_or("Unknown")
}

fn species_name(value: &Value) -> &str {
    value.as_str().unwrap_or("Unknown")
}

fn alive_by_species(state: &State) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for entity in &state.entities {
        if (entity.phase.is_active() || entity.phase.is_sleeping()) && entity.life.is_alive() {
            let species = serde_json::to_value(&entity.species).unwrap_or(Value::Null);
            *counts.entry(species_name(&species).to_owned()).or_insert(0) += 1;
        }
    }
    counts
}

fn state_value(state: &State) -> Value {
    serde_json::to_value(state).unwrap_or_else(|e| fail(format!("serialize State: {e}")))
}

fn hash_u64(bytes: &[u8]) -> String {
    // Stable FNV-1a fingerprint, explicitly not a cryptographic identity hash.
    let mut h = 0xcbf29ce484222325u64;
    for byte in bytes { h = (h ^ u64::from(*byte)).wrapping_mul(0x100000001b3); }
    format!("fnv1a64:{h:016x}")
}

fn main() {
    let started = Instant::now();
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 { fail("usage: shadow_engine_bridge CHECKPOINT.json OUTPUT_DIR TICKS"); }
    let checkpoint_path = PathBuf::from(&args[1]);
    let output_dir = PathBuf::from(&args[2]);
    let ticks: usize = args[3].parse().unwrap_or_else(|_| fail("TICKS must be an integer"));
    if ticks > 1000 { fail("TICKS must be <= 1000"); }
    if output_dir.exists() { fail("OUTPUT_DIR already exists; use a new directory"); }
    let input = fs::read(&checkpoint_path).unwrap_or_else(|e| fail(format!("read checkpoint: {e}")));
    let state: State = serde_json::from_slice(&input).unwrap_or_else(|e| fail(format!("parse checkpoint: {e}")));
    if state.entities.len() > 10_000 { fail("checkpoint exceeds 10,000 entity slots"); }
    fs::create_dir_all(&output_dir).unwrap_or_else(|e| fail(format!("create output: {e}")));
    let initial_alive: BTreeSet<usize> = state.entities.iter().filter(|e| (e.phase.is_active() || e.phase.is_sleeping()) && e.life.is_alive()).map(|e| e.id).collect();
    let mut survived_ticks: BTreeMap<usize, u64> = BTreeMap::new();
    let mut world = state.restore().unwrap_or_else(|e| fail(format!("restore checkpoint: {e}")));
    let initial_state = state_value(&State::capture(&world));
    let fork_at = if ticks >= 105 { 100 } else { 0 };
    let mut fork_state: Option<State> = None;
    let mut timeline = vec![json!({"tick": world.tick_counter, "alive_by_species": alive_by_species(&State::capture(&world)), "rng": world.rng.state()})];
    let event_file = File::create(output_dir.join("events.jsonl")).unwrap_or_else(|e| fail(format!("create events.jsonl: {e}")));
    let mut event_writer = BufWriter::new(event_file);
    let mut event_counts: BTreeMap<String, u64> = BTreeMap::new();
    let mut births: BTreeMap<String, u64> = BTreeMap::new();
    let mut deaths: BTreeMap<String, u64> = BTreeMap::new();
    let mut movement_count = 0u64;
    let mut movement_energy = 0u64;
    let mut attack_count = 0u64;
    let mut damage = 0u64;
    let mut validation_events: Vec<Vec<Value>> = Vec::new();
    let mut validation_states: Vec<Value> = Vec::new();

    for offset in 0..ticks {
        if offset == fork_at { fork_state = Some(State::capture(&world)); }
        let events = world.tick();
        for entity in &world.entities {
            if (entity.phase.is_active() || entity.phase.is_sleeping()) && entity.base.life().is_alive() {
                *survived_ticks.entry(entity.id).or_default() += 1;
            }
        }
        let serialized: Vec<Value> = events.iter().map(|e| serde_json::to_value(e).unwrap_or_else(|x| fail(format!("serialize event: {x}")))).collect();
        for event in &serialized {
            let name = event_name(event);
            *event_counts.entry(name.to_owned()).or_default() += 1;
            let data = event.get(name).unwrap_or(&Value::Null);
            match name {
                "EntityBorn" => *births.entry(species_name(&data["species"]).into()).or_default() += 1,
                "EntityDied" => *deaths.entry(species_name(&data["species"]).into()).or_default() += 1,
                "EntityMoved" => { movement_count += 1; movement_energy += data["energy_cost"].as_u64().unwrap_or(0); }
                "EntityAttacked" => { attack_count += 1; damage += data["damage"].as_u64().unwrap_or(0); }
                _ => {}
            }
        }
        serde_json::to_writer(&mut event_writer, &json!({"tick": world.tick_counter, "events": serialized}))
            .unwrap_or_else(|e| fail(format!("write events.jsonl: {e}")));
        event_writer.write_all(b"\n").unwrap_or_else(|e| fail(format!("write events.jsonl newline: {e}")));
        if offset >= fork_at && offset < fork_at + 5 {
            validation_events.push(serialized);
            validation_states.push(state_value(&State::capture(&world)));
        }
        if world.tick_counter % 100 == 0 || offset + 1 == ticks {
            let state = State::capture(&world);
            timeline.push(json!({"tick": state.tick, "alive_by_species": alive_by_species(&state), "rng": state.rng}));
        }
    }
    event_writer.flush().unwrap_or_else(|e| fail(format!("flush events.jsonl: {e}")));
    let final_state = State::capture(&world);
    let fork_checks = if ticks >= 5 {
        let fork = fork_state.unwrap_or_else(|| State::capture(&world));
        let mut resumed = fork.restore().unwrap_or_else(|e| fail(format!("restore fork: {e}")));
        let mut checks = Vec::new();
        for i in 0..5 {
            let events = resumed.tick();
            let observed: Vec<Value> = events.iter().map(|e| serde_json::to_value(e).unwrap()).collect();
            let expected = &validation_events[i];
            let state = State::capture(&resumed);
            let events_match = &observed == expected;
            let state_matches = state_value(&state) == validation_states[i];
            checks.push(json!({"tick": state.tick, "events_match": events_match, "state_matches": state_matches, "rng": state.rng, "state_hash": hash_u64(state_value(&state).to_string().as_bytes())}));
        }
        checks
    } else { Vec::new() };

    let checkpoint_resume_exact = ticks >= 5 && fork_checks.iter().all(|check|
        check["events_match"] == true && check["state_matches"] == true);

    write_result(&output_dir, &checkpoint_path, ticks, &initial_state, &initial_alive,
        &final_state, &survived_ticks, &timeline, &event_counts, &births, &deaths,
        movement_count, movement_energy, attack_count, damage, fork_checks, checkpoint_resume_exact,
        started.elapsed().as_millis());
}

#[allow(clippy::too_many_arguments)]
fn write_result(output_dir: &std::path::Path, checkpoint_path: &std::path::Path, ticks: usize,
    initial_state: &Value, initial_alive: &BTreeSet<usize>, final_state: &State,
    survived_ticks: &BTreeMap<usize, u64>, timeline: &[Value],
    event_counts: &BTreeMap<String, u64>, births: &BTreeMap<String, u64>, deaths: &BTreeMap<String, u64>,
    movement_count: u64, movement_energy: u64, attack_count: u64, damage: u64,
    fork_checks: Vec<Value>, checkpoint_resume_exact: bool, elapsed_ms: u128) {
    let final_json = state_value(final_state);
    let initial_rng = initial_state["rng"].clone();
    let result = json!({
        "protocol": "shadow_engine_bridge_v1",
        "checkpoint_file": checkpoint_path.file_name().unwrap_or_default(),
        "ticks_requested": ticks,
        "elapsed_ms": elapsed_ms,
        "ticks_executed": final_state.tick.saturating_sub(initial_state["tick"].as_u64().unwrap_or(0) as usize),
        "validation_tick_calls_added": if ticks >= 5 { 5 } else { 0 },
        "checkpoint_resume_exact": checkpoint_resume_exact,
        "initial_state": initial_state,
        "final_state": final_json,
        "initial_rng": initial_rng,
        "final_rng": final_state.rng,
        "initial_state_fnv1a64": hash_u64(initial_state.to_string().as_bytes()),
        "final_state_fnv1a64": hash_u64(final_json.to_string().as_bytes()),
        "initial_alive_count": initial_alive.len(),
        "final_alive_by_species": alive_by_species(final_state),
        "survived_ticks_by_entity": survived_ticks,
        "event_counts": event_counts,
        "births_by_species": births,
        "deaths_by_species": deaths,
        "movement": {"events": movement_count, "energy_cost_sum": movement_energy},
        "attacks": {"events": attack_count, "damage_sum": damage},
        "timeline": timeline,
        "fork_validation_ticks": fork_checks,
    });
    fs::write(output_dir.join("summary.json"), serde_json::to_vec_pretty(&result).unwrap()).unwrap();
}
