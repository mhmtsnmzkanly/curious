use crate::map::direction::Direction;

#[derive(Debug, Clone, Copy, PartialEq, Eq, std::hash::Hash)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

impl Position {
    /// x ve y değerlerinden yeni bir değer oluştur
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
    /// Pozisyonu doğrudan güncellemek için
    pub fn set(&mut self, other: Position) {
        self.x = other.x;
        self.y = other.y;
    }

    /// Manhattan mesafesini hesaplar
    pub fn distance_to(&self, other: Position) -> usize {
        ((self.x as isize - other.x as isize).abs() + (self.y as isize - other.y as isize).abs())
            as usize
    }
}

impl std::ops::Add<Direction> for Position {
    type Output = Position;

    fn add(self, dir: Direction) -> Position {
        match dir {
            Direction::Up => Position {
                x: self.x,
                y: self.y.saturating_sub(1),
            },
            Direction::Down => Position {
                x: self.x,
                y: self.y + 1,
            },
            Direction::Left => Position {
                x: self.x.saturating_sub(1),
                y: self.y,
            },
            Direction::Right => Position {
                x: self.x + 1,
                y: self.y,
            },
        }
    }
}
