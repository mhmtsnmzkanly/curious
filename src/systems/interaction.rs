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
    pub fn resolve_eats(
        entities: &mut [EntitySlot],
        map: &mut Map,
        plans: &[EatPlan],
        occupied: &mut HashMap<Position, usize>,
        events: &mut Vec<SimulationEvent>,
    ) {
        let mut target_winners: HashMap<Position, &EatPlan> = HashMap::new();
        let mut sorted_plans = plans.to_vec();
        sorted_plans.sort_by_key(|p| p.id);

        for plan in &sorted_plans {
            target_winners.entry(plan.new_pos).or_insert(plan);
        }

        let mut winners: Vec<_> = target_winners.values().copied().collect();
        winners.sort_by_key(|p| p.id);
        for plan in winners {
            if let Some(other_id) = occupied.get(&plan.new_pos) {
                if *other_id != plan.id {
                    continue;
                }
            }

            // Önce yiyen birimin ve varsa hedef cesedin indekslerini bulalım
            let eater_idx = entities.iter().position(|s| s.id == plan.id);
            let corpse_idx = entities.iter().position(|s| {
                s.id != plan.id
                    && s.pos == plan.new_pos
                    && matches!(s.phase, EntityPhase::Corpse { .. })
            });

            match (eater_idx, corpse_idx) {
                // Sadece yiyen birim bulundu (Bitki yeme ihtimali)
                (Some(e_idx), None) => {
                    let slot = &mut entities[e_idx];
                    let from = slot.pos;
                    occupied.remove(&from);
                    slot.pos = plan.new_pos;
                    slot.base.life_mut().on_move(plan.cost);
                    occupied.insert(slot.pos, plan.id);

                    if let Some(Cell::Food { amount }) = map.cell(slot.pos) {
                        let eat_amount = (*amount).min(biology::EAT_ENERGY_AMOUNT);
                        slot.entity_mut().life_mut().restore_energy(eat_amount);
                        map.reduce_cell_amount(slot.pos, eat_amount);
                        events.push(SimulationEvent::EntityAte {
                            id: plan.id,
                            pos: slot.pos,
                            amount: eat_amount,
                        });
                    }
                }
                // Hem yiyen hem ceset bulundu (Etçil/Hepçil durumu)
                (Some(e_idx), Some(c_idx)) => {
                    let (first, second) = if e_idx < c_idx {
                        let (left, right) = entities.split_at_mut(c_idx);
                        (&mut left[e_idx], &mut right[0])
                    } else {
                        let (left, right) = entities.split_at_mut(e_idx);
                        (&mut left[c_idx], &mut right[0])
                    };

                    let (eater, corpse) = if e_idx < c_idx {
                        (first, second)
                    } else {
                        (second, first)
                    };

                    // Yiyen birimi hareket ettir
                    let from = eater.pos;
                    occupied.remove(&from);
                    eater.pos = plan.new_pos;
                    eater.base.life_mut().on_move(plan.cost);
                    occupied.insert(eater.pos, plan.id);

                    // Cesedi ye
                    let eat_amount = biology::EAT_ENERGY_AMOUNT * 2;
                    eater.entity_mut().life_mut().restore_energy(eat_amount);
                    corpse.phase = EntityPhase::Removed;

                    events.push(SimulationEvent::EntityAte {
                        id: plan.id,
                        pos: eater.pos,
                        amount: eat_amount,
                    });
                    events.push(SimulationEvent::CorpseConsumed {
                        id: corpse.id,
                        eater: plan.id,
                        pos: corpse.pos,
                    });
                }
                _ => {}
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
        let mut target_winners: HashMap<Position, &DrinkPlan> = HashMap::new();
        let mut sorted_plans = plans.to_vec();
        sorted_plans.sort_by_key(|p| p.id);

        for plan in &sorted_plans {
            target_winners.entry(plan.new_pos).or_insert(plan);
        }

        let mut winners: Vec<_> = target_winners.values().copied().collect();
        winners.sort_by_key(|p| p.id);
        for plan in winners {
            if let Some(other_id) = occupied.get(&plan.new_pos) {
                if *other_id != plan.id {
                    continue;
                }
            }

            if let Some(slot) = entities.iter_mut().find(|s| s.id == plan.id) {
                let from = slot.pos;
                occupied.remove(&from);
                slot.pos = plan.new_pos;
                slot.base.life_mut().on_move(plan.cost);
                occupied.insert(slot.pos, plan.id);

                if let Some(Cell::Water { amount }) = map.cell(slot.pos) {
                    let drink_amount = (*amount).min(biology::DRINK_WATER_AMOUNT);
                    slot.entity_mut().life_mut().restore_water(drink_amount);
                    map.reduce_cell_amount(slot.pos, drink_amount);

                    events.push(SimulationEvent::EntityDrank {
                        id: plan.id,
                        pos: slot.pos,
                        amount: drink_amount,
                    });
                }
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
            entities[s].base.life_mut().on_reproduce();
            entities[t].base.life_mut().on_reproduce();
            let child = entities[t].base.reproduce();
            let species = child.species();
            let id = *next_entity_id;
            *next_entity_id += 1;
            new_entities.push(EntitySlot::new(
                id,
                pos,
                EntityPhase::Active,
                crate::entity::controller::EntityController::AI,
                child,
            ));
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
            if !attacker
                .base
                .life()
                .enough_energy(biology::ATTACK_ENERGY_COST)
            {
                reject(
                    events,
                    plan.attacker_id,
                    "Attack",
                    Some(plan.target_id),
                    R::InsufficientEnergy,
                );
                continue;
            }

            let dx = (attacker.pos.x - target.pos.x).abs();
            let dy = (attacker.pos.y - target.pos.y).abs();
            if dx <= 1 && dy <= 1 {
                attacker
                    .entity_mut()
                    .life_mut()
                    .consume_energy(biology::ATTACK_ENERGY_COST);
                target
                    .entity_mut()
                    .life_mut()
                    .take_damage(biology::ATTACK_DAMAGE_AMOUNT);

                events.push(SimulationEvent::EntityAttacked {
                    attacker: plan.attacker_id,
                    target: plan.target_id,
                    damage: biology::ATTACK_DAMAGE_AMOUNT,
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
