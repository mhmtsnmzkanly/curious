pub mod lifestate;
pub mod perception;
pub mod phase;

use crate::{
    entity::{lifestate::LifeState, perception::Perception, phase::EntityPhase},
    world::WorldView,
    world_types::{Action, Position},
};

/// ===============================
/// CANLI ARAYÜZÜ
/// ===============================
pub trait Entity {
    /// Canlıya ait benzersiz kimlik
    fn id(&self) -> usize;

    /// Canlının bulunduğu konum
    fn position(&self) -> Position;
    fn position_mut(&mut self) -> &mut Position;

    /// Canlının yaşam durumu (genetik + dinamik)
    fn life(&self) -> &LifeState;
    fn life_mut(&mut self) -> &mut LifeState;

    // Varlık durumu
    fn phase(&self) -> EntityPhase;
    fn phase_mut(&mut self) -> &mut EntityPhase;

    // Algılama
    fn perception(&self) -> &Perception;
    fn perception_mut(&mut self) -> &mut Perception;

    /// Karar verme (sadece okuma yapmalı)
    fn think(&self, ctx: &WorldView) -> Action;

    /// Tek tick güncellemesi
    fn tick(&mut self) {
        // Faz kontrolü
        match self.phase_mut() {
            EntityPhase::Sleeping { remaining } => {
                if *remaining > 0 {
                    *remaining -= 1;
                    return;
                } else {
                    *self.phase_mut() = EntityPhase::Active;
                }
            }
            EntityPhase::Corpse | EntityPhase::Removed => {
                return;
            }
            _ => {}
        }

        // Yaşam güncellemesi
        self.life_mut().tick();

        // Ölüm kontrolü
        if !self.life().is_alive() {
            *self.phase_mut() = EntityPhase::Corpse;
        }
    }

    /// Alınan kuralı uygula
    fn apply(&mut self, action: Action);
}
