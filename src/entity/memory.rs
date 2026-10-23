//! Bounded individual knowledge; populated exclusively from local observations.
use super::{
    decision::{Decision, Reason},
    intent::Intent,
    lifestate::LifeState,
    perception::Perception,
};
use crate::{
    environment::ResourceKind,
    map::{
        cell::Cell,
        movement::{DIRECTION_ARRAY, Position, Steps},
    },
};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnownSite {
    pub pos: Position,
    pub kind: ResourceKind,
    pub seen: usize,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnownCompanion {
    pub id: usize,
    pub pos: Position,
    pub seen: usize,
}

pub use super::refuge_learning::{
    MAX_REFUGE_EPISODES_PER_KEY, MAX_REFUGE_OUTCOMES, PendingRefugeClosure, RefugeClosure,
    RefugeEpisode, RefugeKey, RefugeOutcome,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    #[serde(default)]
    pub social: super::social::SocialMemory,
    pub enabled: bool,
    #[serde(default = "enabled_by_default")]
    pub companions_enabled: bool,
    pub sites: Vec<KnownSite>,
    pub dangers: Vec<(Position, usize)>,
    #[serde(default)]
    pub companions: Vec<KnownCompanion>,
    #[serde(default)]
    pub refuge_outcomes_enabled: bool,
    #[serde(default)]
    pub refuge_outcomes: Vec<RefugeOutcome>,
    #[serde(default)]
    pub refuge_episode: Option<RefugeEpisode>,
    #[serde(default)]
    pub pending_refuge_closures: Vec<PendingRefugeClosure>,
}
impl Default for Memory {
    fn default() -> Self {
        Self {
            social: Default::default(),
            enabled: true,
            companions_enabled: true,
            sites: vec![],
            dangers: vec![],
            companions: vec![],
            refuge_outcomes_enabled: false,
            refuge_outcomes: vec![],
            refuge_episode: None,
            pending_refuge_closures: vec![],
        }
    }
}
impl Memory {
    pub fn observe(
        &mut self,
        origin: Position,
        view: &Perception,
        tick: usize,
        species: super::species::Species,
    ) {
        if !self.enabled {
            self.sites.clear();
            self.dangers.clear();
            self.companions.clear();
            self.social = Default::default();
            self.refuge_outcomes.clear();
            self.refuge_episode = None;
            self.pending_refuge_closures.clear();
            return;
        }
        self.refuge_outcomes
            .retain(|r| tick.saturating_sub(r.last_observed_tick) <= 100);
        if let Some(episode) = &mut self.refuge_episode {
            let threat_visible = view
                .entities
                .iter()
                .any(|e| e.id == episode.threat_id && e.species == episode.threat_species);
            episode.unseen_ticks = if threat_visible {
                0
            } else {
                episode.unseen_ticks.saturating_add(1)
            };
        }
        self.sites.retain(|s|tick.saturating_sub(s.seen)<=100 && !view.cells.iter().any(|(p,c)|*p==s.pos && !matches!((s.kind,c),(ResourceKind::Food,Cell::Food{amount})|(ResourceKind::Water,Cell::Water{amount}) if *amount>0)));
        for (pos, cell) in &view.cells {
            let kind = match cell {
                Cell::Food { amount } if *amount > 0 && species.eats_plants() => ResourceKind::Food,
                Cell::Water { amount } if *amount > 0 => ResourceKind::Water,
                _ => continue,
            };
            if let Some(s) = self.sites.iter_mut().find(|s| s.pos == *pos) {
                s.kind = kind;
                s.seen = tick;
            } else {
                self.sites.push(KnownSite {
                    pos: *pos,
                    kind,
                    seen: tick,
                });
            }
        }
        self.sites.sort_by_key(|s| (s.seen, s.pos.y, s.pos.x));
        if self.sites.len() > 32 {
            self.sites.drain(..self.sites.len() - 32);
        }
        self.dangers
            .retain(|(_, seen)| tick.saturating_sub(*seen) <= 30);
        self.companions.retain(|c| {
            self.companions_enabled
                && tick.saturating_sub(c.seen) <= 30
                && (!view.cells.iter().any(|(p, _)| *p == c.pos)
                    || view.entities.iter().any(|e| e.id == c.id))
        });
        for e in &view.entities {
            if self.companions_enabled && e.species == species && e.mature {
                let pos = e.steps.0.iter().fold(origin, |p, d| p + *d);
                if let Some(c) = self.companions.iter_mut().find(|c| c.id == e.id) {
                    c.pos = pos;
                    c.seen = tick;
                } else {
                    self.companions.push(KnownCompanion {
                        id: e.id,
                        pos,
                        seen: tick,
                    });
                }
            }
            if e.species.hunts(&species) {
                let p = e.steps.0.iter().fold(origin, |p, d| p + *d);
                if let Some(old) = self.dangers.iter_mut().find(|(pos, _)| *pos == p) {
                    old.1 = tick;
                } else {
                    self.dangers.push((p, tick));
                }
            }
        }
        if self.dangers.len() > 8 {
            self.dangers.drain(..self.dangers.len() - 8);
        }
        self.companions.sort_by_key(|c| (c.seen, c.id));
        if self.companions.len() > 8 {
            self.companions.drain(..self.companions.len() - 8);
        }
    }

    /// Apply the opt-in, resolver-outcome refuge ranking without changing baseline decisions.
    pub fn choose_refuge(
        &mut self,
        origin: Position,
        life: &LifeState,
        view: &Perception,
        decision: Decision,
        tick: usize,
    ) -> Decision {
        super::refuge_learning::choose_refuge(self, origin, life, view, decision, tick)
    }

    /// Close a route at a bounded experiment boundary as a censored observation.
    pub fn censor_refuge_at_boundary(&mut self, origin: Position, health: usize, tick: usize) {
        super::refuge_learning::censor_at_boundary(self, origin, health, tick);
    }

    pub fn recall(
        &self,
        origin: Position,
        life: &LifeState,
        view: &Perception,
        decision: Decision,
    ) -> Decision {
        if !self.enabled || life.energy == 0 || life.stamina == 0 {
            return decision;
        }
        if self.companions_enabled
            && decision.reason == Reason::NoEligibleMate
            && life.can_reproduce()
        {
            if let Some(c) = self
                .companions
                .iter()
                .min_by_key(|c| (origin.distance_to(c.pos), c.id))
            {
                return Self::toward(origin, c.pos, life, view, Reason::RecallMate)
                    .unwrap_or(decision);
            }
            return decision;
        }
        let (kind, reason) = match decision.reason {
            Reason::SearchWater => (ResourceKind::Water, Reason::RecallWater),
            Reason::SearchFood => (ResourceKind::Food, Reason::RecallFood),
            _ => return decision,
        };
        let Some(site) = self
            .sites
            .iter()
            .filter(|s| s.kind == kind)
            .min_by_key(|s| {
                (
                    self.dangers
                        .iter()
                        .filter(|(p, _)| p.distance_to(s.pos) <= 2)
                        .count(),
                    origin.distance_to(s.pos),
                    s.pos.y,
                    s.pos.x,
                )
            })
        else {
            return decision;
        };
        Self::toward(origin, site.pos, life, view, reason).unwrap_or(decision)
    }
    fn toward(
        origin: Position,
        target: Position,
        life: &LifeState,
        view: &Perception,
        reason: Reason,
    ) -> Option<Decision> {
        let Some(dir) = DIRECTION_ARRAY
            .into_iter()
            .filter(|d| view.directions.get(d).copied().unwrap_or(0) > 0)
            .min_by_key(|d| {
                (
                    (origin + *d).distance_to(target),
                    DIRECTION_ARRAY.iter().position(|x| x == d).unwrap(),
                )
            })
        else {
            return None;
        };
        if (origin + dir).distance_to(target) >= origin.distance_to(target) {
            return None;
        }
        let count = life
            .speed
            .min(view.directions[&dir] as usize)
            .min(life.stamina)
            .min(life.energy / life.move_energy_cost.max(1))
            .min(origin.distance_to(target));
        if count == 0 {
            return None;
        }
        Some(Decision::new(
            Intent::Move {
                steps: Steps(vec![dir; count]),
            },
            reason,
        ))
    }
    pub fn validate(&self) -> crate::Result<()> {
        self.social.validate()?;
        super::refuge_learning::validate(self)?;
        if self.sites.len() > 32
            || self.dangers.len() > 8
            || self.companions.len() > 8
            || self
                .companions
                .iter()
                .any(|c| c.pos.x.unsigned_abs() > 1_000_000 || c.pos.y.unsigned_abs() > 1_000_000)
            || self
                .sites
                .iter()
                .any(|s| s.pos.x.unsigned_abs() > 1_000_000 || s.pos.y.unsigned_abs() > 1_000_000)
            || self
                .dangers
                .iter()
                .any(|(p, _)| p.x.unsigned_abs() > 1_000_000 || p.y.unsigned_abs() > 1_000_000)
        {
            return Err("memory bounds exceeded".into());
        }
        Ok(())
    }
}

fn enabled_by_default() -> bool {
    true
}
