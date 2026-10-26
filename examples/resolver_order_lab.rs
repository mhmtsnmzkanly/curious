//! One-case executable for the frozen resolver-order V1 protocol.
use curious::{simulation::checkpoint::State, world::World};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    io::Write,
    path::Path,
    process::{Command, Stdio},
};

const MAX_INPUT: u64 = 64 * 1024 * 1024;
const MAX_OUTPUT: usize = 64 * 1024 * 1024;
type AppResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

fn main() {
    if let Err(error) = run() {
        eprintln!("resolver_order_lab: {error}");
        std::process::exit(1);
    }
}

fn run() -> AppResult<()> {
    let args: Vec<_> = env::args().collect();
    if args.len() != 5 {
        return Err(
            "usage: resolver_order_lab INPUT_CONTROL.jsonl TARGET_TICK INPUT_SHA256 OUTPUT.json"
                .into(),
        );
    }
    let input_path = Path::new(&args[1]);
    let target_tick: usize = args[2].parse()?;
    let input_sha256 = &args[3];
    if input_sha256.len() != 64
        || !input_sha256
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    {
        return Err("INPUT_SHA256 must be 64 lowercase hexadecimal characters".into());
    }
    let output_path = Path::new(&args[4]);
    if output_path.exists() {
        return Err("output path already exists".into());
    }
    let metadata = fs::metadata(input_path)?;
    if metadata.len() > MAX_INPUT {
        return Err("input exceeds 64 MiB cap".into());
    }
    let bytes = fs::read(input_path)?;
    let actual_sha256 = sha256_bytes(&bytes)?;
    if actual_sha256 != *input_sha256 {
        return Err("supplied input SHA-256 does not match raw bytes".into());
    }
    let text = std::str::from_utf8(&bytes)?;
    let (header, row_by_tick, _summary) = parse_raw_jsonl(text)?;
    let habitat = header
        .get("habitat")
        .and_then(Value::as_str)
        .ok_or("header missing habitat")?
        .to_owned();
    let seed = header
        .get("seed")
        .and_then(Value::as_u64)
        .ok_or("header missing seed")?;
    if header.get("arm").and_then(Value::as_str) != Some("control")
        || header
            .pointer("/candidate_policy/water_meal_priority_enabled")
            .and_then(Value::as_bool)
            != Some(false)
    {
        return Err("input must be the original water-meal-off control".into());
    }
    if target_tick < 1 || row_by_tick.len() < target_tick {
        return Err("requested prefix is absent from raw input".into());
    }
    let initial_value = header
        .get("initial_state")
        .cloned()
        .ok_or("header missing initial_state")?;
    let initial_state: State = serde_json::from_value(initial_value.clone())?;
    if initial_state.tick != 0 || initial_state.policy.water_meal_priority_enabled {
        return Err("input initial State is not tick-zero water-meal-off control".into());
    }
    let founder_ids: BTreeSet<_> = initial_state.entities.iter().map(|e| e.id).collect();
    let mut world = initial_state.clone().restore()?;
    if serde_json::to_value(State::capture(&world))? != initial_value
        || world.rng.state() != initial_state.rng
    {
        return Err("initial State/RNG restore round-trip mismatch".into());
    }
    let mut food_cumulative = 0u64;
    let mut water_cumulative = 0u64;
    let mut movement_cumulative = 0u64;
    let mut prefix_validations = Vec::new();
    for tick in 1..target_tick {
        let events = world.tick();
        let row = row_by_tick
            .get(&tick)
            .ok_or_else(|| format!("missing raw tick {tick}"))?;
        let check = match validate_prefix_tick(
            &world,
            &events,
            row,
            &founder_ids,
            &mut food_cumulative,
            &mut water_cumulative,
            &mut movement_cumulative,
        ) {
            Ok(value) => value,
            Err(error) => {
                let mut failed = failure_record(
                    &habitat,
                    seed,
                    target_tick,
                    input_path,
                    input_sha256,
                    error.to_string(),
                );
                failed["prefix_replay"] = json!({"from_initial_state":true,"validated_through_tick":tick-1,
                    "failed_tick":tick,"raw_tick_row":row,"reconstructed_state":serde_json::to_value(State::capture(&world))?,
                    "reconstructed_rng":world.rng.state(),"per_tick_validations":prefix_validations});
                write_atomic_capped(output_path, &failed)?;
                return Err(error);
            }
        };
        prefix_validations.push(check);
    }
    if world.tick_counter + 1 != target_tick {
        return Err("replay did not reach target pre-tick state".into());
    }
    let pre_state = State::capture(&world);
    let pre_roundtrip = state_roundtrip(&pre_state)?;
    if !pre_roundtrip.0 {
        let failed = failure_record(
            &habitat,
            seed,
            target_tick,
            input_path,
            input_sha256,
            "reconstructed target State/RNG restore round-trip mismatch".into(),
        );
        write_atomic_capped(output_path, &failed)?;
        return Err("reconstructed target State/RNG restore round-trip mismatch".into());
    }

    let target_row = row_by_tick
        .get(&target_tick)
        .ok_or("raw target tick missing")?;
    let mut target_food_cumulative = food_cumulative;
    let mut target_water_cumulative = water_cumulative;
    let mut target_movement_cumulative = movement_cumulative;
    let mut result = match world.resolver_order_lab_tick_checked(|native_off| {
        validate_native_off_target(
            native_off,
            target_row,
            &founder_ids,
            &mut target_food_cumulative,
            &mut target_water_cumulative,
            &mut target_movement_cumulative,
        )
    }) {
        Ok(case) => case.0,
        Err(error) => {
            let failed = json!({"schema":"curious-resolver-order-v1-case","status":"failed","case":{"habitat":habitat,"seed":seed,"target_tick":target_tick,"input_path":input_path,"input_sha256":input_sha256},"prefix_replay_valid":true,"failure":error.to_string()});
            write_atomic_capped(output_path, &failed)?;
            return Err(error);
        }
    };
    result["case"] = json!({"habitat":habitat,"seed":seed,"target_tick":target_tick,"input_path":input_path,"input_sha256":input_sha256});
    result["prefix_replay_valid"] = Value::Bool(true);
    result["prefix_replay"] = json!({"from_initial_state":true,"validated_through_tick":target_tick-1,"target_pre_state":serde_json::to_value(&pre_state)?,"target_pre_rng":pre_state.rng,"per_tick_validations":prefix_validations});
    result["target_validation"]["raw_tick_row"] = target_row.clone();
    result["target_validation"]["raw_focal_decision_trace"] = target_row
        .get("focal_decision_trace")
        .cloned()
        .unwrap_or(Value::Null);
    result["pre_target_roundtrip"] =
        json!({"valid":pre_roundtrip.0,"state_equal":pre_roundtrip.1,"rng_equal":pre_roundtrip.2});
    if !integrity_gates_pass(&result) {
        result["status"] = Value::String("failed".into());
        result["failure"] = Value::String("one or more lab integrity gates failed".into());
        write_atomic_capped(output_path, &result)?;
        return Err("one or more lab integrity gates failed".into());
    }
    write_atomic_capped(output_path, &result)?;
    Ok(())
}

fn integrity_gates_pass(result: &Value) -> bool {
    result.get("status").and_then(Value::as_str) == Some("complete")
        && result
            .pointer("/target_validation/valid")
            .and_then(Value::as_bool)
            == Some(true)
        && result
            .pointer("/references/a_matches_native_off")
            .and_then(Value::as_bool)
            == Some(true)
        && result
            .pointer("/references/c_matches_native_on")
            .and_then(Value::as_bool)
            == Some(true)
        && result
            .get("branch_roundtrip_valid")
            .and_then(Value::as_bool)
            == Some(true)
        && result
            .pointer("/candidate_b_equals_c_pre_resolution_audit/equal")
            .and_then(Value::as_bool)
            == Some(true)
}

fn failure_record(
    habitat: &str,
    seed: u64,
    target_tick: usize,
    input_path: &Path,
    input_sha256: &str,
    reason: String,
) -> Value {
    json!({"schema":"curious-resolver-order-v1-case","status":"failed","case":{"habitat":habitat,"seed":seed,"target_tick":target_tick,"input_path":input_path,"input_sha256":input_sha256},"prefix_replay_valid":false,"failure":reason})
}

fn parse_raw_jsonl(text: &str) -> AppResult<(Value, BTreeMap<usize, Value>, Value)> {
    let mut lines = text.lines().filter(|line| !line.trim().is_empty());
    let header: Value = serde_json::from_str(lines.next().ok_or("missing JSONL header")?)?;
    if header.get("type").and_then(Value::as_str) != Some("header") {
        return Err("first JSONL record is not a header".into());
    }
    let mut ticks = BTreeMap::new();
    let mut summary = None;
    for line in lines {
        let row: Value = serde_json::from_str(line)?;
        match row.get("type").and_then(Value::as_str) {
            Some("tick") if summary.is_none() => {
                let tick = row
                    .get("tick")
                    .and_then(Value::as_u64)
                    .ok_or("tick row missing integer tick")? as usize;
                if ticks.insert(tick, row).is_some() {
                    return Err("duplicate tick row".into());
                }
            }
            Some("summary") if summary.is_none() => summary = Some(row),
            _ => return Err("unexpected JSONL record or data after summary".into()),
        }
    }
    let declared = header
        .get("ticks")
        .and_then(Value::as_u64)
        .ok_or("header missing tick count")? as usize;
    if ticks.len() != declared
        || ticks.keys().copied().collect::<Vec<_>>() != (1..=declared).collect::<Vec<_>>()
    {
        return Err("raw JSONL tick rows do not match declared horizon".into());
    }
    let summary = summary.ok_or("missing final JSONL summary row")?;
    if summary.get("ticks_completed").and_then(Value::as_u64) != Some(declared as u64) {
        return Err("summary tick count differs from header".into());
    }
    Ok((header, ticks, summary))
}

fn sha256_bytes(bytes: &[u8]) -> AppResult<String> {
    let mut child = Command::new("sha256sum")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("sha256sum stdin unavailable")?
        .write_all(bytes)?;
    let output = child.wait_with_output()?;
    if !output.status.success() {
        return Err("sha256sum failed".into());
    }
    let line = String::from_utf8(output.stdout)?;
    Ok(line
        .split_whitespace()
        .next()
        .ok_or("sha256sum produced no digest")?
        .to_owned())
}

fn state_roundtrip(state: &State) -> AppResult<(bool, bool, bool)> {
    let before = serde_json::to_value(state)?;
    let restored = state.clone().restore()?;
    let state_equal = serde_json::to_value(State::capture(&restored))? == before;
    let rng_equal = restored.rng.state() == state.rng;
    Ok((state_equal && rng_equal, state_equal, rng_equal))
}

fn validate_native_off_target(
    reference: &Value,
    raw: &Value,
    founders: &BTreeSet<usize>,
    food_cumulative: &mut u64,
    water_cumulative: &mut u64,
    movement_cumulative: &mut u64,
) -> curious::Result<Value> {
    let state: State = serde_json::from_value(
        reference
            .get("state")
            .cloned()
            .ok_or("native-off reference lacks State")?,
    )?;
    let mut world = state.restore()?;
    world.last_decisions = serde_json::from_value(
        reference
            .get("last_decisions")
            .cloned()
            .ok_or("native-off reference lacks decision traces")?,
    )?;
    world.last_intents = serde_json::from_value(
        reference
            .get("last_intents")
            .cloned()
            .ok_or("native-off reference lacks intents")?,
    )?;
    let events: Vec<curious::simulation::SimulationEvent> = serde_json::from_value(
        reference
            .get("events")
            .cloned()
            .ok_or("native-off reference lacks events")?,
    )?;
    let mut projection = validate_prefix_tick(
        &world,
        &events,
        raw,
        founders,
        food_cumulative,
        water_cumulative,
        movement_cumulative,
    )?;
    projection["valid"] = Value::Bool(true);
    Ok(projection)
}

fn validate_prefix_tick(
    world: &World,
    events: &[curious::simulation::SimulationEvent],
    raw: &Value,
    founders: &BTreeSet<usize>,
    food_cumulative: &mut u64,
    water_cumulative: &mut u64,
    movement_cumulative: &mut u64,
) -> AppResult<Value> {
    let mut actor_events = Vec::new();
    let mut food = 0u64;
    let mut water = 0u64;
    let mut movement = 0u64;
    for event in events {
        let value = serde_json::to_value(event)?;
        let Some((name, payload)) = value.as_object().and_then(|o| o.iter().next()) else {
            continue;
        };
        let resolver = matches!(
            name.as_str(),
            "EntityAte" | "EntityDrank" | "EntityMoved" | "ActionRejected"
        );
        if resolver {
            actor_events.push(json!({"id":payload.get("id"),"event":value}));
        }
        match name.as_str() {
            "EntityAte" if payload.get("corpse_id").unwrap_or(&Value::Null).is_null() => {
                food += payload.get("amount").and_then(Value::as_u64).unwrap_or(0)
            }
            "EntityDrank" => water += payload.get("amount").and_then(Value::as_u64).unwrap_or(0),
            "EntityMoved" => {
                movement += payload
                    .get("energy_cost")
                    .and_then(Value::as_u64)
                    .unwrap_or(0)
            }
            _ => {}
        }
    }
    *food_cumulative += food;
    *water_cumulative += water;
    *movement_cumulative += movement;
    let actual_events = Value::Array(actor_events);
    let stocks = (resource_stock(world, true), resource_stock(world, false));
    let decisions = &world.last_decisions;
    let focal = decisions
        .iter()
        .find(|d| d.id == 1)
        .map(serde_json::to_value)
        .transpose()?
        .unwrap_or(Value::Null);
    let actual = json!({
        "tick":world.tick_counter,
        "actor_resolver_events":actual_events,
        "plant_food_gained_energy":food,"water_gained":water,"movement_energy_paid":movement,
        "plant_food_gained_cumulative":food_cumulative,"water_gained_cumulative":water_cumulative,"movement_energy_cumulative":movement_cumulative,
        "food_stock":stocks.0,"water_stock":stocks.1,
        "focal_decision_trace":focal,
        "live_total":world.entities.iter().filter(|e| e.base.life().is_alive() && matches!(e.phase,curious::entity::phase::EntityPhase::Active|curious::entity::phase::EntityPhase::Sleeping{..})).count(),
        "founders_alive":world.entities.iter().filter(|e| founders.contains(&e.id) && e.base.life().is_alive() && matches!(e.phase,curious::entity::phase::EntityPhase::Active|curious::entity::phase::EntityPhase::Sleeping{..})).count(),
    });
    let keys = [
        "tick",
        "actor_resolver_events",
        "plant_food_gained_energy",
        "water_gained",
        "movement_energy_paid",
        "plant_food_gained_cumulative",
        "water_gained_cumulative",
        "movement_energy_cumulative",
        "food_stock",
        "water_stock",
        "focal_decision_trace",
        "live_total",
        "founders_alive",
    ];
    for key in keys {
        if raw.get(key) != actual.get(key) {
            return Err(format!(
                "prefix replay mismatch at tick {} field {key}",
                world.tick_counter
            )
            .into());
        }
    }
    Ok(actual)
}

fn resource_stock(world: &World, food: bool) -> usize {
    let mut total = 0;
    for y in world.map.min_y..=world.map.max_y {
        for x in world.map.min_x..=world.map.max_x {
            match world.map.cell(curious::map::movement::Position::new(x, y)) {
                Some(curious::map::cell::Cell::Food { amount }) if food => total += amount,
                Some(curious::map::cell::Cell::Water { amount }) if !food => total += amount,
                _ => {}
            }
        }
    }
    total
}

fn write_atomic_capped(path: &Path, value: &Value) -> AppResult<()> {
    let bytes = serde_json::to_vec(value)?;
    if bytes.len() > MAX_OUTPUT {
        return Err("output exceeds 64 MiB cap".into());
    }
    let tmp = path.with_extension("json.tmp");
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&tmp)?;
    file.write_all(&bytes)?;
    file.write_all(b"\n")?;
    file.sync_all()?;
    fs::rename(tmp, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_jsonl_parser_accepts_final_summary_after_ticks() {
        let text = concat!(
            "{\"type\":\"header\",\"ticks\":2}\n",
            "{\"type\":\"tick\",\"tick\":1}\n",
            "{\"type\":\"tick\",\"tick\":2}\n",
            "{\"type\":\"summary\",\"ticks_completed\":2}\n",
        );
        let (_, ticks, summary) = parse_raw_jsonl(text).unwrap();
        assert_eq!(ticks.len(), 2);
        assert_eq!(summary["type"], "summary");
    }

    #[test]
    fn raw_jsonl_parser_rejects_rows_after_summary() {
        let text = concat!(
            "{\"type\":\"header\",\"ticks\":1}\n",
            "{\"type\":\"tick\",\"tick\":1}\n",
            "{\"type\":\"summary\",\"ticks_completed\":1}\n",
            "{\"type\":\"tick\",\"tick\":2}\n",
        );
        assert!(parse_raw_jsonl(text).is_err());
    }

    #[test]
    fn integrity_gate_reads_actual_nested_json_fields() {
        let mut output = json!({
            "status":"complete",
            "target_validation":{"valid":true},
            "references":{"a_matches_native_off":true,"c_matches_native_on":true},
            "branch_roundtrip_valid":true,
            "candidate_b_equals_c_pre_resolution_audit":{"equal":true}
        });
        assert!(integrity_gates_pass(&output));
        output["references"]["c_matches_native_on"] = Value::Bool(false);
        assert!(!integrity_gates_pass(&output));
    }
}
