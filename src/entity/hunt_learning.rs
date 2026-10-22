//! Bounded prey-context learner. It sees perception and audited outcomes only.
//! Disabled is a strict no-op, including RNG and persisted learner state.
use super::{
    decision::{Decision, Reason},
    intent::Intent,
    lifestate::LifeState,
    perception::{PerceivedEntity, Perception},
    species::Species,
};
use crate::random::Random;
use serde::{Deserialize, Serialize};

pub const MAX_HUNT_TICKS: u8 = 20;
pub const MAX_HUNT_VALUES: usize = 108; // 3 energy * 3 stamina * 3 distance * 4 visible health
const MAX_ACCOUNT: usize = 1_000_000;

/// No identity, coordinates, age or hidden target physiology in the learned key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct HuntContext {
    pub energy: u8,
    pub stamina: u8,
    pub distance: u8,
    /// 0 unknown; 1 one visible hit; 2 two hits; 3 more than two hits.
    pub health: u8,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HuntValue {
    pub context: HuntContext,
    pub value: i32,
    pub visits: u32,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HuntEpisode {
    pub target: usize,
    pub context: HuntContext,
    pub ticks: u8,
    pub initial_energy: usize,
    pub initial_stamina: usize,
    pub movement_energy: usize,
    pub attack_energy: usize,
    /// Recorded context only: external injury has uncertain causal attribution.
    pub incoming_damage: usize,
    pub food: usize,
    pub own_kill: bool,
    pub ambiguous: bool,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HuntEnd {
    Fed,
    TargetLost,
    TimedOut,
    Interrupted,
    Died,
    ForeignKill,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HuntResult {
    pub attempt: HuntEpisode,
    pub end: HuntEnd,
    pub reward: Option<i32>,
    pub updated: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HuntLearner {
    pub enabled: bool,
    pub update_enabled: bool,
    pub values: Vec<HuntValue>,
    pub episode: Option<HuntEpisode>,
    pub last_episode: Option<HuntResult>,
    /// Transient audit: at most two endings in one world tick; drained into events.
    #[serde(skip)]
    pub pending_results: Vec<HuntResult>,
}
impl Default for HuntLearner {
    fn default() -> Self {
        Self {
            enabled: false,
            update_enabled: true,
            values: vec![],
            episode: None,
            last_episode: None,
            pending_results: Vec::new(),
        }
    }
}
#[derive(Debug, Clone)]
pub struct HuntMeal {
    pub corpse_id: usize,
    pub amount: usize,
}
/// One executed tick, filtered to this actor by the integration layer. All costs
/// must come from resolver audit fields, never body deltas or guessed profiles.
#[derive(Debug, Clone)]
pub struct HuntLedger {
    pub alive: bool,
    pub movement_energy: usize,
    pub attack_energy: usize,
    pub incoming_damage: usize,
    /// Incoming damage dealt by the current episode's visible prey.
    pub prey_defense_damage: usize,
    /// Incoming damage from any source other than the current prey.
    pub external_damage: usize,
    /// Killer of the pending target, not an arbitrary death in this tick.
    pub target_killed_by: Option<usize>,
    pub food: Vec<HuntMeal>,
    pub target_lost: bool,
    /// Missing attribution/cost evidence must not become a negative example.
    pub ambiguous: bool,
}
impl Default for HuntLedger {
    fn default() -> Self {
        Self {
            alive: true,
            movement_energy: 0,
            attack_energy: 0,
            incoming_damage: 0,
            prey_defense_damage: 0,
            external_damage: 0,
            target_killed_by: None,
            food: vec![],
            target_lost: false,
            ambiguous: false,
        }
    }
}
fn band(value: usize, maximum: usize) -> u8 {
    ((value.saturating_mul(3) / maximum.max(1)).min(2)) as u8
}
impl HuntContext {
    pub fn from_seen(life: &LifeState, prey: &PerceivedEntity, damage: usize) -> Self {
        Self {
            energy: band(life.energy, life.max_energy),
            stamina: band(life.stamina, life.max_stamina),
            distance: match prey.steps.len() {
                0..=1 => 0,
                2..=3 => 1,
                _ => 2,
            },
            health: match prey.health {
                None => 0,
                Some(h) if h <= damage => 1,
                Some(h) if h <= damage.saturating_mul(2) => 2,
                Some(_) => 3,
            },
        }
    }
    fn valid(self) -> bool {
        self.energy < 3 && self.stamina < 3 && self.distance < 3 && self.health < 4
    }
}
impl HuntLearner {
    pub fn choose(
        &mut self,
        species: Species,
        life: &LifeState,
        view: &Perception,
        base: Decision,
        rng: &mut Random,
    ) -> Decision {
        if !self.enabled {
            return base;
        }
        // A kill is not food. Permit only the existing food policy to collect the
        // identified corpse; learner never creates an Eat action or overrides needs.
        if let Some(e) = &self.episode {
            if e.own_kill
                && matches!(&base.intent, Intent::Eat{corpse_id:Some(id),..} if *id==e.target)
            {
                return base;
            }
        }
        if base.reason != Reason::HuntPrey {
            if matches!(base.reason, Reason::HuntLowEnergy | Reason::HuntTooCostly)
                && self.is_audited_budget_cutoff(species, life, view, base.reason)
            {
                self.close_policy_cutoff();
                return base;
            }
            self.interrupt(HuntEnd::Interrupted);
            return base;
        }
        let profile = species.combat_profile();
        let candidates: Vec<_> = view
            .entities
            .iter()
            .filter(|p| {
                let travel = p.steps.len().saturating_sub(1);
                let first = profile
                    .energy_cost
                    .saturating_add(travel.saturating_mul(life.move_energy_cost));
                let hits = p.health.unwrap_or(p.power).div_ceil(profile.damage.max(1));
                let estimate = travel
                    .saturating_mul(life.move_energy_cost)
                    .saturating_add(hits.saturating_mul(profile.energy_cost.saturating_add(1)));
                species.hunts(&p.species)
                    && life.energy >= first
                    && life.stamina > life.max_stamina / 4
                    && life.stamina >= profile.stamina_cost
                    && (travel == 0 || estimate <= life.energy)
            })
            .collect();
        // The episode is an accounting window, never a target lock. Re-rank
        // every HuntPrey tick, using the same Q/epsilon/rule-cost policy.
        let old_end = self.episode.as_ref().map(|e| {
            if view.entities.iter().any(|p| p.id == e.target) {
                HuntEnd::Interrupted
            } else {
                HuntEnd::TargetLost
            }
        });
        if candidates.is_empty() {
            if let Some(end) = old_end {
                self.interrupt(end);
            }
            return base;
        }
        let index = if candidates.len() > 1 && rng.range(0, 99) < 10 {
            rng.range(0, candidates.len() as isize - 1) as usize
        } else {
            (0..candidates.len())
                .max_by_key(|i| {
                    let c = HuntContext::from_seen(life, candidates[*i], profile.damage);
                    let q = self
                        .values
                        .iter()
                        .find(|v| v.context == c)
                        .map_or(0, |v| v.value);
                    // V2: equal learned scores retain the rule policy's
                    // estimated-cost ordering. Nearest-first here silently
                    // changed naive behavior before any experience existed.
                    let prey = candidates[*i];
                    let hits = prey
                        .health
                        .unwrap_or(prey.power)
                        .div_ceil(profile.damage.max(1));
                    let estimated_cost = prey
                        .steps
                        .len()
                        .saturating_sub(1)
                        .saturating_mul(life.move_energy_cost)
                        .saturating_add(hits.saturating_mul(profile.energy_cost + 1));
                    (
                        q,
                        std::cmp::Reverse(estimated_cost),
                        std::cmp::Reverse(prey.steps.len()),
                        std::cmp::Reverse(prey.id),
                    )
                })
                .unwrap()
        };
        let p = candidates[index];
        if self.episode.as_ref().is_some_and(|e| e.target == p.id) {
            // Same selected target: retain initial context, elapsed ticks and
            // every audited cost. Only a genuinely new target starts fresh.
            return Self::decision(p, life);
        }
        if let Some(end) = old_end {
            self.interrupt(end);
        }
        self.episode = Some(HuntEpisode {
            target: p.id,
            context: HuntContext::from_seen(life, p, profile.damage),
            ticks: 0,
            initial_energy: life.energy,
            initial_stamina: life.stamina,
            movement_energy: 0,
            attack_energy: 0,
            incoming_damage: 0,
            food: 0,
            own_kill: false,
            ambiguous: false,
        });
        Self::decision(p, life)
    }
    fn decision(prey: &PerceivedEntity, life: &LifeState) -> Decision {
        Decision::new(
            if prey.steps.len() <= 1 {
                Intent::Attack { target_id: prey.id }
            } else {
                Intent::Move {
                    steps: crate::map::movement::Steps(
                        prey.steps.0.iter().take(life.speed).copied().collect(),
                    ),
                }
            },
            Reason::LearnedHunt,
        )
    }
    fn is_audited_budget_cutoff(
        &self,
        species: Species,
        life: &LifeState,
        view: &Perception,
        reason: Reason,
    ) -> bool {
        let Some(episode) = &self.episode else {
            return false;
        };
        let Some(prey) = view.entities.iter().find(|p| p.id == episode.target) else {
            return false;
        };
        let profile = species.combat_profile();
        if !species.hunts(&prey.species)
            || life.stamina <= life.max_stamina / 4
            || life.stamina < profile.stamina_cost
        {
            return false;
        }
        let travel = prey.steps.len().saturating_sub(1);
        match reason {
            Reason::HuntLowEnergy => {
                life.energy
                    < profile
                        .energy_cost
                        .saturating_add(travel.saturating_mul(life.move_energy_cost))
            }
            Reason::HuntTooCostly => {
                if travel == 0 {
                    return false;
                }
                let hits = prey
                    .health
                    .unwrap_or(prey.power)
                    .div_ceil(profile.damage.max(1));
                let estimate = travel
                    .saturating_mul(life.move_energy_cost)
                    .saturating_add(hits.saturating_mul(profile.energy_cost.saturating_add(1)));
                estimate > life.energy
            }
            _ => false,
        }
    }
    fn close_policy_cutoff(&mut self) {
        let Some(attempt) = self.episode.take() else {
            return;
        };
        // The reason for stopping is own lack of budget. Only prior resolver
        // action costs can support its negative example; external injury,
        // own-kill feeding delay, or no audited pursuit cost stays censored.
        let has_audited_cost = attempt.movement_energy > 0 || attempt.attack_energy > 0;
        let reward =
            if attempt.ambiguous || attempt.own_kill || attempt.food > 0 || !has_audited_cost {
                None
            } else {
                Some(reward_for(&attempt))
            };
        // Keep the existing serialized ending variant for V13 compatibility;
        // the decision trace records HuntLowEnergy/HuntTooCostly as the trigger.
        self.complete(attempt, HuntEnd::Interrupted, reward);
    }
    pub fn interrupt(&mut self, end: HuntEnd) {
        if let Some(attempt) = self.episode.take() {
            self.record_result(HuntResult {
                attempt,
                end,
                reward: None,
                updated: false,
            });
        }
    }
    pub fn finish(&mut self, id: usize, ledger: &HuntLedger) {
        if !self.enabled {
            return;
        }
        let Some(e) = &mut self.episode else {
            return;
        };
        e.ticks = e.ticks.saturating_add(1);
        e.movement_energy = e
            .movement_energy
            .saturating_add(ledger.movement_energy)
            .min(MAX_ACCOUNT);
        e.attack_energy = e
            .attack_energy
            .saturating_add(ledger.attack_energy)
            .min(MAX_ACCOUNT);
        e.incoming_damage = e
            .incoming_damage
            .saturating_add(ledger.incoming_damage)
            .min(MAX_ACCOUNT);
        // Injury is observable context, but its causal cost is not established.
        let damage_attribution_matches = ledger
            .prey_defense_damage
            .checked_add(ledger.external_damage)
            == Some(ledger.incoming_damage);
        e.ambiguous |=
            ledger.ambiguous || !damage_attribution_matches || ledger.external_damage > 0;
        if let Some(killer) = ledger.target_killed_by {
            e.own_kill = killer == id;
        }
        for meal in &ledger.food {
            if meal.corpse_id == e.target {
                if e.own_kill {
                    e.food = e.food.saturating_add(meal.amount).min(MAX_ACCOUNT);
                } else {
                    e.ambiguous = true;
                }
            }
        }
        let foreign = ledger.target_killed_by.is_some_and(|killer| killer != id);
        let end = if !ledger.alive {
            Some(HuntEnd::Died)
        } else if foreign {
            Some(HuntEnd::ForeignKill)
        } else if e.food > 0 {
            Some(HuntEnd::Fed)
        } else if ledger.target_lost && !e.own_kill {
            Some(HuntEnd::TargetLost)
        } else if e.ticks >= MAX_HUNT_TICKS {
            Some(HuntEnd::TimedOut)
        } else {
            None
        };
        let Some(end) = end else {
            return;
        };
        let attempt = self.episode.take().unwrap();
        // Target loss/death/foreign kills are not identifiable causal outcomes.
        let reward = if attempt.ambiguous
            || matches!(
                end,
                HuntEnd::Died | HuntEnd::ForeignKill | HuntEnd::TargetLost | HuntEnd::Interrupted
            ) {
            None
        } else {
            Some(reward_for(&attempt))
        };
        self.complete(attempt, end, reward);
    }
    fn complete(&mut self, attempt: HuntEpisode, end: HuntEnd, reward: Option<i32>) {
        let updated = self.update_enabled && reward.is_some();
        if let Some(q) = reward.filter(|_| self.update_enabled) {
            if let Some(v) = self
                .values
                .iter_mut()
                .find(|v| v.context == attempt.context)
            {
                v.value += (q - v.value) / 4;
                v.visits = v.visits.saturating_add(1);
            } else {
                // There are only 108 unique valid contexts. Defensive bound also
                // prevents malformed, not-yet-validated callers growing state.
                if self.values.len() >= MAX_HUNT_VALUES {
                    self.values.remove(0);
                }
                self.values.push(HuntValue {
                    context: attempt.context,
                    value: q,
                    visits: 1,
                });
            }
        }
        self.record_result(HuntResult {
            attempt,
            end,
            reward,
            updated,
        });
    }
    fn record_result(&mut self, result: HuntResult) {
        if self.pending_results.len() >= 2 {
            self.pending_results.remove(0);
        }
        self.pending_results.push(result.clone());
        self.last_episode = Some(result);
    }
    pub fn take_results(&mut self) -> Vec<HuntResult> {
        std::mem::take(&mut self.pending_results)
    }
    pub fn validate(&self) -> crate::Result<()> {
        let valid_episode = |e: &HuntEpisode| {
            e.context.valid()
                && e.ticks <= MAX_HUNT_TICKS
                && e.initial_energy <= MAX_ACCOUNT
                && e.initial_stamina <= 1000
                && [
                    e.movement_energy,
                    e.attack_energy,
                    e.incoming_damage,
                    e.food,
                ]
                .iter()
                .all(|n| *n <= MAX_ACCOUNT)
        };
        if self.pending_results.len() > 2
            || self.values.len() > MAX_HUNT_VALUES
            || self.values.iter().enumerate().any(|(i, v)| {
                !v.context.valid()
                    || !(-1000..=1000).contains(&v.value)
                    || v.visits == 0
                    || self.values[..i]
                        .iter()
                        .any(|other| other.context == v.context)
            })
            || self
                .episode
                .as_ref()
                .is_some_and(|e| !valid_episode(e) || e.ticks >= MAX_HUNT_TICKS)
            || self.last_episode.as_ref().is_some_and(|r| {
                !valid_episode(&r.attempt)
                    || r.reward.is_some_and(|v| !(-1000..=1000).contains(&v))
                    || (r.updated && r.reward.is_none())
            })
        {
            return Err("hunt learning bounds exceeded".into());
        }
        Ok(())
    }
}
fn reward_for(attempt: &HuntEpisode) -> i32 {
    (attempt.food as i64 * 10
        - attempt.movement_energy as i64 * 2
        - attempt.attack_energy as i64 * 2
        - attempt.ticks as i64)
        .clamp(-1000, 1000) as i32
}
