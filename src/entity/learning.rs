//! Bounded water-choice bandit with multi-tick outcome accounting.
//! Integer values and the world's seeded RNG keep updates reproducible.
use super::{
    decision::{Decision, Reason},
    intent::Intent,
    perception::Perception,
};
use crate::{map::movement::Position, random::Random, simulation::SimulationEvent};
use serde::{Deserialize, Serialize};
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
        }
    }
}
/// Maximum number of executed water-choice ticks in one attempt.
pub const MAX_WATER_TICKS: u8 = 20;
const MAX_ACCOUNT: usize = 1_000_000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaterEpisode {
    pub target: Position,
    pub ticks: u8,
    pub movement_energy: usize,
    pub incoming_damage: usize,
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
        if !self.enabled {
            self.interrupt(WaterEnd::Disabled);
            return base;
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
            return base;
        }
        let positions: Vec<_> = view
            .waters
            .iter()
            .map(|w| w.steps.0.iter().fold(origin, |p, d| p + *d))
            .collect();
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
                    (
                        q,
                        std::cmp::Reverse(view.waters[*i].steps.len()),
                        std::cmp::Reverse(*i),
                    )
                })
                .unwrap()
        };
        self.episode = Some(WaterEpisode {
            target: positions[index],
            ticks: 0,
            movement_energy: 0,
            incoming_damage: 0,
        });
        Decision::new(
            Intent::Drink {
                at: view.waters[index].steps.clone(),
            },
            Reason::LearnedWater,
        )
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
        for event in events {
            match event {
                SimulationEvent::EntityMoved {
                    id: actor,
                    energy_cost,
                    ..
                } if *actor == id => {
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
                    water = water.saturating_add(*amount).min(100);
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
        let attempt = self.episode.take().unwrap();
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
        self.last_episode = Some(WaterResult {
            attempt,
            water,
            end,
            reward: Some(reward),
        });
    }
    pub fn validate(&self) -> crate::Result<()> {
        let valid_pos =
            |p: Position| p.x.unsigned_abs() <= 1_000_000 && p.y.unsigned_abs() <= 1_000_000;
        let valid_attempt = |a: &WaterEpisode| {
            valid_pos(a.target)
                && a.ticks <= MAX_WATER_TICKS
                && a.movement_energy <= MAX_ACCOUNT
                && a.incoming_damage <= MAX_ACCOUNT
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
            || self.values.iter().any(|v| {
                !(-1000..=1000).contains(&v.value)
                    || v.pos.x.unsigned_abs() > 1_000_000
                    || v.pos.y.unsigned_abs() > 1_000_000
            })
        {
            return Err("learning bounds exceeded".into());
        }
        Ok(())
    }
}
