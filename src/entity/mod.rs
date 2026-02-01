pub mod intent;
pub mod lifestate;
pub mod perception;
pub mod phase;
pub mod species;

use crate::{
    entity::{
        intent::Intent, lifestate::LifeState, perception::*, species::Species,
    },
    map::position::Position,
};

/// Canlının temel alacağı arayüz
pub trait Entity {
    /// Canlının yaşam durumu (genetik + dinamik)
    fn life(&self) -> &LifeState;
    fn life_mut(&mut self) -> &mut LifeState;

    /// Varlık türü
    fn species(&self) -> Species;

    /// Karar verme (sadece okuma yapmalı)
    fn make_intent(&self, view: Perception) -> Intent;

    /// Tek tick güncellemesi
    /// World'un işini kolaylaştırmak için var;
    fn tick(&mut self);

    /// Canlının kendi türünden yeni bir üye (yavru) oluşturmasını sağlar.
    /// World bu metodu çağırır ama dönen somut türü (Herbivore vs.) bilmez.
    fn reproduce(&self, pos: Position) -> Box<dyn Entity>;
}
