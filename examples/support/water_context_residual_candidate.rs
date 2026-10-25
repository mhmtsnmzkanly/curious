//! Pure, default-off context-aligned and recency-windowed water residual candidate.
//!
//! Addressed to TODO.md P0 item 2 & ROADMAP.md K3 İz A:
//! Bounded utility water residual with explicit body-locomotion context alignment
//! and finite recency window + TTL expiration to adapt to dry scheduled water sources.
//!
//! This module is standalone and does NOT mutate World, Map, or State.

use std::{
    cmp::Reverse,
    collections::{BTreeMap, VecDeque},
};

pub const MAX_WINDOW_SAMPLES: usize = 8;
pub const MAX_CONTEXT_ROWS: usize = 32;
pub const WATER_RESIDUAL_TTL_TICKS: usize = 100;

pub const MAX_COORD: i32 = 1_000_000;
pub const MIN_SPEED: u8 = 1;
pub const MAX_SPEED: u8 = 32;
pub const MIN_MOVE_ENERGY_COST: u8 = 1;
pub const MAX_MOVE_ENERGY_COST: u8 = 32;

pub const MAX_EPISODE_TICKS: u8 = 30;
pub const MAX_WATER_AMOUNT: u32 = 100;
pub const MAX_MOVEMENT_ENERGY: u32 = 1_000_000;
pub const MAX_CHOICE_BASELINE: i64 = 104_000;
pub const MAX_RESIDUAL: i64 = 101_000_000;

/// Exact context-aligned key isolating geographical location and locomotion capability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct WaterContextKey {
    pub source_pos: (i32, i32),
    pub speed: u8,
    pub move_energy_cost: u8,
}

impl WaterContextKey {
    pub fn new(source_pos: (i32, i32), speed: u8, move_energy_cost: u8) -> Option<Self> {
        let key = Self {
            source_pos,
            speed,
            move_energy_cost,
        };
        if key.is_valid() {
            Some(key)
        } else {
            None
        }
    }

    pub fn is_valid(&self) -> bool {
        self.source_pos.0.abs() <= MAX_COORD
            && self.source_pos.1.abs() <= MAX_COORD
            && self.speed >= MIN_SPEED
            && self.speed <= MAX_SPEED
            && self.move_energy_cost >= MIN_MOVE_ENERGY_COST
            && self.move_energy_cost <= MAX_MOVE_ENERGY_COST
    }
}

/// Bounded recency window storing at most MAX_WINDOW_SAMPLES exact residuals.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WaterContextRow {
    pub window: VecDeque<i64>,
    pub last_observed_tick: usize,
    pub total_visits: u32,
}

impl WaterContextRow {
    pub fn new(first_residual: i64, tick: usize) -> Self {
        let mut window = VecDeque::with_capacity(MAX_WINDOW_SAMPLES);
        window.push_back(first_residual);
        Self {
            window,
            last_observed_tick: tick,
            total_visits: 1,
        }
    }

    pub fn push_residual(&mut self, residual: i64, tick: usize) {
        self.last_observed_tick = tick;
        self.total_visits = self.total_visits.saturating_add(1);
        if self.window.len() >= MAX_WINDOW_SAMPLES {
            self.window.pop_front();
        }
        self.window.push_back(residual);
    }

    pub fn window_mean(&self) -> Option<i64> {
        if self.window.is_empty() {
            None
        } else {
            let sum: i64 = self.window.iter().copied().sum();
            Some(sum / self.window.len() as i64)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaterCensorReason {
    Interrupted,
    TimedOut,
    Died,
    MismatchedDrink,
    Overflowed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaterAttemptOutcome {
    Reached {
        water_gained: u32,
        movement_energy: u32,
        ticks: u8,
        choice_baseline: i64,
    },
    FailedMissingResource {
        movement_energy: u32,
        ticks: u8,
        choice_baseline: i64,
        target_verified: bool,
    },
    Censored(WaterCensorReason),
}

/// Compute exact actual score using the core learning integer formula:
/// ((water * 2 - movement_energy - ticks) * 100) / ticks.max(1)
pub fn compute_actual_score(water: u32, movement_energy: u32, ticks: u8) -> i64 {
    let t = (ticks.max(1)) as i64;
    let benefit = (water as i64).saturating_mul(2);
    let cost = (movement_energy as i64).saturating_add(ticks as i64);
    benefit.saturating_sub(cost).saturating_mul(100) / t
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResidualStatus {
    Known { window_mean: i64, samples: usize },
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WaterCandidateInput {
    pub source_pos: (i32, i32),
    pub choice_baseline: i64,
    pub steps_len: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScoredWaterCandidate {
    pub index: usize,
    pub source_pos: (i32, i32),
    pub choice_baseline: i64,
    pub window_mean: i64,
    pub total_score: i64,
    pub status: ResidualStatus,
    pub steps_len: usize,
}

/// Standalone bounded context table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WaterContextResidualTable {
    pub enabled: bool,
    pub update_enabled: bool,
    pub rows: BTreeMap<WaterContextKey, WaterContextRow>,
}

impl Default for WaterContextResidualTable {
    fn default() -> Self {
        Self {
            enabled: false,
            update_enabled: true,
            rows: BTreeMap::new(),
        }
    }
}

impl WaterContextResidualTable {
    pub fn new(enabled: bool, update_enabled: bool) -> Self {
        Self {
            enabled,
            update_enabled,
            rows: BTreeMap::new(),
        }
    }

    /// Prune rows that have exceeded the TTL expiration relative to current_tick.
    pub fn prune_stale(&mut self, current_tick: usize) {
        self.rows.retain(|_, row| {
            current_tick.saturating_sub(row.last_observed_tick) <= WATER_RESIDUAL_TTL_TICKS
        });
    }

    /// Record a verified episode outcome.
    /// Returns true if the outcome was admitted and updated the table, false otherwise.
    pub fn record_outcome(
        &mut self,
        key: WaterContextKey,
        outcome: WaterAttemptOutcome,
        current_tick: usize,
    ) -> bool {
        if !self.enabled || !self.update_enabled {
            return false;
        }
        if !key.is_valid() {
            return false;
        }

        let (water, movement_energy, ticks, choice_baseline) = match outcome {
            WaterAttemptOutcome::Reached {
                water_gained,
                movement_energy,
                ticks,
                choice_baseline,
            } => {
                if water_gained > MAX_WATER_AMOUNT
                    || movement_energy > MAX_MOVEMENT_ENERGY
                    || ticks == 0
                    || ticks > MAX_EPISODE_TICKS
                    || choice_baseline.unsigned_abs() > MAX_CHOICE_BASELINE as u64
                {
                    return false;
                }
                (water_gained, movement_energy, ticks, choice_baseline)
            }
            WaterAttemptOutcome::FailedMissingResource {
                movement_energy,
                ticks,
                choice_baseline,
                target_verified,
            } => {
                if !target_verified {
                    // Unverified target failure is treated as censored!
                    return false;
                }
                if movement_energy > MAX_MOVEMENT_ENERGY
                    || ticks == 0
                    || ticks > MAX_EPISODE_TICKS
                    || choice_baseline.unsigned_abs() > MAX_CHOICE_BASELINE as u64
                {
                    return false;
                }
                (0, movement_energy, ticks, choice_baseline)
            }
            WaterAttemptOutcome::Censored(_) => return false,
        };

        let actual = compute_actual_score(water, movement_energy, ticks);
        let residual = actual.saturating_sub(choice_baseline);
        if residual.unsigned_abs() > MAX_RESIDUAL as u64 {
            return false;
        }

        // Prune expired rows first
        self.prune_stale(current_tick);

        if let Some(row) = self.rows.get_mut(&key) {
            row.push_residual(residual, current_tick);
        } else {
            if self.rows.len() >= MAX_CONTEXT_ROWS {
                // Evict oldest (last_observed_tick, key)
                if let Some((&oldest_key, _)) = self
                    .rows
                    .iter()
                    .min_by_key(|(k, row)| (row.last_observed_tick, **k))
                {
                    self.rows.remove(&oldest_key);
                }
            }
            self.rows.insert(key, WaterContextRow::new(residual, current_tick));
        }
        true
    }

    /// Query residual for a given key and tick.
    pub fn query_residual(
        &self,
        key: &WaterContextKey,
        current_tick: usize,
    ) -> ResidualStatus {
        if !self.enabled || !key.is_valid() {
            return ResidualStatus::Unknown;
        }
        let Some(row) = self.rows.get(key) else {
            return ResidualStatus::Unknown;
        };
        if current_tick.saturating_sub(row.last_observed_tick) > WATER_RESIDUAL_TTL_TICKS {
            return ResidualStatus::Unknown;
        }
        match row.window_mean() {
            Some(window_mean) => ResidualStatus::Known {
                window_mean,
                samples: row.window.len(),
            },
            None => ResidualStatus::Unknown,
        }
    }

    /// Score candidates by combining choice_baseline with fresh window_mean.
    pub fn score_candidates(
        &self,
        candidates: &[WaterCandidateInput],
        current_speed: u8,
        current_move_energy_cost: u8,
        current_tick: usize,
    ) -> Vec<ScoredWaterCandidate> {
        candidates
            .iter()
            .enumerate()
            .map(|(index, candidate)| {
                let key = WaterContextKey::new(
                    candidate.source_pos,
                    current_speed,
                    current_move_energy_cost,
                );
                let status = match key {
                    Some(ref k) => self.query_residual(k, current_tick),
                    None => ResidualStatus::Unknown,
                };
                let window_mean = match status {
                    ResidualStatus::Known { window_mean, .. } => window_mean,
                    ResidualStatus::Unknown => 0,
                };
                let total_score = candidate.choice_baseline.saturating_add(window_mean);
                ScoredWaterCandidate {
                    index,
                    source_pos: candidate.source_pos,
                    choice_baseline: candidate.choice_baseline,
                    window_mean,
                    total_score,
                    status,
                    steps_len: candidate.steps_len,
                }
            })
            .collect()
    }

    /// Choose the best candidate index ranked by:
    /// (choice_baseline.saturating_add(window_mean), Reverse(steps_len), Reverse(index))
    pub fn choose_water_candidate(
        &self,
        candidates: &[WaterCandidateInput],
        current_speed: u8,
        current_move_energy_cost: u8,
        current_tick: usize,
    ) -> Option<usize> {
        if candidates.is_empty() {
            return None;
        }
        let scores = self.score_candidates(
            candidates,
            current_speed,
            current_move_energy_cost,
            current_tick,
        );
        scores
            .iter()
            .max_by_key(|score| {
                (
                    score.total_score,
                    Reverse(score.steps_len),
                    Reverse(score.index),
                )
            })
            .map(|score| score.index)
    }
}

/// Standalone convenience wrapper for choose_water_candidate.
pub fn choose_water_candidate(
    table: &WaterContextResidualTable,
    candidates: &[WaterCandidateInput],
    current_speed: u8,
    current_move_energy_cost: u8,
    current_tick: usize,
) -> Option<usize> {
    table.choose_water_candidate(candidates, current_speed, current_move_energy_cost, current_tick)
}
