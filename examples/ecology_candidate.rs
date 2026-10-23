//! Bounded single-world ecology/trait throughput diagnostic prototype.
//!
//! Usage: ecology_candidate SCENARIO.json SEED TICKS SAMPLE_EVERY NEW_OUTPUT.json
//! This is a measurement harness, not an acceptance runner. It never changes
//! the supplied scenario or inserts entities/resources after tick zero.
#[path = "support/ecology_audit.rs"]
mod ecology_audit;
use curious::{scenario::Scenario, world::World};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    fs::OpenOptions,
    io::Write,
    path::Path,
};

const MAX_TICKS: usize = 100_000;
const MAX_SAMPLES: usize = 1_001;

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
    let start_stock = ecology_audit::stock(&world);
    let mut previous_stock = start_stock;
    let mut previous_cells = ecology_audit::resource_cells(&world);
    let founders: BTreeSet<usize> = world.entities.iter().map(|e| e.id).collect();
    let founder_species: BTreeMap<usize, String> = world
        .entities
        .iter()
        .map(|e| (e.id, format!("{:?}", e.base.species())))
        .collect();
    let mut audit = ecology_audit::Audit::default();
    audit.seed(&world);
    let mut samples: Vec<Value> = Vec::new();

    for tick in 1..=ticks {
        let events = world.tick();
        (previous_stock, previous_cells) =
            audit.observe_tick(tick, previous_stock, &previous_cells, &world, &events);
        if tick == 1 || tick % sample_every == 0 || tick == ticks {
            if samples.len() == MAX_SAMPLES {
                return Err(format!("sample cap {MAX_SAMPLES} exceeded").into());
            }
            let (food_stock, water_stock) = ecology_audit::stock(&world);
            samples.push(json!({
                "tick": tick,
                "alive": alive(&world),
                "founders_alive": world.entities.iter().filter(|e| founders.contains(&e.id) && e.base.life().is_alive()).count(),
                "descendants_alive": world.entities.iter().filter(|e| !founders.contains(&e.id) && e.base.life().is_alive()).count(),
                "food_stock": food_stock,
                "water_stock": water_stock,
                "plant_food_meals_cumulative": audit.food_meals,
                "corpse_food_meals_cumulative": audit.corpse_meals,
                "water_drunk_cumulative": audit.drinks,
                "births_by_generation": audit.births_by_generation,
                "descendant_breeders_observed": audit.bred_ids.iter().filter(|id| !founders.contains(id)).count()
            }));
        }
    }

    let living_founders = world
        .entities
        .iter()
        .filter(|e| founders.contains(&e.id) && e.base.life().is_alive())
        .count();
    let (food_stock, water_stock) = ecology_audit::stock(&world);
    let audit_result = audit.result(&world);
    let output = json!({
        "schema": "curious-ecology-candidate-v2",
        "status": "diagnostic_only",
        "seed": seed,
        "ticks": ticks,
        "scenario_name": scenario_name,
        "founders_at_tick_0": founders.len(),
        "founder_species": founder_species,
        "food_stock_at_start": start_stock.0,
        "water_stock_at_start": start_stock.1,
        "founders_alive_at_end": living_founders,
        "audit": audit_result,
        "alive_at_end": alive(&world),
        "food_stock_at_end": food_stock,
        "water_stock_at_end": water_stock,
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
