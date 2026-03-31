use std::collections::HashMap;
use crate::{
    map::movement::Position,
    world::{EntitySlot, MovePlan, FleePlan},
    simulation::SimulationEvent,
};

/// Hareketle ilgili simülasyon mantığını yürüten sistem.
pub struct MovementSystem;

impl MovementSystem {
    /// Hareket planlarını çakışma çözümü ile gerçekleştirir.
    pub fn resolve_moves(
        entities: &mut [EntitySlot],
        plans: &[MovePlan],
        occupied: &mut HashMap<Position, usize>,
        events: &mut Vec<SimulationEvent>,
    ) {
        let mut target_winners: HashMap<Position, &MovePlan> = HashMap::new();
        let mut sorted_plans = plans.to_vec();
        sorted_plans.sort_by_key(|p| p.id);

        for plan in &sorted_plans {
            target_winners.entry(plan.new_pos).or_insert(plan);
        }

        for plan in target_winners.values() {
            if let Some(other_id) = occupied.get(&plan.new_pos) {
                if *other_id != plan.id { continue; }
            }

            if let Some(slot) = entities.iter_mut().find(|s| s.id == plan.id) {
                let from = slot.pos;
                occupied.remove(&from);
                slot.base.life_mut().on_move(plan.cost);
                slot.pos = plan.new_pos;
                occupied.insert(slot.pos, plan.id);
                
                events.push(SimulationEvent::EntityMoved { id: plan.id, from, to: plan.new_pos });
            }
        }
    }

    /// Kaçma (flee) planlarını çakışma çözümü ile gerçekleştirir.
    pub fn resolve_flee(
        entities: &mut [EntitySlot],
        plans: &[FleePlan],
        occupied: &mut HashMap<Position, usize>,
        events: &mut Vec<SimulationEvent>,
    ) {
        let mut target_winners: HashMap<Position, &FleePlan> = HashMap::new();
        let mut sorted_plans = plans.to_vec();
        sorted_plans.sort_by_key(|p| p.id);

        for plan in &sorted_plans {
            target_winners.entry(plan.new_pos).or_insert(plan);
        }

        for plan in target_winners.values() {
            if let Some(other_id) = occupied.get(&plan.new_pos) {
                if *other_id != plan.id { continue; }
            }

            if let Some(slot) = entities.iter_mut().find(|s| s.id == plan.id) {
                let from = slot.pos;
                occupied.remove(&from);
                slot.base.life_mut().on_move(plan.cost);
                slot.pos = plan.new_pos;
                occupied.insert(slot.pos, plan.id);
                
                events.push(SimulationEvent::EntityMoved { id: plan.id, from, to: plan.new_pos });
            }
        }
    }
}
