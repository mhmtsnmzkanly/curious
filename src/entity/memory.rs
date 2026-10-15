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
pub struct Memory {
    pub enabled: bool,
    pub sites: Vec<KnownSite>,
    pub dangers: Vec<(Position, usize)>,
}
impl Default for Memory {
    fn default() -> Self {
        Self {
            enabled: true,
            sites: vec![],
            dangers: vec![],
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
            return;
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
        for e in &view.entities {
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
        let Some(dir) = DIRECTION_ARRAY
            .into_iter()
            .filter(|d| view.directions.get(d).copied().unwrap_or(0) > 0)
            .min_by_key(|d| {
                (
                    (origin + *d).distance_to(site.pos),
                    DIRECTION_ARRAY.iter().position(|x| x == d).unwrap(),
                )
            })
        else {
            return decision;
        };
        if (origin + dir).distance_to(site.pos) >= origin.distance_to(site.pos) {
            return decision;
        }
        let count = life
            .speed
            .min(view.directions[&dir] as usize)
            .min(life.stamina)
            .min(life.energy)
            .min(origin.distance_to(site.pos))
            .max(1);
        Decision::new(
            Intent::Move {
                steps: Steps(vec![dir; count]),
            },
            reason,
        )
    }
    pub fn validate(&self) -> crate::Result<()> {
        if self.sites.len() > 32
            || self.dangers.len() > 8
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
