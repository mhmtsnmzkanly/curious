#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum Cell {
    Empty,
    Wall,
    Burrow,
    Food { amount: usize },
    Water { amount: usize },
}
