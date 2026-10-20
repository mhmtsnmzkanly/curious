//! Create a fresh durable world with explicit, qualified initial settings.
//! The serving engine resumes this record; no bootstrap mutation is replayed.
use curious::{
    Result, entity::genetics::Genetics, persistence::Store, policy::PolicySettings,
    scenario::Scenario, simulation::checkpoint::State, world::World,
};
use serde::{Deserialize, Serialize};
use std::{fs, io::Write, path::Path};

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Conditions {
    seed: u64,
    policy: PolicySettings,
    genetics: Genetics,
    learning_enabled: bool,
    companions_enabled: bool,
}
fn prepare(scenario: Scenario, conditions: &Conditions) -> Result<World> {
    if conditions.genetics.mutation_per_mille > 1000 {
        return Err("mutation rate must be 0..1000 per mille".into());
    }
    let mut world = scenario.build(conditions.seed)?;
    world.policy = conditions.policy.clone();
    world.genetics = conditions.genetics.clone();
    for slot in &mut world.entities {
        slot.learner.enabled = conditions.learning_enabled;
        slot.memory.companions_enabled = conditions.companions_enabled;
    }
    Ok(world)
}
fn initialize(directory: &Path, world: &World) -> Result<()> {
    // Never open, overwrite or reset an existing run, including an empty directory.
    fs::create_dir(directory)?;
    let state = State::capture(world);
    let encoded = serde_json::to_vec(&state)?;
    let mut initial = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(directory.join("initial-state.json"))?;
    initial.write_all(&encoded)?;
    initial.sync_all()?;
    let mut store = Store::open(directory, 64 * 1024 * 1024)?;
    store.record(world, &[], "initial", 0, None)?;
    if serde_json::to_value(store.latest()?.ok_or("missing initial checkpoint")?)?
        != serde_json::to_value(&state)?
    {
        return Err("initial persisted checkpoint differs from prepared world".into());
    }
    Ok(())
}
fn run() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 4 {
        return Err("usage: prepare_observation SCENARIO CONDITIONS NEW_DATA_DIRECTORY".into());
    }
    let scenario = serde_json::from_slice(&fs::read(&args[1])?)?;
    let conditions: Conditions = serde_json::from_slice(&fs::read(&args[2])?)?;
    let world = prepare(scenario, &conditions)?;
    initialize(Path::new(&args[3]), &world)?;
    println!(
        "{}",
        serde_json::json!({"tick":world.tick_counter,"seed":conditions.seed,"policy":world.policy,"genetics":world.genetics,"entities":world.entities.len()})
    );
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn conditions() -> Conditions {
        Conditions {
            seed: 20001,
            policy: PolicySettings {
                reserve_opportunities: true,
                mate_rendezvous: true,
            },
            genetics: Genetics::default(),
            learning_enabled: true,
            companions_enabled: true,
        }
    }
    fn scenario() -> Scenario {
        serde_json::from_str(include_str!("../scenarios/mixed-refuge.json")).unwrap()
    }
    #[test]
    fn initial_record_preserves_settings_and_every_future_state_and_rng() {
        let c = conditions();
        let mut direct = scenario().build(c.seed).unwrap();
        // Independent reference: ecology_probe's `both` treatment before its first tick.
        direct.policy.mate_rendezvous = true;
        direct.policy.reserve_opportunities = true;
        direct.genetics.enabled = true;
        for e in &mut direct.entities {
            e.learner.enabled = true;
            e.memory.companions_enabled = true;
        }
        let prepared = prepare(scenario(), &c).unwrap();
        let root = tempfile::tempdir().unwrap();
        let dir = root.path().join("world");
        initialize(&dir, &prepared).unwrap();
        let mut store = Store::open(&dir, 64 * 1024 * 1024).unwrap();
        let mut restored = store.latest().unwrap().unwrap().restore().unwrap();
        assert_eq!(restored.tick_counter, 0);
        for _ in 0..100 {
            assert_eq!(
                serde_json::to_value(State::capture(&direct)).unwrap(),
                serde_json::to_value(State::capture(&restored)).unwrap()
            );
            direct.tick();
            restored.tick();
        }
        assert_eq!(
            serde_json::to_value(State::capture(&direct)).unwrap(),
            serde_json::to_value(State::capture(&restored)).unwrap()
        );
    }
    #[test]
    fn a_second_initialization_never_changes_an_existing_run() {
        let world = prepare(scenario(), &conditions()).unwrap();
        let root = tempfile::tempdir().unwrap();
        let dir = root.path().join("world");
        initialize(&dir, &world).unwrap();
        let before = fs::read(dir.join("initial-state.json")).unwrap();
        assert!(initialize(&dir, &world).is_err());
        assert_eq!(before, fs::read(dir.join("initial-state.json")).unwrap());
    }
    #[test]
    fn invalid_initial_settings_are_rejected_before_storage_is_created() {
        let mut c = conditions();
        c.genetics.mutation_per_mille = 1001;
        assert!(prepare(scenario(), &c).is_err());
    }
}
