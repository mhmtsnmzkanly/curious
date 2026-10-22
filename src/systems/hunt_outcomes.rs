//! Connect resolver audit events to one actor's pending hunt. No hidden world data.
use crate::{
    entity::hunt_learning::{HuntLedger, HuntMeal},
    simulation::SimulationEvent,
    world::EntitySlot,
};

pub fn finish(entities: &mut [EntitySlot], events: &[SimulationEvent]) {
    for slot in entities {
        if !slot.hunt_learner.enabled {
            continue;
        }
        let Some(attempt) = &slot.hunt_learner.episode else {
            continue;
        };
        let target = attempt.target;
        let previously_killed = attempt.own_kill;
        let mut ledger = HuntLedger {
            alive: slot.base.life().is_alive(),
            ..Default::default()
        };
        let mut target_died = false;
        for event in events {
            match event {
                SimulationEvent::EntityMoved {
                    id, energy_cost, ..
                } if *id == slot.id => {
                    ledger.movement_energy = ledger.movement_energy.saturating_add(*energy_cost);
                }
                SimulationEvent::EntityAttacked {
                    attacker,
                    target: victim,
                    damage,
                    energy_cost,
                    killed,
                } => {
                    if *attacker == slot.id {
                        ledger.attack_energy = ledger.attack_energy.saturating_add(*energy_cost);
                    }
                    if *victim == slot.id {
                        ledger.incoming_damage = ledger.incoming_damage.saturating_add(*damage);
                        if *attacker == target {
                            ledger.prey_defense_damage =
                                ledger.prey_defense_damage.saturating_add(*damage);
                        } else {
                            ledger.external_damage = ledger.external_damage.saturating_add(*damage);
                        }
                    }
                    if *victim == target && *killed {
                        ledger.target_killed_by = Some(*attacker);
                    }
                }
                SimulationEvent::EntityAte {
                    id,
                    corpse_id: Some(corpse_id),
                    amount,
                    ..
                } if *id == slot.id => {
                    ledger.food.push(HuntMeal {
                        corpse_id: *corpse_id,
                        amount: *amount,
                    });
                }
                SimulationEvent::EntityDied { id, .. } if *id == target => target_died = true,
                _ => {}
            }
        }
        // A metabolic death has no killer. Do not claim it as an unsuccessful hunt.
        if target_died && ledger.target_killed_by.is_none() && !previously_killed {
            ledger.ambiguous = true;
            ledger.target_lost = true;
        }
        slot.hunt_learner.finish(slot.id, &ledger);
    }
}
