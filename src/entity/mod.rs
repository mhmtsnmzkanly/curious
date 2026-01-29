pub mod lifestate;

use crate::{
    entity::lifestate::LifeState,
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

    /// Konumun değiştirilebilir hali
    fn position_mut(&mut self) -> &mut Position;

    /// Canlının yaşam durumu (genetik + dinamik)
    fn life(&self) -> &LifeState;

    /// Değiştirilebilir yaşam durumu
    fn life_mut(&mut self) -> &mut LifeState;

    /// Karar verme (sadece okuma yapmalı)
    fn think(&self, ctx: &WorldView) -> Action;

    /// Tek tick güncellemesi
    fn tick(&mut self) {
        self.life_mut().tick();
    }

    /// Alınan kuralı uygula
    fn apply(&mut self, action: Action);
}
