pub mod cell;
pub mod direction;
pub mod position;

use crate::map::{cell::Cell, position::Position};

/// ===============================
/// MAP
/// ===============================
///
/// Map:
/// - Dünyanın çevresel durumunu tutar
/// - Entity bilgisi tutmaz
/// - Sadece "burada ne var?" sorusuna cevap verir
///
/// Entity çakışmaları, canlı/ceset kontrolü World seviyesinde yapılır.
pub struct Map {
    pub width: usize,
    pub height: usize,
    pub grid: Vec<Cell>,
}

impl Map {
    /// ===============================
    /// KONUM YARDIMCILARI
    /// ===============================

    /// Bu pozisyon harita sınırları içinde mi?
    pub fn in_bounds(&self, pos: Position) -> bool {
        pos.x < self.width && pos.y < self.height
    }

    /// Pozisyondan index üret
    fn index_of(&self, pos: Position) -> Option<usize> {
        if self.in_bounds(pos) {
            Some(pos.y * self.width + pos.x)
        } else {
            None
        }
    }

    /// ===============================
    /// OKUMA
    /// ===============================

    /// Burada ne var?
    pub fn cell(&self, pos: Position) -> Option<&Cell> {
        self.index_of(pos).map(|i| &self.grid[i])
    }

    /// Buradaki şey bu mu?
    pub fn is_cell(&self, pos: Position, expected: &Cell) -> bool {
        self.cell(pos).map(|c| c == expected).unwrap_or(false)
    }

    /// Buraya hareket edilebilir mi?
    ///
    /// Şimdilik:
    /// - Empty -> evet
    /// - Food / Water -> evet
    ///
    /// Entity kontrolü burada yapılmaz.
    pub fn is_walkable(&self, pos: Position) -> bool {
        matches!(
            self.cell(pos),
            Some(Cell::Empty | Cell::Food { .. } | Cell::Water { .. })
        )
    }

    /// ===============================
    /// YAZMA
    /// ===============================
    /// ⚠️ Map mutable ama "kontrollü" değişir
    /// Entity logic buraya gömülmez

    /// Konuma yeni bir şey yerleştir
    ///
    /// Örnek:
    /// - Food eklemek
    /// - Ceset bırakmak
    pub fn set_cell(&mut self, pos: Position, cell: Cell) -> bool {
        if let Some(i) = self.index_of(pos) {
            self.grid[i] = cell;
            true
        } else {
            false
        }
    }

    /// Konumdaki miktarı azalt
    ///
    /// amount kadar düşer,
    /// 0 veya altına inerse Empty olur
    pub fn reduce_cell_amount(&mut self, pos: Position, amount: usize) -> bool {
        if let Some(i) = self.index_of(pos) {
            match &mut self.grid[i] {
                Cell::Food { amount: a } | Cell::Water { amount: a } => {
                    *a = a.saturating_sub(amount);
                    if *a == 0 {
                        self.grid[i] = Cell::Empty;
                    }
                    true
                }
                _ => false,
            }
        } else {
            false
        }
    }

    /// Konumdaki şeyi tamamen sil
    pub fn clear_cell(&mut self, pos: Position) -> bool {
        self.set_cell(pos, Cell::Empty)
    }
}
