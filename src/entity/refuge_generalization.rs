//! Exact represented-context keys for opt-in refuge outcome reuse.
//!
//! These keys describe only the selected observable representation. Equality
//! is not a claim that two physical situations are causally equivalent.

use super::{lifestate::LifeState, perception::Perception, species::Species};
use crate::map::{
    cell::Cell,
    movement::{Direction, Position, Steps},
};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

pub const MAX_GENERALIZED_REFUGE_OUTCOMES: usize = 8;
pub const MAX_GENERALIZED_REFUGE_SAMPLES: u32 = 1_000_000;
pub const GENERALIZED_REFUGE_TTL_TICKS: usize = 100;
pub const MAX_GENERALIZED_REFUGE_ACCOUNT: u64 = 1_000_000_000_000;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ObservedCellKind {
    Empty,
    Wall,
    Burrow,
    Food,
    Water,
}

impl ObservedCellKind {
    fn from_cell(cell: &Cell) -> Self {
        match cell {
            Cell::Empty => Self::Empty,
            Cell::Wall => Self::Wall,
            Cell::Burrow => Self::Burrow,
            Cell::Food { .. } => Self::Food,
            Cell::Water { .. } => Self::Water,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ObservedStep {
    pub direction_sector: u8,
    pub cell_kind: ObservedCellKind,
}

/// Current observable subject/threat context and the identity of one visible
/// candidate route. IDs and the current alternative set are intentionally
/// excluded. Raw numeric values are exact-match features; this is not empirical
/// calibration of transfer safety.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct GeneralizedRefugeKey {
    pub subject_species: Species,
    pub threat_species: Species,
    pub threat_health: usize,
    pub threat_power: usize,
    /// Eight compass sectors, retained as a discrete representation.
    pub threat_bearing_sector: u8,
    /// Exact number of observed movement steps to the threat.
    pub threat_distance_steps: usize,
    pub subject_health: usize,
    pub subject_max_health: usize,
    pub subject_energy: usize,
    pub subject_max_energy: usize,
    pub subject_stamina: usize,
    pub subject_max_stamina: usize,
    pub subject_speed: usize,
    pub move_energy_cost: usize,
    pub origin_habitat: ObservedCellKind,
    /// Relative endpoint sector and the complete observed route in order.
    pub endpoint_sector: u8,
    pub route: Vec<ObservedStep>,
    pub refuge_kind: ObservedCellKind,
    pub turns: usize,
    /// Estimated current movement energy, distinct from resolver-paid energy.
    pub estimated_movement_energy: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GeneralizedRefugeOutcome {
    pub key: GeneralizedRefugeKey,
    pub last_completed_tick: usize,
    pub completed: u32,
    pub safe_arrivals: u32,
    pub threat_damage: u64,
    pub movement_energy: u64,
}

impl GeneralizedRefugeOutcome {
    /// Add one fully attributed, resolver-verified arrival. All measurements
    /// freeze together at their shared sample/account bound.
    pub fn record_completed(
        &mut self,
        tick: usize,
        threat_damage: u64,
        movement_energy: u64,
    ) -> bool {
        if tick < self.last_completed_tick || self.completed >= MAX_GENERALIZED_REFUGE_SAMPLES {
            return false;
        }
        let Some(next_damage) = self.threat_damage.checked_add(threat_damage) else {
            return false;
        };
        let Some(next_movement) = self.movement_energy.checked_add(movement_energy) else {
            return false;
        };
        if next_damage > MAX_GENERALIZED_REFUGE_ACCOUNT
            || next_movement > MAX_GENERALIZED_REFUGE_ACCOUNT
        {
            return false;
        }
        self.completed += 1;
        self.safe_arrivals += u32::from(threat_damage == 0);
        self.threat_damage = next_damage;
        self.movement_energy = next_movement;
        self.last_completed_tick = tick;
        true
    }
}

/// Record a complete attributed arrival, applying bounded deterministic
/// retention. Censored closures must not call this function.
pub fn record_completed_outcome(
    records: &mut Vec<GeneralizedRefugeOutcome>,
    key: GeneralizedRefugeKey,
    tick: usize,
    threat_damage: u64,
    movement_energy: u64,
) -> bool {
    if threat_damage > MAX_GENERALIZED_REFUGE_ACCOUNT
        || movement_energy > MAX_GENERALIZED_REFUGE_ACCOUNT
    {
        return false;
    }
    if let Some(index) = records.iter().position(|row| row.key == key) {
        if records[index].record_completed(tick, threat_damage, movement_energy) {
            return true;
        }
        return false;
    }
    if records.len() >= MAX_GENERALIZED_REFUGE_OUTCOMES {
        let Some(oldest) = records
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                (a.last_completed_tick, &a.key).cmp(&(b.last_completed_tick, &b.key))
            })
            .map(|(index, _)| index)
        else {
            return false;
        };
        records.remove(oldest);
    }
    let mut row = GeneralizedRefugeOutcome {
        key,
        last_completed_tick: tick,
        completed: 0,
        safe_arrivals: 0,
        threat_damage: 0,
        movement_energy: 0,
    };
    if !row.record_completed(tick, threat_damage, movement_energy) {
        return false;
    }
    records.push(row);
    true
}

/// Return a complete ranking only when every candidate has unique, fresh
/// support. `None` requests the caller's existing fallback for the whole
/// choice. One sample is a mechanism-level minimum, not empirical qualification.
pub fn rank_candidates(
    candidates: &[GeneralizedRefugeKey],
    records: &[GeneralizedRefugeOutcome],
    tick: usize,
) -> Option<Vec<usize>> {
    if candidates.is_empty() {
        return None;
    }
    let mut ranked = Vec::with_capacity(candidates.len());
    let mut used = std::collections::HashSet::new();
    for (candidate_index, candidate) in candidates.iter().enumerate() {
        let mut matches = records.iter().enumerate().filter(|(_, row)| {
            row.key == *candidate
                && row.completed > 0
                && row.last_completed_tick <= tick
                && tick.saturating_sub(row.last_completed_tick) <= GENERALIZED_REFUGE_TTL_TICKS
        });
        let (record_index, row) = matches.next()?;
        if matches.next().is_some() || !used.insert(record_index) {
            return None;
        }
        ranked.push((candidate_index, row));
    }
    ranked.sort_by(|(a_index, a), (b_index, b)| compare_outcome(a, b).then(a_index.cmp(b_index)));
    Some(ranked.into_iter().map(|(index, _)| index).collect())
}

fn compare_outcome(a: &GeneralizedRefugeOutcome, b: &GeneralizedRefugeOutcome) -> Ordering {
    let safety = (b.safe_arrivals as u128 * a.completed as u128)
        .cmp(&(a.safe_arrivals as u128 * b.completed as u128));
    if safety != Ordering::Equal {
        return safety;
    }
    let damage = (a.threat_damage as u128 * b.completed as u128)
        .cmp(&(b.threat_damage as u128 * a.completed as u128));
    if damage != Ordering::Equal {
        return damage;
    }
    (a.movement_energy as u128 * b.completed as u128)
        .cmp(&(b.movement_energy as u128 * a.completed as u128))
}

/// Construct one candidate key solely from current body state and perception.
/// Any absent required local cell/path feature fails closed with `None`.
pub fn candidate_key(
    origin: Position,
    subject_species: Species,
    life: &LifeState,
    view: &Perception,
    threat_species: Species,
    threat_health: usize,
    threat_power: usize,
    threat_steps: &Steps,
    route_steps: &Steps,
) -> Option<GeneralizedRefugeKey> {
    if threat_steps.is_empty() || route_steps.is_empty() || life.move_energy_cost == 0 {
        return None;
    }

    let cells = &view.cells;
    let origin_habitat = ObservedCellKind::from_cell(&cells.iter().find(|(p, _)| *p == origin)?.1);
    let mut position = origin;
    let mut route = Vec::with_capacity(route_steps.len());
    let mut previous_direction = None;
    let mut turns = 0usize;
    for direction in &route_steps.0 {
        position = position + *direction;
        let cell = &cells.iter().find(|(p, _)| *p == position)?.1;
        route.push(ObservedStep {
            direction_sector: direction_sector(*direction),
            cell_kind: ObservedCellKind::from_cell(cell),
        });
        if previous_direction.is_some_and(|previous| previous != *direction) {
            turns += 1;
        }
        previous_direction = Some(*direction);
    }
    let refuge_kind = route.last()?.cell_kind;
    if refuge_kind != ObservedCellKind::Burrow {
        return None;
    }
    let dx = position.x.checked_sub(origin.x)?;
    let dy = position.y.checked_sub(origin.y)?;
    let endpoint_sector = direction_sector(direction_from_delta(dx, dy)?);
    let estimated_movement_energy = u64::try_from(route_steps.len())
        .ok()?
        .checked_mul(u64::try_from(life.move_energy_cost).ok()?)?;

    Some(GeneralizedRefugeKey {
        subject_species,
        threat_species,
        threat_health,
        threat_power,
        threat_bearing_sector: direction_sector(*threat_steps.0.first()?),
        threat_distance_steps: threat_steps.len(),
        subject_health: life.health,
        subject_max_health: life.max_health,
        subject_energy: life.energy,
        subject_max_energy: life.max_energy,
        subject_stamina: life.stamina,
        subject_max_stamina: life.max_stamina,
        subject_speed: life.speed,
        move_energy_cost: life.move_energy_cost,
        origin_habitat,
        endpoint_sector,
        route,
        refuge_kind,
        turns,
        estimated_movement_energy,
    })
}

fn direction_sector(direction: Direction) -> u8 {
    use Direction::*;
    match direction {
        Up => 0,
        UpRight => 1,
        Right => 2,
        DownRight => 3,
        Down => 4,
        DownLeft => 5,
        Left => 6,
        UpLeft => 7,
    }
}

fn direction_from_delta(dx: isize, dy: isize) -> Option<Direction> {
    use Direction::*;
    match (dx.signum(), dy.signum()) {
        (0, -1) => Some(Up),
        (1, -1) => Some(UpRight),
        (1, 0) => Some(Right),
        (1, 1) => Some(DownRight),
        (0, 1) => Some(Down),
        (-1, 1) => Some(DownLeft),
        (-1, 0) => Some(Left),
        (-1, -1) => Some(UpLeft),
        _ => None,
    }
}
