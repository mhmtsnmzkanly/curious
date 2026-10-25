//! Candidate support module for Feature 4: Dynamic Habitat Engineering.
//!
//! Encompasses:
//! 1. Burrow Digging & Durability Decay (with combat stress decay and collapse)
//! 2. Overgrazing Degradation (consecutive depletions halving replenishment)
//! 3. Seed Dispersal & Maturation (well-fed herbivores deposit seeds that mature into Food)
//! 4. Strict stock-flow energy accounting.

use curious::{
    entity::species::Species,
    map::movement::Position,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const DIG_ENERGY_COST: usize = 12;
pub const DIG_STAMINA_COST: usize = 6;
pub const INITIAL_BURROW_DURABILITY: u16 = 50;
pub const DEFAULT_COMBAT_STRESS_DECAY: u16 = 4; // Extra durability decay when adjacent to combat

pub const DEFAULT_OVERGRAZED_THRESHOLD: u32 = 3;
pub const DEFAULT_OVERGRAZED_COOLDOWN: usize = 10;

pub const SEED_ENERGY_COST: usize = 8;
pub const DEFAULT_SEED_MATURATION_TICKS: usize = 5;
pub const DEFAULT_SPROUT_AMOUNT: usize = 15;

fn pos_key(p: Position) -> (isize, isize) {
    (p.x, p.y)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BurrowSite {
    pub pos: Position,
    pub durability: u16,
    pub max_durability: u16,
}

impl BurrowSite {
    pub fn new(pos: Position) -> Self {
        Self {
            pos,
            durability: INITIAL_BURROW_DURABILITY,
            max_durability: INITIAL_BURROW_DURABILITY,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OvergrazingStatus {
    Normal,
    Overgrazed { recovery_cooldown: usize },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoodSiteState {
    pub pos: Position,
    pub capacity: usize,
    pub replenish: usize,
    pub interval: usize,
    pub consecutive_depletions: u32,
    pub status: OvergrazingStatus,
}

impl FoodSiteState {
    pub fn new(pos: Position, capacity: usize, replenish: usize, interval: usize) -> Self {
        Self {
            pos,
            capacity,
            replenish,
            interval,
            consecutive_depletions: 0,
            status: OvergrazingStatus::Normal,
        }
    }

    /// Returns the effective replenishment rate. If overgrazed, replenishment is halved (minimum 1 if replenish >= 2).
    pub fn effective_replenish(&self) -> usize {
        match self.status {
            OvergrazingStatus::Normal => self.replenish,
            OvergrazingStatus::Overgrazed { .. } => (self.replenish / 2).max(1),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeededPatch {
    pub pos: Position,
    pub deposited_by: usize,
    pub maturation_ticks: usize,
    pub sprout_amount: usize,
    pub seed_energy_cost: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DigError {
    NotHerbivore,
    CannotBurrow,
    InsufficientEnergy { available: usize, required: usize },
    InsufficientStamina { available: usize, required: usize },
    CellNotEmpty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SeedError {
    NotHerbivore,
    InsufficientEnergyReserve { available: usize, threshold: usize },
    InsufficientEnergyCost { available: usize, required: usize },
    CellNotEmpty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DynamicHabitatLedger {
    pub initial: usize,
    pub replenished: usize,
    pub sprouted: usize,
    pub eaten: usize,
    pub seed_cost: usize,
    pub current_food: usize,
    pub net_balance: usize,
}

impl DynamicHabitatLedger {
    /// Validates strict stock-flow closure:
    /// initial + replenished + sprouted - eaten - seed_cost == net_balance
    pub fn is_balanced(&self) -> bool {
        self.initial
            .saturating_add(self.replenished)
            .saturating_add(self.sprouted)
            == self
                .net_balance
                .saturating_add(self.eaten)
                .saturating_add(self.seed_cost)
            && self
                .initial
                .saturating_add(self.replenished)
                .saturating_add(self.sprouted)
                .saturating_sub(self.eaten)
                == self.current_food
    }
}

#[derive(Debug, Clone, Default)]
pub struct DynamicHabitatEngine {
    pub burrows: BTreeMap<(isize, isize), BurrowSite>,
    pub food_sites: BTreeMap<(isize, isize), FoodSiteState>,
    pub seeded_patches: Vec<SeededPatch>,
    pub food_cells: BTreeMap<(isize, isize), usize>,

    // Stock-flow counters
    pub initial_food: usize,
    pub total_replenished: usize,
    pub total_sprouted: usize,
    pub total_eaten: usize,
    pub total_seed_cost: usize,
}

impl DynamicHabitatEngine {
    pub fn new() -> Self {
        Self::default()
    }

    /// Checks if a burrow exists at `pos`.
    pub fn has_burrow(&self, pos: Position) -> bool {
        self.burrows.contains_key(&pos_key(pos))
    }

    /// Gets burrow durability at `pos`.
    pub fn burrow_durability(&self, pos: Position) -> Option<u16> {
        self.burrows.get(&pos_key(pos)).map(|b| b.durability)
    }

    /// Sets burrow durability at `pos` (for test fixtures).
    pub fn set_burrow_durability(&mut self, pos: Position, durability: u16) {
        if let Some(burrow) = self.burrows.get_mut(&pos_key(pos)) {
            burrow.durability = durability;
        }
    }

    /// Checks if a food site exists at `pos`.
    pub fn has_food_site(&self, pos: Position) -> bool {
        self.food_sites.contains_key(&pos_key(pos))
    }

    /// Gets food site state at `pos`.
    pub fn food_site(&self, pos: Position) -> Option<&FoodSiteState> {
        self.food_sites.get(&pos_key(pos))
    }

    /// Gets food cell amount at `pos`.
    pub fn food_amount(&self, pos: Position) -> Option<usize> {
        self.food_cells.get(&pos_key(pos)).copied()
    }

    /// Registers an initial food site and corresponding food stock.
    pub fn add_food_site(
        &mut self,
        pos: Position,
        initial_food: usize,
        capacity: usize,
        replenish: usize,
        interval: usize,
    ) {
        let k = pos_key(pos);
        self.food_sites.insert(
            k,
            FoodSiteState::new(pos, capacity, replenish, interval),
        );
        self.food_cells.insert(k, initial_food);
        self.initial_food = self.initial_food.saturating_add(initial_food);
    }

    /// Evaluates and executes burrow digging.
    pub fn dig_burrow(
        &mut self,
        _entity_id: usize,
        species: Species,
        can_burrow: bool,
        energy: usize,
        stamina: usize,
        target_pos: Position,
        cell_is_empty: bool,
    ) -> Result<BurrowSite, DigError> {
        if species != Species::Herbivore {
            return Err(DigError::NotHerbivore);
        }
        if !can_burrow {
            return Err(DigError::CannotBurrow);
        }
        if energy < DIG_ENERGY_COST {
            return Err(DigError::InsufficientEnergy {
                available: energy,
                required: DIG_ENERGY_COST,
            });
        }
        if stamina < DIG_STAMINA_COST {
            return Err(DigError::InsufficientStamina {
                available: stamina,
                required: DIG_STAMINA_COST,
            });
        }
        let k = pos_key(target_pos);
        if !cell_is_empty || self.burrows.contains_key(&k) {
            return Err(DigError::CellNotEmpty);
        }

        let burrow = BurrowSite::new(target_pos);
        self.burrows.insert(k, burrow);
        Ok(burrow)
    }

    /// Decays burrows by 1 durability naturally, plus combat_stress_decay if adjacent to combat.
    /// Collapses to Empty when durability reaches 0. Returns positions of collapsed burrows.
    pub fn decay_burrows(
        &mut self,
        adjacent_combat_positions: &[Position],
        combat_stress_decay: u16,
    ) -> Vec<Position> {
        let mut collapsed = Vec::new();
        self.burrows.retain(|_, burrow| {
            let pos = burrow.pos;
            let extra_decay = if adjacent_combat_positions.iter().any(|c| {
                pos.x.abs_diff(c.x) <= 1 && pos.y.abs_diff(c.y) <= 1
            }) {
                combat_stress_decay
            } else {
                0
            };
            let total_decay = 1u16.saturating_add(extra_decay);
            burrow.durability = burrow.durability.saturating_sub(total_decay);
            if burrow.durability == 0 {
                collapsed.push(pos);
                false
            } else {
                true
            }
        });
        collapsed
    }

    /// Records a grazing action by an animal at `pos`.
    /// Halves replenishment if consecutive depletions >= 3 (Overgrazed state).
    pub fn record_grazing(
        &mut self,
        pos: Position,
        requested_amount: usize,
    ) -> Result<usize, &'static str> {
        let k = pos_key(pos);
        let current_stock = self.food_cells.get_mut(&k).ok_or("no food cell at position")?;
        let actual_eaten = requested_amount.min(*current_stock);
        *current_stock = current_stock.saturating_sub(actual_eaten);
        self.total_eaten = self.total_eaten.saturating_add(actual_eaten);

        if let Some(site) = self.food_sites.get_mut(&k) {
            if *current_stock == 0 {
                site.consecutive_depletions = site.consecutive_depletions.saturating_add(1);
                if site.consecutive_depletions >= DEFAULT_OVERGRAZED_THRESHOLD {
                    site.status = OvergrazingStatus::Overgrazed {
                        recovery_cooldown: DEFAULT_OVERGRAZED_COOLDOWN,
                    };
                }
            }
        }

        Ok(actual_eaten)
    }

    /// Well-fed herbivores (`energy >= max_energy * 3 / 4`) deposit a SeededPatch on an Empty cell.
    pub fn deposit_seed(
        &mut self,
        entity_id: usize,
        species: Species,
        energy: usize,
        max_energy: usize,
        target_pos: Position,
        cell_is_empty: bool,
    ) -> Result<SeededPatch, SeedError> {
        if species != Species::Herbivore {
            return Err(SeedError::NotHerbivore);
        }
        let threshold = (max_energy * 3) / 4;
        if energy < threshold {
            return Err(SeedError::InsufficientEnergyReserve {
                available: energy,
                threshold,
            });
        }
        if energy < SEED_ENERGY_COST {
            return Err(SeedError::InsufficientEnergyCost {
                available: energy,
                required: SEED_ENERGY_COST,
            });
        }
        let k = pos_key(target_pos);
        if !cell_is_empty
            || self.food_cells.contains_key(&k)
            || self.burrows.contains_key(&k)
            || self.seeded_patches.iter().any(|s| s.pos == target_pos)
        {
            return Err(SeedError::CellNotEmpty);
        }

        let patch = SeededPatch {
            pos: target_pos,
            deposited_by: entity_id,
            maturation_ticks: DEFAULT_SEED_MATURATION_TICKS,
            sprout_amount: DEFAULT_SPROUT_AMOUNT,
            seed_energy_cost: SEED_ENERGY_COST,
        };
        self.seeded_patches.push(patch);
        self.total_seed_cost = self.total_seed_cost.saturating_add(SEED_ENERGY_COST);
        Ok(patch)
    }

    /// Updates food replenishment and seed maturation for one tick.
    pub fn tick(&mut self, current_tick: usize) -> Vec<Position> {
        // 1. Food replenishment
        for (&k, site) in self.food_sites.iter_mut() {
            // Update cooldown if overgrazed
            if let OvergrazingStatus::Overgrazed { recovery_cooldown } = &mut site.status {
                if *recovery_cooldown > 0 {
                    *recovery_cooldown -= 1;
                    if *recovery_cooldown == 0 {
                        site.status = OvergrazingStatus::Normal;
                        site.consecutive_depletions = 0;
                    }
                }
            }

            if site.interval > 0 && current_tick % site.interval == 0 {
                let current_food = self.food_cells.entry(k).or_insert(0);
                let replenish_rate = site.effective_replenish();
                let space = site.capacity.saturating_sub(*current_food);
                let added = replenish_rate.min(space);
                if added > 0 {
                    *current_food = current_food.saturating_add(added);
                    self.total_replenished = self.total_replenished.saturating_add(added);
                }
            }
        }

        // 2. Seed maturation & sprouting
        let mut sprouted_positions = Vec::new();
        let mut remaining_patches = Vec::new();
        for mut patch in self.seeded_patches.drain(..) {
            if patch.maturation_ticks > 0 {
                patch.maturation_ticks -= 1;
            }
            if patch.maturation_ticks == 0 {
                // Sprouts into renewable food cell
                let k = pos_key(patch.pos);
                self.food_cells.insert(k, patch.sprout_amount);
                self.food_sites.insert(
                    k,
                    FoodSiteState::new(patch.pos, 20, 2, 2),
                );
                self.total_sprouted = self.total_sprouted.saturating_add(patch.sprout_amount);
                sprouted_positions.push(patch.pos);
            } else {
                remaining_patches.push(patch);
            }
        }
        self.seeded_patches = remaining_patches;

        sprouted_positions
    }

    /// Computes the stock-flow ledger for dynamic habitat.
    pub fn audit_stock_flow(&self) -> DynamicHabitatLedger {
        let current_food_stock: usize = self.food_cells.values().sum();
        let total_in = self
            .initial_food
            .saturating_add(self.total_replenished)
            .saturating_add(self.total_sprouted);
        let net_balance = total_in
            .saturating_sub(self.total_eaten)
            .saturating_sub(self.total_seed_cost);
        DynamicHabitatLedger {
            initial: self.initial_food,
            replenished: self.total_replenished,
            sprouted: self.total_sprouted,
            eaten: self.total_eaten,
            seed_cost: self.total_seed_cost,
            current_food: current_food_stock,
            net_balance,
        }
    }
}
