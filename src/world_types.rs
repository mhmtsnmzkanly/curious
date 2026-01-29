#[derive(Debug, Clone)]
pub enum Cell {
    Empty,
    Food { amount: u32 },
    Water { amount: u32 },
}

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

impl Position {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }

    /// Negatif yön kullanmadan hareket:
    /// Taşma olursa None döner (harita sınırı kontrolü dışarıda yapılır)
    pub fn move_dir(&self, dir: Direction, amount: usize) -> Option<Self> {
        match dir {
            Direction::Up => self.y.checked_sub(amount).map(|y| Self { x: self.x, y }),
            Direction::Down => Some(Self {
                x: self.x,
                y: self.y + amount,
            }),
            Direction::Left => self.x.checked_sub(amount).map(|x| Self { x, y: self.y }),
            Direction::Right => Some(Self {
                x: self.x + amount,
                y: self.y,
            }),
        }
    }
}

pub enum Action {
    Move(Direction),
    Eat,
    Attack { target_id: usize },
    Flee(Direction),
    Idle,
}
