//! Bounded single-world ecology/trait throughput diagnostic prototype.
//!
//! Usage: ecology_candidate SCENARIO.json SEED TICKS SAMPLE_EVERY NEW_OUTPUT.json
//! This is a measurement harness, not an acceptance runner. It never changes
//! the supplied scenario or inserts entities/resources after tick zero.
use curious::{
    environment::ResourceKind,
    map::{cell::Cell, movement::Position},
    scenario::Scenario,
    simulation::SimulationEvent,
    world::World,
};
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    fs::OpenOptions,
    io::Write,
    path::Path,
};

const MAX_TICKS: usize = 100_000;
const MAX_SAMPLES: usize = 1_001;

fn stock(world: &World) -> (usize, usize) {
    let (mut food, mut water) = (0, 0);
    for y in world.map.min_y..=world.map.max_y {
        for x in world.map.min_x..=world.map.max_x {
            match world.map.cell(Position::new(x, y)) {
                Some(Cell::Food { amount }) => food += amount,
                Some(Cell::Water { amount }) => water += amount,
                _ => {}
            }
        }
    }
    (food, water)
}

fn alive(world: &World) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for entity in &world.entities {
        if entity.base.life().is_alive()
            && matches!(
                &entity.phase,
                curious::entity::phase::EntityPhase::Active
                    | curious::entity::phase::EntityPhase::Sleeping { .. }
            )
        {
            *counts
                .entry(format!("{:?}", entity.base.species()))
                .or_default() += 1;
        }
    }
    counts
}

fn run() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args: Vec<_> = env::args().collect();
    if args.len() != 6 {
        return Err(
            "usage: ecology_candidate SCENARIO.json SEED TICKS SAMPLE_EVERY NEW_OUTPUT.json".into(),
        );
    }
    let ticks: usize = args[3].parse()?;
    let sample_every: usize = args[4].parse()?;
    if ticks == 0 || ticks > MAX_TICKS || sample_every == 0 {
        return Err(
            format!("ticks must be 1..={MAX_TICKS}; sample interval must be positive").into(),
        );
    }
    let seed: u64 = args[2].parse()?;
    let scenario_text = fs::read_to_string(&args[1])?;
    let scenario: Scenario = serde_json::from_str(&scenario_text)?;
    let scenario_name = scenario.name.clone();
    let mut world = scenario.build(seed)?;
    let (food_stock_at_start, water_stock_at_start) = stock(&world);
    let founders: BTreeSet<usize> = world.entities.iter().map(|e| e.id).collect();
    let founder_species: BTreeMap<usize, String> = world
        .entities
        .iter()
        .map(|e| (e.id, format!("{:?}", e.base.species())))
        .collect();
    let mut generations: BTreeMap<usize, usize> = founders.iter().map(|id| (*id, 0)).collect();
    let mut breeder_ids = BTreeSet::new();
    let mut births_by_generation = BTreeMap::<usize, usize>::new();
    let mut deaths_by_cause = BTreeMap::<String, usize>::new();
    let mut trait_requested = [0usize; 2];
    let mut trait_paid = [0usize; 2];
    let mut food_eaten = 0usize;
    let mut water_drunk = 0usize;
    let mut movement_energy_paid = 0usize;
    let mut resource_replenished_amount = BTreeMap::<String, usize>::new();
    let mut samples: Vec<Value> = Vec::new();

    for tick in 1..=ticks {
        for event in world.tick() {
            match event {
                SimulationEvent::EntityMated {
                    parent1,
                    parent2,
                    child,
                    ..
                } => {
                    breeder_ids.extend([parent1, parent2]);
                    let generation = generations
                        .get(&parent1)
                        .copied()
                        .unwrap_or(0)
                        .max(generations.get(&parent2).copied().unwrap_or(0))
                        + 1;
                    generations.insert(child, generation);
                }
                SimulationEvent::EntityBorn { id, .. } => {
                    let generation = world
                        .entities
                        .iter()
                        .find(|e| e.id == id)
                        .map(|e| e.generation)
                        .or_else(|| generations.get(&id).copied())
                        .unwrap_or(0);
                    generations.entry(id).or_insert(generation);
                    *births_by_generation.entry(generation).or_default() += 1;
                }
                SimulationEvent::EntityDied { cause, .. } => {
                    *deaths_by_cause.entry(format!("{cause:?}")).or_default() += 1;
                }
                SimulationEvent::EntityAte { amount, .. } => food_eaten += amount,
                SimulationEvent::EntityDrank { amount, .. } => water_drunk += amount,
                SimulationEvent::EntityMoved { energy_cost, .. } => {
                    movement_energy_paid += energy_cost
                }
                SimulationEvent::EntityMetabolized {
                    vision_cost,
                    stamina_cost,
                    vision_paid,
                    stamina_paid,
                    ..
                } => {
                    if vision_paid > vision_cost || stamina_paid > stamina_cost {
                        return Err("trait payment exceeded requested maintenance".into());
                    }
                    trait_requested[0] += vision_cost;
                    trait_requested[1] += stamina_cost;
                    trait_paid[0] += vision_paid;
                    trait_paid[1] += stamina_paid;
                }
                SimulationEvent::ResourceReplenished { kind, amount, .. } => {
                    let name = match kind {
                        ResourceKind::Food => "Food",
                        ResourceKind::Water => "Water",
                    };
                    *resource_replenished_amount
                        .entry(name.to_owned())
                        .or_default() += amount;
                }
                _ => {}
            }
        }
        if tick == 1 || tick % sample_every == 0 || tick == ticks {
            if samples.len() == MAX_SAMPLES {
                return Err(format!("sample cap {MAX_SAMPLES} exceeded").into());
            }
            let (food_stock, water_stock) = stock(&world);
            samples.push(json!({
                "tick": tick,
                "alive": alive(&world),
                "founders_alive": world.entities.iter().filter(|e| founders.contains(&e.id) && e.base.life().is_alive()).count(),
                "descendants_alive": world.entities.iter().filter(|e| !founders.contains(&e.id) && e.base.life().is_alive()).count(),
                "food_stock": food_stock,
                "water_stock": water_stock,
                "food_eaten_cumulative": food_eaten,
                "water_drunk_cumulative": water_drunk,
                "movement_energy_paid_cumulative": movement_energy_paid,
                "births_by_generation": births_by_generation,
                "descendant_breeders_observed": breeder_ids.iter().filter(|id| !founders.contains(id)).count()
            }));
        }
    }

    let living_founders = world
        .entities
        .iter()
        .filter(|e| founders.contains(&e.id) && e.base.life().is_alive())
        .count();
    let living_descendant_breeders = world
        .entities
        .iter()
        .filter(|e| {
            !founders.contains(&e.id) && breeder_ids.contains(&e.id) && e.base.life().is_alive()
        })
        .count();
    let f1_breeders_ever = breeder_ids
        .iter()
        .filter(|id| generations.get(id).copied() == Some(1))
        .count();
    let (food_stock, water_stock) = stock(&world);
    let output = json!({
        "schema": "curious-ecology-candidate-v1",
        "status": "diagnostic_only",
        "seed": seed,
        "ticks": ticks,
        "scenario_name": scenario_name,
        "founders_at_tick_0": founders.len(),
        "founder_species": founder_species,
        "food_stock_at_start": food_stock_at_start,
        "water_stock_at_start": water_stock_at_start,
        "founders_alive_at_end": living_founders,
        "total_births_by_generation": births_by_generation,
        "descendant_breeders_ever": breeder_ids.iter().filter(|id| !founders.contains(id)).count(),
        "f1_breeders_ever": f1_breeders_ever,
        "living_descendant_breeders_at_end": living_descendant_breeders,
        "alive_at_end": alive(&world),
        "deaths_by_cause": deaths_by_cause,
        "food_stock_at_end": food_stock,
        "water_stock_at_end": water_stock,
        "food_eaten": food_eaten,
        "water_drunk": water_drunk,
        "movement_energy_paid": movement_energy_paid,
        "trait_maintenance_requested": {"vision": trait_requested[0], "stamina": trait_requested[1]},
        "trait_maintenance_paid": {"vision": trait_paid[0], "stamina": trait_paid[1]},
        "resource_replenished_amount": resource_replenished_amount,
        "samples": samples
    });
    let path = Path::new(&args[5]);
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    serde_json::to_writer_pretty(&mut file, &output)?;
    file.write_all(b"\n")?;
    file.sync_all()?;
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("ecology_candidate: {error}");
        std::process::exit(2);
    }
}
