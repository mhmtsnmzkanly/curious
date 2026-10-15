//! Decision evidence captured at the point of choice, then linked to actual outcomes.
use super::{
    intent::Intent, lifestate::LifeState, perception::Perception, phase::EntityPhase,
    species::Species,
};
use crate::{map::movement::Position, simulation::SimulationEvent};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Reason {
    EscapeThreat,
    SeekShelter,
    CorneredDefense,
    RecoverStamina,
    HuntLowEnergy,
    HuntTooCostly,
    Defend,
    SeekWater,
    SearchWater,
    RecallWater,
    LearnedWater,
    RecallFood,
    SeekFood,
    HuntPrey,
    SearchFood,
    ApproachMate,
    Mate,
    NoEligibleMate,
    Explore,
    Rest,
    ExternalCommand,
    PlayerWaiting,
    CustomPolicy,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    pub intent: Intent,
    pub reason: Reason,
}
impl Decision {
    pub fn new(intent: Intent, reason: Reason) -> Self {
        Self { intent, reason }
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ObservationSettings {
    pub full_all: bool,
    pub full_for: Vec<usize>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeenEntity {
    pub id: usize,
    pub species: Species,
    pub distance: usize,
    pub reproductive: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeenSummary {
    pub foods: usize,
    pub waters: usize,
    pub entities: Vec<SeenEntity>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Target {
    Entity(usize),
    Position(Position),
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Outcome {
    pub pos: Position,
    pub life: LifeState,
    pub phase: EntityPhase,
    pub events: Vec<SimulationEvent>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionTrace {
    pub tick: usize,
    pub id: usize,
    pub species: Species,
    pub pos: Position,
    pub before: LifeState,
    pub rng_before: u64,
    pub seen: SeenSummary,
    pub perception: Option<Perception>,
    pub decision: Decision,
    pub target: Option<Target>,
    pub outcome: Option<Outcome>,
}
impl DecisionTrace {
    pub fn new(
        tick: usize,
        id: usize,
        species: Species,
        pos: Position,
        life: LifeState,
        rng: u64,
        view: Perception,
        decision: Decision,
        full: bool,
    ) -> Self {
        let target = match &decision.intent {
            Intent::Attack { target_id }
            | Intent::Flee { target_id }
            | Intent::Mate { target_id } => Some(Target::Entity(*target_id)),
            Intent::Move { steps }
            | Intent::Eat { at: steps, .. }
            | Intent::Drink { at: steps } => {
                Some(Target::Position(steps.0.iter().fold(pos, |p, d| p + *d)))
            }
            _ => None,
        };
        let seen = SeenSummary {
            foods: view.foods.len(),
            waters: view.waters.len(),
            entities: view
                .entities
                .iter()
                .map(|e| SeenEntity {
                    id: e.id,
                    species: e.species.clone(),
                    distance: e.steps.len(),
                    reproductive: e.reproductive,
                })
                .collect(),
        };
        Self {
            tick,
            id,
            species,
            pos,
            before: life,
            rng_before: rng,
            seen,
            perception: if full { Some(view) } else { None },
            decision,
            target,
            outcome: None,
        }
    }
}
