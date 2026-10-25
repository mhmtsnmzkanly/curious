//! Bounded water-choice bandit with multi-tick outcome accounting.
//! Integer values and the world's seeded RNG keep updates reproducible.
use super::{
    decision::{Decision, Reason},
    intent::Intent,
    perception::Perception,
};
use crate::{map::movement::Position, random::Random, simulation::SimulationEvent};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

pub type WaterLearner = Learner;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContextWaterEntry {
    pub pos: Position,
    pub speed: u8,
    pub move_energy_cost: u16,
    pub last_seen_tick: usize,
    pub observations: VecDeque<i32>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Value {
    pub pos: Position,
    pub value: i32,
    pub visits: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Learner {
    pub enabled: bool,
    #[serde(default = "updates_enabled")]
    pub update_enabled: bool,
    pub values: Vec<Value>,
    #[serde(default)]
    pub episode: Option<WaterEpisode>,
    #[serde(default)]
    pub last_episode: Option<WaterResult>,
    /// Exact bounded correction sum/count for resolver-confirmed outcomes.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub residuals: Vec<ResidualValue>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub context_ttl_enabled: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub context_entries: Vec<ContextWaterEntry>,
}
fn updates_enabled() -> bool {
    true
}
impl Default for Learner {
    fn default() -> Self {
        Self {
            enabled: true,
            update_enabled: true,
            values: vec![],
            episode: None,
            last_episode: None,
            residuals: vec![],
            context_ttl_enabled: false,
            context_entries: vec![],
        }
    }
}
/// Maximum number of executed water-choice ticks in one attempt.
pub const MAX_WATER_TICKS: u8 = 20;
const MAX_ACCOUNT: usize = 1_000_000;
/// Refuse further samples at this count so sum arithmetic stays exact and bounded.
pub const MAX_RESIDUAL_SAMPLES: u32 = 1_000_000;
/// `finish` accounts at most 100 water and 1,000,000 movement energy, with at
/// least one tick: actual score is within [-100,000,100, 19,900]. In foraging,
/// travel / ceil(travel / speed) <= speed <= 32, move cost <= 32, and water
/// benefit <= 20, so the baseline magnitude is below 104,000 independently of
/// path length. 101M bounds the residual difference without tuning a parameter.
const MAX_RESIDUAL: i64 = 101_000_000;
/// Valid body bounds are speed/move cost <=32 and water benefit <=20. The
/// travel/ticks ratio is at most speed, so current baseline magnitude stays
/// below 104k independent of route length.
const MAX_CHOICE_BASELINE: i64 = 104_000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResidualValue {
    pub pos: Position,
    /// Sum of exact integer score residuals; mean is `sum / visits`.
    pub sum: i64,
    pub visits: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WaterEpisode {
    pub target: Position,
    pub ticks: u8,
    pub movement_energy: usize,
    pub incoming_damage: usize,
    /// Current foraging score captured when this episode's source was selected.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub choice_baseline: Option<i64>,
    /// Candidate evidence is censored if the legacy bounded ledger would clip.
    #[serde(default, skip_serializing_if = "is_false")]
    pub outcome_overflowed: bool,
    /// Own drink evidence for another source makes the episode unattributable.
    #[serde(default, skip_serializing_if = "is_false")]
    pub candidate_censored: bool,
}

fn is_false(value: &bool) -> bool {
    !value
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WaterEnd {
    Reached,
    Failed,
    TimedOut,
    Interrupted,
    Disabled,
    Died,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WaterResult {
    pub attempt: WaterEpisode,
    pub water: usize,
    pub end: WaterEnd,
    pub reward: Option<i32>,
}

impl Learner {
    pub fn choose(
        &mut self,
        origin: Position,
        view: &Perception,
        base: Decision,
        rng: &mut Random,
    ) -> Decision {
        self.choose_inner(origin, None, view, base, rng, false, 0, 1)
    }

    /// Choose with the default-off source-residual policy enabled by world config.
    pub fn choose_with_residual(
        &mut self,
        origin: Position,
        life: &crate::entity::lifestate::LifeState,
        view: &Perception,
        base: Decision,
        rng: &mut Random,
        residual_enabled: bool,
    ) -> Decision {
        self.choose_with_residual_and_context(
            origin,
            life,
            view,
            base,
            rng,
            residual_enabled,
            0,
            1,
        )
    }

    pub fn choose_with_residual_and_context(
        &mut self,
        origin: Position,
        life: &crate::entity::lifestate::LifeState,
        view: &Perception,
        base: Decision,
        rng: &mut Random,
        residual_enabled: bool,
        tick: usize,
        move_cost: u16,
    ) -> Decision {
        self.choose_inner(
            origin,
            Some(life),
            view,
            base,
            rng,
            residual_enabled,
            tick,
            move_cost,
        )
    }

    fn choose_inner(
        &mut self,
        origin: Position,
        life: Option<&crate::entity::lifestate::LifeState>,
        view: &Perception,
        base: Decision,
        rng: &mut Random,
        residual_enabled: bool,
        tick: usize,
        move_cost: u16,
    ) -> Decision {
        if !self.enabled {
            self.interrupt(WaterEnd::Disabled);
            return base;
        }
        if self.context_ttl_enabled {
            self.prune_expired_context(tick);
        }
        if base.reason != Reason::SeekWater {
            self.interrupt(WaterEnd::Interrupted);
            return base;
        }
        // Continue only while perception still supplies a route. Never query the world
        // for a remembered resource, and never override a higher-priority decision.
        if let Some(attempt) = &self.episode {
            if let Some(water) = view
                .waters
                .iter()
                .find(|water| water.steps.0.iter().fold(origin, |p, d| p + *d) == attempt.target)
            {
                return Decision::new(
                    Intent::Drink {
                        at: water.steps.clone(),
                    },
                    Reason::LearnedWater,
                );
            }
            self.interrupt(WaterEnd::Interrupted);
        }
        if view.waters.len() < 2 {
            if self.context_ttl_enabled && view.waters.len() == 1 {
                let target = view.waters[0].steps.0.iter().fold(origin, |p, d| p + *d);
                let choice_baseline = life.map(|l| {
                    crate::entity::foraging::value(
                        view.waters[0].amount,
                        view.waters[0].steps.len(),
                        l,
                        true,
                    ) as i64
                });
                self.episode = Some(WaterEpisode {
                    target,
                    ticks: 0,
                    movement_energy: 0,
                    incoming_damage: 0,
                    choice_baseline,
                    outcome_overflowed: false,
                    candidate_censored: false,
                });
            }
            return base;
        }
        let positions: Vec<_> = view
            .waters
            .iter()
            .map(|w| w.steps.0.iter().fold(origin, |p, d| p + *d))
            .collect();
        let speed = life.map(|l| l.speed.max(1) as u8).unwrap_or(1);
        let index = if rng.range(0, 99) < 10 {
            rng.range(0, positions.len() as isize - 1) as usize
        } else {
            (0..positions.len())
                .max_by_key(|i| {
                    let q = self
                        .values
                        .iter()
                        .find(|v| v.pos == positions[*i])
                        .map(|v| v.value)
                        .unwrap_or(200);
                    let residual = if residual_enabled {
                        self.residuals
                            .iter()
                            .find(|v| v.pos == positions[*i] && v.visits > 0)
                            .map(|v| v.sum / v.visits as i64)
                            .unwrap_or(0)
                    } else {
                        0
                    };
                    let baseline = if residual_enabled || self.context_ttl_enabled {
                        life.map(|l| {
                            crate::entity::foraging::value(
                                view.waters[*i].amount,
                                view.waters[*i].steps.len(),
                                l,
                                true,
                            ) as i64
                        })
                        .unwrap_or(0)
                    } else {
                        0
                    };
                    let score = if self.context_ttl_enabled {
                        let ctx =
                            self.predict_context_residual(positions[*i], speed, move_cost, tick);
                        if let Some(ctx_res) = ctx {
                            baseline.saturating_add(ctx_res as i64)
                        } else if residual_enabled {
                            baseline.saturating_add(residual)
                        } else {
                            baseline
                        }
                    } else if residual_enabled {
                        baseline.saturating_add(residual)
                    } else {
                        q as i64
                    };
                    (
                        score,
                        std::cmp::Reverse(view.waters[*i].steps.len()),
                        std::cmp::Reverse(*i),
                    )
                })
                .unwrap()
        };
        let choice_baseline = (residual_enabled || self.context_ttl_enabled)
            .then(|| {
                life.map(|l| {
                    crate::entity::foraging::value(
                        view.waters[index].amount,
                        view.waters[index].steps.len(),
                        l,
                        true,
                    ) as i64
                })
            })
            .flatten();
        self.episode = Some(WaterEpisode {
            target: positions[index],
            ticks: 0,
            movement_energy: 0,
            incoming_damage: 0,
            choice_baseline,
            outcome_overflowed: false,
            candidate_censored: false,
        });
        Decision::new(
            Intent::Drink {
                at: view.waters[index].steps.clone(),
            },
            Reason::LearnedWater,
        )
    }

    /// Computes the context residual in the exact same unit/scale as `foraging::value`
    /// (`(2 * water - movement_energy - ticks) * 100 / ticks - choice_baseline`).
    /// Returns `None` if `choice_baseline` is absent or the episode is non-qualifying.
    pub fn episode_physical_residual(result: &WaterResult) -> Option<i32> {
        if !matches!(result.end, WaterEnd::Reached | WaterEnd::Failed)
            || result.attempt.outcome_overflowed
            || result.attempt.candidate_censored
        {
            return None;
        }
        let baseline = result.attempt.choice_baseline?;
        let actual = (result.water as i64)
            .checked_mul(2)
            .and_then(|v| v.checked_sub(result.attempt.movement_energy as i64))
            .and_then(|v| v.checked_sub(result.attempt.ticks as i64))
            .and_then(|v| v.checked_mul(100))
            .map(|v| v / result.attempt.ticks.max(1) as i64)?;
        let residual = actual.checked_sub(baseline)?;
        Some(residual.clamp(-1000, 1000) as i32)
    }
    /// A forced decision does not constitute evidence that a water source is bad.
    pub fn interrupt(&mut self, end: WaterEnd) {
        if let Some(attempt) = self.episode.take() {
            self.last_episode = Some(WaterResult {
                attempt,
                water: 0,
                end,
                reward: None,
            });
        }
    }

    pub fn finish(&mut self, id: usize, alive: bool, events: &[SimulationEvent]) {
        self.finish_with_evidence(id, alive, events, false);
    }

    /// Finish with world-verified intent evidence for missing-resource failures.
    pub fn finish_with_evidence(
        &mut self,
        id: usize,
        alive: bool,
        events: &[SimulationEvent],
        missing_resource_target_matches: bool,
    ) {
        if !self.enabled {
            self.interrupt(WaterEnd::Disabled);
            return;
        }
        let Some(attempt) = &mut self.episode else {
            return;
        };
        attempt.ticks = attempt.ticks.saturating_add(1);
        let mut water = 0usize;
        let mut failed = false;
        let mut mismatched_drink = false;
        for event in events {
            match event {
                SimulationEvent::EntityMoved {
                    id: actor,
                    energy_cost,
                    ..
                } if *actor == id => {
                    if attempt.choice_baseline.is_some()
                        && attempt.movement_energy.saturating_add(*energy_cost) > MAX_ACCOUNT
                    {
                        attempt.outcome_overflowed = true;
                    }
                    attempt.movement_energy = attempt
                        .movement_energy
                        .saturating_add(*energy_cost)
                        .min(MAX_ACCOUNT);
                }
                SimulationEvent::EntityAttacked { target, damage, .. } if *target == id => {
                    // Recorded as context, not charged as a causal cost of water choice.
                    attempt.incoming_damage = attempt
                        .incoming_damage
                        .saturating_add(*damage)
                        .min(MAX_ACCOUNT);
                }
                SimulationEvent::EntityDrank {
                    id: actor,
                    pos,
                    amount,
                } if *actor == id && *pos == attempt.target => {
                    if attempt.choice_baseline.is_some() && water.saturating_add(*amount) > 100 {
                        attempt.outcome_overflowed = true;
                    }
                    water = water.saturating_add(*amount).min(100);
                }
                SimulationEvent::EntityDrank { id: actor, .. } if *actor == id => {
                    mismatched_drink = true;
                    if attempt.choice_baseline.is_some() {
                        attempt.candidate_censored = true;
                    }
                }
                SimulationEvent::ActionRejected {
                    id: actor,
                    action,
                    reason,
                    ..
                } if *actor == id
                    && action == "Drink"
                    && matches!(
                        reason,
                        crate::simulation::event::RejectionReason::MissingResource
                    ) =>
                {
                    failed = true;
                }
                _ => {}
            }
        }
        if !alive {
            self.interrupt(WaterEnd::Died);
            if let Some(result) = &mut self.last_episode {
                result.water = water;
            }
            return;
        }
        let end = if water > 0 {
            WaterEnd::Reached
        } else if failed {
            WaterEnd::Failed
        } else if attempt.ticks >= MAX_WATER_TICKS {
            WaterEnd::TimedOut
        } else {
            return;
        };
        let mut attempt = self.episode.take().unwrap();
        // Explicit resolver cost only: metabolism, healing and reproduction are not
        // subtracted again through a net energy delta. Time is a separate cost.
        let reward = (water as i64 * 10
            - attempt.movement_energy as i64 * 2
            - attempt.ticks as i64
            - if water == 0 { 10 } else { 0 })
        .clamp(-1000, 1000) as i32;
        if self.update_enabled {
            if let Some(v) = self.values.iter_mut().find(|v| v.pos == attempt.target) {
                v.value += (reward - v.value) / 4;
                v.visits = v.visits.saturating_add(1);
            } else {
                if self.values.len() == 32 {
                    self.values.remove(0);
                }
                self.values.push(Value {
                    pos: attempt.target,
                    value: reward,
                    visits: 1,
                });
            }
        }
        if self.update_enabled
            && !mismatched_drink
            && !attempt.outcome_overflowed
            && !attempt.candidate_censored
            && (end == WaterEnd::Reached
                || (end == WaterEnd::Failed && missing_resource_target_matches))
        {
            if let Some(baseline) = attempt.choice_baseline {
                let actual = (water as i64 * 2)
                    .checked_sub(attempt.movement_energy as i64)
                    .and_then(|v| v.checked_sub(attempt.ticks as i64))
                    .and_then(|v| v.checked_mul(100))
                    .map(|v| v / attempt.ticks.max(1) as i64);
                if let Some(residual) = actual.and_then(|v| v.checked_sub(baseline)) {
                    if !self.record_residual(attempt.target, residual) {
                        attempt.candidate_censored = true;
                    }
                } else {
                    attempt.candidate_censored = true;
                }
            }
        }
        self.last_episode = Some(WaterResult {
            attempt,
            water,
            end,
            reward: Some(reward),
        });
    }
    fn record_residual(&mut self, pos: Position, residual: i64) -> bool {
        if residual.unsigned_abs() > MAX_RESIDUAL as u64 {
            return false;
        }
        if let Some(value) = self.residuals.iter_mut().find(|v| v.pos == pos) {
            if value.visits >= MAX_RESIDUAL_SAMPLES {
                return false;
            }
            let Some(sum) = value.sum.checked_add(residual) else {
                return false;
            };
            value.sum = sum;
            value.visits += 1;
        } else {
            while self.residuals.len() >= 32 {
                self.residuals.remove(0);
            }
            self.residuals.push(ResidualValue {
                pos,
                sum: residual,
                visits: 1,
            });
        }
        true
    }
    pub fn prune_expired_context(&mut self, current_tick: usize) {
        self.context_entries
            .retain(|e| current_tick.saturating_sub(e.last_seen_tick) <= 48);
    }

    pub fn record_context_observation(
        &mut self,
        pos: Position,
        speed: u8,
        move_cost: u16,
        tick: usize,
        net_water_delta: i32,
    ) {
        if !self.context_ttl_enabled {
            return;
        }
        let net_water_delta = net_water_delta.clamp(-1000, 1000);
        self.prune_expired_context(tick);
        if let Some(entry) = self
            .context_entries
            .iter_mut()
            .find(|e| e.pos == pos && e.speed == speed && e.move_energy_cost == move_cost)
        {
            entry.last_seen_tick = tick;
            if entry.observations.len() >= 4 {
                entry.observations.pop_front();
            }
            entry.observations.push_back(net_water_delta);
        } else {
            if self.context_entries.len() >= 16 {
                if let Some((oldest_idx, _)) = self
                    .context_entries
                    .iter()
                    .enumerate()
                    .min_by_key(|(_, e)| (e.last_seen_tick, e.pos.y, e.pos.x))
                {
                    self.context_entries.remove(oldest_idx);
                }
            }
            let mut observations = VecDeque::with_capacity(4);
            observations.push_back(net_water_delta);
            self.context_entries.push(ContextWaterEntry {
                pos,
                speed,
                move_energy_cost: move_cost,
                last_seen_tick: tick,
                observations,
            });
        }
    }

    pub fn predict_context_residual(
        &self,
        pos: Position,
        current_speed: u8,
        current_move_cost: u16,
        current_tick: usize,
    ) -> Option<i32> {
        if !self.context_ttl_enabled {
            return None;
        }
        let entry = self.context_entries.iter().find(|e| {
            e.pos == pos
                && e.speed == current_speed
                && e.move_energy_cost == current_move_cost
                && current_tick.saturating_sub(e.last_seen_tick) <= 48
        })?;
        if entry.observations.is_empty() {
            return None;
        }
        let sum: i64 = entry.observations.iter().map(|&v| v as i64).sum();
        Some((sum / entry.observations.len() as i64) as i32)
    }

    pub fn validate(&self) -> crate::Result<()> {
        let valid_pos =
            |p: Position| p.x.unsigned_abs() <= 1_000_000 && p.y.unsigned_abs() <= 1_000_000;
        let valid_attempt = |a: &WaterEpisode| {
            valid_pos(a.target)
                && a.ticks <= MAX_WATER_TICKS
                && a.movement_energy <= MAX_ACCOUNT
                && a.incoming_damage <= MAX_ACCOUNT
                && a.choice_baseline
                    .is_none_or(|b| b.unsigned_abs() <= MAX_CHOICE_BASELINE as u64)
        };
        if self
            .episode
            .as_ref()
            .is_some_and(|a| !valid_attempt(a) || a.ticks >= MAX_WATER_TICKS)
            || self.last_episode.as_ref().is_some_and(|r| {
                !valid_attempt(&r.attempt)
                    || r.water > 100
                    || r.reward.is_some_and(|q| !(-1000..=1000).contains(&q))
            })
            || self.values.len() > 32
            || self.residuals.len() > 32
            || self.residuals.iter().any(|v| {
                v.visits == 0
                    || v.visits > MAX_RESIDUAL_SAMPLES
                    || v.sum.unsigned_abs() > (MAX_RESIDUAL as u64).saturating_mul(v.visits as u64)
                    || !valid_pos(v.pos)
            })
            || self
                .residuals
                .iter()
                .enumerate()
                .any(|(i, v)| self.residuals[..i].iter().any(|old| old.pos == v.pos))
            || self.values.iter().any(|v| {
                !(-1000..=1000).contains(&v.value)
                    || v.pos.x.unsigned_abs() > 1_000_000
                    || v.pos.y.unsigned_abs() > 1_000_000
            })
            || self.context_entries.len() > 16
            || self.context_entries.iter().any(|e| {
                !valid_pos(e.pos)
                    || e.speed == 0
                    || e.move_energy_cost == 0
                    || e.observations.len() > 4
                    || e.observations.iter().any(|&v| !(-1000..=1000).contains(&v))
            })
        {
            return Err("learning bounds exceeded".into());
        }
        Ok(())
    }
}
