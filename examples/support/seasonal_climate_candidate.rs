//! Candidate support module for Feature 5: Seasonal Climate Cycle & Hibernation / Thermoregulation.
//!
//! Encompasses:
//! 1. Deterministic SeasonCycle (Spring, Summer, Autumn, Winter)
//! 2. Regional temperature & harshness based on season and latitude
//! 3. Winter resource replenishment scaling (winter_replenish_per_mille)
//! 4. Hibernation / Sleep strategy in burrows with cold immunity and metabolic conservation.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Season {
    Spring,
    Summer,
    Autumn,
    Winter,
}

impl Season {
    pub fn is_winter(&self) -> bool {
        matches!(self, Season::Winter)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeasonCycle {
    pub year_period: usize,
}

impl SeasonCycle {
    pub fn new(year_period: usize) -> Self {
        assert!(year_period >= 4, "year_period must be at least 4");
        Self { year_period }
    }

    /// Determines the season at a given tick deterministically.
    pub fn season_at(&self, tick: usize) -> Season {
        let ticks_per_season = self.year_period / 4;
        let season_idx = (tick % self.year_period) / ticks_per_season;
        match season_idx {
            0 => Season::Spring,
            1 => Season::Summer,
            2 => Season::Autumn,
            _ => Season::Winter,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClimateConfig {
    pub cold_latitude_threshold: isize,
    pub mild_thermoregulation_surcharge: usize,
    pub severe_thermoregulation_surcharge: usize,
    pub winter_replenish_per_mille: usize, // e.g. 500 = 50%, 250 = 25%
}

impl Default for ClimateConfig {
    fn default() -> Self {
        Self {
            cold_latitude_threshold: 4,
            mild_thermoregulation_surcharge: 1,
            severe_thermoregulation_surcharge: 2,
            winter_replenish_per_mille: 500,
        }
    }
}

impl ClimateConfig {
    /// Computes the regional temperature harshness / thermoregulation surcharge.
    /// In Winter:
    /// - |y| > cold_latitude_threshold: severe surcharge (+2 energy/tick)
    /// - |y| <= cold_latitude_threshold: mild surcharge (+1 energy/tick)
    /// In other seasons: 0 surcharge.
    pub fn thermoregulation_surcharge(&self, season: Season, y: isize) -> usize {
        if season != Season::Winter {
            return 0;
        }
        if y.abs() > self.cold_latitude_threshold {
            self.severe_thermoregulation_surcharge
        } else {
            self.mild_thermoregulation_surcharge
        }
    }

    /// Scales resource replenishment during Winter.
    pub fn scale_replenishment(&self, season: Season, base_replenish: usize) -> usize {
        if season != Season::Winter {
            base_replenish
        } else {
            (base_replenish * self.winter_replenish_per_mille) / 1000
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HibernationState {
    Active,
    Hibernating { ticks_hibernated: usize },
}

impl HibernationState {
    pub fn is_hibernating(&self) -> bool {
        matches!(self, HibernationState::Hibernating { .. })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct HibernationConfig {
    pub min_entry_energy: usize,
    pub min_entry_water: usize,
    pub critical_wake_energy: usize,
    pub critical_wake_water: usize,
    pub metabolic_interval: usize, // 1 drain every N ticks
}

impl Default for HibernationConfig {
    fn default() -> Self {
        Self {
            min_entry_energy: 50,
            min_entry_water: 30,
            critical_wake_energy: 15,
            critical_wake_water: 10,
            metabolic_interval: 4,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WakeReason {
    Attacked,
    CriticalReserves,
    SeasonEnded,
    ExitedBurrow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HibernationOutcome {
    ActiveNoChange,
    EnteredHibernation,
    Maintained {
        ticks_hibernated: usize,
        energy_drained: usize,
        water_drained: usize,
    },
    WokeUp(WakeReason),
}

#[derive(Debug, Clone, Default)]
pub struct SeasonalClimateEngine {
    pub cycle: Option<SeasonCycle>,
    pub climate_config: ClimateConfig,
    pub hibernation_config: HibernationConfig,
}

impl SeasonalClimateEngine {
    pub fn new(year_period: usize) -> Self {
        Self {
            cycle: Some(SeasonCycle::new(year_period)),
            climate_config: ClimateConfig::default(),
            hibernation_config: HibernationConfig::default(),
        }
    }

    pub fn current_season(&self, tick: usize) -> Season {
        self.cycle
            .as_ref()
            .map_or(Season::Spring, |c| c.season_at(tick))
    }

    /// Evaluates whether an individual is eligible to enter Hibernation.
    pub fn evaluate_hibernation(
        &self,
        in_burrow: bool,
        season: Season,
        energy: usize,
        water: usize,
    ) -> bool {
        in_burrow
            && season == Season::Winter
            && energy >= self.hibernation_config.min_entry_energy
            && water >= self.hibernation_config.min_entry_water
    }

    /// Attempts to enter hibernation.
    pub fn try_enter_hibernation(
        &self,
        state: &mut HibernationState,
        in_burrow: bool,
        season: Season,
        energy: usize,
        water: usize,
    ) -> bool {
        if self.evaluate_hibernation(in_burrow, season, energy, water) {
            *state = HibernationState::Hibernating {
                ticks_hibernated: 0,
            };
            true
        } else {
            false
        }
    }

    /// Resolves one tick of hibernation or active metabolism.
    /// During hibernation:
    /// - Checks wake conditions immediately (damage, critical reserves, season end, left burrow).
    /// - Thermoregulation surcharge is eliminated (0 energy penalty).
    /// - Basal metabolic drain is reduced to 1 drain every N ticks.
    pub fn resolve_hibernation_tick(
        &self,
        state: &mut HibernationState,
        season: Season,
        in_burrow: bool,
        energy: &mut usize,
        water: &mut usize,
        damage_taken: usize,
    ) -> HibernationOutcome {
        match state {
            HibernationState::Active => HibernationOutcome::ActiveNoChange,
            HibernationState::Hibernating { ticks_hibernated } => {
                // Wake condition 1: Attacked
                if damage_taken > 0 {
                    *state = HibernationState::Active;
                    return HibernationOutcome::WokeUp(WakeReason::Attacked);
                }

                // Wake condition 2: Season transitioned away from Winter
                if season != Season::Winter {
                    *state = HibernationState::Active;
                    return HibernationOutcome::WokeUp(WakeReason::SeasonEnded);
                }

                // Wake condition 3: Left Burrow
                if !in_burrow {
                    *state = HibernationState::Active;
                    return HibernationOutcome::WokeUp(WakeReason::ExitedBurrow);
                }

                // Wake condition 4: Critical energy/water threshold
                if *energy <= self.hibernation_config.critical_wake_energy
                    || *water <= self.hibernation_config.critical_wake_water
                {
                    *state = HibernationState::Active;
                    return HibernationOutcome::WokeUp(WakeReason::CriticalReserves);
                }

                // Conserved metabolism: drain 1 energy and 1 water every N ticks
                *ticks_hibernated += 1;
                let mut energy_drained = 0;
                let mut water_drained = 0;

                if *ticks_hibernated % self.hibernation_config.metabolic_interval == 0 {
                    if *energy > 0 {
                        *energy -= 1;
                        energy_drained = 1;
                    }
                    if *water > 0 {
                        *water -= 1;
                        water_drained = 1;
                    }
                }

                HibernationOutcome::Maintained {
                    ticks_hibernated: *ticks_hibernated,
                    energy_drained,
                    water_drained,
                }
            }
        }
    }
}
