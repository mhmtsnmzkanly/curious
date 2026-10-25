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
        movement::{Direction, DIRECTION_ARRAY, Position, Steps},
    },
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
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

pub use super::refuge_generalization::{
    GeneralizedRefugeKey, GeneralizedRefugeOutcome, MAX_GENERALIZED_REFUGE_OUTCOMES,
};
pub use super::refuge_learning::{
    FirstIntervalRefugeRecord, MAX_REFUGE_EPISODES_PER_KEY, MAX_REFUGE_OUTCOMES,
    PendingRefugeClosure, RefugeClosure, RefugeEpisode, RefugeKey, RefugeOutcome,
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct KnownSector {
    pub sector_x: isize,
    pub sector_y: isize,
    pub last_visited_tick: usize,
    pub visit_count: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum RefugeLearningMode {
    #[default]
    LegacyExact,
    Generalized,
}

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
    #[serde(default, skip_serializing_if = "is_false")]
    pub refuge_generalization_enabled: bool,
    #[serde(default = "enabled_by_default", skip_serializing_if = "is_true")]
    pub refuge_generalization_update_enabled: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub refuge_generalized_outcomes: Vec<GeneralizedRefugeOutcome>,
    #[serde(default)]
    pub refuge_episode: Option<RefugeEpisode>,
    #[serde(default)]
    pub pending_refuge_closures: Vec<PendingRefugeClosure>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub spatial_curiosity_enabled: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub known_sectors: Vec<KnownSector>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub first_interval_refuge_enabled: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub first_interval_outcomes: Vec<super::refuge_learning::FirstIntervalRefugeRecord>,
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
            refuge_generalization_enabled: false,
            refuge_generalization_update_enabled: true,
            refuge_generalized_outcomes: vec![],
            refuge_episode: None,
            pending_refuge_closures: vec![],
            spatial_curiosity_enabled: false,
            known_sectors: vec![],
            first_interval_refuge_enabled: false,
            first_interval_outcomes: vec![],
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
            if let Some(episode) = self.refuge_episode.take()
                && episode.learning_mode == RefugeLearningMode::Generalized
            {
                self.pending_refuge_closures.push(PendingRefugeClosure {
                    final_pos: origin,
                    ended_tick: tick,
                    episode,
                    closure: RefugeClosure::FeatureDisabled,
                });
            }
            self.sites.clear();
            self.dangers.clear();
            self.companions.clear();
            self.social = Default::default();
            self.refuge_outcomes.clear();
            if self.refuge_generalization_update_enabled {
                self.refuge_generalized_outcomes.clear();
            }
            self.refuge_episode = None;
            self.pending_refuge_closures
                .retain(|closure| closure.episode.learning_mode == RefugeLearningMode::Generalized);
            self.known_sectors.clear();
            self.first_interval_outcomes.clear();
            return;
        }
        if self.spatial_curiosity_enabled {
            self.known_sectors
                .retain(|s| tick.saturating_sub(s.last_visited_tick) <= 120);

            let mut sectors_to_visit = Vec::new();
            sectors_to_visit.push((origin.x.div_euclid(4), origin.y.div_euclid(4)));
            for (pos, _) in &view.cells {
                let sec = (pos.x.div_euclid(4), pos.y.div_euclid(4));
                if !sectors_to_visit.contains(&sec) {
                    sectors_to_visit.push(sec);
                }
            }

            for (sx, sy) in sectors_to_visit {
                if let Some(s) = self
                    .known_sectors
                    .iter_mut()
                    .find(|s| s.sector_x == sx && s.sector_y == sy)
                {
                    s.last_visited_tick = tick;
                    s.visit_count = s.visit_count.saturating_add(1);
                } else {
                    if self.known_sectors.len() >= 16 {
                        if let Some((oldest_idx, _)) = self
                            .known_sectors
                            .iter()
                            .enumerate()
                            .min_by_key(|(_, s)| (s.last_visited_tick, s.sector_y, s.sector_x))
                        {
                            self.known_sectors.remove(oldest_idx);
                        }
                    }
                    self.known_sectors.push(KnownSector {
                        sector_x: sx,
                        sector_y: sy,
                        last_visited_tick: tick,
                        visit_count: 1,
                    });
                }
            }
        }
        self.refuge_outcomes
            .retain(|r| tick.saturating_sub(r.last_observed_tick) <= 100);
        if self.refuge_generalization_enabled && self.refuge_generalization_update_enabled {
            self.refuge_generalized_outcomes.retain(|row| {
                tick.saturating_sub(row.last_completed_tick)
                    <= super::refuge_generalization::GENERALIZED_REFUGE_TTL_TICKS
            });
        }
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
        self.choose_refuge_for_species(
            super::species::Species::Herbivore,
            origin,
            life,
            view,
            decision,
            tick,
        )
    }

    /// Species-aware refuge policy entry point used by world intent collection.
    pub fn choose_refuge_for_species(
        &mut self,
        subject_species: super::species::Species,
        origin: Position,
        life: &LifeState,
        view: &Perception,
        decision: Decision,
        tick: usize,
    ) -> Decision {
        super::refuge_learning::choose_refuge(
            self,
            subject_species,
            origin,
            life,
            view,
            decision,
            tick,
        )
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
        let direct_dir = DIRECTION_ARRAY
            .into_iter()
            .filter(|d| view.directions.get(d).copied().unwrap_or(0) > 0)
            .min_by_key(|d| {
                (
                    (origin + *d).distance_to(target),
                    DIRECTION_ARRAY.iter().position(|x| x == d).unwrap(),
                )
            });

        if let Some(dir) = direct_dir {
            if (origin + dir).distance_to(target) < origin.distance_to(target) {
                let count = life
                    .speed
                    .min(view.directions[&dir] as usize)
                    .min(life.stamina)
                    .min(life.energy / life.move_energy_cost.max(1))
                    .min(origin.distance_to(target));
                if count == 0 {
                    return None;
                }
                return Some(Decision::new(
                    Intent::Move {
                        steps: Steps(vec![dir; count]),
                    },
                    reason,
                ));
            }
        }

        if !life.can_move_for(1) || view.cells.is_empty() {
            return None;
        }

        let cell_map: HashMap<Position, &Cell> =
            view.cells.iter().map(|(pos, cell)| (*pos, cell)).collect();

        let occupied: HashSet<Position> = view
            .entities
            .iter()
            .map(|e| e.steps.iter().fold(origin, |p, d| p + *d))
            .collect();

        let is_walkable = |pos: Position| -> bool {
            if pos != origin && occupied.contains(&pos) {
                return false;
            }
            match cell_map.get(&pos) {
                Some(Cell::Wall) => false,
                Some(_) => true,
                None => pos == origin,
            }
        };

        struct BfsNode {
            pos: Position,
            steps: usize,
            first_dir: Direction,
        }

        let mut visited = HashSet::new();
        visited.insert(origin);
        let mut queue = VecDeque::new();

        for d in DIRECTION_ARRAY {
            if view.directions.get(&d).copied().unwrap_or(0) == 0 {
                continue;
            }
            let next = origin + d;
            if visited.contains(&next) || !is_walkable(next) {
                continue;
            }
            let dx = next.x - origin.x;
            let dy = next.y - origin.y;
            if dx != 0 && dy != 0 {
                if !is_walkable(Position::new(origin.x + dx, origin.y))
                    || !is_walkable(Position::new(origin.x, origin.y + dy))
                {
                    continue;
                }
            }
            visited.insert(next);
            queue.push_back(BfsNode {
                pos: next,
                steps: 1,
                first_dir: d,
            });
        }

        let mut best: Option<(usize, usize, isize, isize, Direction)> = None;

        while let Some(node) = queue.pop_front() {
            if let Some((best_steps, _, _, _, _)) = best {
                if node.steps > best_steps {
                    break;
                }
            }

            if node.pos.distance_to(target) < origin.distance_to(target) {
                let dist = node.pos.distance_to(target);
                let key = (node.steps, dist, node.pos.y, node.pos.x);
                if best
                    .as_ref()
                    .map(|b| (b.0, b.1, b.2, b.3) > key)
                    .unwrap_or(true)
                {
                    best = Some((node.steps, dist, node.pos.y, node.pos.x, node.first_dir));
                }
                continue;
            }

            for d in DIRECTION_ARRAY {
                let next = node.pos + d;
                if visited.contains(&next) || !is_walkable(next) {
                    continue;
                }
                let dx = next.x - node.pos.x;
                let dy = next.y - node.pos.y;
                if dx != 0 && dy != 0 {
                    if !is_walkable(Position::new(node.pos.x + dx, node.pos.y))
                        || !is_walkable(Position::new(node.pos.x, node.pos.y + dy))
                    {
                        continue;
                    }
                }
                visited.insert(next);
                queue.push_back(BfsNode {
                    pos: next,
                    steps: node.steps + 1,
                    first_dir: node.first_dir,
                });
            }
        }

        best.map(|(_, _, _, _, first_dir)| {
            Decision::new(
                Intent::Move {
                    steps: Steps(vec![first_dir]),
                },
                reason,
            )
        })
    }

    pub fn curiosity_direction(
        &self,
        origin: Position,
        view: &Perception,
        life: &LifeState,
        _tick: usize,
    ) -> Option<Direction> {
        if !self.spatial_curiosity_enabled {
            return None;
        }
        if life.energy <= life.max_energy / 2
            || life.water <= life.max_water / 2
            || life.stamina <= life.max_stamina / 3
        {
            return None;
        }
        if !self.dangers.is_empty()
            || view
                .entities
                .iter()
                .any(|e| e.species == super::species::Species::Carnivore)
        {
            return None;
        }
        let cur_sx = origin.x.div_euclid(4);
        let cur_sy = origin.y.div_euclid(4);

        DIRECTION_ARRAY
            .into_iter()
            .filter(|d| view.directions.get(d).copied().unwrap_or(0) > 0)
            .min_by_key(|d| {
                let d_pos = Position::new(0, 0) + *d;
                let target_sx = cur_sx + d_pos.x;
                let target_sy = cur_sy + d_pos.y;
                let (visited, last_visited, visits) = self
                    .known_sectors
                    .iter()
                    .find(|s| s.sector_x == target_sx && s.sector_y == target_sy)
                    .map(|s| (true, s.last_visited_tick, s.visit_count))
                    .unwrap_or((false, 0, 0));
                (
                    visited,
                    last_visited,
                    visits,
                    DIRECTION_ARRAY.iter().position(|x| x == d).unwrap(),
                )
            })
    }

    pub fn validate(&self) -> crate::Result<()> {
        self.social.validate()?;
        super::refuge_learning::validate(self)?;
        if self.sites.len() > 32
            || self.dangers.len() > 8
            || self.companions.len() > 8
            || self.known_sectors.len() > 16
            || self.first_interval_outcomes.len() > 16
            || self.first_interval_outcomes.iter().any(|r| r.episodes.len() > 8)
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
            || self.refuge_generalized_outcomes.len() > MAX_GENERALIZED_REFUGE_OUTCOMES
            || self.refuge_generalized_outcomes.iter().any(|row| {
                row.completed == 0
                    || row.completed > super::refuge_generalization::MAX_GENERALIZED_REFUGE_SAMPLES
                    || row.safe_arrivals > row.completed
                    || row.threat_damage
                        > super::refuge_generalization::MAX_GENERALIZED_REFUGE_ACCOUNT
                    || row.movement_energy
                        > super::refuge_generalization::MAX_GENERALIZED_REFUGE_ACCOUNT
            })
        {
            return Err("memory bounds exceeded".into());
        }
        Ok(())
    }
}

fn enabled_by_default() -> bool {
    true
}

fn is_false(value: &bool) -> bool {
    !value
}

fn is_true(value: &bool) -> bool {
    *value
}
