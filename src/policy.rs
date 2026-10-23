//! Explicit experimental policy options, separate from physical eligibility.
use crate::entity::{
    decision::{Decision, Reason},
    intent::Intent,
    lifestate::LifeState,
    perception::Perception,
    species::Species,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicySettings {
    /// Unqualified herbivore-only rendezvous; opt in independently of reserves.
    #[serde(default)]
    pub mate_rendezvous: bool,
    /// Not ecologically qualified: always opt in, including after legacy restore.
    #[serde(default)]
    pub reserve_opportunities: bool,
    /// Experimental current-foraging baseline plus own-source residual choice.
    #[serde(default, skip_serializing_if = "is_false")]
    pub water_residual_enabled: bool,
}

fn is_false(value: &bool) -> bool {
    !value
}

/// One immediately executable meal, not an abandoned multi-tick journey.
/// Carnivores can prepare for a visible adult before the pair is adjacent.
/// Never overrides survival, thirst, threats, existing hunger, or custom policies.
pub fn prepare_reserve(
    species: Species,
    life: &LifeState,
    view: &Perception,
    base: Decision,
) -> Decision {
    if !matches!(
        base.reason,
        Reason::Rest | Reason::Explore | Reason::NoEligibleMate
    ) || life.reproduction_blocker()
        != Some(crate::simulation::event::RejectionReason::InsufficientEnergy)
        || life.health != life.max_health
        || life.water < life.max_water * 3 / 4
        // Preparing requires a potential adult partner, not an already ready
        // one: otherwise two under-reserved adults wait for each other forever.
        // A carnivore meal can be brief and sparse: eat in place while an adult
        // is visible, rather than requiring adjacency before preparing to meet.
        // This does not authorize travel to food or chasing an unseen partner.
        || !view
            .entities
            .iter()
            .any(|mate| {
                mate.species == species
                    && mate.mature
                    && (species == Species::Carnivore || mate.steps.len() <= 1)
            })
    {
        return base;
    }
    // Leave one unit for next tick's basal cost. Do not count hoped-for future meals.
    let target = life.reproduction_energy_required().saturating_add(1);
    if target > life.max_energy || life.energy >= target {
        return base;
    }
    let needed = target - life.energy;
    let food = view
        .foods
        .iter()
        .filter(|food| {
            food.steps.len() <= 1
                && (if food.is_corpse {
                    species.eats_meat()
                } else {
                    species.eats_plants()
                })
                && food
                    .amount
                    .min(life.max_meal)
                    .min(life.max_energy - life.energy)
                    >= needed
        })
        .max_by_key(|food| food.amount.min(life.max_meal));
    match food {
        Some(food) => Decision::new(
            Intent::Eat {
                at: food.steps.clone(),
                corpse_id: food.corpse_id,
            },
            Reason::PrepareReproduction,
        ),
        None => base,
    }
}
