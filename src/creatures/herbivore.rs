use crate::{
    entity::{Entity, action::Action, lifestate::LifeState, phase::EntityPhase},
    map::{cell::Cell, direction::Direction, position::Position},
    world::WorldView,
};

pub struct Herbivore {
    pub id: usize,
    pub pos: Position,
    pub life: LifeState,
    pub phase: EntityPhase,
}

impl Herbivore {
    pub fn new(id: usize, pos: Position) -> Self {
        Self {
            id,
            pos,
            life: LifeState {
                max_age: 500,
                maturity_age: 10,
                max_health: 50,
                max_energy: 50,
                low_energy_threshold: 20,
                age: 0,
                health: 50,
                energy: 500,
                reproduction_cooldown: 0,
                speed: 1,
                moves_used: 0,
            },
            phase: EntityPhase::Active,
        }
    }

    pub fn move_towards(&self, target: Position) -> Action {
        if target.x > self.pos.x {
            Action::Move(Direction::Right)
        } else if target.x < self.pos.x {
            Action::Move(Direction::Left)
        } else if target.y > self.pos.y {
            Action::Move(Direction::Down)
        } else {
            Action::Move(Direction::Up)
        }
    }

    pub fn random_move(&self) -> Action {
        let seed = self.id + self.pos.x + self.pos.y + self.life.age;
        match seed % 4 {
            0 => Action::Move(Direction::Up),
            1 => Action::Move(Direction::Down),
            2 => Action::Move(Direction::Left),
            _ => Action::Move(Direction::Right),
        }
    }
}

impl Entity for Herbivore {
    fn id(&self) -> usize {
        self.id
    }
    fn position(&self) -> Position {
        self.pos
    }
    fn position_mut(&mut self) -> &mut Position {
        &mut self.pos
    }
    fn life(&self) -> &LifeState {
        &self.life
    }
    fn life_mut(&mut self) -> &mut LifeState {
        &mut self.life
    }
    fn phase(&self) -> EntityPhase {
        self.phase
    }
    fn phase_mut(&mut self) -> &mut EntityPhase {
        &mut self.phase
    }

    fn think(&self, ctx: &WorldView) -> Action {
        let l = self.life();

        // 1. ÖNCELİK: HAYATTA KALMA (Açlık Kontrolü)
        if l.is_energy_low() {
            // ... Mevcut yemek arama kodların ...
            // Eğer yakında yemek varsa Action::Eat veya Action::Move döndür
        }

        // 2. ÖNCELİK: ÜREME (Aç değilse ve üreyebiliyorsa)
        if l.can_reproduce() {
            // Çevredeki diğer canlıları tara (Mesafe: 4)
            let nearby = ctx.nearby_entities(self.pos, 4);

            for (other_pos, other_id) in nearby {
                if other_id != self.id {
                    // Kendisi değilse
                    if other_pos == self.pos {
                        // Aynı karedeysek: Çiftleşme teklif et!
                        return Action::Mate {
                            target_id: other_id,
                        };
                    } else {
                        // Yakındaysa: Ona doğru yürü!
                        return self.move_towards(other_pos);
                    }
                }
            }
        }

        // 3. ÖNCELİK: RASTGELE GEZİNTİ
        self.random_move()
    }

    fn reproduce(&self, new_id: usize, pos: Position) -> Box<dyn Entity> {
        // Herbivore, kendisinden bir tane daha Herbivore yaratır.
        // İleride buraya genetik aktarım da eklenebilir.
        Box::new(Herbivore::new(new_id, pos))
    }

    fn apply(&mut self, _action: Action) {}
}
