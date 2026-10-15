use crate::{
    world::{World, EntitySlot, ActionPlans, MovePlan, EatPlan, DrinkPlan, MatePlan, AttackPlan, FleePlan, SleepPlan},
    entity::intent::Intent,
    map::movement::{Position, Steps, DIRECTION_ARRAY},
};

/// Ham niyetleri (Intent) doğrulanmış ve maliyeti hesaplanmış eylem planlarına dönüştüren sistem.
pub struct PlanningSystem;

impl PlanningSystem {
    /// Tüm birimlerin niyetlerini değerlendirir ve bir ActionPlans objesi döner.
    pub fn plan(world: &mut World, intents: Vec<(usize, Intent)>, log_lines: &mut Vec<String>) -> ActionPlans {
        let mut plans = ActionPlans::default();

        for (id, intent) in intents {
            match intent {
                Intent::Move { steps } => {
                    if let Some(slot) = Self::find_slot(world, id) {
                        let (new_pos, cost, path) = Self::evaluate_path(world, slot, &steps);
                        {
                            plans.moves.push(MovePlan { id, new_pos, cost, path });
                            log_lines.push(format!("[Plan] Move @{} -> {:?} adim:{}", id, new_pos, cost));
                        }
                    }
                }
                Intent::Eat { at, corpse_id } => {
                    if let Some(slot) = Self::find_slot(world, id) {
                        let resource_pos=at.0.iter().fold(slot.pos,|p,d|p+*d);
                        let approach=Steps(at.0.iter().take(at.len().saturating_sub(1)).copied().collect());
                        let (new_pos, cost, path) = Self::evaluate_path(world, slot, &approach);
                        plans.eats.push(EatPlan { id, new_pos, cost, path, resource_pos,corpse_id });
                        log_lines.push(format!("[Plan] Eat @{} -> {:?} adim:{}", id, new_pos, cost));
                    }
                }
                Intent::Drink { at } => {
                    if let Some(slot) = Self::find_slot(world, id) {
                        let resource_pos=at.0.iter().fold(slot.pos,|p,d|p+*d);
                        let approach=Steps(at.0.iter().take(at.len().saturating_sub(1)).copied().collect());
                        let (new_pos, cost, path) = Self::evaluate_path(world, slot, &approach);
                        plans.drinks.push(DrinkPlan { id, new_pos, cost, path, resource_pos });
                        log_lines.push(format!("[Plan] Drink @{} -> {:?} adim:{}", id, new_pos, cost));
                    }
                }
                Intent::Mate { target_id } => {
                    plans.mates.push(MatePlan { self_id: id, target_id });
                    log_lines.push(format!("[Plan] Mate @{} + @{}", id, target_id));
                }
                Intent::Attack { target_id } => {
                    plans.attacks.push(AttackPlan { attacker_id: id, target_id });
                    log_lines.push(format!("[Plan] Attack @{} -> @{}", id, target_id));
                }
                Intent::Flee { target_id } => {
                    if let (Some(slot), Some(target)) = (Self::find_slot(world, id), Self::find_slot(world, target_id)) {
                        let (new_pos, cost, path) = Self::calculate_flee_path(world, slot, target.pos);
                        plans.flees.push(FleePlan { id, new_pos, cost, path });
                        log_lines.push(format!("[Plan] Flee @{} -> {:?} (hedef @{}) adim:{}", id, new_pos, target_id, cost));
                    }
                }
                Intent::Idle { .. } => {}
                Intent::Sleep { duration } => {
                    plans.sleeps.push(SleepPlan { id, duration });
                    log_lines.push(format!("[Plan] Sleep @{} sure:{}", id, duration));
                }
            }
        }
        plans
    }

    fn find_slot<'a>(world: &'a World, id: usize) -> Option<&'a EntitySlot> {
        world.entities.iter().find(|s| s.id == id)
    }

    fn evaluate_path(world: &World, slot: &EntitySlot, steps: &Steps) -> (Position, usize, Vec<Position>) {
        let mut curr_pos = slot.pos;
        let mut cost = 0;
        let mut path=Vec::new();
        for dir in steps {
            let next = curr_pos + *dir;
            if !world.map.can_step(curr_pos,next) || world.entities.iter().any(|s|s.id!=slot.id && s.pos==next && matches!(s.phase,crate::entity::phase::EntityPhase::Active|crate::entity::phase::EntityPhase::Sleeping{..})) || !slot.base.life().can_move_for(cost + 1) {
                break;
            }
            curr_pos = next;
            path.push(curr_pos);
            cost += 1;
        }
        (curr_pos, cost, path)
    }

    fn calculate_flee_path(world: &World, slot: &EntitySlot, target_pos: Position) -> (Position, usize, Vec<Position>) {
        let mut curr_pos = slot.pos;
        let mut cost = 0;
        let mut path=Vec::new();
        let speed = slot.base.life().speed;

        for _ in 0..speed {
            let mut best_dir = None;
            let mut max_dist = curr_pos.distance_to(target_pos);

            for dir in DIRECTION_ARRAY {
                let candidate = curr_pos + dir;
                if !world.map.can_step(curr_pos,candidate) || world.entities.iter().any(|s|s.id!=slot.id && s.pos==candidate && matches!(s.phase,crate::entity::phase::EntityPhase::Active|crate::entity::phase::EntityPhase::Sleeping{..})) { continue; }
                let d = candidate.distance_to(target_pos);
                if d > max_dist {
                    max_dist = d;
                    best_dir = Some(dir);
                }
            }

            match best_dir {
                Some(dir) if slot.base.life().can_move_for(cost + 1) => {
                    curr_pos = curr_pos + dir;
                    path.push(curr_pos);
                    cost += 1;
                }
                _ => break,
            }
        }
        (curr_pos, cost, path)
    }

}
