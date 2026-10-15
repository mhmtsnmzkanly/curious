//! Reproducible random stream owned by one world, never shared between worlds.
#[derive(Debug, Clone)]
pub struct Random {
    state: u64,
}
impl Random {
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }
    pub fn state(&self) -> u64 {
        self.state
    }
    pub fn next_u64(&mut self) -> u64 {
        let previous = self.state;
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1);
        previous
    }
    pub fn range(&mut self, min: isize, max: isize) -> isize {
        assert!(min <= max);
        let width = (max - min) as u64;
        if width == 0 {
            min
        } else {
            min + (self.next_u64() % (width + 1)) as isize
        }
    }
}
