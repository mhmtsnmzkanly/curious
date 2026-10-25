//! Ambush (Stealth) vs Pursuit Predation Specialization Prototype.
//!
//! Models predator ambush mechanics: stationary stealth near cover, detectability radius
//! reduction bounded to >= 1, first-strike bonuses with immediate ambush clearance,
//! and specialization trade-offs based on K6 alleles (`mobility: u8`, `stamina: u8`).

use curious::{
    entity::genetics::Genome,
    map::cell::Cell,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoverKind {
    Burrow,
    Water,
    Food,
}

/// Identifies whether a cell kind provides natural cover for ambushing.
pub fn is_cover_cell(cell: &Cell) -> Option<CoverKind> {
    match cell {
        Cell::Burrow => Some(CoverKind::Burrow),
        Cell::Water { .. } => Some(CoverKind::Water),
        Cell::Food { .. } => Some(CoverKind::Food),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AmbushState {
    None,
    Active {
        ticks_focused: usize,
        cover: CoverKind,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PredatorSpecialization {
    PursuitSpecialist, // mobility == 2
    AmbushSpecialist,  // mobility == 0 && stamina == 2
    Generalist,        // other allele combinations
}

/// Classifies predator specialization from K6 genome alleles.
pub fn classify_specialization(genome: &Genome) -> PredatorSpecialization {
    if genome.mobility == 2 {
        PredatorSpecialization::PursuitSpecialist
    } else if genome.mobility == 0 && genome.stamina == 2 {
        PredatorSpecialization::AmbushSpecialist
    } else {
        PredatorSpecialization::Generalist
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpecializationProfile {
    pub specialization: PredatorSpecialization,
    pub ambush_focus_energy_cost: usize,
    pub stealth_reduction: usize,
    pub bonus_ambush_damage: usize,
    pub pursuit_stamina_cost_per_step: usize,
    pub max_pursuit_steps: usize,
}

impl SpecializationProfile {
    pub fn from_genome(genome: &Genome) -> Self {
        let specialization = classify_specialization(genome);
        match specialization {
            PredatorSpecialization::AmbushSpecialist => Self {
                specialization,
                ambush_focus_energy_cost: 1,      // Low focus maintenance cost
                stealth_reduction: 3,             // High stealth reduction
                bonus_ambush_damage: 4,           // High first-strike burst damage
                pursuit_stamina_cost_per_step: 3, // Inefficient pursuer
                max_pursuit_steps: 1,             // Minimal sprint capacity
            },
            PredatorSpecialization::PursuitSpecialist => Self {
                specialization,
                ambush_focus_energy_cost: 2,      // Restless waiting cost
                stealth_reduction: 1,             // Poor camouflage
                bonus_ambush_damage: 1,           // Small surprise bonus
                pursuit_stamina_cost_per_step: 1, // Highly stamina-efficient runner
                max_pursuit_steps: 3,             // Multi-step pursuit
            },
            PredatorSpecialization::Generalist => Self {
                specialization,
                ambush_focus_energy_cost: 2,
                stealth_reduction: 2,
                bonus_ambush_damage: 2,
                pursuit_stamina_cost_per_step: 2,
                max_pursuit_steps: 2,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PredatorState {
    pub id: usize,
    pub pos: (i32, i32),
    pub energy: usize,
    pub stamina: usize,
    pub ambush_state: AmbushState,
    pub specialization_profile: SpecializationProfile,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AmbushUpdateResult {
    Activated { cover: CoverKind, energy_paid: usize },
    Maintained { cover: CoverKind, ticks_focused: usize, energy_paid: usize },
    BrokenNotStationary,
    BrokenNoAdjacentCover,
    BrokenInsufficientEnergy,
}

/// Updates predator ambush state based on stationary status and cover adjacency.
///
/// If stationary adjacent to cover and has sufficient energy, enters or maintains ambush
/// while deducting `ambush_focus_energy_cost`. Otherwise clears ambush state.
pub fn update_ambush_state(
    predator: &mut PredatorState,
    is_stationary: bool,
    adjacent_cover: Option<CoverKind>,
) -> AmbushUpdateResult {
    if !is_stationary {
        predator.ambush_state = AmbushState::None;
        return AmbushUpdateResult::BrokenNotStationary;
    }

    let Some(cover) = adjacent_cover else {
        predator.ambush_state = AmbushState::None;
        return AmbushUpdateResult::BrokenNoAdjacentCover;
    };

    let focus_cost = predator.specialization_profile.ambush_focus_energy_cost;
    if predator.energy < focus_cost {
        predator.ambush_state = AmbushState::None;
        return AmbushUpdateResult::BrokenInsufficientEnergy;
    }

    predator.energy = predator.energy.saturating_sub(focus_cost);

    match predator.ambush_state {
        AmbushState::None => {
            predator.ambush_state = AmbushState::Active {
                ticks_focused: 1,
                cover,
            };
            AmbushUpdateResult::Activated {
                cover,
                energy_paid: focus_cost,
            }
        }
        AmbushState::Active { ticks_focused, .. } => {
            let next_ticks = ticks_focused.saturating_add(1);
            predator.ambush_state = AmbushState::Active {
                ticks_focused: next_ticks,
                cover,
            };
            AmbushUpdateResult::Maintained {
                cover,
                ticks_focused: next_ticks,
                energy_paid: focus_cost,
            }
        }
    }
}

/// Computes effective detectability radius against prey vision range.
///
/// Bounded to `.max(1)`: stealth never makes a predator entirely invisible at zero distance.
pub fn effective_detectability_radius(prey_vision_range: usize, stealth_reduction: usize) -> usize {
    prey_vision_range.saturating_sub(stealth_reduction).max(1)
}

/// Tests whether a prey spots the predator given positions, prey vision range, and predator ambush state.
pub fn is_predator_spotted_by_prey(
    predator: &PredatorState,
    prey_pos: (i32, i32),
    prey_vision_range: usize,
) -> bool {
    let dx = if predator.pos.0 >= prey_pos.0 { predator.pos.0 - prey_pos.0 } else { prey_pos.0 - predator.pos.0 } as usize;
    let dy = if predator.pos.1 >= prey_pos.1 { predator.pos.1 - prey_pos.1 } else { prey_pos.1 - predator.pos.1 } as usize;
    let distance = if dx >= dy { dx } else { dy }; // Chebyshev metric

    let effective_range = match predator.ambush_state {
        AmbushState::Active { .. } => effective_detectability_radius(
            prey_vision_range,
            predator.specialization_profile.stealth_reduction,
        ),
        AmbushState::None => prey_vision_range,
    };

    distance <= effective_range
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttackOutcome {
    pub is_ambush_first_strike: bool,
    pub total_damage: usize,
    pub bonus_ambush_damage: usize,
    pub stamina_paid: usize,
    pub predator_stamina_after: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttackError {
    InsufficientStamina,
}

/// Resolves an attack executed by the predator.
///
/// When launched from a verified ambush state:
/// - Grants reduced stamina cost (`stamina_cost.saturating_sub(1).max(1)`)
/// - Adds bonus ambush damage
/// - Immediately clears the ambush state
///
/// When executed without ambush:
/// - Standard damage and full stamina cost are applied.
pub fn resolve_attack(
    predator: &mut PredatorState,
    base_damage: usize,
    base_stamina_cost: usize,
) -> Result<AttackOutcome, AttackError> {
    match predator.ambush_state {
        AmbushState::Active { .. } => {
            let reduced_stamina_cost = base_stamina_cost.saturating_sub(1).max(1);
            if predator.stamina < reduced_stamina_cost {
                return Err(AttackError::InsufficientStamina);
            }

            let bonus = predator.specialization_profile.bonus_ambush_damage;
            let total = base_damage.saturating_add(bonus);

            predator.stamina = predator.stamina.saturating_sub(reduced_stamina_cost);
            // Immediately clears the ambush state!
            predator.ambush_state = AmbushState::None;

            Ok(AttackOutcome {
                is_ambush_first_strike: true,
                total_damage: total,
                bonus_ambush_damage: bonus,
                stamina_paid: reduced_stamina_cost,
                predator_stamina_after: predator.stamina,
            })
        }
        AmbushState::None => {
            if predator.stamina < base_stamina_cost {
                return Err(AttackError::InsufficientStamina);
            }

            predator.stamina = predator.stamina.saturating_sub(base_stamina_cost);

            Ok(AttackOutcome {
                is_ambush_first_strike: false,
                total_damage: base_damage,
                bonus_ambush_damage: 0,
                stamina_paid: base_stamina_cost,
                predator_stamina_after: predator.stamina,
            })
        }
    }
}
