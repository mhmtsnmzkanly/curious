//! Diagnostic counterfactuals from a copied checkpoint; never a live-world fix.
use curious::{
    entity::species::Species,
    simulation::{SimulationEvent, checkpoint::State},
};
use serde_json::json;
use std::{collections::BTreeMap, env, fs};
#[path = "support/rendezvous_audit.rs"]
mod rendezvous_audit;
fn main() {
    let args: Vec<_> = env::args().collect();
    let mut w: curious::world::World =
        serde_json::from_slice::<State>(&fs::read(&args[1]).unwrap())
            .unwrap()
            .restore()
            .unwrap();
    assert!(
        !w.genetics.enabled,
        "counterfactual protocol requires fixed genetics"
    );
    assert!(
        w.command_inputs.is_empty(),
        "counterfactual protocol excludes pending external commands"
    );
    let ticks: usize = args[2].parse().unwrap();
    let treatment = args[3].as_str();
    assert!(
        [
            "unchanged",
            "reserve-off",
            "without-predators",
            "rendezvous"
        ]
        .contains(&treatment)
    );
    if treatment == "rendezvous" {
        w.policy.mate_rendezvous = true;
    }
    if treatment == "reserve-off" {
        w.policy.reserve_opportunities = false;
    }
    if treatment == "without-predators" {
        w.entities
            .retain(|e| e.base.species() != Species::Carnivore);
    }
    let mut audit = args.get(5).map(|path| rendezvous_audit::Audit::new(path));
    if audit.is_some() {
        w.observation.full_all = true;
    }
    let start = w.tick_counter;
    let mut reasons = BTreeMap::<String, usize>::new();
    let mut births = BTreeMap::<String, usize>::new();
    let mut deaths = BTreeMap::<String, usize>::new();
    for _ in 0..ticks {
        for e in &mut w.entities {
            e.learner.enabled = false;
            e.memory.companions_enabled = true;
        }
        let events = w.tick();
        if let Some(audit) = &mut audit {
            audit.observe(&w, &events);
        }
        for event in events {
            match event {
                SimulationEvent::EntityBorn { species, .. } => {
                    *births.entry(format!("{species:?}")).or_default() += 1
                }
                SimulationEvent::EntityDied { species, cause, .. } => {
                    *deaths.entry(format!("{species:?}.{cause:?}")).or_default() += 1
                }
                _ => {}
            }
        }
        for trace in &w.last_decisions {
            if trace.species == Species::Herbivore {
                *reasons
                    .entry(format!("{:?}", trace.decision.reason))
                    .or_default() += 1;
            }
        }
        if w.tick_counter % 100 == 0 || w.tick_counter == start + ticks {
            let mut alive = BTreeMap::<String, usize>::new();
            for e in &w.entities {
                if e.base.life().health > 0 {
                    *alive.entry(format!("{:?}", e.base.species())).or_default() += 1;
                }
            }
            println!(
                "{}",
                json!({"tick":w.tick_counter,"start":start,"treatment":treatment,"mate_rendezvous":w.policy.mate_rendezvous,"herbivore_reasons":reasons,"alive":alive,"births":births,"deaths":deaths,"rng":w.rng.state()})
            );
        }
    }
    if let Some(path) = args.get(4) {
        use std::io::Write;
        let mut out = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .unwrap();
        out.write_all(&serde_json::to_vec(&State::capture(&w)).unwrap())
            .unwrap();
    }
}
