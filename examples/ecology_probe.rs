//! Fast deterministic ecology diagnostics; does not touch any runtime/history.
use curious::{scenario::Scenario, simulation::SimulationEvent};
use std::{collections::BTreeMap, env, fs, time::Instant};
fn main() {
    let args: Vec<_> = env::args().collect();
    let file = &args[1];
    let ticks: usize = args[2].parse().unwrap();
    for seed in args[3].split(',').map(|s| s.parse::<u64>().unwrap()) {
        let mut w = serde_json::from_str::<Scenario>(&fs::read_to_string(file).unwrap())
            .unwrap()
            .build(seed)
            .unwrap();
        let mut births: BTreeMap<String, usize> = BTreeMap::new();
        let mut breeders = BTreeMap::new();
        let mut generations = BTreeMap::new();
        let mut last_alive = BTreeMap::new();
        let start = Instant::now();
        for _ in 0..ticks {
            let events = w.tick();
            for e in events {
                match e {
                    SimulationEvent::EntityBorn { species, .. } => {
                        *births.entry(format!("{:?}", species)).or_insert(0) += 1;
                    }
                    SimulationEvent::EntityMated {
                        parent1, parent2, ..
                    } => {
                        for id in [parent1, parent2] {
                            let p = w.entities.iter().find(|e| e.id == id).unwrap();
                            if p.generation > 0 {
                                breeders.insert(id, format!("{:?}", p.base.species()));
                            }
                        }
                    }
                    _ => {}
                }
            }
            for e in &w.entities {
                let name = format!("{:?}", e.base.species());
                generations
                    .entry(name.clone())
                    .and_modify(|g: &mut usize| *g = (*g).max(e.generation))
                    .or_insert(e.generation);
                if e.base.life().health > 0 {
                    last_alive.insert(name, w.tick_counter);
                }
            }
        }
        let mut alive: BTreeMap<String, usize> = BTreeMap::new();
        for e in &w.entities {
            if e.base.life().health > 0 {
                *alive.entry(format!("{:?}", e.base.species())).or_insert(0) += 1;
            }
        }
        let mut reproduced: BTreeMap<String, usize> = BTreeMap::new();
        for species in breeders.values() {
            *reproduced.entry(species.clone()).or_insert(0) += 1;
        }
        println!(
            "{}",
            serde_json::json!({"scenario":file,"seed":seed,"ticks":ticks,"alive":alive,"last_alive":last_alive,"births":births,"descendant_breeders":reproduced,"max_generation":generations,"seconds":start.elapsed().as_secs_f64()})
        );
    }
}
