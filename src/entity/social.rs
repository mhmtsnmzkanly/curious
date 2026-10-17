//! Experimental bounded rendezvous, using only observations and remembered contacts.
use super::{
    decision::{Decision, Reason},
    intent::Intent,
    lifestate::LifeState,
    memory::KnownCompanion,
    perception::Perception,
    species::Species,
};
use crate::map::{
    cell::Cell,
    movement::{DIRECTION_ARRAY, Position, Steps},
};
use serde::{Deserialize, Serialize};
const LIMIT: usize = 30;
const RETRY: usize = 10;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rendezvous {
    pub id: usize,
    pub at: Position,
    pub seen: usize,
    pub started: usize,
    pub visited: Vec<Position>,
    #[serde(default)]
    pub searching: bool,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SocialMemory {
    pub target: Option<Rendezvous>,
    pub retry_after: usize,
}
impl SocialMemory {
    fn close(a: Position, b: Position) -> bool {
        a.x.abs_diff(b.x).max(a.y.abs_diff(b.y)) <= 2
    }
    fn safe_cell(
        origin: Position,
        p: Position,
        view: &Perception,
        dangers: &[(Position, usize)],
        tick: usize,
    ) -> bool {
        matches!(
            view.cells.iter().find(|(at, _)| *at == p).map(|(_, c)| c),
            Some(Cell::Burrow)
        ) || (!dangers
            .iter()
            .any(|(at, seen)| tick.saturating_sub(*seen) <= LIMIT && Self::close(*at, p))
            && !view.entities.iter().any(|e| {
                e.species.hunts(&Species::Herbivore)
                    && Self::close(e.steps.iter().fold(origin, |q, d| q + *d), p)
            }))
    }
    fn fallback(base: Decision) -> Decision {
        if matches!(base.reason, Reason::RecallMate | Reason::ApproachMate) {
            Decision::new(Intent::Idle { duration: 1 }, Reason::PauseMateSearch)
        } else {
            base
        }
    }
    pub fn validate(&self) -> crate::Result<()> {
        if self.target.as_ref().is_some_and(|g| {
            g.id == 0
                || g.visited.len() > 32
                || std::iter::once(&g.at)
                    .chain(g.visited.iter())
                    .any(|p| p.x.unsigned_abs() > 1_000_000 || p.y.unsigned_abs() > 1_000_000)
        }) {
            return Err("social memory bounds exceeded".into());
        }
        Ok(())
    }
    pub fn decide(
        &mut self,
        origin: Position,
        life: &LifeState,
        view: &Perception,
        contacts: &[KnownCompanion],
        dangers: &[(Position, usize)],
        tick: usize,
        base: Decision,
    ) -> Decision {
        if !life.is_alive() || !life.is_mature() {
            self.target = None;
            return base;
        }
        if self.target.as_ref().is_some_and(|g| {
            tick.saturating_sub(g.started) >= LIMIT || tick.saturating_sub(g.seen) > LIMIT
        }) {
            self.target = None;
            self.retry_after = tick.saturating_add(RETRY);
        }
        // Backoff limits stale searches, not a currently visible real mating
        // opportunity. It does not create/restart a search or bypass safety.
        if tick < self.retry_after
            && base.reason == Reason::ApproachMate
            && life.can_reproduce()
            && life.can_move_for(1)
        {
            if let Some(dir) =
                view.entities
                    .iter()
                    .filter(|e| {
                        e.species == Species::Herbivore
                            && e.mature
                            && e.reproductive
                            && e.steps.len() > 1
                    })
                    .filter_map(|e| e.steps.0.first().copied().map(|dir| (e, dir)))
                    .filter(|(_, dir)| {
                        view.directions.get(dir).copied().unwrap_or(0) > 0
                            && !view.entities.iter().any(|e| {
                                e.steps.iter().fold(origin, |p, d| p + *d) == origin + *dir
                            })
                            && Self::safe_cell(origin, origin + *dir, view, dangers, tick)
                    })
                    .min_by_key(|(e, _)| (e.steps.len(), e.id))
                    .map(|(_, dir)| dir)
            {
                return Decision::new(
                    Intent::Move {
                        steps: Steps(vec![dir]),
                    },
                    Reason::SeekCompanion,
                );
            }
        }
        // Never override physical mating, survival, threats, thirst, hunger,
        // recovery, player commands or a custom policy. Interruptions use time too.
        if !matches!(
            base.reason,
            Reason::Rest
                | Reason::Explore
                | Reason::NoEligibleMate
                | Reason::RecallMate
                | Reason::ApproachMate
        ) || tick < self.retry_after
            || life.energy <= life.low_energy_threshold()
            || life.stamina < 2
        {
            return Self::fallback(base);
        }
        let visible = |id| {
            view.entities
                .iter()
                .find(|e| e.id == id && e.species == Species::Herbivore && e.mature)
        };
        if self.target.is_none() {
            let contact = contacts
                .iter()
                .filter(|c| tick.saturating_sub(c.seen) <= LIMIT)
                .min_by_key(|c| (origin.distance_to(c.pos), c.id));
            let Some(c) = contact else {
                return Self::fallback(base);
            };
            // Do not start an expedition that exceeds remaining lifespan or reserves.
            let distance = origin.distance_to(c.pos);
            let budget = life.energy.saturating_sub(life.low_energy_threshold())
                / (life.move_energy_cost.saturating_add(1));
            if distance.saturating_add(1) >= life.max_age.saturating_sub(life.age)
                || distance > budget
            {
                return Self::fallback(base);
            }
            self.target = Some(Rendezvous {
                id: c.id,
                at: c.pos,
                seen: c.seen,
                started: tick,
                visited: vec![origin],
                searching: false,
            });
        }
        // An actually ready visible mate takes precedence over waiting for an
        // unavailable contact. Keep the ordinary mating opportunity, safely.
        if base.reason == Reason::ApproachMate {
            if let Some(mate) = view
                .entities
                .iter()
                .filter(|e| e.species == Species::Herbivore && e.reproductive && e.mature)
                .min_by_key(|e| (e.steps.len(), e.id))
            {
                if self.target.as_ref().is_some_and(|g| g.id != mate.id) {
                    self.target = Some(Rendezvous {
                        id: mate.id,
                        at: mate.steps.iter().fold(origin, |p, d| p + *d),
                        seen: tick,
                        started: tick,
                        visited: vec![origin],
                        searching: false,
                    });
                }
            }
        }
        let goal = self.target.as_mut().unwrap();
        if let Some(mate) = visible(goal.id) {
            goal.at = mate.steps.iter().fold(origin, |p, d| p + *d);
            goal.seen = tick;
            goal.searching = false;
            if mate.steps.len() <= 1 {
                return Decision::new(Intent::Idle { duration: 1 }, Reason::WaitForMate);
            }
        }
        let occupied: Vec<_> = view
            .entities
            .iter()
            .map(|e| e.steps.iter().fold(origin, |p, d| p + *d))
            .collect();
        let safe = |p| Self::safe_cell(origin, p, view, dangers, tick);
        // Follow a visible path one step at a time. Lost contacts get a bounded
        // local search near their last observed position; no current-world lookup.
        let direct = visible(goal.id).and_then(|e| e.steps.0.first().copied());
        if visible(goal.id).is_none() && origin.distance_to(goal.at) <= 1 {
            goal.searching = true;
        }
        if !goal.visited.contains(&origin) && goal.visited.len() < 32 {
            goal.visited.push(origin);
        }
        let near = goal.searching;
        let direction = direct
            .filter(|d| {
                view.directions.get(d).copied().unwrap_or(0) > 0
                    && !occupied.contains(&(origin + *d))
                    && safe(origin + *d)
            })
            .or_else(|| {
                DIRECTION_ARRAY
                    .into_iter()
                    .filter(|d| view.directions.get(d).copied().unwrap_or(0) > 0)
                    .filter(|d| !occupied.contains(&(origin + *d)) && safe(origin + *d))
                    .filter(|d| {
                        if near {
                            !goal.visited.contains(&(origin + *d))
                                && (origin + *d).distance_to(goal.at) <= life.vision_range.min(6)
                        } else {
                            (origin + *d).distance_to(goal.at) < origin.distance_to(goal.at)
                        }
                    })
                    .min_by_key(|d| {
                        (
                            (origin + *d).distance_to(goal.at),
                            DIRECTION_ARRAY.iter().position(|x| x == d).unwrap(),
                        )
                    })
            });
        if let Some(dir) = direction {
            if !life.can_move_for(1) {
                return Decision::new(Intent::Idle { duration: 1 }, Reason::PauseMateSearch);
            }
            Decision::new(
                Intent::Move {
                    steps: Steps(vec![dir]),
                },
                Reason::SeekCompanion,
            )
        } else {
            self.target = None;
            self.retry_after = tick.saturating_add(RETRY);
            // An unsafe old RecallMate/ApproachMate must not bypass this guard.
            Decision::new(Intent::Idle { duration: 1 }, Reason::PauseMateSearch)
        }
    }
}
