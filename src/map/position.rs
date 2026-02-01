use std::ops::Add;

use crate::map::direction::Direction;

/// - Dünya koordinatıdır (chunk bağımsız)
/// - Negatif koordinatları destekler
/// - (0,0) merkezli dünya için uygundur
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    pub x: isize,
    pub y: isize,
}

impl Position {
    /// Yeni bir pozisyon oluştur
    pub fn new(x: isize, y: isize) -> Self {
        Self { x, y }
    }

    /// Pozisyonu doğrudan güncelle
    pub fn set(&mut self, other: Position) {
        self.x = other.x;
        self.y = other.y;
    }

    /// Manhattan mesafesi
    /// Çapraz yönler olsa bile karar mekanizması için hâlâ en stabil metriktir
    pub fn distance_to(&self, other: Position) -> usize {
        ((self.x - other.x).abs() + (self.y - other.y).abs()) as usize
    }

    /// Yön bazlı yeni pozisyon (immutable)
    pub fn offset(&self, dir: Direction) -> Position {
        *self + dir
    }
}

impl From<(isize, isize)> for Position {
    fn from(t: (isize, isize)) -> Position {
        Position { x: t.0, y: t.1 }
    }
}

/// Position + Direction → Position
///
/// Çapraz hareketler desteklenir.
/// World isterse çaprazı yasaklayabilir (Map / validation katmanı).
impl Add<Direction> for Position {
    type Output = Position;

    fn add(self, dir: Direction) -> Position {
        match dir {
            Direction::Up => Position {
                x: self.x,
                y: self.y - 1,
            },
            Direction::Down => Position {
                x: self.x,
                y: self.y + 1,
            },
            Direction::Left => Position {
                x: self.x - 1,
                y: self.y,
            },
            Direction::Right => Position {
                x: self.x + 1,
                y: self.y,
            },
            Direction::UpLeft => Position {
                x: self.x - 1,
                y: self.y - 1,
            },
            Direction::UpRight => Position {
                x: self.x + 1,
                y: self.y - 1,
            },
            Direction::DownLeft => Position {
                x: self.x - 1,
                y: self.y + 1,
            },
            Direction::DownRight => Position {
                x: self.x + 1,
                y: self.y + 1,
            },
        }
    }
}
