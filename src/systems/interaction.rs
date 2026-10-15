use crate::{
    entity::phase::EntityPhase,
    map::{Map, cell::Cell, movement::Position},
    rules::biology,
    simulation::SimulationEvent,
    world::{AttackPlan, DrinkPlan, EatPlan, EntitySlot, MatePlan},
};
use std::collections::HashMap;

/// Canlıların birbirleriyle veya harita öğeleriyle etkileşimini yöneten sistem.
pub struct InteractionSystem;

impl InteractionSystem {
    fn move_actor(
        entities: &mut [EntitySlot],
        id: usize,
        pos: Position,
        cost: usize,
        occupied: &mut HashMap<Position, usize>,
        events: &mut Vec<SimulationEvent>,
    ) -> bool {
        use crate::simulation::event::{RejectionReason as R, reject};
        if occupied.get(&pos).is_some_and(|other| *other != id) {
            reject(events, id, "Approach", None, R::Occupied);
            return false;
        }
        let Some(slot) = entities
            .iter_mut()
            .find(|s| s.id == id && s.phase.is_active() && s.base.life().is_alive())
        else {
            return false;
        };
        if cost > 0 {
            let from = slot.pos;
            occupied.remove(&from);
            slot.pos = pos;
            slot.base.life_mut().on_move(cost);
            occupied.insert(pos, id);
            events.push(SimulationEvent::EntityMoved { id, from, to: pos });
        }
        true
    }
    fn adjacent(a: Position, b: Position) -> bool {
        (a.x - b.x).abs() <= 1 && (a.y - b.y).abs() <= 1
    }

    pub fn resolve_eats(
        entities: &mut [EntitySlot],
        map: &mut Map,
        plans: &[EatPlan],
        occupied: &mut HashMap<Position, usize>,
        events: &mut Vec<SimulationEvent>,
    ) {
        use crate::simulation::event::{RejectionReason as R, reject};
        let plans = plans.to_vec();
        for p in plans {
            if !Self::move_actor(entities, p.id, p.new_pos, p.cost, occupied, events)
                || !Self::adjacent(p.new_pos, p.resource_pos)
            {
                continue;
            }
            let Some(eater) = entities.iter().position(|s| s.id == p.id) else {
                continue;
            };
            let species = entities[eater].base.species();
            let remaining = entities[eater]
                .base
                .life()
                .max_energy
                .saturating_sub(entities[eater].base.life().energy);
            let amount = if let Some(id) = p.corpse_id {
                if !species.eats_meat() {
                    reject(events, p.id, "Eat", Some(id), R::InvalidDiet);
                    continue;
                }
                let Some(corpse) = entities.iter().position(|s| {
                    s.id == id
                        && s.pos == p.resource_pos
                        && matches!(s.phase, EntityPhase::Corpse { .. })
                }) else {
                    reject(events, p.id, "Eat", Some(id), R::MissingResource);
                    continue;
                };
                let amount = entities[corpse]
                    .corpse_energy
                    .min(biology::EAT_ENERGY_AMOUNT)
                    .min(remaining);
                entities[corpse].corpse_energy -= amount;
                if entities[corpse].corpse_energy == 0 {
                    entities[corpse].phase = EntityPhase::Removed;
                    events.push(SimulationEvent::CorpseConsumed {
                        id,
                        eater: p.id,
                        pos: p.resource_pos,
                    });
                }
                amount
            } else {
                if !species.eats_plants() {
                    reject(events, p.id, "Eat", None, R::InvalidDiet);
                    continue;
                }
                let Some(Cell::Food { amount }) = map.cell(p.resource_pos) else {
                    reject(events, p.id, "Eat", None, R::MissingResource);
                    continue;
                };
                let amount = (*amount).min(biology::EAT_ENERGY_AMOUNT).min(remaining);
                map.reduce_cell_amount(p.resource_pos, amount);
                amount
            };
            if amount > 0 {
                entities[eater].base.life_mut().restore_energy(amount);
                events.push(SimulationEvent::EntityAte {
                    id: p.id,
                    pos: p.resource_pos,
                    amount,
                });
            }
        }
    }
    pub fn resolve_drinks(
        entities: &mut [EntitySlot],
        map: &mut Map,
        plans: &[DrinkPlan],
        occupied: &mut HashMap<Position, usize>,
        events: &mut Vec<SimulationEvent>,
    ) {
        use crate::simulation::event::{RejectionReason as R, reject};
        let plans = plans.to_vec();
        for p in plans {
            if !Self::move_actor(entities, p.id, p.new_pos, p.cost, occupied, events)
                || !Self::adjacent(p.new_pos, p.resource_pos)
            {
                continue;
            }
            let Some(slot) = entities.iter_mut().find(|s| s.id == p.id) else {
                continue;
            };
            let Some(Cell::Water { amount }) = map.cell(p.resource_pos) else {
                reject(events, p.id, "Drink", None, R::MissingResource);
                continue;
            };
            let amount = (*amount).min(biology::DRINK_WATER_AMOUNT).min(
                slot.base
                    .life()
                    .max_water
                    .saturating_sub(slot.base.life().water),
            );
            if amount > 0 {
                slot.base.life_mut().restore_water(amount);
                map.reduce_cell_amount(p.resource_pos, amount);
                events.push(SimulationEvent::EntityDrank {
                    id: p.id,
                    pos: p.resource_pos,
                    amount,
                });
            }
        }
    }

    pub fn resolve_mates(
        entities: &mut Vec<EntitySlot>,
        map: &Map,
        plans: &[MatePlan],
        occupied: &mut HashMap<Position, usize>,
        events: &mut Vec<SimulationEvent>,
        next_entity_id: &mut usize,
    ) {
        use crate::simulation::event::{RejectionReason as R, reject};
        let mut new_entities = Vec::new();
        for plan in plans {
            let s_idx = entities.iter().position(|s| s.id == plan.self_id);
            let t_idx = entities.iter().position(|s| s.id == plan.target_id);
            let (s, t) = match (s_idx, t_idx) {
                (Some(s), Some(t)) if s != t => (s, t),
                _ => {
                    reject(
                        events,
                        plan.self_id,
                        "Mate",
                        Some(plan.target_id),
                        R::MissingEntity,
                    );
                    continue;
                }
            };
            let parent = &entities[s];
            let partner = &entities[t];
            let reason = if !parent.phase.is_active() || !partner.phase.is_active() {
                Some(R::Inactive)
            } else if parent.base.species() != partner.base.species() {
                Some(R::DifferentSpecies)
            } else if (parent.pos.x - partner.pos.x).abs() > 1
                || (parent.pos.y - partner.pos.y).abs() > 1
            {
                Some(R::OutOfRange)
            } else {
                parent
                    .base
                    .life()
                    .reproduction_blocker()
                    .or_else(|| partner.base.life().reproduction_blocker())
            };
            if let Some(reason) = reason {
                reject(events, plan.self_id, "Mate", Some(plan.target_id), reason);
                continue;
            }
            let Some(pos) = crate::world::World::pick_empty_neighbor(map, partner.pos, occupied)
            else {
                reject(
                    events,
                    plan.self_id,
                    "Mate",
                    Some(plan.target_id),
                    R::NoBirthSpace,
                );
                continue;
            };
            let energy_budget =
                entities[s].base.life().max_energy / 4 + entities[t].base.life().max_energy / 4;
            let water_budget =
                entities[s].base.life().max_water / 4 + entities[t].base.life().max_water / 4;
            entities[s].base.life_mut().on_reproduce();
            entities[t].base.life_mut().on_reproduce();
            let mut child = entities[t].base.reproduce();
            child.life_mut().energy = energy_budget.min(child.life().max_energy);
            child.life_mut().water = water_budget.min(child.life().max_water);
            let species = child.species();
            let id = *next_entity_id;
            *next_entity_id += 1;
            let generation = entities[s].generation.max(entities[t].generation) + 1;
            new_entities.push(EntitySlot::new(
                id,
                pos,
                EntityPhase::Active,
                crate::entity::controller::EntityController::AI,
                child,
            ));
            if let Some(child) = new_entities.last_mut() {
                child.generation = generation;
                child.parents = vec![plan.self_id, plan.target_id];
            }
            occupied.insert(pos, id);
            events.push(SimulationEvent::EntityMated {
                parent1: plan.self_id,
                parent2: plan.target_id,
                child: id,
                pos,
            });
            events.push(SimulationEvent::EntityBorn { id, pos, species });
        }
        entities.extend(new_entities);
    }

    pub fn resolve_attacks(
        entities: &mut [EntitySlot],
        plans: &[AttackPlan],
        events: &mut Vec<SimulationEvent>,
    ) {
        use crate::simulation::event::{RejectionReason as R, reject};
        for plan in plans {
            let a_idx = entities.iter().position(|s| s.id == plan.attacker_id);
            let t_idx = entities.iter().position(|s| s.id == plan.target_id);

            let (a_idx, t_idx) = match (a_idx, t_idx) {
                (Some(a), Some(t)) if a != t => (a, t),
                _ => {
                    reject(
                        events,
                        plan.attacker_id,
                        "Attack",
                        Some(plan.target_id),
                        R::MissingEntity,
                    );
                    continue;
                }
            };

            let (first, second) = if a_idx < t_idx {
                let (left, right) = entities.split_at_mut(t_idx);
                (&mut left[a_idx], &mut right[0])
            } else {
                let (left, right) = entities.split_at_mut(a_idx);
                (&mut left[t_idx], &mut right[0])
            };

            let (attacker, target) = if a_idx < t_idx {
                (first, second)
            } else {
                (second, first)
            };

            if !attacker.phase.is_active()
                || !attacker.base.life().is_alive()
                || !matches!(
                    target.phase,
                    EntityPhase::Active | EntityPhase::Sleeping { .. }
                )
                || !target.base.life().is_alive()
            {
                reject(
                    events,
                    plan.attacker_id,
                    "Attack",
                    Some(plan.target_id),
                    R::Inactive,
                );
                continue;
            }
            let combat = attacker.base.combat();
            if !attacker.base.life().enough_energy(combat.energy_cost) {
                reject(
                    events,
                    plan.attacker_id,
                    "Attack",
                    Some(plan.target_id),
                    R::InsufficientEnergy,
                );
                continue;
            }

            if attacker.base.life().stamina < combat.stamina_cost {
                reject(
                    events,
                    plan.attacker_id,
                    "Attack",
                    Some(plan.target_id),
                    R::Fatigued,
                );
                continue;
            }
            let dx = (attacker.pos.x - target.pos.x).abs();
            let dy = (attacker.pos.y - target.pos.y).abs();
            if dx <= 1 && dy <= 1 {
                attacker
                    .entity_mut()
                    .life_mut()
                    .consume_energy(combat.energy_cost);
                attacker.base.life_mut().exert(combat.stamina_cost);
                target.entity_mut().life_mut().take_damage(combat.damage);

                events.push(SimulationEvent::EntityAttacked {
                    attacker: plan.attacker_id,
                    target: plan.target_id,
                    damage: combat.damage,
                });
            } else {
                reject(
                    events,
                    plan.attacker_id,
                    "Attack",
                    Some(plan.target_id),
                    R::OutOfRange,
                );
            }
        }
    }
}
