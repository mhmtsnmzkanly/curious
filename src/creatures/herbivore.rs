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
                max_age: 100,
                maturity_age: 10,
                max_health: 50,
                max_energy: 50,
                low_energy_threshold: 20,
                age: 0,
                health: 50,
                energy: 30,
                reproduction_cooldown: 0,
                speed: 1,
                moves_used: 0,
            },
            phase: EntityPhase::Active,
        }
    }

    fn move_towards(&self, target: Position) -> Action {
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
        // 1. Eğer tam yemeğin üzerindeyse YE
        if let Some(Cell::Food { .. }) = ctx.cell(self.pos) {
            if self.life.is_energy_low() {
                return Action::Eat;
            }
        }

        // 2. Acıkınca çevreyi tara (Manhattan Mesafe: 3)
        if self.life.is_energy_low() {
            // Basit bir tarama: Yakınlardaki hücrelere bak
            for dx in -3..=3 {
                for dy in -3..=3 {
                    let target_pos = Position {
                        x: (self.pos.x as isize + dx).max(0) as usize,
                        y: (self.pos.y as isize + dy).max(0) as usize,
                    };

                    if ctx.in_bounds(target_pos) {
                        if let Some(Cell::Food { .. }) = ctx.cell(target_pos) {
                            return self.move_towards(target_pos);
                        }
                    }
                }
            }
        }

        // 3. Hiçbir şey yoksa rastgele hareket
        let seed = self.id + self.pos.x + self.life.age;
        match seed % 4 {
            0 => Action::Move(Direction::Up),
            1 => Action::Move(Direction::Down),
            2 => Action::Move(Direction::Left),
            _ => Action::Move(Direction::Right),
        }
    }

    fn apply(&mut self, _action: Action) {}
}
