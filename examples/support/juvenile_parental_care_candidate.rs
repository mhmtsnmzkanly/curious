//! Ontogenetic Juvenile Physiology & Resolver-Verified Parental Care Prototype.
//!
//! Models juvenile stage ontogeny where physical traits scale up with age until maturity,
//! and physical parental care actions (feeding and guarding) governed by strict conservation
//! and zero learned memory inheritance.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScaledPhysiology {
    pub speed: usize,
    pub vision: usize,
    pub damage: usize,
    pub bmr: usize,
    pub is_juvenile: bool,
}

/// Computes physical traits for an individual given their age and maturity threshold.
///
/// Juveniles (age < maturity_age) have scaled-down speed (`(speed * 2 / 3).max(1)`),
/// vision (`(vision * 2 / 3).max(2)`), combat damage (`(damage / 2).max(1)`), and
/// basal metabolic rate (`(bmr / 2).max(1)`).
/// Full adult capacity is reached at `age >= maturity_age`.
pub fn juvenile_physiology(
    age: usize,
    maturity_age: usize,
    adult_speed: usize,
    adult_vision: usize,
    adult_damage: usize,
    adult_bmr: usize,
) -> ScaledPhysiology {
    if age < maturity_age {
        ScaledPhysiology {
            speed: (adult_speed * 2 / 3).max(1),
            vision: (adult_vision * 2 / 3).max(2),
            damage: (adult_damage / 2).max(1),
            bmr: (adult_bmr / 2).max(1),
            is_juvenile: true,
        }
    } else {
        ScaledPhysiology {
            speed: adult_speed,
            vision: adult_vision,
            damage: adult_damage,
            bmr: adult_bmr,
            is_juvenile: false,
        }
    }
}

/// Checks if two coordinates are adjacent (Chebyshev distance <= 1).
pub const fn is_adjacent(pos1: (i32, i32), pos2: (i32, i32)) -> bool {
    let dx = if pos1.0 >= pos2.0 { pos1.0 - pos2.0 } else { pos2.0 - pos1.0 };
    let dy = if pos1.1 >= pos2.1 { pos1.1 - pos2.1 } else { pos2.1 - pos1.1 };
    dx <= 1 && dy <= 1
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParentState {
    pub id: usize,
    pub pos: (i32, i32),
    pub energy: usize,
    pub stamina: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OffspringState {
    pub id: usize,
    pub parents: Vec<usize>,
    pub pos: (i32, i32),
    pub age: usize,
    pub maturity_age: usize,
    pub energy: usize,
    pub max_energy: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FeedError {
    NotOwnOffspring,
    ChildMature,
    NotAdjacent,
    InsufficientStamina,
    ParentHasNoEnergy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeedOutcome {
    pub parent_id: usize,
    pub child_id: usize,
    pub transferred_energy: usize,
    pub parent_energy_after: usize,
    pub parent_stamina_after: usize,
    pub child_energy_after: usize,
}

/// Resolves physical feeding from parent to adjacent juvenile offspring.
///
/// Guarantees exact energy conservation: only the actual restored amount is deducted
/// from the parent, capped by child's capacity and parent's available energy.
/// Deducts feeding exertion cost from parent's stamina.
pub fn resolve_feed_offspring(
    parent: &mut ParentState,
    child: &mut OffspringState,
    transfer_amount: usize,
    feeding_exertion_cost: usize,
) -> Result<FeedOutcome, FeedError> {
    if !child.parents.contains(&parent.id) {
        return Err(FeedError::NotOwnOffspring);
    }
    if child.age >= child.maturity_age {
        return Err(FeedError::ChildMature);
    }
    if !is_adjacent(parent.pos, child.pos) {
        return Err(FeedError::NotAdjacent);
    }
    if parent.stamina < feeding_exertion_cost {
        return Err(FeedError::InsufficientStamina);
    }
    if parent.energy == 0 {
        return Err(FeedError::ParentHasNoEnergy);
    }

    let needed = child.max_energy.saturating_sub(child.energy);
    let actual_transferred = transfer_amount.min(needed).min(parent.energy);

    parent.energy = parent.energy.saturating_sub(actual_transferred);
    parent.stamina = parent.stamina.saturating_sub(feeding_exertion_cost);
    child.energy = (child.energy + actual_transferred).min(child.max_energy);

    Ok(FeedOutcome {
        parent_id: parent.id,
        child_id: child.id,
        transferred_energy: actual_transferred,
        parent_energy_after: parent.energy,
        parent_stamina_after: parent.stamina,
        child_energy_after: child.energy,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuardianState {
    pub id: usize,
    pub pos: (i32, i32),
    pub health: usize,
    pub stamina: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JuvenileVictimState {
    pub id: usize,
    pub parents: Vec<usize>,
    pub pos: (i32, i32),
    pub age: usize,
    pub maturity_age: usize,
    pub health: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuardOutcome {
    pub guardian_id: usize,
    pub juvenile_id: usize,
    pub attacker_id: usize,
    pub intercepted: bool,
    pub intercepted_damage: usize,
    pub damage_taken_by_juvenile: usize,
    pub guardian_health_after: usize,
    pub guardian_stamina_after: usize,
    pub juvenile_health_after: usize,
}

/// Resolves physical guarding of a juvenile by an adjacent parent against an incoming attack.
///
/// When valid (parent of juvenile, juvenile < maturity_age, adjacent, and has sufficient stamina),
/// the guardian intercepts all attack damage and pays the guard stamina cost.
/// If invalid, the juvenile receives the damage directly.
/// Produces an audited outcome with explicit entity attribution.
pub fn resolve_guard_offspring(
    guardian: &mut GuardianState,
    juvenile: &mut JuvenileVictimState,
    attacker_id: usize,
    attack_damage: usize,
    guard_stamina_cost: usize,
) -> GuardOutcome {
    let is_parent = juvenile.parents.contains(&guardian.id);
    let is_juvenile = juvenile.age < juvenile.maturity_age;
    let adjacent = is_adjacent(guardian.pos, juvenile.pos);
    let has_stamina = guardian.stamina >= guard_stamina_cost;

    if is_parent && is_juvenile && adjacent && has_stamina {
        guardian.health = guardian.health.saturating_sub(attack_damage);
        guardian.stamina = guardian.stamina.saturating_sub(guard_stamina_cost);
        GuardOutcome {
            guardian_id: guardian.id,
            juvenile_id: juvenile.id,
            attacker_id,
            intercepted: true,
            intercepted_damage: attack_damage,
            damage_taken_by_juvenile: 0,
            guardian_health_after: guardian.health,
            guardian_stamina_after: guardian.stamina,
            juvenile_health_after: juvenile.health,
        }
    } else {
        juvenile.health = juvenile.health.saturating_sub(attack_damage);
        GuardOutcome {
            guardian_id: guardian.id,
            juvenile_id: juvenile.id,
            attacker_id,
            intercepted: false,
            intercepted_damage: 0,
            damage_taken_by_juvenile: attack_damage,
            guardian_health_after: guardian.health,
            guardian_stamina_after: guardian.stamina,
            juvenile_health_after: juvenile.health,
        }
    }
}
