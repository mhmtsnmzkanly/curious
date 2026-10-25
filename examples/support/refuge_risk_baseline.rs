//! Pure current-observation refuge effort/risk baseline.
//!
//! The helper does not predict damage. It reports it as unknown until a
//! separately reviewed observational calibration exists.

use curious::{
    entity::{lifestate::LifeState, perception::Perception, species::Species},
    map::{
        cell::Cell,
        movement::{Direction, Position, Steps},
    },
};
use serde::Serialize;

const DAMAGE_UNKNOWN_REASON: &str =
    "no calibrated expected-damage model; visible power is health plus energy, not attack damage";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ObservedCellKind {
    Empty,
    Wall,
    Burrow,
    Food,
    Water,
}

impl From<&Cell> for ObservedCellKind {
    fn from(cell: &Cell) -> Self {
        match cell {
            Cell::Empty => Self::Empty,
            Cell::Wall => Self::Wall,
            Cell::Burrow => Self::Burrow,
            Cell::Food { .. } => Self::Food,
            Cell::Water { .. } => Self::Water,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DamageBaseline {
    pub expected_threat_damage: Option<u64>,
    pub calibration_status: &'static str,
    pub reason: &'static str,
    /// Public species-profile amount, for physical context only. It is not an
    /// expected value and is not used to rank routes.
    pub descriptive_species_attack_damage: Option<u64>,
}

impl DamageBaseline {
    pub fn unknown(threat_species: Option<&Species>) -> Self {
        Self {
            expected_threat_damage: None,
            calibration_status: "unknown_uncalibrated",
            reason: DAMAGE_UNKNOWN_REASON,
            descriptive_species_attack_damage: threat_species
                .map(|species| species.combat_profile().damage as u64),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EffortEstimate {
    /// Conditional cost if every observed path step resolves with the current
    /// per-step movement cost and no route change.
    pub nominal_paid_movement_energy: Option<u64>,
    /// Nominal minimum ticks at current maximum speed, assuming no
    /// interruption, invalidation, or energy/stamina shortfall.
    pub nominal_elapsed_ticks: Option<u64>,
    pub status: &'static str,
}

pub fn estimate_effort(path_steps: usize, speed: usize, move_energy_cost: usize) -> EffortEstimate {
    if path_steps == 0 || speed == 0 {
        return EffortEstimate {
            nominal_paid_movement_energy: None,
            nominal_elapsed_ticks: None,
            status: "unknown_missing_path_or_speed",
        };
    }
    let energy = u64::try_from(path_steps)
        .ok()
        .and_then(|steps| steps.checked_mul(move_energy_cost as u64));
    let ticks = u64::try_from(path_steps)
        .ok()
        .and_then(|steps| u64::try_from(speed).ok().map(|rate| steps.div_ceil(rate)));
    match (energy, ticks) {
        (Some(energy), Some(ticks)) => EffortEstimate {
            nominal_paid_movement_energy: Some(energy),
            nominal_elapsed_ticks: Some(ticks),
            status: "conditional_nominal_estimate",
        },
        _ => EffortEstimate {
            nominal_paid_movement_energy: None,
            nominal_elapsed_ticks: None,
            status: "unknown_checked_arithmetic_limit",
        },
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StaticContactGeometry {
    /// Minimum Chebyshev distance from the threat's current perceived cell to
    /// any currently observed cell on this route. This is current geometry,
    /// not a forecast of threat movement or attack.
    pub minimum_current_clearance: Option<usize>,
    pub route_cells_currently_within_one_cell: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VisibleThreatSummary {
    pub id: usize,
    pub species: Species,
    pub observed_steps: Vec<Direction>,
    /// The current perception's power field (health + energy in this engine).
    pub observed_power: usize,
    /// Optional current observed health; absence remains unknown.
    pub observed_health: Option<usize>,
    pub static_contact_geometry: StaticContactGeometry,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RouteBaseline {
    pub endpoint: Position,
    pub endpoint_kind: Option<ObservedCellKind>,
    pub perception_order: usize,
    pub path: Vec<Direction>,
    pub path_steps: usize,
    pub turns: usize,
    /// One entry per path step; `None` identifies a cell outside the current
    /// perception instead of silently treating it as empty.
    pub observed_route_cells: Vec<Option<ObservedCellKind>>,
    pub effort: EffortEstimate,
    pub damage: DamageBaseline,
    pub selected_threat: Option<VisibleThreatSummary>,
}

/// Return every currently perceived refuge route that the ordinary behavior
/// can enter: non-empty paths with unoccupied endpoints. Input order remains
/// the perception order used by its shortest-path tie break. The opt-in
/// learner's eight-row memory cap does not apply to this ordinary behavior.
pub fn route_baselines(
    origin: Position,
    life: &LifeState,
    view: &Perception,
) -> Vec<RouteBaseline> {
    let selected_threat = select_current_threat(view);
    view.shelters
        .iter()
        .enumerate()
        .filter_map(|(perception_order, path)| {
            if path.0.is_empty() || endpoint_occupied(path, view) {
                return None;
            }
            Some(route_baseline(
                origin,
                life,
                view,
                perception_order,
                path,
                selected_threat.as_ref(),
            ))
        })
        .collect()
}

pub fn baseline_choice(routes: &[RouteBaseline]) -> Option<usize> {
    routes
        .iter()
        .enumerate()
        .min_by_key(|(_, route)| (route.path_steps, route.perception_order))
        .map(|(index, _)| index)
}

pub fn select_current_threat(
    view: &Perception,
) -> Option<curious::entity::perception::PerceivedEntity> {
    // Match the ordinary threat-instinct predicate: the first visible
    // species hunting herbivores within two steps. Keep an empty path if the
    // perception supplies one; the memory learner's extra nonempty-path gate
    // is specific to that opt-in feature.
    view.entities
        .iter()
        .find(|threat| threat.species.hunts(&Species::Herbivore) && threat.steps.len() <= 2)
        .cloned()
}

fn route_baseline(
    origin: Position,
    life: &LifeState,
    view: &Perception,
    perception_order: usize,
    path: &Steps,
    threat: Option<&curious::entity::perception::PerceivedEntity>,
) -> RouteBaseline {
    let mut position = origin;
    let mut observed_cells = Vec::with_capacity(path.len());
    let mut turns = 0usize;
    let mut previous = None;
    for direction in &path.0 {
        position = position + *direction;
        if previous.is_some_and(|prior| prior != *direction) {
            turns += 1;
        }
        previous = Some(*direction);
        match view
            .cells
            .iter()
            .find(|(cell_pos, _)| *cell_pos == position)
        {
            Some((_, cell)) => observed_cells.push(Some(ObservedCellKind::from(cell))),
            None => observed_cells.push(None),
        }
    }
    let endpoint_kind = view
        .cells
        .iter()
        .find(|(cell_pos, _)| *cell_pos == position)
        .map(|(_, cell)| ObservedCellKind::from(cell));
    let contact = threat.map(|visible| summarize_threat(origin, path, visible));
    RouteBaseline {
        endpoint: position,
        endpoint_kind,
        perception_order,
        path: path.0.clone(),
        path_steps: path.len(),
        turns,
        observed_route_cells: observed_cells,
        effort: estimate_effort(path.len(), life.speed, life.move_energy_cost),
        damage: DamageBaseline::unknown(threat.map(|visible| &visible.species)),
        selected_threat: contact,
    }
}

fn endpoint_occupied(path: &Steps, view: &Perception) -> bool {
    let endpoint = path
        .0
        .iter()
        .fold(Position::new(0, 0), |position, direction| {
            position + *direction
        });
    view.entities.iter().any(|entity| {
        entity
            .steps
            .0
            .iter()
            .fold(Position::new(0, 0), |position, direction| {
                position + *direction
            })
            == endpoint
    })
}

fn summarize_threat(
    origin: Position,
    route: &Steps,
    threat: &curious::entity::perception::PerceivedEntity,
) -> VisibleThreatSummary {
    let threat_position = threat
        .steps
        .0
        .iter()
        .fold(origin, |position, direction| position + *direction);
    let route_positions = route
        .0
        .iter()
        .scan(origin, |position, direction| {
            *position = *position + *direction;
            Some(*position)
        })
        .collect::<Vec<_>>();
    let clearances = route_positions
        .iter()
        .map(|position| chebyshev_distance(*position, threat_position))
        .collect::<Vec<_>>();
    VisibleThreatSummary {
        id: threat.id,
        species: threat.species.clone(),
        observed_steps: threat.steps.0.clone(),
        observed_power: threat.power,
        observed_health: threat.health,
        static_contact_geometry: StaticContactGeometry {
            minimum_current_clearance: clearances.iter().copied().min(),
            route_cells_currently_within_one_cell: Some(
                clearances.iter().filter(|distance| **distance <= 1).count(),
            ),
        },
    }
}

fn chebyshev_distance(left: Position, right: Position) -> usize {
    left.x.abs_diff(right.x).max(left.y.abs_diff(right.y))
}
