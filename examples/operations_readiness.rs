//! Offline, bounded operations-readiness probe for the frozen V12 qualification
//! checkpoint. It reads one fixed-hash input and writes only to a new output dir.
use curious::{
    archive::Reader,
    persistence::{State, Store},
    world::World,
    Result,
};
use serde_json::{json, Value};
use std::{
    env,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Instant, SystemTime, UNIX_EPOCH},
};

const INPUT_SHA256: &str = "357870bd9a3d414704ed6befbea050852028f54af304c27cdca482dc31abdad2";
const INPUT_TICK: usize = 12_639;
const INPUT_ENTITIES: usize = 630;
const DEFAULT_TICKS: usize = 120;
const MAX_TICKS: usize = 120;
const STORAGE_SHARD_BYTES: u64 = 1024 * 1024;
const STORAGE_HISTORY_BYTES: u64 = 8 * 1024 * 1024;
const MAX_OUTPUT_BYTES: u64 = 256 * 1024 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    Off,
    Care,
    Refuge,
    Bundle,
}

impl Mode {
    fn parse(value: &str) -> Result<Self> {
        match value {
            "off" => Ok(Self::Off),
            "care" => Ok(Self::Care),
            "refuge" => Ok(Self::Refuge),
            "bundle" => Ok(Self::Bundle),
            _ => Err(format!("invalid mode {value:?}; use off|care|refuge|bundle").into()),
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Care => "care",
            Self::Refuge => "refuge",
            Self::Bundle => "bundle",
        }
    }

    fn flags(self) -> (bool, bool, bool) {
        match self {
            Self::Off => (false, false, false),
            Self::Care => (true, false, false),
            Self::Refuge => (false, true, true),
            Self::Bundle => (true, true, true),
        }
    }
}

fn configure_mode(world: &mut World, mode: Mode) {
    let (care, refuge_first_interval, refuge_outcomes) = mode.flags();
    world.policy.juvenile_care_enabled = care;
    for entity in &mut world.entities {
        entity.memory.first_interval_refuge_enabled = refuge_first_interval;
        entity.memory.refuge_outcomes_enabled = refuge_outcomes;
    }
}

fn sha256_bytes(bytes: &[u8]) -> Result<String> {
    let mut child = Command::new("sha256sum")
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("sha256sum stdin unavailable")?
        .write_all(bytes)?;
    let output = child.wait_with_output()?;
    if !output.status.success() {
        return Err(format!(
            "sha256sum failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    let line = String::from_utf8(output.stdout)?;
    Ok(line
        .split_whitespace()
        .next()
        .ok_or("sha256sum returned no digest")?
        .to_owned())
}

fn sha256_file(path: &Path) -> Result<String> {
    let mut file = File::open(path)?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    sha256_bytes(&bytes)
}

fn git_value(args: &[&str]) -> String {
    Command::new("git")
        .args(args)
        .output()
        .ok()
        .filter(|out| out.status.success())
        .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_owned())
        .unwrap_or_else(|| "unavailable".to_owned())
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}

fn directory_bytes(path: &Path) -> Result<u64> {
    let mut total = 0_u64;
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        let bytes = if kind.is_dir() {
            directory_bytes(&entry.path())?
        } else if kind.is_file() {
            entry.metadata()?.len()
        } else {
            0
        };
        total = total
            .checked_add(bytes)
            .ok_or("output byte count overflow")?;
        if total > MAX_OUTPUT_BYTES {
            return Err(format!("output exceeded {} byte budget", MAX_OUTPUT_BYTES).into());
        }
    }
    Ok(total)
}

fn create_fresh_dir(path: &Path) -> Result<()> {
    match fs::symlink_metadata(path) {
        Ok(_) => Err(format!("output already exists: {}", path.display()).into()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            fs::create_dir(path)?;
            Ok(())
        }
        Err(error) => Err(error.into()),
    }
}

fn sqlite_integrity(path: &Path) -> Result<String> {
    let reader = Reader::open(path)?;
    let result: String = reader.query_row("PRAGMA integrity_check", [], |row| row.get(0))?;
    if result != "ok" {
        return Err(format!(
            "SQLite integrity_check failed for {}: {result}",
            path.display()
        )
        .into());
    }
    Ok(result)
}

/// Exercise isolated Store rotation using the actual restored V12-derived state.
/// Reader::open transparently restores a closed zstd shard into a temporary DB.
fn storage_smoke(initial: &State, directory: &Path) -> Result<Value> {
    if serde_json::to_vec(initial)?.len() < STORAGE_SHARD_BYTES as usize {
        return Err(
            "input-derived checkpoint is too small to exercise 1 MiB shard rotation".into(),
        );
    }
    create_fresh_dir(directory)?;
    let mut world = initial.clone().restore()?;
    let mut store = Store::open(directory, STORAGE_SHARD_BYTES)?;
    store.max_history_bytes = STORAGE_HISTORY_BYTES;
    store.record(&world, &[], "readiness_initial", 0, None)?;
    let first_state = State::capture(&world);
    let event_start = Instant::now();
    let events = world.tick();
    let storage_tick_us = event_start.elapsed().as_micros();
    store.record(&world, &events, "tick", storage_tick_us, None)?;
    let inventory_before_close = store.inventory()?;
    drop(store);

    let mut reopened = Store::open(directory, STORAGE_SHARD_BYTES)?;
    reopened.max_history_bytes = STORAGE_HISTORY_BYTES;
    let inventory = reopened.inventory()?;
    let files = inventory
        .as_array()
        .ok_or("Store inventory is not an array")?;
    let mut saw_closed_archive = false;
    let mut checks = Vec::new();
    for item in files {
        let shard = item
            .get("shard")
            .and_then(Value::as_u64)
            .ok_or("inventory shard missing")?;
        let compressed = item
            .get("compressed")
            .and_then(Value::as_bool)
            .ok_or("inventory compressed flag missing")?;
        let path = directory.join(format!(
            "history-{shard:06}.sqlite{}",
            if compressed { ".zst" } else { "" }
        ));
        if compressed {
            saw_closed_archive = true;
        }
        checks.push(json!({"shard":shard,"compressed":compressed,"integrity_check":sqlite_integrity(&path)?}));
    }
    if !saw_closed_archive {
        return Err("Store smoke did not produce a closed zstd shard".into());
    }
    let latest = reopened
        .latest()?
        .ok_or("Store latest returned no state after reopen")?;
    let expected = State::capture(&world);
    if serde_json::to_value(&latest)? != serde_json::to_value(&expected)? {
        return Err("Store latest differs from the state recorded in the isolated smoke".into());
    }
    let latest_bytes = serde_json::to_vec(&latest)?;
    drop(reopened);
    Ok(json!({
        "status":"passed",
        "scope":"isolated output directory only; no writer-lock claim",
        "max_shard_bytes":STORAGE_SHARD_BYTES,
        "max_history_bytes":STORAGE_HISTORY_BYTES,
        "records":2,
        "tick_count":1,
        "first_state_tick":first_state.tick,
        "latest_tick":latest.tick,
        "storage_tick_us":storage_tick_us,
        "inventory_at_close":inventory_before_close,
        "reopened_inventory":inventory,
        "sqlite_integrity_checks":checks,
        "closed_zstd_archive_restored_via_Reader":saw_closed_archive,
        "latest_state_sha256":sha256_bytes(&latest_bytes)?,
    }))
}

fn percentile(values: &[u128], numerator: usize, denominator: usize) -> u128 {
    if values.is_empty() {
        return 0;
    }
    let rank = (values.len() * numerator).div_ceil(denominator).max(1);
    values[rank.saturating_sub(1).min(values.len() - 1)]
}

fn now_unix_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

fn run() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if !(4..=5).contains(&args.len()) {
        return Err("usage: operations_readiness INPUT_V12_JSON NEW_OUTPUT_DIR off|care|refuge|bundle [TICKS<=120]".into());
    }
    let input = PathBuf::from(&args[1]);
    let output_dir = PathBuf::from(&args[2]);
    let mode = Mode::parse(&args[3])?;
    let ticks = args
        .get(4)
        .map(|v| v.parse())
        .transpose()?
        .unwrap_or(DEFAULT_TICKS);
    if ticks == 0 || ticks > MAX_TICKS {
        return Err(format!("ticks must be in 1..={MAX_TICKS}").into());
    }

    // Validate immutable input and fresh output path before creating anything.
    let input_bytes = fs::read(&input)?;
    let input_sha = sha256_bytes(&input_bytes)?;
    if input_sha != INPUT_SHA256 {
        return Err(
            format!("input SHA-256 mismatch: expected {INPUT_SHA256}, got {input_sha}").into(),
        );
    }
    let input_state: State = serde_json::from_slice(&input_bytes)?;
    if input_state.version != 12
        || input_state.tick != INPUT_TICK
        || input_state.entities.len() != INPUT_ENTITIES
    {
        return Err(format!(
            "expected V12 tick {INPUT_TICK} with {INPUT_ENTITIES} entities; got V{} tick {} with {} entities",
            input_state.version,
            input_state.tick,
            input_state.entities.len()
        )
        .into());
    }
    let started_ms = now_unix_ms();
    let restore_start = Instant::now();
    let mut world = input_state.clone().restore()?;
    let restore_us = restore_start.elapsed().as_micros();
    let flag_start = Instant::now();
    configure_mode(&mut world, mode);
    let flag_setup_us = flag_start.elapsed().as_micros();
    let restored_initial = State::capture(&world);
    if restored_initial.version != 19 {
        return Err("V12 input did not migrate to V19 on restore".into());
    }
    if restored_initial.tick != input_state.tick
        || restored_initial.entities.len() != input_state.entities.len()
    {
        return Err(format!(
            "V12 migration changed tick/entity count: {} -> {}, {} -> {}",
            input_state.tick,
            restored_initial.tick,
            input_state.entities.len(),
            restored_initial.entities.len()
        )
        .into());
    }

    create_fresh_dir(&output_dir)?;
    let initial_capture_start = Instant::now();
    let initial_state = State::capture(&world);
    let initial_capture_us = initial_capture_start.elapsed().as_micros();
    let initial_serialize_start = Instant::now();
    let initial_bytes = serde_json::to_vec_pretty(&initial_state)?;
    let initial_serialize_us = initial_serialize_start.elapsed().as_micros();
    let initial_hash = sha256_bytes(&initial_bytes)?;
    let initial_roundtrip_start = Instant::now();
    let initial_recaptured =
        State::capture(&serde_json::from_slice::<State>(&initial_bytes)?.restore()?);
    let initial_state_roundtrip_exact =
        serde_json::to_value(&initial_recaptured)? == serde_json::to_value(&initial_state)?;
    let initial_restore_recapture_us = initial_roundtrip_start.elapsed().as_micros();
    if !initial_state_roundtrip_exact {
        return Err("initial migrated V19 State/RNG restore and recapture were not exact".into());
    }
    write_new(&output_dir.join("frame-000000.json"), &initial_bytes)?;
    let storage = storage_smoke(&initial_state, &output_dir.join("storage"))?;
    let output_bytes_after_storage = directory_bytes(&output_dir)?;

    // Timing encloses exactly World::tick; capture, serialization and storage are separate.
    let mut tick_us = Vec::with_capacity(ticks);
    for _ in 0..ticks {
        let start = Instant::now();
        let _events = world.tick();
        tick_us.push(start.elapsed().as_micros());
    }
    let tick_total_us: u128 = tick_us.iter().sum();
    let final_capture_start = Instant::now();
    let final_state = State::capture(&world);
    let expected_final_tick = input_state
        .tick
        .checked_add(ticks)
        .ok_or("expected final tick overflow")?;
    if final_state.tick != expected_final_tick {
        return Err(format!(
            "final tick mismatch: expected {expected_final_tick}, got {}",
            final_state.tick
        )
        .into());
    }
    let final_capture_us = final_capture_start.elapsed().as_micros();
    let final_serialize_start = Instant::now();
    let final_bytes = serde_json::to_vec_pretty(&final_state)?;
    let final_serialize_us = final_serialize_start.elapsed().as_micros();
    let final_hash = sha256_bytes(&final_bytes)?;
    write_new(
        &output_dir.join(format!("frame-{ticks:06}.json")),
        &final_bytes,
    )?;
    let roundtrip_start = Instant::now();
    let recaptured = State::capture(&serde_json::from_slice::<State>(&final_bytes)?.restore()?);
    let final_state_roundtrip_exact =
        serde_json::to_value(&recaptured)? == serde_json::to_value(&final_state)?;
    let final_restore_recapture_us = roundtrip_start.elapsed().as_micros();
    if !final_state_roundtrip_exact {
        return Err("final V19 State/RNG restore and recapture were not exact".into());
    }
    let output_bytes_before_manifest = directory_bytes(&output_dir)?;

    let executable = env::current_exe()?;
    let executable_sha = sha256_file(&executable)?;
    let code_sha = sha256_file(Path::new("examples/operations_readiness.rs"))?;
    let manifest = json!({
        "status":"completed",
        "started_unix_ms":started_ms,
        "finished_unix_ms":now_unix_ms(),
        "argv":args,
        "input_path":input,
        "input_sha256":input_sha,
        "input_expected_sha256":INPUT_SHA256,
        "input_bytes":input_bytes.len(),
        "input_state":{"version":input_state.version,"tick":input_state.tick,"entities":input_state.entities.len()},
        "migrated_initial_state":{"version":initial_state.version,"tick":initial_state.tick,"entities":initial_state.entities.len(),"sha256":initial_hash},
        "requested_mode":mode.label(),
        "policy_flags":{"juvenile_care_enabled":world.policy.juvenile_care_enabled,"first_interval_refuge_enabled_entities":world.entities.iter().filter(|e|e.memory.first_interval_refuge_enabled).count(),"refuge_outcomes_enabled_entities":world.entities.iter().filter(|e|e.memory.refuge_outcomes_enabled).count()},
        "promotion_allowed":false,
        "promotion_blocker":"current utility gate not passed; this offline probe is not deployment qualification",
        "ticks":ticks,
        "final_state":{"version":final_state.version,"tick":final_state.tick,"entities":final_state.entities.len(),"sha256":final_hash,"restore_recapture_state_rng_exact":final_state_roundtrip_exact},
        "timing_us":{"input_restore":restore_us,"flag_setup":flag_setup_us,"initial_state_capture":initial_capture_us,"initial_state_serde":initial_serialize_us,"initial_restore_recapture":initial_restore_recapture_us,"world_tick_total":tick_total_us,"world_tick_mean":tick_total_us as f64/ticks as f64,"world_tick_p50":percentile(&tick_us,50,100),"world_tick_p95":percentile(&tick_us,95,100),"world_tick_max":tick_us.iter().copied().max().unwrap_or(0),"final_state_capture":final_capture_us,"final_state_serde":final_serialize_us,"final_restore_recapture":final_restore_recapture_us},
        "initial_state":{"migration_tick_preserved":initial_state.tick==input_state.tick,"entity_count_preserved":initial_state.entities.len()==input_state.entities.len(),"restore_recapture_state_rng_exact":initial_state_roundtrip_exact},
        "process":{"executable":executable,"executable_sha256":executable_sha,"repository_head":git_value(&["rev-parse","HEAD"]),"repository_status_porcelain":git_value(&["status","--porcelain"]),"example_source_sha256":code_sha,"rss_measurement":"capture externally with /usr/bin/time -v"},
        "storage_smoke":storage,
        "output_budget_bytes":MAX_OUTPUT_BYTES,
        "output_bytes_after_storage":output_bytes_after_storage,
        "output_bytes_before_manifest":output_bytes_before_manifest,
    });
    let manifest_bytes = serde_json::to_vec_pretty(&manifest)?;
    let expected_final_output_bytes = output_bytes_before_manifest
        .checked_add(manifest_bytes.len() as u64)
        .ok_or("projected output byte count overflow")?;
    if expected_final_output_bytes > MAX_OUTPUT_BYTES {
        return Err("output directory exceeded fixed disk budget".into());
    }
    write_new(&output_dir.join("run.json"), &manifest_bytes)?;
    File::open(&output_dir)?.sync_all()?;
    println!("{}", serde_json::to_string_pretty(&manifest)?);
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("operations_readiness: {error}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use curious::{
        creatures::herbivore::HerbivoreEntity,
        entity::{controller::EntityController, phase::EntityPhase},
        map::movement::Position,
        world::EntitySlot,
    };

    fn one_entity_world() -> World {
        let entity = EntitySlot::new(
            1,
            Position::new(0, 0),
            EntityPhase::Active,
            EntityController::AI,
            Box::new(HerbivoreEntity::default()),
        );
        World::with_seed(0, 1, 0, 1, vec![entity], 62490)
    }

    #[test]
    fn each_mode_sets_only_the_requested_readiness_flags() {
        for (mode, care, refuge_first_interval, refuge_outcomes) in [
            (Mode::Off, false, false, false),
            (Mode::Care, true, false, false),
            (Mode::Refuge, false, true, true),
            (Mode::Bundle, true, true, true),
        ] {
            let mut world = one_entity_world();
            world.policy.juvenile_care_enabled = !care;
            configure_mode(&mut world, mode);
            assert_eq!(world.policy.juvenile_care_enabled, care);
            assert!(world.entities.iter().all(|e| {
                e.memory.first_interval_refuge_enabled == refuge_first_interval
                    && e.memory.refuge_outcomes_enabled == refuge_outcomes
            }));
        }
    }

    #[test]
    fn migration_configures_flags_after_v12_restore_and_roundtrips_state_rng() {
        let v12 = State::capture(&one_entity_world());
        let mut v12 = v12;
        v12.version = 12;
        let mut world = v12.clone().restore().unwrap();
        assert_eq!(State::capture(&world).version, 19);
        assert_eq!(State::capture(&world).tick, v12.tick);
        assert_eq!(State::capture(&world).entities.len(), v12.entities.len());
        configure_mode(&mut world, Mode::Off);
        let before = State::capture(&world);
        let after = State::capture(&before.clone().restore().unwrap());
        assert_eq!(
            serde_json::to_value(before).unwrap(),
            serde_json::to_value(after).unwrap()
        );
    }

    #[test]
    fn refuses_existing_output_without_modifying_it() {
        let root = tempfile::tempdir().unwrap();
        let output = root.path().join("already-used");
        fs::create_dir(&output).unwrap();
        let marker = output.join("keep.txt");
        fs::write(&marker, b"preserve").unwrap();
        assert!(create_fresh_dir(&output).is_err());
        assert_eq!(fs::read(marker).unwrap(), b"preserve");
    }
}
