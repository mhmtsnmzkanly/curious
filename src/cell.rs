#[derive(Debug, Clone)]
pub enum Cell {
    Empty,
    Food { amount: u32 },
    Water { amount: u32 },
}
