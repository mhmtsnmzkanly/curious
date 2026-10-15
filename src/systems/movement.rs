use crate::{
    map::{Map, movement::Position},
    simulation::{
        SimulationEvent,
        event::{RejectionReason, reject},
    },
    world::{EntitySlot, FleePlan, MovePlan},
};
use std::collections::HashMap;
pub struct MovementSystem;
impl MovementSystem {
    pub fn resolve_moves(
        entities: &mut [EntitySlot],
        plans: &[MovePlan],
        map: &Map,
        occupied: &mut HashMap<Position, usize>,
        events: &mut Vec<SimulationEvent>,
    ) {
        for p in plans {
            if p.path.is_empty() {
                reject(events, p.id, "Move", None, RejectionReason::NoPath);
            } else {
                super::locomotion::apply(entities, p.id, &p.path, map, occupied, events);
            }
        }
    }
    pub fn resolve_flee(
        entities: &mut [EntitySlot],
        plans: &[FleePlan],
        map: &Map,
        occupied: &mut HashMap<Position, usize>,
        events: &mut Vec<SimulationEvent>,
    ) {
        for p in plans {
            if p.path.is_empty() {
                reject(events, p.id, "Flee", None, RejectionReason::NoPath);
            } else {
                super::locomotion::apply(entities, p.id, &p.path, map, occupied, events);
            }
        }
    }
}
