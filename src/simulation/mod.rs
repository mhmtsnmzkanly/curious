pub mod checkpoint;
pub mod event;
pub mod snapshot;
pub use event::SimulationEvent;
pub use snapshot::{EntitySnapshot, WorldSnapshot};

use crate::entity::intent::Intent;
use crate::world::World;

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
        checkpoint::State::capture(&self.world)
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
