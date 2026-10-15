//! A bounded contextual bandit for visible water sources, independent of world rules.
//! Integer values and the world's seeded RNG keep updates reproducible.
use super::{
    decision::{Decision, Reason},
    intent::Intent,
    lifestate::LifeState,
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
    pub values: Vec<Value>,
    #[serde(default)]
    pub pending: Option<Position>,
}
impl Default for Learner {
    fn default() -> Self {
        Self {
            enabled: true,
            values: vec![],
            pending: None,
        }
    }
}
impl Learner {
    pub fn choose(
        &mut self,
        origin: Position,
        view: &Perception,
        base: Decision,
        rng: &mut Random,
    ) -> Decision {
        self.pending = None;
        if !self.enabled || base.reason != Reason::SeekWater || view.waters.len() < 2 {
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
        self.pending = Some(positions[index]);
        Decision::new(
            Intent::Drink {
                at: view.waters[index].steps.clone(),
            },
            Reason::LearnedWater,
        )
    }
    pub fn finish(
        &mut self,
        id: usize,
        before: &LifeState,
        after: &LifeState,
        events: &[SimulationEvent],
    ) {
        let Some(pos) = self.pending.take() else {
            return;
        };
        let gained: usize = events
            .iter()
            .filter_map(|e| match e {
                SimulationEvent::EntityDrank {
                    id: actor,
                    pos: p,
                    amount,
                } if *actor == id && *p == pos => Some(*amount),
                _ => None,
            })
            .sum();
        let travel = before.energy.saturating_sub(after.energy).min(100) as i32;
        let injury = before.health.saturating_sub(after.health).min(100) as i32;
        let reward = (gained.min(100) as i32 * 10
            - travel * 2
            - injury * 5
            - if gained == 0 { 10 } else { 0 })
        .clamp(-1000, 1000);
        if let Some(v) = self.values.iter_mut().find(|v| v.pos == pos) {
            v.value += (reward - v.value) / 4;
            v.visits = v.visits.saturating_add(1);
        } else {
            if self.values.len() == 32 {
                self.values.remove(0);
            }
            self.values.push(Value {
                pos,
                value: reward,
                visits: 1,
            });
        }
    }
    pub fn validate(&self) -> crate::Result<()> {
        if self.values.len() > 32
            || self
                .values
                .iter()
                .any(|v| !(-1000..=1000).contains(&v.value))
        {
            return Err("learning bounds exceeded".into());
        }
        Ok(())
    }
}
