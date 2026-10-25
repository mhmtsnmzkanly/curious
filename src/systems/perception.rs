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
        Self::with_index(
            world,
            current,
            &super::spatial::SpatialIndex::new(&world.entities),
        )
    }
    pub fn perceive_all(world: &World) -> Vec<Perception> {
        let index = super::spatial::SpatialIndex::new(&world.entities);
        world
            .entities
            .iter()
            .map(|slot| Self::with_index(world, slot, &index))
            .collect()
    }
    pub fn with_index(
        world: &World,
        current: &EntitySlot,
        index: &super::spatial::SpatialIndex,
    ) -> Perception {
        let base_vision = current.base.life().vision_range;
        let radius = if let Some(cycle) = world.environment.day_night {
            cycle.effective_vision_range(world.tick_counter, base_vision)
        } else {
            base_vision
        };
        let nearby = index.nearby(current.pos, radius);
        let blocked: std::collections::HashSet<_> = nearby
            .iter()
            .map(|i| &world.entities[*i])
            .filter(|s| {
                s.id != current.id
                    && matches!(s.phase, EntityPhase::Active | EntityPhase::Sleeping { .. })
            })
            .map(|s| s.pos)
            .collect();
        let paths =
            world
                .map
                .paths_within_for(current.pos, radius, &blocked, current.base.can_burrow());
        let mut positions: Vec<_> = paths.keys().copied().collect();
        positions.sort_by_key(|p| (p.y, p.x));
        let mut view = Perception::empty();
        for pos in positions {
            if matches!(world.map.cell(pos), Some(crate::map::cell::Cell::Burrow)) {
                view.shelters.push(paths[&pos].clone());
            }
            view.cells.push((
                pos,
                world
                    .map
                    .cell(pos)
                    .cloned()
                    .unwrap_or(crate::map::cell::Cell::Empty),
            ));
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
        for other in nearby.iter().map(|i| &world.entities[*i]) {
            if !other.base.can_burrow()
                && current.base.can_burrow()
                && matches!(
                    world.map.cell(current.pos),
                    Some(crate::map::cell::Cell::Burrow)
                )
                && other.base.species().hunts(&current.base.species())
            {
                continue;
            }
            if other.id == current.id {
                continue;
            }
            // Eating stops beside a resource. Requiring the corpse's own
            // cell to be walkable hides meals the resolver can already eat.
            // Extend only a locally reachable, unoccupied approach by its last
            // interaction step; this never opens a path through the corpse cell.
            let steps = paths.get(&other.pos).cloned().or_else(|| {
                if !matches!(other.phase, EntityPhase::Corpse { .. })
                    || other.corpse_energy == 0
                    || current.pos.distance_to(other.pos) > radius
                {
                    return None;
                }
                DIRECTION_ARRAY
                    .into_iter()
                    .filter_map(|dir| {
                        let approach = other.pos + dir;
                        if approach != current.pos && blocked.contains(&approach) {
                            return None;
                        }
                        let path = paths.get(&approach)?;
                        if path.len() >= radius {
                            return None;
                        }
                        let last = DIRECTION_ARRAY
                            .into_iter()
                            .find(|d| approach + *d == other.pos)?;
                        let mut steps = path.clone();
                        steps.0.push(last);
                        Some((steps, approach))
                    })
                    .min_by_key(|(steps, p)| (steps.len(), p.y, p.x))
                    .map(|(steps, _)| steps)
            });
            let Some(steps) = steps else {
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
                if world.policy.ambush_predation_enabled
                    && is_in_ambush_cover(&world.map, other.pos)
                    && life.stamina == life.max_stamina
                {
                    let max_detect = radius.saturating_sub(1).max(1);
                    if current.pos.distance_to(other.pos) > max_detect || steps.len() > max_detect {
                        continue;
                    }
                }
                view.add_entity(
                    other.id,
                    other.base.species(),
                    life.health + life.energy,
                    steps.clone(),
                    other.phase.is_active() && life.can_reproduce(),
                );
                view.entities.last_mut().unwrap().health = Some(life.health);
                view.entities.last_mut().unwrap().mature = life.is_mature();
            }
        }
        for dir in DIRECTION_ARRAY {
            let mut p = current.pos;
            let mut distance = 0;
            for _ in 0..radius.min(255) {
                let next = p + dir;
                if !world.map.can_step_for(p, next, current.base.can_burrow())
                    || blocked.contains(&next)
                {
                    break;
                }
                p = next;
                distance += 1;
            }
            view.add_direction(dir, distance);
        }
        if world.environment.signals_enabled {
            view.signals = world
                .environment
                .perceive_signals_at(current.pos, world.tick_counter);
        }
        view
    }
}

pub fn is_in_ambush_cover(map: &crate::map::Map, pos: crate::map::movement::Position) -> bool {
    matches!(
        map.cell(pos),
        Some(crate::map::cell::Cell::Burrow | crate::map::cell::Cell::Food { .. })
    ) || crate::map::movement::DIRECTION_ARRAY.into_iter().any(|dir| {
        matches!(
            map.cell(pos + dir),
            Some(
                crate::map::cell::Cell::Burrow
                    | crate::map::cell::Cell::Wall
                    | crate::map::cell::Cell::Food { .. }
            )
        )
    })
}
