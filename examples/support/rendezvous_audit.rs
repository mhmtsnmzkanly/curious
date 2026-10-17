//! Opt-in per-decision audit; receives an immutable world, never consumes RNG.
use curious::{entity::species::Species, simulation::SimulationEvent, world::World};
use serde_json::json;
use std::{
    fs::OpenOptions,
    io::{BufWriter, Write},
};
pub struct Audit(BufWriter<std::fs::File>);
impl Audit {
    pub fn new(path: &str) -> Self {
        Self(BufWriter::new(
            OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(path)
                .unwrap(),
        ))
    }
    pub fn observe(&mut self, w: &World, events: &[SimulationEvent]) {
        for trace in &w.last_decisions {
            if trace.species != Species::Herbivore {
                continue;
            }
            let slot = w.entities.iter().find(|e| e.id == trace.id).unwrap();
            writeln!(self.0,"{}",json!({"type":"decision","trace":trace,"ready":trace.before.can_reproduce(),"blocker":trace.before.reproduction_blocker(),"social":slot.memory.social,"companions":slot.memory.companions,"dangers":slot.memory.dangers})).unwrap();
        }
        for event in events {
            let relevant = match event {
                SimulationEvent::EntityBorn { species, .. }
                | SimulationEvent::EntityDied { species, .. } => *species == Species::Herbivore,
                SimulationEvent::EntityMated { parent1, .. } => w
                    .entities
                    .iter()
                    .any(|e| e.id == *parent1 && e.base.species() == Species::Herbivore),
                _ => false,
            };
            if relevant {
                writeln!(
                    self.0,
                    "{}",
                    json!({"type":"event","tick":w.tick_counter,"event":event})
                )
                .unwrap();
            }
        }
    }
}
