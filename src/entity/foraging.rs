//! Local acquisition value per occupied tick; no future resource knowledge.
use super::lifestate::LifeState;
pub fn value(amount: usize, distance: usize, life: &LifeState, water: bool) -> isize {
    let travel = distance.saturating_sub(1);
    let ticks = travel.div_ceil(life.speed.max(1)).max(1);
    let capacity = if water {
        life.max_water.saturating_sub(life.water)
    } else {
        life.max_energy.saturating_sub(life.energy)
    };
    let meal = amount.min(capacity).min(if water {
        crate::rules::biology::DRINK_WATER_AMOUNT
    } else {
        life.max_meal
    });
    // Water and energy are distinct currencies. Keep water's existing double
    // weight; charge real movement energy and the time spent acquiring either.
    let benefit = meal as isize * if water { 2 } else { 1 };
    let cost = travel
        .saturating_mul(life.move_energy_cost)
        .saturating_add(ticks);
    (benefit - cost as isize) * 100 / ticks as isize
}
