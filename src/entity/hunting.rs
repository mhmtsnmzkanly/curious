//! Local prey choice and pursuit budget; no world knowledge or learned state.
use super::{
    decision::{Decision, Reason},
    intent::Intent,
    lifestate::LifeState,
    perception::Perception,
    species::Species,
};
pub fn decide(species: Species, life: &LifeState, view: &Perception) -> Option<Decision> {
    let profile = species.combat_profile();
    let candidates: Vec<_> = view
        .entities
        .iter()
        .filter(|e| species.hunts(&e.species))
        .collect();
    if candidates.is_empty() {
        return None;
    }
    // Movement and at least one attack must remain physically affordable.
    if candidates.iter().all(|e| {
        life.energy
            < profile.energy_cost.saturating_add(
                e.steps
                    .len()
                    .saturating_sub(1)
                    .saturating_mul(life.move_energy_cost),
            )
    }) {
        return Some(Decision::new(
            Intent::Idle { duration: 1 },
            Reason::HuntLowEnergy,
        ));
    }
    if life.stamina <= life.max_stamina / 4 || life.stamina < profile.stamina_cost {
        return Some(Decision::new(
            Intent::Idle { duration: 1 },
            Reason::RecoverStamina,
        ));
    }
    // Prefer a wounded reachable animal over repeatedly starting expensive fresh hunts.
    let cost = |e: &&super::perception::PerceivedEntity| {
        let hits = e.health.unwrap_or(e.power).div_ceil(profile.damage.max(1));
        let travel = e.steps.len().saturating_sub(1);
        travel
            .saturating_mul(life.move_energy_cost)
            .saturating_add(hits.saturating_mul(profile.energy_cost + 1))
    };
    let prey = candidates
        .into_iter()
        .min_by_key(|e| (cost(e), e.steps.len(), e.id))?;
    let estimate = cost(&prey);
    if estimate > life.energy && prey.steps.len() > 1 {
        return Some(Decision::new(
            Intent::Idle { duration: 1 },
            Reason::HuntTooCostly,
        ));
    }
    // Adjacent prey still permits a last affordable attack; failure is not hidden.
    Some(Decision::new(
        if prey.steps.len() <= 1 {
            Intent::Attack { target_id: prey.id }
        } else {
            Intent::Move {
                steps: super::super::map::movement::Steps(
                    prey.steps.0.iter().take(life.speed).copied().collect(),
                ),
            }
        },
        Reason::HuntPrey,
    ))
}
