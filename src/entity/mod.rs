pub mod intent;
pub mod lifestate;
pub mod perception;
pub mod phase;

use crate::{
    entity::{intent::Intent, lifestate::LifeState, perception::*, phase::EntityPhase},
    map::position::Position,
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

    /// Karar verme (sadece okuma yapmalı)
    fn make_intent(&self, ctx: Perception) -> Intent;

    /// Tek tick güncellemesi
    fn tick(&mut self);

    /// Canlının kendi türünden yeni bir üye (yavru) oluşturmasını sağlar.
    /// World bu metodu çağırır ama dönen somut türü (Herbivore vs.) bilmez.
    fn reproduce(&self, new_id: usize, pos: Position) -> Box<dyn Entity>;
}
