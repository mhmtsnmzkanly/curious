//! Pure, standalone, default-off per-owner First-Resolver-Interval Refuge Damage Learner prototype.
//!
//! This module is intentionally decoupled from `World`, live `State`, and persistent checkpoints.
//! It learns observable threat damage and movement energy specifically for the first resolver
//! interval of a refuge choice. Later full-route censoring (interruption, third-party damage
//! on tick 2+) does NOT censor the already-resolved, attributed first-interval outcome.

pub const MAX_FIRST_INTERVAL_ROWS: usize = 8;
pub const MAX_INTERVAL_SAMPLES: u64 = 1_000_000;
pub const MAX_INTERVAL_ACCOUNT: u64 = 1_000_000_000;
pub const FIRST_INTERVAL_TTL_TICKS: u64 = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[allow(dead_code)]
pub enum Species {
    Herbivore,
    Carnivore,
    Omnivore,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FirstIntervalKey {
    pub subject_species: Species,
    pub threat_species: Species,
    pub threat_bearing_sector: u8,
    pub threat_distance_steps: u8,
    pub first_step_sector: u8,
    pub endpoint: (i32, i32),
    pub subject_speed: u8,
}

impl FirstIntervalKey {
    pub fn validate(&self) -> bool {
        (1..=2).contains(&self.threat_distance_steps)
            && self.threat_bearing_sector < 8
            && self.first_step_sector < 8
            && self.subject_speed > 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FirstIntervalCensorReason {
    ThirdPartyDamage,
    UnattributedHealthLoss,
    PreResolutionInterrupt,
    MissingThreatAttribution,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FirstIntervalOutcome {
    CompletedFirstInterval {
        threat_damage: u64,
        paid_movement_energy: u64,
        later_full_route_censored: bool,
    },
    CensoredFirstInterval(FirstIntervalCensorReason),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordResult {
    Applied { updated_existing: bool },
    Censored(FirstIntervalCensorReason),
    Disabled,
    Frozen,
    WrongOwner,
    InvalidKey,
    InvalidOutcomeValue,
    AccountLimitExceeded,
    SampleLimitExceeded,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FirstIntervalRow {
    pub key: FirstIntervalKey,
    pub completed_count: u64,
    pub total_threat_damage: u64,
    pub total_movement_energy: u64,
    pub later_censored_count: u64,
    pub last_updated_tick: u64,
}

impl FirstIntervalRow {
    pub fn is_expired(&self, current_tick: u64) -> bool {
        current_tick.saturating_sub(self.last_updated_tick) > FIRST_INTERVAL_TTL_TICKS
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EstimateStatus {
    Known,
    Unknown,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FirstIntervalEstimate {
    pub candidate_index: usize,
    pub key: FirstIntervalKey,
    pub status: EstimateStatus,
    pub completed_samples: u64,
    pub total_threat_damage: u64,
    pub total_movement_energy: u64,
    pub later_censored_samples: u64,
    pub last_updated_tick: Option<u64>,
}

impl FirstIntervalEstimate {
    pub fn expected_threat_damage(&self) -> Option<f64> {
        if self.status == EstimateStatus::Known && self.completed_samples > 0 {
            Some(self.total_threat_damage as f64 / self.completed_samples as f64)
        } else {
            None
        }
    }

    pub fn expected_movement_energy(&self) -> Option<f64> {
        if self.status == EstimateStatus::Known && self.completed_samples > 0 {
            Some(self.total_movement_energy as f64 / self.completed_samples as f64)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FirstIntervalRanking {
    pub estimates: Vec<FirstIntervalEstimate>,
    pub ranked_indices: Option<Vec<usize>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FirstIntervalLearner {
    pub owner_id: usize,
    pub enabled: bool,
    pub update_enabled: bool,
    pub rows: Vec<FirstIntervalRow>,
    pub total_samples_recorded: u64,
    pub total_account_value: u64,
}

impl FirstIntervalLearner {
    pub fn new(owner_id: usize) -> Self {
        Self {
            owner_id,
            enabled: false,
            update_enabled: false,
            rows: Vec::new(),
            total_samples_recorded: 0,
            total_account_value: 0,
        }
    }

    pub fn with_enabled(owner_id: usize, enabled: bool, update_enabled: bool) -> Self {
        Self {
            owner_id,
            enabled,
            update_enabled,
            rows: Vec::new(),
            total_samples_recorded: 0,
            total_account_value: 0,
        }
    }

    pub fn record_episode(
        &mut self,
        owner_id: usize,
        key: FirstIntervalKey,
        outcome: FirstIntervalOutcome,
        current_tick: u64,
    ) -> RecordResult {
        if !self.enabled {
            return RecordResult::Disabled;
        }
        if !self.update_enabled {
            return RecordResult::Frozen;
        }
        if owner_id != self.owner_id {
            return RecordResult::WrongOwner;
        }
        if !key.validate() {
            return RecordResult::InvalidKey;
        }

        let (threat_damage, paid_movement_energy, later_full_route_censored) = match outcome {
            FirstIntervalOutcome::CompletedFirstInterval {
                threat_damage,
                paid_movement_energy,
                later_full_route_censored,
            } => (threat_damage, paid_movement_energy, later_full_route_censored),
            FirstIntervalOutcome::CensoredFirstInterval(reason) => {
                return RecordResult::Censored(reason);
            }
        };

        if threat_damage > MAX_INTERVAL_ACCOUNT || paid_movement_energy > MAX_INTERVAL_ACCOUNT {
            return RecordResult::InvalidOutcomeValue;
        }

        if self.total_samples_recorded >= MAX_INTERVAL_SAMPLES {
            return RecordResult::SampleLimitExceeded;
        }

        let episode_account = match threat_damage.checked_add(paid_movement_energy) {
            Some(acc) => acc,
            None => return RecordResult::AccountLimitExceeded,
        };
        let new_total_account = match self.total_account_value.checked_add(episode_account) {
            Some(tot) if tot <= MAX_INTERVAL_ACCOUNT => tot,
            _ => return RecordResult::AccountLimitExceeded,
        };

        if let Some(index) = self.rows.iter().position(|r| r.key == key) {
            let row = &self.rows[index];
            let new_completed = match row.completed_count.checked_add(1) {
                Some(c) if c <= MAX_INTERVAL_SAMPLES => c,
                _ => return RecordResult::SampleLimitExceeded,
            };
            let new_damage = match row.total_threat_damage.checked_add(threat_damage) {
                Some(d) if d <= MAX_INTERVAL_ACCOUNT => d,
                _ => return RecordResult::AccountLimitExceeded,
            };
            let new_energy = match row.total_movement_energy.checked_add(paid_movement_energy) {
                Some(e) if e <= MAX_INTERVAL_ACCOUNT => e,
                _ => return RecordResult::AccountLimitExceeded,
            };
            let new_later_censored = if later_full_route_censored {
                match row.later_censored_count.checked_add(1) {
                    Some(lc) => lc,
                    None => return RecordResult::SampleLimitExceeded,
                }
            } else {
                row.later_censored_count
            };

            let target_row = &mut self.rows[index];
            target_row.completed_count = new_completed;
            target_row.total_threat_damage = new_damage;
            target_row.total_movement_energy = new_energy;
            target_row.later_censored_count = new_later_censored;
            target_row.last_updated_tick = current_tick;

            self.total_samples_recorded += 1;
            self.total_account_value = new_total_account;

            RecordResult::Applied {
                updated_existing: true,
            }
        } else {
            while self.rows.len() >= MAX_FIRST_INTERVAL_ROWS {
                let evict_idx = self.choose_eviction_index(current_tick);
                self.rows.remove(evict_idx);
            }

            self.rows.push(FirstIntervalRow {
                key,
                completed_count: 1,
                total_threat_damage: threat_damage,
                total_movement_energy: paid_movement_energy,
                later_censored_count: if later_full_route_censored { 1 } else { 0 },
                last_updated_tick: current_tick,
            });

            self.total_samples_recorded += 1;
            self.total_account_value = new_total_account;

            RecordResult::Applied {
                updated_existing: false,
            }
        }
    }

    fn choose_eviction_index(&self, current_tick: u64) -> usize {
        if let Some((idx, _)) = self
            .rows
            .iter()
            .enumerate()
            .filter(|(_, r)| r.is_expired(current_tick))
            .min_by(|(_, a), (_, b)| a.last_updated_tick.cmp(&b.last_updated_tick))
        {
            return idx;
        }

        self.rows
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| a.last_updated_tick.cmp(&b.last_updated_tick))
            .map(|(idx, _)| idx)
            .unwrap_or(0)
    }

    pub fn rank_first_interval_candidates(
        &self,
        candidates: &[FirstIntervalKey],
        current_tick: u64,
    ) -> FirstIntervalRanking {
        if !self.enabled || candidates.is_empty() {
            let estimates = candidates
                .iter()
                .enumerate()
                .map(|(candidate_index, &key)| FirstIntervalEstimate {
                    candidate_index,
                    key,
                    status: EstimateStatus::Unknown,
                    completed_samples: 0,
                    total_threat_damage: 0,
                    total_movement_energy: 0,
                    later_censored_samples: 0,
                    last_updated_tick: None,
                })
                .collect();
            return FirstIntervalRanking {
                estimates,
                ranked_indices: None,
            };
        }

        let mut estimates = Vec::with_capacity(candidates.len());
        let mut all_known = true;

        for (candidate_index, &key) in candidates.iter().enumerate() {
            let row = self
                .rows
                .iter()
                .find(|r| r.key == key && !r.is_expired(current_tick));

            if let Some(r) = row {
                estimates.push(FirstIntervalEstimate {
                    candidate_index,
                    key,
                    status: EstimateStatus::Known,
                    completed_samples: r.completed_count,
                    total_threat_damage: r.total_threat_damage,
                    total_movement_energy: r.total_movement_energy,
                    later_censored_samples: r.later_censored_count,
                    last_updated_tick: Some(r.last_updated_tick),
                });
            } else {
                all_known = false;
                estimates.push(FirstIntervalEstimate {
                    candidate_index,
                    key,
                    status: EstimateStatus::Unknown,
                    completed_samples: 0,
                    total_threat_damage: 0,
                    total_movement_energy: 0,
                    later_censored_samples: 0,
                    last_updated_tick: None,
                });
            }
        }

        if !all_known {
            return FirstIntervalRanking {
                estimates,
                ranked_indices: None,
            };
        }

        let mut sorted_indices: Vec<usize> = (0..candidates.len()).collect();
        sorted_indices.sort_by(|&i, &j| {
            let est_a = &estimates[i];
            let est_b = &estimates[j];

            let dmg_a = (est_a.total_threat_damage as u128) * (est_b.completed_samples as u128);
            let dmg_b = (est_b.total_threat_damage as u128) * (est_a.completed_samples as u128);
            if dmg_a != dmg_b {
                return dmg_a.cmp(&dmg_b);
            }

            let nrg_a = (est_a.total_movement_energy as u128) * (est_b.completed_samples as u128);
            let nrg_b = (est_b.total_movement_energy as u128) * (est_a.completed_samples as u128);
            if nrg_a != nrg_b {
                return nrg_a.cmp(&nrg_b);
            }

            i.cmp(&j)
        });

        FirstIntervalRanking {
            estimates,
            ranked_indices: Some(sorted_indices),
        }
    }
}
