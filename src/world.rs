use crate::{entity::{Entity,action::Action},  map::Map};

pub struct WorldView<'a> {
    pub map: &'a Map,
}

pub struct World {
    pub map: Map,
    pub entities: Vec<Box<dyn Entity>>,
}

impl World {
    pub fn tick(&mut self) {
        let view = WorldView { map: &self.map };

        let actions: Vec<(usize, Action)> = self
            .entities
            .iter()
            .map(|e| (e.id(), e.think(&view)))
            .collect();

        for (id, action) in actions {
            if let Some(entity) = self.entities.iter_mut().find(|e| e.id() == id) {
                entity.apply(action);
            }
        }
    }
}
