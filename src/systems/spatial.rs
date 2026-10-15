//! A per-tick read-only index. It owns indices, never references or biological state.
use crate::{map::movement::Position, world::EntitySlot};
use std::collections::HashMap;
pub struct SpatialIndex {
    at: HashMap<Position, Vec<usize>>,
    linear: Option<usize>,
}
impl SpatialIndex {
    pub fn new(entities: &[EntitySlot]) -> Self {
        if entities.len() < 256 {
            return Self {
                at: HashMap::new(),
                linear: Some(entities.len()),
            };
        }
        let mut at: HashMap<Position, Vec<usize>> = HashMap::new();
        for (i, e) in entities.iter().enumerate() {
            at.entry(e.pos).or_default().push(i);
        }
        Self { at, linear: None }
    }
    pub fn nearby(&self, origin: Position, radius: usize) -> Vec<usize> {
        if let Some(len) = self.linear {
            return (0..len).collect();
        }
        let r = radius as isize;
        let mut found = Vec::new();
        for y in origin.y - r..=origin.y + r {
            for x in origin.x - r..=origin.x + r {
                let p = Position::new(x, y);
                if origin.distance_to(p) <= radius {
                    if let Some(ids) = self.at.get(&p) {
                        found.extend(ids);
                    }
                }
            }
        }
        // Preserve entity ordering: policy tie-breaking must not depend on HashMap order.
        found.sort_unstable();
        found
    }
}
