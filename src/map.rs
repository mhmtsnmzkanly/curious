use crate::{cell::Cell, position::Position};

pub struct Map {
    pub width: usize,
    pub height: usize,
    pub grid: Vec<Cell>,
}

impl Map {
    fn index_of(&self, pos: Position) -> Option<usize> {
        if pos.x < self.width && pos.y < self.height {
            Some(pos.y * self.width + pos.x)
        } else {
            None
        }
    }

    fn get(&self, pos: Position) -> Option<&Cell> {
        self.index_of(pos).map(|i| &self.grid[i])
    }

    fn get_mut(&mut self, pos: Position) -> Option<&mut Cell> {
        self.index_of(pos).map(move |i| &mut self.grid[i])
    }
}
