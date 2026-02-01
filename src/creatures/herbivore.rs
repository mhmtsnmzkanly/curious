use crate::entity::{
    Entity, intent::Intent, lifestate::LifeState, perception::Perception, species::Species,
};

pub struct HerbivoreEntity {
    pub life_state: LifeState,
}

impl HerbivoreEntity {
    pub fn new(life_state: LifeState) -> Self {
        Self { life_state }
    }

    pub fn default() -> Self {
        Self {
            life_state: LifeState {
                max_age: 105,
                max_health: 120,
                max_energy: 80,
                maturity_age: 20,
                vision_range: 6,
                age: 0,
                health: 120,
                energy: 80,
                reproduction_cooldown: 0,
                speed: 3,
                moves_used: 0,
            },
        }
    }
}

impl Entity for HerbivoreEntity {
    fn life(&self) -> &LifeState {
        &self.life_state
    }

    fn life_mut(&mut self) -> &mut LifeState {
        &mut self.life_state
    }

    fn species(&self) -> Species {
        Species::Herbivore
    }

    fn make_intent(&self, perception: Perception) -> Intent {
        // ===============================
        // 1. Yakınında yiyecek var mı?
        // ===============================
        if perception.foods.is_empty() {
            // Yiyecek yok → ara
            // Basit: rastgele bir yön seç
            use crate::map::direction::Direction::*;
            let dirs = [Up, Down, Left, Right];
            let dir = dirs[crate::gen_range(0, dirs.len() as isize - 1) as usize];
            Intent::Move { steps: vec![dir] }
        } else {
            // Yiyecek var → yemeyi planla
            let nearest_food = &perception.foods[0]; // Basit: ilk bulduğu yiyecek
            // Eğer tok değilse ye
            if !self.life_state.is_energy_full() {
                Intent::Eat {
                    at: vec![nearest_food.steps.0[0]],
                    corpse_id: None,
                }
            } else if self.life_state.can_reproduce() {
                // Tok ve üreme zamanı → eş ara
                if let Some(target) = perception
                    .entities
                    .iter()
                    .find(|e| e.species == Species::Herbivore)
                {
                    Intent::Mate {
                        target_id: target.id,
                    }
                } else {
                    // Eş yoksa yakındaki bir yere hareket et
                    Intent::Idle { duration: 1 }
                }
            } else {
                // Tok ama üreme zamanı değil → bekle
                Intent::Idle { duration: 1 }
            }
        }
    }

    fn tick(&mut self) {
        self.life_state.tick();
    }

    fn reproduce(&self) -> Box<dyn Entity> {
        let mut child_life = self.life_state.clone();
        child_life.age = 0;
        child_life.energy = child_life.max_energy / 2;
        child_life.health = child_life.max_health / 2;
        Box::new(HerbivoreEntity::new(child_life))
    }
}
