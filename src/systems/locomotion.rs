//! Revalidate every traversed cell at resolution, including moving competitors.
use crate::{
    map::{Map, movement::Position},
    simulation::{
        SimulationEvent,
        event::{RejectionReason as R, reject},
    },
    world::EntitySlot,
};
use std::collections::HashMap;
pub fn apply(
    entities: &mut [EntitySlot],
    id: usize,
    path: &[Position],
    map: &Map,
    occupied: &mut HashMap<Position, usize>,
    events: &mut Vec<SimulationEvent>,
) -> Option<Position> {
    let slot = entities
        .iter_mut()
        .find(|e| e.id == id && e.phase.is_active() && e.base.life().is_alive())?;
    let from = slot.pos;
    let mut cost = 0;
    for next in path {
        let reason = if !map.can_step_for(slot.pos, *next, slot.base.can_burrow()) {
            Some(R::NoPath)
        } else if occupied.get(next).is_some_and(|other| *other != id) {
            Some(R::Occupied)
        } else if slot.base.life().stamina < cost + 1 {
            Some(R::Fatigued)
        } else if !slot.base.life().can_move_for(cost + 1) {
            Some(R::InsufficientEnergy)
        } else {
            None
        };
        if let Some(reason) = reason {
            reject(events, id, "Move", None, reason);
            break;
        }
        occupied.remove(&slot.pos);
        slot.pos = *next;
        occupied.insert(*next, id);
        cost += 1;
    }
    if cost > 0 {
        let before_energy = slot.base.life().energy;
        slot.base.life_mut().on_move(cost);
        let energy_cost = before_energy.saturating_sub(slot.base.life().energy);
        events.push(SimulationEvent::EntityMoved {
            id,
            energy_cost,
            from,
            to: slot.pos,
        });
    }
    Some(slot.pos)
}
