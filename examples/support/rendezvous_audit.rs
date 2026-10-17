//! Opt-in per-decision audit; receives an immutable world, never consumes RNG.
use curious::{entity::species::Species, simulation::SimulationEvent, world::World};
use serde_json::json;
use std::{
    fs::OpenOptions,
    io::{BufWriter, Write},
};
pub struct Audit(BufWriter<std::fs::File>, Species, Vec<serde_json::Value>);
impl Audit {
    pub fn new(path: &str, species: Species) -> Self {
        Self(
            BufWriter::new(
                OpenOptions::new()
                    .create_new(true)
                    .write(true)
                    .open(path)
                    .unwrap(),
            ),
            species,
            Vec::new(),
        )
    }
    pub fn before_tick(&mut self, w: &World) {
        self.2=w.entities.iter().filter(|e|matches!(e.phase,curious::entity::phase::EntityPhase::Corpse{remaining} if remaining>1) && e.corpse_energy>0)
            .map(|e|json!({"id":e.id,"pos":e.pos,"amount":e.corpse_energy,"burrow":matches!(w.map.cell(e.pos),Some(curious::map::cell::Cell::Burrow))})).collect();
    }
    pub fn observe(&mut self, w: &World, events: &[SimulationEvent]) {
        for trace in &w.last_decisions {
            if trace.species != self.1 {
                continue;
            }
            let slot = w.entities.iter().find(|e| e.id == trace.id).unwrap();
            let hidden_adjacent: Vec<_> = self
                .2
                .iter()
                .filter(|c| {
                    let p = &c["pos"];
                    trace.pos.x.abs_diff(p["x"].as_i64().unwrap() as isize) <= 1
                        && trace.pos.y.abs_diff(p["y"].as_i64().unwrap() as isize) <= 1
                        && !trace.perception.as_ref().is_some_and(|v| {
                            v.foods
                                .iter()
                                .any(|f| f.corpse_id == c["id"].as_u64().map(|id| id as usize))
                        })
                })
                .collect();
            writeln!(self.0,"{}",json!({"type":"decision","trace":trace,"ready":trace.before.can_reproduce(),"blocker":trace.before.reproduction_blocker(),"social":slot.memory.social,"companions":slot.memory.companions,"dangers":slot.memory.dangers,"hidden_adjacent_corpses":hidden_adjacent})).unwrap();
        }
        for event in events {
            let relevant = match event {
                SimulationEvent::EntityBorn { species, .. }
                | SimulationEvent::EntityDied { species, .. } => *species == self.1,
                SimulationEvent::EntityMated { parent1, .. } => w
                    .entities
                    .iter()
                    .any(|e| e.id == *parent1 && e.base.species() == self.1),
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
