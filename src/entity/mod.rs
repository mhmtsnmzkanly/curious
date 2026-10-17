pub mod behavior;
pub mod controller;
pub mod decision;
pub mod foraging;
pub mod genetics;
pub mod hunting;
pub mod instinct;
pub mod intent;
pub mod learning;
pub mod lifestate;
pub mod memory;
pub mod perception;
pub mod phase;
pub mod social;
pub mod species;

use crate::entity::{intent::Intent, lifestate::LifeState, perception::*, species::Species};

/// Canlının temel alacağı arayüz
pub trait Entity {
    /// Canlının yaşam durumu (genetik + dinamik)
    fn life(&self) -> &LifeState;
    fn life_mut(&mut self) -> &mut LifeState;

    /// Varlık türü
    fn species(&self) -> Species;

    fn can_burrow(&self) -> bool {
        self.species() == Species::Herbivore
    }

    fn combat(&self) -> crate::rules::combat::CombatProfile {
        self.species().combat_profile()
    }

    /// Karar verme (sadece okuma yapmalı)
    fn make_intent(&self, view: Perception, rng: &mut crate::random::Random) -> Intent;

    /// Custom policies may supply their own evidence; legacy implementations remain usable.
    fn decide(&self, view: Perception, rng: &mut crate::random::Random) -> decision::Decision {
        decision::Decision::new(self.make_intent(view, rng), decision::Reason::CustomPolicy)
    }

    /// Tek tick güncellemesi
    /// World'un işini kolaylaştırmak için var;
    fn tick(&mut self);

    /// Canlının kendi türünden yeni bir üye (yavru) oluşturmasını sağlar.
    /// World bu metodu çağırır ama dönen somut türü (Herbivore vs.) bilmez.
    fn reproduce(&self) -> Box<dyn Entity>;
}
