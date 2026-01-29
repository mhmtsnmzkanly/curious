#[derive(Debug, Clone)]
pub enum Action {
    Move { dx: i32, dy: i32 },
    Eat,
    Attack { target_id: usize },
    Idle,
}
