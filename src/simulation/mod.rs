pub mod event;
pub mod snapshot;
pub use event::SimulationEvent;
pub use snapshot::{WorldSnapshot, EntitySnapshot};

use crate::world::World;
use crate::entity::intent::Intent;

/// Simülasyon motoru. World ile dış dünya (input/render) arasındaki köprü.
pub struct SimulationEngine {
    world: World,
}

impl SimulationEngine {
    pub fn new(world: World) -> Self {
        Self { world }
    }

    /// Bir simülasyon adımı işletir ve oluşan olayları döner.
    pub fn step(&mut self) -> Vec<crate::simulation::SimulationEvent> {
        self.world.tick()
    }

    /// Bir birime dışarıdan komut iletir.
    pub fn send_command(&mut self, entity_id: usize, intent: Intent) {
        self.world.push_command(entity_id, intent);
    }

    /// Simülasyonun o anki durumunun tam bir kopyasını (Snapshot) döner.
    pub fn snapshot(&self) -> crate::simulation::snapshot::WorldSnapshot {
        let mut entity_snapshots = Vec::new();
        for slot in &self.world.entities {
            entity_snapshots.push(crate::simulation::snapshot::EntitySnapshot {
                id: slot.id,
                pos: slot.pos,
                species: slot.base.species(),
                phase: slot.phase.clone(),
                life: slot.base.life().clone(),
            });
        }

        let mut resources = Vec::new();
        for y in self.world.map.min_y..=self.world.map.max_y {
            for x in self.world.map.min_x..=self.world.map.max_x {
                let pos = (x, y).into();
                if let Some(cell) = self.world.map.cell(pos) {
                    match cell {
                        crate::map::cell::Cell::Food { amount } | crate::map::cell::Cell::Water { amount } => {
                            resources.push((pos, *amount));
                        }
                        _ => {}
                    }
                }
            }
        }

        crate::simulation::snapshot::WorldSnapshot {
            tick: self.world.tick_counter,
            entities: entity_snapshots,
            map_resources: resources,
        }
    }

    /// World referansı (okuma amaçlı)
    pub fn world(&self) -> &World {
        &self.world
    }

    /// World mutable referansı (özel durumlar için)
    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }
}
