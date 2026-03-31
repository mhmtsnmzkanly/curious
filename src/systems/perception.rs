use crate::{
    world::{World, EntitySlot},
    entity::perception::Perception,
};

/// Canlıların çevrelerini algılamasını sağlayan sistem.
pub struct PerceptionSystem;

impl PerceptionSystem {
    /// Bir canlı (EntitySlot) için çevresindeki verileri tarar ve Perception objesi üretir.
    pub fn build(world: &World, current_slot: &EntitySlot) -> Perception {
        let mut perception = Perception::empty();
        let radius = current_slot.base.life().vision_range;

        // 1. Yakındaki Yiyecekleri Tara
        let found_foods = world.map.scan_foods_within(current_slot.pos, radius);
        for (_f_pos, steps, amount) in found_foods {
            perception.add_food(amount, false, steps);
        }

        // 1.1 Yakındaki Suları Tara
        let found_waters = world.map.scan_waters_within(current_slot.pos, radius);
        for (_w_pos, steps, amount) in found_waters {
            perception.add_water(amount, steps);
        }

        // 2. Yakındaki Diğer Canlıları Tara
        for other in &world.entities {
            if other.id == current_slot.id { continue; }
            let dist = current_slot.pos.distance_to(other.pos);
            if dist <= radius {
                if let Some(steps) = world.map.bfs_steps_to(current_slot.pos, other.pos, radius) {
                    let other_life = other.entity().life();
                    let power = other_life.health + other_life.energy;
                    perception.add_entity(other.id, other.entity().species(), power, steps);
                }
            }
        }

        // 3. Çevredeki yürünebilir alanları Tara
        let walkable_map = world.map.walkable_distances(current_slot.pos);
        for (dir, dist) in walkable_map {
            perception.add_direction(dir, dist);
        }

        perception
    }
}
