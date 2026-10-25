//! Candidate support module for Feature 6: Corpse Decay, Scavenging Scent & Spoilage Risk.
//!
//! Encompasses:
//! 1. Gradual Corpse Decay with Freshness Stages (Fresh, Aging, Putrid)
//! 2. Scent signal emission scaling with decay stage
//! 3. Ecological niche separation between Carnivore (predator) and Omnivore (scavenger)
//! 4. Exact stock-flow energy accounting:
//!    initial_corpse_energy == eaten_by_carnivores + eaten_by_omnivores + decayed_energy + remaining_corpse_energy

use curious::{
    entity::species::Species,
    map::movement::Position,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SignalKind {
    Footstep,
    Scent,
    Sound,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum FreshnessStage {
    Fresh,
    Aging,
    Putrid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScentSignal {
    pub origin: Position,
    pub kind: SignalKind,
    pub intensity: u16,
    pub radius: u8,
    pub decay_per_tick: u16,
    pub emitted_tick: usize,
    pub stage: FreshnessStage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MealOutcome {
    pub energy_gained: usize,
    pub health_penalty: usize,
    pub stamina_penalty: usize,
    pub stage: FreshnessStage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CorpseScavengingAudit {
    pub initial_corpse_energy: usize,
    pub eaten_by_carnivores: usize,
    pub eaten_by_omnivores: usize,
    pub decayed_energy: usize,
    pub remaining_corpse_energy: usize,
}

impl CorpseScavengingAudit {
    /// Validates exact conservation of corpse energy:
    /// initial_corpse_energy == eaten_by_carnivores + eaten_by_omnivores + decayed_energy + remaining_corpse_energy
    pub fn is_balanced(&self) -> bool {
        self.initial_corpse_energy
            == self
                .eaten_by_carnivores
                .saturating_add(self.eaten_by_omnivores)
                .saturating_add(self.decayed_energy)
                .saturating_add(self.remaining_corpse_energy)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecayingCorpse {
    pub id: usize,
    pub pos: Position,
    pub initial_energy: usize,
    pub remaining_energy: usize,
    pub decay_rate_per_tick: usize,
    pub age_ticks: usize,

    // Ledger counters
    pub eaten_by_carnivores: usize,
    pub eaten_by_omnivores: usize,
    pub total_decayed_energy: usize,
}

impl DecayingCorpse {
    pub fn new(id: usize, pos: Position, initial_energy: usize, decay_rate_per_tick: usize) -> Self {
        Self {
            id,
            pos,
            initial_energy,
            remaining_energy: initial_energy,
            decay_rate_per_tick,
            age_ticks: 0,
            eaten_by_carnivores: 0,
            eaten_by_omnivores: 0,
            total_decayed_energy: 0,
        }
    }

    /// Determines the freshness stage based on remaining energy ratio.
    /// - Fresh: >= 70% of initial energy
    /// - Aging: 30% .. 70% of initial energy
    /// - Putrid: < 30% of initial energy
    pub fn freshness_stage(&self) -> FreshnessStage {
        if self.initial_energy == 0 || self.remaining_energy == 0 {
            return FreshnessStage::Putrid;
        }
        let pct = (self.remaining_energy * 100) / self.initial_energy;
        if pct >= 70 {
            FreshnessStage::Fresh
        } else if pct >= 30 {
            FreshnessStage::Aging
        } else {
            FreshnessStage::Putrid
        }
    }

    /// Generates the bounded scent signal corresponding to the current decay stage.
    /// Scent radius and intensity expand as the corpse rots, attracting scavengers.
    pub fn emit_scent(&self, current_tick: usize) -> Option<ScentSignal> {
        if self.remaining_energy == 0 {
            return None;
        }
        let stage = self.freshness_stage();
        let (radius, intensity, decay_per_tick) = match stage {
            FreshnessStage::Fresh => (3, 10, 2),
            FreshnessStage::Aging => (6, 25, 3),
            FreshnessStage::Putrid => (10, 50, 5),
        };

        Some(ScentSignal {
            origin: self.pos,
            kind: SignalKind::Scent,
            intensity,
            radius,
            decay_per_tick,
            emitted_tick: current_tick,
            stage,
        })
    }

    /// Executes one tick of natural corpse decay.
    /// Returns the energy decayed this tick.
    pub fn decay_tick(&mut self) -> usize {
        if self.remaining_energy == 0 {
            return 0;
        }
        self.age_ticks += 1;
        let decay_amount = self.decay_rate_per_tick.min(self.remaining_energy);
        self.remaining_energy = self.remaining_energy.saturating_sub(decay_amount);
        self.total_decayed_energy = self.total_decayed_energy.saturating_add(decay_amount);
        decay_amount
    }

    /// Resolves feeding on the corpse by an animal of the specified species.
    /// Herbivores cannot eat meat.
    /// Omnivores (scavengers) have scavenger resistance (reduced/zero spoilage penalties).
    /// Carnivores (fresh-kill predators) suffer severe spoilage penalties on Putrid corpses.
    pub fn feed(
        &mut self,
        species: Species,
        requested_amount: usize,
    ) -> Result<MealOutcome, &'static str> {
        if species == Species::Herbivore {
            return Err("herbivores cannot feed on meat");
        }
        if self.remaining_energy == 0 {
            return Err("corpse is exhausted");
        }

        let stage = self.freshness_stage();
        let actual_eaten = requested_amount.min(self.remaining_energy);
        self.remaining_energy = self.remaining_energy.saturating_sub(actual_eaten);

        match species {
            Species::Carnivore => {
                self.eaten_by_carnivores = self.eaten_by_carnivores.saturating_add(actual_eaten);
            }
            Species::Omnivore => {
                self.eaten_by_omnivores = self.eaten_by_omnivores.saturating_add(actual_eaten);
            }
            Species::Herbivore => unreachable!(),
        }

        // Calculate spoilage penalties according to ecological niche
        let (health_penalty, stamina_penalty) = match (stage, species) {
            // Fresh corpses provide clean nutrition with 0 penalty to both
            (FreshnessStage::Fresh, _) => (0, 0),

            // Aging corpses: Carnivores experience mild discomfort; Omnivores resist
            (FreshnessStage::Aging, Species::Carnivore) => (2, 3),
            (FreshnessStage::Aging, Species::Omnivore) => (0, 0),

            // Putrid corpses: Carnivores suffer severe spoilage illness; Omnivores resist
            (FreshnessStage::Putrid, Species::Carnivore) => (6, 8),
            (FreshnessStage::Putrid, Species::Omnivore) => (0, 0),

            (_, Species::Herbivore) => unreachable!(),
        };

        Ok(MealOutcome {
            energy_gained: actual_eaten,
            health_penalty,
            stamina_penalty,
            stage,
        })
    }

    /// Checks if corpse has completely decayed or been eaten to exhaustion.
    pub fn is_exhausted(&self) -> bool {
        self.remaining_energy == 0
    }

    /// Returns the exact stock-flow audit of this corpse.
    pub fn audit(&self) -> CorpseScavengingAudit {
        CorpseScavengingAudit {
            initial_corpse_energy: self.initial_energy,
            eaten_by_carnivores: self.eaten_by_carnivores,
            eaten_by_omnivores: self.eaten_by_omnivores,
            decayed_energy: self.total_decayed_energy,
            remaining_corpse_energy: self.remaining_energy,
        }
    }
}
