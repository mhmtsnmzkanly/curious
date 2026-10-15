#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum Cell {
    Empty,
    Food { amount: usize },
    Water { amount: usize },
}
