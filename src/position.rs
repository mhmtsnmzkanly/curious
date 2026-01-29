#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

impl Position {
    fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }

    fn offset(&self, dx: i32, dy: i32) -> Option<Self> {
        let nx = self.x as i32 + dx;
        let ny = self.y as i32 + dy;

        if nx < 0 || ny < 0 {
            None
        } else {
            Some(Self {
                x: nx as usize,
                y: ny as usize,
            })
        }
    }
}
