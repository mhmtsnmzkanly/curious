use std::ops::{Add, AddAssign};

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
}

impl From<(isize, isize)> for Position {
    fn from(t: (isize, isize)) -> Position {
        Position { x: t.0, y: t.1 }
    }
}

#[derive(Hash, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
    UpLeft,
    UpRight,
    DownLeft,
    DownRight,
}

pub const DIRECTION_ARRAY: [Direction; 8] = [
    Direction::Down,
    Direction::Up,
    Direction::Left,
    Direction::Right,
    Direction::UpLeft,
    Direction::UpRight,
    Direction::DownLeft,
    Direction::DownRight,
];
/// Hareket etme talimat dizisi
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Steps(pub Vec<Direction>);

impl Steps {
    /// Boş bir hareket dizisi oluşturur
    pub fn empty() -> Steps {
        Steps(Vec::new())
    }

    /// Girdiyi Hareket talimatı yapısına ekler
    pub fn new(value: Vec<Direction>) -> Steps {
        Steps(value)
    }

    /// Listenin başından bir eleman al
    /// Eğer boşsa None döner
    pub fn pop_front(&mut self) -> Option<Direction> {
        if self.0.is_empty() {
            None
        } else {
            Some(self.0.remove(0))
        }
    }

    /// Listenin başındaki elemanı gör ama silme
    pub fn peek_front(&self) -> Option<&Direction> {
        self.0.first()
    }

    /// Vec<Direction> ekle
    pub fn extend(&mut self, other: Steps) {
        self.0.extend(other.0);
    }

    /// Iterator ile erişim
    pub fn iter(&self) -> std::slice::Iter<'_, Direction> {
        self.0.iter()
    }

    /// Mutable iterator
    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, Direction> {
        self.0.iter_mut()
    }
    /// Adım sayısını döner
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Boş mu kontrol
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// IntoIterator implementasyonu (for x in steps)
impl IntoIterator for Steps {
    type Item = Direction;
    type IntoIter = std::vec::IntoIter<Direction>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

/// &Steps için iterator
impl<'a> IntoIterator for &'a Steps {
    type Item = &'a Direction;
    type IntoIter = std::slice::Iter<'a, Direction>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

/// &mut Steps için iterator
impl<'a> IntoIterator for &'a mut Steps {
    type Item = &'a mut Direction;
    type IntoIter = std::slice::IterMut<'a, Direction>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}

// + Direction -> yeni Steps
impl Add<Direction> for Steps {
    type Output = Steps;

    fn add(mut self, rhs: Direction) -> Steps {
        self.0.push(rhs);
        self
    }
}

// + Vec<Direction> -> yeni Steps
impl Add<Vec<Direction>> for Steps {
    type Output = Steps;

    fn add(mut self, rhs: Vec<Direction>) -> Steps {
        self.0.extend(rhs);
        self
    }
}

/// `Steps += Direction` ile sonuna ekleme
impl AddAssign<Direction> for Steps {
    fn add_assign(&mut self, rhs: Direction) {
        self.0.push(rhs);
    }
}

/// `Steps += Vec<Direction>` ile birden fazla ekleme
impl AddAssign<Vec<Direction>> for Steps {
    fn add_assign(&mut self, rhs: Vec<Direction>) {
        self.0.extend(rhs);
    }
}

/// `Steps += Steps` ile birden fazla ekleme
impl AddAssign<Steps> for Steps {
    fn add_assign(&mut self, rhs: Steps) {
        self.0.extend(rhs);
    }
}

/// Vec<Direction> -> Steps
impl From<Vec<Direction>> for Steps {
    fn from(vec: Vec<Direction>) -> Steps {
        Steps(vec)
    }
}

/// Steps -> Vec<Direction>
impl From<Steps> for Vec<Direction> {
    fn from(steps: Steps) -> Vec<Direction> {
        steps.0
    }
}

/// Position + Direction → Position
///
/// Çapraz hareketler desteklenir.
/// World isterse çaprazı yasaklayabilir (Map / validation katmanı).
impl Add<Direction> for Position {
    type Output = Position;

    /// Yön bazlı yeni pozisyon (immutable)
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
