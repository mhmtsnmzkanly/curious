use crate::{
    map::movement::Position,
    simulation::SimulationEvent,
    world::{EntitySlot, FleePlan, MovePlan},
};
use std::collections::HashMap;

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
        for plan in plans {
            target_winners.entry(plan.new_pos).or_insert(plan);
        }

        for plan in plans {
            if target_winners
                .get(&plan.new_pos)
                .is_none_or(|winner| winner.id != plan.id)
            {
                continue;
            }
            if let Some(other_id) = occupied.get(&plan.new_pos) {
                if *other_id != plan.id {
                    continue;
                }
            }

            if let Some(slot) = entities
                .iter_mut()
                .find(|s| s.id == plan.id && s.phase.is_active() && s.base.life().is_alive())
            {
                let from = slot.pos;
                occupied.remove(&from);
                slot.base.life_mut().on_move(plan.cost);
                slot.pos = plan.new_pos;
                occupied.insert(slot.pos, plan.id);

                events.push(SimulationEvent::EntityMoved {
                    id: plan.id,
                    from,
                    to: plan.new_pos,
                });
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
        for plan in plans {
            target_winners.entry(plan.new_pos).or_insert(plan);
        }

        for plan in plans {
            if target_winners
                .get(&plan.new_pos)
                .is_none_or(|winner| winner.id != plan.id)
            {
                continue;
            }
            if let Some(other_id) = occupied.get(&plan.new_pos) {
                if *other_id != plan.id {
                    continue;
                }
            }

            if let Some(slot) = entities
                .iter_mut()
                .find(|s| s.id == plan.id && s.phase.is_active() && s.base.life().is_alive())
            {
                let from = slot.pos;
                occupied.remove(&from);
                slot.base.life_mut().on_move(plan.cost);
                slot.pos = plan.new_pos;
                occupied.insert(slot.pos, plan.id);

                events.push(SimulationEvent::EntityMoved {
                    id: plan.id,
                    from,
                    to: plan.new_pos,
                });
            }
        }
    }
}
