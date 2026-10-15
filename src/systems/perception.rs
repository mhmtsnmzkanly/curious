use crate::{
    entity::perception::{PerceivedFood, Perception},
    entity::phase::EntityPhase,
    map::movement::DIRECTION_ARRAY,
    world::{EntitySlot, World},
};

/// Bounded local perception; one traversal supplies all resource/entity paths.
pub struct PerceptionSystem;
impl PerceptionSystem {
    pub fn build(world: &World, current: &EntitySlot) -> Perception {
        let radius = current.base.life().vision_range;
        let blocked: std::collections::HashSet<_> = world
            .entities
            .iter()
            .filter(|s| {
                s.id != current.id
                    && matches!(s.phase, EntityPhase::Active | EntityPhase::Sleeping { .. })
            })
            .map(|s| s.pos)
            .collect();
        let paths = world.map.paths_within(current.pos, radius, &blocked);
        let mut positions: Vec<_> = paths.keys().copied().collect();
        positions.sort_by_key(|p| (p.y, p.x));
        let mut view = Perception::empty();
        for pos in positions {
            view.cells.push((pos,world.map.cell(pos).cloned().unwrap_or(crate::map::cell::Cell::Empty)));
            match world.map.cell(pos) {
                Some(crate::map::cell::Cell::Food { amount }) if *amount > 0 => {
                    view.add_food(*amount, false, paths[&pos].clone())
                }
                Some(crate::map::cell::Cell::Water { amount }) if *amount > 0 => {
                    view.add_water(*amount, paths[&pos].clone())
                }
                _ => {}
            }
        }
        for other in &world.entities {
            if other.id == current.id {
                continue;
            }
            let Some(steps) = paths.get(&other.pos) else {
                continue;
            };
            if matches!(other.phase, EntityPhase::Corpse { .. }) && other.corpse_energy > 0 {
                view.foods.push(PerceivedFood {
                    corpse_id: Some(other.id),
                    amount: other.corpse_energy,
                    is_corpse: true,
                    steps: steps.clone(),
                });
            } else if matches!(
                other.phase,
                EntityPhase::Active | EntityPhase::Sleeping { .. }
            ) && other.base.life().is_alive()
            {
                let life = other.base.life();
                view.add_entity(
                    other.id,
                    other.base.species(),
                    life.health + life.energy,
                    steps.clone(),
                    other.phase.is_active() && life.can_reproduce(),
                );
                view.entities.last_mut().unwrap().health=Some(life.health);
            }
        }
        for dir in DIRECTION_ARRAY {
            let mut p = current.pos;
            let mut distance = 0;
            for _ in 0..radius.min(255) {
                let next = p + dir;
                if !world.map.can_step(p,next) || blocked.contains(&next) {
                    break;
                }
                p = next;
                distance += 1;
            }
            view.add_direction(dir, distance);
        }
        view
    }
}
