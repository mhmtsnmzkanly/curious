use crate::{
    world::{World, EntitySlot, ActionPlans, MovePlan, EatPlan, DrinkPlan, MatePlan, AttackPlan, FleePlan, SleepPlan},
    entity::intent::Intent,
    map::movement::{Position, Steps, DIRECTION_ARRAY},
    gen_range,
};

/// Ham niyetleri (Intent) doğrulanmış ve maliyeti hesaplanmış eylem planlarına dönüştüren sistem.
pub struct PlanningSystem;

impl PlanningSystem {
    /// Tüm birimlerin niyetlerini değerlendirir ve bir ActionPlans objesi döner.
    pub fn plan(world: &World, intents: Vec<(usize, Intent)>, log_lines: &mut Vec<String>) -> ActionPlans {
        let mut plans = ActionPlans::default();

        for (id, intent) in intents {
            match intent {
                Intent::Move { steps } => {
                    if let Some(slot) = Self::find_slot(world, id) {
                        let (new_pos, cost) = Self::evaluate_path(world, slot, &steps);
                        if cost > 0 {
                            plans.moves.push(MovePlan { id, new_pos, cost });
                            log_lines.push(format!("[Plan] Move @{} -> {:?} adim:{}", id, new_pos, cost));
                        }
                    }
                }
                Intent::Eat { at, .. } => {
                    if let Some(slot) = Self::find_slot(world, id) {
                        let (new_pos, cost) = Self::evaluate_path(world, slot, &at);
                        plans.eats.push(EatPlan { id, new_pos, cost });
                        log_lines.push(format!("[Plan] Eat @{} -> {:?} adim:{}", id, new_pos, cost));
                    }
                }
                Intent::Drink { at } => {
                    if let Some(slot) = Self::find_slot(world, id) {
                        let (new_pos, cost) = Self::evaluate_path(world, slot, &at);
                        plans.drinks.push(DrinkPlan { id, new_pos, cost });
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
                        let (new_pos, cost) = Self::calculate_flee_path(world, slot, target.pos);
                        plans.flees.push(FleePlan { id, new_pos, cost });
                        log_lines.push(format!("[Plan] Flee @{} -> {:?} (hedef @{}) adim:{}", id, new_pos, target_id, cost));
                    }
                }
                Intent::Idle { .. } => {
                    if let Some(slot) = Self::find_slot(world, id) {
                        // Hafif gezinme davranışı: %30 ihtimalle rastgele 1 adım
                        if gen_range(1, 100) <= 30 && slot.base.life().can_move_for(1) {
                            if let Some(pos) = Self::pick_random_neighbor(world, slot.pos) {
                                plans.moves.push(MovePlan { id, new_pos: pos, cost: 1 });
                                log_lines.push(format!("[Plan] Idle->Move @{} -> {:?}", id, pos));
                            }
                        }
                    }
                }
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

    fn evaluate_path(world: &World, slot: &EntitySlot, steps: &Steps) -> (Position, usize) {
        let mut curr_pos = slot.pos;
        let mut cost = 0;
        for dir in steps {
            let next = curr_pos + *dir;
            if !world.map.is_walkable(next) || !slot.base.life().can_move_for(cost + 1) {
                break;
            }
            curr_pos = next;
            cost += 1;
        }
        (curr_pos, cost)
    }

    fn calculate_flee_path(world: &World, slot: &EntitySlot, target_pos: Position) -> (Position, usize) {
        let mut curr_pos = slot.pos;
        let mut cost = 0;
        let speed = slot.base.life().speed;

        for _ in 0..speed {
            let mut best_dir = None;
            let mut max_dist = curr_pos.distance_to(target_pos);

            for dir in DIRECTION_ARRAY {
                let candidate = curr_pos + dir;
                if !world.map.is_walkable(candidate) { continue; }
                let d = candidate.distance_to(target_pos);
                if d > max_dist {
                    max_dist = d;
                    best_dir = Some(dir);
                }
            }

            match best_dir {
                Some(dir) if slot.base.life().can_move_for(cost + 1) => {
                    curr_pos = curr_pos + dir;
                    cost += 1;
                }
                _ => break,
            }
        }
        (curr_pos, cost)
    }

    fn pick_random_neighbor(world: &World, pos: Position) -> Option<Position> {
        let dirs = DIRECTION_ARRAY;
        for _ in 0..8 {
            let d = dirs[gen_range(0, 7) as usize];
            let p = pos + d;
            if world.map.is_walkable(p) { return Some(p); }
        }
        None
    }
}
